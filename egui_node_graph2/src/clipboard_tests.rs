use super::*;
use std::borrow::Cow;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TestData;

#[derive(Clone, Debug, Default, PartialEq)]
struct TestValue(i32);

type Names = SecondaryMap<NodeId, String>;
type Editor = GraphEditorState<TestData, TestData, TestValue, TestData, Names>;
type TestGraph = Graph<TestData, TestData, TestValue>;

impl UserResponseTrait for TestData {}
impl DataTypeTrait<Names> for TestData {
    fn data_type_color(&self, _: &mut Names) -> Color32 {
        Color32::WHITE
    }
    fn name(&self) -> Cow<'_, str> {
        "test".into()
    }
}
impl WidgetValueTrait for TestValue {
    type Response = TestData;
    type UserState = Names;
    type NodeData = TestData;
    fn value_widget(
        &mut self,
        _: &str,
        _: NodeId,
        _: &mut Ui,
        _: &mut Names,
        _: &TestData,
        _: InputId,
    ) -> Vec<TestData> {
        vec![]
    }
}
impl NodeDataTrait for TestData {
    type Response = TestData;
    type UserState = Names;
    type DataType = TestData;
    type ValueType = TestValue;
    fn node_names(names: &mut Names) -> Option<&mut Names> {
        Some(names)
    }
    fn bottom_ui(
        &self,
        _: &mut Ui,
        _: NodeId,
        _: &TestGraph,
        _: &mut Names,
    ) -> Vec<NodeResponse<TestData, Self>> {
        vec![]
    }
}
impl NodeTemplateTrait for TestData {
    type NodeData = TestData;
    type DataType = TestData;
    type ValueType = TestValue;
    type UserState = Names;
    type CategoryType = ();
    fn node_finder_label(&self, _: &mut Names) -> Cow<'_, str> {
        "test".into()
    }
    fn node_graph_label(&self, _: &mut Names) -> String {
        "test".into()
    }
    fn node_finder_description<'a>(&'a self, _: &'a Names) -> Option<Cow<'a, str>> {
        None
    }
    fn user_data(&self, _: &mut Names) -> TestData {
        TestData
    }
    fn build_node(&self, _: &mut TestGraph, _: &mut Names, _: NodeId) {}
}
impl NodeTemplateIter for TestData {
    type Item = TestData;
    fn all_kinds(&self) -> Vec<TestData> {
        vec![]
    }
}

fn add_node(editor: &mut Editor, position: Pos2) -> (NodeId, InputId, OutputId) {
    let node = editor
        .graph
        .add_node("Function".into(), TestData, |_, _| {});
    let input = editor.graph.add_wide_input_param(
        node,
        "in".into(),
        TestData,
        TestValue(42),
        InputParamKind::ConnectionOrConstant,
        NonZeroU32::new(4),
        false,
    );
    let output = editor.graph.add_output_param(node, "out".into(), TestData);
    editor.node_order.push(node);
    editor.node_positions.insert(node, position);
    editor.node_sizes.insert(node, vec2(240.0, 100.0));
    (node, input, output)
}

#[test]
fn copies_internal_links_in_order_with_fresh_ids_and_preserves_parameters() {
    let mut editor = Editor::default();
    let mut names = Names::default();
    let (external, external_in, external_out) = add_node(&mut editor, pos2(0.0, 0.0));
    let (a, _, a_out) = add_node(&mut editor, pos2(10.0, 20.0));
    let (b, b_in, b_out) = add_node(&mut editor, pos2(310.0, 120.0));
    // The excluded connection precedes retained ones on a wide port.
    editor.graph.add_connection(external_out, b_in, 0);
    editor.graph.add_connection(a_out, b_in, 1);
    editor.graph.add_connection(b_out, b_in, 2);
    editor.graph.add_connection(a_out, external_in, 0);
    editor.selected_nodes = vec![b, a];
    editor.copy_selected_nodes(&mut names);
    let pasted = editor.paste_clipboard(pos2(100.0, 100.0), Rect::ZERO, &mut names);
    assert_eq!(pasted.len(), 2);
    assert_eq!(editor.selected_nodes, pasted);
    assert!(pasted.iter().all(|id| ![external, a, b].contains(id)));
    let new_input = editor.graph[pasted[1]].inputs[0].1;
    let new_outputs: Vec<_> = pasted
        .iter()
        .map(|id| editor.graph[*id].outputs[0].1)
        .collect();
    assert_eq!(editor.graph.connections[new_input], new_outputs);
    assert_eq!(
        editor.graph.connections[b_in],
        vec![external_out, a_out, b_out]
    );
    assert_eq!(editor.graph.connections[external_in], vec![a_out]);
    assert_ne!(new_input, b_in);
    let input = editor.graph.get_input(new_input);
    assert_eq!(input.node, pasted[1]);
    assert_eq!(input.value, TestValue(42));
    assert_eq!(input.kind, InputParamKind::ConnectionOrConstant);
    assert_eq!(input.max_connections, NonZeroU32::new(4));
    assert!(!input.shown_inline);
    assert_eq!(editor.node_sizes[pasted[0]], vec2(240.0, 100.0));
}

#[test]
fn snapshots_names_and_values_and_allocates_unique_names_on_every_paste() {
    let mut editor = Editor::default();
    let mut names = Names::default();
    let (a, input, _) = add_node(&mut editor, Pos2::ZERO);
    let (b, _, _) = add_node(&mut editor, pos2(300.0, 0.0));
    let (other, _, _) = add_node(&mut editor, Pos2::ZERO);
    names.insert(a, "Run".into());
    names.insert(b, "Run".into());
    names.insert(other, "Run_copy".into());
    editor.selected_nodes = vec![a, b];
    editor.copy_selected_nodes(&mut names);
    names.insert(a, "Changed".into());
    editor.graph[input].value = TestValue(99);
    for suffix in [2, 4] {
        let pasted = editor.paste_clipboard(Pos2::ZERO, Rect::ZERO, &mut names);
        assert_eq!(names[pasted[0]], format!("Run_copy_{suffix}"));
        assert_eq!(names[pasted[1]], format!("Run_copy_{}", suffix + 1));
        assert_eq!(
            editor.graph[editor.graph[pasted[0]].inputs[0].1].value,
            TestValue(42)
        );
    }
    assert_eq!(names[a], "Changed");
}

#[test]
fn paste_uses_cursor_pan_editor_origin_and_current_zoom_even_after_source_deletion() {
    let mut editor = Editor::default();
    let mut names = Names::default();
    let (a, _, _) = add_node(&mut editor, pos2(10.0, 20.0));
    let (b, _, _) = add_node(&mut editor, pos2(110.0, 70.0));
    editor.pan_zoom.zoom = 0.5;
    editor.selected_nodes = vec![a, b];
    editor.copy_selected_nodes(&mut names);
    editor.graph.remove_node(a);
    editor.graph.remove_node(b);
    editor.node_order.clear();
    editor.selected_nodes.clear();
    editor.copy_selected_nodes(&mut names); // Empty copy retains the snapshot.
    editor.pan_zoom.zoom = 2.0;
    editor.pan_zoom.pan = vec2(-30.0, 50.0);
    let rect = Rect::from_min_size(pos2(100.0, 200.0), vec2(800.0, 600.0));
    let cursor = pos2(500.0, 600.0);
    let pasted = editor.paste_clipboard(cursor, rect, &mut names);
    let screen = editor.node_positions[pasted[0]] + editor.pan_zoom.pan + rect.min.to_vec2();
    assert_eq!(screen, cursor);
    assert_eq!(
        editor.node_positions[pasted[1]] - editor.node_positions[pasted[0]],
        vec2(400.0, 200.0)
    );
}

#[test]
fn shortcuts_require_command_and_support_native_clipboard_events() {
    let key = |key, modifiers| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    };
    assert_eq!(
        clipboard_shortcut(&key(Key::C, Modifiers::NONE)),
        (false, false)
    );
    assert_eq!(
        clipboard_shortcut(&key(Key::V, Modifiers::NONE)),
        (false, false)
    );
    assert_eq!(
        clipboard_shortcut(&key(Key::C, Modifiers::COMMAND)),
        (true, false)
    );
    assert_eq!(
        clipboard_shortcut(&key(Key::V, Modifiers::COMMAND)),
        (false, true)
    );
    assert_eq!(clipboard_shortcut(&Event::Copy), (true, false));
    assert_eq!(
        clipboard_shortcut(&Event::Paste(String::new())),
        (false, true)
    );
}

#[test]
fn ui_pastes_once_for_native_and_key_events_and_leaves_text_editing_alone() {
    let ctx = Context::default();
    let mut editor = Editor::default();
    let mut names = Names::default();
    let (node, _, _) = add_node(&mut editor, Pos2::ZERO);
    editor.selected_nodes = vec![node];
    editor.copy_selected_nodes(&mut names);
    let mut text = String::new();
    for focused in [false, true] {
        // egui hit testing uses the preceding frame's widget rectangles.
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 800.0))),
                events: vec![Event::PointerMoved(pos2(700.0, 500.0))],
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    if focused {
                        ui.text_edit_singleline(&mut text).request_focus();
                    }
                    let _ = editor.draw_graph_editor(ui, TestData, &mut names, vec![]);
                });
            },
        );
        if !focused {
            editor.clipboard = None;
            let output = ctx.run(
                RawInput {
                    events: vec![Event::Copy],
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        let _ = editor.draw_graph_editor(ui, TestData, &mut names, vec![]);
                    });
                },
            );
            assert!(editor.clipboard.is_some());
            assert!(output.platform_output.commands.iter().any(
                |command| matches!(command, egui::OutputCommand::CopyText(text) if !text.is_empty())
            ));
        }
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 800.0))),
            events: vec![
                Event::PointerMoved(pos2(700.0, 500.0)),
                Event::Paste("text".into()),
                Event::Key {
                    key: Key::V,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::COMMAND,
                },
            ],
            ..Default::default()
        };
        let count = editor.graph.nodes.len();
        let _ = ctx.run(input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                if focused {
                    ui.text_edit_singleline(&mut text).request_focus();
                }
                let response = editor.draw_graph_editor(ui, TestData, &mut names, vec![]);
                assert_eq!(
                    response
                        .node_responses
                        .iter()
                        .filter(|r| matches!(r, NodeResponse::PastedNodes(_)))
                        .count(),
                    usize::from(!focused)
                );
            });
        });
        assert_eq!(editor.graph.nodes.len(), count + usize::from(!focused));
        if focused {
            assert_eq!(text, "text");
        }
    }
}
