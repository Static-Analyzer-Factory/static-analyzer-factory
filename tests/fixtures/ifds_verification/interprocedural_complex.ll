; ModuleID = 'interprocedural_complex.c'
source_filename = "interprocedural_complex.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Data = type { ptr, ptr }

@.str = private unnamed_addr constant [10 x i8] c"RECURSIVE\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"MUTUAL\00", align 1
@.str.2 = private unnamed_addr constant [8 x i8] c"TAINTED\00", align 1
@.str.3 = private unnamed_addr constant [6 x i8] c"clean\00", align 1
@.str.4 = private unnamed_addr constant [6 x i8] c"ALIAS\00", align 1
@.str.5 = private unnamed_addr constant [7 x i8] c"STRUCT\00", align 1
@.str.6 = private unnamed_addr constant [2 x i8] c"A\00", align 1
@.str.7 = private unnamed_addr constant [2 x i8] c"B\00", align 1
@.str.8 = private unnamed_addr constant [8 x i8] c"DIAMOND\00", align 1
@.str.9 = private unnamed_addr constant [6 x i8] c"EARLY\00", align 1
@.str.10 = private unnamed_addr constant [8 x i8] c"default\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @recursive_pass(ptr noundef %p, i32 noundef %depth) #0 {
entry:
  %retval = alloca ptr, align 8
  %p.addr = alloca ptr, align 8
  %depth.addr = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  store i32 %depth, ptr %depth.addr, align 4
  %0 = load i32, ptr %depth.addr, align 4
  %cmp = icmp sle i32 %0, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p.addr, align 8
  store ptr %1, ptr %retval, align 8
  br label %return

if.end:                                           ; preds = %entry
  %2 = load ptr, ptr %p.addr, align 8
  %3 = load i32, ptr %depth.addr, align 4
  %sub = sub nsw i32 %3, 1
  %call = call ptr @recursive_pass(ptr noundef %2, i32 noundef %sub)
  store ptr %call, ptr %retval, align 8
  br label %return

return:                                           ; preds = %if.end, %if.then
  %4 = load ptr, ptr %retval, align 8
  ret ptr %4
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_recursive_taint() #0 {
entry:
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @recursive_pass(ptr noundef %0, i32 noundef 5)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
  ret void
}

declare ptr @getenv(ptr noundef) #1

declare void @sink(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @ping(ptr noundef %p, i32 noundef %n) #0 {
entry:
  %retval = alloca ptr, align 8
  %p.addr = alloca ptr, align 8
  %n.addr = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr %n.addr, align 4
  %cmp = icmp sle i32 %0, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p.addr, align 8
  store ptr %1, ptr %retval, align 8
  br label %return

if.end:                                           ; preds = %entry
  %2 = load ptr, ptr %p.addr, align 8
  %3 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %3, 1
  %call = call ptr @pong(ptr noundef %2, i32 noundef %sub)
  store ptr %call, ptr %retval, align 8
  br label %return

return:                                           ; preds = %if.end, %if.then
  %4 = load ptr, ptr %retval, align 8
  ret ptr %4
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @pong(ptr noundef %p, i32 noundef %n) #0 {
entry:
  %retval = alloca ptr, align 8
  %p.addr = alloca ptr, align 8
  %n.addr = alloca i32, align 4
  store ptr %p, ptr %p.addr, align 8
  store i32 %n, ptr %n.addr, align 4
  %0 = load i32, ptr %n.addr, align 4
  %cmp = icmp sle i32 %0, 0
  br i1 %cmp, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %p.addr, align 8
  store ptr %1, ptr %retval, align 8
  br label %return

if.end:                                           ; preds = %entry
  %2 = load ptr, ptr %p.addr, align 8
  %3 = load i32, ptr %n.addr, align 4
  %sub = sub nsw i32 %3, 1
  %call = call ptr @ping(ptr noundef %2, i32 noundef %sub)
  store ptr %call, ptr %retval, align 8
  br label %return

return:                                           ; preds = %if.end, %if.then
  %4 = load ptr, ptr %retval, align 8
  ret ptr %4
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_mutual_recursion() #0 {
entry:
  %data = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.1)
  store ptr %call, ptr %data, align 8
  %0 = load ptr, ptr %data, align 8
  %call1 = call ptr @ping(ptr noundef %0, i32 noundef 4)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @maybe_taint(i32 noundef %cond) #0 {
entry:
  %retval = alloca ptr, align 8
  %cond.addr = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %call = call ptr @getenv(ptr noundef @.str.2)
  store ptr %call, ptr %retval, align 8
  br label %return

if.else:                                          ; preds = %entry
  store ptr @.str.3, ptr %retval, align 8
  br label %return

return:                                           ; preds = %if.else, %if.then
  %1 = load ptr, ptr %retval, align 8
  ret ptr %1
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_multiple_returns() #0 {
entry:
  %result = alloca ptr, align 8
  %call = call ptr @maybe_taint(i32 noundef 1)
  store ptr %call, ptr %result, align 8
  %0 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @alias_sink(ptr noundef %pp) #0 {
entry:
  %pp.addr = alloca ptr, align 8
  store ptr %pp, ptr %pp.addr, align 8
  %0 = load ptr, ptr %pp.addr, align 8
  %1 = load ptr, ptr %0, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_pointer_alias_taint() #0 {
entry:
  %data = alloca ptr, align 8
  %ptr = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.4)
  store ptr %call, ptr %data, align 8
  store ptr %data, ptr %ptr, align 8
  %0 = load ptr, ptr %ptr, align 8
  call void @alias_sink(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @set_field(ptr noundef %d, ptr noundef %val) #0 {
entry:
  %d.addr = alloca ptr, align 8
  %val.addr = alloca ptr, align 8
  store ptr %d, ptr %d.addr, align 8
  store ptr %val, ptr %val.addr, align 8
  %0 = load ptr, ptr %val.addr, align 8
  %1 = load ptr, ptr %d.addr, align 8
  %field1 = getelementptr inbounds %struct.Data, ptr %1, i32 0, i32 0
  store ptr %0, ptr %field1, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @get_field(ptr noundef %d) #0 {
entry:
  %d.addr = alloca ptr, align 8
  store ptr %d, ptr %d.addr, align 8
  %0 = load ptr, ptr %d.addr, align 8
  %field1 = getelementptr inbounds %struct.Data, ptr %0, i32 0, i32 0
  %1 = load ptr, ptr %field1, align 8
  ret ptr %1
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_struct_taint_flow() #0 {
entry:
  %d = alloca %struct.Data, align 8
  %tainted = alloca ptr, align 8
  %result = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.5)
  store ptr %call, ptr %tainted, align 8
  %0 = load ptr, ptr %tainted, align 8
  call void @set_field(ptr noundef %d, ptr noundef %0)
  %call1 = call ptr @get_field(ptr noundef %d)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @inner_sink(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  call void @sink(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @middle(ptr noundef %p) #0 {
entry:
  %p.addr = alloca ptr, align 8
  store ptr %p, ptr %p.addr, align 8
  %0 = load ptr, ptr %p.addr, align 8
  call void @inner_sink(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @outer_a() #0 {
entry:
  %a = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.6)
  store ptr %call, ptr %a, align 8
  %0 = load ptr, ptr %a, align 8
  call void @middle(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @outer_b() #0 {
entry:
  %b = alloca ptr, align 8
  %call = call ptr @getenv(ptr noundef @.str.7)
  store ptr %call, ptr %b, align 8
  %0 = load ptr, ptr %b, align 8
  call void @middle(ptr noundef %0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_diamond_flow(i32 noundef %c1, i32 noundef %c2) #0 {
entry:
  %c1.addr = alloca i32, align 4
  %c2.addr = alloca i32, align 4
  %data = alloca ptr, align 8
  store i32 %c1, ptr %c1.addr, align 4
  store i32 %c2, ptr %c2.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str.8)
  store ptr %call, ptr %data, align 8
  %0 = load i32, ptr %c1.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else3

if.then:                                          ; preds = %entry
  %1 = load i32, ptr %c2.addr, align 4
  %tobool1 = icmp ne i32 %1, 0
  br i1 %tobool1, label %if.then2, label %if.else

if.then2:                                         ; preds = %if.then
  br label %if.end

if.else:                                          ; preds = %if.then
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then2
  br label %if.end4

if.else3:                                         ; preds = %entry
  br label %if.end4

if.end4:                                          ; preds = %if.else3, %if.end
  %2 = load ptr, ptr %data, align 8
  call void @sink(ptr noundef %2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @early_return(i32 noundef %cond) #0 {
entry:
  %retval = alloca ptr, align 8
  %cond.addr = alloca i32, align 4
  %data = alloca ptr, align 8
  store i32 %cond, ptr %cond.addr, align 4
  %call = call ptr @getenv(ptr noundef @.str.9)
  store ptr %call, ptr %data, align 8
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %1 = load ptr, ptr %data, align 8
  store ptr %1, ptr %retval, align 8
  br label %return

if.end:                                           ; preds = %entry
  store ptr @.str.10, ptr %retval, align 8
  br label %return

return:                                           ; preds = %if.end, %if.then
  %2 = load ptr, ptr %retval, align 8
  ret ptr %2
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_early_return() #0 {
entry:
  %result = alloca ptr, align 8
  %call = call ptr @early_return(i32 noundef 1)
  store ptr %call, ptr %result, align 8
  %0 = load ptr, ptr %result, align 8
  call void @sink(ptr noundef %0)
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
