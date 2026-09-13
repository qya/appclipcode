use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::slice;

use appclipcode::{
    generate, generate_with_template, read_svg,
    templates, CodeType, Options,
};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

fn set_error(err: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(err).ok();
    });
}

#[no_mangle]
pub extern "C" fn appclip_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        Some(cs) => cs.as_ptr(),
        None => std::ptr::null(),
    })
}

#[no_mangle]
pub extern "C" fn appclip_alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn appclip_dealloc(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, size);
        }
    }
}

#[no_mangle]
pub extern "C" fn appclip_free_buffer(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let len_ptr = ptr as *const u32;
            let len = (*len_ptr) as usize;
            let total_size = len + 4;
            let _ = Vec::from_raw_parts(ptr, total_size, total_size);
        }
    }
}

fn return_buffer(bytes: &[u8]) -> *mut u8 {
    let total_size = bytes.len() + 4;
    let mut buf = Vec::with_capacity(total_size);
    let len = bytes.len() as u32;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

unsafe fn parse_str<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    if ptr.is_null() || len == 0 {
        return Ok("");
    }
    let slice = slice::from_raw_parts(ptr, len);
    std::str::from_utf8(slice).map_err(|e| format!("invalid UTF-8: {}", e))
}

#[no_mangle]
pub unsafe extern "C" fn appclip_generate_svg(
    url_ptr: *const u8,
    url_len: usize,
    template_index: i32,
    code_type_val: i32,
) -> *mut u8 {
    let url = match parse_str(url_ptr, url_len) {
        Ok(u) => u,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    let code_type = if code_type_val == 1 {
        CodeType::NFC
    } else {
        CodeType::Camera
    };
    let opts = Some(Options { code_type });

    match generate_with_template(url, template_index.max(0) as usize, opts) {
        Ok(svg) => return_buffer(svg.as_bytes()),
        Err(e) => {
            set_error(e);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn appclip_generate_custom_svg(
    url_ptr: *const u8,
    url_len: usize,
    fg_ptr: *const u8,
    fg_len: usize,
    bg_ptr: *const u8,
    bg_len: usize,
    code_type_val: i32,
) -> *mut u8 {
    let url = match parse_str(url_ptr, url_len) {
        Ok(u) => u,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };
    let fg = match parse_str(fg_ptr, fg_len) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };
    let bg = match parse_str(bg_ptr, bg_len) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    let code_type = if code_type_val == 1 {
        CodeType::NFC
    } else {
        CodeType::Camera
    };
    let opts = Some(Options { code_type });

    match generate(url, fg, bg, opts) {
        Ok(svg) => return_buffer(svg.as_bytes()),
        Err(e) => {
            set_error(e);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn appclip_decode_svg(svg_ptr: *const u8, svg_len: usize) -> *mut u8 {
    let svg = match parse_str(svg_ptr, svg_len) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    match read_svg(svg) {
        Ok(url) => return_buffer(url.as_bytes()),
        Err(e) => {
            set_error(e);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn appclip_get_templates_json() -> *mut u8 {
    let tmpls = templates();
    let mut json = String::from("[");
    for (i, t) in tmpls.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"index":{},"foreground":"{}","background":"{}","third":"{}"}}"#,
            t.index,
            t.foreground.hex(),
            t.background.hex(),
            t.third.hex()
        ));
    }
    json.push(']');
    return_buffer(json.as_bytes())
}
