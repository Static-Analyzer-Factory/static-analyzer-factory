; ModuleID = 'phi_placement_diamond.c'
source_filename = "phi_placement_diamond.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @diamond_phi() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  %call = call i32 @condition()
  %tobool = icmp ne i32 %call, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %0 = load ptr, ptr %p, align 8
  store i32 10, ptr %0, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  %1 = load ptr, ptr %p, align 8
  store i32 20, ptr %1, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %2 = load ptr, ptr %p, align 8
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %result, align 4
  %4 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %4)
  ret void
}

declare i32 @condition() #1

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @simple_diamond(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %val = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 1, ptr %val, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  store i32 2, ptr %val, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %1 = load i32, ptr %val, align 4
  call void @sink(i32 noundef %1)
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
