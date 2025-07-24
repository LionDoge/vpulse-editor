#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use libpulseedit::PulseGraphEditor;
use std::sync::Arc;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let d = eframe::icon_data::from_png_bytes(include_bytes!("../icon.png"))
        .expect("The icon data must be valid");
    use eframe::egui::ViewportBuilder;
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..Default::default()
    };
    options.viewport.icon = Some(Arc::new(d));
    eframe::run_native(
        "Pulse Graph Editor",
        options,
        Box::new(|cc| {
            Ok(Box::new(PulseGraphEditor::new(cc)))
            // #[cfg(not(feature = "persistence"))]
            // Ok(Box::<PulseGraphEditor>::default())
        }),
    )
    .expect("Failed to run app");
}
