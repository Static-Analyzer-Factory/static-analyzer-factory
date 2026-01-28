// fspta_rust_unsafe.rs — Rust unsafe raw pointer sequential writes
//
// Sequential stores through a raw pointer should have the later store
// kill the earlier one via flow-sensitive strong update.

static mut A_VAL: i32 = 0;
static mut B_VAL: i32 = 0;

unsafe fn test_sequential_writes() {
    let p: *mut i32 = &raw mut A_VAL;
    *p = 10;
    let p: *mut i32 = &raw mut B_VAL;
    *p = 20;
    // After: p -> {B_VAL} (flow-sensitive)
    //        p -> {A_VAL, B_VAL} (Andersen CI)
}

fn main() {
    unsafe {
        test_sequential_writes();
    }
}
