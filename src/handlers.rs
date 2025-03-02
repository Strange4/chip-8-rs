use log::info;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    js_sys::Uint8Array, Document, Event, HtmlButtonElement, HtmlDivElement, HtmlInputElement, HtmlTableRowElement,
};

use crate::{
    app::{self, get_canvas_context, window, Runner},
    debugger::{render_debugger, BREAKPOINTS, INTERVAL_HANDLE, RENDER_DEBUGGER},
    emulator::{self, get_program},
    ui::{add_class_name, get_element, remove_class_name, render_emulator, to_number},
};

pub fn set_handlers() {
    let document = &app::document();
    start_button_handler(document);
    stop_button_handler(document);
    step_button_handler(document);
    load_rom_handler(document);
    debugger_checkbox_handler(document);
    toggle_breakpoint_handler(document);
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
                let mut program = emulator::get_program().lock().unwrap();
                program.load_rom(&u8_vec);
                render_emulator(&program, &get_canvas_context());
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
        render_emulator(&emulator, &app::get_canvas_context());
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
        let document = &crate::app::document();
        let checkbox: HtmlInputElement = e
            .current_target()
            .expect("Could not get target of event")
            .dyn_into()
            .expect("Could not dyn into a checkbox");
        let debugger_area: HtmlDivElement = get_element(document, "#debugger");
        let is_checked = checkbox.checked();
        if is_checked {
            remove_class_name(&debugger_area, "off");
            // debugger_area.set_class_name("");
        } else {
            add_class_name(&debugger_area, "off");
            // debugger_area.set_class_name("off");
        }

        let mut a = RENDER_DEBUGGER
            .lock()
            .expect("Could not acquire debugger variable");
        *a = is_checked;
        // drop the mutex before rendering since it needs it
        drop(a);
        let program = get_program().lock().unwrap();
        render_debugger(&program);
    });
}

fn toggle_breakpoint_handler(document: &Document) {
    let rows = document
        .query_selector_all("#memory-table tr")
        .expect("The query was wrong");
    for i in 0..rows.length() {
        let row = rows.item(i).unwrap();
        add_event_listener(&row, "click", |e| {
            let row: HtmlTableRowElement = e
                .current_target()
                .expect("Could not get row in event")
                .dyn_into()
                .expect("Could not dyn into a row");

            let address: usize = to_number(
                &row.child_nodes()
                    .item(1)
                    .expect("Could not get the child node for the address"),
            );

            let is_selected = row.class_name().contains("breakpoint");
            let mut breakpoints = BREAKPOINTS
                .lock()
                .expect("Could not acquire breakpoint lock");
            if is_selected {
                row.set_class_name("");
                let index = breakpoints
                    .iter()
                    .position(|breakpoint| *breakpoint == address)
                    .unwrap();
                breakpoints.remove(index);
            } else {
                row.set_class_name("breakpoint");
                breakpoints.push(address);
            }
        });
    }
}

pub fn add_event_listener(target: &web_sys::EventTarget, event_name: &str, func: fn(e: Event)) {
    let closure: Closure<dyn Fn(Event)> = Closure::new(func);
    target
        .add_event_listener_with_event_listener(event_name, closure.as_ref().unchecked_ref())
        .expect("Could not add event listener");
    closure.forget();
}
