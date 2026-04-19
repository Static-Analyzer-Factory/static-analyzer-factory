// LLVM 20+ moved the GEP `inrange` qualifier from an inline index attribute
// (`inrange i32 N`) to a GEP-level attribute (`inrange(low, high)`). Clang-22
// emits the new syntax whenever a C++ constructor installs a vtable pointer,
// since the GEP into a vtable global is always in-range.
//
// This source was chosen to produce the shape `getelementptr inbounds
// inrange(-16, 8) ({ [N x ptr] }, ptr @_ZTV..., i32 0, i32 0, i32 K)`
// in `A::A()`'s prologue — the bug surfaced on PTABen's
// `basic_cpp_tests/virtual-inheritance-2.ll` reduces to exactly this pattern.

class A {
 public:
  virtual void f() {}
};

// Force an out-of-line definition so the constructor lowers into explicit IR
// that installs the vtable pointer (avoiding clang folding it away).
A make_a() { return A(); }
