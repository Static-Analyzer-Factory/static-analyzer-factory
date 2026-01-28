// Abstract interpretation test: Rust unsafe pointer arithmetic
// Tests interval tracking through raw pointer operations.

use std::alloc::{alloc, dealloc, Layout};

unsafe fn fill_buffer(ptr: *mut i32, count: usize) {
    // Safe if count <= allocated size
    let mut i: usize = 0;
    while i < count {
        *ptr.add(i) = (i * 2) as i32;
        i += 1;
    }
}

unsafe fn sum_buffer(ptr: *const i32, count: usize) -> i32 {
    let mut total: i32 = 0;
    let mut i: usize = 0;
    while i < count {
        total = total.wrapping_add(*ptr.add(i));
        i += 1;
    }
    total
}

fn main() {
    let count: usize = 8;
    let layout = Layout::array::<i32>(count).unwrap();

    unsafe {
        let ptr = alloc(layout) as *mut i32;
        if ptr.is_null() {
            return;
        }

        fill_buffer(ptr, count);
        let result = sum_buffer(ptr, count);

        // Use the result to prevent optimization
        if result > 0 {
            *ptr = result;
        }

        dealloc(ptr as *mut u8, layout);
    }
}
