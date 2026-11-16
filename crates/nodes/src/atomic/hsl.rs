use super::ATOMIC;
use crate::ops;
use crate::{CookContext, CookResult, NodeDefinition, node};
use materialcraft_image::Kind;

pub(super) fn definition() -> NodeDefinition {
    node("hsl", "HSL", ATOMIC)
        .desc("Hue, saturation and lightness adjustments (0.5 = unchanged).")
        .atomic()
        .color_in("input", "Input")
        .float("hue", "Hue", 0.5, 0.0, 1.0)
        .float("saturation", "Saturation", 0.5, 0.0, 1.0)
        .float("lightness", "Lightness", 0.5, 0.0, 1.0)
        .color_out()
        .cook(hsl)
}

fn hsl(context: &CookContext) -> CookResult {
    let image = context.any_or(0, Kind::Color, [0.5, 0.5, 0.5, 1.0]).to_color();

    // The normalized slider spans one full turn in either direction from its midpoint
    let (hue_shift, saturation, lightness) = ((context.f("hue") - 0.5) * 2.0, context.f("saturation"), context.f("lightness"));
    Ok(vec![image.map(Kind::Color, |pixel| {
        let [h, s, l] = ops::rgb_to_hsl(pixel);
        let s = if saturation <= 0.5 { s * saturation * 2.0 } else { s + (1.0 - s) * (saturation - 0.5) * 2.0 };
        let l = if lightness <= 0.5 { l * lightness * 2.0 } else { l + (1.0 - l) * (lightness - 0.5) * 2.0 };
        let color = ops::hsl_to_rgb(h + hue_shift, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
        [color[0], color[1], color[2], pixel[3]]
    })])
}
