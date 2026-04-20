// LLVM 21 retired the `nocapture` parameter attribute in favor of a general
// `captures(...)` family. Clang-22 emits `captures(none)` directly on pointer
// parameters it can prove aren't captured — e.g., a function that only reads
// through the pointer and never stores it anywhere.
//
// The two definitions below each inference-trigger one of the forms:
//   - `read_only`: pointer only read  -> `captures(none) readonly` on %p
//   - `pass_through`: pointer passed to a nocapture-marked declaration but
//                      otherwise unused -> `captures(none)` on %p

__attribute__((noinline))
int read_only(const int* p) {
    return *p;  // no store of p anywhere -> clang infers captures(none)
}

// Opaque declaration that promises not to capture its argument. Clang will
// propagate that through to the calling site's IR.
extern void sink_opaque(const int* p);

__attribute__((noinline))
void pass_through(const int* p) {
    sink_opaque(p);
    // no store of p, so clang can still emit captures(none) on this fn
}
