mod init_dom;
mod loader;
mod utils;
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

const CAMERA_TARGET: Vec3 = Vec3::ZERO;
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 200.0,
};

const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 400.0;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // register a panic hook that forwards Rust panics to JS console
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // init our necessary DOM elements (file uploader and webGL canvas)
    let dom = Dom::new()?;
    let web_gl_rendering_context: Rc<RefCell<WebGLState>> =
        Rc::new(RefCell::new(WebGLState::new(dom.canvas())?));

    // TODO: reintegrate user-uploaded files.  As a temporary workaround, load local file
    let dummy_file = include_str!("../minicooper.obj");
    let mut reader = BufReader::new(Cursor::new(dummy_file));
    let vertices = load_model(&mut reader).unwrap();
    log!("vertices: {:?}", vertices);

    // update the global state by adding the vertices
    web_gl_rendering_context
        .borrow_mut()
        .set_vertices(Some(vertices));

    log!("updated rendering context: {:?}", web_gl_rendering_context);

    web_gl_rendering_context
        .borrow()
        .draw(dom.canvas().width(), dom.canvas().height())?;

    Ok(())
}
