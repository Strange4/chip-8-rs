use crate::{
    app::{self, Runner},
    emulator::{self, Program},
};
use log::info;
use wasm_bindgen::{prelude::Closure, Clamped, JsCast};
use web_sys::{
    js_sys::WebAssembly::Instance, CanvasRenderingContext2d, Document, Event, HtmlButtonElement,
    HtmlCanvasElement, ImageData, Window,
};
use web_time::Instant;

pub fn render_emulator(display: Vec<u8>, ctx: &CanvasRenderingContext2d) {
    let width = Program::width() as u32;
    let height = Program::height() as u32;

    let start = Instant::now();

    let data = ImageData::new_with_u8_clamped_array(Clamped(&display), width)
        .expect("Could not create the image data");

    ctx.put_image_data(&data, 0.0, 0.0)
        .expect("Could not put image data");

    info!("Rendering took: {:?}", start.elapsed());
}

pub fn set_handlers() {
    start_button_handler(&app::document());
}

fn start_button(document: &Document) -> HtmlButtonElement {
    document
        .query_selector("#start_button")
        .expect("The query was wrong")
        .expect("There was no button")
        .dyn_into()
        .expect("Could not dyn into a button")
}

fn start_button_handler(document: &Document) {
    let button = start_button(document);
    add_event_listener(&button, "click", |_| {
        emulator::get_program()
            .lock()
            .unwrap()
            .load_rom(emulator::ROM);
        let starter = Runner::start_loop();
        starter();
    });
}

fn add_event_listener(target: &web_sys::EventTarget, event_name: &str, func: fn(e: Event)) {
    let closure: Closure<dyn Fn(Event)> = Closure::new(func);
    target
        .add_event_listener_with_event_listener(event_name, closure.as_ref().unchecked_ref())
        .expect("Could not add event listener");
    closure.forget();
}
