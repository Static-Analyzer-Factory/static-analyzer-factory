; ModuleID = 'lapi.c'
source_filename = "lapi.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.1, i16, i16 }
%union.anon = type { %struct.anon.0 }
%struct.anon.0 = type { ptr, i64, i64 }
%union.anon.1 = type { i32 }
%union.StackValue = type { %struct.TValue }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.CClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x %struct.TValue] }
%struct.GCObject = type { ptr, i8, i8 }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%struct.Udata = type { ptr, i8, i8, i16, i64, ptr, ptr, [1 x %union.UValue] }
%union.UValue = type { %struct.TValue }
%struct.__va_list_tag = type { i32, i32, ptr, ptr }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }
%struct.CallS = type { ptr, i32 }
%struct.Zio = type { i64, ptr, ptr, ptr, ptr }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.UpVal = type { ptr, i8, i8, %union.anon.5, %union.anon.6 }
%union.anon.5 = type { ptr }
%union.anon.6 = type { %struct.anon.7 }
%struct.anon.7 = type { ptr, ptr }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }

@lua_ident = dso_local constant [129 x i8] c"$LuaVersion: Lua 5.4.7  Copyright (C) 1994-2024 Lua.org, PUC-Rio $$LuaAuthors: R. Ierusalimschy, L. H. de Figueiredo, W. Celes $\00", align 16
@luaT_typenames_ = external hidden constant [12 x ptr], align 16
@.str = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"?\00", align 1
@.str.2 = private unnamed_addr constant [10 x i8] c"(no name)\00", align 1
@getupvalref.nullup = internal constant ptr null, align 8

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_checkstack(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 8
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 9
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = ptrtoint ptr %13 to i64
  %18 = ptrtoint ptr %16 to i64
  %19 = sub i64 %17, %18
  %20 = sdiv exact i64 %19, 16
  %21 = load i32, ptr %4, align 4
  %22 = sext i32 %21 to i64
  %23 = icmp sgt i64 %20, %22
  br i1 %23, label %24, label %25

24:                                               ; preds = %2
  store i32 1, ptr %5, align 4
  br label %29

25:                                               ; preds = %2
  %26 = load ptr, ptr %3, align 8
  %27 = load i32, ptr %4, align 4
  %28 = call i32 @luaD_growstack(ptr noundef %26, i32 noundef %27, i32 noundef 0)
  store i32 %28, ptr %5, align 4
  br label %29

29:                                               ; preds = %25, %24
  %30 = load i32, ptr %5, align 4
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %52

32:                                               ; preds = %29
  %33 = load ptr, ptr %6, align 8
  %34 = getelementptr inbounds %struct.CallInfo, ptr %33, i32 0, i32 1
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %4, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds %union.StackValue, ptr %38, i64 %40
  %42 = icmp ult ptr %35, %41
  br i1 %42, label %43, label %52

43:                                               ; preds = %32
  %44 = load ptr, ptr %3, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 6
  %46 = load ptr, ptr %45, align 8
  %47 = load i32, ptr %4, align 4
  %48 = sext i32 %47 to i64
  %49 = getelementptr inbounds %union.StackValue, ptr %46, i64 %48
  %50 = load ptr, ptr %6, align 8
  %51 = getelementptr inbounds %struct.CallInfo, ptr %50, i32 0, i32 1
  store ptr %49, ptr %51, align 8
  br label %52

52:                                               ; preds = %43, %32, %29
  %53 = load i32, ptr %5, align 4
  ret i32 %53
}

declare hidden i32 @luaD_growstack(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_xmove(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %10, %11
  br i1 %12, label %13, label %14

13:                                               ; preds = %3
  br label %56

14:                                               ; preds = %3
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = sext i32 %18 to i64
  %23 = sub i64 0, %22
  %24 = getelementptr inbounds %union.StackValue, ptr %21, i64 %23
  store ptr %24, ptr %20, align 8
  store i32 0, ptr %7, align 4
  br label %25

25:                                               ; preds = %53, %14
  %26 = load i32, ptr %7, align 4
  %27 = load i32, ptr %6, align 4
  %28 = icmp slt i32 %26, %27
  br i1 %28, label %29, label %56

29:                                               ; preds = %25
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  store ptr %32, ptr %8, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 6
  %35 = load ptr, ptr %34, align 8
  %36 = load i32, ptr %7, align 4
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds %union.StackValue, ptr %35, i64 %37
  store ptr %38, ptr %9, align 8
  %39 = load ptr, ptr %8, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %9, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %40, ptr align 8 %42, i64 8, i1 false)
  %43 = load ptr, ptr %9, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = load ptr, ptr %8, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  store i8 %45, ptr %47, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %union.StackValue, ptr %51, i32 1
  store ptr %52, ptr %50, align 8
  br label %53

53:                                               ; preds = %29
  %54 = load i32, ptr %7, align 4
  %55 = add nsw i32 %54, 1
  store i32 %55, ptr %7, align 4
  br label %25, !llvm.loop !6

56:                                               ; preds = %13, %25
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_atpanic(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 39
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 39
  store ptr %11, ptr %15, align 8
  %16 = load ptr, ptr %5, align 8
  ret ptr %16
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local double @lua_version(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  ret double 5.040000e+02
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_absindex(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = icmp sgt i32 %5, 0
  br i1 %6, label %10, label %7

7:                                                ; preds = %2
  %8 = load i32, ptr %4, align 4
  %9 = icmp sle i32 %8, -1001000
  br i1 %9, label %10, label %12

10:                                               ; preds = %7, %2
  %11 = load i32, ptr %4, align 4
  br label %28

12:                                               ; preds = %7
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 8
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.CallInfo, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = ptrtoint ptr %15 to i64
  %22 = ptrtoint ptr %20 to i64
  %23 = sub i64 %21, %22
  %24 = sdiv exact i64 %23, 16
  %25 = trunc i64 %24 to i32
  %26 = load i32, ptr %4, align 4
  %27 = add nsw i32 %25, %26
  br label %28

28:                                               ; preds = %12, %10
  %29 = phi i32 [ %11, %10 ], [ %27, %12 ]
  ret i32 %29
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_gettop(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 6
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 8
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %union.StackValue, ptr %10, i64 1
  %12 = ptrtoint ptr %5 to i64
  %13 = ptrtoint ptr %11 to i64
  %14 = sub i64 %12, %13
  %15 = sdiv exact i64 %14, 16
  %16 = trunc i64 %15 to i32
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_settop(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.CallInfo, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %6, align 8
  %15 = load i32, ptr %4, align 4
  %16 = icmp sge i32 %15, 0
  br i1 %16, label %17, label %44

17:                                               ; preds = %2
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %union.StackValue, ptr %19, i64 1
  %21 = load i32, ptr %4, align 4
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds %union.StackValue, ptr %20, i64 %22
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.lua_State, ptr %24, i32 0, i32 6
  %26 = load ptr, ptr %25, align 8
  %27 = ptrtoint ptr %23 to i64
  %28 = ptrtoint ptr %26 to i64
  %29 = sub i64 %27, %28
  %30 = sdiv exact i64 %29, 16
  store i64 %30, ptr %8, align 8
  br label %31

31:                                               ; preds = %40, %17
  %32 = load i64, ptr %8, align 8
  %33 = icmp sgt i64 %32, 0
  br i1 %33, label %34, label %43

34:                                               ; preds = %31
  %35 = load ptr, ptr %3, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %union.StackValue, ptr %37, i32 1
  store ptr %38, ptr %36, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 1
  store i8 0, ptr %39, align 8
  br label %40

40:                                               ; preds = %34
  %41 = load i64, ptr %8, align 8
  %42 = add nsw i64 %41, -1
  store i64 %42, ptr %8, align 8
  br label %31, !llvm.loop !8

43:                                               ; preds = %31
  br label %49

44:                                               ; preds = %2
  %45 = load ptr, ptr %3, align 8
  %46 = load i32, ptr %4, align 4
  %47 = add nsw i32 %46, 1
  %48 = sext i32 %47 to i64
  store i64 %48, ptr %8, align 8
  br label %49

49:                                               ; preds = %44, %43
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %3, align 8
  %52 = getelementptr inbounds %struct.lua_State, ptr %51, i32 0, i32 6
  %53 = load ptr, ptr %52, align 8
  %54 = load i64, ptr %8, align 8
  %55 = getelementptr inbounds %union.StackValue, ptr %53, i64 %54
  store ptr %55, ptr %7, align 8
  %56 = load i64, ptr %8, align 8
  %57 = icmp slt i64 %56, 0
  br i1 %57, label %58, label %68

58:                                               ; preds = %49
  %59 = load ptr, ptr %3, align 8
  %60 = getelementptr inbounds %struct.lua_State, ptr %59, i32 0, i32 12
  %61 = load ptr, ptr %60, align 8
  %62 = load ptr, ptr %7, align 8
  %63 = icmp uge ptr %61, %62
  br i1 %63, label %64, label %68

64:                                               ; preds = %58
  %65 = load ptr, ptr %3, align 8
  %66 = load ptr, ptr %7, align 8
  %67 = call ptr @luaF_close(ptr noundef %65, ptr noundef %66, i32 noundef -1, i32 noundef 0)
  store ptr %67, ptr %7, align 8
  br label %68

68:                                               ; preds = %64, %58, %49
  %69 = load ptr, ptr %7, align 8
  %70 = load ptr, ptr %3, align 8
  %71 = getelementptr inbounds %struct.lua_State, ptr %70, i32 0, i32 6
  store ptr %69, ptr %71, align 8
  ret void
}

declare hidden ptr @luaF_close(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_closeslot(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2stack(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = call ptr @luaF_close(ptr noundef %10, ptr noundef %11, i32 noundef -1, i32 noundef 0)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  store i8 0, ptr %14, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @index2stack(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 8
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %6, align 8
  %11 = load i32, ptr %5, align 4
  %12 = icmp sgt i32 %11, 0
  br i1 %12, label %13, label %22

13:                                               ; preds = %2
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = load i32, ptr %5, align 4
  %18 = sext i32 %17 to i64
  %19 = getelementptr inbounds %union.StackValue, ptr %16, i64 %18
  store ptr %19, ptr %7, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = load ptr, ptr %7, align 8
  store ptr %21, ptr %3, align 8
  br label %31

22:                                               ; preds = %2
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = load i32, ptr %5, align 4
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds %union.StackValue, ptr %27, i64 %29
  store ptr %30, ptr %3, align 8
  br label %31

31:                                               ; preds = %22, %13
  %32 = load ptr, ptr %3, align 8
  ret ptr %32
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_rotate(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 6
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %union.StackValue, ptr %12, i64 -1
  store ptr %13, ptr %8, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call ptr @index2stack(ptr noundef %14, i32 noundef %15)
  store ptr %16, ptr %7, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %6, align 4
  %19 = icmp sge i32 %18, 0
  br i1 %19, label %20, label %26

20:                                               ; preds = %3
  %21 = load ptr, ptr %8, align 8
  %22 = load i32, ptr %6, align 4
  %23 = sext i32 %22 to i64
  %24 = sub i64 0, %23
  %25 = getelementptr inbounds %union.StackValue, ptr %21, i64 %24
  br label %33

26:                                               ; preds = %3
  %27 = load ptr, ptr %7, align 8
  %28 = load i32, ptr %6, align 4
  %29 = sext i32 %28 to i64
  %30 = sub i64 0, %29
  %31 = getelementptr inbounds %union.StackValue, ptr %27, i64 %30
  %32 = getelementptr inbounds %union.StackValue, ptr %31, i64 -1
  br label %33

33:                                               ; preds = %26, %20
  %34 = phi ptr [ %25, %20 ], [ %32, %26 ]
  store ptr %34, ptr %9, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = load ptr, ptr %7, align 8
  %37 = load ptr, ptr %9, align 8
  call void @reverse(ptr noundef %35, ptr noundef %36, ptr noundef %37)
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %9, align 8
  %40 = getelementptr inbounds %union.StackValue, ptr %39, i64 1
  %41 = load ptr, ptr %8, align 8
  call void @reverse(ptr noundef %38, ptr noundef %40, ptr noundef %41)
  %42 = load ptr, ptr %4, align 8
  %43 = load ptr, ptr %7, align 8
  %44 = load ptr, ptr %8, align 8
  call void @reverse(ptr noundef %42, ptr noundef %43, ptr noundef %44)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @reverse(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.TValue, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  br label %14

14:                                               ; preds = %53, %3
  %15 = load ptr, ptr %5, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = icmp ult ptr %15, %16
  br i1 %17, label %18, label %58

18:                                               ; preds = %14
  store ptr %7, ptr %8, align 8
  %19 = load ptr, ptr %5, align 8
  store ptr %19, ptr %9, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %9, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %21, ptr align 8 %23, i64 8, i1 false)
  %24 = load ptr, ptr %9, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  store i8 %26, ptr %28, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %5, align 8
  store ptr %30, ptr %10, align 8
  %31 = load ptr, ptr %6, align 8
  store ptr %31, ptr %11, align 8
  %32 = load ptr, ptr %10, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %11, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %33, ptr align 8 %35, i64 8, i1 false)
  %36 = load ptr, ptr %11, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 1
  %38 = load i8, ptr %37, align 8
  %39 = load ptr, ptr %10, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 1
  store i8 %38, ptr %40, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = load ptr, ptr %6, align 8
  store ptr %42, ptr %12, align 8
  store ptr %7, ptr %13, align 8
  %43 = load ptr, ptr %12, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  %45 = load ptr, ptr %13, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %44, ptr align 8 %46, i64 8, i1 false)
  %47 = load ptr, ptr %13, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 1
  %49 = load i8, ptr %48, align 8
  %50 = load ptr, ptr %12, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  store i8 %49, ptr %51, align 8
  %52 = load ptr, ptr %4, align 8
  br label %53

53:                                               ; preds = %18
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %union.StackValue, ptr %54, i32 1
  store ptr %55, ptr %5, align 8
  %56 = load ptr, ptr %6, align 8
  %57 = getelementptr inbounds %union.StackValue, ptr %56, i32 -1
  store ptr %57, ptr %6, align 8
  br label %14, !llvm.loop !9

58:                                               ; preds = %14
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_copy(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = call ptr @index2value(ptr noundef %11, i32 noundef %12)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %6, align 4
  %16 = call ptr @index2value(ptr noundef %14, i32 noundef %15)
  store ptr %16, ptr %8, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = load ptr, ptr %8, align 8
  store ptr %18, ptr %9, align 8
  %19 = load ptr, ptr %7, align 8
  store ptr %19, ptr %10, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %10, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %21, ptr align 8 %23, i64 8, i1 false)
  %24 = load ptr, ptr %10, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = load ptr, ptr %9, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  store i8 %26, ptr %28, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = load i32, ptr %6, align 4
  %31 = icmp slt i32 %30, -1001000
  br i1 %31, label %32, label %77

32:                                               ; preds = %3
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 1
  %35 = load i8, ptr %34, align 8
  %36 = zext i8 %35 to i32
  %37 = and i32 %36, 64
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %39, label %75

39:                                               ; preds = %32
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 8
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %struct.CallInfo, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %struct.CClosure, ptr %46, i32 0, i32 2
  %48 = load i8, ptr %47, align 1
  %49 = zext i8 %48 to i32
  %50 = and i32 %49, 32
  %51 = icmp ne i32 %50, 0
  br i1 %51, label %52, label %73

52:                                               ; preds = %39
  %53 = load ptr, ptr %7, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.GCObject, ptr %55, i32 0, i32 2
  %57 = load i8, ptr %56, align 1
  %58 = zext i8 %57 to i32
  %59 = and i32 %58, 24
  %60 = icmp ne i32 %59, 0
  br i1 %60, label %61, label %73

61:                                               ; preds = %52
  %62 = load ptr, ptr %4, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.lua_State, ptr %63, i32 0, i32 8
  %65 = load ptr, ptr %64, align 8
  %66 = getelementptr inbounds %struct.CallInfo, ptr %65, i32 0, i32 0
  %67 = load ptr, ptr %66, align 8
  %68 = getelementptr inbounds %struct.TValue, ptr %67, i32 0, i32 0
  %69 = load ptr, ptr %68, align 8
  %70 = load ptr, ptr %7, align 8
  %71 = getelementptr inbounds %struct.TValue, ptr %70, i32 0, i32 0
  %72 = load ptr, ptr %71, align 8
  call void @luaC_barrier_(ptr noundef %62, ptr noundef %69, ptr noundef %72)
  br label %74

73:                                               ; preds = %52, %39
  br label %74

74:                                               ; preds = %73, %61
  br label %76

75:                                               ; preds = %32
  br label %76

76:                                               ; preds = %75, %74
  br label %77

77:                                               ; preds = %76, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @index2value(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %6, align 8
  %12 = load i32, ptr %5, align 4
  %13 = icmp sgt i32 %12, 0
  br i1 %13, label %14, label %34

14:                                               ; preds = %2
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.CallInfo, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = load i32, ptr %5, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds %union.StackValue, ptr %17, i64 %19
  store ptr %20, ptr %7, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = icmp uge ptr %22, %25
  br i1 %26, label %27, label %32

27:                                               ; preds = %14
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 7
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %struct.global_State, ptr %30, i32 0, i32 8
  store ptr %31, ptr %3, align 8
  br label %96

32:                                               ; preds = %14
  %33 = load ptr, ptr %7, align 8
  store ptr %33, ptr %3, align 8
  br label %96

34:                                               ; preds = %2
  %35 = load i32, ptr %5, align 4
  %36 = icmp sle i32 %35, -1001000
  br i1 %36, label %45, label %37

37:                                               ; preds = %34
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = load i32, ptr %5, align 4
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds %union.StackValue, ptr %41, i64 %43
  store ptr %44, ptr %3, align 8
  br label %96

45:                                               ; preds = %34
  %46 = load i32, ptr %5, align 4
  %47 = icmp eq i32 %46, -1001000
  br i1 %47, label %48, label %53

48:                                               ; preds = %45
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 7
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.global_State, ptr %51, i32 0, i32 7
  store ptr %52, ptr %3, align 8
  br label %96

53:                                               ; preds = %45
  %54 = load i32, ptr %5, align 4
  %55 = sub nsw i32 -1001000, %54
  store i32 %55, ptr %5, align 4
  %56 = load ptr, ptr %4, align 8
  %57 = load ptr, ptr %6, align 8
  %58 = getelementptr inbounds %struct.CallInfo, ptr %57, i32 0, i32 0
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 1
  %61 = load i8, ptr %60, align 8
  %62 = zext i8 %61 to i32
  %63 = icmp eq i32 %62, 102
  br i1 %63, label %64, label %90

64:                                               ; preds = %53
  %65 = load ptr, ptr %6, align 8
  %66 = getelementptr inbounds %struct.CallInfo, ptr %65, i32 0, i32 0
  %67 = load ptr, ptr %66, align 8
  %68 = getelementptr inbounds %struct.TValue, ptr %67, i32 0, i32 0
  %69 = load ptr, ptr %68, align 8
  store ptr %69, ptr %8, align 8
  %70 = load i32, ptr %5, align 4
  %71 = load ptr, ptr %8, align 8
  %72 = getelementptr inbounds %struct.CClosure, ptr %71, i32 0, i32 3
  %73 = load i8, ptr %72, align 2
  %74 = zext i8 %73 to i32
  %75 = icmp sle i32 %70, %74
  br i1 %75, label %76, label %83

76:                                               ; preds = %64
  %77 = load ptr, ptr %8, align 8
  %78 = getelementptr inbounds %struct.CClosure, ptr %77, i32 0, i32 6
  %79 = load i32, ptr %5, align 4
  %80 = sub nsw i32 %79, 1
  %81 = sext i32 %80 to i64
  %82 = getelementptr inbounds [1 x %struct.TValue], ptr %78, i64 0, i64 %81
  br label %88

83:                                               ; preds = %64
  %84 = load ptr, ptr %4, align 8
  %85 = getelementptr inbounds %struct.lua_State, ptr %84, i32 0, i32 7
  %86 = load ptr, ptr %85, align 8
  %87 = getelementptr inbounds %struct.global_State, ptr %86, i32 0, i32 8
  br label %88

88:                                               ; preds = %83, %76
  %89 = phi ptr [ %82, %76 ], [ %87, %83 ]
  store ptr %89, ptr %3, align 8
  br label %96

90:                                               ; preds = %53
  %91 = load ptr, ptr %4, align 8
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.lua_State, ptr %92, i32 0, i32 7
  %94 = load ptr, ptr %93, align 8
  %95 = getelementptr inbounds %struct.global_State, ptr %94, i32 0, i32 8
  store ptr %95, ptr %3, align 8
  br label %96

96:                                               ; preds = %90, %88, %48, %37, %32, %27
  %97 = load ptr, ptr %3, align 8
  ret ptr %97
}

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushvalue(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %14, ptr align 8 %16, i64 8, i1 false)
  %17 = load ptr, ptr %6, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  store i8 %19, ptr %21, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i32 1
  store ptr %26, ptr %24, align 8
  %27 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_type(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = and i32 %12, 15
  %14 = icmp eq i32 %13, 0
  br i1 %14, label %15, label %22

15:                                               ; preds = %2
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 7
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.global_State, ptr %19, i32 0, i32 8
  %21 = icmp ne ptr %16, %20
  br i1 %21, label %22, label %28

22:                                               ; preds = %15, %2
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 15
  br label %29

28:                                               ; preds = %15
  br label %29

29:                                               ; preds = %28, %22
  %30 = phi i32 [ %27, %22 ], [ -1, %28 ]
  ret i32 %30
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_typename(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = add nsw i32 %7, 1
  %9 = sext i32 %8 to i64
  %10 = getelementptr inbounds [12 x ptr], ptr @luaT_typenames_, i64 0, i64 %9
  %11 = load ptr, ptr %10, align 8
  ret ptr %11
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_iscfunction(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 22
  br i1 %13, label %20, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = icmp eq i32 %18, 102
  br label %20

20:                                               ; preds = %14, %2
  %21 = phi i1 [ true, %2 ], [ %19, %14 ]
  %22 = zext i1 %21 to i32
  ret i32 %22
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_isinteger(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 3
  %14 = zext i1 %13 to i32
  ret i32 %14
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_isnumber(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca double, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = call ptr @index2value(ptr noundef %7, i32 noundef %8)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = icmp eq i32 %13, 19
  br i1 %14, label %15, label %19

15:                                               ; preds = %2
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load double, ptr %17, align 8
  store double %18, ptr %5, align 8
  br label %22

19:                                               ; preds = %2
  %20 = load ptr, ptr %6, align 8
  %21 = call i32 @luaV_tonumber_(ptr noundef %20, ptr noundef %5)
  br label %22

22:                                               ; preds = %19, %15
  %23 = phi i32 [ 1, %15 ], [ %21, %19 ]
  ret i32 %23
}

declare hidden i32 @luaV_tonumber_(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_isstring(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = and i32 %12, 15
  %14 = icmp eq i32 %13, 4
  br i1 %14, label %22, label %15

15:                                               ; preds = %2
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  %20 = and i32 %19, 15
  %21 = icmp eq i32 %20, 3
  br label %22

22:                                               ; preds = %15, %2
  %23 = phi i1 [ true, %2 ], [ %21, %15 ]
  %24 = zext i1 %23 to i32
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_isuserdata(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 71
  br i1 %13, label %20, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = icmp eq i32 %18, 2
  br label %20

20:                                               ; preds = %14, %2
  %21 = phi i1 [ true, %2 ], [ %19, %14 ]
  %22 = zext i1 %21 to i32
  ret i32 %22
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_rawequal(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %5, align 4
  %11 = call ptr @index2value(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load i32, ptr %6, align 4
  %14 = call ptr @index2value(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %8, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %21, label %28

21:                                               ; preds = %3
  %22 = load ptr, ptr %7, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 7
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 8
  %27 = icmp ne ptr %22, %26
  br i1 %27, label %28, label %46

28:                                               ; preds = %21, %3
  %29 = load ptr, ptr %8, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 15
  %34 = icmp eq i32 %33, 0
  br i1 %34, label %35, label %42

35:                                               ; preds = %28
  %36 = load ptr, ptr %8, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 7
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 8
  %41 = icmp ne ptr %36, %40
  br i1 %41, label %42, label %46

42:                                               ; preds = %35, %28
  %43 = load ptr, ptr %7, align 8
  %44 = load ptr, ptr %8, align 8
  %45 = call i32 @luaV_equalobj(ptr noundef null, ptr noundef %43, ptr noundef %44)
  br label %47

46:                                               ; preds = %35, %21
  br label %47

47:                                               ; preds = %46, %42
  %48 = phi i32 [ %45, %42 ], [ 0, %46 ]
  ret i32 %48
}

declare hidden i32 @luaV_equalobj(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_arith(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load i32, ptr %4, align 4
  %8 = icmp ne i32 %7, 12
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load i32, ptr %4, align 4
  %11 = icmp ne i32 %10, 13
  br i1 %11, label %12, label %14

12:                                               ; preds = %9
  %13 = load ptr, ptr %3, align 8
  br label %38

14:                                               ; preds = %9, %2
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %5, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %union.StackValue, ptr %21, i64 -1
  store ptr %22, ptr %6, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %24, ptr align 8 %26, i64 8, i1 false)
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 %29, ptr %31, align 8
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 6
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i32 1
  store ptr %36, ptr %34, align 8
  %37 = load ptr, ptr %3, align 8
  br label %38

38:                                               ; preds = %14, %12
  %39 = load ptr, ptr %3, align 8
  %40 = load i32, ptr %4, align 4
  %41 = load ptr, ptr %3, align 8
  %42 = getelementptr inbounds %struct.lua_State, ptr %41, i32 0, i32 6
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %union.StackValue, ptr %43, i64 -2
  %45 = load ptr, ptr %3, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 6
  %47 = load ptr, ptr %46, align 8
  %48 = getelementptr inbounds %union.StackValue, ptr %47, i64 -1
  %49 = load ptr, ptr %3, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %union.StackValue, ptr %51, i64 -2
  call void @luaO_arith(ptr noundef %39, i32 noundef %40, ptr noundef %44, ptr noundef %48, ptr noundef %52)
  %53 = load ptr, ptr %3, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 6
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %union.StackValue, ptr %55, i32 -1
  store ptr %56, ptr %54, align 8
  ret void
}

declare hidden void @luaO_arith(ptr noundef, i32 noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_compare(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  store i32 0, ptr %11, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = call ptr @index2value(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %7, align 4
  %17 = call ptr @index2value(ptr noundef %15, i32 noundef %16)
  store ptr %17, ptr %10, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 1
  %20 = load i8, ptr %19, align 8
  %21 = zext i8 %20 to i32
  %22 = and i32 %21, 15
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %31

24:                                               ; preds = %4
  %25 = load ptr, ptr %9, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.lua_State, ptr %26, i32 0, i32 7
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 8
  %30 = icmp ne ptr %25, %29
  br i1 %30, label %31, label %65

31:                                               ; preds = %24, %4
  %32 = load ptr, ptr %10, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = and i32 %35, 15
  %37 = icmp eq i32 %36, 0
  br i1 %37, label %38, label %45

38:                                               ; preds = %31
  %39 = load ptr, ptr %10, align 8
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 7
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 8
  %44 = icmp ne ptr %39, %43
  br i1 %44, label %45, label %65

45:                                               ; preds = %38, %31
  %46 = load i32, ptr %8, align 4
  switch i32 %46, label %62 [
    i32 0, label %47
    i32 1, label %52
    i32 2, label %57
  ]

47:                                               ; preds = %45
  %48 = load ptr, ptr %5, align 8
  %49 = load ptr, ptr %9, align 8
  %50 = load ptr, ptr %10, align 8
  %51 = call i32 @luaV_equalobj(ptr noundef %48, ptr noundef %49, ptr noundef %50)
  store i32 %51, ptr %11, align 4
  br label %64

52:                                               ; preds = %45
  %53 = load ptr, ptr %5, align 8
  %54 = load ptr, ptr %9, align 8
  %55 = load ptr, ptr %10, align 8
  %56 = call i32 @luaV_lessthan(ptr noundef %53, ptr noundef %54, ptr noundef %55)
  store i32 %56, ptr %11, align 4
  br label %64

57:                                               ; preds = %45
  %58 = load ptr, ptr %5, align 8
  %59 = load ptr, ptr %9, align 8
  %60 = load ptr, ptr %10, align 8
  %61 = call i32 @luaV_lessequal(ptr noundef %58, ptr noundef %59, ptr noundef %60)
  store i32 %61, ptr %11, align 4
  br label %64

62:                                               ; preds = %45
  %63 = load ptr, ptr %5, align 8
  br label %64

64:                                               ; preds = %62, %57, %52, %47
  br label %65

65:                                               ; preds = %64, %38, %24
  %66 = load i32, ptr %11, align 4
  ret i32 %66
}

declare hidden i32 @luaV_lessthan(ptr noundef, ptr noundef, ptr noundef) #1

declare hidden i32 @luaV_lessequal(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @lua_stringtonumber(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = call i64 @luaO_str2num(ptr noundef %6, ptr noundef %9)
  store i64 %10, ptr %5, align 8
  %11 = load i64, ptr %5, align 8
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %13, label %19

13:                                               ; preds = %2
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load ptr, ptr %3, align 8
  br label %19

19:                                               ; preds = %13, %2
  %20 = load i64, ptr %5, align 8
  ret i64 %20
}

declare hidden i64 @luaO_str2num(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local double @lua_tonumberx(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca double, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  store double 0.000000e+00, ptr %7, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %5, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = icmp eq i32 %16, 19
  br i1 %17, label %18, label %22

18:                                               ; preds = %3
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load double, ptr %20, align 8
  store double %21, ptr %7, align 8
  br label %25

22:                                               ; preds = %3
  %23 = load ptr, ptr %8, align 8
  %24 = call i32 @luaV_tonumber_(ptr noundef %23, ptr noundef %7)
  br label %25

25:                                               ; preds = %22, %18
  %26 = phi i32 [ 1, %18 ], [ %24, %22 ]
  store i32 %26, ptr %9, align 4
  %27 = load ptr, ptr %6, align 8
  %28 = icmp ne ptr %27, null
  br i1 %28, label %29, label %32

29:                                               ; preds = %25
  %30 = load i32, ptr %9, align 4
  %31 = load ptr, ptr %6, align 8
  store i32 %30, ptr %31, align 4
  br label %32

32:                                               ; preds = %29, %25
  %33 = load double, ptr %7, align 8
  ret double %33
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @lua_tointegerx(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  store i64 0, ptr %7, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %5, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = icmp eq i32 %16, 3
  %18 = zext i1 %17 to i32
  %19 = icmp ne i32 %18, 0
  %20 = zext i1 %19 to i32
  %21 = sext i32 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %23, label %27

23:                                               ; preds = %3
  %24 = load ptr, ptr %8, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  %26 = load i64, ptr %25, align 8
  store i64 %26, ptr %7, align 8
  br label %30

27:                                               ; preds = %3
  %28 = load ptr, ptr %8, align 8
  %29 = call i32 @luaV_tointeger(ptr noundef %28, ptr noundef %7, i32 noundef 0)
  br label %30

30:                                               ; preds = %27, %23
  %31 = phi i32 [ 1, %23 ], [ %29, %27 ]
  store i32 %31, ptr %9, align 4
  %32 = load ptr, ptr %6, align 8
  %33 = icmp ne ptr %32, null
  br i1 %33, label %34, label %37

34:                                               ; preds = %30
  %35 = load i32, ptr %9, align 4
  %36 = load ptr, ptr %6, align 8
  store i32 %35, ptr %36, align 4
  br label %37

37:                                               ; preds = %34, %30
  %38 = load i64, ptr %7, align 8
  ret i64 %38
}

declare hidden i32 @luaV_tointeger(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_toboolean(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 1
  br i1 %13, label %21, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 0
  br label %21

21:                                               ; preds = %14, %2
  %22 = phi i1 [ true, %2 ], [ %20, %14 ]
  %23 = xor i1 %22, true
  %24 = zext i1 %23 to i32
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_tolstring(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call ptr @index2value(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %8, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 15
  %17 = icmp eq i32 %16, 4
  br i1 %17, label %46, label %18

18:                                               ; preds = %3
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 1
  %21 = load i8, ptr %20, align 8
  %22 = zext i8 %21 to i32
  %23 = and i32 %22, 15
  %24 = icmp eq i32 %23, 3
  br i1 %24, label %31, label %25

25:                                               ; preds = %18
  %26 = load ptr, ptr %7, align 8
  %27 = icmp ne ptr %26, null
  br i1 %27, label %28, label %30

28:                                               ; preds = %25
  %29 = load ptr, ptr %7, align 8
  store i64 0, ptr %29, align 8
  br label %30

30:                                               ; preds = %28, %25
  store ptr null, ptr %4, align 8
  br label %79

31:                                               ; preds = %18
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %8, align 8
  call void @luaO_tostring(ptr noundef %32, ptr noundef %33)
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 7
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.global_State, ptr %36, i32 0, i32 3
  %38 = load i64, ptr %37, align 8
  %39 = icmp sgt i64 %38, 0
  br i1 %39, label %40, label %42

40:                                               ; preds = %31
  %41 = load ptr, ptr %5, align 8
  call void @luaC_step(ptr noundef %41)
  br label %42

42:                                               ; preds = %40, %31
  %43 = load ptr, ptr %5, align 8
  %44 = load i32, ptr %6, align 4
  %45 = call ptr @index2value(ptr noundef %43, i32 noundef %44)
  store ptr %45, ptr %8, align 8
  br label %46

46:                                               ; preds = %42, %3
  %47 = load ptr, ptr %7, align 8
  %48 = icmp ne ptr %47, null
  br i1 %48, label %49, label %73

49:                                               ; preds = %46
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 0
  %52 = load ptr, ptr %51, align 8
  %53 = getelementptr inbounds %struct.TString, ptr %52, i32 0, i32 4
  %54 = load i8, ptr %53, align 1
  %55 = zext i8 %54 to i32
  %56 = icmp ne i32 %55, 255
  br i1 %56, label %57, label %64

57:                                               ; preds = %49
  %58 = load ptr, ptr %8, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 0
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds %struct.TString, ptr %60, i32 0, i32 4
  %62 = load i8, ptr %61, align 1
  %63 = zext i8 %62 to i64
  br label %70

64:                                               ; preds = %49
  %65 = load ptr, ptr %8, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 0
  %67 = load ptr, ptr %66, align 8
  %68 = getelementptr inbounds %struct.TString, ptr %67, i32 0, i32 6
  %69 = load i64, ptr %68, align 8
  br label %70

70:                                               ; preds = %64, %57
  %71 = phi i64 [ %63, %57 ], [ %69, %64 ]
  %72 = load ptr, ptr %7, align 8
  store i64 %71, ptr %72, align 8
  br label %73

73:                                               ; preds = %70, %46
  %74 = load ptr, ptr %8, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  %76 = load ptr, ptr %75, align 8
  %77 = getelementptr inbounds %struct.TString, ptr %76, i32 0, i32 7
  %78 = getelementptr inbounds [1 x i8], ptr %77, i64 0, i64 0
  store ptr %78, ptr %4, align 8
  br label %79

79:                                               ; preds = %73, %30
  %80 = load ptr, ptr %4, align 8
  ret ptr %80
}

declare hidden void @luaO_tostring(ptr noundef, ptr noundef) #1

declare hidden void @luaC_step(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @lua_rawlen(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call ptr @index2value(ptr noundef %7, i32 noundef %8)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = and i32 %13, 63
  switch i32 %14, label %39 [
    i32 4, label %15
    i32 20, label %22
    i32 7, label %28
    i32 5, label %34
  ]

15:                                               ; preds = %2
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.TString, ptr %18, i32 0, i32 4
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i64
  store i64 %21, ptr %3, align 8
  br label %40

22:                                               ; preds = %2
  %23 = load ptr, ptr %6, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.TString, ptr %25, i32 0, i32 6
  %27 = load i64, ptr %26, align 8
  store i64 %27, ptr %3, align 8
  br label %40

28:                                               ; preds = %2
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 0
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.Udata, ptr %31, i32 0, i32 4
  %33 = load i64, ptr %32, align 8
  store i64 %33, ptr %3, align 8
  br label %40

34:                                               ; preds = %2
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 0
  %37 = load ptr, ptr %36, align 8
  %38 = call i64 @luaH_getn(ptr noundef %37)
  store i64 %38, ptr %3, align 8
  br label %40

39:                                               ; preds = %2
  store i64 0, ptr %3, align 8
  br label %40

40:                                               ; preds = %39, %34, %28, %22, %15
  %41 = load i64, ptr %3, align 8
  ret i64 %41
}

declare hidden i64 @luaH_getn(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_tocfunction(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call ptr @index2value(ptr noundef %7, i32 noundef %8)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = icmp eq i32 %13, 22
  br i1 %14, label %15, label %19

15:                                               ; preds = %2
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %3, align 8
  br label %32

19:                                               ; preds = %2
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  %24 = icmp eq i32 %23, 102
  br i1 %24, label %25, label %31

25:                                               ; preds = %19
  %26 = load ptr, ptr %6, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.CClosure, ptr %28, i32 0, i32 5
  %30 = load ptr, ptr %29, align 8
  store ptr %30, ptr %3, align 8
  br label %32

31:                                               ; preds = %19
  store ptr null, ptr %3, align 8
  br label %32

32:                                               ; preds = %31, %25, %15
  %33 = load ptr, ptr %3, align 8
  ret ptr %33
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_touserdata(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = call ptr @touserdata(ptr noundef %9)
  ret ptr %10
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @touserdata(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = getelementptr inbounds %struct.TValue, ptr %4, i32 0, i32 1
  %6 = load i8, ptr %5, align 8
  %7 = zext i8 %6 to i32
  %8 = and i32 %7, 15
  switch i32 %8, label %37 [
    i32 7, label %9
    i32 2, label %33
  ]

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.Udata, ptr %15, i32 0, i32 3
  %17 = load i16, ptr %16, align 2
  %18 = zext i16 %17 to i32
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %21

20:                                               ; preds = %9
  br label %30

21:                                               ; preds = %9
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.Udata, ptr %24, i32 0, i32 3
  %26 = load i16, ptr %25, align 2
  %27 = zext i16 %26 to i64
  %28 = mul i64 16, %27
  %29 = add i64 40, %28
  br label %30

30:                                               ; preds = %21, %20
  %31 = phi i64 [ 32, %20 ], [ %29, %21 ]
  %32 = getelementptr inbounds i8, ptr %12, i64 %31
  store ptr %32, ptr %2, align 8
  br label %38

33:                                               ; preds = %1
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  store ptr %36, ptr %2, align 8
  br label %38

37:                                               ; preds = %1
  store ptr null, ptr %2, align 8
  br label %38

38:                                               ; preds = %37, %33, %30
  %39 = load ptr, ptr %2, align 8
  ret ptr %39
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_tothread(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 72
  br i1 %13, label %15, label %14

14:                                               ; preds = %2
  br label %19

15:                                               ; preds = %2
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  br label %19

19:                                               ; preds = %15, %14
  %20 = phi ptr [ null, %14 ], [ %18, %15 ]
  ret ptr %20
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_topointer(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call ptr @index2value(ptr noundef %7, i32 noundef %8)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = and i32 %13, 63
  switch i32 %14, label %24 [
    i32 22, label %15
    i32 7, label %21
    i32 2, label %21
  ]

15:                                               ; preds = %2
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  %19 = ptrtoint ptr %18 to i64
  %20 = inttoptr i64 %19 to ptr
  store ptr %20, ptr %3, align 8
  br label %36

21:                                               ; preds = %2, %2
  %22 = load ptr, ptr %6, align 8
  %23 = call ptr @touserdata(ptr noundef %22)
  store ptr %23, ptr %3, align 8
  br label %36

24:                                               ; preds = %2
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  %27 = load i8, ptr %26, align 8
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 64
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %35

31:                                               ; preds = %24
  %32 = load ptr, ptr %6, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %33, align 8
  store ptr %34, ptr %3, align 8
  br label %36

35:                                               ; preds = %24
  store ptr null, ptr %3, align 8
  br label %36

36:                                               ; preds = %35, %31, %21, %15
  %37 = load ptr, ptr %3, align 8
  ret ptr %37
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushnil(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 6
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.TValue, ptr %5, i32 0, i32 1
  store i8 0, ptr %6, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %union.StackValue, ptr %9, i32 1
  store ptr %10, ptr %8, align 8
  %11 = load ptr, ptr %2, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushnumber(ptr noundef %0, double noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca double, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store double %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load double, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store double %9, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 19, ptr %13, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushinteger(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load i64, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store i64 %9, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 3, ptr %13, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_pushlstring(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %10 = load i64, ptr %6, align 8
  %11 = icmp eq i64 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %3
  %13 = load ptr, ptr %4, align 8
  %14 = call ptr @luaS_new(ptr noundef %13, ptr noundef @.str)
  br label %20

15:                                               ; preds = %3
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load i64, ptr %6, align 8
  %19 = call ptr @luaS_newlstr(ptr noundef %16, ptr noundef %17, i64 noundef %18)
  br label %20

20:                                               ; preds = %15, %12
  %21 = phi ptr [ %14, %12 ], [ %19, %15 ]
  store ptr %21, ptr %7, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 6
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %8, align 8
  %25 = load ptr, ptr %7, align 8
  store ptr %25, ptr %9, align 8
  %26 = load ptr, ptr %9, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  store ptr %26, ptr %28, align 8
  %29 = load ptr, ptr %9, align 8
  %30 = getelementptr inbounds %struct.TString, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = or i32 %32, 64
  %34 = trunc i32 %33 to i8
  %35 = load ptr, ptr %8, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  store i8 %34, ptr %36, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 6
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds %union.StackValue, ptr %40, i32 1
  store ptr %41, ptr %39, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 7
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 3
  %47 = load i64, ptr %46, align 8
  %48 = icmp sgt i64 %47, 0
  br i1 %48, label %49, label %51

49:                                               ; preds = %20
  %50 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %50)
  br label %51

51:                                               ; preds = %49, %20
  %52 = load ptr, ptr %7, align 8
  %53 = getelementptr inbounds %struct.TString, ptr %52, i32 0, i32 7
  %54 = getelementptr inbounds [1 x i8], ptr %53, i64 0, i64 0
  ret ptr %54
}

declare hidden ptr @luaS_new(ptr noundef, ptr noundef) #1

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_pushstring(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = icmp eq ptr %8, null
  br i1 %9, label %10, label %15

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  store i8 0, ptr %14, align 8
  br label %38

15:                                               ; preds = %2
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = call ptr @luaS_new(ptr noundef %16, ptr noundef %17)
  store ptr %18, ptr %5, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %6, align 8
  %22 = load ptr, ptr %5, align 8
  store ptr %22, ptr %7, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  store ptr %23, ptr %25, align 8
  %26 = load ptr, ptr %7, align 8
  %27 = getelementptr inbounds %struct.TString, ptr %26, i32 0, i32 1
  %28 = load i8, ptr %27, align 8
  %29 = zext i8 %28 to i32
  %30 = or i32 %29, 64
  %31 = trunc i32 %30 to i8
  %32 = load ptr, ptr %6, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  store i8 %31, ptr %33, align 8
  %34 = load ptr, ptr %3, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.TString, ptr %35, i32 0, i32 7
  %37 = getelementptr inbounds [1 x i8], ptr %36, i64 0, i64 0
  store ptr %37, ptr %4, align 8
  br label %38

38:                                               ; preds = %15, %10
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %union.StackValue, ptr %41, i32 1
  store ptr %42, ptr %40, align 8
  %43 = load ptr, ptr %3, align 8
  %44 = load ptr, ptr %3, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 7
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %struct.global_State, ptr %46, i32 0, i32 3
  %48 = load i64, ptr %47, align 8
  %49 = icmp sgt i64 %48, 0
  br i1 %49, label %50, label %52

50:                                               ; preds = %38
  %51 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %51)
  br label %52

52:                                               ; preds = %50, %38
  %53 = load ptr, ptr %4, align 8
  ret ptr %53
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_pushvfstring(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = call ptr @luaO_pushvfstring(ptr noundef %8, ptr noundef %9, ptr noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 3
  %16 = load i64, ptr %15, align 8
  %17 = icmp sgt i64 %16, 0
  br i1 %17, label %18, label %20

18:                                               ; preds = %3
  %19 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %19)
  br label %20

20:                                               ; preds = %18, %3
  %21 = load ptr, ptr %7, align 8
  ret ptr %21
}

declare hidden ptr @luaO_pushvfstring(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_pushfstring(ptr noundef %0, ptr noundef %1, ...) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca [1 x %struct.__va_list_tag], align 16
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_start(ptr %7)
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %11 = call ptr @luaO_pushvfstring(ptr noundef %8, ptr noundef %9, ptr noundef %10)
  store ptr %11, ptr %5, align 8
  %12 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_end(ptr %12)
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 3
  %17 = load i64, ptr %16, align 8
  %18 = icmp sgt i64 %17, 0
  br i1 %18, label %19, label %21

19:                                               ; preds = %2
  %20 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %20)
  br label %21

21:                                               ; preds = %19, %2
  %22 = load ptr, ptr %5, align 8
  ret ptr %22
}

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_start(ptr) #3

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_end(ptr) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushcclosure(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %13 = load i32, ptr %6, align 4
  %14 = icmp eq i32 %13, 0
  br i1 %14, label %15, label %29

15:                                               ; preds = %3
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  store ptr %19, ptr %21, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  store i8 22, ptr %23, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.lua_State, ptr %24, i32 0, i32 6
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %union.StackValue, ptr %26, i32 1
  store ptr %27, ptr %25, align 8
  %28 = load ptr, ptr %4, align 8
  br label %96

29:                                               ; preds = %3
  %30 = load ptr, ptr %4, align 8
  %31 = load ptr, ptr %4, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = load i32, ptr %6, align 4
  %34 = call ptr @luaF_newCclosure(ptr noundef %32, i32 noundef %33)
  store ptr %34, ptr %8, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = load ptr, ptr %8, align 8
  %37 = getelementptr inbounds %struct.CClosure, ptr %36, i32 0, i32 5
  store ptr %35, ptr %37, align 8
  %38 = load i32, ptr %6, align 4
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = sext i32 %38 to i64
  %43 = sub i64 0, %42
  %44 = getelementptr inbounds %union.StackValue, ptr %41, i64 %43
  store ptr %44, ptr %40, align 8
  br label %45

45:                                               ; preds = %49, %29
  %46 = load i32, ptr %6, align 4
  %47 = add nsw i32 %46, -1
  store i32 %47, ptr %6, align 4
  %48 = icmp ne i32 %46, 0
  br i1 %48, label %49, label %71

49:                                               ; preds = %45
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.CClosure, ptr %50, i32 0, i32 6
  %52 = load i32, ptr %6, align 4
  %53 = sext i32 %52 to i64
  %54 = getelementptr inbounds [1 x %struct.TValue], ptr %51, i64 0, i64 %53
  store ptr %54, ptr %9, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.lua_State, ptr %55, i32 0, i32 6
  %57 = load ptr, ptr %56, align 8
  %58 = load i32, ptr %6, align 4
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds %union.StackValue, ptr %57, i64 %59
  store ptr %60, ptr %10, align 8
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %10, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %62, ptr align 8 %64, i64 8, i1 false)
  %65 = load ptr, ptr %10, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 1
  %67 = load i8, ptr %66, align 8
  %68 = load ptr, ptr %9, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 1
  store i8 %67, ptr %69, align 8
  %70 = load ptr, ptr %4, align 8
  br label %45, !llvm.loop !10

71:                                               ; preds = %45
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 6
  %74 = load ptr, ptr %73, align 8
  store ptr %74, ptr %11, align 8
  %75 = load ptr, ptr %8, align 8
  store ptr %75, ptr %12, align 8
  %76 = load ptr, ptr %12, align 8
  %77 = load ptr, ptr %11, align 8
  %78 = getelementptr inbounds %struct.TValue, ptr %77, i32 0, i32 0
  store ptr %76, ptr %78, align 8
  %79 = load ptr, ptr %11, align 8
  %80 = getelementptr inbounds %struct.TValue, ptr %79, i32 0, i32 1
  store i8 102, ptr %80, align 8
  %81 = load ptr, ptr %4, align 8
  %82 = load ptr, ptr %4, align 8
  %83 = getelementptr inbounds %struct.lua_State, ptr %82, i32 0, i32 6
  %84 = load ptr, ptr %83, align 8
  %85 = getelementptr inbounds %union.StackValue, ptr %84, i32 1
  store ptr %85, ptr %83, align 8
  %86 = load ptr, ptr %4, align 8
  %87 = load ptr, ptr %4, align 8
  %88 = getelementptr inbounds %struct.lua_State, ptr %87, i32 0, i32 7
  %89 = load ptr, ptr %88, align 8
  %90 = getelementptr inbounds %struct.global_State, ptr %89, i32 0, i32 3
  %91 = load i64, ptr %90, align 8
  %92 = icmp sgt i64 %91, 0
  br i1 %92, label %93, label %95

93:                                               ; preds = %71
  %94 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %94)
  br label %95

95:                                               ; preds = %93, %71
  br label %96

96:                                               ; preds = %95, %15
  ret void
}

declare hidden ptr @luaF_newCclosure(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushboolean(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %12

7:                                                ; preds = %2
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 6
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  store i8 17, ptr %11, align 8
  br label %17

12:                                               ; preds = %2
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  store i8 1, ptr %16, align 8
  br label %17

17:                                               ; preds = %12, %7
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 6
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %union.StackValue, ptr %20, i32 1
  store ptr %21, ptr %19, align 8
  %22 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_pushlightuserdata(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 2, ptr %13, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_pushthread(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.lua_State, ptr %5, i32 0, i32 6
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 72, ptr %13, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %union.StackValue, ptr %17, i32 1
  store ptr %18, ptr %16, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 7
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.global_State, ptr %22, i32 0, i32 40
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %2, align 8
  %26 = icmp eq ptr %24, %25
  %27 = zext i1 %26 to i32
  ret i32 %27
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getglobal(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 7
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i64 1
  store ptr %14, ptr %5, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = call i32 @auxgetstr(ptr noundef %15, ptr noundef %16, ptr noundef %17)
  ret i32 %18
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @auxgetstr(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = call ptr @luaS_new(ptr noundef %13, ptr noundef %14)
  store ptr %15, ptr %8, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  %20 = icmp eq i32 %19, 69
  br i1 %20, label %22, label %21

21:                                               ; preds = %3
  store ptr null, ptr %7, align 8
  br i1 false, label %35, label %55

22:                                               ; preds = %3
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %8, align 8
  %27 = call ptr @luaH_getstr(ptr noundef %25, ptr noundef %26)
  store ptr %27, ptr %7, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  %30 = load i8, ptr %29, align 8
  %31 = zext i8 %30 to i32
  %32 = and i32 %31, 15
  %33 = icmp eq i32 %32, 0
  %34 = xor i1 %33, true
  br i1 %34, label %35, label %55

35:                                               ; preds = %22, %21
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  store ptr %38, ptr %9, align 8
  %39 = load ptr, ptr %7, align 8
  store ptr %39, ptr %10, align 8
  %40 = load ptr, ptr %9, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load ptr, ptr %10, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %41, ptr align 8 %43, i64 8, i1 false)
  %44 = load ptr, ptr %10, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 1
  %46 = load i8, ptr %45, align 8
  %47 = load ptr, ptr %9, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 1
  store i8 %46, ptr %48, align 8
  %49 = load ptr, ptr %4, align 8
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.lua_State, ptr %50, i32 0, i32 6
  %52 = load ptr, ptr %51, align 8
  %53 = getelementptr inbounds %union.StackValue, ptr %52, i32 1
  store ptr %53, ptr %51, align 8
  %54 = load ptr, ptr %4, align 8
  br label %88

55:                                               ; preds = %22, %21
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.lua_State, ptr %56, i32 0, i32 6
  %58 = load ptr, ptr %57, align 8
  store ptr %58, ptr %11, align 8
  %59 = load ptr, ptr %8, align 8
  store ptr %59, ptr %12, align 8
  %60 = load ptr, ptr %12, align 8
  %61 = load ptr, ptr %11, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  store ptr %60, ptr %62, align 8
  %63 = load ptr, ptr %12, align 8
  %64 = getelementptr inbounds %struct.TString, ptr %63, i32 0, i32 1
  %65 = load i8, ptr %64, align 8
  %66 = zext i8 %65 to i32
  %67 = or i32 %66, 64
  %68 = trunc i32 %67 to i8
  %69 = load ptr, ptr %11, align 8
  %70 = getelementptr inbounds %struct.TValue, ptr %69, i32 0, i32 1
  store i8 %68, ptr %70, align 8
  %71 = load ptr, ptr %4, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 6
  %74 = load ptr, ptr %73, align 8
  %75 = getelementptr inbounds %union.StackValue, ptr %74, i32 1
  store ptr %75, ptr %73, align 8
  %76 = load ptr, ptr %4, align 8
  %77 = load ptr, ptr %4, align 8
  %78 = load ptr, ptr %5, align 8
  %79 = load ptr, ptr %4, align 8
  %80 = getelementptr inbounds %struct.lua_State, ptr %79, i32 0, i32 6
  %81 = load ptr, ptr %80, align 8
  %82 = getelementptr inbounds %union.StackValue, ptr %81, i64 -1
  %83 = load ptr, ptr %4, align 8
  %84 = getelementptr inbounds %struct.lua_State, ptr %83, i32 0, i32 6
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %union.StackValue, ptr %85, i64 -1
  %87 = load ptr, ptr %7, align 8
  call void @luaV_finishget(ptr noundef %77, ptr noundef %78, ptr noundef %82, ptr noundef %86, ptr noundef %87)
  br label %88

88:                                               ; preds = %55, %35
  %89 = load ptr, ptr %4, align 8
  %90 = getelementptr inbounds %struct.lua_State, ptr %89, i32 0, i32 6
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %union.StackValue, ptr %91, i64 -1
  %93 = getelementptr inbounds %struct.TValue, ptr %92, i32 0, i32 1
  %94 = load i8, ptr %93, align 8
  %95 = zext i8 %94 to i32
  %96 = and i32 %95, 15
  ret i32 %96
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_gettable(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = load i32, ptr %4, align 4
  %11 = call ptr @index2value(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %6, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = icmp eq i32 %15, 69
  br i1 %16, label %18, label %17

17:                                               ; preds = %2
  store ptr null, ptr %5, align 8
  br i1 false, label %34, label %50

18:                                               ; preds = %2
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 6
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %union.StackValue, ptr %24, i64 -1
  %26 = call ptr @luaH_get(ptr noundef %21, ptr noundef %25)
  store ptr %26, ptr %5, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 15
  %32 = icmp eq i32 %31, 0
  %33 = xor i1 %32, true
  br i1 %33, label %34, label %50

34:                                               ; preds = %18, %17
  %35 = load ptr, ptr %3, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %union.StackValue, ptr %37, i64 -1
  store ptr %38, ptr %7, align 8
  %39 = load ptr, ptr %5, align 8
  store ptr %39, ptr %8, align 8
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load ptr, ptr %8, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %41, ptr align 8 %43, i64 8, i1 false)
  %44 = load ptr, ptr %8, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 1
  %46 = load i8, ptr %45, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 1
  store i8 %46, ptr %48, align 8
  %49 = load ptr, ptr %3, align 8
  br label %62

50:                                               ; preds = %18, %17
  %51 = load ptr, ptr %3, align 8
  %52 = load ptr, ptr %6, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 6
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %union.StackValue, ptr %55, i64 -1
  %57 = load ptr, ptr %3, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 6
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %union.StackValue, ptr %59, i64 -1
  %61 = load ptr, ptr %5, align 8
  call void @luaV_finishget(ptr noundef %51, ptr noundef %52, ptr noundef %56, ptr noundef %60, ptr noundef %61)
  br label %62

62:                                               ; preds = %50, %34
  %63 = load ptr, ptr %3, align 8
  %64 = getelementptr inbounds %struct.lua_State, ptr %63, i32 0, i32 6
  %65 = load ptr, ptr %64, align 8
  %66 = getelementptr inbounds %union.StackValue, ptr %65, i64 -1
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 1
  %68 = load i8, ptr %67, align 8
  %69 = zext i8 %68 to i32
  %70 = and i32 %69, 15
  ret i32 %70
}

declare hidden ptr @luaH_get(ptr noundef, ptr noundef) #1

declare hidden void @luaV_finishget(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getfield(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call ptr @index2value(ptr noundef %8, i32 noundef %9)
  %11 = load ptr, ptr %6, align 8
  %12 = call i32 @auxgetstr(ptr noundef %7, ptr noundef %10, ptr noundef %11)
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_geti(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca %struct.TValue, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call ptr @index2value(ptr noundef %13, i32 noundef %14)
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  %20 = icmp eq i32 %19, 69
  br i1 %20, label %22, label %21

21:                                               ; preds = %3
  store ptr null, ptr %8, align 8
  br i1 false, label %56, label %71

22:                                               ; preds = %3
  %23 = load i64, ptr %6, align 8
  %24 = sub i64 %23, 1
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.Table, ptr %27, i32 0, i32 5
  %29 = load i32, ptr %28, align 4
  %30 = zext i32 %29 to i64
  %31 = icmp ult i64 %24, %30
  br i1 %31, label %32, label %41

32:                                               ; preds = %22
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.Table, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = load i64, ptr %6, align 8
  %39 = sub nsw i64 %38, 1
  %40 = getelementptr inbounds %struct.TValue, ptr %37, i64 %39
  br label %47

41:                                               ; preds = %22
  %42 = load ptr, ptr %7, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = load i64, ptr %6, align 8
  %46 = call ptr @luaH_getint(ptr noundef %44, i64 noundef %45)
  br label %47

47:                                               ; preds = %41, %32
  %48 = phi ptr [ %40, %32 ], [ %46, %41 ]
  store ptr %48, ptr %8, align 8
  %49 = load ptr, ptr %8, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 1
  %51 = load i8, ptr %50, align 8
  %52 = zext i8 %51 to i32
  %53 = and i32 %52, 15
  %54 = icmp eq i32 %53, 0
  %55 = xor i1 %54, true
  br i1 %55, label %56, label %71

56:                                               ; preds = %47, %21
  %57 = load ptr, ptr %4, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 6
  %59 = load ptr, ptr %58, align 8
  store ptr %59, ptr %9, align 8
  %60 = load ptr, ptr %8, align 8
  store ptr %60, ptr %10, align 8
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %10, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %62, ptr align 8 %64, i64 8, i1 false)
  %65 = load ptr, ptr %10, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 1
  %67 = load i8, ptr %66, align 8
  %68 = load ptr, ptr %9, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 1
  store i8 %67, ptr %69, align 8
  %70 = load ptr, ptr %4, align 8
  br label %83

71:                                               ; preds = %47, %21
  store ptr %11, ptr %12, align 8
  %72 = load i64, ptr %6, align 8
  %73 = load ptr, ptr %12, align 8
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 0
  store i64 %72, ptr %74, align 8
  %75 = load ptr, ptr %12, align 8
  %76 = getelementptr inbounds %struct.TValue, ptr %75, i32 0, i32 1
  store i8 3, ptr %76, align 8
  %77 = load ptr, ptr %4, align 8
  %78 = load ptr, ptr %7, align 8
  %79 = load ptr, ptr %4, align 8
  %80 = getelementptr inbounds %struct.lua_State, ptr %79, i32 0, i32 6
  %81 = load ptr, ptr %80, align 8
  %82 = load ptr, ptr %8, align 8
  call void @luaV_finishget(ptr noundef %77, ptr noundef %78, ptr noundef %11, ptr noundef %81, ptr noundef %82)
  br label %83

83:                                               ; preds = %71, %56
  %84 = load ptr, ptr %4, align 8
  %85 = getelementptr inbounds %struct.lua_State, ptr %84, i32 0, i32 6
  %86 = load ptr, ptr %85, align 8
  %87 = getelementptr inbounds %union.StackValue, ptr %86, i32 1
  store ptr %87, ptr %85, align 8
  %88 = load ptr, ptr %4, align 8
  %89 = load ptr, ptr %4, align 8
  %90 = getelementptr inbounds %struct.lua_State, ptr %89, i32 0, i32 6
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %union.StackValue, ptr %91, i64 -1
  %93 = getelementptr inbounds %struct.TValue, ptr %92, i32 0, i32 1
  %94 = load i8, ptr %93, align 8
  %95 = zext i8 %94 to i32
  %96 = and i32 %95, 15
  ret i32 %96
}

declare hidden ptr @luaH_getint(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_rawget(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load i32, ptr %4, align 4
  %10 = call ptr @gettable(ptr noundef %8, i32 noundef %9)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 6
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %union.StackValue, ptr %14, i64 -1
  %16 = call ptr @luaH_get(ptr noundef %11, ptr noundef %15)
  store ptr %16, ptr %6, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 6
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %union.StackValue, ptr %19, i32 -1
  store ptr %20, ptr %18, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 @finishrawget(ptr noundef %21, ptr noundef %22)
  ret i32 %23
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @gettable(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  ret ptr %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @finishrawget(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.TValue, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  %11 = and i32 %10, 15
  %12 = icmp eq i32 %11, 0
  br i1 %12, label %13, label %18

13:                                               ; preds = %2
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  store i8 0, ptr %17, align 8
  br label %33

18:                                               ; preds = %2
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %5, align 8
  %22 = load ptr, ptr %4, align 8
  store ptr %22, ptr %6, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %24, ptr align 8 %26, i64 8, i1 false)
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 %29, ptr %31, align 8
  %32 = load ptr, ptr %3, align 8
  br label %33

33:                                               ; preds = %18, %13
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 6
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %union.StackValue, ptr %36, i32 1
  store ptr %37, ptr %35, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %union.StackValue, ptr %41, i64 -1
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 1
  %44 = load i8, ptr %43, align 8
  %45 = zext i8 %44 to i32
  %46 = and i32 %45, 15
  ret i32 %46
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_rawgeti(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call ptr @gettable(ptr noundef %8, i32 noundef %9)
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = load i64, ptr %6, align 8
  %14 = call ptr @luaH_getint(ptr noundef %12, i64 noundef %13)
  %15 = call i32 @finishrawget(ptr noundef %11, ptr noundef %14)
  ret i32 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_rawgetp(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %struct.TValue, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %5, align 4
  %12 = call ptr @gettable(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %7, align 8
  store ptr %8, ptr %9, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  store i8 2, ptr %17, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = call ptr @luaH_get(ptr noundef %19, ptr noundef %8)
  %21 = call i32 @finishrawget(ptr noundef %18, ptr noundef %20)
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_createtable(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = call ptr @luaH_new(ptr noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 6
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %8, align 8
  %15 = load ptr, ptr %7, align 8
  store ptr %15, ptr %9, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = load ptr, ptr %8, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 0
  store ptr %16, ptr %18, align 8
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 1
  store i8 69, ptr %20, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 6
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %union.StackValue, ptr %24, i32 1
  store ptr %25, ptr %23, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = load i32, ptr %5, align 4
  %28 = icmp sgt i32 %27, 0
  br i1 %28, label %32, label %29

29:                                               ; preds = %3
  %30 = load i32, ptr %6, align 4
  %31 = icmp sgt i32 %30, 0
  br i1 %31, label %32, label %37

32:                                               ; preds = %29, %3
  %33 = load ptr, ptr %4, align 8
  %34 = load ptr, ptr %7, align 8
  %35 = load i32, ptr %5, align 4
  %36 = load i32, ptr %6, align 4
  call void @luaH_resize(ptr noundef %33, ptr noundef %34, i32 noundef %35, i32 noundef %36)
  br label %37

37:                                               ; preds = %32, %29
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 7
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds %struct.global_State, ptr %40, i32 0, i32 3
  %42 = load i64, ptr %41, align 8
  %43 = icmp sgt i64 %42, 0
  br i1 %43, label %44, label %46

44:                                               ; preds = %37
  %45 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %45)
  br label %46

46:                                               ; preds = %44, %37
  ret void
}

declare hidden ptr @luaH_new(ptr noundef) #1

declare hidden void @luaH_resize(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getmetatable(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  store i32 0, ptr %7, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 15
  switch i32 %17, label %30 [
    i32 5, label %18
    i32 7, label %24
  ]

18:                                               ; preds = %2
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.Table, ptr %21, i32 0, i32 9
  %23 = load ptr, ptr %22, align 8
  store ptr %23, ptr %6, align 8
  br label %43

24:                                               ; preds = %2
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.Udata, ptr %27, i32 0, i32 5
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %6, align 8
  br label %43

30:                                               ; preds = %2
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 7
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %struct.global_State, ptr %33, i32 0, i32 43
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = zext i8 %37 to i32
  %39 = and i32 %38, 15
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds [9 x ptr], ptr %34, i64 0, i64 %40
  %42 = load ptr, ptr %41, align 8
  store ptr %42, ptr %6, align 8
  br label %43

43:                                               ; preds = %30, %24, %18
  %44 = load ptr, ptr %6, align 8
  %45 = icmp ne ptr %44, null
  br i1 %45, label %46, label %62

46:                                               ; preds = %43
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 6
  %49 = load ptr, ptr %48, align 8
  store ptr %49, ptr %8, align 8
  %50 = load ptr, ptr %6, align 8
  store ptr %50, ptr %9, align 8
  %51 = load ptr, ptr %9, align 8
  %52 = load ptr, ptr %8, align 8
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 0
  store ptr %51, ptr %53, align 8
  %54 = load ptr, ptr %8, align 8
  %55 = getelementptr inbounds %struct.TValue, ptr %54, i32 0, i32 1
  store i8 69, ptr %55, align 8
  %56 = load ptr, ptr %3, align 8
  %57 = load ptr, ptr %3, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 6
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %union.StackValue, ptr %59, i32 1
  store ptr %60, ptr %58, align 8
  %61 = load ptr, ptr %3, align 8
  store i32 1, ptr %7, align 4
  br label %62

62:                                               ; preds = %46, %43
  %63 = load i32, ptr %7, align 4
  ret i32 %63
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getiuservalue(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = call ptr @index2value(ptr noundef %11, i32 noundef %12)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %6, align 4
  %16 = icmp sle i32 %15, 0
  br i1 %16, label %26, label %17

17:                                               ; preds = %3
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.Udata, ptr %21, i32 0, i32 3
  %23 = load i16, ptr %22, align 2
  %24 = zext i16 %23 to i32
  %25 = icmp sgt i32 %18, %24
  br i1 %25, label %26, label %31

26:                                               ; preds = %17, %3
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 6
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  store i8 0, ptr %30, align 8
  store i32 -1, ptr %8, align 4
  br label %60

31:                                               ; preds = %17
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 6
  %34 = load ptr, ptr %33, align 8
  store ptr %34, ptr %9, align 8
  %35 = load ptr, ptr %7, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 0
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %struct.Udata, ptr %37, i32 0, i32 7
  %39 = load i32, ptr %6, align 4
  %40 = sub nsw i32 %39, 1
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds [1 x %union.UValue], ptr %38, i64 0, i64 %41
  store ptr %42, ptr %10, align 8
  %43 = load ptr, ptr %9, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  %45 = load ptr, ptr %10, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %44, ptr align 8 %46, i64 8, i1 false)
  %47 = load ptr, ptr %10, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 1
  %49 = load i8, ptr %48, align 8
  %50 = load ptr, ptr %9, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  store i8 %49, ptr %51, align 8
  %52 = load ptr, ptr %4, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 6
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 8
  %58 = zext i8 %57 to i32
  %59 = and i32 %58, 15
  store i32 %59, ptr %8, align 4
  br label %60

60:                                               ; preds = %31, %26
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 6
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %union.StackValue, ptr %63, i32 1
  store ptr %64, ptr %62, align 8
  %65 = load ptr, ptr %4, align 8
  %66 = load i32, ptr %8, align 4
  ret i32 %66
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_setglobal(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 7
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i64 1
  store ptr %14, ptr %5, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %4, align 8
  call void @auxsetstr(ptr noundef %15, ptr noundef %16, ptr noundef %17)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @auxsetstr(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = call ptr @luaS_new(ptr noundef %13, ptr noundef %14)
  store ptr %15, ptr %8, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 69
  br i1 %21, label %23, label %22

22:                                               ; preds = %3
  store ptr null, ptr %7, align 8
  br i1 false, label %36, label %95

23:                                               ; preds = %3
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = call ptr @luaH_getstr(ptr noundef %26, ptr noundef %27)
  store ptr %28, ptr %7, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 15
  %34 = icmp eq i32 %33, 0
  %35 = xor i1 %34, true
  br i1 %35, label %36, label %95

36:                                               ; preds = %23, %22
  %37 = load ptr, ptr %7, align 8
  store ptr %37, ptr %9, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 6
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds %union.StackValue, ptr %40, i64 -1
  store ptr %41, ptr %10, align 8
  %42 = load ptr, ptr %9, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %10, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %43, ptr align 8 %45, i64 8, i1 false)
  %46 = load ptr, ptr %10, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  %48 = load i8, ptr %47, align 8
  %49 = load ptr, ptr %9, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 1
  store i8 %48, ptr %50, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = load ptr, ptr %4, align 8
  %53 = getelementptr inbounds %struct.lua_State, ptr %52, i32 0, i32 6
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr inbounds %union.StackValue, ptr %54, i64 -1
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 8
  %58 = zext i8 %57 to i32
  %59 = and i32 %58, 64
  %60 = icmp ne i32 %59, 0
  br i1 %60, label %61, label %89

61:                                               ; preds = %36
  %62 = load ptr, ptr %5, align 8
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 0
  %64 = load ptr, ptr %63, align 8
  %65 = getelementptr inbounds %struct.GCObject, ptr %64, i32 0, i32 2
  %66 = load i8, ptr %65, align 1
  %67 = zext i8 %66 to i32
  %68 = and i32 %67, 32
  %69 = icmp ne i32 %68, 0
  br i1 %69, label %70, label %87

70:                                               ; preds = %61
  %71 = load ptr, ptr %4, align 8
  %72 = getelementptr inbounds %struct.lua_State, ptr %71, i32 0, i32 6
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr inbounds %union.StackValue, ptr %73, i64 -1
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  %76 = load ptr, ptr %75, align 8
  %77 = getelementptr inbounds %struct.GCObject, ptr %76, i32 0, i32 2
  %78 = load i8, ptr %77, align 1
  %79 = zext i8 %78 to i32
  %80 = and i32 %79, 24
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %87

82:                                               ; preds = %70
  %83 = load ptr, ptr %4, align 8
  %84 = load ptr, ptr %5, align 8
  %85 = getelementptr inbounds %struct.TValue, ptr %84, i32 0, i32 0
  %86 = load ptr, ptr %85, align 8
  call void @luaC_barrierback_(ptr noundef %83, ptr noundef %86)
  br label %88

87:                                               ; preds = %70, %61
  br label %88

88:                                               ; preds = %87, %82
  br label %90

89:                                               ; preds = %36
  br label %90

90:                                               ; preds = %89, %88
  %91 = load ptr, ptr %4, align 8
  %92 = getelementptr inbounds %struct.lua_State, ptr %91, i32 0, i32 6
  %93 = load ptr, ptr %92, align 8
  %94 = getelementptr inbounds %union.StackValue, ptr %93, i32 -1
  store ptr %94, ptr %92, align 8
  br label %132

95:                                               ; preds = %23, %22
  %96 = load ptr, ptr %4, align 8
  %97 = getelementptr inbounds %struct.lua_State, ptr %96, i32 0, i32 6
  %98 = load ptr, ptr %97, align 8
  store ptr %98, ptr %11, align 8
  %99 = load ptr, ptr %8, align 8
  store ptr %99, ptr %12, align 8
  %100 = load ptr, ptr %12, align 8
  %101 = load ptr, ptr %11, align 8
  %102 = getelementptr inbounds %struct.TValue, ptr %101, i32 0, i32 0
  store ptr %100, ptr %102, align 8
  %103 = load ptr, ptr %12, align 8
  %104 = getelementptr inbounds %struct.TString, ptr %103, i32 0, i32 1
  %105 = load i8, ptr %104, align 8
  %106 = zext i8 %105 to i32
  %107 = or i32 %106, 64
  %108 = trunc i32 %107 to i8
  %109 = load ptr, ptr %11, align 8
  %110 = getelementptr inbounds %struct.TValue, ptr %109, i32 0, i32 1
  store i8 %108, ptr %110, align 8
  %111 = load ptr, ptr %4, align 8
  %112 = load ptr, ptr %4, align 8
  %113 = getelementptr inbounds %struct.lua_State, ptr %112, i32 0, i32 6
  %114 = load ptr, ptr %113, align 8
  %115 = getelementptr inbounds %union.StackValue, ptr %114, i32 1
  store ptr %115, ptr %113, align 8
  %116 = load ptr, ptr %4, align 8
  %117 = load ptr, ptr %4, align 8
  %118 = load ptr, ptr %5, align 8
  %119 = load ptr, ptr %4, align 8
  %120 = getelementptr inbounds %struct.lua_State, ptr %119, i32 0, i32 6
  %121 = load ptr, ptr %120, align 8
  %122 = getelementptr inbounds %union.StackValue, ptr %121, i64 -1
  %123 = load ptr, ptr %4, align 8
  %124 = getelementptr inbounds %struct.lua_State, ptr %123, i32 0, i32 6
  %125 = load ptr, ptr %124, align 8
  %126 = getelementptr inbounds %union.StackValue, ptr %125, i64 -2
  %127 = load ptr, ptr %7, align 8
  call void @luaV_finishset(ptr noundef %117, ptr noundef %118, ptr noundef %122, ptr noundef %126, ptr noundef %127)
  %128 = load ptr, ptr %4, align 8
  %129 = getelementptr inbounds %struct.lua_State, ptr %128, i32 0, i32 6
  %130 = load ptr, ptr %129, align 8
  %131 = getelementptr inbounds %union.StackValue, ptr %130, i64 -2
  store ptr %131, ptr %129, align 8
  br label %132

132:                                              ; preds = %95, %90
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_settable(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = icmp eq i32 %16, 69
  br i1 %17, label %19, label %18

18:                                               ; preds = %2
  store ptr null, ptr %6, align 8
  br i1 false, label %35, label %90

19:                                               ; preds = %2
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i64 -2
  %27 = call ptr @luaH_get(ptr noundef %22, ptr noundef %26)
  store ptr %27, ptr %6, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  %30 = load i8, ptr %29, align 8
  %31 = zext i8 %30 to i32
  %32 = and i32 %31, 15
  %33 = icmp eq i32 %32, 0
  %34 = xor i1 %33, true
  br i1 %34, label %35, label %90

35:                                               ; preds = %19, %18
  %36 = load ptr, ptr %6, align 8
  store ptr %36, ptr %7, align 8
  %37 = load ptr, ptr %3, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 6
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %union.StackValue, ptr %39, i64 -1
  store ptr %40, ptr %8, align 8
  %41 = load ptr, ptr %7, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %42, ptr align 8 %44, i64 8, i1 false)
  %45 = load ptr, ptr %8, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 1
  %47 = load i8, ptr %46, align 8
  %48 = load ptr, ptr %7, align 8
  %49 = getelementptr inbounds %struct.TValue, ptr %48, i32 0, i32 1
  store i8 %47, ptr %49, align 8
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %3, align 8
  %52 = getelementptr inbounds %struct.lua_State, ptr %51, i32 0, i32 6
  %53 = load ptr, ptr %52, align 8
  %54 = getelementptr inbounds %union.StackValue, ptr %53, i64 -1
  %55 = getelementptr inbounds %struct.TValue, ptr %54, i32 0, i32 1
  %56 = load i8, ptr %55, align 8
  %57 = zext i8 %56 to i32
  %58 = and i32 %57, 64
  %59 = icmp ne i32 %58, 0
  br i1 %59, label %60, label %88

60:                                               ; preds = %35
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %struct.GCObject, ptr %63, i32 0, i32 2
  %65 = load i8, ptr %64, align 1
  %66 = zext i8 %65 to i32
  %67 = and i32 %66, 32
  %68 = icmp ne i32 %67, 0
  br i1 %68, label %69, label %86

69:                                               ; preds = %60
  %70 = load ptr, ptr %3, align 8
  %71 = getelementptr inbounds %struct.lua_State, ptr %70, i32 0, i32 6
  %72 = load ptr, ptr %71, align 8
  %73 = getelementptr inbounds %union.StackValue, ptr %72, i64 -1
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 0
  %75 = load ptr, ptr %74, align 8
  %76 = getelementptr inbounds %struct.GCObject, ptr %75, i32 0, i32 2
  %77 = load i8, ptr %76, align 1
  %78 = zext i8 %77 to i32
  %79 = and i32 %78, 24
  %80 = icmp ne i32 %79, 0
  br i1 %80, label %81, label %86

81:                                               ; preds = %69
  %82 = load ptr, ptr %3, align 8
  %83 = load ptr, ptr %5, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  call void @luaC_barrierback_(ptr noundef %82, ptr noundef %85)
  br label %87

86:                                               ; preds = %69, %60
  br label %87

87:                                               ; preds = %86, %81
  br label %89

88:                                               ; preds = %35
  br label %89

89:                                               ; preds = %88, %87
  br label %102

90:                                               ; preds = %19, %18
  %91 = load ptr, ptr %3, align 8
  %92 = load ptr, ptr %5, align 8
  %93 = load ptr, ptr %3, align 8
  %94 = getelementptr inbounds %struct.lua_State, ptr %93, i32 0, i32 6
  %95 = load ptr, ptr %94, align 8
  %96 = getelementptr inbounds %union.StackValue, ptr %95, i64 -2
  %97 = load ptr, ptr %3, align 8
  %98 = getelementptr inbounds %struct.lua_State, ptr %97, i32 0, i32 6
  %99 = load ptr, ptr %98, align 8
  %100 = getelementptr inbounds %union.StackValue, ptr %99, i64 -1
  %101 = load ptr, ptr %6, align 8
  call void @luaV_finishset(ptr noundef %91, ptr noundef %92, ptr noundef %96, ptr noundef %100, ptr noundef %101)
  br label %102

102:                                              ; preds = %90, %89
  %103 = load ptr, ptr %3, align 8
  %104 = getelementptr inbounds %struct.lua_State, ptr %103, i32 0, i32 6
  %105 = load ptr, ptr %104, align 8
  %106 = getelementptr inbounds %union.StackValue, ptr %105, i64 -2
  store ptr %106, ptr %104, align 8
  ret void
}

declare hidden void @luaC_barrierback_(ptr noundef, ptr noundef) #1

declare hidden void @luaV_finishset(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_setfield(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call ptr @index2value(ptr noundef %8, i32 noundef %9)
  %11 = load ptr, ptr %6, align 8
  call void @auxsetstr(ptr noundef %7, ptr noundef %10, ptr noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_seti(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca %struct.TValue, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call ptr @index2value(ptr noundef %14, i32 noundef %15)
  store ptr %16, ptr %7, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 69
  br i1 %21, label %23, label %22

22:                                               ; preds = %3
  store ptr null, ptr %8, align 8
  br i1 false, label %57, label %112

23:                                               ; preds = %3
  %24 = load i64, ptr %6, align 8
  %25 = sub i64 %24, 1
  %26 = load ptr, ptr %7, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.Table, ptr %28, i32 0, i32 5
  %30 = load i32, ptr %29, align 4
  %31 = zext i32 %30 to i64
  %32 = icmp ult i64 %25, %31
  br i1 %32, label %33, label %42

33:                                               ; preds = %23
  %34 = load ptr, ptr %7, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.Table, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = load i64, ptr %6, align 8
  %40 = sub nsw i64 %39, 1
  %41 = getelementptr inbounds %struct.TValue, ptr %38, i64 %40
  br label %48

42:                                               ; preds = %23
  %43 = load ptr, ptr %7, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  %45 = load ptr, ptr %44, align 8
  %46 = load i64, ptr %6, align 8
  %47 = call ptr @luaH_getint(ptr noundef %45, i64 noundef %46)
  br label %48

48:                                               ; preds = %42, %33
  %49 = phi ptr [ %41, %33 ], [ %47, %42 ]
  store ptr %49, ptr %8, align 8
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  %52 = load i8, ptr %51, align 8
  %53 = zext i8 %52 to i32
  %54 = and i32 %53, 15
  %55 = icmp eq i32 %54, 0
  %56 = xor i1 %55, true
  br i1 %56, label %57, label %112

57:                                               ; preds = %48, %22
  %58 = load ptr, ptr %8, align 8
  store ptr %58, ptr %9, align 8
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.lua_State, ptr %59, i32 0, i32 6
  %61 = load ptr, ptr %60, align 8
  %62 = getelementptr inbounds %union.StackValue, ptr %61, i64 -1
  store ptr %62, ptr %10, align 8
  %63 = load ptr, ptr %9, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 0
  %65 = load ptr, ptr %10, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %64, ptr align 8 %66, i64 8, i1 false)
  %67 = load ptr, ptr %10, align 8
  %68 = getelementptr inbounds %struct.TValue, ptr %67, i32 0, i32 1
  %69 = load i8, ptr %68, align 8
  %70 = load ptr, ptr %9, align 8
  %71 = getelementptr inbounds %struct.TValue, ptr %70, i32 0, i32 1
  store i8 %69, ptr %71, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = load ptr, ptr %4, align 8
  %74 = getelementptr inbounds %struct.lua_State, ptr %73, i32 0, i32 6
  %75 = load ptr, ptr %74, align 8
  %76 = getelementptr inbounds %union.StackValue, ptr %75, i64 -1
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 1
  %78 = load i8, ptr %77, align 8
  %79 = zext i8 %78 to i32
  %80 = and i32 %79, 64
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %110

82:                                               ; preds = %57
  %83 = load ptr, ptr %7, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %struct.GCObject, ptr %85, i32 0, i32 2
  %87 = load i8, ptr %86, align 1
  %88 = zext i8 %87 to i32
  %89 = and i32 %88, 32
  %90 = icmp ne i32 %89, 0
  br i1 %90, label %91, label %108

91:                                               ; preds = %82
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.lua_State, ptr %92, i32 0, i32 6
  %94 = load ptr, ptr %93, align 8
  %95 = getelementptr inbounds %union.StackValue, ptr %94, i64 -1
  %96 = getelementptr inbounds %struct.TValue, ptr %95, i32 0, i32 0
  %97 = load ptr, ptr %96, align 8
  %98 = getelementptr inbounds %struct.GCObject, ptr %97, i32 0, i32 2
  %99 = load i8, ptr %98, align 1
  %100 = zext i8 %99 to i32
  %101 = and i32 %100, 24
  %102 = icmp ne i32 %101, 0
  br i1 %102, label %103, label %108

103:                                              ; preds = %91
  %104 = load ptr, ptr %4, align 8
  %105 = load ptr, ptr %7, align 8
  %106 = getelementptr inbounds %struct.TValue, ptr %105, i32 0, i32 0
  %107 = load ptr, ptr %106, align 8
  call void @luaC_barrierback_(ptr noundef %104, ptr noundef %107)
  br label %109

108:                                              ; preds = %91, %82
  br label %109

109:                                              ; preds = %108, %103
  br label %111

110:                                              ; preds = %57
  br label %111

111:                                              ; preds = %110, %109
  br label %125

112:                                              ; preds = %48, %22
  store ptr %11, ptr %12, align 8
  %113 = load i64, ptr %6, align 8
  %114 = load ptr, ptr %12, align 8
  %115 = getelementptr inbounds %struct.TValue, ptr %114, i32 0, i32 0
  store i64 %113, ptr %115, align 8
  %116 = load ptr, ptr %12, align 8
  %117 = getelementptr inbounds %struct.TValue, ptr %116, i32 0, i32 1
  store i8 3, ptr %117, align 8
  %118 = load ptr, ptr %4, align 8
  %119 = load ptr, ptr %7, align 8
  %120 = load ptr, ptr %4, align 8
  %121 = getelementptr inbounds %struct.lua_State, ptr %120, i32 0, i32 6
  %122 = load ptr, ptr %121, align 8
  %123 = getelementptr inbounds %union.StackValue, ptr %122, i64 -1
  %124 = load ptr, ptr %8, align 8
  call void @luaV_finishset(ptr noundef %118, ptr noundef %119, ptr noundef %11, ptr noundef %123, ptr noundef %124)
  br label %125

125:                                              ; preds = %112, %111
  %126 = load ptr, ptr %4, align 8
  %127 = getelementptr inbounds %struct.lua_State, ptr %126, i32 0, i32 6
  %128 = load ptr, ptr %127, align 8
  %129 = getelementptr inbounds %union.StackValue, ptr %128, i32 -1
  store ptr %129, ptr %127, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_rawset(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %union.StackValue, ptr %9, i64 -2
  call void @aux_rawset(ptr noundef %5, i32 noundef %6, ptr noundef %10, i32 noundef 2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @aux_rawset(ptr noundef %0, i32 noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %6, align 4
  %13 = call ptr @gettable(ptr noundef %11, i32 noundef %12)
  store ptr %13, ptr %9, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 6
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %union.StackValue, ptr %19, i64 -1
  call void @luaH_set(ptr noundef %14, ptr noundef %15, ptr noundef %16, ptr noundef %20)
  %21 = load ptr, ptr %9, align 8
  %22 = getelementptr inbounds %struct.Table, ptr %21, i32 0, i32 3
  %23 = load i8, ptr %22, align 2
  %24 = zext i8 %23 to i32
  %25 = and i32 %24, -64
  %26 = trunc i32 %25 to i8
  store i8 %26, ptr %22, align 2
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 6
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %union.StackValue, ptr %29, i64 -1
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  %32 = load i8, ptr %31, align 8
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, 64
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %36, label %60

36:                                               ; preds = %4
  %37 = load ptr, ptr %9, align 8
  %38 = getelementptr inbounds %struct.GCObject, ptr %37, i32 0, i32 2
  %39 = load i8, ptr %38, align 1
  %40 = zext i8 %39 to i32
  %41 = and i32 %40, 32
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %43, label %58

43:                                               ; preds = %36
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 6
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %union.StackValue, ptr %46, i64 -1
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.GCObject, ptr %49, i32 0, i32 2
  %51 = load i8, ptr %50, align 1
  %52 = zext i8 %51 to i32
  %53 = and i32 %52, 24
  %54 = icmp ne i32 %53, 0
  br i1 %54, label %55, label %58

55:                                               ; preds = %43
  %56 = load ptr, ptr %5, align 8
  %57 = load ptr, ptr %9, align 8
  call void @luaC_barrierback_(ptr noundef %56, ptr noundef %57)
  br label %59

58:                                               ; preds = %43, %36
  br label %59

59:                                               ; preds = %58, %55
  br label %61

60:                                               ; preds = %4
  br label %61

61:                                               ; preds = %60, %59
  %62 = load i32, ptr %8, align 4
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.lua_State, ptr %63, i32 0, i32 6
  %65 = load ptr, ptr %64, align 8
  %66 = sext i32 %62 to i64
  %67 = sub i64 0, %66
  %68 = getelementptr inbounds %union.StackValue, ptr %65, i64 %67
  store ptr %68, ptr %64, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_rawsetp(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca %struct.TValue, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  store ptr %7, ptr %8, align 8
  %9 = load ptr, ptr %6, align 8
  %10 = load ptr, ptr %8, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  store i8 2, ptr %13, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  call void @aux_rawset(ptr noundef %14, i32 noundef %15, ptr noundef %7, i32 noundef 1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_rawseti(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %5, align 4
  %11 = call ptr @gettable(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = load i64, ptr %6, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %union.StackValue, ptr %17, i64 -1
  call void @luaH_setint(ptr noundef %12, ptr noundef %13, i64 noundef %14, ptr noundef %18)
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %union.StackValue, ptr %21, i64 -1
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = and i32 %25, 64
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %52

28:                                               ; preds = %3
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds %struct.GCObject, ptr %29, i32 0, i32 2
  %31 = load i8, ptr %30, align 1
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 32
  %34 = icmp ne i32 %33, 0
  br i1 %34, label %35, label %50

35:                                               ; preds = %28
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %union.StackValue, ptr %38, i64 -1
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %struct.GCObject, ptr %41, i32 0, i32 2
  %43 = load i8, ptr %42, align 1
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 24
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %50

47:                                               ; preds = %35
  %48 = load ptr, ptr %4, align 8
  %49 = load ptr, ptr %7, align 8
  call void @luaC_barrierback_(ptr noundef %48, ptr noundef %49)
  br label %51

50:                                               ; preds = %35, %28
  br label %51

51:                                               ; preds = %50, %47
  br label %53

52:                                               ; preds = %3
  br label %53

53:                                               ; preds = %52, %51
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.lua_State, ptr %54, i32 0, i32 6
  %56 = load ptr, ptr %55, align 8
  %57 = getelementptr inbounds %union.StackValue, ptr %56, i32 -1
  store ptr %57, ptr %55, align 8
  ret void
}

declare hidden void @luaH_setint(ptr noundef, ptr noundef, i64 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_setmetatable(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load i32, ptr %4, align 4
  %10 = call ptr @index2value(ptr noundef %8, i32 noundef %9)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %union.StackValue, ptr %13, i64 -1
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 1
  %16 = load i8, ptr %15, align 8
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 15
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %21

20:                                               ; preds = %2
  store ptr null, ptr %6, align 8
  br label %29

21:                                               ; preds = %2
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i64 -1
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  store ptr %28, ptr %6, align 8
  br label %29

29:                                               ; preds = %21, %20
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  %32 = load i8, ptr %31, align 8
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, 15
  switch i32 %34, label %111 [
    i32 5, label %35
    i32 7, label %73
  ]

35:                                               ; preds = %29
  %36 = load ptr, ptr %6, align 8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 0
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.Table, ptr %39, i32 0, i32 9
  store ptr %36, ptr %40, align 8
  %41 = load ptr, ptr %6, align 8
  %42 = icmp ne ptr %41, null
  br i1 %42, label %43, label %72

43:                                               ; preds = %35
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %struct.GCObject, ptr %46, i32 0, i32 2
  %48 = load i8, ptr %47, align 1
  %49 = zext i8 %48 to i32
  %50 = and i32 %49, 32
  %51 = icmp ne i32 %50, 0
  br i1 %51, label %52, label %65

52:                                               ; preds = %43
  %53 = load ptr, ptr %6, align 8
  %54 = getelementptr inbounds %struct.Table, ptr %53, i32 0, i32 2
  %55 = load i8, ptr %54, align 1
  %56 = zext i8 %55 to i32
  %57 = and i32 %56, 24
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %65

59:                                               ; preds = %52
  %60 = load ptr, ptr %3, align 8
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  %64 = load ptr, ptr %6, align 8
  call void @luaC_barrier_(ptr noundef %60, ptr noundef %63, ptr noundef %64)
  br label %66

65:                                               ; preds = %52, %43
  br label %66

66:                                               ; preds = %65, %59
  %67 = load ptr, ptr %3, align 8
  %68 = load ptr, ptr %5, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  %70 = load ptr, ptr %69, align 8
  %71 = load ptr, ptr %6, align 8
  call void @luaC_checkfinalizer(ptr noundef %67, ptr noundef %70, ptr noundef %71)
  br label %72

72:                                               ; preds = %66, %35
  br label %124

73:                                               ; preds = %29
  %74 = load ptr, ptr %6, align 8
  %75 = load ptr, ptr %5, align 8
  %76 = getelementptr inbounds %struct.TValue, ptr %75, i32 0, i32 0
  %77 = load ptr, ptr %76, align 8
  %78 = getelementptr inbounds %struct.Udata, ptr %77, i32 0, i32 5
  store ptr %74, ptr %78, align 8
  %79 = load ptr, ptr %6, align 8
  %80 = icmp ne ptr %79, null
  br i1 %80, label %81, label %110

81:                                               ; preds = %73
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.TValue, ptr %82, i32 0, i32 0
  %84 = load ptr, ptr %83, align 8
  %85 = getelementptr inbounds %struct.Udata, ptr %84, i32 0, i32 2
  %86 = load i8, ptr %85, align 1
  %87 = zext i8 %86 to i32
  %88 = and i32 %87, 32
  %89 = icmp ne i32 %88, 0
  br i1 %89, label %90, label %103

90:                                               ; preds = %81
  %91 = load ptr, ptr %6, align 8
  %92 = getelementptr inbounds %struct.Table, ptr %91, i32 0, i32 2
  %93 = load i8, ptr %92, align 1
  %94 = zext i8 %93 to i32
  %95 = and i32 %94, 24
  %96 = icmp ne i32 %95, 0
  br i1 %96, label %97, label %103

97:                                               ; preds = %90
  %98 = load ptr, ptr %3, align 8
  %99 = load ptr, ptr %5, align 8
  %100 = getelementptr inbounds %struct.TValue, ptr %99, i32 0, i32 0
  %101 = load ptr, ptr %100, align 8
  %102 = load ptr, ptr %6, align 8
  call void @luaC_barrier_(ptr noundef %98, ptr noundef %101, ptr noundef %102)
  br label %104

103:                                              ; preds = %90, %81
  br label %104

104:                                              ; preds = %103, %97
  %105 = load ptr, ptr %3, align 8
  %106 = load ptr, ptr %5, align 8
  %107 = getelementptr inbounds %struct.TValue, ptr %106, i32 0, i32 0
  %108 = load ptr, ptr %107, align 8
  %109 = load ptr, ptr %6, align 8
  call void @luaC_checkfinalizer(ptr noundef %105, ptr noundef %108, ptr noundef %109)
  br label %110

110:                                              ; preds = %104, %73
  br label %124

111:                                              ; preds = %29
  %112 = load ptr, ptr %6, align 8
  %113 = load ptr, ptr %3, align 8
  %114 = getelementptr inbounds %struct.lua_State, ptr %113, i32 0, i32 7
  %115 = load ptr, ptr %114, align 8
  %116 = getelementptr inbounds %struct.global_State, ptr %115, i32 0, i32 43
  %117 = load ptr, ptr %5, align 8
  %118 = getelementptr inbounds %struct.TValue, ptr %117, i32 0, i32 1
  %119 = load i8, ptr %118, align 8
  %120 = zext i8 %119 to i32
  %121 = and i32 %120, 15
  %122 = sext i32 %121 to i64
  %123 = getelementptr inbounds [9 x ptr], ptr %116, i64 0, i64 %122
  store ptr %112, ptr %123, align 8
  br label %124

124:                                              ; preds = %111, %110, %72
  %125 = load ptr, ptr %3, align 8
  %126 = getelementptr inbounds %struct.lua_State, ptr %125, i32 0, i32 6
  %127 = load ptr, ptr %126, align 8
  %128 = getelementptr inbounds %union.StackValue, ptr %127, i32 -1
  store ptr %128, ptr %126, align 8
  ret i32 1
}

declare hidden void @luaC_checkfinalizer(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_setiuservalue(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load i32, ptr %5, align 4
  %14 = call ptr @index2value(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %7, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = load i32, ptr %6, align 4
  %17 = sub i32 %16, 1
  %18 = load ptr, ptr %7, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %struct.Udata, ptr %20, i32 0, i32 3
  %22 = load i16, ptr %21, align 2
  %23 = zext i16 %22 to i32
  %24 = icmp ult i32 %17, %23
  br i1 %24, label %26, label %25

25:                                               ; preds = %3
  store i32 0, ptr %8, align 4
  br label %88

26:                                               ; preds = %3
  %27 = load ptr, ptr %7, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.Udata, ptr %29, i32 0, i32 7
  %31 = load i32, ptr %6, align 4
  %32 = sub nsw i32 %31, 1
  %33 = sext i32 %32 to i64
  %34 = getelementptr inbounds [1 x %union.UValue], ptr %30, i64 0, i64 %33
  store ptr %34, ptr %9, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %union.StackValue, ptr %37, i64 -1
  store ptr %38, ptr %10, align 8
  %39 = load ptr, ptr %9, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %10, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %40, ptr align 8 %42, i64 8, i1 false)
  %43 = load ptr, ptr %10, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = load ptr, ptr %9, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  store i8 %45, ptr %47, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %union.StackValue, ptr %51, i64 -1
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 1
  %54 = load i8, ptr %53, align 8
  %55 = zext i8 %54 to i32
  %56 = and i32 %55, 64
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %58, label %86

58:                                               ; preds = %26
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  %61 = load ptr, ptr %60, align 8
  %62 = getelementptr inbounds %struct.GCObject, ptr %61, i32 0, i32 2
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = and i32 %64, 32
  %66 = icmp ne i32 %65, 0
  br i1 %66, label %67, label %84

67:                                               ; preds = %58
  %68 = load ptr, ptr %4, align 8
  %69 = getelementptr inbounds %struct.lua_State, ptr %68, i32 0, i32 6
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %union.StackValue, ptr %70, i64 -1
  %72 = getelementptr inbounds %struct.TValue, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr inbounds %struct.GCObject, ptr %73, i32 0, i32 2
  %75 = load i8, ptr %74, align 1
  %76 = zext i8 %75 to i32
  %77 = and i32 %76, 24
  %78 = icmp ne i32 %77, 0
  br i1 %78, label %79, label %84

79:                                               ; preds = %67
  %80 = load ptr, ptr %4, align 8
  %81 = load ptr, ptr %7, align 8
  %82 = getelementptr inbounds %struct.TValue, ptr %81, i32 0, i32 0
  %83 = load ptr, ptr %82, align 8
  call void @luaC_barrierback_(ptr noundef %80, ptr noundef %83)
  br label %85

84:                                               ; preds = %67, %58
  br label %85

85:                                               ; preds = %84, %79
  br label %87

86:                                               ; preds = %26
  br label %87

87:                                               ; preds = %86, %85
  store i32 1, ptr %8, align 4
  br label %88

88:                                               ; preds = %87, %25
  %89 = load ptr, ptr %4, align 8
  %90 = getelementptr inbounds %struct.lua_State, ptr %89, i32 0, i32 6
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %union.StackValue, ptr %91, i32 -1
  store ptr %92, ptr %90, align 8
  %93 = load i32, ptr %8, align 4
  ret i32 %93
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_callk(ptr noundef %0, i32 noundef %1, i32 noundef %2, i64 noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i64 %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  %19 = load i32, ptr %7, align 4
  %20 = add nsw i32 %19, 1
  %21 = sext i32 %20 to i64
  %22 = sub i64 0, %21
  %23 = getelementptr inbounds %union.StackValue, ptr %18, i64 %22
  store ptr %23, ptr %11, align 8
  %24 = load ptr, ptr %10, align 8
  %25 = icmp ne ptr %24, null
  br i1 %25, label %26, label %48

26:                                               ; preds = %5
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 19
  %29 = load i32, ptr %28, align 8
  %30 = and i32 %29, -65536
  %31 = icmp eq i32 %30, 0
  br i1 %31, label %32, label %48

32:                                               ; preds = %26
  %33 = load ptr, ptr %10, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 8
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.CallInfo, ptr %36, i32 0, i32 4
  %38 = getelementptr inbounds %struct.anon.0, ptr %37, i32 0, i32 0
  store ptr %33, ptr %38, align 8
  %39 = load i64, ptr %9, align 8
  %40 = load ptr, ptr %6, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 8
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %struct.CallInfo, ptr %42, i32 0, i32 4
  %44 = getelementptr inbounds %struct.anon.0, ptr %43, i32 0, i32 2
  store i64 %39, ptr %44, align 8
  %45 = load ptr, ptr %6, align 8
  %46 = load ptr, ptr %11, align 8
  %47 = load i32, ptr %8, align 4
  call void @luaD_call(ptr noundef %45, ptr noundef %46, i32 noundef %47)
  br label %52

48:                                               ; preds = %26, %5
  %49 = load ptr, ptr %6, align 8
  %50 = load ptr, ptr %11, align 8
  %51 = load i32, ptr %8, align 4
  call void @luaD_callnoyield(ptr noundef %49, ptr noundef %50, i32 noundef %51)
  br label %52

52:                                               ; preds = %48, %32
  %53 = load i32, ptr %8, align 4
  %54 = icmp sle i32 %53, -1
  br i1 %54, label %55, label %73

55:                                               ; preds = %52
  %56 = load ptr, ptr %6, align 8
  %57 = getelementptr inbounds %struct.lua_State, ptr %56, i32 0, i32 8
  %58 = load ptr, ptr %57, align 8
  %59 = getelementptr inbounds %struct.CallInfo, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %59, align 8
  %61 = load ptr, ptr %6, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 6
  %63 = load ptr, ptr %62, align 8
  %64 = icmp ult ptr %60, %63
  br i1 %64, label %65, label %73

65:                                               ; preds = %55
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds %struct.lua_State, ptr %66, i32 0, i32 6
  %68 = load ptr, ptr %67, align 8
  %69 = load ptr, ptr %6, align 8
  %70 = getelementptr inbounds %struct.lua_State, ptr %69, i32 0, i32 8
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %struct.CallInfo, ptr %71, i32 0, i32 1
  store ptr %68, ptr %72, align 8
  br label %73

73:                                               ; preds = %65, %55, %52
  ret void
}

declare hidden void @luaD_call(ptr noundef, ptr noundef, i32 noundef) #1

declare hidden void @luaD_callnoyield(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_pcallk(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i64 noundef %4, ptr noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  %13 = alloca %struct.CallS, align 8
  %14 = alloca i32, align 4
  %15 = alloca i64, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store i32 %2, ptr %9, align 4
  store i32 %3, ptr %10, align 4
  store i64 %4, ptr %11, align 8
  store ptr %5, ptr %12, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = load i32, ptr %10, align 4
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %25

24:                                               ; preds = %6
  store i64 0, ptr %15, align 8
  br label %37

25:                                               ; preds = %6
  %26 = load ptr, ptr %7, align 8
  %27 = load i32, ptr %10, align 4
  %28 = call ptr @index2stack(ptr noundef %26, i32 noundef %27)
  store ptr %28, ptr %16, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = load ptr, ptr %16, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 10
  %33 = load ptr, ptr %32, align 8
  %34 = ptrtoint ptr %30 to i64
  %35 = ptrtoint ptr %33 to i64
  %36 = sub i64 %34, %35
  store i64 %36, ptr %15, align 8
  br label %37

37:                                               ; preds = %25, %24
  %38 = load ptr, ptr %7, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 6
  %40 = load ptr, ptr %39, align 8
  %41 = load i32, ptr %8, align 4
  %42 = add nsw i32 %41, 1
  %43 = sext i32 %42 to i64
  %44 = sub i64 0, %43
  %45 = getelementptr inbounds %union.StackValue, ptr %40, i64 %44
  %46 = getelementptr inbounds %struct.CallS, ptr %13, i32 0, i32 0
  store ptr %45, ptr %46, align 8
  %47 = load ptr, ptr %12, align 8
  %48 = icmp eq ptr %47, null
  br i1 %48, label %55, label %49

49:                                               ; preds = %37
  %50 = load ptr, ptr %7, align 8
  %51 = getelementptr inbounds %struct.lua_State, ptr %50, i32 0, i32 19
  %52 = load i32, ptr %51, align 8
  %53 = and i32 %52, -65536
  %54 = icmp eq i32 %53, 0
  br i1 %54, label %69, label %55

55:                                               ; preds = %49, %37
  %56 = load i32, ptr %9, align 4
  %57 = getelementptr inbounds %struct.CallS, ptr %13, i32 0, i32 1
  store i32 %56, ptr %57, align 8
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.CallS, ptr %13, i32 0, i32 0
  %60 = load ptr, ptr %59, align 8
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 10
  %63 = load ptr, ptr %62, align 8
  %64 = ptrtoint ptr %60 to i64
  %65 = ptrtoint ptr %63 to i64
  %66 = sub i64 %64, %65
  %67 = load i64, ptr %15, align 8
  %68 = call i32 @luaD_pcall(ptr noundef %58, ptr noundef @f_call, ptr noundef %13, i64 noundef %66, i64 noundef %67)
  store i32 %68, ptr %14, align 4
  br label %136

69:                                               ; preds = %49
  %70 = load ptr, ptr %7, align 8
  %71 = getelementptr inbounds %struct.lua_State, ptr %70, i32 0, i32 8
  %72 = load ptr, ptr %71, align 8
  store ptr %72, ptr %17, align 8
  %73 = load ptr, ptr %12, align 8
  %74 = load ptr, ptr %17, align 8
  %75 = getelementptr inbounds %struct.CallInfo, ptr %74, i32 0, i32 4
  %76 = getelementptr inbounds %struct.anon.0, ptr %75, i32 0, i32 0
  store ptr %73, ptr %76, align 8
  %77 = load i64, ptr %11, align 8
  %78 = load ptr, ptr %17, align 8
  %79 = getelementptr inbounds %struct.CallInfo, ptr %78, i32 0, i32 4
  %80 = getelementptr inbounds %struct.anon.0, ptr %79, i32 0, i32 2
  store i64 %77, ptr %80, align 8
  %81 = getelementptr inbounds %struct.CallS, ptr %13, i32 0, i32 0
  %82 = load ptr, ptr %81, align 8
  %83 = load ptr, ptr %7, align 8
  %84 = getelementptr inbounds %struct.lua_State, ptr %83, i32 0, i32 10
  %85 = load ptr, ptr %84, align 8
  %86 = ptrtoint ptr %82 to i64
  %87 = ptrtoint ptr %85 to i64
  %88 = sub i64 %86, %87
  %89 = trunc i64 %88 to i32
  %90 = load ptr, ptr %17, align 8
  %91 = getelementptr inbounds %struct.CallInfo, ptr %90, i32 0, i32 5
  store i32 %89, ptr %91, align 8
  %92 = load ptr, ptr %7, align 8
  %93 = getelementptr inbounds %struct.lua_State, ptr %92, i32 0, i32 18
  %94 = load i64, ptr %93, align 8
  %95 = load ptr, ptr %17, align 8
  %96 = getelementptr inbounds %struct.CallInfo, ptr %95, i32 0, i32 4
  %97 = getelementptr inbounds %struct.anon.0, ptr %96, i32 0, i32 1
  store i64 %94, ptr %97, align 8
  %98 = load i64, ptr %15, align 8
  %99 = load ptr, ptr %7, align 8
  %100 = getelementptr inbounds %struct.lua_State, ptr %99, i32 0, i32 18
  store i64 %98, ptr %100, align 8
  %101 = load ptr, ptr %17, align 8
  %102 = getelementptr inbounds %struct.CallInfo, ptr %101, i32 0, i32 7
  %103 = load i16, ptr %102, align 2
  %104 = zext i16 %103 to i32
  %105 = and i32 %104, -2
  %106 = load ptr, ptr %7, align 8
  %107 = getelementptr inbounds %struct.lua_State, ptr %106, i32 0, i32 4
  %108 = load i8, ptr %107, align 1
  %109 = zext i8 %108 to i32
  %110 = or i32 %105, %109
  %111 = trunc i32 %110 to i16
  %112 = load ptr, ptr %17, align 8
  %113 = getelementptr inbounds %struct.CallInfo, ptr %112, i32 0, i32 7
  store i16 %111, ptr %113, align 2
  %114 = load ptr, ptr %17, align 8
  %115 = getelementptr inbounds %struct.CallInfo, ptr %114, i32 0, i32 7
  %116 = load i16, ptr %115, align 2
  %117 = zext i16 %116 to i32
  %118 = or i32 %117, 16
  %119 = trunc i32 %118 to i16
  store i16 %119, ptr %115, align 2
  %120 = load ptr, ptr %7, align 8
  %121 = getelementptr inbounds %struct.CallS, ptr %13, i32 0, i32 0
  %122 = load ptr, ptr %121, align 8
  %123 = load i32, ptr %9, align 4
  call void @luaD_call(ptr noundef %120, ptr noundef %122, i32 noundef %123)
  %124 = load ptr, ptr %17, align 8
  %125 = getelementptr inbounds %struct.CallInfo, ptr %124, i32 0, i32 7
  %126 = load i16, ptr %125, align 2
  %127 = zext i16 %126 to i32
  %128 = and i32 %127, -17
  %129 = trunc i32 %128 to i16
  store i16 %129, ptr %125, align 2
  %130 = load ptr, ptr %17, align 8
  %131 = getelementptr inbounds %struct.CallInfo, ptr %130, i32 0, i32 4
  %132 = getelementptr inbounds %struct.anon.0, ptr %131, i32 0, i32 1
  %133 = load i64, ptr %132, align 8
  %134 = load ptr, ptr %7, align 8
  %135 = getelementptr inbounds %struct.lua_State, ptr %134, i32 0, i32 18
  store i64 %133, ptr %135, align 8
  store i32 0, ptr %14, align 4
  br label %136

136:                                              ; preds = %69, %55
  %137 = load i32, ptr %9, align 4
  %138 = icmp sle i32 %137, -1
  br i1 %138, label %139, label %157

139:                                              ; preds = %136
  %140 = load ptr, ptr %7, align 8
  %141 = getelementptr inbounds %struct.lua_State, ptr %140, i32 0, i32 8
  %142 = load ptr, ptr %141, align 8
  %143 = getelementptr inbounds %struct.CallInfo, ptr %142, i32 0, i32 1
  %144 = load ptr, ptr %143, align 8
  %145 = load ptr, ptr %7, align 8
  %146 = getelementptr inbounds %struct.lua_State, ptr %145, i32 0, i32 6
  %147 = load ptr, ptr %146, align 8
  %148 = icmp ult ptr %144, %147
  br i1 %148, label %149, label %157

149:                                              ; preds = %139
  %150 = load ptr, ptr %7, align 8
  %151 = getelementptr inbounds %struct.lua_State, ptr %150, i32 0, i32 6
  %152 = load ptr, ptr %151, align 8
  %153 = load ptr, ptr %7, align 8
  %154 = getelementptr inbounds %struct.lua_State, ptr %153, i32 0, i32 8
  %155 = load ptr, ptr %154, align 8
  %156 = getelementptr inbounds %struct.CallInfo, ptr %155, i32 0, i32 1
  store ptr %152, ptr %156, align 8
  br label %157

157:                                              ; preds = %149, %139, %136
  %158 = load i32, ptr %14, align 4
  ret i32 %158
}

declare hidden i32 @luaD_pcall(ptr noundef, ptr noundef, ptr noundef, i64 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @f_call(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  store ptr %6, ptr %5, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.CallS, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.CallS, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 8
  call void @luaD_callnoyield(ptr noundef %7, ptr noundef %10, i32 noundef %13)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_load(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca %struct.Zio, align 8
  %12 = alloca i32, align 4
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %17 = load ptr, ptr %9, align 8
  %18 = icmp ne ptr %17, null
  br i1 %18, label %20, label %19

19:                                               ; preds = %5
  store ptr @.str.1, ptr %9, align 8
  br label %20

20:                                               ; preds = %19, %5
  %21 = load ptr, ptr %6, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = load ptr, ptr %8, align 8
  call void @luaZ_init(ptr noundef %21, ptr noundef %11, ptr noundef %22, ptr noundef %23)
  %24 = load ptr, ptr %6, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = load ptr, ptr %10, align 8
  %27 = call i32 @luaD_protectedparser(ptr noundef %24, ptr noundef %11, ptr noundef %25, ptr noundef %26)
  store i32 %27, ptr %12, align 4
  %28 = load i32, ptr %12, align 4
  %29 = icmp eq i32 %28, 0
  br i1 %29, label %30, label %108

30:                                               ; preds = %20
  %31 = load ptr, ptr %6, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 6
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %union.StackValue, ptr %33, i64 -1
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  store ptr %36, ptr %13, align 8
  %37 = load ptr, ptr %13, align 8
  %38 = getelementptr inbounds %struct.LClosure, ptr %37, i32 0, i32 3
  %39 = load i8, ptr %38, align 2
  %40 = zext i8 %39 to i32
  %41 = icmp sge i32 %40, 1
  br i1 %41, label %42, label %107

42:                                               ; preds = %30
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 7
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 7
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  %49 = getelementptr inbounds %struct.Table, ptr %48, i32 0, i32 6
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i64 1
  store ptr %51, ptr %14, align 8
  %52 = load ptr, ptr %13, align 8
  %53 = getelementptr inbounds %struct.LClosure, ptr %52, i32 0, i32 6
  %54 = getelementptr inbounds [1 x ptr], ptr %53, i64 0, i64 0
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.UpVal, ptr %55, i32 0, i32 3
  %57 = load ptr, ptr %56, align 8
  store ptr %57, ptr %15, align 8
  %58 = load ptr, ptr %14, align 8
  store ptr %58, ptr %16, align 8
  %59 = load ptr, ptr %15, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  %61 = load ptr, ptr %16, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %60, ptr align 8 %62, i64 8, i1 false)
  %63 = load ptr, ptr %16, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 1
  %65 = load i8, ptr %64, align 8
  %66 = load ptr, ptr %15, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 1
  store i8 %65, ptr %67, align 8
  %68 = load ptr, ptr %6, align 8
  %69 = load ptr, ptr %14, align 8
  %70 = getelementptr inbounds %struct.TValue, ptr %69, i32 0, i32 1
  %71 = load i8, ptr %70, align 8
  %72 = zext i8 %71 to i32
  %73 = and i32 %72, 64
  %74 = icmp ne i32 %73, 0
  br i1 %74, label %75, label %105

75:                                               ; preds = %42
  %76 = load ptr, ptr %13, align 8
  %77 = getelementptr inbounds %struct.LClosure, ptr %76, i32 0, i32 6
  %78 = getelementptr inbounds [1 x ptr], ptr %77, i64 0, i64 0
  %79 = load ptr, ptr %78, align 8
  %80 = getelementptr inbounds %struct.UpVal, ptr %79, i32 0, i32 2
  %81 = load i8, ptr %80, align 1
  %82 = zext i8 %81 to i32
  %83 = and i32 %82, 32
  %84 = icmp ne i32 %83, 0
  br i1 %84, label %85, label %103

85:                                               ; preds = %75
  %86 = load ptr, ptr %14, align 8
  %87 = getelementptr inbounds %struct.TValue, ptr %86, i32 0, i32 0
  %88 = load ptr, ptr %87, align 8
  %89 = getelementptr inbounds %struct.GCObject, ptr %88, i32 0, i32 2
  %90 = load i8, ptr %89, align 1
  %91 = zext i8 %90 to i32
  %92 = and i32 %91, 24
  %93 = icmp ne i32 %92, 0
  br i1 %93, label %94, label %103

94:                                               ; preds = %85
  %95 = load ptr, ptr %6, align 8
  %96 = load ptr, ptr %13, align 8
  %97 = getelementptr inbounds %struct.LClosure, ptr %96, i32 0, i32 6
  %98 = getelementptr inbounds [1 x ptr], ptr %97, i64 0, i64 0
  %99 = load ptr, ptr %98, align 8
  %100 = load ptr, ptr %14, align 8
  %101 = getelementptr inbounds %struct.TValue, ptr %100, i32 0, i32 0
  %102 = load ptr, ptr %101, align 8
  call void @luaC_barrier_(ptr noundef %95, ptr noundef %99, ptr noundef %102)
  br label %104

103:                                              ; preds = %85, %75
  br label %104

104:                                              ; preds = %103, %94
  br label %106

105:                                              ; preds = %42
  br label %106

106:                                              ; preds = %105, %104
  br label %107

107:                                              ; preds = %106, %30
  br label %108

108:                                              ; preds = %107, %20
  %109 = load i32, ptr %12, align 4
  ret i32 %109
}

declare hidden void @luaZ_init(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

declare hidden i32 @luaD_protectedparser(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_dump(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 6
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %union.StackValue, ptr %14, i64 -1
  store ptr %15, ptr %10, align 8
  %16 = load ptr, ptr %10, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  %20 = icmp eq i32 %19, 70
  br i1 %20, label %21, label %32

21:                                               ; preds = %4
  %22 = load ptr, ptr %5, align 8
  %23 = load ptr, ptr %10, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.LClosure, ptr %25, i32 0, i32 5
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = load i32, ptr %8, align 4
  %31 = call i32 @luaU_dump(ptr noundef %22, ptr noundef %27, ptr noundef %28, ptr noundef %29, i32 noundef %30)
  store i32 %31, ptr %9, align 4
  br label %33

32:                                               ; preds = %4
  store i32 1, ptr %9, align 4
  br label %33

33:                                               ; preds = %32, %21
  %34 = load i32, ptr %9, align 4
  ret i32 %34
}

declare hidden i32 @luaU_dump(ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_status(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 3
  %5 = load i8, ptr %4, align 2
  %6 = zext i8 %5 to i32
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_gc(ptr noundef %0, i32 noundef %1, ...) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca [1 x %struct.__va_list_tag], align 16
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i64, align 8
  %11 = alloca i8, align 1
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  %16 = alloca i32, align 4
  %17 = alloca i32, align 4
  %18 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 0, ptr %7, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 7
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %8, align 8
  %22 = load ptr, ptr %8, align 8
  %23 = getelementptr inbounds %struct.global_State, ptr %22, i32 0, i32 16
  %24 = load i8, ptr %23, align 2
  %25 = zext i8 %24 to i32
  %26 = and i32 %25, 2
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %29

28:                                               ; preds = %2
  store i32 -1, ptr %3, align 4
  br label %341

29:                                               ; preds = %2
  %30 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_start(ptr %30)
  %31 = load i32, ptr %5, align 4
  switch i32 %31, label %337 [
    i32 0, label %32
    i32 1, label %35
    i32 2, label %39
    i32 3, label %41
    i32 4, label %51
    i32 5, label %61
    i32 6, label %121
    i32 7, label %148
    i32 9, label %175
    i32 10, label %182
    i32 11, label %247
  ]

32:                                               ; preds = %29
  %33 = load ptr, ptr %8, align 8
  %34 = getelementptr inbounds %struct.global_State, ptr %33, i32 0, i32 16
  store i8 1, ptr %34, align 2
  br label %338

35:                                               ; preds = %29
  %36 = load ptr, ptr %8, align 8
  call void @luaE_setdebt(ptr noundef %36, i64 noundef 0)
  %37 = load ptr, ptr %8, align 8
  %38 = getelementptr inbounds %struct.global_State, ptr %37, i32 0, i32 16
  store i8 0, ptr %38, align 2
  br label %338

39:                                               ; preds = %29
  %40 = load ptr, ptr %4, align 8
  call void @luaC_fullgc(ptr noundef %40, i32 noundef 0)
  br label %338

41:                                               ; preds = %29
  %42 = load ptr, ptr %8, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 2
  %44 = load i64, ptr %43, align 8
  %45 = load ptr, ptr %8, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 3
  %47 = load i64, ptr %46, align 8
  %48 = add nsw i64 %44, %47
  %49 = lshr i64 %48, 10
  %50 = trunc i64 %49 to i32
  store i32 %50, ptr %7, align 4
  br label %338

51:                                               ; preds = %29
  %52 = load ptr, ptr %8, align 8
  %53 = getelementptr inbounds %struct.global_State, ptr %52, i32 0, i32 2
  %54 = load i64, ptr %53, align 8
  %55 = load ptr, ptr %8, align 8
  %56 = getelementptr inbounds %struct.global_State, ptr %55, i32 0, i32 3
  %57 = load i64, ptr %56, align 8
  %58 = add nsw i64 %54, %57
  %59 = and i64 %58, 1023
  %60 = trunc i64 %59 to i32
  store i32 %60, ptr %7, align 4
  br label %338

61:                                               ; preds = %29
  %62 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %63 = getelementptr inbounds %struct.__va_list_tag, ptr %62, i32 0, i32 0
  %64 = load i32, ptr %63, align 16
  %65 = icmp ule i32 %64, 40
  br i1 %65, label %66, label %71

66:                                               ; preds = %61
  %67 = getelementptr inbounds %struct.__va_list_tag, ptr %62, i32 0, i32 3
  %68 = load ptr, ptr %67, align 16
  %69 = getelementptr i8, ptr %68, i32 %64
  %70 = add i32 %64, 8
  store i32 %70, ptr %63, align 16
  br label %75

71:                                               ; preds = %61
  %72 = getelementptr inbounds %struct.__va_list_tag, ptr %62, i32 0, i32 2
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr i8, ptr %73, i32 8
  store ptr %74, ptr %72, align 8
  br label %75

75:                                               ; preds = %71, %66
  %76 = phi ptr [ %69, %66 ], [ %73, %71 ]
  %77 = load i32, ptr %76, align 4
  store i32 %77, ptr %9, align 4
  store i64 1, ptr %10, align 8
  %78 = load ptr, ptr %8, align 8
  %79 = getelementptr inbounds %struct.global_State, ptr %78, i32 0, i32 16
  %80 = load i8, ptr %79, align 2
  store i8 %80, ptr %11, align 1
  %81 = load ptr, ptr %8, align 8
  %82 = getelementptr inbounds %struct.global_State, ptr %81, i32 0, i32 16
  store i8 0, ptr %82, align 2
  %83 = load i32, ptr %9, align 4
  %84 = icmp eq i32 %83, 0
  br i1 %84, label %85, label %88

85:                                               ; preds = %75
  %86 = load ptr, ptr %8, align 8
  call void @luaE_setdebt(ptr noundef %86, i64 noundef 0)
  %87 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %87)
  br label %107

88:                                               ; preds = %75
  %89 = load i32, ptr %9, align 4
  %90 = sext i32 %89 to i64
  %91 = mul nsw i64 %90, 1024
  %92 = load ptr, ptr %8, align 8
  %93 = getelementptr inbounds %struct.global_State, ptr %92, i32 0, i32 3
  %94 = load i64, ptr %93, align 8
  %95 = add nsw i64 %91, %94
  store i64 %95, ptr %10, align 8
  %96 = load ptr, ptr %8, align 8
  %97 = load i64, ptr %10, align 8
  call void @luaE_setdebt(ptr noundef %96, i64 noundef %97)
  %98 = load ptr, ptr %4, align 8
  %99 = getelementptr inbounds %struct.lua_State, ptr %98, i32 0, i32 7
  %100 = load ptr, ptr %99, align 8
  %101 = getelementptr inbounds %struct.global_State, ptr %100, i32 0, i32 3
  %102 = load i64, ptr %101, align 8
  %103 = icmp sgt i64 %102, 0
  br i1 %103, label %104, label %106

104:                                              ; preds = %88
  %105 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %105)
  br label %106

106:                                              ; preds = %104, %88
  br label %107

107:                                              ; preds = %106, %85
  %108 = load i8, ptr %11, align 1
  %109 = load ptr, ptr %8, align 8
  %110 = getelementptr inbounds %struct.global_State, ptr %109, i32 0, i32 16
  store i8 %108, ptr %110, align 2
  %111 = load i64, ptr %10, align 8
  %112 = icmp sgt i64 %111, 0
  br i1 %112, label %113, label %120

113:                                              ; preds = %107
  %114 = load ptr, ptr %8, align 8
  %115 = getelementptr inbounds %struct.global_State, ptr %114, i32 0, i32 11
  %116 = load i8, ptr %115, align 1
  %117 = zext i8 %116 to i32
  %118 = icmp eq i32 %117, 8
  br i1 %118, label %119, label %120

119:                                              ; preds = %113
  store i32 1, ptr %7, align 4
  br label %120

120:                                              ; preds = %119, %113, %107
  br label %338

121:                                              ; preds = %29
  %122 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %123 = getelementptr inbounds %struct.__va_list_tag, ptr %122, i32 0, i32 0
  %124 = load i32, ptr %123, align 16
  %125 = icmp ule i32 %124, 40
  br i1 %125, label %126, label %131

126:                                              ; preds = %121
  %127 = getelementptr inbounds %struct.__va_list_tag, ptr %122, i32 0, i32 3
  %128 = load ptr, ptr %127, align 16
  %129 = getelementptr i8, ptr %128, i32 %124
  %130 = add i32 %124, 8
  store i32 %130, ptr %123, align 16
  br label %135

131:                                              ; preds = %121
  %132 = getelementptr inbounds %struct.__va_list_tag, ptr %122, i32 0, i32 2
  %133 = load ptr, ptr %132, align 8
  %134 = getelementptr i8, ptr %133, i32 8
  store ptr %134, ptr %132, align 8
  br label %135

135:                                              ; preds = %131, %126
  %136 = phi ptr [ %129, %126 ], [ %133, %131 ]
  %137 = load i32, ptr %136, align 4
  store i32 %137, ptr %12, align 4
  %138 = load ptr, ptr %8, align 8
  %139 = getelementptr inbounds %struct.global_State, ptr %138, i32 0, i32 18
  %140 = load i8, ptr %139, align 4
  %141 = zext i8 %140 to i32
  %142 = mul nsw i32 %141, 4
  store i32 %142, ptr %7, align 4
  %143 = load i32, ptr %12, align 4
  %144 = sdiv i32 %143, 4
  %145 = trunc i32 %144 to i8
  %146 = load ptr, ptr %8, align 8
  %147 = getelementptr inbounds %struct.global_State, ptr %146, i32 0, i32 18
  store i8 %145, ptr %147, align 4
  br label %338

148:                                              ; preds = %29
  %149 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %150 = getelementptr inbounds %struct.__va_list_tag, ptr %149, i32 0, i32 0
  %151 = load i32, ptr %150, align 16
  %152 = icmp ule i32 %151, 40
  br i1 %152, label %153, label %158

153:                                              ; preds = %148
  %154 = getelementptr inbounds %struct.__va_list_tag, ptr %149, i32 0, i32 3
  %155 = load ptr, ptr %154, align 16
  %156 = getelementptr i8, ptr %155, i32 %151
  %157 = add i32 %151, 8
  store i32 %157, ptr %150, align 16
  br label %162

158:                                              ; preds = %148
  %159 = getelementptr inbounds %struct.__va_list_tag, ptr %149, i32 0, i32 2
  %160 = load ptr, ptr %159, align 8
  %161 = getelementptr i8, ptr %160, i32 8
  store ptr %161, ptr %159, align 8
  br label %162

162:                                              ; preds = %158, %153
  %163 = phi ptr [ %156, %153 ], [ %160, %158 ]
  %164 = load i32, ptr %163, align 4
  store i32 %164, ptr %13, align 4
  %165 = load ptr, ptr %8, align 8
  %166 = getelementptr inbounds %struct.global_State, ptr %165, i32 0, i32 19
  %167 = load i8, ptr %166, align 1
  %168 = zext i8 %167 to i32
  %169 = mul nsw i32 %168, 4
  store i32 %169, ptr %7, align 4
  %170 = load i32, ptr %13, align 4
  %171 = sdiv i32 %170, 4
  %172 = trunc i32 %171 to i8
  %173 = load ptr, ptr %8, align 8
  %174 = getelementptr inbounds %struct.global_State, ptr %173, i32 0, i32 19
  store i8 %172, ptr %174, align 1
  br label %338

175:                                              ; preds = %29
  %176 = load ptr, ptr %8, align 8
  %177 = getelementptr inbounds %struct.global_State, ptr %176, i32 0, i32 16
  %178 = load i8, ptr %177, align 2
  %179 = zext i8 %178 to i32
  %180 = icmp eq i32 %179, 0
  %181 = zext i1 %180 to i32
  store i32 %181, ptr %7, align 4
  br label %338

182:                                              ; preds = %29
  %183 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %184 = getelementptr inbounds %struct.__va_list_tag, ptr %183, i32 0, i32 0
  %185 = load i32, ptr %184, align 16
  %186 = icmp ule i32 %185, 40
  br i1 %186, label %187, label %192

187:                                              ; preds = %182
  %188 = getelementptr inbounds %struct.__va_list_tag, ptr %183, i32 0, i32 3
  %189 = load ptr, ptr %188, align 16
  %190 = getelementptr i8, ptr %189, i32 %185
  %191 = add i32 %185, 8
  store i32 %191, ptr %184, align 16
  br label %196

192:                                              ; preds = %182
  %193 = getelementptr inbounds %struct.__va_list_tag, ptr %183, i32 0, i32 2
  %194 = load ptr, ptr %193, align 8
  %195 = getelementptr i8, ptr %194, i32 8
  store ptr %195, ptr %193, align 8
  br label %196

196:                                              ; preds = %192, %187
  %197 = phi ptr [ %190, %187 ], [ %194, %192 ]
  %198 = load i32, ptr %197, align 4
  store i32 %198, ptr %14, align 4
  %199 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %200 = getelementptr inbounds %struct.__va_list_tag, ptr %199, i32 0, i32 0
  %201 = load i32, ptr %200, align 16
  %202 = icmp ule i32 %201, 40
  br i1 %202, label %203, label %208

203:                                              ; preds = %196
  %204 = getelementptr inbounds %struct.__va_list_tag, ptr %199, i32 0, i32 3
  %205 = load ptr, ptr %204, align 16
  %206 = getelementptr i8, ptr %205, i32 %201
  %207 = add i32 %201, 8
  store i32 %207, ptr %200, align 16
  br label %212

208:                                              ; preds = %196
  %209 = getelementptr inbounds %struct.__va_list_tag, ptr %199, i32 0, i32 2
  %210 = load ptr, ptr %209, align 8
  %211 = getelementptr i8, ptr %210, i32 8
  store ptr %211, ptr %209, align 8
  br label %212

212:                                              ; preds = %208, %203
  %213 = phi ptr [ %206, %203 ], [ %210, %208 ]
  %214 = load i32, ptr %213, align 4
  store i32 %214, ptr %15, align 4
  %215 = load ptr, ptr %8, align 8
  %216 = getelementptr inbounds %struct.global_State, ptr %215, i32 0, i32 12
  %217 = load i8, ptr %216, align 2
  %218 = zext i8 %217 to i32
  %219 = icmp eq i32 %218, 1
  br i1 %219, label %225, label %220

220:                                              ; preds = %212
  %221 = load ptr, ptr %8, align 8
  %222 = getelementptr inbounds %struct.global_State, ptr %221, i32 0, i32 5
  %223 = load i64, ptr %222, align 8
  %224 = icmp ne i64 %223, 0
  br label %225

225:                                              ; preds = %220, %212
  %226 = phi i1 [ true, %212 ], [ %224, %220 ]
  %227 = zext i1 %226 to i64
  %228 = select i1 %226, i32 10, i32 11
  store i32 %228, ptr %7, align 4
  %229 = load i32, ptr %14, align 4
  %230 = icmp ne i32 %229, 0
  br i1 %230, label %231, label %236

231:                                              ; preds = %225
  %232 = load i32, ptr %14, align 4
  %233 = trunc i32 %232 to i8
  %234 = load ptr, ptr %8, align 8
  %235 = getelementptr inbounds %struct.global_State, ptr %234, i32 0, i32 14
  store i8 %233, ptr %235, align 8
  br label %236

236:                                              ; preds = %231, %225
  %237 = load i32, ptr %15, align 4
  %238 = icmp ne i32 %237, 0
  br i1 %238, label %239, label %245

239:                                              ; preds = %236
  %240 = load i32, ptr %15, align 4
  %241 = sdiv i32 %240, 4
  %242 = trunc i32 %241 to i8
  %243 = load ptr, ptr %8, align 8
  %244 = getelementptr inbounds %struct.global_State, ptr %243, i32 0, i32 15
  store i8 %242, ptr %244, align 1
  br label %245

245:                                              ; preds = %239, %236
  %246 = load ptr, ptr %4, align 8
  call void @luaC_changemode(ptr noundef %246, i32 noundef 1)
  br label %338

247:                                              ; preds = %29
  %248 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %249 = getelementptr inbounds %struct.__va_list_tag, ptr %248, i32 0, i32 0
  %250 = load i32, ptr %249, align 16
  %251 = icmp ule i32 %250, 40
  br i1 %251, label %252, label %257

252:                                              ; preds = %247
  %253 = getelementptr inbounds %struct.__va_list_tag, ptr %248, i32 0, i32 3
  %254 = load ptr, ptr %253, align 16
  %255 = getelementptr i8, ptr %254, i32 %250
  %256 = add i32 %250, 8
  store i32 %256, ptr %249, align 16
  br label %261

257:                                              ; preds = %247
  %258 = getelementptr inbounds %struct.__va_list_tag, ptr %248, i32 0, i32 2
  %259 = load ptr, ptr %258, align 8
  %260 = getelementptr i8, ptr %259, i32 8
  store ptr %260, ptr %258, align 8
  br label %261

261:                                              ; preds = %257, %252
  %262 = phi ptr [ %255, %252 ], [ %259, %257 ]
  %263 = load i32, ptr %262, align 4
  store i32 %263, ptr %16, align 4
  %264 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %265 = getelementptr inbounds %struct.__va_list_tag, ptr %264, i32 0, i32 0
  %266 = load i32, ptr %265, align 16
  %267 = icmp ule i32 %266, 40
  br i1 %267, label %268, label %273

268:                                              ; preds = %261
  %269 = getelementptr inbounds %struct.__va_list_tag, ptr %264, i32 0, i32 3
  %270 = load ptr, ptr %269, align 16
  %271 = getelementptr i8, ptr %270, i32 %266
  %272 = add i32 %266, 8
  store i32 %272, ptr %265, align 16
  br label %277

273:                                              ; preds = %261
  %274 = getelementptr inbounds %struct.__va_list_tag, ptr %264, i32 0, i32 2
  %275 = load ptr, ptr %274, align 8
  %276 = getelementptr i8, ptr %275, i32 8
  store ptr %276, ptr %274, align 8
  br label %277

277:                                              ; preds = %273, %268
  %278 = phi ptr [ %271, %268 ], [ %275, %273 ]
  %279 = load i32, ptr %278, align 4
  store i32 %279, ptr %17, align 4
  %280 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %281 = getelementptr inbounds %struct.__va_list_tag, ptr %280, i32 0, i32 0
  %282 = load i32, ptr %281, align 16
  %283 = icmp ule i32 %282, 40
  br i1 %283, label %284, label %289

284:                                              ; preds = %277
  %285 = getelementptr inbounds %struct.__va_list_tag, ptr %280, i32 0, i32 3
  %286 = load ptr, ptr %285, align 16
  %287 = getelementptr i8, ptr %286, i32 %282
  %288 = add i32 %282, 8
  store i32 %288, ptr %281, align 16
  br label %293

289:                                              ; preds = %277
  %290 = getelementptr inbounds %struct.__va_list_tag, ptr %280, i32 0, i32 2
  %291 = load ptr, ptr %290, align 8
  %292 = getelementptr i8, ptr %291, i32 8
  store ptr %292, ptr %290, align 8
  br label %293

293:                                              ; preds = %289, %284
  %294 = phi ptr [ %287, %284 ], [ %291, %289 ]
  %295 = load i32, ptr %294, align 4
  store i32 %295, ptr %18, align 4
  %296 = load ptr, ptr %8, align 8
  %297 = getelementptr inbounds %struct.global_State, ptr %296, i32 0, i32 12
  %298 = load i8, ptr %297, align 2
  %299 = zext i8 %298 to i32
  %300 = icmp eq i32 %299, 1
  br i1 %300, label %306, label %301

301:                                              ; preds = %293
  %302 = load ptr, ptr %8, align 8
  %303 = getelementptr inbounds %struct.global_State, ptr %302, i32 0, i32 5
  %304 = load i64, ptr %303, align 8
  %305 = icmp ne i64 %304, 0
  br label %306

306:                                              ; preds = %301, %293
  %307 = phi i1 [ true, %293 ], [ %305, %301 ]
  %308 = zext i1 %307 to i64
  %309 = select i1 %307, i32 10, i32 11
  store i32 %309, ptr %7, align 4
  %310 = load i32, ptr %16, align 4
  %311 = icmp ne i32 %310, 0
  br i1 %311, label %312, label %318

312:                                              ; preds = %306
  %313 = load i32, ptr %16, align 4
  %314 = sdiv i32 %313, 4
  %315 = trunc i32 %314 to i8
  %316 = load ptr, ptr %8, align 8
  %317 = getelementptr inbounds %struct.global_State, ptr %316, i32 0, i32 18
  store i8 %315, ptr %317, align 4
  br label %318

318:                                              ; preds = %312, %306
  %319 = load i32, ptr %17, align 4
  %320 = icmp ne i32 %319, 0
  br i1 %320, label %321, label %327

321:                                              ; preds = %318
  %322 = load i32, ptr %17, align 4
  %323 = sdiv i32 %322, 4
  %324 = trunc i32 %323 to i8
  %325 = load ptr, ptr %8, align 8
  %326 = getelementptr inbounds %struct.global_State, ptr %325, i32 0, i32 19
  store i8 %324, ptr %326, align 1
  br label %327

327:                                              ; preds = %321, %318
  %328 = load i32, ptr %18, align 4
  %329 = icmp ne i32 %328, 0
  br i1 %329, label %330, label %335

330:                                              ; preds = %327
  %331 = load i32, ptr %18, align 4
  %332 = trunc i32 %331 to i8
  %333 = load ptr, ptr %8, align 8
  %334 = getelementptr inbounds %struct.global_State, ptr %333, i32 0, i32 20
  store i8 %332, ptr %334, align 2
  br label %335

335:                                              ; preds = %330, %327
  %336 = load ptr, ptr %4, align 8
  call void @luaC_changemode(ptr noundef %336, i32 noundef 0)
  br label %338

337:                                              ; preds = %29
  store i32 -1, ptr %7, align 4
  br label %338

338:                                              ; preds = %337, %335, %245, %175, %162, %135, %120, %51, %41, %39, %35, %32
  %339 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_end(ptr %339)
  %340 = load i32, ptr %7, align 4
  store i32 %340, ptr %3, align 4
  br label %341

341:                                              ; preds = %338, %28
  %342 = load i32, ptr %3, align 4
  ret i32 %342
}

declare hidden void @luaE_setdebt(ptr noundef, i64 noundef) #1

declare hidden void @luaC_fullgc(ptr noundef, i32 noundef) #1

declare hidden void @luaC_changemode(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_error(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 6
  %6 = load ptr, ptr %5, align 8
  %7 = getelementptr inbounds %union.StackValue, ptr %6, i64 -1
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 68
  br i1 %13, label %14, label %26

14:                                               ; preds = %1
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 7
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 41
  %22 = load ptr, ptr %21, align 8
  %23 = icmp eq ptr %17, %22
  br i1 %23, label %24, label %26

24:                                               ; preds = %14
  %25 = load ptr, ptr %2, align 8
  call void @luaD_throw(ptr noundef %25, i32 noundef 4) #5
  unreachable

26:                                               ; preds = %14, %1
  %27 = load ptr, ptr %2, align 8
  call void @luaG_errormsg(ptr noundef %27) #5
  unreachable
}

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #4

; Function Attrs: noreturn
declare hidden void @luaG_errormsg(ptr noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_next(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load i32, ptr %4, align 4
  %10 = call ptr @gettable(ptr noundef %8, i32 noundef %9)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %union.StackValue, ptr %15, i64 -1
  %17 = call i32 @luaH_next(ptr noundef %11, ptr noundef %12, ptr noundef %16)
  store i32 %17, ptr %6, align 4
  %18 = load i32, ptr %6, align 4
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %26

20:                                               ; preds = %2
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 6
  %23 = load ptr, ptr %22, align 8
  %24 = getelementptr inbounds %union.StackValue, ptr %23, i32 1
  store ptr %24, ptr %22, align 8
  %25 = load ptr, ptr %3, align 8
  br label %31

26:                                               ; preds = %2
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 6
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %union.StackValue, ptr %29, i64 -1
  store ptr %30, ptr %28, align 8
  br label %31

31:                                               ; preds = %26, %20
  %32 = load i32, ptr %6, align 4
  ret i32 %32
}

declare hidden i32 @luaH_next(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_toclose(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = call ptr @index2stack(ptr noundef %7, i32 noundef %8)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 8
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.CallInfo, ptr %12, i32 0, i32 6
  %14 = load i16, ptr %13, align 4
  %15 = sext i16 %14 to i32
  store i32 %15, ptr %5, align 4
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %6, align 8
  call void @luaF_newtbcupval(ptr noundef %17, ptr noundef %18)
  %19 = load i32, ptr %5, align 4
  %20 = icmp slt i32 %19, -1
  br i1 %20, label %30, label %21

21:                                               ; preds = %2
  %22 = load i32, ptr %5, align 4
  %23 = sub nsw i32 0, %22
  %24 = sub nsw i32 %23, 3
  %25 = trunc i32 %24 to i16
  %26 = load ptr, ptr %3, align 8
  %27 = getelementptr inbounds %struct.lua_State, ptr %26, i32 0, i32 8
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.CallInfo, ptr %28, i32 0, i32 6
  store i16 %25, ptr %29, align 4
  br label %30

30:                                               ; preds = %21, %2
  ret void
}

declare hidden void @luaF_newtbcupval(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_concat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = icmp sgt i32 %8, 0
  br i1 %9, label %10, label %13

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8
  %12 = load i32, ptr %4, align 4
  call void @luaV_concat(ptr noundef %11, i32 noundef %12)
  br label %36

13:                                               ; preds = %2
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %5, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = call ptr @luaS_newlstr(ptr noundef %17, ptr noundef @.str, i64 noundef 0)
  store ptr %18, ptr %6, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  store ptr %19, ptr %21, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = getelementptr inbounds %struct.TString, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = or i32 %25, 64
  %27 = trunc i32 %26 to i8
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  store i8 %27, ptr %29, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 6
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %union.StackValue, ptr %33, i32 1
  store ptr %34, ptr %32, align 8
  %35 = load ptr, ptr %3, align 8
  br label %36

36:                                               ; preds = %13, %10
  %37 = load ptr, ptr %3, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 7
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 3
  %41 = load i64, ptr %40, align 8
  %42 = icmp sgt i64 %41, 0
  br i1 %42, label %43, label %45

43:                                               ; preds = %36
  %44 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %44)
  br label %45

45:                                               ; preds = %43, %36
  ret void
}

declare hidden void @luaV_concat(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_len(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call ptr @index2value(ptr noundef %6, i32 noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 6
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %5, align 8
  call void @luaV_objlen(ptr noundef %9, ptr noundef %12, ptr noundef %13)
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load ptr, ptr %3, align 8
  ret void
}

declare hidden void @luaV_objlen(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_getallocf(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = icmp ne ptr %6, null
  br i1 %7, label %8, label %15

8:                                                ; preds = %2
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 7
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 1
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %4, align 8
  store ptr %13, ptr %14, align 8
  br label %15

15:                                               ; preds = %8, %2
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 7
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  store ptr %20, ptr %5, align 8
  %21 = load ptr, ptr %5, align 8
  ret ptr %21
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_setallocf(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 1
  store ptr %7, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 0
  store ptr %12, ptr %16, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_setwarnf(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 46
  store ptr %7, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 45
  store ptr %12, ptr %16, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_warning(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  call void @luaE_warning(ptr noundef %7, ptr noundef %8, i32 noundef %9)
  ret void
}

declare hidden void @luaE_warning(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_newuserdatauv(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load i64, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = call ptr @luaS_newudata(ptr noundef %11, i64 noundef %12, i32 noundef %13)
  store ptr %14, ptr %7, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %8, align 8
  %18 = load ptr, ptr %7, align 8
  store ptr %18, ptr %9, align 8
  %19 = load ptr, ptr %9, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  store ptr %19, ptr %21, align 8
  %22 = load ptr, ptr %8, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  store i8 71, ptr %23, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %union.StackValue, ptr %27, i32 1
  store ptr %28, ptr %26, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 7
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 3
  %34 = load i64, ptr %33, align 8
  %35 = icmp sgt i64 %34, 0
  br i1 %35, label %36, label %38

36:                                               ; preds = %3
  %37 = load ptr, ptr %4, align 8
  call void @luaC_step(ptr noundef %37)
  br label %38

38:                                               ; preds = %36, %3
  %39 = load ptr, ptr %7, align 8
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.Udata, ptr %40, i32 0, i32 3
  %42 = load i16, ptr %41, align 2
  %43 = zext i16 %42 to i32
  %44 = icmp eq i32 %43, 0
  br i1 %44, label %45, label %46

45:                                               ; preds = %38
  br label %53

46:                                               ; preds = %38
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.Udata, ptr %47, i32 0, i32 3
  %49 = load i16, ptr %48, align 2
  %50 = zext i16 %49 to i64
  %51 = mul i64 16, %50
  %52 = add i64 40, %51
  br label %53

53:                                               ; preds = %46, %45
  %54 = phi i64 [ 32, %45 ], [ %52, %46 ]
  %55 = getelementptr inbounds i8, ptr %39, i64 %54
  ret ptr %55
}

declare hidden ptr @luaS_newudata(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_getupvalue(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  store ptr null, ptr %8, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = call ptr @index2value(ptr noundef %11, i32 noundef %12)
  %14 = load i32, ptr %6, align 4
  %15 = call ptr @aux_upvalue(ptr noundef %13, i32 noundef %14, ptr noundef %8, ptr noundef null)
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = icmp ne ptr %16, null
  br i1 %17, label %18, label %38

18:                                               ; preds = %3
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %9, align 8
  %22 = load ptr, ptr %8, align 8
  store ptr %22, ptr %10, align 8
  %23 = load ptr, ptr %9, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %10, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %24, ptr align 8 %26, i64 8, i1 false)
  %27 = load ptr, ptr %10, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = load ptr, ptr %9, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 %29, ptr %31, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 6
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i32 1
  store ptr %36, ptr %34, align 8
  %37 = load ptr, ptr %4, align 8
  br label %38

38:                                               ; preds = %18, %3
  %39 = load ptr, ptr %7, align 8
  ret ptr %39
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @aux_upvalue(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 1
  %16 = load i8, ptr %15, align 8
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 63
  switch i32 %18, label %100 [
    i32 38, label %19
    i32 6, label %45
  ]

19:                                               ; preds = %4
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr %10, align 8
  %23 = load i32, ptr %7, align 4
  %24 = sub i32 %23, 1
  %25 = load ptr, ptr %10, align 8
  %26 = getelementptr inbounds %struct.CClosure, ptr %25, i32 0, i32 3
  %27 = load i8, ptr %26, align 2
  %28 = zext i8 %27 to i32
  %29 = icmp ult i32 %24, %28
  br i1 %29, label %31, label %30

30:                                               ; preds = %19
  store ptr null, ptr %5, align 8
  br label %101

31:                                               ; preds = %19
  %32 = load ptr, ptr %10, align 8
  %33 = getelementptr inbounds %struct.CClosure, ptr %32, i32 0, i32 6
  %34 = load i32, ptr %7, align 4
  %35 = sub nsw i32 %34, 1
  %36 = sext i32 %35 to i64
  %37 = getelementptr inbounds [1 x %struct.TValue], ptr %33, i64 0, i64 %36
  %38 = load ptr, ptr %8, align 8
  store ptr %37, ptr %38, align 8
  %39 = load ptr, ptr %9, align 8
  %40 = icmp ne ptr %39, null
  br i1 %40, label %41, label %44

41:                                               ; preds = %31
  %42 = load ptr, ptr %10, align 8
  %43 = load ptr, ptr %9, align 8
  store ptr %42, ptr %43, align 8
  br label %44

44:                                               ; preds = %41, %31
  store ptr @.str, ptr %5, align 8
  br label %101

45:                                               ; preds = %4
  %46 = load ptr, ptr %6, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  store ptr %48, ptr %11, align 8
  %49 = load ptr, ptr %11, align 8
  %50 = getelementptr inbounds %struct.LClosure, ptr %49, i32 0, i32 5
  %51 = load ptr, ptr %50, align 8
  store ptr %51, ptr %13, align 8
  %52 = load i32, ptr %7, align 4
  %53 = sub i32 %52, 1
  %54 = load ptr, ptr %13, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 6
  %56 = load i32, ptr %55, align 8
  %57 = icmp ult i32 %53, %56
  br i1 %57, label %59, label %58

58:                                               ; preds = %45
  store ptr null, ptr %5, align 8
  br label %101

59:                                               ; preds = %45
  %60 = load ptr, ptr %11, align 8
  %61 = getelementptr inbounds %struct.LClosure, ptr %60, i32 0, i32 6
  %62 = load i32, ptr %7, align 4
  %63 = sub nsw i32 %62, 1
  %64 = sext i32 %63 to i64
  %65 = getelementptr inbounds [1 x ptr], ptr %61, i64 0, i64 %64
  %66 = load ptr, ptr %65, align 8
  %67 = getelementptr inbounds %struct.UpVal, ptr %66, i32 0, i32 3
  %68 = load ptr, ptr %67, align 8
  %69 = load ptr, ptr %8, align 8
  store ptr %68, ptr %69, align 8
  %70 = load ptr, ptr %9, align 8
  %71 = icmp ne ptr %70, null
  br i1 %71, label %72, label %81

72:                                               ; preds = %59
  %73 = load ptr, ptr %11, align 8
  %74 = getelementptr inbounds %struct.LClosure, ptr %73, i32 0, i32 6
  %75 = load i32, ptr %7, align 4
  %76 = sub nsw i32 %75, 1
  %77 = sext i32 %76 to i64
  %78 = getelementptr inbounds [1 x ptr], ptr %74, i64 0, i64 %77
  %79 = load ptr, ptr %78, align 8
  %80 = load ptr, ptr %9, align 8
  store ptr %79, ptr %80, align 8
  br label %81

81:                                               ; preds = %72, %59
  %82 = load ptr, ptr %13, align 8
  %83 = getelementptr inbounds %struct.Proto, ptr %82, i32 0, i32 18
  %84 = load ptr, ptr %83, align 8
  %85 = load i32, ptr %7, align 4
  %86 = sub nsw i32 %85, 1
  %87 = sext i32 %86 to i64
  %88 = getelementptr inbounds %struct.Upvaldesc, ptr %84, i64 %87
  %89 = getelementptr inbounds %struct.Upvaldesc, ptr %88, i32 0, i32 0
  %90 = load ptr, ptr %89, align 8
  store ptr %90, ptr %12, align 8
  %91 = load ptr, ptr %12, align 8
  %92 = icmp eq ptr %91, null
  br i1 %92, label %93, label %94

93:                                               ; preds = %81
  br label %98

94:                                               ; preds = %81
  %95 = load ptr, ptr %12, align 8
  %96 = getelementptr inbounds %struct.TString, ptr %95, i32 0, i32 7
  %97 = getelementptr inbounds [1 x i8], ptr %96, i64 0, i64 0
  br label %98

98:                                               ; preds = %94, %93
  %99 = phi ptr [ @.str.2, %93 ], [ %97, %94 ]
  store ptr %99, ptr %5, align 8
  br label %101

100:                                              ; preds = %4
  store ptr null, ptr %5, align 8
  br label %101

101:                                              ; preds = %100, %98, %58, %44, %30
  %102 = load ptr, ptr %5, align 8
  ret ptr %102
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_setupvalue(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  store ptr null, ptr %8, align 8
  store ptr null, ptr %9, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call ptr @index2value(ptr noundef %13, i32 noundef %14)
  store ptr %15, ptr %10, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %10, align 8
  %18 = load i32, ptr %6, align 4
  %19 = call ptr @aux_upvalue(ptr noundef %17, i32 noundef %18, ptr noundef %8, ptr noundef %9)
  store ptr %19, ptr %7, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %73

22:                                               ; preds = %3
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i32 -1
  store ptr %26, ptr %24, align 8
  %27 = load ptr, ptr %8, align 8
  store ptr %27, ptr %11, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 6
  %30 = load ptr, ptr %29, align 8
  store ptr %30, ptr %12, align 8
  %31 = load ptr, ptr %11, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %12, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %32, ptr align 8 %34, i64 8, i1 false)
  %35 = load ptr, ptr %12, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = load ptr, ptr %11, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 1
  store i8 %37, ptr %39, align 8
  %40 = load ptr, ptr %4, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 1
  %43 = load i8, ptr %42, align 8
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 64
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %71

47:                                               ; preds = %22
  %48 = load ptr, ptr %9, align 8
  %49 = getelementptr inbounds %struct.GCObject, ptr %48, i32 0, i32 2
  %50 = load i8, ptr %49, align 1
  %51 = zext i8 %50 to i32
  %52 = and i32 %51, 32
  %53 = icmp ne i32 %52, 0
  br i1 %53, label %54, label %69

54:                                               ; preds = %47
  %55 = load ptr, ptr %8, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 0
  %57 = load ptr, ptr %56, align 8
  %58 = getelementptr inbounds %struct.GCObject, ptr %57, i32 0, i32 2
  %59 = load i8, ptr %58, align 1
  %60 = zext i8 %59 to i32
  %61 = and i32 %60, 24
  %62 = icmp ne i32 %61, 0
  br i1 %62, label %63, label %69

63:                                               ; preds = %54
  %64 = load ptr, ptr %4, align 8
  %65 = load ptr, ptr %9, align 8
  %66 = load ptr, ptr %8, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 0
  %68 = load ptr, ptr %67, align 8
  call void @luaC_barrier_(ptr noundef %64, ptr noundef %65, ptr noundef %68)
  br label %70

69:                                               ; preds = %54, %47
  br label %70

70:                                               ; preds = %69, %63
  br label %72

71:                                               ; preds = %22
  br label %72

72:                                               ; preds = %71, %70
  br label %73

73:                                               ; preds = %72, %3
  %74 = load ptr, ptr %7, align 8
  ret ptr %74
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_upvalueid(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = load i32, ptr %6, align 4
  %12 = call ptr @index2value(ptr noundef %10, i32 noundef %11)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 63
  switch i32 %17, label %46 [
    i32 6, label %18
    i32 38, label %24
    i32 22, label %45
  ]

18:                                               ; preds = %3
  %19 = load ptr, ptr %5, align 8
  %20 = load i32, ptr %6, align 4
  %21 = load i32, ptr %7, align 4
  %22 = call ptr @getupvalref(ptr noundef %19, i32 noundef %20, i32 noundef %21, ptr noundef null)
  %23 = load ptr, ptr %22, align 8
  store ptr %23, ptr %4, align 8
  br label %48

24:                                               ; preds = %3
  %25 = load ptr, ptr %8, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  store ptr %27, ptr %9, align 8
  %28 = load i32, ptr %7, align 4
  %29 = icmp sle i32 1, %28
  br i1 %29, label %30, label %44

30:                                               ; preds = %24
  %31 = load i32, ptr %7, align 4
  %32 = load ptr, ptr %9, align 8
  %33 = getelementptr inbounds %struct.CClosure, ptr %32, i32 0, i32 3
  %34 = load i8, ptr %33, align 2
  %35 = zext i8 %34 to i32
  %36 = icmp sle i32 %31, %35
  br i1 %36, label %37, label %44

37:                                               ; preds = %30
  %38 = load ptr, ptr %9, align 8
  %39 = getelementptr inbounds %struct.CClosure, ptr %38, i32 0, i32 6
  %40 = load i32, ptr %7, align 4
  %41 = sub nsw i32 %40, 1
  %42 = sext i32 %41 to i64
  %43 = getelementptr inbounds [1 x %struct.TValue], ptr %39, i64 0, i64 %42
  store ptr %43, ptr %4, align 8
  br label %48

44:                                               ; preds = %30, %24
  br label %45

45:                                               ; preds = %3, %44
  store ptr null, ptr %4, align 8
  br label %48

46:                                               ; preds = %3
  %47 = load ptr, ptr %5, align 8
  store ptr null, ptr %4, align 8
  br label %48

48:                                               ; preds = %46, %45, %37, %18
  %49 = load ptr, ptr %4, align 8
  ret ptr %49
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getupvalref(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = load i32, ptr %7, align 4
  %14 = call ptr @index2value(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %11, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = load ptr, ptr %11, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %10, align 8
  %19 = load ptr, ptr %9, align 8
  %20 = icmp ne ptr %19, null
  br i1 %20, label %21, label %24

21:                                               ; preds = %4
  %22 = load ptr, ptr %10, align 8
  %23 = load ptr, ptr %9, align 8
  store ptr %22, ptr %23, align 8
  br label %24

24:                                               ; preds = %21, %4
  %25 = load i32, ptr %8, align 4
  %26 = icmp sle i32 1, %25
  br i1 %26, label %27, label %42

27:                                               ; preds = %24
  %28 = load i32, ptr %8, align 4
  %29 = load ptr, ptr %10, align 8
  %30 = getelementptr inbounds %struct.LClosure, ptr %29, i32 0, i32 5
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.Proto, ptr %31, i32 0, i32 6
  %33 = load i32, ptr %32, align 8
  %34 = icmp sle i32 %28, %33
  br i1 %34, label %35, label %42

35:                                               ; preds = %27
  %36 = load ptr, ptr %10, align 8
  %37 = getelementptr inbounds %struct.LClosure, ptr %36, i32 0, i32 6
  %38 = load i32, ptr %8, align 4
  %39 = sub nsw i32 %38, 1
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds [1 x ptr], ptr %37, i64 0, i64 %40
  store ptr %41, ptr %5, align 8
  br label %43

42:                                               ; preds = %27, %24
  store ptr @getupvalref.nullup, ptr %5, align 8
  br label %43

43:                                               ; preds = %42, %35
  %44 = load ptr, ptr %5, align 8
  ret ptr %44
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_upvaluejoin(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %14 = load ptr, ptr %6, align 8
  %15 = load i32, ptr %7, align 4
  %16 = load i32, ptr %8, align 4
  %17 = call ptr @getupvalref(ptr noundef %14, i32 noundef %15, i32 noundef %16, ptr noundef %11)
  store ptr %17, ptr %12, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = load i32, ptr %9, align 4
  %20 = load i32, ptr %10, align 4
  %21 = call ptr @getupvalref(ptr noundef %18, i32 noundef %19, i32 noundef %20, ptr noundef null)
  store ptr %21, ptr %13, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %13, align 8
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %12, align 8
  store ptr %24, ptr %25, align 8
  %26 = load ptr, ptr %11, align 8
  %27 = getelementptr inbounds %struct.LClosure, ptr %26, i32 0, i32 2
  %28 = load i8, ptr %27, align 1
  %29 = zext i8 %28 to i32
  %30 = and i32 %29, 32
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %45

32:                                               ; preds = %5
  %33 = load ptr, ptr %12, align 8
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %struct.UpVal, ptr %34, i32 0, i32 2
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 24
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %45

40:                                               ; preds = %32
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %11, align 8
  %43 = load ptr, ptr %12, align 8
  %44 = load ptr, ptr %43, align 8
  call void @luaC_barrier_(ptr noundef %41, ptr noundef %42, ptr noundef %44)
  br label %46

45:                                               ; preds = %32, %5
  br label %46

46:                                               ; preds = %45, %40
  ret void
}

declare hidden ptr @luaH_getstr(ptr noundef, ptr noundef) #1

declare hidden void @luaH_set(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { nocallback nofree nosync nounwind willreturn }
attributes #4 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { noreturn }

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
