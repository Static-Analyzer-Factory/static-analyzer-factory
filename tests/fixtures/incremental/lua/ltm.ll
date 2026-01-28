; ModuleID = 'ltm.c'
source_filename = "ltm.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.1, i16, i16 }
%union.anon = type { %struct.anon.0 }
%struct.anon.0 = type { ptr, i64, i64 }
%union.anon.1 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }
%struct.Udata = type { ptr, i8, i8, i16, i64, ptr, ptr, [1 x %union.UValue] }
%union.UValue = type { %struct.TValue }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.3, [1 x i8] }
%union.anon.3 = type { i64 }
%union.StackValue = type { %struct.TValue }
%struct.anon = type { ptr, i32, i32 }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }

@.str = private unnamed_addr constant [9 x i8] c"no value\00", align 1
@.str.1 = private unnamed_addr constant [4 x i8] c"nil\00", align 1
@.str.2 = private unnamed_addr constant [8 x i8] c"boolean\00", align 1
@udatatypename = internal constant [9 x i8] c"userdata\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"number\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"string\00", align 1
@.str.5 = private unnamed_addr constant [6 x i8] c"table\00", align 1
@.str.6 = private unnamed_addr constant [9 x i8] c"function\00", align 1
@.str.7 = private unnamed_addr constant [7 x i8] c"thread\00", align 1
@.str.8 = private unnamed_addr constant [8 x i8] c"upvalue\00", align 1
@.str.9 = private unnamed_addr constant [6 x i8] c"proto\00", align 1
@luaT_typenames_ = hidden constant [12 x ptr] [ptr @.str, ptr @.str.1, ptr @.str.2, ptr @udatatypename, ptr @.str.3, ptr @.str.4, ptr @.str.5, ptr @.str.6, ptr @udatatypename, ptr @.str.7, ptr @.str.8, ptr @.str.9], align 16
@luaT_init.luaT_eventname = internal constant [25 x ptr] [ptr @.str.10, ptr @.str.11, ptr @.str.12, ptr @.str.13, ptr @.str.14, ptr @.str.15, ptr @.str.16, ptr @.str.17, ptr @.str.18, ptr @.str.19, ptr @.str.20, ptr @.str.21, ptr @.str.22, ptr @.str.23, ptr @.str.24, ptr @.str.25, ptr @.str.26, ptr @.str.27, ptr @.str.28, ptr @.str.29, ptr @.str.30, ptr @.str.31, ptr @.str.32, ptr @.str.33, ptr @.str.34], align 16
@.str.10 = private unnamed_addr constant [8 x i8] c"__index\00", align 1
@.str.11 = private unnamed_addr constant [11 x i8] c"__newindex\00", align 1
@.str.12 = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@.str.13 = private unnamed_addr constant [7 x i8] c"__mode\00", align 1
@.str.14 = private unnamed_addr constant [6 x i8] c"__len\00", align 1
@.str.15 = private unnamed_addr constant [5 x i8] c"__eq\00", align 1
@.str.16 = private unnamed_addr constant [6 x i8] c"__add\00", align 1
@.str.17 = private unnamed_addr constant [6 x i8] c"__sub\00", align 1
@.str.18 = private unnamed_addr constant [6 x i8] c"__mul\00", align 1
@.str.19 = private unnamed_addr constant [6 x i8] c"__mod\00", align 1
@.str.20 = private unnamed_addr constant [6 x i8] c"__pow\00", align 1
@.str.21 = private unnamed_addr constant [6 x i8] c"__div\00", align 1
@.str.22 = private unnamed_addr constant [7 x i8] c"__idiv\00", align 1
@.str.23 = private unnamed_addr constant [7 x i8] c"__band\00", align 1
@.str.24 = private unnamed_addr constant [6 x i8] c"__bor\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"__bxor\00", align 1
@.str.26 = private unnamed_addr constant [6 x i8] c"__shl\00", align 1
@.str.27 = private unnamed_addr constant [6 x i8] c"__shr\00", align 1
@.str.28 = private unnamed_addr constant [6 x i8] c"__unm\00", align 1
@.str.29 = private unnamed_addr constant [7 x i8] c"__bnot\00", align 1
@.str.30 = private unnamed_addr constant [5 x i8] c"__lt\00", align 1
@.str.31 = private unnamed_addr constant [5 x i8] c"__le\00", align 1
@.str.32 = private unnamed_addr constant [9 x i8] c"__concat\00", align 1
@.str.33 = private unnamed_addr constant [7 x i8] c"__call\00", align 1
@.str.34 = private unnamed_addr constant [8 x i8] c"__close\00", align 1
@.str.35 = private unnamed_addr constant [7 x i8] c"__name\00", align 1
@.str.36 = private unnamed_addr constant [29 x i8] c"perform bitwise operation on\00", align 1
@.str.37 = private unnamed_addr constant [22 x i8] c"perform arithmetic on\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_init(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 0, ptr %3, align 4
  br label %4

4:                                                ; preds = %30, %1
  %5 = load i32, ptr %3, align 4
  %6 = icmp slt i32 %5, 25
  br i1 %6, label %7, label %33

7:                                                ; preds = %4
  %8 = load ptr, ptr %2, align 8
  %9 = load i32, ptr %3, align 4
  %10 = sext i32 %9 to i64
  %11 = getelementptr inbounds [25 x ptr], ptr @luaT_init.luaT_eventname, i64 0, i64 %10
  %12 = load ptr, ptr %11, align 8
  %13 = call ptr @luaS_new(ptr noundef %8, ptr noundef %12)
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 7
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 42
  %18 = load i32, ptr %3, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds [25 x ptr], ptr %17, i64 0, i64 %19
  store ptr %13, ptr %20, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = load ptr, ptr %2, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 7
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.global_State, ptr %24, i32 0, i32 42
  %26 = load i32, ptr %3, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds [25 x ptr], ptr %25, i64 0, i64 %27
  %29 = load ptr, ptr %28, align 8
  call void @luaC_fix(ptr noundef %21, ptr noundef %29)
  br label %30

30:                                               ; preds = %7
  %31 = load i32, ptr %3, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %3, align 4
  br label %4, !llvm.loop !6

33:                                               ; preds = %4
  ret void
}

declare hidden ptr @luaS_new(ptr noundef, ptr noundef) #1

declare hidden void @luaC_fix(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaT_gettm(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %7, align 8
  %11 = call ptr @luaH_getshortstr(ptr noundef %9, ptr noundef %10)
  store ptr %11, ptr %8, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 15
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %29

18:                                               ; preds = %3
  %19 = load i32, ptr %6, align 4
  %20 = shl i32 1, %19
  %21 = trunc i32 %20 to i8
  %22 = zext i8 %21 to i32
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Table, ptr %23, i32 0, i32 3
  %25 = load i8, ptr %24, align 2
  %26 = zext i8 %25 to i32
  %27 = or i32 %26, %22
  %28 = trunc i32 %27 to i8
  store i8 %28, ptr %24, align 2
  store ptr null, ptr %4, align 8
  br label %31

29:                                               ; preds = %3
  %30 = load ptr, ptr %8, align 8
  store ptr %30, ptr %4, align 8
  br label %31

31:                                               ; preds = %29, %18
  %32 = load ptr, ptr %4, align 8
  ret ptr %32
}

declare hidden ptr @luaH_getshortstr(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaT_gettmbyobj(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  switch i32 %12, label %25 [
    i32 5, label %13
    i32 7, label %19
  ]

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.Table, ptr %16, i32 0, i32 9
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %7, align 8
  br label %38

19:                                               ; preds = %3
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.Udata, ptr %22, i32 0, i32 5
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %7, align 8
  br label %38

25:                                               ; preds = %3
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.lua_State, ptr %26, i32 0, i32 7
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 43
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  %32 = load i8, ptr %31, align 8
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, 15
  %35 = sext i32 %34 to i64
  %36 = getelementptr inbounds [9 x ptr], ptr %29, i64 0, i64 %35
  %37 = load ptr, ptr %36, align 8
  store ptr %37, ptr %7, align 8
  br label %38

38:                                               ; preds = %25, %19, %13
  %39 = load ptr, ptr %7, align 8
  %40 = icmp ne ptr %39, null
  br i1 %40, label %41, label %52

41:                                               ; preds = %38
  %42 = load ptr, ptr %7, align 8
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 7
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 42
  %47 = load i32, ptr %6, align 4
  %48 = zext i32 %47 to i64
  %49 = getelementptr inbounds [25 x ptr], ptr %46, i64 0, i64 %48
  %50 = load ptr, ptr %49, align 8
  %51 = call ptr @luaH_getshortstr(ptr noundef %42, ptr noundef %50)
  br label %57

52:                                               ; preds = %38
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 7
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.global_State, ptr %55, i32 0, i32 8
  br label %57

57:                                               ; preds = %52, %41
  %58 = phi ptr [ %51, %41 ], [ %56, %52 ]
  ret ptr %58
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaT_objtypename(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = icmp eq i32 %11, 69
  br i1 %12, label %13, label %20

13:                                               ; preds = %2
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.Table, ptr %16, i32 0, i32 9
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %6, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %33, label %20

20:                                               ; preds = %13, %2
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 1
  %23 = load i8, ptr %22, align 8
  %24 = zext i8 %23 to i32
  %25 = icmp eq i32 %24, 71
  br i1 %25, label %26, label %51

26:                                               ; preds = %20
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.Udata, ptr %29, i32 0, i32 5
  %31 = load ptr, ptr %30, align 8
  store ptr %31, ptr %6, align 8
  %32 = icmp ne ptr %31, null
  br i1 %32, label %33, label %51

33:                                               ; preds = %26, %13
  %34 = load ptr, ptr %6, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = call ptr @luaS_new(ptr noundef %35, ptr noundef @.str.35)
  %37 = call ptr @luaH_getshortstr(ptr noundef %34, ptr noundef %36)
  store ptr %37, ptr %7, align 8
  %38 = load ptr, ptr %7, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 1
  %40 = load i8, ptr %39, align 8
  %41 = zext i8 %40 to i32
  %42 = and i32 %41, 15
  %43 = icmp eq i32 %42, 4
  br i1 %43, label %44, label %50

44:                                               ; preds = %33
  %45 = load ptr, ptr %7, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 0
  %47 = load ptr, ptr %46, align 8
  %48 = getelementptr inbounds %struct.TString, ptr %47, i32 0, i32 7
  %49 = getelementptr inbounds [1 x i8], ptr %48, i64 0, i64 0
  store ptr %49, ptr %3, align 8
  br label %61

50:                                               ; preds = %33
  br label %51

51:                                               ; preds = %50, %26, %20
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 1
  %54 = load i8, ptr %53, align 8
  %55 = zext i8 %54 to i32
  %56 = and i32 %55, 15
  %57 = add nsw i32 %56, 1
  %58 = sext i32 %57 to i64
  %59 = getelementptr inbounds [12 x ptr], ptr @luaT_typenames_, i64 0, i64 %58
  %60 = load ptr, ptr %59, align 8
  store ptr %60, ptr %3, align 8
  br label %61

61:                                               ; preds = %51, %44
  %62 = load ptr, ptr %3, align 8
  ret ptr %62
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_callTM(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 6
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr %11, align 8
  %23 = load ptr, ptr %11, align 8
  store ptr %23, ptr %12, align 8
  %24 = load ptr, ptr %7, align 8
  store ptr %24, ptr %13, align 8
  %25 = load ptr, ptr %12, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %13, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %26, ptr align 8 %28, i64 8, i1 false)
  %29 = load ptr, ptr %13, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = load ptr, ptr %12, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  store i8 %31, ptr %33, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = load ptr, ptr %11, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i64 1
  store ptr %36, ptr %14, align 8
  %37 = load ptr, ptr %8, align 8
  store ptr %37, ptr %15, align 8
  %38 = load ptr, ptr %14, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  %40 = load ptr, ptr %15, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %39, ptr align 8 %41, i64 8, i1 false)
  %42 = load ptr, ptr %15, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 1
  %44 = load i8, ptr %43, align 8
  %45 = load ptr, ptr %14, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 1
  store i8 %44, ptr %46, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = load ptr, ptr %11, align 8
  %49 = getelementptr inbounds %union.StackValue, ptr %48, i64 2
  store ptr %49, ptr %16, align 8
  %50 = load ptr, ptr %9, align 8
  store ptr %50, ptr %17, align 8
  %51 = load ptr, ptr %16, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %17, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %52, ptr align 8 %54, i64 8, i1 false)
  %55 = load ptr, ptr %17, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 8
  %58 = load ptr, ptr %16, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 1
  store i8 %57, ptr %59, align 8
  %60 = load ptr, ptr %6, align 8
  %61 = load ptr, ptr %11, align 8
  %62 = getelementptr inbounds %union.StackValue, ptr %61, i64 3
  store ptr %62, ptr %18, align 8
  %63 = load ptr, ptr %10, align 8
  store ptr %63, ptr %19, align 8
  %64 = load ptr, ptr %18, align 8
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 0
  %66 = load ptr, ptr %19, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %65, ptr align 8 %67, i64 8, i1 false)
  %68 = load ptr, ptr %19, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 1
  %70 = load i8, ptr %69, align 8
  %71 = load ptr, ptr %18, align 8
  %72 = getelementptr inbounds %struct.TValue, ptr %71, i32 0, i32 1
  store i8 %70, ptr %72, align 8
  %73 = load ptr, ptr %6, align 8
  %74 = load ptr, ptr %11, align 8
  %75 = getelementptr inbounds %union.StackValue, ptr %74, i64 4
  %76 = load ptr, ptr %6, align 8
  %77 = getelementptr inbounds %struct.lua_State, ptr %76, i32 0, i32 6
  store ptr %75, ptr %77, align 8
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.lua_State, ptr %78, i32 0, i32 8
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds %struct.CallInfo, ptr %80, i32 0, i32 7
  %82 = load i16, ptr %81, align 2
  %83 = zext i16 %82 to i32
  %84 = and i32 %83, 10
  %85 = icmp ne i32 %84, 0
  br i1 %85, label %89, label %86

86:                                               ; preds = %5
  %87 = load ptr, ptr %6, align 8
  %88 = load ptr, ptr %11, align 8
  call void @luaD_call(ptr noundef %87, ptr noundef %88, i32 noundef 0)
  br label %92

89:                                               ; preds = %5
  %90 = load ptr, ptr %6, align 8
  %91 = load ptr, ptr %11, align 8
  call void @luaD_callnoyield(ptr noundef %90, ptr noundef %91, i32 noundef 0)
  br label %92

92:                                               ; preds = %89, %86
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

declare hidden void @luaD_call(ptr noundef, ptr noundef, i32 noundef) #1

declare hidden void @luaD_callnoyield(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_callTMres(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  %20 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %21 = load ptr, ptr %10, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 10
  %24 = load ptr, ptr %23, align 8
  %25 = ptrtoint ptr %21 to i64
  %26 = ptrtoint ptr %24 to i64
  %27 = sub i64 %25, %26
  store i64 %27, ptr %11, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 6
  %30 = load ptr, ptr %29, align 8
  store ptr %30, ptr %12, align 8
  %31 = load ptr, ptr %12, align 8
  store ptr %31, ptr %13, align 8
  %32 = load ptr, ptr %7, align 8
  store ptr %32, ptr %14, align 8
  %33 = load ptr, ptr %13, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %14, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %34, ptr align 8 %36, i64 8, i1 false)
  %37 = load ptr, ptr %14, align 8
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 1
  %39 = load i8, ptr %38, align 8
  %40 = load ptr, ptr %13, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 1
  store i8 %39, ptr %41, align 8
  %42 = load ptr, ptr %6, align 8
  %43 = load ptr, ptr %12, align 8
  %44 = getelementptr inbounds %union.StackValue, ptr %43, i64 1
  store ptr %44, ptr %15, align 8
  %45 = load ptr, ptr %8, align 8
  store ptr %45, ptr %16, align 8
  %46 = load ptr, ptr %15, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %16, align 8
  %49 = getelementptr inbounds %struct.TValue, ptr %48, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %47, ptr align 8 %49, i64 8, i1 false)
  %50 = load ptr, ptr %16, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  %52 = load i8, ptr %51, align 8
  %53 = load ptr, ptr %15, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 1
  store i8 %52, ptr %54, align 8
  %55 = load ptr, ptr %6, align 8
  %56 = load ptr, ptr %12, align 8
  %57 = getelementptr inbounds %union.StackValue, ptr %56, i64 2
  store ptr %57, ptr %17, align 8
  %58 = load ptr, ptr %9, align 8
  store ptr %58, ptr %18, align 8
  %59 = load ptr, ptr %17, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  %61 = load ptr, ptr %18, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %60, ptr align 8 %62, i64 8, i1 false)
  %63 = load ptr, ptr %18, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 1
  %65 = load i8, ptr %64, align 8
  %66 = load ptr, ptr %17, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 1
  store i8 %65, ptr %67, align 8
  %68 = load ptr, ptr %6, align 8
  %69 = load ptr, ptr %6, align 8
  %70 = getelementptr inbounds %struct.lua_State, ptr %69, i32 0, i32 6
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %union.StackValue, ptr %71, i64 3
  store ptr %72, ptr %70, align 8
  %73 = load ptr, ptr %6, align 8
  %74 = getelementptr inbounds %struct.lua_State, ptr %73, i32 0, i32 8
  %75 = load ptr, ptr %74, align 8
  %76 = getelementptr inbounds %struct.CallInfo, ptr %75, i32 0, i32 7
  %77 = load i16, ptr %76, align 2
  %78 = zext i16 %77 to i32
  %79 = and i32 %78, 10
  %80 = icmp ne i32 %79, 0
  br i1 %80, label %84, label %81

81:                                               ; preds = %5
  %82 = load ptr, ptr %6, align 8
  %83 = load ptr, ptr %12, align 8
  call void @luaD_call(ptr noundef %82, ptr noundef %83, i32 noundef 1)
  br label %87

84:                                               ; preds = %5
  %85 = load ptr, ptr %6, align 8
  %86 = load ptr, ptr %12, align 8
  call void @luaD_callnoyield(ptr noundef %85, ptr noundef %86, i32 noundef 1)
  br label %87

87:                                               ; preds = %84, %81
  %88 = load ptr, ptr %6, align 8
  %89 = getelementptr inbounds %struct.lua_State, ptr %88, i32 0, i32 10
  %90 = load ptr, ptr %89, align 8
  %91 = load i64, ptr %11, align 8
  %92 = getelementptr inbounds i8, ptr %90, i64 %91
  store ptr %92, ptr %10, align 8
  %93 = load ptr, ptr %10, align 8
  store ptr %93, ptr %19, align 8
  %94 = load ptr, ptr %6, align 8
  %95 = getelementptr inbounds %struct.lua_State, ptr %94, i32 0, i32 6
  %96 = load ptr, ptr %95, align 8
  %97 = getelementptr inbounds %union.StackValue, ptr %96, i32 -1
  store ptr %97, ptr %95, align 8
  store ptr %97, ptr %20, align 8
  %98 = load ptr, ptr %19, align 8
  %99 = getelementptr inbounds %struct.TValue, ptr %98, i32 0, i32 0
  %100 = load ptr, ptr %20, align 8
  %101 = getelementptr inbounds %struct.TValue, ptr %100, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %99, ptr align 8 %101, i64 8, i1 false)
  %102 = load ptr, ptr %20, align 8
  %103 = getelementptr inbounds %struct.TValue, ptr %102, i32 0, i32 1
  %104 = load i8, ptr %103, align 8
  %105 = load ptr, ptr %19, align 8
  %106 = getelementptr inbounds %struct.TValue, ptr %105, i32 0, i32 1
  store i8 %104, ptr %106, align 8
  %107 = load ptr, ptr %6, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_trybinTM(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  %11 = load ptr, ptr %6, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = load i32, ptr %10, align 4
  %16 = call i32 @callbinTM(ptr noundef %11, ptr noundef %12, ptr noundef %13, ptr noundef %14, i32 noundef %15)
  %17 = icmp ne i32 %16, 0
  %18 = xor i1 %17, true
  %19 = zext i1 %18 to i32
  %20 = icmp ne i32 %19, 0
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %24, label %52

24:                                               ; preds = %5
  %25 = load i32, ptr %10, align 4
  switch i32 %25, label %48 [
    i32 13, label %26
    i32 14, label %26
    i32 15, label %26
    i32 16, label %26
    i32 17, label %26
    i32 19, label %26
  ]

26:                                               ; preds = %24, %24, %24, %24, %24, %24
  %27 = load ptr, ptr %7, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 15
  %32 = icmp eq i32 %31, 3
  br i1 %32, label %33, label %44

33:                                               ; preds = %26
  %34 = load ptr, ptr %8, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  %36 = load i8, ptr %35, align 8
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 15
  %39 = icmp eq i32 %38, 3
  br i1 %39, label %40, label %44

40:                                               ; preds = %33
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = load ptr, ptr %8, align 8
  call void @luaG_tointerror(ptr noundef %41, ptr noundef %42, ptr noundef %43) #4
  unreachable

44:                                               ; preds = %33, %26
  %45 = load ptr, ptr %6, align 8
  %46 = load ptr, ptr %7, align 8
  %47 = load ptr, ptr %8, align 8
  call void @luaG_opinterror(ptr noundef %45, ptr noundef %46, ptr noundef %47, ptr noundef @.str.36) #4
  unreachable

48:                                               ; preds = %24
  %49 = load ptr, ptr %6, align 8
  %50 = load ptr, ptr %7, align 8
  %51 = load ptr, ptr %8, align 8
  call void @luaG_opinterror(ptr noundef %49, ptr noundef %50, ptr noundef %51, ptr noundef @.str.37) #4
  unreachable

52:                                               ; preds = %5
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @callbinTM(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #0 {
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i32 %4, ptr %11, align 4
  %13 = load ptr, ptr %7, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = load i32, ptr %11, align 4
  %16 = call ptr @luaT_gettmbyobj(ptr noundef %13, ptr noundef %14, i32 noundef %15)
  store ptr %16, ptr %12, align 8
  %17 = load ptr, ptr %12, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = and i32 %20, 15
  %22 = icmp eq i32 %21, 0
  br i1 %22, label %23, label %28

23:                                               ; preds = %5
  %24 = load ptr, ptr %7, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = load i32, ptr %11, align 4
  %27 = call ptr @luaT_gettmbyobj(ptr noundef %24, ptr noundef %25, i32 noundef %26)
  store ptr %27, ptr %12, align 8
  br label %28

28:                                               ; preds = %23, %5
  %29 = load ptr, ptr %12, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 15
  %34 = icmp eq i32 %33, 0
  br i1 %34, label %35, label %36

35:                                               ; preds = %28
  store i32 0, ptr %6, align 4
  br label %42

36:                                               ; preds = %28
  %37 = load ptr, ptr %7, align 8
  %38 = load ptr, ptr %12, align 8
  %39 = load ptr, ptr %8, align 8
  %40 = load ptr, ptr %9, align 8
  %41 = load ptr, ptr %10, align 8
  call void @luaT_callTMres(ptr noundef %37, ptr noundef %38, ptr noundef %39, ptr noundef %40, ptr noundef %41)
  store i32 1, ptr %6, align 4
  br label %42

42:                                               ; preds = %36, %35
  %43 = load i32, ptr %6, align 4
  ret i32 %43
}

; Function Attrs: noreturn
declare hidden void @luaG_tointerror(ptr noundef, ptr noundef, ptr noundef) #3

; Function Attrs: noreturn
declare hidden void @luaG_opinterror(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_tryconcatTM(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 6
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %union.StackValue, ptr %8, i64 -2
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %union.StackValue, ptr %10, i64 -1
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %union.StackValue, ptr %12, i64 -2
  %14 = call i32 @callbinTM(ptr noundef %7, ptr noundef %9, ptr noundef %11, ptr noundef %13, i32 noundef 22)
  %15 = icmp ne i32 %14, 0
  %16 = xor i1 %15, true
  %17 = zext i1 %16 to i32
  %18 = icmp ne i32 %17, 0
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %22, label %28

22:                                               ; preds = %1
  %23 = load ptr, ptr %2, align 8
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %union.StackValue, ptr %24, i64 -2
  %26 = load ptr, ptr %3, align 8
  %27 = getelementptr inbounds %union.StackValue, ptr %26, i64 -1
  call void @luaG_concaterror(ptr noundef %23, ptr noundef %25, ptr noundef %27) #4
  unreachable

28:                                               ; preds = %1
  ret void
}

; Function Attrs: noreturn
declare hidden void @luaG_concaterror(ptr noundef, ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_trybinassocTM(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, ptr noundef %4, i32 noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store i32 %3, ptr %10, align 4
  store ptr %4, ptr %11, align 8
  store i32 %5, ptr %12, align 4
  %13 = load i32, ptr %10, align 4
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %15, label %21

15:                                               ; preds = %6
  %16 = load ptr, ptr %7, align 8
  %17 = load ptr, ptr %9, align 8
  %18 = load ptr, ptr %8, align 8
  %19 = load ptr, ptr %11, align 8
  %20 = load i32, ptr %12, align 4
  call void @luaT_trybinTM(ptr noundef %16, ptr noundef %17, ptr noundef %18, ptr noundef %19, i32 noundef %20)
  br label %27

21:                                               ; preds = %6
  %22 = load ptr, ptr %7, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = load ptr, ptr %9, align 8
  %25 = load ptr, ptr %11, align 8
  %26 = load i32, ptr %12, align 4
  call void @luaT_trybinTM(ptr noundef %22, ptr noundef %23, ptr noundef %24, ptr noundef %25, i32 noundef %26)
  br label %27

27:                                               ; preds = %21, %15
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_trybiniTM(ptr noundef %0, ptr noundef %1, i64 noundef %2, i32 noundef %3, ptr noundef %4, i32 noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca %struct.TValue, align 8
  %14 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store i64 %2, ptr %9, align 8
  store i32 %3, ptr %10, align 4
  store ptr %4, ptr %11, align 8
  store i32 %5, ptr %12, align 4
  store ptr %13, ptr %14, align 8
  %15 = load i64, ptr %9, align 8
  %16 = load ptr, ptr %14, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 0
  store i64 %15, ptr %17, align 8
  %18 = load ptr, ptr %14, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 1
  store i8 3, ptr %19, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = load i32, ptr %10, align 4
  %23 = load ptr, ptr %11, align 8
  %24 = load i32, ptr %12, align 4
  call void @luaT_trybinassocTM(ptr noundef %20, ptr noundef %21, ptr noundef %13, i32 noundef %22, ptr noundef %23, i32 noundef %24)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaT_callorderTM(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 6
  %14 = load ptr, ptr %13, align 8
  %15 = load i32, ptr %8, align 4
  %16 = call i32 @callbinTM(ptr noundef %9, ptr noundef %10, ptr noundef %11, ptr noundef %14, i32 noundef %15)
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %39

18:                                               ; preds = %4
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 1
  %23 = load i8, ptr %22, align 8
  %24 = zext i8 %23 to i32
  %25 = icmp eq i32 %24, 1
  br i1 %25, label %35, label %26

26:                                               ; preds = %18
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 6
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 15
  %34 = icmp eq i32 %33, 0
  br label %35

35:                                               ; preds = %26, %18
  %36 = phi i1 [ true, %18 ], [ %34, %26 ]
  %37 = xor i1 %36, true
  %38 = zext i1 %37 to i32
  ret i32 %38

39:                                               ; preds = %4
  %40 = load ptr, ptr %5, align 8
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %7, align 8
  call void @luaG_ordererror(ptr noundef %40, ptr noundef %41, ptr noundef %42) #4
  unreachable
}

; Function Attrs: noreturn
declare hidden void @luaG_ordererror(ptr noundef, ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaT_callorderiTM(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4, i32 noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca %struct.TValue, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store i32 %2, ptr %9, align 4
  store i32 %3, ptr %10, align 4
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %17 = load i32, ptr %11, align 4
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %26

19:                                               ; preds = %6
  store ptr %13, ptr %15, align 8
  %20 = load i32, ptr %9, align 4
  %21 = sitofp i32 %20 to double
  %22 = load ptr, ptr %15, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  store double %21, ptr %23, align 8
  %24 = load ptr, ptr %15, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  store i8 19, ptr %25, align 8
  br label %33

26:                                               ; preds = %6
  store ptr %13, ptr %16, align 8
  %27 = load i32, ptr %9, align 4
  %28 = sext i32 %27 to i64
  %29 = load ptr, ptr %16, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 0
  store i64 %28, ptr %30, align 8
  %31 = load ptr, ptr %16, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 1
  store i8 3, ptr %32, align 8
  br label %33

33:                                               ; preds = %26, %19
  %34 = load i32, ptr %10, align 4
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %36, label %38

36:                                               ; preds = %33
  %37 = load ptr, ptr %8, align 8
  store ptr %37, ptr %14, align 8
  store ptr %13, ptr %8, align 8
  br label %39

38:                                               ; preds = %33
  store ptr %13, ptr %14, align 8
  br label %39

39:                                               ; preds = %38, %36
  %40 = load ptr, ptr %7, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = load ptr, ptr %14, align 8
  %43 = load i32, ptr %12, align 4
  %44 = call i32 @luaT_callorderTM(ptr noundef %40, ptr noundef %41, ptr noundef %42, i32 noundef %43)
  ret i32 %44
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_adjustvarargs(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  %22 = ptrtoint ptr %18 to i64
  %23 = ptrtoint ptr %21 to i64
  %24 = sub i64 %22, %23
  %25 = sdiv exact i64 %24, 16
  %26 = trunc i64 %25 to i32
  %27 = sub nsw i32 %26, 1
  store i32 %27, ptr %10, align 4
  %28 = load i32, ptr %10, align 4
  %29 = load i32, ptr %6, align 4
  %30 = sub nsw i32 %28, %29
  store i32 %30, ptr %11, align 4
  %31 = load i32, ptr %11, align 4
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.CallInfo, ptr %32, i32 0, i32 4
  %34 = getelementptr inbounds %struct.anon, ptr %33, i32 0, i32 2
  store i32 %31, ptr %34, align 4
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 9
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 6
  %40 = load ptr, ptr %39, align 8
  %41 = ptrtoint ptr %37 to i64
  %42 = ptrtoint ptr %40 to i64
  %43 = sub i64 %41, %42
  %44 = sdiv exact i64 %43, 16
  %45 = load ptr, ptr %8, align 8
  %46 = getelementptr inbounds %struct.Proto, ptr %45, i32 0, i32 5
  %47 = load i8, ptr %46, align 4
  %48 = zext i8 %47 to i32
  %49 = add nsw i32 %48, 1
  %50 = sext i32 %49 to i64
  %51 = icmp sle i64 %44, %50
  %52 = zext i1 %51 to i32
  %53 = icmp ne i32 %52, 0
  %54 = zext i1 %53 to i32
  %55 = sext i32 %54 to i64
  %56 = icmp ne i64 %55, 0
  br i1 %56, label %57, label %65

57:                                               ; preds = %4
  %58 = load ptr, ptr %5, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = getelementptr inbounds %struct.Proto, ptr %59, i32 0, i32 5
  %61 = load i8, ptr %60, align 4
  %62 = zext i8 %61 to i32
  %63 = add nsw i32 %62, 1
  %64 = call i32 @luaD_growstack(ptr noundef %58, i32 noundef %63, i32 noundef 1)
  br label %66

65:                                               ; preds = %4
  br label %66

66:                                               ; preds = %65, %57
  %67 = load ptr, ptr %5, align 8
  %68 = getelementptr inbounds %struct.lua_State, ptr %67, i32 0, i32 6
  %69 = load ptr, ptr %68, align 8
  %70 = getelementptr inbounds %union.StackValue, ptr %69, i32 1
  store ptr %70, ptr %68, align 8
  store ptr %69, ptr %12, align 8
  %71 = load ptr, ptr %7, align 8
  %72 = getelementptr inbounds %struct.CallInfo, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  store ptr %73, ptr %13, align 8
  %74 = load ptr, ptr %12, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  %76 = load ptr, ptr %13, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %75, ptr align 8 %77, i64 8, i1 false)
  %78 = load ptr, ptr %13, align 8
  %79 = getelementptr inbounds %struct.TValue, ptr %78, i32 0, i32 1
  %80 = load i8, ptr %79, align 8
  %81 = load ptr, ptr %12, align 8
  %82 = getelementptr inbounds %struct.TValue, ptr %81, i32 0, i32 1
  store i8 %80, ptr %82, align 8
  %83 = load ptr, ptr %5, align 8
  store i32 1, ptr %9, align 4
  br label %84

84:                                               ; preds = %116, %66
  %85 = load i32, ptr %9, align 4
  %86 = load i32, ptr %6, align 4
  %87 = icmp sle i32 %85, %86
  br i1 %87, label %88, label %119

88:                                               ; preds = %84
  %89 = load ptr, ptr %5, align 8
  %90 = getelementptr inbounds %struct.lua_State, ptr %89, i32 0, i32 6
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %union.StackValue, ptr %91, i32 1
  store ptr %92, ptr %90, align 8
  store ptr %91, ptr %14, align 8
  %93 = load ptr, ptr %7, align 8
  %94 = getelementptr inbounds %struct.CallInfo, ptr %93, i32 0, i32 0
  %95 = load ptr, ptr %94, align 8
  %96 = load i32, ptr %9, align 4
  %97 = sext i32 %96 to i64
  %98 = getelementptr inbounds %union.StackValue, ptr %95, i64 %97
  store ptr %98, ptr %15, align 8
  %99 = load ptr, ptr %14, align 8
  %100 = getelementptr inbounds %struct.TValue, ptr %99, i32 0, i32 0
  %101 = load ptr, ptr %15, align 8
  %102 = getelementptr inbounds %struct.TValue, ptr %101, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %100, ptr align 8 %102, i64 8, i1 false)
  %103 = load ptr, ptr %15, align 8
  %104 = getelementptr inbounds %struct.TValue, ptr %103, i32 0, i32 1
  %105 = load i8, ptr %104, align 8
  %106 = load ptr, ptr %14, align 8
  %107 = getelementptr inbounds %struct.TValue, ptr %106, i32 0, i32 1
  store i8 %105, ptr %107, align 8
  %108 = load ptr, ptr %5, align 8
  %109 = load ptr, ptr %7, align 8
  %110 = getelementptr inbounds %struct.CallInfo, ptr %109, i32 0, i32 0
  %111 = load ptr, ptr %110, align 8
  %112 = load i32, ptr %9, align 4
  %113 = sext i32 %112 to i64
  %114 = getelementptr inbounds %union.StackValue, ptr %111, i64 %113
  %115 = getelementptr inbounds %struct.TValue, ptr %114, i32 0, i32 1
  store i8 0, ptr %115, align 8
  br label %116

116:                                              ; preds = %88
  %117 = load i32, ptr %9, align 4
  %118 = add nsw i32 %117, 1
  store i32 %118, ptr %9, align 4
  br label %84, !llvm.loop !8

119:                                              ; preds = %84
  %120 = load i32, ptr %10, align 4
  %121 = add nsw i32 %120, 1
  %122 = load ptr, ptr %7, align 8
  %123 = getelementptr inbounds %struct.CallInfo, ptr %122, i32 0, i32 0
  %124 = load ptr, ptr %123, align 8
  %125 = sext i32 %121 to i64
  %126 = getelementptr inbounds %union.StackValue, ptr %124, i64 %125
  store ptr %126, ptr %123, align 8
  %127 = load i32, ptr %10, align 4
  %128 = add nsw i32 %127, 1
  %129 = load ptr, ptr %7, align 8
  %130 = getelementptr inbounds %struct.CallInfo, ptr %129, i32 0, i32 1
  %131 = load ptr, ptr %130, align 8
  %132 = sext i32 %128 to i64
  %133 = getelementptr inbounds %union.StackValue, ptr %131, i64 %132
  store ptr %133, ptr %130, align 8
  ret void
}

declare hidden i32 @luaD_growstack(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaT_getvarargs(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 4
  %16 = getelementptr inbounds %struct.anon, ptr %15, i32 0, i32 2
  %17 = load i32, ptr %16, align 4
  store i32 %17, ptr %10, align 4
  %18 = load i32, ptr %8, align 4
  %19 = icmp slt i32 %18, 0
  br i1 %19, label %20, label %73

20:                                               ; preds = %4
  %21 = load i32, ptr %10, align 4
  store i32 %21, ptr %8, align 4
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 9
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = ptrtoint ptr %24 to i64
  %29 = ptrtoint ptr %27 to i64
  %30 = sub i64 %28, %29
  %31 = sdiv exact i64 %30, 16
  %32 = load i32, ptr %10, align 4
  %33 = sext i32 %32 to i64
  %34 = icmp sle i64 %31, %33
  %35 = zext i1 %34 to i32
  %36 = icmp ne i32 %35, 0
  %37 = zext i1 %36 to i32
  %38 = sext i32 %37 to i64
  %39 = icmp ne i64 %38, 0
  br i1 %39, label %40, label %65

40:                                               ; preds = %20
  %41 = load ptr, ptr %7, align 8
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.lua_State, ptr %42, i32 0, i32 10
  %44 = load ptr, ptr %43, align 8
  %45 = ptrtoint ptr %41 to i64
  %46 = ptrtoint ptr %44 to i64
  %47 = sub i64 %45, %46
  store i64 %47, ptr %11, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.lua_State, ptr %48, i32 0, i32 7
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.global_State, ptr %50, i32 0, i32 3
  %52 = load i64, ptr %51, align 8
  %53 = icmp sgt i64 %52, 0
  br i1 %53, label %54, label %56

54:                                               ; preds = %40
  %55 = load ptr, ptr %5, align 8
  call void @luaC_step(ptr noundef %55)
  br label %56

56:                                               ; preds = %54, %40
  %57 = load ptr, ptr %5, align 8
  %58 = load i32, ptr %10, align 4
  %59 = call i32 @luaD_growstack(ptr noundef %57, i32 noundef %58, i32 noundef 1)
  %60 = load ptr, ptr %5, align 8
  %61 = getelementptr inbounds %struct.lua_State, ptr %60, i32 0, i32 10
  %62 = load ptr, ptr %61, align 8
  %63 = load i64, ptr %11, align 8
  %64 = getelementptr inbounds i8, ptr %62, i64 %63
  store ptr %64, ptr %7, align 8
  br label %66

65:                                               ; preds = %20
  br label %66

66:                                               ; preds = %65, %56
  %67 = load ptr, ptr %7, align 8
  %68 = load i32, ptr %10, align 4
  %69 = sext i32 %68 to i64
  %70 = getelementptr inbounds %union.StackValue, ptr %67, i64 %69
  %71 = load ptr, ptr %5, align 8
  %72 = getelementptr inbounds %struct.lua_State, ptr %71, i32 0, i32 6
  store ptr %70, ptr %72, align 8
  br label %73

73:                                               ; preds = %66, %4
  store i32 0, ptr %9, align 4
  br label %74

74:                                               ; preds = %109, %73
  %75 = load i32, ptr %9, align 4
  %76 = load i32, ptr %8, align 4
  %77 = icmp slt i32 %75, %76
  br i1 %77, label %78, label %82

78:                                               ; preds = %74
  %79 = load i32, ptr %9, align 4
  %80 = load i32, ptr %10, align 4
  %81 = icmp slt i32 %79, %80
  br label %82

82:                                               ; preds = %78, %74
  %83 = phi i1 [ false, %74 ], [ %81, %78 ]
  br i1 %83, label %84, label %112

84:                                               ; preds = %82
  %85 = load ptr, ptr %7, align 8
  %86 = load i32, ptr %9, align 4
  %87 = sext i32 %86 to i64
  %88 = getelementptr inbounds %union.StackValue, ptr %85, i64 %87
  store ptr %88, ptr %12, align 8
  %89 = load ptr, ptr %6, align 8
  %90 = getelementptr inbounds %struct.CallInfo, ptr %89, i32 0, i32 0
  %91 = load ptr, ptr %90, align 8
  %92 = load i32, ptr %10, align 4
  %93 = sext i32 %92 to i64
  %94 = sub i64 0, %93
  %95 = getelementptr inbounds %union.StackValue, ptr %91, i64 %94
  %96 = load i32, ptr %9, align 4
  %97 = sext i32 %96 to i64
  %98 = getelementptr inbounds %union.StackValue, ptr %95, i64 %97
  store ptr %98, ptr %13, align 8
  %99 = load ptr, ptr %12, align 8
  %100 = getelementptr inbounds %struct.TValue, ptr %99, i32 0, i32 0
  %101 = load ptr, ptr %13, align 8
  %102 = getelementptr inbounds %struct.TValue, ptr %101, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %100, ptr align 8 %102, i64 8, i1 false)
  %103 = load ptr, ptr %13, align 8
  %104 = getelementptr inbounds %struct.TValue, ptr %103, i32 0, i32 1
  %105 = load i8, ptr %104, align 8
  %106 = load ptr, ptr %12, align 8
  %107 = getelementptr inbounds %struct.TValue, ptr %106, i32 0, i32 1
  store i8 %105, ptr %107, align 8
  %108 = load ptr, ptr %5, align 8
  br label %109

109:                                              ; preds = %84
  %110 = load i32, ptr %9, align 4
  %111 = add nsw i32 %110, 1
  store i32 %111, ptr %9, align 4
  br label %74, !llvm.loop !9

112:                                              ; preds = %82
  br label %113

113:                                              ; preds = %123, %112
  %114 = load i32, ptr %9, align 4
  %115 = load i32, ptr %8, align 4
  %116 = icmp slt i32 %114, %115
  br i1 %116, label %117, label %126

117:                                              ; preds = %113
  %118 = load ptr, ptr %7, align 8
  %119 = load i32, ptr %9, align 4
  %120 = sext i32 %119 to i64
  %121 = getelementptr inbounds %union.StackValue, ptr %118, i64 %120
  %122 = getelementptr inbounds %struct.TValue, ptr %121, i32 0, i32 1
  store i8 0, ptr %122, align 8
  br label %123

123:                                              ; preds = %117
  %124 = load i32, ptr %9, align 4
  %125 = add nsw i32 %124, 1
  store i32 %125, ptr %9, align 4
  br label %113, !llvm.loop !10

126:                                              ; preds = %113
  ret void
}

declare hidden void @luaC_step(ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noreturn }

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
