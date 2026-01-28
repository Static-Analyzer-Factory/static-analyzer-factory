; ModuleID = 'clobber_query.c'
source_filename = "clobber_query.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @no_alias_clobber() #0 {
entry:
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  %p = alloca ptr, align 8
  %q = alloca ptr, align 8
  %r1 = alloca i32, align 4
  %r2 = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  store ptr %y, ptr %q, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 10, ptr %0, align 4
  %1 = load ptr, ptr %q, align 8
  store i32 20, ptr %1, align 4
  %2 = load ptr, ptr %p, align 8
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %r1, align 4
  %4 = load ptr, ptr %q, align 8
  %5 = load i32, ptr %4, align 4
  store i32 %5, ptr %r2, align 4
  %6 = load i32, ptr %r1, align 4
  call void @sink(i32 noundef %6)
  %7 = load i32, ptr %r2, align 4
  call void @sink(i32 noundef %7)
  ret void
}

declare void @sink(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @must_alias_clobber() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %q = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  store ptr %x, ptr %q, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 100, ptr %0, align 4
  %1 = load ptr, ptr %q, align 8
  %2 = load i32, ptr %1, align 4
  store i32 %2, ptr %result, align 4
  %3 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %3)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @may_alias_param(ptr noundef %p, ptr noundef %q) #0 {
entry:
  %p.addr = alloca ptr, align 8
  %q.addr = alloca ptr, align 8
  %r = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  store ptr %q, ptr %q.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  store i32 10, ptr %0, align 4
  %1 = load ptr, ptr %q.addr, align 8
  store i32 20, ptr %1, align 4
  %2 = load ptr, ptr %p.addr, align 8
  %3 = load i32, ptr %2, align 4
  store i32 %3, ptr %r, align 4
  %4 = load i32, ptr %r, align 4
  call void @sink(i32 noundef %4)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @clobber_chain() #0 {
entry:
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 1, ptr %0, align 4
  %1 = load ptr, ptr %p, align 8
  store i32 2, ptr %1, align 4
  %2 = load ptr, ptr %p, align 8
  store i32 3, ptr %2, align 4
  %3 = load ptr, ptr %p, align 8
  %4 = load i32, ptr %3, align 4
  store i32 %4, ptr %result, align 4
  %5 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %5)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @clobber_with_phi(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %x = alloca i32, align 4
  %p = alloca ptr, align 8
  %result = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  store ptr %x, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  store i32 0, ptr %0, align 4
  %1 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %1, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %2 = load ptr, ptr %p, align 8
  store i32 10, ptr %2, align 4
  br label %if.end

if.else:                                          ; preds = %entry
  %3 = load ptr, ptr %p, align 8
  store i32 20, ptr %3, align 4
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %4 = load ptr, ptr %p, align 8
  %5 = load i32, ptr %4, align 4
  store i32 %5, ptr %result, align 4
  %6 = load i32, ptr %result, align 4
  call void @sink(i32 noundef %6)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @array_clobber(i32 noundef %i, i32 noundef %j) #0 {
entry:
  %i.addr = alloca i32, align 4
  %j.addr = alloca i32, align 4
  %arr = alloca [10 x i32], align 4
  %r = alloca i32, align 4
  store i32 %i, ptr %i.addr, align 4
  store i32 %j, ptr %j.addr, align 4
  %0 = load i32, ptr %i.addr, align 4
  %idxprom = sext i32 %0 to i64
  %arrayidx = getelementptr inbounds [10 x i32], ptr %arr, i64 0, i64 %idxprom
  store i32 100, ptr %arrayidx, align 4
  %1 = load i32, ptr %j.addr, align 4
  %idxprom1 = sext i32 %1 to i64
  %arrayidx2 = getelementptr inbounds [10 x i32], ptr %arr, i64 0, i64 %idxprom1
  store i32 200, ptr %arrayidx2, align 4
  %2 = load i32, ptr %i.addr, align 4
  %idxprom3 = sext i32 %2 to i64
  %arrayidx4 = getelementptr inbounds [10 x i32], ptr %arr, i64 0, i64 %idxprom3
  %3 = load i32, ptr %arrayidx4, align 4
  store i32 %3, ptr %r, align 4
  %4 = load i32, ptr %r, align 4
  call void @sink(i32 noundef %4)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @no_clobber_live_on_entry(ptr noundef %external_ptr) #0 {
entry:
  %external_ptr.addr = alloca ptr, align 8
  %val = alloca i32, align 4
  store ptr %external_ptr, ptr %external_ptr.addr, align 8
  %0 = load ptr, ptr %external_ptr.addr, align 8
  %1 = load i32, ptr %0, align 4
  store i32 %1, ptr %val, align 4
  %2 = load i32, ptr %val, align 4
  call void @sink(i32 noundef %2)
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
