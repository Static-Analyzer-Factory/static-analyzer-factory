; Test fixture: strlen called on a function parameter (not a local alloca).
; Used to verify that resolve_computed_bound gracefully falls back to TOP
; when the argument cannot be traced to an allocation.

declare i64 @strlen(ptr)

define i64 @test_strlen_param(ptr %buf) {
entry:
  %len = call i64 @strlen(ptr %buf)
  ret i64 %len
}
