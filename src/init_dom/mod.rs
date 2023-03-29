use std::{cell::RefCell, mem, rc::Rc};

/**
 * sets up the initial DOM state
 */
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlInputElement, MouseEvent, WheelEvent};

use crate::{log, SharedState};

extern crate web_sys;

pub struct Dom {
    pub canvas: HtmlCanvasElement,
    pub file_input: HtmlInputElement,
}

impl Dom {
    pub fn new() -> Result<Dom, JsValue> {
        let window = web_sys::window().expect("window exists in DOM");
        let document = window.document().expect("document exists in widow");
        let body = document.body().expect("body exists in document");

        let container = document.create_element("div")?;
        body.append_child(&container)?;

        let file_input = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()?;
        file_input.set_attribute("type", "file")?;
        file_input.set_attribute("id", "file_upload_input")?;
        container.append_child(&file_input)?;

        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        canvas.set_attribute("width", "600px")?;
        canvas.set_attribute("height", "400px")?;
        container.append_child(&canvas)?;

        let submit_button = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()?;
        submit_button.set_attribute("type", "submit")?;
        submit_button.set_value("Confirm Orientation");
        container.append_child(&submit_button)?;

        Ok(Dom { canvas, file_input })
    }

    pub fn register_dom_event_callbacks(self, shared_state: Rc<RefCell<SharedState>>) {
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

        let mouse_wheel_shared_state = shared_state.clone();
        let mouse_wheel_event_callback = Closure::wrap(Box::new(move |e: WheelEvent| {
            e.prevent_default();
            let prev_offset = mouse_wheel_shared_state.borrow_mut().camera_offset;
            let scroll_delta = e.delta_y();
            log!("mouse wheel delta: {:?}", scroll_delta);
            let scaling_factor: f64 = 0.25;
            let new_camera_offset = prev_offset + (scaling_factor * scroll_delta) as f32;
            let new_camera_offset = new_camera_offset.clamp(1.0, 1000.0);
            let _ = mem::replace(
                &mut mouse_wheel_shared_state.borrow_mut().camera_offset,
                new_camera_offset,
            );
            mouse_wheel_shared_state.borrow().web_gl_state.draw(
                800,
                600,
                mouse_wheel_shared_state.borrow().current_rotation[0],
                mouse_wheel_shared_state.borrow().current_rotation[1],
                mouse_wheel_shared_state.borrow().z_near,
                mouse_wheel_shared_state.borrow().z_far,
                mouse_wheel_shared_state.borrow().camera_offset,
            );
        }) as Box<dyn FnMut(WheelEvent)>);

        let mouse_drag_shared_state = shared_state;
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

                let prev_rotation_xy = mouse_drag_shared_state.borrow().current_rotation;
                let new_rotation_x = prev_rotation_xy[0] + delta_x as f32;
                let new_rotation_x = new_rotation_x % 360.0;
                let new_rotation_y = prev_rotation_xy[1] + delta_y as f32;
                let new_rotation_y = new_rotation_y % 360.0;

                let _ = mem::replace(
                    &mut mouse_drag_shared_state.borrow_mut().current_rotation,
                    [new_rotation_x, new_rotation_y],
                );
                mouse_drag_shared_state.borrow().web_gl_state.draw(
                    800,
                    600,
                    mouse_drag_shared_state.borrow().current_rotation[0],
                    mouse_drag_shared_state.borrow().current_rotation[1],
                    mouse_drag_shared_state.borrow().z_near,
                    mouse_drag_shared_state.borrow().z_far,
                    mouse_drag_shared_state.borrow().camera_offset,
                );
            }

            let _ = mem::replace(
                &mut mouse_drag_shared_state
                    .borrow_mut()
                    .canvas_cursor_xy_coordinates,
                [e.client_x(), e.client_y()],
            );
        }) as Box<dyn FnMut(MouseEvent)>);

        let _ = self.canvas.add_event_listener_with_callback(
            "mousedown",
            mouse_down_event_callback.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.add_event_listener_with_callback(
            "mouseup",
            mouse_up_event_callback.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.add_event_listener_with_callback(
            "mousemove",
            mouse_drag_event_callback.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.add_event_listener_with_callback(
            "wheel",
            mouse_wheel_event_callback.as_ref().unchecked_ref(),
        );

        mouse_down_event_callback.forget();
        mouse_up_event_callback.forget();
        mouse_drag_event_callback.forget();
        mouse_wheel_event_callback.forget();
    }
}
