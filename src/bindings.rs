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
pub enum PolimorphicTypeInfo {
    TypeParam(String),
    FullType(String),
    ToSubtype(String),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize))]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(skip_deserializing)]
    pub pulsetype: PulseValueType,
    #[serde(deserialize_with = "deserialize_polymorphic_arg", default)]
    pub polymorphic_arg: Option<PolimorphicTypeInfo>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
//#[cfg_attr(feature = "persistence", derive(Serialize))]
pub struct FunctionBinding {
    #[serde(rename = "type")]
    pub typ: LibraryBindingType,
    pub displayname: String,
    pub libname: String,
    pub description: Option<String>,
    pub inparams: Option<Vec<ParamInfo>>,
    pub outparams: Option<Vec<ParamInfo>>,
    #[serde(deserialize_with = "deserialize_polymorphic_arg", default)]
    pub polymorphic_return: Option<PolimorphicTypeInfo>,
}

impl FunctionBinding {
    pub fn find_inparam_by_name(&self, name: &str) -> Option<&ParamInfo> {
        self.inparams.as_ref()?.iter().find(|p| p.name == name)
    }

    pub fn find_outparam_by_name(&self, name: &str) -> Option<&ParamInfo> {
        self.outparams.as_ref()?.iter().find(|p| p.name == name)
    }
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct EventBinding {
    pub displayname: String,
    pub libname: String,
    pub inparams: Option<Vec<ParamInfo>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct HookBinding {
    pub displayname: String,
    pub libname: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct GraphBindings {
    pub gamefunctions: Vec<FunctionBinding>,
    pub events: Vec<EventBinding>,
    pub hooks: Vec<HookBinding>,
}

// stub used for Undo functionality, there's no need to clone these.
impl Clone for GraphBindings {
    fn clone(&self) -> Self {
        GraphBindings {
            gamefunctions: Vec::default(),
            events: Vec::default(),
            hooks: Vec::default(),
        }
    }
}

impl PartialEq for GraphBindings {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn process_params(params: &mut Option<Vec<ParamInfo>>) -> anyhow::Result<()> {
    if let Some(param_list) = params {
        for param in param_list.iter_mut() {
            // deliberately panic to signify invalid data in bindings
            param.pulsetype = try_string_to_pulsevalue(&param.typ).map_err(|err| {
                anyhow::anyhow!("Invalid PulseValueType in bindings: {}: {}", param.typ, err)
            })?;
            param.typ.clear();
        }
    }
    Ok(())
}

pub fn load_bindings(filepath: &std::path::Path) -> anyhow::Result<GraphBindings> {
    let json = std::fs::read_to_string(filepath)?;
    let mut bindings = from_str::<GraphBindings>(&json)?;
    for binding in bindings.gamefunctions.iter_mut() {
        process_params(&mut binding.inparams)?;
        process_params(&mut binding.outparams)?;
    }
    for binding in bindings.events.iter_mut() {
        process_params(&mut binding.inparams)?;
    }
    Ok(bindings)
}

fn deserialize_polymorphic_arg<'de, D>(deserializer: D) -> Result<Option<PolimorphicTypeInfo>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // First, deserialize as a string
    let s = String::deserialize(deserializer)?;
    
    let parts: Vec<&str> = s.split(':').collect();
    
    if parts.len() != 2 {
        return Ok(None);
    }
    // example: "a:typeparam"
    let param_name = parts[0].to_string();
    let type_enum = parts[1].to_string();
    
    match type_enum.as_str() {
        "typeparam" => Ok(Some(PolimorphicTypeInfo::TypeParam(param_name))),
        "fulltype" => Ok(Some(PolimorphicTypeInfo::FullType(param_name))),
        "to_subtype" => Ok(Some(PolimorphicTypeInfo::ToSubtype(param_name))),
        _ => Ok(None),
    }
}
