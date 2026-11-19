use materialcraft_core::{FunctionGraph, Graph, Package, Value};

#[test]
fn function_loop_limit_uses_scalar_integer_data() {
    let mut function = FunctionGraph::default();
    let initial = function.add_constant(Value::Float(0.25));
    let condition = function.add_constant(Value::Bool(true));
    let body = function.add_constant(Value::Float(0.75));
    function.output = Some(function.add("while", &[initial, condition, body], Some(Value::Int(3))));
    let mut graph = Graph::new("Loop serialization");
    graph.function = Some(function);
    let mut package = Package::new("Original function data");
    package.graphs.push(graph);

    let bytes = materialcraft_formats::sbs::write_sbs(&package).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let document = roxmltree::Document::parse(&text).unwrap();
    let loop_node = document
        .descendants()
        .find(|node| node.has_tag_name("paramNode") && node.children().any(|child| child.has_tag_name("function") && child.attribute("v") == Some("while")))
        .unwrap();
    let limit = loop_node.descendants().find(|node| node.has_tag_name("constantValue")).unwrap().children().find(roxmltree::Node::is_element).unwrap();

    assert_eq!(limit.tag_name().name(), "constantValueInt1");
    assert_eq!(limit.attribute("v"), Some("3"));
}
