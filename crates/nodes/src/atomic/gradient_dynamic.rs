use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("gradient_dynamic", "Gradient (Dynamic)", ATOMIC)
        .desc("Maps a grayscale input through the colors of a row of a gradient image.")
        .atomic()
        .gray_in("input", "Grayscale input")
        .any_in("gradient", "Gradient input")
        .float("position", "Position", 0.0, 0.0, 1.0)
        .choice("orientation", "Orientation", &["Horizontal", "Vertical"], 0)
        .any_out()
        .cook(gradient_dynamic)
}

fn gradient_dynamic(context: &CookContext) -> CookResult {
    let Some(gradient) = context.input(1) else {
        return Ok(vec![Image::filled(context.w, context.h, Kind::Gray, [0.0; 4])]);
    };

    let input = context.gray_or(0, 0.0);
    let position = context.f("position");
    let vertical = context.i("orientation") == 1;

    Ok(vec![Image::generate(context.w, context.h, gradient.kind(), |x, y| {
        let intensity = input.get_gray(i64::from(x), i64::from(y), false).clamp(0.0, 1.0);
        if vertical { gradient.sample(position, intensity, false) } else { gradient.sample(intensity, position, false) }
    })])
}
