; ModuleID = 'svfg_unsafe_ptr.fe620bc25059f44b-cgu.0'
source_filename = "svfg_unsafe_ptr.fe620bc25059f44b-cgu.0"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "aarch64-unknown-linux-gnu"

@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h462dfff83f1fa54eE", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h666b6e68739e72f2E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h666b6e68739e72f2E" }>, align 8
@alloc_4700d28a9336bc333e190190b9574643 = private unnamed_addr constant <{ [49 x i8] }> <{ [49 x i8] c"/workspace/tests/programs/rust/svfg_unsafe_ptr.rs" }>, align 1
@alloc_55071ec8a3572948e04a867922762d81 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_4700d28a9336bc333e190190b9574643, [16 x i8] c"1\00\00\00\00\00\00\00\0F\00\00\00\09\00\00\00" }>, align 8
@alloc_dd7e8f521cf58d6836779d98c9b06ccd = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_4700d28a9336bc333e190190b9574643, [16 x i8] c"1\00\00\00\00\00\00\00\10\00\00\00\16\00\00\00" }>, align 8

; std::rt::lang_start
; Function Attrs: uwtable
define hidden i64 @_ZN3std2rt10lang_start17h41a211f03919d479E(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #0 {
start:
  %_8 = alloca [8 x i8], align 8
  %_5 = alloca [8 x i8], align 8
  store ptr %main, ptr %_8, align 8
; call std::rt::lang_start_internal
  %0 = call i64 @_ZN3std2rt19lang_start_internal17he1ad9a314bd0157aE(ptr align 1 %_8, ptr align 8 @vtable.0, i64 %argc, ptr %argv, i8 %sigpipe)
  store i64 %0, ptr %_5, align 8
  %v = load i64, ptr %_5, align 8
  ret i64 %v
}

; std::rt::lang_start::{{closure}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h666b6e68739e72f2E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace
  call void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h97ffd6e7cebdd63fE(ptr %_4)
; call <() as std::process::Termination>::report
  %self = call i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hdaec1cb4943b7c51E"()
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; std::sys::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline uwtable
define internal void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h97ffd6e7cebdd63fE(ptr %f) unnamed_addr #2 {
start:
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17h80c9090cd798a432E(ptr %f)
  call void asm sideeffect "", "~{memory}"(), !srcloc !3
  ret void
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h462dfff83f1fa54eE"(ptr %_1) unnamed_addr #1 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call core::ops::function::FnOnce::call_once
  %_0 = call i32 @_ZN4core3ops8function6FnOnce9call_once17h58e47eb98cbdc542E(ptr %0)
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i32 @_ZN4core3ops8function6FnOnce9call_once17h58e47eb98cbdc542E(ptr %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::{{closure}}
  %_0 = invoke i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h666b6e68739e72f2E"(ptr align 8 %_1)
          to label %bb1 unwind label %cleanup

bb3:                                              ; preds = %cleanup
  %2 = load ptr, ptr %1, align 8
  %3 = getelementptr inbounds i8, ptr %1, i64 8
  %4 = load i32, ptr %3, align 8
  %5 = insertvalue { ptr, i32 } poison, ptr %2, 0
  %6 = insertvalue { ptr, i32 } %5, i32 %4, 1
  resume { ptr, i32 } %6

cleanup:                                          ; preds = %start
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %1, align 8
  %10 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb3

bb1:                                              ; preds = %start
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17h80c9090cd798a432E(ptr %_1) unnamed_addr #1 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; core::ptr::drop_in_place<std::rt::lang_start<()>::{{closure}}>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17h69c124ef75cee825E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  ret void
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hdaec1cb4943b7c51E"() unnamed_addr #1 {
start:
  ret i8 0
}

; svfg_unsafe_ptr::test
; Function Attrs: uwtable
define internal void @_ZN15svfg_unsafe_ptr4test17h725ad8241aa3946dE() unnamed_addr #0 {
start:
  %buf = alloca [4 x i8], align 4
  store i32 0, ptr %buf, align 4
  %tainted = call i32 @source() #6
  %_14 = ptrtoint ptr %buf to i64
  %_17 = and i64 %_14, 3
  %_18 = icmp eq i64 %_17, 0
  br i1 %_18, label %bb4, label %panic

bb4:                                              ; preds = %start
  store i32 %tainted, ptr %buf, align 4
  %_8 = ptrtoint ptr %buf to i64
  %_11 = and i64 %_8, 3
  %_12 = icmp eq i64 %_11, 0
  br i1 %_12, label %bb3, label %panic1

panic:                                            ; preds = %start
; call core::panicking::panic_misaligned_pointer_dereference
  call void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64 4, i64 %_14, ptr align 8 @alloc_55071ec8a3572948e04a867922762d81) #7
  unreachable

bb3:                                              ; preds = %bb4
  %loaded = load i32, ptr %buf, align 4
  call void @sink(i32 %loaded) #6
  ret void

panic1:                                           ; preds = %bb4
; call core::panicking::panic_misaligned_pointer_dereference
  call void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64 4, i64 %_8, ptr align 8 @alloc_dd7e8f521cf58d6836779d98c9b06ccd) #7
  unreachable
}

; svfg_unsafe_ptr::main
; Function Attrs: uwtable
define internal void @_ZN15svfg_unsafe_ptr4main17h0528c0f5f9865d10E() unnamed_addr #0 {
start:
; call svfg_unsafe_ptr::test
  call void @_ZN15svfg_unsafe_ptr4test17h725ad8241aa3946dE()
  ret void
}

; std::rt::lang_start_internal
; Function Attrs: uwtable
declare i64 @_ZN3std2rt19lang_start_internal17he1ad9a314bd0157aE(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #0

; Function Attrs: nounwind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #3

; Function Attrs: nounwind uwtable
declare i32 @source() unnamed_addr #3

; core::panicking::panic_misaligned_pointer_dereference
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64, i64, ptr align 8) unnamed_addr #4

; Function Attrs: nounwind uwtable
declare void @sink(i32) unnamed_addr #3

define i32 @main(i32 %0, ptr %1) unnamed_addr #5 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start
  %3 = call i64 @_ZN3std2rt10lang_start17h41a211f03919d479E(ptr @_ZN15svfg_unsafe_ptr4main17h0528c0f5f9865d10E, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #1 = { inlinehint uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #2 = { noinline uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #3 = { nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #4 = { cold minsize noinline noreturn nounwind optsize uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #5 = { "target-cpu"="generic" }
attributes #6 = { nounwind }
attributes #7 = { noreturn nounwind }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{!"rustc version 1.85.0 (4d91de4e4 2025-02-17)"}
!3 = !{i64 5198362128316351}
