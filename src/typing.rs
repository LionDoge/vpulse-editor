use crate::app::{PulseDataType, PulseGraphValueType};
use crate::pulsetypes::{SchemaEnumType, SchemaEnumValue};
use egui_node_graph2::InputParamKind;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PulseTypeError {
    StringToEnumConversionMissing(String),
    StringToEnumSubtypeParseError(String),
}
impl fmt::Display for PulseTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseTypeError::StringToEnumConversionMissing(s) => {
                write!(f, "Could not convert string to enum: {s}")
            }
            PulseTypeError::StringToEnumSubtypeParseError(s) => {
                write!(f, "Could not parse subtype from string: {s}")
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
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
    PVAL_COLOR_RGB(Option<Vec3>),
    PVAL_SNDEVT_GUID(Option<String>),
    PVAL_SNDEVT_NAME(Option<String>),
    PVAL_BOOL(Option<bool>),
    DOMAIN_ENTITY_NAME,
    PVAL_ACT, // only used in the editor, not in the engine
    PVAL_ANY,
    PVAL_SCHEMA_ENUM(SchemaEnumType)
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
            PulseValueType::PVAL_VEC3(_) => write!(f, "PVAL_VEC3"),
            PulseValueType::PVAL_COLOR_RGB(_) => write!(f, "PVAL_COLOR_RGB"),
            PulseValueType::PVAL_BOOL(_) => write!(f, "PVAL_BOOL"),
            PulseValueType::PVAL_SNDEVT_GUID(_) => write!(f, "PVAL_SNDEVT_GUID"),
            PulseValueType::PVAL_SNDEVT_NAME(_) => write!(f, "PVAL_SNDEVT_NAME"),
            PulseValueType::PVAL_ACT => write!(f, "PVAL_ACT"),
            PulseValueType::PVAL_ANY => write!(f, "PVAL_ANY"),
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
                write!(f, "PVAL_SCHEMA_ENUM:{}", enum_type.to_str())
            }
        }
    }
}
impl PulseValueType {
    pub fn get_operation_suffix_name(&self) -> &'static str {
        match self {
            PulseValueType::PVAL_FLOAT(_) => "_FLOAT",
            PulseValueType::PVAL_INT(_) => "_INT",
            PulseValueType::PVAL_VEC3(_) => "", // Vec3 uses generic comparison (I think)
            PulseValueType::PVAL_EHANDLE(_) => "_EHANDLE",
            PulseValueType::PVAL_STRING(_) => "_STRING",
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
            PulseValueType::PVAL_VEC3(_) => "Vector 3D",
            PulseValueType::PVAL_COLOR_RGB(_) => "Color RGB",
            PulseValueType::PVAL_BOOL(_) => "Boolean",
            PulseValueType::PVAL_SNDEVT_GUID(_) => "Sound Event",
            PulseValueType::PVAL_SNDEVT_NAME(_) => "Sound Event Name",
            PulseValueType::PVAL_ACT => "Action",
            PulseValueType::PVAL_ANY => "Any Type",
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => enum_type.to_str_ui(),
        }
    }
}

pub fn try_string_to_pulsevalue(s: &str) -> Result<PulseValueType, PulseTypeError> {
    match s {
        "PVAL_INT" | "PVAL_TYPESAFE_INT" => Ok(PulseValueType::PVAL_INT(None)),
        "PVAL_FLOAT" => Ok(PulseValueType::PVAL_FLOAT(None)),
        "PVAL_BOOL" => Ok(PulseValueType::PVAL_BOOL(None)),
        "PVAL_STRING" => Ok(PulseValueType::PVAL_STRING(None)),
        "PVAL_EHANDLE" => Ok(PulseValueType::PVAL_EHANDLE(None)),
        "PVAL_VEC3" => Ok(PulseValueType::PVAL_VEC3(None)),
        "PVAL_COLOR_RGB" => Ok(PulseValueType::PVAL_COLOR_RGB(None)),
        "PVAL_INVALID" => Ok(PulseValueType::PVAL_INVALID),
        "PVAL_SNDEVT_GUID" => Ok(PulseValueType::PVAL_SNDEVT_GUID(None)),
        "PVAL_ENTITY_NAME" => Ok(PulseValueType::DOMAIN_ENTITY_NAME),
        "PVAL_SNDEVT_NAME" => Ok(PulseValueType::PVAL_SNDEVT_NAME(None)),
        "PVAL_ACT" => Ok(PulseValueType::PVAL_ACT),
        "PVAL_ANY" => Ok(PulseValueType::PVAL_ANY),
        _ => {
            if s.starts_with("PVAL_EHANDLE:") {
                let ent_type = s.split_at(13).1;
                Ok(PulseValueType::PVAL_EHANDLE(Some(ent_type.to_string())))
            } else if s.starts_with("PVAL_SCHEMA_ENUM:") {
                let enum_type = s.split_at(17).1;
                let en = SchemaEnumType::from_str(enum_type)
                    .map_err(|_| PulseTypeError::StringToEnumConversionMissing(enum_type.to_string()))?;
                Ok(PulseValueType::PVAL_SCHEMA_ENUM(en))
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
        | PulseValueType::PVAL_FLOAT(_)
        | PulseValueType::PVAL_TYPESAFE_INT(_, _) => (
            PulseDataType::Scalar,
            PulseGraphValueType::Scalar { value: 0f32 },
        ),
        PulseValueType::PVAL_VEC3(_) => (
            PulseDataType::Vec3,
            PulseGraphValueType::Vec3 {
                value: Vec3::default(),
            },
        ),
        PulseValueType::PVAL_STRING(_) => (
            PulseDataType::String,
            PulseGraphValueType::String {
                value: String::default(),
            },
        ),
        PulseValueType::PVAL_BOOL(_) => (
            PulseDataType::Bool,
            PulseGraphValueType::Bool { value: false },
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
        | PulseValueType::DOMAIN_ENTITY_NAME
        | PulseValueType::PVAL_COLOR_RGB(_)
        | PulseValueType::PVAL_SNDEVT_NAME(_) => InputParamKind::ConnectionOrConstant,

        PulseValueType::PVAL_EHANDLE(_)
        | PulseValueType::PVAL_SNDEVT_GUID(_)
        | PulseValueType::PVAL_INVALID
        | PulseValueType::PVAL_ACT
        | PulseValueType::PVAL_ANY => InputParamKind::ConnectionOnly,

        PulseValueType::PVAL_BOOL(_)
        | PulseValueType::PVAL_SCHEMA_ENUM(_) => InputParamKind::ConstantOnly,
    }
}
