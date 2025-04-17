#![allow(dead_code)]
use serde::Deserialize;
use serde_json::from_str;
use crate::typing::{PulseValueType, PulseTypeError, try_string_to_pulsevalue};

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryBindingType {
    Action,
    Value,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(skip)]
    pub pulsetype: PulseValueType,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionBinding {
    #[serde(rename = "type")]
    pub typ: LibraryBindingType,
    pub displayname: String,
    pub libname: String,
    pub inparams: Option<Vec<ParamInfo>>,
    pub outparams: Option<Vec<ParamInfo>>
}
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct EventBinding {
    pub displayname: String,
    pub libname: String,
    pub inparams: Option<Vec<ParamInfo>>,
}

#[derive(Deserialize, Debug, Default)]
pub struct GraphBindings {
    pub gamefunctions: Vec<FunctionBinding>,
    pub events: Vec<EventBinding>
}

fn process_params(params: &mut Option<Vec<ParamInfo>>) {
    if let Some(param_list) = params {
        for param in param_list.iter_mut() {
            // deliberately panic to signify invalid data in bindings
            param.pulsetype = try_string_to_pulsevalue(&param.typ).unwrap();
            param.typ.clear();
        }
    }
}

pub fn load_bindings(filepath: &std::path::Path) -> Result<GraphBindings, std::io::Error> {
    let json = std::fs::read_to_string(filepath);
    match json {
        Ok(json) => {
            let bindings = from_str::<GraphBindings>(&json);
            if bindings.is_err() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, bindings.unwrap_err()));
            }
            let mut bindings = bindings.unwrap();
            for binding in bindings.gamefunctions.iter_mut() {
                process_params(&mut binding.inparams);
                process_params(&mut binding.outparams);
            }
            for binding in bindings.events.iter_mut() {
                process_params(&mut binding.inparams);
            }
            Ok(bindings)
        }
        Err(e) => Err(e)
    }
}