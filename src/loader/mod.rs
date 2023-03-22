use std::{
    error::Error,
    io::{BufRead, BufReader},
};

use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};

type Verts = Vec<f32>;

use crate::{loader, log};

/**
 * When a user uploads a file, first we evaluate the list of uploaded files by name
 *
 * If at least one file is present in the upload list, we register a callback function
 * that fires when the file has finished uploading
 *
 * This callback parses the file contents into a list of vertices
 */
#[wasm_bindgen]
pub fn load_obj(file_input: web_sys::HtmlInputElement) {
    //Check the file list from the input
    let filelist = match file_input.files() {
        Some(files) => files,
        None => {
            log!("files: None");
            panic!();
        }
    };

    let file = filelist.get(0).expect("Failed to get File from filelist!");
    let file_reader: web_sys::FileReader = match web_sys::FileReader::new() {
        Ok(f) => f,
        Err(_) => web_sys::FileReader::new().expect(""),
    };

    let fr_c = file_reader.clone();

    // create onLoadEnd callback
    let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
        let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
        let arr_slice = array.to_vec();
        let mut reader = BufReader::new(&arr_slice[..]);
        match loader::load_model(&mut reader) {
            Err(e) => {
                log!("Failed to parse into verts, tris, normals {:?}", e);
            }
            Ok(_vertices) => {
                // TODO: do something with the list of vertices
            }
        };
    }) as Box<dyn Fn(web_sys::ProgressEvent)>);

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader
        .read_as_array_buffer(&file)
        .expect("blob not readable");
    onloadend_cb.forget();
}

pub fn load_model(reader: &mut impl BufRead) -> Result<Verts, Box<dyn Error>> {
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
