use super::{ATOMIC, direction};
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("directional_warp", "Directional warp", ATOMIC)
        .desc("Offsets the input along a direction, by an intensity map.")
        .atomic()
        .any_in("input", "Input")
        .gray_in("intensity_input", "Intensity input")
        .float("intensity", "Intensity", 10.0, 0.0, 64.0)
        .angle("angle", "Warp angle", 0.0)
        .any_out()
        .cook(directional_warp)
}

fn directional_warp(context: &CookContext) -> CookResult {
    let image = context.any_or(0, Kind::Gray, [0.0; 4]);
    let map = context.gray_or(1, 0.0);
    let (dx, dy) = direction(context.f("angle"));
    const BASE_RESOLUTION: f32 = 256.0;
    let intensity_scale = context.f("intensity") / BASE_RESOLUTION;
    Ok(vec![Image::generate_uv(context.w, context.h, image.kind(), |u, v| {
        let offset = map.sample_gray(u, v, context.tile) * intensity_scale;
        image.sample(u + dx * offset, v + dy * offset, context.tile)
    })])
}
