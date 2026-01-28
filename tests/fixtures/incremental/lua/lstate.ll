; ModuleID = 'lstate.c'
source_filename = "lstate.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.1, i16, i16 }
%union.anon = type { %struct.anon.0 }
%struct.anon.0 = type { ptr, i64, i64 }
%union.anon.1 = type { i32 }
%struct.anon = type { ptr, i32, i32 }
%union.StackValue = type { %struct.TValue }
%struct.LG = type { %struct.LX, %struct.global_State }
%struct.LX = type { [8 x i8], %struct.lua_State }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.3, [1 x i8] }
%union.anon.3 = type { i64 }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }

@.str = private unnamed_addr constant [17 x i8] c"C stack overflow\00", align 1
@.str.1 = private unnamed_addr constant [29 x i8] c"error object is not a string\00", align 1
@.str.2 = private unnamed_addr constant [10 x i8] c"error in \00", align 1
@.str.3 = private unnamed_addr constant [3 x i8] c" (\00", align 1
@.str.4 = private unnamed_addr constant [2 x i8] c")\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_setdebt(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.global_State, ptr %6, i32 0, i32 2
  %8 = load i64, ptr %7, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 3
  %11 = load i64, ptr %10, align 8
  %12 = add nsw i64 %8, %11
  store i64 %12, ptr %5, align 8
  %13 = load i64, ptr %4, align 8
  %14 = load i64, ptr %5, align 8
  %15 = sub nsw i64 %14, 9223372036854775807
  %16 = icmp slt i64 %13, %15
  br i1 %16, label %17, label %20

17:                                               ; preds = %2
  %18 = load i64, ptr %5, align 8
  %19 = sub nsw i64 %18, 9223372036854775807
  store i64 %19, ptr %4, align 8
  br label %20

20:                                               ; preds = %17, %2
  %21 = load i64, ptr %5, align 8
  %22 = load i64, ptr %4, align 8
  %23 = sub nsw i64 %21, %22
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.global_State, ptr %24, i32 0, i32 2
  store i64 %23, ptr %25, align 8
  %26 = load i64, ptr %4, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 3
  store i64 %26, ptr %28, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_setcstacklimit(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  ret i32 200
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaE_extendCI(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaM_malloc_(ptr noundef %4, i64 noundef 64, i32 noundef 0)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 8
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.CallInfo, ptr %9, i32 0, i32 3
  store ptr %6, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 8
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 2
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.CallInfo, ptr %16, i32 0, i32 3
  store ptr null, ptr %17, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.CallInfo, ptr %18, i32 0, i32 4
  %20 = getelementptr inbounds %struct.anon, ptr %19, i32 0, i32 1
  store volatile i32 0, ptr %20, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 5
  %23 = load i16, ptr %22, align 4
  %24 = add i16 %23, 1
  store i16 %24, ptr %22, align 4
  %25 = load ptr, ptr %3, align 8
  ret ptr %25
}

declare hidden ptr @luaM_malloc_(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_shrinkCI(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 8
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 3
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %3, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %1
  br label %42

14:                                               ; preds = %1
  br label %15

15:                                               ; preds = %41, %14
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.CallInfo, ptr %16, i32 0, i32 3
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %4, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %20, label %42

20:                                               ; preds = %15
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.CallInfo, ptr %21, i32 0, i32 3
  %23 = load ptr, ptr %22, align 8
  store ptr %23, ptr %5, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.CallInfo, ptr %25, i32 0, i32 3
  store ptr %24, ptr %26, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 5
  %29 = load i16, ptr %28, align 4
  %30 = add i16 %29, -1
  store i16 %30, ptr %28, align 4
  %31 = load ptr, ptr %2, align 8
  %32 = load ptr, ptr %4, align 8
  call void @luaM_free_(ptr noundef %31, ptr noundef %32, i64 noundef 64)
  %33 = load ptr, ptr %5, align 8
  %34 = icmp eq ptr %33, null
  br i1 %34, label %35, label %36

35:                                               ; preds = %20
  br label %42

36:                                               ; preds = %20
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.CallInfo, ptr %38, i32 0, i32 2
  store ptr %37, ptr %39, align 8
  %40 = load ptr, ptr %5, align 8
  store ptr %40, ptr %3, align 8
  br label %41

41:                                               ; preds = %36
  br label %15, !llvm.loop !6

42:                                               ; preds = %13, %35, %15
  ret void
}

declare hidden void @luaM_free_(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_checkcstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 19
  %5 = load i32, ptr %4, align 8
  %6 = and i32 %5, 65535
  %7 = icmp eq i32 %6, 200
  br i1 %7, label %8, label %10

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %9, ptr noundef @.str) #5
  unreachable

10:                                               ; preds = %1
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 19
  %13 = load i32, ptr %12, align 8
  %14 = and i32 %13, 65535
  %15 = icmp uge i32 %14, 220
  br i1 %15, label %16, label %18

16:                                               ; preds = %10
  %17 = load ptr, ptr %2, align 8
  call void @luaD_throw(ptr noundef %17, i32 noundef 5) #5
  unreachable

18:                                               ; preds = %10
  br label %19

19:                                               ; preds = %18
  ret void
}

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #2

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_incCstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 19
  %5 = load i32, ptr %4, align 8
  %6 = add i32 %5, 1
  store i32 %6, ptr %4, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 19
  %9 = load i32, ptr %8, align 8
  %10 = and i32 %9, 65535
  %11 = icmp uge i32 %10, 200
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %19

17:                                               ; preds = %1
  %18 = load ptr, ptr %2, align 8
  call void @luaE_checkcstack(ptr noundef %18)
  br label %19

19:                                               ; preds = %17, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_newthread(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %3, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 7
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 3
  %15 = load i64, ptr %14, align 8
  %16 = icmp sgt i64 %15, 0
  br i1 %16, label %17, label %19

17:                                               ; preds = %1
  %18 = load ptr, ptr %2, align 8
  call void @luaC_step(ptr noundef %18)
  br label %19

19:                                               ; preds = %17, %1
  %20 = load ptr, ptr %2, align 8
  %21 = call ptr @luaC_newobjdt(ptr noundef %20, i32 noundef 8, i64 noundef 208, i64 noundef 8)
  store ptr %21, ptr %4, align 8
  %22 = load ptr, ptr %4, align 8
  store ptr %22, ptr %5, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  store ptr %25, ptr %6, align 8
  %26 = load ptr, ptr %5, align 8
  store ptr %26, ptr %7, align 8
  %27 = load ptr, ptr %7, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  store ptr %27, ptr %29, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 72, ptr %31, align 8
  %32 = load ptr, ptr %2, align 8
  %33 = load ptr, ptr %2, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 6
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i32 1
  store ptr %36, ptr %34, align 8
  %37 = load ptr, ptr %2, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = load ptr, ptr %3, align 8
  call void @preinit_thread(ptr noundef %38, ptr noundef %39)
  %40 = load ptr, ptr %2, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 23
  %42 = load volatile i32, ptr %41, align 8
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 23
  store volatile i32 %42, ptr %44, align 8
  %45 = load ptr, ptr %2, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 21
  %47 = load i32, ptr %46, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.lua_State, ptr %48, i32 0, i32 21
  store i32 %47, ptr %49, align 8
  %50 = load ptr, ptr %2, align 8
  %51 = getelementptr inbounds %struct.lua_State, ptr %50, i32 0, i32 17
  %52 = load volatile ptr, ptr %51, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 17
  store volatile ptr %52, ptr %54, align 8
  %55 = load ptr, ptr %5, align 8
  %56 = getelementptr inbounds %struct.lua_State, ptr %55, i32 0, i32 21
  %57 = load i32, ptr %56, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.lua_State, ptr %58, i32 0, i32 22
  store i32 %57, ptr %59, align 4
  %60 = load ptr, ptr %5, align 8
  %61 = getelementptr inbounds i8, ptr %60, i64 -8
  %62 = load ptr, ptr %3, align 8
  %63 = getelementptr inbounds %struct.global_State, ptr %62, i32 0, i32 40
  %64 = load ptr, ptr %63, align 8
  %65 = getelementptr inbounds i8, ptr %64, i64 -8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %61, ptr align 1 %65, i64 8, i1 false)
  %66 = load ptr, ptr %2, align 8
  %67 = load ptr, ptr %5, align 8
  %68 = load ptr, ptr %2, align 8
  call void @stack_init(ptr noundef %67, ptr noundef %68)
  %69 = load ptr, ptr %5, align 8
  ret ptr %69
}

declare hidden void @luaC_step(ptr noundef) #1

declare hidden ptr @luaC_newobjdt(ptr noundef, i32 noundef, i64 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @preinit_thread(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  store ptr %5, ptr %7, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 10
  store ptr null, ptr %9, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 8
  store ptr null, ptr %11, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 5
  store i16 0, ptr %13, align 4
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 14
  store ptr %14, ptr %16, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 19
  store i32 0, ptr %18, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 15
  store ptr null, ptr %20, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 17
  store volatile ptr null, ptr %22, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 23
  store volatile i32 0, ptr %24, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 21
  store i32 0, ptr %26, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 4
  store i8 1, ptr %28, align 1
  %29 = load ptr, ptr %3, align 8
  %30 = getelementptr inbounds %struct.lua_State, ptr %29, i32 0, i32 21
  %31 = load i32, ptr %30, align 8
  %32 = load ptr, ptr %3, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 22
  store i32 %31, ptr %33, align 4
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 11
  store ptr null, ptr %35, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 3
  store i8 0, ptr %37, align 2
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 18
  store i64 0, ptr %39, align 8
  %40 = load ptr, ptr %3, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 20
  store i32 0, ptr %41, align 4
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @stack_init(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call ptr @luaM_malloc_(ptr noundef %7, i64 noundef 720, i32 noundef 0)
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 10
  store ptr %8, ptr %10, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 10
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 12
  store ptr %13, ptr %15, align 8
  store i32 0, ptr %5, align 4
  br label %16

16:                                               ; preds = %27, %2
  %17 = load i32, ptr %5, align 4
  %18 = icmp slt i32 %17, 45
  br i1 %18, label %19, label %30

19:                                               ; preds = %16
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 10
  %22 = load ptr, ptr %21, align 8
  %23 = load i32, ptr %5, align 4
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds %union.StackValue, ptr %22, i64 %24
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  store i8 0, ptr %26, align 8
  br label %27

27:                                               ; preds = %19
  %28 = load i32, ptr %5, align 4
  %29 = add nsw i32 %28, 1
  store i32 %29, ptr %5, align 4
  br label %16, !llvm.loop !8

30:                                               ; preds = %16
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 10
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 6
  store ptr %33, ptr %35, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 10
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %union.StackValue, ptr %38, i64 40
  %40 = load ptr, ptr %3, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 9
  store ptr %39, ptr %41, align 8
  %42 = load ptr, ptr %3, align 8
  %43 = getelementptr inbounds %struct.lua_State, ptr %42, i32 0, i32 16
  store ptr %43, ptr %6, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = getelementptr inbounds %struct.CallInfo, ptr %44, i32 0, i32 2
  store ptr null, ptr %45, align 8
  %46 = load ptr, ptr %6, align 8
  %47 = getelementptr inbounds %struct.CallInfo, ptr %46, i32 0, i32 3
  store ptr null, ptr %47, align 8
  %48 = load ptr, ptr %6, align 8
  %49 = getelementptr inbounds %struct.CallInfo, ptr %48, i32 0, i32 7
  store i16 2, ptr %49, align 2
  %50 = load ptr, ptr %3, align 8
  %51 = getelementptr inbounds %struct.lua_State, ptr %50, i32 0, i32 6
  %52 = load ptr, ptr %51, align 8
  %53 = load ptr, ptr %6, align 8
  %54 = getelementptr inbounds %struct.CallInfo, ptr %53, i32 0, i32 0
  store ptr %52, ptr %54, align 8
  %55 = load ptr, ptr %6, align 8
  %56 = getelementptr inbounds %struct.CallInfo, ptr %55, i32 0, i32 4
  %57 = getelementptr inbounds %struct.anon.0, ptr %56, i32 0, i32 0
  store ptr null, ptr %57, align 8
  %58 = load ptr, ptr %6, align 8
  %59 = getelementptr inbounds %struct.CallInfo, ptr %58, i32 0, i32 6
  store i16 0, ptr %59, align 4
  %60 = load ptr, ptr %3, align 8
  %61 = getelementptr inbounds %struct.lua_State, ptr %60, i32 0, i32 6
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 1
  store i8 0, ptr %63, align 8
  %64 = load ptr, ptr %3, align 8
  %65 = getelementptr inbounds %struct.lua_State, ptr %64, i32 0, i32 6
  %66 = load ptr, ptr %65, align 8
  %67 = getelementptr inbounds %union.StackValue, ptr %66, i32 1
  store ptr %67, ptr %65, align 8
  %68 = load ptr, ptr %3, align 8
  %69 = getelementptr inbounds %struct.lua_State, ptr %68, i32 0, i32 6
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %union.StackValue, ptr %70, i64 20
  %72 = load ptr, ptr %6, align 8
  %73 = getelementptr inbounds %struct.CallInfo, ptr %72, i32 0, i32 1
  store ptr %71, ptr %73, align 8
  %74 = load ptr, ptr %6, align 8
  %75 = load ptr, ptr %3, align 8
  %76 = getelementptr inbounds %struct.lua_State, ptr %75, i32 0, i32 8
  store ptr %74, ptr %76, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_freethread(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds i8, ptr %6, i64 -8
  store ptr %7, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 10
  %11 = load ptr, ptr %10, align 8
  call void @luaF_closeupval(ptr noundef %8, ptr noundef %11)
  %12 = load ptr, ptr %3, align 8
  %13 = load ptr, ptr %4, align 8
  call void @freestack(ptr noundef %13)
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %5, align 8
  call void @luaM_free_(ptr noundef %14, ptr noundef %15, i64 noundef 208)
  ret void
}

declare hidden void @luaF_closeupval(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freestack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 10
  %5 = load ptr, ptr %4, align 8
  %6 = icmp eq ptr %5, null
  br i1 %6, label %7, label %8

7:                                                ; preds = %1
  br label %32

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 16
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 8
  store ptr %10, ptr %12, align 8
  %13 = load ptr, ptr %2, align 8
  call void @freeCI(ptr noundef %13)
  %14 = load ptr, ptr %2, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 10
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 9
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 10
  %23 = load ptr, ptr %22, align 8
  %24 = ptrtoint ptr %20 to i64
  %25 = ptrtoint ptr %23 to i64
  %26 = sub i64 %24, %25
  %27 = sdiv exact i64 %26, 16
  %28 = trunc i64 %27 to i32
  %29 = add nsw i32 %28, 5
  %30 = sext i32 %29 to i64
  %31 = mul i64 %30, 16
  call void @luaM_free_(ptr noundef %14, ptr noundef %17, i64 noundef %31)
  br label %32

32:                                               ; preds = %8, %7
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaE_resetthread(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 16
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 8
  store ptr %7, ptr %9, align 8
  store ptr %7, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 10
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 0, ptr %13, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 10
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 0
  store ptr %16, ptr %18, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 7
  store i16 2, ptr %20, align 2
  %21 = load i32, ptr %4, align 4
  %22 = icmp eq i32 %21, 1
  br i1 %22, label %23, label %24

23:                                               ; preds = %2
  store i32 0, ptr %4, align 4
  br label %24

24:                                               ; preds = %23, %2
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 3
  store i8 0, ptr %26, align 2
  %27 = load ptr, ptr %3, align 8
  %28 = load i32, ptr %4, align 4
  %29 = call i32 @luaD_closeprotected(ptr noundef %27, i64 noundef 1, i32 noundef %28)
  store i32 %29, ptr %4, align 4
  %30 = load i32, ptr %4, align 4
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %39

32:                                               ; preds = %24
  %33 = load ptr, ptr %3, align 8
  %34 = load i32, ptr %4, align 4
  %35 = load ptr, ptr %3, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 10
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %union.StackValue, ptr %37, i64 1
  call void @luaD_seterrorobj(ptr noundef %33, i32 noundef %34, ptr noundef %38)
  br label %46

39:                                               ; preds = %24
  %40 = load ptr, ptr %3, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 10
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %union.StackValue, ptr %42, i64 1
  %44 = load ptr, ptr %3, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 6
  store ptr %43, ptr %45, align 8
  br label %46

46:                                               ; preds = %39, %32
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 6
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %union.StackValue, ptr %49, i64 20
  %51 = load ptr, ptr %5, align 8
  %52 = getelementptr inbounds %struct.CallInfo, ptr %51, i32 0, i32 1
  store ptr %50, ptr %52, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %struct.CallInfo, ptr %54, i32 0, i32 1
  %56 = load ptr, ptr %55, align 8
  %57 = load ptr, ptr %3, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 10
  %59 = load ptr, ptr %58, align 8
  %60 = ptrtoint ptr %56 to i64
  %61 = ptrtoint ptr %59 to i64
  %62 = sub i64 %60, %61
  %63 = sdiv exact i64 %62, 16
  %64 = trunc i64 %63 to i32
  %65 = call i32 @luaD_reallocstack(ptr noundef %53, i32 noundef %64, i32 noundef 0)
  %66 = load i32, ptr %4, align 4
  ret i32 %66
}

declare hidden i32 @luaD_closeprotected(ptr noundef, i64 noundef, i32 noundef) #1

declare hidden void @luaD_seterrorobj(ptr noundef, i32 noundef, ptr noundef) #1

declare hidden i32 @luaD_reallocstack(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_closethread(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = icmp ne ptr %6, null
  br i1 %7, label %8, label %13

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 19
  %11 = load i32, ptr %10, align 8
  %12 = and i32 %11, 65535
  br label %14

13:                                               ; preds = %2
  br label %14

14:                                               ; preds = %13, %8
  %15 = phi i32 [ %12, %8 ], [ 0, %13 ]
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 19
  store i32 %15, ptr %17, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 3
  %21 = load i8, ptr %20, align 2
  %22 = zext i8 %21 to i32
  %23 = call i32 @luaE_resetthread(ptr noundef %18, i32 noundef %22)
  store i32 %23, ptr %5, align 4
  %24 = load i32, ptr %5, align 4
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_resetthread(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @lua_closethread(ptr noundef %3, ptr noundef null)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_newstate(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = call ptr %11(ptr noundef %12, ptr noundef null, i64 noundef 8, i64 noundef 1624)
  store ptr %13, ptr %9, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = icmp eq ptr %14, null
  br i1 %15, label %16, label %17

16:                                               ; preds = %2
  store ptr null, ptr %3, align 8
  br label %163

17:                                               ; preds = %2
  %18 = load ptr, ptr %9, align 8
  %19 = getelementptr inbounds %struct.LG, ptr %18, i32 0, i32 0
  %20 = getelementptr inbounds %struct.LX, ptr %19, i32 0, i32 1
  store ptr %20, ptr %7, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = getelementptr inbounds %struct.LG, ptr %21, i32 0, i32 1
  store ptr %22, ptr %8, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 1
  store i8 8, ptr %24, align 8
  %25 = load ptr, ptr %8, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 10
  store i8 8, ptr %26, align 4
  %27 = load ptr, ptr %8, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 10
  %29 = load i8, ptr %28, align 4
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 24
  %32 = trunc i32 %31 to i8
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 2
  store i8 %32, ptr %34, align 1
  %35 = load ptr, ptr %7, align 8
  %36 = load ptr, ptr %8, align 8
  call void @preinit_thread(ptr noundef %35, ptr noundef %36)
  %37 = load ptr, ptr %7, align 8
  %38 = load ptr, ptr %8, align 8
  %39 = getelementptr inbounds %struct.global_State, ptr %38, i32 0, i32 21
  store ptr %37, ptr %39, align 8
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 0
  store ptr null, ptr %41, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = getelementptr inbounds %struct.lua_State, ptr %42, i32 0, i32 19
  %44 = load i32, ptr %43, align 8
  %45 = add i32 %44, 65536
  store i32 %45, ptr %43, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = load ptr, ptr %8, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 0
  store ptr %46, ptr %48, align 8
  %49 = load ptr, ptr %5, align 8
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.global_State, ptr %50, i32 0, i32 1
  store ptr %49, ptr %51, align 8
  %52 = load ptr, ptr %8, align 8
  %53 = getelementptr inbounds %struct.global_State, ptr %52, i32 0, i32 45
  store ptr null, ptr %53, align 8
  %54 = load ptr, ptr %8, align 8
  %55 = getelementptr inbounds %struct.global_State, ptr %54, i32 0, i32 46
  store ptr null, ptr %55, align 8
  %56 = load ptr, ptr %7, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = getelementptr inbounds %struct.global_State, ptr %57, i32 0, i32 40
  store ptr %56, ptr %58, align 8
  %59 = load ptr, ptr %7, align 8
  %60 = call i32 @luai_makeseed(ptr noundef %59)
  %61 = load ptr, ptr %8, align 8
  %62 = getelementptr inbounds %struct.global_State, ptr %61, i32 0, i32 9
  store i32 %60, ptr %62, align 8
  %63 = load ptr, ptr %8, align 8
  %64 = getelementptr inbounds %struct.global_State, ptr %63, i32 0, i32 16
  store i8 2, ptr %64, align 2
  %65 = load ptr, ptr %8, align 8
  %66 = getelementptr inbounds %struct.global_State, ptr %65, i32 0, i32 6
  %67 = getelementptr inbounds %struct.stringtable, ptr %66, i32 0, i32 1
  store i32 0, ptr %67, align 8
  %68 = load ptr, ptr %8, align 8
  %69 = getelementptr inbounds %struct.global_State, ptr %68, i32 0, i32 6
  %70 = getelementptr inbounds %struct.stringtable, ptr %69, i32 0, i32 2
  store i32 0, ptr %70, align 4
  %71 = load ptr, ptr %8, align 8
  %72 = getelementptr inbounds %struct.global_State, ptr %71, i32 0, i32 6
  %73 = getelementptr inbounds %struct.stringtable, ptr %72, i32 0, i32 0
  store ptr null, ptr %73, align 8
  %74 = load ptr, ptr %8, align 8
  %75 = getelementptr inbounds %struct.global_State, ptr %74, i32 0, i32 7
  %76 = getelementptr inbounds %struct.TValue, ptr %75, i32 0, i32 1
  store i8 0, ptr %76, align 8
  %77 = load ptr, ptr %8, align 8
  %78 = getelementptr inbounds %struct.global_State, ptr %77, i32 0, i32 39
  store ptr null, ptr %78, align 8
  %79 = load ptr, ptr %8, align 8
  %80 = getelementptr inbounds %struct.global_State, ptr %79, i32 0, i32 11
  store i8 8, ptr %80, align 1
  %81 = load ptr, ptr %8, align 8
  %82 = getelementptr inbounds %struct.global_State, ptr %81, i32 0, i32 12
  store i8 0, ptr %82, align 2
  %83 = load ptr, ptr %8, align 8
  %84 = getelementptr inbounds %struct.global_State, ptr %83, i32 0, i32 13
  store i8 0, ptr %84, align 1
  %85 = load ptr, ptr %8, align 8
  %86 = getelementptr inbounds %struct.global_State, ptr %85, i32 0, i32 17
  store i8 0, ptr %86, align 1
  %87 = load ptr, ptr %8, align 8
  %88 = getelementptr inbounds %struct.global_State, ptr %87, i32 0, i32 30
  store ptr null, ptr %88, align 8
  %89 = load ptr, ptr %8, align 8
  %90 = getelementptr inbounds %struct.global_State, ptr %89, i32 0, i32 29
  store ptr null, ptr %90, align 8
  %91 = load ptr, ptr %8, align 8
  %92 = getelementptr inbounds %struct.global_State, ptr %91, i32 0, i32 23
  store ptr null, ptr %92, align 8
  %93 = load ptr, ptr %8, align 8
  %94 = getelementptr inbounds %struct.global_State, ptr %93, i32 0, i32 33
  store ptr null, ptr %94, align 8
  %95 = load ptr, ptr %8, align 8
  %96 = getelementptr inbounds %struct.global_State, ptr %95, i32 0, i32 32
  store ptr null, ptr %96, align 8
  %97 = load ptr, ptr %8, align 8
  %98 = getelementptr inbounds %struct.global_State, ptr %97, i32 0, i32 31
  store ptr null, ptr %98, align 8
  %99 = load ptr, ptr %8, align 8
  %100 = getelementptr inbounds %struct.global_State, ptr %99, i32 0, i32 34
  store ptr null, ptr %100, align 8
  %101 = load ptr, ptr %8, align 8
  %102 = getelementptr inbounds %struct.global_State, ptr %101, i32 0, i32 37
  store ptr null, ptr %102, align 8
  %103 = load ptr, ptr %8, align 8
  %104 = getelementptr inbounds %struct.global_State, ptr %103, i32 0, i32 36
  store ptr null, ptr %104, align 8
  %105 = load ptr, ptr %8, align 8
  %106 = getelementptr inbounds %struct.global_State, ptr %105, i32 0, i32 35
  store ptr null, ptr %106, align 8
  %107 = load ptr, ptr %8, align 8
  %108 = getelementptr inbounds %struct.global_State, ptr %107, i32 0, i32 22
  store ptr null, ptr %108, align 8
  %109 = load ptr, ptr %8, align 8
  %110 = getelementptr inbounds %struct.global_State, ptr %109, i32 0, i32 25
  store ptr null, ptr %110, align 8
  %111 = load ptr, ptr %8, align 8
  %112 = getelementptr inbounds %struct.global_State, ptr %111, i32 0, i32 24
  store ptr null, ptr %112, align 8
  %113 = load ptr, ptr %8, align 8
  %114 = getelementptr inbounds %struct.global_State, ptr %113, i32 0, i32 28
  store ptr null, ptr %114, align 8
  %115 = load ptr, ptr %8, align 8
  %116 = getelementptr inbounds %struct.global_State, ptr %115, i32 0, i32 27
  store ptr null, ptr %116, align 8
  %117 = load ptr, ptr %8, align 8
  %118 = getelementptr inbounds %struct.global_State, ptr %117, i32 0, i32 26
  store ptr null, ptr %118, align 8
  %119 = load ptr, ptr %8, align 8
  %120 = getelementptr inbounds %struct.global_State, ptr %119, i32 0, i32 38
  store ptr null, ptr %120, align 8
  %121 = load ptr, ptr %8, align 8
  %122 = getelementptr inbounds %struct.global_State, ptr %121, i32 0, i32 2
  store i64 1624, ptr %122, align 8
  %123 = load ptr, ptr %8, align 8
  %124 = getelementptr inbounds %struct.global_State, ptr %123, i32 0, i32 3
  store i64 0, ptr %124, align 8
  %125 = load ptr, ptr %8, align 8
  %126 = getelementptr inbounds %struct.global_State, ptr %125, i32 0, i32 5
  store i64 0, ptr %126, align 8
  %127 = load ptr, ptr %8, align 8
  %128 = getelementptr inbounds %struct.global_State, ptr %127, i32 0, i32 8
  store ptr %128, ptr %10, align 8
  %129 = load ptr, ptr %10, align 8
  %130 = getelementptr inbounds %struct.TValue, ptr %129, i32 0, i32 0
  store i64 0, ptr %130, align 8
  %131 = load ptr, ptr %10, align 8
  %132 = getelementptr inbounds %struct.TValue, ptr %131, i32 0, i32 1
  store i8 3, ptr %132, align 8
  %133 = load ptr, ptr %8, align 8
  %134 = getelementptr inbounds %struct.global_State, ptr %133, i32 0, i32 18
  store i8 50, ptr %134, align 4
  %135 = load ptr, ptr %8, align 8
  %136 = getelementptr inbounds %struct.global_State, ptr %135, i32 0, i32 19
  store i8 25, ptr %136, align 1
  %137 = load ptr, ptr %8, align 8
  %138 = getelementptr inbounds %struct.global_State, ptr %137, i32 0, i32 20
  store i8 13, ptr %138, align 2
  %139 = load ptr, ptr %8, align 8
  %140 = getelementptr inbounds %struct.global_State, ptr %139, i32 0, i32 15
  store i8 25, ptr %140, align 1
  %141 = load ptr, ptr %8, align 8
  %142 = getelementptr inbounds %struct.global_State, ptr %141, i32 0, i32 14
  store i8 20, ptr %142, align 8
  store i32 0, ptr %6, align 4
  br label %143

143:                                              ; preds = %152, %17
  %144 = load i32, ptr %6, align 4
  %145 = icmp slt i32 %144, 9
  br i1 %145, label %146, label %155

146:                                              ; preds = %143
  %147 = load ptr, ptr %8, align 8
  %148 = getelementptr inbounds %struct.global_State, ptr %147, i32 0, i32 43
  %149 = load i32, ptr %6, align 4
  %150 = sext i32 %149 to i64
  %151 = getelementptr inbounds [9 x ptr], ptr %148, i64 0, i64 %150
  store ptr null, ptr %151, align 8
  br label %152

152:                                              ; preds = %146
  %153 = load i32, ptr %6, align 4
  %154 = add nsw i32 %153, 1
  store i32 %154, ptr %6, align 4
  br label %143, !llvm.loop !9

155:                                              ; preds = %143
  %156 = load ptr, ptr %7, align 8
  %157 = call i32 @luaD_rawrunprotected(ptr noundef %156, ptr noundef @f_luaopen, ptr noundef null)
  %158 = icmp ne i32 %157, 0
  br i1 %158, label %159, label %161

159:                                              ; preds = %155
  %160 = load ptr, ptr %7, align 8
  call void @close_state(ptr noundef %160)
  store ptr null, ptr %7, align 8
  br label %161

161:                                              ; preds = %159, %155
  %162 = load ptr, ptr %7, align 8
  store ptr %162, ptr %3, align 8
  br label %163

163:                                              ; preds = %161, %16
  %164 = load ptr, ptr %3, align 8
  ret ptr %164
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luai_makeseed(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca [24 x i8], align 16
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %9 = call i64 @time(ptr noundef null) #6
  %10 = trunc i64 %9 to i32
  store i32 %10, ptr %4, align 4
  store i32 0, ptr %5, align 4
  %11 = load ptr, ptr %2, align 8
  %12 = ptrtoint ptr %11 to i64
  store i64 %12, ptr %6, align 8
  %13 = getelementptr inbounds [24 x i8], ptr %3, i64 0, i64 0
  %14 = load i32, ptr %5, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds i8, ptr %13, i64 %15
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %16, ptr align 8 %6, i64 8, i1 false)
  %17 = load i32, ptr %5, align 4
  %18 = sext i32 %17 to i64
  %19 = add i64 %18, 8
  %20 = trunc i64 %19 to i32
  store i32 %20, ptr %5, align 4
  %21 = ptrtoint ptr %4 to i64
  store i64 %21, ptr %7, align 8
  %22 = getelementptr inbounds [24 x i8], ptr %3, i64 0, i64 0
  %23 = load i32, ptr %5, align 4
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds i8, ptr %22, i64 %24
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %25, ptr align 8 %7, i64 8, i1 false)
  %26 = load i32, ptr %5, align 4
  %27 = sext i32 %26 to i64
  %28 = add i64 %27, 8
  %29 = trunc i64 %28 to i32
  store i32 %29, ptr %5, align 4
  store i64 ptrtoint (ptr @lua_newstate to i64), ptr %8, align 8
  %30 = getelementptr inbounds [24 x i8], ptr %3, i64 0, i64 0
  %31 = load i32, ptr %5, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds i8, ptr %30, i64 %32
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %33, ptr align 8 %8, i64 8, i1 false)
  %34 = load i32, ptr %5, align 4
  %35 = sext i32 %34 to i64
  %36 = add i64 %35, 8
  %37 = trunc i64 %36 to i32
  store i32 %37, ptr %5, align 4
  %38 = getelementptr inbounds [24 x i8], ptr %3, i64 0, i64 0
  %39 = load i32, ptr %5, align 4
  %40 = sext i32 %39 to i64
  %41 = load i32, ptr %4, align 4
  %42 = call i32 @luaS_hash(ptr noundef %38, i64 noundef %40, i32 noundef %41)
  ret i32 %42
}

declare hidden i32 @luaD_rawrunprotected(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @f_luaopen(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %3, align 8
  call void @stack_init(ptr noundef %10, ptr noundef %11)
  %12 = load ptr, ptr %3, align 8
  %13 = load ptr, ptr %5, align 8
  call void @init_registry(ptr noundef %12, ptr noundef %13)
  %14 = load ptr, ptr %3, align 8
  call void @luaS_init(ptr noundef %14)
  %15 = load ptr, ptr %3, align 8
  call void @luaT_init(ptr noundef %15)
  %16 = load ptr, ptr %3, align 8
  call void @luaX_init(ptr noundef %16)
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 16
  store i8 0, ptr %18, align 2
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.global_State, ptr %19, i32 0, i32 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  store i8 0, ptr %21, align 8
  %22 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @close_state(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 7
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %16, label %14

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  call void @luaC_freeallobjects(ptr noundef %15)
  br label %25

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 16
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 8
  store ptr %18, ptr %20, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = call i32 @luaD_closeprotected(ptr noundef %21, i64 noundef 1, i32 noundef 0)
  %23 = load ptr, ptr %2, align 8
  call void @luaC_freeallobjects(ptr noundef %23)
  %24 = load ptr, ptr %2, align 8
  br label %25

25:                                               ; preds = %16, %14
  %26 = load ptr, ptr %2, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 7
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 6
  %31 = getelementptr inbounds %struct.stringtable, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %2, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 7
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 6
  %37 = getelementptr inbounds %struct.stringtable, ptr %36, i32 0, i32 2
  %38 = load i32, ptr %37, align 4
  %39 = sext i32 %38 to i64
  %40 = mul i64 %39, 8
  call void @luaM_free_(ptr noundef %26, ptr noundef %32, i64 noundef %40)
  %41 = load ptr, ptr %2, align 8
  call void @freestack(ptr noundef %41)
  %42 = load ptr, ptr %3, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = load ptr, ptr %3, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 1
  %47 = load ptr, ptr %46, align 8
  %48 = load ptr, ptr %2, align 8
  %49 = getelementptr inbounds i8, ptr %48, i64 -8
  %50 = call ptr %44(ptr noundef %47, ptr noundef %49, i64 noundef 1624, i64 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_close(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 7
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 40
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  call void @close_state(ptr noundef %8)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_warning(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 45
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %24

15:                                               ; preds = %3
  %16 = load ptr, ptr %7, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 7
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.global_State, ptr %19, i32 0, i32 46
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = load i32, ptr %6, align 4
  call void %16(ptr noundef %21, ptr noundef %22, i32 noundef %23)
  br label %24

24:                                               ; preds = %15, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaE_warnerror(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %union.StackValue, ptr %9, i64 -1
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 1
  %13 = load i8, ptr %12, align 8
  %14 = zext i8 %13 to i32
  %15 = and i32 %14, 15
  %16 = icmp eq i32 %15, 4
  br i1 %16, label %17, label %23

17:                                               ; preds = %2
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %struct.TString, ptr %20, i32 0, i32 7
  %22 = getelementptr inbounds [1 x i8], ptr %21, i64 0, i64 0
  br label %24

23:                                               ; preds = %2
  br label %24

24:                                               ; preds = %23, %17
  %25 = phi ptr [ %22, %17 ], [ @.str.1, %23 ]
  store ptr %25, ptr %6, align 8
  %26 = load ptr, ptr %3, align 8
  call void @luaE_warning(ptr noundef %26, ptr noundef @.str.2, i32 noundef 1)
  %27 = load ptr, ptr %3, align 8
  %28 = load ptr, ptr %4, align 8
  call void @luaE_warning(ptr noundef %27, ptr noundef %28, i32 noundef 1)
  %29 = load ptr, ptr %3, align 8
  call void @luaE_warning(ptr noundef %29, ptr noundef @.str.3, i32 noundef 1)
  %30 = load ptr, ptr %3, align 8
  %31 = load ptr, ptr %6, align 8
  call void @luaE_warning(ptr noundef %30, ptr noundef %31, i32 noundef 1)
  %32 = load ptr, ptr %3, align 8
  call void @luaE_warning(ptr noundef %32, ptr noundef @.str.4, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeCI(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.lua_State, ptr %5, i32 0, i32 8
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 3
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.CallInfo, ptr %11, i32 0, i32 3
  store ptr null, ptr %12, align 8
  br label %13

13:                                               ; preds = %16, %1
  %14 = load ptr, ptr %4, align 8
  store ptr %14, ptr %3, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %26

16:                                               ; preds = %13
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 3
  %19 = load ptr, ptr %18, align 8
  store ptr %19, ptr %4, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = load ptr, ptr %3, align 8
  call void @luaM_free_(ptr noundef %20, ptr noundef %21, i64 noundef 64)
  %22 = load ptr, ptr %2, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 5
  %24 = load i16, ptr %23, align 4
  %25 = add i16 %24, -1
  store i16 %25, ptr %23, align 4
  br label %13, !llvm.loop !10

26:                                               ; preds = %13
  ret void
}

; Function Attrs: nounwind
declare i64 @time(ptr noundef) #4

declare hidden i32 @luaS_hash(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @init_registry(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @luaH_new(ptr noundef %12)
  store ptr %13, ptr %5, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 7
  store ptr %15, ptr %6, align 8
  %16 = load ptr, ptr %5, align 8
  store ptr %16, ptr %7, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 0
  store ptr %17, ptr %19, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  store i8 69, ptr %21, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %5, align 8
  call void @luaH_resize(ptr noundef %23, ptr noundef %24, i32 noundef 2, i32 noundef 0)
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.Table, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i64 0
  store ptr %28, ptr %8, align 8
  %29 = load ptr, ptr %3, align 8
  store ptr %29, ptr %9, align 8
  %30 = load ptr, ptr %9, align 8
  %31 = load ptr, ptr %8, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  store ptr %30, ptr %32, align 8
  %33 = load ptr, ptr %8, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 1
  store i8 72, ptr %34, align 8
  %35 = load ptr, ptr %3, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.Table, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i64 1
  store ptr %39, ptr %10, align 8
  %40 = load ptr, ptr %3, align 8
  %41 = call ptr @luaH_new(ptr noundef %40)
  store ptr %41, ptr %11, align 8
  %42 = load ptr, ptr %11, align 8
  %43 = load ptr, ptr %10, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  store ptr %42, ptr %44, align 8
  %45 = load ptr, ptr %10, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 1
  store i8 69, ptr %46, align 8
  %47 = load ptr, ptr %3, align 8
  ret void
}

declare hidden void @luaS_init(ptr noundef) #1

declare hidden void @luaT_init(ptr noundef) #1

declare hidden void @luaX_init(ptr noundef) #1

declare hidden ptr @luaH_new(ptr noundef) #1

declare hidden void @luaH_resize(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

declare hidden void @luaC_freeallobjects(ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { noreturn }
attributes #6 = { nounwind }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!6 = distinct !{!6, !7}
!7 = !{!"llvm.loop.mustprogress"}
!8 = distinct !{!8, !7}
!9 = distinct !{!9, !7}
!10 = distinct !{!10, !7}
