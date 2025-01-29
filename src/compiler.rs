use egui_node_graph2::*;
use crate::app::{MyGraph, MyNodeData, MyNodeTemplate, MyDataType, MyValueType};
use crate::instruction_templates;
use crate::pulsetypes::*;
use crate::serialization::*;
use std::fs;

pub enum NodeInput {
    ConstantOnly(String, PulseValueType),
    ConnectionOnly(String, PulseValueType),
    ConstantOrConnection(String, PulseValueType),
}

#[derive(PartialEq, Eq)]
pub enum NodeType {
    Evaluate,
    Action,
}
pub trait PulseGraphNode {
    fn get_registered_inputs(&self) -> Vec<NodeInput>;
    fn get_registered_outputs(&self) -> Vec<String>;
    fn get_type(&self) -> NodeType;
    fn get_binding_name(&self) -> Option<String>;
    fn get_cell_type(&self) -> Option<CellType>;
}
struct PulseNodeInfo {
    defined_inputs: Vec<String>,
    defined_outputs: Vec<String>,
    node_type: NodeType,
    binding_name: Option<String>,
    cell_type: Option<CellType>,
}

macro_rules! graph_next_action {
    ($graph:ident, $current_node:ident, $graph_def:ident, $target_chunk:ident) => {
        let connected_node = get_next_action_node($current_node, $graph, "outAction");
        if connected_node.is_some() {
            return traverse_nodes_and_populate($graph, connected_node.unwrap(), $graph_def, $target_chunk, &None);
        }
    };
}

fn populate_based_on_node(graph: &MyGraph, node_data: &impl PulseGraphNode, graph_def: &mut PulseGraphDef, current_node: &Node<MyNodeData>, target_chunk: i32) {
    let mut register_map = RegisterMap::default();
    for node_input in node_data.get_registered_inputs().iter() {
        match node_input {
            NodeInput::ConstantOnly(name, valtyp)
            | NodeInput::ConstantOrConnection(name, valtyp) => {
                let reg = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, name, valtyp.clone());
                register_map.add_inparam(name.clone(), reg);
            }
            NodeInput::ConnectionOnly(_, pulse_value_type) => todo!(),
        }
    }
    let mut chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
    // only if the node actually calls a binding... which we can find out by checking if the binding name is set.
    let binding_name = node_data.get_binding_name();
    if binding_name.is_some() {
        let binding = InvokeBinding {
            register_map,
            func_name: binding_name.unwrap(),
            cell_index: graph_def.cells.len() as i32 - 1,
            src_chunk: target_chunk,
            src_instruction: chunk.get_last_instruction_id() + 1,
        };
    }

    if node_data.get_type() == NodeType::Action {
        //graph_next_action!(graph, current_node, graph_def, target_chunk);
    }   
}

fn get_connected_output_node(graph: &MyGraph, out_action_id: &OutputId) -> Option<NodeId> {
    // dumb way of finding outgoing connection node.
    for group in graph.iter_connection_groups() {
        for connection in group.1 {
            if connection == *out_action_id {
                let input_action: &InputParam<MyDataType, MyValueType> = graph.inputs.get(group.0).expect("Can't find input value");
                return Some(input_action.node);
            }
        }
    }
    None
}

fn get_next_action_node<'a>(origin_node: &'a Node<MyNodeData>, graph: &'a MyGraph, name: &str) -> Option<&'a Node<MyNodeData>> {
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

fn traverse_event_cell(graph: &MyGraph, node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef) {
    let input_id = node.get_input("eventName").expect("Can't find input 'eventName'");
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
    graph_def.cells.push(Box::from(CellType::InflowEvent(cell_event)));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
    }
}

fn traverse_entry_cell(graph: &MyGraph, node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef)
{
    let input_id = node.get_input("name").expect("Can't find input 'name'");
    let input_param = graph.inputs.get(input_id).expect("Can't find input value");
    // create new pulse cell node.
    let mut cell_method = CPulseCell_Inflow_Method::default();
    let chunk_id = graph_def.create_chunk();
    cell_method.name = input_param.value.clone().try_to_string().unwrap();
    cell_method.entry_chunk = chunk_id;
    cell_method.return_type = String::from("PVAL_INVALID");
    // get action connection
    let out_action_id = node.get_output("outAction").expect("Can't find output 'outAction'");
    //let out_action_param = graph.outputs.get(out_action_id).expect("Can't find output value");
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    // create argument1 (TODO only if connection exists)
    let reg_id_arg1 = chunk.add_register(String::from("PVAL_STRING"), 0);
    let output_id_arg1 = node.get_output("argument1").expect("Can't find output 'argument1'");
    cell_method.add_arg(String::from("arg1"), String::default(), String::from("PVAL_STRING"), reg_id_arg1);
    graph_def.add_register_mapping(output_id_arg1, reg_id_arg1);

    let cell_enum = CellType::InflowMethod(cell_method);
    graph_def.cells.push(Box::from(cell_enum));
    //let mut connected_node_id = NodeId::default();
    // dumb way of finding outgoing connection node.
    // graph.iter_connection_groups().for_each(|group| {
    //     group.1.iter().for_each(|connection| {
    //         if *connection == out_action_id {
    //             let input_action: &InputParam<MyDataType, MyValueType> = graph.inputs.get(group.0).expect("Can't find input value");
    //             connected_node_id = input_action.node;
    //             return;
    //         }
    //     });
    // });
    let connected_node_id = get_connected_output_node(graph, &out_action_id);
    if connected_node_id.is_some() {
        let connected_node = graph.nodes.get(connected_node_id.unwrap());
        if connected_node.is_some() {
            traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
        }
    }
}

pub fn compile_graph(graph: &MyGraph) {
    let mut graph_def = PulseGraphDef::default();
    graph_def.map_name = String::from("maps/test.vmap");
    graph_def.xml_name = String::default();
    for node in graph.iter_nodes() {
        let data: &Node<MyNodeData> = graph.nodes.get(node).unwrap();
        // start at all possible entry points
        match data.user_data.template {
            MyNodeTemplate::EventHandler => traverse_event_cell(graph, &data, &mut graph_def),
            MyNodeTemplate::CellPublicMethod => traverse_entry_cell(graph, &data, &mut graph_def),
            _ => {}
        }
    }
    let mut data = String::from("<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:vpulse13:version{354e36cb-dbe4-41c0-8fe3-2279dd194022} -->\n");
    data.push_str(graph_def.serialize().as_str());
    fs::write("graph_out/graph.vpulse", data).expect("Cannot write to file!");
}

fn try_find_output_mapping(graph_def: &PulseGraphDef, output_id: &Option<OutputId>) -> i32 {
    match output_id {
        Some(output_id) => {
            match graph_def.get_mapped_reigster(*output_id) {
                Some(reg) => { // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => { return -1; }
            }
        }
        None => { return -1; }
    }
}

fn create_or_get_variable(graph_def: &mut PulseGraphDef, name: &str, defaults: PulseValueType) -> i32 {
    match graph_def.get_variable_index(&name) {
        Some(var) => {
            return var as i32;
        }
        None => {
            let var = PulseVariable {
                name: name.to_string(),
                typ_and_default_value: defaults
            };
            return graph_def.add_variable(var);
        }
    }
}

fn try_find_input_mapping(graph_def: &PulseGraphDef, input_id: &Option<InputId>) -> i32 {
    match input_id {
        Some(input_id) => {
            match graph_def.get_mapped_reigster_input(*input_id) {
                Some(reg) => { // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => { return -1; }
            }
        }
        None => { return -1; }
    }
}

fn get_input_register_or_create_constant(graph: &MyGraph, current_node: &Node<MyNodeData>,
     graph_def: &mut PulseGraphDef, chunk_id: i32, input_name: &str, value_type: PulseValueType) -> i32 {

    let input_id = current_node.get_input(input_name).expect(format!("Can't find input {}", input_name).as_str());
    let connection_to_input = graph.connection(input_id);
    let mut target_register: i32;
    // if we find a connection, then traverse to that node, whatever happens we should get a register id back.
    match connection_to_input {
        Some(out) => {
            // connection found to an outputid of the connected node. Traverse to that node, and get the register
            let out_param = graph.get_output(out);
            let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
            target_register = traverse_nodes_and_populate(graph, out_node, graph_def, chunk_id, &Some(out));
        }
        None => {
            // no connection found, create a constant value for the input
            // but first check if we have already created a constant for this value
            target_register = try_find_input_mapping(graph_def, &Some(input_id));
            if target_register != -1 {
                return target_register
            }
            let new_constant_id = graph_def.get_current_constant_id() + 1;
            let new_domain_val_id = graph_def.get_current_domain_val_id() + 1;
            let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
            target_register = chunk.add_register(value_type.to_string(), chunk.get_last_instruction_id() + 1);
            let input_param = graph.get_input(input_id);

            let instruction: Instruction;
            match value_type {
                PulseValueType::PVAL_INT(_) => 
                {
                    instruction = instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Integer(input_value as i32));
                }
                PulseValueType::PVAL_FLOAT(_) => 
                {
                    instruction = instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Float(input_value));
                }
                PulseValueType::PVAL_STRING(_) => 
                {
                    instruction = instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_string().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::String(input_value));
                }
                PulseValueType::DOMAIN_ENTITY_NAME => {
                    instruction = instruction_templates::get_domain_value(target_register, new_domain_val_id);
                    let input_value = input_param.value().clone().try_to_string().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.create_domain_value(String::from("ENTITY_NAME"), input_value.clone(), String::new());
                }
                PulseValueType::PVAL_VEC3(_) => 
                {
                    instruction = instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_vec3().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Vec3(input_value));
                }
                PulseValueType::PVAL_COLOR_RGB(_) => 
                {
                    instruction = instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_vec3().expect("Failed to unwrap input value");
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Color_RGB(input_value));
                }
                _ => panic!("Unsupported value type")
            };
        }
    }
    target_register
}

fn traverse_nodes_and_populate(graph: &MyGraph, current_node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef, target_chunk: i32, output_id: &Option<OutputId>) -> i32 {
    match current_node.user_data.template {
        MyNodeTemplate::CellPublicMethod => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        MyNodeTemplate::EventHandler => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        MyNodeTemplate::CellWait => {
            let time_input_id = current_node.get_input("time").expect("Can't find input 'time'");
            let connection_to_time = graph.connection(time_input_id);
            let time_input_register: i32;
            match connection_to_time {
                Some(out) => {
                    // get existing register id to note for the wait time
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    time_input_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                }
                None => {
                    // create a constant value for the wait time
                    // have to do this way because of the borrow checker's mutability rules.
                    let new_constant_id = graph_def.get_current_constant_id() + 1;

                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    // add a new register to store the wait time
                    time_input_register = chunk.add_register(String::from("PVAL_FLOAT"), chunk.get_last_instruction_id() + 1);
                    // get the wait time constant
                    let instruction = instruction_templates::get_const(new_constant_id, time_input_register);
                    chunk.add_instruction(instruction);
                    // add the actual constant definition for the waiting time
                    let value_param = graph.get_input(time_input_id);
                    let time = value_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                    let constant = PulseConstant::Float(time);
                    graph_def.add_constant(constant);
                }
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // ! Important (might change). We assume that after waiting we go to the next instruction after the cell invoke.
            let cell_wait = CPulseCell_Inflow_Wait::new(target_chunk, chunk.get_last_instruction_id() + 3);
            graph_def.cells.push(Box::from(CellType::InflowWait(cell_wait)));

            let chunk_opt = graph_def.chunks.get(target_chunk as usize);
            if chunk_opt.is_some() {
                let chunk = chunk_opt.unwrap();
                let mut register_map = RegisterMap::default();
                register_map.add_inparam(String::from("flDurationSec"), time_input_register);
                let binding = InvokeBinding {
                    register_map,
                    func_name: String::from("Wait"),
                    cell_index: graph_def.cells.len() as i32 - 1,
                    src_chunk: target_chunk,
                    src_instruction: chunk.get_last_instruction_id() + 1,
                };
                let binding_idx = graph_def.add_binding(binding);
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::cell_invoke(binding_idx));
                // early return.
                let mut instr_ret_void = Instruction::default();
                instr_ret_void.code = String::from("RETURN_VOID");
                chunk.add_instruction(instr_ret_void);
            }

            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        MyNodeTemplate::EntFire => {
            // create EntFire (step) cell
            let entity_id = current_node.get_input("entity").expect("Can't find input 'entity'");
            let value_entity = graph.inputs.get(entity_id).expect("Can't find input value").value.clone().try_to_string();
            if let Ok(value) = value_entity {
                // create domain value (only if we know value already)
                let domain_val_idx = graph_def.create_domain_value(String::from("ENTITY_NAME"), value.clone(), String::new());
                
                let input_id = current_node.get_input("input").expect("Can't find input 'input'");
                let input_param = graph.inputs.get(input_id).expect("Can't find input value").value.clone().try_to_string().expect("Failed to unwrap input value");
                let step_cell = CPulseCell_Step_EntFire::new(input_param.clone());
                let cell_enum = CellType::StepEntFire(step_cell);
                
                let value_id = current_node.get_input("value").expect("Can't find input 'value'");
                let mut value_input_register: i32 = -1;
                // try to resolve the value input node
                let connection_to_value = graph.connection(value_id);
                match connection_to_value {
                    Some(out) => {
                        let out_param = graph.get_output(out);
                        let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                        value_input_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                    }
                    None => { // no connection, create constant value
                        let value_param = graph.get_input(value_id);
                        let str= value_param.value().clone().try_to_string();
                        match str {
                            Ok(str) => {
                                if !str.is_empty() {
                                    let constant = PulseConstant::String(str);
                                    let const_idx = graph_def.add_constant(constant);
                                    // create register to hold this value
                                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                                    value_input_register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                                    // create instruction to load this value now.
                                    let instruction = instruction_templates::get_const(const_idx, value_input_register);
                                    chunk.add_instruction(instruction);
                                }
                            }
                            Err(err) => {
                                println!("Error getting string from node value: {}", err);
                            }
                        }
                    }
                }
                
                //let value_param = graph.get_input(value_id)
                graph_def.cells.push(Box::from(cell_enum));
                // now build instructions and bindings to get the domain value, and invoke the cell
                let chunk_opt = graph_def.chunks.get_mut(target_chunk as usize);
                if chunk_opt.is_some() {
                    let chunk = chunk_opt.unwrap();
                    // new register to load in the domain value
                    let reg_id = chunk.add_register(String::from("PVAL_ENTITY_NAME"), chunk.get_last_instruction_id() + 1);
                    // load the domain value instruction
                    chunk.add_instruction(instruction_templates::get_domain_value(reg_id, domain_val_idx));
                    // add invoke binding for FireAtName cell
                    let mut register_map = RegisterMap::default();
                    register_map.add_inparam(String::from("TargetName"), reg_id);
                    if value_input_register != -1 {
                        register_map.add_inparam(String::from("pParam"), value_input_register);
                    }
                    let binding = InvokeBinding {
                        register_map: register_map,
                        func_name: String::from("FireAtName"),
                        cell_index: graph_def.cells.len() as i32 - 1,
                        src_chunk: target_chunk,
                        src_instruction: chunk.get_last_instruction_id() + 1,
                    };
                    let binding_idx = graph_def.add_binding(binding);
                    // add instruction for invoking the binding.
                    // rust doesn't like reusing the borrowed chunks reference, but we know that it doesn't change.
                    graph_def.chunks.get_mut(target_chunk as usize).unwrap()
                    .add_instruction(instruction_templates::cell_invoke(binding_idx));
                    //graph.connection(input)
                    let output_connection = OutputConnection::new(
                        String::from("Step_EntFire:-1"), 
                        value, input_param, if value_input_register != -1 { String::from("param") } else { String::default() });
                    graph_def.add_output_connection(output_connection);
                }

                graph_next_action!(graph, current_node, graph_def, target_chunk);
            }
            
        }
        MyNodeTemplate::ConcatString => {
            let id_a = current_node.get_input("A").expect("Can't find input A in node");
            let id_b = current_node.get_input("B").expect("Can't find input B in node");
            let input_ids = [id_a, id_b];
            let connection_to_a = graph.connection(id_a);
            let connection_to_b = graph.connection(id_b);
            let connections_to_resolve: [Option<OutputId>; 2] = [connection_to_a, connection_to_b]; 
            let mut input_registers: [i32; 2] = [-1, -1];

            for (i, connection) in connections_to_resolve.iter().enumerate() {
                match connection {
                    Some(out) => {
                        let out_param = graph.get_output(*out);
                        let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                        // grab the register that the value will come from.
                        input_registers[i] = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(*out));
                    }
                    None => {
                        // no connection.. First search if we already created it, if not create the constant input value
                        let register = try_find_input_mapping(graph_def, &Some(input_ids[i]));
                        if register == -1 {
                            let input_info: &InputParam<MyDataType, MyValueType> = graph.get_input(input_ids[i]);
                            let constant = PulseConstant::String(input_info.value.clone().try_to_string().unwrap());
                            let const_idx = graph_def.add_constant(constant);
                            // create register to hold this value
                            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                            input_registers[i] = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                            // create instruction to load this value now.
                            let instruction = instruction_templates::get_const(const_idx, input_registers[i]);
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
                register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                let instruction = instruction_templates::add_string(input_registers[0], input_registers[1], register);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        MyNodeTemplate::GetVar => {
            let name_id = current_node.get_input("name").expect("Can't find input 'name'");
            // name is a constant value
            let name = graph.get_input(name_id).value().clone().try_to_string().expect("Can't find name parameter");
            let var_id = create_or_get_variable(graph_def, name.as_str());
            // add register
            // add instruction to load the variable value
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg = chunk.add_register(String::from("PVAL_INT"), chunk.get_last_instruction_id() + 1);
            chunk.add_instruction(instruction_templates::get_var(reg, var_id as i32));
            return reg;
        }
        MyNodeTemplate::IntToString => {
            let value_id = current_node.get_input("value").expect("Can't find input 'value'");
            let connection_to_value = graph.connection(value_id);
            let register_input: i32;
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    // grab the register that the value will come from.
                    register_input = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                }
                None => {
                    print!("No connection found for input value for IntToString node");
                    return -1;
                } 
            }
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                let instruction = instruction_templates::convert_value(register, register_input);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        MyNodeTemplate::SetVar => {
            let name_id = current_node.get_input("name").expect("Can't find input 'name'");
            // name is a constant value
            let name = graph.get_input(name_id).value().clone().try_to_string().expect("Can't find name parameter");
            let var_id = create_or_get_variable(graph_def, name.as_str());
            let value_id = current_node.get_input("value").expect("Can't find input 'value'");
            let connection_to_value = graph.connection(value_id);
            let mut target_register: i32;
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    target_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                }
                None => {
                    target_register = try_find_input_mapping(graph_def, &Some(value_id));
                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    if target_register == -1 {
                        let value_param = graph.get_input(value_id);
                        let value = value_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                        target_register = chunk.add_register(String::from("PVAL_INT"), chunk.get_last_instruction_id() + 1);
                        let instruction = instruction_templates::get_const(value as i32, target_register);
                        chunk.add_instruction(instruction);
                    }
                }
                
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            chunk.add_instruction(instruction_templates::set_var(var_id as i32, target_register));
        }
        MyNodeTemplate::Operation => {
            let reg_a = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "A", PulseValueType::PVAL_FLOAT(None));
            let reg_b = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "B", PulseValueType::PVAL_FLOAT(None));
            let operation_input_id = current_node.get_input("operation").expect("Can't find input 'operation'");
            let operation_input_param = graph.get_input(operation_input_id).value().clone().try_to_string().expect("Can't find input 'operation'");
            let operation_instr_name: &str = match operation_input_param.as_str() {
                "+" => "ADD_FLOAT",
                "-" => "SUB_FLOAT",
                "*" => "MUL_FLOAT",
                "/" => "DIV_FLOAT",
                "%" => "MOD_FLOAT",
                _ => "ADD_FLOAT"
            };
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let register_output = chunk.add_register(String::from("PVAL_FLOAT"), chunk.get_last_instruction_id() + 1);
            let mut instr = Instruction::default();
            instr.code = String::from(operation_instr_name);
            instr.reg0 = register_output;
            instr.reg1 = reg_a;
            instr.reg2 = reg_b;
            chunk.add_instruction(instr);
        }
        MyNodeTemplate::FindEntByName => {
            let reg_entname = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "entName", PulseValueType::DOMAIN_ENTITY_NAME);
            let entclass_input_id = current_node.get_input("entClass").expect("Can't find input 'entClass'");
            let entclass_input_param = graph.get_input(entclass_input_id).value().clone().try_to_string().expect("Can't find input 'entClass'");
            let new_binding_idx = graph_def.get_current_binding_id() + 1;
            let mut register_map = RegisterMap::default();
            let mut reg_output = try_find_output_mapping(graph_def, output_id);
            if reg_output == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                reg_output = chunk.add_register(PulseValueType::PVAL_EHANDLE(Some(entclass_input_param.clone())).to_string(), chunk.get_last_instruction_id() + 1);
                if let Some(out) = output_id {
                    graph_def.add_register_mapping(*out, reg_output);
                }
            } else {
                return reg_output;
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let cell = CPulseCell_Value_FindEntByName::new(entclass_input_param);
            let cell_enum = CellType::ValueFindEntByName(cell);
            graph_def.cells.push(Box::from(cell_enum));
            register_map.add_inparam(String::from("pName"), reg_entname);
            register_map.add_outparam(String::from("retval"), reg_output);
            let instr = chunk.add_instruction(instruction_templates::cell_invoke(new_binding_idx));
            let binding = InvokeBinding {
                register_map,
                func_name: String::from("Eval"),
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: instr,
            };
            graph_def.add_binding(binding);
            return reg_output;
        }
        MyNodeTemplate::DebugWorldText => {
            let reg_message = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "pMessage", PulseValueType::PVAL_STRING(None));
            // resolve connection to hEntity
            let hentity_input_id = current_node.get_input("hEntity").expect("Can't find input 'value'");
            let connection_to_hentity = graph.connection(hentity_input_id);
            if connection_to_hentity.is_none() {
                println!("No connection found for hEntity input in DebugWorldText node. Node will not be processed, next action won't execute.");
                return -1;
            }
            let connection_to_hentity = connection_to_hentity.unwrap();
            let hentity_param = graph.get_output(connection_to_hentity);
            let out_node = graph.nodes.get(hentity_param.node).expect("Can't find output node");
            let reg_hentity = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(connection_to_hentity));
            // other params
            let reg_ntextoffset = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "nTextOffset", PulseValueType::PVAL_INT(None));
            let reg_flduration = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "flDuration", PulseValueType::PVAL_FLOAT(None));
            let reg_flverticaloffset = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "flVerticalOffset", PulseValueType::PVAL_FLOAT(None));
            // color:
            let reg_color = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "color", PulseValueType::PVAL_COLOR_RGB(None));
            let reg_alpha = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "flAlpha", PulseValueType::PVAL_FLOAT(None));
            let reg_scale = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "flScale", PulseValueType::PVAL_FLOAT(None));
            // bAttached:
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let battached_input_id = current_node.get_input("bAttached").expect("Can't find input 'bAttached'");
            let battached_input_param = graph.get_input(battached_input_id).value().clone().try_to_bool().expect("Can't find input 'bAttached'");
            graph_def.add_constant(PulseConstant::Bool(battached_input_param));
            // create constant, add instruction and a register to load it into.
            let new_constant_id = graph_def.get_current_constant_id();
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg_battached = chunk.add_register(String::from("PVAL_BOOL"), chunk.get_last_instruction_id() + 1);
            let instruction = instruction_templates::get_const(new_constant_id, reg_battached);
            chunk.add_instruction(instruction);
            let mut register_map = RegisterMap::default();
            register_map.add_inparam(String::from("hEntity"), reg_hentity);
            register_map.add_inparam(String::from("nTextOffset"), reg_ntextoffset);
            register_map.add_inparam(String::from("pMessage"), reg_message);
            register_map.add_inparam(String::from("flDuration"), reg_flduration);
            register_map.add_inparam(String::from("flVerticalOffset"), reg_flverticaloffset);
            register_map.add_inparam(String::from("bAttached"), reg_battached);
            register_map.add_inparam(String::from("color"), reg_color);
            register_map.add_inparam(String::from("flAlpha"), reg_alpha);
            register_map.add_inparam(String::from("flScale"), reg_scale);
            let binding = InvokeBinding {
                register_map,
                func_name: String::from("CPulseServerFuncs!DebugWorldText"),
                cell_index: -1,
                src_chunk: -1,
                src_instruction: -1,
            };
            chunk.add_instruction(instruction_templates::library_invoke(new_binding_id));
            graph_def.add_binding(binding);

            // go to next action.
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        MyNodeTemplate::DebugLog => {
            let reg_message = get_input_register_or_create_constant(graph, current_node, graph_def, target_chunk, "pMessage", PulseValueType::PVAL_STRING(None));
            let cell_enum = CellType::DebugLog;
            graph_def.cells.push(Box::from(cell_enum));
            let mut register_map = RegisterMap::default();
            register_map.add_inparam(String::from("pMessage"), reg_message);
            let new_binding_id = graph_def.get_current_binding_id() + 1;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let binding = InvokeBinding {
                register_map,
                func_name: String::from("Run"),
                cell_index: graph_def.cells.len() as i32 - 1,
                src_chunk: target_chunk,
                src_instruction: chunk.get_last_instruction_id() + 1,
            };
            chunk.add_instruction(instruction_templates::cell_invoke(new_binding_id));
            graph_def.add_binding(binding);

            // go to next action.
            graph_next_action!(graph, current_node, graph_def, target_chunk);
        }
        _ => todo!("Implement node template: {:?}", current_node.user_data.template),
    }
    return -1;
}