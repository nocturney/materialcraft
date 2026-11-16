use materialcraft_core::OutputFormat;
use materialcraft_image::Image;
use std::sync::Arc;

pub(crate) fn apply(mut image: Image, format: OutputFormat) -> Image {
    const HALF_FLOAT_MAX: f32 = 65504.0;
    const U8_FIXED_SCALE: f64 = 4096.0;
    const U16_FIXED_SCALE: f64 = 1_048_576.0;

    image.integer_sampling = integer_sampling(format);
    if format == OutputFormat::F16 {
        for pixel in Arc::make_mut(&mut image.data) {
            // Half-float storage saturates finite overflow and truncates toward zero
            // Leaving the f32 unconverted changes every downstream read and export
            let value = pixel.clamp(-HALF_FLOAT_MAX, HALF_FLOAT_MAX);
            let rounded = half::f16::from_f32(value);
            *pixel = if rounded.to_f32().abs() > value.abs() { half::f16::from_bits(rounded.to_bits().saturating_sub(1)).to_f32() } else { rounded.to_f32() };
        }

        return image;
    }

    let (fraction, maximum) = match format {
        OutputFormat::U8 => (U8_FIXED_SCALE, f64::from(u8::MAX)),
        OutputFormat::Inherit | OutputFormat::U16 => (U16_FIXED_SCALE, f64::from(u16::MAX)),
        OutputFormat::F16 | OutputFormat::F32 => return image,
    };

    for pixel in Arc::make_mut(&mut image.data) {
        // Integer outputs retain four fractional guard bits before conversion
        // Halfway-down rounding affects constants and downstream blends
        let value = if pixel.is_nan() { 0.0 } else { f64::from(*pixel).clamp(0.0, 1.0) };
        let fixed = (value * fraction).floor() / fraction;
        let integer = (fixed * maximum - 0.5).ceil() as u32;
        *pixel = integer as f32 / maximum as f32;
    }

    image
}

pub(crate) fn integer_sampling(format: OutputFormat) -> bool {
    matches!(format, OutputFormat::Inherit | OutputFormat::U8 | OutputFormat::U16)
}
