use serde::Deserialize;

use crate::pulsetypes::PulseValueType;

#[derive(Deserialize)]
pub struct ParamInfo {
    name: String,
    typ: PulseValueType
}

#[derive(Deserialize)]
pub struct FunctionBinding {
    displayname: String,
    libname: String,
    inparams: Vec<ParamInfo>,
    outparams: Vec<ParamInfo>
}

#[derive(Deserialize)]
pub struct GraphBindings {
    functions: Vec<FunctionBinding>
}