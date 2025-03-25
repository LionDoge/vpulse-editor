#![allow(nonstandard_style)]
use std::fmt;
use serde::{Deserialize, Serialize};

use crate::{app::{PulseDataType, Vec3}, serialization::{PulseRuntimeArgument, RegisterMap}};

pub enum CellType {
    Inflow,
    Step,
    Outflow,
    Value,
    Debug,
}

pub trait GetCellType {
    fn get_cell_type(&self) -> CellType;
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
impl GetCellType for CPulseCell_Inflow_Method {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_EventHandler {
    pub register_map: RegisterMap,
    pub entry_chunk: i32,
    pub event_name: String,
}
impl GetCellType for CPulseCell_Inflow_EventHandler {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Wait {
    pub(super) dest_chunk: i32,
    pub(super) instruction: i32
}
impl GetCellType for CPulseCell_Inflow_Wait {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Step_EntFire {
    pub input: String,
}
impl GetCellType for CPulseCell_Step_EntFire {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Value_FindEntByName {
    pub(super) entity_type: String,
}
impl GetCellType for CPulseCell_Value_FindEntByName {
    fn get_cell_type(&self) -> CellType {
        CellType::Value
    }
}

#[derive(Default)]
pub struct CPulseCell_Step_DebugLog;
impl GetCellType for CPulseCell_Step_DebugLog {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}

pub struct CPulseCell_Step_PublicOutput {
    pub(super) output_idx: i32,
}
impl GetCellType for CPulseCell_Step_PublicOutput {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}
impl CPulseCell_Step_PublicOutput {
    pub fn new(output_idx: i32) -> Self {
        Self {
            output_idx
        }
    }
}

pub struct CPulseCell_Inflow_GraphHook {
    pub(super) hook_name: String,
    pub(super) register_map: RegisterMap,
    pub(super) entry_chunk: i32,
}
impl GetCellType for CPulseCell_Inflow_GraphHook {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}
impl CPulseCell_Inflow_GraphHook {
    pub fn new(hook_name: String, register_map: RegisterMap, entry_chunk: i32) -> Self {
        Self {
            hook_name,
            register_map,
            entry_chunk
        }
    }
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

#[derive(Clone)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseVariable {
    pub name: String,
    pub typ_and_default_value: PulseValueType,
    // ui related
    pub data_type: PulseDataType,
    pub old_typ: PulseValueType,
    pub default_value_buffer: String,
}