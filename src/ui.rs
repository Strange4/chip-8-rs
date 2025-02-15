use crate::emulator::Program;
use log::info;
use wasm_bindgen::prelude::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_time::Instant;

pub fn render_emulator(
    display: Vec<u8>,
    ctx: CanvasRenderingContext2d,
    canvas_width: u32,
    canvas_height: u32,
) {
    let width = Program::width() as u32;
    let height = Program::height() as u32;
    // let canvas_width = canvas.width();
    // let canvas_height = canvas.height();
    let pixel_width = canvas_width / width;
    let pixel_height = canvas_height / height;

    // let ctx = canvas

    let start = Instant::now();

    draw_lines(&ctx, width, height, pixel_width, pixel_height);
    draw_pixels(
        &ctx,
        width,
        height,
        display.as_slice(),
        pixel_width,
        pixel_height,
    );

    info!("Took {:?} to render", start.elapsed());
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

    ctx.stroke();
}

fn draw_lines(
    ctx: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    pixel_width: u32,
    pixel_height: u32,
) {
    let canvas_x_end = width * pixel_width;
    let canvas_y_end = height * pixel_height;
    ctx.set_stroke_style_str("#ffffff");
    for y in 0..height {
        ctx.move_to(0 as f64, (y * pixel_height) as f64);
        ctx.line_to(canvas_x_end as f64, (y * pixel_height) as f64);
    }
    for x in 0..width {
        ctx.move_to((x * pixel_width) as f64, 0 as f64);
        ctx.line_to((x * pixel_width) as f64, canvas_y_end as f64);
    }

    ctx.stroke();
}
