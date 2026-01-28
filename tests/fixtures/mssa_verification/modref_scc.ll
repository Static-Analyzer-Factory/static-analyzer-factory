; ModuleID = 'modref_scc.c'
source_filename = "modref_scc.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@global_x = dso_local global i32 0, align 4
@global_y = dso_local global i32 0, align 4

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @self_recursive(i32 noundef %n, ptr noundef %p) #0 {
entry:
  %n.addr = alloca i32, align 4
  %p.addr = alloca ptr, align 8
  store i32 %n, ptr %n.addr, align 4
  store ptr %p, ptr %p.addr, align 8
  %0 = load i32, ptr %n.addr, align 4
  %cmp = icmp sle i32 %0, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p.addr, align 8
  store i32 0, ptr %1, align 4
  br label %return

if.end:                                           ; preds = %entry
  %2 = load i32, ptr %n.addr, align 4
  %3 = load ptr, ptr %p.addr, align 8
  store i32 %2, ptr %3, align 4
  %4 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %4, 1
  %5 = load ptr, ptr %p.addr, align 8
  call void @self_recursive(i32 noundef %sub, ptr noundef %5)
  br label %return

return:                                           ; preds = %if.end, %if.then
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @ping(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr %n.addr, align 4
  store i32 %0, ptr @global_x, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp sgt i32 %1, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %2 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %2, 1
  call void @pong(i32 noundef %sub)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @pong(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %tmp = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr @global_x, align 4
  store i32 %0, ptr %tmp, align 4
  %1 = load i32, ptr %tmp, align 4
  store i32 %1, ptr @global_y, align 4
  %2 = load i32, ptr %n.addr, align 4
  %cmp = icmp sgt i32 %2, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %3 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %3, 1
  call void @ping(i32 noundef %sub)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @func_a(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr %n.addr, align 4
  store i32 %0, ptr @global_x, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp sgt i32 %1, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %2 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %2, 1
  call void @func_b(i32 noundef %sub)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @func_b(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr %n.addr, align 4
  store i32 %0, ptr @global_y, align 4
  %1 = load i32, ptr %n.addr, align 4
  %cmp = icmp sgt i32 %1, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %2 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %2, 1
  call void @func_c(i32 noundef %sub)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @func_c(i32 noundef %n) #0 {
entry:
  %n.addr = alloca i32, align 4
  %sum = alloca i32, align 4
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr @global_x, align 4
  %1 = load i32, ptr @global_y, align 4
  %add = add nsw i32 %0, %1
  store i32 %add, ptr %sum, align 4
  %2 = load i32, ptr %n.addr, align 4
  %cmp = icmp sgt i32 %2, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %3 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %3, 1
  call void @func_a(i32 noundef %sub)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  %4 = load i32, ptr %sum, align 4
  call void @sink(i32 noundef %4)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @leaf_modifier(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  store i32 42, ptr %0, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @transitive_caller(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  call void @leaf_modifier(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @modref_test_entry() #0 {
entry:
  %local = alloca i32, align 4
  %p = alloca ptr, align 8
  store i32 0, ptr %local, align 4
  store ptr %local, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  call void @self_recursive(i32 noundef 5, ptr noundef %0)
  %1 = load ptr, ptr %p, align 8
  %2 = load i32, ptr %1, align 4
  call void @sink(i32 noundef %2)
  call void @ping(i32 noundef 3)
  %3 = load i32, ptr @global_x, align 4
  call void @sink(i32 noundef %3)
  %4 = load i32, ptr @global_y, align 4
  call void @sink(i32 noundef %4)
  %5 = load ptr, ptr %p, align 8
  call void @transitive_caller(ptr noundef %5)
  %6 = load ptr, ptr %p, align 8
  %7 = load i32, ptr %6, align 4
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
