use super::types::{PulseGraph, PulseGraphState, PulseGraphValueType, PulseNodeTemplate};
use crate::typing::{EnumBindingIndex, EnumBindingValueIndex, PulseValueType};
use egui_node_graph2::{NodeId, OutputId};

fn enum_category(typ: &PulseValueType) -> Option<EnumBindingIndex> {
    match typ {
        PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(category, _) => *category,
        _ => None,
    }
}

/// Resolve the specific output, rather than assuming every output of a node
/// has the same category. UI port types only record the broad SchemaEnum type.
fn output_enum(
    graph: &PulseGraph,
    state: &PulseGraphState,
    output: OutputId,
) -> Option<EnumBindingIndex> {
    let port = graph.outputs.get(output)?;
    let node = graph.nodes.get(port.node)?;
    let output_name = &node.outputs.iter().find(|(_, id)| *id == output)?.0;
    let input_value = |name| {
        graph
            .inputs
            .get(node.get_input(name).ok()?)
            .map(|input| &input.value)
    };
    match node.user_data.template {
        PulseNodeTemplate::GetVar => {
            let PulseGraphValueType::InternalVariableName { value, .. } =
                input_value("variableName")?
            else {
                return None;
            };
            let variable = state.get_variable_from_name(value)?;
            let PulseGraphValueType::SchemaEnumChoice { enum_type, .. } = &variable.stored_value
            else {
                return None;
            };
            Some(*enum_type)
        }
        PulseNodeTemplate::LibraryBindingAssigned { binding } => {
            let binding = state.bindings.find_function_by_id(binding)?;
            let param = binding.find_outparam_by_name(output_name)?;
            if param.polymorphic_arg.is_some()
                || (output_name == "retval" && binding.polymorphic_return.is_some())
            {
                node.user_data
                    .custom_output_type
                    .as_ref()
                    .and_then(enum_category)
                    .or_else(|| enum_category(&param.pulsetype))
            } else {
                enum_category(&param.pulsetype)
            }
        }
        PulseNodeTemplate::EventHandler => {
            let PulseGraphValueType::EventBindingChoice { value } = input_value("event")? else {
                return None;
            };
            let binding = state.bindings.find_event_by_id(*value)?;
            enum_category(
                &binding
                    .inparams
                    .as_ref()?
                    .iter()
                    .find(|param| &param.name == output_name)?
                    .pulsetype,
            )
        }
        PulseNodeTemplate::Convert => {
            let PulseGraphValueType::Typ { value } = input_value("typeto")? else {
                return None;
            };
            enum_category(value)
        }
        _ => node
            .user_data
            .custom_output_type
            .as_ref()
            .and_then(enum_category),
    }
}

/// Update a single Compare after an operand connection or source type changes.
/// The inferred category is part of node data, so undo and load restore it.
pub(super) fn update_compare_enum(
    graph: &mut PulseGraph,
    state: &PulseGraphState,
    node_id: NodeId,
) {
    let Some(node) = graph.nodes.get(node_id) else {
        return;
    };
    if !matches!(
        node.user_data.template,
        PulseNodeTemplate::Compare | PulseNodeTemplate::CompareOutput
    ) {
        return;
    }
    let category = ["A", "B"].into_iter().find_map(|name| {
        let input = node.get_input(name).ok()?;
        graph
            .connections
            .get(input)?
            .iter()
            .find_map(|output| output_enum(graph, state, *output))
    });
    graph[node_id].user_data.inferred_enum_category = category;
    let Some(category) = category else { return };
    for name in ["A", "B"] {
        let Ok(input) = graph[node_id].get_input(name) else {
            continue;
        };
        if graph
            .connections
            .get(input)
            .is_some_and(|connections| !connections.is_empty())
        {
            continue;
        }
        if let PulseGraphValueType::SchemaEnumChoice {
            enum_type,
            enum_variant,
        } = &mut graph[input].value
        {
            if *enum_type != category {
                *enum_type = category;
                *enum_variant = EnumBindingValueIndex::default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::types::{PulseDataType, PulseNodeData};
    use crate::bindings::{FunctionBinding, LibraryBindingType, ParamInfo};
    use crate::pulsetypes::PulseVariable;
    use crate::typing::LibraryBindingIndex;
    use egui_node_graph2::{InputId, InputParamKind, NodeTemplateTrait};

    fn choice(category: u32, variant: usize) -> PulseGraphValueType {
        PulseGraphValueType::SchemaEnumChoice {
            enum_type: EnumBindingIndex(category),
            enum_variant: EnumBindingValueIndex(variant),
        }
    }

    #[test]
    fn static_widget_selects_category_then_variant() {
        use crate::bindings::{BindingEnum, EnumVariant};
        use eframe::egui::{self, Event, PointerButton, Pos2, Rect, Vec2};
        use egui_node_graph2::WidgetValueTrait;

        let ctx = egui::Context::default();
        let mut state = PulseGraphState::default();
        state.bindings.enums = [0, 7]
            .into_iter()
            .map(|id| BindingEnum {
                id: EnumBindingIndex(id),
                name: format!("Category{id}"),
                name_ui: format!("Category{id}"),
                variants: (0..2)
                    .map(|variant| EnumVariant {
                        name: format!("Variant{id}_{variant}"),
                        name_ui: format!("Variant{id}_{variant}"),
                    })
                    .collect(),
            })
            .collect();
        let mut graph = PulseGraph::default();
        let (node, input, _) = compare(&mut graph, PulseNodeTemplate::CompareOutput);
        let mut value = choice(0, 1);
        let mut frame = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        value.value_widget(
                            "A",
                            node,
                            ui,
                            &mut state,
                            &graph[node].user_data,
                            input,
                        );
                    });
                },
            )
        };
        let text_position = |output: &egui::FullOutput, text: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(shape) if shape.galley.job.text == text => {
                        Some(shape.visual_bounding_rect().center())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Missing UI text: {text}"))
        };
        let click = |pos| {
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        // Warm up egui's hit testing before clicking each newly opened popup.
        frame(vec![]);
        let output = frame(vec![]);
        frame(click(text_position(&output, "Category0")));
        let output = frame(vec![]);
        frame(click(text_position(&output, "Category7")));
        let output = frame(vec![]);
        // Changing category must reset the previous variant and render the new list.
        frame(click(text_position(&output, "Variant7_0")));
        let output = frame(vec![]);
        frame(click(text_position(&output, "Variant7_1")));
        drop(frame);
        assert_eq!(value, choice(7, 1));
    }

    fn compare(graph: &mut PulseGraph, template: PulseNodeTemplate) -> (NodeId, InputId, InputId) {
        let node = graph.add_node(
            "Compare".into(),
            PulseNodeData {
                template,
                ..Default::default()
            },
            |_, _| {},
        );
        let mut input = |name: &str| {
            graph.add_input_param(
                node,
                name.into(),
                PulseDataType::SchemaEnum,
                choice(0, 2),
                InputParamKind::ConnectionOrConstant,
                true,
            )
        };
        let a = input("A");
        let b = input("B");
        (node, a, b)
    }

    #[test]
    fn infers_from_either_operand_and_tracks_variable_changes_and_disconnects() {
        for template in [PulseNodeTemplate::Compare, PulseNodeTemplate::CompareOutput] {
            for connect_a in [true, false] {
                let mut graph = PulseGraph::default();
                let mut state = PulseGraphState::default();
                state.variables.push(PulseVariable {
                    name: "mode".into(),
                    data_type: PulseDataType::SchemaEnum,
                    stored_value: choice(7, 1),
                    typ_and_default_value: Default::default(),
                    default_value_buffer: String::new(),
                });
                let source = graph.add_node(
                    "Get variable".into(),
                    PulseNodeData {
                        template: PulseNodeTemplate::GetVar,
                        ..Default::default()
                    },
                    |_, _| {},
                );
                PulseNodeTemplate::GetVar.build_node(&mut graph, &mut state, source);
                let selector = graph[source].get_input("variableName").unwrap();
                graph[selector].value = PulseGraphValueType::InternalVariableName {
                    value: "mode".into(),
                    prevvalue: String::new(),
                };
                let output =
                    graph.add_output_param(source, "value".into(), PulseDataType::SchemaEnum);
                let (node, a, b) = compare(&mut graph, template);
                let (connected, constant) = if connect_a { (a, b) } else { (b, a) };
                graph.add_connection(output, connected, 0);
                update_compare_enum(&mut graph, &state, node);
                assert_eq!(graph[constant].value, choice(7, 0));
                assert_eq!(
                    graph[node].user_data.inferred_enum_category,
                    Some(EnumBindingIndex(7))
                );
                // A chosen variant survives repeated source updates in the same category.
                graph[constant].value = choice(7, 1);
                update_compare_enum(&mut graph, &state, node);
                assert_eq!(graph[constant].value, choice(7, 1));
                state.variables[0].stored_value = choice(12, 0);
                update_compare_enum(&mut graph, &state, node);
                assert_eq!(graph[constant].value, choice(12, 0));
                graph.connections.remove(connected);
                update_compare_enum(&mut graph, &state, node);
                assert_eq!(graph[node].user_data.inferred_enum_category, None);
                graph[constant].value = choice(99, 3);
                update_compare_enum(&mut graph, &state, node);
                assert_eq!(graph[constant].value, choice(99, 3));
            }
        }
    }

    #[test]
    fn resolves_each_library_output_and_leaves_two_connected_operands_alone() {
        let mut graph = PulseGraph::default();
        let mut state = PulseGraphState::default();
        state.bindings.gamefunctions.push(FunctionBinding {
            id: LibraryBindingIndex(1),
            typ: LibraryBindingType::Value,
            displayname: "Enums".into(),
            libname: "Enums".into(),
            description: None,
            inparams: None,
            polymorphic_return: None,
            outparams: Some(
                [7, 12]
                    .into_iter()
                    .map(|category| ParamInfo {
                        name: format!("enum{category}"),
                        typ: String::new(),
                        pulsetype: PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(
                            Some(EnumBindingIndex(category)),
                            None,
                        ),
                        polymorphic_arg: None,
                    })
                    .collect(),
            ),
        });
        let source = graph.add_node(
            "Enums".into(),
            PulseNodeData {
                template: PulseNodeTemplate::LibraryBindingAssigned {
                    binding: LibraryBindingIndex(1),
                },
                ..Default::default()
            },
            |_, _| {},
        );
        let out7 = graph.add_output_param(source, "enum7".into(), PulseDataType::SchemaEnum);
        let out12 = graph.add_output_param(source, "enum12".into(), PulseDataType::SchemaEnum);
        let (node, a, b) = compare(&mut graph, PulseNodeTemplate::CompareOutput);
        graph.add_connection(out12, a, 0);
        update_compare_enum(&mut graph, &state, node);
        assert_eq!(graph[b].value, choice(12, 0));
        graph.add_connection(out7, a, 0);
        update_compare_enum(&mut graph, &state, node);
        assert_eq!(graph[b].value, choice(7, 0));
        graph.add_connection(out12, b, 0);
        let previous = graph[b].value.clone();
        update_compare_enum(&mut graph, &state, node);
        assert_eq!(graph[b].value, previous);
        assert_eq!(graph.connection(a), Some(out7));
        assert_eq!(graph.connection(b), Some(out12));
    }

    #[test]
    fn polymorphic_array_element_updates_downstream_compare() {
        let mut editor = crate::app::PulseGraphEditor::default();
        let graph = &mut editor.full_state.state.graph;
        let source = graph.add_node(
            "Array element".into(),
            PulseNodeData {
                template: PulseNodeTemplate::GetArrayElement,
                ..Default::default()
            },
            |_, _| {},
        );
        let output = graph.add_output_param(source, "out".into(), PulseDataType::SchemaEnum);
        let (node, a, b) = compare(graph, PulseNodeTemplate::CompareOutput);
        graph.add_connection(output, a, 0);
        for category in [7, 12] {
            editor
                .update_polymorphic_output_types(
                    source,
                    Some(PulseValueType::PVAL_ARRAY(Box::new(
                        PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(
                            Some(EnumBindingIndex(category)),
                            None,
                        ),
                    ))),
                    Some("array"),
                )
                .unwrap();
            let graph = &editor.full_state.state.graph;
            assert_eq!(
                graph[node].user_data.inferred_enum_category,
                Some(EnumBindingIndex(category))
            );
            assert_eq!(graph[b].value, choice(category, 0));
        }
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn load_and_undo_restore_node_category_without_recalculating() {
        use crate::app::FullGraphState;
        use eframe::egui::util::undoer::Undoer;
        let mut full = FullGraphState::default();
        let (node, _, b) = compare(&mut full.state.graph, PulseNodeTemplate::CompareOutput);
        full.state.graph[node].user_data.inferred_enum_category = Some(EnumBindingIndex(7));
        full.state.graph[b].value = choice(7, 1);
        let serialized = ron::to_string(&full).unwrap();
        let loaded: FullGraphState = ron::from_str(&serialized).unwrap();
        assert_eq!(
            loaded.state.graph[node].user_data.inferred_enum_category,
            Some(EnumBindingIndex(7))
        );
        assert_eq!(loaded.state.graph[b].value, choice(7, 1));

        let mut undoer = Undoer::default();
        undoer.feed_state(0.0, &full);
        full.state.graph[node].user_data.inferred_enum_category = None;
        full.state.graph[b].value = choice(12, 0);
        undoer.feed_state(1.0, &full);
        let previous = undoer.undo(&full).unwrap().clone();
        full.load_from_state(previous);
        assert_eq!(
            full.state.graph[node].user_data.inferred_enum_category,
            Some(EnumBindingIndex(7))
        );
        assert_eq!(full.state.graph[b].value, choice(7, 1));
    }
}
