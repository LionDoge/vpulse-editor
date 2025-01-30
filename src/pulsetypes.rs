#![allow(nonstandard_style)]
use std::fmt;
use serde::{Deserialize, Serialize};

use crate::{app::{PulseDataType, Vec3}, serialization::{PulseRuntimeArgument, RegisterMap}};

pub enum CellType {
    InflowMethod(CPulseCell_Inflow_Method),
    InflowEvent(CPulseCell_Inflow_EventHandler),
    StepEntFire(CPulseCell_Step_EntFire),
    InflowWait(CPulseCell_Inflow_Wait),
    ValueFindEntByName(CPulseCell_Value_FindEntByName),
    StepPublicOutput(i32),
    DebugLog,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Method {
    pub register_map: RegisterMap,
    pub entry_chunk: i32,
    pub name: String,
    pub description: String,
    pub return_type: String,
    pub args: Vec<PulseRuntimeArgument>,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_EventHandler {
    pub register_map: RegisterMap,
    pub entry_chunk: i32,
    pub event_name: String,
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Wait {
    pub(super) dest_chunk: i32,
    pub(super) instruction: i32
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Step_EntFire {
    pub input: String,
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Value_FindEntByName {
    pub(super) entity_type: String,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PulseValueType {
    PVAL_INT(Option<i32>),
    PVAL_FLOAT(Option<f32>),
    PVAL_STRING(Option<String>),
    PVAL_INVALID,
    PVAL_EHANDLE(Option<String>),
    PVAL_VEC3(Option<Vec3>),
    PVAL_COLOR_RGB(Option<Vec3>),
    DOMAIN_ENTITY_NAME,
}
impl fmt::Display for PulseValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseValueType::PVAL_INT(_) => write!(f, "PVAL_INT"),
            PulseValueType::PVAL_FLOAT(_) => write!(f, "PVAL_FLOAT"),
            PulseValueType::PVAL_STRING(_) => write!(f, "PVAL_STRING"),
            PulseValueType::PVAL_INVALID => write!(f, "PVAL_INVALID"),
            PulseValueType::DOMAIN_ENTITY_NAME => write!(f, "PVAL_ENTITY_NAME"),
            PulseValueType::PVAL_EHANDLE(ent_type) => write!(f, "PVAL_EHANDLE:{}", ent_type.clone().unwrap_or_default()),
            PulseValueType::PVAL_VEC3(_) => write!(f, "PVAL_VEC3"),
            PulseValueType::PVAL_COLOR_RGB(_) => write!(f, "PVAL_COLOR_RGB"),
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]

pub struct PulseVariable {
    pub name: String,
    pub typ_and_default_value: PulseValueType,
    // ui related
    pub data_type: PulseDataType,
    pub old_typ: PulseValueType,
    pub default_value_buffer: String,
}