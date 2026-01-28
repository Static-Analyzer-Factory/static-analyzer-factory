; ModuleID = 'select_fp_indirect'
source_filename = "select_fp_indirect.ll"

declare void @foo()
declare void @bar()

define i32 @main(i1 %c) {
entry:
  %f_foo = bitcast void ()* @foo to ptr
  %f_bar = bitcast void ()* @bar to ptr
  %sel = select i1 %c, ptr %f_foo, ptr %f_bar
  %fp = bitcast ptr %sel to void ()*
  call void %fp()
  ret i32 0
}
