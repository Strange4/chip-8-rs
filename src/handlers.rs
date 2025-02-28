use log::info;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{js_sys::Uint8Array, Document, Event, HtmlButtonElement, HtmlInputElement};

use crate::{
    app::{self, window, Runner},
    debugger::{INTERVAL_HANDLE, RENDER_DEBUGGER},
    emulator,
    ui::{get_element, render_emulator},
};

pub fn set_handlers() {
    start_button_handler(&app::document());
    stop_button_handler(&app::document());
    step_button_handler(&app::document());
    load_rom_handler(&app::document());
    debugger_checkbox_handler(&app::document());
}

fn start_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#start_button");
    add_event_listener(&button, "click", |_| {
        let starter = Runner::start_loop();
        starter();
    });
}

fn stop_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#stop_button");
    add_event_listener(&button, "click", |_| stop_runner());
}

fn load_rom_handler(document: &Document) {
    let input_element: HtmlInputElement = get_element(document, "#load_rom");
    add_event_listener(&input_element, "change", |e| {
        let input = e
            .current_target()
            .expect("There was no target for this event")
            .dyn_into::<HtmlInputElement>()
            .expect("Could not dyn into input element");
        if let Some(files) = input.files() {
            let file = files.item(0).unwrap();
            info!("Loading rom: {}", file.name());

            let closure = Closure::new(|js_value: JsValue| {
                let u8_vec = Uint8Array::new(&js_value).to_vec();
                emulator::get_program().lock().unwrap().load_rom(&u8_vec);
                info!("Loaded rom!");
            });
            let _ = file.array_buffer().then(&closure);
            closure.forget();
        }
    });
}

fn step_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#step_button");
    add_event_listener(&button, "click", |_| {
        stop_runner();
        let mut emulator = emulator::get_program()
            .lock()
            .expect("Could not lock the program");
        emulator.tick();
        emulator.timer_tick();
        render_emulator(
            &emulator,
            &app::get_canvas_context(),
            &app::get_debugger_area(),
        );
        info!("stepped through {}", emulator.program_counter)
    });
}

fn stop_runner() {
    let handle = INTERVAL_HANDLE
        .lock()
        .expect("Could not get intveral handle");
    if handle.is_some() {
        window().clear_interval_with_handle(handle.unwrap());
    }
}

fn debugger_checkbox_handler(document: &Document) {
    let checkbox: HtmlInputElement = get_element(document, "#show_debugger");
    add_event_listener(&checkbox, "change", |e| {
        let checkbox: HtmlInputElement = e
            .current_target()
            .expect("Could not get target of event")
            .dyn_into()
            .expect("Could not dyn into a checkbox");
        let mut a = RENDER_DEBUGGER
            .lock()
            .expect("Could not acquire debugger variable");
        *a = checkbox.checked();
    });
}

pub fn add_event_listener(target: &web_sys::EventTarget, event_name: &str, func: fn(e: Event)) {
    let closure: Closure<dyn Fn(Event)> = Closure::new(func);
    target
        .add_event_listener_with_event_listener(event_name, closure.as_ref().unchecked_ref())
        .expect("Could not add event listener");
    closure.forget();
}
