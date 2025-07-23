mod cells;
pub use cells::*;

use std::{fmt::Debug, str::FromStr};
use serde::{Deserialize, Serialize};
use crate::app::types::PulseDataType;
use crate::typing::PulseValueType;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PulseVariable {
    pub name: String,
    pub typ_and_default_value: PulseValueType,
    // ui related
    pub data_type: PulseDataType,
    pub default_value_buffer: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputDefinition {
    pub name: String,
    pub typ: PulseValueType,
    pub typ_old: PulseValueType, // used for detecting change in combobox, eugh.
}

pub trait SchemaEnumTrait {
    fn to_str(self) -> &'static str;
    fn to_str_ui(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PulseCursorCancelPriority {
    None,
    CancelOnSucceeded,
    SoftCancel,
    HardCancel,
}

impl SchemaEnumTrait for PulseCursorCancelPriority {
    fn to_str(self) -> &'static str {
        match self {
            PulseCursorCancelPriority::None => "None",
            PulseCursorCancelPriority::CancelOnSucceeded => "CancelOnSucceeded",
            PulseCursorCancelPriority::SoftCancel => "SoftCancel",
            PulseCursorCancelPriority::HardCancel => "HardCancel",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseCursorCancelPriority::None => "Keep running normally.",
            PulseCursorCancelPriority::CancelOnSucceeded => "Kill after current node.",
            PulseCursorCancelPriority::SoftCancel => "Kill elegantly.",
            PulseCursorCancelPriority::HardCancel => "Kill immediately.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaEnumType {
    PulseCursorCancelPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaEnumValue {
    PulseCursorCancelPriority(PulseCursorCancelPriority),
}

impl FromStr for SchemaEnumType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "PulseCursorCancelPriority_t" => Ok(SchemaEnumType::PulseCursorCancelPriority),
            _ => Err(anyhow::anyhow!(
                "Unknown SchemaEnumType: {}",
                s
            )),
        }
    }
}

impl SchemaEnumType {
    pub fn get_all_types_as_enums(&self) -> Vec<SchemaEnumValue> {
        match self {
            SchemaEnumType::PulseCursorCancelPriority => {
                vec![
                    SchemaEnumValue::PulseCursorCancelPriority(PulseCursorCancelPriority::None),
                    SchemaEnumValue::PulseCursorCancelPriority(
                        PulseCursorCancelPriority::CancelOnSucceeded,
                    ),
                    SchemaEnumValue::PulseCursorCancelPriority(
                        PulseCursorCancelPriority::SoftCancel,
                    ),
                    SchemaEnumValue::PulseCursorCancelPriority(
                        PulseCursorCancelPriority::HardCancel,
                    ),
                ]
            }
        }
    }
    pub fn to_str(self) -> &'static str {
        match self {
            SchemaEnumType::PulseCursorCancelPriority => "PulseCursorCancelPriority_t"
        }
    }
    pub fn to_str_ui(self) -> &'static str {
        match self {
            SchemaEnumType::PulseCursorCancelPriority => "Cursor Cancel Priority",
        }
    }
}

impl SchemaEnumValue {
    pub fn get_ui_name(&self) -> &'static str {
        match self {
            SchemaEnumValue::PulseCursorCancelPriority(value) => value.to_str_ui(),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            SchemaEnumValue::PulseCursorCancelPriority(value) => value.to_str(),
        }
    }
    pub fn default_from_type(typ: &SchemaEnumType) -> Self {
        match typ {
            SchemaEnumType::PulseCursorCancelPriority => {
                SchemaEnumValue::PulseCursorCancelPriority(PulseCursorCancelPriority::None)
            }
        }
    }
}