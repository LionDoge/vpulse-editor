#![allow(nonstandard_style)]
use std::borrow::Cow;
use super::enumerators::SoundEventStartType;
use crate::compiler::serialization::{KV3Serialize, PulseRuntimeArgument, RegisterMap};

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
pub trait PulseCellTrait: PulseCell + KV3Serialize {}
// blanket impl to make sure all cells implement the trait
impl<T> PulseCellTrait for T where T: PulseCell + KV3Serialize {}

// Inflow Cells
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Method {
    pub(crate) register_map: RegisterMap,
    pub(crate) entry_chunk: i32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) return_type: String,
    pub(crate) args: Vec<PulseRuntimeArgument>,
}
impl PulseCell for CPulseCell_Inflow_Method {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_EventHandler {
    pub(crate) register_map: RegisterMap,
    pub(crate) entry_chunk: i32,
    pub(crate) event_name: Cow<'static, str>,
}
impl PulseCell for CPulseCell_Inflow_EventHandler {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

#[allow(non_camel_case_types)]
pub struct CPulseCell_Inflow_Wait {
    pub(crate) dest_chunk: i32,
    pub(crate) instruction: i32,
}
impl PulseCell for CPulseCell_Inflow_Wait {
    fn get_cell_type(&self) -> CellType {
        CellType::Inflow
    }
}

pub struct CPulseCell_Inflow_GraphHook {
    pub(crate) hook_name: Cow<'static, str>,
    pub(crate) register_map: RegisterMap,
    pub(crate) entry_chunk: i32,
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
    pub(crate) input: Cow<'static, str>,
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
    pub(crate) output_idx: i32,
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
    pub(crate) default_outflow: OutflowConnection,
    pub(crate) ouflows: Vec<OutflowConnection>,
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

// Other cells
pub struct CPulseCell_SoundEventStart {
    pub(crate) typ: SoundEventStartType,
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
    pub(crate) outflow_onfired: OutflowConnection,
    pub(crate) outflow_oncanceled: OutflowConnection,
    pub(crate) entity_output: String,
    pub(crate) entity_output_param: String,
    pub(crate) listen_until_canceled: bool,
}

impl PulseCell for CPulseCell_Outflow_ListenForEntityOutput {
    fn get_cell_type(&self) -> CellType {
        CellType::Outflow
    }
}

pub struct TimelineEvent {
    pub(crate) time_from_previous: f32,
    pub(crate) pause_for_previous_events: f32,
    pub(crate) call_mode_sync: bool,
    pub(crate) event_outflow: OutflowConnection,
}
pub struct CPulseCell_Timeline {
    pub(crate) outflow_onfinished: OutflowConnection,
    pub(crate) wait_for_child_outflows: bool,
    pub(crate) timeline_events: Vec<TimelineEvent>,
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

pub struct CPulseCell_Step_SetAnimGraphParam {
    pub(crate) param_name: Cow<'static, str>,
}

impl PulseCell for CPulseCell_Step_SetAnimGraphParam {
    fn get_cell_type(&self) -> CellType {
        CellType::Step
    }
}

impl CPulseCell_Step_SetAnimGraphParam {
    pub fn new(param_name: Cow<'static, str>) -> Self {
        Self { param_name }
    }
}
#[derive(Default)]
pub struct CPulseCell_Value_RandomInt;

impl PulseCell for CPulseCell_Value_RandomInt {
    fn get_cell_type(&self) -> CellType {
        CellType::Value
    }
}
#[derive(Default)]
pub struct CPulseCell_Value_RandomFloat;

impl PulseCell for CPulseCell_Value_RandomFloat {
    fn get_cell_type(&self) -> CellType {
        CellType::Value
    }
}