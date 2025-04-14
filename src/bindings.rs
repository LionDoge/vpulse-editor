#![allow(dead_code)]
use serde::Deserialize;
use std::path::Path;
use serde_json::from_str;

#[derive(Deserialize, Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Deserialize, Debug)]
pub struct FunctionBinding {
    #[serde(rename = "type")]
    typ: String,
    displayname: String,
    libname: String,
    inparams: Option<Vec<ParamInfo>>,
    outparams: Option<Vec<ParamInfo>>
}
#[derive(Deserialize, Debug, Clone)]
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

pub fn load_bindings(filepath: &std::path::Path) -> Result<GraphBindings, std::io::Error> {
    let json = std::fs::read_to_string(filepath);
    match json {
        Ok(json) => {
            let bindings = from_str::<GraphBindings>(&json);
            match bindings {
                Ok(bindings) => Ok(bindings),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }
        }
        Err(e) => Err(e)
    }
}