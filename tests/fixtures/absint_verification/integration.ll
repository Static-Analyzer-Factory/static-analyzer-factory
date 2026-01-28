; ModuleID = 'integration.c'
source_filename = "integration.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @buffer_overflow_test(i32 noundef %index) #0 {
entry:
  %index.addr = alloca i32, align 4
  %buffer = alloca [100 x i32], align 4
  store i32 %index, ptr %index.addr, align 4
  %0 = load i32, ptr %index.addr, align 4
  %idxprom = sext i32 %0 to i64
  %arrayidx = getelementptr inbounds [100 x i32], ptr %buffer, i64 0, i64 %idxprom
  store i32 42, ptr %arrayidx, align 4
  %1 = load i32, ptr %index.addr, align 4
  %cmp = icmp sge i32 %1, 0
  br i1 %cmp, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %2 = load i32, ptr %index.addr, align 4
  %cmp1 = icmp slt i32 %2, 100
  br i1 %cmp1, label %if.then, label %if.end

if.then:                                          ; preds = %land.lhs.true
  %3 = load i32, ptr %index.addr, align 4
  %idxprom2 = sext i32 %3 to i64
  %arrayidx3 = getelementptr inbounds [100 x i32], ptr %buffer, i64 0, i64 %idxprom2
  store i32 100, ptr %arrayidx3, align 4
  br label %if.end

if.end:                                           ; preds = %if.then, %land.lhs.true, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @integer_overflow_test(i32 noundef %a, i32 noundef %b) #0 {
entry:
  %a.addr = alloca i32, align 4
  %b.addr = alloca i32, align 4
  %sum = alloca i32, align 4
  %safe_sum = alloca i32, align 4
  store i32 %a, ptr %a.addr, align 4
  store i32 %b, ptr %b.addr, align 4
  %0 = load i32, ptr %a.addr, align 4
  %1 = load i32, ptr %b.addr, align 4
  %add = add nsw i32 %0, %1
  store i32 %add, ptr %sum, align 4
  %2 = load i32, ptr %sum, align 4
  call void @sink(i32 noundef %2)
  %3 = load i32, ptr %a.addr, align 4
  %cmp = icmp sge i32 %3, 0
  br i1 %cmp, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %4 = load i32, ptr %a.addr, align 4
  %cmp1 = icmp sle i32 %4, 1000
  br i1 %cmp1, label %land.lhs.true2, label %if.end

land.lhs.true2:                                   ; preds = %land.lhs.true
  %5 = load i32, ptr %b.addr, align 4
  %cmp3 = icmp sge i32 %5, 0
  br i1 %cmp3, label %land.lhs.true4, label %if.end

land.lhs.true4:                                   ; preds = %land.lhs.true2
  %6 = load i32, ptr %b.addr, align 4
  %cmp5 = icmp sle i32 %6, 1000
  br i1 %cmp5, label %if.then, label %if.end

if.then:                                          ; preds = %land.lhs.true4
  %7 = load i32, ptr %a.addr, align 4
  %8 = load i32, ptr %b.addr, align 4
  %add6 = add nsw i32 %7, %8
  store i32 %add6, ptr %safe_sum, align 4
  %9 = load i32, ptr %safe_sum, align 4
  call void @sink(i32 noundef %9)
  br label %if.end

if.end:                                           ; preds = %if.then, %land.lhs.true4, %land.lhs.true2, %land.lhs.true, %entry
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @safe_loop_access() #0 {
entry:
  %arr = alloca [10 x i32], align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc, %entry
  %0 = load i32, ptr %i, align 4
  %cmp = icmp slt i32 %0, 10
  br i1 %cmp, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %1 = load i32, ptr %i, align 4
  %mul = mul nsw i32 %1, 2
  %2 = load i32, ptr %i, align 4
  %idxprom = sext i32 %2 to i64
  %arrayidx = getelementptr inbounds [10 x i32], ptr %arr, i64 0, i64 %idxprom
  store i32 %mul, ptr %arrayidx, align 4
  br label %for.inc

for.inc:                                          ; preds = %for.body
  %3 = load i32, ptr %i, align 4
  %inc = add nsw i32 %3, 1
  store i32 %inc, ptr %i, align 4
  br label %for.cond, !llvm.loop !6

for.end:                                          ; preds = %for.cond
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @deterministic_computation(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %sum = alloca i32, align 4
  %i = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %sum, align 4
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc, %entry
  %0 = load i32, ptr %i, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %2 = load i32, ptr %i, align 4
  %3 = load i32, ptr %sum, align 4
  %add = add nsw i32 %3, %2
  store i32 %add, ptr %sum, align 4
  br label %for.inc

for.inc:                                          ; preds = %for.body
  %4 = load i32, ptr %i, align 4
  %inc = add nsw i32 %4, 1
  store i32 %inc, ptr %i, align 4
  br label %for.cond, !llvm.loop !8

for.end:                                          ; preds = %for.cond
  %5 = load i32, ptr %sum, align 4
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @complex_arithmetic(i32 noundef %x, i32 noundef %y, i32 noundef %mode) #0 {
entry:
  %x.addr = alloca i32, align 4
  %y.addr = alloca i32, align 4
  %mode.addr = alloca i32, align 4
  %result = alloca i32, align 4
  store i32 %x, ptr %x.addr, align 4
  store i32 %y, ptr %y.addr, align 4
  store i32 %mode, ptr %mode.addr, align 4
  store i32 0, ptr %result, align 4
  %0 = load i32, ptr %mode.addr, align 4
  %cmp = icmp eq i32 %0, 0
  br i1 %cmp, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %x.addr, align 4
  %2 = load i32, ptr %y.addr, align 4
  %add = add nsw i32 %1, %2
  store i32 %add, ptr %result, align 4
  br label %if.end11

if.else:                                          ; preds = %entry
  %3 = load i32, ptr %mode.addr, align 4
  %cmp1 = icmp eq i32 %3, 1
  br i1 %cmp1, label %if.then2, label %if.else3

if.then2:                                         ; preds = %if.else
  %4 = load i32, ptr %x.addr, align 4
  %5 = load i32, ptr %y.addr, align 4
  %mul = mul nsw i32 %4, %5
  store i32 %mul, ptr %result, align 4
  br label %if.end10

if.else3:                                         ; preds = %if.else
  %6 = load i32, ptr %mode.addr, align 4
  %cmp4 = icmp eq i32 %6, 2
  br i1 %cmp4, label %if.then5, label %if.else8

if.then5:                                         ; preds = %if.else3
  %7 = load i32, ptr %y.addr, align 4
  %cmp6 = icmp ne i32 %7, 0
  br i1 %cmp6, label %if.then7, label %if.end

if.then7:                                         ; preds = %if.then5
  %8 = load i32, ptr %x.addr, align 4
  %9 = load i32, ptr %y.addr, align 4
  %div = sdiv i32 %8, %9
  store i32 %div, ptr %result, align 4
  br label %if.end

if.end:                                           ; preds = %if.then7, %if.then5
  br label %if.end9

if.else8:                                         ; preds = %if.else3
  %10 = load i32, ptr %x.addr, align 4
  %11 = load i32, ptr %y.addr, align 4
  %sub = sub nsw i32 %10, %11
  store i32 %sub, ptr %result, align 4
  br label %if.end9

if.end9:                                          ; preds = %if.else8, %if.end
  br label %if.end10

if.end10:                                         ; preds = %if.end9, %if.then2
  br label %if.end11

if.end11:                                         ; preds = %if.end10, %if.then
  %12 = load i32, ptr %result, align 4
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @cascading_bounds(i32 noundef %x) #0 {
entry:
  %x.addr = alloca i32, align 4
  store i32 %x, ptr %x.addr, align 4
  %0 = load i32, ptr %x.addr, align 4
  %cmp = icmp sgt i32 %0, -100
  br i1 %cmp, label %if.then, label %if.end6

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %x.addr, align 4
  %cmp1 = icmp slt i32 %1, 100
  br i1 %cmp1, label %if.then2, label %if.end5

if.then2:                                         ; preds = %if.then
  %2 = load i32, ptr %x.addr, align 4
  %cmp3 = icmp sge i32 %2, 0
  br i1 %cmp3, label %if.then4, label %if.end

if.then4:                                         ; preds = %if.then2
  %3 = load i32, ptr %x.addr, align 4
  call void @sink(i32 noundef %3)
  br label %if.end

if.end:                                           ; preds = %if.then4, %if.then2
  br label %if.end5

if.end5:                                          ; preds = %if.end, %if.then
  br label %if.end6

if.end6:                                          ; preds = %if.end5, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multi_counter_loop(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %a = alloca i32, align 4
  %b = alloca i32, align 4
  %c = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %a, align 4
  %0 = load i32, ptr %n.addr, align 4
  store i32 %0, ptr %b, align 4
  store i32 0, ptr %c, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %1 = load i32, ptr %a, align 4
  %2 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %1, %2
  br i1 %cmp, label %land.rhs, label %land.end

land.rhs:                                         ; preds = %while.cond
  %3 = load i32, ptr %b, align 4
  %cmp1 = icmp sgt i32 %3, 0
  br label %land.end

land.end:                                         ; preds = %land.rhs, %while.cond
  %4 = phi i1 [ false, %while.cond ], [ %cmp1, %land.rhs ]
  br i1 %4, label %while.body, label %while.end

while.body:                                       ; preds = %land.end
  %5 = load i32, ptr %a, align 4
  %add = add nsw i32 %5, 1
  store i32 %add, ptr %a, align 4
  %6 = load i32, ptr %b, align 4
  %sub = sub nsw i32 %6, 1
  store i32 %sub, ptr %b, align 4
  %7 = load i32, ptr %a, align 4
  %8 = load i32, ptr %b, align 4
  %add2 = add nsw i32 %7, %8
  store i32 %add2, ptr %c, align 4
  %9 = load i32, ptr %c, align 4
  call void @sink(i32 noundef %9)
  br label %while.cond, !llvm.loop !9

while.end:                                        ; preds = %land.end
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
entry:
  %retval = alloca i32, align 4
  %d = alloca i32, align 4
  %r = alloca i32, align 4
  store i32 0, ptr %retval, align 4
  call void @buffer_overflow_test(i32 noundef 50)
  call void @integer_overflow_test(i32 noundef 100, i32 noundef 200)
  call void @safe_loop_access()
  %call = call i32 @deterministic_computation(i32 noundef 10)
  store i32 %call, ptr %d, align 4
  %0 = load i32, ptr %d, align 4
  call void @sink(i32 noundef %0)
  %call1 = call i32 @complex_arithmetic(i32 noundef 10, i32 noundef 5, i32 noundef 0)
  store i32 %call1, ptr %r, align 4
  %1 = load i32, ptr %r, align 4
  call void @sink(i32 noundef %1)
  call void @cascading_bounds(i32 noundef 42)
  call void @multi_counter_loop(i32 noundef 100)
  ret i32 0
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
!8 = distinct !{!8, !7}
!9 = distinct !{!9, !7}
