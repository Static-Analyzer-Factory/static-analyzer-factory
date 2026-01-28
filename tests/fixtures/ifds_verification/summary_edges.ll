; ModuleID = 'summary_edges.c'
source_filename = "summary_edges.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [6 x i8] c"clean\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"VAR1\00", align 1
@.str.2 = private unnamed_addr constant [5 x i8] c"VAR2\00", align 1
@.str.3 = private unnamed_addr constant [3 x i8] c"C1\00", align 1
@.str.4 = private unnamed_addr constant [3 x i8] c"C2\00", align 1
@.str.5 = private unnamed_addr constant [8 x i8] c"TAINTED\00", align 1
@.str.6 = private unnamed_addr constant [5 x i8] c"DATA\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @passthrough(ptr noundef %data) #0 {
entry:
  %data.addr = alloca ptr, align 8
  store ptr %data, ptr %data.addr, align 8
  %0 = load ptr, ptr %data.addr, align 8
  ret ptr %0
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @sanitize(ptr noundef %data) #0 {
entry:
  %data.addr = alloca ptr, align 8
  store ptr %data, ptr %data.addr, align 8
  ret ptr @.str
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_summary_reuse() #0 {
entry:
  %t1 = alloca ptr, align 8
  %t2 = alloca ptr, align 8
  %r1 = alloca ptr, align 8
  %r2 = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.1)
  store ptr %call, ptr %t1, align 8
  %call1 = call ptr @getenv(ptr noundef @.str.2)
  store ptr %call1, ptr %t2, align 8
  %0 = load ptr, ptr %t1, align 8
  %call2 = call ptr @passthrough(ptr noundef %0)
  store ptr %call2, ptr %r1, align 8
  %1 = load ptr, ptr %t2, align 8
  %call3 = call ptr @passthrough(ptr noundef %1)
  store ptr %call3, ptr %r2, align 8
  %2 = load ptr, ptr %r1, align 8
  call void @sink(ptr noundef %2)
  %3 = load ptr, ptr %r2, align 8
  call void @sink(ptr noundef %3)
  ret void
}

declare ptr @getenv(ptr noundef) #1

declare void @sink(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @caller1() #0 {
entry:
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.3)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @passthrough(ptr noundef %0)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @caller2() #0 {
entry:
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.4)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @passthrough(ptr noundef %0)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_sanitizer_summary() #0 {
entry:
  %tainted = alloca ptr, align 8
  %clean = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.5)
  store ptr %call, ptr %tainted, align 8
  %0 = load ptr, ptr %tainted, align 8
  %call1 = call ptr @sanitize(ptr noundef %0)
  store ptr %call1, ptr %clean, align 8
  %1 = load ptr, ptr %clean, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @level2(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  ret ptr %0
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @level1(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  %call = call ptr @level2(ptr noundef %0)
  ret ptr %call
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_multi_level_summary() #0 {
entry:
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.6)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @level1(ptr noundef %0)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
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
