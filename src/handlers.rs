use log::{info, warn};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    js_sys::{Promise, Uint8Array},
    Document, Event, HtmlButtonElement, HtmlDivElement, HtmlInputElement, HtmlSelectElement,
    HtmlTableRowElement, Request, RequestInit, Response,
};

use crate::{
    debugger::{render_debugger, BREAKPOINTS, INTERVAL_HANDLE, RENDER_DEBUGGER},
    emulator::{self, get_program},
    runner::{Runner, UPDATES_PER_SECOND},
    ui::{
        self, add_class_name, document, get_canvas_context, get_element, remove_class_name,
        render_emulator, to_number, window,
    },
};

pub fn set_handlers() {
    let document = &document();
    start_button_handler(document);
    stop_button_handler(document);
    step_button_handler(document);
    load_rom_handler(document);
    debugger_on_handler(document);
    toggle_breakpoint_handler(document);
    set_clock_speed_handler(document);
    select_rom_handler(document);
    reset_emulator_handler(document);
}

fn start_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#start-button");
    add_event_listener(&button, "click", |_| {
        let starter = Runner::start_loop();
        starter();
    });
}

fn stop_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#stop-button");
    add_event_listener(&button, "click", |_| stop_runner());
}

fn load_rom_handler(document: &Document) {
    let input_element: HtmlInputElement = get_element(document, "#load-rom");
    add_event_listener(&input_element, "change", |e| {
        let input = e
            .current_target()
            .expect("There was no target for this event")
            .dyn_into::<HtmlInputElement>()
            .expect("Could not dyn into input element");
        if let Some(files) = input.files() {
            let file = files.item(0).unwrap();
            info!("Loading rom: {}", file.name());

            let closure = load_rom_from_array_promise();
            let _ = file.array_buffer().then(&closure);
            closure.forget();
        }
    });
}

fn load_rom_from_array_promise() -> Closure<dyn FnMut(JsValue)> {
    Closure::new(|js_value: JsValue| {
        let u8_vec = Uint8Array::new(&js_value).to_vec();
        let mut program = emulator::get_program().lock().unwrap();
        program.load_rom(&u8_vec);
        render_emulator(&program, &get_canvas_context());
        info!("Loaded rom!");
        Runner::start_loop()();
    })
}

fn step_button_handler(document: &Document) {
    let button: HtmlButtonElement = get_element(document, "#step-button");
    add_event_listener(&button, "click", |_| {
        stop_runner();
        let mut emulator = emulator::get_program()
            .lock()
            .expect("Could not lock the program");
        emulator.tick();
        emulator.timer_tick();
        render_emulator(&emulator, &get_canvas_context());
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

fn debugger_on_handler(document: &Document) {
    let checkbox: HtmlButtonElement = get_element(document, "#show-debugger");
    add_event_listener(&checkbox, "click", |e| {
        let document = &crate::ui::document();
        let checkbox: HtmlButtonElement = e
            .current_target()
            .expect("Could not get target of event")
            .dyn_into()
            .expect("Could not dyn into a checkbox");
        let debugger_area: HtmlDivElement = get_element(document, "#debugger");
        let turn_on = !checkbox.class_name().contains("checked");
        if turn_on {
            add_class_name(&checkbox, "checked");
            remove_class_name(&debugger_area, "off");
        } else {
            add_class_name(&debugger_area, "off");
            remove_class_name(&checkbox, "checked");
        }

        let mut a = RENDER_DEBUGGER
            .lock()
            .expect("Could not acquire debugger variable");
        *a = turn_on;
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

fn set_clock_speed_handler(document: &Document) {
    let slider: HtmlInputElement = get_element(document, "#speed-knob");
    let number_input: HtmlInputElement = get_element(document, "#speed-display");
    let update_func = |event: Event| {
        // having so many unwraps and dyn intos is the reason that I think running js like code from rust is bad
        if let Ok(value) = event
            .current_target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap()
            .value()
            .parse::<f64>()
        {
            let display: HtmlInputElement = get_element(&ui::document(), "#speed-display");
            *UPDATES_PER_SECOND.lock().unwrap() = value;
            display.set_value_as_number(value);
        } else {
            warn!("Too big of a number");
        }
    };
    add_event_listener(&slider, "input", update_func);
    add_event_listener(&number_input, "change", update_func);
}

fn select_rom_handler(document: &Document) {
    let selector: HtmlSelectElement = get_element(document, "#rom-selector");
    add_event_listener(&selector, "change", |event| {
        stop_runner();
        let selector = event
            .current_target()
            .unwrap()
            .dyn_into::<HtmlSelectElement>()
            .unwrap();
        let rom_path = selector.value();
        selector.blur().unwrap();
        let closure = Closure::new(|value: JsValue| {
            let response: Response = value
                .dyn_into()
                .expect("did not get a response object from fetching the rom");
            let closure = load_rom_from_array_promise();
            let _ = response
                .array_buffer()
                .expect("Could not turn rom into an array buffer")
                .then(&closure);
            closure.forget();
        });

        let _ = fetch_rom(&rom_path).then(&closure);
        closure.forget();
    });
}

fn fetch_rom(path: &str) -> Promise {
    info!("Fetching rom: {path}");
    let options = RequestInit::new();
    options.set_method("GET");
    let current_path = window().location().href().unwrap();
    let url = current_path
        .trim_end_matches("index.html")
        .trim_end_matches("/");
    let request = Request::new_with_str_and_init(&format!("{url}/roms/{path}"), &options)
        .expect("Could not create request to fetch rom");
    request
        .headers()
        .set("Accept", "application/octet-stream")
        .expect("Could not set header for rom request");
    window().fetch_with_request(&request)
}

fn reset_emulator_handler(document: &Document) {
    let reset_button: HtmlButtonElement = get_element(document, "#reset");
    add_event_listener(&reset_button, "click", |_| {
        get_program().lock().unwrap().reset();
    });
}

pub fn add_event_listener(target: &web_sys::EventTarget, event_name: &str, func: fn(e: Event)) {
    let closure: Closure<dyn Fn(Event)> = Closure::new(func);
    target
        .add_event_listener_with_event_listener(event_name, closure.as_ref().unchecked_ref())
        .expect("Could not add event listener");
    closure.forget();
}

pub fn trigger_select_splash_screen() {
    let event = Event::new("change").unwrap();
    let selector: HtmlSelectElement = get_element(&document(), "#rom-selector");
    selector.dispatch_event(&event).unwrap();
}
