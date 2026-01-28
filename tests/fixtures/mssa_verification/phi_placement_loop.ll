; ModuleID = 'phi_placement_loop.c'
source_filename = "phi_placement_loop.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @loop_phi(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %i = alloca i32, align 4
  %prev = alloca i32, align 4
  %result = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store i32 0, ptr %x, align 4
  store ptr %x, ptr %p, align 8
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc, %entry
  %0 = load i32, ptr %i, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %2 = load ptr, ptr %p, align 8
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %prev, align 4
  %4 = load i32, ptr %prev, align 4
  %add = add nsw i32 %4, 1
  %5 = load ptr, ptr %p, align 8
  store i32 %add, ptr %5, align 4
  br label %for.inc

for.inc:                                          ; preds = %for.body
  %6 = load i32, ptr %i, align 4
  %inc = add nsw i32 %6, 1
  store i32 %inc, ptr %i, align 4
  br label %for.cond, !llvm.loop !6

for.end:                                          ; preds = %for.cond
  %7 = load ptr, ptr %p, align 8
  %8 = load i32, ptr %7, align 4
  store i32 %8, ptr %result, align 4
  %9 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %9)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @nested_loop_phi(i32 noundef %m, i32 noundef %n) #0 {
entry:
  %m.addr = alloca i32, align 4
  %n.addr = alloca i32, align 4
  %arr = alloca [10 x i32], align 4
  %p = alloca ptr, align 8
  %i = alloca i32, align 4
  %j = alloca i32, align 4
  %result = alloca i32, align 4
  store i32 %m, ptr %m.addr, align 4
  store i32 %n, ptr %n.addr, align 4
  %arrayidx = getelementptr inbounds [10 x i32], ptr %arr, i64 0, i64 0
  store ptr %arrayidx, ptr %p, align 8
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc4, %entry
  %0 = load i32, ptr %i, align 4
  %1 = load i32, ptr %m.addr, align 4
  %cmp = icmp slt i32 %0, %1
  br i1 %cmp, label %for.body, label %for.end6

for.body:                                         ; preds = %for.cond
  store i32 0, ptr %j, align 4
  br label %for.cond1

for.cond1:                                        ; preds = %for.inc, %for.body
  %2 = load i32, ptr %j, align 4
  %3 = load i32, ptr %n.addr, align 4
  %cmp2 = icmp slt i32 %2, %3
  br i1 %cmp2, label %for.body3, label %for.end

for.body3:                                        ; preds = %for.cond1
  %4 = load i32, ptr %i, align 4
  %5 = load i32, ptr %n.addr, align 4
  %mul = mul nsw i32 %4, %5
  %6 = load i32, ptr %j, align 4
  %add = add nsw i32 %mul, %6
  %7 = load ptr, ptr %p, align 8
  store i32 %add, ptr %7, align 4
  br label %for.inc

for.inc:                                          ; preds = %for.body3
  %8 = load i32, ptr %j, align 4
  %inc = add nsw i32 %8, 1
  store i32 %inc, ptr %j, align 4
  br label %for.cond1, !llvm.loop !8

for.end:                                          ; preds = %for.cond1
  br label %for.inc4

for.inc4:                                         ; preds = %for.end
  %9 = load i32, ptr %i, align 4
  %inc5 = add nsw i32 %9, 1
  store i32 %inc5, ptr %i, align 4
  br label %for.cond, !llvm.loop !9

for.end6:                                         ; preds = %for.cond
  %10 = load ptr, ptr %p, align 8
  %11 = load i32, ptr %10, align 4
  store i32 %11, ptr %result, align 4
  %12 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %12)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @while_loop_phi(i32 noundef %cond, i32 noundef %limit) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %limit.addr = alloca i32, align 4
  %counter = alloca i32, align 4
  %p = alloca ptr, align 8
  store i32 %cond, ptr %cond.addr, align 4
  store i32 %limit, ptr %limit.addr, align 4
  store ptr %counter, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 0, ptr %0, align 4
  br label %while.cond

while.cond:                                       ; preds = %while.body, %entry
  %1 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %1, 0
  br i1 %tobool, label %land.rhs, label %land.end

land.rhs:                                         ; preds = %while.cond
  %2 = load ptr, ptr %p, align 8
  %3 = load i32, ptr %2, align 4
  %4 = load i32, ptr %limit.addr, align 4
  %cmp = icmp slt i32 %3, %4
  br label %land.end

land.end:                                         ; preds = %land.rhs, %while.cond
  %5 = phi i1 [ false, %while.cond ], [ %cmp, %land.rhs ]
  br i1 %5, label %while.body, label %while.end

while.body:                                       ; preds = %land.end
  %6 = load ptr, ptr %p, align 8
  %7 = load i32, ptr %6, align 4
  %add = add nsw i32 %7, 1
  %8 = load ptr, ptr %p, align 8
  store i32 %add, ptr %8, align 4
  br label %while.cond, !llvm.loop !10

while.end:                                        ; preds = %land.end
  %9 = load ptr, ptr %p, align 8
  %10 = load i32, ptr %9, align 4
  call void @sink(i32 noundef %10)
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
