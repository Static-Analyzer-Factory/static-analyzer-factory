; ModuleID = 'lundump.c'
source_filename = "lundump.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.LoadState = type { ptr, ptr, ptr }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.2, i16, i16 }
%union.anon = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Zio = type { i64, ptr, ptr, ptr, ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%union.StackValue = type { %struct.TValue }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }
%struct.AbsLineInfo = type { i32, i32 }
%struct.LocVar = type { ptr, i32, i32 }

@.str = private unnamed_addr constant [5 x i8] c"\1BLua\00", align 1
@.str.1 = private unnamed_addr constant [14 x i8] c"binary string\00", align 1
@.str.2 = private unnamed_addr constant [19 x i8] c"not a binary chunk\00", align 1
@.str.3 = private unnamed_addr constant [17 x i8] c"version mismatch\00", align 1
@.str.4 = private unnamed_addr constant [16 x i8] c"format mismatch\00", align 1
@.str.5 = private unnamed_addr constant [7 x i8] c"\19\93\0D\0A\1A\0A\00", align 1
@.str.6 = private unnamed_addr constant [16 x i8] c"corrupted chunk\00", align 1
@.str.7 = private unnamed_addr constant [12 x i8] c"Instruction\00", align 1
@.str.8 = private unnamed_addr constant [12 x i8] c"lua_Integer\00", align 1
@.str.9 = private unnamed_addr constant [11 x i8] c"lua_Number\00", align 1
@.str.10 = private unnamed_addr constant [24 x i8] c"integer format mismatch\00", align 1
@.str.11 = private unnamed_addr constant [22 x i8] c"float format mismatch\00", align 1
@.str.12 = private unnamed_addr constant [16 x i8] c"truncated chunk\00", align 1
@.str.13 = private unnamed_addr constant [27 x i8] c"%s: bad binary format (%s)\00", align 1
@.str.14 = private unnamed_addr constant [17 x i8] c"%s size mismatch\00", align 1
@.str.15 = private unnamed_addr constant [17 x i8] c"integer overflow\00", align 1
@.str.16 = private unnamed_addr constant [31 x i8] c"bad format for constant string\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaU_undump(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.LoadState, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = load i8, ptr %11, align 1
  %13 = sext i8 %12 to i32
  %14 = icmp eq i32 %13, 64
  br i1 %14, label %20, label %15

15:                                               ; preds = %3
  %16 = load ptr, ptr %6, align 8
  %17 = load i8, ptr %16, align 1
  %18 = sext i8 %17 to i32
  %19 = icmp eq i32 %18, 61
  br i1 %19, label %20, label %24

20:                                               ; preds = %15, %3
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 1
  %23 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 2
  store ptr %22, ptr %23, align 8
  br label %37

24:                                               ; preds = %15
  %25 = load ptr, ptr %6, align 8
  %26 = load i8, ptr %25, align 1
  %27 = sext i8 %26 to i32
  %28 = load i8, ptr @.str, align 1
  %29 = sext i8 %28 to i32
  %30 = icmp eq i32 %27, %29
  br i1 %30, label %31, label %33

31:                                               ; preds = %24
  %32 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 2
  store ptr @.str.1, ptr %32, align 8
  br label %36

33:                                               ; preds = %24
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 2
  store ptr %34, ptr %35, align 8
  br label %36

36:                                               ; preds = %33, %31
  br label %37

37:                                               ; preds = %36, %20
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 0
  store ptr %38, ptr %39, align 8
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 1
  store ptr %40, ptr %41, align 8
  call void @checkHeader(ptr noundef %7)
  %42 = load ptr, ptr %4, align 8
  %43 = call zeroext i8 @loadByte(ptr noundef %7)
  %44 = zext i8 %43 to i32
  %45 = call ptr @luaF_newLclosure(ptr noundef %42, i32 noundef %44)
  store ptr %45, ptr %8, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.lua_State, ptr %46, i32 0, i32 6
  %48 = load ptr, ptr %47, align 8
  store ptr %48, ptr %9, align 8
  %49 = load ptr, ptr %8, align 8
  store ptr %49, ptr %10, align 8
  %50 = load ptr, ptr %10, align 8
  %51 = load ptr, ptr %9, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  store ptr %50, ptr %52, align 8
  %53 = load ptr, ptr %9, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 1
  store i8 70, ptr %54, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = load ptr, ptr %4, align 8
  call void @luaD_inctop(ptr noundef %56)
  %57 = load ptr, ptr %4, align 8
  %58 = call ptr @luaF_newproto(ptr noundef %57)
  %59 = load ptr, ptr %8, align 8
  %60 = getelementptr inbounds %struct.LClosure, ptr %59, i32 0, i32 5
  store ptr %58, ptr %60, align 8
  %61 = load ptr, ptr %8, align 8
  %62 = getelementptr inbounds %struct.LClosure, ptr %61, i32 0, i32 2
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = and i32 %64, 32
  %66 = icmp ne i32 %65, 0
  br i1 %66, label %67, label %82

67:                                               ; preds = %37
  %68 = load ptr, ptr %8, align 8
  %69 = getelementptr inbounds %struct.LClosure, ptr %68, i32 0, i32 5
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.Proto, ptr %70, i32 0, i32 2
  %72 = load i8, ptr %71, align 1
  %73 = zext i8 %72 to i32
  %74 = and i32 %73, 24
  %75 = icmp ne i32 %74, 0
  br i1 %75, label %76, label %82

76:                                               ; preds = %67
  %77 = load ptr, ptr %4, align 8
  %78 = load ptr, ptr %8, align 8
  %79 = load ptr, ptr %8, align 8
  %80 = getelementptr inbounds %struct.LClosure, ptr %79, i32 0, i32 5
  %81 = load ptr, ptr %80, align 8
  call void @luaC_barrier_(ptr noundef %77, ptr noundef %78, ptr noundef %81)
  br label %83

82:                                               ; preds = %67, %37
  br label %83

83:                                               ; preds = %82, %76
  %84 = load ptr, ptr %8, align 8
  %85 = getelementptr inbounds %struct.LClosure, ptr %84, i32 0, i32 5
  %86 = load ptr, ptr %85, align 8
  call void @loadFunction(ptr noundef %7, ptr noundef %86, ptr noundef null)
  %87 = load ptr, ptr %8, align 8
  ret ptr %87
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkHeader(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @checkliteral(ptr noundef %3, ptr noundef getelementptr inbounds ([5 x i8], ptr @.str, i64 0, i64 1), ptr noundef @.str.2)
  %4 = load ptr, ptr %2, align 8
  %5 = call zeroext i8 @loadByte(ptr noundef %4)
  %6 = zext i8 %5 to i32
  %7 = icmp ne i32 %6, 84
  br i1 %7, label %8, label %10

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  call void @error(ptr noundef %9, ptr noundef @.str.3) #5
  unreachable

10:                                               ; preds = %1
  %11 = load ptr, ptr %2, align 8
  %12 = call zeroext i8 @loadByte(ptr noundef %11)
  %13 = zext i8 %12 to i32
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %15, label %17

15:                                               ; preds = %10
  %16 = load ptr, ptr %2, align 8
  call void @error(ptr noundef %16, ptr noundef @.str.4) #5
  unreachable

17:                                               ; preds = %10
  %18 = load ptr, ptr %2, align 8
  call void @checkliteral(ptr noundef %18, ptr noundef @.str.5, ptr noundef @.str.6)
  %19 = load ptr, ptr %2, align 8
  call void @fchecksize(ptr noundef %19, i64 noundef 4, ptr noundef @.str.7)
  %20 = load ptr, ptr %2, align 8
  call void @fchecksize(ptr noundef %20, i64 noundef 8, ptr noundef @.str.8)
  %21 = load ptr, ptr %2, align 8
  call void @fchecksize(ptr noundef %21, i64 noundef 8, ptr noundef @.str.9)
  %22 = load ptr, ptr %2, align 8
  %23 = call i64 @loadInteger(ptr noundef %22)
  %24 = icmp ne i64 %23, 22136
  br i1 %24, label %25, label %27

25:                                               ; preds = %17
  %26 = load ptr, ptr %2, align 8
  call void @error(ptr noundef %26, ptr noundef @.str.10) #5
  unreachable

27:                                               ; preds = %17
  %28 = load ptr, ptr %2, align 8
  %29 = call double @loadNumber(ptr noundef %28)
  %30 = fcmp une double %29, 3.705000e+02
  br i1 %30, label %31, label %33

31:                                               ; preds = %27
  %32 = load ptr, ptr %2, align 8
  call void @error(ptr noundef %32, ptr noundef @.str.11) #5
  unreachable

33:                                               ; preds = %27
  ret void
}

declare hidden ptr @luaF_newLclosure(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal zeroext i8 @loadByte(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LoadState, ptr %4, i32 0, i32 1
  %6 = load ptr, ptr %5, align 8
  %7 = getelementptr inbounds %struct.Zio, ptr %6, i32 0, i32 0
  %8 = load i64, ptr %7, align 8
  %9 = add i64 %8, -1
  store i64 %9, ptr %7, align 8
  %10 = icmp ugt i64 %8, 0
  br i1 %10, label %11, label %20

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.LoadState, ptr %12, i32 0, i32 1
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Zio, ptr %14, i32 0, i32 1
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds i8, ptr %16, i32 1
  store ptr %17, ptr %15, align 8
  %18 = load i8, ptr %16, align 1
  %19 = zext i8 %18 to i32
  br label %25

20:                                               ; preds = %1
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.LoadState, ptr %21, i32 0, i32 1
  %23 = load ptr, ptr %22, align 8
  %24 = call i32 @luaZ_fill(ptr noundef %23)
  br label %25

25:                                               ; preds = %20, %11
  %26 = phi i32 [ %19, %11 ], [ %24, %20 ]
  store i32 %26, ptr %3, align 4
  %27 = load i32, ptr %3, align 4
  %28 = icmp eq i32 %27, -1
  br i1 %28, label %29, label %31

29:                                               ; preds = %25
  %30 = load ptr, ptr %2, align 8
  call void @error(ptr noundef %30, ptr noundef @.str.12) #5
  unreachable

31:                                               ; preds = %25
  %32 = load i32, ptr %3, align 4
  %33 = trunc i32 %32 to i8
  ret i8 %33
}

declare hidden void @luaD_inctop(ptr noundef) #1

declare hidden ptr @luaF_newproto(ptr noundef) #1

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadFunction(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = call ptr @loadStringN(ptr noundef %7, ptr noundef %8)
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.Proto, ptr %10, i32 0, i32 22
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 22
  %14 = load ptr, ptr %13, align 8
  %15 = icmp eq ptr %14, null
  br i1 %15, label %16, label %20

16:                                               ; preds = %3
  %17 = load ptr, ptr %6, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 22
  store ptr %17, ptr %19, align 8
  br label %20

20:                                               ; preds = %16, %3
  %21 = load ptr, ptr %4, align 8
  %22 = call i32 @loadInt(ptr noundef %21)
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Proto, ptr %23, i32 0, i32 13
  store i32 %22, ptr %24, align 4
  %25 = load ptr, ptr %4, align 8
  %26 = call i32 @loadInt(ptr noundef %25)
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.Proto, ptr %27, i32 0, i32 14
  store i32 %26, ptr %28, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = call zeroext i8 @loadByte(ptr noundef %29)
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.Proto, ptr %31, i32 0, i32 3
  store i8 %30, ptr %32, align 2
  %33 = load ptr, ptr %4, align 8
  %34 = call zeroext i8 @loadByte(ptr noundef %33)
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.Proto, ptr %35, i32 0, i32 4
  store i8 %34, ptr %36, align 1
  %37 = load ptr, ptr %4, align 8
  %38 = call zeroext i8 @loadByte(ptr noundef %37)
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.Proto, ptr %39, i32 0, i32 5
  store i8 %38, ptr %40, align 4
  %41 = load ptr, ptr %4, align 8
  %42 = load ptr, ptr %5, align 8
  call void @loadCode(ptr noundef %41, ptr noundef %42)
  %43 = load ptr, ptr %4, align 8
  %44 = load ptr, ptr %5, align 8
  call void @loadConstants(ptr noundef %43, ptr noundef %44)
  %45 = load ptr, ptr %4, align 8
  %46 = load ptr, ptr %5, align 8
  call void @loadUpvalues(ptr noundef %45, ptr noundef %46)
  %47 = load ptr, ptr %4, align 8
  %48 = load ptr, ptr %5, align 8
  call void @loadProtos(ptr noundef %47, ptr noundef %48)
  %49 = load ptr, ptr %4, align 8
  %50 = load ptr, ptr %5, align 8
  call void @loadDebug(ptr noundef %49, ptr noundef %50)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkliteral(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca [12 x i8], align 1
  %8 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = call i64 @strlen(ptr noundef %9) #6
  store i64 %10, ptr %8, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds [12 x i8], ptr %7, i64 0, i64 0
  %13 = load i64, ptr %8, align 8
  %14 = mul i64 %13, 1
  call void @loadBlock(ptr noundef %11, ptr noundef %12, i64 noundef %14)
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds [12 x i8], ptr %7, i64 0, i64 0
  %17 = load i64, ptr %8, align 8
  %18 = call i32 @memcmp(ptr noundef %15, ptr noundef %16, i64 noundef %17) #6
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %3
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %6, align 8
  call void @error(ptr noundef %21, ptr noundef %22) #5
  unreachable

23:                                               ; preds = %3
  ret void
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @error(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.LoadState, ptr %5, i32 0, i32 0
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LoadState, ptr %8, i32 0, i32 2
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %7, ptr noundef @.str.13, ptr noundef %10, ptr noundef %11)
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.LoadState, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  call void @luaD_throw(ptr noundef %15, i32 noundef 3) #5
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fchecksize(ptr noundef %0, i64 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call zeroext i8 @loadByte(ptr noundef %7)
  %9 = zext i8 %8 to i64
  %10 = load i64, ptr %5, align 8
  %11 = icmp ne i64 %9, %10
  br i1 %11, label %12, label %19

12:                                               ; preds = %3
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.LoadState, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %6, align 8
  %18 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %16, ptr noundef @.str.14, ptr noundef %17)
  call void @error(ptr noundef %13, ptr noundef %18) #5
  unreachable

19:                                               ; preds = %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @loadInteger(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @loadBlock(ptr noundef %4, ptr noundef %3, i64 noundef 8)
  %5 = load i64, ptr %3, align 8
  ret i64 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define internal double @loadNumber(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @loadBlock(ptr noundef %4, ptr noundef %3, i64 noundef 8)
  %5 = load double, ptr %3, align 8
  ret double %5
}

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadBlock(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.LoadState, ptr %7, i32 0, i32 1
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load i64, ptr %6, align 8
  %12 = call i64 @luaZ_read(ptr noundef %9, ptr noundef %10, i64 noundef %11)
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %14, label %16

14:                                               ; preds = %3
  %15 = load ptr, ptr %4, align 8
  call void @error(ptr noundef %15, ptr noundef @.str.12) #5
  unreachable

16:                                               ; preds = %3
  ret void
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @memcmp(ptr noundef, ptr noundef, i64 noundef) #3

declare hidden i64 @luaZ_read(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden ptr @luaO_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #4

declare hidden i32 @luaZ_fill(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @loadStringN(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca [40 x i8], align 16
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LoadState, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = call i64 @loadSize(ptr noundef %15)
  store i64 %16, ptr %8, align 8
  %17 = load i64, ptr %8, align 8
  %18 = icmp eq i64 %17, 0
  br i1 %18, label %19, label %20

19:                                               ; preds = %2
  store ptr null, ptr %3, align 8
  br label %86

20:                                               ; preds = %2
  %21 = load i64, ptr %8, align 8
  %22 = add i64 %21, -1
  store i64 %22, ptr %8, align 8
  %23 = icmp ule i64 %22, 40
  br i1 %23, label %24, label %33

24:                                               ; preds = %20
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds [40 x i8], ptr %9, i64 0, i64 0
  %27 = load i64, ptr %8, align 8
  %28 = mul i64 %27, 1
  call void @loadBlock(ptr noundef %25, ptr noundef %26, i64 noundef %28)
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds [40 x i8], ptr %9, i64 0, i64 0
  %31 = load i64, ptr %8, align 8
  %32 = call ptr @luaS_newlstr(ptr noundef %29, ptr noundef %30, i64 noundef %31)
  store ptr %32, ptr %7, align 8
  br label %64

33:                                               ; preds = %20
  %34 = load ptr, ptr %6, align 8
  %35 = load i64, ptr %8, align 8
  %36 = call ptr @luaS_createlngstrobj(ptr noundef %34, i64 noundef %35)
  store ptr %36, ptr %7, align 8
  %37 = load ptr, ptr %6, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 6
  %39 = load ptr, ptr %38, align 8
  store ptr %39, ptr %10, align 8
  %40 = load ptr, ptr %7, align 8
  store ptr %40, ptr %11, align 8
  %41 = load ptr, ptr %11, align 8
  %42 = load ptr, ptr %10, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  store ptr %41, ptr %43, align 8
  %44 = load ptr, ptr %11, align 8
  %45 = getelementptr inbounds %struct.TString, ptr %44, i32 0, i32 1
  %46 = load i8, ptr %45, align 8
  %47 = zext i8 %46 to i32
  %48 = or i32 %47, 64
  %49 = trunc i32 %48 to i8
  %50 = load ptr, ptr %10, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  store i8 %49, ptr %51, align 8
  %52 = load ptr, ptr %6, align 8
  %53 = load ptr, ptr %6, align 8
  call void @luaD_inctop(ptr noundef %53)
  %54 = load ptr, ptr %4, align 8
  %55 = load ptr, ptr %7, align 8
  %56 = getelementptr inbounds %struct.TString, ptr %55, i32 0, i32 7
  %57 = getelementptr inbounds [1 x i8], ptr %56, i64 0, i64 0
  %58 = load i64, ptr %8, align 8
  %59 = mul i64 %58, 1
  call void @loadBlock(ptr noundef %54, ptr noundef %57, i64 noundef %59)
  %60 = load ptr, ptr %6, align 8
  %61 = getelementptr inbounds %struct.lua_State, ptr %60, i32 0, i32 6
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds %union.StackValue, ptr %62, i32 -1
  store ptr %63, ptr %61, align 8
  br label %64

64:                                               ; preds = %33, %24
  br label %65

65:                                               ; preds = %64
  %66 = load ptr, ptr %5, align 8
  %67 = getelementptr inbounds %struct.Proto, ptr %66, i32 0, i32 2
  %68 = load i8, ptr %67, align 1
  %69 = zext i8 %68 to i32
  %70 = and i32 %69, 32
  %71 = icmp ne i32 %70, 0
  br i1 %71, label %72, label %83

72:                                               ; preds = %65
  %73 = load ptr, ptr %7, align 8
  %74 = getelementptr inbounds %struct.TString, ptr %73, i32 0, i32 2
  %75 = load i8, ptr %74, align 1
  %76 = zext i8 %75 to i32
  %77 = and i32 %76, 24
  %78 = icmp ne i32 %77, 0
  br i1 %78, label %79, label %83

79:                                               ; preds = %72
  %80 = load ptr, ptr %6, align 8
  %81 = load ptr, ptr %5, align 8
  %82 = load ptr, ptr %7, align 8
  call void @luaC_barrier_(ptr noundef %80, ptr noundef %81, ptr noundef %82)
  br label %84

83:                                               ; preds = %72, %65
  br label %84

84:                                               ; preds = %83, %79
  %85 = load ptr, ptr %7, align 8
  store ptr %85, ptr %3, align 8
  br label %86

86:                                               ; preds = %84, %19
  %87 = load ptr, ptr %3, align 8
  ret ptr %87
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @loadInt(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i64 @loadUnsigned(ptr noundef %3, i64 noundef 2147483647)
  %5 = trunc i64 %4 to i32
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadCode(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @loadInt(ptr noundef %6)
  store i32 %7, ptr %5, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.LoadState, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = load i32, ptr %5, align 4
  %12 = sext i32 %11 to i64
  %13 = mul i64 %12, 4
  %14 = call ptr @luaM_malloc_(ptr noundef %10, i64 noundef %13, i32 noundef 0)
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.Proto, ptr %15, i32 0, i32 16
  store ptr %14, ptr %16, align 8
  %17 = load i32, ptr %5, align 4
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 8
  store i32 %17, ptr %19, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 16
  %23 = load ptr, ptr %22, align 8
  %24 = load i32, ptr %5, align 4
  %25 = sext i32 %24 to i64
  %26 = mul i64 %25, 4
  call void @loadBlock(ptr noundef %20, ptr noundef %23, i64 noundef %26)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadConstants(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = call i32 @loadInt(ptr noundef %13)
  store i32 %14, ptr %6, align 4
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.LoadState, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = load i32, ptr %6, align 4
  %19 = sext i32 %18 to i64
  %20 = mul i64 %19, 16
  %21 = call ptr @luaM_malloc_(ptr noundef %17, i64 noundef %20, i32 noundef 0)
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 15
  store ptr %21, ptr %23, align 8
  %24 = load i32, ptr %6, align 4
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 7
  store i32 %24, ptr %26, align 4
  store i32 0, ptr %5, align 4
  br label %27

27:                                               ; preds = %39, %2
  %28 = load i32, ptr %5, align 4
  %29 = load i32, ptr %6, align 4
  %30 = icmp slt i32 %28, %29
  br i1 %30, label %31, label %42

31:                                               ; preds = %27
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.Proto, ptr %32, i32 0, i32 15
  %34 = load ptr, ptr %33, align 8
  %35 = load i32, ptr %5, align 4
  %36 = sext i32 %35 to i64
  %37 = getelementptr inbounds %struct.TValue, ptr %34, i64 %36
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 1
  store i8 0, ptr %38, align 8
  br label %39

39:                                               ; preds = %31
  %40 = load i32, ptr %5, align 4
  %41 = add nsw i32 %40, 1
  store i32 %41, ptr %5, align 4
  br label %27, !llvm.loop !6

42:                                               ; preds = %27
  store i32 0, ptr %5, align 4
  br label %43

43:                                               ; preds = %104, %42
  %44 = load i32, ptr %5, align 4
  %45 = load i32, ptr %6, align 4
  %46 = icmp slt i32 %44, %45
  br i1 %46, label %47, label %107

47:                                               ; preds = %43
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.Proto, ptr %48, i32 0, i32 15
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %5, align 4
  %52 = sext i32 %51 to i64
  %53 = getelementptr inbounds %struct.TValue, ptr %50, i64 %52
  store ptr %53, ptr %7, align 8
  %54 = load ptr, ptr %3, align 8
  %55 = call zeroext i8 @loadByte(ptr noundef %54)
  %56 = zext i8 %55 to i32
  store i32 %56, ptr %8, align 4
  %57 = load i32, ptr %8, align 4
  switch i32 %57, label %102 [
    i32 0, label %58
    i32 1, label %61
    i32 17, label %64
    i32 19, label %67
    i32 3, label %75
    i32 4, label %83
    i32 20, label %83
  ]

58:                                               ; preds = %47
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 1
  store i8 0, ptr %60, align 8
  br label %103

61:                                               ; preds = %47
  %62 = load ptr, ptr %7, align 8
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 1
  store i8 1, ptr %63, align 8
  br label %103

64:                                               ; preds = %47
  %65 = load ptr, ptr %7, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 1
  store i8 17, ptr %66, align 8
  br label %103

67:                                               ; preds = %47
  %68 = load ptr, ptr %7, align 8
  store ptr %68, ptr %9, align 8
  %69 = load ptr, ptr %3, align 8
  %70 = call double @loadNumber(ptr noundef %69)
  %71 = load ptr, ptr %9, align 8
  %72 = getelementptr inbounds %struct.TValue, ptr %71, i32 0, i32 0
  store double %70, ptr %72, align 8
  %73 = load ptr, ptr %9, align 8
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 1
  store i8 19, ptr %74, align 8
  br label %103

75:                                               ; preds = %47
  %76 = load ptr, ptr %7, align 8
  store ptr %76, ptr %10, align 8
  %77 = load ptr, ptr %3, align 8
  %78 = call i64 @loadInteger(ptr noundef %77)
  %79 = load ptr, ptr %10, align 8
  %80 = getelementptr inbounds %struct.TValue, ptr %79, i32 0, i32 0
  store i64 %78, ptr %80, align 8
  %81 = load ptr, ptr %10, align 8
  %82 = getelementptr inbounds %struct.TValue, ptr %81, i32 0, i32 1
  store i8 3, ptr %82, align 8
  br label %103

83:                                               ; preds = %47, %47
  %84 = load ptr, ptr %7, align 8
  store ptr %84, ptr %11, align 8
  %85 = load ptr, ptr %3, align 8
  %86 = load ptr, ptr %4, align 8
  %87 = call ptr @loadString(ptr noundef %85, ptr noundef %86)
  store ptr %87, ptr %12, align 8
  %88 = load ptr, ptr %12, align 8
  %89 = load ptr, ptr %11, align 8
  %90 = getelementptr inbounds %struct.TValue, ptr %89, i32 0, i32 0
  store ptr %88, ptr %90, align 8
  %91 = load ptr, ptr %12, align 8
  %92 = getelementptr inbounds %struct.TString, ptr %91, i32 0, i32 1
  %93 = load i8, ptr %92, align 8
  %94 = zext i8 %93 to i32
  %95 = or i32 %94, 64
  %96 = trunc i32 %95 to i8
  %97 = load ptr, ptr %11, align 8
  %98 = getelementptr inbounds %struct.TValue, ptr %97, i32 0, i32 1
  store i8 %96, ptr %98, align 8
  %99 = load ptr, ptr %3, align 8
  %100 = getelementptr inbounds %struct.LoadState, ptr %99, i32 0, i32 0
  %101 = load ptr, ptr %100, align 8
  br label %103

102:                                              ; preds = %47
  br label %103

103:                                              ; preds = %102, %83, %75, %67, %64, %61, %58
  br label %104

104:                                              ; preds = %103
  %105 = load i32, ptr %5, align 4
  %106 = add nsw i32 %105, 1
  store i32 %106, ptr %5, align 4
  br label %43, !llvm.loop !8

107:                                              ; preds = %43
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadUpvalues(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @loadInt(ptr noundef %7)
  store i32 %8, ptr %6, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LoadState, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %6, align 4
  %13 = sext i32 %12 to i64
  %14 = mul i64 %13, 16
  %15 = call ptr @luaM_malloc_(ptr noundef %11, i64 noundef %14, i32 noundef 0)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 18
  store ptr %15, ptr %17, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.Proto, ptr %19, i32 0, i32 6
  store i32 %18, ptr %20, align 8
  store i32 0, ptr %5, align 4
  br label %21

21:                                               ; preds = %33, %2
  %22 = load i32, ptr %5, align 4
  %23 = load i32, ptr %6, align 4
  %24 = icmp slt i32 %22, %23
  br i1 %24, label %25, label %36

25:                                               ; preds = %21
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.Proto, ptr %26, i32 0, i32 18
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %5, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds %struct.Upvaldesc, ptr %28, i64 %30
  %32 = getelementptr inbounds %struct.Upvaldesc, ptr %31, i32 0, i32 0
  store ptr null, ptr %32, align 8
  br label %33

33:                                               ; preds = %25
  %34 = load i32, ptr %5, align 4
  %35 = add nsw i32 %34, 1
  store i32 %35, ptr %5, align 4
  br label %21, !llvm.loop !9

36:                                               ; preds = %21
  store i32 0, ptr %5, align 4
  br label %37

37:                                               ; preds = %69, %36
  %38 = load i32, ptr %5, align 4
  %39 = load i32, ptr %6, align 4
  %40 = icmp slt i32 %38, %39
  br i1 %40, label %41, label %72

41:                                               ; preds = %37
  %42 = load ptr, ptr %3, align 8
  %43 = call zeroext i8 @loadByte(ptr noundef %42)
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.Proto, ptr %44, i32 0, i32 18
  %46 = load ptr, ptr %45, align 8
  %47 = load i32, ptr %5, align 4
  %48 = sext i32 %47 to i64
  %49 = getelementptr inbounds %struct.Upvaldesc, ptr %46, i64 %48
  %50 = getelementptr inbounds %struct.Upvaldesc, ptr %49, i32 0, i32 1
  store i8 %43, ptr %50, align 8
  %51 = load ptr, ptr %3, align 8
  %52 = call zeroext i8 @loadByte(ptr noundef %51)
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.Proto, ptr %53, i32 0, i32 18
  %55 = load ptr, ptr %54, align 8
  %56 = load i32, ptr %5, align 4
  %57 = sext i32 %56 to i64
  %58 = getelementptr inbounds %struct.Upvaldesc, ptr %55, i64 %57
  %59 = getelementptr inbounds %struct.Upvaldesc, ptr %58, i32 0, i32 2
  store i8 %52, ptr %59, align 1
  %60 = load ptr, ptr %3, align 8
  %61 = call zeroext i8 @loadByte(ptr noundef %60)
  %62 = load ptr, ptr %4, align 8
  %63 = getelementptr inbounds %struct.Proto, ptr %62, i32 0, i32 18
  %64 = load ptr, ptr %63, align 8
  %65 = load i32, ptr %5, align 4
  %66 = sext i32 %65 to i64
  %67 = getelementptr inbounds %struct.Upvaldesc, ptr %64, i64 %66
  %68 = getelementptr inbounds %struct.Upvaldesc, ptr %67, i32 0, i32 3
  store i8 %61, ptr %68, align 2
  br label %69

69:                                               ; preds = %41
  %70 = load i32, ptr %5, align 4
  %71 = add nsw i32 %70, 1
  store i32 %71, ptr %5, align 4
  br label %37, !llvm.loop !10

72:                                               ; preds = %37
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadProtos(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @loadInt(ptr noundef %7)
  store i32 %8, ptr %6, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LoadState, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %6, align 4
  %13 = sext i32 %12 to i64
  %14 = mul i64 %13, 8
  %15 = call ptr @luaM_malloc_(ptr noundef %11, i64 noundef %14, i32 noundef 0)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 17
  store ptr %15, ptr %17, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.Proto, ptr %19, i32 0, i32 10
  store i32 %18, ptr %20, align 8
  store i32 0, ptr %5, align 4
  br label %21

21:                                               ; preds = %32, %2
  %22 = load i32, ptr %5, align 4
  %23 = load i32, ptr %6, align 4
  %24 = icmp slt i32 %22, %23
  br i1 %24, label %25, label %35

25:                                               ; preds = %21
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.Proto, ptr %26, i32 0, i32 17
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %5, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds ptr, ptr %28, i64 %30
  store ptr null, ptr %31, align 8
  br label %32

32:                                               ; preds = %25
  %33 = load i32, ptr %5, align 4
  %34 = add nsw i32 %33, 1
  store i32 %34, ptr %5, align 4
  br label %21, !llvm.loop !11

35:                                               ; preds = %21
  store i32 0, ptr %5, align 4
  br label %36

36:                                               ; preds = %95, %35
  %37 = load i32, ptr %5, align 4
  %38 = load i32, ptr %6, align 4
  %39 = icmp slt i32 %37, %38
  br i1 %39, label %40, label %98

40:                                               ; preds = %36
  %41 = load ptr, ptr %3, align 8
  %42 = getelementptr inbounds %struct.LoadState, ptr %41, i32 0, i32 0
  %43 = load ptr, ptr %42, align 8
  %44 = call ptr @luaF_newproto(ptr noundef %43)
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.Proto, ptr %45, i32 0, i32 17
  %47 = load ptr, ptr %46, align 8
  %48 = load i32, ptr %5, align 4
  %49 = sext i32 %48 to i64
  %50 = getelementptr inbounds ptr, ptr %47, i64 %49
  store ptr %44, ptr %50, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.Proto, ptr %51, i32 0, i32 2
  %53 = load i8, ptr %52, align 1
  %54 = zext i8 %53 to i32
  %55 = and i32 %54, 32
  %56 = icmp ne i32 %55, 0
  br i1 %56, label %57, label %82

57:                                               ; preds = %40
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 17
  %60 = load ptr, ptr %59, align 8
  %61 = load i32, ptr %5, align 4
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds ptr, ptr %60, i64 %62
  %64 = load ptr, ptr %63, align 8
  %65 = getelementptr inbounds %struct.Proto, ptr %64, i32 0, i32 2
  %66 = load i8, ptr %65, align 1
  %67 = zext i8 %66 to i32
  %68 = and i32 %67, 24
  %69 = icmp ne i32 %68, 0
  br i1 %69, label %70, label %82

70:                                               ; preds = %57
  %71 = load ptr, ptr %3, align 8
  %72 = getelementptr inbounds %struct.LoadState, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = load ptr, ptr %4, align 8
  %75 = load ptr, ptr %4, align 8
  %76 = getelementptr inbounds %struct.Proto, ptr %75, i32 0, i32 17
  %77 = load ptr, ptr %76, align 8
  %78 = load i32, ptr %5, align 4
  %79 = sext i32 %78 to i64
  %80 = getelementptr inbounds ptr, ptr %77, i64 %79
  %81 = load ptr, ptr %80, align 8
  call void @luaC_barrier_(ptr noundef %73, ptr noundef %74, ptr noundef %81)
  br label %83

82:                                               ; preds = %57, %40
  br label %83

83:                                               ; preds = %82, %70
  %84 = load ptr, ptr %3, align 8
  %85 = load ptr, ptr %4, align 8
  %86 = getelementptr inbounds %struct.Proto, ptr %85, i32 0, i32 17
  %87 = load ptr, ptr %86, align 8
  %88 = load i32, ptr %5, align 4
  %89 = sext i32 %88 to i64
  %90 = getelementptr inbounds ptr, ptr %87, i64 %89
  %91 = load ptr, ptr %90, align 8
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.Proto, ptr %92, i32 0, i32 22
  %94 = load ptr, ptr %93, align 8
  call void @loadFunction(ptr noundef %84, ptr noundef %91, ptr noundef %94)
  br label %95

95:                                               ; preds = %83
  %96 = load i32, ptr %5, align 4
  %97 = add nsw i32 %96, 1
  store i32 %97, ptr %5, align 4
  br label %36, !llvm.loop !12

98:                                               ; preds = %36
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @loadDebug(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @loadInt(ptr noundef %7)
  store i32 %8, ptr %6, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.LoadState, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %6, align 4
  %13 = sext i32 %12 to i64
  %14 = mul i64 %13, 1
  %15 = call ptr @luaM_malloc_(ptr noundef %11, i64 noundef %14, i32 noundef 0)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 19
  store ptr %15, ptr %17, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.Proto, ptr %19, i32 0, i32 9
  store i32 %18, ptr %20, align 4
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 19
  %24 = load ptr, ptr %23, align 8
  %25 = load i32, ptr %6, align 4
  %26 = sext i32 %25 to i64
  %27 = mul i64 %26, 1
  call void @loadBlock(ptr noundef %21, ptr noundef %24, i64 noundef %27)
  %28 = load ptr, ptr %3, align 8
  %29 = call i32 @loadInt(ptr noundef %28)
  store i32 %29, ptr %6, align 4
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.LoadState, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = load i32, ptr %6, align 4
  %34 = sext i32 %33 to i64
  %35 = mul i64 %34, 8
  %36 = call ptr @luaM_malloc_(ptr noundef %32, i64 noundef %35, i32 noundef 0)
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.Proto, ptr %37, i32 0, i32 20
  store ptr %36, ptr %38, align 8
  %39 = load i32, ptr %6, align 4
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.Proto, ptr %40, i32 0, i32 12
  store i32 %39, ptr %41, align 8
  store i32 0, ptr %5, align 4
  br label %42

42:                                               ; preds = %65, %2
  %43 = load i32, ptr %5, align 4
  %44 = load i32, ptr %6, align 4
  %45 = icmp slt i32 %43, %44
  br i1 %45, label %46, label %68

46:                                               ; preds = %42
  %47 = load ptr, ptr %3, align 8
  %48 = call i32 @loadInt(ptr noundef %47)
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.Proto, ptr %49, i32 0, i32 20
  %51 = load ptr, ptr %50, align 8
  %52 = load i32, ptr %5, align 4
  %53 = sext i32 %52 to i64
  %54 = getelementptr inbounds %struct.AbsLineInfo, ptr %51, i64 %53
  %55 = getelementptr inbounds %struct.AbsLineInfo, ptr %54, i32 0, i32 0
  store i32 %48, ptr %55, align 4
  %56 = load ptr, ptr %3, align 8
  %57 = call i32 @loadInt(ptr noundef %56)
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 20
  %60 = load ptr, ptr %59, align 8
  %61 = load i32, ptr %5, align 4
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds %struct.AbsLineInfo, ptr %60, i64 %62
  %64 = getelementptr inbounds %struct.AbsLineInfo, ptr %63, i32 0, i32 1
  store i32 %57, ptr %64, align 4
  br label %65

65:                                               ; preds = %46
  %66 = load i32, ptr %5, align 4
  %67 = add nsw i32 %66, 1
  store i32 %67, ptr %5, align 4
  br label %42, !llvm.loop !13

68:                                               ; preds = %42
  %69 = load ptr, ptr %3, align 8
  %70 = call i32 @loadInt(ptr noundef %69)
  store i32 %70, ptr %6, align 4
  %71 = load ptr, ptr %3, align 8
  %72 = getelementptr inbounds %struct.LoadState, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = load i32, ptr %6, align 4
  %75 = sext i32 %74 to i64
  %76 = mul i64 %75, 16
  %77 = call ptr @luaM_malloc_(ptr noundef %73, i64 noundef %76, i32 noundef 0)
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 21
  store ptr %77, ptr %79, align 8
  %80 = load i32, ptr %6, align 4
  %81 = load ptr, ptr %4, align 8
  %82 = getelementptr inbounds %struct.Proto, ptr %81, i32 0, i32 11
  store i32 %80, ptr %82, align 4
  store i32 0, ptr %5, align 4
  br label %83

83:                                               ; preds = %95, %68
  %84 = load i32, ptr %5, align 4
  %85 = load i32, ptr %6, align 4
  %86 = icmp slt i32 %84, %85
  br i1 %86, label %87, label %98

87:                                               ; preds = %83
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds %struct.Proto, ptr %88, i32 0, i32 21
  %90 = load ptr, ptr %89, align 8
  %91 = load i32, ptr %5, align 4
  %92 = sext i32 %91 to i64
  %93 = getelementptr inbounds %struct.LocVar, ptr %90, i64 %92
  %94 = getelementptr inbounds %struct.LocVar, ptr %93, i32 0, i32 0
  store ptr null, ptr %94, align 8
  br label %95

95:                                               ; preds = %87
  %96 = load i32, ptr %5, align 4
  %97 = add nsw i32 %96, 1
  store i32 %97, ptr %5, align 4
  br label %83, !llvm.loop !14

98:                                               ; preds = %83
  store i32 0, ptr %5, align 4
  br label %99

99:                                               ; preds = %132, %98
  %100 = load i32, ptr %5, align 4
  %101 = load i32, ptr %6, align 4
  %102 = icmp slt i32 %100, %101
  br i1 %102, label %103, label %135

103:                                              ; preds = %99
  %104 = load ptr, ptr %3, align 8
  %105 = load ptr, ptr %4, align 8
  %106 = call ptr @loadStringN(ptr noundef %104, ptr noundef %105)
  %107 = load ptr, ptr %4, align 8
  %108 = getelementptr inbounds %struct.Proto, ptr %107, i32 0, i32 21
  %109 = load ptr, ptr %108, align 8
  %110 = load i32, ptr %5, align 4
  %111 = sext i32 %110 to i64
  %112 = getelementptr inbounds %struct.LocVar, ptr %109, i64 %111
  %113 = getelementptr inbounds %struct.LocVar, ptr %112, i32 0, i32 0
  store ptr %106, ptr %113, align 8
  %114 = load ptr, ptr %3, align 8
  %115 = call i32 @loadInt(ptr noundef %114)
  %116 = load ptr, ptr %4, align 8
  %117 = getelementptr inbounds %struct.Proto, ptr %116, i32 0, i32 21
  %118 = load ptr, ptr %117, align 8
  %119 = load i32, ptr %5, align 4
  %120 = sext i32 %119 to i64
  %121 = getelementptr inbounds %struct.LocVar, ptr %118, i64 %120
  %122 = getelementptr inbounds %struct.LocVar, ptr %121, i32 0, i32 1
  store i32 %115, ptr %122, align 8
  %123 = load ptr, ptr %3, align 8
  %124 = call i32 @loadInt(ptr noundef %123)
  %125 = load ptr, ptr %4, align 8
  %126 = getelementptr inbounds %struct.Proto, ptr %125, i32 0, i32 21
  %127 = load ptr, ptr %126, align 8
  %128 = load i32, ptr %5, align 4
  %129 = sext i32 %128 to i64
  %130 = getelementptr inbounds %struct.LocVar, ptr %127, i64 %129
  %131 = getelementptr inbounds %struct.LocVar, ptr %130, i32 0, i32 2
  store i32 %124, ptr %131, align 4
  br label %132

132:                                              ; preds = %103
  %133 = load i32, ptr %5, align 4
  %134 = add nsw i32 %133, 1
  store i32 %134, ptr %5, align 4
  br label %99, !llvm.loop !15

135:                                              ; preds = %99
  %136 = load ptr, ptr %3, align 8
  %137 = call i32 @loadInt(ptr noundef %136)
  store i32 %137, ptr %6, align 4
  %138 = load i32, ptr %6, align 4
  %139 = icmp ne i32 %138, 0
  br i1 %139, label %140, label %144

140:                                              ; preds = %135
  %141 = load ptr, ptr %4, align 8
  %142 = getelementptr inbounds %struct.Proto, ptr %141, i32 0, i32 6
  %143 = load i32, ptr %142, align 8
  store i32 %143, ptr %6, align 4
  br label %144

144:                                              ; preds = %140, %135
  store i32 0, ptr %5, align 4
  br label %145

145:                                              ; preds = %160, %144
  %146 = load i32, ptr %5, align 4
  %147 = load i32, ptr %6, align 4
  %148 = icmp slt i32 %146, %147
  br i1 %148, label %149, label %163

149:                                              ; preds = %145
  %150 = load ptr, ptr %3, align 8
  %151 = load ptr, ptr %4, align 8
  %152 = call ptr @loadStringN(ptr noundef %150, ptr noundef %151)
  %153 = load ptr, ptr %4, align 8
  %154 = getelementptr inbounds %struct.Proto, ptr %153, i32 0, i32 18
  %155 = load ptr, ptr %154, align 8
  %156 = load i32, ptr %5, align 4
  %157 = sext i32 %156 to i64
  %158 = getelementptr inbounds %struct.Upvaldesc, ptr %155, i64 %157
  %159 = getelementptr inbounds %struct.Upvaldesc, ptr %158, i32 0, i32 0
  store ptr %152, ptr %159, align 8
  br label %160

160:                                              ; preds = %149
  %161 = load i32, ptr %5, align 4
  %162 = add nsw i32 %161, 1
  store i32 %162, ptr %5, align 4
  br label %145, !llvm.loop !16

163:                                              ; preds = %145
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @loadSize(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i64 @loadUnsigned(ptr noundef %3, i64 noundef -1)
  ret i64 %4
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden ptr @luaS_createlngstrobj(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @loadUnsigned(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  store i64 0, ptr %5, align 8
  %7 = load i64, ptr %4, align 8
  %8 = lshr i64 %7, 7
  store i64 %8, ptr %4, align 8
  br label %9

9:                                                ; preds = %25, %2
  %10 = load ptr, ptr %3, align 8
  %11 = call zeroext i8 @loadByte(ptr noundef %10)
  %12 = zext i8 %11 to i32
  store i32 %12, ptr %6, align 4
  %13 = load i64, ptr %5, align 8
  %14 = load i64, ptr %4, align 8
  %15 = icmp uge i64 %13, %14
  br i1 %15, label %16, label %18

16:                                               ; preds = %9
  %17 = load ptr, ptr %3, align 8
  call void @error(ptr noundef %17, ptr noundef @.str.15) #5
  unreachable

18:                                               ; preds = %9
  %19 = load i64, ptr %5, align 8
  %20 = shl i64 %19, 7
  %21 = load i32, ptr %6, align 4
  %22 = and i32 %21, 127
  %23 = sext i32 %22 to i64
  %24 = or i64 %20, %23
  store i64 %24, ptr %5, align 8
  br label %25

25:                                               ; preds = %18
  %26 = load i32, ptr %6, align 4
  %27 = and i32 %26, 128
  %28 = icmp eq i32 %27, 0
  br i1 %28, label %9, label %29, !llvm.loop !17

29:                                               ; preds = %25
  %30 = load i64, ptr %5, align 8
  ret i64 %30
}

declare hidden ptr @luaM_malloc_(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @loadString(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call ptr @loadStringN(ptr noundef %6, ptr noundef %7)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = icmp eq ptr %9, null
  br i1 %10, label %11, label %13

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  call void @error(ptr noundef %12, ptr noundef @.str.16) #5
  unreachable

13:                                               ; preds = %2
  %14 = load ptr, ptr %5, align 8
  ret ptr %14
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { noreturn }
attributes #6 = { nounwind willreturn memory(read) }

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
