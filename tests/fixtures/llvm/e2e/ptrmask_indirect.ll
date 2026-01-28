; ModuleID = 'ptrmask_indirect'
source_filename = "ptrmask_indirect.ll"

declare void @foo()
declare ptr @llvm.ptrmask.p0(ptr, i64)

define i32 @main() {
entry:
  %f0 = bitcast void ()* @foo to ptr
  %f1 = call ptr @llvm.ptrmask.p0(ptr %f0, i64 -8)
  %f2 = bitcast ptr %f1 to void ()*
  call void %f2()
  ret i32 0
}
