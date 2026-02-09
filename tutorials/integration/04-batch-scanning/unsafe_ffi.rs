// CWE-78: Unsafe FFI command injection (Rust)
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(cmd: *const c_char) -> i32;
}

fn main() {
    let key = CString::new("USER_CMD").unwrap();
    unsafe {
        let val = getenv(key.as_ptr());
        if !val.is_null() {
            system(val);
        }
    }
}
