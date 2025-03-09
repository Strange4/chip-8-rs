use std::{cell::RefCell, rc::Rc, sync::Mutex};

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::CanvasRenderingContext2d;
use web_time::{Duration, Instant};

use crate::{
    debugger::INTERVAL_HANDLE,
    emulator::{get_program, Program},
    ui::{get_canvas_context, render_emulator, window},
};

const MIN_REPAINT_TIME: Duration = Duration::from_millis(16);
pub static UPDATES_PER_SECOND: Mutex<f64> = Mutex::new(1_000.0);

pub struct Runner {
    last_update: Instant,
    last_paint: Instant,
    context: CanvasRenderingContext2d,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            last_paint: Instant::now(),
            context: get_canvas_context(),
        }
    }

    pub fn start_loop() -> Box<dyn FnOnce()> {
        let function = Rc::new(RefCell::new(None));
        let starter = function.clone();
        let mut runner = Runner::new();

        *starter.borrow_mut() = Some(Closure::new(move || {
            let mut emulator = get_program().lock().unwrap();
            let time_since_last_update = runner.last_update.elapsed();
            let how_many_updates = (time_since_last_update.as_secs_f64()
                * *UPDATES_PER_SECOND.lock().unwrap())
            .floor() as usize;

            // because we can't update that fast, we'll run the updates that should've
            // been done since the last time it was updates
            for _ in 0..how_many_updates {
                emulator.tick();
            }

            if runner.last_paint.elapsed() > MIN_REPAINT_TIME {
                emulator.timer_tick();

                Runner::render(&emulator, &runner.context);
                runner.last_paint = Instant::now();
            }

            if how_many_updates != 0 {
                runner.last_update = Instant::now();
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
