; ModuleID = 'freeze_indirect'
source_filename = "freeze_indirect.ll"

declare void @foo()

define i32 @main() {
entry:
  %f0 = bitcast void ()* @foo to ptr
  %f1 = freeze ptr %f0
  %f2 = bitcast ptr %f1 to void ()*
  call void %f2()
  ret i32 0
}
