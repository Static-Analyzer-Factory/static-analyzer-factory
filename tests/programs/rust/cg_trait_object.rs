// CG refinement E2E: Rust trait object dispatch.
// Trait object virtual call resolved via CHA / vtable analysis.
#![allow(unused_unsafe)]

use std::os::raw::c_char;

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(cmd: *const c_char) -> i32;
}

trait Handler {
    fn handle(&self, data: *const c_char);
}

struct UnsafeHandler;

impl Handler for UnsafeHandler {
    fn handle(&self, data: *const c_char) {
        unsafe {
            system(data);
        }
    }
}

fn dispatch(handler: &dyn Handler, data: *const c_char) {
    handler.handle(data); // trait object dispatch
}

fn main() {
    let handler = UnsafeHandler;
    let input = unsafe { getenv(b"CMD\0".as_ptr() as *const c_char) };
    dispatch(&handler, input);
}
