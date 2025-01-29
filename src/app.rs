use std::borrow::BorrowMut;
use serde_json::to_string_pretty;
use serde::{Serialize, Deserialize};
use std::usize;
use std::{borrow::Cow, collections::HashMap};
use eframe::egui::{self, ComboBox, DragValue};
use egui_file_dialog::FileDialog;
use egui_node_graph2::*;
use slotmap::SecondaryMap;
use crate::pulsetypes::*;
pub use crate::outputdefinition::*;
use crate::compiler::compile_graph;

// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyNodeData {
    pub template: MyNodeTemplate,
    pub custom_named_outputs: HashMap<OutputId, String>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum MyDataType {
    Scalar,
    Vec2,
    Vec3,
    String,
    Bool,
    Action,
    EHandle,
    InternalOutputName,
    InternalVariableName,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum MyValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    String { value: String },
    Bool { value: bool },
    Vec3 { value: Vec3 },
    EHandle,
    Action,
    InternalOutputName { prevvalue: String, value: String },
    InternalVariableName { prevvalue: String, value: String },
}

impl Default for MyValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Scalar { value: 0.0 }
    }
}

impl MyValueType {
    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let MyValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }

    pub fn try_to_string(self) -> anyhow::Result<String> {
        if let MyValueType::String { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to string", self)
        }
    }

    pub fn try_to_bool(self) -> anyhow::Result<bool> {
        if let MyValueType::Bool { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to bool", self)
        }
    }

    pub fn try_to_vec3(self) -> anyhow::Result<Vec3> {
        if let MyValueType::Vec3 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec3", self)
        }
    }

    pub fn try_output_name(self) -> anyhow::Result<String> {
        if let MyValueType::InternalOutputName { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to output name", self)
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum MyNodeTemplate {
    MakeScalar,
    AddScalar,
    SubtractScalar,
    MakeVector,
    AddVector,
    SubtractVector,
    VectorTimesScalar,
    CellPublicMethod,
    EntFire,
    Compare,
    ConcatString,
    CellWait,
    GetVar,
    SetVar,
    EventHandler,
    IntToString,
    Operation,
    FindEntByName,
    DebugWorldText,
    DebugLog,
    FireOutput,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MyResponse {
    AddOutputParam(NodeId, String),
    RemoveOutputParam(NodeId, String),
    ChangeOutputParamType(NodeId, String),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct MyGraphState {
    pub custom_input_string: String,
    pub added_parameters: SecondaryMap<NodeId, Vec<String>>,
    pub public_outputs: Vec<OutputDefinition>,
    pub variables: Vec<PulseVariable>
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<MyGraphState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
        match self {
            MyDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            MyDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            MyDataType::Vec3 => egui::Color32::from_rgb(238, 207, 109),
            MyDataType::String => egui::Color32::from_rgb(52, 171, 235),
            MyDataType::Action => egui::Color32::from_rgb(252, 3, 165),
            MyDataType::EHandle => egui::Color32::from_rgb(18, 227, 81),
            MyDataType::Bool => egui::Color32::from_rgb(54, 61, 194),
            MyDataType::InternalOutputName => egui::Color32::from_rgb(0, 0, 0),
            MyDataType::InternalVariableName => egui::Color32::from_rgb(0, 0, 0),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            MyDataType::Scalar => Cow::Borrowed("scalar"),
            MyDataType::Vec2 => Cow::Borrowed("2d vector"),
            MyDataType::Vec3 => Cow::Borrowed("3d vector"),
            MyDataType::String => Cow::Borrowed("string"),
            MyDataType::Bool => Cow::Borrowed("bool"),
            MyDataType::Action => Cow::Borrowed("action"),
            MyDataType::EHandle => Cow::Borrowed("EHandle"),
            MyDataType::InternalOutputName => Cow::Borrowed("Output name"),
            MyDataType::InternalVariableName => Cow::Borrowed("Variable name"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            MyNodeTemplate::MakeScalar => "New scalar",
            MyNodeTemplate::AddScalar => "Scalar add",
            MyNodeTemplate::SubtractScalar => "Scalar subtract",
            MyNodeTemplate::MakeVector => "New vector",
            MyNodeTemplate::AddVector => "Vector add",
            MyNodeTemplate::SubtractVector => "Vector subtract",
            MyNodeTemplate::VectorTimesScalar => "Vector times scalar",
            MyNodeTemplate::CellPublicMethod => "Public Method",
            MyNodeTemplate::EntFire => "EntFire",
            MyNodeTemplate::Compare => "Compare",
            MyNodeTemplate::ConcatString => "Concatenate strings",
            MyNodeTemplate::CellWait => "Wait",
            MyNodeTemplate::GetVar => "Load variable",
            MyNodeTemplate::SetVar => "Save variable",
            MyNodeTemplate::EventHandler => "Event Handler",
            MyNodeTemplate::IntToString => "Int to string",
            MyNodeTemplate::Operation => "Operation",
            MyNodeTemplate::FindEntByName => "Find entity by name",
            MyNodeTemplate::DebugWorldText => "Debug world text",
            MyNodeTemplate::DebugLog => "Debug log",
            MyNodeTemplate::FireOutput => "Fire output",
        })
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            MyNodeTemplate::MakeScalar
            | MyNodeTemplate::AddScalar
            | MyNodeTemplate::SubtractScalar => vec!["Scalar"],
            MyNodeTemplate::MakeVector
            | MyNodeTemplate::AddVector
            | MyNodeTemplate::SubtractVector => vec!["Vector"],
            MyNodeTemplate::VectorTimesScalar => vec!["Vector", "Scalar"],
            MyNodeTemplate::CellPublicMethod | MyNodeTemplate::EventHandler => vec!["Inflow"],
            MyNodeTemplate::EntFire
            | MyNodeTemplate::FindEntByName => vec!["Entities"],
            MyNodeTemplate::Compare => vec!["Logic"],
            MyNodeTemplate::Operation => vec!["Math"],
            MyNodeTemplate::ConcatString => vec!["String"],
            MyNodeTemplate::CellWait => vec!["Utility"],
            MyNodeTemplate::GetVar | MyNodeTemplate::SetVar => vec!["Variables"],
            MyNodeTemplate::IntToString => vec!["Conversion"],
            MyNodeTemplate::DebugWorldText
            | MyNodeTemplate::DebugLog => vec!["Debug"],
            MyNodeTemplate::FireOutput => vec!["Outflow"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData { template: *self, custom_named_outputs: HashMap::new() }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        // We define some closures here to avoid boilerplate. Note that this is
        // entirely optional.
        let input_string = |graph: &mut MyGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::String,
                MyValueType::String {value: String::default()},
                kind,
                true,
            );
        };
        let input_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Scalar,
                MyValueType::Scalar { value: 0.0 },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_bool = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Bool,
                MyValueType::Bool { value: false },
                InputParamKind::ConstantOnly,
                true,
            );
        };
        let input_ehandle = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::EHandle,
                MyValueType::EHandle,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Vec2,
                MyValueType::Vec2 {
                    value: egui::vec2(0.0, 0.0),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector3 = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Vec3,
                MyValueType::Vec3 {
                    value: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_action = |graph: &mut MyGraph| {
            graph.add_input_param(
                node_id,
                "ActionIn".to_string(),
                MyDataType::Action,
                MyValueType::Action,
                InputParamKind::ConnectionOnly,
                true,
            );
        };

        let output_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Scalar);
        };
        let output_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Vec2);
        };
        let output_string = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::String);
        };
        let output_action = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Action);
        };
        let output_ehandle = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::EHandle);
        };

        // input_action(graph);
        // output_action(graph);
        match self {
            MyNodeTemplate::AddScalar => {
                // The first input param doesn't use the closure so we can comment
                // it in more detail.
                graph.add_input_param(
                    node_id,
                    // This is the name of the parameter. Can be later used to
                    // retrieve the value. Parameter names should be unique.
                    "A".into(),
                    // The data type for this input. In this case, a scalar
                    MyDataType::Scalar,
                    // The value type for this input. We store zero as default
                    MyValueType::Scalar { value: 0.0 },
                    // The input parameter kind. This allows defining whether a
                    // parameter accepts input connections and/or an inline
                    // widget to set its value.
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::SubtractScalar => {
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::VectorTimesScalar => {
                input_scalar(graph, "scalar");
                input_vector(graph, "vector");
                output_vector(graph, "out");
            }
            MyNodeTemplate::AddVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            MyNodeTemplate::SubtractVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            MyNodeTemplate::MakeVector => {
                input_scalar(graph, "x");
                input_scalar(graph, "y");
                output_vector(graph, "out");
            }
            MyNodeTemplate::MakeScalar => {
                input_scalar(graph, "value");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::CellPublicMethod => {
                graph.add_input_param(
                    node_id,
                    "name".into(),
                    MyDataType::String,
                    MyValueType::String {
                        value: "method".to_string(),
                    },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                output_string(graph, "argument1");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::EntFire => {
                input_action(graph);
                input_string(graph, "entity", InputParamKind::ConstantOnly);
                input_string(graph, "input", InputParamKind::ConstantOnly);
                input_string(graph, "value", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            MyNodeTemplate::Compare => {
                input_action(graph);
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_string(graph, "Data type (optional)", InputParamKind::ConstantOnly);
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_action(graph, "True");
                output_action(graph, "False");
            }
            MyNodeTemplate::ConcatString => {
                input_string(graph, "A", InputParamKind::ConnectionOrConstant);
                input_string(graph, "B", InputParamKind::ConnectionOrConstant);
                output_string(graph, "out");
            }
            MyNodeTemplate::CellWait => {
                input_action(graph);
                input_scalar(graph, "time");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::GetVar => {
                graph.add_input_param(node_id, String::from("variableName"),
                 MyDataType::InternalOutputName,
                  MyValueType::InternalVariableName { prevvalue: String::default(), value: String::from("CHOOSE") },
                  InputParamKind::ConstantOnly, true);
                //output_scalar(graph, "out");
            }
            MyNodeTemplate::SetVar => {
                input_action(graph);
                graph.add_input_param(node_id, String::from("variableName"),
                 MyDataType::InternalOutputName,
                  MyValueType::InternalVariableName { prevvalue: String::default(), value: String::from("CHOOSE") },
                  InputParamKind::ConstantOnly, true);
                //input_scalar(graph, "value");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::EventHandler => {
                input_action(graph);
                input_string(graph, "eventName", InputParamKind::ConstantOnly);
                output_action(graph, "outAction");
            }
            MyNodeTemplate::IntToString => {
                input_scalar(graph, "value");
                output_string(graph, "out");
            }
            MyNodeTemplate::Operation => {
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::FindEntByName => {
                input_string(graph, "entName", InputParamKind::ConstantOnly);
                input_string(graph, "entClass", InputParamKind::ConstantOnly);
                output_ehandle(graph, "out");
            }
            MyNodeTemplate::DebugWorldText => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                input_ehandle(graph, "hEntity");
                input_scalar(graph, "nTextOffset");
                input_scalar(graph, "flDuration");
                input_scalar(graph, "flVerticalOffset");
                input_bool(graph, "bAttached");
                input_vector3(graph, "color");
                input_scalar(graph, "flAlpha");
                input_scalar(graph, "flScale");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::DebugLog => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            MyNodeTemplate::FireOutput => {
                input_action(graph);
                graph.add_input_param(node_id, String::from("outputName"),
                 MyDataType::InternalOutputName,
                  MyValueType::InternalOutputName { prevvalue: String::default(), value: String::from("CHOOSE") },
                  InputParamKind::ConstantOnly, true);
                output_action(graph, "outAction");
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MyNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        vec![
            MyNodeTemplate::MakeScalar,
            MyNodeTemplate::MakeVector,
            MyNodeTemplate::AddScalar,
            MyNodeTemplate::SubtractScalar,
            MyNodeTemplate::AddVector,
            MyNodeTemplate::SubtractVector,
            MyNodeTemplate::VectorTimesScalar,
            MyNodeTemplate::CellPublicMethod,
            MyNodeTemplate::EntFire,
            MyNodeTemplate::Compare,
            MyNodeTemplate::ConcatString,
            MyNodeTemplate::CellWait,
            MyNodeTemplate::GetVar,
            MyNodeTemplate::SetVar,
            MyNodeTemplate::EventHandler,
            MyNodeTemplate::IntToString,
            MyNodeTemplate::Operation,
            MyNodeTemplate::FindEntByName,
            MyNodeTemplate::DebugWorldText,
            MyNodeTemplate::DebugLog,
            MyNodeTemplate::FireOutput,
        ]
    }
}

impl WidgetValueTrait for MyValueType {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type NodeData = MyNodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut MyGraphState,
        _node_data: &MyNodeData,
    ) -> Vec<MyResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        let mut responses = vec![];
        match self {
            MyValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value.y));
                });
            }
            MyValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    // if this is a custom added parameter...
                    let vec_params = _user_state.added_parameters.get(_node_id);
                    if let Some(params) = vec_params {
                        if params.iter().find(|&x| x == param_name).is_some() {
                            if ui.button("X").on_hover_text("Remove parameter").clicked() {
                                responses.push(MyResponse::RemoveOutputParam(_node_id, param_name.to_string()));
                            }
                        }
                    }
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            MyValueType::String { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value);
                });
            }
            MyValueType::Bool { value } => {
                ui.horizontal(|ui| {
                    ui.checkbox(value, param_name);
                });
            }
            MyValueType::Vec3 { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(&mut value.x).range(0..=255));
                    ui.add(DragValue::new(&mut value.y).range(0..=255));
                    ui.add(DragValue::new(&mut value.z).range(0..=255));
                });
            }
            MyValueType::Action => {
                ui.label("ACT");
            }
            MyValueType::EHandle => {
                ui.label("EHandle");
            }
            MyValueType::InternalOutputName {prevvalue, value} => {
                ui.horizontal(|ui| {
                    ui.label("Output");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.clone())
                        .show_ui(ui, |ui| {
                            for outputparam in _user_state.public_outputs.iter() {
                                ui.selectable_value(value, outputparam.name.clone(), outputparam.name.clone());
                            }
                        }
                    );
                });
                if prevvalue != value {
                    responses.push(MyResponse::ChangeOutputParamType(_node_id, value.clone()));
                    *prevvalue = value.clone();
                }
            }
            MyValueType::InternalVariableName {prevvalue, value} => {
                ui.horizontal(|ui| {
                    ui.label("Variable");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.clone())
                        .show_ui(ui, |ui| {
                            for var in _user_state.variables.iter() {
                                ui.selectable_value(value, var.name.clone(), var.name.clone());
                            }
                        }
                    );
                });
                if prevvalue != value {
                    responses.push(MyResponse::ChangeOutputParamType(_node_id, value.clone()));
                    *prevvalue = value.clone();
                }
            }
        }
        // This allows you to return your responses from the inline widgets.
        responses
    }
}

impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<MyNodeData, MyDataType, MyValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<MyResponse, MyNodeData>>
    where
        MyResponse: UserResponseTrait,
    {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.

        let mut responses = vec![];
        // add param to event handler node.
        if _graph.nodes.get(node_id).unwrap().user_data.template == MyNodeTemplate::EventHandler {
            let textbox_str: &mut String = user_state.custom_input_string.borrow_mut();
            ui.separator();
            ui.text_edit_singleline(textbox_str);
            if ui.button("Add parameter").clicked() {
                responses.push(NodeResponse::User(MyResponse::AddOutputParam(node_id, user_state.custom_input_string.clone())));
                if let Some(vec_params) = user_state.added_parameters.get_mut(node_id) {
                    vec_params.push(user_state.custom_input_string.clone());
                } else {
                    user_state.added_parameters.insert(node_id, vec![user_state.custom_input_string.clone()]);
                }
            }
        }
        responses
    }
}

pub type MyGraph = Graph<MyNodeData, MyDataType, MyValueType>;
type MyEditorState =
    GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

#[derive(Default, Serialize, Deserialize)]
pub struct NodeGraphExample {
    // The `GraphEditorState` is the top-level object. You "register" all your
    // custom types by specifying it as its generic parameters.
    state: MyEditorState,
    user_state: MyGraphState,
    outputs_dropdown_choices: Vec<PulseValueType>
}

#[cfg(feature = "persistence")]
impl NodeGraphExample {
    /// If the persistence feature is enabled, Called once before the first frame.
    /// Load previous app state (if any).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let grph: NodeGraphExample = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
            .unwrap_or_default();
        Self {
            state: grph.state,
            //file_dialog: FileDialog::new(),
            user_state: grph.user_state,
            outputs_dropdown_choices: vec![],
        }
    }
    pub fn update_output_node_param(&mut self, node_id: NodeId, name: &String) {
        let param = self.state.graph.nodes.get_mut(node_id).unwrap().get_input("param");
        if param.is_ok() {
            self.state.graph.remove_input_param(param.unwrap());
        }
        let mut dattype: MyDataType;
        let mut valtype: MyValueType;
        for output in self.user_state.public_outputs.iter() {
            if output.name == *name {
                match output.typ {
                    PulseValueType::PVAL_FLOAT(_)
                    | PulseValueType::PVAL_INT(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from("param"),
                            MyDataType::Scalar,
                            MyValueType::Scalar { value: 0f32 },
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_STRING(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from("param"),
                            MyDataType::String,
                            MyValueType::String {value: String::default()},
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_VEC3(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from("param"),
                            MyDataType::Vec3,
                            MyValueType::Vec3 { value: Vec3 { x: 0.0, y: 0.0, z: 0.0 } },
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    PulseValueType::PVAL_EHANDLE(_) => {
                        self.state.graph.add_input_param(
                            node_id,
                            String::from("param"),
                            MyDataType::EHandle,
                            MyValueType::EHandle,
                            InputParamKind::ConnectionOrConstant,
                            true,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(feature = "persistence")]
const PERSISTENCE_KEY: &str = "egui_node_graph";

impl eframe::App for NodeGraphExample {
    #[cfg(feature = "persistence")]
    /// If the persistence function is enabled,
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        use std::env;
        let save_dir = env::current_dir();
        if save_dir.is_ok() {
            let save_dir = save_dir.unwrap();
            let save_dir_str = save_dir.to_str();
            if save_dir_str.is_some() {
                eframe::storage_dir(save_dir_str.unwrap());
            }
        }
        eframe::set_value(storage, PERSISTENCE_KEY, &self);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("Pick save path").clicked() {
                    //self.file_dialog.pick_file();
                }
                if ui.button("Compile").clicked() {
                    compile_graph(&self.state.graph);
                }
                
            });
        });
        let mut output_scheduled_for_deletion: usize = usize::MAX; // we can get away with just one reference (it's not like the user can click more than one at once)
        let mut variable_scheduled_for_deletion: usize = usize::MAX;
        let mut output_node_updates = vec![];
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Outputs:");
            if ui.button("Add output").clicked() {
                self.outputs_dropdown_choices.push(PulseValueType::PVAL_INT(None));
                self.user_state.public_outputs.push(OutputDefinition { name: String::default(), typ: PulseValueType::PVAL_INT(None), typ_old: PulseValueType::PVAL_INT(None) });
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
                    ComboBox::from_label(format!("outputpick{}", idx))
                        .selected_text(format!("{:?}", outputdef.typ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut outputdef.typ, PulseValueType::PVAL_INT(None), "Integer");
                            ui.selectable_value(&mut outputdef.typ, PulseValueType::PVAL_STRING(None), "String");
                            ui.selectable_value(&mut outputdef.typ, PulseValueType::PVAL_FLOAT(None), "Float");
                            ui.selectable_value(&mut outputdef.typ, PulseValueType::PVAL_VEC3(None), "Vec3");
                            ui.selectable_value(&mut outputdef.typ, PulseValueType::PVAL_EHANDLE(None), "Entity Handle");
                        }
                    );
                });
                if outputdef.typ != outputdef.typ_old {
                    let node_ids: Vec<_> = self.state.graph.iter_nodes().collect();
                    for nodeid in node_ids {
                        let node = self.state.graph.nodes.get(nodeid).unwrap();
                        match node.user_data.template {
                            MyNodeTemplate::FireOutput => {
                                let inp = node.get_input("outputName");
                                let val = self.state.graph.get_input(inp.unwrap()).value().clone().try_output_name().unwrap();
                                if outputdef.name == val {
                                    output_node_updates.push((nodeid, outputdef.name.clone()));
                                }
                            }
                            _ => {}
                        }
                    }
                    outputdef.typ_old = outputdef.typ.clone();
                }
                // }
                // output_frame.end(ui);
            }
            ui.label("Variables:");
            for (idx, var) in self.user_state.variables.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("X").clicked() {
                        variable_scheduled_for_deletion = idx;
                    }
                    ui.label("Name");
                    ui.text_edit_singleline(&mut var.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Default value");
                    ui.text_edit_singleline(&mut var.default_value_buffer);
                });
                ui.horizontal(|ui| {
                    ui.label("Param type");
                    ComboBox::from_label(format!("varpick{}", idx))
                        .selected_text(format!("{:?}", &var.typ_and_default_value))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut var.typ_and_default_value, PulseValueType::PVAL_INT(None), "Integer");
                            ui.selectable_value(&mut var.typ_and_default_value, PulseValueType::PVAL_STRING(None), "String");
                            ui.selectable_value(&mut var.typ_and_default_value, PulseValueType::PVAL_FLOAT(None), "Float");
                            ui.selectable_value(&mut var.typ_and_default_value, PulseValueType::PVAL_VEC3(None), "Vec3");
                            ui.selectable_value(&mut var.typ_and_default_value, PulseValueType::PVAL_EHANDLE(None), "Entity Handle");
                        }
                    );
                    // compare only the variant of the enums
                    if std::mem::discriminant(&var.typ_and_default_value) != std::mem::discriminant(&var.old_typ) {
                        var.typ_and_default_value = match &var.typ_and_default_value {
                            PulseValueType::PVAL_INT(_) => {
                                var.default_value_buffer.parse::<i32>()
                                    .map(|x| PulseValueType::PVAL_INT(Some(x)))
                                    .unwrap_or(PulseValueType::PVAL_INT(None))
                            }
                            PulseValueType::PVAL_FLOAT(_) => {
                                var.default_value_buffer.parse::<f32>()
                                    .map(|x| PulseValueType::PVAL_FLOAT(Some(x)))
                                    .unwrap_or(PulseValueType::PVAL_FLOAT(None))
                            }
                            PulseValueType::PVAL_STRING(_) => {
                                PulseValueType::PVAL_STRING(Some(var.default_value_buffer.clone()))
                            }
                            _ => var.typ_and_default_value.to_owned()
                        };
                        var.typ_and_default_value = var.old_typ.clone();
                    }
                });
            }
        });
        if output_scheduled_for_deletion != usize::MAX {
            self.user_state.public_outputs.remove(output_scheduled_for_deletion);
        }
        if variable_scheduled_for_deletion != usize::MAX {
            self.user_state.variables.remove(variable_scheduled_for_deletion);
        }
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                let graph_response = self.state.draw_graph_editor(
                    ui,
                    AllMyNodeTemplates,
                    &mut self.user_state,
                    Vec::default(),
                );
                graph_response
            })
            .inner;

        for node_response in graph_response.node_responses {
            // Here, we ignore all other graph events. But you may find
            // some use for them. For example, by playing a sound when a new
            // connection is created
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    MyResponse::AddOutputParam(node_id, name) => {
                        let output_id = self.state.graph.add_output_param(
                            node_id,
                            name.clone(),
                            MyDataType::Scalar,
                        );
                        let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                        node.user_data.custom_named_outputs.insert(output_id, name);
                    }
                    MyResponse::RemoveOutputParam(node_id, name ) => {
                        let param = self.state.graph.nodes.get_mut(node_id).unwrap().get_output(&name).unwrap();
                        self.state.graph.remove_output_param(param);
                        let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                        let keys_to_remove: Vec<_> = node.user_data.custom_named_outputs.iter()
                            .filter_map(|(k, v)| if v == &name { Some(*k) } else { None })
                            .collect();
                        for k in keys_to_remove {
                            node.user_data.custom_named_outputs.remove(&k);
                        }
                    }
                    MyResponse::ChangeOutputParamType(node_id, name) => {
                        self.update_output_node_param(node_id, &name);
                    }
                }
            }
        }
        for (nodeid, name) in output_node_updates {
            self.update_output_node_param(nodeid, &name);
        }
    }
}

