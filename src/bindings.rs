#![allow(dead_code)]

use crate::typing::{EnumBindingIndex, EnumBindingValueIndex, EventBindingIndex, HookBindingIndex, LibraryBindingIndex, PulseValueType, try_string_to_pulsevalue};
use serde::{Deserialize, Serialize};

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
    pub id: LibraryBindingIndex,
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
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct EventBinding {
    pub id: EventBindingIndex,
    pub displayname: String,
    pub libname: String,
    pub inparams: Option<Vec<ParamInfo>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HookBinding {
    pub id: HookBindingIndex,
    pub displayname: String,
    pub libname: String,
    pub description: Option<String>,
}

pub type EnumBindings = Vec<BindingEnum>;
#[derive(Deserialize, Debug, Default)]
pub struct GraphBindings {
    // Allow defaults, as not all games have to define everything
     #[serde(default)]
    pub gamefunctions: Vec<FunctionBinding>,
     #[serde(default)]
    pub events: Vec<EventBinding>,
     #[serde(default)]
    pub hooks: Vec<HookBinding>,
    #[serde(default)]
    pub enums: EnumBindings,
}

#[derive(Deserialize, Debug)]
struct GameBindingManifest {
    pub name: String,
    pub mod_name: String,
    pub name_prefix: String,
    pub bindings_file: String,
}

#[derive(Deserialize, Debug)]
struct BindingsManifest {
    pub shared_bindings_file: String,
    pub enums_file: String,
    pub bindings_list: Vec<GameBindingManifest>,
}

impl GraphBindings {
    pub fn find_function_by_libname(&self, libname: &str) -> Option<&FunctionBinding> {
        self.gamefunctions.iter().find(|f| f.libname == libname)
    }

    pub fn find_event_by_libname(&self, libname: &str) -> Option<&EventBinding> {
        self.events.iter().find(|e| e.libname == libname)
    }

    pub fn find_hook_by_libname(&self, libname: &str) -> Option<&HookBinding> {
        self.hooks.iter().find(|h| h.libname == libname)
    }

    pub fn find_function_by_id(&self, id: LibraryBindingIndex) -> Option<&FunctionBinding> {
        self.gamefunctions.iter().find(|f| f.id == id)
    }

    pub fn find_event_by_id(&self, id: EventBindingIndex) -> Option<&EventBinding> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn find_hook_by_id(&self, id: HookBindingIndex) -> Option<&HookBinding> {
        self.hooks.iter().find(|h| h.id == id)
    }

    pub fn find_enum_by_name(&self, name: &str) -> Option<&BindingEnum> {
        self.enums.iter().find(|e| e.name == name)
    }

    pub fn find_enum_by_id(&self, id: EnumBindingIndex) -> Option<&BindingEnum> {
        self.enums.iter().find(|e| e.id == id)
    }

    fn append_from(&mut self, other: &mut GraphBindings) {
        self.gamefunctions.append(&mut other.gamefunctions);
        self.events.append(&mut other.events);
        self.hooks.append(&mut other.hooks);
    }
}

// stub used for Undo functionality, there's no need to clone these.
impl Clone for GraphBindings {
    fn clone(&self) -> Self {
        GraphBindings::default()
    }
}

impl PartialEq for GraphBindings {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn process_params(enum_bindings: &EnumBindings, params: &mut Option<Vec<ParamInfo>>) -> anyhow::Result<()> {
    if let Some(param_list) = params {
        for param in param_list.iter_mut() {
            // deliberately panic to signify invalid data in bindings
            param.pulsetype = try_string_to_pulsevalue(enum_bindings, &param.typ).map_err(|err| {
                anyhow::anyhow!("Invalid PulseValueType in bindings: {}: {}", param.typ, err)
            })?;
            param.typ.clear();
        }
    }
    Ok(())
}

// enum bindings are required to process the proper types
fn load_game_bindings(enum_bindings: &EnumBindings, filepath: &std::path::Path) -> anyhow::Result<GraphBindings> {
    let json = std::fs::read_to_string(filepath)?;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    let mut bindings: GraphBindings = serde_path_to_error::deserialize(&mut deserializer)?;
    for binding in bindings.gamefunctions.iter_mut() {
        process_params(enum_bindings, &mut binding.inparams)?;
        process_params(enum_bindings, &mut binding.outparams)?;
    }
    for binding in bindings.events.iter_mut() {
        process_params(enum_bindings, &mut binding.inparams)?;
    }
    
    Ok(bindings)
}

pub fn load_bindings(filepath: &std::path::Path) -> anyhow::Result<GraphBindings> {
    let json = std::fs::read_to_string(filepath)?;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    let bindings_manifest: BindingsManifest = serde_path_to_error::deserialize(&mut deserializer)?;

    let json_enums = std::fs::read_to_string(&bindings_manifest.enums_file)?;
    let mut deserializer_enums = serde_json::Deserializer::from_str(&json_enums);
    let enums = serde_path_to_error::deserialize(&mut deserializer_enums)?;
    
    // load shared first, combine others, this might change in the future, if we only want to load for a given game.
    let mut all_bindings = load_game_bindings(&enums, std::path::Path::new(&bindings_manifest.shared_bindings_file))?;
    for game_binding_manifest in bindings_manifest.bindings_list.iter() {
        let mut game_bindings = load_game_bindings(&enums, std::path::Path::new(&game_binding_manifest.bindings_file))?;
        // For now add game prefix to displayname
        for func in game_bindings.gamefunctions.iter_mut() {
            func.displayname = format!("({}) {}", game_binding_manifest.name_prefix, func.displayname);
        }
        for event in game_bindings.events.iter_mut() {
            event.displayname = format!("({}) {}", game_binding_manifest.name_prefix, event.displayname);
        }
        for hook in game_bindings.hooks.iter_mut() {
            hook.displayname = format!("({}) {}", game_binding_manifest.name_prefix, hook.displayname);
        }
        all_bindings.append_from(&mut game_bindings);
    }

    all_bindings.enums = enums;
    Ok(all_bindings)
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
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BindingEnum {
    pub id: EnumBindingIndex,
    pub name: String,
    pub name_ui: String,
    #[serde(skip_serializing)]
    pub variants: Vec<EnumVariant>,
}

impl BindingEnum {
    pub fn get_variant_by_id(&self, id: EnumBindingValueIndex) -> Option<&EnumVariant> {
        self.variants.get(id.0)
    }
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub name_ui: String,
}
