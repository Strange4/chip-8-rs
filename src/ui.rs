use crate::{
    app::{self, Runner},
    emulator::{self, Program},
};
use log::info;
use wasm_bindgen::{prelude::Closure, Clamped, JsCast, JsValue};
use web_sys::{
    js_sys::Uint8Array, CanvasRenderingContext2d, Document, Event, HtmlButtonElement,
    HtmlInputElement, ImageData,
};

pub fn render_emulator(display: Vec<u8>, ctx: &CanvasRenderingContext2d) {
    let width = Program::width() as u32;

    let data = ImageData::new_with_u8_clamped_array(Clamped(&display), width)
        .expect("Could not create the image data");

    ctx.put_image_data(&data, 0.0, 0.0)
        .expect("Could not put image data");
}

pub fn set_handlers() {
    start_button_handler(&app::document());
    load_rom_button(&app::document());
}

fn start_button_handler(document: &Document) {
    let button = start_button(document);
    add_event_listener(&button, "click", |_| {
        let starter = Runner::start_loop();
        starter();
    });
}

fn load_rom_button(document: &Document) {
    let input_element = load_button(document);
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

fn add_event_listener(target: &web_sys::EventTarget, event_name: &str, func: fn(e: Event)) {
    let closure: Closure<dyn Fn(Event)> = Closure::new(func);
    target
        .add_event_listener_with_event_listener(event_name, closure.as_ref().unchecked_ref())
        .expect("Could not add event listener");
    closure.forget();
}

fn start_button(document: &Document) -> HtmlButtonElement {
    document
        .query_selector("#start_button")
        .expect("The query was wrong")
        .expect("There was no button")
        .dyn_into()
        .expect("Could not dyn into a button")
}
fn load_button(document: &Document) -> HtmlInputElement {
    document
        .query_selector("#load_rom")
        .expect("The query was wrong")
        .expect("There was no button")
        .dyn_into()
        .expect("Could not dyn into a button")
}
