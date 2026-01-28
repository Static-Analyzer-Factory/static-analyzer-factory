; ModuleID = 'lcode.c'
source_filename = "lcode.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.expdesc = type { i32, %union.anon, i32, i32 }
%union.anon = type { i64 }
%struct.LexState = type { i32, i32, i32, %struct.Token, %struct.Token, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Token = type { i32, %union.SemInfo }
%union.SemInfo = type { double }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.1, [1 x i8] }
%union.anon.1 = type { i64 }
%struct.FuncState = type { ptr, ptr, ptr, ptr, i32, i32, i32, i32, i32, i32, i32, i32, i16, i8, i8, i8, i8, i8 }
%struct.Dyndata = type { %struct.anon.11, %struct.Labellist, %struct.Labellist }
%struct.anon.11 = type { ptr, i32, i32 }
%struct.Labellist = type { ptr, i32, i32 }
%union.Vardesc = type { %struct.anon.10 }
%struct.anon.10 = type { %union.Value, i8, i8, i8, i16, ptr }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.AbsLineInfo = type { i32, i32 }
%struct.anon.0 = type { i8, i16 }
%struct.anon = type { i16, i8 }
%struct.GCObject = type { ptr, i8, i8 }

@.str = private unnamed_addr constant [8 x i8] c"opcodes\00", align 1
@.str.1 = private unnamed_addr constant [48 x i8] c"function or expression needs too many registers\00", align 1
@luaK_prefix.ef = internal constant %struct.expdesc { i32 6, %union.anon zeroinitializer, i32 -1, i32 -1 }, align 8
@previousinstruction.invalidinstruction = internal constant i32 -1, align 4
@.str.2 = private unnamed_addr constant [27 x i8] c"control structure too long\00", align 1
@luaP_opmodes = external hidden constant [83 x i8], align 16
@.str.3 = private unnamed_addr constant [6 x i8] c"lines\00", align 1
@.str.4 = private unnamed_addr constant [10 x i8] c"constants\00", align 1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaK_semerror(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 3
  %7 = getelementptr inbounds %struct.Token, ptr %6, i32 0, i32 0
  store i32 0, ptr %7, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  call void @luaX_syntaxerror(ptr noundef %8, ptr noundef %9) #7
  unreachable
}

; Function Attrs: noreturn
declare hidden void @luaX_syntaxerror(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_exp2const(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 2
  %14 = load i32, ptr %13, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.expdesc, ptr %15, i32 0, i32 3
  %17 = load i32, ptr %16, align 4
  %18 = icmp ne i32 %14, %17
  br i1 %18, label %19, label %20

19:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %77

20:                                               ; preds = %3
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 0
  %23 = load i32, ptr %22, align 8
  switch i32 %23, label %73 [
    i32 3, label %24
    i32 2, label %27
    i32 1, label %30
    i32 7, label %33
    i32 11, label %54
  ]

24:                                               ; preds = %20
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  store i8 1, ptr %26, align 8
  store i32 1, ptr %4, align 4
  br label %77

27:                                               ; preds = %20
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  store i8 17, ptr %29, align 8
  store i32 1, ptr %4, align 4
  br label %77

30:                                               ; preds = %20
  %31 = load ptr, ptr %7, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 1
  store i8 0, ptr %32, align 8
  store i32 1, ptr %4, align 4
  br label %77

33:                                               ; preds = %20
  %34 = load ptr, ptr %7, align 8
  store ptr %34, ptr %8, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 1
  %37 = load ptr, ptr %36, align 8
  store ptr %37, ptr %9, align 8
  %38 = load ptr, ptr %9, align 8
  %39 = load ptr, ptr %8, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  store ptr %38, ptr %40, align 8
  %41 = load ptr, ptr %9, align 8
  %42 = getelementptr inbounds %struct.TString, ptr %41, i32 0, i32 1
  %43 = load i8, ptr %42, align 8
  %44 = zext i8 %43 to i32
  %45 = or i32 %44, 64
  %46 = trunc i32 %45 to i8
  %47 = load ptr, ptr %8, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 1
  store i8 %46, ptr %48, align 8
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.FuncState, ptr %49, i32 0, i32 2
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.LexState, ptr %51, i32 0, i32 6
  %53 = load ptr, ptr %52, align 8
  store i32 1, ptr %4, align 4
  br label %77

54:                                               ; preds = %20
  %55 = load ptr, ptr %7, align 8
  store ptr %55, ptr %10, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = load ptr, ptr %6, align 8
  %58 = call ptr @const2val(ptr noundef %56, ptr noundef %57)
  store ptr %58, ptr %11, align 8
  %59 = load ptr, ptr %10, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  %61 = load ptr, ptr %11, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %60, ptr align 8 %62, i64 8, i1 false)
  %63 = load ptr, ptr %11, align 8
  %64 = getelementptr inbounds %struct.TValue, ptr %63, i32 0, i32 1
  %65 = load i8, ptr %64, align 8
  %66 = load ptr, ptr %10, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 1
  store i8 %65, ptr %67, align 8
  %68 = load ptr, ptr %5, align 8
  %69 = getelementptr inbounds %struct.FuncState, ptr %68, i32 0, i32 2
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.LexState, ptr %70, i32 0, i32 6
  %72 = load ptr, ptr %71, align 8
  store i32 1, ptr %4, align 4
  br label %77

73:                                               ; preds = %20
  %74 = load ptr, ptr %6, align 8
  %75 = load ptr, ptr %7, align 8
  %76 = call i32 @tonumeral(ptr noundef %74, ptr noundef %75)
  store i32 %76, ptr %4, align 4
  br label %77

77:                                               ; preds = %73, %54, %33, %30, %27, %24, %19
  %78 = load i32, ptr %4, align 4
  ret i32 %78
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @const2val(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.FuncState, ptr %5, i32 0, i32 2
  %7 = load ptr, ptr %6, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 10
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Dyndata, ptr %9, i32 0, i32 0
  %11 = getelementptr inbounds %struct.anon.11, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 1
  %15 = load i32, ptr %14, align 8
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds %union.Vardesc, ptr %12, i64 %16
  ret ptr %17
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tonumeral(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 2
  %10 = load i32, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 3
  %13 = load i32, ptr %12, align 4
  %14 = icmp ne i32 %10, %13
  br i1 %14, label %15, label %16

15:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %47

16:                                               ; preds = %2
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.expdesc, ptr %17, i32 0, i32 0
  %19 = load i32, ptr %18, align 8
  switch i32 %19, label %46 [
    i32 6, label %20
    i32 5, label %33
  ]

20:                                               ; preds = %16
  %21 = load ptr, ptr %5, align 8
  %22 = icmp ne ptr %21, null
  br i1 %22, label %23, label %32

23:                                               ; preds = %20
  %24 = load ptr, ptr %5, align 8
  store ptr %24, ptr %6, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.expdesc, ptr %25, i32 0, i32 1
  %27 = load i64, ptr %26, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  store i64 %27, ptr %29, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 3, ptr %31, align 8
  br label %32

32:                                               ; preds = %23, %20
  store i32 1, ptr %3, align 4
  br label %47

33:                                               ; preds = %16
  %34 = load ptr, ptr %5, align 8
  %35 = icmp ne ptr %34, null
  br i1 %35, label %36, label %45

36:                                               ; preds = %33
  %37 = load ptr, ptr %5, align 8
  store ptr %37, ptr %7, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.expdesc, ptr %38, i32 0, i32 1
  %40 = load double, ptr %39, align 8
  %41 = load ptr, ptr %7, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  store double %40, ptr %42, align 8
  %43 = load ptr, ptr %7, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  store i8 19, ptr %44, align 8
  br label %45

45:                                               ; preds = %36, %33
  store i32 1, ptr %3, align 4
  br label %47

46:                                               ; preds = %16
  store i32 0, ptr %3, align 4
  br label %47

47:                                               ; preds = %46, %45, %32, %15
  %48 = load i32, ptr %3, align 4
  ret i32 %48
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_nil(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %11 = load i32, ptr %5, align 4
  %12 = load i32, ptr %6, align 4
  %13 = add nsw i32 %11, %12
  %14 = sub nsw i32 %13, 1
  store i32 %14, ptr %7, align 4
  %15 = load ptr, ptr %4, align 8
  %16 = call ptr @previousinstruction(ptr noundef %15)
  store ptr %16, ptr %8, align 8
  %17 = load ptr, ptr %8, align 8
  %18 = load i32, ptr %17, align 4
  %19 = lshr i32 %18, 0
  %20 = and i32 %19, 127
  %21 = icmp eq i32 %20, 8
  br i1 %21, label %22, label %82

22:                                               ; preds = %3
  %23 = load ptr, ptr %8, align 8
  %24 = load i32, ptr %23, align 4
  %25 = lshr i32 %24, 7
  %26 = and i32 %25, 255
  store i32 %26, ptr %9, align 4
  %27 = load i32, ptr %9, align 4
  %28 = load ptr, ptr %8, align 8
  %29 = load i32, ptr %28, align 4
  %30 = lshr i32 %29, 16
  %31 = and i32 %30, 255
  %32 = add nsw i32 %27, %31
  store i32 %32, ptr %10, align 4
  %33 = load i32, ptr %9, align 4
  %34 = load i32, ptr %5, align 4
  %35 = icmp sle i32 %33, %34
  br i1 %35, label %36, label %41

36:                                               ; preds = %22
  %37 = load i32, ptr %5, align 4
  %38 = load i32, ptr %10, align 4
  %39 = add nsw i32 %38, 1
  %40 = icmp sle i32 %37, %39
  br i1 %40, label %50, label %41

41:                                               ; preds = %36, %22
  %42 = load i32, ptr %5, align 4
  %43 = load i32, ptr %9, align 4
  %44 = icmp sle i32 %42, %43
  br i1 %44, label %45, label %81

45:                                               ; preds = %41
  %46 = load i32, ptr %9, align 4
  %47 = load i32, ptr %7, align 4
  %48 = add nsw i32 %47, 1
  %49 = icmp sle i32 %46, %48
  br i1 %49, label %50, label %81

50:                                               ; preds = %45, %36
  %51 = load i32, ptr %9, align 4
  %52 = load i32, ptr %5, align 4
  %53 = icmp slt i32 %51, %52
  br i1 %53, label %54, label %56

54:                                               ; preds = %50
  %55 = load i32, ptr %9, align 4
  store i32 %55, ptr %5, align 4
  br label %56

56:                                               ; preds = %54, %50
  %57 = load i32, ptr %10, align 4
  %58 = load i32, ptr %7, align 4
  %59 = icmp sgt i32 %57, %58
  br i1 %59, label %60, label %62

60:                                               ; preds = %56
  %61 = load i32, ptr %10, align 4
  store i32 %61, ptr %7, align 4
  br label %62

62:                                               ; preds = %60, %56
  %63 = load ptr, ptr %8, align 8
  %64 = load i32, ptr %63, align 4
  %65 = and i32 %64, -32641
  %66 = load i32, ptr %5, align 4
  %67 = shl i32 %66, 7
  %68 = and i32 %67, 32640
  %69 = or i32 %65, %68
  %70 = load ptr, ptr %8, align 8
  store i32 %69, ptr %70, align 4
  %71 = load ptr, ptr %8, align 8
  %72 = load i32, ptr %71, align 4
  %73 = and i32 %72, -16711681
  %74 = load i32, ptr %7, align 4
  %75 = load i32, ptr %5, align 4
  %76 = sub nsw i32 %74, %75
  %77 = shl i32 %76, 16
  %78 = and i32 %77, 16711680
  %79 = or i32 %73, %78
  %80 = load ptr, ptr %8, align 8
  store i32 %79, ptr %80, align 4
  br label %88

81:                                               ; preds = %45, %41
  br label %82

82:                                               ; preds = %81, %3
  %83 = load ptr, ptr %4, align 8
  %84 = load i32, ptr %5, align 4
  %85 = load i32, ptr %6, align 4
  %86 = sub nsw i32 %85, 1
  %87 = call i32 @luaK_codeABCk(ptr noundef %83, i32 noundef 8, i32 noundef %84, i32 noundef %86, i32 noundef 0, i32 noundef 0)
  br label %88

88:                                               ; preds = %82, %62
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @previousinstruction(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = getelementptr inbounds %struct.FuncState, ptr %4, i32 0, i32 4
  %6 = load i32, ptr %5, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 5
  %9 = load i32, ptr %8, align 4
  %10 = icmp sgt i32 %6, %9
  br i1 %10, label %11, label %23

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Proto, ptr %14, i32 0, i32 16
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 4
  %19 = load i32, ptr %18, align 8
  %20 = sub nsw i32 %19, 1
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds i32, ptr %16, i64 %21
  store ptr %22, ptr %2, align 8
  br label %24

23:                                               ; preds = %1
  store ptr @previousinstruction.invalidinstruction, ptr %2, align 8
  br label %24

24:                                               ; preds = %23, %11
  %25 = load ptr, ptr %2, align 8
  ret ptr %25
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_codeABCk(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store i32 %2, ptr %9, align 4
  store i32 %3, ptr %10, align 4
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %13 = load ptr, ptr %7, align 8
  %14 = load i32, ptr %8, align 4
  %15 = shl i32 %14, 0
  %16 = load i32, ptr %9, align 4
  %17 = shl i32 %16, 7
  %18 = or i32 %15, %17
  %19 = load i32, ptr %10, align 4
  %20 = shl i32 %19, 16
  %21 = or i32 %18, %20
  %22 = load i32, ptr %11, align 4
  %23 = shl i32 %22, 24
  %24 = or i32 %21, %23
  %25 = load i32, ptr %12, align 4
  %26 = shl i32 %25, 15
  %27 = or i32 %24, %26
  %28 = call i32 @luaK_code(ptr noundef %13, i32 noundef %27)
  ret i32 %28
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_concat(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %9 = load i32, ptr %6, align 4
  %10 = icmp eq i32 %9, -1
  br i1 %10, label %11, label %12

11:                                               ; preds = %3
  br label %34

12:                                               ; preds = %3
  %13 = load ptr, ptr %5, align 8
  %14 = load i32, ptr %13, align 4
  %15 = icmp eq i32 %14, -1
  br i1 %15, label %16, label %19

16:                                               ; preds = %12
  %17 = load i32, ptr %6, align 4
  %18 = load ptr, ptr %5, align 8
  store i32 %17, ptr %18, align 4
  br label %33

19:                                               ; preds = %12
  %20 = load ptr, ptr %5, align 8
  %21 = load i32, ptr %20, align 4
  store i32 %21, ptr %7, align 4
  br label %22

22:                                               ; preds = %27, %19
  %23 = load ptr, ptr %4, align 8
  %24 = load i32, ptr %7, align 4
  %25 = call i32 @getjump(ptr noundef %23, i32 noundef %24)
  store i32 %25, ptr %8, align 4
  %26 = icmp ne i32 %25, -1
  br i1 %26, label %27, label %29

27:                                               ; preds = %22
  %28 = load i32, ptr %8, align 4
  store i32 %28, ptr %7, align 4
  br label %22, !llvm.loop !6

29:                                               ; preds = %22
  %30 = load ptr, ptr %4, align 8
  %31 = load i32, ptr %7, align 4
  %32 = load i32, ptr %6, align 4
  call void @fixjump(ptr noundef %30, i32 noundef %31, i32 noundef %32)
  br label %33

33:                                               ; preds = %29, %16
  br label %34

34:                                               ; preds = %11, %33
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getjump(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 16
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %5, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds i32, ptr %11, i64 %13
  %15 = load i32, ptr %14, align 4
  %16 = lshr i32 %15, 7
  %17 = and i32 %16, 33554431
  %18 = sub nsw i32 %17, 16777215
  store i32 %18, ptr %6, align 4
  %19 = load i32, ptr %6, align 4
  %20 = icmp eq i32 %19, -1
  br i1 %20, label %21, label %22

21:                                               ; preds = %2
  store i32 -1, ptr %3, align 4
  br label %27

22:                                               ; preds = %2
  %23 = load i32, ptr %5, align 4
  %24 = add nsw i32 %23, 1
  %25 = load i32, ptr %6, align 4
  %26 = add nsw i32 %24, %25
  store i32 %26, ptr %3, align 4
  br label %27

27:                                               ; preds = %22, %21
  %28 = load i32, ptr %3, align 4
  ret i32 %28
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fixjump(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.Proto, ptr %11, i32 0, i32 16
  %13 = load ptr, ptr %12, align 8
  %14 = load i32, ptr %5, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds i32, ptr %13, i64 %15
  store ptr %16, ptr %7, align 8
  %17 = load i32, ptr %6, align 4
  %18 = load i32, ptr %5, align 4
  %19 = add nsw i32 %18, 1
  %20 = sub nsw i32 %17, %19
  store i32 %20, ptr %8, align 4
  %21 = load i32, ptr %8, align 4
  %22 = icmp sle i32 -16777215, %21
  br i1 %22, label %23, label %26

23:                                               ; preds = %3
  %24 = load i32, ptr %8, align 4
  %25 = icmp sle i32 %24, 16777216
  br i1 %25, label %30, label %26

26:                                               ; preds = %23, %3
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.FuncState, ptr %27, i32 0, i32 2
  %29 = load ptr, ptr %28, align 8
  call void @luaX_syntaxerror(ptr noundef %29, ptr noundef @.str.2) #7
  unreachable

30:                                               ; preds = %23
  %31 = load ptr, ptr %7, align 8
  %32 = load i32, ptr %31, align 4
  %33 = and i32 %32, 127
  %34 = load i32, ptr %8, align 4
  %35 = add nsw i32 %34, 16777215
  %36 = shl i32 %35, 7
  %37 = and i32 %36, -128
  %38 = or i32 %33, %37
  %39 = load ptr, ptr %7, align 8
  store i32 %38, ptr %39, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_jump(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @codesJ(ptr noundef %3, i32 noundef 56, i32 noundef -1, i32 noundef 0)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @codesJ(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %10 = load i32, ptr %7, align 4
  %11 = add nsw i32 %10, 16777215
  store i32 %11, ptr %9, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = shl i32 %13, 0
  %15 = load i32, ptr %9, align 4
  %16 = shl i32 %15, 7
  %17 = or i32 %14, %16
  %18 = load i32, ptr %8, align 4
  %19 = shl i32 %18, 15
  %20 = or i32 %17, %19
  %21 = call i32 @luaK_code(ptr noundef %12, i32 noundef %20)
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_ret(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %8 = load i32, ptr %6, align 4
  switch i32 %8, label %11 [
    i32 0, label %9
    i32 1, label %10
  ]

9:                                                ; preds = %3
  store i32 71, ptr %7, align 4
  br label %12

10:                                               ; preds = %3
  store i32 72, ptr %7, align 4
  br label %12

11:                                               ; preds = %3
  store i32 70, ptr %7, align 4
  br label %12

12:                                               ; preds = %11, %10, %9
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %7, align 4
  %15 = load i32, ptr %5, align 4
  %16 = load i32, ptr %6, align 4
  %17 = add nsw i32 %16, 1
  %18 = call i32 @luaK_codeABCk(ptr noundef %13, i32 noundef %14, i32 noundef %15, i32 noundef %17, i32 noundef 0, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_getlabel(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.FuncState, ptr %3, i32 0, i32 4
  %5 = load i32, ptr %4, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.FuncState, ptr %6, i32 0, i32 5
  store i32 %5, ptr %7, align 4
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 4
  %10 = load i32, ptr %9, align 8
  ret i32 %10
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_patchlist(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = load i32, ptr %6, align 4
  %10 = load i32, ptr %6, align 4
  call void @patchlistaux(ptr noundef %7, i32 noundef %8, i32 noundef %9, i32 noundef 255, i32 noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @patchlistaux(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  br label %12

12:                                               ; preds = %32, %5
  %13 = load i32, ptr %7, align 4
  %14 = icmp ne i32 %13, -1
  br i1 %14, label %15, label %34

15:                                               ; preds = %12
  %16 = load ptr, ptr %6, align 8
  %17 = load i32, ptr %7, align 4
  %18 = call i32 @getjump(ptr noundef %16, i32 noundef %17)
  store i32 %18, ptr %11, align 4
  %19 = load ptr, ptr %6, align 8
  %20 = load i32, ptr %7, align 4
  %21 = load i32, ptr %9, align 4
  %22 = call i32 @patchtestreg(ptr noundef %19, i32 noundef %20, i32 noundef %21)
  %23 = icmp ne i32 %22, 0
  br i1 %23, label %24, label %28

24:                                               ; preds = %15
  %25 = load ptr, ptr %6, align 8
  %26 = load i32, ptr %7, align 4
  %27 = load i32, ptr %8, align 4
  call void @fixjump(ptr noundef %25, i32 noundef %26, i32 noundef %27)
  br label %32

28:                                               ; preds = %15
  %29 = load ptr, ptr %6, align 8
  %30 = load i32, ptr %7, align 4
  %31 = load i32, ptr %10, align 4
  call void @fixjump(ptr noundef %29, i32 noundef %30, i32 noundef %31)
  br label %32

32:                                               ; preds = %28, %24
  %33 = load i32, ptr %11, align 4
  store i32 %33, ptr %7, align 4
  br label %12, !llvm.loop !8

34:                                               ; preds = %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_patchtohere(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @luaK_getlabel(ptr noundef %6)
  store i32 %7, ptr %5, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = load i32, ptr %4, align 4
  %10 = load i32, ptr %5, align 4
  call void @luaK_patchlist(ptr noundef %8, i32 noundef %9, i32 noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_code(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.FuncState, ptr %6, i32 0, i32 0
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.FuncState, ptr %9, i32 0, i32 2
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.Proto, ptr %14, i32 0, i32 16
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 4
  %19 = load i32, ptr %18, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.Proto, ptr %20, i32 0, i32 8
  %22 = call ptr @luaM_growaux_(ptr noundef %13, ptr noundef %16, i32 noundef %19, ptr noundef %21, i32 noundef 4, i32 noundef 2147483647, ptr noundef @.str)
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Proto, ptr %23, i32 0, i32 16
  store ptr %22, ptr %24, align 8
  %25 = load i32, ptr %4, align 4
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.Proto, ptr %26, i32 0, i32 16
  %28 = load ptr, ptr %27, align 8
  %29 = load ptr, ptr %3, align 8
  %30 = getelementptr inbounds %struct.FuncState, ptr %29, i32 0, i32 4
  %31 = load i32, ptr %30, align 8
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %30, align 8
  %33 = sext i32 %31 to i64
  %34 = getelementptr inbounds i32, ptr %28, i64 %33
  store i32 %25, ptr %34, align 4
  %35 = load ptr, ptr %3, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = load ptr, ptr %3, align 8
  %38 = getelementptr inbounds %struct.FuncState, ptr %37, i32 0, i32 2
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.LexState, ptr %39, i32 0, i32 2
  %41 = load i32, ptr %40, align 8
  call void @savelineinfo(ptr noundef %35, ptr noundef %36, i32 noundef %41)
  %42 = load ptr, ptr %3, align 8
  %43 = getelementptr inbounds %struct.FuncState, ptr %42, i32 0, i32 4
  %44 = load i32, ptr %43, align 8
  %45 = sub nsw i32 %44, 1
  ret i32 %45
}

declare hidden ptr @luaM_growaux_(ptr noundef, ptr noundef, i32 noundef, ptr noundef, i32 noundef, i32 noundef, ptr noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @savelineinfo(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %9 = load i32, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 6
  %12 = load i32, ptr %11, align 8
  %13 = sub nsw i32 %9, %12
  store i32 %13, ptr %7, align 4
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 4
  %16 = load i32, ptr %15, align 8
  %17 = sub nsw i32 %16, 1
  store i32 %17, ptr %8, align 4
  %18 = load i32, ptr %7, align 4
  %19 = call i32 @llvm.abs.i32(i32 %18, i1 true)
  %20 = icmp sge i32 %19, 128
  br i1 %20, label %28, label %21

21:                                               ; preds = %3
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 16
  %24 = load i8, ptr %23, align 1
  %25 = add i8 %24, 1
  store i8 %25, ptr %23, align 1
  %26 = zext i8 %24 to i32
  %27 = icmp sge i32 %26, 128
  br i1 %27, label %28, label %68

28:                                               ; preds = %21, %3
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.FuncState, ptr %29, i32 0, i32 2
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.LexState, ptr %31, i32 0, i32 6
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 20
  %36 = load ptr, ptr %35, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.FuncState, ptr %37, i32 0, i32 9
  %39 = load i32, ptr %38, align 4
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.Proto, ptr %40, i32 0, i32 12
  %42 = call ptr @luaM_growaux_(ptr noundef %33, ptr noundef %36, i32 noundef %39, ptr noundef %41, i32 noundef 8, i32 noundef 2147483647, ptr noundef @.str.3)
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.Proto, ptr %43, i32 0, i32 20
  store ptr %42, ptr %44, align 8
  %45 = load i32, ptr %8, align 4
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.Proto, ptr %46, i32 0, i32 20
  %48 = load ptr, ptr %47, align 8
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.FuncState, ptr %49, i32 0, i32 9
  %51 = load i32, ptr %50, align 4
  %52 = sext i32 %51 to i64
  %53 = getelementptr inbounds %struct.AbsLineInfo, ptr %48, i64 %52
  %54 = getelementptr inbounds %struct.AbsLineInfo, ptr %53, i32 0, i32 0
  store i32 %45, ptr %54, align 4
  %55 = load i32, ptr %6, align 4
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.Proto, ptr %56, i32 0, i32 20
  %58 = load ptr, ptr %57, align 8
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.FuncState, ptr %59, i32 0, i32 9
  %61 = load i32, ptr %60, align 4
  %62 = add nsw i32 %61, 1
  store i32 %62, ptr %60, align 4
  %63 = sext i32 %61 to i64
  %64 = getelementptr inbounds %struct.AbsLineInfo, ptr %58, i64 %63
  %65 = getelementptr inbounds %struct.AbsLineInfo, ptr %64, i32 0, i32 1
  store i32 %55, ptr %65, align 4
  store i32 -128, ptr %7, align 4
  %66 = load ptr, ptr %4, align 8
  %67 = getelementptr inbounds %struct.FuncState, ptr %66, i32 0, i32 16
  store i8 1, ptr %67, align 1
  br label %68

68:                                               ; preds = %28, %21
  %69 = load ptr, ptr %4, align 8
  %70 = getelementptr inbounds %struct.FuncState, ptr %69, i32 0, i32 2
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %struct.LexState, ptr %71, i32 0, i32 6
  %73 = load ptr, ptr %72, align 8
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.Proto, ptr %74, i32 0, i32 19
  %76 = load ptr, ptr %75, align 8
  %77 = load i32, ptr %8, align 4
  %78 = load ptr, ptr %5, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 9
  %80 = call ptr @luaM_growaux_(ptr noundef %73, ptr noundef %76, i32 noundef %77, ptr noundef %79, i32 noundef 1, i32 noundef 2147483647, ptr noundef @.str)
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds %struct.Proto, ptr %81, i32 0, i32 19
  store ptr %80, ptr %82, align 8
  %83 = load i32, ptr %7, align 4
  %84 = trunc i32 %83 to i8
  %85 = load ptr, ptr %5, align 8
  %86 = getelementptr inbounds %struct.Proto, ptr %85, i32 0, i32 19
  %87 = load ptr, ptr %86, align 8
  %88 = load i32, ptr %8, align 4
  %89 = sext i32 %88 to i64
  %90 = getelementptr inbounds i8, ptr %87, i64 %89
  store i8 %84, ptr %90, align 1
  %91 = load i32, ptr %6, align 4
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.FuncState, ptr %92, i32 0, i32 6
  store i32 %91, ptr %93, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_codeABx(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #2 {
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
  %11 = shl i32 %10, 0
  %12 = load i32, ptr %7, align 4
  %13 = shl i32 %12, 7
  %14 = or i32 %11, %13
  %15 = load i32, ptr %8, align 4
  %16 = shl i32 %15, 15
  %17 = or i32 %14, %16
  %18 = call i32 @luaK_code(ptr noundef %9, i32 noundef %17)
  ret i32 %18
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_checkstack(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.FuncState, ptr %6, i32 0, i32 15
  %8 = load i8, ptr %7, align 4
  %9 = zext i8 %8 to i32
  %10 = load i32, ptr %4, align 4
  %11 = add nsw i32 %9, %10
  store i32 %11, ptr %5, align 4
  %12 = load i32, ptr %5, align 4
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.Proto, ptr %15, i32 0, i32 5
  %17 = load i8, ptr %16, align 4
  %18 = zext i8 %17 to i32
  %19 = icmp sgt i32 %12, %18
  br i1 %19, label %20, label %34

20:                                               ; preds = %2
  %21 = load i32, ptr %5, align 4
  %22 = icmp sge i32 %21, 255
  br i1 %22, label %23, label %27

23:                                               ; preds = %20
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.FuncState, ptr %24, i32 0, i32 2
  %26 = load ptr, ptr %25, align 8
  call void @luaX_syntaxerror(ptr noundef %26, ptr noundef @.str.1) #7
  unreachable

27:                                               ; preds = %20
  %28 = load i32, ptr %5, align 4
  %29 = trunc i32 %28 to i8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.FuncState, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.Proto, ptr %32, i32 0, i32 5
  store i8 %29, ptr %33, align 4
  br label %34

34:                                               ; preds = %27, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_reserveregs(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  call void @luaK_checkstack(ptr noundef %5, i32 noundef %6)
  %7 = load i32, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 15
  %10 = load i8, ptr %9, align 4
  %11 = zext i8 %10 to i32
  %12 = add nsw i32 %11, %7
  %13 = trunc i32 %12 to i8
  store i8 %13, ptr %9, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_int(ptr noundef %0, i32 noundef %1, i64 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %7 = load i64, ptr %6, align 8
  %8 = call i32 @fitsBx(i64 noundef %7)
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %16

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = load i64, ptr %6, align 8
  %14 = trunc i64 %13 to i32
  %15 = call i32 @codeAsBx(ptr noundef %11, i32 noundef 1, i32 noundef %12, i32 noundef %14)
  br label %23

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %5, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = load i64, ptr %6, align 8
  %21 = call i32 @luaK_intK(ptr noundef %19, i64 noundef %20)
  %22 = call i32 @luaK_codek(ptr noundef %17, i32 noundef %18, i32 noundef %21)
  br label %23

23:                                               ; preds = %16, %10
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @fitsBx(i64 noundef %0) #2 {
  %2 = alloca i64, align 8
  store i64 %0, ptr %2, align 8
  %3 = load i64, ptr %2, align 8
  %4 = icmp sle i64 -65535, %3
  br i1 %4, label %5, label %8

5:                                                ; preds = %1
  %6 = load i64, ptr %2, align 8
  %7 = icmp sle i64 %6, 65536
  br label %8

8:                                                ; preds = %5, %1
  %9 = phi i1 [ false, %1 ], [ %7, %5 ]
  %10 = zext i1 %9 to i32
  ret i32 %10
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @codeAsBx(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %10 = load i32, ptr %8, align 4
  %11 = add nsw i32 %10, 65535
  store i32 %11, ptr %9, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = shl i32 %13, 0
  %15 = load i32, ptr %7, align 4
  %16 = shl i32 %15, 7
  %17 = or i32 %14, %16
  %18 = load i32, ptr %9, align 4
  %19 = shl i32 %18, 15
  %20 = or i32 %17, %19
  %21 = call i32 @luaK_code(ptr noundef %12, i32 noundef %20)
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaK_codek(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %9 = load i32, ptr %7, align 4
  %10 = icmp sle i32 %9, 131071
  br i1 %10, label %11, label %16

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = load i32, ptr %7, align 4
  %15 = call i32 @luaK_codeABx(ptr noundef %12, i32 noundef 3, i32 noundef %13, i32 noundef %14)
  store i32 %15, ptr %4, align 4
  br label %24

16:                                               ; preds = %3
  %17 = load ptr, ptr %5, align 8
  %18 = load i32, ptr %6, align 4
  %19 = call i32 @luaK_codeABx(ptr noundef %17, i32 noundef 4, i32 noundef %18, i32 noundef 0)
  store i32 %19, ptr %8, align 4
  %20 = load ptr, ptr %5, align 8
  %21 = load i32, ptr %7, align 4
  %22 = call i32 @codeextraarg(ptr noundef %20, i32 noundef %21)
  %23 = load i32, ptr %8, align 4
  store i32 %23, ptr %4, align 4
  br label %24

24:                                               ; preds = %16, %11
  %25 = load i32, ptr %4, align 4
  ret i32 %25
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaK_intK(ptr noundef %0, i64 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca %struct.TValue, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  store ptr %5, ptr %6, align 8
  %7 = load i64, ptr %4, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 0
  store i64 %7, ptr %9, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 1
  store i8 3, ptr %11, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = call i32 @addk(ptr noundef %12, ptr noundef %5, ptr noundef %5)
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_setreturns(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.Proto, ptr %10, i32 0, i32 16
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 1
  %15 = load i32, ptr %14, align 8
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds i32, ptr %12, i64 %16
  store ptr %17, ptr %7, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 18
  br i1 %21, label %22, label %32

22:                                               ; preds = %3
  %23 = load ptr, ptr %7, align 8
  %24 = load i32, ptr %23, align 4
  %25 = and i32 %24, 16777215
  %26 = load i32, ptr %6, align 4
  %27 = add nsw i32 %26, 1
  %28 = shl i32 %27, 24
  %29 = and i32 %28, -16777216
  %30 = or i32 %25, %29
  %31 = load ptr, ptr %7, align 8
  store i32 %30, ptr %31, align 4
  br label %54

32:                                               ; preds = %3
  %33 = load ptr, ptr %7, align 8
  %34 = load i32, ptr %33, align 4
  %35 = and i32 %34, 16777215
  %36 = load i32, ptr %6, align 4
  %37 = add nsw i32 %36, 1
  %38 = shl i32 %37, 24
  %39 = and i32 %38, -16777216
  %40 = or i32 %35, %39
  %41 = load ptr, ptr %7, align 8
  store i32 %40, ptr %41, align 4
  %42 = load ptr, ptr %7, align 8
  %43 = load i32, ptr %42, align 4
  %44 = and i32 %43, -32641
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.FuncState, ptr %45, i32 0, i32 15
  %47 = load i8, ptr %46, align 4
  %48 = zext i8 %47 to i32
  %49 = shl i32 %48, 7
  %50 = and i32 %49, 32640
  %51 = or i32 %44, %50
  %52 = load ptr, ptr %7, align 8
  store i32 %51, ptr %52, align 4
  %53 = load ptr, ptr %4, align 8
  call void @luaK_reserveregs(ptr noundef %53, i32 noundef 1)
  br label %54

54:                                               ; preds = %32, %22
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_setoneret(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  %8 = icmp eq i32 %7, 18
  br i1 %8, label %9, label %27

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 0
  store i32 8, ptr %11, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Proto, ptr %14, i32 0, i32 16
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.expdesc, ptr %17, i32 0, i32 1
  %19 = load i32, ptr %18, align 8
  %20 = sext i32 %19 to i64
  %21 = getelementptr inbounds i32, ptr %16, i64 %20
  %22 = load i32, ptr %21, align 4
  %23 = lshr i32 %22, 7
  %24 = and i32 %23, 255
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.expdesc, ptr %25, i32 0, i32 1
  store i32 %24, ptr %26, align 8
  br label %59

27:                                               ; preds = %2
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 0
  %30 = load i32, ptr %29, align 8
  %31 = icmp eq i32 %30, 19
  br i1 %31, label %32, label %58

32:                                               ; preds = %27
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.FuncState, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.Proto, ptr %35, i32 0, i32 16
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.expdesc, ptr %38, i32 0, i32 1
  %40 = load i32, ptr %39, align 8
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds i32, ptr %37, i64 %41
  %43 = load i32, ptr %42, align 4
  %44 = and i32 %43, 16777215
  %45 = or i32 %44, 33554432
  %46 = load ptr, ptr %3, align 8
  %47 = getelementptr inbounds %struct.FuncState, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  %49 = getelementptr inbounds %struct.Proto, ptr %48, i32 0, i32 16
  %50 = load ptr, ptr %49, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.expdesc, ptr %51, i32 0, i32 1
  %53 = load i32, ptr %52, align 8
  %54 = sext i32 %53 to i64
  %55 = getelementptr inbounds i32, ptr %50, i64 %54
  store i32 %45, ptr %55, align 4
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.expdesc, ptr %56, i32 0, i32 0
  store i32 17, ptr %57, align 8
  br label %58

58:                                               ; preds = %32, %27
  br label %59

59:                                               ; preds = %58, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_dischargevars(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  switch i32 %8, label %129 [
    i32 11, label %9
    i32 9, label %14
    i32 10, label %25
    i32 13, label %35
    i32 14, label %52
    i32 15, label %75
    i32 12, label %98
    i32 19, label %126
    i32 18, label %126
  ]

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @const2val(ptr noundef %10, ptr noundef %11)
  %13 = load ptr, ptr %4, align 8
  call void @const2exp(ptr noundef %12, ptr noundef %13)
  br label %130

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.expdesc, ptr %15, i32 0, i32 1
  %17 = getelementptr inbounds %struct.anon.0, ptr %16, i32 0, i32 0
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  store i32 %19, ptr %5, align 4
  %20 = load i32, ptr %5, align 4
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 1
  store i32 %20, ptr %22, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.expdesc, ptr %23, i32 0, i32 0
  store i32 8, ptr %24, align 8
  br label %130

25:                                               ; preds = %2
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.expdesc, ptr %27, i32 0, i32 1
  %29 = load i32, ptr %28, align 8
  %30 = call i32 @luaK_codeABCk(ptr noundef %26, i32 noundef 9, i32 noundef 0, i32 noundef %29, i32 noundef 0, i32 noundef 0)
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.expdesc, ptr %31, i32 0, i32 1
  store i32 %30, ptr %32, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.expdesc, ptr %33, i32 0, i32 0
  store i32 17, ptr %34, align 8
  br label %130

35:                                               ; preds = %2
  %36 = load ptr, ptr %3, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.expdesc, ptr %37, i32 0, i32 1
  %39 = getelementptr inbounds %struct.anon, ptr %38, i32 0, i32 1
  %40 = load i8, ptr %39, align 2
  %41 = zext i8 %40 to i32
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.expdesc, ptr %42, i32 0, i32 1
  %44 = getelementptr inbounds %struct.anon, ptr %43, i32 0, i32 0
  %45 = load i16, ptr %44, align 8
  %46 = sext i16 %45 to i32
  %47 = call i32 @luaK_codeABCk(ptr noundef %36, i32 noundef 11, i32 noundef 0, i32 noundef %41, i32 noundef %46, i32 noundef 0)
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.expdesc, ptr %48, i32 0, i32 1
  store i32 %47, ptr %49, align 8
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.expdesc, ptr %50, i32 0, i32 0
  store i32 17, ptr %51, align 8
  br label %130

52:                                               ; preds = %2
  %53 = load ptr, ptr %3, align 8
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.expdesc, ptr %54, i32 0, i32 1
  %56 = getelementptr inbounds %struct.anon, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 2
  %58 = zext i8 %57 to i32
  call void @freereg(ptr noundef %53, i32 noundef %58)
  %59 = load ptr, ptr %3, align 8
  %60 = load ptr, ptr %4, align 8
  %61 = getelementptr inbounds %struct.expdesc, ptr %60, i32 0, i32 1
  %62 = getelementptr inbounds %struct.anon, ptr %61, i32 0, i32 1
  %63 = load i8, ptr %62, align 2
  %64 = zext i8 %63 to i32
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.expdesc, ptr %65, i32 0, i32 1
  %67 = getelementptr inbounds %struct.anon, ptr %66, i32 0, i32 0
  %68 = load i16, ptr %67, align 8
  %69 = sext i16 %68 to i32
  %70 = call i32 @luaK_codeABCk(ptr noundef %59, i32 noundef 13, i32 noundef 0, i32 noundef %64, i32 noundef %69, i32 noundef 0)
  %71 = load ptr, ptr %4, align 8
  %72 = getelementptr inbounds %struct.expdesc, ptr %71, i32 0, i32 1
  store i32 %70, ptr %72, align 8
  %73 = load ptr, ptr %4, align 8
  %74 = getelementptr inbounds %struct.expdesc, ptr %73, i32 0, i32 0
  store i32 17, ptr %74, align 8
  br label %130

75:                                               ; preds = %2
  %76 = load ptr, ptr %3, align 8
  %77 = load ptr, ptr %4, align 8
  %78 = getelementptr inbounds %struct.expdesc, ptr %77, i32 0, i32 1
  %79 = getelementptr inbounds %struct.anon, ptr %78, i32 0, i32 1
  %80 = load i8, ptr %79, align 2
  %81 = zext i8 %80 to i32
  call void @freereg(ptr noundef %76, i32 noundef %81)
  %82 = load ptr, ptr %3, align 8
  %83 = load ptr, ptr %4, align 8
  %84 = getelementptr inbounds %struct.expdesc, ptr %83, i32 0, i32 1
  %85 = getelementptr inbounds %struct.anon, ptr %84, i32 0, i32 1
  %86 = load i8, ptr %85, align 2
  %87 = zext i8 %86 to i32
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds %struct.expdesc, ptr %88, i32 0, i32 1
  %90 = getelementptr inbounds %struct.anon, ptr %89, i32 0, i32 0
  %91 = load i16, ptr %90, align 8
  %92 = sext i16 %91 to i32
  %93 = call i32 @luaK_codeABCk(ptr noundef %82, i32 noundef 14, i32 noundef 0, i32 noundef %87, i32 noundef %92, i32 noundef 0)
  %94 = load ptr, ptr %4, align 8
  %95 = getelementptr inbounds %struct.expdesc, ptr %94, i32 0, i32 1
  store i32 %93, ptr %95, align 8
  %96 = load ptr, ptr %4, align 8
  %97 = getelementptr inbounds %struct.expdesc, ptr %96, i32 0, i32 0
  store i32 17, ptr %97, align 8
  br label %130

98:                                               ; preds = %2
  %99 = load ptr, ptr %3, align 8
  %100 = load ptr, ptr %4, align 8
  %101 = getelementptr inbounds %struct.expdesc, ptr %100, i32 0, i32 1
  %102 = getelementptr inbounds %struct.anon, ptr %101, i32 0, i32 1
  %103 = load i8, ptr %102, align 2
  %104 = zext i8 %103 to i32
  %105 = load ptr, ptr %4, align 8
  %106 = getelementptr inbounds %struct.expdesc, ptr %105, i32 0, i32 1
  %107 = getelementptr inbounds %struct.anon, ptr %106, i32 0, i32 0
  %108 = load i16, ptr %107, align 8
  %109 = sext i16 %108 to i32
  call void @freeregs(ptr noundef %99, i32 noundef %104, i32 noundef %109)
  %110 = load ptr, ptr %3, align 8
  %111 = load ptr, ptr %4, align 8
  %112 = getelementptr inbounds %struct.expdesc, ptr %111, i32 0, i32 1
  %113 = getelementptr inbounds %struct.anon, ptr %112, i32 0, i32 1
  %114 = load i8, ptr %113, align 2
  %115 = zext i8 %114 to i32
  %116 = load ptr, ptr %4, align 8
  %117 = getelementptr inbounds %struct.expdesc, ptr %116, i32 0, i32 1
  %118 = getelementptr inbounds %struct.anon, ptr %117, i32 0, i32 0
  %119 = load i16, ptr %118, align 8
  %120 = sext i16 %119 to i32
  %121 = call i32 @luaK_codeABCk(ptr noundef %110, i32 noundef 12, i32 noundef 0, i32 noundef %115, i32 noundef %120, i32 noundef 0)
  %122 = load ptr, ptr %4, align 8
  %123 = getelementptr inbounds %struct.expdesc, ptr %122, i32 0, i32 1
  store i32 %121, ptr %123, align 8
  %124 = load ptr, ptr %4, align 8
  %125 = getelementptr inbounds %struct.expdesc, ptr %124, i32 0, i32 0
  store i32 17, ptr %125, align 8
  br label %130

126:                                              ; preds = %2, %2
  %127 = load ptr, ptr %3, align 8
  %128 = load ptr, ptr %4, align 8
  call void @luaK_setoneret(ptr noundef %127, ptr noundef %128)
  br label %130

129:                                              ; preds = %2
  br label %130

130:                                              ; preds = %129, %126, %98, %75, %52, %35, %25, %14, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @const2exp(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.TValue, ptr %5, i32 0, i32 1
  %7 = load i8, ptr %6, align 8
  %8 = zext i8 %7 to i32
  %9 = and i32 %8, 63
  switch i32 %9, label %43 [
    i32 3, label %10
    i32 19, label %18
    i32 1, label %26
    i32 17, label %29
    i32 0, label %32
    i32 4, label %35
    i32 20, label %35
  ]

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 0
  store i32 6, ptr %12, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 0
  %15 = load i64, ptr %14, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 1
  store i64 %15, ptr %17, align 8
  br label %44

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.expdesc, ptr %19, i32 0, i32 0
  store i32 5, ptr %20, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 0
  %23 = load double, ptr %22, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.expdesc, ptr %24, i32 0, i32 1
  store double %23, ptr %25, align 8
  br label %44

26:                                               ; preds = %2
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.expdesc, ptr %27, i32 0, i32 0
  store i32 3, ptr %28, align 8
  br label %44

29:                                               ; preds = %2
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 0
  store i32 2, ptr %31, align 8
  br label %44

32:                                               ; preds = %2
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.expdesc, ptr %33, i32 0, i32 0
  store i32 1, ptr %34, align 8
  br label %44

35:                                               ; preds = %2, %2
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.expdesc, ptr %36, i32 0, i32 0
  store i32 7, ptr %37, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  %40 = load ptr, ptr %39, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.expdesc, ptr %41, i32 0, i32 1
  store ptr %40, ptr %42, align 8
  br label %44

43:                                               ; preds = %2
  br label %44

44:                                               ; preds = %43, %35, %32, %29, %26, %18, %10
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freereg(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @luaY_nvarstack(ptr noundef %6)
  %8 = icmp sge i32 %5, %7
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.FuncState, ptr %10, i32 0, i32 15
  %12 = load i8, ptr %11, align 4
  %13 = add i8 %12, -1
  store i8 %13, ptr %11, align 4
  br label %14

14:                                               ; preds = %9, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeregs(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load i32, ptr %5, align 4
  %8 = load i32, ptr %6, align 4
  %9 = icmp sgt i32 %7, %8
  br i1 %9, label %10, label %15

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  call void @freereg(ptr noundef %11, i32 noundef %12)
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %6, align 4
  call void @freereg(ptr noundef %13, i32 noundef %14)
  br label %20

15:                                               ; preds = %3
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %6, align 4
  call void @freereg(ptr noundef %16, i32 noundef %17)
  %18 = load ptr, ptr %4, align 8
  %19 = load i32, ptr %5, align 4
  call void @freereg(ptr noundef %18, i32 noundef %19)
  br label %20

20:                                               ; preds = %15, %10
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_exp2nextreg(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  call void @luaK_dischargevars(ptr noundef %5, ptr noundef %6)
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %4, align 8
  call void @freeexp(ptr noundef %7, ptr noundef %8)
  %9 = load ptr, ptr %3, align 8
  call void @luaK_reserveregs(ptr noundef %9, i32 noundef 1)
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 15
  %14 = load i8, ptr %13, align 4
  %15 = zext i8 %14 to i32
  %16 = sub nsw i32 %15, 1
  call void @exp2reg(ptr noundef %10, ptr noundef %11, i32 noundef %16)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeexp(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  %8 = icmp eq i32 %7, 8
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 8
  call void @freereg(ptr noundef %10, i32 noundef %13)
  br label %14

14:                                               ; preds = %9, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @exp2reg(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  call void @discharge2reg(ptr noundef %11, ptr noundef %12, i32 noundef %13)
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  %17 = icmp eq i32 %16, 16
  br i1 %17, label %18, label %25

18:                                               ; preds = %3
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.expdesc, ptr %20, i32 0, i32 2
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.expdesc, ptr %22, i32 0, i32 1
  %24 = load i32, ptr %23, align 8
  call void @luaK_concat(ptr noundef %19, ptr noundef %21, i32 noundef %24)
  br label %25

25:                                               ; preds = %18, %3
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.expdesc, ptr %26, i32 0, i32 2
  %28 = load i32, ptr %27, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.expdesc, ptr %29, i32 0, i32 3
  %31 = load i32, ptr %30, align 4
  %32 = icmp ne i32 %28, %31
  br i1 %32, label %33, label %83

33:                                               ; preds = %25
  store i32 -1, ptr %8, align 4
  store i32 -1, ptr %9, align 4
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 2
  %37 = load i32, ptr %36, align 8
  %38 = call i32 @need_value(ptr noundef %34, i32 noundef %37)
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %47, label %40

40:                                               ; preds = %33
  %41 = load ptr, ptr %4, align 8
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.expdesc, ptr %42, i32 0, i32 3
  %44 = load i32, ptr %43, align 4
  %45 = call i32 @need_value(ptr noundef %41, i32 noundef %44)
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %66

47:                                               ; preds = %40, %33
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.expdesc, ptr %48, i32 0, i32 0
  %50 = load i32, ptr %49, align 8
  %51 = icmp eq i32 %50, 16
  br i1 %51, label %52, label %53

52:                                               ; preds = %47
  br label %56

53:                                               ; preds = %47
  %54 = load ptr, ptr %4, align 8
  %55 = call i32 @luaK_jump(ptr noundef %54)
  br label %56

56:                                               ; preds = %53, %52
  %57 = phi i32 [ -1, %52 ], [ %55, %53 ]
  store i32 %57, ptr %10, align 4
  %58 = load ptr, ptr %4, align 8
  %59 = load i32, ptr %6, align 4
  %60 = call i32 @code_loadbool(ptr noundef %58, i32 noundef %59, i32 noundef 6)
  store i32 %60, ptr %8, align 4
  %61 = load ptr, ptr %4, align 8
  %62 = load i32, ptr %6, align 4
  %63 = call i32 @code_loadbool(ptr noundef %61, i32 noundef %62, i32 noundef 7)
  store i32 %63, ptr %9, align 4
  %64 = load ptr, ptr %4, align 8
  %65 = load i32, ptr %10, align 4
  call void @luaK_patchtohere(ptr noundef %64, i32 noundef %65)
  br label %66

66:                                               ; preds = %56, %40
  %67 = load ptr, ptr %4, align 8
  %68 = call i32 @luaK_getlabel(ptr noundef %67)
  store i32 %68, ptr %7, align 4
  %69 = load ptr, ptr %4, align 8
  %70 = load ptr, ptr %5, align 8
  %71 = getelementptr inbounds %struct.expdesc, ptr %70, i32 0, i32 3
  %72 = load i32, ptr %71, align 4
  %73 = load i32, ptr %7, align 4
  %74 = load i32, ptr %6, align 4
  %75 = load i32, ptr %8, align 4
  call void @patchlistaux(ptr noundef %69, i32 noundef %72, i32 noundef %73, i32 noundef %74, i32 noundef %75)
  %76 = load ptr, ptr %4, align 8
  %77 = load ptr, ptr %5, align 8
  %78 = getelementptr inbounds %struct.expdesc, ptr %77, i32 0, i32 2
  %79 = load i32, ptr %78, align 8
  %80 = load i32, ptr %7, align 4
  %81 = load i32, ptr %6, align 4
  %82 = load i32, ptr %9, align 4
  call void @patchlistaux(ptr noundef %76, i32 noundef %79, i32 noundef %80, i32 noundef %81, i32 noundef %82)
  br label %83

83:                                               ; preds = %66, %25
  %84 = load ptr, ptr %5, align 8
  %85 = getelementptr inbounds %struct.expdesc, ptr %84, i32 0, i32 2
  store i32 -1, ptr %85, align 8
  %86 = load ptr, ptr %5, align 8
  %87 = getelementptr inbounds %struct.expdesc, ptr %86, i32 0, i32 3
  store i32 -1, ptr %87, align 4
  %88 = load i32, ptr %6, align 4
  %89 = load ptr, ptr %5, align 8
  %90 = getelementptr inbounds %struct.expdesc, ptr %89, i32 0, i32 1
  store i32 %88, ptr %90, align 8
  %91 = load ptr, ptr %5, align 8
  %92 = getelementptr inbounds %struct.expdesc, ptr %91, i32 0, i32 0
  store i32 8, ptr %92, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaK_exp2anyreg(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %5, align 8
  call void @luaK_dischargevars(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  %11 = icmp eq i32 %10, 8
  br i1 %11, label %12, label %41

12:                                               ; preds = %2
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 2
  %15 = load i32, ptr %14, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 3
  %18 = load i32, ptr %17, align 4
  %19 = icmp ne i32 %15, %18
  br i1 %19, label %24, label %20

20:                                               ; preds = %12
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 1
  %23 = load i32, ptr %22, align 8
  store i32 %23, ptr %3, align 4
  br label %47

24:                                               ; preds = %12
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.expdesc, ptr %25, i32 0, i32 1
  %27 = load i32, ptr %26, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = call i32 @luaY_nvarstack(ptr noundef %28)
  %30 = icmp sge i32 %27, %29
  br i1 %30, label %31, label %40

31:                                               ; preds = %24
  %32 = load ptr, ptr %4, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.expdesc, ptr %34, i32 0, i32 1
  %36 = load i32, ptr %35, align 8
  call void @exp2reg(ptr noundef %32, ptr noundef %33, i32 noundef %36)
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.expdesc, ptr %37, i32 0, i32 1
  %39 = load i32, ptr %38, align 8
  store i32 %39, ptr %3, align 4
  br label %47

40:                                               ; preds = %24
  br label %41

41:                                               ; preds = %40, %2
  %42 = load ptr, ptr %4, align 8
  %43 = load ptr, ptr %5, align 8
  call void @luaK_exp2nextreg(ptr noundef %42, ptr noundef %43)
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.expdesc, ptr %44, i32 0, i32 1
  %46 = load i32, ptr %45, align 8
  store i32 %46, ptr %3, align 4
  br label %47

47:                                               ; preds = %41, %31, %20
  %48 = load i32, ptr %3, align 4
  ret i32 %48
}

declare hidden i32 @luaY_nvarstack(ptr noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_exp2anyregup(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  %8 = icmp ne i32 %7, 10
  br i1 %8, label %17, label %9

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 2
  %12 = load i32, ptr %11, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 3
  %15 = load i32, ptr %14, align 4
  %16 = icmp ne i32 %12, %15
  br i1 %16, label %17, label %21

17:                                               ; preds = %9, %2
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @luaK_exp2anyreg(ptr noundef %18, ptr noundef %19)
  br label %21

21:                                               ; preds = %17, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_exp2val(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 2
  %7 = load i32, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 3
  %10 = load i32, ptr %9, align 4
  %11 = icmp ne i32 %7, %10
  br i1 %11, label %12, label %16

12:                                               ; preds = %2
  %13 = load ptr, ptr %3, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = call i32 @luaK_exp2anyreg(ptr noundef %13, ptr noundef %14)
  br label %19

16:                                               ; preds = %2
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  call void @luaK_dischargevars(ptr noundef %17, ptr noundef %18)
  br label %19

19:                                               ; preds = %16, %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_storevar(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  switch i32 %10, label %83 [
    i32 9, label %11
    i32 10, label %21
    i32 13, label %31
    i32 14, label %44
    i32 15, label %57
    i32 12, label %70
  ]

11:                                               ; preds = %3
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %6, align 8
  call void @freeexp(ptr noundef %12, ptr noundef %13)
  %14 = load ptr, ptr %4, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 1
  %18 = getelementptr inbounds %struct.anon.0, ptr %17, i32 0, i32 0
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  call void @exp2reg(ptr noundef %14, ptr noundef %15, i32 noundef %20)
  br label %87

21:                                               ; preds = %3
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %6, align 8
  %24 = call i32 @luaK_exp2anyreg(ptr noundef %22, ptr noundef %23)
  store i32 %24, ptr %7, align 4
  %25 = load ptr, ptr %4, align 8
  %26 = load i32, ptr %7, align 4
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.expdesc, ptr %27, i32 0, i32 1
  %29 = load i32, ptr %28, align 8
  %30 = call i32 @luaK_codeABCk(ptr noundef %25, i32 noundef 10, i32 noundef %26, i32 noundef %29, i32 noundef 0, i32 noundef 0)
  br label %84

31:                                               ; preds = %3
  %32 = load ptr, ptr %4, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.expdesc, ptr %33, i32 0, i32 1
  %35 = getelementptr inbounds %struct.anon, ptr %34, i32 0, i32 1
  %36 = load i8, ptr %35, align 2
  %37 = zext i8 %36 to i32
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.expdesc, ptr %38, i32 0, i32 1
  %40 = getelementptr inbounds %struct.anon, ptr %39, i32 0, i32 0
  %41 = load i16, ptr %40, align 8
  %42 = sext i16 %41 to i32
  %43 = load ptr, ptr %6, align 8
  call void @codeABRK(ptr noundef %32, i32 noundef 15, i32 noundef %37, i32 noundef %42, ptr noundef %43)
  br label %84

44:                                               ; preds = %3
  %45 = load ptr, ptr %4, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.expdesc, ptr %46, i32 0, i32 1
  %48 = getelementptr inbounds %struct.anon, ptr %47, i32 0, i32 1
  %49 = load i8, ptr %48, align 2
  %50 = zext i8 %49 to i32
  %51 = load ptr, ptr %5, align 8
  %52 = getelementptr inbounds %struct.expdesc, ptr %51, i32 0, i32 1
  %53 = getelementptr inbounds %struct.anon, ptr %52, i32 0, i32 0
  %54 = load i16, ptr %53, align 8
  %55 = sext i16 %54 to i32
  %56 = load ptr, ptr %6, align 8
  call void @codeABRK(ptr noundef %45, i32 noundef 17, i32 noundef %50, i32 noundef %55, ptr noundef %56)
  br label %84

57:                                               ; preds = %3
  %58 = load ptr, ptr %4, align 8
  %59 = load ptr, ptr %5, align 8
  %60 = getelementptr inbounds %struct.expdesc, ptr %59, i32 0, i32 1
  %61 = getelementptr inbounds %struct.anon, ptr %60, i32 0, i32 1
  %62 = load i8, ptr %61, align 2
  %63 = zext i8 %62 to i32
  %64 = load ptr, ptr %5, align 8
  %65 = getelementptr inbounds %struct.expdesc, ptr %64, i32 0, i32 1
  %66 = getelementptr inbounds %struct.anon, ptr %65, i32 0, i32 0
  %67 = load i16, ptr %66, align 8
  %68 = sext i16 %67 to i32
  %69 = load ptr, ptr %6, align 8
  call void @codeABRK(ptr noundef %58, i32 noundef 18, i32 noundef %63, i32 noundef %68, ptr noundef %69)
  br label %84

70:                                               ; preds = %3
  %71 = load ptr, ptr %4, align 8
  %72 = load ptr, ptr %5, align 8
  %73 = getelementptr inbounds %struct.expdesc, ptr %72, i32 0, i32 1
  %74 = getelementptr inbounds %struct.anon, ptr %73, i32 0, i32 1
  %75 = load i8, ptr %74, align 2
  %76 = zext i8 %75 to i32
  %77 = load ptr, ptr %5, align 8
  %78 = getelementptr inbounds %struct.expdesc, ptr %77, i32 0, i32 1
  %79 = getelementptr inbounds %struct.anon, ptr %78, i32 0, i32 0
  %80 = load i16, ptr %79, align 8
  %81 = sext i16 %80 to i32
  %82 = load ptr, ptr %6, align 8
  call void @codeABRK(ptr noundef %71, i32 noundef 16, i32 noundef %76, i32 noundef %81, ptr noundef %82)
  br label %84

83:                                               ; preds = %3
  br label %84

84:                                               ; preds = %83, %70, %57, %44, %31, %21
  %85 = load ptr, ptr %4, align 8
  %86 = load ptr, ptr %6, align 8
  call void @freeexp(ptr noundef %85, ptr noundef %86)
  br label %87

87:                                               ; preds = %84, %11
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeABRK(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, ptr noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store ptr %4, ptr %10, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = load ptr, ptr %10, align 8
  %14 = call i32 @exp2RK(ptr noundef %12, ptr noundef %13)
  store i32 %14, ptr %11, align 4
  %15 = load ptr, ptr %6, align 8
  %16 = load i32, ptr %7, align 4
  %17 = load i32, ptr %8, align 4
  %18 = load i32, ptr %9, align 4
  %19 = load ptr, ptr %10, align 8
  %20 = getelementptr inbounds %struct.expdesc, ptr %19, i32 0, i32 1
  %21 = load i32, ptr %20, align 8
  %22 = load i32, ptr %11, align 4
  %23 = call i32 @luaK_codeABCk(ptr noundef %15, i32 noundef %16, i32 noundef %17, i32 noundef %18, i32 noundef %21, i32 noundef %22)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_self(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = call i32 @luaK_exp2anyreg(ptr noundef %8, ptr noundef %9)
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 8
  store i32 %13, ptr %7, align 4
  %14 = load ptr, ptr %4, align 8
  %15 = load ptr, ptr %5, align 8
  call void @freeexp(ptr noundef %14, ptr noundef %15)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.FuncState, ptr %16, i32 0, i32 15
  %18 = load i8, ptr %17, align 4
  %19 = zext i8 %18 to i32
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.expdesc, ptr %20, i32 0, i32 1
  store i32 %19, ptr %21, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.expdesc, ptr %22, i32 0, i32 0
  store i32 8, ptr %23, align 8
  %24 = load ptr, ptr %4, align 8
  call void @luaK_reserveregs(ptr noundef %24, i32 noundef 2)
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.expdesc, ptr %26, i32 0, i32 1
  %28 = load i32, ptr %27, align 8
  %29 = load i32, ptr %7, align 4
  %30 = load ptr, ptr %6, align 8
  call void @codeABRK(ptr noundef %25, i32 noundef 20, i32 noundef %28, i32 noundef %29, ptr noundef %30)
  %31 = load ptr, ptr %4, align 8
  %32 = load ptr, ptr %6, align 8
  call void @freeexp(ptr noundef %31, ptr noundef %32)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_goiftrue(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @luaK_dischargevars(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  switch i32 %10, label %18 [
    i32 16, label %11
    i32 4, label %17
    i32 5, label %17
    i32 6, label %17
    i32 7, label %17
    i32 2, label %17
  ]

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  %13 = load ptr, ptr %4, align 8
  call void @negatecondition(ptr noundef %12, ptr noundef %13)
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 1
  %16 = load i32, ptr %15, align 8
  store i32 %16, ptr %5, align 4
  br label %22

17:                                               ; preds = %2, %2, %2, %2, %2
  store i32 -1, ptr %5, align 4
  br label %22

18:                                               ; preds = %2
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = call i32 @jumponcond(ptr noundef %19, ptr noundef %20, i32 noundef 0)
  store i32 %21, ptr %5, align 4
  br label %22

22:                                               ; preds = %18, %17, %11
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.expdesc, ptr %24, i32 0, i32 3
  %26 = load i32, ptr %5, align 4
  call void @luaK_concat(ptr noundef %23, ptr noundef %25, i32 noundef %26)
  %27 = load ptr, ptr %3, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 2
  %30 = load i32, ptr %29, align 8
  call void @luaK_patchtohere(ptr noundef %27, i32 noundef %30)
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.expdesc, ptr %31, i32 0, i32 2
  store i32 -1, ptr %32, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @negatecondition(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 1
  %9 = load i32, ptr %8, align 8
  %10 = call ptr @getjumpcontrol(ptr noundef %6, i32 noundef %9)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %11, align 4
  %13 = and i32 %12, -32769
  %14 = load ptr, ptr %5, align 8
  %15 = load i32, ptr %14, align 4
  %16 = lshr i32 %15, 15
  %17 = and i32 %16, 1
  %18 = xor i32 %17, 1
  %19 = shl i32 %18, 15
  %20 = and i32 %19, 32768
  %21 = or i32 %13, %20
  %22 = load ptr, ptr %5, align 8
  store i32 %21, ptr %22, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @jumponcond(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %9 = load ptr, ptr %6, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  %12 = icmp eq i32 %11, 17
  br i1 %12, label %13, label %41

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.FuncState, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 16
  %18 = load ptr, ptr %17, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.expdesc, ptr %19, i32 0, i32 1
  %21 = load i32, ptr %20, align 8
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds i32, ptr %18, i64 %22
  %24 = load i32, ptr %23, align 4
  store i32 %24, ptr %8, align 4
  %25 = load i32, ptr %8, align 4
  %26 = lshr i32 %25, 0
  %27 = and i32 %26, 127
  %28 = icmp eq i32 %27, 51
  br i1 %28, label %29, label %40

29:                                               ; preds = %13
  %30 = load ptr, ptr %5, align 8
  call void @removelastinstruction(ptr noundef %30)
  %31 = load ptr, ptr %5, align 8
  %32 = load i32, ptr %8, align 4
  %33 = lshr i32 %32, 16
  %34 = and i32 %33, 255
  %35 = load i32, ptr %7, align 4
  %36 = icmp ne i32 %35, 0
  %37 = xor i1 %36, true
  %38 = zext i1 %37 to i32
  %39 = call i32 @condjump(ptr noundef %31, i32 noundef 66, i32 noundef %34, i32 noundef 0, i32 noundef 0, i32 noundef %38)
  store i32 %39, ptr %4, align 4
  br label %52

40:                                               ; preds = %13
  br label %41

41:                                               ; preds = %40, %3
  %42 = load ptr, ptr %5, align 8
  %43 = load ptr, ptr %6, align 8
  call void @discharge2anyreg(ptr noundef %42, ptr noundef %43)
  %44 = load ptr, ptr %5, align 8
  %45 = load ptr, ptr %6, align 8
  call void @freeexp(ptr noundef %44, ptr noundef %45)
  %46 = load ptr, ptr %5, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.expdesc, ptr %47, i32 0, i32 1
  %49 = load i32, ptr %48, align 8
  %50 = load i32, ptr %7, align 4
  %51 = call i32 @condjump(ptr noundef %46, i32 noundef 67, i32 noundef 255, i32 noundef %49, i32 noundef 0, i32 noundef %50)
  store i32 %51, ptr %4, align 4
  br label %52

52:                                               ; preds = %41, %29
  %53 = load i32, ptr %4, align 4
  ret i32 %53
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_goiffalse(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @luaK_dischargevars(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  switch i32 %10, label %16 [
    i32 16, label %11
    i32 1, label %15
    i32 3, label %15
  ]

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 1
  %14 = load i32, ptr %13, align 8
  store i32 %14, ptr %5, align 4
  br label %20

15:                                               ; preds = %2, %2
  store i32 -1, ptr %5, align 4
  br label %20

16:                                               ; preds = %2
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = call i32 @jumponcond(ptr noundef %17, ptr noundef %18, i32 noundef 1)
  store i32 %19, ptr %5, align 4
  br label %20

20:                                               ; preds = %16, %15, %11
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.expdesc, ptr %22, i32 0, i32 2
  %24 = load i32, ptr %5, align 4
  call void @luaK_concat(ptr noundef %21, ptr noundef %23, i32 noundef %24)
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.expdesc, ptr %26, i32 0, i32 3
  %28 = load i32, ptr %27, align 4
  call void @luaK_patchtohere(ptr noundef %25, i32 noundef %28)
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.expdesc, ptr %29, i32 0, i32 3
  store i32 -1, ptr %30, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_indexed(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  %11 = icmp eq i32 %10, 7
  br i1 %11, label %12, label %15

12:                                               ; preds = %3
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %6, align 8
  call void @str2K(ptr noundef %13, ptr noundef %14)
  br label %15

15:                                               ; preds = %12, %3
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  %19 = icmp eq i32 %18, 10
  br i1 %19, label %20, label %29

20:                                               ; preds = %15
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 @isKstr(ptr noundef %21, ptr noundef %22)
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %29, label %25

25:                                               ; preds = %20
  %26 = load ptr, ptr %4, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = call i32 @luaK_exp2anyreg(ptr noundef %26, ptr noundef %27)
  br label %29

29:                                               ; preds = %25, %20, %15
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 0
  %32 = load i32, ptr %31, align 8
  %33 = icmp eq i32 %32, 10
  br i1 %33, label %34, label %52

34:                                               ; preds = %29
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 1
  %37 = load i32, ptr %36, align 8
  store i32 %37, ptr %7, align 4
  %38 = load i32, ptr %7, align 4
  %39 = trunc i32 %38 to i8
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.expdesc, ptr %40, i32 0, i32 1
  %42 = getelementptr inbounds %struct.anon, ptr %41, i32 0, i32 1
  store i8 %39, ptr %42, align 2
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.expdesc, ptr %43, i32 0, i32 1
  %45 = load i32, ptr %44, align 8
  %46 = trunc i32 %45 to i16
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %struct.expdesc, ptr %47, i32 0, i32 1
  %49 = getelementptr inbounds %struct.anon, ptr %48, i32 0, i32 0
  store i16 %46, ptr %49, align 8
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %struct.expdesc, ptr %50, i32 0, i32 0
  store i32 13, ptr %51, align 8
  br label %114

52:                                               ; preds = %29
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.expdesc, ptr %53, i32 0, i32 0
  %55 = load i32, ptr %54, align 8
  %56 = icmp eq i32 %55, 9
  br i1 %56, label %57, label %63

57:                                               ; preds = %52
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.expdesc, ptr %58, i32 0, i32 1
  %60 = getelementptr inbounds %struct.anon.0, ptr %59, i32 0, i32 0
  %61 = load i8, ptr %60, align 8
  %62 = zext i8 %61 to i32
  br label %67

63:                                               ; preds = %52
  %64 = load ptr, ptr %5, align 8
  %65 = getelementptr inbounds %struct.expdesc, ptr %64, i32 0, i32 1
  %66 = load i32, ptr %65, align 8
  br label %67

67:                                               ; preds = %63, %57
  %68 = phi i32 [ %62, %57 ], [ %66, %63 ]
  %69 = trunc i32 %68 to i8
  %70 = load ptr, ptr %5, align 8
  %71 = getelementptr inbounds %struct.expdesc, ptr %70, i32 0, i32 1
  %72 = getelementptr inbounds %struct.anon, ptr %71, i32 0, i32 1
  store i8 %69, ptr %72, align 2
  %73 = load ptr, ptr %4, align 8
  %74 = load ptr, ptr %6, align 8
  %75 = call i32 @isKstr(ptr noundef %73, ptr noundef %74)
  %76 = icmp ne i32 %75, 0
  br i1 %76, label %77, label %87

77:                                               ; preds = %67
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.expdesc, ptr %78, i32 0, i32 1
  %80 = load i32, ptr %79, align 8
  %81 = trunc i32 %80 to i16
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.expdesc, ptr %82, i32 0, i32 1
  %84 = getelementptr inbounds %struct.anon, ptr %83, i32 0, i32 0
  store i16 %81, ptr %84, align 8
  %85 = load ptr, ptr %5, align 8
  %86 = getelementptr inbounds %struct.expdesc, ptr %85, i32 0, i32 0
  store i32 15, ptr %86, align 8
  br label %113

87:                                               ; preds = %67
  %88 = load ptr, ptr %6, align 8
  %89 = call i32 @isCint(ptr noundef %88)
  %90 = icmp ne i32 %89, 0
  br i1 %90, label %91, label %102

91:                                               ; preds = %87
  %92 = load ptr, ptr %6, align 8
  %93 = getelementptr inbounds %struct.expdesc, ptr %92, i32 0, i32 1
  %94 = load i64, ptr %93, align 8
  %95 = trunc i64 %94 to i32
  %96 = trunc i32 %95 to i16
  %97 = load ptr, ptr %5, align 8
  %98 = getelementptr inbounds %struct.expdesc, ptr %97, i32 0, i32 1
  %99 = getelementptr inbounds %struct.anon, ptr %98, i32 0, i32 0
  store i16 %96, ptr %99, align 8
  %100 = load ptr, ptr %5, align 8
  %101 = getelementptr inbounds %struct.expdesc, ptr %100, i32 0, i32 0
  store i32 14, ptr %101, align 8
  br label %112

102:                                              ; preds = %87
  %103 = load ptr, ptr %4, align 8
  %104 = load ptr, ptr %6, align 8
  %105 = call i32 @luaK_exp2anyreg(ptr noundef %103, ptr noundef %104)
  %106 = trunc i32 %105 to i16
  %107 = load ptr, ptr %5, align 8
  %108 = getelementptr inbounds %struct.expdesc, ptr %107, i32 0, i32 1
  %109 = getelementptr inbounds %struct.anon, ptr %108, i32 0, i32 0
  store i16 %106, ptr %109, align 8
  %110 = load ptr, ptr %5, align 8
  %111 = getelementptr inbounds %struct.expdesc, ptr %110, i32 0, i32 0
  store i32 12, ptr %111, align 8
  br label %112

112:                                              ; preds = %102, %91
  br label %113

113:                                              ; preds = %112, %77
  br label %114

114:                                              ; preds = %113, %34
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @str2K(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 1
  %8 = load ptr, ptr %7, align 8
  %9 = call i32 @stringK(ptr noundef %5, ptr noundef %8)
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 1
  store i32 %9, ptr %11, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 0
  store i32 4, ptr %13, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isKstr(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  %8 = icmp eq i32 %7, 4
  br i1 %8, label %9, label %37

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 2
  %12 = load i32, ptr %11, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 3
  %15 = load i32, ptr %14, align 4
  %16 = icmp ne i32 %12, %15
  br i1 %16, label %37, label %17

17:                                               ; preds = %9
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 1
  %20 = load i32, ptr %19, align 8
  %21 = icmp sle i32 %20, 255
  br i1 %21, label %22, label %37

22:                                               ; preds = %17
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.FuncState, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 15
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 1
  %30 = load i32, ptr %29, align 8
  %31 = sext i32 %30 to i64
  %32 = getelementptr inbounds %struct.TValue, ptr %27, i64 %31
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = icmp eq i32 %35, 68
  br label %37

37:                                               ; preds = %22, %17, %9, %2
  %38 = phi i1 [ false, %17 ], [ false, %9 ], [ false, %2 ], [ %36, %22 ]
  %39 = zext i1 %38 to i32
  ret i32 %39
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isCint(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @isKint(ptr noundef %3)
  %5 = icmp ne i32 %4, 0
  br i1 %5, label %6, label %11

6:                                                ; preds = %1
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 1
  %9 = load i64, ptr %8, align 8
  %10 = icmp ule i64 %9, 255
  br label %11

11:                                               ; preds = %6, %1
  %12 = phi i1 [ false, %1 ], [ %10, %6 ]
  %13 = zext i1 %12 to i32
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_prefix(ptr noundef %0, i32 noundef %1, ptr noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %7, align 8
  call void @luaK_dischargevars(ptr noundef %9, ptr noundef %10)
  %11 = load i32, ptr %6, align 4
  switch i32 %11, label %30 [
    i32 0, label %12
    i32 1, label %12
    i32 3, label %21
    i32 2, label %27
  ]

12:                                               ; preds = %4, %4
  %13 = load ptr, ptr %5, align 8
  %14 = load i32, ptr %6, align 4
  %15 = add i32 %14, 12
  %16 = load ptr, ptr %7, align 8
  %17 = call i32 @constfolding(ptr noundef %13, i32 noundef %15, ptr noundef %16, ptr noundef @luaK_prefix.ef)
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %20

19:                                               ; preds = %12
  br label %31

20:                                               ; preds = %12
  br label %21

21:                                               ; preds = %4, %20
  %22 = load ptr, ptr %5, align 8
  %23 = load i32, ptr %6, align 4
  %24 = call i32 @unopr2op(i32 noundef %23)
  %25 = load ptr, ptr %7, align 8
  %26 = load i32, ptr %8, align 4
  call void @codeunexpval(ptr noundef %22, i32 noundef %24, ptr noundef %25, i32 noundef %26)
  br label %31

27:                                               ; preds = %4
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %7, align 8
  call void @codenot(ptr noundef %28, ptr noundef %29)
  br label %31

30:                                               ; preds = %4
  br label %31

31:                                               ; preds = %30, %27, %21, %19
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @constfolding(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #2 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %struct.TValue, align 8
  %11 = alloca %struct.TValue, align 8
  %12 = alloca %struct.TValue, align 8
  %13 = alloca double, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = call i32 @tonumeral(ptr noundef %14, ptr noundef %10)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %25

17:                                               ; preds = %4
  %18 = load ptr, ptr %9, align 8
  %19 = call i32 @tonumeral(ptr noundef %18, ptr noundef %11)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %25

21:                                               ; preds = %17
  %22 = load i32, ptr %7, align 4
  %23 = call i32 @validop(i32 noundef %22, ptr noundef %10, ptr noundef %11)
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %26, label %25

25:                                               ; preds = %21, %17, %4
  store i32 0, ptr %5, align 4
  br label %62

26:                                               ; preds = %21
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %struct.FuncState, ptr %27, i32 0, i32 2
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.LexState, ptr %29, i32 0, i32 6
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %7, align 4
  %33 = call i32 @luaO_rawarith(ptr noundef %31, i32 noundef %32, ptr noundef %10, ptr noundef %11, ptr noundef %12)
  %34 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %35 = load i8, ptr %34, align 8
  %36 = zext i8 %35 to i32
  %37 = icmp eq i32 %36, 3
  br i1 %37, label %38, label %45

38:                                               ; preds = %26
  %39 = load ptr, ptr %8, align 8
  %40 = getelementptr inbounds %struct.expdesc, ptr %39, i32 0, i32 0
  store i32 6, ptr %40, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 0
  %42 = load i64, ptr %41, align 8
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds %struct.expdesc, ptr %43, i32 0, i32 1
  store i64 %42, ptr %44, align 8
  br label %61

45:                                               ; preds = %26
  %46 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 0
  %47 = load double, ptr %46, align 8
  store double %47, ptr %13, align 8
  %48 = load double, ptr %13, align 8
  %49 = load double, ptr %13, align 8
  %50 = fcmp oeq double %48, %49
  br i1 %50, label %51, label %54

51:                                               ; preds = %45
  %52 = load double, ptr %13, align 8
  %53 = fcmp oeq double %52, 0.000000e+00
  br i1 %53, label %54, label %55

54:                                               ; preds = %51, %45
  store i32 0, ptr %5, align 4
  br label %62

55:                                               ; preds = %51
  %56 = load ptr, ptr %8, align 8
  %57 = getelementptr inbounds %struct.expdesc, ptr %56, i32 0, i32 0
  store i32 5, ptr %57, align 8
  %58 = load double, ptr %13, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = getelementptr inbounds %struct.expdesc, ptr %59, i32 0, i32 1
  store double %58, ptr %60, align 8
  br label %61

61:                                               ; preds = %55, %38
  store i32 1, ptr %5, align 4
  br label %62

62:                                               ; preds = %61, %54, %25
  %63 = load i32, ptr %5, align 4
  ret i32 %63
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeunexpval(ptr noundef %0, i32 noundef %1, ptr noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = call i32 @luaK_exp2anyreg(ptr noundef %10, ptr noundef %11)
  store i32 %12, ptr %9, align 4
  %13 = load ptr, ptr %5, align 8
  %14 = load ptr, ptr %7, align 8
  call void @freeexp(ptr noundef %13, ptr noundef %14)
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %6, align 4
  %17 = load i32, ptr %9, align 4
  %18 = call i32 @luaK_codeABCk(ptr noundef %15, i32 noundef %16, i32 noundef 0, i32 noundef %17, i32 noundef 0, i32 noundef 0)
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.expdesc, ptr %19, i32 0, i32 1
  store i32 %18, ptr %20, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 0
  store i32 17, ptr %22, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = load i32, ptr %8, align 4
  call void @luaK_fixline(ptr noundef %23, i32 noundef %24)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @unopr2op(i32 noundef %0) #2 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  %3 = load i32, ptr %2, align 4
  %4 = sub nsw i32 %3, 0
  %5 = add nsw i32 %4, 49
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codenot(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.expdesc, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  switch i32 %8, label %32 [
    i32 1, label %9
    i32 3, label %9
    i32 4, label %12
    i32 5, label %12
    i32 6, label %12
    i32 7, label %12
    i32 2, label %12
    i32 16, label %15
    i32 17, label %18
    i32 8, label %18
  ]

9:                                                ; preds = %2, %2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 0
  store i32 2, ptr %11, align 8
  br label %33

12:                                               ; preds = %2, %2, %2, %2, %2
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.expdesc, ptr %13, i32 0, i32 0
  store i32 3, ptr %14, align 8
  br label %33

15:                                               ; preds = %2
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %4, align 8
  call void @negatecondition(ptr noundef %16, ptr noundef %17)
  br label %33

18:                                               ; preds = %2, %2
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %4, align 8
  call void @discharge2anyreg(ptr noundef %19, ptr noundef %20)
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  call void @freeexp(ptr noundef %21, ptr noundef %22)
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.expdesc, ptr %24, i32 0, i32 1
  %26 = load i32, ptr %25, align 8
  %27 = call i32 @luaK_codeABCk(ptr noundef %23, i32 noundef 51, i32 noundef 0, i32 noundef %26, i32 noundef 0, i32 noundef 0)
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 1
  store i32 %27, ptr %29, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 0
  store i32 17, ptr %31, align 8
  br label %33

32:                                               ; preds = %2
  br label %33

33:                                               ; preds = %32, %18, %15, %12, %9
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.expdesc, ptr %34, i32 0, i32 3
  %36 = load i32, ptr %35, align 4
  store i32 %36, ptr %5, align 4
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.expdesc, ptr %37, i32 0, i32 2
  %39 = load i32, ptr %38, align 8
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.expdesc, ptr %40, i32 0, i32 3
  store i32 %39, ptr %41, align 4
  %42 = load i32, ptr %5, align 4
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.expdesc, ptr %43, i32 0, i32 2
  store i32 %42, ptr %44, align 8
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.expdesc, ptr %46, i32 0, i32 3
  %48 = load i32, ptr %47, align 4
  call void @removevalues(ptr noundef %45, i32 noundef %48)
  %49 = load ptr, ptr %3, align 8
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.expdesc, ptr %50, i32 0, i32 2
  %52 = load i32, ptr %51, align 8
  call void @removevalues(ptr noundef %49, i32 noundef %52)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_infix(ptr noundef %0, i32 noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %6, align 8
  call void @luaK_dischargevars(ptr noundef %9, ptr noundef %10)
  %11 = load i32, ptr %5, align 4
  switch i32 %11, label %48 [
    i32 19, label %12
    i32 20, label %15
    i32 12, label %18
    i32 0, label %21
    i32 1, label %21
    i32 2, label %21
    i32 5, label %21
    i32 6, label %21
    i32 3, label %21
    i32 4, label %21
    i32 7, label %21
    i32 8, label %21
    i32 9, label %21
    i32 10, label %21
    i32 11, label %21
    i32 13, label %30
    i32 16, label %30
    i32 14, label %39
    i32 15, label %39
    i32 17, label %39
    i32 18, label %39
  ]

12:                                               ; preds = %3
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %6, align 8
  call void @luaK_goiftrue(ptr noundef %13, ptr noundef %14)
  br label %49

15:                                               ; preds = %3
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %6, align 8
  call void @luaK_goiffalse(ptr noundef %16, ptr noundef %17)
  br label %49

18:                                               ; preds = %3
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %6, align 8
  call void @luaK_exp2nextreg(ptr noundef %19, ptr noundef %20)
  br label %49

21:                                               ; preds = %3, %3, %3, %3, %3, %3, %3, %3, %3, %3, %3, %3
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 @tonumeral(ptr noundef %22, ptr noundef null)
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %29, label %25

25:                                               ; preds = %21
  %26 = load ptr, ptr %4, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = call i32 @luaK_exp2anyreg(ptr noundef %26, ptr noundef %27)
  br label %29

29:                                               ; preds = %25, %21
  br label %49

30:                                               ; preds = %3, %3
  %31 = load ptr, ptr %6, align 8
  %32 = call i32 @tonumeral(ptr noundef %31, ptr noundef null)
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %38, label %34

34:                                               ; preds = %30
  %35 = load ptr, ptr %4, align 8
  %36 = load ptr, ptr %6, align 8
  %37 = call i32 @exp2RK(ptr noundef %35, ptr noundef %36)
  br label %38

38:                                               ; preds = %34, %30
  br label %49

39:                                               ; preds = %3, %3, %3, %3
  %40 = load ptr, ptr %6, align 8
  %41 = call i32 @isSCnumber(ptr noundef %40, ptr noundef %7, ptr noundef %8)
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %47, label %43

43:                                               ; preds = %39
  %44 = load ptr, ptr %4, align 8
  %45 = load ptr, ptr %6, align 8
  %46 = call i32 @luaK_exp2anyreg(ptr noundef %44, ptr noundef %45)
  br label %47

47:                                               ; preds = %43, %39
  br label %49

48:                                               ; preds = %3
  br label %49

49:                                               ; preds = %48, %47, %38, %29, %18, %15, %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @exp2RK(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = call i32 @luaK_exp2K(ptr noundef %6, ptr noundef %7)
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %11

10:                                               ; preds = %2
  store i32 1, ptr %3, align 4
  br label %15

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @luaK_exp2anyreg(ptr noundef %12, ptr noundef %13)
  store i32 0, ptr %3, align 4
  br label %15

15:                                               ; preds = %11, %10
  %16 = load i32, ptr %3, align 4
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isSCnumber(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  %12 = icmp eq i32 %11, 6
  br i1 %12, label %13, label %17

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 1
  %16 = load i64, ptr %15, align 8
  store i64 %16, ptr %8, align 8
  br label %32

17:                                               ; preds = %3
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 0
  %20 = load i32, ptr %19, align 8
  %21 = icmp eq i32 %20, 5
  br i1 %21, label %22, label %30

22:                                               ; preds = %17
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.expdesc, ptr %23, i32 0, i32 1
  %25 = load double, ptr %24, align 8
  %26 = call i32 @luaV_flttointeger(double noundef %25, ptr noundef %8, i32 noundef 0)
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %30

28:                                               ; preds = %22
  %29 = load ptr, ptr %7, align 8
  store i32 1, ptr %29, align 4
  br label %31

30:                                               ; preds = %22, %17
  store i32 0, ptr %4, align 4
  br label %50

31:                                               ; preds = %28
  br label %32

32:                                               ; preds = %31, %13
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.expdesc, ptr %33, i32 0, i32 2
  %35 = load i32, ptr %34, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.expdesc, ptr %36, i32 0, i32 3
  %38 = load i32, ptr %37, align 4
  %39 = icmp ne i32 %35, %38
  br i1 %39, label %49, label %40

40:                                               ; preds = %32
  %41 = load i64, ptr %8, align 8
  %42 = call i32 @fitsC(i64 noundef %41)
  %43 = icmp ne i32 %42, 0
  br i1 %43, label %44, label %49

44:                                               ; preds = %40
  %45 = load i64, ptr %8, align 8
  %46 = trunc i64 %45 to i32
  %47 = add nsw i32 %46, 127
  %48 = load ptr, ptr %6, align 8
  store i32 %47, ptr %48, align 4
  store i32 1, ptr %4, align 4
  br label %50

49:                                               ; preds = %40, %32
  store i32 0, ptr %4, align 4
  br label %50

50:                                               ; preds = %49, %44, %30
  %51 = load i32, ptr %4, align 4
  ret i32 %51
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_posfix(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  %11 = load ptr, ptr %6, align 8
  %12 = load ptr, ptr %9, align 8
  call void @luaK_dischargevars(ptr noundef %11, ptr noundef %12)
  %13 = load i32, ptr %7, align 4
  %14 = icmp ule i32 %13, 11
  br i1 %14, label %15, label %24

15:                                               ; preds = %5
  %16 = load ptr, ptr %6, align 8
  %17 = load i32, ptr %7, align 4
  %18 = add i32 %17, 0
  %19 = load ptr, ptr %8, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = call i32 @constfolding(ptr noundef %16, i32 noundef %18, ptr noundef %19, ptr noundef %20)
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %15
  br label %138

24:                                               ; preds = %15, %5
  %25 = load i32, ptr %7, align 4
  switch i32 %25, label %137 [
    i32 19, label %26
    i32 20, label %35
    i32 12, label %44
    i32 0, label %51
    i32 2, label %51
    i32 1, label %57
    i32 5, label %66
    i32 6, label %66
    i32 3, label %66
    i32 4, label %66
    i32 7, label %72
    i32 8, label %72
    i32 9, label %72
    i32 10, label %78
    i32 11, label %105
    i32 13, label %121
    i32 16, label %121
    i32 17, label %126
    i32 18, label %126
    i32 14, label %132
    i32 15, label %132
  ]

26:                                               ; preds = %24
  %27 = load ptr, ptr %6, align 8
  %28 = load ptr, ptr %9, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 3
  %30 = load ptr, ptr %8, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 3
  %32 = load i32, ptr %31, align 4
  call void @luaK_concat(ptr noundef %27, ptr noundef %29, i32 noundef %32)
  %33 = load ptr, ptr %8, align 8
  %34 = load ptr, ptr %9, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %33, ptr align 8 %34, i64 24, i1 false)
  br label %138

35:                                               ; preds = %24
  %36 = load ptr, ptr %6, align 8
  %37 = load ptr, ptr %9, align 8
  %38 = getelementptr inbounds %struct.expdesc, ptr %37, i32 0, i32 2
  %39 = load ptr, ptr %8, align 8
  %40 = getelementptr inbounds %struct.expdesc, ptr %39, i32 0, i32 2
  %41 = load i32, ptr %40, align 8
  call void @luaK_concat(ptr noundef %36, ptr noundef %38, i32 noundef %41)
  %42 = load ptr, ptr %8, align 8
  %43 = load ptr, ptr %9, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %42, ptr align 8 %43, i64 24, i1 false)
  br label %138

44:                                               ; preds = %24
  %45 = load ptr, ptr %6, align 8
  %46 = load ptr, ptr %9, align 8
  call void @luaK_exp2nextreg(ptr noundef %45, ptr noundef %46)
  %47 = load ptr, ptr %6, align 8
  %48 = load ptr, ptr %8, align 8
  %49 = load ptr, ptr %9, align 8
  %50 = load i32, ptr %10, align 4
  call void @codeconcat(ptr noundef %47, ptr noundef %48, ptr noundef %49, i32 noundef %50)
  br label %138

51:                                               ; preds = %24, %24
  %52 = load ptr, ptr %6, align 8
  %53 = load i32, ptr %7, align 4
  %54 = load ptr, ptr %8, align 8
  %55 = load ptr, ptr %9, align 8
  %56 = load i32, ptr %10, align 4
  call void @codecommutative(ptr noundef %52, i32 noundef %53, ptr noundef %54, ptr noundef %55, i32 noundef %56)
  br label %138

57:                                               ; preds = %24
  %58 = load ptr, ptr %6, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = load ptr, ptr %9, align 8
  %61 = load i32, ptr %10, align 4
  %62 = call i32 @finishbinexpneg(ptr noundef %58, ptr noundef %59, ptr noundef %60, i32 noundef 21, i32 noundef %61, i32 noundef 7)
  %63 = icmp ne i32 %62, 0
  br i1 %63, label %64, label %65

64:                                               ; preds = %57
  br label %138

65:                                               ; preds = %57
  br label %66

66:                                               ; preds = %24, %24, %24, %24, %65
  %67 = load ptr, ptr %6, align 8
  %68 = load i32, ptr %7, align 4
  %69 = load ptr, ptr %8, align 8
  %70 = load ptr, ptr %9, align 8
  %71 = load i32, ptr %10, align 4
  call void @codearith(ptr noundef %67, i32 noundef %68, ptr noundef %69, ptr noundef %70, i32 noundef 0, i32 noundef %71)
  br label %138

72:                                               ; preds = %24, %24, %24
  %73 = load ptr, ptr %6, align 8
  %74 = load i32, ptr %7, align 4
  %75 = load ptr, ptr %8, align 8
  %76 = load ptr, ptr %9, align 8
  %77 = load i32, ptr %10, align 4
  call void @codebitwise(ptr noundef %73, i32 noundef %74, ptr noundef %75, ptr noundef %76, i32 noundef %77)
  br label %138

78:                                               ; preds = %24
  %79 = load ptr, ptr %8, align 8
  %80 = call i32 @isSCint(ptr noundef %79)
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %89

82:                                               ; preds = %78
  %83 = load ptr, ptr %8, align 8
  %84 = load ptr, ptr %9, align 8
  call void @swapexps(ptr noundef %83, ptr noundef %84)
  %85 = load ptr, ptr %6, align 8
  %86 = load ptr, ptr %8, align 8
  %87 = load ptr, ptr %9, align 8
  %88 = load i32, ptr %10, align 4
  call void @codebini(ptr noundef %85, i32 noundef 33, ptr noundef %86, ptr noundef %87, i32 noundef 1, i32 noundef %88, i32 noundef 16)
  br label %104

89:                                               ; preds = %78
  %90 = load ptr, ptr %6, align 8
  %91 = load ptr, ptr %8, align 8
  %92 = load ptr, ptr %9, align 8
  %93 = load i32, ptr %10, align 4
  %94 = call i32 @finishbinexpneg(ptr noundef %90, ptr noundef %91, ptr noundef %92, i32 noundef 32, i32 noundef %93, i32 noundef 16)
  %95 = icmp ne i32 %94, 0
  br i1 %95, label %96, label %97

96:                                               ; preds = %89
  br label %103

97:                                               ; preds = %89
  %98 = load ptr, ptr %6, align 8
  %99 = load i32, ptr %7, align 4
  %100 = load ptr, ptr %8, align 8
  %101 = load ptr, ptr %9, align 8
  %102 = load i32, ptr %10, align 4
  call void @codebinexpval(ptr noundef %98, i32 noundef %99, ptr noundef %100, ptr noundef %101, i32 noundef %102)
  br label %103

103:                                              ; preds = %97, %96
  br label %104

104:                                              ; preds = %103, %82
  br label %138

105:                                              ; preds = %24
  %106 = load ptr, ptr %9, align 8
  %107 = call i32 @isSCint(ptr noundef %106)
  %108 = icmp ne i32 %107, 0
  br i1 %108, label %109, label %114

109:                                              ; preds = %105
  %110 = load ptr, ptr %6, align 8
  %111 = load ptr, ptr %8, align 8
  %112 = load ptr, ptr %9, align 8
  %113 = load i32, ptr %10, align 4
  call void @codebini(ptr noundef %110, i32 noundef 32, ptr noundef %111, ptr noundef %112, i32 noundef 0, i32 noundef %113, i32 noundef 17)
  br label %120

114:                                              ; preds = %105
  %115 = load ptr, ptr %6, align 8
  %116 = load i32, ptr %7, align 4
  %117 = load ptr, ptr %8, align 8
  %118 = load ptr, ptr %9, align 8
  %119 = load i32, ptr %10, align 4
  call void @codebinexpval(ptr noundef %115, i32 noundef %116, ptr noundef %117, ptr noundef %118, i32 noundef %119)
  br label %120

120:                                              ; preds = %114, %109
  br label %138

121:                                              ; preds = %24, %24
  %122 = load ptr, ptr %6, align 8
  %123 = load i32, ptr %7, align 4
  %124 = load ptr, ptr %8, align 8
  %125 = load ptr, ptr %9, align 8
  call void @codeeq(ptr noundef %122, i32 noundef %123, ptr noundef %124, ptr noundef %125)
  br label %138

126:                                              ; preds = %24, %24
  %127 = load ptr, ptr %8, align 8
  %128 = load ptr, ptr %9, align 8
  call void @swapexps(ptr noundef %127, ptr noundef %128)
  %129 = load i32, ptr %7, align 4
  %130 = sub i32 %129, 17
  %131 = add i32 %130, 14
  store i32 %131, ptr %7, align 4
  br label %132

132:                                              ; preds = %24, %24, %126
  %133 = load ptr, ptr %6, align 8
  %134 = load i32, ptr %7, align 4
  %135 = load ptr, ptr %8, align 8
  %136 = load ptr, ptr %9, align 8
  call void @codeorder(ptr noundef %133, i32 noundef %134, ptr noundef %135, ptr noundef %136)
  br label %138

137:                                              ; preds = %24
  br label %138

138:                                              ; preds = %23, %137, %132, %121, %120, %104, %72, %66, %64, %51, %44, %35, %26
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeconcat(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = call ptr @previousinstruction(ptr noundef %11)
  store ptr %12, ptr %9, align 8
  %13 = load ptr, ptr %9, align 8
  %14 = load i32, ptr %13, align 4
  %15 = lshr i32 %14, 0
  %16 = and i32 %15, 127
  %17 = icmp eq i32 %16, 53
  br i1 %17, label %18, label %44

18:                                               ; preds = %4
  %19 = load ptr, ptr %9, align 8
  %20 = load i32, ptr %19, align 4
  %21 = lshr i32 %20, 16
  %22 = and i32 %21, 255
  store i32 %22, ptr %10, align 4
  %23 = load ptr, ptr %5, align 8
  %24 = load ptr, ptr %7, align 8
  call void @freeexp(ptr noundef %23, ptr noundef %24)
  %25 = load ptr, ptr %9, align 8
  %26 = load i32, ptr %25, align 4
  %27 = and i32 %26, -32641
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.expdesc, ptr %28, i32 0, i32 1
  %30 = load i32, ptr %29, align 8
  %31 = shl i32 %30, 7
  %32 = and i32 %31, 32640
  %33 = or i32 %27, %32
  %34 = load ptr, ptr %9, align 8
  store i32 %33, ptr %34, align 4
  %35 = load ptr, ptr %9, align 8
  %36 = load i32, ptr %35, align 4
  %37 = and i32 %36, -16711681
  %38 = load i32, ptr %10, align 4
  %39 = add nsw i32 %38, 1
  %40 = shl i32 %39, 16
  %41 = and i32 %40, 16711680
  %42 = or i32 %37, %41
  %43 = load ptr, ptr %9, align 8
  store i32 %42, ptr %43, align 4
  br label %54

44:                                               ; preds = %4
  %45 = load ptr, ptr %5, align 8
  %46 = load ptr, ptr %6, align 8
  %47 = getelementptr inbounds %struct.expdesc, ptr %46, i32 0, i32 1
  %48 = load i32, ptr %47, align 8
  %49 = call i32 @luaK_codeABCk(ptr noundef %45, i32 noundef 53, i32 noundef %48, i32 noundef 2, i32 noundef 0, i32 noundef 0)
  %50 = load ptr, ptr %5, align 8
  %51 = load ptr, ptr %7, align 8
  call void @freeexp(ptr noundef %50, ptr noundef %51)
  %52 = load ptr, ptr %5, align 8
  %53 = load i32, ptr %8, align 4
  call void @luaK_fixline(ptr noundef %52, i32 noundef %53)
  br label %54

54:                                               ; preds = %44, %18
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codecommutative(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  store i32 0, ptr %11, align 4
  %12 = load ptr, ptr %8, align 8
  %13 = call i32 @tonumeral(ptr noundef %12, ptr noundef null)
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %5
  %16 = load ptr, ptr %8, align 8
  %17 = load ptr, ptr %9, align 8
  call void @swapexps(ptr noundef %16, ptr noundef %17)
  store i32 1, ptr %11, align 4
  br label %18

18:                                               ; preds = %15, %5
  %19 = load i32, ptr %7, align 4
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %21, label %31

21:                                               ; preds = %18
  %22 = load ptr, ptr %9, align 8
  %23 = call i32 @isSCint(ptr noundef %22)
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %25, label %31

25:                                               ; preds = %21
  %26 = load ptr, ptr %6, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = load ptr, ptr %9, align 8
  %29 = load i32, ptr %11, align 4
  %30 = load i32, ptr %10, align 4
  call void @codebini(ptr noundef %26, i32 noundef 21, ptr noundef %27, ptr noundef %28, i32 noundef %29, i32 noundef %30, i32 noundef 6)
  br label %38

31:                                               ; preds = %21, %18
  %32 = load ptr, ptr %6, align 8
  %33 = load i32, ptr %7, align 4
  %34 = load ptr, ptr %8, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = load i32, ptr %11, align 4
  %37 = load i32, ptr %10, align 4
  call void @codearith(ptr noundef %32, i32 noundef %33, ptr noundef %34, ptr noundef %35, i32 noundef %36, i32 noundef %37)
  br label %38

38:                                               ; preds = %31, %25
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @finishbinexpneg(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i64, align 8
  %15 = alloca i32, align 4
  store ptr %0, ptr %8, align 8
  store ptr %1, ptr %9, align 8
  store ptr %2, ptr %10, align 8
  store i32 %3, ptr %11, align 4
  store i32 %4, ptr %12, align 4
  store i32 %5, ptr %13, align 4
  %16 = load ptr, ptr %10, align 8
  %17 = call i32 @isKint(ptr noundef %16)
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %20, label %19

19:                                               ; preds = %6
  store i32 0, ptr %7, align 4
  br label %74

20:                                               ; preds = %6
  %21 = load ptr, ptr %10, align 8
  %22 = getelementptr inbounds %struct.expdesc, ptr %21, i32 0, i32 1
  %23 = load i64, ptr %22, align 8
  store i64 %23, ptr %14, align 8
  %24 = load i64, ptr %14, align 8
  %25 = call i32 @fitsC(i64 noundef %24)
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %32

27:                                               ; preds = %20
  %28 = load i64, ptr %14, align 8
  %29 = sub nsw i64 0, %28
  %30 = call i32 @fitsC(i64 noundef %29)
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %33, label %32

32:                                               ; preds = %27, %20
  store i32 0, ptr %7, align 4
  br label %74

33:                                               ; preds = %27
  %34 = load i64, ptr %14, align 8
  %35 = trunc i64 %34 to i32
  store i32 %35, ptr %15, align 4
  %36 = load ptr, ptr %8, align 8
  %37 = load ptr, ptr %9, align 8
  %38 = load ptr, ptr %10, align 8
  %39 = load i32, ptr %11, align 4
  %40 = load i32, ptr %15, align 4
  %41 = sub nsw i32 0, %40
  %42 = add nsw i32 %41, 127
  %43 = load i32, ptr %12, align 4
  %44 = load i32, ptr %13, align 4
  call void @finishbinexpval(ptr noundef %36, ptr noundef %37, ptr noundef %38, i32 noundef %39, i32 noundef %42, i32 noundef 0, i32 noundef %43, i32 noundef 47, i32 noundef %44)
  %45 = load ptr, ptr %8, align 8
  %46 = getelementptr inbounds %struct.FuncState, ptr %45, i32 0, i32 0
  %47 = load ptr, ptr %46, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 16
  %49 = load ptr, ptr %48, align 8
  %50 = load ptr, ptr %8, align 8
  %51 = getelementptr inbounds %struct.FuncState, ptr %50, i32 0, i32 4
  %52 = load i32, ptr %51, align 8
  %53 = sub nsw i32 %52, 1
  %54 = sext i32 %53 to i64
  %55 = getelementptr inbounds i32, ptr %49, i64 %54
  %56 = load i32, ptr %55, align 4
  %57 = and i32 %56, -16711681
  %58 = load i32, ptr %15, align 4
  %59 = add nsw i32 %58, 127
  %60 = shl i32 %59, 16
  %61 = and i32 %60, 16711680
  %62 = or i32 %57, %61
  %63 = load ptr, ptr %8, align 8
  %64 = getelementptr inbounds %struct.FuncState, ptr %63, i32 0, i32 0
  %65 = load ptr, ptr %64, align 8
  %66 = getelementptr inbounds %struct.Proto, ptr %65, i32 0, i32 16
  %67 = load ptr, ptr %66, align 8
  %68 = load ptr, ptr %8, align 8
  %69 = getelementptr inbounds %struct.FuncState, ptr %68, i32 0, i32 4
  %70 = load i32, ptr %69, align 8
  %71 = sub nsw i32 %70, 1
  %72 = sext i32 %71 to i64
  %73 = getelementptr inbounds i32, ptr %67, i64 %72
  store i32 %62, ptr %73, align 4
  store i32 1, ptr %7, align 4
  br label %74

74:                                               ; preds = %33, %32, %19
  %75 = load i32, ptr %7, align 4
  ret i32 %75
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codearith(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %13 = load ptr, ptr %10, align 8
  %14 = call i32 @tonumeral(ptr noundef %13, ptr noundef null)
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %28

16:                                               ; preds = %6
  %17 = load ptr, ptr %7, align 8
  %18 = load ptr, ptr %10, align 8
  %19 = call i32 @luaK_exp2K(ptr noundef %17, ptr noundef %18)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %28

21:                                               ; preds = %16
  %22 = load ptr, ptr %7, align 8
  %23 = load i32, ptr %8, align 4
  %24 = load ptr, ptr %9, align 8
  %25 = load ptr, ptr %10, align 8
  %26 = load i32, ptr %11, align 4
  %27 = load i32, ptr %12, align 4
  call void @codebinK(ptr noundef %22, i32 noundef %23, ptr noundef %24, ptr noundef %25, i32 noundef %26, i32 noundef %27)
  br label %35

28:                                               ; preds = %16, %6
  %29 = load ptr, ptr %7, align 8
  %30 = load i32, ptr %8, align 4
  %31 = load ptr, ptr %9, align 8
  %32 = load ptr, ptr %10, align 8
  %33 = load i32, ptr %11, align 4
  %34 = load i32, ptr %12, align 4
  call void @codebinNoK(ptr noundef %29, i32 noundef %30, ptr noundef %31, ptr noundef %32, i32 noundef %33, i32 noundef %34)
  br label %35

35:                                               ; preds = %28, %21
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codebitwise(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  store i32 0, ptr %11, align 4
  %12 = load ptr, ptr %8, align 8
  %13 = getelementptr inbounds %struct.expdesc, ptr %12, i32 0, i32 0
  %14 = load i32, ptr %13, align 8
  %15 = icmp eq i32 %14, 6
  br i1 %15, label %16, label %19

16:                                               ; preds = %5
  %17 = load ptr, ptr %8, align 8
  %18 = load ptr, ptr %9, align 8
  call void @swapexps(ptr noundef %17, ptr noundef %18)
  store i32 1, ptr %11, align 4
  br label %19

19:                                               ; preds = %16, %5
  %20 = load ptr, ptr %9, align 8
  %21 = getelementptr inbounds %struct.expdesc, ptr %20, i32 0, i32 0
  %22 = load i32, ptr %21, align 8
  %23 = icmp eq i32 %22, 6
  br i1 %23, label %24, label %36

24:                                               ; preds = %19
  %25 = load ptr, ptr %6, align 8
  %26 = load ptr, ptr %9, align 8
  %27 = call i32 @luaK_exp2K(ptr noundef %25, ptr noundef %26)
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %36

29:                                               ; preds = %24
  %30 = load ptr, ptr %6, align 8
  %31 = load i32, ptr %7, align 4
  %32 = load ptr, ptr %8, align 8
  %33 = load ptr, ptr %9, align 8
  %34 = load i32, ptr %11, align 4
  %35 = load i32, ptr %10, align 4
  call void @codebinK(ptr noundef %30, i32 noundef %31, ptr noundef %32, ptr noundef %33, i32 noundef %34, i32 noundef %35)
  br label %43

36:                                               ; preds = %24, %19
  %37 = load ptr, ptr %6, align 8
  %38 = load i32, ptr %7, align 4
  %39 = load ptr, ptr %8, align 8
  %40 = load ptr, ptr %9, align 8
  %41 = load i32, ptr %11, align 4
  %42 = load i32, ptr %10, align 4
  call void @codebinNoK(ptr noundef %37, i32 noundef %38, ptr noundef %39, ptr noundef %40, i32 noundef %41, i32 noundef %42)
  br label %43

43:                                               ; preds = %36, %29
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isSCint(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @isKint(ptr noundef %3)
  %5 = icmp ne i32 %4, 0
  br i1 %5, label %6, label %12

6:                                                ; preds = %1
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 1
  %9 = load i64, ptr %8, align 8
  %10 = call i32 @fitsC(i64 noundef %9)
  %11 = icmp ne i32 %10, 0
  br label %12

12:                                               ; preds = %6, %1
  %13 = phi i1 [ false, %1 ], [ %11, %6 ]
  %14 = zext i1 %13 to i32
  ret i32 %14
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @swapexps(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.expdesc, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %5, ptr align 8 %6, i64 24, i1 false)
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7, ptr align 8 %8, i64 24, i1 false)
  %9 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %9, ptr align 8 %5, i64 24, i1 false)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codebini(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4, i32 noundef %5, i32 noundef %6) #2 {
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  store ptr %0, ptr %8, align 8
  store i32 %1, ptr %9, align 4
  store ptr %2, ptr %10, align 8
  store ptr %3, ptr %11, align 8
  store i32 %4, ptr %12, align 4
  store i32 %5, ptr %13, align 4
  store i32 %6, ptr %14, align 4
  %16 = load ptr, ptr %11, align 8
  %17 = getelementptr inbounds %struct.expdesc, ptr %16, i32 0, i32 1
  %18 = load i64, ptr %17, align 8
  %19 = trunc i64 %18 to i32
  %20 = add nsw i32 %19, 127
  store i32 %20, ptr %15, align 4
  %21 = load ptr, ptr %8, align 8
  %22 = load ptr, ptr %10, align 8
  %23 = load ptr, ptr %11, align 8
  %24 = load i32, ptr %9, align 4
  %25 = load i32, ptr %15, align 4
  %26 = load i32, ptr %12, align 4
  %27 = load i32, ptr %13, align 4
  %28 = load i32, ptr %14, align 4
  call void @finishbinexpval(ptr noundef %21, ptr noundef %22, ptr noundef %23, i32 noundef %24, i32 noundef %25, i32 noundef %26, i32 noundef %27, i32 noundef 47, i32 noundef %28)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codebinexpval(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  %13 = load i32, ptr %7, align 4
  %14 = call i32 @binopr2op(i32 noundef %13, i32 noundef 0, i32 noundef 34)
  store i32 %14, ptr %11, align 4
  %15 = load ptr, ptr %6, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = call i32 @luaK_exp2anyreg(ptr noundef %15, ptr noundef %16)
  store i32 %17, ptr %12, align 4
  %18 = load ptr, ptr %6, align 8
  %19 = load ptr, ptr %8, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = load i32, ptr %11, align 4
  %22 = load i32, ptr %12, align 4
  %23 = load i32, ptr %10, align 4
  %24 = load i32, ptr %7, align 4
  %25 = call i32 @binopr2TM(i32 noundef %24)
  call void @finishbinexpval(ptr noundef %18, ptr noundef %19, ptr noundef %20, i32 noundef %21, i32 noundef %22, i32 noundef 0, i32 noundef %23, i32 noundef 46, i32 noundef %25)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeeq(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  store i32 0, ptr %12, align 4
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  %17 = icmp ne i32 %16, 8
  br i1 %17, label %18, label %21

18:                                               ; preds = %4
  %19 = load ptr, ptr %7, align 8
  %20 = load ptr, ptr %8, align 8
  call void @swapexps(ptr noundef %19, ptr noundef %20)
  br label %21

21:                                               ; preds = %18, %4
  %22 = load ptr, ptr %5, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = call i32 @luaK_exp2anyreg(ptr noundef %22, ptr noundef %23)
  store i32 %24, ptr %9, align 4
  %25 = load ptr, ptr %8, align 8
  %26 = call i32 @isSCnumber(ptr noundef %25, ptr noundef %11, ptr noundef %12)
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %30

28:                                               ; preds = %21
  store i32 61, ptr %13, align 4
  %29 = load i32, ptr %11, align 4
  store i32 %29, ptr %10, align 4
  br label %44

30:                                               ; preds = %21
  %31 = load ptr, ptr %5, align 8
  %32 = load ptr, ptr %8, align 8
  %33 = call i32 @exp2RK(ptr noundef %31, ptr noundef %32)
  %34 = icmp ne i32 %33, 0
  br i1 %34, label %35, label %39

35:                                               ; preds = %30
  store i32 60, ptr %13, align 4
  %36 = load ptr, ptr %8, align 8
  %37 = getelementptr inbounds %struct.expdesc, ptr %36, i32 0, i32 1
  %38 = load i32, ptr %37, align 8
  store i32 %38, ptr %10, align 4
  br label %43

39:                                               ; preds = %30
  store i32 57, ptr %13, align 4
  %40 = load ptr, ptr %5, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = call i32 @luaK_exp2anyreg(ptr noundef %40, ptr noundef %41)
  store i32 %42, ptr %10, align 4
  br label %43

43:                                               ; preds = %39, %35
  br label %44

44:                                               ; preds = %43, %28
  %45 = load ptr, ptr %5, align 8
  %46 = load ptr, ptr %7, align 8
  %47 = load ptr, ptr %8, align 8
  call void @freeexps(ptr noundef %45, ptr noundef %46, ptr noundef %47)
  %48 = load ptr, ptr %5, align 8
  %49 = load i32, ptr %13, align 4
  %50 = load i32, ptr %9, align 4
  %51 = load i32, ptr %10, align 4
  %52 = load i32, ptr %12, align 4
  %53 = load i32, ptr %6, align 4
  %54 = icmp eq i32 %53, 13
  %55 = zext i1 %54 to i32
  %56 = call i32 @condjump(ptr noundef %48, i32 noundef %49, i32 noundef %50, i32 noundef %51, i32 noundef %52, i32 noundef %55)
  %57 = load ptr, ptr %7, align 8
  %58 = getelementptr inbounds %struct.expdesc, ptr %57, i32 0, i32 1
  store i32 %56, ptr %58, align 8
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.expdesc, ptr %59, i32 0, i32 0
  store i32 16, ptr %60, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codeorder(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  store i32 0, ptr %12, align 4
  %14 = load ptr, ptr %8, align 8
  %15 = call i32 @isSCnumber(ptr noundef %14, ptr noundef %11, ptr noundef %12)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %24

17:                                               ; preds = %4
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = call i32 @luaK_exp2anyreg(ptr noundef %18, ptr noundef %19)
  store i32 %20, ptr %9, align 4
  %21 = load i32, ptr %11, align 4
  store i32 %21, ptr %10, align 4
  %22 = load i32, ptr %6, align 4
  %23 = call i32 @binopr2op(i32 noundef %22, i32 noundef 14, i32 noundef 62)
  store i32 %23, ptr %13, align 4
  br label %45

24:                                               ; preds = %4
  %25 = load ptr, ptr %7, align 8
  %26 = call i32 @isSCnumber(ptr noundef %25, ptr noundef %11, ptr noundef %12)
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %35

28:                                               ; preds = %24
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %8, align 8
  %31 = call i32 @luaK_exp2anyreg(ptr noundef %29, ptr noundef %30)
  store i32 %31, ptr %9, align 4
  %32 = load i32, ptr %11, align 4
  store i32 %32, ptr %10, align 4
  %33 = load i32, ptr %6, align 4
  %34 = call i32 @binopr2op(i32 noundef %33, i32 noundef 14, i32 noundef 64)
  store i32 %34, ptr %13, align 4
  br label %44

35:                                               ; preds = %24
  %36 = load ptr, ptr %5, align 8
  %37 = load ptr, ptr %7, align 8
  %38 = call i32 @luaK_exp2anyreg(ptr noundef %36, ptr noundef %37)
  store i32 %38, ptr %9, align 4
  %39 = load ptr, ptr %5, align 8
  %40 = load ptr, ptr %8, align 8
  %41 = call i32 @luaK_exp2anyreg(ptr noundef %39, ptr noundef %40)
  store i32 %41, ptr %10, align 4
  %42 = load i32, ptr %6, align 4
  %43 = call i32 @binopr2op(i32 noundef %42, i32 noundef 14, i32 noundef 58)
  store i32 %43, ptr %13, align 4
  br label %44

44:                                               ; preds = %35, %28
  br label %45

45:                                               ; preds = %44, %17
  %46 = load ptr, ptr %5, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = load ptr, ptr %8, align 8
  call void @freeexps(ptr noundef %46, ptr noundef %47, ptr noundef %48)
  %49 = load ptr, ptr %5, align 8
  %50 = load i32, ptr %13, align 4
  %51 = load i32, ptr %9, align 4
  %52 = load i32, ptr %10, align 4
  %53 = load i32, ptr %12, align 4
  %54 = call i32 @condjump(ptr noundef %49, i32 noundef %50, i32 noundef %51, i32 noundef %52, i32 noundef %53, i32 noundef 1)
  %55 = load ptr, ptr %7, align 8
  %56 = getelementptr inbounds %struct.expdesc, ptr %55, i32 0, i32 1
  store i32 %54, ptr %56, align 8
  %57 = load ptr, ptr %7, align 8
  %58 = getelementptr inbounds %struct.expdesc, ptr %57, i32 0, i32 0
  store i32 16, ptr %58, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_fixline(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  call void @removelastlineinfo(ptr noundef %5)
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  %10 = load i32, ptr %4, align 4
  call void @savelineinfo(ptr noundef %6, ptr noundef %9, i32 noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @removelastlineinfo(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.FuncState, ptr %5, i32 0, i32 0
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 4
  %10 = load i32, ptr %9, align 8
  %11 = sub nsw i32 %10, 1
  store i32 %11, ptr %4, align 4
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 19
  %14 = load ptr, ptr %13, align 8
  %15 = load i32, ptr %4, align 4
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds i8, ptr %14, i64 %16
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp ne i32 %19, -128
  br i1 %20, label %21, label %38

21:                                               ; preds = %1
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 19
  %24 = load ptr, ptr %23, align 8
  %25 = load i32, ptr %4, align 4
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds i8, ptr %24, i64 %26
  %28 = load i8, ptr %27, align 1
  %29 = sext i8 %28 to i32
  %30 = load ptr, ptr %2, align 8
  %31 = getelementptr inbounds %struct.FuncState, ptr %30, i32 0, i32 6
  %32 = load i32, ptr %31, align 8
  %33 = sub nsw i32 %32, %29
  store i32 %33, ptr %31, align 8
  %34 = load ptr, ptr %2, align 8
  %35 = getelementptr inbounds %struct.FuncState, ptr %34, i32 0, i32 16
  %36 = load i8, ptr %35, align 1
  %37 = add i8 %36, -1
  store i8 %37, ptr %35, align 1
  br label %45

38:                                               ; preds = %1
  %39 = load ptr, ptr %2, align 8
  %40 = getelementptr inbounds %struct.FuncState, ptr %39, i32 0, i32 9
  %41 = load i32, ptr %40, align 4
  %42 = add nsw i32 %41, -1
  store i32 %42, ptr %40, align 4
  %43 = load ptr, ptr %2, align 8
  %44 = getelementptr inbounds %struct.FuncState, ptr %43, i32 0, i32 16
  store i8 -127, ptr %44, align 1
  br label %45

45:                                               ; preds = %38, %21
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_settablesize(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #2 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.FuncState, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 16
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %7, align 4
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds i32, ptr %20, i64 %22
  store ptr %23, ptr %11, align 8
  %24 = load i32, ptr %10, align 4
  %25 = icmp ne i32 %24, 0
  br i1 %25, label %26, label %30

26:                                               ; preds = %5
  %27 = load i32, ptr %10, align 4
  %28 = call i32 @luaO_ceillog2(i32 noundef %27)
  %29 = add nsw i32 %28, 1
  br label %31

30:                                               ; preds = %5
  br label %31

31:                                               ; preds = %30, %26
  %32 = phi i32 [ %29, %26 ], [ 0, %30 ]
  store i32 %32, ptr %12, align 4
  %33 = load i32, ptr %9, align 4
  %34 = sdiv i32 %33, 256
  store i32 %34, ptr %13, align 4
  %35 = load i32, ptr %9, align 4
  %36 = srem i32 %35, 256
  store i32 %36, ptr %14, align 4
  %37 = load i32, ptr %13, align 4
  %38 = icmp sgt i32 %37, 0
  %39 = zext i1 %38 to i32
  store i32 %39, ptr %15, align 4
  %40 = load i32, ptr %8, align 4
  %41 = shl i32 %40, 7
  %42 = or i32 19, %41
  %43 = load i32, ptr %12, align 4
  %44 = shl i32 %43, 16
  %45 = or i32 %42, %44
  %46 = load i32, ptr %14, align 4
  %47 = shl i32 %46, 24
  %48 = or i32 %45, %47
  %49 = load i32, ptr %15, align 4
  %50 = shl i32 %49, 15
  %51 = or i32 %48, %50
  %52 = load ptr, ptr %11, align 8
  store i32 %51, ptr %52, align 4
  %53 = load i32, ptr %13, align 4
  %54 = shl i32 %53, 7
  %55 = or i32 82, %54
  %56 = load ptr, ptr %11, align 8
  %57 = getelementptr inbounds i32, ptr %56, i64 1
  store i32 %55, ptr %57, align 4
  ret void
}

declare hidden i32 @luaO_ceillog2(i32 noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_setlist(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #2 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %10 = load i32, ptr %8, align 4
  %11 = icmp eq i32 %10, -1
  br i1 %11, label %12, label %13

12:                                               ; preds = %4
  store i32 0, ptr %8, align 4
  br label %13

13:                                               ; preds = %12, %4
  %14 = load i32, ptr %7, align 4
  %15 = icmp sle i32 %14, 255
  br i1 %15, label %16, label %22

16:                                               ; preds = %13
  %17 = load ptr, ptr %5, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load i32, ptr %8, align 4
  %20 = load i32, ptr %7, align 4
  %21 = call i32 @luaK_codeABCk(ptr noundef %17, i32 noundef 78, i32 noundef %18, i32 noundef %19, i32 noundef %20, i32 noundef 0)
  br label %35

22:                                               ; preds = %13
  %23 = load i32, ptr %7, align 4
  %24 = sdiv i32 %23, 256
  store i32 %24, ptr %9, align 4
  %25 = load i32, ptr %7, align 4
  %26 = srem i32 %25, 256
  store i32 %26, ptr %7, align 4
  %27 = load ptr, ptr %5, align 8
  %28 = load i32, ptr %6, align 4
  %29 = load i32, ptr %8, align 4
  %30 = load i32, ptr %7, align 4
  %31 = call i32 @luaK_codeABCk(ptr noundef %27, i32 noundef 78, i32 noundef %28, i32 noundef %29, i32 noundef %30, i32 noundef 1)
  %32 = load ptr, ptr %5, align 8
  %33 = load i32, ptr %9, align 4
  %34 = call i32 @codeextraarg(ptr noundef %32, i32 noundef %33)
  br label %35

35:                                               ; preds = %22, %16
  %36 = load i32, ptr %6, align 4
  %37 = add nsw i32 %36, 1
  %38 = trunc i32 %37 to i8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.FuncState, ptr %39, i32 0, i32 15
  store i8 %38, ptr %40, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @codeextraarg(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = shl i32 %6, 7
  %8 = or i32 82, %7
  %9 = call i32 @luaK_code(ptr noundef %5, i32 noundef %8)
  ret i32 %9
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaK_finish(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %4, align 8
  store i32 0, ptr %3, align 4
  br label %10

10:                                               ; preds = %87, %1
  %11 = load i32, ptr %3, align 4
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.FuncState, ptr %12, i32 0, i32 4
  %14 = load i32, ptr %13, align 8
  %15 = icmp slt i32 %11, %14
  br i1 %15, label %16, label %90

16:                                               ; preds = %10
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.Proto, ptr %17, i32 0, i32 16
  %19 = load ptr, ptr %18, align 8
  %20 = load i32, ptr %3, align 4
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds i32, ptr %19, i64 %21
  store ptr %22, ptr %5, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = load i32, ptr %23, align 4
  %25 = lshr i32 %24, 0
  %26 = and i32 %25, 127
  switch i32 %26, label %85 [
    i32 71, label %27
    i32 72, label %27
    i32 70, label %46
    i32 69, label %46
    i32 56, label %76
  ]

27:                                               ; preds = %16, %16
  %28 = load ptr, ptr %2, align 8
  %29 = getelementptr inbounds %struct.FuncState, ptr %28, i32 0, i32 17
  %30 = load i8, ptr %29, align 2
  %31 = zext i8 %30 to i32
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %40, label %33

33:                                               ; preds = %27
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 4
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %40, label %39

39:                                               ; preds = %33
  br label %86

40:                                               ; preds = %33, %27
  %41 = load ptr, ptr %5, align 8
  %42 = load i32, ptr %41, align 4
  %43 = and i32 %42, -128
  %44 = or i32 %43, 70
  %45 = load ptr, ptr %5, align 8
  store i32 %44, ptr %45, align 4
  br label %46

46:                                               ; preds = %16, %16, %40
  %47 = load ptr, ptr %2, align 8
  %48 = getelementptr inbounds %struct.FuncState, ptr %47, i32 0, i32 17
  %49 = load i8, ptr %48, align 2
  %50 = icmp ne i8 %49, 0
  br i1 %50, label %51, label %57

51:                                               ; preds = %46
  %52 = load ptr, ptr %5, align 8
  %53 = load i32, ptr %52, align 4
  %54 = and i32 %53, -32769
  %55 = or i32 %54, 32768
  %56 = load ptr, ptr %5, align 8
  store i32 %55, ptr %56, align 4
  br label %57

57:                                               ; preds = %51, %46
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 4
  %60 = load i8, ptr %59, align 1
  %61 = icmp ne i8 %60, 0
  br i1 %61, label %62, label %75

62:                                               ; preds = %57
  %63 = load ptr, ptr %5, align 8
  %64 = load i32, ptr %63, align 4
  %65 = and i32 %64, 16777215
  %66 = load ptr, ptr %4, align 8
  %67 = getelementptr inbounds %struct.Proto, ptr %66, i32 0, i32 3
  %68 = load i8, ptr %67, align 2
  %69 = zext i8 %68 to i32
  %70 = add nsw i32 %69, 1
  %71 = shl i32 %70, 24
  %72 = and i32 %71, -16777216
  %73 = or i32 %65, %72
  %74 = load ptr, ptr %5, align 8
  store i32 %73, ptr %74, align 4
  br label %75

75:                                               ; preds = %62, %57
  br label %86

76:                                               ; preds = %16
  %77 = load ptr, ptr %4, align 8
  %78 = getelementptr inbounds %struct.Proto, ptr %77, i32 0, i32 16
  %79 = load ptr, ptr %78, align 8
  %80 = load i32, ptr %3, align 4
  %81 = call i32 @finaltarget(ptr noundef %79, i32 noundef %80)
  store i32 %81, ptr %6, align 4
  %82 = load ptr, ptr %2, align 8
  %83 = load i32, ptr %3, align 4
  %84 = load i32, ptr %6, align 4
  call void @fixjump(ptr noundef %82, i32 noundef %83, i32 noundef %84)
  br label %86

85:                                               ; preds = %16
  br label %86

86:                                               ; preds = %85, %76, %75, %39
  br label %87

87:                                               ; preds = %86
  %88 = load i32, ptr %3, align 4
  %89 = add nsw i32 %88, 1
  store i32 %89, ptr %3, align 4
  br label %10, !llvm.loop !9

90:                                               ; preds = %10
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @finaltarget(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  store i32 0, ptr %5, align 4
  br label %7

7:                                                ; preds = %30, %2
  %8 = load i32, ptr %5, align 4
  %9 = icmp slt i32 %8, 100
  br i1 %9, label %10, label %33

10:                                               ; preds = %7
  %11 = load ptr, ptr %3, align 8
  %12 = load i32, ptr %4, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds i32, ptr %11, i64 %13
  %15 = load i32, ptr %14, align 4
  store i32 %15, ptr %6, align 4
  %16 = load i32, ptr %6, align 4
  %17 = lshr i32 %16, 0
  %18 = and i32 %17, 127
  %19 = icmp ne i32 %18, 56
  br i1 %19, label %20, label %21

20:                                               ; preds = %10
  br label %33

21:                                               ; preds = %10
  %22 = load i32, ptr %6, align 4
  %23 = lshr i32 %22, 7
  %24 = and i32 %23, 33554431
  %25 = sub nsw i32 %24, 16777215
  %26 = add nsw i32 %25, 1
  %27 = load i32, ptr %4, align 4
  %28 = add nsw i32 %27, %26
  store i32 %28, ptr %4, align 4
  br label %29

29:                                               ; preds = %21
  br label %30

30:                                               ; preds = %29
  %31 = load i32, ptr %5, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %5, align 4
  br label %7, !llvm.loop !10

33:                                               ; preds = %20, %7
  %34 = load i32, ptr %4, align 4
  ret i32 %34
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @patchtestreg(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call ptr @getjumpcontrol(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %8, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = load i32, ptr %12, align 4
  %14 = lshr i32 %13, 0
  %15 = and i32 %14, 127
  %16 = icmp ne i32 %15, 67
  br i1 %16, label %17, label %18

17:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %54

18:                                               ; preds = %3
  %19 = load i32, ptr %7, align 4
  %20 = icmp ne i32 %19, 255
  br i1 %20, label %21, label %37

21:                                               ; preds = %18
  %22 = load i32, ptr %7, align 4
  %23 = load ptr, ptr %8, align 8
  %24 = load i32, ptr %23, align 4
  %25 = lshr i32 %24, 16
  %26 = and i32 %25, 255
  %27 = icmp ne i32 %22, %26
  br i1 %27, label %28, label %37

28:                                               ; preds = %21
  %29 = load ptr, ptr %8, align 8
  %30 = load i32, ptr %29, align 4
  %31 = and i32 %30, -32641
  %32 = load i32, ptr %7, align 4
  %33 = shl i32 %32, 7
  %34 = and i32 %33, 32640
  %35 = or i32 %31, %34
  %36 = load ptr, ptr %8, align 8
  store i32 %35, ptr %36, align 4
  br label %53

37:                                               ; preds = %21, %18
  %38 = load ptr, ptr %8, align 8
  %39 = load i32, ptr %38, align 4
  %40 = lshr i32 %39, 16
  %41 = and i32 %40, 255
  %42 = shl i32 %41, 7
  %43 = or i32 66, %42
  %44 = or i32 %43, 0
  %45 = or i32 %44, 0
  %46 = load ptr, ptr %8, align 8
  %47 = load i32, ptr %46, align 4
  %48 = lshr i32 %47, 15
  %49 = and i32 %48, 1
  %50 = shl i32 %49, 15
  %51 = or i32 %45, %50
  %52 = load ptr, ptr %8, align 8
  store i32 %51, ptr %52, align 4
  br label %53

53:                                               ; preds = %37, %28
  store i32 1, ptr %4, align 4
  br label %54

54:                                               ; preds = %53, %17
  %55 = load i32, ptr %4, align 4
  ret i32 %55
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getjumpcontrol(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.FuncState, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 16
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %5, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds i32, ptr %11, i64 %13
  store ptr %14, ptr %6, align 8
  %15 = load i32, ptr %5, align 4
  %16 = icmp sge i32 %15, 1
  br i1 %16, label %17, label %32

17:                                               ; preds = %2
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds i32, ptr %18, i64 -1
  %20 = load i32, ptr %19, align 4
  %21 = lshr i32 %20, 0
  %22 = and i32 %21, 127
  %23 = zext i32 %22 to i64
  %24 = getelementptr inbounds [83 x i8], ptr @luaP_opmodes, i64 0, i64 %23
  %25 = load i8, ptr %24, align 1
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 16
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %32

29:                                               ; preds = %17
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds i32, ptr %30, i64 -1
  store ptr %31, ptr %3, align 8
  br label %34

32:                                               ; preds = %17, %2
  %33 = load ptr, ptr %6, align 8
  store ptr %33, ptr %3, align 8
  br label %34

34:                                               ; preds = %32, %29
  %35 = load ptr, ptr %3, align 8
  ret ptr %35
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.abs.i32(i32, i1 immarg) #5

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @addk(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %struct.TValue, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.FuncState, ptr %17, i32 0, i32 2
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %9, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = getelementptr inbounds %struct.FuncState, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %10, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.FuncState, ptr %25, i32 0, i32 2
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.LexState, ptr %27, i32 0, i32 9
  %29 = load ptr, ptr %28, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = call ptr @luaH_get(ptr noundef %29, ptr noundef %30)
  store ptr %31, ptr %11, align 8
  %32 = load ptr, ptr %11, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = icmp eq i32 %35, 3
  br i1 %36, label %37, label %77

37:                                               ; preds = %3
  %38 = load ptr, ptr %11, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  %40 = load i64, ptr %39, align 8
  %41 = trunc i64 %40 to i32
  store i32 %41, ptr %12, align 4
  %42 = load i32, ptr %12, align 4
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.FuncState, ptr %43, i32 0, i32 7
  %45 = load i32, ptr %44, align 4
  %46 = icmp slt i32 %42, %45
  br i1 %46, label %47, label %76

47:                                               ; preds = %37
  %48 = load ptr, ptr %10, align 8
  %49 = getelementptr inbounds %struct.Proto, ptr %48, i32 0, i32 15
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %12, align 4
  %52 = sext i32 %51 to i64
  %53 = getelementptr inbounds %struct.TValue, ptr %50, i64 %52
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 1
  %55 = load i8, ptr %54, align 8
  %56 = zext i8 %55 to i32
  %57 = and i32 %56, 63
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 1
  %60 = load i8, ptr %59, align 8
  %61 = zext i8 %60 to i32
  %62 = and i32 %61, 63
  %63 = icmp eq i32 %57, %62
  br i1 %63, label %64, label %76

64:                                               ; preds = %47
  %65 = load ptr, ptr %10, align 8
  %66 = getelementptr inbounds %struct.Proto, ptr %65, i32 0, i32 15
  %67 = load ptr, ptr %66, align 8
  %68 = load i32, ptr %12, align 4
  %69 = sext i32 %68 to i64
  %70 = getelementptr inbounds %struct.TValue, ptr %67, i64 %69
  %71 = load ptr, ptr %7, align 8
  %72 = call i32 @luaV_equalobj(ptr noundef null, ptr noundef %70, ptr noundef %71)
  %73 = icmp ne i32 %72, 0
  br i1 %73, label %74, label %76

74:                                               ; preds = %64
  %75 = load i32, ptr %12, align 4
  store i32 %75, ptr %4, align 4
  br label %178

76:                                               ; preds = %64, %47, %37
  br label %77

77:                                               ; preds = %76, %3
  %78 = load ptr, ptr %10, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 7
  %80 = load i32, ptr %79, align 4
  store i32 %80, ptr %13, align 4
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds %struct.FuncState, ptr %81, i32 0, i32 7
  %83 = load i32, ptr %82, align 4
  store i32 %83, ptr %12, align 4
  store ptr %8, ptr %14, align 8
  %84 = load i32, ptr %12, align 4
  %85 = sext i32 %84 to i64
  %86 = load ptr, ptr %14, align 8
  %87 = getelementptr inbounds %struct.TValue, ptr %86, i32 0, i32 0
  store i64 %85, ptr %87, align 8
  %88 = load ptr, ptr %14, align 8
  %89 = getelementptr inbounds %struct.TValue, ptr %88, i32 0, i32 1
  store i8 3, ptr %89, align 8
  %90 = load ptr, ptr %9, align 8
  %91 = load ptr, ptr %5, align 8
  %92 = getelementptr inbounds %struct.FuncState, ptr %91, i32 0, i32 2
  %93 = load ptr, ptr %92, align 8
  %94 = getelementptr inbounds %struct.LexState, ptr %93, i32 0, i32 9
  %95 = load ptr, ptr %94, align 8
  %96 = load ptr, ptr %6, align 8
  %97 = load ptr, ptr %11, align 8
  call void @luaH_finishset(ptr noundef %90, ptr noundef %95, ptr noundef %96, ptr noundef %97, ptr noundef %8)
  %98 = load ptr, ptr %9, align 8
  %99 = load ptr, ptr %10, align 8
  %100 = getelementptr inbounds %struct.Proto, ptr %99, i32 0, i32 15
  %101 = load ptr, ptr %100, align 8
  %102 = load i32, ptr %12, align 4
  %103 = load ptr, ptr %10, align 8
  %104 = getelementptr inbounds %struct.Proto, ptr %103, i32 0, i32 7
  %105 = call ptr @luaM_growaux_(ptr noundef %98, ptr noundef %101, i32 noundef %102, ptr noundef %104, i32 noundef 16, i32 noundef 33554431, ptr noundef @.str.4)
  %106 = load ptr, ptr %10, align 8
  %107 = getelementptr inbounds %struct.Proto, ptr %106, i32 0, i32 15
  store ptr %105, ptr %107, align 8
  br label %108

108:                                              ; preds = %114, %77
  %109 = load i32, ptr %13, align 4
  %110 = load ptr, ptr %10, align 8
  %111 = getelementptr inbounds %struct.Proto, ptr %110, i32 0, i32 7
  %112 = load i32, ptr %111, align 4
  %113 = icmp slt i32 %109, %112
  br i1 %113, label %114, label %123

114:                                              ; preds = %108
  %115 = load ptr, ptr %10, align 8
  %116 = getelementptr inbounds %struct.Proto, ptr %115, i32 0, i32 15
  %117 = load ptr, ptr %116, align 8
  %118 = load i32, ptr %13, align 4
  %119 = add nsw i32 %118, 1
  store i32 %119, ptr %13, align 4
  %120 = sext i32 %118 to i64
  %121 = getelementptr inbounds %struct.TValue, ptr %117, i64 %120
  %122 = getelementptr inbounds %struct.TValue, ptr %121, i32 0, i32 1
  store i8 0, ptr %122, align 8
  br label %108, !llvm.loop !11

123:                                              ; preds = %108
  %124 = load ptr, ptr %10, align 8
  %125 = getelementptr inbounds %struct.Proto, ptr %124, i32 0, i32 15
  %126 = load ptr, ptr %125, align 8
  %127 = load i32, ptr %12, align 4
  %128 = sext i32 %127 to i64
  %129 = getelementptr inbounds %struct.TValue, ptr %126, i64 %128
  store ptr %129, ptr %15, align 8
  %130 = load ptr, ptr %7, align 8
  store ptr %130, ptr %16, align 8
  %131 = load ptr, ptr %15, align 8
  %132 = getelementptr inbounds %struct.TValue, ptr %131, i32 0, i32 0
  %133 = load ptr, ptr %16, align 8
  %134 = getelementptr inbounds %struct.TValue, ptr %133, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %132, ptr align 8 %134, i64 8, i1 false)
  %135 = load ptr, ptr %16, align 8
  %136 = getelementptr inbounds %struct.TValue, ptr %135, i32 0, i32 1
  %137 = load i8, ptr %136, align 8
  %138 = load ptr, ptr %15, align 8
  %139 = getelementptr inbounds %struct.TValue, ptr %138, i32 0, i32 1
  store i8 %137, ptr %139, align 8
  %140 = load ptr, ptr %9, align 8
  %141 = load ptr, ptr %5, align 8
  %142 = getelementptr inbounds %struct.FuncState, ptr %141, i32 0, i32 7
  %143 = load i32, ptr %142, align 4
  %144 = add nsw i32 %143, 1
  store i32 %144, ptr %142, align 4
  %145 = load ptr, ptr %7, align 8
  %146 = getelementptr inbounds %struct.TValue, ptr %145, i32 0, i32 1
  %147 = load i8, ptr %146, align 8
  %148 = zext i8 %147 to i32
  %149 = and i32 %148, 64
  %150 = icmp ne i32 %149, 0
  br i1 %150, label %151, label %175

151:                                              ; preds = %123
  %152 = load ptr, ptr %10, align 8
  %153 = getelementptr inbounds %struct.Proto, ptr %152, i32 0, i32 2
  %154 = load i8, ptr %153, align 1
  %155 = zext i8 %154 to i32
  %156 = and i32 %155, 32
  %157 = icmp ne i32 %156, 0
  br i1 %157, label %158, label %173

158:                                              ; preds = %151
  %159 = load ptr, ptr %7, align 8
  %160 = getelementptr inbounds %struct.TValue, ptr %159, i32 0, i32 0
  %161 = load ptr, ptr %160, align 8
  %162 = getelementptr inbounds %struct.GCObject, ptr %161, i32 0, i32 2
  %163 = load i8, ptr %162, align 1
  %164 = zext i8 %163 to i32
  %165 = and i32 %164, 24
  %166 = icmp ne i32 %165, 0
  br i1 %166, label %167, label %173

167:                                              ; preds = %158
  %168 = load ptr, ptr %9, align 8
  %169 = load ptr, ptr %10, align 8
  %170 = load ptr, ptr %7, align 8
  %171 = getelementptr inbounds %struct.TValue, ptr %170, i32 0, i32 0
  %172 = load ptr, ptr %171, align 8
  call void @luaC_barrier_(ptr noundef %168, ptr noundef %169, ptr noundef %172)
  br label %174

173:                                              ; preds = %158, %151
  br label %174

174:                                              ; preds = %173, %167
  br label %176

175:                                              ; preds = %123
  br label %176

176:                                              ; preds = %175, %174
  %177 = load i32, ptr %12, align 4
  store i32 %177, ptr %4, align 4
  br label %178

178:                                              ; preds = %176, %74
  %179 = load i32, ptr %4, align 4
  ret i32 %179
}

declare hidden ptr @luaH_get(ptr noundef, ptr noundef) #4

declare hidden i32 @luaV_equalobj(ptr noundef, ptr noundef, ptr noundef) #4

declare hidden void @luaH_finishset(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #4

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @discharge2reg(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %5, align 8
  call void @luaK_dischargevars(ptr noundef %8, ptr noundef %9)
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 8
  switch i32 %12, label %79 [
    i32 1, label %13
    i32 3, label %16
    i32 2, label %20
    i32 7, label %24
    i32 4, label %27
    i32 5, label %34
    i32 6, label %40
    i32 17, label %46
    i32 8, label %65
  ]

13:                                               ; preds = %3
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %6, align 4
  call void @luaK_nil(ptr noundef %14, i32 noundef %15, i32 noundef 1)
  br label %80

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %6, align 4
  %19 = call i32 @luaK_codeABCk(ptr noundef %17, i32 noundef 5, i32 noundef %18, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  br label %80

20:                                               ; preds = %3
  %21 = load ptr, ptr %4, align 8
  %22 = load i32, ptr %6, align 4
  %23 = call i32 @luaK_codeABCk(ptr noundef %21, i32 noundef 7, i32 noundef %22, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  br label %80

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  call void @str2K(ptr noundef %25, ptr noundef %26)
  br label %27

27:                                               ; preds = %3, %24
  %28 = load ptr, ptr %4, align 8
  %29 = load i32, ptr %6, align 4
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.expdesc, ptr %30, i32 0, i32 1
  %32 = load i32, ptr %31, align 8
  %33 = call i32 @luaK_codek(ptr noundef %28, i32 noundef %29, i32 noundef %32)
  br label %80

34:                                               ; preds = %3
  %35 = load ptr, ptr %4, align 8
  %36 = load i32, ptr %6, align 4
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.expdesc, ptr %37, i32 0, i32 1
  %39 = load double, ptr %38, align 8
  call void @luaK_float(ptr noundef %35, i32 noundef %36, double noundef %39)
  br label %80

40:                                               ; preds = %3
  %41 = load ptr, ptr %4, align 8
  %42 = load i32, ptr %6, align 4
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.expdesc, ptr %43, i32 0, i32 1
  %45 = load i64, ptr %44, align 8
  call void @luaK_int(ptr noundef %41, i32 noundef %42, i64 noundef %45)
  br label %80

46:                                               ; preds = %3
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.FuncState, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.Proto, ptr %49, i32 0, i32 16
  %51 = load ptr, ptr %50, align 8
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.expdesc, ptr %52, i32 0, i32 1
  %54 = load i32, ptr %53, align 8
  %55 = sext i32 %54 to i64
  %56 = getelementptr inbounds i32, ptr %51, i64 %55
  store ptr %56, ptr %7, align 8
  %57 = load ptr, ptr %7, align 8
  %58 = load i32, ptr %57, align 4
  %59 = and i32 %58, -32641
  %60 = load i32, ptr %6, align 4
  %61 = shl i32 %60, 7
  %62 = and i32 %61, 32640
  %63 = or i32 %59, %62
  %64 = load ptr, ptr %7, align 8
  store i32 %63, ptr %64, align 4
  br label %80

65:                                               ; preds = %3
  %66 = load i32, ptr %6, align 4
  %67 = load ptr, ptr %5, align 8
  %68 = getelementptr inbounds %struct.expdesc, ptr %67, i32 0, i32 1
  %69 = load i32, ptr %68, align 8
  %70 = icmp ne i32 %66, %69
  br i1 %70, label %71, label %78

71:                                               ; preds = %65
  %72 = load ptr, ptr %4, align 8
  %73 = load i32, ptr %6, align 4
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.expdesc, ptr %74, i32 0, i32 1
  %76 = load i32, ptr %75, align 8
  %77 = call i32 @luaK_codeABCk(ptr noundef %72, i32 noundef 0, i32 noundef %73, i32 noundef %76, i32 noundef 0, i32 noundef 0)
  br label %78

78:                                               ; preds = %71, %65
  br label %80

79:                                               ; preds = %3
  br label %86

80:                                               ; preds = %78, %46, %40, %34, %27, %20, %16, %13
  %81 = load i32, ptr %6, align 4
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.expdesc, ptr %82, i32 0, i32 1
  store i32 %81, ptr %83, align 8
  %84 = load ptr, ptr %5, align 8
  %85 = getelementptr inbounds %struct.expdesc, ptr %84, i32 0, i32 0
  store i32 8, ptr %85, align 8
  br label %86

86:                                               ; preds = %80, %79
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @need_value(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  br label %7

7:                                                ; preds = %21, %2
  %8 = load i32, ptr %5, align 4
  %9 = icmp ne i32 %8, -1
  br i1 %9, label %10, label %25

10:                                               ; preds = %7
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = call ptr @getjumpcontrol(ptr noundef %11, i32 noundef %12)
  %14 = load i32, ptr %13, align 4
  store i32 %14, ptr %6, align 4
  %15 = load i32, ptr %6, align 4
  %16 = lshr i32 %15, 0
  %17 = and i32 %16, 127
  %18 = icmp ne i32 %17, 67
  br i1 %18, label %19, label %20

19:                                               ; preds = %10
  store i32 1, ptr %3, align 4
  br label %26

20:                                               ; preds = %10
  br label %21

21:                                               ; preds = %20
  %22 = load ptr, ptr %4, align 8
  %23 = load i32, ptr %5, align 4
  %24 = call i32 @getjump(ptr noundef %22, i32 noundef %23)
  store i32 %24, ptr %5, align 4
  br label %7, !llvm.loop !12

25:                                               ; preds = %7
  store i32 0, ptr %3, align 4
  br label %26

26:                                               ; preds = %25, %19
  %27 = load i32, ptr %3, align 4
  ret i32 %27
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @code_loadbool(ptr noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @luaK_getlabel(ptr noundef %7)
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %6, align 4
  %11 = load i32, ptr %5, align 4
  %12 = call i32 @luaK_codeABCk(ptr noundef %9, i32 noundef %10, i32 noundef %11, i32 noundef 0, i32 noundef 0, i32 noundef 0)
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @luaK_float(ptr noundef %0, i32 noundef %1, double noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca double, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store double %2, ptr %6, align 8
  %8 = load double, ptr %6, align 8
  %9 = call i32 @luaV_flttointeger(double noundef %8, ptr noundef %7, i32 noundef 0)
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %21

11:                                               ; preds = %3
  %12 = load i64, ptr %7, align 8
  %13 = call i32 @fitsBx(i64 noundef %12)
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %15, label %21

15:                                               ; preds = %11
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %5, align 4
  %18 = load i64, ptr %7, align 8
  %19 = trunc i64 %18 to i32
  %20 = call i32 @codeAsBx(ptr noundef %16, i32 noundef 2, i32 noundef %17, i32 noundef %19)
  br label %28

21:                                               ; preds = %11, %3
  %22 = load ptr, ptr %4, align 8
  %23 = load i32, ptr %5, align 4
  %24 = load ptr, ptr %4, align 8
  %25 = load double, ptr %6, align 8
  %26 = call i32 @luaK_numberK(ptr noundef %24, double noundef %25)
  %27 = call i32 @luaK_codek(ptr noundef %22, i32 noundef %23, i32 noundef %26)
  br label %28

28:                                               ; preds = %21, %15
  ret void
}

declare hidden i32 @luaV_flttointeger(double noundef, ptr noundef, i32 noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaK_numberK(ptr noundef %0, double noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca double, align 8
  %6 = alloca %struct.TValue, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca double, align 8
  %11 = alloca double, align 8
  %12 = alloca %struct.TValue, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store double %1, ptr %5, align 8
  store ptr %6, ptr %8, align 8
  %14 = load double, ptr %5, align 8
  %15 = load ptr, ptr %8, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 0
  store double %14, ptr %16, align 8
  %17 = load ptr, ptr %8, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  store i8 19, ptr %18, align 8
  %19 = load double, ptr %5, align 8
  %20 = call i32 @luaV_flttointeger(double noundef %19, ptr noundef %7, i32 noundef 0)
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %25, label %22

22:                                               ; preds = %2
  %23 = load ptr, ptr %4, align 8
  %24 = call i32 @addk(ptr noundef %23, ptr noundef %6, ptr noundef %6)
  store i32 %24, ptr %3, align 4
  br label %45

25:                                               ; preds = %2
  store i32 53, ptr %9, align 4
  %26 = call double @ldexp(double noundef 1.000000e+00, i32 noundef -52) #8
  store double %26, ptr %10, align 8
  %27 = load i64, ptr %7, align 8
  %28 = icmp eq i64 %27, 0
  br i1 %28, label %29, label %31

29:                                               ; preds = %25
  %30 = load double, ptr %10, align 8
  br label %36

31:                                               ; preds = %25
  %32 = load double, ptr %5, align 8
  %33 = load double, ptr %5, align 8
  %34 = load double, ptr %10, align 8
  %35 = call double @llvm.fmuladd.f64(double %33, double %34, double %32)
  br label %36

36:                                               ; preds = %31, %29
  %37 = phi double [ %30, %29 ], [ %35, %31 ]
  store double %37, ptr %11, align 8
  store ptr %12, ptr %13, align 8
  %38 = load double, ptr %11, align 8
  %39 = load ptr, ptr %13, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  store double %38, ptr %40, align 8
  %41 = load ptr, ptr %13, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 1
  store i8 19, ptr %42, align 8
  %43 = load ptr, ptr %4, align 8
  %44 = call i32 @addk(ptr noundef %43, ptr noundef %12, ptr noundef %6)
  store i32 %44, ptr %3, align 4
  br label %45

45:                                               ; preds = %36, %22
  %46 = load i32, ptr %3, align 4
  ret i32 %46
}

; Function Attrs: nounwind
declare double @ldexp(double noundef, i32 noundef) #6

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.fmuladd.f64(double, double, double) #5

; Function Attrs: noinline nounwind optnone uwtable
define internal void @removelastinstruction(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @removelastlineinfo(ptr noundef %3)
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.FuncState, ptr %4, i32 0, i32 4
  %6 = load i32, ptr %5, align 8
  %7 = add nsw i32 %6, -1
  store i32 %7, ptr %5, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @condjump(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store i32 %2, ptr %9, align 4
  store i32 %3, ptr %10, align 4
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %13 = load ptr, ptr %7, align 8
  %14 = load i32, ptr %8, align 4
  %15 = load i32, ptr %9, align 4
  %16 = load i32, ptr %10, align 4
  %17 = load i32, ptr %11, align 4
  %18 = load i32, ptr %12, align 4
  %19 = call i32 @luaK_codeABCk(ptr noundef %13, i32 noundef %14, i32 noundef %15, i32 noundef %16, i32 noundef %17, i32 noundef %18)
  %20 = load ptr, ptr %7, align 8
  %21 = call i32 @luaK_jump(ptr noundef %20)
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @discharge2anyreg(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.expdesc, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  %8 = icmp ne i32 %7, 8
  br i1 %8, label %9, label %18

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  call void @luaK_reserveregs(ptr noundef %10, i32 noundef 1)
  %11 = load ptr, ptr %3, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.FuncState, ptr %13, i32 0, i32 15
  %15 = load i8, ptr %14, align 4
  %16 = zext i8 %15 to i32
  %17 = sub nsw i32 %16, 1
  call void @discharge2reg(ptr noundef %11, ptr noundef %12, i32 noundef %17)
  br label %18

18:                                               ; preds = %9, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @stringK(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.TValue, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %5, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  store ptr %8, ptr %7, align 8
  %9 = load ptr, ptr %7, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.TString, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = or i32 %15, 64
  %17 = trunc i32 %16 to i8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 1
  store i8 %17, ptr %19, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.FuncState, ptr %20, i32 0, i32 2
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.LexState, ptr %22, i32 0, i32 6
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = call i32 @addk(ptr noundef %25, ptr noundef %5, ptr noundef %5)
  ret i32 %26
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isKint(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.expdesc, ptr %3, i32 0, i32 0
  %5 = load i32, ptr %4, align 8
  %6 = icmp eq i32 %5, 6
  br i1 %6, label %7, label %16

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.expdesc, ptr %8, i32 0, i32 2
  %10 = load i32, ptr %9, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.expdesc, ptr %11, i32 0, i32 3
  %13 = load i32, ptr %12, align 4
  %14 = icmp ne i32 %10, %13
  %15 = xor i1 %14, true
  br label %16

16:                                               ; preds = %7, %1
  %17 = phi i1 [ false, %1 ], [ %15, %7 ]
  %18 = zext i1 %17 to i32
  ret i32 %18
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @validop(i32 noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  store i32 %0, ptr %5, align 4
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %9 = load i32, ptr %5, align 4
  switch i32 %9, label %40 [
    i32 7, label %10
    i32 8, label %10
    i32 9, label %10
    i32 10, label %10
    i32 11, label %10
    i32 13, label %10
    i32 5, label %21
    i32 6, label %21
    i32 3, label %21
  ]

10:                                               ; preds = %3, %3, %3, %3, %3, %3
  %11 = load ptr, ptr %6, align 8
  %12 = call i32 @luaV_tointegerns(ptr noundef %11, ptr noundef %8, i32 noundef 0)
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %18

14:                                               ; preds = %10
  %15 = load ptr, ptr %7, align 8
  %16 = call i32 @luaV_tointegerns(ptr noundef %15, ptr noundef %8, i32 noundef 0)
  %17 = icmp ne i32 %16, 0
  br label %18

18:                                               ; preds = %14, %10
  %19 = phi i1 [ false, %10 ], [ %17, %14 ]
  %20 = zext i1 %19 to i32
  store i32 %20, ptr %4, align 4
  br label %41

21:                                               ; preds = %3, %3, %3
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = icmp eq i32 %25, 3
  br i1 %26, label %27, label %32

27:                                               ; preds = %21
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  %30 = load i64, ptr %29, align 8
  %31 = sitofp i64 %30 to double
  br label %36

32:                                               ; preds = %21
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  %35 = load double, ptr %34, align 8
  br label %36

36:                                               ; preds = %32, %27
  %37 = phi double [ %31, %27 ], [ %35, %32 ]
  %38 = fcmp une double %37, 0.000000e+00
  %39 = zext i1 %38 to i32
  store i32 %39, ptr %4, align 4
  br label %41

40:                                               ; preds = %3
  store i32 1, ptr %4, align 4
  br label %41

41:                                               ; preds = %40, %36, %18
  %42 = load i32, ptr %4, align 4
  ret i32 %42
}

declare hidden i32 @luaO_rawarith(ptr noundef, i32 noundef, ptr noundef, ptr noundef, ptr noundef) #4

declare hidden i32 @luaV_tointegerns(ptr noundef, ptr noundef, i32 noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @removevalues(ptr noundef %0, i32 noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  br label %5

5:                                                ; preds = %12, %2
  %6 = load i32, ptr %4, align 4
  %7 = icmp ne i32 %6, -1
  br i1 %7, label %8, label %16

8:                                                ; preds = %5
  %9 = load ptr, ptr %3, align 8
  %10 = load i32, ptr %4, align 4
  %11 = call i32 @patchtestreg(ptr noundef %9, i32 noundef %10, i32 noundef 255)
  br label %12

12:                                               ; preds = %8
  %13 = load ptr, ptr %3, align 8
  %14 = load i32, ptr %4, align 4
  %15 = call i32 @getjump(ptr noundef %13, i32 noundef %14)
  store i32 %15, ptr %4, align 4
  br label %5, !llvm.loop !13

16:                                               ; preds = %5
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaK_exp2K(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = getelementptr inbounds %struct.expdesc, ptr %7, i32 0, i32 2
  %9 = load i32, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.expdesc, ptr %10, i32 0, i32 3
  %12 = load i32, ptr %11, align 4
  %13 = icmp ne i32 %9, %12
  br i1 %13, label %60, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.expdesc, ptr %15, i32 0, i32 0
  %17 = load i32, ptr %16, align 8
  switch i32 %17, label %49 [
    i32 2, label %18
    i32 3, label %21
    i32 1, label %24
    i32 6, label %27
    i32 5, label %33
    i32 7, label %39
    i32 4, label %45
  ]

18:                                               ; preds = %14
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @boolT(ptr noundef %19)
  store i32 %20, ptr %6, align 4
  br label %50

21:                                               ; preds = %14
  %22 = load ptr, ptr %4, align 8
  %23 = call i32 @boolF(ptr noundef %22)
  store i32 %23, ptr %6, align 4
  br label %50

24:                                               ; preds = %14
  %25 = load ptr, ptr %4, align 8
  %26 = call i32 @nilK(ptr noundef %25)
  store i32 %26, ptr %6, align 4
  br label %50

27:                                               ; preds = %14
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.expdesc, ptr %29, i32 0, i32 1
  %31 = load i64, ptr %30, align 8
  %32 = call i32 @luaK_intK(ptr noundef %28, i64 noundef %31)
  store i32 %32, ptr %6, align 4
  br label %50

33:                                               ; preds = %14
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 1
  %37 = load double, ptr %36, align 8
  %38 = call i32 @luaK_numberK(ptr noundef %34, double noundef %37)
  store i32 %38, ptr %6, align 4
  br label %50

39:                                               ; preds = %14
  %40 = load ptr, ptr %4, align 8
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.expdesc, ptr %41, i32 0, i32 1
  %43 = load ptr, ptr %42, align 8
  %44 = call i32 @stringK(ptr noundef %40, ptr noundef %43)
  store i32 %44, ptr %6, align 4
  br label %50

45:                                               ; preds = %14
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.expdesc, ptr %46, i32 0, i32 1
  %48 = load i32, ptr %47, align 8
  store i32 %48, ptr %6, align 4
  br label %50

49:                                               ; preds = %14
  store i32 0, ptr %3, align 4
  br label %61

50:                                               ; preds = %45, %39, %33, %27, %24, %21, %18
  %51 = load i32, ptr %6, align 4
  %52 = icmp sle i32 %51, 255
  br i1 %52, label %53, label %59

53:                                               ; preds = %50
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %struct.expdesc, ptr %54, i32 0, i32 0
  store i32 4, ptr %55, align 8
  %56 = load i32, ptr %6, align 4
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.expdesc, ptr %57, i32 0, i32 1
  store i32 %56, ptr %58, align 8
  store i32 1, ptr %3, align 4
  br label %61

59:                                               ; preds = %50
  br label %60

60:                                               ; preds = %59, %2
  store i32 0, ptr %3, align 4
  br label %61

61:                                               ; preds = %60, %53, %49
  %62 = load i32, ptr %3, align 4
  ret i32 %62
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @boolT(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.TValue, align 8
  store ptr %0, ptr %2, align 8
  %4 = getelementptr inbounds %struct.TValue, ptr %3, i32 0, i32 1
  store i8 17, ptr %4, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @addk(ptr noundef %5, ptr noundef %3, ptr noundef %3)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @boolF(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.TValue, align 8
  store ptr %0, ptr %2, align 8
  %4 = getelementptr inbounds %struct.TValue, ptr %3, i32 0, i32 1
  store i8 1, ptr %4, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @addk(ptr noundef %5, ptr noundef %3, ptr noundef %3)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @nilK(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.TValue, align 8
  %4 = alloca %struct.TValue, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = getelementptr inbounds %struct.TValue, ptr %4, i32 0, i32 1
  store i8 0, ptr %7, align 8
  store ptr %3, ptr %5, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.FuncState, ptr %8, i32 0, i32 2
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 9
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  store i8 69, ptr %17, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.FuncState, ptr %18, i32 0, i32 2
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 6
  %22 = load ptr, ptr %21, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = call i32 @addk(ptr noundef %23, ptr noundef %3, ptr noundef %4)
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @fitsC(i64 noundef %0) #2 {
  %2 = alloca i64, align 8
  store i64 %0, ptr %2, align 8
  %3 = load i64, ptr %2, align 8
  %4 = add i64 %3, 127
  %5 = icmp ule i64 %4, 255
  %6 = zext i1 %5 to i32
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @finishbinexpval(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, i32 noundef %4, i32 noundef %5, i32 noundef %6, i32 noundef %7, i32 noundef %8) #2 {
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  %16 = alloca i32, align 4
  %17 = alloca i32, align 4
  %18 = alloca i32, align 4
  %19 = alloca i32, align 4
  %20 = alloca i32, align 4
  store ptr %0, ptr %10, align 8
  store ptr %1, ptr %11, align 8
  store ptr %2, ptr %12, align 8
  store i32 %3, ptr %13, align 4
  store i32 %4, ptr %14, align 4
  store i32 %5, ptr %15, align 4
  store i32 %6, ptr %16, align 4
  store i32 %7, ptr %17, align 4
  store i32 %8, ptr %18, align 4
  %21 = load ptr, ptr %10, align 8
  %22 = load ptr, ptr %11, align 8
  %23 = call i32 @luaK_exp2anyreg(ptr noundef %21, ptr noundef %22)
  store i32 %23, ptr %19, align 4
  %24 = load ptr, ptr %10, align 8
  %25 = load i32, ptr %13, align 4
  %26 = load i32, ptr %19, align 4
  %27 = load i32, ptr %14, align 4
  %28 = call i32 @luaK_codeABCk(ptr noundef %24, i32 noundef %25, i32 noundef 0, i32 noundef %26, i32 noundef %27, i32 noundef 0)
  store i32 %28, ptr %20, align 4
  %29 = load ptr, ptr %10, align 8
  %30 = load ptr, ptr %11, align 8
  %31 = load ptr, ptr %12, align 8
  call void @freeexps(ptr noundef %29, ptr noundef %30, ptr noundef %31)
  %32 = load i32, ptr %20, align 4
  %33 = load ptr, ptr %11, align 8
  %34 = getelementptr inbounds %struct.expdesc, ptr %33, i32 0, i32 1
  store i32 %32, ptr %34, align 8
  %35 = load ptr, ptr %11, align 8
  %36 = getelementptr inbounds %struct.expdesc, ptr %35, i32 0, i32 0
  store i32 17, ptr %36, align 8
  %37 = load ptr, ptr %10, align 8
  %38 = load i32, ptr %16, align 4
  call void @luaK_fixline(ptr noundef %37, i32 noundef %38)
  %39 = load ptr, ptr %10, align 8
  %40 = load i32, ptr %17, align 4
  %41 = load i32, ptr %19, align 4
  %42 = load i32, ptr %14, align 4
  %43 = load i32, ptr %18, align 4
  %44 = load i32, ptr %15, align 4
  %45 = call i32 @luaK_codeABCk(ptr noundef %39, i32 noundef %40, i32 noundef %41, i32 noundef %42, i32 noundef %43, i32 noundef %44)
  %46 = load ptr, ptr %10, align 8
  %47 = load i32, ptr %16, align 4
  call void @luaK_fixline(ptr noundef %46, i32 noundef %47)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeexps(ptr noundef %0, ptr noundef %1, ptr noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.expdesc, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  %12 = icmp eq i32 %11, 8
  br i1 %12, label %13, label %17

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.expdesc, ptr %14, i32 0, i32 1
  %16 = load i32, ptr %15, align 8
  br label %18

17:                                               ; preds = %3
  br label %18

18:                                               ; preds = %17, %13
  %19 = phi i32 [ %16, %13 ], [ -1, %17 ]
  store i32 %19, ptr %7, align 4
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.expdesc, ptr %20, i32 0, i32 0
  %22 = load i32, ptr %21, align 8
  %23 = icmp eq i32 %22, 8
  br i1 %23, label %24, label %28

24:                                               ; preds = %18
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.expdesc, ptr %25, i32 0, i32 1
  %27 = load i32, ptr %26, align 8
  br label %29

28:                                               ; preds = %18
  br label %29

29:                                               ; preds = %28, %24
  %30 = phi i32 [ %27, %24 ], [ -1, %28 ]
  store i32 %30, ptr %8, align 4
  %31 = load ptr, ptr %4, align 8
  %32 = load i32, ptr %7, align 4
  %33 = load i32, ptr %8, align 4
  call void @freeregs(ptr noundef %31, i32 noundef %32, i32 noundef %33)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codebinK(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %16 = load i32, ptr %8, align 4
  %17 = call i32 @binopr2TM(i32 noundef %16)
  store i32 %17, ptr %13, align 4
  %18 = load ptr, ptr %10, align 8
  %19 = getelementptr inbounds %struct.expdesc, ptr %18, i32 0, i32 1
  %20 = load i32, ptr %19, align 8
  store i32 %20, ptr %14, align 4
  %21 = load i32, ptr %8, align 4
  %22 = call i32 @binopr2op(i32 noundef %21, i32 noundef 0, i32 noundef 22)
  store i32 %22, ptr %15, align 4
  %23 = load ptr, ptr %7, align 8
  %24 = load ptr, ptr %9, align 8
  %25 = load ptr, ptr %10, align 8
  %26 = load i32, ptr %15, align 4
  %27 = load i32, ptr %14, align 4
  %28 = load i32, ptr %11, align 4
  %29 = load i32, ptr %12, align 4
  %30 = load i32, ptr %13, align 4
  call void @finishbinexpval(ptr noundef %23, ptr noundef %24, ptr noundef %25, i32 noundef %26, i32 noundef %27, i32 noundef %28, i32 noundef %29, i32 noundef 48, i32 noundef %30)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @codebinNoK(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4, i32 noundef %5) #2 {
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i32 %4, ptr %11, align 4
  store i32 %5, ptr %12, align 4
  %13 = load i32, ptr %11, align 4
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %6
  %16 = load ptr, ptr %9, align 8
  %17 = load ptr, ptr %10, align 8
  call void @swapexps(ptr noundef %16, ptr noundef %17)
  br label %18

18:                                               ; preds = %15, %6
  %19 = load ptr, ptr %7, align 8
  %20 = load i32, ptr %8, align 4
  %21 = load ptr, ptr %9, align 8
  %22 = load ptr, ptr %10, align 8
  %23 = load i32, ptr %12, align 4
  call void @codebinexpval(ptr noundef %19, i32 noundef %20, ptr noundef %21, ptr noundef %22, i32 noundef %23)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @binopr2TM(i32 noundef %0) #2 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  %3 = load i32, ptr %2, align 4
  %4 = sub nsw i32 %3, 0
  %5 = add nsw i32 %4, 6
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @binopr2op(i32 noundef %0, i32 noundef %1, i32 noundef %2) #2 {
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load i32, ptr %4, align 4
  %8 = load i32, ptr %5, align 4
  %9 = sub nsw i32 %7, %8
  %10 = load i32, ptr %6, align 4
  %11 = add nsw i32 %9, %10
  ret i32 %11
}

attributes #0 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #6 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #7 = { noreturn }
attributes #8 = { nounwind }

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
