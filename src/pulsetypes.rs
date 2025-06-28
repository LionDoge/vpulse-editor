#![allow(nonstandard_style)]
use crate::app::PulseDataType;
use crate::serialization::{PulseRuntimeArgument, RegisterMap};
use crate::typing::PulseValueType;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use typetag;

// Pulse Cells
#[allow(unused)]
pub enum CellType {
    Inflow,
    Step,
    Outflow,
    Value,
    Debug,
    Other,
}
#[allow(unused)]
pub trait PulseCell {
    fn get_cell_type(&self) -> CellType;
}
pub trait PulseCellTrait: PulseCell + crate::serialization::KV3Serialize {}
// blanket impl to make sure all cells implement the trait
impl<T> PulseCellTrait for T where T: PulseCell + crate::serialization::KV3Serialize {}

// Inflow Cells
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
impl PulseCell for CPulseCell_Inflow_Method {
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
impl PulseCell for CPulseCell_Inflow_EventHandler {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Wait {
    pub(super) dest_chunk: i32,
    pub(super) instruction: i32,
}
impl PulseCell for CPulseCell_Inflow_Wait {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

pub struct CPulseCell_Inflow_GraphHook {
    pub(super) hook_name: Cow<'static, str>,
    pub(super) register_map: RegisterMap,
    pub(super) entry_chunk: i32,
}
impl PulseCell for CPulseCell_Inflow_GraphHook {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}
impl CPulseCell_Inflow_GraphHook {
    pub fn new(hook_name: Cow<'static, str>, register_map: RegisterMap, entry_chunk: i32) -> Self {
        Self {
            hook_name,
            register_map,
            entry_chunk,
        }
    }
}

// Step Cells
#[allow(non_camel_case_types)]
pub struct CPulseCell_Step_EntFire {
    pub input: Cow<'static, str>,
}
impl PulseCell for CPulseCell_Step_EntFire {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}
impl CPulseCell_Step_EntFire {
    pub fn new(input: Cow<'static, str>) -> CPulseCell_Step_EntFire {
        CPulseCell_Step_EntFire { input }
    }
}

#[derive(Default)]
pub struct CPulseCell_Step_DebugLog;
impl PulseCell for CPulseCell_Step_DebugLog {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}
pub struct CPulseCell_Step_PublicOutput {
    pub(super) output_idx: i32,
}
impl PulseCell for CPulseCell_Step_PublicOutput {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}
impl CPulseCell_Step_PublicOutput {
    pub fn new(output_idx: i32) -> Self {
        Self { output_idx }
    }
}

// Value Cells
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Value_FindEntByName {
    pub(super) entity_type: Cow<'static, str>,
}
impl PulseCell for CPulseCell_Value_FindEntByName {
    fn get_cell_type(&self) -> CellType {
        CellType::Value
    }
}
impl CPulseCell_Value_FindEntByName {
    pub fn new(entity_type: Cow<'static, str>) -> CPulseCell_Value_FindEntByName {
        CPulseCell_Value_FindEntByName { entity_type }
    }
}

pub struct CPulseCell_Value_FindEntByClassNameWithin {
    pub(super) entity_type: Cow<'static, str>,
}
impl CPulseCell_Value_FindEntByClassNameWithin {
    pub fn new(entity_type: Cow<'static, str>) -> Self {
        Self { entity_type }
    }
}
impl PulseCell for CPulseCell_Value_FindEntByClassNameWithin {
    fn get_cell_type(&self) -> CellType {
        CellType::Value
    }
}

// Outflow Cells
pub struct OutflowConnection {
    pub outflow_name: Cow<'static, str>,
    pub dest_chunk: i32,
    pub dest_instruction: i32,
    pub register_map: Option<RegisterMap>,
}
impl Default for OutflowConnection {
    fn default() -> Self {
        Self {
            outflow_name: Cow::Borrowed(""),
            dest_chunk: -1,
            dest_instruction: -1,
            register_map: None,
        }
    }
}
impl OutflowConnection {
    pub fn new(
        outflow_name: Cow<'static, str>,
        dest_chunk: i32,
        dest_instruction: i32,
        register_map: Option<RegisterMap>,
    ) -> Self {
        Self {
            outflow_name,
            dest_chunk,
            dest_instruction,
            register_map,
        }
    }
}
pub struct CPulseCell_Outflow_IntSwitch {
    pub(super) default_outflow: OutflowConnection,
    pub(super) ouflows: Vec<OutflowConnection>,
}
impl PulseCell for CPulseCell_Outflow_IntSwitch {
    fn get_cell_type(&self) -> CellType {
        CellType::Outflow
    }
}
impl CPulseCell_Outflow_IntSwitch {
    pub fn new(default_outflow: OutflowConnection, ouflows: Vec<OutflowConnection>) -> Self {
        Self {
            default_outflow,
            ouflows,
        }
    }
}

// Other
#[allow(unused)]
pub enum SoundEventStartType {
    SOUNDEVENT_START_PLAYER,
    SOUNDEVENT_START_WORLD,
    SOUNDEVENT_START_ENTITY,
}

pub struct CPulseCell_SoundEventStart {
    pub(super) typ: SoundEventStartType,
}
impl PulseCell for CPulseCell_SoundEventStart {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}
impl CPulseCell_SoundEventStart {
    pub fn new(typ: SoundEventStartType) -> Self {
        Self { typ }
    }
}
pub struct CPulseCell_Outflow_ListenForEntityOutput {
    pub outflow_onfired: OutflowConnection,
    pub outflow_oncanceled: OutflowConnection,
    pub entity_output: String,
    pub entity_output_param: String,
    pub listen_until_canceled: bool,
}

impl PulseCell for CPulseCell_Outflow_ListenForEntityOutput {
    fn get_cell_type(&self) -> CellType {
        CellType::Outflow
    }
}

pub struct TimelineEvent {
    pub(super) time_from_previous: f32,
    pub(super) pause_for_previous_events: f32,
    pub(super) call_mode_sync: bool,
    pub(super) event_outflow: OutflowConnection,
}
pub struct CPulseCell_Timeline {
    pub(super) outflow_onfinished: OutflowConnection,
    pub(super) wait_for_child_outflows: bool,
    pub(super) timeline_events: Vec<TimelineEvent>,
}

impl PulseCell for CPulseCell_Timeline {
    fn get_cell_type(&self) -> CellType {
        CellType::Other
    }
}

impl CPulseCell_Timeline {
    pub fn new(outflow_onfinished: OutflowConnection, wait_for_child_outflows: bool) -> Self {
        Self {
            outflow_onfinished,
            wait_for_child_outflows,
            timeline_events: Vec::new(),
        }
    }
    pub fn add_event(
        &mut self,
        time_from_previous: f32,
        pause_for_previous_events: f32,
        call_mode_sync: bool,
        event_outflow: OutflowConnection,
    ) {
        let event = TimelineEvent {
            time_from_previous,
            pause_for_previous_events,
            call_mode_sync,
            event_outflow,
        };
        self.timeline_events.push(event);
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PulseVariable {
    pub name: String,
    pub typ_and_default_value: PulseValueType,
    // ui related
    pub data_type: PulseDataType,
    pub old_typ: PulseValueType,
    pub default_value_buffer: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputDefinition {
    pub name: String,
    pub typ: PulseValueType,
    pub typ_old: PulseValueType, // used for detecting change in combobox, eugh.
}

pub trait SchemaEnumTrait {
    fn get_ui_name(&self) -> &'static str;
    fn get_self_kv_name(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PulseCursorCancelPriority {
    None,
    CancelOnSucceeded,
    SoftCancel,
    HardCancel,
}

impl SchemaEnumTrait for PulseCursorCancelPriority {
    fn get_ui_name(&self) -> &'static str {
        match self {
            PulseCursorCancelPriority::None => "Keep running normally.",
            PulseCursorCancelPriority::CancelOnSucceeded => "Kill after current node.",
            PulseCursorCancelPriority::SoftCancel => "Kill elegantly.",
            PulseCursorCancelPriority::HardCancel => "Kill immediately.",
        }
    }
    fn get_self_kv_name(&self) -> &'static str {
        "PulseCursorCancelPriority_t"
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

impl SchemaEnumValue {
    pub fn get_ui_name(&self) -> &'static str {
        match self {
            SchemaEnumValue::PulseCursorCancelPriority(value) => value.get_ui_name(),
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
}
