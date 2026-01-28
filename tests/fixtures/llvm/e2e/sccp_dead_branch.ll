; Hand-written fixture: SCCP can prove %x is always 42, so the false branch is dead.
source_filename = "sccp_dead_branch.ll"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

define i32 @sccp_test() {
entry:
  %x = add i32 0, 42
  %cmp = icmp eq i32 %x, 42
  br i1 %cmp, label %then, label %else

then:
  ret i32 1

else:
  ; SCCP should detect this as dead
  ret i32 0
}
