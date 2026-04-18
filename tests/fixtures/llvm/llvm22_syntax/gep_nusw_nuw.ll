; LLVM 19+ added `nusw` and `nuw` no-wrap flags to `getelementptr`.
; Semantically the GEP computes the same address; SAF should treat these
; like plain `getelementptr inbounds` — FieldPath descent is identical.
source_filename = "gep_nusw_nuw.ll"
target triple = "x86_64-unknown-linux-gnu"

@arr = external global [16 x i32]

define ptr @gep_nusw(i64 %i) {
entry:
  %p1 = getelementptr nusw i32, ptr @arr, i64 %i
  ret ptr %p1
}

define ptr @gep_nuw(i64 %i) {
entry:
  %p2 = getelementptr nuw i32, ptr @arr, i64 %i
  ret ptr %p2
}

define ptr @gep_nusw_nuw(i64 %i) {
entry:
  %p3 = getelementptr nusw nuw i32, ptr @arr, i64 %i
  ret ptr %p3
}

define ptr @gep_nusw_inbounds(i64 %i) {
entry:
  %p4 = getelementptr inbounds nusw i32, ptr @arr, i64 %i
  ret ptr %p4
}
