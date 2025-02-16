use std::{cell::RefCell, rc::Rc, sync::Mutex};

use log::info;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_time::{Duration, Instant};

use crate::{
    emulator::{get_program, Program},
    ui,
};

const MIN_REPAINT_TIME: Duration = Duration::from_millis(16);
static INTERVAL_HANDLE: Mutex<Option<i32>> = Mutex::new(None);

pub struct Runner {
    last_update: Instant,
    last_paint: Instant,
    last_info: Instant,
    tick_number: i32,
    // emulator: Program,
    updates_per_second: f64,
    context: CanvasRenderingContext2d,
    // canvas: HtmlCanvasElement,
    // window: Window,
}

impl Runner {
    pub fn new() -> Self {
        // let mut emul = Program::new();
        // emul.load_rom(ROM);
        Self {
            last_update: Instant::now(),
            last_paint: Instant::now(),
            last_info: Instant::now(),
            tick_number: 0,
            // emulator: emul,
            updates_per_second: 1_000_000.0,
            context: get_context(),
            // canvas: canvas(),
            // window: window(),
        }
    }

    pub fn start_loop() -> Box<dyn FnOnce()> {
        let function = Rc::new(RefCell::new(None));
        let starter: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = function.clone();
        let mut app = Runner::new();
        // app.set_event_handlers();

        *starter.borrow_mut() = Some(Closure::new(move || {
            let mut emulator = get_program().lock().unwrap();
            let time_since_last_update = app.last_update.elapsed();
            let how_many_updates =
                (time_since_last_update.as_secs_f64() * app.updates_per_second).ceil() as usize;

            // because we can't update that fast, we'll run the updates that should've
            // been done since the last time it was updates
            for _ in 0..how_many_updates {
                app.tick_number += 1;
                emulator.tick();
            }

            if how_many_updates != 0 {
                app.last_update = Instant::now();
            }

            if app.last_paint.elapsed() > MIN_REPAINT_TIME {
                emulator.timer_tick();
                Runner::render(&emulator, &app.context);
                app.last_paint = Instant::now();
            }

            if app.last_info.elapsed() > Duration::from_secs(1) {
                info!(
                    "Update number: {}, time passed: {:?}",
                    app.tick_number, app.last_info
                );
                app.last_info = Instant::now();
            }

            // loop and reloop
            set_timeout(function.borrow().as_ref().unwrap());
        }));

        Box::new(move || {
            // after this function is invocted, the starter function will be dropped
            // and there will only be one function in the rc
            set_timeout(starter.borrow().as_ref().unwrap());
        })
    }

    fn render(emulator: &Program, ctx: &CanvasRenderingContext2d) {
        let display = emulator.get_display();
        ui::render_emulator(display, ctx);
    }
}

pub fn canvas() -> HtmlCanvasElement {
    document()
        .query_selector("canvas")
        .expect("the selector is not valid")
        .expect("There was no canvas in the html document")
        .dyn_into()
        .expect("Could not dyn into canvas")
}

fn get_context() -> CanvasRenderingContext2d {
    canvas()
        .get_context("2d")
        .expect("Could not get the canvas context")
        .expect("There was no context")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .expect("Couldn't transform the js object into canvas context")
}

fn set_timeout(f: &Closure<dyn FnMut()>) {
    let handle = window()
        .set_timeout_with_callback(f.as_ref().unchecked_ref())
        .expect("Couldn't register 'request_animation_frame'");
    INTERVAL_HANDLE.lock().unwrap().replace(handle);
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global 'window' found")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("there was no document for this window")
}

fn body() -> web_sys::HtmlElement {
    document()
        .body()
        .expect("This document doesn't have a body")
}
