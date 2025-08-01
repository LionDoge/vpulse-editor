use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use egui_node_graph2::*;
use crate::typing::*;
use crate::pulsetypes::*;
use crate::bindings::{GraphBindings, FunctionBinding, EventBinding};

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseNodeData {
    pub template: PulseNodeTemplate,
    pub custom_named_outputs: HashMap<OutputId, CustomOutputInfo>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(Default, PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum PulseDataType {
    #[default]
    Scalar,
    Vec2,
    Vec3,
    Vec3Local,
    Color,
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
    Any,
    SchemaEnum,
    CommentBox,
    Vec4,
    QAngle,
    Transform,
    TransformWorldspace,
    Resource,
    Array,
    GameTime,
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
    Vec2 {
        value: Vec2,
    },
    Scalar {
        value: f32,
    },
    String {
        value: String,
    },
    Bool {
        value: bool,
    },
    Vec3 {
        value: Vec3,
    },
    Vec3Local {
        value: Vec3,
    },
    Color {
        value: [f32; 4],
    },
    EHandle,
    SndEventHandle,
    SoundEventName {
        value: String,
    },
    EntityName {
        value: String,
    },
    Action,
    InternalOutputName {
        prevvalue: String,
        value: String,
    },
    InternalVariableName {
        prevvalue: String,
        value: String,
    },
    Typ {
        value: PulseValueType,
    },
    EventBindingChoice {
        value: EventBindingIndex,
    },
    LibraryBindingChoice {
        value: LibraryBindingIndex,
    },
    NodeChoice {
        node: Option<NodeId>,
    },
    Any,
    SchemaEnum {
        enum_type: SchemaEnumType,
        value: SchemaEnumValue,
    },
    CommentBox {value: String},
    Vec4 {
        value: Vec4,
    },
    QAngle {
        value: Vec3,
    },
    Transform,
    TransformWorldspace,
    Resource {
        resource_type: Option<String>, // Used for displaying in the UI only.
        value: String,
    },
    Array,
    GameTime,
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
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
    ListenForEntityOutput,
    Timeline,
    #[default]
    Comment,
    SetAnimGraphParam,
    ConstantBool,
    ConstantFloat,
    ConstantString,
    ConstantVec3,
    ConstantInt,
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
    ChangeRemoteNodeId(NodeId, NodeId),
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
    pub exposed_nodes: SecondaryMap<NodeId, String>,
    pub outputs_dropdown_choices: Vec<PulseValueType>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    pub save_file_path: Option<PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub bindings: GraphBindings,
}

// Compare this snippet from src/instruction_templates.rs:
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomOutputInfo {
    pub name: String,
    pub data: PulseValueType,
}
pub struct AllMyNodeTemplates;

#[cfg(feature = "nongame_asset_build")]
#[derive(Deserialize)]
pub struct EditorConfig {
    pub python_interpreter: String,
    pub assetassembler_path: PathBuf,
    pub red2_template_path: PathBuf,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "persistence", serde(tag = "version"))]
pub enum FileVersion {
    #[default]
    #[cfg_attr(feature = "persistence", serde(rename = "v1"))]
    V1,
    #[cfg_attr(feature = "persistence", serde(rename = "v2"))]
    V2,
}

pub type PulseGraph = Graph<PulseNodeData, PulseDataType, PulseGraphValueType>;
pub type MyEditorState = GraphEditorState<
    PulseNodeData,
    PulseDataType,
    PulseGraphValueType,
    PulseNodeTemplate,
    PulseGraphState,
>;
