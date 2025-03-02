use std::{cell::RefCell, rc::Rc};

use log::info;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_time::{Duration, Instant};

use crate::{
    debugger::INTERVAL_HANDLE,
    emulator::{get_program, Program},
    ui::render_emulator,
};

const MIN_REPAINT_TIME: Duration = Duration::from_millis(16);

pub struct Runner {
    last_update: Instant,
    last_paint: Instant,
    last_info: Instant,
    tick_number: i32,
    updates_per_second: f64,
    context: CanvasRenderingContext2d,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            last_paint: Instant::now(),
            last_info: Instant::now(),
            tick_number: 0,
            updates_per_second: 1_000_000.0,
            context: get_canvas_context(),
        }
    }

    pub fn start_loop() -> Box<dyn FnOnce()> {
        let function = Rc::new(RefCell::new(None));
        let starter = function.clone();
        let mut app = Runner::new();

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
        render_emulator(emulator, ctx);
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

pub fn get_canvas_context() -> CanvasRenderingContext2d {
    canvas()
        .get_context("2d")
        .expect("Could not get the canvas context")
        .expect("There was no context")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .expect("Couldn't transform the js object into canvas context")
}

fn set_timeout(f: &Closure<dyn FnMut()>) {
    let window = window();
    let mut old_handle = INTERVAL_HANDLE.lock().unwrap();

    if old_handle.is_some() {
        window.clear_interval_with_handle(old_handle.unwrap());
    }

    let new_handle = window
        .set_timeout_with_callback(f.as_ref().unchecked_ref())
        .expect("Couldn't register 'request_animation_frame'");
    old_handle.replace(new_handle);
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global 'window' found")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("there was no document for this window")
}
