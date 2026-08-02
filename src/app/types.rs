use std::marker::PhantomData;
use std::{path::PathBuf, borrow::Cow};
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use egui_node_graph2::*;
use crate::typing::*;
use crate::pulsetypes::*;
use crate::bindings::{GraphBindings, FunctionBinding, EventBinding};

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseNodeData {
    pub template: PulseNodeTemplate,
    #[serde(skip)]
    #[allow(dead_code)]
    pub custom_named_outputs: PhantomData<()>, // deprecated (left for compatibility)
    #[serde(skip)]
    #[allow(dead_code)]
    pub added_parameters: PhantomData<()>, // deprecated (left for compatibility)
    pub input_hint_text: Option<Cow<'static, str>>,
    // used for polymorphic output types
    pub custom_output_type: Option<PulseValueType>,
    #[serde(default)]
    pub added_inputs: Vec<InputId>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(Default, PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum PulseDataType {
    #[default]
    Scalar,
    Integer,
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
    HookBindingChoice,
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
    TypeSafeInteger,
    GeneralEnum,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
///
/// Empty enums mean that we do not support inputting, or storing a default value for that type.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum PulseGraphValueType {
    Vec2 {
        value: Vec2,
    },
    Integer {
        value: i32,
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
    HookBindingChoice {
        value: HookBindingIndex,
    },
    NodeChoice {
        node: Option<NodeId>,
    },
    Any,
    /* Deprecated version, left for compatibility with old saved graphs, any older ones should migrate to SchemaEnumChoice */
    SchemaEnum {
        enum_type: SchemaEnumType,
        value: SchemaEnumValue,
    },
    SchemaEnumChoice {
        enum_type: EnumBindingIndex,
        enum_variant: EnumBindingValueIndex,
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
    Array {
        #[serde(default)]
        array_type: PulseDataType
    },
    GameTime,
    TypeSafeInteger {
        integer_type: String,
    },
    GeneralEnumChoice {
        value: GeneralEnumChoice,
    }
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
    NewArray,
    LibraryBindingAssigned { binding: LibraryBindingIndex },
    GetArrayElement,
    ScaleVector,
    ReturnValue,
    ForEach,
    And,
    Or,
    Not,
    RandomInt,
    RandomFloat,
    EntOutputHandler,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug)]
pub enum PulseGraphResponse {
    AddOutputParam(NodeId, String, PulseDataType),
    // autoindex (bool) will automatically append the last element index + 1 to the provided name
    AddCustomInputParam(NodeId, String, PulseDataType, PulseGraphValueType, InputParamKind, bool),
    RemoveCustomInputParam(NodeId, InputId),
    RemoveOutputParam(NodeId, String),
    ChangeOutputParamType(NodeId, String),
    ChangeVariableParamType(NodeId, String),
    ChangeParamType(NodeId, String, PulseValueType),
    ChangeEventBinding(NodeId, EventBinding),
    #[allow(dead_code)]
    ChangeFunctionBinding(NodeId, FunctionBinding),
    ChangeRemoteNodeId(NodeId, NodeId),
    UpdatePolymorphicTypes(NodeId),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseGraphState {
    pub public_outputs: Vec<OutputDefinition>,
    pub variables: Vec<PulseVariable>,
    pub exposed_nodes: SecondaryMap<NodeId, String>,
    pub outputs_dropdown_choices: Vec<PulseValueType>,

    pub save_file_path: Option<PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub bindings: GraphBindings,

    #[cfg_attr(feature = "persistence", serde(default))]
    pub graph_domain: String,
    #[cfg_attr(feature = "persistence", serde(default))]
    pub graph_subtype: String,
}

impl Default for PulseGraphState {
    fn default() -> Self {
        PulseGraphState {
            public_outputs: Vec::new(),
            variables: Vec::new(),
            exposed_nodes: SecondaryMap::new(),
            outputs_dropdown_choices: vec![],
            save_file_path: None,
            bindings: GraphBindings::default(),
            graph_domain: "ServerEntity".to_string(),
            graph_subtype: "PVAL_EHANDLE:point_pulse".to_string(),
        }
    }
}

pub struct AllMyNodeTemplates {
    pub game_function_count: usize,
}

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

impl PulseDataType {
    pub fn get_comparable_types() -> Vec<PulseDataType> {
        vec![
            PulseDataType::Integer,
            PulseDataType::Scalar,
            PulseDataType::String,
            PulseDataType::Bool,
            PulseDataType::EHandle,
            PulseDataType::EntityName,
            PulseDataType::Vec2,
            PulseDataType::Vec3,
            PulseDataType::Vec3Local,
            PulseDataType::Vec4,
            PulseDataType::Color,
            PulseDataType::Array,
            PulseDataType::QAngle,
            PulseDataType::GameTime,
        ]
    }
    pub fn get_operatable_types() -> Vec<PulseDataType> {
        vec![
            PulseDataType::Integer,
            PulseDataType::Scalar,
            PulseDataType::String,
            PulseDataType::Bool,
            PulseDataType::EHandle,
            PulseDataType::EntityName,
            PulseDataType::Vec2,
            PulseDataType::Vec3,
            PulseDataType::Vec3Local,
            PulseDataType::Vec4,
        ]
    }
    pub fn get_scalable_types() -> Vec<PulseDataType> {
        vec![
            PulseDataType::Vec2,
            PulseDataType::Vec3,
            PulseDataType::Vec3Local,
            PulseDataType::Vec4,
        ]
    }
    pub fn get_variable_supported_types() -> Vec<PulseDataType> {
        vec![
            PulseDataType::Integer,
            PulseDataType::Scalar,
            PulseDataType::String,
            PulseDataType::Bool,
            PulseDataType::Vec2,
            PulseDataType::Vec3,
            PulseDataType::Vec3Local,
            PulseDataType::Vec4,
            PulseDataType::QAngle,
            PulseDataType::Transform,
            PulseDataType::TransformWorldspace,
            PulseDataType::Color,
            PulseDataType::EHandle,
            PulseDataType::EntityName,
            PulseDataType::SndEventHandle,
            PulseDataType::Array,
            PulseDataType::Resource,
            PulseDataType::GameTime,
            PulseDataType::TypeSafeInteger,
            PulseDataType::SchemaEnum,
        ]
    }
    pub fn get_vector_types() -> Vec<PulseDataType> {
        vec![
            PulseDataType::Vec2,
            PulseDataType::Vec3,
            PulseDataType::Vec3Local,
            PulseDataType::Vec4,
        ]
    }
}

#[allow(clippy::from_over_into)] // Providing From conversion won't work in this scenario
impl Into<PulseGraphValueType> for PulseDataType {
    fn into(self) -> PulseGraphValueType {
        match self {
            PulseDataType::Integer => PulseGraphValueType::Integer { value: 0 },
            PulseDataType::Scalar => PulseGraphValueType::Scalar { value: 0.0 },
            PulseDataType::String => PulseGraphValueType::String { value: String::new() },
            PulseDataType::Bool => PulseGraphValueType::Bool { value: false },
            PulseDataType::EHandle => PulseGraphValueType::EHandle,
            PulseDataType::EntityName => PulseGraphValueType::EntityName { value: String::new() },
            PulseDataType::Vec2 => PulseGraphValueType::Vec2 { value: Vec2::default() },
            PulseDataType::Vec3 => PulseGraphValueType::Vec3 { value: Vec3::default() },
            PulseDataType::Vec3Local => PulseGraphValueType::Vec3Local { value: Vec3::default() },
            PulseDataType::Vec4 => PulseGraphValueType::Vec4 { value: Vec4::default() },
            PulseDataType::Color => PulseGraphValueType::Color { value: [0.0, 0.0, 0.0, 0.0] },
            PulseDataType::Array => PulseGraphValueType::Array { array_type: PulseDataType::Any},
            PulseDataType::QAngle => PulseGraphValueType::QAngle { value: Vec3::default() },
            PulseDataType::Transform => PulseGraphValueType::Transform,
            PulseDataType::TransformWorldspace => PulseGraphValueType::TransformWorldspace,
            PulseDataType::Resource => PulseGraphValueType::Resource { resource_type: None, value: String::new() },
            PulseDataType::GameTime => PulseGraphValueType::GameTime,
            PulseDataType::TypeSafeInteger => PulseGraphValueType::TypeSafeInteger { integer_type: String::new() },
            PulseDataType::SoundEventName => PulseGraphValueType::SoundEventName { value: String::new() },
            PulseDataType::SndEventHandle => PulseGraphValueType::SndEventHandle,
            PulseDataType::SchemaEnum => PulseGraphValueType::SchemaEnumChoice { 
                enum_type: EnumBindingIndex::default(), enum_variant: EnumBindingValueIndex::default() 
            },
            _ => PulseGraphValueType::Any,
        }
    }
}

pub fn pulse_value_type_to_node_types(
    typ: &PulseValueType,
) -> (PulseDataType, PulseGraphValueType) {
    match typ {
        PulseValueType::PVAL_INT(val) => (
            PulseDataType::Scalar,
            PulseGraphValueType::Scalar {
                value: val.map(|v| v as f32).unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_FLOAT(val) => (
            PulseDataType::Scalar,
            PulseGraphValueType::Scalar {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_VEC3(val) => (
            PulseDataType::Vec3,
            PulseGraphValueType::Vec3 {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_VEC3_LOCAL(val) => (
            PulseDataType::Vec3Local,
            PulseGraphValueType::Vec3Local {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_STRING(val) => (
            PulseDataType::String,
            PulseGraphValueType::String {
                value: val.clone().unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_BOOL => (
            PulseDataType::Bool,
            PulseGraphValueType::Bool { value: false },
        ),
        PulseValueType::PVAL_BOOL_VALUE(val) => (
            PulseDataType::Bool,
            PulseGraphValueType::Bool { value: val.unwrap_or_default() },
        ),
        PulseValueType::PVAL_EHANDLE(_) => (PulseDataType::EHandle, PulseGraphValueType::EHandle),
        PulseValueType::PVAL_COLOR_RGB(val) => (
            PulseDataType::Color,
            PulseGraphValueType::Color {
                value: val
                    .map(|v| [v.x, v.y, v.z, 0.0])
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]),
            },
        ),
        PulseValueType::PVAL_SNDEVT_GUID(_) => (
            PulseDataType::SndEventHandle,
            PulseGraphValueType::SndEventHandle,
        ),
        PulseValueType::PVAL_SNDEVT_NAME(val) => (
            PulseDataType::SoundEventName,
            PulseGraphValueType::SoundEventName {
                value: val.clone().unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
            (
                PulseDataType::SchemaEnum,
                PulseGraphValueType::SchemaEnum {
                    enum_type: *enum_type,
                    value: enum_type
                        .get_all_types_as_enums()
                        .into_iter()
                        .next()
                        .expect("Schema enum variants list must not be empty"),
                },
            )
        }
        PulseValueType::DOMAIN_ENTITY_NAME => (
            PulseDataType::EntityName,
            PulseGraphValueType::EntityName {
                value: String::default(),
            },
        ),
        PulseValueType::PVAL_ACT => (PulseDataType::Action, PulseGraphValueType::Action),
        PulseValueType::PVAL_ANY => (PulseDataType::Any, PulseGraphValueType::Any),
        PulseValueType::PVAL_SCHEMA_ENUM_CHOICE(enum_binding) => {
            (
                PulseDataType::SchemaEnum,
                PulseGraphValueType::SchemaEnumChoice { 
                    enum_type: enum_binding.id,
                    enum_variant: EnumBindingValueIndex::default()
                },
            )
        }
        PulseValueType::PVAL_VEC2(val) => (
            PulseDataType::Vec2,
            PulseGraphValueType::Vec2 {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_VEC4(val) => (
            PulseDataType::Vec4,
            PulseGraphValueType::Vec4 {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_QANGLE(val) => (
            PulseDataType::QAngle,
            PulseGraphValueType::QAngle {
                value: val.unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_TRANSFORM(_) => (
            PulseDataType::Transform,
            PulseGraphValueType::Transform,
        ),
        PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => (
            PulseDataType::TransformWorldspace,
            PulseGraphValueType::TransformWorldspace,
        ),
        PulseValueType::PVAL_RESOURCE(resource_type, val) => (
            PulseDataType::Resource,
            PulseGraphValueType::Resource {
                resource_type: resource_type.clone(),
                value: val.clone().unwrap_or_default(),
            },
        ),
        PulseValueType::PVAL_ARRAY(array_type) =>
        (
            PulseDataType::Array,
            PulseGraphValueType::Array {
                array_type: PulseDataType::from((**array_type).clone()),
            }
        ),
        PulseValueType::PVAL_GAMETIME(_) => (
            PulseDataType::GameTime,
            PulseGraphValueType::GameTime,
        ),
        PulseValueType::PVAL_TYPESAFE_INT(int_type, _) => (
            PulseDataType::TypeSafeInteger,
            PulseGraphValueType::TypeSafeInteger {
                integer_type: int_type.clone().unwrap_or_default(),
            }
        ),
        _ => (
            PulseDataType::Any,
            PulseGraphValueType::Any
        )
    }
}

impl From<PulseValueType> for PulseGraphValueType {
    fn from(value: PulseValueType) -> Self {
        pulse_value_type_to_node_types(&value).1
    }
}

impl From<PulseValueType> for PulseDataType {
    fn from(value: PulseValueType) -> Self {
        pulse_value_type_to_node_types(&value).0
    }
}
