; LLVM 22 introduced `ptrtoaddr` — returns the numeric address of a pointer
; without capturing provenance (unlike `ptrtoint`). SAF must at minimum parse
; this instruction without crashing; ideally it's classified like a cast.
source_filename = "ptrtoaddr.ll"
target triple = "x86_64-unknown-linux-gnu"

@g = external global i32

define i64 @addr_of_global() {
entry:
  %a = ptrtoaddr ptr @g to i64
  ret i64 %a
}

define i64 @addr_of_local() {
entry:
  %slot = alloca i32
  %a = ptrtoaddr ptr %slot to i64
  ret i64 %a
}
