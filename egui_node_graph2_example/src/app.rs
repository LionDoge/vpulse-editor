use std::borrow::{Borrow, BorrowMut};
use std::{borrow::Cow, cell, collections::HashMap, default};
use std::fs::{self, File};
use std::io::prelude::*;
use eframe::egui::{self, DragValue, TextBuffer, TextStyle};
use egui_node_graph2::*;
use slotmap::SecondaryMap;
use crate::serialization::*;
use crate::instruction_templates;

// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyNodeData {
    template: MyNodeTemplate,
    custom_named_outputs: HashMap<OutputId, String>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyDataType {
    Scalar,
    Vec2,
    String,
    Action,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    String { value: String },
    Action,
}

impl Default for MyValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Scalar { value: 0.0 }
    }
}

impl MyValueType {
    /// Tries to downcast this value type to a vector
    pub fn try_to_vec2(self) -> anyhow::Result<egui::Vec2> {
        if let MyValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self)
        }
    }

    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let MyValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }

    pub fn try_to_string(self) -> anyhow::Result<String> {
        if let MyValueType::String { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to string", self)
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyNodeTemplate {
    MakeScalar,
    AddScalar,
    SubtractScalar,
    MakeVector,
    AddVector,
    SubtractVector,
    VectorTimesScalar,
    CellPublicMethod,
    EntFire,
    Compare,
    ConcatString,
    CellWait,
    GetVar,
    SetVar,
    EventHandler,
    IntToString,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MyResponse {
    AddOutputParam(NodeId, String),
    RemoveOutputParam(NodeId, String),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyGraphState {
    pub custom_input_string: String,
    pub added_parameters: SecondaryMap<NodeId, Vec<String>>,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<MyGraphState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
        match self {
            MyDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            MyDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            MyDataType::String => egui::Color32::from_rgb(52, 171, 235),
            MyDataType::Action => egui::Color32::from_rgb(252, 3, 165),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            MyDataType::Scalar => Cow::Borrowed("scalar"),
            MyDataType::Vec2 => Cow::Borrowed("2d vector"),
            MyDataType::String => Cow::Borrowed("string"),
            MyDataType::Action => Cow::Borrowed("action"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            MyNodeTemplate::MakeScalar => "New scalar",
            MyNodeTemplate::AddScalar => "Scalar add",
            MyNodeTemplate::SubtractScalar => "Scalar subtract",
            MyNodeTemplate::MakeVector => "New vector",
            MyNodeTemplate::AddVector => "Vector add",
            MyNodeTemplate::SubtractVector => "Vector subtract",
            MyNodeTemplate::VectorTimesScalar => "Vector times scalar",
            MyNodeTemplate::CellPublicMethod => "Public Method",
            MyNodeTemplate::EntFire => "EntFire",
            MyNodeTemplate::Compare => "Compare",
            MyNodeTemplate::ConcatString => "Concatenate strings",
            MyNodeTemplate::CellWait => "Wait",
            MyNodeTemplate::GetVar => "Load variable",
            MyNodeTemplate::SetVar => "Save variable",
            MyNodeTemplate::EventHandler => "Event Handler",
            MyNodeTemplate::IntToString => "Int to string",
        })
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            MyNodeTemplate::MakeScalar
            | MyNodeTemplate::AddScalar
            | MyNodeTemplate::SubtractScalar => vec!["Scalar"],
            MyNodeTemplate::MakeVector
            | MyNodeTemplate::AddVector
            | MyNodeTemplate::SubtractVector => vec!["Vector"],
            MyNodeTemplate::VectorTimesScalar => vec!["Vector", "Scalar"],
            MyNodeTemplate::CellPublicMethod | MyNodeTemplate::EventHandler => vec!["Inflow"],
            MyNodeTemplate::EntFire => vec!["Entities"],
            MyNodeTemplate::Compare => vec!["Logic"],
            MyNodeTemplate::ConcatString => vec!["String"],
            MyNodeTemplate::CellWait => vec!["Utility"],
            MyNodeTemplate::GetVar | MyNodeTemplate::SetVar => vec!["Variables"],
            MyNodeTemplate::IntToString => vec!["Conversion"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData { template: *self, custom_named_outputs: HashMap::new() }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        // We define some closures here to avoid boilerplate. Note that this is
        // entirely optional.
        let input_string = |graph: &mut MyGraph, name: &str, kind: InputParamKind| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::String,
                MyValueType::String {value: String::default()},
                kind,
                true,
            );
        };
        let input_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Scalar,
                MyValueType::Scalar { value: 0.0 },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                MyDataType::Vec2,
                MyValueType::Vec2 {
                    value: egui::vec2(0.0, 0.0),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_action = |graph: &mut MyGraph| {
            graph.add_input_param(
                node_id,
                "ActionIn".to_string(),
                MyDataType::Action,
                MyValueType::Action,
                InputParamKind::ConnectionOnly,
                true,
            );
        };

        let output_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Scalar);
        };
        let output_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Vec2);
        };
        let output_string = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::String);
        };
        let output_action = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), MyDataType::Action);
        };

        // input_action(graph);
        // output_action(graph);
        match self {
            MyNodeTemplate::AddScalar => {
                // The first input param doesn't use the closure so we can comment
                // it in more detail.
                graph.add_input_param(
                    node_id,
                    // This is the name of the parameter. Can be later used to
                    // retrieve the value. Parameter names should be unique.
                    "A".into(),
                    // The data type for this input. In this case, a scalar
                    MyDataType::Scalar,
                    // The value type for this input. We store zero as default
                    MyValueType::Scalar { value: 0.0 },
                    // The input parameter kind. This allows defining whether a
                    // parameter accepts input connections and/or an inline
                    // widget to set its value.
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::SubtractScalar => {
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::VectorTimesScalar => {
                input_scalar(graph, "scalar");
                input_vector(graph, "vector");
                output_vector(graph, "out");
            }
            MyNodeTemplate::AddVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            MyNodeTemplate::SubtractVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            MyNodeTemplate::MakeVector => {
                input_scalar(graph, "x");
                input_scalar(graph, "y");
                output_vector(graph, "out");
            }
            MyNodeTemplate::MakeScalar => {
                input_scalar(graph, "value");
                output_scalar(graph, "out");
            }
            MyNodeTemplate::CellPublicMethod => {
                graph.add_input_param(
                    node_id,
                    "name".into(),
                    MyDataType::String,
                    MyValueType::String {
                        value: "method".to_string(),
                    },
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                output_string(graph, "argument1");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::EntFire => {
                input_action(graph);
                input_string(graph, "entity", InputParamKind::ConstantOnly);
                input_string(graph, "input", InputParamKind::ConstantOnly);
                input_string(graph, "value", InputParamKind::ConnectionOrConstant);
                output_action(graph, "outAction");
            }
            MyNodeTemplate::Compare => {
                input_action(graph);
                input_string(graph, "operation", InputParamKind::ConstantOnly);
                input_string(graph, "Data type (optional)", InputParamKind::ConstantOnly);
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_action(graph, "True");
                output_action(graph, "False");
            }
            MyNodeTemplate::ConcatString => {
                input_string(graph, "A", InputParamKind::ConnectionOrConstant);
                input_string(graph, "B", InputParamKind::ConnectionOrConstant);
                output_string(graph, "out");
            }
            MyNodeTemplate::CellWait => {
                input_action(graph);
                input_scalar(graph, "time");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::GetVar => {
                input_string(graph, "name", InputParamKind::ConstantOnly);
                output_scalar(graph, "out");
            }
            MyNodeTemplate::SetVar => {
                input_action(graph);
                input_string(graph, "name", InputParamKind::ConstantOnly);
                input_scalar(graph, "value");
                output_action(graph, "outAction");
            }
            MyNodeTemplate::EventHandler => {
                input_action(graph);
                input_string(graph, "eventName", InputParamKind::ConstantOnly);
                output_action(graph, "outAction");
            }
            MyNodeTemplate::IntToString => {
                input_scalar(graph, "value");
                output_string(graph, "out");
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MyNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        vec![
            MyNodeTemplate::MakeScalar,
            MyNodeTemplate::MakeVector,
            MyNodeTemplate::AddScalar,
            MyNodeTemplate::SubtractScalar,
            MyNodeTemplate::AddVector,
            MyNodeTemplate::SubtractVector,
            MyNodeTemplate::VectorTimesScalar,
            MyNodeTemplate::CellPublicMethod,
            MyNodeTemplate::EntFire,
            MyNodeTemplate::Compare,
            MyNodeTemplate::ConcatString,
            MyNodeTemplate::CellWait,
            MyNodeTemplate::GetVar,
            MyNodeTemplate::SetVar,
            MyNodeTemplate::EventHandler,
            MyNodeTemplate::IntToString,
        ]
    }
}

impl WidgetValueTrait for MyValueType {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type NodeData = MyNodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut MyGraphState,
        _node_data: &MyNodeData,
    ) -> Vec<MyResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        let mut responses = vec![];
        match self {
            MyValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value.y));
                });
            }
            MyValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    // if this is a custom added parameter...
                    let vec_params = _user_state.added_parameters.get(_node_id);
                    if let Some(params) = vec_params {
                        if params.iter().find(|&x| x == param_name).is_some() {
                            if ui.button("X").on_hover_text("Remove parameter").clicked() {
                                responses.push(MyResponse::RemoveOutputParam(_node_id, param_name.to_string()));
                            }
                        }
                    }
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            MyValueType::String { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value);
                });
            }
            MyValueType::Action => {
                ui.label("ACT");
            }
        }
        // This allows you to return your responses from the inline widgets.
        responses
    }
}

impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<MyNodeData, MyDataType, MyValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<MyResponse, MyNodeData>>
    where
        MyResponse: UserResponseTrait,
    {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.

        let mut responses = vec![];
        // add param to event handler node.
        if _graph.nodes.get(node_id).unwrap().user_data.template == MyNodeTemplate::EventHandler {
            let textbox_str: &mut String = user_state.custom_input_string.borrow_mut();
            ui.text_edit_singleline(textbox_str);
            if ui.button("Add parameter").clicked() {
                responses.push(NodeResponse::User(MyResponse::AddOutputParam(node_id, user_state.custom_input_string.clone())));
                if let Some(vec_params) = user_state.added_parameters.get_mut(node_id) {
                    vec_params.push(user_state.custom_input_string.clone());
                } else {
                    user_state.added_parameters.insert(node_id, vec![user_state.custom_input_string.clone()]);
                }
            }
        }
        responses
    }
}

type MyGraph = Graph<MyNodeData, MyDataType, MyValueType>;
type MyEditorState =
    GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

#[derive(Default)]
pub struct NodeGraphExample {
    // The `GraphEditorState` is the top-level object. You "register" all your
    // custom types by specifying it as its generic parameters.
    state: MyEditorState,

    user_state: MyGraphState,
}

#[cfg(feature = "persistence")]
const PERSISTENCE_KEY: &str = "egui_node_graph";

#[cfg(feature = "persistence")]
impl NodeGraphExample {
    /// If the persistence feature is enabled, Called once before the first frame.
    /// Load previous app state (if any).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
            .unwrap_or_default();
        Self {
            state,
            user_state: MyGraphState::default(),
        }
    }
}

impl eframe::App for NodeGraphExample {
    #[cfg(feature = "persistence")]
    /// If the persistence function is enabled,
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, PERSISTENCE_KEY, &self.state);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("Compile").clicked() {
                    compile_graph(&self.state.graph);
                }
            });
        });
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                let graph_response = self.state.draw_graph_editor(
                    ui,
                    AllMyNodeTemplates,
                    &mut self.user_state,
                    Vec::default(),
                );
                graph_response
            })
            .inner;
        for node_response in graph_response.node_responses {
            // Here, we ignore all other graph events. But you may find
            // some use for them. For example, by playing a sound when a new
            // connection is created
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    MyResponse::AddOutputParam(node_id, name) => {
                        let output_id = self.state.graph.add_output_param(
                            node_id,
                            name.clone(),
                            MyDataType::Scalar,
                        );
                        let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                        node.user_data.custom_named_outputs.insert(output_id, name);
                    }
                    MyResponse::RemoveOutputParam(node_id, name ) => {
                        let param = self.state.graph.nodes.get_mut(node_id).unwrap().get_output(&name).unwrap();
                        self.state.graph.remove_output_param(param);
                        let node = self.state.graph.nodes.get_mut(node_id).unwrap();
                        let keys_to_remove: Vec<_> = node.user_data.custom_named_outputs.iter()
                            .filter_map(|(k, v)| if v == &name { Some(*k) } else { None })
                            .collect();
                        for k in keys_to_remove {
                            node.user_data.custom_named_outputs.remove(&k);
                        }
                    }
                }
            }
        }

        // if let Some(node) = self.user_state.active_node {
        //     if self.state.graph.nodes.contains_key(node) {
        //         let text = match evaluate_node(&self.state.graph, node, &mut HashMap::new()) {
        //             Ok(value) => format!("The result is: {:?}", value),
        //             Err(err) => format!("Execution error: {}", err),
        //         };
        //         ctx.debug_painter().text(
        //             egui::pos2(10.0, 35.0),
        //             egui::Align2::LEFT_TOP,
        //             text,
        //             TextStyle::Button.resolve(&ctx.style()),
        //             egui::Color32::WHITE,
        //         );
        //     } else {
        //         self.user_state.active_node = None;
        //     }
        // }
    }
}

type OutputsCache = HashMap<OutputId, MyValueType>;

/// Recursively evaluates all dependencies of this node, then evaluates the node itself.
pub fn evaluate_node(
    graph: &MyGraph,
    node_id: NodeId,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<MyValueType> {
    // To solve a similar problem as creating node types above, we define an
    // Evaluator as a convenience. It may be overkill for this small example,
    // but something like this makes the code much more readable when the
    // number of nodes starts growing.

    struct Evaluator<'a> {
        graph: &'a MyGraph,
        outputs_cache: &'a mut OutputsCache,
        node_id: NodeId,
    }
    impl<'a> Evaluator<'a> {
        fn new(graph: &'a MyGraph, outputs_cache: &'a mut OutputsCache, node_id: NodeId) -> Self {
            Self {
                graph,
                outputs_cache,
                node_id,
            }
        }
        fn evaluate_input(&mut self, name: &str) -> anyhow::Result<MyValueType> {
            // Calling `evaluate_input` recursively evaluates other nodes in the
            // graph until the input value for a paramater has been computed.
            evaluate_input(self.graph, self.node_id, name, self.outputs_cache)
        }
        fn populate_output(
            &mut self,
            name: &str,
            value: MyValueType,
        ) -> anyhow::Result<MyValueType> {
            // After computing an output, we don't just return it, but we also
            // populate the outputs cache with it. This ensures the evaluation
            // only ever computes an output once.
            //
            // The return value of the function is the "final" output of the
            // node, the thing we want to get from the evaluation. The example
            // would be slightly more contrived when we had multiple output
            // values, as we would need to choose which of the outputs is the
            // one we want to return. Other outputs could be used as
            // intermediate values.
            //
            // Note that this is just one possible semantic interpretation of
            // the graphs, you can come up with your own evaluation semantics!
            populate_output(self.graph, self.outputs_cache, self.node_id, name, value)
        }
        fn input_vector(&mut self, name: &str) -> anyhow::Result<egui::Vec2> {
            self.evaluate_input(name)?.try_to_vec2()
        }
        fn input_scalar(&mut self, name: &str) -> anyhow::Result<f32> {
            self.evaluate_input(name)?.try_to_scalar()
        }
        fn input_string(&mut self, name: &str) -> anyhow::Result<String> {
            self.evaluate_input(name)?.try_to_string()
        }
        fn output_vector(&mut self, name: &str, value: egui::Vec2) -> anyhow::Result<MyValueType> {
            self.populate_output(name, MyValueType::Vec2 { value })
        }
        fn output_scalar(&mut self, name: &str, value: f32) -> anyhow::Result<MyValueType> {
            self.populate_output(name, MyValueType::Scalar { value })
        }
        fn output_string(&mut self, name: &str, value: String) -> anyhow::Result<MyValueType> {
            self.populate_output(name, MyValueType::String { value })
        }
        fn output_action(&mut self) -> anyhow::Result<MyValueType> {
            self.populate_output("", MyValueType::Action)
        }
    }

    let node = &graph[node_id];
    let mut evaluator = Evaluator::new(graph, outputs_cache, node_id);
    match node.user_data.template {
        MyNodeTemplate::AddScalar => {
            let a = evaluator.input_scalar("A")?;
            let b = evaluator.input_scalar("B")?;
            evaluator.output_scalar("out", a + b)
        }
        MyNodeTemplate::SubtractScalar => {
            let a = evaluator.input_scalar("A")?;
            let b = evaluator.input_scalar("B")?;
            evaluator.output_scalar("out", a - b)
        }
        MyNodeTemplate::VectorTimesScalar => {
            let scalar = evaluator.input_scalar("scalar")?;
            let vector = evaluator.input_vector("vector")?;
            evaluator.output_vector("out", vector * scalar)
        }
        MyNodeTemplate::AddVector => {
            let v1 = evaluator.input_vector("v1")?;
            let v2 = evaluator.input_vector("v2")?;
            evaluator.output_vector("out", v1 + v2)
        }
        MyNodeTemplate::SubtractVector => {
            let v1 = evaluator.input_vector("v1")?;
            let v2 = evaluator.input_vector("v2")?;
            evaluator.output_vector("out", v1 - v2)
        }
        MyNodeTemplate::MakeVector => {
            let x = evaluator.input_scalar("x")?;
            let y = evaluator.input_scalar("y")?;
            evaluator.output_vector("out", egui::vec2(x, y))
        }
        MyNodeTemplate::MakeScalar => {
            let value = evaluator.input_scalar("value")?;
            evaluator.output_scalar("out", value)
        }
        MyNodeTemplate::CellPublicMethod => {
            let name = evaluator.input_string("name")?;
            match name.as_str() {
                "method" => evaluator.output_string("out", name),
                _ => anyhow::bail!("Unknown method: {}", name),
            }
        }
        MyNodeTemplate::EntFire => {
            let entity = evaluator.input_string("entity")?;
            let input = evaluator.input_string("input")?;
            let value = evaluator.input_string("value")?;
            let out = format!("EntFire {} {} {}", entity, input, value);
            evaluator.output_string("out", out)
        }
        MyNodeTemplate::Compare => {
            let operation = evaluator.input_string("operation")?;
            let a = evaluator.input_scalar("A")?;
            let b = evaluator.input_scalar("B")?;
            let out = match operation.as_str() {
                "==" => a == b,
                "!=" => a != b,
                "<" => a < b,
                "<=" => a <= b,
                ">" => a > b,
                ">=" => a >= b,
                _ => anyhow::bail!("Unknown operation: {}", operation),
            };
            evaluator.output_scalar("out", out as i32 as f32)
        }
        MyNodeTemplate::ConcatString => {
            let a = evaluator.input_string("A")?;
            let b = evaluator.input_string("B")?;
            evaluator.output_string("out", format!("{}{}", a, b))
        }
        MyNodeTemplate::CellWait => {
            let time = evaluator.input_scalar("time")?;
            let out = format!("Wait {}", time);
            evaluator.output_string("out", out)
        }
        MyNodeTemplate::GetVar => {
            let name = evaluator.input_string("name")?;
            let out = 0.0;
            evaluator.output_scalar("out", out)
        }
        MyNodeTemplate::SetVar => {
            let name = evaluator.input_string("name")?;
            let value = evaluator.input_scalar("value")?;
            let out = format!("SetVar {} {}", name, value);
            evaluator.output_string("out", out)
        }
        MyNodeTemplate::EventHandler => {
            let event_name = evaluator.input_string("eventName")?;
            let out = format!("Event handler for {}", event_name);
            evaluator.output_string("out", out)
        }
        MyNodeTemplate::IntToString => {
            let value = evaluator.input_scalar("value")?;
            let out = format!("{}", value as i32);
            evaluator.output_string("out", out)
        }
    }
}

fn populate_output(
    graph: &MyGraph,
    outputs_cache: &mut OutputsCache,
    node_id: NodeId,
    param_name: &str,
    value: MyValueType,
) -> anyhow::Result<MyValueType> {
    let output_id = graph[node_id].get_output(param_name)?;
    outputs_cache.insert(output_id, value.clone());
    Ok(value)
}

// Evaluates the input value of
fn evaluate_input(
    graph: &MyGraph,
    node_id: NodeId,
    param_name: &str,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<MyValueType> {
    let input_id = graph[node_id].get_input(param_name)?;

    // The output of another node is connected.
    if let Some(other_output_id) = graph.connection(input_id) {
        // The value was already computed due to the evaluation of some other
        // node. We simply return value from the cache.
        if let Some(other_value) = outputs_cache.get(&other_output_id) {
            Ok(other_value.clone())
        }
        // This is the first time encountering this node, so we need to
        // recursively evaluate it.
        else {
            // Calling this will populate the cache
            evaluate_node(graph, graph[other_output_id].node, outputs_cache)?;

            let other_value = outputs_cache.get(&other_output_id).expect("cache should be populated");
            Ok(other_value.clone())
            // Now that we know the value is cached, return it
            // Ok(*outputs_cache
            //     .get(&other_output_id)
            //     .expect("Cache should be populated"))
        }
    }
    // No existing connection, take the inline value instead.
    else {
        Ok(graph[input_id].value.clone())
    }
}

pub fn get_connected_output_node(graph: &MyGraph, out_action_id: &OutputId) -> Option<NodeId> {
    // dumb way of finding outgoing connection node.
    for group in graph.iter_connection_groups() {
        for connection in group.1 {
            if connection == *out_action_id {
                let input_action: &InputParam<MyDataType, MyValueType> = graph.inputs.get(group.0).expect("Can't find input value");
                return Some(input_action.node);
            }
        }
    }
    None
}

pub fn get_next_action_node<'a>(origin_node: &'a Node<MyNodeData>, graph: &'a MyGraph, name: &str) -> Option<&'a Node<MyNodeData>> {
    let out_action_id = origin_node.get_output(name);
    if out_action_id.is_ok() {
        let out_action_id = out_action_id.unwrap();
        let connected_node_id = get_connected_output_node(graph, &out_action_id);
        if connected_node_id.is_some() {
            return graph.nodes.get(connected_node_id.unwrap());
        }
    }
    return None;
}

pub fn traverse_event_cell(graph: &MyGraph, node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef) {
    let input_id = node.get_input("eventName").expect("Can't find input 'eventName'");
    let input_param = graph.inputs.get(input_id).expect("Can't find input value");
    let event_name = input_param.value.clone().try_to_string().unwrap();
    // create new pulse cell node.
    let chunk_id = graph_def.create_chunk();
    let mut cell_event = CPulseCell_Inflow_EventHandler::new(chunk_id, event_name);
    
    for (output_id, name) in node.user_data.custom_named_outputs.iter() {
        let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
        let reg_id = chunk.add_register(String::from("PVAL_INT"), 0);
        cell_event.add_outparam(name.clone(), reg_id);
        graph_def.add_register_mapping(*output_id, reg_id);
    }
    graph_def.cells.push(Box::from(CellType::InflowEvent(cell_event)));
    let connected_node = get_next_action_node(node, graph, "outAction");
    if connected_node.is_some() {
        traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
    }
}

pub fn traverse_entry_cell(graph: &MyGraph, node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef)
{
    let input_id = node.get_input("name").expect("Can't find input 'name'");
    let input_param = graph.inputs.get(input_id).expect("Can't find input value");
    // create new pulse cell node.
    let mut cell_method = CPulseCell_Inflow_Method::default();
    let chunk_id = graph_def.create_chunk();
    cell_method.name = input_param.value.clone().try_to_string().unwrap();
    cell_method.entry_chunk = chunk_id;
    cell_method.return_type = String::from("PVAL_INVALID");
    // get action connection
    let out_action_id = node.get_output("outAction").expect("Can't find output 'outAction'");
    //let out_action_param = graph.outputs.get(out_action_id).expect("Can't find output value");
    let chunk = graph_def.chunks.get_mut(chunk_id as usize).unwrap();
    // create argument1 (TODO only if connection exists)
    let reg_id_arg1 = chunk.add_register(String::from("PVAL_STRING"), 0);
    let output_id_arg1 = node.get_output("argument1").expect("Can't find output 'argument1'");
    cell_method.add_arg(String::from("arg1"), String::default(), String::from("PVAL_STRING"), reg_id_arg1);
    graph_def.add_register_mapping(output_id_arg1, reg_id_arg1);

    let cell_enum = CellType::InflowMethod(cell_method);
    graph_def.cells.push(Box::from(cell_enum));
    //let mut connected_node_id = NodeId::default();
    // dumb way of finding outgoing connection node.
    // graph.iter_connection_groups().for_each(|group| {
    //     group.1.iter().for_each(|connection| {
    //         if *connection == out_action_id {
    //             let input_action: &InputParam<MyDataType, MyValueType> = graph.inputs.get(group.0).expect("Can't find input value");
    //             connected_node_id = input_action.node;
    //             return;
    //         }
    //     });
    // });
    let connected_node_id = get_connected_output_node(graph, &out_action_id);
    if connected_node_id.is_some() {
        let connected_node = graph.nodes.get(connected_node_id.unwrap());
        if connected_node.is_some() {
            traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, chunk_id, &None);
        }
    }
}

pub fn compile_graph(graph: &MyGraph) {
    let mut graph_def = PulseGraphDef::default();
    graph_def.map_name = String::from("maps/test.vmap");
    graph_def.xml_name = String::default();
    for node in graph.iter_nodes() {
        let data: &Node<MyNodeData> = graph.nodes.get(node).unwrap();
        // start at all possible entry points
        match data.user_data.template {
            MyNodeTemplate::EventHandler => traverse_event_cell(graph, &data, &mut graph_def),
            MyNodeTemplate::CellPublicMethod => traverse_entry_cell(graph, &data, &mut graph_def),
            _ => {}
        }
    }
    let mut data = String::from("<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:vpulse13:version{354e36cb-dbe4-41c0-8fe3-2279dd194022} -->\n");
    data.push_str(graph_def.serialize().as_str());
    fs::write("graph_out/graph.vpulse", data).expect("Cannot write to file!");
}

pub fn try_find_output_mapping(graph_def: &PulseGraphDef, output_id: &Option<OutputId>) -> i32 {
    match output_id {
        Some(output_id) => {
            match graph_def.get_mapped_reigster(*output_id) {
                Some(reg) => { // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => { return -1; }
            }
        }
        None => { return -1; }
    }
}

pub fn create_or_get_variable(graph_def: &mut PulseGraphDef, name: &str) -> i32 {
    match graph_def.get_variable_index(&name) {
        Some(var) => {
            return var as i32;
        }
        None => {
            let var = Variable::new(name.to_string(), String::from("PVAL_INT"), 0);
            return graph_def.add_variable(var);
        }
    }
}

pub fn try_find_input_mapping(graph_def: &PulseGraphDef, input_id: &Option<InputId>) -> i32 {
    match input_id {
        Some(input_id) => {
            match graph_def.get_mapped_reigster_input(*input_id) {
                Some(reg) => { // we found a mapping! So we know which register to use for this
                    return *reg;
                }
                None => { return -1; }
            }
        }
        None => { return -1; }
    }
}

pub fn traverse_nodes_and_populate(graph: &MyGraph, current_node: &Node<MyNodeData>, graph_def: &mut PulseGraphDef, target_chunk: i32, output_id: &Option<OutputId>) -> i32 {
    match current_node.user_data.template {
        MyNodeTemplate::CellPublicMethod => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        MyNodeTemplate::EventHandler => {
            // here we resolve connections to the argument outputs
            return try_find_output_mapping(graph_def, output_id);
        }
        MyNodeTemplate::CellWait => {
            let time_input_id = current_node.get_input("time").expect("Can't find input 'time'");
            let connection_to_time = graph.connection(time_input_id);
            let time_input_register: i32;
            match connection_to_time {
                Some(out) => {
                    // get existing register id to note for the wait time
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    time_input_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                }
                None => {
                    // create a constant value for the wait time
                    // have to do this way because of the borrow checker's mutability rules.
                    let new_constant_id = graph_def.get_current_constant_id() + 1;

                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    // add a new register to store the wait time
                    time_input_register = chunk.add_register(String::from("PVAL_FLOAT"), chunk.get_last_instruction_id() + 1);
                    // get the wait time constant
                    let instruction = instruction_templates::get_const(new_constant_id, time_input_register);
                    chunk.add_instruction(instruction);
                    // add the actual constant definition for the waiting time
                    let value_param = graph.get_input(time_input_id);
                    let time = value_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                    let constant = PulseConstant::Float(time);
                    graph_def.add_constant(constant);
                }
            }
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            // ! Important (might change). We assume that after waiting we go to the next instruction after the cell invoke.
            let cell_wait = CPulseCell_Inflow_Wait::new(target_chunk, chunk.get_last_instruction_id() + 3);
            graph_def.cells.push(Box::from(CellType::InflowWait(cell_wait)));

            let chunk_opt = graph_def.chunks.get(target_chunk as usize);
            if chunk_opt.is_some() {
                let chunk = chunk_opt.unwrap();
                let mut register_map = RegisterMap::default();
                register_map.add_inparam(String::from("flDurationSec"), time_input_register);
                let binding = InvokeBinding {
                    register_map,
                    func_name: String::from("Wait"),
                    cell_index: graph_def.cells.len() as i32 - 1,
                    src_chunk: target_chunk,
                    src_instruction: chunk.get_last_instruction_id() + 1,
                };
                let binding_idx = graph_def.add_binding(binding);
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                chunk.add_instruction(instruction_templates::cell_invoke(binding_idx));
                // early return.
                let mut instr_ret_void = Instruction::default();
                instr_ret_void.code = String::from("RETURN_VOID");
                chunk.add_instruction(instr_ret_void);
            }
            let connected_node = get_next_action_node(current_node, graph, "outAction");
            if connected_node.is_some() {
                return traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, target_chunk, &None);
            }
        }
        MyNodeTemplate::EntFire => {
            // create EntFire (step) cell
            let entity_id = current_node.get_input("entity").expect("Can't find input 'entity'");
            let value_entity = graph.inputs.get(entity_id).expect("Can't find input value").value.clone().try_to_string();
            if let Ok(value) = value_entity {
                // create domain value (only if we know value already)
                let domain_val_idx = graph_def.create_domain_value(String::from("ENTITY_NAME"), value.clone(), String::new());
                
                let input_id = current_node.get_input("input").expect("Can't find input 'input'");
                let input_param = graph.inputs.get(input_id).expect("Can't find input value").value.clone().try_to_string().expect("Failed to unwrap input value");
                let step_cell = CPulseCell_Step_EntFire::new(input_param.clone());
                let cell_enum = CellType::StepEntFire(step_cell);
                
                let value_id = current_node.get_input("value").expect("Can't find input 'value'");
                let mut value_input_register: i32 = -1;
                // try to resolve the value input node
                let connection_to_value = graph.connection(value_id);
                match connection_to_value {
                    Some(out) => {
                        let out_param = graph.get_output(out);
                        let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                        value_input_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                    }
                    None => { // no connection, create constant value
                        let value_param = graph.get_input(value_id);
                        let str= value_param.value().clone().try_to_string();
                        match str {
                            Ok(str) => {
                                if !str.is_empty() {
                                    let constant = PulseConstant::String(str);
                                    let const_idx = graph_def.add_constant(constant);
                                    // create register to hold this value
                                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                                    value_input_register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                                    // create instruction to load this value now.
                                    let instruction = instruction_templates::get_const(const_idx, value_input_register);
                                    chunk.add_instruction(instruction);
                                }
                            }
                            Err(err) => {
                                println!("Error getting string from node value: {}", err);
                            }
                        }
                    }
                }
                
                //let value_param = graph.get_input(value_id)
                graph_def.cells.push(Box::from(cell_enum));
                // now build instructions and bindings to get the domain value, and invoke the cell
                let chunk_opt = graph_def.chunks.get_mut(target_chunk as usize);
                if chunk_opt.is_some() {
                    let chunk = chunk_opt.unwrap();
                    // new register to load in the domain value
                    let reg_id = chunk.add_register(String::from("PVAL_ENTITY_NAME"), chunk.get_last_instruction_id() + 1);
                    // load the domain value instruction
                    chunk.add_instruction(instruction_templates::get_domain_value(reg_id, domain_val_idx));
                    // add invoke binding for FireAtName cell
                    let mut register_map = RegisterMap::default();
                    register_map.add_inparam(String::from("TargetName"), reg_id);
                    if value_input_register != -1 {
                        register_map.add_inparam(String::from("pParam"), value_input_register);
                    }
                    let binding = InvokeBinding {
                        register_map: register_map,
                        func_name: String::from("FireAtName"),
                        cell_index: graph_def.cells.len() as i32 - 1,
                        src_chunk: target_chunk,
                        src_instruction: chunk.get_last_instruction_id() + 1,
                    };
                    let binding_idx = graph_def.add_binding(binding);
                    // add instruction for invoking the binding.
                    // rust doesn't like reusing the borrowed chunks reference, but we know that it doesn't change.
                    graph_def.chunks.get_mut(target_chunk as usize).unwrap()
                    .add_instruction(instruction_templates::cell_invoke(binding_idx));
                    //graph.connection(input)
                    let output_connection = OutputConnection::new(
                        String::from("Step_EntFire:-1"), 
                        value, input_param, if value_input_register != -1 { String::from("param") } else { String::default() });
                    graph_def.add_output_connection(output_connection);
                }
                let connected_node = get_next_action_node(current_node, graph, "outAction");
                if connected_node.is_some() {
                    return traverse_nodes_and_populate(graph, connected_node.unwrap(), graph_def, target_chunk, &None);
                }
            }
            
        }
        MyNodeTemplate::ConcatString => {
            let id_a = current_node.get_input("A").expect("Can't find input A in node");
            let id_b = current_node.get_input("B").expect("Can't find input B in node");
            let input_ids = [id_a, id_b];
            let connection_to_a = graph.connection(id_a);
            let connection_to_b = graph.connection(id_b);
            let connections_to_resolve: [Option<OutputId>; 2] = [connection_to_a, connection_to_b]; 
            let mut input_registers: [i32; 2] = [-1, -1];

            for (i, connection) in connections_to_resolve.iter().enumerate() {
                match connection {
                    Some(out) => {
                        let out_param = graph.get_output(*out);
                        let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                        // grab the register that the value will come from.
                        input_registers[i] = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(*out));
                    }
                    None => {
                        // no connection.. First search if we already created it, if not create the constant input value
                        let register = try_find_input_mapping(graph_def, &Some(input_ids[i]));
                        if register == -1 {
                            let input_info: &InputParam<MyDataType, MyValueType> = graph.get_input(input_ids[i]);
                            let constant = PulseConstant::String(input_info.value.clone().try_to_string().unwrap());
                            let const_idx = graph_def.add_constant(constant);
                            // create register to hold this value
                            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                            input_registers[i] = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                            // create instruction to load this value now.
                            let instruction = instruction_templates::get_const(const_idx, input_registers[i]);
                            chunk.add_instruction(instruction);
                            graph_def.add_register_mapping_input(input_ids[i], input_registers[i]);
                        } else {
                            input_registers[i] = register;
                        }
                    }
                }
            }
            // registers are figured out. now prepare the output register and the instruction
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                let instruction = instruction_templates::add_string(input_registers[0], input_registers[1], register);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        MyNodeTemplate::GetVar => {
            let name_id = current_node.get_input("name").expect("Can't find input 'name'");
            // name is a constant value
            let name = graph.get_input(name_id).value().clone().try_to_string().expect("Can't find name parameter");
            let var_id = create_or_get_variable(graph_def, name.borrow());
            // add register
            // add instruction to load the variable value
            let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
            let reg = chunk.add_register(String::from("PVAL_INT"), chunk.get_last_instruction_id() + 1);
            chunk.add_instruction(instruction_templates::get_var(reg, var_id as i32));
            return reg;
        }
        MyNodeTemplate::IntToString => {
            let value_id = current_node.get_input("value").expect("Can't find input 'value'");
            let connection_to_value = graph.connection(value_id);
            let mut register_input: i32 = -1;
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    // grab the register that the value will come from.
                    register_input = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                }
                None => {
                    print!("No connection found for input value for IntToString node");
                    return -1;
                } 
            }
            let mut register = try_find_output_mapping(graph_def, output_id);
            if register == -1 {
                let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                register = chunk.add_register(String::from("PVAL_STRING"), chunk.get_last_instruction_id() + 1);
                let instruction = instruction_templates::convert_value(register, register_input);
                chunk.add_instruction(instruction);
                graph_def.add_register_mapping(output_id.unwrap(), register);
            }
            return register;
        }
        MyNodeTemplate::SetVar => {
            let name_id = current_node.get_input("name").expect("Can't find input 'name'");
            // name is a constant value
            let name = graph.get_input(name_id).value().clone().try_to_string().expect("Can't find name parameter");
            let var_id = create_or_get_variable(graph_def, name.borrow());
            let value_id = current_node.get_input("value").expect("Can't find input 'value'");
            let connection_to_value = graph.connection(value_id);
            match connection_to_value {
                Some(out) => {
                    let out_param = graph.get_output(out);
                    let out_node = graph.nodes.get(out_param.node).expect("Can't find output node");
                    let value_register = traverse_nodes_and_populate(graph, out_node, graph_def, target_chunk, &Some(out));
                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    chunk.add_instruction(instruction_templates::set_var(var_id as i32, value_register));
                }
                None => {
                    let mut register = try_find_input_mapping(graph_def, &Some(value_id));
                    let chunk = graph_def.chunks.get_mut(target_chunk as usize).unwrap();
                    if register == -1 {
                        let value_param = graph.get_input(value_id);
                        let value = value_param.value().clone().try_to_scalar().expect("Failed to unwrap input value");
                        register = chunk.add_register(String::from("PVAL_INT"), chunk.get_last_instruction_id() + 1);
                        let instruction = instruction_templates::get_const(value as i32, register);
                        chunk.add_instruction(instruction);
                    }
                    chunk.add_instruction(instruction_templates::set_var(var_id as i32, register));
                }
            }
        }
        _ => {}
    }
    return -1;
}