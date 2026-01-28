; ModuleID = 'lgc.c'
source_filename = "lgc.c"
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
%struct.GCObject = type { ptr, i8, i8 }
%struct.UpVal = type { ptr, i8, i8, %union.anon.4, %union.anon.5 }
%union.anon.4 = type { ptr }
%union.anon.5 = type { %struct.anon.6 }
%struct.anon.6 = type { ptr, ptr }
%struct.Udata = type { ptr, i8, i8, i16, i64, ptr, ptr, [1 x %union.UValue] }
%union.UValue = type { %struct.TValue }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.CClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x %struct.TValue] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.3, [1 x i8] }
%union.anon.3 = type { i64 }
%union.Node = type { %struct.NodeKey }
%struct.NodeKey = type { %union.Value, i8, i8, i32, %union.Value }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }
%struct.LocVar = type { ptr, i32, i32 }
%union.StackValue = type { %struct.TValue }

@.str = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@sweepgen.nextage = internal constant [7 x i8] c"\01\03\03\04\04\05\06", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_barrier_(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 11
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = icmp sle i32 %14, 2
  br i1 %15, label %16, label %36

16:                                               ; preds = %3
  %17 = load ptr, ptr %7, align 8
  %18 = load ptr, ptr %6, align 8
  call void @reallymarkobject(ptr noundef %17, ptr noundef %18)
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.GCObject, ptr %19, i32 0, i32 2
  %21 = load i8, ptr %20, align 1
  %22 = zext i8 %21 to i32
  %23 = and i32 %22, 7
  %24 = icmp sgt i32 %23, 1
  br i1 %24, label %25, label %35

25:                                               ; preds = %16
  %26 = load ptr, ptr %6, align 8
  %27 = getelementptr inbounds %struct.GCObject, ptr %26, i32 0, i32 2
  %28 = load i8, ptr %27, align 1
  %29 = zext i8 %28 to i32
  %30 = and i32 %29, -8
  %31 = or i32 %30, 2
  %32 = trunc i32 %31 to i8
  %33 = load ptr, ptr %6, align 8
  %34 = getelementptr inbounds %struct.GCObject, ptr %33, i32 0, i32 2
  store i8 %32, ptr %34, align 1
  br label %35

35:                                               ; preds = %25, %16
  br label %60

36:                                               ; preds = %3
  %37 = load ptr, ptr %7, align 8
  %38 = getelementptr inbounds %struct.global_State, ptr %37, i32 0, i32 12
  %39 = load i8, ptr %38, align 2
  %40 = zext i8 %39 to i32
  %41 = icmp eq i32 %40, 0
  br i1 %41, label %42, label %59

42:                                               ; preds = %36
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.GCObject, ptr %43, i32 0, i32 2
  %45 = load i8, ptr %44, align 1
  %46 = zext i8 %45 to i32
  %47 = and i32 %46, -57
  %48 = load ptr, ptr %7, align 8
  %49 = getelementptr inbounds %struct.global_State, ptr %48, i32 0, i32 10
  %50 = load i8, ptr %49, align 4
  %51 = zext i8 %50 to i32
  %52 = and i32 %51, 24
  %53 = trunc i32 %52 to i8
  %54 = zext i8 %53 to i32
  %55 = or i32 %47, %54
  %56 = trunc i32 %55 to i8
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.GCObject, ptr %57, i32 0, i32 2
  store i8 %56, ptr %58, align 1
  br label %59

59:                                               ; preds = %42, %36
  br label %60

60:                                               ; preds = %59, %35
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @reallymarkobject(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.GCObject, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  switch i32 %10, label %121 [
    i32 4, label %11
    i32 20, label %11
    i32 9, label %21
    i32 7, label %77
    i32 6, label %115
    i32 38, label %115
    i32 5, label %115
    i32 8, label %115
    i32 10, label %115
  ]

11:                                               ; preds = %2, %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.GCObject, ptr %12, i32 0, i32 2
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, -25
  %17 = or i32 %16, 32
  %18 = trunc i32 %17 to i8
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.GCObject, ptr %19, i32 0, i32 2
  store i8 %18, ptr %20, align 1
  br label %122

21:                                               ; preds = %2
  %22 = load ptr, ptr %4, align 8
  store ptr %22, ptr %5, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.UpVal, ptr %23, i32 0, i32 3
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.UpVal, ptr %26, i32 0, i32 4
  %28 = icmp ne ptr %25, %27
  br i1 %28, label %29, label %36

29:                                               ; preds = %21
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.UpVal, ptr %30, i32 0, i32 2
  %32 = load i8, ptr %31, align 1
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, 199
  %35 = trunc i32 %34 to i8
  store i8 %35, ptr %31, align 1
  br label %46

36:                                               ; preds = %21
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.UpVal, ptr %37, i32 0, i32 2
  %39 = load i8, ptr %38, align 1
  %40 = zext i8 %39 to i32
  %41 = and i32 %40, -25
  %42 = or i32 %41, 32
  %43 = trunc i32 %42 to i8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.UpVal, ptr %44, i32 0, i32 2
  store i8 %43, ptr %45, align 1
  br label %46

46:                                               ; preds = %36, %29
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 40
  %49 = load ptr, ptr %48, align 8
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %struct.UpVal, ptr %50, i32 0, i32 3
  %52 = load ptr, ptr %51, align 8
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 1
  %54 = load i8, ptr %53, align 8
  %55 = zext i8 %54 to i32
  %56 = and i32 %55, 64
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %58, label %76

58:                                               ; preds = %46
  %59 = load ptr, ptr %5, align 8
  %60 = getelementptr inbounds %struct.UpVal, ptr %59, i32 0, i32 3
  %61 = load ptr, ptr %60, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %struct.GCObject, ptr %63, i32 0, i32 2
  %65 = load i8, ptr %64, align 1
  %66 = zext i8 %65 to i32
  %67 = and i32 %66, 24
  %68 = icmp ne i32 %67, 0
  br i1 %68, label %69, label %76

69:                                               ; preds = %58
  %70 = load ptr, ptr %3, align 8
  %71 = load ptr, ptr %5, align 8
  %72 = getelementptr inbounds %struct.UpVal, ptr %71, i32 0, i32 3
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 0
  %75 = load ptr, ptr %74, align 8
  call void @reallymarkobject(ptr noundef %70, ptr noundef %75)
  br label %76

76:                                               ; preds = %69, %58, %46
  br label %122

77:                                               ; preds = %2
  %78 = load ptr, ptr %4, align 8
  store ptr %78, ptr %6, align 8
  %79 = load ptr, ptr %6, align 8
  %80 = getelementptr inbounds %struct.Udata, ptr %79, i32 0, i32 3
  %81 = load i16, ptr %80, align 2
  %82 = zext i16 %81 to i32
  %83 = icmp eq i32 %82, 0
  br i1 %83, label %84, label %114

84:                                               ; preds = %77
  %85 = load ptr, ptr %6, align 8
  %86 = getelementptr inbounds %struct.Udata, ptr %85, i32 0, i32 5
  %87 = load ptr, ptr %86, align 8
  %88 = icmp ne ptr %87, null
  br i1 %88, label %89, label %104

89:                                               ; preds = %84
  %90 = load ptr, ptr %6, align 8
  %91 = getelementptr inbounds %struct.Udata, ptr %90, i32 0, i32 5
  %92 = load ptr, ptr %91, align 8
  %93 = getelementptr inbounds %struct.Table, ptr %92, i32 0, i32 2
  %94 = load i8, ptr %93, align 1
  %95 = zext i8 %94 to i32
  %96 = and i32 %95, 24
  %97 = icmp ne i32 %96, 0
  br i1 %97, label %98, label %103

98:                                               ; preds = %89
  %99 = load ptr, ptr %3, align 8
  %100 = load ptr, ptr %6, align 8
  %101 = getelementptr inbounds %struct.Udata, ptr %100, i32 0, i32 5
  %102 = load ptr, ptr %101, align 8
  call void @reallymarkobject(ptr noundef %99, ptr noundef %102)
  br label %103

103:                                              ; preds = %98, %89
  br label %104

104:                                              ; preds = %103, %84
  %105 = load ptr, ptr %6, align 8
  %106 = getelementptr inbounds %struct.Udata, ptr %105, i32 0, i32 2
  %107 = load i8, ptr %106, align 1
  %108 = zext i8 %107 to i32
  %109 = and i32 %108, -25
  %110 = or i32 %109, 32
  %111 = trunc i32 %110 to i8
  %112 = load ptr, ptr %6, align 8
  %113 = getelementptr inbounds %struct.Udata, ptr %112, i32 0, i32 2
  store i8 %111, ptr %113, align 1
  br label %122

114:                                              ; preds = %77
  br label %115

115:                                              ; preds = %2, %2, %2, %2, %2, %114
  %116 = load ptr, ptr %4, align 8
  %117 = load ptr, ptr %4, align 8
  %118 = call ptr @getgclist(ptr noundef %117)
  %119 = load ptr, ptr %3, align 8
  %120 = getelementptr inbounds %struct.global_State, ptr %119, i32 0, i32 24
  call void @linkgclist_(ptr noundef %116, ptr noundef %118, ptr noundef %120)
  br label %122

121:                                              ; preds = %2
  br label %122

122:                                              ; preds = %121, %115, %104, %76, %11
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_barrierback_(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.GCObject, ptr %9, i32 0, i32 2
  %11 = load i8, ptr %10, align 1
  %12 = zext i8 %11 to i32
  %13 = and i32 %12, 7
  %14 = icmp eq i32 %13, 6
  br i1 %14, label %15, label %22

15:                                               ; preds = %2
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.GCObject, ptr %16, i32 0, i32 2
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = and i32 %19, 199
  %21 = trunc i32 %20 to i8
  store i8 %21, ptr %17, align 1
  br label %28

22:                                               ; preds = %2
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = call ptr @getgclist(ptr noundef %24)
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 25
  call void @linkgclist_(ptr noundef %23, ptr noundef %25, ptr noundef %27)
  br label %28

28:                                               ; preds = %22, %15
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.GCObject, ptr %29, i32 0, i32 2
  %31 = load i8, ptr %30, align 1
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 7
  %34 = icmp sgt i32 %33, 1
  br i1 %34, label %35, label %45

35:                                               ; preds = %28
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.GCObject, ptr %36, i32 0, i32 2
  %38 = load i8, ptr %37, align 1
  %39 = zext i8 %38 to i32
  %40 = and i32 %39, -8
  %41 = or i32 %40, 5
  %42 = trunc i32 %41 to i8
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.GCObject, ptr %43, i32 0, i32 2
  store i8 %42, ptr %44, align 1
  br label %45

45:                                               ; preds = %35, %28
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @linkgclist_(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  store ptr %8, ptr %9, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %6, align 8
  store ptr %10, ptr %11, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.GCObject, ptr %12, i32 0, i32 2
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 199
  %17 = trunc i32 %16 to i8
  store i8 %17, ptr %13, align 1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getgclist(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.GCObject, ptr %5, i32 0, i32 1
  %7 = load i8, ptr %6, align 8
  %8 = zext i8 %7 to i32
  switch i32 %8, label %28 [
    i32 5, label %9
    i32 6, label %12
    i32 38, label %15
    i32 8, label %18
    i32 10, label %21
    i32 7, label %24
  ]

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.Table, ptr %10, i32 0, i32 10
  store ptr %11, ptr %2, align 8
  br label %29

12:                                               ; preds = %1
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.LClosure, ptr %13, i32 0, i32 4
  store ptr %14, ptr %2, align 8
  br label %29

15:                                               ; preds = %1
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.CClosure, ptr %16, i32 0, i32 4
  store ptr %17, ptr %2, align 8
  br label %29

18:                                               ; preds = %1
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 13
  store ptr %20, ptr %2, align 8
  br label %29

21:                                               ; preds = %1
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 23
  store ptr %23, ptr %2, align 8
  br label %29

24:                                               ; preds = %1
  %25 = load ptr, ptr %3, align 8
  store ptr %25, ptr %4, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.Udata, ptr %26, i32 0, i32 6
  store ptr %27, ptr %2, align 8
  br label %29

28:                                               ; preds = %1
  store ptr null, ptr %2, align 8
  br label %29

29:                                               ; preds = %28, %24, %21, %18, %15, %12, %9
  %30 = load ptr, ptr %2, align 8
  ret ptr %30
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_fix(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.GCObject, ptr %9, i32 0, i32 2
  %11 = load i8, ptr %10, align 1
  %12 = zext i8 %11 to i32
  %13 = and i32 %12, 199
  %14 = trunc i32 %13 to i8
  store i8 %14, ptr %10, align 1
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.GCObject, ptr %15, i32 0, i32 2
  %17 = load i8, ptr %16, align 1
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, -8
  %20 = or i32 %19, 4
  %21 = trunc i32 %20 to i8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.GCObject, ptr %22, i32 0, i32 2
  store i8 %21, ptr %23, align 1
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.GCObject, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 21
  store ptr %26, ptr %28, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 30
  %31 = load ptr, ptr %30, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.GCObject, ptr %32, i32 0, i32 0
  store ptr %31, ptr %33, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 30
  store ptr %34, ptr %36, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaC_newobjdt(ptr noundef %0, i32 noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i64 %2, ptr %7, align 8
  store i64 %3, ptr %8, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = load i64, ptr %7, align 8
  %17 = load i32, ptr %6, align 4
  %18 = and i32 %17, 15
  %19 = call ptr @luaM_malloc_(ptr noundef %15, i64 noundef %16, i32 noundef %18)
  store ptr %19, ptr %10, align 8
  %20 = load ptr, ptr %10, align 8
  %21 = load i64, ptr %8, align 8
  %22 = getelementptr inbounds i8, ptr %20, i64 %21
  store ptr %22, ptr %11, align 8
  %23 = load ptr, ptr %9, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 10
  %25 = load i8, ptr %24, align 4
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 24
  %28 = trunc i32 %27 to i8
  %29 = load ptr, ptr %11, align 8
  %30 = getelementptr inbounds %struct.GCObject, ptr %29, i32 0, i32 2
  store i8 %28, ptr %30, align 1
  %31 = load i32, ptr %6, align 4
  %32 = trunc i32 %31 to i8
  %33 = load ptr, ptr %11, align 8
  %34 = getelementptr inbounds %struct.GCObject, ptr %33, i32 0, i32 1
  store i8 %32, ptr %34, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 21
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %11, align 8
  %39 = getelementptr inbounds %struct.GCObject, ptr %38, i32 0, i32 0
  store ptr %37, ptr %39, align 8
  %40 = load ptr, ptr %11, align 8
  %41 = load ptr, ptr %9, align 8
  %42 = getelementptr inbounds %struct.global_State, ptr %41, i32 0, i32 21
  store ptr %40, ptr %42, align 8
  %43 = load ptr, ptr %11, align 8
  ret ptr %43
}

declare hidden ptr @luaM_malloc_(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaC_newobj(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = load i64, ptr %6, align 8
  %10 = call ptr @luaC_newobjdt(ptr noundef %7, i32 noundef %8, i64 noundef %9, i64 noundef 0)
  ret ptr %10
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_checkfinalizer(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 7
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.GCObject, ptr %12, i32 0, i32 2
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 64
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %49, label %18

18:                                               ; preds = %3
  %19 = load ptr, ptr %6, align 8
  %20 = icmp eq ptr %19, null
  br i1 %20, label %21, label %22

21:                                               ; preds = %18
  br label %39

22:                                               ; preds = %18
  %23 = load ptr, ptr %6, align 8
  %24 = getelementptr inbounds %struct.Table, ptr %23, i32 0, i32 3
  %25 = load i8, ptr %24, align 2
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 4
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %30

29:                                               ; preds = %22
  br label %37

30:                                               ; preds = %22
  %31 = load ptr, ptr %6, align 8
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 42
  %34 = getelementptr inbounds [25 x ptr], ptr %33, i64 0, i64 2
  %35 = load ptr, ptr %34, align 8
  %36 = call ptr @luaT_gettm(ptr noundef %31, i32 noundef 2, ptr noundef %35)
  br label %37

37:                                               ; preds = %30, %29
  %38 = phi ptr [ null, %29 ], [ %36, %30 ]
  br label %39

39:                                               ; preds = %37, %21
  %40 = phi ptr [ null, %21 ], [ %38, %37 ]
  %41 = icmp eq ptr %40, null
  br i1 %41, label %49, label %42

42:                                               ; preds = %39
  %43 = load ptr, ptr %7, align 8
  %44 = getelementptr inbounds %struct.global_State, ptr %43, i32 0, i32 16
  %45 = load i8, ptr %44, align 2
  %46 = zext i8 %45 to i32
  %47 = and i32 %46, 4
  %48 = icmp ne i32 %47, 0
  br i1 %48, label %49, label %50

49:                                               ; preds = %42, %39, %3
  br label %129

50:                                               ; preds = %42
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.global_State, ptr %51, i32 0, i32 11
  %53 = load i8, ptr %52, align 1
  %54 = zext i8 %53 to i32
  %55 = icmp sle i32 3, %54
  br i1 %55, label %56, label %94

56:                                               ; preds = %50
  %57 = load ptr, ptr %7, align 8
  %58 = getelementptr inbounds %struct.global_State, ptr %57, i32 0, i32 11
  %59 = load i8, ptr %58, align 1
  %60 = zext i8 %59 to i32
  %61 = icmp sle i32 %60, 6
  br i1 %61, label %62, label %94

62:                                               ; preds = %56
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.GCObject, ptr %63, i32 0, i32 2
  %65 = load i8, ptr %64, align 1
  %66 = zext i8 %65 to i32
  %67 = and i32 %66, -57
  %68 = load ptr, ptr %7, align 8
  %69 = getelementptr inbounds %struct.global_State, ptr %68, i32 0, i32 10
  %70 = load i8, ptr %69, align 4
  %71 = zext i8 %70 to i32
  %72 = and i32 %71, 24
  %73 = trunc i32 %72 to i8
  %74 = zext i8 %73 to i32
  %75 = or i32 %67, %74
  %76 = trunc i32 %75 to i8
  %77 = load ptr, ptr %5, align 8
  %78 = getelementptr inbounds %struct.GCObject, ptr %77, i32 0, i32 2
  store i8 %76, ptr %78, align 1
  %79 = load ptr, ptr %7, align 8
  %80 = getelementptr inbounds %struct.global_State, ptr %79, i32 0, i32 22
  %81 = load ptr, ptr %80, align 8
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.GCObject, ptr %82, i32 0, i32 0
  %84 = icmp eq ptr %81, %83
  br i1 %84, label %85, label %93

85:                                               ; preds = %62
  %86 = load ptr, ptr %4, align 8
  %87 = load ptr, ptr %7, align 8
  %88 = getelementptr inbounds %struct.global_State, ptr %87, i32 0, i32 22
  %89 = load ptr, ptr %88, align 8
  %90 = call ptr @sweeptolive(ptr noundef %86, ptr noundef %89)
  %91 = load ptr, ptr %7, align 8
  %92 = getelementptr inbounds %struct.global_State, ptr %91, i32 0, i32 22
  store ptr %90, ptr %92, align 8
  br label %93

93:                                               ; preds = %85, %62
  br label %97

94:                                               ; preds = %56, %50
  %95 = load ptr, ptr %7, align 8
  %96 = load ptr, ptr %5, align 8
  call void @correctpointers(ptr noundef %95, ptr noundef %96)
  br label %97

97:                                               ; preds = %94, %93
  %98 = load ptr, ptr %7, align 8
  %99 = getelementptr inbounds %struct.global_State, ptr %98, i32 0, i32 21
  store ptr %99, ptr %8, align 8
  br label %100

100:                                              ; preds = %106, %97
  %101 = load ptr, ptr %8, align 8
  %102 = load ptr, ptr %101, align 8
  %103 = load ptr, ptr %5, align 8
  %104 = icmp ne ptr %102, %103
  br i1 %104, label %105, label %110

105:                                              ; preds = %100
  br label %106

106:                                              ; preds = %105
  %107 = load ptr, ptr %8, align 8
  %108 = load ptr, ptr %107, align 8
  %109 = getelementptr inbounds %struct.GCObject, ptr %108, i32 0, i32 0
  store ptr %109, ptr %8, align 8
  br label %100, !llvm.loop !6

110:                                              ; preds = %100
  %111 = load ptr, ptr %5, align 8
  %112 = getelementptr inbounds %struct.GCObject, ptr %111, i32 0, i32 0
  %113 = load ptr, ptr %112, align 8
  %114 = load ptr, ptr %8, align 8
  store ptr %113, ptr %114, align 8
  %115 = load ptr, ptr %7, align 8
  %116 = getelementptr inbounds %struct.global_State, ptr %115, i32 0, i32 23
  %117 = load ptr, ptr %116, align 8
  %118 = load ptr, ptr %5, align 8
  %119 = getelementptr inbounds %struct.GCObject, ptr %118, i32 0, i32 0
  store ptr %117, ptr %119, align 8
  %120 = load ptr, ptr %5, align 8
  %121 = load ptr, ptr %7, align 8
  %122 = getelementptr inbounds %struct.global_State, ptr %121, i32 0, i32 23
  store ptr %120, ptr %122, align 8
  %123 = load ptr, ptr %5, align 8
  %124 = getelementptr inbounds %struct.GCObject, ptr %123, i32 0, i32 2
  %125 = load i8, ptr %124, align 1
  %126 = zext i8 %125 to i32
  %127 = or i32 %126, 64
  %128 = trunc i32 %127 to i8
  store i8 %128, ptr %124, align 1
  br label %129

129:                                              ; preds = %49, %110
  ret void
}

declare hidden ptr @luaT_gettm(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @sweeptolive(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  store ptr %6, ptr %5, align 8
  br label %7

7:                                                ; preds = %11, %2
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @sweeplist(ptr noundef %8, ptr noundef %9, i32 noundef 1, ptr noundef null)
  store ptr %10, ptr %4, align 8
  br label %11

11:                                               ; preds = %7
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = icmp eq ptr %12, %13
  br i1 %14, label %7, label %15, !llvm.loop !8

15:                                               ; preds = %11
  %16 = load ptr, ptr %4, align 8
  ret ptr %16
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @correctpointers(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 31
  %7 = load ptr, ptr %4, align 8
  call void @checkpointer(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 32
  %10 = load ptr, ptr %4, align 8
  call void @checkpointer(ptr noundef %9, ptr noundef %10)
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 33
  %13 = load ptr, ptr %4, align 8
  call void @checkpointer(ptr noundef %12, ptr noundef %13)
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 34
  %16 = load ptr, ptr %4, align 8
  call void @checkpointer(ptr noundef %15, ptr noundef %16)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_changemode(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load i32, ptr %4, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 12
  %12 = load i8, ptr %11, align 2
  %13 = zext i8 %12 to i32
  %14 = icmp ne i32 %9, %13
  br i1 %14, label %15, label %25

15:                                               ; preds = %2
  %16 = load i32, ptr %4, align 4
  %17 = icmp eq i32 %16, 1
  br i1 %17, label %18, label %22

18:                                               ; preds = %15
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = call i64 @entergen(ptr noundef %19, ptr noundef %20)
  br label %24

22:                                               ; preds = %15
  %23 = load ptr, ptr %5, align 8
  call void @enterinc(ptr noundef %23)
  br label %24

24:                                               ; preds = %22, %18
  br label %25

25:                                               ; preds = %24, %2
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 5
  store i64 0, ptr %27, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @entergen(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %6, i32 noundef 256)
  %7 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %3, align 8
  %9 = call i64 @atomic(ptr noundef %8)
  store i64 %9, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  call void @atomic2gen(ptr noundef %10, ptr noundef %11)
  %12 = load ptr, ptr %4, align 8
  call void @setminordebt(ptr noundef %12)
  %13 = load i64, ptr %5, align 8
  ret i64 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @enterinc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.global_State, ptr %4, i32 0, i32 21
  %6 = load ptr, ptr %5, align 8
  call void @whitelist(ptr noundef %3, ptr noundef %6)
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 31
  store ptr null, ptr %8, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 32
  store ptr null, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 33
  store ptr null, ptr %12, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 23
  %16 = load ptr, ptr %15, align 8
  call void @whitelist(ptr noundef %13, ptr noundef %16)
  %17 = load ptr, ptr %2, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 29
  %20 = load ptr, ptr %19, align 8
  call void @whitelist(ptr noundef %17, ptr noundef %20)
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 35
  store ptr null, ptr %22, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 36
  store ptr null, ptr %24, align 8
  %25 = load ptr, ptr %2, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 37
  store ptr null, ptr %26, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 11
  store i8 8, ptr %28, align 1
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 12
  store i8 0, ptr %30, align 2
  %31 = load ptr, ptr %2, align 8
  %32 = getelementptr inbounds %struct.global_State, ptr %31, i32 0, i32 5
  store i64 0, ptr %32, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_freeallobjects(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 7
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 16
  store i8 4, ptr %8, align 2
  %9 = load ptr, ptr %2, align 8
  call void @luaC_changemode(ptr noundef %9, i32 noundef 0)
  %10 = load ptr, ptr %3, align 8
  call void @separatetobefnz(ptr noundef %10, i32 noundef 1)
  %11 = load ptr, ptr %2, align 8
  call void @callallpendingfinalizers(ptr noundef %11)
  %12 = load ptr, ptr %2, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 21
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 40
  %18 = load ptr, ptr %17, align 8
  call void @deletelist(ptr noundef %12, ptr noundef %15, ptr noundef %18)
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 30
  %22 = load ptr, ptr %21, align 8
  call void @deletelist(ptr noundef %19, ptr noundef %22, ptr noundef null)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @separatetobefnz(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 23
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 29
  %12 = call ptr @findlast(ptr noundef %11)
  store ptr %12, ptr %7, align 8
  br label %13

13:                                               ; preds = %58, %2
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %5, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 36
  %18 = load ptr, ptr %17, align 8
  %19 = icmp ne ptr %15, %18
  br i1 %19, label %20, label %59

20:                                               ; preds = %13
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.GCObject, ptr %21, i32 0, i32 2
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = and i32 %24, 24
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %33, label %27

27:                                               ; preds = %20
  %28 = load i32, ptr %4, align 4
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %33, label %30

30:                                               ; preds = %27
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.GCObject, ptr %31, i32 0, i32 0
  store ptr %32, ptr %6, align 8
  br label %58

33:                                               ; preds = %27, %20
  %34 = load ptr, ptr %5, align 8
  %35 = load ptr, ptr %3, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 35
  %37 = load ptr, ptr %36, align 8
  %38 = icmp eq ptr %34, %37
  br i1 %38, label %39, label %45

39:                                               ; preds = %33
  %40 = load ptr, ptr %5, align 8
  %41 = getelementptr inbounds %struct.GCObject, ptr %40, i32 0, i32 0
  %42 = load ptr, ptr %41, align 8
  %43 = load ptr, ptr %3, align 8
  %44 = getelementptr inbounds %struct.global_State, ptr %43, i32 0, i32 35
  store ptr %42, ptr %44, align 8
  br label %45

45:                                               ; preds = %39, %33
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.GCObject, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  %49 = load ptr, ptr %6, align 8
  store ptr %48, ptr %49, align 8
  %50 = load ptr, ptr %7, align 8
  %51 = load ptr, ptr %50, align 8
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.GCObject, ptr %52, i32 0, i32 0
  store ptr %51, ptr %53, align 8
  %54 = load ptr, ptr %5, align 8
  %55 = load ptr, ptr %7, align 8
  store ptr %54, ptr %55, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.GCObject, ptr %56, i32 0, i32 0
  store ptr %57, ptr %7, align 8
  br label %58

58:                                               ; preds = %45, %30
  br label %13, !llvm.loop !9

59:                                               ; preds = %13
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @callallpendingfinalizers(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 7
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  br label %7

7:                                                ; preds = %12, %1
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 29
  %10 = load ptr, ptr %9, align 8
  %11 = icmp ne ptr %10, null
  br i1 %11, label %12, label %14

12:                                               ; preds = %7
  %13 = load ptr, ptr %2, align 8
  call void @GCTM(ptr noundef %13)
  br label %7, !llvm.loop !10

14:                                               ; preds = %7
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @deletelist(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  br label %8

8:                                                ; preds = %12, %3
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = icmp ne ptr %9, %10
  br i1 %11, label %12, label %19

12:                                               ; preds = %8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.GCObject, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %5, align 8
  call void @freeobj(ptr noundef %16, ptr noundef %17)
  %18 = load ptr, ptr %7, align 8
  store ptr %18, ptr %5, align 8
  br label %8, !llvm.loop !11

19:                                               ; preds = %8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_runtilstate(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  br label %9

9:                                                ; preds = %19, %2
  %10 = load i32, ptr %4, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 11
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = shl i32 1, %14
  %16 = and i32 %10, %15
  %17 = icmp ne i32 %16, 0
  %18 = xor i1 %17, true
  br i1 %18, label %19, label %22

19:                                               ; preds = %9
  %20 = load ptr, ptr %3, align 8
  %21 = call i64 @singlestep(ptr noundef %20)
  br label %9, !llvm.loop !12

22:                                               ; preds = %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @singlestep(ptr noundef %0) #0 {
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 13
  store i8 1, ptr %10, align 1
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 11
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  switch i32 %14, label %89 [
    i32 8, label %15
    i32 0, label %19
    i32 1, label %31
    i32 3, label %44
    i32 4, label %51
    i32 5, label %58
    i32 6, label %63
    i32 7, label %68
  ]

15:                                               ; preds = %1
  %16 = load ptr, ptr %4, align 8
  call void @restartcollection(ptr noundef %16)
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 11
  store i8 0, ptr %18, align 1
  store i64 1, ptr %5, align 8
  br label %90

19:                                               ; preds = %1
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 24
  %22 = load ptr, ptr %21, align 8
  %23 = icmp eq ptr %22, null
  br i1 %23, label %24, label %27

24:                                               ; preds = %19
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 11
  store i8 1, ptr %26, align 1
  store i64 0, ptr %5, align 8
  br label %30

27:                                               ; preds = %19
  %28 = load ptr, ptr %4, align 8
  %29 = call i64 @propagatemark(ptr noundef %28)
  store i64 %29, ptr %5, align 8
  br label %30

30:                                               ; preds = %27, %24
  br label %90

31:                                               ; preds = %1
  %32 = load ptr, ptr %3, align 8
  %33 = call i64 @atomic(ptr noundef %32)
  store i64 %33, ptr %5, align 8
  %34 = load ptr, ptr %3, align 8
  call void @entersweep(ptr noundef %34)
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 2
  %37 = load i64, ptr %36, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.global_State, ptr %38, i32 0, i32 3
  %40 = load i64, ptr %39, align 8
  %41 = add nsw i64 %37, %40
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 4
  store i64 %41, ptr %43, align 8
  br label %90

44:                                               ; preds = %1
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 23
  %49 = call i32 @sweepstep(ptr noundef %45, ptr noundef %46, i32 noundef 4, ptr noundef %48)
  %50 = sext i32 %49 to i64
  store i64 %50, ptr %5, align 8
  br label %90

51:                                               ; preds = %1
  %52 = load ptr, ptr %3, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.global_State, ptr %54, i32 0, i32 29
  %56 = call i32 @sweepstep(ptr noundef %52, ptr noundef %53, i32 noundef 5, ptr noundef %55)
  %57 = sext i32 %56 to i64
  store i64 %57, ptr %5, align 8
  br label %90

58:                                               ; preds = %1
  %59 = load ptr, ptr %3, align 8
  %60 = load ptr, ptr %4, align 8
  %61 = call i32 @sweepstep(ptr noundef %59, ptr noundef %60, i32 noundef 6, ptr noundef null)
  %62 = sext i32 %61 to i64
  store i64 %62, ptr %5, align 8
  br label %90

63:                                               ; preds = %1
  %64 = load ptr, ptr %3, align 8
  %65 = load ptr, ptr %4, align 8
  call void @checkSizes(ptr noundef %64, ptr noundef %65)
  %66 = load ptr, ptr %4, align 8
  %67 = getelementptr inbounds %struct.global_State, ptr %66, i32 0, i32 11
  store i8 7, ptr %67, align 1
  store i64 0, ptr %5, align 8
  br label %90

68:                                               ; preds = %1
  %69 = load ptr, ptr %4, align 8
  %70 = getelementptr inbounds %struct.global_State, ptr %69, i32 0, i32 29
  %71 = load ptr, ptr %70, align 8
  %72 = icmp ne ptr %71, null
  br i1 %72, label %73, label %85

73:                                               ; preds = %68
  %74 = load ptr, ptr %4, align 8
  %75 = getelementptr inbounds %struct.global_State, ptr %74, i32 0, i32 17
  %76 = load i8, ptr %75, align 1
  %77 = icmp ne i8 %76, 0
  br i1 %77, label %85, label %78

78:                                               ; preds = %73
  %79 = load ptr, ptr %4, align 8
  %80 = getelementptr inbounds %struct.global_State, ptr %79, i32 0, i32 13
  store i8 0, ptr %80, align 1
  %81 = load ptr, ptr %3, align 8
  %82 = call i32 @runafewfinalizers(ptr noundef %81, i32 noundef 10)
  %83 = mul nsw i32 %82, 50
  %84 = sext i32 %83 to i64
  store i64 %84, ptr %5, align 8
  br label %88

85:                                               ; preds = %73, %68
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.global_State, ptr %86, i32 0, i32 11
  store i8 8, ptr %87, align 1
  store i64 0, ptr %5, align 8
  br label %88

88:                                               ; preds = %85, %78
  br label %90

89:                                               ; preds = %1
  store i64 0, ptr %2, align 8
  br label %94

90:                                               ; preds = %88, %63, %58, %51, %44, %31, %30, %15
  %91 = load ptr, ptr %4, align 8
  %92 = getelementptr inbounds %struct.global_State, ptr %91, i32 0, i32 13
  store i8 0, ptr %92, align 1
  %93 = load i64, ptr %5, align 8
  store i64 %93, ptr %2, align 8
  br label %94

94:                                               ; preds = %90, %89
  %95 = load i64, ptr %2, align 8
  ret i64 %95
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_step(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 7
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 16
  %9 = load i8, ptr %8, align 2
  %10 = zext i8 %9 to i32
  %11 = icmp eq i32 %10, 0
  br i1 %11, label %14, label %12

12:                                               ; preds = %1
  %13 = load ptr, ptr %3, align 8
  call void @luaE_setdebt(ptr noundef %13, i64 noundef -2000)
  br label %32

14:                                               ; preds = %1
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 12
  %17 = load i8, ptr %16, align 2
  %18 = zext i8 %17 to i32
  %19 = icmp eq i32 %18, 1
  br i1 %19, label %25, label %20

20:                                               ; preds = %14
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 5
  %23 = load i64, ptr %22, align 8
  %24 = icmp ne i64 %23, 0
  br i1 %24, label %25, label %28

25:                                               ; preds = %20, %14
  %26 = load ptr, ptr %2, align 8
  %27 = load ptr, ptr %3, align 8
  call void @genstep(ptr noundef %26, ptr noundef %27)
  br label %31

28:                                               ; preds = %20
  %29 = load ptr, ptr %2, align 8
  %30 = load ptr, ptr %3, align 8
  call void @incstep(ptr noundef %29, ptr noundef %30)
  br label %31

31:                                               ; preds = %28, %25
  br label %32

32:                                               ; preds = %31, %12
  ret void
}

declare hidden void @luaE_setdebt(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @genstep(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 5
  %10 = load i64, ptr %9, align 8
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %2
  %13 = load ptr, ptr %3, align 8
  %14 = load ptr, ptr %4, align 8
  call void @stepgenfull(ptr noundef %13, ptr noundef %14)
  br label %75

15:                                               ; preds = %2
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 4
  %18 = load i64, ptr %17, align 8
  store i64 %18, ptr %5, align 8
  %19 = load i64, ptr %5, align 8
  %20 = udiv i64 %19, 100
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 15
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = mul nsw i32 %24, 4
  %26 = sext i32 %25 to i64
  %27 = mul i64 %20, %26
  store i64 %27, ptr %6, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 3
  %30 = load i64, ptr %29, align 8
  %31 = icmp sgt i64 %30, 0
  br i1 %31, label %32, label %67

32:                                               ; preds = %15
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.global_State, ptr %33, i32 0, i32 2
  %35 = load i64, ptr %34, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.global_State, ptr %36, i32 0, i32 3
  %38 = load i64, ptr %37, align 8
  %39 = add nsw i64 %35, %38
  %40 = load i64, ptr %5, align 8
  %41 = load i64, ptr %6, align 8
  %42 = add i64 %40, %41
  %43 = icmp ugt i64 %39, %42
  br i1 %43, label %44, label %67

44:                                               ; preds = %32
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = call i64 @fullgen(ptr noundef %45, ptr noundef %46)
  store i64 %47, ptr %7, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.global_State, ptr %48, i32 0, i32 2
  %50 = load i64, ptr %49, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.global_State, ptr %51, i32 0, i32 3
  %53 = load i64, ptr %52, align 8
  %54 = add nsw i64 %50, %53
  %55 = load i64, ptr %5, align 8
  %56 = load i64, ptr %6, align 8
  %57 = udiv i64 %56, 2
  %58 = add i64 %55, %57
  %59 = icmp ult i64 %54, %58
  br i1 %59, label %60, label %61

60:                                               ; preds = %44
  br label %66

61:                                               ; preds = %44
  %62 = load i64, ptr %7, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.global_State, ptr %63, i32 0, i32 5
  store i64 %62, ptr %64, align 8
  %65 = load ptr, ptr %4, align 8
  call void @setpause(ptr noundef %65)
  br label %66

66:                                               ; preds = %61, %60
  br label %74

67:                                               ; preds = %32, %15
  %68 = load ptr, ptr %3, align 8
  %69 = load ptr, ptr %4, align 8
  call void @youngcollection(ptr noundef %68, ptr noundef %69)
  %70 = load ptr, ptr %4, align 8
  call void @setminordebt(ptr noundef %70)
  %71 = load i64, ptr %5, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.global_State, ptr %72, i32 0, i32 4
  store i64 %71, ptr %73, align 8
  br label %74

74:                                               ; preds = %67, %66
  br label %75

75:                                               ; preds = %74, %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @incstep(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 19
  %11 = load i8, ptr %10, align 1
  %12 = zext i8 %11 to i32
  %13 = mul nsw i32 %12, 4
  %14 = or i32 %13, 1
  store i32 %14, ptr %5, align 4
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 3
  %17 = load i64, ptr %16, align 8
  %18 = udiv i64 %17, 16
  %19 = load i32, ptr %5, align 4
  %20 = sext i32 %19 to i64
  %21 = mul i64 %18, %20
  store i64 %21, ptr %6, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.global_State, ptr %22, i32 0, i32 20
  %24 = load i8, ptr %23, align 2
  %25 = zext i8 %24 to i64
  %26 = icmp ule i64 %25, 62
  br i1 %26, label %27, label %38

27:                                               ; preds = %2
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 20
  %30 = load i8, ptr %29, align 2
  %31 = zext i8 %30 to i32
  %32 = zext i32 %31 to i64
  %33 = shl i64 1, %32
  %34 = udiv i64 %33, 16
  %35 = load i32, ptr %5, align 4
  %36 = sext i32 %35 to i64
  %37 = mul i64 %34, %36
  br label %39

38:                                               ; preds = %2
  br label %39

39:                                               ; preds = %38, %27
  %40 = phi i64 [ %37, %27 ], [ 9223372036854775807, %38 ]
  store i64 %40, ptr %7, align 8
  br label %41

41:                                               ; preds = %58, %39
  %42 = load ptr, ptr %3, align 8
  %43 = call i64 @singlestep(ptr noundef %42)
  store i64 %43, ptr %8, align 8
  %44 = load i64, ptr %8, align 8
  %45 = load i64, ptr %6, align 8
  %46 = sub i64 %45, %44
  store i64 %46, ptr %6, align 8
  br label %47

47:                                               ; preds = %41
  %48 = load i64, ptr %6, align 8
  %49 = load i64, ptr %7, align 8
  %50 = sub nsw i64 0, %49
  %51 = icmp sgt i64 %48, %50
  br i1 %51, label %52, label %58

52:                                               ; preds = %47
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.global_State, ptr %53, i32 0, i32 11
  %55 = load i8, ptr %54, align 1
  %56 = zext i8 %55 to i32
  %57 = icmp ne i32 %56, 8
  br label %58

58:                                               ; preds = %52, %47
  %59 = phi i1 [ false, %47 ], [ %57, %52 ]
  br i1 %59, label %41, label %60, !llvm.loop !13

60:                                               ; preds = %58
  %61 = load ptr, ptr %4, align 8
  %62 = getelementptr inbounds %struct.global_State, ptr %61, i32 0, i32 11
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = icmp eq i32 %64, 8
  br i1 %65, label %66, label %68

66:                                               ; preds = %60
  %67 = load ptr, ptr %4, align 8
  call void @setpause(ptr noundef %67)
  br label %76

68:                                               ; preds = %60
  %69 = load i64, ptr %6, align 8
  %70 = load i32, ptr %5, align 4
  %71 = sext i32 %70 to i64
  %72 = sdiv i64 %69, %71
  %73 = mul i64 %72, 16
  store i64 %73, ptr %6, align 8
  %74 = load ptr, ptr %4, align 8
  %75 = load i64, ptr %6, align 8
  call void @luaE_setdebt(ptr noundef %74, i64 noundef %75)
  br label %76

76:                                               ; preds = %68, %66
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaC_fullgc(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 7
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load i32, ptr %4, align 4
  %10 = trunc i32 %9 to i8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 17
  store i8 %10, ptr %12, align 1
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 12
  %15 = load i8, ptr %14, align 2
  %16 = zext i8 %15 to i32
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %21

18:                                               ; preds = %2
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %5, align 8
  call void @fullinc(ptr noundef %19, ptr noundef %20)
  br label %25

21:                                               ; preds = %2
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = call i64 @fullgen(ptr noundef %22, ptr noundef %23)
  br label %25

25:                                               ; preds = %21, %18
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 17
  store i8 0, ptr %27, align 1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fullinc(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 11
  %7 = load i8, ptr %6, align 1
  %8 = zext i8 %7 to i32
  %9 = icmp sle i32 %8, 2
  br i1 %9, label %10, label %12

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8
  call void @entersweep(ptr noundef %11)
  br label %12

12:                                               ; preds = %10, %2
  %13 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %13, i32 noundef 256)
  %14 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %14, i32 noundef 1)
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 11
  store i8 1, ptr %16, align 1
  %17 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %17, i32 noundef 128)
  %18 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %18, i32 noundef 256)
  %19 = load ptr, ptr %4, align 8
  call void @setpause(ptr noundef %19)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @fullgen(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  call void @enterinc(ptr noundef %5)
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i64 @entergen(ptr noundef %6, ptr noundef %7)
  ret i64 %8
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @sweeplist(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca ptr, align 8
  %14 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 7
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %9, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 10
  %20 = load i8, ptr %19, align 4
  %21 = zext i8 %20 to i32
  %22 = xor i32 %21, 24
  store i32 %22, ptr %10, align 4
  %23 = load ptr, ptr %9, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 10
  %25 = load i8, ptr %24, align 4
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 24
  %28 = trunc i32 %27 to i8
  %29 = zext i8 %28 to i32
  store i32 %29, ptr %12, align 4
  store i32 0, ptr %11, align 4
  br label %30

30:                                               ; preds = %69, %4
  %31 = load ptr, ptr %6, align 8
  %32 = load ptr, ptr %31, align 8
  %33 = icmp ne ptr %32, null
  br i1 %33, label %34, label %38

34:                                               ; preds = %30
  %35 = load i32, ptr %11, align 4
  %36 = load i32, ptr %7, align 4
  %37 = icmp slt i32 %35, %36
  br label %38

38:                                               ; preds = %34, %30
  %39 = phi i1 [ false, %30 ], [ %37, %34 ]
  br i1 %39, label %40, label %72

40:                                               ; preds = %38
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %41, align 8
  store ptr %42, ptr %13, align 8
  %43 = load ptr, ptr %13, align 8
  %44 = getelementptr inbounds %struct.GCObject, ptr %43, i32 0, i32 2
  %45 = load i8, ptr %44, align 1
  %46 = zext i8 %45 to i32
  store i32 %46, ptr %14, align 4
  %47 = load i32, ptr %14, align 4
  %48 = load i32, ptr %10, align 4
  %49 = and i32 %47, %48
  %50 = icmp ne i32 %49, 0
  br i1 %50, label %51, label %58

51:                                               ; preds = %40
  %52 = load ptr, ptr %13, align 8
  %53 = getelementptr inbounds %struct.GCObject, ptr %52, i32 0, i32 0
  %54 = load ptr, ptr %53, align 8
  %55 = load ptr, ptr %6, align 8
  store ptr %54, ptr %55, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = load ptr, ptr %13, align 8
  call void @freeobj(ptr noundef %56, ptr noundef %57)
  br label %68

58:                                               ; preds = %40
  %59 = load i32, ptr %14, align 4
  %60 = and i32 %59, -64
  %61 = load i32, ptr %12, align 4
  %62 = or i32 %60, %61
  %63 = trunc i32 %62 to i8
  %64 = load ptr, ptr %13, align 8
  %65 = getelementptr inbounds %struct.GCObject, ptr %64, i32 0, i32 2
  store i8 %63, ptr %65, align 1
  %66 = load ptr, ptr %13, align 8
  %67 = getelementptr inbounds %struct.GCObject, ptr %66, i32 0, i32 0
  store ptr %67, ptr %6, align 8
  br label %68

68:                                               ; preds = %58, %51
  br label %69

69:                                               ; preds = %68
  %70 = load i32, ptr %11, align 4
  %71 = add nsw i32 %70, 1
  store i32 %71, ptr %11, align 4
  br label %30, !llvm.loop !14

72:                                               ; preds = %38
  %73 = load ptr, ptr %8, align 8
  %74 = icmp ne ptr %73, null
  br i1 %74, label %75, label %78

75:                                               ; preds = %72
  %76 = load i32, ptr %11, align 4
  %77 = load ptr, ptr %8, align 8
  store i32 %76, ptr %77, align 4
  br label %78

78:                                               ; preds = %75, %72
  %79 = load ptr, ptr %6, align 8
  %80 = load ptr, ptr %79, align 8
  %81 = icmp eq ptr %80, null
  br i1 %81, label %82, label %83

82:                                               ; preds = %78
  br label %85

83:                                               ; preds = %78
  %84 = load ptr, ptr %6, align 8
  br label %85

85:                                               ; preds = %83, %82
  %86 = phi ptr [ null, %82 ], [ %84, %83 ]
  ret ptr %86
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeobj(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.GCObject, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  switch i32 %13, label %95 [
    i32 10, label %14
    i32 9, label %17
    i32 6, label %20
    i32 38, label %31
    i32 5, label %42
    i32 8, label %45
    i32 7, label %48
    i32 4, label %71
    i32 20, label %85
  ]

14:                                               ; preds = %2
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %4, align 8
  call void @luaF_freeproto(ptr noundef %15, ptr noundef %16)
  br label %96

17:                                               ; preds = %2
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %4, align 8
  call void @freeupval(ptr noundef %18, ptr noundef %19)
  br label %96

20:                                               ; preds = %2
  %21 = load ptr, ptr %4, align 8
  store ptr %21, ptr %5, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.LClosure, ptr %24, i32 0, i32 3
  %26 = load i8, ptr %25, align 2
  %27 = zext i8 %26 to i32
  %28 = mul nsw i32 8, %27
  %29 = add nsw i32 32, %28
  %30 = sext i32 %29 to i64
  call void @luaM_free_(ptr noundef %22, ptr noundef %23, i64 noundef %30)
  br label %96

31:                                               ; preds = %2
  %32 = load ptr, ptr %4, align 8
  store ptr %32, ptr %6, align 8
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.CClosure, ptr %35, i32 0, i32 3
  %37 = load i8, ptr %36, align 2
  %38 = zext i8 %37 to i32
  %39 = mul nsw i32 16, %38
  %40 = add nsw i32 32, %39
  %41 = sext i32 %40 to i64
  call void @luaM_free_(ptr noundef %33, ptr noundef %34, i64 noundef %41)
  br label %96

42:                                               ; preds = %2
  %43 = load ptr, ptr %3, align 8
  %44 = load ptr, ptr %4, align 8
  call void @luaH_free(ptr noundef %43, ptr noundef %44)
  br label %96

45:                                               ; preds = %2
  %46 = load ptr, ptr %3, align 8
  %47 = load ptr, ptr %4, align 8
  call void @luaE_freethread(ptr noundef %46, ptr noundef %47)
  br label %96

48:                                               ; preds = %2
  %49 = load ptr, ptr %4, align 8
  store ptr %49, ptr %7, align 8
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = load ptr, ptr %7, align 8
  %53 = getelementptr inbounds %struct.Udata, ptr %52, i32 0, i32 3
  %54 = load i16, ptr %53, align 2
  %55 = zext i16 %54 to i32
  %56 = icmp eq i32 %55, 0
  br i1 %56, label %57, label %58

57:                                               ; preds = %48
  br label %65

58:                                               ; preds = %48
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds %struct.Udata, ptr %59, i32 0, i32 3
  %61 = load i16, ptr %60, align 2
  %62 = zext i16 %61 to i64
  %63 = mul i64 16, %62
  %64 = add i64 40, %63
  br label %65

65:                                               ; preds = %58, %57
  %66 = phi i64 [ 32, %57 ], [ %64, %58 ]
  %67 = load ptr, ptr %7, align 8
  %68 = getelementptr inbounds %struct.Udata, ptr %67, i32 0, i32 4
  %69 = load i64, ptr %68, align 8
  %70 = add i64 %66, %69
  call void @luaM_free_(ptr noundef %50, ptr noundef %51, i64 noundef %70)
  br label %96

71:                                               ; preds = %2
  %72 = load ptr, ptr %4, align 8
  store ptr %72, ptr %8, align 8
  %73 = load ptr, ptr %3, align 8
  %74 = load ptr, ptr %8, align 8
  call void @luaS_remove(ptr noundef %73, ptr noundef %74)
  %75 = load ptr, ptr %3, align 8
  %76 = load ptr, ptr %8, align 8
  %77 = load ptr, ptr %8, align 8
  %78 = getelementptr inbounds %struct.TString, ptr %77, i32 0, i32 4
  %79 = load i8, ptr %78, align 1
  %80 = zext i8 %79 to i32
  %81 = add nsw i32 %80, 1
  %82 = sext i32 %81 to i64
  %83 = mul i64 %82, 1
  %84 = add i64 24, %83
  call void @luaM_free_(ptr noundef %75, ptr noundef %76, i64 noundef %84)
  br label %96

85:                                               ; preds = %2
  %86 = load ptr, ptr %4, align 8
  store ptr %86, ptr %9, align 8
  %87 = load ptr, ptr %3, align 8
  %88 = load ptr, ptr %9, align 8
  %89 = load ptr, ptr %9, align 8
  %90 = getelementptr inbounds %struct.TString, ptr %89, i32 0, i32 6
  %91 = load i64, ptr %90, align 8
  %92 = add i64 %91, 1
  %93 = mul i64 %92, 1
  %94 = add i64 24, %93
  call void @luaM_free_(ptr noundef %87, ptr noundef %88, i64 noundef %94)
  br label %96

95:                                               ; preds = %2
  br label %96

96:                                               ; preds = %95, %85, %71, %65, %45, %42, %31, %20, %17, %14
  ret void
}

declare hidden void @luaF_freeproto(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @freeupval(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.UpVal, ptr %5, i32 0, i32 3
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.UpVal, ptr %8, i32 0, i32 4
  %10 = icmp ne ptr %7, %9
  br i1 %10, label %11, label %13

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  call void @luaF_unlinkupval(ptr noundef %12)
  br label %13

13:                                               ; preds = %11, %2
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %4, align 8
  call void @luaM_free_(ptr noundef %14, ptr noundef %15, i64 noundef 40)
  ret void
}

declare hidden void @luaM_free_(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden void @luaH_free(ptr noundef, ptr noundef) #1

declare hidden void @luaE_freethread(ptr noundef, ptr noundef) #1

declare hidden void @luaS_remove(ptr noundef, ptr noundef) #1

declare hidden void @luaF_unlinkupval(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkpointer(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %6, align 8
  %8 = icmp eq ptr %5, %7
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.GCObject, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %3, align 8
  store ptr %12, ptr %13, align 8
  br label %14

14:                                               ; preds = %9, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @atomic(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %3, align 8
  store i64 0, ptr %4, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 25
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 25
  store ptr null, ptr %15, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 11
  store i8 2, ptr %17, align 1
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 2
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i32
  %22 = and i32 %21, 24
  %23 = icmp ne i32 %22, 0
  br i1 %23, label %24, label %27

24:                                               ; preds = %1
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %2, align 8
  call void @reallymarkobject(ptr noundef %25, ptr noundef %26)
  br label %27

27:                                               ; preds = %24, %1
  %28 = load ptr, ptr %3, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 40
  %30 = load ptr, ptr %29, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.global_State, ptr %31, i32 0, i32 7
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = and i32 %35, 64
  %37 = icmp ne i32 %36, 0
  br i1 %37, label %38, label %54

38:                                               ; preds = %27
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 7
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 0
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %struct.GCObject, ptr %42, i32 0, i32 2
  %44 = load i8, ptr %43, align 1
  %45 = zext i8 %44 to i32
  %46 = and i32 %45, 24
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %48, label %54

48:                                               ; preds = %38
  %49 = load ptr, ptr %3, align 8
  %50 = load ptr, ptr %3, align 8
  %51 = getelementptr inbounds %struct.global_State, ptr %50, i32 0, i32 7
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %52, align 8
  call void @reallymarkobject(ptr noundef %49, ptr noundef %53)
  br label %54

54:                                               ; preds = %48, %38, %27
  %55 = load ptr, ptr %3, align 8
  call void @markmt(ptr noundef %55)
  %56 = load ptr, ptr %3, align 8
  %57 = call i64 @propagateall(ptr noundef %56)
  %58 = load i64, ptr %4, align 8
  %59 = add i64 %58, %57
  store i64 %59, ptr %4, align 8
  %60 = load ptr, ptr %3, align 8
  %61 = call i32 @remarkupvals(ptr noundef %60)
  %62 = sext i32 %61 to i64
  %63 = load i64, ptr %4, align 8
  %64 = add i64 %63, %62
  store i64 %64, ptr %4, align 8
  %65 = load ptr, ptr %3, align 8
  %66 = call i64 @propagateall(ptr noundef %65)
  %67 = load i64, ptr %4, align 8
  %68 = add i64 %67, %66
  store i64 %68, ptr %4, align 8
  %69 = load ptr, ptr %7, align 8
  %70 = load ptr, ptr %3, align 8
  %71 = getelementptr inbounds %struct.global_State, ptr %70, i32 0, i32 24
  store ptr %69, ptr %71, align 8
  %72 = load ptr, ptr %3, align 8
  %73 = call i64 @propagateall(ptr noundef %72)
  %74 = load i64, ptr %4, align 8
  %75 = add i64 %74, %73
  store i64 %75, ptr %4, align 8
  %76 = load ptr, ptr %3, align 8
  call void @convergeephemerons(ptr noundef %76)
  %77 = load ptr, ptr %3, align 8
  %78 = load ptr, ptr %3, align 8
  %79 = getelementptr inbounds %struct.global_State, ptr %78, i32 0, i32 26
  %80 = load ptr, ptr %79, align 8
  call void @clearbyvalues(ptr noundef %77, ptr noundef %80, ptr noundef null)
  %81 = load ptr, ptr %3, align 8
  %82 = load ptr, ptr %3, align 8
  %83 = getelementptr inbounds %struct.global_State, ptr %82, i32 0, i32 28
  %84 = load ptr, ptr %83, align 8
  call void @clearbyvalues(ptr noundef %81, ptr noundef %84, ptr noundef null)
  %85 = load ptr, ptr %3, align 8
  %86 = getelementptr inbounds %struct.global_State, ptr %85, i32 0, i32 26
  %87 = load ptr, ptr %86, align 8
  store ptr %87, ptr %5, align 8
  %88 = load ptr, ptr %3, align 8
  %89 = getelementptr inbounds %struct.global_State, ptr %88, i32 0, i32 28
  %90 = load ptr, ptr %89, align 8
  store ptr %90, ptr %6, align 8
  %91 = load ptr, ptr %3, align 8
  call void @separatetobefnz(ptr noundef %91, i32 noundef 0)
  %92 = load ptr, ptr %3, align 8
  %93 = call i64 @markbeingfnz(ptr noundef %92)
  %94 = load i64, ptr %4, align 8
  %95 = add i64 %94, %93
  store i64 %95, ptr %4, align 8
  %96 = load ptr, ptr %3, align 8
  %97 = call i64 @propagateall(ptr noundef %96)
  %98 = load i64, ptr %4, align 8
  %99 = add i64 %98, %97
  store i64 %99, ptr %4, align 8
  %100 = load ptr, ptr %3, align 8
  call void @convergeephemerons(ptr noundef %100)
  %101 = load ptr, ptr %3, align 8
  %102 = load ptr, ptr %3, align 8
  %103 = getelementptr inbounds %struct.global_State, ptr %102, i32 0, i32 27
  %104 = load ptr, ptr %103, align 8
  call void @clearbykeys(ptr noundef %101, ptr noundef %104)
  %105 = load ptr, ptr %3, align 8
  %106 = load ptr, ptr %3, align 8
  %107 = getelementptr inbounds %struct.global_State, ptr %106, i32 0, i32 28
  %108 = load ptr, ptr %107, align 8
  call void @clearbykeys(ptr noundef %105, ptr noundef %108)
  %109 = load ptr, ptr %3, align 8
  %110 = load ptr, ptr %3, align 8
  %111 = getelementptr inbounds %struct.global_State, ptr %110, i32 0, i32 26
  %112 = load ptr, ptr %111, align 8
  %113 = load ptr, ptr %5, align 8
  call void @clearbyvalues(ptr noundef %109, ptr noundef %112, ptr noundef %113)
  %114 = load ptr, ptr %3, align 8
  %115 = load ptr, ptr %3, align 8
  %116 = getelementptr inbounds %struct.global_State, ptr %115, i32 0, i32 28
  %117 = load ptr, ptr %116, align 8
  %118 = load ptr, ptr %6, align 8
  call void @clearbyvalues(ptr noundef %114, ptr noundef %117, ptr noundef %118)
  %119 = load ptr, ptr %3, align 8
  call void @luaS_clearcache(ptr noundef %119)
  %120 = load ptr, ptr %3, align 8
  %121 = getelementptr inbounds %struct.global_State, ptr %120, i32 0, i32 10
  %122 = load i8, ptr %121, align 4
  %123 = zext i8 %122 to i32
  %124 = xor i32 %123, 24
  %125 = trunc i32 %124 to i8
  %126 = load ptr, ptr %3, align 8
  %127 = getelementptr inbounds %struct.global_State, ptr %126, i32 0, i32 10
  store i8 %125, ptr %127, align 4
  %128 = load i64, ptr %4, align 8
  ret i64 %128
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @atomic2gen(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  call void @cleargraylists(ptr noundef %5)
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.global_State, ptr %6, i32 0, i32 11
  store i8 3, ptr %7, align 1
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 21
  call void @sweep2old(ptr noundef %8, ptr noundef %10)
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 21
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 31
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 32
  store ptr %13, ptr %17, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 33
  store ptr %13, ptr %19, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 34
  store ptr null, ptr %21, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 23
  call void @sweep2old(ptr noundef %22, ptr noundef %24)
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 23
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 35
  store ptr %27, ptr %29, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.global_State, ptr %30, i32 0, i32 36
  store ptr %27, ptr %31, align 8
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 37
  store ptr %27, ptr %33, align 8
  %34 = load ptr, ptr %3, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 29
  call void @sweep2old(ptr noundef %34, ptr noundef %36)
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.global_State, ptr %37, i32 0, i32 12
  store i8 1, ptr %38, align 2
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 5
  store i64 0, ptr %40, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.global_State, ptr %41, i32 0, i32 2
  %43 = load i64, ptr %42, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.global_State, ptr %44, i32 0, i32 3
  %46 = load i64, ptr %45, align 8
  %47 = add nsw i64 %43, %46
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.global_State, ptr %48, i32 0, i32 4
  store i64 %47, ptr %49, align 8
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  call void @finishgencycle(ptr noundef %50, ptr noundef %51)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setminordebt(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.global_State, ptr %4, i32 0, i32 2
  %6 = load i64, ptr %5, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 3
  %9 = load i64, ptr %8, align 8
  %10 = add nsw i64 %6, %9
  %11 = udiv i64 %10, 100
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 14
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i64
  %16 = mul nsw i64 %11, %15
  %17 = sub nsw i64 0, %16
  call void @luaE_setdebt(ptr noundef %3, i64 noundef %17)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @markmt(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 0, ptr %3, align 4
  br label %4

4:                                                ; preds = %37, %1
  %5 = load i32, ptr %3, align 4
  %6 = icmp slt i32 %5, 9
  br i1 %6, label %7, label %40

7:                                                ; preds = %4
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 43
  %10 = load i32, ptr %3, align 4
  %11 = sext i32 %10 to i64
  %12 = getelementptr inbounds [9 x ptr], ptr %9, i64 0, i64 %11
  %13 = load ptr, ptr %12, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %36

15:                                               ; preds = %7
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 43
  %18 = load i32, ptr %3, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds [9 x ptr], ptr %17, i64 0, i64 %19
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.Table, ptr %21, i32 0, i32 2
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = and i32 %24, 24
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %35

27:                                               ; preds = %15
  %28 = load ptr, ptr %2, align 8
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 43
  %31 = load i32, ptr %3, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds [9 x ptr], ptr %30, i64 0, i64 %32
  %34 = load ptr, ptr %33, align 8
  call void @reallymarkobject(ptr noundef %28, ptr noundef %34)
  br label %35

35:                                               ; preds = %27, %15
  br label %36

36:                                               ; preds = %35, %7
  br label %37

37:                                               ; preds = %36
  %38 = load i32, ptr %3, align 4
  %39 = add nsw i32 %38, 1
  store i32 %39, ptr %3, align 4
  br label %4, !llvm.loop !15

40:                                               ; preds = %4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @propagateall(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  store i64 0, ptr %3, align 8
  br label %4

4:                                                ; preds = %9, %1
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 24
  %7 = load ptr, ptr %6, align 8
  %8 = icmp ne ptr %7, null
  br i1 %8, label %9, label %14

9:                                                ; preds = %4
  %10 = load ptr, ptr %2, align 8
  %11 = call i64 @propagatemark(ptr noundef %10)
  %12 = load i64, ptr %3, align 8
  %13 = add i64 %12, %11
  store i64 %13, ptr %3, align 8
  br label %4, !llvm.loop !16

14:                                               ; preds = %4
  %15 = load i64, ptr %3, align 8
  ret i64 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @remarkupvals(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 38
  store ptr %8, ptr %4, align 8
  store i32 0, ptr %5, align 4
  br label %9

9:                                                ; preds = %91, %1
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %3, align 8
  %12 = icmp ne ptr %11, null
  br i1 %12, label %13, label %92

13:                                               ; preds = %9
  %14 = load i32, ptr %5, align 4
  %15 = add nsw i32 %14, 1
  store i32 %15, ptr %5, align 4
  %16 = load ptr, ptr %3, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 2
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = and i32 %19, 24
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %30, label %22

22:                                               ; preds = %13
  %23 = load ptr, ptr %3, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 11
  %25 = load ptr, ptr %24, align 8
  %26 = icmp ne ptr %25, null
  br i1 %26, label %27, label %30

27:                                               ; preds = %22
  %28 = load ptr, ptr %3, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 14
  store ptr %29, ptr %4, align 8
  br label %91

30:                                               ; preds = %22, %13
  %31 = load ptr, ptr %3, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 14
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %4, align 8
  store ptr %33, ptr %34, align 8
  %35 = load ptr, ptr %3, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 14
  store ptr %35, ptr %37, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %struct.lua_State, ptr %38, i32 0, i32 11
  %40 = load ptr, ptr %39, align 8
  store ptr %40, ptr %6, align 8
  br label %41

41:                                               ; preds = %85, %30
  %42 = load ptr, ptr %6, align 8
  %43 = icmp ne ptr %42, null
  br i1 %43, label %44, label %90

44:                                               ; preds = %41
  %45 = load i32, ptr %5, align 4
  %46 = add nsw i32 %45, 1
  store i32 %46, ptr %5, align 4
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.UpVal, ptr %47, i32 0, i32 2
  %49 = load i8, ptr %48, align 1
  %50 = zext i8 %49 to i32
  %51 = and i32 %50, 24
  %52 = icmp ne i32 %51, 0
  br i1 %52, label %84, label %53

53:                                               ; preds = %44
  %54 = load ptr, ptr %2, align 8
  %55 = getelementptr inbounds %struct.global_State, ptr %54, i32 0, i32 40
  %56 = load ptr, ptr %55, align 8
  %57 = load ptr, ptr %6, align 8
  %58 = getelementptr inbounds %struct.UpVal, ptr %57, i32 0, i32 3
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 1
  %61 = load i8, ptr %60, align 8
  %62 = zext i8 %61 to i32
  %63 = and i32 %62, 64
  %64 = icmp ne i32 %63, 0
  br i1 %64, label %65, label %83

65:                                               ; preds = %53
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds %struct.UpVal, ptr %66, i32 0, i32 3
  %68 = load ptr, ptr %67, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.GCObject, ptr %70, i32 0, i32 2
  %72 = load i8, ptr %71, align 1
  %73 = zext i8 %72 to i32
  %74 = and i32 %73, 24
  %75 = icmp ne i32 %74, 0
  br i1 %75, label %76, label %83

76:                                               ; preds = %65
  %77 = load ptr, ptr %2, align 8
  %78 = load ptr, ptr %6, align 8
  %79 = getelementptr inbounds %struct.UpVal, ptr %78, i32 0, i32 3
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds %struct.TValue, ptr %80, i32 0, i32 0
  %82 = load ptr, ptr %81, align 8
  call void @reallymarkobject(ptr noundef %77, ptr noundef %82)
  br label %83

83:                                               ; preds = %76, %65, %53
  br label %84

84:                                               ; preds = %83, %44
  br label %85

85:                                               ; preds = %84
  %86 = load ptr, ptr %6, align 8
  %87 = getelementptr inbounds %struct.UpVal, ptr %86, i32 0, i32 4
  %88 = getelementptr inbounds %struct.anon.6, ptr %87, i32 0, i32 0
  %89 = load ptr, ptr %88, align 8
  store ptr %89, ptr %6, align 8
  br label %41, !llvm.loop !17

90:                                               ; preds = %41
  br label %91

91:                                               ; preds = %90, %27
  br label %9, !llvm.loop !18

92:                                               ; preds = %9
  %93 = load i32, ptr %5, align 4
  ret i32 %93
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @convergeephemerons(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  store i32 0, ptr %4, align 4
  br label %8

8:                                                ; preds = %42, %1
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 27
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %6, align 8
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 27
  store ptr null, ptr %13, align 8
  store i32 0, ptr %3, align 4
  br label %14

14:                                               ; preds = %36, %8
  %15 = load ptr, ptr %6, align 8
  store ptr %15, ptr %5, align 8
  %16 = icmp ne ptr %15, null
  br i1 %16, label %17, label %37

17:                                               ; preds = %14
  %18 = load ptr, ptr %5, align 8
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.Table, ptr %19, i32 0, i32 10
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr %6, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 2
  %24 = load i8, ptr %23, align 1
  %25 = zext i8 %24 to i32
  %26 = or i32 %25, 32
  %27 = trunc i32 %26 to i8
  store i8 %27, ptr %23, align 1
  %28 = load ptr, ptr %2, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = load i32, ptr %4, align 4
  %31 = call i32 @traverseephemeron(ptr noundef %28, ptr noundef %29, i32 noundef %30)
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %36

33:                                               ; preds = %17
  %34 = load ptr, ptr %2, align 8
  %35 = call i64 @propagateall(ptr noundef %34)
  store i32 1, ptr %3, align 4
  br label %36

36:                                               ; preds = %33, %17
  br label %14, !llvm.loop !19

37:                                               ; preds = %14
  %38 = load i32, ptr %4, align 4
  %39 = icmp ne i32 %38, 0
  %40 = xor i1 %39, true
  %41 = zext i1 %40 to i32
  store i32 %41, ptr %4, align 4
  br label %42

42:                                               ; preds = %37
  %43 = load i32, ptr %3, align 4
  %44 = icmp ne i32 %43, 0
  br i1 %44, label %8, label %45, !llvm.loop !20

45:                                               ; preds = %42
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @clearbyvalues(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  br label %13

13:                                               ; preds = %108, %3
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = icmp ne ptr %14, %15
  br i1 %16, label %17, label %112

17:                                               ; preds = %13
  %18 = load ptr, ptr %5, align 8
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.Table, ptr %19, i32 0, i32 7
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.Table, ptr %22, i32 0, i32 4
  %24 = load i8, ptr %23, align 1
  %25 = zext i8 %24 to i32
  %26 = shl i32 1, %25
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds %union.Node, ptr %21, i64 %27
  store ptr %28, ptr %9, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = call i32 @luaH_realasize(ptr noundef %29)
  store i32 %30, ptr %11, align 4
  store i32 0, ptr %10, align 4
  br label %31

31:                                               ; preds = %62, %17
  %32 = load i32, ptr %10, align 4
  %33 = load i32, ptr %11, align 4
  %34 = icmp ult i32 %32, %33
  br i1 %34, label %35, label %65

35:                                               ; preds = %31
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.Table, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %10, align 4
  %40 = zext i32 %39 to i64
  %41 = getelementptr inbounds %struct.TValue, ptr %38, i64 %40
  store ptr %41, ptr %12, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = load ptr, ptr %12, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = zext i8 %45 to i32
  %47 = and i32 %46, 64
  %48 = icmp ne i32 %47, 0
  br i1 %48, label %49, label %53

49:                                               ; preds = %35
  %50 = load ptr, ptr %12, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 0
  %52 = load ptr, ptr %51, align 8
  br label %54

53:                                               ; preds = %35
  br label %54

54:                                               ; preds = %53, %49
  %55 = phi ptr [ %52, %49 ], [ null, %53 ]
  %56 = call i32 @iscleared(ptr noundef %42, ptr noundef %55)
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %58, label %61

58:                                               ; preds = %54
  %59 = load ptr, ptr %12, align 8
  %60 = getelementptr inbounds %struct.TValue, ptr %59, i32 0, i32 1
  store i8 16, ptr %60, align 8
  br label %61

61:                                               ; preds = %58, %54
  br label %62

62:                                               ; preds = %61
  %63 = load i32, ptr %10, align 4
  %64 = add i32 %63, 1
  store i32 %64, ptr %10, align 4
  br label %31, !llvm.loop !21

65:                                               ; preds = %31
  %66 = load ptr, ptr %7, align 8
  %67 = getelementptr inbounds %struct.Table, ptr %66, i32 0, i32 7
  %68 = load ptr, ptr %67, align 8
  %69 = getelementptr inbounds %union.Node, ptr %68, i64 0
  store ptr %69, ptr %8, align 8
  br label %70

70:                                               ; preds = %104, %65
  %71 = load ptr, ptr %8, align 8
  %72 = load ptr, ptr %9, align 8
  %73 = icmp ult ptr %71, %72
  br i1 %73, label %74, label %107

74:                                               ; preds = %70
  %75 = load ptr, ptr %4, align 8
  %76 = load ptr, ptr %8, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 1
  %78 = load i8, ptr %77, align 8
  %79 = zext i8 %78 to i32
  %80 = and i32 %79, 64
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %86

82:                                               ; preds = %74
  %83 = load ptr, ptr %8, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  br label %87

86:                                               ; preds = %74
  br label %87

87:                                               ; preds = %86, %82
  %88 = phi ptr [ %85, %82 ], [ null, %86 ]
  %89 = call i32 @iscleared(ptr noundef %75, ptr noundef %88)
  %90 = icmp ne i32 %89, 0
  br i1 %90, label %91, label %94

91:                                               ; preds = %87
  %92 = load ptr, ptr %8, align 8
  %93 = getelementptr inbounds %struct.TValue, ptr %92, i32 0, i32 1
  store i8 16, ptr %93, align 8
  br label %94

94:                                               ; preds = %91, %87
  %95 = load ptr, ptr %8, align 8
  %96 = getelementptr inbounds %struct.TValue, ptr %95, i32 0, i32 1
  %97 = load i8, ptr %96, align 8
  %98 = zext i8 %97 to i32
  %99 = and i32 %98, 15
  %100 = icmp eq i32 %99, 0
  br i1 %100, label %101, label %103

101:                                              ; preds = %94
  %102 = load ptr, ptr %8, align 8
  call void @clearkey(ptr noundef %102)
  br label %103

103:                                              ; preds = %101, %94
  br label %104

104:                                              ; preds = %103
  %105 = load ptr, ptr %8, align 8
  %106 = getelementptr inbounds %union.Node, ptr %105, i32 1
  store ptr %106, ptr %8, align 8
  br label %70, !llvm.loop !22

107:                                              ; preds = %70
  br label %108

108:                                              ; preds = %107
  %109 = load ptr, ptr %5, align 8
  %110 = getelementptr inbounds %struct.Table, ptr %109, i32 0, i32 10
  %111 = load ptr, ptr %110, align 8
  store ptr %111, ptr %5, align 8
  br label %13, !llvm.loop !23

112:                                              ; preds = %13
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @markbeingfnz(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  store i64 0, ptr %4, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 29
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  br label %8

8:                                                ; preds = %24, %1
  %9 = load ptr, ptr %3, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %28

11:                                               ; preds = %8
  %12 = load i64, ptr %4, align 8
  %13 = add i64 %12, 1
  store i64 %13, ptr %4, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.GCObject, ptr %14, i32 0, i32 2
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 24
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %11
  %21 = load ptr, ptr %2, align 8
  %22 = load ptr, ptr %3, align 8
  call void @reallymarkobject(ptr noundef %21, ptr noundef %22)
  br label %23

23:                                               ; preds = %20, %11
  br label %24

24:                                               ; preds = %23
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.GCObject, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  store ptr %27, ptr %3, align 8
  br label %8, !llvm.loop !24

28:                                               ; preds = %8
  %29 = load i64, ptr %4, align 8
  ret i64 %29
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @clearbykeys(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  br label %8

8:                                                ; preds = %65, %2
  %9 = load ptr, ptr %4, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %69

11:                                               ; preds = %8
  %12 = load ptr, ptr %4, align 8
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 7
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.Table, ptr %16, i32 0, i32 4
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = shl i32 1, %19
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds %union.Node, ptr %15, i64 %21
  store ptr %22, ptr %6, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.Table, ptr %23, i32 0, i32 7
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.Node, ptr %25, i64 0
  store ptr %26, ptr %7, align 8
  br label %27

27:                                               ; preds = %61, %11
  %28 = load ptr, ptr %7, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = icmp ult ptr %28, %29
  br i1 %30, label %31, label %64

31:                                               ; preds = %27
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.NodeKey, ptr %33, i32 0, i32 2
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  %37 = and i32 %36, 64
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %39, label %43

39:                                               ; preds = %31
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.NodeKey, ptr %40, i32 0, i32 4
  %42 = load ptr, ptr %41, align 8
  br label %44

43:                                               ; preds = %31
  br label %44

44:                                               ; preds = %43, %39
  %45 = phi ptr [ %42, %39 ], [ null, %43 ]
  %46 = call i32 @iscleared(ptr noundef %32, ptr noundef %45)
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %48, label %51

48:                                               ; preds = %44
  %49 = load ptr, ptr %7, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 1
  store i8 16, ptr %50, align 8
  br label %51

51:                                               ; preds = %48, %44
  %52 = load ptr, ptr %7, align 8
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 1
  %54 = load i8, ptr %53, align 8
  %55 = zext i8 %54 to i32
  %56 = and i32 %55, 15
  %57 = icmp eq i32 %56, 0
  br i1 %57, label %58, label %60

58:                                               ; preds = %51
  %59 = load ptr, ptr %7, align 8
  call void @clearkey(ptr noundef %59)
  br label %60

60:                                               ; preds = %58, %51
  br label %61

61:                                               ; preds = %60
  %62 = load ptr, ptr %7, align 8
  %63 = getelementptr inbounds %union.Node, ptr %62, i32 1
  store ptr %63, ptr %7, align 8
  br label %27, !llvm.loop !25

64:                                               ; preds = %27
  br label %65

65:                                               ; preds = %64
  %66 = load ptr, ptr %4, align 8
  %67 = getelementptr inbounds %struct.Table, ptr %66, i32 0, i32 10
  %68 = load ptr, ptr %67, align 8
  store ptr %68, ptr %4, align 8
  br label %8, !llvm.loop !26

69:                                               ; preds = %8
  ret void
}

declare hidden void @luaS_clearcache(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @propagatemark(ptr noundef %0) #0 {
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 24
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.GCObject, ptr %8, i32 0, i32 2
  %10 = load i8, ptr %9, align 1
  %11 = zext i8 %10 to i32
  %12 = or i32 %11, 32
  %13 = trunc i32 %12 to i8
  store i8 %13, ptr %9, align 1
  %14 = load ptr, ptr %4, align 8
  %15 = call ptr @getgclist(ptr noundef %14)
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 24
  store ptr %16, ptr %18, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.GCObject, ptr %19, i32 0, i32 1
  %21 = load i8, ptr %20, align 8
  %22 = zext i8 %21 to i32
  switch i32 %22, label %52 [
    i32 5, label %23
    i32 7, label %27
    i32 6, label %32
    i32 38, label %37
    i32 10, label %42
    i32 8, label %47
  ]

23:                                               ; preds = %1
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = call i64 @traversetable(ptr noundef %24, ptr noundef %25)
  store i64 %26, ptr %2, align 8
  br label %53

27:                                               ; preds = %1
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = call i32 @traverseudata(ptr noundef %28, ptr noundef %29)
  %31 = sext i32 %30 to i64
  store i64 %31, ptr %2, align 8
  br label %53

32:                                               ; preds = %1
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = call i32 @traverseLclosure(ptr noundef %33, ptr noundef %34)
  %36 = sext i32 %35 to i64
  store i64 %36, ptr %2, align 8
  br label %53

37:                                               ; preds = %1
  %38 = load ptr, ptr %3, align 8
  %39 = load ptr, ptr %4, align 8
  %40 = call i32 @traverseCclosure(ptr noundef %38, ptr noundef %39)
  %41 = sext i32 %40 to i64
  store i64 %41, ptr %2, align 8
  br label %53

42:                                               ; preds = %1
  %43 = load ptr, ptr %3, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = call i32 @traverseproto(ptr noundef %43, ptr noundef %44)
  %46 = sext i32 %45 to i64
  store i64 %46, ptr %2, align 8
  br label %53

47:                                               ; preds = %1
  %48 = load ptr, ptr %3, align 8
  %49 = load ptr, ptr %4, align 8
  %50 = call i32 @traversethread(ptr noundef %48, ptr noundef %49)
  %51 = sext i32 %50 to i64
  store i64 %51, ptr %2, align 8
  br label %53

52:                                               ; preds = %1
  store i64 0, ptr %2, align 8
  br label %53

53:                                               ; preds = %52, %47, %42, %37, %32, %27, %23
  %54 = load i64, ptr %2, align 8
  ret i64 %54
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @traversetable(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.Table, ptr %9, i32 0, i32 9
  %11 = load ptr, ptr %10, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %2
  br label %35

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.Table, ptr %15, i32 0, i32 9
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.Table, ptr %17, i32 0, i32 3
  %19 = load i8, ptr %18, align 2
  %20 = zext i8 %19 to i32
  %21 = and i32 %20, 8
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %14
  br label %33

24:                                               ; preds = %14
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.Table, ptr %25, i32 0, i32 9
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %3, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 42
  %30 = getelementptr inbounds [25 x ptr], ptr %29, i64 0, i64 3
  %31 = load ptr, ptr %30, align 8
  %32 = call ptr @luaT_gettm(ptr noundef %27, i32 noundef 3, ptr noundef %31)
  br label %33

33:                                               ; preds = %24, %23
  %34 = phi ptr [ null, %23 ], [ %32, %24 ]
  br label %35

35:                                               ; preds = %33, %13
  %36 = phi ptr [ null, %13 ], [ %34, %33 ]
  store ptr %36, ptr %7, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.Table, ptr %37, i32 0, i32 9
  %39 = load ptr, ptr %38, align 8
  %40 = icmp ne ptr %39, null
  br i1 %40, label %41, label %56

41:                                               ; preds = %35
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.Table, ptr %42, i32 0, i32 9
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.Table, ptr %44, i32 0, i32 2
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = and i32 %47, 24
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %55

50:                                               ; preds = %41
  %51 = load ptr, ptr %3, align 8
  %52 = load ptr, ptr %4, align 8
  %53 = getelementptr inbounds %struct.Table, ptr %52, i32 0, i32 9
  %54 = load ptr, ptr %53, align 8
  call void @reallymarkobject(ptr noundef %51, ptr noundef %54)
  br label %55

55:                                               ; preds = %50, %41
  br label %56

56:                                               ; preds = %55, %35
  %57 = load ptr, ptr %7, align 8
  %58 = icmp ne ptr %57, null
  br i1 %58, label %59, label %105

59:                                               ; preds = %56
  %60 = load ptr, ptr %7, align 8
  %61 = getelementptr inbounds %struct.TValue, ptr %60, i32 0, i32 1
  %62 = load i8, ptr %61, align 8
  %63 = zext i8 %62 to i32
  %64 = icmp eq i32 %63, 68
  br i1 %64, label %65, label %105

65:                                               ; preds = %59
  %66 = load ptr, ptr %7, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 0
  %68 = load ptr, ptr %67, align 8
  store ptr %68, ptr %8, align 8
  %69 = load ptr, ptr %8, align 8
  %70 = getelementptr inbounds %struct.TString, ptr %69, i32 0, i32 7
  %71 = getelementptr inbounds [1 x i8], ptr %70, i64 0, i64 0
  %72 = call ptr @strchr(ptr noundef %71, i32 noundef 107) #4
  store ptr %72, ptr %5, align 8
  %73 = load ptr, ptr %8, align 8
  %74 = getelementptr inbounds %struct.TString, ptr %73, i32 0, i32 7
  %75 = getelementptr inbounds [1 x i8], ptr %74, i64 0, i64 0
  %76 = call ptr @strchr(ptr noundef %75, i32 noundef 118) #4
  store ptr %76, ptr %6, align 8
  %77 = load ptr, ptr %5, align 8
  %78 = icmp ne ptr %77, null
  br i1 %78, label %82, label %79

79:                                               ; preds = %65
  %80 = load ptr, ptr %6, align 8
  %81 = icmp ne ptr %80, null
  br label %82

82:                                               ; preds = %79, %65
  %83 = phi i1 [ true, %65 ], [ %81, %79 ]
  br i1 %83, label %84, label %105

84:                                               ; preds = %82
  %85 = load ptr, ptr %5, align 8
  %86 = icmp ne ptr %85, null
  br i1 %86, label %90, label %87

87:                                               ; preds = %84
  %88 = load ptr, ptr %3, align 8
  %89 = load ptr, ptr %4, align 8
  call void @traverseweakvalue(ptr noundef %88, ptr noundef %89)
  br label %104

90:                                               ; preds = %84
  %91 = load ptr, ptr %6, align 8
  %92 = icmp ne ptr %91, null
  br i1 %92, label %97, label %93

93:                                               ; preds = %90
  %94 = load ptr, ptr %3, align 8
  %95 = load ptr, ptr %4, align 8
  %96 = call i32 @traverseephemeron(ptr noundef %94, ptr noundef %95, i32 noundef 0)
  br label %103

97:                                               ; preds = %90
  %98 = load ptr, ptr %4, align 8
  %99 = load ptr, ptr %4, align 8
  %100 = getelementptr inbounds %struct.Table, ptr %99, i32 0, i32 10
  %101 = load ptr, ptr %3, align 8
  %102 = getelementptr inbounds %struct.global_State, ptr %101, i32 0, i32 28
  call void @linkgclist_(ptr noundef %98, ptr noundef %100, ptr noundef %102)
  br label %103

103:                                              ; preds = %97, %93
  br label %104

104:                                              ; preds = %103, %87
  br label %108

105:                                              ; preds = %82, %59, %56
  %106 = load ptr, ptr %3, align 8
  %107 = load ptr, ptr %4, align 8
  call void @traversestrongtable(ptr noundef %106, ptr noundef %107)
  br label %108

108:                                              ; preds = %105, %104
  %109 = load ptr, ptr %4, align 8
  %110 = getelementptr inbounds %struct.Table, ptr %109, i32 0, i32 5
  %111 = load i32, ptr %110, align 4
  %112 = add i32 1, %111
  %113 = load ptr, ptr %4, align 8
  %114 = getelementptr inbounds %struct.Table, ptr %113, i32 0, i32 8
  %115 = load ptr, ptr %114, align 8
  %116 = icmp eq ptr %115, null
  br i1 %116, label %117, label %118

117:                                              ; preds = %108
  br label %124

118:                                              ; preds = %108
  %119 = load ptr, ptr %4, align 8
  %120 = getelementptr inbounds %struct.Table, ptr %119, i32 0, i32 4
  %121 = load i8, ptr %120, align 1
  %122 = zext i8 %121 to i32
  %123 = shl i32 1, %122
  br label %124

124:                                              ; preds = %118, %117
  %125 = phi i32 [ 0, %117 ], [ %123, %118 ]
  %126 = mul nsw i32 2, %125
  %127 = add i32 %112, %126
  %128 = zext i32 %127 to i64
  ret i64 %128
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traverseudata(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Udata, ptr %6, i32 0, i32 5
  %8 = load ptr, ptr %7, align 8
  %9 = icmp ne ptr %8, null
  br i1 %9, label %10, label %25

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Udata, ptr %11, i32 0, i32 5
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.Table, ptr %13, i32 0, i32 2
  %15 = load i8, ptr %14, align 1
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 24
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %24

19:                                               ; preds = %10
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.Udata, ptr %21, i32 0, i32 5
  %23 = load ptr, ptr %22, align 8
  call void @reallymarkobject(ptr noundef %20, ptr noundef %23)
  br label %24

24:                                               ; preds = %19, %10
  br label %25

25:                                               ; preds = %24, %2
  store i32 0, ptr %5, align 4
  br label %26

26:                                               ; preds = %70, %25
  %27 = load i32, ptr %5, align 4
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Udata, ptr %28, i32 0, i32 3
  %30 = load i16, ptr %29, align 2
  %31 = zext i16 %30 to i32
  %32 = icmp slt i32 %27, %31
  br i1 %32, label %33, label %73

33:                                               ; preds = %26
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.global_State, ptr %34, i32 0, i32 40
  %36 = load ptr, ptr %35, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.Udata, ptr %37, i32 0, i32 7
  %39 = load i32, ptr %5, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds [1 x %union.UValue], ptr %38, i64 0, i64 %40
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 1
  %43 = load i8, ptr %42, align 8
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 64
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %69

47:                                               ; preds = %33
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.Udata, ptr %48, i32 0, i32 7
  %50 = load i32, ptr %5, align 4
  %51 = sext i32 %50 to i64
  %52 = getelementptr inbounds [1 x %union.UValue], ptr %49, i64 0, i64 %51
  %53 = getelementptr inbounds %struct.TValue, ptr %52, i32 0, i32 0
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr inbounds %struct.GCObject, ptr %54, i32 0, i32 2
  %56 = load i8, ptr %55, align 1
  %57 = zext i8 %56 to i32
  %58 = and i32 %57, 24
  %59 = icmp ne i32 %58, 0
  br i1 %59, label %60, label %69

60:                                               ; preds = %47
  %61 = load ptr, ptr %3, align 8
  %62 = load ptr, ptr %4, align 8
  %63 = getelementptr inbounds %struct.Udata, ptr %62, i32 0, i32 7
  %64 = load i32, ptr %5, align 4
  %65 = sext i32 %64 to i64
  %66 = getelementptr inbounds [1 x %union.UValue], ptr %63, i64 0, i64 %65
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 0
  %68 = load ptr, ptr %67, align 8
  call void @reallymarkobject(ptr noundef %61, ptr noundef %68)
  br label %69

69:                                               ; preds = %60, %47, %33
  br label %70

70:                                               ; preds = %69
  %71 = load i32, ptr %5, align 4
  %72 = add nsw i32 %71, 1
  store i32 %72, ptr %5, align 4
  br label %26, !llvm.loop !27

73:                                               ; preds = %26
  %74 = load ptr, ptr %3, align 8
  %75 = load ptr, ptr %4, align 8
  call void @genlink(ptr noundef %74, ptr noundef %75)
  %76 = load ptr, ptr %4, align 8
  %77 = getelementptr inbounds %struct.Udata, ptr %76, i32 0, i32 3
  %78 = load i16, ptr %77, align 2
  %79 = zext i16 %78 to i32
  %80 = add nsw i32 1, %79
  ret i32 %80
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traverseLclosure(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.LClosure, ptr %7, i32 0, i32 5
  %9 = load ptr, ptr %8, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %26

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LClosure, ptr %12, i32 0, i32 5
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Proto, ptr %14, i32 0, i32 2
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 24
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %11
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.LClosure, ptr %22, i32 0, i32 5
  %24 = load ptr, ptr %23, align 8
  call void @reallymarkobject(ptr noundef %21, ptr noundef %24)
  br label %25

25:                                               ; preds = %20, %11
  br label %26

26:                                               ; preds = %25, %2
  store i32 0, ptr %5, align 4
  br label %27

27:                                               ; preds = %55, %26
  %28 = load i32, ptr %5, align 4
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.LClosure, ptr %29, i32 0, i32 3
  %31 = load i8, ptr %30, align 2
  %32 = zext i8 %31 to i32
  %33 = icmp slt i32 %28, %32
  br i1 %33, label %34, label %58

34:                                               ; preds = %27
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.LClosure, ptr %35, i32 0, i32 6
  %37 = load i32, ptr %5, align 4
  %38 = sext i32 %37 to i64
  %39 = getelementptr inbounds [1 x ptr], ptr %36, i64 0, i64 %38
  %40 = load ptr, ptr %39, align 8
  store ptr %40, ptr %6, align 8
  %41 = load ptr, ptr %6, align 8
  %42 = icmp ne ptr %41, null
  br i1 %42, label %43, label %54

43:                                               ; preds = %34
  %44 = load ptr, ptr %6, align 8
  %45 = getelementptr inbounds %struct.UpVal, ptr %44, i32 0, i32 2
  %46 = load i8, ptr %45, align 1
  %47 = zext i8 %46 to i32
  %48 = and i32 %47, 24
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %53

50:                                               ; preds = %43
  %51 = load ptr, ptr %3, align 8
  %52 = load ptr, ptr %6, align 8
  call void @reallymarkobject(ptr noundef %51, ptr noundef %52)
  br label %53

53:                                               ; preds = %50, %43
  br label %54

54:                                               ; preds = %53, %34
  br label %55

55:                                               ; preds = %54
  %56 = load i32, ptr %5, align 4
  %57 = add nsw i32 %56, 1
  store i32 %57, ptr %5, align 4
  br label %27, !llvm.loop !28

58:                                               ; preds = %27
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.LClosure, ptr %59, i32 0, i32 3
  %61 = load i8, ptr %60, align 2
  %62 = zext i8 %61 to i32
  %63 = add nsw i32 1, %62
  ret i32 %63
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traverseCclosure(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %5, align 4
  br label %6

6:                                                ; preds = %50, %2
  %7 = load i32, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.CClosure, ptr %8, i32 0, i32 3
  %10 = load i8, ptr %9, align 2
  %11 = zext i8 %10 to i32
  %12 = icmp slt i32 %7, %11
  br i1 %12, label %13, label %53

13:                                               ; preds = %6
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 40
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.CClosure, ptr %17, i32 0, i32 6
  %19 = load i32, ptr %5, align 4
  %20 = sext i32 %19 to i64
  %21 = getelementptr inbounds [1 x %struct.TValue], ptr %18, i64 0, i64 %20
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 1
  %23 = load i8, ptr %22, align 8
  %24 = zext i8 %23 to i32
  %25 = and i32 %24, 64
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %49

27:                                               ; preds = %13
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.CClosure, ptr %28, i32 0, i32 6
  %30 = load i32, ptr %5, align 4
  %31 = sext i32 %30 to i64
  %32 = getelementptr inbounds [1 x %struct.TValue], ptr %29, i64 0, i64 %31
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 0
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %struct.GCObject, ptr %34, i32 0, i32 2
  %36 = load i8, ptr %35, align 1
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 24
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %49

40:                                               ; preds = %27
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.CClosure, ptr %42, i32 0, i32 6
  %44 = load i32, ptr %5, align 4
  %45 = sext i32 %44 to i64
  %46 = getelementptr inbounds [1 x %struct.TValue], ptr %43, i64 0, i64 %45
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  call void @reallymarkobject(ptr noundef %41, ptr noundef %48)
  br label %49

49:                                               ; preds = %40, %27, %13
  br label %50

50:                                               ; preds = %49
  %51 = load i32, ptr %5, align 4
  %52 = add nsw i32 %51, 1
  store i32 %52, ptr %5, align 4
  br label %6, !llvm.loop !29

53:                                               ; preds = %6
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.CClosure, ptr %54, i32 0, i32 3
  %56 = load i8, ptr %55, align 2
  %57 = zext i8 %56 to i32
  %58 = add nsw i32 1, %57
  ret i32 %58
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traverseproto(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Proto, ptr %6, i32 0, i32 22
  %8 = load ptr, ptr %7, align 8
  %9 = icmp ne ptr %8, null
  br i1 %9, label %10, label %25

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Proto, ptr %11, i32 0, i32 22
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.TString, ptr %13, i32 0, i32 2
  %15 = load i8, ptr %14, align 1
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 24
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %24

19:                                               ; preds = %10
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 22
  %23 = load ptr, ptr %22, align 8
  call void @reallymarkobject(ptr noundef %20, ptr noundef %23)
  br label %24

24:                                               ; preds = %19, %10
  br label %25

25:                                               ; preds = %24, %2
  store i32 0, ptr %5, align 4
  br label %26

26:                                               ; preds = %72, %25
  %27 = load i32, ptr %5, align 4
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.Proto, ptr %28, i32 0, i32 7
  %30 = load i32, ptr %29, align 4
  %31 = icmp slt i32 %27, %30
  br i1 %31, label %32, label %75

32:                                               ; preds = %26
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.global_State, ptr %33, i32 0, i32 40
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.Proto, ptr %36, i32 0, i32 15
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %5, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds %struct.TValue, ptr %38, i64 %40
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 1
  %43 = load i8, ptr %42, align 8
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 64
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %71

47:                                               ; preds = %32
  %48 = load ptr, ptr %4, align 8
  %49 = getelementptr inbounds %struct.Proto, ptr %48, i32 0, i32 15
  %50 = load ptr, ptr %49, align 8
  %51 = load i32, ptr %5, align 4
  %52 = sext i32 %51 to i64
  %53 = getelementptr inbounds %struct.TValue, ptr %50, i64 %52
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  %55 = load ptr, ptr %54, align 8
  %56 = getelementptr inbounds %struct.GCObject, ptr %55, i32 0, i32 2
  %57 = load i8, ptr %56, align 1
  %58 = zext i8 %57 to i32
  %59 = and i32 %58, 24
  %60 = icmp ne i32 %59, 0
  br i1 %60, label %61, label %71

61:                                               ; preds = %47
  %62 = load ptr, ptr %3, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.Proto, ptr %63, i32 0, i32 15
  %65 = load ptr, ptr %64, align 8
  %66 = load i32, ptr %5, align 4
  %67 = sext i32 %66 to i64
  %68 = getelementptr inbounds %struct.TValue, ptr %65, i64 %67
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  %70 = load ptr, ptr %69, align 8
  call void @reallymarkobject(ptr noundef %62, ptr noundef %70)
  br label %71

71:                                               ; preds = %61, %47, %32
  br label %72

72:                                               ; preds = %71
  %73 = load i32, ptr %5, align 4
  %74 = add nsw i32 %73, 1
  store i32 %74, ptr %5, align 4
  br label %26, !llvm.loop !30

75:                                               ; preds = %26
  store i32 0, ptr %5, align 4
  br label %76

76:                                               ; preds = %118, %75
  %77 = load i32, ptr %5, align 4
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.Proto, ptr %78, i32 0, i32 6
  %80 = load i32, ptr %79, align 8
  %81 = icmp slt i32 %77, %80
  br i1 %81, label %82, label %121

82:                                               ; preds = %76
  %83 = load ptr, ptr %4, align 8
  %84 = getelementptr inbounds %struct.Proto, ptr %83, i32 0, i32 18
  %85 = load ptr, ptr %84, align 8
  %86 = load i32, ptr %5, align 4
  %87 = sext i32 %86 to i64
  %88 = getelementptr inbounds %struct.Upvaldesc, ptr %85, i64 %87
  %89 = getelementptr inbounds %struct.Upvaldesc, ptr %88, i32 0, i32 0
  %90 = load ptr, ptr %89, align 8
  %91 = icmp ne ptr %90, null
  br i1 %91, label %92, label %117

92:                                               ; preds = %82
  %93 = load ptr, ptr %4, align 8
  %94 = getelementptr inbounds %struct.Proto, ptr %93, i32 0, i32 18
  %95 = load ptr, ptr %94, align 8
  %96 = load i32, ptr %5, align 4
  %97 = sext i32 %96 to i64
  %98 = getelementptr inbounds %struct.Upvaldesc, ptr %95, i64 %97
  %99 = getelementptr inbounds %struct.Upvaldesc, ptr %98, i32 0, i32 0
  %100 = load ptr, ptr %99, align 8
  %101 = getelementptr inbounds %struct.TString, ptr %100, i32 0, i32 2
  %102 = load i8, ptr %101, align 1
  %103 = zext i8 %102 to i32
  %104 = and i32 %103, 24
  %105 = icmp ne i32 %104, 0
  br i1 %105, label %106, label %116

106:                                              ; preds = %92
  %107 = load ptr, ptr %3, align 8
  %108 = load ptr, ptr %4, align 8
  %109 = getelementptr inbounds %struct.Proto, ptr %108, i32 0, i32 18
  %110 = load ptr, ptr %109, align 8
  %111 = load i32, ptr %5, align 4
  %112 = sext i32 %111 to i64
  %113 = getelementptr inbounds %struct.Upvaldesc, ptr %110, i64 %112
  %114 = getelementptr inbounds %struct.Upvaldesc, ptr %113, i32 0, i32 0
  %115 = load ptr, ptr %114, align 8
  call void @reallymarkobject(ptr noundef %107, ptr noundef %115)
  br label %116

116:                                              ; preds = %106, %92
  br label %117

117:                                              ; preds = %116, %82
  br label %118

118:                                              ; preds = %117
  %119 = load i32, ptr %5, align 4
  %120 = add nsw i32 %119, 1
  store i32 %120, ptr %5, align 4
  br label %76, !llvm.loop !31

121:                                              ; preds = %76
  store i32 0, ptr %5, align 4
  br label %122

122:                                              ; preds = %161, %121
  %123 = load i32, ptr %5, align 4
  %124 = load ptr, ptr %4, align 8
  %125 = getelementptr inbounds %struct.Proto, ptr %124, i32 0, i32 10
  %126 = load i32, ptr %125, align 8
  %127 = icmp slt i32 %123, %126
  br i1 %127, label %128, label %164

128:                                              ; preds = %122
  %129 = load ptr, ptr %4, align 8
  %130 = getelementptr inbounds %struct.Proto, ptr %129, i32 0, i32 17
  %131 = load ptr, ptr %130, align 8
  %132 = load i32, ptr %5, align 4
  %133 = sext i32 %132 to i64
  %134 = getelementptr inbounds ptr, ptr %131, i64 %133
  %135 = load ptr, ptr %134, align 8
  %136 = icmp ne ptr %135, null
  br i1 %136, label %137, label %160

137:                                              ; preds = %128
  %138 = load ptr, ptr %4, align 8
  %139 = getelementptr inbounds %struct.Proto, ptr %138, i32 0, i32 17
  %140 = load ptr, ptr %139, align 8
  %141 = load i32, ptr %5, align 4
  %142 = sext i32 %141 to i64
  %143 = getelementptr inbounds ptr, ptr %140, i64 %142
  %144 = load ptr, ptr %143, align 8
  %145 = getelementptr inbounds %struct.Proto, ptr %144, i32 0, i32 2
  %146 = load i8, ptr %145, align 1
  %147 = zext i8 %146 to i32
  %148 = and i32 %147, 24
  %149 = icmp ne i32 %148, 0
  br i1 %149, label %150, label %159

150:                                              ; preds = %137
  %151 = load ptr, ptr %3, align 8
  %152 = load ptr, ptr %4, align 8
  %153 = getelementptr inbounds %struct.Proto, ptr %152, i32 0, i32 17
  %154 = load ptr, ptr %153, align 8
  %155 = load i32, ptr %5, align 4
  %156 = sext i32 %155 to i64
  %157 = getelementptr inbounds ptr, ptr %154, i64 %156
  %158 = load ptr, ptr %157, align 8
  call void @reallymarkobject(ptr noundef %151, ptr noundef %158)
  br label %159

159:                                              ; preds = %150, %137
  br label %160

160:                                              ; preds = %159, %128
  br label %161

161:                                              ; preds = %160
  %162 = load i32, ptr %5, align 4
  %163 = add nsw i32 %162, 1
  store i32 %163, ptr %5, align 4
  br label %122, !llvm.loop !32

164:                                              ; preds = %122
  store i32 0, ptr %5, align 4
  br label %165

165:                                              ; preds = %207, %164
  %166 = load i32, ptr %5, align 4
  %167 = load ptr, ptr %4, align 8
  %168 = getelementptr inbounds %struct.Proto, ptr %167, i32 0, i32 11
  %169 = load i32, ptr %168, align 4
  %170 = icmp slt i32 %166, %169
  br i1 %170, label %171, label %210

171:                                              ; preds = %165
  %172 = load ptr, ptr %4, align 8
  %173 = getelementptr inbounds %struct.Proto, ptr %172, i32 0, i32 21
  %174 = load ptr, ptr %173, align 8
  %175 = load i32, ptr %5, align 4
  %176 = sext i32 %175 to i64
  %177 = getelementptr inbounds %struct.LocVar, ptr %174, i64 %176
  %178 = getelementptr inbounds %struct.LocVar, ptr %177, i32 0, i32 0
  %179 = load ptr, ptr %178, align 8
  %180 = icmp ne ptr %179, null
  br i1 %180, label %181, label %206

181:                                              ; preds = %171
  %182 = load ptr, ptr %4, align 8
  %183 = getelementptr inbounds %struct.Proto, ptr %182, i32 0, i32 21
  %184 = load ptr, ptr %183, align 8
  %185 = load i32, ptr %5, align 4
  %186 = sext i32 %185 to i64
  %187 = getelementptr inbounds %struct.LocVar, ptr %184, i64 %186
  %188 = getelementptr inbounds %struct.LocVar, ptr %187, i32 0, i32 0
  %189 = load ptr, ptr %188, align 8
  %190 = getelementptr inbounds %struct.TString, ptr %189, i32 0, i32 2
  %191 = load i8, ptr %190, align 1
  %192 = zext i8 %191 to i32
  %193 = and i32 %192, 24
  %194 = icmp ne i32 %193, 0
  br i1 %194, label %195, label %205

195:                                              ; preds = %181
  %196 = load ptr, ptr %3, align 8
  %197 = load ptr, ptr %4, align 8
  %198 = getelementptr inbounds %struct.Proto, ptr %197, i32 0, i32 21
  %199 = load ptr, ptr %198, align 8
  %200 = load i32, ptr %5, align 4
  %201 = sext i32 %200 to i64
  %202 = getelementptr inbounds %struct.LocVar, ptr %199, i64 %201
  %203 = getelementptr inbounds %struct.LocVar, ptr %202, i32 0, i32 0
  %204 = load ptr, ptr %203, align 8
  call void @reallymarkobject(ptr noundef %196, ptr noundef %204)
  br label %205

205:                                              ; preds = %195, %181
  br label %206

206:                                              ; preds = %205, %171
  br label %207

207:                                              ; preds = %206
  %208 = load i32, ptr %5, align 4
  %209 = add nsw i32 %208, 1
  store i32 %209, ptr %5, align 4
  br label %165, !llvm.loop !33

210:                                              ; preds = %165
  %211 = load ptr, ptr %4, align 8
  %212 = getelementptr inbounds %struct.Proto, ptr %211, i32 0, i32 7
  %213 = load i32, ptr %212, align 4
  %214 = add nsw i32 1, %213
  %215 = load ptr, ptr %4, align 8
  %216 = getelementptr inbounds %struct.Proto, ptr %215, i32 0, i32 6
  %217 = load i32, ptr %216, align 8
  %218 = add nsw i32 %214, %217
  %219 = load ptr, ptr %4, align 8
  %220 = getelementptr inbounds %struct.Proto, ptr %219, i32 0, i32 10
  %221 = load i32, ptr %220, align 8
  %222 = add nsw i32 %218, %221
  %223 = load ptr, ptr %4, align 8
  %224 = getelementptr inbounds %struct.Proto, ptr %223, i32 0, i32 11
  %225 = load i32, ptr %224, align 4
  %226 = add nsw i32 %222, %225
  ret i32 %226
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traversethread(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 10
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 2
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = and i32 %14, 7
  %16 = icmp sgt i32 %15, 1
  br i1 %16, label %23, label %17

17:                                               ; preds = %2
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 11
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i32
  %22 = icmp eq i32 %21, 0
  br i1 %22, label %23, label %29

23:                                               ; preds = %17, %2
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 13
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 25
  call void @linkgclist_(ptr noundef %24, ptr noundef %26, ptr noundef %28)
  br label %29

29:                                               ; preds = %23, %17
  %30 = load ptr, ptr %7, align 8
  %31 = icmp eq ptr %30, null
  br i1 %31, label %32, label %33

32:                                               ; preds = %29
  store i32 1, ptr %3, align 4
  br label %155

33:                                               ; preds = %29
  br label %34

34:                                               ; preds = %65, %33
  %35 = load ptr, ptr %7, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = icmp ult ptr %35, %38
  br i1 %39, label %40, label %68

40:                                               ; preds = %34
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.global_State, ptr %41, i32 0, i32 40
  %43 = load ptr, ptr %42, align 8
  %44 = load ptr, ptr %7, align 8
  %45 = getelementptr inbounds %struct.TValue, ptr %44, i32 0, i32 1
  %46 = load i8, ptr %45, align 8
  %47 = zext i8 %46 to i32
  %48 = and i32 %47, 64
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %64

50:                                               ; preds = %40
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %52, align 8
  %54 = getelementptr inbounds %struct.GCObject, ptr %53, i32 0, i32 2
  %55 = load i8, ptr %54, align 1
  %56 = zext i8 %55 to i32
  %57 = and i32 %56, 24
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %64

59:                                               ; preds = %50
  %60 = load ptr, ptr %4, align 8
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  call void @reallymarkobject(ptr noundef %60, ptr noundef %63)
  br label %64

64:                                               ; preds = %59, %50, %40
  br label %65

65:                                               ; preds = %64
  %66 = load ptr, ptr %7, align 8
  %67 = getelementptr inbounds %union.StackValue, ptr %66, i32 1
  store ptr %67, ptr %7, align 8
  br label %34, !llvm.loop !34

68:                                               ; preds = %34
  %69 = load ptr, ptr %5, align 8
  %70 = getelementptr inbounds %struct.lua_State, ptr %69, i32 0, i32 11
  %71 = load ptr, ptr %70, align 8
  store ptr %71, ptr %6, align 8
  br label %72

72:                                               ; preds = %86, %68
  %73 = load ptr, ptr %6, align 8
  %74 = icmp ne ptr %73, null
  br i1 %74, label %75, label %91

75:                                               ; preds = %72
  %76 = load ptr, ptr %6, align 8
  %77 = getelementptr inbounds %struct.UpVal, ptr %76, i32 0, i32 2
  %78 = load i8, ptr %77, align 1
  %79 = zext i8 %78 to i32
  %80 = and i32 %79, 24
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %85

82:                                               ; preds = %75
  %83 = load ptr, ptr %4, align 8
  %84 = load ptr, ptr %6, align 8
  call void @reallymarkobject(ptr noundef %83, ptr noundef %84)
  br label %85

85:                                               ; preds = %82, %75
  br label %86

86:                                               ; preds = %85
  %87 = load ptr, ptr %6, align 8
  %88 = getelementptr inbounds %struct.UpVal, ptr %87, i32 0, i32 4
  %89 = getelementptr inbounds %struct.anon.6, ptr %88, i32 0, i32 0
  %90 = load ptr, ptr %89, align 8
  store ptr %90, ptr %6, align 8
  br label %72, !llvm.loop !35

91:                                               ; preds = %72
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.global_State, ptr %92, i32 0, i32 11
  %94 = load i8, ptr %93, align 1
  %95 = zext i8 %94 to i32
  %96 = icmp eq i32 %95, 2
  br i1 %96, label %97, label %142

97:                                               ; preds = %91
  %98 = load ptr, ptr %4, align 8
  %99 = getelementptr inbounds %struct.global_State, ptr %98, i32 0, i32 17
  %100 = load i8, ptr %99, align 1
  %101 = icmp ne i8 %100, 0
  br i1 %101, label %104, label %102

102:                                              ; preds = %97
  %103 = load ptr, ptr %5, align 8
  call void @luaD_shrinkstack(ptr noundef %103)
  br label %104

104:                                              ; preds = %102, %97
  %105 = load ptr, ptr %5, align 8
  %106 = getelementptr inbounds %struct.lua_State, ptr %105, i32 0, i32 6
  %107 = load ptr, ptr %106, align 8
  store ptr %107, ptr %7, align 8
  br label %108

108:                                              ; preds = %118, %104
  %109 = load ptr, ptr %7, align 8
  %110 = load ptr, ptr %5, align 8
  %111 = getelementptr inbounds %struct.lua_State, ptr %110, i32 0, i32 9
  %112 = load ptr, ptr %111, align 8
  %113 = getelementptr inbounds %union.StackValue, ptr %112, i64 5
  %114 = icmp ult ptr %109, %113
  br i1 %114, label %115, label %121

115:                                              ; preds = %108
  %116 = load ptr, ptr %7, align 8
  %117 = getelementptr inbounds %struct.TValue, ptr %116, i32 0, i32 1
  store i8 0, ptr %117, align 8
  br label %118

118:                                              ; preds = %115
  %119 = load ptr, ptr %7, align 8
  %120 = getelementptr inbounds %union.StackValue, ptr %119, i32 1
  store ptr %120, ptr %7, align 8
  br label %108, !llvm.loop !36

121:                                              ; preds = %108
  %122 = load ptr, ptr %5, align 8
  %123 = getelementptr inbounds %struct.lua_State, ptr %122, i32 0, i32 14
  %124 = load ptr, ptr %123, align 8
  %125 = load ptr, ptr %5, align 8
  %126 = icmp ne ptr %124, %125
  br i1 %126, label %141, label %127

127:                                              ; preds = %121
  %128 = load ptr, ptr %5, align 8
  %129 = getelementptr inbounds %struct.lua_State, ptr %128, i32 0, i32 11
  %130 = load ptr, ptr %129, align 8
  %131 = icmp ne ptr %130, null
  br i1 %131, label %132, label %141

132:                                              ; preds = %127
  %133 = load ptr, ptr %4, align 8
  %134 = getelementptr inbounds %struct.global_State, ptr %133, i32 0, i32 38
  %135 = load ptr, ptr %134, align 8
  %136 = load ptr, ptr %5, align 8
  %137 = getelementptr inbounds %struct.lua_State, ptr %136, i32 0, i32 14
  store ptr %135, ptr %137, align 8
  %138 = load ptr, ptr %5, align 8
  %139 = load ptr, ptr %4, align 8
  %140 = getelementptr inbounds %struct.global_State, ptr %139, i32 0, i32 38
  store ptr %138, ptr %140, align 8
  br label %141

141:                                              ; preds = %132, %127, %121
  br label %142

142:                                              ; preds = %141, %91
  %143 = load ptr, ptr %5, align 8
  %144 = getelementptr inbounds %struct.lua_State, ptr %143, i32 0, i32 9
  %145 = load ptr, ptr %144, align 8
  %146 = load ptr, ptr %5, align 8
  %147 = getelementptr inbounds %struct.lua_State, ptr %146, i32 0, i32 10
  %148 = load ptr, ptr %147, align 8
  %149 = ptrtoint ptr %145 to i64
  %150 = ptrtoint ptr %148 to i64
  %151 = sub i64 %149, %150
  %152 = sdiv exact i64 %151, 16
  %153 = trunc i64 %152 to i32
  %154 = add nsw i32 1, %153
  store i32 %154, ptr %3, align 4
  br label %155

155:                                              ; preds = %142, %32
  %156 = load i32, ptr %3, align 4
  ret i32 %156
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @traverseweakvalue(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.Table, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.Table, ptr %11, i32 0, i32 4
  %13 = load i8, ptr %12, align 1
  %14 = zext i8 %13 to i32
  %15 = shl i32 1, %14
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds %union.Node, ptr %10, i64 %16
  store ptr %17, ptr %6, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.Table, ptr %18, i32 0, i32 5
  %20 = load i32, ptr %19, align 4
  %21 = icmp ugt i32 %20, 0
  %22 = zext i1 %21 to i32
  store i32 %22, ptr %7, align 4
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.Table, ptr %23, i32 0, i32 7
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %union.Node, ptr %25, i64 0
  store ptr %26, ptr %5, align 8
  br label %27

27:                                               ; preds = %84, %2
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = icmp ult ptr %28, %29
  br i1 %30, label %31, label %87

31:                                               ; preds = %27
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = and i32 %35, 15
  %37 = icmp eq i32 %36, 0
  br i1 %37, label %38, label %40

38:                                               ; preds = %31
  %39 = load ptr, ptr %5, align 8
  call void @clearkey(ptr noundef %39)
  br label %83

40:                                               ; preds = %31
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.NodeKey, ptr %41, i32 0, i32 2
  %43 = load i8, ptr %42, align 1
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 64
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %61

47:                                               ; preds = %40
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.NodeKey, ptr %48, i32 0, i32 4
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.GCObject, ptr %50, i32 0, i32 2
  %52 = load i8, ptr %51, align 1
  %53 = zext i8 %52 to i32
  %54 = and i32 %53, 24
  %55 = icmp ne i32 %54, 0
  br i1 %55, label %56, label %61

56:                                               ; preds = %47
  %57 = load ptr, ptr %3, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.NodeKey, ptr %58, i32 0, i32 4
  %60 = load ptr, ptr %59, align 8
  call void @reallymarkobject(ptr noundef %57, ptr noundef %60)
  br label %61

61:                                               ; preds = %56, %47, %40
  %62 = load i32, ptr %7, align 4
  %63 = icmp ne i32 %62, 0
  br i1 %63, label %82, label %64

64:                                               ; preds = %61
  %65 = load ptr, ptr %3, align 8
  %66 = load ptr, ptr %5, align 8
  %67 = getelementptr inbounds %struct.TValue, ptr %66, i32 0, i32 1
  %68 = load i8, ptr %67, align 8
  %69 = zext i8 %68 to i32
  %70 = and i32 %69, 64
  %71 = icmp ne i32 %70, 0
  br i1 %71, label %72, label %76

72:                                               ; preds = %64
  %73 = load ptr, ptr %5, align 8
  %74 = getelementptr inbounds %struct.TValue, ptr %73, i32 0, i32 0
  %75 = load ptr, ptr %74, align 8
  br label %77

76:                                               ; preds = %64
  br label %77

77:                                               ; preds = %76, %72
  %78 = phi ptr [ %75, %72 ], [ null, %76 ]
  %79 = call i32 @iscleared(ptr noundef %65, ptr noundef %78)
  %80 = icmp ne i32 %79, 0
  br i1 %80, label %81, label %82

81:                                               ; preds = %77
  store i32 1, ptr %7, align 4
  br label %82

82:                                               ; preds = %81, %77, %61
  br label %83

83:                                               ; preds = %82, %38
  br label %84

84:                                               ; preds = %83
  %85 = load ptr, ptr %5, align 8
  %86 = getelementptr inbounds %union.Node, ptr %85, i32 1
  store ptr %86, ptr %5, align 8
  br label %27, !llvm.loop !37

87:                                               ; preds = %27
  %88 = load ptr, ptr %3, align 8
  %89 = getelementptr inbounds %struct.global_State, ptr %88, i32 0, i32 11
  %90 = load i8, ptr %89, align 1
  %91 = zext i8 %90 to i32
  %92 = icmp eq i32 %91, 2
  br i1 %92, label %93, label %102

93:                                               ; preds = %87
  %94 = load i32, ptr %7, align 4
  %95 = icmp ne i32 %94, 0
  br i1 %95, label %96, label %102

96:                                               ; preds = %93
  %97 = load ptr, ptr %4, align 8
  %98 = load ptr, ptr %4, align 8
  %99 = getelementptr inbounds %struct.Table, ptr %98, i32 0, i32 10
  %100 = load ptr, ptr %3, align 8
  %101 = getelementptr inbounds %struct.global_State, ptr %100, i32 0, i32 26
  call void @linkgclist_(ptr noundef %97, ptr noundef %99, ptr noundef %101)
  br label %108

102:                                              ; preds = %93, %87
  %103 = load ptr, ptr %4, align 8
  %104 = load ptr, ptr %4, align 8
  %105 = getelementptr inbounds %struct.Table, ptr %104, i32 0, i32 10
  %106 = load ptr, ptr %3, align 8
  %107 = getelementptr inbounds %struct.global_State, ptr %106, i32 0, i32 25
  call void @linkgclist_(ptr noundef %103, ptr noundef %105, ptr noundef %107)
  br label %108

108:                                              ; preds = %102, %96
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @traverseephemeron(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  store i32 0, ptr %7, align 4
  store i32 0, ptr %8, align 4
  store i32 0, ptr %9, align 4
  %14 = load ptr, ptr %5, align 8
  %15 = call i32 @luaH_realasize(ptr noundef %14)
  store i32 %15, ptr %11, align 4
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.Table, ptr %16, i32 0, i32 4
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = shl i32 1, %19
  store i32 %20, ptr %12, align 4
  store i32 0, ptr %10, align 4
  br label %21

21:                                               ; preds = %62, %3
  %22 = load i32, ptr %10, align 4
  %23 = load i32, ptr %11, align 4
  %24 = icmp ult i32 %22, %23
  br i1 %24, label %25, label %65

25:                                               ; preds = %21
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.Table, ptr %26, i32 0, i32 6
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %10, align 4
  %30 = zext i32 %29 to i64
  %31 = getelementptr inbounds %struct.TValue, ptr %28, i64 %30
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 1
  %33 = load i8, ptr %32, align 8
  %34 = zext i8 %33 to i32
  %35 = and i32 %34, 64
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %37, label %61

37:                                               ; preds = %25
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.Table, ptr %38, i32 0, i32 6
  %40 = load ptr, ptr %39, align 8
  %41 = load i32, ptr %10, align 4
  %42 = zext i32 %41 to i64
  %43 = getelementptr inbounds %struct.TValue, ptr %40, i64 %42
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %struct.GCObject, ptr %45, i32 0, i32 2
  %47 = load i8, ptr %46, align 1
  %48 = zext i8 %47 to i32
  %49 = and i32 %48, 24
  %50 = icmp ne i32 %49, 0
  br i1 %50, label %51, label %61

51:                                               ; preds = %37
  store i32 1, ptr %7, align 4
  %52 = load ptr, ptr %4, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.Table, ptr %53, i32 0, i32 6
  %55 = load ptr, ptr %54, align 8
  %56 = load i32, ptr %10, align 4
  %57 = zext i32 %56 to i64
  %58 = getelementptr inbounds %struct.TValue, ptr %55, i64 %57
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 0
  %60 = load ptr, ptr %59, align 8
  call void @reallymarkobject(ptr noundef %52, ptr noundef %60)
  br label %61

61:                                               ; preds = %51, %37, %25
  br label %62

62:                                               ; preds = %61
  %63 = load i32, ptr %10, align 4
  %64 = add i32 %63, 1
  store i32 %64, ptr %10, align 4
  br label %21, !llvm.loop !38

65:                                               ; preds = %21
  store i32 0, ptr %10, align 4
  br label %66

66:                                               ; preds = %159, %65
  %67 = load i32, ptr %10, align 4
  %68 = load i32, ptr %12, align 4
  %69 = icmp ult i32 %67, %68
  br i1 %69, label %70, label %162

70:                                               ; preds = %66
  %71 = load i32, ptr %6, align 4
  %72 = icmp ne i32 %71, 0
  br i1 %72, label %73, label %83

73:                                               ; preds = %70
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.Table, ptr %74, i32 0, i32 7
  %76 = load ptr, ptr %75, align 8
  %77 = load i32, ptr %12, align 4
  %78 = sub i32 %77, 1
  %79 = load i32, ptr %10, align 4
  %80 = sub i32 %78, %79
  %81 = zext i32 %80 to i64
  %82 = getelementptr inbounds %union.Node, ptr %76, i64 %81
  br label %90

83:                                               ; preds = %70
  %84 = load ptr, ptr %5, align 8
  %85 = getelementptr inbounds %struct.Table, ptr %84, i32 0, i32 7
  %86 = load ptr, ptr %85, align 8
  %87 = load i32, ptr %10, align 4
  %88 = zext i32 %87 to i64
  %89 = getelementptr inbounds %union.Node, ptr %86, i64 %88
  br label %90

90:                                               ; preds = %83, %73
  %91 = phi ptr [ %82, %73 ], [ %89, %83 ]
  store ptr %91, ptr %13, align 8
  %92 = load ptr, ptr %13, align 8
  %93 = getelementptr inbounds %struct.TValue, ptr %92, i32 0, i32 1
  %94 = load i8, ptr %93, align 8
  %95 = zext i8 %94 to i32
  %96 = and i32 %95, 15
  %97 = icmp eq i32 %96, 0
  br i1 %97, label %98, label %100

98:                                               ; preds = %90
  %99 = load ptr, ptr %13, align 8
  call void @clearkey(ptr noundef %99)
  br label %158

100:                                              ; preds = %90
  %101 = load ptr, ptr %4, align 8
  %102 = load ptr, ptr %13, align 8
  %103 = getelementptr inbounds %struct.NodeKey, ptr %102, i32 0, i32 2
  %104 = load i8, ptr %103, align 1
  %105 = zext i8 %104 to i32
  %106 = and i32 %105, 64
  %107 = icmp ne i32 %106, 0
  br i1 %107, label %108, label %112

108:                                              ; preds = %100
  %109 = load ptr, ptr %13, align 8
  %110 = getelementptr inbounds %struct.NodeKey, ptr %109, i32 0, i32 4
  %111 = load ptr, ptr %110, align 8
  br label %113

112:                                              ; preds = %100
  br label %113

113:                                              ; preds = %112, %108
  %114 = phi ptr [ %111, %108 ], [ null, %112 ]
  %115 = call i32 @iscleared(ptr noundef %101, ptr noundef %114)
  %116 = icmp ne i32 %115, 0
  br i1 %116, label %117, label %135

117:                                              ; preds = %113
  store i32 1, ptr %8, align 4
  %118 = load ptr, ptr %13, align 8
  %119 = getelementptr inbounds %struct.TValue, ptr %118, i32 0, i32 1
  %120 = load i8, ptr %119, align 8
  %121 = zext i8 %120 to i32
  %122 = and i32 %121, 64
  %123 = icmp ne i32 %122, 0
  br i1 %123, label %124, label %134

124:                                              ; preds = %117
  %125 = load ptr, ptr %13, align 8
  %126 = getelementptr inbounds %struct.TValue, ptr %125, i32 0, i32 0
  %127 = load ptr, ptr %126, align 8
  %128 = getelementptr inbounds %struct.GCObject, ptr %127, i32 0, i32 2
  %129 = load i8, ptr %128, align 1
  %130 = zext i8 %129 to i32
  %131 = and i32 %130, 24
  %132 = icmp ne i32 %131, 0
  br i1 %132, label %133, label %134

133:                                              ; preds = %124
  store i32 1, ptr %9, align 4
  br label %134

134:                                              ; preds = %133, %124, %117
  br label %157

135:                                              ; preds = %113
  %136 = load ptr, ptr %13, align 8
  %137 = getelementptr inbounds %struct.TValue, ptr %136, i32 0, i32 1
  %138 = load i8, ptr %137, align 8
  %139 = zext i8 %138 to i32
  %140 = and i32 %139, 64
  %141 = icmp ne i32 %140, 0
  br i1 %141, label %142, label %156

142:                                              ; preds = %135
  %143 = load ptr, ptr %13, align 8
  %144 = getelementptr inbounds %struct.TValue, ptr %143, i32 0, i32 0
  %145 = load ptr, ptr %144, align 8
  %146 = getelementptr inbounds %struct.GCObject, ptr %145, i32 0, i32 2
  %147 = load i8, ptr %146, align 1
  %148 = zext i8 %147 to i32
  %149 = and i32 %148, 24
  %150 = icmp ne i32 %149, 0
  br i1 %150, label %151, label %156

151:                                              ; preds = %142
  store i32 1, ptr %7, align 4
  %152 = load ptr, ptr %4, align 8
  %153 = load ptr, ptr %13, align 8
  %154 = getelementptr inbounds %struct.TValue, ptr %153, i32 0, i32 0
  %155 = load ptr, ptr %154, align 8
  call void @reallymarkobject(ptr noundef %152, ptr noundef %155)
  br label %156

156:                                              ; preds = %151, %142, %135
  br label %157

157:                                              ; preds = %156, %134
  br label %158

158:                                              ; preds = %157, %98
  br label %159

159:                                              ; preds = %158
  %160 = load i32, ptr %10, align 4
  %161 = add i32 %160, 1
  store i32 %161, ptr %10, align 4
  br label %66, !llvm.loop !39

162:                                              ; preds = %66
  %163 = load ptr, ptr %4, align 8
  %164 = getelementptr inbounds %struct.global_State, ptr %163, i32 0, i32 11
  %165 = load i8, ptr %164, align 1
  %166 = zext i8 %165 to i32
  %167 = icmp eq i32 %166, 0
  br i1 %167, label %168, label %174

168:                                              ; preds = %162
  %169 = load ptr, ptr %5, align 8
  %170 = load ptr, ptr %5, align 8
  %171 = getelementptr inbounds %struct.Table, ptr %170, i32 0, i32 10
  %172 = load ptr, ptr %4, align 8
  %173 = getelementptr inbounds %struct.global_State, ptr %172, i32 0, i32 25
  call void @linkgclist_(ptr noundef %169, ptr noundef %171, ptr noundef %173)
  br label %197

174:                                              ; preds = %162
  %175 = load i32, ptr %9, align 4
  %176 = icmp ne i32 %175, 0
  br i1 %176, label %177, label %183

177:                                              ; preds = %174
  %178 = load ptr, ptr %5, align 8
  %179 = load ptr, ptr %5, align 8
  %180 = getelementptr inbounds %struct.Table, ptr %179, i32 0, i32 10
  %181 = load ptr, ptr %4, align 8
  %182 = getelementptr inbounds %struct.global_State, ptr %181, i32 0, i32 27
  call void @linkgclist_(ptr noundef %178, ptr noundef %180, ptr noundef %182)
  br label %196

183:                                              ; preds = %174
  %184 = load i32, ptr %8, align 4
  %185 = icmp ne i32 %184, 0
  br i1 %185, label %186, label %192

186:                                              ; preds = %183
  %187 = load ptr, ptr %5, align 8
  %188 = load ptr, ptr %5, align 8
  %189 = getelementptr inbounds %struct.Table, ptr %188, i32 0, i32 10
  %190 = load ptr, ptr %4, align 8
  %191 = getelementptr inbounds %struct.global_State, ptr %190, i32 0, i32 28
  call void @linkgclist_(ptr noundef %187, ptr noundef %189, ptr noundef %191)
  br label %195

192:                                              ; preds = %183
  %193 = load ptr, ptr %4, align 8
  %194 = load ptr, ptr %5, align 8
  call void @genlink(ptr noundef %193, ptr noundef %194)
  br label %195

195:                                              ; preds = %192, %186
  br label %196

196:                                              ; preds = %195, %177
  br label %197

197:                                              ; preds = %196, %168
  %198 = load i32, ptr %7, align 4
  ret i32 %198
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @traversestrongtable(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.Table, ptr %9, i32 0, i32 7
  %11 = load ptr, ptr %10, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.Table, ptr %12, i32 0, i32 4
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = shl i32 1, %15
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds %union.Node, ptr %11, i64 %17
  store ptr %18, ptr %6, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @luaH_realasize(ptr noundef %19)
  store i32 %20, ptr %8, align 4
  store i32 0, ptr %7, align 4
  br label %21

21:                                               ; preds = %65, %2
  %22 = load i32, ptr %7, align 4
  %23 = load i32, ptr %8, align 4
  %24 = icmp ult i32 %22, %23
  br i1 %24, label %25, label %68

25:                                               ; preds = %21
  %26 = load ptr, ptr %3, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 40
  %28 = load ptr, ptr %27, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.Table, ptr %29, i32 0, i32 6
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %7, align 4
  %33 = zext i32 %32 to i64
  %34 = getelementptr inbounds %struct.TValue, ptr %31, i64 %33
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  %36 = load i8, ptr %35, align 8
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 64
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %64

40:                                               ; preds = %25
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.Table, ptr %41, i32 0, i32 6
  %43 = load ptr, ptr %42, align 8
  %44 = load i32, ptr %7, align 4
  %45 = zext i32 %44 to i64
  %46 = getelementptr inbounds %struct.TValue, ptr %43, i64 %45
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load ptr, ptr %47, align 8
  %49 = getelementptr inbounds %struct.GCObject, ptr %48, i32 0, i32 2
  %50 = load i8, ptr %49, align 1
  %51 = zext i8 %50 to i32
  %52 = and i32 %51, 24
  %53 = icmp ne i32 %52, 0
  br i1 %53, label %54, label %64

54:                                               ; preds = %40
  %55 = load ptr, ptr %3, align 8
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.Table, ptr %56, i32 0, i32 6
  %58 = load ptr, ptr %57, align 8
  %59 = load i32, ptr %7, align 4
  %60 = zext i32 %59 to i64
  %61 = getelementptr inbounds %struct.TValue, ptr %58, i64 %60
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  call void @reallymarkobject(ptr noundef %55, ptr noundef %63)
  br label %64

64:                                               ; preds = %54, %40, %25
  br label %65

65:                                               ; preds = %64
  %66 = load i32, ptr %7, align 4
  %67 = add i32 %66, 1
  store i32 %67, ptr %7, align 4
  br label %21, !llvm.loop !40

68:                                               ; preds = %21
  %69 = load ptr, ptr %4, align 8
  %70 = getelementptr inbounds %struct.Table, ptr %69, i32 0, i32 7
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %union.Node, ptr %71, i64 0
  store ptr %72, ptr %5, align 8
  br label %73

73:                                               ; preds = %133, %68
  %74 = load ptr, ptr %5, align 8
  %75 = load ptr, ptr %6, align 8
  %76 = icmp ult ptr %74, %75
  br i1 %76, label %77, label %136

77:                                               ; preds = %73
  %78 = load ptr, ptr %5, align 8
  %79 = getelementptr inbounds %struct.TValue, ptr %78, i32 0, i32 1
  %80 = load i8, ptr %79, align 8
  %81 = zext i8 %80 to i32
  %82 = and i32 %81, 15
  %83 = icmp eq i32 %82, 0
  br i1 %83, label %84, label %86

84:                                               ; preds = %77
  %85 = load ptr, ptr %5, align 8
  call void @clearkey(ptr noundef %85)
  br label %132

86:                                               ; preds = %77
  %87 = load ptr, ptr %5, align 8
  %88 = getelementptr inbounds %struct.NodeKey, ptr %87, i32 0, i32 2
  %89 = load i8, ptr %88, align 1
  %90 = zext i8 %89 to i32
  %91 = and i32 %90, 64
  %92 = icmp ne i32 %91, 0
  br i1 %92, label %93, label %107

93:                                               ; preds = %86
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %struct.NodeKey, ptr %94, i32 0, i32 4
  %96 = load ptr, ptr %95, align 8
  %97 = getelementptr inbounds %struct.GCObject, ptr %96, i32 0, i32 2
  %98 = load i8, ptr %97, align 1
  %99 = zext i8 %98 to i32
  %100 = and i32 %99, 24
  %101 = icmp ne i32 %100, 0
  br i1 %101, label %102, label %107

102:                                              ; preds = %93
  %103 = load ptr, ptr %3, align 8
  %104 = load ptr, ptr %5, align 8
  %105 = getelementptr inbounds %struct.NodeKey, ptr %104, i32 0, i32 4
  %106 = load ptr, ptr %105, align 8
  call void @reallymarkobject(ptr noundef %103, ptr noundef %106)
  br label %107

107:                                              ; preds = %102, %93, %86
  %108 = load ptr, ptr %3, align 8
  %109 = getelementptr inbounds %struct.global_State, ptr %108, i32 0, i32 40
  %110 = load ptr, ptr %109, align 8
  %111 = load ptr, ptr %5, align 8
  %112 = getelementptr inbounds %struct.TValue, ptr %111, i32 0, i32 1
  %113 = load i8, ptr %112, align 8
  %114 = zext i8 %113 to i32
  %115 = and i32 %114, 64
  %116 = icmp ne i32 %115, 0
  br i1 %116, label %117, label %131

117:                                              ; preds = %107
  %118 = load ptr, ptr %5, align 8
  %119 = getelementptr inbounds %struct.TValue, ptr %118, i32 0, i32 0
  %120 = load ptr, ptr %119, align 8
  %121 = getelementptr inbounds %struct.GCObject, ptr %120, i32 0, i32 2
  %122 = load i8, ptr %121, align 1
  %123 = zext i8 %122 to i32
  %124 = and i32 %123, 24
  %125 = icmp ne i32 %124, 0
  br i1 %125, label %126, label %131

126:                                              ; preds = %117
  %127 = load ptr, ptr %3, align 8
  %128 = load ptr, ptr %5, align 8
  %129 = getelementptr inbounds %struct.TValue, ptr %128, i32 0, i32 0
  %130 = load ptr, ptr %129, align 8
  call void @reallymarkobject(ptr noundef %127, ptr noundef %130)
  br label %131

131:                                              ; preds = %126, %117, %107
  br label %132

132:                                              ; preds = %131, %84
  br label %133

133:                                              ; preds = %132
  %134 = load ptr, ptr %5, align 8
  %135 = getelementptr inbounds %union.Node, ptr %134, i32 1
  store ptr %135, ptr %5, align 8
  br label %73, !llvm.loop !41

136:                                              ; preds = %73
  %137 = load ptr, ptr %3, align 8
  %138 = load ptr, ptr %4, align 8
  call void @genlink(ptr noundef %137, ptr noundef %138)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @clearkey(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.NodeKey, ptr %3, i32 0, i32 2
  %5 = load i8, ptr %4, align 1
  %6 = zext i8 %5 to i32
  %7 = and i32 %6, 64
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %12

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.NodeKey, ptr %10, i32 0, i32 2
  store i8 11, ptr %11, align 1
  br label %12

12:                                               ; preds = %9, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @iscleared(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %5, align 8
  %7 = icmp eq ptr %6, null
  br i1 %7, label %8, label %9

8:                                                ; preds = %2
  store i32 0, ptr %3, align 4
  br label %33

9:                                                ; preds = %2
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.GCObject, ptr %10, i32 0, i32 1
  %12 = load i8, ptr %11, align 8
  %13 = zext i8 %12 to i32
  %14 = and i32 %13, 15
  %15 = icmp eq i32 %14, 4
  br i1 %15, label %16, label %27

16:                                               ; preds = %9
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.GCObject, ptr %17, i32 0, i32 2
  %19 = load i8, ptr %18, align 1
  %20 = zext i8 %19 to i32
  %21 = and i32 %20, 24
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %26

23:                                               ; preds = %16
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %5, align 8
  call void @reallymarkobject(ptr noundef %24, ptr noundef %25)
  br label %26

26:                                               ; preds = %23, %16
  store i32 0, ptr %3, align 4
  br label %33

27:                                               ; preds = %9
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.GCObject, ptr %28, i32 0, i32 2
  %30 = load i8, ptr %29, align 1
  %31 = zext i8 %30 to i32
  %32 = and i32 %31, 24
  store i32 %32, ptr %3, align 4
  br label %33

33:                                               ; preds = %27, %26, %8
  %34 = load i32, ptr %3, align 4
  ret i32 %34
}

declare hidden i32 @luaH_realasize(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @genlink(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = getelementptr inbounds %struct.GCObject, ptr %5, i32 0, i32 2
  %7 = load i8, ptr %6, align 1
  %8 = zext i8 %7 to i32
  %9 = and i32 %8, 7
  %10 = icmp eq i32 %9, 5
  br i1 %10, label %11, label %17

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = call ptr @getgclist(ptr noundef %13)
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 25
  call void @linkgclist_(ptr noundef %12, ptr noundef %14, ptr noundef %16)
  br label %32

17:                                               ; preds = %2
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.GCObject, ptr %18, i32 0, i32 2
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i32
  %22 = and i32 %21, 7
  %23 = icmp eq i32 %22, 6
  br i1 %23, label %24, label %31

24:                                               ; preds = %17
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.GCObject, ptr %25, i32 0, i32 2
  %27 = load i8, ptr %26, align 1
  %28 = zext i8 %27 to i32
  %29 = xor i32 %28, 2
  %30 = trunc i32 %29 to i8
  store i8 %30, ptr %26, align 1
  br label %31

31:                                               ; preds = %24, %17
  br label %32

32:                                               ; preds = %31, %11
  ret void
}

declare hidden void @luaD_shrinkstack(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @cleargraylists(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.global_State, ptr %3, i32 0, i32 25
  store ptr null, ptr %4, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.global_State, ptr %5, i32 0, i32 24
  store ptr null, ptr %6, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 27
  store ptr null, ptr %8, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.global_State, ptr %9, i32 0, i32 28
  store ptr null, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 26
  store ptr null, ptr %12, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @sweep2old(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %6, align 8
  br label %11

11:                                               ; preds = %82, %2
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %5, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %83

15:                                               ; preds = %11
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.GCObject, ptr %16, i32 0, i32 2
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = and i32 %19, 24
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %22, label %29

22:                                               ; preds = %15
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.GCObject, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %4, align 8
  store ptr %25, ptr %26, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = load ptr, ptr %5, align 8
  call void @freeobj(ptr noundef %27, ptr noundef %28)
  br label %82

29:                                               ; preds = %15
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.GCObject, ptr %30, i32 0, i32 2
  %32 = load i8, ptr %31, align 1
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, -8
  %35 = or i32 %34, 4
  %36 = trunc i32 %35 to i8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.GCObject, ptr %37, i32 0, i32 2
  store i8 %36, ptr %38, align 1
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.GCObject, ptr %39, i32 0, i32 1
  %41 = load i8, ptr %40, align 8
  %42 = zext i8 %41 to i32
  %43 = icmp eq i32 %42, 8
  br i1 %43, label %44, label %51

44:                                               ; preds = %29
  %45 = load ptr, ptr %5, align 8
  store ptr %45, ptr %7, align 8
  %46 = load ptr, ptr %7, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 13
  %49 = load ptr, ptr %6, align 8
  %50 = getelementptr inbounds %struct.global_State, ptr %49, i32 0, i32 25
  call void @linkgclist_(ptr noundef %46, ptr noundef %48, ptr noundef %50)
  br label %79

51:                                               ; preds = %29
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.GCObject, ptr %52, i32 0, i32 1
  %54 = load i8, ptr %53, align 8
  %55 = zext i8 %54 to i32
  %56 = icmp eq i32 %55, 9
  br i1 %56, label %57, label %71

57:                                               ; preds = %51
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.UpVal, ptr %58, i32 0, i32 3
  %60 = load ptr, ptr %59, align 8
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %struct.UpVal, ptr %61, i32 0, i32 4
  %63 = icmp ne ptr %60, %62
  br i1 %63, label %64, label %71

64:                                               ; preds = %57
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.GCObject, ptr %65, i32 0, i32 2
  %67 = load i8, ptr %66, align 1
  %68 = zext i8 %67 to i32
  %69 = and i32 %68, 199
  %70 = trunc i32 %69 to i8
  store i8 %70, ptr %66, align 1
  br label %78

71:                                               ; preds = %57, %51
  %72 = load ptr, ptr %5, align 8
  %73 = getelementptr inbounds %struct.GCObject, ptr %72, i32 0, i32 2
  %74 = load i8, ptr %73, align 1
  %75 = zext i8 %74 to i32
  %76 = or i32 %75, 32
  %77 = trunc i32 %76 to i8
  store i8 %77, ptr %73, align 1
  br label %78

78:                                               ; preds = %71, %64
  br label %79

79:                                               ; preds = %78, %44
  %80 = load ptr, ptr %5, align 8
  %81 = getelementptr inbounds %struct.GCObject, ptr %80, i32 0, i32 0
  store ptr %81, ptr %4, align 8
  br label %82

82:                                               ; preds = %79, %22
  br label %11, !llvm.loop !42

83:                                               ; preds = %11
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @finishgencycle(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  call void @correctgraylists(ptr noundef %5)
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  call void @checkSizes(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.global_State, ptr %8, i32 0, i32 11
  store i8 0, ptr %9, align 1
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 17
  %12 = load i8, ptr %11, align 1
  %13 = icmp ne i8 %12, 0
  br i1 %13, label %16, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %3, align 8
  call void @callallpendingfinalizers(ptr noundef %15)
  br label %16

16:                                               ; preds = %14, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @correctgraylists(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.global_State, ptr %4, i32 0, i32 25
  %6 = call ptr @correctgraylist(ptr noundef %5)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 26
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %3, align 8
  store ptr %9, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 26
  store ptr null, ptr %12, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = call ptr @correctgraylist(ptr noundef %13)
  store ptr %14, ptr %3, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 28
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %3, align 8
  store ptr %17, ptr %18, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.global_State, ptr %19, i32 0, i32 28
  store ptr null, ptr %20, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = call ptr @correctgraylist(ptr noundef %21)
  store ptr %22, ptr %3, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 27
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %3, align 8
  store ptr %25, ptr %26, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 27
  store ptr null, ptr %28, align 8
  %29 = load ptr, ptr %3, align 8
  %30 = call ptr @correctgraylist(ptr noundef %29)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkSizes(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.global_State, ptr %6, i32 0, i32 17
  %8 = load i8, ptr %7, align 1
  %9 = icmp ne i8 %8, 0
  br i1 %9, label %41, label %10

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 6
  %13 = getelementptr inbounds %struct.stringtable, ptr %12, i32 0, i32 1
  %14 = load i32, ptr %13, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 6
  %17 = getelementptr inbounds %struct.stringtable, ptr %16, i32 0, i32 2
  %18 = load i32, ptr %17, align 4
  %19 = sdiv i32 %18, 4
  %20 = icmp slt i32 %14, %19
  br i1 %20, label %21, label %40

21:                                               ; preds = %10
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.global_State, ptr %22, i32 0, i32 3
  %24 = load i64, ptr %23, align 8
  store i64 %24, ptr %5, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 6
  %28 = getelementptr inbounds %struct.stringtable, ptr %27, i32 0, i32 2
  %29 = load i32, ptr %28, align 4
  %30 = sdiv i32 %29, 2
  call void @luaS_resize(ptr noundef %25, i32 noundef %30)
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.global_State, ptr %31, i32 0, i32 3
  %33 = load i64, ptr %32, align 8
  %34 = load i64, ptr %5, align 8
  %35 = sub nsw i64 %33, %34
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.global_State, ptr %36, i32 0, i32 4
  %38 = load i64, ptr %37, align 8
  %39 = add i64 %38, %35
  store i64 %39, ptr %37, align 8
  br label %40

40:                                               ; preds = %21, %10
  br label %41

41:                                               ; preds = %40, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @correctgraylist(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  br label %5

5:                                                ; preds = %71, %67, %1
  %6 = load ptr, ptr %2, align 8
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %3, align 8
  %8 = icmp ne ptr %7, null
  br i1 %8, label %9, label %73

9:                                                ; preds = %5
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @getgclist(ptr noundef %10)
  store ptr %11, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.GCObject, ptr %12, i32 0, i32 2
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 24
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %19

18:                                               ; preds = %9
  br label %67

19:                                               ; preds = %9
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.GCObject, ptr %20, i32 0, i32 2
  %22 = load i8, ptr %21, align 1
  %23 = zext i8 %22 to i32
  %24 = and i32 %23, 7
  %25 = icmp eq i32 %24, 5
  br i1 %25, label %26, label %39

26:                                               ; preds = %19
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.GCObject, ptr %27, i32 0, i32 2
  %29 = load i8, ptr %28, align 1
  %30 = zext i8 %29 to i32
  %31 = or i32 %30, 32
  %32 = trunc i32 %31 to i8
  store i8 %32, ptr %28, align 1
  %33 = load ptr, ptr %3, align 8
  %34 = getelementptr inbounds %struct.GCObject, ptr %33, i32 0, i32 2
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  %37 = xor i32 %36, 3
  %38 = trunc i32 %37 to i8
  store i8 %38, ptr %34, align 1
  br label %71

39:                                               ; preds = %19
  %40 = load ptr, ptr %3, align 8
  %41 = getelementptr inbounds %struct.GCObject, ptr %40, i32 0, i32 1
  %42 = load i8, ptr %41, align 8
  %43 = zext i8 %42 to i32
  %44 = icmp eq i32 %43, 8
  br i1 %44, label %45, label %46

45:                                               ; preds = %39
  br label %71

46:                                               ; preds = %39
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.GCObject, ptr %47, i32 0, i32 2
  %49 = load i8, ptr %48, align 1
  %50 = zext i8 %49 to i32
  %51 = and i32 %50, 7
  %52 = icmp eq i32 %51, 6
  br i1 %52, label %53, label %60

53:                                               ; preds = %46
  %54 = load ptr, ptr %3, align 8
  %55 = getelementptr inbounds %struct.GCObject, ptr %54, i32 0, i32 2
  %56 = load i8, ptr %55, align 1
  %57 = zext i8 %56 to i32
  %58 = xor i32 %57, 2
  %59 = trunc i32 %58 to i8
  store i8 %59, ptr %55, align 1
  br label %60

60:                                               ; preds = %53, %46
  %61 = load ptr, ptr %3, align 8
  %62 = getelementptr inbounds %struct.GCObject, ptr %61, i32 0, i32 2
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = or i32 %64, 32
  %66 = trunc i32 %65 to i8
  store i8 %66, ptr %62, align 1
  br label %67

67:                                               ; preds = %60, %18
  %68 = load ptr, ptr %4, align 8
  %69 = load ptr, ptr %68, align 8
  %70 = load ptr, ptr %2, align 8
  store ptr %69, ptr %70, align 8
  br label %5, !llvm.loop !43

71:                                               ; preds = %45, %26
  %72 = load ptr, ptr %4, align 8
  store ptr %72, ptr %2, align 8
  br label %5, !llvm.loop !43

73:                                               ; preds = %5
  %74 = load ptr, ptr %2, align 8
  ret ptr %74
}

declare hidden void @luaS_resize(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @whitelist(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.global_State, ptr %6, i32 0, i32 10
  %8 = load i8, ptr %7, align 4
  %9 = zext i8 %8 to i32
  %10 = and i32 %9, 24
  %11 = trunc i32 %10 to i8
  %12 = zext i8 %11 to i32
  store i32 %12, ptr %5, align 4
  br label %13

13:                                               ; preds = %27, %2
  %14 = load ptr, ptr %4, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %31

16:                                               ; preds = %13
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.GCObject, ptr %17, i32 0, i32 2
  %19 = load i8, ptr %18, align 1
  %20 = zext i8 %19 to i32
  %21 = and i32 %20, -64
  %22 = load i32, ptr %5, align 4
  %23 = or i32 %21, %22
  %24 = trunc i32 %23 to i8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.GCObject, ptr %25, i32 0, i32 2
  store i8 %24, ptr %26, align 1
  br label %27

27:                                               ; preds = %16
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.GCObject, ptr %28, i32 0, i32 0
  %30 = load ptr, ptr %29, align 8
  store ptr %30, ptr %4, align 8
  br label %13, !llvm.loop !44

31:                                               ; preds = %13
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @findlast(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  br label %3

3:                                                ; preds = %7, %1
  %4 = load ptr, ptr %2, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = icmp ne ptr %5, null
  br i1 %6, label %7, label %11

7:                                                ; preds = %3
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.GCObject, ptr %9, i32 0, i32 0
  store ptr %10, ptr %2, align 8
  br label %3, !llvm.loop !45

11:                                               ; preds = %3
  %12 = load ptr, ptr %2, align 8
  ret ptr %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @GCTM(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.TValue, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i8, align 1
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 7
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %3, align 8
  store ptr %5, ptr %6, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = call ptr @udata2finalize(ptr noundef %18)
  store ptr %19, ptr %7, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 0
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.GCObject, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  %27 = or i32 %26, 64
  %28 = trunc i32 %27 to i8
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  store i8 %28, ptr %30, align 8
  %31 = load ptr, ptr %2, align 8
  %32 = call ptr @luaT_gettmbyobj(ptr noundef %31, ptr noundef %5, i32 noundef 2)
  store ptr %32, ptr %4, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 1
  %35 = load i8, ptr %34, align 8
  %36 = zext i8 %35 to i32
  %37 = and i32 %36, 15
  %38 = icmp eq i32 %37, 0
  br i1 %38, label %133, label %39

39:                                               ; preds = %1
  %40 = load ptr, ptr %2, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 4
  %42 = load i8, ptr %41, align 1
  store i8 %42, ptr %9, align 1
  %43 = load ptr, ptr %3, align 8
  %44 = getelementptr inbounds %struct.global_State, ptr %43, i32 0, i32 16
  %45 = load i8, ptr %44, align 2
  %46 = zext i8 %45 to i32
  store i32 %46, ptr %10, align 4
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 16
  %49 = load i8, ptr %48, align 2
  %50 = zext i8 %49 to i32
  %51 = or i32 %50, 2
  %52 = trunc i32 %51 to i8
  store i8 %52, ptr %48, align 2
  %53 = load ptr, ptr %2, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 4
  store i8 0, ptr %54, align 1
  %55 = load ptr, ptr %2, align 8
  %56 = getelementptr inbounds %struct.lua_State, ptr %55, i32 0, i32 6
  %57 = load ptr, ptr %56, align 8
  %58 = getelementptr inbounds %union.StackValue, ptr %57, i32 1
  store ptr %58, ptr %56, align 8
  store ptr %57, ptr %11, align 8
  %59 = load ptr, ptr %4, align 8
  store ptr %59, ptr %12, align 8
  %60 = load ptr, ptr %11, align 8
  %61 = getelementptr inbounds %struct.TValue, ptr %60, i32 0, i32 0
  %62 = load ptr, ptr %12, align 8
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %61, ptr align 8 %63, i64 8, i1 false)
  %64 = load ptr, ptr %12, align 8
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 1
  %66 = load i8, ptr %65, align 8
  %67 = load ptr, ptr %11, align 8
  %68 = getelementptr inbounds %struct.TValue, ptr %67, i32 0, i32 1
  store i8 %66, ptr %68, align 8
  %69 = load ptr, ptr %2, align 8
  %70 = load ptr, ptr %2, align 8
  %71 = getelementptr inbounds %struct.lua_State, ptr %70, i32 0, i32 6
  %72 = load ptr, ptr %71, align 8
  %73 = getelementptr inbounds %union.StackValue, ptr %72, i32 1
  store ptr %73, ptr %71, align 8
  store ptr %72, ptr %13, align 8
  store ptr %5, ptr %14, align 8
  %74 = load ptr, ptr %13, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  %76 = load ptr, ptr %14, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %75, ptr align 8 %77, i64 8, i1 false)
  %78 = load ptr, ptr %14, align 8
  %79 = getelementptr inbounds %struct.TValue, ptr %78, i32 0, i32 1
  %80 = load i8, ptr %79, align 8
  %81 = load ptr, ptr %13, align 8
  %82 = getelementptr inbounds %struct.TValue, ptr %81, i32 0, i32 1
  store i8 %80, ptr %82, align 8
  %83 = load ptr, ptr %2, align 8
  %84 = load ptr, ptr %2, align 8
  %85 = getelementptr inbounds %struct.lua_State, ptr %84, i32 0, i32 8
  %86 = load ptr, ptr %85, align 8
  %87 = getelementptr inbounds %struct.CallInfo, ptr %86, i32 0, i32 7
  %88 = load i16, ptr %87, align 2
  %89 = zext i16 %88 to i32
  %90 = or i32 %89, 128
  %91 = trunc i32 %90 to i16
  store i16 %91, ptr %87, align 2
  %92 = load ptr, ptr %2, align 8
  %93 = load ptr, ptr %2, align 8
  %94 = getelementptr inbounds %struct.lua_State, ptr %93, i32 0, i32 6
  %95 = load ptr, ptr %94, align 8
  %96 = getelementptr inbounds %union.StackValue, ptr %95, i64 -2
  %97 = load ptr, ptr %2, align 8
  %98 = getelementptr inbounds %struct.lua_State, ptr %97, i32 0, i32 10
  %99 = load ptr, ptr %98, align 8
  %100 = ptrtoint ptr %96 to i64
  %101 = ptrtoint ptr %99 to i64
  %102 = sub i64 %100, %101
  %103 = call i32 @luaD_pcall(ptr noundef %92, ptr noundef @dothecall, ptr noundef null, i64 noundef %102, i64 noundef 0)
  store i32 %103, ptr %8, align 4
  %104 = load ptr, ptr %2, align 8
  %105 = getelementptr inbounds %struct.lua_State, ptr %104, i32 0, i32 8
  %106 = load ptr, ptr %105, align 8
  %107 = getelementptr inbounds %struct.CallInfo, ptr %106, i32 0, i32 7
  %108 = load i16, ptr %107, align 2
  %109 = zext i16 %108 to i32
  %110 = and i32 %109, -129
  %111 = trunc i32 %110 to i16
  store i16 %111, ptr %107, align 2
  %112 = load i8, ptr %9, align 1
  %113 = load ptr, ptr %2, align 8
  %114 = getelementptr inbounds %struct.lua_State, ptr %113, i32 0, i32 4
  store i8 %112, ptr %114, align 1
  %115 = load i32, ptr %10, align 4
  %116 = trunc i32 %115 to i8
  %117 = load ptr, ptr %3, align 8
  %118 = getelementptr inbounds %struct.global_State, ptr %117, i32 0, i32 16
  store i8 %116, ptr %118, align 2
  %119 = load i32, ptr %8, align 4
  %120 = icmp ne i32 %119, 0
  %121 = zext i1 %120 to i32
  %122 = icmp ne i32 %121, 0
  %123 = zext i1 %122 to i32
  %124 = sext i32 %123 to i64
  %125 = icmp ne i64 %124, 0
  br i1 %125, label %126, label %132

126:                                              ; preds = %39
  %127 = load ptr, ptr %2, align 8
  call void @luaE_warnerror(ptr noundef %127, ptr noundef @.str)
  %128 = load ptr, ptr %2, align 8
  %129 = getelementptr inbounds %struct.lua_State, ptr %128, i32 0, i32 6
  %130 = load ptr, ptr %129, align 8
  %131 = getelementptr inbounds %union.StackValue, ptr %130, i32 -1
  store ptr %131, ptr %129, align 8
  br label %132

132:                                              ; preds = %126, %39
  br label %133

133:                                              ; preds = %132, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @udata2finalize(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.global_State, ptr %4, i32 0, i32 29
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.GCObject, ptr %7, i32 0, i32 0
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 29
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 21
  %14 = load ptr, ptr %13, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.GCObject, ptr %15, i32 0, i32 0
  store ptr %14, ptr %16, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 21
  store ptr %17, ptr %19, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.GCObject, ptr %20, i32 0, i32 2
  %22 = load i8, ptr %21, align 1
  %23 = zext i8 %22 to i32
  %24 = and i32 %23, 191
  %25 = trunc i32 %24 to i8
  store i8 %25, ptr %21, align 1
  %26 = load ptr, ptr %2, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 11
  %28 = load i8, ptr %27, align 1
  %29 = zext i8 %28 to i32
  %30 = icmp sle i32 3, %29
  br i1 %30, label %31, label %54

31:                                               ; preds = %1
  %32 = load ptr, ptr %2, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 11
  %34 = load i8, ptr %33, align 1
  %35 = zext i8 %34 to i32
  %36 = icmp sle i32 %35, 6
  br i1 %36, label %37, label %54

37:                                               ; preds = %31
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %struct.GCObject, ptr %38, i32 0, i32 2
  %40 = load i8, ptr %39, align 1
  %41 = zext i8 %40 to i32
  %42 = and i32 %41, -57
  %43 = load ptr, ptr %2, align 8
  %44 = getelementptr inbounds %struct.global_State, ptr %43, i32 0, i32 10
  %45 = load i8, ptr %44, align 4
  %46 = zext i8 %45 to i32
  %47 = and i32 %46, 24
  %48 = trunc i32 %47 to i8
  %49 = zext i8 %48 to i32
  %50 = or i32 %42, %49
  %51 = trunc i32 %50 to i8
  %52 = load ptr, ptr %3, align 8
  %53 = getelementptr inbounds %struct.GCObject, ptr %52, i32 0, i32 2
  store i8 %51, ptr %53, align 1
  br label %66

54:                                               ; preds = %31, %1
  %55 = load ptr, ptr %3, align 8
  %56 = getelementptr inbounds %struct.GCObject, ptr %55, i32 0, i32 2
  %57 = load i8, ptr %56, align 1
  %58 = zext i8 %57 to i32
  %59 = and i32 %58, 7
  %60 = icmp eq i32 %59, 3
  br i1 %60, label %61, label %65

61:                                               ; preds = %54
  %62 = load ptr, ptr %3, align 8
  %63 = load ptr, ptr %2, align 8
  %64 = getelementptr inbounds %struct.global_State, ptr %63, i32 0, i32 34
  store ptr %62, ptr %64, align 8
  br label %65

65:                                               ; preds = %61, %54
  br label %66

66:                                               ; preds = %65, %37
  %67 = load ptr, ptr %3, align 8
  ret ptr %67
}

declare hidden ptr @luaT_gettmbyobj(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

declare hidden i32 @luaD_pcall(ptr noundef, ptr noundef, ptr noundef, i64 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @dothecall(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %union.StackValue, ptr %9, i64 -2
  call void @luaD_callnoyield(ptr noundef %6, ptr noundef %10, i32 noundef 0)
  ret void
}

declare hidden void @luaE_warnerror(ptr noundef, ptr noundef) #1

declare hidden void @luaD_callnoyield(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @restartcollection(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @cleargraylists(ptr noundef %3)
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.global_State, ptr %4, i32 0, i32 40
  %6 = load ptr, ptr %5, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 2
  %8 = load i8, ptr %7, align 1
  %9 = zext i8 %8 to i32
  %10 = and i32 %9, 24
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %12, label %17

12:                                               ; preds = %1
  %13 = load ptr, ptr %2, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 40
  %16 = load ptr, ptr %15, align 8
  call void @reallymarkobject(ptr noundef %13, ptr noundef %16)
  br label %17

17:                                               ; preds = %12, %1
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 40
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 7
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = and i32 %25, 64
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %44

28:                                               ; preds = %17
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 7
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.GCObject, ptr %32, i32 0, i32 2
  %34 = load i8, ptr %33, align 1
  %35 = zext i8 %34 to i32
  %36 = and i32 %35, 24
  %37 = icmp ne i32 %36, 0
  br i1 %37, label %38, label %44

38:                                               ; preds = %28
  %39 = load ptr, ptr %2, align 8
  %40 = load ptr, ptr %2, align 8
  %41 = getelementptr inbounds %struct.global_State, ptr %40, i32 0, i32 7
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  %43 = load ptr, ptr %42, align 8
  call void @reallymarkobject(ptr noundef %39, ptr noundef %43)
  br label %44

44:                                               ; preds = %38, %28, %17
  %45 = load ptr, ptr %2, align 8
  call void @markmt(ptr noundef %45)
  %46 = load ptr, ptr %2, align 8
  %47 = call i64 @markbeingfnz(ptr noundef %46)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @entersweep(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.lua_State, ptr %4, i32 0, i32 7
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 11
  store i8 3, ptr %8, align 1
  %9 = load ptr, ptr %2, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 21
  %12 = call ptr @sweeptolive(ptr noundef %9, ptr noundef %11)
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 22
  store ptr %12, ptr %14, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @sweepstep(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 22
  %14 = load ptr, ptr %13, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %37

16:                                               ; preds = %4
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 3
  %19 = load i64, ptr %18, align 8
  store i64 %19, ptr %10, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 22
  %23 = load ptr, ptr %22, align 8
  %24 = call ptr @sweeplist(ptr noundef %20, ptr noundef %23, i32 noundef 100, ptr noundef %11)
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 22
  store ptr %24, ptr %26, align 8
  %27 = load ptr, ptr %7, align 8
  %28 = getelementptr inbounds %struct.global_State, ptr %27, i32 0, i32 3
  %29 = load i64, ptr %28, align 8
  %30 = load i64, ptr %10, align 8
  %31 = sub nsw i64 %29, %30
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 4
  %34 = load i64, ptr %33, align 8
  %35 = add i64 %34, %31
  store i64 %35, ptr %33, align 8
  %36 = load i32, ptr %11, align 4
  store i32 %36, ptr %5, align 4
  br label %45

37:                                               ; preds = %4
  %38 = load i32, ptr %8, align 4
  %39 = trunc i32 %38 to i8
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds %struct.global_State, ptr %40, i32 0, i32 11
  store i8 %39, ptr %41, align 1
  %42 = load ptr, ptr %9, align 8
  %43 = load ptr, ptr %7, align 8
  %44 = getelementptr inbounds %struct.global_State, ptr %43, i32 0, i32 22
  store ptr %42, ptr %44, align 8
  store i32 0, ptr %5, align 4
  br label %45

45:                                               ; preds = %37, %16
  %46 = load i32, ptr %5, align 4
  ret i32 %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @runafewfinalizers(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 7
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  store i32 0, ptr %6, align 4
  br label %10

10:                                               ; preds = %23, %2
  %11 = load i32, ptr %6, align 4
  %12 = load i32, ptr %4, align 4
  %13 = icmp slt i32 %11, %12
  br i1 %13, label %14, label %19

14:                                               ; preds = %10
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 29
  %17 = load ptr, ptr %16, align 8
  %18 = icmp ne ptr %17, null
  br label %19

19:                                               ; preds = %14, %10
  %20 = phi i1 [ false, %10 ], [ %18, %14 ]
  br i1 %20, label %21, label %26

21:                                               ; preds = %19
  %22 = load ptr, ptr %3, align 8
  call void @GCTM(ptr noundef %22)
  br label %23

23:                                               ; preds = %21
  %24 = load i32, ptr %6, align 4
  %25 = add nsw i32 %24, 1
  store i32 %25, ptr %6, align 4
  br label %10, !llvm.loop !46

26:                                               ; preds = %19
  %27 = load i32, ptr %6, align 4
  ret i32 %27
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @stepgenfull(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 5
  %9 = load i64, ptr %8, align 8
  store i64 %9, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.global_State, ptr %10, i32 0, i32 12
  %12 = load i8, ptr %11, align 2
  %13 = zext i8 %12 to i32
  %14 = icmp eq i32 %13, 1
  br i1 %14, label %15, label %17

15:                                               ; preds = %2
  %16 = load ptr, ptr %4, align 8
  call void @enterinc(ptr noundef %16)
  br label %17

17:                                               ; preds = %15, %2
  %18 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %18, i32 noundef 1)
  %19 = load ptr, ptr %3, align 8
  %20 = call i64 @atomic(ptr noundef %19)
  store i64 %20, ptr %5, align 8
  %21 = load i64, ptr %5, align 8
  %22 = load i64, ptr %6, align 8
  %23 = load i64, ptr %6, align 8
  %24 = lshr i64 %23, 3
  %25 = add i64 %22, %24
  %26 = icmp ult i64 %21, %25
  br i1 %26, label %27, label %31

27:                                               ; preds = %17
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %4, align 8
  call void @atomic2gen(ptr noundef %28, ptr noundef %29)
  %30 = load ptr, ptr %4, align 8
  call void @setminordebt(ptr noundef %30)
  br label %47

31:                                               ; preds = %17
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 2
  %34 = load i64, ptr %33, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 3
  %37 = load i64, ptr %36, align 8
  %38 = add nsw i64 %34, %37
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 4
  store i64 %38, ptr %40, align 8
  %41 = load ptr, ptr %3, align 8
  call void @entersweep(ptr noundef %41)
  %42 = load ptr, ptr %3, align 8
  call void @luaC_runtilstate(ptr noundef %42, i32 noundef 256)
  %43 = load ptr, ptr %4, align 8
  call void @setpause(ptr noundef %43)
  %44 = load i64, ptr %5, align 8
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.global_State, ptr %45, i32 0, i32 5
  store i64 %44, ptr %46, align 8
  br label %47

47:                                               ; preds = %31, %27
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setpause(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 18
  %9 = load i8, ptr %8, align 4
  %10 = zext i8 %9 to i32
  %11 = mul nsw i32 %10, 4
  store i32 %11, ptr %5, align 4
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.global_State, ptr %12, i32 0, i32 4
  %14 = load i64, ptr %13, align 8
  %15 = udiv i64 %14, 100
  store i64 %15, ptr %6, align 8
  %16 = load i32, ptr %5, align 4
  %17 = sext i32 %16 to i64
  %18 = load i64, ptr %6, align 8
  %19 = sdiv i64 9223372036854775807, %18
  %20 = icmp slt i64 %17, %19
  br i1 %20, label %21, label %26

21:                                               ; preds = %1
  %22 = load i64, ptr %6, align 8
  %23 = load i32, ptr %5, align 4
  %24 = sext i32 %23 to i64
  %25 = mul nsw i64 %22, %24
  br label %27

26:                                               ; preds = %1
  br label %27

27:                                               ; preds = %26, %21
  %28 = phi i64 [ %25, %21 ], [ 9223372036854775807, %26 ]
  store i64 %28, ptr %3, align 8
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds %struct.global_State, ptr %29, i32 0, i32 2
  %31 = load i64, ptr %30, align 8
  %32 = load ptr, ptr %2, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 3
  %34 = load i64, ptr %33, align 8
  %35 = add nsw i64 %31, %34
  %36 = load i64, ptr %3, align 8
  %37 = sub i64 %35, %36
  store i64 %37, ptr %4, align 8
  %38 = load i64, ptr %4, align 8
  %39 = icmp sgt i64 %38, 0
  br i1 %39, label %40, label %41

40:                                               ; preds = %27
  store i64 0, ptr %4, align 8
  br label %41

41:                                               ; preds = %40, %27
  %42 = load ptr, ptr %2, align 8
  %43 = load i64, ptr %4, align 8
  call void @luaE_setdebt(ptr noundef %42, i64 noundef %43)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @youngcollection(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.global_State, ptr %7, i32 0, i32 34
  %9 = load ptr, ptr %8, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %21

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.global_State, ptr %13, i32 0, i32 34
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 33
  %18 = load ptr, ptr %17, align 8
  call void @markold(ptr noundef %12, ptr noundef %15, ptr noundef %18)
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.global_State, ptr %19, i32 0, i32 34
  store ptr null, ptr %20, align 8
  br label %21

21:                                               ; preds = %11, %2
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.global_State, ptr %23, i32 0, i32 23
  %25 = load ptr, ptr %24, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.global_State, ptr %26, i32 0, i32 37
  %28 = load ptr, ptr %27, align 8
  call void @markold(ptr noundef %22, ptr noundef %25, ptr noundef %28)
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.global_State, ptr %30, i32 0, i32 29
  %32 = load ptr, ptr %31, align 8
  call void @markold(ptr noundef %29, ptr noundef %32, ptr noundef null)
  %33 = load ptr, ptr %3, align 8
  %34 = call i64 @atomic(ptr noundef %33)
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.global_State, ptr %35, i32 0, i32 11
  store i8 3, ptr %36, align 1
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 21
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.global_State, ptr %41, i32 0, i32 31
  %43 = load ptr, ptr %42, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.global_State, ptr %44, i32 0, i32 34
  %46 = call ptr @sweepgen(ptr noundef %37, ptr noundef %38, ptr noundef %40, ptr noundef %43, ptr noundef %45)
  store ptr %46, ptr %5, align 8
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = load ptr, ptr %5, align 8
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.global_State, ptr %50, i32 0, i32 32
  %52 = load ptr, ptr %51, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.global_State, ptr %53, i32 0, i32 34
  %55 = call ptr @sweepgen(ptr noundef %47, ptr noundef %48, ptr noundef %49, ptr noundef %52, ptr noundef %54)
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.global_State, ptr %56, i32 0, i32 32
  %58 = load ptr, ptr %57, align 8
  %59 = load ptr, ptr %4, align 8
  %60 = getelementptr inbounds %struct.global_State, ptr %59, i32 0, i32 33
  store ptr %58, ptr %60, align 8
  %61 = load ptr, ptr %5, align 8
  %62 = load ptr, ptr %61, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.global_State, ptr %63, i32 0, i32 32
  store ptr %62, ptr %64, align 8
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.global_State, ptr %65, i32 0, i32 21
  %67 = load ptr, ptr %66, align 8
  %68 = load ptr, ptr %4, align 8
  %69 = getelementptr inbounds %struct.global_State, ptr %68, i32 0, i32 31
  store ptr %67, ptr %69, align 8
  store ptr null, ptr %6, align 8
  %70 = load ptr, ptr %3, align 8
  %71 = load ptr, ptr %4, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.global_State, ptr %72, i32 0, i32 23
  %74 = load ptr, ptr %4, align 8
  %75 = getelementptr inbounds %struct.global_State, ptr %74, i32 0, i32 35
  %76 = load ptr, ptr %75, align 8
  %77 = call ptr @sweepgen(ptr noundef %70, ptr noundef %71, ptr noundef %73, ptr noundef %76, ptr noundef %6)
  store ptr %77, ptr %5, align 8
  %78 = load ptr, ptr %3, align 8
  %79 = load ptr, ptr %4, align 8
  %80 = load ptr, ptr %5, align 8
  %81 = load ptr, ptr %4, align 8
  %82 = getelementptr inbounds %struct.global_State, ptr %81, i32 0, i32 36
  %83 = load ptr, ptr %82, align 8
  %84 = call ptr @sweepgen(ptr noundef %78, ptr noundef %79, ptr noundef %80, ptr noundef %83, ptr noundef %6)
  %85 = load ptr, ptr %4, align 8
  %86 = getelementptr inbounds %struct.global_State, ptr %85, i32 0, i32 36
  %87 = load ptr, ptr %86, align 8
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds %struct.global_State, ptr %88, i32 0, i32 37
  store ptr %87, ptr %89, align 8
  %90 = load ptr, ptr %5, align 8
  %91 = load ptr, ptr %90, align 8
  %92 = load ptr, ptr %4, align 8
  %93 = getelementptr inbounds %struct.global_State, ptr %92, i32 0, i32 36
  store ptr %91, ptr %93, align 8
  %94 = load ptr, ptr %4, align 8
  %95 = getelementptr inbounds %struct.global_State, ptr %94, i32 0, i32 23
  %96 = load ptr, ptr %95, align 8
  %97 = load ptr, ptr %4, align 8
  %98 = getelementptr inbounds %struct.global_State, ptr %97, i32 0, i32 35
  store ptr %96, ptr %98, align 8
  %99 = load ptr, ptr %3, align 8
  %100 = load ptr, ptr %4, align 8
  %101 = load ptr, ptr %4, align 8
  %102 = getelementptr inbounds %struct.global_State, ptr %101, i32 0, i32 29
  %103 = call ptr @sweepgen(ptr noundef %99, ptr noundef %100, ptr noundef %102, ptr noundef null, ptr noundef %6)
  %104 = load ptr, ptr %3, align 8
  %105 = load ptr, ptr %4, align 8
  call void @finishgencycle(ptr noundef %104, ptr noundef %105)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @markold(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %5, align 8
  store ptr %8, ptr %7, align 8
  br label %9

9:                                                ; preds = %38, %3
  %10 = load ptr, ptr %7, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = icmp ne ptr %10, %11
  br i1 %12, label %13, label %42

13:                                               ; preds = %9
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.GCObject, ptr %14, i32 0, i32 2
  %16 = load i8, ptr %15, align 1
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 7
  %19 = icmp eq i32 %18, 3
  br i1 %19, label %20, label %37

20:                                               ; preds = %13
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.GCObject, ptr %21, i32 0, i32 2
  %23 = load i8, ptr %22, align 1
  %24 = zext i8 %23 to i32
  %25 = xor i32 %24, 7
  %26 = trunc i32 %25 to i8
  store i8 %26, ptr %22, align 1
  %27 = load ptr, ptr %7, align 8
  %28 = getelementptr inbounds %struct.GCObject, ptr %27, i32 0, i32 2
  %29 = load i8, ptr %28, align 1
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 32
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %36

33:                                               ; preds = %20
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %7, align 8
  call void @reallymarkobject(ptr noundef %34, ptr noundef %35)
  br label %36

36:                                               ; preds = %33, %20
  br label %37

37:                                               ; preds = %36, %13
  br label %38

38:                                               ; preds = %37
  %39 = load ptr, ptr %7, align 8
  %40 = getelementptr inbounds %struct.GCObject, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  store ptr %41, ptr %7, align 8
  br label %9, !llvm.loop !47

42:                                               ; preds = %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @sweepgen(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 10
  %16 = load i8, ptr %15, align 4
  %17 = zext i8 %16 to i32
  %18 = and i32 %17, 24
  %19 = trunc i32 %18 to i8
  %20 = zext i8 %19 to i32
  store i32 %20, ptr %11, align 4
  br label %21

21:                                               ; preds = %96, %5
  %22 = load ptr, ptr %8, align 8
  %23 = load ptr, ptr %22, align 8
  store ptr %23, ptr %12, align 8
  %24 = load ptr, ptr %9, align 8
  %25 = icmp ne ptr %23, %24
  br i1 %25, label %26, label %97

26:                                               ; preds = %21
  %27 = load ptr, ptr %12, align 8
  %28 = getelementptr inbounds %struct.GCObject, ptr %27, i32 0, i32 2
  %29 = load i8, ptr %28, align 1
  %30 = zext i8 %29 to i32
  %31 = and i32 %30, 24
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %40

33:                                               ; preds = %26
  %34 = load ptr, ptr %12, align 8
  %35 = getelementptr inbounds %struct.GCObject, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = load ptr, ptr %8, align 8
  store ptr %36, ptr %37, align 8
  %38 = load ptr, ptr %6, align 8
  %39 = load ptr, ptr %12, align 8
  call void @freeobj(ptr noundef %38, ptr noundef %39)
  br label %96

40:                                               ; preds = %26
  %41 = load ptr, ptr %12, align 8
  %42 = getelementptr inbounds %struct.GCObject, ptr %41, i32 0, i32 2
  %43 = load i8, ptr %42, align 1
  %44 = zext i8 %43 to i32
  %45 = and i32 %44, 7
  %46 = icmp eq i32 %45, 0
  br i1 %46, label %47, label %60

47:                                               ; preds = %40
  %48 = load ptr, ptr %12, align 8
  %49 = getelementptr inbounds %struct.GCObject, ptr %48, i32 0, i32 2
  %50 = load i8, ptr %49, align 1
  %51 = zext i8 %50 to i32
  %52 = and i32 %51, -64
  store i32 %52, ptr %13, align 4
  %53 = load i32, ptr %13, align 4
  %54 = or i32 %53, 1
  %55 = load i32, ptr %11, align 4
  %56 = or i32 %54, %55
  %57 = trunc i32 %56 to i8
  %58 = load ptr, ptr %12, align 8
  %59 = getelementptr inbounds %struct.GCObject, ptr %58, i32 0, i32 2
  store i8 %57, ptr %59, align 1
  br label %93

60:                                               ; preds = %40
  %61 = load ptr, ptr %12, align 8
  %62 = getelementptr inbounds %struct.GCObject, ptr %61, i32 0, i32 2
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = and i32 %64, -8
  %66 = load ptr, ptr %12, align 8
  %67 = getelementptr inbounds %struct.GCObject, ptr %66, i32 0, i32 2
  %68 = load i8, ptr %67, align 1
  %69 = zext i8 %68 to i32
  %70 = and i32 %69, 7
  %71 = sext i32 %70 to i64
  %72 = getelementptr inbounds [7 x i8], ptr @sweepgen.nextage, i64 0, i64 %71
  %73 = load i8, ptr %72, align 1
  %74 = zext i8 %73 to i32
  %75 = or i32 %65, %74
  %76 = trunc i32 %75 to i8
  %77 = load ptr, ptr %12, align 8
  %78 = getelementptr inbounds %struct.GCObject, ptr %77, i32 0, i32 2
  store i8 %76, ptr %78, align 1
  %79 = load ptr, ptr %12, align 8
  %80 = getelementptr inbounds %struct.GCObject, ptr %79, i32 0, i32 2
  %81 = load i8, ptr %80, align 1
  %82 = zext i8 %81 to i32
  %83 = and i32 %82, 7
  %84 = icmp eq i32 %83, 3
  br i1 %84, label %85, label %92

85:                                               ; preds = %60
  %86 = load ptr, ptr %10, align 8
  %87 = load ptr, ptr %86, align 8
  %88 = icmp eq ptr %87, null
  br i1 %88, label %89, label %92

89:                                               ; preds = %85
  %90 = load ptr, ptr %12, align 8
  %91 = load ptr, ptr %10, align 8
  store ptr %90, ptr %91, align 8
  br label %92

92:                                               ; preds = %89, %85, %60
  br label %93

93:                                               ; preds = %92, %47
  %94 = load ptr, ptr %12, align 8
  %95 = getelementptr inbounds %struct.GCObject, ptr %94, i32 0, i32 0
  store ptr %95, ptr %8, align 8
  br label %96

96:                                               ; preds = %93, %33
  br label %21, !llvm.loop !48

97:                                               ; preds = %21
  %98 = load ptr, ptr %8, align 8
  ret ptr %98
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { nounwind willreturn memory(read) }

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
!32 = distinct !{!32, !7}
!33 = distinct !{!33, !7}
!34 = distinct !{!34, !7}
!35 = distinct !{!35, !7}
!36 = distinct !{!36, !7}
!37 = distinct !{!37, !7}
!38 = distinct !{!38, !7}
!39 = distinct !{!39, !7}
!40 = distinct !{!40, !7}
!41 = distinct !{!41, !7}
!42 = distinct !{!42, !7}
!43 = distinct !{!43, !7}
!44 = distinct !{!44, !7}
!45 = distinct !{!45, !7}
!46 = distinct !{!46, !7}
!47 = distinct !{!47, !7}
!48 = distinct !{!48, !7}
