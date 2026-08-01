mod cells;
pub mod enumerators;
pub use cells::*;
pub use enumerators::*;

use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::app::types::{PulseDataType, PulseGraphValueType};
use crate::typing::PulseValueType;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PulseVariable {
    pub name: String,
    pub data_type: PulseDataType,
    pub stored_value: PulseGraphValueType,

    // deprecated
    #[serde(skip_serializing)]
    pub typ_and_default_value: PulseValueType,
    #[serde(skip_serializing)]
    pub default_value_buffer: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OutputDefinition {
    pub name: String,
    pub data_type: PulseDataType,
    pub value_type: PulseGraphValueType, // we don't hold the default value, but needed for inner-types.

    // deprecated
    #[serde(skip_serializing)]
    pub typ: PulseValueType,
    #[serde(skip_serializing)]
    pub typ_old: PulseValueType,
}