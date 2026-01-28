; ModuleID = 'binary_ops'
source_filename = "binary_ops.c"
target triple = "x86_64-unknown-linux-gnu"

; Test all binary operations

; Integer arithmetic
define i32 @add_i32(i32 %a, i32 %b) {
entry:
  %result = add i32 %a, %b
  ret i32 %result
}

define i32 @sub_i32(i32 %a, i32 %b) {
entry:
  %result = sub i32 %a, %b
  ret i32 %result
}

define i32 @mul_i32(i32 %a, i32 %b) {
entry:
  %result = mul i32 %a, %b
  ret i32 %result
}

define i32 @udiv_i32(i32 %a, i32 %b) {
entry:
  %result = udiv i32 %a, %b
  ret i32 %result
}

define i32 @sdiv_i32(i32 %a, i32 %b) {
entry:
  %result = sdiv i32 %a, %b
  ret i32 %result
}

define i32 @urem_i32(i32 %a, i32 %b) {
entry:
  %result = urem i32 %a, %b
  ret i32 %result
}

define i32 @srem_i32(i32 %a, i32 %b) {
entry:
  %result = srem i32 %a, %b
  ret i32 %result
}

; Floating point arithmetic
define double @fadd_f64(double %a, double %b) {
entry:
  %result = fadd double %a, %b
  ret double %result
}

define double @fsub_f64(double %a, double %b) {
entry:
  %result = fsub double %a, %b
  ret double %result
}

define double @fmul_f64(double %a, double %b) {
entry:
  %result = fmul double %a, %b
  ret double %result
}

define double @fdiv_f64(double %a, double %b) {
entry:
  %result = fdiv double %a, %b
  ret double %result
}

define double @frem_f64(double %a, double %b) {
entry:
  %result = frem double %a, %b
  ret double %result
}

; Bitwise operations
define i32 @and_i32(i32 %a, i32 %b) {
entry:
  %result = and i32 %a, %b
  ret i32 %result
}

define i32 @or_i32(i32 %a, i32 %b) {
entry:
  %result = or i32 %a, %b
  ret i32 %result
}

define i32 @xor_i32(i32 %a, i32 %b) {
entry:
  %result = xor i32 %a, %b
  ret i32 %result
}

define i32 @shl_i32(i32 %a, i32 %b) {
entry:
  %result = shl i32 %a, %b
  ret i32 %result
}

define i32 @lshr_i32(i32 %a, i32 %b) {
entry:
  %result = lshr i32 %a, %b
  ret i32 %result
}

define i32 @ashr_i32(i32 %a, i32 %b) {
entry:
  %result = ashr i32 %a, %b
  ret i32 %result
}

; Integer comparisons
define i1 @icmp_eq(i32 %a, i32 %b) {
entry:
  %result = icmp eq i32 %a, %b
  ret i1 %result
}

define i1 @icmp_ne(i32 %a, i32 %b) {
entry:
  %result = icmp ne i32 %a, %b
  ret i1 %result
}

define i1 @icmp_ugt(i32 %a, i32 %b) {
entry:
  %result = icmp ugt i32 %a, %b
  ret i1 %result
}

define i1 @icmp_uge(i32 %a, i32 %b) {
entry:
  %result = icmp uge i32 %a, %b
  ret i1 %result
}

define i1 @icmp_ult(i32 %a, i32 %b) {
entry:
  %result = icmp ult i32 %a, %b
  ret i1 %result
}

define i1 @icmp_ule(i32 %a, i32 %b) {
entry:
  %result = icmp ule i32 %a, %b
  ret i1 %result
}

define i1 @icmp_sgt(i32 %a, i32 %b) {
entry:
  %result = icmp sgt i32 %a, %b
  ret i1 %result
}

define i1 @icmp_sge(i32 %a, i32 %b) {
entry:
  %result = icmp sge i32 %a, %b
  ret i1 %result
}

define i1 @icmp_slt(i32 %a, i32 %b) {
entry:
  %result = icmp slt i32 %a, %b
  ret i1 %result
}

define i1 @icmp_sle(i32 %a, i32 %b) {
entry:
  %result = icmp sle i32 %a, %b
  ret i1 %result
}

; Floating point comparisons
define i1 @fcmp_oeq(double %a, double %b) {
entry:
  %result = fcmp oeq double %a, %b
  ret i1 %result
}

define i1 @fcmp_one(double %a, double %b) {
entry:
  %result = fcmp one double %a, %b
  ret i1 %result
}

define i1 @fcmp_ogt(double %a, double %b) {
entry:
  %result = fcmp ogt double %a, %b
  ret i1 %result
}

define i1 @fcmp_olt(double %a, double %b) {
entry:
  %result = fcmp olt double %a, %b
  ret i1 %result
}
