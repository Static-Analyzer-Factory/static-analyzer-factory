; ModuleID = 'widening_loop_simple.c'
source_filename = "widening_loop_simple.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @simple_loop(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %x, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %0 = load i32, ptr %x, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %2 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %2)
  %3 = load i32, ptr %x, align 4
  %add = add nsw i32 %3, 1
  store i32 %add, ptr %x, align 4
  br label %while.cond, !llvm.loop !6

while.end:                                        ; preds = %while.cond
  %4 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %4)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @nested_loops(i32 noundef %m, i32 noundef %n) #0 {
entry:
  %m.addr = alloca i32, align 4
  %n.addr = alloca i32, align 4
  %outer = alloca i32, align 4
  %inner = alloca i32, align 4
  store i32 %m, ptr %m.addr, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %outer, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.end, %entry
  %0 = load i32, ptr %outer, align 4
  %1 = load i32, ptr %m.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %while.body, label %while.end6

while.body:                                       ; preds = %while.cond
  store i32 0, ptr %inner, align 4
  br label %while.cond1

while.cond1:                                      ; preds = %while.body3, %while.body
  %2 = load i32, ptr %inner, align 4
  %3 = load i32, ptr %n.addr, align 4
  %cmp2 = icmp slt i32 %2, %3
  br i1 %cmp2, label %while.body3, label %while.end

while.body3:                                      ; preds = %while.cond1
  %4 = load i32, ptr %outer, align 4
  %5 = load i32, ptr %inner, align 4
  %add = add nsw i32 %4, %5
  call void @sink(i32 noundef %add)
  %6 = load i32, ptr %inner, align 4
  %add4 = add nsw i32 %6, 1
  store i32 %add4, ptr %inner, align 4
  br label %while.cond1, !llvm.loop !8

while.end:                                        ; preds = %while.cond1
  %7 = load i32, ptr %outer, align 4
  %add5 = add nsw i32 %7, 1
  store i32 %add5, ptr %outer, align 4
  br label %while.cond, !llvm.loop !9

while.end6:                                       ; preds = %while.cond
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @multiple_exits(ptr noundef %arr, i32 noundef %len) #0 {
entry:
  %retval = alloca i32, align 4
  %arr.addr = alloca ptr, align 8
  %len.addr = alloca i32, align 4
  %sum = alloca i32, align 4
  %i = alloca i32, align 4
  store ptr %arr, ptr %arr.addr, align 8
  store i32 %len, ptr %len.addr, align 4
  store i32 0, ptr %sum, align 4
  store i32 0, ptr %i, align 4
  br label %while.cond

while.cond:                                       ; preds = %if.end, %entry
  %0 = load i32, ptr %i, align 4
  %1 = load i32, ptr %len.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %while.body, label %while.end

while.body:                                       ; preds = %while.cond
  %2 = load ptr, ptr %arr.addr, align 8
  %3 = load i32, ptr %i, align 4
  %idxprom = sext i32 %3 to i64
  %arrayidx = getelementptr inbounds i32, ptr %2, i64 %idxprom
  %4 = load i32, ptr %arrayidx, align 4
  %cmp1 = icmp slt i32 %4, 0
  br i1 %cmp1, label %if.then, label %if.end

if.then:                                          ; preds = %while.body
  %5 = load i32, ptr %sum, align 4
  store i32 %5, ptr %retval, align 4
  br label %return

if.end:                                           ; preds = %while.body
  %6 = load i32, ptr %sum, align 4
  %7 = load ptr, ptr %arr.addr, align 8
  %8 = load i32, ptr %i, align 4
  %idxprom2 = sext i32 %8 to i64
  %arrayidx3 = getelementptr inbounds i32, ptr %7, i64 %idxprom2
  %9 = load i32, ptr %arrayidx3, align 4
  %add = add nsw i32 %6, %9
  store i32 %add, ptr %sum, align 4
  %10 = load i32, ptr %i, align 4
  %add4 = add nsw i32 %10, 1
  store i32 %add4, ptr %i, align 4
  br label %while.cond, !llvm.loop !10

while.end:                                        ; preds = %while.cond
  %11 = load i32, ptr %sum, align 4
  store i32 %11, ptr %retval, align 4
  br label %return

return:                                           ; preds = %while.end, %if.then
  %12 = load i32, ptr %retval, align 4
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @unbounded_counter() #0 {
entry:
  %count = alloca i32, align 4
  store i32 0, ptr %count, align 4
  br label %while.body

while.body:                                       ; preds = %entry, %if.end
  %0 = load i32, ptr %count, align 4
  call void @sink(i32 noundef %0)
  %1 = load i32, ptr %count, align 4
  %add = add nsw i32 %1, 1
  store i32 %add, ptr %count, align 4
  %2 = load i32, ptr %count, align 4
  %cmp = icmp sgt i32 %2, 1000000
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %while.body
  br label %while.end

if.end:                                           ; preds = %while.body
  br label %while.body

while.end:                                        ; preds = %if.then
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
