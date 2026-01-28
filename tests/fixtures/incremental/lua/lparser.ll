; ModuleID = 'lparser.c'
source_filename = "lparser.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.anon.13 = type { i8, i8 }
%struct.FuncState = type { ptr, ptr, ptr, ptr, i32, i32, i32, i32, i32, i32, i32, i32, i16, i8, i8, i8, i8, i8 }
%struct.anon.9 = type { %union.Value, i8, i8, i8, i16, ptr }
%union.Value = type { ptr }
%struct.LexState = type { i32, i32, i32, %struct.Token, %struct.Token, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Token = type { i32, %union.SemInfo }
%union.SemInfo = type { double }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.2, i16, i16 }
%union.anon = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.TValue = type { %union.Value, i8 }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%struct.Dyndata = type { %struct.anon.8, %struct.Labellist, %struct.Labellist }
%struct.anon.8 = type { ptr, i32, i32 }
%struct.Labellist = type { ptr, i32, i32 }
%union.StackValue = type { %struct.TValue }
%struct.BlockCnt = type { ptr, i32, i32, i8, i8, i8, i8 }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }
%union.Vardesc = type { %struct.anon.9 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.expdesc = type { i32, %union.anon.10, i32, i32 }
%union.anon.10 = type { i64 }
%struct.LocVar = type { ptr, i32, i32 }
%struct.Labeldesc = type { ptr, i32, i32, i8, i8 }
%struct.LHS_assign = type { ptr, %struct.expdesc }
%struct.ConsControl = type { %struct.expdesc, ptr, i32, i32, i32 }
%struct.anon.12 = type { i8, i16 }
%struct.anon.11 = type { i16, i8 }

@.str = private unnamed_addr constant [9 x i8] c"upvalues\00", align 1
@.str.1 = private unnamed_addr constant [14 x i8] c"main function\00", align 1
@.str.2 = private unnamed_addr constant [20 x i8] c"function at line %d\00", align 1
@.str.3 = private unnamed_addr constant [32 x i8] c"too many %s (limit is %d) in %s\00", align 1
@.str.4 = private unnamed_addr constant [6 x i8] c"break\00", align 1
@priority = internal constant [21 x %struct.anon.13] [%struct.anon.13 { i8 10, i8 10 }, %struct.anon.13 { i8 10, i8 10 }, %struct.anon.13 { i8 11, i8 11 }, %struct.anon.13 { i8 11, i8 11 }, %struct.anon.13 { i8 14, i8 13 }, %struct.anon.13 { i8 11, i8 11 }, %struct.anon.13 { i8 11, i8 11 }, %struct.anon.13 { i8 6, i8 6 }, %struct.anon.13 { i8 4, i8 4 }, %struct.anon.13 { i8 5, i8 5 }, %struct.anon.13 { i8 7, i8 7 }, %struct.anon.13 { i8 7, i8 7 }, %struct.anon.13 { i8 9, i8 8 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 3, i8 3 }, %struct.anon.13 { i8 2, i8 2 }, %struct.anon.13 { i8 1, i8 1 }], align 16
@.str.5 = private unnamed_addr constant [43 x i8] c"cannot use '...' outside a vararg function\00", align 1
@.str.6 = private unnamed_addr constant [23 x i8] c"items in a constructor\00", align 1
@.str.7 = private unnamed_addr constant [5 x i8] c"self\00", align 1
@.str.8 = private unnamed_addr constant [10 x i8] c"functions\00", align 1
@.str.9 = private unnamed_addr constant [16 x i8] c"local variables\00", align 1
@.str.10 = private unnamed_addr constant [25 x i8] c"<name> or '...' expected\00", align 1
@.str.11 = private unnamed_addr constant [18 x i8] c"unexpected symbol\00", align 1
@.str.12 = private unnamed_addr constant [28 x i8] c"function arguments expected\00", align 1
@.str.13 = private unnamed_addr constant [13 x i8] c"labels/gotos\00", align 1
@.str.14 = private unnamed_addr constant [56 x i8] c"<goto %s> at line %d jumps into the scope of local '%s'\00", align 1
@.str.15 = private unnamed_addr constant [30 x i8] c"break outside loop at line %d\00", align 1
@.str.16 = private unnamed_addr constant [44 x i8] c"no visible label '%s' for <goto> at line %d\00", align 1
@.str.17 = private unnamed_addr constant [37 x i8] c"%s expected (to close %s at line %d)\00", align 1
@.str.18 = private unnamed_addr constant [12 x i8] c"%s expected\00", align 1
@.str.19 = private unnamed_addr constant [21 x i8] c"'=' or 'in' expected\00", align 1
@.str.20 = private unnamed_addr constant [12 x i8] c"(for state)\00", align 1
@forbody.forprep = internal constant [2 x i32] [i32 74, i32 75], align 4
@forbody.forloop = internal constant [2 x i32] [i32 73, i32 77], align 4
@.str.21 = private unnamed_addr constant [27 x i8] c"control structure too long\00", align 1
@.str.22 = private unnamed_addr constant [41 x i8] c"attempt to assign to const variable '%s'\00", align 1
@.str.23 = private unnamed_addr constant [46 x i8] c"multiple to-be-closed variables in local list\00", align 1
@.str.24 = private unnamed_addr constant [6 x i8] c"const\00", align 1
@.str.25 = private unnamed_addr constant [6 x i8] c"close\00", align 1
@.str.26 = private unnamed_addr constant [23 x i8] c"unknown attribute '%s'\00", align 1
@.str.27 = private unnamed_addr constant [38 x i8] c"label '%s' already defined on line %d\00", align 1
@.str.28 = private unnamed_addr constant [13 x i8] c"syntax error\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaY_nvarstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.FuncState, ptr %4, i32 0, i32 13
  %6 = load i8, ptr %5, align 2
  %7 = zext i8 %6 to i32
  %8 = call i32 @reglevel(ptr noundef %3, i32 noundef %7)
  ret i32 %8
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @reglevel(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  br label %7

7:                                                ; preds = %26, %2
  %8 = load i32, ptr %5, align 4
  %9 = add nsw i32 %8, -1
  store i32 %9, ptr %5, align 4
  %10 = icmp sgt i32 %8, 0
  br i1 %10, label %11, label %27

11:                                               ; preds = %7
  %12 = load ptr, ptr %4, align 8
  %13 = load i32, ptr %5, align 4
  %14 = call ptr @getlocalvardesc(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.anon.9, ptr %15, i32 0, i32 2
  %17 = load i8, ptr %16, align 1
  %18 = zext i8 %17 to i32
  %19 = icmp ne i32 %18, 3
  br i1 %19, label %20, label %26

20:                                               ; preds = %11
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.anon.9, ptr %21, i32 0, i32 3
  %23 = load i8, ptr %22, align 2
  %24 = zext i8 %23 to i32
  %25 = add nsw i32 %24, 1
  store i32 %25, ptr %3, align 4
  br label %28

26:                                               ; preds = %11
  br label %7, !llvm.loop !6

27:                                               ; preds = %7
  store i32 0, ptr %3, align 4
  br label %28

28:                                               ; preds = %27, %20
  %29 = load i32, ptr %3, align 4
  ret i32 %29
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaY_parser(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4, i32 noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca %struct.LexState, align 8
  %14 = alloca %struct.FuncState, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store ptr %4, ptr %11, align 8
  store i32 %5, ptr %12, align 4
  %20 = load ptr, ptr %7, align 8
  %21 = call ptr @luaF_newLclosure(ptr noundef %20, i32 noundef 1)
  store ptr %21, ptr %15, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 6
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %16, align 8
  %25 = load ptr, ptr %15, align 8
  store ptr %25, ptr %17, align 8
  %26 = load ptr, ptr %17, align 8
  %27 = load ptr, ptr %16, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  store ptr %26, ptr %28, align 8
  %29 = load ptr, ptr %16, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  store i8 70, ptr %30, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = load ptr, ptr %7, align 8
  call void @luaD_inctop(ptr noundef %32)
  %33 = load ptr, ptr %7, align 8
  %34 = call ptr @luaH_new(ptr noundef %33)
  %35 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 9
  store ptr %34, ptr %35, align 8
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  store ptr %38, ptr %18, align 8
  %39 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 9
  %40 = load ptr, ptr %39, align 8
  store ptr %40, ptr %19, align 8
  %41 = load ptr, ptr %19, align 8
  %42 = load ptr, ptr %18, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  store ptr %41, ptr %43, align 8
  %44 = load ptr, ptr %18, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 1
  store i8 69, ptr %45, align 8
  %46 = load ptr, ptr %7, align 8
  %47 = load ptr, ptr %7, align 8
  call void @luaD_inctop(ptr noundef %47)
  %48 = load ptr, ptr %7, align 8
  %49 = call ptr @luaF_newproto(ptr noundef %48)
  %50 = load ptr, ptr %15, align 8
  %51 = getelementptr inbounds %struct.LClosure, ptr %50, i32 0, i32 5
  store ptr %49, ptr %51, align 8
  %52 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  store ptr %49, ptr %52, align 8
  %53 = load ptr, ptr %15, align 8
  %54 = getelementptr inbounds %struct.LClosure, ptr %53, i32 0, i32 2
  %55 = load i8, ptr %54, align 1
  %56 = zext i8 %55 to i32
  %57 = and i32 %56, 32
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %74

59:                                               ; preds = %6
  %60 = load ptr, ptr %15, align 8
  %61 = getelementptr inbounds %struct.LClosure, ptr %60, i32 0, i32 5
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds %struct.Proto, ptr %62, i32 0, i32 2
  %64 = load i8, ptr %63, align 1
  %65 = zext i8 %64 to i32
  %66 = and i32 %65, 24
  %67 = icmp ne i32 %66, 0
  br i1 %67, label %68, label %74

68:                                               ; preds = %59
  %69 = load ptr, ptr %7, align 8
  %70 = load ptr, ptr %15, align 8
  %71 = load ptr, ptr %15, align 8
  %72 = getelementptr inbounds %struct.LClosure, ptr %71, i32 0, i32 5
  %73 = load ptr, ptr %72, align 8
  call void @luaC_barrier_(ptr noundef %69, ptr noundef %70, ptr noundef %73)
  br label %75

74:                                               ; preds = %59, %6
  br label %75

75:                                               ; preds = %74, %68
  %76 = load ptr, ptr %7, align 8
  %77 = load ptr, ptr %11, align 8
  %78 = call ptr @luaS_new(ptr noundef %76, ptr noundef %77)
  %79 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds %struct.Proto, ptr %80, i32 0, i32 22
  store ptr %78, ptr %81, align 8
  %82 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %83 = load ptr, ptr %82, align 8
  %84 = getelementptr inbounds %struct.Proto, ptr %83, i32 0, i32 2
  %85 = load i8, ptr %84, align 1
  %86 = zext i8 %85 to i32
  %87 = and i32 %86, 32
  %88 = icmp ne i32 %87, 0
  br i1 %88, label %89, label %107

89:                                               ; preds = %75
  %90 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %struct.Proto, ptr %91, i32 0, i32 22
  %93 = load ptr, ptr %92, align 8
  %94 = getelementptr inbounds %struct.TString, ptr %93, i32 0, i32 2
  %95 = load i8, ptr %94, align 1
  %96 = zext i8 %95 to i32
  %97 = and i32 %96, 24
  %98 = icmp ne i32 %97, 0
  br i1 %98, label %99, label %107

99:                                               ; preds = %89
  %100 = load ptr, ptr %7, align 8
  %101 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %102 = load ptr, ptr %101, align 8
  %103 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %104 = load ptr, ptr %103, align 8
  %105 = getelementptr inbounds %struct.Proto, ptr %104, i32 0, i32 22
  %106 = load ptr, ptr %105, align 8
  call void @luaC_barrier_(ptr noundef %100, ptr noundef %102, ptr noundef %106)
  br label %108

107:                                              ; preds = %89, %75
  br label %108

108:                                              ; preds = %107, %99
  %109 = load ptr, ptr %9, align 8
  %110 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 8
  store ptr %109, ptr %110, align 8
  %111 = load ptr, ptr %10, align 8
  %112 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 10
  store ptr %111, ptr %112, align 8
  %113 = load ptr, ptr %10, align 8
  %114 = getelementptr inbounds %struct.Dyndata, ptr %113, i32 0, i32 2
  %115 = getelementptr inbounds %struct.Labellist, ptr %114, i32 0, i32 1
  store i32 0, ptr %115, align 8
  %116 = load ptr, ptr %10, align 8
  %117 = getelementptr inbounds %struct.Dyndata, ptr %116, i32 0, i32 1
  %118 = getelementptr inbounds %struct.Labellist, ptr %117, i32 0, i32 1
  store i32 0, ptr %118, align 8
  %119 = load ptr, ptr %10, align 8
  %120 = getelementptr inbounds %struct.Dyndata, ptr %119, i32 0, i32 0
  %121 = getelementptr inbounds %struct.anon.8, ptr %120, i32 0, i32 1
  store i32 0, ptr %121, align 8
  %122 = load ptr, ptr %7, align 8
  %123 = load ptr, ptr %8, align 8
  %124 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %125 = load ptr, ptr %124, align 8
  %126 = getelementptr inbounds %struct.Proto, ptr %125, i32 0, i32 22
  %127 = load ptr, ptr %126, align 8
  %128 = load i32, ptr %12, align 4
  call void @luaX_setinput(ptr noundef %122, ptr noundef %13, ptr noundef %123, ptr noundef %127, i32 noundef %128)
  call void @mainfunc(ptr noundef %13, ptr noundef %14)
  %129 = load ptr, ptr %7, align 8
  %130 = getelementptr inbounds %struct.lua_State, ptr %129, i32 0, i32 6
  %131 = load ptr, ptr %130, align 8
  %132 = getelementptr inbounds %union.StackValue, ptr %131, i32 -1
  store ptr %132, ptr %130, align 8
  %133 = load ptr, ptr %15, align 8
  ret ptr %133
}

declare hidden ptr @luaF_newLclosure(ptr noundef, i32 noundef) #1

declare hidden void @luaD_inctop(ptr noundef) #1

declare hidden ptr @luaH_new(ptr noundef) #1

declare hidden ptr @luaF_newproto(ptr noundef) #1

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #1

declare hidden ptr @luaS_new(ptr noundef, ptr noundef) #1

declare hidden void @luaX_setinput(ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @mainfunc(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.BlockCnt, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %4, align 8
  call void @open_func(ptr noundef %7, ptr noundef %8, ptr noundef %5)
  %9 = load ptr, ptr %4, align 8
  call void @setvararg(ptr noundef %9, i32 noundef 0)
  %10 = load ptr, ptr %4, align 8
  %11 = call ptr @allocupvalue(ptr noundef %10)
  store ptr %11, ptr %6, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.Upvaldesc, ptr %12, i32 0, i32 1
  store i8 1, ptr %13, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.Upvaldesc, ptr %14, i32 0, i32 2
  store i8 0, ptr %15, align 1
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.Upvaldesc, ptr %16, i32 0, i32 3
  store i8 0, ptr %17, align 2
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 12
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.Upvaldesc, ptr %21, i32 0, i32 0
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.FuncState, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 2
  %27 = load i8, ptr %26, align 1
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 32
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %50

31:                                               ; preds = %2
  %32 = load ptr, ptr %6, align 8
  %33 = getelementptr inbounds %struct.Upvaldesc, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %struct.TString, ptr %34, i32 0, i32 2
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 24
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %50

40:                                               ; preds = %31
  %41 = load ptr, ptr %3, align 8
  %42 = getelementptr inbounds %struct.LexState, ptr %41, i32 0, i32 6
  %43 = load ptr, ptr %42, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.FuncState, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.Upvaldesc, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  call void @luaC_barrier_(ptr noundef %43, ptr noundef %46, ptr noundef %49)
  br label %51

50:                                               ; preds = %31, %2
  br label %51

51:                                               ; preds = %50, %40
  %52 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %52)
  %53 = load ptr, ptr %3, align 8
  call void @statlist(ptr noundef %53)
  %54 = load ptr, ptr %3, align 8
  call void @check(ptr noundef %54, i32 noundef 288)
  %55 = load ptr, ptr %3, align 8
  call void @close_func(ptr noundef %55)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getlocalvardesc(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.FuncState, ptr %5, i32 0, i32 2
  %7 = load ptr, ptr %6, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 10
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Dyndata, ptr %9, i32 0, i32 0
  %11 = getelementptr inbounds %struct.anon.8, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 10
  %15 = load i32, ptr %14, align 8
  %16 = load i32, ptr %4, align 4
  %17 = add nsw i32 %15, %16
  %18 = sext i32 %17 to i64
  %19 = getelementptr inbounds %union.Vardesc, ptr %12, i64 %18
  ret ptr %19
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @open_func(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 1
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 2
  store ptr %16, ptr %18, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 5
  store ptr %19, ptr %21, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 4
  store i32 0, ptr %23, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 13
  %26 = load i32, ptr %25, align 4
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.FuncState, ptr %27, i32 0, i32 6
  store i32 %26, ptr %28, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.FuncState, ptr %29, i32 0, i32 16
  store i8 0, ptr %30, align 1
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.FuncState, ptr %31, i32 0, i32 5
  store i32 0, ptr %32, align 4
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.FuncState, ptr %33, i32 0, i32 15
  store i8 0, ptr %34, align 4
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.FuncState, ptr %35, i32 0, i32 7
  store i32 0, ptr %36, align 4
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.FuncState, ptr %37, i32 0, i32 9
  store i32 0, ptr %38, align 4
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.FuncState, ptr %39, i32 0, i32 8
  store i32 0, ptr %40, align 8
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.FuncState, ptr %41, i32 0, i32 14
  store i8 0, ptr %42, align 1
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.FuncState, ptr %43, i32 0, i32 12
  store i16 0, ptr %44, align 8
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.FuncState, ptr %45, i32 0, i32 13
  store i8 0, ptr %46, align 2
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %struct.FuncState, ptr %47, i32 0, i32 17
  store i8 0, ptr %48, align 2
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.LexState, ptr %49, i32 0, i32 10
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.Dyndata, ptr %51, i32 0, i32 0
  %53 = getelementptr inbounds %struct.anon.8, ptr %52, i32 0, i32 1
  %54 = load i32, ptr %53, align 8
  %55 = load ptr, ptr %5, align 8
  %56 = getelementptr inbounds %struct.FuncState, ptr %55, i32 0, i32 10
  store i32 %54, ptr %56, align 8
  %57 = load ptr, ptr %4, align 8
  %58 = getelementptr inbounds %struct.LexState, ptr %57, i32 0, i32 10
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %struct.Dyndata, ptr %59, i32 0, i32 2
  %61 = getelementptr inbounds %struct.Labellist, ptr %60, i32 0, i32 1
  %62 = load i32, ptr %61, align 8
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.FuncState, ptr %63, i32 0, i32 11
  store i32 %62, ptr %64, align 4
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.FuncState, ptr %65, i32 0, i32 3
  store ptr null, ptr %66, align 8
  %67 = load ptr, ptr %4, align 8
  %68 = getelementptr inbounds %struct.LexState, ptr %67, i32 0, i32 11
  %69 = load ptr, ptr %68, align 8
  %70 = load ptr, ptr %7, align 8
  %71 = getelementptr inbounds %struct.Proto, ptr %70, i32 0, i32 22
  store ptr %69, ptr %71, align 8
  %72 = load ptr, ptr %7, align 8
  %73 = getelementptr inbounds %struct.Proto, ptr %72, i32 0, i32 2
  %74 = load i8, ptr %73, align 1
  %75 = zext i8 %74 to i32
  %76 = and i32 %75, 32
  %77 = icmp ne i32 %76, 0
  br i1 %77, label %78, label %95

78:                                               ; preds = %3
  %79 = load ptr, ptr %7, align 8
  %80 = getelementptr inbounds %struct.Proto, ptr %79, i32 0, i32 22
  %81 = load ptr, ptr %80, align 8
  %82 = getelementptr inbounds %struct.TString, ptr %81, i32 0, i32 2
  %83 = load i8, ptr %82, align 1
  %84 = zext i8 %83 to i32
  %85 = and i32 %84, 24
  %86 = icmp ne i32 %85, 0
  br i1 %86, label %87, label %95

87:                                               ; preds = %78
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds %struct.LexState, ptr %88, i32 0, i32 6
  %90 = load ptr, ptr %89, align 8
  %91 = load ptr, ptr %7, align 8
  %92 = load ptr, ptr %7, align 8
  %93 = getelementptr inbounds %struct.Proto, ptr %92, i32 0, i32 22
  %94 = load ptr, ptr %93, align 8
  call void @luaC_barrier_(ptr noundef %90, ptr noundef %91, ptr noundef %94)
  br label %96

95:                                               ; preds = %78, %3
  br label %96

96:                                               ; preds = %95, %87
  %97 = load ptr, ptr %7, align 8
  %98 = getelementptr inbounds %struct.Proto, ptr %97, i32 0, i32 5
  store i8 2, ptr %98, align 4
  %99 = load ptr, ptr %5, align 8
  %100 = load ptr, ptr %6, align 8
  call void @enterblock(ptr noundef %99, ptr noundef %100, i8 noundef zeroext 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setvararg(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.FuncState, ptr %5, i32 0, i32 0
  %7 = load ptr, ptr %6, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 4
  store i8 1, ptr %8, align 1
  %9 = load ptr, ptr %3, align 8
  %10 = load i32, ptr %4, align 4
  %11 = call i32 @luaK_codeABCk(ptr noundef %9, i32 noundef 81, i32 noundef %10, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @allocupvalue(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.FuncState, ptr %5, i32 0, i32 0
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.Proto, ptr %8, i32 0, i32 6
  %10 = load i32, ptr %9, align 8
  store i32 %10, ptr %4, align 4
  %11 = load ptr, ptr %2, align 8
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 14
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = add nsw i32 %15, 1
  call void @checklimit(ptr noundef %11, i32 noundef %16, i32 noundef 255, ptr noundef @.str)
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 2
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 18
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %2, align 8
  %26 = getelementptr inbounds %struct.FuncState, ptr %25, i32 0, i32 14
  %27 = load i8, ptr %26, align 1
  %28 = zext i8 %27 to i32
  %29 = load ptr, ptr %3, align 8
  %30 = getelementptr inbounds %struct.Proto, ptr %29, i32 0, i32 6
  %31 = call ptr @luaM_growaux_(ptr noundef %21, ptr noundef %24, i32 noundef %28, ptr noundef %30, i32 noundef 16, i32 noundef 255, ptr noundef @.str)
  %32 = load ptr, ptr %3, align 8
  %33 = getelementptr inbounds %struct.Proto, ptr %32, i32 0, i32 18
  store ptr %31, ptr %33, align 8
  br label %34

34:                                               ; preds = %40, %1
  %35 = load i32, ptr %4, align 4
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 6
  %38 = load i32, ptr %37, align 8
  %39 = icmp slt i32 %35, %38
  br i1 %39, label %40, label %49

40:                                               ; preds = %34
  %41 = load ptr, ptr %3, align 8
  %42 = getelementptr inbounds %struct.Proto, ptr %41, i32 0, i32 18
  %43 = load ptr, ptr %42, align 8
  %44 = load i32, ptr %4, align 4
  %45 = add nsw i32 %44, 1
  store i32 %45, ptr %4, align 4
  %46 = sext i32 %44 to i64
  %47 = getelementptr inbounds %struct.Upvaldesc, ptr %43, i64 %46
  %48 = getelementptr inbounds %struct.Upvaldesc, ptr %47, i32 0, i32 0
  store ptr null, ptr %48, align 8
  br label %34, !llvm.loop !8

49:                                               ; preds = %34
  %50 = load ptr, ptr %3, align 8
  %51 = getelementptr inbounds %struct.Proto, ptr %50, i32 0, i32 18
  %52 = load ptr, ptr %51, align 8
  %53 = load ptr, ptr %2, align 8
  %54 = getelementptr inbounds %struct.FuncState, ptr %53, i32 0, i32 14
  %55 = load i8, ptr %54, align 1
  %56 = add i8 %55, 1
  store i8 %56, ptr %54, align 1
  %57 = zext i8 %55 to i64
  %58 = getelementptr inbounds %struct.Upvaldesc, ptr %52, i64 %57
  ret ptr %58
}

declare hidden void @luaX_next(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @statlist(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  br label %3

3:                                                ; preds = %16, %1
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @block_follow(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  %7 = xor i1 %6, true
  br i1 %7, label %8, label %18

8:                                                ; preds = %3
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 3
  %11 = getelementptr inbounds %struct.Token, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 8
  %13 = icmp eq i32 %12, 273
  br i1 %13, label %14, label %16

14:                                               ; preds = %8
  %15 = load ptr, ptr %2, align 8
  call void @statement(ptr noundef %15)
  br label %18

16:                                               ; preds = %8
  %17 = load ptr, ptr %2, align 8
  call void @statement(ptr noundef %17)
  br label %3, !llvm.loop !9

18:                                               ; preds = %14, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @check(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 3
  %7 = getelementptr inbounds %struct.Token, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  %9 = load i32, ptr %4, align 4
  %10 = icmp ne i32 %8, %9
  br i1 %10, label %11, label %14

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  %13 = load i32, ptr %4, align 4
  call void @error_expected(ptr noundef %12, i32 noundef %13) #6
  unreachable

14:                                               ; preds = %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @close_func(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 5
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %4, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %5, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = call i32 @luaY_nvarstack(ptr noundef %16)
  call void @luaK_ret(ptr noundef %15, i32 noundef %17, i32 noundef 0)
  %18 = load ptr, ptr %4, align 8
  call void @leaveblock(ptr noundef %18)
  %19 = load ptr, ptr %4, align 8
  call void @luaK_finish(ptr noundef %19)
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 16
  %23 = load ptr, ptr %22, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.FuncState, ptr %26, i32 0, i32 4
  %28 = load i32, ptr %27, align 8
  %29 = call ptr @luaM_shrinkvector_(ptr noundef %20, ptr noundef %23, ptr noundef %25, i32 noundef %28, i32 noundef 4)
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.Proto, ptr %30, i32 0, i32 16
  store ptr %29, ptr %31, align 8
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.Proto, ptr %33, i32 0, i32 19
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 9
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.FuncState, ptr %38, i32 0, i32 4
  %40 = load i32, ptr %39, align 8
  %41 = call ptr @luaM_shrinkvector_(ptr noundef %32, ptr noundef %35, ptr noundef %37, i32 noundef %40, i32 noundef 1)
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.Proto, ptr %42, i32 0, i32 19
  store ptr %41, ptr %43, align 8
  %44 = load ptr, ptr %3, align 8
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.Proto, ptr %45, i32 0, i32 20
  %47 = load ptr, ptr %46, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.Proto, ptr %48, i32 0, i32 12
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.FuncState, ptr %50, i32 0, i32 9
  %52 = load i32, ptr %51, align 4
  %53 = call ptr @luaM_shrinkvector_(ptr noundef %44, ptr noundef %47, ptr noundef %49, i32 noundef %52, i32 noundef 8)
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 20
  store ptr %53, ptr %55, align 8
  %56 = load ptr, ptr %3, align 8
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.Proto, ptr %57, i32 0, i32 15
  %59 = load ptr, ptr %58, align 8
  %60 = load ptr, ptr %5, align 8
  %61 = getelementptr inbounds %struct.Proto, ptr %60, i32 0, i32 7
  %62 = load ptr, ptr %4, align 8
  %63 = getelementptr inbounds %struct.FuncState, ptr %62, i32 0, i32 7
  %64 = load i32, ptr %63, align 4
  %65 = call ptr @luaM_shrinkvector_(ptr noundef %56, ptr noundef %59, ptr noundef %61, i32 noundef %64, i32 noundef 16)
  %66 = load ptr, ptr %5, align 8
  %67 = getelementptr inbounds %struct.Proto, ptr %66, i32 0, i32 15
  store ptr %65, ptr %67, align 8
  %68 = load ptr, ptr %3, align 8
  %69 = load ptr, ptr %5, align 8
  %70 = getelementptr inbounds %struct.Proto, ptr %69, i32 0, i32 17
  %71 = load ptr, ptr %70, align 8
  %72 = load ptr, ptr %5, align 8
  %73 = getelementptr inbounds %struct.Proto, ptr %72, i32 0, i32 10
  %74 = load ptr, ptr %4, align 8
  %75 = getelementptr inbounds %struct.FuncState, ptr %74, i32 0, i32 8
  %76 = load i32, ptr %75, align 8
  %77 = call ptr @luaM_shrinkvector_(ptr noundef %68, ptr noundef %71, ptr noundef %73, i32 noundef %76, i32 noundef 8)
  %78 = load ptr, ptr %5, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 17
  store ptr %77, ptr %79, align 8
  %80 = load ptr, ptr %3, align 8
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds %struct.Proto, ptr %81, i32 0, i32 21
  %83 = load ptr, ptr %82, align 8
  %84 = load ptr, ptr %5, align 8
  %85 = getelementptr inbounds %struct.Proto, ptr %84, i32 0, i32 11
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.FuncState, ptr %86, i32 0, i32 12
  %88 = load i16, ptr %87, align 8
  %89 = sext i16 %88 to i32
  %90 = call ptr @luaM_shrinkvector_(ptr noundef %80, ptr noundef %83, ptr noundef %85, i32 noundef %89, i32 noundef 16)
  %91 = load ptr, ptr %5, align 8
  %92 = getelementptr inbounds %struct.Proto, ptr %91, i32 0, i32 21
  store ptr %90, ptr %92, align 8
  %93 = load ptr, ptr %3, align 8
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %struct.Proto, ptr %94, i32 0, i32 18
  %96 = load ptr, ptr %95, align 8
  %97 = load ptr, ptr %5, align 8
  %98 = getelementptr inbounds %struct.Proto, ptr %97, i32 0, i32 6
  %99 = load ptr, ptr %4, align 8
  %100 = getelementptr inbounds %struct.FuncState, ptr %99, i32 0, i32 14
  %101 = load i8, ptr %100, align 1
  %102 = zext i8 %101 to i32
  %103 = call ptr @luaM_shrinkvector_(ptr noundef %93, ptr noundef %96, ptr noundef %98, i32 noundef %102, i32 noundef 16)
  %104 = load ptr, ptr %5, align 8
  %105 = getelementptr inbounds %struct.Proto, ptr %104, i32 0, i32 18
  store ptr %103, ptr %105, align 8
  %106 = load ptr, ptr %4, align 8
  %107 = getelementptr inbounds %struct.FuncState, ptr %106, i32 0, i32 1
  %108 = load ptr, ptr %107, align 8
  %109 = load ptr, ptr %2, align 8
  %110 = getelementptr inbounds %struct.LexState, ptr %109, i32 0, i32 5
  store ptr %108, ptr %110, align 8
  %111 = load ptr, ptr %3, align 8
  %112 = getelementptr inbounds %struct.lua_State, ptr %111, i32 0, i32 7
  %113 = load ptr, ptr %112, align 8
  %114 = getelementptr inbounds %struct.global_State, ptr %113, i32 0, i32 3
  %115 = load i64, ptr %114, align 8
  %116 = icmp sgt i64 %115, 0
  br i1 %116, label %117, label %119

117:                                              ; preds = %1
  %118 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %118)
  br label %119

119:                                              ; preds = %117, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @enterblock(ptr noundef %0, ptr noundef %1, i8 noundef zeroext %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i8, align 1
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i8 %2, ptr %6, align 1
  %7 = load i8, ptr %6, align 1
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.BlockCnt, ptr %8, i32 0, i32 5
  store i8 %7, ptr %9, align 2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 13
  %12 = load i8, ptr %11, align 2
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.BlockCnt, ptr %13, i32 0, i32 3
  store i8 %12, ptr %14, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.FuncState, ptr %15, i32 0, i32 2
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.LexState, ptr %17, i32 0, i32 10
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.Dyndata, ptr %19, i32 0, i32 2
  %21 = getelementptr inbounds %struct.Labellist, ptr %20, i32 0, i32 1
  %22 = load i32, ptr %21, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.BlockCnt, ptr %23, i32 0, i32 1
  store i32 %22, ptr %24, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.FuncState, ptr %25, i32 0, i32 2
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.LexState, ptr %27, i32 0, i32 10
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.Dyndata, ptr %29, i32 0, i32 1
  %31 = getelementptr inbounds %struct.Labellist, ptr %30, i32 0, i32 1
  %32 = load i32, ptr %31, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.BlockCnt, ptr %33, i32 0, i32 2
  store i32 %32, ptr %34, align 4
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.BlockCnt, ptr %35, i32 0, i32 4
  store i8 0, ptr %36, align 1
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.FuncState, ptr %37, i32 0, i32 3
  %39 = load ptr, ptr %38, align 8
  %40 = icmp ne ptr %39, null
  br i1 %40, label %41, label %49

41:                                               ; preds = %3
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.FuncState, ptr %42, i32 0, i32 3
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.BlockCnt, ptr %44, i32 0, i32 6
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = icmp ne i32 %47, 0
  br label %49

49:                                               ; preds = %41, %3
  %50 = phi i1 [ false, %3 ], [ %48, %41 ]
  %51 = zext i1 %50 to i32
  %52 = trunc i32 %51 to i8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.BlockCnt, ptr %53, i32 0, i32 6
  store i8 %52, ptr %54, align 1
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.FuncState, ptr %55, i32 0, i32 3
  %57 = load ptr, ptr %56, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.BlockCnt, ptr %58, i32 0, i32 0
  store ptr %57, ptr %59, align 8
  %60 = load ptr, ptr %5, align 8
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.FuncState, ptr %61, i32 0, i32 3
  store ptr %60, ptr %62, align 8
  ret void
}

declare hidden i32 @luaK_codeABCk(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checklimit(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %9 = load i32, ptr %6, align 4
  %10 = load i32, ptr %7, align 4
  %11 = icmp sgt i32 %9, %10
  br i1 %11, label %12, label %16

12:                                               ; preds = %4
  %13 = load ptr, ptr %5, align 8
  %14 = load i32, ptr %7, align 4
  %15 = load ptr, ptr %8, align 8
  call void @errorlimit(ptr noundef %13, i32 noundef %14, ptr noundef %15) #6
  unreachable

16:                                               ; preds = %4
  ret void
}

declare hidden ptr @luaM_growaux_(ptr noundef, ptr noundef, i32 noundef, ptr noundef, i32 noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @errorlimit(ptr noundef %0, i32 noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.FuncState, ptr %11, i32 0, i32 2
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.FuncState, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 13
  %20 = load i32, ptr %19, align 4
  store i32 %20, ptr %9, align 4
  %21 = load i32, ptr %9, align 4
  %22 = icmp eq i32 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %3
  br label %28

24:                                               ; preds = %3
  %25 = load ptr, ptr %7, align 8
  %26 = load i32, ptr %9, align 4
  %27 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %25, ptr noundef @.str.2, i32 noundef %26)
  br label %28

28:                                               ; preds = %24, %23
  %29 = phi ptr [ @.str.1, %23 ], [ %27, %24 ]
  store ptr %29, ptr %10, align 8
  %30 = load ptr, ptr %7, align 8
  %31 = load ptr, ptr %6, align 8
  %32 = load i32, ptr %5, align 4
  %33 = load ptr, ptr %10, align 8
  %34 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %30, ptr noundef @.str.3, ptr noundef %31, i32 noundef %32, ptr noundef %33)
  store ptr %34, ptr %8, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.FuncState, ptr %35, i32 0, i32 2
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %8, align 8
  call void @luaX_syntaxerror(ptr noundef %37, ptr noundef %38) #6
  unreachable
}

declare hidden ptr @luaO_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noreturn
declare hidden void @luaX_syntaxerror(ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @block_follow(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 3
  %8 = getelementptr inbounds %struct.Token, ptr %7, i32 0, i32 0
  %9 = load i32, ptr %8, align 8
  switch i32 %9, label %13 [
    i32 259, label %10
    i32 260, label %10
    i32 261, label %10
    i32 288, label %10
    i32 276, label %11
  ]

10:                                               ; preds = %2, %2, %2, %2
  store i32 1, ptr %3, align 4
  br label %14

11:                                               ; preds = %2
  %12 = load i32, ptr %5, align 4
  store i32 %12, ptr %3, align 4
  br label %14

13:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %14

14:                                               ; preds = %13, %11, %10
  %15 = load i32, ptr %3, align 4
  ret i32 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @statement(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LexState, ptr %4, i32 0, i32 1
  %6 = load i32, ptr %5, align 4
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  call void @luaE_incCstack(ptr noundef %9)
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 3
  %12 = getelementptr inbounds %struct.Token, ptr %11, i32 0, i32 0
  %13 = load i32, ptr %12, align 8
  switch i32 %13, label %60 [
    i32 59, label %14
    i32 266, label %16
    i32 277, label %19
    i32 258, label %22
    i32 263, label %27
    i32 272, label %30
    i32 264, label %33
    i32 268, label %36
    i32 287, label %46
    i32 273, label %52
    i32 257, label %55
    i32 265, label %57
  ]

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %15)
  br label %62

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = load i32, ptr %3, align 4
  call void @ifstat(ptr noundef %17, i32 noundef %18)
  br label %62

19:                                               ; preds = %1
  %20 = load ptr, ptr %2, align 8
  %21 = load i32, ptr %3, align 4
  call void @whilestat(ptr noundef %20, i32 noundef %21)
  br label %62

22:                                               ; preds = %1
  %23 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %23)
  %24 = load ptr, ptr %2, align 8
  call void @block(ptr noundef %24)
  %25 = load ptr, ptr %2, align 8
  %26 = load i32, ptr %3, align 4
  call void @check_match(ptr noundef %25, i32 noundef 261, i32 noundef 258, i32 noundef %26)
  br label %62

27:                                               ; preds = %1
  %28 = load ptr, ptr %2, align 8
  %29 = load i32, ptr %3, align 4
  call void @forstat(ptr noundef %28, i32 noundef %29)
  br label %62

30:                                               ; preds = %1
  %31 = load ptr, ptr %2, align 8
  %32 = load i32, ptr %3, align 4
  call void @repeatstat(ptr noundef %31, i32 noundef %32)
  br label %62

33:                                               ; preds = %1
  %34 = load ptr, ptr %2, align 8
  %35 = load i32, ptr %3, align 4
  call void @funcstat(ptr noundef %34, i32 noundef %35)
  br label %62

36:                                               ; preds = %1
  %37 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %37)
  %38 = load ptr, ptr %2, align 8
  %39 = call i32 @testnext(ptr noundef %38, i32 noundef 264)
  %40 = icmp ne i32 %39, 0
  br i1 %40, label %41, label %43

41:                                               ; preds = %36
  %42 = load ptr, ptr %2, align 8
  call void @localfunc(ptr noundef %42)
  br label %45

43:                                               ; preds = %36
  %44 = load ptr, ptr %2, align 8
  call void @localstat(ptr noundef %44)
  br label %45

45:                                               ; preds = %43, %41
  br label %62

46:                                               ; preds = %1
  %47 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %47)
  %48 = load ptr, ptr %2, align 8
  %49 = load ptr, ptr %2, align 8
  %50 = call ptr @str_checkname(ptr noundef %49)
  %51 = load i32, ptr %3, align 4
  call void @labelstat(ptr noundef %48, ptr noundef %50, i32 noundef %51)
  br label %62

52:                                               ; preds = %1
  %53 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %53)
  %54 = load ptr, ptr %2, align 8
  call void @retstat(ptr noundef %54)
  br label %62

55:                                               ; preds = %1
  %56 = load ptr, ptr %2, align 8
  call void @breakstat(ptr noundef %56)
  br label %62

57:                                               ; preds = %1
  %58 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %58)
  %59 = load ptr, ptr %2, align 8
  call void @gotostat(ptr noundef %59)
  br label %62

60:                                               ; preds = %1
  %61 = load ptr, ptr %2, align 8
  call void @exprstat(ptr noundef %61)
  br label %62

62:                                               ; preds = %60, %57, %55, %52, %46, %45, %33, %30, %27, %22, %19, %16, %14
  %63 = load ptr, ptr %2, align 8
  %64 = getelementptr inbounds %struct.LexState, ptr %63, i32 0, i32 5
  %65 = load ptr, ptr %64, align 8
  %66 = call i32 @luaY_nvarstack(ptr noundef %65)
  %67 = trunc i32 %66 to i8
  %68 = load ptr, ptr %2, align 8
  %69 = getelementptr inbounds %struct.LexState, ptr %68, i32 0, i32 5
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.FuncState, ptr %70, i32 0, i32 15
  store i8 %67, ptr %71, align 4
  %72 = load ptr, ptr %2, align 8
  %73 = getelementptr inbounds %struct.LexState, ptr %72, i32 0, i32 6
  %74 = load ptr, ptr %73, align 8
  %75 = getelementptr inbounds %struct.lua_State, ptr %74, i32 0, i32 19
  %76 = load i32, ptr %75, align 8
  %77 = add i32 %76, -1
  store i32 %77, ptr %75, align 8
  ret void
}

declare hidden void @luaE_incCstack(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @ifstat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  store i32 -1, ptr %6, align 4
  %10 = load ptr, ptr %3, align 8
  call void @test_then_block(ptr noundef %10, ptr noundef %6)
  br label %11

11:                                               ; preds = %17, %2
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 3
  %14 = getelementptr inbounds %struct.Token, ptr %13, i32 0, i32 0
  %15 = load i32, ptr %14, align 8
  %16 = icmp eq i32 %15, 260
  br i1 %16, label %17, label %19

17:                                               ; preds = %11
  %18 = load ptr, ptr %3, align 8
  call void @test_then_block(ptr noundef %18, ptr noundef %6)
  br label %11, !llvm.loop !10

19:                                               ; preds = %11
  %20 = load ptr, ptr %3, align 8
  %21 = call i32 @testnext(ptr noundef %20, i32 noundef 259)
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %25

23:                                               ; preds = %19
  %24 = load ptr, ptr %3, align 8
  call void @block(ptr noundef %24)
  br label %25

25:                                               ; preds = %23, %19
  %26 = load ptr, ptr %3, align 8
  %27 = load i32, ptr %4, align 4
  call void @check_match(ptr noundef %26, i32 noundef 261, i32 noundef 266, i32 noundef %27)
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %6, align 4
  call void @luaK_patchtohere(ptr noundef %28, i32 noundef %29)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @whilestat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca %struct.BlockCnt, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 5
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %12)
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @luaK_getlabel(ptr noundef %13)
  store i32 %14, ptr %6, align 4
  %15 = load ptr, ptr %3, align 8
  %16 = call i32 @cond(ptr noundef %15)
  store i32 %16, ptr %7, align 4
  %17 = load ptr, ptr %5, align 8
  call void @enterblock(ptr noundef %17, ptr noundef %8, i8 noundef zeroext 1)
  %18 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %18, i32 noundef 258)
  %19 = load ptr, ptr %3, align 8
  call void @block(ptr noundef %19)
  %20 = load ptr, ptr %5, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = call i32 @luaK_jump(ptr noundef %21)
  %23 = load i32, ptr %6, align 4
  call void @luaK_patchlist(ptr noundef %20, i32 noundef %22, i32 noundef %23)
  %24 = load ptr, ptr %3, align 8
  %25 = load i32, ptr %4, align 4
  call void @check_match(ptr noundef %24, i32 noundef 261, i32 noundef 277, i32 noundef %25)
  %26 = load ptr, ptr %5, align 8
  call void @leaveblock(ptr noundef %26)
  %27 = load ptr, ptr %5, align 8
  %28 = load i32, ptr %7, align 4
  call void @luaK_patchtohere(ptr noundef %27, i32 noundef %28)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @block(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %struct.BlockCnt, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 5
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  call void @enterblock(ptr noundef %8, ptr noundef %4, i8 noundef zeroext 0)
  %9 = load ptr, ptr %2, align 8
  call void @statlist(ptr noundef %9)
  %10 = load ptr, ptr %3, align 8
  call void @leaveblock(ptr noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @check_match(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call i32 @testnext(ptr noundef %9, i32 noundef %10)
  %12 = icmp ne i32 %11, 0
  %13 = xor i1 %12, true
  %14 = zext i1 %13 to i32
  %15 = icmp ne i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = sext i32 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %19, label %41

19:                                               ; preds = %4
  %20 = load i32, ptr %8, align 4
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.LexState, ptr %21, i32 0, i32 1
  %23 = load i32, ptr %22, align 4
  %24 = icmp eq i32 %20, %23
  br i1 %24, label %25, label %28

25:                                               ; preds = %19
  %26 = load ptr, ptr %5, align 8
  %27 = load i32, ptr %6, align 4
  call void @error_expected(ptr noundef %26, i32 noundef %27) #6
  unreachable

28:                                               ; preds = %19
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = load i32, ptr %6, align 4
  %35 = call ptr @luaX_token2str(ptr noundef %33, i32 noundef %34)
  %36 = load ptr, ptr %5, align 8
  %37 = load i32, ptr %7, align 4
  %38 = call ptr @luaX_token2str(ptr noundef %36, i32 noundef %37)
  %39 = load i32, ptr %8, align 4
  %40 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %32, ptr noundef @.str.17, ptr noundef %35, ptr noundef %38, i32 noundef %39)
  call void @luaX_syntaxerror(ptr noundef %29, ptr noundef %40) #6
  unreachable

41:                                               ; preds = %4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @forstat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.BlockCnt, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 5
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  call void @enterblock(ptr noundef %11, ptr noundef %7, i8 noundef zeroext 1)
  %12 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %12)
  %13 = load ptr, ptr %3, align 8
  %14 = call ptr @str_checkname(ptr noundef %13)
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 3
  %17 = getelementptr inbounds %struct.Token, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  switch i32 %18, label %26 [
    i32 61, label %19
    i32 44, label %23
    i32 267, label %23
  ]

19:                                               ; preds = %2
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = load i32, ptr %4, align 4
  call void @fornum(ptr noundef %20, ptr noundef %21, i32 noundef %22)
  br label %28

23:                                               ; preds = %2, %2
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %6, align 8
  call void @forlist(ptr noundef %24, ptr noundef %25)
  br label %28

26:                                               ; preds = %2
  %27 = load ptr, ptr %3, align 8
  call void @luaX_syntaxerror(ptr noundef %27, ptr noundef @.str.19) #6
  unreachable

28:                                               ; preds = %23, %19
  %29 = load ptr, ptr %3, align 8
  %30 = load i32, ptr %4, align 4
  call void @check_match(ptr noundef %29, i32 noundef 261, i32 noundef 263, i32 noundef %30)
  %31 = load ptr, ptr %5, align 8
  call void @leaveblock(ptr noundef %31)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @repeatstat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca %struct.BlockCnt, align 8
  %9 = alloca %struct.BlockCnt, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = call i32 @luaK_getlabel(ptr noundef %14)
  store i32 %15, ptr %7, align 4
  %16 = load ptr, ptr %6, align 8
  call void @enterblock(ptr noundef %16, ptr noundef %8, i8 noundef zeroext 1)
  %17 = load ptr, ptr %6, align 8
  call void @enterblock(ptr noundef %17, ptr noundef %9, i8 noundef zeroext 0)
  %18 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %18)
  %19 = load ptr, ptr %3, align 8
  call void @statlist(ptr noundef %19)
  %20 = load ptr, ptr %3, align 8
  %21 = load i32, ptr %4, align 4
  call void @check_match(ptr noundef %20, i32 noundef 276, i32 noundef 272, i32 noundef %21)
  %22 = load ptr, ptr %3, align 8
  %23 = call i32 @cond(ptr noundef %22)
  store i32 %23, ptr %5, align 4
  %24 = load ptr, ptr %6, align 8
  call void @leaveblock(ptr noundef %24)
  %25 = getelementptr inbounds %struct.BlockCnt, ptr %9, i32 0, i32 4
  %26 = load i8, ptr %25, align 1
  %27 = icmp ne i8 %26, 0
  br i1 %27, label %28, label %44

28:                                               ; preds = %2
  %29 = load ptr, ptr %6, align 8
  %30 = call i32 @luaK_jump(ptr noundef %29)
  store i32 %30, ptr %10, align 4
  %31 = load ptr, ptr %6, align 8
  %32 = load i32, ptr %5, align 4
  call void @luaK_patchtohere(ptr noundef %31, i32 noundef %32)
  %33 = load ptr, ptr %6, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.BlockCnt, ptr %9, i32 0, i32 3
  %36 = load i8, ptr %35, align 8
  %37 = zext i8 %36 to i32
  %38 = call i32 @reglevel(ptr noundef %34, i32 noundef %37)
  %39 = call i32 @luaK_codeABCk(ptr noundef %33, i32 noundef 54, i32 noundef %38, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  %40 = load ptr, ptr %6, align 8
  %41 = call i32 @luaK_jump(ptr noundef %40)
  store i32 %41, ptr %5, align 4
  %42 = load ptr, ptr %6, align 8
  %43 = load i32, ptr %10, align 4
  call void @luaK_patchtohere(ptr noundef %42, i32 noundef %43)
  br label %44

44:                                               ; preds = %28, %2
  %45 = load ptr, ptr %6, align 8
  %46 = load i32, ptr %5, align 4
  %47 = load i32, ptr %7, align 4
  call void @luaK_patchlist(ptr noundef %45, i32 noundef %46, i32 noundef %47)
  %48 = load ptr, ptr %6, align 8
  call void @leaveblock(ptr noundef %48)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @funcstat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca %struct.expdesc, align 8
  %7 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %8)
  %9 = load ptr, ptr %3, align 8
  %10 = call i32 @funcname(ptr noundef %9, ptr noundef %6)
  store i32 %10, ptr %5, align 4
  %11 = load ptr, ptr %3, align 8
  %12 = load i32, ptr %5, align 4
  %13 = load i32, ptr %4, align 4
  call void @body(ptr noundef %11, ptr noundef %7, i32 noundef %12, i32 noundef %13)
  %14 = load ptr, ptr %3, align 8
  call void @check_readonly(ptr noundef %14, ptr noundef %6)
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 5
  %17 = load ptr, ptr %16, align 8
  call void @luaK_storevar(ptr noundef %17, ptr noundef %6, ptr noundef %7)
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 5
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %4, align 4
  call void @luaK_fixline(ptr noundef %20, i32 noundef %21)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @testnext(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 3
  %8 = getelementptr inbounds %struct.Token, ptr %7, i32 0, i32 0
  %9 = load i32, ptr %8, align 8
  %10 = load i32, ptr %5, align 4
  %11 = icmp eq i32 %9, %10
  br i1 %11, label %12, label %14

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  call void @luaX_next(ptr noundef %13)
  store i32 1, ptr %3, align 4
  br label %15

14:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %15

15:                                               ; preds = %14, %12
  %16 = load i32, ptr %3, align 4
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @localfunc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.expdesc, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 5
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 13
  %11 = load i8, ptr %10, align 2
  %12 = zext i8 %11 to i32
  store i32 %12, ptr %5, align 4
  %13 = load ptr, ptr %2, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = call ptr @str_checkname(ptr noundef %14)
  %16 = call i32 @new_localvar(ptr noundef %13, ptr noundef %15)
  %17 = load ptr, ptr %2, align 8
  call void @adjustlocalvars(ptr noundef %17, i32 noundef 1)
  %18 = load ptr, ptr %2, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 1
  %21 = load i32, ptr %20, align 4
  call void @body(ptr noundef %18, ptr noundef %3, i32 noundef 0, i32 noundef %21)
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 4
  %24 = load i32, ptr %23, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = load i32, ptr %5, align 4
  %27 = call ptr @localdebuginfo(ptr noundef %25, i32 noundef %26)
  %28 = getelementptr inbounds %struct.LocVar, ptr %27, i32 0, i32 1
  store i32 %24, ptr %28, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @localstat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %2, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %3, align 8
  store i32 -1, ptr %4, align 4
  store i32 0, ptr %8, align 4
  br label %14

14:                                               ; preds = %44, %1
  %15 = load ptr, ptr %2, align 8
  %16 = load ptr, ptr %2, align 8
  %17 = call ptr @str_checkname(ptr noundef %16)
  %18 = call i32 @new_localvar(ptr noundef %15, ptr noundef %17)
  store i32 %18, ptr %6, align 4
  %19 = load ptr, ptr %2, align 8
  %20 = call i32 @getlocalattribute(ptr noundef %19)
  store i32 %20, ptr %7, align 4
  %21 = load i32, ptr %7, align 4
  %22 = trunc i32 %21 to i8
  %23 = load ptr, ptr %3, align 8
  %24 = load i32, ptr %6, align 4
  %25 = call ptr @getlocalvardesc(ptr noundef %23, i32 noundef %24)
  %26 = getelementptr inbounds %struct.anon.9, ptr %25, i32 0, i32 2
  store i8 %22, ptr %26, align 1
  %27 = load i32, ptr %7, align 4
  %28 = icmp eq i32 %27, 2
  br i1 %28, label %29, label %41

29:                                               ; preds = %14
  %30 = load i32, ptr %4, align 4
  %31 = icmp ne i32 %30, -1
  br i1 %31, label %32, label %34

32:                                               ; preds = %29
  %33 = load ptr, ptr %2, align 8
  call void @luaK_semerror(ptr noundef %33, ptr noundef @.str.23) #6
  unreachable

34:                                               ; preds = %29
  %35 = load ptr, ptr %3, align 8
  %36 = getelementptr inbounds %struct.FuncState, ptr %35, i32 0, i32 13
  %37 = load i8, ptr %36, align 2
  %38 = zext i8 %37 to i32
  %39 = load i32, ptr %8, align 4
  %40 = add nsw i32 %38, %39
  store i32 %40, ptr %4, align 4
  br label %41

41:                                               ; preds = %34, %14
  %42 = load i32, ptr %8, align 4
  %43 = add nsw i32 %42, 1
  store i32 %43, ptr %8, align 4
  br label %44

44:                                               ; preds = %41
  %45 = load ptr, ptr %2, align 8
  %46 = call i32 @testnext(ptr noundef %45, i32 noundef 44)
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %14, label %48, !llvm.loop !11

48:                                               ; preds = %44
  %49 = load ptr, ptr %2, align 8
  %50 = call i32 @testnext(ptr noundef %49, i32 noundef 61)
  %51 = icmp ne i32 %50, 0
  br i1 %51, label %52, label %55

52:                                               ; preds = %48
  %53 = load ptr, ptr %2, align 8
  %54 = call i32 @explist(ptr noundef %53, ptr noundef %10)
  store i32 %54, ptr %9, align 4
  br label %57

55:                                               ; preds = %48
  %56 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 0
  store i32 0, ptr %56, align 8
  store i32 0, ptr %9, align 4
  br label %57

57:                                               ; preds = %55, %52
  %58 = load ptr, ptr %3, align 8
  %59 = load i32, ptr %6, align 4
  %60 = call ptr @getlocalvardesc(ptr noundef %58, i32 noundef %59)
  store ptr %60, ptr %5, align 8
  %61 = load i32, ptr %8, align 4
  %62 = load i32, ptr %9, align 4
  %63 = icmp eq i32 %61, %62
  br i1 %63, label %64, label %85

64:                                               ; preds = %57
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.anon.9, ptr %65, i32 0, i32 2
  %67 = load i8, ptr %66, align 1
  %68 = zext i8 %67 to i32
  %69 = icmp eq i32 %68, 1
  br i1 %69, label %70, label %85

70:                                               ; preds = %64
  %71 = load ptr, ptr %3, align 8
  %72 = load ptr, ptr %5, align 8
  %73 = call i32 @luaK_exp2const(ptr noundef %71, ptr noundef %10, ptr noundef %72)
  %74 = icmp ne i32 %73, 0
  br i1 %74, label %75, label %85

75:                                               ; preds = %70
  %76 = load ptr, ptr %5, align 8
  %77 = getelementptr inbounds %struct.anon.9, ptr %76, i32 0, i32 2
  store i8 3, ptr %77, align 1
  %78 = load ptr, ptr %2, align 8
  %79 = load i32, ptr %8, align 4
  %80 = sub nsw i32 %79, 1
  call void @adjustlocalvars(ptr noundef %78, i32 noundef %80)
  %81 = load ptr, ptr %3, align 8
  %82 = getelementptr inbounds %struct.FuncState, ptr %81, i32 0, i32 13
  %83 = load i8, ptr %82, align 2
  %84 = add i8 %83, 1
  store i8 %84, ptr %82, align 2
  br label %91

85:                                               ; preds = %70, %64, %57
  %86 = load ptr, ptr %2, align 8
  %87 = load i32, ptr %8, align 4
  %88 = load i32, ptr %9, align 4
  call void @adjust_assign(ptr noundef %86, i32 noundef %87, i32 noundef %88, ptr noundef %10)
  %89 = load ptr, ptr %2, align 8
  %90 = load i32, ptr %8, align 4
  call void @adjustlocalvars(ptr noundef %89, i32 noundef %90)
  br label %91

91:                                               ; preds = %85, %75
  %92 = load ptr, ptr %3, align 8
  %93 = load i32, ptr %4, align 4
  call void @checktoclose(ptr noundef %92, i32 noundef %93)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @labelstat(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  call void @checknext(ptr noundef %7, i32 noundef 287)
  br label %8

8:                                                ; preds = %22, %3
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 3
  %11 = getelementptr inbounds %struct.Token, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 8
  %13 = icmp eq i32 %12, 59
  br i1 %13, label %20, label %14

14:                                               ; preds = %8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 3
  %17 = getelementptr inbounds %struct.Token, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  %19 = icmp eq i32 %18, 287
  br label %20

20:                                               ; preds = %14, %8
  %21 = phi i1 [ true, %8 ], [ %19, %14 ]
  br i1 %21, label %22, label %24

22:                                               ; preds = %20
  %23 = load ptr, ptr %4, align 8
  call void @statement(ptr noundef %23)
  br label %8, !llvm.loop !12

24:                                               ; preds = %20
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  call void @checkrepeated(ptr noundef %25, ptr noundef %26)
  %27 = load ptr, ptr %4, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %6, align 4
  %30 = load ptr, ptr %4, align 8
  %31 = call i32 @block_follow(ptr noundef %30, i32 noundef 0)
  %32 = call i32 @createlabel(ptr noundef %27, ptr noundef %28, i32 noundef %29, i32 noundef %31)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @str_checkname(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @check(ptr noundef %4, i32 noundef 291)
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 3
  %7 = getelementptr inbounds %struct.Token, ptr %6, i32 0, i32 1
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %9)
  %10 = load ptr, ptr %3, align 8
  ret ptr %10
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @retstat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %struct.expdesc, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @luaY_nvarstack(ptr noundef %10)
  store i32 %11, ptr %6, align 4
  %12 = load ptr, ptr %2, align 8
  %13 = call i32 @block_follow(ptr noundef %12, i32 noundef 1)
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %21, label %15

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 3
  %18 = getelementptr inbounds %struct.Token, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  %20 = icmp eq i32 %19, 59
  br i1 %20, label %21, label %22

21:                                               ; preds = %15, %1
  store i32 0, ptr %5, align 4
  br label %80

22:                                               ; preds = %15
  %23 = load ptr, ptr %2, align 8
  %24 = call i32 @explist(ptr noundef %23, ptr noundef %4)
  store i32 %24, ptr %5, align 4
  %25 = getelementptr inbounds %struct.expdesc, ptr %4, i32 0, i32 0
  %26 = load i32, ptr %25, align 8
  %27 = icmp eq i32 %26, 18
  br i1 %27, label %32, label %28

28:                                               ; preds = %22
  %29 = getelementptr inbounds %struct.expdesc, ptr %4, i32 0, i32 0
  %30 = load i32, ptr %29, align 8
  %31 = icmp eq i32 %30, 19
  br i1 %31, label %32, label %70

32:                                               ; preds = %28, %22
  %33 = load ptr, ptr %3, align 8
  call void @luaK_setreturns(ptr noundef %33, ptr noundef %4, i32 noundef -1)
  %34 = getelementptr inbounds %struct.expdesc, ptr %4, i32 0, i32 0
  %35 = load i32, ptr %34, align 8
  %36 = icmp eq i32 %35, 18
  br i1 %36, label %37, label %69

37:                                               ; preds = %32
  %38 = load i32, ptr %5, align 4
  %39 = icmp eq i32 %38, 1
  br i1 %39, label %40, label %69

40:                                               ; preds = %37
  %41 = load ptr, ptr %3, align 8
  %42 = getelementptr inbounds %struct.FuncState, ptr %41, i32 0, i32 3
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %struct.BlockCnt, ptr %43, i32 0, i32 6
  %45 = load i8, ptr %44, align 1
  %46 = icmp ne i8 %45, 0
  br i1 %46, label %69, label %47

47:                                               ; preds = %40
  %48 = load ptr, ptr %3, align 8
  %49 = getelementptr inbounds %struct.FuncState, ptr %48, i32 0, i32 0
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.Proto, ptr %50, i32 0, i32 16
  %52 = load ptr, ptr %51, align 8
  %53 = getelementptr inbounds %struct.expdesc, ptr %4, i32 0, i32 1
  %54 = load i32, ptr %53, align 8
  %55 = sext i32 %54 to i64
  %56 = getelementptr inbounds i32, ptr %52, i64 %55
  %57 = load i32, ptr %56, align 4
  %58 = and i32 %57, -128
  %59 = or i32 %58, 69
  %60 = load ptr, ptr %3, align 8
  %61 = getelementptr inbounds %struct.FuncState, ptr %60, i32 0, i32 0
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds %struct.Proto, ptr %62, i32 0, i32 16
  %64 = load ptr, ptr %63, align 8
  %65 = getelementptr inbounds %struct.expdesc, ptr %4, i32 0, i32 1
  %66 = load i32, ptr %65, align 8
  %67 = sext i32 %66 to i64
  %68 = getelementptr inbounds i32, ptr %64, i64 %67
  store i32 %59, ptr %68, align 4
  br label %69

69:                                               ; preds = %47, %40, %37, %32
  store i32 -1, ptr %5, align 4
  br label %79

70:                                               ; preds = %28
  %71 = load i32, ptr %5, align 4
  %72 = icmp eq i32 %71, 1
  br i1 %72, label %73, label %76

73:                                               ; preds = %70
  %74 = load ptr, ptr %3, align 8
  %75 = call i32 @luaK_exp2anyreg(ptr noundef %74, ptr noundef %4)
  store i32 %75, ptr %6, align 4
  br label %78

76:                                               ; preds = %70
  %77 = load ptr, ptr %3, align 8
  call void @luaK_exp2nextreg(ptr noundef %77, ptr noundef %4)
  br label %78

78:                                               ; preds = %76, %73
  br label %79

79:                                               ; preds = %78, %69
  br label %80

80:                                               ; preds = %79, %21
  %81 = load ptr, ptr %3, align 8
  %82 = load i32, ptr %6, align 4
  %83 = load i32, ptr %5, align 4
  call void @luaK_ret(ptr noundef %81, i32 noundef %82, i32 noundef %83)
  %84 = load ptr, ptr %2, align 8
  %85 = call i32 @testnext(ptr noundef %84, i32 noundef 59)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @breakstat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LexState, ptr %4, i32 0, i32 1
  %6 = load i32, ptr %5, align 4
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %7)
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 6
  %11 = load ptr, ptr %10, align 8
  %12 = call ptr @luaS_newlstr(ptr noundef %11, ptr noundef @.str.4, i64 noundef 5)
  %13 = load i32, ptr %3, align 4
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 5
  %16 = load ptr, ptr %15, align 8
  %17 = call i32 @luaK_jump(ptr noundef %16)
  %18 = call i32 @newgotoentry(ptr noundef %8, ptr noundef %12, i32 noundef %13, i32 noundef %17)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @gotostat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 5
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %3, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 4
  store i32 %13, ptr %4, align 4
  %14 = load ptr, ptr %2, align 8
  %15 = call ptr @str_checkname(ptr noundef %14)
  store ptr %15, ptr %5, align 8
  %16 = load ptr, ptr %2, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = call ptr @findlabel(ptr noundef %16, ptr noundef %17)
  store ptr %18, ptr %6, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = icmp eq ptr %19, null
  br i1 %20, label %21, label %28

21:                                               ; preds = %1
  %22 = load ptr, ptr %2, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = load i32, ptr %4, align 4
  %25 = load ptr, ptr %3, align 8
  %26 = call i32 @luaK_jump(ptr noundef %25)
  %27 = call i32 @newgotoentry(ptr noundef %22, ptr noundef %23, i32 noundef %24, i32 noundef %26)
  br label %50

28:                                               ; preds = %1
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.Labeldesc, ptr %30, i32 0, i32 3
  %32 = load i8, ptr %31, align 8
  %33 = zext i8 %32 to i32
  %34 = call i32 @reglevel(ptr noundef %29, i32 noundef %33)
  store i32 %34, ptr %7, align 4
  %35 = load ptr, ptr %3, align 8
  %36 = call i32 @luaY_nvarstack(ptr noundef %35)
  %37 = load i32, ptr %7, align 4
  %38 = icmp sgt i32 %36, %37
  br i1 %38, label %39, label %43

39:                                               ; preds = %28
  %40 = load ptr, ptr %3, align 8
  %41 = load i32, ptr %7, align 4
  %42 = call i32 @luaK_codeABCk(ptr noundef %40, i32 noundef 54, i32 noundef %41, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  br label %43

43:                                               ; preds = %39, %28
  %44 = load ptr, ptr %3, align 8
  %45 = load ptr, ptr %3, align 8
  %46 = call i32 @luaK_jump(ptr noundef %45)
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.Labeldesc, ptr %47, i32 0, i32 1
  %49 = load i32, ptr %48, align 8
  call void @luaK_patchlist(ptr noundef %44, i32 noundef %46, i32 noundef %49)
  br label %50

50:                                               ; preds = %43, %21
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @exprstat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %struct.LHS_assign, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 5
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LHS_assign, ptr %4, i32 0, i32 1
  call void @suffixedexp(ptr noundef %9, ptr noundef %10)
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 3
  %13 = getelementptr inbounds %struct.Token, ptr %12, i32 0, i32 0
  %14 = load i32, ptr %13, align 8
  %15 = icmp eq i32 %14, 61
  br i1 %15, label %22, label %16

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.LexState, ptr %17, i32 0, i32 3
  %19 = getelementptr inbounds %struct.Token, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 44
  br i1 %21, label %22, label %25

22:                                               ; preds = %16, %1
  %23 = getelementptr inbounds %struct.LHS_assign, ptr %4, i32 0, i32 0
  store ptr null, ptr %23, align 8
  %24 = load ptr, ptr %2, align 8
  call void @restassign(ptr noundef %24, ptr noundef %4, i32 noundef 1)
  br label %48

25:                                               ; preds = %16
  %26 = getelementptr inbounds %struct.LHS_assign, ptr %4, i32 0, i32 1
  %27 = getelementptr inbounds %struct.expdesc, ptr %26, i32 0, i32 0
  %28 = load i32, ptr %27, align 8
  %29 = icmp eq i32 %28, 18
  br i1 %29, label %32, label %30

30:                                               ; preds = %25
  %31 = load ptr, ptr %2, align 8
  call void @luaX_syntaxerror(ptr noundef %31, ptr noundef @.str.28) #6
  unreachable

32:                                               ; preds = %25
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.FuncState, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.Proto, ptr %35, i32 0, i32 16
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds %struct.LHS_assign, ptr %4, i32 0, i32 1
  %39 = getelementptr inbounds %struct.expdesc, ptr %38, i32 0, i32 1
  %40 = load i32, ptr %39, align 8
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds i32, ptr %37, i64 %41
  store ptr %42, ptr %5, align 8
  %43 = load ptr, ptr %5, align 8
  %44 = load i32, ptr %43, align 4
  %45 = and i32 %44, 16777215
  %46 = or i32 %45, 16777216
  %47 = load ptr, ptr %5, align 8
  store i32 %46, ptr %47, align 4
  br label %48

48:                                               ; preds = %32, %22
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @test_then_block(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.BlockCnt, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.expdesc, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %13)
  %14 = load ptr, ptr %3, align 8
  call void @expr(ptr noundef %14, ptr noundef %7)
  %15 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %15, i32 noundef 274)
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 3
  %18 = getelementptr inbounds %struct.Token, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  %20 = icmp eq i32 %19, 257
  br i1 %20, label %21, label %54

21:                                               ; preds = %2
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.LexState, ptr %22, i32 0, i32 1
  %24 = load i32, ptr %23, align 4
  store i32 %24, ptr %9, align 4
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.LexState, ptr %25, i32 0, i32 5
  %27 = load ptr, ptr %26, align 8
  call void @luaK_goiffalse(ptr noundef %27, ptr noundef %7)
  %28 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %28)
  %29 = load ptr, ptr %6, align 8
  call void @enterblock(ptr noundef %29, ptr noundef %5, i8 noundef zeroext 0)
  %30 = load ptr, ptr %3, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.LexState, ptr %31, i32 0, i32 6
  %33 = load ptr, ptr %32, align 8
  %34 = call ptr @luaS_newlstr(ptr noundef %33, ptr noundef @.str.4, i64 noundef 5)
  %35 = load i32, ptr %9, align 4
  %36 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 2
  %37 = load i32, ptr %36, align 8
  %38 = call i32 @newgotoentry(ptr noundef %30, ptr noundef %34, i32 noundef %35, i32 noundef %37)
  br label %39

39:                                               ; preds = %43, %21
  %40 = load ptr, ptr %3, align 8
  %41 = call i32 @testnext(ptr noundef %40, i32 noundef 59)
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %43, label %44

43:                                               ; preds = %39
  br label %39, !llvm.loop !13

44:                                               ; preds = %39
  %45 = load ptr, ptr %3, align 8
  %46 = call i32 @block_follow(ptr noundef %45, i32 noundef 0)
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %48, label %50

48:                                               ; preds = %44
  %49 = load ptr, ptr %6, align 8
  call void @leaveblock(ptr noundef %49)
  br label %83

50:                                               ; preds = %44
  %51 = load ptr, ptr %6, align 8
  %52 = call i32 @luaK_jump(ptr noundef %51)
  store i32 %52, ptr %8, align 4
  br label %53

53:                                               ; preds = %50
  br label %61

54:                                               ; preds = %2
  %55 = load ptr, ptr %3, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 5
  %57 = load ptr, ptr %56, align 8
  call void @luaK_goiftrue(ptr noundef %57, ptr noundef %7)
  %58 = load ptr, ptr %6, align 8
  call void @enterblock(ptr noundef %58, ptr noundef %5, i8 noundef zeroext 0)
  %59 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 3
  %60 = load i32, ptr %59, align 4
  store i32 %60, ptr %8, align 4
  br label %61

61:                                               ; preds = %54, %53
  %62 = load ptr, ptr %3, align 8
  call void @statlist(ptr noundef %62)
  %63 = load ptr, ptr %6, align 8
  call void @leaveblock(ptr noundef %63)
  %64 = load ptr, ptr %3, align 8
  %65 = getelementptr inbounds %struct.LexState, ptr %64, i32 0, i32 3
  %66 = getelementptr inbounds %struct.Token, ptr %65, i32 0, i32 0
  %67 = load i32, ptr %66, align 8
  %68 = icmp eq i32 %67, 259
  br i1 %68, label %75, label %69

69:                                               ; preds = %61
  %70 = load ptr, ptr %3, align 8
  %71 = getelementptr inbounds %struct.LexState, ptr %70, i32 0, i32 3
  %72 = getelementptr inbounds %struct.Token, ptr %71, i32 0, i32 0
  %73 = load i32, ptr %72, align 8
  %74 = icmp eq i32 %73, 260
  br i1 %74, label %75, label %80

75:                                               ; preds = %69, %61
  %76 = load ptr, ptr %6, align 8
  %77 = load ptr, ptr %4, align 8
  %78 = load ptr, ptr %6, align 8
  %79 = call i32 @luaK_jump(ptr noundef %78)
  call void @luaK_concat(ptr noundef %76, ptr noundef %77, i32 noundef %79)
  br label %80

80:                                               ; preds = %75, %69
  %81 = load ptr, ptr %6, align 8
  %82 = load i32, ptr %8, align 4
  call void @luaK_patchtohere(ptr noundef %81, i32 noundef %82)
  br label %83

83:                                               ; preds = %80, %48
  ret void
}

declare hidden void @luaK_patchtohere(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @expr(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = call i32 @subexpr(ptr noundef %5, ptr noundef %6, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checknext(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  call void @check(ptr noundef %5, i32 noundef %6)
  %7 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %7)
  ret void
}

declare hidden void @luaK_goiffalse(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @newgotoentry(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 10
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.Dyndata, ptr %12, i32 0, i32 1
  %14 = load ptr, ptr %6, align 8
  %15 = load i32, ptr %7, align 4
  %16 = load i32, ptr %8, align 4
  %17 = call i32 @newlabelentry(ptr noundef %9, ptr noundef %13, ptr noundef %14, i32 noundef %15, i32 noundef %16)
  ret i32 %17
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @leaveblock(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 3
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 2
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %4, align 8
  store i32 0, ptr %5, align 4
  %13 = load ptr, ptr %2, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.BlockCnt, ptr %14, i32 0, i32 3
  %16 = load i8, ptr %15, align 8
  %17 = zext i8 %16 to i32
  %18 = call i32 @reglevel(ptr noundef %13, i32 noundef %17)
  store i32 %18, ptr %6, align 4
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.BlockCnt, ptr %20, i32 0, i32 3
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  call void @removevars(ptr noundef %19, i32 noundef %23)
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.BlockCnt, ptr %24, i32 0, i32 5
  %26 = load i8, ptr %25, align 2
  %27 = icmp ne i8 %26, 0
  br i1 %27, label %28, label %35

28:                                               ; preds = %1
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  %33 = call ptr @luaS_newlstr(ptr noundef %32, ptr noundef @.str.4, i64 noundef 5)
  %34 = call i32 @createlabel(ptr noundef %29, ptr noundef %33, i32 noundef 0, i32 noundef 0)
  store i32 %34, ptr %5, align 4
  br label %35

35:                                               ; preds = %28, %1
  %36 = load i32, ptr %5, align 4
  %37 = icmp ne i32 %36, 0
  br i1 %37, label %53, label %38

38:                                               ; preds = %35
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.BlockCnt, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  %42 = icmp ne ptr %41, null
  br i1 %42, label %43, label %53

43:                                               ; preds = %38
  %44 = load ptr, ptr %3, align 8
  %45 = getelementptr inbounds %struct.BlockCnt, ptr %44, i32 0, i32 4
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = icmp ne i32 %47, 0
  br i1 %48, label %49, label %53

49:                                               ; preds = %43
  %50 = load ptr, ptr %2, align 8
  %51 = load i32, ptr %6, align 4
  %52 = call i32 @luaK_codeABCk(ptr noundef %50, i32 noundef 54, i32 noundef %51, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  br label %53

53:                                               ; preds = %49, %43, %38, %35
  %54 = load i32, ptr %6, align 4
  %55 = trunc i32 %54 to i8
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds %struct.FuncState, ptr %56, i32 0, i32 15
  store i8 %55, ptr %57, align 4
  %58 = load ptr, ptr %3, align 8
  %59 = getelementptr inbounds %struct.BlockCnt, ptr %58, i32 0, i32 1
  %60 = load i32, ptr %59, align 8
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.LexState, ptr %61, i32 0, i32 10
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %struct.Dyndata, ptr %63, i32 0, i32 2
  %65 = getelementptr inbounds %struct.Labellist, ptr %64, i32 0, i32 1
  store i32 %60, ptr %65, align 8
  %66 = load ptr, ptr %3, align 8
  %67 = getelementptr inbounds %struct.BlockCnt, ptr %66, i32 0, i32 0
  %68 = load ptr, ptr %67, align 8
  %69 = load ptr, ptr %2, align 8
  %70 = getelementptr inbounds %struct.FuncState, ptr %69, i32 0, i32 3
  store ptr %68, ptr %70, align 8
  %71 = load ptr, ptr %3, align 8
  %72 = getelementptr inbounds %struct.BlockCnt, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = icmp ne ptr %73, null
  br i1 %74, label %75, label %78

75:                                               ; preds = %53
  %76 = load ptr, ptr %2, align 8
  %77 = load ptr, ptr %3, align 8
  call void @movegotosout(ptr noundef %76, ptr noundef %77)
  br label %103

78:                                               ; preds = %53
  %79 = load ptr, ptr %3, align 8
  %80 = getelementptr inbounds %struct.BlockCnt, ptr %79, i32 0, i32 2
  %81 = load i32, ptr %80, align 4
  %82 = load ptr, ptr %4, align 8
  %83 = getelementptr inbounds %struct.LexState, ptr %82, i32 0, i32 10
  %84 = load ptr, ptr %83, align 8
  %85 = getelementptr inbounds %struct.Dyndata, ptr %84, i32 0, i32 1
  %86 = getelementptr inbounds %struct.Labellist, ptr %85, i32 0, i32 1
  %87 = load i32, ptr %86, align 8
  %88 = icmp slt i32 %81, %87
  br i1 %88, label %89, label %102

89:                                               ; preds = %78
  %90 = load ptr, ptr %4, align 8
  %91 = load ptr, ptr %4, align 8
  %92 = getelementptr inbounds %struct.LexState, ptr %91, i32 0, i32 10
  %93 = load ptr, ptr %92, align 8
  %94 = getelementptr inbounds %struct.Dyndata, ptr %93, i32 0, i32 1
  %95 = getelementptr inbounds %struct.Labellist, ptr %94, i32 0, i32 0
  %96 = load ptr, ptr %95, align 8
  %97 = load ptr, ptr %3, align 8
  %98 = getelementptr inbounds %struct.BlockCnt, ptr %97, i32 0, i32 2
  %99 = load i32, ptr %98, align 4
  %100 = sext i32 %99 to i64
  %101 = getelementptr inbounds %struct.Labeldesc, ptr %96, i64 %100
  call void @undefgoto(ptr noundef %90, ptr noundef %101) #6
  unreachable

102:                                              ; preds = %78
  br label %103

103:                                              ; preds = %102, %75
  ret void
}

declare hidden i32 @luaK_jump(ptr noundef) #1

declare hidden void @luaK_goiftrue(ptr noundef, ptr noundef) #1

declare hidden void @luaK_concat(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @subexpr(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca %struct.expdesc, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  call void @luaE_incCstack(ptr noundef %15)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 3
  %18 = getelementptr inbounds %struct.Token, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  %20 = call i32 @getunopr(i32 noundef %19)
  store i32 %20, ptr %8, align 4
  %21 = load i32, ptr %8, align 4
  %22 = icmp ne i32 %21, 4
  br i1 %22, label %23, label %37

23:                                               ; preds = %3
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 1
  %26 = load i32, ptr %25, align 4
  store i32 %26, ptr %9, align 4
  %27 = load ptr, ptr %4, align 8
  call void @luaX_next(ptr noundef %27)
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = call i32 @subexpr(ptr noundef %28, ptr noundef %29, i32 noundef 12)
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.LexState, ptr %31, i32 0, i32 5
  %33 = load ptr, ptr %32, align 8
  %34 = load i32, ptr %8, align 4
  %35 = load ptr, ptr %5, align 8
  %36 = load i32, ptr %9, align 4
  call void @luaK_prefix(ptr noundef %33, i32 noundef %34, ptr noundef %35, i32 noundef %36)
  br label %40

37:                                               ; preds = %3
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %5, align 8
  call void @simpleexp(ptr noundef %38, ptr noundef %39)
  br label %40

40:                                               ; preds = %37, %23
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.LexState, ptr %41, i32 0, i32 3
  %43 = getelementptr inbounds %struct.Token, ptr %42, i32 0, i32 0
  %44 = load i32, ptr %43, align 8
  %45 = call i32 @getbinopr(i32 noundef %44)
  store i32 %45, ptr %7, align 4
  br label %46

46:                                               ; preds = %60, %40
  %47 = load i32, ptr %7, align 4
  %48 = icmp ne i32 %47, 21
  br i1 %48, label %49, label %58

49:                                               ; preds = %46
  %50 = load i32, ptr %7, align 4
  %51 = zext i32 %50 to i64
  %52 = getelementptr inbounds [21 x %struct.anon.13], ptr @priority, i64 0, i64 %51
  %53 = getelementptr inbounds %struct.anon.13, ptr %52, i32 0, i32 0
  %54 = load i8, ptr %53, align 2
  %55 = zext i8 %54 to i32
  %56 = load i32, ptr %6, align 4
  %57 = icmp sgt i32 %55, %56
  br label %58

58:                                               ; preds = %49, %46
  %59 = phi i1 [ false, %46 ], [ %57, %49 ]
  br i1 %59, label %60, label %85

60:                                               ; preds = %58
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.LexState, ptr %61, i32 0, i32 1
  %63 = load i32, ptr %62, align 4
  store i32 %63, ptr %12, align 4
  %64 = load ptr, ptr %4, align 8
  call void @luaX_next(ptr noundef %64)
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.LexState, ptr %65, i32 0, i32 5
  %67 = load ptr, ptr %66, align 8
  %68 = load i32, ptr %7, align 4
  %69 = load ptr, ptr %5, align 8
  call void @luaK_infix(ptr noundef %67, i32 noundef %68, ptr noundef %69)
  %70 = load ptr, ptr %4, align 8
  %71 = load i32, ptr %7, align 4
  %72 = zext i32 %71 to i64
  %73 = getelementptr inbounds [21 x %struct.anon.13], ptr @priority, i64 0, i64 %72
  %74 = getelementptr inbounds %struct.anon.13, ptr %73, i32 0, i32 1
  %75 = load i8, ptr %74, align 1
  %76 = zext i8 %75 to i32
  %77 = call i32 @subexpr(ptr noundef %70, ptr noundef %10, i32 noundef %76)
  store i32 %77, ptr %11, align 4
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.LexState, ptr %78, i32 0, i32 5
  %80 = load ptr, ptr %79, align 8
  %81 = load i32, ptr %7, align 4
  %82 = load ptr, ptr %5, align 8
  %83 = load i32, ptr %12, align 4
  call void @luaK_posfix(ptr noundef %80, i32 noundef %81, ptr noundef %82, ptr noundef %10, i32 noundef %83)
  %84 = load i32, ptr %11, align 4
  store i32 %84, ptr %7, align 4
  br label %46, !llvm.loop !14

85:                                               ; preds = %58
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.LexState, ptr %86, i32 0, i32 6
  %88 = load ptr, ptr %87, align 8
  %89 = getelementptr inbounds %struct.lua_State, ptr %88, i32 0, i32 19
  %90 = load i32, ptr %89, align 8
  %91 = add i32 %90, -1
  store i32 %91, ptr %89, align 8
  %92 = load i32, ptr %7, align 4
  ret i32 %92
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getunopr(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  %4 = load i32, ptr %3, align 4
  switch i32 %4, label %9 [
    i32 270, label %5
    i32 45, label %6
    i32 126, label %7
    i32 35, label %8
  ]

5:                                                ; preds = %1
  store i32 2, ptr %2, align 4
  br label %10

6:                                                ; preds = %1
  store i32 0, ptr %2, align 4
  br label %10

7:                                                ; preds = %1
  store i32 1, ptr %2, align 4
  br label %10

8:                                                ; preds = %1
  store i32 3, ptr %2, align 4
  br label %10

9:                                                ; preds = %1
  store i32 4, ptr %2, align 4
  br label %10

10:                                               ; preds = %9, %8, %7, %6, %5
  %11 = load i32, ptr %2, align 4
  ret i32 %11
}

declare hidden void @luaK_prefix(ptr noundef, i32 noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @simpleexp(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 3
  %8 = getelementptr inbounds %struct.Token, ptr %7, i32 0, i32 0
  %9 = load i32, ptr %8, align 8
  switch i32 %9, label %64 [
    i32 289, label %10
    i32 290, label %18
    i32 292, label %26
    i32 269, label %32
    i32 275, label %34
    i32 262, label %36
    i32 280, label %38
    i32 123, label %54
    i32 264, label %57
  ]

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  call void @init_exp(ptr noundef %11, i32 noundef 5, i32 noundef 0)
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 3
  %14 = getelementptr inbounds %struct.Token, ptr %13, i32 0, i32 1
  %15 = load double, ptr %14, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 1
  store double %15, ptr %17, align 8
  br label %67

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  call void @init_exp(ptr noundef %19, i32 noundef 6, i32 noundef 0)
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 3
  %22 = getelementptr inbounds %struct.Token, ptr %21, i32 0, i32 1
  %23 = load i64, ptr %22, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.expdesc, ptr %24, i32 0, i32 1
  store i64 %23, ptr %25, align 8
  br label %67

26:                                               ; preds = %2
  %27 = load ptr, ptr %4, align 8
  %28 = load ptr, ptr %3, align 8
  %29 = getelementptr inbounds %struct.LexState, ptr %28, i32 0, i32 3
  %30 = getelementptr inbounds %struct.Token, ptr %29, i32 0, i32 1
  %31 = load ptr, ptr %30, align 8
  call void @codestring(ptr noundef %27, ptr noundef %31)
  br label %67

32:                                               ; preds = %2
  %33 = load ptr, ptr %4, align 8
  call void @init_exp(ptr noundef %33, i32 noundef 1, i32 noundef 0)
  br label %67

34:                                               ; preds = %2
  %35 = load ptr, ptr %4, align 8
  call void @init_exp(ptr noundef %35, i32 noundef 2, i32 noundef 0)
  br label %67

36:                                               ; preds = %2
  %37 = load ptr, ptr %4, align 8
  call void @init_exp(ptr noundef %37, i32 noundef 3, i32 noundef 0)
  br label %67

38:                                               ; preds = %2
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.LexState, ptr %39, i32 0, i32 5
  %41 = load ptr, ptr %40, align 8
  store ptr %41, ptr %5, align 8
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.FuncState, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.Proto, ptr %44, i32 0, i32 4
  %46 = load i8, ptr %45, align 1
  %47 = icmp ne i8 %46, 0
  br i1 %47, label %50, label %48

48:                                               ; preds = %38
  %49 = load ptr, ptr %3, align 8
  call void @luaX_syntaxerror(ptr noundef %49, ptr noundef @.str.5) #6
  unreachable

50:                                               ; preds = %38
  %51 = load ptr, ptr %4, align 8
  %52 = load ptr, ptr %5, align 8
  %53 = call i32 @luaK_codeABCk(ptr noundef %52, i32 noundef 80, i32 noundef 0, i32 noundef 0, i32 noundef 1, i32 noundef 0)
  call void @init_exp(ptr noundef %51, i32 noundef 19, i32 noundef %53)
  br label %67

54:                                               ; preds = %2
  %55 = load ptr, ptr %3, align 8
  %56 = load ptr, ptr %4, align 8
  call void @constructor(ptr noundef %55, ptr noundef %56)
  br label %69

57:                                               ; preds = %2
  %58 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %58)
  %59 = load ptr, ptr %3, align 8
  %60 = load ptr, ptr %4, align 8
  %61 = load ptr, ptr %3, align 8
  %62 = getelementptr inbounds %struct.LexState, ptr %61, i32 0, i32 1
  %63 = load i32, ptr %62, align 4
  call void @body(ptr noundef %59, ptr noundef %60, i32 noundef 0, i32 noundef %63)
  br label %69

64:                                               ; preds = %2
  %65 = load ptr, ptr %3, align 8
  %66 = load ptr, ptr %4, align 8
  call void @suffixedexp(ptr noundef %65, ptr noundef %66)
  br label %69

67:                                               ; preds = %50, %36, %34, %32, %26, %18, %10
  %68 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %68)
  br label %69

69:                                               ; preds = %67, %64, %57, %54
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getbinopr(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  %4 = load i32, ptr %3, align 4
  switch i32 %4, label %26 [
    i32 43, label %5
    i32 45, label %6
    i32 42, label %7
    i32 37, label %8
    i32 94, label %9
    i32 47, label %10
    i32 278, label %11
    i32 38, label %12
    i32 124, label %13
    i32 126, label %14
    i32 285, label %15
    i32 286, label %16
    i32 279, label %17
    i32 284, label %18
    i32 281, label %19
    i32 60, label %20
    i32 283, label %21
    i32 62, label %22
    i32 282, label %23
    i32 256, label %24
    i32 271, label %25
  ]

5:                                                ; preds = %1
  store i32 0, ptr %2, align 4
  br label %27

6:                                                ; preds = %1
  store i32 1, ptr %2, align 4
  br label %27

7:                                                ; preds = %1
  store i32 2, ptr %2, align 4
  br label %27

8:                                                ; preds = %1
  store i32 3, ptr %2, align 4
  br label %27

9:                                                ; preds = %1
  store i32 4, ptr %2, align 4
  br label %27

10:                                               ; preds = %1
  store i32 5, ptr %2, align 4
  br label %27

11:                                               ; preds = %1
  store i32 6, ptr %2, align 4
  br label %27

12:                                               ; preds = %1
  store i32 7, ptr %2, align 4
  br label %27

13:                                               ; preds = %1
  store i32 8, ptr %2, align 4
  br label %27

14:                                               ; preds = %1
  store i32 9, ptr %2, align 4
  br label %27

15:                                               ; preds = %1
  store i32 10, ptr %2, align 4
  br label %27

16:                                               ; preds = %1
  store i32 11, ptr %2, align 4
  br label %27

17:                                               ; preds = %1
  store i32 12, ptr %2, align 4
  br label %27

18:                                               ; preds = %1
  store i32 16, ptr %2, align 4
  br label %27

19:                                               ; preds = %1
  store i32 13, ptr %2, align 4
  br label %27

20:                                               ; preds = %1
  store i32 14, ptr %2, align 4
  br label %27

21:                                               ; preds = %1
  store i32 15, ptr %2, align 4
  br label %27

22:                                               ; preds = %1
  store i32 17, ptr %2, align 4
  br label %27

23:                                               ; preds = %1
  store i32 18, ptr %2, align 4
  br label %27

24:                                               ; preds = %1
  store i32 19, ptr %2, align 4
  br label %27

25:                                               ; preds = %1
  store i32 20, ptr %2, align 4
  br label %27

26:                                               ; preds = %1
  store i32 21, ptr %2, align 4
  br label %27

27:                                               ; preds = %26, %25, %24, %23, %22, %21, %20, %19, %18, %17, %16, %15, %14, %13, %12, %11, %10, %9, %8, %7, %6, %5
  %28 = load i32, ptr %2, align 4
  ret i32 %28
}

declare hidden void @luaK_infix(ptr noundef, i32 noundef, ptr noundef) #1

declare hidden void @luaK_posfix(ptr noundef, i32 noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @init_exp(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 2
  store i32 -1, ptr %8, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 3
  store i32 -1, ptr %10, align 4
  %11 = load i32, ptr %5, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 0
  store i32 %11, ptr %13, align 8
  %14 = load i32, ptr %6, align 4
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.expdesc, ptr %15, i32 0, i32 1
  store i32 %14, ptr %16, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codestring(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 2
  store i32 -1, ptr %6, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 3
  store i32 -1, ptr %8, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 0
  store i32 7, ptr %10, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 1
  store ptr %11, ptr %13, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @constructor(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca %struct.ConsControl, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 5
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 1
  %14 = load i32, ptr %13, align 4
  store i32 %14, ptr %6, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = call i32 @luaK_codeABCk(ptr noundef %15, i32 noundef 19, i32 noundef 0, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  store i32 %16, ptr %7, align 4
  %17 = load ptr, ptr %5, align 8
  %18 = call i32 @luaK_code(ptr noundef %17, i32 noundef 0)
  %19 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 4
  store i32 0, ptr %19, align 8
  %20 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 2
  store i32 0, ptr %20, align 8
  %21 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 3
  store i32 0, ptr %21, align 4
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 1
  store ptr %22, ptr %23, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.FuncState, ptr %25, i32 0, i32 15
  %27 = load i8, ptr %26, align 4
  %28 = zext i8 %27 to i32
  call void @init_exp(ptr noundef %24, i32 noundef 8, i32 noundef %28)
  %29 = load ptr, ptr %5, align 8
  call void @luaK_reserveregs(ptr noundef %29, i32 noundef 1)
  %30 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 0
  call void @init_exp(ptr noundef %30, i32 noundef 0, i32 noundef 0)
  %31 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %31, i32 noundef 123)
  br label %32

32:                                               ; preds = %50, %2
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.LexState, ptr %33, i32 0, i32 3
  %35 = getelementptr inbounds %struct.Token, ptr %34, i32 0, i32 0
  %36 = load i32, ptr %35, align 8
  %37 = icmp eq i32 %36, 125
  br i1 %37, label %38, label %39

38:                                               ; preds = %32
  br label %52

39:                                               ; preds = %32
  %40 = load ptr, ptr %5, align 8
  call void @closelistfield(ptr noundef %40, ptr noundef %8)
  %41 = load ptr, ptr %3, align 8
  call void @field(ptr noundef %41, ptr noundef %8)
  br label %42

42:                                               ; preds = %39
  %43 = load ptr, ptr %3, align 8
  %44 = call i32 @testnext(ptr noundef %43, i32 noundef 44)
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %50, label %46

46:                                               ; preds = %42
  %47 = load ptr, ptr %3, align 8
  %48 = call i32 @testnext(ptr noundef %47, i32 noundef 59)
  %49 = icmp ne i32 %48, 0
  br label %50

50:                                               ; preds = %46, %42
  %51 = phi i1 [ true, %42 ], [ %49, %46 ]
  br i1 %51, label %32, label %52, !llvm.loop !15

52:                                               ; preds = %50, %38
  %53 = load ptr, ptr %3, align 8
  %54 = load i32, ptr %6, align 4
  call void @check_match(ptr noundef %53, i32 noundef 125, i32 noundef 123, i32 noundef %54)
  %55 = load ptr, ptr %5, align 8
  call void @lastlistfield(ptr noundef %55, ptr noundef %8)
  %56 = load ptr, ptr %5, align 8
  %57 = load i32, ptr %7, align 4
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.expdesc, ptr %58, i32 0, i32 1
  %60 = load i32, ptr %59, align 8
  %61 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 3
  %62 = load i32, ptr %61, align 4
  %63 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 2
  %64 = load i32, ptr %63, align 8
  call void @luaK_settablesize(ptr noundef %56, i32 noundef %57, i32 noundef %60, i32 noundef %62, i32 noundef %64)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @body(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca %struct.FuncState, align 8
  %10 = alloca %struct.BlockCnt, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = call ptr @addprototype(ptr noundef %11)
  %13 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 0
  store ptr %12, ptr %13, align 8
  %14 = load i32, ptr %8, align 4
  %15 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 13
  store i32 %14, ptr %17, align 4
  %18 = load ptr, ptr %5, align 8
  call void @open_func(ptr noundef %18, ptr noundef %9, ptr noundef %10)
  %19 = load ptr, ptr %5, align 8
  call void @checknext(ptr noundef %19, i32 noundef 40)
  %20 = load i32, ptr %7, align 4
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %22, label %28

22:                                               ; preds = %4
  %23 = load ptr, ptr %5, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = call ptr @luaX_newstring(ptr noundef %24, ptr noundef @.str.7, i64 noundef 4)
  %26 = call i32 @new_localvar(ptr noundef %23, ptr noundef %25)
  %27 = load ptr, ptr %5, align 8
  call void @adjustlocalvars(ptr noundef %27, i32 noundef 1)
  br label %28

28:                                               ; preds = %22, %4
  %29 = load ptr, ptr %5, align 8
  call void @parlist(ptr noundef %29)
  %30 = load ptr, ptr %5, align 8
  call void @checknext(ptr noundef %30, i32 noundef 41)
  %31 = load ptr, ptr %5, align 8
  call void @statlist(ptr noundef %31)
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds %struct.LexState, ptr %32, i32 0, i32 1
  %34 = load i32, ptr %33, align 4
  %35 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 14
  store i32 %34, ptr %37, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = load i32, ptr %8, align 4
  call void @check_match(ptr noundef %38, i32 noundef 261, i32 noundef 264, i32 noundef %39)
  %40 = load ptr, ptr %5, align 8
  %41 = load ptr, ptr %6, align 8
  call void @codeclosure(ptr noundef %40, ptr noundef %41)
  %42 = load ptr, ptr %5, align 8
  call void @close_func(ptr noundef %42)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @suffixedexp(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.expdesc, align 8
  %7 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 5
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = load ptr, ptr %4, align 8
  call void @primaryexp(ptr noundef %11, ptr noundef %12)
  br label %13

13:                                               ; preds = %40, %2
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 3
  %16 = getelementptr inbounds %struct.Token, ptr %15, i32 0, i32 0
  %17 = load i32, ptr %16, align 8
  switch i32 %17, label %39 [
    i32 46, label %18
    i32 91, label %21
    i32 58, label %27
    i32 40, label %34
    i32 292, label %34
    i32 123, label %34
  ]

18:                                               ; preds = %13
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %4, align 8
  call void @fieldsel(ptr noundef %19, ptr noundef %20)
  br label %40

21:                                               ; preds = %13
  %22 = load ptr, ptr %5, align 8
  %23 = load ptr, ptr %4, align 8
  call void @luaK_exp2anyregup(ptr noundef %22, ptr noundef %23)
  %24 = load ptr, ptr %3, align 8
  call void @yindex(ptr noundef %24, ptr noundef %6)
  %25 = load ptr, ptr %5, align 8
  %26 = load ptr, ptr %4, align 8
  call void @luaK_indexed(ptr noundef %25, ptr noundef %26, ptr noundef %6)
  br label %40

27:                                               ; preds = %13
  %28 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %28)
  %29 = load ptr, ptr %3, align 8
  call void @codename(ptr noundef %29, ptr noundef %7)
  %30 = load ptr, ptr %5, align 8
  %31 = load ptr, ptr %4, align 8
  call void @luaK_self(ptr noundef %30, ptr noundef %31, ptr noundef %7)
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %4, align 8
  call void @funcargs(ptr noundef %32, ptr noundef %33)
  br label %40

34:                                               ; preds = %13, %13, %13
  %35 = load ptr, ptr %5, align 8
  %36 = load ptr, ptr %4, align 8
  call void @luaK_exp2nextreg(ptr noundef %35, ptr noundef %36)
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %4, align 8
  call void @funcargs(ptr noundef %37, ptr noundef %38)
  br label %40

39:                                               ; preds = %13
  ret void

40:                                               ; preds = %34, %27, %21, %18
  br label %13
}

declare hidden i32 @luaK_code(ptr noundef, i32 noundef) #1

declare hidden void @luaK_reserveregs(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @closelistfield(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.ConsControl, ptr %5, i32 0, i32 0
  %7 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %10, label %11

10:                                               ; preds = %2
  br label %44

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.ConsControl, ptr %13, i32 0, i32 0
  call void @luaK_exp2nextreg(ptr noundef %12, ptr noundef %14)
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.ConsControl, ptr %15, i32 0, i32 0
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 0
  store i32 0, ptr %17, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.ConsControl, ptr %18, i32 0, i32 4
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 50
  br i1 %21, label %22, label %44

22:                                               ; preds = %11
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.ConsControl, ptr %24, i32 0, i32 1
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %struct.expdesc, ptr %26, i32 0, i32 1
  %28 = load i32, ptr %27, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.ConsControl, ptr %29, i32 0, i32 3
  %31 = load i32, ptr %30, align 4
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.ConsControl, ptr %32, i32 0, i32 4
  %34 = load i32, ptr %33, align 8
  call void @luaK_setlist(ptr noundef %23, i32 noundef %28, i32 noundef %31, i32 noundef %34)
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.ConsControl, ptr %35, i32 0, i32 4
  %37 = load i32, ptr %36, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.ConsControl, ptr %38, i32 0, i32 3
  %40 = load i32, ptr %39, align 4
  %41 = add nsw i32 %40, %37
  store i32 %41, ptr %39, align 4
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.ConsControl, ptr %42, i32 0, i32 4
  store i32 0, ptr %43, align 8
  br label %44

44:                                               ; preds = %10, %22, %11
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @field(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 3
  %7 = getelementptr inbounds %struct.Token, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  switch i32 %8, label %23 [
    i32 291, label %9
    i32 91, label %20
  ]

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @luaX_lookahead(ptr noundef %10)
  %12 = icmp ne i32 %11, 61
  br i1 %12, label %13, label %16

13:                                               ; preds = %9
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %4, align 8
  call void @listfield(ptr noundef %14, ptr noundef %15)
  br label %19

16:                                               ; preds = %9
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  call void @recfield(ptr noundef %17, ptr noundef %18)
  br label %19

19:                                               ; preds = %16, %13
  br label %26

20:                                               ; preds = %2
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  call void @recfield(ptr noundef %21, ptr noundef %22)
  br label %26

23:                                               ; preds = %2
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %4, align 8
  call void @listfield(ptr noundef %24, ptr noundef %25)
  br label %26

26:                                               ; preds = %23, %20, %19
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @lastlistfield(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.ConsControl, ptr %5, i32 0, i32 4
  %7 = load i32, ptr %6, align 8
  %8 = icmp eq i32 %7, 0
  br i1 %8, label %9, label %10

9:                                                ; preds = %2
  br label %70

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.ConsControl, ptr %11, i32 0, i32 0
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 0
  %14 = load i32, ptr %13, align 8
  %15 = icmp eq i32 %14, 18
  br i1 %15, label %22, label %16

16:                                               ; preds = %10
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.ConsControl, ptr %17, i32 0, i32 0
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 19
  br i1 %21, label %22, label %39

22:                                               ; preds = %16, %10
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.ConsControl, ptr %24, i32 0, i32 0
  call void @luaK_setreturns(ptr noundef %23, ptr noundef %25, i32 noundef -1)
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.ConsControl, ptr %27, i32 0, i32 1
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.expdesc, ptr %29, i32 0, i32 1
  %31 = load i32, ptr %30, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.ConsControl, ptr %32, i32 0, i32 3
  %34 = load i32, ptr %33, align 4
  call void @luaK_setlist(ptr noundef %26, i32 noundef %31, i32 noundef %34, i32 noundef -1)
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.ConsControl, ptr %35, i32 0, i32 3
  %37 = load i32, ptr %36, align 4
  %38 = add nsw i32 %37, -1
  store i32 %38, ptr %36, align 4
  br label %62

39:                                               ; preds = %16
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.ConsControl, ptr %40, i32 0, i32 0
  %42 = getelementptr inbounds %struct.expdesc, ptr %41, i32 0, i32 0
  %43 = load i32, ptr %42, align 8
  %44 = icmp ne i32 %43, 0
  br i1 %44, label %45, label %49

45:                                               ; preds = %39
  %46 = load ptr, ptr %3, align 8
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.ConsControl, ptr %47, i32 0, i32 0
  call void @luaK_exp2nextreg(ptr noundef %46, ptr noundef %48)
  br label %49

49:                                               ; preds = %45, %39
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.ConsControl, ptr %51, i32 0, i32 1
  %53 = load ptr, ptr %52, align 8
  %54 = getelementptr inbounds %struct.expdesc, ptr %53, i32 0, i32 1
  %55 = load i32, ptr %54, align 8
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.ConsControl, ptr %56, i32 0, i32 3
  %58 = load i32, ptr %57, align 4
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.ConsControl, ptr %59, i32 0, i32 4
  %61 = load i32, ptr %60, align 8
  call void @luaK_setlist(ptr noundef %50, i32 noundef %55, i32 noundef %58, i32 noundef %61)
  br label %62

62:                                               ; preds = %49, %22
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.ConsControl, ptr %63, i32 0, i32 4
  %65 = load i32, ptr %64, align 8
  %66 = load ptr, ptr %4, align 8
  %67 = getelementptr inbounds %struct.ConsControl, ptr %66, i32 0, i32 3
  %68 = load i32, ptr %67, align 4
  %69 = add nsw i32 %68, %65
  store i32 %69, ptr %67, align 4
  br label %70

70:                                               ; preds = %62, %9
  ret void
}

declare hidden void @luaK_settablesize(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef) #1

declare hidden void @luaK_exp2nextreg(ptr noundef, ptr noundef) #1

declare hidden void @luaK_setlist(ptr noundef, i32 noundef, i32 noundef, i32 noundef) #1

declare hidden i32 @luaX_lookahead(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @listfield(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.ConsControl, ptr %6, i32 0, i32 0
  call void @expr(ptr noundef %5, ptr noundef %7)
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.ConsControl, ptr %8, i32 0, i32 4
  %10 = load i32, ptr %9, align 8
  %11 = add nsw i32 %10, 1
  store i32 %11, ptr %9, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @recfield(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.expdesc, align 8
  %8 = alloca %struct.expdesc, align 8
  %9 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 5
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.FuncState, ptr %15, i32 0, i32 15
  %17 = load i8, ptr %16, align 4
  %18 = zext i8 %17 to i32
  store i32 %18, ptr %6, align 4
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 3
  %21 = getelementptr inbounds %struct.Token, ptr %20, i32 0, i32 0
  %22 = load i32, ptr %21, align 8
  %23 = icmp eq i32 %22, 291
  br i1 %23, label %24, label %30

24:                                               ; preds = %2
  %25 = load ptr, ptr %5, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.ConsControl, ptr %26, i32 0, i32 2
  %28 = load i32, ptr %27, align 8
  call void @checklimit(ptr noundef %25, i32 noundef %28, i32 noundef 2147483647, ptr noundef @.str.6)
  %29 = load ptr, ptr %3, align 8
  call void @codename(ptr noundef %29, ptr noundef %8)
  br label %32

30:                                               ; preds = %2
  %31 = load ptr, ptr %3, align 8
  call void @yindex(ptr noundef %31, ptr noundef %8)
  br label %32

32:                                               ; preds = %30, %24
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.ConsControl, ptr %33, i32 0, i32 2
  %35 = load i32, ptr %34, align 8
  %36 = add nsw i32 %35, 1
  store i32 %36, ptr %34, align 8
  %37 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %37, i32 noundef 61)
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.ConsControl, ptr %38, i32 0, i32 1
  %40 = load ptr, ptr %39, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7, ptr align 8 %40, i64 24, i1 false)
  %41 = load ptr, ptr %5, align 8
  call void @luaK_indexed(ptr noundef %41, ptr noundef %7, ptr noundef %8)
  %42 = load ptr, ptr %3, align 8
  call void @expr(ptr noundef %42, ptr noundef %9)
  %43 = load ptr, ptr %5, align 8
  call void @luaK_storevar(ptr noundef %43, ptr noundef %7, ptr noundef %9)
  %44 = load i32, ptr %6, align 4
  %45 = trunc i32 %44 to i8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.FuncState, ptr %46, i32 0, i32 15
  store i8 %45, ptr %47, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codename(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @str_checkname(ptr noundef %6)
  call void @codestring(ptr noundef %5, ptr noundef %7)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @yindex(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %5)
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @expr(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 5
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  call void @luaK_exp2val(ptr noundef %10, ptr noundef %11)
  %12 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %12, i32 noundef 93)
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #4

declare hidden void @luaK_indexed(ptr noundef, ptr noundef, ptr noundef) #1

declare hidden void @luaK_storevar(ptr noundef, ptr noundef, ptr noundef) #1

declare hidden void @luaK_exp2val(ptr noundef, ptr noundef) #1

declare hidden void @luaK_setreturns(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @addprototype(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 6
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %5, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %6, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 8
  %19 = load i32, ptr %18, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.Proto, ptr %20, i32 0, i32 10
  %22 = load i32, ptr %21, align 8
  %23 = icmp sge i32 %19, %22
  br i1 %23, label %24, label %55

24:                                               ; preds = %1
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 10
  %27 = load i32, ptr %26, align 8
  store i32 %27, ptr %7, align 4
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.Proto, ptr %29, i32 0, i32 17
  %31 = load ptr, ptr %30, align 8
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds %struct.FuncState, ptr %32, i32 0, i32 8
  %34 = load i32, ptr %33, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.Proto, ptr %35, i32 0, i32 10
  %37 = call ptr @luaM_growaux_(ptr noundef %28, ptr noundef %31, i32 noundef %34, ptr noundef %36, i32 noundef 8, i32 noundef 131071, ptr noundef @.str.8)
  %38 = load ptr, ptr %6, align 8
  %39 = getelementptr inbounds %struct.Proto, ptr %38, i32 0, i32 17
  store ptr %37, ptr %39, align 8
  br label %40

40:                                               ; preds = %46, %24
  %41 = load i32, ptr %7, align 4
  %42 = load ptr, ptr %6, align 8
  %43 = getelementptr inbounds %struct.Proto, ptr %42, i32 0, i32 10
  %44 = load i32, ptr %43, align 8
  %45 = icmp slt i32 %41, %44
  br i1 %45, label %46, label %54

46:                                               ; preds = %40
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 17
  %49 = load ptr, ptr %48, align 8
  %50 = load i32, ptr %7, align 4
  %51 = add nsw i32 %50, 1
  store i32 %51, ptr %7, align 4
  %52 = sext i32 %50 to i64
  %53 = getelementptr inbounds ptr, ptr %49, i64 %52
  store ptr null, ptr %53, align 8
  br label %40, !llvm.loop !16

54:                                               ; preds = %40
  br label %55

55:                                               ; preds = %54, %1
  %56 = load ptr, ptr %4, align 8
  %57 = call ptr @luaF_newproto(ptr noundef %56)
  store ptr %57, ptr %3, align 8
  %58 = load ptr, ptr %6, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 17
  %60 = load ptr, ptr %59, align 8
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %struct.FuncState, ptr %61, i32 0, i32 8
  %63 = load i32, ptr %62, align 8
  %64 = add nsw i32 %63, 1
  store i32 %64, ptr %62, align 8
  %65 = sext i32 %63 to i64
  %66 = getelementptr inbounds ptr, ptr %60, i64 %65
  store ptr %57, ptr %66, align 8
  %67 = load ptr, ptr %6, align 8
  %68 = getelementptr inbounds %struct.Proto, ptr %67, i32 0, i32 2
  %69 = load i8, ptr %68, align 1
  %70 = zext i8 %69 to i32
  %71 = and i32 %70, 32
  %72 = icmp ne i32 %71, 0
  br i1 %72, label %73, label %84

73:                                               ; preds = %55
  %74 = load ptr, ptr %3, align 8
  %75 = getelementptr inbounds %struct.Proto, ptr %74, i32 0, i32 2
  %76 = load i8, ptr %75, align 1
  %77 = zext i8 %76 to i32
  %78 = and i32 %77, 24
  %79 = icmp ne i32 %78, 0
  br i1 %79, label %80, label %84

80:                                               ; preds = %73
  %81 = load ptr, ptr %4, align 8
  %82 = load ptr, ptr %6, align 8
  %83 = load ptr, ptr %3, align 8
  call void @luaC_barrier_(ptr noundef %81, ptr noundef %82, ptr noundef %83)
  br label %85

84:                                               ; preds = %73, %55
  br label %85

85:                                               ; preds = %84, %80
  %86 = load ptr, ptr %3, align 8
  ret ptr %86
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @new_localvar(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 6
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 5
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 10
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %7, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.Dyndata, ptr %19, i32 0, i32 0
  %21 = getelementptr inbounds %struct.anon.8, ptr %20, i32 0, i32 1
  %22 = load i32, ptr %21, align 8
  %23 = add nsw i32 %22, 1
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.FuncState, ptr %24, i32 0, i32 10
  %26 = load i32, ptr %25, align 8
  %27 = sub nsw i32 %23, %26
  call void @checklimit(ptr noundef %18, i32 noundef %27, i32 noundef 200, ptr noundef @.str.9)
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds %struct.Dyndata, ptr %29, i32 0, i32 0
  %31 = getelementptr inbounds %struct.anon.8, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.Dyndata, ptr %33, i32 0, i32 0
  %35 = getelementptr inbounds %struct.anon.8, ptr %34, i32 0, i32 1
  %36 = load i32, ptr %35, align 8
  %37 = add nsw i32 %36, 1
  %38 = load ptr, ptr %7, align 8
  %39 = getelementptr inbounds %struct.Dyndata, ptr %38, i32 0, i32 0
  %40 = getelementptr inbounds %struct.anon.8, ptr %39, i32 0, i32 2
  %41 = call ptr @luaM_growaux_(ptr noundef %28, ptr noundef %32, i32 noundef %37, ptr noundef %40, i32 noundef 24, i32 noundef 65535, ptr noundef @.str.9)
  %42 = load ptr, ptr %7, align 8
  %43 = getelementptr inbounds %struct.Dyndata, ptr %42, i32 0, i32 0
  %44 = getelementptr inbounds %struct.anon.8, ptr %43, i32 0, i32 0
  store ptr %41, ptr %44, align 8
  %45 = load ptr, ptr %7, align 8
  %46 = getelementptr inbounds %struct.Dyndata, ptr %45, i32 0, i32 0
  %47 = getelementptr inbounds %struct.anon.8, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  %49 = load ptr, ptr %7, align 8
  %50 = getelementptr inbounds %struct.Dyndata, ptr %49, i32 0, i32 0
  %51 = getelementptr inbounds %struct.anon.8, ptr %50, i32 0, i32 1
  %52 = load i32, ptr %51, align 8
  %53 = add nsw i32 %52, 1
  store i32 %53, ptr %51, align 8
  %54 = sext i32 %52 to i64
  %55 = getelementptr inbounds %union.Vardesc, ptr %48, i64 %54
  store ptr %55, ptr %8, align 8
  %56 = load ptr, ptr %8, align 8
  %57 = getelementptr inbounds %struct.anon.9, ptr %56, i32 0, i32 2
  store i8 0, ptr %57, align 1
  %58 = load ptr, ptr %4, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = getelementptr inbounds %struct.anon.9, ptr %59, i32 0, i32 5
  store ptr %58, ptr %60, align 8
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.Dyndata, ptr %61, i32 0, i32 0
  %63 = getelementptr inbounds %struct.anon.8, ptr %62, i32 0, i32 1
  %64 = load i32, ptr %63, align 8
  %65 = sub nsw i32 %64, 1
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds %struct.FuncState, ptr %66, i32 0, i32 10
  %68 = load i32, ptr %67, align 8
  %69 = sub nsw i32 %65, %68
  ret i32 %69
}

declare hidden ptr @luaX_newstring(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @adjustlocalvars(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @luaY_nvarstack(ptr noundef %13)
  store i32 %14, ptr %6, align 4
  store i32 0, ptr %7, align 4
  br label %15

15:                                               ; preds = %42, %2
  %16 = load i32, ptr %7, align 4
  %17 = load i32, ptr %4, align 4
  %18 = icmp slt i32 %16, %17
  br i1 %18, label %19, label %45

19:                                               ; preds = %15
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.FuncState, ptr %20, i32 0, i32 13
  %22 = load i8, ptr %21, align 2
  %23 = add i8 %22, 1
  store i8 %23, ptr %21, align 2
  %24 = zext i8 %22 to i32
  store i32 %24, ptr %8, align 4
  %25 = load ptr, ptr %5, align 8
  %26 = load i32, ptr %8, align 4
  %27 = call ptr @getlocalvardesc(ptr noundef %25, i32 noundef %26)
  store ptr %27, ptr %9, align 8
  %28 = load i32, ptr %6, align 4
  %29 = add nsw i32 %28, 1
  store i32 %29, ptr %6, align 4
  %30 = trunc i32 %28 to i8
  %31 = load ptr, ptr %9, align 8
  %32 = getelementptr inbounds %struct.anon.9, ptr %31, i32 0, i32 3
  store i8 %30, ptr %32, align 2
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds %struct.anon.9, ptr %35, i32 0, i32 5
  %37 = load ptr, ptr %36, align 8
  %38 = call i32 @registerlocalvar(ptr noundef %33, ptr noundef %34, ptr noundef %37)
  %39 = trunc i32 %38 to i16
  %40 = load ptr, ptr %9, align 8
  %41 = getelementptr inbounds %struct.anon.9, ptr %40, i32 0, i32 4
  store i16 %39, ptr %41, align 4
  br label %42

42:                                               ; preds = %19
  %43 = load i32, ptr %7, align 4
  %44 = add nsw i32 %43, 1
  store i32 %44, ptr %7, align 4
  br label %15, !llvm.loop !17

45:                                               ; preds = %15
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @parlist(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %4, align 8
  store i32 0, ptr %5, align 4
  store i32 0, ptr %6, align 4
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 3
  %15 = getelementptr inbounds %struct.Token, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  %17 = icmp ne i32 %16, 41
  br i1 %17, label %18, label %46

18:                                               ; preds = %1
  br label %19

19:                                               ; preds = %43, %18
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 3
  %22 = getelementptr inbounds %struct.Token, ptr %21, i32 0, i32 0
  %23 = load i32, ptr %22, align 8
  switch i32 %23, label %33 [
    i32 291, label %24
    i32 280, label %31
  ]

24:                                               ; preds = %19
  %25 = load ptr, ptr %2, align 8
  %26 = load ptr, ptr %2, align 8
  %27 = call ptr @str_checkname(ptr noundef %26)
  %28 = call i32 @new_localvar(ptr noundef %25, ptr noundef %27)
  %29 = load i32, ptr %5, align 4
  %30 = add nsw i32 %29, 1
  store i32 %30, ptr %5, align 4
  br label %35

31:                                               ; preds = %19
  %32 = load ptr, ptr %2, align 8
  call void @luaX_next(ptr noundef %32)
  store i32 1, ptr %6, align 4
  br label %35

33:                                               ; preds = %19
  %34 = load ptr, ptr %2, align 8
  call void @luaX_syntaxerror(ptr noundef %34, ptr noundef @.str.10) #6
  unreachable

35:                                               ; preds = %31, %24
  br label %36

36:                                               ; preds = %35
  %37 = load i32, ptr %6, align 4
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %43, label %39

39:                                               ; preds = %36
  %40 = load ptr, ptr %2, align 8
  %41 = call i32 @testnext(ptr noundef %40, i32 noundef 44)
  %42 = icmp ne i32 %41, 0
  br label %43

43:                                               ; preds = %39, %36
  %44 = phi i1 [ false, %36 ], [ %42, %39 ]
  br i1 %44, label %19, label %45, !llvm.loop !18

45:                                               ; preds = %43
  br label %46

46:                                               ; preds = %45, %1
  %47 = load ptr, ptr %2, align 8
  %48 = load i32, ptr %5, align 4
  call void @adjustlocalvars(ptr noundef %47, i32 noundef %48)
  %49 = load ptr, ptr %3, align 8
  %50 = getelementptr inbounds %struct.FuncState, ptr %49, i32 0, i32 13
  %51 = load i8, ptr %50, align 2
  %52 = load ptr, ptr %4, align 8
  %53 = getelementptr inbounds %struct.Proto, ptr %52, i32 0, i32 3
  store i8 %51, ptr %53, align 2
  %54 = load i32, ptr %6, align 4
  %55 = icmp ne i32 %54, 0
  br i1 %55, label %56, label %62

56:                                               ; preds = %46
  %57 = load ptr, ptr %3, align 8
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 3
  %60 = load i8, ptr %59, align 2
  %61 = zext i8 %60 to i32
  call void @setvararg(ptr noundef %57, i32 noundef %61)
  br label %62

62:                                               ; preds = %56, %46
  %63 = load ptr, ptr %3, align 8
  %64 = load ptr, ptr %3, align 8
  %65 = getelementptr inbounds %struct.FuncState, ptr %64, i32 0, i32 13
  %66 = load i8, ptr %65, align 2
  %67 = zext i8 %66 to i32
  call void @luaK_reserveregs(ptr noundef %63, i32 noundef %67)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeclosure(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 5
  %8 = load ptr, ptr %7, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 1
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 8
  %15 = load i32, ptr %14, align 8
  %16 = sub nsw i32 %15, 1
  %17 = call i32 @luaK_codeABx(ptr noundef %12, i32 noundef 79, i32 noundef 0, i32 noundef %16)
  call void @init_exp(ptr noundef %11, i32 noundef 17, i32 noundef %17)
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %4, align 8
  call void @luaK_exp2nextreg(ptr noundef %18, ptr noundef %19)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @registerlocalvar(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 11
  %14 = load i32, ptr %13, align 4
  store i32 %14, ptr %8, align 4
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 21
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.FuncState, ptr %21, i32 0, i32 12
  %23 = load i16, ptr %22, align 8
  %24 = sext i16 %23 to i32
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 11
  %27 = call ptr @luaM_growaux_(ptr noundef %17, ptr noundef %20, i32 noundef %24, ptr noundef %26, i32 noundef 16, i32 noundef 32767, ptr noundef @.str.9)
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 21
  store ptr %27, ptr %29, align 8
  br label %30

30:                                               ; preds = %36, %3
  %31 = load i32, ptr %8, align 4
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.Proto, ptr %32, i32 0, i32 11
  %34 = load i32, ptr %33, align 4
  %35 = icmp slt i32 %31, %34
  br i1 %35, label %36, label %45

36:                                               ; preds = %30
  %37 = load ptr, ptr %7, align 8
  %38 = getelementptr inbounds %struct.Proto, ptr %37, i32 0, i32 21
  %39 = load ptr, ptr %38, align 8
  %40 = load i32, ptr %8, align 4
  %41 = add nsw i32 %40, 1
  store i32 %41, ptr %8, align 4
  %42 = sext i32 %40 to i64
  %43 = getelementptr inbounds %struct.LocVar, ptr %39, i64 %42
  %44 = getelementptr inbounds %struct.LocVar, ptr %43, i32 0, i32 0
  store ptr null, ptr %44, align 8
  br label %30, !llvm.loop !19

45:                                               ; preds = %30
  %46 = load ptr, ptr %6, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 21
  %49 = load ptr, ptr %48, align 8
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %struct.FuncState, ptr %50, i32 0, i32 12
  %52 = load i16, ptr %51, align 8
  %53 = sext i16 %52 to i64
  %54 = getelementptr inbounds %struct.LocVar, ptr %49, i64 %53
  %55 = getelementptr inbounds %struct.LocVar, ptr %54, i32 0, i32 0
  store ptr %46, ptr %55, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.FuncState, ptr %56, i32 0, i32 4
  %58 = load i32, ptr %57, align 8
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.Proto, ptr %59, i32 0, i32 21
  %61 = load ptr, ptr %60, align 8
  %62 = load ptr, ptr %5, align 8
  %63 = getelementptr inbounds %struct.FuncState, ptr %62, i32 0, i32 12
  %64 = load i16, ptr %63, align 8
  %65 = sext i16 %64 to i64
  %66 = getelementptr inbounds %struct.LocVar, ptr %61, i64 %65
  %67 = getelementptr inbounds %struct.LocVar, ptr %66, i32 0, i32 1
  store i32 %58, ptr %67, align 8
  %68 = load ptr, ptr %7, align 8
  %69 = getelementptr inbounds %struct.Proto, ptr %68, i32 0, i32 2
  %70 = load i8, ptr %69, align 1
  %71 = zext i8 %70 to i32
  %72 = and i32 %71, 32
  %73 = icmp ne i32 %72, 0
  br i1 %73, label %74, label %87

74:                                               ; preds = %45
  %75 = load ptr, ptr %6, align 8
  %76 = getelementptr inbounds %struct.TString, ptr %75, i32 0, i32 2
  %77 = load i8, ptr %76, align 1
  %78 = zext i8 %77 to i32
  %79 = and i32 %78, 24
  %80 = icmp ne i32 %79, 0
  br i1 %80, label %81, label %87

81:                                               ; preds = %74
  %82 = load ptr, ptr %4, align 8
  %83 = getelementptr inbounds %struct.LexState, ptr %82, i32 0, i32 6
  %84 = load ptr, ptr %83, align 8
  %85 = load ptr, ptr %7, align 8
  %86 = load ptr, ptr %6, align 8
  call void @luaC_barrier_(ptr noundef %84, ptr noundef %85, ptr noundef %86)
  br label %88

87:                                               ; preds = %74, %45
  br label %88

88:                                               ; preds = %87, %81
  %89 = load ptr, ptr %5, align 8
  %90 = getelementptr inbounds %struct.FuncState, ptr %89, i32 0, i32 12
  %91 = load i16, ptr %90, align 8
  %92 = add i16 %91, 1
  store i16 %92, ptr %90, align 8
  %93 = sext i16 %91 to i32
  ret i32 %93
}

declare hidden i32 @luaK_codeABx(ptr noundef, i32 noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @primaryexp(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 3
  %8 = getelementptr inbounds %struct.Token, ptr %7, i32 0, i32 0
  %9 = load i32, ptr %8, align 8
  switch i32 %9, label %26 [
    i32 40, label %10
    i32 291, label %23
  ]

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 4
  store i32 %13, ptr %5, align 4
  %14 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %14)
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %4, align 8
  call void @expr(ptr noundef %15, ptr noundef %16)
  %17 = load ptr, ptr %3, align 8
  %18 = load i32, ptr %5, align 4
  call void @check_match(ptr noundef %17, i32 noundef 41, i32 noundef 40, i32 noundef %18)
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 5
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %4, align 8
  call void @luaK_dischargevars(ptr noundef %21, ptr noundef %22)
  br label %28

23:                                               ; preds = %2
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %4, align 8
  call void @singlevar(ptr noundef %24, ptr noundef %25)
  br label %28

26:                                               ; preds = %2
  %27 = load ptr, ptr %3, align 8
  call void @luaX_syntaxerror(ptr noundef %27, ptr noundef @.str.11) #6
  unreachable

28:                                               ; preds = %23, %10
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fieldsel(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  call void @luaK_exp2anyregup(ptr noundef %10, ptr noundef %11)
  %12 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %12)
  %13 = load ptr, ptr %3, align 8
  call void @codename(ptr noundef %13, ptr noundef %6)
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %4, align 8
  call void @luaK_indexed(ptr noundef %14, ptr noundef %15, ptr noundef %6)
  ret void
}

declare hidden void @luaK_exp2anyregup(ptr noundef, ptr noundef) #1

declare hidden void @luaK_self(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @funcargs(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.expdesc, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 1
  %15 = load i32, ptr %14, align 4
  store i32 %15, ptr %9, align 4
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 3
  %18 = getelementptr inbounds %struct.Token, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  switch i32 %19, label %53 [
    i32 40, label %20
    i32 123, label %45
    i32 292, label %47
  ]

20:                                               ; preds = %2
  %21 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %21)
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.LexState, ptr %22, i32 0, i32 3
  %24 = getelementptr inbounds %struct.Token, ptr %23, i32 0, i32 0
  %25 = load i32, ptr %24, align 8
  %26 = icmp eq i32 %25, 41
  br i1 %26, label %27, label %29

27:                                               ; preds = %20
  %28 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  store i32 0, ptr %28, align 8
  br label %42

29:                                               ; preds = %20
  %30 = load ptr, ptr %3, align 8
  %31 = call i32 @explist(ptr noundef %30, ptr noundef %6)
  %32 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %33 = load i32, ptr %32, align 8
  %34 = icmp eq i32 %33, 18
  br i1 %34, label %39, label %35

35:                                               ; preds = %29
  %36 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %37 = load i32, ptr %36, align 8
  %38 = icmp eq i32 %37, 19
  br i1 %38, label %39, label %41

39:                                               ; preds = %35, %29
  %40 = load ptr, ptr %5, align 8
  call void @luaK_setreturns(ptr noundef %40, ptr noundef %6, i32 noundef -1)
  br label %41

41:                                               ; preds = %39, %35
  br label %42

42:                                               ; preds = %41, %27
  %43 = load ptr, ptr %3, align 8
  %44 = load i32, ptr %9, align 4
  call void @check_match(ptr noundef %43, i32 noundef 41, i32 noundef 40, i32 noundef %44)
  br label %55

45:                                               ; preds = %2
  %46 = load ptr, ptr %3, align 8
  call void @constructor(ptr noundef %46, ptr noundef %6)
  br label %55

47:                                               ; preds = %2
  %48 = load ptr, ptr %3, align 8
  %49 = getelementptr inbounds %struct.LexState, ptr %48, i32 0, i32 3
  %50 = getelementptr inbounds %struct.Token, ptr %49, i32 0, i32 1
  %51 = load ptr, ptr %50, align 8
  call void @codestring(ptr noundef %6, ptr noundef %51)
  %52 = load ptr, ptr %3, align 8
  call void @luaX_next(ptr noundef %52)
  br label %55

53:                                               ; preds = %2
  %54 = load ptr, ptr %3, align 8
  call void @luaX_syntaxerror(ptr noundef %54, ptr noundef @.str.12) #6
  unreachable

55:                                               ; preds = %47, %45, %42
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.expdesc, ptr %56, i32 0, i32 1
  %58 = load i32, ptr %57, align 8
  store i32 %58, ptr %7, align 4
  %59 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %60 = load i32, ptr %59, align 8
  %61 = icmp eq i32 %60, 18
  br i1 %61, label %66, label %62

62:                                               ; preds = %55
  %63 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %64 = load i32, ptr %63, align 8
  %65 = icmp eq i32 %64, 19
  br i1 %65, label %66, label %67

66:                                               ; preds = %62, %55
  store i32 -1, ptr %8, align 4
  br label %81

67:                                               ; preds = %62
  %68 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %69 = load i32, ptr %68, align 8
  %70 = icmp ne i32 %69, 0
  br i1 %70, label %71, label %73

71:                                               ; preds = %67
  %72 = load ptr, ptr %5, align 8
  call void @luaK_exp2nextreg(ptr noundef %72, ptr noundef %6)
  br label %73

73:                                               ; preds = %71, %67
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.FuncState, ptr %74, i32 0, i32 15
  %76 = load i8, ptr %75, align 4
  %77 = zext i8 %76 to i32
  %78 = load i32, ptr %7, align 4
  %79 = add nsw i32 %78, 1
  %80 = sub nsw i32 %77, %79
  store i32 %80, ptr %8, align 4
  br label %81

81:                                               ; preds = %73, %66
  %82 = load ptr, ptr %4, align 8
  %83 = load ptr, ptr %5, align 8
  %84 = load i32, ptr %7, align 4
  %85 = load i32, ptr %8, align 4
  %86 = add nsw i32 %85, 1
  %87 = call i32 @luaK_codeABCk(ptr noundef %83, i32 noundef 68, i32 noundef %84, i32 noundef %86, i32 noundef 2, i32 noundef 0)
  call void @init_exp(ptr noundef %82, i32 noundef 18, i32 noundef %87)
  %88 = load ptr, ptr %5, align 8
  %89 = load i32, ptr %9, align 4
  call void @luaK_fixline(ptr noundef %88, i32 noundef %89)
  %90 = load i32, ptr %7, align 4
  %91 = add nsw i32 %90, 1
  %92 = trunc i32 %91 to i8
  %93 = load ptr, ptr %5, align 8
  %94 = getelementptr inbounds %struct.FuncState, ptr %93, i32 0, i32 15
  store i8 %92, ptr %94, align 4
  ret void
}

declare hidden void @luaK_dischargevars(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @singlevar(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @str_checkname(ptr noundef %8)
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %4, align 8
  call void @singlevaraux(ptr noundef %13, ptr noundef %14, ptr noundef %15, i32 noundef 1)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %31

20:                                               ; preds = %2
  %21 = load ptr, ptr %6, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.LexState, ptr %22, i32 0, i32 12
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %4, align 8
  call void @singlevaraux(ptr noundef %21, ptr noundef %24, ptr noundef %25, i32 noundef 1)
  %26 = load ptr, ptr %6, align 8
  %27 = load ptr, ptr %4, align 8
  call void @luaK_exp2anyregup(ptr noundef %26, ptr noundef %27)
  %28 = load ptr, ptr %5, align 8
  call void @codestring(ptr noundef %7, ptr noundef %28)
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %4, align 8
  call void @luaK_indexed(ptr noundef %29, ptr noundef %30, ptr noundef %7)
  br label %31

31:                                               ; preds = %20, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @singlevaraux(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %15

13:                                               ; preds = %4
  %14 = load ptr, ptr %7, align 8
  call void @init_exp(ptr noundef %14, i32 noundef 0, i32 noundef 0)
  br label %68

15:                                               ; preds = %4
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %6, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = call i32 @searchvar(ptr noundef %16, ptr noundef %17, ptr noundef %18)
  store i32 %19, ptr %9, align 4
  %20 = load i32, ptr %9, align 4
  %21 = icmp sge i32 %20, 0
  br i1 %21, label %22, label %36

22:                                               ; preds = %15
  %23 = load i32, ptr %9, align 4
  %24 = icmp eq i32 %23, 9
  br i1 %24, label %25, label %35

25:                                               ; preds = %22
  %26 = load i32, ptr %8, align 4
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %35, label %28

28:                                               ; preds = %25
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %7, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 1
  %32 = getelementptr inbounds %struct.anon.12, ptr %31, i32 0, i32 1
  %33 = load i16, ptr %32, align 2
  %34 = zext i16 %33 to i32
  call void @markupval(ptr noundef %29, i32 noundef %34)
  br label %35

35:                                               ; preds = %28, %25, %22
  br label %67

36:                                               ; preds = %15
  %37 = load ptr, ptr %5, align 8
  %38 = load ptr, ptr %6, align 8
  %39 = call i32 @searchupvalue(ptr noundef %37, ptr noundef %38)
  store i32 %39, ptr %10, align 4
  %40 = load i32, ptr %10, align 4
  %41 = icmp slt i32 %40, 0
  br i1 %41, label %42, label %64

42:                                               ; preds = %36
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.FuncState, ptr %43, i32 0, i32 1
  %45 = load ptr, ptr %44, align 8
  %46 = load ptr, ptr %6, align 8
  %47 = load ptr, ptr %7, align 8
  call void @singlevaraux(ptr noundef %45, ptr noundef %46, ptr noundef %47, i32 noundef 0)
  %48 = load ptr, ptr %7, align 8
  %49 = getelementptr inbounds %struct.expdesc, ptr %48, i32 0, i32 0
  %50 = load i32, ptr %49, align 8
  %51 = icmp eq i32 %50, 9
  br i1 %51, label %57, label %52

52:                                               ; preds = %42
  %53 = load ptr, ptr %7, align 8
  %54 = getelementptr inbounds %struct.expdesc, ptr %53, i32 0, i32 0
  %55 = load i32, ptr %54, align 8
  %56 = icmp eq i32 %55, 10
  br i1 %56, label %57, label %62

57:                                               ; preds = %52, %42
  %58 = load ptr, ptr %5, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = load ptr, ptr %7, align 8
  %61 = call i32 @newupvalue(ptr noundef %58, ptr noundef %59, ptr noundef %60)
  store i32 %61, ptr %10, align 4
  br label %63

62:                                               ; preds = %52
  br label %68

63:                                               ; preds = %57
  br label %64

64:                                               ; preds = %63, %36
  %65 = load ptr, ptr %7, align 8
  %66 = load i32, ptr %10, align 4
  call void @init_exp(ptr noundef %65, i32 noundef 10, i32 noundef %66)
  br label %67

67:                                               ; preds = %64, %35
  br label %68

68:                                               ; preds = %62, %67, %13
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searchvar(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 13
  %12 = load i8, ptr %11, align 2
  %13 = zext i8 %12 to i32
  %14 = sub nsw i32 %13, 1
  store i32 %14, ptr %8, align 4
  br label %15

15:                                               ; preds = %49, %3
  %16 = load i32, ptr %8, align 4
  %17 = icmp sge i32 %16, 0
  br i1 %17, label %18, label %52

18:                                               ; preds = %15
  %19 = load ptr, ptr %5, align 8
  %20 = load i32, ptr %8, align 4
  %21 = call ptr @getlocalvardesc(ptr noundef %19, i32 noundef %20)
  store ptr %21, ptr %9, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %9, align 8
  %24 = getelementptr inbounds %struct.anon.9, ptr %23, i32 0, i32 5
  %25 = load ptr, ptr %24, align 8
  %26 = icmp eq ptr %22, %25
  br i1 %26, label %27, label %48

27:                                               ; preds = %18
  %28 = load ptr, ptr %9, align 8
  %29 = getelementptr inbounds %struct.anon.9, ptr %28, i32 0, i32 2
  %30 = load i8, ptr %29, align 1
  %31 = zext i8 %30 to i32
  %32 = icmp eq i32 %31, 3
  br i1 %32, label %33, label %40

33:                                               ; preds = %27
  %34 = load ptr, ptr %7, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.FuncState, ptr %35, i32 0, i32 10
  %37 = load i32, ptr %36, align 8
  %38 = load i32, ptr %8, align 4
  %39 = add nsw i32 %37, %38
  call void @init_exp(ptr noundef %34, i32 noundef 11, i32 noundef %39)
  br label %44

40:                                               ; preds = %27
  %41 = load ptr, ptr %5, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = load i32, ptr %8, align 4
  call void @init_var(ptr noundef %41, ptr noundef %42, i32 noundef %43)
  br label %44

44:                                               ; preds = %40, %33
  %45 = load ptr, ptr %7, align 8
  %46 = getelementptr inbounds %struct.expdesc, ptr %45, i32 0, i32 0
  %47 = load i32, ptr %46, align 8
  store i32 %47, ptr %4, align 4
  br label %53

48:                                               ; preds = %18
  br label %49

49:                                               ; preds = %48
  %50 = load i32, ptr %8, align 4
  %51 = add nsw i32 %50, -1
  store i32 %51, ptr %8, align 4
  br label %15, !llvm.loop !20

52:                                               ; preds = %15
  store i32 -1, ptr %4, align 4
  br label %53

53:                                               ; preds = %52, %44
  %54 = load i32, ptr %4, align 4
  ret i32 %54
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @markupval(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.FuncState, ptr %6, i32 0, i32 3
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  br label %9

9:                                                ; preds = %16, %2
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.BlockCnt, ptr %10, i32 0, i32 3
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = load i32, ptr %4, align 4
  %15 = icmp sgt i32 %13, %14
  br i1 %15, label %16, label %20

16:                                               ; preds = %9
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.BlockCnt, ptr %17, i32 0, i32 0
  %19 = load ptr, ptr %18, align 8
  store ptr %19, ptr %5, align 8
  br label %9, !llvm.loop !21

20:                                               ; preds = %9
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.BlockCnt, ptr %21, i32 0, i32 4
  store i8 1, ptr %22, align 1
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.FuncState, ptr %23, i32 0, i32 17
  store i8 1, ptr %24, align 2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searchupvalue(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.Proto, ptr %10, i32 0, i32 18
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  store i32 0, ptr %6, align 4
  br label %13

13:                                               ; preds = %32, %2
  %14 = load i32, ptr %6, align 4
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.FuncState, ptr %15, i32 0, i32 14
  %17 = load i8, ptr %16, align 1
  %18 = zext i8 %17 to i32
  %19 = icmp slt i32 %14, %18
  br i1 %19, label %20, label %35

20:                                               ; preds = %13
  %21 = load ptr, ptr %7, align 8
  %22 = load i32, ptr %6, align 4
  %23 = sext i32 %22 to i64
  %24 = getelementptr inbounds %struct.Upvaldesc, ptr %21, i64 %23
  %25 = getelementptr inbounds %struct.Upvaldesc, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = icmp eq ptr %26, %27
  br i1 %28, label %29, label %31

29:                                               ; preds = %20
  %30 = load i32, ptr %6, align 4
  store i32 %30, ptr %3, align 4
  br label %36

31:                                               ; preds = %20
  br label %32

32:                                               ; preds = %31
  %33 = load i32, ptr %6, align 4
  %34 = add nsw i32 %33, 1
  store i32 %34, ptr %6, align 4
  br label %13, !llvm.loop !22

35:                                               ; preds = %13
  store i32 -1, ptr %3, align 4
  br label %36

36:                                               ; preds = %35, %29
  %37 = load i32, ptr %3, align 4
  ret i32 %37
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @newupvalue(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @allocupvalue(ptr noundef %9)
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.FuncState, ptr %11, i32 0, i32 1
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %8, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  %17 = icmp eq i32 %16, 9
  br i1 %17, label %18, label %38

18:                                               ; preds = %3
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.Upvaldesc, ptr %19, i32 0, i32 1
  store i8 1, ptr %20, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 1
  %23 = getelementptr inbounds %struct.anon.12, ptr %22, i32 0, i32 0
  %24 = load i8, ptr %23, align 8
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.Upvaldesc, ptr %25, i32 0, i32 2
  store i8 %24, ptr %26, align 1
  %27 = load ptr, ptr %8, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 1
  %30 = getelementptr inbounds %struct.anon.12, ptr %29, i32 0, i32 1
  %31 = load i16, ptr %30, align 2
  %32 = zext i16 %31 to i32
  %33 = call ptr @getlocalvardesc(ptr noundef %27, i32 noundef %32)
  %34 = getelementptr inbounds %struct.anon.9, ptr %33, i32 0, i32 2
  %35 = load i8, ptr %34, align 1
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.Upvaldesc, ptr %36, i32 0, i32 3
  store i8 %35, ptr %37, align 2
  br label %61

38:                                               ; preds = %3
  %39 = load ptr, ptr %7, align 8
  %40 = getelementptr inbounds %struct.Upvaldesc, ptr %39, i32 0, i32 1
  store i8 0, ptr %40, align 8
  %41 = load ptr, ptr %6, align 8
  %42 = getelementptr inbounds %struct.expdesc, ptr %41, i32 0, i32 1
  %43 = load i32, ptr %42, align 8
  %44 = trunc i32 %43 to i8
  %45 = load ptr, ptr %7, align 8
  %46 = getelementptr inbounds %struct.Upvaldesc, ptr %45, i32 0, i32 2
  store i8 %44, ptr %46, align 1
  %47 = load ptr, ptr %8, align 8
  %48 = getelementptr inbounds %struct.FuncState, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.Proto, ptr %49, i32 0, i32 18
  %51 = load ptr, ptr %50, align 8
  %52 = load ptr, ptr %6, align 8
  %53 = getelementptr inbounds %struct.expdesc, ptr %52, i32 0, i32 1
  %54 = load i32, ptr %53, align 8
  %55 = sext i32 %54 to i64
  %56 = getelementptr inbounds %struct.Upvaldesc, ptr %51, i64 %55
  %57 = getelementptr inbounds %struct.Upvaldesc, ptr %56, i32 0, i32 3
  %58 = load i8, ptr %57, align 2
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.Upvaldesc, ptr %59, i32 0, i32 3
  store i8 %58, ptr %60, align 2
  br label %61

61:                                               ; preds = %38, %18
  %62 = load ptr, ptr %5, align 8
  %63 = load ptr, ptr %7, align 8
  %64 = getelementptr inbounds %struct.Upvaldesc, ptr %63, i32 0, i32 0
  store ptr %62, ptr %64, align 8
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.FuncState, ptr %65, i32 0, i32 0
  %67 = load ptr, ptr %66, align 8
  %68 = getelementptr inbounds %struct.Proto, ptr %67, i32 0, i32 2
  %69 = load i8, ptr %68, align 1
  %70 = zext i8 %69 to i32
  %71 = and i32 %70, 32
  %72 = icmp ne i32 %71, 0
  br i1 %72, label %73, label %90

73:                                               ; preds = %61
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.TString, ptr %74, i32 0, i32 2
  %76 = load i8, ptr %75, align 1
  %77 = zext i8 %76 to i32
  %78 = and i32 %77, 24
  %79 = icmp ne i32 %78, 0
  br i1 %79, label %80, label %90

80:                                               ; preds = %73
  %81 = load ptr, ptr %4, align 8
  %82 = getelementptr inbounds %struct.FuncState, ptr %81, i32 0, i32 2
  %83 = load ptr, ptr %82, align 8
  %84 = getelementptr inbounds %struct.LexState, ptr %83, i32 0, i32 6
  %85 = load ptr, ptr %84, align 8
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.FuncState, ptr %86, i32 0, i32 0
  %88 = load ptr, ptr %87, align 8
  %89 = load ptr, ptr %5, align 8
  call void @luaC_barrier_(ptr noundef %85, ptr noundef %88, ptr noundef %89)
  br label %91

90:                                               ; preds = %73, %61
  br label %91

91:                                               ; preds = %90, %80
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.FuncState, ptr %92, i32 0, i32 14
  %94 = load i8, ptr %93, align 1
  %95 = zext i8 %94 to i32
  %96 = sub nsw i32 %95, 1
  ret i32 %96
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @init_var(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %5, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 2
  store i32 -1, ptr %8, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 3
  store i32 -1, ptr %10, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 0
  store i32 9, ptr %12, align 8
  %13 = load i32, ptr %6, align 4
  %14 = trunc i32 %13 to i16
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.expdesc, ptr %15, i32 0, i32 1
  %17 = getelementptr inbounds %struct.anon.12, ptr %16, i32 0, i32 1
  store i16 %14, ptr %17, align 2
  %18 = load ptr, ptr %4, align 8
  %19 = load i32, ptr %6, align 4
  %20 = call ptr @getlocalvardesc(ptr noundef %18, i32 noundef %19)
  %21 = getelementptr inbounds %struct.anon.9, ptr %20, i32 0, i32 3
  %22 = load i8, ptr %21, align 2
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.expdesc, ptr %23, i32 0, i32 1
  %25 = getelementptr inbounds %struct.anon.12, ptr %24, i32 0, i32 0
  store i8 %22, ptr %25, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @explist(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 1, ptr %5, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @expr(ptr noundef %6, ptr noundef %7)
  br label %8

8:                                                ; preds = %12, %2
  %9 = load ptr, ptr %3, align 8
  %10 = call i32 @testnext(ptr noundef %9, i32 noundef 44)
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %12, label %21

12:                                               ; preds = %8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 5
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %4, align 8
  call void @luaK_exp2nextreg(ptr noundef %15, ptr noundef %16)
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  call void @expr(ptr noundef %17, ptr noundef %18)
  %19 = load i32, ptr %5, align 4
  %20 = add nsw i32 %19, 1
  store i32 %20, ptr %5, align 4
  br label %8, !llvm.loop !23

21:                                               ; preds = %8
  %22 = load i32, ptr %5, align 4
  ret i32 %22
}

declare hidden void @luaK_fixline(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @newlabelentry(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.Labellist, ptr %12, i32 0, i32 1
  %14 = load i32, ptr %13, align 8
  store i32 %14, ptr %11, align 4
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = getelementptr inbounds %struct.Labellist, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %11, align 4
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.Labellist, ptr %22, i32 0, i32 2
  %24 = call ptr @luaM_growaux_(ptr noundef %17, ptr noundef %20, i32 noundef %21, ptr noundef %23, i32 noundef 24, i32 noundef 32767, ptr noundef @.str.13)
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.Labellist, ptr %25, i32 0, i32 0
  store ptr %24, ptr %26, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.Labellist, ptr %28, i32 0, i32 0
  %30 = load ptr, ptr %29, align 8
  %31 = load i32, ptr %11, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds %struct.Labeldesc, ptr %30, i64 %32
  %34 = getelementptr inbounds %struct.Labeldesc, ptr %33, i32 0, i32 0
  store ptr %27, ptr %34, align 8
  %35 = load i32, ptr %9, align 4
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.Labellist, ptr %36, i32 0, i32 0
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %11, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds %struct.Labeldesc, ptr %38, i64 %40
  %42 = getelementptr inbounds %struct.Labeldesc, ptr %41, i32 0, i32 2
  store i32 %35, ptr %42, align 4
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.LexState, ptr %43, i32 0, i32 5
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %struct.FuncState, ptr %45, i32 0, i32 13
  %47 = load i8, ptr %46, align 2
  %48 = load ptr, ptr %7, align 8
  %49 = getelementptr inbounds %struct.Labellist, ptr %48, i32 0, i32 0
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %11, align 4
  %52 = sext i32 %51 to i64
  %53 = getelementptr inbounds %struct.Labeldesc, ptr %50, i64 %52
  %54 = getelementptr inbounds %struct.Labeldesc, ptr %53, i32 0, i32 3
  store i8 %47, ptr %54, align 8
  %55 = load ptr, ptr %7, align 8
  %56 = getelementptr inbounds %struct.Labellist, ptr %55, i32 0, i32 0
  %57 = load ptr, ptr %56, align 8
  %58 = load i32, ptr %11, align 4
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds %struct.Labeldesc, ptr %57, i64 %59
  %61 = getelementptr inbounds %struct.Labeldesc, ptr %60, i32 0, i32 4
  store i8 0, ptr %61, align 1
  %62 = load i32, ptr %10, align 4
  %63 = load ptr, ptr %7, align 8
  %64 = getelementptr inbounds %struct.Labellist, ptr %63, i32 0, i32 0
  %65 = load ptr, ptr %64, align 8
  %66 = load i32, ptr %11, align 4
  %67 = sext i32 %66 to i64
  %68 = getelementptr inbounds %struct.Labeldesc, ptr %65, i64 %67
  %69 = getelementptr inbounds %struct.Labeldesc, ptr %68, i32 0, i32 1
  store i32 %62, ptr %69, align 8
  %70 = load i32, ptr %11, align 4
  %71 = add nsw i32 %70, 1
  %72 = load ptr, ptr %7, align 8
  %73 = getelementptr inbounds %struct.Labellist, ptr %72, i32 0, i32 1
  store i32 %71, ptr %73, align 8
  %74 = load i32, ptr %11, align 4
  ret i32 %74
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @removevars(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.FuncState, ptr %6, i32 0, i32 13
  %8 = load i8, ptr %7, align 2
  %9 = zext i8 %8 to i32
  %10 = load i32, ptr %4, align 4
  %11 = sub nsw i32 %9, %10
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 2
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 10
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.Dyndata, ptr %16, i32 0, i32 0
  %18 = getelementptr inbounds %struct.anon.8, ptr %17, i32 0, i32 1
  %19 = load i32, ptr %18, align 8
  %20 = sub nsw i32 %19, %11
  store i32 %20, ptr %18, align 8
  br label %21

21:                                               ; preds = %44, %2
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 13
  %24 = load i8, ptr %23, align 2
  %25 = zext i8 %24 to i32
  %26 = load i32, ptr %4, align 4
  %27 = icmp sgt i32 %25, %26
  br i1 %27, label %28, label %45

28:                                               ; preds = %21
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.FuncState, ptr %30, i32 0, i32 13
  %32 = load i8, ptr %31, align 2
  %33 = add i8 %32, -1
  store i8 %33, ptr %31, align 2
  %34 = zext i8 %33 to i32
  %35 = call ptr @localdebuginfo(ptr noundef %29, i32 noundef %34)
  store ptr %35, ptr %5, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = icmp ne ptr %36, null
  br i1 %37, label %38, label %44

38:                                               ; preds = %28
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.FuncState, ptr %39, i32 0, i32 4
  %41 = load i32, ptr %40, align 8
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.LocVar, ptr %42, i32 0, i32 2
  store i32 %41, ptr %43, align 4
  br label %44

44:                                               ; preds = %38, %28
  br label %21, !llvm.loop !24

45:                                               ; preds = %21
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @createlabel(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  %13 = load ptr, ptr %6, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 5
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %10, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 10
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.Dyndata, ptr %18, i32 0, i32 2
  store ptr %19, ptr %11, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = load ptr, ptr %11, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = load i32, ptr %8, align 4
  %24 = load ptr, ptr %10, align 8
  %25 = call i32 @luaK_getlabel(ptr noundef %24)
  %26 = call i32 @newlabelentry(ptr noundef %20, ptr noundef %21, ptr noundef %22, i32 noundef %23, i32 noundef %25)
  store i32 %26, ptr %12, align 4
  %27 = load i32, ptr %9, align 4
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %42

29:                                               ; preds = %4
  %30 = load ptr, ptr %10, align 8
  %31 = getelementptr inbounds %struct.FuncState, ptr %30, i32 0, i32 3
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.BlockCnt, ptr %32, i32 0, i32 3
  %34 = load i8, ptr %33, align 8
  %35 = load ptr, ptr %11, align 8
  %36 = getelementptr inbounds %struct.Labellist, ptr %35, i32 0, i32 0
  %37 = load ptr, ptr %36, align 8
  %38 = load i32, ptr %12, align 4
  %39 = sext i32 %38 to i64
  %40 = getelementptr inbounds %struct.Labeldesc, ptr %37, i64 %39
  %41 = getelementptr inbounds %struct.Labeldesc, ptr %40, i32 0, i32 3
  store i8 %34, ptr %41, align 8
  br label %42

42:                                               ; preds = %29, %4
  %43 = load ptr, ptr %6, align 8
  %44 = load ptr, ptr %11, align 8
  %45 = getelementptr inbounds %struct.Labellist, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = load i32, ptr %12, align 4
  %48 = sext i32 %47 to i64
  %49 = getelementptr inbounds %struct.Labeldesc, ptr %46, i64 %48
  %50 = call i32 @solvegotos(ptr noundef %43, ptr noundef %49)
  %51 = icmp ne i32 %50, 0
  br i1 %51, label %52, label %57

52:                                               ; preds = %42
  %53 = load ptr, ptr %10, align 8
  %54 = load ptr, ptr %10, align 8
  %55 = call i32 @luaY_nvarstack(ptr noundef %54)
  %56 = call i32 @luaK_codeABCk(ptr noundef %53, i32 noundef 54, i32 noundef %55, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  store i32 1, ptr %5, align 4
  br label %58

57:                                               ; preds = %42
  store i32 0, ptr %5, align 4
  br label %58

58:                                               ; preds = %57, %52
  %59 = load i32, ptr %5, align 4
  ret i32 %59
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @movegotosout(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 2
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 10
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.Dyndata, ptr %12, i32 0, i32 1
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.BlockCnt, ptr %14, i32 0, i32 2
  %16 = load i32, ptr %15, align 4
  store i32 %16, ptr %5, align 4
  br label %17

17:                                               ; preds = %60, %2
  %18 = load i32, ptr %5, align 4
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.Labellist, ptr %19, i32 0, i32 1
  %21 = load i32, ptr %20, align 8
  %22 = icmp slt i32 %18, %21
  br i1 %22, label %23, label %63

23:                                               ; preds = %17
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.Labellist, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = load i32, ptr %5, align 4
  %28 = sext i32 %27 to i64
  %29 = getelementptr inbounds %struct.Labeldesc, ptr %26, i64 %28
  store ptr %29, ptr %7, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = getelementptr inbounds %struct.Labeldesc, ptr %31, i32 0, i32 3
  %33 = load i8, ptr %32, align 8
  %34 = zext i8 %33 to i32
  %35 = call i32 @reglevel(ptr noundef %30, i32 noundef %34)
  %36 = load ptr, ptr %3, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.BlockCnt, ptr %37, i32 0, i32 3
  %39 = load i8, ptr %38, align 8
  %40 = zext i8 %39 to i32
  %41 = call i32 @reglevel(ptr noundef %36, i32 noundef %40)
  %42 = icmp sgt i32 %35, %41
  br i1 %42, label %43, label %54

43:                                               ; preds = %23
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.BlockCnt, ptr %44, i32 0, i32 4
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = load ptr, ptr %7, align 8
  %49 = getelementptr inbounds %struct.Labeldesc, ptr %48, i32 0, i32 4
  %50 = load i8, ptr %49, align 1
  %51 = zext i8 %50 to i32
  %52 = or i32 %51, %47
  %53 = trunc i32 %52 to i8
  store i8 %53, ptr %49, align 1
  br label %54

54:                                               ; preds = %43, %23
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.BlockCnt, ptr %55, i32 0, i32 3
  %57 = load i8, ptr %56, align 8
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.Labeldesc, ptr %58, i32 0, i32 3
  store i8 %57, ptr %59, align 8
  br label %60

60:                                               ; preds = %54
  %61 = load i32, ptr %5, align 4
  %62 = add nsw i32 %61, 1
  store i32 %62, ptr %5, align 4
  br label %17, !llvm.loop !25

63:                                               ; preds = %17
  ret void
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @undefgoto(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Labeldesc, ptr %6, i32 0, i32 0
  %8 = load ptr, ptr %7, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 6
  %11 = load ptr, ptr %10, align 8
  %12 = call ptr @luaS_newlstr(ptr noundef %11, ptr noundef @.str.4, i64 noundef 5)
  %13 = icmp eq ptr %8, %12
  br i1 %13, label %14, label %23

14:                                               ; preds = %2
  store ptr @.str.15, ptr %5, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.Labeldesc, ptr %19, i32 0, i32 2
  %21 = load i32, ptr %20, align 4
  %22 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %17, ptr noundef %18, i32 noundef %21)
  store ptr %22, ptr %5, align 8
  br label %37

23:                                               ; preds = %2
  store ptr @.str.16, ptr %5, align 8
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 6
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Labeldesc, ptr %28, i32 0, i32 0
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %struct.TString, ptr %30, i32 0, i32 7
  %32 = getelementptr inbounds [1 x i8], ptr %31, i64 0, i64 0
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.Labeldesc, ptr %33, i32 0, i32 2
  %35 = load i32, ptr %34, align 4
  %36 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %26, ptr noundef %27, ptr noundef %32, i32 noundef %35)
  store ptr %36, ptr %5, align 8
  br label %37

37:                                               ; preds = %23, %14
  %38 = load ptr, ptr %3, align 8
  %39 = load ptr, ptr %5, align 8
  call void @luaK_semerror(ptr noundef %38, ptr noundef %39) #6
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @localdebuginfo(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call ptr @getlocalvardesc(ptr noundef %8, i32 noundef %9)
  store ptr %10, ptr %6, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds %struct.anon.9, ptr %11, i32 0, i32 2
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = icmp eq i32 %14, 3
  br i1 %15, label %16, label %17

16:                                               ; preds = %2
  store ptr null, ptr %3, align 8
  br label %30

17:                                               ; preds = %2
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.anon.9, ptr %18, i32 0, i32 4
  %20 = load i16, ptr %19, align 4
  %21 = sext i16 %20 to i32
  store i32 %21, ptr %7, align 4
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 21
  %26 = load ptr, ptr %25, align 8
  %27 = load i32, ptr %7, align 4
  %28 = sext i32 %27 to i64
  %29 = getelementptr inbounds %struct.LocVar, ptr %26, i64 %28
  store ptr %29, ptr %3, align 8
  br label %30

30:                                               ; preds = %17, %16
  %31 = load ptr, ptr %3, align 8
  ret ptr %31
}

declare hidden i32 @luaK_getlabel(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @solvegotos(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 10
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.Dyndata, ptr %10, i32 0, i32 1
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 5
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 3
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.BlockCnt, ptr %16, i32 0, i32 2
  %18 = load i32, ptr %17, align 4
  store i32 %18, ptr %6, align 4
  store i32 0, ptr %7, align 4
  br label %19

19:                                               ; preds = %56, %2
  %20 = load i32, ptr %6, align 4
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.Labellist, ptr %21, i32 0, i32 1
  %23 = load i32, ptr %22, align 8
  %24 = icmp slt i32 %20, %23
  br i1 %24, label %25, label %57

25:                                               ; preds = %19
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.Labellist, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %6, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds %struct.Labeldesc, ptr %28, i64 %30
  %32 = getelementptr inbounds %struct.Labeldesc, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.Labeldesc, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = icmp eq ptr %33, %36
  br i1 %37, label %38, label %53

38:                                               ; preds = %25
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.Labellist, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  %42 = load i32, ptr %6, align 4
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds %struct.Labeldesc, ptr %41, i64 %43
  %45 = getelementptr inbounds %struct.Labeldesc, ptr %44, i32 0, i32 4
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = load i32, ptr %7, align 4
  %49 = or i32 %48, %47
  store i32 %49, ptr %7, align 4
  %50 = load ptr, ptr %3, align 8
  %51 = load i32, ptr %6, align 4
  %52 = load ptr, ptr %4, align 8
  call void @solvegoto(ptr noundef %50, i32 noundef %51, ptr noundef %52)
  br label %56

53:                                               ; preds = %25
  %54 = load i32, ptr %6, align 4
  %55 = add nsw i32 %54, 1
  store i32 %55, ptr %6, align 4
  br label %56

56:                                               ; preds = %53, %38
  br label %19, !llvm.loop !26

57:                                               ; preds = %19
  %58 = load i32, ptr %7, align 4
  ret i32 %58
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @solvegoto(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 10
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.Dyndata, ptr %12, i32 0, i32 1
  store ptr %13, ptr %8, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = getelementptr inbounds %struct.Labellist, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = load i32, ptr %5, align 4
  %18 = sext i32 %17 to i64
  %19 = getelementptr inbounds %struct.Labeldesc, ptr %16, i64 %18
  store ptr %19, ptr %9, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = getelementptr inbounds %struct.Labeldesc, ptr %20, i32 0, i32 3
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.Labeldesc, ptr %24, i32 0, i32 3
  %26 = load i8, ptr %25, align 8
  %27 = zext i8 %26 to i32
  %28 = icmp slt i32 %23, %27
  %29 = zext i1 %28 to i32
  %30 = icmp ne i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %34, label %37

34:                                               ; preds = %3
  %35 = load ptr, ptr %4, align 8
  %36 = load ptr, ptr %9, align 8
  call void @jumpscopeerror(ptr noundef %35, ptr noundef %36) #6
  unreachable

37:                                               ; preds = %3
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.LexState, ptr %38, i32 0, i32 5
  %40 = load ptr, ptr %39, align 8
  %41 = load ptr, ptr %9, align 8
  %42 = getelementptr inbounds %struct.Labeldesc, ptr %41, i32 0, i32 1
  %43 = load i32, ptr %42, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = getelementptr inbounds %struct.Labeldesc, ptr %44, i32 0, i32 1
  %46 = load i32, ptr %45, align 8
  call void @luaK_patchlist(ptr noundef %40, i32 noundef %43, i32 noundef %46)
  %47 = load i32, ptr %5, align 4
  store i32 %47, ptr %7, align 4
  br label %48

48:                                               ; preds = %69, %37
  %49 = load i32, ptr %7, align 4
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.Labellist, ptr %50, i32 0, i32 1
  %52 = load i32, ptr %51, align 8
  %53 = sub nsw i32 %52, 1
  %54 = icmp slt i32 %49, %53
  br i1 %54, label %55, label %72

55:                                               ; preds = %48
  %56 = load ptr, ptr %8, align 8
  %57 = getelementptr inbounds %struct.Labellist, ptr %56, i32 0, i32 0
  %58 = load ptr, ptr %57, align 8
  %59 = load i32, ptr %7, align 4
  %60 = sext i32 %59 to i64
  %61 = getelementptr inbounds %struct.Labeldesc, ptr %58, i64 %60
  %62 = load ptr, ptr %8, align 8
  %63 = getelementptr inbounds %struct.Labellist, ptr %62, i32 0, i32 0
  %64 = load ptr, ptr %63, align 8
  %65 = load i32, ptr %7, align 4
  %66 = add nsw i32 %65, 1
  %67 = sext i32 %66 to i64
  %68 = getelementptr inbounds %struct.Labeldesc, ptr %64, i64 %67
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %61, ptr align 8 %68, i64 24, i1 false)
  br label %69

69:                                               ; preds = %55
  %70 = load i32, ptr %7, align 4
  %71 = add nsw i32 %70, 1
  store i32 %71, ptr %7, align 4
  br label %48, !llvm.loop !27

72:                                               ; preds = %48
  %73 = load ptr, ptr %8, align 8
  %74 = getelementptr inbounds %struct.Labellist, ptr %73, i32 0, i32 1
  %75 = load i32, ptr %74, align 8
  %76 = add nsw i32 %75, -1
  store i32 %76, ptr %74, align 8
  ret void
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @jumpscopeerror(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.Labeldesc, ptr %10, i32 0, i32 3
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = call ptr @getlocalvardesc(ptr noundef %9, i32 noundef %13)
  %15 = getelementptr inbounds %struct.anon.9, ptr %14, i32 0, i32 5
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.TString, ptr %16, i32 0, i32 7
  %18 = getelementptr inbounds [1 x i8], ptr %17, i64 0, i64 0
  store ptr %18, ptr %5, align 8
  store ptr @.str.14, ptr %6, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.Labeldesc, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.TString, ptr %25, i32 0, i32 7
  %27 = getelementptr inbounds [1 x i8], ptr %26, i64 0, i64 0
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Labeldesc, ptr %28, i32 0, i32 2
  %30 = load i32, ptr %29, align 4
  %31 = load ptr, ptr %5, align 8
  %32 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %21, ptr noundef %22, ptr noundef %27, i32 noundef %30, ptr noundef %31)
  store ptr %32, ptr %6, align 8
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %6, align 8
  call void @luaK_semerror(ptr noundef %33, ptr noundef %34) #6
  unreachable
}

declare hidden void @luaK_patchlist(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noreturn
declare hidden void @luaK_semerror(ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @cond(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @expr(ptr noundef %4, ptr noundef %3)
  %5 = getelementptr inbounds %struct.expdesc, ptr %3, i32 0, i32 0
  %6 = load i32, ptr %5, align 8
  %7 = icmp eq i32 %6, 1
  br i1 %7, label %8, label %10

8:                                                ; preds = %1
  %9 = getelementptr inbounds %struct.expdesc, ptr %3, i32 0, i32 0
  store i32 3, ptr %9, align 8
  br label %10

10:                                               ; preds = %8, %1
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  call void @luaK_goiftrue(ptr noundef %13, ptr noundef %3)
  %14 = getelementptr inbounds %struct.expdesc, ptr %3, i32 0, i32 3
  %15 = load i32, ptr %14, align 4
  ret i32 %15
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @error_expected(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load i32, ptr %4, align 4
  %11 = call ptr @luaX_token2str(ptr noundef %9, i32 noundef %10)
  %12 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %8, ptr noundef @.str.18, ptr noundef %11)
  call void @luaX_syntaxerror(ptr noundef %5, ptr noundef %12) #6
  unreachable
}

declare hidden ptr @luaX_token2str(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fornum(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 5
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 15
  %14 = load i8, ptr %13, align 4
  %15 = zext i8 %14 to i32
  store i32 %15, ptr %8, align 4
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = call ptr @luaX_newstring(ptr noundef %17, ptr noundef @.str.20, i64 noundef 11)
  %19 = call i32 @new_localvar(ptr noundef %16, ptr noundef %18)
  %20 = load ptr, ptr %4, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = call ptr @luaX_newstring(ptr noundef %21, ptr noundef @.str.20, i64 noundef 11)
  %23 = call i32 @new_localvar(ptr noundef %20, ptr noundef %22)
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = call ptr @luaX_newstring(ptr noundef %25, ptr noundef @.str.20, i64 noundef 11)
  %27 = call i32 @new_localvar(ptr noundef %24, ptr noundef %26)
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = call i32 @new_localvar(ptr noundef %28, ptr noundef %29)
  %31 = load ptr, ptr %4, align 8
  call void @checknext(ptr noundef %31, i32 noundef 61)
  %32 = load ptr, ptr %4, align 8
  call void @exp1(ptr noundef %32)
  %33 = load ptr, ptr %4, align 8
  call void @checknext(ptr noundef %33, i32 noundef 44)
  %34 = load ptr, ptr %4, align 8
  call void @exp1(ptr noundef %34)
  %35 = load ptr, ptr %4, align 8
  %36 = call i32 @testnext(ptr noundef %35, i32 noundef 44)
  %37 = icmp ne i32 %36, 0
  br i1 %37, label %38, label %40

38:                                               ; preds = %3
  %39 = load ptr, ptr %4, align 8
  call void @exp1(ptr noundef %39)
  br label %47

40:                                               ; preds = %3
  %41 = load ptr, ptr %7, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = getelementptr inbounds %struct.FuncState, ptr %42, i32 0, i32 15
  %44 = load i8, ptr %43, align 4
  %45 = zext i8 %44 to i32
  call void @luaK_int(ptr noundef %41, i32 noundef %45, i64 noundef 1)
  %46 = load ptr, ptr %7, align 8
  call void @luaK_reserveregs(ptr noundef %46, i32 noundef 1)
  br label %47

47:                                               ; preds = %40, %38
  %48 = load ptr, ptr %4, align 8
  call void @adjustlocalvars(ptr noundef %48, i32 noundef 3)
  %49 = load ptr, ptr %4, align 8
  %50 = load i32, ptr %8, align 4
  %51 = load i32, ptr %6, align 4
  call void @forbody(ptr noundef %49, i32 noundef %50, i32 noundef %51, i32 noundef 1, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @forlist(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.expdesc, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
  store i32 5, ptr %7, align 4
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 15
  %15 = load i8, ptr %14, align 4
  %16 = zext i8 %15 to i32
  store i32 %16, ptr %9, align 4
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = call ptr @luaX_newstring(ptr noundef %18, ptr noundef @.str.20, i64 noundef 11)
  %20 = call i32 @new_localvar(ptr noundef %17, ptr noundef %19)
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = call ptr @luaX_newstring(ptr noundef %22, ptr noundef @.str.20, i64 noundef 11)
  %24 = call i32 @new_localvar(ptr noundef %21, ptr noundef %23)
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %3, align 8
  %27 = call ptr @luaX_newstring(ptr noundef %26, ptr noundef @.str.20, i64 noundef 11)
  %28 = call i32 @new_localvar(ptr noundef %25, ptr noundef %27)
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = call ptr @luaX_newstring(ptr noundef %30, ptr noundef @.str.20, i64 noundef 11)
  %32 = call i32 @new_localvar(ptr noundef %29, ptr noundef %31)
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = call i32 @new_localvar(ptr noundef %33, ptr noundef %34)
  br label %36

36:                                               ; preds = %40, %2
  %37 = load ptr, ptr %3, align 8
  %38 = call i32 @testnext(ptr noundef %37, i32 noundef 44)
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %47

40:                                               ; preds = %36
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %3, align 8
  %43 = call ptr @str_checkname(ptr noundef %42)
  %44 = call i32 @new_localvar(ptr noundef %41, ptr noundef %43)
  %45 = load i32, ptr %7, align 4
  %46 = add nsw i32 %45, 1
  store i32 %46, ptr %7, align 4
  br label %36, !llvm.loop !28

47:                                               ; preds = %36
  %48 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %48, i32 noundef 267)
  %49 = load ptr, ptr %3, align 8
  %50 = getelementptr inbounds %struct.LexState, ptr %49, i32 0, i32 1
  %51 = load i32, ptr %50, align 4
  store i32 %51, ptr %8, align 4
  %52 = load ptr, ptr %3, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = call i32 @explist(ptr noundef %53, ptr noundef %6)
  call void @adjust_assign(ptr noundef %52, i32 noundef 4, i32 noundef %54, ptr noundef %6)
  %55 = load ptr, ptr %3, align 8
  call void @adjustlocalvars(ptr noundef %55, i32 noundef 4)
  %56 = load ptr, ptr %5, align 8
  call void @marktobeclosed(ptr noundef %56)
  %57 = load ptr, ptr %5, align 8
  call void @luaK_checkstack(ptr noundef %57, i32 noundef 3)
  %58 = load ptr, ptr %3, align 8
  %59 = load i32, ptr %9, align 4
  %60 = load i32, ptr %8, align 4
  %61 = load i32, ptr %7, align 4
  %62 = sub nsw i32 %61, 4
  call void @forbody(ptr noundef %58, i32 noundef %59, i32 noundef %60, i32 noundef %62, i32 noundef 1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @exp1(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @expr(ptr noundef %4, ptr noundef %3)
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 5
  %7 = load ptr, ptr %6, align 8
  call void @luaK_exp2nextreg(ptr noundef %7, ptr noundef %3)
  ret void
}

declare hidden void @luaK_int(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @forbody(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca %struct.BlockCnt, align 8
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 5
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %12, align 8
  %18 = load ptr, ptr %6, align 8
  call void @checknext(ptr noundef %18, i32 noundef 258)
  %19 = load ptr, ptr %12, align 8
  %20 = load i32, ptr %10, align 4
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds [2 x i32], ptr @forbody.forprep, i64 0, i64 %21
  %23 = load i32, ptr %22, align 4
  %24 = load i32, ptr %7, align 4
  %25 = call i32 @luaK_codeABx(ptr noundef %19, i32 noundef %23, i32 noundef %24, i32 noundef 0)
  store i32 %25, ptr %13, align 4
  %26 = load ptr, ptr %12, align 8
  call void @enterblock(ptr noundef %26, ptr noundef %11, i8 noundef zeroext 0)
  %27 = load ptr, ptr %6, align 8
  %28 = load i32, ptr %9, align 4
  call void @adjustlocalvars(ptr noundef %27, i32 noundef %28)
  %29 = load ptr, ptr %12, align 8
  %30 = load i32, ptr %9, align 4
  call void @luaK_reserveregs(ptr noundef %29, i32 noundef %30)
  %31 = load ptr, ptr %6, align 8
  call void @block(ptr noundef %31)
  %32 = load ptr, ptr %12, align 8
  call void @leaveblock(ptr noundef %32)
  %33 = load ptr, ptr %12, align 8
  %34 = load i32, ptr %13, align 4
  %35 = load ptr, ptr %12, align 8
  %36 = call i32 @luaK_getlabel(ptr noundef %35)
  call void @fixforjump(ptr noundef %33, i32 noundef %34, i32 noundef %36, i32 noundef 0)
  %37 = load i32, ptr %10, align 4
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %39, label %46

39:                                               ; preds = %5
  %40 = load ptr, ptr %12, align 8
  %41 = load i32, ptr %7, align 4
  %42 = load i32, ptr %9, align 4
  %43 = call i32 @luaK_codeABCk(ptr noundef %40, i32 noundef 76, i32 noundef %41, i32 noundef 0, i32 noundef %42, i32 noundef 0)
  %44 = load ptr, ptr %12, align 8
  %45 = load i32, ptr %8, align 4
  call void @luaK_fixline(ptr noundef %44, i32 noundef %45)
  br label %46

46:                                               ; preds = %39, %5
  %47 = load ptr, ptr %12, align 8
  %48 = load i32, ptr %10, align 4
  %49 = sext i32 %48 to i64
  %50 = getelementptr inbounds [2 x i32], ptr @forbody.forloop, i64 0, i64 %49
  %51 = load i32, ptr %50, align 4
  %52 = load i32, ptr %7, align 4
  %53 = call i32 @luaK_codeABx(ptr noundef %47, i32 noundef %51, i32 noundef %52, i32 noundef 0)
  store i32 %53, ptr %14, align 4
  %54 = load ptr, ptr %12, align 8
  %55 = load i32, ptr %14, align 4
  %56 = load i32, ptr %13, align 4
  %57 = add nsw i32 %56, 1
  call void @fixforjump(ptr noundef %54, i32 noundef %55, i32 noundef %57, i32 noundef 1)
  %58 = load ptr, ptr %12, align 8
  %59 = load i32, ptr %8, align 4
  call void @luaK_fixline(ptr noundef %58, i32 noundef %59)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fixforjump(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.FuncState, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.Proto, ptr %13, i32 0, i32 16
  %15 = load ptr, ptr %14, align 8
  %16 = load i32, ptr %6, align 4
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds i32, ptr %15, i64 %17
  store ptr %18, ptr %9, align 8
  %19 = load i32, ptr %7, align 4
  %20 = load i32, ptr %6, align 4
  %21 = add nsw i32 %20, 1
  %22 = sub nsw i32 %19, %21
  store i32 %22, ptr %10, align 4
  %23 = load i32, ptr %8, align 4
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %25, label %28

25:                                               ; preds = %4
  %26 = load i32, ptr %10, align 4
  %27 = sub nsw i32 0, %26
  store i32 %27, ptr %10, align 4
  br label %28

28:                                               ; preds = %25, %4
  %29 = load i32, ptr %10, align 4
  %30 = icmp sgt i32 %29, 131071
  %31 = zext i1 %30 to i32
  %32 = icmp ne i32 %31, 0
  %33 = zext i1 %32 to i32
  %34 = sext i32 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %36, label %40

36:                                               ; preds = %28
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.FuncState, ptr %37, i32 0, i32 2
  %39 = load ptr, ptr %38, align 8
  call void @luaX_syntaxerror(ptr noundef %39, ptr noundef @.str.21) #6
  unreachable

40:                                               ; preds = %28
  %41 = load ptr, ptr %9, align 8
  %42 = load i32, ptr %41, align 4
  %43 = and i32 %42, 32767
  %44 = load i32, ptr %10, align 4
  %45 = shl i32 %44, 15
  %46 = and i32 %45, -32768
  %47 = or i32 %43, %46
  %48 = load ptr, ptr %9, align 8
  store i32 %47, ptr %48, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @adjust_assign(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 5
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %9, align 8
  %15 = load i32, ptr %6, align 4
  %16 = load i32, ptr %7, align 4
  %17 = sub nsw i32 %15, %16
  store i32 %17, ptr %10, align 4
  %18 = load ptr, ptr %8, align 8
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 18
  br i1 %21, label %27, label %22

22:                                               ; preds = %4
  %23 = load ptr, ptr %8, align 8
  %24 = getelementptr inbounds %struct.expdesc, ptr %23, i32 0, i32 0
  %25 = load i32, ptr %24, align 8
  %26 = icmp eq i32 %25, 19
  br i1 %26, label %27, label %37

27:                                               ; preds = %22, %4
  %28 = load i32, ptr %10, align 4
  %29 = add nsw i32 %28, 1
  store i32 %29, ptr %11, align 4
  %30 = load i32, ptr %11, align 4
  %31 = icmp slt i32 %30, 0
  br i1 %31, label %32, label %33

32:                                               ; preds = %27
  store i32 0, ptr %11, align 4
  br label %33

33:                                               ; preds = %32, %27
  %34 = load ptr, ptr %9, align 8
  %35 = load ptr, ptr %8, align 8
  %36 = load i32, ptr %11, align 4
  call void @luaK_setreturns(ptr noundef %34, ptr noundef %35, i32 noundef %36)
  br label %56

37:                                               ; preds = %22
  %38 = load ptr, ptr %8, align 8
  %39 = getelementptr inbounds %struct.expdesc, ptr %38, i32 0, i32 0
  %40 = load i32, ptr %39, align 8
  %41 = icmp ne i32 %40, 0
  br i1 %41, label %42, label %45

42:                                               ; preds = %37
  %43 = load ptr, ptr %9, align 8
  %44 = load ptr, ptr %8, align 8
  call void @luaK_exp2nextreg(ptr noundef %43, ptr noundef %44)
  br label %45

45:                                               ; preds = %42, %37
  %46 = load i32, ptr %10, align 4
  %47 = icmp sgt i32 %46, 0
  br i1 %47, label %48, label %55

48:                                               ; preds = %45
  %49 = load ptr, ptr %9, align 8
  %50 = load ptr, ptr %9, align 8
  %51 = getelementptr inbounds %struct.FuncState, ptr %50, i32 0, i32 15
  %52 = load i8, ptr %51, align 4
  %53 = zext i8 %52 to i32
  %54 = load i32, ptr %10, align 4
  call void @luaK_nil(ptr noundef %49, i32 noundef %53, i32 noundef %54)
  br label %55

55:                                               ; preds = %48, %45
  br label %56

56:                                               ; preds = %55, %33
  %57 = load i32, ptr %10, align 4
  %58 = icmp sgt i32 %57, 0
  br i1 %58, label %59, label %62

59:                                               ; preds = %56
  %60 = load ptr, ptr %9, align 8
  %61 = load i32, ptr %10, align 4
  call void @luaK_reserveregs(ptr noundef %60, i32 noundef %61)
  br label %70

62:                                               ; preds = %56
  %63 = load i32, ptr %10, align 4
  %64 = load ptr, ptr %9, align 8
  %65 = getelementptr inbounds %struct.FuncState, ptr %64, i32 0, i32 15
  %66 = load i8, ptr %65, align 4
  %67 = zext i8 %66 to i32
  %68 = add nsw i32 %67, %63
  %69 = trunc i32 %68 to i8
  store i8 %69, ptr %65, align 4
  br label %70

70:                                               ; preds = %62, %59
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @marktobeclosed(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.FuncState, ptr %4, i32 0, i32 3
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.BlockCnt, ptr %7, i32 0, i32 4
  store i8 1, ptr %8, align 1
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.BlockCnt, ptr %9, i32 0, i32 6
  store i8 1, ptr %10, align 1
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.FuncState, ptr %11, i32 0, i32 17
  store i8 1, ptr %12, align 2
  ret void
}

declare hidden void @luaK_checkstack(ptr noundef, i32 noundef) #1

declare hidden void @luaK_nil(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @funcname(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %5, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @singlevar(ptr noundef %6, ptr noundef %7)
  br label %8

8:                                                ; preds = %14, %2
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 3
  %11 = getelementptr inbounds %struct.Token, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 8
  %13 = icmp eq i32 %12, 46
  br i1 %13, label %14, label %17

14:                                               ; preds = %8
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %4, align 8
  call void @fieldsel(ptr noundef %15, ptr noundef %16)
  br label %8, !llvm.loop !29

17:                                               ; preds = %8
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 3
  %20 = getelementptr inbounds %struct.Token, ptr %19, i32 0, i32 0
  %21 = load i32, ptr %20, align 8
  %22 = icmp eq i32 %21, 58
  br i1 %22, label %23, label %26

23:                                               ; preds = %17
  store i32 1, ptr %5, align 4
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %4, align 8
  call void @fieldsel(ptr noundef %24, ptr noundef %25)
  br label %26

26:                                               ; preds = %23, %17
  %27 = load i32, ptr %5, align 4
  ret i32 %27
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @check_readonly(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %5, align 8
  store ptr null, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 0
  %15 = load i32, ptr %14, align 8
  switch i32 %15, label %69 [
    i32 11, label %16
    i32 9, label %30
    i32 10, label %48
  ]

16:                                               ; preds = %2
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.LexState, ptr %17, i32 0, i32 10
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.Dyndata, ptr %19, i32 0, i32 0
  %21 = getelementptr inbounds %struct.anon.8, ptr %20, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.expdesc, ptr %23, i32 0, i32 1
  %25 = load i32, ptr %24, align 8
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds %union.Vardesc, ptr %22, i64 %26
  %28 = getelementptr inbounds %struct.anon.9, ptr %27, i32 0, i32 5
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %6, align 8
  br label %70

30:                                               ; preds = %2
  %31 = load ptr, ptr %5, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.expdesc, ptr %32, i32 0, i32 1
  %34 = getelementptr inbounds %struct.anon.12, ptr %33, i32 0, i32 1
  %35 = load i16, ptr %34, align 2
  %36 = zext i16 %35 to i32
  %37 = call ptr @getlocalvardesc(ptr noundef %31, i32 noundef %36)
  store ptr %37, ptr %7, align 8
  %38 = load ptr, ptr %7, align 8
  %39 = getelementptr inbounds %struct.anon.9, ptr %38, i32 0, i32 2
  %40 = load i8, ptr %39, align 1
  %41 = zext i8 %40 to i32
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %43, label %47

43:                                               ; preds = %30
  %44 = load ptr, ptr %7, align 8
  %45 = getelementptr inbounds %struct.anon.9, ptr %44, i32 0, i32 5
  %46 = load ptr, ptr %45, align 8
  store ptr %46, ptr %6, align 8
  br label %47

47:                                               ; preds = %43, %30
  br label %70

48:                                               ; preds = %2
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.FuncState, ptr %49, i32 0, i32 0
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.Proto, ptr %51, i32 0, i32 18
  %53 = load ptr, ptr %52, align 8
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.expdesc, ptr %54, i32 0, i32 1
  %56 = load i32, ptr %55, align 8
  %57 = sext i32 %56 to i64
  %58 = getelementptr inbounds %struct.Upvaldesc, ptr %53, i64 %57
  store ptr %58, ptr %8, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = getelementptr inbounds %struct.Upvaldesc, ptr %59, i32 0, i32 3
  %61 = load i8, ptr %60, align 2
  %62 = zext i8 %61 to i32
  %63 = icmp ne i32 %62, 0
  br i1 %63, label %64, label %68

64:                                               ; preds = %48
  %65 = load ptr, ptr %8, align 8
  %66 = getelementptr inbounds %struct.Upvaldesc, ptr %65, i32 0, i32 0
  %67 = load ptr, ptr %66, align 8
  store ptr %67, ptr %6, align 8
  br label %68

68:                                               ; preds = %64, %48
  br label %70

69:                                               ; preds = %2
  br label %83

70:                                               ; preds = %68, %47, %16
  %71 = load ptr, ptr %6, align 8
  %72 = icmp ne ptr %71, null
  br i1 %72, label %73, label %83

73:                                               ; preds = %70
  %74 = load ptr, ptr %3, align 8
  %75 = getelementptr inbounds %struct.LexState, ptr %74, i32 0, i32 6
  %76 = load ptr, ptr %75, align 8
  %77 = load ptr, ptr %6, align 8
  %78 = getelementptr inbounds %struct.TString, ptr %77, i32 0, i32 7
  %79 = getelementptr inbounds [1 x i8], ptr %78, i64 0, i64 0
  %80 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %76, ptr noundef @.str.22, ptr noundef %79)
  store ptr %80, ptr %9, align 8
  %81 = load ptr, ptr %3, align 8
  %82 = load ptr, ptr %9, align 8
  call void @luaK_semerror(ptr noundef %81, ptr noundef %82) #6
  unreachable

83:                                               ; preds = %69, %70
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getlocalattribute(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call i32 @testnext(ptr noundef %5, i32 noundef 60)
  %7 = icmp ne i32 %6, 0
  br i1 %7, label %8, label %30

8:                                                ; preds = %1
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @str_checkname(ptr noundef %9)
  %11 = getelementptr inbounds %struct.TString, ptr %10, i32 0, i32 7
  %12 = getelementptr inbounds [1 x i8], ptr %11, i64 0, i64 0
  store ptr %12, ptr %4, align 8
  %13 = load ptr, ptr %3, align 8
  call void @checknext(ptr noundef %13, i32 noundef 62)
  %14 = load ptr, ptr %4, align 8
  %15 = call i32 @strcmp(ptr noundef %14, ptr noundef @.str.24) #7
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %18

17:                                               ; preds = %8
  store i32 1, ptr %2, align 4
  br label %31

18:                                               ; preds = %8
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @strcmp(ptr noundef %19, ptr noundef @.str.25) #7
  %21 = icmp eq i32 %20, 0
  br i1 %21, label %22, label %23

22:                                               ; preds = %18
  store i32 2, ptr %2, align 4
  br label %31

23:                                               ; preds = %18
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.LexState, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %27, ptr noundef @.str.26, ptr noundef %28)
  call void @luaK_semerror(ptr noundef %24, ptr noundef %29) #6
  unreachable

30:                                               ; preds = %1
  store i32 0, ptr %2, align 4
  br label %31

31:                                               ; preds = %30, %22, %17
  %32 = load i32, ptr %2, align 4
  ret i32 %32
}

declare hidden i32 @luaK_exp2const(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checktoclose(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = icmp ne i32 %5, -1
  br i1 %6, label %7, label %14

7:                                                ; preds = %2
  %8 = load ptr, ptr %3, align 8
  call void @marktobeclosed(ptr noundef %8)
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = call i32 @reglevel(ptr noundef %10, i32 noundef %11)
  %13 = call i32 @luaK_codeABCk(ptr noundef %9, i32 noundef 55, i32 noundef %12, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  br label %14

14:                                               ; preds = %7, %2
  ret void
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #5

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkrepeated(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call ptr @findlabel(ptr noundef %7, ptr noundef %8)
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = icmp ne ptr %10, null
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %31

17:                                               ; preds = %2
  store ptr @.str.27, ptr %6, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 6
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.TString, ptr %22, i32 0, i32 7
  %24 = getelementptr inbounds [1 x i8], ptr %23, i64 0, i64 0
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.Labeldesc, ptr %25, i32 0, i32 2
  %27 = load i32, ptr %26, align 4
  %28 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %20, ptr noundef %21, ptr noundef %24, i32 noundef %27)
  store ptr %28, ptr %6, align 8
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %6, align 8
  call void @luaK_semerror(ptr noundef %29, ptr noundef %30) #6
  unreachable

31:                                               ; preds = %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @findlabel(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 10
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 5
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 11
  %16 = load i32, ptr %15, align 4
  store i32 %16, ptr %6, align 4
  br label %17

17:                                               ; preds = %40, %2
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.Dyndata, ptr %19, i32 0, i32 2
  %21 = getelementptr inbounds %struct.Labellist, ptr %20, i32 0, i32 1
  %22 = load i32, ptr %21, align 8
  %23 = icmp slt i32 %18, %22
  br i1 %23, label %24, label %43

24:                                               ; preds = %17
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.Dyndata, ptr %25, i32 0, i32 2
  %27 = getelementptr inbounds %struct.Labellist, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %6, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds %struct.Labeldesc, ptr %28, i64 %30
  store ptr %31, ptr %8, align 8
  %32 = load ptr, ptr %8, align 8
  %33 = getelementptr inbounds %struct.Labeldesc, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %33, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = icmp eq ptr %34, %35
  br i1 %36, label %37, label %39

37:                                               ; preds = %24
  %38 = load ptr, ptr %8, align 8
  store ptr %38, ptr %3, align 8
  br label %44

39:                                               ; preds = %24
  br label %40

40:                                               ; preds = %39
  %41 = load i32, ptr %6, align 4
  %42 = add nsw i32 %41, 1
  store i32 %42, ptr %6, align 4
  br label %17, !llvm.loop !30

43:                                               ; preds = %17
  store ptr null, ptr %3, align 8
  br label %44

44:                                               ; preds = %43, %37
  %45 = load ptr, ptr %3, align 8
  ret ptr %45
}

declare hidden i32 @luaK_exp2anyreg(ptr noundef, ptr noundef) #1

declare hidden void @luaK_ret(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @restassign(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.expdesc, align 8
  %8 = alloca %struct.LHS_assign, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.LHS_assign, ptr %10, i32 0, i32 1
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 0
  %13 = load i32, ptr %12, align 8
  %14 = icmp ule i32 9, %13
  br i1 %14, label %15, label %21

15:                                               ; preds = %3
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.LHS_assign, ptr %16, i32 0, i32 1
  %18 = getelementptr inbounds %struct.expdesc, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  %20 = icmp ule i32 %19, 15
  br i1 %20, label %23, label %21

21:                                               ; preds = %15, %3
  %22 = load ptr, ptr %4, align 8
  call void @luaX_syntaxerror(ptr noundef %22, ptr noundef @.str.28) #6
  unreachable

23:                                               ; preds = %15
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.LHS_assign, ptr %25, i32 0, i32 1
  call void @check_readonly(ptr noundef %24, ptr noundef %26)
  %27 = load ptr, ptr %4, align 8
  %28 = call i32 @testnext(ptr noundef %27, i32 noundef 44)
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %30, label %61

30:                                               ; preds = %23
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.LHS_assign, ptr %8, i32 0, i32 0
  store ptr %31, ptr %32, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.LHS_assign, ptr %8, i32 0, i32 1
  call void @suffixedexp(ptr noundef %33, ptr noundef %34)
  %35 = getelementptr inbounds %struct.LHS_assign, ptr %8, i32 0, i32 1
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 0
  %37 = load i32, ptr %36, align 8
  %38 = icmp ule i32 12, %37
  br i1 %38, label %39, label %44

39:                                               ; preds = %30
  %40 = getelementptr inbounds %struct.LHS_assign, ptr %8, i32 0, i32 1
  %41 = getelementptr inbounds %struct.expdesc, ptr %40, i32 0, i32 0
  %42 = load i32, ptr %41, align 8
  %43 = icmp ule i32 %42, 15
  br i1 %43, label %48, label %44

44:                                               ; preds = %39, %30
  %45 = load ptr, ptr %4, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.LHS_assign, ptr %8, i32 0, i32 1
  call void @check_conflict(ptr noundef %45, ptr noundef %46, ptr noundef %47)
  br label %48

48:                                               ; preds = %44, %39
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.LexState, ptr %49, i32 0, i32 6
  %51 = load ptr, ptr %50, align 8
  call void @luaE_incCstack(ptr noundef %51)
  %52 = load ptr, ptr %4, align 8
  %53 = load i32, ptr %6, align 4
  %54 = add nsw i32 %53, 1
  call void @restassign(ptr noundef %52, ptr noundef %8, i32 noundef %54)
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 6
  %57 = load ptr, ptr %56, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 19
  %59 = load i32, ptr %58, align 8
  %60 = add i32 %59, -1
  store i32 %60, ptr %58, align 8
  br label %82

61:                                               ; preds = %23
  %62 = load ptr, ptr %4, align 8
  call void @checknext(ptr noundef %62, i32 noundef 61)
  %63 = load ptr, ptr %4, align 8
  %64 = call i32 @explist(ptr noundef %63, ptr noundef %7)
  store i32 %64, ptr %9, align 4
  %65 = load i32, ptr %9, align 4
  %66 = load i32, ptr %6, align 4
  %67 = icmp ne i32 %65, %66
  br i1 %67, label %68, label %72

68:                                               ; preds = %61
  %69 = load ptr, ptr %4, align 8
  %70 = load i32, ptr %6, align 4
  %71 = load i32, ptr %9, align 4
  call void @adjust_assign(ptr noundef %69, i32 noundef %70, i32 noundef %71, ptr noundef %7)
  br label %81

72:                                               ; preds = %61
  %73 = load ptr, ptr %4, align 8
  %74 = getelementptr inbounds %struct.LexState, ptr %73, i32 0, i32 5
  %75 = load ptr, ptr %74, align 8
  call void @luaK_setoneret(ptr noundef %75, ptr noundef %7)
  %76 = load ptr, ptr %4, align 8
  %77 = getelementptr inbounds %struct.LexState, ptr %76, i32 0, i32 5
  %78 = load ptr, ptr %77, align 8
  %79 = load ptr, ptr %5, align 8
  %80 = getelementptr inbounds %struct.LHS_assign, ptr %79, i32 0, i32 1
  call void @luaK_storevar(ptr noundef %78, ptr noundef %80, ptr noundef %7)
  br label %95

81:                                               ; preds = %68
  br label %82

82:                                               ; preds = %81, %48
  %83 = load ptr, ptr %4, align 8
  %84 = getelementptr inbounds %struct.LexState, ptr %83, i32 0, i32 5
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %struct.FuncState, ptr %85, i32 0, i32 15
  %87 = load i8, ptr %86, align 4
  %88 = zext i8 %87 to i32
  %89 = sub nsw i32 %88, 1
  call void @init_exp(ptr noundef %7, i32 noundef 8, i32 noundef %89)
  %90 = load ptr, ptr %4, align 8
  %91 = getelementptr inbounds %struct.LexState, ptr %90, i32 0, i32 5
  %92 = load ptr, ptr %91, align 8
  %93 = load ptr, ptr %5, align 8
  %94 = getelementptr inbounds %struct.LHS_assign, ptr %93, i32 0, i32 1
  call void @luaK_storevar(ptr noundef %92, ptr noundef %94, ptr noundef %7)
  br label %95

95:                                               ; preds = %82, %72
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @check_conflict(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 5
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 15
  %15 = load i8, ptr %14, align 4
  %16 = zext i8 %15 to i32
  store i32 %16, ptr %8, align 4
  store i32 0, ptr %9, align 4
  br label %17

17:                                               ; preds = %124, %3
  %18 = load ptr, ptr %5, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %20, label %128

20:                                               ; preds = %17
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.LHS_assign, ptr %21, i32 0, i32 1
  %23 = getelementptr inbounds %struct.expdesc, ptr %22, i32 0, i32 0
  %24 = load i32, ptr %23, align 8
  %25 = icmp ule i32 12, %24
  br i1 %25, label %26, label %123

26:                                               ; preds = %20
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.LHS_assign, ptr %27, i32 0, i32 1
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 0
  %30 = load i32, ptr %29, align 8
  %31 = icmp ule i32 %30, 15
  br i1 %31, label %32, label %123

32:                                               ; preds = %26
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.LHS_assign, ptr %33, i32 0, i32 1
  %35 = getelementptr inbounds %struct.expdesc, ptr %34, i32 0, i32 0
  %36 = load i32, ptr %35, align 8
  %37 = icmp eq i32 %36, 13
  br i1 %37, label %38, label %65

38:                                               ; preds = %32
  %39 = load ptr, ptr %6, align 8
  %40 = getelementptr inbounds %struct.expdesc, ptr %39, i32 0, i32 0
  %41 = load i32, ptr %40, align 8
  %42 = icmp eq i32 %41, 10
  br i1 %42, label %43, label %64

43:                                               ; preds = %38
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.LHS_assign, ptr %44, i32 0, i32 1
  %46 = getelementptr inbounds %struct.expdesc, ptr %45, i32 0, i32 1
  %47 = getelementptr inbounds %struct.anon.11, ptr %46, i32 0, i32 1
  %48 = load i8, ptr %47, align 2
  %49 = zext i8 %48 to i32
  %50 = load ptr, ptr %6, align 8
  %51 = getelementptr inbounds %struct.expdesc, ptr %50, i32 0, i32 1
  %52 = load i32, ptr %51, align 8
  %53 = icmp eq i32 %49, %52
  br i1 %53, label %54, label %64

54:                                               ; preds = %43
  store i32 1, ptr %9, align 4
  %55 = load ptr, ptr %5, align 8
  %56 = getelementptr inbounds %struct.LHS_assign, ptr %55, i32 0, i32 1
  %57 = getelementptr inbounds %struct.expdesc, ptr %56, i32 0, i32 0
  store i32 15, ptr %57, align 8
  %58 = load i32, ptr %8, align 4
  %59 = trunc i32 %58 to i8
  %60 = load ptr, ptr %5, align 8
  %61 = getelementptr inbounds %struct.LHS_assign, ptr %60, i32 0, i32 1
  %62 = getelementptr inbounds %struct.expdesc, ptr %61, i32 0, i32 1
  %63 = getelementptr inbounds %struct.anon.11, ptr %62, i32 0, i32 1
  store i8 %59, ptr %63, align 2
  br label %64

64:                                               ; preds = %54, %43, %38
  br label %122

65:                                               ; preds = %32
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds %struct.expdesc, ptr %66, i32 0, i32 0
  %68 = load i32, ptr %67, align 8
  %69 = icmp eq i32 %68, 9
  br i1 %69, label %70, label %90

70:                                               ; preds = %65
  %71 = load ptr, ptr %5, align 8
  %72 = getelementptr inbounds %struct.LHS_assign, ptr %71, i32 0, i32 1
  %73 = getelementptr inbounds %struct.expdesc, ptr %72, i32 0, i32 1
  %74 = getelementptr inbounds %struct.anon.11, ptr %73, i32 0, i32 1
  %75 = load i8, ptr %74, align 2
  %76 = zext i8 %75 to i32
  %77 = load ptr, ptr %6, align 8
  %78 = getelementptr inbounds %struct.expdesc, ptr %77, i32 0, i32 1
  %79 = getelementptr inbounds %struct.anon.12, ptr %78, i32 0, i32 0
  %80 = load i8, ptr %79, align 8
  %81 = zext i8 %80 to i32
  %82 = icmp eq i32 %76, %81
  br i1 %82, label %83, label %90

83:                                               ; preds = %70
  store i32 1, ptr %9, align 4
  %84 = load i32, ptr %8, align 4
  %85 = trunc i32 %84 to i8
  %86 = load ptr, ptr %5, align 8
  %87 = getelementptr inbounds %struct.LHS_assign, ptr %86, i32 0, i32 1
  %88 = getelementptr inbounds %struct.expdesc, ptr %87, i32 0, i32 1
  %89 = getelementptr inbounds %struct.anon.11, ptr %88, i32 0, i32 1
  store i8 %85, ptr %89, align 2
  br label %90

90:                                               ; preds = %83, %70, %65
  %91 = load ptr, ptr %5, align 8
  %92 = getelementptr inbounds %struct.LHS_assign, ptr %91, i32 0, i32 1
  %93 = getelementptr inbounds %struct.expdesc, ptr %92, i32 0, i32 0
  %94 = load i32, ptr %93, align 8
  %95 = icmp eq i32 %94, 12
  br i1 %95, label %96, label %121

96:                                               ; preds = %90
  %97 = load ptr, ptr %6, align 8
  %98 = getelementptr inbounds %struct.expdesc, ptr %97, i32 0, i32 0
  %99 = load i32, ptr %98, align 8
  %100 = icmp eq i32 %99, 9
  br i1 %100, label %101, label %121

101:                                              ; preds = %96
  %102 = load ptr, ptr %5, align 8
  %103 = getelementptr inbounds %struct.LHS_assign, ptr %102, i32 0, i32 1
  %104 = getelementptr inbounds %struct.expdesc, ptr %103, i32 0, i32 1
  %105 = getelementptr inbounds %struct.anon.11, ptr %104, i32 0, i32 0
  %106 = load i16, ptr %105, align 8
  %107 = sext i16 %106 to i32
  %108 = load ptr, ptr %6, align 8
  %109 = getelementptr inbounds %struct.expdesc, ptr %108, i32 0, i32 1
  %110 = getelementptr inbounds %struct.anon.12, ptr %109, i32 0, i32 0
  %111 = load i8, ptr %110, align 8
  %112 = zext i8 %111 to i32
  %113 = icmp eq i32 %107, %112
  br i1 %113, label %114, label %121

114:                                              ; preds = %101
  store i32 1, ptr %9, align 4
  %115 = load i32, ptr %8, align 4
  %116 = trunc i32 %115 to i16
  %117 = load ptr, ptr %5, align 8
  %118 = getelementptr inbounds %struct.LHS_assign, ptr %117, i32 0, i32 1
  %119 = getelementptr inbounds %struct.expdesc, ptr %118, i32 0, i32 1
  %120 = getelementptr inbounds %struct.anon.11, ptr %119, i32 0, i32 0
  store i16 %116, ptr %120, align 8
  br label %121

121:                                              ; preds = %114, %101, %96, %90
  br label %122

122:                                              ; preds = %121, %64
  br label %123

123:                                              ; preds = %122, %26, %20
  br label %124

124:                                              ; preds = %123
  %125 = load ptr, ptr %5, align 8
  %126 = getelementptr inbounds %struct.LHS_assign, ptr %125, i32 0, i32 0
  %127 = load ptr, ptr %126, align 8
  store ptr %127, ptr %5, align 8
  br label %17, !llvm.loop !31

128:                                              ; preds = %17
  %129 = load i32, ptr %9, align 4
  %130 = icmp ne i32 %129, 0
  br i1 %130, label %131, label %154

131:                                              ; preds = %128
  %132 = load ptr, ptr %6, align 8
  %133 = getelementptr inbounds %struct.expdesc, ptr %132, i32 0, i32 0
  %134 = load i32, ptr %133, align 8
  %135 = icmp eq i32 %134, 9
  br i1 %135, label %136, label %145

136:                                              ; preds = %131
  %137 = load ptr, ptr %7, align 8
  %138 = load i32, ptr %8, align 4
  %139 = load ptr, ptr %6, align 8
  %140 = getelementptr inbounds %struct.expdesc, ptr %139, i32 0, i32 1
  %141 = getelementptr inbounds %struct.anon.12, ptr %140, i32 0, i32 0
  %142 = load i8, ptr %141, align 8
  %143 = zext i8 %142 to i32
  %144 = call i32 @luaK_codeABCk(ptr noundef %137, i32 noundef 0, i32 noundef %138, i32 noundef %143, i32 noundef 0, i32 noundef 0)
  br label %152

145:                                              ; preds = %131
  %146 = load ptr, ptr %7, align 8
  %147 = load i32, ptr %8, align 4
  %148 = load ptr, ptr %6, align 8
  %149 = getelementptr inbounds %struct.expdesc, ptr %148, i32 0, i32 1
  %150 = load i32, ptr %149, align 8
  %151 = call i32 @luaK_codeABCk(ptr noundef %146, i32 noundef 9, i32 noundef %147, i32 noundef %150, i32 noundef 0, i32 noundef 0)
  br label %152

152:                                              ; preds = %145, %136
  %153 = load ptr, ptr %7, align 8
  call void @luaK_reserveregs(ptr noundef %153, i32 noundef 1)
  br label %154

154:                                              ; preds = %152, %128
  ret void
}

declare hidden void @luaK_setoneret(ptr noundef, ptr noundef) #1

declare hidden void @luaK_finish(ptr noundef) #1

declare hidden ptr @luaM_shrinkvector_(ptr noundef, ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

declare hidden void @luaC_step(ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { noreturn }
attributes #7 = { nounwind willreturn memory(read) }

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
!16 = distinct !{!16, !7}
!17 = distinct !{!17, !7}
!18 = distinct !{!18, !7}
!19 = distinct !{!19, !7}
!20 = distinct !{!20, !7}
!21 = distinct !{!21, !7}
!22 = distinct !{!22, !7}
!23 = distinct !{!23, !7}
!24 = distinct !{!24, !7}
!25 = distinct !{!25, !7}
!26 = distinct !{!26, !7}
!27 = distinct !{!27, !7}
!28 = distinct !{!28, !7}
!29 = distinct !{!29, !7}
!30 = distinct !{!30, !7}
!31 = distinct !{!31, !7}
