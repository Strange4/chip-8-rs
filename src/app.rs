use std::{cell::RefCell, rc::Rc};

use log::info;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{js_sys::Function, CanvasRenderingContext2d, HtmlCanvasElement};
use web_time::{Duration, Instant};

use crate::{emulator::Program, ui};

const ROM: &'static [u8; 132] = include_bytes!("../roms/IBM Logo.ch8");

const MIN_REPAINT_TIME: Duration = Duration::from_millis(16);

pub struct App {
    last_update: Instant,
    last_paint: Instant,
    tick_number: i32,
    emulator: Program,
    updates_per_second: f64,
    context: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
}

impl App {
    fn new() -> Self {
        let mut emul = Program::new();
        emul.load_rom(ROM);
        Self {
            last_update: Instant::now(),
            last_paint: Instant::now(),
            tick_number: 0,
            emulator: emul,
            updates_per_second: 1.0,
            context: get_context(),
            canvas: canvas(),
        }
    }

    pub fn start_loop() -> Box<dyn FnOnce()> {
        let function = Rc::new(RefCell::new(None));
        let starter: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = function.clone();
        let mut app = App::new();

        *starter.borrow_mut() = Some(Closure::new(move || {
            let time_since_last_update = app.last_update.elapsed();
            let how_many_updates =
                (time_since_last_update.as_secs_f64() * app.updates_per_second).ceil() as usize;

            // because we can't update that fast, we'll run the updates that should've
            // been done since the last time it was updates
            for _ in 0..how_many_updates {
                app.tick_number += 1;
                app.emulator.tick();
            }

            if how_many_updates != 0 {
                app.last_update = Instant::now();
                app.emulator.timer_tick();
            }

            if app.last_paint.elapsed() > MIN_REPAINT_TIME {
                app.render();
                app.last_paint = Instant::now();
                info!("Just painted");
            }

            if app.tick_number % 10 == 0 {
                info!(
                    "Update number: {}, time passed: {:?}",
                    app.tick_number, app.last_update
                );
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

    fn render(&self) {
        let display = self.emulator.get_display();

        let width = self.canvas.width();
        let height = self.canvas.height();
        let ctx = self.context.clone();

        request_animation_frame(
            Closure::once_into_js(move || {
                ui::render_emulator(display, ctx, width, height);
            })
            .unchecked_ref(),
        );
    }
}

fn canvas() -> HtmlCanvasElement {
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
    window()
        .set_timeout_with_callback(f.as_ref().unchecked_ref())
        .expect("Couldn't register 'request_animation_frame'");
}

fn request_animation_frame(f: &Function) {
    window()
        .request_animation_frame(f)
        .expect("Could not set request animation frame");
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global 'window' found")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("there was no document for this window")
}

fn body() -> web_sys::HtmlElement {
    document()
        .body()
        .expect("This document doesn't have a body")
}
