use craft_texture::{Image, Limits};
use materialcraft_core::OutputFormat;
use std::sync::Arc;

pub(crate) fn apply(image: Image, format: OutputFormat) -> Image {
    const HALF_FLOAT_MAX: f32 = 65504.0;
    const U8_FIXED_SCALE: f64 = 4096.0;
    const U16_FIXED_SCALE: f64 = 1_048_576.0;

    let integer = integer_sampling(format);
    if format == OutputFormat::F16 {
        return with_pixels(image, integer, |pixel| {
            // Half-float storage saturates finite overflow and truncates toward zero
            // Leaving the f32 unconverted changes every downstream read and export
            let value = pixel.clamp(-HALF_FLOAT_MAX, HALF_FLOAT_MAX);
            let rounded = half::f16::from_f32(value);
            *pixel = if rounded.to_f32().abs() > value.abs() { half::f16::from_bits(rounded.to_bits().saturating_sub(1)).to_f32() } else { rounded.to_f32() };
        });
    }

    let (fraction, maximum) = match format {
        OutputFormat::U8 => (U8_FIXED_SCALE, f64::from(u8::MAX)),
        OutputFormat::Inherit | OutputFormat::U16 => (U16_FIXED_SCALE, f64::from(u16::MAX)),
        OutputFormat::F16 | OutputFormat::F32 => return image.with_integer_sampling(integer),
    };

    with_pixels(image, integer, |pixel| {
        // Integer outputs retain four fractional guard bits before conversion
        // Halfway-down rounding affects constants and downstream blends
        let value = if pixel.is_nan() { 0.0 } else { f64::from(*pixel).clamp(0.0, 1.0) };
        let fixed = (value * fraction).floor() / fraction;
        let integer = (fixed * maximum - 0.5).ceil() as u32;
        *pixel = integer as f32 / maximum as f32;
    })
}

pub(crate) fn integer_sampling(format: OutputFormat) -> bool {
    matches!(format, OutputFormat::Inherit | OutputFormat::U8 | OutputFormat::U16)
}

/// `image` with `convert` applied to every float and the integer-sampling flag set. The buffer is
/// changed in place when no other image shares it, and copied once otherwise.
fn with_pixels(image: Image, integer_sampling: bool, convert: impl Fn(&mut f32)) -> Image {
    let (width, height, kind) = (image.width(), image.height(), image.kind());

    // Limits that admit any valid image, so one made with raised limits keeps its size
    let mut limits = Limits::default();
    limits.max_side = u32::MAX;
    limits.max_pixels = u64::MAX;
    limits.max_bytes = u64::MAX;

    let mut data = Arc::clone(image.shared_data());
    drop(image);

    Arc::make_mut(&mut data).iter_mut().for_each(convert);

    // `data` keeps the size and kind of a valid image, so `from_data` accepts it
    Image::from_data(width, height, kind, data, &limits)
        .unwrap_or_else(|_| Image::filled(width, height, kind, [0.0; 4]))
        .with_integer_sampling(integer_sampling)
}
