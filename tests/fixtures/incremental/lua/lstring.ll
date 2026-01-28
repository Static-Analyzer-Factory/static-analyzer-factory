; ModuleID = 'lstring.c'
source_filename = "lstring.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon.0, %union.anon.2, i16, i16 }
%union.anon.0 = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.Udata = type { ptr, i8, i8, i16, i64, ptr, ptr, [1 x %union.UValue] }
%union.UValue = type { %struct.TValue }

@.str = private unnamed_addr constant [18 x i8] c"not enough memory\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaS_eqlngstr(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.TString, ptr %6, i32 0, i32 6
  %8 = load i64, ptr %7, align 8
  store i64 %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = icmp eq ptr %9, %10
  br i1 %11, label %30, label %12

12:                                               ; preds = %2
  %13 = load i64, ptr %5, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.TString, ptr %14, i32 0, i32 6
  %16 = load i64, ptr %15, align 8
  %17 = icmp eq i64 %13, %16
  br i1 %17, label %18, label %28

18:                                               ; preds = %12
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.TString, ptr %19, i32 0, i32 7
  %21 = getelementptr inbounds [1 x i8], ptr %20, i64 0, i64 0
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.TString, ptr %22, i32 0, i32 7
  %24 = getelementptr inbounds [1 x i8], ptr %23, i64 0, i64 0
  %25 = load i64, ptr %5, align 8
  %26 = call i32 @memcmp(ptr noundef %21, ptr noundef %24, i64 noundef %25) #5
  %27 = icmp eq i32 %26, 0
  br label %28

28:                                               ; preds = %18, %12
  %29 = phi i1 [ false, %12 ], [ %27, %18 ]
  br label %30

30:                                               ; preds = %28, %2
  %31 = phi i1 [ true, %2 ], [ %29, %28 ]
  %32 = zext i1 %31 to i32
  ret i32 %32
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @memcmp(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaS_hash(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load i32, ptr %6, align 4
  %9 = load i64, ptr %5, align 8
  %10 = trunc i64 %9 to i32
  %11 = xor i32 %8, %10
  store i32 %11, ptr %7, align 4
  br label %12

12:                                               ; preds = %30, %3
  %13 = load i64, ptr %5, align 8
  %14 = icmp ugt i64 %13, 0
  br i1 %14, label %15, label %33

15:                                               ; preds = %12
  %16 = load i32, ptr %7, align 4
  %17 = shl i32 %16, 5
  %18 = load i32, ptr %7, align 4
  %19 = lshr i32 %18, 2
  %20 = add i32 %17, %19
  %21 = load ptr, ptr %4, align 8
  %22 = load i64, ptr %5, align 8
  %23 = sub i64 %22, 1
  %24 = getelementptr inbounds i8, ptr %21, i64 %23
  %25 = load i8, ptr %24, align 1
  %26 = zext i8 %25 to i32
  %27 = add i32 %20, %26
  %28 = load i32, ptr %7, align 4
  %29 = xor i32 %28, %27
  store i32 %29, ptr %7, align 4
  br label %30

30:                                               ; preds = %15
  %31 = load i64, ptr %5, align 8
  %32 = add i64 %31, -1
  store i64 %32, ptr %5, align 8
  br label %12, !llvm.loop !6

33:                                               ; preds = %12
  %34 = load i32, ptr %7, align 4
  ret i32 %34
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaS_hashlongstr(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.TString, ptr %4, i32 0, i32 3
  %6 = load i8, ptr %5, align 2
  %7 = zext i8 %6 to i32
  %8 = icmp eq i32 %7, 0
  br i1 %8, label %9, label %25

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.TString, ptr %10, i32 0, i32 6
  %12 = load i64, ptr %11, align 8
  store i64 %12, ptr %3, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.TString, ptr %13, i32 0, i32 7
  %15 = getelementptr inbounds [1 x i8], ptr %14, i64 0, i64 0
  %16 = load i64, ptr %3, align 8
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.TString, ptr %17, i32 0, i32 5
  %19 = load i32, ptr %18, align 4
  %20 = call i32 @luaS_hash(ptr noundef %15, i64 noundef %16, i32 noundef %19)
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.TString, ptr %21, i32 0, i32 5
  store i32 %20, ptr %22, align 4
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 3
  store i8 1, ptr %24, align 2
  br label %25

25:                                               ; preds = %9, %1
  %26 = load ptr, ptr %2, align 8
  %27 = getelementptr inbounds %struct.TString, ptr %26, i32 0, i32 5
  %28 = load i32, ptr %27, align 4
  ret i32 %28
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaS_resize(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 6
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.stringtable, ptr %12, i32 0, i32 2
  %14 = load i32, ptr %13, align 4
  store i32 %14, ptr %6, align 4
  %15 = load i32, ptr %4, align 4
  %16 = load i32, ptr %6, align 4
  %17 = icmp slt i32 %15, %16
  br i1 %17, label %18, label %24

18:                                               ; preds = %2
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.stringtable, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  %22 = load i32, ptr %6, align 4
  %23 = load i32, ptr %4, align 4
  call void @tablerehash(ptr noundef %21, i32 noundef %22, i32 noundef %23)
  br label %24

24:                                               ; preds = %18, %2
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.stringtable, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %6, align 4
  %30 = sext i32 %29 to i64
  %31 = mul i64 %30, 8
  %32 = load i32, ptr %4, align 4
  %33 = sext i32 %32 to i64
  %34 = mul i64 %33, 8
  %35 = call ptr @luaM_realloc_(ptr noundef %25, ptr noundef %28, i64 noundef %31, i64 noundef %34)
  store ptr %35, ptr %7, align 8
  %36 = load ptr, ptr %7, align 8
  %37 = icmp eq ptr %36, null
  %38 = zext i1 %37 to i32
  %39 = icmp ne i32 %38, 0
  %40 = zext i1 %39 to i32
  %41 = sext i32 %40 to i64
  %42 = icmp ne i64 %41, 0
  br i1 %42, label %43, label %54

43:                                               ; preds = %24
  %44 = load i32, ptr %4, align 4
  %45 = load i32, ptr %6, align 4
  %46 = icmp slt i32 %44, %45
  br i1 %46, label %47, label %53

47:                                               ; preds = %43
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.stringtable, ptr %48, i32 0, i32 0
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %4, align 4
  %52 = load i32, ptr %6, align 4
  call void @tablerehash(ptr noundef %50, i32 noundef %51, i32 noundef %52)
  br label %53

53:                                               ; preds = %47, %43
  br label %69

54:                                               ; preds = %24
  %55 = load ptr, ptr %7, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.stringtable, ptr %56, i32 0, i32 0
  store ptr %55, ptr %57, align 8
  %58 = load i32, ptr %4, align 4
  %59 = load ptr, ptr %5, align 8
  %60 = getelementptr inbounds %struct.stringtable, ptr %59, i32 0, i32 2
  store i32 %58, ptr %60, align 4
  %61 = load i32, ptr %4, align 4
  %62 = load i32, ptr %6, align 4
  %63 = icmp sgt i32 %61, %62
  br i1 %63, label %64, label %68

64:                                               ; preds = %54
  %65 = load ptr, ptr %7, align 8
  %66 = load i32, ptr %6, align 4
  %67 = load i32, ptr %4, align 4
  call void @tablerehash(ptr noundef %65, i32 noundef %66, i32 noundef %67)
  br label %68

68:                                               ; preds = %64, %54
  br label %69

69:                                               ; preds = %68, %53
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @tablerehash(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %11 = load i32, ptr %5, align 4
  store i32 %11, ptr %7, align 4
  br label %12

12:                                               ; preds = %21, %3
  %13 = load i32, ptr %7, align 4
  %14 = load i32, ptr %6, align 4
  %15 = icmp slt i32 %13, %14
  br i1 %15, label %16, label %24

16:                                               ; preds = %12
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %7, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds ptr, ptr %17, i64 %19
  store ptr null, ptr %20, align 8
  br label %21

21:                                               ; preds = %16
  %22 = load i32, ptr %7, align 4
  %23 = add nsw i32 %22, 1
  store i32 %23, ptr %7, align 4
  br label %12, !llvm.loop !8

24:                                               ; preds = %12
  store i32 0, ptr %7, align 4
  br label %25

25:                                               ; preds = %66, %24
  %26 = load i32, ptr %7, align 4
  %27 = load i32, ptr %5, align 4
  %28 = icmp slt i32 %26, %27
  br i1 %28, label %29, label %69

29:                                               ; preds = %25
  %30 = load ptr, ptr %4, align 8
  %31 = load i32, ptr %7, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds ptr, ptr %30, i64 %32
  %34 = load ptr, ptr %33, align 8
  store ptr %34, ptr %8, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = load i32, ptr %7, align 4
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds ptr, ptr %35, i64 %37
  store ptr null, ptr %38, align 8
  br label %39

39:                                               ; preds = %42, %29
  %40 = load ptr, ptr %8, align 8
  %41 = icmp ne ptr %40, null
  br i1 %41, label %42, label %65

42:                                               ; preds = %39
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds %struct.TString, ptr %43, i32 0, i32 6
  %45 = load ptr, ptr %44, align 8
  store ptr %45, ptr %9, align 8
  %46 = load ptr, ptr %8, align 8
  %47 = getelementptr inbounds %struct.TString, ptr %46, i32 0, i32 5
  %48 = load i32, ptr %47, align 4
  %49 = load i32, ptr %6, align 4
  %50 = sub nsw i32 %49, 1
  %51 = and i32 %48, %50
  store i32 %51, ptr %10, align 4
  %52 = load ptr, ptr %4, align 8
  %53 = load i32, ptr %10, align 4
  %54 = zext i32 %53 to i64
  %55 = getelementptr inbounds ptr, ptr %52, i64 %54
  %56 = load ptr, ptr %55, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = getelementptr inbounds %struct.TString, ptr %57, i32 0, i32 6
  store ptr %56, ptr %58, align 8
  %59 = load ptr, ptr %8, align 8
  %60 = load ptr, ptr %4, align 8
  %61 = load i32, ptr %10, align 4
  %62 = zext i32 %61 to i64
  %63 = getelementptr inbounds ptr, ptr %60, i64 %62
  store ptr %59, ptr %63, align 8
  %64 = load ptr, ptr %9, align 8
  store ptr %64, ptr %8, align 8
  br label %39, !llvm.loop !9

65:                                               ; preds = %39
  br label %66

66:                                               ; preds = %65
  %67 = load i32, ptr %7, align 4
  %68 = add nsw i32 %67, 1
  store i32 %68, ptr %7, align 4
  br label %25, !llvm.loop !10

69:                                               ; preds = %25
  ret void
}

declare hidden ptr @luaM_realloc_(ptr noundef, ptr noundef, i64 noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaS_clearcache(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 0, ptr %3, align 4
  br label %5

5:                                                ; preds = %44, %1
  %6 = load i32, ptr %3, align 4
  %7 = icmp slt i32 %6, 53
  br i1 %7, label %8, label %47

8:                                                ; preds = %5
  store i32 0, ptr %4, align 4
  br label %9

9:                                                ; preds = %40, %8
  %10 = load i32, ptr %4, align 4
  %11 = icmp slt i32 %10, 2
  br i1 %11, label %12, label %43

12:                                               ; preds = %9
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 44
  %15 = load i32, ptr %3, align 4
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds [53 x [2 x ptr]], ptr %14, i64 0, i64 %16
  %18 = load i32, ptr %4, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds [2 x ptr], ptr %17, i64 0, i64 %19
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.TString, ptr %21, i32 0, i32 2
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = and i32 %24, 24
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %39

27:                                               ; preds = %12
  %28 = load ptr, ptr %2, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 41
  %30 = load ptr, ptr %29, align 8
  %31 = load ptr, ptr %2, align 8
  %32 = getelementptr inbounds %struct.global_State, ptr %31, i32 0, i32 44
  %33 = load i32, ptr %3, align 4
  %34 = sext i32 %33 to i64
  %35 = getelementptr inbounds [53 x [2 x ptr]], ptr %32, i64 0, i64 %34
  %36 = load i32, ptr %4, align 4
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds [2 x ptr], ptr %35, i64 0, i64 %37
  store ptr %30, ptr %38, align 8
  br label %39

39:                                               ; preds = %27, %12
  br label %40

40:                                               ; preds = %39
  %41 = load i32, ptr %4, align 4
  %42 = add nsw i32 %41, 1
  store i32 %42, ptr %4, align 4
  br label %9, !llvm.loop !11

43:                                               ; preds = %9
  br label %44

44:                                               ; preds = %43
  %45 = load i32, ptr %3, align 4
  %46 = add nsw i32 %45, 1
  store i32 %46, ptr %3, align 4
  br label %5, !llvm.loop !12

47:                                               ; preds = %5
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaS_init(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 7
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 7
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 6
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = call ptr @luaM_malloc_(ptr noundef %14, i64 noundef 1024, i32 noundef 0)
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.stringtable, ptr %16, i32 0, i32 0
  store ptr %15, ptr %17, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.stringtable, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  call void @tablerehash(ptr noundef %20, i32 noundef 0, i32 noundef 128)
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.stringtable, ptr %21, i32 0, i32 2
  store i32 128, ptr %22, align 4
  %23 = load ptr, ptr %2, align 8
  %24 = call ptr @luaS_newlstr(ptr noundef %23, ptr noundef @.str, i64 noundef 17)
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 41
  store ptr %24, ptr %26, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = load ptr, ptr %3, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 41
  %30 = load ptr, ptr %29, align 8
  call void @luaC_fix(ptr noundef %27, ptr noundef %30)
  store i32 0, ptr %4, align 4
  br label %31

31:                                               ; preds = %54, %1
  %32 = load i32, ptr %4, align 4
  %33 = icmp slt i32 %32, 53
  br i1 %33, label %34, label %57

34:                                               ; preds = %31
  store i32 0, ptr %5, align 4
  br label %35

35:                                               ; preds = %50, %34
  %36 = load i32, ptr %5, align 4
  %37 = icmp slt i32 %36, 2
  br i1 %37, label %38, label %53

38:                                               ; preds = %35
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 41
  %41 = load ptr, ptr %40, align 8
  %42 = load ptr, ptr %3, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 44
  %44 = load i32, ptr %4, align 4
  %45 = sext i32 %44 to i64
  %46 = getelementptr inbounds [53 x [2 x ptr]], ptr %43, i64 0, i64 %45
  %47 = load i32, ptr %5, align 4
  %48 = sext i32 %47 to i64
  %49 = getelementptr inbounds [2 x ptr], ptr %46, i64 0, i64 %48
  store ptr %41, ptr %49, align 8
  br label %50

50:                                               ; preds = %38
  %51 = load i32, ptr %5, align 4
  %52 = add nsw i32 %51, 1
  store i32 %52, ptr %5, align 4
  br label %35, !llvm.loop !13

53:                                               ; preds = %35
  br label %54

54:                                               ; preds = %53
  %55 = load i32, ptr %4, align 4
  %56 = add nsw i32 %55, 1
  store i32 %56, ptr %4, align 4
  br label %31, !llvm.loop !14

57:                                               ; preds = %31
  ret void
}

declare hidden ptr @luaM_malloc_(ptr noundef, i64 noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaS_newlstr(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  %9 = load i64, ptr %7, align 8
  %10 = icmp ule i64 %9, 40
  br i1 %10, label %11, label %16

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load i64, ptr %7, align 8
  %15 = call ptr @internshrstr(ptr noundef %12, ptr noundef %13, i64 noundef %14)
  store ptr %15, ptr %4, align 8
  br label %38

16:                                               ; preds = %3
  %17 = load i64, ptr %7, align 8
  %18 = mul i64 %17, 1
  %19 = icmp uge i64 %18, 9223372036854775775
  %20 = zext i1 %19 to i32
  %21 = icmp ne i32 %20, 0
  %22 = zext i1 %21 to i32
  %23 = sext i32 %22 to i64
  %24 = icmp ne i64 %23, 0
  br i1 %24, label %25, label %27

25:                                               ; preds = %16
  %26 = load ptr, ptr %5, align 8
  call void @luaM_toobig(ptr noundef %26) #6
  unreachable

27:                                               ; preds = %16
  %28 = load ptr, ptr %5, align 8
  %29 = load i64, ptr %7, align 8
  %30 = call ptr @luaS_createlngstrobj(ptr noundef %28, i64 noundef %29)
  store ptr %30, ptr %8, align 8
  %31 = load ptr, ptr %8, align 8
  %32 = getelementptr inbounds %struct.TString, ptr %31, i32 0, i32 7
  %33 = getelementptr inbounds [1 x i8], ptr %32, i64 0, i64 0
  %34 = load ptr, ptr %6, align 8
  %35 = load i64, ptr %7, align 8
  %36 = mul i64 %35, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %33, ptr align 1 %34, i64 %36, i1 false)
  %37 = load ptr, ptr %8, align 8
  store ptr %37, ptr %4, align 8
  br label %38

38:                                               ; preds = %27, %11
  %39 = load ptr, ptr %4, align 8
  ret ptr %39
}

declare hidden void @luaC_fix(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaS_createlngstrobj(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load i64, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 9
  %12 = load i32, ptr %11, align 8
  %13 = call ptr @createstrobj(ptr noundef %6, i64 noundef %7, i32 noundef 20, i32 noundef %12)
  store ptr %13, ptr %5, align 8
  %14 = load i64, ptr %4, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.TString, ptr %15, i32 0, i32 6
  store i64 %14, ptr %16, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.TString, ptr %17, i32 0, i32 4
  store i8 -1, ptr %18, align 1
  %19 = load ptr, ptr %5, align 8
  ret ptr %19
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @createstrobj(ptr noundef %0, i64 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %12 = load i64, ptr %6, align 8
  %13 = add i64 %12, 1
  %14 = mul i64 %13, 1
  %15 = add i64 24, %14
  store i64 %15, ptr %11, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = load i32, ptr %7, align 4
  %18 = load i64, ptr %11, align 8
  %19 = call ptr @luaC_newobj(ptr noundef %16, i32 noundef %17, i64 noundef %18)
  store ptr %19, ptr %10, align 8
  %20 = load ptr, ptr %10, align 8
  store ptr %20, ptr %9, align 8
  %21 = load i32, ptr %8, align 4
  %22 = load ptr, ptr %9, align 8
  %23 = getelementptr inbounds %struct.TString, ptr %22, i32 0, i32 5
  store i32 %21, ptr %23, align 4
  %24 = load ptr, ptr %9, align 8
  %25 = getelementptr inbounds %struct.TString, ptr %24, i32 0, i32 3
  store i8 0, ptr %25, align 2
  %26 = load ptr, ptr %9, align 8
  %27 = getelementptr inbounds %struct.TString, ptr %26, i32 0, i32 7
  %28 = load i64, ptr %6, align 8
  %29 = getelementptr inbounds [1 x i8], ptr %27, i64 0, i64 %28
  store i8 0, ptr %29, align 1
  %30 = load ptr, ptr %9, align 8
  ret ptr %30
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaS_remove(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 7
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 6
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.stringtable, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.TString, ptr %14, i32 0, i32 5
  %16 = load i32, ptr %15, align 4
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.stringtable, ptr %17, i32 0, i32 2
  %19 = load i32, ptr %18, align 4
  %20 = sub nsw i32 %19, 1
  %21 = and i32 %16, %20
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds ptr, ptr %13, i64 %22
  store ptr %23, ptr %6, align 8
  br label %24

24:                                               ; preds = %29, %2
  %25 = load ptr, ptr %6, align 8
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = icmp ne ptr %26, %27
  br i1 %28, label %29, label %33

29:                                               ; preds = %24
  %30 = load ptr, ptr %6, align 8
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.TString, ptr %31, i32 0, i32 6
  store ptr %32, ptr %6, align 8
  br label %24, !llvm.loop !15

33:                                               ; preds = %24
  %34 = load ptr, ptr %6, align 8
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.TString, ptr %35, i32 0, i32 6
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %6, align 8
  store ptr %37, ptr %38, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.stringtable, ptr %39, i32 0, i32 1
  %41 = load i32, ptr %40, align 8
  %42 = add nsw i32 %41, -1
  store i32 %42, ptr %40, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @internshrstr(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %9, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 6
  store ptr %17, ptr %10, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = load i64, ptr %7, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 9
  %22 = load i32, ptr %21, align 8
  %23 = call i32 @luaS_hash(ptr noundef %18, i64 noundef %19, i32 noundef %22)
  store i32 %23, ptr %11, align 4
  %24 = load ptr, ptr %10, align 8
  %25 = getelementptr inbounds %struct.stringtable, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = load i32, ptr %11, align 4
  %28 = load ptr, ptr %10, align 8
  %29 = getelementptr inbounds %struct.stringtable, ptr %28, i32 0, i32 2
  %30 = load i32, ptr %29, align 4
  %31 = sub nsw i32 %30, 1
  %32 = and i32 %27, %31
  %33 = sext i32 %32 to i64
  %34 = getelementptr inbounds ptr, ptr %26, i64 %33
  store ptr %34, ptr %12, align 8
  %35 = load ptr, ptr %12, align 8
  %36 = load ptr, ptr %35, align 8
  store ptr %36, ptr %8, align 8
  br label %37

37:                                               ; preds = %78, %3
  %38 = load ptr, ptr %8, align 8
  %39 = icmp ne ptr %38, null
  br i1 %39, label %40, label %82

40:                                               ; preds = %37
  %41 = load i64, ptr %7, align 8
  %42 = load ptr, ptr %8, align 8
  %43 = getelementptr inbounds %struct.TString, ptr %42, i32 0, i32 4
  %44 = load i8, ptr %43, align 1
  %45 = zext i8 %44 to i64
  %46 = icmp eq i64 %41, %45
  br i1 %46, label %47, label %77

47:                                               ; preds = %40
  %48 = load ptr, ptr %6, align 8
  %49 = load ptr, ptr %8, align 8
  %50 = getelementptr inbounds %struct.TString, ptr %49, i32 0, i32 7
  %51 = getelementptr inbounds [1 x i8], ptr %50, i64 0, i64 0
  %52 = load i64, ptr %7, align 8
  %53 = mul i64 %52, 1
  %54 = call i32 @memcmp(ptr noundef %48, ptr noundef %51, i64 noundef %53) #5
  %55 = icmp eq i32 %54, 0
  br i1 %55, label %56, label %77

56:                                               ; preds = %47
  %57 = load ptr, ptr %8, align 8
  %58 = getelementptr inbounds %struct.TString, ptr %57, i32 0, i32 2
  %59 = load i8, ptr %58, align 1
  %60 = zext i8 %59 to i32
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %struct.global_State, ptr %61, i32 0, i32 10
  %63 = load i8, ptr %62, align 4
  %64 = zext i8 %63 to i32
  %65 = xor i32 %64, 24
  %66 = and i32 %60, %65
  %67 = icmp ne i32 %66, 0
  br i1 %67, label %68, label %75

68:                                               ; preds = %56
  %69 = load ptr, ptr %8, align 8
  %70 = getelementptr inbounds %struct.TString, ptr %69, i32 0, i32 2
  %71 = load i8, ptr %70, align 1
  %72 = zext i8 %71 to i32
  %73 = xor i32 %72, 24
  %74 = trunc i32 %73 to i8
  store i8 %74, ptr %70, align 1
  br label %75

75:                                               ; preds = %68, %56
  %76 = load ptr, ptr %8, align 8
  store ptr %76, ptr %4, align 8
  br label %130

77:                                               ; preds = %47, %40
  br label %78

78:                                               ; preds = %77
  %79 = load ptr, ptr %8, align 8
  %80 = getelementptr inbounds %struct.TString, ptr %79, i32 0, i32 6
  %81 = load ptr, ptr %80, align 8
  store ptr %81, ptr %8, align 8
  br label %37, !llvm.loop !16

82:                                               ; preds = %37
  %83 = load ptr, ptr %10, align 8
  %84 = getelementptr inbounds %struct.stringtable, ptr %83, i32 0, i32 1
  %85 = load i32, ptr %84, align 8
  %86 = load ptr, ptr %10, align 8
  %87 = getelementptr inbounds %struct.stringtable, ptr %86, i32 0, i32 2
  %88 = load i32, ptr %87, align 4
  %89 = icmp sge i32 %85, %88
  br i1 %89, label %90, label %104

90:                                               ; preds = %82
  %91 = load ptr, ptr %5, align 8
  %92 = load ptr, ptr %10, align 8
  call void @growstrtab(ptr noundef %91, ptr noundef %92)
  %93 = load ptr, ptr %10, align 8
  %94 = getelementptr inbounds %struct.stringtable, ptr %93, i32 0, i32 0
  %95 = load ptr, ptr %94, align 8
  %96 = load i32, ptr %11, align 4
  %97 = load ptr, ptr %10, align 8
  %98 = getelementptr inbounds %struct.stringtable, ptr %97, i32 0, i32 2
  %99 = load i32, ptr %98, align 4
  %100 = sub nsw i32 %99, 1
  %101 = and i32 %96, %100
  %102 = sext i32 %101 to i64
  %103 = getelementptr inbounds ptr, ptr %95, i64 %102
  store ptr %103, ptr %12, align 8
  br label %104

104:                                              ; preds = %90, %82
  %105 = load ptr, ptr %5, align 8
  %106 = load i64, ptr %7, align 8
  %107 = load i32, ptr %11, align 4
  %108 = call ptr @createstrobj(ptr noundef %105, i64 noundef %106, i32 noundef 4, i32 noundef %107)
  store ptr %108, ptr %8, align 8
  %109 = load i64, ptr %7, align 8
  %110 = trunc i64 %109 to i8
  %111 = load ptr, ptr %8, align 8
  %112 = getelementptr inbounds %struct.TString, ptr %111, i32 0, i32 4
  store i8 %110, ptr %112, align 1
  %113 = load ptr, ptr %8, align 8
  %114 = getelementptr inbounds %struct.TString, ptr %113, i32 0, i32 7
  %115 = getelementptr inbounds [1 x i8], ptr %114, i64 0, i64 0
  %116 = load ptr, ptr %6, align 8
  %117 = load i64, ptr %7, align 8
  %118 = mul i64 %117, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %115, ptr align 1 %116, i64 %118, i1 false)
  %119 = load ptr, ptr %12, align 8
  %120 = load ptr, ptr %119, align 8
  %121 = load ptr, ptr %8, align 8
  %122 = getelementptr inbounds %struct.TString, ptr %121, i32 0, i32 6
  store ptr %120, ptr %122, align 8
  %123 = load ptr, ptr %8, align 8
  %124 = load ptr, ptr %12, align 8
  store ptr %123, ptr %124, align 8
  %125 = load ptr, ptr %10, align 8
  %126 = getelementptr inbounds %struct.stringtable, ptr %125, i32 0, i32 1
  %127 = load i32, ptr %126, align 8
  %128 = add nsw i32 %127, 1
  store i32 %128, ptr %126, align 8
  %129 = load ptr, ptr %8, align 8
  store ptr %129, ptr %4, align 8
  br label %130

130:                                              ; preds = %104, %75
  %131 = load ptr, ptr %4, align 8
  ret ptr %131
}

; Function Attrs: noreturn
declare hidden void @luaM_toobig(ptr noundef) #3

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #4

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaS_new(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = ptrtoint ptr %9 to i64
  %11 = and i64 %10, 4294967295
  %12 = trunc i64 %11 to i32
  %13 = urem i32 %12, 53
  store i32 %13, ptr %6, align 4
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 7
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 44
  %18 = load i32, ptr %6, align 4
  %19 = zext i32 %18 to i64
  %20 = getelementptr inbounds [53 x [2 x ptr]], ptr %17, i64 0, i64 %19
  %21 = getelementptr inbounds [2 x ptr], ptr %20, i64 0, i64 0
  store ptr %21, ptr %8, align 8
  store i32 0, ptr %7, align 4
  br label %22

22:                                               ; preds = %43, %2
  %23 = load i32, ptr %7, align 4
  %24 = icmp slt i32 %23, 2
  br i1 %24, label %25, label %46

25:                                               ; preds = %22
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = load i32, ptr %7, align 4
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds ptr, ptr %27, i64 %29
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.TString, ptr %31, i32 0, i32 7
  %33 = getelementptr inbounds [1 x i8], ptr %32, i64 0, i64 0
  %34 = call i32 @strcmp(ptr noundef %26, ptr noundef %33) #5
  %35 = icmp eq i32 %34, 0
  br i1 %35, label %36, label %42

36:                                               ; preds = %25
  %37 = load ptr, ptr %8, align 8
  %38 = load i32, ptr %7, align 4
  %39 = sext i32 %38 to i64
  %40 = getelementptr inbounds ptr, ptr %37, i64 %39
  %41 = load ptr, ptr %40, align 8
  store ptr %41, ptr %3, align 8
  br label %75

42:                                               ; preds = %25
  br label %43

43:                                               ; preds = %42
  %44 = load i32, ptr %7, align 4
  %45 = add nsw i32 %44, 1
  store i32 %45, ptr %7, align 4
  br label %22, !llvm.loop !17

46:                                               ; preds = %22
  store i32 1, ptr %7, align 4
  br label %47

47:                                               ; preds = %61, %46
  %48 = load i32, ptr %7, align 4
  %49 = icmp sgt i32 %48, 0
  br i1 %49, label %50, label %64

50:                                               ; preds = %47
  %51 = load ptr, ptr %8, align 8
  %52 = load i32, ptr %7, align 4
  %53 = sub nsw i32 %52, 1
  %54 = sext i32 %53 to i64
  %55 = getelementptr inbounds ptr, ptr %51, i64 %54
  %56 = load ptr, ptr %55, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = load i32, ptr %7, align 4
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds ptr, ptr %57, i64 %59
  store ptr %56, ptr %60, align 8
  br label %61

61:                                               ; preds = %50
  %62 = load i32, ptr %7, align 4
  %63 = add nsw i32 %62, -1
  store i32 %63, ptr %7, align 4
  br label %47, !llvm.loop !18

64:                                               ; preds = %47
  %65 = load ptr, ptr %4, align 8
  %66 = load ptr, ptr %5, align 8
  %67 = load ptr, ptr %5, align 8
  %68 = call i64 @strlen(ptr noundef %67) #5
  %69 = call ptr @luaS_newlstr(ptr noundef %65, ptr noundef %66, i64 noundef %68)
  %70 = load ptr, ptr %8, align 8
  %71 = getelementptr inbounds ptr, ptr %70, i64 0
  store ptr %69, ptr %71, align 8
  %72 = load ptr, ptr %8, align 8
  %73 = getelementptr inbounds ptr, ptr %72, i64 0
  %74 = load ptr, ptr %73, align 8
  store ptr %74, ptr %3, align 8
  br label %75

75:                                               ; preds = %64, %36
  %76 = load ptr, ptr %3, align 8
  ret ptr %76
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaS_newudata(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load i64, ptr %5, align 8
  %11 = load i32, ptr %6, align 4
  %12 = icmp eq i32 %11, 0
  br i1 %12, label %13, label %14

13:                                               ; preds = %3
  br label %19

14:                                               ; preds = %3
  %15 = load i32, ptr %6, align 4
  %16 = sext i32 %15 to i64
  %17 = mul i64 16, %16
  %18 = add i64 40, %17
  br label %19

19:                                               ; preds = %14, %13
  %20 = phi i64 [ 32, %13 ], [ %18, %14 ]
  %21 = sub i64 9223372036854775807, %20
  %22 = icmp ugt i64 %10, %21
  %23 = zext i1 %22 to i32
  %24 = icmp ne i32 %23, 0
  %25 = zext i1 %24 to i32
  %26 = sext i32 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %28, label %30

28:                                               ; preds = %19
  %29 = load ptr, ptr %4, align 8
  call void @luaM_toobig(ptr noundef %29) #6
  unreachable

30:                                               ; preds = %19
  %31 = load ptr, ptr %4, align 8
  %32 = load i32, ptr %6, align 4
  %33 = icmp eq i32 %32, 0
  br i1 %33, label %34, label %35

34:                                               ; preds = %30
  br label %40

35:                                               ; preds = %30
  %36 = load i32, ptr %6, align 4
  %37 = sext i32 %36 to i64
  %38 = mul i64 16, %37
  %39 = add i64 40, %38
  br label %40

40:                                               ; preds = %35, %34
  %41 = phi i64 [ 32, %34 ], [ %39, %35 ]
  %42 = load i64, ptr %5, align 8
  %43 = add i64 %41, %42
  %44 = call ptr @luaC_newobj(ptr noundef %31, i32 noundef 7, i64 noundef %43)
  store ptr %44, ptr %9, align 8
  %45 = load ptr, ptr %9, align 8
  store ptr %45, ptr %7, align 8
  %46 = load i64, ptr %5, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.Udata, ptr %47, i32 0, i32 4
  store i64 %46, ptr %48, align 8
  %49 = load i32, ptr %6, align 4
  %50 = trunc i32 %49 to i16
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.Udata, ptr %51, i32 0, i32 3
  store i16 %50, ptr %52, align 2
  %53 = load ptr, ptr %7, align 8
  %54 = getelementptr inbounds %struct.Udata, ptr %53, i32 0, i32 5
  store ptr null, ptr %54, align 8
  store i32 0, ptr %8, align 4
  br label %55

55:                                               ; preds = %66, %40
  %56 = load i32, ptr %8, align 4
  %57 = load i32, ptr %6, align 4
  %58 = icmp slt i32 %56, %57
  br i1 %58, label %59, label %69

59:                                               ; preds = %55
  %60 = load ptr, ptr %7, align 8
  %61 = getelementptr inbounds %struct.Udata, ptr %60, i32 0, i32 7
  %62 = load i32, ptr %8, align 4
  %63 = sext i32 %62 to i64
  %64 = getelementptr inbounds [1 x %union.UValue], ptr %61, i64 0, i64 %63
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 1
  store i8 0, ptr %65, align 8
  br label %66

66:                                               ; preds = %59
  %67 = load i32, ptr %8, align 4
  %68 = add nsw i32 %67, 1
  store i32 %68, ptr %8, align 4
  br label %55, !llvm.loop !19

69:                                               ; preds = %55
  %70 = load ptr, ptr %7, align 8
  ret ptr %70
}

declare hidden ptr @luaC_newobj(ptr noundef, i32 noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @growstrtab(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.stringtable, ptr %5, i32 0, i32 1
  %7 = load i32, ptr %6, align 8
  %8 = icmp eq i32 %7, 2147483647
  %9 = zext i1 %8 to i32
  %10 = icmp ne i32 %9, 0
  %11 = zext i1 %10 to i32
  %12 = sext i32 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %14, label %23

14:                                               ; preds = %2
  %15 = load ptr, ptr %3, align 8
  call void @luaC_fullgc(ptr noundef %15, i32 noundef 1)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.stringtable, ptr %16, i32 0, i32 1
  %18 = load i32, ptr %17, align 8
  %19 = icmp eq i32 %18, 2147483647
  br i1 %19, label %20, label %22

20:                                               ; preds = %14
  %21 = load ptr, ptr %3, align 8
  call void @luaD_throw(ptr noundef %21, i32 noundef 4) #6
  unreachable

22:                                               ; preds = %14
  br label %23

23:                                               ; preds = %22, %2
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.stringtable, ptr %24, i32 0, i32 2
  %26 = load i32, ptr %25, align 4
  %27 = icmp sle i32 %26, 1073741823
  br i1 %27, label %28, label %34

28:                                               ; preds = %23
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.stringtable, ptr %30, i32 0, i32 2
  %32 = load i32, ptr %31, align 4
  %33 = mul nsw i32 %32, 2
  call void @luaS_resize(ptr noundef %29, i32 noundef %33)
  br label %34

34:                                               ; preds = %28, %23
  ret void
}

declare hidden void @luaC_fullgc(ptr noundef, i32 noundef) #2

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind willreturn memory(read) }
attributes #6 = { noreturn }

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
