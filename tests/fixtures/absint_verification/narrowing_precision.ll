; ModuleID = 'narrowing_precision.c'
source_filename = "narrowing_precision.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @basic_narrowing() #0 {
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
  %add = add nsw i32 %1, 1
  store i32 %add, ptr %x, align 4
  br label %while.cond, !llvm.loop !6

while.end:                                        ; preds = %while.cond
  %2 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %2)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multi_value_narrowing() #0 {
entry:
  %a = alloca i32, align 4
  %b = alloca i32, align 4
  store i32 0, ptr %a, align 4
  store i32 10, ptr %b, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %a, align 4
  %cmp = icmp slt i32 %0, 50
  br i1 %cmp, label %land.rhs, label %land.end

land.rhs:                                         ; preds = %while.cond
  %1 = load i32, ptr %b, align 4
  %cmp1 = icmp slt i32 %1, 60
  br label %land.end

land.end:                                         ; preds = %land.rhs, %while.cond
  %2 = phi i1 [ false, %while.cond ], [ %cmp1, %land.rhs ]
  br i1 %2, label %while.body, label %while.end

while.body:                                       ; preds = %land.end
  %3 = load i32, ptr %a, align 4
  %add = add nsw i32 %3, 1
  store i32 %add, ptr %a, align 4
  %4 = load i32, ptr %b, align 4
  %add2 = add nsw i32 %4, 1
  store i32 %add2, ptr %b, align 4
  br label %while.cond, !llvm.loop !8

while.end:                                        ; preds = %land.end
  %5 = load i32, ptr %a, align 4
  call void @sink(i32 noundef %5)
  %6 = load i32, ptr %b, align 4
  call void @sink(i32 noundef %6)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @nested_narrowing() #0 {
entry:
  %outer = alloca i32, align 4
  %inner = alloca i32, align 4
  store i32 0, ptr %outer, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.end, %entry
  %0 = load i32, ptr %outer, align 4
  %cmp = icmp slt i32 %0, 10
  br i1 %cmp, label %while.body, label %while.end5

while.body:                                       ; preds = %while.cond
  store i32 0, ptr %inner, align 4
  br label %while.cond1

while.cond1:                                      ; preds = %while.body3, %while.body
  %1 = load i32, ptr %inner, align 4
  %cmp2 = icmp slt i32 %1, 5
  br i1 %cmp2, label %while.body3, label %while.end

while.body3:                                      ; preds = %while.cond1
  %2 = load i32, ptr %inner, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %inner, align 4
  br label %while.cond1, !llvm.loop !9

while.end:                                        ; preds = %while.cond1
  %3 = load i32, ptr %inner, align 4
  call void @sink(i32 noundef %3)
  %4 = load i32, ptr %outer, align 4
  %add4 = add nsw i32 %4, 1
  store i32 %add4, ptr %outer, align 4
  br label %while.cond, !llvm.loop !10

while.end5:                                       ; preds = %while.cond
  %5 = load i32, ptr %outer, align 4
  call void @sink(i32 noundef %5)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @branch_narrowing(i32 noundef %input) #0 {
entry:
  %input.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %input, ptr %input.addr, align 4
  %0 = load i32, ptr %input.addr, align 4
  store i32 %0, ptr %x, align 4
  %1 = load i32, ptr %x, align 4
  %cmp = icmp sgt i32 %1, 0
  br i1 %cmp, label %if.then, label %if.end3

if.then:                                          ; preds = %entry
  %2 = load i32, ptr %x, align 4
  %cmp1 = icmp slt i32 %2, 100
  br i1 %cmp1, label %if.then2, label %if.end

if.then2:                                         ; preds = %if.then
  %3 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %3)
  br label %if.end

if.end:                                           ; preds = %if.then2, %if.then
  br label %if.end3

if.end3:                                          ; preds = %if.end, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @condition_narrowing() #0 {
entry:
  %i = alloca i32, align 4
  %sum = alloca i32, align 4
  store i32 0, ptr %i, align 4
  store i32 0, ptr %sum, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %i, align 4
  %cmp = icmp slt i32 %0, 10
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %1 = load i32, ptr %sum, align 4
  %2 = load i32, ptr %i, align 4
  %add = add nsw i32 %1, %2
  store i32 %add, ptr %sum, align 4
  %3 = load i32, ptr %i, align 4
  %add1 = add nsw i32 %3, 1
  store i32 %add1, ptr %i, align 4
  br label %while.cond, !llvm.loop !11

while.end:                                        ; preds = %while.cond
  %4 = load i32, ptr %i, align 4
  %cmp2 = icmp eq i32 %4, 10
  br i1 %cmp2, label %if.then, label %if.end

if.then:                                          ; preds = %while.end
  %5 = load i32, ptr %sum, align 4
  call void @sink(i32 noundef %5)
  br label %if.end

if.end:                                           ; preds = %if.then, %while.end
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @interleaved_narrowing(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %x, align 4
  store i32 100, ptr %y, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %x, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %2 = load i32, ptr %x, align 4
  %add = add nsw i32 %2, 1
  store i32 %add, ptr %x, align 4
  %3 = load i32, ptr %y, align 4
  %sub = sub nsw i32 %3, 1
  store i32 %sub, ptr %y, align 4
  br label %while.cond, !llvm.loop !12

while.end:                                        ; preds = %while.cond
  %4 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %4)
  %5 = load i32, ptr %y, align 4
  call void @sink(i32 noundef %5)
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
