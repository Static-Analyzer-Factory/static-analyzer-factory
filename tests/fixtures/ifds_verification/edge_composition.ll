; ModuleID = 'edge_composition.c'
source_filename = "edge_composition.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"test.txt\00", align 1
@.str.1 = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1
@.str.2 = private unnamed_addr constant [6 x i8] c"INPUT\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"constant\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"SECRET\00", align 1
@.str.5 = private unnamed_addr constant [5 x i8] c"DATA\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_file_open_close() #0 {
entry:
  %fd = alloca i32, align 4
  %call = call i32 @open(ptr noundef @.str, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %fd, align 4
  %call1 = call i32 @close(i32 noundef %0)
  ret void
}

declare i32 @open(ptr noundef, i32 noundef) #1

declare i32 @close(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_use_after_close() #0 {
entry:
  %fd = alloca i32, align 4
  %buf = alloca [100 x i8], align 1
  %call = call i32 @open(ptr noundef @.str.1, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %fd, align 4
  %call1 = call i32 @close(i32 noundef %0)
  %1 = load i32, ptr %fd, align 4
  %arraydecay = getelementptr inbounds [100 x i8], ptr %buf, i64 0, i64 0
  %call2 = call i32 @read(i32 noundef %1, ptr noundef %arraydecay, i32 noundef 100)
  ret void
}

declare i32 @read(i32 noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @transform1(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  ret ptr %0
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @transform2(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  ret ptr %0
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_composition_chain() #0 {
entry:
  %data = alloca ptr, align 8
  %t1 = alloca ptr, align 8
  %t2 = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.2)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @transform1(ptr noundef %0)
  store ptr %call1, ptr %t1, align 8
  %1 = load ptr, ptr %t1, align 8
  %call2 = call ptr @transform2(ptr noundef %1)
  store ptr %call2, ptr %t2, align 8
  %2 = load ptr, ptr %t2, align 8
  call void @sink(ptr noundef %2)
  ret void
}

declare ptr @getenv(ptr noundef) #1

declare void @sink(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @kill_taint(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  ret ptr @.str.3
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_composition_with_kill() #0 {
entry:
  %data = alloca ptr, align 8
  %killed = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.4)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @kill_taint(ptr noundef %0)
  store ptr %call1, ptr %killed, align 8
  %1 = load ptr, ptr %killed, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_branch_composition(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  store i32 %cond, ptr %cond.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str.5)
  store ptr %call, ptr %data, align 8
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %data, align 8
  %call1 = call ptr @transform1(ptr noundef %1)
  store ptr %call1, ptr %result, align 8
  br label %if.end

if.else:                                          ; preds = %entry
  %2 = load ptr, ptr %data, align 8
  %call2 = call ptr @transform2(ptr noundef %2)
  store ptr %call2, ptr %result, align 8
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %3)
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
