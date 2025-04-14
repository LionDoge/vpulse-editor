#![forbid(unsafe_code)]
//#![allow(nonstandard_style)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod instruction_templates;
mod pulsetypes;
mod serialization;
mod outputdefinition;
mod compiler;
mod bindings;
mod typing;
pub use app::PulseGraphEditor;

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = PulseGraphEditor::default();
    eframe::start_web(canvas_id, Box::new(app))
}
