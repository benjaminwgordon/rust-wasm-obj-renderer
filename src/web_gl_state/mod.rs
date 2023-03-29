use std::f32::consts::PI;

use glam::{Mat4, Vec3};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::{loader::ModelData, log, CAMERA_TARGET};

pub struct WebGLState {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    model_data: Option<ModelData>,
}

impl WebGLState {
    pub fn set_model_data(&mut self, model_data: Option<ModelData>) {
        self.model_data = model_data;
    }

    pub fn new(canvas: &HtmlCanvasElement) -> Result<WebGLState, JsValue> {
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
              gl_Position = u_projection * u_view * u_world * vec4(a_position, 1.0);
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

        Ok(WebGLState {
            context,
            program,
            model_data: None,
        })
    }

    pub fn draw(
        &self,
        canvas_width: u32,
        canvas_height: u32,
        x_rot: f32,
        y_rot: f32,
        z_near: f32,
        z_far: f32,
        camera_offset: f32,
    ) {
        match &self.model_data {
            None => {
                log!("no vertices available to buffer");
                panic!();
            }
            Some(model_data) => {
                self.context
                    .viewport(0, 0, canvas_width as i32, canvas_height as i32);
                self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
                self.context.enable(WebGl2RenderingContext::CULL_FACE);
                self.context.cull_face(WebGl2RenderingContext::BACK);
                self.context.use_program(Some(&self.program));

                let world_matrix = Mat4::IDENTITY;
                let field_of_view_radians = 60.0 * PI / 180.0;
                let aspect: f32 = canvas_width as f32 / canvas_height as f32;
                let projection_matrix =
                    Mat4::perspective_lh(field_of_view_radians, aspect, z_near, z_far);
                let up: Vec3 = Vec3::from([0.0, 1.0, 0.0]);
                let view_matrix = Mat4::look_at_lh(
                    Vec3::from([0.0, camera_offset, camera_offset]),
                    CAMERA_TARGET,
                    up,
                );

                // TODO: rotate world space
                let x_rotation_matrix = Mat4::from_rotation_x(-1.0 * y_rot * PI / 180.0);
                let y_rotation_matrix = Mat4::from_rotation_y(0.0);
                let z_rotation_matrix = Mat4::from_rotation_z(x_rot * PI / 180.0);

                let rotated_world_matrix = world_matrix
                    .mul_mat4(&x_rotation_matrix)
                    .mul_mat4(&y_rotation_matrix)
                    .mul_mat4(&z_rotation_matrix);

                // clear the scene
                let _ = self.context.clear_color(0.2, 0.2, 0.2, 1.0);
                self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                // get shader uniform locations
                let u_view = self.context.get_uniform_location(&self.program, "u_view");
                let u_world = self.context.get_uniform_location(&self.program, "u_world");
                let u_projection = self
                    .context
                    .get_uniform_location(&self.program, "u_projection");

                // set shader uniforms
                self.context.uniform_matrix4fv_with_f32_array(
                    u_view.as_ref(),
                    false,
                    &view_matrix.to_cols_array(),
                );

                self.context.uniform_matrix4fv_with_f32_array(
                    u_world.as_ref(),
                    false,
                    &rotated_world_matrix.to_cols_array(),
                );
                self.context.uniform_matrix4fv_with_f32_array(
                    u_projection.as_ref(),
                    false,
                    &projection_matrix.to_cols_array(),
                );

                let mut vertex_count = 0;
                // load vertex position and index data into a buffer for each model rendered
                let vert_position_count = self.load_buffer_from_array(
                    "a_position",
                    model_data.vertices.clone(),
                    WebGl2RenderingContext::FLOAT,
                );
                let index_count = self.load_index_buffer_from_array(model_data.indices.clone());
                vertex_count += vert_position_count;

                self.context.draw_elements_with_i32(
                    WebGl2RenderingContext::POINTS,
                    index_count,
                    WebGl2RenderingContext::UNSIGNED_INT,
                    0,
                );
            }
        }
    }

    pub fn load_buffer_from_array(&self, location: &str, array: Vec<f32>, data_type: u32) -> i32 {
        let position_attribute_location = self.context.get_attrib_location(&self.program, location);

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

        (array.len() / 3) as i32
    }

    pub fn load_index_buffer_from_array(&self, array: Vec<u32>) -> i32 {
        let buffer = self
            .context
            .create_buffer()
            .ok_or("Failed to create buffer")
            .unwrap();

        self.context
            .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let index_array_buf_view = js_sys::Uint32Array::view(&array);

            self.context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &index_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        array.len() as i32
    }
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
