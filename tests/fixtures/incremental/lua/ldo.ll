; ModuleID = 'ldo.c'
source_filename = "ldo.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.2, i16, i16 }
%union.anon = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%union.StackValue = type { %struct.TValue }
%struct.lua_longjmp = type { ptr, [1 x %struct.__jmp_buf_tag], i32 }
%struct.__jmp_buf_tag = type { [8 x i64], i32, %struct.__sigset_t }
%struct.__sigset_t = type { [16 x i64] }
%struct.UpVal = type { ptr, i8, i8, %union.anon.5, %union.anon.6 }
%union.anon.5 = type { ptr }
%union.anon.6 = type { %struct.anon.7 }
%struct.anon.7 = type { ptr, ptr }
%struct.anon.0 = type { ptr, i32, i32 }
%struct.lua_Debug = type { i32, ptr, ptr, ptr, ptr, i64, i32, i32, i32, i8, i8, i8, i8, i16, i16, [60 x i8], ptr }
%struct.anon.3 = type { i16, i16 }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.CClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x %struct.TValue] }
%struct.CloseP = type { ptr, i32 }
%struct.SParser = type { ptr, %struct.Mbuffer, %struct.Dyndata, ptr, ptr }
%struct.Mbuffer = type { ptr, i64, i64 }
%struct.Dyndata = type { %struct.anon.8, %struct.Labellist, %struct.Labellist }
%struct.anon.8 = type { ptr, i32, i32 }
%struct.Labellist = type { ptr, i32, i32 }
%struct.Zio = type { i64, ptr, ptr, ptr, ptr }

@.str = private unnamed_addr constant [24 x i8] c"error in error handling\00", align 1
@.str.1 = private unnamed_addr constant [15 x i8] c"stack overflow\00", align 1
@.str.2 = private unnamed_addr constant [38 x i8] c"cannot resume non-suspended coroutine\00", align 1
@.str.3 = private unnamed_addr constant [29 x i8] c"cannot resume dead coroutine\00", align 1
@.str.4 = private unnamed_addr constant [17 x i8] c"C stack overflow\00", align 1
@.str.5 = private unnamed_addr constant [42 x i8] c"attempt to yield across a C-call boundary\00", align 1
@.str.6 = private unnamed_addr constant [42 x i8] c"attempt to yield from outside a coroutine\00", align 1
@.str.7 = private unnamed_addr constant [5 x i8] c"\1BLua\00", align 1
@.str.8 = private unnamed_addr constant [7 x i8] c"binary\00", align 1
@.str.9 = private unnamed_addr constant [5 x i8] c"text\00", align 1
@.str.10 = private unnamed_addr constant [42 x i8] c"attempt to load a %s chunk (mode is '%s')\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_seterrorobj(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %13 = load i32, ptr %5, align 4
  switch i32 %13, label %52 [
    i32 4, label %14
    i32 5, label %33
    i32 0, label %49
  ]

14:                                               ; preds = %3
  %15 = load ptr, ptr %6, align 8
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 7
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 41
  %20 = load ptr, ptr %19, align 8
  store ptr %20, ptr %8, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  store ptr %21, ptr %23, align 8
  %24 = load ptr, ptr %8, align 8
  %25 = getelementptr inbounds %struct.TString, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = zext i8 %26 to i32
  %28 = or i32 %27, 64
  %29 = trunc i32 %28 to i8
  %30 = load ptr, ptr %7, align 8
  %31 = getelementptr inbounds %struct.TValue, ptr %30, i32 0, i32 1
  store i8 %29, ptr %31, align 8
  %32 = load ptr, ptr %4, align 8
  br label %68

33:                                               ; preds = %3
  %34 = load ptr, ptr %6, align 8
  store ptr %34, ptr %9, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = call ptr @luaS_newlstr(ptr noundef %35, ptr noundef @.str, i64 noundef 23)
  store ptr %36, ptr %10, align 8
  %37 = load ptr, ptr %10, align 8
  %38 = load ptr, ptr %9, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  store ptr %37, ptr %39, align 8
  %40 = load ptr, ptr %10, align 8
  %41 = getelementptr inbounds %struct.TString, ptr %40, i32 0, i32 1
  %42 = load i8, ptr %41, align 8
  %43 = zext i8 %42 to i32
  %44 = or i32 %43, 64
  %45 = trunc i32 %44 to i8
  %46 = load ptr, ptr %9, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 1
  store i8 %45, ptr %47, align 8
  %48 = load ptr, ptr %4, align 8
  br label %68

49:                                               ; preds = %3
  %50 = load ptr, ptr %6, align 8
  %51 = getelementptr inbounds %struct.TValue, ptr %50, i32 0, i32 1
  store i8 0, ptr %51, align 8
  br label %68

52:                                               ; preds = %3
  %53 = load ptr, ptr %6, align 8
  store ptr %53, ptr %11, align 8
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.lua_State, ptr %54, i32 0, i32 6
  %56 = load ptr, ptr %55, align 8
  %57 = getelementptr inbounds %union.StackValue, ptr %56, i64 -1
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
  %67 = load ptr, ptr %4, align 8
  br label %68

68:                                               ; preds = %52, %49, %33, %14
  %69 = load ptr, ptr %6, align 8
  %70 = getelementptr inbounds %union.StackValue, ptr %69, i64 1
  %71 = load ptr, ptr %4, align 8
  %72 = getelementptr inbounds %struct.lua_State, ptr %71, i32 0, i32 6
  store ptr %70, ptr %72, align 8
  ret void
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaD_throw(ptr noundef %0, i32 noundef %1) #3 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 15
  %10 = load ptr, ptr %9, align 8
  %11 = icmp ne ptr %10, null
  br i1 %11, label %12, label %23

12:                                               ; preds = %2
  %13 = load i32, ptr %4, align 4
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 15
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.lua_longjmp, ptr %16, i32 0, i32 2
  store volatile i32 %13, ptr %17, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 15
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds %struct.lua_longjmp, ptr %20, i32 0, i32 1
  %22 = getelementptr inbounds [1 x %struct.__jmp_buf_tag], ptr %21, i64 0, i64 0
  call void @longjmp(ptr noundef %22, i32 noundef 1) #8
  unreachable

23:                                               ; preds = %2
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.lua_State, ptr %24, i32 0, i32 7
  %26 = load ptr, ptr %25, align 8
  store ptr %26, ptr %5, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = load i32, ptr %4, align 4
  %29 = call i32 @luaE_resetthread(ptr noundef %27, i32 noundef %28)
  store i32 %29, ptr %4, align 4
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.global_State, ptr %30, i32 0, i32 40
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 15
  %34 = load ptr, ptr %33, align 8
  %35 = icmp ne ptr %34, null
  br i1 %35, label %36, label %61

36:                                               ; preds = %23
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.global_State, ptr %37, i32 0, i32 40
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %union.StackValue, ptr %41, i32 1
  store ptr %42, ptr %40, align 8
  store ptr %41, ptr %6, align 8
  %43 = load ptr, ptr %3, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 6
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds %union.StackValue, ptr %45, i64 -1
  store ptr %46, ptr %7, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %7, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %48, ptr align 8 %50, i64 8, i1 false)
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 1
  %53 = load i8, ptr %52, align 8
  %54 = load ptr, ptr %6, align 8
  %55 = getelementptr inbounds %struct.TValue, ptr %54, i32 0, i32 1
  store i8 %53, ptr %55, align 8
  %56 = load ptr, ptr %3, align 8
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.global_State, ptr %57, i32 0, i32 40
  %59 = load ptr, ptr %58, align 8
  %60 = load i32, ptr %4, align 4
  call void @luaD_throw(ptr noundef %59, i32 noundef %60) #9
  unreachable

61:                                               ; preds = %23
  %62 = load ptr, ptr %5, align 8
  %63 = getelementptr inbounds %struct.global_State, ptr %62, i32 0, i32 39
  %64 = load ptr, ptr %63, align 8
  %65 = icmp ne ptr %64, null
  br i1 %65, label %66, label %72

66:                                               ; preds = %61
  %67 = load ptr, ptr %5, align 8
  %68 = getelementptr inbounds %struct.global_State, ptr %67, i32 0, i32 39
  %69 = load ptr, ptr %68, align 8
  %70 = load ptr, ptr %3, align 8
  %71 = call i32 %69(ptr noundef %70)
  br label %72

72:                                               ; preds = %66, %61
  call void @abort() #8
  unreachable
}

; Function Attrs: noreturn nounwind
declare void @longjmp(ptr noundef, i32 noundef) #4

declare hidden i32 @luaE_resetthread(ptr noundef, i32 noundef) #1

; Function Attrs: noreturn nounwind
declare void @abort() #4

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_rawrunprotected(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca %struct.lua_longjmp, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 19
  %11 = load i32, ptr %10, align 8
  store i32 %11, ptr %7, align 4
  %12 = getelementptr inbounds %struct.lua_longjmp, ptr %8, i32 0, i32 2
  store volatile i32 0, ptr %12, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 15
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds %struct.lua_longjmp, ptr %8, i32 0, i32 0
  store ptr %15, ptr %16, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 15
  store ptr %8, ptr %18, align 8
  %19 = getelementptr inbounds %struct.lua_longjmp, ptr %8, i32 0, i32 1
  %20 = getelementptr inbounds [1 x %struct.__jmp_buf_tag], ptr %19, i64 0, i64 0
  %21 = call i32 @_setjmp(ptr noundef %20) #10
  %22 = icmp eq i32 %21, 0
  br i1 %22, label %23, label %27

23:                                               ; preds = %3
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %6, align 8
  call void %24(ptr noundef %25, ptr noundef %26)
  br label %27

27:                                               ; preds = %23, %3
  %28 = getelementptr inbounds %struct.lua_longjmp, ptr %8, i32 0, i32 0
  %29 = load ptr, ptr %28, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 15
  store ptr %29, ptr %31, align 8
  %32 = load i32, ptr %7, align 4
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.lua_State, ptr %33, i32 0, i32 19
  store i32 %32, ptr %34, align 8
  %35 = getelementptr inbounds %struct.lua_longjmp, ptr %8, i32 0, i32 2
  %36 = load volatile i32, ptr %35, align 8
  ret i32 %36
}

; Function Attrs: nounwind returns_twice
declare i32 @_setjmp(ptr noundef) #5

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_reallocstack(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 9
  %14 = load ptr, ptr %13, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 10
  %17 = load ptr, ptr %16, align 8
  %18 = ptrtoint ptr %14 to i64
  %19 = ptrtoint ptr %17 to i64
  %20 = sub i64 %18, %19
  %21 = sdiv exact i64 %20, 16
  %22 = trunc i64 %21 to i32
  store i32 %22, ptr %8, align 4
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 7
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.global_State, ptr %25, i32 0, i32 13
  %27 = load i8, ptr %26, align 1
  %28 = zext i8 %27 to i32
  store i32 %28, ptr %11, align 4
  %29 = load ptr, ptr %5, align 8
  call void @relstack(ptr noundef %29)
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 7
  %32 = load ptr, ptr %31, align 8
  %33 = getelementptr inbounds %struct.global_State, ptr %32, i32 0, i32 13
  store i8 1, ptr %33, align 1
  %34 = load ptr, ptr %5, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 10
  %37 = load ptr, ptr %36, align 8
  %38 = load i32, ptr %8, align 4
  %39 = add nsw i32 %38, 5
  %40 = sext i32 %39 to i64
  %41 = mul i64 %40, 16
  %42 = load i32, ptr %6, align 4
  %43 = add nsw i32 %42, 5
  %44 = sext i32 %43 to i64
  %45 = mul i64 %44, 16
  %46 = call ptr @luaM_realloc_(ptr noundef %34, ptr noundef %37, i64 noundef %41, i64 noundef %45)
  store ptr %46, ptr %10, align 8
  %47 = load i32, ptr %11, align 4
  %48 = trunc i32 %47 to i8
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 7
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.global_State, ptr %51, i32 0, i32 13
  store i8 %48, ptr %52, align 1
  %53 = load ptr, ptr %10, align 8
  %54 = icmp eq ptr %53, null
  %55 = zext i1 %54 to i32
  %56 = icmp ne i32 %55, 0
  %57 = zext i1 %56 to i32
  %58 = sext i32 %57 to i64
  %59 = icmp ne i64 %58, 0
  br i1 %59, label %60, label %67

60:                                               ; preds = %3
  %61 = load ptr, ptr %5, align 8
  call void @correctstack(ptr noundef %61)
  %62 = load i32, ptr %7, align 4
  %63 = icmp ne i32 %62, 0
  br i1 %63, label %64, label %66

64:                                               ; preds = %60
  %65 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %65, i32 noundef 4) #9
  unreachable

66:                                               ; preds = %60
  store i32 0, ptr %4, align 4
  br label %97

67:                                               ; preds = %3
  %68 = load ptr, ptr %10, align 8
  %69 = load ptr, ptr %5, align 8
  %70 = getelementptr inbounds %struct.lua_State, ptr %69, i32 0, i32 10
  store ptr %68, ptr %70, align 8
  %71 = load ptr, ptr %5, align 8
  call void @correctstack(ptr noundef %71)
  %72 = load ptr, ptr %5, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 10
  %74 = load ptr, ptr %73, align 8
  %75 = load i32, ptr %6, align 4
  %76 = sext i32 %75 to i64
  %77 = getelementptr inbounds %union.StackValue, ptr %74, i64 %76
  %78 = load ptr, ptr %5, align 8
  %79 = getelementptr inbounds %struct.lua_State, ptr %78, i32 0, i32 9
  store ptr %77, ptr %79, align 8
  %80 = load i32, ptr %8, align 4
  %81 = add nsw i32 %80, 5
  store i32 %81, ptr %9, align 4
  br label %82

82:                                               ; preds = %93, %67
  %83 = load i32, ptr %9, align 4
  %84 = load i32, ptr %6, align 4
  %85 = add nsw i32 %84, 5
  %86 = icmp slt i32 %83, %85
  br i1 %86, label %87, label %96

87:                                               ; preds = %82
  %88 = load ptr, ptr %10, align 8
  %89 = load i32, ptr %9, align 4
  %90 = sext i32 %89 to i64
  %91 = getelementptr inbounds %union.StackValue, ptr %88, i64 %90
  %92 = getelementptr inbounds %struct.TValue, ptr %91, i32 0, i32 1
  store i8 0, ptr %92, align 8
  br label %93

93:                                               ; preds = %87
  %94 = load i32, ptr %9, align 4
  %95 = add nsw i32 %94, 1
  store i32 %95, ptr %9, align 4
  br label %82, !llvm.loop !6

96:                                               ; preds = %82
  store i32 1, ptr %4, align 4
  br label %97

97:                                               ; preds = %96, %66
  %98 = load i32, ptr %4, align 4
  ret i32 %98
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @relstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.lua_State, ptr %5, i32 0, i32 6
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 10
  %10 = load ptr, ptr %9, align 8
  %11 = ptrtoint ptr %7 to i64
  %12 = ptrtoint ptr %10 to i64
  %13 = sub i64 %11, %12
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 6
  store i64 %13, ptr %15, align 8
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 12
  %18 = load ptr, ptr %17, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 10
  %21 = load ptr, ptr %20, align 8
  %22 = ptrtoint ptr %18 to i64
  %23 = ptrtoint ptr %21 to i64
  %24 = sub i64 %22, %23
  %25 = load ptr, ptr %2, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 12
  store i64 %24, ptr %26, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 11
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %4, align 8
  br label %30

30:                                               ; preds = %45, %1
  %31 = load ptr, ptr %4, align 8
  %32 = icmp ne ptr %31, null
  br i1 %32, label %33, label %50

33:                                               ; preds = %30
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.UpVal, ptr %34, i32 0, i32 3
  %36 = load ptr, ptr %35, align 8
  %37 = load ptr, ptr %2, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 10
  %39 = load ptr, ptr %38, align 8
  %40 = ptrtoint ptr %36 to i64
  %41 = ptrtoint ptr %39 to i64
  %42 = sub i64 %40, %41
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.UpVal, ptr %43, i32 0, i32 3
  store i64 %42, ptr %44, align 8
  br label %45

45:                                               ; preds = %33
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.UpVal, ptr %46, i32 0, i32 4
  %48 = getelementptr inbounds %struct.anon.7, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  store ptr %49, ptr %4, align 8
  br label %30, !llvm.loop !8

50:                                               ; preds = %30
  %51 = load ptr, ptr %2, align 8
  %52 = getelementptr inbounds %struct.lua_State, ptr %51, i32 0, i32 8
  %53 = load ptr, ptr %52, align 8
  store ptr %53, ptr %3, align 8
  br label %54

54:                                               ; preds = %80, %50
  %55 = load ptr, ptr %3, align 8
  %56 = icmp ne ptr %55, null
  br i1 %56, label %57, label %84

57:                                               ; preds = %54
  %58 = load ptr, ptr %3, align 8
  %59 = getelementptr inbounds %struct.CallInfo, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %59, align 8
  %61 = load ptr, ptr %2, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 10
  %63 = load ptr, ptr %62, align 8
  %64 = ptrtoint ptr %60 to i64
  %65 = ptrtoint ptr %63 to i64
  %66 = sub i64 %64, %65
  %67 = load ptr, ptr %3, align 8
  %68 = getelementptr inbounds %struct.CallInfo, ptr %67, i32 0, i32 1
  store i64 %66, ptr %68, align 8
  %69 = load ptr, ptr %3, align 8
  %70 = getelementptr inbounds %struct.CallInfo, ptr %69, i32 0, i32 0
  %71 = load ptr, ptr %70, align 8
  %72 = load ptr, ptr %2, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 10
  %74 = load ptr, ptr %73, align 8
  %75 = ptrtoint ptr %71 to i64
  %76 = ptrtoint ptr %74 to i64
  %77 = sub i64 %75, %76
  %78 = load ptr, ptr %3, align 8
  %79 = getelementptr inbounds %struct.CallInfo, ptr %78, i32 0, i32 0
  store i64 %77, ptr %79, align 8
  br label %80

80:                                               ; preds = %57
  %81 = load ptr, ptr %3, align 8
  %82 = getelementptr inbounds %struct.CallInfo, ptr %81, i32 0, i32 2
  %83 = load ptr, ptr %82, align 8
  store ptr %83, ptr %3, align 8
  br label %54, !llvm.loop !9

84:                                               ; preds = %54
  ret void
}

declare hidden ptr @luaM_realloc_(ptr noundef, ptr noundef, i64 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @correctstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.lua_State, ptr %5, i32 0, i32 10
  %7 = load ptr, ptr %6, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 6
  %10 = load i64, ptr %9, align 8
  %11 = getelementptr inbounds i8, ptr %7, i64 %10
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 6
  store ptr %11, ptr %13, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 10
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 12
  %19 = load i64, ptr %18, align 8
  %20 = getelementptr inbounds i8, ptr %16, i64 %19
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 12
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 11
  %25 = load ptr, ptr %24, align 8
  store ptr %25, ptr %4, align 8
  br label %26

26:                                               ; preds = %39, %1
  %27 = load ptr, ptr %4, align 8
  %28 = icmp ne ptr %27, null
  br i1 %28, label %29, label %44

29:                                               ; preds = %26
  %30 = load ptr, ptr %2, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 10
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.UpVal, ptr %33, i32 0, i32 3
  %35 = load i64, ptr %34, align 8
  %36 = getelementptr inbounds i8, ptr %32, i64 %35
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.UpVal, ptr %37, i32 0, i32 3
  store ptr %36, ptr %38, align 8
  br label %39

39:                                               ; preds = %29
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.UpVal, ptr %40, i32 0, i32 4
  %42 = getelementptr inbounds %struct.anon.7, ptr %41, i32 0, i32 0
  %43 = load ptr, ptr %42, align 8
  store ptr %43, ptr %4, align 8
  br label %26, !llvm.loop !10

44:                                               ; preds = %26
  %45 = load ptr, ptr %2, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 8
  %47 = load ptr, ptr %46, align 8
  store ptr %47, ptr %3, align 8
  br label %48

48:                                               ; preds = %81, %44
  %49 = load ptr, ptr %3, align 8
  %50 = icmp ne ptr %49, null
  br i1 %50, label %51, label %85

51:                                               ; preds = %48
  %52 = load ptr, ptr %2, align 8
  %53 = getelementptr inbounds %struct.lua_State, ptr %52, i32 0, i32 10
  %54 = load ptr, ptr %53, align 8
  %55 = load ptr, ptr %3, align 8
  %56 = getelementptr inbounds %struct.CallInfo, ptr %55, i32 0, i32 1
  %57 = load i64, ptr %56, align 8
  %58 = getelementptr inbounds i8, ptr %54, i64 %57
  %59 = load ptr, ptr %3, align 8
  %60 = getelementptr inbounds %struct.CallInfo, ptr %59, i32 0, i32 1
  store ptr %58, ptr %60, align 8
  %61 = load ptr, ptr %2, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 10
  %63 = load ptr, ptr %62, align 8
  %64 = load ptr, ptr %3, align 8
  %65 = getelementptr inbounds %struct.CallInfo, ptr %64, i32 0, i32 0
  %66 = load i64, ptr %65, align 8
  %67 = getelementptr inbounds i8, ptr %63, i64 %66
  %68 = load ptr, ptr %3, align 8
  %69 = getelementptr inbounds %struct.CallInfo, ptr %68, i32 0, i32 0
  store ptr %67, ptr %69, align 8
  %70 = load ptr, ptr %3, align 8
  %71 = getelementptr inbounds %struct.CallInfo, ptr %70, i32 0, i32 7
  %72 = load i16, ptr %71, align 2
  %73 = zext i16 %72 to i32
  %74 = and i32 %73, 2
  %75 = icmp ne i32 %74, 0
  br i1 %75, label %80, label %76

76:                                               ; preds = %51
  %77 = load ptr, ptr %3, align 8
  %78 = getelementptr inbounds %struct.CallInfo, ptr %77, i32 0, i32 4
  %79 = getelementptr inbounds %struct.anon.0, ptr %78, i32 0, i32 1
  store volatile i32 1, ptr %79, align 8
  br label %80

80:                                               ; preds = %76, %51
  br label %81

81:                                               ; preds = %80
  %82 = load ptr, ptr %3, align 8
  %83 = getelementptr inbounds %struct.CallInfo, ptr %82, i32 0, i32 2
  %84 = load ptr, ptr %83, align 8
  store ptr %84, ptr %3, align 8
  br label %48, !llvm.loop !11

85:                                               ; preds = %48
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_growstack(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
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
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 9
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 10
  %16 = load ptr, ptr %15, align 8
  %17 = ptrtoint ptr %13 to i64
  %18 = ptrtoint ptr %16 to i64
  %19 = sub i64 %17, %18
  %20 = sdiv exact i64 %19, 16
  %21 = trunc i64 %20 to i32
  store i32 %21, ptr %8, align 4
  %22 = load i32, ptr %8, align 4
  %23 = icmp sgt i32 %22, 1000000
  %24 = zext i1 %23 to i32
  %25 = icmp ne i32 %24, 0
  %26 = zext i1 %25 to i32
  %27 = sext i32 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %29, label %35

29:                                               ; preds = %3
  %30 = load i32, ptr %7, align 4
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %34

32:                                               ; preds = %29
  %33 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %33, i32 noundef 5) #9
  unreachable

34:                                               ; preds = %29
  store i32 0, ptr %4, align 4
  br label %87

35:                                               ; preds = %3
  %36 = load i32, ptr %6, align 4
  %37 = icmp slt i32 %36, 1000000
  br i1 %37, label %38, label %77

38:                                               ; preds = %35
  %39 = load i32, ptr %8, align 4
  %40 = mul nsw i32 2, %39
  store i32 %40, ptr %9, align 4
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.lua_State, ptr %41, i32 0, i32 6
  %43 = load ptr, ptr %42, align 8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 10
  %46 = load ptr, ptr %45, align 8
  %47 = ptrtoint ptr %43 to i64
  %48 = ptrtoint ptr %46 to i64
  %49 = sub i64 %47, %48
  %50 = sdiv exact i64 %49, 16
  %51 = trunc i64 %50 to i32
  %52 = load i32, ptr %6, align 4
  %53 = add nsw i32 %51, %52
  store i32 %53, ptr %10, align 4
  %54 = load i32, ptr %9, align 4
  %55 = icmp sgt i32 %54, 1000000
  br i1 %55, label %56, label %57

56:                                               ; preds = %38
  store i32 1000000, ptr %9, align 4
  br label %57

57:                                               ; preds = %56, %38
  %58 = load i32, ptr %9, align 4
  %59 = load i32, ptr %10, align 4
  %60 = icmp slt i32 %58, %59
  br i1 %60, label %61, label %63

61:                                               ; preds = %57
  %62 = load i32, ptr %10, align 4
  store i32 %62, ptr %9, align 4
  br label %63

63:                                               ; preds = %61, %57
  %64 = load i32, ptr %9, align 4
  %65 = icmp sle i32 %64, 1000000
  %66 = zext i1 %65 to i32
  %67 = icmp ne i32 %66, 0
  %68 = zext i1 %67 to i32
  %69 = sext i32 %68 to i64
  %70 = icmp ne i64 %69, 0
  br i1 %70, label %71, label %76

71:                                               ; preds = %63
  %72 = load ptr, ptr %5, align 8
  %73 = load i32, ptr %9, align 4
  %74 = load i32, ptr %7, align 4
  %75 = call i32 @luaD_reallocstack(ptr noundef %72, i32 noundef %73, i32 noundef %74)
  store i32 %75, ptr %4, align 4
  br label %87

76:                                               ; preds = %63
  br label %77

77:                                               ; preds = %76, %35
  br label %78

78:                                               ; preds = %77
  %79 = load ptr, ptr %5, align 8
  %80 = load i32, ptr %7, align 4
  %81 = call i32 @luaD_reallocstack(ptr noundef %79, i32 noundef 1000200, i32 noundef %80)
  %82 = load i32, ptr %7, align 4
  %83 = icmp ne i32 %82, 0
  br i1 %83, label %84, label %86

84:                                               ; preds = %78
  %85 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %85, ptr noundef @.str.1) #9
  unreachable

86:                                               ; preds = %78
  store i32 0, ptr %4, align 4
  br label %87

87:                                               ; preds = %86, %71, %34
  %88 = load i32, ptr %4, align 4
  ret i32 %88
}

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #6

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_shrinkstack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @stackinuse(ptr noundef %6)
  store i32 %7, ptr %3, align 4
  %8 = load i32, ptr %3, align 4
  %9 = icmp sgt i32 %8, 333333
  br i1 %9, label %10, label %11

10:                                               ; preds = %1
  br label %14

11:                                               ; preds = %1
  %12 = load i32, ptr %3, align 4
  %13 = mul nsw i32 %12, 3
  br label %14

14:                                               ; preds = %11, %10
  %15 = phi i32 [ 1000000, %10 ], [ %13, %11 ]
  store i32 %15, ptr %4, align 4
  %16 = load i32, ptr %3, align 4
  %17 = icmp sle i32 %16, 1000000
  br i1 %17, label %18, label %44

18:                                               ; preds = %14
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 9
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %2, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 10
  %24 = load ptr, ptr %23, align 8
  %25 = ptrtoint ptr %21 to i64
  %26 = ptrtoint ptr %24 to i64
  %27 = sub i64 %25, %26
  %28 = sdiv exact i64 %27, 16
  %29 = trunc i64 %28 to i32
  %30 = load i32, ptr %4, align 4
  %31 = icmp sgt i32 %29, %30
  br i1 %31, label %32, label %44

32:                                               ; preds = %18
  %33 = load i32, ptr %3, align 4
  %34 = icmp sgt i32 %33, 500000
  br i1 %34, label %35, label %36

35:                                               ; preds = %32
  br label %39

36:                                               ; preds = %32
  %37 = load i32, ptr %3, align 4
  %38 = mul nsw i32 %37, 2
  br label %39

39:                                               ; preds = %36, %35
  %40 = phi i32 [ 1000000, %35 ], [ %38, %36 ]
  store i32 %40, ptr %5, align 4
  %41 = load ptr, ptr %2, align 8
  %42 = load i32, ptr %5, align 4
  %43 = call i32 @luaD_reallocstack(ptr noundef %41, i32 noundef %42, i32 noundef 0)
  br label %45

44:                                               ; preds = %18, %14
  br label %45

45:                                               ; preds = %44, %39
  %46 = load ptr, ptr %2, align 8
  call void @luaE_shrinkCI(ptr noundef %46)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @stackinuse(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 8
  %11 = load ptr, ptr %10, align 8
  store ptr %11, ptr %3, align 8
  br label %12

12:                                               ; preds = %26, %1
  %13 = load ptr, ptr %3, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %30

15:                                               ; preds = %12
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.CallInfo, ptr %17, i32 0, i32 1
  %19 = load ptr, ptr %18, align 8
  %20 = icmp ult ptr %16, %19
  br i1 %20, label %21, label %25

21:                                               ; preds = %15
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.CallInfo, ptr %22, i32 0, i32 1
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %5, align 8
  br label %25

25:                                               ; preds = %21, %15
  br label %26

26:                                               ; preds = %25
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.CallInfo, ptr %27, i32 0, i32 2
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %3, align 8
  br label %12, !llvm.loop !12

30:                                               ; preds = %12
  %31 = load ptr, ptr %5, align 8
  %32 = load ptr, ptr %2, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 10
  %34 = load ptr, ptr %33, align 8
  %35 = ptrtoint ptr %31 to i64
  %36 = ptrtoint ptr %34 to i64
  %37 = sub i64 %35, %36
  %38 = sdiv exact i64 %37, 16
  %39 = trunc i64 %38 to i32
  %40 = add nsw i32 %39, 1
  store i32 %40, ptr %4, align 4
  %41 = load i32, ptr %4, align 4
  %42 = icmp slt i32 %41, 20
  br i1 %42, label %43, label %44

43:                                               ; preds = %30
  store i32 20, ptr %4, align 4
  br label %44

44:                                               ; preds = %43, %30
  %45 = load i32, ptr %4, align 4
  ret i32 %45
}

declare hidden void @luaE_shrinkCI(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_inctop(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 9
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.lua_State, ptr %6, i32 0, i32 6
  %8 = load ptr, ptr %7, align 8
  %9 = ptrtoint ptr %5 to i64
  %10 = ptrtoint ptr %8 to i64
  %11 = sub i64 %9, %10
  %12 = sdiv exact i64 %11, 16
  %13 = icmp sle i64 %12, 1
  %14 = zext i1 %13 to i32
  %15 = icmp ne i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = sext i32 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %19, label %22

19:                                               ; preds = %1
  %20 = load ptr, ptr %2, align 8
  %21 = call i32 @luaD_growstack(ptr noundef %20, i32 noundef 1, i32 noundef 1)
  br label %23

22:                                               ; preds = %1
  br label %23

23:                                               ; preds = %22, %19
  %24 = load ptr, ptr %2, align 8
  %25 = getelementptr inbounds %struct.lua_State, ptr %24, i32 0, i32 6
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %union.StackValue, ptr %26, i32 1
  store ptr %27, ptr %25, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_hook(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  %13 = alloca ptr, align 8
  %14 = alloca i64, align 8
  %15 = alloca i64, align 8
  %16 = alloca %struct.lua_Debug, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %17 = load ptr, ptr %6, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 17
  %19 = load volatile ptr, ptr %18, align 8
  store ptr %19, ptr %11, align 8
  %20 = load ptr, ptr %11, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %165

22:                                               ; preds = %5
  %23 = load ptr, ptr %6, align 8
  %24 = getelementptr inbounds %struct.lua_State, ptr %23, i32 0, i32 4
  %25 = load i8, ptr %24, align 1
  %26 = zext i8 %25 to i32
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %165

28:                                               ; preds = %22
  store i32 8, ptr %12, align 4
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.lua_State, ptr %29, i32 0, i32 8
  %31 = load ptr, ptr %30, align 8
  store ptr %31, ptr %13, align 8
  %32 = load ptr, ptr %6, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 6
  %34 = load ptr, ptr %33, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.lua_State, ptr %35, i32 0, i32 10
  %37 = load ptr, ptr %36, align 8
  %38 = ptrtoint ptr %34 to i64
  %39 = ptrtoint ptr %37 to i64
  %40 = sub i64 %38, %39
  store i64 %40, ptr %14, align 8
  %41 = load ptr, ptr %13, align 8
  %42 = getelementptr inbounds %struct.CallInfo, ptr %41, i32 0, i32 1
  %43 = load ptr, ptr %42, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 10
  %46 = load ptr, ptr %45, align 8
  %47 = ptrtoint ptr %43 to i64
  %48 = ptrtoint ptr %46 to i64
  %49 = sub i64 %47, %48
  store i64 %49, ptr %15, align 8
  %50 = load i32, ptr %7, align 4
  %51 = getelementptr inbounds %struct.lua_Debug, ptr %16, i32 0, i32 0
  store i32 %50, ptr %51, align 8
  %52 = load i32, ptr %8, align 4
  %53 = getelementptr inbounds %struct.lua_Debug, ptr %16, i32 0, i32 6
  store i32 %52, ptr %53, align 8
  %54 = load ptr, ptr %13, align 8
  %55 = getelementptr inbounds %struct.lua_Debug, ptr %16, i32 0, i32 16
  store ptr %54, ptr %55, align 8
  %56 = load i32, ptr %10, align 4
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %58, label %71

58:                                               ; preds = %28
  %59 = load i32, ptr %12, align 4
  %60 = or i32 %59, 256
  store i32 %60, ptr %12, align 4
  %61 = load i32, ptr %9, align 4
  %62 = trunc i32 %61 to i16
  %63 = load ptr, ptr %13, align 8
  %64 = getelementptr inbounds %struct.CallInfo, ptr %63, i32 0, i32 5
  %65 = getelementptr inbounds %struct.anon.3, ptr %64, i32 0, i32 0
  store i16 %62, ptr %65, align 8
  %66 = load i32, ptr %10, align 4
  %67 = trunc i32 %66 to i16
  %68 = load ptr, ptr %13, align 8
  %69 = getelementptr inbounds %struct.CallInfo, ptr %68, i32 0, i32 5
  %70 = getelementptr inbounds %struct.anon.3, ptr %69, i32 0, i32 1
  store i16 %67, ptr %70, align 2
  br label %71

71:                                               ; preds = %58, %28
  %72 = load ptr, ptr %13, align 8
  %73 = getelementptr inbounds %struct.CallInfo, ptr %72, i32 0, i32 7
  %74 = load i16, ptr %73, align 2
  %75 = zext i16 %74 to i32
  %76 = and i32 %75, 2
  %77 = icmp ne i32 %76, 0
  br i1 %77, label %92, label %78

78:                                               ; preds = %71
  %79 = load ptr, ptr %6, align 8
  %80 = getelementptr inbounds %struct.lua_State, ptr %79, i32 0, i32 6
  %81 = load ptr, ptr %80, align 8
  %82 = load ptr, ptr %13, align 8
  %83 = getelementptr inbounds %struct.CallInfo, ptr %82, i32 0, i32 1
  %84 = load ptr, ptr %83, align 8
  %85 = icmp ult ptr %81, %84
  br i1 %85, label %86, label %92

86:                                               ; preds = %78
  %87 = load ptr, ptr %13, align 8
  %88 = getelementptr inbounds %struct.CallInfo, ptr %87, i32 0, i32 1
  %89 = load ptr, ptr %88, align 8
  %90 = load ptr, ptr %6, align 8
  %91 = getelementptr inbounds %struct.lua_State, ptr %90, i32 0, i32 6
  store ptr %89, ptr %91, align 8
  br label %92

92:                                               ; preds = %86, %78, %71
  %93 = load ptr, ptr %6, align 8
  %94 = getelementptr inbounds %struct.lua_State, ptr %93, i32 0, i32 9
  %95 = load ptr, ptr %94, align 8
  %96 = load ptr, ptr %6, align 8
  %97 = getelementptr inbounds %struct.lua_State, ptr %96, i32 0, i32 6
  %98 = load ptr, ptr %97, align 8
  %99 = ptrtoint ptr %95 to i64
  %100 = ptrtoint ptr %98 to i64
  %101 = sub i64 %99, %100
  %102 = sdiv exact i64 %101, 16
  %103 = icmp sle i64 %102, 20
  %104 = zext i1 %103 to i32
  %105 = icmp ne i32 %104, 0
  %106 = zext i1 %105 to i32
  %107 = sext i32 %106 to i64
  %108 = icmp ne i64 %107, 0
  br i1 %108, label %109, label %112

109:                                              ; preds = %92
  %110 = load ptr, ptr %6, align 8
  %111 = call i32 @luaD_growstack(ptr noundef %110, i32 noundef 20, i32 noundef 1)
  br label %113

112:                                              ; preds = %92
  br label %113

113:                                              ; preds = %112, %109
  %114 = load ptr, ptr %13, align 8
  %115 = getelementptr inbounds %struct.CallInfo, ptr %114, i32 0, i32 1
  %116 = load ptr, ptr %115, align 8
  %117 = load ptr, ptr %6, align 8
  %118 = getelementptr inbounds %struct.lua_State, ptr %117, i32 0, i32 6
  %119 = load ptr, ptr %118, align 8
  %120 = getelementptr inbounds %union.StackValue, ptr %119, i64 20
  %121 = icmp ult ptr %116, %120
  br i1 %121, label %122, label %129

122:                                              ; preds = %113
  %123 = load ptr, ptr %6, align 8
  %124 = getelementptr inbounds %struct.lua_State, ptr %123, i32 0, i32 6
  %125 = load ptr, ptr %124, align 8
  %126 = getelementptr inbounds %union.StackValue, ptr %125, i64 20
  %127 = load ptr, ptr %13, align 8
  %128 = getelementptr inbounds %struct.CallInfo, ptr %127, i32 0, i32 1
  store ptr %126, ptr %128, align 8
  br label %129

129:                                              ; preds = %122, %113
  %130 = load ptr, ptr %6, align 8
  %131 = getelementptr inbounds %struct.lua_State, ptr %130, i32 0, i32 4
  store i8 0, ptr %131, align 1
  %132 = load i32, ptr %12, align 4
  %133 = load ptr, ptr %13, align 8
  %134 = getelementptr inbounds %struct.CallInfo, ptr %133, i32 0, i32 7
  %135 = load i16, ptr %134, align 2
  %136 = zext i16 %135 to i32
  %137 = or i32 %136, %132
  %138 = trunc i32 %137 to i16
  store i16 %138, ptr %134, align 2
  %139 = load ptr, ptr %11, align 8
  %140 = load ptr, ptr %6, align 8
  call void %139(ptr noundef %140, ptr noundef %16)
  %141 = load ptr, ptr %6, align 8
  %142 = getelementptr inbounds %struct.lua_State, ptr %141, i32 0, i32 4
  store i8 1, ptr %142, align 1
  %143 = load ptr, ptr %6, align 8
  %144 = getelementptr inbounds %struct.lua_State, ptr %143, i32 0, i32 10
  %145 = load ptr, ptr %144, align 8
  %146 = load i64, ptr %15, align 8
  %147 = getelementptr inbounds i8, ptr %145, i64 %146
  %148 = load ptr, ptr %13, align 8
  %149 = getelementptr inbounds %struct.CallInfo, ptr %148, i32 0, i32 1
  store ptr %147, ptr %149, align 8
  %150 = load ptr, ptr %6, align 8
  %151 = getelementptr inbounds %struct.lua_State, ptr %150, i32 0, i32 10
  %152 = load ptr, ptr %151, align 8
  %153 = load i64, ptr %14, align 8
  %154 = getelementptr inbounds i8, ptr %152, i64 %153
  %155 = load ptr, ptr %6, align 8
  %156 = getelementptr inbounds %struct.lua_State, ptr %155, i32 0, i32 6
  store ptr %154, ptr %156, align 8
  %157 = load i32, ptr %12, align 4
  %158 = xor i32 %157, -1
  %159 = load ptr, ptr %13, align 8
  %160 = getelementptr inbounds %struct.CallInfo, ptr %159, i32 0, i32 7
  %161 = load i16, ptr %160, align 2
  %162 = zext i16 %161 to i32
  %163 = and i32 %162, %158
  %164 = trunc i32 %163 to i16
  store i16 %164, ptr %160, align 2
  br label %165

165:                                              ; preds = %129, %22, %5
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_hookcall(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.lua_State, ptr %7, i32 0, i32 20
  store i32 0, ptr %8, align 4
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.lua_State, ptr %9, i32 0, i32 23
  %11 = load volatile i32, ptr %10, align 8
  %12 = and i32 %11, 1
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %46

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.CallInfo, ptr %15, i32 0, i32 7
  %17 = load i16, ptr %16, align 2
  %18 = zext i16 %17 to i32
  %19 = and i32 %18, 32
  %20 = icmp ne i32 %19, 0
  %21 = zext i1 %20 to i64
  %22 = select i1 %20, i32 4, i32 0
  store i32 %22, ptr %5, align 4
  %23 = load ptr, ptr %4, align 8
  %24 = getelementptr inbounds %struct.CallInfo, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.LClosure, ptr %27, i32 0, i32 5
  %29 = load ptr, ptr %28, align 8
  store ptr %29, ptr %6, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.CallInfo, ptr %30, i32 0, i32 4
  %32 = getelementptr inbounds %struct.anon.0, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds i32, ptr %33, i32 1
  store ptr %34, ptr %32, align 8
  %35 = load ptr, ptr %3, align 8
  %36 = load i32, ptr %5, align 4
  %37 = load ptr, ptr %6, align 8
  %38 = getelementptr inbounds %struct.Proto, ptr %37, i32 0, i32 3
  %39 = load i8, ptr %38, align 2
  %40 = zext i8 %39 to i32
  call void @luaD_hook(ptr noundef %35, i32 noundef %36, i32 noundef -1, i32 noundef 1, i32 noundef %40)
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.CallInfo, ptr %41, i32 0, i32 4
  %43 = getelementptr inbounds %struct.anon.0, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds i32, ptr %44, i32 -1
  store ptr %45, ptr %43, align 8
  br label %46

46:                                               ; preds = %14, %2
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_poscall(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.CallInfo, ptr %8, i32 0, i32 6
  %10 = load i16, ptr %9, align 4
  %11 = sext i16 %10 to i32
  store i32 %11, ptr %7, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 23
  %14 = load volatile i32, ptr %13, align 8
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %20

16:                                               ; preds = %3
  %17 = load i32, ptr %7, align 4
  %18 = icmp slt i32 %17, -1
  %19 = xor i1 %18, true
  br label %20

20:                                               ; preds = %16, %3
  %21 = phi i1 [ false, %3 ], [ %19, %16 ]
  %22 = zext i1 %21 to i32
  %23 = icmp ne i32 %22, 0
  %24 = zext i1 %23 to i32
  %25 = sext i32 %24 to i64
  %26 = icmp ne i64 %25, 0
  br i1 %26, label %27, label %31

27:                                               ; preds = %20
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = load i32, ptr %6, align 4
  call void @rethook(ptr noundef %28, ptr noundef %29, i32 noundef %30)
  br label %31

31:                                               ; preds = %27, %20
  %32 = load ptr, ptr %4, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.CallInfo, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = load i32, ptr %6, align 4
  %37 = load i32, ptr %7, align 4
  call void @moveresults(ptr noundef %32, ptr noundef %35, i32 noundef %36, i32 noundef %37)
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.CallInfo, ptr %38, i32 0, i32 2
  %40 = load ptr, ptr %39, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.lua_State, ptr %41, i32 0, i32 8
  store ptr %40, ptr %42, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @rethook(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 23
  %13 = load volatile i32, ptr %12, align 8
  %14 = and i32 %13, 2
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %81

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 6
  %19 = load ptr, ptr %18, align 8
  %20 = load i32, ptr %6, align 4
  %21 = sext i32 %20 to i64
  %22 = sub i64 0, %21
  %23 = getelementptr inbounds %union.StackValue, ptr %19, i64 %22
  store ptr %23, ptr %7, align 8
  store i32 0, ptr %8, align 4
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.CallInfo, ptr %24, i32 0, i32 7
  %26 = load i16, ptr %25, align 2
  %27 = zext i16 %26 to i32
  %28 = and i32 %27, 2
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %54, label %30

30:                                               ; preds = %16
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.CallInfo, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.LClosure, ptr %35, i32 0, i32 5
  %37 = load ptr, ptr %36, align 8
  store ptr %37, ptr %10, align 8
  %38 = load ptr, ptr %10, align 8
  %39 = getelementptr inbounds %struct.Proto, ptr %38, i32 0, i32 4
  %40 = load i8, ptr %39, align 1
  %41 = icmp ne i8 %40, 0
  br i1 %41, label %42, label %53

42:                                               ; preds = %30
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.CallInfo, ptr %43, i32 0, i32 4
  %45 = getelementptr inbounds %struct.anon.0, ptr %44, i32 0, i32 2
  %46 = load i32, ptr %45, align 4
  %47 = load ptr, ptr %10, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 3
  %49 = load i8, ptr %48, align 2
  %50 = zext i8 %49 to i32
  %51 = add nsw i32 %46, %50
  %52 = add nsw i32 %51, 1
  store i32 %52, ptr %8, align 4
  br label %53

53:                                               ; preds = %42, %30
  br label %54

54:                                               ; preds = %53, %16
  %55 = load i32, ptr %8, align 4
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %struct.CallInfo, ptr %56, i32 0, i32 0
  %58 = load ptr, ptr %57, align 8
  %59 = sext i32 %55 to i64
  %60 = getelementptr inbounds %union.StackValue, ptr %58, i64 %59
  store ptr %60, ptr %57, align 8
  %61 = load ptr, ptr %7, align 8
  %62 = load ptr, ptr %5, align 8
  %63 = getelementptr inbounds %struct.CallInfo, ptr %62, i32 0, i32 0
  %64 = load ptr, ptr %63, align 8
  %65 = ptrtoint ptr %61 to i64
  %66 = ptrtoint ptr %64 to i64
  %67 = sub i64 %65, %66
  %68 = sdiv exact i64 %67, 16
  %69 = trunc i64 %68 to i16
  %70 = zext i16 %69 to i32
  store i32 %70, ptr %9, align 4
  %71 = load ptr, ptr %4, align 8
  %72 = load i32, ptr %9, align 4
  %73 = load i32, ptr %6, align 4
  call void @luaD_hook(ptr noundef %71, i32 noundef 1, i32 noundef -1, i32 noundef %72, i32 noundef %73)
  %74 = load i32, ptr %8, align 4
  %75 = load ptr, ptr %5, align 8
  %76 = getelementptr inbounds %struct.CallInfo, ptr %75, i32 0, i32 0
  %77 = load ptr, ptr %76, align 8
  %78 = sext i32 %74 to i64
  %79 = sub i64 0, %78
  %80 = getelementptr inbounds %union.StackValue, ptr %77, i64 %79
  store ptr %80, ptr %76, align 8
  br label %81

81:                                               ; preds = %54, %3
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.CallInfo, ptr %82, i32 0, i32 2
  %84 = load ptr, ptr %83, align 8
  store ptr %84, ptr %5, align 8
  %85 = getelementptr inbounds %struct.CallInfo, ptr %84, i32 0, i32 7
  %86 = load i16, ptr %85, align 2
  %87 = zext i16 %86 to i32
  %88 = and i32 %87, 2
  %89 = icmp ne i32 %88, 0
  br i1 %89, label %112, label %90

90:                                               ; preds = %81
  %91 = load ptr, ptr %5, align 8
  %92 = getelementptr inbounds %struct.CallInfo, ptr %91, i32 0, i32 4
  %93 = getelementptr inbounds %struct.anon.0, ptr %92, i32 0, i32 0
  %94 = load ptr, ptr %93, align 8
  %95 = load ptr, ptr %5, align 8
  %96 = getelementptr inbounds %struct.CallInfo, ptr %95, i32 0, i32 0
  %97 = load ptr, ptr %96, align 8
  %98 = getelementptr inbounds %struct.TValue, ptr %97, i32 0, i32 0
  %99 = load ptr, ptr %98, align 8
  %100 = getelementptr inbounds %struct.LClosure, ptr %99, i32 0, i32 5
  %101 = load ptr, ptr %100, align 8
  %102 = getelementptr inbounds %struct.Proto, ptr %101, i32 0, i32 16
  %103 = load ptr, ptr %102, align 8
  %104 = ptrtoint ptr %94 to i64
  %105 = ptrtoint ptr %103 to i64
  %106 = sub i64 %104, %105
  %107 = sdiv exact i64 %106, 4
  %108 = trunc i64 %107 to i32
  %109 = sub nsw i32 %108, 1
  %110 = load ptr, ptr %4, align 8
  %111 = getelementptr inbounds %struct.lua_State, ptr %110, i32 0, i32 20
  store i32 %109, ptr %111, align 4
  br label %112

112:                                              ; preds = %90, %81
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @moveresults(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca i64, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %16 = load i32, ptr %8, align 4
  switch i32 %16, label %53 [
    i32 0, label %17
    i32 1, label %21
    i32 -1, label %51
  ]

17:                                               ; preds = %4
  %18 = load ptr, ptr %6, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 6
  store ptr %18, ptr %20, align 8
  br label %174

21:                                               ; preds = %4
  %22 = load i32, ptr %7, align 4
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %27

24:                                               ; preds = %21
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  store i8 0, ptr %26, align 8
  br label %46

27:                                               ; preds = %21
  %28 = load ptr, ptr %6, align 8
  store ptr %28, ptr %11, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.lua_State, ptr %29, i32 0, i32 6
  %31 = load ptr, ptr %30, align 8
  %32 = load i32, ptr %7, align 4
  %33 = sext i32 %32 to i64
  %34 = sub i64 0, %33
  %35 = getelementptr inbounds %union.StackValue, ptr %31, i64 %34
  store ptr %35, ptr %12, align 8
  %36 = load ptr, ptr %11, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  %38 = load ptr, ptr %12, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %37, ptr align 8 %39, i64 8, i1 false)
  %40 = load ptr, ptr %12, align 8
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 1
  %42 = load i8, ptr %41, align 8
  %43 = load ptr, ptr %11, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  store i8 %42, ptr %44, align 8
  %45 = load ptr, ptr %5, align 8
  br label %46

46:                                               ; preds = %27, %24
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds %union.StackValue, ptr %47, i64 1
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.lua_State, ptr %49, i32 0, i32 6
  store ptr %48, ptr %50, align 8
  br label %174

51:                                               ; preds = %4
  %52 = load i32, ptr %7, align 4
  store i32 %52, ptr %8, align 4
  br label %113

53:                                               ; preds = %4
  %54 = load i32, ptr %8, align 4
  %55 = icmp slt i32 %54, -1
  br i1 %55, label %56, label %112

56:                                               ; preds = %53
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.lua_State, ptr %57, i32 0, i32 8
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds %struct.CallInfo, ptr %59, i32 0, i32 7
  %61 = load i16, ptr %60, align 2
  %62 = zext i16 %61 to i32
  %63 = or i32 %62, 512
  %64 = trunc i32 %63 to i16
  store i16 %64, ptr %60, align 2
  %65 = load i32, ptr %7, align 4
  %66 = load ptr, ptr %5, align 8
  %67 = getelementptr inbounds %struct.lua_State, ptr %66, i32 0, i32 8
  %68 = load ptr, ptr %67, align 8
  %69 = getelementptr inbounds %struct.CallInfo, ptr %68, i32 0, i32 5
  store i32 %65, ptr %69, align 8
  %70 = load ptr, ptr %5, align 8
  %71 = load ptr, ptr %6, align 8
  %72 = call ptr @luaF_close(ptr noundef %70, ptr noundef %71, i32 noundef -1, i32 noundef 1)
  store ptr %72, ptr %6, align 8
  %73 = load ptr, ptr %5, align 8
  %74 = getelementptr inbounds %struct.lua_State, ptr %73, i32 0, i32 8
  %75 = load ptr, ptr %74, align 8
  %76 = getelementptr inbounds %struct.CallInfo, ptr %75, i32 0, i32 7
  %77 = load i16, ptr %76, align 2
  %78 = zext i16 %77 to i32
  %79 = and i32 %78, -513
  %80 = trunc i32 %79 to i16
  store i16 %80, ptr %76, align 2
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds %struct.lua_State, ptr %81, i32 0, i32 23
  %83 = load volatile i32, ptr %82, align 8
  %84 = icmp ne i32 %83, 0
  br i1 %84, label %85, label %103

85:                                               ; preds = %56
  %86 = load ptr, ptr %6, align 8
  %87 = load ptr, ptr %5, align 8
  %88 = getelementptr inbounds %struct.lua_State, ptr %87, i32 0, i32 10
  %89 = load ptr, ptr %88, align 8
  %90 = ptrtoint ptr %86 to i64
  %91 = ptrtoint ptr %89 to i64
  %92 = sub i64 %90, %91
  store i64 %92, ptr %13, align 8
  %93 = load ptr, ptr %5, align 8
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %struct.lua_State, ptr %94, i32 0, i32 8
  %96 = load ptr, ptr %95, align 8
  %97 = load i32, ptr %7, align 4
  call void @rethook(ptr noundef %93, ptr noundef %96, i32 noundef %97)
  %98 = load ptr, ptr %5, align 8
  %99 = getelementptr inbounds %struct.lua_State, ptr %98, i32 0, i32 10
  %100 = load ptr, ptr %99, align 8
  %101 = load i64, ptr %13, align 8
  %102 = getelementptr inbounds i8, ptr %100, i64 %101
  store ptr %102, ptr %6, align 8
  br label %103

103:                                              ; preds = %85, %56
  %104 = load i32, ptr %8, align 4
  %105 = sub nsw i32 0, %104
  %106 = sub nsw i32 %105, 3
  store i32 %106, ptr %8, align 4
  %107 = load i32, ptr %8, align 4
  %108 = icmp eq i32 %107, -1
  br i1 %108, label %109, label %111

109:                                              ; preds = %103
  %110 = load i32, ptr %7, align 4
  store i32 %110, ptr %8, align 4
  br label %111

111:                                              ; preds = %109, %103
  br label %112

112:                                              ; preds = %111, %53
  br label %113

113:                                              ; preds = %112, %51
  %114 = load ptr, ptr %5, align 8
  %115 = getelementptr inbounds %struct.lua_State, ptr %114, i32 0, i32 6
  %116 = load ptr, ptr %115, align 8
  %117 = load i32, ptr %7, align 4
  %118 = sext i32 %117 to i64
  %119 = sub i64 0, %118
  %120 = getelementptr inbounds %union.StackValue, ptr %116, i64 %119
  store ptr %120, ptr %9, align 8
  %121 = load i32, ptr %7, align 4
  %122 = load i32, ptr %8, align 4
  %123 = icmp sgt i32 %121, %122
  br i1 %123, label %124, label %126

124:                                              ; preds = %113
  %125 = load i32, ptr %8, align 4
  store i32 %125, ptr %7, align 4
  br label %126

126:                                              ; preds = %124, %113
  store i32 0, ptr %10, align 4
  br label %127

127:                                              ; preds = %150, %126
  %128 = load i32, ptr %10, align 4
  %129 = load i32, ptr %7, align 4
  %130 = icmp slt i32 %128, %129
  br i1 %130, label %131, label %153

131:                                              ; preds = %127
  %132 = load ptr, ptr %6, align 8
  %133 = load i32, ptr %10, align 4
  %134 = sext i32 %133 to i64
  %135 = getelementptr inbounds %union.StackValue, ptr %132, i64 %134
  store ptr %135, ptr %14, align 8
  %136 = load ptr, ptr %9, align 8
  %137 = load i32, ptr %10, align 4
  %138 = sext i32 %137 to i64
  %139 = getelementptr inbounds %union.StackValue, ptr %136, i64 %138
  store ptr %139, ptr %15, align 8
  %140 = load ptr, ptr %14, align 8
  %141 = getelementptr inbounds %struct.TValue, ptr %140, i32 0, i32 0
  %142 = load ptr, ptr %15, align 8
  %143 = getelementptr inbounds %struct.TValue, ptr %142, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %141, ptr align 8 %143, i64 8, i1 false)
  %144 = load ptr, ptr %15, align 8
  %145 = getelementptr inbounds %struct.TValue, ptr %144, i32 0, i32 1
  %146 = load i8, ptr %145, align 8
  %147 = load ptr, ptr %14, align 8
  %148 = getelementptr inbounds %struct.TValue, ptr %147, i32 0, i32 1
  store i8 %146, ptr %148, align 8
  %149 = load ptr, ptr %5, align 8
  br label %150

150:                                              ; preds = %131
  %151 = load i32, ptr %10, align 4
  %152 = add nsw i32 %151, 1
  store i32 %152, ptr %10, align 4
  br label %127, !llvm.loop !13

153:                                              ; preds = %127
  br label %154

154:                                              ; preds = %164, %153
  %155 = load i32, ptr %10, align 4
  %156 = load i32, ptr %8, align 4
  %157 = icmp slt i32 %155, %156
  br i1 %157, label %158, label %167

158:                                              ; preds = %154
  %159 = load ptr, ptr %6, align 8
  %160 = load i32, ptr %10, align 4
  %161 = sext i32 %160 to i64
  %162 = getelementptr inbounds %union.StackValue, ptr %159, i64 %161
  %163 = getelementptr inbounds %struct.TValue, ptr %162, i32 0, i32 1
  store i8 0, ptr %163, align 8
  br label %164

164:                                              ; preds = %158
  %165 = load i32, ptr %10, align 4
  %166 = add nsw i32 %165, 1
  store i32 %166, ptr %10, align 4
  br label %154, !llvm.loop !14

167:                                              ; preds = %154
  %168 = load ptr, ptr %6, align 8
  %169 = load i32, ptr %8, align 4
  %170 = sext i32 %169 to i64
  %171 = getelementptr inbounds %union.StackValue, ptr %168, i64 %170
  %172 = load ptr, ptr %5, align 8
  %173 = getelementptr inbounds %struct.lua_State, ptr %172, i32 0, i32 6
  store ptr %171, ptr %173, align 8
  br label %174

174:                                              ; preds = %167, %46, %17
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_pretailcall(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  %16 = alloca i64, align 8
  %17 = alloca ptr, align 8
  %18 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store i32 %3, ptr %10, align 4
  store i32 %4, ptr %11, align 4
  br label %19

19:                                               ; preds = %182, %5
  %20 = load ptr, ptr %9, align 8
  %21 = getelementptr inbounds %struct.TValue, ptr %20, i32 0, i32 1
  %22 = load i8, ptr %21, align 8
  %23 = zext i8 %22 to i32
  %24 = and i32 %23, 63
  switch i32 %24, label %182 [
    i32 38, label %25
    i32 22, label %34
    i32 6, label %41
  ]

25:                                               ; preds = %19
  %26 = load ptr, ptr %7, align 8
  %27 = load ptr, ptr %9, align 8
  %28 = load ptr, ptr %9, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %struct.CClosure, ptr %30, i32 0, i32 5
  %32 = load ptr, ptr %31, align 8
  %33 = call i32 @precallC(ptr noundef %26, ptr noundef %27, i32 noundef -1, ptr noundef %32)
  store i32 %33, ptr %6, align 4
  br label %188

34:                                               ; preds = %19
  %35 = load ptr, ptr %7, align 8
  %36 = load ptr, ptr %9, align 8
  %37 = load ptr, ptr %9, align 8
  %38 = getelementptr inbounds %struct.TValue, ptr %37, i32 0, i32 0
  %39 = load ptr, ptr %38, align 8
  %40 = call i32 @precallC(ptr noundef %35, ptr noundef %36, i32 noundef -1, ptr noundef %39)
  store i32 %40, ptr %6, align 4
  br label %188

41:                                               ; preds = %19
  %42 = load ptr, ptr %9, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.LClosure, ptr %44, i32 0, i32 5
  %46 = load ptr, ptr %45, align 8
  store ptr %46, ptr %12, align 8
  %47 = load ptr, ptr %12, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 5
  %49 = load i8, ptr %48, align 4
  %50 = zext i8 %49 to i32
  store i32 %50, ptr %13, align 4
  %51 = load ptr, ptr %12, align 8
  %52 = getelementptr inbounds %struct.Proto, ptr %51, i32 0, i32 3
  %53 = load i8, ptr %52, align 2
  %54 = zext i8 %53 to i32
  store i32 %54, ptr %14, align 4
  %55 = load ptr, ptr %7, align 8
  %56 = getelementptr inbounds %struct.lua_State, ptr %55, i32 0, i32 9
  %57 = load ptr, ptr %56, align 8
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.lua_State, ptr %58, i32 0, i32 6
  %60 = load ptr, ptr %59, align 8
  %61 = ptrtoint ptr %57 to i64
  %62 = ptrtoint ptr %60 to i64
  %63 = sub i64 %61, %62
  %64 = sdiv exact i64 %63, 16
  %65 = load i32, ptr %13, align 4
  %66 = load i32, ptr %11, align 4
  %67 = sub nsw i32 %65, %66
  %68 = sext i32 %67 to i64
  %69 = icmp sle i64 %64, %68
  %70 = zext i1 %69 to i32
  %71 = icmp ne i32 %70, 0
  %72 = zext i1 %71 to i32
  %73 = sext i32 %72 to i64
  %74 = icmp ne i64 %73, 0
  br i1 %74, label %75, label %102

75:                                               ; preds = %41
  %76 = load ptr, ptr %9, align 8
  %77 = load ptr, ptr %7, align 8
  %78 = getelementptr inbounds %struct.lua_State, ptr %77, i32 0, i32 10
  %79 = load ptr, ptr %78, align 8
  %80 = ptrtoint ptr %76 to i64
  %81 = ptrtoint ptr %79 to i64
  %82 = sub i64 %80, %81
  store i64 %82, ptr %16, align 8
  %83 = load ptr, ptr %7, align 8
  %84 = getelementptr inbounds %struct.lua_State, ptr %83, i32 0, i32 7
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %struct.global_State, ptr %85, i32 0, i32 3
  %87 = load i64, ptr %86, align 8
  %88 = icmp sgt i64 %87, 0
  br i1 %88, label %89, label %91

89:                                               ; preds = %75
  %90 = load ptr, ptr %7, align 8
  call void @luaC_step(ptr noundef %90)
  br label %91

91:                                               ; preds = %89, %75
  %92 = load ptr, ptr %7, align 8
  %93 = load i32, ptr %13, align 4
  %94 = load i32, ptr %11, align 4
  %95 = sub nsw i32 %93, %94
  %96 = call i32 @luaD_growstack(ptr noundef %92, i32 noundef %95, i32 noundef 1)
  %97 = load ptr, ptr %7, align 8
  %98 = getelementptr inbounds %struct.lua_State, ptr %97, i32 0, i32 10
  %99 = load ptr, ptr %98, align 8
  %100 = load i64, ptr %16, align 8
  %101 = getelementptr inbounds i8, ptr %99, i64 %100
  store ptr %101, ptr %9, align 8
  br label %103

102:                                              ; preds = %41
  br label %103

103:                                              ; preds = %102, %91
  %104 = load i32, ptr %11, align 4
  %105 = load ptr, ptr %8, align 8
  %106 = getelementptr inbounds %struct.CallInfo, ptr %105, i32 0, i32 0
  %107 = load ptr, ptr %106, align 8
  %108 = sext i32 %104 to i64
  %109 = sub i64 0, %108
  %110 = getelementptr inbounds %union.StackValue, ptr %107, i64 %109
  store ptr %110, ptr %106, align 8
  store i32 0, ptr %15, align 4
  br label %111

111:                                              ; preds = %136, %103
  %112 = load i32, ptr %15, align 4
  %113 = load i32, ptr %10, align 4
  %114 = icmp slt i32 %112, %113
  br i1 %114, label %115, label %139

115:                                              ; preds = %111
  %116 = load ptr, ptr %8, align 8
  %117 = getelementptr inbounds %struct.CallInfo, ptr %116, i32 0, i32 0
  %118 = load ptr, ptr %117, align 8
  %119 = load i32, ptr %15, align 4
  %120 = sext i32 %119 to i64
  %121 = getelementptr inbounds %union.StackValue, ptr %118, i64 %120
  store ptr %121, ptr %17, align 8
  %122 = load ptr, ptr %9, align 8
  %123 = load i32, ptr %15, align 4
  %124 = sext i32 %123 to i64
  %125 = getelementptr inbounds %union.StackValue, ptr %122, i64 %124
  store ptr %125, ptr %18, align 8
  %126 = load ptr, ptr %17, align 8
  %127 = getelementptr inbounds %struct.TValue, ptr %126, i32 0, i32 0
  %128 = load ptr, ptr %18, align 8
  %129 = getelementptr inbounds %struct.TValue, ptr %128, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %127, ptr align 8 %129, i64 8, i1 false)
  %130 = load ptr, ptr %18, align 8
  %131 = getelementptr inbounds %struct.TValue, ptr %130, i32 0, i32 1
  %132 = load i8, ptr %131, align 8
  %133 = load ptr, ptr %17, align 8
  %134 = getelementptr inbounds %struct.TValue, ptr %133, i32 0, i32 1
  store i8 %132, ptr %134, align 8
  %135 = load ptr, ptr %7, align 8
  br label %136

136:                                              ; preds = %115
  %137 = load i32, ptr %15, align 4
  %138 = add nsw i32 %137, 1
  store i32 %138, ptr %15, align 4
  br label %111, !llvm.loop !15

139:                                              ; preds = %111
  %140 = load ptr, ptr %8, align 8
  %141 = getelementptr inbounds %struct.CallInfo, ptr %140, i32 0, i32 0
  %142 = load ptr, ptr %141, align 8
  store ptr %142, ptr %9, align 8
  br label %143

143:                                              ; preds = %153, %139
  %144 = load i32, ptr %10, align 4
  %145 = load i32, ptr %14, align 4
  %146 = icmp sle i32 %144, %145
  br i1 %146, label %147, label %156

147:                                              ; preds = %143
  %148 = load ptr, ptr %9, align 8
  %149 = load i32, ptr %10, align 4
  %150 = sext i32 %149 to i64
  %151 = getelementptr inbounds %union.StackValue, ptr %148, i64 %150
  %152 = getelementptr inbounds %struct.TValue, ptr %151, i32 0, i32 1
  store i8 0, ptr %152, align 8
  br label %153

153:                                              ; preds = %147
  %154 = load i32, ptr %10, align 4
  %155 = add nsw i32 %154, 1
  store i32 %155, ptr %10, align 4
  br label %143, !llvm.loop !16

156:                                              ; preds = %143
  %157 = load ptr, ptr %9, align 8
  %158 = getelementptr inbounds %union.StackValue, ptr %157, i64 1
  %159 = load i32, ptr %13, align 4
  %160 = sext i32 %159 to i64
  %161 = getelementptr inbounds %union.StackValue, ptr %158, i64 %160
  %162 = load ptr, ptr %8, align 8
  %163 = getelementptr inbounds %struct.CallInfo, ptr %162, i32 0, i32 1
  store ptr %161, ptr %163, align 8
  %164 = load ptr, ptr %12, align 8
  %165 = getelementptr inbounds %struct.Proto, ptr %164, i32 0, i32 16
  %166 = load ptr, ptr %165, align 8
  %167 = load ptr, ptr %8, align 8
  %168 = getelementptr inbounds %struct.CallInfo, ptr %167, i32 0, i32 4
  %169 = getelementptr inbounds %struct.anon.0, ptr %168, i32 0, i32 0
  store ptr %166, ptr %169, align 8
  %170 = load ptr, ptr %8, align 8
  %171 = getelementptr inbounds %struct.CallInfo, ptr %170, i32 0, i32 7
  %172 = load i16, ptr %171, align 2
  %173 = zext i16 %172 to i32
  %174 = or i32 %173, 32
  %175 = trunc i32 %174 to i16
  store i16 %175, ptr %171, align 2
  %176 = load ptr, ptr %9, align 8
  %177 = load i32, ptr %10, align 4
  %178 = sext i32 %177 to i64
  %179 = getelementptr inbounds %union.StackValue, ptr %176, i64 %178
  %180 = load ptr, ptr %7, align 8
  %181 = getelementptr inbounds %struct.lua_State, ptr %180, i32 0, i32 6
  store ptr %179, ptr %181, align 8
  store i32 -1, ptr %6, align 4
  br label %188

182:                                              ; preds = %19
  %183 = load ptr, ptr %7, align 8
  %184 = load ptr, ptr %9, align 8
  %185 = call ptr @tryfuncTM(ptr noundef %183, ptr noundef %184)
  store ptr %185, ptr %9, align 8
  %186 = load i32, ptr %10, align 4
  %187 = add nsw i32 %186, 1
  store i32 %187, ptr %10, align 4
  br label %19

188:                                              ; preds = %156, %34, %25
  %189 = load i32, ptr %6, align 4
  ret i32 %189
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @precallC(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca i64, align 8
  %12 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 9
  %15 = load ptr, ptr %14, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  %19 = ptrtoint ptr %15 to i64
  %20 = ptrtoint ptr %18 to i64
  %21 = sub i64 %19, %20
  %22 = sdiv exact i64 %21, 16
  %23 = icmp sle i64 %22, 20
  %24 = zext i1 %23 to i32
  %25 = icmp ne i32 %24, 0
  %26 = zext i1 %25 to i32
  %27 = sext i32 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %29, label %53

29:                                               ; preds = %4
  %30 = load ptr, ptr %6, align 8
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 10
  %33 = load ptr, ptr %32, align 8
  %34 = ptrtoint ptr %30 to i64
  %35 = ptrtoint ptr %33 to i64
  %36 = sub i64 %34, %35
  store i64 %36, ptr %11, align 8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds %struct.lua_State, ptr %37, i32 0, i32 7
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.global_State, ptr %39, i32 0, i32 3
  %41 = load i64, ptr %40, align 8
  %42 = icmp sgt i64 %41, 0
  br i1 %42, label %43, label %45

43:                                               ; preds = %29
  %44 = load ptr, ptr %5, align 8
  call void @luaC_step(ptr noundef %44)
  br label %45

45:                                               ; preds = %43, %29
  %46 = load ptr, ptr %5, align 8
  %47 = call i32 @luaD_growstack(ptr noundef %46, i32 noundef 20, i32 noundef 1)
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.lua_State, ptr %48, i32 0, i32 10
  %50 = load ptr, ptr %49, align 8
  %51 = load i64, ptr %11, align 8
  %52 = getelementptr inbounds i8, ptr %50, i64 %51
  store ptr %52, ptr %6, align 8
  br label %54

53:                                               ; preds = %4
  br label %54

54:                                               ; preds = %53, %45
  %55 = load ptr, ptr %5, align 8
  %56 = load ptr, ptr %6, align 8
  %57 = load i32, ptr %7, align 4
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.lua_State, ptr %58, i32 0, i32 6
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds %union.StackValue, ptr %60, i64 20
  %62 = call ptr @prepCallInfo(ptr noundef %55, ptr noundef %56, i32 noundef %57, i32 noundef 2, ptr noundef %61)
  store ptr %62, ptr %10, align 8
  %63 = load ptr, ptr %5, align 8
  %64 = getelementptr inbounds %struct.lua_State, ptr %63, i32 0, i32 8
  store ptr %62, ptr %64, align 8
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.lua_State, ptr %65, i32 0, i32 23
  %67 = load volatile i32, ptr %66, align 8
  %68 = and i32 %67, 1
  %69 = icmp ne i32 %68, 0
  %70 = zext i1 %69 to i32
  %71 = sext i32 %70 to i64
  %72 = icmp ne i64 %71, 0
  br i1 %72, label %73, label %86

73:                                               ; preds = %54
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.lua_State, ptr %74, i32 0, i32 6
  %76 = load ptr, ptr %75, align 8
  %77 = load ptr, ptr %6, align 8
  %78 = ptrtoint ptr %76 to i64
  %79 = ptrtoint ptr %77 to i64
  %80 = sub i64 %78, %79
  %81 = sdiv exact i64 %80, 16
  %82 = trunc i64 %81 to i32
  %83 = sub nsw i32 %82, 1
  store i32 %83, ptr %12, align 4
  %84 = load ptr, ptr %5, align 8
  %85 = load i32, ptr %12, align 4
  call void @luaD_hook(ptr noundef %84, i32 noundef 0, i32 noundef -1, i32 noundef 1, i32 noundef %85)
  br label %86

86:                                               ; preds = %73, %54
  %87 = load ptr, ptr %8, align 8
  %88 = load ptr, ptr %5, align 8
  %89 = call i32 %87(ptr noundef %88)
  store i32 %89, ptr %9, align 4
  %90 = load ptr, ptr %5, align 8
  %91 = load ptr, ptr %5, align 8
  %92 = load ptr, ptr %10, align 8
  %93 = load i32, ptr %9, align 4
  call void @luaD_poscall(ptr noundef %91, ptr noundef %92, i32 noundef %93)
  %94 = load i32, ptr %9, align 4
  ret i32 %94
}

declare hidden void @luaC_step(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @tryfuncTM(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 9
  %14 = load ptr, ptr %13, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 6
  %17 = load ptr, ptr %16, align 8
  %18 = ptrtoint ptr %14 to i64
  %19 = ptrtoint ptr %17 to i64
  %20 = sub i64 %18, %19
  %21 = sdiv exact i64 %20, 16
  %22 = icmp sle i64 %21, 1
  %23 = zext i1 %22 to i32
  %24 = icmp ne i32 %23, 0
  %25 = zext i1 %24 to i32
  %26 = sext i32 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %28, label %52

28:                                               ; preds = %2
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 10
  %32 = load ptr, ptr %31, align 8
  %33 = ptrtoint ptr %29 to i64
  %34 = ptrtoint ptr %32 to i64
  %35 = sub i64 %33, %34
  store i64 %35, ptr %7, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 7
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %struct.global_State, ptr %38, i32 0, i32 3
  %40 = load i64, ptr %39, align 8
  %41 = icmp sgt i64 %40, 0
  br i1 %41, label %42, label %44

42:                                               ; preds = %28
  %43 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %43)
  br label %44

44:                                               ; preds = %42, %28
  %45 = load ptr, ptr %3, align 8
  %46 = call i32 @luaD_growstack(ptr noundef %45, i32 noundef 1, i32 noundef 1)
  %47 = load ptr, ptr %3, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 10
  %49 = load ptr, ptr %48, align 8
  %50 = load i64, ptr %7, align 8
  %51 = getelementptr inbounds i8, ptr %49, i64 %50
  store ptr %51, ptr %4, align 8
  br label %53

52:                                               ; preds = %2
  br label %53

53:                                               ; preds = %52, %44
  %54 = load ptr, ptr %3, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = call ptr @luaT_gettmbyobj(ptr noundef %54, ptr noundef %55, i32 noundef 23)
  store ptr %56, ptr %5, align 8
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.TValue, ptr %57, i32 0, i32 1
  %59 = load i8, ptr %58, align 8
  %60 = zext i8 %59 to i32
  %61 = and i32 %60, 15
  %62 = icmp eq i32 %61, 0
  %63 = zext i1 %62 to i32
  %64 = icmp ne i32 %63, 0
  %65 = zext i1 %64 to i32
  %66 = sext i32 %65 to i64
  %67 = icmp ne i64 %66, 0
  br i1 %67, label %68, label %71

68:                                               ; preds = %53
  %69 = load ptr, ptr %3, align 8
  %70 = load ptr, ptr %4, align 8
  call void @luaG_callerror(ptr noundef %69, ptr noundef %70) #9
  unreachable

71:                                               ; preds = %53
  %72 = load ptr, ptr %3, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 6
  %74 = load ptr, ptr %73, align 8
  store ptr %74, ptr %6, align 8
  br label %75

75:                                               ; preds = %93, %71
  %76 = load ptr, ptr %6, align 8
  %77 = load ptr, ptr %4, align 8
  %78 = icmp ugt ptr %76, %77
  br i1 %78, label %79, label %96

79:                                               ; preds = %75
  %80 = load ptr, ptr %6, align 8
  store ptr %80, ptr %8, align 8
  %81 = load ptr, ptr %6, align 8
  %82 = getelementptr inbounds %union.StackValue, ptr %81, i64 -1
  store ptr %82, ptr %9, align 8
  %83 = load ptr, ptr %8, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %9, align 8
  %86 = getelementptr inbounds %struct.TValue, ptr %85, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %84, ptr align 8 %86, i64 8, i1 false)
  %87 = load ptr, ptr %9, align 8
  %88 = getelementptr inbounds %struct.TValue, ptr %87, i32 0, i32 1
  %89 = load i8, ptr %88, align 8
  %90 = load ptr, ptr %8, align 8
  %91 = getelementptr inbounds %struct.TValue, ptr %90, i32 0, i32 1
  store i8 %89, ptr %91, align 8
  %92 = load ptr, ptr %3, align 8
  br label %93

93:                                               ; preds = %79
  %94 = load ptr, ptr %6, align 8
  %95 = getelementptr inbounds %union.StackValue, ptr %94, i32 -1
  store ptr %95, ptr %6, align 8
  br label %75, !llvm.loop !17

96:                                               ; preds = %75
  %97 = load ptr, ptr %3, align 8
  %98 = getelementptr inbounds %struct.lua_State, ptr %97, i32 0, i32 6
  %99 = load ptr, ptr %98, align 8
  %100 = getelementptr inbounds %union.StackValue, ptr %99, i32 1
  store ptr %100, ptr %98, align 8
  %101 = load ptr, ptr %4, align 8
  store ptr %101, ptr %10, align 8
  %102 = load ptr, ptr %5, align 8
  store ptr %102, ptr %11, align 8
  %103 = load ptr, ptr %10, align 8
  %104 = getelementptr inbounds %struct.TValue, ptr %103, i32 0, i32 0
  %105 = load ptr, ptr %11, align 8
  %106 = getelementptr inbounds %struct.TValue, ptr %105, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %104, ptr align 8 %106, i64 8, i1 false)
  %107 = load ptr, ptr %11, align 8
  %108 = getelementptr inbounds %struct.TValue, ptr %107, i32 0, i32 1
  %109 = load i8, ptr %108, align 8
  %110 = load ptr, ptr %10, align 8
  %111 = getelementptr inbounds %struct.TValue, ptr %110, i32 0, i32 1
  store i8 %109, ptr %111, align 8
  %112 = load ptr, ptr %3, align 8
  %113 = load ptr, ptr %4, align 8
  ret ptr %113
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaD_precall(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  br label %14

14:                                               ; preds = %139, %3
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 63
  switch i32 %19, label %139 [
    i32 38, label %20
    i32 22, label %30
    i32 6, label %38
  ]

20:                                               ; preds = %14
  %21 = load ptr, ptr %5, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = load i32, ptr %7, align 4
  %24 = load ptr, ptr %6, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %struct.CClosure, ptr %26, i32 0, i32 5
  %28 = load ptr, ptr %27, align 8
  %29 = call i32 @precallC(ptr noundef %21, ptr noundef %22, i32 noundef %23, ptr noundef %28)
  store ptr null, ptr %4, align 8
  br label %143

30:                                               ; preds = %14
  %31 = load ptr, ptr %5, align 8
  %32 = load ptr, ptr %6, align 8
  %33 = load i32, ptr %7, align 4
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 0
  %36 = load ptr, ptr %35, align 8
  %37 = call i32 @precallC(ptr noundef %31, ptr noundef %32, i32 noundef %33, ptr noundef %36)
  store ptr null, ptr %4, align 8
  br label %143

38:                                               ; preds = %14
  %39 = load ptr, ptr %6, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds %struct.LClosure, ptr %41, i32 0, i32 5
  %43 = load ptr, ptr %42, align 8
  store ptr %43, ptr %9, align 8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 6
  %46 = load ptr, ptr %45, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = ptrtoint ptr %46 to i64
  %49 = ptrtoint ptr %47 to i64
  %50 = sub i64 %48, %49
  %51 = sdiv exact i64 %50, 16
  %52 = trunc i64 %51 to i32
  %53 = sub nsw i32 %52, 1
  store i32 %53, ptr %10, align 4
  %54 = load ptr, ptr %9, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 3
  %56 = load i8, ptr %55, align 2
  %57 = zext i8 %56 to i32
  store i32 %57, ptr %11, align 4
  %58 = load ptr, ptr %9, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 5
  %60 = load i8, ptr %59, align 4
  %61 = zext i8 %60 to i32
  store i32 %61, ptr %12, align 4
  %62 = load ptr, ptr %5, align 8
  %63 = getelementptr inbounds %struct.lua_State, ptr %62, i32 0, i32 9
  %64 = load ptr, ptr %63, align 8
  %65 = load ptr, ptr %5, align 8
  %66 = getelementptr inbounds %struct.lua_State, ptr %65, i32 0, i32 6
  %67 = load ptr, ptr %66, align 8
  %68 = ptrtoint ptr %64 to i64
  %69 = ptrtoint ptr %67 to i64
  %70 = sub i64 %68, %69
  %71 = sdiv exact i64 %70, 16
  %72 = load i32, ptr %12, align 4
  %73 = sext i32 %72 to i64
  %74 = icmp sle i64 %71, %73
  %75 = zext i1 %74 to i32
  %76 = icmp ne i32 %75, 0
  %77 = zext i1 %76 to i32
  %78 = sext i32 %77 to i64
  %79 = icmp ne i64 %78, 0
  br i1 %79, label %80, label %105

80:                                               ; preds = %38
  %81 = load ptr, ptr %6, align 8
  %82 = load ptr, ptr %5, align 8
  %83 = getelementptr inbounds %struct.lua_State, ptr %82, i32 0, i32 10
  %84 = load ptr, ptr %83, align 8
  %85 = ptrtoint ptr %81 to i64
  %86 = ptrtoint ptr %84 to i64
  %87 = sub i64 %85, %86
  store i64 %87, ptr %13, align 8
  %88 = load ptr, ptr %5, align 8
  %89 = getelementptr inbounds %struct.lua_State, ptr %88, i32 0, i32 7
  %90 = load ptr, ptr %89, align 8
  %91 = getelementptr inbounds %struct.global_State, ptr %90, i32 0, i32 3
  %92 = load i64, ptr %91, align 8
  %93 = icmp sgt i64 %92, 0
  br i1 %93, label %94, label %96

94:                                               ; preds = %80
  %95 = load ptr, ptr %5, align 8
  call void @luaC_step(ptr noundef %95)
  br label %96

96:                                               ; preds = %94, %80
  %97 = load ptr, ptr %5, align 8
  %98 = load i32, ptr %12, align 4
  %99 = call i32 @luaD_growstack(ptr noundef %97, i32 noundef %98, i32 noundef 1)
  %100 = load ptr, ptr %5, align 8
  %101 = getelementptr inbounds %struct.lua_State, ptr %100, i32 0, i32 10
  %102 = load ptr, ptr %101, align 8
  %103 = load i64, ptr %13, align 8
  %104 = getelementptr inbounds i8, ptr %102, i64 %103
  store ptr %104, ptr %6, align 8
  br label %106

105:                                              ; preds = %38
  br label %106

106:                                              ; preds = %105, %96
  %107 = load ptr, ptr %5, align 8
  %108 = load ptr, ptr %6, align 8
  %109 = load i32, ptr %7, align 4
  %110 = load ptr, ptr %6, align 8
  %111 = getelementptr inbounds %union.StackValue, ptr %110, i64 1
  %112 = load i32, ptr %12, align 4
  %113 = sext i32 %112 to i64
  %114 = getelementptr inbounds %union.StackValue, ptr %111, i64 %113
  %115 = call ptr @prepCallInfo(ptr noundef %107, ptr noundef %108, i32 noundef %109, i32 noundef 0, ptr noundef %114)
  store ptr %115, ptr %8, align 8
  %116 = load ptr, ptr %5, align 8
  %117 = getelementptr inbounds %struct.lua_State, ptr %116, i32 0, i32 8
  store ptr %115, ptr %117, align 8
  %118 = load ptr, ptr %9, align 8
  %119 = getelementptr inbounds %struct.Proto, ptr %118, i32 0, i32 16
  %120 = load ptr, ptr %119, align 8
  %121 = load ptr, ptr %8, align 8
  %122 = getelementptr inbounds %struct.CallInfo, ptr %121, i32 0, i32 4
  %123 = getelementptr inbounds %struct.anon.0, ptr %122, i32 0, i32 0
  store ptr %120, ptr %123, align 8
  br label %124

124:                                              ; preds = %134, %106
  %125 = load i32, ptr %10, align 4
  %126 = load i32, ptr %11, align 4
  %127 = icmp slt i32 %125, %126
  br i1 %127, label %128, label %137

128:                                              ; preds = %124
  %129 = load ptr, ptr %5, align 8
  %130 = getelementptr inbounds %struct.lua_State, ptr %129, i32 0, i32 6
  %131 = load ptr, ptr %130, align 8
  %132 = getelementptr inbounds %union.StackValue, ptr %131, i32 1
  store ptr %132, ptr %130, align 8
  %133 = getelementptr inbounds %struct.TValue, ptr %131, i32 0, i32 1
  store i8 0, ptr %133, align 8
  br label %134

134:                                              ; preds = %128
  %135 = load i32, ptr %10, align 4
  %136 = add nsw i32 %135, 1
  store i32 %136, ptr %10, align 4
  br label %124, !llvm.loop !18

137:                                              ; preds = %124
  %138 = load ptr, ptr %8, align 8
  store ptr %138, ptr %4, align 8
  br label %143

139:                                              ; preds = %14
  %140 = load ptr, ptr %5, align 8
  %141 = load ptr, ptr %6, align 8
  %142 = call ptr @tryfuncTM(ptr noundef %140, ptr noundef %141)
  store ptr %142, ptr %6, align 8
  br label %14

143:                                              ; preds = %137, %30, %20
  %144 = load ptr, ptr %4, align 8
  ret ptr %144
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @prepCallInfo(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store ptr %4, ptr %10, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 8
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 3
  %16 = load ptr, ptr %15, align 8
  %17 = icmp ne ptr %16, null
  br i1 %17, label %18, label %24

18:                                               ; preds = %5
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds %struct.lua_State, ptr %19, i32 0, i32 8
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.CallInfo, ptr %21, i32 0, i32 3
  %23 = load ptr, ptr %22, align 8
  br label %27

24:                                               ; preds = %5
  %25 = load ptr, ptr %6, align 8
  %26 = call ptr @luaE_extendCI(ptr noundef %25)
  br label %27

27:                                               ; preds = %24, %18
  %28 = phi ptr [ %23, %18 ], [ %26, %24 ]
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.lua_State, ptr %29, i32 0, i32 8
  store ptr %28, ptr %30, align 8
  store ptr %28, ptr %11, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = load ptr, ptr %11, align 8
  %33 = getelementptr inbounds %struct.CallInfo, ptr %32, i32 0, i32 0
  store ptr %31, ptr %33, align 8
  %34 = load i32, ptr %8, align 4
  %35 = trunc i32 %34 to i16
  %36 = load ptr, ptr %11, align 8
  %37 = getelementptr inbounds %struct.CallInfo, ptr %36, i32 0, i32 6
  store i16 %35, ptr %37, align 4
  %38 = load i32, ptr %9, align 4
  %39 = trunc i32 %38 to i16
  %40 = load ptr, ptr %11, align 8
  %41 = getelementptr inbounds %struct.CallInfo, ptr %40, i32 0, i32 7
  store i16 %39, ptr %41, align 2
  %42 = load ptr, ptr %10, align 8
  %43 = load ptr, ptr %11, align 8
  %44 = getelementptr inbounds %struct.CallInfo, ptr %43, i32 0, i32 1
  store ptr %42, ptr %44, align 8
  %45 = load ptr, ptr %11, align 8
  ret ptr %45
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_call(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  call void @ccall(ptr noundef %7, ptr noundef %8, i32 noundef %9, i32 noundef 1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @ccall(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load i32, ptr %8, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 19
  %14 = load i32, ptr %13, align 8
  %15 = add i32 %14, %11
  store i32 %15, ptr %13, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 19
  %18 = load i32, ptr %17, align 8
  %19 = and i32 %18, 65535
  %20 = icmp uge i32 %19, 200
  %21 = zext i1 %20 to i32
  %22 = icmp ne i32 %21, 0
  %23 = zext i1 %22 to i32
  %24 = sext i32 %23 to i64
  %25 = icmp ne i64 %24, 0
  br i1 %25, label %26, label %61

26:                                               ; preds = %4
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.lua_State, ptr %27, i32 0, i32 9
  %29 = load ptr, ptr %28, align 8
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  %33 = ptrtoint ptr %29 to i64
  %34 = ptrtoint ptr %32 to i64
  %35 = sub i64 %33, %34
  %36 = sdiv exact i64 %35, 16
  %37 = icmp sle i64 %36, 0
  %38 = zext i1 %37 to i32
  %39 = icmp ne i32 %38, 0
  %40 = zext i1 %39 to i32
  %41 = sext i32 %40 to i64
  %42 = icmp ne i64 %41, 0
  br i1 %42, label %43, label %58

43:                                               ; preds = %26
  %44 = load ptr, ptr %6, align 8
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 10
  %47 = load ptr, ptr %46, align 8
  %48 = ptrtoint ptr %44 to i64
  %49 = ptrtoint ptr %47 to i64
  %50 = sub i64 %48, %49
  store i64 %50, ptr %10, align 8
  %51 = load ptr, ptr %5, align 8
  %52 = call i32 @luaD_growstack(ptr noundef %51, i32 noundef 0, i32 noundef 1)
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 10
  %55 = load ptr, ptr %54, align 8
  %56 = load i64, ptr %10, align 8
  %57 = getelementptr inbounds i8, ptr %55, i64 %56
  store ptr %57, ptr %6, align 8
  br label %59

58:                                               ; preds = %26
  br label %59

59:                                               ; preds = %58, %43
  %60 = load ptr, ptr %5, align 8
  call void @luaE_checkcstack(ptr noundef %60)
  br label %61

61:                                               ; preds = %59, %4
  %62 = load ptr, ptr %5, align 8
  %63 = load ptr, ptr %6, align 8
  %64 = load i32, ptr %7, align 4
  %65 = call ptr @luaD_precall(ptr noundef %62, ptr noundef %63, i32 noundef %64)
  store ptr %65, ptr %9, align 8
  %66 = icmp ne ptr %65, null
  br i1 %66, label %67, label %72

67:                                               ; preds = %61
  %68 = load ptr, ptr %9, align 8
  %69 = getelementptr inbounds %struct.CallInfo, ptr %68, i32 0, i32 7
  store i16 4, ptr %69, align 2
  %70 = load ptr, ptr %5, align 8
  %71 = load ptr, ptr %9, align 8
  call void @luaV_execute(ptr noundef %70, ptr noundef %71)
  br label %72

72:                                               ; preds = %67, %61
  %73 = load i32, ptr %8, align 4
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.lua_State, ptr %74, i32 0, i32 19
  %76 = load i32, ptr %75, align 8
  %77 = sub i32 %76, %73
  store i32 %77, ptr %75, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaD_callnoyield(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  call void @ccall(ptr noundef %7, ptr noundef %8, i32 noundef %9, i32 noundef 65537)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_resume(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store ptr %3, ptr %9, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 3
  %13 = load i8, ptr %12, align 2
  %14 = zext i8 %13 to i32
  %15 = icmp eq i32 %14, 0
  br i1 %15, label %16, label %50

16:                                               ; preds = %4
  %17 = load ptr, ptr %6, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 8
  %19 = load ptr, ptr %18, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 16
  %22 = icmp ne ptr %19, %21
  br i1 %22, label %23, label %27

23:                                               ; preds = %16
  %24 = load ptr, ptr %6, align 8
  %25 = load i32, ptr %8, align 4
  %26 = call i32 @resume_error(ptr noundef %24, ptr noundef @.str.2, i32 noundef %25)
  store i32 %26, ptr %5, align 4
  br label %149

27:                                               ; preds = %16
  %28 = load ptr, ptr %6, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 6
  %30 = load ptr, ptr %29, align 8
  %31 = load ptr, ptr %6, align 8
  %32 = getelementptr inbounds %struct.lua_State, ptr %31, i32 0, i32 8
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %struct.CallInfo, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %union.StackValue, ptr %35, i64 1
  %37 = ptrtoint ptr %30 to i64
  %38 = ptrtoint ptr %36 to i64
  %39 = sub i64 %37, %38
  %40 = sdiv exact i64 %39, 16
  %41 = load i32, ptr %8, align 4
  %42 = sext i32 %41 to i64
  %43 = icmp eq i64 %40, %42
  br i1 %43, label %44, label %48

44:                                               ; preds = %27
  %45 = load ptr, ptr %6, align 8
  %46 = load i32, ptr %8, align 4
  %47 = call i32 @resume_error(ptr noundef %45, ptr noundef @.str.3, i32 noundef %46)
  store i32 %47, ptr %5, align 4
  br label %149

48:                                               ; preds = %27
  br label %49

49:                                               ; preds = %48
  br label %61

50:                                               ; preds = %4
  %51 = load ptr, ptr %6, align 8
  %52 = getelementptr inbounds %struct.lua_State, ptr %51, i32 0, i32 3
  %53 = load i8, ptr %52, align 2
  %54 = zext i8 %53 to i32
  %55 = icmp ne i32 %54, 1
  br i1 %55, label %56, label %60

56:                                               ; preds = %50
  %57 = load ptr, ptr %6, align 8
  %58 = load i32, ptr %8, align 4
  %59 = call i32 @resume_error(ptr noundef %57, ptr noundef @.str.3, i32 noundef %58)
  store i32 %59, ptr %5, align 4
  br label %149

60:                                               ; preds = %50
  br label %61

61:                                               ; preds = %60, %49
  %62 = load ptr, ptr %7, align 8
  %63 = icmp ne ptr %62, null
  br i1 %63, label %64, label %69

64:                                               ; preds = %61
  %65 = load ptr, ptr %7, align 8
  %66 = getelementptr inbounds %struct.lua_State, ptr %65, i32 0, i32 19
  %67 = load i32, ptr %66, align 8
  %68 = and i32 %67, 65535
  br label %70

69:                                               ; preds = %61
  br label %70

70:                                               ; preds = %69, %64
  %71 = phi i32 [ %68, %64 ], [ 0, %69 ]
  %72 = load ptr, ptr %6, align 8
  %73 = getelementptr inbounds %struct.lua_State, ptr %72, i32 0, i32 19
  store i32 %71, ptr %73, align 8
  %74 = load ptr, ptr %6, align 8
  %75 = getelementptr inbounds %struct.lua_State, ptr %74, i32 0, i32 19
  %76 = load i32, ptr %75, align 8
  %77 = and i32 %76, 65535
  %78 = icmp uge i32 %77, 200
  br i1 %78, label %79, label %83

79:                                               ; preds = %70
  %80 = load ptr, ptr %6, align 8
  %81 = load i32, ptr %8, align 4
  %82 = call i32 @resume_error(ptr noundef %80, ptr noundef @.str.4, i32 noundef %81)
  store i32 %82, ptr %5, align 4
  br label %149

83:                                               ; preds = %70
  %84 = load ptr, ptr %6, align 8
  %85 = getelementptr inbounds %struct.lua_State, ptr %84, i32 0, i32 19
  %86 = load i32, ptr %85, align 8
  %87 = add i32 %86, 1
  store i32 %87, ptr %85, align 8
  %88 = load ptr, ptr %6, align 8
  %89 = load ptr, ptr %6, align 8
  %90 = load ptr, ptr %6, align 8
  %91 = call i32 @luaD_rawrunprotected(ptr noundef %90, ptr noundef @resume, ptr noundef %8)
  store i32 %91, ptr %10, align 4
  %92 = load ptr, ptr %6, align 8
  %93 = load i32, ptr %10, align 4
  %94 = call i32 @precover(ptr noundef %92, i32 noundef %93)
  store i32 %94, ptr %10, align 4
  %95 = load i32, ptr %10, align 4
  %96 = icmp sgt i32 %95, 1
  %97 = xor i1 %96, true
  %98 = zext i1 %97 to i32
  %99 = icmp ne i32 %98, 0
  %100 = zext i1 %99 to i32
  %101 = sext i32 %100 to i64
  %102 = icmp ne i64 %101, 0
  br i1 %102, label %103, label %104

103:                                              ; preds = %83
  br label %121

104:                                              ; preds = %83
  %105 = load i32, ptr %10, align 4
  %106 = trunc i32 %105 to i8
  %107 = load ptr, ptr %6, align 8
  %108 = getelementptr inbounds %struct.lua_State, ptr %107, i32 0, i32 3
  store i8 %106, ptr %108, align 2
  %109 = load ptr, ptr %6, align 8
  %110 = load i32, ptr %10, align 4
  %111 = load ptr, ptr %6, align 8
  %112 = getelementptr inbounds %struct.lua_State, ptr %111, i32 0, i32 6
  %113 = load ptr, ptr %112, align 8
  call void @luaD_seterrorobj(ptr noundef %109, i32 noundef %110, ptr noundef %113)
  %114 = load ptr, ptr %6, align 8
  %115 = getelementptr inbounds %struct.lua_State, ptr %114, i32 0, i32 6
  %116 = load ptr, ptr %115, align 8
  %117 = load ptr, ptr %6, align 8
  %118 = getelementptr inbounds %struct.lua_State, ptr %117, i32 0, i32 8
  %119 = load ptr, ptr %118, align 8
  %120 = getelementptr inbounds %struct.CallInfo, ptr %119, i32 0, i32 1
  store ptr %116, ptr %120, align 8
  br label %121

121:                                              ; preds = %104, %103
  %122 = load i32, ptr %10, align 4
  %123 = icmp eq i32 %122, 1
  br i1 %123, label %124, label %130

124:                                              ; preds = %121
  %125 = load ptr, ptr %6, align 8
  %126 = getelementptr inbounds %struct.lua_State, ptr %125, i32 0, i32 8
  %127 = load ptr, ptr %126, align 8
  %128 = getelementptr inbounds %struct.CallInfo, ptr %127, i32 0, i32 5
  %129 = load i32, ptr %128, align 8
  br label %145

130:                                              ; preds = %121
  %131 = load ptr, ptr %6, align 8
  %132 = getelementptr inbounds %struct.lua_State, ptr %131, i32 0, i32 6
  %133 = load ptr, ptr %132, align 8
  %134 = load ptr, ptr %6, align 8
  %135 = getelementptr inbounds %struct.lua_State, ptr %134, i32 0, i32 8
  %136 = load ptr, ptr %135, align 8
  %137 = getelementptr inbounds %struct.CallInfo, ptr %136, i32 0, i32 0
  %138 = load ptr, ptr %137, align 8
  %139 = getelementptr inbounds %union.StackValue, ptr %138, i64 1
  %140 = ptrtoint ptr %133 to i64
  %141 = ptrtoint ptr %139 to i64
  %142 = sub i64 %140, %141
  %143 = sdiv exact i64 %142, 16
  %144 = trunc i64 %143 to i32
  br label %145

145:                                              ; preds = %130, %124
  %146 = phi i32 [ %129, %124 ], [ %144, %130 ]
  %147 = load ptr, ptr %9, align 8
  store i32 %146, ptr %147, align 4
  %148 = load i32, ptr %10, align 4
  store i32 %148, ptr %5, align 4
  br label %149

149:                                              ; preds = %145, %79, %56, %44, %23
  %150 = load i32, ptr %5, align 4
  ret i32 %150
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @resume_error(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %9 = load i32, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 6
  %12 = load ptr, ptr %11, align 8
  %13 = sext i32 %9 to i64
  %14 = sub i64 0, %13
  %15 = getelementptr inbounds %union.StackValue, ptr %12, i64 %14
  store ptr %15, ptr %11, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.lua_State, ptr %16, i32 0, i32 6
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = call ptr @luaS_new(ptr noundef %19, ptr noundef %20)
  store ptr %21, ptr %8, align 8
  %22 = load ptr, ptr %8, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  store ptr %22, ptr %24, align 8
  %25 = load ptr, ptr %8, align 8
  %26 = getelementptr inbounds %struct.TString, ptr %25, i32 0, i32 1
  %27 = load i8, ptr %26, align 8
  %28 = zext i8 %27 to i32
  %29 = or i32 %28, 64
  %30 = trunc i32 %29 to i8
  %31 = load ptr, ptr %7, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 1
  store i8 %30, ptr %32, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 6
  %36 = load ptr, ptr %35, align 8
  %37 = getelementptr inbounds %union.StackValue, ptr %36, i32 1
  store ptr %37, ptr %35, align 8
  %38 = load ptr, ptr %4, align 8
  ret i32 2
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @resume(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %8, align 4
  store i32 %9, ptr %5, align 4
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 6
  %12 = load ptr, ptr %11, align 8
  %13 = load i32, ptr %5, align 4
  %14 = sext i32 %13 to i64
  %15 = sub i64 0, %14
  %16 = getelementptr inbounds %union.StackValue, ptr %12, i64 %15
  store ptr %16, ptr %6, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 8
  %19 = load ptr, ptr %18, align 8
  store ptr %19, ptr %7, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 3
  %22 = load i8, ptr %21, align 2
  %23 = zext i8 %22 to i32
  %24 = icmp eq i32 %23, 0
  br i1 %24, label %25, label %29

25:                                               ; preds = %2
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %union.StackValue, ptr %27, i64 -1
  call void @ccall(ptr noundef %26, ptr noundef %28, i32 noundef -1, i32 noundef 0)
  br label %73

29:                                               ; preds = %2
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.lua_State, ptr %30, i32 0, i32 3
  store i8 0, ptr %31, align 2
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.CallInfo, ptr %32, i32 0, i32 7
  %34 = load i16, ptr %33, align 2
  %35 = zext i16 %34 to i32
  %36 = and i32 %35, 2
  %37 = icmp ne i32 %36, 0
  br i1 %37, label %49, label %38

38:                                               ; preds = %29
  %39 = load ptr, ptr %7, align 8
  %40 = getelementptr inbounds %struct.CallInfo, ptr %39, i32 0, i32 4
  %41 = getelementptr inbounds %struct.anon.0, ptr %40, i32 0, i32 0
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds i32, ptr %42, i32 -1
  store ptr %43, ptr %41, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = load ptr, ptr %3, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 6
  store ptr %44, ptr %46, align 8
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %7, align 8
  call void @luaV_execute(ptr noundef %47, ptr noundef %48)
  br label %71

49:                                               ; preds = %29
  %50 = load ptr, ptr %7, align 8
  %51 = getelementptr inbounds %struct.CallInfo, ptr %50, i32 0, i32 4
  %52 = getelementptr inbounds %struct.anon.1, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %52, align 8
  %54 = icmp ne ptr %53, null
  br i1 %54, label %55, label %67

55:                                               ; preds = %49
  %56 = load ptr, ptr %7, align 8
  %57 = getelementptr inbounds %struct.CallInfo, ptr %56, i32 0, i32 4
  %58 = getelementptr inbounds %struct.anon.1, ptr %57, i32 0, i32 0
  %59 = load ptr, ptr %58, align 8
  %60 = load ptr, ptr %3, align 8
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.CallInfo, ptr %61, i32 0, i32 4
  %63 = getelementptr inbounds %struct.anon.1, ptr %62, i32 0, i32 2
  %64 = load i64, ptr %63, align 8
  %65 = call i32 %59(ptr noundef %60, i32 noundef 1, i64 noundef %64)
  store i32 %65, ptr %5, align 4
  %66 = load ptr, ptr %3, align 8
  br label %67

67:                                               ; preds = %55, %49
  %68 = load ptr, ptr %3, align 8
  %69 = load ptr, ptr %7, align 8
  %70 = load i32, ptr %5, align 4
  call void @luaD_poscall(ptr noundef %68, ptr noundef %69, i32 noundef %70)
  br label %71

71:                                               ; preds = %67, %38
  %72 = load ptr, ptr %3, align 8
  call void @unroll(ptr noundef %72, ptr noundef null)
  br label %73

73:                                               ; preds = %71, %25
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @precover(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  br label %6

6:                                                ; preds = %15, %2
  %7 = load i32, ptr %4, align 4
  %8 = icmp sgt i32 %7, 1
  br i1 %8, label %9, label %13

9:                                                ; preds = %6
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @findpcall(ptr noundef %10)
  store ptr %11, ptr %5, align 8
  %12 = icmp ne ptr %11, null
  br label %13

13:                                               ; preds = %9, %6
  %14 = phi i1 [ false, %6 ], [ %12, %9 ]
  br i1 %14, label %15, label %32

15:                                               ; preds = %13
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 8
  store ptr %16, ptr %18, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.CallInfo, ptr %19, i32 0, i32 7
  %21 = load i16, ptr %20, align 2
  %22 = zext i16 %21 to i32
  %23 = and i32 %22, -7169
  %24 = load i32, ptr %4, align 4
  %25 = shl i32 %24, 10
  %26 = or i32 %23, %25
  %27 = trunc i32 %26 to i16
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.CallInfo, ptr %28, i32 0, i32 7
  store i16 %27, ptr %29, align 2
  %30 = load ptr, ptr %3, align 8
  %31 = call i32 @luaD_rawrunprotected(ptr noundef %30, ptr noundef @unroll, ptr noundef null)
  store i32 %31, ptr %4, align 4
  br label %6, !llvm.loop !19

32:                                               ; preds = %13
  %33 = load i32, ptr %4, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_isyieldable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.lua_State, ptr %3, i32 0, i32 19
  %5 = load i32, ptr %4, align 8
  %6 = and i32 %5, -65536
  %7 = icmp eq i32 %6, 0
  %8 = zext i1 %7 to i32
  ret i32 %8
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @lua_yieldk(ptr noundef %0, i32 noundef %1, i64 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i64 %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 8
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %9, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 19
  %17 = load i32, ptr %16, align 8
  %18 = and i32 %17, -65536
  %19 = icmp eq i32 %18, 0
  %20 = xor i1 %19, true
  %21 = zext i1 %20 to i32
  %22 = icmp ne i32 %21, 0
  %23 = zext i1 %22 to i32
  %24 = sext i32 %23 to i64
  %25 = icmp ne i64 %24, 0
  br i1 %25, label %26, label %38

26:                                               ; preds = %4
  %27 = load ptr, ptr %5, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 7
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %struct.global_State, ptr %30, i32 0, i32 40
  %32 = load ptr, ptr %31, align 8
  %33 = icmp ne ptr %27, %32
  br i1 %33, label %34, label %36

34:                                               ; preds = %26
  %35 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %35, ptr noundef @.str.5) #9
  unreachable

36:                                               ; preds = %26
  %37 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %37, ptr noundef @.str.6) #9
  unreachable

38:                                               ; preds = %4
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 3
  store i8 1, ptr %40, align 2
  %41 = load i32, ptr %6, align 4
  %42 = load ptr, ptr %9, align 8
  %43 = getelementptr inbounds %struct.CallInfo, ptr %42, i32 0, i32 5
  store i32 %41, ptr %43, align 8
  %44 = load ptr, ptr %9, align 8
  %45 = getelementptr inbounds %struct.CallInfo, ptr %44, i32 0, i32 7
  %46 = load i16, ptr %45, align 2
  %47 = zext i16 %46 to i32
  %48 = and i32 %47, 2
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %53, label %50

50:                                               ; preds = %38
  %51 = load ptr, ptr %5, align 8
  %52 = load ptr, ptr %5, align 8
  br label %66

53:                                               ; preds = %38
  %54 = load ptr, ptr %8, align 8
  %55 = load ptr, ptr %9, align 8
  %56 = getelementptr inbounds %struct.CallInfo, ptr %55, i32 0, i32 4
  %57 = getelementptr inbounds %struct.anon.1, ptr %56, i32 0, i32 0
  store ptr %54, ptr %57, align 8
  %58 = icmp ne ptr %54, null
  br i1 %58, label %59, label %64

59:                                               ; preds = %53
  %60 = load i64, ptr %7, align 8
  %61 = load ptr, ptr %9, align 8
  %62 = getelementptr inbounds %struct.CallInfo, ptr %61, i32 0, i32 4
  %63 = getelementptr inbounds %struct.anon.1, ptr %62, i32 0, i32 2
  store i64 %60, ptr %63, align 8
  br label %64

64:                                               ; preds = %59, %53
  %65 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %65, i32 noundef 1) #9
  unreachable

66:                                               ; preds = %50
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_closeprotected(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i8, align 1
  %9 = alloca %struct.CloseP, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.lua_State, ptr %10, i32 0, i32 8
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 4
  %15 = load i8, ptr %14, align 1
  store i8 %15, ptr %8, align 1
  br label %16

16:                                               ; preds = %44, %3
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.lua_State, ptr %17, i32 0, i32 10
  %19 = load ptr, ptr %18, align 8
  %20 = load i64, ptr %5, align 8
  %21 = getelementptr inbounds i8, ptr %19, i64 %20
  %22 = getelementptr inbounds %struct.CloseP, ptr %9, i32 0, i32 0
  store ptr %21, ptr %22, align 8
  %23 = load i32, ptr %6, align 4
  %24 = getelementptr inbounds %struct.CloseP, ptr %9, i32 0, i32 1
  store i32 %23, ptr %24, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = call i32 @luaD_rawrunprotected(ptr noundef %25, ptr noundef @closepaux, ptr noundef %9)
  store i32 %26, ptr %6, align 4
  %27 = load i32, ptr %6, align 4
  %28 = icmp eq i32 %27, 0
  %29 = zext i1 %28 to i32
  %30 = icmp ne i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %34, label %37

34:                                               ; preds = %16
  %35 = getelementptr inbounds %struct.CloseP, ptr %9, i32 0, i32 1
  %36 = load i32, ptr %35, align 8
  ret i32 %36

37:                                               ; preds = %16
  %38 = load ptr, ptr %7, align 8
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 8
  store ptr %38, ptr %40, align 8
  %41 = load i8, ptr %8, align 1
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.lua_State, ptr %42, i32 0, i32 4
  store i8 %41, ptr %43, align 1
  br label %44

44:                                               ; preds = %37
  br label %16
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @closepaux(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  store ptr %6, ptr %5, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.CloseP, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.CloseP, ptr %11, i32 0, i32 1
  %13 = load i32, ptr %12, align 8
  %14 = call ptr @luaF_close(ptr noundef %7, ptr noundef %10, i32 noundef %13, i32 noundef 0)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_pcall(ptr noundef %0, ptr noundef %1, ptr noundef %2, i64 noundef %3, i64 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca i64, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i8, align 1
  %14 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  store i64 %4, ptr %10, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.lua_State, ptr %15, i32 0, i32 8
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %12, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 4
  %20 = load i8, ptr %19, align 1
  store i8 %20, ptr %13, align 1
  %21 = load ptr, ptr %6, align 8
  %22 = getelementptr inbounds %struct.lua_State, ptr %21, i32 0, i32 18
  %23 = load i64, ptr %22, align 8
  store i64 %23, ptr %14, align 8
  %24 = load i64, ptr %10, align 8
  %25 = load ptr, ptr %6, align 8
  %26 = getelementptr inbounds %struct.lua_State, ptr %25, i32 0, i32 18
  store i64 %24, ptr %26, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = load ptr, ptr %8, align 8
  %30 = call i32 @luaD_rawrunprotected(ptr noundef %27, ptr noundef %28, ptr noundef %29)
  store i32 %30, ptr %11, align 4
  %31 = load i32, ptr %11, align 4
  %32 = icmp ne i32 %31, 0
  %33 = zext i1 %32 to i32
  %34 = icmp ne i32 %33, 0
  %35 = zext i1 %34 to i32
  %36 = sext i32 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %38, label %57

38:                                               ; preds = %5
  %39 = load ptr, ptr %12, align 8
  %40 = load ptr, ptr %6, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 8
  store ptr %39, ptr %41, align 8
  %42 = load i8, ptr %13, align 1
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.lua_State, ptr %43, i32 0, i32 4
  store i8 %42, ptr %44, align 1
  %45 = load ptr, ptr %6, align 8
  %46 = load i64, ptr %9, align 8
  %47 = load i32, ptr %11, align 4
  %48 = call i32 @luaD_closeprotected(ptr noundef %45, i64 noundef %46, i32 noundef %47)
  store i32 %48, ptr %11, align 4
  %49 = load ptr, ptr %6, align 8
  %50 = load i32, ptr %11, align 4
  %51 = load ptr, ptr %6, align 8
  %52 = getelementptr inbounds %struct.lua_State, ptr %51, i32 0, i32 10
  %53 = load ptr, ptr %52, align 8
  %54 = load i64, ptr %9, align 8
  %55 = getelementptr inbounds i8, ptr %53, i64 %54
  call void @luaD_seterrorobj(ptr noundef %49, i32 noundef %50, ptr noundef %55)
  %56 = load ptr, ptr %6, align 8
  call void @luaD_shrinkstack(ptr noundef %56)
  br label %57

57:                                               ; preds = %38, %5
  %58 = load i64, ptr %14, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = getelementptr inbounds %struct.lua_State, ptr %59, i32 0, i32 18
  store i64 %58, ptr %60, align 8
  %61 = load i32, ptr %11, align 4
  ret i32 %61
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaD_protectedparser(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca %struct.SParser, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 19
  %13 = load i32, ptr %12, align 8
  %14 = add i32 %13, 65536
  store i32 %14, ptr %12, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 0
  store ptr %15, ptr %16, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 4
  store ptr %17, ptr %18, align 8
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 3
  store ptr %19, ptr %20, align 8
  %21 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %22 = getelementptr inbounds %struct.Dyndata, ptr %21, i32 0, i32 0
  %23 = getelementptr inbounds %struct.anon.8, ptr %22, i32 0, i32 0
  store ptr null, ptr %23, align 8
  %24 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %25 = getelementptr inbounds %struct.Dyndata, ptr %24, i32 0, i32 0
  %26 = getelementptr inbounds %struct.anon.8, ptr %25, i32 0, i32 2
  store i32 0, ptr %26, align 4
  %27 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %28 = getelementptr inbounds %struct.Dyndata, ptr %27, i32 0, i32 1
  %29 = getelementptr inbounds %struct.Labellist, ptr %28, i32 0, i32 0
  store ptr null, ptr %29, align 8
  %30 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %31 = getelementptr inbounds %struct.Dyndata, ptr %30, i32 0, i32 1
  %32 = getelementptr inbounds %struct.Labellist, ptr %31, i32 0, i32 2
  store i32 0, ptr %32, align 4
  %33 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %34 = getelementptr inbounds %struct.Dyndata, ptr %33, i32 0, i32 2
  %35 = getelementptr inbounds %struct.Labellist, ptr %34, i32 0, i32 0
  store ptr null, ptr %35, align 8
  %36 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %37 = getelementptr inbounds %struct.Dyndata, ptr %36, i32 0, i32 2
  %38 = getelementptr inbounds %struct.Labellist, ptr %37, i32 0, i32 2
  store i32 0, ptr %38, align 4
  %39 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %40 = getelementptr inbounds %struct.Mbuffer, ptr %39, i32 0, i32 0
  store ptr null, ptr %40, align 8
  %41 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %42 = getelementptr inbounds %struct.Mbuffer, ptr %41, i32 0, i32 2
  store i64 0, ptr %42, align 8
  %43 = load ptr, ptr %5, align 8
  %44 = load ptr, ptr %5, align 8
  %45 = getelementptr inbounds %struct.lua_State, ptr %44, i32 0, i32 6
  %46 = load ptr, ptr %45, align 8
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 10
  %49 = load ptr, ptr %48, align 8
  %50 = ptrtoint ptr %46 to i64
  %51 = ptrtoint ptr %49 to i64
  %52 = sub i64 %50, %51
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.lua_State, ptr %53, i32 0, i32 18
  %55 = load i64, ptr %54, align 8
  %56 = call i32 @luaD_pcall(ptr noundef %43, ptr noundef @f_parser, ptr noundef %9, i64 noundef %52, i64 noundef %55)
  store i32 %56, ptr %10, align 4
  %57 = load ptr, ptr %5, align 8
  %58 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %59 = getelementptr inbounds %struct.Mbuffer, ptr %58, i32 0, i32 0
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %62 = getelementptr inbounds %struct.Mbuffer, ptr %61, i32 0, i32 2
  %63 = load i64, ptr %62, align 8
  %64 = mul i64 %63, 1
  %65 = call ptr @luaM_saferealloc_(ptr noundef %57, ptr noundef %60, i64 noundef %64, i64 noundef 0)
  %66 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %67 = getelementptr inbounds %struct.Mbuffer, ptr %66, i32 0, i32 0
  store ptr %65, ptr %67, align 8
  %68 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 1
  %69 = getelementptr inbounds %struct.Mbuffer, ptr %68, i32 0, i32 2
  store i64 0, ptr %69, align 8
  %70 = load ptr, ptr %5, align 8
  %71 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %72 = getelementptr inbounds %struct.Dyndata, ptr %71, i32 0, i32 0
  %73 = getelementptr inbounds %struct.anon.8, ptr %72, i32 0, i32 0
  %74 = load ptr, ptr %73, align 8
  %75 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %76 = getelementptr inbounds %struct.Dyndata, ptr %75, i32 0, i32 0
  %77 = getelementptr inbounds %struct.anon.8, ptr %76, i32 0, i32 2
  %78 = load i32, ptr %77, align 4
  %79 = sext i32 %78 to i64
  %80 = mul i64 %79, 24
  call void @luaM_free_(ptr noundef %70, ptr noundef %74, i64 noundef %80)
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %83 = getelementptr inbounds %struct.Dyndata, ptr %82, i32 0, i32 1
  %84 = getelementptr inbounds %struct.Labellist, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %87 = getelementptr inbounds %struct.Dyndata, ptr %86, i32 0, i32 1
  %88 = getelementptr inbounds %struct.Labellist, ptr %87, i32 0, i32 2
  %89 = load i32, ptr %88, align 4
  %90 = sext i32 %89 to i64
  %91 = mul i64 %90, 24
  call void @luaM_free_(ptr noundef %81, ptr noundef %85, i64 noundef %91)
  %92 = load ptr, ptr %5, align 8
  %93 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %94 = getelementptr inbounds %struct.Dyndata, ptr %93, i32 0, i32 2
  %95 = getelementptr inbounds %struct.Labellist, ptr %94, i32 0, i32 0
  %96 = load ptr, ptr %95, align 8
  %97 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 2
  %98 = getelementptr inbounds %struct.Dyndata, ptr %97, i32 0, i32 2
  %99 = getelementptr inbounds %struct.Labellist, ptr %98, i32 0, i32 2
  %100 = load i32, ptr %99, align 4
  %101 = sext i32 %100 to i64
  %102 = mul i64 %101, 24
  call void @luaM_free_(ptr noundef %92, ptr noundef %96, i64 noundef %102)
  %103 = load ptr, ptr %5, align 8
  %104 = getelementptr inbounds %struct.lua_State, ptr %103, i32 0, i32 19
  %105 = load i32, ptr %104, align 8
  %106 = sub i32 %105, 65536
  store i32 %106, ptr %104, align 8
  %107 = load i32, ptr %10, align 4
  ret i32 %107
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @f_parser(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  store ptr %8, ptr %6, align 8
  %9 = load ptr, ptr %6, align 8
  %10 = getelementptr inbounds %struct.SParser, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.Zio, ptr %11, i32 0, i32 0
  %13 = load i64, ptr %12, align 8
  %14 = add i64 %13, -1
  store i64 %14, ptr %12, align 8
  %15 = icmp ugt i64 %13, 0
  br i1 %15, label %16, label %25

16:                                               ; preds = %2
  %17 = load ptr, ptr %6, align 8
  %18 = getelementptr inbounds %struct.SParser, ptr %17, i32 0, i32 0
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.Zio, ptr %19, i32 0, i32 1
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds i8, ptr %21, i32 1
  store ptr %22, ptr %20, align 8
  %23 = load i8, ptr %21, align 1
  %24 = zext i8 %23 to i32
  br label %30

25:                                               ; preds = %2
  %26 = load ptr, ptr %6, align 8
  %27 = getelementptr inbounds %struct.SParser, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = call i32 @luaZ_fill(ptr noundef %28)
  br label %30

30:                                               ; preds = %25, %16
  %31 = phi i32 [ %24, %16 ], [ %29, %25 ]
  store i32 %31, ptr %7, align 4
  %32 = load i32, ptr %7, align 4
  %33 = load i8, ptr @.str.7, align 1
  %34 = sext i8 %33 to i32
  %35 = icmp eq i32 %32, %34
  br i1 %35, label %36, label %49

36:                                               ; preds = %30
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %6, align 8
  %39 = getelementptr inbounds %struct.SParser, ptr %38, i32 0, i32 3
  %40 = load ptr, ptr %39, align 8
  call void @checkmode(ptr noundef %37, ptr noundef %40, ptr noundef @.str.8)
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %6, align 8
  %43 = getelementptr inbounds %struct.SParser, ptr %42, i32 0, i32 0
  %44 = load ptr, ptr %43, align 8
  %45 = load ptr, ptr %6, align 8
  %46 = getelementptr inbounds %struct.SParser, ptr %45, i32 0, i32 4
  %47 = load ptr, ptr %46, align 8
  %48 = call ptr @luaU_undump(ptr noundef %41, ptr noundef %44, ptr noundef %47)
  store ptr %48, ptr %5, align 8
  br label %67

49:                                               ; preds = %30
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %6, align 8
  %52 = getelementptr inbounds %struct.SParser, ptr %51, i32 0, i32 3
  %53 = load ptr, ptr %52, align 8
  call void @checkmode(ptr noundef %50, ptr noundef %53, ptr noundef @.str.9)
  %54 = load ptr, ptr %3, align 8
  %55 = load ptr, ptr %6, align 8
  %56 = getelementptr inbounds %struct.SParser, ptr %55, i32 0, i32 0
  %57 = load ptr, ptr %56, align 8
  %58 = load ptr, ptr %6, align 8
  %59 = getelementptr inbounds %struct.SParser, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %6, align 8
  %61 = getelementptr inbounds %struct.SParser, ptr %60, i32 0, i32 2
  %62 = load ptr, ptr %6, align 8
  %63 = getelementptr inbounds %struct.SParser, ptr %62, i32 0, i32 4
  %64 = load ptr, ptr %63, align 8
  %65 = load i32, ptr %7, align 4
  %66 = call ptr @luaY_parser(ptr noundef %54, ptr noundef %57, ptr noundef %59, ptr noundef %61, ptr noundef %64, i32 noundef %65)
  store ptr %66, ptr %5, align 8
  br label %67

67:                                               ; preds = %49, %36
  %68 = load ptr, ptr %3, align 8
  %69 = load ptr, ptr %5, align 8
  call void @luaF_initupvals(ptr noundef %68, ptr noundef %69)
  ret void
}

declare hidden ptr @luaM_saferealloc_(ptr noundef, ptr noundef, i64 noundef, i64 noundef) #1

declare hidden void @luaM_free_(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden ptr @luaF_close(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

declare hidden ptr @luaT_gettmbyobj(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noreturn
declare hidden void @luaG_callerror(ptr noundef, ptr noundef) #6

declare hidden ptr @luaE_extendCI(ptr noundef) #1

declare hidden void @luaE_checkcstack(ptr noundef) #1

declare hidden void @luaV_execute(ptr noundef, ptr noundef) #1

declare hidden ptr @luaS_new(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @unroll(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  br label %7

7:                                                ; preds = %28, %2
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 8
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 16
  %13 = icmp ne ptr %10, %12
  br i1 %13, label %14, label %29

14:                                               ; preds = %7
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.CallInfo, ptr %15, i32 0, i32 7
  %17 = load i16, ptr %16, align 2
  %18 = zext i16 %17 to i32
  %19 = and i32 %18, 2
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %24

21:                                               ; preds = %14
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %5, align 8
  call void @finishCcall(ptr noundef %22, ptr noundef %23)
  br label %28

24:                                               ; preds = %14
  %25 = load ptr, ptr %3, align 8
  call void @luaV_finishOp(ptr noundef %25)
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %5, align 8
  call void @luaV_execute(ptr noundef %26, ptr noundef %27)
  br label %28

28:                                               ; preds = %24, %21
  br label %7, !llvm.loop !20

29:                                               ; preds = %7
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @finishCcall(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.CallInfo, ptr %7, i32 0, i32 7
  %9 = load i16, ptr %8, align 2
  %10 = zext i16 %9 to i32
  %11 = and i32 %10, 512
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %13, label %17

13:                                               ; preds = %2
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.CallInfo, ptr %14, i32 0, i32 5
  %16 = load i32, ptr %15, align 8
  store i32 %16, ptr %5, align 4
  br label %59

17:                                               ; preds = %2
  store i32 1, ptr %6, align 4
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.CallInfo, ptr %18, i32 0, i32 7
  %20 = load i16, ptr %19, align 2
  %21 = zext i16 %20 to i32
  %22 = and i32 %21, 16
  %23 = icmp ne i32 %22, 0
  br i1 %23, label %24, label %28

24:                                               ; preds = %17
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = call i32 @finishpcallk(ptr noundef %25, ptr noundef %26)
  store i32 %27, ptr %6, align 4
  br label %28

28:                                               ; preds = %24, %17
  %29 = load ptr, ptr %3, align 8
  %30 = getelementptr inbounds %struct.lua_State, ptr %29, i32 0, i32 8
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.CallInfo, ptr %31, i32 0, i32 1
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.lua_State, ptr %34, i32 0, i32 6
  %36 = load ptr, ptr %35, align 8
  %37 = icmp ult ptr %33, %36
  br i1 %37, label %38, label %46

38:                                               ; preds = %28
  %39 = load ptr, ptr %3, align 8
  %40 = getelementptr inbounds %struct.lua_State, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = load ptr, ptr %3, align 8
  %43 = getelementptr inbounds %struct.lua_State, ptr %42, i32 0, i32 8
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.CallInfo, ptr %44, i32 0, i32 1
  store ptr %41, ptr %45, align 8
  br label %46

46:                                               ; preds = %38, %28
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.CallInfo, ptr %47, i32 0, i32 4
  %49 = getelementptr inbounds %struct.anon.1, ptr %48, i32 0, i32 0
  %50 = load ptr, ptr %49, align 8
  %51 = load ptr, ptr %3, align 8
  %52 = load i32, ptr %6, align 4
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.CallInfo, ptr %53, i32 0, i32 4
  %55 = getelementptr inbounds %struct.anon.1, ptr %54, i32 0, i32 2
  %56 = load i64, ptr %55, align 8
  %57 = call i32 %50(ptr noundef %51, i32 noundef %52, i64 noundef %56)
  store i32 %57, ptr %5, align 4
  %58 = load ptr, ptr %3, align 8
  br label %59

59:                                               ; preds = %46, %13
  %60 = load ptr, ptr %3, align 8
  %61 = load ptr, ptr %4, align 8
  %62 = load i32, ptr %5, align 4
  call void @luaD_poscall(ptr noundef %60, ptr noundef %61, i32 noundef %62)
  ret void
}

declare hidden void @luaV_finishOp(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @finishpcallk(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.CallInfo, ptr %7, i32 0, i32 7
  %9 = load i16, ptr %8, align 2
  %10 = zext i16 %9 to i32
  %11 = ashr i32 %10, 10
  %12 = and i32 %11, 7
  store i32 %12, ptr %5, align 4
  %13 = load i32, ptr %5, align 4
  %14 = icmp eq i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %21

20:                                               ; preds = %2
  store i32 1, ptr %5, align 4
  br label %55

21:                                               ; preds = %2
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.lua_State, ptr %22, i32 0, i32 10
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.CallInfo, ptr %25, i32 0, i32 5
  %27 = load i32, ptr %26, align 8
  %28 = sext i32 %27 to i64
  %29 = getelementptr inbounds i8, ptr %24, i64 %28
  store ptr %29, ptr %6, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.CallInfo, ptr %30, i32 0, i32 7
  %32 = load i16, ptr %31, align 2
  %33 = zext i16 %32 to i32
  %34 = and i32 %33, 1
  %35 = trunc i32 %34 to i8
  %36 = load ptr, ptr %3, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 4
  store i8 %35, ptr %37, align 1
  %38 = load ptr, ptr %3, align 8
  %39 = load ptr, ptr %6, align 8
  %40 = load i32, ptr %5, align 4
  %41 = call ptr @luaF_close(ptr noundef %38, ptr noundef %39, i32 noundef %40, i32 noundef 1)
  store ptr %41, ptr %6, align 8
  %42 = load ptr, ptr %3, align 8
  %43 = load i32, ptr %5, align 4
  %44 = load ptr, ptr %6, align 8
  call void @luaD_seterrorobj(ptr noundef %42, i32 noundef %43, ptr noundef %44)
  %45 = load ptr, ptr %3, align 8
  call void @luaD_shrinkstack(ptr noundef %45)
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.CallInfo, ptr %46, i32 0, i32 7
  %48 = load i16, ptr %47, align 2
  %49 = zext i16 %48 to i32
  %50 = and i32 %49, -7169
  %51 = or i32 %50, 0
  %52 = trunc i32 %51 to i16
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.CallInfo, ptr %53, i32 0, i32 7
  store i16 %52, ptr %54, align 2
  br label %55

55:                                               ; preds = %21, %20
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.CallInfo, ptr %56, i32 0, i32 7
  %58 = load i16, ptr %57, align 2
  %59 = zext i16 %58 to i32
  %60 = and i32 %59, -17
  %61 = trunc i32 %60 to i16
  store i16 %61, ptr %57, align 2
  %62 = load ptr, ptr %4, align 8
  %63 = getelementptr inbounds %struct.CallInfo, ptr %62, i32 0, i32 4
  %64 = getelementptr inbounds %struct.anon.1, ptr %63, i32 0, i32 1
  %65 = load i64, ptr %64, align 8
  %66 = load ptr, ptr %3, align 8
  %67 = getelementptr inbounds %struct.lua_State, ptr %66, i32 0, i32 18
  store i64 %65, ptr %67, align 8
  %68 = load i32, ptr %5, align 4
  ret i32 %68
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @findpcall(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.lua_State, ptr %5, i32 0, i32 8
  %7 = load ptr, ptr %6, align 8
  store ptr %7, ptr %4, align 8
  br label %8

8:                                                ; preds = %21, %1
  %9 = load ptr, ptr %4, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %25

11:                                               ; preds = %8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.CallInfo, ptr %12, i32 0, i32 7
  %14 = load i16, ptr %13, align 2
  %15 = zext i16 %14 to i32
  %16 = and i32 %15, 16
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %20

18:                                               ; preds = %11
  %19 = load ptr, ptr %4, align 8
  store ptr %19, ptr %2, align 8
  br label %26

20:                                               ; preds = %11
  br label %21

21:                                               ; preds = %20
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.CallInfo, ptr %22, i32 0, i32 2
  %24 = load ptr, ptr %23, align 8
  store ptr %24, ptr %4, align 8
  br label %8, !llvm.loop !21

25:                                               ; preds = %8
  store ptr null, ptr %2, align 8
  br label %26

26:                                               ; preds = %25, %18
  %27 = load ptr, ptr %2, align 8
  ret ptr %27
}

declare hidden i32 @luaZ_fill(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkmode(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = icmp ne ptr %7, null
  br i1 %8, label %9, label %23

9:                                                ; preds = %3
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds i8, ptr %11, i64 0
  %13 = load i8, ptr %12, align 1
  %14 = sext i8 %13 to i32
  %15 = call ptr @strchr(ptr noundef %10, i32 noundef %14) #11
  %16 = icmp eq ptr %15, null
  br i1 %16, label %17, label %23

17:                                               ; preds = %9
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %6, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %18, ptr noundef @.str.10, ptr noundef %19, ptr noundef %20)
  %22 = load ptr, ptr %4, align 8
  call void @luaD_throw(ptr noundef %22, i32 noundef 3) #9
  unreachable

23:                                               ; preds = %9, %3
  ret void
}

declare hidden ptr @luaU_undump(ptr noundef, ptr noundef, ptr noundef) #1

declare hidden ptr @luaY_parser(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

declare hidden void @luaF_initupvals(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #7

declare hidden ptr @luaO_pushfstring(ptr noundef, ptr noundef, ...) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noreturn nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nounwind returns_twice "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #7 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #8 = { noreturn nounwind }
attributes #9 = { noreturn }
attributes #10 = { nounwind returns_twice }
attributes #11 = { nounwind willreturn memory(read) }

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
