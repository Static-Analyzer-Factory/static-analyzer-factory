// mssa_rust_unsafe.rs — Rust unsafe pointer operations in Memory SSA.
//
// Same pattern as mssa_store_load_simple.c but compiled from Rust.
// Validates that Memory SSA handles Rust-generated LLVM IR correctly.

extern "C" {
    fn source() -> i32;
    fn sink(x: i32);
}

unsafe fn test() {
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    let p: *mut i32 = &mut a;
    let q: *mut i32 = &mut b;
    *p = source();   // S1
    *q = 99;         // S2
    let x = *p;      // L1: clobber should be S1
    sink(x);
}

fn main() {
    unsafe { test(); }
}
