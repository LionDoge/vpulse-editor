use super::serialization::*;

pub fn get_domain_value(register_id: i32, domain_val_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_DOMAIN_VALUE"),
        reg0: register_id,
        domain_value_idx: domain_val_id,
        ..Default::default()
    }
}

pub fn cell_invoke(invoke_binding_id: i32) -> Instruction {
    Instruction {
        code: String::from("CELL_INVOKE"),
        invoke_binding_index: invoke_binding_id,
        ..Default::default()
    }
}

pub fn get_const(const_id: i32, register_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_CONST"),
        reg0: register_id,
        const_idx: const_id,
        ..Default::default()
    }
}

pub fn add_string(in_a: i32, in_b: i32, out_id: i32) -> Instruction {
    Instruction {
        code: String::from("ADD_STRING"),
        var: -1,
        reg0: out_id,
        reg1: in_a,
        reg2: in_b,
        ..Default::default()
    }
}

pub fn get_var(register_id: i32, var_id: i32) -> Instruction {
    Instruction {
        code: String::from("GET_VAR"),
        var: var_id,
        reg0: register_id,
        ..Default::default()
    }
}

pub fn set_var(register_id: i32, var_id: i32) -> Instruction {
    Instruction {
        code: String::from("SET_VAR"),
        var: var_id,
        reg0: register_id,
        ..Default::default()
    }
}

pub fn convert_value(register_to: i32, register_from: i32) -> Instruction {
    Instruction {
        code: String::from("CONVERT_VALUE"),
        reg0: register_to,
        reg1: register_from,
        ..Default::default()
    }
}

pub fn reinterpret_instance(register_to: i32, register_from: i32) -> Instruction {
    Instruction {
        code: String::from("REINTERPRET_INSTANCE"),
        reg0: register_to,
        reg1: register_from,
        ..Default::default()
    }
}

pub fn library_invoke(invoke_binding_id: i32) -> Instruction {
    Instruction {
        code: String::from("LIBRARY_INVOKE"),
        invoke_binding_index: invoke_binding_id,
        ..Default::default()
    }
}

pub fn return_void() -> Instruction {
    Instruction {
        code: String::from("RETURN_VOID"),
        ..Default::default()
    }
}

pub fn jump(instruction_id: i32) -> Instruction {
    Instruction {
        code: String::from("JUMP"),
        dest_instruction: instruction_id,
        ..Default::default()
    }
}

pub fn jump_cond(register_cond: i32, instruction_id: i32) -> Instruction {
    let mut instr = jump(instruction_id);
    instr.code = String::from("JUMP_COND");
    instr.reg0 = register_cond;
    instr
}

pub fn copy_value(register_to: i32, register_from: i32) -> Instruction {
    Instruction {
        code: String::from("COPY"),
        reg0: register_to,
        reg1: register_from,
        ..Default::default()
    }
}

pub fn add_value(register_to: i32, register_from: i32, register_out: i32) -> Instruction {
    Instruction {
        code: String::from("ADD_INT"),
        reg0: register_out,
        reg1: register_to,
        reg2: register_from,
        ..Default::default()
    }
}

pub fn call_sync(call_info_index: i32, dest_chunk: i32, dest_instruction: i32) -> Instruction {
    Instruction {
        code: String::from("PULSE_CALL_SYNC"),
        call_info_index,
        chunk: dest_chunk,
        dest_instruction,
        ..Default::default()
    }
}

pub fn get_array_element(
    register_to: i32,
    array_register: i32,
    index_register: i32,
) -> Instruction {
    Instruction {
        code: String::from("GET_ARRAY_ELEMENT"),
        reg0: register_to,
        reg1: array_register,
        reg2: index_register,
        ..Default::default()
    }
}

pub fn return_value(register_id: i32) -> Instruction {
    Instruction {
        code: String::from("RETURN_VALUE"),
        reg0: register_id,
        ..Default::default()
    }
}
