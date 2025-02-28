use std::mem;

use crate::{
    app::{self, document},
    debugger::{BREAKPOINTS, RENDER_DEBUGGER},
    emulator::{self, Program},
    handlers,
};
use wasm_bindgen::{prelude::Closure, Clamped, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, Event, HtmlDivElement, HtmlTableCellElement,
    HtmlTableElement, HtmlTableRowElement, HtmlTableSectionElement, ImageData,
};

pub fn render_emulator(
    program: &Program,
    ctx: &CanvasRenderingContext2d,
    debugger_area: &HtmlDivElement,
) {
    let width = Program::width() as u32;

    let data = ImageData::new_with_u8_clamped_array(Clamped(&program.get_display()), width)
        .expect("Could not create the image data");

    ctx.put_image_data(&data, 0.0, 0.0)
        .expect("Could not put image data");
    if *RENDER_DEBUGGER.lock().unwrap() {
        render_debugger(program, debugger_area);
    } else {
        debugger_area.set_inner_text("");
    }
}

fn render_debugger(program: &Program, render_area: &HtmlDivElement) {
    let registers: HtmlDivElement = create_element("div");
    let memory: HtmlDivElement = create_element("div");
    registers
        .append_child(&render_registers(&program.variable_regsiters))
        .unwrap();
    registers.set_class_name("hovering-table");
    memory
        .append_child(&render_memory(&program.memory))
        .unwrap();

    // clearing render area before rendering
    render_area.set_text_content(Some(""));
    render_area.append_child(&registers).unwrap();
    render_area.append_child(&memory).unwrap();
}

fn render_registers(registers: &[u8]) -> HtmlTableElement {
    let table: HtmlTableElement = create_element("table");
    let table_body: HtmlTableSectionElement = create_element("tbody");

    let header = create_table_header(&["Register Name", "Register Value"]);

    // setting data
    registers.iter().enumerate().for_each(|(i, value)| {
        let row = create_row_from_data(i, *value);
        table_body.append_child(&row).unwrap();
    });
    table.append_child(&header).unwrap();
    table.append_child(&table_body).unwrap();
    table
}

fn render_memory(memory: &[u8]) -> HtmlTableElement {
    let table: HtmlTableElement = create_element("table");
    let table_body: HtmlTableSectionElement = create_element("tbody");

    let header = create_table_header(&["Memory Address", "Memory Value"]);

    // setting data
    memory.iter().enumerate().for_each(|(i, value)| {
        let row = create_row_from_data(i, *value);
        let closure: Closure<dyn Fn(Event)> = Closure::new(move |e: Event| {
            let row: HtmlTableRowElement = e
                .current_target()
                .expect("There was no target")
                .dyn_into()
                .expect("Could not dyn into a row");
            let is_selected = row.class_name().contains("breakpoint");
            let mut breakpoints = BREAKPOINTS
                .lock()
                .expect("Could not acquire breakpoint lock");
            if is_selected {
                row.set_class_name("");
                let index = breakpoints.iter().position(|index| *index == i).unwrap();
                breakpoints.remove(index);
            } else {
                row.set_class_name("breakpoint");
                breakpoints.push(i);
            }
        });
        row.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .expect("Could not add event listener to row");
        closure.forget();
        table_body.append_child(&row).unwrap();
    });
    table.append_child(&header).unwrap();
    table.append_child(&table_body).unwrap();
    table
}

fn create_table_header(headers: &[&str]) -> HtmlTableSectionElement {
    let header: HtmlTableSectionElement = create_element("thead");
    let header_row = header.insert_row_with_index(-1).unwrap();
    for name in headers {
        let cell: HtmlTableCellElement = create_element("th");
        cell.set_text_content(Some(&name));
        header_row.append_child(&cell).unwrap();
    }
    header
}

fn create_row_from_data(i: usize, data: u8) -> HtmlTableRowElement {
    let index_cell: HtmlTableCellElement = create_element("td");
    let value_cell: HtmlTableCellElement = create_element("td");
    let row: HtmlTableRowElement = create_element("tr");

    index_cell.set_text_content(Some(format!("{:#04x}", i).as_str()));
    value_cell.set_text_content(Some(format!("{:#04x}", data).as_str()));
    row.append_child(&index_cell).unwrap();
    row.append_child(&value_cell).unwrap();
    row
}
fn create_element<T: JsCast>(name: &str) -> T {
    document()
        .create_element(name)
        .expect(format!("Could not create element {}", name).as_str())
        .dyn_into()
        .expect(format!("Could not dyn into element {}", std::any::type_name::<T>()).as_str())
}

pub fn get_element<T: JsCast>(document: &Document, id: &str) -> T {
    document
        .query_selector(id)
        .expect("The query was wrong")
        .expect(
            format!(
                "The element {} could not be found",
                std::any::type_name::<T>()
            )
            .as_str(),
        )
        .dyn_into()
        .expect(
            format!(
                "Could not dyn into the element {}",
                std::any::type_name::<T>()
            )
            .as_str(),
        )
}
