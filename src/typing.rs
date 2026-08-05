use std::{fmt, fmt::Display, borrow::Cow};
use serde::{Deserialize, Serialize};
use egui_node_graph2::InputParamKind;
use crate::compiler::serialization::PulseConstant;
use crate::pulsetypes::SchemaEnumType;
use crate::app::types::PulseGraphValueType;
use crate::bindings::{EnumBindings, GraphBindings};

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
pub struct LibraryBindingIndex(pub u32);
impl Display for LibraryBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LibraryBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventBindingIndex(pub u32);
impl Display for EventBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookBindingIndex(pub u32);
impl Display for HookBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HookBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumBindingIndex(pub u32);
impl Display for EnumBindingIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnumBindingIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumBindingValueIndex(pub usize);
impl Display for EnumBindingValueIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnumBindingValueIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VariableIndex(pub usize);
impl Display for VariableIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VariableIndex({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PublicOutputIndex(pub usize);
impl Display for PublicOutputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicOutputIndex({})", self.0)
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
    PVAL_SCHEMA_ENUM_INDEXED(Option<EnumBindingIndex>, Option<EnumBindingValueIndex>),
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

// impl fmt::Display for PulseValueType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             PulseValueType::PVAL_INT(_) => write!(f, "PVAL_INT"),
//             PulseValueType::PVAL_TYPESAFE_INT(int_type, _) => {
//                 if let Some(int_type) = int_type {
//                     write!(f, "PVAL_TYPESAFE_INT:{int_type}")
//                 } else {
//                     write!(f, "PVAL_TYPESAFE_INT")
//                 }
//             }
//             PulseValueType::PVAL_FLOAT(_) => write!(f, "PVAL_FLOAT"),
//             PulseValueType::PVAL_STRING(_) => write!(f, "PVAL_STRING"),
//             PulseValueType::PVAL_INVALID => write!(f, "PVAL_INVALID"),
//             PulseValueType::DOMAIN_ENTITY_NAME => write!(f, "PVAL_ENTITY_NAME"),
//             PulseValueType::PVAL_EHANDLE(ent_type) => {
//                 if let Some(ent_type) = ent_type {
//                     write!(f, "PVAL_EHANDLE:{ent_type}")
//                 } else {
//                     write!(f, "PVAL_EHANDLE")
//                 }
//             }
//             PulseValueType::PVAL_VEC3(_) => write!(f, "PVAL_VEC3_WORLDSPACE"),
//             PulseValueType::PVAL_VEC3_LOCAL(_) => write!(f, "PVAL_VEC3"),
//             PulseValueType::PVAL_COLOR_RGB(_) => write!(f, "PVAL_COLOR_RGB"),
//             PulseValueType::PVAL_BOOL => write!(f, "PVAL_BOOL"),
//             PulseValueType::PVAL_BOOL_VALUE(_) => write!(f, "PVAL_BOOL"),
//             PulseValueType::PVAL_SNDEVT_GUID(_) => write!(f, "PVAL_SNDEVT_GUID"),
//             PulseValueType::PVAL_SNDEVT_NAME(_) => write!(f, "PVAL_SNDEVT_NAME"),
//             PulseValueType::PVAL_ACT => write!(f, "PVAL_ACT"),
//             PulseValueType::PVAL_ANY => write!(f, "PVAL_VARIANT"),
//             PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
//                 write!(f, "PVAL_SCHEMA_ENUM:{}", enum_type.to_str())
//             }
//             PulseValueType::PVAL_SCHEMA_ENUM_CHOICE(enum_binding) => {
//                 write!(f, "PVAL_SCHEMA_ENUM:{}", enum_binding.name)
//             }
//             PulseValueType::PVAL_VEC2(_) => write!(f, "PVAL_VEC2"),
//             PulseValueType::PVAL_VEC4(_) => write!(f, "PVAL_VEC4"),
//             PulseValueType::PVAL_QANGLE(_) => write!(f, "PVAL_QANGLE"),
//             PulseValueType::PVAL_TRANSFORM(_) => write!(f, "PVAL_TRANSFORM"),
//             PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => write!(f, "PVAL_TRANSFORM_WORLDSPACE"),
//             PulseValueType::PVAL_RESOURCE(resource_type, _) => {
//                 match resource_type.as_deref() {
//                     Some(rt) if !rt.is_empty() => write!(f, "PVAL_RESOURCE:{rt}"),
//                     _ => write!(f, "PVAL_RESOURCE"),
//                 }
//             }
//             PulseValueType::PVAL_ARRAY(arr_type) => {
//                 write!(f, "PVAL_ARRAY:{arr_type}")
//             }
//             PulseValueType::PVAL_GAMETIME(_) => write!(f, "PVAL_GAMETIME"),
//             PulseValueType::PVAL_VOID => write!(f, "PVAL_VOID"),
//         }
//     }
// }

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
    pub fn get_enum_string(&self, graph_bindings: &GraphBindings) -> Cow<'static, str> {
        match self {
            PulseValueType::PVAL_INT(_) => "PVAL_INT".into(),
            PulseValueType::PVAL_TYPESAFE_INT(int_type, _) => {
                if let Some(int_type) = int_type {
                    format!("PVAL_TYPESAFE_INT:{int_type}").into()
                } else {
                    "PVAL_TYPESAFE_INT".into()
                }
            }
            PulseValueType::PVAL_FLOAT(_) => "PVAL_FLOAT".into(),
            PulseValueType::PVAL_STRING(_) => "PVAL_STRING".into(),
            PulseValueType::PVAL_INVALID => "PVAL_INVALID".into(),
            PulseValueType::DOMAIN_ENTITY_NAME => "PVAL_ENTITY_NAME".into(),
            PulseValueType::PVAL_EHANDLE(ent_type) => {
                if let Some(ent_type) = ent_type {
                    format!("PVAL_EHANDLE:{ent_type}").into()
                } else {
                    "PVAL_EHANDLE".into()
                }
            }
            PulseValueType::PVAL_VEC3(_) => "PVAL_VEC3_WORLDSPACE".into(),
            PulseValueType::PVAL_VEC3_LOCAL(_) => "PVAL_VEC3".into(),
            PulseValueType::PVAL_COLOR_RGB(_) => "PVAL_COLOR_RGB".into(),
            PulseValueType::PVAL_BOOL => "PVAL_BOOL".into(),
            PulseValueType::PVAL_BOOL_VALUE(_) => "PVAL_BOOL".into(),
            PulseValueType::PVAL_SNDEVT_GUID(_) => "PVAL_SNDEVT_GUID".into(),
            PulseValueType::PVAL_SNDEVT_NAME(_) => "PVAL_SNDEVT_NAME".into(),
            PulseValueType::PVAL_ACT => "PVAL_ACT".into(),
            PulseValueType::PVAL_ANY => "PVAL_VARIANT".into(),
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => {
                format!("PVAL_SCHEMA_ENUM:{}", enum_type.to_str()).into()
            }
            PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(enum_type, _) => {
                match enum_type {
                    Some(enum_type) => {
                        if let Some(enum_binding) = graph_bindings.find_enum_by_id(*enum_type) {
                            format!("PVAL_SCHEMA_ENUM:{}", enum_binding.name).into()
                        } else {
                            "PVAL_SCHEMA_ENUM".into()
                        }
                    }
                    None => "PVAL_SCHEMA_ENUM".into(),
                }
            }
            PulseValueType::PVAL_VEC2(_) => "PVAL_VEC2".into(),
            PulseValueType::PVAL_VEC4(_) => "PVAL_VEC4".into(),
            PulseValueType::PVAL_QANGLE(_) => "PVAL_QANGLE".into(),
            PulseValueType::PVAL_TRANSFORM(_) => "PVAL_TRANSFORM".into(),
            PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => "PVAL_TRANSFORM_WORLDSPACE".into(),
            PulseValueType::PVAL_RESOURCE(resource_type, _) => {
                match resource_type.as_deref() {
                    Some(rt) if !rt.is_empty() => format!("PVAL_RESOURCE:{rt}").into(),
                    _ => "PVAL_RESOURCE".into(),
                }
            }
            PulseValueType::PVAL_ARRAY(arr_type) => {
                let arr_type_str = arr_type.get_enum_string(graph_bindings);
                format!("PVAL_ARRAY:{arr_type_str}").into()
            }
            PulseValueType::PVAL_GAMETIME(_) => "PVAL_GAMETIME".into(),
            PulseValueType::PVAL_VOID => "PVAL_VOID".into(),
        }
    }
    pub fn get_ui_name(&self) -> Cow<'static, str> {
        match self {
            PulseValueType::PVAL_INT(_) => "Integer".into(),
            PulseValueType::PVAL_TYPESAFE_INT(_, _) => "Typesafe Integer".into(),
            PulseValueType::PVAL_FLOAT(_) => "Float".into(),
            PulseValueType::PVAL_STRING(_) => "String".into(),
            PulseValueType::PVAL_INVALID => "Invalid".into(),
            PulseValueType::DOMAIN_ENTITY_NAME => "Entity Name".into(),
            PulseValueType::PVAL_EHANDLE(_) => "Entity".into(),
            PulseValueType::PVAL_VEC3(_) => "World Vector".into(),
            PulseValueType::PVAL_VEC3_LOCAL(_) => "Local Vector".into(),
            PulseValueType::PVAL_COLOR_RGB(_) => "Color RGB".into(),
            PulseValueType::PVAL_BOOL | PulseValueType::PVAL_BOOL_VALUE(_) => "Boolean".into(),
            PulseValueType::PVAL_SNDEVT_GUID(_) => "Sound Event".into(),
            PulseValueType::PVAL_SNDEVT_NAME(_) => "Sound Event Name".into(),
            PulseValueType::PVAL_ACT => "Action".into(),
            PulseValueType::PVAL_ANY => "Any Type".into(),
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type) => enum_type.to_str_ui().into(),
            PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(_, _) => "Schema Enum".into(),
            PulseValueType::PVAL_VEC2(_) => "Vector 2D".into(),
            PulseValueType::PVAL_VEC4(_) => "Vector 4D".into(),
            PulseValueType::PVAL_QANGLE(_) => "QAngle".into(),
            PulseValueType::PVAL_TRANSFORM(_) => "Transform".into(),
            PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_) => "World Transform".into(),
            PulseValueType::PVAL_RESOURCE(_, _) => "Resource".into(),
            PulseValueType::PVAL_ARRAY(_) => "Array".into(),
            PulseValueType::PVAL_GAMETIME(_) => "Game Time".into(),
            PulseValueType::PVAL_VOID => "Void".into(),
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
            PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(None, None),
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
            PulseValueType::PVAL_TYPESAFE_INT(None, None)
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

pub fn try_string_to_pulsevalue(enums: &EnumBindings, s: &str) -> Result<PulseValueType, PulseTypeError> {
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
                if let Some(enum_choice) = enums.iter().find(|e| e.name == enum_type) {
                    Ok(PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(Some(enum_choice.id), None))
                } else {
                    Err(PulseTypeError::StringToEnumConversionMissing(enum_type.to_string()))
                }
            } else if s.starts_with("PVAL_RESOURCE:") {
                let res_type = s.split_at(14).1;
                Ok(PulseValueType::PVAL_RESOURCE(Some(res_type.to_string()), None))
            } else if s.starts_with("PVAL_TYPESAFE_INT:") {
                let int_type = s.split_at(18).1;
                Ok(PulseValueType::PVAL_TYPESAFE_INT(Some(int_type.to_string()), None))
            } else if s.starts_with("PVAL_ARRAY:") {
                let arr_type = s.split_at(11).1;
                Ok(PulseValueType::PVAL_ARRAY(Box::new(
                    try_string_to_pulsevalue(enums, arr_type).unwrap_or(PulseValueType::PVAL_ANY)
                )))
            } else {
                Err(PulseTypeError::StringToEnumConversionMissing(s.to_string()))
            }
        }
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
        | PulseValueType::PVAL_GAMETIME(_)
        | PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(_, _) => InputParamKind::ConnectionOrConstant,

        PulseValueType::PVAL_EHANDLE(_)
        | PulseValueType::PVAL_SNDEVT_GUID(_)
        | PulseValueType::PVAL_INVALID
        | PulseValueType::PVAL_ACT
        | PulseValueType::PVAL_ANY
        | PulseValueType::PVAL_ARRAY(_)
        | PulseValueType::PVAL_VOID => InputParamKind::ConnectionOnly,

        PulseValueType::PVAL_BOOL
        | PulseValueType::PVAL_BOOL_VALUE(_)
        | PulseValueType::PVAL_SCHEMA_ENUM(_) => InputParamKind::ConstantOnly
    }
}

// The target is to later get rid of these conversions, for now however it will be easier to do this way, cause so much of UI code still depends on this.
pub fn pulsevaluetype_from_valuetype(valuetype: PulseGraphValueType) -> PulseValueType {
    match valuetype {
        PulseGraphValueType::Integer { value } => PulseValueType::PVAL_INT(Some(value)),
        PulseGraphValueType::Scalar { value } => PulseValueType::PVAL_FLOAT(Some(value)),
        PulseGraphValueType::String { value } => PulseValueType::PVAL_STRING(Some(value)),
        PulseGraphValueType::Bool { value } => PulseValueType::PVAL_BOOL_VALUE(Some(value)),
        PulseGraphValueType::Vec2 { value } => PulseValueType::PVAL_VEC2(Some(value)),
        PulseGraphValueType::Vec3 { value } => PulseValueType::PVAL_VEC3(Some(value)),
        PulseGraphValueType::Vec3Local { value } => PulseValueType::PVAL_VEC3_LOCAL(Some(value)),
        PulseGraphValueType::Vec4 { value } => PulseValueType::PVAL_VEC4(Some(value)),
        PulseGraphValueType::QAngle { value } => PulseValueType::PVAL_QANGLE(Some(value)),
        PulseGraphValueType::Transform => PulseValueType::PVAL_TRANSFORM(None),
        PulseGraphValueType::TransformWorldspace => PulseValueType::PVAL_TRANSFORM_WORLDSPACE(None),
        PulseGraphValueType::Color { value } => {
            PulseValueType::PVAL_COLOR_RGB(Some(Vec3::new(value[0], value[1], value[2])))
        }
        PulseGraphValueType::EHandle => PulseValueType::PVAL_EHANDLE(None),
        PulseGraphValueType::EntityName { .. } => PulseValueType::DOMAIN_ENTITY_NAME,
        PulseGraphValueType::SoundEventName { value } => PulseValueType::PVAL_SNDEVT_NAME(Some(value)),
        PulseGraphValueType::SndEventHandle => PulseValueType::PVAL_SNDEVT_GUID(None),
        PulseGraphValueType::Action => PulseValueType::PVAL_ACT,
        PulseGraphValueType::Typ { value } => value,
        PulseGraphValueType::SchemaEnum { enum_type, .. } => {
            PulseValueType::PVAL_SCHEMA_ENUM(enum_type)
        }
        PulseGraphValueType::SchemaEnumChoice { enum_type, enum_variant } => {
            PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(Some(enum_type), Some(enum_variant))
        }
        PulseGraphValueType::Resource { resource_type, value } => {
            PulseValueType::PVAL_RESOURCE(resource_type, Some(value))
        }
        PulseGraphValueType::ArrayVal { array_type } => {
            PulseValueType::PVAL_ARRAY(Box::new(pulsevaluetype_from_valuetype(array_type.into())))
        }
        PulseGraphValueType::GameTime => PulseValueType::PVAL_GAMETIME(None),
        PulseGraphValueType::TypeSafeInteger { integer_type } => {
            PulseValueType::PVAL_TYPESAFE_INT(Some(integer_type), None)
        }
        _ => PulseValueType::PVAL_ANY,
    }
}

impl From<PulseGraphValueType> for PulseValueType {
    fn from(value: PulseGraphValueType) -> Self {
        pulsevaluetype_from_valuetype(value)
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
