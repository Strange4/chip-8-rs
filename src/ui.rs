use crate::{
    app::{self, document},
    emulator::{self, Program},
};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlDivElement, HtmlTableCellElement, HtmlTableElement,
    HtmlTableRowElement, HtmlTableSectionElement, ImageData,
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
    if *app::RENDER_DEBUGGER.lock().unwrap() {
        render_debugger(program, debugger_area);
    } else {
        debugger_area.set_inner_text("");
    }
}

fn render_debugger(program: &Program, render_area: &HtmlDivElement) {
    let div: HtmlDivElement = create_element("div");
    div.set_class_name("scrollable");
    let table: HtmlTableElement = create_element("table");
    let row: HtmlTableRowElement = create_element("tr");
    let table_body: HtmlTableSectionElement = create_element("tbody");

    // headers
    let header: HtmlTableSectionElement = create_element("thead");
    let header_row = header.insert_row_with_index(-1).unwrap();
    let header1: HtmlTableCellElement = create_element("th");
    let header2: HtmlTableCellElement = create_element("th");
    header1.set_text_content(Some("Register Name"));
    header2.set_text_content(Some("Register Value"));
    header_row
        .append_child(&header1)
        .expect("Could not append header");
    header_row
        .append_child(&header2)
        .expect("Could not append header");

    // setting data
    program
        .variable_regsiters
        .iter()
        .enumerate()
        .for_each(|(i, value)| {
            let name_cell: HtmlTableCellElement = create_element("td");
            let value_cell: HtmlTableCellElement = create_element("td");
            let row = row.clone_node().unwrap();

            name_cell.set_text_content(Some(format!("{:#04x}", i).as_str()));
            value_cell.set_text_content(Some(format!("{:#04x}", *value).as_str()));
            row.append_child(&name_cell)
                .expect("Could not append name cell to row");
            row.append_child(&value_cell)
                .expect("Could not append value cell to row");
            table_body
                .append_child(&row)
                .expect("Could not append row to table");
        });
    table.append_child(&header).unwrap();
    table.append_child(&table_body).unwrap();
    div.append_child(&table).unwrap();
    // clearing render area before rendering
    render_area.set_text_content(Some(""));
    render_area
        .append_child(&div)
        .expect("Could not append the table to the render area");
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
