use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, KeyboardEvent};

use crate::{emulator, runner};

pub fn set_handlers() {
    let document = runner::document();
    const KEYS: [&str; 16] = [
        "KeyX", "Digit1", "Digit2", "Digit3", "KeyQ", "KeyW", "KeyE", "KeyA", "KeyS", "KeyD",
        "KeyZ", "KeyC", "Digit4", "KeyR", "KeyF", "KeyV",
    ];

    let key_down_handler: Closure<dyn Fn(Event)> = Closure::new(|e: Event| {
        let key_code = e
            .dyn_into::<KeyboardEvent>()
            .expect("Could not dyn into keyboard event")
            .code();
        for (i, &key) in KEYS.iter().enumerate() {
            if key == key_code.as_str() {
                emulator::get_program()
                    .lock()
                    .unwrap()
                    .set_key_down(i as u8);
            }
        }
    });
    let key_up_handler: Closure<dyn Fn(Event)> = Closure::new(|e: Event| {
        let key_code = e
            .dyn_into::<KeyboardEvent>()
            .expect("Could not dyn into keyboard event")
            .code();
        for (i, &key) in KEYS.iter().enumerate() {
            if key == key_code.as_str() {
                emulator::get_program().lock().unwrap().set_key_up(i as u8);
            }
        }
    });

    document
        .add_event_listener_with_event_listener(
            "keydown",
            key_down_handler.as_ref().unchecked_ref(),
        )
        .expect("Could not set keydown event listener");
    document
        .add_event_listener_with_event_listener("keyup", key_up_handler.as_ref().unchecked_ref())
        .expect("Could not set keydown event listener");
    key_up_handler.forget();
    key_down_handler.forget();
}
