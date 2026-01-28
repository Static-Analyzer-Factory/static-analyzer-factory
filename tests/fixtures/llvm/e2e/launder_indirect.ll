; ModuleID = 'launder_indirect'
source_filename = "launder_indirect.ll"

declare void @foo()
declare ptr @llvm.launder.invariant.group.p0(ptr)

define i32 @main() {
entry:
  %f0 = bitcast void ()* @foo to ptr
  %f1 = call ptr @llvm.launder.invariant.group.p0(ptr %f0)
  %f2 = bitcast ptr %f1 to void ()*
  call void %f2()
  ret i32 0
}
