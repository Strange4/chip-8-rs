mod app;
mod emulator;
mod handlers;
mod ui;

use std::panic;
use wasm_bindgen::prelude::wasm_bindgen;
extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn start() {
    init_console();
    ui::set_handlers();
}

fn init_console() {
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize the console");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
