use serde::{Deserialize, Serialize};

use crate::pulsetypes::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputDefinition {
    pub name: String,
    pub typ: PulseValueType,
    pub typ_old: PulseValueType // used for detecting change in combobox, eugh.
}
