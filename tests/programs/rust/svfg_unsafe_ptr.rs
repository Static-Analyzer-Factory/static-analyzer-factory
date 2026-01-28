// SVFG E2E test: Rust raw pointer write + read in unsafe block.
// Tests Rust code path for indirect edges.

extern "C" {
    fn source() -> i32;
    fn sink(v: i32);
}

fn test() {
    unsafe {
        let mut buf: i32 = 0;
        let ptr: *mut i32 = &mut buf;

        let tainted = source();
        *ptr = tainted;     // raw pointer write
        let loaded = *ptr;  // raw pointer read
        sink(loaded);
    }
}

fn main() {
    test();
}
