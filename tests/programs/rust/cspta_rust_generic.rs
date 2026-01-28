/// cspta_rust_generic.rs — Test CS-PTA with Rust unsafe wrapper functions.
///
/// wrap_alloc() is an unsafe wrapper called from two different sites.
/// CS-PTA should distinguish the two allocation contexts.

use std::alloc::{Layout, alloc, dealloc};

unsafe fn wrap_alloc(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align_unchecked(size, 8);
    alloc(layout)
}

unsafe fn wrap_dealloc(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align_unchecked(size, 8);
    dealloc(ptr, layout);
}

fn main() {
    unsafe {
        let a = wrap_alloc(32);   // site 1
        let b = wrap_alloc(64);   // site 2

        // With CS-PTA: a and b point to distinct allocations
        *a = 1;
        *b = 2;

        wrap_dealloc(a, 32);
        wrap_dealloc(b, 64);
    }
}
