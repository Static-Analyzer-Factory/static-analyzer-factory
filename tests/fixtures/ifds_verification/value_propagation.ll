; ModuleID = 'value_propagation.c'
source_filename = "value_propagation.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"file.txt\00", align 1
@.str.1 = private unnamed_addr constant [9 x i8] c"test.txt\00", align 1
@.str.2 = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1
@.str.3 = private unnamed_addr constant [14 x i8] c"important.txt\00", align 1
@.str.4 = private unnamed_addr constant [11 x i8] c"leaked.txt\00", align 1
@.str.5 = private unnamed_addr constant [12 x i8] c"correct.txt\00", align 1
@.str.6 = private unnamed_addr constant [5 x i8] c"data\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_basic_typestate() #0 {
entry:
  %fd = alloca i32, align 4
  %buf = alloca [64 x i8], align 1
  %call = call i32 @open(ptr noundef @.str, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %fd, align 4
  %arraydecay = getelementptr inbounds [64 x i8], ptr %buf, i64 0, i64 0
  %call1 = call i32 @read(i32 noundef %0, ptr noundef %arraydecay, i32 noundef 64)
  %1 = load i32, ptr %fd, align 4
  %call2 = call i32 @close(i32 noundef %1)
  ret void
}

declare i32 @open(ptr noundef, i32 noundef) #1

declare i32 @read(i32 noundef, ptr noundef, i32 noundef) #1

declare i32 @close(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_value_join_at_merge(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %fd = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %call = call i32 @open(ptr noundef @.str.1, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %fd, align 4
  %call1 = call i32 @close(i32 noundef %1)
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @helper_close(i32 noundef %fd) #0 {
entry:
  %fd.addr = alloca i32, align 4
  store i32 %fd, ptr %fd.addr, align 4
  %0 = load i32, ptr %fd.addr, align 4
  %call = call i32 @close(i32 noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_interproc_value_propagation() #0 {
entry:
  %fd = alloca i32, align 4
  %call = call i32 @open(ptr noundef @.str.2, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %fd, align 4
  call void @helper_close(i32 noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_double_close() #0 {
entry:
  %fd = alloca i32, align 4
  %call = call i32 @open(ptr noundef @.str.3, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %fd, align 4
  %call1 = call i32 @close(i32 noundef %0)
  %1 = load i32, ptr %fd, align 4
  %call2 = call i32 @close(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_file_leak(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %fd = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %call = call i32 @open(ptr noundef @.str.4, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  br label %return

if.end:                                           ; preds = %entry
  %1 = load i32, ptr %fd, align 4
  %call1 = call i32 @close(i32 noundef %1)
  br label %return

return:                                           ; preds = %if.end, %if.then
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_correct_usage(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %fd = alloca i32, align 4
  %buf = alloca [32 x i8], align 1
  store i32 %cond, ptr %cond.addr, align 4
  %call = call i32 @open(ptr noundef @.str.5, i32 noundef 0)
  store i32 %call, ptr %fd, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %fd, align 4
  %arraydecay = getelementptr inbounds [32 x i8], ptr %buf, i64 0, i64 0
  %call1 = call i32 @read(i32 noundef %1, ptr noundef %arraydecay, i32 noundef 32)
  br label %if.end

if.else:                                          ; preds = %entry
  %2 = load i32, ptr %fd, align 4
  %call2 = call i32 @write(i32 noundef %2, ptr noundef @.str.6, i32 noundef 4)
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %3 = load i32, ptr %fd, align 4
  %call3 = call i32 @close(i32 noundef %3)
  ret void
}

declare i32 @write(i32 noundef, ptr noundef, i32 noundef) #1

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
