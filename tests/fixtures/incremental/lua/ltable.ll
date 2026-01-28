; ModuleID = 'ltable.c'
source_filename = "ltable.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%union.Node = type { %struct.NodeKey }
%struct.NodeKey = type { %union.Value, i8, i8, i32, %union.Value }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }
%union.StackValue = type { %struct.TValue }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.GCObject = type { ptr, i8, i8 }

@absentkey = internal constant %struct.TValue { %union.Value zeroinitializer, i8 32 }, align 8
@.str = private unnamed_addr constant [22 x i8] c"invalid key to 'next'\00", align 1
@dummynode_ = internal constant %union.Node { %struct.NodeKey { %union.Value zeroinitializer, i8 16, i8 0, i32 0, %union.Value zeroinitializer } }, align 8
@.str.1 = private unnamed_addr constant [15 x i8] c"table overflow\00", align 1
@.str.2 = private unnamed_addr constant [19 x i8] c"table index is nil\00", align 1
@.str.3 = private unnamed_addr constant [19 x i8] c"table index is NaN\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaH_realasize(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.Table, ptr %5, i32 0, i32 3
  %7 = load i8, ptr %6, align 2
  %8 = zext i8 %7 to i32
  %9 = and i32 %8, 128
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %21

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.Table, ptr %12, i32 0, i32 5
  %14 = load i32, ptr %13, align 4
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.Table, ptr %15, i32 0, i32 5
  %17 = load i32, ptr %16, align 4
  %18 = sub i32 %17, 1
  %19 = and i32 %14, %18
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %21, label %25

21:                                               ; preds = %11, %1
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 5
  %24 = load i32, ptr %23, align 4
  store i32 %24, ptr %2, align 4
  br label %52

25:                                               ; preds = %11
  %26 = load ptr, ptr %3, align 8
  %27 = getelementptr inbounds %struct.Table, ptr %26, i32 0, i32 5
  %28 = load i32, ptr %27, align 4
  store i32 %28, ptr %4, align 4
  %29 = load i32, ptr %4, align 4
  %30 = lshr i32 %29, 1
  %31 = load i32, ptr %4, align 4
  %32 = or i32 %31, %30
  store i32 %32, ptr %4, align 4
  %33 = load i32, ptr %4, align 4
  %34 = lshr i32 %33, 2
  %35 = load i32, ptr %4, align 4
  %36 = or i32 %35, %34
  store i32 %36, ptr %4, align 4
  %37 = load i32, ptr %4, align 4
  %38 = lshr i32 %37, 4
  %39 = load i32, ptr %4, align 4
  %40 = or i32 %39, %38
  store i32 %40, ptr %4, align 4
  %41 = load i32, ptr %4, align 4
  %42 = lshr i32 %41, 8
  %43 = load i32, ptr %4, align 4
  %44 = or i32 %43, %42
  store i32 %44, ptr %4, align 4
  %45 = load i32, ptr %4, align 4
  %46 = lshr i32 %45, 16
  %47 = load i32, ptr %4, align 4
  %48 = or i32 %47, %46
  store i32 %48, ptr %4, align 4
  %49 = load i32, ptr %4, align 4
  %50 = add i32 %49, 1
  store i32 %50, ptr %4, align 4
  %51 = load i32, ptr %4, align 4
  store i32 %51, ptr %2, align 4
  br label %52

52:                                               ; preds = %25, %21
  %53 = load i32, ptr %2, align 4
  ret i32 %53
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaH_next(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = call i32 @luaH_realasize(ptr noundef %18)
  store i32 %19, ptr %8, align 4
  %20 = load ptr, ptr %5, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = load i32, ptr %8, align 4
  %24 = call i32 @findindex(ptr noundef %20, ptr noundef %21, ptr noundef %22, i32 noundef %23)
  store i32 %24, ptr %9, align 4
  br label %25

25:                                               ; preds = %69, %3
  %26 = load i32, ptr %9, align 4
  %27 = load i32, ptr %8, align 4
  %28 = icmp ult i32 %26, %27
  br i1 %28, label %29, label %72

29:                                               ; preds = %25
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.Table, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  %33 = load i32, ptr %9, align 4
  %34 = zext i32 %33 to i64
  %35 = getelementptr inbounds %struct.TValue, ptr %32, i64 %34
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = zext i8 %37 to i32
  %39 = and i32 %38, 15
  %40 = icmp eq i32 %39, 0
  br i1 %40, label %68, label %41

41:                                               ; preds = %29
  %42 = load ptr, ptr %7, align 8
  store ptr %42, ptr %10, align 8
  %43 = load i32, ptr %9, align 4
  %44 = add i32 %43, 1
  %45 = zext i32 %44 to i64
  %46 = load ptr, ptr %10, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  store i64 %45, ptr %47, align 8
  %48 = load ptr, ptr %10, align 8
  %49 = getelementptr inbounds %struct.TValue, ptr %48, i32 0, i32 1
  store i8 3, ptr %49, align 8
  %50 = load ptr, ptr %7, align 8
  %51 = getelementptr inbounds %union.StackValue, ptr %50, i64 1
  store ptr %51, ptr %11, align 8
  %52 = load ptr, ptr %6, align 8
  %53 = getelementptr inbounds %struct.Table, ptr %52, i32 0, i32 6
  %54 = load ptr, ptr %53, align 8
  %55 = load i32, ptr %9, align 4
  %56 = zext i32 %55 to i64
  %57 = getelementptr inbounds %struct.TValue, ptr %54, i64 %56
  store ptr %57, ptr %12, align 8
  %58 = load ptr, ptr %11, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 0
  %60 = load ptr, ptr %12, align 8
  %61 = getelementptr inbounds %struct.TValue, ptr %60, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %59, ptr align 8 %61, i64 8, i1 false)
  %62 = load ptr, ptr %12, align 8
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 1
  %64 = load i8, ptr %63, align 8
  %65 = load ptr, ptr %11, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 1
  store i8 %64, ptr %66, align 8
  %67 = load ptr, ptr %5, align 8
  store i32 1, ptr %4, align 4
  br label %133

68:                                               ; preds = %29
  br label %69

69:                                               ; preds = %68
  %70 = load i32, ptr %9, align 4
  %71 = add i32 %70, 1
  store i32 %71, ptr %9, align 4
  br label %25, !llvm.loop !6

72:                                               ; preds = %25
  %73 = load i32, ptr %8, align 4
  %74 = load i32, ptr %9, align 4
  %75 = sub i32 %74, %73
  store i32 %75, ptr %9, align 4
  br label %76

76:                                               ; preds = %129, %72
  %77 = load i32, ptr %9, align 4
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.Table, ptr %78, i32 0, i32 4
  %80 = load i8, ptr %79, align 1
  %81 = zext i8 %80 to i32
  %82 = shl i32 1, %81
  %83 = icmp slt i32 %77, %82
  br i1 %83, label %84, label %132

84:                                               ; preds = %76
  %85 = load ptr, ptr %6, align 8
  %86 = getelementptr inbounds %struct.Table, ptr %85, i32 0, i32 7
  %87 = load ptr, ptr %86, align 8
  %88 = load i32, ptr %9, align 4
  %89 = zext i32 %88 to i64
  %90 = getelementptr inbounds %union.Node, ptr %87, i64 %89
  %91 = getelementptr inbounds %struct.TValue, ptr %90, i32 0, i32 1
  %92 = load i8, ptr %91, align 8
  %93 = zext i8 %92 to i32
  %94 = and i32 %93, 15
  %95 = icmp eq i32 %94, 0
  br i1 %95, label %128, label %96

96:                                               ; preds = %84
  %97 = load ptr, ptr %6, align 8
  %98 = getelementptr inbounds %struct.Table, ptr %97, i32 0, i32 7
  %99 = load ptr, ptr %98, align 8
  %100 = load i32, ptr %9, align 4
  %101 = zext i32 %100 to i64
  %102 = getelementptr inbounds %union.Node, ptr %99, i64 %101
  store ptr %102, ptr %13, align 8
  %103 = load ptr, ptr %7, align 8
  store ptr %103, ptr %14, align 8
  %104 = load ptr, ptr %13, align 8
  store ptr %104, ptr %15, align 8
  %105 = load ptr, ptr %14, align 8
  %106 = getelementptr inbounds %struct.TValue, ptr %105, i32 0, i32 0
  %107 = load ptr, ptr %15, align 8
  %108 = getelementptr inbounds %struct.NodeKey, ptr %107, i32 0, i32 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %106, ptr align 8 %108, i64 8, i1 false)
  %109 = load ptr, ptr %15, align 8
  %110 = getelementptr inbounds %struct.NodeKey, ptr %109, i32 0, i32 2
  %111 = load i8, ptr %110, align 1
  %112 = load ptr, ptr %14, align 8
  %113 = getelementptr inbounds %struct.TValue, ptr %112, i32 0, i32 1
  store i8 %111, ptr %113, align 8
  %114 = load ptr, ptr %5, align 8
  %115 = load ptr, ptr %7, align 8
  %116 = getelementptr inbounds %union.StackValue, ptr %115, i64 1
  store ptr %116, ptr %16, align 8
  %117 = load ptr, ptr %13, align 8
  store ptr %117, ptr %17, align 8
  %118 = load ptr, ptr %16, align 8
  %119 = getelementptr inbounds %struct.TValue, ptr %118, i32 0, i32 0
  %120 = load ptr, ptr %17, align 8
  %121 = getelementptr inbounds %struct.TValue, ptr %120, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %119, ptr align 8 %121, i64 8, i1 false)
  %122 = load ptr, ptr %17, align 8
  %123 = getelementptr inbounds %struct.TValue, ptr %122, i32 0, i32 1
  %124 = load i8, ptr %123, align 8
  %125 = load ptr, ptr %16, align 8
  %126 = getelementptr inbounds %struct.TValue, ptr %125, i32 0, i32 1
  store i8 %124, ptr %126, align 8
  %127 = load ptr, ptr %5, align 8
  store i32 1, ptr %4, align 4
  br label %133

128:                                              ; preds = %84
  br label %129

129:                                              ; preds = %128
  %130 = load i32, ptr %9, align 4
  %131 = add i32 %130, 1
  store i32 %131, ptr %9, align 4
  br label %76, !llvm.loop !8

132:                                              ; preds = %76
  store i32 0, ptr %4, align 4
  br label %133

133:                                              ; preds = %132, %96, %41
  %134 = load i32, ptr %4, align 4
  ret i32 %134
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @findindex(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store i32 %3, ptr %9, align 4
  %12 = load ptr, ptr %8, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 15
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %19

18:                                               ; preds = %4
  store i32 0, ptr %5, align 4
  br label %70

19:                                               ; preds = %4
  %20 = load ptr, ptr %8, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  %24 = icmp eq i32 %23, 3
  br i1 %24, label %25, label %30

25:                                               ; preds = %19
  %26 = load ptr, ptr %8, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load i64, ptr %27, align 8
  %29 = call i32 @arrayindex(i64 noundef %28)
  br label %31

30:                                               ; preds = %19
  br label %31

31:                                               ; preds = %30, %25
  %32 = phi i32 [ %29, %25 ], [ 0, %30 ]
  store i32 %32, ptr %10, align 4
  %33 = load i32, ptr %10, align 4
  %34 = sub i32 %33, 1
  %35 = load i32, ptr %9, align 4
  %36 = icmp ult i32 %34, %35
  br i1 %36, label %37, label %39

37:                                               ; preds = %31
  %38 = load i32, ptr %10, align 4
  store i32 %38, ptr %5, align 4
  br label %70

39:                                               ; preds = %31
  %40 = load ptr, ptr %7, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = call ptr @getgeneric(ptr noundef %40, ptr noundef %41, i32 noundef 1)
  store ptr %42, ptr %11, align 8
  %43 = load ptr, ptr %11, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = zext i8 %45 to i32
  %47 = icmp eq i32 %46, 32
  %48 = zext i1 %47 to i32
  %49 = icmp ne i32 %48, 0
  %50 = zext i1 %49 to i32
  %51 = sext i32 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %53, label %55

53:                                               ; preds = %39
  %54 = load ptr, ptr %6, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %54, ptr noundef @.str) #5
  unreachable

55:                                               ; preds = %39
  %56 = load ptr, ptr %11, align 8
  %57 = load ptr, ptr %7, align 8
  %58 = getelementptr inbounds %struct.Table, ptr %57, i32 0, i32 7
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %union.Node, ptr %59, i64 0
  %61 = ptrtoint ptr %56 to i64
  %62 = ptrtoint ptr %60 to i64
  %63 = sub i64 %61, %62
  %64 = sdiv exact i64 %63, 24
  %65 = trunc i64 %64 to i32
  store i32 %65, ptr %10, align 4
  %66 = load i32, ptr %10, align 4
  %67 = add i32 %66, 1
  %68 = load i32, ptr %9, align 4
  %69 = add i32 %67, %68
  store i32 %69, ptr %5, align 4
  br label %70

70:                                               ; preds = %55, %37, %18
  %71 = load i32, ptr %5, align 4
  ret i32 %71
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_resize(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca %struct.Table, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %13 = load ptr, ptr %6, align 8
  %14 = call i32 @setlimittosize(ptr noundef %13)
  store i32 %14, ptr %11, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %8, align 4
  call void @setnodevector(ptr noundef %15, ptr noundef %10, i32 noundef %16)
  %17 = load i32, ptr %7, align 4
  %18 = load i32, ptr %11, align 4
  %19 = icmp ult i32 %17, %18
  br i1 %19, label %20, label %63

20:                                               ; preds = %4
  %21 = load i32, ptr %7, align 4
  %22 = load ptr, ptr %6, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 5
  store i32 %21, ptr %23, align 4
  %24 = load ptr, ptr %6, align 8
  call void @exchangehashpart(ptr noundef %24, ptr noundef %10)
  %25 = load i32, ptr %7, align 4
  store i32 %25, ptr %9, align 4
  br label %26

26:                                               ; preds = %55, %20
  %27 = load i32, ptr %9, align 4
  %28 = load i32, ptr %11, align 4
  %29 = icmp ult i32 %27, %28
  br i1 %29, label %30, label %58

30:                                               ; preds = %26
  %31 = load ptr, ptr %6, align 8
  %32 = getelementptr inbounds %struct.Table, ptr %31, i32 0, i32 6
  %33 = load ptr, ptr %32, align 8
  %34 = load i32, ptr %9, align 4
  %35 = zext i32 %34 to i64
  %36 = getelementptr inbounds %struct.TValue, ptr %33, i64 %35
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 1
  %38 = load i8, ptr %37, align 8
  %39 = zext i8 %38 to i32
  %40 = and i32 %39, 15
  %41 = icmp eq i32 %40, 0
  br i1 %41, label %54, label %42

42:                                               ; preds = %30
  %43 = load ptr, ptr %5, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = load i32, ptr %9, align 4
  %46 = add i32 %45, 1
  %47 = zext i32 %46 to i64
  %48 = load ptr, ptr %6, align 8
  %49 = getelementptr inbounds %struct.Table, ptr %48, i32 0, i32 6
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %9, align 4
  %52 = zext i32 %51 to i64
  %53 = getelementptr inbounds %struct.TValue, ptr %50, i64 %52
  call void @luaH_setint(ptr noundef %43, ptr noundef %44, i64 noundef %47, ptr noundef %53)
  br label %54

54:                                               ; preds = %42, %30
  br label %55

55:                                               ; preds = %54
  %56 = load i32, ptr %9, align 4
  %57 = add i32 %56, 1
  store i32 %57, ptr %9, align 4
  br label %26, !llvm.loop !9

58:                                               ; preds = %26
  %59 = load i32, ptr %11, align 4
  %60 = load ptr, ptr %6, align 8
  %61 = getelementptr inbounds %struct.Table, ptr %60, i32 0, i32 5
  store i32 %59, ptr %61, align 4
  %62 = load ptr, ptr %6, align 8
  call void @exchangehashpart(ptr noundef %62, ptr noundef %10)
  br label %63

63:                                               ; preds = %58, %4
  %64 = load ptr, ptr %5, align 8
  %65 = load ptr, ptr %6, align 8
  %66 = getelementptr inbounds %struct.Table, ptr %65, i32 0, i32 6
  %67 = load ptr, ptr %66, align 8
  %68 = load i32, ptr %11, align 4
  %69 = zext i32 %68 to i64
  %70 = mul i64 %69, 16
  %71 = load i32, ptr %7, align 4
  %72 = zext i32 %71 to i64
  %73 = mul i64 %72, 16
  %74 = call ptr @luaM_realloc_(ptr noundef %64, ptr noundef %67, i64 noundef %70, i64 noundef %73)
  store ptr %74, ptr %12, align 8
  %75 = load ptr, ptr %12, align 8
  %76 = icmp eq ptr %75, null
  br i1 %76, label %77, label %80

77:                                               ; preds = %63
  %78 = load i32, ptr %7, align 4
  %79 = icmp ugt i32 %78, 0
  br label %80

80:                                               ; preds = %77, %63
  %81 = phi i1 [ false, %63 ], [ %79, %77 ]
  %82 = zext i1 %81 to i32
  %83 = icmp ne i32 %82, 0
  %84 = zext i1 %83 to i32
  %85 = sext i32 %84 to i64
  %86 = icmp ne i64 %85, 0
  br i1 %86, label %87, label %90

87:                                               ; preds = %80
  %88 = load ptr, ptr %5, align 8
  call void @freehash(ptr noundef %88, ptr noundef %10)
  %89 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %89, i32 noundef 4) #5
  unreachable

90:                                               ; preds = %80
  %91 = load ptr, ptr %6, align 8
  call void @exchangehashpart(ptr noundef %91, ptr noundef %10)
  %92 = load ptr, ptr %12, align 8
  %93 = load ptr, ptr %6, align 8
  %94 = getelementptr inbounds %struct.Table, ptr %93, i32 0, i32 6
  store ptr %92, ptr %94, align 8
  %95 = load i32, ptr %7, align 4
  %96 = load ptr, ptr %6, align 8
  %97 = getelementptr inbounds %struct.Table, ptr %96, i32 0, i32 5
  store i32 %95, ptr %97, align 4
  %98 = load i32, ptr %11, align 4
  store i32 %98, ptr %9, align 4
  br label %99

99:                                               ; preds = %111, %90
  %100 = load i32, ptr %9, align 4
  %101 = load i32, ptr %7, align 4
  %102 = icmp ult i32 %100, %101
  br i1 %102, label %103, label %114

103:                                              ; preds = %99
  %104 = load ptr, ptr %6, align 8
  %105 = getelementptr inbounds %struct.Table, ptr %104, i32 0, i32 6
  %106 = load ptr, ptr %105, align 8
  %107 = load i32, ptr %9, align 4
  %108 = zext i32 %107 to i64
  %109 = getelementptr inbounds %struct.TValue, ptr %106, i64 %108
  %110 = getelementptr inbounds %struct.TValue, ptr %109, i32 0, i32 1
  store i8 16, ptr %110, align 8
  br label %111

111:                                              ; preds = %103
  %112 = load i32, ptr %9, align 4
  %113 = add i32 %112, 1
  store i32 %113, ptr %9, align 4
  br label %99, !llvm.loop !10

114:                                              ; preds = %99
  %115 = load ptr, ptr %5, align 8
  %116 = load ptr, ptr %6, align 8
  call void @reinsert(ptr noundef %115, ptr noundef %10, ptr noundef %116)
  %117 = load ptr, ptr %5, align 8
  call void @freehash(ptr noundef %117, ptr noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @setlimittosize(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @luaH_realasize(ptr noundef %3)
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.Table, ptr %5, i32 0, i32 5
  store i32 %4, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.Table, ptr %7, i32 0, i32 3
  %9 = load i8, ptr %8, align 2
  %10 = zext i8 %9 to i32
  %11 = and i32 %10, 127
  %12 = trunc i32 %11 to i8
  store i8 %12, ptr %8, align 2
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 5
  %15 = load i32, ptr %14, align 4
  ret i32 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setnodevector(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load i32, ptr %6, align 4
  %11 = icmp eq i32 %10, 0
  br i1 %11, label %12, label %19

12:                                               ; preds = %3
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 7
  store ptr @dummynode_, ptr %14, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.Table, ptr %15, i32 0, i32 4
  store i8 0, ptr %16, align 1
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.Table, ptr %17, i32 0, i32 8
  store ptr null, ptr %18, align 8
  br label %73

19:                                               ; preds = %3
  %20 = load i32, ptr %6, align 4
  %21 = call i32 @luaO_ceillog2(i32 noundef %20)
  store i32 %21, ptr %8, align 4
  %22 = load i32, ptr %8, align 4
  %23 = icmp sgt i32 %22, 30
  br i1 %23, label %28, label %24

24:                                               ; preds = %19
  %25 = load i32, ptr %8, align 4
  %26 = shl i32 1, %25
  %27 = icmp ugt i32 %26, 1073741824
  br i1 %27, label %28, label %30

28:                                               ; preds = %24, %19
  %29 = load ptr, ptr %4, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %29, ptr noundef @.str.1) #5
  unreachable

30:                                               ; preds = %24
  %31 = load i32, ptr %8, align 4
  %32 = shl i32 1, %31
  store i32 %32, ptr %6, align 4
  %33 = load ptr, ptr %4, align 8
  %34 = load i32, ptr %6, align 4
  %35 = zext i32 %34 to i64
  %36 = mul i64 %35, 24
  %37 = call ptr @luaM_malloc_(ptr noundef %33, i64 noundef %36, i32 noundef 0)
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.Table, ptr %38, i32 0, i32 7
  store ptr %37, ptr %39, align 8
  store i32 0, ptr %7, align 4
  br label %40

40:                                               ; preds = %57, %30
  %41 = load i32, ptr %7, align 4
  %42 = load i32, ptr %6, align 4
  %43 = icmp slt i32 %41, %42
  br i1 %43, label %44, label %60

44:                                               ; preds = %40
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.Table, ptr %45, i32 0, i32 7
  %47 = load ptr, ptr %46, align 8
  %48 = load i32, ptr %7, align 4
  %49 = sext i32 %48 to i64
  %50 = getelementptr inbounds %union.Node, ptr %47, i64 %49
  store ptr %50, ptr %9, align 8
  %51 = load ptr, ptr %9, align 8
  %52 = getelementptr inbounds %struct.NodeKey, ptr %51, i32 0, i32 3
  store i32 0, ptr %52, align 4
  %53 = load ptr, ptr %9, align 8
  %54 = getelementptr inbounds %struct.NodeKey, ptr %53, i32 0, i32 2
  store i8 0, ptr %54, align 1
  %55 = load ptr, ptr %9, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  store i8 16, ptr %56, align 8
  br label %57

57:                                               ; preds = %44
  %58 = load i32, ptr %7, align 4
  %59 = add nsw i32 %58, 1
  store i32 %59, ptr %7, align 4
  br label %40, !llvm.loop !11

60:                                               ; preds = %40
  %61 = load i32, ptr %8, align 4
  %62 = trunc i32 %61 to i8
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.Table, ptr %63, i32 0, i32 4
  store i8 %62, ptr %64, align 1
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.Table, ptr %65, i32 0, i32 7
  %67 = load ptr, ptr %66, align 8
  %68 = load i32, ptr %6, align 4
  %69 = zext i32 %68 to i64
  %70 = getelementptr inbounds %union.Node, ptr %67, i64 %69
  %71 = load ptr, ptr %5, align 8
  %72 = getelementptr inbounds %struct.Table, ptr %71, i32 0, i32 8
  store ptr %70, ptr %72, align 8
  br label %73

73:                                               ; preds = %60, %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @exchangehashpart(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i8, align 1
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 4
  %10 = load i8, ptr %9, align 1
  store i8 %10, ptr %5, align 1
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 7
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.Table, ptr %14, i32 0, i32 8
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %7, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.Table, ptr %17, i32 0, i32 4
  %19 = load i8, ptr %18, align 1
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.Table, ptr %20, i32 0, i32 4
  store i8 %19, ptr %21, align 1
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 7
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.Table, ptr %25, i32 0, i32 7
  store ptr %24, ptr %26, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.Table, ptr %27, i32 0, i32 8
  %29 = load ptr, ptr %28, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.Table, ptr %30, i32 0, i32 8
  store ptr %29, ptr %31, align 8
  %32 = load i8, ptr %5, align 1
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.Table, ptr %33, i32 0, i32 4
  store i8 %32, ptr %34, align 1
  %35 = load ptr, ptr %6, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.Table, ptr %36, i32 0, i32 7
  store ptr %35, ptr %37, align 8
  %38 = load ptr, ptr %7, align 8
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.Table, ptr %39, i32 0, i32 8
  store ptr %38, ptr %40, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_setint(ptr noundef %0, ptr noundef %1, i64 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %struct.TValue, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = load i64, ptr %7, align 8
  %16 = call ptr @luaH_getint(ptr noundef %14, i64 noundef %15)
  store ptr %16, ptr %9, align 8
  %17 = load ptr, ptr %9, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 32
  br i1 %21, label %22, label %31

22:                                               ; preds = %4
  store ptr %10, ptr %11, align 8
  %23 = load i64, ptr %7, align 8
  %24 = load ptr, ptr %11, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  store i64 %23, ptr %25, align 8
  %26 = load ptr, ptr %11, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 1
  store i8 3, ptr %27, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %8, align 8
  call void @luaH_newkey(ptr noundef %28, ptr noundef %29, ptr noundef %10, ptr noundef %30)
  br label %44

31:                                               ; preds = %4
  %32 = load ptr, ptr %9, align 8
  store ptr %32, ptr %12, align 8
  %33 = load ptr, ptr %8, align 8
  store ptr %33, ptr %13, align 8
  %34 = load ptr, ptr %12, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %13, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %35, ptr align 8 %37, i64 8, i1 false)
  %38 = load ptr, ptr %13, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 1
  %40 = load i8, ptr %39, align 8
  %41 = load ptr, ptr %12, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 1
  store i8 %40, ptr %42, align 8
  %43 = load ptr, ptr %5, align 8
  br label %44

44:                                               ; preds = %31, %22
  ret void
}

declare hidden ptr @luaM_realloc_(ptr noundef, ptr noundef, i64 noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freehash(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.Table, ptr %5, i32 0, i32 8
  %7 = load ptr, ptr %6, align 8
  %8 = icmp eq ptr %7, null
  br i1 %8, label %21, label %9

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 7
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.Table, ptr %14, i32 0, i32 4
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = shl i32 1, %17
  %19 = sext i32 %18 to i64
  %20 = mul i64 %19, 24
  call void @luaM_free_(ptr noundef %10, ptr noundef %13, i64 noundef %20)
  br label %21

21:                                               ; preds = %9, %2
  ret void
}

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @reinsert(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca %struct.TValue, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 4
  %15 = load i8, ptr %14, align 1
  %16 = zext i8 %15 to i32
  %17 = shl i32 1, %16
  store i32 %17, ptr %8, align 4
  store i32 0, ptr %7, align 4
  br label %18

18:                                               ; preds = %51, %3
  %19 = load i32, ptr %7, align 4
  %20 = load i32, ptr %8, align 4
  %21 = icmp slt i32 %19, %20
  br i1 %21, label %22, label %54

22:                                               ; preds = %18
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Table, ptr %23, i32 0, i32 7
  %25 = load ptr, ptr %24, align 8
  %26 = load i32, ptr %7, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds %union.Node, ptr %25, i64 %27
  store ptr %28, ptr %9, align 8
  %29 = load ptr, ptr %9, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  %31 = load i8, ptr %30, align 8
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 15
  %34 = icmp eq i32 %33, 0
  br i1 %34, label %50, label %35

35:                                               ; preds = %22
  store ptr %10, ptr %11, align 8
  %36 = load ptr, ptr %9, align 8
  store ptr %36, ptr %12, align 8
  %37 = load ptr, ptr %11, align 8
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 0
  %39 = load ptr, ptr %12, align 8
  %40 = getelementptr inbounds %struct.NodeKey, ptr %39, i32 0, i32 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %38, ptr align 8 %40, i64 8, i1 false)
  %41 = load ptr, ptr %12, align 8
  %42 = getelementptr inbounds %struct.NodeKey, ptr %41, i32 0, i32 2
  %43 = load i8, ptr %42, align 1
  %44 = load ptr, ptr %11, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 1
  store i8 %43, ptr %45, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = load ptr, ptr %4, align 8
  %48 = load ptr, ptr %6, align 8
  %49 = load ptr, ptr %9, align 8
  call void @luaH_set(ptr noundef %47, ptr noundef %48, ptr noundef %10, ptr noundef %49)
  br label %50

50:                                               ; preds = %35, %22
  br label %51

51:                                               ; preds = %50
  %52 = load i32, ptr %7, align 4
  %53 = add nsw i32 %52, 1
  store i32 %53, ptr %7, align 4
  br label %18, !llvm.loop !12

54:                                               ; preds = %18
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_resizearray(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 8
  %10 = load ptr, ptr %9, align 8
  %11 = icmp eq ptr %10, null
  br i1 %11, label %12, label %13

12:                                               ; preds = %3
  br label %19

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.Table, ptr %14, i32 0, i32 4
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = shl i32 1, %17
  br label %19

19:                                               ; preds = %13, %12
  %20 = phi i32 [ 0, %12 ], [ %18, %13 ]
  store i32 %20, ptr %7, align 4
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = load i32, ptr %6, align 4
  %24 = load i32, ptr %7, align 4
  call void @luaH_resize(ptr noundef %21, ptr noundef %22, i32 noundef %23, i32 noundef %24)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaH_new(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaC_newobj(ptr noundef %5, i32 noundef 5, i64 noundef 56)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 9
  store ptr null, ptr %9, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.Table, ptr %10, i32 0, i32 3
  store i8 63, ptr %11, align 2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.Table, ptr %12, i32 0, i32 6
  store ptr null, ptr %13, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.Table, ptr %14, i32 0, i32 5
  store i32 0, ptr %15, align 4
  %16 = load ptr, ptr %2, align 8
  %17 = load ptr, ptr %4, align 8
  call void @setnodevector(ptr noundef %16, ptr noundef %17, i32 noundef 0)
  %18 = load ptr, ptr %4, align 8
  ret ptr %18
}

declare hidden ptr @luaC_newobj(ptr noundef, i32 noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_free(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  call void @freehash(ptr noundef %5, ptr noundef %6)
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 6
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call i32 @luaH_realasize(ptr noundef %11)
  %13 = zext i32 %12 to i64
  %14 = mul i64 %13, 16
  call void @luaM_free_(ptr noundef %7, ptr noundef %10, i64 noundef %14)
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %4, align 8
  call void @luaM_free_(ptr noundef %15, ptr noundef %16, i64 noundef 56)
  ret void
}

declare hidden void @luaM_free_(ptr noundef, ptr noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaH_getint(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.Table, ptr %9, i32 0, i32 5
  %11 = load i32, ptr %10, align 4
  %12 = zext i32 %11 to i64
  store i64 %12, ptr %6, align 8
  %13 = load i64, ptr %5, align 8
  %14 = sub i64 %13, 1
  %15 = load i64, ptr %6, align 8
  %16 = icmp ult i64 %14, %15
  br i1 %16, label %17, label %24

17:                                               ; preds = %2
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Table, ptr %18, i32 0, i32 6
  %20 = load ptr, ptr %19, align 8
  %21 = load i64, ptr %5, align 8
  %22 = sub nsw i64 %21, 1
  %23 = getelementptr inbounds %struct.TValue, ptr %20, i64 %22
  store ptr %23, ptr %3, align 8
  br label %83

24:                                               ; preds = %2
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.Table, ptr %25, i32 0, i32 3
  %27 = load i8, ptr %26, align 2
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 128
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %51

31:                                               ; preds = %24
  %32 = load i64, ptr %5, align 8
  %33 = sub i64 %32, 1
  %34 = load i64, ptr %6, align 8
  %35 = sub i64 %34, 1
  %36 = xor i64 %35, -1
  %37 = and i64 %33, %36
  %38 = load i64, ptr %6, align 8
  %39 = icmp ult i64 %37, %38
  br i1 %39, label %40, label %51

40:                                               ; preds = %31
  %41 = load i64, ptr %5, align 8
  %42 = trunc i64 %41 to i32
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.Table, ptr %43, i32 0, i32 5
  store i32 %42, ptr %44, align 4
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.Table, ptr %45, i32 0, i32 6
  %47 = load ptr, ptr %46, align 8
  %48 = load i64, ptr %5, align 8
  %49 = sub nsw i64 %48, 1
  %50 = getelementptr inbounds %struct.TValue, ptr %47, i64 %49
  store ptr %50, ptr %3, align 8
  br label %83

51:                                               ; preds = %31, %24
  %52 = load ptr, ptr %4, align 8
  %53 = load i64, ptr %5, align 8
  %54 = call ptr @hashint(ptr noundef %52, i64 noundef %53)
  store ptr %54, ptr %7, align 8
  br label %55

55:                                               ; preds = %81, %51
  %56 = load ptr, ptr %7, align 8
  %57 = getelementptr inbounds %struct.NodeKey, ptr %56, i32 0, i32 2
  %58 = load i8, ptr %57, align 1
  %59 = zext i8 %58 to i32
  %60 = icmp eq i32 %59, 3
  br i1 %60, label %61, label %69

61:                                               ; preds = %55
  %62 = load ptr, ptr %7, align 8
  %63 = getelementptr inbounds %struct.NodeKey, ptr %62, i32 0, i32 4
  %64 = load i64, ptr %63, align 8
  %65 = load i64, ptr %5, align 8
  %66 = icmp eq i64 %64, %65
  br i1 %66, label %67, label %69

67:                                               ; preds = %61
  %68 = load ptr, ptr %7, align 8
  store ptr %68, ptr %3, align 8
  br label %83

69:                                               ; preds = %61, %55
  %70 = load ptr, ptr %7, align 8
  %71 = getelementptr inbounds %struct.NodeKey, ptr %70, i32 0, i32 3
  %72 = load i32, ptr %71, align 4
  store i32 %72, ptr %8, align 4
  %73 = load i32, ptr %8, align 4
  %74 = icmp eq i32 %73, 0
  br i1 %74, label %75, label %76

75:                                               ; preds = %69
  br label %82

76:                                               ; preds = %69
  %77 = load i32, ptr %8, align 4
  %78 = load ptr, ptr %7, align 8
  %79 = sext i32 %77 to i64
  %80 = getelementptr inbounds %union.Node, ptr %78, i64 %79
  store ptr %80, ptr %7, align 8
  br label %81

81:                                               ; preds = %76
  br label %55

82:                                               ; preds = %75
  store ptr @absentkey, ptr %3, align 8
  br label %83

83:                                               ; preds = %82, %67, %40, %17
  %84 = load ptr, ptr %3, align 8
  ret ptr %84
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @hashint(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %7 = load i64, ptr %5, align 8
  store i64 %7, ptr %6, align 8
  %8 = load i64, ptr %6, align 8
  %9 = icmp ule i64 %8, 2147483647
  br i1 %9, label %10, label %26

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 7
  %13 = load ptr, ptr %12, align 8
  %14 = load i64, ptr %6, align 8
  %15 = trunc i64 %14 to i32
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.Table, ptr %16, i32 0, i32 4
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = shl i32 1, %19
  %21 = sub nsw i32 %20, 1
  %22 = or i32 %21, 1
  %23 = srem i32 %15, %22
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds %union.Node, ptr %13, i64 %24
  store ptr %25, ptr %3, align 8
  br label %41

26:                                               ; preds = %2
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.Table, ptr %27, i32 0, i32 7
  %29 = load ptr, ptr %28, align 8
  %30 = load i64, ptr %6, align 8
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.Table, ptr %31, i32 0, i32 4
  %33 = load i8, ptr %32, align 1
  %34 = zext i8 %33 to i32
  %35 = shl i32 1, %34
  %36 = sub nsw i32 %35, 1
  %37 = or i32 %36, 1
  %38 = sext i32 %37 to i64
  %39 = urem i64 %30, %38
  %40 = getelementptr inbounds %union.Node, ptr %29, i64 %39
  store ptr %40, ptr %3, align 8
  br label %41

41:                                               ; preds = %26, %10
  %42 = load ptr, ptr %3, align 8
  ret ptr %42
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaH_getshortstr(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.TString, ptr %11, i32 0, i32 5
  %13 = load i32, ptr %12, align 4
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.Table, ptr %14, i32 0, i32 4
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = shl i32 1, %17
  %19 = sub nsw i32 %18, 1
  %20 = and i32 %13, %19
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds %union.Node, ptr %10, i64 %21
  store ptr %22, ptr %6, align 8
  br label %23

23:                                               ; preds = %49, %2
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.NodeKey, ptr %24, i32 0, i32 2
  %26 = load i8, ptr %25, align 1
  %27 = zext i8 %26 to i32
  %28 = icmp eq i32 %27, 68
  br i1 %28, label %29, label %37

29:                                               ; preds = %23
  %30 = load ptr, ptr %6, align 8
  %31 = getelementptr inbounds %struct.NodeKey, ptr %30, i32 0, i32 4
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = icmp eq ptr %32, %33
  br i1 %34, label %35, label %37

35:                                               ; preds = %29
  %36 = load ptr, ptr %6, align 8
  store ptr %36, ptr %3, align 8
  br label %50

37:                                               ; preds = %29, %23
  %38 = load ptr, ptr %6, align 8
  %39 = getelementptr inbounds %struct.NodeKey, ptr %38, i32 0, i32 3
  %40 = load i32, ptr %39, align 4
  store i32 %40, ptr %7, align 4
  %41 = load i32, ptr %7, align 4
  %42 = icmp eq i32 %41, 0
  br i1 %42, label %43, label %44

43:                                               ; preds = %37
  store ptr @absentkey, ptr %3, align 8
  br label %50

44:                                               ; preds = %37
  %45 = load i32, ptr %7, align 4
  %46 = load ptr, ptr %6, align 8
  %47 = sext i32 %45 to i64
  %48 = getelementptr inbounds %union.Node, ptr %46, i64 %47
  store ptr %48, ptr %6, align 8
  br label %49

49:                                               ; preds = %44
  br label %23

50:                                               ; preds = %43, %35
  %51 = load ptr, ptr %3, align 8
  ret ptr %51
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaH_getstr(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.TValue, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds %struct.TString, ptr %9, i32 0, i32 1
  %11 = load i8, ptr %10, align 8
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 4
  br i1 %13, label %14, label %18

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = call ptr @luaH_getshortstr(ptr noundef %15, ptr noundef %16)
  store ptr %17, ptr %3, align 8
  br label %33

18:                                               ; preds = %2
  store ptr %6, ptr %7, align 8
  %19 = load ptr, ptr %5, align 8
  store ptr %19, ptr %8, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 0
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  %27 = or i32 %26, 64
  %28 = trunc i32 %27 to i8
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  store i8 %28, ptr %30, align 8
  %31 = load ptr, ptr %4, align 8
  %32 = call ptr @getgeneric(ptr noundef %31, ptr noundef %6, i32 noundef 0)
  store ptr %32, ptr %3, align 8
  br label %33

33:                                               ; preds = %18, %14
  %34 = load ptr, ptr %3, align 8
  ret ptr %34
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getgeneric(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = call ptr @mainpositionTV(ptr noundef %10, ptr noundef %11)
  store ptr %12, ptr %8, align 8
  br label %13

13:                                               ; preds = %33, %3
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %8, align 8
  %16 = load i32, ptr %7, align 4
  %17 = call i32 @equalkey(ptr noundef %14, ptr noundef %15, i32 noundef %16)
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %21

19:                                               ; preds = %13
  %20 = load ptr, ptr %8, align 8
  store ptr %20, ptr %4, align 8
  br label %34

21:                                               ; preds = %13
  %22 = load ptr, ptr %8, align 8
  %23 = getelementptr inbounds %struct.NodeKey, ptr %22, i32 0, i32 3
  %24 = load i32, ptr %23, align 4
  store i32 %24, ptr %9, align 4
  %25 = load i32, ptr %9, align 4
  %26 = icmp eq i32 %25, 0
  br i1 %26, label %27, label %28

27:                                               ; preds = %21
  store ptr @absentkey, ptr %4, align 8
  br label %34

28:                                               ; preds = %21
  %29 = load i32, ptr %9, align 4
  %30 = load ptr, ptr %8, align 8
  %31 = sext i32 %29 to i64
  %32 = getelementptr inbounds %union.Node, ptr %30, i64 %31
  store ptr %32, ptr %8, align 8
  br label %33

33:                                               ; preds = %28
  br label %13

34:                                               ; preds = %27, %19
  %35 = load ptr, ptr %4, align 8
  ret ptr %35
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaH_get(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = getelementptr inbounds %struct.TValue, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  %11 = and i32 %10, 63
  switch i32 %11, label %36 [
    i32 4, label %12
    i32 3, label %18
    i32 0, label %24
    i32 19, label %25
  ]

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = call ptr @luaH_getshortstr(ptr noundef %13, ptr noundef %16)
  store ptr %17, ptr %3, align 8
  br label %40

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 0
  %22 = load i64, ptr %21, align 8
  %23 = call ptr @luaH_getint(ptr noundef %19, i64 noundef %22)
  store ptr %23, ptr %3, align 8
  br label %40

24:                                               ; preds = %2
  store ptr @absentkey, ptr %3, align 8
  br label %40

25:                                               ; preds = %2
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load double, ptr %27, align 8
  %29 = call i32 @luaV_flttointeger(double noundef %28, ptr noundef %6, i32 noundef 0)
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %35

31:                                               ; preds = %25
  %32 = load ptr, ptr %4, align 8
  %33 = load i64, ptr %6, align 8
  %34 = call ptr @luaH_getint(ptr noundef %32, i64 noundef %33)
  store ptr %34, ptr %3, align 8
  br label %40

35:                                               ; preds = %25
  br label %36

36:                                               ; preds = %2, %35
  %37 = load ptr, ptr %4, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = call ptr @getgeneric(ptr noundef %37, ptr noundef %38, i32 noundef 0)
  store ptr %39, ptr %3, align 8
  br label %40

40:                                               ; preds = %36, %31, %24, %18, %12
  %41 = load ptr, ptr %3, align 8
  ret ptr %41
}

declare hidden i32 @luaV_flttointeger(double noundef, ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_finishset(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %13 = load ptr, ptr %9, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = icmp eq i32 %16, 32
  br i1 %17, label %18, label %23

18:                                               ; preds = %5
  %19 = load ptr, ptr %6, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = load ptr, ptr %10, align 8
  call void @luaH_newkey(ptr noundef %19, ptr noundef %20, ptr noundef %21, ptr noundef %22)
  br label %36

23:                                               ; preds = %5
  %24 = load ptr, ptr %9, align 8
  store ptr %24, ptr %11, align 8
  %25 = load ptr, ptr %10, align 8
  store ptr %25, ptr %12, align 8
  %26 = load ptr, ptr %11, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %12, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %27, ptr align 8 %29, i64 8, i1 false)
  %30 = load ptr, ptr %12, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  %32 = load i8, ptr %31, align 8
  %33 = load ptr, ptr %11, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 1
  store i8 %32, ptr %34, align 8
  %35 = load ptr, ptr %6, align 8
  br label %36

36:                                               ; preds = %23, %18
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @luaH_newkey(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %struct.TValue, align 8
  %11 = alloca double, align 8
  %12 = alloca i64, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  %24 = and i32 %23, 15
  %25 = icmp eq i32 %24, 0
  %26 = zext i1 %25 to i32
  %27 = icmp ne i32 %26, 0
  %28 = zext i1 %27 to i32
  %29 = sext i32 %28 to i64
  %30 = icmp ne i64 %29, 0
  br i1 %30, label %31, label %33

31:                                               ; preds = %4
  %32 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %32, ptr noundef @.str.2) #5
  unreachable

33:                                               ; preds = %4
  %34 = load ptr, ptr %7, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  %36 = load i8, ptr %35, align 8
  %37 = zext i8 %36 to i32
  %38 = icmp eq i32 %37, 19
  br i1 %38, label %39, label %66

39:                                               ; preds = %33
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load double, ptr %41, align 8
  store double %42, ptr %11, align 8
  %43 = load double, ptr %11, align 8
  %44 = call i32 @luaV_flttointeger(double noundef %43, ptr noundef %12, i32 noundef 0)
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %46, label %52

46:                                               ; preds = %39
  store ptr %10, ptr %13, align 8
  %47 = load i64, ptr %12, align 8
  %48 = load ptr, ptr %13, align 8
  %49 = getelementptr inbounds %struct.TValue, ptr %48, i32 0, i32 0
  store i64 %47, ptr %49, align 8
  %50 = load ptr, ptr %13, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  store i8 3, ptr %51, align 8
  store ptr %10, ptr %7, align 8
  br label %65

52:                                               ; preds = %39
  %53 = load double, ptr %11, align 8
  %54 = load double, ptr %11, align 8
  %55 = fcmp oeq double %53, %54
  %56 = xor i1 %55, true
  %57 = zext i1 %56 to i32
  %58 = icmp ne i32 %57, 0
  %59 = zext i1 %58 to i32
  %60 = sext i32 %59 to i64
  %61 = icmp ne i64 %60, 0
  br i1 %61, label %62, label %64

62:                                               ; preds = %52
  %63 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %63, ptr noundef @.str.3) #5
  unreachable

64:                                               ; preds = %52
  br label %65

65:                                               ; preds = %64, %46
  br label %66

66:                                               ; preds = %65, %33
  br label %67

67:                                               ; preds = %66
  %68 = load ptr, ptr %8, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 1
  %70 = load i8, ptr %69, align 8
  %71 = zext i8 %70 to i32
  %72 = and i32 %71, 15
  %73 = icmp eq i32 %72, 0
  br i1 %73, label %74, label %75

74:                                               ; preds = %67
  br label %247

75:                                               ; preds = %67
  %76 = load ptr, ptr %6, align 8
  %77 = load ptr, ptr %7, align 8
  %78 = call ptr @mainpositionTV(ptr noundef %76, ptr noundef %77)
  store ptr %78, ptr %9, align 8
  %79 = load ptr, ptr %9, align 8
  %80 = getelementptr inbounds %struct.TValue, ptr %79, i32 0, i32 1
  %81 = load i8, ptr %80, align 8
  %82 = zext i8 %81 to i32
  %83 = and i32 %82, 15
  %84 = icmp eq i32 %83, 0
  br i1 %84, label %85, label %90

85:                                               ; preds = %75
  %86 = load ptr, ptr %6, align 8
  %87 = getelementptr inbounds %struct.Table, ptr %86, i32 0, i32 8
  %88 = load ptr, ptr %87, align 8
  %89 = icmp eq ptr %88, null
  br i1 %89, label %90, label %193

90:                                               ; preds = %85, %75
  %91 = load ptr, ptr %6, align 8
  %92 = call ptr @getfreepos(ptr noundef %91)
  store ptr %92, ptr %15, align 8
  %93 = load ptr, ptr %15, align 8
  %94 = icmp eq ptr %93, null
  br i1 %94, label %95, label %103

95:                                               ; preds = %90
  %96 = load ptr, ptr %5, align 8
  %97 = load ptr, ptr %6, align 8
  %98 = load ptr, ptr %7, align 8
  call void @rehash(ptr noundef %96, ptr noundef %97, ptr noundef %98)
  %99 = load ptr, ptr %5, align 8
  %100 = load ptr, ptr %6, align 8
  %101 = load ptr, ptr %7, align 8
  %102 = load ptr, ptr %8, align 8
  call void @luaH_set(ptr noundef %99, ptr noundef %100, ptr noundef %101, ptr noundef %102)
  br label %247

103:                                              ; preds = %90
  %104 = load ptr, ptr %6, align 8
  %105 = load ptr, ptr %9, align 8
  %106 = call ptr @mainpositionfromnode(ptr noundef %104, ptr noundef %105)
  store ptr %106, ptr %14, align 8
  %107 = load ptr, ptr %14, align 8
  %108 = load ptr, ptr %9, align 8
  %109 = icmp ne ptr %107, %108
  br i1 %109, label %110, label %160

110:                                              ; preds = %103
  br label %111

111:                                              ; preds = %120, %110
  %112 = load ptr, ptr %14, align 8
  %113 = load ptr, ptr %14, align 8
  %114 = getelementptr inbounds %struct.NodeKey, ptr %113, i32 0, i32 3
  %115 = load i32, ptr %114, align 4
  %116 = sext i32 %115 to i64
  %117 = getelementptr inbounds %union.Node, ptr %112, i64 %116
  %118 = load ptr, ptr %9, align 8
  %119 = icmp ne ptr %117, %118
  br i1 %119, label %120, label %127

120:                                              ; preds = %111
  %121 = load ptr, ptr %14, align 8
  %122 = getelementptr inbounds %struct.NodeKey, ptr %121, i32 0, i32 3
  %123 = load i32, ptr %122, align 4
  %124 = load ptr, ptr %14, align 8
  %125 = sext i32 %123 to i64
  %126 = getelementptr inbounds %union.Node, ptr %124, i64 %125
  store ptr %126, ptr %14, align 8
  br label %111, !llvm.loop !13

127:                                              ; preds = %111
  %128 = load ptr, ptr %15, align 8
  %129 = load ptr, ptr %14, align 8
  %130 = ptrtoint ptr %128 to i64
  %131 = ptrtoint ptr %129 to i64
  %132 = sub i64 %130, %131
  %133 = sdiv exact i64 %132, 24
  %134 = trunc i64 %133 to i32
  %135 = load ptr, ptr %14, align 8
  %136 = getelementptr inbounds %struct.NodeKey, ptr %135, i32 0, i32 3
  store i32 %134, ptr %136, align 4
  %137 = load ptr, ptr %15, align 8
  %138 = load ptr, ptr %9, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %137, ptr align 8 %138, i64 24, i1 false)
  %139 = load ptr, ptr %9, align 8
  %140 = getelementptr inbounds %struct.NodeKey, ptr %139, i32 0, i32 3
  %141 = load i32, ptr %140, align 4
  %142 = icmp ne i32 %141, 0
  br i1 %142, label %143, label %157

143:                                              ; preds = %127
  %144 = load ptr, ptr %9, align 8
  %145 = load ptr, ptr %15, align 8
  %146 = ptrtoint ptr %144 to i64
  %147 = ptrtoint ptr %145 to i64
  %148 = sub i64 %146, %147
  %149 = sdiv exact i64 %148, 24
  %150 = trunc i64 %149 to i32
  %151 = load ptr, ptr %15, align 8
  %152 = getelementptr inbounds %struct.NodeKey, ptr %151, i32 0, i32 3
  %153 = load i32, ptr %152, align 4
  %154 = add nsw i32 %153, %150
  store i32 %154, ptr %152, align 4
  %155 = load ptr, ptr %9, align 8
  %156 = getelementptr inbounds %struct.NodeKey, ptr %155, i32 0, i32 3
  store i32 0, ptr %156, align 4
  br label %157

157:                                              ; preds = %143, %127
  %158 = load ptr, ptr %9, align 8
  %159 = getelementptr inbounds %struct.TValue, ptr %158, i32 0, i32 1
  store i8 16, ptr %159, align 8
  br label %192

160:                                              ; preds = %103
  %161 = load ptr, ptr %9, align 8
  %162 = getelementptr inbounds %struct.NodeKey, ptr %161, i32 0, i32 3
  %163 = load i32, ptr %162, align 4
  %164 = icmp ne i32 %163, 0
  br i1 %164, label %165, label %180

165:                                              ; preds = %160
  %166 = load ptr, ptr %9, align 8
  %167 = load ptr, ptr %9, align 8
  %168 = getelementptr inbounds %struct.NodeKey, ptr %167, i32 0, i32 3
  %169 = load i32, ptr %168, align 4
  %170 = sext i32 %169 to i64
  %171 = getelementptr inbounds %union.Node, ptr %166, i64 %170
  %172 = load ptr, ptr %15, align 8
  %173 = ptrtoint ptr %171 to i64
  %174 = ptrtoint ptr %172 to i64
  %175 = sub i64 %173, %174
  %176 = sdiv exact i64 %175, 24
  %177 = trunc i64 %176 to i32
  %178 = load ptr, ptr %15, align 8
  %179 = getelementptr inbounds %struct.NodeKey, ptr %178, i32 0, i32 3
  store i32 %177, ptr %179, align 4
  br label %181

180:                                              ; preds = %160
  br label %181

181:                                              ; preds = %180, %165
  %182 = load ptr, ptr %15, align 8
  %183 = load ptr, ptr %9, align 8
  %184 = ptrtoint ptr %182 to i64
  %185 = ptrtoint ptr %183 to i64
  %186 = sub i64 %184, %185
  %187 = sdiv exact i64 %186, 24
  %188 = trunc i64 %187 to i32
  %189 = load ptr, ptr %9, align 8
  %190 = getelementptr inbounds %struct.NodeKey, ptr %189, i32 0, i32 3
  store i32 %188, ptr %190, align 4
  %191 = load ptr, ptr %15, align 8
  store ptr %191, ptr %9, align 8
  br label %192

192:                                              ; preds = %181, %157
  br label %193

193:                                              ; preds = %192, %85
  %194 = load ptr, ptr %9, align 8
  store ptr %194, ptr %16, align 8
  %195 = load ptr, ptr %7, align 8
  store ptr %195, ptr %17, align 8
  %196 = load ptr, ptr %16, align 8
  %197 = getelementptr inbounds %struct.NodeKey, ptr %196, i32 0, i32 4
  %198 = load ptr, ptr %17, align 8
  %199 = getelementptr inbounds %struct.TValue, ptr %198, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %197, ptr align 8 %199, i64 8, i1 false)
  %200 = load ptr, ptr %17, align 8
  %201 = getelementptr inbounds %struct.TValue, ptr %200, i32 0, i32 1
  %202 = load i8, ptr %201, align 8
  %203 = load ptr, ptr %16, align 8
  %204 = getelementptr inbounds %struct.NodeKey, ptr %203, i32 0, i32 2
  store i8 %202, ptr %204, align 1
  %205 = load ptr, ptr %5, align 8
  %206 = load ptr, ptr %7, align 8
  %207 = getelementptr inbounds %struct.TValue, ptr %206, i32 0, i32 1
  %208 = load i8, ptr %207, align 8
  %209 = zext i8 %208 to i32
  %210 = and i32 %209, 64
  %211 = icmp ne i32 %210, 0
  br i1 %211, label %212, label %233

212:                                              ; preds = %193
  %213 = load ptr, ptr %6, align 8
  %214 = getelementptr inbounds %struct.GCObject, ptr %213, i32 0, i32 2
  %215 = load i8, ptr %214, align 1
  %216 = zext i8 %215 to i32
  %217 = and i32 %216, 32
  %218 = icmp ne i32 %217, 0
  br i1 %218, label %219, label %231

219:                                              ; preds = %212
  %220 = load ptr, ptr %7, align 8
  %221 = getelementptr inbounds %struct.TValue, ptr %220, i32 0, i32 0
  %222 = load ptr, ptr %221, align 8
  %223 = getelementptr inbounds %struct.GCObject, ptr %222, i32 0, i32 2
  %224 = load i8, ptr %223, align 1
  %225 = zext i8 %224 to i32
  %226 = and i32 %225, 24
  %227 = icmp ne i32 %226, 0
  br i1 %227, label %228, label %231

228:                                              ; preds = %219
  %229 = load ptr, ptr %5, align 8
  %230 = load ptr, ptr %6, align 8
  call void @luaC_barrierback_(ptr noundef %229, ptr noundef %230)
  br label %232

231:                                              ; preds = %219, %212
  br label %232

232:                                              ; preds = %231, %228
  br label %234

233:                                              ; preds = %193
  br label %234

234:                                              ; preds = %233, %232
  %235 = load ptr, ptr %9, align 8
  store ptr %235, ptr %18, align 8
  %236 = load ptr, ptr %8, align 8
  store ptr %236, ptr %19, align 8
  %237 = load ptr, ptr %18, align 8
  %238 = getelementptr inbounds %struct.TValue, ptr %237, i32 0, i32 0
  %239 = load ptr, ptr %19, align 8
  %240 = getelementptr inbounds %struct.TValue, ptr %239, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %238, ptr align 8 %240, i64 8, i1 false)
  %241 = load ptr, ptr %19, align 8
  %242 = getelementptr inbounds %struct.TValue, ptr %241, i32 0, i32 1
  %243 = load i8, ptr %242, align 8
  %244 = load ptr, ptr %18, align 8
  %245 = getelementptr inbounds %struct.TValue, ptr %244, i32 0, i32 1
  store i8 %243, ptr %245, align 8
  %246 = load ptr, ptr %5, align 8
  br label %247

247:                                              ; preds = %234, %95, %74
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaH_set(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = call ptr @luaH_get(ptr noundef %10, ptr noundef %11)
  store ptr %12, ptr %9, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = load ptr, ptr %8, align 8
  call void @luaH_finishset(ptr noundef %13, ptr noundef %14, ptr noundef %15, ptr noundef %16, ptr noundef %17)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaH_getn(ptr noundef %0) #0 {
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.Table, ptr %7, i32 0, i32 5
  %9 = load i32, ptr %8, align 4
  store i32 %9, ptr %4, align 4
  %10 = load i32, ptr %4, align 4
  %11 = icmp ugt i32 %10, 0
  br i1 %11, label %12, label %96

12:                                               ; preds = %1
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  %16 = load i32, ptr %4, align 4
  %17 = sub i32 %16, 1
  %18 = zext i32 %17 to i64
  %19 = getelementptr inbounds %struct.TValue, ptr %15, i64 %18
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 1
  %21 = load i8, ptr %20, align 8
  %22 = zext i8 %21 to i32
  %23 = and i32 %22, 15
  %24 = icmp eq i32 %23, 0
  br i1 %24, label %25, label %96

25:                                               ; preds = %12
  %26 = load i32, ptr %4, align 4
  %27 = icmp uge i32 %26, 2
  br i1 %27, label %28, label %68

28:                                               ; preds = %25
  %29 = load ptr, ptr %3, align 8
  %30 = getelementptr inbounds %struct.Table, ptr %29, i32 0, i32 6
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %4, align 4
  %33 = sub i32 %32, 2
  %34 = zext i32 %33 to i64
  %35 = getelementptr inbounds %struct.TValue, ptr %31, i64 %34
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = zext i8 %37 to i32
  %39 = and i32 %38, 15
  %40 = icmp eq i32 %39, 0
  br i1 %40, label %68, label %41

41:                                               ; preds = %28
  %42 = load ptr, ptr %3, align 8
  %43 = call i32 @ispow2realasize(ptr noundef %42)
  %44 = icmp ne i32 %43, 0
  br i1 %44, label %45, label %64

45:                                               ; preds = %41
  %46 = load i32, ptr %4, align 4
  %47 = sub i32 %46, 1
  %48 = load i32, ptr %4, align 4
  %49 = sub i32 %48, 1
  %50 = sub i32 %49, 1
  %51 = and i32 %47, %50
  %52 = icmp eq i32 %51, 0
  br i1 %52, label %64, label %53

53:                                               ; preds = %45
  %54 = load i32, ptr %4, align 4
  %55 = sub i32 %54, 1
  %56 = load ptr, ptr %3, align 8
  %57 = getelementptr inbounds %struct.Table, ptr %56, i32 0, i32 5
  store i32 %55, ptr %57, align 4
  %58 = load ptr, ptr %3, align 8
  %59 = getelementptr inbounds %struct.Table, ptr %58, i32 0, i32 3
  %60 = load i8, ptr %59, align 2
  %61 = zext i8 %60 to i32
  %62 = or i32 %61, 128
  %63 = trunc i32 %62 to i8
  store i8 %63, ptr %59, align 2
  br label %64

64:                                               ; preds = %53, %45, %41
  %65 = load i32, ptr %4, align 4
  %66 = sub i32 %65, 1
  %67 = zext i32 %66 to i64
  store i64 %67, ptr %2, align 8
  br label %182

68:                                               ; preds = %28, %25
  %69 = load ptr, ptr %3, align 8
  %70 = getelementptr inbounds %struct.Table, ptr %69, i32 0, i32 6
  %71 = load ptr, ptr %70, align 8
  %72 = load i32, ptr %4, align 4
  %73 = call i32 @binsearch(ptr noundef %71, i32 noundef 0, i32 noundef %72)
  store i32 %73, ptr %5, align 4
  %74 = load ptr, ptr %3, align 8
  %75 = call i32 @ispow2realasize(ptr noundef %74)
  %76 = icmp ne i32 %75, 0
  br i1 %76, label %77, label %93

77:                                               ; preds = %68
  %78 = load i32, ptr %5, align 4
  %79 = load ptr, ptr %3, align 8
  %80 = call i32 @luaH_realasize(ptr noundef %79)
  %81 = udiv i32 %80, 2
  %82 = icmp ugt i32 %78, %81
  br i1 %82, label %83, label %93

83:                                               ; preds = %77
  %84 = load i32, ptr %5, align 4
  %85 = load ptr, ptr %3, align 8
  %86 = getelementptr inbounds %struct.Table, ptr %85, i32 0, i32 5
  store i32 %84, ptr %86, align 4
  %87 = load ptr, ptr %3, align 8
  %88 = getelementptr inbounds %struct.Table, ptr %87, i32 0, i32 3
  %89 = load i8, ptr %88, align 2
  %90 = zext i8 %89 to i32
  %91 = or i32 %90, 128
  %92 = trunc i32 %91 to i8
  store i8 %92, ptr %88, align 2
  br label %93

93:                                               ; preds = %83, %77, %68
  %94 = load i32, ptr %5, align 4
  %95 = zext i32 %94 to i64
  store i64 %95, ptr %2, align 8
  br label %182

96:                                               ; preds = %12, %1
  %97 = load ptr, ptr %3, align 8
  %98 = getelementptr inbounds %struct.Table, ptr %97, i32 0, i32 3
  %99 = load i8, ptr %98, align 2
  %100 = zext i8 %99 to i32
  %101 = and i32 %100, 128
  %102 = icmp ne i32 %101, 0
  br i1 %102, label %103, label %158

103:                                              ; preds = %96
  %104 = load ptr, ptr %3, align 8
  %105 = getelementptr inbounds %struct.Table, ptr %104, i32 0, i32 5
  %106 = load i32, ptr %105, align 4
  %107 = load ptr, ptr %3, align 8
  %108 = getelementptr inbounds %struct.Table, ptr %107, i32 0, i32 5
  %109 = load i32, ptr %108, align 4
  %110 = sub i32 %109, 1
  %111 = and i32 %106, %110
  %112 = icmp eq i32 %111, 0
  br i1 %112, label %158, label %113

113:                                              ; preds = %103
  %114 = load ptr, ptr %3, align 8
  %115 = getelementptr inbounds %struct.Table, ptr %114, i32 0, i32 6
  %116 = load ptr, ptr %115, align 8
  %117 = load i32, ptr %4, align 4
  %118 = zext i32 %117 to i64
  %119 = getelementptr inbounds %struct.TValue, ptr %116, i64 %118
  %120 = getelementptr inbounds %struct.TValue, ptr %119, i32 0, i32 1
  %121 = load i8, ptr %120, align 8
  %122 = zext i8 %121 to i32
  %123 = and i32 %122, 15
  %124 = icmp eq i32 %123, 0
  br i1 %124, label %125, label %128

125:                                              ; preds = %113
  %126 = load i32, ptr %4, align 4
  %127 = zext i32 %126 to i64
  store i64 %127, ptr %2, align 8
  br label %182

128:                                              ; preds = %113
  %129 = load ptr, ptr %3, align 8
  %130 = call i32 @luaH_realasize(ptr noundef %129)
  store i32 %130, ptr %4, align 4
  %131 = load ptr, ptr %3, align 8
  %132 = getelementptr inbounds %struct.Table, ptr %131, i32 0, i32 6
  %133 = load ptr, ptr %132, align 8
  %134 = load i32, ptr %4, align 4
  %135 = sub i32 %134, 1
  %136 = zext i32 %135 to i64
  %137 = getelementptr inbounds %struct.TValue, ptr %133, i64 %136
  %138 = getelementptr inbounds %struct.TValue, ptr %137, i32 0, i32 1
  %139 = load i8, ptr %138, align 8
  %140 = zext i8 %139 to i32
  %141 = and i32 %140, 15
  %142 = icmp eq i32 %141, 0
  br i1 %142, label %143, label %157

143:                                              ; preds = %128
  %144 = load ptr, ptr %3, align 8
  %145 = getelementptr inbounds %struct.Table, ptr %144, i32 0, i32 6
  %146 = load ptr, ptr %145, align 8
  %147 = load ptr, ptr %3, align 8
  %148 = getelementptr inbounds %struct.Table, ptr %147, i32 0, i32 5
  %149 = load i32, ptr %148, align 4
  %150 = load i32, ptr %4, align 4
  %151 = call i32 @binsearch(ptr noundef %146, i32 noundef %149, i32 noundef %150)
  store i32 %151, ptr %6, align 4
  %152 = load i32, ptr %6, align 4
  %153 = load ptr, ptr %3, align 8
  %154 = getelementptr inbounds %struct.Table, ptr %153, i32 0, i32 5
  store i32 %152, ptr %154, align 4
  %155 = load i32, ptr %6, align 4
  %156 = zext i32 %155 to i64
  store i64 %156, ptr %2, align 8
  br label %182

157:                                              ; preds = %128
  br label %158

158:                                              ; preds = %157, %103, %96
  %159 = load ptr, ptr %3, align 8
  %160 = getelementptr inbounds %struct.Table, ptr %159, i32 0, i32 8
  %161 = load ptr, ptr %160, align 8
  %162 = icmp eq ptr %161, null
  br i1 %162, label %174, label %163

163:                                              ; preds = %158
  %164 = load ptr, ptr %3, align 8
  %165 = load i32, ptr %4, align 4
  %166 = add i32 %165, 1
  %167 = zext i32 %166 to i64
  %168 = call ptr @luaH_getint(ptr noundef %164, i64 noundef %167)
  %169 = getelementptr inbounds %struct.TValue, ptr %168, i32 0, i32 1
  %170 = load i8, ptr %169, align 8
  %171 = zext i8 %170 to i32
  %172 = and i32 %171, 15
  %173 = icmp eq i32 %172, 0
  br i1 %173, label %174, label %177

174:                                              ; preds = %163, %158
  %175 = load i32, ptr %4, align 4
  %176 = zext i32 %175 to i64
  store i64 %176, ptr %2, align 8
  br label %182

177:                                              ; preds = %163
  %178 = load ptr, ptr %3, align 8
  %179 = load i32, ptr %4, align 4
  %180 = zext i32 %179 to i64
  %181 = call i64 @hash_search(ptr noundef %178, i64 noundef %180)
  store i64 %181, ptr %2, align 8
  br label %182

182:                                              ; preds = %177, %174, %143, %125, %93, %64
  %183 = load i64, ptr %2, align 8
  ret i64 %183
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @ispow2realasize(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.Table, ptr %3, i32 0, i32 3
  %5 = load i8, ptr %4, align 2
  %6 = zext i8 %5 to i32
  %7 = and i32 %6, 128
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %19, label %9

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.Table, ptr %10, i32 0, i32 5
  %12 = load i32, ptr %11, align 4
  %13 = load ptr, ptr %2, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 5
  %15 = load i32, ptr %14, align 4
  %16 = sub i32 %15, 1
  %17 = and i32 %12, %16
  %18 = icmp eq i32 %17, 0
  br label %19

19:                                               ; preds = %9, %1
  %20 = phi i1 [ true, %1 ], [ %18, %9 ]
  %21 = zext i1 %20 to i32
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @binsearch(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  br label %8

8:                                                ; preds = %32, %3
  %9 = load i32, ptr %6, align 4
  %10 = load i32, ptr %5, align 4
  %11 = sub i32 %9, %10
  %12 = icmp ugt i32 %11, 1
  br i1 %12, label %13, label %33

13:                                               ; preds = %8
  %14 = load i32, ptr %5, align 4
  %15 = load i32, ptr %6, align 4
  %16 = add i32 %14, %15
  %17 = udiv i32 %16, 2
  store i32 %17, ptr %7, align 4
  %18 = load ptr, ptr %4, align 8
  %19 = load i32, ptr %7, align 4
  %20 = sub i32 %19, 1
  %21 = zext i32 %20 to i64
  %22 = getelementptr inbounds %struct.TValue, ptr %18, i64 %21
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = and i32 %25, 15
  %27 = icmp eq i32 %26, 0
  br i1 %27, label %28, label %30

28:                                               ; preds = %13
  %29 = load i32, ptr %7, align 4
  store i32 %29, ptr %6, align 4
  br label %32

30:                                               ; preds = %13
  %31 = load i32, ptr %7, align 4
  store i32 %31, ptr %5, align 4
  br label %32

32:                                               ; preds = %30, %28
  br label %8, !llvm.loop !14

33:                                               ; preds = %8
  %34 = load i32, ptr %5, align 4
  ret i32 %34
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @hash_search(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %8 = load i64, ptr %5, align 8
  %9 = icmp eq i64 %8, 0
  br i1 %9, label %10, label %13

10:                                               ; preds = %2
  %11 = load i64, ptr %5, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %5, align 8
  br label %13

13:                                               ; preds = %10, %2
  br label %14

14:                                               ; preds = %34, %13
  %15 = load i64, ptr %5, align 8
  store i64 %15, ptr %6, align 8
  %16 = load i64, ptr %5, align 8
  %17 = icmp ule i64 %16, 4611686018427387903
  br i1 %17, label %18, label %21

18:                                               ; preds = %14
  %19 = load i64, ptr %5, align 8
  %20 = mul i64 %19, 2
  store i64 %20, ptr %5, align 8
  br label %33

21:                                               ; preds = %14
  store i64 9223372036854775807, ptr %5, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = load i64, ptr %5, align 8
  %24 = call ptr @luaH_getint(ptr noundef %22, i64 noundef %23)
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = zext i8 %26 to i32
  %28 = and i32 %27, 15
  %29 = icmp eq i32 %28, 0
  br i1 %29, label %30, label %31

30:                                               ; preds = %21
  br label %44

31:                                               ; preds = %21
  %32 = load i64, ptr %5, align 8
  store i64 %32, ptr %3, align 8
  br label %70

33:                                               ; preds = %18
  br label %34

34:                                               ; preds = %33
  %35 = load ptr, ptr %4, align 8
  %36 = load i64, ptr %5, align 8
  %37 = call ptr @luaH_getint(ptr noundef %35, i64 noundef %36)
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 1
  %39 = load i8, ptr %38, align 8
  %40 = zext i8 %39 to i32
  %41 = and i32 %40, 15
  %42 = icmp eq i32 %41, 0
  %43 = xor i1 %42, true
  br i1 %43, label %14, label %44, !llvm.loop !15

44:                                               ; preds = %34, %30
  br label %45

45:                                               ; preds = %67, %44
  %46 = load i64, ptr %5, align 8
  %47 = load i64, ptr %6, align 8
  %48 = sub i64 %46, %47
  %49 = icmp ugt i64 %48, 1
  br i1 %49, label %50, label %68

50:                                               ; preds = %45
  %51 = load i64, ptr %6, align 8
  %52 = load i64, ptr %5, align 8
  %53 = add i64 %51, %52
  %54 = udiv i64 %53, 2
  store i64 %54, ptr %7, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = load i64, ptr %7, align 8
  %57 = call ptr @luaH_getint(ptr noundef %55, i64 noundef %56)
  %58 = getelementptr inbounds %struct.TValue, ptr %57, i32 0, i32 1
  %59 = load i8, ptr %58, align 8
  %60 = zext i8 %59 to i32
  %61 = and i32 %60, 15
  %62 = icmp eq i32 %61, 0
  br i1 %62, label %63, label %65

63:                                               ; preds = %50
  %64 = load i64, ptr %7, align 8
  store i64 %64, ptr %5, align 8
  br label %67

65:                                               ; preds = %50
  %66 = load i64, ptr %7, align 8
  store i64 %66, ptr %6, align 8
  br label %67

67:                                               ; preds = %65, %63
  br label %45, !llvm.loop !16

68:                                               ; preds = %45
  %69 = load i64, ptr %6, align 8
  store i64 %69, ptr %3, align 8
  br label %70

70:                                               ; preds = %68, %31
  %71 = load i64, ptr %3, align 8
  ret i64 %71
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arrayindex(i64 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i64, align 8
  store i64 %0, ptr %3, align 8
  %4 = load i64, ptr %3, align 8
  %5 = sub i64 %4, 1
  %6 = icmp ult i64 %5, 2147483648
  br i1 %6, label %7, label %10

7:                                                ; preds = %1
  %8 = load i64, ptr %3, align 8
  %9 = trunc i64 %8 to i32
  store i32 %9, ptr %2, align 4
  br label %11

10:                                               ; preds = %1
  store i32 0, ptr %2, align 4
  br label %11

11:                                               ; preds = %10, %7
  %12 = load i32, ptr %2, align 4
  ret i32 %12
}

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #3

declare hidden i32 @luaO_ceillog2(i32 noundef) #2

declare hidden ptr @luaM_malloc_(ptr noundef, i64 noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @mainpositionTV(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca double, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 63
  switch i32 %17, label %149 [
    i32 3, label %18
    i32 19, label %25
    i32 4, label %44
    i32 20, label %63
    i32 1, label %81
    i32 17, label %94
    i32 2, label %107
    i32 22, label %128
  ]

18:                                               ; preds = %2
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load i64, ptr %20, align 8
  store i64 %21, ptr %6, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = load i64, ptr %6, align 8
  %24 = call ptr @hashint(ptr noundef %22, i64 noundef %23)
  store ptr %24, ptr %3, align 8
  br label %170

25:                                               ; preds = %2
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load double, ptr %27, align 8
  store double %28, ptr %7, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.Table, ptr %29, i32 0, i32 7
  %31 = load ptr, ptr %30, align 8
  %32 = load double, ptr %7, align 8
  %33 = call i32 @l_hashfloat(double noundef %32)
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.Table, ptr %34, i32 0, i32 4
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = shl i32 1, %37
  %39 = sub nsw i32 %38, 1
  %40 = or i32 %39, 1
  %41 = srem i32 %33, %40
  %42 = sext i32 %41 to i64
  %43 = getelementptr inbounds %union.Node, ptr %31, i64 %42
  store ptr %43, ptr %3, align 8
  br label %170

44:                                               ; preds = %2
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.TValue, ptr %45, i32 0, i32 0
  %47 = load ptr, ptr %46, align 8
  store ptr %47, ptr %8, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.Table, ptr %48, i32 0, i32 7
  %50 = load ptr, ptr %49, align 8
  %51 = load ptr, ptr %8, align 8
  %52 = getelementptr inbounds %struct.TString, ptr %51, i32 0, i32 5
  %53 = load i32, ptr %52, align 4
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.Table, ptr %54, i32 0, i32 4
  %56 = load i8, ptr %55, align 1
  %57 = zext i8 %56 to i32
  %58 = shl i32 1, %57
  %59 = sub nsw i32 %58, 1
  %60 = and i32 %53, %59
  %61 = sext i32 %60 to i64
  %62 = getelementptr inbounds %union.Node, ptr %50, i64 %61
  store ptr %62, ptr %3, align 8
  br label %170

63:                                               ; preds = %2
  %64 = load ptr, ptr %5, align 8
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 0
  %66 = load ptr, ptr %65, align 8
  store ptr %66, ptr %9, align 8
  %67 = load ptr, ptr %4, align 8
  %68 = getelementptr inbounds %struct.Table, ptr %67, i32 0, i32 7
  %69 = load ptr, ptr %68, align 8
  %70 = load ptr, ptr %9, align 8
  %71 = call i32 @luaS_hashlongstr(ptr noundef %70)
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.Table, ptr %72, i32 0, i32 4
  %74 = load i8, ptr %73, align 1
  %75 = zext i8 %74 to i32
  %76 = shl i32 1, %75
  %77 = sub nsw i32 %76, 1
  %78 = and i32 %71, %77
  %79 = sext i32 %78 to i64
  %80 = getelementptr inbounds %union.Node, ptr %69, i64 %79
  store ptr %80, ptr %3, align 8
  br label %170

81:                                               ; preds = %2
  %82 = load ptr, ptr %4, align 8
  %83 = getelementptr inbounds %struct.Table, ptr %82, i32 0, i32 7
  %84 = load ptr, ptr %83, align 8
  %85 = load ptr, ptr %4, align 8
  %86 = getelementptr inbounds %struct.Table, ptr %85, i32 0, i32 4
  %87 = load i8, ptr %86, align 1
  %88 = zext i8 %87 to i32
  %89 = shl i32 1, %88
  %90 = sub nsw i32 %89, 1
  %91 = and i32 0, %90
  %92 = sext i32 %91 to i64
  %93 = getelementptr inbounds %union.Node, ptr %84, i64 %92
  store ptr %93, ptr %3, align 8
  br label %170

94:                                               ; preds = %2
  %95 = load ptr, ptr %4, align 8
  %96 = getelementptr inbounds %struct.Table, ptr %95, i32 0, i32 7
  %97 = load ptr, ptr %96, align 8
  %98 = load ptr, ptr %4, align 8
  %99 = getelementptr inbounds %struct.Table, ptr %98, i32 0, i32 4
  %100 = load i8, ptr %99, align 1
  %101 = zext i8 %100 to i32
  %102 = shl i32 1, %101
  %103 = sub nsw i32 %102, 1
  %104 = and i32 1, %103
  %105 = sext i32 %104 to i64
  %106 = getelementptr inbounds %union.Node, ptr %97, i64 %105
  store ptr %106, ptr %3, align 8
  br label %170

107:                                              ; preds = %2
  %108 = load ptr, ptr %5, align 8
  %109 = getelementptr inbounds %struct.TValue, ptr %108, i32 0, i32 0
  %110 = load ptr, ptr %109, align 8
  store ptr %110, ptr %10, align 8
  %111 = load ptr, ptr %4, align 8
  %112 = getelementptr inbounds %struct.Table, ptr %111, i32 0, i32 7
  %113 = load ptr, ptr %112, align 8
  %114 = load ptr, ptr %10, align 8
  %115 = ptrtoint ptr %114 to i64
  %116 = and i64 %115, 4294967295
  %117 = trunc i64 %116 to i32
  %118 = load ptr, ptr %4, align 8
  %119 = getelementptr inbounds %struct.Table, ptr %118, i32 0, i32 4
  %120 = load i8, ptr %119, align 1
  %121 = zext i8 %120 to i32
  %122 = shl i32 1, %121
  %123 = sub nsw i32 %122, 1
  %124 = or i32 %123, 1
  %125 = urem i32 %117, %124
  %126 = zext i32 %125 to i64
  %127 = getelementptr inbounds %union.Node, ptr %113, i64 %126
  store ptr %127, ptr %3, align 8
  br label %170

128:                                              ; preds = %2
  %129 = load ptr, ptr %5, align 8
  %130 = getelementptr inbounds %struct.TValue, ptr %129, i32 0, i32 0
  %131 = load ptr, ptr %130, align 8
  store ptr %131, ptr %11, align 8
  %132 = load ptr, ptr %4, align 8
  %133 = getelementptr inbounds %struct.Table, ptr %132, i32 0, i32 7
  %134 = load ptr, ptr %133, align 8
  %135 = load ptr, ptr %11, align 8
  %136 = ptrtoint ptr %135 to i64
  %137 = and i64 %136, 4294967295
  %138 = trunc i64 %137 to i32
  %139 = load ptr, ptr %4, align 8
  %140 = getelementptr inbounds %struct.Table, ptr %139, i32 0, i32 4
  %141 = load i8, ptr %140, align 1
  %142 = zext i8 %141 to i32
  %143 = shl i32 1, %142
  %144 = sub nsw i32 %143, 1
  %145 = or i32 %144, 1
  %146 = urem i32 %138, %145
  %147 = zext i32 %146 to i64
  %148 = getelementptr inbounds %union.Node, ptr %134, i64 %147
  store ptr %148, ptr %3, align 8
  br label %170

149:                                              ; preds = %2
  %150 = load ptr, ptr %5, align 8
  %151 = getelementptr inbounds %struct.TValue, ptr %150, i32 0, i32 0
  %152 = load ptr, ptr %151, align 8
  store ptr %152, ptr %12, align 8
  %153 = load ptr, ptr %4, align 8
  %154 = getelementptr inbounds %struct.Table, ptr %153, i32 0, i32 7
  %155 = load ptr, ptr %154, align 8
  %156 = load ptr, ptr %12, align 8
  %157 = ptrtoint ptr %156 to i64
  %158 = and i64 %157, 4294967295
  %159 = trunc i64 %158 to i32
  %160 = load ptr, ptr %4, align 8
  %161 = getelementptr inbounds %struct.Table, ptr %160, i32 0, i32 4
  %162 = load i8, ptr %161, align 1
  %163 = zext i8 %162 to i32
  %164 = shl i32 1, %163
  %165 = sub nsw i32 %164, 1
  %166 = or i32 %165, 1
  %167 = urem i32 %159, %166
  %168 = zext i32 %167 to i64
  %169 = getelementptr inbounds %union.Node, ptr %155, i64 %168
  store ptr %169, ptr %3, align 8
  br label %170

170:                                              ; preds = %149, %128, %107, %94, %81, %63, %44, %25, %18
  %171 = load ptr, ptr %3, align 8
  ret ptr %171
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @equalkey(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.NodeKey, ptr %12, i32 0, i32 2
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = icmp ne i32 %11, %15
  br i1 %16, label %17, label %34

17:                                               ; preds = %3
  %18 = load i32, ptr %7, align 4
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %33

20:                                               ; preds = %17
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.NodeKey, ptr %21, i32 0, i32 2
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = icmp eq i32 %24, 11
  br i1 %25, label %26, label %33

26:                                               ; preds = %20
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 64
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %34, label %33

33:                                               ; preds = %26, %20, %17
  store i32 0, ptr %4, align 4
  br label %93

34:                                               ; preds = %26, %3
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.NodeKey, ptr %35, i32 0, i32 2
  %37 = load i8, ptr %36, align 1
  %38 = zext i8 %37 to i32
  switch i32 %38, label %84 [
    i32 0, label %39
    i32 1, label %39
    i32 17, label %39
    i32 3, label %40
    i32 19, label %49
    i32 2, label %58
    i32 22, label %67
    i32 84, label %76
  ]

39:                                               ; preds = %34, %34, %34
  store i32 1, ptr %4, align 4
  br label %93

40:                                               ; preds = %34
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  %43 = load i64, ptr %42, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = getelementptr inbounds %struct.NodeKey, ptr %44, i32 0, i32 4
  %46 = load i64, ptr %45, align 8
  %47 = icmp eq i64 %43, %46
  %48 = zext i1 %47 to i32
  store i32 %48, ptr %4, align 4
  br label %93

49:                                               ; preds = %34
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 0
  %52 = load double, ptr %51, align 8
  %53 = load ptr, ptr %6, align 8
  %54 = getelementptr inbounds %struct.NodeKey, ptr %53, i32 0, i32 4
  %55 = load double, ptr %54, align 8
  %56 = fcmp oeq double %52, %55
  %57 = zext i1 %56 to i32
  store i32 %57, ptr %4, align 4
  br label %93

58:                                               ; preds = %34
  %59 = load ptr, ptr %5, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 0
  %61 = load ptr, ptr %60, align 8
  %62 = load ptr, ptr %6, align 8
  %63 = getelementptr inbounds %struct.NodeKey, ptr %62, i32 0, i32 4
  %64 = load ptr, ptr %63, align 8
  %65 = icmp eq ptr %61, %64
  %66 = zext i1 %65 to i32
  store i32 %66, ptr %4, align 4
  br label %93

67:                                               ; preds = %34
  %68 = load ptr, ptr %5, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  %70 = load ptr, ptr %69, align 8
  %71 = load ptr, ptr %6, align 8
  %72 = getelementptr inbounds %struct.NodeKey, ptr %71, i32 0, i32 4
  %73 = load ptr, ptr %72, align 8
  %74 = icmp eq ptr %70, %73
  %75 = zext i1 %74 to i32
  store i32 %75, ptr %4, align 4
  br label %93

76:                                               ; preds = %34
  %77 = load ptr, ptr %5, align 8
  %78 = getelementptr inbounds %struct.TValue, ptr %77, i32 0, i32 0
  %79 = load ptr, ptr %78, align 8
  %80 = load ptr, ptr %6, align 8
  %81 = getelementptr inbounds %struct.NodeKey, ptr %80, i32 0, i32 4
  %82 = load ptr, ptr %81, align 8
  %83 = call i32 @luaS_eqlngstr(ptr noundef %79, ptr noundef %82)
  store i32 %83, ptr %4, align 4
  br label %93

84:                                               ; preds = %34
  %85 = load ptr, ptr %5, align 8
  %86 = getelementptr inbounds %struct.TValue, ptr %85, i32 0, i32 0
  %87 = load ptr, ptr %86, align 8
  %88 = load ptr, ptr %6, align 8
  %89 = getelementptr inbounds %struct.NodeKey, ptr %88, i32 0, i32 4
  %90 = load ptr, ptr %89, align 8
  %91 = icmp eq ptr %87, %90
  %92 = zext i1 %91 to i32
  store i32 %92, ptr %4, align 4
  br label %93

93:                                               ; preds = %84, %76, %67, %58, %49, %40, %39, %33
  %94 = load i32, ptr %4, align 4
  ret i32 %94
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @l_hashfloat(double noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca double, align 8
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  store double %0, ptr %3, align 8
  %7 = load double, ptr %3, align 8
  %8 = call double @frexp(double noundef %7, ptr noundef %4) #6
  %9 = fmul double %8, 0x41E0000000000000
  store double %9, ptr %3, align 8
  %10 = load double, ptr %3, align 8
  %11 = fcmp oge double %10, 0xC3E0000000000000
  br i1 %11, label %12, label %18

12:                                               ; preds = %1
  %13 = load double, ptr %3, align 8
  %14 = fcmp olt double %13, 0x43E0000000000000
  br i1 %14, label %15, label %18

15:                                               ; preds = %12
  %16 = load double, ptr %3, align 8
  %17 = fptosi double %16 to i64
  store i64 %17, ptr %5, align 8
  br i1 true, label %19, label %18

18:                                               ; preds = %15, %12, %1
  store i32 0, ptr %2, align 4
  br label %33

19:                                               ; preds = %15
  %20 = load i32, ptr %4, align 4
  %21 = load i64, ptr %5, align 8
  %22 = trunc i64 %21 to i32
  %23 = add i32 %20, %22
  store i32 %23, ptr %6, align 4
  %24 = load i32, ptr %6, align 4
  %25 = icmp ule i32 %24, 2147483647
  br i1 %25, label %26, label %28

26:                                               ; preds = %19
  %27 = load i32, ptr %6, align 4
  br label %31

28:                                               ; preds = %19
  %29 = load i32, ptr %6, align 4
  %30 = xor i32 %29, -1
  br label %31

31:                                               ; preds = %28, %26
  %32 = phi i32 [ %27, %26 ], [ %30, %28 ]
  store i32 %32, ptr %2, align 4
  br label %33

33:                                               ; preds = %31, %18
  %34 = load i32, ptr %2, align 4
  ret i32 %34
}

declare hidden i32 @luaS_hashlongstr(ptr noundef) #2

; Function Attrs: nounwind
declare double @frexp(double noundef, ptr noundef) #4

declare hidden i32 @luaS_eqlngstr(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getfreepos(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = getelementptr inbounds %struct.Table, ptr %4, i32 0, i32 8
  %6 = load ptr, ptr %5, align 8
  %7 = icmp eq ptr %6, null
  br i1 %7, label %35, label %8

8:                                                ; preds = %1
  br label %9

9:                                                ; preds = %33, %8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.Table, ptr %10, i32 0, i32 8
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  %16 = icmp ugt ptr %12, %15
  br i1 %16, label %17, label %34

17:                                               ; preds = %9
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.Table, ptr %18, i32 0, i32 8
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %union.Node, ptr %20, i32 -1
  store ptr %21, ptr %19, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 8
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds %struct.NodeKey, ptr %24, i32 0, i32 2
  %26 = load i8, ptr %25, align 1
  %27 = zext i8 %26 to i32
  %28 = icmp eq i32 %27, 0
  br i1 %28, label %29, label %33

29:                                               ; preds = %17
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.Table, ptr %30, i32 0, i32 8
  %32 = load ptr, ptr %31, align 8
  store ptr %32, ptr %2, align 8
  br label %36

33:                                               ; preds = %17
  br label %9, !llvm.loop !17

34:                                               ; preds = %9
  br label %35

35:                                               ; preds = %34, %1
  store ptr null, ptr %2, align 8
  br label %36

36:                                               ; preds = %35, %29
  %37 = load ptr, ptr %2, align 8
  ret ptr %37
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @rehash(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca [32 x i32], align 16
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  store i32 0, ptr %10, align 4
  br label %12

12:                                               ; preds = %19, %3
  %13 = load i32, ptr %10, align 4
  %14 = icmp sle i32 %13, 31
  br i1 %14, label %15, label %22

15:                                               ; preds = %12
  %16 = load i32, ptr %10, align 4
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds [32 x i32], ptr %9, i64 0, i64 %17
  store i32 0, ptr %18, align 4
  br label %19

19:                                               ; preds = %15
  %20 = load i32, ptr %10, align 4
  %21 = add nsw i32 %20, 1
  store i32 %21, ptr %10, align 4
  br label %12, !llvm.loop !18

22:                                               ; preds = %12
  %23 = load ptr, ptr %5, align 8
  %24 = call i32 @setlimittosize(ptr noundef %23)
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds [32 x i32], ptr %9, i64 0, i64 0
  %27 = call i32 @numusearray(ptr noundef %25, ptr noundef %26)
  store i32 %27, ptr %8, align 4
  %28 = load i32, ptr %8, align 4
  store i32 %28, ptr %11, align 4
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds [32 x i32], ptr %9, i64 0, i64 0
  %31 = call i32 @numusehash(ptr noundef %29, ptr noundef %30, ptr noundef %8)
  %32 = load i32, ptr %11, align 4
  %33 = add nsw i32 %32, %31
  store i32 %33, ptr %11, align 4
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  %36 = load i8, ptr %35, align 8
  %37 = zext i8 %36 to i32
  %38 = icmp eq i32 %37, 3
  br i1 %38, label %39, label %47

39:                                               ; preds = %22
  %40 = load ptr, ptr %6, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load i64, ptr %41, align 8
  %43 = getelementptr inbounds [32 x i32], ptr %9, i64 0, i64 0
  %44 = call i32 @countint(i64 noundef %42, ptr noundef %43)
  %45 = load i32, ptr %8, align 4
  %46 = add i32 %45, %44
  store i32 %46, ptr %8, align 4
  br label %47

47:                                               ; preds = %39, %22
  %48 = load i32, ptr %11, align 4
  %49 = add nsw i32 %48, 1
  store i32 %49, ptr %11, align 4
  %50 = getelementptr inbounds [32 x i32], ptr %9, i64 0, i64 0
  %51 = call i32 @computesizes(ptr noundef %50, ptr noundef %8)
  store i32 %51, ptr %7, align 4
  %52 = load ptr, ptr %4, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = load i32, ptr %7, align 4
  %55 = load i32, ptr %11, align 4
  %56 = load i32, ptr %8, align 4
  %57 = sub i32 %55, %56
  call void @luaH_resize(ptr noundef %52, ptr noundef %53, i32 noundef %54, i32 noundef %57)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @mainpositionfromnode(ptr noundef %0, ptr noundef %1) #0 {
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
  %9 = load ptr, ptr %6, align 8
  %10 = getelementptr inbounds %struct.TValue, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %7, align 8
  %12 = getelementptr inbounds %struct.NodeKey, ptr %11, i32 0, i32 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %10, ptr align 8 %12, i64 8, i1 false)
  %13 = load ptr, ptr %7, align 8
  %14 = getelementptr inbounds %struct.NodeKey, ptr %13, i32 0, i32 2
  %15 = load i8, ptr %14, align 1
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  store i8 %15, ptr %17, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = call ptr @mainpositionTV(ptr noundef %18, ptr noundef %5)
  ret ptr %19
}

declare hidden void @luaC_barrierback_(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @numusearray(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %7, align 4
  store i32 1, ptr %8, align 4
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.Table, ptr %12, i32 0, i32 5
  %14 = load i32, ptr %13, align 4
  store i32 %14, ptr %9, align 4
  store i32 0, ptr %5, align 4
  store i32 1, ptr %6, align 4
  br label %15

15:                                               ; preds = %66, %2
  %16 = load i32, ptr %5, align 4
  %17 = icmp sle i32 %16, 31
  br i1 %17, label %18, label %71

18:                                               ; preds = %15
  store i32 0, ptr %10, align 4
  %19 = load i32, ptr %6, align 4
  store i32 %19, ptr %11, align 4
  %20 = load i32, ptr %11, align 4
  %21 = load i32, ptr %9, align 4
  %22 = icmp ugt i32 %20, %21
  br i1 %22, label %23, label %30

23:                                               ; preds = %18
  %24 = load i32, ptr %9, align 4
  store i32 %24, ptr %11, align 4
  %25 = load i32, ptr %8, align 4
  %26 = load i32, ptr %11, align 4
  %27 = icmp ugt i32 %25, %26
  br i1 %27, label %28, label %29

28:                                               ; preds = %23
  br label %71

29:                                               ; preds = %23
  br label %30

30:                                               ; preds = %29, %18
  br label %31

31:                                               ; preds = %52, %30
  %32 = load i32, ptr %8, align 4
  %33 = load i32, ptr %11, align 4
  %34 = icmp ule i32 %32, %33
  br i1 %34, label %35, label %55

35:                                               ; preds = %31
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.Table, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %8, align 4
  %40 = sub i32 %39, 1
  %41 = zext i32 %40 to i64
  %42 = getelementptr inbounds %struct.TValue, ptr %38, i64 %41
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 1
  %44 = load i8, ptr %43, align 8
  %45 = zext i8 %44 to i32
  %46 = and i32 %45, 15
  %47 = icmp eq i32 %46, 0
  br i1 %47, label %51, label %48

48:                                               ; preds = %35
  %49 = load i32, ptr %10, align 4
  %50 = add i32 %49, 1
  store i32 %50, ptr %10, align 4
  br label %51

51:                                               ; preds = %48, %35
  br label %52

52:                                               ; preds = %51
  %53 = load i32, ptr %8, align 4
  %54 = add i32 %53, 1
  store i32 %54, ptr %8, align 4
  br label %31, !llvm.loop !19

55:                                               ; preds = %31
  %56 = load i32, ptr %10, align 4
  %57 = load ptr, ptr %4, align 8
  %58 = load i32, ptr %5, align 4
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds i32, ptr %57, i64 %59
  %61 = load i32, ptr %60, align 4
  %62 = add i32 %61, %56
  store i32 %62, ptr %60, align 4
  %63 = load i32, ptr %10, align 4
  %64 = load i32, ptr %7, align 4
  %65 = add i32 %64, %63
  store i32 %65, ptr %7, align 4
  br label %66

66:                                               ; preds = %55
  %67 = load i32, ptr %5, align 4
  %68 = add nsw i32 %67, 1
  store i32 %68, ptr %5, align 4
  %69 = load i32, ptr %6, align 4
  %70 = mul i32 %69, 2
  store i32 %70, ptr %6, align 4
  br label %15, !llvm.loop !20

71:                                               ; preds = %28, %15
  %72 = load i32, ptr %7, align 4
  ret i32 %72
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @numusehash(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  store i32 0, ptr %7, align 4
  store i32 0, ptr %8, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 4
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = shl i32 1, %14
  store i32 %15, ptr %9, align 4
  br label %16

16:                                               ; preds = %50, %3
  %17 = load i32, ptr %9, align 4
  %18 = add nsw i32 %17, -1
  store i32 %18, ptr %9, align 4
  %19 = icmp ne i32 %17, 0
  br i1 %19, label %20, label %51

20:                                               ; preds = %16
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.Table, ptr %21, i32 0, i32 7
  %23 = load ptr, ptr %22, align 8
  %24 = load i32, ptr %9, align 4
  %25 = sext i32 %24 to i64
  %26 = getelementptr inbounds %union.Node, ptr %23, i64 %25
  store ptr %26, ptr %10, align 8
  %27 = load ptr, ptr %10, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 15
  %32 = icmp eq i32 %31, 0
  br i1 %32, label %50, label %33

33:                                               ; preds = %20
  %34 = load ptr, ptr %10, align 8
  %35 = getelementptr inbounds %struct.NodeKey, ptr %34, i32 0, i32 2
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = icmp eq i32 %37, 3
  br i1 %38, label %39, label %47

39:                                               ; preds = %33
  %40 = load ptr, ptr %10, align 8
  %41 = getelementptr inbounds %struct.NodeKey, ptr %40, i32 0, i32 4
  %42 = load i64, ptr %41, align 8
  %43 = load ptr, ptr %5, align 8
  %44 = call i32 @countint(i64 noundef %42, ptr noundef %43)
  %45 = load i32, ptr %8, align 4
  %46 = add nsw i32 %45, %44
  store i32 %46, ptr %8, align 4
  br label %47

47:                                               ; preds = %39, %33
  %48 = load i32, ptr %7, align 4
  %49 = add nsw i32 %48, 1
  store i32 %49, ptr %7, align 4
  br label %50

50:                                               ; preds = %47, %20
  br label %16, !llvm.loop !21

51:                                               ; preds = %16
  %52 = load i32, ptr %8, align 4
  %53 = load ptr, ptr %6, align 8
  %54 = load i32, ptr %53, align 4
  %55 = add i32 %54, %52
  store i32 %55, ptr %53, align 4
  %56 = load i32, ptr %7, align 4
  ret i32 %56
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @countint(i64 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store i64 %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load i64, ptr %4, align 8
  %8 = call i32 @arrayindex(i64 noundef %7)
  store i32 %8, ptr %6, align 4
  %9 = load i32, ptr %6, align 4
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %19

11:                                               ; preds = %2
  %12 = load ptr, ptr %5, align 8
  %13 = load i32, ptr %6, align 4
  %14 = call i32 @luaO_ceillog2(i32 noundef %13)
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds i32, ptr %12, i64 %15
  %17 = load i32, ptr %16, align 4
  %18 = add i32 %17, 1
  store i32 %18, ptr %16, align 4
  store i32 1, ptr %3, align 4
  br label %20

19:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %20

20:                                               ; preds = %19, %11
  %21 = load i32, ptr %3, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @computesizes(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %7, align 4
  store i32 0, ptr %8, align 4
  store i32 0, ptr %9, align 4
  store i32 0, ptr %5, align 4
  store i32 1, ptr %6, align 4
  br label %10

10:                                               ; preds = %37, %2
  %11 = load i32, ptr %6, align 4
  %12 = icmp ugt i32 %11, 0
  br i1 %12, label %13, label %19

13:                                               ; preds = %10
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %14, align 4
  %16 = load i32, ptr %6, align 4
  %17 = udiv i32 %16, 2
  %18 = icmp ugt i32 %15, %17
  br label %19

19:                                               ; preds = %13, %10
  %20 = phi i1 [ false, %10 ], [ %18, %13 ]
  br i1 %20, label %21, label %42

21:                                               ; preds = %19
  %22 = load ptr, ptr %3, align 8
  %23 = load i32, ptr %5, align 4
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds i32, ptr %22, i64 %24
  %26 = load i32, ptr %25, align 4
  %27 = load i32, ptr %7, align 4
  %28 = add i32 %27, %26
  store i32 %28, ptr %7, align 4
  %29 = load i32, ptr %7, align 4
  %30 = load i32, ptr %6, align 4
  %31 = udiv i32 %30, 2
  %32 = icmp ugt i32 %29, %31
  br i1 %32, label %33, label %36

33:                                               ; preds = %21
  %34 = load i32, ptr %6, align 4
  store i32 %34, ptr %9, align 4
  %35 = load i32, ptr %7, align 4
  store i32 %35, ptr %8, align 4
  br label %36

36:                                               ; preds = %33, %21
  br label %37

37:                                               ; preds = %36
  %38 = load i32, ptr %5, align 4
  %39 = add nsw i32 %38, 1
  store i32 %39, ptr %5, align 4
  %40 = load i32, ptr %6, align 4
  %41 = mul i32 %40, 2
  store i32 %41, ptr %6, align 4
  br label %10, !llvm.loop !22

42:                                               ; preds = %19
  %43 = load i32, ptr %8, align 4
  %44 = load ptr, ptr %4, align 8
  store i32 %43, ptr %44, align 4
  %45 = load i32, ptr %9, align 4
  ret i32 %45
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
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
