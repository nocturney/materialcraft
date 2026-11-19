use super::{ATOMIC, box_filter};
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::Kind;

pub(super) fn definition() -> NodeDefinition {
    node("blur", "Blur", ATOMIC)
        .desc("Box blur with fractional pixel coverage.")
        .atomic()
        .any_in("input", "Input")
        .float("intensity", "Intensity", 10.0, 0.0, 32.0)
        .any_out()
        .cook(blur)
}

fn blur(context: &CookContext) -> CookResult {
    let input = context.any_or(0, Kind::Gray, [0.0; 4]);
    const BASE_RESOLUTION: f32 = 256.0;
    let radius = context.f("intensity").max(0.0) * context.w.min(context.h) as f32 / BASE_RESOLUTION;
    let tile = context.tiling_axes();

    Ok(vec![box_filter::axis(&box_filter::axis(&input, radius, true, tile[0]), radius, false, tile[1])])
}
