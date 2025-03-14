use crate::app::{
    PulseDataType, PulseGraph, PulseGraphState, PulseGraphValueType, PulseNodeData,
    PulseNodeTemplate,
};
use crate::instruction_templates;
use crate::pulsetypes::*;
use crate::serialization::*;
use egui_node_graph2::*;
use std::fs;

const PULSE_KV3_HEADER: &str = "<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:vpulse12:version{354e36cb-dbe4-41c0-8fe3-2279dd194022} -->\n";
macro_rules! graph_next_action {
    ($graph:ident, $current_node:ident, $graph_def:ident, $target_chunk:ident) => {
        let connected_node = get_next_action_node($current_node, $graph, "outAction");
        if connected_node.is_some() {
            return traverse_nodes_and_populate(
                $graph,
                connected_node.unwrap(),
                $graph_def,
                $target_chunk,
                &None,
            );
        }
    };
}

macro_rules! get_constant_graph_input_value {
    ($graph:ident, $node:ident, $input:literal, $typ_func:ident) => {{
        let input_id = $node
            .get_input($input)
            .expect(format!("Can't find input {}", $input).as_str());
        let input_param = $graph.inputs.get(input_id).expect("Can't find input value");
        input_param
            .value
            .clone()
            .$typ_func()
            .expect("Failed to unwrap input value")
    }};
}

fn get_connected_output_node(graph: &PulseGraph, out_action_id: &OutputId) -> Option<NodeId> {
    // dumb way of finding outgoing connection node.
    for group in graph.iter_connection_groups() {
        for connection in group.1 {
            if connection == *out_action_id {
                let input_action: &InputParam<PulseDataType, PulseGraphValueType> =
                    graph.inputs.get(group.0).expect("Can't find input value");
                return Some(input_action.node);
            }
        }
    }
    None
}

fn get_next_action_node<'a>(
    origin_node: &'a Node<PulseNodeData>,
    graph: &'a PulseGraph,
    name: &str,
) -> Option<&'a Node<PulseNodeData>> {
    let out_action_id = origin_node.get_output(name);
    if out_action_id.is_ok() {
        let out_action_id = out_action_id.unwrap();
        let connected_node_id = get_connected_output_node(graph, &out_action_id);
        if connected_node_id.is_some() {
            return graph.nodes.get(connected_node_id.unwrap());
        }
    }
    return None;
}

// process all inflow nodes and logic chain.
// returns false if no inflow node was processed
fn traverse_inflow_nodes(graph: &PulseGraph, graph_def: &mut PulseGraphDef) -> bool {
    let mut processed: bool = false;
    for node in graph.iter_nodes() {
        let data: &Node<PulseNodeData> = graph.nodes.get(node).unwrap();
        // start at all possible entry points

        match data.user_data.template {
            PulseNodeTemplate::EventHandler => {
                processed = true;
                traverse_event_cell(graph, &data, graph_def)
            }
            PulseNodeTemplate::CellPublicMethod => {
                processed = true;
                traverse_entry_cell(graph, &data, graph_def)
            }
            PulseNodeTemplate::GraphHook => {
                processed = true;
                traverse_graphhook_cell(graph, &data, graph_def)
            }
            _ => {}
        }
    }
    processed
}

fn traverse_event_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
) {
    let input_id = node
        .get_input("eventName")
        .expect("Can't find input 'eventName'");
    let input_param = graph.inputs.get(input_id).expect("Can't find input value");
    let event_name = input_param.value.clone().try_to_string().unwrap();
    // create new pulse cell node.
    let chunk_id = graph_def.create_chunk();
    let mut cell_event = CPulseCell_Inflow_EventHandler::new(chunk_id, event_name);

    for (output_id, name) in node.user_data.custom_named_outputs.iter() {
        let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
        let reg_id = chunk.add_register(String::from("PVAL_INT"), 0);
        cell_event.add_outparam(name.clone(), reg_id);
        graph_def.add_register_mapping(*output_id, reg_id);
    }
    graph_def.cells.push(Box::from(cell_event));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
}

fn traverse_graphhook_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
) {
    let hook_name = get_constant_graph_input_value!(graph, node, "hookName", try_to_string);
    let chunk_id = graph_def.create_chunk();
    let cell_hook = CPulseCell_Inflow_GraphHook::new(hook_name, RegisterMap::default(), chunk_id);
    graph_def.cells.push(Box::from(cell_hook));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
}

fn traverse_entry_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
) {
    let mut cell_method = CPulseCell_Inflow_Method::default();
    let chunk_id = graph_def.create_chunk();
    cell_method.name = get_constant_graph_input_value!(graph, node, "name", try_to_string);
    cell_method.entry_chunk = chunk_id;
    cell_method.return_type = String::from("PVAL_INVALID");

    //let out_action_param = graph.outputs.get(out_action_id).expect("Can't find output value");
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    // create argument1 (TODO only if connection exists)
    let reg_id_arg1 = chunk.add_register(String::from("PVAL_STRING"), 0);
    let output_id_arg1 = node
        .get_output("argument1")
        .expect("Can't find output 'argument1'");
    cell_method.add_arg(
        String::from("arg1"),
        String::default(),
        String::from("PVAL_STRING"),
        reg_id_arg1,
    );
    graph_def.add_register_mapping(output_id_arg1, reg_id_arg1);

    graph_def.cells.push(Box::from(cell_method));
    // get action connection
    let out_action_id = node
        .get_output("outAction")
        .expect("Can't find output 'outAction'");
    let connected_node_id = get_connected_output_node(graph, &out_action_id);
    if connected_node_id.is_some() {
        let connected_node = graph.nodes.get(connected_node_id.unwrap());
        if connected_node.is_some() {
            traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
        }
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
}

pub fn compile_graph<'a>(graph: &PulseGraph, graph_state: &PulseGraphState) -> Result<(), String> {
    let mut graph_def = PulseGraphDef::default();
    graph_def.variables = graph_state.variables.clone();
    graph_def.public_outputs = graph_state.public_outputs.clone();
    graph_def.map_name = String::from("maps/main.vmap");
    graph_def.xml_name = String::default();

    if !traverse_inflow_nodes(graph, &mut graph_def) {
        return Err("No inflow nodes found in graph".to_string());
    }
    let mut data = String::from(PULSE_KV3_HEADER);
    data.push_str(graph_def.serialize().as_str());
    let file_dir = graph_state.save_file_path.as_path().parent();
    if file_dir.is_none() {
        return Err("Output file path is set incorrectly".to_string());
    }
    let file_dir = file_dir.unwrap();
    let dir_res = fs::create_dir_all(file_dir);
    if dir_res.is_err() {
        return Err(format!(
            "Failed to create output directory: {}",
            dir_res.err().unwrap()
        ));
    }
    fs::write(graph_state.save_file_path.as_path(), data)
        .map_err(|e| format!("Failed to write to file: {}", e))
}

fn try_find_output_mapping(graph_def: &PulseGraphDef, output_id: &Option<OutputId>) -> i32 {
    match output_id {
        Some(output_id) => {
            match graph_def.get_mapped_reigster(*output_id) {
                Some(reg) => {
                    // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => {
                    return -1;
                }
            }
        }
        None => {
            return -1;
        }
    }
}

fn get_variable(graph_def: &mut PulseGraphDef, name: &str) -> Option<i32> {
    let var = graph_def.get_variable_index(name);
    if var.is_some() {
        return Some(var.unwrap() as i32);
    }
    None
}

fn try_find_input_mapping(graph_def: &PulseGraphDef, input_id: &Option<InputId>) -> i32 {
    match input_id {
        Some(input_id) => {
            match graph_def.get_mapped_reigster_input(*input_id) {
                Some(reg) => {
                    // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => {
                    return -1;
                }
            }
        }
        None => {
            return -1;
        }
    }
}

// traverse to the neihbors of the current node, connected to inputs, and gather their information
// can choose if the generated value from the input will be reused, or if it should always be evaluated as new
// as a new input (depends on the task of the node really)
fn get_input_register_or_create_constant(
    graph: &PulseGraph,
    current_node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    chunk_id: i32,
    input_name: &str,
    value_type: PulseValueType,
    always_reevaluate: bool,
) -> i32 {
    let input_id = current_node
        .get_input(input_name)
        .expect(format!("Can't find input {}", input_name).as_str());
    let connection_to_input: Option<OutputId> = graph.connection(input_id);
    let mut target_register: i32;
    // if we find a connection, then traverse to that node, whatever happens we should get a register id back.
    match connection_to_input {
        Some(out) => {
            // connection found to an outputid of the connected node. Traverse to that node, and get the register
            let out_param = graph.get_output(out);
            let out_node = graph
                .nodes
                .get(out_param.node)
                .expect("Can't find output node");
            target_register =
                traverse_nodes_and_populate(graph, out_node, graph_def, chunk_id, &Some(out));
        }
        None => {
            if !always_reevaluate {
                // no connection found, create a constant value for the input
                // but first check if we have already created a constant for this value
                target_register = try_find_input_mapping(graph_def, &Some(input_id));
                if target_register != -1 {
                    return target_register;
                }
            }
            let new_constant_id = graph_def.get_current_constant_id() + 1;
            let new_domain_val_id = graph_def.get_current_domain_val_id() + 1;
            let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
            target_register =
                chunk.add_register(value_type.to_string(), chunk.get_last_instruction_id() + 1);
            let input_param = graph.get_input(input_id);

            let instruction: Instruction;
            match value_type {
                PulseValueType::PVAL_INT(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_scalar()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Integer(input_value as i32));
                }
                PulseValueType::PVAL_FLOAT(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_scalar()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Float(input_value));
                }
                PulseValueType::PVAL_STRING(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_string()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::String(input_value));
                }
                PulseValueType::DOMAIN_ENTITY_NAME => {
                    instruction =
                        instruction_templates::get_domain_value(target_register, new_domain_val_id);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_entity_name()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.create_domain_value(
                        String::from("ENTITY_NAME"),
                        input_value.clone(),
                        String::new(),
                    );
                }
                PulseValueType::PVAL_VEC3(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_vec3()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Vec3(input_value));
                }
                PulseValueType::PVAL_COLOR_RGB(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_vec3()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Color_RGB(input_value));
                }
                _ => panic!("Unsupported value type"),
            };
        }
    }
    target_register
}

// recurse along connected nodes, and generate instructions, cells, and bindings depending on the node type.
// takes care of referencing already assigned registers or other data (like visisted list in a graph traversal)
// it operates ONLY on a target chunk - which is basically a set of instructions related to one flow of logic
// inside the GUI a chunk is one continous flow of logic.
fn traverse_nodes_and_populate(
    graph: &PulseGraph,
    current_node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    target_chunk: i32,
    output_id: &Option<OutputId>, // if this is Some, then this was called by a node requesting a value, (not action)
) -> i32 {
    match current_node.user_data.template {
        PulseNodeTemplate::CellPublicMethod => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        PulseNodeTemplate::EventHandler => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        PulseNodeTemplate::CellWait => {
            let time_input_id = current_node
                .get_input("time")
                .expect("Can't find input 'time'");
            let connection_to_time = graph.connection(time_input_id);
            let time_input_register: i32;
            match connection_to_time {
                Some(out) => {
                    // get existing register id to note for the wait time
                    let out_param = graph.get_output(out);
                    let out_node = graph
                        .nodes
                        .get(out_param.node)
                        .expect("Can't find output node");
                    time_input_register = traverse_nodes_and_populate(
                        graph,
                        out_node,
                        graph_def,
                        target_chunk,
                        &Some(out),
                    );
                }
                None => {
                    // create a constant value for the wait time
                    // have to do this way because of the borrow checker's mutability rules.
                    let new_constant_id = graph_def.get_current_constant_id() + 1;

                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    // add a new register to store the wait time
                    time_input_register = chunk.add_register(
                        String::from("PVAL_FLOAT"),
                        chunk.get_last_instruction_id() + 1,
                    );
                    // get the wait time constant
                    let instruction =
                        instruction_templates::get_const(new_constant_id, time_input_register);
                    chunk.add_instruction(instruction);
                    // add the actual constant definition for the waiting time
                    let value_param = graph.get_input(time_input_id);
                    let time = value_param
                        .value()
                        .clone()
                        .try_to_scalar()
                        .expect("Failed to unwrap input value");
                    let constant = PulseConstant::Float(time);
                    graph_def.add_constant(constant);
                }
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // ! Important (might change). We assume that after waiting we go to the next instruction after the cell invoke.
            let cell_wait =
                CPulseCell_Inflow_Wait::new(target_chunk, chunk.get_last_instruction_id() + 3);
            graph_def.cells.push(Box::from(cell_wait));

            let chunk_opt = graph_def.chunks.get(target_chunk as usize);
            if chunk_opt.is_some() {
                let chunk = chunk_opt.unwrap();
                let mut register_map = RegisterMap::default();
                register_map.add_inparam("flDurationSec", time_input_register);
                let binding = InvokeBinding {
                    register_map,
                    func_name: "Wait",
                    cell_index: graph_def.cells.len() as i32 - 1,
                    src_chunk: target_chunk,
                    src_instruction: chunk.get_last_instruction_id() + 1,
                };
                let binding_idx = graph_def.add_invoke_binding(binding);
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::cell_invoke(binding_idx));
                // early return.
                let mut instr_ret_void = Instruction::default();
                instr_ret_void.code = String::from("RETURN_VOID");
                chunk.add_instruction(instr_ret_void);
            }

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::EntFire => {
            let reg_entity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "entity",
                PulseValueType::DOMAIN_ENTITY_NAME,
                false,
            );
            let input_value = get_constant_graph_input_value!(graph, current_node, "input", try_to_string);
            // this one might be empty, but we want to use it for OutputConnection if we know it at compile time.
            let entity_name_static_value = get_constant_graph_input_value!(graph, current_node, "entity", try_entity_name); 

            // check for existence of the parameter value (connection, or non empty string)
            // to determine if we need to add it to the EntFire call
            let param_value_exists = 'checkParmValue: {
                let input_id = current_node
                    .get_input("value")
                    .expect("Can't find input value");
                let connection_to_input: Option<OutputId> = graph.connection(input_id);
                if connection_to_input.is_some() {
                    break 'checkParmValue true;
                }

                let val = graph.get_input(input_id).value().clone().try_to_string().unwrap();
                break 'checkParmValue !val.is_empty();
            };

            let mut reg_param = -1;
            if param_value_exists {
                reg_param = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    target_chunk,
                    "value",
                    PulseValueType::PVAL_STRING(None),
                    false,
                );
            }
            // create EntFire (step) cell
            let step_cell = CPulseCell_Step_EntFire::new(input_value.clone());
            graph_def.cells.push(Box::from(step_cell));
            // now build instructions and bindings to get the domain value, and invoke the cell
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();

            // add invoke binding for FireAtName cell
            let mut register_map = RegisterMap::default();
            register_map.add_inparam("TargetName", reg_entity);
            if param_value_exists {  
                register_map.add_inparam("pParam", reg_param);
            }
            let binding = InvokeBinding {
                register_map: register_map,
                func_name: "FireAtName",
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            let binding_idx = graph_def.add_invoke_binding(binding);
            // add instruction for invoking the binding.
            // rust doesn't like reusing the borrowed chunks reference, but we know that it doesn't change.
            graph_def
                .chunks
                .get_mut(target_chunk as usize)
                .unwrap()
                .add_instruction(instruction_templates::cell_invoke(binding_idx));

            let output_connection = OutputConnection::new(
                String::from("Step_EntFire:-1"),
                entity_name_static_value,
                input_value,
                if param_value_exists {
                    String::from("param")
                } else {
                    String::default()
                },
            );
            graph_def.add_output_connection(output_connection);

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::ConcatString => {
            let id_a = current_node
                .get_input("A")
                .expect("Can't find input A in node");
            let id_b = current_node
                .get_input("B")
                .expect("Can't find input B in node");
            let input_ids = [id_a, id_b];
            let connection_to_a = graph.connection(id_a);
            let connection_to_b = graph.connection(id_b);
            let connections_to_resolve: [Option<OutputId>; 2] = [connection_to_a, connection_to_b];
            let mut input_registers: [i32; 2] = [-1, -1];

            for (i, connection) in connections_to_resolve.iter().enumerate() {
                match connection {
                    Some(out) => {
                        let out_param = graph.get_output(*out);
                        let out_node = graph
                            .nodes
                            .get(out_param.node)
                            .expect("Can't find output node");
                        // grab the register that the value will come from.
                        input_registers[i] = traverse_nodes_and_populate(
                            graph,
                            out_node,
                            graph_def,
                            target_chunk,
                            &Some(*out),
                        );
                    }
                    None => {
                        // no connection.. First search if we already created it, if not create the constant input value
                        let register = try_find_input_mapping(graph_def, &Some(input_ids[i]));
                        if register == -1 {
                            let input_info: &InputParam<PulseDataType, PulseGraphValueType> =
                                graph.get_input(input_ids[i]);
                            let constant = PulseConstant::String(
                                input_info.value.clone().try_to_string().unwrap(),
                            );
                            let const_idx = graph_def.add_constant(constant);
                            // create register to hold this value
                            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                            input_registers[i] = chunk.add_register(
                                String::from("PVAL_STRING"),
                                chunk.get_last_instruction_id() + 1,
                            );
                            // create instruction to load this value now.
                            let instruction =
                                instruction_templates::get_const(const_idx, input_registers[i]);
                            chunk.add_instruction(instruction);
                            graph_def.add_register_mapping_input(input_ids[i], input_registers[i]);
                        } else {
                            input_registers[i] = register;
                        }
                    }
                }
            }
            // registers are figured out. now prepare the output register and the instruction
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register = chunk.add_register(
                    String::from("PVAL_STRING"),
                    chunk.get_last_instruction_id() + 1,
                );
                let instruction = instruction_templates::add_string(
                    input_registers[0],
                    input_registers[1],
                    register,
                );
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        PulseNodeTemplate::GetVar => {
            let name_id = current_node
                .get_input("variableName")
                .expect("Can't find input 'variableName'");
            // name is a constant value
            let name = graph
                .get_input(name_id)
                .value()
                .clone()
                .try_variable_name()
                .expect("Can't find variableName parameter");
            let var_id = get_variable(graph_def, name.as_str());
            if var_id.is_none() {
                panic!("Variable {name} not found in list, it should be here!");
            }
            let typ = graph_def
                .variables
                .get(var_id.unwrap() as usize)
                .unwrap()
                .typ_and_default_value
                .to_string();
            // add register
            // add instruction to load the variable value
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg = chunk.add_register(typ, chunk.get_last_instruction_id() + 1);
            chunk.add_instruction(instruction_templates::get_var(reg, var_id.unwrap()));
            return reg;
        }
        PulseNodeTemplate::IntToString => {
            let value_id = current_node
                .get_input("value")
                .expect("Can't find input 'value'");
            let connection_to_value = graph.connection(value_id);
            let register_input: i32;
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph
                        .nodes
                        .get(out_param.node)
                        .expect("Can't find output node");
                    // grab the register that the value will come from.
                    register_input = traverse_nodes_and_populate(
                        graph,
                        out_node,
                        graph_def,
                        target_chunk,
                        &Some(out),
                    );
                }
                None => {
                    print!("No connection found for input value for IntToString node");
                    return -1;
                }
            }
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register = chunk.add_register(
                    String::from("PVAL_STRING"),
                    chunk.get_last_instruction_id() + 1,
                );
                let instruction = instruction_templates::convert_value(register, register_input);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        PulseNodeTemplate::SetVar => {
            let name_id = current_node
                .get_input("variableName")
                .expect("Can't find input 'variableName'");
            // name is a constant value
            let name = graph
                .get_input(name_id)
                .value()
                .clone()
                .try_variable_name()
                .expect("Can't find variableName parameter");
            let var_id = get_variable(graph_def, name.as_str());
            if var_id.is_none() {
                panic!("Variable {name} not found in list, it should be here!");
            }
            let typ = graph_def
                .variables
                .get(var_id.unwrap() as usize)
                .unwrap()
                .typ_and_default_value
                .clone();
            let reg_value = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "value",
                typ,
                false,
            );
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            chunk.add_instruction(instruction_templates::set_var(reg_value, var_id.unwrap()));

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::Operation => {
            let operation_typ =
                get_constant_graph_input_value!(graph, current_node, "type", try_pulse_type);
            let reg_a = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "A",
                operation_typ.clone(),
                false,
            );
            let reg_b = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "B",
                operation_typ.clone(),
                false,
            );
            let operation_input_param =
                get_constant_graph_input_value!(graph, current_node, "operation", try_to_string);
            let operation_suffix = operation_typ.get_operation_suffix_name();
            let operation_instr_name: String = match operation_input_param.as_str() {
                "+" => format!("ADD{}", operation_suffix),
                "-" => format!("SUB{}", operation_suffix),
                "*" => format!("MUL{}", operation_suffix),
                "/" => format!("DIV{}", operation_suffix),
                "%" => format!("MOD{}", operation_suffix),
                _ => format!("ADD{}", operation_suffix),
            };
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let register_output = chunk.add_register(
                String::from(operation_typ.to_string()),
                chunk.get_last_instruction_id() + 1,
            );
            let mut instr = Instruction::default();
            instr.code = operation_instr_name;
            instr.reg0 = register_output;
            instr.reg1 = reg_a;
            instr.reg2 = reg_b;
            chunk.add_instruction(instr);
            return register_output;
        }
        PulseNodeTemplate::FindEntByName => {
            let reg_entname = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "entName",
                PulseValueType::DOMAIN_ENTITY_NAME,
                false,
            );
            let entclass_input_id = current_node
                .get_input("entClass")
                .expect("Can't find input 'entClass'");
            let entclass_input_param = graph
                .get_input(entclass_input_id)
                .value()
                .clone()
                .try_to_string()
                .expect("Can't find input 'entClass'");
            let new_binding_idx = graph_def.get_current_binding_id() + 1;
            let mut register_map = RegisterMap::default();
            let mut reg_output = try_find_output_mapping(graph_def, output_id);
            if reg_output == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                reg_output = chunk.add_register(
                    PulseValueType::PVAL_EHANDLE(Some(entclass_input_param.clone())).to_string(),
                    chunk.get_last_instruction_id() + 1,
                );
                if let Some(out) = output_id {
                    graph_def.add_register_mapping(*out, reg_output);
                }
            } else {
                return reg_output;
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let cell = CPulseCell_Value_FindEntByName::new(entclass_input_param);
            graph_def.cells.push(Box::from(cell));
            register_map.add_inparam("pName", reg_entname);
            register_map.add_outparam(String::from("retval"), reg_output);
            let instr = chunk.add_instruction(instruction_templates::cell_invoke(new_binding_idx));
            let binding = InvokeBinding {
                register_map,
                func_name: "Eval",
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: instr,
            };
            graph_def.add_invoke_binding(binding);
            return reg_output;
        }
        PulseNodeTemplate::DebugWorldText => {
            let reg_message = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "pMessage",
                PulseValueType::PVAL_STRING(None),
                false,
            );
            // resolve connection to hEntity
            let hentity_input_id = current_node
                .get_input("hEntity")
                .expect("Can't find input 'value'");
            let connection_to_hentity = graph.connection(hentity_input_id);
            if connection_to_hentity.is_none() {
                println!("No connection found for hEntity input in DebugWorldText node. Node will not be processed, next action won't execute.");
                return -1;
            }
            let connection_to_hentity = connection_to_hentity.unwrap();
            let hentity_param = graph.get_output(connection_to_hentity);
            let out_node = graph
                .nodes
                .get(hentity_param.node)
                .expect("Can't find output node");
            let reg_hentity = traverse_nodes_and_populate(
                graph,
                out_node,
                graph_def,
                target_chunk,
                &Some(connection_to_hentity),
            );
            // other params
            let reg_ntextoffset = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "nTextOffset",
                PulseValueType::PVAL_INT(None),
                false,
            );
            let reg_flduration = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "flDuration",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            let reg_flverticaloffset = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "flVerticalOffset",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            // color:
            let reg_color = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "color",
                PulseValueType::PVAL_COLOR_RGB(None),
                false,
            );
            let reg_alpha = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "flAlpha",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            let reg_scale = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "flScale",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            // bAttached:
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let attached =
                get_constant_graph_input_value!(graph, current_node, "bAttached", try_to_bool);
            graph_def.add_constant(PulseConstant::Bool(attached));
            // create constant, add instruction and a register to load it into.
            let new_constant_id = graph_def.get_current_constant_id();
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg_battached = chunk.add_register(
                String::from("PVAL_BOOL"),
                chunk.get_last_instruction_id() + 1,
            );
            let instruction = instruction_templates::get_const(new_constant_id, reg_battached);
            chunk.add_instruction(instruction);
            let mut register_map = RegisterMap::default();
            register_map.add_inparam("hEntity", reg_hentity);
            register_map.add_inparam("nTextOffset", reg_ntextoffset);
            register_map.add_inparam("pMessage", reg_message);
            register_map.add_inparam("flDuration", reg_flduration);
            register_map.add_inparam("flVerticalOffset", reg_flverticaloffset);
            register_map.add_inparam("bAttached", reg_battached);
            register_map.add_inparam("color", reg_color);
            register_map.add_inparam("flAlpha", reg_alpha);
            register_map.add_inparam("flScale", reg_scale);
            let binding = InvokeBinding {
                register_map,
                func_name: "CPulseServerFuncs!DebugWorldText",
                cell_index: -1,
                src_chunk: -1,
                src_instruction: -1,
            };
            chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
            graph_def.add_invoke_binding(binding);

            // go to next action.
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::DebugLog => {
            let reg_message = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "pMessage",
                PulseValueType::PVAL_STRING(None),
                false,
            );
            graph_def
                .cells
                .push(Box::from(CPulseCell_Step_DebugLog::default()));
            let mut register_map = RegisterMap::default();
            register_map.add_inparam("pMessage", reg_message);
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let binding = InvokeBinding {
                register_map,
                func_name: "Run",
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            chunk.add_instruction(instruction_templates::cell_invoke(new_binding_id));
            graph_def.add_invoke_binding(binding);

            // go to next action.
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::FireOutput => {
            let input_id = current_node
                .get_input("outputName")
                .expect(format!("Can't find input outputName").as_str());
            let input_val = graph
                .get_input(input_id)
                .value()
                .clone()
                .try_output_name()
                .expect("Failed to unwrap input outputName");
            let pub_output = graph_def.get_public_output_index(input_val.as_str());
            if pub_output.is_some() {
                graph_def
                    .cells
                    .push(Box::from(CPulseCell_Step_PublicOutput::new(
                        pub_output.unwrap() as i32,
                    )));
            }
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::GetGameTime => {
            // create output return value
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg_output = chunk.add_register(
                String::from("PVAL_FLOAT"),
                chunk.get_last_instruction_id() + 1,
            );
            let mut register_map = RegisterMap::default();
            register_map.add_outparam(String::from("retval"), reg_output);
            let binding = InvokeBinding {
                register_map,
                func_name: "CPulseServerFuncs!GetGameTime",
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
            graph_def.add_invoke_binding(binding);
            return reg_output;
        }
        PulseNodeTemplate::SetNextThink => {
            let reg_dt = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "dt",
                PulseValueType::PVAL_STRING(None),
                false,
            );
            let mut register_map = RegisterMap::default();
            register_map.add_inparam("dt", reg_dt);
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let binding = InvokeBinding {
                register_map,
                func_name: "CPulseServerFuncs!SetNextThink",
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
            graph_def.add_invoke_binding(binding);

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::Convert => {
            let type_from =
                get_constant_graph_input_value!(graph, current_node, "typefrom", try_pulse_type);
            let mut type_to =
                get_constant_graph_input_value!(graph, current_node, "typeto", try_pulse_type);
            if type_to == PulseValueType::PVAL_EHANDLE(None) {
                let str_entclass = get_constant_graph_input_value!(
                    graph,
                    current_node,
                    "entityclass",
                    try_to_string
                );
                type_to = PulseValueType::PVAL_EHANDLE(Some(str_entclass));
            }
            let reg_input = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "input",
                type_from.clone(),
                false,
            );
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register =
                    chunk.add_register(type_to.to_string(), chunk.get_last_instruction_id() + 1);
                let instruction = instruction_templates::convert_value(register, reg_input);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        PulseNodeTemplate::Compare => {
            let compare_type =
                get_constant_graph_input_value!(graph, current_node, "type", try_pulse_type);
            let reg_a = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "A",
                compare_type.clone(),
                false,
            );
            let reg_b = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "B",
                compare_type.clone(),
                false,
            );
            // TODO: only EQ for now
            let mut instr_compare = Instruction::default();
            instr_compare.code =
                String::from(format!("EQ{}", compare_type.get_operation_suffix_name()));
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg_cond = chunk.add_register(
                String::from("PVAL_BOOL"),
                chunk.get_last_instruction_id() + 1,
            );
            instr_compare.reg0 = reg_cond;
            instr_compare.reg1 = reg_a;
            instr_compare.reg2 = reg_b;
            chunk.add_instruction(instr_compare);

            // JUMP_COND[reg_cond][trueCondition] (over the next unconditional jump to the false condition)
            // JUMP[falseCondition] (to the false condition)
            // trueCondition
            // ...
            // JUMP[end] (to the end)
            // falseCondition
            // ...
            // end

            let instr_jump_cond =
                instruction_templates::jump_cond(reg_cond, chunk.get_last_instruction_id() + 3);
            chunk.add_instruction(instr_jump_cond);
            let instr_jump_false = instruction_templates::jump(-1); // the id is yet unknown. Note this instruction id, and modify the instruction later.
            let jump_false_instr_id = chunk.add_instruction(instr_jump_false);
            // instruction set for the true condition (if exists)
            let connected_node = get_next_action_node(current_node, graph, "True");
            if connected_node.is_some() {
                traverse_nodes_and_populate(
                    graph,
                    connected_node.unwrap(),
                    graph_def,
                    target_chunk,
                    &None,
                );
            }
            // have to reborrow the chunk after we did borrow of graph_def.
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let false_condition_instr_id = chunk.get_last_instruction_id() + 2;
            // jump over the false condition
            let instr_jump_end = instruction_templates::jump(-1);
            let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

            let connected_node_false = get_next_action_node(current_node, graph, "False");
            if connected_node_false.is_some() {
                traverse_nodes_and_populate(
                    graph,
                    connected_node_false.unwrap(),
                    graph_def,
                    target_chunk,
                    &None,
                );
            }
            // aaand borrow yet again lol
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // for now we just return. But we could have a 3rd port, that executes actions after doing the one in the chosen condition.
            chunk.add_instruction(instruction_templates::return_void());
            let ending_instr_id = chunk.get_last_instruction_id();
            let instr_jump_false = chunk.get_instruction_from_id_mut(jump_false_instr_id);
            if instr_jump_false.is_some() {
                instr_jump_false.unwrap().dest_instruction = false_condition_instr_id;
            } else {
                panic!(
                    "Compare node: Failed to find JUMP[false_condition] with id: {}",
                    jump_false_instr_id
                );
            }
            let instr_jump_end = chunk.get_instruction_from_id_mut(jump_end_instr_id);
            if instr_jump_end.is_some() {
                instr_jump_end.unwrap().dest_instruction = ending_instr_id;
            } else {
                panic!(
                    "Compare node: Failed to find JUMP[end] with id: {}",
                    jump_end_instr_id
                );
            }
        }
        PulseNodeTemplate::ForLoop => {
            if output_id.is_some() {
                let reg_idx = try_find_output_mapping(graph_def, output_id);
                if reg_idx != -1 {
                    return reg_idx;
                } else {
                    println!("[WARN] ForLoop node: Failed to find output register for 'index' when a node requested it.
                    This means that the connected node tried to get the value, before the loop node had it's logic generated by an inflow action.");
                    return -1;
                }
            }
            let reg_from = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "start",
                PulseValueType::PVAL_INT(None),
                false
            );
            let reg_to = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "end",
                PulseValueType::PVAL_INT(None),
                false
            );
            let reg_step = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "step",
                PulseValueType::PVAL_INT(None),
                false
            );
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // new constant = 1 for incrementing
            // new register and constant
            // new reg "idx" (written by yet unknown)
            // copy "from" value to "idx"
            // new reg "cond" (written by following instruction)
            // "idx" LTE "to" "cond"
            // JUMP_COND[cond][curr + 3]
            // JUMP[end] (to the end)
            // body
            // "idx" ADD 1 "idx"
            // JUMP[cond] (to the condition)

            // we don't actually add the register right now, because we know the written_by_instruction id to make one.
            // however the index it will have is known, so we are free to use it, and then actually add it later, once the instruction id is known
            let reg_idx = chunk.add_register(String::from("PVAL_INT"), chunk.get_last_instruction_id() + 1);
            // remember the output index, for nodes that want to access this output
            let output_idx_id = current_node.get_output("index").expect("Can't find output 'idx'");
            graph_def.add_register_mapping(output_idx_id, reg_idx);
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let instr_copy = instruction_templates::copy_value(reg_idx, reg_from);
            chunk.add_instruction(instr_copy);
            let reg_cond = chunk.add_register(String::from("PVAL_BOOL"), chunk.get_last_instruction_id() + 1);
            let mut instr_compare = Instruction::default();
            instr_compare.code = String::from("LTE_INT");
            instr_compare.reg0 = reg_cond;
            instr_compare.reg1 = reg_idx;
            instr_compare.reg2 = reg_to;
            let instr_compare_id = chunk.add_instruction(instr_compare);
            // jump over the unconditional jump to the end
            let instr_jump_cond = instruction_templates::jump_cond(reg_cond, chunk.get_last_instruction_id() + 3);
            chunk.add_instruction(instr_jump_cond);
            let instr_jump_end = instruction_templates::jump(-1);
            let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

            let action_node = get_next_action_node(current_node, graph, "loopAction");
            if action_node.is_some() {
                traverse_nodes_and_populate(
                    graph,
                    action_node.unwrap(),
                    graph_def,
                    target_chunk,
                    &None
                );
            }
            // borrow again (we know that it still is fine)
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // increment the index by step
            let mut instr_add = Instruction::default();
            instr_add.code = String::from("ADD_INT");
            instr_add.reg0 = reg_idx;
            instr_add.reg1 = reg_idx;
            instr_add.reg2 = reg_step;
            chunk.add_instruction(instr_add);
            
            // jump to conditional check
            let instr_jump = instruction_templates::jump(instr_compare_id);
            chunk.add_instruction(instr_jump);
            let end_instr_id = chunk.get_last_instruction_id() + 1;
            let instr_jump_end = chunk.get_instruction_from_id_mut(jump_end_instr_id);
            if instr_jump_end.is_some() {
                instr_jump_end.unwrap().dest_instruction = end_instr_id;
            } else {
                panic!(
                    "ForLoop node: Failed to find JUMP[end] with id: {}",
                    jump_end_instr_id
                );
            }

            let end_action_node = get_next_action_node(current_node, graph, "endAction");
            if end_action_node.is_some() {
                traverse_nodes_and_populate(
                    graph,
                    end_action_node.unwrap(),
                    graph_def,
                    target_chunk,
                    &None
                );
            }
        }
        PulseNodeTemplate::StringToEntityName => {
            let reg_input = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "entityName",
                PulseValueType::PVAL_STRING(None),
                false,
            );
            let mut reg_out = try_find_output_mapping(graph_def, output_id);
            if reg_out == -1 {
                let binding_id = graph_def.get_current_binding_id() + 1; // new binding id, I've put it here because borrow checker
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let mut reg_map = RegisterMap::default();
                reg_out = chunk.add_register(
                    String::from("PVAL_ENTITY_NAME"),
                    chunk.get_last_instruction_id() + 1,
                );
                reg_map.add_inparam("pStr", reg_input);
                reg_map.add_outparam(String::from("retval"), reg_out);
                let invoke_binding = InvokeBinding {
                    register_map: reg_map,
                    func_name: "CPulseServerFuncs!StringToEntityName",
                    cell_index: -1,
                    src_chunk: -1,
                    src_instruction: -1,
                };
                chunk.add_instruction(instruction_templates::library_invoke(binding_id));
                graph_def.add_invoke_binding(invoke_binding);
                graph_def.add_register_mapping(output_id.unwrap(), reg_out);
            }
            return reg_out;
        }
        _ => todo!(
            "Implement node template: {:?}",
            current_node.user_data.template
        ),
    }
    return -1;
}
