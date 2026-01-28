; ModuleID = 'svfg_store_load.c'
source_filename = "svfg_store_load.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @simple_store_load() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %value = alloca i32, align 4
  %result = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  store i32 42, ptr %value, align 4
  %0 = load i32, ptr %value, align 4
  %1 = load ptr, ptr %p, align 8
  store i32 %0, ptr %1, align 4
  %2 = load ptr, ptr %p, align 8
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %result, align 4
  %4 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %4)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multiple_stores_single_load() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 1, ptr %0, align 4
  %1 = load ptr, ptr %p, align 8
  store i32 2, ptr %1, align 4
  %2 = load ptr, ptr %p, align 8
  store i32 3, ptr %2, align 4
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %result, align 4
  %5 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %5)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @single_store_multiple_loads() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %r1 = alloca i32, align 4
  %r2 = alloca i32, align 4
  %r3 = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 100, ptr %0, align 4
  %1 = load ptr, ptr %p, align 8
  %2 = load i32, ptr %1, align 4
  store i32 %2, ptr %r1, align 4
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %r2, align 4
  %5 = load ptr, ptr %p, align 8
  %6 = load i32, ptr %5, align 4
  store i32 %6, ptr %r3, align 4
  %7 = load i32, ptr %r1, align 4
  call void @sink(i32 noundef %7)
  %8 = load i32, ptr %r2, align 4
  call void @sink(i32 noundef %8)
  %9 = load i32, ptr %r3, align 4
  call void @sink(i32 noundef %9)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @interleaved_store_load() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %r1 = alloca i32, align 4
  %r2 = alloca i32, align 4
  %r3 = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 10, ptr %0, align 4
  %1 = load ptr, ptr %p, align 8
  %2 = load i32, ptr %1, align 4
  store i32 %2, ptr %r1, align 4
  %3 = load ptr, ptr %p, align 8
  store i32 20, ptr %3, align 4
  %4 = load ptr, ptr %p, align 8
  %5 = load i32, ptr %4, align 4
  store i32 %5, ptr %r2, align 4
  %6 = load ptr, ptr %p, align 8
  store i32 30, ptr %6, align 4
  %7 = load ptr, ptr %p, align 8
  %8 = load i32, ptr %7, align 4
  store i32 %8, ptr %r3, align 4
  %9 = load i32, ptr %r1, align 4
  call void @sink(i32 noundef %9)
  %10 = load i32, ptr %r2, align 4
  call void @sink(i32 noundef %10)
  %11 = load i32, ptr %r3, align 4
  call void @sink(i32 noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @store_helper(ptr noundef %p, i32 noundef %val) #0 {
entry:
  %p.addr = alloca ptr, align 8
  %val.addr = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  store i32 %val, ptr %val.addr, align 4
  %0 = load i32, ptr %val.addr, align 4
  %1 = load ptr, ptr %p.addr, align 8
  store i32 %0, ptr %1, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @load_helper(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  %1 = load i32, ptr %0, align 4
  store i32 %1, ptr %result, align 4
  %2 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @interproc_store_load() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  call void @store_helper(ptr noundef %0, i32 noundef 999)
  %1 = load ptr, ptr %p, align 8
  call void @load_helper(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @no_edge_different_locations() #0 {
entry:
  %a = alloca i32, align 4
  %b = alloca i32, align 4
  %pa = alloca ptr, align 8
  %pb = alloca ptr, align 8
  %r = alloca i32, align 4
  store ptr %a, ptr %pa, align 8
  store ptr %b, ptr %pb, align 8
  %0 = load ptr, ptr %pa, align 8
  store i32 111, ptr %0, align 4
  %1 = load ptr, ptr %pb, align 8
  %2 = load i32, ptr %1, align 4
  store i32 %2, ptr %r, align 4
  %3 = load i32, ptr %r, align 4
  call void @sink(i32 noundef %3)
  ret void
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
