; ModuleID = 'control_flow'
source_filename = "control_flow.c"
target triple = "x86_64-unknown-linux-gnu"

; Test branch, conditional branch, phi, switch, and select

define i32 @unconditional_branch() {
entry:
  br label %target

target:
  ret i32 0
}

define i32 @conditional_branch(i1 %cond) {
entry:
  br i1 %cond, label %then, label %else

then:
  br label %merge

else:
  br label %merge

merge:
  %result = phi i32 [ 1, %then ], [ 0, %else ]
  ret i32 %result
}

define i32 @phi_node(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 0
  br i1 %cmp, label %positive, label %negative

positive:
  %val1 = add i32 %n, 10
  br label %merge

negative:
  %val2 = sub i32 0, %n
  br label %merge

merge:
  %result = phi i32 [ %val1, %positive ], [ %val2, %negative ]
  ret i32 %result
}

define i32 @switch_stmt(i32 %val) {
entry:
  switch i32 %val, label %default [
    i32 0, label %case0
    i32 1, label %case1
    i32 2, label %case2
  ]

case0:
  ret i32 100

case1:
  ret i32 200

case2:
  ret i32 300

default:
  ret i32 -1
}

define i32 @select_expr(i1 %cond, i32 %a, i32 %b) {
entry:
  %result = select i1 %cond, i32 %a, i32 %b
  ret i32 %result
}

define i32 @loop_example(i32 %n) {
entry:
  br label %loop

loop:
  %i = phi i32 [ 0, %entry ], [ %next_i, %loop ]
  %sum = phi i32 [ 0, %entry ], [ %next_sum, %loop ]
  %next_sum = add i32 %sum, %i
  %next_i = add i32 %i, 1
  %cond = icmp slt i32 %next_i, %n
  br i1 %cond, label %loop, label %exit

exit:
  ret i32 %sum
}
