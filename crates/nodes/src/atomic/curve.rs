use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::Kind;

pub(super) fn definition() -> NodeDefinition {
    node("curve", "Curve", ATOMIC).desc("Remaps values through a tone curve.").atomic().any_in("input", "Input").curve("curve", "Curve").any_out().cook(curve)
}

fn curve(context: &CookContext) -> CookResult {
    let image = context.any_or(0, Kind::Gray, [0.0; 4]);
    let curve = context.curve("curve");

    // Linear interpolation over 1024 intervals approximates the smooth curve
    const TABLE_INTERVALS: usize = 1024;
    let table: Vec<f32> = (0..=TABLE_INTERVALS).map(|i| curve.eval(i as f32 / TABLE_INTERVALS as f32)).collect();
    let lookup = |value: f32| {
        let position = value.clamp(0.0, 1.0) * TABLE_INTERVALS as f32;
        let i = position as usize;
        let a = table.get(i).copied().unwrap_or(0.0);
        let b = table.get(i + 1).copied().unwrap_or(a);
        a + (b - a) * (position - i as f32)
    };
    Ok(vec![image.map(image.kind(), |p| [lookup(p[0]), lookup(p[1]), lookup(p[2]), p[3]])])
}
