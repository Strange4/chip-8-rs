use std::{cell::RefCell, rc::Rc};

use log::info;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_time::{Duration, Instant};

use crate::emulator::Program;

pub struct App {
    last_update: Instant,
    tick_number: i32,
    emulator: Program,
    time_per_frame: Duration,
}

impl App {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            tick_number: 0,
            emulator: Program::new(),
            time_per_frame: Duration::from_millis(16),
        }
    }

    pub fn start_loop() -> Box<dyn FnOnce()> {
        let function = Rc::new(RefCell::new(None));
        let starter: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = function.clone();
        let mut app = App::new();
        *starter.borrow_mut() = Some(Closure::new(move || {
            if app.last_update.elapsed() > app.time_per_frame {
                app.tick_number += 1;
                app.last_update = Instant::now();
                body().set_text_content(Some(&format!(
                    "Update number: {}, time passed: {:?}",
                    app.tick_number, app.last_update
                )));
            }
            info!("called request animation frame");
            if app.tick_number > 1_000_000 {
                body().set_text_content(Some("Ya done now"));

                // removing the reference so it gets cleaned up
                let _ = function.take();
                return;
            }
            // loop and reloop
            request_animation_frame(function.borrow().as_ref().unwrap());
        }));

        Box::new(move || {
            // after this function is invocted, the starter function will be dropped
            // and there will only be one function in the rc
            request_animation_frame(starter.borrow().as_ref().unwrap());
        })
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Couldn't register 'request_animation_frame'");
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
