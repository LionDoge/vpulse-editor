use std::fmt;
use egui_node_graph2::InputParamKind;
use serde::{Deserialize, Serialize};
use anyhow::Error;
use crate::app::{PulseDataType, PulseGraphValueType};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PulseTypeError {
    StringToEnumConversionMissing(String),
    StringToEnumSubtypeParseError(String),
}
impl fmt::Display for PulseTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseTypeError::StringToEnumConversionMissing(s) => 
                write!(f, "Could not convert string to enum: {}", s),
            PulseTypeError::StringToEnumSubtypeParseError(s) => 
                write!(f, "Could not parse subtype from string: {}", s),
        }
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PulseValueType {
    PVAL_INT(Option<i32>),
    PVAL_TYPESAFE_INT(Option<String>, Option<i32>),
    PVAL_FLOAT(Option<f32>),
    PVAL_STRING(Option<String>),
    PVAL_INVALID,
    PVAL_EHANDLE(Option<String>),
    PVAL_VEC3(Option<Vec3>),
    PVAL_COLOR_RGB(Option<Vec3>),
    PVAL_SNDEVT_GUID(Option<String>),
    PVAL_BOOL,
    DOMAIN_ENTITY_NAME,
}
impl Default for PulseValueType {
    fn default() -> Self {
        PulseValueType::PVAL_INVALID
    }
}
impl fmt::Display for PulseValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseValueType::PVAL_INT(_) => write!(f, "PVAL_INT"),
            PulseValueType::PVAL_TYPESAFE_INT(int_type, _) => write!(f, "PVAL_TYPESAFE_INT:{}", int_type.clone().unwrap_or_default()),
            PulseValueType::PVAL_FLOAT(_) => write!(f, "PVAL_FLOAT"),
            PulseValueType::PVAL_STRING(_) => write!(f, "PVAL_STRING"),
            PulseValueType::PVAL_INVALID => write!(f, "PVAL_INVALID"),
            PulseValueType::DOMAIN_ENTITY_NAME => write!(f, "PVAL_ENTITY_NAME"),
            PulseValueType::PVAL_EHANDLE(ent_type) => {
                if let Some(ent_type) = ent_type {
                    write!(f, "PVAL_EHANDLE:{}", ent_type)
                } else {
                    write!(f, "PVAL_EHANDLE")
                }
            },
            PulseValueType::PVAL_VEC3(_) => write!(f, "PVAL_VEC3"),
            PulseValueType::PVAL_COLOR_RGB(_) => write!(f, "PVAL_COLOR_RGB"),
            PulseValueType::PVAL_BOOL => write!(f, "PVAL_BOOL"),
            PulseValueType::PVAL_SNDEVT_GUID(_) => write!(f, "PVAL_SNDEVT_GUID"),
        }
    }
}
impl PulseValueType {
    pub fn get_operation_suffix_name(&self) -> &'static str {
        return match self {
            PulseValueType::PVAL_FLOAT(_) => "_FLOAT",
            PulseValueType::PVAL_INT(_) => "_INT",
            PulseValueType::PVAL_VEC3(_) => "", // Vec3 uses generic comparison (I think)
            PulseValueType::PVAL_EHANDLE(_) => "_EHANDLE",
            PulseValueType::PVAL_STRING(_) => "_STRING",
            _ => ""
        }
    }
}

pub fn try_string_to_pulsevalue(s: &str) -> Result<PulseValueType, PulseTypeError> {
    match s {
        "PVAL_INT" | "PVAL_TYPESAFE_INT" => Ok(PulseValueType::PVAL_INT(None)),
        "PVAL_FLOAT" => Ok(PulseValueType::PVAL_FLOAT(None)),
        "PVAL_BOOL" => Ok(PulseValueType::PVAL_BOOL),
        "PVAL_STRING" => Ok(PulseValueType::PVAL_STRING(None)),
        "PVAL_EHANDLE" => Ok(PulseValueType::PVAL_EHANDLE(None)),
        "PVAL_VEC3" => Ok(PulseValueType::PVAL_VEC3(None)),
        "PVAL_COLOR_RGB" => Ok(PulseValueType::PVAL_COLOR_RGB(None)),
        "PVAL_INVALID" => Ok(PulseValueType::PVAL_INVALID),
        "PVAL_SNDEVT_GUID" => Ok(PulseValueType::PVAL_SNDEVT_GUID(None)),
        "PVAL_ENTITY_NAME" => Ok(PulseValueType::DOMAIN_ENTITY_NAME),
        _ => {
            if s.starts_with("PVAL_EHANDLE:") {
                let ent_type = s.split_at(13).1;
                Ok(PulseValueType::PVAL_EHANDLE(Some(ent_type.to_string())))
            } else {
                Err(PulseTypeError::StringToEnumConversionMissing(s.to_string()))
            }
        }
    }
}

pub fn data_type_to_value_type(typ: &PulseDataType) -> PulseGraphValueType {
    return match typ {
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
        _ => PulseGraphValueType::Scalar { value: 0f32 },
    };
}

pub fn pulse_value_type_to_node_types(typ: &PulseValueType) -> (PulseDataType, PulseGraphValueType) {
    match typ {
        PulseValueType::PVAL_INT(_) | PulseValueType::PVAL_FLOAT(_) | PulseValueType::PVAL_TYPESAFE_INT(_, _) => (
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
        PulseValueType::PVAL_BOOL => (PulseDataType::Bool, PulseGraphValueType::Bool { value: false }),
        PulseValueType::PVAL_EHANDLE(_) => (PulseDataType::EHandle, PulseGraphValueType::EHandle),
        PulseValueType::PVAL_COLOR_RGB(_) => (
            PulseDataType::Vec3,
            PulseGraphValueType::Vec3 {
                value: Vec3::default(),
            },
        ),
        PulseValueType::PVAL_SNDEVT_GUID(_) => (PulseDataType::SndEventHandle, PulseGraphValueType::SndEventHandle),
        PulseValueType::DOMAIN_ENTITY_NAME => (PulseDataType::EntityName, PulseGraphValueType::EntityName { value: String::default() }),
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
        | PulseValueType::PVAL_COLOR_RGB(_) => InputParamKind::ConnectionOrConstant,

        PulseValueType::PVAL_EHANDLE(_)
        | PulseValueType::PVAL_SNDEVT_GUID(_)
        | PulseValueType::PVAL_INVALID => InputParamKind::ConnectionOnly,

        PulseValueType::PVAL_BOOL => InputParamKind::ConstantOnly,
   }
}