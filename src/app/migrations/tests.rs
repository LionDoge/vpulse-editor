use super::*;
use crate::app::types::PulseNodeData;
use crate::bindings::{FunctionBinding, LibraryBindingType, ParamInfo, PolimorphicTypeInfo};
use crate::typing::LibraryBindingIndex;

fn param(name: &str, pulsetype: PulseValueType) -> ParamInfo {
    ParamInfo {
        name: name.into(),
        typ: String::new(),
        pulsetype,
        polymorphic_arg: None,
    }
}

fn binding_node(outputs: Vec<ParamInfo>) -> (FullGraphState, NodeId) {
    let mut state = FullGraphState::default();
    state.user_state.bindings.gamefunctions.push(FunctionBinding {
        id: LibraryBindingIndex(11),
        typ: LibraryBindingType::Action,
        displayname: "Remove Entity".into(),
        libname: "CPulseServerFuncs::RemoveEntity".into(),
        description: None,
        inparams: None,
        outparams: Some(outputs),
        polymorphic_return: None,
    });
    let node = state.state.graph.add_node(
        "binding".into(),
        PulseNodeData {
            template: PulseNodeTemplate::LibraryBindingAssigned { binding: LibraryBindingIndex(11) },
            ..Default::default()
        },
        |_, _| {},
    );
    (state, node)
}

#[test]
fn adds_missing_retval_even_without_inputs_and_preserves_action_connections() {
    let (mut state, node) = binding_node(vec![param("retval", PulseValueType::PVAL_BOOL)]);
    let graph = &mut state.state.graph;
    let action = graph.add_output_param(node, "outAction".into(), PulseDataType::Action);
    let sink = graph.add_node("sink".into(), Default::default(), |_, _| {});
    let input = graph.add_input_param(sink, "ActionIn".into(), PulseDataType::Action,
        PulseGraphValueType::Action, InputParamKind::ConnectionOnly, true);
    graph.add_connection(action, input, 0);

    verify_compat(&mut state);
    let retval = state.state.graph[node].get_output("retval").unwrap();
    assert_eq!(state.state.graph[retval].typ, PulseDataType::Bool);
    assert_eq!(state.state.graph.connections[input], vec![action]);
    verify_compat(&mut state);
    assert_eq!(state.state.graph[node].get_output("retval").unwrap(), retval);
    assert_eq!(state.state.graph[node].outputs.len(), 2);
}

#[test]
fn updates_return_type_and_matches_output_names_before_renames() {
    let (mut state, node) = binding_node(vec![
        param("new_name", PulseValueType::PVAL_INT(None)),
        param("unchanged", PulseValueType::PVAL_INT(None)),
        param("retval", PulseValueType::PVAL_BOOL),
        param("added", PulseValueType::PVAL_STRING(None)),
    ]);
    let graph = &mut state.state.graph;
    // Deliberately use a different order from the current binding.
    let unchanged = graph.add_output_param(node, "unchanged".into(), PulseDataType::Integer);
    let renamed = graph.add_output_param(node, "old_name".into(), PulseDataType::Integer);
    let retval = graph.add_output_param(node, "retval".into(), PulseDataType::String);
    graph[node].user_data.custom_output_type = Some(PulseValueType::PVAL_STRING(None));
    let sink = graph.add_node("sink".into(), Default::default(), |_, _| {});
    let input = graph.add_input_param(sink, "value".into(), PulseDataType::Integer,
        PulseGraphValueType::Integer { value: 0 }, InputParamKind::ConnectionOnly, true);
    graph.add_connection(renamed, input, 0);

    verify_compat(&mut state);
    let graph = &state.state.graph;
    assert_eq!(graph[node].get_output("new_name").unwrap(), renamed);
    assert_eq!(graph[node].get_output("unchanged").unwrap(), unchanged);
    assert_eq!(graph[node].get_output("retval").unwrap(), retval);
    assert_eq!(graph[retval].typ, PulseDataType::Bool);
    assert_eq!(graph.connections[input], vec![renamed]);
    assert!(graph[node].user_data.custom_output_type.is_none());
    assert_eq!(graph[graph[node].get_output("added").unwrap()].typ, PulseDataType::String);
    verify_compat(&mut state);
    assert_eq!(state.state.graph[node].outputs.len(), 4);
}

#[test]
fn retains_resolved_polymorphic_return_without_overriding_other_outputs() {
    let (mut state, node) = binding_node(vec![
        param("retval", PulseValueType::PVAL_ANY),
        param("success", PulseValueType::PVAL_BOOL),
    ]);
    state.user_state.bindings.gamefunctions[0].polymorphic_return =
        Some(PolimorphicTypeInfo::FullType("value".into()));
    state.state.graph[node].user_data.custom_output_type = Some(PulseValueType::PVAL_STRING(None));
    verify_compat(&mut state);
    let graph = &state.state.graph;
    assert_eq!(graph[graph[node].get_output("retval").unwrap()].typ, PulseDataType::String);
    assert_eq!(graph[graph[node].get_output("success").unwrap()].typ, PulseDataType::Bool);
    assert_eq!(graph[node].user_data.custom_output_type, Some(PulseValueType::PVAL_STRING(None)));
}

#[test]
fn input_type_refresh_converts_numeric_constants_instead_of_resetting_them() {
    let cases = [
        (PulseGraphValueType::Scalar { value: 42.0 }, PulseValueType::PVAL_INT(None),
            PulseGraphValueType::Integer { value: 42 }),
        (PulseGraphValueType::Scalar { value: -12.75 }, PulseValueType::PVAL_INT(None),
            PulseGraphValueType::Integer { value: -12 }),
        (PulseGraphValueType::Integer { value: -123 }, PulseValueType::PVAL_FLOAT(None),
            PulseGraphValueType::Scalar { value: -123.0 }),
        // Unrelated types still use the new parameter's default.
        (PulseGraphValueType::String { value: "42".into() }, PulseValueType::PVAL_INT(None),
            PulseGraphValueType::Integer { value: 0 }),
    ];
    for (old_value, new_type, expected_value) in cases {
        let (mut state, node) = binding_node(vec![]);
        state.user_state.bindings.gamefunctions[0].inparams = Some(vec![param("value", new_type.clone())]);
        let old_type = match &old_value {
            PulseGraphValueType::Scalar { .. } => PulseDataType::Scalar,
            PulseGraphValueType::Integer { .. } => PulseDataType::Integer,
            _ => PulseDataType::String,
        };
        let input = state.state.graph.add_input_param(node, "value".into(), old_type,
            old_value, InputParamKind::ConnectionOrConstant, true);
        verify_compat(&mut state);
        assert_eq!(state.state.graph[node].get_input("value").unwrap(), input);
        assert_eq!(state.state.graph[input].typ, pulse_value_type_to_node_types(&new_type).0);
        assert_eq!(state.state.graph[input].value, expected_value);
        // Reloading an already migrated graph must leave the converted value alone.
        verify_compat(&mut state);
        assert_eq!(state.state.graph[input].value, expected_value);
    }
}
