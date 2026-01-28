; ModuleID = 'intrinsics'
source_filename = "intrinsics.c"
target triple = "x86_64-unknown-linux-gnu"

; Test LLVM intrinsic handling

; Intrinsic declarations
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare void @llvm.lifetime.start.p0(i64, ptr)
declare void @llvm.lifetime.end.p0(i64, ptr)
declare i64 @llvm.expect.i64(i64, i64)
declare void @llvm.dbg.declare(metadata, metadata, metadata)
declare void @llvm.dbg.value(metadata, metadata, metadata)
declare void @llvm.assume(i1)

; memcpy intrinsic (should map to Memcpy operation)
define void @test_memcpy(ptr %dst, ptr %src, i64 %len) {
entry:
  call void @llvm.memcpy.p0.p0.i64(ptr %dst, ptr %src, i64 %len, i1 false)
  ret void
}

; memmove intrinsic (should map to Memcpy operation)
define void @test_memmove(ptr %dst, ptr %src, i64 %len) {
entry:
  call void @llvm.memmove.p0.p0.i64(ptr %dst, ptr %src, i64 %len, i1 false)
  ret void
}

; memset intrinsic (should map to Memset operation)
define void @test_memset(ptr %dst, i64 %len) {
entry:
  call void @llvm.memset.p0.i64(ptr %dst, i8 0, i64 %len, i1 false)
  ret void
}

; lifetime intrinsics (should be skipped)
define void @test_lifetime() {
entry:
  %x = alloca i32, align 4
  call void @llvm.lifetime.start.p0(i64 4, ptr %x)
  store i32 42, ptr %x, align 4
  %val = load i32, ptr %x, align 4
  call void @llvm.lifetime.end.p0(i64 4, ptr %x)
  ret void
}

; expect intrinsic (should be pass-through/copy)
define i64 @test_expect(i64 %val) {
entry:
  %likely = call i64 @llvm.expect.i64(i64 %val, i64 1)
  ret i64 %likely
}

; assume intrinsic (should be skipped)
define void @test_assume(i32 %val) {
entry:
  %cmp = icmp ne i32 %val, 0
  call void @llvm.assume(i1 %cmp)
  ret void
}

; Combined example with multiple intrinsics
define void @combined_intrinsics(ptr %buf, i64 %size) {
entry:
  %tmp = alloca [256 x i8], align 1
  call void @llvm.lifetime.start.p0(i64 256, ptr %tmp)

  ; Clear buffer
  call void @llvm.memset.p0.i64(ptr %tmp, i8 0, i64 256, i1 false)

  ; Copy data
  call void @llvm.memcpy.p0.p0.i64(ptr %tmp, ptr %buf, i64 %size, i1 false)

  ; Copy back
  call void @llvm.memcpy.p0.p0.i64(ptr %buf, ptr %tmp, i64 %size, i1 false)

  call void @llvm.lifetime.end.p0(i64 256, ptr %tmp)
  ret void
}
