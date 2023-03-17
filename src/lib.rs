mod utils;

use std::error::Error;
use std::io::{BufRead, BufReader};

use glam::{Mat4, Vec3};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};
extern crate web_sys;

const CAMERA_TARGET: Vec3 = glam::f32::Vec3::ZERO;
const CAMERA_POSITION: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 200.0,
};
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 200.0;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WebGLState {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
}

#[wasm_bindgen]
impl WebGLState {
    #[wasm_bindgen]
    pub fn new() -> Result<WebGLState, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"
            attribute vec3 a_position;
            
            uniform mat4 u_projection;
            uniform mat4 u_view;
            uniform mat4 u_world;
                 
            void main() {
              gl_Position = u_projection * u_view * u_world * vec4(a_position, 0.0);
            }
            "##,
        )?;

        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"precision mediump float;

            void main() {
                gl_FragColor = vec4(1.0,0.7,0.0,1.0);
            }
            "##,
        )?;

        let program = link_program(&context, &vert_shader, &frag_shader)?;

        Ok(WebGLState { context, program })
    }

    pub fn update_render_verts(&self, vertices: Vec<f32>) {
        let canvas = self
            .context
            .canvas()
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        // calculate camera matrix for view projection
        self.context
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.context.enable(WebGl2RenderingContext::CULL_FACE);
        self.context.cull_face(WebGl2RenderingContext::BACK);

        // get shader uniform locations

        let u_view = self.context.get_uniform_location(&self.program, "u_view");
        let u_world = self.context.get_uniform_location(&self.program, "u_world");
        let u_projection = self
            .context
            .get_uniform_location(&self.program, "u_projection");

        let field_of_view_radians = 60.0 * 3.141592653589793 / 180.0;
        let aspect: f32 = canvas.width() as f32 / canvas.height() as f32;
        let projection =
            glam::f32::Mat4::perspective_lh(field_of_view_radians, aspect, Z_NEAR, Z_FAR);

        let up: Vec3 = Vec3::from([0.0, 1.0, 0.0]);
        let camera = glam::f32::Mat4::look_at_lh(CAMERA_POSITION, CAMERA_TARGET, up);
        let view = camera.inverse();

        self.context.use_program(Some(&self.program));

        // set shader uniforms
        self.context.uniform_matrix4fv_with_f32_array(
            u_view.as_ref(),
            false,
            &view.to_cols_array(),
        );
        self.context.uniform_matrix4fv_with_f32_array(
            u_world.as_ref(),
            false,
            &Mat4::IDENTITY.to_cols_array(),
        );
        self.context.uniform_matrix4fv_with_f32_array(
            u_projection.as_ref(),
            false,
            &projection.to_cols_array(),
        );

        let vert_position_count =
            self.load_buffer_from_array("a_position", vertices, WebGl2RenderingContext::FLOAT);

        self.context.clear_color(0.9, 0.9, 0.9, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, vert_position_count, 0);
    }

    #[wasm_bindgen]
    pub fn load_obj(self, file_input: web_sys::HtmlInputElement) {
        //Check the file list from the input
        log!("file input: {:?}", file_input);
        let filelist = match file_input.files() {
            Some(files) => {
                log!("files: {:?}", files);
                files
            }
            None => {
                log!("files: None");
                panic!();
            }
        };

        //Do not allow blank inputs
        if filelist.length() < 1 {
            alert("Please select at least one file.");
        }
        if filelist.get(0) == None {
            alert("Please select a valid file");
        }

        let file = filelist.get(0).expect("Failed to get File from filelist!");
        log!("file content: {:?}", file);
        let file_reader: web_sys::FileReader = match web_sys::FileReader::new() {
            Ok(f) => f,
            Err(_) => {
                alert("There was an error creating a file reader");
                web_sys::FileReader::new().expect("")
            }
        };

        let fr_c = file_reader.clone();
        // create onLoadEnd callback

        let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
            let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
            let arr_slice = array.to_vec();
            let mut reader = BufReader::new(&arr_slice[..]);
            match load_model(&mut reader) {
                Err(e) => log!("Failed to parse into verts, tris, normals {:?}", e),
                Ok(verts) => {
                    let _ = &self.clone().update_render_verts(verts);
                }
            };
        }) as Box<dyn Fn(web_sys::ProgressEvent)>);

        file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
        file_reader
            .read_as_array_buffer(&file)
            .expect("blob not readable");
        onloadend_cb.forget();
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

type Verts = Vec<f32>;

fn load_model(reader: &mut impl BufRead) -> Result<Verts, Box<dyn Error>> {
    let (models, _) = tobj::load_obj_buf(
        reader,
        &tobj::LoadOptions {
            reorder_data: false,
            single_index: true, //nah
            triangulate: false, //yes please
            ignore_points: true,
            ignore_lines: true,
        },
        |_| Ok((Vec::new(), Default::default())),
    )?;

    let mut verticies: Vec<f32> = Vec::new();

    for tobj::Model { mesh, .. } in models {
        verticies.extend(mesh.positions);
    }
    Ok(verticies)
}

impl WebGLState {
    pub fn load_buffer_from_array(&self, location: &str, array: Vec<f32>, data_type: u32) -> i32 {
        log!(
            "Getting vertex attribute location for location({:?})",
            location
        );

        let position_attribute_location = self.context.get_attrib_location(&self.program, location);

        log!(
            "array_buffer_attribute_location: {:?}",
            position_attribute_location
        );

        let buffer = self
            .context
            .create_buffer()
            .ok_or("Failed to create buffer")
            .unwrap();
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&array);

            self.context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        self.context.vertex_attrib_pointer_with_i32(
            position_attribute_location as u32,
            3,
            data_type,
            false,
            0,
            0,
        );
        self.context
            .enable_vertex_attrib_array(position_attribute_location as u32);

        let vert_count = (array.len() / 3) as i32;
        vert_count
    }
}
