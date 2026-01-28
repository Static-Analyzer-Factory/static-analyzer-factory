; ModuleID = 'dominator_computation.c'
source_filename = "dominator_computation.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @linear_cfg() #0 {
entry:
  %a = alloca i32, align 4
  %b = alloca i32, align 4
  %c = alloca i32, align 4
  %d = alloca i32, align 4
  store i32 1, ptr %a, align 4
  %0 = load i32, ptr %a, align 4
  %add = add nsw i32 %0, 1
  store i32 %add, ptr %b, align 4
  %1 = load i32, ptr %b, align 4
  %add1 = add nsw i32 %1, 1
  store i32 %add1, ptr %c, align 4
  %2 = load i32, ptr %c, align 4
  %add2 = add nsw i32 %2, 1
  store i32 %add2, ptr %d, align 4
  %3 = load i32, ptr %d, align 4
  call void @sink(i32 noundef %3)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @diamond_cfg(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 10, ptr %x, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  store i32 20, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %1 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @nested_diamond(i32 noundef %c1, i32 noundef %c2) #0 {
entry:
  %c1.addr = alloca i32, align 4
  %c2.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %c1, ptr %c1.addr, align 4
  store i32 %c2, ptr %c2.addr, align 4
  %0 = load i32, ptr %c1.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else3

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %c2.addr, align 4
  %tobool1 = icmp ne i32 %1, 0
  br i1 %tobool1, label %if.then2, label %if.else

if.then2:                                         ; preds = %if.then
  store i32 1, ptr %x, align 4
  br label %if.end

if.else:                                          ; preds = %if.then
  store i32 2, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then2
  br label %if.end4

if.else3:                                         ; preds = %entry
  store i32 3, ptr %x, align 4
  br label %if.end4

if.end4:                                          ; preds = %if.else3, %if.end
  %2 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multi_way_cfg(i32 noundef %sel) #0 {
entry:
  %sel.addr = alloca i32, align 4
  %result = alloca i32, align 4
  store i32 %sel, ptr %sel.addr, align 4
  %0 = load i32, ptr %sel.addr, align 4
  %cmp = icmp eq i32 %0, 0
  br i1 %cmp, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 100, ptr %result, align 4
  br label %if.end8

if.else:                                          ; preds = %entry
  %1 = load i32, ptr %sel.addr, align 4
  %cmp1 = icmp eq i32 %1, 1
  br i1 %cmp1, label %if.then2, label %if.else3

if.then2:                                         ; preds = %if.else
  store i32 200, ptr %result, align 4
  br label %if.end7

if.else3:                                         ; preds = %if.else
  %2 = load i32, ptr %sel.addr, align 4
  %cmp4 = icmp eq i32 %2, 2
  br i1 %cmp4, label %if.then5, label %if.else6

if.then5:                                         ; preds = %if.else3
  store i32 300, ptr %result, align 4
  br label %if.end

if.else6:                                         ; preds = %if.else3
  store i32 400, ptr %result, align 4
  br label %if.end

if.end:                                           ; preds = %if.else6, %if.then5
  br label %if.end7

if.end7:                                          ; preds = %if.end, %if.then2
  br label %if.end8

if.end8:                                          ; preds = %if.end7, %if.then
  %3 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %3)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @loop_dominator(i32 noundef %n) #0 {
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
  br label %for.cond, !llvm.loop !6

for.end:                                          ; preds = %for.cond
  %5 = load i32, ptr %sum, align 4
  call void @sink(i32 noundef %5)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @multiple_entry_simulation(i32 noundef %cond1, i32 noundef %cond2) #0 {
entry:
  %cond1.addr = alloca i32, align 4
  %cond2.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %cond1, ptr %cond1.addr, align 4
  store i32 %cond2, ptr %cond2.addr, align 4
  store i32 0, ptr %x, align 4
  %0 = load i32, ptr %cond1.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.end3

if.then:                                          ; preds = %entry
  store i32 1, ptr %x, align 4
  %1 = load i32, ptr %cond2.addr, align 4
  %tobool1 = icmp ne i32 %1, 0
  br i1 %tobool1, label %if.then2, label %if.end

if.then2:                                         ; preds = %if.then
  %2 = load i32, ptr %x, align 4
  %add = add nsw i32 %2, 10
  store i32 %add, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.then2, %if.then
  br label %if.end3

if.end3:                                          ; preds = %if.end, %entry
  %3 = load i32, ptr %cond2.addr, align 4
  %tobool4 = icmp ne i32 %3, 0
  br i1 %tobool4, label %if.then5, label %if.end7

if.then5:                                         ; preds = %if.end3
  %4 = load i32, ptr %x, align 4
  %add6 = add nsw i32 %4, 100
  store i32 %add6, ptr %x, align 4
  br label %if.end7

if.end7:                                          ; preds = %if.then5, %if.end3
  %5 = load i32, ptr %x, align 4
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
