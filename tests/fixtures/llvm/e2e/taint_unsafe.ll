; ModuleID = 'taint_unsafe.836aae2104260936-cgu.0'
source_filename = "taint_unsafe.836aae2104260936-cgu.0"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "aarch64-unknown-linux-gnu"

%"alloc::string::String" = type { %"alloc::vec::Vec<u8>" }
%"alloc::vec::Vec<u8>" = type { %"alloc::raw_vec::RawVec<u8>", i64 }
%"alloc::raw_vec::RawVec<u8>" = type { %"alloc::raw_vec::RawVecInner", %"core::marker::PhantomData<u8>" }
%"alloc::raw_vec::RawVecInner" = type { i64, ptr, %"alloc::alloc::Global" }
%"alloc::alloc::Global" = type {}
%"core::marker::PhantomData<u8>" = type {}
%"std::ffi::os_str::OsString" = type { %"std::sys::os_str::bytes::Buf" }
%"std::sys::os_str::bytes::Buf" = type { %"alloc::vec::Vec<u8>" }

@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h7ae639dd42b41cc7E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hb39a040a5e63a0acE", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hb39a040a5e63a0acE" }>, align 8
@vtable.1 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h9ac37d5c5961f8bdE" }>, align 8
@alloc_ec595fc0e82ef92fc59bd74f68296eae = private unnamed_addr constant <{ [73 x i8] }> <{ [73 x i8] c"assertion failed: 0 < pointee_size && pointee_size <= isize::MAX as usize" }>, align 1
@alloc_2d5b6a4803df6aa853303aedba87a5fe = private unnamed_addr constant <{ [81 x i8] }> <{ [81 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/ptr/const_ptr.rs" }>, align 1
@alloc_69ac747af8a1bac76e0a9aa808491447 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_2d5b6a4803df6aa853303aedba87a5fe, [16 x i8] c"Q\00\00\00\00\00\00\00 \03\00\00\09\00\00\00" }>, align 8
@alloc_7efb3a7632b3620f628ce83a521b4d9b = private unnamed_addr constant <{ [71 x i8] }> <{ [71 x i8] c"unsafe precondition(s) violated: ptr::sub_ptr requires `self >= origin`" }>, align 1
@alloc_ab14703751a9eb3585c49b2e55e9a9e5 = private unnamed_addr constant <{ [104 x i8] }> <{ [104 x i8] c"unsafe precondition(s) violated: hint::assert_unchecked must never be called when the condition is false" }>, align 1
@alloc_92e94fa929ea109d17457c88ea3d4b96 = private unnamed_addr constant <{ [90 x i8] }> <{ [90 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/iter/traits/exact_size.rs" }>, align 1
@alloc_f9a12aa962eb349e0d12fd6ea34d8b62 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_92e94fa929ea109d17457c88ea3d4b96, [16 x i8] c"Z\00\00\00\00\00\00\00z\00\00\00\09\00\00\00" }>, align 8
@alloc_8f8b97b746543e34920be8d369edf237 = private unnamed_addr constant <{ [88 x i8] }> <{ [88 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/iter/traits/iterator.rs" }>, align 1
@alloc_31ba853be958e604da5cf95140670bc2 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_8f8b97b746543e34920be8d369edf237, [16 x i8] c"X\00\00\00\00\00\00\00\B3\07\00\00\09\00\00\00" }>, align 8
@alloc_cd1513ae8d1ae22acf9342b8dfa1561d = private unnamed_addr constant <{ [164 x i8] }> <{ [164 x i8] c"unsafe precondition(s) violated: Layout::from_size_align_unchecked requires that align is a power of 2 and the rounded-up allocation size does not exceed isize::MAX" }>, align 1
@alloc_4da616ab4fab553451f0131bd308a0a1 = private unnamed_addr constant <{ [77 x i8] }> <{ [77 x i8] c"/rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/core/src/ub_checks.rs" }>, align 1
@alloc_ccfb06f4f0e3e21a92aa1cae417f3225 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_4da616ab4fab553451f0131bd308a0a1, [16 x i8] c"M\00\00\00\00\00\00\00\86\00\00\006\00\00\00" }>, align 8
@alloc_11195730e5236cfdc15ea13be1c301f9 = private unnamed_addr constant <{ [162 x i8] }> <{ [162 x i8] c"unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`" }>, align 1
@alloc_fad0cd83b7d1858a846a172eb260e593 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"is_aligned_to: align is not a power-of-two" }>, align 1
@alloc_e92e94d0ff530782b571cfd99ec66aef = private unnamed_addr constant <{ ptr, [8 x i8] }> <{ ptr @alloc_fad0cd83b7d1858a846a172eb260e593, [8 x i8] c"*\00\00\00\00\00\00\00" }>, align 8
@0 = private unnamed_addr constant <{ [8 x i8], [8 x i8] }> <{ [8 x i8] zeroinitializer, [8 x i8] undef }>, align 8
@alloc_369fae759a17104aba57fba3370bcbfb = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_2d5b6a4803df6aa853303aedba87a5fe, [16 x i8] c"Q\00\00\00\00\00\00\00\C8\05\00\00\0D\00\00\00" }>, align 8
@vtable.2 = private unnamed_addr constant <{ ptr, [16 x i8], ptr }> <{ ptr @"_ZN4core3ptr48drop_in_place$LT$alloc..ffi..c_str..NulError$GT$17hed00ef0c138c90cfE", [16 x i8] c" \00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN64_$LT$alloc..ffi..c_str..NulError$u20$as$u20$core..fmt..Debug$GT$3fmt17h998d0b57d1ebbee1E" }>, align 8
@alloc_00ae4b301f7fab8ac9617c03fcbd7274 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@vtable.3 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17hfc7f7844df3ca281E" }>, align 8
@vtable.4 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h577bef94385bfb3cE" }>, align 8
@alloc_49c0eff15ce41ce22a2d8c8b146a94ef = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"NulError" }>, align 1
@alloc_e96fb6e25c55edb0aec8b24d111f5d7f = private unnamed_addr constant <{ [101 x i8] }> <{ [101 x i8] c"unsafe precondition(s) violated: slice::get_unchecked_mut requires that the index is within the slice" }>, align 1
@alloc_41f459a1ce436c4b40759266f9823475 = private unnamed_addr constant <{ [46 x i8] }> <{ [46 x i8] c"/workspace/tests/programs/rust/taint_unsafe.rs" }>, align 1
@alloc_425effdcb2ac5d66167160ce84f7c716 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_41f459a1ce436c4b40759266f9823475, [16 x i8] c".\00\00\00\00\00\00\00\11\00\00\00\14\00\00\00" }>, align 8
@alloc_4f05f4a8f7ae5dff1b6009babd7dab65 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_41f459a1ce436c4b40759266f9823475, [16 x i8] c".\00\00\00\00\00\00\00\13\00\00\000\00\00\00" }>, align 8

; <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
; Function Attrs: inlinehint uwtable
define internal void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h2cf6abf74155070cE"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
  %_13 = alloca [16 x i8], align 8
  %exact = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
  %_10 = getelementptr inbounds i8, ptr %self, i64 24
  %_8 = load ptr, ptr %_10, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_11 = load ptr, ptr %0, align 8
; call core::ptr::non_null::NonNull<T>::sub_ptr
  %1 = call i64 @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$7sub_ptr17h4a6f9cc3cebb074eE"(ptr %_8, ptr %_11)
  store i64 %1, ptr %exact, align 8
  br label %bb4

bb4:                                              ; preds = %bb2
  %_12 = load i64, ptr %exact, align 8
  %_14 = load i64, ptr %exact, align 8
  %2 = getelementptr inbounds i8, ptr %_13, i64 8
  store i64 %_14, ptr %2, align 8
  store i64 1, ptr %_13, align 8
  store i64 %_12, ptr %_0, align 8
  %3 = load i64, ptr %_13, align 8
  %4 = getelementptr inbounds i8, ptr %_13, i64 8
  %5 = load i64, ptr %4, align 8
  %6 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %3, ptr %6, align 8
  %7 = getelementptr inbounds i8, ptr %6, i64 8
  store i64 %5, ptr %7, align 8
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
; Function Attrs: uwtable
define internal void @"_ZN111_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter_nested..SpecFromIterNested$LT$T$C$I$GT$$GT$9from_iter17h8dbb3058bb4d5433E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [8 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %_20 = alloca [1 x i8], align 1
  %_19 = alloca [32 x i8], align 8
  %src = alloca [24 x i8], align 8
  %vector1 = alloca [24 x i8], align 8
  %_8 = alloca [24 x i8], align 8
  %element = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %vector = alloca [24 x i8], align 8
  store i8 1, ptr %_20, align 1
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::next
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h7da987a022c3da75E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %iterator)
          to label %bb1 unwind label %cleanup

bb11:                                             ; preds = %bb9, %bb7, %cleanup
  %3 = load i8, ptr %_20, align 1
  %4 = trunc i8 %3 to i1
  br i1 %4, label %bb10, label %bb8

cleanup:                                          ; preds = %start
  %5 = landingpad { ptr, i32 }
          cleanup
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
  store ptr %6, ptr %2, align 8
  %8 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %7, ptr %8, align 8
  br label %bb11

bb1:                                              ; preds = %start
  %9 = load i64, ptr %_3, align 8
  %10 = icmp eq i64 %9, -9223372036854775808
  %_5 = select i1 %10, i64 0, i64 1
  %11 = icmp eq i64 %_5, 0
  br i1 %11, label %bb12, label %bb3

bb12:                                             ; preds = %bb1
  store i64 0, ptr %_0, align 8
  %12 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr getelementptr (i8, ptr null, i64 8), ptr %12, align 8
  %13 = getelementptr inbounds i8, ptr %_0, i64 16
  store i64 0, ptr %13, align 8
; call core::ptr::drop_in_place<std::env::Args>
  call void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17h0e0f28defd111449E"(ptr align 8 %iterator)
  br label %bb6

bb3:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %element, ptr align 8 %_3, i64 24, i1 false)
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h834281278c633905E"(ptr sret([24 x i8]) align 8 %_8, ptr align 8 %iterator)
          to label %bb4 unwind label %cleanup2

bb6:                                              ; preds = %bb5, %bb12
  ret void

bb9:                                              ; preds = %cleanup2
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %element) #15
          to label %bb11 unwind label %terminate

cleanup2:                                         ; preds = %bb14, %bb4, %bb3
  %14 = landingpad { ptr, i32 }
          cleanup
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
  store ptr %15, ptr %2, align 8
  %17 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %16, ptr %17, align 8
  br label %bb9

bb4:                                              ; preds = %bb3
  %lower = load i64, ptr %_8, align 8
  %18 = call i64 @llvm.uadd.sat.i64(i64 %lower, i64 1)
  store i64 %18, ptr %1, align 8
  %v2 = load i64, ptr %1, align 8
; invoke core::cmp::max_by
  %initial_capacity = invoke i64 @_ZN4core3cmp6max_by17hb9c984b4a0d08107E(i64 4, i64 %v2)
          to label %bb14 unwind label %cleanup2

bb14:                                             ; preds = %bb4
; invoke alloc::raw_vec::RawVecInner<A>::with_capacity_in
  %19 = invoke { i64, ptr } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$16with_capacity_in17h3f0b3cda90ff39b9E"(i64 %initial_capacity, i64 8, i64 24, ptr align 8 %0)
          to label %bb15 unwind label %cleanup2

bb15:                                             ; preds = %bb14
  %_26.0 = extractvalue { i64, ptr } %19, 0
  %_26.1 = extractvalue { i64, ptr } %19, 1
  store i64 %_26.0, ptr %vector1, align 8
  %20 = getelementptr inbounds i8, ptr %vector1, i64 8
  store ptr %_26.1, ptr %20, align 8
  %21 = getelementptr inbounds i8, ptr %vector1, i64 16
  store i64 0, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %vector1, i64 8
  %_30 = load ptr, ptr %22, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %src, ptr align 8 %element, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_30, ptr align 8 %src, i64 24, i1 false)
  %23 = getelementptr inbounds i8, ptr %vector1, i64 16
  store i64 1, ptr %23, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %vector, ptr align 8 %vector1, i64 24, i1 false)
  store i8 0, ptr %_20, align 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_19, ptr align 8 %iterator, i64 32, i1 false)
; invoke <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<T,I>>::spec_extend
  invoke void @"_ZN97_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$T$C$I$GT$$GT$11spec_extend17hf8af74ff453fff59E"(ptr align 8 %vector, ptr align 8 %_19, ptr align 8 %0)
          to label %bb5 unwind label %cleanup3

bb7:                                              ; preds = %cleanup3
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  invoke void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17hbd10dcd190b7dadaE"(ptr align 8 %vector) #15
          to label %bb11 unwind label %terminate

cleanup3:                                         ; preds = %bb15
  %24 = landingpad { ptr, i32 }
          cleanup
  %25 = extractvalue { ptr, i32 } %24, 0
  %26 = extractvalue { ptr, i32 } %24, 1
  store ptr %25, ptr %2, align 8
  %27 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %26, ptr %27, align 8
  br label %bb7

bb5:                                              ; preds = %bb15
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %vector, i64 24, i1 false)
  br label %bb6

terminate:                                        ; preds = %bb10, %bb9, %bb7
  %28 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %29 = extractvalue { ptr, i32 } %28, 0
  %30 = extractvalue { ptr, i32 } %28, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb2:                                              ; No predecessors!
  unreachable

bb8:                                              ; preds = %bb10, %bb11
  %31 = load ptr, ptr %2, align 8
  %32 = getelementptr inbounds i8, ptr %2, i64 8
  %33 = load i32, ptr %32, align 8
  %34 = insertvalue { ptr, i32 } poison, ptr %31, 0
  %35 = insertvalue { ptr, i32 } %34, i32 %33, 1
  resume { ptr, i32 } %35

bb10:                                             ; preds = %bb11
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17h0e0f28defd111449E"(ptr align 8 %iterator) #15
          to label %bb8 unwind label %terminate
}

; <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf398ea37a8290977E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %capacity = alloca [8 x i8], align 8
  %_4 = alloca [16 x i8], align 8
  %_7 = load ptr, ptr %self, align 8
  %slot = getelementptr inbounds i8, ptr %_7, i64 32
  %_8 = load ptr, ptr %self, align 8
  %ptr = load ptr, ptr %_8, align 8
  %_9 = load ptr, ptr %self, align 8
  %0 = getelementptr inbounds i8, ptr %_9, i64 16
  %capacity1 = load i64, ptr %0, align 8
  br label %bb4

bb4:                                              ; preds = %start
  store i64 %capacity1, ptr %capacity, align 8
  br label %bb2

bb2:                                              ; preds = %bb4
  %cap = load i64, ptr %capacity, align 8
  store i64 %cap, ptr %_4, align 8
  %1 = getelementptr inbounds i8, ptr %_4, i64 8
  store ptr %ptr, ptr %1, align 8
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17hc2d427da46fc894fE"(ptr align 8 %_4)
  ret void

bb3:                                              ; No predecessors!
  unreachable
}

; std::rt::lang_start
; Function Attrs: uwtable
define hidden i64 @_ZN3std2rt10lang_start17h7bd44b8437c8e17eE(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #1 {
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
define internal i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hb39a040a5e63a0acE"(ptr align 8 %_1) unnamed_addr #0 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace
  call void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h3c64f0181387c7edE(ptr %_4)
; call <() as std::process::Termination>::report
  %self = call i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hcbdd2bc10105ff4fE"()
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; std::sys::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline uwtable
define internal void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h3c64f0181387c7edE(ptr %f) unnamed_addr #2 {
start:
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17h51e7e953690d0072E(ptr %f)
  call void asm sideeffect "", "~{memory}"(), !srcloc !3
  ret void
}

; <&T as core::fmt::Debug>::fmt
; Function Attrs: uwtable
define internal zeroext i1 @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h577bef94385bfb3cE"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #1 {
start:
  %_3 = load ptr, ptr %self, align 8
; call <alloc::vec::Vec<T,A> as core::fmt::Debug>::fmt
  %_0 = call zeroext i1 @"_ZN65_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h8c4fdc0f776f2397E"(ptr align 8 %_3, ptr align 8 %f)
  ret i1 %_0
}

; <&T as core::fmt::Debug>::fmt
; Function Attrs: uwtable
define internal zeroext i1 @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h9ac37d5c5961f8bdE"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #1 {
start:
  %_3 = load ptr, ptr %self, align 8
; call core::fmt::num::<impl core::fmt::Debug for u8>::fmt
  %_0 = call zeroext i1 @"_ZN4core3fmt3num49_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u8$GT$3fmt17h97460367163c84dcE"(ptr align 1 %_3, ptr align 8 %f)
  ret i1 %_0
}

; <[T] as core::fmt::Debug>::fmt
; Function Attrs: uwtable
define internal zeroext i1 @"_ZN48_$LT$$u5b$T$u5d$$u20$as$u20$core..fmt..Debug$GT$3fmt17h2c7a11b238c7fa5fE"(ptr align 1 %self.0, i64 %self.1, ptr align 8 %f) unnamed_addr #1 {
start:
  %end_or_len = alloca [8 x i8], align 8
  %_5 = alloca [16 x i8], align 8
; call core::fmt::Formatter::debug_list
  call void @_ZN4core3fmt9Formatter10debug_list17hbd42fc068dd3484cE(ptr sret([16 x i8]) align 8 %_5, ptr align 8 %f)
  br label %bb5

bb5:                                              ; preds = %start
  %_11 = getelementptr inbounds i8, ptr %self.0, i64 %self.1
  store ptr %_11, ptr %end_or_len, align 8
  br label %bb6

bb6:                                              ; preds = %bb5
  %_13 = load ptr, ptr %end_or_len, align 8
; call core::fmt::builders::DebugList::entries
  %_3 = call align 8 ptr @_ZN4core3fmt8builders9DebugList7entries17h7ff6f0a0943ebe6fE(ptr align 8 %_5, ptr %self.0, ptr %_13)
; call core::fmt::builders::DebugList::finish
  %_0 = call zeroext i1 @_ZN4core3fmt8builders9DebugList6finish17h58eea66f4eeff9fdE(ptr align 8 %_3)
  ret i1 %_0

bb4:                                              ; No predecessors!
  unreachable
}

; core::cmp::impls::<impl core::cmp::Ord for usize>::cmp
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN4core3cmp5impls50_$LT$impl$u20$core..cmp..Ord$u20$for$u20$usize$GT$3cmp17h6574519b1dd6bdccE"(ptr align 8 %self, ptr align 8 %other) unnamed_addr #0 {
start:
  %_3 = load i64, ptr %self, align 8
  %_4 = load i64, ptr %other, align 8
  %0 = icmp ugt i64 %_3, %_4
  %1 = zext i1 %0 to i8
  %2 = icmp ult i64 %_3, %_4
  %3 = zext i1 %2 to i8
  %_0 = sub nsw i8 %1, %3
  ret i8 %_0
}

; core::cmp::max_by
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core3cmp6max_by17hb9c984b4a0d08107E(i64 %0, i64 %1) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %2 = alloca [16 x i8], align 8
  %_9 = alloca [1 x i8], align 1
  %_4 = alloca [1 x i8], align 1
  %_0 = alloca [8 x i8], align 8
  %v2 = alloca [8 x i8], align 8
  %v1 = alloca [8 x i8], align 8
  store i64 %0, ptr %v1, align 8
  store i64 %1, ptr %v2, align 8
  store i8 1, ptr %_9, align 1
; invoke core::ops::function::FnOnce::call_once
  %3 = invoke i8 @_ZN4core3ops8function6FnOnce9call_once17he458027e20818dc4E(ptr align 8 %v1, ptr align 8 %v2)
          to label %bb1 unwind label %cleanup

bb6:                                              ; preds = %cleanup
  br label %bb10

cleanup:                                          ; preds = %start
  %4 = landingpad { ptr, i32 }
          cleanup
  %5 = extractvalue { ptr, i32 } %4, 0
  %6 = extractvalue { ptr, i32 } %4, 1
  store ptr %5, ptr %2, align 8
  %7 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %6, ptr %7, align 8
  br label %bb6

bb1:                                              ; preds = %start
  store i8 %3, ptr %_4, align 1
  %_8 = load i8, ptr %_4, align 1
  switch i8 %_8, label %bb2 [
    i8 -1, label %bb4
    i8 0, label %bb4
    i8 1, label %bb3
  ]

bb2:                                              ; preds = %bb1
  unreachable

bb4:                                              ; preds = %bb1, %bb1
  %8 = load i64, ptr %v2, align 8
  store i64 %8, ptr %_0, align 8
  %9 = load i8, ptr %_9, align 1
  %10 = trunc i8 %9 to i1
  br i1 %10, label %bb8, label %bb5

bb3:                                              ; preds = %bb1
  store i8 0, ptr %_9, align 1
  %11 = load i64, ptr %v1, align 8
  store i64 %11, ptr %_0, align 8
  br label %bb5

bb5:                                              ; preds = %bb3, %bb8, %bb4
  %12 = load i64, ptr %_0, align 8
  ret i64 %12

bb8:                                              ; preds = %bb4
  br label %bb5

bb10:                                             ; preds = %bb6
  %13 = load i8, ptr %_9, align 1
  %14 = trunc i8 %13 to i1
  br i1 %14, label %bb9, label %bb7

bb7:                                              ; preds = %bb9, %bb10
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds i8, ptr %2, i64 8
  %17 = load i32, ptr %16, align 8
  %18 = insertvalue { ptr, i32 } poison, ptr %15, 0
  %19 = insertvalue { ptr, i32 } %18, i32 %17, 1
  resume { ptr, i32 } %19

bb9:                                              ; preds = %bb10
  br label %bb7
}

; core::ffi::c_str::CStr::as_ptr
; Function Attrs: inlinehint uwtable
define internal ptr @_ZN4core3ffi5c_str4CStr6as_ptr17h4b6be2744fa460e1E(ptr align 1 %self.0, i64 %self.1) unnamed_addr #0 {
start:
  ret ptr %self.0
}

; core::fmt::num::<impl core::fmt::Debug for u8>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN4core3fmt3num49_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u8$GT$3fmt17h97460367163c84dcE"(ptr align 1 %self, ptr align 8 %f) unnamed_addr #0 {
start:
  %_0 = alloca [1 x i8], align 1
  %0 = getelementptr inbounds i8, ptr %f, i64 36
  %_4 = load i32, ptr %0, align 4
  %_3 = and i32 %_4, 16
  %1 = icmp eq i32 %_3, 0
  br i1 %1, label %bb2, label %bb1

bb2:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %f, i64 36
  %_6 = load i32, ptr %2, align 4
  %_5 = and i32 %_6, 32
  %3 = icmp eq i32 %_5, 0
  br i1 %3, label %bb4, label %bb3

bb1:                                              ; preds = %start
; call core::fmt::num::<impl core::fmt::LowerHex for u8>::fmt
  %4 = call zeroext i1 @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h8618cb0e3425f163E"(ptr align 1 %self, ptr align 8 %f)
  %5 = zext i1 %4 to i8
  store i8 %5, ptr %_0, align 1
  br label %bb6

bb4:                                              ; preds = %bb2
; call core::fmt::num::imp::<impl core::fmt::Display for u8>::fmt
  %6 = call zeroext i1 @"_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17h1d9c5594c24edd21E"(ptr align 1 %self, ptr align 8 %f)
  %7 = zext i1 %6 to i8
  store i8 %7, ptr %_0, align 1
  br label %bb5

bb3:                                              ; preds = %bb2
; call core::fmt::num::<impl core::fmt::UpperHex for u8>::fmt
  %8 = call zeroext i1 @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h6214023d3fccb560E"(ptr align 1 %self, ptr align 8 %f)
  %9 = zext i1 %8 to i8
  store i8 %9, ptr %_0, align 1
  br label %bb5

bb5:                                              ; preds = %bb3, %bb4
  br label %bb6

bb6:                                              ; preds = %bb1, %bb5
  %10 = load i8, ptr %_0, align 1
  %11 = trunc i8 %10 to i1
  ret i1 %11
}

; core::fmt::num::<impl core::fmt::Debug for usize>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17hfc7f7844df3ca281E"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #0 {
start:
  %_0 = alloca [1 x i8], align 1
  %0 = getelementptr inbounds i8, ptr %f, i64 36
  %_4 = load i32, ptr %0, align 4
  %_3 = and i32 %_4, 16
  %1 = icmp eq i32 %_3, 0
  br i1 %1, label %bb2, label %bb1

bb2:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %f, i64 36
  %_6 = load i32, ptr %2, align 4
  %_5 = and i32 %_6, 32
  %3 = icmp eq i32 %_5, 0
  br i1 %3, label %bb4, label %bb3

bb1:                                              ; preds = %start
; call core::fmt::num::<impl core::fmt::LowerHex for usize>::fmt
  %4 = call zeroext i1 @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17ha3e1dcd1e2b6c57fE"(ptr align 8 %self, ptr align 8 %f)
  %5 = zext i1 %4 to i8
  store i8 %5, ptr %_0, align 1
  br label %bb6

bb4:                                              ; preds = %bb2
; call core::fmt::num::imp::<impl core::fmt::Display for usize>::fmt
  %6 = call zeroext i1 @"_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h37c9cfb43684526bE"(ptr align 8 %self, ptr align 8 %f)
  %7 = zext i1 %6 to i8
  store i8 %7, ptr %_0, align 1
  br label %bb5

bb3:                                              ; preds = %bb2
; call core::fmt::num::<impl core::fmt::UpperHex for usize>::fmt
  %8 = call zeroext i1 @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17he8c8aa69aaec47fdE"(ptr align 8 %self, ptr align 8 %f)
  %9 = zext i1 %8 to i8
  store i8 %9, ptr %_0, align 1
  br label %bb5

bb5:                                              ; preds = %bb3, %bb4
  br label %bb6

bb6:                                              ; preds = %bb1, %bb5
  %10 = load i8, ptr %_0, align 1
  %11 = trunc i8 %10 to i1
  ret i1 %11
}

; core::fmt::builders::DebugList::entries
; Function Attrs: uwtable
define internal align 8 ptr @_ZN4core3fmt8builders9DebugList7entries17h7ff6f0a0943ebe6fE(ptr align 8 %self, ptr %entries.0, ptr %entries.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %entry = alloca [8 x i8], align 8
  %_5 = alloca [8 x i8], align 8
  %iter = alloca [16 x i8], align 8
; call <I as core::iter::traits::collect::IntoIterator>::into_iter
  %1 = call { ptr, ptr } @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17hf527dfa6a941dddfE"(ptr %entries.0, ptr %entries.1)
  %_3.0 = extractvalue { ptr, ptr } %1, 0
  %_3.1 = extractvalue { ptr, ptr } %1, 1
  store ptr %_3.0, ptr %iter, align 8
  %2 = getelementptr inbounds i8, ptr %iter, i64 8
  store ptr %_3.1, ptr %2, align 8
  br label %bb2

bb2:                                              ; preds = %bb8, %start
; invoke <core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next
  %3 = invoke align 1 ptr @"_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h19b501c5fbbb2d0aE"(ptr align 8 %iter)
          to label %bb3 unwind label %cleanup

bb11:                                             ; preds = %bb10, %cleanup
  %4 = load ptr, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  %6 = load i32, ptr %5, align 8
  %7 = insertvalue { ptr, i32 } poison, ptr %4, 0
  %8 = insertvalue { ptr, i32 } %7, i32 %6, 1
  resume { ptr, i32 } %8

cleanup:                                          ; preds = %bb2
  %9 = landingpad { ptr, i32 }
          cleanup
  %10 = extractvalue { ptr, i32 } %9, 0
  %11 = extractvalue { ptr, i32 } %9, 1
  store ptr %10, ptr %0, align 8
  %12 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %11, ptr %12, align 8
  br label %bb11

bb3:                                              ; preds = %bb2
  store ptr %3, ptr %_5, align 8
  %13 = load ptr, ptr %_5, align 8
  %14 = ptrtoint ptr %13 to i64
  %15 = icmp eq i64 %14, 0
  %_7 = select i1 %15, i64 0, i64 1
  %16 = icmp eq i64 %_7, 0
  br i1 %16, label %bb6, label %bb5

bb6:                                              ; preds = %bb3
  ret ptr %self

bb5:                                              ; preds = %bb3
  %17 = load ptr, ptr %_5, align 8
  store ptr %17, ptr %entry, align 8
; invoke core::fmt::builders::DebugList::entry
  %_9 = invoke align 8 ptr @_ZN4core3fmt8builders9DebugList5entry17hd4cde1ea99b71f8dE(ptr align 8 %self, ptr align 1 %entry, ptr align 8 @vtable.1)
          to label %bb7 unwind label %cleanup1

bb10:                                             ; preds = %cleanup1
  br label %bb11

cleanup1:                                         ; preds = %bb5
  %18 = landingpad { ptr, i32 }
          cleanup
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
  store ptr %19, ptr %0, align 8
  %21 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %20, ptr %21, align 8
  br label %bb10

bb7:                                              ; preds = %bb5
  br label %bb8

bb8:                                              ; preds = %bb7
  br label %bb2

bb4:                                              ; No predecessors!
  unreachable
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h7ae639dd42b41cc7E"(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call core::ops::function::FnOnce::call_once
  %_0 = call i32 @_ZN4core3ops8function6FnOnce9call_once17h2060c72d1cb0fd83E(ptr %0)
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i32 @_ZN4core3ops8function6FnOnce9call_once17h2060c72d1cb0fd83E(ptr %0) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::{{closure}}
  %_0 = invoke i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hb39a040a5e63a0acE"(ptr align 8 %_1)
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
define internal void @_ZN4core3ops8function6FnOnce9call_once17h51e7e953690d0072E(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i8 @_ZN4core3ops8function6FnOnce9call_once17he458027e20818dc4E(ptr align 8 %0, ptr align 8 %1) unnamed_addr #0 {
start:
  %_2 = alloca [16 x i8], align 8
  store ptr %0, ptr %_2, align 8
  %2 = getelementptr inbounds i8, ptr %_2, i64 8
  store ptr %1, ptr %2, align 8
  %3 = load ptr, ptr %_2, align 8
  %4 = getelementptr inbounds i8, ptr %_2, i64 8
  %5 = load ptr, ptr %4, align 8
; call core::cmp::impls::<impl core::cmp::Ord for usize>::cmp
  %_0 = call i8 @"_ZN4core3cmp5impls50_$LT$impl$u20$core..cmp..Ord$u20$for$u20$usize$GT$3cmp17h6574519b1dd6bdccE"(ptr align 8 %3, ptr align 8 %5)
  ret i8 %_0
}

; core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17h9ea33ad1e2a233ebE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf398ea37a8290977E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<usize>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr26drop_in_place$LT$usize$GT$17h2e92b8de9f8e5ae6E"(ptr align 8 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<&u8>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr27drop_in_place$LT$$RF$u8$GT$17hc727e3dac24b6f32E"(ptr align 8 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<std::env::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17h0e0f28defd111449E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::env::ArgsOs>
  call void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17hd11400b4914d585eE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::env::ArgsOs>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17hd11400b4914d585eE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::pal::unix::args::Args>
  call void @"_ZN4core3ptr52drop_in_place$LT$std..sys..pal..unix..args..Args$GT$17h0fb1c36df737626fE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::string::String>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h88159fe782bb8a45E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h88159fe782bb8a45E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6a7d6a72d1b9e9b7E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  invoke void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hf358b56a26eab698E"(ptr align 8 %_1) #15
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb4:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  call void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hf358b56a26eab698E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb1:                                              ; preds = %bb3
  %8 = load ptr, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  %10 = load i32, ptr %9, align 8
  %11 = insertvalue { ptr, i32 } poison, ptr %8, 0
  %12 = insertvalue { ptr, i32 } %11, i32 %10, 1
  resume { ptr, i32 } %12
}

; core::ptr::drop_in_place<alloc::ffi::c_str::CString>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h8922e58224b554a9E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::ffi::c_str::CString as core::ops::drop::Drop>::drop
  invoke void @"_ZN68_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3ccfd437ff471786E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::boxed::Box<[u8]>>
  invoke void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h2a1f1da80f5540a3E"(ptr align 8 %_1) #15
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb4:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::boxed::Box<[u8]>>
  call void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h2a1f1da80f5540a3E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb1:                                              ; preds = %bb3
  %8 = load ptr, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  %10 = load i32, ptr %9, align 8
  %11 = insertvalue { ptr, i32 } poison, ptr %8, 0
  %12 = insertvalue { ptr, i32 } %11, i32 %10, 1
  resume { ptr, i32 } %12
}

; core::ptr::drop_in_place<std::ffi::os_str::OsString>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17h1d651ed9cd1d168bE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h44f5092a11bc58cfE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::ffi::c_str::NulError>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr48drop_in_place$LT$alloc..ffi..c_str..NulError$GT$17hed00ef0c138c90cfE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h88159fe782bb8a45E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h44f5092a11bc58cfE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h88159fe782bb8a45E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<&alloc::vec::Vec<u8>>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr50drop_in_place$LT$$RF$alloc..vec..Vec$LT$u8$GT$$GT$17hfac71bb48dff0030E"(ptr align 8 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<[alloc::string::String]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr52drop_in_place$LT$$u5b$alloc..string..String$u5d$$GT$17h1f0f1e6fadba3fdeE"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_3 = alloca [8 x i8], align 8
  store i64 0, ptr %_3, align 8
  br label %bb6

bb6:                                              ; preds = %bb5, %start
  %1 = load i64, ptr %_3, align 8
  %_7 = icmp eq i64 %1, %_1.1
  br i1 %_7, label %bb1, label %bb5

bb5:                                              ; preds = %bb6
  %2 = load i64, ptr %_3, align 8
  %_6 = getelementptr inbounds %"alloc::string::String", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %_6)
          to label %bb6 unwind label %cleanup

bb1:                                              ; preds = %bb6
  ret void

bb4:                                              ; preds = %bb3, %cleanup
  %5 = load i64, ptr %_3, align 8
  %_5 = icmp eq i64 %5, %_1.1
  br i1 %_5, label %bb2, label %bb3

cleanup:                                          ; preds = %bb5
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  br label %bb4

bb3:                                              ; preds = %bb4
  %10 = load i64, ptr %_3, align 8
  %_4 = getelementptr inbounds %"alloc::string::String", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %_4) #15
          to label %bb4 unwind label %terminate

bb2:                                              ; preds = %bb4
  %13 = load ptr, ptr %0, align 8
  %14 = getelementptr inbounds i8, ptr %0, i64 8
  %15 = load i32, ptr %14, align 8
  %16 = insertvalue { ptr, i32 } poison, ptr %13, 0
  %17 = insertvalue { ptr, i32 } %16, i32 %15, 1
  resume { ptr, i32 } %17

terminate:                                        ; preds = %bb3
  %18 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable
}

; core::ptr::drop_in_place<std::sys::pal::unix::args::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr52drop_in_place$LT$std..sys..pal..unix..args..Args$GT$17h0fb1c36df737626fE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17hf2231645fa557fe4E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hf358b56a26eab698E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h99196b9ebf8cb15aE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17he9dacd6beb921f1bE"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_3 = alloca [8 x i8], align 8
  store i64 0, ptr %_3, align 8
  br label %bb6

bb6:                                              ; preds = %bb5, %start
  %1 = load i64, ptr %_3, align 8
  %_7 = icmp eq i64 %1, %_1.1
  br i1 %_7, label %bb1, label %bb5

bb5:                                              ; preds = %bb6
  %2 = load i64, ptr %_3, align 8
  %_6 = getelementptr inbounds %"std::ffi::os_str::OsString", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17h1d651ed9cd1d168bE"(ptr align 8 %_6)
          to label %bb6 unwind label %cleanup

bb1:                                              ; preds = %bb6
  ret void

bb4:                                              ; preds = %bb3, %cleanup
  %5 = load i64, ptr %_3, align 8
  %_5 = icmp eq i64 %5, %_1.1
  br i1 %_5, label %bb2, label %bb3

cleanup:                                          ; preds = %bb5
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  br label %bb4

bb3:                                              ; preds = %bb4
  %10 = load i64, ptr %_3, align 8
  %_4 = getelementptr inbounds %"std::ffi::os_str::OsString", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17h1d651ed9cd1d168bE"(ptr align 8 %_4) #15
          to label %bb4 unwind label %terminate

bb2:                                              ; preds = %bb4
  %13 = load ptr, ptr %0, align 8
  %14 = getelementptr inbounds i8, ptr %0, i64 8
  %15 = load i32, ptr %14, align 8
  %16 = insertvalue { ptr, i32 } poison, ptr %13, 0
  %17 = insertvalue { ptr, i32 } %16, i32 %15, 1
  resume { ptr, i32 } %17

terminate:                                        ; preds = %bb3
  %18 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable
}

; core::ptr::drop_in_place<alloc::boxed::Box<[u8]>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h2a1f1da80f5540a3E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6.0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %_6.1 = load i64, ptr %1, align 8
  br label %bb3

bb3:                                              ; preds = %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h84994bb7aae41a85E"(ptr align 8 %_1)
  ret void

bb4:                                              ; No predecessors!
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h84994bb7aae41a85E"(ptr align 8 %_1) #15
          to label %bb1 unwind label %terminate

terminate:                                        ; preds = %bb4
  %2 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb1:                                              ; preds = %bb4
  %5 = load ptr, ptr %0, align 8
  %6 = getelementptr inbounds i8, ptr %0, i64 8
  %7 = load i32, ptr %6, align 8
  %8 = insertvalue { ptr, i32 } poison, ptr %5, 0
  %9 = insertvalue { ptr, i32 } %8, i32 %7, 1
  resume { ptr, i32 } %9
}

; core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17hbd10dcd190b7dadaE"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17haea450922298e9faE"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
  invoke void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h5a9afb923a2f6dd3E"(ptr align 8 %_1) #15
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb4:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
  call void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h5a9afb923a2f6dd3E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb1:                                              ; preds = %bb3
  %8 = load ptr, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  %10 = load i32, ptr %9, align 8
  %11 = insertvalue { ptr, i32 } poison, ptr %8, 0
  %12 = insertvalue { ptr, i32 } %11, i32 %10, 1
  resume { ptr, i32 } %12
}

; core::ptr::drop_in_place<core::option::Option<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr70drop_in_place$LT$core..option..Option$LT$alloc..string..String$GT$$GT$17hb19a1a0f4fa45853E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load i64, ptr %_1, align 8
  %1 = icmp eq i64 %0, -9223372036854775808
  %_2 = select i1 %1, i64 0, i64 1
  %2 = icmp eq i64 %_2, 0
  br i1 %2, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h5a9afb923a2f6dd3E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h964010a1af5225c7E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17hc2d427da46fc894fE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h7e6a25f9e97beb3bE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::rt::lang_start<()>::{{closure}}>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17hf7aefeb47653faafE"(ptr align 8 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17hf2231645fa557fe4E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3ede4dabe4c614cbE"(ptr align 8 %_1)
  ret void
}

; core::ptr::non_null::NonNull<T>::sub_ptr
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$7sub_ptr17h4a6f9cc3cebb074eE"(ptr %self, ptr %subtracted) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
; call core::ptr::const_ptr::<impl *const T>::sub_ptr::precondition_check
  call void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$7sub_ptr18precondition_check17hc0ab9fc34b3cf355E"(ptr %self, ptr %subtracted) #17
  br label %bb4

bb4:                                              ; preds = %bb2
  br label %bb5

bb5:                                              ; preds = %bb4
  br label %bb6

bb6:                                              ; preds = %bb5
  %1 = ptrtoint ptr %self to i64
  %2 = ptrtoint ptr %subtracted to i64
  %3 = sub nuw i64 %1, %2
  %4 = udiv exact i64 %3, 24
  store i64 %4, ptr %0, align 8
  %_0 = load i64, ptr %0, align 8
  ret i64 %_0

bb7:                                              ; No predecessors!
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h9ebd1fefce6d2f82E(ptr align 1 @alloc_ec595fc0e82ef92fc59bd74f68296eae, i64 73, ptr align 8 @alloc_69ac747af8a1bac76e0a9aa808491447) #18
  unreachable
}

; core::ptr::const_ptr::<impl *const T>::sub_ptr::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$7sub_ptr18precondition_check17hc0ab9fc34b3cf355E"(ptr %this, ptr %origin) unnamed_addr #3 {
start:
  %_3 = icmp uge ptr %this, %origin
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_7efb3a7632b3620f628ce83a521b4d9b, i64 71) #19
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; core::hint::assert_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core4hint16assert_unchecked18precondition_check17h3cbe1fe57b48ea08E(i1 zeroext %cond) unnamed_addr #3 {
start:
  br i1 %cond, label %bb2, label %bb1

bb1:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_ab14703751a9eb3585c49b2e55e9a9e5, i64 104) #19
  unreachable

bb2:                                              ; preds = %start
  ret void
}

; core::iter::traits::exact_size::ExactSizeIterator::len
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core4iter6traits10exact_size17ExactSizeIterator3len17had17fef180b42e05E(ptr align 8 %self) unnamed_addr #0 {
start:
  %_9 = alloca [48 x i8], align 8
  %_7 = alloca [1 x i8], align 1
  %_6 = alloca [16 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %upper = alloca [16 x i8], align 8
; call <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
  call void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h2cf6abf74155070cE"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %self)
  %lower = load i64, ptr %_3, align 8
  %0 = getelementptr inbounds i8, ptr %_3, i64 8
  %1 = load i64, ptr %0, align 8
  %2 = getelementptr inbounds i8, ptr %0, i64 8
  %3 = load i64, ptr %2, align 8
  store i64 %1, ptr %upper, align 8
  %4 = getelementptr inbounds i8, ptr %upper, i64 8
  store i64 %3, ptr %4, align 8
  %5 = getelementptr inbounds i8, ptr %_6, i64 8
  store i64 %lower, ptr %5, align 8
  store i64 1, ptr %_6, align 8
  %_12 = load i64, ptr %upper, align 8
  %6 = icmp eq i64 %_12, 0
  br i1 %6, label %bb7, label %bb6

bb7:                                              ; preds = %start
  %_11 = load i64, ptr %_6, align 8
  %7 = icmp eq i64 %_11, 0
  %8 = zext i1 %7 to i8
  store i8 %8, ptr %_7, align 1
  br label %bb4

bb6:                                              ; preds = %start
  %_10 = load i64, ptr %_6, align 8
  %9 = icmp eq i64 %_10, 0
  br i1 %9, label %bb8, label %bb9

bb4:                                              ; preds = %bb9, %bb7
  %10 = load i8, ptr %_7, align 1
  %11 = trunc i8 %10 to i1
  br i1 %11, label %bb2, label %bb3

bb8:                                              ; preds = %bb6
  br label %bb3

bb9:                                              ; preds = %bb6
  %l = getelementptr inbounds i8, ptr %upper, i64 8
  %r = getelementptr inbounds i8, ptr %_6, i64 8
  %12 = getelementptr inbounds i8, ptr %upper, i64 8
  %_15 = load i64, ptr %12, align 8
  %13 = getelementptr inbounds i8, ptr %_6, i64 8
  %_16 = load i64, ptr %13, align 8
  %14 = icmp eq i64 %_15, %_16
  %15 = zext i1 %14 to i8
  store i8 %15, ptr %_7, align 1
  br label %bb4

bb3:                                              ; preds = %bb4, %bb8
  store ptr null, ptr %_9, align 8
; call core::panicking::assert_failed
  call void @_ZN4core9panicking13assert_failed17h1c688646bbe431b3E(i8 0, ptr align 8 %upper, ptr align 8 %_6, ptr align 8 %_9, ptr align 8 @alloc_f9a12aa962eb349e0d12fd6ea34d8b62) #18
  unreachable

bb2:                                              ; preds = %bb4
  ret i64 %lower

bb5:                                              ; No predecessors!
  unreachable
}

; core::iter::traits::iterator::Iterator::collect
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core4iter6traits8iterator8Iterator7collect17hcc1ce397e3bf5996E(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
; call <alloc::vec::Vec<T> as core::iter::traits::collect::FromIterator<T>>::from_iter
  call void @"_ZN95_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$core..iter..traits..collect..FromIterator$LT$T$GT$$GT$9from_iter17hf6c42df3a935ffabE"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self, ptr align 8 @alloc_31ba853be958e604da5cf95140670bc2)
  ret void
}

; core::alloc::layout::Layout::from_size_align_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5alloc6layout6Layout25from_size_align_unchecked18precondition_check17ha85b89112662adabE(i64 %size, i64 %align) unnamed_addr #3 personality ptr @rust_eh_personality {
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
  call void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() #16
  unreachable

bb1:                                              ; preds = %start
  br i1 %_3, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_cd1513ae8d1ae22acf9342b8dfa1561d, i64 164) #19
  unreachable

bb2:                                              ; preds = %bb1
  ret void
}

; core::slice::raw::from_raw_parts::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h9899a5128162cb38E(ptr %data, i64 %size, i64 %align, i64 %len) unnamed_addr #3 personality ptr @rust_eh_personality {
start:
  %0 = alloca [4 x i8], align 4
  %max_len = alloca [8 x i8], align 8
  %_11 = alloca [48 x i8], align 8
  %1 = call i64 @llvm.ctpop.i64(i64 %align)
  %2 = trunc i64 %1 to i32
  store i32 %2, ptr %0, align 4
  %_15 = load i32, ptr %0, align 4
  %3 = icmp eq i32 %_15, 1
  br i1 %3, label %bb8, label %bb9

bb8:                                              ; preds = %start
  %_13 = ptrtoint ptr %data to i64
  %_14 = sub i64 %align, 1
  %_12 = and i64 %_13, %_14
  %4 = icmp eq i64 %_12, 0
  br i1 %4, label %bb6, label %bb7

bb9:                                              ; preds = %start
  store ptr @alloc_e92e94d0ff530782b571cfd99ec66aef, ptr %_11, align 8
  %5 = getelementptr inbounds i8, ptr %_11, i64 8
  store i64 1, ptr %5, align 8
  %6 = load ptr, ptr @0, align 8
  %7 = load i64, ptr getelementptr inbounds (i8, ptr @0, i64 8), align 8
  %8 = getelementptr inbounds i8, ptr %_11, i64 32
  store ptr %6, ptr %8, align 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  store i64 %7, ptr %9, align 8
  %10 = getelementptr inbounds i8, ptr %_11, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %10, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 8
  store i64 0, ptr %11, align 8
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h169d7389ef09f3baE(ptr align 8 %_11, ptr align 8 @alloc_369fae759a17104aba57fba3370bcbfb) #18
          to label %unreachable unwind label %terminate

bb6:                                              ; preds = %bb8
  %_9 = icmp eq i64 %_13, 0
  %_5 = xor i1 %_9, true
  br i1 %_5, label %bb1, label %bb4

bb7:                                              ; preds = %bb8
  br label %bb4

bb4:                                              ; preds = %bb7, %bb6
  br label %bb5

bb1:                                              ; preds = %bb6
  %_19 = icmp eq i64 %size, 0
  %12 = icmp eq i64 %size, 0
  br i1 %12, label %bb11, label %bb12

bb11:                                             ; preds = %bb1
  store i64 -1, ptr %max_len, align 8
  br label %bb14

bb12:                                             ; preds = %bb1
  br i1 %_19, label %panic, label %bb13

bb14:                                             ; preds = %bb13, %bb11
  %_20 = load i64, ptr %max_len, align 8
  %_7 = icmp ule i64 %len, %_20
  br i1 %_7, label %bb2, label %bb3

bb13:                                             ; preds = %bb12
  %13 = udiv i64 9223372036854775807, %size
  store i64 %13, ptr %max_len, align 8
  br label %bb14

panic:                                            ; preds = %bb12
; invoke core::panicking::panic_const::panic_const_div_by_zero
  invoke void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17he9029e02cc767683E(ptr align 8 @alloc_ccfb06f4f0e3e21a92aa1cae417f3225) #18
          to label %unreachable unwind label %terminate

terminate:                                        ; preds = %bb9, %panic
  %14 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
; call core::panicking::panic_cannot_unwind
  call void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() #16
  unreachable

unreachable:                                      ; preds = %bb9, %panic
  unreachable

bb3:                                              ; preds = %bb14
  br label %bb5

bb2:                                              ; preds = %bb14
  ret void

bb5:                                              ; preds = %bb4, %bb3
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_11195730e5236cfdc15ea13be1c301f9, i64 162) #19
  unreachable
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hcbdd2bc10105ff4fE"() unnamed_addr #0 {
start:
  ret i8 0
}

; alloc::ffi::c_str::CString::new
; Function Attrs: uwtable
define internal void @_ZN5alloc3ffi5c_str7CString3new17h0b21ac9c55cfc3d7E(ptr sret([32 x i8]) align 8 %_0, ptr align 1 %t.0, i64 %t.1) unnamed_addr #1 {
start:
; call <&str as alloc::ffi::c_str::CString::new::SpecNewImpl>::spec_new_impl
  call void @"_ZN72_$LT$$RF$str$u20$as$u20$alloc..ffi..c_str..CString..new..SpecNewImpl$GT$13spec_new_impl17hdf17333e3fe57099E"(ptr sret([32 x i8]) align 8 %_0, ptr align 1 %t.0, i64 %t.1)
  ret void
}

; alloc::vec::Vec<T,A>::extend_desugared
; Function Attrs: uwtable
define internal void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16extend_desugared17hf6124153ce5728ecE"(ptr align 8 %self, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [8 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %src = alloca [24 x i8], align 8
  %_11 = alloca [24 x i8], align 8
  %_9 = alloca [8 x i8], align 8
  %element = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  br label %bb1

bb1:                                              ; preds = %bb8, %start
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::next
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h7da987a022c3da75E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %iterator)
          to label %bb2 unwind label %cleanup.loopexit

bb12:                                             ; preds = %bb14, %cleanup
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17h0e0f28defd111449E"(ptr align 8 %iterator) #15
          to label %bb13 unwind label %terminate

cleanup.loopexit:                                 ; preds = %bb1
  %lpad.loopexit = landingpad { ptr, i32 }
          cleanup
  br label %cleanup

cleanup.loopexit.split-lp:                        ; preds = %bb9
  %lpad.loopexit.split-lp = landingpad { ptr, i32 }
          cleanup
  br label %cleanup

cleanup:                                          ; preds = %cleanup.loopexit.split-lp, %cleanup.loopexit
  %lpad.phi = phi { ptr, i32 } [ %lpad.loopexit, %cleanup.loopexit ], [ %lpad.loopexit.split-lp, %cleanup.loopexit.split-lp ]
  %3 = extractvalue { ptr, i32 } %lpad.phi, 0
  %4 = extractvalue { ptr, i32 } %lpad.phi, 1
  store ptr %3, ptr %2, align 8
  %5 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb12

bb2:                                              ; preds = %bb1
  %6 = load i64, ptr %_3, align 8
  %7 = icmp eq i64 %6, -9223372036854775808
  %_5 = select i1 %7, i64 0, i64 1
  %8 = icmp eq i64 %_5, 1
  br i1 %8, label %bb3, label %bb9

bb3:                                              ; preds = %bb2
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %element, ptr align 8 %_3, i64 24, i1 false)
  %9 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %9, align 8
  %_19 = icmp ule i64 %len, 384307168202282325
  br label %bb17

bb9:                                              ; preds = %bb2
; invoke core::ptr::drop_in_place<core::option::Option<alloc::string::String>>
  invoke void @"_ZN4core3ptr70drop_in_place$LT$core..option..Option$LT$alloc..string..String$GT$$GT$17hb19a1a0f4fa45853E"(ptr align 8 %_3)
          to label %bb10 unwind label %cleanup.loopexit.split-lp

bb17:                                             ; preds = %bb3
  %10 = load i64, ptr %self, align 8
  store i64 %10, ptr %_9, align 8
  br label %bb15

bb16:                                             ; No predecessors!
  store i64 -1, ptr %_9, align 8
  unreachable

bb15:                                             ; preds = %bb17
  %11 = load i64, ptr %_9, align 8
  %_8 = icmp eq i64 %len, %11
  br i1 %_8, label %bb4, label %bb7

bb7:                                              ; preds = %bb15
  br label %bb8

bb4:                                              ; preds = %bb15
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h834281278c633905E"(ptr sret([24 x i8]) align 8 %_11, ptr align 8 %iterator)
          to label %bb5 unwind label %cleanup1

bb8:                                              ; preds = %bb6, %bb7
  %12 = getelementptr inbounds i8, ptr %self, i64 8
  %_26 = load ptr, ptr %12, align 8
  %dst = getelementptr inbounds %"alloc::string::String", ptr %_26, i64 %len
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %src, ptr align 8 %element, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %dst, ptr align 8 %src, i64 24, i1 false)
  %new_len = add i64 %len, 1
  %13 = getelementptr inbounds i8, ptr %self, i64 16
  store i64 %new_len, ptr %13, align 8
  br label %bb1

bb14:                                             ; preds = %cleanup1
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he5865ad3631aaf25E"(ptr align 8 %element) #15
          to label %bb12 unwind label %terminate

cleanup1:                                         ; preds = %bb5, %bb4
  %14 = landingpad { ptr, i32 }
          cleanup
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
  store ptr %15, ptr %2, align 8
  %17 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %16, ptr %17, align 8
  br label %bb14

bb5:                                              ; preds = %bb4
  %lower = load i64, ptr %_11, align 8
  %18 = call i64 @llvm.uadd.sat.i64(i64 %lower, i64 1)
  store i64 %18, ptr %1, align 8
  %_14 = load i64, ptr %1, align 8
; invoke alloc::vec::Vec<T,A>::reserve
  invoke void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h7a5069952c7a1e2dE"(ptr align 8 %self, i64 %_14, ptr align 8 %0)
          to label %bb6 unwind label %cleanup1

bb6:                                              ; preds = %bb5
  br label %bb8

terminate:                                        ; preds = %bb12, %bb14
  %19 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %20 = extractvalue { ptr, i32 } %19, 0
  %21 = extractvalue { ptr, i32 } %19, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb10:                                             ; preds = %bb9
; call core::ptr::drop_in_place<std::env::Args>
  call void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17h0e0f28defd111449E"(ptr align 8 %iterator)
  ret void

bb19:                                             ; No predecessors!
  unreachable

bb13:                                             ; preds = %bb12
  %22 = load ptr, ptr %2, align 8
  %23 = getelementptr inbounds i8, ptr %2, i64 8
  %24 = load i32, ptr %23, align 8
  %25 = insertvalue { ptr, i32 } poison, ptr %22, 0
  %26 = insertvalue { ptr, i32 } %25, i32 %24, 1
  resume { ptr, i32 } %26
}

; alloc::vec::Vec<T,A>::len
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h117b283c26c9a7ebE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 16
  %_0 = load i64, ptr %0, align 8
  %_2 = icmp ule i64 %_0, 384307168202282325
  ret i64 %_0
}

; alloc::vec::Vec<T,A>::reserve
; Function Attrs: uwtable
define internal void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h7a5069952c7a1e2dE"(ptr align 8 %self, i64 %additional, ptr align 8 %0) unnamed_addr #1 {
start:
  %self1 = alloca [8 x i8], align 8
  %elem_layout = alloca [16 x i8], align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  store i64 8, ptr %elem_layout, align 8
  %2 = getelementptr inbounds i8, ptr %elem_layout, i64 8
  store i64 24, ptr %2, align 8
  br label %bb6

bb6:                                              ; preds = %start
  %3 = load i64, ptr %self, align 8
  store i64 %3, ptr %self1, align 8
  br label %bb4

bb5:                                              ; No predecessors!
  store i64 -1, ptr %self1, align 8
  br label %bb4

bb4:                                              ; preds = %bb6, %bb5
  %4 = load i64, ptr %self1, align 8
  %_10 = sub i64 %4, %len
  %_7 = icmp ugt i64 %additional, %_10
  br i1 %_7, label %bb1, label %bb2

bb2:                                              ; preds = %bb4
  br label %bb3

bb1:                                              ; preds = %bb4
; call alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h32f2f6d36a81d018E"(ptr align 8 %self, i64 %len, i64 %additional, i64 8, i64 24)
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  ret void
}

; alloc::string::String::as_str
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @_ZN5alloc6string6String6as_str17ha1621dfb55efaeedE(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_10 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h9899a5128162cb38E(ptr %_10, i64 1, i64 1, i64 %len) #17
  br label %bb3

bb3:                                              ; preds = %bb1
  %2 = insertvalue { ptr, i64 } poison, ptr %_10, 0
  %3 = insertvalue { ptr, i64 } %2, i64 %len, 1
  ret { ptr, i64 } %3
}

; alloc::raw_vec::RawVecInner<A>::with_capacity_in
; Function Attrs: inlinehint uwtable
define internal { i64, ptr } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$16with_capacity_in17h3f0b3cda90ff39b9E"(i64 %capacity, i64 %elem_layout.0, i64 %elem_layout.1, ptr align 8 %0) unnamed_addr #0 {
start:
  %self = alloca [8 x i8], align 8
  %elem_layout = alloca [16 x i8], align 8
  %this = alloca [16 x i8], align 8
  %_4 = alloca [24 x i8], align 8
; call alloc::raw_vec::RawVecInner<A>::try_allocate_in
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$15try_allocate_in17h6618b9d04215e743E"(ptr sret([24 x i8]) align 8 %_4, i64 %capacity, i1 zeroext false, i64 %elem_layout.0, i64 %elem_layout.1)
  %_5 = load i64, ptr %_4, align 8
  %1 = icmp eq i64 %_5, 0
  br i1 %1, label %bb4, label %bb3

bb4:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %_4, i64 8
  %3 = load i64, ptr %2, align 8
  %4 = getelementptr inbounds i8, ptr %2, i64 8
  %5 = load ptr, ptr %4, align 8
  store i64 %3, ptr %this, align 8
  %6 = getelementptr inbounds i8, ptr %this, i64 8
  store ptr %5, ptr %6, align 8
  store i64 %elem_layout.0, ptr %elem_layout, align 8
  %7 = getelementptr inbounds i8, ptr %elem_layout, i64 8
  store i64 %elem_layout.1, ptr %7, align 8
  %8 = icmp eq i64 %elem_layout.1, 0
  br i1 %8, label %bb6, label %bb7

bb3:                                              ; preds = %start
  %9 = getelementptr inbounds i8, ptr %_4, i64 8
  %err.0 = load i64, ptr %9, align 8
  %10 = getelementptr inbounds i8, ptr %9, i64 8
  %err.1 = load i64, ptr %10, align 8
; call alloc::raw_vec::handle_error
  call void @_ZN5alloc7raw_vec12handle_error17h7f36dc445c6de4b2E(i64 %err.0, i64 %err.1, ptr align 8 %0) #18
  unreachable

bb6:                                              ; preds = %bb4
  store i64 -1, ptr %self, align 8
  br label %bb5

bb7:                                              ; preds = %bb4
  %11 = load i64, ptr %this, align 8
  store i64 %11, ptr %self, align 8
  br label %bb5

bb5:                                              ; preds = %bb7, %bb6
  %12 = load i64, ptr %self, align 8
  %_13 = sub i64 %12, 0
  %_8 = icmp ugt i64 %capacity, %_13
  %cond = xor i1 %_8, true
  br label %bb8

bb8:                                              ; preds = %bb5
; call core::hint::assert_unchecked::precondition_check
  call void @_ZN4core4hint16assert_unchecked18precondition_check17h3cbe1fe57b48ea08E(i1 zeroext %cond) #17
  br label %bb9

bb9:                                              ; preds = %bb8
  %_0.0 = load i64, ptr %this, align 8
  %13 = getelementptr inbounds i8, ptr %this, i64 8
  %_0.1 = load ptr, ptr %13, align 8
  %14 = insertvalue { i64, ptr } poison, i64 %_0.0, 0
  %15 = insertvalue { i64, ptr } %14, ptr %_0.1, 1
  ret { i64, ptr } %15

bb2:                                              ; No predecessors!
  unreachable
}

; <I as core::iter::traits::collect::IntoIterator>::into_iter
; Function Attrs: inlinehint uwtable
define internal void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h32173405814f58dcE"(ptr sret([32 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %self, i64 32, i1 false)
  ret void
}

; <I as core::iter::traits::collect::IntoIterator>::into_iter
; Function Attrs: inlinehint uwtable
define internal { ptr, ptr } @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17hf527dfa6a941dddfE"(ptr %self.0, ptr %self.1) unnamed_addr #0 {
start:
  %0 = insertvalue { ptr, ptr } poison, ptr %self.0, 0
  %1 = insertvalue { ptr, ptr } %0, ptr %self.1, 1
  ret { ptr, ptr } %1
}

; <alloc::alloc::Global as core::alloc::Allocator>::deallocate
; Function Attrs: inlinehint uwtable
define internal void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h37e234ed6827a0ffE"(ptr align 1 %self, ptr %ptr, i64 %0, i64 %1) unnamed_addr #0 {
start:
  %_12 = alloca [8 x i8], align 8
  %layout1 = alloca [16 x i8], align 8
  %layout = alloca [16 x i8], align 8
  store i64 %0, ptr %layout, align 8
  %2 = getelementptr inbounds i8, ptr %layout, i64 8
  store i64 %1, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %layout, i64 8
  %_4 = load i64, ptr %3, align 8
  %4 = icmp eq i64 %_4, 0
  br i1 %4, label %bb2, label %bb1

bb2:                                              ; preds = %bb1, %start
  ret void

bb1:                                              ; preds = %start
  %5 = load i64, ptr %layout, align 8
  %6 = getelementptr inbounds i8, ptr %layout, i64 8
  %7 = load i64, ptr %6, align 8
  store i64 %5, ptr %layout1, align 8
  %8 = getelementptr inbounds i8, ptr %layout1, i64 8
  store i64 %7, ptr %8, align 8
  %_11 = load i64, ptr %layout, align 8
  store i64 %_11, ptr %_12, align 8
  %_13 = load i64, ptr %_12, align 8
  %_14 = icmp uge i64 %_13, 1
  %_15 = icmp ule i64 %_13, -9223372036854775808
  %_16 = and i1 %_14, %_15
  call void @__rust_dealloc(ptr %ptr, i64 %_4, i64 %_13) #17
  br label %bb2
}

; <alloc::ffi::c_str::NulError as core::fmt::Debug>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN64_$LT$alloc..ffi..c_str..NulError$u20$as$u20$core..fmt..Debug$GT$3fmt17h998d0b57d1ebbee1E"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #0 {
start:
  %_7 = alloca [8 x i8], align 8
  %_4 = getelementptr inbounds i8, ptr %self, i64 24
  store ptr %self, ptr %_7, align 8
; call core::fmt::Formatter::debug_tuple_field2_finish
  %_0 = call zeroext i1 @_ZN4core3fmt9Formatter25debug_tuple_field2_finish17h46634f1938cd4c84E(ptr align 8 %f, ptr align 1 @alloc_49c0eff15ce41ce22a2d8c8b146a94ef, i64 8, ptr align 1 %_4, ptr align 8 @vtable.3, ptr align 1 %_7, ptr align 8 @vtable.4)
  ret i1 %_0
}

; <alloc::vec::Vec<T,A> as core::fmt::Debug>::fmt
; Function Attrs: uwtable
define internal zeroext i1 @"_ZN65_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h8c4fdc0f776f2397E"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_10 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  br label %bb2

bb2:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h9899a5128162cb38E(ptr %_10, i64 1, i64 1, i64 %len) #17
  br label %bb4

bb4:                                              ; preds = %bb2
; call <[T] as core::fmt::Debug>::fmt
  %_0 = call zeroext i1 @"_ZN48_$LT$$u5b$T$u5d$$u20$as$u20$core..fmt..Debug$GT$3fmt17h2c7a11b238c7fa5fE"(ptr align 1 %_10, i64 %len, ptr align 8 %f)
  ret i1 %_0
}

; <alloc::ffi::c_str::CString as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN68_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3ccfd437ff471786E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %_2.0 = load ptr, ptr %self, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_2.1 = load i64, ptr %0, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hf79c21da3f1c9f2dE"(i64 0, i64 %_2.1) #17
  br label %bb3

bb3:                                              ; preds = %bb1
  store i8 0, ptr %_2.0, align 1
  ret void
}

; <alloc::ffi::c_str::CString as core::ops::deref::Deref>::deref
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @"_ZN70_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..deref..Deref$GT$5deref17he09c9d1305d1f153E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %_4.0 = load ptr, ptr %self, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_4.1 = load i64, ptr %0, align 8
  %1 = insertvalue { ptr, i64 } poison, ptr %_4.0, 0
  %2 = insertvalue { ptr, i64 } %1, i64 %_4.1, 1
  ret { ptr, i64 } %2
}

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17haea450922298e9faE"(ptr align 8 %self) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_8 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
; call core::ptr::drop_in_place<[alloc::string::String]>
  call void @"_ZN4core3ptr52drop_in_place$LT$$u5b$alloc..string..String$u5d$$GT$17h1f0f1e6fadba3fdeE"(ptr align 8 %_8, i64 %len)
  ret void
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h84994bb7aae41a85E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %layout = alloca [16 x i8], align 8
  %ptr.0 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  %ptr.1 = load i64, ptr %2, align 8
  %3 = mul nsw i64 %ptr.1, 1
  store i64 %3, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  %4 = mul nsw i64 %ptr.1, 1
  store i64 1, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::alloc::layout::Layout::from_size_align_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout25from_size_align_unchecked18precondition_check17ha85b89112662adabE(i64 %size, i64 %align) #17
  br label %bb7

bb7:                                              ; preds = %bb6
  %5 = getelementptr inbounds i8, ptr %layout, i64 8
  store i64 %size, ptr %5, align 8
  store i64 %align, ptr %layout, align 8
  %6 = icmp eq i64 %size, 0
  br i1 %6, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb7
  ret void

bb1:                                              ; preds = %bb7
  %_7 = getelementptr inbounds i8, ptr %self, i64 16
  %7 = load i64, ptr %layout, align 8
  %8 = getelementptr inbounds i8, ptr %layout, i64 8
  %9 = load i64, ptr %8, align 8
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h37e234ed6827a0ffE"(ptr align 1 %_7, ptr %ptr.0, i64 %7, i64 %9)
  br label %bb3
}

; <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hf79c21da3f1c9f2dE"(i64 %this, i64 %len) unnamed_addr #3 {
start:
  %_3 = icmp ult i64 %this, %len
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1 @alloc_e96fb6e25c55edb0aec8b24d111f5d7f, i64 101) #19
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; <usize as core::slice::index::SliceIndex<[T]>>::index
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17h70053014d4131092E"(i64 %self, ptr align 8 %slice.0, i64 %slice.1, ptr align 8 %0) unnamed_addr #0 {
start:
  %_4 = icmp ult i64 %self, %slice.1
  br i1 %_4, label %bb1, label %panic

bb1:                                              ; preds = %start
  %_0 = getelementptr inbounds %"alloc::string::String", ptr %slice.0, i64 %self
  ret ptr %_0

panic:                                            ; preds = %start
; call core::panicking::panic_bounds_check
  call void @_ZN4core9panicking18panic_bounds_check17h899c84cea2bff5b8E(i64 %self, i64 %slice.1, ptr align 8 %0) #18
  unreachable
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h7e6a25f9e97beb3bE"(ptr align 8 %self) unnamed_addr #1 {
start:
; call alloc::raw_vec::RawVecInner<A>::deallocate
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h3287a4638d2822faE"(ptr align 8 %self, i64 8, i64 24)
  ret void
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h964010a1af5225c7E"(ptr align 8 %self) unnamed_addr #1 {
start:
; call alloc::raw_vec::RawVecInner<A>::deallocate
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h3287a4638d2822faE"(ptr align 8 %self, i64 8, i64 24)
  ret void
}

; <alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN81_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..index..Index$LT$I$GT$$GT$5index17h46425518ac4dbef7E"(ptr align 8 %self, i64 %index, ptr align 8 %0) unnamed_addr #0 {
start:
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %_10 = load ptr, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %2, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h9899a5128162cb38E(ptr %_10, i64 24, i64 8, i64 %len) #17
  br label %bb3

bb3:                                              ; preds = %bb1
; call <usize as core::slice::index::SliceIndex<[T]>>::index
  %_0 = call align 8 ptr @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17h70053014d4131092E"(i64 %index, ptr align 8 %_10, i64 %len, ptr align 8 %0)
  ret ptr %_0
}

; <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3ede4dabe4c614cbE"(ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %self1 = alloca [8 x i8], align 8
  %guard = alloca [8 x i8], align 8
  store ptr %self, ptr %guard, align 8
  %_6 = load ptr, ptr %guard, align 8
  store ptr %_6, ptr %self1, align 8
  %1 = getelementptr inbounds i8, ptr %_6, i64 8
  %self2 = load ptr, ptr %1, align 8
; invoke core::iter::traits::exact_size::ExactSizeIterator::len
  %len = invoke i64 @_ZN4core4iter6traits10exact_size17ExactSizeIterator3len17had17fef180b42e05E(ptr align 8 %_6)
          to label %bb5 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  invoke void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17h9ea33ad1e2a233ebE"(ptr align 8 %guard) #15
          to label %bb4 unwind label %terminate

cleanup:                                          ; preds = %bb5, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb3

bb5:                                              ; preds = %start
; invoke core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
  invoke void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17he9dacd6beb921f1bE"(ptr align 8 %self2, i64 %len)
          to label %bb1 unwind label %cleanup

bb1:                                              ; preds = %bb5
; call core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  call void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17h9ea33ad1e2a233ebE"(ptr align 8 %guard)
  ret void

terminate:                                        ; preds = %bb3
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb4:                                              ; preds = %bb3
  %9 = load ptr, ptr %0, align 8
  %10 = getelementptr inbounds i8, ptr %0, i64 8
  %11 = load i32, ptr %10, align 8
  %12 = insertvalue { ptr, i32 } poison, ptr %9, 0
  %13 = insertvalue { ptr, i32 } %12, i32 %11, 1
  resume { ptr, i32 } %13
}

; <core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next
; Function Attrs: inlinehint uwtable
define internal align 1 ptr @"_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h19b501c5fbbb2d0aE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %old = alloca [8 x i8], align 8
  %end = alloca [8 x i8], align 8
  %_2 = alloca [1 x i8], align 1
  %_0 = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
  %self1 = getelementptr inbounds i8, ptr %self, i64 8
  %0 = load ptr, ptr %self1, align 8
  store ptr %0, ptr %end, align 8
  %_12 = load ptr, ptr %self, align 8
  %_13 = load ptr, ptr %end, align 8
  %1 = icmp eq ptr %_12, %_13
  %2 = zext i1 %1 to i8
  store i8 %2, ptr %_2, align 1
  br label %bb3

bb3:                                              ; preds = %bb2
  %3 = load i8, ptr %_2, align 1
  %4 = trunc i8 %3 to i1
  br i1 %4, label %bb4, label %bb5

bb5:                                              ; preds = %bb3
  %5 = load ptr, ptr %self, align 8
  store ptr %5, ptr %old, align 8
  br label %bb9

bb4:                                              ; preds = %bb3
  store ptr null, ptr %_0, align 8
  br label %bb6

bb9:                                              ; preds = %bb5
  %self2 = getelementptr inbounds i8, ptr %self, i64 8
  %self3 = load ptr, ptr %self, align 8
  %_24 = getelementptr inbounds i8, ptr %self3, i64 1
  store ptr %_24, ptr %self, align 8
  br label %bb7

bb7:                                              ; preds = %bb9
  %_28 = load ptr, ptr %old, align 8
  store ptr %_28, ptr %_0, align 8
  br label %bb6

bb6:                                              ; preds = %bb4, %bb7
  %6 = load ptr, ptr %_0, align 8
  ret ptr %6

bb1:                                              ; No predecessors!
  unreachable

bb8:                                              ; No predecessors!
  unreachable
}

; <alloc::vec::Vec<T> as core::iter::traits::collect::FromIterator<T>>::from_iter
; Function Attrs: inlinehint uwtable
define internal void @"_ZN95_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$core..iter..traits..collect..FromIterator$LT$T$GT$$GT$9from_iter17hf6c42df3a935ffabE"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iter, ptr align 8 %0) unnamed_addr #0 {
start:
  %_2 = alloca [32 x i8], align 8
; call <I as core::iter::traits::collect::IntoIterator>::into_iter
  call void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h32173405814f58dcE"(ptr sret([32 x i8]) align 8 %_2, ptr align 8 %iter)
; call <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter
  call void @"_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h747eb3a72772f638E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %_2, ptr align 8 %0)
  ret void
}

; <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<T,I>>::spec_extend
; Function Attrs: uwtable
define internal void @"_ZN97_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$T$C$I$GT$$GT$11spec_extend17hf8af74ff453fff59E"(ptr align 8 %self, ptr align 8 %iter, ptr align 8 %0) unnamed_addr #1 {
start:
; call alloc::vec::Vec<T,A>::extend_desugared
  call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16extend_desugared17hf6124153ce5728ecE"(ptr align 8 %self, ptr align 8 %iter, ptr align 8 %0)
  ret void
}

; <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter
; Function Attrs: uwtable
define internal void @"_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h747eb3a72772f638E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 {
start:
; call <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
  call void @"_ZN111_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter_nested..SpecFromIterNested$LT$T$C$I$GT$$GT$9from_iter17h8dbb3058bb4d5433E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0)
  ret void
}

; taint_unsafe::main
; Function Attrs: uwtable
define internal void @_ZN12taint_unsafe4main17h1b017c1895fa0b30E() unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %e.i = alloca [32 x i8], align 8
  %1 = alloca [16 x i8], align 8
  %_9 = alloca [32 x i8], align 8
  %c_cmd = alloca [16 x i8], align 8
  %_2 = alloca [32 x i8], align 8
  %args = alloca [24 x i8], align 8
; call std::env::args
  call void @_ZN3std3env4args17hcb0b85f54a96d99cE(ptr sret([32 x i8]) align 8 %_2)
; call core::iter::traits::iterator::Iterator::collect
  call void @_ZN4core4iter6traits8iterator8Iterator7collect17hcc1ce397e3bf5996E(ptr sret([24 x i8]) align 8 %args, ptr align 8 %_2)
; invoke alloc::vec::Vec<T,A>::len
  %_4 = invoke i64 @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h117b283c26c9a7ebE"(ptr align 8 %args)
          to label %bb3 unwind label %cleanup

bb16:                                             ; preds = %bb15, %cleanup.body
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  invoke void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17hbd10dcd190b7dadaE"(ptr align 8 %args) #15
          to label %bb17 unwind label %terminate

cleanup:                                          ; preds = %bb11, %bb7, %bb6, %bb5, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  br label %cleanup.body

cleanup.body:                                     ; preds = %bb5.i, %cleanup
  %eh.lpad-body = phi { ptr, i32 } [ %2, %cleanup ], [ %20, %bb5.i ]
  %3 = extractvalue { ptr, i32 } %eh.lpad-body, 0
  %4 = extractvalue { ptr, i32 } %eh.lpad-body, 1
  store ptr %3, ptr %1, align 8
  %5 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb16

bb3:                                              ; preds = %start
  %_3 = icmp ult i64 %_4, 2
  br i1 %_3, label %bb4, label %bb5

bb5:                                              ; preds = %bb3
; invoke <alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
  %cmd = invoke align 8 ptr @"_ZN81_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..index..Index$LT$I$GT$$GT$5index17h46425518ac4dbef7E"(ptr align 8 %args, i64 1, ptr align 8 @alloc_425effdcb2ac5d66167160ce84f7c716)
          to label %bb6 unwind label %cleanup

bb4:                                              ; preds = %bb3
; call core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  call void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17hbd10dcd190b7dadaE"(ptr align 8 %args)
  br label %bb14

bb6:                                              ; preds = %bb5
; invoke alloc::string::String::as_str
  %6 = invoke { ptr, i64 } @_ZN5alloc6string6String6as_str17ha1621dfb55efaeedE(ptr align 8 %cmd)
          to label %bb7 unwind label %cleanup

bb7:                                              ; preds = %bb6
  %_10.0 = extractvalue { ptr, i64 } %6, 0
  %_10.1 = extractvalue { ptr, i64 } %6, 1
; invoke alloc::ffi::c_str::CString::new
  invoke void @_ZN5alloc3ffi5c_str7CString3new17h0b21ac9c55cfc3d7E(ptr sret([32 x i8]) align 8 %_9, ptr align 1 %_10.0, i64 %_10.1)
          to label %bb8 unwind label %cleanup

bb8:                                              ; preds = %bb7
  %7 = load i64, ptr %_9, align 8
  %8 = icmp eq i64 %7, -9223372036854775808
  %_2.i = select i1 %8, i64 0, i64 1
  br i1 %8, label %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h478593894b7e39b8E.exit", label %bb2.i

bb2.i:                                            ; preds = %bb8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %e.i, ptr align 8 %_9, i64 32, i1 false)
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17ha5a2c69de0fb8105E(ptr align 1 @alloc_00ae4b301f7fab8ac9617c03fcbd7274, i64 43, ptr align 1 %e.i, ptr align 8 @vtable.2, ptr align 8 @alloc_4f05f4a8f7ae5dff1b6009babd7dab65) #18
          to label %unreachable.i unwind label %cleanup.i

cleanup.i:                                        ; preds = %bb2.i
  %9 = landingpad { ptr, i32 }
          cleanup
  %10 = extractvalue { ptr, i32 } %9, 0
  %11 = extractvalue { ptr, i32 } %9, 1
  store ptr %10, ptr %0, align 8
  %12 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %11, ptr %12, align 8
; invoke core::ptr::drop_in_place<alloc::ffi::c_str::NulError>
  invoke void @"_ZN4core3ptr48drop_in_place$LT$alloc..ffi..c_str..NulError$GT$17hed00ef0c138c90cfE"(ptr align 8 %e.i) #15
          to label %bb5.i unwind label %terminate.i

unreachable.i:                                    ; preds = %bb2.i
  unreachable

terminate.i:                                      ; preds = %cleanup.i
  %13 = landingpad { ptr, i32 }
          cleanup
          filter [0 x ptr] zeroinitializer
  %14 = extractvalue { ptr, i32 } %13, 0
  %15 = extractvalue { ptr, i32 } %13, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb5.i:                                            ; preds = %cleanup.i
  %16 = load ptr, ptr %0, align 8
  %17 = getelementptr inbounds i8, ptr %0, i64 8
  %18 = load i32, ptr %17, align 8
  %19 = insertvalue { ptr, i32 } poison, ptr %16, 0
  %20 = insertvalue { ptr, i32 } %19, i32 %18, 1
  br label %cleanup.body

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h478593894b7e39b8E.exit": ; preds = %bb8
  %21 = getelementptr inbounds i8, ptr %_9, i64 8
  %t.0.i = load ptr, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 8
  %t.1.i = load i64, ptr %22, align 8
  %23 = insertvalue { ptr, i64 } poison, ptr %t.0.i, 0
  %24 = insertvalue { ptr, i64 } %23, i64 %t.1.i, 1
  br label %bb9

bb9:                                              ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h478593894b7e39b8E.exit"
  %25 = extractvalue { ptr, i64 } %24, 0
  %26 = extractvalue { ptr, i64 } %24, 1
  store ptr %25, ptr %c_cmd, align 8
  %27 = getelementptr inbounds i8, ptr %c_cmd, i64 8
  store i64 %26, ptr %27, align 8
; invoke <alloc::ffi::c_str::CString as core::ops::deref::Deref>::deref
  %28 = invoke { ptr, i64 } @"_ZN70_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..deref..Deref$GT$5deref17he09c9d1305d1f153E"(ptr align 8 %c_cmd)
          to label %bb10 unwind label %cleanup1

bb15:                                             ; preds = %cleanup1
; invoke core::ptr::drop_in_place<alloc::ffi::c_str::CString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h8922e58224b554a9E"(ptr align 8 %c_cmd) #15
          to label %bb16 unwind label %terminate

cleanup1:                                         ; preds = %bb10, %bb9
  %29 = landingpad { ptr, i32 }
          cleanup
  %30 = extractvalue { ptr, i32 } %29, 0
  %31 = extractvalue { ptr, i32 } %29, 1
  store ptr %30, ptr %1, align 8
  %32 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %31, ptr %32, align 8
  br label %bb15

bb10:                                             ; preds = %bb9
  %_13.0 = extractvalue { ptr, i64 } %28, 0
  %_13.1 = extractvalue { ptr, i64 } %28, 1
; invoke core::ffi::c_str::CStr::as_ptr
  %_12 = invoke ptr @_ZN4core3ffi5c_str4CStr6as_ptr17h4b6be2744fa460e1E(ptr align 1 %_13.0, i64 %_13.1)
          to label %bb11 unwind label %cleanup1

bb11:                                             ; preds = %bb10
  %_11 = call i32 @system(ptr %_12) #17
; invoke core::ptr::drop_in_place<alloc::ffi::c_str::CString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h8922e58224b554a9E"(ptr align 8 %c_cmd)
          to label %bb13 unwind label %cleanup

bb13:                                             ; preds = %bb11
; call core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  call void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17hbd10dcd190b7dadaE"(ptr align 8 %args)
  br label %bb14

bb14:                                             ; preds = %bb4, %bb13
  ret void

terminate:                                        ; preds = %bb16, %bb15
  %33 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %34 = extractvalue { ptr, i32 } %33, 0
  %35 = extractvalue { ptr, i32 } %33, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() #16
  unreachable

bb17:                                             ; preds = %bb16
  %36 = load ptr, ptr %1, align 8
  %37 = getelementptr inbounds i8, ptr %1, i64 8
  %38 = load i32, ptr %37, align 8
  %39 = insertvalue { ptr, i32 } poison, ptr %36, 0
  %40 = insertvalue { ptr, i32 } %39, i32 %38, 1
  resume { ptr, i32 } %40
}

; Function Attrs: nounwind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #4

; <std::env::Args as core::iter::traits::iterator::Iterator>::next
; Function Attrs: uwtable
declare void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h7da987a022c3da75E"(ptr sret([24 x i8]) align 8, ptr align 8) unnamed_addr #1

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #5

; <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
; Function Attrs: uwtable
declare void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h834281278c633905E"(ptr sret([24 x i8]) align 8, ptr align 8) unnamed_addr #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.uadd.sat.i64(i64, i64) #6

; core::panicking::panic_in_cleanup
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking16panic_in_cleanup17h66a1f9f3cf3aa424E() unnamed_addr #7

; std::rt::lang_start_internal
; Function Attrs: uwtable
declare i64 @_ZN3std2rt19lang_start_internal17he1ad9a314bd0157aE(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #1

; core::fmt::Formatter::debug_list
; Function Attrs: uwtable
declare void @_ZN4core3fmt9Formatter10debug_list17hbd42fc068dd3484cE(ptr sret([16 x i8]) align 8, ptr align 8) unnamed_addr #1

; core::fmt::builders::DebugList::finish
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core3fmt8builders9DebugList6finish17h58eea66f4eeff9fdE(ptr align 8) unnamed_addr #1

; core::fmt::num::imp::<impl core::fmt::Display for u8>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17h1d9c5594c24edd21E"(ptr align 1, ptr align 8) unnamed_addr #1

; core::fmt::num::<impl core::fmt::UpperHex for u8>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h6214023d3fccb560E"(ptr align 1, ptr align 8) unnamed_addr #1

; core::fmt::num::<impl core::fmt::LowerHex for u8>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h8618cb0e3425f163E"(ptr align 1, ptr align 8) unnamed_addr #1

; core::fmt::num::imp::<impl core::fmt::Display for usize>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17h37c9cfb43684526bE"(ptr align 8, ptr align 8) unnamed_addr #1

; core::fmt::num::<impl core::fmt::UpperHex for usize>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17he8c8aa69aaec47fdE"(ptr align 8, ptr align 8) unnamed_addr #1

; core::fmt::num::<impl core::fmt::LowerHex for usize>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17ha3e1dcd1e2b6c57fE"(ptr align 8, ptr align 8) unnamed_addr #1

; core::fmt::builders::DebugList::entry
; Function Attrs: uwtable
declare align 8 ptr @_ZN4core3fmt8builders9DebugList5entry17hd4cde1ea99b71f8dE(ptr align 8, ptr align 1, ptr align 8) unnamed_addr #1

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6a7d6a72d1b9e9b7E"(ptr align 8) unnamed_addr #1

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h99196b9ebf8cb15aE"(ptr align 8) unnamed_addr #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17h9ebd1fefce6d2f82E(ptr align 1, i64, ptr align 8) unnamed_addr #8

; core::panicking::panic_nounwind
; Function Attrs: cold noinline noreturn nounwind uwtable
declare void @_ZN4core9panicking14panic_nounwind17h9be992513d3fde16E(ptr align 1, i64) unnamed_addr #9

; core::panicking::assert_failed
; Function Attrs: cold minsize noinline noreturn optsize uwtable
declare void @_ZN4core9panicking13assert_failed17h1c688646bbe431b3E(i8, ptr align 8, ptr align 8, ptr align 8, ptr align 8) unnamed_addr #10

; core::alloc::layout::Layout::is_size_align_valid
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core5alloc6layout6Layout19is_size_align_valid17hba5c1dd91ce45078E(i64, i64) unnamed_addr #1

; core::panicking::panic_cannot_unwind
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking19panic_cannot_unwind17h12e05d862732edabE() unnamed_addr #7

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctpop.i64(i64) #6

; core::panicking::panic_const::panic_const_div_by_zero
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17he9029e02cc767683E(ptr align 8) unnamed_addr #8

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking9panic_fmt17h169d7389ef09f3baE(ptr align 8, ptr align 8) unnamed_addr #8

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core6result13unwrap_failed17ha5a2c69de0fb8105E(ptr align 1, i64, ptr align 1, ptr align 8, ptr align 8) unnamed_addr #8

; <&str as alloc::ffi::c_str::CString::new::SpecNewImpl>::spec_new_impl
; Function Attrs: uwtable
declare void @"_ZN72_$LT$$RF$str$u20$as$u20$alloc..ffi..c_str..CString..new..SpecNewImpl$GT$13spec_new_impl17hdf17333e3fe57099E"(ptr sret([32 x i8]) align 8, ptr align 1, i64) unnamed_addr #1

; alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
; Function Attrs: cold uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h32f2f6d36a81d018E"(ptr align 8, i64, i64, i64, i64) unnamed_addr #11

; alloc::raw_vec::RawVecInner<A>::try_allocate_in
; Function Attrs: uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$15try_allocate_in17h6618b9d04215e743E"(ptr sret([24 x i8]) align 8, i64, i1 zeroext, i64, i64) unnamed_addr #1

; alloc::raw_vec::handle_error
; Function Attrs: cold minsize noreturn optsize uwtable
declare void @_ZN5alloc7raw_vec12handle_error17h7f36dc445c6de4b2E(i64, i64, ptr align 8) unnamed_addr #12

; Function Attrs: nounwind allockind("free") uwtable
declare void @__rust_dealloc(ptr allocptr, i64, i64) unnamed_addr #13

; core::fmt::Formatter::debug_tuple_field2_finish
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core3fmt9Formatter25debug_tuple_field2_finish17h46634f1938cd4c84E(ptr align 8, ptr align 1, i64, ptr align 1, ptr align 8, ptr align 1, ptr align 8) unnamed_addr #1

; core::panicking::panic_bounds_check
; Function Attrs: cold minsize noinline noreturn optsize uwtable
declare void @_ZN4core9panicking18panic_bounds_check17h899c84cea2bff5b8E(i64, i64, ptr align 8) unnamed_addr #10

; alloc::raw_vec::RawVecInner<A>::deallocate
; Function Attrs: uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h3287a4638d2822faE"(ptr align 8, i64, i64) unnamed_addr #1

; std::env::args
; Function Attrs: uwtable
declare void @_ZN3std3env4args17hcb0b85f54a96d99cE(ptr sret([32 x i8]) align 8) unnamed_addr #1

; Function Attrs: nounwind uwtable
declare i32 @system(ptr) unnamed_addr #4

define i32 @main(i32 %0, ptr %1) unnamed_addr #14 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start
  %3 = call i64 @_ZN3std2rt10lang_start17h7bd44b8437c8e17eE(ptr @_ZN12taint_unsafe4main17h1b017c1895fa0b30E, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { inlinehint uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #1 = { uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #2 = { noinline uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #3 = { inlinehint nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #4 = { nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #5 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #6 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #7 = { cold minsize noinline noreturn nounwind optsize uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #8 = { cold noinline noreturn uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #9 = { cold noinline noreturn nounwind uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #10 = { cold minsize noinline noreturn optsize uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #11 = { cold uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #12 = { cold minsize noreturn optsize uwtable "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #13 = { nounwind allockind("free") uwtable "alloc-family"="__rust_alloc" "probe-stack"="inline-asm" "target-cpu"="generic" "target-features"="+v8a,+outline-atomics" }
attributes #14 = { "target-cpu"="generic" }
attributes #15 = { cold }
attributes #16 = { cold noreturn nounwind }
attributes #17 = { nounwind }
attributes #18 = { noreturn }
attributes #19 = { noreturn nounwind }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{!"rustc version 1.85.0 (4d91de4e4 2025-02-17)"}
!3 = !{i64 13086670864678431}
