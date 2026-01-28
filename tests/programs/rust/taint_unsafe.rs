// CWE-78: Command Injection in Rust via unsafe
// User input from env::args() flows to libc::system() through unsafe block.
//
// Expected finding: env::args() -> libc::system() (taint flow)
use std::env;
use std::ffi::{c_char, CString};

extern "C" {
    fn system(command: *const c_char) -> i32;
}

fn main() {
    let args: Vec<String> = env::args().collect();   // SOURCE
    if args.len() < 2 {
        return;
    }
    let cmd = &args[1];
    unsafe {
        let c_cmd = CString::new(cmd.as_str()).unwrap();
        system(c_cmd.as_ptr());                       // SINK
    }
}
