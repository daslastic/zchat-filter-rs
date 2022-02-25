#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = eframe_template::ZoomApp::default();
    let native_options = eframe::NativeOptions::default();
    //App::clear_color(1);
    eframe::run_native(Box::new(app), native_options);
}