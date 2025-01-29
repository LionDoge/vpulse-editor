#![allow(non_camel_case_types)]
#![allow(nonstandard_style)]
use egui_node_graph2::{InputId, OutputId};
use indoc::formatdoc;
use slotmap::SecondaryMap;
use crate::pulsetypes::*;
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
        formatdoc!{
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
        formatdoc!{
            "
            {{
                _class = \"CPulseCell_Inflow_Method\"
                m_nEditorNodeID = -1
                m_EntryChunk = {}
                m_RegisterMap = {}
                m_MethodName = \"{}\"
                m_Description = \"{}\"
                m_bIsPublic = true
                m_ReturnType = \"PVAL_INVALID\"
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
        self.register_map.add_outparam(name, out_register);
    }
}

impl KV3Serialize for CPulseCell_Inflow_EventHandler {
    fn serialize(&self) -> String {
        formatdoc!{
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
    pub fn new(entry_chunk: i32, event_name: String) -> CPulseCell_Inflow_EventHandler {
        CPulseCell_Inflow_EventHandler {
            register_map: RegisterMap::default(),
            entry_chunk,
            event_name,
        }
    }
    pub fn add_outparam(&mut self, name: String, num: i32) {
        self.register_map.add_outparam(name, num);
    }
}

impl KV3Serialize for CPulseCell_Inflow_Wait {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                _class = \"CPulseCell_Inflow_Wait\"
                m_nEditorNodeID = -1
                m_WakeResume = 
                {{
                    m_SourceOutflowName = \"m_WakeResume\"
                    m_DestChunk = {}
                    m_Instruction = {}
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

impl CPulseCell_Step_EntFire {
    pub fn new(input: String) -> CPulseCell_Step_EntFire {
        CPulseCell_Step_EntFire {
            input: input,
        }
    }
}

impl KV3Serialize for CPulseCell_Step_EntFire {
    fn serialize(&self) -> String {
        formatdoc!{
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

impl CPulseCell_Value_FindEntByName {
    pub fn new(entity_type: String) -> CPulseCell_Value_FindEntByName {
        CPulseCell_Value_FindEntByName {
            entity_type,
        }
    }
}
impl KV3Serialize for CPulseCell_Value_FindEntByName {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                _class = \"CPulseCell_Value_FindEntByName\"
                m_nEditorNodeID = -1
                m_EntityType = \"{}\"
            }}
            "
            , self.entity_type
        }
    }
}

impl KV3Serialize for Register {
    fn serialize(&self) -> String {
        formatdoc!{
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

#[derive(Default)]
pub struct RegisterMap {
    pub inparams: Vec<(String, i32)>,
    pub outparams: Vec<(String, i32)>,
}
impl RegisterMap {
    pub fn add_inparam(&mut self, name: String, num: i32) {
        self.inparams.push((name, num));
    }
    pub fn add_outparam(&mut self, name: String, num: i32) {
        self.outparams.push((name, num));
    }
}

impl KV3Serialize for RegisterMap {
    fn serialize(&self) -> String {
        let inparams_str: String = if self.inparams.len() > 0 {
            formatdoc!(
                "{{
                    {}
                }}",
                self.inparams.iter().map(|(name, num)| format!("{} = {}", name, num)).collect::<Vec<String>>().join("\n"))
        } else {
            String::from("null")
        };
        let outparams_str: String = if self.outparams.len() > 0 {
            formatdoc!(
                "{{
                    {}
                }}",
                self.outparams.iter().map(|(name, num)| format!("{} = {}", name, num)).collect::<Vec<String>>().join("\n"))
        } else {
            String::from("null")
        };
        formatdoc!{
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
        formatdoc!{
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
}
impl KV3Serialize for PulseChunk {
    fn serialize(&self) -> String {
        formatdoc!{
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
    pub func_name: String,
    pub cell_index: i32,
    pub src_chunk: i32,
    pub src_instruction: i32
}

impl KV3Serialize for InvokeBinding {
    fn serialize(&self) -> String {
        formatdoc!{
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
    pub typ: String,
    pub value: String,
    pub expected_runtime_type: String,
}
impl KV3Serialize for DomainValue {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                m_nType = \"{}\"
                m_Value = \"{}\"
                m_ExpectedRuntimeType = \"{}\"
            }}
            "
            , self.typ, self.value, self.expected_runtime_type
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
    pub fn new(source_output: String, target_entity: String, target_input: String, param: String) -> OutputConnection {
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
        formatdoc!{
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
pub enum PulseConstant {
    String(String),
    Float(f32),
    Integer(i32),
    Vec3(crate::app::Vec3),
    Color_RGB(crate::app::Vec3),
    Bool(bool),
}
impl KV3Serialize for PulseConstant {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                m_Type = \"{}\"
                m_Value = {}
            }}
            "
            , 
            match self {
                PulseConstant::String(_) => "PVAL_STRING",
                PulseConstant::Float(_) => "PVAL_FLOAT",
                PulseConstant::Integer(_) => "PVAL_INT",
                PulseConstant::Vec3(_) => "PVAL_VEC3",
                PulseConstant::Color_RGB(_) => "PVAL_COLOR_RGB",
                PulseConstant::Bool(_) => "PVAL_BOOL",
            },
            match self {
                PulseConstant::String(value) => format!("\"{}\"", value),
                PulseConstant::Float(value) => format!("{:.8}", value),
                PulseConstant::Integer(value) => value.to_string(),
                PulseConstant::Vec3(value) => format!("[{}, {}, {}]", value.x, value.y, value.z),
                PulseConstant::Color_RGB(value) => format!("[{}, {}, {}]", value.x, value.y, value.z),
                PulseConstant::Bool(value) => value.to_string(),
            }
        }
    }
}

pub struct Variable {
    pub name: String,
    pub typ: String,
    pub default_value: i32,
}

impl KV3Serialize for Variable {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                m_Name = \"{}\"
                m_Description = \"\"
                m_Type = \"{}\"
                m_DefaultValue = {}
                m_bIsPublic = true
                m_bIsObservable = false
                m_nEditorNodeID = -1
            }}
            "
            , self.name, self.typ, self.default_value
        }
    }
}
impl KV3Serialize for PulseVariable {
    fn serialize(&self) -> String {
        let literal = match &self.typ_and_default_value {
            PulseValueType::PVAL_STRING(value) => format!("\"{}\"", value.clone().unwrap_or_default()),
            PulseValueType::PVAL_FLOAT(value) => format!("{:.8}", value.unwrap_or_default()),
            PulseValueType::PVAL_INT(value) => format!("{:?}", value.unwrap_or_default()),
            PulseValueType::PVAL_VEC3(value) => {
                let val = value.unwrap_or_default();
                format!("[{}, {}, {}]", val.x, val.y, val.z)
            }
            PulseValueType::PVAL_COLOR_RGB(value) => {
                let val = value.clone().unwrap_or_default();
                format!("[{}, {}, {}]", val.x, val.y, val.z)
            }
            PulseValueType::PVAL_EHANDLE(_) => String::from("null"), // can't have a default value for ehandle
            PulseValueType::DOMAIN_ENTITY_NAME => String::from("null"),
            PulseValueType::PVAL_INVALID => String::from("PVAL_INVALID"),
        };
        formatdoc!{"
            {{
                m_Name = \"{}\"
                m_Description = \"\"
                m_Type = \"{}\"
                m_DefaultValue = {}
                m_bIsPublic = true
                m_bIsObservable = false
                m_nEditorNodeID = -1
            }}
            "
            , self.name, self.typ_and_default_value.to_string(), literal
        }
    }
}
impl Variable{
    pub fn new<'a>(name: String, typ: String, default_value: i32) -> Variable {
        Variable {
            name,
            typ,
            default_value,
        }
    }
}

#[derive(Default)]
pub struct PulseGraphDef {
    mapped_registers_outputs: SecondaryMap<OutputId, i32>,
    mapped_registers_inputs: SecondaryMap<InputId, i32>,
    pub cells: Vec<Box<CellType>>,
    pub constants: Vec<Box<PulseConstant>>,
    pub bindings: Vec<InvokeBinding>,
    pub chunks: Vec<PulseChunk>,
    pub output_connections: Vec<OutputConnection>,
    pub domain_values: Vec<DomainValue>,
    pub variables: Vec<PulseVariable>,
    pub map_name: String,
    pub xml_name: String,
}
impl PulseGraphDef {
    pub fn create_chunk(&mut self) -> i32 {
        let chunk = PulseChunk::default();
        self.chunks.push(chunk);
        self.chunks.len() as i32 - 1
    }
    pub fn create_domain_value(&mut self, typ: String, value: String, expected_runtime_type: String) -> i32 {
        let domain_value = DomainValue {
            typ,
            value,
            expected_runtime_type,
        };
        self.domain_values.push(domain_value);
        self.domain_values.len() as i32 - 1
    }
    // pub fn create_invoke_binding(&mut self, register_map: RegisterMap, func_name: String, cell_index: i32, src_chunk: i32, src_instruction: i32) -> i32 {
    //     let binding = InvokeBinding {
    //         register_map,
    //         func_name,
    //         cell_index,
    //         src_chunk,
    //         src_instruction,
    //     };
    //     self.bindings.push(binding);
    //     self.bindings.len() as i32 - 1
    // }
    pub fn add_binding(&mut self, binding: InvokeBinding) -> i32 {
        self.bindings.push(binding);
        self.bindings.len() as i32 - 1
    }
    pub fn add_constant(&mut self, constant: PulseConstant) -> i32 {
        self.constants.push(Box::from(constant));
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
    pub fn get_current_constant_id(&self) -> i32 {
        self.constants.len() as i32 - 1
    }
    pub fn get_current_domain_val_id(&self) -> i32 {
        self.domain_values.len() as i32 - 1
    }
    pub fn get_current_binding_id(&self) -> i32 {
        self.bindings.len() as i32 - 1
    }
    pub fn add_variable(&mut self, variable: PulseVariable) -> i32 {
        self.variables.push(variable);
        self.variables.len() as i32 - 1
    }
    pub fn get_variable_index(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|variable| variable.name == name)
    }
}

impl KV3Serialize for PulseGraphDef {
    fn serialize(&self) -> String {
        formatdoc!{
            "
            {{
                m_Cells = 
                [
                    {}
                ]
                m_DomainIdentifier = \"ServerPointEntity\"
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
                m_PublicOutputs = []
                m_OutputConnections = 
                [
                    {}
                ]
                m_InvokeBindings = 
                [
                    {}
                ]
                m_CallInfos = []
            }}
            "
            , self.cells.iter().map(|cell| {
                match cell.as_ref() {
                    CellType::InflowMethod(cell) => cell.serialize(),
                    CellType::StepEntFire(cell) => cell.serialize(),
                    CellType::InflowWait(cell) => cell.serialize(),
                    CellType::InflowEvent(cell) => cell.serialize(),
                    CellType::ValueFindEntByName(cell) => cell.serialize(),
                    CellType::DebugLog => formatdoc!("
                        {{
                            _class = \"CPulseCell_Step_DebugLog\"
                            m_nEditorNodeID = -1
                        }}"),
                }
            }).collect::<Vec<String>>().join(",\n\n")
            , self.map_name, self.xml_name
            , self.chunks.iter().map(|chunk| chunk.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.domain_values.iter().map(|domain_value| domain_value.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.variables.iter().map(|variable| variable.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.constants.iter().map(|constant| constant.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.output_connections.iter().map(|output_connection| output_connection.serialize()).collect::<Vec<String>>().join(",\n\n")
            , self.bindings.iter().map(|binding| binding.serialize()).collect::<Vec<String>>().join(",\n\n")
        }
    }
}

