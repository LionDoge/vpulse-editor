#![allow(non_camel_case_types)]
#![allow(nonstandard_style)]

use std::borrow::Cow;
use egui_node_graph2::{InputId, NodeId, OutputId};
use indoc::formatdoc;
use slotmap::SecondaryMap;
use crate::{
    pulsetypes::*,
    typing::{PulseValueType, Vec2, Vec3, Vec4},
};

pub trait KV3Serialize {
    fn serialize(&self) -> String;
}
pub struct PulseRuntimeArgument {
    pub name: String,
    pub description: String,
    pub typ: String,
}

impl KV3Serialize for PulseRuntimeArgument {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_Name = \"{}\"
                m_Description = \"{}\"
                m_Type = \"{}\"
            }}
            "
            , self.name, self.description, self.typ
        }
    }
}

impl KV3Serialize for CPulseCell_Inflow_Method {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Inflow_Method\"
                m_nEditorNodeID = -1
                m_EntryChunk = {}
                m_RegisterMap = {}
                m_MethodName = \"{}\"
                m_Description = \"{}\"
                m_bIsPublic = true
                m_ReturnType = \"PVAL_VOID\"
                m_Args =
                [
                    {}
                ]
            }}
            "
            , self.entry_chunk, self.register_map.serialize(), self.name, self.description
            , self.args.iter().map(|arg| arg.serialize()).collect::<Vec<String>>().join(",\n\n")
        }
    }
}

impl CPulseCell_Inflow_Method {
    pub fn add_arg(&mut self, name: String, description: String, typ: String, out_register: i32) {
        let arg = PulseRuntimeArgument {
            name: name.clone(),
            description,
            typ,
        };
        self.args.push(arg);
        self.register_map.add_outparam(name.into(), out_register);
    }
}

impl KV3Serialize for CPulseCell_Inflow_EventHandler {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Inflow_EventHandler\"
                m_nEditorNodeID = -1
                m_EntryChunk = {}
                m_RegisterMap = {}
                m_EventName = \"{}\"
            }}
            "
            , self.entry_chunk, self.register_map.serialize(), self.event_name
        }
    }
}
impl CPulseCell_Inflow_EventHandler {
    pub fn new(entry_chunk: i32, event_name: Cow<'static, str>) -> CPulseCell_Inflow_EventHandler {
        CPulseCell_Inflow_EventHandler {
            register_map: RegisterMap::default(),
            entry_chunk,
            event_name,
        }
    }
    pub fn add_outparam(&mut self, name: Cow<'static, str>, num: i32) {
        self.register_map.add_outparam(name, num);
    }
}

impl KV3Serialize for CPulseCell_Inflow_Wait {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Inflow_Wait\"
                m_nEditorNodeID = -1
                m_WakeResume = 
                {{
                    m_SourceOutflowName = \"m_WakeResume\"
                    m_nDestChunk = {}
                    m_nInstruction = {}
                }}
            }}
            "
            , self.dest_chunk, self.instruction
        }
    }
}
impl CPulseCell_Inflow_Wait {
    pub fn new(dest_chunk: i32, instruction: i32) -> CPulseCell_Inflow_Wait {
        CPulseCell_Inflow_Wait {
            dest_chunk,
            instruction,
        }
    }
}

impl KV3Serialize for CPulseCell_Step_EntFire {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Step_EntFire\"
                m_nEditorNodeID = -1
                m_Input = \"{}\"
            }}
            "
            , self.input
        }
    }
}

impl KV3Serialize for CPulseCell_Step_DebugLog {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Step_DebugLog\"
                m_nEditorNodeID = -1
            }}
            "
        }
    }
}

impl KV3Serialize for CPulseCell_Step_PublicOutput {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Step_PublicOutput\"
                m_nEditorNodeID = -1
                m_OutputIndex = {}
            }}
            "
            , self.output_idx
        }
    }
}

impl KV3Serialize for CPulseCell_Inflow_GraphHook {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Inflow_GraphHook\"
                m_nEditorNodeID = -1
                m_EntryChunk = {}
                m_RegisterMap = {}
                m_HookName = \"{}\"
            }}
            "
            , self.entry_chunk, self.register_map.serialize(), self.hook_name
        }
    }
}

impl KV3Serialize for Register {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_nReg = {}
                m_Type = \"{}\"
                m_OriginName = \"0:null\"
                m_nWrittenByInstruction = {}
                m_nLastReadByInstruction = -1
            }}
            "
            , self.num, self.reg_type, self.written_by_instruction
        }
    }
}

#[derive(Default, Clone)]
pub struct RegisterMap {
    pub inparams: Vec<(Cow<'static, str>, i32)>,
    pub outparams: Vec<(Cow<'static, str>, i32)>,
}
impl RegisterMap {
    pub fn add_inparam(&mut self, name: Cow<'static, str>, num: i32) {
        self.inparams.push((name, num));
    }
    pub fn add_outparam(&mut self, name: Cow<'static, str>, num: i32) {
        self.outparams.push((name, num));
    }
    #[allow(dead_code)]
    pub fn get_inparam_by_name(&self, name: &str) -> Option<i32> {
        self.inparams.iter().find(|(n, _)| n == name).map(|(_, num)| *num)
    }
    pub fn get_outparam_by_name(&self, name: &str) -> Option<i32> {
        self.outparams.iter().find(|(n, _)| n == name).map(|(_, num)| *num)
    }
}

impl KV3Serialize for RegisterMap {
    fn serialize(&self) -> String {
        let inparams_str: String = if !self.inparams.is_empty() {
            formatdoc!(
                "{{
                    {}
                }}",
                self.inparams
                    .iter()
                    .map(|(name, num)| format!("{name} = {num}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        } else {
            String::from("null")
        };
        let outparams_str: String = if !self.outparams.is_empty() {
            formatdoc!(
                "{{
                    {}
                }}",
                self.outparams
                    .iter()
                    .map(|(name, num)| format!("{name} = {num}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        } else {
            String::from("null")
        };
        formatdoc! {
            "
            {{
                m_Inparams = {}
                m_Outparams = {}
            }}
            "
            , inparams_str, outparams_str
        }
    }
}

pub struct Instruction {
    pub code: String,
    pub var: i32,
    pub reg0: i32,
    pub reg1: i32,
    pub reg2: i32,
    pub invoke_binding_index: i32,
    pub chunk: i32,
    pub dest_instruction: i32,
    pub call_info_index: i32,
    pub const_idx: i32,
    pub domain_value_idx: i32,
    pub blackboard_reference_idx: i32,
}
impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            code: String::from("NOP"),
            var: -1,
            reg0: -1,
            reg1: -1,
            reg2: -1,
            invoke_binding_index: -1,
            chunk: -1,
            dest_instruction: 0,
            call_info_index: -1,
            const_idx: -1,
            domain_value_idx: -1,
            blackboard_reference_idx: -1,
        }
    }
}
impl KV3Serialize for Instruction {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_nCode = \"{}\"
                m_nVar = {}
                m_nReg0 = {}
                m_nReg1 = {}
                m_nReg2 = {}
                m_nInvokeBindingIndex = {}
                m_nChunk = {}
                m_nDestInstruction = {}
                m_nCallInfoIndex = {}
                m_nConstIdx = {}
                m_nDomainValueIdx = {}
                m_nBlackboardReferenceIdx = {}
            }}
            "
            , self.code, self.var, self.reg0, self.reg1, self.reg2,
             self.invoke_binding_index, self.chunk, self.dest_instruction,
            self.call_info_index, self.const_idx, self.domain_value_idx, self.blackboard_reference_idx
        }
    }
}

#[derive(Default)]
pub struct PulseChunk {
    instructions: Vec<Instruction>,
    registers: Vec<Register>,
    pub instruction_editor_ids: Vec<i32>,
}
impl PulseChunk {
    pub fn add_register(&mut self, reg_type: String, written_by_instruction: i32) -> i32 {
        let num = self.registers.len() as i32;
        let register = Register::new(num, reg_type, written_by_instruction);
        self.registers.push(register);
        num
    }
    pub fn add_instruction(&mut self, instruction: Instruction) -> i32 {
        self.instructions.push(instruction);
        self.instruction_editor_ids.push(-1);
        self.instructions.len() as i32 - 1
    }
    pub fn get_last_instruction_id(&self) -> i32 {
        self.instructions.len() as i32 - 1
    }
    pub fn get_instruction_from_id_mut(&mut self, id: i32) -> Option<&mut Instruction> {
        self.instructions.get_mut(id as usize)
    }
    #[allow(dead_code)]
    pub fn get_last_register_id(&self) -> i32 {
        self.registers.len() as i32 - 1
    }
}
impl KV3Serialize for PulseChunk {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_Instructions = 
                [
                    {}
                ]
                m_Registers = 
                [
                    {}
                ]
                m_InstructionEditorIDs = 
                [
                    {}
                ]
            }}
            "
            , self.instructions.iter().map(|instruction| instruction.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.registers.iter().map(|register| register.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.instruction_editor_ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(",\n\n")
        }
    }
}

pub struct InvokeBinding {
    pub register_map: RegisterMap,
    pub func_name: Cow<'static, str>,
    pub cell_index: i32,
    pub src_chunk: i32,
    pub src_instruction: i32,
}

impl KV3Serialize for InvokeBinding {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_RegisterMap = {}
                m_FuncName = \"{}\"
                m_nCellIndex = {}
                m_nSrcChunk = {}
                m_nSrcInstruction = {}
            }}
            "
            , self.register_map.serialize(), self.func_name, self.cell_index, self.src_chunk, self.src_instruction
        }
    }
}

#[derive(Default)]
pub struct DomainValue {
    pub typ: Cow<'static, str>,
    pub value: Cow<'static, str>,
    pub __deprecated_expected_runtime_type: Cow<'static, str>,
    pub required_runtime_type: Cow<'static, str>,
}
impl KV3Serialize for DomainValue {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_nType = \"{}\"
                m_Value = \"{}\"
                m_RequiredRuntimeType = \"{}\"
            }}
            "
            , self.typ, self.value, self.required_runtime_type
        }
    }
}

pub struct OutputConnection {
    pub source_output: String,
    pub target_entity: String,
    pub target_input: String,
    pub param: String,
}
impl OutputConnection {
    pub fn new(
        source_output: String,
        target_entity: String,
        target_input: String,
        param: String,
    ) -> OutputConnection {
        OutputConnection {
            source_output,
            target_entity,
            target_input,
            param,
        }
    }
}
impl KV3Serialize for OutputConnection {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_SourceOutput = \"{}\"
                m_TargetEntity = \"{}\"
                m_TargetInput = \"{}\"
                m_Param = \"{}\"
            }}
            "
            , self.source_output, self.target_entity, self.target_input, self.param
        }
    }
}

impl KV3Serialize for OutputDefinition {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_Name = \"{}\"
                m_Description = \"\"
                m_ParamType = \"{}\"
            }}
            "
            , self.name, self.typ.to_string()
        }
    }
}

#[derive(Default)]
pub struct Register {
    num: i32,
    reg_type: String,
    pub written_by_instruction: i32,
}
impl Register {
    pub fn new(num: i32, reg_type: String, written_by_instruction: i32) -> Self {
        Register {
            num,
            reg_type,
            written_by_instruction,
        }
    }
}
#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum PulseConstant {
    String(String),
    SoundEventName(String),
    Float(f32),
    Integer(i32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Vec3Local(Vec3),
    QAngle(Vec3),
    Color_RGB([f32; 4]),
    Bool(bool),
    SchemaEnum(SchemaEnumType, SchemaEnumValue),
    Resource(Option<String>, String), // (resource_type, value)
    Array(PulseValueType, String), // raw KV3 array content
}
impl KV3Serialize for PulseConstant {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_Type = \"{}\"
                m_Value = {}
            }}
            "
            ,
            match self {
                PulseConstant::String(_) => "PVAL_STRING".to_string(),
                PulseConstant::SoundEventName(_) => "PVAL_SNDEVT_NAME".to_string(),
                PulseConstant::Float(_) => "PVAL_FLOAT".to_string(),
                PulseConstant::Integer(_) => "PVAL_INT".to_string(),
                PulseConstant::Vec2(_) => "PVAL_VEC2".to_string(),
                PulseConstant::Vec3(_) => "PVAL_VEC3_WORLDSPACE".to_string(),
                PulseConstant::Vec4(_) => "PVAL_VEC4".to_string(),
                PulseConstant::QAngle(_) => "PVAL_QANGLE".to_string(),
                PulseConstant::Vec3Local(_) =>  "PVAL_VEC3".to_string(),
                PulseConstant::Color_RGB(_) => "PVAL_COLOR_RGB".to_string(),
                PulseConstant::Bool(_) => "PVAL_BOOL".to_string(),
                PulseConstant::SchemaEnum(typ, _) => format!("PVAL_SCHEMA_ENUM:{}", typ.to_str()),
                PulseConstant::Resource(resource_type, _) => {
                    if let Some(resource_type) = resource_type {
                        format!("PVAL_RESOURCE:{resource_type}")
                    } else {
                        "PVAL_RESOURCE".to_string()
                    }
                }
                PulseConstant::Array(typ, _) => {
                    format!("PVAL_ARRAY:{}", typ)
                }
            },
            match self {
                PulseConstant::String(value)
                | PulseConstant::SoundEventName(value) => format!("\"{value}\""),
                PulseConstant::Float(value) => format!("{value:.8}"),
                PulseConstant::Integer(value) => value.to_string(),
                PulseConstant::Vec2(value) => format!("[{:.3}, {:.3}]", value.x, value.y),
                PulseConstant::Vec3(value)
                | PulseConstant::Vec3Local(value)
                | PulseConstant::QAngle(value) => format!("[{:.3}, {:.3}, {:.3}]", value.x, value.y, value.z),
                PulseConstant::Vec4(value) => format!("[{:.3}, {:.3}, {:.3}, {:.3}]", value.x, value.y, value.z, value.w),
                PulseConstant::Color_RGB(value) => format!("[{}, {}, {}]", value[0], value[1], value[2]),
                PulseConstant::Bool(value) => value.to_string(),
                PulseConstant::SchemaEnum(_, value) => format!("\"{}\"", value.to_str()),
                PulseConstant::Resource(_, value) => format!("resource:\"{value}\""),
                PulseConstant::Array(_, value) => value.clone(), // raw KV3 array content
            }
        }
    }
}

impl KV3Serialize for PulseVariable {
    fn serialize(&self) -> String {
        // convert default values to KV3 literal string.
        let literal = match &self.typ_and_default_value {
            PulseValueType::PVAL_STRING(value) => {
                if let Some(value) = value {
                    format!("\"{value}\"")
                } else {
                    String::from("")
                }
            }
            PulseValueType::PVAL_RESOURCE(_, value) => {
                if let Some(value) = value {
                    format!("\"{value}\"")
                } else {
                    String::from("")
                }
            }
            PulseValueType::PVAL_FLOAT(value) 
            | PulseValueType::PVAL_GAMETIME(value) => format!("{:.6}", value.unwrap_or_default()),
            PulseValueType::PVAL_INT(value) => format!("{:?}", value.unwrap_or_default()),
            PulseValueType::PVAL_TYPESAFE_INT(_, value) => {
                format!("{:?}", value.unwrap_or_default())
            }
            PulseValueType::PVAL_VEC2(value) => {
                let val = value.unwrap_or_default();
                format!("[{:.6}, {:.6}]", val.x, val.y)
            }
            PulseValueType::PVAL_VEC3(value)
            | PulseValueType::PVAL_VEC3_LOCAL(value)
            | PulseValueType::PVAL_QANGLE(value) => {
                let val = value.unwrap_or_default();
                format!("[{:.6}, {:.6}, {:.6}]", val.x, val.y, val.z)
            }
            PulseValueType::PVAL_VEC4(value) => {
                let val = value.unwrap_or_default();
                format!("[{:.6}, {:.6}, {:.6}, {:.6}]", val.x, val.y, val.z, val.w)
            }
            PulseValueType::PVAL_COLOR_RGB(value) => {
                let val = value.unwrap_or_default();
                format!("[{}, {}, {}]", val.x, val.y, val.z)
            }
            
            PulseValueType::PVAL_BOOL_VALUE(value) => value.unwrap_or_default().to_string(),
            PulseValueType::PVAL_BOOL => String::from("false"), // default value for bool is false
            PulseValueType::PVAL_SNDEVT_NAME(val) => val.clone().unwrap_or_default(),
            PulseValueType::PVAL_SCHEMA_ENUM(en) => {
                format!("\"{}\"", en.to_str())
            }

            PulseValueType::PVAL_TRANSFORM(_)
            | PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_)
            | PulseValueType::PVAL_EHANDLE(_) // can't have a default value for ehandle
            | PulseValueType::DOMAIN_ENTITY_NAME
            | PulseValueType::PVAL_INVALID
            | PulseValueType::PVAL_SNDEVT_GUID(_)
            | PulseValueType::PVAL_ACT
            | PulseValueType::PVAL_ANY
            | PulseValueType::PVAL_ARRAY(_) => String::from("null"), // Any type doesn't have a default value
        };
        formatdoc! {"
            {{
                m_Name = \"{}\"
                m_Description = \"\"
                m_Type = \"{}\"
                m_DefaultValue = {}
                m_nKeysSource = \"PRIVATE\"
                m_bIsPublic = true
                m_bIsPublicBlackboardVariable = false
                m_bIsObservable = false
                m_nEditorNodeID = -1
            }}
            "
            , self.name, self.typ_and_default_value.to_string(), literal
        }
    }
}

impl KV3Serialize for OutflowConnection {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_SourceOutflowName = \"{}\"
                m_nDestChunk = {}
                m_nInstruction = {}
                {}
            }}
            "
            , self.outflow_name, self.dest_chunk, self.dest_instruction
            , if let Some(register_map) = &self.register_map {
                format!("m_OutflowRegisterMap = {}", register_map.serialize())
            } else {
                String::default()
            }
        }
    }
}

impl KV3Serialize for CPulseCell_Outflow_IntSwitch {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Outflow_IntSwitch\"
                m_nEditorNodeID = -1
                m_DefaultCaseOutflow = {}
                m_CaseOutflows = 
                [
                    {}
                ]
            }}
            "
            , self.default_outflow.serialize()
            , self.ouflows.iter().map(|outflow| outflow.serialize()).collect::<Vec<String>>().join(",\n\n")
        }
    }
}

impl KV3Serialize for SoundEventStartType {
    fn serialize(&self) -> String {
        match self {
            SoundEventStartType::SOUNDEVENT_START_PLAYER => "SOUNDEVENT_START_PLAYER".into(),
            SoundEventStartType::SOUNDEVENT_START_WORLD => "SOUNDEVENT_START_WORLD".into(),
            SoundEventStartType::SOUNDEVENT_START_ENTITY => "SOUNDEVENT_START_ENTITY".into(),
        }
    }
}

impl KV3Serialize for CPulseCell_SoundEventStart {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_SoundEventStart\"
                m_nEditorNodeID = -1
                m_Type = \"{}\"
            }}
            "
            , self.typ.serialize()
        }
    }
}

pub struct CallInfo {
    pub port_name: Cow<'static, str>,
    pub register_map: RegisterMap,
    pub call_method_id: i32, // also used for debugging only?
    pub src_chunk: i32,
    pub src_instruction: i32,
}

impl KV3Serialize for CallInfo {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_PortName = \"{}\"
                m_nEditorNodeID = -1
                m_RegisterMap = {}
                m_nCallMethodID = {}
                m_nSrcChunk = {}
                m_nSrcInstruction = {}
            }}
            ",
            self.port_name,
            self.register_map.serialize(),
            self.call_method_id, self.src_chunk,
            self.src_instruction
        }
    }
}

impl KV3Serialize for CPulseCell_Outflow_ListenForEntityOutput {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Outflow_ListenForEntityOutput\"
                m_nEditorNodeID = -1
                m_OnFired = {}
                m_OnCanceled = {}
                m_strEntityOutput = \"{}\"
                m_strEntityOutputParam = \"{}\"
                m_bListenUntilCanceled = {}
            }}
            "
            , self.outflow_onfired.serialize()
            , self.outflow_oncanceled.serialize()
            , self.entity_output, self.entity_output_param, self.listen_until_canceled
        }
    }
}

impl KV3Serialize for TimelineEvent {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                m_flTimeFromPrevious = {:.6}
                m_bPauseForPreviousEvents = {:.6}
                m_bCallModeSync = {}
                m_EventOutflow = {}
            }}
            "
            , self.time_from_previous, self.pause_for_previous_events, self.call_mode_sync, self.event_outflow.serialize()
        }
    }
}

impl KV3Serialize for CPulseCell_Timeline {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Timeline\"
                m_nEditorNodeID = -1
                m_OnFinished = {}
                m_bWaitForChildOutflows = {}
                m_TimelineEvents = 
                [
                    {}
                ]
            }}
            "
            , self.outflow_onfinished.serialize()
            , self.wait_for_child_outflows
            , self.timeline_events.iter().map(|event| event.serialize()).collect::<Vec<String>>().join(",\n\n")
        }
    }
}

impl KV3Serialize for CPulseCell_Step_SetAnimGraphParam {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {{
                _class = \"CPulseCell_Step_SetAnimGraphParam\"
                m_nEditorNodeID = -1
                m_ParamName = \"{}\"
            }}
            "
            , self.param_name
        }
    }
}

#[derive(Default)]
pub struct PulseGraphDef {
    mapped_registers_outputs: SecondaryMap<OutputId, i32>,
    mapped_registers_inputs: SecondaryMap<InputId, i32>,
    mapped_registers_node_outputs: SecondaryMap<NodeId, SecondaryMap<OutputId, i32>>,
    pub traversed_entrypoints: Vec<(NodeId, i32)>, // used to track which entrypoints have been traversed
    pub cells: Vec<Box<dyn PulseCellTrait>>,
    pub constants: Vec<PulseConstant>,
    pub bindings: Vec<InvokeBinding>,
    pub chunks: Vec<PulseChunk>,
    pub output_connections: Vec<OutputConnection>,
    pub domain_values: Vec<DomainValue>,
    pub public_outputs: Vec<OutputDefinition>,
    pub variables: Vec<PulseVariable>,
    pub call_infos: Vec<CallInfo>,
    pub map_name: String,
    pub xml_name: String,
}
impl PulseGraphDef {
    pub fn create_chunk(&mut self) -> i32 {
        let chunk = PulseChunk::default();
        self.chunks.push(chunk);
        self.chunks.len() as i32 - 1
    }
    pub fn create_domain_value(
        &mut self,
        typ: Cow<'static, str>,
        value: Cow<'static, str>,
        __deprecated_expected_runtime_type: Cow<'static, str>,
        required_runtime_type: Cow<'static, str>,
    ) -> i32 {
        let domain_value = DomainValue {
            typ,
            value,
            __deprecated_expected_runtime_type,
            required_runtime_type,
        };
        self.domain_values.push(domain_value);
        self.domain_values.len() as i32 - 1
    }
    pub fn add_invoke_binding(&mut self, binding: InvokeBinding) -> i32 {
        self.bindings.push(binding);
        self.bindings.len() as i32 - 1
    }
    pub fn add_constant(&mut self, constant: PulseConstant) -> i32 {
        // try to find existing constant (TODO: hashing constants, and caching the hashes to compare would be faster)
        // if self.constants.iter().any(|c| **c == constant) {
        //     return self.constants.iter().position(|c| **c == constant).unwrap() as i32;
        // }
        self.constants.push(constant);
        self.constants.len() as i32 - 1
    }
    pub fn add_output_connection(&mut self, output_connection: OutputConnection) {
        self.output_connections.push(output_connection);
    }
    pub fn get_mapped_reigster(&self, output_id: OutputId) -> Option<&i32> {
        self.mapped_registers_outputs.get(output_id)
    }
    pub fn add_register_mapping(&mut self, output_id: OutputId, register_id: i32) {
        self.mapped_registers_outputs.insert(output_id, register_id);
    }
    pub fn get_mapped_reigster_input(&self, input_id: InputId) -> Option<&i32> {
        self.mapped_registers_inputs.get(input_id)
    }
    pub fn add_register_mapping_input(&mut self, input_id: InputId, register_id: i32) {
        self.mapped_registers_inputs.insert(input_id, register_id);
    }
    pub fn get_mapped_register_node_outputs(&self, node_id: NodeId, output_id: OutputId) -> Option<&i32> {
        self.mapped_registers_node_outputs.get(node_id)
            .and_then(|map| map.get(output_id))
    }
    pub fn add_register_mapping_node_outputs(&mut self, node_id: NodeId, output_map: SecondaryMap<OutputId, i32>) {
        self.mapped_registers_node_outputs.insert(node_id, output_map);
    }
    pub fn get_current_constant_id(&self) -> i32 {
        self.constants.len() as i32 - 1
    }
    pub fn get_current_domain_val_id(&self) -> i32 {
        self.domain_values.len() as i32 - 1
    }
    pub fn get_current_binding_id(&self) -> i32 {
        self.bindings.len() as i32 - 1
    }
    pub fn get_variable_index(&self, name: &str) -> Option<usize> {
        self.variables
            .iter()
            .position(|variable| variable.name == name)
    }
    pub fn get_public_output_index(&self, name: &str) -> Option<usize> {
        self.public_outputs
            .iter()
            .position(|output| output.name == name)
    }
    pub fn get_chunk_last_instruction_id(&self, chunk_id: i32) -> i32 {
        if let Some(chunk) = self.chunks.get(chunk_id as usize) {
            chunk.get_last_instruction_id()
        } else {
            -1
        }
    }
    pub fn add_call_info(&mut self, call_info: CallInfo) -> i32 {
        self.call_infos.push(call_info);
        self.call_infos.len() as i32 - 1
    }
    pub fn add_cell(&mut self, cell: Box<dyn PulseCellTrait>) -> usize {
        self.cells.push(cell);
        self.cells.len() - 1
    }
    pub fn get_last_cell_id(&self) -> usize {
        self.cells.len() - 1
    }
    pub fn get_invoke_binding_mut(&mut self, index: i32) -> Option<&mut InvokeBinding> {
        self.bindings.get_mut(index as usize)
    }
}

#[cfg(feature = "nongame_asset_build")]
const PULSE_KV3_HEADER: &str = "<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:vpulse12:version{354e36cb-dbe4-41c0-8fe3-2279dd194022} -->\n";
#[cfg(not(feature = "nongame_asset_build"))]
const PULSE_KV3_HEADER: &str = "<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:generic:version{7412167c-06e9-4698-aff2-e63eb59037e7} -->\n";
impl KV3Serialize for PulseGraphDef {
    fn serialize(&self) -> String {
        formatdoc! {
            "
            {PULSE_KV3_HEADER}
            {{
                generic_data_type = \"Vpulse\"
                m_Cells = 
                [
                    {}
                ]
                m_DomainIdentifier = \"ServerEntity\"
                m_DomainSubType = \"PVAL_EHANDLE:point_pulse\"
                m_ParentMapName = \"{}\"
                m_ParentXmlName = \"{}\"
                m_vecGameBlackboards = []
                m_BlackboardReferences = []
                m_Chunks = 
                [
                    {}
                ]
                m_DomainValues = 
                [
                    {}
                ]
                m_Vars =
                [
                    {}
                ]
                m_Constants = 
                [
                    {}
                ]
                m_PublicOutputs = 
                [
                    {}
                ]
                m_OutputConnections = 
                [
                    {}
                ]
                m_InvokeBindings = 
                [
                    {}
                ]
                m_CallInfos = 
                [
                    {}
                ]
            }}
            "
            , self.cells.iter().map(|cell| {
                cell.serialize()
            }).collect::<Vec<_>>().join(",\n\n")
            , self.map_name, self.xml_name
            , self.chunks.iter().map(|chunk| chunk.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.domain_values.iter().map(|domain_value| domain_value.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.variables.iter().map(|variable| variable.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.constants.iter().map(|constant| constant.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.public_outputs.iter().map(|variable| variable.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.output_connections.iter().map(|output_connection| output_connection.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.bindings.iter().map(|binding| binding.serialize()).collect::<Vec<_>>().join(",\n\n")
            , self.call_infos.iter().map(|callinfo| callinfo.serialize()).collect::<Vec<_>>().join(",\n\n")
        }
    }
}
