mod impls;
// contains help text
mod help;
pub mod types;
mod migrations;

use std::{path::PathBuf, fs, thread};
use core::panic;
use serde::{Deserialize, Serialize};
use rfd::{FileDialog, MessageDialog};
use anyhow::anyhow;
use eframe::egui::{self, ComboBox};
use egui_node_graph2::*;
use crate::bindings::*;
use crate::compiler::compile_graph;
use crate::pulsetypes::*;
use crate::typing::*;
use types::*;

// this includes default values inside enums if applicable.
fn get_supported_ui_types() -> Vec<PulseValueType> {
    vec![
        PulseValueType::PVAL_INT(None),
        PulseValueType::PVAL_FLOAT(None),
        PulseValueType::PVAL_STRING(None),
        PulseValueType::PVAL_BOOL_VALUE(None),
        PulseValueType::PVAL_VEC2(None),
        PulseValueType::PVAL_VEC3(None),
        PulseValueType::PVAL_VEC3_LOCAL(None),
        PulseValueType::PVAL_VEC4(None),
        PulseValueType::PVAL_QANGLE(None),
        PulseValueType::PVAL_TRANSFORM(None),
        PulseValueType::PVAL_TRANSFORM_WORLDSPACE(None),
        PulseValueType::PVAL_COLOR_RGB(None),
        PulseValueType::PVAL_EHANDLE(None),
        PulseValueType::DOMAIN_ENTITY_NAME,
        PulseValueType::PVAL_SNDEVT_GUID(None),
        PulseValueType::PVAL_ARRAY(None),
        PulseValueType::PVAL_RESOURCE(None, None),
        PulseValueType::PVAL_GAMETIME(None),
    ]
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
        // clear deprecated references
        // this is a bit inefficient but will do for now.
        for node in self.state.graph.nodes.iter() {
            for exposed_node in self.user_state.exposed_nodes.iter_mut() {
                if exposed_node.0 == node.0 {
                    // remove the node if it doesn't exist anymore.
                    if !self.state.graph.nodes.contains_key(node.0) {
                        exposed_node.1.clear();
                    }
                }
            }
        }
        let dest_path;
        if let Some(filepath) = filepath {
            dest_path = filepath;
        } else {
            // if no filepath is provided, assume the one in saved state
            if let Some(filepath) = &self.user_state.save_file_path {
                dest_path = filepath;
            } else {
                return Err(anyhow!(
                    "No file path provided for saving the graph. This should not happen"
                ));
            }
        }
        self.save_graph(dest_path)?;
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
    // loads MyGraphState, and applies some corrections if some data is missing for files saved in older versions
    fn load_state_with_backwards_compat(&mut self, new_state: MyEditorState) {
        self.state = new_state;
        // v0.1.1 introduces a SecondaryMap node_sizes in GraphEditorState
        // make sure that it is populated with every existing node.
        if self.state.node_sizes.is_empty() {
            for node in self.state.graph.nodes.iter() {
                self.state.node_sizes.insert(node.0, egui::vec2(200.0, 200.0));
            }
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
        self.load_state_with_backwards_compat(loaded_graph.state);
        self.user_state.load_from(loaded_graph.user_state);
        // we don't serialize file path since the file could be moved between save/open.
        self.user_state.save_file_path = Some(filepath);
        Ok(())
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
                if var.is_some() {
                    let var_unwrp = var.unwrap();
                    self.add_node_output_simple(node_id, var_unwrp.data_type.clone(), "value");
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
                if var.is_some() {
                    let var_unwrp = var.unwrap();
                    let val_typ = data_type_to_value_type(&var_unwrp.data_type);
                    self.add_node_input_simple(
                        node_id,
                        var_unwrp.data_type.clone(),
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
                    if param_input.is_ok() {
                        self.state.graph.remove_input_param(param_input.unwrap());
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
                    if param_output.is_ok() {
                        self.state.graph.remove_output_param(param_output.unwrap());
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
            var.data_type = PulseDataType::Scalar;
            var.typ_and_default_value.to_owned()
        }
    };
}

#[cfg(feature = "persistence")]
const PERSISTENCE_KEY: &str = "egui_node_graph";

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
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui: &mut egui::Ui| {
                if ui.button("Compile").clicked() {
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
                        }
                    }
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
                    // let output_frame = egui::Frame::default().inner_margin(4.0).begin(ui);
                    // {
                    ui.horizontal(|ui| {
                        if ui.button("X").clicked() {
                            output_scheduled_for_deletion = idx;
                        }
                        ui.label("Name");
                        ui.text_edit_singleline(&mut outputdef.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Param type");
                        ComboBox::from_id_salt(format!("output{idx}"))
                            .selected_text(outputdef.typ.get_ui_name())
                            .show_ui(ui, |ui| {
                                for typ in get_supported_ui_types() {
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
                    // }
                    // output_frame.end(ui);
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
                    ui.horizontal(|ui| {
                        if ui.button("X").clicked() {
                            variable_scheduled_for_deletion = idx;
                        }
                        ui.label("Name");
                        ui.text_edit_singleline(&mut var.name);
                    });
                    ui.horizontal(|ui| {
                        // change the label text if we're working on an EHandle type, as it can't have a default value.
                        // the internal value will be used and updated approperiately as the ehandle type instead of the default value.
                        if matches!(var.typ_and_default_value, PulseValueType::PVAL_EHANDLE(_)) {
                            ui.label("EHandle class");
                        } else {
                            ui.label("Default value");
                        }

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
                            PulseValueType::DOMAIN_ENTITY_NAME 
                            | PulseValueType::PVAL_SNDEVT_GUID(_)
                            | PulseValueType::PVAL_TRANSFORM(_)
                            | PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_)
                            | PulseValueType::PVAL_ARRAY(_) => {}
                            _ => {
                                if ui.text_edit_singleline(&mut var.default_value_buffer).changed() {
                                    update_variable_data(var);
                                }
                            }
                        }
                            
                    });
                    ui.horizontal(|ui| {
                        ui.label("Param type");
                        ComboBox::from_id_salt(format!("var{idx}"))
                            .selected_text(var.typ_and_default_value.get_ui_name())
                            .show_ui(ui, |ui| {
                                for typ in get_supported_ui_types() {
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
                    AllMyNodeTemplates,
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
                        PulseGraphResponse::AddOutputParam(node_id, name, data) => {
                            {
                                let node = self.state.graph.nodes.get(node_id).unwrap();
                                // check if the output of the name exists already...
                                let nam = node
                                    .user_data
                                    .custom_named_outputs
                                    .iter()
                                    .find(|v| v.1.name == name);
                                if nam.is_some() {
                                    continue;
                                }
                            }
                            let output_id = self.state.graph.add_output_param(
                                node_id,
                                name.clone(),
                                pulse_value_type_to_node_types(&data).0,
                            );
                            let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                            let output_info = CustomOutputInfo { name, data };
                            // remember the custom output
                            node.user_data
                                .custom_named_outputs
                                .insert(output_id, output_info);
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
                            let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                            // in practice it will only be one, in theory there could be a bunch of the same name...
                            let keys_to_remove: Vec<_> = node
                                .user_data
                                .custom_named_outputs
                                .iter()
                                .filter_map(|(k, v)| if v.name == name { Some(*k) } else { None })
                                .collect();
                            for k in keys_to_remove {
                                node.user_data.custom_named_outputs.remove(&k);
                            }
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
                    }
                }
                NodeResponse::DeleteNodeFull { node_id, .. } => {
                    self.user_state.exposed_nodes.remove(node_id);
                }
                _ => {}
            }
        }
        for (nodeid, name) in output_node_updates {
            self.update_output_node_param(nodeid, &name, "param");
        }
    }
}
