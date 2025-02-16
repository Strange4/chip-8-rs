use crate::{
    app::{self, Runner},
    emulator::{self, Program},
};
use log::info;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, Event, HtmlButtonElement, HtmlCanvasElement, Window,
};

pub fn render_emulator(
    display: Vec<u8>,
    ctx: &CanvasRenderingContext2d,
    canvas_width: u32,
    canvas_height: u32,
) {
    let width = Program::width() as u32;
    let height = Program::height() as u32;
    let pixel_width = canvas_width / width;
    let pixel_height = canvas_height / height;

    draw_pixels(
        &ctx,
        width,
        height,
        display.as_slice(),
        pixel_width,
        pixel_height,
    );
}

fn draw_pixels(
    ctx: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    display: &[u8],
    pixel_width: u32,
    pixel_height: u32,
) {
    const OFF_COLOR: &str = "#000000";
    const ON_COLOR: &str = "#ff5733";

    for y in 0..height {
        for x in 0..width {
            let location = Program::pixel_location(x as u8, y as u8);
            if display[location] == 1 {
                ctx.set_fill_style_str(ON_COLOR);
            } else {
                ctx.set_fill_style_str(OFF_COLOR);
            }
            ctx.fill_rect(
                x as f64 * pixel_width as f64,
                y as f64 * pixel_height as f64,
                pixel_width as f64,
                pixel_height as f64,
            );
        }
    }
}

fn fix_dpi(window: &Window, canvas: &mut HtmlCanvasElement) {
    let dpi = window.device_pixel_ratio();
    let computed = window
        .get_computed_style(&canvas)
        .expect("could not get the computed style")
        .expect("There was no computed style");

    let height = computed
        .get_property_value("height")
        .expect("Could not get the height")
        .strip_suffix("px")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let width = computed
        .get_property_value("width")
        .expect("Could not get the width")
        .strip_suffix("px")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    canvas
        .set_attribute("height", format!("{}px", (height as f64) * dpi).as_str())
        .expect("Damm that's crazyy");
    canvas
        .set_attribute("width", format!("{}px", (width as f64) * dpi).as_str())
        .expect("That's even crazier");
}

pub fn set_handlers() {
    start_button_handler(&app::document());
    resize_handler(&app::window());
}

fn resize_handler(window: &Window) {
    // fix once and then set the resize event
    fix_dpi(window, &mut app::canvas());
    let window2 = window.clone();

    let closure: Closure<dyn Fn(web_sys::Event)> = Closure::new(move |_: web_sys::Event| {
        info!("Resizing");
        fix_dpi(&window2, &mut app::canvas());
    });
    window
        .add_event_listener_with_event_listener("resize", closure.as_ref().unchecked_ref())
        .expect("Could not add even listener");
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
