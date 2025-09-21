use std::{fmt, fmt::Display, str::FromStr};
use serde::{Deserialize, Serialize};
use egui_node_graph2::InputParamKind;
use crate::compiler::serialization::PulseConstant;
use crate::pulsetypes::{SchemaEnumType, SchemaEnumValue};
use crate::app::types::{PulseDataType, PulseGraphValueType};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PulseTypeError {
    StringToEnumConversionMissing(String),
    StringToEnumSubtypeParseError(String),
}
impl fmt::Display for PulseTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseTypeError::StringToEnumConversionMissing(s) => {
                write!(f, "Could not get enumerator from name: '{s}'")
            }
            PulseTypeError::StringToEnumSubtypeParseError(s) => {
                write!(f, "Could not parse subtype from string: '{s}'")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibraryBindingIndex(pub usize);
impl Display for LibraryBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LibraryBindingIndex({})", self.0)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventBindingIndex(pub usize);
impl Display for EventBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookBindingIndex(pub usize);
impl Display for HookBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HookBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[allow(non_camel_case_types)]
pub enum PulseValueType {
    PVAL_INT(Option<i32>),
    PVAL_TYPESAFE_INT(Option<String>, Option<i32>),
    PVAL_FLOAT(Option<f32>),
    PVAL_STRING(Option<String>),
    #[default]
    PVAL_INVALID,
    PVAL_EHANDLE(Option<String>),
    PVAL_VEC3(Option<Vec3>),
    PVAL_VEC3_LOCAL(Option<Vec3>),
    PVAL_COLOR_RGB(Option<Vec3>),
    PVAL_SNDEVT_GUID(Option<String>),
    PVAL_SNDEVT_NAME(Option<String>),
    PVAL_BOOL,
    PVAL_BOOL_VALUE(Option<bool>),
    DOMAIN_ENTITY_NAME,
    PVAL_ACT, // only used in the editor, not in the engine
    PVAL_ANY,
    PVAL_SCHEMA_ENUM(SchemaEnumType),
    PVAL_VEC2(Option<Vec2>),
    PVAL_VEC4(Option<Vec4>),
    PVAL_QANGLE(Option<Vec3>),
    PVAL_TRANSFORM(Option<Transform>),
    PVAL_TRANSFORM_WORLDSPACE(Option<Transform>),
    PVAL_RESOURCE(Option<String>, Option<String>), // (resource_type, resource_name)
    PVAL_ARRAY(Box<PulseValueType>),
    PVAL_GAMETIME(Option<f32>),
    PVAL_VOID,
}

impl fmt::Display for PulseValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseValueType::PVAL_INT(_) => write!(f, "PVAL_INT"),
            PulseValueType::PVAL_TYPESAFE_INT(int_type, _) => {
                if let Some(int_type) = int_type {
                    write!(f, "PVAL_TYPESAFE_INT:{int_type}")
                } else {
                    write!(f, "PVAL_TYPESAFE_INT")
                }
            }
            PulseValueType::PVAL_FLOAT(_) => write!(f, "PVAL_FLOAT"),
            PulseValueType::PVAL_STRING(_) => write!(f, "PVAL_STRING"),
            PulseValueType::PVAL_INVALID => write!(f, "PVAL_INVALID"),
            PulseValueType::DOMAIN_ENTITY_NAME => write!(f, "PVAL_ENTITY_NAME"),
            PulseValueType::PVAL_EHANDLE(ent_type) => {
                if let Some(ent_type) = ent_type {
                    write!(f, "PVAL_EHANDLE:{ent_type}")
                } else {
                    write!(f, "PVAL_EHANDLE")
                }
            }
            PulseValueType::PVAL_VEC3(_) => write!(f, "PVAL_VEC3_WORLDSPACE"),
            PulseValueType::PVAL_VEC3_LOCAL(_) => write!(f, "PVAL_VEC3"),
            PulseValueType::PVAL_COLOR_RGB(_) => write!(f, "PVAL_COLOR_RGB"),
            PulseValueType::PVAL_BOOL => write!(f, "PVAL_BOOL"),
            PulseValueType::PVAL_BOOL_VALUE(_) => write!(f, "PVAL_BOOL"),
            PulseValueType::PVAL_SNDEVT_GUID(_) => write!(f, "PVAL_SNDEVT_GUID"),
            PulseValueType::PVAL_SNDEVT_NAME(_) => write!(f, "PVAL_SNDEVT_NAME"),
            PulseValueType::PVAL_ACT => write!(f, "PVAL_ACT"),
            PulseValueType::PVAL_ANY => write!(f, "PVAL_VARIANT"),
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
                write!(f, "PVAL_SCHEMA_ENUM:{}", enum_type.to_str())
            }
            PulseValueType::PVAL_VEC2(_) => write!(f, "PVAL_VEC2"),
            PulseValueType::PVAL_VEC4(_) => write!(f, "PVAL_VEC4"),
            PulseValueType::PVAL_QANGLE(_) => write!(f, "PVAL_QANGLE"),
            PulseValueType::PVAL_TRANSFORM(_) => write!(f, "PVAL_TRANSFORM"),
            PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => write!(f, "PVAL_TRANSFORM_WORLDSPACE"),
            PulseValueType::PVAL_RESOURCE(resource_type, _) => {
                match resource_type.as_deref() {
                    Some(rt) if !rt.is_empty() => write!(f, "PVAL_RESOURCE:{rt}"),
                    _ => write!(f, "PVAL_RESOURCE"),
                }
            }
            PulseValueType::PVAL_ARRAY(arr_type) => {
                write!(f, "PVAL_ARRAY:{arr_type}")
            }
            PulseValueType::PVAL_GAMETIME(_) => write!(f, "PVAL_GAMETIME"),
            PulseValueType::PVAL_VOID => write!(f, "PVAL_VOID"),
        }
    }
}

impl PulseValueType {
    // defines the suffix for the operation name used in instructions eg. EQ_STRING, ADD_INT
    pub fn get_operation_suffix_name(&self) -> &'static str {
        match self {
            PulseValueType::PVAL_BOOL => "_BOOL",
            PulseValueType::PVAL_INT(_) => "_INT",
            PulseValueType::PVAL_FLOAT(_) => "_FLOAT",
            PulseValueType::PVAL_STRING(_) => "_STRING",
            PulseValueType::PVAL_VEC2(_) => "_VEC2",
            PulseValueType::PVAL_VEC3(_)
            | PulseValueType::PVAL_VEC3_LOCAL(_) => "_VEC3",
            PulseValueType::PVAL_VEC4(_) => "_VEC4",
            PulseValueType::PVAL_EHANDLE(_) => "_EHANDLE",
            PulseValueType::DOMAIN_ENTITY_NAME => "_ENTITY_NAME",
            PulseValueType::PVAL_SCHEMA_ENUM(_) => "_SCHEMA_ENUM",
            PulseValueType::PVAL_COLOR_RGB(_) => "_COLOR_RGB",
            PulseValueType::PVAL_ARRAY(_) => "_ARRAY",
            PulseValueType::PVAL_GAMETIME(_) => "_GAMETIME",
            _ => "",
        }
    }
    pub fn get_ui_name(&self) -> &'static str {
        match self {
            PulseValueType::PVAL_INT(_) => "Integer",
            PulseValueType::PVAL_TYPESAFE_INT(_, _) => "Typesafe Integer",
            PulseValueType::PVAL_FLOAT(_) => "Float",
            PulseValueType::PVAL_STRING(_) => "String",
            PulseValueType::PVAL_INVALID => "Invalid",
            PulseValueType::DOMAIN_ENTITY_NAME => "Entity Name",
            PulseValueType::PVAL_EHANDLE(_) => "Entity",
            PulseValueType::PVAL_VEC3(_) => "World Vector",
            PulseValueType::PVAL_VEC3_LOCAL(_) => "Local Vector",
            PulseValueType::PVAL_COLOR_RGB(_) => "Color RGB",
            PulseValueType::PVAL_BOOL | PulseValueType::PVAL_BOOL_VALUE(_) => "Boolean",
            PulseValueType::PVAL_SNDEVT_GUID(_) => "Sound Event",
            PulseValueType::PVAL_SNDEVT_NAME(_) => "Sound Event Name",
            PulseValueType::PVAL_ACT => "Action",
            PulseValueType::PVAL_ANY => "Any Type",
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => enum_type.to_str_ui(),
            PulseValueType::PVAL_VEC2(_) => "Vector 2D",
            PulseValueType::PVAL_VEC4(_) => "Vector 4D",
            PulseValueType::PVAL_QANGLE(_) => "QAngle",
            PulseValueType::PVAL_TRANSFORM(_) => "Transform",
            PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => "World Transform",
            PulseValueType::PVAL_RESOURCE(_, _) => "Resource",
            PulseValueType::PVAL_ARRAY(_) => "Array",
            PulseValueType::PVAL_GAMETIME(_) => "Game Time",
            PulseValueType::PVAL_VOID => "Void",
        }
    }
    pub fn get_comparable_types() -> Vec<PulseValueType> {
        vec![
            PulseValueType::PVAL_INT(None),
            PulseValueType::PVAL_FLOAT(None),
            PulseValueType::PVAL_STRING(None),
            PulseValueType::PVAL_BOOL,
            PulseValueType::PVAL_EHANDLE(None),
            PulseValueType::DOMAIN_ENTITY_NAME,
            PulseValueType::PVAL_VEC2(None),
            PulseValueType::PVAL_VEC3(None),
            PulseValueType::PVAL_VEC3_LOCAL(None),
            PulseValueType::PVAL_VEC4(None),
            PulseValueType::PVAL_COLOR_RGB(None),
            PulseValueType::PVAL_ARRAY(Box::new(PulseValueType::PVAL_ANY)),
            PulseValueType::PVAL_QANGLE(None), // it doesn't have it's own suffix, but maybe it works.
            PulseValueType::PVAL_GAMETIME(None),
        ]
    }
    pub fn get_operatable_types() -> Vec<PulseValueType> {
        vec![
            PulseValueType::PVAL_INT(None),
            PulseValueType::PVAL_FLOAT(None),
            PulseValueType::PVAL_STRING(None),
            PulseValueType::PVAL_VEC2(None),
            PulseValueType::PVAL_VEC3(None),
            PulseValueType::PVAL_VEC3_LOCAL(None),
            PulseValueType::PVAL_VEC4(None),
        ]
    }
    pub fn get_scalable_types() -> Vec<PulseValueType> {
        vec![
            PulseValueType::PVAL_VEC2(None),
            PulseValueType::PVAL_VEC3(None),
            PulseValueType::PVAL_VEC3_LOCAL(None),
            PulseValueType::PVAL_VEC4(None),
        ]
    }
    pub fn get_variable_supported_types() -> Vec<PulseValueType> {
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
            PulseValueType::PVAL_ARRAY(Box::new(PulseValueType::PVAL_ANY)),
            PulseValueType::PVAL_RESOURCE(None, None),
            PulseValueType::PVAL_GAMETIME(None),
        ]
    }
    pub fn get_vector_types() -> Vec<PulseValueType> {
        vec![
            PulseValueType::PVAL_VEC2(None),
            PulseValueType::PVAL_VEC3(None),
            PulseValueType::PVAL_VEC3_LOCAL(None),
            PulseValueType::PVAL_VEC4(None),
        ]
    }
}

pub fn try_string_to_pulsevalue(s: &str) -> Result<PulseValueType, PulseTypeError> {
    match s {
        "PVAL_INT" => Ok(PulseValueType::PVAL_INT(None)),
        "PVAL_FLOAT" => Ok(PulseValueType::PVAL_FLOAT(None)),
        "PVAL_BOOL" => Ok(PulseValueType::PVAL_BOOL),
        "PVAL_STRING" => Ok(PulseValueType::PVAL_STRING(None)),
        "PVAL_EHANDLE" => Ok(PulseValueType::PVAL_EHANDLE(None)),
        "PVAL_VEC3_WORLDSPACE" => Ok(PulseValueType::PVAL_VEC3(None)),
        "PVAL_VEC3" => Ok(PulseValueType::PVAL_VEC3_LOCAL(None)),
        "PVAL_COLOR_RGB" => Ok(PulseValueType::PVAL_COLOR_RGB(None)),
        "PVAL_INVALID" => Ok(PulseValueType::PVAL_INVALID),
        "PVAL_SNDEVT_GUID" => Ok(PulseValueType::PVAL_SNDEVT_GUID(None)),
        "PVAL_ENTITY_NAME" => Ok(PulseValueType::DOMAIN_ENTITY_NAME),
        "PVAL_SNDEVT_NAME" => Ok(PulseValueType::PVAL_SNDEVT_NAME(None)),
        "PVAL_ACT" => Ok(PulseValueType::PVAL_ACT),
        "PVAL_ANY" | "PVAL_VARIANT" => Ok(PulseValueType::PVAL_ANY),
        "PVAL_VEC2" => Ok(PulseValueType::PVAL_VEC2(None)),
        "PVAL_VEC4" => Ok(PulseValueType::PVAL_VEC4(None)),
        "PVAL_QANGLE" => Ok(PulseValueType::PVAL_QANGLE(None)),
        "PVAL_TRANSFORM" => Ok(PulseValueType::PVAL_TRANSFORM(None)),
        "PVAL_TRANSFORM_WORLDSPACE" => Ok(PulseValueType::PVAL_TRANSFORM_WORLDSPACE(None)),
        "PVAL_RESOURCE" => Ok(PulseValueType::PVAL_RESOURCE(None, None)),
        "PVAL_ARRAY" => Ok(PulseValueType::PVAL_ARRAY(Box::new(PulseValueType::PVAL_ANY))),
        "PVAL_GAMETIME" => Ok(PulseValueType::PVAL_GAMETIME(None)),
        "PVAL_VOID" => Ok(PulseValueType::PVAL_VOID),
        _ => {
            if s.starts_with("PVAL_EHANDLE:") {
                let ent_type = s.split_at(13).1;
                Ok(PulseValueType::PVAL_EHANDLE(Some(ent_type.to_string())))
            } else if s.starts_with("PVAL_SCHEMA_ENUM:") {
                let enum_type = s.split_at(17).1;
                let en = SchemaEnumType::from_str(enum_type)
                    .map_err(|_| PulseTypeError::StringToEnumConversionMissing(enum_type.to_string()))?;
                Ok(PulseValueType::PVAL_SCHEMA_ENUM(en))
            } else if s.starts_with("PVAL_RESOURCE:") {
                let res_type = s.split_at(14).1;
                Ok(PulseValueType::PVAL_RESOURCE(Some(res_type.to_string()), None))
            } else if s.starts_with("PVAL_TYPESAFE_INT:") {
                let int_type = s.split_at(18).1;
                Ok(PulseValueType::PVAL_TYPESAFE_INT(Some(int_type.to_string()), None))
            } else if s.starts_with("PVAL_ARRAY:") {
                let arr_type = s.split_at(11).1;
                Ok(PulseValueType::PVAL_ARRAY(Box::new(
                    try_string_to_pulsevalue(arr_type).unwrap_or(PulseValueType::PVAL_ANY)
                )))
            } else {
                Err(PulseTypeError::StringToEnumConversionMissing(s.to_string()))
            }
        }
    }
}

pub fn data_type_to_value_type(typ: &PulseDataType) -> PulseGraphValueType {
    match typ {
        PulseDataType::Scalar => PulseGraphValueType::Scalar { value: 0f32 },
        PulseDataType::String => PulseGraphValueType::String {
            value: String::default(),
        },
        PulseDataType::Vec3 => PulseGraphValueType::Vec3 {
            value: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        PulseDataType::Vec3Local => PulseGraphValueType::Vec3Local {
            value: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        PulseDataType::EHandle => PulseGraphValueType::EHandle,
        PulseDataType::Bool => PulseGraphValueType::Bool { value: false },
        PulseDataType::SndEventHandle => PulseGraphValueType::SndEventHandle,
        PulseDataType::EntityName => PulseGraphValueType::EntityName {
            value: String::default(),
        },
        PulseDataType::Action => PulseGraphValueType::Action,
        PulseDataType::SoundEventName => PulseGraphValueType::SoundEventName {
            value: String::default(),
        },
        PulseDataType::Color => PulseGraphValueType::Color {
            value: [0.0, 0.0, 0.0, 0.0],
        },
        PulseDataType::Any => PulseGraphValueType::Any,
        _ => PulseGraphValueType::Scalar { value: 0f32 },
    }
}

pub fn pulse_value_type_to_node_types(
    typ: &PulseValueType,
) -> (PulseDataType, PulseGraphValueType) {
    match typ {
        PulseValueType::PVAL_INT(_)
        | PulseValueType::PVAL_FLOAT(_) => (
            PulseDataType::Scalar,
            PulseGraphValueType::Scalar { value: 0f32 },
        ),
        PulseValueType::PVAL_VEC3(_) => (
            PulseDataType::Vec3,
            PulseGraphValueType::Vec3 {
                value: Vec3::default(),
            },
        ),
        PulseValueType::PVAL_VEC3_LOCAL(_) => (
            PulseDataType::Vec3Local,
            PulseGraphValueType::Vec3Local {
                value: Vec3::default(),
            },
        ),
        PulseValueType::PVAL_STRING(_) => (
            PulseDataType::String,
            PulseGraphValueType::String {
                value: String::default(),
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
        PulseValueType::PVAL_COLOR_RGB(_) => (
            PulseDataType::Color,
            PulseGraphValueType::Color {
                value: [0.0, 0.0, 0.0, 0.0],
            },
        ),
        PulseValueType::PVAL_SNDEVT_GUID(_) => (
            PulseDataType::SndEventHandle,
            PulseGraphValueType::SndEventHandle,
        ),
        PulseValueType::PVAL_SNDEVT_NAME(_) => (
            PulseDataType::SoundEventName,
            PulseGraphValueType::SoundEventName {
                value: String::default(),
            },
        ),
        PulseValueType::DOMAIN_ENTITY_NAME => (
            PulseDataType::EntityName,
            PulseGraphValueType::EntityName {
                value: String::default(),
            },
        ),
        PulseValueType::PVAL_ACT => (PulseDataType::Action, PulseGraphValueType::Action),
        PulseValueType::PVAL_ANY => (PulseDataType::Any, PulseGraphValueType::Any),
        PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
            let val = SchemaEnumValue::default_from_type(enum_type);
            (
                PulseDataType::SchemaEnum,
                PulseGraphValueType::SchemaEnum {
                    enum_type: *enum_type,
                    value: val,
                }
            )
        }
        PulseValueType::PVAL_VEC2(_) => (
            PulseDataType::Vec2,
            PulseGraphValueType::Vec2 {
                value: Vec2::default(),
            },
        ),
        PulseValueType::PVAL_VEC4(_) => (
            PulseDataType::Vec4,
            PulseGraphValueType::Vec4 {
                value: Vec4::default(),
            },
        ),
        PulseValueType::PVAL_QANGLE(_) => (
            PulseDataType::QAngle,
            PulseGraphValueType::QAngle {
                value: Vec3::default(),
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
        PulseValueType::PVAL_RESOURCE(resource_type, _) => (
            PulseDataType::Resource,
            PulseGraphValueType::Resource {
                resource_type: resource_type.clone(),
                value: String::default(),
            },
        ),
        PulseValueType::PVAL_ARRAY(_) =>
        (
            PulseDataType::Array,
            PulseGraphValueType::Array
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
        _ => todo!("Implement more type conversions"),
    }
}

pub fn get_preffered_inputparamkind_from_type(typ: &PulseValueType) -> InputParamKind {
    match typ {
        PulseValueType::PVAL_INT(_)
        | PulseValueType::PVAL_TYPESAFE_INT(_, _)
        | PulseValueType::PVAL_FLOAT(_)
        | PulseValueType::PVAL_STRING(_)
        | PulseValueType::PVAL_VEC3(_)
        | PulseValueType::PVAL_VEC3_LOCAL(_)
        | PulseValueType::PVAL_VEC2(_)
        | PulseValueType::PVAL_VEC4(_)
        | PulseValueType::PVAL_QANGLE(_)
        | PulseValueType::PVAL_TRANSFORM(_)
        | PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_)
        | PulseValueType::DOMAIN_ENTITY_NAME
        | PulseValueType::PVAL_COLOR_RGB(_)
        | PulseValueType::PVAL_SNDEVT_NAME(_)
        | PulseValueType::PVAL_RESOURCE(_, _)
        | PulseValueType::PVAL_GAMETIME(_) => InputParamKind::ConnectionOrConstant,

        PulseValueType::PVAL_EHANDLE(_)
        | PulseValueType::PVAL_SNDEVT_GUID(_)
        | PulseValueType::PVAL_INVALID
        | PulseValueType::PVAL_ACT
        | PulseValueType::PVAL_ANY
        | PulseValueType::PVAL_ARRAY(_)
        | PulseValueType::PVAL_VOID => InputParamKind::ConnectionOnly,

        PulseValueType::PVAL_BOOL
        | PulseValueType::PVAL_BOOL_VALUE(_)
        | PulseValueType::PVAL_SCHEMA_ENUM(_) => InputParamKind::ConstantOnly,
    }
}

pub fn get_pulse_constant_from_graph_value(typ: PulseGraphValueType) -> anyhow::Result<PulseConstant> {
    match typ {
        PulseGraphValueType::Scalar { value } => Ok(PulseConstant::Float(value)),
        PulseGraphValueType::String { value } => Ok(PulseConstant::String(value)),
        PulseGraphValueType::Vec3 { value } => Ok(PulseConstant::Vec3(value)),
        PulseGraphValueType::Vec3Local { value } => Ok(PulseConstant::Vec3Local(value)),
        PulseGraphValueType::Color { value } => Ok(PulseConstant::Color_RGB(value)),
        PulseGraphValueType::Bool { value } => Ok(PulseConstant::Bool(value)),
        PulseGraphValueType::SoundEventName { value } => Ok(PulseConstant::SoundEventName(value)),
        PulseGraphValueType::SchemaEnum { enum_type, value } => {
            Ok(PulseConstant::SchemaEnum(enum_type, value))
        }
        PulseGraphValueType::Vec2 { value } => Ok(PulseConstant::Vec2(value)),
        PulseGraphValueType::Vec4 { value } => Ok(PulseConstant::Vec4(value)),
        PulseGraphValueType::QAngle { value } => Ok(PulseConstant::QAngle(value)),
        PulseGraphValueType::Resource {
            resource_type,
            value,
        } => Ok(PulseConstant::Resource(resource_type, value)),
        _ => Err(anyhow::anyhow!("Unsupported constant value type for {:?}", typ)),
    }
}
