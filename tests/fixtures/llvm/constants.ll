; ModuleID = 'constants'
source_filename = "constants.c"
target triple = "x86_64-unknown-linux-gnu"

; Test constant values of various types

; Integer constants
define i32 @return_int_const() {
entry:
  ret i32 42
}

define i64 @return_large_int_const() {
entry:
  ret i64 9223372036854775807  ; i64 max
}

define i8 @return_small_int_const() {
entry:
  ret i8 -1
}

; Float constants
define float @return_float_const() {
entry:
  ret float 3.140000e+00
}

define double @return_double_const() {
entry:
  ret double 2.718281828459045e+00
}

; Null pointer constant
define ptr @return_null() {
entry:
  ret ptr null
}

; Undef value
define i32 @return_undef() {
entry:
  ret i32 undef
}

; Aggregate constants
%struct.Vec2 = type { i32, i32 }

@const_struct = constant %struct.Vec2 { i32 10, i32 20 }, align 4
@const_array = constant [3 x i32] [i32 1, i32 2, i32 3], align 4
@const_string = private unnamed_addr constant [6 x i8] c"hello\00", align 1

define i32 @use_const_struct() {
entry:
  %ptr = getelementptr %struct.Vec2, ptr @const_struct, i32 0, i32 0
  %val = load i32, ptr %ptr, align 4
  ret i32 %val
}

define i32 @use_const_array() {
entry:
  %ptr = getelementptr [3 x i32], ptr @const_array, i32 0, i32 1
  %val = load i32, ptr %ptr, align 4
  ret i32 %val
}

; Zero initializer
@zero_struct = global %struct.Vec2 zeroinitializer, align 4
@zero_array = global [10 x i32] zeroinitializer, align 4

define i32 @read_zero_init() {
entry:
  %ptr = getelementptr %struct.Vec2, ptr @zero_struct, i32 0, i32 0
  %val = load i32, ptr %ptr, align 4
  ret i32 %val
}

; Constant expressions
define i64 @const_expr_ptr() {
entry:
  ; ptrtoint constant expression
  %addr = ptrtoint ptr @const_string to i64
  ret i64 %addr
}

define ptr @const_expr_gep() {
entry:
  ; getelementptr constant expression
  ret ptr getelementptr ([6 x i8], ptr @const_string, i32 0, i32 0)
}
