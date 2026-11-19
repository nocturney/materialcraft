use craft_texture::{Image, Kind, luminance};

use crate::CookContext;

pub(crate) fn composite(context: &CookContext, foreground: impl Fn(u32, u32) -> [f32; 4] + Sync) -> Image {
    let kind = if context.b("color_mode") { Kind::Color } else { Kind::Gray };
    let background = match context.input(0) {
        Some(image) if image.kind() == kind => image.resize(context.w, context.h, context.tile),

        // An incompatible connection replaces the constant background with empty pixels
        Some(_) => Image::filled(1, 1, kind, [0.0; 4]),
        None => Image::filled(1, 1, Kind::Color, context.v4("background")),
    };
    let gray_background = background.is_gray();

    // Clamped sampling broadcasts constant backgrounds from one pixel
    Image::generate(context.w, context.h, kind, |x, y| {
        let foreground = foreground(x, y);
        let background = background.get(i64::from(x), i64::from(y), false);
        let foreground_alpha = foreground[3].clamp(0.0, 1.0);
        if foreground_alpha == 0.0 {
            // Empty content must preserve hidden RGB in a transparent background
            return if kind == Kind::Gray && !gray_background { [luminance(background), 0.0, 0.0, 0.0] } else { background };
        }

        let background_alpha = background[3].clamp(0.0, 1.0) * (1.0 - foreground_alpha);
        let alpha = foreground_alpha + background_alpha;
        if alpha <= 0.0 {
            return [0.0; 4];
        }

        let channel = |index: usize| (foreground[index] * foreground_alpha + background[index] * background_alpha) / alpha;
        let pixel = [channel(0), channel(1), channel(2), alpha];
        if kind == Kind::Gray { [luminance(pixel), 0.0, 0.0, 0.0] } else { pixel }
    })
}
