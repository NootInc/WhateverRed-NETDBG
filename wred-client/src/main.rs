#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(
    warnings,
    clippy::cargo,
    clippy::nursery,
    unused_extern_crates,
    rust_2021_compatibility
)]

#[macro_use]
extern crate cfg_if;

mod app;
mod requests;
mod style;
mod utils;

fn main() {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();

            wasm_bindgen_futures::spawn_local(async {
                eframe::start_web(
                    "main_canvas",
                    eframe::WebOptions::default(),
                    Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
                )
                .await
                .expect("failed to start eframe");
            });
        } else {
            eframe::run_native(
                "com.ChefKissInc.WRedNetDbgClient",
                eframe::NativeOptions {
                    #[cfg(target_os = "macos")]
                    fullsize_content: true,
                    ..Default::default()
                },
                Box::new(|cc| Box::new(app::WRedNetDbgApp::new(cc))),
            );
        }
    }
}
