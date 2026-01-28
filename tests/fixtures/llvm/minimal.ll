; ModuleID = 'minimal'
source_filename = "minimal.c"
target triple = "x86_64-unknown-linux-gnu"

; Minimal function that returns void
define void @minimal_func() {
entry:
  ret void
}

; Another simple function that returns an integer
define i32 @return_42() {
entry:
  ret i32 42
}
