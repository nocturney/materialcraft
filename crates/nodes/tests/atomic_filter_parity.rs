use craft_texture::{Image, Kind};
use materialcraft_core::{OutputFormat, Value};
use materialcraft_nodes::{CookContext, find, resolve_params};
use std::collections::BTreeMap;

fn cook(id: &str, inputs: &[Option<Image>], values: &[(&str, Value)], size: [u32; 2], tile: bool) -> Image {
    let definition = find(id).unwrap();
    let values = values.iter().map(|(key, value)| ((*key).into(), value.clone())).collect();
    let params = resolve_params(definition, &values);
    let resource = |_: &str| None;
    let vars = BTreeMap::new();
    let context = CookContext { w: size[0], h: size[1], seed: 0, format: OutputFormat::F32, tile, inputs, params: &params, resource: &resource, vars: &vars };
    (definition.cook)(&context).unwrap().remove(0)
}

#[test]
fn shuffle_reads_grayscale_from_every_selected_channel() {
    let inputs = [Some(Image::filled(3, 5, Kind::Gray, [0.25; 4])), Some(Image::filled(3, 5, Kind::Gray, [0.75; 4]))];
    for channel in 0..8 {
        let params = ["red", "green", "blue", "alpha"].map(|key| (key, Value::Enum(channel)));

        let output = cook("channels_shuffle", &inputs, &params, [3, 5], true);

        assert_eq!(output.get(1, 2, false), [if channel < 4 { 0.25 } else { 0.75 }; 4], "channel {channel}");
    }
}

#[test]
fn shuffle_wraps_selectors_over_the_eight_input_channels() {
    let inputs = [Some(Image::filled(3, 5, Kind::Color, [0.125, 0.25, 0.375, 0.5])), Some(Image::filled(3, 5, Kind::Color, [0.625, 0.75, 0.875, 1.0]))];
    for channel in [0, 3, 4, 7, 8, 9, 10, 16, 255] {
        let params = ["red", "green", "blue", "alpha"].map(|key| (key, Value::Enum(channel)));

        let output = cook("channels_shuffle", &inputs, &params, [3, 5], true);

        assert_eq!(output.get(1, 2, false), [(channel % 8 + 1) as f32 / 8.0; 4], "channel {channel}");
    }
    assert_eq!(find("channels_shuffle").unwrap().param("red").unwrap().options.len(), 8);
}

#[test]
fn shuffle_resolves_out_of_range_selectors_after_input_kind() {
    for kinds in [[Kind::Gray, Kind::Gray], [Kind::Color, Kind::Gray], [Kind::Gray, Kind::Color]] {
        let inputs = [Some(Image::filled(3, 5, kinds[0], [0.125, 0.25, 0.375, 0.5])), Some(Image::filled(3, 5, kinds[1], [0.625, 0.75, 0.875, 1.0]))];
        let params = [("red", Value::Enum(8)), ("green", Value::Enum(9)), ("blue", Value::Enum(11)), ("alpha", Value::Enum(u32::MAX - 1))];

        let output = cook("channels_shuffle", &inputs, &params, [3, 5], true);

        let expected = if kinds[1] == Kind::Gray { [0.625; 4] } else { [0.125, 0.125, 1.0, 0.875] };

        assert_eq!(output.get(1, 2, false), expected, "kinds {kinds:?}");
    }
}

#[test]
fn sharpen_preserves_float_order_on_ramps() {
    let input = Image::gray(16, 16, |x, _| x as f32 / 15.0);

    let output = cook("sharpen", &[Some(input)], &[("intensity", Value::Float(1.0))], [16, 16], true);

    // Float32 ramps distinguish the Laplacian's arithmetic order
    for (x, bits) in [(1, 0x3d88_8889), (3, 0x3e4c_cccb), (7, 0x3eee_eeed), (13, 0x3f5d_ddde)] {
        assert_eq!(output.get_gray(x, 7, false).to_bits(), bits, "column {x}");
    }
}

#[test]
fn sharpen_preserves_finite_hdr_constant() {
    let constant = 2.0e38;
    let input = Image::filled(16, 16, Kind::Gray, [constant; 4]);

    let output = cook("sharpen", &[Some(input)], &[("intensity", Value::Float(1.0))], [16, 16], true);

    assert!(output.data().iter().all(|value| value.to_bits() == constant.to_bits()));
}

#[test]
fn blur_uses_fractional_box_coverage() {
    let input = Image::gray(16, 16, |x, y| f32::from(x == 8 && y == 8));

    let output = cook("blur", &[Some(input)], &[("intensity", Value::Float(10.0))], [16, 16], true);

    // A 0.625-pixel radius covers the center pixel and one eighth of each neighbor
    for (x, y, expected) in [(8, 8, 0.64), (7, 8, 0.08), (8, 7, 0.08), (7, 7, 0.01), (6, 8, 0.0)] {
        assert!((output.get_gray(x, y, false) - expected).abs() < 1e-7, "at {x},{y}");
    }
}

#[test]
fn blur_preserves_finite_hdr_constant() {
    let input = Image::filled(16, 16, Kind::Gray, [1.0e38; 4]);

    let output = cook("blur", &[Some(input)], &[("intensity", Value::Float(64.0))], [16, 16], true);

    assert!(output.data().iter().all(|value| value.is_finite() && (*value - 1.0e38).abs() < 1.0e32));
}

#[test]
fn directional_blur_uses_full_radius_and_fractional_endpoints() {
    let input = Image::gray(256, 256, |x, y| f32::from(x == 128 && y == 128));

    let output = cook("directional_blur", &[Some(input)], &[("intensity", Value::Float(4.0))], [256, 256], true);

    for offset in -4..=4 {
        let expected = if offset == -4 || offset == 4 { 0.0625 } else { 0.125 };

        assert_eq!(output.get_gray(128 + offset, 128, false), expected, "offset {offset}");
    }
    assert_eq!(output.get_gray(128, 129, false), 0.0);
}

#[test]
fn sharpen_has_center_gain_and_clamps_negative_lobes() {
    let input = Image::gray(16, 16, |x, y| f32::from(x == 8 && y == 8));

    let output = cook("sharpen", &[Some(input)], &[("intensity", Value::Float(1.0))], [16, 16], true);

    assert_eq!(output.get_gray(8, 8, false), 5.0);
    assert!(output.data().iter().all(|value| *value >= 0.0));
}

#[test]
fn empty_raster_preserves_transparent_rgb_and_rejects_mismatched_background_kind() {
    for id in ["text", "svg"] {
        let pixel = [0.25, 0.5, 0.75, 0.0];
        let color = Some(Image::filled(16, 16, Kind::Color, pixel));
        let gray = Some(Image::filled(16, 16, Kind::Gray, [0.25; 4]));

        let output = cook(id, std::slice::from_ref(&color), &[("color_mode", Value::Bool(true))], [16, 16], true);

        assert_eq!(output.get(0, 0, false), pixel, "{id} transparent color");

        let output = cook(id, &[color], &[("color_mode", Value::Bool(false))], [16, 16], true);

        assert_eq!(output.get_gray(0, 0, false), 0.0, "{id} color into gray");

        let output = cook(id, &[gray], &[("color_mode", Value::Bool(true))], [16, 16], true);

        assert_eq!(output.get(0, 0, false), [0.0; 4], "{id} gray into color");
    }
}

#[test]
fn rectangular_filters_scale_from_the_shorter_side() {
    for size in [[32, 16], [16, 32]] {
        let input = Image::gray(size[0], size[1], |x, y| f32::from(x == size[0] / 2 && y == size[1] / 2));
        for (operation, expected) in [("blur", 0.64), ("directional_blur", 0.8)] {
            let output = cook(operation, &[Some(input.clone())], &[("intensity", Value::Float(10.0))], size, true);

            assert!((output.get_gray(i64::from(size[0] / 2), i64::from(size[1] / 2), false) - expected).abs() < 1e-7, "{operation}, {size:?}");
        }
        let output = cook("sharpen", &[Some(input)], &[("intensity", Value::Float(1.0))], size, true);

        assert_eq!(output.get_gray(i64::from(size[0] / 2), i64::from(size[1] / 2), false), 4.0, "{size:?}");
    }
}
