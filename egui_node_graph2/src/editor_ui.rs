use std::collections::HashSet;
use std::num::NonZeroU32;

use crate::color_hex_utils::*;
use crate::utils::ColorUtils;

use super::*;
use egui::epaint::{CubicBezierShape, RectShape};
use egui::*;

/// Mapping from parameter id to positions of hooks it contains.
///
/// Outputs and short inputs always only have one hook, so the value is
/// just `vec![port_position]`. Wide inputs may have multiple hooks.
pub type PortLocations = std::collections::HashMap<AnyParameterId, Vec<Pos2>>;

/// Destination positions of connections made to a given input.
///
/// This is not equivalent to [`PortLocations`] because connections may be moved
/// around (e.g. while an in-progress connection is hovered over a wide port),
/// while hooks within a port are strictly a function of the port.
pub type ConnLocations = std::collections::HashMap<InputId, Vec<Pos2>>;

/// Rectangle containing each node.
pub type NodeRects = std::collections::HashMap<NodeId, Rect>;

const DISTANCE_TO_CONNECT: f32 = 10.0;
const NODE_INITIAL_SIZE: Vec2 = Vec2::new(200.0, 200.0);
const NODE_MAX_SIZE: Vec2 = Vec2::new(800.0, 800.0);

/// Nodes communicate certain events to the parent graph when drawn. There is
/// one special `User` variant which can be used by users as the return value
/// when executing some custom actions in the UI of the node.
#[derive(Clone, Debug)]
pub enum NodeResponse<UserResponse: UserResponseTrait, NodeData: NodeDataTrait> {
    ConnectEventStarted(NodeId, AnyParameterId),
    ConnectEventEnded {
        output: OutputId,
        input: InputId,
        /// Index of the connection in wide input ports.
        ///
        /// If the input isn't a wide port this is always 0 and may be ignored.
        input_hook: usize,
    },
    CreatedNode(NodeId),
    SelectNode(NodeId),
    /// As a user of this library, prefer listening for `DeleteNodeFull` which
    /// will also contain the user data for the deleted node.
    DeleteNodeUi(NodeId),
    /// Emitted when a node is deleted. The node will no longer exist in the
    /// graph after this response is returned from the draw function, but its
    /// contents are passed along with the event.
    DeleteNodeFull {
        node_id: NodeId,
        node: Node<NodeData>,
    },
    DisconnectEvent {
        output: OutputId,
        input: InputId,
    },
    /// Emitted when a node is interacted with, and should be raised
    RaiseNode(NodeId),
    MoveNode {
        node: NodeId,
        drag_delta: Vec2,
    },
    ResizeNode {
        node: NodeId,
        resize_delta: Vec2,
    },
    User(UserResponse),
}

/// The return value of [`draw_graph_editor`]. This value can be used to make
/// user code react to specific events that happened when drawing the graph.
#[derive(Clone, Debug)]
pub struct GraphResponse<UserResponse: UserResponseTrait, NodeData: NodeDataTrait> {
    /// Events that occurred during this frame of rendering the graph. Check the
    /// [`UserResponse`] type for a description of each event.
    pub node_responses: Vec<NodeResponse<UserResponse, NodeData>>,
    /// Is the mouse currently hovering the graph editor? Note that the node
    /// finder is considered part of the graph editor, even when it floats
    /// outside the graph editor rect.
    pub cursor_in_editor: bool,
    /// Is the mouse currently hovering the node finder?
    pub cursor_in_finder: bool,
}
impl<UserResponse: UserResponseTrait, NodeData: NodeDataTrait> Default
    for GraphResponse<UserResponse, NodeData>
{
    fn default() -> Self {
        Self {
            node_responses: Default::default(),
            cursor_in_editor: false,
            cursor_in_finder: false,
        }
    }
}
pub struct GraphNodeWidget<'a, NodeData, DataType, ValueType> {
    pub position: &'a mut Pos2,
    pub graph: &'a mut Graph<NodeData, DataType, ValueType>,
    pub port_locations: &'a mut PortLocations,
    pub conn_locations: &'a mut ConnLocations,
    pub node_rects: &'a mut NodeRects,
    pub node_id: NodeId,
    pub ongoing_drag: Option<(NodeId, AnyParameterId)>,
    pub selected: bool,
    pub pan: egui::Vec2,
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserResponse, UserState, CategoryType>
    GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, UserState>
where
    NodeData: NodeDataTrait<
        Response = UserResponse,
        UserState = UserState,
        DataType = DataType,
        ValueType = ValueType,
    >,
    UserResponse: UserResponseTrait,
    ValueType:
        WidgetValueTrait<Response = UserResponse, UserState = UserState, NodeData = NodeData>,
    NodeTemplate: NodeTemplateTrait<
        NodeData = NodeData,
        DataType = DataType,
        ValueType = ValueType,
        UserState = UserState,
        CategoryType = CategoryType,
    >,
    DataType: DataTypeTrait<UserState>,
    CategoryType: CategoryTrait,
{
    #[must_use]
    pub fn draw_graph_editor(
        &mut self,
        ui: &mut Ui,
        all_kinds: impl NodeTemplateIter<Item = NodeTemplate>,
        user_state: &mut UserState,
        prepend_responses: Vec<NodeResponse<UserResponse, NodeData>>,
    ) -> GraphResponse<UserResponse, NodeData> {
        ui.set_clip_rect(ui.max_rect());
        let clip_rect = ui.clip_rect();
        // Zoom may have never taken place, so ensure we use parent style
        if !self.pan_zoom.started {
            self.zoom(ui, 1.0);
            self.pan_zoom.started = true;
        }

        // Zoom only within area where graph is shown
        if ui.rect_contains_pointer(clip_rect) {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_delta = (scroll_delta * 0.002).exp();
                self.zoom(ui, zoom_delta);
            }
        }

        // Render graph zoomed
        let zoomed_style = self.pan_zoom.zoomed_style.clone();
        let graph_response = show_zoomed(ui.style().clone(), zoomed_style, ui, |ui| {
            self.draw_graph_editor_inside_zoom(ui, all_kinds, user_state, prepend_responses)
        });

        graph_response
    }

    /// Reset zoom to 1.0
    pub fn reset_zoom(&mut self, ui: &Ui) {
        let new_zoom = 1.0 / self.pan_zoom.zoom;
        self.zoom(ui, new_zoom);
    }

    /// Zoom within the where you call `draw_graph_editor`. Use values like 1.01, or 0.99 to zoom.
    /// For example: `let zoom_delta = (scroll_delta * 0.002).exp();`
    pub fn zoom(&mut self, ui: &Ui, zoom_delta: f32) {
        // Update zoom, and styles
        let zoom_before = self.pan_zoom.zoom;
        self.pan_zoom.zoom(ui.clip_rect(), ui.style(), zoom_delta);
        if zoom_before != self.pan_zoom.zoom {
            let actual_delta = self.pan_zoom.zoom / zoom_before;
            self.update_node_positions_after_zoom(actual_delta);
        }
    }

    fn update_node_positions_after_zoom(&mut self, zoom_delta: f32) {
        // Update node positions, zoom towards center
        let half_size = self.pan_zoom.clip_rect.size() / 2.0;
        for (_id, node_pos) in self.node_positions.iter_mut() {
            // 1. Get node local position (relative to origo)
            let local_pos = node_pos.to_vec2() - half_size + self.pan_zoom.pan;
            // 2. Scale local position by zoom delta
            let scaled_local_pos = (local_pos * zoom_delta).to_pos2();
            // 3. Transform back to global position
            *node_pos = scaled_local_pos + half_size - self.pan_zoom.pan;
            // This way we can retain pan untouched when zooming :)
        }
    }

    fn draw_graph_editor_inside_zoom(
        &mut self,
        ui: &mut Ui,
        all_kinds: impl NodeTemplateIter<Item = NodeTemplate>,
        user_state: &mut UserState,
        prepend_responses: Vec<NodeResponse<UserResponse, NodeData>>,
    ) -> GraphResponse<UserResponse, NodeData> {
        // This causes the graph editor to use as much free space as it can.
        // (so for windows it will use up to the resizeably set limit
        // and for a Panel it will fill it completely)
        let editor_rect = ui.max_rect();
        let resp = ui.allocate_rect(editor_rect, Sense::hover());

        let cursor_pos = ui
            .ctx()
            .input(|i| i.pointer.hover_pos().unwrap_or(Pos2::ZERO));
        let mut cursor_in_editor = resp.contains_pointer();
        let mut cursor_in_finder = false;

        // Gets filled with the node metrics as they are drawn
        let mut port_locations = PortLocations::new();
        let mut node_rects = NodeRects::new();

        // actual dest location of each connection
        let mut conn_locations = ConnLocations::default();

        // The responses returned from node drawing have side effects that are best
        // executed at the end of this function.
        let mut delayed_responses: Vec<NodeResponse<UserResponse, NodeData>> = prepend_responses;

        // Used to detect when the background was clicked
        let mut click_on_background = false;

        // Used to detect drag events in the background
        let mut drag_started_on_background = false;
        let mut drag_released_on_background = false;

        debug_assert_eq!(
            self.node_order.iter().copied().collect::<HashSet<_>>(),
            self.graph.iter_nodes().collect::<HashSet<_>>(),
            "The node_order field of the GraphEditorself was left in an \
        inconsistent self. It has either more or less values than the graph."
        );

        // Allocate rect before the nodes, otherwise this will block the interaction
        // with the nodes.
        let r = ui.allocate_rect(ui.min_rect(), Sense::click().union(Sense::drag()));
        if r.clicked() {
            click_on_background = true;
        } else if r.drag_started() {
            drag_started_on_background = true;
        } else if r.drag_stopped() {
            drag_released_on_background = true;
        }

        /* Draw nodes */
        for node_id in self.node_order.iter().copied() {
            let responses = GraphNodeWidget {
                position: self.node_positions.get_mut(node_id).unwrap(),
                graph: &mut self.graph,
                port_locations: &mut port_locations,
                conn_locations: &mut conn_locations,
                node_rects: &mut node_rects,
                node_id,
                ongoing_drag: self.connection_in_progress,
                selected: self
                    .selected_nodes
                    .iter()
                    .any(|selected| *selected == node_id),
                pan: self.pan_zoom.pan + editor_rect.min.to_vec2(),
            }
            .show(&self.pan_zoom, ui, user_state, *self.node_sizes.get(node_id).unwrap_or(&NODE_INITIAL_SIZE));

            // Actions executed later
            delayed_responses.extend(responses);
        }

        /* Draw the node finder, if open */
        let mut should_close_node_finder = false;
        if let Some(ref mut node_finder) = self.node_finder {
            let mut node_finder_area = Area::new(Id::new("node_finder")).order(Order::Foreground);
            if let Some(pos) = node_finder.position {
                node_finder_area = node_finder_area.current_pos(pos);
            }
            node_finder_area.show(ui.ctx(), |ui| {
                if let Some(node_kind) = node_finder.show(ui, all_kinds, user_state) {
                    let new_node = self.graph.add_node(
                        node_kind.node_graph_label(user_state),
                        node_kind.user_data(user_state),
                        |graph, node_id| node_kind.build_node(graph, user_state, node_id),
                    );
                    self.node_positions.insert(
                        new_node,
                        node_finder.position.unwrap_or(cursor_pos)
                            - self.pan_zoom.pan
                            - editor_rect.min.to_vec2(),
                    );
                    self.node_sizes.insert(new_node, NODE_INITIAL_SIZE);
                    self.node_order.push(new_node);

                    should_close_node_finder = true;
                    delayed_responses.push(NodeResponse::CreatedNode(new_node));
                }
                let finder_rect = ui.min_rect();
                // If the cursor is not in the main editor, check if the cursor is in the finder
                // if the cursor is in the finder, then we can consider that also in the editor.
                if finder_rect.contains(cursor_pos) {
                    cursor_in_editor = true;
                    cursor_in_finder = true;
                }
            });
        }
        if should_close_node_finder {
            self.node_finder = None;
        }

        // draw in-progress connections
        if let Some((_, ref locator)) = self.connection_in_progress {
            let port_type = self.graph.any_param_type(*locator).unwrap();
            let connection_color = port_type.data_type_color(user_state);

            // outputs can't be wide yet so this is fine.
            let start_pos = *port_locations[locator].last().unwrap();

            // Find a port to connect to
            fn snap_to_ports<
                NodeData,
                UserState,
                DataType: DataTypeTrait<UserState>,
                ValueType,
                Key: slotmap::Key + Into<AnyParameterId>,
                Value,
            >(
                graph: &Graph<NodeData, DataType, ValueType>,
                port_type: &DataType,
                ports: &SlotMap<Key, Value>,
                port_locations: &PortLocations,
                cursor_pos: Pos2,
            ) -> Pos2 {
                ports
                    .iter()
                    .find_map(|(port_id, _)| {
                        let compatible_ports = graph
                            .any_param_type(port_id.into())
                            .map(|other| other == port_type)
                            .unwrap_or(false);

                        if compatible_ports {
                            port_locations.get(&port_id.into()).and_then(|hooks| {
                                hooks
                                    .iter()
                                    .min_by(|hook1, hook2| {
                                        hook1
                                            .distance(cursor_pos)
                                            .partial_cmp(&hook2.distance(cursor_pos))
                                            .unwrap()
                                    })
                                    .filter(|nearest_hook| {
                                        nearest_hook.distance(cursor_pos) < DISTANCE_TO_CONNECT
                                    })
                                    .copied()
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or(cursor_pos)
            }

            let (src_pos, dst_pos) = match locator {
                AnyParameterId::Output(_) => (
                    start_pos,
                    snap_to_ports(
                        &self.graph,
                        port_type,
                        &self.graph.inputs,
                        &port_locations,
                        cursor_pos,
                    ),
                ),
                AnyParameterId::Input(_) => (
                    snap_to_ports(
                        &self.graph,
                        port_type,
                        &self.graph.outputs,
                        &port_locations,
                        cursor_pos,
                    ),
                    start_pos,
                ),
            };
            draw_connection(
                &self.pan_zoom,
                ui.painter(),
                src_pos,
                dst_pos,
                connection_color,
            );
        }

        // draw existing connections
        for (input, outputs) in self.graph.iter_connection_groups() {
            for (hook_n, &output) in outputs.iter().enumerate() {
                let port_type = self
                    .graph
                    .any_param_type(AnyParameterId::Output(output))
                    .unwrap();
                let connection_color = port_type.data_type_color(user_state);
                // outputs can't be wide yet so this is fine.
                let src_pos = port_locations[&AnyParameterId::Output(output)][0];
                let dst_pos = conn_locations[&input][hook_n];
                draw_connection(
                    &self.pan_zoom,
                    ui.painter(),
                    src_pos,
                    dst_pos,
                    connection_color,
                );
            }
        }

        /* Handle responses from drawing nodes */

        // Some responses generate additional responses when processed. These
        // are stored here to report them back to the user.
        let mut extra_responses: Vec<NodeResponse<UserResponse, NodeData>> = Vec::new();

        for response in delayed_responses.iter() {
            match response {
                NodeResponse::ConnectEventStarted(node_id, port) => {
                    self.connection_in_progress = Some((*node_id, *port));
                }
                NodeResponse::ConnectEventEnded {
                    output,
                    input,
                    input_hook,
                } => self.graph.add_connection(*output, *input, *input_hook),
                NodeResponse::CreatedNode(_) => {
                    //Convenience NodeResponse for users
                }
                NodeResponse::SelectNode(node_id) => {
                    self.selected_nodes = Vec::from([*node_id]);
                }
                NodeResponse::DeleteNodeUi(node_id) => {
                    let (node, disc_events) = self.graph.remove_node(*node_id);

                    // Pass the disconnection responses first so user code can perform cleanup
                    // before node removal response.
                    extra_responses.extend(
                        disc_events
                            .into_iter()
                            .map(|(input, output)| NodeResponse::DisconnectEvent { input, output }),
                    );
                    // Pass the full node as a response so library users can
                    // listen for it and get their user data.
                    extra_responses.push(NodeResponse::DeleteNodeFull {
                        node_id: *node_id,
                        node,
                    });
                    self.node_positions.remove(*node_id);
                    self.node_sizes.remove(*node_id);
                    // Make sure to not leave references to old nodes hanging
                    self.selected_nodes.retain(|id| *id != *node_id);
                    self.node_order.retain(|id| *id != *node_id);
                }
                NodeResponse::DisconnectEvent { input, output } => {
                    let other_node = self.graph.get_output(*output).node;
                    self.graph.remove_connection(*input, *output);
                    self.connection_in_progress =
                        Some((other_node, AnyParameterId::Output(*output)));
                }
                NodeResponse::RaiseNode(node_id) => {
                    let old_pos = self
                        .node_order
                        .iter()
                        .position(|id| *id == *node_id)
                        .expect("Node to be raised should be in `node_order`");
                    self.node_order.remove(old_pos);
                    self.node_order.push(*node_id);
                }
                NodeResponse::MoveNode { node, drag_delta } => {
                    self.node_positions[*node] += *drag_delta;
                    // Handle multi-node selection movement
                    if self.selected_nodes.contains(node) && self.selected_nodes.len() > 1 {
                        for n in self.selected_nodes.iter().copied() {
                            if n != *node {
                                self.node_positions[n] += *drag_delta;
                            }
                        }
                    }
                }
                NodeResponse::ResizeNode { node, resize_delta } => {
                    // Resize the node by the delta
                    self.node_sizes[*node] = (self.node_sizes[*node] + *resize_delta).clamp(
                        NODE_INITIAL_SIZE,
                        NODE_MAX_SIZE,
                    );
                }
                NodeResponse::User(_) => {
                    // These are handled by the user code.
                }
                NodeResponse::DeleteNodeFull { .. } => {
                    unreachable!("The UI should never produce a DeleteNodeFull event.")
                }
            }
        }

        // Handle box selection
        if let Some(box_start) = self.ongoing_box_selection {
            let selection_rect = Rect::from_two_pos(cursor_pos, box_start);
            let bg_color = Color32::from_rgba_unmultiplied(200, 200, 200, 20);
            let stroke_color = Color32::from_rgba_unmultiplied(200, 200, 200, 180);
            ui.painter().rect(
                selection_rect,
                2.0,
                bg_color,
                Stroke::new(3.0, stroke_color),
            );

            self.selected_nodes = node_rects
                .into_iter()
                .filter_map(|(node_id, rect)| {
                    if selection_rect.intersects(rect) {
                        Some(node_id)
                    } else {
                        None
                    }
                })
                .collect();
        }

        // Push any responses that were generated during response handling.
        // These are only informative for the end-user and need no special
        // treatment here.
        delayed_responses.extend(extra_responses);

        /* Mouse input handling */

        // This locks the context, so don't hold on to it for too long.
        let mouse = &ui.ctx().input(|i| i.pointer.clone());

        if mouse.any_released() && self.connection_in_progress.is_some() {
            self.connection_in_progress = None;
        }

        if mouse.secondary_released() && cursor_in_editor && !cursor_in_finder {
            self.node_finder = Some(NodeFinder::new_at(cursor_pos));
        }
        if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
            self.node_finder = None;
        }

        if r.dragged()
            && ui
                .ctx()
                .input(|i| i.pointer.middle_down() || i.modifiers.command_only())
        {
            self.pan_zoom.pan += ui.ctx().input(|i| i.pointer.delta());
        }

        // Deselect and deactivate finder if the editor background is clicked,
        // *or* if the the mouse clicks off the ui
        if click_on_background || (mouse.any_click() && !cursor_in_editor) {
            self.selected_nodes = Vec::new();
            self.node_finder = None;
        }

        if drag_started_on_background
            && mouse.primary_down()
            && !ui.ctx().input(|i| i.modifiers.command_only())
        {
            self.ongoing_box_selection = Some(cursor_pos);
        }
        if mouse.primary_released() || drag_released_on_background {
            self.ongoing_box_selection = None;
        }

        GraphResponse {
            node_responses: delayed_responses,
            cursor_in_editor,
            cursor_in_finder,
        }
    }
}

fn draw_connection(
    pan_zoom: &PanZoom,
    painter: &Painter,
    src_pos: Pos2,
    dst_pos: Pos2,
    color: Color32,
) {
    let connection_stroke = egui::Stroke {
        width: 5.0 * pan_zoom.zoom,
        color,
    };

    let control_scale = ((dst_pos.x - src_pos.x) * pan_zoom.zoom / 2.0).max(30.0 * pan_zoom.zoom);
    let src_control = src_pos + Vec2::X * control_scale;
    let dst_control = dst_pos - Vec2::X * control_scale;

    let bezier = CubicBezierShape::from_points_stroke(
        [src_pos, src_control, dst_control, dst_pos],
        false,
        Color32::TRANSPARENT,
        connection_stroke,
    );

    painter.add(bezier);
}

#[derive(Clone, Copy, Debug)]
struct OuterRectMemory(Rect);

impl<'a, NodeData, DataType, ValueType, UserResponse, UserState>
    GraphNodeWidget<'a, NodeData, DataType, ValueType>
where
    NodeData: NodeDataTrait<
        Response = UserResponse,
        UserState = UserState,
        DataType = DataType,
        ValueType = ValueType,
    >,
    UserResponse: UserResponseTrait,
    ValueType:
        WidgetValueTrait<Response = UserResponse, UserState = UserState, NodeData = NodeData>,
    DataType: DataTypeTrait<UserState>,
{
    pub fn show(
        self,
        pan_zoom: &PanZoom,
        ui: &mut Ui,
        user_state: &mut UserState,
        size: Vec2,
    ) -> Vec<NodeResponse<UserResponse, NodeData>> {
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(self.node_id)
                .max_rect(
                    Rect::from_min_size(*self.position + self.pan, size),
                )
                .layout(*ui.layout())
        );
        Self::show_graph_node(self, pan_zoom, &mut child_ui, user_state)
    }

    /// Draws this node. Also fills in the list of port locations with all of its ports.
    /// Returns responses indicating multiple events.
    fn show_graph_node(
        self,
        pan_zoom: &PanZoom,
        ui: &mut Ui,
        user_state: &mut UserState,
    ) -> Vec<NodeResponse<UserResponse, NodeData>> {
        let margin = egui::vec2(15.0, 5.0) * pan_zoom.zoom;
        let mut responses = Vec::<NodeResponse<UserResponse, NodeData>>::new();

        let background_color;
        let text_color;
        if ui.visuals().dark_mode {
            background_color = color_from_hex("#3f3f3f").unwrap();
            text_color = color_from_hex("#fefefe").unwrap();
        } else {
            background_color = color_from_hex("#ffffff").unwrap();
            text_color = color_from_hex("#505050").unwrap();
        }

        ui.visuals_mut().widgets.noninteractive.fg_stroke =
            Stroke::new(2.0 * pan_zoom.zoom, text_color);

        // Preallocate shapes to paint below contents
        let outline_shape = ui.painter().add(Shape::Noop);
        let background_shape = ui.painter().add(Shape::Noop);

        let mut outer_rect_bounds = ui.available_rect_before_wrap();
        // Scale hack, otherwise some (larger) rects expand too much when zoomed out
        outer_rect_bounds.max.x =
            outer_rect_bounds.min.x + outer_rect_bounds.width() * pan_zoom.zoom;

        let mut inner_rect = outer_rect_bounds.shrink2(margin);

        // Make sure we don't shrink to the negative:
        inner_rect.max.x = inner_rect.max.x.max(inner_rect.min.x);
        inner_rect.max.y = inner_rect.max.y.max(inner_rect.min.y);

        let mut child_ui = ui.child_ui(inner_rect, *ui.layout(), None);

        // Get interaction rect from memory, it may expand after the window response on resize.
        let interaction_rect = ui
            .ctx()
            .memory_mut(|mem| {
                mem.data
                    .get_temp::<OuterRectMemory>(child_ui.id())
                    .map(|stored| stored.0)
            })
            .unwrap_or(outer_rect_bounds);
        // After 0.20, layers added over others can block hover interaction. Call this first
        // before creating the node content.
        let window_response = ui.interact(
            interaction_rect,
            Id::new((self.node_id, "window")),
            Sense::click_and_drag(),
        );
        let mut title_height = 0.0;

        let mut input_port_heights = vec![];
        let mut output_port_heights = vec![];

        child_ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    RichText::new(&self.graph[self.node_id].label)
                        .text_style(TextStyle::Button)
                        .color(text_color),
                ));
                responses.extend(self.graph[self.node_id].user_data.top_bar_ui(
                    ui,
                    self.node_id,
                    self.graph,
                    user_state,
                ));
                ui.add_space(8.0 * pan_zoom.zoom); // The size of the little cross icon
            });
            ui.add_space(margin.y);
            title_height = ui.min_size().y;

            // First pass: Draw the inner fields. Compute port heights
            let inputs = self.graph[self.node_id].inputs.clone();
            for (param_name, param_id) in inputs {
                if self.graph[param_id].shown_inline {
                    let height_before = ui.min_rect().bottom();

                    if self.graph[param_id].max_connections == NonZeroU32::new(1) {
                        // NOTE: We want to pass the `user_data` to
                        // `value_widget`, but we can't since that would require
                        // borrowing the graph twice. Here, we make the
                        // assumption that the value is cheaply replaced, and
                        // use `std::mem::take` to temporarily replace it with a
                        // dummy value. This requires `ValueType` to implement
                        // Default, but results in a totally safe alternative.
                        let mut value = std::mem::take(&mut self.graph[param_id].value);

                        if !self.graph.connections(param_id).is_empty() {
                            let node_responses = value.value_widget_connected(
                                &param_name,
                                self.node_id,
                                ui,
                                user_state,
                                &self.graph[self.node_id].user_data,
                            );

                            responses.extend(node_responses.into_iter().map(NodeResponse::User));
                        } else {
                            let node_responses = value.value_widget(
                                &param_name,
                                self.node_id,
                                ui,
                                user_state,
                                &self.graph[self.node_id].user_data,
                            );

                            responses.extend(node_responses.into_iter().map(NodeResponse::User));
                        }

                        self.graph[param_id].value = value;
                    } else {
                        ui.label(param_name);
                    }

                    let height_intermediate = ui.min_rect().bottom();

                    let max_connections = self.graph[param_id]
                        .max_connections
                        .map(NonZeroU32::get)
                        .unwrap_or(std::u32::MAX)
                        as usize;
                    let port_height = port_height(
                        max_connections != 1,
                        self.graph.connections(param_id).len(),
                        max_connections,
                    );
                    let margin = 5.0;
                    let missing_space =
                        port_height - (height_intermediate - height_before) + margin;
                    if missing_space > 0.0 {
                        ui.add_space(missing_space);
                    }

                    self.graph[self.node_id].user_data.separator(
                        ui,
                        self.node_id,
                        AnyParameterId::Input(param_id),
                        self.graph,
                        user_state,
                    );

                    let height_after = ui.min_rect().bottom();

                    input_port_heights.push((height_before + height_after) / 2.0);
                }
            }

            let outputs = self.graph[self.node_id].outputs.clone();
            for (param_name, param_id) in outputs {
                let height_before = ui.min_rect().bottom();
                responses.extend(
                    self.graph[self.node_id]
                        .user_data
                        .output_ui(ui, self.node_id, self.graph, user_state, &param_name)
                        .into_iter(),
                );

                self.graph[self.node_id].user_data.separator(
                    ui,
                    self.node_id,
                    AnyParameterId::Output(param_id),
                    self.graph,
                    user_state,
                );

                let height_after = ui.min_rect().bottom();
                output_port_heights.push((height_before + height_after) / 2.0);
            }

            responses.extend(self.graph[self.node_id].user_data.bottom_ui(
                ui,
                self.node_id,
                self.graph,
                user_state,
            ));
        });

        // Add a small draggable interaction rect to the right of the node
        let drag_box_width = 6.0 * pan_zoom.zoom;
        let drag_box_rect = Rect::from_min_max(
            pos2(interaction_rect.right() - drag_box_width, interaction_rect.top() + title_height),
            pos2(interaction_rect.right(), interaction_rect.bottom() + title_height),
        );
        let drag_box_resp = ui.allocate_rect(drag_box_rect, Sense::click_and_drag())
            .on_hover_cursor(
                egui::CursorIcon::ResizeHorizontal,
            );

        if drag_box_resp.dragged() {
            let drag_delta = drag_box_resp.drag_delta();
            if drag_delta.length_sq() > 0.0 {
                // Convert screen space drag delta to zoom-adjusted space
                let zoom_adjusted_delta = drag_delta.x / pan_zoom.zoom;
                responses.push(NodeResponse::ResizeNode {
                    node: self.node_id,
                    resize_delta: Vec2::new(zoom_adjusted_delta, 0.0),
                });
            }
        }

        // Second pass, iterate again to draw the ports. This happens outside
        // the child_ui because we want ports to overflow the node background.

        let outer_rect = child_ui.min_rect().expand2(margin);
        let port_left = outer_rect.left();
        let port_right = outer_rect.right();

        // Save expanded rect to memory.
        ui.ctx().memory_mut(|mem| {
            mem.data
                .insert_temp(child_ui.id(), OuterRectMemory(outer_rect))
        });

        fn port_height(wide_port: bool, connections: usize, max_connections: usize) -> f32 {
            let port_full = connections == max_connections;
            if wide_port {
                let hooks = connections + if port_full { 0 } else { 1 };

                5.0 + (10.0 * hooks as f32).max(10.0)
            } else {
                10.0
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_port<NodeData, DataType, ValueType, UserResponse, UserState>(
            pan_zoom: &PanZoom,
            ui: &mut Ui,
            graph: &Graph<NodeData, DataType, ValueType>,
            node_id: NodeId,
            user_state: &mut UserState,
            port_pos: Pos2,
            responses: &mut Vec<NodeResponse<UserResponse, NodeData>>,
            param_id: AnyParameterId,
            port_locations: &mut PortLocations,
            conn_locations: &mut ConnLocations,
            ongoing_drag: Option<(NodeId, AnyParameterId)>,
            wide_port: bool,
            connections: usize,
            max_connections: usize,
        ) where
            DataType: DataTypeTrait<UserState>,
            UserResponse: UserResponseTrait,
            NodeData: NodeDataTrait,
        {
            let port_type = graph.any_param_type(param_id).unwrap();

            let port_rect = Rect::from_center_size(
                port_pos,
                egui::vec2(10.0, port_height(wide_port, connections, max_connections))
                    * pan_zoom.zoom,
            );

            let port_full = connections == max_connections;

            let inner_ports = if wide_port {
                connections + if port_full { 0 } else { 1 }
            } else {
                1
            };

            port_locations.insert(
                param_id,
                (0..inner_ports)
                    .map(|k| {
                        port_rect.center_top()
                            + Vec2::new(0.0, 5.0 * pan_zoom.zoom)
                            + Vec2::new(0.0, 10.0 * pan_zoom.zoom) * k as f32
                    })
                    .collect(),
            );

            let sense = if ongoing_drag.is_some() {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            };

            let resp = ui.allocate_rect(port_rect, sense);

            // Check if the distance between the port and the mouse is the distance to connect
            let close_enough = if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                port_rect.center().distance(pointer_pos) < DISTANCE_TO_CONNECT * pan_zoom.zoom
            } else {
                false
            };

            let port_color = if close_enough {
                Color32::WHITE
            } else {
                port_type.data_type_color(user_state)
            };

            if wide_port {
                ui.painter()
                    .rect_filled(port_rect, 5.0 * pan_zoom.zoom, port_color);
            } else {
                ui.painter().circle(
                    port_rect.center(),
                    5.0 * pan_zoom.zoom,
                    port_color,
                    Stroke::NONE,
                );
            }

            if connections > 0 {
                if let AnyParameterId::Input(input) = param_id {
                    for (k, dst_pos) in port_locations[&AnyParameterId::Input(input)]
                        .iter()
                        .enumerate()
                    {
                        conn_locations.entry(input).or_default().insert(k, *dst_pos);
                    }
                }
            }

            let nearest_hook = ui
                .input(|in_state| in_state.pointer.hover_pos())
                .and_then(|mouse_pos| match param_id {
                    AnyParameterId::Input(input) => Some((mouse_pos, input)),
                    AnyParameterId::Output(_) => None,
                })
                .and_then(|(mouse_pos, input)| {
                    let hooks = 0..inner_ports;
                    hooks.min_by(|&hook1, &hook2| {
                        let out1_dist = conn_locations[&input][hook1].distance(mouse_pos);
                        let out2_dist = conn_locations[&input][hook2].distance(mouse_pos);

                        out1_dist.partial_cmp(&out2_dist).unwrap()
                    })
                });

            if resp.drag_started() {
                match param_id {
                    AnyParameterId::Input(input) => {
                        match nearest_hook
                            .and_then(|hook| graph.connections(input).get(hook).copied())
                        {
                            Some(output) => {
                                responses.push(NodeResponse::DisconnectEvent { input, output });
                            }
                            None => {
                                responses
                                    .push(NodeResponse::ConnectEventStarted(node_id, param_id));
                            }
                        }
                    }
                    AnyParameterId::Output(_) => {
                        responses.push(NodeResponse::ConnectEventStarted(node_id, param_id));
                    }
                }
            }

            if let Some((origin_node, origin_param)) = ongoing_drag {
                if origin_node != node_id {
                    // Don't allow self-loops
                    // origin_param is the port type that is being dragged
                    // param_type is the port that is being hovered over
                    // Allow incompatible connections if one of the ports was defined as supporting any type by user.
                    let origin_type = graph.any_param_type(origin_param).unwrap();
                    if (port_type.allow_any_type() || origin_type.allow_any_type() || origin_type == port_type)
                        && close_enough {
                        match (param_id, origin_param) {
                            (AnyParameterId::Input(input), AnyParameterId::Output(output))
                            | (AnyParameterId::Output(output), AnyParameterId::Input(input)) => {
                                let input_hook =
                                    nearest_hook.unwrap_or(graph.connections(input).len());

                                if ui.input(|i| i.pointer.any_released()) {
                                    responses.push(NodeResponse::ConnectEventEnded {
                                        output,
                                        input,
                                        input_hook,
                                    });
                                } else if wide_port && !port_full {
                                    // move connections below the in-progress one to a lower position
                                    for k in input_hook..graph.connections(input).len() {
                                        conn_locations.get_mut(&input).unwrap()[k].y += 7.5;
                                    }
                                }
                            }
                            _ => { /* Ignore in-in or out-out connections */ }
                        }
                    }
                }
            }
        }

        // Input ports
        for ((_, param), port_height) in self.graph[self.node_id]
            .inputs
            .iter()
            .zip(input_port_heights.into_iter())
        {
            let should_draw = match self.graph[*param].kind() {
                InputParamKind::ConnectionOnly => true,
                InputParamKind::ConstantOnly => false,
                InputParamKind::ConnectionOrConstant => true,
            };

            if should_draw {
                let pos_left = pos2(port_left, port_height);
                let max_connections = self.graph[*param]
                    .max_connections
                    .map(NonZeroU32::get)
                    .unwrap_or(std::u32::MAX) as usize;
                draw_port(
                    pan_zoom,
                    ui,
                    self.graph,
                    self.node_id,
                    user_state,
                    pos_left,
                    &mut responses,
                    AnyParameterId::Input(*param),
                    self.port_locations,
                    self.conn_locations,
                    self.ongoing_drag,
                    max_connections > 1,
                    self.graph.connections(*param).len(),
                    max_connections,
                );
            }
        }

        // Output ports
        for ((_, param), port_height) in self.graph[self.node_id]
            .outputs
            .iter()
            .zip(output_port_heights.into_iter())
        {
            let pos_right = pos2(port_right, port_height);
            draw_port(
                pan_zoom,
                ui,
                self.graph,
                self.node_id,
                user_state,
                pos_right,
                &mut responses,
                AnyParameterId::Output(*param),
                self.port_locations,
                self.conn_locations,
                self.ongoing_drag,
                false,
                0,
                1,
            );
        }

        // Draw the background shape.
        // NOTE: This code is a bit more involved than it needs to be because egui
        // does not support drawing rectangles with asymmetrical round corners.

        let (shape, outline) = {
            let rounding_radius = 4.0 * pan_zoom.zoom;
            let rounding = Rounding::same(rounding_radius);

            let titlebar_height = title_height + margin.y;
            let titlebar_rect =
                Rect::from_min_size(outer_rect.min, vec2(outer_rect.width(), titlebar_height));
            let titlebar = Shape::Rect(RectShape {
                rect: titlebar_rect,
                rounding,
                fill: self.graph[self.node_id]
                    .user_data
                    .titlebar_color(ui, self.node_id, self.graph, user_state)
                    .unwrap_or_else(|| background_color.lighten(0.8)),
                stroke: Stroke::NONE,
                blur_width: 0.0,
                fill_texture_id: Default::default(),
                uv: Rect::ZERO,
            });

            let body_rect = Rect::from_min_size(
                outer_rect.min + vec2(0.0, titlebar_height - rounding_radius),
                vec2(outer_rect.width(), outer_rect.height() - titlebar_height),
            );
            let body = Shape::Rect(RectShape {
                rect: body_rect,
                rounding: Rounding::ZERO,
                fill: background_color,
                stroke: Stroke::NONE,
                blur_width: 0.0,
                fill_texture_id: Default::default(),
                uv: Rect::ZERO,
            });

            let bottom_body_rect = Rect::from_min_size(
                body_rect.min + vec2(0.0, body_rect.height() - titlebar_height * 0.5),
                vec2(outer_rect.width(), titlebar_height),
            );
            let bottom_body = Shape::Rect(RectShape {
                rect: bottom_body_rect,
                rounding,
                fill: background_color,
                stroke: Stroke::NONE,
                blur_width: 0.0,
                fill_texture_id: Default::default(),
                uv: Rect::ZERO,
            });

            let node_rect = titlebar_rect.union(body_rect).union(bottom_body_rect);
            let outline = if self.selected {
                Shape::Rect(RectShape {
                    rect: node_rect.expand(1.0 * pan_zoom.zoom),
                    rounding,
                    fill: Color32::WHITE.lighten(0.8),
                    stroke: Stroke::NONE,
                    blur_width: 0.0,
                    fill_texture_id: Default::default(),
                    uv: Rect::ZERO,
                })
            } else {
                Shape::Noop
            };

            // Take note of the node rect, so the editor can use it later to compute intersections.
            self.node_rects.insert(self.node_id, node_rect);

            (Shape::Vec(vec![titlebar, body, bottom_body]), outline)
        };

        ui.painter().set(background_shape, shape);
        ui.painter().set(outline_shape, outline);

        // --- Interaction ---

        // Titlebar buttons
        let can_delete = self.graph.nodes[self.node_id].user_data.can_delete(
            self.node_id,
            self.graph,
            user_state,
        );

        if can_delete && Self::close_button(pan_zoom, ui, outer_rect).clicked() {
            responses.push(NodeResponse::DeleteNodeUi(self.node_id));
        };

        // Movement
        let drag_delta = window_response.drag_delta();
        if drag_delta.length_sq() > 0.0 {
            responses.push(NodeResponse::MoveNode {
                node: self.node_id,
                drag_delta,
            });
            responses.push(NodeResponse::RaiseNode(self.node_id));
        }

        // Node selection
        //
        // HACK: Only set the select response when no other response is active.
        // This prevents some issues.
        if responses.is_empty() && window_response.clicked_by(PointerButton::Primary) {
            responses.push(NodeResponse::SelectNode(self.node_id));
            responses.push(NodeResponse::RaiseNode(self.node_id));
        }

        responses
    }

    fn close_button(pan_zoom: &PanZoom, ui: &mut Ui, node_rect: Rect) -> Response {
        // Measurements
        let margin = 8.0 * pan_zoom.zoom;
        let size = 10.0 * pan_zoom.zoom;
        let stroke_width = 2.0;
        let offs = margin + size / 2.0;

        let position = pos2(node_rect.right() - offs, node_rect.top() + offs);
        let rect = Rect::from_center_size(position, vec2(size, size));
        let resp = ui.allocate_rect(rect, Sense::click());

        let dark_mode = ui.visuals().dark_mode;
        let color = if resp.clicked() {
            if dark_mode {
                color_from_hex("#ffffff").unwrap()
            } else {
                color_from_hex("#000000").unwrap()
            }
        } else if resp.hovered() {
            if dark_mode {
                color_from_hex("#dddddd").unwrap()
            } else {
                color_from_hex("#222222").unwrap()
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if dark_mode {
                color_from_hex("#aaaaaa").unwrap()
            } else {
                color_from_hex("#555555").unwrap()
            }
        };
        let stroke = Stroke {
            width: stroke_width,
            color,
        };

        ui.painter()
            .line_segment([rect.left_top(), rect.right_bottom()], stroke);
        ui.painter()
            .line_segment([rect.right_top(), rect.left_bottom()], stroke);

        resp
    }
}
