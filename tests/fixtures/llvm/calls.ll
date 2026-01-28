; ModuleID = 'calls'
source_filename = "calls.c"
target triple = "x86_64-unknown-linux-gnu"

; Test direct calls, indirect calls, and invoke (flattened to call)

; External function declarations
declare i32 @external_func(i32)
declare void @external_void()

; Helper function
define i32 @helper(i32 %x) {
entry:
  %result = add i32 %x, 1
  ret i32 %result
}

; Direct call to internal function
define i32 @direct_internal_call(i32 %n) {
entry:
  %result = call i32 @helper(i32 %n)
  ret i32 %result
}

; Direct call to external function
define i32 @direct_external_call(i32 %n) {
entry:
  %result = call i32 @external_func(i32 %n)
  ret i32 %result
}

; Indirect call through function pointer
define i32 @indirect_call(ptr %func_ptr, i32 %arg) {
entry:
  %result = call i32 %func_ptr(i32 %arg)
  ret i32 %result
}

; Multiple arguments
define i32 @multi_arg_call() {
entry:
  call void @external_void()
  %r1 = call i32 @helper(i32 10)
  %r2 = call i32 @helper(i32 %r1)
  ret i32 %r2
}

; Call chain
define i32 @call_chain(i32 %n) {
entry:
  %step1 = call i32 @helper(i32 %n)
  %step2 = call i32 @helper(i32 %step1)
  %step3 = call i32 @helper(i32 %step2)
  ret i32 %step3
}

; Recursive call
define i32 @factorial(i32 %n) {
entry:
  %cmp = icmp sle i32 %n, 1
  br i1 %cmp, label %base, label %recurse

base:
  ret i32 1

recurse:
  %n_minus_1 = sub i32 %n, 1
  %sub_result = call i32 @factorial(i32 %n_minus_1)
  %result = mul i32 %n, %sub_result
  ret i32 %result
}
