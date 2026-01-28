; ModuleID = 'svfg_memory_phi.c'
source_filename = "svfg_memory_phi.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @simple_mem_phi(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p, align 8
  store i32 10, ptr %1, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  %2 = load ptr, ptr %p, align 8
  store i32 20, ptr %2, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %result, align 4
  %5 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %5)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @nested_mem_phi(i32 noundef %c1, i32 noundef %c2) #0 {
entry:
  %c1.addr = alloca i32, align 4
  %c2.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store i32 %c1, ptr %c1.addr, align 4
  store i32 %c2, ptr %c2.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load i32, ptr %c1.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else3

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %c2.addr, align 4
  %tobool1 = icmp ne i32 %1, 0
  br i1 %tobool1, label %if.then2, label %if.else

if.then2:                                         ; preds = %if.then
  %2 = load ptr, ptr %p, align 8
  store i32 1, ptr %2, align 4
  br label %if.end

if.else:                                          ; preds = %if.then
  %3 = load ptr, ptr %p, align 8
  store i32 2, ptr %3, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then2
  br label %if.end4

if.else3:                                         ; preds = %entry
  %4 = load ptr, ptr %p, align 8
  store i32 3, ptr %4, align 4
  br label %if.end4

if.end4:                                          ; preds = %if.else3, %if.end
  %5 = load ptr, ptr %p, align 8
  %6 = load i32, ptr %5, align 4
  store i32 %6, ptr %result, align 4
  %7 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %7)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @loop_mem_phi(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %i = alloca i32, align 4
  %cur = alloca i32, align 4
  %result = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 0, ptr %0, align 4
  store i32 0, ptr %i, align 4
  br label %for.cond

for.cond:                                         ; preds = %for.inc, %entry
  %1 = load i32, ptr %i, align 4
  %2 = load i32, ptr %n.addr, align 4
  %cmp = icmp slt i32 %1, %2
  br i1 %cmp, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %cur, align 4
  %5 = load i32, ptr %cur, align 4
  %add = add nsw i32 %5, 1
  %6 = load ptr, ptr %p, align 8
  store i32 %add, ptr %6, align 4
  br label %for.inc

for.inc:                                          ; preds = %for.body
  %7 = load i32, ptr %i, align 4
  %inc = add nsw i32 %7, 1
  store i32 %inc, ptr %i, align 4
  br label %for.cond, !llvm.loop !6

for.end:                                          ; preds = %for.cond
  %8 = load ptr, ptr %p, align 8
  %9 = load i32, ptr %8, align 4
  store i32 %9, ptr %result, align 4
  %10 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @sequential_mem_phi(i32 noundef %c1, i32 noundef %c2) #0 {
entry:
  %c1.addr = alloca i32, align 4
  %c2.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %intermediate = alloca i32, align 4
  %final_val = alloca i32, align 4
  store i32 %c1, ptr %c1.addr, align 4
  store i32 %c2, ptr %c2.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load i32, ptr %c1.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p, align 8
  store i32 100, ptr %1, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  %2 = load ptr, ptr %p, align 8
  store i32 200, ptr %2, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %intermediate, align 4
  %5 = load i32, ptr %c2.addr, align 4
  %tobool1 = icmp ne i32 %5, 0
  br i1 %tobool1, label %if.then2, label %if.else3

if.then2:                                         ; preds = %if.end
  %6 = load ptr, ptr %p, align 8
  store i32 300, ptr %6, align 4
  br label %if.end4

if.else3:                                         ; preds = %if.end
  %7 = load ptr, ptr %p, align 8
  store i32 400, ptr %7, align 4
  br label %if.end4

if.end4:                                          ; preds = %if.else3, %if.then2
  %8 = load ptr, ptr %p, align 8
  %9 = load i32, ptr %8, align 4
  store i32 %9, ptr %final_val, align 4
  %10 = load i32, ptr %intermediate, align 4
  call void @sink(i32 noundef %10)
  %11 = load i32, ptr %final_val, align 4
  call void @sink(i32 noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @phi_with_live_on_entry(ptr noundef %external, i32 noundef %cond) #0 {
entry:
  %external.addr = alloca ptr, align 8
  %cond.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %external, ptr %external.addr, align 8
  store i32 %cond, ptr %cond.addr, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %external.addr, align 8
  store ptr %1, ptr %p, align 8
  br label %if.end

if.else:                                          ; preds = %entry
  store ptr %x, ptr %p, align 8
  %2 = load ptr, ptr %p, align 8
  store i32 50, ptr %2, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %result, align 4
  %5 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %5)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @three_way_mem_phi(i32 noundef %sel) #0 {
entry:
  %sel.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store i32 %sel, ptr %sel.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load i32, ptr %sel.addr, align 4
  %cmp = icmp eq i32 %0, 0
  br i1 %cmp, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p, align 8
  store i32 1, ptr %1, align 4
  br label %if.end4

if.else:                                          ; preds = %entry
  %2 = load i32, ptr %sel.addr, align 4
  %cmp1 = icmp eq i32 %2, 1
  br i1 %cmp1, label %if.then2, label %if.else3

if.then2:                                         ; preds = %if.else
  %3 = load ptr, ptr %p, align 8
  store i32 2, ptr %3, align 4
  br label %if.end

if.else3:                                         ; preds = %if.else
  %4 = load ptr, ptr %p, align 8
  store i32 3, ptr %4, align 4
  br label %if.end

if.end:                                           ; preds = %if.else3, %if.then2
  br label %if.end4

if.end4:                                          ; preds = %if.end, %if.then
  %5 = load ptr, ptr %p, align 8
  %6 = load i32, ptr %5, align 4
  store i32 %6, ptr %result, align 4
  %7 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %7)
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
