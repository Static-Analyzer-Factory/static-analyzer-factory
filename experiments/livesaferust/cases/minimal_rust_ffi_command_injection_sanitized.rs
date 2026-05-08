use std::ffi::{c_char, c_int};

extern "C" {
    fn getenv(name: *const c_char) -> *mut c_char;
    fn system(command: *const c_char) -> c_int;
}

static KEY: &[u8] = b"SAF_INPUT\0";

fn sanitize_input(cmd: *mut c_char) -> *mut c_char {
    cmd
}

fn main() {
    unsafe {
        let raw = getenv(KEY.as_ptr().cast());
        let cmd = sanitize_input(raw);
        let _ = system(cmd.cast());
    }
}
