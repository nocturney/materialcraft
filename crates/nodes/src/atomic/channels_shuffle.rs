use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use craft_texture::{Image, Kind};
use materialcraft_core::Value;

pub(super) fn definition() -> NodeDefinition {
    node("channels_shuffle", "Channels shuffle", ATOMIC)
        .desc("Builds a color image from any channels of two inputs.")
        .atomic()
        .any_in("input1", "Input 1")
        .any_in("input2", "Input 2")
        .choice("red", "Red", CHANNEL_SOURCES, 0)
        .choice("green", "Green", CHANNEL_SOURCES, 1)
        .choice("blue", "Blue", CHANNEL_SOURCES, 2)
        .choice("alpha", "Alpha", CHANNEL_SOURCES, 3)
        .color_out()
        .cook(channels_shuffle)
}

fn channels_shuffle(context: &CookContext) -> CookResult {
    // Sampling clamps the unconnected fallback to its single zero pixel
    let first_input = context.sized(0).unwrap_or_else(|| Image::filled(1, 1, Kind::Color, [0.0; 4]));
    let second_input = context.sized(1).unwrap_or_else(|| Image::filled(1, 1, Kind::Color, [0.0; 4]));
    let channel_sources = ["red", "green", "blue", "alpha"].map(|id| match context.value(id) {
        Some(Value::Enum(index)) => *index,
        _ => 0,
    });

    Ok(vec![Image::generate(context.w, context.h, Kind::Color, |x, y| {
        let first_pixel = first_input.get(x as i64, y as i64, false);
        let second_pixel = second_input.get(x as i64, y as i64, false);

        channel_sources.map(|selector| {
            // Gray selection precedes the packed color swizzle, including for file values above seven
            if selector < 4 && first_input.is_gray() {
                return first_pixel[0];
            }

            if selector >= 4 && second_input.is_gray() {
                return second_pixel[0];
            }

            const CHANNEL_MASK: u32 = 0b111;
            match selector & CHANNEL_MASK {
                channel @ 0..=3 => first_pixel[channel as usize],
                channel => second_pixel[(channel - 4) as usize],
            }
        })
    })])
}

const CHANNEL_SOURCES: &[&str] =
    &["Input 1 red", "Input 1 green", "Input 1 blue", "Input 1 alpha", "Input 2 red", "Input 2 green", "Input 2 blue", "Input 2 alpha"];
