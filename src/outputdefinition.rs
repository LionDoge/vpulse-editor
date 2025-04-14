use serde::{Deserialize, Serialize};
use crate::typing::PulseValueType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputDefinition {
    pub name: String,
    pub typ: PulseValueType,
    pub typ_old: PulseValueType // used for detecting change in combobox, eugh.
}
