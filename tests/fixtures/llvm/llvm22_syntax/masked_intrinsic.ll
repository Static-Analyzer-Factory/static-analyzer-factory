; LLVM 22 removed the alignment argument from masked load/store/gather/scatter
; intrinsics — alignment is now carried via the `align` attribute on the pointer
; operand. SAF's intrinsic classifier should handle the new shape.
source_filename = "masked_intrinsic.ll"
target triple = "x86_64-unknown-linux-gnu"

declare <4 x float> @llvm.masked.load.v4f32.p0(ptr, <4 x i1>, <4 x float>)
declare void @llvm.masked.store.v4f32.p0(<4 x float>, ptr, <4 x i1>)

define <4 x float> @masked_load(ptr align 16 %p, <4 x i1> %mask, <4 x float> %passthru) {
entry:
  %v = call <4 x float> @llvm.masked.load.v4f32.p0(ptr align 16 %p, <4 x i1> %mask, <4 x float> %passthru)
  ret <4 x float> %v
}

define void @masked_store(<4 x float> %v, ptr align 16 %p, <4 x i1> %mask) {
entry:
  call void @llvm.masked.store.v4f32.p0(<4 x float> %v, ptr align 16 %p, <4 x i1> %mask)
  ret void
}
