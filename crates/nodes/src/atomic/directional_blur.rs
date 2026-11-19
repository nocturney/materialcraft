use super::{ATOMIC, box_filter, direction};
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("directional_blur", "Directional blur", ATOMIC)
        .desc("Blurs along one direction.")
        .atomic()
        .any_in("input", "Input")
        .float("intensity", "Intensity", 10.0, 0.0, 64.0)
        .angle("angle", "Angle", 0.0)
        .any_out()
        .cook(directional_blur)
}

fn directional_blur(context: &CookContext) -> CookResult {
    blur(context, context.f("angle"))
}

pub(super) fn blur(context: &CookContext, angle: f32) -> CookResult {
    let input = context.any_or(0, Kind::Gray, [0.0; 4]);
    const BASE_RESOLUTION: f32 = 256.0;
    const MAX_HALF_SAMPLES: i32 = 512;
    let radius = context.f("intensity").max(0.0) * context.w.min(context.h) as f32 / BASE_RESOLUTION;
    if radius <= 0.5 {
        return Ok(vec![input]);
    }

    let (dx, dy) = direction(angle);

    // Image-space rotation follows increasing rows
    let dy = -dy;
    let tile = context.tiling_axes();
    if dy == 0.0 {
        return Ok(vec![box_filter::axis(&input, radius, true, tile[0])]);
    }

    let major = dx.abs().max(dy.abs());
    let projected_radius = 0.5 + (radius - 0.5) * major;
    let half_samples = (projected_radius + 0.5).floor().clamp(1.0, MAX_HALF_SAMPLES as f32) as i32;

    // Bound oblique sampling cost while preserving the extent of large kernels
    let step = ((projected_radius - 0.5) / half_samples as f32).max(1.0);
    let weight = step / (2.0 * projected_radius);
    let edge_weight = (0.5 - (half_samples as f32 - 0.5) * weight).max(0.0);
    let (inverse_width, inverse_height) = (1.0 / context.w as f32, 1.0 / context.h as f32);

    Ok(vec![Image::generate_uv(context.w, context.h, input.kind(), |u, v| {
        let mut pixel = [0.0; 4];
        for i in -half_samples..=half_samples {
            let sample_weight = if i.abs() == half_samples { edge_weight } else { weight };
            if sample_weight == 0.0 {
                continue;
            }

            let offset = i as f32 * step / major;
            let sample = input.sample_axes(u + dx * offset * inverse_width, v + dy * offset * inverse_height, tile);
            for channel in 0..4 {
                pixel[channel] += sample[channel] * sample_weight;
            }
        }
        pixel
    })])
}
