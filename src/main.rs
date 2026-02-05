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

    use gui_panic_handler::AppInfo;
    gui_panic_handler::register(AppInfo {
        name: "Pulse Graph Editor",
        additional_text: "The editor has crashed, if this happens consistently please report along with relevant information.",
        links: vec![],
        report_bug_url: Some(gui_panic_handler::GitHubBugReporter::new(
            String::from("LionDoge"),
            String::from("vpulse-editor"),
        )),
    });

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
