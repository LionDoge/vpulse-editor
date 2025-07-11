#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![windows_subsystem = "windows"]

use pulseedit::PulseGraphEditor;
use std::sync::Arc;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let d = eframe::icon_data::from_png_bytes(include_bytes!("../icon.png"))
        .expect("The icon data must be valid");
    use eframe::egui::{ViewportBuilder, Visuals};
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..Default::default()
    };
    options.viewport.icon = Some(Arc::new(d));
    eframe::run_native(
        "Pulse Graph Editor",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Ok(Box::new(PulseGraphEditor::new(cc)))
            // #[cfg(not(feature = "persistence"))]
            // Ok(Box::<PulseGraphEditor>::default())
        }),
    )
    .expect("Failed to run app");
}
