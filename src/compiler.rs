use crate::app::{
    PulseDataType, PulseGraph, PulseGraphState, PulseGraphValueType, PulseNodeData,
    PulseNodeTemplate,
};
use crate::typing::get_preffered_inputparamkind_from_type;
use std::borrow::Cow;
use crate::instruction_templates;
use crate::pulsetypes::*;
use crate::serialization::*;
use crate::typing::PulseValueType;
use crate::bindings::LibraryBindingType;
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
                None,
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

macro_rules! get_connection_only_graph_input_value {
    ($graph:ident, $node: ident, $input:literal, $graph_def:ident, $target_chunk:ident, $context:ident) => {{
        let input_id = $node
            .get_input($input)
            .expect(format!("Can't find input {}", $input).as_str());
        let connection = $graph.connection(input_id);
        let result: i32 = if connection.is_some() {
            let connection = connection.unwrap();
            let param = $graph.get_output(connection);
            let out_node = $graph
                .nodes
                .get(param.node)
                .expect("Can't find output node");
            traverse_nodes_and_populate(
                $graph,
                out_node,
                $graph_def,
                $target_chunk,
                &Some(connection),
                $context,
            )
        } else {
            -1
        };
        result
    }}
    
}

// Create a register map and add it's inputs by passing in pairs of string and identifiers containing the value.
// reg_opt_val is expected to be an Optional and if it's None it won't be added to the register map
macro_rules! reg_map_setup_inputs {
    ( $( $reg_name:expr, $reg_opt_val:ident ),* ) => {
        {
            let mut register_map = RegisterMap::default();
            $(
                if let Some(reg_val) = $reg_opt_val {
                    register_map.add_inparam($reg_name.into(), reg_val);
                }
            )*
            register_map
        }
    }
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

fn add_cell_and_invoking(
    graph_def: &mut PulseGraphDef,
    cell: Box<dyn PulseCellTrait>,
    register_map: RegisterMap,
    target_chunk: i32,
    func_name: Cow<'static, str>
) {
    let binding_id = graph_def.get_current_binding_id() + 1; // new binding id, it's here because of borrow checker
    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
    let instr = chunk.add_instruction(instruction_templates::cell_invoke(binding_id));
    // NOTE: Cell invokes require information about where they're been called from
    let binding = InvokeBinding {
        register_map,
        func_name: func_name.into(),
        cell_index: graph_def.cells.len() as i32, // the cell to be added
        src_chunk: target_chunk,
        src_instruction: instr,
    };
    graph_def.cells.push(cell);
    graph_def.add_invoke_binding(binding);
}

fn add_library_invoking(
    graph_def: &mut PulseGraphDef,
    register_map: RegisterMap,
    target_chunk: i32,
    func_name: Cow<'static, str>
) {
    // NOTE: Library invokes don't require source information unlike cell bindings.
    let invoke_binding = InvokeBinding {
        register_map,
        func_name,
        cell_index: -1,
        src_chunk: -1,
        src_instruction: -1,
    };
    let binding_id = graph_def.get_current_binding_id() + 1;
    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
    chunk.add_instruction(instruction_templates::library_invoke(binding_id));
    graph_def.add_invoke_binding(invoke_binding);
}

fn traverse_event_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
) {
    let input_id = node
        .get_input("event")
        .expect("Can't find input 'event'");
    let input_param = graph.inputs.get(input_id).expect("Can't find input value");
    let event_binding = input_param.value.clone().try_event_binding().unwrap();
    // create new pulse cell node.
    let chunk_id = graph_def.create_chunk();
    let mut cell_event = 
        CPulseCell_Inflow_EventHandler::new(chunk_id, event_binding.libname.into());

    // iterate all event params and add them as registers that can be used in the chunk
    // they will be all added even if no connections exist, but that's alright.
    if let Some(inparams) = event_binding.inparams {
        for param in inparams.iter() {
            let output_id = node
                .get_output(param.name.as_str())
                .expect(format!("Can't find output {}", param.name).as_str());

            let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
            let reg_id = chunk.add_register(param.pulsetype.to_string(), 0);
            cell_event.add_outparam(param.name.clone().into(), reg_id);
            graph_def.add_register_mapping(output_id, reg_id);
        }
    }

    graph_def.cells.push(Box::from(cell_event));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None, None);
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
    let cell_hook = CPulseCell_Inflow_GraphHook::new(hook_name.into(), RegisterMap::default(), chunk_id);
    graph_def.cells.push(Box::from(cell_hook));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None, None);
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
            traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None, None);
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
    let file_dir = &graph_state.save_file_path;
    if file_dir.is_none() {
        return Err("Output file path is set incorrectly".to_string());
    }
    let file_dir = file_dir.as_ref().unwrap().parent().unwrap();
    let dir_res = fs::create_dir_all(file_dir);
    if dir_res.is_err() {
        return Err(format!(
            "Failed to create output directory: {}",
            dir_res.err().unwrap()
        ));
    }
    fs::write(graph_state.save_file_path.as_ref().unwrap().as_path(), data)
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

fn try_find_input_mapping(graph_def: &PulseGraphDef, input_id: Option<&InputId>) -> Option<i32> {
    input_id.and_then(|id| graph_def.get_mapped_reigster_input(*id).copied())
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
) -> Option<i32> {
    let input_id = current_node
        .get_input(input_name)
        .expect(format!("Can't find input {}", input_name).as_str());
    let connection_to_input: Option<OutputId> = graph.connection(input_id);
    let target_register: i32;
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
                traverse_nodes_and_populate(graph, out_node, graph_def, chunk_id, &Some(out), None);
        }
        None => {
            if !always_reevaluate {
                // no connection found, create a constant value for the input
                // but first check if we have already created a constant for this value
                let inp_mapping = try_find_input_mapping(graph_def, Some(&input_id));
                if inp_mapping.is_some() {
                    return inp_mapping;
                }
            }
            if matches!(get_preffered_inputparamkind_from_type(&value_type), InputParamKind::ConnectionOnly) {
                println!("[INFO] Connection only input type without a connection, no constant will be created.");
                return None;
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
                PulseValueType::PVAL_SNDEVT_NAME(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_sndevt_name()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::SoundEventName(input_value));
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
                PulseValueType::PVAL_BOOL => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param
                        .value()
                        .clone()
                        .try_to_bool()
                        .expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Bool(input_value));
                }
                // Having a constant value for these doesn't make sense.
                PulseValueType::PVAL_EHANDLE(_) | PulseValueType::PVAL_SNDEVT_GUID(_) => {
                    return None;
                }
                _ => {
                    println!("Warning: Unsupported constant value type for input - None will be returned {}: {}", input_name, value_type);
                    return None;
                    // if we don't know the type, we can't create a constant for it.
                }
            };
            graph_def.add_register_mapping_input(input_id, target_register);
        }
    }
    Some(target_register)
}


// this might be used to get more context information while traversing other nodes
// for example WhileLoop requires jumping back to the original instruction that
// reevaluates the condition, so we need to know which instruction that was.
#[derive(Debug, Clone, Copy)]
pub enum NodeTraverseOriginContext {
    WhileLoop(i32), // i32 is instruction id, we want to find the lowest one.
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
    context: Option<NodeTraverseOriginContext>, // if this is Some, then we are in a context of a node that needs to be evaluated
) -> i32 {
    let mut new_context = context.clone();
    if let Some(context) = context {
        match context {
            NodeTraverseOriginContext::WhileLoop(instr_id) => {
                // we are in a while loop, so we need to set the instruction id to the one that will be evaluated
                // this is the one that will be used for the next iteration of the loop.
                new_context = Some(NodeTraverseOriginContext::WhileLoop(instr_id));
            }
        }
    }
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
            let reg_time = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "time",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            
            let register_map = reg_map_setup_inputs!(
                "flDurationSec", reg_time
            );
            let cell =
                CPulseCell_Inflow_Wait::new(target_chunk, graph_def.get_chunk_last_instruction_id(target_chunk) + 3);
            add_cell_and_invoking(graph_def, Box::from(cell), register_map, target_chunk, "Wait".into());
            // early return.
            let mut instr_ret_void = Instruction::default();
            instr_ret_void.code = String::from("RETURN_VOID");
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            chunk.add_instruction(instr_ret_void);

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

            let mut reg_param= None;
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
            
            // add invoke binding for FireAtName cell
            let register_map = reg_map_setup_inputs!(
                "TargetName", reg_entity,
                "pParam", reg_param
            );
            let cell = CPulseCell_Step_EntFire::new(input_value.clone().into());
            add_cell_and_invoking(graph_def, Box::from(cell), register_map, target_chunk, "FireAtName".into());
            let output_connection = OutputConnection::new(
                String::from("Step_EntFire:-1"),
                entity_name_static_value,
                input_value.clone(),
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
                            new_context,
                        );
                    }
                    None => {
                        // no connection.. First search if we already created it, if not create the constant input value
                        let register = try_find_input_mapping(graph_def, Some(&input_ids[i]));
                        if register.is_none() {
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
                            input_registers[i] = register.unwrap();
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
                        new_context
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
            if let Some(reg_value) = reg_value {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::set_var(reg_value, var_id.unwrap()));
            }

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::Operation => {
            let existing_reg_mapping = try_find_output_mapping(graph_def, output_id);
            if existing_reg_mapping != -1 {
                return existing_reg_mapping;
            }
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
            instr.reg1 = reg_a.unwrap_or(-1);
            instr.reg2 = reg_b.unwrap_or(-1);
            chunk.add_instruction(instr);
            if let Some(output) = output_id {
                graph_def.add_register_mapping(*output, register_output);
            }
            return register_output;
        }
        PulseNodeTemplate::FindEntByName => {
            let entclass_input_id = current_node
                .get_input("entClass")
                .expect("Can't find input 'entClass'");
            let entclass_input_param = graph
                .get_input(entclass_input_id)
                .value()
                .clone()
                .try_to_string()
                .expect("Can't find input 'entClass'");
            let mut reg_output = try_find_output_mapping(graph_def, output_id);
            let reg_entname = if reg_output == -1 { get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "entName",
                PulseValueType::DOMAIN_ENTITY_NAME,
                false,
            ) } else {
                None
            };
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
            let mut register_map = reg_map_setup_inputs!(
                "pName", reg_entname
            );
            register_map.add_outparam("retval".into(), reg_output);
            let cell = CPulseCell_Value_FindEntByName::new(entclass_input_param.into());
            add_cell_and_invoking(graph_def, Box::from(cell), register_map, target_chunk, "Eval".into());
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
            let reg_hentity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "hEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
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
            let mut register_map = reg_map_setup_inputs!(
                "hEntity", reg_hentity,
                "pMessage", reg_message,
                "nTextOffset", reg_ntextoffset,
                "flDuration", reg_flduration,
                "flVerticalOffset", reg_flverticaloffset,
                "color", reg_color,
                "flAlpha", reg_alpha,
                "flScale", reg_scale
            );
            register_map.add_inparam("bAttached".into(), reg_battached);
            add_library_invoking(graph_def, register_map, target_chunk, "CPulseServerFuncs!DebugWorldText".into());

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
            if let Some(reg_message) = reg_message {
                register_map.add_inparam("pMessage".into(), reg_message);
            }
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let binding = InvokeBinding {
                register_map,
                func_name: "Run".into(),
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
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            chunk.add_instruction(instruction_templates::cell_invoke(new_binding_id));
            let binding = InvokeBinding {
                register_map: RegisterMap::default(),
                func_name: "Run".into(),
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            graph_def.add_invoke_binding(binding);
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
            register_map.add_outparam("retval".into(), reg_output);
            let binding = InvokeBinding {
                register_map,
                func_name: "CPulseServerFuncs!GetGameTime".into(),
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
            if let Some(reg_dt) = reg_dt {
                register_map.add_inparam("dt".into(), reg_dt);
            }
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let binding = InvokeBinding {
                register_map,
                func_name: "CPulseServerFuncs!SetNextThink".into(),
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
            graph_def.add_invoke_binding(binding);

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::Convert => {
            
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
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
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register =
                    chunk.add_register(type_to.to_string(), chunk.get_last_instruction_id() + 1);
                if let Some(reg_input) = reg_input {
                    let instruction = instruction_templates::convert_value(register, reg_input);
                    chunk.add_instruction(instruction);
                }
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
            instr_compare.reg1 = reg_a.unwrap_or(-1);
            instr_compare.reg2 = reg_b.unwrap_or(-1);
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
                    new_context
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
                    new_context
                );
            } else {
                // empty instruction, so we can jump to it. (don't think that's necessary tho)
                chunk.add_instruction(Instruction::default());
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
        PulseNodeTemplate::CompareIf => {
            let reg_cond = get_connection_only_graph_input_value!(
                graph,
                current_node,
                "condition",
                graph_def,
                target_chunk,
                context
            );
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
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
                    new_context
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
                    new_context
                );
            } else {
                // empty instruction, so we can jump to it. (don't think that's necessary tho)
                chunk.add_instruction(Instruction::default());
            }
            // aaand borrow yet again lol
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // for now we just return. But we could have a 3rd port, that executes actions after doing the one in the chosen condition.
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
            // JUMP_COND{reg_cond}[curr + 3]
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
            let instr_copy = instruction_templates::copy_value(
                reg_idx, 
                reg_from.expect("ForLoop node: Input 'from' somehow ended up resulting to None!"));
            chunk.add_instruction(instr_copy);
            let reg_cond = chunk.add_register(String::from("PVAL_BOOL"), chunk.get_last_instruction_id() + 1);
            let mut instr_compare = Instruction::default();
            instr_compare.code = String::from("LTE_INT");
            instr_compare.reg0 = reg_cond;
            instr_compare.reg1 = reg_idx;
            instr_compare.reg2 = reg_to.expect("ForLoop node: Input 'to' somehow ended up resulting to None!");
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
                    &None,
                    new_context
                );
            }
            // borrow again (we know that it still is fine)
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // increment the index by step
            let instr_add = instruction_templates::add_value(
                reg_idx, 
                reg_step.expect("ForLoop node: Input 'step' somehow ended up resulting to None!"), 
                reg_idx);
            chunk.add_instruction(instr_add);
            
            // jump to conditional check
            let instr_jump = instruction_templates::jump(instr_compare_id);
            chunk.add_instruction(instr_jump);
            let end_instr_id = chunk.get_last_instruction_id() + 1;
            // update the jump instruction defined ealier to point to the end of the loop
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
                    &None,
                    new_context
                );
            }
        }
        PulseNodeTemplate::WhileLoop => {
            let context = Some(
                NodeTraverseOriginContext::WhileLoop(
                    graph_def.chunks
                        .get_mut(target_chunk as usize)
                        .unwrap()
                        .get_last_instruction_id()
                    )
                );
            
            let is_dowhile_loop = get_constant_graph_input_value!(
                graph, current_node, "do-while", try_to_bool
            );
            
            if !is_dowhile_loop {
                // While loop:
                // JUMP_COND{reg_condition == true}[curr + 3] (over the next jump)
                // JUMP[end] (to the end)
                // instructions
                // JUMP[evaluator] (to the condition evaluation)

                // save the current instruction id before populating the instruction for the condition
                // this will be the instruction that will be used to jump to the condition check
                let cond_instr_id = graph_def.chunks
                    .get_mut(target_chunk as usize)
                    .unwrap()
                    .get_last_instruction_id() + 1;
                let reg_condition = get_connection_only_graph_input_value!(
                    graph,
                    current_node,
                    "condition",
                    graph_def,
                    target_chunk,
                    context
                );
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr_jump_cond = instruction_templates::jump_cond(
                    reg_condition, chunk.get_last_instruction_id() + 3);
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
                        &None,
                        new_context 
                    );
                }
                // reborrow the chunk after we did borrow of graph_def.
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr_jump = instruction_templates::jump(cond_instr_id);
                chunk.add_instruction(instr_jump);
                chunk.get_instruction_from_id_mut(jump_end_instr_id)
                    .unwrap().dest_instruction = chunk.get_last_instruction_id() + 1;
                chunk.add_instruction(Instruction::default()); // NOP just in case.
            } else {
                // Do-While loop:
                // loopAction instructions
                // run condition checks
                // JUMP_COND{reg_condition == true}[start of loopAction instructions]
                // loopEnd instructions

                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                // remember the instruction id of the first instruction of the loop action to jump to later
                let loop_action_instructions_start = chunk.get_last_instruction_id() + 1;
                // first we fill out the instructions to run per iteration (it will always be run first without checking the condition)
                let action_node = get_next_action_node(current_node, graph, "loopAction");
                if action_node.is_some() {
                    traverse_nodes_and_populate(
                        graph,
                        action_node.unwrap(),
                        graph_def,
                        target_chunk,
                        &None,
                        new_context 
                    );
                }
                // next do all the condition check instructions
                let reg_condition = get_connection_only_graph_input_value!(
                    graph,
                    current_node,
                    "condition",
                    graph_def,
                    target_chunk,
                    context
                );
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                // jump back if condition is true
                let instr_jump_cond = instruction_templates::jump_cond(
                    reg_condition, loop_action_instructions_start);
                chunk.add_instruction(instr_jump_cond);
            }

            // after loop is finished (and if something is connected here) proceed.
            let end_action_node = get_next_action_node(current_node, graph, "endAction");
            if end_action_node.is_some() {
                traverse_nodes_and_populate(
                    graph,
                    end_action_node.unwrap(),
                    graph_def,
                    target_chunk,
                    &None,
                    new_context
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
                if let Some(reg_input) = reg_input {
                    reg_map.add_inparam("pStr".into(), reg_input);
                }
                reg_map.add_outparam("retval".into(), reg_out);
                let invoke_binding = InvokeBinding {
                    register_map: reg_map,
                    func_name: "CPulseServerFuncs!StringToEntityName".into(),
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
        PulseNodeTemplate::InvokeLibraryBinding => {
            let binding = get_constant_graph_input_value!(
                graph,
                current_node,
                "binding",
                try_library_binding
            );
            let mut register_map = RegisterMap::default();
            if let Some(inparams) = binding.inparams {
                for param in inparams.iter() {
                    let inp = get_input_register_or_create_constant(
                        graph,
                        current_node,
                        graph_def,
                        target_chunk,
                        &param.name,
                        param.pulsetype.clone(),
                        false
                    );
                    // if the input is connection only, and it happens to be unconnected, we don't want to add it to the register map.
                    if let Some(inp) = inp {
                        register_map.add_inparam(param.name.clone().into(), inp);
                    }
                }
            }

            let mut reg_output = -1;
            let mut evaluation_required: bool = true; // set to true if we need to invoke something instead of re-using the register
            if let Some(outparams) = binding.outparams {
                // super dirty assumption as we only support one return value for now.
                if outparams.len() > 1 {
                    todo!("InvokeLibraryBinding node: More than one output parameter is not supported yet.");
                }
                if let Some(retval) = outparams.first() {
                    reg_output = try_find_output_mapping(graph_def, output_id);
                    if reg_output == -1 {
                        let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                        reg_output = chunk.add_register(
                            retval.pulsetype.to_string(),
                            chunk.get_last_instruction_id() + 1,
                        );
                        if let Some(out) = output_id {
                            graph_def.add_register_mapping(*out, reg_output);
                        }
                    } else {
                        // we already have a register for this output, so we don't need to invoke the binding again.
                        evaluation_required = false;
                    }
                    register_map.add_outparam("retval".into(), reg_output);
                }
            }
            if evaluation_required {
                let invoke_binding = InvokeBinding {
                    register_map,
                    func_name: binding.libname.into(),
                    cell_index: -1,
                    src_chunk: -1,
                    src_instruction: -1,
                };
                let new_binding_id = graph_def.get_current_binding_id() + 1;
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
                graph_def.add_invoke_binding(invoke_binding);
            }
            match binding.typ {
                LibraryBindingType::Action => {
                    graph_next_action!(graph, current_node, graph_def, target_chunk);
                },
                LibraryBindingType::Value => return reg_output,
            }
        }
        PulseNodeTemplate::FindEntitiesWithin => {
            let classname = get_constant_graph_input_value!(
                graph,
                current_node,
                "classname",
                try_to_string
            );
            // Has this been already evaulated?
            let mut reg_output = try_find_output_mapping(graph_def, output_id);
            if reg_output == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                reg_output = chunk.add_register(
                    PulseValueType::PVAL_EHANDLE(Some(classname.clone())).to_string(),
                    chunk.get_last_instruction_id() + 1,
                );
                if let Some(out) = output_id {
                    graph_def.add_register_mapping(*out, reg_output);
                }
            } else {
                return reg_output;
            }
            // connection only
            let reg_searchfroment = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "pSearchFromEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            );
            let reg_radius = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "flSearchRadius",
                PulseValueType::PVAL_FLOAT(None),
                false,
            );
            // connection only
            let reg_startentity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "pStartEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            );

            let mut register_map = reg_map_setup_inputs!(
                "pSearchFromEntity", reg_searchfroment,
                "flSearchRadius", reg_radius,
                "pStartEntity", reg_startentity
            );
            register_map.add_outparam("retval".into(), reg_output);
            let cell = CPulseCell_Value_FindEntByClassNameWithin::new(classname.into());
            add_cell_and_invoking(graph_def, Box::from(cell), register_map, target_chunk, "Eval".into());
            return reg_output;
        }
        PulseNodeTemplate::IsValidEntity => {
            let hentity_input_id = current_node
                .get_input("hEntity")
                .expect("Can't find input 'value'");
            let connection_to_hentity = graph.connection(hentity_input_id);
            if connection_to_hentity.is_none() {
                println!("No connection found for hEntity input in IsValidEntity node. Node will not be processed, next action won't execute.");
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
                new_context
            );
            // This is literally copy-paste from Condition node. TODO: refactor this. PLEASE!!!!!!
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let instr_jump_cond =
                instruction_templates::jump_cond(reg_hentity, chunk.get_last_instruction_id() + 3);
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
                    new_context
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
                    new_context
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
                    "IsEntValid node: Failed to find JUMP[false_condition] with id: {}",
                    jump_false_instr_id
                );
            }
            let instr_jump_end = chunk.get_instruction_from_id_mut(jump_end_instr_id);
            if instr_jump_end.is_some() {
                instr_jump_end.unwrap().dest_instruction = ending_instr_id;
            } else {
                panic!(
                    "IsEntValid node: Failed to find JUMP[end] with id: {}",
                    jump_end_instr_id
                );
            }
        }
        PulseNodeTemplate::CompareOutput => {
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
            // value that we will match to generate a operation
            // this might get turned into a proper enum at some point
            let compare_value = get_constant_graph_input_value!(
                graph,
                current_node,
                "operation",
                try_to_string
            );
            let mut create_comp_instructions =
             |code: Cow<'_, str>, reg_a: Option<i32>, reg_b: Option<i32>| -> i32 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let mut instr_compare = Instruction::default();
                instr_compare.code = String::from(code);
                let reg_cond;
                reg_cond = chunk.add_register(
                    String::from("PVAL_BOOL"),
                    chunk.get_last_instruction_id() + 1,
                );
                instr_compare.reg0 = reg_cond;
                instr_compare.reg1 = reg_a.unwrap_or(-1);
                instr_compare.reg2 = reg_b.unwrap_or(-1);
                chunk.add_instruction(instr_compare);
                reg_cond
            };
            let reg_comp: i32 = match compare_value.as_str() {
                "==" => {
                    create_comp_instructions(
                        format!("EQ{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b)
                }
                "!=" => {
                    create_comp_instructions(
                        format!("NE{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b)
                }
                "<" => {
                    create_comp_instructions(
                        format!("LT{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b)
                }
                "<=" => {
                    create_comp_instructions(
                        format!("LTE{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b)
                }
                // > is NOT <=
                ">" => {
                    let reg_lte= create_comp_instructions(
                        format!("LTE{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b);
                    
                    create_comp_instructions(
                        "NOT".into(),
                        Some(reg_lte),
                        None)
                }
                // >= is NOT <
                ">=" => {
                    let reg_lt= create_comp_instructions(
                        format!("LT{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b);
                    
                    create_comp_instructions(
                        "NOT".into(),
                        Some(reg_lt),
                        None)
                }
                _ => {
                    panic!(
                        "CompareOutput node: Unknown operation value: {}",
                        compare_value
                    );
                }
            };
            return reg_comp;
        }
        PulseNodeTemplate::IntSwitch => {
            let reg_value = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                target_chunk,
                "value",
                PulseValueType::PVAL_INT(None),
                false,
            );
            let register_map = reg_map_setup_inputs!(
                "nSwitchValue", reg_value
            );
            // now comes the processing of the outflows
            let mut outflow_connections = vec![];
            let mut instructions_jump_end = vec![];
            let mut value_idx = 0; // used for outflow names only
            current_node.outputs.iter().for_each(|out| {
                if out.0.parse::<i32>().is_ok() {
                    let next_action_node_id = get_connected_output_node(graph,&out.1);
                    if let Some(next_action_node_id) = next_action_node_id {
                        let this_action_instruction_id = graph_def.chunks.get(target_chunk as usize).unwrap().get_last_instruction_id() + 1;
                        let node = graph.nodes.get(next_action_node_id).unwrap();
                        traverse_nodes_and_populate(
                            graph,
                            node,
                            graph_def,
                            target_chunk,
                            &None,
                            new_context 
                        );
                        // add a JUMP instruction to the end of all of the cases
                        // we don't really know where that will be so we will have to note down the instruction id and modify it later.
                        let instr_jump = instruction_templates::jump(-1);
                        instructions_jump_end.push(
                            graph_def.chunks.get_mut(target_chunk as usize).unwrap().add_instruction(instr_jump));

                        outflow_connections.push(OutflowConnection::new(
                            out.0.clone().into(),
                            target_chunk,
                            this_action_instruction_id, 
                            RegisterMap::default()
                        ));
                    }
                    value_idx += 1;
                }
            });
            // now let's process the default case
            let default_case_instruction_id = graph_def.chunks.get(target_chunk as usize).unwrap().get_last_instruction_id() + 1;
            let default_action_node = get_next_action_node(current_node, graph, "defaultcase");
            let mut default_case_outflow = None;
            if let Some(default_action_node) = default_action_node {
                traverse_nodes_and_populate(graph, default_action_node, graph_def, target_chunk, output_id, context);
                default_case_outflow = Some(OutflowConnection::new(
                    "default".into(),
                    target_chunk,
                    default_case_instruction_id,
                    RegisterMap::default()
                ));
            }
            
            // as the default case is the last one, we do not need a jump here.
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let ending_instruction_id = chunk.add_instruction(Instruction::default()); // NOP for the end jump
            // Now update all the jumps defined before to point to the end of the switch statement.
            for instr in instructions_jump_end.iter() {
                let instr_jump_end = chunk.get_instruction_from_id_mut(*instr);
                if let Some(instr_jump_end) = instr_jump_end {
                    instr_jump_end.dest_instruction = ending_instruction_id;
                } else { // We expect this to be a valid instruction id, so we panic if it isn't.
                    panic!(
                        "IntSwitch node: Failed to find JUMP[end] with id: {}",
                        instr
                    );
                }
            }
            let cell = CPulseCell_Outflow_IntSwitch::new(
                default_case_outflow.unwrap_or_default(),
                outflow_connections,
            );
            add_cell_and_invoking(graph_def, Box::from(cell), register_map, target_chunk, "Run".into());
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        PulseNodeTemplate::SoundEventStart => {
            let mut reg_out = try_find_output_mapping(graph_def, output_id);
            if reg_out == -1 {
                let reg_soundevent = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    target_chunk,
                    "strSoundEventName",
                    PulseValueType::PVAL_SNDEVT_NAME(None),
                    false,
                );
                let reg_target_entity = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    target_chunk,
                    "hTargetEntity",
                    PulseValueType::PVAL_EHANDLE(None),
                    false,
                );
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr = chunk.get_last_instruction_id() + 1;
                reg_out = chunk.add_register(
                    PulseValueType::PVAL_SNDEVT_GUID(None).to_string(),
                    instr,
                );
                if let Some(out) = output_id {
                    graph_def.add_register_mapping(*out, reg_out);
                }
                let mut register_map = reg_map_setup_inputs!(
                    "strSoundEventName", reg_soundevent,
                    "hTargetEntity", reg_target_entity
                );
                register_map.add_outparam("retval".into(), reg_out);
                let cell = CPulseCell_SoundEventStart::new(SoundEventStartType::SOUNDEVENT_START_ENTITY);
                add_cell_and_invoking(graph_def, Box::new(cell), register_map, target_chunk, "Run".into());
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
