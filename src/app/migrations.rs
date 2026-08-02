use eframe::egui;
use egui_node_graph2::{InputParamKind, NodeId};
use ron::{Value, value::{Map, Number, F32}};

use crate::{app::{FullGraphState, types::{PulseDataType, PulseGraphValueType, PulseNodeTemplate, pulse_value_type_to_node_types}}, pulsetypes::{GeneralEnumChoice, SoundEventStartType}};
use crate::typing::{get_preffered_inputparamkind_from_type};

// This is currently unused due to issues with RON deserializing into Value type without losing version information
#[allow(dead_code)]
fn migrate_v2(editor: &mut Value) -> anyhow::Result<()> {
    // new entry node_sized needs to be populated with node_ids in the graph.
    if let Value::Map(state) = editor {
        let nodes = &state[&Value::String("node_order".to_string())];
        if let Value::Map(nodes) = nodes {
            let mut node_sizes = Value::Map(Map::new());
            let node_sizes_map = match &mut node_sizes {
                Value::Map(map) => map,
                _ => unreachable!("Can't be not a map"),
            };
            for (version, _) in nodes.iter() {
                let mut sizes_map = Map::new();
                sizes_map.insert(
                    Value::String("x".into()),
                    Value::Number(Number::F32(F32::new(200.0)))
                );
                sizes_map.insert(
                    Value::String("y".into()),
                    Value::Number(Number::F32(F32::new(200.0)))
                );
                let opt = Value::Option(Some(Box::from(Value::Map(sizes_map))));
                node_sizes_map.insert(Value::String("value".into()), opt);
                node_sizes_map.insert(Value::String("version".into()), version.clone());
            }
            state.insert(ron::Value::String("node_sizes".to_string()), node_sizes);
        }
    }
    Ok(())
}


pub fn verify_compat(full_state: &mut FullGraphState) {
    // v0.1.1 introduces a SecondaryMap node_sizes in GraphEditorState
    // make sure that it is populated with every existing node.
    if full_state.state.node_sizes.is_empty() {
        for node in full_state.state.graph.nodes.iter() {
            full_state.state.node_sizes.insert(node.0, egui::vec2(200.0, 200.0));
        }
    }
    let mut sound_event_nodes = vec![];
    let mut entfire_nodes = vec![];
    let mut call_func_nodes = vec![];
    let mut listen_entity_output_nodes = vec![];
    struct QueuedAddParams {
        node_id: NodeId,
        param_name: String,
        types: (PulseDataType, PulseGraphValueType),
        connection_type: InputParamKind,
    }
    let mut queued_add_params: Vec<QueuedAddParams> = vec![];
    for node_id in full_state.state.graph.iter_nodes().collect::<Vec<_>>() {
        let node = match full_state.state.graph.nodes.get_mut(node_id) {
            Some(node) => node,
            None => continue,
        };  
        let template = node.user_data.template;
        match template {
            // verify that all existing library binding nodes have correct parameters, in case they have been updated between sessions.
            // NOTE: this does not remove any parameters from the node, they would be just ignored.
            PulseNodeTemplate::LibraryBindingAssigned { binding } => {
                if let Some(binding) = full_state.user_state.bindings.find_function_by_id(binding) {
                    if binding.inparams.is_none() {
                        continue;
                    }
                    let mut inputs = node.inputs.iter_mut().filter(|input| {
                        let nam_lowercase = input.0.to_lowercase();
                        !nam_lowercase.contains("action") && !nam_lowercase.contains("binding")
                    }).collect::<Vec<_>>();

                    for (idx, param) in binding.inparams.as_ref().unwrap().iter().enumerate() {
                        if idx < inputs.len() {
                            // Safety: we checked the length above
                            inputs[idx].0 = param.name.clone();
                        } else {
                            // quque up missing parameters to be added after the loop to avoid borrow checker issues
                            queued_add_params.push(QueuedAddParams { 
                                node_id,
                                param_name: param.name.clone(),
                                types: pulse_value_type_to_node_types(&param.pulsetype),
                                connection_type: get_preffered_inputparamkind_from_type(&param.pulsetype) 
                            });
                        }
                    }
                }
            }
            // v0.3.1 we added sound event source input.
            PulseNodeTemplate::SoundEventStart
                // if the input is not present, add it to a list, and then add the input later
                // can't do it here because of borrow checker
                if node.get_input("soundEventType").is_err() => {
                    sound_event_nodes.push(node_id);
                }
            // v0.3.1 Added entity handle input to EntFire
            PulseNodeTemplate::EntFire
                if node.get_input("entityHandle").is_err() => {
                    entfire_nodes.push(node_id);
                }
            // v0.3.1 Added Async fire mode to Call Node for functions
            PulseNodeTemplate::CallNode
                if node.get_input("Async").is_err() => {
                    let target_node_id = node
                        .get_input("nodeId")
                        .ok()
                        .and_then(|input_id| {
                            full_state.state.graph.get_input(input_id).value().clone().try_node_id().ok()
                        });

                    if let Some(target_node_id) = target_node_id {
                        if let Some(target_node) = full_state.state.graph.nodes.get(target_node_id) {
                            match target_node.user_data.template {
                                PulseNodeTemplate::Function => { call_func_nodes.push(node_id); },
                                PulseNodeTemplate::ListenForEntityOutput => { listen_entity_output_nodes.push(node_id); },
                                _ => {}
                            }
                        }
                    }
                }
            _ => (),
        }
    }
    for node_id in sound_event_nodes {
        full_state.state.graph.add_input_param(
            node_id,
            "soundEventType".to_string(),
            PulseDataType::GeneralEnum,
            PulseGraphValueType::GeneralEnumChoice {
                value: GeneralEnumChoice::SoundEventStartType(SoundEventStartType::default())
            },
            InputParamKind::ConstantOnly,
            true,
        );
        // TODO: would be good to have some publically accessible simplifications for adding common inputs
        full_state.state.graph.add_input_param(
            node_id,
            "ActionIn".to_string(),
            PulseDataType::Action,
            PulseGraphValueType::Action,
            InputParamKind::ConnectionOnly,
            true,
        );
        full_state.state.graph.add_output_param(node_id, "outAction".to_string(), PulseDataType::Action);
        // all of this below is just to move the input action to the top, since the library doesn't really make that easy.
        let node = full_state.state.graph.nodes.get_mut(node_id).unwrap();
        let mut input_id = None;
        node.inputs.retain(|input| {
            input_id = Some(input.1);
            input.0 != "ActionIn"
        });
        if let Some(input_id) = input_id {
            node.inputs.insert(0,("ActionIn".to_string(), input_id));
        }
    }
    for node_id in entfire_nodes {
        full_state.state.graph.add_input_param(
            node_id,
            "entityHandle".to_string(),
            PulseDataType::EHandle,
            PulseGraphValueType::EHandle,
            InputParamKind::ConnectionOnly,
            true,
        );
    }
    for node_id in call_func_nodes {
        full_state.state.graph.add_input_param(
            node_id,
            "Async".to_string(),
            PulseDataType::Bool,
            PulseGraphValueType::Bool { value: false },
            InputParamKind::ConstantOnly,
            true,
        );
    }
    for node_id in listen_entity_output_nodes {
        let node = full_state.state.graph.nodes.get_mut(node_id).unwrap();
        if let Ok(o) = node.get_output("outAction") { 
            full_state.state.graph.remove_output_param(o);
        }
    }

    for param in queued_add_params {
        full_state.state.graph.add_input_param(
            param.node_id,
            param.param_name,
            param.types.0,
            param.types.1,
            param.connection_type,
            true,
        );
    }

    // this fills out the default domain and subdomain if they're not set at launch time
    if full_state.user_state.graph_domain.is_empty() {
        full_state.user_state.graph_domain = "ServerEntity".to_string();
    }
    if full_state.user_state.graph_subtype.is_empty() {
        full_state.user_state.graph_subtype = "PVAL_EHANDLE:point_pulse".to_string();
    }


    // variables and outputs use PulseDataType now
    for var in full_state.user_state.variables.iter_mut() {
        let types = pulse_value_type_to_node_types(&var.typ_and_default_value);
        var.data_type = types.0;
        var.stored_value = types.1;
    }

    for output in full_state.user_state.public_outputs.iter_mut() {
        let types = pulse_value_type_to_node_types(&output.typ);
        output.data_type = types.0;
        output.value_type = types.1;
    }
}