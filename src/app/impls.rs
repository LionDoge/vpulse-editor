use std::borrow::Cow;
use egui_node_graph2::*;
use eframe::egui::Color32;
use eframe::egui::{self, ComboBox, DragValue};
use super::types::*;
use crate::typing::*;
use super::help;
use crate::pulsetypes::*;

impl Default for PulseGraphValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Scalar { value: 0.0 }
    }
}

impl PulseGraphState {
    // pub fn add_node_custom_param(&mut self, param_name: String, node_id: NodeId) {
    //     if let Some(vec_params) = self.added_parameters.get_mut(node_id) {
    //         vec_params.push(param_name);
    //     } else {
    //         self.added_parameters.insert(node_id, vec![param_name]);
    //     }
    // }

    pub fn load_from(&mut self, other: PulseGraphState) {
        self.public_outputs = other.public_outputs;
        self.variables = other.variables;
        self.exposed_nodes = other.exposed_nodes;
        self.outputs_dropdown_choices = other.outputs_dropdown_choices;
        // rewrite everything but the save file path and bindings
    }
}

impl PulseGraphValueType {
    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let PulseGraphValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }

    pub fn try_to_string(self) -> anyhow::Result<String> {
        match self {
            PulseGraphValueType::String { value } => Ok(value),
            PulseGraphValueType::InternalOutputName { value, .. } => Ok(value),
            PulseGraphValueType::InternalVariableName { value, .. } => Ok(value),
            _ => anyhow::bail!("Invalid cast from {:?} to string", self),
        }
    }

    pub fn try_to_resource(self) -> anyhow::Result<(Option<String>, String)> {
        if let PulseGraphValueType::Resource { resource_type, value } = self {
            Ok((resource_type, value))
        } else {
            anyhow::bail!("Invalid cast from {:?} to resource", self)
        }
    }

    pub fn try_to_bool(self) -> anyhow::Result<bool> {
        if let PulseGraphValueType::Bool { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to bool", self)
        }
    }

    pub fn try_to_vec2(self) -> anyhow::Result<Vec2> {
        if let PulseGraphValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self)
        }
    }

    pub fn try_to_vec3(self) -> anyhow::Result<Vec3> {
        match self {
            PulseGraphValueType::Vec3 { value } 
            | PulseGraphValueType::Vec3Local {value}
            | PulseGraphValueType::QAngle { value } => Ok(value),
            _ => anyhow::bail!("Invalid cast from {:?} to vec3", self),
        }
    }

    pub fn try_to_vec4(self) -> anyhow::Result<Vec4> {
        if let PulseGraphValueType::Vec4 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec4", self)
        }
    }

    pub fn try_to_color_rgba(self) -> anyhow::Result<[f32; 4]> {
        if let PulseGraphValueType::Color { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to color", self)
        }
    }

    pub fn try_output_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::InternalOutputName { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to output name", self)
        }
    }

    pub fn try_variable_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::InternalVariableName { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to variable name", self)
        }
    }

    pub fn try_pulse_type(self) -> anyhow::Result<PulseValueType> {
        if let PulseGraphValueType::Typ { value, .. } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to variable name", self)
        }
    }

    pub fn try_entity_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::EntityName { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to entity name", self)
        }
    }

    pub fn try_event_binding_id(self) -> anyhow::Result<EventBindingIndex> {
        if let PulseGraphValueType::EventBindingChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to event binding", self)
        }
    }

    pub fn try_library_binding(self) -> anyhow::Result<LibraryBindingIndex> {
        if let PulseGraphValueType::LibraryBindingChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to library binding", self)
        }
    }

    pub fn try_hook_binding(self) -> anyhow::Result<HookBindingIndex> {
        if let PulseGraphValueType::HookBindingChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to hook binding", self)
        }
    }

    pub fn try_sndevt_name(self) -> anyhow::Result<String> {
        if let PulseGraphValueType::SoundEventName { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to string", self)
        }
    }

    pub fn try_node_id(self) -> anyhow::Result<NodeId> {
        if let PulseGraphValueType::NodeChoice { node } = self {
            if let Some(node_id) = node {
                Ok(node_id)
            } else {
                anyhow::bail!("Node choice is empty")
            }
        } else {
            anyhow::bail!("Invalid cast from {:?} to node id", self)
        }
    }

    pub fn try_enum(self) -> anyhow::Result<(SchemaEnumType, SchemaEnumValue)> {
        if let PulseGraphValueType::SchemaEnum { enum_type, value } = self {
            Ok((enum_type, value))
        } else {
            anyhow::bail!("Invalid cast from {:?} to schema enum", self)
        }
    }
    
    pub fn try_general_enum(self) -> anyhow::Result<GeneralEnumChoice> {
        if let PulseGraphValueType::GeneralEnumChoice { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to general enum", self)
        }
    }
}

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<PulseGraphState> for PulseDataType {
    fn data_type_color(&self, _user_state: &mut PulseGraphState) -> egui::Color32 {
        match self {
            PulseDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            PulseDataType::Vec2 => egui::Color32::from_rgb(238, 163, 109),
            PulseDataType::Vec3 => egui::Color32::from_rgb(238, 207, 109),
            PulseDataType::Vec3Local => egui::Color32::from_rgb(168, 144, 91),
            PulseDataType::Color => egui::Color32::from_rgb(111, 66, 245),
            PulseDataType::String => egui::Color32::from_rgb(52, 171, 235),
            PulseDataType::Action => egui::Color32::from_rgb(252, 3, 165),
            PulseDataType::EHandle => egui::Color32::from_rgb(11, 200, 31),
            PulseDataType::EntityName => egui::Color32::from_rgb(11, 77, 31),
            PulseDataType::Bool => egui::Color32::from_rgb(54, 61, 194),
            PulseDataType::InternalOutputName => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::InternalVariableName => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::Typ => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::EventBindingChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::LibraryBindingChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::HookBindingChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::SndEventHandle => egui::Color32::from_rgb(224, 123, 216),
            PulseDataType::SoundEventName => egui::Color32::from_rgb(52, 100, 120),
            PulseDataType::NoideChoice => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::Any => egui::Color32::from_rgb(200, 200, 200),
            PulseDataType::SchemaEnum => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::GeneralEnum => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::CommentBox => egui::Color32::from_rgb(0, 0, 0),
            PulseDataType::Vec4 => egui::Color32::from_rgb(210, 238, 109),
            PulseDataType::QAngle => egui::Color32::from_rgb(240, 252, 194),
            PulseDataType::Transform => egui::Color32::from_rgb(110, 100, 176),
            PulseDataType::TransformWorldspace => egui::Color32::from_rgb(169, 143, 247),
            PulseDataType::Resource => egui::Color32::from_rgb(250, 110, 192),
            PulseDataType::Array => egui::Color32::from_rgb(235, 113, 7),
            PulseDataType::GameTime => egui::Color32::from_rgb(118, 160, 219),
            PulseDataType::TypeSafeInteger => egui::Color32::from_rgb(11, 125, 91),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            PulseDataType::Scalar => Cow::Borrowed("scalar"),
            PulseDataType::Vec2 => Cow::Borrowed("2d vector"),
            PulseDataType::Vec3 => Cow::Borrowed("3d world vector"),
            PulseDataType::Vec3Local => Cow::Borrowed("3d local vector"),
            PulseDataType::Color => Cow::Borrowed("color"),
            PulseDataType::String => Cow::Borrowed("string"),
            PulseDataType::Bool => Cow::Borrowed("bool"),
            PulseDataType::Action => Cow::Borrowed("action"),
            PulseDataType::EHandle => Cow::Borrowed("EHandle"),
            PulseDataType::EntityName => Cow::Borrowed("Entity name"),
            PulseDataType::InternalOutputName => Cow::Borrowed("Output name"),
            PulseDataType::InternalVariableName => Cow::Borrowed("Variable name"),
            PulseDataType::Typ => Cow::Borrowed("Type"),
            PulseDataType::EventBindingChoice => Cow::Borrowed("Event binding"),
            PulseDataType::LibraryBindingChoice => Cow::Borrowed("Library binding"),
            PulseDataType::HookBindingChoice => Cow::Borrowed("Hook binding"),
            PulseDataType::SndEventHandle => Cow::Borrowed("Sound event handle"),
            PulseDataType::SoundEventName => Cow::Borrowed("Sound event name"),
            PulseDataType::NoideChoice => Cow::Borrowed("Node reference"),
            PulseDataType::Any => Cow::Borrowed("Any type"),
            PulseDataType::SchemaEnum => Cow::Borrowed("Schema enum"),
            PulseDataType::GeneralEnum => Cow::Borrowed("Enum"),
            PulseDataType::CommentBox => Cow::Borrowed("Comment box"),
            PulseDataType::Vec4 => Cow::Borrowed("4d vector"),
            PulseDataType::QAngle => Cow::Borrowed("QAngle"),
            PulseDataType::Transform => Cow::Borrowed("Transform"),
            PulseDataType::TransformWorldspace => Cow::Borrowed("Worldspace transform"),
            PulseDataType::Resource => Cow::Borrowed("Resource"),
            PulseDataType::Array => Cow::Borrowed("Array"),
            PulseDataType::GameTime => Cow::Borrowed("Game time"),
            PulseDataType::TypeSafeInteger => Cow::Borrowed("Type-safe integer"),
        }
    }

    fn allow_any_type(&self) -> bool {
        matches!(self, PulseDataType::Any)
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for PulseNodeTemplate {
    type NodeData = PulseNodeData;
    type DataType = PulseDataType;
    type ValueType = PulseGraphValueType;
    type UserState = PulseGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        match self {
            PulseNodeTemplate::CellPublicMethod => "Public Method".into(),
            PulseNodeTemplate::EntFire => "EntFire".into(),
            PulseNodeTemplate::Compare => "Compare".into(),
            PulseNodeTemplate::ConcatString => "Concatenate strings".into(),
            PulseNodeTemplate::CellWait => "Wait".into(),
            PulseNodeTemplate::GetVar => "Load variable".into(),
            PulseNodeTemplate::SetVar => "Save variable".into(),
            PulseNodeTemplate::EventHandler => "Event Handler".into(),
            PulseNodeTemplate::IntToString => "Int to string".into(),
            PulseNodeTemplate::Operation => "Operation".into(),
            PulseNodeTemplate::FindEntByName => "Find entity by name".into(),
            PulseNodeTemplate::DebugWorldText => "Debug world text".into(),
            PulseNodeTemplate::DebugLog => "Debug log".into(),
            PulseNodeTemplate::FireOutput => "Fire output".into(),
            PulseNodeTemplate::GraphHook => "Graph Hook".into(),
            PulseNodeTemplate::GetGameTime => "Get game time".into(),
            PulseNodeTemplate::SetNextThink => "Set next think".into(),
            PulseNodeTemplate::Convert => "Convert".into(),
            PulseNodeTemplate::ForLoop => "For loop".into(),
            PulseNodeTemplate::WhileLoop => "While loop".into(),
            PulseNodeTemplate::StringToEntityName => "String to entity name".into(),
            PulseNodeTemplate::InvokeLibraryBinding => "Invoke library binding".into(),
            PulseNodeTemplate::FindEntitiesWithin => "Find entities within".into(),
            PulseNodeTemplate::IsValidEntity => "Is valid entity".into(),
            PulseNodeTemplate::CompareOutput => "Compare".into(),
            PulseNodeTemplate::CompareIf => "If".into(),
            PulseNodeTemplate::IntSwitch => "Int Switch".into(),
            PulseNodeTemplate::SoundEventStart => "Sound event start".into(),
            PulseNodeTemplate::Function => "Function".into(),
            PulseNodeTemplate::CallNode => "Call node".into(),
            PulseNodeTemplate::ListenForEntityOutput => "Listen for entity output".into(),
            PulseNodeTemplate::Timeline => "Timeline".into(),
            PulseNodeTemplate::Comment => "Comment".into(),
            PulseNodeTemplate::SetAnimGraphParam => "Set AnimGraph param".into(),
            PulseNodeTemplate::ConstantBool => "Constant Bool".into(),
            PulseNodeTemplate::ConstantFloat => "Constant Float".into(),
            PulseNodeTemplate::ConstantString => "Constant String".into(),
            PulseNodeTemplate::ConstantVec3 => "Constant Vec3".into(),
            PulseNodeTemplate::ConstantInt => "Constant Int".into(),
            PulseNodeTemplate::NewArray => "Make Array".into(),
            PulseNodeTemplate::LibraryBindingAssigned { binding } => {
                // TODO: Setup proper lifetimes so we don't have to clone
                _user_state.bindings.find_function_by_id(*binding)
                    .map_or("[INVALID]".into(), |f| f.displayname.clone().into()
                )
            }
            PulseNodeTemplate::GetArrayElement => "Get array element".into(),
            PulseNodeTemplate::ScaleVector => "Scale vector".into(),
            PulseNodeTemplate::ReturnValue => "Return value".into(),
            PulseNodeTemplate::ForEach => "For each".into(),
            PulseNodeTemplate::And => "And".into(),
            PulseNodeTemplate::Or => "Or".into(),
            PulseNodeTemplate::Not => "Not".into(),
            PulseNodeTemplate::RandomFloat => "Random float".into(),
            PulseNodeTemplate::RandomInt => "Random int".into(),
            PulseNodeTemplate::EntOutputHandler => "Entity Output Handler".into(),
        }
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            PulseNodeTemplate::CellPublicMethod
            | PulseNodeTemplate::EventHandler
            | PulseNodeTemplate::GraphHook
            | PulseNodeTemplate::EntOutputHandler => vec!["Inflow"],
            PulseNodeTemplate::EntFire
            | PulseNodeTemplate::FindEntByName
            | PulseNodeTemplate::FindEntitiesWithin
            | PulseNodeTemplate::IsValidEntity
            | PulseNodeTemplate::ListenForEntityOutput => vec!["Entities"],
            PulseNodeTemplate::Compare
            | PulseNodeTemplate::CompareOutput
            | PulseNodeTemplate::CompareIf
            | PulseNodeTemplate::IntSwitch
            | PulseNodeTemplate::CallNode
            | PulseNodeTemplate::Function 
            | PulseNodeTemplate::ReturnValue
            | PulseNodeTemplate::And
            | PulseNodeTemplate::Or
            | PulseNodeTemplate::Not => vec!["Logic"],
            PulseNodeTemplate::Operation 
            | PulseNodeTemplate::ScaleVector
            | PulseNodeTemplate::RandomFloat
            | PulseNodeTemplate::RandomInt => vec!["Math"],
            PulseNodeTemplate::ConcatString => vec!["String"],
            PulseNodeTemplate::CellWait | PulseNodeTemplate::Timeline => vec!["Timing"],
            PulseNodeTemplate::GetVar 
            | PulseNodeTemplate::SetVar
            | PulseNodeTemplate::GetArrayElement => vec!["Variables"],
            PulseNodeTemplate::IntToString
            | PulseNodeTemplate::Convert
            | PulseNodeTemplate::StringToEntityName => vec!["Conversion"],
            PulseNodeTemplate::DebugWorldText | PulseNodeTemplate::DebugLog => vec!["Debug"],
            PulseNodeTemplate::FireOutput => vec!["Outflow"],
            PulseNodeTemplate::GetGameTime
            | PulseNodeTemplate::SetNextThink
            | PulseNodeTemplate::InvokeLibraryBinding => {
                vec!["Game functions"]
            }
            PulseNodeTemplate::LibraryBindingAssigned { binding: _ } => {
                vec!["Game functions"]
            }
            PulseNodeTemplate::ForLoop 
            | PulseNodeTemplate::WhileLoop
            | PulseNodeTemplate::ForEach => vec!["Loops"],
            PulseNodeTemplate::SoundEventStart => vec!["Sound"],
            PulseNodeTemplate::Comment => vec!["Editor"],
            PulseNodeTemplate::SetAnimGraphParam => vec!["Animation"],
            PulseNodeTemplate::ConstantBool
            | PulseNodeTemplate::ConstantFloat
            | PulseNodeTemplate::ConstantString
            | PulseNodeTemplate::ConstantVec3
            | PulseNodeTemplate::ConstantInt 
            | PulseNodeTemplate::NewArray => vec!["Constants"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        PulseNodeData {
            template: *self,
            custom_named_outputs: Default::default(),
            added_parameters: Default::default(),
            input_hint_text: None,
            custom_output_type: None,
            added_inputs: Vec::new(),
        }
    }

    #[allow(unused_variables)]
    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        // We define some closures here to avoid boilerplate. Note that this is
        // entirely optional.
        let input_string = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::String,
                PulseGraphValueType::String {
                    value: String::default(),
                },
                kind,
                true,
            );
        };
        let input_scalar =
            |graph: &mut PulseGraph, name: &str, kind: InputParamKind, default: f32| {
                graph.add_input_param(
                    node_id,
                    name.to_string(),
                    PulseDataType::Scalar,
                    PulseGraphValueType::Scalar { value: default },
                    kind,
                    true,
                );
            };
        let input_bool = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Bool,
                PulseGraphValueType::Bool { value: false },
                kind,
                true,
            );
        };
        let input_ehandle = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::EHandle,
                PulseGraphValueType::EHandle,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_entityname = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::EntityName,
                PulseGraphValueType::EntityName {
                    value: String::default(),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector3 = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Vec3,
                PulseGraphValueType::Vec3 {
                    value: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_color = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Color,
                PulseGraphValueType::Color {
                    value: [1.0, 1.0, 1.0, 1.0],
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_action = |graph: &mut PulseGraph| {
            graph.add_input_param(
                node_id,
                "ActionIn".to_string(),
                PulseDataType::Action,
                PulseGraphValueType::Action,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_typ = |graph: &mut PulseGraph, name: &str, def_typ: PulseValueType| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Typ,
                PulseGraphValueType::Typ {
                    value: def_typ,
                },
                InputParamKind::ConstantOnly,
                true,
            );
        };
        let input_sndevt_name = |graph: &mut PulseGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::SoundEventName,
                PulseGraphValueType::SoundEventName {
                    value: String::default(),
                },
                kind,
                true,
            );
        };
        let input_any = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Any,
                PulseGraphValueType::Any,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_array = |graph: &mut PulseGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::Array,
                PulseGraphValueType::Array,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let input_general_enum = |graph: &mut PulseGraph, name: &str, enum_type: GeneralEnumChoice| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                PulseDataType::GeneralEnum,
                PulseGraphValueType::GeneralEnumChoice {
                    value: enum_type,
                },
                InputParamKind::ConstantOnly,
                true,
            );
        };
        let output_scalar = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Scalar);
        };
        let output_string = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::String);
        };
        let output_action = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Action);
        };
        let output_ehandle = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::EHandle);
        };
        let output_entityname = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::EntityName);
        };
        let output_bool = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Bool);
        };
        let output_vector3 = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Vec3);
        };
        let output_array = |graph: &mut PulseGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), PulseDataType::Array);
        };

        let mut make_referencable = || {
            user_state.exposed_nodes.insert(node_id, String::default());
        };
        match self {
            PulseNodeTemplate::CellPublicMethod => {
                graph.add_input_param(
                    node_id,
                    "name".into(),
                    PulseDataType::String,
                    PulseGraphValueType::String {
                        value: "method".to_string(),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_string(graph, "argument1");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::EntFire => {
                input_action(graph);
                input_entityname(graph, "entity");
                input_ehandle(graph, "entityHandle");
                input_string(graph, "input", InputParamKind::ConstantOnly);
                input_string(graph, "value", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::Compare => {
                input_action(graph);
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_typ(graph, "type", PulseValueType::PVAL_INT(None));
                input_scalar(graph, "A", InputParamKind::ConnectionOrConstant, 0.0);
                input_scalar(graph, "B", InputParamKind::ConnectionOrConstant, 0.0);
                output_action(graph, "True");
                output_action(graph, "False");
            }
            PulseNodeTemplate::ConcatString => {
                input_string(graph, "A", InputParamKind::ConnectionOrConstant);
                input_string(graph, "B", InputParamKind::ConnectionOrConstant);
                output_string(graph, "out");
            }
            PulseNodeTemplate::CellWait => {
                input_action(graph);
                input_scalar(graph, "time", InputParamKind::ConnectionOrConstant, 0.0);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GetVar => {
                graph.add_input_param(
                    node_id,
                    String::from("variableName"),
                    PulseDataType::InternalVariableName,
                    PulseGraphValueType::InternalVariableName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                //output_scalar(graph, "out");
            }
            PulseNodeTemplate::SetVar => {
                input_action(graph);
                graph.add_input_param(
                    node_id,
                    String::from("variableName"),
                    PulseDataType::InternalVariableName,
                    PulseGraphValueType::InternalVariableName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                //input_scalar(graph, "value");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::EventHandler => {
                graph.add_input_param(
                    node_id,
                    String::from("event"),
                    PulseDataType::EventBindingChoice,
                    PulseGraphValueType::EventBindingChoice {
                        value: EventBindingIndex(1),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::IntToString => {
                input_scalar(graph, "value", InputParamKind::ConnectionOrConstant, 0.0);
                output_string(graph, "out");
            }
            PulseNodeTemplate::Operation => {
                input_typ(graph, "type", PulseValueType::PVAL_INT(None));
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_scalar(graph, "A", InputParamKind::ConnectionOrConstant, 0.0);
                input_scalar(graph, "B", InputParamKind::ConnectionOrConstant, 0.0);
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::FindEntByName => {
                input_entityname(graph, "entName");
                input_string(graph, "entClass", InputParamKind::ConnectionOrConstant);
                output_ehandle(graph, "out");
            }
            PulseNodeTemplate::DebugWorldText => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                input_ehandle(graph, "hEntity");
                input_scalar(
                    graph,
                    "nTextOffset",
                    InputParamKind::ConnectionOrConstant,
                    0.0,
                );
                input_scalar(
                    graph,
                    "flDuration",
                    InputParamKind::ConnectionOrConstant,
                    5.0,
                );
                input_scalar(
                    graph,
                    "flVerticalOffset",
                    InputParamKind::ConnectionOrConstant,
                    0.0,
                );
                input_bool(graph, "bAttached", InputParamKind::ConstantOnly);
                input_color(graph, "color");
                input_scalar(graph, "flAlpha", InputParamKind::ConnectionOrConstant, 1.0);
                input_scalar(graph, "flScale", InputParamKind::ConnectionOrConstant, 1.0);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::DebugLog => {
                input_action(graph);
                input_string(graph, "pMessage", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::FireOutput => {
                input_action(graph);
                graph.add_input_param(
                    node_id,
                    String::from("outputName"),
                    PulseDataType::InternalOutputName,
                    PulseGraphValueType::InternalOutputName {
                        prevvalue: String::default(),
                        value: String::from("CHOOSE"),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GraphHook => {
                graph.add_input_param(
                    node_id,
                    String::from("hook"),
                    PulseDataType::HookBindingChoice,
                    PulseGraphValueType::HookBindingChoice {
                        value: HookBindingIndex(0),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::GetGameTime => {
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::SetNextThink => {
                input_action(graph);
                input_scalar(graph, "dt", InputParamKind::ConnectionOrConstant, 0.0);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::Convert => {
                input_typ(graph, "typeto", PulseValueType::PVAL_STRING(None));
                input_string(graph, "entityclass", InputParamKind::ConstantOnly);
                input_any(graph, "input");
                output_string(graph, "out");
            }
            PulseNodeTemplate::ForLoop => {
                input_action(graph);
                input_scalar(graph, "start", InputParamKind::ConnectionOrConstant, 0.0);
                input_scalar(graph, "end", InputParamKind::ConnectionOrConstant, 5.0);
                input_scalar(graph, "step", InputParamKind::ConnectionOrConstant, 1.0);
                output_scalar(graph, "index");
                output_action(graph, "loopAction");
                output_action(graph, "endAction");
            }
            PulseNodeTemplate::StringToEntityName => {
                input_string(graph, "entityName", InputParamKind::ConnectionOrConstant);
                output_entityname(graph, "out");
            }
            PulseNodeTemplate::InvokeLibraryBinding => {
                graph.add_input_param(
                    node_id,
                    String::from("binding"),
                    PulseDataType::LibraryBindingChoice,
                    PulseGraphValueType::LibraryBindingChoice {
                        value: LibraryBindingIndex(1),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
            }
            PulseNodeTemplate::FindEntitiesWithin => {
                input_string(graph, "classname", InputParamKind::ConstantOnly);
                input_ehandle(graph, "pSearchFromEntity");
                input_scalar(
                    graph,
                    "flSearchRadius",
                    InputParamKind::ConnectionOrConstant,
                    0.0,
                );
                input_ehandle(graph, "pStartEntity");
                output_ehandle(graph, "out");
            }
            PulseNodeTemplate::IsValidEntity => {
                input_action(graph);
                input_ehandle(graph, "hEntity");
                output_action(graph, "True");
                output_action(graph, "False");
            }
            PulseNodeTemplate::CompareOutput => {
                input_typ(graph, "type", PulseValueType::PVAL_INT(None));
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_scalar(graph, "A", InputParamKind::ConnectionOrConstant, 0.0);
                input_scalar(graph, "B", InputParamKind::ConnectionOrConstant, 0.0);
                output_bool(graph, "out");
            }
            PulseNodeTemplate::WhileLoop => {
                input_action(graph);
                input_bool(graph, "do-while", InputParamKind::ConstantOnly);
                input_bool(graph, "condition", InputParamKind::ConnectionOnly);
                output_action(graph, "loopAction");
                output_action(graph, "endAction");
            }
            PulseNodeTemplate::CompareIf => {
                input_action(graph);
                input_bool(graph, "condition", InputParamKind::ConnectionOnly);
                output_action(graph, "True");
                output_action(graph, "False");
                output_action(graph, "Either");
            }
            PulseNodeTemplate::IntSwitch => {
                input_action(graph);
                input_scalar(graph, "value", InputParamKind::ConnectionOrConstant, 0.0);
                // cases will be added dynamically by user
                // this field will be a buffer that will be used to create the cases
                // once the button to add it is pressed - which is defined in bottom_ui func.
                graph.add_input_param(
                    node_id,
                    "caselabel".into(),
                    PulseDataType::Scalar,
                    PulseGraphValueType::Scalar { value: 0.0 },
                    InputParamKind::ConstantOnly,
                    true,
                );
                output_action(graph, "defaultcase");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::SoundEventStart => {
                input_action(graph);
                input_sndevt_name(
                    graph,
                    "strSoundEventName",
                    InputParamKind::ConnectionOrConstant,
                );
                input_general_enum(
                    graph,
                    "soundEventType",
                    GeneralEnumChoice::SoundEventStartType(SoundEventStartType::default()),
                );
                input_ehandle(graph, "hTargetEntity");
                output_action(graph, "outAction");
                graph.add_output_param(node_id, "retval".into(), PulseDataType::SndEventHandle);
            }
            PulseNodeTemplate::Function => {
                make_referencable();
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::CallNode => {
                graph.add_input_param(
                    node_id,
                    "nodeId".into(),
                    PulseDataType::NoideChoice,
                    PulseGraphValueType::NodeChoice { node: None },
                    InputParamKind::ConstantOnly,
                    true,
                );
                input_action(graph);
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::ListenForEntityOutput => {
                make_referencable();
                input_string(graph, "outputName", InputParamKind::ConstantOnly);
                input_string(graph, "outputParam", InputParamKind::ConstantOnly);
                input_bool(graph, "bListenUntilCanceled", InputParamKind::ConstantOnly);
                output_ehandle(graph, "pActivator");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::Timeline => {
                graph.add_input_param(
                    node_id,
                    "Start".to_string(),
                    PulseDataType::Action,
                    PulseGraphValueType::Action,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                input_scalar(
                    graph,
                    "timeFromPrevious1",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction1");
                input_scalar(
                    graph,
                    "timeFromPrevious2",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction2");
                input_scalar(
                    graph,
                    "timeFromPrevious3",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction3");
                input_scalar(
                    graph,
                    "timeFromPrevious4",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction4");
                input_scalar(
                    graph,
                    "timeFromPrevious5",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction5");
                input_scalar(
                    graph,
                    "timeFromPrevious6",
                    InputParamKind::ConstantOnly,
                    0.5,
                );
                output_action(graph, "outAction6");
            }
            PulseNodeTemplate::Comment => {
                // This is a special node that is used to display comments in the graph.
                // It does not have any inputs or outputs, but it can be used to display
                // text in the graph.
                graph.add_input_param(
                    node_id,
                    "text".into(),
                    PulseDataType::CommentBox,
                    PulseGraphValueType::CommentBox {
                        value: String::default(),
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
            }
            PulseNodeTemplate::SetAnimGraphParam => {
                input_action(graph);
                input_ehandle(graph, "hEntity");
                input_string(graph, "paramName", InputParamKind::ConstantOnly);
                input_any(graph, "pParamValue");
                output_action(graph, "outAction");
            }
            PulseNodeTemplate::ConstantBool => {
                input_bool(graph, "value", InputParamKind::ConstantOnly);
                output_bool(graph, "out");
            }
            PulseNodeTemplate::ConstantFloat 
            | PulseNodeTemplate::ConstantInt => {
                input_scalar(
                    graph,
                    "value",
                    InputParamKind::ConstantOnly,
                    0.0,
                );
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::ConstantString => {
                input_string(graph, "value", InputParamKind::ConstantOnly);
                output_string(graph, "out");
            }
            PulseNodeTemplate::ConstantVec3 => {
                input_vector3(graph, "value", InputParamKind::ConstantOnly);
                output_vector3(graph, "out");
            }
            PulseNodeTemplate::NewArray => {
                input_typ(graph, "arrayType", PulseValueType::PVAL_INT(None));
                output_array(graph, "out");
            }
            PulseNodeTemplate::LibraryBindingAssigned { binding } => {
                graph.add_input_param(
                    node_id,
                    String::from("binding"),
                    PulseDataType::LibraryBindingChoice,
                    PulseGraphValueType::LibraryBindingChoice {
                        value: *binding,
                    },
                    InputParamKind::ConstantOnly,
                    true,
                );
            }
            PulseNodeTemplate::GetArrayElement => {
                //input_typ(graph, "expectedType");
                input_array(graph, "array");
                input_scalar(graph, "index", InputParamKind::ConnectionOrConstant, 0.0);
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::ScaleVector => {
                input_typ(graph, "type", PulseValueType::PVAL_VEC3(None));
                input_bool(graph, "invert", InputParamKind::ConstantOnly);
                input_scalar(graph, "scale", InputParamKind::ConnectionOrConstant, 1.0);
                input_vector3(graph, "vector", InputParamKind::ConnectionOrConstant);
                output_vector3(graph, "out");
            }
            PulseNodeTemplate::ReturnValue => {
                input_action(graph);
                input_any(graph, "value");
            }
            PulseNodeTemplate::ForEach => {
                input_action(graph);
                input_array(graph, "array");
                output_scalar(graph, "index");
                output_scalar(graph, "out"); // actual type will be set depending on the connected array.
                output_action(graph, "loopAction");
                output_action(graph, "endAction");
            }
            PulseNodeTemplate::And
            | PulseNodeTemplate::Or => {
                input_bool(graph, "A", InputParamKind::ConnectionOrConstant);
                input_bool(graph, "B", InputParamKind::ConnectionOrConstant);
                output_bool(graph, "out");
            }
            PulseNodeTemplate::Not => {
                input_bool(graph, "in", InputParamKind::ConnectionOrConstant);
                output_bool(graph, "out");
            }
            PulseNodeTemplate::RandomFloat
            | PulseNodeTemplate::RandomInt => {
                input_scalar(graph, "min", InputParamKind::ConnectionOrConstant, 0.0);
                input_scalar(graph, "max", InputParamKind::ConnectionOrConstant, 10.0);
                output_scalar(graph, "out");
            }
            PulseNodeTemplate::EntOutputHandler => {
                input_string(graph, "entityName", InputParamKind::ConstantOnly);
                input_string(graph, "outputName", InputParamKind::ConstantOnly);
                // TODO: implement passing the output parameter
                //input_typ(graph, "expectedType", PulseValueType::PVAL_ANY);
                output_action(graph, "outAction");
            }
        }
    }
}

impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = PulseNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        let mut templates = vec![
            PulseNodeTemplate::CellPublicMethod,
            PulseNodeTemplate::EntFire,
            //PulseNodeTemplate::Compare,
            PulseNodeTemplate::ConcatString,
            PulseNodeTemplate::CellWait,
            PulseNodeTemplate::GetVar,
            PulseNodeTemplate::SetVar,
            PulseNodeTemplate::EventHandler,
            //PulseNodeTemplate::IntToString,
            PulseNodeTemplate::Operation,
            PulseNodeTemplate::FindEntByName,
            PulseNodeTemplate::DebugWorldText,
            PulseNodeTemplate::DebugLog,
            PulseNodeTemplate::FireOutput,
            PulseNodeTemplate::GraphHook,
            //PulseNodeTemplate::GetGameTime,
            //PulseNodeTemplate::SetNextThink,
            PulseNodeTemplate::Convert,
            PulseNodeTemplate::ForLoop,
            PulseNodeTemplate::WhileLoop,
            PulseNodeTemplate::StringToEntityName,
            //PulseNodeTemplate::InvokeLibraryBinding, // replaced by LibraryBindingAssigned (displayed in node finder)
            PulseNodeTemplate::FindEntitiesWithin,
            //PulseNodeTemplate::IsValidEntity,
            PulseNodeTemplate::CompareOutput,
            PulseNodeTemplate::CompareIf,
            PulseNodeTemplate::IntSwitch,
            PulseNodeTemplate::SoundEventStart,
            PulseNodeTemplate::Function,
            PulseNodeTemplate::CallNode,
            PulseNodeTemplate::ListenForEntityOutput,
            PulseNodeTemplate::Timeline,
            PulseNodeTemplate::Comment,
            PulseNodeTemplate::SetAnimGraphParam,
            PulseNodeTemplate::ConstantBool,
            PulseNodeTemplate::ConstantFloat,
            PulseNodeTemplate::ConstantString,
            PulseNodeTemplate::ConstantVec3,
            PulseNodeTemplate::ConstantInt,
            PulseNodeTemplate::NewArray,
            PulseNodeTemplate::GetArrayElement,
            PulseNodeTemplate::ScaleVector,
            PulseNodeTemplate::ReturnValue,
            PulseNodeTemplate::ForEach,
            PulseNodeTemplate::And,
            PulseNodeTemplate::Or,
            PulseNodeTemplate::Not,
            PulseNodeTemplate::RandomInt,
            PulseNodeTemplate::RandomFloat,
            PulseNodeTemplate::EntOutputHandler,
        ];
        templates.extend(
                (0..self.game_function_count).map(|i| PulseNodeTemplate::LibraryBindingAssigned {
                // ! If we skip an ID in the actual bindings list then it could cause problems!
                binding: LibraryBindingIndex(i as u32),
            }),
        );
        templates
    }
}

impl PulseNodeTemplate {
    fn has_user_addable_outputs(&self) -> bool {
        // This function is used to determine if the node has outputs that can be
        // added and removed by the user. This is used to prove an add parameter button, as well as
        // to display the "X" button next to custom added output params.
        //matches!(self, PulseNodeTemplate::CellPublicMethod)
        false
    }
}

pub fn type_selection_widget(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    current_value: &mut PulseValueType,
    type_list: Vec<PulseValueType>,
    mut callback: impl FnMut(PulseValueType) // when a type selection is made
) {
    ComboBox::from_id_salt(id_salt)
        .width(0.0)
        .selected_text(current_value.get_ui_name())
        .show_ui(ui, |ui| {
            for typ in type_list {
                let name = typ.get_ui_name();
                if ui.selectable_value(current_value,
                    typ.clone(),
                    name
                ).clicked() {
                    callback(typ.clone());
                }
            }
        });
}

impl WidgetValueTrait for PulseGraphValueType {
    type Response = PulseGraphResponse;
    type UserState = PulseGraphState;
    type NodeData = PulseNodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut PulseGraphState,
        node_data: &PulseNodeData,
        input_id: InputId,
    ) -> Vec<PulseGraphResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        let mut responses = vec![];
        ui.horizontal(|ui| {
            if node_data.added_inputs.contains(&input_id) {
                // if this is a user added parameter, we want to show a remove button
                if ui.button("X").clicked() {
                    responses.push(PulseGraphResponse::RemoveCustomInputParam(node_id, input_id));
                }
            }
            match self {
                PulseGraphValueType::Scalar { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.add(DragValue::new(value));
                    });
                }
                PulseGraphValueType::String { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.text_edit_singleline(value)
                    });
                }
                PulseGraphValueType::Bool { value } => {
                    ui.horizontal(|ui| {
                        ui.checkbox(value, param_name);
                    });
                }
                PulseGraphValueType::Vec2 { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.add(DragValue::new(&mut value.x));
                        ui.add(DragValue::new(&mut value.y));
                    });
                }
                PulseGraphValueType::Vec3 {value} 
                | PulseGraphValueType::Vec3Local { value }
                | PulseGraphValueType::QAngle { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.add(DragValue::new(&mut value.x));
                        ui.add(DragValue::new(&mut value.y));
                        ui.add(DragValue::new(&mut value.z));
                    });
                }
                PulseGraphValueType::Vec4 { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.add(DragValue::new(&mut value.x));
                        ui.add(DragValue::new(&mut value.y));
                        ui.add(DragValue::new(&mut value.z));
                        ui.add(DragValue::new(&mut value.w));
                    });
                }
                PulseGraphValueType::Color { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.color_edit_button_rgba_unmultiplied(value);
                    });
                }
                PulseGraphValueType::Action => {
                    ui.label(format!("Action {param_name}"));
                }
                PulseGraphValueType::EHandle => {
                    ui.label(format!("EHandle {param_name}"));
                }
                PulseGraphValueType::SndEventHandle => {
                    ui.label(format!("SNDEVT {param_name}"));
                }
                PulseGraphValueType::SoundEventName { value } => {
                    ui.horizontal(|ui| {
                        ui.label(format!("SNDEVT {param_name}"));
                        ui.text_edit_singleline(value);
                    });
                }
                PulseGraphValueType::EntityName { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ui.text_edit_singleline(value);
                    });
                }
                PulseGraphValueType::InternalOutputName { prevvalue: _, value } => {
                    ui.horizontal(|ui| {
                        ui.label("Output");
                        ComboBox::from_id_salt(("outch", node_id))
                            .width(0.0)
                            .selected_text(value.clone())
                            .show_ui(ui, |ui| {
                                for outputparam in user_state.public_outputs.iter() {
                                    if ui.selectable_value(
                                        value,
                                        outputparam.name.clone(),
                                        outputparam.name.clone(),
                                    ).clicked() {
                                        responses.push(PulseGraphResponse::ChangeOutputParamType(
                                            node_id,
                                            value.to_string(),
                                        ));
                                    }
                                }
                            });
                    });
                }
                PulseGraphValueType::InternalVariableName { prevvalue: _, value } => {
                    ui.horizontal(|ui| {
                        ui.label("Variable");
                        ComboBox::from_id_salt(("varch", node_id))
                            .width(0.0)
                            .selected_text(value.clone())
                            .show_ui(ui, |ui| {
                                for var in user_state.variables.iter() {
                                    if ui.selectable_value(value, var.name.clone(), var.name.clone()).clicked() {
                                        responses.push(PulseGraphResponse::ChangeVariableParamType(
                                            node_id,
                                            value.to_string(),
                                        ));
                                        responses.push(PulseGraphResponse::UpdatePolymorphicTypes(node_id));
                                    }
                                }
                            });
                    });
                }
                // NOTE: Available types in the combobox are defined by the node template type.
                // We only want to allow some types to be selected depending on the context.
                PulseGraphValueType::Typ { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        let type_list: Vec<PulseValueType> = match &node_data.template {
                            PulseNodeTemplate::CompareOutput => PulseValueType::get_comparable_types(),
                            PulseNodeTemplate::Operation => PulseValueType::get_operatable_types(),
                            PulseNodeTemplate::ScaleVector => PulseValueType::get_vector_types(),
                            _ => PulseValueType::get_variable_supported_types(),
                        };
                        let callback = |new_type: PulseValueType| {
                            responses.push(PulseGraphResponse::ChangeParamType(
                                node_id,
                                param_name.to_string(),
                                new_type,
                            ));
                            responses.push(PulseGraphResponse::UpdatePolymorphicTypes(node_id));
                        };
                        type_selection_widget(
                            ui,
                            (node_id, param_name),
                            value,
                            type_list,
                            callback
                        );
                    });
                }
                PulseGraphValueType::EventBindingChoice { value } => {
                    ui.horizontal(|ui| {
                        ui.label("Event");
                        ComboBox::from_id_salt(node_id)
                            .width(0.0)
                            .selected_text(
                                user_state.bindings
                                    .find_event_by_id(*value)
                                    .map_or("[INVALID]", |e| e.displayname.as_str())
                            )
                            .show_ui(ui, |ui| {
                                for event in user_state.bindings.events.iter() {
                                    let str = event.displayname.as_str();
                                    if ui
                                        .selectable_value::<EventBindingIndex>(
                                            value,
                                            event.id,
                                            str,
                                        )
                                        .clicked()
                                    {
                                        responses.push(PulseGraphResponse::ChangeEventBinding(
                                            node_id,
                                            event.clone(),
                                        ));
                                    }
                                }
                            });
                    });
                }
                PulseGraphValueType::LibraryBindingChoice { value: _ } => { /* hidden */ }
                PulseGraphValueType::HookBindingChoice { value } => {
                    ui.horizontal(|ui| {
                        ui.label("Hook");
                        ComboBox::from_id_salt((param_name, node_id))
                            .width(0.0)
                            .selected_text(
                                user_state.bindings
                                    .find_hook_by_id(*value)
                                    .map_or("[INVALID]", |h| h.displayname.as_str())
                            )
                            .show_ui(ui, |ui| {
                                for hook in user_state.bindings.hooks.iter() {
                                    let str = hook.displayname.as_str();
                                    ui.selectable_value::<HookBindingIndex>(
                                        value,
                                        hook.id,
                                        str,
                                    );
                                }
                            });
                    });
                }
                PulseGraphValueType::NodeChoice { node } => {
                    ui.horizontal(|ui| {
                        ui.label("Node");
                        let node_name = match node {
                            Some(n) => user_state
                                .exposed_nodes
                                .get(*n)
                                .map(|s| s.as_str())
                                .unwrap_or("-- CHOOSE --"),
                            None => "-- CHOOSE --",
                        };
                        ComboBox::from_id_salt(node_id)
                            .width(0.0)
                            .selected_text(node_name)
                            .show_ui(ui, |ui| {
                                for node_pair in user_state.exposed_nodes.iter() {
                                    let str: &str = node_pair.1.as_str();
                                    if ui
                                        .selectable_value::<Option<NodeId>>(
                                            node,
                                            Some(node_pair.0),
                                            str,
                                        )
                                        .clicked()
                                    {
                                        responses.push(PulseGraphResponse::ChangeRemoteNodeId(
                                            node_id,
                                            node_pair.0,
                                        ));
                                    }
                                }
                            });
                    });
                }
                PulseGraphValueType::Any => {
                    ui.label(format!("Any {param_name}"));
                }
                PulseGraphValueType::SchemaEnum { enum_type, value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ComboBox::from_id_salt((node_id, param_name))
                            .width(0.0)
                            .selected_text(value.get_ui_name())
                            .show_ui(ui, |ui| {
                                for choice in enum_type.get_all_types_as_enums().iter() {
                                    let str = choice.get_ui_name();
                                    ui.selectable_value::<SchemaEnumValue>(value, choice.clone(), str);
                                }
                            });
                    });
                }
                PulseGraphValueType::CommentBox { value } => {
                    let available_width = ui.available_width().max(100.0);
                    // same background as node, for less busy look.
                    ui.style_mut().visuals.extreme_bg_color = Color32::from_black_alpha(0);
                    ui.add_sized(
                        [available_width, 20.0], // width, height
                        egui::TextEdit::multiline(value)
                            .desired_rows(2)
                            .desired_width(available_width)
                    );
                }
                // Transforms are made from MakeTransform node, so they are not editable directly.
                PulseGraphValueType::Transform => {
                    ui.label(format!("Transform {param_name}"));
                }
                PulseGraphValueType::TransformWorldspace => {
                    ui.label(format!("Transform (world) {param_name}"));
                }
                PulseGraphValueType::Resource { resource_type, value } => {
                    ui.horizontal(|ui| {
                        if let Some(resource_type) = resource_type {
                            ui.label(format!("Resource {param_name} ({resource_type})"));
                        } else {
                            ui.label(format!("Resource {param_name}"));
                        }
                        ui.text_edit_singleline(value);
                    });
                }
                PulseGraphValueType::GameTime => {
                    ui.label(format!("Game Time {param_name}"));
                }
                PulseGraphValueType::Array => {
                    ui.label(format!("Array {param_name}"));
                }
                PulseGraphValueType::TypeSafeInteger { integer_type } => {
                    ui.label(&**integer_type);
                }
                PulseGraphValueType::GeneralEnumChoice { value } => {
                    ui.horizontal(|ui| {
                        ui.label(param_name);
                        ComboBox::from_id_salt((node_id, param_name))
                            .width(0.0)
                            .selected_text(value.to_str_ui())
                            .show_ui(ui, |ui| {
                                for choice in value.get_all_choices().iter() {
                                    let str = choice.to_str_ui();
                                    ui.selectable_value::<GeneralEnumChoice>(value, choice.clone(), str);
                                }
                            });
                    });
                }
            }
        });
        // This allows you to return your responses from the inline widgets.
        responses
    }
}

impl UserResponseTrait for PulseGraphResponse {}
impl NodeDataTrait for PulseNodeData {
    type Response = PulseGraphResponse;
    type UserState = PulseGraphState;
    type DataType = PulseDataType;
    type ValueType = PulseGraphValueType;

    fn top_bar_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        let node_template = _graph.nodes.get(_node_id).unwrap().user_data.template;
        let help_text = help::help_hover_text(node_template, user_state);
        if !help_text.is_empty() {
            _ui.label("ℹ").on_hover_text(help_text);
        }
        if let Some(node_name) = user_state.exposed_nodes.get_mut(_node_id) {
            _ui.text_edit_singleline(node_name);
        }
        vec![]
    }

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<PulseNodeData, PulseDataType, PulseGraphValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<PulseGraphResponse, PulseNodeData>>
    where
        PulseGraphResponse: UserResponseTrait,
    {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.

        let mut responses = vec![];
        // add param to event handler node.
        let node = graph.nodes.get(node_id).unwrap();
        match node.user_data.template {
            PulseNodeTemplate::IntSwitch => {
                let param = node.get_input("caselabel").expect(
                    "caselabel is not defined for IntSwitch node, this is a programming error!",
                );
                let param_value = graph
                    .get_input(param)
                    .value()
                    .clone()
                    .try_to_scalar()
                    .unwrap()
                    .round() as i32;
                if ui.button("Add parameter").clicked() {
                    let param_name = format!("{param_value}");
                    responses.push(NodeResponse::User(PulseGraphResponse::AddOutputParam(
                        node_id,
                        param_name.clone(),
                        PulseDataType::Action,
                    )));
                }
            }
            PulseNodeTemplate::NewArray => {
                let inp = graph.nodes.get(node_id).unwrap().get_input("arrayType").expect(
                    "arrayType is not defined for NewArray node, this is a programming error!",
                );
                if let Ok(typ) = graph.get_input(inp).value().clone().try_pulse_type() {
                    if ui.button("Add element").clicked() {
                        let graph_types = pulse_value_type_to_node_types(&typ);
                        responses.push(NodeResponse::User(PulseGraphResponse::AddCustomInputParam(
                            node_id,
                            Default::default(),
                            graph_types.0,
                            graph_types.1,
                            InputParamKind::ConstantOnly,
                            false,
                        )));
                    }
                }
            }
            _ => { /* no custom bottom ui */ }
        }
        responses
    }

    fn titlebar_color(
        &self,
        _ui: &egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Option<Color32> {
        match self.template {
            PulseNodeTemplate::CellPublicMethod
            | PulseNodeTemplate::EventHandler
            | PulseNodeTemplate::GraphHook
            | PulseNodeTemplate::EntOutputHandler => Some(Color32::from_rgb(186, 52, 146)),
            PulseNodeTemplate::EntFire
            | PulseNodeTemplate::FindEntByName
            | PulseNodeTemplate::FindEntitiesWithin
            | PulseNodeTemplate::IsValidEntity
            | PulseNodeTemplate::ListenForEntityOutput => Some(Color32::from_rgb(46, 191, 80)),
            PulseNodeTemplate::Compare
            | PulseNodeTemplate::CompareOutput
            | PulseNodeTemplate::CompareIf
            | PulseNodeTemplate::IntSwitch
            | PulseNodeTemplate::ForLoop
            | PulseNodeTemplate::ForEach
            | PulseNodeTemplate::WhileLoop
            | PulseNodeTemplate::And
            | PulseNodeTemplate::Not
            | PulseNodeTemplate::Or => Some(Color32::from_rgb(166, 99, 41)),
            PulseNodeTemplate::CallNode | PulseNodeTemplate::Function => {
                Some(Color32::from_rgb(28, 67, 150))
            }
            PulseNodeTemplate::Operation => Some(Color32::from_rgb(29, 181, 184)),
            PulseNodeTemplate::CellWait | PulseNodeTemplate::Timeline => {
                Some(Color32::from_rgb(184, 64, 28))
            }
            PulseNodeTemplate::GetVar 
            | PulseNodeTemplate::SetVar
            | PulseNodeTemplate::GetArrayElement => {
                Some(Color32::from_rgb(50, 125, 168))
            }
            PulseNodeTemplate::IntToString
            | PulseNodeTemplate::Convert
            | PulseNodeTemplate::StringToEntityName => Some(Color32::from_rgb(98, 41, 196)),
            PulseNodeTemplate::DebugWorldText | PulseNodeTemplate::DebugLog => None,
            PulseNodeTemplate::FireOutput => None,
            PulseNodeTemplate::GetGameTime
            | PulseNodeTemplate::SetNextThink
            | PulseNodeTemplate::InvokeLibraryBinding
            | PulseNodeTemplate::LibraryBindingAssigned { .. } => Some(Color32::from_rgb(41, 139, 196)),
            | PulseNodeTemplate::SoundEventStart => Some(Color32::from_rgb(41, 139, 196)),
            PulseNodeTemplate::ConstantBool
            | PulseNodeTemplate::ConstantFloat
            | PulseNodeTemplate::ConstantString
            | PulseNodeTemplate::ConstantInt
            | PulseNodeTemplate::ConstantVec3 
            | PulseNodeTemplate::NewArray => Some(Color32::from_rgb(77, 100, 105)),
            PulseNodeTemplate::ConcatString
            | PulseNodeTemplate::Comment
            | PulseNodeTemplate::SetAnimGraphParam
            | PulseNodeTemplate::ReturnValue
            | PulseNodeTemplate::ScaleVector
            | PulseNodeTemplate::RandomFloat
            | PulseNodeTemplate::RandomInt => None,
        }
    }

    fn output_ui(
        &self,
        ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        param_name: &str,
    ) -> Vec<NodeResponse<Self::Response, Self>> {
        let mut responses = vec![];
        ui.horizontal(|ui| {
            if self.template.has_user_addable_outputs()
                // don't allow removing of the default outflowing action that's present in most nodes.
                // can't figure out the exact data type from the lib, so that's the best we got.
                && !matches!(param_name, "outAction") 
                && ui.button("X").on_hover_text("Remove").clicked() {
                responses.push(
                    NodeResponse::User(PulseGraphResponse::RemoveOutputParam(_node_id, param_name.to_string()))
                );
            }
            ui.label(param_name);
        });
        responses
    }
}

#[cfg(feature = "nongame_asset_build")]
impl Default for EditorConfig {
    fn default() -> Self {
        EditorConfig {
            // TODO: it could be python3 on Linux
            python_interpreter: String::from("python"),
            assetassembler_path: PathBuf::from(""),
            red2_template_path: PathBuf::from("graph_red2_template.kv3"),
        }
    }
}