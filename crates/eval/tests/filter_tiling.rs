use craft_texture::Image;
use materialcraft_core::{Graph, OutputFormat, Package, ParameterInheritance, Value};
use materialcraft_eval::{Cache, Options, cook};
use std::sync::Mutex;

#[test]
fn atomic_filters_wrap_only_the_requested_axes_and_invalidate_cache() {
    for operation in ["blur", "directional_blur", "sharpen"] {
        let mut graph = Graph::new("Filter boundaries");
        graph.size = [4; 2];
        graph.format = OutputFormat::F32;
        graph.tiling_mode = ParameterInheritance::Absolute;
        let input = graph.add_node("input_grayscale", [0.0; 2]);
        graph.node_mut(input).unwrap().params.insert("identifier".into(), Value::Text("source".into()));
        let filter = graph.add_node(operation, [100.0, 0.0]);
        graph.node_mut(filter).unwrap().params.insert("intensity".into(), Value::Float(if operation == "sharpen" { 1.0 } else { 16.0 }));
        graph.connect(input, "output", filter, "input").unwrap();
        let mut package = Package::new("Filter boundaries");
        package.graphs.push(graph);
        let resource = |_: &str| None;
        let mut options = Options::new(&resource);
        options.inputs.insert("source".into(), Image::gray(16, 16, |x, y| if x == 0 && y == 0 { 0.5625 } else { 0.5 }));
        let cache = Mutex::new(Cache::default());

        for tiling in [[false, false], [true, false], [false, true], [true, true]] {
            package.graphs[0].tiling = tiling;

            let cooked = cook(&package, 0, &options, &cache);
            let image = cooked.image(filter, 0).unwrap();

            assert_eq!(cooked.nodes[&filter].error, None);
            assert!(!cooked.nodes[&filter].cached);
            assert_eq!(image.get_gray(15, 0, false) != 0.5, tiling[0], "{operation}, x, {tiling:?}");
            assert_eq!(image.get_gray(0, 15, false) != 0.5, tiling[1] && operation != "directional_blur", "{operation}, y, {tiling:?}");

            let cached = cook(&package, 0, &options, &cache);

            assert!(cached.nodes[&filter].cached);
        }
    }
}
