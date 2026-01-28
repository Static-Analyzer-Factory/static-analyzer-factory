; Hand-written fixture: branch on parameter, both paths reachable.
; The partition mechanism can split at the branch on %n.
source_filename = "partition_const_branch.ll"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

define i32 @partition_test(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 0
  br i1 %cmp, label %then, label %else

then:
  %a = add i32 %n, 1
  br label %merge

else:
  %b = sub i32 %n, 1
  br label %merge

merge:
  %result = phi i32 [ %a, %then ], [ %b, %else ]
  ret i32 %result
}
