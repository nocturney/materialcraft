use materialcraft_core::ValueType;

/// Missing operands have their port's type, which can differ from the result.
pub(super) fn value_type(operation: &str, input_name: &str, inferred_type_code: u32, sibling_type_code: Option<u32>) -> ValueType {
    if matches!(input_name, "cond" | "condition") || matches!(operation, "and" | "or" | "not") {
        return ValueType::Boolean;
    }

    if operation == "dot" {
        return sibling_type_code
            .and_then(ValueType::from_code)
            .filter(|kind| matches!(kind, ValueType::Float | ValueType::Float2 | ValueType::Float3 | ValueType::Float4))
            .unwrap_or(ValueType::Float2);
    }

    if matches!(operation, "samplelum" | "samplecol" | "atan2") {
        return ValueType::Float2;
    }

    if operation == "cartesian" || (operation == "mulscalar" && input_name == "scalar") {
        return ValueType::Float;
    }

    let component_count = operation.chars().last().and_then(|character| character.to_digit(10)).unwrap_or(1);
    if operation.starts_with("tofloat") {
        return numeric_type(true, component_count);
    }

    if operation.starts_with("toint") {
        return numeric_type(false, component_count);
    }

    if operation.contains("swizzle") {
        return numeric_type(operation.starts_with('i'), 1);
    }

    if operation.contains("vector") {
        let sibling_width = sibling_type_code.and_then(ValueType::from_code).map(|kind| match kind {
            ValueType::Integer2 | ValueType::Float2 => 2,
            ValueType::Integer3 | ValueType::Float3 => 3,
            ValueType::Integer4 | ValueType::Float4 => 4,
            _ => 1,
        });
        let width = sibling_width
            .map_or(if input_name == "componentslast" { 1 } else { component_count.saturating_sub(1) }, |width| component_count.saturating_sub(width));
        return numeric_type(operation.starts_with('i'), width);
    }

    ValueType::from_code(inferred_type_code).unwrap_or(ValueType::Float)
}

fn numeric_type(is_integer: bool, component_count: u32) -> ValueType {
    match (is_integer, component_count) {
        (true, 2) => ValueType::Integer2,
        (true, 3) => ValueType::Integer3,
        (true, 4) => ValueType::Integer4,
        (true, _) => ValueType::Integer,
        (false, 2) => ValueType::Float2,
        (false, 3) => ValueType::Float3,
        (false, 4) => ValueType::Float4,
        (false, _) => ValueType::Float,
    }
}
