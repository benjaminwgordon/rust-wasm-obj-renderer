use core::panic;
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

use ahash::AHashMap;
use obj::ObjMaterial;
use tobj::{load_mtl_buf, MTLLoadResult};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};

type Verts = Vec<f32>;
type Indices = Vec<u32>;

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

#[derive(Debug)]
pub struct ModelData {
    pub vertices: Verts,
    pub indices: Indices,
}

pub fn load_model(reader: &mut impl BufRead) -> Result<ModelData, Box<dyn Error>> {
    // minimal obj parser that ignores materials, normals, etc...
    // only parses positions and index matches paths

    let mut vertex_position_list = Vec::<[f32; 3]>::new();
    // dummy coordinate to support 1 based indexing
    vertex_position_list.push([0.0, 0.0, 0.0]);

    let mut triangle_list = Vec::<[u32; 3]>::new();
    //let mut vertex_index_offset: usize = 0;
    triangle_list.push([0, 0, 0]);

    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() != 0 {
        let mut split = buf.split_whitespace();
        let prefix = split.next();
        match prefix {
            Some(_) => match prefix {
                Some(char) => {
                    //log!("line: {}", buf);
                    match char {
                        "v" => {
                            // assume we have x y and z data
                            let x_coord = split
                                .next()
                                .expect("there is x data")
                                .parse()
                                .expect("x_coord should be parsable into f32");
                            let y_coord = split
                                .next()
                                .expect("there is y data")
                                .parse()
                                .expect("y_coord should be parsable into f32");
                            let z_coord = split
                                .next()
                                .expect("there is z data")
                                .parse()
                                .expect("z_coord should be parsable into f32");
                            vertex_position_list.push([x_coord, y_coord, z_coord]);
                        }
                        "f" => {
                            let vertex_1_index: u32 = split
                                .next()
                                .expect("there is data for vertex 1")
                                .split("/")
                                .next()
                                .expect("there is a vertex number for vertex 1")
                                .parse()
                                .expect("there is a u32 parsable index for vertex 1");
                            let vertex_2_index: u32 = split
                                .next()
                                .expect("there is data for vertex 2")
                                .split("/")
                                .next()
                                .expect("there is a vertex number for vertex 2")
                                .parse()
                                .expect("there is a u32 parsable index for vertex 2");
                            let vertex_3_index: u32 = split
                                .next()
                                .expect("there is data for vertex 3")
                                .split("/")
                                .next()
                                .expect("there is a vertex number for vertex 3")
                                .parse()
                                .expect("there is a u32 parsable index for vertex 3");
                            triangle_list.push([vertex_1_index, vertex_2_index, vertex_3_index]);
                        }
                        "g" => {
                            // starts a new object (vertex numbering resets)
                            // vertex_index_offset = vertex_list.len();
                        }
                        _ => {
                            //log!("unreadable line: start with: {}", char);
                        }
                    }
                }
                None => (),
            },
            None => (),
        }
        buf.clear();
    }

    // log!("{:?}", vertex_position_list);
    // log!("{:?}", triangle_list);

    // flatten vertex data from polygon paths into a single list
    let flat_triangle_vertex_indexes: Vec<u32> = triangle_list.into_iter().flatten().collect();
    log!(
        "{:?}\nlen: {}",
        flat_triangle_vertex_indexes,
        flat_triangle_vertex_indexes.len()
    );

    let flat_vertex_coordinates: Vec<f32> = vertex_position_list.into_iter().flatten().collect();
    log!(
        "{:?}\nlen: {}",
        flat_vertex_coordinates,
        flat_vertex_coordinates.len()
    );

    Ok(ModelData {
        vertices: flat_vertex_coordinates,
        indices: flat_triangle_vertex_indexes,
    })
}
