; ModuleID = 'absint_rust_unsafe.729b7a9133a2b3a1-cgu.0'
source_filename = "absint_rust_unsafe.729b7a9133a2b3a1-cgu.0"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "aarch64-unknown-linux-gnu"

@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h2f9a303dc8c8c1d5E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h400430bdee0fc100E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h400430bdee0fc100E" }>, align 8
@alloc_d4d2a2a8539eafc62756407d946babb3 = private unnamed_addr constant <{ [110 x i8] }> <{ [110 x i8] c"unsafe precondition(s) violated: ptr::read_volatile requires that the pointer argument is aligned and non-null" }>, align 1
@alloc_fad0cd83b7d1858a846a172eb260e593 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"is_aligned_to: align is not a power-of-two" }>, align 1
@alloc_e92e94d0ff530782b571cfd99ec66aef = private unnamed_addr constant <{ ptr, [8 x i8] }> <{ ptr @alloc_fad0cd83b7d1858a846a172eb260e593, [8 x i8] c"*\00\00\00\00\00\00\00" }>, align 8
@0 = private unnamed_addr constant <{ [8 x i8], [8 x i8] }> <{ [8 x i8] zeroinitializer, [8 x i8] undef }>, align 8
@alloc_2d5b6a4803df6aa853303aedba87a5fe = private unnamed_addr constant <{ [81 x i8] }> <{ [81 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/ptr/const_ptr.rs" }>, align 1
@alloc_369fae759a17104aba57fba3370bcbfb = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_2d5b6a4803df6aa853303aedba87a5fe, [16 x i8] c"Q\00\00\00\00\00\00\00\C8\05\00\00\0D\00\00\00" }>, align 8
@alloc_cd1513ae8d1ae22acf9342b8dfa1561d = private unnamed_addr constant <{ [164 x i8] }> <{ [164 x i8] c"unsafe precondition(s) violated: Layout::from_size_align_unchecked requires that align is a power of 2 and the rounded-up allocation size does not exceed isize::MAX" }>, align 1
@alloc_b3bef90a878a4d6e375055f2a4545e5a = private unnamed_addr constant <{ [80 x i8] }> <{ [80 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/alloc/layout.rs" }>, align 1
@alloc_264bf61461bec25857606b12e7f296ec = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_b3bef90a878a4d6e375055f2a4545e5a, [16 x i8] c"P\00\00\00\00\00\00\00\FC\01\00\00)\00\00\00" }>, align 8
@vtable.1 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", ptr @"_ZN69_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$3fmt17hbb0a12f0fb8e59e1E" }>, align 8
@alloc_00ae4b301f7fab8ac9617c03fcbd7274 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@__rust_no_alloc_shim_is_unstable = external global i8
@alloc_477d6fd19596379e27b9bd4358550877 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"LayoutError" }>, align 1
@alloc_da5380d635bb872102d7ea86f6faa867 = private unnamed_addr constant <{ [52 x i8] }> <{ [52 x i8] c"/workspace/tests/programs/rust/absint_rust_unsafe.rs" }>, align 1
@alloc_6e2729e3998b9f7fa255858dc4b6768e = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\0A\00\00\00\17\00\00\00" }>, align 8
@alloc_df786ddc2ceecd18523377e43ddb972c = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\0A\00\00\00\09\00\00\00" }>, align 8
@alloc_c7d09e11100a7625eecbef574109d7ae = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\0B\00\00\00\09\00\00\00" }>, align 8
@alloc_9f5bd7b72be290ba2a5ad7f432b7dbbe = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\13\00\00\00$\00\00\00" }>, align 8
@alloc_3c69dd3f9458a33b68ba82e7fa768a72 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\14\00\00\00\09\00\00\00" }>, align 8
@alloc_e417d40d34051c06476c4fe3f7e17f7a = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00\1B\00\00\00.\00\00\00" }>, align 8
@alloc_af89d28216c7809dc696dcc45c848409 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_da5380d635bb872102d7ea86f6faa867, [16 x i8] c"4\00\00\00\00\00\00\00(\00\00\00\0D\00\00\00" }>, align 8

; std::rt::lang_start
; Function Attrs: uwtable
define hidden i64 @_ZN3std2rt10lang_start17h284b81f5805dd514E(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #0 {
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
define internal i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h400430bdee0fc100E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace
  call void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h11f675cf3353f936E(ptr %_4)
; call <() as std::process::Termination>::report
  %self = call i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17heca3b4a5cc167409E"()
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; std::sys::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline uwtable
define internal void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h11f675cf3353f936E(ptr %f) unnamed_addr #2 {
start:
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17hbce64f16a17901acE(ptr %f)
  call void asm sideeffect "", "~{memory}"(), !srcloc !3
  ret void
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h2f9a303dc8c8c1d5E"(ptr %_1) unnamed_addr #1 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call core::ops::function::FnOnce::call_once
  %_0 = call i32 @_ZN4core3ops8function6FnOnce9call_once17hfb0595f0efd46549E(ptr %0)
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17hbce64f16a17901acE(ptr %_1) unnamed_addr #1 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i32 @_ZN4core3ops8function6FnOnce9call_once17hfb0595f0efd46549E(ptr %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::{{closure}}
  %_0 = invoke i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h400430bdee0fc100E"(ptr align 8 %_1)
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

; core::ptr::read_volatile::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core3ptr13read_volatile18precondition_check17hf68604e7045007d4E(ptr %addr, i64 %align, i1 zeroext %is_zst) unnamed_addr #3 personality ptr @rust_eh_personality {
start:
  %0 = alloca [4 x i8], align 4
  %_8 = alloca [48 x i8], align 8
  %1 = call i64 @llvm.ctpop.i64(i64 %align)
  %2 = trunc i64 %1 to i32
  store i32 %2, ptr %0, align 4
  %_12 = load i32, ptr %0, align 4
  %3 = icmp eq i32 %_12, 1
  br i1 %3, label %bb7, label %bb8

bb7:                                              ; preds = %start
  %_10 = ptrtoint ptr %addr to i64
  %_11 = sub i64 %align, 1
  %_9 = and i64 %_10, %_11
  %4 = icmp eq i64 %_9, 0
  br i1 %4, label %bb3, label %bb4

bb8:                                              ; preds = %start
  store ptr @alloc_e92e94d0ff530782b571cfd99ec66aef, ptr %_8, align 8
  %5 = getelementptr inbounds i8, ptr %_8, i64 8
  store i64 1, ptr %5, align 8
  %6 = load ptr, ptr @0, align 8
  %7 = load i64, ptr getelementptr inbounds (i8, ptr @0, i64 8), align 8
  %8 = getelementptr inbounds i8, ptr %_8, i64 32
  store ptr %6, ptr %8, align 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  store i64 %7, ptr %9, align 8
  %10 = getelementptr inbounds i8, ptr %_8, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %10, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 8
  store i64 0, ptr %11, align 8
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h169d7389ef09f3baE(ptr align 8 %_8, ptr align 8 @alloc_369fae759a17104aba57fba3370bcbfb) #12
          to label %unreachable unwind label %terminate

bb3:                                              ; preds = %bb7
  br i1 %is_zst, label %bb5, label %bb6

bb4:                                              ; preds = %bb7
  br label %bb2

bb6:                                              ; preds = %bb3
  %_6 = icmp eq i64 %_10, 0
  %_4 = xor i1 %_6, true
  br i1 %_4, label %bb1, label %bb2

bb5:                                              ; preds = %bb3
  br label %bb1

bb2:                                              ; preds = %bb4, %bb6
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_d4d2a2a8539eafc62756407d946babb3, i64 110) #13
  unreachable

bb1:                                              ; preds = %bb5, %bb6
  ret void

terminate:                                        ; preds = %bb8
  %12 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %13 = extractvalue { ptr, i32 } %12, 0
  %14 = extractvalue { ptr, i32 } %12, 1
; call core::panicking::panic_cannot_unwind
  call void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() #14
  unreachable

unreachable:                                      ; preds = %bb8
  unreachable
}

; core::ptr::drop_in_place<core::alloc::layout::LayoutError>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr53drop_in_place$LT$core..alloc..layout..LayoutError$GT$17h9bb632d0e6c34fb6E"(ptr align 1 %_1) unnamed_addr #1 {
start:
  ret void
}

; core::ptr::mut_ptr::<impl *mut T>::is_null
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$7is_null17h090bb3162c5a283dE"(ptr %self) unnamed_addr #1 {
start:
  %_3 = ptrtoint ptr %self to i64
  %_0 = icmp eq i64 %_3, 0
  ret i1 %_0
}

; core::ptr::drop_in_place<std::rt::lang_start<()>::{{closure}}>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17h78c118e7fbdbcfe8E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  ret void
}

; core::alloc::layout::Layout::from_size_align_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5alloc6layout6Layout25from_size_align_unchecked18precondition_check17h0af761f156c2c52fE(i64 %size, i64 %align) unnamed_addr #3 personality ptr @rust_eh_personality {
start:
; invoke core::alloc::layout::Layout::is_size_align_valid
  %_3 = invoke zeroext i1 @_ZN4core5alloc6layout6Layout19is_size_align_valid17hba5c1dd91ce45078E(i64 %size, i64 %align)
          to label %bb1 unwind label %terminate

terminate:                                        ; preds = %start
  %0 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %1 = extractvalue { ptr, i32 } %0, 0
  %2 = extractvalue { ptr, i32 } %0, 1
; call core::panicking::panic_cannot_unwind
  call void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() #14
  unreachable

bb1:                                              ; preds = %start
  br i1 %_3, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_cd1513ae8d1ae22acf9342b8dfa1561d, i64 164) #13
  unreachable

bb2:                                              ; preds = %bb1
  ret void
}

; core::alloc::layout::Layout::array
; Function Attrs: inlinehint uwtable
define internal { i64, i64 } @_ZN4core5alloc6layout6Layout5array17h75d3052110da3143E(i64 %n) unnamed_addr #1 {
start:
; call core::alloc::layout::Layout::array::inner
  %0 = call { i64, i64 } @_ZN4core5alloc6layout6Layout5array5inner17h2cc1a6230615bad2E(i64 4, i64 4, i64 %n)
  %_0.0 = extractvalue { i64, i64 } %0, 0
  %_0.1 = extractvalue { i64, i64 } %0, 1
  %1 = insertvalue { i64, i64 } poison, i64 %_0.0, 0
  %2 = insertvalue { i64, i64 } %1, i64 %_0.1, 1
  ret { i64, i64 } %2
}

; core::alloc::layout::Layout::array::inner
; Function Attrs: inlinehint uwtable
define internal { i64, i64 } @_ZN4core5alloc6layout6Layout5array5inner17h2cc1a6230615bad2E(i64 %element_layout.0, i64 %element_layout.1, i64 %n) unnamed_addr #1 {
start:
  %_18 = alloca [8 x i8], align 8
  %_13 = alloca [8 x i8], align 8
  %_0 = alloca [16 x i8], align 8
  %0 = icmp eq i64 %element_layout.1, 0
  br i1 %0, label %bb5, label %bb1

bb5:                                              ; preds = %bb4, %start
  %array_size = mul nuw i64 %element_layout.1, %n
  store i64 %element_layout.0, ptr %_18, align 8
  %_19 = load i64, ptr %_18, align 8
  %_20 = icmp uge i64 %_19, 1
  %_21 = icmp ule i64 %_19, -9223372036854775808
  %_22 = and i1 %_20, %_21
  br label %bb7

bb1:                                              ; preds = %start
  store i64 %element_layout.0, ptr %_13, align 8
  %_14 = load i64, ptr %_13, align 8
  %_15 = icmp uge i64 %_14, 1
  %_16 = icmp ule i64 %_14, -9223372036854775808
  %_17 = and i1 %_15, %_16
  %_7 = sub nuw i64 -9223372036854775808, %_14
  %_8 = icmp eq i64 %element_layout.1, 0
  br i1 %_8, label %panic, label %bb2

bb2:                                              ; preds = %bb1
  %_6 = udiv i64 %_7, %element_layout.1
  %_5 = icmp ugt i64 %n, %_6
  br i1 %_5, label %bb3, label %bb4

panic:                                            ; preds = %bb1
; call core::panicking::panic_const::panic_const_div_by_zero
  call void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17he9029e02cc767683E(ptr align 8 @alloc_264bf61461bec25857606b12e7f296ec) #12
  unreachable

bb4:                                              ; preds = %bb2
  br label %bb5

bb3:                                              ; preds = %bb2
  %1 = load i64, ptr @0, align 8
  %2 = load i64, ptr getelementptr inbounds (i8, ptr @0, i64 8), align 8
  store i64 %1, ptr %_0, align 8
  %3 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %2, ptr %3, align 8
  br label %bb6

bb7:                                              ; preds = %bb5
; call core::alloc::layout::Layout::from_size_align_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout25from_size_align_unchecked18precondition_check17h0af761f156c2c52fE(i64 %array_size, i64 %_19) #15
  br label %bb8

bb8:                                              ; preds = %bb7
  store i64 %_19, ptr %_0, align 8
  %4 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %array_size, ptr %4, align 8
  br label %bb6

bb6:                                              ; preds = %bb3, %bb8
  %5 = load i64, ptr %_0, align 8
  %6 = getelementptr inbounds i8, ptr %_0, i64 8
  %7 = load i64, ptr %6, align 8
  %8 = insertvalue { i64, i64 } poison, i64 %5, 0
  %9 = insertvalue { i64, i64 } %8, i64 %7, 1
  ret { i64, i64 } %9
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17heca3b4a5cc167409E"() unnamed_addr #1 {
start:
  ret i8 0
}

; alloc::alloc::alloc
; Function Attrs: inlinehint uwtable
define internal ptr @_ZN5alloc5alloc5alloc17hcf4c878396f6a8b3E(i64 %0, i64 %1) unnamed_addr #1 {
start:
  %2 = alloca [1 x i8], align 1
  %_11 = alloca [8 x i8], align 8
  %layout = alloca [16 x i8], align 8
  store i64 %0, ptr %layout, align 8
  %3 = getelementptr inbounds i8, ptr %layout, i64 8
  store i64 %1, ptr %3, align 8
  br label %bb3

bb3:                                              ; preds = %start
; call core::ptr::read_volatile::precondition_check
  call void @_ZN4core3ptr13read_volatile18precondition_check17hf68604e7045007d4E(ptr @__rust_no_alloc_shim_is_unstable, i64 1, i1 zeroext false) #15
  br label %bb5

bb5:                                              ; preds = %bb3
  %4 = load volatile i8, ptr @__rust_no_alloc_shim_is_unstable, align 1
  store i8 %4, ptr %2, align 1
  %_2 = load i8, ptr %2, align 1
  %5 = getelementptr inbounds i8, ptr %layout, i64 8
  %_3 = load i64, ptr %5, align 8
  %_10 = load i64, ptr %layout, align 8
  store i64 %_10, ptr %_11, align 8
  %_12 = load i64, ptr %_11, align 8
  %_13 = icmp uge i64 %_12, 1
  %_14 = icmp ule i64 %_12, -9223372036854775808
  %_15 = and i1 %_13, %_14
  %_0 = call ptr @__rust_alloc(i64 %_3, i64 %_12) #15
  ret ptr %_0
}

; alloc::alloc::dealloc
; Function Attrs: inlinehint uwtable
define internal void @_ZN5alloc5alloc7dealloc17he0a031102c73b8aaE(ptr %ptr, i64 %0, i64 %1) unnamed_addr #1 {
start:
  %_8 = alloca [8 x i8], align 8
  %layout = alloca [16 x i8], align 8
  store i64 %0, ptr %layout, align 8
  %2 = getelementptr inbounds i8, ptr %layout, i64 8
  store i64 %1, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %layout, i64 8
  %_3 = load i64, ptr %3, align 8
  %_7 = load i64, ptr %layout, align 8
  store i64 %_7, ptr %_8, align 8
  %_9 = load i64, ptr %_8, align 8
  %_10 = icmp uge i64 %_9, 1
  %_11 = icmp ule i64 %_9, -9223372036854775808
  %_12 = and i1 %_10, %_11
  call void @__rust_dealloc(ptr %ptr, i64 %_3, i64 %_9) #15
  ret void
}

; <core::alloc::layout::LayoutError as core::fmt::Debug>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN69_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$3fmt17hbb0a12f0fb8e59e1E"(ptr align 1 %self, ptr align 8 %f) unnamed_addr #1 {
start:
; call core::fmt::Formatter::write_str
  %_0 = call zeroext i1 @_ZN4core3fmt9Formatter9write_str17h5e37095e3834c6f9E(ptr align 8 %f, ptr align 1 @alloc_477d6fd19596379e27b9bd4358550877, i64 11)
  ret i1 %_0
}

; absint_rust_unsafe::fill_buffer
; Function Attrs: uwtable
define internal void @_ZN18absint_rust_unsafe11fill_buffer17h3d7f58bddc241b5eE(ptr %ptr, i64 %count) unnamed_addr #0 {
start:
  %i = alloca [8 x i8], align 8
  store i64 0, ptr %i, align 8
  br label %bb1

bb1:                                              ; preds = %bb5, %start
  %_5 = load i64, ptr %i, align 8
  %_4 = icmp ult i64 %_5, %count
  br i1 %_4, label %bb2, label %bb6

bb6:                                              ; preds = %bb1
  ret void

bb2:                                              ; preds = %bb1
  %_7 = load i64, ptr %i, align 8
  %0 = call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %_7, i64 2)
  %_8.0 = extractvalue { i64, i1 } %0, 0
  %_8.1 = extractvalue { i64, i1 } %0, 1
  br i1 %_8.1, label %panic, label %bb3

bb3:                                              ; preds = %bb2
  %_10 = load i64, ptr %i, align 8
  %_0.i = getelementptr inbounds i32, ptr %ptr, i64 %_10
  %_13 = ptrtoint ptr %_0.i to i64
  %_16 = and i64 %_13, 3
  %_17 = icmp eq i64 %_16, 0
  br i1 %_17, label %bb7, label %panic1

panic:                                            ; preds = %bb2
; call core::panicking::panic_const::panic_const_mul_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_mul_overflow17hbd7a0374daff3d72E(ptr align 8 @alloc_6e2729e3998b9f7fa255858dc4b6768e) #12
  unreachable

bb7:                                              ; preds = %bb3
  %1 = trunc i64 %_8.0 to i32
  store i32 %1, ptr %_0.i, align 4
  %2 = load i64, ptr %i, align 8
  %3 = call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %2, i64 1)
  %_11.0 = extractvalue { i64, i1 } %3, 0
  %_11.1 = extractvalue { i64, i1 } %3, 1
  br i1 %_11.1, label %panic2, label %bb5

panic1:                                           ; preds = %bb3
  %_13.lcssa = phi i64 [ %_13, %bb3 ]
; call core::panicking::panic_misaligned_pointer_dereference
  call void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64 4, i64 %_13.lcssa, ptr align 8 @alloc_df786ddc2ceecd18523377e43ddb972c) #13
  unreachable

bb5:                                              ; preds = %bb7
  store i64 %_11.0, ptr %i, align 8
  br label %bb1

panic2:                                           ; preds = %bb7
; call core::panicking::panic_const::panic_const_add_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h77c4220018e275ffE(ptr align 8 @alloc_c7d09e11100a7625eecbef574109d7ae) #12
  unreachable
}

; absint_rust_unsafe::sum_buffer
; Function Attrs: uwtable
define internal i32 @_ZN18absint_rust_unsafe10sum_buffer17h6b8aba50f7ef0857E(ptr %ptr, i64 %count) unnamed_addr #0 {
start:
  %i = alloca [8 x i8], align 8
  %total = alloca [4 x i8], align 4
  store i32 0, ptr %total, align 4
  store i64 0, ptr %i, align 8
  br label %bb1

bb1:                                              ; preds = %bb5, %start
  %_6 = load i64, ptr %i, align 8
  %_5 = icmp ult i64 %_6, %count
  br i1 %_5, label %bb2, label %bb6

bb6:                                              ; preds = %bb1
  %_0 = load i32, ptr %total, align 4
  ret i32 %_0

bb2:                                              ; preds = %bb1
  %_8 = load i32, ptr %total, align 4
  %_11 = load i64, ptr %i, align 8
  %_0.i2 = getelementptr inbounds i32, ptr %ptr, i64 %_11
  %_14 = ptrtoint ptr %_0.i2 to i64
  %_17 = and i64 %_14, 3
  %_18 = icmp eq i64 %_17, 0
  br i1 %_18, label %bb7, label %panic

bb7:                                              ; preds = %bb2
  %_9 = load i32, ptr %_0.i2, align 4
  %_0.i = add i32 %_8, %_9
  store i32 %_0.i, ptr %total, align 4
  %0 = load i64, ptr %i, align 8
  %1 = call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %0, i64 1)
  %_12.0 = extractvalue { i64, i1 } %1, 0
  %_12.1 = extractvalue { i64, i1 } %1, 1
  br i1 %_12.1, label %panic1, label %bb5

panic:                                            ; preds = %bb2
  %_14.lcssa = phi i64 [ %_14, %bb2 ]
; call core::panicking::panic_misaligned_pointer_dereference
  call void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64 4, i64 %_14.lcssa, ptr align 8 @alloc_9f5bd7b72be290ba2a5ad7f432b7dbbe) #13
  unreachable

bb5:                                              ; preds = %bb7
  store i64 %_12.0, ptr %i, align 8
  br label %bb1

panic1:                                           ; preds = %bb7
; call core::panicking::panic_const::panic_const_add_overflow
  call void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h77c4220018e275ffE(ptr align 8 @alloc_3c69dd3f9458a33b68ba82e7fa768a72) #12
  unreachable
}

; absint_rust_unsafe::main
; Function Attrs: uwtable
define internal void @_ZN18absint_rust_unsafe4main17h06da306b3c566479E() unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %e.i = alloca [0 x i8], align 1
  %self.i = alloca [16 x i8], align 8
; call core::alloc::layout::Layout::array
  %1 = call { i64, i64 } @_ZN4core5alloc6layout6Layout5array17h75d3052110da3143E(i64 8)
  %_3.0 = extractvalue { i64, i64 } %1, 0
  %_3.1 = extractvalue { i64, i64 } %1, 1
  store i64 %_3.0, ptr %self.i, align 8
  %2 = getelementptr inbounds i8, ptr %self.i, i64 8
  store i64 %_3.1, ptr %2, align 8
  %3 = load i64, ptr %self.i, align 8
  %4 = icmp eq i64 %3, 0
  %_2.i = select i1 %4, i64 1, i64 0
  %5 = icmp eq i64 %_2.i, 0
  br i1 %5, label %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h57b18de5ee2afd5dE.exit", label %bb2.i

bb2.i:                                            ; preds = %start
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17ha5a2c69de0fb8105E(ptr align 1 @alloc_00ae4b301f7fab8ac9617c03fcbd7274, i64 43, ptr align 1 %e.i, ptr align 8 @vtable.1, ptr align 8 @alloc_e417d40d34051c06476c4fe3f7e17f7a) #12
          to label %unreachable.i unwind label %cleanup.i

cleanup.i:                                        ; preds = %bb2.i
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  %10 = load ptr, ptr %0, align 8
  %11 = getelementptr inbounds i8, ptr %0, i64 8
  %12 = load i32, ptr %11, align 8
  %13 = insertvalue { ptr, i32 } poison, ptr %10, 0
  %14 = insertvalue { ptr, i32 } %13, i32 %12, 1
  resume { ptr, i32 } %14

unreachable.i:                                    ; preds = %bb2.i
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h57b18de5ee2afd5dE.exit": ; preds = %start
  %t.0.i = load i64, ptr %self.i, align 8
  %15 = getelementptr inbounds i8, ptr %self.i, i64 8
  %t.1.i = load i64, ptr %15, align 8
  %16 = insertvalue { i64, i64 } poison, i64 %t.0.i, 0
  %17 = insertvalue { i64, i64 } %16, i64 %t.1.i, 1
  %layout.0 = extractvalue { i64, i64 } %17, 0
  %layout.1 = extractvalue { i64, i64 } %17, 1
; call alloc::alloc::alloc
  %_5 = call ptr @_ZN5alloc5alloc5alloc17hcf4c878396f6a8b3E(i64 %layout.0, i64 %layout.1)
; call core::ptr::mut_ptr::<impl *mut T>::is_null
  %_6 = call zeroext i1 @"_ZN4core3ptr7mut_ptr31_$LT$impl$u20$$BP$mut$u20$T$GT$7is_null17h090bb3162c5a283dE"(ptr %_5)
  br i1 %_6, label %bb10, label %bb5

bb5:                                              ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h57b18de5ee2afd5dE.exit"
; call absint_rust_unsafe::fill_buffer
  call void @_ZN18absint_rust_unsafe11fill_buffer17h3d7f58bddc241b5eE(ptr %_5, i64 8)
; call absint_rust_unsafe::sum_buffer
  %result = call i32 @_ZN18absint_rust_unsafe10sum_buffer17h6b8aba50f7ef0857E(ptr %_5, i64 8)
  %_10 = icmp sgt i32 %result, 0
  br i1 %_10, label %bb8, label %bb9

bb10:                                             ; preds = %bb9, %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h57b18de5ee2afd5dE.exit"
  ret void

bb9:                                              ; preds = %bb11, %bb5
; call alloc::alloc::dealloc
  call void @_ZN5alloc5alloc7dealloc17he0a031102c73b8aaE(ptr %_5, i64 %layout.0, i64 %layout.1)
  br label %bb10

bb8:                                              ; preds = %bb5
  %_14 = ptrtoint ptr %_5 to i64
  %_17 = and i64 %_14, 3
  %_18 = icmp eq i64 %_17, 0
  br i1 %_18, label %bb11, label %panic

bb11:                                             ; preds = %bb8
  store i32 %result, ptr %_5, align 4
  br label %bb9

panic:                                            ; preds = %bb8
; call core::panicking::panic_misaligned_pointer_dereference
  call void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64 4, i64 %_14, ptr align 8 @alloc_af89d28216c7809dc696dcc45c848409) #13
  unreachable
}

; std::rt::lang_start_internal
; Function Attrs: uwtable
declare i64 @_ZN3std2rt19lang_start_internal17he1ad9a314bd0157aE(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #0

; Function Attrs: nounwind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #4

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctpop.i64(i64) #5

; core::panicking::panic_nounwind
; Function Attrs: cold noinline noreturn nounwind uwtable
declare void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1, i64) unnamed_addr #6

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking9panic_fmt17h169d7389ef09f3baE(ptr align 8, ptr align 8) unnamed_addr #7

; core::panicking::panic_cannot_unwind
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() unnamed_addr #8

; core::alloc::layout::Layout::is_size_align_valid
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core5alloc6layout6Layout19is_size_align_valid17hba5c1dd91ce45078E(i64, i64) unnamed_addr #0

; core::panicking::panic_const::panic_const_div_by_zero
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17he9029e02cc767683E(ptr align 8) unnamed_addr #7

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core6result13unwrap_failed17ha5a2c69de0fb8105E(ptr align 1, i64, ptr align 1, ptr align 8, ptr align 8) unnamed_addr #7

; Function Attrs: nounwind allockind("alloc,uninitialized,aligned") allocsize(0) uwtable
declare noalias ptr @__rust_alloc(i64, i64 allocalign) unnamed_addr #9

; Function Attrs: nounwind allockind("free") uwtable
declare void @__rust_dealloc(ptr allocptr, i64, i64) unnamed_addr #10

; core::fmt::Formatter::write_str
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core3fmt9Formatter9write_str17h5e37095e3834c6f9E(ptr align 8, ptr align 1, i64) unnamed_addr #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i64, i1 } @llvm.umul.with.overflow.i64(i64, i64) #5

; core::panicking::panic_const::panic_const_mul_overflow
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const24panic_const_mul_overflow17hbd7a0374daff3d72E(ptr align 8) unnamed_addr #7

; core::panicking::panic_misaligned_pointer_dereference
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking36panic_misaligned_pointer_dereference17h008a4d0db91bd7f0E(i64, i64, ptr align 8) unnamed_addr #8

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i64, i1 } @llvm.uadd.with.overflow.i64(i64, i64) #5

; core::panicking::panic_const::panic_const_add_overflow
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const24panic_const_add_overflow17h77c4220018e275ffE(ptr align 8) unnamed_addr #7

define i32 @main(i32 %0, ptr %1) unnamed_addr #11 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start
  %3 = call i64 @_ZN3std2rt10lang_start17h284b81f5805dd514E(ptr @_ZN18absint_rust_unsafe4main17h06da306b3c566479E, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #1 = { inlinehint uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #2 = { noinline uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #3 = { inlinehint nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #4 = { nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #5 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #6 = { cold noinline noreturn nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #7 = { cold noinline noreturn uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #8 = { cold minsize noinline noreturn nounwind optsize uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #9 = { nounwind allockind("alloc,uninitialized,aligned") allocsize(0) uwtable "alloc-family"="__rust_alloc" "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #10 = { nounwind allockind("free") uwtable "alloc-family"="__rust_alloc" "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #11 = { "target-cpu"="generic" }
attributes #12 = { noreturn }
attributes #13 = { noreturn nounwind }
attributes #14 = { cold noreturn nounwind }
attributes #15 = { nounwind }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{!"rustc version 1.85.0 (4d91de4e4 2025-02-17)"}
!3 = !{i64 11053390396605661}
