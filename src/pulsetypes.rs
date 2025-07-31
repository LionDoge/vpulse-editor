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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PulseTraceContents {
    StaticLevel,
    Solid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PulseCollisionGroup {
    Default,
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

impl SchemaEnumTrait for PulseTraceContents {
    fn to_str(self) -> &'static str {
        match self {
            PulseTraceContents::StaticLevel => "STATIC_LEVEL",
            PulseTraceContents::Solid => "SOLID",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseTraceContents::StaticLevel => "Static Level",
            PulseTraceContents::Solid => "Solid",
        }
    }
}

impl SchemaEnumTrait for PulseCollisionGroup {
    fn to_str(self) -> &'static str {
        match self {
            PulseCollisionGroup::Default => "DEFAULT",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseCollisionGroup::Default => "Default",
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaEnumType {
    CursorCancelPriority,
    TraceContents,
    CollisionGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaEnumValue {
    CursorCancelPriority(PulseCursorCancelPriority),
    TraceContents(PulseTraceContents),
    CollisionGroup(PulseCollisionGroup),
}

impl FromStr for SchemaEnumType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "PulseCursorCancelPriority_t" => Ok(SchemaEnumType::CursorCancelPriority),
            "PulseTraceContents_t" => Ok(SchemaEnumType::TraceContents),
            "PulseCollisionGroup_t" => Ok(SchemaEnumType::CollisionGroup),
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
            SchemaEnumType::CursorCancelPriority => {
                vec![
                    SchemaEnumValue::CursorCancelPriority(PulseCursorCancelPriority::None),
                    SchemaEnumValue::CursorCancelPriority(
                        PulseCursorCancelPriority::CancelOnSucceeded,
                    ),
                    SchemaEnumValue::CursorCancelPriority(
                        PulseCursorCancelPriority::SoftCancel,
                    ),
                    SchemaEnumValue::CursorCancelPriority(
                        PulseCursorCancelPriority::HardCancel,
                    ),
                ]
            }
            SchemaEnumType::TraceContents => {
                vec![
                    SchemaEnumValue::TraceContents(PulseTraceContents::StaticLevel),
                    SchemaEnumValue::TraceContents(PulseTraceContents::Solid),
                ]
            }
            SchemaEnumType::CollisionGroup => {
                vec![SchemaEnumValue::CollisionGroup(PulseCollisionGroup::Default)]
            }
        }
    }
    pub fn to_str(self) -> &'static str {
        match self {
            SchemaEnumType::CursorCancelPriority => "PulseCursorCancelPriority_t",
            SchemaEnumType::TraceContents => "PulseTraceContents_t",
            SchemaEnumType::CollisionGroup => "PulseCollisionGroup_t",
        }
    }
    pub fn to_str_ui(self) -> &'static str {
        match self {
            SchemaEnumType::CursorCancelPriority => "Cursor Cancel Priority",
            SchemaEnumType::TraceContents => "Trace Contents",
            SchemaEnumType::CollisionGroup => "Collision Group",
        }
    }
}

impl SchemaEnumValue {
    pub fn get_ui_name(&self) -> &'static str {
        match self {
            SchemaEnumValue::CursorCancelPriority(value) => value.to_str_ui(),
            SchemaEnumValue::TraceContents(value) => value.to_str_ui(),
            SchemaEnumValue::CollisionGroup(value) => value.to_str_ui(),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            SchemaEnumValue::CursorCancelPriority(value) => value.to_str(),
            SchemaEnumValue::TraceContents(value) => value.to_str(),
            SchemaEnumValue::CollisionGroup(value) => value.to_str(),
        }
    }
    pub fn default_from_type(typ: &SchemaEnumType) -> Self {
        match typ {
            SchemaEnumType::CursorCancelPriority => {
                SchemaEnumValue::CursorCancelPriority(PulseCursorCancelPriority::None)
            }
            SchemaEnumType::TraceContents => {
                SchemaEnumValue::TraceContents(PulseTraceContents::StaticLevel)
            }
            SchemaEnumType::CollisionGroup => {
                SchemaEnumValue::CollisionGroup(PulseCollisionGroup::Default)
            }
        }
    }
}