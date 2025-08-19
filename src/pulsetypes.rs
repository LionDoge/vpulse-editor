mod cells;
pub mod enumerators;
pub use cells::*;
pub use enumerators::*;

use std::fmt::Debug;
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