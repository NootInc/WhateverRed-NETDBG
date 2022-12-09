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
        "WhateverRed NetDbg",
        eframe::NativeOptions {
            #[cfg(target_os = "macos")]
            fullsize_content: true,
            ..Default::default()
        },
        Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
    );
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), eframe::wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    eframe::start_web(
        "main_canvas",
        eframe::WebOptions::default(),
        Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
    )
    .await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
