use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_ulonglong};
use std::panic;
use std::path::Path;
use std::sync::Once;

use tracing::{error, info};

use crate::parse;

static mut VERSION: *const c_char = std::ptr::null();
static INIT: Once = Once::new();

#[repr(C)]
pub struct ParsedModelData {
    vertices_ptr: *const c_float,
    vertices_len: usize,
    uvs_ptr: *const c_float,
    uvs_len: usize,
    faces_ptr: *const c_ulonglong,
    faces_len: usize,
    vertices: Vec<f32>,
    uvs: Vec<f32>,
    faces: Vec<u64>,
}

#[no_mangle]
pub extern "C" fn ffi_version() -> *const c_char {
    unsafe {
        INIT.call_once(|| {
            let version = env!("CARGO_PKG_VERSION");
            let version = CString::new(version).unwrap();
            VERSION = version.into_raw();
        });
        VERSION
    }
}

#[no_mangle]
pub extern "C" fn ffi_parse(xml_file_path: *const c_char) -> *mut ParsedModelData {
    let c_str = unsafe { CStr::from_ptr(xml_file_path) };
    let xml_file = Path::new(c_str.to_str().unwrap());
    let result = panic::catch_unwind(|| {
        let (vertices, uvs, faces) = parse(xml_file);

        let vertices_flat: Vec<f32> = vertices.into_iter().flatten().collect();
        let uvs_flat: Vec<f32> = uvs.into_iter().flatten().collect();
        let faces_flat: Vec<u64> = faces.into_iter().flatten().collect();

        info!(
            "Packing {} vertices, {} uvs, and {} faces",
            vertices_flat.len(),
            uvs_flat.len(),
            faces_flat.len()
        );

        let result = Box::new(ParsedModelData {
            vertices_ptr: vertices_flat.as_ptr() as *const c_float,
            vertices_len: vertices_flat.len(),
            uvs_ptr: uvs_flat.as_ptr() as *const c_float,
            uvs_len: uvs_flat.len(),
            faces_ptr: faces_flat.as_ptr() as *const c_ulonglong,
            faces_len: faces_flat.len(),
            vertices: vertices_flat,
            uvs: uvs_flat,
            faces: faces_flat,
        });

        Box::into_raw(result)
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            error!("Failed to parse model data for {:?}", xml_file);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn ffi_free(result: *mut ParsedModelData) {
    if !result.is_null() {
        unsafe {
            drop(Box::from_raw(result));
        }
    }
}
