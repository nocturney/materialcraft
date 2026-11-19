use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind, luminance};

pub(super) fn definition() -> NodeDefinition {
    node("uniform", "Uniform color", ATOMIC)
        .desc("A single color or gray value.")
        .atomic()
        .generator()
        .color("color", "Color", [0.0, 0.0, 0.0, 1.0])
        .boolean("color_mode", "Color output", true)
        .any_out()
        .cook(uniform)
}

fn uniform(context: &CookContext) -> CookResult {
    let color = context.v4("color");

    Ok(vec![if !context.b("color_mode") {
        Image::filled(context.w, context.h, Kind::Gray, [luminance(color); 4])
    } else {
        Image::filled(context.w, context.h, Kind::Color, color)
    }])
}
