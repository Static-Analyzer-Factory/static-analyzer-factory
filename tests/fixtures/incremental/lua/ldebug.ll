; ModuleID = 'ldebug.c'
source_filename = "ldebug.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.AbsLineInfo = type { i32, i32 }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.1, i16, i16 }
%union.anon = type { %struct.anon.0 }
%struct.anon.0 = type { ptr, i64, i64 }
%union.anon.1 = type { i32 }
%struct.anon = type { ptr, i32, i32 }
%struct.lua_Debug = type { i32, ptr, ptr, ptr, ptr, i64, i32, i32, i32, i8, i8, i8, i8, i16, i16, [60 x i8], ptr }
%union.StackValue = type { %struct.TValue }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.CClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x %struct.TValue] }
%struct.anon.2 = type { i16, i16 }
%struct.__va_list_tag = type { i32, i32, ptr, ptr }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%struct.UpVal = type { ptr, i8, i8, %union.anon.5, %union.anon.6 }
%union.anon.5 = type { ptr }
%union.anon.6 = type { %struct.anon.7 }
%struct.anon.7 = type { ptr, ptr }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }

@.str = private unnamed_addr constant [12 x i8] c"(temporary)\00", align 1
@.str.1 = private unnamed_addr constant [14 x i8] c"(C temporary)\00", align 1
@.str.2 = private unnamed_addr constant [5 x i8] c"call\00", align 1
@.str.3 = private unnamed_addr constant [39 x i8] c"bad 'for' %s (number expected, got %s)\00", align 1
@.str.4 = private unnamed_addr constant [12 x i8] c"concatenate\00", align 1
@.str.5 = private unnamed_addr constant [39 x i8] c"number%s has no integer representation\00", align 1
@.str.6 = private unnamed_addr constant [33 x i8] c"attempt to compare two %s values\00", align 1
@.str.7 = private unnamed_addr constant [30 x i8] c"attempt to compare %s with %s\00", align 1
@.str.8 = private unnamed_addr constant [10 x i8] c"%s:%d: %s\00", align 1
@luaP_opmodes = external hidden constant [83 x i8], align 16
@.str.9 = private unnamed_addr constant [9 x i8] c"(vararg)\00", align 1
@.str.10 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.11 = private unnamed_addr constant [5 x i8] c"=[C]\00", align 1
@.str.12 = private unnamed_addr constant [2 x i8] c"C\00", align 1
@.str.13 = private unnamed_addr constant [3 x i8] c"=?\00", align 1
@.str.14 = private unnamed_addr constant [5 x i8] c"main\00", align 1
@.str.15 = private unnamed_addr constant [4 x i8] c"Lua\00", align 1
@.str.16 = private unnamed_addr constant [27 x i8] c"attempt to %s a %s value%s\00", align 1
@.str.17 = private unnamed_addr constant [8 x i8] c"upvalue\00", align 1
@.str.18 = private unnamed_addr constant [2 x i8] c"?\00", align 1
@.str.19 = private unnamed_addr constant [14 x i8] c"integer index\00", align 1
@.str.20 = private unnamed_addr constant [6 x i8] c"field\00", align 1
@.str.21 = private unnamed_addr constant [7 x i8] c"method\00", align 1
@.str.22 = private unnamed_addr constant [6 x i8] c"local\00", align 1
@.str.23 = private unnamed_addr constant [9 x i8] c"constant\00", align 1
@.str.24 = private unnamed_addr constant [5 x i8] c"_ENV\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"global\00", align 1
@.str.26 = private unnamed_addr constant [5 x i8] c"hook\00", align 1
@.str.27 = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@.str.28 = private unnamed_addr constant [11 x i8] c"metamethod\00", align 1
@.str.29 = private unnamed_addr constant [13 x i8] c"for iterator\00", align 1
@.str.30 = private unnamed_addr constant [11 x i8] c" (%s '%s')\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaG_getfuncline(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Proto, ptr %8, i32 0, i32 19
  %10 = load ptr, ptr %9, align 8
  %11 = icmp eq ptr %10, null
  br i1 %11, label %12, label %13

12:                                               ; preds = %2
  store i32 -1, ptr %3, align 4
  br label %35

13:                                               ; preds = %2
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call i32 @getbaseline(ptr noundef %14, i32 noundef %15, ptr noundef %6)
  store i32 %16, ptr %7, align 4
  br label %17

17:                                               ; preds = %22, %13
  %18 = load i32, ptr %6, align 4
  %19 = add nsw i32 %18, 1
  store i32 %19, ptr %6, align 4
  %20 = load i32, ptr %5, align 4
  %21 = icmp slt i32 %18, %20
  br i1 %21, label %22, label %33

22:                                               ; preds = %17
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.Proto, ptr %23, i32 0, i32 19
  %25 = load ptr, ptr %24, align 8
  %26 = load i32, ptr %6, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds i8, ptr %25, i64 %27
  %29 = load i8, ptr %28, align 1
  %30 = sext i8 %29 to i32
  %31 = load i32, ptr %7, align 4
  %32 = add nsw i32 %31, %30
  store i32 %32, ptr %7, align 4
  br label %17, !llvm.loop !6

33:                                               ; preds = %17
  %34 = load i32, ptr %7, align 4
  store i32 %34, ptr %3, align 4
  br label %35

35:                                               ; preds = %33, %12
  %36 = load i32, ptr %3, align 4
  ret i32 %36
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getbaseline(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 12
  %11 = load i32, ptr %10, align 8
  %12 = icmp eq i32 %11, 0
  br i1 %12, label %22, label %13

13:                                               ; preds = %3
  %14 = load i32, ptr %6, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.Proto, ptr %15, i32 0, i32 20
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.AbsLineInfo, ptr %17, i64 0
  %19 = getelementptr inbounds %struct.AbsLineInfo, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 4
  %21 = icmp slt i32 %14, %20
  br i1 %21, label %22, label %27

22:                                               ; preds = %13, %3
  %23 = load ptr, ptr %7, align 8
  store i32 -1, ptr %23, align 4
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 13
  %26 = load i32, ptr %25, align 4
  store i32 %26, ptr %4, align 4
  br label %73

27:                                               ; preds = %13
  %28 = load i32, ptr %6, align 4
  %29 = udiv i32 %28, 128
  %30 = sub i32 %29, 1
  store i32 %30, ptr %8, align 4
  br label %31

31:                                               ; preds = %52, %27
  %32 = load i32, ptr %8, align 4
  %33 = add nsw i32 %32, 1
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 12
  %36 = load i32, ptr %35, align 8
  %37 = icmp slt i32 %33, %36
  br i1 %37, label %38, label %50

38:                                               ; preds = %31
  %39 = load i32, ptr %6, align 4
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.Proto, ptr %40, i32 0, i32 20
  %42 = load ptr, ptr %41, align 8
  %43 = load i32, ptr %8, align 4
  %44 = add nsw i32 %43, 1
  %45 = sext i32 %44 to i64
  %46 = getelementptr inbounds %struct.AbsLineInfo, ptr %42, i64 %45
  %47 = getelementptr inbounds %struct.AbsLineInfo, ptr %46, i32 0, i32 0
  %48 = load i32, ptr %47, align 4
  %49 = icmp sge i32 %39, %48
  br label %50

50:                                               ; preds = %38, %31
  %51 = phi i1 [ false, %31 ], [ %49, %38 ]
  br i1 %51, label %52, label %55

52:                                               ; preds = %50
  %53 = load i32, ptr %8, align 4
  %54 = add nsw i32 %53, 1
  store i32 %54, ptr %8, align 4
  br label %31, !llvm.loop !8

55:                                               ; preds = %50
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.Proto, ptr %56, i32 0, i32 20
  %58 = load ptr, ptr %57, align 8
  %59 = load i32, ptr %8, align 4
  %60 = sext i32 %59 to i64
  %61 = getelementptr inbounds %struct.AbsLineInfo, ptr %58, i64 %60
  %62 = getelementptr inbounds %struct.AbsLineInfo, ptr %61, i32 0, i32 0
  %63 = load i32, ptr %62, align 4
  %64 = load ptr, ptr %7, align 8
  store i32 %63, ptr %64, align 4
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.Proto, ptr %65, i32 0, i32 20
  %67 = load ptr, ptr %66, align 8
  %68 = load i32, ptr %8, align 4
  %69 = sext i32 %68 to i64
  %70 = getelementptr inbounds %struct.AbsLineInfo, ptr %67, i64 %69
  %71 = getelementptr inbounds %struct.AbsLineInfo, ptr %70, i32 0, i32 1
  %72 = load i32, ptr %71, align 4
  store i32 %72, ptr %4, align 4
  br label %73

73:                                               ; preds = %55, %22
  %74 = load i32, ptr %4, align 4
  ret i32 %74
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @lua_sethook(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %6, align 8
  %10 = icmp eq ptr %9, null
  br i1 %10, label %14, label %11

11:                                               ; preds = %4
  %12 = load i32, ptr %7, align 4
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %14, label %15

14:                                               ; preds = %11, %4
  store i32 0, ptr %7, align 4
  store ptr null, ptr %6, align 8
  br label %15

15:                                               ; preds = %14, %11
  %16 = load ptr, ptr %6, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 17
  store volatile ptr %16, ptr %18, align 8
  %19 = load i32, ptr %8, align 4
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 21
  store i32 %19, ptr %21, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 21
  %24 = load i32, ptr %23, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 22
  store i32 %24, ptr %26, align 4
  %27 = load i32, ptr %7, align 4
  %28 = trunc i32 %27 to i8
  %29 = zext i8 %28 to i32
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 23
  store volatile i32 %29, ptr %31, align 8
  %32 = load i32, ptr %7, align 4
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %34, label %38

34:                                               ; preds = %15
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 8
  %37 = load ptr, ptr %36, align 8
  call void @settraps(ptr noundef %37)
  br label %38

38:                                               ; preds = %34, %15
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @settraps(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  br label %3

3:                                                ; preds = %18, %1
  %4 = load ptr, ptr %2, align 8
  %5 = icmp ne ptr %4, null
  br i1 %5, label %6, label %22

6:                                                ; preds = %3
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.CallInfo, ptr %7, i32 0, i32 7
  %9 = load i16, ptr %8, align 2
  %10 = zext i16 %9 to i32
  %11 = and i32 %10, 2
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %17, label %13

13:                                               ; preds = %6
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 4
  %16 = getelementptr inbounds %struct.anon, ptr %15, i32 0, i32 1
  store volatile i32 1, ptr %16, align 8
  br label %17

17:                                               ; preds = %13, %6
  br label %18

18:                                               ; preds = %17
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 2
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %2, align 8
  br label %3, !llvm.loop !9

22:                                               ; preds = %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_gethook(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 17
  %5 = load volatile ptr, ptr %4, align 8
  ret ptr %5
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_gethookmask(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 23
  %5 = load volatile i32, ptr %4, align 8
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_gethookcount(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 21
  %5 = load i32, ptr %4, align 8
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getstack(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %10 = load i32, ptr %6, align 4
  %11 = icmp slt i32 %10, 0
  br i1 %11, label %12, label %13

12:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %49

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 8
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %9, align 8
  br label %17

17:                                               ; preds = %30, %13
  %18 = load i32, ptr %6, align 4
  %19 = icmp sgt i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %17
  %21 = load ptr, ptr %9, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 16
  %24 = icmp ne ptr %21, %23
  br label %25

25:                                               ; preds = %20, %17
  %26 = phi i1 [ false, %17 ], [ %24, %20 ]
  br i1 %26, label %27, label %34

27:                                               ; preds = %25
  %28 = load i32, ptr %6, align 4
  %29 = add nsw i32 %28, -1
  store i32 %29, ptr %6, align 4
  br label %30

30:                                               ; preds = %27
  %31 = load ptr, ptr %9, align 8
  %32 = getelementptr inbounds %struct.CallInfo, ptr %31, i32 0, i32 2
  %33 = load ptr, ptr %32, align 8
  store ptr %33, ptr %9, align 8
  br label %17, !llvm.loop !10

34:                                               ; preds = %25
  %35 = load i32, ptr %6, align 4
  %36 = icmp eq i32 %35, 0
  br i1 %36, label %37, label %46

37:                                               ; preds = %34
  %38 = load ptr, ptr %9, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 16
  %41 = icmp ne ptr %38, %40
  br i1 %41, label %42, label %46

42:                                               ; preds = %37
  store i32 1, ptr %8, align 4
  %43 = load ptr, ptr %9, align 8
  %44 = load ptr, ptr %7, align 8
  %45 = getelementptr inbounds %struct.lua_Debug, ptr %44, i32 0, i32 16
  store ptr %43, ptr %45, align 8
  br label %47

46:                                               ; preds = %37, %34
  store i32 0, ptr %8, align 4
  br label %47

47:                                               ; preds = %46, %42
  %48 = load i32, ptr %8, align 4
  store i32 %48, ptr %4, align 4
  br label %49

49:                                               ; preds = %47, %12
  %50 = load i32, ptr %4, align 4
  ret i32 %50
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaG_findlocal(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = getelementptr inbounds %struct.CallInfo, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %union.StackValue, ptr %15, i64 1
  store ptr %16, ptr %10, align 8
  store ptr null, ptr %11, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 7
  %19 = load i16, ptr %18, align 2
  %20 = zext i16 %19 to i32
  %21 = and i32 %20, 2
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %44, label %23

23:                                               ; preds = %4
  %24 = load i32, ptr %8, align 4
  %25 = icmp slt i32 %24, 0
  br i1 %25, label %26, label %31

26:                                               ; preds = %23
  %27 = load ptr, ptr %7, align 8
  %28 = load i32, ptr %8, align 4
  %29 = load ptr, ptr %9, align 8
  %30 = call ptr @findvararg(ptr noundef %27, i32 noundef %28, ptr noundef %29)
  store ptr %30, ptr %5, align 8
  br label %101

31:                                               ; preds = %23
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.CallInfo, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.LClosure, ptr %36, i32 0, i32 5
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %8, align 4
  %40 = load ptr, ptr %7, align 8
  %41 = call i32 @currentpc(ptr noundef %40)
  %42 = call ptr @luaF_getlocalname(ptr noundef %38, i32 noundef %39, i32 noundef %41)
  store ptr %42, ptr %11, align 8
  br label %43

43:                                               ; preds = %31
  br label %44

44:                                               ; preds = %43, %4
  %45 = load ptr, ptr %11, align 8
  %46 = icmp eq ptr %45, null
  br i1 %46, label %47, label %89

47:                                               ; preds = %44
  %48 = load ptr, ptr %7, align 8
  %49 = load ptr, ptr %6, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 8
  %51 = load ptr, ptr %50, align 8
  %52 = icmp eq ptr %48, %51
  br i1 %52, label %53, label %57

53:                                               ; preds = %47
  %54 = load ptr, ptr %6, align 8
  %55 = getelementptr inbounds %struct.lua_State, ptr %54, i32 0, i32 6
  %56 = load ptr, ptr %55, align 8
  br label %63

57:                                               ; preds = %47
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.CallInfo, ptr %58, i32 0, i32 3
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds %struct.CallInfo, ptr %60, i32 0, i32 0
  %62 = load ptr, ptr %61, align 8
  br label %63

63:                                               ; preds = %57, %53
  %64 = phi ptr [ %56, %53 ], [ %62, %57 ]
  store ptr %64, ptr %12, align 8
  %65 = load ptr, ptr %12, align 8
  %66 = load ptr, ptr %10, align 8
  %67 = ptrtoint ptr %65 to i64
  %68 = ptrtoint ptr %66 to i64
  %69 = sub i64 %67, %68
  %70 = sdiv exact i64 %69, 16
  %71 = load i32, ptr %8, align 4
  %72 = sext i32 %71 to i64
  %73 = icmp sge i64 %70, %72
  br i1 %73, label %74, label %87

74:                                               ; preds = %63
  %75 = load i32, ptr %8, align 4
  %76 = icmp sgt i32 %75, 0
  br i1 %76, label %77, label %87

77:                                               ; preds = %74
  %78 = load ptr, ptr %7, align 8
  %79 = getelementptr inbounds %struct.CallInfo, ptr %78, i32 0, i32 7
  %80 = load i16, ptr %79, align 2
  %81 = zext i16 %80 to i32
  %82 = and i32 %81, 2
  %83 = icmp ne i32 %82, 0
  %84 = xor i1 %83, true
  %85 = zext i1 %84 to i64
  %86 = select i1 %84, ptr @.str, ptr @.str.1
  store ptr %86, ptr %11, align 8
  br label %88

87:                                               ; preds = %74, %63
  store ptr null, ptr %5, align 8
  br label %101

88:                                               ; preds = %77
  br label %89

89:                                               ; preds = %88, %44
  %90 = load ptr, ptr %9, align 8
  %91 = icmp ne ptr %90, null
  br i1 %91, label %92, label %99

92:                                               ; preds = %89
  %93 = load ptr, ptr %10, align 8
  %94 = load i32, ptr %8, align 4
  %95 = sub nsw i32 %94, 1
  %96 = sext i32 %95 to i64
  %97 = getelementptr inbounds %union.StackValue, ptr %93, i64 %96
  %98 = load ptr, ptr %9, align 8
  store ptr %97, ptr %98, align 8
  br label %99

99:                                               ; preds = %92, %89
  %100 = load ptr, ptr %11, align 8
  store ptr %100, ptr %5, align 8
  br label %101

101:                                              ; preds = %99, %87, %26
  %102 = load ptr, ptr %5, align 8
  ret ptr %102
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @findvararg(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.CallInfo, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.LClosure, ptr %13, i32 0, i32 5
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.Proto, ptr %15, i32 0, i32 4
  %17 = load i8, ptr %16, align 1
  %18 = icmp ne i8 %17, 0
  br i1 %18, label %19, label %43

19:                                               ; preds = %3
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.CallInfo, ptr %20, i32 0, i32 4
  %22 = getelementptr inbounds %struct.anon, ptr %21, i32 0, i32 2
  %23 = load i32, ptr %22, align 4
  store i32 %23, ptr %8, align 4
  %24 = load i32, ptr %6, align 4
  %25 = load i32, ptr %8, align 4
  %26 = sub nsw i32 0, %25
  %27 = icmp sge i32 %24, %26
  br i1 %27, label %28, label %42

28:                                               ; preds = %19
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.CallInfo, ptr %29, i32 0, i32 0
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %8, align 4
  %33 = sext i32 %32 to i64
  %34 = sub i64 0, %33
  %35 = getelementptr inbounds %union.StackValue, ptr %31, i64 %34
  %36 = load i32, ptr %6, align 4
  %37 = add nsw i32 %36, 1
  %38 = sext i32 %37 to i64
  %39 = sub i64 0, %38
  %40 = getelementptr inbounds %union.StackValue, ptr %35, i64 %39
  %41 = load ptr, ptr %7, align 8
  store ptr %40, ptr %41, align 8
  store ptr @.str.9, ptr %4, align 8
  br label %44

42:                                               ; preds = %19
  br label %43

43:                                               ; preds = %42, %3
  store ptr null, ptr %4, align 8
  br label %44

44:                                               ; preds = %43, %28
  %45 = load ptr, ptr %4, align 8
  ret ptr %45
}

declare hidden ptr @luaF_getlocalname(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @currentpc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.CallInfo, ptr %3, i32 0, i32 4
  %5 = getelementptr inbounds %struct.anon, ptr %4, i32 0, i32 0
  %6 = load ptr, ptr %5, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.CallInfo, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.LClosure, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.Proto, ptr %13, i32 0, i32 16
  %15 = load ptr, ptr %14, align 8
  %16 = ptrtoint ptr %6 to i64
  %17 = ptrtoint ptr %15 to i64
  %18 = sub i64 %16, %17
  %19 = sdiv exact i64 %18, 4
  %20 = trunc i64 %19 to i32
  %21 = sub nsw i32 %20, 1
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_getlocal(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %35

13:                                               ; preds = %3
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %union.StackValue, ptr %16, i64 -1
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 70
  br i1 %21, label %23, label %22

22:                                               ; preds = %13
  store ptr null, ptr %7, align 8
  br label %34

23:                                               ; preds = %13
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.lua_State, ptr %24, i32 0, i32 6
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %union.StackValue, ptr %26, i64 -1
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.LClosure, ptr %29, i32 0, i32 5
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %6, align 4
  %33 = call ptr @luaF_getlocalname(ptr noundef %31, i32 noundef %32, i32 noundef 0)
  store ptr %33, ptr %7, align 8
  br label %34

34:                                               ; preds = %23, %22
  br label %65

35:                                               ; preds = %3
  store ptr null, ptr %8, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.lua_Debug, ptr %37, i32 0, i32 16
  %39 = load ptr, ptr %38, align 8
  %40 = load i32, ptr %6, align 4
  %41 = call ptr @luaG_findlocal(ptr noundef %36, ptr noundef %39, i32 noundef %40, ptr noundef %8)
  store ptr %41, ptr %7, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = icmp ne ptr %42, null
  br i1 %43, label %44, label %64

44:                                               ; preds = %35
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 6
  %47 = load ptr, ptr %46, align 8
  store ptr %47, ptr %9, align 8
  %48 = load ptr, ptr %8, align 8
  store ptr %48, ptr %10, align 8
  %49 = load ptr, ptr %9, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 0
  %51 = load ptr, ptr %10, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %50, ptr align 8 %52, i64 8, i1 false)
  %53 = load ptr, ptr %10, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 1
  %55 = load i8, ptr %54, align 8
  %56 = load ptr, ptr %9, align 8
  %57 = getelementptr inbounds %struct.TValue, ptr %56, i32 0, i32 1
  store i8 %55, ptr %57, align 8
  %58 = load ptr, ptr %4, align 8
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.lua_State, ptr %59, i32 0, i32 6
  %61 = load ptr, ptr %60, align 8
  %62 = getelementptr inbounds %union.StackValue, ptr %61, i32 1
  store ptr %62, ptr %60, align 8
  %63 = load ptr, ptr %4, align 8
  br label %64

64:                                               ; preds = %44, %35
  br label %65

65:                                               ; preds = %64, %34
  %66 = load ptr, ptr %7, align 8
  ret ptr %66
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @lua_setlocal(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  store ptr null, ptr %7, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_Debug, ptr %12, i32 0, i32 16
  %14 = load ptr, ptr %13, align 8
  %15 = load i32, ptr %6, align 4
  %16 = call ptr @luaG_findlocal(ptr noundef %11, ptr noundef %14, i32 noundef %15, ptr noundef %7)
  store ptr %16, ptr %8, align 8
  %17 = load ptr, ptr %8, align 8
  %18 = icmp ne ptr %17, null
  br i1 %18, label %19, label %39

19:                                               ; preds = %3
  %20 = load ptr, ptr %7, align 8
  store ptr %20, ptr %9, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 6
  %23 = load ptr, ptr %22, align 8
  %24 = getelementptr inbounds %union.StackValue, ptr %23, i64 -1
  store ptr %24, ptr %10, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %10, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %26, ptr align 8 %28, i64 8, i1 false)
  %29 = load ptr, ptr %10, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = load ptr, ptr %9, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  store i8 %31, ptr %33, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %union.StackValue, ptr %37, i32 -1
  store ptr %38, ptr %36, align 8
  br label %39

39:                                               ; preds = %19, %3
  %40 = load ptr, ptr %8, align 8
  ret ptr %40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_getinfo(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp eq i32 %15, 62
  br i1 %16, label %17, label %29

17:                                               ; preds = %3
  store ptr null, ptr %9, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 6
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %union.StackValue, ptr %20, i64 -1
  store ptr %21, ptr %10, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds i8, ptr %23, i32 1
  store ptr %24, ptr %5, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %union.StackValue, ptr %27, i32 -1
  store ptr %28, ptr %26, align 8
  br label %36

29:                                               ; preds = %3
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.lua_Debug, ptr %30, i32 0, i32 16
  %32 = load ptr, ptr %31, align 8
  store ptr %32, ptr %9, align 8
  %33 = load ptr, ptr %9, align 8
  %34 = getelementptr inbounds %struct.CallInfo, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  store ptr %35, ptr %10, align 8
  br label %36

36:                                               ; preds = %29, %17
  %37 = load ptr, ptr %10, align 8
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 1
  %39 = load i8, ptr %38, align 8
  %40 = zext i8 %39 to i32
  %41 = icmp eq i32 %40, 70
  br i1 %41, label %48, label %42

42:                                               ; preds = %36
  %43 = load ptr, ptr %10, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = zext i8 %45 to i32
  %47 = icmp eq i32 %46, 102
  br i1 %47, label %48, label %52

48:                                               ; preds = %42, %36
  %49 = load ptr, ptr %10, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 0
  %51 = load ptr, ptr %50, align 8
  br label %53

52:                                               ; preds = %42
  br label %53

53:                                               ; preds = %52, %48
  %54 = phi ptr [ %51, %48 ], [ null, %52 ]
  store ptr %54, ptr %8, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = load ptr, ptr %6, align 8
  %58 = load ptr, ptr %8, align 8
  %59 = load ptr, ptr %9, align 8
  %60 = call i32 @auxgetinfo(ptr noundef %55, ptr noundef %56, ptr noundef %57, ptr noundef %58, ptr noundef %59)
  store i32 %60, ptr %7, align 4
  %61 = load ptr, ptr %5, align 8
  %62 = call ptr @strchr(ptr noundef %61, i32 noundef 102) #7
  %63 = icmp ne ptr %62, null
  br i1 %63, label %64, label %84

64:                                               ; preds = %53
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.lua_State, ptr %65, i32 0, i32 6
  %67 = load ptr, ptr %66, align 8
  store ptr %67, ptr %11, align 8
  %68 = load ptr, ptr %10, align 8
  store ptr %68, ptr %12, align 8
  %69 = load ptr, ptr %11, align 8
  %70 = getelementptr inbounds %struct.TValue, ptr %69, i32 0, i32 0
  %71 = load ptr, ptr %12, align 8
  %72 = getelementptr inbounds %struct.TValue, ptr %71, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %70, ptr align 8 %72, i64 8, i1 false)
  %73 = load ptr, ptr %12, align 8
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 1
  %75 = load i8, ptr %74, align 8
  %76 = load ptr, ptr %11, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 1
  store i8 %75, ptr %77, align 8
  %78 = load ptr, ptr %4, align 8
  %79 = load ptr, ptr %4, align 8
  %80 = getelementptr inbounds %struct.lua_State, ptr %79, i32 0, i32 6
  %81 = load ptr, ptr %80, align 8
  %82 = getelementptr inbounds %union.StackValue, ptr %81, i32 1
  store ptr %82, ptr %80, align 8
  %83 = load ptr, ptr %4, align 8
  br label %84

84:                                               ; preds = %64, %53
  %85 = load ptr, ptr %5, align 8
  %86 = call ptr @strchr(ptr noundef %85, i32 noundef 76) #7
  %87 = icmp ne ptr %86, null
  br i1 %87, label %88, label %91

88:                                               ; preds = %84
  %89 = load ptr, ptr %4, align 8
  %90 = load ptr, ptr %8, align 8
  call void @collectvalidlines(ptr noundef %89, ptr noundef %90)
  br label %91

91:                                               ; preds = %88, %84
  %92 = load i32, ptr %7, align 4
  ret i32 %92
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @auxgetinfo(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  store i32 1, ptr %11, align 4
  br label %12

12:                                               ; preds = %149, %5
  %13 = load ptr, ptr %7, align 8
  %14 = load i8, ptr %13, align 1
  %15 = icmp ne i8 %14, 0
  br i1 %15, label %16, label %152

16:                                               ; preds = %12
  %17 = load ptr, ptr %7, align 8
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  switch i32 %19, label %147 [
    i32 83, label %20
    i32 108, label %23
    i32 117, label %41
    i32 116, label %84
    i32 110, label %99
    i32 114, label %117
    i32 76, label %146
    i32 102, label %146
  ]

20:                                               ; preds = %16
  %21 = load ptr, ptr %8, align 8
  %22 = load ptr, ptr %9, align 8
  call void @funcinfo(ptr noundef %21, ptr noundef %22)
  br label %148

23:                                               ; preds = %16
  %24 = load ptr, ptr %10, align 8
  %25 = icmp ne ptr %24, null
  br i1 %25, label %26, label %36

26:                                               ; preds = %23
  %27 = load ptr, ptr %10, align 8
  %28 = getelementptr inbounds %struct.CallInfo, ptr %27, i32 0, i32 7
  %29 = load i16, ptr %28, align 2
  %30 = zext i16 %29 to i32
  %31 = and i32 %30, 2
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %36, label %33

33:                                               ; preds = %26
  %34 = load ptr, ptr %10, align 8
  %35 = call i32 @getcurrentline(ptr noundef %34)
  br label %37

36:                                               ; preds = %26, %23
  br label %37

37:                                               ; preds = %36, %33
  %38 = phi i32 [ %35, %33 ], [ -1, %36 ]
  %39 = load ptr, ptr %8, align 8
  %40 = getelementptr inbounds %struct.lua_Debug, ptr %39, i32 0, i32 6
  store i32 %38, ptr %40, align 8
  br label %148

41:                                               ; preds = %16
  %42 = load ptr, ptr %9, align 8
  %43 = icmp eq ptr %42, null
  br i1 %43, label %44, label %45

44:                                               ; preds = %41
  br label %50

45:                                               ; preds = %41
  %46 = load ptr, ptr %9, align 8
  %47 = getelementptr inbounds %struct.CClosure, ptr %46, i32 0, i32 3
  %48 = load i8, ptr %47, align 2
  %49 = zext i8 %48 to i32
  br label %50

50:                                               ; preds = %45, %44
  %51 = phi i32 [ 0, %44 ], [ %49, %45 ]
  %52 = trunc i32 %51 to i8
  %53 = load ptr, ptr %8, align 8
  %54 = getelementptr inbounds %struct.lua_Debug, ptr %53, i32 0, i32 9
  store i8 %52, ptr %54, align 4
  %55 = load ptr, ptr %9, align 8
  %56 = icmp ne ptr %55, null
  br i1 %56, label %57, label %63

57:                                               ; preds = %50
  %58 = load ptr, ptr %9, align 8
  %59 = getelementptr inbounds %struct.CClosure, ptr %58, i32 0, i32 1
  %60 = load i8, ptr %59, align 8
  %61 = zext i8 %60 to i32
  %62 = icmp eq i32 %61, 6
  br i1 %62, label %68, label %63

63:                                               ; preds = %57, %50
  %64 = load ptr, ptr %8, align 8
  %65 = getelementptr inbounds %struct.lua_Debug, ptr %64, i32 0, i32 11
  store i8 1, ptr %65, align 2
  %66 = load ptr, ptr %8, align 8
  %67 = getelementptr inbounds %struct.lua_Debug, ptr %66, i32 0, i32 10
  store i8 0, ptr %67, align 1
  br label %83

68:                                               ; preds = %57
  %69 = load ptr, ptr %9, align 8
  %70 = getelementptr inbounds %struct.LClosure, ptr %69, i32 0, i32 5
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %struct.Proto, ptr %71, i32 0, i32 4
  %73 = load i8, ptr %72, align 1
  %74 = load ptr, ptr %8, align 8
  %75 = getelementptr inbounds %struct.lua_Debug, ptr %74, i32 0, i32 11
  store i8 %73, ptr %75, align 2
  %76 = load ptr, ptr %9, align 8
  %77 = getelementptr inbounds %struct.LClosure, ptr %76, i32 0, i32 5
  %78 = load ptr, ptr %77, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 3
  %80 = load i8, ptr %79, align 2
  %81 = load ptr, ptr %8, align 8
  %82 = getelementptr inbounds %struct.lua_Debug, ptr %81, i32 0, i32 10
  store i8 %80, ptr %82, align 1
  br label %83

83:                                               ; preds = %68, %63
  br label %148

84:                                               ; preds = %16
  %85 = load ptr, ptr %10, align 8
  %86 = icmp ne ptr %85, null
  br i1 %86, label %87, label %93

87:                                               ; preds = %84
  %88 = load ptr, ptr %10, align 8
  %89 = getelementptr inbounds %struct.CallInfo, ptr %88, i32 0, i32 7
  %90 = load i16, ptr %89, align 2
  %91 = zext i16 %90 to i32
  %92 = and i32 %91, 32
  br label %94

93:                                               ; preds = %84
  br label %94

94:                                               ; preds = %93, %87
  %95 = phi i32 [ %92, %87 ], [ 0, %93 ]
  %96 = trunc i32 %95 to i8
  %97 = load ptr, ptr %8, align 8
  %98 = getelementptr inbounds %struct.lua_Debug, ptr %97, i32 0, i32 12
  store i8 %96, ptr %98, align 1
  br label %148

99:                                               ; preds = %16
  %100 = load ptr, ptr %6, align 8
  %101 = load ptr, ptr %10, align 8
  %102 = load ptr, ptr %8, align 8
  %103 = getelementptr inbounds %struct.lua_Debug, ptr %102, i32 0, i32 1
  %104 = call ptr @getfuncname(ptr noundef %100, ptr noundef %101, ptr noundef %103)
  %105 = load ptr, ptr %8, align 8
  %106 = getelementptr inbounds %struct.lua_Debug, ptr %105, i32 0, i32 2
  store ptr %104, ptr %106, align 8
  %107 = load ptr, ptr %8, align 8
  %108 = getelementptr inbounds %struct.lua_Debug, ptr %107, i32 0, i32 2
  %109 = load ptr, ptr %108, align 8
  %110 = icmp eq ptr %109, null
  br i1 %110, label %111, label %116

111:                                              ; preds = %99
  %112 = load ptr, ptr %8, align 8
  %113 = getelementptr inbounds %struct.lua_Debug, ptr %112, i32 0, i32 2
  store ptr @.str.10, ptr %113, align 8
  %114 = load ptr, ptr %8, align 8
  %115 = getelementptr inbounds %struct.lua_Debug, ptr %114, i32 0, i32 1
  store ptr null, ptr %115, align 8
  br label %116

116:                                              ; preds = %111, %99
  br label %148

117:                                              ; preds = %16
  %118 = load ptr, ptr %10, align 8
  %119 = icmp eq ptr %118, null
  br i1 %119, label %127, label %120

120:                                              ; preds = %117
  %121 = load ptr, ptr %10, align 8
  %122 = getelementptr inbounds %struct.CallInfo, ptr %121, i32 0, i32 7
  %123 = load i16, ptr %122, align 2
  %124 = zext i16 %123 to i32
  %125 = and i32 %124, 256
  %126 = icmp ne i32 %125, 0
  br i1 %126, label %132, label %127

127:                                              ; preds = %120, %117
  %128 = load ptr, ptr %8, align 8
  %129 = getelementptr inbounds %struct.lua_Debug, ptr %128, i32 0, i32 14
  store i16 0, ptr %129, align 2
  %130 = load ptr, ptr %8, align 8
  %131 = getelementptr inbounds %struct.lua_Debug, ptr %130, i32 0, i32 13
  store i16 0, ptr %131, align 8
  br label %145

132:                                              ; preds = %120
  %133 = load ptr, ptr %10, align 8
  %134 = getelementptr inbounds %struct.CallInfo, ptr %133, i32 0, i32 5
  %135 = getelementptr inbounds %struct.anon.2, ptr %134, i32 0, i32 0
  %136 = load i16, ptr %135, align 8
  %137 = load ptr, ptr %8, align 8
  %138 = getelementptr inbounds %struct.lua_Debug, ptr %137, i32 0, i32 13
  store i16 %136, ptr %138, align 8
  %139 = load ptr, ptr %10, align 8
  %140 = getelementptr inbounds %struct.CallInfo, ptr %139, i32 0, i32 5
  %141 = getelementptr inbounds %struct.anon.2, ptr %140, i32 0, i32 1
  %142 = load i16, ptr %141, align 2
  %143 = load ptr, ptr %8, align 8
  %144 = getelementptr inbounds %struct.lua_Debug, ptr %143, i32 0, i32 14
  store i16 %142, ptr %144, align 2
  br label %145

145:                                              ; preds = %132, %127
  br label %148

146:                                              ; preds = %16, %16
  br label %148

147:                                              ; preds = %16
  store i32 0, ptr %11, align 4
  br label %148

148:                                              ; preds = %147, %146, %145, %116, %94, %83, %37, %20
  br label %149

149:                                              ; preds = %148
  %150 = load ptr, ptr %7, align 8
  %151 = getelementptr inbounds i8, ptr %150, i32 1
  store ptr %151, ptr %7, align 8
  br label %12, !llvm.loop !11

152:                                              ; preds = %12
  %153 = load i32, ptr %11, align 4
  ret i32 %153
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @collectvalidlines(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca %struct.TValue, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = icmp ne ptr %12, null
  br i1 %13, label %14, label %20

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.CClosure, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = icmp eq i32 %18, 6
  br i1 %19, label %30, label %20

20:                                               ; preds = %14, %2
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 6
  %23 = load ptr, ptr %22, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 1
  store i8 0, ptr %24, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %union.StackValue, ptr %27, i32 1
  store ptr %28, ptr %26, align 8
  %29 = load ptr, ptr %3, align 8
  br label %90

30:                                               ; preds = %14
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.LClosure, ptr %31, i32 0, i32 5
  %33 = load ptr, ptr %32, align 8
  store ptr %33, ptr %5, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 13
  %36 = load i32, ptr %35, align 4
  store i32 %36, ptr %6, align 4
  %37 = load ptr, ptr %3, align 8
  %38 = call ptr @luaH_new(ptr noundef %37)
  store ptr %38, ptr %7, align 8
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  store ptr %41, ptr %8, align 8
  %42 = load ptr, ptr %7, align 8
  store ptr %42, ptr %9, align 8
  %43 = load ptr, ptr %9, align 8
  %44 = load ptr, ptr %8, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 0
  store ptr %43, ptr %45, align 8
  %46 = load ptr, ptr %8, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  store i8 69, ptr %47, align 8
  %48 = load ptr, ptr %3, align 8
  %49 = load ptr, ptr %3, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %union.StackValue, ptr %51, i32 1
  store ptr %52, ptr %50, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 19
  %56 = load ptr, ptr %55, align 8
  %57 = icmp ne ptr %56, null
  br i1 %57, label %58, label %89

58:                                               ; preds = %30
  %59 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 1
  store i8 17, ptr %59, align 8
  %60 = load ptr, ptr %5, align 8
  %61 = getelementptr inbounds %struct.Proto, ptr %60, i32 0, i32 4
  %62 = load i8, ptr %61, align 1
  %63 = icmp ne i8 %62, 0
  br i1 %63, label %65, label %64

64:                                               ; preds = %58
  store i32 0, ptr %10, align 4
  br label %69

65:                                               ; preds = %58
  %66 = load ptr, ptr %5, align 8
  %67 = load i32, ptr %6, align 4
  %68 = call i32 @nextline(ptr noundef %66, i32 noundef %67, i32 noundef 0)
  store i32 %68, ptr %6, align 4
  store i32 1, ptr %10, align 4
  br label %69

69:                                               ; preds = %65, %64
  br label %70

70:                                               ; preds = %85, %69
  %71 = load i32, ptr %10, align 4
  %72 = load ptr, ptr %5, align 8
  %73 = getelementptr inbounds %struct.Proto, ptr %72, i32 0, i32 9
  %74 = load i32, ptr %73, align 4
  %75 = icmp slt i32 %71, %74
  br i1 %75, label %76, label %88

76:                                               ; preds = %70
  %77 = load ptr, ptr %5, align 8
  %78 = load i32, ptr %6, align 4
  %79 = load i32, ptr %10, align 4
  %80 = call i32 @nextline(ptr noundef %77, i32 noundef %78, i32 noundef %79)
  store i32 %80, ptr %6, align 4
  %81 = load ptr, ptr %3, align 8
  %82 = load ptr, ptr %7, align 8
  %83 = load i32, ptr %6, align 4
  %84 = sext i32 %83 to i64
  call void @luaH_setint(ptr noundef %81, ptr noundef %82, i64 noundef %84, ptr noundef %11)
  br label %85

85:                                               ; preds = %76
  %86 = load i32, ptr %10, align 4
  %87 = add nsw i32 %86, 1
  store i32 %87, ptr %10, align 4
  br label %70, !llvm.loop !12

88:                                               ; preds = %70
  br label %89

89:                                               ; preds = %88, %30
  br label %90

90:                                               ; preds = %89, %20
  ret void
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_typeerror(ptr noundef %0, ptr noundef %1, ptr noundef %2) #4 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load ptr, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = call ptr @varinfo(ptr noundef %10, ptr noundef %11)
  call void @typeerror(ptr noundef %7, ptr noundef %8, ptr noundef %9, ptr noundef %12) #8
  unreachable
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @typeerror(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #4 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = call ptr @luaT_objtypename(ptr noundef %10, ptr noundef %11)
  store ptr %12, ptr %9, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = load ptr, ptr %8, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %13, ptr noundef @.str.16, ptr noundef %14, ptr noundef %15, ptr noundef %16) #8
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @varinfo(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  store ptr null, ptr %6, align 8
  store ptr null, ptr %7, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.CallInfo, ptr %12, i32 0, i32 7
  %14 = load i16, ptr %13, align 2
  %15 = zext i16 %14 to i32
  %16 = and i32 %15, 2
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %44, label %18

18:                                               ; preds = %2
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = call ptr @getupvalname(ptr noundef %19, ptr noundef %20, ptr noundef %6)
  store ptr %21, ptr %7, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = icmp ne ptr %22, null
  br i1 %23, label %43, label %24

24:                                               ; preds = %18
  %25 = load ptr, ptr %5, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = call i32 @instack(ptr noundef %25, ptr noundef %26)
  store i32 %27, ptr %8, align 4
  %28 = load i32, ptr %8, align 4
  %29 = icmp sge i32 %28, 0
  br i1 %29, label %30, label %42

30:                                               ; preds = %24
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.CallInfo, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.LClosure, ptr %35, i32 0, i32 5
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = call i32 @currentpc(ptr noundef %38)
  %40 = load i32, ptr %8, align 4
  %41 = call ptr @getobjname(ptr noundef %37, i32 noundef %39, i32 noundef %40, ptr noundef %6)
  store ptr %41, ptr %7, align 8
  br label %42

42:                                               ; preds = %30, %24
  br label %43

43:                                               ; preds = %42, %18
  br label %44

44:                                               ; preds = %43, %2
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %7, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = call ptr @formatvarinfo(ptr noundef %45, ptr noundef %46, ptr noundef %47)
  ret ptr %48
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_callerror(ptr noundef %0, ptr noundef %1) #4 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  store ptr null, ptr %6, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = call ptr @funcnamefromcall(ptr noundef %12, ptr noundef %13, ptr noundef %6)
  store ptr %14, ptr %7, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = icmp ne ptr %15, null
  br i1 %16, label %17, label %22

17:                                               ; preds = %2
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = call ptr @formatvarinfo(ptr noundef %18, ptr noundef %19, ptr noundef %20)
  br label %26

22:                                               ; preds = %2
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = call ptr @varinfo(ptr noundef %23, ptr noundef %24)
  br label %26

26:                                               ; preds = %22, %17
  %27 = phi ptr [ %21, %17 ], [ %25, %22 ]
  store ptr %27, ptr %8, align 8
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %8, align 8
  call void @typeerror(ptr noundef %28, ptr noundef %29, ptr noundef @.str.2, ptr noundef %30) #8
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @funcnamefromcall(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 7
  %10 = load i16, ptr %9, align 2
  %11 = zext i16 %10 to i32
  %12 = and i32 %11, 8
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %16

14:                                               ; preds = %3
  %15 = load ptr, ptr %7, align 8
  store ptr @.str.18, ptr %15, align 8
  store ptr @.str.26, ptr %4, align 8
  br label %46

16:                                               ; preds = %3
  %17 = load ptr, ptr %6, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 7
  %19 = load i16, ptr %18, align 2
  %20 = zext i16 %19 to i32
  %21 = and i32 %20, 128
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %25

23:                                               ; preds = %16
  %24 = load ptr, ptr %7, align 8
  store ptr @.str.27, ptr %24, align 8
  store ptr @.str.28, ptr %4, align 8
  br label %46

25:                                               ; preds = %16
  %26 = load ptr, ptr %6, align 8
  %27 = getelementptr inbounds %struct.CallInfo, ptr %26, i32 0, i32 7
  %28 = load i16, ptr %27, align 2
  %29 = zext i16 %28 to i32
  %30 = and i32 %29, 2
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %45, label %32

32:                                               ; preds = %25
  %33 = load ptr, ptr %5, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.CallInfo, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %struct.LClosure, ptr %38, i32 0, i32 5
  %40 = load ptr, ptr %39, align 8
  %41 = load ptr, ptr %6, align 8
  %42 = call i32 @currentpc(ptr noundef %41)
  %43 = load ptr, ptr %7, align 8
  %44 = call ptr @funcnamefromcode(ptr noundef %33, ptr noundef %40, i32 noundef %42, ptr noundef %43)
  store ptr %44, ptr %4, align 8
  br label %46

45:                                               ; preds = %25
  store ptr null, ptr %4, align 8
  br label %46

46:                                               ; preds = %45, %32, %23, %14
  %47 = load ptr, ptr %4, align 8
  ret ptr %47
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @formatvarinfo(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = icmp eq ptr %8, null
  br i1 %9, label %10, label %11

10:                                               ; preds = %3
  store ptr @.str.10, ptr %4, align 8
  br label %16

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %12, ptr noundef @.str.30, ptr noundef %13, ptr noundef %14)
  store ptr %15, ptr %4, align 8
  br label %16

16:                                               ; preds = %11, %10
  %17 = load ptr, ptr %4, align 8
  ret ptr %17
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_forerror(ptr noundef %0, ptr noundef %1, ptr noundef %2) #4 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call ptr @luaT_objtypename(ptr noundef %9, ptr noundef %10)
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %7, ptr noundef @.str.3, ptr noundef %8, ptr noundef %11) #8
  unreachable
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_runerror(ptr noundef %0, ptr noundef %1, ...) #4 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca [1 x %struct.__va_list_tag], align 16
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 8
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
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
  %22 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %7, i64 0, i64 0
  call void @llvm.va_start(ptr %22)
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %7, i64 0, i64 0
  %26 = call ptr @luaO_pushvfstring(ptr noundef %23, ptr noundef %24, ptr noundef %25)
  store ptr %26, ptr %6, align 8
  %27 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %7, i64 0, i64 0
  call void @llvm.va_end(ptr %27)
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.CallInfo, ptr %28, i32 0, i32 7
  %30 = load i16, ptr %29, align 2
  %31 = zext i16 %30 to i32
  %32 = and i32 %31, 2
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %71, label %34

34:                                               ; preds = %21
  %35 = load ptr, ptr %3, align 8
  %36 = load ptr, ptr %6, align 8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.CallInfo, ptr %37, i32 0, i32 0
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %struct.LClosure, ptr %41, i32 0, i32 5
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %struct.Proto, ptr %43, i32 0, i32 22
  %45 = load ptr, ptr %44, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = call i32 @getcurrentline(ptr noundef %46)
  %48 = call ptr @luaG_addinfo(ptr noundef %35, ptr noundef %36, ptr noundef %45, i32 noundef %47)
  %49 = load ptr, ptr %3, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %union.StackValue, ptr %51, i64 -2
  store ptr %52, ptr %8, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 6
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %union.StackValue, ptr %55, i64 -1
  store ptr %56, ptr %9, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = getelementptr inbounds %struct.TValue, ptr %57, i32 0, i32 0
  %59 = load ptr, ptr %9, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %58, ptr align 8 %60, i64 8, i1 false)
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 1
  %63 = load i8, ptr %62, align 8
  %64 = load ptr, ptr %8, align 8
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 1
  store i8 %63, ptr %65, align 8
  %66 = load ptr, ptr %3, align 8
  %67 = load ptr, ptr %3, align 8
  %68 = getelementptr inbounds %struct.lua_State, ptr %67, i32 0, i32 6
  %69 = load ptr, ptr %68, align 8
  %70 = getelementptr inbounds %union.StackValue, ptr %69, i32 -1
  store ptr %70, ptr %68, align 8
  br label %71

71:                                               ; preds = %34, %21
  %72 = load ptr, ptr %3, align 8
  call void @luaG_errormsg(ptr noundef %72) #8
  unreachable
}

declare hidden ptr @luaT_objtypename(ptr noundef, ptr noundef) #1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_concaterror(ptr noundef %0, ptr noundef %1, ptr noundef %2) #4 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = getelementptr inbounds %struct.TValue, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  %11 = and i32 %10, 15
  %12 = icmp eq i32 %11, 4
  br i1 %12, label %20, label %13

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 1
  %16 = load i8, ptr %15, align 8
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 15
  %19 = icmp eq i32 %18, 3
  br i1 %19, label %20, label %22

20:                                               ; preds = %13, %3
  %21 = load ptr, ptr %6, align 8
  store ptr %21, ptr %5, align 8
  br label %22

22:                                               ; preds = %20, %13
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %5, align 8
  call void @luaG_typeerror(ptr noundef %23, ptr noundef %24, ptr noundef @.str.4) #8
  unreachable
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_opinterror(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #4 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %9 = load ptr, ptr %6, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = and i32 %12, 15
  %14 = icmp eq i32 %13, 3
  br i1 %14, label %17, label %15

15:                                               ; preds = %4
  %16 = load ptr, ptr %6, align 8
  store ptr %16, ptr %7, align 8
  br label %17

17:                                               ; preds = %15, %4
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = load ptr, ptr %8, align 8
  call void @luaG_typeerror(ptr noundef %18, ptr noundef %19, ptr noundef %20) #8
  unreachable
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_tointerror(ptr noundef %0, ptr noundef %1, ptr noundef %2) #4 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = call i32 @luaV_tointegerns(ptr noundef %8, ptr noundef %7, i32 noundef 0)
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %13, label %11

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  store ptr %12, ptr %6, align 8
  br label %13

13:                                               ; preds = %11, %3
  %14 = load ptr, ptr %4, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = call ptr @varinfo(ptr noundef %15, ptr noundef %16)
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %14, ptr noundef @.str.5, ptr noundef %17) #8
  unreachable
}

declare hidden i32 @luaV_tointegerns(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_ordererror(ptr noundef %0, ptr noundef %1, ptr noundef %2) #4 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call ptr @luaT_objtypename(ptr noundef %9, ptr noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = call ptr @luaT_objtypename(ptr noundef %12, ptr noundef %13)
  store ptr %14, ptr %8, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = load ptr, ptr %8, align 8
  %17 = call i32 @strcmp(ptr noundef %15, ptr noundef %16) #7
  %18 = icmp eq i32 %17, 0
  br i1 %18, label %19, label %22

19:                                               ; preds = %3
  %20 = load ptr, ptr %4, align 8
  %21 = load ptr, ptr %7, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %20, ptr noundef @.str.6, ptr noundef %21) #8
  unreachable

22:                                               ; preds = %3
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = load ptr, ptr %8, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %23, ptr noundef @.str.7, ptr noundef %24, ptr noundef %25) #8
  unreachable
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaG_addinfo(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca [60 x i8], align 16
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %10 = load ptr, ptr %7, align 8
  %11 = icmp ne ptr %10, null
  br i1 %11, label %12, label %33

12:                                               ; preds = %4
  %13 = getelementptr inbounds [60 x i8], ptr %9, i64 0, i64 0
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.TString, ptr %14, i32 0, i32 7
  %16 = getelementptr inbounds [1 x i8], ptr %15, i64 0, i64 0
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.TString, ptr %17, i32 0, i32 4
  %19 = load i8, ptr %18, align 1
  %20 = zext i8 %19 to i32
  %21 = icmp ne i32 %20, 255
  br i1 %21, label %22, label %27

22:                                               ; preds = %12
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 4
  %25 = load i8, ptr %24, align 1
  %26 = zext i8 %25 to i64
  br label %31

27:                                               ; preds = %12
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.TString, ptr %28, i32 0, i32 6
  %30 = load i64, ptr %29, align 8
  br label %31

31:                                               ; preds = %27, %22
  %32 = phi i64 [ %26, %22 ], [ %30, %27 ]
  call void @luaO_chunkid(ptr noundef %13, ptr noundef %16, i64 noundef %32)
  br label %36

33:                                               ; preds = %4
  %34 = getelementptr inbounds [60 x i8], ptr %9, i64 0, i64 0
  store i8 63, ptr %34, align 16
  %35 = getelementptr inbounds [60 x i8], ptr %9, i64 0, i64 1
  store i8 0, ptr %35, align 1
  br label %36

36:                                               ; preds = %33, %31
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds [60 x i8], ptr %9, i64 0, i64 0
  %39 = load i32, ptr %8, align 4
  %40 = load ptr, ptr %6, align 8
  %41 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %37, ptr noundef @.str.8, ptr noundef %38, i32 noundef %39, ptr noundef %40)
  ret ptr %41
}

declare hidden void @luaO_chunkid(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden ptr @luaO_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaG_errormsg(ptr noundef %0) #4 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 18
  %10 = load i64, ptr %9, align 8
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %12, label %61

12:                                               ; preds = %1
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 10
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 18
  %18 = load i64, ptr %17, align 8
  %19 = getelementptr inbounds i8, ptr %15, i64 %18
  store ptr %19, ptr %3, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 6
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr %4, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 6
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i64 -1
  store ptr %26, ptr %5, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %28, ptr align 8 %30, i64 8, i1 false)
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 1
  %33 = load i8, ptr %32, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  store i8 %33, ptr %35, align 8
  %36 = load ptr, ptr %2, align 8
  %37 = load ptr, ptr %2, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 6
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %union.StackValue, ptr %39, i64 -1
  store ptr %40, ptr %6, align 8
  %41 = load ptr, ptr %3, align 8
  store ptr %41, ptr %7, align 8
  %42 = load ptr, ptr %6, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %7, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %43, ptr align 8 %45, i64 8, i1 false)
  %46 = load ptr, ptr %7, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  %48 = load i8, ptr %47, align 8
  %49 = load ptr, ptr %6, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 1
  store i8 %48, ptr %50, align 8
  %51 = load ptr, ptr %2, align 8
  %52 = load ptr, ptr %2, align 8
  %53 = getelementptr inbounds %struct.lua_State, ptr %52, i32 0, i32 6
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr inbounds %union.StackValue, ptr %54, i32 1
  store ptr %55, ptr %53, align 8
  %56 = load ptr, ptr %2, align 8
  %57 = load ptr, ptr %2, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 6
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %union.StackValue, ptr %59, i64 -2
  call void @luaD_callnoyield(ptr noundef %56, ptr noundef %60, i32 noundef 1)
  br label %61

61:                                               ; preds = %12, %1
  %62 = load ptr, ptr %2, align 8
  call void @luaD_throw(ptr noundef %62, i32 noundef 2) #8
  unreachable
}

declare hidden void @luaD_callnoyield(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #5

declare hidden void @luaC_step(ptr noundef) #1

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_start(ptr) #6

declare hidden ptr @luaO_pushvfstring(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_end(ptr) #6

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getcurrentline(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.CallInfo, ptr %3, i32 0, i32 0
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.TValue, ptr %5, i32 0, i32 0
  %7 = load ptr, ptr %6, align 8
  %8 = getelementptr inbounds %struct.LClosure, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = call i32 @currentpc(ptr noundef %10)
  %12 = call i32 @luaG_getfuncline(ptr noundef %9, i32 noundef %11)
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaG_tracecall(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 8
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.CallInfo, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.LClosure, ptr %13, i32 0, i32 5
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %5, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.CallInfo, ptr %16, i32 0, i32 4
  %18 = getelementptr inbounds %struct.anon, ptr %17, i32 0, i32 1
  store volatile i32 1, ptr %18, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 4
  %21 = getelementptr inbounds %struct.anon, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Proto, ptr %23, i32 0, i32 16
  %25 = load ptr, ptr %24, align 8
  %26 = icmp eq ptr %22, %25
  br i1 %26, label %27, label %45

27:                                               ; preds = %1
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 4
  %30 = load i8, ptr %29, align 1
  %31 = icmp ne i8 %30, 0
  br i1 %31, label %32, label %33

32:                                               ; preds = %27
  store i32 0, ptr %2, align 4
  br label %46

33:                                               ; preds = %27
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.CallInfo, ptr %34, i32 0, i32 7
  %36 = load i16, ptr %35, align 2
  %37 = zext i16 %36 to i32
  %38 = and i32 %37, 64
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %43, label %40

40:                                               ; preds = %33
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %4, align 8
  call void @luaD_hookcall(ptr noundef %41, ptr noundef %42)
  br label %43

43:                                               ; preds = %40, %33
  br label %44

44:                                               ; preds = %43
  br label %45

45:                                               ; preds = %44, %1
  store i32 1, ptr %2, align 4
  br label %46

46:                                               ; preds = %45, %32
  %47 = load i32, ptr %2, align 4
  ret i32 %47
}

declare hidden void @luaD_hookcall(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaG_traceexec(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i8, align 1
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 8
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %6, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 23
  %18 = load volatile i32, ptr %17, align 8
  %19 = trunc i32 %18 to i8
  store i8 %19, ptr %7, align 1
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.CallInfo, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.LClosure, ptr %24, i32 0, i32 5
  %26 = load ptr, ptr %25, align 8
  store ptr %26, ptr %8, align 8
  %27 = load i8, ptr %7, align 1
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 12
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %35, label %31

31:                                               ; preds = %2
  %32 = load ptr, ptr %6, align 8
  %33 = getelementptr inbounds %struct.CallInfo, ptr %32, i32 0, i32 4
  %34 = getelementptr inbounds %struct.anon, ptr %33, i32 0, i32 1
  store volatile i32 0, ptr %34, align 8
  store i32 0, ptr %3, align 4
  br label %190

35:                                               ; preds = %2
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds i32, ptr %36, i32 1
  store ptr %37, ptr %5, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = load ptr, ptr %6, align 8
  %40 = getelementptr inbounds %struct.CallInfo, ptr %39, i32 0, i32 4
  %41 = getelementptr inbounds %struct.anon, ptr %40, i32 0, i32 0
  store ptr %38, ptr %41, align 8
  %42 = load i8, ptr %7, align 1
  %43 = zext i8 %42 to i32
  %44 = and i32 %43, 8
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %46, label %52

46:                                               ; preds = %35
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 22
  %49 = load i32, ptr %48, align 4
  %50 = add nsw i32 %49, -1
  store i32 %50, ptr %48, align 4
  %51 = icmp eq i32 %50, 0
  br label %52

52:                                               ; preds = %46, %35
  %53 = phi i1 [ false, %35 ], [ %51, %46 ]
  %54 = zext i1 %53 to i32
  store i32 %54, ptr %9, align 4
  %55 = load i32, ptr %9, align 4
  %56 = icmp ne i32 %55, 0
  br i1 %56, label %57, label %63

57:                                               ; preds = %52
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.lua_State, ptr %58, i32 0, i32 21
  %60 = load i32, ptr %59, align 8
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 22
  store i32 %60, ptr %62, align 4
  br label %70

63:                                               ; preds = %52
  %64 = load i8, ptr %7, align 1
  %65 = zext i8 %64 to i32
  %66 = and i32 %65, 4
  %67 = icmp ne i32 %66, 0
  br i1 %67, label %69, label %68

68:                                               ; preds = %63
  store i32 1, ptr %3, align 4
  br label %190

69:                                               ; preds = %63
  br label %70

70:                                               ; preds = %69, %57
  %71 = load ptr, ptr %6, align 8
  %72 = getelementptr inbounds %struct.CallInfo, ptr %71, i32 0, i32 7
  %73 = load i16, ptr %72, align 2
  %74 = zext i16 %73 to i32
  %75 = and i32 %74, 64
  %76 = icmp ne i32 %75, 0
  br i1 %76, label %77, label %84

77:                                               ; preds = %70
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.CallInfo, ptr %78, i32 0, i32 7
  %80 = load i16, ptr %79, align 2
  %81 = zext i16 %80 to i32
  %82 = and i32 %81, -65
  %83 = trunc i32 %82 to i16
  store i16 %83, ptr %79, align 2
  store i32 1, ptr %3, align 4
  br label %190

84:                                               ; preds = %70
  %85 = load ptr, ptr %6, align 8
  %86 = getelementptr inbounds %struct.CallInfo, ptr %85, i32 0, i32 4
  %87 = getelementptr inbounds %struct.anon, ptr %86, i32 0, i32 0
  %88 = load ptr, ptr %87, align 8
  %89 = getelementptr inbounds i32, ptr %88, i64 -1
  %90 = load i32, ptr %89, align 4
  %91 = lshr i32 %90, 0
  %92 = and i32 %91, 127
  %93 = zext i32 %92 to i64
  %94 = getelementptr inbounds [83 x i8], ptr @luaP_opmodes, i64 0, i64 %93
  %95 = load i8, ptr %94, align 1
  %96 = zext i8 %95 to i32
  %97 = and i32 %96, 32
  %98 = icmp ne i32 %97, 0
  br i1 %98, label %99, label %109

99:                                               ; preds = %84
  %100 = load ptr, ptr %6, align 8
  %101 = getelementptr inbounds %struct.CallInfo, ptr %100, i32 0, i32 4
  %102 = getelementptr inbounds %struct.anon, ptr %101, i32 0, i32 0
  %103 = load ptr, ptr %102, align 8
  %104 = getelementptr inbounds i32, ptr %103, i64 -1
  %105 = load i32, ptr %104, align 4
  %106 = lshr i32 %105, 16
  %107 = and i32 %106, 255
  %108 = icmp eq i32 %107, 0
  br i1 %108, label %115, label %109

109:                                              ; preds = %99, %84
  %110 = load ptr, ptr %6, align 8
  %111 = getelementptr inbounds %struct.CallInfo, ptr %110, i32 0, i32 1
  %112 = load ptr, ptr %111, align 8
  %113 = load ptr, ptr %4, align 8
  %114 = getelementptr inbounds %struct.lua_State, ptr %113, i32 0, i32 6
  store ptr %112, ptr %114, align 8
  br label %115

115:                                              ; preds = %109, %99
  %116 = load i32, ptr %9, align 4
  %117 = icmp ne i32 %116, 0
  br i1 %117, label %118, label %120

118:                                              ; preds = %115
  %119 = load ptr, ptr %4, align 8
  call void @luaD_hook(ptr noundef %119, i32 noundef 3, i32 noundef -1, i32 noundef 0, i32 noundef 0)
  br label %120

120:                                              ; preds = %118, %115
  %121 = load i8, ptr %7, align 1
  %122 = zext i8 %121 to i32
  %123 = and i32 %122, 4
  %124 = icmp ne i32 %123, 0
  br i1 %124, label %125, label %169

125:                                              ; preds = %120
  %126 = load ptr, ptr %4, align 8
  %127 = getelementptr inbounds %struct.lua_State, ptr %126, i32 0, i32 20
  %128 = load i32, ptr %127, align 4
  %129 = load ptr, ptr %8, align 8
  %130 = getelementptr inbounds %struct.Proto, ptr %129, i32 0, i32 8
  %131 = load i32, ptr %130, align 8
  %132 = icmp slt i32 %128, %131
  br i1 %132, label %133, label %137

133:                                              ; preds = %125
  %134 = load ptr, ptr %4, align 8
  %135 = getelementptr inbounds %struct.lua_State, ptr %134, i32 0, i32 20
  %136 = load i32, ptr %135, align 4
  br label %138

137:                                              ; preds = %125
  br label %138

138:                                              ; preds = %137, %133
  %139 = phi i32 [ %136, %133 ], [ 0, %137 ]
  store i32 %139, ptr %10, align 4
  %140 = load ptr, ptr %5, align 8
  %141 = load ptr, ptr %8, align 8
  %142 = getelementptr inbounds %struct.Proto, ptr %141, i32 0, i32 16
  %143 = load ptr, ptr %142, align 8
  %144 = ptrtoint ptr %140 to i64
  %145 = ptrtoint ptr %143 to i64
  %146 = sub i64 %144, %145
  %147 = sdiv exact i64 %146, 4
  %148 = trunc i64 %147 to i32
  %149 = sub nsw i32 %148, 1
  store i32 %149, ptr %11, align 4
  %150 = load i32, ptr %11, align 4
  %151 = load i32, ptr %10, align 4
  %152 = icmp sle i32 %150, %151
  br i1 %152, label %159, label %153

153:                                              ; preds = %138
  %154 = load ptr, ptr %8, align 8
  %155 = load i32, ptr %10, align 4
  %156 = load i32, ptr %11, align 4
  %157 = call i32 @changedline(ptr noundef %154, i32 noundef %155, i32 noundef %156)
  %158 = icmp ne i32 %157, 0
  br i1 %158, label %159, label %165

159:                                              ; preds = %153, %138
  %160 = load ptr, ptr %8, align 8
  %161 = load i32, ptr %11, align 4
  %162 = call i32 @luaG_getfuncline(ptr noundef %160, i32 noundef %161)
  store i32 %162, ptr %12, align 4
  %163 = load ptr, ptr %4, align 8
  %164 = load i32, ptr %12, align 4
  call void @luaD_hook(ptr noundef %163, i32 noundef 2, i32 noundef %164, i32 noundef 0, i32 noundef 0)
  br label %165

165:                                              ; preds = %159, %153
  %166 = load i32, ptr %11, align 4
  %167 = load ptr, ptr %4, align 8
  %168 = getelementptr inbounds %struct.lua_State, ptr %167, i32 0, i32 20
  store i32 %166, ptr %168, align 4
  br label %169

169:                                              ; preds = %165, %120
  %170 = load ptr, ptr %4, align 8
  %171 = getelementptr inbounds %struct.lua_State, ptr %170, i32 0, i32 3
  %172 = load i8, ptr %171, align 2
  %173 = zext i8 %172 to i32
  %174 = icmp eq i32 %173, 1
  br i1 %174, label %175, label %189

175:                                              ; preds = %169
  %176 = load i32, ptr %9, align 4
  %177 = icmp ne i32 %176, 0
  br i1 %177, label %178, label %181

178:                                              ; preds = %175
  %179 = load ptr, ptr %4, align 8
  %180 = getelementptr inbounds %struct.lua_State, ptr %179, i32 0, i32 22
  store i32 1, ptr %180, align 4
  br label %181

181:                                              ; preds = %178, %175
  %182 = load ptr, ptr %6, align 8
  %183 = getelementptr inbounds %struct.CallInfo, ptr %182, i32 0, i32 7
  %184 = load i16, ptr %183, align 2
  %185 = zext i16 %184 to i32
  %186 = or i32 %185, 64
  %187 = trunc i32 %186 to i16
  store i16 %187, ptr %183, align 2
  %188 = load ptr, ptr %4, align 8
  call void @luaD_throw(ptr noundef %188, i32 noundef 1) #8
  unreachable

189:                                              ; preds = %169
  store i32 1, ptr %3, align 4
  br label %190

190:                                              ; preds = %189, %77, %68, %31
  %191 = load i32, ptr %3, align 4
  ret i32 %191
}

declare hidden void @luaD_hook(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @changedline(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.Proto, ptr %11, i32 0, i32 19
  %13 = load ptr, ptr %12, align 8
  %14 = icmp eq ptr %13, null
  br i1 %14, label %15, label %16

15:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %58

16:                                               ; preds = %3
  %17 = load i32, ptr %7, align 4
  %18 = load i32, ptr %6, align 4
  %19 = sub nsw i32 %17, %18
  %20 = icmp slt i32 %19, 64
  br i1 %20, label %21, label %49

21:                                               ; preds = %16
  store i32 0, ptr %8, align 4
  %22 = load i32, ptr %6, align 4
  store i32 %22, ptr %9, align 4
  br label %23

23:                                               ; preds = %47, %21
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 19
  %26 = load ptr, ptr %25, align 8
  %27 = load i32, ptr %9, align 4
  %28 = add nsw i32 %27, 1
  store i32 %28, ptr %9, align 4
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds i8, ptr %26, i64 %29
  %31 = load i8, ptr %30, align 1
  %32 = sext i8 %31 to i32
  store i32 %32, ptr %10, align 4
  %33 = load i32, ptr %10, align 4
  %34 = icmp eq i32 %33, -128
  br i1 %34, label %35, label %36

35:                                               ; preds = %23
  br label %48

36:                                               ; preds = %23
  %37 = load i32, ptr %10, align 4
  %38 = load i32, ptr %8, align 4
  %39 = add nsw i32 %38, %37
  store i32 %39, ptr %8, align 4
  %40 = load i32, ptr %9, align 4
  %41 = load i32, ptr %7, align 4
  %42 = icmp eq i32 %40, %41
  br i1 %42, label %43, label %47

43:                                               ; preds = %36
  %44 = load i32, ptr %8, align 4
  %45 = icmp ne i32 %44, 0
  %46 = zext i1 %45 to i32
  store i32 %46, ptr %4, align 4
  br label %58

47:                                               ; preds = %36
  br label %23

48:                                               ; preds = %35
  br label %49

49:                                               ; preds = %48, %16
  %50 = load ptr, ptr %5, align 8
  %51 = load i32, ptr %6, align 4
  %52 = call i32 @luaG_getfuncline(ptr noundef %50, i32 noundef %51)
  %53 = load ptr, ptr %5, align 8
  %54 = load i32, ptr %7, align 4
  %55 = call i32 @luaG_getfuncline(ptr noundef %53, i32 noundef %54)
  %56 = icmp ne i32 %52, %55
  %57 = zext i1 %56 to i32
  store i32 %57, ptr %4, align 4
  br label %58

58:                                               ; preds = %49, %43, %15
  %59 = load i32, ptr %4, align 4
  ret i32 %59
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @funcinfo(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = icmp ne ptr %6, null
  br i1 %7, label %8, label %14

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.CClosure, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 6
  br i1 %13, label %25, label %14

14:                                               ; preds = %8, %2
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.lua_Debug, ptr %15, i32 0, i32 4
  store ptr @.str.11, ptr %16, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_Debug, ptr %17, i32 0, i32 5
  store i64 4, ptr %18, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_Debug, ptr %19, i32 0, i32 7
  store i32 -1, ptr %20, align 4
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.lua_Debug, ptr %21, i32 0, i32 8
  store i32 -1, ptr %22, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_Debug, ptr %23, i32 0, i32 3
  store ptr @.str.12, ptr %24, align 8
  br label %89

25:                                               ; preds = %8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.LClosure, ptr %26, i32 0, i32 5
  %28 = load ptr, ptr %27, align 8
  store ptr %28, ptr %5, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.Proto, ptr %29, i32 0, i32 22
  %31 = load ptr, ptr %30, align 8
  %32 = icmp ne ptr %31, null
  br i1 %32, label %33, label %65

33:                                               ; preds = %25
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 22
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.TString, ptr %36, i32 0, i32 7
  %38 = getelementptr inbounds [1 x i8], ptr %37, i64 0, i64 0
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.lua_Debug, ptr %39, i32 0, i32 4
  store ptr %38, ptr %40, align 8
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.Proto, ptr %41, i32 0, i32 22
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %struct.TString, ptr %43, i32 0, i32 4
  %45 = load i8, ptr %44, align 1
  %46 = zext i8 %45 to i32
  %47 = icmp ne i32 %46, 255
  br i1 %47, label %48, label %55

48:                                               ; preds = %33
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.Proto, ptr %49, i32 0, i32 22
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.TString, ptr %51, i32 0, i32 4
  %53 = load i8, ptr %52, align 1
  %54 = zext i8 %53 to i64
  br label %61

55:                                               ; preds = %33
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.Proto, ptr %56, i32 0, i32 22
  %58 = load ptr, ptr %57, align 8
  %59 = getelementptr inbounds %struct.TString, ptr %58, i32 0, i32 6
  %60 = load i64, ptr %59, align 8
  br label %61

61:                                               ; preds = %55, %48
  %62 = phi i64 [ %54, %48 ], [ %60, %55 ]
  %63 = load ptr, ptr %3, align 8
  %64 = getelementptr inbounds %struct.lua_Debug, ptr %63, i32 0, i32 5
  store i64 %62, ptr %64, align 8
  br label %70

65:                                               ; preds = %25
  %66 = load ptr, ptr %3, align 8
  %67 = getelementptr inbounds %struct.lua_Debug, ptr %66, i32 0, i32 4
  store ptr @.str.13, ptr %67, align 8
  %68 = load ptr, ptr %3, align 8
  %69 = getelementptr inbounds %struct.lua_Debug, ptr %68, i32 0, i32 5
  store i64 2, ptr %69, align 8
  br label %70

70:                                               ; preds = %65, %61
  %71 = load ptr, ptr %5, align 8
  %72 = getelementptr inbounds %struct.Proto, ptr %71, i32 0, i32 13
  %73 = load i32, ptr %72, align 4
  %74 = load ptr, ptr %3, align 8
  %75 = getelementptr inbounds %struct.lua_Debug, ptr %74, i32 0, i32 7
  store i32 %73, ptr %75, align 4
  %76 = load ptr, ptr %5, align 8
  %77 = getelementptr inbounds %struct.Proto, ptr %76, i32 0, i32 14
  %78 = load i32, ptr %77, align 8
  %79 = load ptr, ptr %3, align 8
  %80 = getelementptr inbounds %struct.lua_Debug, ptr %79, i32 0, i32 8
  store i32 %78, ptr %80, align 8
  %81 = load ptr, ptr %3, align 8
  %82 = getelementptr inbounds %struct.lua_Debug, ptr %81, i32 0, i32 7
  %83 = load i32, ptr %82, align 4
  %84 = icmp eq i32 %83, 0
  %85 = zext i1 %84 to i64
  %86 = select i1 %84, ptr @.str.14, ptr @.str.15
  %87 = load ptr, ptr %3, align 8
  %88 = getelementptr inbounds %struct.lua_Debug, ptr %87, i32 0, i32 3
  store ptr %86, ptr %88, align 8
  br label %89

89:                                               ; preds = %70, %14
  %90 = load ptr, ptr %3, align 8
  %91 = getelementptr inbounds %struct.lua_Debug, ptr %90, i32 0, i32 15
  %92 = getelementptr inbounds [60 x i8], ptr %91, i64 0, i64 0
  %93 = load ptr, ptr %3, align 8
  %94 = getelementptr inbounds %struct.lua_Debug, ptr %93, i32 0, i32 4
  %95 = load ptr, ptr %94, align 8
  %96 = load ptr, ptr %3, align 8
  %97 = getelementptr inbounds %struct.lua_Debug, ptr %96, i32 0, i32 5
  %98 = load i64, ptr %97, align 8
  call void @luaO_chunkid(ptr noundef %92, ptr noundef %95, i64 noundef %98)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getfuncname(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = icmp ne ptr %8, null
  br i1 %9, label %10, label %24

10:                                               ; preds = %3
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds %struct.CallInfo, ptr %11, i32 0, i32 7
  %13 = load i16, ptr %12, align 2
  %14 = zext i16 %13 to i32
  %15 = and i32 %14, 32
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %24, label %17

17:                                               ; preds = %10
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 2
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = call ptr @funcnamefromcall(ptr noundef %18, ptr noundef %21, ptr noundef %22)
  store ptr %23, ptr %4, align 8
  br label %25

24:                                               ; preds = %10, %3
  store ptr null, ptr %4, align 8
  br label %25

25:                                               ; preds = %24, %17
  %26 = load ptr, ptr %4, align 8
  ret ptr %26
}

declare hidden ptr @luaH_new(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @nextline(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.Proto, ptr %8, i32 0, i32 19
  %10 = load ptr, ptr %9, align 8
  %11 = load i32, ptr %7, align 4
  %12 = sext i32 %11 to i64
  %13 = getelementptr inbounds i8, ptr %10, i64 %12
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp ne i32 %15, -128
  br i1 %16, label %17, label %28

17:                                               ; preds = %3
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.Proto, ptr %19, i32 0, i32 19
  %21 = load ptr, ptr %20, align 8
  %22 = load i32, ptr %7, align 4
  %23 = sext i32 %22 to i64
  %24 = getelementptr inbounds i8, ptr %21, i64 %23
  %25 = load i8, ptr %24, align 1
  %26 = sext i8 %25 to i32
  %27 = add nsw i32 %18, %26
  store i32 %27, ptr %4, align 4
  br label %32

28:                                               ; preds = %3
  %29 = load ptr, ptr %5, align 8
  %30 = load i32, ptr %7, align 4
  %31 = call i32 @luaG_getfuncline(ptr noundef %29, i32 noundef %30)
  store i32 %31, ptr %4, align 4
  br label %32

32:                                               ; preds = %28, %17
  %33 = load i32, ptr %4, align 4
  ret i32 %33
}

declare hidden void @luaH_setint(ptr noundef, ptr noundef, i64 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getupvalname(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.CallInfo, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %8, align 8
  store i32 0, ptr %9, align 4
  br label %15

15:                                               ; preds = %41, %3
  %16 = load i32, ptr %9, align 4
  %17 = load ptr, ptr %8, align 8
  %18 = getelementptr inbounds %struct.LClosure, ptr %17, i32 0, i32 3
  %19 = load i8, ptr %18, align 2
  %20 = zext i8 %19 to i32
  %21 = icmp slt i32 %16, %20
  br i1 %21, label %22, label %44

22:                                               ; preds = %15
  %23 = load ptr, ptr %8, align 8
  %24 = getelementptr inbounds %struct.LClosure, ptr %23, i32 0, i32 6
  %25 = load i32, ptr %9, align 4
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds [1 x ptr], ptr %24, i64 0, i64 %26
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.UpVal, ptr %28, i32 0, i32 3
  %30 = load ptr, ptr %29, align 8
  %31 = load ptr, ptr %6, align 8
  %32 = icmp eq ptr %30, %31
  br i1 %32, label %33, label %40

33:                                               ; preds = %22
  %34 = load ptr, ptr %8, align 8
  %35 = getelementptr inbounds %struct.LClosure, ptr %34, i32 0, i32 5
  %36 = load ptr, ptr %35, align 8
  %37 = load i32, ptr %9, align 4
  %38 = call ptr @upvalname(ptr noundef %36, i32 noundef %37)
  %39 = load ptr, ptr %7, align 8
  store ptr %38, ptr %39, align 8
  store ptr @.str.17, ptr %4, align 8
  br label %45

40:                                               ; preds = %22
  br label %41

41:                                               ; preds = %40
  %42 = load i32, ptr %9, align 4
  %43 = add nsw i32 %42, 1
  store i32 %43, ptr %9, align 4
  br label %15, !llvm.loop !13

44:                                               ; preds = %15
  store ptr null, ptr %4, align 8
  br label %45

45:                                               ; preds = %44, %33
  %46 = load ptr, ptr %4, align 8
  ret ptr %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @instack(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %union.StackValue, ptr %10, i64 1
  store ptr %11, ptr %7, align 8
  store i32 0, ptr %6, align 4
  br label %12

12:                                               ; preds = %31, %2
  %13 = load ptr, ptr %7, align 8
  %14 = load i32, ptr %6, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds %union.StackValue, ptr %13, i64 %15
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 1
  %19 = load ptr, ptr %18, align 8
  %20 = icmp ult ptr %16, %19
  br i1 %20, label %21, label %34

21:                                               ; preds = %12
  %22 = load ptr, ptr %5, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = load i32, ptr %6, align 4
  %25 = sext i32 %24 to i64
  %26 = getelementptr inbounds %union.StackValue, ptr %23, i64 %25
  %27 = icmp eq ptr %22, %26
  br i1 %27, label %28, label %30

28:                                               ; preds = %21
  %29 = load i32, ptr %6, align 4
  store i32 %29, ptr %3, align 4
  br label %35

30:                                               ; preds = %21
  br label %31

31:                                               ; preds = %30
  %32 = load i32, ptr %6, align 4
  %33 = add nsw i32 %32, 1
  store i32 %33, ptr %6, align 4
  br label %12, !llvm.loop !14

34:                                               ; preds = %12
  store i32 -1, ptr %3, align 4
  br label %35

35:                                               ; preds = %34, %28
  %36 = load i32, ptr %3, align 4
  ret i32 %36
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getobjname(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = load i32, ptr %8, align 4
  %18 = load ptr, ptr %9, align 8
  %19 = call ptr @basicgetobjname(ptr noundef %16, ptr noundef %7, i32 noundef %17, ptr noundef %18)
  store ptr %19, ptr %10, align 8
  %20 = load ptr, ptr %10, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %24

22:                                               ; preds = %4
  %23 = load ptr, ptr %10, align 8
  store ptr %23, ptr %5, align 8
  br label %86

24:                                               ; preds = %4
  %25 = load i32, ptr %7, align 4
  %26 = icmp ne i32 %25, -1
  br i1 %26, label %27, label %84

27:                                               ; preds = %24
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 16
  %30 = load ptr, ptr %29, align 8
  %31 = load i32, ptr %7, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds i32, ptr %30, i64 %32
  %34 = load i32, ptr %33, align 4
  store i32 %34, ptr %11, align 4
  %35 = load i32, ptr %11, align 4
  %36 = lshr i32 %35, 0
  %37 = and i32 %36, 127
  store i32 %37, ptr %12, align 4
  %38 = load i32, ptr %12, align 4
  switch i32 %38, label %82 [
    i32 11, label %39
    i32 12, label %51
    i32 13, label %63
    i32 14, label %65
    i32 20, label %77
  ]

39:                                               ; preds = %27
  %40 = load i32, ptr %11, align 4
  %41 = lshr i32 %40, 24
  %42 = and i32 %41, 255
  store i32 %42, ptr %13, align 4
  %43 = load ptr, ptr %6, align 8
  %44 = load i32, ptr %13, align 4
  %45 = load ptr, ptr %9, align 8
  %46 = call ptr @kname(ptr noundef %43, i32 noundef %44, ptr noundef %45)
  %47 = load ptr, ptr %6, align 8
  %48 = load i32, ptr %7, align 4
  %49 = load i32, ptr %11, align 4
  %50 = call ptr @isEnv(ptr noundef %47, i32 noundef %48, i32 noundef %49, i32 noundef 1)
  store ptr %50, ptr %5, align 8
  br label %86

51:                                               ; preds = %27
  %52 = load i32, ptr %11, align 4
  %53 = lshr i32 %52, 24
  %54 = and i32 %53, 255
  store i32 %54, ptr %14, align 4
  %55 = load ptr, ptr %6, align 8
  %56 = load i32, ptr %7, align 4
  %57 = load i32, ptr %14, align 4
  %58 = load ptr, ptr %9, align 8
  call void @rname(ptr noundef %55, i32 noundef %56, i32 noundef %57, ptr noundef %58)
  %59 = load ptr, ptr %6, align 8
  %60 = load i32, ptr %7, align 4
  %61 = load i32, ptr %11, align 4
  %62 = call ptr @isEnv(ptr noundef %59, i32 noundef %60, i32 noundef %61, i32 noundef 0)
  store ptr %62, ptr %5, align 8
  br label %86

63:                                               ; preds = %27
  %64 = load ptr, ptr %9, align 8
  store ptr @.str.19, ptr %64, align 8
  store ptr @.str.20, ptr %5, align 8
  br label %86

65:                                               ; preds = %27
  %66 = load i32, ptr %11, align 4
  %67 = lshr i32 %66, 24
  %68 = and i32 %67, 255
  store i32 %68, ptr %15, align 4
  %69 = load ptr, ptr %6, align 8
  %70 = load i32, ptr %15, align 4
  %71 = load ptr, ptr %9, align 8
  %72 = call ptr @kname(ptr noundef %69, i32 noundef %70, ptr noundef %71)
  %73 = load ptr, ptr %6, align 8
  %74 = load i32, ptr %7, align 4
  %75 = load i32, ptr %11, align 4
  %76 = call ptr @isEnv(ptr noundef %73, i32 noundef %74, i32 noundef %75, i32 noundef 0)
  store ptr %76, ptr %5, align 8
  br label %86

77:                                               ; preds = %27
  %78 = load ptr, ptr %6, align 8
  %79 = load i32, ptr %7, align 4
  %80 = load i32, ptr %11, align 4
  %81 = load ptr, ptr %9, align 8
  call void @rkname(ptr noundef %78, i32 noundef %79, i32 noundef %80, ptr noundef %81)
  store ptr @.str.21, ptr %5, align 8
  br label %86

82:                                               ; preds = %27
  br label %83

83:                                               ; preds = %82
  br label %84

84:                                               ; preds = %83, %24
  br label %85

85:                                               ; preds = %84
  store ptr null, ptr %5, align 8
  br label %86

86:                                               ; preds = %85, %77, %65, %63, %51, %39, %22
  %87 = load ptr, ptr %5, align 8
  ret ptr %87
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @upvalname(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 18
  %9 = load ptr, ptr %8, align 8
  %10 = load i32, ptr %5, align 4
  %11 = sext i32 %10 to i64
  %12 = getelementptr inbounds %struct.Upvaldesc, ptr %9, i64 %11
  %13 = getelementptr inbounds %struct.Upvaldesc, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = icmp eq ptr %15, null
  br i1 %16, label %17, label %18

17:                                               ; preds = %2
  store ptr @.str.18, ptr %3, align 8
  br label %22

18:                                               ; preds = %2
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.TString, ptr %19, i32 0, i32 7
  %21 = getelementptr inbounds [1 x i8], ptr %20, i64 0, i64 0
  store ptr %21, ptr %3, align 8
  br label %22

22:                                               ; preds = %18, %17
  %23 = load ptr, ptr %3, align 8
  ret ptr %23
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @basicgetobjname(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load i32, ptr %14, align 4
  store i32 %15, ptr %10, align 4
  %16 = load ptr, ptr %6, align 8
  %17 = load i32, ptr %8, align 4
  %18 = add nsw i32 %17, 1
  %19 = load i32, ptr %10, align 4
  %20 = call ptr @luaF_getlocalname(ptr noundef %16, i32 noundef %18, i32 noundef %19)
  %21 = load ptr, ptr %9, align 8
  store ptr %20, ptr %21, align 8
  %22 = load ptr, ptr %9, align 8
  %23 = load ptr, ptr %22, align 8
  %24 = icmp ne ptr %23, null
  br i1 %24, label %25, label %26

25:                                               ; preds = %4
  store ptr @.str.22, ptr %5, align 8
  br label %93

26:                                               ; preds = %4
  %27 = load ptr, ptr %6, align 8
  %28 = load i32, ptr %10, align 4
  %29 = load i32, ptr %8, align 4
  %30 = call i32 @findsetreg(ptr noundef %27, i32 noundef %28, i32 noundef %29)
  store i32 %30, ptr %10, align 4
  %31 = load ptr, ptr %7, align 8
  store i32 %30, ptr %31, align 4
  %32 = load i32, ptr %10, align 4
  %33 = icmp ne i32 %32, -1
  br i1 %33, label %34, label %92

34:                                               ; preds = %26
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.Proto, ptr %35, i32 0, i32 16
  %37 = load ptr, ptr %36, align 8
  %38 = load i32, ptr %10, align 4
  %39 = sext i32 %38 to i64
  %40 = getelementptr inbounds i32, ptr %37, i64 %39
  %41 = load i32, ptr %40, align 4
  store i32 %41, ptr %11, align 4
  %42 = load i32, ptr %11, align 4
  %43 = lshr i32 %42, 0
  %44 = and i32 %43, 127
  store i32 %44, ptr %12, align 4
  %45 = load i32, ptr %12, align 4
  switch i32 %45, label %90 [
    i32 0, label %46
    i32 9, label %62
    i32 3, label %69
    i32 4, label %76
  ]

46:                                               ; preds = %34
  %47 = load i32, ptr %11, align 4
  %48 = lshr i32 %47, 16
  %49 = and i32 %48, 255
  store i32 %49, ptr %13, align 4
  %50 = load i32, ptr %13, align 4
  %51 = load i32, ptr %11, align 4
  %52 = lshr i32 %51, 7
  %53 = and i32 %52, 255
  %54 = icmp slt i32 %50, %53
  br i1 %54, label %55, label %61

55:                                               ; preds = %46
  %56 = load ptr, ptr %6, align 8
  %57 = load ptr, ptr %7, align 8
  %58 = load i32, ptr %13, align 4
  %59 = load ptr, ptr %9, align 8
  %60 = call ptr @basicgetobjname(ptr noundef %56, ptr noundef %57, i32 noundef %58, ptr noundef %59)
  store ptr %60, ptr %5, align 8
  br label %93

61:                                               ; preds = %46
  br label %91

62:                                               ; preds = %34
  %63 = load ptr, ptr %6, align 8
  %64 = load i32, ptr %11, align 4
  %65 = lshr i32 %64, 16
  %66 = and i32 %65, 255
  %67 = call ptr @upvalname(ptr noundef %63, i32 noundef %66)
  %68 = load ptr, ptr %9, align 8
  store ptr %67, ptr %68, align 8
  store ptr @.str.17, ptr %5, align 8
  br label %93

69:                                               ; preds = %34
  %70 = load ptr, ptr %6, align 8
  %71 = load i32, ptr %11, align 4
  %72 = lshr i32 %71, 15
  %73 = and i32 %72, 131071
  %74 = load ptr, ptr %9, align 8
  %75 = call ptr @kname(ptr noundef %70, i32 noundef %73, ptr noundef %74)
  store ptr %75, ptr %5, align 8
  br label %93

76:                                               ; preds = %34
  %77 = load ptr, ptr %6, align 8
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 16
  %80 = load ptr, ptr %79, align 8
  %81 = load i32, ptr %10, align 4
  %82 = add nsw i32 %81, 1
  %83 = sext i32 %82 to i64
  %84 = getelementptr inbounds i32, ptr %80, i64 %83
  %85 = load i32, ptr %84, align 4
  %86 = lshr i32 %85, 7
  %87 = and i32 %86, 33554431
  %88 = load ptr, ptr %9, align 8
  %89 = call ptr @kname(ptr noundef %77, i32 noundef %87, ptr noundef %88)
  store ptr %89, ptr %5, align 8
  br label %93

90:                                               ; preds = %34
  br label %91

91:                                               ; preds = %90, %61
  br label %92

92:                                               ; preds = %91, %26
  store ptr null, ptr %5, align 8
  br label %93

93:                                               ; preds = %92, %76, %69, %62, %55, %25
  %94 = load ptr, ptr %5, align 8
  ret ptr %94
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @kname(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 15
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %6, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds %struct.TValue, ptr %11, i64 %13
  store ptr %14, ptr %8, align 8
  %15 = load ptr, ptr %8, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 4
  br i1 %20, label %21, label %28

21:                                               ; preds = %3
  %22 = load ptr, ptr %8, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.TString, ptr %24, i32 0, i32 7
  %26 = getelementptr inbounds [1 x i8], ptr %25, i64 0, i64 0
  %27 = load ptr, ptr %7, align 8
  store ptr %26, ptr %27, align 8
  store ptr @.str.23, ptr %4, align 8
  br label %30

28:                                               ; preds = %3
  %29 = load ptr, ptr %7, align 8
  store ptr @.str.18, ptr %29, align 8
  store ptr null, ptr %4, align 8
  br label %30

30:                                               ; preds = %28, %21
  %31 = load ptr, ptr %4, align 8
  ret ptr %31
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @isEnv(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load i32, ptr %7, align 4
  %12 = lshr i32 %11, 16
  %13 = and i32 %12, 255
  store i32 %13, ptr %9, align 4
  %14 = load i32, ptr %8, align 4
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %20

16:                                               ; preds = %4
  %17 = load ptr, ptr %5, align 8
  %18 = load i32, ptr %9, align 4
  %19 = call ptr @upvalname(ptr noundef %17, i32 noundef %18)
  store ptr %19, ptr %10, align 8
  br label %24

20:                                               ; preds = %4
  %21 = load ptr, ptr %5, align 8
  %22 = load i32, ptr %9, align 4
  %23 = call ptr @basicgetobjname(ptr noundef %21, ptr noundef %6, i32 noundef %22, ptr noundef %10)
  br label %24

24:                                               ; preds = %20, %16
  %25 = load ptr, ptr %10, align 8
  %26 = icmp ne ptr %25, null
  br i1 %26, label %27, label %31

27:                                               ; preds = %24
  %28 = load ptr, ptr %10, align 8
  %29 = call i32 @strcmp(ptr noundef %28, ptr noundef @.str.24) #7
  %30 = icmp eq i32 %29, 0
  br label %31

31:                                               ; preds = %27, %24
  %32 = phi i1 [ false, %24 ], [ %30, %27 ]
  %33 = zext i1 %32 to i64
  %34 = select i1 %32, ptr @.str.25, ptr @.str.20
  ret ptr %34
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @rname(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load i32, ptr %7, align 4
  %12 = load ptr, ptr %8, align 8
  %13 = call ptr @basicgetobjname(ptr noundef %10, ptr noundef %6, i32 noundef %11, ptr noundef %12)
  store ptr %13, ptr %9, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %21

16:                                               ; preds = %4
  %17 = load ptr, ptr %9, align 8
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp eq i32 %19, 99
  br i1 %20, label %23, label %21

21:                                               ; preds = %16, %4
  %22 = load ptr, ptr %8, align 8
  store ptr @.str.18, ptr %22, align 8
  br label %23

23:                                               ; preds = %21, %16
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @rkname(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %10 = load i32, ptr %7, align 4
  %11 = lshr i32 %10, 24
  %12 = and i32 %11, 255
  store i32 %12, ptr %9, align 4
  %13 = load i32, ptr %7, align 4
  %14 = lshr i32 %13, 15
  %15 = and i32 %14, 1
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %22

17:                                               ; preds = %4
  %18 = load ptr, ptr %5, align 8
  %19 = load i32, ptr %9, align 4
  %20 = load ptr, ptr %8, align 8
  %21 = call ptr @kname(ptr noundef %18, i32 noundef %19, ptr noundef %20)
  br label %27

22:                                               ; preds = %4
  %23 = load ptr, ptr %5, align 8
  %24 = load i32, ptr %6, align 4
  %25 = load i32, ptr %9, align 4
  %26 = load ptr, ptr %8, align 8
  call void @rname(ptr noundef %23, i32 noundef %24, i32 noundef %25, ptr noundef %26)
  br label %27

27:                                               ; preds = %22, %17
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @findsetreg(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  %16 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  store i32 -1, ptr %8, align 4
  store i32 0, ptr %9, align 4
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.Proto, ptr %17, i32 0, i32 16
  %19 = load ptr, ptr %18, align 8
  %20 = load i32, ptr %5, align 4
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds i32, ptr %19, i64 %21
  %23 = load i32, ptr %22, align 4
  %24 = lshr i32 %23, 0
  %25 = and i32 %24, 127
  %26 = zext i32 %25 to i64
  %27 = getelementptr inbounds [83 x i8], ptr @luaP_opmodes, i64 0, i64 %26
  %28 = load i8, ptr %27, align 1
  %29 = zext i8 %28 to i32
  %30 = and i32 %29, 128
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %35

32:                                               ; preds = %3
  %33 = load i32, ptr %5, align 4
  %34 = add nsw i32 %33, -1
  store i32 %34, ptr %5, align 4
  br label %35

35:                                               ; preds = %32, %3
  store i32 0, ptr %7, align 4
  br label %36

36:                                               ; preds = %124, %35
  %37 = load i32, ptr %7, align 4
  %38 = load i32, ptr %5, align 4
  %39 = icmp slt i32 %37, %38
  br i1 %39, label %40, label %127

40:                                               ; preds = %36
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.Proto, ptr %41, i32 0, i32 16
  %43 = load ptr, ptr %42, align 8
  %44 = load i32, ptr %7, align 4
  %45 = sext i32 %44 to i64
  %46 = getelementptr inbounds i32, ptr %43, i64 %45
  %47 = load i32, ptr %46, align 4
  store i32 %47, ptr %10, align 4
  %48 = load i32, ptr %10, align 4
  %49 = lshr i32 %48, 0
  %50 = and i32 %49, 127
  store i32 %50, ptr %11, align 4
  %51 = load i32, ptr %10, align 4
  %52 = lshr i32 %51, 7
  %53 = and i32 %52, 255
  store i32 %53, ptr %12, align 4
  %54 = load i32, ptr %11, align 4
  switch i32 %54, label %101 [
    i32 8, label %55
    i32 76, label %71
    i32 68, label %77
    i32 69, label %77
    i32 56, label %82
  ]

55:                                               ; preds = %40
  %56 = load i32, ptr %10, align 4
  %57 = lshr i32 %56, 16
  %58 = and i32 %57, 255
  store i32 %58, ptr %14, align 4
  %59 = load i32, ptr %12, align 4
  %60 = load i32, ptr %6, align 4
  %61 = icmp sle i32 %59, %60
  br i1 %61, label %62, label %68

62:                                               ; preds = %55
  %63 = load i32, ptr %6, align 4
  %64 = load i32, ptr %12, align 4
  %65 = load i32, ptr %14, align 4
  %66 = add nsw i32 %64, %65
  %67 = icmp sle i32 %63, %66
  br label %68

68:                                               ; preds = %62, %55
  %69 = phi i1 [ false, %55 ], [ %67, %62 ]
  %70 = zext i1 %69 to i32
  store i32 %70, ptr %13, align 4
  br label %116

71:                                               ; preds = %40
  %72 = load i32, ptr %6, align 4
  %73 = load i32, ptr %12, align 4
  %74 = add nsw i32 %73, 2
  %75 = icmp sge i32 %72, %74
  %76 = zext i1 %75 to i32
  store i32 %76, ptr %13, align 4
  br label %116

77:                                               ; preds = %40, %40
  %78 = load i32, ptr %6, align 4
  %79 = load i32, ptr %12, align 4
  %80 = icmp sge i32 %78, %79
  %81 = zext i1 %80 to i32
  store i32 %81, ptr %13, align 4
  br label %116

82:                                               ; preds = %40
  %83 = load i32, ptr %10, align 4
  %84 = lshr i32 %83, 7
  %85 = and i32 %84, 33554431
  %86 = sub nsw i32 %85, 16777215
  store i32 %86, ptr %15, align 4
  %87 = load i32, ptr %7, align 4
  %88 = add nsw i32 %87, 1
  %89 = load i32, ptr %15, align 4
  %90 = add nsw i32 %88, %89
  store i32 %90, ptr %16, align 4
  %91 = load i32, ptr %16, align 4
  %92 = load i32, ptr %5, align 4
  %93 = icmp sle i32 %91, %92
  br i1 %93, label %94, label %100

94:                                               ; preds = %82
  %95 = load i32, ptr %16, align 4
  %96 = load i32, ptr %9, align 4
  %97 = icmp sgt i32 %95, %96
  br i1 %97, label %98, label %100

98:                                               ; preds = %94
  %99 = load i32, ptr %16, align 4
  store i32 %99, ptr %9, align 4
  br label %100

100:                                              ; preds = %98, %94, %82
  store i32 0, ptr %13, align 4
  br label %116

101:                                              ; preds = %40
  %102 = load i32, ptr %11, align 4
  %103 = zext i32 %102 to i64
  %104 = getelementptr inbounds [83 x i8], ptr @luaP_opmodes, i64 0, i64 %103
  %105 = load i8, ptr %104, align 1
  %106 = zext i8 %105 to i32
  %107 = and i32 %106, 8
  %108 = icmp ne i32 %107, 0
  br i1 %108, label %109, label %113

109:                                              ; preds = %101
  %110 = load i32, ptr %6, align 4
  %111 = load i32, ptr %12, align 4
  %112 = icmp eq i32 %110, %111
  br label %113

113:                                              ; preds = %109, %101
  %114 = phi i1 [ false, %101 ], [ %112, %109 ]
  %115 = zext i1 %114 to i32
  store i32 %115, ptr %13, align 4
  br label %116

116:                                              ; preds = %113, %100, %77, %71, %68
  %117 = load i32, ptr %13, align 4
  %118 = icmp ne i32 %117, 0
  br i1 %118, label %119, label %123

119:                                              ; preds = %116
  %120 = load i32, ptr %7, align 4
  %121 = load i32, ptr %9, align 4
  %122 = call i32 @filterpc(i32 noundef %120, i32 noundef %121)
  store i32 %122, ptr %8, align 4
  br label %123

123:                                              ; preds = %119, %116
  br label %124

124:                                              ; preds = %123
  %125 = load i32, ptr %7, align 4
  %126 = add nsw i32 %125, 1
  store i32 %126, ptr %7, align 4
  br label %36, !llvm.loop !15

127:                                              ; preds = %36
  %128 = load i32, ptr %8, align 4
  ret i32 %128
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @filterpc(i32 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  %6 = load i32, ptr %4, align 4
  %7 = load i32, ptr %5, align 4
  %8 = icmp slt i32 %6, %7
  br i1 %8, label %9, label %10

9:                                                ; preds = %2
  store i32 -1, ptr %3, align 4
  br label %12

10:                                               ; preds = %2
  %11 = load i32, ptr %4, align 4
  store i32 %11, ptr %3, align 4
  br label %12

12:                                               ; preds = %10, %9
  %13 = load i32, ptr %3, align 4
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @funcnamefromcode(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  store i32 0, ptr %10, align 4
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 16
  %14 = load ptr, ptr %13, align 8
  %15 = load i32, ptr %8, align 4
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds i32, ptr %14, i64 %16
  %18 = load i32, ptr %17, align 4
  store i32 %18, ptr %11, align 4
  %19 = load i32, ptr %11, align 4
  %20 = lshr i32 %19, 0
  %21 = and i32 %20, 127
  switch i32 %21, label %46 [
    i32 68, label %22
    i32 69, label %22
    i32 76, label %30
    i32 20, label %32
    i32 11, label %32
    i32 12, label %32
    i32 13, label %32
    i32 14, label %32
    i32 15, label %33
    i32 16, label %33
    i32 17, label %33
    i32 18, label %33
    i32 46, label %34
    i32 47, label %34
    i32 48, label %34
    i32 49, label %38
    i32 50, label %39
    i32 52, label %40
    i32 53, label %41
    i32 57, label %42
    i32 58, label %43
    i32 62, label %43
    i32 64, label %43
    i32 59, label %44
    i32 63, label %44
    i32 65, label %44
    i32 54, label %45
    i32 70, label %45
  ]

22:                                               ; preds = %4, %4
  %23 = load ptr, ptr %7, align 8
  %24 = load i32, ptr %8, align 4
  %25 = load i32, ptr %11, align 4
  %26 = lshr i32 %25, 7
  %27 = and i32 %26, 255
  %28 = load ptr, ptr %9, align 8
  %29 = call ptr @getobjname(ptr noundef %23, i32 noundef %24, i32 noundef %27, ptr noundef %28)
  store ptr %29, ptr %5, align 8
  br label %60

30:                                               ; preds = %4
  %31 = load ptr, ptr %9, align 8
  store ptr @.str.29, ptr %31, align 8
  store ptr @.str.29, ptr %5, align 8
  br label %60

32:                                               ; preds = %4, %4, %4, %4, %4
  store i32 0, ptr %10, align 4
  br label %47

33:                                               ; preds = %4, %4, %4, %4
  store i32 1, ptr %10, align 4
  br label %47

34:                                               ; preds = %4, %4, %4
  %35 = load i32, ptr %11, align 4
  %36 = lshr i32 %35, 24
  %37 = and i32 %36, 255
  store i32 %37, ptr %10, align 4
  br label %47

38:                                               ; preds = %4
  store i32 18, ptr %10, align 4
  br label %47

39:                                               ; preds = %4
  store i32 19, ptr %10, align 4
  br label %47

40:                                               ; preds = %4
  store i32 4, ptr %10, align 4
  br label %47

41:                                               ; preds = %4
  store i32 22, ptr %10, align 4
  br label %47

42:                                               ; preds = %4
  store i32 5, ptr %10, align 4
  br label %47

43:                                               ; preds = %4, %4, %4
  store i32 20, ptr %10, align 4
  br label %47

44:                                               ; preds = %4, %4, %4
  store i32 21, ptr %10, align 4
  br label %47

45:                                               ; preds = %4, %4
  store i32 24, ptr %10, align 4
  br label %47

46:                                               ; preds = %4
  store ptr null, ptr %5, align 8
  br label %60

47:                                               ; preds = %45, %44, %43, %42, %41, %40, %39, %38, %34, %33, %32
  %48 = load ptr, ptr %6, align 8
  %49 = getelementptr inbounds %struct.lua_State, ptr %48, i32 0, i32 7
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.global_State, ptr %50, i32 0, i32 42
  %52 = load i32, ptr %10, align 4
  %53 = zext i32 %52 to i64
  %54 = getelementptr inbounds [25 x ptr], ptr %51, i64 0, i64 %53
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.TString, ptr %55, i32 0, i32 7
  %57 = getelementptr inbounds [1 x i8], ptr %56, i64 0, i64 0
  %58 = getelementptr inbounds i8, ptr %57, i64 2
  %59 = load ptr, ptr %9, align 8
  store ptr %58, ptr %59, align 8
  store ptr @.str.28, ptr %5, align 8
  br label %60

60:                                               ; preds = %47, %46, %30, %22
  %61 = load ptr, ptr %5, align 8
  ret ptr %61
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { nocallback nofree nosync nounwind willreturn }
attributes #7 = { nounwind willreturn memory(read) }
attributes #8 = { noreturn }

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
!11 = distinct !{!11, !7}
!12 = distinct !{!12, !7}
!13 = distinct !{!13, !7}
!14 = distinct !{!14, !7}
!15 = distinct !{!15, !7}
