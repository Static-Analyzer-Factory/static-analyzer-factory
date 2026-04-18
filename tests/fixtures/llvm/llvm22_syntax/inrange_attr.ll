; LLVM 20+ moved the `inrange` GEP qualifier from inline (e.g. `inrange i32 0`)
; to a GEP attribute position: `inrange(low, high)`. SAF's constant-GEP
; decomposer previously stripped the inline `inrange` by accident and treated
; every parsed index as a field step — on the new syntax both bugs stopped
; cancelling and PTA walked off the type. This fixture reproduces the vtable-
; shape GEP whose decomposition must yield FieldPath [Field{0}, Field{2}].
source_filename = "inrange_attr.ll"
target triple = "x86_64-unknown-linux-gnu"

; Fake vtable: struct wrapping [3 x ptr]. An aggregate initializer is required
; for decompose_constant_gep to synthesize an explicit Gep — external globals
; with no initializer flow through the simpler "base-address" path.
@null_fn = external global i8
@_ZTV1A = unnamed_addr constant { [3 x ptr] }
  { [3 x ptr] [ptr null, ptr null, ptr @null_fn] }

define void @install_vtable(ptr %this) {
entry:
  ; Store slot 2 of the vtable into *this — exactly the shape clang emits in a
  ; C++ constructor when installing the vtable pointer. LLVM 22 syntax:
  ;   getelementptr inbounds inrange(-16, 8) ({ [3 x ptr] }, ptr @_ZTV1A, i32 0, i32 0, i32 2)
  store ptr getelementptr inbounds inrange(-16, 8) ({ [3 x ptr] }, ptr @_ZTV1A, i32 0, i32 0, i32 2),
        ptr %this, align 8
  ret void
}
