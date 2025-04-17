#![allow(nonstandard_style)]
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::typing::{PulseValueType, Vec3};
use crate::{app::{PulseDataType}, serialization::{PulseRuntimeArgument, RegisterMap}};
use std::borrow::Cow;
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
    pub event_name: Cow<'static, str>,
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

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct PulseVariable {
    pub name: String,
    pub typ_and_default_value: PulseValueType,
    // ui related
    pub data_type: PulseDataType,
    pub old_typ: PulseValueType,
    pub default_value_buffer: String,
}