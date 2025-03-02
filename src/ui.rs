use std::str::FromStr;

use crate::{
    app::document,
    debugger::{render_debugger, RENDER_DEBUGGER},
    emulator::Program,
};
use log::info;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, Element, ImageData, Node};
use web_time::Instant;

pub fn render_emulator(program: &Program, ctx: &CanvasRenderingContext2d) {
    let width = Program::width() as u32;

    let data = ImageData::new_with_u8_clamped_array(Clamped(&program.get_display()), width)
        .expect("Could not create the image data");

    ctx.put_image_data(&data, 0.0, 0.0)
        .expect("Could not put image data");
    let start = Instant::now();
    if *RENDER_DEBUGGER.lock().unwrap() {
        render_debugger(program);
    }
    info!("Rendering debugger took {:?}", start.elapsed());
}

pub fn create_element<T: JsCast>(name: &str) -> T {
    document()
        .create_element(name)
        .unwrap_or_else(|_| panic!("Could not create element {}", name))
        .dyn_into()
        .unwrap_or_else(|_| panic!("Could not dyn into element {}", std::any::type_name::<T>()))
}

pub fn get_element<T: JsCast>(document: &Document, id: &str) -> T {
    document
        .query_selector(id)
        .expect("The query was wrong")
        .unwrap_or_else(|| {
            panic!(
                "The element {} could not be found",
                std::any::type_name::<T>()
            )
        })
        .dyn_into()
        .unwrap_or_else(|_| {
            panic!(
                "Could not dyn into the element {}",
                std::any::type_name::<T>()
            )
        })
}

pub fn to_number<T>(node: &Node) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    node.text_content()
        .expect("Could not get text content of node")
        .parse()
        .expect("Could not parse node to number")
}

pub fn add_class_name(element: &Element, name: &str) {
    let mut classes = element.class_name();
    classes.push_str(&format!(" {name}"));
    element.set_class_name(&classes);
}

pub fn remove_class_name(element: &Element, name: &str) {
    let new_classes = element.class_name().replace(name, "").replace("  ", "");
    element.set_class_name(new_classes.trim());
}
