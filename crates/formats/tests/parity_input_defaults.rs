use materialcraft_core::{Graph, Package, Value};

#[test]
fn grayscale_image_defaults_are_scalar_in_sbs() {
    let mut package = Package::new("Grayscale defaults");
    let mut graph = Graph::new("defaults");
    graph.add_node("input_grayscale", [0.0, 0.0]);
    let colored = graph.add_node("input_grayscale", [0.0, 100.0]);
    graph.node_mut(colored).unwrap().params.insert("default".into(), Value::Color([0.25, 0.5, 0.75, 0.125]));
    package.graphs.push(graph);

    let bytes = materialcraft_formats::sbs::write_sbs(&package).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let document = roxmltree::Document::parse(&text).unwrap();
    let defaults: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("paraminput"))
        .map(|node| {
            let default = node.children().find(|node| node.has_tag_name("defaultValue")).unwrap();
            let constant = default.children().find(roxmltree::Node::is_element).unwrap();
            (constant.tag_name().name().to_string(), constant.attribute("v").unwrap().to_string())
        })
        .collect();

    assert_eq!(defaults, vec![("constantValueFloat1".into(), "0.5".into()), ("constantValueFloat1".into(), "0.25".into())]);
}
