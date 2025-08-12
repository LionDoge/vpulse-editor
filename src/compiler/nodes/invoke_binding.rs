use egui_node_graph2::*;
use slotmap::SecondaryMap;
use crate::app::types::{
    PulseGraph, PulseGraphState,PulseNodeData
};
use crate::typing::PulseValueType;
use crate::compiler::serialization::*;
use crate::compiler::*;

pub fn compile_node(
    graph: &PulseGraph,
    current_node: &Node<PulseNodeData>,
    graph_def: &mut PulseGraphDef,
    graph_state: &PulseGraphState,
    target_chunk: i32,
    output_id: &Option<OutputId>,
) -> anyhow::Result<Option<i32>> {
    // if we're requesting for a value then we can try to find the output mapping first
    if let Some(output_id) = output_id {
        let existing_reg = graph_def.get_mapped_register_node_outputs(current_node.id, *output_id);
        if let Some(reg) = existing_reg {
            return Ok(Some(*reg));
        }
    }

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

    let mut remembered_outputs: SecondaryMap<OutputId, i32> = SecondaryMap::default();
    if let Some(outparams) = &binding.outparams {
        for param in outparams.iter() {
            // looking for the outputid matching the parameter name
            // this should be setup on the UI side, if not, then it's a programming error
            let current_output_id = current_node.get_output(&param.name).map_err(|e| {
                anyhow::anyhow!(e).context("\n Failed in LibraryBinding node. Mismatch between UI and ParamInfo")
            })?;
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let ret_type = if binding.polymorphic_return.is_some() {
                // if the binding is polymorphic, we need to use the polymorphic type
                current_node.user_data
                    .custom_output_type
                    .as_ref()
                    .unwrap_or(&param.pulsetype)
            } else {
                &param.pulsetype
            };
            println!(
                "InvokeLibraryBinding - {}: Adding output parameter {} with type {}",
                binding.displayname, param.name, ret_type
            );
            let reg_out = chunk.add_register(
                ret_type.to_string(),
                chunk.get_last_instruction_id() + 1,
            );

            // rememer the output in case it's requested later
            remembered_outputs.insert(current_output_id, reg_out);
            register_map.add_outparam(param.name.clone().into(), reg_out);
        }
    }
    // set the current returned register to the output that was requested (if any)
    let reg_output = if let Some(output_id) = output_id {
        remembered_outputs.get(*output_id).copied().unwrap_or(-1)
    } else {
        println!("Warning: InvokeLibraryBinding node: Output requested, but not present as a parameter, using -1 as default register output");
        -1
    };
    graph_def.add_register_mapping_node_outputs(current_node.id, remembered_outputs);
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
    match binding.typ {
        LibraryBindingType::Action => Ok(None),
        LibraryBindingType::Value => Ok(Some(reg_output)),
    }
}