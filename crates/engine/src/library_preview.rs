//! Library previews: a small image of what a library graph makes from sample inputs

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use craft_texture::{Image, Kind};
use materialcraft_core::Package;

use crate::{NodeLibrary, Session};

/// Smallest spread between an image's darkest and brightest sample that still shows something
const FLAT_SPREAD: f32 = 0.01;
/// Side in pixels a graph is cooked at before it is scaled down to the preview
const COOKED_SIZE: u32 = 128;

impl NodeLibrary {
    /// What graph `graph` of package `package` makes at `size` pixels, from a wave height, its
    /// colors and its normal map on the graph's inputs. `None` when the graph has no output that
    /// cooks or its image is one flat value
    pub fn preview(library: &Arc<NodeLibrary>, package: &str, graph: &str, size: u32) -> Option<Image> {
        Self::preview_cancellable(library, package, graph, size, &AtomicBool::new(false))
    }

    /// Stop stale library work between nodes when its tile leaves the visible panel.
    pub fn preview_cancellable(library: &Arc<NodeLibrary>, package: &str, graph: &str, size: u32, cancel: &AtomicBool) -> Option<Image> {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }

        let source = library.packages.get(package)?;
        let index = source.graphs.iter().position(|candidate| candidate.function.is_none() && candidate.sbs_path() == graph)?;
        let size = size.clamp(1, COOKED_SIZE);

        // A session resolves library references just as it does for an opened package
        let mut session = Session::with_library(Package::clone(source), library.clone());
        session.limit_preview_caches();
        if cancel.load(Ordering::Relaxed) {
            return None;
        }

        let graph = session.pkg.graphs.get(index)?;
        let decode = session.resource_decoder();
        let mut options = materialcraft_eval::Options::new(&decode);

        // Cook larger before downsampling so fine detail survives in thumbnails
        let cooked_size = size.max(COOKED_SIZE);
        options.max_size = Some(cooked_size);
        options.cancel = Some(cancel);

        // Unconnected experiments and intermediate previews do not contribute to a library tile
        options.only = Some(graph.nodes.iter().filter(|node| node.def == "output").map(|node| node.id).collect());
        options.inputs = sample_inputs(graph, cooked_size);

        let cooked = materialcraft_eval::cook(&session.pkg, index, &options, &session.cache);
        if cooked.cancelled || cancel.load(Ordering::Relaxed) {
            return None;
        }

        let outputs = materialcraft_eval::outputs(graph, &cooked);
        let base_color = outputs.iter().find(|(_, usage, _, image)| *usage == "baseColor" && image.is_some());
        let image = base_color.or_else(|| outputs.iter().find(|output| output.3.is_some()))?.3?;
        (!is_flat(image)).then(|| image.resize(size.min(image.width()), size.min(image.height()), false))
    }
}

/// An image for every image input of `graph`, by identifier: normals for normal inputs, colors for color inputs, the height for the rest
fn sample_inputs(graph: &materialcraft_core::Graph, size: u32) -> BTreeMap<String, Image> {
    // Strong relief keeps normal inputs legible in small previews
    const NORMAL_INTENSITY: f32 = 4.0;

    let mut input_nodes = graph.nodes.iter().filter(|node| matches!(node.def.as_str(), "input_color" | "input_grayscale")).peekable();
    if input_nodes.peek().is_none() {
        return BTreeMap::new();
    }

    let height = Image::generate_uv(size, size, Kind::Gray, |u, v| {
        [0.5 + 0.25 * (u * std::f32::consts::TAU).sin() + 0.25 * (v * std::f32::consts::TAU).cos(), 0.0, 0.0, 0.0]
    });
    let mut color = None;
    let mut normal = None;

    input_nodes
        .map(|node| {
            let identifier = materialcraft_eval::identifier(node);
            let image: &Image = if identifier.to_ascii_lowercase().contains("normal") {
                normal.get_or_insert_with(|| materialcraft_nodes::atomic::height_to_normal(&height, NORMAL_INTENSITY, false, true, false))
            } else if node.def == "input_color" {
                // A blue-to-sand palette distinguishes color inputs from the grayscale height
                color.get_or_insert_with(|| height.map(Kind::Color, |pixel| [0.15 + 0.7 * pixel[0], 0.35 + 0.3 * pixel[0], 0.8 - 0.5 * pixel[0], 1.0]))
            } else {
                &height
            };

            (identifier, image.clone())
        })
        .collect()
}

/// Whether every pixel of `image` has nearly the same value
fn is_flat(image: &Image) -> bool {
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    for y in 0..i64::from(image.height()) {
        for x in 0..i64::from(image.width()) {
            let pixel = image.get(x, y, false);
            for channel in 0..3 {
                low[channel] = low[channel].min(pixel[channel]);
                high[channel] = high[channel].max(pixel[channel]);
            }
        }
    }

    (0..3).all(|channel| high[channel] - low[channel] < FLAT_SPREAD)
}
