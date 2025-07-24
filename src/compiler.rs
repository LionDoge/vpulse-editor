mod instruction_templates;
pub mod serialization;

use std::{fs, borrow::Cow};
use anyhow::anyhow;
use egui_node_graph2::*;
use crate::app::types::{
    PulseDataType, PulseGraph, PulseGraphState, PulseGraphValueType, PulseNodeData,
    PulseNodeTemplate,
};
use crate::bindings::LibraryBindingType;
use crate::pulsetypes::*;
use crate::typing::get_preffered_inputparamkind_from_type;
use crate::typing::PulseValueType;
use serialization::*;

#[cfg(feature = "nongame_asset_build")]
use std::{path::PathBuf, path, process::Command};
#[cfg(feature = "nongame_asset_build")]
use crate::app::types::EditorConfig;

macro_rules! graph_next_action {
    ($graph:ident, $current_node:ident, $graph_def:ident, $graph_state:ident, $target_chunk:ident) => {
        let connected_nodes = get_next_action_nodes($current_node, $graph, "outAction");
        if connected_nodes.is_ok() {
            for (connected_node, input_name) in connected_nodes.unwrap().iter() {
                return traverse_nodes_and_populate(
                    $graph,
                    connected_node,
                    $graph_def,
                    $graph_state,
                    $target_chunk,
                    &None,
                    &Some(Cow::Borrowed(input_name)),
                );
            }
        }
    };
}

macro_rules! graph_run_next_actions_no_return {
    ($graph:ident, $current_node:ident, $graph_def:ident, $graph_state:ident, $target_chunk:ident, $action_name:expr) => {{
        let connected_nodes = get_next_action_nodes($current_node, $graph, $action_name);
        if let Ok(connected_nodes) = connected_nodes {
            let mut any = false;
            for (connected_node, input_name) in connected_nodes.iter() {
                traverse_nodes_and_populate(
                    $graph,
                    connected_node,
                    $graph_def,
                    $graph_state,
                    $target_chunk,
                    &None,
                    &Some(Cow::Borrowed(input_name)),
                )?;
                any = true;
            }
            any
        } else {
            false
        }
    }};
}

macro_rules! get_constant_graph_input_value {
    ($graph:ident, $node:ident, $input:literal, $typ_func:ident) => {{
        let input_id = $node.get_input($input).map_err(|e| {
            anyhow::anyhow!(e).context(format!(
                "Get constant input value for {} node {:?}",
                $input, $node.user_data.template
            ))
        })?;
        let input_param = $graph.inputs.get(input_id).ok_or(
            anyhow!("Can't find input value of {}", $input).context(format!(
                "Get constant input value for node {:?}",
                $node.user_data.template
            )),
        )?;
        input_param.value.clone().$typ_func().map_err(|e| {
            anyhow::anyhow!(e).context(format!(
                "Get constant input value for {} node {:?}",
                $input, $node.user_data.template
            ))
        })?
    }};
}

macro_rules! get_connection_only_graph_input_value {
    ($graph:ident, $node: ident, $input:literal, $graph_def:ident, $graph_state:ident, $target_chunk:ident) => {{
        let input_id = $node.get_input($input).map_err(|e| {
            anyhow::anyhow!(e).context(format!(
                "Can't get input port {} node {:?}",
                $input, $node.user_data.template
            ))
        })?;
        let connection = $graph.connection(input_id);
        let result: i32 = if connection.is_some() {
            let connection = connection.unwrap();
            let param = $graph.get_output(connection);
            let out_node = $graph.nodes.get(param.node).ok_or(
                anyhow!("Can't find input value of {}", $input).context(format!(
                    "Get constant input value for node {:?}",
                    $node.user_data.template
                )),
            )?;
            traverse_nodes_and_populate(
                $graph,
                out_node,
                $graph_def,
                $graph_state,
                $target_chunk,
                &Some(connection),
                &None,
            )?
        } else {
            -1
        };
        result
    }};
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

#[allow(dead_code)]
fn get_connected_output_node(
    graph: &PulseGraph,
    out_action_id: &OutputId,
) -> anyhow::Result<Option<NodeId>> {
    // dumb way of finding outgoing connection node.
    for group in graph.iter_connection_groups() {
        for connection in group.1 {
            if connection == *out_action_id {
                let input_action: &InputParam<PulseDataType, PulseGraphValueType> =
                    graph.inputs.get(group.0).ok_or(
                        anyhow!("Can't find input value {:?}", group.0)
                            .context("get_connected_output_node"),
                    )?;
                return Ok(Some(input_action.node));
            }
        }
    }
    Ok(None)
}

// returns list of pairs of nodes and inputs connected to a given output.
// TODO could be optimized to return a group of node and it's inputs
fn get_connected_action_nodes_and_inputs(
    graph: &PulseGraph,
    out_action_id: &OutputId,
) -> anyhow::Result<Vec<(NodeId, InputId)>> {
    let mut node_input_pairs = vec![];
    for connection in graph.iter_connections() {
        if connection.1 == *out_action_id {
            let node_of_input = graph
                .inputs
                .get(connection.0)
                .ok_or(anyhow!("Can't find input value {:?}", connection.0))?
                .node();
            let input_id = connection.0;
            node_input_pairs.push((node_of_input, input_id));
        }
    }
    Ok(node_input_pairs)
}

#[allow(dead_code)]
fn get_next_action_node<'a>(
    origin_node: &'a Node<PulseNodeData>,
    graph: &'a PulseGraph,
    name: &str,
) -> anyhow::Result<Option<&'a Node<PulseNodeData>>> {
    let out_action_id = origin_node.get_output(name)?;
    let connected_node_id = get_connected_output_node(graph, &out_action_id)?;
    match connected_node_id {
        Some(id) => Ok(graph.nodes.get(id)),
        None => Ok(None),
    }
}

// return list of pairs of the connected node and corresponding name
fn get_next_action_nodes<'a>(
    origin_node: &'a Node<PulseNodeData>,
    graph: &'a PulseGraph,
    name: &str,
) -> anyhow::Result<Vec<(&'a Node<PulseNodeData>, &'a str)>> {
    let mut res = vec![];
    let out_action_id = origin_node.get_output(name)?;
    let connected_nodes_inputs = get_connected_action_nodes_and_inputs(graph, &out_action_id)?;
    for conn in connected_nodes_inputs.iter() {
        let node = graph.nodes.get(conn.0).ok_or_else(|| {
            anyhow::anyhow!("Node with id {:?} not found in the graph", conn.0)
                .context("get_next_action_nodes found NodeId, but couldn't get node data")
        })?;
        let input_name: &'a str = node
            .inputs
            .iter()
            .find(|item| item.1 == conn.1)
            .ok_or_else(|| {
                anyhow::anyhow!("Input with id {:?} not found in the node", conn.1)
                    .context("get_next_action_nodes found InputId, but couldn't get input data")
            })
            .map(|e| e.0.as_ref())?;
        res.push((node, input_name));
    }
    Ok(res)
}

// process all inflow nodes and logic chain.
// returns false if no inflow node was processed
fn traverse_inflow_nodes(
    graph: &PulseGraph,
    graph_def: &mut PulseGraphDef,
    _graph_state: &PulseGraphState,
) -> anyhow::Result<bool> {
    let mut processed: bool = false;
    for node in graph.iter_nodes() {
        let data: &Node<PulseNodeData> = graph.nodes.get(node).unwrap();
        // start at all possible entry points
        match data.user_data.template {
            PulseNodeTemplate::EventHandler => {
                processed = true;
                traverse_event_cell(graph, data, graph_def, _graph_state)?;
            }
            PulseNodeTemplate::CellPublicMethod => {
                processed = true;
                traverse_entry_cell(graph, data, graph_def, _graph_state)?;
            }
            PulseNodeTemplate::GraphHook => {
                processed = true;
                traverse_graphhook_cell(graph, data, graph_def, _graph_state)?;
            }
            _ => {}
        }
    }
    Ok(processed)
}

fn add_cell_invoke_binding(
    graph_def: &mut PulseGraphDef,
    register_map: RegisterMap,
    target_chunk: i32,
    func_name: Cow<'static, str>,
    cell_id: i32,
) -> i32 {
    let binding_id = graph_def.get_current_binding_id() + 1; // new binding id, it's here because of borrow checker
    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
    let instr = chunk.add_instruction(instruction_templates::cell_invoke(binding_id));
    // NOTE: Cell invokes require information about where they're been called from
    let binding = InvokeBinding {
        register_map,
        func_name,
        cell_index: cell_id, // the cell to be added
        src_chunk: target_chunk,
        src_instruction: instr,
    };
    graph_def.add_invoke_binding(binding)
}

fn add_cell_and_invoking(
    graph_def: &mut PulseGraphDef,
    cell: Box<dyn PulseCellTrait>,
    register_map: RegisterMap,
    target_chunk: i32,
    func_name: Cow<'static, str>,
) {
    graph_def.add_cell(cell);
    add_cell_invoke_binding(
        graph_def,
        register_map,
        target_chunk,
        func_name,
        graph_def.cells.len() as i32 - 1,
    );
}

fn add_library_invoking(
    graph_def: &mut PulseGraphDef,
    register_map: RegisterMap,
    target_chunk: i32,
    func_name: Cow<'static, str>,
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

fn add_call_reference(graph_def: &mut PulseGraphDef, src_chunk: i32, src_instruction: i32) -> i32 {
    // add call info, return the index
    // PULSE_CALL_SYNC instruction is required later to call it
    let call_info = CallInfo {
        port_name: "Call".into(),
        register_map: RegisterMap::default(), // no parameter support yet
        call_method_id: -1,
        src_chunk,
        src_instruction,
    };
    graph_def.add_call_info(call_info)
}

fn traverse_event_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    _graph_state: &PulseGraphState,
) -> anyhow::Result<()> {
    let input_id = node
        .get_input("event")
        .map_err(|e| anyhow!(e).context("Traverse event cell node"))?;
    let input_param = graph
        .inputs
        .get(input_id)
        .ok_or(anyhow!("Can't find input value").context("Traverse event cell node"))?;
    let event_binding_id = input_param.value.clone().try_event_binding_id()?;
    let event_binding = _graph_state
        .get_event_binding_from_index(&event_binding_id)
        .ok_or_else(|| anyhow::anyhow!("Event binding with id {} not found", event_binding_id))?;
    // create new pulse cell node.
    let chunk_id = graph_def.create_chunk();
    let mut cell_event =
        CPulseCell_Inflow_EventHandler::new(chunk_id, event_binding.libname.clone().into());

    // iterate all event params and add them as registers that can be used in the chunk
    // they will be all added even if no connections exist, but that's alright.
    if let Some(inparams) = &event_binding.inparams {
        for param in inparams.iter() {
            let output_id = node
                .get_output(param.name.as_str())
                .map_err(|e| anyhow!(e).context("Traverse event cell node"))?;

            let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
            let reg_id = chunk.add_register(param.pulsetype.to_string(), 0);
            cell_event.add_outparam(param.name.clone().into(), reg_id);
            graph_def.add_register_mapping(output_id, reg_id);
        }
    }

    graph_def.cells.push(Box::from(cell_event));
    let connected_node = get_next_action_nodes(node, graph, "outAction")?;
    for (connected_node, input_name) in connected_node.iter() {
        traverse_nodes_and_populate(
            graph,
            connected_node,
            graph_def,
            _graph_state,
            chunk_id,
            &None,
            &Some(Cow::Borrowed(*input_name)),
        )?;
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
    Ok(())
}

fn traverse_graphhook_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    _graph_state: &PulseGraphState,
) -> anyhow::Result<()> {
    let hook_name = get_constant_graph_input_value!(graph, node, "hookName", try_to_string);
    let chunk_id = graph_def.create_chunk();
    let cell_hook =
        CPulseCell_Inflow_GraphHook::new(hook_name.into(), RegisterMap::default(), chunk_id);
    graph_def.cells.push(Box::from(cell_hook));
    let connected_node = get_next_action_nodes(node, graph, "outAction")?;
    for (connected_node, input_name) in connected_node.iter() {
        traverse_nodes_and_populate(
            graph,
            connected_node,
            graph_def,
            _graph_state,
            chunk_id,
            &None,
            &Some(Cow::Borrowed(*input_name)),
        )?;
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
    Ok(())
}

// traverse a function node that can be referenced to call remotely.
// currently will either return a chunk id or a cell id to run depending on the context.
fn traverse_function_entry(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    _graph_state: &PulseGraphState,
) -> anyhow::Result<i32> {
    let existing_entrypoint = graph_def
        .traversed_entrypoints
        .iter()
        .find(|&x| x.0 == node.id);
    if existing_entrypoint.is_none() {
        let chunk_id = graph_def.create_chunk();
        let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
        let ret_value;
        // node specific thingies.
        match node.user_data.template {
            PulseNodeTemplate::ListenForEntityOutput => {
                let reg_id_activator = chunk.add_register(String::from("PVAL_EHANDLE"), 0);
                let output_id_activator = node.get_output("pActivator")?;
                graph_def.add_register_mapping(output_id_activator, reg_id_activator);

                let mut reg_map = RegisterMap::default();
                reg_map.add_outparam("pActivator".into(), reg_id_activator);
                let outflow_onfired = OutflowConnection {
                    outflow_name: "OnFired".into(),
                    dest_chunk: chunk_id,
                    dest_instruction: 0,
                    register_map: Some(reg_map),
                };
                let cell_listen = CPulseCell_Outflow_ListenForEntityOutput {
                    outflow_onfired,
                    outflow_oncanceled: OutflowConnection::default(),
                    entity_output: get_constant_graph_input_value!(
                        graph,
                        node,
                        "outputName",
                        try_to_string
                    ),
                    entity_output_param: get_constant_graph_input_value!(
                        graph,
                        node,
                        "outputParam",
                        try_to_string
                    ),
                    listen_until_canceled: get_constant_graph_input_value!(
                        graph,
                        node,
                        "bListenUntilCanceled",
                        try_to_bool
                    ),
                };
                graph_def.cells.push(Box::from(cell_listen));
                ret_value = graph_def.cells.len() as i32 - 1;
            }
            PulseNodeTemplate::Function => {
                ret_value = chunk_id;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported node type for function entry: {:?}",
                    node.user_data.template
                ));
            }
        };
        // remember that we traversed this already!
        graph_def.traversed_entrypoints.push((node.id, ret_value));
        graph_run_next_actions_no_return!(
            graph,
            node,
            graph_def,
            _graph_state,
            chunk_id,
            "outAction"
        );
        let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
        chunk.add_instruction(instruction_templates::return_void());
        Ok(ret_value)
    } else {
        // we already traversed this entrypoint, so we can just return the chunk id
        Ok(existing_entrypoint.unwrap().1)
    }
}

fn traverse_entry_cell(
    graph: &PulseGraph,
    node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    _graph_state: &PulseGraphState,
) -> anyhow::Result<()> {
    let mut cell_method = CPulseCell_Inflow_Method::default();
    let chunk_id = graph_def.create_chunk();
    cell_method.name = get_constant_graph_input_value!(graph, node, "name", try_to_string);
    cell_method.entry_chunk = chunk_id;
    cell_method.return_type = String::from("PVAL_INVALID");

    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    // create argument1 (TODO only if connection exists)
    let reg_id_arg1 = chunk.add_register(String::from("PVAL_STRING"), 0);
    let output_id_arg1 = node
        .get_output("argument1")
        .map_err(|e| anyhow!(e).context("Traverse public method node"))?;
    cell_method.add_arg(
        String::from("arg1"),
        String::default(),
        String::from("PVAL_STRING"),
        reg_id_arg1,
    );
    graph_def.add_register_mapping(output_id_arg1, reg_id_arg1);
    graph_def.cells.push(Box::from(cell_method));

    let connected_node = get_next_action_nodes(node, graph, "outAction")?;
    for (connected_node, input_name) in connected_node.iter() {
        traverse_nodes_and_populate(
            graph,
            connected_node,
            graph_def,
            _graph_state,
            chunk_id,
            &None,
            &Some(Cow::Borrowed(*input_name)),
        )?;
    }
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    chunk.add_instruction(instruction_templates::return_void());
    Ok(())
}

pub fn compile_graph(
    graph: &PulseGraph,
    graph_state: &PulseGraphState,
    #[cfg(feature = "nongame_asset_build")]
    config: &EditorConfig,
) -> anyhow::Result<()> {
    let mut graph_def = PulseGraphDef::default();
    let file_dir = graph_state
        .save_file_path
        .as_ref()
        .ok_or(anyhow!("File needs to be saved before compiling"))?;
    graph_def.variables = graph_state.variables.clone();
    graph_def.public_outputs = graph_state.public_outputs.clone();
    graph_def.map_name = String::from("maps/main.vmap");
    graph_def.xml_name = String::default();

    match traverse_inflow_nodes(graph, &mut graph_def, graph_state) {
        Ok(true) => {
            // we found inflow nodes, so we can continue
        }
        Ok(false) => {
            anyhow::bail!("No inflow nodes found in graph");
        }
        Err(e) => {
            anyhow::bail!("Graph compile failed: {}", e);
        }
    }
    let data = graph_def.serialize();
    let _ = fs::create_dir_all(file_dir).map_err(|e| {
        anyhow!(
            "Graph compile failed: Failed to create output directory: {}",
            e
        )
    });

    #[cfg(not(feature = "nongame_asset_build"))] {
        let mut file_path = file_dir.clone();
        file_path.set_extension("vpulse");
        fs::write(file_path, data)
            .map_err(|e| anyhow!("Graph compile failed: Failed to write to file: {}", e))?
    }

    #[cfg(feature = "nongame_asset_build")] {
        let file_name = file_dir.file_name().ok_or_else(|| {
            anyhow::anyhow!("The provided file source path doesn't contain a filename, please re-save the file: '{}'", file_dir.display())
        })?;
        // create a temporary file in the system temp directory with random suffix to avoid confilicts (very unlikely anyways)
        use rand::{Rng, distributions::Alphanumeric};
        let temp_dir_file = std::env::temp_dir().join(
            format!("{}_{}.vpulse", file_name.display(), 
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect::<String>()
        ));
        fs::write(&temp_dir_file, data)
            .map_err(|e| anyhow!("Graph compile failed: Failed to write to file: {}", e))?;
        run_asset_builder(config, &temp_dir_file, file_dir)
            .map_err(|e| anyhow!("Graph compile failed: Failed to run asset builder: {}", e))?;
        let _ = fs::remove_file(&temp_dir_file); // ok to ignore
    }
    Ok(())
}

#[cfg(feature = "nongame_asset_build")]
fn get_output_path(original_path: &path::Path) -> anyhow::Result<PathBuf> {
    let mut out_path = original_path;
    let mut addon_dir: Option<path::Component<'_>> = None;
    let mut components = out_path.components();
    // this will be used to reconstruct the inner file directory after the game directory is determined (if it will be).
    let mut popped_components = vec![];
    components.next(); // skip root directory
    for curr_dir in components.rev() {
        if curr_dir.as_os_str() == "content" {
            // found the content directory, see if we have corresponding addon directory in the game path.
            let new_path = out_path.parent().unwrap();
            if addon_dir.is_some() {
                let addon_game_path = new_path.join("game");
                if fs::exists(&addon_game_path)? {
                    // Reconstruct the path (after game/addon/...)
                    let mut full_path = addon_game_path;
                    let reconstructed: PathBuf = popped_components.iter().rev().collect();
                    full_path.push(reconstructed);
                    fs::create_dir_all(&full_path)?;
                    return Ok(full_path);
                }
            }
        }
        // next iteration it will be the previous directory from back.
        addon_dir = Some(curr_dir);
        // should be ok, since the root path is removed, thus the parent result should never go beyond that.
        popped_components.push(out_path.iter().next_back().unwrap());
        // If parent() returns None, then we can't go further
        match out_path.parent() {
            Some(parent) => out_path = parent,
            None => break,
        }
    }
    // we reached root directory without finding 'content', or a valid existing addon directory.
    Ok(original_path.into())
}

#[cfg(feature = "nongame_asset_build")]
fn run_asset_builder(config: &EditorConfig, path_src: &path::Path, path_editor_file: &path::Path) -> anyhow::Result<()> {
    println!("Running asset assembler for file: {}", path_src.display());
    let assetbuilder_path = config.assetassembler_path.as_path();
    let red2_path = config.red2_template_path.as_path();
    if !assetbuilder_path.exists() || !assetbuilder_path.is_file() {
        return Err(anyhow::anyhow!(
            "Asset assembler location was specified incorrectly in the config: {}",
            assetbuilder_path.display()
        ));
    }
    if !red2_path.exists() || !red2_path.is_file() {
        return Err(anyhow::anyhow!(
            "RED2 template location was specified incorrectly in the config: {}",
            red2_path.display()
        ));
    }
    if !path_editor_file.exists() || !path_editor_file.is_file() {
        return Err(anyhow::anyhow!("File needs to be saved before compilation"));
    }
    let file_name = path_editor_file.file_name().ok_or_else(|| {
        anyhow::anyhow!("Can't get file name from the path: {}", path_src.display())
    })?;
    // get rid of file name for the output path
    let mut out_file = get_output_path(path_editor_file.parent().unwrap())?.join(file_name);
    out_file.set_extension("vpulse_c");
    println!("Determined full output path: {}", out_file.display());
    let mut process = Command::new(config.python_interpreter.as_str())
        .arg(assetbuilder_path)
        .arg("-p")
        .arg("vpulse")
        .arg("-f")
        .arg(red2_path)
        .arg(path_src)
        .arg("-o")
        .arg(out_file.as_os_str())
        .spawn()?;
    // it's fine to freeze the UI here, because recompiling while loading could lead to issues.
    // this process should usually be fast enough to not be very noticable.
    process.wait()?;
    Ok(())
}

fn try_find_output_mapping(graph_def: &PulseGraphDef, output_id: &Option<OutputId>) -> i32 {
    match output_id {
        Some(output_id) => {
            match graph_def.get_mapped_reigster(*output_id) {
                Some(reg) => {
                    // we found a mapping! So we know which register to use for this
                    *reg
                }
                None => -1,
            }
        }
        None => -1,
    }
}

fn get_variable(graph_def: &mut PulseGraphDef, name: &str) -> Option<i32> {
    let var = graph_def.get_variable_index(name);
    if let Some(var) = var {
        return Some(var as i32);
    }
    None
}

fn try_find_input_mapping(graph_def: &PulseGraphDef, input_id: Option<&InputId>) -> Option<i32> {
    input_id.and_then(|id| graph_def.get_mapped_reigster_input(*id).copied())
}

// traverse to the neihbors of the current node, connected to inputs, and gather their information
// can choose if the generated value from the input will be reused, or if it should always be evaluated as new
// as a new input (depends on the task of the node really)
#[allow(clippy::too_many_arguments)]
fn get_input_register_or_create_constant(
    graph: &PulseGraph,
    current_node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    graph_state: &PulseGraphState,
    chunk_id: i32,
    input_name: &str,
    value_type: PulseValueType,
    always_reevaluate: bool,
) -> anyhow::Result<Option<i32>> {
    let input_id = current_node.get_input(input_name).map_err(|e| {
        anyhow::anyhow!(e).context(format!("{:?} node", &current_node.user_data.template))
    })?;
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
                .ok_or(anyhow!("Can't find output node"))?;
            target_register = traverse_nodes_and_populate(
                graph,
                out_node,
                graph_def,
                graph_state,
                chunk_id,
                &Some(out),
                &None,
            )?;
        }
        None => {
            if !always_reevaluate {
                // no connection found, create a constant value for the input
                // but first check if we have already created a constant for this value
                let inp_mapping = try_find_input_mapping(graph_def, Some(&input_id));
                if inp_mapping.is_some() {
                    return Ok(inp_mapping);
                }
            }
            if matches!(
                get_preffered_inputparamkind_from_type(&value_type),
                InputParamKind::ConnectionOnly
            ) {
                println!("[INFO] Connection only input type without a connection, no constant will be created. for type: {value_type}, input: {input_name}");
                return Ok(None);
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
                    let input_value = input_param.value().clone().try_to_scalar()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Integer(input_value as i32));
                }
                PulseValueType::PVAL_FLOAT(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_scalar()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Float(input_value));
                }
                PulseValueType::PVAL_STRING(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_string()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::String(input_value));
                }
                PulseValueType::PVAL_SNDEVT_NAME(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_sndevt_name()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::SoundEventName(input_value));
                }
                PulseValueType::DOMAIN_ENTITY_NAME => {
                    instruction =
                        instruction_templates::get_domain_value(target_register, new_domain_val_id);
                    let input_value = input_param.value().clone().try_entity_name()?;
                    chunk.add_instruction(instruction);
                    graph_def.create_domain_value(
                        "ENTITY_NAME".into(),
                        input_value.into(),
                        "".into(),
                        "PVAL_ENTITY_NAME".into(),
                    );
                }
                PulseValueType::PVAL_VEC3(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_vec3()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Vec3(input_value));
                }
                PulseValueType::PVAL_COLOR_RGB(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let mut input_value = input_param.value().clone().try_to_color_rgba()?;
                    chunk.add_instruction(instruction);
                    input_value[0] *= 255.0;
                    input_value[1] *= 255.0;
                    input_value[2] *= 255.0;
                    graph_def.add_constant(PulseConstant::Color_RGB(input_value));
                }
                PulseValueType::PVAL_BOOL
                | PulseValueType::PVAL_BOOL_VALUE(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_value = input_param.value().clone().try_to_bool()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::Bool(input_value));
                }
                PulseValueType::PVAL_SCHEMA_ENUM(_) => {
                    instruction =
                        instruction_templates::get_const(new_constant_id, target_register);
                    let input_typ_and_value = input_param.value().clone().try_enum()?;
                    chunk.add_instruction(instruction);
                    graph_def.add_constant(PulseConstant::SchemaEnum(input_typ_and_value.0, input_typ_and_value.1));
                }
                // Having a constant value for these doesn't make sense.
                PulseValueType::PVAL_EHANDLE(_) | PulseValueType::PVAL_SNDEVT_GUID(_) => {
                    return Ok(None);
                }
                _ => {
                    println!("Warning: Unsupported constant value type for input - None will be returned {input_name}: {value_type}");
                    return Ok(None);
                    // if we don't know the type, we can't create a constant for it.
                }
            };
            graph_def.add_register_mapping_input(input_id, target_register);
        }
    }
    Ok(Some(target_register))
}

// recurse along connected nodes, and generate instructions, cells, and bindings depending on the node type.
// takes care of referencing already assigned registers or other data (like visisted list in a graph traversal)
// it operates ONLY on a target chunk - which is basically a set of instructions related to one flow of logic
// inside the GUI a chunk is one continous flow of logic.
fn traverse_nodes_and_populate<'a>(
    graph: &PulseGraph,
    current_node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    graph_state: &PulseGraphState,
    target_chunk: i32,
    output_id: &Option<OutputId>, // if this is Some, then this was called by a node requesting a value, (not action)
    source_input_name: &Option<Cow<'a, str>>, // mostly useful for traversing to next actions. It lets know about what action was specified for nodes that have multiple action inputs
) -> anyhow::Result<i32> {
    match current_node.user_data.template {
        PulseNodeTemplate::CellPublicMethod => {
            // here we resolve connections to the argument outputs
            return Ok(try_find_output_mapping(graph_def, output_id));
        }
        PulseNodeTemplate::EventHandler => {
            // here we resolve connections to the argument outputs
            return Ok(try_find_output_mapping(graph_def, output_id));
        }
        PulseNodeTemplate::CellWait => {
            let reg_time = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "time",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;

            let register_map = reg_map_setup_inputs!("flDurationSec", reg_time);
            let cell = CPulseCell_Inflow_Wait::new(
                target_chunk,
                graph_def.get_chunk_last_instruction_id(target_chunk) + 3,
            );
            add_cell_and_invoking(
                graph_def,
                Box::from(cell),
                register_map,
                target_chunk,
                "Wait".into(),
            );
            // early return.
            let instr_ret_void = Instruction {
                code: String::from("RETURN_VOID"),
                ..Default::default()
            };
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            chunk.add_instruction(instr_ret_void);

            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::EntFire => {
            let reg_entity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "entity",
                PulseValueType::DOMAIN_ENTITY_NAME,
                false,
            )?;
            let input_value =
                get_constant_graph_input_value!(graph, current_node, "input", try_to_string);
            // this one might be empty, but we want to use it for OutputConnection if we know it at compile time.
            let entity_name_static_value =
                get_constant_graph_input_value!(graph, current_node, "entity", try_entity_name);

            // check for existence of the parameter value (connection, or non empty string)
            // to determine if we need to add it to the EntFire call
            let param_value_exists = 'checkParmValue: {
                let input_id = current_node.get_input("value")?;
                let connection_to_input: Option<OutputId> = graph.connection(input_id);
                if connection_to_input.is_some() {
                    break 'checkParmValue true;
                }

                let val = graph
                    .get_input(input_id)
                    .value()
                    .clone()
                    .try_to_string()
                    .unwrap();
                break 'checkParmValue !val.is_empty();
            };

            let mut reg_param = None;
            if param_value_exists {
                reg_param = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "value",
                    PulseValueType::PVAL_STRING(None),
                    false,
                )?;
            }

            // add invoke binding for FireAtName cell
            let register_map = reg_map_setup_inputs!("TargetName", reg_entity, "pParam", reg_param);
            let cell = CPulseCell_Step_EntFire::new(input_value.clone().into());
            add_cell_and_invoking(
                graph_def,
                Box::from(cell),
                register_map,
                target_chunk,
                "FireAtName".into(),
            );
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

            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::ConcatString => {
            let id_a = current_node
                .get_input("A")
                .map_err(|e| anyhow!(e).context("ConcatString node"))?;
            let id_b = current_node
                .get_input("B")
                .map_err(|e| anyhow!(e).context("ConcatString node"))?;
            let input_ids = [id_a, id_b];
            let connection_to_a = graph.connection(id_a);
            let connection_to_b = graph.connection(id_b);
            let connections_to_resolve: [Option<OutputId>; 2] = [connection_to_a, connection_to_b];
            let mut input_registers: [i32; 2] = [-1, -1];

            for (i, connection) in connections_to_resolve.iter().enumerate() {
                match connection {
                    Some(out) => {
                        let out_param = graph.get_output(*out);
                        let out_node = graph.nodes.get(out_param.node).ok_or(
                            anyhow!("Can't find output node").context("ConcatString node"),
                        )?;
                        // grab the register that the value will come from.
                        input_registers[i] = traverse_nodes_and_populate(
                            graph,
                            out_node,
                            graph_def,
                            graph_state,
                            target_chunk,
                            &Some(*out),
                            source_input_name,
                        )?;
                    }
                    None => {
                        // no connection.. First search if we already created it, if not create the constant input value
                        let register = try_find_input_mapping(graph_def, Some(&input_ids[i]));
                        if register.is_none() {
                            let input_info: &InputParam<PulseDataType, PulseGraphValueType> =
                                graph.get_input(input_ids[i]);
                            let constant =
                                PulseConstant::String(input_info.value.clone().try_to_string()?);
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
            return Ok(register);
        }
        PulseNodeTemplate::GetVar => {
            let name_id = current_node
                .get_input("variableName")
                .map_err(|e| anyhow!(e).context("GetVar node"))?;
            // name is a constant value
            let name = graph
                .get_input(name_id)
                .value()
                .clone()
                .try_variable_name()
                .map_err(|e| anyhow!(e).context("GetVar node"))?;
            let var_id = get_variable(graph_def, name.as_str());
            if var_id.is_none() {
                anyhow::bail!(
                    anyhow!("Variable {name} not found in variables list").context("GetVar node")
                );
            }
            let var_id = var_id.unwrap();
            let typ = graph_def
                .variables
                .get(var_id as usize)
                .ok_or(
                    anyhow!("Variable id: {var_id} not found in variables list")
                        .context("GetVar node"),
                )?
                .typ_and_default_value
                .to_string();
            // add register
            // add instruction to load the variable value
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg = chunk.add_register(typ, chunk.get_last_instruction_id() + 1);
            chunk.add_instruction(instruction_templates::get_var(reg, var_id));
            return Ok(reg);
        }
        PulseNodeTemplate::IntToString => {
            let value_id = current_node
                .get_input("value")
                .map_err(|e| anyhow!(e).context("IntToString node"))?;
            let connection_to_value = graph.connection(value_id);
            let register_input: i32;
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph
                        .nodes
                        .get(out_param.node)
                        .ok_or(anyhow!("Can't find output node").context("IntToString node"))?;
                    // grab the register that the value will come from.
                    register_input = traverse_nodes_and_populate(
                        graph,
                        out_node,
                        graph_def,
                        graph_state,
                        target_chunk,
                        &Some(out),
                        source_input_name,
                    )?;
                }
                None => {
                    print!("No connection found for input value for IntToString node");
                    return Ok(-1);
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
            return Ok(register);
        }
        PulseNodeTemplate::SetVar => {
            let name_id = current_node
                .get_input("variableName")
                .map_err(|e| anyhow!(e).context("SetVar node"))?;
            // name is a constant value
            let name = graph
                .get_input(name_id)
                .value()
                .clone()
                .try_variable_name()
                .map_err(|e| anyhow!(e).context("GetVar node"))?;
            let var_id = get_variable(graph_def, name.as_str());
            if var_id.is_none() {
                anyhow::bail!(
                    anyhow!("Variable {name} not found in variables list").context("SetVar node")
                );
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
                graph_state,
                target_chunk,
                "value",
                typ,
                false,
            )?;
            if let Some(reg_value) = reg_value {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::set_var(reg_value, var_id.unwrap()));
            }

            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::Operation => {
            let existing_reg_mapping = try_find_output_mapping(graph_def, output_id);
            if existing_reg_mapping != -1 {
                return Ok(existing_reg_mapping);
            }
            let operation_typ =
                get_constant_graph_input_value!(graph, current_node, "type", try_pulse_type);
            let reg_a = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "A",
                operation_typ.clone(),
                false,
            )?;
            let reg_b = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "B",
                operation_typ.clone(),
                false,
            )?;
            let operation_input_param =
                get_constant_graph_input_value!(graph, current_node, "operation", try_to_string);
            let operation_suffix = operation_typ.get_operation_suffix_name();
            let operation_instr_name: String = match operation_input_param.as_str() {
                "+" => format!("ADD{operation_suffix}"),
                "-" => format!("SUB{operation_suffix}"),
                "*" => format!("MUL{operation_suffix}"),
                "/" => format!("DIV{operation_suffix}"),
                "%" => format!("MOD{operation_suffix}"),
                _ => format!("ADD{operation_suffix}"),
            };
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let register_output = chunk.add_register(
                operation_typ.to_string(),
                chunk.get_last_instruction_id() + 1,
            );
            let instr = Instruction {
                code: operation_instr_name,
                reg0: register_output,
                reg1: reg_a.unwrap_or(-1),
                reg2: reg_b.unwrap_or(-1),
                ..Default::default()
            };
            chunk.add_instruction(instr);
            if let Some(output) = output_id {
                graph_def.add_register_mapping(*output, register_output);
            }
            return Ok(register_output);
        }
        PulseNodeTemplate::FindEntByName => {
            let entclass_input_id = current_node
                .get_input("entClass")
                .map_err(|e| anyhow!(e).context("FindEntByName node"))?;
            let entclass_input_param = graph
                .get_input(entclass_input_id)
                .value()
                .clone()
                .try_to_string()
                .map_err(|e| anyhow!(e).context("FindEntByName node"))?;
            let mut reg_output = try_find_output_mapping(graph_def, output_id);
            let reg_entname = if reg_output == -1 {
                get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "entName",
                    PulseValueType::DOMAIN_ENTITY_NAME,
                    false,
                )?
            } else {
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
                return Ok(reg_output);
            }
            let mut register_map = reg_map_setup_inputs!("pName", reg_entname);
            register_map.add_outparam("retval".into(), reg_output);
            let cell = CPulseCell_Value_FindEntByName::new(entclass_input_param.into());
            add_cell_and_invoking(
                graph_def,
                Box::from(cell),
                register_map,
                target_chunk,
                "Eval".into(),
            );
            return Ok(reg_output);
        }
        PulseNodeTemplate::DebugWorldText => {
            let reg_message = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "pMessage",
                PulseValueType::PVAL_STRING(None),
                false,
            )?;
            // resolve connection to hEntity
            let hentity_input_id = current_node
                .get_input("hEntity")
                .map_err(|e| anyhow!(e).context("DebugWorldText node"))?;
            let connection_to_hentity = graph.connection(hentity_input_id);
            if connection_to_hentity.is_none() {
                println!("No connection found for hEntity input in DebugWorldText node. Node will not be processed, next action won't execute.");
                return Ok(-1);
            }
            let reg_hentity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "hEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            )?;
            // other params
            let reg_ntextoffset = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "nTextOffset",
                PulseValueType::PVAL_INT(None),
                false,
            )?;
            let reg_flduration = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "flDuration",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
            let reg_flverticaloffset = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "flVerticalOffset",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
            // color:
            let reg_color = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "color",
                PulseValueType::PVAL_COLOR_RGB(None),
                false,
            )?;
            let reg_alpha = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "flAlpha",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
            let reg_scale = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "flScale",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
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
                "hEntity",
                reg_hentity,
                "pMessage",
                reg_message,
                "nTextOffset",
                reg_ntextoffset,
                "flDuration",
                reg_flduration,
                "flVerticalOffset",
                reg_flverticaloffset,
                "color",
                reg_color,
                "flAlpha",
                reg_alpha,
                "flScale",
                reg_scale
            );
            register_map.add_inparam("bAttached".into(), reg_battached);
            add_library_invoking(
                graph_def,
                register_map,
                target_chunk,
                "CPulseServerFuncs!DebugWorldText".into(),
            );

            // go to next action.
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::DebugLog => {
            let reg_message = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "pMessage",
                PulseValueType::PVAL_STRING(None),
                false,
            )?;
            graph_def.cells.push(Box::from(CPulseCell_Step_DebugLog));
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
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::FireOutput => {
            let input_id = current_node
                .get_input("outputName")
                .map_err(|e| anyhow!(e).context("FireOutput node"))?;
            let input_val = graph
                .get_input(input_id)
                .value()
                .clone()
                .try_output_name()
                .map_err(|e| anyhow!(e).context("FireOutput node"))?;
            let pub_output = graph_def.get_public_output_index(input_val.as_str());
            if let Some(pub_output) = pub_output {
                graph_def
                    .cells
                    .push(Box::from(CPulseCell_Step_PublicOutput::new(
                        pub_output as i32,
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
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
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
            return Ok(reg_output);
        }
        PulseNodeTemplate::SetNextThink => {
            let reg_dt = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "dt",
                PulseValueType::PVAL_STRING(None),
                false,
            )?;
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

            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::Convert => {
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let type_from = get_constant_graph_input_value!(
                    graph,
                    current_node,
                    "typefrom",
                    try_pulse_type
                );
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
                    graph_state,
                    target_chunk,
                    "input",
                    type_from.clone(),
                    false,
                )?;
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register =
                    chunk.add_register(type_to.to_string(), chunk.get_last_instruction_id() + 1);
                if let Some(reg_input) = reg_input {
                    let instruction = instruction_templates::convert_value(register, reg_input);
                    chunk.add_instruction(instruction);
                }
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return Ok(register);
        }
        PulseNodeTemplate::Compare => {
            let compare_type =
                get_constant_graph_input_value!(graph, current_node, "type", try_pulse_type);
            let reg_a = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "A",
                compare_type.clone(),
                false,
            )?;
            let reg_b = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "B",
                compare_type.clone(),
                false,
            )?;
            // TODO: only EQ for now
            let mut instr_compare = Instruction::default();
            instr_compare.code = format!("EQ{}", compare_type.get_operation_suffix_name());
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
            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "True"
            );
            // have to reborrow the chunk after we did borrow of graph_def.
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let false_condition_instr_id = chunk.get_last_instruction_id() + 2;
            // jump over the false condition
            let instr_jump_end = instruction_templates::jump(-1);
            let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "False"
            );
            // aaand borrow yet again lol
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // for now we just return. But we could have a 3rd port, that executes actions after doing the one in the chosen condition.
            chunk.add_instruction(instruction_templates::return_void());
            let ending_instr_id = chunk.get_last_instruction_id();
            let instr_jump_false = chunk.get_instruction_from_id_mut(jump_false_instr_id);
            if instr_jump_false.is_some() {
                instr_jump_false.unwrap().dest_instruction = false_condition_instr_id;
            } else {
                anyhow::bail!(
                    "Compare node: Failed to find JUMP[false_condition] with id: {}",
                    jump_false_instr_id
                );
            }
            let instr_jump_end = chunk.get_instruction_from_id_mut(jump_end_instr_id);
            if instr_jump_end.is_some() {
                instr_jump_end.unwrap().dest_instruction = ending_instr_id;
            } else {
                anyhow::bail!(
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
                graph_state,
                target_chunk
            );
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let instr_jump_cond =
                instruction_templates::jump_cond(reg_cond, chunk.get_last_instruction_id() + 3);
            chunk.add_instruction(instr_jump_cond);
            let instr_jump_false = instruction_templates::jump(-1); // the id is yet unknown. Note this instruction id, and modify the instruction later.
            let jump_false_instr_id = chunk.add_instruction(instr_jump_false);
            // instruction set for the true condition (if exists)
            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "True"
            );
            // have to reborrow the chunk after we did borrow of graph_def.
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let false_condition_instr_id = chunk.get_last_instruction_id() + 2;
            // jump over the false condition
            let instr_jump_end = instruction_templates::jump(-1);
            let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

            if !graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "False"
            ) {
                // if no actions were run, we still need to add an empty instruction, so we can jump to it.
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(Instruction::default());
            }
            // aaand borrow yet again lol
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // for now we just return. But we could have a 3rd port, that executes actions after doing the one in the chosen condition.
            let ending_instr_id = chunk.get_last_instruction_id() + 1;
            let instr_jump_false = chunk.get_instruction_from_id_mut(jump_false_instr_id);
            if instr_jump_false.is_some() {
                instr_jump_false.unwrap().dest_instruction = false_condition_instr_id;
            } else {
                anyhow::bail!(
                    "Compare node: Failed to find JUMP[false_condition] with id: {}",
                    jump_false_instr_id
                );
            }
            let instr_jump_end = chunk.get_instruction_from_id_mut(jump_end_instr_id);
            if instr_jump_end.is_some() {
                instr_jump_end.unwrap().dest_instruction = ending_instr_id;
            } else {
                anyhow::bail!(
                    "Compare node: Failed to find JUMP[end] with id: {}",
                    jump_end_instr_id
                );
            }
        }
        PulseNodeTemplate::ForLoop => {
            if output_id.is_some() {
                let reg_idx = try_find_output_mapping(graph_def, output_id);
                if reg_idx != -1 {
                    return Ok(reg_idx);
                } else {
                    println!("[WARN] ForLoop node: Failed to find output register for 'index' when a node requested it.
                    This means that the connected node tried to get the value, before the loop node had it's logic generated by an inflow action.");
                    return Ok(-1);
                }
            }
            let reg_from = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "start",
                PulseValueType::PVAL_INT(None),
                false,
            );
            let reg_to = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "end",
                PulseValueType::PVAL_INT(None),
                false,
            );
            let reg_step = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "step",
                PulseValueType::PVAL_INT(None),
                false,
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
            let reg_idx = chunk.add_register(
                String::from("PVAL_INT"),
                chunk.get_last_instruction_id() + 1,
            );
            // remember the output index, for nodes that want to access this output
            let output_idx_id = current_node
                .get_output("index")
                .map_err(|e| anyhow!(e).context("ForLoop node"))?;
            graph_def.add_register_mapping(output_idx_id, reg_idx);
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg_from = reg_from?.ok_or(anyhow!(
                "ForLoop node: Input 'from' somehow ended up resulting to None!"
            ))?;
            let instr_copy = instruction_templates::copy_value(reg_idx, reg_from);
            chunk.add_instruction(instr_copy);
            let reg_cond = chunk.add_register(
                String::from("PVAL_BOOL"),
                chunk.get_last_instruction_id() + 1,
            );
            let mut instr_compare = Instruction::default();
            let reg_to = reg_to?.ok_or(anyhow!(
                "ForLoop node: Input 'to' somehow ended up resulting to None!"
            ))?;
            instr_compare.code = String::from("LTE_INT");
            instr_compare.reg0 = reg_cond;
            instr_compare.reg1 = reg_idx;
            instr_compare.reg2 = reg_to;
            let instr_compare_id = chunk.add_instruction(instr_compare);
            // jump over the unconditional jump to the end
            let instr_jump_cond =
                instruction_templates::jump_cond(reg_cond, chunk.get_last_instruction_id() + 3);
            chunk.add_instruction(instr_jump_cond);
            let instr_jump_end = instruction_templates::jump(-1);
            let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "loopAction"
            );
            // borrow again (we know that it still is fine)
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // increment the index by step
            let reg_step = reg_step?.ok_or(anyhow!(
                "ForLoop node: Input 'step' somehow ended up resulting to None!"
            ))?;
            let instr_add = instruction_templates::add_value(reg_idx, reg_step, reg_idx);
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
                anyhow::bail!(
                    "ForLoop node: Failed to find JUMP[end] with id: {}",
                    jump_end_instr_id
                );
            }

            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "endAction"
            );
        }
        PulseNodeTemplate::WhileLoop => {
            let is_dowhile_loop =
                get_constant_graph_input_value!(graph, current_node, "do-while", try_to_bool);

            if !is_dowhile_loop {
                // While loop:
                // JUMP_COND{reg_condition == true}[curr + 3] (over the next jump)
                // JUMP[end] (to the end)
                // instructions
                // JUMP[evaluator] (to the condition evaluation)

                // save the current instruction id before populating the instruction for the condition
                // this will be the instruction that will be used to jump to the condition check
                let cond_instr_id = graph_def
                    .chunks
                    .get_mut(target_chunk as usize)
                    .unwrap()
                    .get_last_instruction_id()
                    + 1;
                let reg_condition = get_connection_only_graph_input_value!(
                    graph,
                    current_node,
                    "condition",
                    graph_def,
                    graph_state,
                    target_chunk
                );
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr_jump_cond = instruction_templates::jump_cond(
                    reg_condition,
                    chunk.get_last_instruction_id() + 3,
                );
                chunk.add_instruction(instr_jump_cond);
                let instr_jump_end = instruction_templates::jump(-1);
                let jump_end_instr_id = chunk.add_instruction(instr_jump_end);

                graph_run_next_actions_no_return!(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "loopAction"
                );
                // reborrow the chunk after we did borrow of graph_def.
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr_jump = instruction_templates::jump(cond_instr_id);
                chunk.add_instruction(instr_jump);
                chunk
                    .get_instruction_from_id_mut(jump_end_instr_id)
                    .unwrap()
                    .dest_instruction = chunk.get_last_instruction_id() + 1;
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
                graph_run_next_actions_no_return!(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "loopAction"
                );
                // next do all the condition check instructions
                let reg_condition = get_connection_only_graph_input_value!(
                    graph,
                    current_node,
                    "condition",
                    graph_def,
                    graph_state,
                    target_chunk
                );
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                // jump back if condition is true
                let instr_jump_cond =
                    instruction_templates::jump_cond(reg_condition, loop_action_instructions_start);
                chunk.add_instruction(instr_jump_cond);
            }

            // after loop is finished (and if something is connected here) proceed.
            graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "endAction"
            );
        }
        PulseNodeTemplate::StringToEntityName => {
            let reg_input = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "entityName",
                PulseValueType::PVAL_STRING(None),
                false,
            )?;
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
            return Ok(reg_out);
        }
        PulseNodeTemplate::InvokeLibraryBinding => {
            let binding_idx = get_constant_graph_input_value!(
                graph,
                current_node,
                "binding",
                try_library_binding
            );
            let binding = graph_state.get_library_binding_from_index(&binding_idx);
            if binding.is_none() {
                anyhow::bail!(
                    "InvokeLibraryBinding node: Failed to find library binding with index {}",
                    binding_idx
                );
            }
            let binding = binding.unwrap();
            let mut register_map = RegisterMap::default();
            if let Some(inparams) = &binding.inparams {
                for param in inparams.iter() {
                    let inp = get_input_register_or_create_constant(
                        graph,
                        current_node,
                        graph_def,
                        graph_state,
                        target_chunk,
                        &param.name,
                        param.pulsetype.clone(),
                        false,
                    )?;
                    // if the input is connection only, and it happens to be unconnected, we don't want to add it to the register map.
                    if let Some(inp) = inp {
                        // check if we need to reinterpet ehandle instance type
                        let inp = match &param.pulsetype {
                            PulseValueType::PVAL_EHANDLE(subtype) => {
                                let inp = if let Some(entclass) = subtype {
                                    // TODO: check if we actually need to do it if the source register already has a matching subtype, however redundant conversion shouldn't be an issue
                                    let chunk =
                                        graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                                    let reg_reinterpreted = chunk.add_register(
                                        format!("PVAL_EHANDLE:{entclass}"),
                                        chunk.get_last_instruction_id() + 1,
                                    );
                                    let instruction = instruction_templates::reinterpret_instance(
                                        reg_reinterpreted,
                                        inp,
                                    );
                                    chunk.add_instruction(instruction);
                                    reg_reinterpreted
                                } else {
                                    inp
                                };
                                inp
                            }
                            _ => inp,
                        };
                        register_map.add_inparam(param.name.clone().into(), inp);
                    }
                }
            }

            let mut reg_output = -1;
            let mut evaluation_required: bool = true; // set to true if we need to invoke something instead of re-using the register
            if let Some(outparams) = &binding.outparams {
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
                    func_name: binding.libname.clone().into(),
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
                    graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
                }
                LibraryBindingType::Value => return Ok(reg_output),
            }
        }
        PulseNodeTemplate::FindEntitiesWithin => {
            let classname =
                get_constant_graph_input_value!(graph, current_node, "classname", try_to_string);
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
                return Ok(reg_output);
            }
            // connection only
            let reg_searchfroment = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "pSearchFromEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            )?;
            let reg_radius = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "flSearchRadius",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
            // connection only
            let reg_startentity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "pStartEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            )?;
            let mut register_map = reg_map_setup_inputs!(
                "pSearchFromEntity",
                reg_searchfroment,
                "flSearchRadius",
                reg_radius,
                "pStartEntity",
                reg_startentity
            );
            register_map.add_outparam("retval".into(), reg_output);
            let cell = CPulseCell_Value_FindEntByClassNameWithin::new(classname.into());
            add_cell_and_invoking(
                graph_def,
                Box::from(cell),
                register_map,
                target_chunk,
                "Eval".into(),
            );
            return Ok(reg_output);
        }
        PulseNodeTemplate::CompareOutput => {
            let compare_type =
                get_constant_graph_input_value!(graph, current_node, "type", try_pulse_type);
            let reg_a = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "A",
                compare_type.clone(),
                false,
            )?;
            let reg_b = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "B",
                compare_type.clone(),
                false,
            )?;
            // value that we will match to generate a operation
            // this might get turned into a proper enum at some point
            let compare_value =
                get_constant_graph_input_value!(graph, current_node, "operation", try_to_string);
            let mut create_comp_instructions =
                |code: Cow<'_, str>, reg_a: Option<i32>, reg_b: Option<i32>| -> i32 {
                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    let mut instr_compare = Instruction {
                        code: String::from(code),
                        ..Default::default()
                    };

                    let reg_cond = chunk.add_register(
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
                "==" => create_comp_instructions(
                    format!("EQ{}", compare_type.get_operation_suffix_name()).into(),
                    reg_a,
                    reg_b,
                ),
                "!=" => create_comp_instructions(
                    format!("NE{}", compare_type.get_operation_suffix_name()).into(),
                    reg_a,
                    reg_b,
                ),
                "<" => create_comp_instructions(
                    format!("LT{}", compare_type.get_operation_suffix_name()).into(),
                    reg_a,
                    reg_b,
                ),
                "<=" => create_comp_instructions(
                    format!("LTE{}", compare_type.get_operation_suffix_name()).into(),
                    reg_a,
                    reg_b,
                ),
                // > is NOT <=
                ">" => {
                    let reg_lte = create_comp_instructions(
                        format!("LTE{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b,
                    );

                    create_comp_instructions("NOT".into(), Some(reg_lte), None)
                }
                // >= is NOT <
                ">=" => {
                    let reg_lt = create_comp_instructions(
                        format!("LT{}", compare_type.get_operation_suffix_name()).into(),
                        reg_a,
                        reg_b,
                    );

                    create_comp_instructions("NOT".into(), Some(reg_lt), None)
                }
                _ => {
                    anyhow::bail!(
                        "CompareOutput node: Unknown operation value: {}",
                        compare_value
                    );
                }
            };
            return Ok(reg_comp);
        }
        PulseNodeTemplate::IntSwitch => {
            let reg_value = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_INT(None),
                false,
            )?;
            let register_map = reg_map_setup_inputs!("nSwitchValue", reg_value);
            // now comes the processing of the outflows
            let mut outflow_connections = vec![];
            // the cell id will need to be adjusted later after we process all the cases and construct the cell itself.
            let cell_binding_id =
                add_cell_invoke_binding(graph_def, register_map, target_chunk, "Run".into(), -1);
            let mut instructions_jump_end = vec![];
            for out in current_node.outputs.iter() {
                if out.0.parse::<i32>().is_err() {
                    continue;
                }

                let next_actions = get_connected_action_nodes_and_inputs(graph, &out.1)?;
                if !next_actions.is_empty() {
                    let this_action_instruction_id = graph_def
                        .chunks
                        .get(target_chunk as usize)
                        .unwrap()
                        .get_last_instruction_id()
                        + 1;
                    // get connected input names TODO: reduce it to a function, this is repeating from a different func.
                    for conn in next_actions.iter() {
                        // ! this code is ass, literally nothing can go wrong here.
                        let node = graph.nodes.get(conn.0).unwrap();
                        let input_name = node
                            .inputs
                            .iter()
                            .find(|item| item.1 == conn.1)
                            .unwrap()
                            .0
                            .as_str();
                        traverse_nodes_and_populate(
                            graph,
                            node,
                            graph_def,
                            graph_state,
                            target_chunk,
                            &None,
                            &Some(input_name.into()),
                        )?;
                    }
                    // add a JUMP instruction to the end of all of the cases
                    // we don't really know where that will be so we will have to note down the instruction id and modify it later.
                    let instr_jump = instruction_templates::jump(-1);
                    instructions_jump_end.push(
                        graph_def
                            .chunks
                            .get_mut(target_chunk as usize)
                            .unwrap()
                            .add_instruction(instr_jump),
                    );

                    outflow_connections.push(OutflowConnection::new(
                        out.0.clone().into(),
                        target_chunk,
                        this_action_instruction_id,
                        Some(RegisterMap::default()),
                    ));
                }
            }
            // now let's process the default case
            let mut default_case_outflow = None;
            if graph_run_next_actions_no_return!(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "defaultcase"
            ) {
                let default_case_instruction_id = graph_def
                    .chunks
                    .get(target_chunk as usize)
                    .unwrap()
                    .get_last_instruction_id()
                    + 1;
                default_case_outflow = Some(OutflowConnection::new(
                    "default".into(),
                    target_chunk,
                    default_case_instruction_id,
                    Some(RegisterMap::default()),
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
                } else {
                    anyhow::bail!(
                        "IntSwitch node: Failed to find JUMP[end] with id: {}",
                        instr
                    );
                }
            }
            let cell = CPulseCell_Outflow_IntSwitch::new(
                default_case_outflow.unwrap_or_default(),
                outflow_connections,
            );
            graph_def.add_cell(Box::from(cell));
            // correct the cell invoke binding to point to the new cell id
            graph_def
                .get_invoke_binding_mut(cell_binding_id)
                .unwrap()
                .cell_index = graph_def.get_last_cell_id() as i32;
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::SoundEventStart => {
            let mut reg_out = try_find_output_mapping(graph_def, output_id);
            if reg_out == -1 {
                let reg_soundevent = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "strSoundEventName",
                    PulseValueType::PVAL_SNDEVT_NAME(None),
                    false,
                )?;
                let reg_target_entity = get_input_register_or_create_constant(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    "hTargetEntity",
                    PulseValueType::PVAL_EHANDLE(None),
                    false,
                )?;
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                let instr = chunk.get_last_instruction_id() + 1;
                reg_out =
                    chunk.add_register(PulseValueType::PVAL_SNDEVT_GUID(None).to_string(), instr);
                if let Some(out) = output_id {
                    graph_def.add_register_mapping(*out, reg_out);
                }
                let mut register_map = reg_map_setup_inputs!(
                    "strSoundEventName",
                    reg_soundevent,
                    "hTargetEntity",
                    reg_target_entity
                );
                register_map.add_outparam("retval".into(), reg_out);
                let cell =
                    CPulseCell_SoundEventStart::new(SoundEventStartType::SOUNDEVENT_START_ENTITY);
                add_cell_and_invoking(
                    graph_def,
                    Box::new(cell),
                    register_map,
                    target_chunk,
                    "Run".into(),
                );
            }
            return Ok(reg_out);
        }
        PulseNodeTemplate::CallNode => {
            // CallNode is a special node that is used to call another node, which is defined by the template.
            let node_id =
                get_constant_graph_input_value!(graph, current_node, "nodeId", try_node_id);
            if let Some(node) = graph.nodes.get(node_id) {
                let call_instr_id = graph_def.get_chunk_last_instruction_id(target_chunk) + 1;
                let remote_chunk_or_cell =
                    traverse_function_entry(graph, node, graph_def, graph_state).unwrap();

                match node.user_data.template {
                    PulseNodeTemplate::Function => {
                        let instr = instruction_templates::call_sync(
                            add_call_reference(graph_def, target_chunk, call_instr_id),
                            remote_chunk_or_cell,
                            0,
                        );
                        let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                        chunk.add_instruction(instr);
                    }
                    PulseNodeTemplate::ListenForEntityOutput => {
                        let reg_entity = get_input_register_or_create_constant(
                            graph,
                            current_node,
                            graph_def,
                            graph_state,
                            target_chunk,
                            "hEntity",
                            PulseValueType::PVAL_EHANDLE(None),
                            false,
                        )?;
                        let register_map = reg_map_setup_inputs!("hEntity", reg_entity);
                        let binding_id = graph_def.get_current_binding_id() + 1;
                        let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                        let instr =
                            chunk.add_instruction(instruction_templates::cell_invoke(binding_id));
                        let binding = InvokeBinding {
                            register_map,
                            func_name: source_input_name.as_ref().unwrap().to_string().into(), // XD
                            cell_index: remote_chunk_or_cell,
                            src_chunk: target_chunk,
                            src_instruction: instr,
                        };
                        graph_def.add_invoke_binding(binding);
                    }
                    _ => {
                        println!(
                            "CallNode: Node template remote {:?} is not supported for CallNode.",
                            node.user_data.template
                        );
                    }
                }
            } else {
                println!("CallNode: Node not found in the graph.");
            }
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::ListenForEntityOutput => {
            // just get the saved register and return it. If we get here it's already cached.
            return Ok(try_find_output_mapping(graph_def, output_id));
        }
        PulseNodeTemplate::Timeline => {
            // Timeline is a special node that is used to run a sequence of actions in a specific order.
            // It has a list of actions that are run in order.\
            // TODO: support onfinished outflow in UI
            let outflow_onfinished = OutflowConnection::new("".into(), -1, -1, None);

            let mut timeline_cell = CPulseCell_Timeline::new(outflow_onfinished, true);
            let cell_id = graph_def.get_last_cell_id() + 1;
            let binding_id = add_cell_invoke_binding(
                graph_def,
                RegisterMap::default(),
                target_chunk,
                "Start".into(),
                cell_id as i32,
            );
            // traverse all connected actions, they will be in the same chunk separated by returns, as it seems to be the way that it's done officially.
            for i in 1..7 {
                let delay_input = current_node.get_input(format!("timeFromPrevious{i}").as_str());
                let delay_param = graph
                    .get_input(delay_input.unwrap())
                    .value()
                    .clone()
                    .try_to_scalar()
                    .unwrap();
                let instr_id = graph_def.get_chunk_last_instruction_id(target_chunk) + 1;
                if graph_run_next_actions_no_return!(
                    graph,
                    current_node,
                    graph_def,
                    graph_state,
                    target_chunk,
                    format!("outAction{i}").as_str()
                ) {
                    graph_def
                        .chunks
                        .get_mut(target_chunk as usize)
                        .unwrap()
                        .add_instruction(instruction_templates::return_void());

                    let outflow = OutflowConnection::new(
                        format!("event_{}", i - 1).into(),
                        target_chunk,
                        instr_id,
                        None,
                    );
                    timeline_cell.add_event(delay_param, 0.0, true, outflow);
                }
            }
            let cell_id = graph_def.get_last_cell_id() + 1;
            graph_def.add_cell(Box::new(timeline_cell));
            // fixup the cell invoke binding
            let binding = graph_def.get_invoke_binding_mut(binding_id);
            if let Some(binding) = binding {
                binding.cell_index = cell_id as i32;
            } else {
                anyhow::bail!(
                    "Timeline node: Failed to find invoke binding with id: {}",
                    binding_id
                );
            }
        }
        PulseNodeTemplate::SetAnimGraphParam => {
            let reg_param_name = get_constant_graph_input_value!(graph, current_node, "paramName", try_to_string);
            let reg_entity = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "hEntity",
                PulseValueType::PVAL_EHANDLE(None),
                false,
            )?;
            let reg_param_value = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "pParamValue",
                PulseValueType::PVAL_ANY,
                false,
            )?;

            let cell = CPulseCell_Step_SetAnimGraphParam::new(
                reg_param_name.into(),
            );
            let reg_map = reg_map_setup_inputs!(
                "hEntity",
                reg_entity,
                "pParamValue",
                reg_param_value
            );
            add_cell_and_invoking(graph_def, Box::new(cell), reg_map, target_chunk, "Run".into());
            graph_next_action!(graph, current_node, graph_def, graph_state, target_chunk);
        }
        PulseNodeTemplate::ConstantBool => {
            let reg_out = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_BOOL,
                false,
            )?;
            return Ok(reg_out.unwrap_or(-1));
        }
        PulseNodeTemplate::ConstantFloat => {
            let reg_out = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_FLOAT(None),
                false,
            )?;
            return Ok(reg_out.unwrap_or(-1));
        }
        PulseNodeTemplate::ConstantInt => {
            let reg_out = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_INT(None),
                false,
            )?;
            return Ok(reg_out.unwrap_or(-1));
        }
        PulseNodeTemplate::ConstantVec3 => {
            let reg_out = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_VEC3(None),
                false,
            )?;
            return Ok(reg_out.unwrap_or(-1));
        }
        PulseNodeTemplate::ConstantString => {
            let reg_out = get_input_register_or_create_constant(
                graph,
                current_node,
                graph_def,
                graph_state,
                target_chunk,
                "value",
                PulseValueType::PVAL_STRING(None),
                false,
            )?;
            return Ok(reg_out.unwrap_or(-1));
        }
        _ => todo!(
            "Implement node template: {:?}",
            current_node.user_data.template
        ),
    }
    Ok(-1)
}
