// IFDS E2E: Rust unsafe FFI taint propagation
//
// Taint flows through Rust code using C FFI:
//   getenv() → transform() → system()
//
// The extern "C" functions keep their names in LLVM IR (no mangling).
// The Rust function `transform` gets mangled but IFDS tracks taint
// through store/load chains regardless of function name.

extern "C" {
    fn getenv(name: *const i8) -> *mut i8;
    fn system(cmd: *const i8) -> i32;
}

fn transform(input: *mut i8) -> *mut i8 {
    // Pass through — taint should propagate
    input
}

fn main() {
    unsafe {
        let env = getenv(b"USER_CMD\0".as_ptr() as *const i8);
        let processed = transform(env);
        system(processed);
    }
}
