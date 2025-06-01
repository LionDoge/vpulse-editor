use crate::serialization::*;

pub fn get_domain_value(register_id: i32, domain_val_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_DOMAIN_VALUE"),
        var: -1,
        reg0: register_id,
        reg1: -1,
        reg2: -1,
        invoke_binding_index: -1,
        chunk: -1,
        dest_instruction: 0,
        call_info_index: -1,
        const_idx: -1,
        domain_value_idx: domain_val_id,
        blackboard_reference_idx: -1,
    }
}

pub fn cell_invoke(invoke_binding_id: i32) -> Instruction {
    Instruction {
        code: String::from("CELL_INVOKE"),
        var: -1,
        reg0: -1,
        reg1: -1,
        reg2: -1,
        invoke_binding_index: invoke_binding_id,
        chunk: -1,
        dest_instruction: 0,
        call_info_index: -1,
        const_idx: -1,
        domain_value_idx: -1,
        blackboard_reference_idx: -1,
    }
}

pub fn get_const(const_id: i32, register_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_CONST"),
        var: -1,
        reg0: register_id,
        reg1: -1,
        reg2: -1,
        invoke_binding_index: -1,
        chunk: -1,
        dest_instruction: 0,
        call_info_index: -1,
        const_idx: const_id,
        domain_value_idx: -1,
        blackboard_reference_idx: -1,
    }
}

pub fn add_string(in_a: i32, in_b: i32, out_id: i32) -> Instruction {
    Instruction {
        code: String::from("ADD_STRING"),
        var: -1,
        reg0: out_id,
        reg1: in_a,
        reg2: in_b,
        invoke_binding_index: -1,
        chunk: -1,
        dest_instruction: 0,
        call_info_index: -1,
        const_idx: -1,
        domain_value_idx: -1,
        blackboard_reference_idx: -1,
    }
}

pub fn get_var(register_id: i32, var_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_VAR"),
        var: var_id,
        reg0: register_id,
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

pub fn set_var(register_id: i32, var_id: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("SET_VAR");
    instr.var = var_id;
    instr.reg0 = register_id;
    instr
}

pub fn convert_value(register_to: i32, register_from: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("CONVERT_VALUE");
    instr.reg0 = register_to;
    instr.reg1 = register_from;
    instr
}

pub fn library_invoke(invoke_binding_id: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("LIBRARY_INVOKE");
    instr.invoke_binding_index = invoke_binding_id;
    instr
}

pub fn return_void() -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("RETURN_VOID");
    instr
}

pub fn jump(instruction_id: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("JUMP");
    instr.dest_instruction = instruction_id;
    instr
}

pub fn jump_cond(register_cond: i32, instruction_id: i32) -> Instruction {
    let mut instr = jump(instruction_id);
    instr.code = String::from("JUMP_COND");
    instr.reg0 = register_cond;
    instr
}

pub fn copy_value(register_to: i32, register_from: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("COPY");
    instr.reg0 = register_to;
    instr.reg1 = register_from;
    instr
}

pub fn add_value(register_to: i32, register_from: i32, register_out: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("ADD_INT");
    instr.reg0 = register_out;
    instr.reg1 = register_to;
    instr.reg2 = register_from;
    instr
}

pub fn call_sync(call_info_index: i32, dest_chunk: i32, dest_instruction: i32) -> Instruction {
    let mut instr = Instruction::default();
    instr.code = String::from("PULSE_CALL_SYNC");
    instr.call_info_index = call_info_index;
    instr.chunk = dest_chunk;
    instr.dest_instruction = dest_instruction;
    instr
}