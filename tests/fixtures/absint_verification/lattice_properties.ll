; ModuleID = 'lattice_properties.c'
source_filename = "lattice_properties.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @join_merge(i32 noundef %cond) #0 {
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

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @join_three_way(i32 noundef %mode) #0 {
entry:
  %mode.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %mode, ptr %mode.addr, align 4
  %0 = load i32, ptr %mode.addr, align 4
  %cmp = icmp eq i32 %0, 0
  br i1 %cmp, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 5, ptr %x, align 4
  br label %if.end4

if.else:                                          ; preds = %entry
  %1 = load i32, ptr %mode.addr, align 4
  %cmp1 = icmp eq i32 %1, 1
  br i1 %cmp1, label %if.then2, label %if.else3

if.then2:                                         ; preds = %if.else
  store i32 15, ptr %x, align 4
  br label %if.end

if.else3:                                         ; preds = %if.else
  store i32 25, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else3, %if.then2
  br label %if.end4

if.end4:                                          ; preds = %if.end, %if.then
  %2 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @join_idempotent(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 42, ptr %x, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  store i32 42, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %1 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @meet_branch(i32 noundef %x) #0 {
entry:
  %x.addr = alloca i32, align 4
  store i32 %x, ptr %x.addr, align 4
  %0 = load i32, ptr %x.addr, align 4
  %cmp = icmp sge i32 %0, 0
  br i1 %cmp, label %if.then, label %if.end3

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %x.addr, align 4
  %cmp1 = icmp sle i32 %1, 100
  br i1 %cmp1, label %if.then2, label %if.end

if.then2:                                         ; preds = %if.then
  %2 = load i32, ptr %x.addr, align 4
  call void @sink(i32 noundef %2)
  br label %if.end

if.end:                                           ; preds = %if.then2, %if.then
  br label %if.end3

if.end3:                                          ; preds = %if.end, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @bottom_absorption(i32 noundef %x) #0 {
entry:
  %x.addr = alloca i32, align 4
  store i32 %x, ptr %x.addr, align 4
  %0 = load i32, ptr %x.addr, align 4
  %cmp = icmp sgt i32 %0, 0
  br i1 %cmp, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %1 = load i32, ptr %x.addr, align 4
  %cmp1 = icmp slt i32 %1, 0
  br i1 %cmp1, label %if.then, label %if.end

if.then:                                          ; preds = %land.lhs.true
  %2 = load i32, ptr %x.addr, align 4
  call void @sink(i32 noundef %2)
  br label %if.end

if.end:                                           ; preds = %if.then, %land.lhs.true, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @leq_subset(i32 noundef %mode) #0 {
entry:
  %mode.addr = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 %mode, ptr %mode.addr, align 4
  %0 = load i32, ptr %mode.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  store i32 5, ptr %x, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  store i32 3, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %1 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @top_bottom_identity(ptr noundef %ptr) #0 {
entry:
  %ptr.addr = alloca ptr, align 8
  %x = alloca i32, align 4
  store ptr %ptr, ptr %ptr.addr, align 8
  %0 = load ptr, ptr %ptr.addr, align 8
  %tobool = icmp ne ptr %0, null
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %ptr.addr, align 8
  %2 = load i32, ptr %1, align 4
  store i32 %2, ptr %x, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  store i32 0, ptr %x, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load i32, ptr %x, align 4
  call void @sink(i32 noundef %3)
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
