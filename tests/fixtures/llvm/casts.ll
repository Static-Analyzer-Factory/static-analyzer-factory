; ModuleID = 'casts'
source_filename = "casts.c"
target triple = "x86_64-unknown-linux-gnu"

; Test all cast operations

; Integer truncation
define i16 @trunc_i32_to_i16(i32 %x) {
entry:
  %result = trunc i32 %x to i16
  ret i16 %result
}

; Zero extension
define i64 @zext_i32_to_i64(i32 %x) {
entry:
  %result = zext i32 %x to i64
  ret i64 %result
}

; Sign extension
define i64 @sext_i32_to_i64(i32 %x) {
entry:
  %result = sext i32 %x to i64
  ret i64 %result
}

; Float to unsigned int
define i32 @fptoui_f64_to_i32(double %x) {
entry:
  %result = fptoui double %x to i32
  ret i32 %result
}

; Float to signed int
define i32 @fptosi_f64_to_i32(double %x) {
entry:
  %result = fptosi double %x to i32
  ret i32 %result
}

; Unsigned int to float
define double @uitofp_i32_to_f64(i32 %x) {
entry:
  %result = uitofp i32 %x to double
  ret double %result
}

; Signed int to float
define double @sitofp_i32_to_f64(i32 %x) {
entry:
  %result = sitofp i32 %x to double
  ret double %result
}

; Float truncation
define float @fptrunc_f64_to_f32(double %x) {
entry:
  %result = fptrunc double %x to float
  ret float %result
}

; Float extension
define double @fpext_f32_to_f64(float %x) {
entry:
  %result = fpext float %x to double
  ret double %result
}

; Pointer to int
define i64 @ptrtoint_ptr_to_i64(ptr %p) {
entry:
  %result = ptrtoint ptr %p to i64
  ret i64 %result
}

; Int to pointer
define ptr @inttoptr_i64_to_ptr(i64 %x) {
entry:
  %result = inttoptr i64 %x to ptr
  ret ptr %result
}

; Bitcast (reinterpret bits)
define i64 @bitcast_f64_to_i64(double %x) {
entry:
  %result = bitcast double %x to i64
  ret i64 %result
}

; Multiple casts in chain
define i64 @cast_chain(i8 %x) {
entry:
  %ext16 = zext i8 %x to i16
  %ext32 = zext i16 %ext16 to i32
  %ext64 = zext i32 %ext32 to i64
  ret i64 %ext64
}
