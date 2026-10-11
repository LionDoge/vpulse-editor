use super::types::*;
use crate::typing::{get_preffered_inputparamkind_from_type, PulseValueType};
use egui_node_graph2::{InputParamKind, NodeId};

fn callers(graph: &PulseGraph, function: NodeId) -> Vec<NodeId> {
    graph.nodes.iter().filter_map(|(id, node)| {
        if node.user_data.template != PulseNodeTemplate::CallNode { return None; }
        let input = node.get_input("nodeId").ok()?;
        match graph[input].value {
            PulseGraphValueType::NodeChoice { node: Some(target) } if target == function => Some(id),
            _ => None,
        }
    }).collect()
}

pub(super) fn add_argument(graph: &mut PulseGraph, function: NodeId) {
    let args = &graph[function].user_data.function_arguments;
    let name = (1..).map(|i| format!("arg{i}"))
        .find(|name| !args.iter().any(|arg| &arg.name == name)).unwrap();
    graph[function].user_data.function_arguments.push(FunctionArgument {
        name, typ: PulseValueType::PVAL_FLOAT(None),
    });
    sync_function(graph, function);
}

pub(super) fn edit_argument(graph: &mut PulseGraph, function: NodeId, index: usize, new: Option<FunctionArgument>) {
    let Some(old) = graph[function].user_data.function_arguments.get(index).cloned() else { return; };
    if let Some(new) = &new {
        if !FunctionArgument::valid_name(&new.name) || graph[function].user_data.function_arguments.iter()
            .enumerate().any(|(i, arg)| i != index && arg.name == new.name) { return; }
    }
    // Rename in place before reconciling the signature so wires and constants retain their IDs.
    for caller in callers(graph, function) {
        if let Ok(input) = graph[caller].get_input(&old.name) {
            if let Some(new) = &new {
                graph[caller].inputs.iter_mut().find(|(_, id)| *id == input).unwrap().0 = new.name.clone();
            } else {
                graph.remove_input_param(input);
            }
        }
    }
    if let Ok(output) = graph[function].get_output(&old.name) {
        if let Some(new) = &new {
            graph[function].outputs.iter_mut().find(|(_, id)| *id == output).unwrap().0 = new.name.clone();
        } else {
            graph.remove_output_param(output);
        }
    }
    if let Some(new) = new {
        graph[function].user_data.function_arguments[index] = new;
    } else {
        graph[function].user_data.function_arguments.remove(index);
    }
    sync_function(graph, function);
}

pub(super) fn sync_function(graph: &mut PulseGraph, function: NodeId) {
    for arg in graph[function].user_data.function_arguments.clone() {
        let typ = pulse_value_type_to_node_types(&arg.typ).0;
        if let Ok(output) = graph[function].get_output(&arg.name) {
            graph[output].typ = typ;
        } else {
            graph.add_output_param(function, arg.name, typ);
        }
    }
    for caller in callers(graph, function) {
        sync_call(graph, caller, function);
    }
}

pub(super) fn sync_call(graph: &mut PulseGraph, caller: NodeId, function: NodeId) {
    let args = graph[function].user_data.function_arguments.clone();
    // Only remove ports absent from the new target; preserve action wires and Async.
    let obsolete: Vec<_> = graph[caller].inputs.iter()
        .filter(|(name, _)| !["nodeId", "ActionIn", "Async"].contains(&name.as_str())
            && !args.iter().any(|arg| &arg.name == name)).map(|(_, id)| *id).collect();
    for input in obsolete { graph.remove_input_param(input); }
    let obsolete: Vec<_> = graph[caller].outputs.iter()
        .filter(|(name, _)| name != "outAction").map(|(_, id)| *id).collect();
    for output in obsolete { graph.remove_output_param(output); }
    if graph[caller].get_input("ActionIn").is_err() {
        graph.add_input_param(caller, "ActionIn".into(), PulseDataType::Action,
            PulseGraphValueType::Action, InputParamKind::ConnectionOnly, true);
    }
    if graph[caller].get_input("Async").is_err() {
        graph.add_input_param(caller, "Async".into(), PulseDataType::Bool,
            PulseGraphValueType::Bool { value: false }, InputParamKind::ConstantOnly, true);
    }
    if graph[caller].get_output("outAction").is_err() {
        graph.add_output_param(caller, "outAction".into(), PulseDataType::Action);
    }
    for arg in args {
        let (typ, default) = pulse_value_type_to_node_types(&arg.typ);
        let kind = get_preffered_inputparamkind_from_type(&arg.typ);
        if let Ok(input) = graph[caller].get_input(&arg.name) {
            let input = &mut graph[input];
            if input.typ != typ {
                input.value = match (&input.value, default) {
                    (PulseGraphValueType::Scalar { value }, PulseGraphValueType::Integer { .. }) =>
                        PulseGraphValueType::Integer { value: *value as i32 },
                    (PulseGraphValueType::Integer { value }, PulseGraphValueType::Scalar { .. }) =>
                        PulseGraphValueType::Scalar { value: *value as f32 },
                    (_, default) => default,
                };
                input.typ = typ;
            }
            input.kind = kind;
        } else {
            graph.add_input_param(caller, arg.name, typ, default, kind, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::FullGraphState;

    #[test]
    fn signature_edits_preserve_ports_values_and_connections_and_survive_save() {
        let mut graph = PulseGraph::default();
        let function = graph.add_node("Function".into(), PulseNodeData {
            template: PulseNodeTemplate::Function, ..Default::default()
        }, |_, _| {});
        let caller = graph.add_node("Call".into(), PulseNodeData {
            template: PulseNodeTemplate::CallNode, ..Default::default()
        }, |_, _| {});
        graph.add_input_param(caller, "nodeId".into(), PulseDataType::NoideChoice,
            PulseGraphValueType::NodeChoice { node: Some(function) }, InputParamKind::ConstantOnly, true);
        add_argument(&mut graph, function);
        let output = graph[function].get_output("arg1").unwrap();
        let input = graph[caller].get_input("arg1").unwrap();
        graph[input].value = PulseGraphValueType::Scalar { value: 42.75 };
        let action = graph[caller].get_input("ActionIn").unwrap();
        let async_input = graph[caller].get_input("Async").unwrap();
        graph[async_input].value = PulseGraphValueType::Bool { value: true };
        // These connections test preservation only, not execution of this recursive graph.
        graph.add_connection(output, input, 0);
        edit_argument(&mut graph, function, 0, Some(FunctionArgument {
            name: "count".into(), typ: PulseValueType::PVAL_INT(None),
        }));
        assert_eq!(graph[function].get_output("count").unwrap(), output);
        assert_eq!(graph[caller].get_input("count").unwrap(), input);
        assert_eq!(graph[input].value, PulseGraphValueType::Integer { value: 42 });
        assert_eq!(graph[input].typ, PulseDataType::Integer);
        assert_eq!(graph[output].typ, PulseDataType::Integer);
        assert_eq!(graph.connection(input), Some(output));
        assert_eq!(graph[caller].get_input("ActionIn").unwrap(), action);
        assert_eq!(graph[async_input].value, PulseGraphValueType::Bool { value: true });
        add_argument(&mut graph, function);
        assert_eq!(graph[function].user_data.function_arguments.len(), 2);
        // Reject collisions with another argument and with control inputs.
        for name in ["count", "Async", "two words", "", "2arg"] {
            edit_argument(&mut graph, function, 1, Some(FunctionArgument {
                name: name.into(), typ: PulseValueType::PVAL_FLOAT(None),
            }));
            assert_eq!(graph[function].user_data.function_arguments[1].name, "arg1");
        }
        let mut saved = FullGraphState::default();
        saved.state.graph = graph;
        let text = ron::to_string(&saved).unwrap();
        let mut loaded: FullGraphState = ron::from_str(&text).unwrap();
        super::super::migrations::verify_compat(&mut loaded);
        let graph = &mut loaded.state.graph;
        assert_eq!(graph[function].user_data.function_arguments[0].name, "count");
        assert_eq!(graph.connection(input), Some(output));
        assert_eq!(graph[input].value, PulseGraphValueType::Integer { value: 42 });
        edit_argument(graph, function, 0, None);
        assert!(graph.inputs.get(input).is_none());
        assert!(graph.outputs.get(output).is_none());
        assert!(graph[caller].get_input("count").is_err());
        assert!(graph[caller].get_input("arg1").is_ok());
    }

    #[test]
    fn old_node_data_defaults_to_no_function_arguments() {
        let original = PulseNodeData { template: PulseNodeTemplate::Function, ..Default::default() };
        let text = ron::to_string(&original).unwrap().replace(",function_arguments:[]", "");
        assert!(!text.contains("function_arguments"));
        let restored: PulseNodeData = ron::from_str(&text).unwrap();
        assert!(restored.function_arguments.is_empty());
    }
}
