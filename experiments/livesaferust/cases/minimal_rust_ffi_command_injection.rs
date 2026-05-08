use std::ffi::{c_char, c_int};

extern "C" {
    fn getenv(name: *const c_char) -> *mut c_char;
    fn system(command: *const c_char) -> c_int;
}

static KEY: &[u8] = b"SAF_INPUT\0";

fn main() {
    unsafe {
        let cmd = getenv(KEY.as_ptr().cast());
        let _ = system(cmd.cast());
    }
}
