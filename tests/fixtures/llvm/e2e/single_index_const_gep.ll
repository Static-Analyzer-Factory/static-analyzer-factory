; Regression fixture for the single-index constant-expression GEP fallback
; bug in `decompose_constant_gep` / `resolve_constant_gep_element`
; (mapping.rs). A single-index GEP `getelementptr T, ptr @g, i64 K` has no
; type descent — it's purely a pointer-level step. SAF must NOT synthesize
; a Gep instruction for it (which would produce a bogus one-step
; FieldPath). Instead the caller should fall through to simple base-address
; resolution.
;
; The store operand below is such a single-index constant GEP. After the
; fix the mapping must produce exactly one Store and zero Gep instructions
; in `consume_single_gep`.
source_filename = "single_index_const_gep.ll"
target triple = "x86_64-unknown-linux-gnu"

@g = global [4 x i32] [i32 10, i32 20, i32 30, i32 40]

define void @consume_single_gep(ptr %dst) {
entry:
  store ptr getelementptr inbounds ([4 x i32], ptr @g, i64 0), ptr %dst, align 8
  ret void
}
