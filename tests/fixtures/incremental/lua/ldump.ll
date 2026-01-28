; ModuleID = 'ldump.c'
source_filename = "ldump.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.DumpState = type { ptr, ptr, ptr, i32, i32 }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }
%struct.AbsLineInfo = type { i32, i32 }
%struct.LocVar = type { ptr, i32, i32 }

@.str = private unnamed_addr constant [5 x i8] c"\1BLua\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"\19\93\0D\0A\1A\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaU_dump(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca %struct.DumpState, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 0
  store ptr %12, ptr %13, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 1
  store ptr %14, ptr %15, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 2
  store ptr %16, ptr %17, align 8
  %18 = load i32, ptr %10, align 4
  %19 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 3
  store i32 %18, ptr %19, align 8
  %20 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 4
  store i32 0, ptr %20, align 4
  call void @dumpHeader(ptr noundef %11)
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 6
  %23 = load i32, ptr %22, align 8
  call void @dumpByte(ptr noundef %11, i32 noundef %23)
  %24 = load ptr, ptr %7, align 8
  call void @dumpFunction(ptr noundef %11, ptr noundef %24, ptr noundef null)
  %25 = getelementptr inbounds %struct.DumpState, ptr %11, i32 0, i32 4
  %26 = load i32, ptr %25, align 4
  ret i32 %26
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpHeader(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @dumpBlock(ptr noundef %3, ptr noundef @.str, i64 noundef 4)
  %4 = load ptr, ptr %2, align 8
  call void @dumpByte(ptr noundef %4, i32 noundef 84)
  %5 = load ptr, ptr %2, align 8
  call void @dumpByte(ptr noundef %5, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @dumpBlock(ptr noundef %6, ptr noundef @.str.1, i64 noundef 6)
  %7 = load ptr, ptr %2, align 8
  call void @dumpByte(ptr noundef %7, i32 noundef 4)
  %8 = load ptr, ptr %2, align 8
  call void @dumpByte(ptr noundef %8, i32 noundef 8)
  %9 = load ptr, ptr %2, align 8
  call void @dumpByte(ptr noundef %9, i32 noundef 8)
  %10 = load ptr, ptr %2, align 8
  call void @dumpInteger(ptr noundef %10, i64 noundef 22136)
  %11 = load ptr, ptr %2, align 8
  call void @dumpNumber(ptr noundef %11, double noundef 3.705000e+02)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpByte(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i8, align 1
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load i32, ptr %4, align 4
  %7 = trunc i32 %6 to i8
  store i8 %7, ptr %5, align 1
  %8 = load ptr, ptr %3, align 8
  call void @dumpBlock(ptr noundef %8, ptr noundef %5, i64 noundef 1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpFunction(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.DumpState, ptr %7, i32 0, i32 3
  %9 = load i32, ptr %8, align 8
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %17, label %11

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 22
  %14 = load ptr, ptr %13, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = icmp eq ptr %14, %15
  br i1 %16, label %17, label %19

17:                                               ; preds = %11, %3
  %18 = load ptr, ptr %4, align 8
  call void @dumpString(ptr noundef %18, ptr noundef null)
  br label %24

19:                                               ; preds = %11
  %20 = load ptr, ptr %4, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 22
  %23 = load ptr, ptr %22, align 8
  call void @dumpString(ptr noundef %20, ptr noundef %23)
  br label %24

24:                                               ; preds = %19, %17
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.Proto, ptr %26, i32 0, i32 13
  %28 = load i32, ptr %27, align 4
  call void @dumpInt(ptr noundef %25, i32 noundef %28)
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.Proto, ptr %30, i32 0, i32 14
  %32 = load i32, ptr %31, align 8
  call void @dumpInt(ptr noundef %29, i32 noundef %32)
  %33 = load ptr, ptr %4, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 3
  %36 = load i8, ptr %35, align 2
  %37 = zext i8 %36 to i32
  call void @dumpByte(ptr noundef %33, i32 noundef %37)
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.Proto, ptr %39, i32 0, i32 4
  %41 = load i8, ptr %40, align 1
  %42 = zext i8 %41 to i32
  call void @dumpByte(ptr noundef %38, i32 noundef %42)
  %43 = load ptr, ptr %4, align 8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.Proto, ptr %44, i32 0, i32 5
  %46 = load i8, ptr %45, align 4
  %47 = zext i8 %46 to i32
  call void @dumpByte(ptr noundef %43, i32 noundef %47)
  %48 = load ptr, ptr %4, align 8
  %49 = load ptr, ptr %5, align 8
  call void @dumpCode(ptr noundef %48, ptr noundef %49)
  %50 = load ptr, ptr %4, align 8
  %51 = load ptr, ptr %5, align 8
  call void @dumpConstants(ptr noundef %50, ptr noundef %51)
  %52 = load ptr, ptr %4, align 8
  %53 = load ptr, ptr %5, align 8
  call void @dumpUpvalues(ptr noundef %52, ptr noundef %53)
  %54 = load ptr, ptr %4, align 8
  %55 = load ptr, ptr %5, align 8
  call void @dumpProtos(ptr noundef %54, ptr noundef %55)
  %56 = load ptr, ptr %4, align 8
  %57 = load ptr, ptr %5, align 8
  call void @dumpDebug(ptr noundef %56, ptr noundef %57)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpBlock(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.DumpState, ptr %7, i32 0, i32 4
  %9 = load i32, ptr %8, align 4
  %10 = icmp eq i32 %9, 0
  br i1 %10, label %11, label %29

11:                                               ; preds = %3
  %12 = load i64, ptr %6, align 8
  %13 = icmp ugt i64 %12, 0
  br i1 %13, label %14, label %29

14:                                               ; preds = %11
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.DumpState, ptr %15, i32 0, i32 1
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.DumpState, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = load i64, ptr %6, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.DumpState, ptr %23, i32 0, i32 2
  %25 = load ptr, ptr %24, align 8
  %26 = call i32 %17(ptr noundef %20, ptr noundef %21, i64 noundef %22, ptr noundef %25)
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.DumpState, ptr %27, i32 0, i32 4
  store i32 %26, ptr %28, align 4
  br label %29

29:                                               ; preds = %14, %11, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpInteger(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  call void @dumpBlock(ptr noundef %5, ptr noundef %4, i64 noundef 8)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpNumber(ptr noundef %0, double noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca double, align 8
  store ptr %0, ptr %3, align 8
  store double %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  call void @dumpBlock(ptr noundef %5, ptr noundef %4, i64 noundef 8)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpString(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = icmp eq ptr %7, null
  br i1 %8, label %9, label %11

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  call void @dumpSize(ptr noundef %10, i64 noundef 0)
  br label %38

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.TString, ptr %12, i32 0, i32 4
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = icmp ne i32 %15, 255
  br i1 %16, label %17, label %22

17:                                               ; preds = %11
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.TString, ptr %18, i32 0, i32 4
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i64
  br label %26

22:                                               ; preds = %11
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 6
  %25 = load i64, ptr %24, align 8
  br label %26

26:                                               ; preds = %22, %17
  %27 = phi i64 [ %21, %17 ], [ %25, %22 ]
  store i64 %27, ptr %5, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.TString, ptr %28, i32 0, i32 7
  %30 = getelementptr inbounds [1 x i8], ptr %29, i64 0, i64 0
  store ptr %30, ptr %6, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = load i64, ptr %5, align 8
  %33 = add i64 %32, 1
  call void @dumpSize(ptr noundef %31, i64 noundef %33)
  %34 = load ptr, ptr %3, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = load i64, ptr %5, align 8
  %37 = mul i64 %36, 1
  call void @dumpBlock(ptr noundef %34, ptr noundef %35, i64 noundef %37)
  br label %38

38:                                               ; preds = %26, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpInt(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = sext i32 %6 to i64
  call void @dumpSize(ptr noundef %5, i64 noundef %7)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpCode(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Proto, ptr %6, i32 0, i32 8
  %8 = load i32, ptr %7, align 8
  call void @dumpInt(ptr noundef %5, i32 noundef %8)
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.Proto, ptr %10, i32 0, i32 16
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.Proto, ptr %13, i32 0, i32 8
  %15 = load i32, ptr %14, align 8
  %16 = sext i32 %15 to i64
  %17 = mul i64 %16, 4
  call void @dumpBlock(ptr noundef %9, ptr noundef %12, i64 noundef %17)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpConstants(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 7
  %11 = load i32, ptr %10, align 4
  store i32 %11, ptr %6, align 4
  %12 = load ptr, ptr %3, align 8
  %13 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %12, i32 noundef %13)
  store i32 0, ptr %5, align 4
  br label %14

14:                                               ; preds = %50, %2
  %15 = load i32, ptr %5, align 4
  %16 = load i32, ptr %6, align 4
  %17 = icmp slt i32 %15, %16
  br i1 %17, label %18, label %53

18:                                               ; preds = %14
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.Proto, ptr %19, i32 0, i32 15
  %21 = load ptr, ptr %20, align 8
  %22 = load i32, ptr %5, align 4
  %23 = sext i32 %22 to i64
  %24 = getelementptr inbounds %struct.TValue, ptr %21, i64 %23
  store ptr %24, ptr %7, align 8
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  %27 = load i8, ptr %26, align 8
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 63
  store i32 %29, ptr %8, align 4
  %30 = load ptr, ptr %3, align 8
  %31 = load i32, ptr %8, align 4
  call void @dumpByte(ptr noundef %30, i32 noundef %31)
  %32 = load i32, ptr %8, align 4
  switch i32 %32, label %48 [
    i32 19, label %33
    i32 3, label %38
    i32 4, label %43
    i32 20, label %43
  ]

33:                                               ; preds = %18
  %34 = load ptr, ptr %3, align 8
  %35 = load ptr, ptr %7, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 0
  %37 = load double, ptr %36, align 8
  call void @dumpNumber(ptr noundef %34, double noundef %37)
  br label %49

38:                                               ; preds = %18
  %39 = load ptr, ptr %3, align 8
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load i64, ptr %41, align 8
  call void @dumpInteger(ptr noundef %39, i64 noundef %42)
  br label %49

43:                                               ; preds = %18, %18
  %44 = load ptr, ptr %3, align 8
  %45 = load ptr, ptr %7, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 0
  %47 = load ptr, ptr %46, align 8
  call void @dumpString(ptr noundef %44, ptr noundef %47)
  br label %49

48:                                               ; preds = %18
  br label %49

49:                                               ; preds = %48, %43, %38, %33
  br label %50

50:                                               ; preds = %49
  %51 = load i32, ptr %5, align 4
  %52 = add nsw i32 %51, 1
  store i32 %52, ptr %5, align 4
  br label %14, !llvm.loop !6

53:                                               ; preds = %14
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpUpvalues(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 6
  %9 = load i32, ptr %8, align 8
  store i32 %9, ptr %6, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %10, i32 noundef %11)
  store i32 0, ptr %5, align 4
  br label %12

12:                                               ; preds = %47, %2
  %13 = load i32, ptr %5, align 4
  %14 = load i32, ptr %6, align 4
  %15 = icmp slt i32 %13, %14
  br i1 %15, label %16, label %50

16:                                               ; preds = %12
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 18
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %5, align 4
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds %struct.Upvaldesc, ptr %20, i64 %22
  %24 = getelementptr inbounds %struct.Upvaldesc, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  call void @dumpByte(ptr noundef %17, i32 noundef %26)
  %27 = load ptr, ptr %3, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 18
  %30 = load ptr, ptr %29, align 8
  %31 = load i32, ptr %5, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds %struct.Upvaldesc, ptr %30, i64 %32
  %34 = getelementptr inbounds %struct.Upvaldesc, ptr %33, i32 0, i32 2
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  call void @dumpByte(ptr noundef %27, i32 noundef %36)
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.Proto, ptr %38, i32 0, i32 18
  %40 = load ptr, ptr %39, align 8
  %41 = load i32, ptr %5, align 4
  %42 = sext i32 %41 to i64
  %43 = getelementptr inbounds %struct.Upvaldesc, ptr %40, i64 %42
  %44 = getelementptr inbounds %struct.Upvaldesc, ptr %43, i32 0, i32 3
  %45 = load i8, ptr %44, align 2
  %46 = zext i8 %45 to i32
  call void @dumpByte(ptr noundef %37, i32 noundef %46)
  br label %47

47:                                               ; preds = %16
  %48 = load i32, ptr %5, align 4
  %49 = add nsw i32 %48, 1
  store i32 %49, ptr %5, align 4
  br label %12, !llvm.loop !8

50:                                               ; preds = %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpProtos(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 10
  %9 = load i32, ptr %8, align 8
  store i32 %9, ptr %6, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %10, i32 noundef %11)
  store i32 0, ptr %5, align 4
  br label %12

12:                                               ; preds = %28, %2
  %13 = load i32, ptr %5, align 4
  %14 = load i32, ptr %6, align 4
  %15 = icmp slt i32 %13, %14
  br i1 %15, label %16, label %31

16:                                               ; preds = %12
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 17
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %5, align 4
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds ptr, ptr %20, i64 %22
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.Proto, ptr %25, i32 0, i32 22
  %27 = load ptr, ptr %26, align 8
  call void @dumpFunction(ptr noundef %17, ptr noundef %24, ptr noundef %27)
  br label %28

28:                                               ; preds = %16
  %29 = load i32, ptr %5, align 4
  %30 = add nsw i32 %29, 1
  store i32 %30, ptr %5, align 4
  br label %12, !llvm.loop !9

31:                                               ; preds = %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpDebug(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.DumpState, ptr %7, i32 0, i32 3
  %9 = load i32, ptr %8, align 8
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %12

11:                                               ; preds = %2
  br label %16

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.Proto, ptr %13, i32 0, i32 9
  %15 = load i32, ptr %14, align 4
  br label %16

16:                                               ; preds = %12, %11
  %17 = phi i32 [ 0, %11 ], [ %15, %12 ]
  store i32 %17, ptr %6, align 4
  %18 = load ptr, ptr %3, align 8
  %19 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %18, i32 noundef %19)
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 19
  %23 = load ptr, ptr %22, align 8
  %24 = load i32, ptr %6, align 4
  %25 = sext i32 %24 to i64
  %26 = mul i64 %25, 1
  call void @dumpBlock(ptr noundef %20, ptr noundef %23, i64 noundef %26)
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.DumpState, ptr %27, i32 0, i32 3
  %29 = load i32, ptr %28, align 8
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %32

31:                                               ; preds = %16
  br label %36

32:                                               ; preds = %16
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.Proto, ptr %33, i32 0, i32 12
  %35 = load i32, ptr %34, align 8
  br label %36

36:                                               ; preds = %32, %31
  %37 = phi i32 [ 0, %31 ], [ %35, %32 ]
  store i32 %37, ptr %6, align 4
  %38 = load ptr, ptr %3, align 8
  %39 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %38, i32 noundef %39)
  store i32 0, ptr %5, align 4
  br label %40

40:                                               ; preds = %63, %36
  %41 = load i32, ptr %5, align 4
  %42 = load i32, ptr %6, align 4
  %43 = icmp slt i32 %41, %42
  br i1 %43, label %44, label %66

44:                                               ; preds = %40
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.Proto, ptr %46, i32 0, i32 20
  %48 = load ptr, ptr %47, align 8
  %49 = load i32, ptr %5, align 4
  %50 = sext i32 %49 to i64
  %51 = getelementptr inbounds %struct.AbsLineInfo, ptr %48, i64 %50
  %52 = getelementptr inbounds %struct.AbsLineInfo, ptr %51, i32 0, i32 0
  %53 = load i32, ptr %52, align 4
  call void @dumpInt(ptr noundef %45, i32 noundef %53)
  %54 = load ptr, ptr %3, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.Proto, ptr %55, i32 0, i32 20
  %57 = load ptr, ptr %56, align 8
  %58 = load i32, ptr %5, align 4
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds %struct.AbsLineInfo, ptr %57, i64 %59
  %61 = getelementptr inbounds %struct.AbsLineInfo, ptr %60, i32 0, i32 1
  %62 = load i32, ptr %61, align 4
  call void @dumpInt(ptr noundef %54, i32 noundef %62)
  br label %63

63:                                               ; preds = %44
  %64 = load i32, ptr %5, align 4
  %65 = add nsw i32 %64, 1
  store i32 %65, ptr %5, align 4
  br label %40, !llvm.loop !10

66:                                               ; preds = %40
  %67 = load ptr, ptr %3, align 8
  %68 = getelementptr inbounds %struct.DumpState, ptr %67, i32 0, i32 3
  %69 = load i32, ptr %68, align 8
  %70 = icmp ne i32 %69, 0
  br i1 %70, label %71, label %72

71:                                               ; preds = %66
  br label %76

72:                                               ; preds = %66
  %73 = load ptr, ptr %4, align 8
  %74 = getelementptr inbounds %struct.Proto, ptr %73, i32 0, i32 11
  %75 = load i32, ptr %74, align 4
  br label %76

76:                                               ; preds = %72, %71
  %77 = phi i32 [ 0, %71 ], [ %75, %72 ]
  store i32 %77, ptr %6, align 4
  %78 = load ptr, ptr %3, align 8
  %79 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %78, i32 noundef %79)
  store i32 0, ptr %5, align 4
  br label %80

80:                                               ; preds = %112, %76
  %81 = load i32, ptr %5, align 4
  %82 = load i32, ptr %6, align 4
  %83 = icmp slt i32 %81, %82
  br i1 %83, label %84, label %115

84:                                               ; preds = %80
  %85 = load ptr, ptr %3, align 8
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.Proto, ptr %86, i32 0, i32 21
  %88 = load ptr, ptr %87, align 8
  %89 = load i32, ptr %5, align 4
  %90 = sext i32 %89 to i64
  %91 = getelementptr inbounds %struct.LocVar, ptr %88, i64 %90
  %92 = getelementptr inbounds %struct.LocVar, ptr %91, i32 0, i32 0
  %93 = load ptr, ptr %92, align 8
  call void @dumpString(ptr noundef %85, ptr noundef %93)
  %94 = load ptr, ptr %3, align 8
  %95 = load ptr, ptr %4, align 8
  %96 = getelementptr inbounds %struct.Proto, ptr %95, i32 0, i32 21
  %97 = load ptr, ptr %96, align 8
  %98 = load i32, ptr %5, align 4
  %99 = sext i32 %98 to i64
  %100 = getelementptr inbounds %struct.LocVar, ptr %97, i64 %99
  %101 = getelementptr inbounds %struct.LocVar, ptr %100, i32 0, i32 1
  %102 = load i32, ptr %101, align 8
  call void @dumpInt(ptr noundef %94, i32 noundef %102)
  %103 = load ptr, ptr %3, align 8
  %104 = load ptr, ptr %4, align 8
  %105 = getelementptr inbounds %struct.Proto, ptr %104, i32 0, i32 21
  %106 = load ptr, ptr %105, align 8
  %107 = load i32, ptr %5, align 4
  %108 = sext i32 %107 to i64
  %109 = getelementptr inbounds %struct.LocVar, ptr %106, i64 %108
  %110 = getelementptr inbounds %struct.LocVar, ptr %109, i32 0, i32 2
  %111 = load i32, ptr %110, align 4
  call void @dumpInt(ptr noundef %103, i32 noundef %111)
  br label %112

112:                                              ; preds = %84
  %113 = load i32, ptr %5, align 4
  %114 = add nsw i32 %113, 1
  store i32 %114, ptr %5, align 4
  br label %80, !llvm.loop !11

115:                                              ; preds = %80
  %116 = load ptr, ptr %3, align 8
  %117 = getelementptr inbounds %struct.DumpState, ptr %116, i32 0, i32 3
  %118 = load i32, ptr %117, align 8
  %119 = icmp ne i32 %118, 0
  br i1 %119, label %120, label %121

120:                                              ; preds = %115
  br label %125

121:                                              ; preds = %115
  %122 = load ptr, ptr %4, align 8
  %123 = getelementptr inbounds %struct.Proto, ptr %122, i32 0, i32 6
  %124 = load i32, ptr %123, align 8
  br label %125

125:                                              ; preds = %121, %120
  %126 = phi i32 [ 0, %120 ], [ %124, %121 ]
  store i32 %126, ptr %6, align 4
  %127 = load ptr, ptr %3, align 8
  %128 = load i32, ptr %6, align 4
  call void @dumpInt(ptr noundef %127, i32 noundef %128)
  store i32 0, ptr %5, align 4
  br label %129

129:                                              ; preds = %143, %125
  %130 = load i32, ptr %5, align 4
  %131 = load i32, ptr %6, align 4
  %132 = icmp slt i32 %130, %131
  br i1 %132, label %133, label %146

133:                                              ; preds = %129
  %134 = load ptr, ptr %3, align 8
  %135 = load ptr, ptr %4, align 8
  %136 = getelementptr inbounds %struct.Proto, ptr %135, i32 0, i32 18
  %137 = load ptr, ptr %136, align 8
  %138 = load i32, ptr %5, align 4
  %139 = sext i32 %138 to i64
  %140 = getelementptr inbounds %struct.Upvaldesc, ptr %137, i64 %139
  %141 = getelementptr inbounds %struct.Upvaldesc, ptr %140, i32 0, i32 0
  %142 = load ptr, ptr %141, align 8
  call void @dumpString(ptr noundef %134, ptr noundef %142)
  br label %143

143:                                              ; preds = %133
  %144 = load i32, ptr %5, align 4
  %145 = add nsw i32 %144, 1
  store i32 %145, ptr %5, align 4
  br label %129, !llvm.loop !12

146:                                              ; preds = %129
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dumpSize(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca [10 x i8], align 1
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  store i32 0, ptr %6, align 4
  br label %7

7:                                                ; preds = %18, %2
  %8 = load i64, ptr %4, align 8
  %9 = and i64 %8, 127
  %10 = trunc i64 %9 to i8
  %11 = load i32, ptr %6, align 4
  %12 = add nsw i32 %11, 1
  store i32 %12, ptr %6, align 4
  %13 = sext i32 %12 to i64
  %14 = sub i64 10, %13
  %15 = getelementptr inbounds [10 x i8], ptr %5, i64 0, i64 %14
  store i8 %10, ptr %15, align 1
  %16 = load i64, ptr %4, align 8
  %17 = lshr i64 %16, 7
  store i64 %17, ptr %4, align 8
  br label %18

18:                                               ; preds = %7
  %19 = load i64, ptr %4, align 8
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %7, label %21, !llvm.loop !13

21:                                               ; preds = %18
  %22 = getelementptr inbounds [10 x i8], ptr %5, i64 0, i64 9
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = or i32 %24, 128
  %26 = trunc i32 %25 to i8
  store i8 %26, ptr %22, align 1
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds [10 x i8], ptr %5, i64 0, i64 0
  %29 = getelementptr inbounds i8, ptr %28, i64 10
  %30 = load i32, ptr %6, align 4
  %31 = sext i32 %30 to i64
  %32 = sub i64 0, %31
  %33 = getelementptr inbounds i8, ptr %29, i64 %32
  %34 = load i32, ptr %6, align 4
  %35 = sext i32 %34 to i64
  %36 = mul i64 %35, 1
  call void @dumpBlock(ptr noundef %27, ptr noundef %33, i64 noundef %36)
  ret void
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

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
