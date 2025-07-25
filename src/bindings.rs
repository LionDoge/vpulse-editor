#![allow(dead_code)]
use crate::typing::{try_string_to_pulsevalue, PulseValueType};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize))]
#[serde(rename_all = "snake_case")]
pub enum LibraryBindingType {
    Action,
    Value,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize))]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(skip_deserializing)]
    pub pulsetype: PulseValueType,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize))]
pub struct FunctionBinding {
    #[serde(rename = "type")]
    pub typ: LibraryBindingType,
    pub displayname: String,
    pub libname: String,
    pub description: Option<String>,
    pub inparams: Option<Vec<ParamInfo>>,
    pub outparams: Option<Vec<ParamInfo>>,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct EventBinding {
    pub displayname: String,
    pub libname: String,
    pub inparams: Option<Vec<ParamInfo>>,
}

#[derive(Deserialize, Debug, Default)]
pub struct GraphBindings {
    pub gamefunctions: Vec<FunctionBinding>,
    pub events: Vec<EventBinding>,
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
            if let Err(err) = bindings {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err));
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
        Err(e) => Err(e),
    }
}
