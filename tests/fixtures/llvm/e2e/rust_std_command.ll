; ModuleID = 'minimal_command_injection.3e1ba58bccd0ce4d-cgu.0'
source_filename = "minimal_command_injection.3e1ba58bccd0ce4d-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "arm64-apple-macosx11.0.0"

%"alloc::boxed::Box<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>" = type { %"core::ptr::unique::Unique<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>", %"alloc::alloc::Global" }
%"core::ptr::unique::Unique<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>" = type { %"core::ptr::non_null::NonNull<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>", %"core::marker::PhantomData<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>" }
%"core::ptr::non_null::NonNull<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>" = type { { ptr, ptr } }
%"core::marker::PhantomData<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>" = type {}
%"alloc::alloc::Global" = type {}
%"std::ffi::os_str::OsString" = type { %"std::sys::os_str::bytes::Buf" }
%"std::sys::os_str::bytes::Buf" = type { %"alloc::vec::Vec<u8>" }
%"alloc::vec::Vec<u8>" = type { %"alloc::raw_vec::RawVec<u8>", i64 }
%"alloc::raw_vec::RawVec<u8>" = type { %"alloc::raw_vec::RawVecInner", %"core::marker::PhantomData<u8>" }
%"alloc::raw_vec::RawVecInner" = type { i64, ptr, %"alloc::alloc::Global" }
%"core::marker::PhantomData<u8>" = type {}
%"core::fmt::rt::Argument<'_>" = type { %"core::fmt::rt::ArgumentType<'_>" }
%"core::fmt::rt::ArgumentType<'_>" = type { ptr, [1 x i64] }
%"core::mem::maybe_uninit::MaybeUninit<std::ffi::os_str::OsString>" = type { [3 x i64] }
%"core::mem::maybe_uninit::MaybeUninit<core::option::Option<std::ffi::os_str::OsString>>" = type { [3 x i64] }

@alloc_b2524a2bb97c9778f74d3a6dcdf08ea6 = private unnamed_addr constant [114 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs\00", align 1
@alloc_cfc9953579a6501e54b0a981fe1d0adf = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_b2524a2bb97c9778f74d3a6dcdf08ea6, [16 x i8] c"q\00\00\00\00\00\00\00\B9\03\00\00 \00\00\00" }>, align 8
@alloc_ec595fc0e82ef92fc59bd74f68296eae = private unnamed_addr constant [73 x i8] c"assertion failed: 0 < pointee_size && pointee_size <= isize::MAX as usize", align 1
@vtable.0 = private unnamed_addr constant <{ ptr, [16 x i8], ptr, ptr, ptr }> <{ ptr @"_ZN4core3ptr93drop_in_place$LT$std..io..default_write_fmt..Adapter$LT$std..sys..stdio..unix..Stderr$GT$$GT$17ha94dfba2c791a5f3E", [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN81_$LT$std..io..default_write_fmt..Adapter$LT$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h6ed374c6060c81d0E", ptr @_ZN4core3fmt5Write10write_char17hfcd0fb5d42725371E, ptr @_ZN4core3fmt5Write9write_fmt17habdc83ce072a46d9E }>, align 8
@alloc_a439077caaee6bab9af745f1531d72a4 = private unnamed_addr constant [86 x i8] c"a formatting trait implementation returned an error when the underlying stream did not", align 1
@alloc_1a0a387c60de9dc9d399cc7846eb8bd8 = private unnamed_addr constant [107 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/std/src/io/mod.rs\00", align 1
@alloc_f27b16caf13fe4b69afd3152fa84a148 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1a0a387c60de9dc9d399cc7846eb8bd8, [16 x i8] c"j\00\00\00\00\00\00\00\88\02\00\00\11\00\00\00" }>, align 8
@alloc_8d68fcbc011419193bd208f22e2789d1 = private unnamed_addr constant [28 x i8] c"failed to write whole buffer", align 1
@alloc_0c6bd0c1dc63d8d7fb1eb0c800f5dec2 = private unnamed_addr constant <{ ptr, [9 x i8], [7 x i8] }> <{ ptr @alloc_8d68fcbc011419193bd208f22e2789d1, [9 x i8] c"\1C\00\00\00\00\00\00\00\17", [7 x i8] undef }>, align 8
@alloc_fe1a9d31c5e3365baf7d9bcb6ad1b550 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1a0a387c60de9dc9d399cc7846eb8bd8, [16 x i8] c"j\00\00\00\00\00\00\00Y\07\00\00$\00\00\00" }>, align 8
@alloc_286087041dd3a746673d8d1b638cd75b = private unnamed_addr constant [124 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/std/src/io/error/repr_bitpacked.rs\00", align 1
@alloc_870b5c744f7c5ea40313f826b340dcff = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_286087041dd3a746673d8d1b638cd75b, [16 x i8] c"{\00\00\00\00\00\00\00\08\01\00\00\1A\00\00\00" }>, align 8
@alloc_a500d906b91607583596fa15e63c2ada = private unnamed_addr constant [40 x i8] c"internal error: entered unreachable code", align 1
@alloc_f7dbf264bb11d70e050fbce2a83b3fe0 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_286087041dd3a746673d8d1b638cd75b, [16 x i8] c"{\00\00\00\00\00\00\00\19\01\00\00\0D\00\00\00" }>, align 8
@vtable.1 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h2f2c9d3cbd8de4e3E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h6895314435e84710E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h6895314435e84710E" }>, align 8
@alloc_670745891beb4cee10ca252b3f3da521 = private unnamed_addr constant [89 x i8] c"fatal runtime error: IO Safety violation: owned file descriptor already closed, aborting\0A", align 1
@alloc_eb8733fbd7aafbd8507b5e26d5686574 = private unnamed_addr constant [109 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/fmt/mod.rs\00", align 1
@alloc_7a51f9194d5f96080ea9d51850247e42 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_eb8733fbd7aafbd8507b5e26d5686574, [16 x i8] c"l\00\00\00\00\00\00\00q\03\00\00*\00\00\00" }>, align 8
@anon.3859b584a99d2d85503c18879f5b49e0.0 = private unnamed_addr constant <{ [8 x i8], [8 x i8] }> <{ [8 x i8] zeroinitializer, [8 x i8] undef }>, align 8
@alloc_560a59ed819b9d9a5841f6e731c4c8e5 = private unnamed_addr constant [210 x i8] c"unsafe precondition(s) violated: NonNull::new_unchecked requires that the pointer is non-null\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_0625062a5eee489a7813ee965a38d15a = private unnamed_addr constant [198 x i8] c"unsafe precondition(s) violated: Alignment::new_unchecked requires a power of two\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_fad0cd83b7d1858a846a172eb260e593 = private unnamed_addr constant [42 x i8] c"is_aligned_to: align is not a power-of-two", align 1
@alloc_3f700da50326608841a67a1169db9dcb = private unnamed_addr constant [115 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs\00", align 1
@alloc_4e4cc478ba2b105df6f8c26c45900512 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_3f700da50326608841a67a1169db9dcb, [16 x i8] c"r\00\00\00\00\00\00\00^\05\00\00\0D\00\00\00" }>, align 8
@alloc_de4e626d456b04760e72bc785ed7e52a = private unnamed_addr constant [201 x i8] c"unsafe precondition(s) violated: ptr::offset_from_unsigned requires `self >= origin`\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_f66a9f5d576768070f4df1f6930215f3 = private unnamed_addr constant [114 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/char/methods.rs\00", align 1
@alloc_f6549330ef387f3ed559c7abaf671c9e = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_f66a9f5d576768070f4df1f6930215f3, [16 x i8] c"q\00\00\00\00\00\00\00b\07\00\00\0E\00\00\00" }>, align 8
@alloc_88fbea7d0d583f188a306688d9d39b2c = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_f66a9f5d576768070f4df1f6930215f3, [16 x i8] c"q\00\00\00\00\00\00\00U\07\00\00\09\00\00\00" }>, align 8
@alloc_2800f9fd9aa1f3e24fde08ad5f3cf8ec = private unnamed_addr constant [71 x i8] c"\12encode_utf8: need \C0\13 bytes to encode U+\C3 \00\00i\04\00\15 but buffer has just \C0\00", align 1
@alloc_75fb06c2453febd814e73f5f2e72ae38 = private unnamed_addr constant [199 x i8] c"unsafe precondition(s) violated: hint::unreachable_unchecked must never be reached\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_8cf3531fd3c828ab298b686071a40c17 = private unnamed_addr constant [259 x i8] c"unsafe precondition(s) violated: Layout::from_size_alignment_unchecked requires that the rounded-up allocation size does not exceed isize::MAX\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_a28e8c8fd5088943a8b5d44af697ff83 = private unnamed_addr constant [279 x i8] c"unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_5c1a2f972552229672fc942406cfc298 = private unnamed_addr constant [283 x i8] c"unsafe precondition(s) violated: slice::from_raw_parts_mut requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_1e3827474b4b8ed59446bd8fe57a54d5 = private unnamed_addr constant [125 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/collections/btree/node.rs\00", align 1
@alloc_32339b860b6b3b1fb6c38cd0d6eecaf7 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1e3827474b4b8ed59446bd8fe57a54d5, [16 x i8] c"|\00\00\00\00\00\00\00\B9\04\00\00!\00\00\00" }>, align 8
@alloc_ecdfd039b9ddf961aa4157d4266b5990 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1e3827474b4b8ed59446bd8fe57a54d5, [16 x i8] c"|\00\00\00\00\00\00\00\BA\04\00\00!\00\00\00" }>, align 8
@alloc_22025f2193ce509b60b3c47e41728876 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1e3827474b4b8ed59446bd8fe57a54d5, [16 x i8] c"|\00\00\00\00\00\00\00[\04\00\001\00\00\00" }>, align 8
@alloc_a0eed1c0a4b50397b4d33b8a339c9cce = private unnamed_addr constant [129 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/collections/btree/navigate.rs\00", align 1
@alloc_655180d482d4b2af927263fa0d679f4a = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a0eed1c0a4b50397b4d33b8a339c9cce, [16 x i8] c"\80\00\00\00\00\00\00\00X\02\00\000\00\00\00" }>, align 8
@alloc_185024b4a9041c4ec014ffc119918981 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a0eed1c0a4b50397b4d33b8a339c9cce, [16 x i8] c"\80\00\00\00\00\00\00\00\E7\00\00\007\00\00\00" }>, align 8
@alloc_a5c6e51765d6e94a394410a5a9df1cec = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a0eed1c0a4b50397b4d33b8a339c9cce, [16 x i8] c"\80\00\00\00\00\00\00\00\C6\00\00\00'\00\00\00" }>, align 8
@alloc_8f092ca583247499d81fca4fed8e37e5 = private unnamed_addr constant [124 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/exact_size.rs\00", align 1
@alloc_bc6f83648b52e3ef006f2e1708c639c7 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_8f092ca583247499d81fca4fed8e37e5, [16 x i8] c"{\00\00\00\00\00\00\00z\00\00\00\09\00\00\00" }>, align 8
@alloc_8e4cfe0ddc6f79e121b760a1ea46e1f2 = private unnamed_addr constant [112 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/ffi/c_str.rs\00", align 1
@alloc_64af571682933ba57a453028bb35ab34 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_8e4cfe0ddc6f79e121b760a1ea46e1f2, [16 x i8] c"o\00\00\00\00\00\00\00\BE\02\00\00\19\00\00\00" }>, align 8
@alloc_07de8a6797dfa3b527bed99f27e46d4d = private unnamed_addr constant [115 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/alignment.rs\00", align 1
@alloc_1bec1b27080c9ef8ecee934b35cfe627 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_07de8a6797dfa3b527bed99f27e46d4d, [16 x i8] c"r\00\00\00\00\00\00\00\7F\00\00\00\12\00\00\00" }>, align 8
@alloc_6e9aff6b801aaa7f5f5138e095764cff = private unnamed_addr constant [114 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/alloc/layout.rs\00", align 1
@alloc_371e9657d30ef9d7d62fff435570ff5b = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_6e9aff6b801aaa7f5f5138e095764cff, [16 x i8] c"q\00\00\00\00\00\00\00\01\01\00\00\12\00\00\00" }>, align 8
@alloc_97d92cbf2a68a6ac45a1b13da79836e4 = private unnamed_addr constant [214 x i8] c"unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_5a8fdd84b3281310cbf6b74bb6bf0065 = private unnamed_addr constant [218 x i8] c"unsafe precondition(s) violated: slice::get_unchecked_mut requires that the index is within the slice\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_4a5583e54c46be478e6ceff39a501c5d = private unnamed_addr constant [112 x i8] c"/Users/zihao/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/unique.rs\00", align 1
@alloc_e220089d5016f4a98719c6bb086d229f = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_4a5583e54c46be478e6ceff39a501c5d, [16 x i8] c"o\00\00\00\00\00\00\00Y\00\00\00$\00\00\00" }>, align 8

; <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
; Function Attrs: inlinehint uwtable
define internal void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17he3d89b131cd49a0bE"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
  %exact = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
  %_10 = getelementptr inbounds i8, ptr %self, i64 24
  %self1 = load ptr, ptr %_10, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %subtracted = load ptr, ptr %0, align 8
  br label %bb5

bb5:                                              ; preds = %bb2
; call core::ptr::const_ptr::<impl *const T>::offset_from_unsigned::precondition_check
  call void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$20offset_from_unsigned18precondition_check17habad995d57f98f8fE"(ptr %self1, ptr %subtracted, ptr align 8 @alloc_cfc9953579a6501e54b0a981fe1d0adf) #19
  br label %bb7

bb7:                                              ; preds = %bb5
  br label %bb8

bb8:                                              ; preds = %bb7
  br label %bb9

bb9:                                              ; preds = %bb8
  %1 = ptrtoint ptr %self1 to i64
  %2 = ptrtoint ptr %subtracted to i64
  %3 = sub nuw i64 %1, %2
  %4 = udiv exact i64 %3, 24
  store i64 %4, ptr %exact, align 8
  br label %bb3

bb10:                                             ; No predecessors!
; call core::panicking::panic
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking5panic(ptr align 1 @alloc_ec595fc0e82ef92fc59bd74f68296eae, i64 73, ptr align 8 @alloc_cfc9953579a6501e54b0a981fe1d0adf) #20
  unreachable

bb3:                                              ; preds = %bb9
  %_12 = load i64, ptr %exact, align 8
  %_13.1 = load i64, ptr %exact, align 8
  store i64 %_12, ptr %_0, align 8
  %5 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 1, ptr %5, align 8
  %6 = getelementptr inbounds i8, ptr %5, i64 8
  store i64 %_13.1, ptr %6, align 8
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; <core::option::Option<T> as core::ops::try_trait::FromResidual<core::option::Option<core::convert::Infallible>>>::from_residual
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN145_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..FromResidual$LT$core..option..Option$LT$core..convert..Infallible$GT$$GT$$GT$13from_residual17h71f6be104875a5f4E"() unnamed_addr #0 {
start:
  ret i64 0
}

; <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd4f2e7bd58139e8bE"(ptr align 8 %self) unnamed_addr #1 {
start:
  %capacity = alloca [8 x i8], align 8
  %_4 = alloca [16 x i8], align 8
  %_7 = load ptr, ptr %self, align 8
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
  %_12.0 = load i64, ptr %capacity, align 8
  store i64 %_12.0, ptr %_4, align 8
  %1 = getelementptr inbounds i8, ptr %_4, i64 8
  store ptr %ptr, ptr %1, align 8
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17hb76fb17fa1eab069E"(ptr align 8 %_4)
  ret void

bb3:                                              ; No predecessors!
  unreachable
}

; <<alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop::DropGuard<K,V,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN174_$LT$$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h25db7b4f4a612004E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %kv = alloca [24 x i8], align 8
  %_2 = alloca [24 x i8], align 8
  br label %bb1

bb1:                                              ; preds = %bb3, %start
  %_6 = load ptr, ptr %self, align 8
; call alloc::collections::btree::map::IntoIter<K,V,A>::dying_next
  call void @"_ZN5alloc11collections5btree3map25IntoIter$LT$K$C$V$C$A$GT$10dying_next17hc659d595af5fbe87E"(ptr sret([24 x i8]) align 8 %_2, ptr align 8 %_6)
  %0 = load ptr, ptr %_2, align 8
  %1 = ptrtoint ptr %0 to i64
  %2 = icmp eq i64 %1, 0
  %_3 = select i1 %2, i64 0, i64 1
  %3 = trunc nuw i64 %_3 to i1
  br i1 %3, label %bb3, label %bb5

bb3:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %kv, ptr align 8 %_2, i64 24, i1 false)
; call alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val
  call void @"_ZN5alloc11collections5btree4node173Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$12drop_key_val17h8689e92183180c52E"(ptr align 8 %kv) #21
  br label %bb1

bb5:                                              ; preds = %bb1
  ret void

bb6:                                              ; No predecessors!
  unreachable
}

; minimal_command_injection::main
; Function Attrs: uwtable
define hidden void @_ZN25minimal_command_injection4main17hc4412d17c931f476E() unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_9 = alloca [1 x i8], align 1
  %_8 = alloca [24 x i8], align 8
  %_7 = alloca [200 x i8], align 8
  %_5 = alloca [16 x i8], align 8
  %_4 = alloca [32 x i8], align 8
  %_2 = alloca [24 x i8], align 8
  %cmd = alloca [24 x i8], align 8
  store i8 0, ptr %_9, align 1
; call std::env::args
  call void @_RNvNtCsg55jX0GwzBC_3std3env4args(ptr sret([32 x i8]) align 8 %_4)
; invoke core::iter::traits::iterator::Iterator::nth
  invoke void @_ZN4core4iter6traits8iterator8Iterator3nth17hb010d30a7e72e147E(ptr sret([24 x i8]) align 8 %_2, ptr align 8 %_4, i64 1)
          to label %bb2 unwind label %cleanup

bb10:                                             ; preds = %cleanup
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17heb4d1ac16b5f8206E"(ptr align 8 %_4) #22
          to label %bb11 unwind label %terminate

cleanup:                                          ; preds = %bb2, %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb10

bb2:                                              ; preds = %start
; invoke core::option::Option<T>::unwrap_or_default
  invoke void @"_ZN4core6option15Option$LT$T$GT$17unwrap_or_default17he6cc1db57abb6876E"(ptr sret([24 x i8]) align 8 %cmd, ptr align 8 %_2)
          to label %bb3 unwind label %cleanup

bb3:                                              ; preds = %bb2
  store i8 1, ptr %_9, align 1
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17heb4d1ac16b5f8206E"(ptr align 8 %_4)
          to label %bb4 unwind label %cleanup1

bb13:                                             ; preds = %bb9, %cleanup1
  %5 = load i8, ptr %_9, align 1
  %6 = trunc nuw i8 %5 to i1
  br i1 %6, label %bb12, label %bb11

cleanup1:                                         ; preds = %bb7, %bb4, %bb3
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %0, align 8
  %10 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb13

bb4:                                              ; preds = %bb3
  store i8 0, ptr %_9, align 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_8, ptr align 8 %cmd, i64 24, i1 false)
; invoke std::process::Command::new
  invoke void @_ZN3std7process7Command3new17hdfbc64f0a259b374E(ptr sret([200 x i8]) align 8 %_7, ptr align 8 %_8)
          to label %bb5 unwind label %cleanup1

bb5:                                              ; preds = %bb4
; invoke <std::process::Command>::status
  invoke void @_RNvMsk_NtCsg55jX0GwzBC_3std7processNtB5_7Command6status(ptr sret([16 x i8]) align 8 %_5, ptr align 8 %_7)
          to label %bb6 unwind label %cleanup2

bb9:                                              ; preds = %cleanup2
; invoke core::ptr::drop_in_place<std::process::Command>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$std..process..Command$GT$17h0ab653a9bcd4d1e4E"(ptr align 8 %_7) #22
          to label %bb13 unwind label %terminate

cleanup2:                                         ; preds = %bb6, %bb5
  %11 = landingpad { ptr, i32 }
          cleanup
  %12 = extractvalue { ptr, i32 } %11, 0
  %13 = extractvalue { ptr, i32 } %11, 1
  store ptr %12, ptr %0, align 8
  %14 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %13, ptr %14, align 8
  br label %bb9

bb6:                                              ; preds = %bb5
; invoke core::ptr::drop_in_place<core::result::Result<std::process::ExitStatus,std::io::error::Error>>
  invoke void @"_ZN4core3ptr97drop_in_place$LT$core..result..Result$LT$std..process..ExitStatus$C$std..io..error..Error$GT$$GT$17hecf567af432d4777E"(ptr align 8 %_5)
          to label %bb7 unwind label %cleanup2

bb7:                                              ; preds = %bb6
; invoke core::ptr::drop_in_place<std::process::Command>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$std..process..Command$GT$17h0ab653a9bcd4d1e4E"(ptr align 8 %_7)
          to label %bb8 unwind label %cleanup1

bb8:                                              ; preds = %bb7
  store i8 0, ptr %_9, align 1
  ret void

terminate:                                        ; preds = %bb10, %bb12, %bb9
  %15 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb11:                                             ; preds = %bb10, %bb12, %bb13
  %16 = load ptr, ptr %0, align 8
  %17 = getelementptr inbounds i8, ptr %0, i64 8
  %18 = load i32, ptr %17, align 8
  %19 = insertvalue { ptr, i32 } poison, ptr %16, 0
  %20 = insertvalue { ptr, i32 } %19, i32 %18, 1
  resume { ptr, i32 } %20

bb12:                                             ; preds = %bb13
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he154a73b91055e9aE"(ptr align 8 %cmd) #22
          to label %bb11 unwind label %terminate
}

; <alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val::Dropper<T> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN280_$LT$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$..drop_key_val..Dropper$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4f9d13ef07bd4e1eE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %self1 = load ptr, ptr %self, align 8
; call core::ptr::drop_in_place<core::option::Option<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$17hcde64bce32fbc5c6E"(ptr align 8 %self1)
  ret void
}

; std::io::default_write_fmt
; Function Attrs: uwtable
define internal ptr @_ZN3std2io17default_write_fmt17hb41c115b835de1beE(ptr align 1 %this, ptr %args.0, ptr %args.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %output = alloca [16 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  store ptr %this, ptr %output, align 8
  %1 = getelementptr inbounds i8, ptr %output, i64 8
  store ptr null, ptr %1, align 8
; invoke core::fmt::write
  %_4 = invoke zeroext i1 @_RNvNtCsl8K0bEFm1U0_4core3fmt5write(ptr align 1 %output, ptr align 8 @vtable.0, ptr %args.0, ptr %args.1)
          to label %bb1 unwind label %cleanup

bb7:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<std::io::default_write_fmt::Adapter<std::sys::stdio::unix::Stderr>>
  invoke void @"_ZN4core3ptr93drop_in_place$LT$std..io..default_write_fmt..Adapter$LT$std..sys..stdio..unix..Stderr$GT$$GT$17ha94dfba2c791a5f3E"(ptr align 8 %output) #22
          to label %bb8 unwind label %terminate

cleanup:                                          ; preds = %bb6, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb7

bb1:                                              ; preds = %start
  %_7 = zext i1 %_4 to i64
  %6 = trunc nuw i64 %_7 to i1
  br i1 %6, label %bb3, label %bb4

bb3:                                              ; preds = %bb1
  %7 = getelementptr inbounds i8, ptr %output, i64 8
  %8 = load ptr, ptr %7, align 8
  %9 = ptrtoint ptr %8 to i64
  %10 = icmp eq i64 %9, 0
  %_13 = select i1 %10, i64 0, i64 1
  %_12 = icmp eq i64 %_13, 0
  %_8 = xor i1 %_12, true
  br i1 %_8, label %bb5, label %bb6

bb4:                                              ; preds = %bb1
  store ptr null, ptr %_0, align 8
  %11 = getelementptr inbounds i8, ptr %output, i64 8
; call core::ptr::drop_in_place<core::result::Result<(),std::io::error::Error>>
  call void @"_ZN4core3ptr81drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$GT$17ha0d2151b81f17565E"(ptr align 8 %11)
  br label %bb9

bb9:                                              ; preds = %bb5, %bb4
  %12 = load ptr, ptr %_0, align 8
  ret ptr %12

bb6:                                              ; preds = %bb3
; invoke core::panicking::panic_fmt
  invoke void @_RNvNtCsl8K0bEFm1U0_4core9panicking9panic_fmt(ptr @alloc_a439077caaee6bab9af745f1531d72a4, ptr inttoptr (i64 173 to ptr), ptr align 8 @alloc_f27b16caf13fe4b69afd3152fa84a148) #24
          to label %unreachable unwind label %cleanup

bb5:                                              ; preds = %bb3
  %13 = getelementptr inbounds i8, ptr %output, i64 8
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %_0, align 8
  br label %bb9

unreachable:                                      ; preds = %bb6
  unreachable

bb2:                                              ; No predecessors!
  unreachable

terminate:                                        ; preds = %bb7
  %15 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb8:                                              ; preds = %bb7
  %16 = load ptr, ptr %0, align 8
  %17 = getelementptr inbounds i8, ptr %0, i64 8
  %18 = load i32, ptr %17, align 8
  %19 = insertvalue { ptr, i32 } poison, ptr %16, 0
  %20 = insertvalue { ptr, i32 } %19, i32 %18, 1
  resume { ptr, i32 } %20
}

; std::io::Write::write_all
; Function Attrs: uwtable
define internal ptr @_ZN3std2io5Write9write_all17h8af9bb9b2151de98E(ptr align 1 %self, ptr align 1 %0, i64 %1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %2 = alloca [16 x i8], align 8
  %_15 = alloca [16 x i8], align 8
  %_10 = alloca [1 x i8], align 1
  %_4 = alloca [16 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  %buf = alloca [16 x i8], align 8
  store ptr %0, ptr %buf, align 8
  %3 = getelementptr inbounds i8, ptr %buf, i64 8
  store i64 %1, ptr %3, align 8
  br label %bb1

bb1:                                              ; preds = %bb15, %start
  %self.0 = load ptr, ptr %buf, align 8
  %4 = getelementptr inbounds i8, ptr %buf, i64 8
  %self.1 = load i64, ptr %4, align 8
  %5 = icmp eq i64 %self.1, 0
  br i1 %5, label %bb2, label %bb3

bb2:                                              ; preds = %bb1
  store ptr null, ptr %_0, align 8
  br label %bb13

bb3:                                              ; preds = %bb1
  %_5.0 = load ptr, ptr %buf, align 8
  %6 = getelementptr inbounds i8, ptr %buf, i64 8
  %_5.1 = load i64, ptr %6, align 8
; call <std::sys::stdio::unix::Stderr as std::io::Write>::write
  %7 = call { i64, ptr } @_RNvXs3_NtNtNtCsg55jX0GwzBC_3std3sys5stdio4unixNtB5_6StderrNtNtBb_2io5Write5write(ptr align 1 %self, ptr align 1 %_5.0, i64 %_5.1)
  %8 = extractvalue { i64, ptr } %7, 0
  %9 = extractvalue { i64, ptr } %7, 1
  store i64 %8, ptr %_4, align 8
  %10 = getelementptr inbounds i8, ptr %_4, i64 8
  store ptr %9, ptr %10, align 8
  %_6 = load i64, ptr %_4, align 8
  %11 = getelementptr inbounds i8, ptr %_4, i64 8
  %12 = load ptr, ptr %11, align 8
  %13 = trunc nuw i64 %_6 to i1
  br i1 %13, label %bb7, label %bb6

bb13:                                             ; preds = %bb12, %bb2
  %14 = load ptr, ptr %_0, align 8
  ret ptr %14

bb7:                                              ; preds = %bb3
  %15 = getelementptr inbounds i8, ptr %_4, i64 8
  %_25 = load ptr, ptr %15, align 8
; invoke std::io::error::repr_bitpacked::decode_repr
  invoke void @_ZN3std2io5error14repr_bitpacked11decode_repr17hd3711c555d973467E(ptr sret([16 x i8]) align 8 %_15, ptr %_25)
          to label %bb25 unwind label %cleanup

bb6:                                              ; preds = %bb3
  %16 = getelementptr inbounds i8, ptr %_4, i64 8
  %17 = load i64, ptr %16, align 8
  %18 = icmp eq i64 %17, 0
  br i1 %18, label %bb9, label %bb8

bb9:                                              ; preds = %bb6
  store ptr @alloc_0c6bd0c1dc63d8d7fb1eb0c800f5dec2, ptr %_0, align 8
  br label %bb12

bb8:                                              ; preds = %bb6
  %19 = getelementptr inbounds i8, ptr %_4, i64 8
  %n = load i64, ptr %19, align 8
  %20 = load ptr, ptr %buf, align 8
  %21 = getelementptr inbounds i8, ptr %buf, i64 8
  %_30 = load i64, ptr %21, align 8
  %_29 = icmp ugt i64 %n, %_30
  br i1 %_29, label %bb26, label %bb27

bb12:                                             ; preds = %bb11, %bb9
  br label %bb13

bb27:                                             ; preds = %bb8
  %new_len = sub nuw i64 %_30, %n
  %_34.0 = load ptr, ptr %buf, align 8
  %22 = getelementptr inbounds i8, ptr %buf, i64 8
  %_34.1 = load i64, ptr %22, align 8
  %_36 = getelementptr inbounds nuw i8, ptr %_34.0, i64 %n
  store ptr %_36, ptr %buf, align 8
  %23 = getelementptr inbounds i8, ptr %buf, i64 8
  store i64 %new_len, ptr %23, align 8
  br label %bb17

bb26:                                             ; preds = %bb8
; invoke core::slice::index::slice_index_fail
  invoke void @_RNvNtNtCsl8K0bEFm1U0_4core5slice5index16slice_index_fail(i64 %n, i64 %_30, i64 %_30, ptr align 8 @alloc_fe1a9d31c5e3365baf7d9bcb6ad1b550) #24
          to label %unreachable unwind label %cleanup

bb17:                                             ; preds = %bb10, %bb27
  %_12 = load i64, ptr %_4, align 8
  %24 = getelementptr inbounds i8, ptr %_4, i64 8
  %25 = load ptr, ptr %24, align 8
  %26 = trunc nuw i64 %_12 to i1
  br i1 %26, label %bb16, label %bb15

bb19:                                             ; preds = %cleanup
  %_13 = load i64, ptr %_4, align 8
  %27 = getelementptr inbounds i8, ptr %_4, i64 8
  %28 = load ptr, ptr %27, align 8
  %29 = icmp eq i64 %_13, 1
  br i1 %29, label %bb18, label %bb14

cleanup:                                          ; preds = %bb7, %bb26
  %30 = landingpad { ptr, i32 }
          cleanup
  %31 = extractvalue { ptr, i32 } %30, 0
  %32 = extractvalue { ptr, i32 } %30, 1
  store ptr %31, ptr %2, align 8
  %33 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %32, ptr %33, align 8
  br label %bb19

unreachable:                                      ; preds = %bb26
  unreachable

bb25:                                             ; preds = %bb7
  %34 = load i8, ptr %_15, align 8
  %_17 = zext i8 %34 to i64
  switch i64 %_17, label %bb5 [
    i64 0, label %bb23
    i64 1, label %bb21
    i64 2, label %bb20
    i64 3, label %bb22
  ]

bb5:                                              ; preds = %bb25
  unreachable

bb23:                                             ; preds = %bb25
  %35 = getelementptr inbounds i8, ptr %_15, i64 4
  %code = load i32, ptr %35, align 4
  %36 = icmp eq i32 %code, 4
  %37 = zext i1 %36 to i8
  store i8 %37, ptr %_10, align 1
  br label %bb24

bb21:                                             ; preds = %bb25
  %38 = getelementptr inbounds i8, ptr %_15, i64 1
  %kind = load i8, ptr %38, align 1
  %_27 = zext i8 %kind to i64
  %39 = icmp eq i64 %_27, 35
  %40 = zext i1 %39 to i8
  store i8 %40, ptr %_10, align 1
  br label %bb24

bb20:                                             ; preds = %bb25
  %41 = getelementptr inbounds i8, ptr %_15, i64 8
  %m = load ptr, ptr %41, align 8
  %42 = getelementptr inbounds i8, ptr %m, i64 16
  %43 = load i8, ptr %42, align 8
  %_26 = zext i8 %43 to i64
  %44 = icmp eq i64 %_26, 35
  %45 = zext i1 %44 to i8
  store i8 %45, ptr %_10, align 1
  br label %bb24

bb22:                                             ; preds = %bb25
  %46 = getelementptr inbounds i8, ptr %_15, i64 8
  %c = load ptr, ptr %46, align 8
  %47 = getelementptr inbounds i8, ptr %c, i64 16
  %48 = load i8, ptr %47, align 8
  %_28 = zext i8 %48 to i64
  %49 = icmp eq i64 %_28, 35
  %50 = zext i1 %49 to i8
  store i8 %50, ptr %_10, align 1
  br label %bb24

bb24:                                             ; preds = %bb22, %bb20, %bb21, %bb23
  %51 = load i8, ptr %_10, align 1
  %52 = trunc nuw i8 %51 to i1
  br i1 %52, label %bb10, label %bb11

bb11:                                             ; preds = %bb24
  %53 = getelementptr inbounds i8, ptr %_4, i64 8
  %e = load ptr, ptr %53, align 8
  store ptr %e, ptr %_0, align 8
  br label %bb12

bb10:                                             ; preds = %bb24
  br label %bb17

bb16:                                             ; preds = %bb17
  %54 = getelementptr inbounds i8, ptr %_4, i64 8
; call core::ptr::drop_in_place<std::io::error::Error>
  call void @"_ZN4core3ptr42drop_in_place$LT$std..io..error..Error$GT$17hbf06dc6e9bf4f7e0E"(ptr align 8 %54)
  br label %bb15

bb15:                                             ; preds = %bb16, %bb17
  br label %bb1

bb18:                                             ; preds = %bb19
  %55 = getelementptr inbounds i8, ptr %_4, i64 8
; invoke core::ptr::drop_in_place<std::io::error::Error>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$std..io..error..Error$GT$17hbf06dc6e9bf4f7e0E"(ptr align 8 %55) #22
          to label %bb14 unwind label %terminate

bb14:                                             ; preds = %bb18, %bb19
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds i8, ptr %2, i64 8
  %58 = load i32, ptr %57, align 8
  %59 = insertvalue { ptr, i32 } poison, ptr %56, 0
  %60 = insertvalue { ptr, i32 } %59, i32 %58, 1
  resume { ptr, i32 } %60

terminate:                                        ; preds = %bb18
  %61 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable
}

; std::io::Write::write_fmt
; Function Attrs: uwtable
define internal ptr @_ZN3std2io5Write9write_fmt17hc69cbeab8b6551b3E(ptr align 1 %self, ptr %0, ptr %1) unnamed_addr #1 {
start:
  %_3 = alloca [16 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  %args = alloca [16 x i8], align 8
  store ptr %0, ptr %args, align 8
  %2 = getelementptr inbounds i8, ptr %args, i64 8
  store ptr %1, ptr %2, align 8
; call core::fmt::Arguments::as_statically_known_str
  %3 = call { ptr, i64 } @_ZN4core3fmt9Arguments23as_statically_known_str17h16f3697c0ce049d6E(ptr align 8 %args) #21
  %4 = extractvalue { ptr, i64 } %3, 0
  %5 = extractvalue { ptr, i64 } %3, 1
  store ptr %4, ptr %_3, align 8
  %6 = getelementptr inbounds i8, ptr %_3, i64 8
  store i64 %5, ptr %6, align 8
  %7 = load ptr, ptr %_3, align 8
  %8 = getelementptr inbounds i8, ptr %_3, i64 8
  %9 = load i64, ptr %8, align 8
  %10 = ptrtoint ptr %7 to i64
  %11 = icmp eq i64 %10, 0
  %_5 = select i1 %11, i64 0, i64 1
  %12 = trunc nuw i64 %_5 to i1
  br i1 %12, label %bb2, label %bb4

bb2:                                              ; preds = %start
  %s.0 = load ptr, ptr %_3, align 8
  %13 = getelementptr inbounds i8, ptr %_3, i64 8
  %s.1 = load i64, ptr %13, align 8
; call std::io::Write::write_all
  %14 = call ptr @_ZN3std2io5Write9write_all17h8af9bb9b2151de98E(ptr align 1 %self, ptr align 1 %s.0, i64 %s.1)
  store ptr %14, ptr %_0, align 8
  br label %bb5

bb4:                                              ; preds = %start
  %15 = load ptr, ptr %args, align 8
  %16 = getelementptr inbounds i8, ptr %args, i64 8
  %17 = load ptr, ptr %16, align 8
; call std::io::default_write_fmt
  %18 = call ptr @_ZN3std2io17default_write_fmt17hb41c115b835de1beE(ptr align 1 %self, ptr %15, ptr %17)
  store ptr %18, ptr %_0, align 8
  br label %bb5

bb5:                                              ; preds = %bb4, %bb2
  %19 = load ptr, ptr %_0, align 8
  ret ptr %19

bb6:                                              ; No predecessors!
  unreachable
}

; std::io::error::repr_bitpacked::decode_repr
; Function Attrs: inlinehint uwtable
define internal void @_ZN3std2io5error14repr_bitpacked11decode_repr17h9d4147e2b60c0c89E(ptr sret([16 x i8]) align 8 %_0, ptr %ptr) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [16 x i8], align 8
  %_21 = alloca [1 x i8], align 1
  %self = alloca [1 x i8], align 1
  store i8 1, ptr %_21, align 1
  %bits = ptrtoint ptr %ptr to i64
  %_5 = and i64 %bits, 3
  switch i64 %_5, label %bb1 [
    i64 2, label %bb5
    i64 3, label %bb4
    i64 0, label %bb3
    i64 1, label %bb2
  ]

bb1:                                              ; preds = %start
; invoke core::panicking::panic
  invoke void @_RNvNtCsl8K0bEFm1U0_4core9panicking5panic(ptr align 1 @alloc_a500d906b91607583596fa15e63c2ada, i64 40, ptr align 8 @alloc_f7dbf264bb11d70e050fbce2a83b3fe0) #24
          to label %unreachable unwind label %cleanup

bb5:                                              ; preds = %start
  %_7 = ashr i64 %bits, 32
  %code = trunc i64 %_7 to i32
  %2 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %code, ptr %2, align 4
  store i8 0, ptr %_0, align 8
  br label %bb16

bb4:                                              ; preds = %start
  %_10 = lshr i64 %bits, 32
  %kind_bits = trunc i64 %_10 to i32
; invoke std::io::error::repr_bitpacked::kind_from_prim
  %3 = invoke i8 @_ZN3std2io5error14repr_bitpacked14kind_from_prim17h481d5cc500c748ebE(i32 %kind_bits)
          to label %bb6 unwind label %cleanup

bb3:                                              ; preds = %start
  %4 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr %ptr, ptr %4, align 8
  store i8 2, ptr %_0, align 8
  br label %bb16

bb2:                                              ; preds = %start
  %5 = getelementptr i8, ptr %ptr, i64 -1
  store ptr %5, ptr %0, align 8
  %_33 = load ptr, ptr %0, align 8
  store i8 0, ptr %_21, align 1
; invoke <std::io::error::repr_bitpacked::Repr as core::ops::drop::Drop>::drop::{{closure}}
  %_17 = invoke align 8 ptr @"_ZN78_$LT$std..io..error..repr_bitpacked..Repr$u20$as$u20$core..ops..drop..Drop$GT$4drop28_$u7b$$u7b$closure$u7d$$u7d$17h7f350cb5994ee852E"(ptr %_33)
          to label %bb7 unwind label %cleanup

bb16:                                             ; preds = %bb3, %bb14, %bb5
  br label %bb8

bb11:                                             ; preds = %cleanup
  %6 = load i8, ptr %_21, align 1
  %7 = trunc nuw i8 %6 to i1
  br i1 %7, label %bb10, label %bb9

cleanup:                                          ; preds = %bb1, %bb2, %bb4
  %8 = landingpad { ptr, i32 }
          cleanup
  %9 = extractvalue { ptr, i32 } %8, 0
  %10 = extractvalue { ptr, i32 } %8, 1
  store ptr %9, ptr %1, align 8
  %11 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %10, ptr %11, align 8
  br label %bb11

bb6:                                              ; preds = %bb4
  store i8 %3, ptr %self, align 1
  %12 = load i8, ptr %self, align 1
  %13 = icmp eq i8 %12, 42
  %_23 = select i1 %13, i64 0, i64 1
  %14 = trunc nuw i64 %_23 to i1
  br i1 %14, label %bb14, label %bb13

bb14:                                             ; preds = %bb6
  %kind = load i8, ptr %self, align 1
  %15 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 %kind, ptr %15, align 1
  store i8 1, ptr %_0, align 8
  br label %bb16

bb13:                                             ; preds = %bb6
; call core::hint::unreachable_unchecked::precondition_check
  call void @_ZN4core4hint21unreachable_unchecked18precondition_check17h3199eaece5aa5bf6E(ptr align 8 @alloc_870b5c744f7c5ea40313f826b340dcff) #19
  br label %bb12

bb12:                                             ; preds = %bb13
  unreachable

bb8:                                              ; preds = %bb7, %bb16
  ret void

bb7:                                              ; preds = %bb2
  %16 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr %_17, ptr %16, align 8
  store i8 3, ptr %_0, align 8
  br label %bb8

unreachable:                                      ; preds = %bb1
  unreachable

bb9:                                              ; preds = %bb10, %bb11
  %17 = load ptr, ptr %1, align 8
  %18 = getelementptr inbounds i8, ptr %1, i64 8
  %19 = load i32, ptr %18, align 8
  %20 = insertvalue { ptr, i32 } poison, ptr %17, 0
  %21 = insertvalue { ptr, i32 } %20, i32 %19, 1
  resume { ptr, i32 } %21

bb10:                                             ; preds = %bb11
  br label %bb9
}

; std::io::error::repr_bitpacked::decode_repr
; Function Attrs: inlinehint uwtable
define internal void @_ZN3std2io5error14repr_bitpacked11decode_repr17hd3711c555d973467E(ptr sret([16 x i8]) align 8 %_0, ptr %ptr) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [16 x i8], align 8
  %_21 = alloca [1 x i8], align 1
  %self = alloca [1 x i8], align 1
  store i8 1, ptr %_21, align 1
  %bits = ptrtoint ptr %ptr to i64
  %_5 = and i64 %bits, 3
  switch i64 %_5, label %bb1 [
    i64 2, label %bb5
    i64 3, label %bb4
    i64 0, label %bb3
    i64 1, label %bb2
  ]

bb1:                                              ; preds = %start
; invoke core::panicking::panic
  invoke void @_RNvNtCsl8K0bEFm1U0_4core9panicking5panic(ptr align 1 @alloc_a500d906b91607583596fa15e63c2ada, i64 40, ptr align 8 @alloc_f7dbf264bb11d70e050fbce2a83b3fe0) #24
          to label %unreachable unwind label %cleanup

bb5:                                              ; preds = %start
  %_7 = ashr i64 %bits, 32
  %code = trunc i64 %_7 to i32
  %2 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %code, ptr %2, align 4
  store i8 0, ptr %_0, align 8
  br label %bb16

bb4:                                              ; preds = %start
  %_10 = lshr i64 %bits, 32
  %kind_bits = trunc i64 %_10 to i32
; invoke std::io::error::repr_bitpacked::kind_from_prim
  %3 = invoke i8 @_ZN3std2io5error14repr_bitpacked14kind_from_prim17h481d5cc500c748ebE(i32 %kind_bits)
          to label %bb6 unwind label %cleanup

bb3:                                              ; preds = %start
  %4 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr %ptr, ptr %4, align 8
  store i8 2, ptr %_0, align 8
  br label %bb16

bb2:                                              ; preds = %start
  %5 = getelementptr i8, ptr %ptr, i64 -1
  store ptr %5, ptr %0, align 8
  %_33 = load ptr, ptr %0, align 8
  store i8 0, ptr %_21, align 1
; invoke std::io::error::repr_bitpacked::Repr::data::{{closure}}
  %_17 = invoke align 8 ptr @"_ZN3std2io5error14repr_bitpacked4Repr4data28_$u7b$$u7b$closure$u7d$$u7d$17h9d7794893e915c56E"(ptr %_33)
          to label %bb7 unwind label %cleanup

bb16:                                             ; preds = %bb3, %bb14, %bb5
  br label %bb8

bb11:                                             ; preds = %cleanup
  %6 = load i8, ptr %_21, align 1
  %7 = trunc nuw i8 %6 to i1
  br i1 %7, label %bb10, label %bb9

cleanup:                                          ; preds = %bb1, %bb2, %bb4
  %8 = landingpad { ptr, i32 }
          cleanup
  %9 = extractvalue { ptr, i32 } %8, 0
  %10 = extractvalue { ptr, i32 } %8, 1
  store ptr %9, ptr %1, align 8
  %11 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %10, ptr %11, align 8
  br label %bb11

bb6:                                              ; preds = %bb4
  store i8 %3, ptr %self, align 1
  %12 = load i8, ptr %self, align 1
  %13 = icmp eq i8 %12, 42
  %_23 = select i1 %13, i64 0, i64 1
  %14 = trunc nuw i64 %_23 to i1
  br i1 %14, label %bb14, label %bb13

bb14:                                             ; preds = %bb6
  %kind = load i8, ptr %self, align 1
  %15 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 %kind, ptr %15, align 1
  store i8 1, ptr %_0, align 8
  br label %bb16

bb13:                                             ; preds = %bb6
; call core::hint::unreachable_unchecked::precondition_check
  call void @_ZN4core4hint21unreachable_unchecked18precondition_check17h3199eaece5aa5bf6E(ptr align 8 @alloc_870b5c744f7c5ea40313f826b340dcff) #19
  br label %bb12

bb12:                                             ; preds = %bb13
  unreachable

bb8:                                              ; preds = %bb7, %bb16
  ret void

bb7:                                              ; preds = %bb2
  %16 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr %_17, ptr %16, align 8
  store i8 3, ptr %_0, align 8
  br label %bb8

unreachable:                                      ; preds = %bb1
  unreachable

bb9:                                              ; preds = %bb10, %bb11
  %17 = load ptr, ptr %1, align 8
  %18 = getelementptr inbounds i8, ptr %1, i64 8
  %19 = load i32, ptr %18, align 8
  %20 = insertvalue { ptr, i32 } poison, ptr %17, 0
  %21 = insertvalue { ptr, i32 } %20, i32 %19, 1
  resume { ptr, i32 } %21

bb10:                                             ; preds = %bb11
  br label %bb9
}

; std::io::error::repr_bitpacked::kind_from_prim
; Function Attrs: inlinehint uwtable
define internal i8 @_ZN3std2io5error14repr_bitpacked14kind_from_prim17h481d5cc500c748ebE(i32 %ek) unnamed_addr #0 {
start:
  %_0 = alloca [1 x i8], align 1
  %0 = icmp eq i32 %ek, 0
  br i1 %0, label %bb1, label %bb2

bb1:                                              ; preds = %start
  store i8 0, ptr %_0, align 1
  br label %bb85

bb2:                                              ; preds = %start
  %1 = icmp eq i32 %ek, 1
  br i1 %1, label %bb3, label %bb4

bb85:                                             ; preds = %bb84, %bb83, %bb81, %bb79, %bb77, %bb75, %bb73, %bb71, %bb69, %bb67, %bb65, %bb63, %bb61, %bb59, %bb57, %bb55, %bb53, %bb51, %bb49, %bb47, %bb45, %bb43, %bb41, %bb39, %bb37, %bb35, %bb33, %bb31, %bb29, %bb27, %bb25, %bb23, %bb21, %bb19, %bb17, %bb15, %bb13, %bb11, %bb9, %bb7, %bb5, %bb3, %bb1
  %2 = load i8, ptr %_0, align 1
  ret i8 %2

bb3:                                              ; preds = %bb2
  store i8 1, ptr %_0, align 1
  br label %bb85

bb4:                                              ; preds = %bb2
  %3 = icmp eq i32 %ek, 2
  br i1 %3, label %bb5, label %bb6

bb5:                                              ; preds = %bb4
  store i8 2, ptr %_0, align 1
  br label %bb85

bb6:                                              ; preds = %bb4
  %4 = icmp eq i32 %ek, 3
  br i1 %4, label %bb7, label %bb8

bb7:                                              ; preds = %bb6
  store i8 3, ptr %_0, align 1
  br label %bb85

bb8:                                              ; preds = %bb6
  %5 = icmp eq i32 %ek, 4
  br i1 %5, label %bb9, label %bb10

bb9:                                              ; preds = %bb8
  store i8 4, ptr %_0, align 1
  br label %bb85

bb10:                                             ; preds = %bb8
  %6 = icmp eq i32 %ek, 5
  br i1 %6, label %bb11, label %bb12

bb11:                                             ; preds = %bb10
  store i8 5, ptr %_0, align 1
  br label %bb85

bb12:                                             ; preds = %bb10
  %7 = icmp eq i32 %ek, 6
  br i1 %7, label %bb13, label %bb14

bb13:                                             ; preds = %bb12
  store i8 6, ptr %_0, align 1
  br label %bb85

bb14:                                             ; preds = %bb12
  %8 = icmp eq i32 %ek, 7
  br i1 %8, label %bb15, label %bb16

bb15:                                             ; preds = %bb14
  store i8 7, ptr %_0, align 1
  br label %bb85

bb16:                                             ; preds = %bb14
  %9 = icmp eq i32 %ek, 8
  br i1 %9, label %bb17, label %bb18

bb17:                                             ; preds = %bb16
  store i8 8, ptr %_0, align 1
  br label %bb85

bb18:                                             ; preds = %bb16
  %10 = icmp eq i32 %ek, 9
  br i1 %10, label %bb19, label %bb20

bb19:                                             ; preds = %bb18
  store i8 9, ptr %_0, align 1
  br label %bb85

bb20:                                             ; preds = %bb18
  %11 = icmp eq i32 %ek, 10
  br i1 %11, label %bb21, label %bb22

bb21:                                             ; preds = %bb20
  store i8 10, ptr %_0, align 1
  br label %bb85

bb22:                                             ; preds = %bb20
  %12 = icmp eq i32 %ek, 11
  br i1 %12, label %bb23, label %bb24

bb23:                                             ; preds = %bb22
  store i8 11, ptr %_0, align 1
  br label %bb85

bb24:                                             ; preds = %bb22
  %13 = icmp eq i32 %ek, 12
  br i1 %13, label %bb25, label %bb26

bb25:                                             ; preds = %bb24
  store i8 12, ptr %_0, align 1
  br label %bb85

bb26:                                             ; preds = %bb24
  %14 = icmp eq i32 %ek, 13
  br i1 %14, label %bb27, label %bb28

bb27:                                             ; preds = %bb26
  store i8 13, ptr %_0, align 1
  br label %bb85

bb28:                                             ; preds = %bb26
  %15 = icmp eq i32 %ek, 14
  br i1 %15, label %bb29, label %bb30

bb29:                                             ; preds = %bb28
  store i8 14, ptr %_0, align 1
  br label %bb85

bb30:                                             ; preds = %bb28
  %16 = icmp eq i32 %ek, 15
  br i1 %16, label %bb31, label %bb32

bb31:                                             ; preds = %bb30
  store i8 15, ptr %_0, align 1
  br label %bb85

bb32:                                             ; preds = %bb30
  %17 = icmp eq i32 %ek, 16
  br i1 %17, label %bb33, label %bb34

bb33:                                             ; preds = %bb32
  store i8 16, ptr %_0, align 1
  br label %bb85

bb34:                                             ; preds = %bb32
  %18 = icmp eq i32 %ek, 17
  br i1 %18, label %bb35, label %bb36

bb35:                                             ; preds = %bb34
  store i8 17, ptr %_0, align 1
  br label %bb85

bb36:                                             ; preds = %bb34
  %19 = icmp eq i32 %ek, 18
  br i1 %19, label %bb37, label %bb38

bb37:                                             ; preds = %bb36
  store i8 18, ptr %_0, align 1
  br label %bb85

bb38:                                             ; preds = %bb36
  %20 = icmp eq i32 %ek, 19
  br i1 %20, label %bb39, label %bb40

bb39:                                             ; preds = %bb38
  store i8 19, ptr %_0, align 1
  br label %bb85

bb40:                                             ; preds = %bb38
  %21 = icmp eq i32 %ek, 20
  br i1 %21, label %bb41, label %bb42

bb41:                                             ; preds = %bb40
  store i8 20, ptr %_0, align 1
  br label %bb85

bb42:                                             ; preds = %bb40
  %22 = icmp eq i32 %ek, 21
  br i1 %22, label %bb43, label %bb44

bb43:                                             ; preds = %bb42
  store i8 21, ptr %_0, align 1
  br label %bb85

bb44:                                             ; preds = %bb42
  %23 = icmp eq i32 %ek, 22
  br i1 %23, label %bb45, label %bb46

bb45:                                             ; preds = %bb44
  store i8 22, ptr %_0, align 1
  br label %bb85

bb46:                                             ; preds = %bb44
  %24 = icmp eq i32 %ek, 23
  br i1 %24, label %bb47, label %bb48

bb47:                                             ; preds = %bb46
  store i8 23, ptr %_0, align 1
  br label %bb85

bb48:                                             ; preds = %bb46
  %25 = icmp eq i32 %ek, 24
  br i1 %25, label %bb49, label %bb50

bb49:                                             ; preds = %bb48
  store i8 24, ptr %_0, align 1
  br label %bb85

bb50:                                             ; preds = %bb48
  %26 = icmp eq i32 %ek, 25
  br i1 %26, label %bb51, label %bb52

bb51:                                             ; preds = %bb50
  store i8 25, ptr %_0, align 1
  br label %bb85

bb52:                                             ; preds = %bb50
  %27 = icmp eq i32 %ek, 26
  br i1 %27, label %bb53, label %bb54

bb53:                                             ; preds = %bb52
  store i8 26, ptr %_0, align 1
  br label %bb85

bb54:                                             ; preds = %bb52
  %28 = icmp eq i32 %ek, 27
  br i1 %28, label %bb55, label %bb56

bb55:                                             ; preds = %bb54
  store i8 27, ptr %_0, align 1
  br label %bb85

bb56:                                             ; preds = %bb54
  %29 = icmp eq i32 %ek, 28
  br i1 %29, label %bb57, label %bb58

bb57:                                             ; preds = %bb56
  store i8 28, ptr %_0, align 1
  br label %bb85

bb58:                                             ; preds = %bb56
  %30 = icmp eq i32 %ek, 29
  br i1 %30, label %bb59, label %bb60

bb59:                                             ; preds = %bb58
  store i8 29, ptr %_0, align 1
  br label %bb85

bb60:                                             ; preds = %bb58
  %31 = icmp eq i32 %ek, 30
  br i1 %31, label %bb61, label %bb62

bb61:                                             ; preds = %bb60
  store i8 30, ptr %_0, align 1
  br label %bb85

bb62:                                             ; preds = %bb60
  %32 = icmp eq i32 %ek, 31
  br i1 %32, label %bb63, label %bb64

bb63:                                             ; preds = %bb62
  store i8 31, ptr %_0, align 1
  br label %bb85

bb64:                                             ; preds = %bb62
  %33 = icmp eq i32 %ek, 32
  br i1 %33, label %bb65, label %bb66

bb65:                                             ; preds = %bb64
  store i8 32, ptr %_0, align 1
  br label %bb85

bb66:                                             ; preds = %bb64
  %34 = icmp eq i32 %ek, 33
  br i1 %34, label %bb67, label %bb68

bb67:                                             ; preds = %bb66
  store i8 33, ptr %_0, align 1
  br label %bb85

bb68:                                             ; preds = %bb66
  %35 = icmp eq i32 %ek, 34
  br i1 %35, label %bb69, label %bb70

bb69:                                             ; preds = %bb68
  store i8 34, ptr %_0, align 1
  br label %bb85

bb70:                                             ; preds = %bb68
  %36 = icmp eq i32 %ek, 35
  br i1 %36, label %bb71, label %bb72

bb71:                                             ; preds = %bb70
  store i8 35, ptr %_0, align 1
  br label %bb85

bb72:                                             ; preds = %bb70
  %37 = icmp eq i32 %ek, 40
  br i1 %37, label %bb73, label %bb74

bb73:                                             ; preds = %bb72
  store i8 40, ptr %_0, align 1
  br label %bb85

bb74:                                             ; preds = %bb72
  %38 = icmp eq i32 %ek, 37
  br i1 %38, label %bb75, label %bb76

bb75:                                             ; preds = %bb74
  store i8 37, ptr %_0, align 1
  br label %bb85

bb76:                                             ; preds = %bb74
  %39 = icmp eq i32 %ek, 36
  br i1 %39, label %bb77, label %bb78

bb77:                                             ; preds = %bb76
  store i8 36, ptr %_0, align 1
  br label %bb85

bb78:                                             ; preds = %bb76
  %40 = icmp eq i32 %ek, 38
  br i1 %40, label %bb79, label %bb80

bb79:                                             ; preds = %bb78
  store i8 38, ptr %_0, align 1
  br label %bb85

bb80:                                             ; preds = %bb78
  %41 = icmp eq i32 %ek, 39
  br i1 %41, label %bb81, label %bb82

bb81:                                             ; preds = %bb80
  store i8 39, ptr %_0, align 1
  br label %bb85

bb82:                                             ; preds = %bb80
  %42 = icmp eq i32 %ek, 41
  br i1 %42, label %bb83, label %bb84

bb83:                                             ; preds = %bb82
  store i8 41, ptr %_0, align 1
  br label %bb85

bb84:                                             ; preds = %bb82
  store i8 42, ptr %_0, align 1
  br label %bb85
}

; std::io::error::repr_bitpacked::Repr::data::{{closure}}
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN3std2io5error14repr_bitpacked4Repr4data28_$u7b$$u7b$closure$u7d$$u7d$17h9d7794893e915c56E"(ptr %c) unnamed_addr #0 {
start:
  ret ptr %c
}

; std::rt::lang_start
; Function Attrs: uwtable
define hidden i64 @_ZN3std2rt10lang_start17ha17f72ac3d52a94cE(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #1 {
start:
  %_7 = alloca [8 x i8], align 8
  store ptr %main, ptr %_7, align 8
; call std::rt::lang_start_internal
  %_0 = call i64 @_RNvNtCsg55jX0GwzBC_3std2rt19lang_start_internal(ptr align 1 %_7, ptr align 8 @vtable.1, i64 %argc, ptr %argv, i8 %sigpipe)
  ret i64 %_0
}

; std::rt::lang_start::{{closure}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h6895314435e84710E"(ptr align 8 %_1) unnamed_addr #0 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace
  call void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h7a33e8f31bf94ccfE(ptr %_4) #25
; call <() as std::process::Termination>::report
  %self = call i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hd9a8d3b7111e3981E"() #21
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; std::ffi::os_str::<impl core::convert::AsRef<std::ffi::os_str::OsStr> for alloc::string::String>::as_ref
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @"_ZN3std3ffi6os_str103_$LT$impl$u20$core..convert..AsRef$LT$std..ffi..os_str..OsStr$GT$$u20$for$u20$alloc..string..String$GT$6as_ref17hf3efc0747e0bd7faE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_7 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %_6 = load i64, ptr %1, align 8
  %2 = insertvalue { ptr, i64 } poison, ptr %_7, 0
  %3 = insertvalue { ptr, i64 } %2, i64 %_6, 1
  ret { ptr, i64 } %3
}

; std::sys::fs::unix::debug_assert_fd_is_open
; Function Attrs: inlinehint uwtable
define internal void @_ZN3std3sys2fs4unix23debug_assert_fd_is_open17hee43a986c86ff907E(i32 %fd) unnamed_addr #0 {
start:
  %_5 = alloca [8 x i8], align 8
  %_4 = alloca [0 x i8], align 1
  br label %bb1

bb1:                                              ; preds = %start
  %_2 = call i32 (i32, i32, ...) @fcntl(i32 %fd, i32 1) #26
  %0 = icmp eq i32 %_2, -1
  br i1 %0, label %bb3, label %bb8

bb3:                                              ; preds = %bb1
  %_9 = call ptr @__error() #26
  %_3 = load i32, ptr %_9, align 4
  %1 = icmp eq i32 %_3, 9
  br i1 %1, label %bb4, label %bb7

bb8:                                              ; preds = %bb1
  br label %bb9

bb4:                                              ; preds = %bb3
; call std::io::Write::write_fmt
  %2 = call ptr @_ZN3std2io5Write9write_fmt17hc69cbeab8b6551b3E(ptr align 1 %_4, ptr @alloc_670745891beb4cee10ca252b3f3da521, ptr inttoptr (i64 179 to ptr))
  store ptr %2, ptr %_5, align 8
; call core::ptr::drop_in_place<core::result::Result<(),std::io::error::Error>>
  call void @"_ZN4core3ptr81drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$GT$17ha0d2151b81f17565E"(ptr align 8 %_5)
; call std::process::abort
  call void @_RNvNtCsg55jX0GwzBC_3std7process5abort() #24
  unreachable

bb7:                                              ; preds = %bb3
  br label %bb9

bb9:                                              ; preds = %bb8, %bb7
  br label %bb10

bb10:                                             ; preds = %bb9
  ret void
}

; std::sys::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline uwtable
define internal void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h7a33e8f31bf94ccfE(ptr %f) unnamed_addr #2 {
start:
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17h53fc9c41ca852277E(ptr %f) #21
  call void asm sideeffect "", "~{memory}"(), !srcloc !3
  ret void
}

; std::process::Command::new
; Function Attrs: uwtable
define internal void @_ZN3std7process7Command3new17hdfbc64f0a259b374E(ptr sret([200 x i8]) align 8 %_0, ptr align 8 %program) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_2 = alloca [200 x i8], align 8
; invoke std::ffi::os_str::<impl core::convert::AsRef<std::ffi::os_str::OsStr> for alloc::string::String>::as_ref
  %1 = invoke { ptr, i64 } @"_ZN3std3ffi6os_str103_$LT$impl$u20$core..convert..AsRef$LT$std..ffi..os_str..OsStr$GT$$u20$for$u20$alloc..string..String$GT$6as_ref17hf3efc0747e0bd7faE"(ptr align 8 %program)
          to label %bb1 unwind label %cleanup

bb4:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he154a73b91055e9aE"(ptr align 8 %program) #22
          to label %bb5 unwind label %terminate

cleanup:                                          ; preds = %bb1, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb4

bb1:                                              ; preds = %start
  %_3.0 = extractvalue { ptr, i64 } %1, 0
  %_3.1 = extractvalue { ptr, i64 } %1, 1
; invoke <std::sys::process::unix::common::Command>::new
  invoke void @_RNvMs_NtNtNtNtCsg55jX0GwzBC_3std3sys7process4unix6commonNtB4_7Command3new(ptr sret([200 x i8]) align 8 %_2, ptr align 1 %_3.0, i64 %_3.1)
          to label %bb2 unwind label %cleanup

bb2:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_2, i64 200, i1 false)
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he154a73b91055e9aE"(ptr align 8 %program)
  ret void

terminate:                                        ; preds = %bb4
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb5:                                              ; preds = %bb4
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  %9 = load i32, ptr %8, align 8
  %10 = insertvalue { ptr, i32 } poison, ptr %7, 0
  %11 = insertvalue { ptr, i32 } %10, i32 %9, 1
  resume { ptr, i32 } %11
}

; core::intrinsics::is_val_statically_known
; Function Attrs: nounwind uwtable
define internal zeroext i1 @_ZN4core10intrinsics23is_val_statically_known17h406d0c68124c11cdE(i1 zeroext %_arg) unnamed_addr #3 {
start:
  ret i1 false
}

; core::fmt::Write::write_char
; Function Attrs: uwtable
define internal zeroext i1 @_ZN4core3fmt5Write10write_char17hfcd0fb5d42725371E(ptr align 8 %self, i32 %c) unnamed_addr #1 {
start:
  %_6 = alloca [4 x i8], align 1
  call void @llvm.memset.p0.i64(ptr align 1 %_6, i8 0, i64 4, i1 false)
; call core::char::methods::encode_utf8_raw
  %0 = call { ptr, i64 } @_ZN4core4char7methods15encode_utf8_raw17h999c1f1a9f51eae9E(i32 %c, ptr align 1 %_6, i64 4) #21
  %v.0 = extractvalue { ptr, i64 } %0, 0
  %v.1 = extractvalue { ptr, i64 } %0, 1
; call <std::io::default_write_fmt::Adapter<T> as core::fmt::Write>::write_str
  %_0 = call zeroext i1 @"_ZN81_$LT$std..io..default_write_fmt..Adapter$LT$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h6ed374c6060c81d0E"(ptr align 8 %self, ptr align 1 %v.0, i64 %v.1)
  ret i1 %_0
}

; core::fmt::Write::write_fmt
; Function Attrs: uwtable
define internal zeroext i1 @_ZN4core3fmt5Write9write_fmt17habdc83ce072a46d9E(ptr align 8 %self, ptr %args.0, ptr %args.1) unnamed_addr #1 {
start:
; call <&mut W as core::fmt::Write::write_fmt::SpecWriteFmt>::spec_write_fmt
  %_0 = call zeroext i1 @"_ZN75_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write..write_fmt..SpecWriteFmt$GT$14spec_write_fmt17he1ee08f932a6c24dE"(ptr align 8 %self, ptr %args.0, ptr %args.1) #21
  ret i1 %_0
}

; core::fmt::Arguments::as_statically_known_str
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @_ZN4core3fmt9Arguments23as_statically_known_str17h16f3697c0ce049d6E(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [1 x i8], align 1
  %s = alloca [16 x i8], align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %_6 = load ptr, ptr %1, align 8
  %bits = ptrtoint ptr %_6 to i64
  %_7 = and i64 %bits, 1
  %2 = icmp eq i64 %_7, 1
  br i1 %2, label %bb6, label %bb7

bb6:                                              ; preds = %start
  %self1 = load ptr, ptr %self, align 8
  %len = lshr i64 %bits, 1
  br label %bb8

bb7:                                              ; preds = %start
  %3 = load ptr, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, align 8
  %4 = load i64, ptr getelementptr inbounds (i8, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, i64 8), align 8
  store ptr %3, ptr %s, align 8
  %5 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 %4, ptr %5, align 8
  br label %bb5

bb8:                                              ; preds = %bb6
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h72bc7164f537abccE(ptr %self1, i64 1, i64 1, i64 %len, ptr align 8 @alloc_7a51f9194d5f96080ea9d51850247e42) #19
  br label %bb10

bb10:                                             ; preds = %bb8
  store ptr %self1, ptr %s, align 8
  %6 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 %len, ptr %6, align 8
  br label %bb5

bb5:                                              ; preds = %bb7, %bb10
  %7 = load ptr, ptr %s, align 8
  %8 = getelementptr inbounds i8, ptr %s, i64 8
  %9 = load i64, ptr %8, align 8
  %10 = ptrtoint ptr %7 to i64
  %11 = icmp eq i64 %10, 0
  %_16 = select i1 %11, i64 0, i64 1
  %_3 = icmp eq i64 %_16, 1
  %12 = call i1 @llvm.is.constant.i1(i1 %_3)
  %13 = zext i1 %12 to i8
  store i8 %13, ptr %0, align 1
  %14 = load i8, ptr %0, align 1
  %_2 = trunc nuw i8 %14 to i1
  br i1 %_2, label %bb2, label %bb3

bb3:                                              ; preds = %bb5
  %15 = load ptr, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, align 8
  %16 = load i64, ptr getelementptr inbounds (i8, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, i64 8), align 8
  store ptr %15, ptr %s, align 8
  %17 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 %16, ptr %17, align 8
  br label %bb4

bb2:                                              ; preds = %bb5
  br label %bb4

bb4:                                              ; preds = %bb2, %bb3
  %18 = load ptr, ptr %s, align 8
  %19 = getelementptr inbounds i8, ptr %s, i64 8
  %20 = load i64, ptr %19, align 8
  %21 = insertvalue { ptr, i64 } poison, ptr %18, 0
  %22 = insertvalue { ptr, i64 } %21, i64 %20, 1
  ret { ptr, i64 } %22
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h2f2c9d3cbd8de4e3E"(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call core::ops::function::FnOnce::call_once
  %_0 = call i32 @_ZN4core3ops8function6FnOnce9call_once17hc27e28c0315874edE(ptr %0) #21
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17h53fc9c41ca852277E(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i32 @_ZN4core3ops8function6FnOnce9call_once17hc27e28c0315874edE(ptr %0) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::{{closure}}
  %_0 = invoke i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h6895314435e84710E"(ptr align 8 %_1)
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

; core::ptr::drop_in_place<std::io::error::ErrorData<alloc::boxed::Box<std::io::error::Custom>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr101drop_in_place$LT$std..io..error..ErrorData$LT$alloc..boxed..Box$LT$std..io..error..Custom$GT$$GT$$GT$17h19b3082c65695e7aE"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load i8, ptr %_1, align 8
  %_2 = zext i8 %0 to i64
  switch i64 %_2, label %bb2 [
    i64 0, label %bb1
    i64 1, label %bb1
    i64 2, label %bb1
  ]

bb2:                                              ; preds = %start
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
; call core::ptr::drop_in_place<alloc::boxed::Box<std::io::error::Custom>>
  call void @"_ZN4core3ptr68drop_in_place$LT$alloc..boxed..Box$LT$std..io..error..Custom$GT$$GT$17h74b7464dd3221182E"(ptr align 8 %1)
  br label %bb1

bb1:                                              ; preds = %bb2, %start, %start, %start
  ret void
}

; core::ptr::drop_in_place<alloc::boxed::Box<dyn core::error::Error+core::marker::Sync+core::marker::Send>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr118drop_in_place$LT$alloc..boxed..Box$LT$dyn$u20$core..error..Error$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$17hfb130acc66122ed2E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6.0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %_6.1 = load ptr, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %_6.1, i64 0
  %3 = load ptr, ptr %2, align 8, !invariant.load !4
  %4 = icmp ne ptr %3, null
  br i1 %4, label %is_not_null, label %bb3

is_not_null:                                      ; preds = %start
  invoke void %3(ptr %_6.0)
          to label %bb3 unwind label %cleanup

bb3:                                              ; preds = %is_not_null, %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h588c89ca19c4fbd6E"(ptr align 8 %_1) #21
  ret void

bb4:                                              ; preds = %cleanup
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h588c89ca19c4fbd6E"(ptr align 8 %_1) #22
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %is_not_null
  %5 = landingpad { ptr, i32 }
          cleanup
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
  store ptr %6, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %7, ptr %8, align 8
  br label %bb4

terminate:                                        ; preds = %bb4
  %9 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb4
  %10 = load ptr, ptr %0, align 8
  %11 = getelementptr inbounds i8, ptr %0, i64 8
  %12 = load i32, ptr %11, align 8
  %13 = insertvalue { ptr, i32 } poison, ptr %10, 0
  %14 = insertvalue { ptr, i32 } %13, i32 %12, 1
  resume { ptr, i32 } %14
}

; core::ptr::drop_in_place<alloc::collections::btree::map::BTreeMap<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr152drop_in_place$LT$alloc..collections..btree..map..BTreeMap$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17hb3ad95002b1888ecE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::collections::btree::map::BTreeMap<K,V,A> as core::ops::drop::Drop>::drop
  call void @"_ZN99_$LT$alloc..collections..btree..map..BTreeMap$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h2b7d51c5cc3fb0dfE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::collections::btree::map::IntoIter<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr152drop_in_place$LT$alloc..collections..btree..map..IntoIter$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17hb68047b497a6efbdE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop
  call void @"_ZN99_$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h1ff518a7ee2d1eefE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17hd6b4225b5a6a79c8E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd4f2e7bd58139e8bE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr203drop_in_place$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$17h9c2a67896483b404E"(ptr align 1 %_1.0, ptr align 8 %_1.1) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %_1.1, i64 0
  %1 = load ptr, ptr %0, align 8, !invariant.load !4
  %2 = icmp ne ptr %1, null
  br i1 %2, label %is_not_null, label %bb1

is_not_null:                                      ; preds = %start
  call void %1(ptr %_1.0) #21
  br label %bb1

bb1:                                              ; preds = %is_not_null, %start
  ret void
}

; core::ptr::drop_in_place<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr228drop_in_place$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$17h045e5391b14ec966E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6.0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %_6.1 = load ptr, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %_6.1, i64 0
  %3 = load ptr, ptr %2, align 8, !invariant.load !4
  %4 = icmp ne ptr %3, null
  br i1 %4, label %is_not_null, label %bb3

is_not_null:                                      ; preds = %start
  invoke void %3(ptr %_6.0)
          to label %bb3 unwind label %cleanup

bb3:                                              ; preds = %is_not_null, %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h969baa51dfa67748E"(ptr align 8 %_1) #21
  ret void

bb4:                                              ; preds = %cleanup
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h969baa51dfa67748E"(ptr align 8 %_1) #22
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %is_not_null
  %5 = landingpad { ptr, i32 }
          cleanup
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
  store ptr %6, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %7, ptr %8, align 8
  br label %bb4

terminate:                                        ; preds = %bb4
  %9 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb4
  %10 = load ptr, ptr %0, align 8
  %11 = getelementptr inbounds i8, ptr %0, i64 8
  %12 = load i32, ptr %11, align 8
  %13 = insertvalue { ptr, i32 } poison, ptr %10, 0
  %14 = insertvalue { ptr, i32 } %13, i32 %12, 1
  resume { ptr, i32 } %14
}

; core::ptr::drop_in_place<[alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr238drop_in_place$LT$$u5b$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$u5d$$GT$17h8676cb3bc051b2b1E"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
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
  %_6 = getelementptr inbounds nuw %"alloc::boxed::Box<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>
  invoke void @"_ZN4core3ptr228drop_in_place$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$17h045e5391b14ec966E"(ptr align 8 %_6)
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
  %_4 = getelementptr inbounds nuw %"alloc::boxed::Box<dyn core::ops::function::FnMut() -> core::result::Result<(), std::io::error::Error> + core::marker::Send + core::marker::Sync>", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>
  invoke void @"_ZN4core3ptr228drop_in_place$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$17h045e5391b14ec966E"(ptr align 8 %_4) #22
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
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable
}

; core::ptr::drop_in_place<<alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>,alloc::alloc::Global>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr250drop_in_place$LT$$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$C$alloc..alloc..Global$GT$$GT$17hdc20274c8d168defE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <<alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop::DropGuard<K,V,A> as core::ops::drop::Drop>::drop
  call void @"_ZN174_$LT$$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h25db7b4f4a612004E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr251drop_in_place$LT$alloc..vec..Vec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17hba64280c8b2f082dE"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3f578f1ebdb22535E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
  invoke void @"_ZN4core3ptr258drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17h9761dfef1dcaeedaE"(ptr align 8 %_1) #22
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
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
  call void @"_ZN4core3ptr258drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17h9761dfef1dcaeedaE"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr258drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17h9761dfef1dcaeedaE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h5b91aa02db46e479E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val::Dropper<core::option::Option<std::ffi::os_str::OsString>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr312drop_in_place$LT$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$..drop_key_val..Dropper$LT$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17h3e98ba6ad7dbe731E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val::Dropper<T> as core::ops::drop::Drop>::drop
  call void @"_ZN280_$LT$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$..drop_key_val..Dropper$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4f9d13ef07bd4e1eE"(ptr align 8 %_1) #21
  ret void
}

; core::ptr::drop_in_place<std::env::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17heb4d1ac16b5f8206E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::env::ArgsOs>
  call void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17h1ff5c9d42c231229E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::env::ArgsOs>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17h1ff5c9d42c231229E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::args::common::Args>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..args..common..Args$GT$17hdec574a9ea93d9a6E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::string::String>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he154a73b91055e9aE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17hee116d23331a8df2E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::io::error::Error>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr42drop_in_place$LT$std..io..error..Error$GT$17hbf06dc6e9bf4f7e0E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::io::error::repr_bitpacked::Repr>
  call void @"_ZN4core3ptr57drop_in_place$LT$std..io..error..repr_bitpacked..Repr$GT$17hacbb2612d8a2b5f0E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::process::Command>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr42drop_in_place$LT$std..process..Command$GT$17h0ab653a9bcd4d1e4E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::process::unix::common::Command>
  call void @"_ZN4core3ptr61drop_in_place$LT$std..sys..process..unix..common..Command$GT$17h2038fa09ec53299cE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::io::error::Custom>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr43drop_in_place$LT$std..io..error..Custom$GT$17h2463f27c9c511750E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::boxed::Box<dyn core::error::Error+core::marker::Sync+core::marker::Send>>
  call void @"_ZN4core3ptr118drop_in_place$LT$alloc..boxed..Box$LT$dyn$u20$core..error..Error$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$17hfb130acc66122ed2E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17hee116d23331a8df2E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<u8> as core::ops::drop::Drop>::drop
  invoke void @_RNvXso_NtCs1OjIl8oxbrv_5alloc3vecINtB5_3VechENtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4dropCscUtGwbhD4WH_5gimli(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  invoke void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hc5b5dc1ba47d29bcE"(ptr align 8 %_1) #22
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
  call void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hc5b5dc1ba47d29bcE"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<alloc::ffi::c_str::CString>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h4c0b66d7332eebc0E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::ffi::c_str::CString as core::ops::drop::Drop>::drop
  invoke void @"_ZN68_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4e26a65b5556f848E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::boxed::Box<[u8]>>
  invoke void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h162b8a087b0747bcE"(ptr align 8 %_1) #22
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
  call void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h162b8a087b0747bcE"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<std::ffi::os_str::OsString>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17hfff473a3034bbccdE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h0ff40dbabe66838eE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::os::fd::owned::OwnedFd>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr48drop_in_place$LT$std..os..fd..owned..OwnedFd$GT$17h06b7231b1bb91fe6E"(ptr align 4 %_1) unnamed_addr #1 {
start:
; call <std::os::fd::owned::OwnedFd as core::ops::drop::Drop>::drop
  call void @"_ZN69_$LT$std..os..fd..owned..OwnedFd$u20$as$u20$core..ops..drop..Drop$GT$4drop17h85c614e29f0a5643E"(ptr align 4 %_1) #21
  ret void
}

; core::ptr::drop_in_place<std::sys::args::common::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..args..common..Args$GT$17hdec574a9ea93d9a6E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17h5fe6d379d04242f6E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::fd::unix::FileDesc>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..fd..unix..FileDesc$GT$17h39bd0b21f22bda1cE"(ptr align 4 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::os::fd::owned::OwnedFd>
  call void @"_ZN4core3ptr48drop_in_place$LT$std..os..fd..owned..OwnedFd$GT$17h06b7231b1bb91fe6E"(ptr align 4 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h0ff40dbabe66838eE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17hee116d23331a8df2E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17hc5b5dc1ba47d29bcE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<u8> as core::ops::drop::Drop>::drop
  call void @_RNvXs1_NtCs1OjIl8oxbrv_5alloc7raw_vecINtB5_6RawVechENtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4dropCscUtGwbhD4WH_5gimli(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::process::env::CommandEnv>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr55drop_in_place$LT$std..sys..process..env..CommandEnv$GT$17hf2df2e9ac43d7297E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::collections::btree::map::BTreeMap<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>>>
  call void @"_ZN4core3ptr152drop_in_place$LT$alloc..collections..btree..map..BTreeMap$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17hb3ad95002b1888ecE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17h45398f46b2529b7dE"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
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
  %_6 = getelementptr inbounds nuw %"std::ffi::os_str::OsString", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17hfff473a3034bbccdE"(ptr align 8 %_6)
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
  %_4 = getelementptr inbounds nuw %"std::ffi::os_str::OsString", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17hfff473a3034bbccdE"(ptr align 8 %_4) #22
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
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable
}

; core::ptr::drop_in_place<std::io::error::repr_bitpacked::Repr>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr57drop_in_place$LT$std..io..error..repr_bitpacked..Repr$GT$17hacbb2612d8a2b5f0E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <std::io::error::repr_bitpacked::Repr as core::ops::drop::Drop>::drop
  call void @"_ZN78_$LT$std..io..error..repr_bitpacked..Repr$u20$as$u20$core..ops..drop..Drop$GT$4drop17h2e6ef5378d1bc1ecE"(ptr align 8 %_1) #21
  ret void
}

; core::ptr::drop_in_place<alloc::boxed::Box<[u8]>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr58drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u8$u5d$$GT$$GT$17h162b8a087b0747bcE"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6.0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %_6.1 = load i64, ptr %1, align 8
  br label %bb3

bb3:                                              ; preds = %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h98b8d573cc986ceeE"(ptr align 8 %_1) #21
  ret void

bb4:                                              ; No predecessors!
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h98b8d573cc986ceeE"(ptr align 8 %_1) #22
          to label %bb1 unwind label %terminate

terminate:                                        ; preds = %bb4
  %2 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb4
  %3 = load ptr, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  %5 = load i32, ptr %4, align 8
  %6 = insertvalue { ptr, i32 } poison, ptr %3, 0
  %7 = insertvalue { ptr, i32 } %6, i32 %5, 1
  resume { ptr, i32 } %7
}

; core::ptr::drop_in_place<alloc::boxed::Box<[u32]>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr59drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u32$u5d$$GT$$GT$17hd171cad13bafcb17E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6.0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %_6.1 = load i64, ptr %1, align 8
  br label %bb3

bb3:                                              ; preds = %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h7b8f9a55c191995fE"(ptr align 8 %_1) #21
  ret void

bb4:                                              ; No predecessors!
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h7b8f9a55c191995fE"(ptr align 8 %_1) #22
          to label %bb1 unwind label %terminate

terminate:                                        ; preds = %bb4
  %2 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb4
  %3 = load ptr, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  %5 = load i32, ptr %4, align 8
  %6 = insertvalue { ptr, i32 } poison, ptr %3, 0
  %7 = insertvalue { ptr, i32 } %6, i32 %5, 1
  resume { ptr, i32 } %7
}

; core::ptr::drop_in_place<std::sys::process::unix::common::Stdio>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr59drop_in_place$LT$std..sys..process..unix..common..Stdio$GT$17hea6a39faa6efcbc0E"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %0 = load i32, ptr %_1, align 4
  %1 = getelementptr inbounds i8, ptr %_1, i64 4
  %2 = load i32, ptr %1, align 4
  %_2 = zext i32 %0 to i64
  %3 = icmp eq i64 %_2, 3
  br i1 %3, label %bb2, label %bb1

bb2:                                              ; preds = %start
  %4 = getelementptr inbounds i8, ptr %_1, i64 4
; call core::ptr::drop_in_place<std::sys::fd::unix::FileDesc>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..fd..unix..FileDesc$GT$17h39bd0b21f22bda1cE"(ptr align 4 %4)
  br label %bb1

bb1:                                              ; preds = %bb2, %start
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<*const i8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr60drop_in_place$LT$alloc..vec..Vec$LT$$BP$const$u20$i8$GT$$GT$17hb7d0858469a9daebE"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h920154bf79b14a07E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<*const i8>>
  invoke void @"_ZN4core3ptr67drop_in_place$LT$alloc..raw_vec..RawVec$LT$$BP$const$u20$i8$GT$$GT$17hddf64d374a182918E"(ptr align 8 %_1) #22
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
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<*const i8>>
  call void @"_ZN4core3ptr67drop_in_place$LT$alloc..raw_vec..RawVec$LT$$BP$const$u20$i8$GT$$GT$17hddf64d374a182918E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<std::sys::process::unix::common::Command>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr61drop_in_place$LT$std..sys..process..unix..common..Command$GT$17h2038fa09ec53299cE"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 128
; invoke core::ptr::drop_in_place<alloc::ffi::c_str::CString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h4c0b66d7332eebc0E"(ptr align 8 %1)
          to label %bb20 unwind label %cleanup

bb11:                                             ; preds = %cleanup
; invoke core::ptr::drop_in_place<std::sys::process::unix::common::cstring_array::CStringArray>
  invoke void @"_ZN4core3ptr81drop_in_place$LT$std..sys..process..unix..common..cstring_array..CStringArray$GT$17h7a99728c593dca71E"(ptr align 8 %_1) #22
          to label %bb10 unwind label %terminate

cleanup:                                          ; preds = %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb11

bb20:                                             ; preds = %start
; invoke core::ptr::drop_in_place<std::sys::process::unix::common::cstring_array::CStringArray>
  invoke void @"_ZN4core3ptr81drop_in_place$LT$std..sys..process..unix..common..cstring_array..CStringArray$GT$17h7a99728c593dca71E"(ptr align 8 %_1)
          to label %bb19 unwind label %cleanup1

bb10:                                             ; preds = %bb11, %cleanup1
  %6 = getelementptr inbounds i8, ptr %_1, i64 96
; invoke core::ptr::drop_in_place<std::sys::process::env::CommandEnv>
  invoke void @"_ZN4core3ptr55drop_in_place$LT$std..sys..process..env..CommandEnv$GT$17hf2df2e9ac43d7297E"(ptr align 8 %6) #22
          to label %bb9 unwind label %terminate

cleanup1:                                         ; preds = %bb20
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %0, align 8
  %10 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb10

bb19:                                             ; preds = %bb20
  %11 = getelementptr inbounds i8, ptr %_1, i64 96
; invoke core::ptr::drop_in_place<std::sys::process::env::CommandEnv>
  invoke void @"_ZN4core3ptr55drop_in_place$LT$std..sys..process..env..CommandEnv$GT$17hf2df2e9ac43d7297E"(ptr align 8 %11)
          to label %bb18 unwind label %cleanup2

bb9:                                              ; preds = %bb10, %cleanup2
  %12 = getelementptr inbounds i8, ptr %_1, i64 144
; invoke core::ptr::drop_in_place<core::option::Option<alloc::ffi::c_str::CString>>
  invoke void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$alloc..ffi..c_str..CString$GT$$GT$17h6ac08077715d3cb8E"(ptr align 8 %12) #22
          to label %bb8 unwind label %terminate

cleanup2:                                         ; preds = %bb19
  %13 = landingpad { ptr, i32 }
          cleanup
  %14 = extractvalue { ptr, i32 } %13, 0
  %15 = extractvalue { ptr, i32 } %13, 1
  store ptr %14, ptr %0, align 8
  %16 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %15, ptr %16, align 8
  br label %bb9

bb18:                                             ; preds = %bb19
  %17 = getelementptr inbounds i8, ptr %_1, i64 144
; invoke core::ptr::drop_in_place<core::option::Option<alloc::ffi::c_str::CString>>
  invoke void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$alloc..ffi..c_str..CString$GT$$GT$17h6ac08077715d3cb8E"(ptr align 8 %17)
          to label %bb17 unwind label %cleanup3

bb8:                                              ; preds = %bb9, %cleanup3
  %18 = getelementptr inbounds i8, ptr %_1, i64 160
; invoke core::ptr::drop_in_place<core::option::Option<alloc::ffi::c_str::CString>>
  invoke void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$alloc..ffi..c_str..CString$GT$$GT$17h6ac08077715d3cb8E"(ptr align 8 %18) #22
          to label %bb7 unwind label %terminate

cleanup3:                                         ; preds = %bb18
  %19 = landingpad { ptr, i32 }
          cleanup
  %20 = extractvalue { ptr, i32 } %19, 0
  %21 = extractvalue { ptr, i32 } %19, 1
  store ptr %20, ptr %0, align 8
  %22 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %21, ptr %22, align 8
  br label %bb8

bb17:                                             ; preds = %bb18
  %23 = getelementptr inbounds i8, ptr %_1, i64 160
; invoke core::ptr::drop_in_place<core::option::Option<alloc::ffi::c_str::CString>>
  invoke void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$alloc..ffi..c_str..CString$GT$$GT$17h6ac08077715d3cb8E"(ptr align 8 %23)
          to label %bb16 unwind label %cleanup4

bb7:                                              ; preds = %bb8, %cleanup4
  %24 = getelementptr inbounds i8, ptr %_1, i64 24
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
  invoke void @"_ZN4core3ptr251drop_in_place$LT$alloc..vec..Vec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17hba64280c8b2f082dE"(ptr align 8 %24) #22
          to label %bb6 unwind label %terminate

cleanup4:                                         ; preds = %bb17
  %25 = landingpad { ptr, i32 }
          cleanup
  %26 = extractvalue { ptr, i32 } %25, 0
  %27 = extractvalue { ptr, i32 } %25, 1
  store ptr %26, ptr %0, align 8
  %28 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %27, ptr %28, align 8
  br label %bb7

bb16:                                             ; preds = %bb17
  %29 = getelementptr inbounds i8, ptr %_1, i64 24
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>>>
  invoke void @"_ZN4core3ptr251drop_in_place$LT$alloc..vec..Vec$LT$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$GT$$GT$17hba64280c8b2f082dE"(ptr align 8 %29)
          to label %bb15 unwind label %cleanup5

bb6:                                              ; preds = %bb7, %cleanup5
  %30 = getelementptr inbounds i8, ptr %_1, i64 176
; invoke core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<[u32]>>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$alloc..boxed..Box$LT$$u5b$u32$u5d$$GT$$GT$$GT$17hf3c39e9608ef7e7cE"(ptr align 8 %30) #22
          to label %bb5 unwind label %terminate

cleanup5:                                         ; preds = %bb16
  %31 = landingpad { ptr, i32 }
          cleanup
  %32 = extractvalue { ptr, i32 } %31, 0
  %33 = extractvalue { ptr, i32 } %31, 1
  store ptr %32, ptr %0, align 8
  %34 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %33, ptr %34, align 8
  br label %bb6

bb15:                                             ; preds = %bb16
  %35 = getelementptr inbounds i8, ptr %_1, i64 176
; invoke core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<[u32]>>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$alloc..boxed..Box$LT$$u5b$u32$u5d$$GT$$GT$$GT$17hf3c39e9608ef7e7cE"(ptr align 8 %35)
          to label %bb14 unwind label %cleanup6

bb5:                                              ; preds = %bb6, %cleanup6
  %36 = getelementptr inbounds i8, ptr %_1, i64 72
; invoke core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %36) #22
          to label %bb4 unwind label %terminate

cleanup6:                                         ; preds = %bb15
  %37 = landingpad { ptr, i32 }
          cleanup
  %38 = extractvalue { ptr, i32 } %37, 0
  %39 = extractvalue { ptr, i32 } %37, 1
  store ptr %38, ptr %0, align 8
  %40 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %39, ptr %40, align 8
  br label %bb5

bb14:                                             ; preds = %bb15
  %41 = getelementptr inbounds i8, ptr %_1, i64 72
; invoke core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %41)
          to label %bb13 unwind label %cleanup7

bb4:                                              ; preds = %bb5, %cleanup7
  %42 = getelementptr inbounds i8, ptr %_1, i64 80
; invoke core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %42) #22
          to label %bb3 unwind label %terminate

cleanup7:                                         ; preds = %bb14
  %43 = landingpad { ptr, i32 }
          cleanup
  %44 = extractvalue { ptr, i32 } %43, 0
  %45 = extractvalue { ptr, i32 } %43, 1
  store ptr %44, ptr %0, align 8
  %46 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %45, ptr %46, align 8
  br label %bb4

bb13:                                             ; preds = %bb14
  %47 = getelementptr inbounds i8, ptr %_1, i64 80
; invoke core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %47)
          to label %bb12 unwind label %cleanup8

bb3:                                              ; preds = %bb4, %cleanup8
  %48 = getelementptr inbounds i8, ptr %_1, i64 88
; invoke core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  invoke void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %48) #22
          to label %bb1 unwind label %terminate

cleanup8:                                         ; preds = %bb13
  %49 = landingpad { ptr, i32 }
          cleanup
  %50 = extractvalue { ptr, i32 } %49, 0
  %51 = extractvalue { ptr, i32 } %49, 1
  store ptr %50, ptr %0, align 8
  %52 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %51, ptr %52, align 8
  br label %bb3

bb12:                                             ; preds = %bb13
  %53 = getelementptr inbounds i8, ptr %_1, i64 88
; call core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
  call void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %53)
  ret void

terminate:                                        ; preds = %bb3, %bb4, %bb5, %bb6, %bb7, %bb8, %bb9, %bb10, %bb11
  %54 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %55 = load ptr, ptr %0, align 8
  %56 = getelementptr inbounds i8, ptr %0, i64 8
  %57 = load i32, ptr %56, align 8
  %58 = insertvalue { ptr, i32 } poison, ptr %55, 0
  %59 = insertvalue { ptr, i32 } %58, i32 %57, 1
  resume { ptr, i32 } %59
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<*const i8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr67drop_in_place$LT$alloc..raw_vec..RawVec$LT$$BP$const$u20$i8$GT$$GT$17hddf64d374a182918E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h9b5fb7669e1418a6E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::boxed::Box<std::io::error::Custom>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr68drop_in_place$LT$alloc..boxed..Box$LT$std..io..error..Custom$GT$$GT$17h74b7464dd3221182E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6 = load ptr, ptr %_1, align 8
; invoke core::ptr::drop_in_place<std::io::error::Custom>
  invoke void @"_ZN4core3ptr43drop_in_place$LT$std..io..error..Custom$GT$17h2463f27c9c511750E"(ptr align 8 %_6)
          to label %bb3 unwind label %cleanup

bb4:                                              ; preds = %cleanup
; invoke <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd510c2283e6ae505E"(ptr align 8 %_1) #22
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb4

bb3:                                              ; preds = %start
; call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd510c2283e6ae505E"(ptr align 8 %_1) #21
  ret void

terminate:                                        ; preds = %bb4
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb4
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<alloc::collections::btree::mem::replace::PanicGuard>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr72drop_in_place$LT$alloc..collections..btree..mem..replace..PanicGuard$GT$17h4d26e39a8d66cab6E"(ptr align 1 %_1) unnamed_addr #1 {
start:
; call <alloc::collections::btree::mem::replace::PanicGuard as core::ops::drop::Drop>::drop
  call void @"_ZN93_$LT$alloc..collections..btree..mem..replace..PanicGuard$u20$as$u20$core..ops..drop..Drop$GT$4drop17h90be998b891f4008E"(ptr align 1 %_1)
  ret void
}

; core::ptr::drop_in_place<core::option::Option<alloc::ffi::c_str::CString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$alloc..ffi..c_str..CString$GT$$GT$17h6ac08077715d3cb8E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %2 = load i64, ptr %1, align 8
  %3 = ptrtoint ptr %0 to i64
  %4 = icmp eq i64 %3, 0
  %_2 = select i1 %4, i64 0, i64 1
  %5 = icmp eq i64 %_2, 0
  br i1 %5, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::ffi::c_str::CString>
  call void @"_ZN4core3ptr47drop_in_place$LT$alloc..ffi..c_str..CString$GT$17h4c0b66d7332eebc0E"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<core::option::Option<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr75drop_in_place$LT$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$17hcde64bce32fbc5c6E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load i64, ptr %_1, align 8
  %1 = icmp eq i64 %0, -9223372036854775808
  %_2 = select i1 %1, i64 0, i64 1
  %2 = icmp eq i64 %_2, 0
  br i1 %2, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<std::ffi::os_str::OsString>
  call void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17hfff473a3034bbccdE"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17hb76fb17fa1eab069E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hb052267a9a60429bE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<core::result::Result<(),std::io::error::Error>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr81drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$GT$17ha0d2151b81f17565E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load ptr, ptr %_1, align 8
  %1 = ptrtoint ptr %0 to i64
  %2 = icmp eq i64 %1, 0
  %_2 = select i1 %2, i64 0, i64 1
  %3 = icmp eq i64 %_2, 0
  br i1 %3, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<std::io::error::Error>
  call void @"_ZN4core3ptr42drop_in_place$LT$std..io..error..Error$GT$17hbf06dc6e9bf4f7e0E"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<std::sys::process::unix::common::cstring_array::CStringArray>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr81drop_in_place$LT$std..sys..process..unix..common..cstring_array..CStringArray$GT$17h7a99728c593dca71E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <std::sys::process::unix::common::cstring_array::CStringArray as core::ops::drop::Drop>::drop
  invoke void @_RNvXs3_NtNtNtNtNtCsg55jX0GwzBC_3std3sys7process4unix6common13cstring_arrayNtB5_12CStringArrayNtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4drop(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::vec::Vec<*const i8>>
  invoke void @"_ZN4core3ptr60drop_in_place$LT$alloc..vec..Vec$LT$$BP$const$u20$i8$GT$$GT$17hb7d0858469a9daebE"(ptr align 8 %_1) #22
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
; call core::ptr::drop_in_place<alloc::vec::Vec<*const i8>>
  call void @"_ZN4core3ptr60drop_in_place$LT$alloc..vec..Vec$LT$$BP$const$u20$i8$GT$$GT$17hb7d0858469a9daebE"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb1:                                              ; preds = %bb3
  %6 = load ptr, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  %8 = load i32, ptr %7, align 8
  %9 = insertvalue { ptr, i32 } poison, ptr %6, 0
  %10 = insertvalue { ptr, i32 } %9, i32 %8, 1
  resume { ptr, i32 } %10
}

; core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17h5fe6d379d04242f6E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17he193351050f4b390E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<[u32]>>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$alloc..boxed..Box$LT$$u5b$u32$u5d$$GT$$GT$$GT$17hf3c39e9608ef7e7cE"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load ptr, ptr %_1, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 8
  %2 = load i64, ptr %1, align 8
  %3 = ptrtoint ptr %0 to i64
  %4 = icmp eq i64 %3, 0
  %_2 = select i1 %4, i64 0, i64 1
  %5 = icmp eq i64 %_2, 0
  br i1 %5, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::boxed::Box<[u32]>>
  call void @"_ZN4core3ptr59drop_in_place$LT$alloc..boxed..Box$LT$$u5b$u32$u5d$$GT$$GT$17hd171cad13bafcb17E"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<core::option::Option<std::sys::process::unix::common::Stdio>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr87drop_in_place$LT$core..option..Option$LT$std..sys..process..unix..common..Stdio$GT$$GT$17h47bc7d09cfad4a77E"(ptr align 4 %_1) unnamed_addr #1 {
start:
  %0 = load i32, ptr %_1, align 4
  %1 = getelementptr inbounds i8, ptr %_1, i64 4
  %2 = load i32, ptr %1, align 4
  %3 = icmp eq i32 %0, 5
  %_2 = select i1 %3, i64 0, i64 1
  %4 = icmp eq i64 %_2, 0
  br i1 %4, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<std::sys::process::unix::common::Stdio>
  call void @"_ZN4core3ptr59drop_in_place$LT$std..sys..process..unix..common..Stdio$GT$17hea6a39faa6efcbc0E"(ptr align 4 %_1)
  br label %bb1
}

; core::ptr::non_null::NonNull<T>::new_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$13new_unchecked18precondition_check17h893a6acd6dd8e9ceE"(ptr %ptr, ptr align 8 %0) unnamed_addr #4 {
start:
  %_5 = ptrtoint ptr %ptr to i64
  %1 = icmp eq i64 %_5, 0
  br i1 %1, label %bb1, label %bb2

bb1:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_560a59ed819b9d9a5841f6e731c4c8e5, ptr inttoptr (i64 421 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb2:                                              ; preds = %start
  ret void
}

; core::ptr::drop_in_place<dyn core::error::Error+core::marker::Sync+core::marker::Send>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr93drop_in_place$LT$dyn$u20$core..error..Error$u2b$core..marker..Sync$u2b$core..marker..Send$GT$17h71f9bb84aa7bad90E"(ptr align 1 %_1.0, ptr align 8 %_1.1) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %_1.1, i64 0
  %1 = load ptr, ptr %0, align 8, !invariant.load !4
  %2 = icmp ne ptr %1, null
  br i1 %2, label %is_not_null, label %bb1

is_not_null:                                      ; preds = %start
  call void %1(ptr %_1.0) #21
  br label %bb1

bb1:                                              ; preds = %is_not_null, %start
  ret void
}

; core::ptr::drop_in_place<std::io::default_write_fmt::Adapter<std::sys::stdio::unix::Stderr>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr93drop_in_place$LT$std..io..default_write_fmt..Adapter$LT$std..sys..stdio..unix..Stderr$GT$$GT$17ha94dfba2c791a5f3E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %_1, i64 8
; call core::ptr::drop_in_place<core::result::Result<(),std::io::error::Error>>
  call void @"_ZN4core3ptr81drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$GT$17ha0d2151b81f17565E"(ptr align 8 %0)
  ret void
}

; core::ptr::drop_in_place<core::result::Result<std::process::ExitStatus,std::io::error::Error>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr97drop_in_place$LT$core..result..Result$LT$std..process..ExitStatus$C$std..io..error..Error$GT$$GT$17hecf567af432d4777E"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load i32, ptr %_1, align 8
  %_2 = zext i32 %0 to i64
  %1 = icmp eq i64 %_2, 0
  br i1 %1, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %_1, i64 8
; call core::ptr::drop_in_place<std::io::error::Error>
  call void @"_ZN4core3ptr42drop_in_place$LT$std..io..error..Error$GT$17hbf06dc6e9bf4f7e0E"(ptr align 8 %2)
  br label %bb1
}

; core::ptr::alignment::Alignment::new_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 %0) unnamed_addr #4 {
start:
  %1 = alloca [4 x i8], align 4
  %2 = call i64 @llvm.ctpop.i64(i64 %align)
  %3 = trunc i64 %2 to i32
  store i32 %3, ptr %1, align 4
  %_5 = load i32, ptr %1, align 4
  %4 = icmp eq i32 %_5, 1
  br i1 %4, label %bb1, label %bb2

bb1:                                              ; preds = %start
  ret void

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_0625062a5eee489a7813ee965a38d15a, ptr inttoptr (i64 397 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable
}

; core::ptr::const_ptr::<impl *const T>::is_aligned_to
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$13is_aligned_to17hac82ab1715dee212E"(ptr %self, i64 %align) unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  %1 = call i64 @llvm.ctpop.i64(i64 %align)
  %2 = trunc i64 %1 to i32
  store i32 %2, ptr %0, align 4
  %_8 = load i32, ptr %0, align 4
  %3 = icmp eq i32 %_8, 1
  br i1 %3, label %bb1, label %bb2

bb1:                                              ; preds = %start
  %_6 = ptrtoint ptr %self to i64
  %_7 = sub i64 %align, 1
  %_5 = and i64 %_6, %_7
  %_0 = icmp eq i64 %_5, 0
  ret i1 %_0

bb2:                                              ; preds = %start
; call core::panicking::panic_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking9panic_fmt(ptr @alloc_fad0cd83b7d1858a846a172eb260e593, ptr inttoptr (i64 85 to ptr), ptr align 8 @alloc_4e4cc478ba2b105df6f8c26c45900512) #20
  unreachable
}

; core::ptr::const_ptr::<impl *const T>::offset_from_unsigned::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$20offset_from_unsigned18precondition_check17habad995d57f98f8fE"(ptr %this, ptr %origin, ptr align 8 %0) unnamed_addr #4 {
start:
  %_3 = icmp uge ptr %this, %origin
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_de4e626d456b04760e72bc785ed7e52a, ptr inttoptr (i64 403 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; core::char::methods::encode_utf8_raw
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @_ZN4core4char7methods15encode_utf8_raw17h999c1f1a9f51eae9E(i32 %code, ptr align 1 %dst.0, i64 %dst.1) unnamed_addr #0 {
start:
  %len = alloca [8 x i8], align 8
  %_11 = icmp ult i32 %code, 128
  br i1 %_11, label %bb10, label %bb5

bb5:                                              ; preds = %start
  %_12 = icmp ult i32 %code, 2048
  br i1 %_12, label %bb9, label %bb6

bb10:                                             ; preds = %start
  store i64 1, ptr %len, align 8
  br label %bb4

bb6:                                              ; preds = %bb5
  %_13 = icmp ult i32 %code, 65536
  br i1 %_13, label %bb8, label %bb7

bb9:                                              ; preds = %bb5
  store i64 2, ptr %len, align 8
  br label %bb4

bb7:                                              ; preds = %bb6
  store i64 4, ptr %len, align 8
  br label %bb4

bb8:                                              ; preds = %bb6
  store i64 3, ptr %len, align 8
  br label %bb4

bb4:                                              ; preds = %bb10, %bb9, %bb8, %bb7
  %_6 = load i64, ptr %len, align 8
  %_4 = icmp ult i64 %dst.1, %_6
  br i1 %_4, label %bb1, label %bb2

bb2:                                              ; preds = %bb4
; call core::char::methods::encode_utf8_raw_unchecked
  call void @_ZN4core4char7methods25encode_utf8_raw_unchecked17hc4c023008803737fE(i32 %code, ptr %dst.0) #21
  br label %bb11

bb1:                                              ; preds = %bb4
  %0 = load i64, ptr %len, align 8
; call core::char::methods::encode_utf8_raw::do_panic::runtime
  call void @_ZN4core4char7methods15encode_utf8_raw8do_panic7runtime17h8dff1e6773e16e75E(i32 %code, i64 %0, i64 %dst.1, ptr align 8 @alloc_88fbea7d0d583f188a306688d9d39b2c) #28
  unreachable

bb11:                                             ; preds = %bb2
  %1 = load i64, ptr %len, align 8
; call core::slice::raw::from_raw_parts_mut::precondition_check
  call void @_ZN4core5slice3raw18from_raw_parts_mut18precondition_check17haeb36e1aeeac61deE(ptr %dst.0, i64 1, i64 1, i64 %1, ptr align 8 @alloc_f6549330ef387f3ed559c7abaf671c9e) #19
  br label %bb13

bb13:                                             ; preds = %bb11
  %_18.1 = load i64, ptr %len, align 8
  %2 = insertvalue { ptr, i64 } poison, ptr %dst.0, 0
  %3 = insertvalue { ptr, i64 } %2, i64 %_18.1, 1
  ret { ptr, i64 } %3
}

; core::char::methods::encode_utf8_raw::do_panic::runtime
; Function Attrs: inlinehint noreturn uwtable
define internal void @_ZN4core4char7methods15encode_utf8_raw8do_panic7runtime17h8dff1e6773e16e75E(i32 %0, i64 %1, i64 %2, ptr align 8 %3) unnamed_addr #5 {
start:
  %_24 = alloca [16 x i8], align 8
  %_19 = alloca [16 x i8], align 8
  %_14 = alloca [16 x i8], align 8
  %_12 = alloca [16 x i8], align 8
  %_11 = alloca [16 x i8], align 8
  %_10 = alloca [16 x i8], align 8
  %args = alloca [48 x i8], align 8
  %dst_len = alloca [8 x i8], align 8
  %len = alloca [8 x i8], align 8
  %code = alloca [4 x i8], align 4
  store i32 %0, ptr %code, align 4
  store i64 %1, ptr %len, align 8
  store i64 %2, ptr %dst_len, align 8
  store ptr %len, ptr %_14, align 8
  %4 = getelementptr inbounds i8, ptr %_14, i64 8
  store ptr @_RNvXsi_NtNtNtCsl8K0bEFm1U0_4core3fmt3num3impjNtB9_7Display3fmt, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_10, ptr align 8 %_14, i64 16, i1 false)
  store ptr %code, ptr %_19, align 8
  %5 = getelementptr inbounds i8, ptr %_19, i64 8
  store ptr @_RNvXsw_NtNtCsl8K0bEFm1U0_4core3fmt3nummNtB7_8UpperHex3fmt, ptr %5, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_11, ptr align 8 %_19, i64 16, i1 false)
  store ptr %dst_len, ptr %_24, align 8
  %6 = getelementptr inbounds i8, ptr %_24, i64 8
  store ptr @_RNvXsi_NtNtNtCsl8K0bEFm1U0_4core3fmt3num3impjNtB9_7Display3fmt, ptr %6, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_12, ptr align 8 %_24, i64 16, i1 false)
  %7 = getelementptr inbounds nuw %"core::fmt::rt::Argument<'_>", ptr %args, i64 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7, ptr align 8 %_10, i64 16, i1 false)
  %8 = getelementptr inbounds nuw %"core::fmt::rt::Argument<'_>", ptr %args, i64 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %8, ptr align 8 %_11, i64 16, i1 false)
  %9 = getelementptr inbounds nuw %"core::fmt::rt::Argument<'_>", ptr %args, i64 2
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %9, ptr align 8 %_12, i64 16, i1 false)
; call core::panicking::panic_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking9panic_fmt(ptr @alloc_2800f9fd9aa1f3e24fde08ad5f3cf8ec, ptr %args, ptr align 8 %3) #20
  unreachable
}

; core::char::methods::encode_utf8_raw_unchecked
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core4char7methods25encode_utf8_raw_unchecked17hc4c023008803737fE(i32 %code, ptr %dst) unnamed_addr #0 {
start:
  %len = alloca [8 x i8], align 8
  %_26 = icmp ult i32 %code, 128
  br i1 %_26, label %bb12, label %bb7

bb7:                                              ; preds = %start
  %_27 = icmp ult i32 %code, 2048
  br i1 %_27, label %bb11, label %bb8

bb12:                                             ; preds = %start
  store i64 1, ptr %len, align 8
  %0 = trunc i32 %code to i8
  store i8 %0, ptr %dst, align 1
  br label %bb5

bb8:                                              ; preds = %bb7
  %_28 = icmp ult i32 %code, 65536
  br i1 %_28, label %bb10, label %bb9

bb11:                                             ; preds = %bb7
  store i64 2, ptr %len, align 8
  br label %bb13

bb9:                                              ; preds = %bb8
  store i64 4, ptr %len, align 8
  br label %bb13

bb10:                                             ; preds = %bb8
  store i64 3, ptr %len, align 8
  br label %bb13

bb13:                                             ; preds = %bb11, %bb10, %bb9
  %_6 = and i32 %code, 63
  %_5 = trunc i32 %_6 to i8
  %last1 = or i8 %_5, -128
  %_10 = lshr i32 %code, 6
  %_9 = and i32 %_10, 63
  %_8 = trunc i32 %_9 to i8
  %last2 = or i8 %_8, -128
  %_14 = lshr i32 %code, 12
  %_13 = and i32 %_14, 63
  %_12 = trunc i32 %_13 to i8
  %last3 = or i8 %_12, -128
  %_18 = lshr i32 %code, 18
  %_17 = and i32 %_18, 63
  %_16 = trunc i32 %_17 to i8
  %last4 = or i8 %_16, -16
  %_19 = load i64, ptr %len, align 8
  %1 = icmp eq i64 %_19, 2
  br i1 %1, label %bb1, label %bb2

bb1:                                              ; preds = %bb13
  %2 = or i8 %last2, -64
  store i8 %2, ptr %dst, align 1
  %_20 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  store i8 %last1, ptr %_20, align 1
  br label %bb5

bb2:                                              ; preds = %bb13
  %3 = load i64, ptr %len, align 8
  %4 = icmp eq i64 %3, 3
  br i1 %4, label %bb3, label %bb4

bb5:                                              ; preds = %bb12, %bb3, %bb1
  br label %bb6

bb3:                                              ; preds = %bb2
  %5 = or i8 %last3, -32
  store i8 %5, ptr %dst, align 1
  %_21 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  store i8 %last2, ptr %_21, align 1
  %_22 = getelementptr inbounds nuw i8, ptr %dst, i64 2
  store i8 %last1, ptr %_22, align 1
  br label %bb5

bb4:                                              ; preds = %bb2
  store i8 %last4, ptr %dst, align 1
  %_23 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  store i8 %last3, ptr %_23, align 1
  %_24 = getelementptr inbounds nuw i8, ptr %dst, i64 2
  store i8 %last2, ptr %_24, align 1
  %_25 = getelementptr inbounds nuw i8, ptr %dst, i64 3
  store i8 %last1, ptr %_25, align 1
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  ret void
}

; core::hint::unreachable_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core4hint21unreachable_unchecked18precondition_check17h3199eaece5aa5bf6E(ptr align 8 %0) unnamed_addr #4 {
start:
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_75fb06c2453febd814e73f5f2e72ae38, ptr inttoptr (i64 399 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable
}

; core::iter::traits::iterator::Iterator::advance_by
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core4iter6traits8iterator8Iterator10advance_by17hbcee17bb24184f17E(ptr align 8 %self, i64 %n) unnamed_addr #0 {
start:
; call <I as core::iter::traits::iterator::Iterator::advance_by::SpecAdvanceBy>::spec_advance_by
  %_0 = call i64 @"_ZN87_$LT$I$u20$as$u20$core..iter..traits..iterator..Iterator..advance_by..SpecAdvanceBy$GT$15spec_advance_by17hdcc7acec5d579568E"(ptr align 8 %self, i64 %n)
  ret i64 %_0
}

; core::iter::traits::iterator::Iterator::nth
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core4iter6traits8iterator8Iterator3nth17hb010d30a7e72e147E(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self, i64 %n) unnamed_addr #0 {
start:
  %self1 = alloca [1 x i8], align 1
; call core::iter::traits::iterator::Iterator::advance_by
  %self2 = call i64 @_ZN4core4iter6traits8iterator8Iterator10advance_by17hbcee17bb24184f17E(ptr align 8 %self, i64 %n) #21
  %0 = icmp eq i64 %self2, 0
  %_6 = select i1 %0, i64 0, i64 1
  %1 = trunc nuw i64 %_6 to i1
  br i1 %1, label %bb4, label %bb5

bb4:                                              ; preds = %start
  store i8 0, ptr %self1, align 1
  store i64 -9223372036854775808, ptr %_0, align 8
  br label %bb3

bb5:                                              ; preds = %start
  store i8 1, ptr %self1, align 1
; call <std::env::Args as core::iter::traits::iterator::Iterator>::next
  call void @_RNvXsa_NtCsg55jX0GwzBC_3std3envNtB5_4ArgsNtNtNtNtCsl8K0bEFm1U0_4core4iter6traits8iterator8Iterator4next(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self)
  br label %bb3

bb3:                                              ; preds = %bb4, %bb5
  ret void

bb2:                                              ; No predecessors!
  unreachable
}

; core::iter::traits::iterator::Iterator::try_fold
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core4iter6traits8iterator8Iterator8try_fold17h5de16c88a03504d2E(ptr align 8 %self, i64 %init) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_17 = alloca [1 x i8], align 1
  %_11 = alloca [32 x i8], align 8
  %_8 = alloca [8 x i8], align 8
  %x = alloca [24 x i8], align 8
  %_5 = alloca [24 x i8], align 8
  %accum = alloca [8 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  %f = alloca [0 x i8], align 1
  store i8 1, ptr %_17, align 1
  store i64 %init, ptr %accum, align 8
  br label %bb1

bb1:                                              ; preds = %bb7, %start
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::next
  invoke void @_RNvXsa_NtCsg55jX0GwzBC_3std3envNtB5_4ArgsNtNtNtNtCsl8K0bEFm1U0_4core4iter6traits8iterator8Iterator4next(ptr sret([24 x i8]) align 8 %_5, ptr align 8 %self)
          to label %bb2 unwind label %cleanup

bb16:                                             ; preds = %cleanup
  %1 = load i8, ptr %_17, align 1
  %2 = trunc nuw i8 %1 to i1
  br i1 %2, label %bb15, label %bb13

cleanup:                                          ; preds = %bb10, %bb8, %bb4, %bb3, %bb1
  %3 = landingpad { ptr, i32 }
          cleanup
  %4 = extractvalue { ptr, i32 } %3, 0
  %5 = extractvalue { ptr, i32 } %3, 1
  store ptr %4, ptr %0, align 8
  %6 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %5, ptr %6, align 8
  br label %bb16

bb2:                                              ; preds = %bb1
  %7 = load i64, ptr %_5, align 8
  %8 = icmp eq i64 %7, -9223372036854775808
  %_6 = select i1 %8, i64 0, i64 1
  %9 = trunc nuw i64 %_6 to i1
  br i1 %9, label %bb3, label %bb10

bb3:                                              ; preds = %bb2
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %x, ptr align 8 %_5, i64 24, i1 false)
  store i8 0, ptr %_17, align 1
  %_12 = load i64, ptr %accum, align 8
  store i64 %_12, ptr %_11, align 8
  %10 = getelementptr inbounds i8, ptr %_11, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %10, ptr align 8 %x, i64 24, i1 false)
  %11 = load i64, ptr %_11, align 8
  %12 = getelementptr inbounds i8, ptr %_11, i64 8
; invoke <I as core::iter::traits::iterator::Iterator::advance_by::SpecAdvanceBy>::spec_advance_by::{{closure}}
  %_9 = invoke i64 @"_ZN87_$LT$I$u20$as$u20$core..iter..traits..iterator..Iterator..advance_by..SpecAdvanceBy$GT$15spec_advance_by28_$u7b$$u7b$closure$u7d$$u7d$17h8c521b01281198a7E"(ptr align 1 %f, i64 %11, ptr align 8 %12)
          to label %bb4 unwind label %cleanup

bb10:                                             ; preds = %bb2
  store i8 0, ptr %_17, align 1
  %_16 = load i64, ptr %accum, align 8
; invoke <core::option::Option<T> as core::ops::try_trait::Try>::from_output
  %13 = invoke i64 @"_ZN75_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..Try$GT$11from_output17h534749e82034c4d7E"(i64 %_16)
          to label %bb11 unwind label %cleanup

bb4:                                              ; preds = %bb3
; invoke <core::option::Option<T> as core::ops::try_trait::Try>::branch
  %14 = invoke i64 @"_ZN75_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..Try$GT$6branch17h06ef99bacb1b3dc8E"(i64 %_9)
          to label %bb5 unwind label %cleanup

bb5:                                              ; preds = %bb4
  store i64 %14, ptr %_8, align 8
  %15 = load i64, ptr %_8, align 8
  %16 = icmp eq i64 %15, 0
  %_13 = select i1 %16, i64 1, i64 0
  %17 = trunc nuw i64 %_13 to i1
  br i1 %17, label %bb8, label %bb7

bb8:                                              ; preds = %bb5
; invoke <core::option::Option<T> as core::ops::try_trait::FromResidual<core::option::Option<core::convert::Infallible>>>::from_residual
  %18 = invoke i64 @"_ZN145_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..FromResidual$LT$core..option..Option$LT$core..convert..Infallible$GT$$GT$$GT$13from_residual17h71f6be104875a5f4E"()
          to label %bb9 unwind label %cleanup

bb7:                                              ; preds = %bb5
  %val = load i64, ptr %_8, align 8
  store i8 1, ptr %_17, align 1
  store i64 %val, ptr %accum, align 8
  br label %bb1

bb9:                                              ; preds = %bb8
  store i64 %18, ptr %_0, align 8
  br label %bb12

bb12:                                             ; preds = %bb11, %bb9
  %19 = load i64, ptr %_0, align 8
  ret i64 %19

bb11:                                             ; preds = %bb10
  store i64 %13, ptr %_0, align 8
  br label %bb12

bb6:                                              ; No predecessors!
  unreachable

bb13:                                             ; preds = %bb15, %bb16
  %20 = load ptr, ptr %0, align 8
  %21 = getelementptr inbounds i8, ptr %0, i64 8
  %22 = load i32, ptr %21, align 8
  %23 = insertvalue { ptr, i32 } poison, ptr %20, 0
  %24 = insertvalue { ptr, i32 } %23, i32 %22, 1
  resume { ptr, i32 } %24

bb15:                                             ; preds = %bb16
  br label %bb13
}

; core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %alignment, ptr align 8 %0) unnamed_addr #4 {
start:
  %_7 = sub nuw i64 -9223372036854775808, %alignment
  %_3 = icmp ule i64 %size, %_7
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_8cf3531fd3c828ab298b686071a40c17, ptr inttoptr (i64 519 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; core::slice::raw::from_raw_parts::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5slice3raw14from_raw_parts18precondition_check17h72bc7164f537abccE(ptr %data, i64 %size, i64 %align, i64 %len, ptr align 8 %0) unnamed_addr #4 personality ptr @rust_eh_personality {
start:
  %max_len = alloca [8 x i8], align 8
; invoke core::ptr::const_ptr::<impl *const T>::is_aligned_to
  %_11 = invoke zeroext i1 @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$13is_aligned_to17hac82ab1715dee212E"(ptr %data, i64 %align)
          to label %bb8 unwind label %terminate

terminate:                                        ; preds = %start
  %1 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_cannot_unwind
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking19panic_cannot_unwind() #23
  unreachable

bb8:                                              ; preds = %start
  br i1 %_11, label %bb6, label %bb7

bb7:                                              ; preds = %bb8
  br label %bb4

bb6:                                              ; preds = %bb8
  %_13 = ptrtoint ptr %data to i64
  %_12 = icmp eq i64 %_13, 0
  %_5 = xor i1 %_12, true
  br i1 %_5, label %bb1, label %bb4

bb4:                                              ; preds = %bb6, %bb7
  br label %bb5

bb1:                                              ; preds = %bb6
  %2 = icmp eq i64 %size, 0
  br i1 %2, label %bb9, label %bb10

bb5:                                              ; preds = %bb3, %bb4
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_a28e8c8fd5088943a8b5d44af697ff83, ptr inttoptr (i64 559 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb9:                                              ; preds = %bb1
  store i64 -1, ptr %max_len, align 8
  br label %bb11

bb10:                                             ; preds = %bb1
  %3 = udiv i64 9223372036854775807, %size
  store i64 %3, ptr %max_len, align 8
  br label %bb11

bb11:                                             ; preds = %bb10, %bb9
  %4 = load i64, ptr %max_len, align 8
  %_7 = icmp ule i64 %len, %4
  br i1 %_7, label %bb2, label %bb3

bb3:                                              ; preds = %bb11
  br label %bb5

bb2:                                              ; preds = %bb11
  ret void
}

; core::slice::raw::from_raw_parts_mut::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5slice3raw18from_raw_parts_mut18precondition_check17haeb36e1aeeac61deE(ptr %data, i64 %size, i64 %align, i64 %len, ptr align 8 %0) unnamed_addr #4 personality ptr @rust_eh_personality {
start:
  %max_len = alloca [8 x i8], align 8
; invoke core::ptr::const_ptr::<impl *const T>::is_aligned_to
  %_11 = invoke zeroext i1 @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$13is_aligned_to17hac82ab1715dee212E"(ptr %data, i64 %align)
          to label %bb8 unwind label %terminate

terminate:                                        ; preds = %start
  %1 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_cannot_unwind
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking19panic_cannot_unwind() #23
  unreachable

bb8:                                              ; preds = %start
  br i1 %_11, label %bb6, label %bb7

bb7:                                              ; preds = %bb8
  br label %bb4

bb6:                                              ; preds = %bb8
  %_13 = ptrtoint ptr %data to i64
  %_12 = icmp eq i64 %_13, 0
  %_5 = xor i1 %_12, true
  br i1 %_5, label %bb1, label %bb4

bb4:                                              ; preds = %bb6, %bb7
  br label %bb5

bb1:                                              ; preds = %bb6
  %2 = icmp eq i64 %size, 0
  br i1 %2, label %bb9, label %bb10

bb5:                                              ; preds = %bb3, %bb4
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_5c1a2f972552229672fc942406cfc298, ptr inttoptr (i64 567 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb9:                                              ; preds = %bb1
  store i64 -1, ptr %max_len, align 8
  br label %bb11

bb10:                                             ; preds = %bb1
  %3 = udiv i64 9223372036854775807, %size
  store i64 %3, ptr %max_len, align 8
  br label %bb11

bb11:                                             ; preds = %bb10, %bb9
  %4 = load i64, ptr %max_len, align 8
  %_7 = icmp ule i64 %len, %4
  br i1 %_7, label %bb2, label %bb3

bb3:                                              ; preds = %bb11
  br label %bb5

bb2:                                              ; preds = %bb11
  ret void
}

; core::option::Option<T>::unwrap_or_default
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core6option15Option$LT$T$GT$17unwrap_or_default17he6cc1db57abb6876E"(ptr sret([24 x i8]) align 8 %x, ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = load i64, ptr %self, align 8
  %1 = icmp eq i64 %0, -9223372036854775808
  %_2 = select i1 %1, i64 0, i64 1
  %2 = trunc nuw i64 %_2 to i1
  br i1 %2, label %bb3, label %bb2

bb3:                                              ; preds = %start
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %x, ptr align 8 %self, i64 24, i1 false)
  br label %bb4

bb2:                                              ; preds = %start
; call <alloc::string::String as core::default::Default>::default
  call void @"_ZN64_$LT$alloc..string..String$u20$as$u20$core..default..Default$GT$7default17h83fb7bf9f8b41871E"(ptr sret([24 x i8]) align 8 %x) #21
  br label %bb4

bb4:                                              ; preds = %bb3, %bb2
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17hd9a8d3b7111e3981E"() unnamed_addr #0 {
start:
  ret i8 0
}

; <alloc::alloc::Global as core::clone::Clone>::clone
; Function Attrs: inlinehint uwtable
define internal void @"_ZN59_$LT$alloc..alloc..Global$u20$as$u20$core..clone..Clone$GT$5clone17ha9b8da7de1d857b7E"(ptr align 1 %self) unnamed_addr #0 {
start:
  ret void
}

; alloc::collections::btree::map::IntoIter<K,V,A>::dying_next
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree3map25IntoIter$LT$K$C$V$C$A$GT$10dying_next17hc659d595af5fbe87E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #1 {
start:
  %_7 = alloca [24 x i8], align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 64
  %_2 = load i64, ptr %0, align 8
  %1 = icmp eq i64 %_2, 0
  br i1 %1, label %bb1, label %bb4

bb1:                                              ; preds = %start
  %_6 = getelementptr inbounds i8, ptr %self, i64 72
; call <alloc::alloc::Global as core::clone::Clone>::clone
  call void @"_ZN59_$LT$alloc..alloc..Global$u20$as$u20$core..clone..Clone$GT$5clone17ha9b8da7de1d857b7E"(ptr align 1 %_6) #21
; call alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::deallocating_end
  call void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$16deallocating_end17hfad789336ed5e8d8E"(ptr align 8 %self) #21
  store ptr null, ptr %_0, align 8
  br label %bb7

bb4:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %self, i64 64
  %3 = getelementptr inbounds i8, ptr %self, i64 64
  %4 = load i64, ptr %3, align 8
  %5 = sub i64 %4, 1
  store i64 %5, ptr %2, align 8
  %_10 = getelementptr inbounds i8, ptr %self, i64 72
; call <alloc::alloc::Global as core::clone::Clone>::clone
  call void @"_ZN59_$LT$alloc..alloc..Global$u20$as$u20$core..clone..Clone$GT$5clone17ha9b8da7de1d857b7E"(ptr align 1 %_10) #21
; call alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::deallocating_next_unchecked
  call void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$27deallocating_next_unchecked17h396b99105fd34df8E"(ptr sret([24 x i8]) align 8 %_7, ptr align 8 %self) #21
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_7, i64 24, i1 false)
  br label %bb7

bb7:                                              ; preds = %bb4, %bb1
  ret void
}

; alloc::collections::btree::mem::replace
; Function Attrs: inlinehint uwtable
define internal void @_ZN5alloc11collections5btree3mem7replace17h7b5cabfd90b3c4f0E(ptr sret([24 x i8]) align 8 %ret, ptr align 8 %v) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_7 = alloca [24 x i8], align 8
  %_6 = alloca [48 x i8], align 8
  %new_value = alloca [24 x i8], align 8
  %value = alloca [24 x i8], align 8
  %_3 = alloca [0 x i8], align 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %value, ptr align 8 %v, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_7, ptr align 8 %value, i64 24, i1 false)
; invoke alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_next_unchecked::{{closure}}
  invoke void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$27deallocating_next_unchecked28_$u7b$$u7b$closure$u7d$$u7d$17h930b67439ab469abE"(ptr sret([48 x i8]) align 8 %_6, ptr align 8 %_7)
          to label %bb1 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::collections::btree::mem::replace::PanicGuard>
  invoke void @"_ZN4core3ptr72drop_in_place$LT$alloc..collections..btree..mem..replace..PanicGuard$GT$17h4d26e39a8d66cab6E"(ptr align 1 %_3) #22
          to label %bb2 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb1:                                              ; preds = %start
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %new_value, ptr align 8 %_6, i64 24, i1 false)
  %5 = getelementptr inbounds i8, ptr %_6, i64 24
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %ret, ptr align 8 %5, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v, ptr align 8 %new_value, i64 24, i1 false)
  ret void

terminate:                                        ; preds = %bb3
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb2:                                              ; preds = %bb3
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  %9 = load i32, ptr %8, align 8
  %10 = insertvalue { ptr, i32 } poison, ptr %7, 0
  %11 = insertvalue { ptr, i32 } %10, i32 %9, 1
  resume { ptr, i32 } %11
}

; alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::LeafOrInternal>::deallocate_and_ascend
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree4node127NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..LeafOrInternal$GT$21deallocate_and_ascend17h54953052088af786E"(ptr sret([24 x i8]) align 8 %ret, ptr %self.0, i64 %self.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %x = alloca [24 x i8], align 8
  %_9 = alloca [16 x i8], align 8
  %self = alloca [24 x i8], align 8
  %alloc = alloca [0 x i8], align 1
; invoke alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>::ascend
  invoke void @"_ZN5alloc11collections5btree4node40NodeRef$LT$BorrowType$C$K$C$V$C$Type$GT$6ascend17h40db0ea0a967d2eaE"(ptr sret([24 x i8]) align 8 %self, ptr %self.0, i64 %self.1)
          to label %bb1 unwind label %cleanup

bb7:                                              ; preds = %cleanup
  %1 = load ptr, ptr %0, align 8
  %2 = getelementptr inbounds i8, ptr %0, i64 8
  %3 = load i32, ptr %2, align 8
  %4 = insertvalue { ptr, i32 } poison, ptr %1, 0
  %5 = insertvalue { ptr, i32 } %4, i32 %3, 1
  resume { ptr, i32 } %5

cleanup:                                          ; preds = %bb4, %start
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  br label %bb7

bb1:                                              ; preds = %start
  %10 = load ptr, ptr %self, align 8
  %11 = ptrtoint ptr %10 to i64
  %12 = icmp eq i64 %11, 0
  %_11 = select i1 %12, i64 1, i64 0
  %13 = trunc nuw i64 %_11 to i1
  br i1 %13, label %bb10, label %bb11

bb10:                                             ; preds = %bb1
  store ptr null, ptr %ret, align 8
  br label %bb12

bb11:                                             ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %x, ptr align 8 %self, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %ret, ptr align 8 %x, i64 24, i1 false)
  br label %bb12

bb12:                                             ; preds = %bb10, %bb11
  %_10 = icmp ugt i64 %self.1, 0
  br i1 %_10, label %bb2, label %bb3

bb3:                                              ; preds = %bb12
  store i64 8, ptr %_9, align 8
  %14 = getelementptr inbounds i8, ptr %_9, i64 8
  store i64 544, ptr %14, align 8
  br label %bb4

bb2:                                              ; preds = %bb12
  store i64 8, ptr %_9, align 8
  %15 = getelementptr inbounds i8, ptr %_9, i64 8
  store i64 640, ptr %15, align 8
  br label %bb4

bb4:                                              ; preds = %bb2, %bb3
  %16 = load i64, ptr %_9, align 8
  %17 = getelementptr inbounds i8, ptr %_9, i64 8
  %18 = load i64, ptr %17, align 8
; invoke <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  invoke void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %alloc, ptr %self.0, i64 %16, i64 %18)
          to label %bb5 unwind label %cleanup

bb5:                                              ; preds = %bb4
  ret void

bb9:                                              ; No predecessors!
  unreachable
}

; alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val
; Function Attrs: inlinehint uwtable
define internal void @"_ZN5alloc11collections5btree4node173Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$12drop_key_val17h8689e92183180c52E"(ptr align 8 %self) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_guard = alloca [8 x i8], align 8
  %_13 = load ptr, ptr %self, align 8
  %_5 = getelementptr inbounds i8, ptr %_13, i64 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %index = load i64, ptr %1, align 8
  br label %bb4

bb4:                                              ; preds = %start
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hc09057a7a020f57dE"(i64 %index, i64 11, ptr align 8 @alloc_32339b860b6b3b1fb6c38cd0d6eecaf7) #19
  br label %bb5

bb5:                                              ; preds = %bb4
  %key = getelementptr inbounds nuw %"core::mem::maybe_uninit::MaybeUninit<std::ffi::os_str::OsString>", ptr %_5, i64 %index
  %_9 = getelementptr inbounds i8, ptr %_13, i64 272
  br label %bb6

bb6:                                              ; preds = %bb5
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hc09057a7a020f57dE"(i64 %index, i64 11, ptr align 8 @alloc_ecdfd039b9ddf961aa4157d4266b5990) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  %_17 = getelementptr inbounds nuw %"core::mem::maybe_uninit::MaybeUninit<core::option::Option<std::ffi::os_str::OsString>>", ptr %_9, i64 %index
  store ptr %_17, ptr %_guard, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17hfff473a3034bbccdE"(ptr align 8 %key)
          to label %bb8 unwind label %cleanup

bb2:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val::Dropper<core::option::Option<std::ffi::os_str::OsString>>>
  invoke void @"_ZN4core3ptr312drop_in_place$LT$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$..drop_key_val..Dropper$LT$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17h3e98ba6ad7dbe731E"(ptr align 8 %_guard) #22
          to label %bb3 unwind label %terminate

cleanup:                                          ; preds = %bb7
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb2

bb8:                                              ; preds = %bb7
; call core::ptr::drop_in_place<alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val::Dropper<core::option::Option<std::ffi::os_str::OsString>>>
  call void @"_ZN4core3ptr312drop_in_place$LT$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$..drop_key_val..Dropper$LT$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17h3e98ba6ad7dbe731E"(ptr align 8 %_guard)
  ret void

terminate:                                        ; preds = %bb2
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb3:                                              ; preds = %bb2
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  %9 = load i32, ptr %8, align 8
  %10 = insertvalue { ptr, i32 } poison, ptr %7, 0
  %11 = insertvalue { ptr, i32 } %10, i32 %9, 1
  resume { ptr, i32 } %11
}

; alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>::ascend
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree4node40NodeRef$LT$BorrowType$C$K$C$V$C$Type$GT$6ascend17h40db0ea0a967d2eaE"(ptr sret([24 x i8]) align 8 %_0, ptr %0, i64 %1) unnamed_addr #1 {
start:
  %f3 = alloca [8 x i8], align 8
  %f = alloca [8 x i8], align 8
  %v = alloca [24 x i8], align 8
  %_22 = alloca [2 x i8], align 2
  %_15 = alloca [24 x i8], align 8
  %err = alloca [16 x i8], align 8
  %_9 = alloca [8 x i8], align 8
  %_8 = alloca [8 x i8], align 8
  %_7 = alloca [8 x i8], align 8
  %self2 = alloca [8 x i8], align 8
  %self1 = alloca [24 x i8], align 8
  %leaf_ptr = alloca [8 x i8], align 8
  %self = alloca [16 x i8], align 8
  store ptr %0, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  store i64 %1, ptr %2, align 8
  %_11 = load ptr, ptr %self, align 8
  store ptr %_11, ptr %leaf_ptr, align 8
  %3 = load ptr, ptr %leaf_ptr, align 8
  %4 = load ptr, ptr %3, align 8
  store ptr %4, ptr %_7, align 8
  %5 = load ptr, ptr %_7, align 8
  %6 = ptrtoint ptr %5 to i64
  %7 = icmp eq i64 %6, 0
  %_12 = select i1 %7, i64 0, i64 1
  %8 = trunc nuw i64 %_12 to i1
  br i1 %8, label %bb3, label %bb2

bb3:                                              ; preds = %start
  store ptr %_7, ptr %self2, align 8
  %9 = getelementptr inbounds i8, ptr %self, i64 8
  store ptr %9, ptr %_8, align 8
  store ptr %leaf_ptr, ptr %_9, align 8
  %10 = load ptr, ptr %_8, align 8
  store ptr %10, ptr %f, align 8
  %11 = load ptr, ptr %_9, align 8
  store ptr %11, ptr %f3, align 8
  %x = load ptr, ptr %self2, align 8
  %_17 = load ptr, ptr %x, align 8
  %12 = getelementptr inbounds i8, ptr %self, i64 8
  %_19 = load i64, ptr %12, align 8
  %_18 = add i64 %_19, 1
  %13 = load ptr, ptr %leaf_ptr, align 8
  %14 = getelementptr inbounds i8, ptr %13, i64 536
  %15 = load i16, ptr %14, align 8
  store i16 %15, ptr %_22, align 2
  %_21 = load i16, ptr %_22, align 2
  %_20 = zext i16 %_21 to i64
  store ptr %_17, ptr %_15, align 8
  %16 = getelementptr inbounds i8, ptr %_15, i64 8
  store i64 %_18, ptr %16, align 8
  %17 = getelementptr inbounds i8, ptr %_15, i64 16
  store i64 %_20, ptr %17, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %self1, ptr align 8 %_15, i64 24, i1 false)
  %18 = load ptr, ptr %self, align 8
  %19 = getelementptr inbounds i8, ptr %self, i64 8
  %20 = load i64, ptr %19, align 8
  store ptr %18, ptr %err, align 8
  %21 = getelementptr inbounds i8, ptr %err, i64 8
  store i64 %20, ptr %21, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v, ptr align 8 %self1, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %v, i64 24, i1 false)
  br label %bb4

bb2:                                              ; preds = %start
  store ptr null, ptr %self2, align 8
  %22 = getelementptr inbounds i8, ptr %self, i64 8
  store ptr %22, ptr %_8, align 8
  store ptr %leaf_ptr, ptr %_9, align 8
  %23 = load ptr, ptr %_8, align 8
  store ptr %23, ptr %f, align 8
  %24 = load ptr, ptr %_9, align 8
  store ptr %24, ptr %f3, align 8
  store ptr null, ptr %self1, align 8
  %25 = load ptr, ptr %self, align 8
  %26 = getelementptr inbounds i8, ptr %self, i64 8
  %27 = load i64, ptr %26, align 8
  store ptr %25, ptr %err, align 8
  %28 = getelementptr inbounds i8, ptr %err, i64 8
  store i64 %27, ptr %28, align 8
  %29 = load ptr, ptr %self, align 8
  %30 = getelementptr inbounds i8, ptr %self, i64 8
  %31 = load i64, ptr %30, align 8
  %32 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr %29, ptr %32, align 8
  %33 = getelementptr inbounds i8, ptr %32, i64 8
  store i64 %31, ptr %33, align 8
  store ptr null, ptr %_0, align 8
  br label %bb4

bb4:                                              ; preds = %bb3, %bb2
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree8navigate235_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$BorrowType$C$K$C$V$C$alloc..collections..btree..node..marker..LeafOrInternal$GT$$C$alloc..collections..btree..node..marker..KV$GT$$GT$14next_leaf_edge17h2e87acb9de082749E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #1 {
start:
  %_22 = alloca [24 x i8], align 8
  %self1 = alloca [8 x i8], align 8
  %node = alloca [8 x i8], align 8
  %_7 = alloca [24 x i8], align 8
  %_5 = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %_2 = alloca [32 x i8], align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %self2 = load i64, ptr %0, align 8
  %self3 = load ptr, ptr %self, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %self4 = load i64, ptr %1, align 8
  %2 = icmp eq i64 %self2, 0
  br i1 %2, label %bb2, label %bb3

bb2:                                              ; preds = %start
  %3 = getelementptr inbounds i8, ptr %_3, i64 8
  store ptr %self3, ptr %3, align 8
  %4 = getelementptr inbounds i8, ptr %3, i64 8
  store i64 0, ptr %4, align 8
  store i64 0, ptr %_3, align 8
  %5 = getelementptr inbounds i8, ptr %_3, i64 8
  %node.0 = load ptr, ptr %5, align 8
  %6 = getelementptr inbounds i8, ptr %5, i64 8
  %node.1 = load i64, ptr %6, align 8
  store ptr %node.0, ptr %_5, align 8
  %7 = getelementptr inbounds i8, ptr %_5, i64 8
  store i64 %node.1, ptr %7, align 8
  %8 = getelementptr inbounds i8, ptr %_5, i64 16
  store i64 %self4, ptr %8, align 8
  %9 = getelementptr inbounds i8, ptr %_2, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %9, ptr align 8 %_5, i64 24, i1 false)
  store i64 0, ptr %_2, align 8
  %10 = getelementptr inbounds i8, ptr %_2, i64 8
  %leaf_kv.0 = load ptr, ptr %10, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 8
  %leaf_kv.1 = load i64, ptr %11, align 8
  %12 = getelementptr inbounds i8, ptr %_2, i64 8
  %13 = getelementptr inbounds i8, ptr %12, i64 16
  %leaf_kv = load i64, ptr %13, align 8
  %_11 = add i64 %leaf_kv, 1
  store ptr %leaf_kv.0, ptr %_0, align 8
  %14 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %leaf_kv.1, ptr %14, align 8
  %15 = getelementptr inbounds i8, ptr %_0, i64 16
  store i64 %_11, ptr %15, align 8
  br label %bb1

bb3:                                              ; preds = %start
  %16 = getelementptr inbounds i8, ptr %_3, i64 8
  store ptr %self3, ptr %16, align 8
  %17 = getelementptr inbounds i8, ptr %16, i64 8
  store i64 %self2, ptr %17, align 8
  store i64 1, ptr %_3, align 8
  %18 = getelementptr inbounds i8, ptr %_3, i64 8
  %node.05 = load ptr, ptr %18, align 8
  %19 = getelementptr inbounds i8, ptr %18, i64 8
  %node.16 = load i64, ptr %19, align 8
  store ptr %node.05, ptr %_7, align 8
  %20 = getelementptr inbounds i8, ptr %_7, i64 8
  store i64 %node.16, ptr %20, align 8
  %21 = getelementptr inbounds i8, ptr %_7, i64 16
  store i64 %self4, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %_2, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %22, ptr align 8 %_7, i64 24, i1 false)
  store i64 1, ptr %_2, align 8
  %23 = getelementptr inbounds i8, ptr %_2, i64 8
  %24 = getelementptr inbounds i8, ptr %23, i64 8
  %internal_kv = load i64, ptr %24, align 8
  %25 = getelementptr inbounds i8, ptr %_2, i64 8
  %internal_kv7 = load ptr, ptr %25, align 8
  %26 = getelementptr inbounds i8, ptr %_2, i64 8
  %27 = getelementptr inbounds i8, ptr %26, i64 16
  %internal_kv8 = load i64, ptr %27, align 8
  %next_internal_edge = add i64 %internal_kv8, 1
  %_16 = getelementptr inbounds i8, ptr %internal_kv7, i64 544
  br label %bb4

bb1:                                              ; preds = %bb7, %bb2
  ret void

bb4:                                              ; preds = %bb3
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$13get_unchecked18precondition_check17hb223293d499b03feE"(i64 %next_internal_edge, i64 12, ptr align 8 @alloc_22025f2193ce509b60b3c47e41728876) #19
  br label %bb5

bb5:                                              ; preds = %bb4
  %_20 = icmp ult i64 %next_internal_edge, 12
  %self9 = getelementptr inbounds nuw ptr, ptr %_16, i64 %next_internal_edge
  %28 = load ptr, ptr %self9, align 8
  store ptr %28, ptr %node, align 8
  %29 = sub i64 %internal_kv, 1
  store i64 %29, ptr %self1, align 8
  br label %bb6

bb6:                                              ; preds = %bb10, %bb5
  %30 = load i64, ptr %self1, align 8
  %31 = icmp eq i64 %30, 0
  br i1 %31, label %bb7, label %bb8

bb7:                                              ; preds = %bb6
  %_24.0 = load ptr, ptr %node, align 8
  %32 = getelementptr inbounds i8, ptr %_22, i64 8
  store ptr %_24.0, ptr %32, align 8
  %33 = getelementptr inbounds i8, ptr %32, i64 8
  store i64 0, ptr %33, align 8
  store i64 0, ptr %_22, align 8
  %34 = getelementptr inbounds i8, ptr %_22, i64 8
  %leaf.0 = load ptr, ptr %34, align 8
  %35 = getelementptr inbounds i8, ptr %34, i64 8
  %leaf.1 = load i64, ptr %35, align 8
  store ptr %leaf.0, ptr %_0, align 8
  %36 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %leaf.1, ptr %36, align 8
  %37 = getelementptr inbounds i8, ptr %_0, i64 16
  store i64 0, ptr %37, align 8
  br label %bb1

bb8:                                              ; preds = %bb6
  %_25.1 = load i64, ptr %self1, align 8
  %_25.0 = load ptr, ptr %node, align 8
  %38 = getelementptr inbounds i8, ptr %_22, i64 8
  store ptr %_25.0, ptr %38, align 8
  %39 = getelementptr inbounds i8, ptr %38, i64 8
  store i64 %_25.1, ptr %39, align 8
  store i64 1, ptr %_22, align 8
  %40 = getelementptr inbounds i8, ptr %_22, i64 8
  %41 = getelementptr inbounds i8, ptr %40, i64 8
  %internal = load i64, ptr %41, align 8
  %42 = getelementptr inbounds i8, ptr %_22, i64 8
  %internal10 = load ptr, ptr %42, align 8
  %_29 = getelementptr inbounds i8, ptr %internal10, i64 544
  br label %bb9

bb9:                                              ; preds = %bb8
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$13get_unchecked18precondition_check17hb223293d499b03feE"(i64 0, i64 12, ptr align 8 @alloc_22025f2193ce509b60b3c47e41728876) #19
  br label %bb10

bb10:                                             ; preds = %bb9
  %self11 = getelementptr inbounds nuw ptr, ptr %_29, i64 0
  %43 = load ptr, ptr %self11, align 8
  store ptr %43, ptr %node, align 8
  %44 = sub i64 %internal, 1
  store i64 %44, ptr %self1, align 8
  br label %bb6
}

; alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_end
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$16deallocating_end17hc0fcf6e22d69ff68E"(ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %edge1 = alloca [8 x i8], align 8
  %edge = alloca [16 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %alloc = alloca [0 x i8], align 1
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %self2 = load i64, ptr %1, align 8
  %self3 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 16
  %3 = load i64, ptr %2, align 8
  store i64 %3, ptr %edge1, align 8
  %4 = getelementptr inbounds i8, ptr %edge, i64 8
  store i64 %self2, ptr %4, align 8
  store ptr %self3, ptr %edge, align 8
  br label %bb1

bb1:                                              ; preds = %bb4, %start
; invoke <alloc::alloc::Global as core::clone::Clone>::clone
  invoke void @"_ZN59_$LT$alloc..alloc..Global$u20$as$u20$core..clone..Clone$GT$5clone17ha9b8da7de1d857b7E"(ptr align 1 %alloc)
          to label %bb2 unwind label %cleanup

bb7:                                              ; preds = %cleanup
  %5 = load ptr, ptr %0, align 8
  %6 = getelementptr inbounds i8, ptr %0, i64 8
  %7 = load i32, ptr %6, align 8
  %8 = insertvalue { ptr, i32 } poison, ptr %5, 0
  %9 = insertvalue { ptr, i32 } %8, i32 %7, 1
  resume { ptr, i32 } %9

cleanup:                                          ; preds = %bb2, %bb1
  %10 = landingpad { ptr, i32 }
          cleanup
  %11 = extractvalue { ptr, i32 } %10, 0
  %12 = extractvalue { ptr, i32 } %10, 1
  store ptr %11, ptr %0, align 8
  %13 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %12, ptr %13, align 8
  br label %bb7

bb2:                                              ; preds = %bb1
  %14 = load ptr, ptr %edge, align 8
  %15 = getelementptr inbounds i8, ptr %edge, i64 8
  %16 = load i64, ptr %15, align 8
; invoke alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::LeafOrInternal>::deallocate_and_ascend
  invoke void @"_ZN5alloc11collections5btree4node127NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..LeafOrInternal$GT$21deallocate_and_ascend17h54953052088af786E"(ptr sret([24 x i8]) align 8 %_3, ptr %14, i64 %16)
          to label %bb3 unwind label %cleanup

bb3:                                              ; preds = %bb2
  %17 = load ptr, ptr %_3, align 8
  %18 = ptrtoint ptr %17 to i64
  %19 = icmp eq i64 %18, 0
  %_6 = select i1 %19, i64 0, i64 1
  %20 = trunc nuw i64 %_6 to i1
  br i1 %20, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  %21 = getelementptr inbounds i8, ptr %_3, i64 8
  %parent_edge = load i64, ptr %21, align 8
  %parent_edge4 = load ptr, ptr %_3, align 8
  %22 = getelementptr inbounds i8, ptr %_3, i64 16
  %23 = load i64, ptr %22, align 8
  store i64 %23, ptr %edge1, align 8
  %24 = getelementptr inbounds i8, ptr %edge, i64 8
  store i64 %parent_edge, ptr %24, align 8
  store ptr %parent_edge4, ptr %edge, align 8
  br label %bb1

bb5:                                              ; preds = %bb3
  ret void

bb9:                                              ; No predecessors!
  unreachable
}

; alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_next
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$17deallocating_next17h244ea673c7608ad3E"(ptr sret([48 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_20 = alloca [24 x i8], align 8
  %_11 = alloca [24 x i8], align 8
  %_10 = alloca [24 x i8], align 8
  %_8 = alloca [24 x i8], align 8
  %_7 = alloca [24 x i8], align 8
  %_6 = alloca [48 x i8], align 8
  %kv = alloca [24 x i8], align 8
  %_4 = alloca [32 x i8], align 8
  %edge = alloca [24 x i8], align 8
  %alloc = alloca [0 x i8], align 1
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %self1 = load i64, ptr %1, align 8
  %self2 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 16
  %self3 = load i64, ptr %2, align 8
  store ptr %self2, ptr %edge, align 8
  %3 = getelementptr inbounds i8, ptr %edge, i64 8
  store i64 %self1, ptr %3, align 8
  %4 = getelementptr inbounds i8, ptr %edge, i64 16
  store i64 %self3, ptr %4, align 8
  br label %bb1

bb1:                                              ; preds = %bb7, %start
  %5 = getelementptr inbounds i8, ptr %edge, i64 16
  %idx = load i64, ptr %5, align 8
  %_24 = load ptr, ptr %edge, align 8
  %6 = getelementptr inbounds i8, ptr %_24, i64 538
  %_22 = load i16, ptr %6, align 2
  %_18 = zext i16 %_22 to i64
  %_16 = icmp ult i64 %idx, %_18
  br i1 %_16, label %bb12, label %bb13

bb13:                                             ; preds = %bb1
  %7 = getelementptr inbounds i8, ptr %_4, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7, ptr align 8 %edge, i64 24, i1 false)
  store i64 1, ptr %_4, align 8
  %8 = getelementptr inbounds i8, ptr %_4, i64 8
  %last_edge.0 = load ptr, ptr %8, align 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  %last_edge.1 = load i64, ptr %9, align 8
  %10 = getelementptr inbounds i8, ptr %_4, i64 8
  %11 = getelementptr inbounds i8, ptr %10, i64 16
  %last_edge = load i64, ptr %11, align 8
; invoke <alloc::alloc::Global as core::clone::Clone>::clone
  invoke void @"_ZN59_$LT$alloc..alloc..Global$u20$as$u20$core..clone..Clone$GT$5clone17ha9b8da7de1d857b7E"(ptr align 1 %alloc)
          to label %bb4 unwind label %cleanup

bb12:                                             ; preds = %bb1
  %node.0 = load ptr, ptr %edge, align 8
  %12 = getelementptr inbounds i8, ptr %edge, i64 8
  %node.1 = load i64, ptr %12, align 8
  store ptr %node.0, ptr %_20, align 8
  %13 = getelementptr inbounds i8, ptr %_20, i64 8
  store i64 %node.1, ptr %13, align 8
  %14 = getelementptr inbounds i8, ptr %_20, i64 16
  store i64 %idx, ptr %14, align 8
  %15 = getelementptr inbounds i8, ptr %_4, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %15, ptr align 8 %_20, i64 24, i1 false)
  store i64 0, ptr %_4, align 8
  %16 = getelementptr inbounds i8, ptr %_4, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %kv, ptr align 8 %16, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_8, ptr align 8 %kv, i64 24, i1 false)
; invoke alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge
  invoke void @"_ZN5alloc11collections5btree8navigate235_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$BorrowType$C$K$C$V$C$alloc..collections..btree..node..marker..LeafOrInternal$GT$$C$alloc..collections..btree..node..marker..KV$GT$$GT$14next_leaf_edge17h2e87acb9de082749E"(ptr sret([24 x i8]) align 8 %_7, ptr align 8 %_8)
          to label %bb3 unwind label %cleanup

bb10:                                             ; preds = %cleanup
  %17 = load ptr, ptr %0, align 8
  %18 = getelementptr inbounds i8, ptr %0, i64 8
  %19 = load i32, ptr %18, align 8
  %20 = insertvalue { ptr, i32 } poison, ptr %17, 0
  %21 = insertvalue { ptr, i32 } %20, i32 %19, 1
  resume { ptr, i32 } %21

cleanup:                                          ; preds = %bb12, %bb4, %bb13
  %22 = landingpad { ptr, i32 }
          cleanup
  %23 = extractvalue { ptr, i32 } %22, 0
  %24 = extractvalue { ptr, i32 } %22, 1
  store ptr %23, ptr %0, align 8
  %25 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %24, ptr %25, align 8
  br label %bb10

bb4:                                              ; preds = %bb13
; invoke alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::LeafOrInternal>::deallocate_and_ascend
  invoke void @"_ZN5alloc11collections5btree4node127NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..LeafOrInternal$GT$21deallocate_and_ascend17h54953052088af786E"(ptr sret([24 x i8]) align 8 %_11, ptr %last_edge.0, i64 %last_edge.1)
          to label %bb5 unwind label %cleanup

bb5:                                              ; preds = %bb4
  %26 = load ptr, ptr %_11, align 8
  %27 = ptrtoint ptr %26 to i64
  %28 = icmp eq i64 %27, 0
  %_14 = select i1 %28, i64 0, i64 1
  %29 = trunc nuw i64 %_14 to i1
  br i1 %29, label %bb7, label %bb6

bb7:                                              ; preds = %bb5
  %30 = getelementptr inbounds i8, ptr %_11, i64 8
  %parent_edge = load i64, ptr %30, align 8
  %parent_edge4 = load ptr, ptr %_11, align 8
  %31 = getelementptr inbounds i8, ptr %_11, i64 16
  %parent_edge5 = load i64, ptr %31, align 8
  store ptr %parent_edge4, ptr %edge, align 8
  %32 = getelementptr inbounds i8, ptr %edge, i64 8
  store i64 %parent_edge, ptr %32, align 8
  %33 = getelementptr inbounds i8, ptr %edge, i64 16
  store i64 %parent_edge5, ptr %33, align 8
  br label %bb1

bb6:                                              ; preds = %bb5
  store ptr null, ptr %_0, align 8
  br label %bb8

bb8:                                              ; preds = %bb3, %bb6
  ret void

bb2:                                              ; No predecessors!
  unreachable

bb3:                                              ; preds = %bb12
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_10, ptr align 8 %kv, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_6, ptr align 8 %_7, i64 24, i1 false)
  %34 = getelementptr inbounds i8, ptr %_6, i64 24
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %34, ptr align 8 %_10, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_6, i64 48, i1 false)
  br label %bb8
}

; alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_next_unchecked::{{closure}}
; Function Attrs: inlinehint uwtable
define internal void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$27deallocating_next_unchecked28_$u7b$$u7b$closure$u7d$$u7d$17h930b67439ab469abE"(ptr sret([48 x i8]) align 8 %val, ptr align 8 %leaf_edge) unnamed_addr #0 {
start:
  %self = alloca [48 x i8], align 8
; call alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_next
  call void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$17deallocating_next17h244ea673c7608ad3E"(ptr sret([48 x i8]) align 8 %self, ptr align 8 %leaf_edge)
  %0 = load ptr, ptr %self, align 8
  %1 = ptrtoint ptr %0 to i64
  %2 = icmp eq i64 %1, 0
  %_5 = select i1 %2, i64 0, i64 1
  %3 = trunc nuw i64 %_5 to i1
  br i1 %3, label %bb4, label %bb3

bb4:                                              ; preds = %start
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %val, ptr align 8 %self, i64 48, i1 false)
  ret void

bb3:                                              ; preds = %start
; call core::option::unwrap_failed
  call void @_RNvNtCsl8K0bEFm1U0_4core6option13unwrap_failed(ptr align 8 @alloc_655180d482d4b2af927263fa0d679f4a) #20
  unreachable

bb2:                                              ; No predecessors!
  unreachable
}

; alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front
; Function Attrs: uwtable
define internal align 8 ptr @"_ZN5alloc11collections5btree8navigate39LazyLeafRange$LT$BorrowType$C$K$C$V$GT$10init_front17hb5eb265bc33006f6E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %self2 = alloca [8 x i8], align 8
  %self1 = alloca [8 x i8], align 8
  %_12 = alloca [24 x i8], align 8
  %_7 = alloca [24 x i8], align 8
  %_6 = alloca [24 x i8], align 8
  %_5 = alloca [32 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  %_3 = load i64, ptr %self, align 8
  %0 = trunc nuw i64 %_3 to i1
  br i1 %0, label %bb1, label %bb3

bb1:                                              ; preds = %start
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %2 = load ptr, ptr %1, align 8
  %3 = ptrtoint ptr %2 to i64
  %4 = icmp eq i64 %3, 0
  %_2 = select i1 %4, i64 0, i64 1
  %5 = trunc nuw i64 %_2 to i1
  br i1 %5, label %bb3, label %bb2

bb3:                                              ; preds = %bb11, %bb1, %start
  %_10 = load i64, ptr %self, align 8
  %6 = trunc nuw i64 %_10 to i1
  br i1 %6, label %bb5, label %bb8

bb2:                                              ; preds = %bb1
  %7 = getelementptr inbounds i8, ptr %self, i64 8
  %src = getelementptr inbounds i8, ptr %7, i64 8
  %8 = getelementptr inbounds i8, ptr %self, i64 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  %10 = getelementptr inbounds i8, ptr %9, i64 8
  %11 = load i64, ptr %10, align 8
  store i64 %11, ptr %self1, align 8
  %12 = getelementptr inbounds i8, ptr %self, i64 8
  %13 = getelementptr inbounds i8, ptr %12, i64 8
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %self2, align 8
  br label %bb10

bb10:                                             ; preds = %bb14, %bb2
  %15 = load i64, ptr %self1, align 8
  %16 = icmp eq i64 %15, 0
  br i1 %16, label %bb11, label %bb12

bb11:                                             ; preds = %bb10
  %_14.0 = load ptr, ptr %self2, align 8
  %17 = getelementptr inbounds i8, ptr %_12, i64 8
  store ptr %_14.0, ptr %17, align 8
  %18 = getelementptr inbounds i8, ptr %17, i64 8
  store i64 0, ptr %18, align 8
  store i64 0, ptr %_12, align 8
  %19 = getelementptr inbounds i8, ptr %_12, i64 8
  %leaf.0 = load ptr, ptr %19, align 8
  %20 = getelementptr inbounds i8, ptr %19, i64 8
  %leaf.1 = load i64, ptr %20, align 8
  store ptr %leaf.0, ptr %_7, align 8
  %21 = getelementptr inbounds i8, ptr %_7, i64 8
  store i64 %leaf.1, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %_7, i64 16
  store i64 0, ptr %22, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_6, ptr align 8 %_7, i64 24, i1 false)
  %23 = getelementptr inbounds i8, ptr %_5, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %23, ptr align 8 %_6, i64 24, i1 false)
  store i64 1, ptr %_5, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %self, ptr align 8 %_5, i64 32, i1 false)
  br label %bb3

bb12:                                             ; preds = %bb10
  %_15.1 = load i64, ptr %self1, align 8
  %_15.0 = load ptr, ptr %self2, align 8
  %24 = getelementptr inbounds i8, ptr %_12, i64 8
  store ptr %_15.0, ptr %24, align 8
  %25 = getelementptr inbounds i8, ptr %24, i64 8
  store i64 %_15.1, ptr %25, align 8
  store i64 1, ptr %_12, align 8
  %26 = getelementptr inbounds i8, ptr %_12, i64 8
  %27 = getelementptr inbounds i8, ptr %26, i64 8
  %internal = load i64, ptr %27, align 8
  %28 = getelementptr inbounds i8, ptr %_12, i64 8
  %internal3 = load ptr, ptr %28, align 8
  %_19 = getelementptr inbounds i8, ptr %internal3, i64 544
  br label %bb13

bb13:                                             ; preds = %bb12
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$13get_unchecked18precondition_check17hb223293d499b03feE"(i64 0, i64 12, ptr align 8 @alloc_22025f2193ce509b60b3c47e41728876) #19
  br label %bb14

bb14:                                             ; preds = %bb13
  %self4 = getelementptr inbounds nuw ptr, ptr %_19, i64 0
  %29 = load ptr, ptr %self4, align 8
  store ptr %29, ptr %self2, align 8
  %30 = sub i64 %internal, 1
  store i64 %30, ptr %self1, align 8
  br label %bb10

bb5:                                              ; preds = %bb3
  %31 = getelementptr inbounds i8, ptr %self, i64 8
  %32 = load ptr, ptr %31, align 8
  %33 = ptrtoint ptr %32 to i64
  %34 = icmp eq i64 %33, 0
  %_9 = select i1 %34, i64 0, i64 1
  %35 = trunc nuw i64 %_9 to i1
  br i1 %35, label %bb7, label %bb6

bb8:                                              ; preds = %bb3
  store ptr null, ptr %_0, align 8
  br label %bb9

bb9:                                              ; preds = %bb7, %bb8
  %36 = load ptr, ptr %_0, align 8
  ret ptr %36

bb7:                                              ; preds = %bb5
  %edge = getelementptr inbounds i8, ptr %self, i64 8
  store ptr %edge, ptr %_0, align 8
  br label %bb9

bb6:                                              ; preds = %bb5
; call core::hint::unreachable_unchecked::precondition_check
  call void @_ZN4core4hint21unreachable_unchecked18precondition_check17h3199eaece5aa5bf6E(ptr align 8 @alloc_185024b4a9041c4ec014ffc119918981) #19
  br label %bb4

bb4:                                              ; preds = %bb6
  unreachable
}

; alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::take_front
; Function Attrs: uwtable
define internal void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$10take_front17hda8ea1ffd885e6c0E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #1 {
start:
  %root2 = alloca [8 x i8], align 8
  %root = alloca [8 x i8], align 8
  %_15 = alloca [24 x i8], align 8
  %v = alloca [24 x i8], align 8
  %_10 = alloca [32 x i8], align 8
  %edge = alloca [24 x i8], align 8
  %_8 = alloca [24 x i8], align 8
  %val = alloca [24 x i8], align 8
  %self1 = alloca [32 x i8], align 8
  %_2 = alloca [32 x i8], align 8
  store i64 0, ptr %_10, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %self1, ptr align 8 %self, i64 32, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %self, ptr align 8 %_10, i64 32, i1 false)
  %_11 = load i64, ptr %self1, align 8
  %0 = trunc nuw i64 %_11 to i1
  br i1 %0, label %bb7, label %bb6

bb7:                                              ; preds = %start
  %1 = getelementptr inbounds i8, ptr %self1, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v, ptr align 8 %1, i64 24, i1 false)
  %2 = getelementptr inbounds i8, ptr %_2, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %2, ptr align 8 %v, i64 24, i1 false)
  store i64 0, ptr %_2, align 8
  %3 = getelementptr inbounds i8, ptr %_2, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %val, ptr align 8 %3, i64 24, i1 false)
  %4 = load ptr, ptr %val, align 8
  %5 = ptrtoint ptr %4 to i64
  %6 = icmp eq i64 %5, 0
  %_7 = select i1 %6, i64 0, i64 1
  %7 = trunc nuw i64 %_7 to i1
  br i1 %7, label %bb2, label %bb3

bb6:                                              ; preds = %start
  store ptr null, ptr %_0, align 8
  br label %bb5

bb5:                                              ; preds = %bb4, %bb6
  ret void

bb2:                                              ; preds = %bb7
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %edge, ptr align 8 %val, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %edge, i64 24, i1 false)
  br label %bb4

bb3:                                              ; preds = %bb7
  %8 = getelementptr inbounds i8, ptr %val, i64 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  %10 = load i64, ptr %9, align 8
  store i64 %10, ptr %root, align 8
  %11 = getelementptr inbounds i8, ptr %val, i64 8
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %root2, align 8
  br label %bb8

bb8:                                              ; preds = %bb12, %bb3
  %13 = load i64, ptr %root, align 8
  %14 = icmp eq i64 %13, 0
  br i1 %14, label %bb9, label %bb10

bb9:                                              ; preds = %bb8
  %_17.0 = load ptr, ptr %root2, align 8
  %15 = getelementptr inbounds i8, ptr %_15, i64 8
  store ptr %_17.0, ptr %15, align 8
  %16 = getelementptr inbounds i8, ptr %15, i64 8
  store i64 0, ptr %16, align 8
  store i64 0, ptr %_15, align 8
  %17 = getelementptr inbounds i8, ptr %_15, i64 8
  %leaf.0 = load ptr, ptr %17, align 8
  %18 = getelementptr inbounds i8, ptr %17, i64 8
  %leaf.1 = load i64, ptr %18, align 8
  store ptr %leaf.0, ptr %_8, align 8
  %19 = getelementptr inbounds i8, ptr %_8, i64 8
  store i64 %leaf.1, ptr %19, align 8
  %20 = getelementptr inbounds i8, ptr %_8, i64 16
  store i64 0, ptr %20, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_8, i64 24, i1 false)
  br label %bb4

bb10:                                             ; preds = %bb8
  %_18.1 = load i64, ptr %root, align 8
  %_18.0 = load ptr, ptr %root2, align 8
  %21 = getelementptr inbounds i8, ptr %_15, i64 8
  store ptr %_18.0, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 8
  store i64 %_18.1, ptr %22, align 8
  store i64 1, ptr %_15, align 8
  %23 = getelementptr inbounds i8, ptr %_15, i64 8
  %24 = getelementptr inbounds i8, ptr %23, i64 8
  %internal = load i64, ptr %24, align 8
  %25 = getelementptr inbounds i8, ptr %_15, i64 8
  %internal3 = load ptr, ptr %25, align 8
  %_22 = getelementptr inbounds i8, ptr %internal3, i64 544
  br label %bb11

bb4:                                              ; preds = %bb2, %bb9
  br label %bb5

bb11:                                             ; preds = %bb10
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$13get_unchecked18precondition_check17hb223293d499b03feE"(i64 0, i64 12, ptr align 8 @alloc_22025f2193ce509b60b3c47e41728876) #19
  br label %bb12

bb12:                                             ; preds = %bb11
  %self4 = getelementptr inbounds nuw ptr, ptr %_22, i64 0
  %26 = load ptr, ptr %self4, align 8
  store ptr %26, ptr %root2, align 8
  %27 = sub i64 %internal, 1
  store i64 %27, ptr %root, align 8
  br label %bb8

bb1:                                              ; No predecessors!
  unreachable
}

; alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::deallocating_end
; Function Attrs: inlinehint uwtable
define internal void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$16deallocating_end17hfad789336ed5e8d8E"(ptr align 8 %self) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_7 = alloca [1 x i8], align 1
  %front = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  store i8 1, ptr %_7, align 1
; invoke alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::take_front
  invoke void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$10take_front17hda8ea1ffd885e6c0E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %self)
          to label %bb1 unwind label %cleanup

bb8:                                              ; preds = %cleanup
  %1 = load i8, ptr %_7, align 1
  %2 = trunc nuw i8 %1 to i1
  br i1 %2, label %bb7, label %bb6

cleanup:                                          ; preds = %bb2, %start
  %3 = landingpad { ptr, i32 }
          cleanup
  %4 = extractvalue { ptr, i32 } %3, 0
  %5 = extractvalue { ptr, i32 } %3, 1
  store ptr %4, ptr %0, align 8
  %6 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %5, ptr %6, align 8
  br label %bb8

bb1:                                              ; preds = %start
  %7 = load ptr, ptr %_3, align 8
  %8 = ptrtoint ptr %7 to i64
  %9 = icmp eq i64 %8, 0
  %_4 = select i1 %9, i64 0, i64 1
  %10 = trunc nuw i64 %_4 to i1
  br i1 %10, label %bb2, label %bb4

bb2:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %front, ptr align 8 %_3, i64 24, i1 false)
  store i8 0, ptr %_7, align 1
; invoke alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_end
  invoke void @"_ZN5alloc11collections5btree8navigate263_$LT$impl$u20$alloc..collections..btree..node..Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$alloc..collections..btree..node..marker..Leaf$GT$$C$alloc..collections..btree..node..marker..Edge$GT$$GT$16deallocating_end17hc0fcf6e22d69ff68E"(ptr align 8 %front)
          to label %bb3 unwind label %cleanup

bb4:                                              ; preds = %bb1
  br label %bb5

bb3:                                              ; preds = %bb2
  br label %bb5

bb5:                                              ; preds = %bb4, %bb3
  ret void

bb9:                                              ; No predecessors!
  unreachable

bb6:                                              ; preds = %bb7, %bb8
  %11 = load ptr, ptr %0, align 8
  %12 = getelementptr inbounds i8, ptr %0, i64 8
  %13 = load i32, ptr %12, align 8
  %14 = insertvalue { ptr, i32 } poison, ptr %11, 0
  %15 = insertvalue { ptr, i32 } %14, i32 %13, 1
  resume { ptr, i32 } %15

bb7:                                              ; preds = %bb8
  br label %bb6
}

; alloc::collections::btree::navigate::LazyLeafRange<alloc::collections::btree::node::marker::Dying,K,V>::deallocating_next_unchecked
; Function Attrs: inlinehint uwtable
define internal void @"_ZN5alloc11collections5btree8navigate75LazyLeafRange$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$GT$27deallocating_next_unchecked17h396b99105fd34df8E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_7 = alloca [1 x i8], align 1
  %self1 = alloca [8 x i8], align 8
  store i8 1, ptr %_7, align 1
; invoke alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front
  %1 = invoke align 8 ptr @"_ZN5alloc11collections5btree8navigate39LazyLeafRange$LT$BorrowType$C$K$C$V$GT$10init_front17hb5eb265bc33006f6E"(ptr align 8 %self)
          to label %bb1 unwind label %cleanup

bb4:                                              ; preds = %cleanup
  %2 = load i8, ptr %_7, align 1
  %3 = trunc nuw i8 %2 to i1
  br i1 %3, label %bb3, label %bb2

cleanup:                                          ; preds = %bb7, %bb6, %start
  %4 = landingpad { ptr, i32 }
          cleanup
  %5 = extractvalue { ptr, i32 } %4, 0
  %6 = extractvalue { ptr, i32 } %4, 1
  store ptr %5, ptr %0, align 8
  %7 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %6, ptr %7, align 8
  br label %bb4

bb1:                                              ; preds = %start
  store ptr %1, ptr %self1, align 8
  %8 = load ptr, ptr %self1, align 8
  %9 = ptrtoint ptr %8 to i64
  %10 = icmp eq i64 %9, 0
  %_8 = select i1 %10, i64 0, i64 1
  %11 = trunc nuw i64 %_8 to i1
  br i1 %11, label %bb7, label %bb6

bb7:                                              ; preds = %bb1
  %front = load ptr, ptr %self1, align 8
  store i8 0, ptr %_7, align 1
; invoke alloc::collections::btree::mem::replace
  invoke void @_ZN5alloc11collections5btree3mem7replace17h7b5cabfd90b3c4f0E(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %front)
          to label %bb8 unwind label %cleanup

bb6:                                              ; preds = %bb1
; invoke core::option::unwrap_failed
  invoke void @_RNvNtCsl8K0bEFm1U0_4core6option13unwrap_failed(ptr align 8 @alloc_a5c6e51765d6e94a394410a5a9df1cec) #24
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb6
  unreachable

bb8:                                              ; preds = %bb7
  ret void

bb5:                                              ; No predecessors!
  unreachable

bb2:                                              ; preds = %bb3, %bb4
  %12 = load ptr, ptr %0, align 8
  %13 = getelementptr inbounds i8, ptr %0, i64 8
  %14 = load i32, ptr %13, align 8
  %15 = insertvalue { ptr, i32 } poison, ptr %12, 0
  %16 = insertvalue { ptr, i32 } %15, i32 %14, 1
  resume { ptr, i32 } %16

bb3:                                              ; preds = %bb4
  br label %bb2
}

; alloc::vec::into_iter::IntoIter<T,A>::as_raw_mut_slice
; Function Attrs: uwtable
define internal { ptr, i64 } @"_ZN5alloc3vec9into_iter21IntoIter$LT$T$C$A$GT$16as_raw_mut_slice17h764ee79bd4b53c64E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %_11 = alloca [16 x i8], align 8
  %_8 = alloca [24 x i8], align 8
  %upper = alloca [16 x i8], align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %self1 = load ptr, ptr %0, align 8
; call <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
  call void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17he3d89b131cd49a0bE"(ptr sret([24 x i8]) align 8 %_8, ptr align 8 %self) #21
  %lower = load i64, ptr %_8, align 8
  %1 = getelementptr inbounds i8, ptr %_8, i64 8
  %2 = load i64, ptr %1, align 8
  %3 = getelementptr inbounds i8, ptr %1, i64 8
  %4 = load i64, ptr %3, align 8
  store i64 %2, ptr %upper, align 8
  %5 = getelementptr inbounds i8, ptr %upper, i64 8
  store i64 %4, ptr %5, align 8
  %6 = getelementptr inbounds i8, ptr %_11, i64 8
  store i64 %lower, ptr %6, align 8
  store i64 1, ptr %_11, align 8
  %_14 = load i64, ptr %upper, align 8
  %7 = getelementptr inbounds i8, ptr %upper, i64 8
  %8 = load i64, ptr %7, align 8
  %9 = trunc nuw i64 %_14 to i1
  br i1 %9, label %bb6, label %bb5

bb6:                                              ; preds = %start
  %10 = getelementptr inbounds i8, ptr %upper, i64 8
  %_17 = load i64, ptr %10, align 8
  %_12 = icmp eq i64 %_17, %lower
  br i1 %_12, label %bb2, label %bb3

bb5:                                              ; preds = %start
  br label %bb3

bb3:                                              ; preds = %bb6, %bb5
  %11 = load ptr, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, align 8
  %12 = load ptr, ptr getelementptr inbounds (i8, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, i64 8), align 8
; call core::panicking::assert_failed::<core::option::Option<usize>, core::option::Option<usize>>
  call void @_RINvNtCsl8K0bEFm1U0_4core9panicking13assert_failedINtNtB4_6option6OptionjEBM_EB4_(i8 0, ptr align 8 %upper, ptr align 8 %_11, ptr %11, ptr %12, ptr align 8 @alloc_bc6f83648b52e3ef006f2e1708c639c7) #20
  unreachable

bb2:                                              ; preds = %bb6
  %13 = insertvalue { ptr, i64 } poison, ptr %self1, 0
  %14 = insertvalue { ptr, i64 } %13, i64 %lower, 1
  ret { ptr, i64 } %14

bb4:                                              ; No predecessors!
  unreachable
}

; <alloc::alloc::Global as core::alloc::Allocator>::deallocate
; Function Attrs: inlinehint uwtable
define internal void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %self, ptr %ptr, i64 %layout.0, i64 %layout.1) unnamed_addr #0 {
start:
  %0 = icmp eq i64 %layout.1, 0
  br i1 %0, label %bb2, label %bb1

bb2:                                              ; preds = %bb1, %start
  ret void

bb1:                                              ; preds = %start
; call __rustc::__rust_dealloc
  call void @_RNvCsfLfy6EI15iL_7___rustc14___rust_dealloc(ptr %ptr, i64 %layout.1, i64 %layout.0) #26
  br label %bb2
}

; <alloc::string::String as core::default::Default>::default
; Function Attrs: inlinehint uwtable
define internal void @"_ZN64_$LT$alloc..string..String$u20$as$u20$core..default..Default$GT$7default17h83fb7bf9f8b41871E"(ptr sret([24 x i8]) align 8 %_0) unnamed_addr #0 {
start:
  %_1 = alloca [24 x i8], align 8
  store i64 0, ptr %_1, align 8
  %0 = getelementptr inbounds i8, ptr %_1, i64 8
  store ptr inttoptr (i64 1 to ptr), ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %_1, i64 16
  store i64 0, ptr %1, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_1, i64 24, i1 false)
  ret void
}

; <alloc::ffi::c_str::CString as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN68_$LT$alloc..ffi..c_str..CString$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4e26a65b5556f848E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %_7.0 = load ptr, ptr %self, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_7.1 = load i64, ptr %0, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
  call void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hc09057a7a020f57dE"(i64 0, i64 %_7.1, ptr align 8 @alloc_64af571682933ba57a453028bb35ab34) #19
  br label %bb3

bb3:                                              ; preds = %bb1
  %_3 = getelementptr inbounds nuw i8, ptr %_7.0, i64 0
  store i8 0, ptr %_3, align 1
  ret void
}

; <std::os::fd::owned::OwnedFd as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN69_$LT$std..os..fd..owned..OwnedFd$u20$as$u20$core..ops..drop..Drop$GT$4drop17h85c614e29f0a5643E"(ptr align 4 %self) unnamed_addr #0 {
start:
  %self1 = load i32, ptr %self, align 4
; call std::sys::fs::unix::debug_assert_fd_is_open
  call void @_ZN3std3sys2fs4unix23debug_assert_fd_is_open17hee43a986c86ff907E(i32 %self1) #21
  %self2 = load i32, ptr %self, align 4
  %_5 = call i32 @close(i32 %self2) #26
  ret void
}

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3f578f1ebdb22535E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_5 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
; call core::ptr::drop_in_place<[alloc::boxed::Box<dyn core::ops::function::FnMut<()>+Output = core::result::Result<(),std::io::error::Error>+core::marker::Sync+core::marker::Send>]>
  call void @"_ZN4core3ptr238drop_in_place$LT$$u5b$alloc..boxed..Box$LT$dyn$u20$core..ops..function..FnMut$LT$$LP$$RP$$GT$$u2b$Output$u20$$u3d$$u20$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$u5d$$GT$17h8676cb3bc051b2b1E"(ptr align 8 %_5, i64 %len)
  ret void
}

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h920154bf79b14a07E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_5 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  ret void
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h588c89ca19c4fbd6E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %ptr.0 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  %ptr.1 = load ptr, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %ptr.1, i64 8
  %4 = load i64, ptr %3, align 8, !invariant.load !4
  %5 = getelementptr inbounds i8, ptr %ptr.1, i64 16
  %6 = load i64, ptr %5, align 8, !invariant.load !4
  store i64 %4, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  %7 = getelementptr inbounds i8, ptr %ptr.1, i64 8
  %8 = load i64, ptr %7, align 8, !invariant.load !4
  %9 = getelementptr inbounds i8, ptr %ptr.1, i64 16
  %10 = load i64, ptr %9, align 8, !invariant.load !4
  store i64 %10, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::ptr::alignment::Alignment::new_unchecked::precondition_check
  call void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 @alloc_1bec1b27080c9ef8ecee934b35cfe627) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  br label %bb8

bb8:                                              ; preds = %bb7
; call core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %align, ptr align 8 @alloc_371e9657d30ef9d7d62fff435570ff5b) #19
  br label %bb9

bb9:                                              ; preds = %bb8
  %11 = icmp eq i64 %size, 0
  br i1 %11, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb9
  ret void

bb1:                                              ; preds = %bb9
  %_7 = getelementptr inbounds i8, ptr %self, i64 16
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %_7, ptr %ptr.0, i64 %align, i64 %size) #21
  br label %bb3
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h7b8f9a55c191995fE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %ptr.0 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  %ptr.1 = load i64, ptr %2, align 8
  %3 = mul nuw nsw i64 %ptr.1, 4
  store i64 %3, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  %4 = mul nuw nsw i64 %ptr.1, 4
  store i64 4, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::ptr::alignment::Alignment::new_unchecked::precondition_check
  call void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 @alloc_1bec1b27080c9ef8ecee934b35cfe627) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  br label %bb8

bb8:                                              ; preds = %bb7
; call core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %align, ptr align 8 @alloc_371e9657d30ef9d7d62fff435570ff5b) #19
  br label %bb9

bb9:                                              ; preds = %bb8
  %5 = icmp eq i64 %size, 0
  br i1 %5, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb9
  ret void

bb1:                                              ; preds = %bb9
  %_7 = getelementptr inbounds i8, ptr %self, i64 16
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %_7, ptr %ptr.0, i64 %align, i64 %size) #21
  br label %bb3
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h969baa51dfa67748E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %ptr.0 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  %ptr.1 = load ptr, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %ptr.1, i64 8
  %4 = load i64, ptr %3, align 8, !invariant.load !4
  %5 = getelementptr inbounds i8, ptr %ptr.1, i64 16
  %6 = load i64, ptr %5, align 8, !invariant.load !4
  store i64 %4, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  %7 = getelementptr inbounds i8, ptr %ptr.1, i64 8
  %8 = load i64, ptr %7, align 8, !invariant.load !4
  %9 = getelementptr inbounds i8, ptr %ptr.1, i64 16
  %10 = load i64, ptr %9, align 8, !invariant.load !4
  store i64 %10, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::ptr::alignment::Alignment::new_unchecked::precondition_check
  call void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 @alloc_1bec1b27080c9ef8ecee934b35cfe627) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  br label %bb8

bb8:                                              ; preds = %bb7
; call core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %align, ptr align 8 @alloc_371e9657d30ef9d7d62fff435570ff5b) #19
  br label %bb9

bb9:                                              ; preds = %bb8
  %11 = icmp eq i64 %size, 0
  br i1 %11, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb9
  ret void

bb1:                                              ; preds = %bb9
  %_7 = getelementptr inbounds i8, ptr %self, i64 16
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %_7, ptr %ptr.0, i64 %align, i64 %size) #21
  br label %bb3
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h98b8d573cc986ceeE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %ptr.0 = load ptr, ptr %self, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  %ptr.1 = load i64, ptr %2, align 8
  %3 = mul nuw nsw i64 %ptr.1, 1
  store i64 %3, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  %4 = mul nuw nsw i64 %ptr.1, 1
  store i64 1, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::ptr::alignment::Alignment::new_unchecked::precondition_check
  call void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 @alloc_1bec1b27080c9ef8ecee934b35cfe627) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  br label %bb8

bb8:                                              ; preds = %bb7
; call core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %align, ptr align 8 @alloc_371e9657d30ef9d7d62fff435570ff5b) #19
  br label %bb9

bb9:                                              ; preds = %bb8
  %5 = icmp eq i64 %size, 0
  br i1 %5, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb9
  ret void

bb1:                                              ; preds = %bb9
  %_7 = getelementptr inbounds i8, ptr %self, i64 16
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %_7, ptr %ptr.0, i64 %align, i64 %size) #21
  br label %bb3
}

; <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN72_$LT$alloc..boxed..Box$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd510c2283e6ae505E"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %ptr = load ptr, ptr %self, align 8
  store i64 24, ptr %1, align 8
  %size = load i64, ptr %1, align 8
  store i64 8, ptr %0, align 8
  %align = load i64, ptr %0, align 8
  br label %bb6

bb6:                                              ; preds = %start
; call core::ptr::alignment::Alignment::new_unchecked::precondition_check
  call void @_ZN4core3ptr9alignment9Alignment13new_unchecked18precondition_check17h7b152de8e849d20dE(i64 %align, ptr align 8 @alloc_1bec1b27080c9ef8ecee934b35cfe627) #19
  br label %bb7

bb7:                                              ; preds = %bb6
  br label %bb8

bb8:                                              ; preds = %bb7
; call core::alloc::layout::Layout::from_size_alignment_unchecked::precondition_check
  call void @_ZN4core5alloc6layout6Layout29from_size_alignment_unchecked18precondition_check17hf03979cedaeda855E(i64 %size, i64 %align, ptr align 8 @alloc_371e9657d30ef9d7d62fff435570ff5b) #19
  br label %bb9

bb9:                                              ; preds = %bb8
  %2 = icmp eq i64 %size, 0
  br i1 %2, label %bb3, label %bb1

bb3:                                              ; preds = %bb1, %bb9
  ret void

bb1:                                              ; preds = %bb9
  %_7 = getelementptr inbounds i8, ptr %self, i64 8
; call <alloc::alloc::Global as core::alloc::Allocator>::deallocate
  call void @"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h31902a1cbe293d1bE"(ptr align 1 %_7, ptr %ptr, i64 %align, i64 %size) #21
  br label %bb3
}

; <&mut W as core::fmt::Write::write_fmt::SpecWriteFmt>::spec_write_fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN75_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write..write_fmt..SpecWriteFmt$GT$14spec_write_fmt17he1ee08f932a6c24dE"(ptr align 8 %self, ptr %0, ptr %1) unnamed_addr #0 {
start:
  %_3 = alloca [16 x i8], align 8
  %_0 = alloca [1 x i8], align 1
  %args = alloca [16 x i8], align 8
  store ptr %0, ptr %args, align 8
  %2 = getelementptr inbounds i8, ptr %args, i64 8
  store ptr %1, ptr %2, align 8
; call core::fmt::Arguments::as_statically_known_str
  %3 = call { ptr, i64 } @_ZN4core3fmt9Arguments23as_statically_known_str17h16f3697c0ce049d6E(ptr align 8 %args) #21
  %4 = extractvalue { ptr, i64 } %3, 0
  %5 = extractvalue { ptr, i64 } %3, 1
  store ptr %4, ptr %_3, align 8
  %6 = getelementptr inbounds i8, ptr %_3, i64 8
  store i64 %5, ptr %6, align 8
  %7 = load ptr, ptr %_3, align 8
  %8 = getelementptr inbounds i8, ptr %_3, i64 8
  %9 = load i64, ptr %8, align 8
  %10 = ptrtoint ptr %7 to i64
  %11 = icmp eq i64 %10, 0
  %_5 = select i1 %11, i64 0, i64 1
  %12 = trunc nuw i64 %_5 to i1
  br i1 %12, label %bb2, label %bb4

bb2:                                              ; preds = %start
  %s.0 = load ptr, ptr %_3, align 8
  %13 = getelementptr inbounds i8, ptr %_3, i64 8
  %s.1 = load i64, ptr %13, align 8
; call <std::io::default_write_fmt::Adapter<T> as core::fmt::Write>::write_str
  %14 = call zeroext i1 @"_ZN81_$LT$std..io..default_write_fmt..Adapter$LT$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h6ed374c6060c81d0E"(ptr align 8 %self, ptr align 1 %s.0, i64 %s.1)
  %15 = zext i1 %14 to i8
  store i8 %15, ptr %_0, align 1
  br label %bb6

bb4:                                              ; preds = %start
  %16 = load ptr, ptr %args, align 8
  %17 = getelementptr inbounds i8, ptr %args, i64 8
  %18 = load ptr, ptr %17, align 8
; call core::fmt::write
  %19 = call zeroext i1 @_RNvNtCsl8K0bEFm1U0_4core3fmt5write(ptr align 1 %self, ptr align 8 @vtable.0, ptr %16, ptr %18)
  %20 = zext i1 %19 to i8
  store i8 %20, ptr %_0, align 1
  br label %bb6

bb6:                                              ; preds = %bb4, %bb2
  %21 = load i8, ptr %_0, align 1
  %22 = trunc nuw i8 %21 to i1
  ret i1 %22

bb7:                                              ; No predecessors!
  unreachable
}

; <core::option::Option<T> as core::ops::try_trait::Try>::from_output
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN75_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..Try$GT$11from_output17h534749e82034c4d7E"(i64 %output) unnamed_addr #0 {
start:
  ret i64 %output
}

; <core::option::Option<T> as core::ops::try_trait::Try>::branch
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN75_$LT$core..option..Option$LT$T$GT$$u20$as$u20$core..ops..try_trait..Try$GT$6branch17h06ef99bacb1b3dc8E"(i64 %0) unnamed_addr #0 {
start:
  %_0 = alloca [8 x i8], align 8
  %self = alloca [8 x i8], align 8
  store i64 %0, ptr %self, align 8
  %1 = load i64, ptr %self, align 8
  %2 = icmp eq i64 %1, 0
  %_2 = select i1 %2, i64 0, i64 1
  %3 = trunc nuw i64 %_2 to i1
  br i1 %3, label %bb3, label %bb2

bb3:                                              ; preds = %start
  %v = load i64, ptr %self, align 8
  store i64 %v, ptr %_0, align 8
  br label %bb4

bb2:                                              ; preds = %start
  store i64 0, ptr %_0, align 8
  br label %bb4

bb4:                                              ; preds = %bb3, %bb2
  %4 = load i64, ptr %_0, align 8
  ret i64 %4

bb1:                                              ; No predecessors!
  unreachable
}

; <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$13get_unchecked18precondition_check17hb223293d499b03feE"(i64 %this, i64 %len, ptr align 8 %0) unnamed_addr #4 {
start:
  %_3 = icmp ult i64 %this, %len
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_97d92cbf2a68a6ac45a1b13da79836e4, ptr inttoptr (i64 429 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; <usize as core::slice::index::SliceIndex<[T]>>::get_unchecked_mut::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$17get_unchecked_mut18precondition_check17hc09057a7a020f57dE"(i64 %this, i64 %len, ptr align 8 %0) unnamed_addr #4 {
start:
  %_3 = icmp ult i64 %this, %len
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind_fmt
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr @alloc_5a8fdd84b3281310cbf6b74bb6bf0065, ptr inttoptr (i64 437 to ptr), i1 zeroext false, ptr align 8 %0) #27
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h5b91aa02db46e479E"(ptr align 8 %self) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVecInner>::deallocate
  call void @_RNvMs2_NtCs1OjIl8oxbrv_5alloc7raw_vecNtB5_11RawVecInner10deallocateCscUtGwbhD4WH_5gimli(ptr align 8 %self, i64 8, i64 16)
  ret void
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h9b5fb7669e1418a6E"(ptr align 8 %self) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVecInner>::deallocate
  call void @_RNvMs2_NtCs1OjIl8oxbrv_5alloc7raw_vecNtB5_11RawVecInner10deallocateCscUtGwbhD4WH_5gimli(ptr align 8 %self, i64 8, i64 8)
  ret void
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hb052267a9a60429bE"(ptr align 8 %self) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVecInner>::deallocate
  call void @_RNvMs2_NtCs1OjIl8oxbrv_5alloc7raw_vecNtB5_11RawVecInner10deallocateCscUtGwbhD4WH_5gimli(ptr align 8 %self, i64 8, i64 24)
  ret void
}

; <std::io::error::repr_bitpacked::Repr as core::ops::drop::Drop>::drop
; Function Attrs: inlinehint uwtable
define internal void @"_ZN78_$LT$std..io..error..repr_bitpacked..Repr$u20$as$u20$core..ops..drop..Drop$GT$4drop17h2e6ef5378d1bc1ecE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %_2 = alloca [16 x i8], align 8
  %_3 = load ptr, ptr %self, align 8
; call std::io::error::repr_bitpacked::decode_repr
  call void @_ZN3std2io5error14repr_bitpacked11decode_repr17h9d4147e2b60c0c89E(ptr sret([16 x i8]) align 8 %_2, ptr %_3) #21
; call core::ptr::drop_in_place<std::io::error::ErrorData<alloc::boxed::Box<std::io::error::Custom>>>
  call void @"_ZN4core3ptr101drop_in_place$LT$std..io..error..ErrorData$LT$alloc..boxed..Box$LT$std..io..error..Custom$GT$$GT$$GT$17h19b3082c65695e7aE"(ptr align 8 %_2)
  ret void
}

; <std::io::error::repr_bitpacked::Repr as core::ops::drop::Drop>::drop::{{closure}}
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN78_$LT$std..io..error..repr_bitpacked..Repr$u20$as$u20$core..ops..drop..Drop$GT$4drop28_$u7b$$u7b$closure$u7d$$u7d$17h7f350cb5994ee852E"(ptr %p) unnamed_addr #0 {
start:
  br label %bb1

bb1:                                              ; preds = %start
; call core::ptr::non_null::NonNull<T>::new_unchecked::precondition_check
  call void @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$13new_unchecked18precondition_check17h893a6acd6dd8e9ceE"(ptr %p, ptr align 8 @alloc_e220089d5016f4a98719c6bb086d229f) #19
  br label %bb3

bb3:                                              ; preds = %bb1
  ret ptr %p
}

; <std::io::default_write_fmt::Adapter<T> as core::fmt::Write>::write_str
; Function Attrs: uwtable
define internal zeroext i1 @"_ZN81_$LT$std..io..default_write_fmt..Adapter$LT$T$GT$$u20$as$u20$core..fmt..Write$GT$9write_str17h6ed374c6060c81d0E"(ptr align 8 %self, ptr align 1 %s.0, i64 %s.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_3 = alloca [8 x i8], align 8
  %_0 = alloca [1 x i8], align 1
  %_7 = load ptr, ptr %self, align 8
; call std::io::Write::write_all
  %1 = call ptr @_ZN3std2io5Write9write_all17h8af9bb9b2151de98E(ptr align 1 %_7, ptr align 1 %s.0, i64 %s.1)
  store ptr %1, ptr %_3, align 8
  %2 = load ptr, ptr %_3, align 8
  %3 = ptrtoint ptr %2 to i64
  %4 = icmp eq i64 %3, 0
  %_5 = select i1 %4, i64 0, i64 1
  %5 = trunc nuw i64 %_5 to i1
  br i1 %5, label %bb3, label %bb4

bb3:                                              ; preds = %start
  %e = load ptr, ptr %_3, align 8
  %6 = getelementptr inbounds i8, ptr %self, i64 8
; invoke core::ptr::drop_in_place<core::result::Result<(),std::io::error::Error>>
  invoke void @"_ZN4core3ptr81drop_in_place$LT$core..result..Result$LT$$LP$$RP$$C$std..io..error..Error$GT$$GT$17ha0d2151b81f17565E"(ptr align 8 %6)
          to label %bb5 unwind label %cleanup

bb4:                                              ; preds = %start
  store i8 0, ptr %_0, align 1
  br label %bb7

bb7:                                              ; preds = %bb5, %bb4
  %7 = load i8, ptr %_0, align 1
  %8 = trunc nuw i8 %7 to i1
  ret i1 %8

bb6:                                              ; preds = %cleanup
  %9 = getelementptr inbounds i8, ptr %self, i64 8
  %10 = load ptr, ptr %_3, align 8
  store ptr %10, ptr %9, align 8
  %11 = load ptr, ptr %0, align 8
  %12 = getelementptr inbounds i8, ptr %0, i64 8
  %13 = load i32, ptr %12, align 8
  %14 = insertvalue { ptr, i32 } poison, ptr %11, 0
  %15 = insertvalue { ptr, i32 } %14, i32 %13, 1
  resume { ptr, i32 } %15

cleanup:                                          ; preds = %bb3
  %16 = landingpad { ptr, i32 }
          cleanup
  %17 = extractvalue { ptr, i32 } %16, 0
  %18 = extractvalue { ptr, i32 } %16, 1
  store ptr %17, ptr %0, align 8
  %19 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %18, ptr %19, align 8
  br label %bb6

bb5:                                              ; preds = %bb3
  %20 = getelementptr inbounds i8, ptr %self, i64 8
  %21 = load ptr, ptr %_3, align 8
  store ptr %21, ptr %20, align 8
  store i8 1, ptr %_0, align 1
  br label %bb7

bb2:                                              ; No predecessors!
  unreachable
}

; <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17he193351050f4b390E"(ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %guard = alloca [8 x i8], align 8
  store ptr %self, ptr %guard, align 8
  %_5 = load ptr, ptr %guard, align 8
; invoke alloc::vec::into_iter::IntoIter<T,A>::as_raw_mut_slice
  %1 = invoke { ptr, i64 } @"_ZN5alloc3vec9into_iter21IntoIter$LT$T$C$A$GT$16as_raw_mut_slice17h764ee79bd4b53c64E"(ptr align 8 %_5)
          to label %bb1 unwind label %cleanup

bb4:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  invoke void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17hd6b4225b5a6a79c8E"(ptr align 8 %guard) #22
          to label %bb5 unwind label %terminate

cleanup:                                          ; preds = %bb1, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb4

bb1:                                              ; preds = %start
  %_4.0 = extractvalue { ptr, i64 } %1, 0
  %_4.1 = extractvalue { ptr, i64 } %1, 1
; invoke core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
  invoke void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17h45398f46b2529b7dE"(ptr align 8 %_4.0, i64 %_4.1)
          to label %bb2 unwind label %cleanup

bb2:                                              ; preds = %bb1
; call core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  call void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17hd6b4225b5a6a79c8E"(ptr align 8 %guard)
  ret void

terminate:                                        ; preds = %bb4
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb5:                                              ; preds = %bb4
  %7 = load ptr, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  %9 = load i32, ptr %8, align 8
  %10 = insertvalue { ptr, i32 } poison, ptr %7, 0
  %11 = insertvalue { ptr, i32 } %10, i32 %9, 1
  resume { ptr, i32 } %11
}

; <I as core::iter::traits::iterator::Iterator::advance_by::SpecAdvanceBy>::spec_advance_by
; Function Attrs: uwtable
define internal i64 @"_ZN87_$LT$I$u20$as$u20$core..iter..traits..iterator..Iterator..advance_by..SpecAdvanceBy$GT$15spec_advance_by17hdcc7acec5d579568E"(ptr align 8 %self, i64 %n) unnamed_addr #1 {
start:
  %res = alloca [8 x i8], align 8
  %_4 = alloca [8 x i8], align 8
  %_0 = alloca [8 x i8], align 8
  store i64 %n, ptr %_4, align 8
  %0 = load i64, ptr %_4, align 8
  %1 = icmp eq i64 %0, 0
  %_5 = select i1 %1, i64 0, i64 1
  %2 = trunc nuw i64 %_5 to i1
  br i1 %2, label %bb1, label %bb2

bb1:                                              ; preds = %start
  %n1 = load i64, ptr %_4, align 8
; call core::iter::traits::iterator::Iterator::try_fold
  %3 = call i64 @_ZN4core4iter6traits8iterator8Iterator8try_fold17h5de16c88a03504d2E(ptr align 8 %self, i64 %n1) #21
  store i64 %3, ptr %res, align 8
  %4 = load i64, ptr %res, align 8
  %5 = icmp eq i64 %4, 0
  %_7 = select i1 %5, i64 0, i64 1
  %6 = trunc nuw i64 %_7 to i1
  br i1 %6, label %bb5, label %bb6

bb2:                                              ; preds = %start
  store i64 0, ptr %_0, align 8
  br label %bb8

bb5:                                              ; preds = %bb1
  %n2 = load i64, ptr %res, align 8
  store i64 %n2, ptr %_0, align 8
  br label %bb7

bb6:                                              ; preds = %bb1
  store i64 0, ptr %_0, align 8
  br label %bb7

bb7:                                              ; preds = %bb5, %bb6
  br label %bb8

bb8:                                              ; preds = %bb2, %bb7
  %7 = load i64, ptr %_0, align 8
  ret i64 %7

bb4:                                              ; No predecessors!
  unreachable
}

; <I as core::iter::traits::iterator::Iterator::advance_by::SpecAdvanceBy>::spec_advance_by::{{closure}}
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN87_$LT$I$u20$as$u20$core..iter..traits..iterator..Iterator..advance_by..SpecAdvanceBy$GT$15spec_advance_by28_$u7b$$u7b$closure$u7d$$u7d$17h8c521b01281198a7E"(ptr align 1 %_1, i64 %n, ptr align 8 %_3) unnamed_addr #0 {
start:
  %n1 = sub i64 %n, 1
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17he154a73b91055e9aE"(ptr align 8 %_3)
  ret i64 %n1
}

; <alloc::collections::btree::mem::replace::PanicGuard as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN93_$LT$alloc..collections..btree..mem..replace..PanicGuard$u20$as$u20$core..ops..drop..Drop$GT$4drop17h90be998b891f4008E"(ptr align 1 %self) unnamed_addr #1 {
start:
  call void @llvm.trap()
  unreachable
}

; <alloc::collections::btree::map::BTreeMap<K,V,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN99_$LT$alloc..collections..btree..map..BTreeMap$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h2b7d51c5cc3fb0dfE"(ptr align 8 %self) unnamed_addr #1 {
start:
  %_23 = alloca [32 x i8], align 8
  %_22 = alloca [24 x i8], align 8
  %_21 = alloca [32 x i8], align 8
  %_20 = alloca [24 x i8], align 8
  %_16 = alloca [64 x i8], align 8
  %full_range = alloca [64 x i8], align 8
  %_5 = alloca [16 x i8], align 8
  %me = alloca [24 x i8], align 8
  %self1 = alloca [24 x i8], align 8
  %_x = alloca [72 x i8], align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %self1, ptr align 8 %self, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_20, ptr align 8 %self1, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %me, ptr align 8 %_20, i64 24, i1 false)
  %0 = load ptr, ptr %me, align 8
  %1 = getelementptr inbounds i8, ptr %me, i64 8
  %2 = load i64, ptr %1, align 8
  store ptr %0, ptr %_5, align 8
  %3 = getelementptr inbounds i8, ptr %_5, i64 8
  store i64 %2, ptr %3, align 8
  %4 = load ptr, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, align 8
  %5 = load i64, ptr getelementptr inbounds (i8, ptr @anon.3859b584a99d2d85503c18879f5b49e0.0, i64 8), align 8
  store ptr %4, ptr %me, align 8
  %6 = getelementptr inbounds i8, ptr %me, i64 8
  store i64 %5, ptr %6, align 8
  %7 = load ptr, ptr %_5, align 8
  %8 = getelementptr inbounds i8, ptr %_5, i64 8
  %9 = load i64, ptr %8, align 8
  %10 = ptrtoint ptr %7 to i64
  %11 = icmp eq i64 %10, 0
  %_8 = select i1 %11, i64 0, i64 1
  %12 = trunc nuw i64 %_8 to i1
  br i1 %12, label %bb1, label %bb2

bb1:                                              ; preds = %start
  %13 = getelementptr inbounds i8, ptr %_5, i64 8
  %root = load i64, ptr %13, align 8
  %root2 = load ptr, ptr %_5, align 8
  %14 = getelementptr inbounds i8, ptr %_22, i64 8
  store ptr %root2, ptr %14, align 8
  %15 = getelementptr inbounds i8, ptr %14, i64 8
  store i64 %root, ptr %15, align 8
  store ptr null, ptr %_22, align 8
  %16 = getelementptr inbounds i8, ptr %_21, i64 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %16, ptr align 8 %_22, i64 24, i1 false)
  store i64 1, ptr %_21, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %full_range, ptr align 8 %_21, i64 32, i1 false)
  %17 = getelementptr inbounds i8, ptr %full_range, i64 32
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %17, ptr align 8 %_21, i64 32, i1 false)
  %18 = getelementptr inbounds i8, ptr %me, i64 16
  %_11 = load i64, ptr %18, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_x, ptr align 8 %full_range, i64 64, i1 false)
  %19 = getelementptr inbounds i8, ptr %_x, i64 64
  store i64 %_11, ptr %19, align 8
  br label %bb3

bb2:                                              ; preds = %start
  store i64 0, ptr %_23, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_16, ptr align 8 %_23, i64 32, i1 false)
  %20 = getelementptr inbounds i8, ptr %_16, i64 32
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %20, ptr align 8 %_23, i64 32, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_x, ptr align 8 %_16, i64 64, i1 false)
  %21 = getelementptr inbounds i8, ptr %_x, i64 64
  store i64 0, ptr %21, align 8
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
; call core::ptr::drop_in_place<alloc::collections::btree::map::IntoIter<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>>>
  call void @"_ZN4core3ptr152drop_in_place$LT$alloc..collections..btree..map..IntoIter$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$GT$$GT$17hb68047b497a6efbdE"(ptr align 8 %_x)
  ret void

bb4:                                              ; No predecessors!
  unreachable
}

; <alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN99_$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h1ff518a7ee2d1eefE"(ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %guard = alloca [8 x i8], align 8
  %kv = alloca [24 x i8], align 8
  %_2 = alloca [24 x i8], align 8
  br label %bb1

bb1:                                              ; preds = %bb4, %start
; call alloc::collections::btree::map::IntoIter<K,V,A>::dying_next
  call void @"_ZN5alloc11collections5btree3map25IntoIter$LT$K$C$V$C$A$GT$10dying_next17hc659d595af5fbe87E"(ptr sret([24 x i8]) align 8 %_2, ptr align 8 %self)
  %1 = load ptr, ptr %_2, align 8
  %2 = ptrtoint ptr %1 to i64
  %3 = icmp eq i64 %2, 0
  %_3 = select i1 %3, i64 0, i64 1
  %4 = trunc nuw i64 %_3 to i1
  br i1 %4, label %bb3, label %bb5

bb3:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %kv, ptr align 8 %_2, i64 24, i1 false)
  store ptr %self, ptr %guard, align 8
; invoke alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::drop_key_val
  invoke void @"_ZN5alloc11collections5btree4node173Handle$LT$alloc..collections..btree..node..NodeRef$LT$alloc..collections..btree..node..marker..Dying$C$K$C$V$C$NodeType$GT$$C$alloc..collections..btree..node..marker..KV$GT$12drop_key_val17h8689e92183180c52E"(ptr align 8 %kv)
          to label %bb4 unwind label %cleanup

bb5:                                              ; preds = %bb1
  ret void

bb7:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<<alloc::collections::btree::map::IntoIter<K,V,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,core::option::Option<std::ffi::os_str::OsString>,alloc::alloc::Global>>
  invoke void @"_ZN4core3ptr250drop_in_place$LT$$LT$alloc..collections..btree..map..IntoIter$LT$K$C$V$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$core..option..Option$LT$std..ffi..os_str..OsString$GT$$C$alloc..alloc..Global$GT$$GT$17hdc20274c8d168defE"(ptr align 8 %guard) #22
          to label %bb6 unwind label %terminate

cleanup:                                          ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          cleanup
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
  store ptr %6, ptr %0, align 8
  %8 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %7, ptr %8, align 8
  br label %bb7

bb4:                                              ; preds = %bb3
  %t = load ptr, ptr %guard, align 8
  br label %bb1

terminate:                                        ; preds = %bb7
  %9 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
; call core::panicking::panic_in_cleanup
  call void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() #23
  unreachable

bb6:                                              ; preds = %bb7
  %10 = load ptr, ptr %0, align 8
  %11 = getelementptr inbounds i8, ptr %0, i64 8
  %12 = load i32, ptr %11, align 8
  %13 = insertvalue { ptr, i32 } poison, ptr %10, 0
  %14 = insertvalue { ptr, i32 } %13, i32 %12, 1
  resume { ptr, i32 } %14

bb8:                                              ; No predecessors!
  unreachable
}

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core9panicking5panic(ptr align 1, i64, ptr align 8) unnamed_addr #6

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #7

; Function Attrs: nounwind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #3

; std::env::args
; Function Attrs: uwtable
declare void @_RNvNtCsg55jX0GwzBC_3std3env4args(ptr sret([32 x i8]) align 8) unnamed_addr #1

; <std::process::Command>::status
; Function Attrs: uwtable
declare void @_RNvMsk_NtCsg55jX0GwzBC_3std7processNtB5_7Command6status(ptr sret([16 x i8]) align 8, ptr align 8) unnamed_addr #1

; core::panicking::panic_in_cleanup
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core9panicking16panic_in_cleanup() unnamed_addr #8

; core::fmt::write
; Function Attrs: uwtable
declare zeroext i1 @_RNvNtCsl8K0bEFm1U0_4core3fmt5write(ptr align 1, ptr align 8, ptr, ptr) unnamed_addr #1

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core9panicking9panic_fmt(ptr, ptr, ptr align 8) unnamed_addr #6

; <std::sys::stdio::unix::Stderr as std::io::Write>::write
; Function Attrs: uwtable
declare { i64, ptr } @_RNvXs3_NtNtNtCsg55jX0GwzBC_3std3sys5stdio4unixNtB5_6StderrNtNtBb_2io5Write5write(ptr align 1, ptr align 1, i64) unnamed_addr #1

; core::slice::index::slice_index_fail
; Function Attrs: cold noinline noreturn uwtable
declare void @_RNvNtNtCsl8K0bEFm1U0_4core5slice5index16slice_index_fail(i64, i64, i64, ptr align 8) unnamed_addr #6

; std::rt::lang_start_internal
; Function Attrs: uwtable
declare i64 @_RNvNtCsg55jX0GwzBC_3std2rt19lang_start_internal(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #1

; Function Attrs: nounwind uwtable
declare i32 @fcntl(i32, i32, ...) unnamed_addr #3

; Function Attrs: nounwind memory(none) uwtable
declare ptr @__error() unnamed_addr #9

; std::process::abort
; Function Attrs: cold noreturn uwtable
declare void @_RNvNtCsg55jX0GwzBC_3std7process5abort() unnamed_addr #10

; <std::sys::process::unix::common::Command>::new
; Function Attrs: uwtable
declare void @_RNvMs_NtNtNtNtCsg55jX0GwzBC_3std3sys7process4unix6commonNtB4_7Command3new(ptr sret([200 x i8]) align 8, ptr align 1, i64) unnamed_addr #1

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr writeonly captures(none), i8, i64, i1 immarg) #11

; Function Attrs: convergent nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.is.constant.i1(i1) #12

; <alloc::vec::Vec<u8> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @_RNvXso_NtCs1OjIl8oxbrv_5alloc3vecINtB5_3VechENtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4dropCscUtGwbhD4WH_5gimli(ptr align 8) unnamed_addr #1

; <alloc::raw_vec::RawVec<u8> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @_RNvXs1_NtCs1OjIl8oxbrv_5alloc7raw_vecINtB5_6RawVechENtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4dropCscUtGwbhD4WH_5gimli(ptr align 8) unnamed_addr #1

; <std::sys::process::unix::common::cstring_array::CStringArray as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @_RNvXs3_NtNtNtNtNtCsg55jX0GwzBC_3std3sys7process4unix6common13cstring_arrayNtB5_12CStringArrayNtNtNtCsl8K0bEFm1U0_4core3ops4drop4Drop4drop(ptr align 8) unnamed_addr #1

; core::panicking::panic_nounwind_fmt
; Function Attrs: cold noinline noreturn nounwind uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core9panicking18panic_nounwind_fmt(ptr, ptr, i1 zeroext, ptr align 8) unnamed_addr #13

; Function Attrs: nocallback nocreateundeforpoison nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctpop.i64(i64) #14

; <usize as core::fmt::Display>::fmt
; Function Attrs: uwtable
declare zeroext i1 @_RNvXsi_NtNtNtCsl8K0bEFm1U0_4core3fmt3num3impjNtB9_7Display3fmt(ptr align 8, ptr align 8) unnamed_addr #1

; <u32 as core::fmt::UpperHex>::fmt
; Function Attrs: uwtable
declare zeroext i1 @_RNvXsw_NtNtCsl8K0bEFm1U0_4core3fmt3nummNtB7_8UpperHex3fmt(ptr align 4, ptr align 8) unnamed_addr #1

; <std::env::Args as core::iter::traits::iterator::Iterator>::next
; Function Attrs: uwtable
declare void @_RNvXsa_NtCsg55jX0GwzBC_3std3envNtB5_4ArgsNtNtNtNtCsl8K0bEFm1U0_4core4iter6traits8iterator8Iterator4next(ptr sret([24 x i8]) align 8, ptr align 8) unnamed_addr #1

; core::panicking::panic_cannot_unwind
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core9panicking19panic_cannot_unwind() unnamed_addr #8

; core::option::unwrap_failed
; Function Attrs: cold noinline noreturn uwtable
declare void @_RNvNtCsl8K0bEFm1U0_4core6option13unwrap_failed(ptr align 8) unnamed_addr #6

; core::panicking::assert_failed::<core::option::Option<usize>, core::option::Option<usize>>
; Function Attrs: cold minsize noinline noreturn optsize uwtable
declare void @_RINvNtCsl8K0bEFm1U0_4core9panicking13assert_failedINtNtB4_6option6OptionjEBM_EB4_(i8, ptr align 8, ptr align 8, ptr, ptr, ptr align 8) unnamed_addr #15

; __rustc::__rust_dealloc
; Function Attrs: nounwind allockind("free") uwtable
declare void @_RNvCsfLfy6EI15iL_7___rustc14___rust_dealloc(ptr allocptr captures(address), i64, i64) unnamed_addr #16

; Function Attrs: nounwind uwtable
declare i32 @close(i32) unnamed_addr #3

; <alloc::raw_vec::RawVecInner>::deallocate
; Function Attrs: uwtable
declare void @_RNvMs2_NtCs1OjIl8oxbrv_5alloc7raw_vecNtB5_11RawVecInner10deallocateCscUtGwbhD4WH_5gimli(ptr align 8, i64, i64) unnamed_addr #1

; Function Attrs: cold noreturn nounwind memory(inaccessiblemem: write)
declare void @llvm.trap() #17

define i32 @main(i32 %0, ptr %1) unnamed_addr #18 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start
  %3 = call i64 @_ZN3std2rt10lang_start17ha17f72ac3d52a94cE(ptr @_ZN25minimal_command_injection4main17hc4412d17c931f476E, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { inlinehint uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #1 = { uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #2 = { noinline uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #3 = { nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #4 = { inlinehint nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #5 = { inlinehint noreturn uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #6 = { cold noinline noreturn uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #7 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #8 = { cold minsize noinline noreturn nounwind optsize uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #9 = { nounwind memory(none) uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #10 = { cold noreturn uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #11 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #12 = { convergent nocallback nofree nosync nounwind willreturn memory(none) }
attributes #13 = { cold noinline noreturn nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #14 = { nocallback nocreateundeforpoison nofree nosync nounwind speculatable willreturn memory(none) }
attributes #15 = { cold minsize noinline noreturn optsize uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #16 = { nounwind allockind("free") uwtable "alloc-family"="__rust_alloc" "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #17 = { cold noreturn nounwind memory(inaccessiblemem: write) }
attributes #18 = { "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #19 = { inlinehint nounwind }
attributes #20 = { noinline noreturn }
attributes #21 = { inlinehint }
attributes #22 = { cold }
attributes #23 = { cold noreturn nounwind }
attributes #24 = { noreturn }
attributes #25 = { noinline }
attributes #26 = { nounwind }
attributes #27 = { noinline noreturn nounwind }
attributes #28 = { inlinehint noreturn }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{!"rustc version 1.95.0 (59807616e 2026-04-14)"}
!3 = !{i64 16040358529531004}
!4 = !{}
