; ModuleID = 'memory_ops'
source_filename = "memory_ops.c"
target triple = "x86_64-unknown-linux-gnu"

; Test alloca, load, store, and gep operations

%struct.Point = type { i32, i32 }

define i32 @alloca_load_store() {
entry:
  ; Stack allocation
  %x = alloca i32, align 4

  ; Store a value
  store i32 10, ptr %x, align 4

  ; Load the value back
  %val = load i32, ptr %x, align 4

  ret i32 %val
}

define i32 @struct_access() {
entry:
  ; Allocate a struct
  %p = alloca %struct.Point, align 4

  ; Store to first field (x)
  %x_ptr = getelementptr inbounds %struct.Point, ptr %p, i32 0, i32 0
  store i32 5, ptr %x_ptr, align 4

  ; Store to second field (y)
  %y_ptr = getelementptr inbounds %struct.Point, ptr %p, i32 0, i32 1
  store i32 10, ptr %y_ptr, align 4

  ; Load x
  %x_val = load i32, ptr %x_ptr, align 4

  ret i32 %x_val
}

define i32 @array_access(i32 %idx) {
entry:
  ; Allocate an array
  %arr = alloca [10 x i32], align 4

  ; Store to element 0
  %elem0 = getelementptr inbounds [10 x i32], ptr %arr, i32 0, i32 0
  store i32 100, ptr %elem0, align 4

  ; Dynamic array access
  %elem_dyn = getelementptr inbounds [10 x i32], ptr %arr, i32 0, i32 %idx
  %val = load i32, ptr %elem_dyn, align 4

  ret i32 %val
}
