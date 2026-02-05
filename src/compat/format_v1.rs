use crate::app::types::{PulseGraphState, FileVersion, MyEditorState};
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PulseGraphEditorV1 {
    #[cfg_attr(feature = "persistence", serde(skip))]
    #[allow(unused)]
    version: FileVersion,
    state: MyEditorState,
    user_state: PulseGraphState,
}