use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use materialcraft_image::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("normal", "Normal", ATOMIC)
        .desc("Converts a height map to a tangent-space normal map.")
        .atomic()
        .gray_in("input", "Input")
        .float("intensity", "Intensity", 1.0, 0.0, 10.0)
        .boolean("opengl", "OpenGL normals", true)
        .boolean("height_alpha", "Height in alpha", false)
        .color_out()
        .cook(normal)
}

fn normal(context: &CookContext) -> CookResult {
    let height_map = context.gray_or(0, 0.0);
    let intensity = context.f("intensity") * context.w as f32 / 256.0;
    Ok(vec![height_to_normal(&height_map, intensity, !context.b("opengl"), context.tile, context.b("height_alpha"))])
}

pub fn height_to_normal(height_map: &Image, intensity: f32, directx: bool, tile: bool, alpha_height: bool) -> Image {
    Image::generate(height_map.w, height_map.h, Kind::Color, |x, y| {
        // Backward differences place alpha wrapped discontinuity on its owning pixel
        let height = height_map.get_gray(x as i64, y as i64, tile);
        let dx = height - height_map.get_gray(x as i64 - 1, y as i64, tile);
        let dy = height - height_map.get_gray(x as i64, y as i64 - 1, tile);
        let (nx, ny, nz) = (-dx * intensity, dy * intensity, 1.0);
        let length = (nx * nx + ny * ny + nz * nz).sqrt();
        let ny = if directx { -ny } else { ny };
        let alpha = if alpha_height { height } else { 1.0 };
        [nx / length * 0.5 + 0.5, ny / length * 0.5 + 0.5, nz / length * 0.5 + 0.5, alpha]
    })
}
