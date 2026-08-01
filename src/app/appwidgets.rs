use eframe::egui::{self, ComboBox, TextStyle, Ui};
use egui_node_graph2::DataTypeTrait;
use crate::app::types::{PulseDataType, PulseGraphValueType};
use crate::pulsetypes::{OutputDefinition, PulseVariable};
use crate::bindings::GraphBindings;
use crate::typing::EnumBindingValueIndex;

pub fn value_picker_widget_from_datatype(ui: &mut Ui, value: &mut PulseGraphValueType, bindings: &GraphBindings, combo_idx: Option<usize>) {
    match value {
        PulseGraphValueType::Scalar { value } => {
            ui.add(egui::DragValue::new(value));
        }
        PulseGraphValueType::Integer { value } => {
            ui.add(egui::DragValue::new(value));
        }
        PulseGraphValueType::Bool {value} => {
            ui.checkbox(value, "");
        }
        PulseGraphValueType::Vec2 {value} => {
            ui.add(egui::DragValue::new(&mut value.x).prefix("X: "));
            ui.add(egui::DragValue::new(&mut value.y).prefix("Y: "));
        }
        PulseGraphValueType::Vec3 {value}
        | PulseGraphValueType::Vec3Local {value}
        | PulseGraphValueType::QAngle {value} => {
            ui.add(egui::DragValue::new(&mut value.x).prefix("X: "));
            ui.add(egui::DragValue::new(&mut value.y).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut value.z).prefix("Z: "));
        }
        PulseGraphValueType::Vec4 {value} => {
            ui.add(egui::DragValue::new(&mut value.x).prefix("X: "));
            ui.add(egui::DragValue::new(&mut value.y).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut value.z).prefix("Z: "));
            ui.add(egui::DragValue::new(&mut value.w).prefix("W: "));
        }
        PulseGraphValueType::Color {value} => {
            let mut color = [value[0], value[1], value[2]];
            // egui doesn't allow length 4 for just RGB.
            if ui.color_edit_button_rgb(&mut color).changed() {
                value[0] = color[0];
                value[1] = color[1];
                value[2] = color[2];
            }
        }
        PulseGraphValueType::Resource {resource_type, value} => {
            if ui.add(egui::TextEdit::singleline(resource_type.get_or_insert_with(Default::default))
                .hint_text("Type")
                .desired_width(40.0)).changed() 
                && resource_type.get_or_insert_with(Default::default).trim().is_empty() {
                    *resource_type = None;
                }
    
            ui.add(egui::TextEdit::singleline(value).hint_text("Resource path"));
        }
        PulseGraphValueType::Array {array_type} => {
            // TODO: make it recursive, so we can have more nested types.
            ComboBox::from_id_salt(format!("array_type_{:?}", combo_idx))
                .selected_text(array_type.name())
                .show_ui(ui, |ui| {
                    for typ in PulseDataType::get_variable_supported_types() {
                        let name = typ.name();
                        ui.selectable_value(array_type,
                            typ.clone(),
                            name
                        );
                    }
                });
        }
        PulseGraphValueType::String { value }
        | PulseGraphValueType::SoundEventName { value }
        | PulseGraphValueType::EntityName { value } => {
            ui.text_edit_singleline(value);
        },
        PulseGraphValueType::TypeSafeInteger { integer_type } => {
            ui.add(egui::TextEdit::singleline(integer_type).hint_text("Type-safe integer type"));
        }
        PulseGraphValueType::SchemaEnumChoice { enum_type, enum_variant } => {
            let binding_enum= bindings.find_enum_by_id(*enum_type);
            ui.vertical(|ui| {
                ComboBox::from_id_salt(format!("enum_choice_{:?}", combo_idx))
                    .selected_text(binding_enum.map(|e| e.name()).unwrap_or_else(|| "<Unrecognized>"))
                    .show_ui(ui, |ui| {
                        for binding_enum in bindings.list_enums() {
                            if ui.selectable_value(enum_type,
                                binding_enum.id,
                                binding_enum.name()
                            ).changed() {
                                // reset variant
                                *enum_variant = EnumBindingValueIndex::default();
                            }
                        }
                    });
                
                if let Some(binding_enum) = binding_enum {
                    let binding_enum_variant = binding_enum.get_variant_by_id(*enum_variant);
                    ComboBox::from_id_salt(format!("enum_variant_choice_{:?}", combo_idx))
                        .selected_text(binding_enum_variant.map(|v| v.name()).unwrap_or_else(|| "<Unrecognized>"))
                        .show_ui(ui, |ui| {
                            for (id, binding_enum_variant) in binding_enum.get_all_variants().iter().enumerate() {
                                ui.selectable_value(enum_variant,
                                    EnumBindingValueIndex(id),
                                    binding_enum_variant.name()
                                );
                            }
                        });
                }
            });
        }
        _ => {
            ui.label("Can not provide a default value for this type");
        }
    }
}

// Does not let change the default value, but lets change the inner type, for example array type, or resouce type.
pub fn inner_type_choice_widget_from_datatype(ui: &mut Ui, value: &mut PulseGraphValueType, bindings: &GraphBindings, combo_idx: Option<usize>) {
    match value {
        PulseGraphValueType::Array { array_type } => {
            ComboBox::from_id_salt(format!("array_type_inner_{:?}", combo_idx))
                .selected_text(array_type.name())
                .show_ui(ui, |ui| {
                    for typ in PulseDataType::get_variable_supported_types() {
                        let name = typ.name();
                        ui.selectable_value(array_type,
                            typ.clone(),
                            name
                        );
                    }
                });
        }
        #[allow(clippy::collapsible_match)]
        PulseGraphValueType::Resource { resource_type, .. } => {
            if ui.add(egui::TextEdit::singleline(resource_type.get_or_insert_with(Default::default))
                .hint_text("Type")).changed() && resource_type.get_or_insert_with(Default::default).trim().is_empty() {
                    *resource_type = None;
                }
        }
        PulseGraphValueType::TypeSafeInteger { integer_type } => {
            ui.add(egui::TextEdit::singleline(integer_type).hint_text("Type-safe integer type"));
        }
        PulseGraphValueType::SchemaEnumChoice { enum_type, enum_variant } => {
            let binding_enum= bindings.find_enum_by_id(*enum_type);
            ComboBox::from_id_salt(format!("enum_choice_{:?}", combo_idx))
                .selected_text(binding_enum.map(|e| e.name()).unwrap_or_else(|| "<Unrecognized>"))
                .show_ui(ui, |ui| {
                    for binding_enum in bindings.list_enums() {
                        if ui.selectable_value(enum_type,
                            binding_enum.id,
                            binding_enum.name()
                        ).changed() {
                            // reset variant
                            *enum_variant = EnumBindingValueIndex::default();
                        }
                    }
                });
        }
        _ => {}
    }
}

pub fn variable_list_widget(ui: &mut Ui, variable_list: &mut [PulseVariable], type_choices: Vec<PulseDataType>, bindings: &GraphBindings) -> Option<usize> {
    let mut variable_idx_scheduled_for_deletion: Option<usize> = None;
    for (idx, var) in variable_list.iter_mut().enumerate() {
        ui.add_space(4.0);
        egui::Frame::default()
            .inner_margin(8.0)
            .fill(egui::Color32::from_rgba_unmultiplied(36, 36, 36, 255))
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("X").clicked() {
                    variable_idx_scheduled_for_deletion = Some(idx);
                }
                ui.add(egui::TextEdit::singleline(&mut var.name)
                    .font(TextStyle::Heading)
                    .hint_text("Variable name")
                );
            });
            ui.horizontal(|ui| {
                ui.label("Param type");
                ComboBox::from_id_salt(format!("var{idx}"))
                    .selected_text(var.data_type.name())
                    .show_ui(ui, |ui| {
                        for typ in type_choices.iter() {
                            if ui.selectable_value(
                                &mut var.data_type,
                                typ.clone(), 
                                typ.name()
                            ).clicked() {
                                var.stored_value = var.data_type.clone().into();
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Starting value:");
                value_picker_widget_from_datatype(ui, &mut var.stored_value, bindings, Some(idx));
            });
        });
    }
    variable_idx_scheduled_for_deletion
}

pub fn public_output_list_widget(ui: &mut Ui, output_list: &mut [OutputDefinition], bindings: &GraphBindings) -> Option<usize> {
    let mut output_scheduled_for_deletion: Option<usize> = None;
    for (idx, outputdef) in output_list.iter_mut().enumerate() {
        ui.add_space(4.0);
        egui::Frame::default()
            .inner_margin(8.0)
            .fill(egui::Color32::from_rgba_unmultiplied(36, 36, 36, 255))
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("X").clicked() {
                    output_scheduled_for_deletion = Some(idx);
                }
                ui.add(egui::TextEdit::singleline(&mut outputdef.name)
                    .font(TextStyle::Heading)
                    .hint_text("Output name")
                );
            });
            ui.horizontal(|ui| {
                ui.label("Param type");
                ComboBox::from_id_salt(format!("output{idx}"))
                    .selected_text(outputdef.data_type.name())
                    .show_ui(ui, |ui| {
                        for typ in PulseDataType::get_variable_supported_types() {
                            if ui.selectable_value(&mut outputdef.data_type,
                                typ.clone(),
                                typ.name()
                            ).clicked() {
                                outputdef.value_type = outputdef.data_type.clone().into();
                            }
                        }
                    });
                inner_type_choice_widget_from_datatype(ui, &mut outputdef.value_type, bindings, Some(idx));
            });
            
            // if outputdef.typ != outputdef.typ_old {
            //     let node_ids: Vec<_> = self.full_state.state.graph.iter_nodes().collect();
            //     for nodeid in node_ids {
            //         let node = self.full_state.state.graph.nodes.get(nodeid).unwrap();
            //         if node.user_data.template == PulseNodeTemplate::FireOutput {
            //             let inp = node.get_input("outputName");
            //             let val = self
            //                 .full_state
            //                 .state
            //                 .graph
            //                 .get_input(inp.unwrap())
            //                 .value()
            //                 .clone()
            //                 .try_output_name()
            //                 .unwrap();
            //             if outputdef.name == val {
            //                 output_node_updates.push((nodeid, outputdef.name.clone()));
            //             }
            //         }
            //     }
            //     outputdef.typ_old = outputdef.typ.clone();
            // }
        });
    }
    output_scheduled_for_deletion
}