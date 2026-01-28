; ModuleID = 'zero_fact_branches.c'
source_filename = "zero_fact_branches.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [5 x i8] c"USER\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"DATA\00", align 1
@.str.2 = private unnamed_addr constant [6 x i8] c"INPUT\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_zero_fact_branches(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %tainted = alloca ptr, align 8
  store i32 %cond, ptr %cond.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str)
  store ptr %call, ptr %tainted, align 8
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %tainted, align 8
  call void @sink(ptr noundef %1)
  br label %if.end

if.else:                                          ; preds = %entry
  %2 = load ptr, ptr %tainted, align 8
  call void @sink(ptr noundef %2)
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load ptr, ptr %tainted, align 8
  call void @sink(ptr noundef %3)
  ret void
}

declare ptr @getenv(ptr noundef) #1

declare void @sink(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_nested_branches(i32 noundef %a, i32 noundef %b) #0 {
entry:
  %a.addr = alloca i32, align 4
  %b.addr = alloca i32, align 4
  %data = alloca ptr, align 8
  store i32 %a, ptr %a.addr, align 4
  store i32 %b, ptr %b.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str.1)
  store ptr %call, ptr %data, align 8
  %0 = load i32, ptr %a.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.end3

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %b.addr, align 4
  %tobool1 = icmp ne i32 %1, 0
  br i1 %tobool1, label %if.then2, label %if.end

if.then2:                                         ; preds = %if.then
  %2 = load ptr, ptr %data, align 8
  call void @sink(ptr noundef %2)
  br label %if.end

if.end:                                           ; preds = %if.then2, %if.then
  br label %if.end3

if.end3:                                          ; preds = %if.end, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_loop_zero_fact(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %input = alloca ptr, align 8
  %i = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str.2)
  store ptr %call, ptr %input, align 8
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc, %entry
  %0 = load i32, ptr %i, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %2 = load ptr, ptr %input, align 8
  call void @sink(ptr noundef %2)
  br label %for.inc

for.inc:                                          ; preds = %for.body
  %3 = load i32, ptr %i, align 4
  %inc = add nsw i32 %3, 1
  store i32 %inc, ptr %i, align 4
  br label %for.cond, !llvm.loop !6

for.end:                                          ; preds = %for.cond
  %4 = load ptr, ptr %input, align 8
  call void @sink(ptr noundef %4)
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
!6 = distinct !{!6, !7}
!7 = !{!"llvm.loop.mustprogress"}
