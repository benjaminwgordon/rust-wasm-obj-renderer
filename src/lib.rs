mod init_dom;
mod loader;
mod wasm_utils;
mod web_gl_state;

use std::{
    cell::RefCell,
    io::{BufReader, Cursor},
    panic,
    rc::Rc,
};

use glam::Vec3;
use init_dom::Dom;
use loader::load_model;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_gl_state::WebGLState;
use web_sys::HtmlCanvasElement;

const CAMERA_TARGET: Vec3 = Vec3 {
    x: -50.0,
    y: 80.0,
    z: 0.0,
};
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 200.0,
    z: 200.0,
};

const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 400.0;

struct SharedState {
    canvas_cursor_is_dragging: bool,
    canvas_cursor_xy_coordinates: [i32; 2],
    current_rotation: [f32; 2],
    web_gl_state: WebGLState,
}

impl SharedState {
    // wrapping this struct in Rc<RefCell<>> is a workaround for needing to be
    // moved into heap-allocated callbacks
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        Self {
            canvas_cursor_is_dragging: false,
            canvas_cursor_xy_coordinates: [0, 0],
            current_rotation: [90.0, 90.0],
            web_gl_state: WebGLState::new(&canvas).unwrap(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // register a panic hook that forwards Rust panics to JS console
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // init DOM and shared mutable state
    let dom = Dom::new()?;
    let shared_state = Rc::new(RefCell::new(SharedState::new(&dom.canvas)));

    // register DOM callbacks for mouse events
    let dom_shared_state = shared_state.clone();
    dom.register_dom_event_callbacks(dom_shared_state);

    // TODO: reintegrate user-uploaded files.  As a temporary workaround, load local file
    let dummy_file = include_str!("../minicooper.obj");
    let mut reader = BufReader::new(Cursor::new(dummy_file));
    let vertices = load_model(&mut reader).unwrap();

    // add loaded model's vertices to shared state
    shared_state
        .clone()
        .borrow_mut()
        .web_gl_state
        .set_vertices(Some(vertices));

    // render one initial frame (all future frame draws are driven by user mouse inputs)
    shared_state
        .clone()
        .borrow_mut()
        .web_gl_state
        .draw(800, 600, 90.0, 90.0)?;

    Ok(())
}
