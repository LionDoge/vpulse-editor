use crate::compiler::compile_graph;
use crate::bindings::*;
use crate::typing::*;
use crate::pulsetypes::*;
use core::panic;
use eframe::egui::output;
use eframe::egui::{self, ComboBox, DragValue};
use egui_node_graph2::*;
use rfd::{FileDialog, MessageDialog};
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use slotmap::SlotMap;
use std::borrow::BorrowMut;
use std::path::PathBuf;
use std::usize;
use std::{borrow::Cow, collections::HashMap};
// Compare this snippet from src/instruction_templates.rs:
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomOutputInfo {
    pub name: String,
    pub data: PulseValueType,
}
// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseNodeData {
    pub template: PulseNodeTemplate,
    pub custom_named_outputs: HashMap<OutputId, CustomOutputInfo>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum PulseDataType {
    Scalar,
    Vec2,
    Vec3,
    String,
    Bool,
    Action,
    EHandle,
    SndEventHandle,
    EntityName,
    InternalOutputName,
    InternalVariableName,
    Typ,
    EventBindingChoice,
    LibraryBindingChoice,
    SoundEventName,
    NoideChoice,
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
pub enum PulseGraphValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    String { value: String },
    Bool { value: bool },
    Vec3 { value: Vec3 },
    EHandle,
    SndEventHandle,
    SoundEventName { value: String },
    EntityName { value: String },
    Action,
    InternalOutputName { prevvalue: String, value: String },
    InternalVariableName { prevvalue: String, value: String },
    Typ { value: PulseValueType },
    EventBindingChoice { value: EventBinding },
    LibraryBindingChoice { value: FunctionBinding },
    NodeChoice {value: String}
}

impl Default for PulseGraphValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Scalar { value: 0.0 }
    }
}

impl PulseGraphValueType {
    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let PulseGraphValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }

    pub fn try_to_string(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::String { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to string", self)
        }
    }

    pub fn try_to_bool(self) -> anyhow::Result<bool> {
        if let PulseGraphValueType::Bool { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to bool", self)
        }
    }

    pub fn try_to_vec3(self) -> anyhow::Result<Vec3> {
        if let PulseGraphValueType::Vec3 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec3", self)
        }
    }

    pub fn try_output_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::InternalOutputName { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to output name", self)
        }
    }

    pub fn try_variable_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::InternalVariableName { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to variable name", self)
        }
    }

    pub fn try_pulse_type(self) -> anyhow::Result<PulseValueType> {
        if let PulseGraphValueType::Typ { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to variable name", self)
        }
    }

    pub fn try_entity_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::EntityName { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to entity name", self)
        }
    }

    pub fn try_event_binding(self) -> anyhow::Result<EventBinding> {
        if let PulseGraphValueType::EventBindingChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to event binding", self)
        }
    }

    pub fn try_library_binding(self) -> anyhow::Result<FunctionBinding> {
        if let PulseGraphValueType::LibraryBindingChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to library binding", self)
        }
    }

    pub fn try_sndevt_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::SoundEventName { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to string", self)
        }
    }

    pub fn try_node_id(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::NodeChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to node id", self)
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum PulseNodeTemplate {
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
    GraphHook,
    GetGameTime,
    SetNextThink,
    Convert,
    ForLoop,
    WhileLoop,
    StringToEntityName,
    InvokeLibraryBinding,
    FindEntitiesWithin,
    IsValidEntity,
    CompareOutput,
    CompareIf,
    IntSwitch,
    SoundEventStart,
    Function,
    CallNode,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug)]
pub enum PulseGraphResponse {
    AddOutputParam(NodeId, String, PulseValueType),
    RemoveOutputParam(NodeId, String),
    ChangeOutputParamType(NodeId, String),
    ChangeVariableParamType(NodeId, String),
    ChangeParamType(NodeId, String, PulseValueType),
    ChangeEventBinding(NodeId, EventBinding),
    ChangeFunctionBinding(NodeId, FunctionBinding),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseGraphState {
    pub added_parameters: SecondaryMap<NodeId, Vec<String>>,
    pub public_outputs: Vec<OutputDefinition>,
    pub variables: Vec<PulseVariable>,
    pub exposed_nodes: HashMap<NodeId, String>,

    pub save_file_path: PathBuf,
    #[serde(skip)]
    pub bindings: GraphBindings,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<PulseGraphState> for PulseDataType {
    fn data_type_color(&self, _user_state: &mut PulseGraphState) -> egui::Color32 {
        match self {
            PulseDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            PulseDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            PulseDataType::Vec3 => egui::Color32::from_rgb(238, 207, 109),
            PulseDataType::String => egui::Color32::from_rgb(52, 171, 235),
            PulseDataType::Action => egui::Color32::from_rgb(252, 3, 165),
            PulseDataType::EHandle => egui::Color32::from_rgb(11, 200, 31),
            PulseDataType::EntityName => egui::Color32::from_rgb(11, 77, 31),
            PulseDataType::Bool => egui::Color32::from_rgb(54, 61, 194),
            PulseDataType::InternalOutputName => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::InternalVariableName => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::Typ => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::EventBindingChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::LibraryBindingChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::SndEventHandle => egui::Color32::from_rgb(224, 123, 216),
            PulseDataType::SoundEventName => egui::Color32::from_rgb(52, 171, 235),
            PulseDataType::NoideChoice => egui::Color32::from_rgb(0, 0, 0),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            PulseDataType::Scalar => Cow::Borrowed("scalar"),
            PulseDataType::Vec2 => Cow::Borrowed("2d vector"),
            PulseDataType::Vec3 => Cow::Borrowed("3d vector"),
            PulseDataType::String => Cow::Borrowed("string"),
            PulseDataType::Bool => Cow::Borrowed("bool"),
            PulseDataType::Action => Cow::Borrowed("action"),
            PulseDataType::EHandle => Cow::Borrowed("EHandle"),
            PulseDataType::EntityName => Cow::Borrowed("Entity name"),
            PulseDataType::InternalOutputName => Cow::Borrowed("Output name"),
            PulseDataType::InternalVariableName => Cow::Borrowed("Variable name"),
            PulseDataType::Typ => Cow::Borrowed("Type"),
            PulseDataType::EventBindingChoice => Cow::Borrowed("Event binding"),
            PulseDataType::LibraryBindingChoice => Cow::Borrowed("Library binding"),
            PulseDataType::SndEventHandle => Cow::Borrowed("Sound event handle"),
            PulseDataType::SoundEventName => Cow::Borrowed("Sound event name"),
            PulseDataType::NoideChoice => Cow::Borrowed("Node reference"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for PulseNodeTemplate {
    type NodeData = PulseNodeData;
    type DataType = PulseDataType;
    type ValueType = PulseGraphValueType;
    type UserState = PulseGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            PulseNodeTemplate::CellPublicMethod => "Public Method",
            PulseNodeTemplate::EntFire => "EntFire",
            PulseNodeTemplate::Compare => "Compare",
            PulseNodeTemplate::ConcatString => "Concatenate strings",
            PulseNodeTemplate::CellWait => "Wait",
            PulseNodeTemplate::GetVar => "Load variable",
            PulseNodeTemplate::SetVar => "Save variable",
            PulseNodeTemplate::EventHandler => "Event Handler",
            PulseNodeTemplate::IntToString => "Int to string",
            PulseNodeTemplate::Operation => "Operation",
            PulseNodeTemplate::FindEntByName => "Find entity by name",
            PulseNodeTemplate::DebugWorldText => "Debug world text",
            PulseNodeTemplate::DebugLog => "Debug log",
            PulseNodeTemplate::FireOutput => "Fire output",
            PulseNodeTemplate::GraphHook => "Graph Hook",
            PulseNodeTemplate::GetGameTime => "Get game time",
            PulseNodeTemplate::SetNextThink => "Set next think",
            PulseNodeTemplate::Convert => "Convert",
            PulseNodeTemplate::ForLoop => "For loop",
            PulseNodeTemplate::WhileLoop => "While loop",
            PulseNodeTemplate::StringToEntityName => "String to entity name",
            PulseNodeTemplate::InvokeLibraryBinding => "Invoke library binding",
            PulseNodeTemplate::FindEntitiesWithin => "Find entities within",
            PulseNodeTemplate::IsValidEntity => "Is valid entity",
            PulseNodeTemplate::CompareOutput => "Compare output",
            PulseNodeTemplate::CompareIf => "If",
            PulseNodeTemplate::IntSwitch => "Int Switch",
            PulseNodeTemplate::SoundEventStart => "Sound event start",
            PulseNodeTemplate::Function => "Function",
            PulseNodeTemplate::CallNode => "Call node",
        })
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            PulseNodeTemplate::CellPublicMethod
            | PulseNodeTemplate::EventHandler
            | PulseNodeTemplate::GraphHook => vec!["Inflow"],
            PulseNodeTemplate::EntFire
            | PulseNodeTemplate::FindEntByName
            | PulseNodeTemplate::FindEntitiesWithin
            | PulseNodeTemplate::IsValidEntity => vec!["Entities"],
            PulseNodeTemplate::Compare
            | PulseNodeTemplate::CompareOutput
            | PulseNodeTemplate::CompareIf
            | PulseNodeTemplate::IntSwitch
            | PulseNodeTemplate::CallNode
            | PulseNodeTemplate::Function => vec!["Logic"],
            PulseNodeTemplate::Operation => vec!["Math"],
            PulseNodeTemplate::ConcatString => vec!["String"],
            PulseNodeTemplate::CellWait => vec!["Utility"],
            PulseNodeTemplate::GetVar | PulseNodeTemplate::SetVar => vec!["Variables"],
            PulseNodeTemplate::IntToString | PulseNodeTemplate::Convert | PulseNodeTemplate::StringToEntityName => vec!["Conversion"],
            PulseNodeTemplate::DebugWorldText | PulseNodeTemplate::DebugLog => vec!["Debug"],
            PulseNodeTemplate::FireOutput => vec!["Outflow"],
            PulseNodeTemplate::GetGameTime | PulseNodeTemplate::SetNextThink | PulseNodeTemplate::InvokeLibraryBinding => {
                vec!["Game functions"]
            }
            PulseNodeTemplate::ForLoop
            | PulseNodeTemplate::WhileLoop => vec!["Loops"],
            PulseNodeTemplate::SoundEventStart => vec!["Sound"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        PulseNodeData {
            template: *self,
            custom_named_outputs: HashMap::new(),
        }
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
        let input_string = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::String,
                PulseGraphValueType::String {
                    value: String::default(),
                },
                kind,
                true,
            );
        };
        let input_scalar = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Scalar,
                PulseGraphValueType::Scalar { value: 0.0 },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_bool = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Bool,
                PulseGraphValueType::Bool { value: false },
                kind,
                true,
            );
        };
        let input_ehandle = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::EHandle,
                PulseGraphValueType::EHandle,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_entityname = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::EntityName,
                PulseGraphValueType::EntityName {
                    value: String::default(),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector3 = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
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
        };
        let input_action = |graph: &mut PulseGraph| {
            graph.add_input_param(
                node_id,
                "ActionIn".to_string(),
                PulseDataType::Action,
                PulseGraphValueType::Action,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_typ = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Typ,
                PulseGraphValueType::Typ {
                    value: PulseValueType::PVAL_INT(None),
                },
                InputParamKind::ConstantOnly,
                true,
            );
        };
        let input_sndevt_name = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::SoundEventName,
                PulseGraphValueType::SoundEventName {
                    value: String::default(),
                },
                kind,
                true,
            );
        };

        let output_scalar = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Scalar);
        };
        let output_string = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::String);
        };
        let output_action = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Action);
        };
        let output_ehandle = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::EHandle);
        };
        let output_entityname = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::EntityName);
        };
        let output_bool = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Bool);
        };

        // input_action(graph);
        // output_action(graph);
        match self {
            PulseNodeTemplate::CellPublicMethod => {
                graph.add_input_param(
                    node_id,
                    "name".into(),
                    PulseDataType::String,
                    PulseGraphValueType::String {
                        value: "method".to_string(),
                    },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                output_string(graph, "argument1");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::EntFire => {
                input_action(graph);
                input_entityname(graph, "entity");
                input_string(graph, "input", InputParamKind::ConstantOnly);
                input_string(graph, "value", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::Compare => {
                input_action(graph);
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_typ(graph, "type");
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_action(graph, "True");
                output_action(graph, "False");
            }
            PulseNodeTemplate::ConcatString => {
                input_string(graph, "A", InputParamKind::ConnectionOrConstant);
                input_string(graph, "B", InputParamKind::ConnectionOrConstant);
                output_string(graph, "out");
            }
            PulseNodeTemplate::CellWait => {
                input_action(graph);
                input_scalar(graph, "time");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GetVar => {
                graph.add_input_param(
                    node_id,
                    String::from("variableName"),
                    PulseDataType::InternalVariableName,
                    PulseGraphValueType::InternalVariableName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                //output_scalar(graph, "out");
            }
            PulseNodeTemplate::SetVar => {
                input_action(graph);
                graph.add_input_param(
                    node_id,
                    String::from("variableName"),
                    PulseDataType::InternalVariableName,
                    PulseGraphValueType::InternalVariableName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                //input_scalar(graph, "value");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::EventHandler => {
                graph.add_input_param(
                    node_id,
                    String::from("event"),
                    PulseDataType::EventBindingChoice,
                    PulseGraphValueType::EventBindingChoice 
                    { value: _user_state.bindings.events[0].clone() },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::IntToString => {
                input_scalar(graph, "value");
                output_string(graph, "out");
            }
            PulseNodeTemplate::Operation => {
                input_typ(graph, "type");
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::FindEntByName => {
                input_entityname(graph, "entName");
                input_string(graph, "entClass", InputParamKind::ConstantOnly);
                output_ehandle(graph, "out");
            }
            PulseNodeTemplate::DebugWorldText => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                input_ehandle(graph, "hEntity");
                input_scalar(graph, "nTextOffset");
                input_scalar(graph, "flDuration");
                input_scalar(graph, "flVerticalOffset");
                input_bool(graph, "bAttached", InputParamKind::ConstantOnly);
                input_vector3(graph, "color");
                input_scalar(graph, "flAlpha");
                input_scalar(graph, "flScale");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::DebugLog => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::FireOutput => {
                input_action(graph);
                graph.add_input_param(
                    node_id,
                    String::from("outputName"),
                    PulseDataType::InternalOutputName,
                    PulseGraphValueType::InternalOutputName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GraphHook => {
                input_string(graph, "hookName", InputParamKind::ConstantOnly);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GetGameTime => {
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::SetNextThink => {
                input_action(graph);
                input_scalar(graph, "dt");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::Convert => {
                input_typ(graph, "typefrom");
                input_typ(graph, "typeto");
                input_string(graph, "entityclass", InputParamKind::ConstantOnly);
                input_scalar(graph, "input");
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::ForLoop => {
                input_action(graph);
                input_scalar(graph, "start");
                input_scalar(graph, "end");
                input_scalar(graph, "step");
                output_scalar(graph, "index");
                output_action(graph, "loopAction");
                output_action(graph, "endAction");
            }
            PulseNodeTemplate::StringToEntityName => {
                input_string(graph, "entityName", InputParamKind::ConnectionOrConstant);
                output_entityname(graph, "out");
            }
            PulseNodeTemplate::InvokeLibraryBinding => {
                graph.add_input_param(
                    node_id,
                    String::from("binding"),
                    PulseDataType::LibraryBindingChoice,
                    PulseGraphValueType::LibraryBindingChoice {
                        value: _user_state.bindings.gamefunctions[0].clone(),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
            }
            PulseNodeTemplate::FindEntitiesWithin => {
                input_string(graph, "classname", InputParamKind::ConstantOnly);
                input_ehandle(graph, "pSearchFromEntity");
                input_scalar(graph, "flSearchRadius");
                input_ehandle(graph, "pStartEntity");
                output_ehandle(graph, "out");
            }
            PulseNodeTemplate::IsValidEntity => {
                input_action(graph);
                input_ehandle(graph, "hEntity");
                output_action(graph, "True");
                output_action(graph, "False");
            }
            PulseNodeTemplate::CompareOutput => {
                input_typ(graph, "type");
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_bool(graph, "out");
            }
            PulseNodeTemplate::WhileLoop => {
                input_action(graph);
                input_bool(graph, "do-while", InputParamKind::ConstantOnly);
                input_bool(graph, "condition", InputParamKind::ConnectionOnly);
                output_action(graph, "loopAction");
                output_action(graph, "endAction");
            }
            PulseNodeTemplate::CompareIf => {
                input_action(graph);
                input_bool(graph, "condition", InputParamKind::ConnectionOnly);
                output_action(graph, "True");
                output_action(graph, "False");
                output_action(graph, "Either");
            }
            PulseNodeTemplate::IntSwitch => {
                input_action(graph);
                input_scalar(graph, "value");
                // cases will be added dynamically by user
                // this field will be a buffer that will be used to create the cases
                // once the button to add it is pressed - which is defined in bottom_ui func.
                graph.add_input_param(
                    node_id,
                    "caselabel".into(),
                    PulseDataType::Scalar,
                    PulseGraphValueType::Scalar { value: 0.0 },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "defaultcase");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::SoundEventStart => {
                input_sndevt_name(graph, "strSoundEventName", InputParamKind::ConnectionOrConstant);
                input_ehandle(graph, "hTargetEntity");
                graph.add_output_param(node_id, "retval".into(), PulseDataType::SndEventHandle);
            }
            PulseNodeTemplate::Function => {
                _user_state.exposed_nodes.insert(node_id, String::default());
                graph.add_input_param(
                    node_id,
                    "funcName".into(),
                    PulseDataType::String,
                    PulseGraphValueType::String { value: String::default() },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::CallNode => {
                graph.add_input_param(
                    node_id,
                    "funcName".into(),
                    PulseDataType::NoideChoice,
                    PulseGraphValueType::NodeChoice { value: String::default() },
                    InputParamKind::ConstantOnly,
                    true,
                );
                input_action(graph);
                output_action(graph, "outAction");
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = PulseNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        vec![
            PulseNodeTemplate::CellPublicMethod,
            PulseNodeTemplate::EntFire,
            //PulseNodeTemplate::Compare,
            PulseNodeTemplate::ConcatString,
            PulseNodeTemplate::CellWait,
            PulseNodeTemplate::GetVar,
            PulseNodeTemplate::SetVar,
            PulseNodeTemplate::EventHandler,
            //PulseNodeTemplate::IntToString,
            PulseNodeTemplate::Operation,
            PulseNodeTemplate::FindEntByName,
            PulseNodeTemplate::DebugWorldText,
            PulseNodeTemplate::DebugLog,
            PulseNodeTemplate::FireOutput,
            PulseNodeTemplate::GraphHook,
            PulseNodeTemplate::GetGameTime,
            PulseNodeTemplate::SetNextThink,
            PulseNodeTemplate::Convert,
            PulseNodeTemplate::ForLoop,
            PulseNodeTemplate::WhileLoop,
            PulseNodeTemplate::StringToEntityName,
            PulseNodeTemplate::InvokeLibraryBinding,
            PulseNodeTemplate::FindEntitiesWithin,
            PulseNodeTemplate::IsValidEntity,
            PulseNodeTemplate::CompareOutput,
            PulseNodeTemplate::CompareIf,
            PulseNodeTemplate::IntSwitch,
            PulseNodeTemplate::SoundEventStart,
            PulseNodeTemplate::Function,
            PulseNodeTemplate::CallNode,
        ]
    }
}

impl WidgetValueTrait for PulseGraphValueType {
    type Response = PulseGraphResponse;
    type UserState = PulseGraphState;
    type NodeData = PulseNodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut PulseGraphState,
        _node_data: &PulseNodeData,
    ) -> Vec<PulseGraphResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        let mut responses = vec![];
        match self {
            PulseGraphValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value.y));
                });
            }
            PulseGraphValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    // if this is a custom added parameter...
                    let vec_params = _user_state.added_parameters.get(_node_id);
                    if let Some(params) = vec_params {
                        if params.iter().find(|&x| x == param_name).is_some() {
                            if ui.button("X").on_hover_text("Remove parameter").clicked() {
                                responses.push(PulseGraphResponse::RemoveOutputParam(
                                    _node_id,
                                    param_name.to_string(),
                                ));
                            }
                        }
                    }
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            PulseGraphValueType::String { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value);
                });
            }
            PulseGraphValueType::Bool { value } => {
                ui.horizontal(|ui| {
                    ui.checkbox(value, param_name);
                });
            }
            PulseGraphValueType::Vec3 { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(&mut value.x).range(0..=255));
                    ui.add(DragValue::new(&mut value.y).range(0..=255));
                    ui.add(DragValue::new(&mut value.z).range(0..=255));
                });
            }
            PulseGraphValueType::Action => {
                ui.label(format!("Action {}", param_name));
            }
            PulseGraphValueType::EHandle => {
                ui.label(format!("EHandle {}", param_name));
            }
            PulseGraphValueType::SndEventHandle => {
                ui.label(format!("SNDEVT {}", param_name));
            }
            PulseGraphValueType::SoundEventName { value } => {
                ui.horizontal(|ui| {
                    ui.label(format!("SNDEVT {}", param_name));
                    ui.text_edit_singleline(value);
                });
            }
            PulseGraphValueType::EntityName { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value);
                });
            }
            PulseGraphValueType::InternalOutputName { prevvalue, value } => {
                ui.horizontal(|ui| {
                    ui.label("Output");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.clone())
                        .show_ui(ui, |ui| {
                            for outputparam in _user_state.public_outputs.iter() {
                                ui.selectable_value(
                                    value,
                                    outputparam.name.clone(),
                                    outputparam.name.clone(),
                                );
                            }
                        });
                });
                if prevvalue != value {
                    responses.push(PulseGraphResponse::ChangeOutputParamType(
                        _node_id,
                        value.clone(),
                    ));
                    *prevvalue = value.clone();
                }
            }
            PulseGraphValueType::InternalVariableName { prevvalue, value } => {
                ui.horizontal(|ui| {
                    ui.label("Variable");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.clone())
                        .show_ui(ui, |ui| {
                            for var in _user_state.variables.iter() {
                                ui.selectable_value(value, var.name.clone(), var.name.clone());
                            }
                        });
                });
                if prevvalue != value {
                    responses.push(PulseGraphResponse::ChangeVariableParamType(
                        _node_id,
                        value.clone(),
                    ));
                    *prevvalue = value.clone();
                }
            }
            PulseGraphValueType::Typ { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ComboBox::from_id_salt((_node_id, param_name))
                        .selected_text(value.to_string())
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(value, PulseValueType::PVAL_INT(None), "Integer")
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_INT(None),
                                ));
                            };
                            if ui
                                .selectable_value(value, PulseValueType::PVAL_FLOAT(None), "Float")
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_FLOAT(None),
                                ));
                            };
                            if ui
                                .selectable_value(value, PulseValueType::PVAL_VEC3(None), "Vector")
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_VEC3(None),
                                ));
                            };
                            if ui
                                .selectable_value(
                                    value,
                                    PulseValueType::PVAL_EHANDLE(None),
                                    "Entity Handle",
                                )
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_EHANDLE(None),
                                ));
                            };
                            if ui
                                .selectable_value(
                                    value,
                                    PulseValueType::PVAL_STRING(None),
                                    "String",
                                )
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_STRING(None),
                                ));
                            };
                            if ui
                                .selectable_value(
                                    value,
                                    PulseValueType::PVAL_BOOL,
                                    "Boolean",
                                )
                                .clicked()
                            {
                                responses.push(PulseGraphResponse::ChangeParamType(
                                    _node_id,
                                    param_name.to_string(),
                                    PulseValueType::PVAL_BOOL,
                                ));
                            };
                        });
                });
            }
            PulseGraphValueType::EventBindingChoice { value } => {
                ui.horizontal(|ui| { 
                    ui.label("Event");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.displayname.clone())
                        .show_ui(ui, |ui| {
                            for event in _user_state.bindings.events.iter() {
                                let str = event.displayname.as_str();
                                if ui.selectable_value::<EventBinding>(value, 
                                    event.clone(),
                                     str).clicked() {
                                    responses.push(
                                        PulseGraphResponse::ChangeEventBinding(_node_id, event.clone())
                                    );
                                }
                            }
                        });
                });
            }
            PulseGraphValueType::LibraryBindingChoice { value } => {
                ui.horizontal(|ui| { 
                    ui.label("Function");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.displayname.clone())
                        .show_ui(ui, |ui| {
                            for func in _user_state.bindings.gamefunctions.iter() {
                                let str = func.displayname.as_str();
                                if ui.selectable_value::<FunctionBinding>(value, 
                                    func.clone(),
                                     str).clicked() {
                                    responses.push(
                                        PulseGraphResponse::ChangeFunctionBinding(_node_id, func.clone())
                                    );
                                }
                            }
                        });
                });
            }
            PulseGraphValueType::NodeChoice { value } => {
                ui.horizontal(|ui| { 
                    ui.label("Node");
                    ComboBox::from_id_salt(_node_id)
                        .selected_text(value.clone())
                        .show_ui(ui, |ui| {
                            for node in _user_state.graph.nodes.iter() {
                                let str = node.user_data.node_graph_label(_user_state);
                                if ui.selectable_value::<String>(value,
                                    str.clone(),
                                     str).clicked() {
                                    responses.push(
                                        PulseGraphResponse::ChangeNodeBinding(_node_id, str)
                                    );
                                }
                            }
                        });
                });
            }
        }
        // This allows you to return your responses from the inline widgets.
        responses
    }
}

impl UserResponseTrait for PulseGraphResponse {}
impl NodeDataTrait for PulseNodeData {
    type Response = PulseGraphResponse;
    type UserState = PulseGraphState;
    type DataType = PulseDataType;
    type ValueType = PulseGraphValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<PulseNodeData, PulseDataType, PulseGraphValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<PulseGraphResponse, PulseNodeData>>
    where
        PulseGraphResponse: UserResponseTrait,
    {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.

        let mut responses = vec![];
        // add param to event handler node.
        let node = _graph.nodes.get(node_id).unwrap();
        if node.user_data.template == PulseNodeTemplate::IntSwitch
        {
            let param = node.get_input("caselabel")
                .expect("caselabel is not defined for IntSwitch node, this is a programming error!");
            let param_value = _graph.get_input(param).value().clone().try_to_scalar().unwrap().round() as i32;
            if ui.button("Add parameter").clicked() {
                let param_name = format!("{}", param_value);
                responses.push(NodeResponse::User(PulseGraphResponse::AddOutputParam(
                    node_id,
                    param_name.clone(),
                    PulseValueType::PVAL_ACT
                )));
                if let Some(vec_params) = user_state.added_parameters.get_mut(node_id) {
                    vec_params.push(param_name);
                } else {
                    user_state
                        .added_parameters
                        .insert(node_id, vec![param_name]);
                }
            }
        }
        responses
    }
}

pub type PulseGraph = Graph<PulseNodeData, PulseDataType, PulseGraphValueType>;
type MyEditorState = GraphEditorState<
    PulseNodeData,
    PulseDataType,
    PulseGraphValueType,
    PulseNodeTemplate,
    PulseGraphState,
>;

#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseGraphEditor {
    state: MyEditorState,
    user_state: PulseGraphState,
    outputs_dropdown_choices: Vec<PulseValueType>,
}


impl PulseGraphEditor {
    pub fn update_output_node_param(&mut self, node_id: NodeId, name: &String, input_name: &str) {
        let param = self
            .state
            .graph
            .nodes
            .get_mut(node_id)
            .unwrap()
            .get_input(input_name);
        if param.is_ok() {
            self.state.graph.remove_input_param(param.unwrap());
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
                if param.is_ok() {
                    self.state.graph.remove_output_param(param.unwrap());
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
                if param.is_ok() {
                    self.state.graph.remove_input_param(param.unwrap());
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
                if !param_a.is_ok() || !param_b.is_ok() || !param_out.is_ok() {
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
            PulseNodeTemplate::Compare
            | PulseNodeTemplate::CompareOutput => {
                if new_type.is_none() {
                    panic!("update_node_inputs_outputs() ended up on node that requires new value type from response, but it was not provided");
                }
                let new_type = new_type.unwrap();
                let param_a = node.get_input("A");
                let param_b = node.get_input("B");
                if !param_a.is_ok() || !param_b.is_ok() {
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
        let binding_chooser_input_id = node.get_input("binding")
        .expect("Expected 'Invoke library binding' node to have 'binding' input param");
        for input in input_ids {
            if input != binding_chooser_input_id {
                self.state.graph.remove_input_param(input);
            }
        }
        // If it's action type (nodes that usually don't provide a value) make it have in and out actions.
        if binding.typ == LibraryBindingType::Action {
            self.state.graph.add_output_param(*node_id, "outAction".to_string(), PulseDataType::Action);
            self.state.graph.add_input_param(*node_id, "ActionIn".to_string(),
            PulseDataType::Action,
            PulseGraphValueType::Action,
            InputParamKind::ConnectionOrConstant,
            true);
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
        self.state.graph.add_output_param(*node_id, "outAction".to_string(), PulseDataType::Action);
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
}

//#[cfg(feature = "persistence")]
impl PulseGraphEditor {
    /// If the persistence feature is enabled, Called once before the first frame.
    /// Load previous app state (if any).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(feature = "persistence")]
        let mut grph: PulseGraphEditor = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
            .unwrap_or_default();
        #[cfg(not(feature = "persistence"))]
        let mut grph: PulseGraphEditor = Default::default();
        // Self {
        //     state: grph.state,
        //     user_state: grph.user_state,
        //     outputs_dropdown_choices: vec![],
        // }
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
        PulseValueType::PVAL_VEC3(_) => {
            var.data_type = PulseDataType::Vec3;
            PulseValueType::PVAL_VEC3(Some(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }))
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
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("Compile").clicked() {
                    let compile_res = compile_graph(&self.state.graph, &self.user_state);
                    if compile_res.is_err() {
                        MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Compile failed")
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_description(compile_res.err().unwrap())
                            .show();
                    }
                }
                if ui.button("Pick save location").clicked() {
                    let chosen_file = FileDialog::new()
                        .add_filter("Pulse Graph", &["vpulse"])
                        .save_file();
                    if chosen_file.is_some() {
                        self.user_state.save_file_path = chosen_file.unwrap();
                    }
                    // else it was most likely cancelled.
                }
            });
        });
        let mut output_scheduled_for_deletion: usize = usize::MAX; // we can get away with just one reference (it's not like the user can click more than one at once)
        let mut variable_scheduled_for_deletion: usize = usize::MAX;
        let mut output_node_updates = vec![];
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Outputs:");
                if ui.button("Add output").clicked() {
                    self.outputs_dropdown_choices
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
                        ComboBox::from_label(format!("outputpick{}", idx))
                            .selected_text(outputdef.typ.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_INT(None),
                                    "Integer",
                                );
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_STRING(None),
                                    "String",
                                );
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_FLOAT(None),
                                    "Float",
                                );
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_VEC3(None),
                                    "Vec3",
                                );
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_EHANDLE(None),
                                    "Entity Handle",
                                );
                                ui.selectable_value(
                                    &mut outputdef.typ,
                                    PulseValueType::PVAL_SNDEVT_GUID(None),
                                    "Sound Event",
                                );
                            });
                    });
                    if outputdef.typ != outputdef.typ_old {
                        let node_ids: Vec<_> = self.state.graph.iter_nodes().collect();
                        for nodeid in node_ids {
                            let node = self.state.graph.nodes.get(nodeid).unwrap();
                            match node.user_data.template {
                                PulseNodeTemplate::FireOutput => {
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
                                _ => {}
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
                    self.outputs_dropdown_choices
                        .push(PulseValueType::PVAL_INT(None));
                    self.user_state.variables.push(PulseVariable {
                        name: String::default(),
                        typ_and_default_value: PulseValueType::PVAL_INT(None),
                        data_type: PulseDataType::Scalar,
                        old_typ: PulseValueType::PVAL_INT(None),
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
                        let response = ui.text_edit_singleline(&mut var.default_value_buffer);
                        if response.changed() {
                            update_variable_data(var);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Param type");
                        ComboBox::from_label(format!("varpick{}", idx))
                            .selected_text(var.typ_and_default_value.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_INT(None),
                                    "Integer",
                                );
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_STRING(None),
                                    "String",
                                );
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_FLOAT(None),
                                    "Float",
                                );
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_VEC3(None),
                                    "Vec3",
                                );
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_EHANDLE(None),
                                    "EHandle",
                                );
                                ui.selectable_value(
                                    &mut var.typ_and_default_value,
                                    PulseValueType::PVAL_SNDEVT_GUID(None),
                                    "Sound Event",
                                );
                            });
                        // add the default value.
                        // compare only the variant of the enums, if they differ assign default value and data type.
                        if std::mem::discriminant(&var.typ_and_default_value)
                            != std::mem::discriminant(&var.old_typ)
                        {
                            update_variable_data(var);
                            var.old_typ = var.typ_and_default_value.clone();
                        }
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
            // handle all responses generated by the graph ui...
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    // node that supports adding parameters is trying to add one
                    PulseGraphResponse::AddOutputParam(node_id, name,data) => {
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
                        let output_info = CustomOutputInfo {
                            name,
                            data,
                        };
                        // remember the custom output
                        node.user_data.custom_named_outputs.insert(output_id, output_info);
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
                }
            }
        }
        for (nodeid, name) in output_node_updates {
            self.update_output_node_param(nodeid, &name, "param");
        }
    }
}
