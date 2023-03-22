/**
 * sets up the initial DOM state
 */
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Document, HtmlCanvasElement, HtmlElement, HtmlInputElement, Window};

extern crate web_sys;

#[wasm_bindgen]
pub struct Dom {
    window: Window,
    document: Document,
    body: HtmlElement,
    canvas: HtmlCanvasElement,
    file_input: HtmlInputElement,
}

#[wasm_bindgen]
impl Dom {
    // getters are required for all non-copy struct fields for passing to JS
    #[wasm_bindgen(getter)]
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        self.canvas = canvas;
    }

    #[wasm_bindgen(getter)]
    pub fn file_input(&self) -> HtmlInputElement {
        self.file_input.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_file_input(&mut self, file_input: HtmlInputElement) {
        self.file_input = file_input;
    }

    #[wasm_bindgen(constructor)]
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

        Ok(Dom {
            window,
            document,
            body,
            canvas,
            file_input,
        })
    }
}
