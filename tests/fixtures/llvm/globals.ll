; ModuleID = 'globals'
source_filename = "globals.c"
target triple = "x86_64-unknown-linux-gnu"

; Test global variables and constants

; Named global variable
@global_int = global i32 42, align 4

; Named global constant
@global_const = constant i32 100, align 4

; Global string constant
@global_str = private unnamed_addr constant [14 x i8] c"Hello, World!\00", align 1

; Unnamed global (common pattern)
@0 = private unnamed_addr constant [5 x i8] c"test\00", align 1

; Zero-initialized global
@zero_init = global i32 0, align 4

; Global struct
%struct.Config = type { i32, ptr }
@config = global %struct.Config { i32 1, ptr @global_str }, align 8

; Access global variable
define i32 @read_global() {
entry:
  %val = load i32, ptr @global_int, align 4
  ret i32 %val
}

; Modify global variable
define void @write_global(i32 %val) {
entry:
  store i32 %val, ptr @global_int, align 4
  ret void
}

; Read global constant
define i32 @read_constant() {
entry:
  %val = load i32, ptr @global_const, align 4
  ret i32 %val
}

; Get pointer to global string
define ptr @get_string() {
entry:
  ret ptr @global_str
}
