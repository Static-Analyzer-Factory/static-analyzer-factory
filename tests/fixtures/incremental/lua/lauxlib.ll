; ModuleID = 'lauxlib.c'
source_filename = "lauxlib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }
%struct.lua_Debug = type { i32, ptr, ptr, ptr, ptr, i64, i32, i32, i32, i8, i8, i8, i8, i16, i16, [60 x i8], ptr }
%struct.__va_list_tag = type { i32, i32, ptr, ptr }
%struct.LoadF = type { i32, ptr, [8192 x i8] }
%struct.LoadS = type { ptr, i64 }
%struct.UBox = type { ptr, i64 }

@.str = private unnamed_addr constant [17 x i8] c"stack traceback:\00", align 1
@.str.1 = private unnamed_addr constant [27 x i8] c"\0A\09...\09(skipping %d levels)\00", align 1
@.str.2 = private unnamed_addr constant [5 x i8] c"Slnt\00", align 1
@.str.3 = private unnamed_addr constant [10 x i8] c"\0A\09%s: in \00", align 1
@.str.4 = private unnamed_addr constant [13 x i8] c"\0A\09%s:%d: in \00", align 1
@.str.5 = private unnamed_addr constant [21 x i8] c"\0A\09(...tail calls...)\00", align 1
@.str.6 = private unnamed_addr constant [22 x i8] c"bad argument #%d (%s)\00", align 1
@.str.7 = private unnamed_addr constant [2 x i8] c"n\00", align 1
@.str.8 = private unnamed_addr constant [7 x i8] c"method\00", align 1
@.str.9 = private unnamed_addr constant [30 x i8] c"calling '%s' on bad self (%s)\00", align 1
@.str.10 = private unnamed_addr constant [2 x i8] c"?\00", align 1
@.str.11 = private unnamed_addr constant [30 x i8] c"bad argument #%d to '%s' (%s)\00", align 1
@.str.12 = private unnamed_addr constant [7 x i8] c"__name\00", align 1
@.str.13 = private unnamed_addr constant [15 x i8] c"light userdata\00", align 1
@.str.14 = private unnamed_addr constant [20 x i8] c"%s expected, got %s\00", align 1
@.str.15 = private unnamed_addr constant [3 x i8] c"Sl\00", align 1
@.str.16 = private unnamed_addr constant [8 x i8] c"%s:%d: \00", align 1
@.str.17 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.18 = private unnamed_addr constant [16 x i8] c"(no extra info)\00", align 1
@.str.19 = private unnamed_addr constant [7 x i8] c"%s: %s\00", align 1
@.str.20 = private unnamed_addr constant [5 x i8] c"exit\00", align 1
@.str.21 = private unnamed_addr constant [20 x i8] c"invalid option '%s'\00", align 1
@.str.22 = private unnamed_addr constant [20 x i8] c"stack overflow (%s)\00", align 1
@.str.23 = private unnamed_addr constant [15 x i8] c"stack overflow\00", align 1
@.str.24 = private unnamed_addr constant [15 x i8] c"value expected\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"=stdin\00", align 1
@stdin = external global ptr, align 8
@.str.26 = private unnamed_addr constant [4 x i8] c"@%s\00", align 1
@.str.27 = private unnamed_addr constant [2 x i8] c"r\00", align 1
@.str.28 = private unnamed_addr constant [5 x i8] c"open\00", align 1
@.str.29 = private unnamed_addr constant [5 x i8] c"\1BLua\00", align 1
@.str.30 = private unnamed_addr constant [3 x i8] c"rb\00", align 1
@.str.31 = private unnamed_addr constant [7 x i8] c"reopen\00", align 1
@.str.32 = private unnamed_addr constant [5 x i8] c"read\00", align 1
@.str.33 = private unnamed_addr constant [32 x i8] c"object length is not an integer\00", align 1
@.str.34 = private unnamed_addr constant [11 x i8] c"__tostring\00", align 1
@.str.35 = private unnamed_addr constant [34 x i8] c"'__tostring' must return a string\00", align 1
@.str.36 = private unnamed_addr constant [3 x i8] c"%I\00", align 1
@.str.37 = private unnamed_addr constant [3 x i8] c"%f\00", align 1
@.str.38 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.39 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.40 = private unnamed_addr constant [4 x i8] c"nil\00", align 1
@.str.41 = private unnamed_addr constant [7 x i8] c"%s: %p\00", align 1
@.str.42 = private unnamed_addr constant [18 x i8] c"too many upvalues\00", align 1
@.str.43 = private unnamed_addr constant [8 x i8] c"_LOADED\00", align 1
@.str.44 = private unnamed_addr constant [49 x i8] c"core and library have incompatible numeric types\00", align 1
@.str.45 = private unnamed_addr constant [54 x i8] c"version mismatch: app. needs %f, Lua core provides %f\00", align 1
@.str.46 = private unnamed_addr constant [14 x i8] c"function '%s'\00", align 1
@.str.47 = private unnamed_addr constant [8 x i8] c"%s '%s'\00", align 1
@.str.48 = private unnamed_addr constant [11 x i8] c"main chunk\00", align 1
@.str.49 = private unnamed_addr constant [17 x i8] c"function <%s:%d>\00", align 1
@.str.50 = private unnamed_addr constant [2 x i8] c"f\00", align 1
@.str.51 = private unnamed_addr constant [17 x i8] c"not enough stack\00", align 1
@.str.52 = private unnamed_addr constant [4 x i8] c"_G.\00", align 1
@.str.53 = private unnamed_addr constant [2 x i8] c".\00", align 1
@.str.54 = private unnamed_addr constant [37 x i8] c"number has no integer representation\00", align 1
@.str.55 = private unnamed_addr constant [17 x i8] c"buffer too large\00", align 1
@.str.56 = private unnamed_addr constant [18 x i8] c"not enough memory\00", align 1
@.str.57 = private unnamed_addr constant [7 x i8] c"_UBOX*\00", align 1
@boxmt = internal constant [3 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.58, ptr @boxgc }, %struct.luaL_Reg { ptr @.str.59, ptr @boxgc }, %struct.luaL_Reg zeroinitializer], align 16
@.str.58 = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@.str.59 = private unnamed_addr constant [8 x i8] c"__close\00", align 1
@.str.60 = private unnamed_addr constant [17 x i8] c"cannot %s %s: %s\00", align 1
@.str.61 = private unnamed_addr constant [13 x i8] c"cannot %s %s\00", align 1
@.str.62 = private unnamed_addr constant [29 x i8] c"error object is not a string\00", align 1
@stderr = external global ptr, align 8
@.str.63 = private unnamed_addr constant [50 x i8] c"PANIC: unprotected error in call to Lua API (%s)\0A\00", align 1
@.str.64 = private unnamed_addr constant [4 x i8] c"off\00", align 1
@.str.65 = private unnamed_addr constant [3 x i8] c"on\00", align 1
@.str.66 = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.str.67 = private unnamed_addr constant [14 x i8] c"Lua warning: \00", align 1
@.str.68 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_traceback(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca %struct.luaL_Buffer, align 8
  %10 = alloca %struct.lua_Debug, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %14 = load ptr, ptr %6, align 8
  %15 = call i32 @lastlevel(ptr noundef %14)
  store i32 %15, ptr %11, align 4
  %16 = load i32, ptr %11, align 4
  %17 = load i32, ptr %8, align 4
  %18 = sub nsw i32 %16, %17
  %19 = icmp sgt i32 %18, 21
  %20 = zext i1 %19 to i64
  %21 = select i1 %19, i32 10, i32 -1
  store i32 %21, ptr %12, align 4
  %22 = load ptr, ptr %5, align 8
  call void @luaL_buffinit(ptr noundef %22, ptr noundef %9)
  %23 = load ptr, ptr %7, align 8
  %24 = icmp ne ptr %23, null
  br i1 %24, label %25, label %44

25:                                               ; preds = %4
  %26 = load ptr, ptr %7, align 8
  call void @luaL_addstring(ptr noundef %9, ptr noundef %26)
  %27 = getelementptr inbounds %struct.luaL_Buffer, ptr %9, i32 0, i32 2
  %28 = load i64, ptr %27, align 8
  %29 = getelementptr inbounds %struct.luaL_Buffer, ptr %9, i32 0, i32 1
  %30 = load i64, ptr %29, align 8
  %31 = icmp ult i64 %28, %30
  br i1 %31, label %35, label %32

32:                                               ; preds = %25
  %33 = call ptr @luaL_prepbuffsize(ptr noundef %9, i64 noundef 1)
  %34 = icmp ne ptr %33, null
  br label %35

35:                                               ; preds = %32, %25
  %36 = phi i1 [ true, %25 ], [ %34, %32 ]
  %37 = zext i1 %36 to i32
  %38 = getelementptr inbounds %struct.luaL_Buffer, ptr %9, i32 0, i32 0
  %39 = load ptr, ptr %38, align 8
  %40 = getelementptr inbounds %struct.luaL_Buffer, ptr %9, i32 0, i32 2
  %41 = load i64, ptr %40, align 8
  %42 = add i64 %41, 1
  store i64 %42, ptr %40, align 8
  %43 = getelementptr inbounds i8, ptr %39, i64 %41
  store i8 10, ptr %43, align 1
  br label %44

44:                                               ; preds = %35, %4
  call void @luaL_addstring(ptr noundef %9, ptr noundef @.str)
  br label %45

45:                                               ; preds = %92, %44
  %46 = load ptr, ptr %6, align 8
  %47 = load i32, ptr %8, align 4
  %48 = add nsw i32 %47, 1
  store i32 %48, ptr %8, align 4
  %49 = call i32 @lua_getstack(ptr noundef %46, i32 noundef %47, ptr noundef %10)
  %50 = icmp ne i32 %49, 0
  br i1 %50, label %51, label %93

51:                                               ; preds = %45
  %52 = load i32, ptr %12, align 4
  %53 = add nsw i32 %52, -1
  store i32 %53, ptr %12, align 4
  %54 = icmp eq i32 %52, 0
  br i1 %54, label %55, label %67

55:                                               ; preds = %51
  %56 = load i32, ptr %11, align 4
  %57 = load i32, ptr %8, align 4
  %58 = sub nsw i32 %56, %57
  %59 = sub nsw i32 %58, 11
  %60 = add nsw i32 %59, 1
  store i32 %60, ptr %13, align 4
  %61 = load ptr, ptr %5, align 8
  %62 = load i32, ptr %13, align 4
  %63 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %61, ptr noundef @.str.1, i32 noundef %62)
  call void @luaL_addvalue(ptr noundef %9)
  %64 = load i32, ptr %13, align 4
  %65 = load i32, ptr %8, align 4
  %66 = add nsw i32 %65, %64
  store i32 %66, ptr %8, align 4
  br label %92

67:                                               ; preds = %51
  %68 = load ptr, ptr %6, align 8
  %69 = call i32 @lua_getinfo(ptr noundef %68, ptr noundef @.str.2, ptr noundef %10)
  %70 = getelementptr inbounds %struct.lua_Debug, ptr %10, i32 0, i32 6
  %71 = load i32, ptr %70, align 8
  %72 = icmp sle i32 %71, 0
  br i1 %72, label %73, label %78

73:                                               ; preds = %67
  %74 = load ptr, ptr %5, align 8
  %75 = getelementptr inbounds %struct.lua_Debug, ptr %10, i32 0, i32 15
  %76 = getelementptr inbounds [60 x i8], ptr %75, i64 0, i64 0
  %77 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %74, ptr noundef @.str.3, ptr noundef %76)
  br label %85

78:                                               ; preds = %67
  %79 = load ptr, ptr %5, align 8
  %80 = getelementptr inbounds %struct.lua_Debug, ptr %10, i32 0, i32 15
  %81 = getelementptr inbounds [60 x i8], ptr %80, i64 0, i64 0
  %82 = getelementptr inbounds %struct.lua_Debug, ptr %10, i32 0, i32 6
  %83 = load i32, ptr %82, align 8
  %84 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %79, ptr noundef @.str.4, ptr noundef %81, i32 noundef %83)
  br label %85

85:                                               ; preds = %78, %73
  call void @luaL_addvalue(ptr noundef %9)
  %86 = load ptr, ptr %5, align 8
  call void @pushfuncname(ptr noundef %86, ptr noundef %10)
  call void @luaL_addvalue(ptr noundef %9)
  %87 = getelementptr inbounds %struct.lua_Debug, ptr %10, i32 0, i32 12
  %88 = load i8, ptr %87, align 1
  %89 = icmp ne i8 %88, 0
  br i1 %89, label %90, label %91

90:                                               ; preds = %85
  call void @luaL_addstring(ptr noundef %9, ptr noundef @.str.5)
  br label %91

91:                                               ; preds = %90, %85
  br label %92

92:                                               ; preds = %91, %55
  br label %45, !llvm.loop !6

93:                                               ; preds = %45
  call void @luaL_pushresult(ptr noundef %9)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @lastlevel(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.lua_Debug, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 1, ptr %4, align 4
  store i32 1, ptr %5, align 4
  br label %7

7:                                                ; preds = %12, %1
  %8 = load ptr, ptr %2, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call i32 @lua_getstack(ptr noundef %8, i32 noundef %9, ptr noundef %3)
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %12, label %16

12:                                               ; preds = %7
  %13 = load i32, ptr %5, align 4
  store i32 %13, ptr %4, align 4
  %14 = load i32, ptr %5, align 4
  %15 = mul nsw i32 %14, 2
  store i32 %15, ptr %5, align 4
  br label %7, !llvm.loop !8

16:                                               ; preds = %7
  br label %17

17:                                               ; preds = %35, %16
  %18 = load i32, ptr %4, align 4
  %19 = load i32, ptr %5, align 4
  %20 = icmp slt i32 %18, %19
  br i1 %20, label %21, label %36

21:                                               ; preds = %17
  %22 = load i32, ptr %4, align 4
  %23 = load i32, ptr %5, align 4
  %24 = add nsw i32 %22, %23
  %25 = sdiv i32 %24, 2
  store i32 %25, ptr %6, align 4
  %26 = load ptr, ptr %2, align 8
  %27 = load i32, ptr %6, align 4
  %28 = call i32 @lua_getstack(ptr noundef %26, i32 noundef %27, ptr noundef %3)
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %30, label %33

30:                                               ; preds = %21
  %31 = load i32, ptr %6, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %4, align 4
  br label %35

33:                                               ; preds = %21
  %34 = load i32, ptr %6, align 4
  store i32 %34, ptr %5, align 4
  br label %35

35:                                               ; preds = %33, %30
  br label %17, !llvm.loop !9

36:                                               ; preds = %17
  %37 = load i32, ptr %5, align 4
  %38 = sub nsw i32 %37, 1
  ret i32 %38
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_buffinit(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.luaL_Buffer, ptr %6, i32 0, i32 3
  store ptr %5, ptr %7, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.luaL_Buffer, ptr %8, i32 0, i32 4
  %10 = getelementptr inbounds [1024 x i8], ptr %9, i64 0, i64 0
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 0
  store ptr %10, ptr %12, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 2
  store i64 0, ptr %14, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.luaL_Buffer, ptr %15, i32 0, i32 1
  store i64 1024, ptr %16, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  call void @lua_pushlightuserdata(ptr noundef %17, ptr noundef %18)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_addstring(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i64 @strlen(ptr noundef %7) #8
  call void @luaL_addlstring(ptr noundef %5, ptr noundef %6, i64 noundef %8)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_prepbuffsize(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load i64, ptr %4, align 8
  %7 = call ptr @prepbuffsize(ptr noundef %5, i64 noundef %6, i32 noundef -1)
  ret ptr %7
}

declare i32 @lua_getstack(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_addvalue(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 3
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @lua_tolstring(ptr noundef %10, i32 noundef -1, ptr noundef %4)
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %2, align 8
  %13 = load i64, ptr %4, align 8
  %14 = call ptr @prepbuffsize(ptr noundef %12, i64 noundef %13, i32 noundef -2)
  store ptr %14, ptr %6, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = load i64, ptr %4, align 8
  %18 = mul i64 %17, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %15, ptr align 1 %16, i64 %18, i1 false)
  %19 = load i64, ptr %4, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds %struct.luaL_Buffer, ptr %20, i32 0, i32 2
  %22 = load i64, ptr %21, align 8
  %23 = add i64 %22, %19
  store i64 %23, ptr %21, align 8
  %24 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %24, i32 noundef -2)
  ret void
}

declare i32 @lua_getinfo(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pushfuncname(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = call i32 @pushglobalfuncname(ptr noundef %5, ptr noundef %6)
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %16

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = call ptr @lua_tolstring(ptr noundef %11, i32 noundef -1, ptr noundef null)
  %13 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %10, ptr noundef @.str.46, ptr noundef %12)
  %14 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %14, i32 noundef -2, i32 noundef -1)
  %15 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %15, i32 noundef -2)
  br label %64

16:                                               ; preds = %2
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.lua_Debug, ptr %17, i32 0, i32 2
  %19 = load ptr, ptr %18, align 8
  %20 = load i8, ptr %19, align 1
  %21 = sext i8 %20 to i32
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %32

23:                                               ; preds = %16
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.lua_Debug, ptr %25, i32 0, i32 2
  %27 = load ptr, ptr %26, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.lua_Debug, ptr %28, i32 0, i32 1
  %30 = load ptr, ptr %29, align 8
  %31 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %24, ptr noundef @.str.47, ptr noundef %27, ptr noundef %30)
  br label %63

32:                                               ; preds = %16
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.lua_Debug, ptr %33, i32 0, i32 3
  %35 = load ptr, ptr %34, align 8
  %36 = load i8, ptr %35, align 1
  %37 = sext i8 %36 to i32
  %38 = icmp eq i32 %37, 109
  br i1 %38, label %39, label %42

39:                                               ; preds = %32
  %40 = load ptr, ptr %3, align 8
  %41 = call ptr @lua_pushstring(ptr noundef %40, ptr noundef @.str.48)
  br label %62

42:                                               ; preds = %32
  %43 = load ptr, ptr %4, align 8
  %44 = getelementptr inbounds %struct.lua_Debug, ptr %43, i32 0, i32 3
  %45 = load ptr, ptr %44, align 8
  %46 = load i8, ptr %45, align 1
  %47 = sext i8 %46 to i32
  %48 = icmp ne i32 %47, 67
  br i1 %48, label %49, label %58

49:                                               ; preds = %42
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.lua_Debug, ptr %51, i32 0, i32 15
  %53 = getelementptr inbounds [60 x i8], ptr %52, i64 0, i64 0
  %54 = load ptr, ptr %4, align 8
  %55 = getelementptr inbounds %struct.lua_Debug, ptr %54, i32 0, i32 7
  %56 = load i32, ptr %55, align 4
  %57 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %50, ptr noundef @.str.49, ptr noundef %53, i32 noundef %56)
  br label %61

58:                                               ; preds = %42
  %59 = load ptr, ptr %3, align 8
  %60 = call ptr @lua_pushstring(ptr noundef %59, ptr noundef @.str.10)
  br label %61

61:                                               ; preds = %58, %49
  br label %62

62:                                               ; preds = %61, %39
  br label %63

63:                                               ; preds = %62, %23
  br label %64

64:                                               ; preds = %63, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_pushresult(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.luaL_Buffer, ptr %4, i32 0, i32 3
  %6 = load ptr, ptr %5, align 8
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.luaL_Buffer, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 2
  %13 = load i64, ptr %12, align 8
  %14 = call ptr @lua_pushlstring(ptr noundef %7, ptr noundef %10, i64 noundef %13)
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.luaL_Buffer, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.luaL_Buffer, ptr %18, i32 0, i32 4
  %20 = getelementptr inbounds [1024 x i8], ptr %19, i64 0, i64 0
  %21 = icmp ne ptr %17, %20
  br i1 %21, label %22, label %24

22:                                               ; preds = %1
  %23 = load ptr, ptr %3, align 8
  call void @lua_closeslot(ptr noundef %23, i32 noundef -2)
  br label %24

24:                                               ; preds = %22, %1
  %25 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %25, i32 noundef -2, i32 noundef -1)
  %26 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %26, i32 noundef -2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_argerror(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca %struct.lua_Debug, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = call i32 @lua_getstack(ptr noundef %9, i32 noundef 0, ptr noundef %8)
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %17, label %12

12:                                               ; preds = %3
  %13 = load ptr, ptr %5, align 8
  %14 = load i32, ptr %6, align 4
  %15 = load ptr, ptr %7, align 8
  %16 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %13, ptr noundef @.str.6, i32 noundef %14, ptr noundef %15)
  store i32 %16, ptr %4, align 4
  br label %58

17:                                               ; preds = %3
  %18 = load ptr, ptr %5, align 8
  %19 = call i32 @lua_getinfo(ptr noundef %18, ptr noundef @.str.7, ptr noundef %8)
  %20 = getelementptr inbounds %struct.lua_Debug, ptr %8, i32 0, i32 2
  %21 = load ptr, ptr %20, align 8
  %22 = call i32 @strcmp(ptr noundef %21, ptr noundef @.str.8) #8
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %36

24:                                               ; preds = %17
  %25 = load i32, ptr %6, align 4
  %26 = add nsw i32 %25, -1
  store i32 %26, ptr %6, align 4
  %27 = load i32, ptr %6, align 4
  %28 = icmp eq i32 %27, 0
  br i1 %28, label %29, label %35

29:                                               ; preds = %24
  %30 = load ptr, ptr %5, align 8
  %31 = getelementptr inbounds %struct.lua_Debug, ptr %8, i32 0, i32 1
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %7, align 8
  %34 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %30, ptr noundef @.str.9, ptr noundef %32, ptr noundef %33)
  store i32 %34, ptr %4, align 4
  br label %58

35:                                               ; preds = %24
  br label %36

36:                                               ; preds = %35, %17
  %37 = getelementptr inbounds %struct.lua_Debug, ptr %8, i32 0, i32 1
  %38 = load ptr, ptr %37, align 8
  %39 = icmp eq ptr %38, null
  br i1 %39, label %40, label %51

40:                                               ; preds = %36
  %41 = load ptr, ptr %5, align 8
  %42 = call i32 @pushglobalfuncname(ptr noundef %41, ptr noundef %8)
  %43 = icmp ne i32 %42, 0
  br i1 %43, label %44, label %47

44:                                               ; preds = %40
  %45 = load ptr, ptr %5, align 8
  %46 = call ptr @lua_tolstring(ptr noundef %45, i32 noundef -1, ptr noundef null)
  br label %48

47:                                               ; preds = %40
  br label %48

48:                                               ; preds = %47, %44
  %49 = phi ptr [ %46, %44 ], [ @.str.10, %47 ]
  %50 = getelementptr inbounds %struct.lua_Debug, ptr %8, i32 0, i32 1
  store ptr %49, ptr %50, align 8
  br label %51

51:                                               ; preds = %48, %36
  %52 = load ptr, ptr %5, align 8
  %53 = load i32, ptr %6, align 4
  %54 = getelementptr inbounds %struct.lua_Debug, ptr %8, i32 0, i32 1
  %55 = load ptr, ptr %54, align 8
  %56 = load ptr, ptr %7, align 8
  %57 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %52, ptr noundef @.str.11, i32 noundef %53, ptr noundef %55, ptr noundef %56)
  store i32 %57, ptr %4, align 4
  br label %58

58:                                               ; preds = %51, %29, %12
  %59 = load i32, ptr %4, align 4
  ret i32 %59
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_error(ptr noundef %0, ptr noundef %1, ...) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca [1 x %struct.__va_list_tag], align 16
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %5, i64 0, i64 0
  call void @llvm.va_start(ptr %6)
  %7 = load ptr, ptr %3, align 8
  call void @luaL_where(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %5, i64 0, i64 0
  %11 = call ptr @lua_pushvfstring(ptr noundef %8, ptr noundef %9, ptr noundef %10)
  %12 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %5, i64 0, i64 0
  call void @llvm.va_end(ptr %12)
  %13 = load ptr, ptr %3, align 8
  call void @lua_concat(ptr noundef %13, i32 noundef 2)
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @lua_error(ptr noundef %14)
  ret i32 %15
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pushglobalfuncname(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call i32 @lua_gettop(ptr noundef %8)
  store i32 %9, ptr %6, align 4
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = call i32 @lua_getinfo(ptr noundef %10, ptr noundef @.str.50, ptr noundef %11)
  %13 = load ptr, ptr %4, align 8
  %14 = call i32 @lua_getfield(ptr noundef %13, i32 noundef -1001000, ptr noundef @.str.43)
  %15 = load ptr, ptr %4, align 8
  call void @luaL_checkstack(ptr noundef %15, i32 noundef 6, ptr noundef @.str.51)
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %6, align 4
  %18 = add nsw i32 %17, 1
  %19 = call i32 @findfield(ptr noundef %16, i32 noundef %18, i32 noundef 2)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %41

21:                                               ; preds = %2
  %22 = load ptr, ptr %4, align 8
  %23 = call ptr @lua_tolstring(ptr noundef %22, i32 noundef -1, ptr noundef null)
  store ptr %23, ptr %7, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = call i32 @strncmp(ptr noundef %24, ptr noundef @.str.52, i64 noundef 3) #8
  %26 = icmp eq i32 %25, 0
  br i1 %26, label %27, label %34

27:                                               ; preds = %21
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds i8, ptr %29, i64 3
  %31 = call ptr @lua_pushstring(ptr noundef %28, ptr noundef %30)
  %32 = load ptr, ptr %4, align 8
  call void @lua_rotate(ptr noundef %32, i32 noundef -2, i32 noundef -1)
  %33 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %33, i32 noundef -2)
  br label %34

34:                                               ; preds = %27, %21
  %35 = load ptr, ptr %4, align 8
  %36 = load i32, ptr %6, align 4
  %37 = add nsw i32 %36, 1
  call void @lua_copy(ptr noundef %35, i32 noundef -1, i32 noundef %37)
  %38 = load ptr, ptr %4, align 8
  %39 = load i32, ptr %6, align 4
  %40 = add nsw i32 %39, 1
  call void @lua_settop(ptr noundef %38, i32 noundef %40)
  store i32 1, ptr %3, align 4
  br label %44

41:                                               ; preds = %2
  %42 = load ptr, ptr %4, align 8
  %43 = load i32, ptr %6, align 4
  call void @lua_settop(ptr noundef %42, i32 noundef %43)
  store i32 0, ptr %3, align 4
  br label %44

44:                                               ; preds = %41, %34
  %45 = load i32, ptr %3, align 4
  ret i32 %45
}

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_typeerror(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %5, align 4
  %11 = call i32 @luaL_getmetafield(ptr noundef %9, i32 noundef %10, ptr noundef @.str.12)
  %12 = icmp eq i32 %11, 4
  br i1 %12, label %13, label %16

13:                                               ; preds = %3
  %14 = load ptr, ptr %4, align 8
  %15 = call ptr @lua_tolstring(ptr noundef %14, i32 noundef -1, ptr noundef null)
  store ptr %15, ptr %8, align 8
  br label %29

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %5, align 4
  %19 = call i32 @lua_type(ptr noundef %17, i32 noundef %18)
  %20 = icmp eq i32 %19, 2
  br i1 %20, label %21, label %22

21:                                               ; preds = %16
  store ptr @.str.13, ptr %8, align 8
  br label %28

22:                                               ; preds = %16
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = load i32, ptr %5, align 4
  %26 = call i32 @lua_type(ptr noundef %24, i32 noundef %25)
  %27 = call ptr @lua_typename(ptr noundef %23, i32 noundef %26)
  store ptr %27, ptr %8, align 8
  br label %28

28:                                               ; preds = %22, %21
  br label %29

29:                                               ; preds = %28, %13
  %30 = load ptr, ptr %4, align 8
  %31 = load ptr, ptr %6, align 8
  %32 = load ptr, ptr %8, align 8
  %33 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %30, ptr noundef @.str.14, ptr noundef %31, ptr noundef %32)
  store ptr %33, ptr %7, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = load i32, ptr %5, align 4
  %36 = load ptr, ptr %7, align 8
  %37 = call i32 @luaL_argerror(ptr noundef %34, i32 noundef %35, ptr noundef %36)
  ret i32 %37
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_getmetafield(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call i32 @lua_getmetatable(ptr noundef %9, i32 noundef %10)
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %14, label %13

13:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %29

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = call ptr @lua_pushstring(ptr noundef %15, ptr noundef %16)
  %18 = load ptr, ptr %5, align 8
  %19 = call i32 @lua_rawget(ptr noundef %18, i32 noundef -2)
  store i32 %19, ptr %8, align 4
  %20 = load i32, ptr %8, align 4
  %21 = icmp eq i32 %20, 0
  br i1 %21, label %22, label %24

22:                                               ; preds = %14
  %23 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %23, i32 noundef -3)
  br label %27

24:                                               ; preds = %14
  %25 = load ptr, ptr %5, align 8
  call void @lua_rotate(ptr noundef %25, i32 noundef -2, i32 noundef -1)
  %26 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %26, i32 noundef -2)
  br label %27

27:                                               ; preds = %24, %22
  %28 = load i32, ptr %8, align 4
  store i32 %28, ptr %4, align 4
  br label %29

29:                                               ; preds = %27, %13
  %30 = load i32, ptr %4, align 4
  ret i32 %30
}

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare ptr @lua_typename(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_where(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca %struct.lua_Debug, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call i32 @lua_getstack(ptr noundef %6, i32 noundef %7, ptr noundef %5)
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %24

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8
  %12 = call i32 @lua_getinfo(ptr noundef %11, ptr noundef @.str.15, ptr noundef %5)
  %13 = getelementptr inbounds %struct.lua_Debug, ptr %5, i32 0, i32 6
  %14 = load i32, ptr %13, align 8
  %15 = icmp sgt i32 %14, 0
  br i1 %15, label %16, label %23

16:                                               ; preds = %10
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.lua_Debug, ptr %5, i32 0, i32 15
  %19 = getelementptr inbounds [60 x i8], ptr %18, i64 0, i64 0
  %20 = getelementptr inbounds %struct.lua_Debug, ptr %5, i32 0, i32 6
  %21 = load i32, ptr %20, align 8
  %22 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %17, ptr noundef @.str.16, ptr noundef %19, i32 noundef %21)
  br label %27

23:                                               ; preds = %10
  br label %24

24:                                               ; preds = %23, %2
  %25 = load ptr, ptr %3, align 8
  %26 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %25, ptr noundef @.str.17)
  br label %27

27:                                               ; preds = %24, %16
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_start(ptr) #3

declare ptr @lua_pushvfstring(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_end(ptr) #3

declare void @lua_concat(ptr noundef, i32 noundef) #1

declare i32 @lua_error(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_fileresult(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %10 = call ptr @__errno_location() #9
  %11 = load i32, ptr %10, align 4
  store i32 %11, ptr %8, align 4
  %12 = load i32, ptr %6, align 4
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %16

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  call void @lua_pushboolean(ptr noundef %15, i32 noundef 1)
  store i32 1, ptr %4, align 4
  br label %41

16:                                               ; preds = %3
  %17 = load ptr, ptr %5, align 8
  call void @lua_pushnil(ptr noundef %17)
  %18 = load i32, ptr %8, align 4
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %16
  %21 = load i32, ptr %8, align 4
  %22 = call ptr @strerror(i32 noundef %21) #10
  br label %24

23:                                               ; preds = %16
  br label %24

24:                                               ; preds = %23, %20
  %25 = phi ptr [ %22, %20 ], [ @.str.18, %23 ]
  store ptr %25, ptr %9, align 8
  %26 = load ptr, ptr %7, align 8
  %27 = icmp ne ptr %26, null
  br i1 %27, label %28, label %33

28:                                               ; preds = %24
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %7, align 8
  %31 = load ptr, ptr %9, align 8
  %32 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %29, ptr noundef @.str.19, ptr noundef %30, ptr noundef %31)
  br label %37

33:                                               ; preds = %24
  %34 = load ptr, ptr %5, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = call ptr @lua_pushstring(ptr noundef %34, ptr noundef %35)
  br label %37

37:                                               ; preds = %33, %28
  %38 = load ptr, ptr %5, align 8
  %39 = load i32, ptr %8, align 4
  %40 = sext i32 %39 to i64
  call void @lua_pushinteger(ptr noundef %38, i64 noundef %40)
  store i32 3, ptr %4, align 4
  br label %41

41:                                               ; preds = %37, %14
  %42 = load i32, ptr %4, align 4
  ret i32 %42
}

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__errno_location() #4

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare void @lua_pushnil(ptr noundef) #1

; Function Attrs: nounwind
declare ptr @strerror(i32 noundef) #5

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_execresult(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load i32, ptr %5, align 4
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %16

9:                                                ; preds = %2
  %10 = call ptr @__errno_location() #9
  %11 = load i32, ptr %10, align 4
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %13, label %16

13:                                               ; preds = %9
  %14 = load ptr, ptr %4, align 8
  %15 = call i32 @luaL_fileresult(ptr noundef %14, i32 noundef 0, ptr noundef null)
  store i32 %15, ptr %3, align 4
  br label %35

16:                                               ; preds = %9, %2
  store ptr @.str.20, ptr %6, align 8
  %17 = load ptr, ptr %6, align 8
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp eq i32 %19, 101
  br i1 %20, label %21, label %26

21:                                               ; preds = %16
  %22 = load i32, ptr %5, align 4
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %26

24:                                               ; preds = %21
  %25 = load ptr, ptr %4, align 8
  call void @lua_pushboolean(ptr noundef %25, i32 noundef 1)
  br label %28

26:                                               ; preds = %21, %16
  %27 = load ptr, ptr %4, align 8
  call void @lua_pushnil(ptr noundef %27)
  br label %28

28:                                               ; preds = %26, %24
  %29 = load ptr, ptr %4, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = call ptr @lua_pushstring(ptr noundef %29, ptr noundef %30)
  %32 = load ptr, ptr %4, align 8
  %33 = load i32, ptr %5, align 4
  %34 = sext i32 %33 to i64
  call void @lua_pushinteger(ptr noundef %32, i64 noundef %34)
  store i32 3, ptr %3, align 4
  br label %35

35:                                               ; preds = %28, %13
  %36 = load i32, ptr %3, align 4
  ret i32 %36
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_newmetatable(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = call i32 @lua_getfield(ptr noundef %6, i32 noundef -1001000, ptr noundef %7)
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %11

10:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %21

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %12, i32 noundef -2)
  %13 = load ptr, ptr %4, align 8
  call void @lua_createtable(ptr noundef %13, i32 noundef 0, i32 noundef 2)
  %14 = load ptr, ptr %4, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = call ptr @lua_pushstring(ptr noundef %14, ptr noundef %15)
  %17 = load ptr, ptr %4, align 8
  call void @lua_setfield(ptr noundef %17, i32 noundef -2, ptr noundef @.str.12)
  %18 = load ptr, ptr %4, align 8
  call void @lua_pushvalue(ptr noundef %18, i32 noundef -1)
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %19, i32 noundef -1001000, ptr noundef %20)
  store i32 1, ptr %3, align 4
  br label %21

21:                                               ; preds = %11, %10
  %22 = load i32, ptr %3, align 4
  ret i32 %22
}

declare i32 @lua_getfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_setmetatable(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = call i32 @lua_getfield(ptr noundef %5, i32 noundef -1001000, ptr noundef %6)
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @lua_setmetatable(ptr noundef %8, i32 noundef -2)
  ret void
}

declare i32 @lua_setmetatable(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_testudata(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call ptr @lua_touserdata(ptr noundef %9, i32 noundef %10)
  store ptr %11, ptr %8, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = icmp ne ptr %12, null
  br i1 %13, label %14, label %31

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %6, align 4
  %17 = call i32 @lua_getmetatable(ptr noundef %15, i32 noundef %16)
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %30

19:                                               ; preds = %14
  %20 = load ptr, ptr %5, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = call i32 @lua_getfield(ptr noundef %20, i32 noundef -1001000, ptr noundef %21)
  %23 = load ptr, ptr %5, align 8
  %24 = call i32 @lua_rawequal(ptr noundef %23, i32 noundef -1, i32 noundef -2)
  %25 = icmp ne i32 %24, 0
  br i1 %25, label %27, label %26

26:                                               ; preds = %19
  store ptr null, ptr %8, align 8
  br label %27

27:                                               ; preds = %26, %19
  %28 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %28, i32 noundef -3)
  %29 = load ptr, ptr %8, align 8
  store ptr %29, ptr %4, align 8
  br label %32

30:                                               ; preds = %14
  br label %31

31:                                               ; preds = %30, %3
  store ptr null, ptr %4, align 8
  br label %32

32:                                               ; preds = %31, %27
  %33 = load ptr, ptr %4, align 8
  ret ptr %33
}

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

declare i32 @lua_getmetatable(ptr noundef, i32 noundef) #1

declare i32 @lua_rawequal(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_checkudata(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = load ptr, ptr %6, align 8
  %11 = call ptr @luaL_testudata(ptr noundef %8, i32 noundef %9, ptr noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = icmp ne ptr %12, null
  %14 = zext i1 %13 to i32
  %15 = icmp ne i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = sext i32 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %25, label %19

19:                                               ; preds = %3
  %20 = load ptr, ptr %4, align 8
  %21 = load i32, ptr %5, align 4
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 @luaL_typeerror(ptr noundef %20, i32 noundef %21, ptr noundef %22)
  %24 = icmp ne i32 %23, 0
  br label %25

25:                                               ; preds = %19, %3
  %26 = phi i1 [ true, %3 ], [ %24, %19 ]
  %27 = zext i1 %26 to i32
  %28 = load ptr, ptr %7, align 8
  ret ptr %28
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_checkoption(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = icmp ne ptr %12, null
  br i1 %13, label %14, label %19

14:                                               ; preds = %4
  %15 = load ptr, ptr %6, align 8
  %16 = load i32, ptr %7, align 4
  %17 = load ptr, ptr %8, align 8
  %18 = call ptr @luaL_optlstring(ptr noundef %15, i32 noundef %16, ptr noundef %17, ptr noundef null)
  br label %23

19:                                               ; preds = %4
  %20 = load ptr, ptr %6, align 8
  %21 = load i32, ptr %7, align 4
  %22 = call ptr @luaL_checklstring(ptr noundef %20, i32 noundef %21, ptr noundef null)
  br label %23

23:                                               ; preds = %19, %14
  %24 = phi ptr [ %18, %14 ], [ %22, %19 ]
  store ptr %24, ptr %10, align 8
  store i32 0, ptr %11, align 4
  br label %25

25:                                               ; preds = %44, %23
  %26 = load ptr, ptr %9, align 8
  %27 = load i32, ptr %11, align 4
  %28 = sext i32 %27 to i64
  %29 = getelementptr inbounds ptr, ptr %26, i64 %28
  %30 = load ptr, ptr %29, align 8
  %31 = icmp ne ptr %30, null
  br i1 %31, label %32, label %47

32:                                               ; preds = %25
  %33 = load ptr, ptr %9, align 8
  %34 = load i32, ptr %11, align 4
  %35 = sext i32 %34 to i64
  %36 = getelementptr inbounds ptr, ptr %33, i64 %35
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %10, align 8
  %39 = call i32 @strcmp(ptr noundef %37, ptr noundef %38) #8
  %40 = icmp eq i32 %39, 0
  br i1 %40, label %41, label %43

41:                                               ; preds = %32
  %42 = load i32, ptr %11, align 4
  store i32 %42, ptr %5, align 4
  br label %54

43:                                               ; preds = %32
  br label %44

44:                                               ; preds = %43
  %45 = load i32, ptr %11, align 4
  %46 = add nsw i32 %45, 1
  store i32 %46, ptr %11, align 4
  br label %25, !llvm.loop !10

47:                                               ; preds = %25
  %48 = load ptr, ptr %6, align 8
  %49 = load i32, ptr %7, align 4
  %50 = load ptr, ptr %6, align 8
  %51 = load ptr, ptr %10, align 8
  %52 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %50, ptr noundef @.str.21, ptr noundef %51)
  %53 = call i32 @luaL_argerror(ptr noundef %48, i32 noundef %49, ptr noundef %52)
  store i32 %53, ptr %5, align 4
  br label %54

54:                                               ; preds = %47, %41
  %55 = load i32, ptr %5, align 4
  ret i32 %55
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_optlstring(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load i32, ptr %7, align 4
  %12 = call i32 @lua_type(ptr noundef %10, i32 noundef %11)
  %13 = icmp sle i32 %12, 0
  br i1 %13, label %14, label %29

14:                                               ; preds = %4
  %15 = load ptr, ptr %9, align 8
  %16 = icmp ne ptr %15, null
  br i1 %16, label %17, label %27

17:                                               ; preds = %14
  %18 = load ptr, ptr %8, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %20, label %23

20:                                               ; preds = %17
  %21 = load ptr, ptr %8, align 8
  %22 = call i64 @strlen(ptr noundef %21) #8
  br label %24

23:                                               ; preds = %17
  br label %24

24:                                               ; preds = %23, %20
  %25 = phi i64 [ %22, %20 ], [ 0, %23 ]
  %26 = load ptr, ptr %9, align 8
  store i64 %25, ptr %26, align 8
  br label %27

27:                                               ; preds = %24, %14
  %28 = load ptr, ptr %8, align 8
  store ptr %28, ptr %5, align 8
  br label %34

29:                                               ; preds = %4
  %30 = load ptr, ptr %6, align 8
  %31 = load i32, ptr %7, align 4
  %32 = load ptr, ptr %9, align 8
  %33 = call ptr @luaL_checklstring(ptr noundef %30, i32 noundef %31, ptr noundef %32)
  store ptr %33, ptr %5, align 8
  br label %34

34:                                               ; preds = %29, %27
  %35 = load ptr, ptr %5, align 8
  ret ptr %35
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_checklstring(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = load ptr, ptr %6, align 8
  %11 = call ptr @lua_tolstring(ptr noundef %8, i32 noundef %9, ptr noundef %10)
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = icmp ne ptr %12, null
  %14 = xor i1 %13, true
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %3
  %21 = load ptr, ptr %4, align 8
  %22 = load i32, ptr %5, align 4
  call void @tag_error(ptr noundef %21, i32 noundef %22, i32 noundef 4)
  br label %23

23:                                               ; preds = %20, %3
  %24 = load ptr, ptr %7, align 8
  ret ptr %24
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_checkstack(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call i32 @lua_checkstack(ptr noundef %7, i32 noundef %8)
  %10 = icmp ne i32 %9, 0
  %11 = xor i1 %10, true
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %28

17:                                               ; preds = %3
  %18 = load ptr, ptr %6, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %20, label %24

20:                                               ; preds = %17
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %21, ptr noundef @.str.22, ptr noundef %22)
  br label %27

24:                                               ; preds = %17
  %25 = load ptr, ptr %4, align 8
  %26 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %25, ptr noundef @.str.23)
  br label %27

27:                                               ; preds = %24, %20
  br label %28

28:                                               ; preds = %27, %3
  ret void
}

declare i32 @lua_checkstack(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_checktype(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call i32 @lua_type(ptr noundef %7, i32 noundef %8)
  %10 = load i32, ptr %6, align 4
  %11 = icmp ne i32 %9, %10
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %21

17:                                               ; preds = %3
  %18 = load ptr, ptr %4, align 8
  %19 = load i32, ptr %5, align 4
  %20 = load i32, ptr %6, align 4
  call void @tag_error(ptr noundef %18, i32 noundef %19, i32 noundef %20)
  br label %21

21:                                               ; preds = %17, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @tag_error(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %6, align 4
  %11 = call ptr @lua_typename(ptr noundef %9, i32 noundef %10)
  %12 = call i32 @luaL_typeerror(ptr noundef %7, i32 noundef %8, ptr noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_checkany(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = call i32 @lua_type(ptr noundef %5, i32 noundef %6)
  %8 = icmp eq i32 %7, -1
  %9 = zext i1 %8 to i32
  %10 = icmp ne i32 %9, 0
  %11 = zext i1 %10 to i32
  %12 = sext i32 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %14, label %18

14:                                               ; preds = %2
  %15 = load ptr, ptr %3, align 8
  %16 = load i32, ptr %4, align 4
  %17 = call i32 @luaL_argerror(ptr noundef %15, i32 noundef %16, ptr noundef @.str.24)
  br label %18

18:                                               ; preds = %14, %2
  ret void
}

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local double @luaL_checknumber(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca double, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = call double @lua_tonumberx(ptr noundef %7, i32 noundef %8, ptr noundef %5)
  store double %9, ptr %6, align 8
  %10 = load i32, ptr %5, align 4
  %11 = icmp ne i32 %10, 0
  %12 = xor i1 %11, true
  %13 = zext i1 %12 to i32
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = sext i32 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %18, label %21

18:                                               ; preds = %2
  %19 = load ptr, ptr %3, align 8
  %20 = load i32, ptr %4, align 4
  call void @tag_error(ptr noundef %19, i32 noundef %20, i32 noundef 3)
  br label %21

21:                                               ; preds = %18, %2
  %22 = load double, ptr %6, align 8
  ret double %22
}

declare double @lua_tonumberx(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local double @luaL_optnumber(ptr noundef %0, i32 noundef %1, double noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store double %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call i32 @lua_type(ptr noundef %7, i32 noundef %8)
  %10 = icmp sle i32 %9, 0
  br i1 %10, label %11, label %13

11:                                               ; preds = %3
  %12 = load double, ptr %6, align 8
  br label %17

13:                                               ; preds = %3
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call double @luaL_checknumber(ptr noundef %14, i32 noundef %15)
  br label %17

17:                                               ; preds = %13, %11
  %18 = phi double [ %12, %11 ], [ %16, %13 ]
  ret double %18
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @luaL_checkinteger(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  %9 = call i64 @lua_tointegerx(ptr noundef %7, i32 noundef %8, ptr noundef %5)
  store i64 %9, ptr %6, align 8
  %10 = load i32, ptr %5, align 4
  %11 = icmp ne i32 %10, 0
  %12 = xor i1 %11, true
  %13 = zext i1 %12 to i32
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = sext i32 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %18, label %21

18:                                               ; preds = %2
  %19 = load ptr, ptr %3, align 8
  %20 = load i32, ptr %4, align 4
  call void @interror(ptr noundef %19, i32 noundef %20)
  br label %21

21:                                               ; preds = %18, %2
  %22 = load i64, ptr %6, align 8
  ret i64 %22
}

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @interror(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load ptr, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = call i32 @lua_isnumber(ptr noundef %5, i32 noundef %6)
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %13

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = call i32 @luaL_argerror(ptr noundef %10, i32 noundef %11, ptr noundef @.str.54)
  br label %16

13:                                               ; preds = %2
  %14 = load ptr, ptr %3, align 8
  %15 = load i32, ptr %4, align 4
  call void @tag_error(ptr noundef %14, i32 noundef %15, i32 noundef 3)
  br label %16

16:                                               ; preds = %13, %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @luaL_optinteger(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = call i32 @lua_type(ptr noundef %7, i32 noundef %8)
  %10 = icmp sle i32 %9, 0
  br i1 %10, label %11, label %13

11:                                               ; preds = %3
  %12 = load i64, ptr %6, align 8
  br label %17

13:                                               ; preds = %3
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call i64 @luaL_checkinteger(ptr noundef %14, i32 noundef %15)
  br label %17

17:                                               ; preds = %13, %11
  %18 = phi i64 [ %12, %11 ], [ %16, %13 ]
  ret i64 %18
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @prepbuffsize(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 1
  %13 = load i64, ptr %12, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 2
  %16 = load i64, ptr %15, align 8
  %17 = sub i64 %13, %16
  %18 = load i64, ptr %6, align 8
  %19 = icmp uge i64 %17, %18
  br i1 %19, label %20, label %28

20:                                               ; preds = %3
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.luaL_Buffer, ptr %21, i32 0, i32 0
  %23 = load ptr, ptr %22, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.luaL_Buffer, ptr %24, i32 0, i32 2
  %26 = load i64, ptr %25, align 8
  %27 = getelementptr inbounds i8, ptr %23, i64 %26
  store ptr %27, ptr %4, align 8
  br label %80

28:                                               ; preds = %3
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.luaL_Buffer, ptr %29, i32 0, i32 3
  %31 = load ptr, ptr %30, align 8
  store ptr %31, ptr %8, align 8
  %32 = load ptr, ptr %5, align 8
  %33 = load i64, ptr %6, align 8
  %34 = call i64 @newbuffsize(ptr noundef %32, i64 noundef %33)
  store i64 %34, ptr %10, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.luaL_Buffer, ptr %35, i32 0, i32 0
  %37 = load ptr, ptr %36, align 8
  %38 = load ptr, ptr %5, align 8
  %39 = getelementptr inbounds %struct.luaL_Buffer, ptr %38, i32 0, i32 4
  %40 = getelementptr inbounds [1024 x i8], ptr %39, i64 0, i64 0
  %41 = icmp ne ptr %37, %40
  br i1 %41, label %42, label %47

42:                                               ; preds = %28
  %43 = load ptr, ptr %8, align 8
  %44 = load i32, ptr %7, align 4
  %45 = load i64, ptr %10, align 8
  %46 = call ptr @resizebox(ptr noundef %43, i32 noundef %44, i64 noundef %45)
  store ptr %46, ptr %9, align 8
  br label %68

47:                                               ; preds = %28
  %48 = load ptr, ptr %8, align 8
  %49 = load i32, ptr %7, align 4
  call void @lua_rotate(ptr noundef %48, i32 noundef %49, i32 noundef -1)
  %50 = load ptr, ptr %8, align 8
  call void @lua_settop(ptr noundef %50, i32 noundef -2)
  %51 = load ptr, ptr %8, align 8
  call void @newbox(ptr noundef %51)
  %52 = load ptr, ptr %8, align 8
  %53 = load i32, ptr %7, align 4
  call void @lua_rotate(ptr noundef %52, i32 noundef %53, i32 noundef 1)
  %54 = load ptr, ptr %8, align 8
  %55 = load i32, ptr %7, align 4
  call void @lua_toclose(ptr noundef %54, i32 noundef %55)
  %56 = load ptr, ptr %8, align 8
  %57 = load i32, ptr %7, align 4
  %58 = load i64, ptr %10, align 8
  %59 = call ptr @resizebox(ptr noundef %56, i32 noundef %57, i64 noundef %58)
  store ptr %59, ptr %9, align 8
  %60 = load ptr, ptr %9, align 8
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %struct.luaL_Buffer, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  %64 = load ptr, ptr %5, align 8
  %65 = getelementptr inbounds %struct.luaL_Buffer, ptr %64, i32 0, i32 2
  %66 = load i64, ptr %65, align 8
  %67 = mul i64 %66, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %60, ptr align 1 %63, i64 %67, i1 false)
  br label %68

68:                                               ; preds = %47, %42
  %69 = load ptr, ptr %9, align 8
  %70 = load ptr, ptr %5, align 8
  %71 = getelementptr inbounds %struct.luaL_Buffer, ptr %70, i32 0, i32 0
  store ptr %69, ptr %71, align 8
  %72 = load i64, ptr %10, align 8
  %73 = load ptr, ptr %5, align 8
  %74 = getelementptr inbounds %struct.luaL_Buffer, ptr %73, i32 0, i32 1
  store i64 %72, ptr %74, align 8
  %75 = load ptr, ptr %9, align 8
  %76 = load ptr, ptr %5, align 8
  %77 = getelementptr inbounds %struct.luaL_Buffer, ptr %76, i32 0, i32 2
  %78 = load i64, ptr %77, align 8
  %79 = getelementptr inbounds i8, ptr %75, i64 %78
  store ptr %79, ptr %4, align 8
  br label %80

80:                                               ; preds = %68, %20
  %81 = load ptr, ptr %4, align 8
  ret ptr %81
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_addlstring(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %8 = load i64, ptr %6, align 8
  %9 = icmp ugt i64 %8, 0
  br i1 %9, label %10, label %23

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i64, ptr %6, align 8
  %13 = call ptr @prepbuffsize(ptr noundef %11, i64 noundef %12, i32 noundef -1)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = load i64, ptr %6, align 8
  %17 = mul i64 %16, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %14, ptr align 1 %15, i64 %17, i1 false)
  %18 = load i64, ptr %6, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.luaL_Buffer, ptr %19, i32 0, i32 2
  %21 = load i64, ptr %20, align 8
  %22 = add i64 %21, %18
  store i64 %22, ptr %20, align 8
  br label %23

23:                                               ; preds = %10, %3
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #6

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare void @lua_closeslot(ptr noundef, i32 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_pushresultsize(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  %5 = load i64, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Buffer, ptr %6, i32 0, i32 2
  %8 = load i64, ptr %7, align 8
  %9 = add i64 %8, %5
  store i64 %9, ptr %7, align 8
  %10 = load ptr, ptr %3, align 8
  call void @luaL_pushresult(ptr noundef %10)
  ret void
}

declare void @lua_pushlightuserdata(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_buffinitsize(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  call void @luaL_buffinit(ptr noundef %7, ptr noundef %8)
  %9 = load ptr, ptr %5, align 8
  %10 = load i64, ptr %6, align 8
  %11 = call ptr @prepbuffsize(ptr noundef %9, i64 noundef %10, i32 noundef -1)
  ret ptr %11
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_ref(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @lua_type(ptr noundef %7, i32 noundef -1)
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %10, label %12

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %11, i32 noundef -2)
  store i32 -1, ptr %3, align 4
  br label %52

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call i32 @lua_absindex(ptr noundef %13, i32 noundef %14)
  store i32 %15, ptr %5, align 4
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %5, align 4
  %18 = call i32 @lua_rawgeti(ptr noundef %16, i32 noundef %17, i64 noundef 3)
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %24

20:                                               ; preds = %12
  store i32 0, ptr %6, align 4
  %21 = load ptr, ptr %4, align 8
  call void @lua_pushinteger(ptr noundef %21, i64 noundef 0)
  %22 = load ptr, ptr %4, align 8
  %23 = load i32, ptr %5, align 4
  call void @lua_rawseti(ptr noundef %22, i32 noundef %23, i64 noundef 3)
  br label %28

24:                                               ; preds = %12
  %25 = load ptr, ptr %4, align 8
  %26 = call i64 @lua_tointegerx(ptr noundef %25, i32 noundef -1, ptr noundef null)
  %27 = trunc i64 %26 to i32
  store i32 %27, ptr %6, align 4
  br label %28

28:                                               ; preds = %24, %20
  %29 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %29, i32 noundef -2)
  %30 = load i32, ptr %6, align 4
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %40

32:                                               ; preds = %28
  %33 = load ptr, ptr %4, align 8
  %34 = load i32, ptr %5, align 4
  %35 = load i32, ptr %6, align 4
  %36 = sext i32 %35 to i64
  %37 = call i32 @lua_rawgeti(ptr noundef %33, i32 noundef %34, i64 noundef %36)
  %38 = load ptr, ptr %4, align 8
  %39 = load i32, ptr %5, align 4
  call void @lua_rawseti(ptr noundef %38, i32 noundef %39, i64 noundef 3)
  br label %46

40:                                               ; preds = %28
  %41 = load ptr, ptr %4, align 8
  %42 = load i32, ptr %5, align 4
  %43 = call i64 @lua_rawlen(ptr noundef %41, i32 noundef %42)
  %44 = trunc i64 %43 to i32
  %45 = add nsw i32 %44, 1
  store i32 %45, ptr %6, align 4
  br label %46

46:                                               ; preds = %40, %32
  %47 = load ptr, ptr %4, align 8
  %48 = load i32, ptr %5, align 4
  %49 = load i32, ptr %6, align 4
  %50 = sext i32 %49 to i64
  call void @lua_rawseti(ptr noundef %47, i32 noundef %48, i64 noundef %50)
  %51 = load i32, ptr %6, align 4
  store i32 %51, ptr %3, align 4
  br label %52

52:                                               ; preds = %46, %10
  %53 = load i32, ptr %3, align 4
  ret i32 %53
}

declare i32 @lua_absindex(ptr noundef, i32 noundef) #1

declare i32 @lua_rawgeti(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_rawseti(ptr noundef, i32 noundef, i64 noundef) #1

declare i64 @lua_rawlen(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_unref(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load i32, ptr %6, align 4
  %8 = icmp sge i32 %7, 0
  br i1 %8, label %9, label %25

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %5, align 4
  %12 = call i32 @lua_absindex(ptr noundef %10, i32 noundef %11)
  store i32 %12, ptr %5, align 4
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call i32 @lua_rawgeti(ptr noundef %13, i32 noundef %14, i64 noundef 3)
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %5, align 4
  %18 = load i32, ptr %6, align 4
  %19 = sext i32 %18 to i64
  call void @lua_rawseti(ptr noundef %16, i32 noundef %17, i64 noundef %19)
  %20 = load ptr, ptr %4, align 8
  %21 = load i32, ptr %6, align 4
  %22 = sext i32 %21 to i64
  call void @lua_pushinteger(ptr noundef %20, i64 noundef %22)
  %23 = load ptr, ptr %4, align 8
  %24 = load i32, ptr %5, align 4
  call void @lua_rawseti(ptr noundef %23, i32 noundef %24, i64 noundef 3)
  br label %25

25:                                               ; preds = %9, %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_loadfilex(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %struct.LoadF, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @lua_gettop(ptr noundef %13)
  %15 = add nsw i32 %14, 1
  store i32 %15, ptr %12, align 4
  %16 = load ptr, ptr %6, align 8
  %17 = icmp eq ptr %16, null
  br i1 %17, label %18, label %23

18:                                               ; preds = %3
  %19 = load ptr, ptr %5, align 8
  %20 = call ptr @lua_pushstring(ptr noundef %19, ptr noundef @.str.25)
  %21 = load ptr, ptr @stdin, align 8
  %22 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  store ptr %21, ptr %22, align 8
  br label %39

23:                                               ; preds = %3
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %6, align 8
  %26 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %24, ptr noundef @.str.26, ptr noundef %25)
  %27 = call ptr @__errno_location() #9
  store i32 0, ptr %27, align 4
  %28 = load ptr, ptr %6, align 8
  %29 = call noalias ptr @fopen64(ptr noundef %28, ptr noundef @.str.27)
  %30 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  store ptr %29, ptr %30, align 8
  %31 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %32 = load ptr, ptr %31, align 8
  %33 = icmp eq ptr %32, null
  br i1 %33, label %34, label %38

34:                                               ; preds = %23
  %35 = load ptr, ptr %5, align 8
  %36 = load i32, ptr %12, align 4
  %37 = call i32 @errfile(ptr noundef %35, ptr noundef @.str.28, i32 noundef %36)
  store i32 %37, ptr %4, align 4
  br label %122

38:                                               ; preds = %23
  br label %39

39:                                               ; preds = %38, %18
  %40 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 0
  store i32 0, ptr %40, align 8
  %41 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %42 = load ptr, ptr %41, align 8
  %43 = call i32 @skipcomment(ptr noundef %42, ptr noundef %11)
  %44 = icmp ne i32 %43, 0
  br i1 %44, label %45, label %52

45:                                               ; preds = %39
  %46 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 2
  %47 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 0
  %48 = load i32, ptr %47, align 8
  %49 = add nsw i32 %48, 1
  store i32 %49, ptr %47, align 8
  %50 = sext i32 %48 to i64
  %51 = getelementptr inbounds [8192 x i8], ptr %46, i64 0, i64 %50
  store i8 10, ptr %51, align 1
  br label %52

52:                                               ; preds = %45, %39
  %53 = load i32, ptr %11, align 4
  %54 = load i8, ptr @.str.29, align 1
  %55 = sext i8 %54 to i32
  %56 = icmp eq i32 %53, %55
  br i1 %56, label %57, label %80

57:                                               ; preds = %52
  %58 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 0
  store i32 0, ptr %58, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = icmp ne ptr %59, null
  br i1 %60, label %61, label %79

61:                                               ; preds = %57
  %62 = call ptr @__errno_location() #9
  store i32 0, ptr %62, align 4
  %63 = load ptr, ptr %6, align 8
  %64 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %65 = load ptr, ptr %64, align 8
  %66 = call ptr @freopen64(ptr noundef %63, ptr noundef @.str.30, ptr noundef %65)
  %67 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  store ptr %66, ptr %67, align 8
  %68 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %69 = load ptr, ptr %68, align 8
  %70 = icmp eq ptr %69, null
  br i1 %70, label %71, label %75

71:                                               ; preds = %61
  %72 = load ptr, ptr %5, align 8
  %73 = load i32, ptr %12, align 4
  %74 = call i32 @errfile(ptr noundef %72, ptr noundef @.str.31, i32 noundef %73)
  store i32 %74, ptr %4, align 4
  br label %122

75:                                               ; preds = %61
  %76 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %77 = load ptr, ptr %76, align 8
  %78 = call i32 @skipcomment(ptr noundef %77, ptr noundef %11)
  br label %79

79:                                               ; preds = %75, %57
  br label %80

80:                                               ; preds = %79, %52
  %81 = load i32, ptr %11, align 4
  %82 = icmp ne i32 %81, -1
  br i1 %82, label %83, label %92

83:                                               ; preds = %80
  %84 = load i32, ptr %11, align 4
  %85 = trunc i32 %84 to i8
  %86 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 2
  %87 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 0
  %88 = load i32, ptr %87, align 8
  %89 = add nsw i32 %88, 1
  store i32 %89, ptr %87, align 8
  %90 = sext i32 %88 to i64
  %91 = getelementptr inbounds [8192 x i8], ptr %86, i64 0, i64 %90
  store i8 %85, ptr %91, align 1
  br label %92

92:                                               ; preds = %83, %80
  %93 = call ptr @__errno_location() #9
  store i32 0, ptr %93, align 4
  %94 = load ptr, ptr %5, align 8
  %95 = load ptr, ptr %5, align 8
  %96 = call ptr @lua_tolstring(ptr noundef %95, i32 noundef -1, ptr noundef null)
  %97 = load ptr, ptr %7, align 8
  %98 = call i32 @lua_load(ptr noundef %94, ptr noundef @getF, ptr noundef %8, ptr noundef %96, ptr noundef %97)
  store i32 %98, ptr %9, align 4
  %99 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %100 = load ptr, ptr %99, align 8
  %101 = call i32 @ferror(ptr noundef %100) #10
  store i32 %101, ptr %10, align 4
  %102 = load ptr, ptr %6, align 8
  %103 = icmp ne ptr %102, null
  br i1 %103, label %104, label %108

104:                                              ; preds = %92
  %105 = getelementptr inbounds %struct.LoadF, ptr %8, i32 0, i32 1
  %106 = load ptr, ptr %105, align 8
  %107 = call i32 @fclose(ptr noundef %106)
  br label %108

108:                                              ; preds = %104, %92
  %109 = load i32, ptr %10, align 4
  %110 = icmp ne i32 %109, 0
  br i1 %110, label %111, label %117

111:                                              ; preds = %108
  %112 = load ptr, ptr %5, align 8
  %113 = load i32, ptr %12, align 4
  call void @lua_settop(ptr noundef %112, i32 noundef %113)
  %114 = load ptr, ptr %5, align 8
  %115 = load i32, ptr %12, align 4
  %116 = call i32 @errfile(ptr noundef %114, ptr noundef @.str.32, i32 noundef %115)
  store i32 %116, ptr %4, align 4
  br label %122

117:                                              ; preds = %108
  %118 = load ptr, ptr %5, align 8
  %119 = load i32, ptr %12, align 4
  call void @lua_rotate(ptr noundef %118, i32 noundef %119, i32 noundef -1)
  %120 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %120, i32 noundef -2)
  %121 = load i32, ptr %9, align 4
  store i32 %121, ptr %4, align 4
  br label %122

122:                                              ; preds = %117, %111, %71, %34
  %123 = load i32, ptr %4, align 4
  ret i32 %123
}

declare i32 @lua_gettop(ptr noundef) #1

declare noalias ptr @fopen64(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @errfile(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %9 = call ptr @__errno_location() #9
  %10 = load i32, ptr %9, align 4
  store i32 %10, ptr %7, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %6, align 4
  %13 = call ptr @lua_tolstring(ptr noundef %11, i32 noundef %12, ptr noundef null)
  %14 = getelementptr inbounds i8, ptr %13, i64 1
  store ptr %14, ptr %8, align 8
  %15 = load i32, ptr %7, align 4
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %24

17:                                               ; preds = %3
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = load i32, ptr %7, align 4
  %22 = call ptr @strerror(i32 noundef %21) #10
  %23 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %18, ptr noundef @.str.60, ptr noundef %19, ptr noundef %20, ptr noundef %22)
  br label %29

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %8, align 8
  %28 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %25, ptr noundef @.str.61, ptr noundef %26, ptr noundef %27)
  br label %29

29:                                               ; preds = %24, %17
  %30 = load ptr, ptr %4, align 8
  %31 = load i32, ptr %6, align 4
  call void @lua_rotate(ptr noundef %30, i32 noundef %31, i32 noundef -1)
  %32 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %32, i32 noundef -2)
  ret i32 6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @skipcomment(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @skipBOM(ptr noundef %7)
  %9 = load ptr, ptr %5, align 8
  store i32 %8, ptr %9, align 4
  store i32 %8, ptr %6, align 4
  %10 = load i32, ptr %6, align 4
  %11 = icmp eq i32 %10, 35
  br i1 %11, label %12, label %28

12:                                               ; preds = %2
  br label %13

13:                                               ; preds = %22, %12
  %14 = load ptr, ptr %4, align 8
  %15 = call i32 @getc(ptr noundef %14)
  store i32 %15, ptr %6, align 4
  br label %16

16:                                               ; preds = %13
  %17 = load i32, ptr %6, align 4
  %18 = icmp ne i32 %17, -1
  br i1 %18, label %19, label %22

19:                                               ; preds = %16
  %20 = load i32, ptr %6, align 4
  %21 = icmp ne i32 %20, 10
  br label %22

22:                                               ; preds = %19, %16
  %23 = phi i1 [ false, %16 ], [ %21, %19 ]
  br i1 %23, label %13, label %24, !llvm.loop !11

24:                                               ; preds = %22
  %25 = load ptr, ptr %4, align 8
  %26 = call i32 @getc(ptr noundef %25)
  %27 = load ptr, ptr %5, align 8
  store i32 %26, ptr %27, align 4
  store i32 1, ptr %3, align 4
  br label %29

28:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %29

29:                                               ; preds = %28, %24
  %30 = load i32, ptr %3, align 4
  ret i32 %30
}

declare ptr @freopen64(ptr noundef, ptr noundef, ptr noundef) #1

declare i32 @lua_load(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getF(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %6, align 8
  store ptr %9, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %8, align 8
  %12 = getelementptr inbounds %struct.LoadF, ptr %11, i32 0, i32 0
  %13 = load i32, ptr %12, align 8
  %14 = icmp sgt i32 %13, 0
  br i1 %14, label %15, label %23

15:                                               ; preds = %3
  %16 = load ptr, ptr %8, align 8
  %17 = getelementptr inbounds %struct.LoadF, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  %19 = sext i32 %18 to i64
  %20 = load ptr, ptr %7, align 8
  store i64 %19, ptr %20, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = getelementptr inbounds %struct.LoadF, ptr %21, i32 0, i32 0
  store i32 0, ptr %22, align 8
  br label %39

23:                                               ; preds = %3
  %24 = load ptr, ptr %8, align 8
  %25 = getelementptr inbounds %struct.LoadF, ptr %24, i32 0, i32 1
  %26 = load ptr, ptr %25, align 8
  %27 = call i32 @feof(ptr noundef %26) #10
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %30

29:                                               ; preds = %23
  store ptr null, ptr %4, align 8
  br label %43

30:                                               ; preds = %23
  %31 = load ptr, ptr %8, align 8
  %32 = getelementptr inbounds %struct.LoadF, ptr %31, i32 0, i32 2
  %33 = getelementptr inbounds [8192 x i8], ptr %32, i64 0, i64 0
  %34 = load ptr, ptr %8, align 8
  %35 = getelementptr inbounds %struct.LoadF, ptr %34, i32 0, i32 1
  %36 = load ptr, ptr %35, align 8
  %37 = call i64 @fread(ptr noundef %33, i64 noundef 1, i64 noundef 8192, ptr noundef %36)
  %38 = load ptr, ptr %7, align 8
  store i64 %37, ptr %38, align 8
  br label %39

39:                                               ; preds = %30, %15
  %40 = load ptr, ptr %8, align 8
  %41 = getelementptr inbounds %struct.LoadF, ptr %40, i32 0, i32 2
  %42 = getelementptr inbounds [8192 x i8], ptr %41, i64 0, i64 0
  store ptr %42, ptr %4, align 8
  br label %43

43:                                               ; preds = %39, %29
  %44 = load ptr, ptr %4, align 8
  ret ptr %44
}

; Function Attrs: nounwind
declare i32 @ferror(ptr noundef) #5

declare i32 @fclose(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_loadbufferx(ptr noundef %0, ptr noundef %1, i64 noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca %struct.LoadS, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i64 %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = getelementptr inbounds %struct.LoadS, ptr %11, i32 0, i32 0
  store ptr %12, ptr %13, align 8
  %14 = load i64, ptr %8, align 8
  %15 = getelementptr inbounds %struct.LoadS, ptr %11, i32 0, i32 1
  store i64 %14, ptr %15, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = load ptr, ptr %9, align 8
  %18 = load ptr, ptr %10, align 8
  %19 = call i32 @lua_load(ptr noundef %16, ptr noundef @getS, ptr noundef %11, ptr noundef %17, ptr noundef %18)
  ret i32 %19
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getS(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %9 = load ptr, ptr %6, align 8
  store ptr %9, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %8, align 8
  %12 = getelementptr inbounds %struct.LoadS, ptr %11, i32 0, i32 1
  %13 = load i64, ptr %12, align 8
  %14 = icmp eq i64 %13, 0
  br i1 %14, label %15, label %16

15:                                               ; preds = %3
  store ptr null, ptr %4, align 8
  br label %26

16:                                               ; preds = %3
  %17 = load ptr, ptr %8, align 8
  %18 = getelementptr inbounds %struct.LoadS, ptr %17, i32 0, i32 1
  %19 = load i64, ptr %18, align 8
  %20 = load ptr, ptr %7, align 8
  store i64 %19, ptr %20, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = getelementptr inbounds %struct.LoadS, ptr %21, i32 0, i32 1
  store i64 0, ptr %22, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = getelementptr inbounds %struct.LoadS, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  store ptr %25, ptr %4, align 8
  br label %26

26:                                               ; preds = %16, %15
  %27 = load ptr, ptr %4, align 8
  ret ptr %27
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_loadstring(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i64 @strlen(ptr noundef %7) #8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @luaL_loadbufferx(ptr noundef %5, ptr noundef %6, i64 noundef %8, ptr noundef %9, ptr noundef null)
  ret i32 %10
}

declare i32 @lua_rawget(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_callmeta(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  %10 = call i32 @lua_absindex(ptr noundef %8, i32 noundef %9)
  store i32 %10, ptr %6, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %6, align 4
  %13 = load ptr, ptr %7, align 8
  %14 = call i32 @luaL_getmetafield(ptr noundef %11, i32 noundef %12, ptr noundef %13)
  %15 = icmp eq i32 %14, 0
  br i1 %15, label %16, label %17

16:                                               ; preds = %3
  store i32 0, ptr %4, align 4
  br label %21

17:                                               ; preds = %3
  %18 = load ptr, ptr %5, align 8
  %19 = load i32, ptr %6, align 4
  call void @lua_pushvalue(ptr noundef %18, i32 noundef %19)
  %20 = load ptr, ptr %5, align 8
  call void @lua_callk(ptr noundef %20, i32 noundef 1, i32 noundef 1, i64 noundef 0, ptr noundef null)
  store i32 1, ptr %4, align 4
  br label %21

21:                                               ; preds = %17, %16
  %22 = load i32, ptr %4, align 4
  ret i32 %22
}

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @luaL_len(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = load i32, ptr %4, align 4
  call void @lua_len(ptr noundef %7, i32 noundef %8)
  %9 = load ptr, ptr %3, align 8
  %10 = call i64 @lua_tointegerx(ptr noundef %9, i32 noundef -1, ptr noundef %6)
  store i64 %10, ptr %5, align 8
  %11 = load i32, ptr %6, align 4
  %12 = icmp ne i32 %11, 0
  %13 = xor i1 %12, true
  %14 = zext i1 %13 to i32
  %15 = icmp ne i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = sext i32 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %19, label %22

19:                                               ; preds = %2
  %20 = load ptr, ptr %3, align 8
  %21 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %20, ptr noundef @.str.33)
  br label %22

22:                                               ; preds = %19, %2
  %23 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %23, i32 noundef -2)
  %24 = load i64, ptr %5, align 8
  ret i64 %24
}

declare void @lua_len(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_tolstring(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load i32, ptr %5, align 4
  %11 = call i32 @lua_absindex(ptr noundef %9, i32 noundef %10)
  store i32 %11, ptr %5, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = load i32, ptr %5, align 4
  %14 = call i32 @luaL_callmeta(ptr noundef %12, i32 noundef %13, ptr noundef @.str.34)
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %24

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = call i32 @lua_isstring(ptr noundef %17, i32 noundef -1)
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %23, label %20

20:                                               ; preds = %16
  %21 = load ptr, ptr %4, align 8
  %22 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %21, ptr noundef @.str.35)
  br label %23

23:                                               ; preds = %20, %16
  br label %91

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  %26 = load i32, ptr %5, align 4
  %27 = call i32 @lua_type(ptr noundef %25, i32 noundef %26)
  switch i32 %27, label %61 [
    i32 3, label %28
    i32 4, label %46
    i32 1, label %49
    i32 0, label %58
  ]

28:                                               ; preds = %24
  %29 = load ptr, ptr %4, align 8
  %30 = load i32, ptr %5, align 4
  %31 = call i32 @lua_isinteger(ptr noundef %29, i32 noundef %30)
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %39

33:                                               ; preds = %28
  %34 = load ptr, ptr %4, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = load i32, ptr %5, align 4
  %37 = call i64 @lua_tointegerx(ptr noundef %35, i32 noundef %36, ptr noundef null)
  %38 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %34, ptr noundef @.str.36, i64 noundef %37)
  br label %45

39:                                               ; preds = %28
  %40 = load ptr, ptr %4, align 8
  %41 = load ptr, ptr %4, align 8
  %42 = load i32, ptr %5, align 4
  %43 = call double @lua_tonumberx(ptr noundef %41, i32 noundef %42, ptr noundef null)
  %44 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %40, ptr noundef @.str.37, double noundef %43)
  br label %45

45:                                               ; preds = %39, %33
  br label %90

46:                                               ; preds = %24
  %47 = load ptr, ptr %4, align 8
  %48 = load i32, ptr %5, align 4
  call void @lua_pushvalue(ptr noundef %47, i32 noundef %48)
  br label %90

49:                                               ; preds = %24
  %50 = load ptr, ptr %4, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = load i32, ptr %5, align 4
  %53 = call i32 @lua_toboolean(ptr noundef %51, i32 noundef %52)
  %54 = icmp ne i32 %53, 0
  %55 = zext i1 %54 to i64
  %56 = select i1 %54, ptr @.str.38, ptr @.str.39
  %57 = call ptr @lua_pushstring(ptr noundef %50, ptr noundef %56)
  br label %90

58:                                               ; preds = %24
  %59 = load ptr, ptr %4, align 8
  %60 = call ptr @lua_pushstring(ptr noundef %59, ptr noundef @.str.40)
  br label %90

61:                                               ; preds = %24
  %62 = load ptr, ptr %4, align 8
  %63 = load i32, ptr %5, align 4
  %64 = call i32 @luaL_getmetafield(ptr noundef %62, i32 noundef %63, ptr noundef @.str.12)
  store i32 %64, ptr %7, align 4
  %65 = load i32, ptr %7, align 4
  %66 = icmp eq i32 %65, 4
  br i1 %66, label %67, label %70

67:                                               ; preds = %61
  %68 = load ptr, ptr %4, align 8
  %69 = call ptr @lua_tolstring(ptr noundef %68, i32 noundef -1, ptr noundef null)
  br label %76

70:                                               ; preds = %61
  %71 = load ptr, ptr %4, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = load i32, ptr %5, align 4
  %74 = call i32 @lua_type(ptr noundef %72, i32 noundef %73)
  %75 = call ptr @lua_typename(ptr noundef %71, i32 noundef %74)
  br label %76

76:                                               ; preds = %70, %67
  %77 = phi ptr [ %69, %67 ], [ %75, %70 ]
  store ptr %77, ptr %8, align 8
  %78 = load ptr, ptr %4, align 8
  %79 = load ptr, ptr %8, align 8
  %80 = load ptr, ptr %4, align 8
  %81 = load i32, ptr %5, align 4
  %82 = call ptr @lua_topointer(ptr noundef %80, i32 noundef %81)
  %83 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %78, ptr noundef @.str.41, ptr noundef %79, ptr noundef %82)
  %84 = load i32, ptr %7, align 4
  %85 = icmp ne i32 %84, 0
  br i1 %85, label %86, label %89

86:                                               ; preds = %76
  %87 = load ptr, ptr %4, align 8
  call void @lua_rotate(ptr noundef %87, i32 noundef -2, i32 noundef -1)
  %88 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %88, i32 noundef -2)
  br label %89

89:                                               ; preds = %86, %76
  br label %90

90:                                               ; preds = %89, %58, %49, %46, %45
  br label %91

91:                                               ; preds = %90, %23
  %92 = load ptr, ptr %4, align 8
  %93 = load ptr, ptr %6, align 8
  %94 = call ptr @lua_tolstring(ptr noundef %92, i32 noundef -1, ptr noundef %93)
  ret ptr %94
}

declare i32 @lua_isstring(ptr noundef, i32 noundef) #1

declare i32 @lua_isinteger(ptr noundef, i32 noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

declare ptr @lua_topointer(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_setfuncs(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %6, align 4
  call void @luaL_checkstack(ptr noundef %8, i32 noundef %9, ptr noundef @.str.42)
  br label %10

10:                                               ; preds = %48, %3
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.luaL_Reg, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %51

15:                                               ; preds = %10
  %16 = load ptr, ptr %5, align 8
  %17 = getelementptr inbounds %struct.luaL_Reg, ptr %16, i32 0, i32 1
  %18 = load ptr, ptr %17, align 8
  %19 = icmp eq ptr %18, null
  br i1 %19, label %20, label %22

20:                                               ; preds = %15
  %21 = load ptr, ptr %4, align 8
  call void @lua_pushboolean(ptr noundef %21, i32 noundef 0)
  br label %40

22:                                               ; preds = %15
  store i32 0, ptr %7, align 4
  br label %23

23:                                               ; preds = %31, %22
  %24 = load i32, ptr %7, align 4
  %25 = load i32, ptr %6, align 4
  %26 = icmp slt i32 %24, %25
  br i1 %26, label %27, label %34

27:                                               ; preds = %23
  %28 = load ptr, ptr %4, align 8
  %29 = load i32, ptr %6, align 4
  %30 = sub nsw i32 0, %29
  call void @lua_pushvalue(ptr noundef %28, i32 noundef %30)
  br label %31

31:                                               ; preds = %27
  %32 = load i32, ptr %7, align 4
  %33 = add nsw i32 %32, 1
  store i32 %33, ptr %7, align 4
  br label %23, !llvm.loop !12

34:                                               ; preds = %23
  %35 = load ptr, ptr %4, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.luaL_Reg, ptr %36, i32 0, i32 1
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %6, align 4
  call void @lua_pushcclosure(ptr noundef %35, ptr noundef %38, i32 noundef %39)
  br label %40

40:                                               ; preds = %34, %20
  %41 = load ptr, ptr %4, align 8
  %42 = load i32, ptr %6, align 4
  %43 = add nsw i32 %42, 2
  %44 = sub nsw i32 0, %43
  %45 = load ptr, ptr %5, align 8
  %46 = getelementptr inbounds %struct.luaL_Reg, ptr %45, i32 0, i32 0
  %47 = load ptr, ptr %46, align 8
  call void @lua_setfield(ptr noundef %41, i32 noundef %44, ptr noundef %47)
  br label %48

48:                                               ; preds = %40
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.luaL_Reg, ptr %49, i32 1
  store ptr %50, ptr %5, align 8
  br label %10, !llvm.loop !13

51:                                               ; preds = %10
  %52 = load ptr, ptr %4, align 8
  %53 = load i32, ptr %6, align 4
  %54 = sub nsw i32 0, %53
  %55 = sub nsw i32 %54, 1
  call void @lua_settop(ptr noundef %52, i32 noundef %55)
  ret void
}

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaL_getsubtable(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  %10 = load ptr, ptr %7, align 8
  %11 = call i32 @lua_getfield(ptr noundef %8, i32 noundef %9, ptr noundef %10)
  %12 = icmp eq i32 %11, 5
  br i1 %12, label %13, label %14

13:                                               ; preds = %3
  store i32 1, ptr %4, align 4
  br label %24

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %15, i32 noundef -2)
  %16 = load ptr, ptr %5, align 8
  %17 = load i32, ptr %6, align 4
  %18 = call i32 @lua_absindex(ptr noundef %16, i32 noundef %17)
  store i32 %18, ptr %6, align 4
  %19 = load ptr, ptr %5, align 8
  call void @lua_createtable(ptr noundef %19, i32 noundef 0, i32 noundef 0)
  %20 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %20, i32 noundef -1)
  %21 = load ptr, ptr %5, align 8
  %22 = load i32, ptr %6, align 4
  %23 = load ptr, ptr %7, align 8
  call void @lua_setfield(ptr noundef %21, i32 noundef %22, ptr noundef %23)
  store i32 0, ptr %4, align 4
  br label %24

24:                                               ; preds = %14, %13
  %25 = load i32, ptr %4, align 4
  ret i32 %25
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_requiref(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = call i32 @luaL_getsubtable(ptr noundef %9, i32 noundef -1001000, ptr noundef @.str.43)
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = call i32 @lua_getfield(ptr noundef %11, i32 noundef -1, ptr noundef %12)
  %14 = load ptr, ptr %5, align 8
  %15 = call i32 @lua_toboolean(ptr noundef %14, i32 noundef -1)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %28, label %17

17:                                               ; preds = %4
  %18 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %18, i32 noundef -2)
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %7, align 8
  call void @lua_pushcclosure(ptr noundef %19, ptr noundef %20, i32 noundef 0)
  %21 = load ptr, ptr %5, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = call ptr @lua_pushstring(ptr noundef %21, ptr noundef %22)
  %24 = load ptr, ptr %5, align 8
  call void @lua_callk(ptr noundef %24, i32 noundef 1, i32 noundef 1, i64 noundef 0, ptr noundef null)
  %25 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %25, i32 noundef -1)
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %6, align 8
  call void @lua_setfield(ptr noundef %26, i32 noundef -3, ptr noundef %27)
  br label %28

28:                                               ; preds = %17, %4
  %29 = load ptr, ptr %5, align 8
  call void @lua_rotate(ptr noundef %29, i32 noundef -2, i32 noundef -1)
  %30 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %30, i32 noundef -2)
  %31 = load i32, ptr %8, align 4
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %37

33:                                               ; preds = %28
  %34 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %34, i32 noundef -1)
  %35 = load ptr, ptr %5, align 8
  %36 = load ptr, ptr %6, align 8
  call void @lua_setglobal(ptr noundef %35, ptr noundef %36)
  br label %37

37:                                               ; preds = %33, %28
  ret void
}

declare void @lua_setglobal(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_addgsub(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = call i64 @strlen(ptr noundef %11) #8
  store i64 %12, ptr %10, align 8
  br label %13

13:                                               ; preds = %18, %4
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = call ptr @strstr(ptr noundef %14, ptr noundef %15) #8
  store ptr %16, ptr %9, align 8
  %17 = icmp ne ptr %16, null
  br i1 %17, label %18, label %31

18:                                               ; preds = %13
  %19 = load ptr, ptr %5, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = ptrtoint ptr %21 to i64
  %24 = ptrtoint ptr %22 to i64
  %25 = sub i64 %23, %24
  call void @luaL_addlstring(ptr noundef %19, ptr noundef %20, i64 noundef %25)
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %8, align 8
  call void @luaL_addstring(ptr noundef %26, ptr noundef %27)
  %28 = load ptr, ptr %9, align 8
  %29 = load i64, ptr %10, align 8
  %30 = getelementptr inbounds i8, ptr %28, i64 %29
  store ptr %30, ptr %6, align 8
  br label %13, !llvm.loop !14

31:                                               ; preds = %13
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %6, align 8
  call void @luaL_addstring(ptr noundef %32, ptr noundef %33)
  ret void
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strstr(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_gsub(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  call void @luaL_buffinit(ptr noundef %10, ptr noundef %9)
  %11 = load ptr, ptr %6, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = load ptr, ptr %8, align 8
  call void @luaL_addgsub(ptr noundef %9, ptr noundef %11, ptr noundef %12, ptr noundef %13)
  call void @luaL_pushresult(ptr noundef %9)
  %14 = load ptr, ptr %5, align 8
  %15 = call ptr @lua_tolstring(ptr noundef %14, i32 noundef -1, ptr noundef null)
  ret ptr %15
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @luaL_newstate() #0 {
  %1 = alloca ptr, align 8
  %2 = call ptr @lua_newstate(ptr noundef @l_alloc, ptr noundef null)
  store ptr %2, ptr %1, align 8
  %3 = load ptr, ptr %1, align 8
  %4 = icmp ne ptr %3, null
  %5 = zext i1 %4 to i32
  %6 = sext i32 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %8, label %13

8:                                                ; preds = %0
  %9 = load ptr, ptr %1, align 8
  %10 = call ptr @lua_atpanic(ptr noundef %9, ptr noundef @panic)
  %11 = load ptr, ptr %1, align 8
  %12 = load ptr, ptr %1, align 8
  call void @lua_setwarnf(ptr noundef %11, ptr noundef @warnfoff, ptr noundef %12)
  br label %13

13:                                               ; preds = %8, %0
  %14 = load ptr, ptr %1, align 8
  ret ptr %14
}

declare ptr @lua_newstate(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @l_alloc(ptr noundef %0, ptr noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i64 %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load i64, ptr %8, align 8
  %12 = load i64, ptr %9, align 8
  %13 = icmp eq i64 %12, 0
  br i1 %13, label %14, label %16

14:                                               ; preds = %4
  %15 = load ptr, ptr %7, align 8
  call void @free(ptr noundef %15) #10
  store ptr null, ptr %5, align 8
  br label %20

16:                                               ; preds = %4
  %17 = load ptr, ptr %7, align 8
  %18 = load i64, ptr %9, align 8
  %19 = call ptr @realloc(ptr noundef %17, i64 noundef %18) #11
  store ptr %19, ptr %5, align 8
  br label %20

20:                                               ; preds = %16, %14
  %21 = load ptr, ptr %5, align 8
  ret ptr %21
}

declare ptr @lua_atpanic(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @panic(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef -1)
  %6 = icmp eq i32 %5, 4
  br i1 %6, label %7, label %10

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @lua_tolstring(ptr noundef %8, i32 noundef -1, ptr noundef null)
  br label %11

10:                                               ; preds = %1
  br label %11

11:                                               ; preds = %10, %7
  %12 = phi ptr [ %9, %7 ], [ @.str.62, %10 ]
  store ptr %12, ptr %3, align 8
  %13 = load ptr, ptr @stderr, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %13, ptr noundef @.str.63, ptr noundef %14)
  %16 = load ptr, ptr @stderr, align 8
  %17 = call i32 @fflush(ptr noundef %16)
  ret i32 0
}

declare void @lua_setwarnf(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @warnfoff(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  %10 = call i32 @checkcontrol(ptr noundef %7, ptr noundef %8, i32 noundef %9)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @luaL_checkversion_(ptr noundef %0, double noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca double, align 8
  %6 = alloca i64, align 8
  %7 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  store double %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call double @lua_version(ptr noundef %8)
  store double %9, ptr %7, align 8
  %10 = load i64, ptr %6, align 8
  %11 = icmp ne i64 %10, 136
  br i1 %11, label %12, label %15

12:                                               ; preds = %3
  %13 = load ptr, ptr %4, align 8
  %14 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %13, ptr noundef @.str.44)
  br label %25

15:                                               ; preds = %3
  %16 = load double, ptr %7, align 8
  %17 = load double, ptr %5, align 8
  %18 = fcmp une double %16, %17
  br i1 %18, label %19, label %24

19:                                               ; preds = %15
  %20 = load ptr, ptr %4, align 8
  %21 = load double, ptr %5, align 8
  %22 = load double, ptr %7, align 8
  %23 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %20, ptr noundef @.str.45, double noundef %21, double noundef %22)
  br label %24

24:                                               ; preds = %19, %15
  br label %25

25:                                               ; preds = %24, %12
  ret void
}

declare double @lua_version(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @findfield(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %8 = load i32, ptr %7, align 4
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %14, label %10

10:                                               ; preds = %3
  %11 = load ptr, ptr %5, align 8
  %12 = call i32 @lua_type(ptr noundef %11, i32 noundef -1)
  %13 = icmp eq i32 %12, 5
  br i1 %13, label %15, label %14

14:                                               ; preds = %10, %3
  store i32 0, ptr %4, align 4
  br label %50

15:                                               ; preds = %10
  %16 = load ptr, ptr %5, align 8
  call void @lua_pushnil(ptr noundef %16)
  br label %17

17:                                               ; preds = %47, %15
  %18 = load ptr, ptr %5, align 8
  %19 = call i32 @lua_next(ptr noundef %18, i32 noundef -2)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %49

21:                                               ; preds = %17
  %22 = load ptr, ptr %5, align 8
  %23 = call i32 @lua_type(ptr noundef %22, i32 noundef -2)
  %24 = icmp eq i32 %23, 4
  br i1 %24, label %25, label %47

25:                                               ; preds = %21
  %26 = load ptr, ptr %5, align 8
  %27 = load i32, ptr %6, align 4
  %28 = call i32 @lua_rawequal(ptr noundef %26, i32 noundef %27, i32 noundef -1)
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %30, label %32

30:                                               ; preds = %25
  %31 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %31, i32 noundef -2)
  store i32 1, ptr %4, align 4
  br label %50

32:                                               ; preds = %25
  %33 = load ptr, ptr %5, align 8
  %34 = load i32, ptr %6, align 4
  %35 = load i32, ptr %7, align 4
  %36 = sub nsw i32 %35, 1
  %37 = call i32 @findfield(ptr noundef %33, i32 noundef %34, i32 noundef %36)
  %38 = icmp ne i32 %37, 0
  br i1 %38, label %39, label %45

39:                                               ; preds = %32
  %40 = load ptr, ptr %5, align 8
  %41 = call ptr @lua_pushstring(ptr noundef %40, ptr noundef @.str.53)
  %42 = load ptr, ptr %5, align 8
  call void @lua_copy(ptr noundef %42, i32 noundef -1, i32 noundef -3)
  %43 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %43, i32 noundef -2)
  %44 = load ptr, ptr %5, align 8
  call void @lua_concat(ptr noundef %44, i32 noundef 3)
  store i32 1, ptr %4, align 4
  br label %50

45:                                               ; preds = %32
  br label %46

46:                                               ; preds = %45
  br label %47

47:                                               ; preds = %46, %21
  %48 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %48, i32 noundef -2)
  br label %17, !llvm.loop !15

49:                                               ; preds = %17
  store i32 0, ptr %4, align 4
  br label %50

50:                                               ; preds = %49, %39, %30, %14
  %51 = load i32, ptr %4, align 4
  ret i32 %51
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strncmp(ptr noundef, ptr noundef, i64 noundef) #2

declare void @lua_copy(ptr noundef, i32 noundef, i32 noundef) #1

declare i32 @lua_next(ptr noundef, i32 noundef) #1

declare i32 @lua_isnumber(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @newbuffsize(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 1
  %9 = load i64, ptr %8, align 8
  %10 = udiv i64 %9, 2
  %11 = mul i64 %10, 3
  store i64 %11, ptr %6, align 8
  %12 = load i64, ptr %5, align 8
  %13 = sub i64 -1, %12
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 2
  %16 = load i64, ptr %15, align 8
  %17 = icmp ult i64 %13, %16
  %18 = zext i1 %17 to i32
  %19 = icmp ne i32 %18, 0
  %20 = zext i1 %19 to i32
  %21 = sext i32 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %23, label %29

23:                                               ; preds = %2
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.luaL_Buffer, ptr %24, i32 0, i32 3
  %26 = load ptr, ptr %25, align 8
  %27 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %26, ptr noundef @.str.55)
  %28 = sext i32 %27 to i64
  store i64 %28, ptr %3, align 8
  br label %45

29:                                               ; preds = %2
  %30 = load i64, ptr %6, align 8
  %31 = load ptr, ptr %4, align 8
  %32 = getelementptr inbounds %struct.luaL_Buffer, ptr %31, i32 0, i32 2
  %33 = load i64, ptr %32, align 8
  %34 = load i64, ptr %5, align 8
  %35 = add i64 %33, %34
  %36 = icmp ult i64 %30, %35
  br i1 %36, label %37, label %43

37:                                               ; preds = %29
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.luaL_Buffer, ptr %38, i32 0, i32 2
  %40 = load i64, ptr %39, align 8
  %41 = load i64, ptr %5, align 8
  %42 = add i64 %40, %41
  store i64 %42, ptr %6, align 8
  br label %43

43:                                               ; preds = %37, %29
  %44 = load i64, ptr %6, align 8
  store i64 %44, ptr %3, align 8
  br label %45

45:                                               ; preds = %43, %23
  %46 = load i64, ptr %3, align 8
  ret i64 %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @resizebox(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @lua_getallocf(ptr noundef %11, ptr noundef %7)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call ptr @lua_touserdata(ptr noundef %13, i32 noundef %14)
  store ptr %15, ptr %9, align 8
  %16 = load ptr, ptr %8, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = getelementptr inbounds %struct.UBox, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = getelementptr inbounds %struct.UBox, ptr %21, i32 0, i32 1
  %23 = load i64, ptr %22, align 8
  %24 = load i64, ptr %6, align 8
  %25 = call ptr %16(ptr noundef %17, ptr noundef %20, i64 noundef %23, i64 noundef %24)
  store ptr %25, ptr %10, align 8
  %26 = load ptr, ptr %10, align 8
  %27 = icmp eq ptr %26, null
  br i1 %27, label %28, label %31

28:                                               ; preds = %3
  %29 = load i64, ptr %6, align 8
  %30 = icmp ugt i64 %29, 0
  br label %31

31:                                               ; preds = %28, %3
  %32 = phi i1 [ false, %3 ], [ %30, %28 ]
  %33 = zext i1 %32 to i32
  %34 = icmp ne i32 %33, 0
  %35 = zext i1 %34 to i32
  %36 = sext i32 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %38, label %43

38:                                               ; preds = %31
  %39 = load ptr, ptr %4, align 8
  %40 = call ptr @lua_pushstring(ptr noundef %39, ptr noundef @.str.56)
  %41 = load ptr, ptr %4, align 8
  %42 = call i32 @lua_error(ptr noundef %41)
  br label %43

43:                                               ; preds = %38, %31
  %44 = load ptr, ptr %10, align 8
  %45 = load ptr, ptr %9, align 8
  %46 = getelementptr inbounds %struct.UBox, ptr %45, i32 0, i32 0
  store ptr %44, ptr %46, align 8
  %47 = load i64, ptr %6, align 8
  %48 = load ptr, ptr %9, align 8
  %49 = getelementptr inbounds %struct.UBox, ptr %48, i32 0, i32 1
  store i64 %47, ptr %49, align 8
  %50 = load ptr, ptr %10, align 8
  ret ptr %50
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @newbox(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @lua_newuserdatauv(ptr noundef %4, i64 noundef 16, i32 noundef 0)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.UBox, ptr %6, i32 0, i32 0
  store ptr null, ptr %7, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.UBox, ptr %8, i32 0, i32 1
  store i64 0, ptr %9, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = call i32 @luaL_newmetatable(ptr noundef %10, ptr noundef @.str.57)
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %13, label %15

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %14, ptr noundef @boxmt, i32 noundef 0)
  br label %15

15:                                               ; preds = %13, %1
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 @lua_setmetatable(ptr noundef %16, i32 noundef -2)
  ret void
}

declare void @lua_toclose(ptr noundef, i32 noundef) #1

declare ptr @lua_getallocf(ptr noundef, ptr noundef) #1

declare ptr @lua_newuserdatauv(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @boxgc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call ptr @resizebox(ptr noundef %3, i32 noundef 1, i64 noundef 0)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @skipBOM(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call i32 @getc(ptr noundef %5)
  store i32 %6, ptr %4, align 4
  %7 = load i32, ptr %4, align 4
  %8 = icmp eq i32 %7, 239
  br i1 %8, label %9, label %20

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @getc(ptr noundef %10)
  %12 = icmp eq i32 %11, 187
  br i1 %12, label %13, label %20

13:                                               ; preds = %9
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @getc(ptr noundef %14)
  %16 = icmp eq i32 %15, 191
  br i1 %16, label %17, label %20

17:                                               ; preds = %13
  %18 = load ptr, ptr %3, align 8
  %19 = call i32 @getc(ptr noundef %18)
  store i32 %19, ptr %2, align 4
  br label %22

20:                                               ; preds = %13, %9, %1
  %21 = load i32, ptr %4, align 4
  store i32 %21, ptr %2, align 4
  br label %22

22:                                               ; preds = %20, %17
  %23 = load i32, ptr %2, align 4
  ret i32 %23
}

declare i32 @getc(ptr noundef) #1

; Function Attrs: nounwind
declare i32 @feof(ptr noundef) #5

declare i64 @fread(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

; Function Attrs: nounwind
declare void @free(ptr noundef) #5

; Function Attrs: nounwind allocsize(1)
declare ptr @realloc(ptr noundef, i64 noundef) #7

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #1

declare i32 @fflush(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @checkcontrol(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %8 = load i32, ptr %7, align 4
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %16, label %10

10:                                               ; preds = %3
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds i8, ptr %11, i32 1
  store ptr %12, ptr %6, align 8
  %13 = load i8, ptr %11, align 1
  %14 = sext i8 %13 to i32
  %15 = icmp ne i32 %14, 64
  br i1 %15, label %16, label %17

16:                                               ; preds = %10, %3
  store i32 0, ptr %4, align 4
  br label %33

17:                                               ; preds = %10
  %18 = load ptr, ptr %6, align 8
  %19 = call i32 @strcmp(ptr noundef %18, ptr noundef @.str.64) #8
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %21, label %24

21:                                               ; preds = %17
  %22 = load ptr, ptr %5, align 8
  %23 = load ptr, ptr %5, align 8
  call void @lua_setwarnf(ptr noundef %22, ptr noundef @warnfoff, ptr noundef %23)
  br label %32

24:                                               ; preds = %17
  %25 = load ptr, ptr %6, align 8
  %26 = call i32 @strcmp(ptr noundef %25, ptr noundef @.str.65) #8
  %27 = icmp eq i32 %26, 0
  br i1 %27, label %28, label %31

28:                                               ; preds = %24
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %5, align 8
  call void @lua_setwarnf(ptr noundef %29, ptr noundef @warnfon, ptr noundef %30)
  br label %31

31:                                               ; preds = %28, %24
  br label %32

32:                                               ; preds = %31, %21
  store i32 1, ptr %4, align 4
  br label %33

33:                                               ; preds = %32, %16
  %34 = load i32, ptr %4, align 4
  ret i32 %34
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @warnfon(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load i32, ptr %6, align 4
  %10 = call i32 @checkcontrol(ptr noundef %7, ptr noundef %8, i32 noundef %9)
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %12, label %13

12:                                               ; preds = %3
  br label %21

13:                                               ; preds = %3
  %14 = load ptr, ptr @stderr, align 8
  %15 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %14, ptr noundef @.str.66, ptr noundef @.str.67)
  %16 = load ptr, ptr @stderr, align 8
  %17 = call i32 @fflush(ptr noundef %16)
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = load i32, ptr %6, align 4
  call void @warnfcont(ptr noundef %18, ptr noundef %19, i32 noundef %20)
  br label %21

21:                                               ; preds = %13, %12
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @warnfcont(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  store ptr %8, ptr %7, align 8
  %9 = load ptr, ptr @stderr, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %9, ptr noundef @.str.66, ptr noundef %10)
  %12 = load ptr, ptr @stderr, align 8
  %13 = call i32 @fflush(ptr noundef %12)
  %14 = load i32, ptr %6, align 4
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %19

16:                                               ; preds = %3
  %17 = load ptr, ptr %7, align 8
  %18 = load ptr, ptr %7, align 8
  call void @lua_setwarnf(ptr noundef %17, ptr noundef @warnfcont, ptr noundef %18)
  br label %26

19:                                               ; preds = %3
  %20 = load ptr, ptr @stderr, align 8
  %21 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %20, ptr noundef @.str.66, ptr noundef @.str.68)
  %22 = load ptr, ptr @stderr, align 8
  %23 = call i32 @fflush(ptr noundef %22)
  %24 = load ptr, ptr %7, align 8
  %25 = load ptr, ptr %7, align 8
  call void @lua_setwarnf(ptr noundef %24, ptr noundef @warnfon, ptr noundef %25)
  br label %26

26:                                               ; preds = %19, %16
  ret void
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nosync nounwind willreturn }
attributes #4 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #7 = { nounwind allocsize(1) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #8 = { nounwind willreturn memory(read) }
attributes #9 = { nounwind willreturn memory(none) }
attributes #10 = { nounwind }
attributes #11 = { nounwind allocsize(1) }

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
