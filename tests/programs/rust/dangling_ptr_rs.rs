// CWE-416: Dangling Pointer in Rust via unsafe
// A raw pointer is created from a Box, the Box is dropped, and
// the raw pointer is dereferenced -- undefined behavior.
//
// Expected finding: raw pointer dereference after drop (use-after-free)

fn main() {
    let ptr: *mut i32;
    {
        let b = Box::new(42);               // SOURCE: heap allocation
        ptr = Box::into_raw(b);
        unsafe {
            let _ = Box::from_raw(ptr);      // drop / free the allocation
        }
    }
    unsafe {
        let _val = *ptr;                     // SINK: use after free
    }
}
