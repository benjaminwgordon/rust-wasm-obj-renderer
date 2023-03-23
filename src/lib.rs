mod init_dom;
mod loader;
mod wasm_utils;
mod web_gl_state;

use std::{
    cell::RefCell,
    io::{BufReader, Cursor},
    mem, panic,
    rc::Rc,
};

use glam::Vec3;
use init_dom::Dom;
use loader::load_model;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use web_gl_state::WebGLState;
use web_sys::{HtmlCanvasElement, MouseEvent};

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

#[wasm_bindgen]
struct SharedState {
    canvas_cursor_is_dragging: bool,
    canvas_cursor_xy_coordinates: [i32; 2],
    current_rotation: [f32; 2],
    web_gl_state: WebGLState,
}

#[wasm_bindgen]
impl SharedState {
    // wrapping this struct in Rc<RefCell<>> is a workaround for needing to be
    // moved into heap-allocated callbacks
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        Self {
            canvas_cursor_is_dragging: false,
            canvas_cursor_xy_coordinates: [0, 0],
            current_rotation: [0.0, 0.0],
            web_gl_state: WebGLState::new(canvas).unwrap(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // register a panic hook that forwards Rust panics to JS console
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // init DOM and shared mutable state
    let dom = Dom::new()?;
    let shared_state = Rc::new(RefCell::new(SharedState::new(dom.canvas())));

    // register the DOM callbacks that handle mouse events
    let mouse_down_shared_state = shared_state.clone();
    let mouse_down_event_callback = Closure::wrap(Box::new(move || {
        let _ = mem::replace(
            &mut mouse_down_shared_state
                .borrow_mut()
                .canvas_cursor_is_dragging,
            true,
        );
    }) as Box<dyn FnMut()>);

    let mouse_up_shared_state = shared_state.clone();
    let mouse_up_event_callback = Closure::wrap(Box::new(move || {
        let _ = mem::replace(
            &mut mouse_up_shared_state.borrow_mut().canvas_cursor_is_dragging,
            false,
        );
    }) as Box<dyn FnMut()>);

    let mouse_drag_shared_state = shared_state.clone();
    let mouse_drag_event_callback = Closure::wrap(Box::new(move |e: MouseEvent| {
        if mouse_drag_shared_state.borrow().canvas_cursor_is_dragging {
            let prev_x = mouse_drag_shared_state
                .borrow()
                .canvas_cursor_xy_coordinates
                .as_ref()[0];
            let prev_y = mouse_drag_shared_state
                .borrow()
                .canvas_cursor_xy_coordinates
                .as_ref()[1];

            let delta_x = prev_x - (e.client_x());
            let delta_y = prev_y - (e.client_y());

            let prev_rotation_xy = mouse_drag_shared_state.borrow().current_rotation.clone();
            let new_rotation_x = prev_rotation_xy[0] + delta_x as f32;
            let new_rotation_x = new_rotation_x % 360.0;
            let new_rotation_y = prev_rotation_xy[1] + delta_y as f32;
            let new_rotation_y = new_rotation_y % 360.0;

            let _ = mem::replace(
                &mut mouse_drag_shared_state.borrow_mut().current_rotation,
                [new_rotation_x, new_rotation_y],
            );
            let _ = mouse_drag_shared_state.borrow().web_gl_state.draw(
                800,
                600,
                mouse_drag_shared_state.borrow().current_rotation[0],
                mouse_drag_shared_state.borrow().current_rotation[1],
            );
        }

        let _ = mem::replace(
            &mut mouse_drag_shared_state
                .borrow_mut()
                .canvas_cursor_xy_coordinates,
            [e.client_x(), e.client_y()],
        );
    }) as Box<dyn FnMut(MouseEvent)>);

    dom.canvas().add_event_listener_with_callback(
        "mousedown",
        mouse_down_event_callback.as_ref().unchecked_ref(),
    )?;

    dom.canvas().add_event_listener_with_callback(
        "mouseup",
        mouse_up_event_callback.as_ref().unchecked_ref(),
    )?;

    dom.canvas().add_event_listener_with_callback(
        "mousemove",
        mouse_drag_event_callback.as_ref().unchecked_ref(),
    )?;

    mouse_down_event_callback.forget();
    mouse_up_event_callback.forget();
    mouse_drag_event_callback.forget();

    // TODO: reintegrate user-uploaded files.  As a temporary workaround, load local file
    let dummy_file = include_str!("../minicooper.obj");
    let mut reader = BufReader::new(Cursor::new(dummy_file));
    let vertices = load_model(&mut reader).unwrap();
    log!("vertices: {:?}", vertices);

    // update the global state by adding the vertices
    shared_state
        .clone()
        .borrow_mut()
        .web_gl_state
        .set_vertices(Some(vertices));

    // render one initial frame (all future frame draws are driven by user rotation inputs)
    shared_state.clone().borrow_mut().web_gl_state.draw(
        dom.canvas().width(),
        dom.canvas().height(),
        0.0,
        0.0,
    )?;

    Ok(())
}
