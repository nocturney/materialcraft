use super::ATOMIC;
use crate::ops;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::Image;

pub(super) fn definition() -> NodeDefinition {
    node("distance", "Distance", ATOMIC)
        .desc("Grows white areas of a mask into a distance gradient; with a source, spreads its colors outward.")
        .atomic()
        .gray_in("mask", "Mask input")
        .any_in("source", "Source input")
        .float("distance", "Maximum distance", 10.0, 0.0, 256.0)
        .boolean("combine", "Combine distance with source", true)
        .any_out()
        .cook(distance)
}

fn distance(context: &CookContext) -> CookResult {
    let mask = context.gray_or(0, 0.0);
    const BASE_RESOLUTION: f32 = 256.0;
    const MIN_DISTANCE_PIXELS: f32 = 1e-3;
    let maximum_distance = (context.f("distance").max(0.0) * context.w as f32 / BASE_RESOLUTION).max(MIN_DISTANCE_PIXELS);
    let data = mask.shared_data().clone();
    let inside = |i| data.get(i).copied().unwrap_or(0.0) > 0.5;
    let Some(source) = context.sized(1) else {
        let squared_distances = ops::distance_sq(context.w, context.h, inside, context.tile);
        return Ok(vec![ops::gray_from(
            context.w,
            context.h,
            squared_distances.iter().map(|d| (1.0 - d.sqrt() / maximum_distance).clamp(0.0, 1.0)).collect(),
        )?]);
    };

    // Propagate the exact nearest site without gaps from gradient-walk stalls
    let (squared_distances, sites) = ops::distance_sq_with_sites(context.w, context.h, inside, context.tile);
    let combine = context.b("combine");
    Ok(vec![Image::generate(context.w, context.h, source.kind(), |x, y| {
        let i = y as usize * context.w as usize + x as usize;
        let site = sites.get(i).copied().unwrap_or(usize::MAX);
        let mut pixel = if site == usize::MAX {
            [0.0, 0.0, 0.0, if source.is_gray() { 1.0 } else { 0.0 }]
        } else {
            source.get((site % context.w as usize) as i64, (site / context.w as usize) as i64, false)
        };
        let d = squared_distances.get(i).copied().unwrap_or(f32::INFINITY).sqrt();
        if d > maximum_distance {
            pixel = [0.0, 0.0, 0.0, if source.is_gray() { 1.0 } else { 0.0 }];
        } else if combine {
            let falloff = (1.0 - d / maximum_distance).clamp(0.0, 1.0);
            pixel = [pixel[0] * falloff, pixel[1] * falloff, pixel[2] * falloff, pixel[3]];
        }
        pixel
    })])
}
