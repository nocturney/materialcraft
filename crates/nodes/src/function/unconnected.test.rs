use super::{EvaluationContext, EvaluationState, compile, definition};
use materialcraft_core::{FunctionGraph, Value};

#[test]
fn unconnected_typed_operations_use_their_operand_types() {
    for (operation, expected) in [
        ("vector3", Value::Float3([0.0; 3])),
        ("vector4", Value::Float4([0.0; 4])),
        ("ivector2", Value::Int2([0; 2])),
        ("ivector3", Value::Int3([0; 3])),
        ("ivector4", Value::Int4([0; 4])),
        ("iswizzle1", Value::Int(0)),
        ("iswizzle4", Value::Int4([0; 4])),
        ("tofloat", Value::Float(0.0)),
        ("tofloat2", Value::Float2([0.0; 2])),
        ("tofloat3", Value::Float3([0.0; 3])),
        ("tofloat4", Value::Float4([0.0; 4])),
        ("toint2", Value::Int2([0; 2])),
        ("toint3", Value::Int3([0; 3])),
        ("toint4", Value::Int4([0; 4])),
        ("and", Value::Bool(false)),
        ("or", Value::Bool(false)),
        ("not", Value::Bool(true)),
        ("atan2", Value::Float(0.0)),
        ("samplelum", Value::Float(0.0)),
        ("samplecol", Value::Float4([0.0; 4])),
    ] {
        let mut graph = FunctionGraph::default();
        let id = graph.add(operation, &[], None);
        graph.nodes.first_mut().unwrap().inputs.resize(definition(operation).unwrap().inputs.len(), None);
        graph.output = Some(id);
        let variables = Default::default();
        let context = EvaluationContext { scope: Default::default(), position: [0.5; 2], size: [16.0; 2], images: &[], tile: true, variables: &variables };

        let value = compile(&graph, 0).unwrap().evaluate(&context, &mut EvaluationState::default());

        assert_eq!(value, Ok(expected), "{operation}");
    }
}

#[test]
fn partially_connected_vectors_fill_only_the_missing_components() {
    for (operation, first, last, expected) in [
        ("vector3", Some(Value::Float2([0.25, 0.75])), None, Value::Float3([0.25, 0.75, 0.0])),
        ("vector4", None, Some(Value::Float2([0.25, 0.75])), Value::Float4([0.0, 0.0, 0.25, 0.75])),
        ("ivector4", Some(Value::Int(7)), None, Value::Int4([7, 0, 0, 0])),
    ] {
        let mut graph = FunctionGraph::default();
        let inputs = [first.map(|value| graph.add_constant(value)), last.map(|value| graph.add_constant(value))];
        let id = graph.add(operation, &[], None);
        graph.nodes.last_mut().unwrap().inputs = inputs.to_vec();
        graph.output = Some(id);
        let variables = Default::default();
        let context = EvaluationContext { scope: Default::default(), position: [0.5; 2], size: [16.0; 2], images: &[], tile: true, variables: &variables };

        assert_eq!(compile(&graph, 0).unwrap().evaluate(&context, &mut EvaluationState::default()), Ok(expected), "{operation}");
    }
}

#[test]
fn partially_connected_dot_uses_the_connected_vector_width() {
    for vector in [Value::Float3([1.0, 2.0, 3.0]), Value::Float4([1.0, 2.0, 3.0, 4.0])] {
        for connected_slot in 0..2 {
            let mut graph = FunctionGraph::default();
            let source = graph.add_constant(vector.clone());
            let dot = graph.add("dot", &[], None);
            graph.nodes.last_mut().unwrap().inputs = if connected_slot == 0 { vec![Some(source), None] } else { vec![None, Some(source)] };
            graph.output = Some(dot);
            let variables = Default::default();
            let context = EvaluationContext { scope: Default::default(), position: [0.5; 2], size: [16.0; 2], images: &[], tile: true, variables: &variables };

            assert_eq!(compile(&graph, 0).unwrap().evaluate(&context, &mut EvaluationState::default()), Ok(Value::Float(0.0)));
        }
    }
}
