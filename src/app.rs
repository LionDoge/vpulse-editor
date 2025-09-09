mod impls;
// contains help text
mod help;
pub mod types;
mod migrations;

use std::{path::PathBuf, fs, thread};
use core::panic;
use eframe::egui::Button;
use eframe::egui::TextStyle;
use eframe::egui::Vec2;
use serde::{Deserialize, Serialize};
use rfd::{FileDialog, MessageDialog};
use anyhow::anyhow;
use eframe::egui::{self, ComboBox, Modal, Id, RichText};
use egui_node_graph2::*;
use crate::bindings::*;
use crate::compiler::compile_graph;
use crate::pulsetypes::*;
use crate::typing::*;
use crate::utils::get_node_ids_connected_to_output;
use types::*;

static APP_NAME: &str = "Pulse Graph Editor";
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum ModalWindowType {
    #[default]
    None,
    ConfirmSave
}

#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct ModalWindow {
    pub window_type: ModalWindowType,
    pub is_open: bool,
}

#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseGraphEditor {
    #[cfg_attr(feature = "persistence", serde(default))]
    version: FileVersion,
    state: MyEditorState,
    user_state: PulseGraphState,
    #[cfg(feature = "nongame_asset_build")]
    #[serde(skip)]
    editor_config: EditorConfig,
    #[serde(skip)]
    current_modal_dialog: ModalWindow,
}

impl PulseGraphEditor {
    fn save_graph(&self, filepath: &PathBuf) -> Result<(), anyhow::Error> {
        let res = ron::ser::to_string_pretty::<PulseGraphEditor>(
            self,
            ron::ser::PrettyConfig::default(),
        )?;
        fs::write(filepath, res)?;
        Ok(())
    }
    // perform a save including including some cleanup
    fn perform_save(&mut self, filepath: Option<&PathBuf>) -> anyhow::Result<()> {
        let dest_path;
        // remove the path on manual save (we don't use serde skip because we want to save it within autosaves)
        let save_path = self.user_state.save_file_path.take();
        if let Some(filepath) = filepath {
            dest_path = filepath;
        } else {
            // if no filepath is provided, assume the one in saved state
            if let Some(filepath) = save_path.as_ref() {
                dest_path = filepath;
            } else {
                return Err(anyhow!(
                    "No file path provided for saving the graph. This should not happen"
                ));
            }
        }
        self.save_graph(dest_path)?;
        // restore the path info to memory.
        self.user_state.save_file_path = save_path;
        Ok(())
    }
    // promts user to choose a file to save the graph to and remembers the location for saving.
    fn dialog_change_save_file(&mut self) -> bool {
        let chosen_file = FileDialog::new()
            .add_filter("Pulse Graph Editor State", &["ron"])
            .save_file();
        let did_pick = chosen_file.as_ref().is_some(); // if not, the user cancelled so we should note that
        if did_pick {
            self.user_state.save_file_path = chosen_file;
        }
        did_pick
    }
    // Applies some corrections if some data is missing or changed for files saved in older versions
    pub fn verify_compat(&mut self) {
        // v0.1.1 introduces a SecondaryMap node_sizes in GraphEditorState
        // make sure that it is populated with every existing node.
        if self.state.node_sizes.is_empty() {
            for node in self.state.graph.nodes.iter() {
                self.state.node_sizes.insert(node.0, egui::vec2(200.0, 200.0));
            }
        }
        let mut sound_event_nodes = vec![];
        let mut entfire_nodes = vec![];
        let mut call_func_nodes = vec![];
        for node_id in self.state.graph.iter_nodes().collect::<Vec<_>>() {
            let node = match self.state.graph.nodes.get_mut(node_id) {
                Some(node) => node,
                None => continue,
            };
            let template = node.user_data.template;
            match template {
                // verify that all existing library binding nodes have correct parameter names, in case they have been updated between sessions.
                PulseNodeTemplate::LibraryBindingAssigned { binding } => {
                    if let Some(binding) = self.user_state.get_library_binding_from_index(&binding) {
                        if binding.inparams.is_none() {
                            continue;
                        }
                        let nodes = node.inputs.iter_mut().filter(|input| {
                            let nam_lowercase = input.0.to_lowercase();
                            !nam_lowercase.contains("action") && !nam_lowercase.contains("binding")
                        });
                        for (input, param) in nodes.zip(binding.inparams.as_ref().unwrap().iter()) {
                            input.0 = param.name.clone();
                        }
                    }
                }
                // v0.3.1 we added sound event source input.
                PulseNodeTemplate::SoundEventStart => {
                    // if the input is not present, add it to a list, and then add the input later
                    // can't do it here because of borrow checker
                    if node.get_input("soundEventType").is_err() {
                        sound_event_nodes.push(node_id);
                    }
                }
                // v0.3.1 Added entity handle input to EntFire
                PulseNodeTemplate::EntFire => {
                    if node.get_input("entityHandle").is_err() {
                        entfire_nodes.push(node_id);
                    }
                }
                // v0.3.1 Added Async fire mode to Call Node for functions
                PulseNodeTemplate::CallNode => {
                    if node.get_input("Async").is_err() {
                        let target_node_id = node
                            .get_input("nodeId")
                            .ok()
                            .and_then(|input_id| {
                                self.state.graph.get_input(input_id).value().clone().try_node_id().ok()
                            });

                        if let Some(target_node_id) = target_node_id {
                            if let Some(target_node) = self.state.graph.nodes.get(target_node_id) {
                                if target_node.user_data.template == PulseNodeTemplate::Function {
                                    call_func_nodes.push(node_id);
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        for node_id in sound_event_nodes {
            self.state.graph.add_input_param(
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
            self.state.graph.add_input_param(
                node_id,
                "ActionIn".to_string(),
                PulseDataType::Action,
                PulseGraphValueType::Action,
                InputParamKind::ConnectionOnly,
                true,
            );
            self.state.graph.add_output_param(node_id, "outAction".to_string(), PulseDataType::Action);
            // all of this below is just to move the input action to the top, since the library doesn't really make that easy.
            let node = self.state.graph.nodes.get_mut(node_id).unwrap();
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
            self.state.graph.add_input_param(
                node_id,
                "entityHandle".to_string(),
                PulseDataType::EHandle,
                PulseGraphValueType::EHandle,
                InputParamKind::ConnectionOnly,
                true,
            );
        }
        for node_id in call_func_nodes {
            self.state.graph.add_input_param(
                node_id,
                "Async".to_string(),
                PulseDataType::Bool,
                PulseGraphValueType::Bool { value: false },
                InputParamKind::ConstantOnly,
                true,
            );
        }
        // this fills out the default domain and subdomain if they're not set at launch time
        if self.user_state.graph_domain.is_empty() {
            self.user_state.graph_domain = "ServerEntity".to_string();
        }
        if self.user_state.graph_subtype.is_empty() {
            self.user_state.graph_subtype = "PVAL_EHANDLE:point_pulse".to_string();
        }
    }
    fn load_graph(&mut self, filepath: PathBuf) -> Result<(), anyhow::Error> {
        let contents = fs::read_to_string(&filepath)?;
        let loaded_graph: PulseGraphEditor = ron::from_str(&contents).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse file: {}",
                e.to_string()
            )
        })?;
        self.state = loaded_graph.state;
        self.user_state.load_from(loaded_graph.user_state);
        // we don't serialize file path since the file could be moved between save/open.
        self.user_state.save_file_path = Some(filepath);
        self.verify_compat();
        Ok(())
    }
    fn new_graph(&mut self, ctx: &egui::Context) {
        self.state = MyEditorState::default();
        self.user_state.load_from(PulseGraphState::default());
        self.user_state.save_file_path = None;
        self.update_titlebar(ctx);
    }
    pub fn update_output_node_param(&mut self, node_id: NodeId, name: &String, input_name: &str) {
        let param = self
            .state
            .graph
            .nodes
            .get_mut(node_id)
            .unwrap()
            .get_input(input_name);
        if let Ok(param) = param {
            self.state.graph.remove_input_param(param);
        }
        for output in self.user_state.public_outputs.iter() {
            if output.name == *name {
                match output.typ {
                    PulseValueType::PVAL_FLOAT(_) | PulseValueType::PVAL_INT(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from(input_name),
                            PulseDataType::Scalar,
                            PulseGraphValueType::Scalar { value: 0f32 },
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_STRING(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from(input_name),
                            PulseDataType::String,
                            PulseGraphValueType::String {
                                value: String::default(),
                            },
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_VEC3(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from(input_name),
                            PulseDataType::Vec3,
                            PulseGraphValueType::Vec3 {
                                value: Vec3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                            },
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_EHANDLE(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from(input_name),
                            PulseDataType::EHandle,
                            PulseGraphValueType::EHandle,
                            InputParamKind::ConnectionOnly,
                            true,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    fn add_node_input_simple(
        &mut self,
        node_id: NodeId,
        data_typ: PulseDataType,
        value_typ: PulseGraphValueType,
        input_name: &str,
        kind: InputParamKind,
    ) {
        self.state.graph.add_input_param(
            node_id,
            String::from(input_name),
            data_typ,
            value_typ,
            kind,
            true,
        );
    }
    fn add_node_output_simple(
        &mut self,
        node_id: NodeId,
        data_typ: PulseDataType,
        output_name: &str,
    ) {
        self.state
            .graph
            .add_output_param(node_id, String::from(output_name), data_typ);
    }
    pub fn update_node_inputs_outputs_types(
        &mut self,
        node_id: NodeId,
        name: &String,
        new_type: Option<PulseValueType>,
    ) {
        let node = self.state.graph.nodes.get_mut(node_id).unwrap();
        match node.user_data.template {
            PulseNodeTemplate::GetVar => {
                let param = node.get_output("value");
                if let Ok(param) = param {
                    self.state.graph.remove_output_param(param);
                }
                let var = self
                    .user_state
                    .variables
                    .iter()
                    .find(|var| var.name == *name);
                if let Some(var) = var {
                    self.add_node_output_simple(node_id, var.data_type.clone(), "value");
                }
            }
            PulseNodeTemplate::SetVar => {
                let param = node.get_input("value");
                if let Ok(param) = param {
                    self.state.graph.remove_input_param(param);
                }
                let var = self
                    .user_state
                    .variables
                    .iter()
                    .find(|var| var.name == *name);
                if let Some(var) = var {
                    let val_typ = data_type_to_value_type(&var.data_type);
                    self.add_node_input_simple(
                        node_id,
                        var.data_type.clone(),
                        val_typ,
                        "value",
                        InputParamKind::ConnectionOrConstant,
                    );
                }
            }
            PulseNodeTemplate::Operation => {
                if new_type.is_none() {
                    panic!("update_node_inputs_outputs() ended up on node that requires new value type from response, but it was not provided");
                }
                let new_type = new_type.unwrap();
                let param_a = node.get_input("A");
                let param_b = node.get_input("B");
                let param_out = node.get_output("out");
                if param_a.is_err() || param_b.is_err() || param_out.is_err() {
                    panic!("node that requires inputs 'A', 'B' and output 'out', but one of them was not found");
                }
                self.state.graph.remove_input_param(param_a.unwrap());
                self.state.graph.remove_input_param(param_b.unwrap());
                self.state.graph.remove_output_param(param_out.unwrap());

                let types = pulse_value_type_to_node_types(&new_type);
                self.add_node_input_simple(
                    node_id,
                    types.0.clone(),
                    types.1.clone(),
                    "A",
                    InputParamKind::ConnectionOrConstant,
                );
                self.add_node_input_simple(
                    node_id,
                    types.0.clone(),
                    types.1,
                    "B",
                    InputParamKind::ConnectionOrConstant,
                );
                self.add_node_output_simple(node_id, types.0, "out");
            }
            PulseNodeTemplate::Convert => {
                if name == "typefrom" {
                    let param_input = node.get_input("input");
                    if let Ok(param_input) = param_input {
                        self.state.graph.remove_input_param(param_input);
                        let types = pulse_value_type_to_node_types(&new_type.unwrap());
                        self.add_node_input_simple(
                            node_id,
                            types.0,
                            types.1,
                            "input",
                            InputParamKind::ConnectionOrConstant,
                        );
                    }
                } else if name == "typeto" {
                    let param_output = node.get_output("out");
                    if let Ok(param_output) = param_output {
                        self.state.graph.remove_output_param(param_output);
                        let types = pulse_value_type_to_node_types(&new_type.unwrap());
                        self.add_node_output_simple(node_id, types.0, "out");
                    }
                }
            }
            PulseNodeTemplate::Compare | PulseNodeTemplate::CompareOutput => {
                if new_type.is_none() {
                    panic!("update_node_inputs_outputs() ended up on node that requires new value type from response, but it was not provided");
                }
                let new_type = new_type.unwrap();
                let param_a = node.get_input("A");
                let param_b = node.get_input("B");
                if param_a.is_err() || param_b.is_err() {
                    panic!("node that requires inputs 'A' and 'B', but one of them was not found");
                }
                self.state.graph.remove_input_param(param_a.unwrap());
                self.state.graph.remove_input_param(param_b.unwrap());

                let types = pulse_value_type_to_node_types(&new_type);
                self.add_node_input_simple(
                    node_id,
                    types.0.clone(),
                    types.1.clone(),
                    "A",
                    InputParamKind::ConnectionOrConstant,
                );
                self.add_node_input_simple(
                    node_id,
                    types.0.clone(),
                    types.1,
                    "B",
                    InputParamKind::ConnectionOrConstant,
                );
            }
            PulseNodeTemplate::GetArrayElement => {
                if new_type.is_none() {
                    panic!("update_node_inputs_outputs() ended up on node that requires new value type from response, but it was not provided");
                }
                let new_type = new_type.unwrap();
                let param_a = node.get_input("expectedType");
                if param_a.is_err() {
                    panic!("node that requires input 'expectedType', but it was not found");
                }
                let param_output = node.get_output("out");
                if let Ok(param_output) = param_output {
                    self.state.graph.remove_output_param(param_output);
                    let types = pulse_value_type_to_node_types(&new_type);
                    self.add_node_output_simple(node_id, types.0, "out");
                }
            }
            PulseNodeTemplate::ScaleVector => {
                if new_type.is_none() {
                    panic!("update_node_inputs_outputs() ended up on node that requires new value type from response, but it was not provided");
                }
                let new_type = new_type.unwrap();
                let types = pulse_value_type_to_node_types(&new_type);
                let output_id = node.get_output("out");
                let param_vec = node.get_input("vector");
                if output_id.is_err() {
                    panic!("node requires output 'out', but it was not found");
                }
                let output = self.state.graph.get_output_mut(output_id.unwrap());
                output.typ = types.0.clone();
                if param_vec.is_err(){
                    panic!("node that requires inputs 'A' and 'B', but one of them was not found");
                }
                self.state.graph.remove_input_param(param_vec.unwrap());

                self.add_node_input_simple(
                    node_id,
                    types.0,
                    types.1,
                    "vector",
                    InputParamKind::ConnectionOrConstant,
                );
            }
            _ => {}
        }
    }

    fn update_library_binding_params(&mut self, node_id: &NodeId, binding: &FunctionBinding) {
        let output_ids: Vec<_> = {
            let node = self.state.graph.nodes.get_mut(*node_id).unwrap();
            node.output_ids().collect()
        };
        for output in output_ids {
            self.state.graph.remove_output_param(output);
        }
        let input_ids: Vec<_> = {
            let node = self.state.graph.nodes.get_mut(*node_id).unwrap();
            node.input_ids().collect()
        };
        let node = self.state.graph.nodes.get(*node_id).unwrap();
        let binding_chooser_input_id = node
            .get_input("binding")
            .expect("Expected 'Invoke library binding' node to have 'binding' input param");
        for input in input_ids {
            if input != binding_chooser_input_id {
                self.state.graph.remove_input_param(input);
            }
        }
        // If it's action type (nodes that usually don't provide a value) make it have in and out actions.
        if binding.typ == LibraryBindingType::Action {
            self.state.graph.add_output_param(
                *node_id,
                "outAction".to_string(),
                PulseDataType::Action,
            );
            self.state.graph.add_input_param(
                *node_id,
                "ActionIn".to_string(),
                PulseDataType::Action,
                PulseGraphValueType::Action,
                InputParamKind::ConnectionOrConstant,
                true,
            );
        }
        if let Some(inparams) = &binding.inparams {
            for param in inparams {
                let connection_kind = get_preffered_inputparamkind_from_type(&param.pulsetype);
                let graph_types = pulse_value_type_to_node_types(&param.pulsetype);
                self.state.graph.add_input_param(
                    *node_id,
                    param.name.clone(),
                    graph_types.0,
                    graph_types.1,
                    connection_kind,
                    true,
                );
            }
        }
        if let Some(outparams) = &binding.outparams {
            for param in outparams {
                self.state.graph.add_output_param(
                    *node_id,
                    param.name.clone(),
                    pulse_value_type_to_node_types(&param.pulsetype).0,
                );
            }
        }
    }

    fn update_event_binding_params(&mut self, node_id: &NodeId, binding: &EventBinding) {
        let output_ids: Vec<_> = {
            let node = self.state.graph.nodes.get_mut(*node_id).unwrap();
            node.output_ids().collect()
        };
        for output in output_ids {
            self.state.graph.remove_output_param(output);
        }
        // TODO: maybe instead of adding this back instead check in the upper loop, altho is seems a bit involved
        // so maybe this is just more efficient?
        self.state
            .graph
            .add_output_param(*node_id, "outAction".to_string(), PulseDataType::Action);
        if let Some(inparams) = &binding.inparams {
            for param in inparams {
                self.state.graph.add_output_param(
                    *node_id,
                    param.name.clone(),
                    pulse_value_type_to_node_types(&param.pulsetype).0,
                );
            }
        }
    }

    // Update inputs on "Call Node" depending on the type of referenced node.
    fn update_remote_node_params(&mut self, node_id: &NodeId, node_id_refrence: &NodeId) {
        let node = self.state.graph.nodes.get_mut(*node_id).unwrap();
        // remove all inputs
        let input_ids: Vec<_> = node.input_ids().collect();
        let input_node_chooser = node
            .get_input("nodeId")
            .expect("Expected 'Call Node' node to have 'nodeId' input param");
        for input in input_ids {
            // don't remove the node chooser input
            if input != input_node_chooser {
                self.state.graph.remove_input_param(input);
            }
        }
        if let Some(reference_node) = self.state.graph.nodes.get(*node_id_refrence) {
            let reference_node_template = reference_node.user_data.template;
            match reference_node_template {
                PulseNodeTemplate::ListenForEntityOutput => {
                    self.state.graph.add_input_param(
                        *node_id,
                        "hEntity".into(),
                        PulseDataType::EHandle,
                        PulseGraphValueType::EHandle,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    self.state.graph.add_input_param(
                        *node_id,
                        "Run".into(),
                        PulseDataType::Action,
                        PulseGraphValueType::Action,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    self.state.graph.add_input_param(
                        *node_id,
                        "Cancel".into(),
                        PulseDataType::Action,
                        PulseGraphValueType::Action,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                }
                PulseNodeTemplate::Function => {
                    self.state.graph.add_input_param(
                        *node_id,
                        "ActionIn".into(),
                        PulseDataType::Action,
                        PulseGraphValueType::Action,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    self.state.graph.add_input_param(*node_id,
                        "Async".into(),
                        PulseDataType::Bool,
                        PulseGraphValueType::Bool { value: Default::default() },
                        InputParamKind::ConstantOnly,
                        true,
                    );
                }
                PulseNodeTemplate::Timeline => {
                    self.state.graph.add_input_param(
                        *node_id,
                        "Start".into(),
                        PulseDataType::Action,
                        PulseGraphValueType::Action,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    self.state.graph.add_input_param(
                        *node_id,
                        "Stop".into(),
                        PulseDataType::Action,
                        PulseGraphValueType::Action,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                }
                _ => {
                    panic!(
                        "update_remote_node_params() called on unsupported node type: {:?}",
                        reference_node_template
                    );
                }
            }
        } else {
            println!("update_remote_node_params() called on node that does not exist in the graph anymore!");
        }
    }
    async fn check_for_updates() -> anyhow::Result<()> {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("liondoge")
            .repo_name("vpulse-editor")
            .build()?
            .fetch()?;
        let rel = releases.first().ok_or(anyhow::anyhow!(
            "No releases present after fetching from GitHub"
        ))?;
        let mut msg_box = rfd::AsyncMessageDialog::new()
            .set_level(rfd::MessageLevel::Info);
        if self_update::version::bump_is_greater(env!("CARGO_PKG_VERSION"), &rel.version)? {
            msg_box = msg_box
                .set_title("Update Available")
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_description(format!(
                    "A new version of Pulse Graph Editor is available: {}.\nDo you want to update?",
                    rel.version
                ));
        } else {
            msg_box = msg_box
                .set_title("Up to date")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description("Pulse Graph Editor is up to date.");
        }
        let response = msg_box.show().await;
        if response == rfd::MessageDialogResult::Yes {
            open::that("https://github.com/LionDoge/vpulse-editor/releases/latest")?;
        }
        Ok(())
    }
    // traverse forward to nodes connected to THIS node's output recursively
    // until we reach a node that doesn't depend on polymorphic return type.
    fn update_polymorphic_output_types(&mut self, node_id: NodeId, source_type: Option<PulseValueType>, source_input_name: Option<&str>) -> anyhow::Result<()> {
        // if the node is a "Make Array" node, we need to update the output type based on the array type
        let node_data = self.state.graph.nodes.get(node_id)
            .ok_or(anyhow::anyhow!("Node with id {:?} not found in the graph", node_id))?;
        if !has_polymorhpic_dependent_return(&node_data.user_data.template, &self.user_state) {
            return Ok(());
        };
        
        let opt_new_type = match node_data.user_data.template {
            PulseNodeTemplate::NewArray => {
                let graph = &self.state.graph;
                // TODO: get_constant_graph_input_value should probably be moved out of the compiler module
                let typ = crate::compiler::get_constant_graph_input_value!(
                    graph,
                    node_data,
                    "arrayType",
                    try_pulse_type
                );
                Some(PulseValueType::PVAL_ARRAY(Box::new(typ)))
            }
            PulseNodeTemplate::GetArrayElement
            | PulseNodeTemplate::ForEach => {
                // if the source type is not None, we can update the output type
                if let Some(source_type) = &source_type {
                    // if the source type is an array, we need to update the output type to the inner type
                    if let PulseValueType::PVAL_ARRAY(inner) = source_type {
                        let node_data_mut = self.state.graph.nodes.get_mut(node_id).unwrap(); // already checked, can unwrap.
                        let out_id = node_data_mut.get_output("out")?;
                        node_data_mut.user_data.custom_output_type = Some((**inner).clone());
                        // update output in UI
                        self.state.graph.get_output_mut(out_id).typ = pulse_value_type_to_node_types(inner).0;
                        Some((**inner).clone())
                    } else {
                        panic!("GetArrayElement/ForEach node expected source type to be an array, but it was not. This is a bug!");
                    }
                } else {
                    None
                }
            }
            PulseNodeTemplate::LibraryBindingAssigned { binding } => {
                let binding = &self.user_state.bindings.gamefunctions[binding.0];
                let new_type = if let Some(typ) = source_type {
                    Some(typ)
                } else {
                   node_data.user_data.custom_output_type.clone()
                };
                if let Some(polymorphic_return) = &binding.polymorphic_return {
                    match &polymorphic_return {
                        // full type means that we copy over the return type from the binding
                        PolimorphicTypeInfo::FullType (param_name) => {
                            println!("source input name: {}", source_input_name.unwrap_or_default());
                            if source_input_name.is_some_and(|f| f == param_name) || source_input_name.is_none() {
                                let node_data_mut = self.state.graph.nodes.get_mut(node_id).unwrap();
                                node_data_mut.user_data.custom_output_type = new_type.clone();
                                new_type
                            } else {
                                None
                            }
                        }
                        PolimorphicTypeInfo::TypeParam (param_name) => {
                            // type param means that we use the type parameter from the binding
                            // Must be an array, otherwise the definition is wrong!
                            println!("source input name: {}", source_input_name.unwrap_or_default());
                            if source_input_name.is_some_and(|f| f == param_name) || source_input_name.is_none() {
                                new_type.map(|new_type| {
                                    if let PulseValueType::PVAL_ARRAY(inner) = new_type {
                                        let node_data_mut = self.state.graph.nodes.get_mut(node_id).unwrap();
                                        node_data_mut.user_data.custom_output_type = Some(*inner.clone());
                                        *inner
                                    } else {
                                        panic!("Polymorphic return type requested inner type from param, but that param was not Array type, which seems wrong!");
                                    }
                                })
                            } else {
                                None
                            }
                        }
                        PolimorphicTypeInfo::ToSubtype(_param_name) => {
                            // FUTURE: to_subtype requests a specific return subtype described in the provided parameter
                            // However it's only really used for EHandles, and if a method requests a specific subtype of EHandle
                            // then it will be upcasted anyways, so doing anything here is not really worth the effort.
                            if let Some(param) = binding.find_outparam_by_name("retval") {
                                Some(param.pulsetype.clone())
                            } else {
                                Some(PulseValueType::PVAL_ARRAY(Box::new(PulseValueType::PVAL_EHANDLE(None))))
                            }
                        }
                    }
                } else {
                    None
                }
            }
            PulseNodeTemplate::GetVar => {
                // compiler should handle the register generation fine without any info from here
                // we only just provide info to connected nodes from this one
                let name_id = node_data
                    .get_input("variableName")
                    .map_err(|e| anyhow!(e).context(": Update polymorphic types"))?;
                let var_name = self.state.graph
                    .get_input(name_id)
                    .value()
                    .clone()
                    .try_variable_name()
                    .map_err(|e| anyhow!(e).context(": Update polymorphic types"))?;
                let var = self
                    .user_state
                    .variables
                    .iter()
                    .find(|var| var.name == *var_name);
                var.map(|var| var.typ_and_default_value.clone())
            }
            _ => None
        };
        // now update the custom type in user data so the compiler can use it
        // this will only happen if the resulting type is not None
        if let Some(new_type) = &opt_new_type {
            let node_data_mut = self.state.graph.nodes.get_mut(node_id).unwrap();
            node_data_mut.user_data.custom_output_type = opt_new_type.clone();
            println!(
                "[UI] Updating polymorphic output type of node {:?} to {:?}",
                node_id, new_type
            );
            // LOL, this needs to be slightly improved to make it more generic.
            let outnodes = get_node_ids_connected_to_output(
                self.state.graph.nodes.get(node_id).unwrap(), &self.state.graph, "out")
                .unwrap_or_default();
            let outnodes2 = get_node_ids_connected_to_output(
                self.state.graph.nodes.get(node_id).unwrap(), &self.state.graph, "retval")
                .unwrap_or_default();
            let outnodes3 = get_node_ids_connected_to_output(
                self.state.graph.nodes.get(node_id).unwrap(), &self.state.graph, "value")
                .unwrap_or_default();
            for node_and_input_name in outnodes.iter().chain(outnodes2.iter()).chain(outnodes3.iter()) {
                // recursively update the output type of connected nodes
                self.update_polymorphic_output_types(
                    node_and_input_name.0,
                     opt_new_type.clone(),
                      Some(&node_and_input_name.1)
                )?;
            }
        }
        Ok(())
    }
    fn clone_node(&mut self, source_node_id: NodeId, pos_offset: egui::Vec2) -> NodeId {
        let source_node_data = self.state.graph.nodes.get(source_node_id).unwrap();
        let source_label = source_node_data.label.clone();
        let source_user_data = source_node_data.user_data.clone();
        let inputs = source_node_data.inputs.clone();
        let outputs = source_node_data.outputs.clone();
        let new_node = self.state.graph.add_node(
            source_label,
            source_user_data,
            |grph, node_id| {
                // clone inputs
                // no input names in InputParam directly, they're stored directly in the node as vec of tuples
                for (input_name, input_id) in inputs {
                    let input_param = grph.get_input(input_id);
                    grph.add_input_param(
                        node_id,
                        input_name,
                        input_param.typ.clone(),
                        input_param.value.clone(),
                        input_param.kind,
                        true,
                    );
                }
                // clone outputs
                for (output_name, output_id) in outputs {
                    let output_param = grph.get_output(output_id);
                    grph.add_output_param(
                        node_id,
                        output_name,
                        output_param.typ.clone(),
                    );
                }
            }
        );
        // unwraps should basically never fail, otherwise there would be bigger issues.
        let orig_pos = self.state.node_positions.get(source_node_id).unwrap();
        self.state.node_positions.insert(new_node,*orig_pos + pos_offset);
        self.state.node_sizes.insert(new_node, *self.state.node_sizes.get(source_node_id).unwrap());
        self.state.node_order.push(new_node);
        // make sure exposed node info gets cloned as well (like function node)
        if let Some(name) = self.user_state.exposed_nodes.get(source_node_id).cloned() {
            self.user_state.exposed_nodes.insert(new_node, format!("{name} clone"));
        }
        new_node
    }

    fn update_titlebar(&self, ctx: &egui::Context) {
        let file_name = if let Some(file_path) = &self.user_state.save_file_path {
            file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<UNSAVED>")
                .to_string()
        } else {
            "<UNSAVED>".to_string()
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            format!("{APP_NAME} - {}", file_name)
        ));
    }
}

impl PulseGraphEditor {
    /// If the persistence feature is enabled, Called once before the first frame.
    /// Load previous app state (if any).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        #[cfg(feature = "persistence")]
        let mut grph: PulseGraphEditor = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
            .unwrap_or_default();

        grph.update_titlebar(&cc.egui_ctx);
        #[cfg(feature = "nongame_asset_build")] {
            let cfg_res: anyhow::Result<EditorConfig> = {
                let cfg_str = std::fs::read_to_string("config.json");
                match cfg_str {
                    Ok(cfg_str) => serde_json::from_str(&cfg_str)
                        .map_err(|e| anyhow::anyhow!("Failed to parse config.json: {}", e)),
                    Err(e) => Err(anyhow::anyhow!("Failed to read config.json: {}", e)),
                }
            };
            if let Err(e) = &cfg_res {
                MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to load config file")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description(format!("Failed to load config.json, compiling will not work fully. Refer to the documentation on how to set up valid configuration.\n {e}"))
                .show();
            };
            grph.editor_config = cfg_res.unwrap_or_default();
        }

        let bindings = load_bindings(std::path::Path::new("bindings_cs2.json"));
        match bindings {
            Ok(bindings) => {
                grph.user_state.bindings = bindings;
            }
            Err(e) => {
                MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Failed to load bindings for CS2")
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_description(e.to_string())
                    .show();
            }
        };
        grph.verify_compat();
        grph
    }
}

// assigns proper default values based on the text buffer, and updates the graph node types (DataTypes)
// this happens when input buffer changes, or the selected type changes.
pub fn update_variable_data(var: &mut PulseVariable) {
    var.typ_and_default_value = match &var.typ_and_default_value {
        PulseValueType::PVAL_INT(_) => {
            var.data_type = PulseDataType::Scalar;
            var.default_value_buffer
                .parse::<i32>()
                .map(|x| PulseValueType::PVAL_INT(Some(x)))
                .unwrap_or(PulseValueType::PVAL_INT(None))
        }
        PulseValueType::PVAL_FLOAT(_) => {
            var.data_type = PulseDataType::Scalar;
            var.default_value_buffer
                .parse::<f32>()
                .map(|x| PulseValueType::PVAL_FLOAT(Some(x)))
                .unwrap_or(PulseValueType::PVAL_FLOAT(None))
        }
        PulseValueType::PVAL_STRING(_) => {
            var.data_type = PulseDataType::String;
            PulseValueType::PVAL_STRING(Some(var.default_value_buffer.clone()))
        }
        PulseValueType::PVAL_VEC2(_) => {
            var.data_type = PulseDataType::Vec2;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::PVAL_VEC3(_) => {
            var.data_type = PulseDataType::Vec3;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::PVAL_VEC3_LOCAL(_) => {
            var.data_type = PulseDataType::Vec3Local;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::PVAL_VEC4(_) => {
            var.data_type = PulseDataType::Vec4;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::PVAL_QANGLE(_) => {
            var.data_type = PulseDataType::QAngle;
            var.typ_and_default_value.to_owned()
        }
        // horrible stuff, this will likely be refactored.
        PulseValueType::PVAL_EHANDLE(_) => {
            var.data_type = PulseDataType::EHandle;
            PulseValueType::PVAL_EHANDLE(Some(var.default_value_buffer.clone()))
        }
        PulseValueType::PVAL_SNDEVT_GUID(_) => {
            var.data_type = PulseDataType::SndEventHandle;
            PulseValueType::PVAL_SNDEVT_GUID(None)
        }
        PulseValueType::PVAL_BOOL_VALUE(_) => {
            var.data_type = PulseDataType::Bool;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::PVAL_COLOR_RGB(_) => {
            var.data_type = PulseDataType::Color;
            var.typ_and_default_value.to_owned()
        }
        PulseValueType::DOMAIN_ENTITY_NAME => {
            var.data_type = PulseDataType::EntityName;
            var.typ_and_default_value.to_owned()
        }
        _ => {
            var.data_type = pulse_value_type_to_node_types(&var.typ_and_default_value).0;
            var.typ_and_default_value.to_owned()
        }
    };
}

#[cfg(feature = "persistence")]
const PERSISTENCE_KEY: &str = "egui_node_graph";

pub fn has_polymorhpic_dependent_return(
    template: &PulseNodeTemplate,
    user_state: &PulseGraphState,
) -> bool {
    match template {
        PulseNodeTemplate::GetArrayElement
        | PulseNodeTemplate::NewArray
        | PulseNodeTemplate::GetVar
        | PulseNodeTemplate::ForEach => true,
        PulseNodeTemplate::LibraryBindingAssigned { binding: idx } => {
            let binding = match user_state.get_library_binding_from_index(idx) {
                Some(binding) => binding,
                None => return false,
            };
            binding.polymorphic_return.is_some()
        }
        _ => false
    }
}

impl eframe::App for PulseGraphEditor {
    #[cfg(feature = "persistence")]
    /// If the persistence function is enabled,
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, PERSISTENCE_KEY, &self);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        ctx.style_mut(|s| s.interaction.selectable_labels = false);
        if self.current_modal_dialog.is_open {
            let modal = Modal::new(Id::new("MainModal")).show(ctx, |ui| {
                match self.current_modal_dialog.window_type {
                    ModalWindowType::ConfirmSave => {
                        ui.set_width(400.0);
                        
                        ui.label(RichText::new("Create new graph").size(24.0));
                        ui.label(RichText::new("Are you sure you want to create a new graph? Unsaved changes will be lost.").size(16.0));

                        egui::Sides::new().show(
                            ui,
                |_ui| {},
                |ui| {
                            let btn_no = ui.add_sized([120., 30.], Button::new(RichText::new("No").size(18.0)));
                            let btn_yes = ui.add_sized([120., 30.], Button::new(RichText::new("Yes").size(18.0)));
                            if btn_no.clicked() {
                                ui.close();
                            }
                            if btn_yes.clicked() {
                                self.new_graph(ctx);
                                ui.close();
                            }
                        });
                    }
                    ModalWindowType::None => {}
                }
            });
            if modal.should_close() {
                self.current_modal_dialog.is_open = false;
            }
        }
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui: &mut egui::Ui| {
                if ui.button("Compile").clicked()
                    || ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::R)) {
                    if let Err(e) =
                        compile_graph(&self.state.graph, &self.user_state, 
                            #[cfg(feature = "nongame_asset_build")]&self.editor_config)
                    {
                        MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Compile failed")
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_description(e.to_string())
                            .show();
                    }
                }
                // User pressed the "Save" button or
                if ui.button("Save").clicked()
                    || ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S))
                    
                {
                    // is path set? if yes then save, if not promt the user first
                    let mut perform_save: bool = true;
                    if self.user_state.save_file_path.is_none() {
                        perform_save = self.dialog_change_save_file();
                        self.update_titlebar(ctx);
                    }
                    if perform_save {
                        if let Err(e) = self.perform_save(None) {
                            MessageDialog::new()
                                .set_level(rfd::MessageLevel::Error)
                                .set_title("Save failed")
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description(e.to_string())
                                .show();
                        }
                    }
                    // else it was most likely cancelled.
                }
                if (ui.button("Save as...").clicked()
                    || ctx.input(|i| {
                        i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::S)
                    }))
                    && self.dialog_change_save_file()
                {
                    // TODO: DRY
                    if let Err(e) = self.perform_save(None) {
                        MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Save failed")
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_description(e.to_string())
                            .show();
                    }
                }
                if ui.button("Open").clicked() {
                    let chosen_file = FileDialog::new()
                        .add_filter("Pulse Graph Editor State", &["ron"])
                        .pick_file();
                    if let Some(filepath) = chosen_file {
                        if let Err(e) = self.load_graph(filepath) {
                            MessageDialog::new()
                                .set_level(rfd::MessageLevel::Error)
                                .set_title("Load failed")
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description(e.to_string())
                                .show();
                        } else {
                            //self.update_titlebar(ctx);
                        }
                    }
                }
                if ui.button("New").clicked()
                    && !self.state.graph.nodes.is_empty() {
                        self.current_modal_dialog.is_open = true;
                        self.current_modal_dialog.window_type = ModalWindowType::ConfirmSave;
                    }
                if !ctx.wants_keyboard_input() 
                    && ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::D)) {
                    let selected_nodes: Vec<_> = self.state.selected_nodes.to_vec();
                    let mut new_nodes: Vec<_> = vec![];
                    for node_id in selected_nodes {
                        new_nodes.push(
                            self.clone_node(node_id, Vec2::new(20.0, 20.0))
                        );
                    }
                    self.state.selected_nodes = new_nodes;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Check for updates").clicked() {
                        thread::spawn(move || {
                            if let Err(e) = smol::block_on(PulseGraphEditor::check_for_updates()) {
                                MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_title("Update check failed")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_description(e.to_string())
                                    .show();
                            }
                        });
                    }
                    ui.label(env!("CARGO_PKG_VERSION"));
                });
            });
        });
        let mut output_scheduled_for_deletion: usize = usize::MAX; // we can get away with just one reference (it's not like the user can click more than one at once)
        let mut variable_scheduled_for_deletion: usize = usize::MAX;
        let mut output_node_updates = vec![];
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            egui::CollapsingHeader::new("Advanced")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Graph domain").on_hover_text("Suggests which context the graph can be used in, and what features are available.");
                        ui.text_edit_singleline(&mut self.user_state.graph_domain);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Graph sub-type").on_hover_text("The type on which the graph will be ran on eg. point entity/model entity/panel.");
                        ui.text_edit_singleline(&mut self.user_state.graph_subtype);
                    });
                });
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Outputs:");
                if ui.button("Add output").clicked() {
                    self.user_state
                        .outputs_dropdown_choices
                        .push(PulseValueType::PVAL_INT(None));
                    self.user_state.public_outputs.push(OutputDefinition {
                        name: String::default(),
                        typ: PulseValueType::PVAL_INT(None),
                        typ_old: PulseValueType::PVAL_INT(None),
                    });
                }
                for (idx, outputdef) in self.user_state.public_outputs.iter_mut().enumerate() {
                    ui.add_space(4.0);
                    egui::Frame::default()
                        .inner_margin(8.0)
                        .fill(egui::Color32::from_rgba_unmultiplied(36, 36, 36, 255))
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("X").clicked() {
                                output_scheduled_for_deletion = idx;
                            }
                            ui.add(egui::TextEdit::singleline(&mut outputdef.name)
                                .font(TextStyle::Heading)
                                .hint_text("Output name")
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Param type");
                            ComboBox::from_id_salt(format!("output{idx}"))
                                .selected_text(outputdef.typ.get_ui_name())
                                .show_ui(ui, |ui| {
                                    for typ in PulseValueType::get_variable_supported_types() {
                                        let name = typ.get_ui_name();
                                        ui.selectable_value(&mut outputdef.typ,
                                            typ,
                                            name
                                        );
                                    }
                                });
                        });
                        if outputdef.typ != outputdef.typ_old {
                            let node_ids: Vec<_> = self.state.graph.iter_nodes().collect();
                            for nodeid in node_ids {
                                let node = self.state.graph.nodes.get(nodeid).unwrap();
                                if node.user_data.template == PulseNodeTemplate::FireOutput {
                                    let inp = node.get_input("outputName");
                                    let val = self
                                        .state
                                        .graph
                                        .get_input(inp.unwrap())
                                        .value()
                                        .clone()
                                        .try_output_name()
                                        .unwrap();
                                    if outputdef.name == val {
                                        output_node_updates.push((nodeid, outputdef.name.clone()));
                                    }
                                }
                            }
                            outputdef.typ_old = outputdef.typ.clone();
                        }
                    });
                }
                ui.separator();
                ui.label("Variables:");
                if ui.button("Add variable").clicked() {
                    self.user_state
                        .outputs_dropdown_choices
                        .push(PulseValueType::PVAL_INT(None));
                    self.user_state.variables.push(PulseVariable {
                        name: String::default(),
                        typ_and_default_value: PulseValueType::PVAL_INT(None),
                        data_type: PulseDataType::Scalar,
                        default_value_buffer: String::default(),
                    });
                }
                for (idx, var) in self.user_state.variables.iter_mut().enumerate() {
                    ui.add_space(4.0);
                    egui::Frame::default()
                        .inner_margin(8.0)
                        .fill(egui::Color32::from_rgba_unmultiplied(36, 36, 36, 255))
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("X").clicked() {
                                variable_scheduled_for_deletion = idx;
                            }
                            ui.add(egui::TextEdit::singleline(&mut var.name)
                                .font(TextStyle::Heading)
                                .hint_text("Variable name")
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Param type");
                            ComboBox::from_id_salt(format!("var{idx}"))
                                .selected_text(var.typ_and_default_value.get_ui_name())
                                .show_ui(ui, |ui| {
                                    for typ in PulseValueType::get_variable_supported_types() {
                                        let name = typ.get_ui_name();
                                        if ui.selectable_value(&mut var.typ_and_default_value,
                                            typ,
                                            name
                                        ).clicked() {
                                            // if the type is changed, update the variable data.
                                            update_variable_data(var);
                                        }
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            // change the label text if we're working on an EHandle type, as it can't have a default value.
                            // the internal value will be used and updated approperiately as the ehandle type instead of the default value.
                            match &var.typ_and_default_value {
                                PulseValueType::PVAL_EHANDLE(_) => ui.label("EHandle class"),
                                PulseValueType::PVAL_ARRAY(_) => ui.label("Array type"),
                                _ => ui.label("Default value"),
                            };

                            match &mut var.typ_and_default_value {
                                PulseValueType::PVAL_BOOL_VALUE(value) => {
                                    ui.checkbox(
                                        value.get_or_insert_default(), ""
                                    );
                                }
                                PulseValueType::PVAL_VEC2(value) => {
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().x).prefix("X: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().y).prefix("Y: "));
                                }
                                PulseValueType::PVAL_VEC3(value)
                                | PulseValueType::PVAL_VEC3_LOCAL(value)
                                | PulseValueType::PVAL_QANGLE(value) => {
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().x).prefix("X: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().y).prefix("Y: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().z).prefix("Z: "));
                                }
                                PulseValueType::PVAL_VEC4(value) => {
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().x).prefix("X: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().y).prefix("Y: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().z).prefix("Z: "));
                                    ui.add(egui::DragValue::new(&mut value.get_or_insert_default().w).prefix("W: "));
                                }
                                PulseValueType::PVAL_COLOR_RGB(value) => {
                                    let color = value.get_or_insert_default();
                                    // there's probably a better way, but our type system is a mess right now, I can't be bothered.
                                    let mut arr = [color.x / 255.0, color.y / 255.0, color.z / 255.0];
                                    if ui.color_edit_button_rgb(&mut arr).changed() {
                                        color.x = arr[0] * 255.0;
                                        color.y = arr[1] * 255.0;
                                        color.z = arr[2] * 255.0;
                                    }
                                }
                                PulseValueType::PVAL_RESOURCE(resource_type, value) => {
                                    let resource_type_val = resource_type.get_or_insert_with(Default::default);
                                    if ui.add(egui::TextEdit::singleline(resource_type_val)
                                        .hint_text("Type")
                                        .desired_width(40.0)).changed() 
                                        && resource_type_val.trim().is_empty() {
                                            *resource_type = None;
                                        }
                            
                                    ui.add(egui::TextEdit::singleline(value.get_or_insert_default()).hint_text("Resource path"));
                                }
                                PulseValueType::PVAL_GAMETIME(value) => {
                                    ui.add(egui::DragValue::new(value.get_or_insert_default()).speed(0.01));
                                }
                                PulseValueType::PVAL_ARRAY(inner_type) => {
                                    // TODO: make it recursive, so we can have more nested types.
                                    ComboBox::from_id_salt(format!("var{idx}_inner"))
                                        .selected_text(inner_type.get_ui_name())
                                        .show_ui(ui, |ui| {
                                            for typ in PulseValueType::get_variable_supported_types() {
                                                let name = typ.get_ui_name();
                                                ui.selectable_value(inner_type,
                                                    Box::from(typ),
                                                    name);
                                            }
                                        });
                                }
                                PulseValueType::DOMAIN_ENTITY_NAME 
                                | PulseValueType::PVAL_SNDEVT_GUID(_)
                                | PulseValueType::PVAL_TRANSFORM(_)
                                | PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => {}
                                _ => {
                                    if ui.text_edit_singleline(&mut var.default_value_buffer).changed() {
                                        update_variable_data(var);
                                    }
                                }
                            }
                                
                        });
                        });
                }
            });
        });
        if output_scheduled_for_deletion != usize::MAX {
            self.user_state
                .public_outputs
                .remove(output_scheduled_for_deletion);
        }
        if variable_scheduled_for_deletion != usize::MAX {
            self.user_state
                .variables
                .remove(variable_scheduled_for_deletion);
        }

        let mut prepended_responses: Vec<NodeResponse<PulseGraphResponse, PulseNodeData>> = vec![];
        if ctx.input(|i| i.key_released(egui::Key::Delete)) {
            // delete selected nodes
            for node_id in self.state.selected_nodes.iter() {
                prepended_responses.push(NodeResponse::DeleteNodeUi(*node_id));
            }
        }

        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(
                    ui,
                    AllMyNodeTemplates {
                        game_function_count: self.user_state.bindings.gamefunctions.len()
                    },
                    &mut self.user_state,
                    prepended_responses,
                )
            })
            .inner;

        for node_response in graph_response.node_responses {
            // handle all responses generated by the graph ui...
            match node_response {
                NodeResponse::User(user_event) => {
                    match user_event {
                        // node that supports adding parameters is trying to add one
                        PulseGraphResponse::AddOutputParam(node_id, name, datatype) => {
                            self.state.graph.add_output_param(
                                node_id,
                                name,
                                datatype,
                            );
                        }
                        PulseGraphResponse::RemoveOutputParam(node_id, name) => {
                            // node that supports adding parameters is removing one
                            let param = self
                                .state
                                .graph
                                .nodes
                                .get_mut(node_id)
                                .unwrap()
                                .get_output(&name)
                                .unwrap();
                            self.state.graph.remove_output_param(param);
                        }
                        PulseGraphResponse::ChangeOutputParamType(node_id, name) => {
                            self.update_output_node_param(node_id, &name, "param");
                        }
                        PulseGraphResponse::ChangeVariableParamType(node_id, name) => {
                            self.update_node_inputs_outputs_types(node_id, &name, None);
                        }
                        PulseGraphResponse::ChangeParamType(node_id, name, typ) => {
                            self.update_node_inputs_outputs_types(node_id, &name, Some(typ));
                        }
                        PulseGraphResponse::ChangeEventBinding(node_id, bindings) => {
                            //let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                            self.update_event_binding_params(&node_id, &bindings);
                        }
                        PulseGraphResponse::ChangeFunctionBinding(node_id, bindings) => {
                            //let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                            self.update_library_binding_params(&node_id, &bindings);
                        }
                        PulseGraphResponse::ChangeRemoteNodeId(node_id, node_id_refrence) => {
                            self.update_remote_node_params(&node_id, &node_id_refrence);
                        }
                        PulseGraphResponse::UpdatePolymorphicTypes(node_id) => {
                            if let Err(e) = self.update_polymorphic_output_types(node_id, None, None) {
                                println!("[UI] Warning: Failed to update polymorphic output types: {e}");
                            }
                        }
                    }
                }
                NodeResponse::DeleteNodeFull { node_id, .. } => {
                    self.user_state.exposed_nodes.remove(node_id);
                }
                NodeResponse::CreatedNode(node_id) => {
                    // This stuff is actually insane btw.
                    // if the node is a library binding, then update the parameters
                    if let PulseNodeTemplate::LibraryBindingAssigned { binding } 
                        = self.state.graph.nodes.get(node_id).unwrap().user_data.template {
                        let binding_index = binding;
                        let binding_opt = self.user_state.get_library_binding_from_index(&binding_index).cloned();
                        if let Some(binding) = binding_opt {
                            self.update_library_binding_params(&node_id, &binding);
                        }
                    }
                }
                NodeResponse::ConnectEventEnded { output, input: _ , input_hook: _} => {
                    let graph = &self.state.graph;
                    let node_id = graph.get_output(output).node;
                    if let Err(e) = self.update_polymorphic_output_types(node_id, None, None) {
                        println!("[UI] Warning: Failed to update polymorphic output types: {e}");
                    }
                }
                _ => {}
            }
        }
        for (nodeid, name) in output_node_updates {
            self.update_output_node_param(nodeid, &name, "param");
        }
    }
}
