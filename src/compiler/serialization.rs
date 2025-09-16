#![allow(non_camel_case_types)]
#![allow(nonstandard_style)]

use kv3::{Metadata, ObjectKey, Value};
use std::borrow::Cow;
use egui_node_graph2::{InputId, NodeId, OutputId};
use slotmap::SecondaryMap;
use crate::{
    pulsetypes::*,
    typing::{PulseValueType, Vec2, Vec3, Vec4},
};

pub trait KV3Serialize {
    fn serialize(&self) -> Value;
}
pub struct PulseRuntimeArgument {
    pub name: String,
    pub description: String,
    pub typ: String,
}

impl KV3Serialize for PulseRuntimeArgument {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_Name".into()), Value::String(self.name.clone())),
            (ObjectKey::Identifier("m_Description".into()), Value::String(self.description.clone())),
            (ObjectKey::Identifier("m_Type".into()), Value::String(self.typ.clone())),
        ])
    }
}

impl KV3Serialize for CPulseCell_Inflow_Method {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Inflow_Method".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_EntryChunk".into()), Value::Number(self.entry_chunk.into())),
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_MethodName".into()), Value::String(self.name.clone())),
            (ObjectKey::Identifier("m_Description".into()), Value::String(self.description.clone())),
            (ObjectKey::Identifier("m_bIsPublic".into()), Value::Bool(true)),
            (ObjectKey::Identifier("m_ReturnType".into()), Value::String("PVAL_VOID".into())),
            (ObjectKey::Identifier("m_Args".into()), Value::Array(self.args.iter().map(|arg| arg.serialize()).collect())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Inflow_EventHandler".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_EntryChunk".into()), Value::Number(self.entry_chunk.into())),
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_EventName".into()), Value::String(self.event_name.to_string())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Inflow_Wait".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_WakeResume".into()), Value::Object(vec![
                (ObjectKey::Identifier("m_SourceOutflowName".into()), Value::String("m_WakeResume".into())),
                (ObjectKey::Identifier("m_nDestChunk".into()), Value::Number(self.dest_chunk.into())),
                (ObjectKey::Identifier("m_nInstruction".into()), Value::Number(self.instruction.into())),
            ])),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Step_EntFire".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_Input".into()), Value::String(self.input.to_string())),
        ])
    }
}

impl KV3Serialize for CPulseCell_Step_DebugLog {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Step_DebugLog".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
        ])
    }
}

impl KV3Serialize for CPulseCell_Step_PublicOutput {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Step_PublicOutput".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_OutputIndex".into()), Value::Number(self.output_idx.into()))
        ])
    }
}

impl KV3Serialize for CPulseCell_Inflow_GraphHook {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Inflow_GraphHook".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_EntryChunk".into()), Value::Number(self.entry_chunk.into())),
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_HookName".into()), Value::String(self.hook_name.to_string())),
        ])
    }
}

impl KV3Serialize for Register {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_nReg".into()), Value::Number(self.num.into())),
            (ObjectKey::Identifier("m_Type".into()), Value::String(self.reg_type.clone())),
            (ObjectKey::Identifier("m_OriginName".into()), Value::String("0:null".into())),
            (ObjectKey::Identifier("m_nWrittenByInstruction".into()), Value::Number(self.written_by_instruction.into())),
            (ObjectKey::Identifier("m_nLastReadByInstruction".into()), Value::Number(-1f64)),
        ])
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
    fn serialize(&self) -> Value {
        fn map_params(params: &Vec<(Cow<'_, str>, i32)>) -> Value {
            let param_list = params
                .iter()
                .map(|(name, num)| 
                    (ObjectKey::Identifier(name.to_string()), Value::Number((*num).into())))
                .collect::<Vec<(ObjectKey, Value)>>();
            Value::Object(param_list)
        }
        let inparams_value: Value = if !self.inparams.is_empty() {
            map_params(&self.inparams)
        } else {
            Value::Null
        };
        let outparams_value: Value = if !self.outparams.is_empty() {
            map_params(&self.outparams)
        } else {
            Value::Null
        };
        Value::Object(vec![
            (ObjectKey::Identifier("m_Inparams".into()), inparams_value),
            (ObjectKey::Identifier("m_Outparams".into()), outparams_value),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_nCode".into()), Value::String(self.code.clone())),
            (ObjectKey::Identifier("m_nVar".into()), Value::Number(self.var.into())),
            (ObjectKey::Identifier("m_nReg0".into()), Value::Number(self.reg0.into())),
            (ObjectKey::Identifier("m_nReg1".into()), Value::Number(self.reg1.into())),
            (ObjectKey::Identifier("m_nReg2".into()), Value::Number(self.reg2.into())),
            (ObjectKey::Identifier("m_nInvokeBindingIndex".into()), Value::Number(self.invoke_binding_index.into())),
            (ObjectKey::Identifier("m_nChunk".into()), Value::Number(self.chunk.into())),
            (ObjectKey::Identifier("m_nDestInstruction".into()), Value::Number(self.dest_instruction.into())),
            (ObjectKey::Identifier("m_nCallInfoIndex".into()), Value::Number(self.call_info_index.into())),
            (ObjectKey::Identifier("m_nConstIdx".into()), Value::Number(self.const_idx.into())),
            (ObjectKey::Identifier("m_nDomainValueIdx".into()), Value::Number(self.domain_value_idx.into())),
            (ObjectKey::Identifier("m_nBlackboardReferenceIdx".into()), Value::Number(self.blackboard_reference_idx.into())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_Instructions".into()), Value::Array(self.instructions.iter().map(|instruction| instruction.serialize()).collect())),
            (ObjectKey::Identifier("m_Registers".into()), Value::Array(self.registers.iter().map(|register| register.serialize()).collect())),
            (ObjectKey::Identifier("m_InstructionEditorIDs".into()), Value::Array(self.instruction_editor_ids.iter().map(|id| Value::Number((*id).into())).collect())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_FuncName".into()), Value::String(self.func_name.to_string())),
            (ObjectKey::Identifier("m_nCellIndex".into()), Value::Number(self.cell_index.into())),
            (ObjectKey::Identifier("m_nSrcChunk".into()), Value::Number(self.src_chunk.into())),
            (ObjectKey::Identifier("m_nSrcInstruction".into()), Value::Number(self.src_instruction.into())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_nType".into()), Value::String(self.typ.to_string())),
            (ObjectKey::Identifier("m_Value".into()), Value::String(self.value.to_string())),
            (ObjectKey::Identifier("m_RequiredRuntimeType".into()), Value::String(self.required_runtime_type.to_string())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_SourceOutput".into()), Value::String(self.source_output.clone())),
            (ObjectKey::Identifier("m_TargetEntity".into()), Value::String(self.target_entity.clone())),
            (ObjectKey::Identifier("m_TargetInput".into()), Value::String(self.target_input.clone())),
            (ObjectKey::Identifier("m_Param".into()), Value::String(self.param.clone())),
        ])
    }
}

impl KV3Serialize for OutputDefinition {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_Name".into()), Value::String(self.name.clone())),
            (ObjectKey::Identifier("m_Description".into()), Value::String("".into())),
            (ObjectKey::Identifier("m_ParamType".into()), Value::String(self.typ.to_string())),
        ])
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
    Array(PulseValueType, Vec<PulseConstant>), // raw KV3 array content
}
impl PulseConstant {
    fn serialize_value(&self) -> Value {
        match self {
            PulseConstant::String(value) => Value::String(value.clone()),
            PulseConstant::SoundEventName(value) => Value::Flag("soundevent".into(), Value::String(value.clone()).into()),
            PulseConstant::Float(value) => Value::Number((*value).into()),
            PulseConstant::Integer(value) => Value::Number((*value).into()),
            PulseConstant::Vec2(value) => Value::Array(vec![Value::Number(value.x.into()), Value::Number(value.y.into())]),
            PulseConstant::Vec3(value)
            | PulseConstant::Vec3Local(value)
            | PulseConstant::QAngle(value) 
                => Value::Array(vec![Value::Number(value.x.into()), Value::Number(value.y.into()), Value::Number(value.z.into())]),
            PulseConstant::Vec4(value) 
                => Value::Array(vec![Value::Number(value.x.into()), Value::Number(value.y.into()), Value::Number(value.z.into()), Value::Number(value.w.into())]),
            PulseConstant::Color_RGB(value) 
                => Value::Array(vec![Value::Number(value[0].into()), Value::Number(value[1].into()), Value::Number(value[2].into())]),
            PulseConstant::Bool(value) => Value::Bool(*value),
            PulseConstant::SchemaEnum(_, value) => Value::String(value.to_str().to_string()),
            PulseConstant::Resource(_, value) => Value::Flag("resource".into(), Value::String(value.clone()).into()),
            PulseConstant::Array(_, value) => {
                let values = value.iter().map(|v| v.serialize_value()).collect();
                Value::Array(values)
            }
        }
    }
}
impl KV3Serialize for PulseConstant {
    fn serialize(&self) -> Value {
        let typ_str =  match self {
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
        };
        let val = self.serialize_value();
        Value::Object(vec![
            (ObjectKey::Identifier("m_Type".into()), Value::String(typ_str)),
            (ObjectKey::Identifier("m_Value".into()), val),
        ])
    }
}

impl KV3Serialize for PulseVariable {
    fn serialize(&self) -> Value {
        // convert default values to KV3 literal string.
        let default_value = match &self.typ_and_default_value {
            PulseValueType::PVAL_STRING(value) => {
                if let Some(value) = value {
                    Value::String(value.clone())
                } else {
                    Value::Null
                }
            }
            PulseValueType::PVAL_RESOURCE(_, value) => {
                if let Some(value) = value {
                    Value::Flag("resource".to_string(), Value::String(value.clone()).into())
                } else {
                    Value::Null
                }
            }
            PulseValueType::PVAL_FLOAT(value) 
            | PulseValueType::PVAL_GAMETIME(value) => Value::Number(value.unwrap_or_default().into()),
            PulseValueType::PVAL_INT(value) => Value::Number(value.unwrap_or_default().into()),
            PulseValueType::PVAL_TYPESAFE_INT(_, value) => {
                Value::Number(value.unwrap_or_default().into())
            }
            PulseValueType::PVAL_VEC2(value) => {
                let val = value.unwrap_or_default();
                Value::Array(vec![Value::Number(val.x.into()), Value::Number(val.y.into())])
            }
            PulseValueType::PVAL_VEC3(value)
            | PulseValueType::PVAL_VEC3_LOCAL(value)
            | PulseValueType::PVAL_QANGLE(value) => {
                let val = value.unwrap_or_default();
                Value::Array(vec![Value::Number(val.x.into()), Value::Number(val.y.into()), Value::Number(val.z.into())])
            }
            PulseValueType::PVAL_VEC4(value) => {
                let val = value.unwrap_or_default();
                Value::Array(vec![Value::Number(val.x.into()), Value::Number(val.y.into()), Value::Number(val.z.into()), Value::Number(val.w.into())])
            }
            PulseValueType::PVAL_COLOR_RGB(value) => {
                let val = value.unwrap_or_default();
                Value::Array(vec![Value::Number(val.x.into()), Value::Number(val.y.into()), Value::Number(val.z.into())])
            }
            PulseValueType::PVAL_BOOL_VALUE(value) 
                => Value::Bool(value.unwrap_or_default()),
            PulseValueType::PVAL_BOOL => Value::Bool(false), // default value for bool is false
            PulseValueType::PVAL_SNDEVT_NAME(val) 
                => Value::Flag("soundevent".into(), Value::String(val.clone().unwrap_or_default()).into()),
            PulseValueType::PVAL_SCHEMA_ENUM(en) 
                => Value::String(en.to_str().to_string()),

            PulseValueType::PVAL_TRANSFORM(_)
            | PulseValueType::PVAL_TRANSFORM_WORLDSPACE(_)
            | PulseValueType::PVAL_EHANDLE(_) // can't have a default value for ehandle
            | PulseValueType::DOMAIN_ENTITY_NAME
            | PulseValueType::PVAL_INVALID
            | PulseValueType::PVAL_SNDEVT_GUID(_)
            | PulseValueType::PVAL_ACT
            | PulseValueType::PVAL_ANY
            | PulseValueType::PVAL_ARRAY(_) => Value::Null,
            _ => Value::Null, // Other types don't have a default value
        };
        Value::Object(vec![
            (ObjectKey::Identifier("m_Name".into()), Value::String(self.name.clone())),
            (ObjectKey::Identifier("m_Description".into()), Value::String("".into())),
            (ObjectKey::Identifier("m_Type".into()), Value::String(self.typ_and_default_value.to_string())),
            (ObjectKey::Identifier("m_DefaultValue".into()), default_value),
            (ObjectKey::Identifier("m_nKeysSource".into()), Value::String("PRIVATE".into())),
            (ObjectKey::Identifier("m_bIsPublic".into()), Value::Bool(true)),
            (ObjectKey::Identifier("m_bIsPublicBlackboardVariable".into()), Value::Bool(false)),
            (ObjectKey::Identifier("m_bIsObservable".into()), Value::Bool(false)),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64))
        ])
    }
}

impl KV3Serialize for OutflowConnection {
    fn serialize(&self) -> Value {
        let mut params = vec![
            (ObjectKey::Identifier("m_SourceOutflowName".into()), Value::String(self.outflow_name.to_string())),
            (ObjectKey::Identifier("m_nDestChunk".into()), Value::Number(self.dest_chunk.into())),
            (ObjectKey::Identifier("m_nInstruction".into()), Value::Number(self.dest_instruction.into())),
        ];
        if let Some(register_map) = &self.register_map {
            params.push((ObjectKey::Identifier("m_OutflowRegisterMap".into()), register_map.serialize()));   
        }
        Value::Object(params)
    }
}

impl KV3Serialize for CPulseCell_Outflow_IntSwitch {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Outflow_IntSwitch".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_DefaultCaseOutflow".into()), self.default_outflow.serialize()),
            (ObjectKey::Identifier("m_CaseOutflows".into()), Value::Array(self.ouflows.iter().map(|outflow| outflow.serialize()).collect())),
        ])
    }
}

impl KV3Serialize for CPulseCell_SoundEventStart {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_SoundEventStart".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_Type".into()), Value::String(self.typ.to_str().to_string())),
        ])
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
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_PortName".into()), Value::String(self.port_name.to_string())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_nCallMethodID".into()), Value::Number(self.call_method_id.into())),
            (ObjectKey::Identifier("m_nSrcChunk".into()), Value::Number(self.src_chunk.into())),
            (ObjectKey::Identifier("m_nSrcInstruction".into()), Value::Number(self.src_instruction.into())),
        ])
    }
}

impl KV3Serialize for CPulseCell_Outflow_ListenForEntityOutput {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Outflow_ListenForEntityOutput".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_OnFired".into()), self.outflow_onfired.serialize()),
            (ObjectKey::Identifier("m_OnCanceled".into()), self.outflow_oncanceled.serialize()),
            (ObjectKey::Identifier("m_strEntityOutput".into()), Value::String(self.entity_output.clone())),
            (ObjectKey::Identifier("m_strEntityOutputParam".into()), Value::String(self.entity_output_param.clone())),
            (ObjectKey::Identifier("m_bListenUntilCanceled".into()), Value::Bool(self.listen_until_canceled)),
        ])
    }
}

impl KV3Serialize for TimelineEvent {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("m_flTimeFromPrevious".into()), Value::Number(self.time_from_previous.into())),
            (ObjectKey::Identifier("m_bPauseForPreviousEvents".into()), Value::Number(self.pause_for_previous_events.into())),
            (ObjectKey::Identifier("m_bCallModeSync".into()), Value::Bool(self.call_mode_sync)),
            (ObjectKey::Identifier("m_EventOutflow".into()), self.event_outflow.serialize()),
        ])
    }
}

impl KV3Serialize for CPulseCell_Timeline {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Timeline".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_OnFinished".into()), self.outflow_onfinished.serialize()),
            (ObjectKey::Identifier("m_bWaitForChildOutflows".into()), Value::Bool(self.wait_for_child_outflows)),
            (ObjectKey::Identifier("m_TimelineEvents".into()), Value::Array(self.timeline_events.iter().map(|event| event.serialize()).collect())),
        ])
    }
}

impl KV3Serialize for CPulseCell_Step_SetAnimGraphParam {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Step_SetAnimGraphParam".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_ParamName".into()), Value::String(self.param_name.to_string())),
        ])
    }
}

impl KV3Serialize for CPulseCell_Value_RandomInt {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Value_RandomInt".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
        ])
    }
}

impl KV3Serialize for CPulseCell_Value_RandomFloat {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Value_RandomFloat".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
        ])
    }
}

impl KV3Serialize for CPulseCell_Inflow_EntOutputHandler {
    fn serialize(&self) -> Value {
        Value::Object(vec![
            (ObjectKey::Identifier("_class".into()), Value::String("CPulseCell_Inflow_EntOutputHandler".into())),
            (ObjectKey::Identifier("m_nEditorNodeID".into()), Value::Number(-1f64)),
            (ObjectKey::Identifier("m_EntryChunk".into()), Value::Number(self.entry_chunk.into())),
            (ObjectKey::Identifier("m_RegisterMap".into()), self.register_map.serialize()),
            (ObjectKey::Identifier("m_SourceEntity".into()), Value::String(self.source_entity.clone())),
            (ObjectKey::Identifier("m_SourceOutput".into()), Value::String(self.source_output.clone())),
            (ObjectKey::Identifier("m_ExpectedParamType".into()), Value::String(self.expected_param_type.to_string())),
        ])
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
    pub graph_domain: String,
    pub graph_subtype: String,
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
    pub fn add_chunk_instruction(
        &mut self,
        chunk_id: usize,
        instruction: Instruction,
    ) -> Option<i32> {
        self.chunks.get_mut(chunk_id)
            .map(|chunk| chunk.add_instruction(instruction))
    }
    // Adds a new register to the specified chunk, and returns the register ID. If written_by_instruction is None, it will use the last instruction ID of the chunk + 1.
    pub fn add_chunk_register(
        &mut self,
        chunk_id: usize,
        reg_type: String,
        written_by_instruction: Option<i32>,
    ) -> Option<i32> {
        self.chunks.get_mut(chunk_id)
            .map(|chunk| chunk.add_register(
                reg_type, written_by_instruction.unwrap_or(chunk.get_last_instruction_id() + 1)
            ))
    }
}

impl KV3Serialize for PulseGraphDef {
    fn serialize(&self) -> Value {
        Value::File(kv3::Header(
            vec![
                Metadata {
                    key: "encoding".into(),
                    value: "text".into(),
                    version: "e21c7f3c-8a33-41c5-9977-a76d3a32aa0d".into()
                },
                Metadata {
                    key: "format".into(),
                    value: "generic".into(),
                    version: "7412167c-06e9-4698-aff2-e63eb59037e7".into()
                },
            ],
        ),
            Box::new(Value::Object(vec![
                (ObjectKey::Identifier("m_Cells".into()), Value::Array(self.cells.iter().map(|cell| cell.serialize()).collect())),
                (ObjectKey::Identifier("m_DomainIdentifier".into()), Value::String(self.graph_domain.to_string())),
                (ObjectKey::Identifier("m_DomainSubType".into()), Value::String(self.graph_subtype.to_string())),
                (ObjectKey::Identifier("m_ParentMapName".into()), Value::String(self.map_name.to_string())),
                (ObjectKey::Identifier("m_ParentXmlName".into()), Value::String(self.xml_name.to_string())),
                (ObjectKey::Identifier("m_vecGameBlackboards".into()), Value::Array(vec![])),
                (ObjectKey::Identifier("m_BlackboardReferences".into()), Value::Array(vec![])),
                (ObjectKey::Identifier("m_Chunks".into()), Value::Array(self.chunks.iter().map(|chunk| chunk.serialize()).collect())),
                (ObjectKey::Identifier("m_DomainValues".into()), Value::Array(self.domain_values.iter().map(|domain_value| domain_value.serialize()).collect())),
                (ObjectKey::Identifier("m_Vars".into()), Value::Array(self.variables.iter().map(|variable| variable.serialize()).collect())),
                (ObjectKey::Identifier("m_Constants".into()), Value::Array(self.constants.iter().map(|constant| constant.serialize()).collect())),
                (ObjectKey::Identifier("m_PublicOutputs".into()), Value::Array(self.public_outputs.iter().map(|variable| variable.serialize()).collect())),
                (ObjectKey::Identifier("m_OutputConnections".into()), Value::Array(self.output_connections.iter().map(|output_connection| output_connection.serialize()).collect())),
                (ObjectKey::Identifier("m_InvokeBindings".into()), Value::Array(self.bindings.iter().map(|binding| binding.serialize()).collect())),
                (ObjectKey::Identifier("m_CallInfos".into()), Value::Array(self.call_infos.iter().map(|callinfo| callinfo.serialize()).collect())),
            ]))
        )
    }
}
