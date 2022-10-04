#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(clippy::module_name_repetitions, clippy::too_many_lines)]

mod app;
mod style;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eframe::run_native(
        "WhateverRed NETDBG",
        eframe::NativeOptions {
            fullsize_content: true,
            ..Default::default()
        },
        Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    eframe::start_web(
        "main_canvas",
        eframe::WebOptions::default(),
        Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
    )
    .expect("Failed to start eframe");
}
