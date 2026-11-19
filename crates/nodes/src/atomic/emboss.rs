use super::{ATOMIC, direction};
use crate::ops;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("emboss", "Emboss", ATOMIC)
        .desc("Lights the input with the relief of a height map.")
        .atomic()
        .any_in("input", "Input")
        .gray_in("height", "Intensity input")
        .float("intensity", "Intensity", 5.0, 0.0, 32.0)
        .angle("light_angle", "Light angle", 0.0)
        .color("highlight", "Highlight color", [1.0, 1.0, 1.0, 1.0])
        .color("shadow", "Shadow color", [0.0; 4])
        .any_out()
        .cook(emboss)
}

fn emboss(context: &CookContext) -> CookResult {
    const INTENSITY_REFERENCE_WIDTH: f32 = 64.0;
    let input = context.any_or(0, Kind::Gray, [0.0; 4]);
    let height_map = context.gray_or(1, 0.0);
    let (light_x, light_y) = direction(context.f("light_angle"));
    let intensity = context.f("intensity") * context.w as f32 / INTENSITY_REFERENCE_WIDTH;
    let (highlight, shadow) = (context.v4("highlight"), context.v4("shadow"));

    Ok(vec![Image::generate(context.w, context.h, input.kind(), |x, y| {
        let (gx, gy) = ops::slope(&height_map, x as i64, y as i64, context.tile);
        let lighting = (-(gx * light_x + gy * light_y) * intensity).clamp(-1.0, 1.0);
        let pixel = input.get(x as i64, y as i64, false);
        let tint = if lighting >= 0.0 { highlight } else { shadow };
        let blend = lighting.abs();
        [pixel[0] + (tint[0] - pixel[0]) * blend, pixel[1] + (tint[1] - pixel[1]) * blend, pixel[2] + (tint[2] - pixel[2]) * blend, pixel[3]]
    })])
}
