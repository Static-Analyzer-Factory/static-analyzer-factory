; LLVM 21 replaced `nocapture` with `captures(none)` on pointer parameters.
; The text reader upgrades `nocapture` -> `captures(none)` automatically.
; SAF's parameter-attribute extraction should accept both forms.
source_filename = "captures_attr.ll"
target triple = "x86_64-unknown-linux-gnu"

declare void @sink(ptr captures(none) %p)
declare void @sink_readonly(ptr readonly captures(none) %p)
declare void @sink_partial(ptr captures(address, provenance) %p)

define void @caller(ptr %x) {
entry:
  call void @sink(ptr captures(none) %x)
  call void @sink_readonly(ptr readonly captures(none) %x)
  ret void
}
