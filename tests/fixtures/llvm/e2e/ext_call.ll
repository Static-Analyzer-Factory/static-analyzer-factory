; ModuleID = 'ext_call'
source_filename = "ext_call.ll"

declare void @foo()

define i32 @main() {
entry:
  call void @foo()
  ret i32 0
}
