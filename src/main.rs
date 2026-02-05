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

    setup_panic_hook();
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

fn setup_panic_hook() {
    use rfd::MessageDialog;
    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_formatted = format!("{:#?}", panic_info);
        let panic_payload_display = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            Some(s.to_string())
        } else {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_owned())
        };

        let res = MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Whoops!")
            .set_description(format!(
                "The editor has crashed due to an unhandled error.\n\n{}\n\n{}",
                panic_formatted,
                panic_payload_display
                    .as_deref()
                    .unwrap_or("<Panic payload unavailable>")
            ))
            .set_buttons(rfd::MessageButtons::OkCustom("Copy log".to_string()))
            .show();

        if let rfd::MessageDialogResult::Custom(_) = res {
            let mut clipboard = arboard::Clipboard::new().unwrap();
            clipboard
                .set_text(format!(
                    "Panic info:\n{}\n\nPanic payload:\n{}",
                    panic_formatted,
                    panic_payload_display
                        .as_deref()
                        .unwrap_or("<Panic payload unavailable>")
                ))
                .unwrap();
        }
    }));
}
