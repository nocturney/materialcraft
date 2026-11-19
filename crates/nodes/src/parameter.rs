use crate::NodeDefinition;
use materialcraft_core::Value;
use std::collections::BTreeMap;

/// How many slider spans the hard limit extends past each end of a parameter's range
const HARD_SPAN: f32 = 16.0;

/// Native channel controls also accept a single grayscale value, broadcast by the evaluator.
/// Other vector parameters keep their exact function output type.
pub fn accepts_scalar_function(definition: &str, parameter: &str) -> bool {
    matches!((definition, parameter), ("levels", "in_low" | "in_mid" | "in_high" | "out_low" | "out_high") | ("fx_map", "background"))
}

/// Clamp `v` to `[lo, hi]`; a non-finite `v` becomes `fallback`
fn clamp_finite(v: f32, lo: f32, hi: f32, fallback: f32) -> f32 {
    if v.is_finite() { v.clamp(lo, hi) } else { fallback }
}

pub fn clamp_params(definition: &NodeDefinition, params: &mut BTreeMap<String, Value>) {
    for parameter in &definition.params {
        let Some(value) = params.get_mut(parameter.id) else { continue };
        if let Value::Enum(index) = value {
            // Shuffle resolves out-of-range selectors against the connected image kinds
            if definition.id == "channels_shuffle" && matches!(parameter.id, "red" | "green" | "blue" | "alpha") {
                continue;
            }

            // An enum without options keeps its default
            *index = match (parameter.options.len().checked_sub(1), &parameter.default) {
                (Some(last), _) => (*index).min(u32::try_from(last).unwrap_or(u32::MAX)),
                (None, Value::Enum(default)) => *default,
                (None, _) => 0,
            };
            continue;
        }

        if let Value::Color(components) = value {
            for component in components {
                *component = clamp_finite(*component, -HARD_SPAN, 1.0 + HARD_SPAN, 0.0);
            }
            continue;
        }

        // An unconstrained Input Value accepts any number; reject only nonnumeric values
        let ranged = parameter.min < parameter.max;
        let span = parameter.max - parameter.min;
        let (lo, hi) = if ranged { (parameter.min - HARD_SPAN * span, parameter.max + HARD_SPAN * span) } else { (f32::MIN, f32::MAX) };

        // The default's component at `index`, or 0 for a different variant
        let fallback = |index: usize| match &parameter.default {
            Value::Float(default) => [*default].get(index).copied(),
            Value::Float2(default) => default.get(index).copied(),
            Value::Float3(default) => default.get(index).copied(),
            Value::Float4(default) => default.get(index).copied(),
            _ => None,
        };

        let clamp_at = |component: &mut f32, index: usize| *component = clamp_finite(*component, lo, hi, fallback(index).unwrap_or(0.0));
        let clamp_int = |component: &mut i32| *component = (*component as f32).clamp(lo, hi).round() as i32;
        match value {
            Value::Float(component) => clamp_at(component, 0),
            Value::Float2(components) => components.iter_mut().enumerate().for_each(|(index, component)| clamp_at(component, index)),
            Value::Float3(components) => components.iter_mut().enumerate().for_each(|(index, component)| clamp_at(component, index)),
            Value::Float4(components) => components.iter_mut().enumerate().for_each(|(index, component)| clamp_at(component, index)),
            Value::Int(component) if ranged => clamp_int(component),
            Value::Int2(components) if ranged => components.iter_mut().for_each(clamp_int),
            _ => {}
        }
    }
}

/// Parameter values for a node: catalog defaults overlaid with `overrides`
pub fn resolve_params(def: &NodeDefinition, overrides: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut m: BTreeMap<String, Value> = def.params.iter().map(|p| (p.id.to_string(), p.default.clone())).collect();
    for (k, v) in overrides {
        // Keep only known parameters whose type matches (a hand-edited file may have anything)
        if let Some(d) = m.get(k)
            && (d.type_name() == v.type_name() || (def.id == "input_value" && k == "default" && materialcraft_core::ValueType::of(v).is_some()))
        {
            m.insert(k.clone(), v.clone());
        }
    }
    clamp_params(def, &mut m);
    m
}

#[cfg(test)]
#[path = "parameter.test.rs"]
mod tests;
