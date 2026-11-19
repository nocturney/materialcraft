use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("sharpen", "Sharpen", ATOMIC)
        .desc("Sharpens local contrast with a four-neighbor Laplacian.")
        .atomic()
        .any_in("input", "Input")
        .float("intensity", "Intensity", 1.0, 0.0, 10.0)
        .any_out()
        .cook(sharpen)
}

fn sharpen(context: &CookContext) -> CookResult {
    let input = context.any_or(0, Kind::Gray, [0.0; 4]);
    let intensity = context.f("intensity");
    let tile = context.tiling_axes();
    let short_side = context.w.min(context.h) as f32;
    let horizontal = intensity * short_side / context.w.max(1) as f32;
    let vertical = intensity * short_side / context.h.max(1) as f32;

    Ok(vec![Image::generate(context.w, context.h, input.kind(), |x, y| {
        let (x, y) = (i64::from(x), i64::from(y));
        let center = input.get(x, y, false);
        let left = input.get_axes(x - 1, y, tile);
        let right = input.get_axes(x + 1, y, tile);
        let top = input.get_axes(x, y - 1, tile);
        let bottom = input.get_axes(x, y + 1, tile);
        std::array::from_fn(|channel| {
            // Keep the axial differences separate to preserve Float32 cancellation order
            let twice_center = 2.0 * center[channel];
            let across = axial_difference(center[channel], twice_center, left[channel], right[channel]);
            let down = axial_difference(center[channel], twice_center, top[channel], bottom[channel]);

            // Float outputs retain HDR values but discard negative sharpening lobes
            (center[channel] + (across * horizontal + down * vertical)).max(0.0)
        })
    })])
}

fn axial_difference(center: f32, twice_center: f32, first: f32, second: f32) -> f32 {
    let difference = twice_center - (first + second);
    if difference.is_finite() || ![center, first, second].iter().all(|value| value.is_finite()) {
        return difference;
    }

    // Finite HDR inputs can overflow both Float32 sums and turn their difference into NaN
    (2.0 * f64::from(center) - (f64::from(first) + f64::from(second))) as f32
}
