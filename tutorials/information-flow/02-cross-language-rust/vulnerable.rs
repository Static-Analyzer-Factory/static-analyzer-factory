// CWE-78: Command Injection in Rust via unsafe FFI
//
// This program demonstrates that even memory-safe languages like Rust
// can have injection vulnerabilities when using unsafe FFI. The program
// reads an environment variable via getenv() and passes it directly
// to libc's system() — a classic command injection pattern.
//
// SAF detects this cross-language taint flow: the C library source
// (getenv return value) flows to the C library sink (system argument),
// even though the glue code is written in Rust. This works because
// SAF operates on LLVM IR, which is language-agnostic.

use std::ffi::{c_char, c_int, CString};

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(command: *const c_char) -> c_int;
}

fn main() {
    unsafe {
        // SOURCE: getenv() returns user-controlled data from the environment.
        let key = CString::new("USER_CMD").unwrap();
        let cmd = getenv(key.as_ptr());

        // SINK: system() executes the string as a shell command.
        // Rust's borrow checker can't prevent this injection vulnerability
        // because the unsafe block bypasses safety guarantees.
        if !cmd.is_null() {
            system(cmd);
        }
    }
}
