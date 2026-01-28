; ModuleID = 'lfunc.c'
source_filename = "lfunc.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.CClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x %struct.TValue] }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.UpVal = type { ptr, i8, i8, %union.anon.4, %union.anon.5 }
%union.anon.4 = type { ptr }
%union.anon.5 = type { %struct.anon.6 }
%struct.anon.6 = type { ptr, ptr }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon.0, %union.anon.2, i16, i16 }
%union.anon.0 = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%union.StackValue = type { %struct.TValue }
%struct.anon.7 = type { %union.Value, i8, i16 }
%struct.GCObject = type { ptr, i8, i8 }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.LocVar = type { ptr, i32, i32 }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }

@.str = private unnamed_addr constant [2 x i8] c"?\00", align 1
@.str.1 = private unnamed_addr constant [39 x i8] c"variable '%s' got a non-closable value\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_newCclosure(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = mul nsw i32 16, %8
  %10 = add nsw i32 32, %9
  %11 = sext i32 %10 to i64
  %12 = call ptr @luaC_newobj(ptr noundef %7, i32 noundef 38, i64 noundef %11)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  store ptr %13, ptr %6, align 8
  %14 = load i32, ptr %4, align 4
  %15 = trunc i32 %14 to i8
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.CClosure, ptr %16, i32 0, i32 3
  store i8 %15, ptr %17, align 2
  %18 = load ptr, ptr %6, align 8
  ret ptr %18
}

declare hidden ptr @luaC_newobj(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_newLclosure(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = mul nsw i32 8, %8
  %10 = add nsw i32 32, %9
  %11 = sext i32 %10 to i64
  %12 = call ptr @luaC_newobj(ptr noundef %7, i32 noundef 6, i64 noundef %11)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = getelementptr inbounds %struct.LClosure, ptr %14, i32 0, i32 5
  store ptr null, ptr %15, align 8
  %16 = load i32, ptr %4, align 4
  %17 = trunc i32 %16 to i8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.LClosure, ptr %18, i32 0, i32 3
  store i8 %17, ptr %19, align 2
  br label %20

20:                                               ; preds = %24, %2
  %21 = load i32, ptr %4, align 4
  %22 = add nsw i32 %21, -1
  store i32 %22, ptr %4, align 4
  %23 = icmp ne i32 %21, 0
  br i1 %23, label %24, label %30

24:                                               ; preds = %20
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.LClosure, ptr %25, i32 0, i32 6
  %27 = load i32, ptr %4, align 4
  %28 = sext i32 %27 to i64
  %29 = getelementptr inbounds [1 x ptr], ptr %26, i64 0, i64 %28
  store ptr null, ptr %29, align 8
  br label %20, !llvm.loop !6

30:                                               ; preds = %20
  %31 = load ptr, ptr %6, align 8
  ret ptr %31
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaF_initupvals(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %5, align 4
  br label %8

8:                                                ; preds = %52, %2
  %9 = load i32, ptr %5, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LClosure, ptr %10, i32 0, i32 3
  %12 = load i8, ptr %11, align 2
  %13 = zext i8 %12 to i32
  %14 = icmp slt i32 %9, %13
  br i1 %14, label %15, label %55

15:                                               ; preds = %8
  %16 = load ptr, ptr %3, align 8
  %17 = call ptr @luaC_newobj(ptr noundef %16, i32 noundef 9, i64 noundef 40)
  store ptr %17, ptr %6, align 8
  %18 = load ptr, ptr %6, align 8
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.UpVal, ptr %19, i32 0, i32 4
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.UpVal, ptr %21, i32 0, i32 3
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.UpVal, ptr %23, i32 0, i32 3
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  store i8 0, ptr %26, align 8
  %27 = load ptr, ptr %7, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.LClosure, ptr %28, i32 0, i32 6
  %30 = load i32, ptr %5, align 4
  %31 = sext i32 %30 to i64
  %32 = getelementptr inbounds [1 x ptr], ptr %29, i64 0, i64 %31
  store ptr %27, ptr %32, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.LClosure, ptr %33, i32 0, i32 2
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  %37 = and i32 %36, 32
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %39, label %50

39:                                               ; preds = %15
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.UpVal, ptr %40, i32 0, i32 2
  %42 = load i8, ptr %41, align 1
  %43 = zext i8 %42 to i32
  %44 = and i32 %43, 24
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %46, label %50

46:                                               ; preds = %39
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = load ptr, ptr %7, align 8
  call void @luaC_barrier_(ptr noundef %47, ptr noundef %48, ptr noundef %49)
  br label %51

50:                                               ; preds = %39, %15
  br label %51

51:                                               ; preds = %50, %46
  br label %52

52:                                               ; preds = %51
  %53 = load i32, ptr %5, align 4
  %54 = add nsw i32 %53, 1
  store i32 %54, ptr %5, align 4
  br label %8, !llvm.loop !8

55:                                               ; preds = %8
  ret void
}

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_findupval(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 11
  store ptr %9, ptr %6, align 8
  br label %10

10:                                               ; preds = %30, %2
  %11 = load ptr, ptr %6, align 8
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  %13 = icmp ne ptr %12, null
  br i1 %13, label %14, label %20

14:                                               ; preds = %10
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.UpVal, ptr %15, i32 0, i32 3
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = icmp uge ptr %17, %18
  br label %20

20:                                               ; preds = %14, %10
  %21 = phi i1 [ false, %10 ], [ %19, %14 ]
  br i1 %21, label %22, label %34

22:                                               ; preds = %20
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.UpVal, ptr %23, i32 0, i32 3
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = icmp eq ptr %25, %26
  br i1 %27, label %28, label %30

28:                                               ; preds = %22
  %29 = load ptr, ptr %7, align 8
  store ptr %29, ptr %3, align 8
  br label %39

30:                                               ; preds = %22
  %31 = load ptr, ptr %7, align 8
  %32 = getelementptr inbounds %struct.UpVal, ptr %31, i32 0, i32 4
  %33 = getelementptr inbounds %struct.anon.6, ptr %32, i32 0, i32 0
  store ptr %33, ptr %6, align 8
  br label %10, !llvm.loop !9

34:                                               ; preds = %20
  %35 = load ptr, ptr %4, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = load ptr, ptr %6, align 8
  %38 = call ptr @newupval(ptr noundef %35, ptr noundef %36, ptr noundef %37)
  store ptr %38, ptr %3, align 8
  br label %39

39:                                               ; preds = %34, %28
  %40 = load ptr, ptr %3, align 8
  ret ptr %40
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @newupval(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = call ptr @luaC_newobj(ptr noundef %10, i32 noundef 9, i64 noundef 40)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %7, align 8
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = load ptr, ptr %8, align 8
  %17 = getelementptr inbounds %struct.UpVal, ptr %16, i32 0, i32 3
  store ptr %15, ptr %17, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds %struct.UpVal, ptr %19, i32 0, i32 4
  %21 = getelementptr inbounds %struct.anon.6, ptr %20, i32 0, i32 0
  store ptr %18, ptr %21, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = getelementptr inbounds %struct.UpVal, ptr %23, i32 0, i32 4
  %25 = getelementptr inbounds %struct.anon.6, ptr %24, i32 0, i32 1
  store ptr %22, ptr %25, align 8
  %26 = load ptr, ptr %9, align 8
  %27 = icmp ne ptr %26, null
  br i1 %27, label %28, label %35

28:                                               ; preds = %3
  %29 = load ptr, ptr %8, align 8
  %30 = getelementptr inbounds %struct.UpVal, ptr %29, i32 0, i32 4
  %31 = getelementptr inbounds %struct.anon.6, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %9, align 8
  %33 = getelementptr inbounds %struct.UpVal, ptr %32, i32 0, i32 4
  %34 = getelementptr inbounds %struct.anon.6, ptr %33, i32 0, i32 1
  store ptr %31, ptr %34, align 8
  br label %35

35:                                               ; preds = %28, %3
  %36 = load ptr, ptr %8, align 8
  %37 = load ptr, ptr %6, align 8
  store ptr %36, ptr %37, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 14
  %40 = load ptr, ptr %39, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = icmp ne ptr %40, %41
  br i1 %42, label %56, label %43

43:                                               ; preds = %35
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 7
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %struct.global_State, ptr %46, i32 0, i32 38
  %48 = load ptr, ptr %47, align 8
  %49 = load ptr, ptr %4, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 14
  store ptr %48, ptr %50, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = load ptr, ptr %4, align 8
  %53 = getelementptr inbounds %struct.lua_State, ptr %52, i32 0, i32 7
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr inbounds %struct.global_State, ptr %54, i32 0, i32 38
  store ptr %51, ptr %55, align 8
  br label %56

56:                                               ; preds = %43, %35
  %57 = load ptr, ptr %8, align 8
  ret ptr %57
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaF_newtbcupval(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.TValue, ptr %5, i32 0, i32 1
  %7 = load i8, ptr %6, align 8
  %8 = zext i8 %7 to i32
  %9 = icmp eq i32 %8, 1
  br i1 %9, label %17, label %10

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 1
  %13 = load i8, ptr %12, align 8
  %14 = zext i8 %13 to i32
  %15 = and i32 %14, 15
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %18

17:                                               ; preds = %10, %2
  br label %57

18:                                               ; preds = %10
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %4, align 8
  call void @checkclosemth(ptr noundef %19, ptr noundef %20)
  br label %21

21:                                               ; preds = %33, %18
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 12
  %25 = load ptr, ptr %24, align 8
  %26 = ptrtoint ptr %22 to i64
  %27 = ptrtoint ptr %25 to i64
  %28 = sub i64 %26, %27
  %29 = sdiv exact i64 %28, 16
  %30 = trunc i64 %29 to i32
  %31 = zext i32 %30 to i64
  %32 = icmp ugt i64 %31, 65535
  br i1 %32, label %33, label %42

33:                                               ; preds = %21
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 12
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %union.StackValue, ptr %36, i64 65535
  store ptr %37, ptr %35, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 12
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds %struct.anon.7, ptr %40, i32 0, i32 2
  store i16 0, ptr %41, align 2
  br label %21, !llvm.loop !10

42:                                               ; preds = %21
  %43 = load ptr, ptr %4, align 8
  %44 = load ptr, ptr %3, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 12
  %46 = load ptr, ptr %45, align 8
  %47 = ptrtoint ptr %43 to i64
  %48 = ptrtoint ptr %46 to i64
  %49 = sub i64 %47, %48
  %50 = sdiv exact i64 %49, 16
  %51 = trunc i64 %50 to i16
  %52 = load ptr, ptr %4, align 8
  %53 = getelementptr inbounds %struct.anon.7, ptr %52, i32 0, i32 2
  store i16 %51, ptr %53, align 2
  %54 = load ptr, ptr %4, align 8
  %55 = load ptr, ptr %3, align 8
  %56 = getelementptr inbounds %struct.lua_State, ptr %55, i32 0, i32 12
  store ptr %54, ptr %56, align 8
  br label %57

57:                                               ; preds = %42, %17
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkclosemth(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @luaT_gettmbyobj(ptr noundef %8, ptr noundef %9, i32 noundef 24)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 1
  %13 = load i8, ptr %12, align 8
  %14 = zext i8 %13 to i32
  %15 = and i32 %14, 15
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %41

17:                                               ; preds = %2
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 8
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.CallInfo, ptr %21, i32 0, i32 0
  %23 = load ptr, ptr %22, align 8
  %24 = ptrtoint ptr %18 to i64
  %25 = ptrtoint ptr %23 to i64
  %26 = sub i64 %24, %25
  %27 = sdiv exact i64 %26, 16
  %28 = trunc i64 %27 to i32
  store i32 %28, ptr %6, align 4
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 8
  %32 = load ptr, ptr %31, align 8
  %33 = load i32, ptr %6, align 4
  %34 = call ptr @luaG_findlocal(ptr noundef %29, ptr noundef %32, i32 noundef %33, ptr noundef null)
  store ptr %34, ptr %7, align 8
  %35 = load ptr, ptr %7, align 8
  %36 = icmp eq ptr %35, null
  br i1 %36, label %37, label %38

37:                                               ; preds = %17
  store ptr @.str, ptr %7, align 8
  br label %38

38:                                               ; preds = %37, %17
  %39 = load ptr, ptr %3, align 8
  %40 = load ptr, ptr %7, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %39, ptr noundef @.str.1, ptr noundef %40) #4
  unreachable

41:                                               ; preds = %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaF_unlinkupval(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.UpVal, ptr %3, i32 0, i32 4
  %5 = getelementptr inbounds %struct.anon.6, ptr %4, i32 0, i32 0
  %6 = load ptr, ptr %5, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.UpVal, ptr %7, i32 0, i32 4
  %9 = getelementptr inbounds %struct.anon.6, ptr %8, i32 0, i32 1
  %10 = load ptr, ptr %9, align 8
  store ptr %6, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.UpVal, ptr %11, i32 0, i32 4
  %13 = getelementptr inbounds %struct.anon.6, ptr %12, i32 0, i32 0
  %14 = load ptr, ptr %13, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %27

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.UpVal, ptr %17, i32 0, i32 4
  %19 = getelementptr inbounds %struct.anon.6, ptr %18, i32 0, i32 1
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.UpVal, ptr %21, i32 0, i32 4
  %23 = getelementptr inbounds %struct.anon.6, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.UpVal, ptr %24, i32 0, i32 4
  %26 = getelementptr inbounds %struct.anon.6, ptr %25, i32 0, i32 1
  store ptr %20, ptr %26, align 8
  br label %27

27:                                               ; preds = %16, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaF_closeupval(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  br label %10

10:                                               ; preds = %89, %2
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 11
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %5, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %21

15:                                               ; preds = %10
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.UpVal, ptr %16, i32 0, i32 3
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %6, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = icmp uge ptr %18, %19
  br label %21

21:                                               ; preds = %15, %10
  %22 = phi i1 [ false, %10 ], [ %20, %15 ]
  br i1 %22, label %23, label %90

23:                                               ; preds = %21
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.UpVal, ptr %24, i32 0, i32 4
  store ptr %25, ptr %7, align 8
  %26 = load ptr, ptr %5, align 8
  call void @luaF_unlinkupval(ptr noundef %26)
  %27 = load ptr, ptr %7, align 8
  store ptr %27, ptr %8, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.UpVal, ptr %28, i32 0, i32 3
  %30 = load ptr, ptr %29, align 8
  store ptr %30, ptr %9, align 8
  %31 = load ptr, ptr %8, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %9, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %32, ptr align 8 %34, i64 8, i1 false)
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = load ptr, ptr %8, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 1
  store i8 %37, ptr %39, align 8
  %40 = load ptr, ptr %3, align 8
  %41 = load ptr, ptr %7, align 8
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.UpVal, ptr %42, i32 0, i32 3
  store ptr %41, ptr %43, align 8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.UpVal, ptr %44, i32 0, i32 2
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = and i32 %47, 24
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %89, label %50

50:                                               ; preds = %23
  %51 = load ptr, ptr %5, align 8
  %52 = getelementptr inbounds %struct.UpVal, ptr %51, i32 0, i32 2
  %53 = load i8, ptr %52, align 1
  %54 = zext i8 %53 to i32
  %55 = or i32 %54, 32
  %56 = trunc i32 %55 to i8
  store i8 %56, ptr %52, align 1
  %57 = load ptr, ptr %7, align 8
  %58 = getelementptr inbounds %struct.TValue, ptr %57, i32 0, i32 1
  %59 = load i8, ptr %58, align 8
  %60 = zext i8 %59 to i32
  %61 = and i32 %60, 64
  %62 = icmp ne i32 %61, 0
  br i1 %62, label %63, label %87

63:                                               ; preds = %50
  %64 = load ptr, ptr %5, align 8
  %65 = getelementptr inbounds %struct.UpVal, ptr %64, i32 0, i32 2
  %66 = load i8, ptr %65, align 1
  %67 = zext i8 %66 to i32
  %68 = and i32 %67, 32
  %69 = icmp ne i32 %68, 0
  br i1 %69, label %70, label %85

70:                                               ; preds = %63
  %71 = load ptr, ptr %7, align 8
  %72 = getelementptr inbounds %struct.TValue, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr inbounds %struct.GCObject, ptr %73, i32 0, i32 2
  %75 = load i8, ptr %74, align 1
  %76 = zext i8 %75 to i32
  %77 = and i32 %76, 24
  %78 = icmp ne i32 %77, 0
  br i1 %78, label %79, label %85

79:                                               ; preds = %70
  %80 = load ptr, ptr %3, align 8
  %81 = load ptr, ptr %5, align 8
  %82 = load ptr, ptr %7, align 8
  %83 = getelementptr inbounds %struct.TValue, ptr %82, i32 0, i32 0
  %84 = load ptr, ptr %83, align 8
  call void @luaC_barrier_(ptr noundef %80, ptr noundef %81, ptr noundef %84)
  br label %86

85:                                               ; preds = %70, %63
  br label %86

86:                                               ; preds = %85, %79
  br label %88

87:                                               ; preds = %50
  br label %88

88:                                               ; preds = %87, %86
  br label %89

89:                                               ; preds = %88, %23
  br label %10, !llvm.loop !11

90:                                               ; preds = %21
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_close(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %6, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 10
  %14 = load ptr, ptr %13, align 8
  %15 = ptrtoint ptr %11 to i64
  %16 = ptrtoint ptr %14 to i64
  %17 = sub i64 %15, %16
  store i64 %17, ptr %9, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %6, align 8
  call void @luaF_closeupval(ptr noundef %18, ptr noundef %19)
  br label %20

20:                                               ; preds = %26, %4
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 12
  %23 = load ptr, ptr %22, align 8
  %24 = load ptr, ptr %6, align 8
  %25 = icmp uge ptr %23, %24
  br i1 %25, label %26, label %40

26:                                               ; preds = %20
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 12
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %10, align 8
  %30 = load ptr, ptr %5, align 8
  call void @poptbclist(ptr noundef %30)
  %31 = load ptr, ptr %5, align 8
  %32 = load ptr, ptr %10, align 8
  %33 = load i32, ptr %7, align 4
  %34 = load i32, ptr %8, align 4
  call void @prepcallclosemth(ptr noundef %31, ptr noundef %32, i32 noundef %33, i32 noundef %34)
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 10
  %37 = load ptr, ptr %36, align 8
  %38 = load i64, ptr %9, align 8
  %39 = getelementptr inbounds i8, ptr %37, i64 %38
  store ptr %39, ptr %6, align 8
  br label %20, !llvm.loop !12

40:                                               ; preds = %20
  %41 = load ptr, ptr %6, align 8
  ret ptr %41
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @poptbclist(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 12
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.anon.7, ptr %7, i32 0, i32 2
  %9 = load i16, ptr %8, align 2
  %10 = zext i16 %9 to i32
  %11 = load ptr, ptr %3, align 8
  %12 = sext i32 %10 to i64
  %13 = sub i64 0, %12
  %14 = getelementptr inbounds %union.StackValue, ptr %11, i64 %13
  store ptr %14, ptr %3, align 8
  br label %15

15:                                               ; preds = %29, %1
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 10
  %19 = load ptr, ptr %18, align 8
  %20 = icmp ugt ptr %16, %19
  br i1 %20, label %21, label %27

21:                                               ; preds = %15
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.anon.7, ptr %22, i32 0, i32 2
  %24 = load i16, ptr %23, align 2
  %25 = zext i16 %24 to i32
  %26 = icmp eq i32 %25, 0
  br label %27

27:                                               ; preds = %21, %15
  %28 = phi i1 [ false, %15 ], [ %26, %21 ]
  br i1 %28, label %29, label %32

29:                                               ; preds = %27
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %union.StackValue, ptr %30, i64 -65535
  store ptr %31, ptr %3, align 8
  br label %15, !llvm.loop !13

32:                                               ; preds = %27
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %2, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 12
  store ptr %33, ptr %35, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @prepcallclosemth(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %6, align 8
  store ptr %11, ptr %9, align 8
  %12 = load i32, ptr %7, align 4
  %13 = icmp eq i32 %12, -1
  br i1 %13, label %14, label %19

14:                                               ; preds = %4
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 7
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 8
  store ptr %18, ptr %10, align 8
  br label %26

19:                                               ; preds = %4
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %union.StackValue, ptr %20, i64 1
  store ptr %21, ptr %10, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = load i32, ptr %7, align 4
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %union.StackValue, ptr %24, i64 1
  call void @luaD_seterrorobj(ptr noundef %22, i32 noundef %23, ptr noundef %25)
  br label %26

26:                                               ; preds = %19, %14
  %27 = load ptr, ptr %5, align 8
  %28 = load ptr, ptr %9, align 8
  %29 = load ptr, ptr %10, align 8
  %30 = load i32, ptr %8, align 4
  call void @callclosemethod(ptr noundef %27, ptr noundef %28, ptr noundef %29, i32 noundef %30)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_newproto(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaC_newobj(ptr noundef %5, i32 noundef 10, i64 noundef 128)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Proto, ptr %8, i32 0, i32 15
  store ptr null, ptr %9, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.Proto, ptr %10, i32 0, i32 7
  store i32 0, ptr %11, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.Proto, ptr %12, i32 0, i32 17
  store ptr null, ptr %13, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.Proto, ptr %14, i32 0, i32 10
  store i32 0, ptr %15, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 16
  store ptr null, ptr %17, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 8
  store i32 0, ptr %19, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.Proto, ptr %20, i32 0, i32 19
  store ptr null, ptr %21, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 9
  store i32 0, ptr %23, align 4
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 20
  store ptr null, ptr %25, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.Proto, ptr %26, i32 0, i32 12
  store i32 0, ptr %27, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 18
  store ptr null, ptr %29, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.Proto, ptr %30, i32 0, i32 6
  store i32 0, ptr %31, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.Proto, ptr %32, i32 0, i32 3
  store i8 0, ptr %33, align 2
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.Proto, ptr %34, i32 0, i32 4
  store i8 0, ptr %35, align 1
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 5
  store i8 0, ptr %37, align 4
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.Proto, ptr %38, i32 0, i32 21
  store ptr null, ptr %39, align 8
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.Proto, ptr %40, i32 0, i32 11
  store i32 0, ptr %41, align 4
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.Proto, ptr %42, i32 0, i32 13
  store i32 0, ptr %43, align 4
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.Proto, ptr %44, i32 0, i32 14
  store i32 0, ptr %45, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.Proto, ptr %46, i32 0, i32 22
  store ptr null, ptr %47, align 8
  %48 = load ptr, ptr %4, align 8
  ret ptr %48
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaF_freeproto(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Proto, ptr %6, i32 0, i32 16
  %8 = load ptr, ptr %7, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 8
  %11 = load i32, ptr %10, align 8
  %12 = sext i32 %11 to i64
  %13 = mul i64 %12, 4
  call void @luaM_free_(ptr noundef %5, ptr noundef %8, i64 noundef %13)
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.Proto, ptr %15, i32 0, i32 17
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 10
  %20 = load i32, ptr %19, align 8
  %21 = sext i32 %20 to i64
  %22 = mul i64 %21, 8
  call void @luaM_free_(ptr noundef %14, ptr noundef %17, i64 noundef %22)
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.Proto, ptr %24, i32 0, i32 15
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.Proto, ptr %27, i32 0, i32 7
  %29 = load i32, ptr %28, align 4
  %30 = sext i32 %29 to i64
  %31 = mul i64 %30, 16
  call void @luaM_free_(ptr noundef %23, ptr noundef %26, i64 noundef %31)
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.Proto, ptr %33, i32 0, i32 19
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 9
  %38 = load i32, ptr %37, align 4
  %39 = sext i32 %38 to i64
  %40 = mul i64 %39, 1
  call void @luaM_free_(ptr noundef %32, ptr noundef %35, i64 noundef %40)
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.Proto, ptr %42, i32 0, i32 20
  %44 = load ptr, ptr %43, align 8
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.Proto, ptr %45, i32 0, i32 12
  %47 = load i32, ptr %46, align 8
  %48 = sext i32 %47 to i64
  %49 = mul i64 %48, 8
  call void @luaM_free_(ptr noundef %41, ptr noundef %44, i64 noundef %49)
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.Proto, ptr %51, i32 0, i32 21
  %53 = load ptr, ptr %52, align 8
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 11
  %56 = load i32, ptr %55, align 4
  %57 = sext i32 %56 to i64
  %58 = mul i64 %57, 16
  call void @luaM_free_(ptr noundef %50, ptr noundef %53, i64 noundef %58)
  %59 = load ptr, ptr %3, align 8
  %60 = load ptr, ptr %4, align 8
  %61 = getelementptr inbounds %struct.Proto, ptr %60, i32 0, i32 18
  %62 = load ptr, ptr %61, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.Proto, ptr %63, i32 0, i32 6
  %65 = load i32, ptr %64, align 8
  %66 = sext i32 %65 to i64
  %67 = mul i64 %66, 16
  call void @luaM_free_(ptr noundef %59, ptr noundef %62, i64 noundef %67)
  %68 = load ptr, ptr %3, align 8
  %69 = load ptr, ptr %4, align 8
  call void @luaM_free_(ptr noundef %68, ptr noundef %69, i64 noundef 128)
  ret void
}

declare hidden void @luaM_free_(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaF_getlocalname(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 0, ptr %8, align 4
  br label %9

9:                                                ; preds = %57, %3
  %10 = load i32, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.Proto, ptr %11, i32 0, i32 11
  %13 = load i32, ptr %12, align 4
  %14 = icmp slt i32 %10, %13
  br i1 %14, label %15, label %26

15:                                               ; preds = %9
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.Proto, ptr %16, i32 0, i32 21
  %18 = load ptr, ptr %17, align 8
  %19 = load i32, ptr %8, align 4
  %20 = sext i32 %19 to i64
  %21 = getelementptr inbounds %struct.LocVar, ptr %18, i64 %20
  %22 = getelementptr inbounds %struct.LocVar, ptr %21, i32 0, i32 1
  %23 = load i32, ptr %22, align 8
  %24 = load i32, ptr %7, align 4
  %25 = icmp sle i32 %23, %24
  br label %26

26:                                               ; preds = %15, %9
  %27 = phi i1 [ false, %9 ], [ %25, %15 ]
  br i1 %27, label %28, label %60

28:                                               ; preds = %26
  %29 = load i32, ptr %7, align 4
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.Proto, ptr %30, i32 0, i32 21
  %32 = load ptr, ptr %31, align 8
  %33 = load i32, ptr %8, align 4
  %34 = sext i32 %33 to i64
  %35 = getelementptr inbounds %struct.LocVar, ptr %32, i64 %34
  %36 = getelementptr inbounds %struct.LocVar, ptr %35, i32 0, i32 2
  %37 = load i32, ptr %36, align 4
  %38 = icmp slt i32 %29, %37
  br i1 %38, label %39, label %56

39:                                               ; preds = %28
  %40 = load i32, ptr %6, align 4
  %41 = add nsw i32 %40, -1
  store i32 %41, ptr %6, align 4
  %42 = load i32, ptr %6, align 4
  %43 = icmp eq i32 %42, 0
  br i1 %43, label %44, label %55

44:                                               ; preds = %39
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.Proto, ptr %45, i32 0, i32 21
  %47 = load ptr, ptr %46, align 8
  %48 = load i32, ptr %8, align 4
  %49 = sext i32 %48 to i64
  %50 = getelementptr inbounds %struct.LocVar, ptr %47, i64 %49
  %51 = getelementptr inbounds %struct.LocVar, ptr %50, i32 0, i32 0
  %52 = load ptr, ptr %51, align 8
  %53 = getelementptr inbounds %struct.TString, ptr %52, i32 0, i32 7
  %54 = getelementptr inbounds [1 x i8], ptr %53, i64 0, i64 0
  store ptr %54, ptr %4, align 8
  br label %61

55:                                               ; preds = %39
  br label %56

56:                                               ; preds = %55, %28
  br label %57

57:                                               ; preds = %56
  %58 = load i32, ptr %8, align 4
  %59 = add nsw i32 %58, 1
  store i32 %59, ptr %8, align 4
  br label %9, !llvm.loop !14

60:                                               ; preds = %26
  store ptr null, ptr %4, align 8
  br label %61

61:                                               ; preds = %60, %44
  %62 = load ptr, ptr %4, align 8
  ret ptr %62
}

declare hidden ptr @luaT_gettmbyobj(ptr noundef, ptr noundef, i32 noundef) #1

declare hidden ptr @luaG_findlocal(ptr noundef, ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #3

declare hidden void @luaD_seterrorobj(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @callclosemethod(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 6
  %19 = load ptr, ptr %18, align 8
  store ptr %19, ptr %9, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = call ptr @luaT_gettmbyobj(ptr noundef %20, ptr noundef %21, i32 noundef 24)
  store ptr %22, ptr %10, align 8
  %23 = load ptr, ptr %9, align 8
  store ptr %23, ptr %11, align 8
  %24 = load ptr, ptr %10, align 8
  store ptr %24, ptr %12, align 8
  %25 = load ptr, ptr %11, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %12, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %26, ptr align 8 %28, i64 8, i1 false)
  %29 = load ptr, ptr %12, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = load ptr, ptr %11, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  store i8 %31, ptr %33, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i64 1
  store ptr %36, ptr %13, align 8
  %37 = load ptr, ptr %6, align 8
  store ptr %37, ptr %14, align 8
  %38 = load ptr, ptr %13, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  %40 = load ptr, ptr %14, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %39, ptr align 8 %41, i64 8, i1 false)
  %42 = load ptr, ptr %14, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 1
  %44 = load i8, ptr %43, align 8
  %45 = load ptr, ptr %13, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 1
  store i8 %44, ptr %46, align 8
  %47 = load ptr, ptr %5, align 8
  %48 = load ptr, ptr %9, align 8
  %49 = getelementptr inbounds %union.StackValue, ptr %48, i64 2
  store ptr %49, ptr %15, align 8
  %50 = load ptr, ptr %7, align 8
  store ptr %50, ptr %16, align 8
  %51 = load ptr, ptr %15, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %16, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %52, ptr align 8 %54, i64 8, i1 false)
  %55 = load ptr, ptr %16, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 8
  %58 = load ptr, ptr %15, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 1
  store i8 %57, ptr %59, align 8
  %60 = load ptr, ptr %5, align 8
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %union.StackValue, ptr %61, i64 3
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.lua_State, ptr %63, i32 0, i32 6
  store ptr %62, ptr %64, align 8
  %65 = load i32, ptr %8, align 4
  %66 = icmp ne i32 %65, 0
  br i1 %66, label %67, label %70

67:                                               ; preds = %4
  %68 = load ptr, ptr %5, align 8
  %69 = load ptr, ptr %9, align 8
  call void @luaD_call(ptr noundef %68, ptr noundef %69, i32 noundef 0)
  br label %73

70:                                               ; preds = %4
  %71 = load ptr, ptr %5, align 8
  %72 = load ptr, ptr %9, align 8
  call void @luaD_callnoyield(ptr noundef %71, ptr noundef %72, i32 noundef 0)
  br label %73

73:                                               ; preds = %70, %67
  ret void
}

declare hidden void @luaD_call(ptr noundef, ptr noundef, i32 noundef) #1

declare hidden void @luaD_callnoyield(ptr noundef, ptr noundef, i32 noundef) #1

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
!11 = distinct !{!11, !7}
!12 = distinct !{!12, !7}
!13 = distinct !{!13, !7}
!14 = distinct !{!14, !7}
