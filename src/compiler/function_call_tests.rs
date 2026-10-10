use super::*;
use kv3::{ObjectKey, Value};

fn node(graph: &mut PulseGraph, template: PulseNodeTemplate) -> NodeId {
    let id = graph.add_node("test".into(), PulseNodeData { template, ..Default::default() }, |_, _| {});
    graph.add_output_param(id, "outAction".into(), PulseDataType::Action);
    id
}

fn call(graph: &mut PulseGraph, target: NodeId, is_async: bool) -> NodeId {
    let id = node(graph, PulseNodeTemplate::CallNode);
    graph.add_input_param(id, "nodeId".into(), PulseDataType::NoideChoice,
        PulseGraphValueType::NodeChoice { node: Some(target) }, InputParamKind::ConstantOnly, true);
    graph.add_input_param(id, "Async".into(), PulseDataType::Bool,
        PulseGraphValueType::Bool { value: is_async }, InputParamKind::ConstantOnly, true);
    graph.add_input_param(id, "inAction".into(), PulseDataType::Action,
        PulseGraphValueType::Action, InputParamKind::ConnectionOnly, true);
    id
}

fn connect(graph: &mut PulseGraph, from: NodeId, to: NodeId) {
    let output = graph.nodes[from].get_output("outAction").unwrap();
    let input = graph.nodes[to].get_input("inAction").unwrap();
    graph.add_connection(output, input, 0);
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    let Value::Object(fields) = value else { panic!("expected object") };
    &fields.iter().find(|(key, _)| matches!(key, ObjectKey::Identifier(key) if key == name))
        .unwrap_or_else(|| panic!("missing {name}")).1
}

fn array(value: &Value) -> &[Value] {
    let Value::Array(values) = value else { panic!("expected array") };
    values
}

fn number(value: &Value) -> i32 {
    let Value::Number(value) = value else { panic!("expected number") };
    *value as i32
}

#[test]
fn sync_async_and_recursive_calls_resolve_to_serialized_method_cells() {
    let mut graph = PulseGraph::default();
    let mut state = PulseGraphState::default();
    let root = node(&mut graph, PulseNodeTemplate::CellPublicMethod);
    graph.add_input_param(root, "name".into(), PulseDataType::String,
        PulseGraphValueType::String { value: "entry".into() }, InputParamKind::ConstantOnly, true);
    graph.add_output_param(root, "argument1".into(), PulseDataType::String);
    let a = node(&mut graph, PulseNodeTemplate::Function);
    let b = node(&mut graph, PulseNodeTemplate::Function);
    state.exposed_nodes.insert(a, "function_a".into());
    state.exposed_nodes.insert(b, "function_b".into());
    let async_a = call(&mut graph, a, true);
    let sync_a = call(&mut graph, a, false);
    let async_b = call(&mut graph, b, true);
    let a_to_b = call(&mut graph, b, false);
    let b_to_a = call(&mut graph, a, true);
    connect(&mut graph, root, async_a);
    connect(&mut graph, async_a, sync_a);
    connect(&mut graph, sync_a, async_b);
    connect(&mut graph, a, a_to_b);
    connect(&mut graph, b, b_to_a);

    let mut compiled = PulseGraphDef::default();
    assert!(traverse_inflow_nodes(&graph, &mut compiled, &state).unwrap());
    // Recursive and repeated references must not duplicate methods or chunks.
    assert_eq!(compiled.cells.len(), 3);
    assert_eq!(compiled.chunks.len(), 3);
    assert_eq!(compiled.call_infos.len(), 5);
    let Value::File(_, graph_value) = compiled.serialize(&state.bindings) else { panic!("expected file") };
    let cells = array(field(&graph_value, "m_Cells"));
    let calls = array(field(&graph_value, "m_CallInfos"));
    let chunks = array(field(&graph_value, "m_Chunks"));
    for (index, expected_name) in ["entry", "function_a", "function_b"].iter().enumerate() {
        assert!(matches!(field(&cells[index], "_class"), Value::String(s) if s == "CPulseCell_Inflow_Method"));
        assert!(matches!(field(&cells[index], "m_MethodName"), Value::String(s) if s == expected_name));
        assert!(matches!(field(&cells[index], "m_bIsPublic"), Value::Bool(value) if *value == (index == 0)));
    }
    let mut opcodes = Vec::new();
    for (index, info) in calls.iter().enumerate() {
        let method_id = number(field(info, "m_CallMethodID"));
        assert!(method_id > 0);
        let method = cells.iter().find(|cell| number(field(cell, "m_nEditorNodeID")) == method_id).unwrap();
        let src_chunk = number(field(info, "m_nSrcChunk")) as usize;
        let src_instruction = number(field(info, "m_nSrcInstruction")) as usize;
        let instruction = &array(field(&chunks[src_chunk], "m_Instructions"))[src_instruction];
        assert_eq!(number(field(instruction, "m_nCallInfoIndex")), index as i32);
        assert_eq!(number(field(instruction, "m_nChunk")), number(field(method, "m_EntryChunk")));
        opcodes.push(field(instruction, "m_nCode"));
        let Value::Object(fields) = info else { unreachable!() };
        assert!(!fields.iter().any(|(key, _)| matches!(key, ObjectKey::Identifier(s) if s == "m_nCallMethodID")));
    }
    assert_eq!(opcodes.iter().filter(|code| matches!(code, Value::String(s) if s == "PULSE_CALL_ASYNC_FIRE")).count(), 3);
    assert_eq!(opcodes.iter().filter(|code| matches!(code, Value::String(s) if s == "PULSE_CALL_SYNC")).count(), 2);
}
