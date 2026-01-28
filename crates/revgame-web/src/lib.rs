// Web entry point - placeholder for now
// This will be implemented with Ratzilla for WASM support

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    console_log::init_with_level(log::Level::Debug).ok();

    log::info!("RevGame web version starting...");

    // TODO: Initialize Ratzilla and start the TUI app
}

#[wasm_bindgen]
pub fn greet() -> String {
    "RevGame - Reverse Engineering Educational Game".to_string()
}
