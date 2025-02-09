mod app;
mod emulator;
mod handlers;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use std::panic;
extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    init_console();
    let starter = app::App::start_loop();
    starter();
    Ok(())
}

fn init_console() {
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize the console");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
