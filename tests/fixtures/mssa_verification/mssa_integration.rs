// Phase 7: MSSA/SVFG Integration Tests (Rust)
// Tests: Unsafe pointer operations, raw pointer store/load

#![allow(dead_code)]

extern "C" {
    fn sink(x: i32);
}

// Test: simple unsafe store/load
unsafe fn simple_store_load() {
    let mut x: i32 = 0;
    let p: *mut i32 = &mut x;

    *p = 42;  // Store through raw pointer
    let r = *p;  // Load through raw pointer

    sink(r);
}

// Test: multiple stores through same pointer
unsafe fn multiple_stores() {
    let mut x: i32 = 0;
    let p: *mut i32 = &mut x;

    *p = 1;
    *p = 2;
    *p = 3;  // Last store is the clobber

    let r = *p;
    sink(r);
}

// Test: separate allocations, no alias
unsafe fn no_alias_test() {
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    let pa: *mut i32 = &mut a;
    let pb: *mut i32 = &mut b;

    *pa = 10;
    *pb = 20;

    let ra = *pa;  // From store to a
    let rb = *pb;  // From store to b

    sink(ra);
    sink(rb);
}

// Test: conditional store
unsafe fn conditional_store(cond: bool) {
    let mut x: i32 = 0;
    let p: *mut i32 = &mut x;

    if cond {
        *p = 100;
    } else {
        *p = 200;
    }

    // Memory phi expected
    let r = *p;
    sink(r);
}

// Test: loop with store
unsafe fn loop_store(n: i32) {
    let mut counter: i32 = 0;
    let p: *mut i32 = &mut counter;

    for i in 0..n {
        *p = *p + 1;
    }

    let r = *p;
    sink(r);
}

// Test: pointer from slice
unsafe fn slice_pointer() {
    let mut arr = [0i32; 4];
    let p: *mut i32 = arr.as_mut_ptr();

    *p = 111;  // arr[0]
    *p.add(1) = 222;  // arr[1]

    let r0 = *p;
    let r1 = *p.add(1);

    sink(r0);
    sink(r1);
}

#[no_mangle]
pub unsafe extern "C" fn rust_mssa_test() {
    simple_store_load();
    multiple_stores();
    no_alias_test();
    conditional_store(true);
    loop_store(5);
    slice_pointer();
}

fn main() {
    unsafe {
        rust_mssa_test();
    }
}
