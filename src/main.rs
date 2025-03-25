#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use pulseedit::PulseGraphEditor;

// struct IconData {
//     rgba: Vec<u8>,
//     width: u32,
//     height: u32,
// }

// fn load_icon(path: &str) -> IconData {
//     let (icon_rgba, icon_width, icon_height) = {
//         let image = image::open(path)
//             .expect("Failed to open icon path")
//             .into_rgba8();
//         let (width, height) = image.dimensions();
//         let rgba = image.into_raw();
//         (rgba, width, height)
//     };

//     IconData {
//         rgba: icon_rgba,
//         width: icon_width,
//         height: icon_height,
//     }
// }
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::egui::{ViewportBuilder, Visuals};
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Pulse Graph Editor",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            #[cfg(feature = "persistence")]
            {
                Ok(Box::new(PulseGraphEditor::new(cc)))
            }
            #[cfg(not(feature = "persistence"))]
            Ok(Box::<PulseGraphEditor>::default())
        }),
    )
    .expect("Failed to run native example");
}
