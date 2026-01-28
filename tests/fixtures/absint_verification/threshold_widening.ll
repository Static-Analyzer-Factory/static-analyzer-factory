; ModuleID = 'threshold_widening.c'
source_filename = "threshold_widening.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @bounded_loop() #0 {
entry:
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %x, align 4
  %cmp = icmp slt i32 %0, 100
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %1 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %1)
  %2 = load i32, ptr %x, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %x, align 4
  br label %while.cond, !llvm.loop !6

while.end:                                        ; preds = %while.cond
  %3 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %3)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @buffer_iteration() #0 {
entry:
  %buffer = alloca [256 x i32], align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %i, align 4
  %cmp = icmp slt i32 %0, 256
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %1 = load i32, ptr %i, align 4
  %idxprom = sext i32 %1 to i64
  %arrayidx = getelementptr inbounds [256 x i32], ptr %buffer, i64 0, i64 %idxprom
  store i32 0, ptr %arrayidx, align 4
  %2 = load i32, ptr %i, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %i, align 4
  br label %while.cond, !llvm.loop !8

while.end:                                        ; preds = %while.cond
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multi_threshold(i32 noundef %mode) #0 {
entry:
  %mode.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %mode, ptr %mode.addr, align 4
  store i32 0, ptr %x, align 4
  %0 = load i32, ptr %mode.addr, align 4
  %cmp = icmp eq i32 %0, 0
  br i1 %cmp, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  br label %while.cond

while.cond:                                       ; preds = %while.body, %if.then
  %1 = load i32, ptr %x, align 4
  %cmp1 = icmp slt i32 %1, 10
  br i1 %cmp1, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %2 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %2)
  %3 = load i32, ptr %x, align 4
  %add = add nsw i32 %3, 1
  store i32 %add, ptr %x, align 4
  br label %while.cond, !llvm.loop !9

while.end:                                        ; preds = %while.cond
  br label %if.end15

if.else:                                          ; preds = %entry
  %4 = load i32, ptr %mode.addr, align 4
  %cmp2 = icmp eq i32 %4, 1
  br i1 %cmp2, label %if.then3, label %if.else9

if.then3:                                         ; preds = %if.else
  br label %while.cond4

while.cond4:                                      ; preds = %while.body6, %if.then3
  %5 = load i32, ptr %x, align 4
  %cmp5 = icmp slt i32 %5, 100
  br i1 %cmp5, label %while.body6, label %while.end8

while.body6:                                      ; preds = %while.cond4
  %6 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %6)
  %7 = load i32, ptr %x, align 4
  %add7 = add nsw i32 %7, 1
  store i32 %add7, ptr %x, align 4
  br label %while.cond4, !llvm.loop !10

while.end8:                                       ; preds = %while.cond4
  br label %if.end

if.else9:                                         ; preds = %if.else
  br label %while.cond10

while.cond10:                                     ; preds = %while.body12, %if.else9
  %8 = load i32, ptr %x, align 4
  %cmp11 = icmp slt i32 %8, 1000
  br i1 %cmp11, label %while.body12, label %while.end14

while.body12:                                     ; preds = %while.cond10
  %9 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %9)
  %10 = load i32, ptr %x, align 4
  %add13 = add nsw i32 %10, 1
  store i32 %add13, ptr %x, align 4
  br label %while.cond10, !llvm.loop !11

while.end14:                                      ; preds = %while.cond10
  br label %if.end

if.end:                                           ; preds = %while.end14, %while.end8
  br label %if.end15

if.end15:                                         ; preds = %if.end, %while.end
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @global_threshold() #0 {
entry:
  %items = alloca i32, align 4
  store i32 0, ptr %items, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %items, align 4
  %cmp = icmp slt i32 %0, 1024
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %1 = load i32, ptr %items, align 4
  call void @sink(i32 noundef %1)
  %2 = load i32, ptr %items, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %items, align 4
  br label %while.cond, !llvm.loop !12

while.end:                                        ; preds = %while.cond
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @off_by_one() #0 {
entry:
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %x, align 4
  %cmp = icmp sle i32 %0, 99
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %1 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %1)
  %2 = load i32, ptr %x, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %x, align 4
  br label %while.cond, !llvm.loop !13

while.end:                                        ; preds = %while.cond
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
!8 = distinct !{!8, !7}
!9 = distinct !{!9, !7}
!10 = distinct !{!10, !7}
!11 = distinct !{!11, !7}
!12 = distinct !{!12, !7}
!13 = distinct !{!13, !7}
