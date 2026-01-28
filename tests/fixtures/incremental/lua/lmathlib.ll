; ModuleID = 'lmathlib.c'
source_filename = "lmathlib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.RanState = type { [4 x i64] }

@mathlib = internal constant [28 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.4, ptr @math_abs }, %struct.luaL_Reg { ptr @.str.5, ptr @math_acos }, %struct.luaL_Reg { ptr @.str.6, ptr @math_asin }, %struct.luaL_Reg { ptr @.str.7, ptr @math_atan }, %struct.luaL_Reg { ptr @.str.8, ptr @math_ceil }, %struct.luaL_Reg { ptr @.str.9, ptr @math_cos }, %struct.luaL_Reg { ptr @.str.10, ptr @math_deg }, %struct.luaL_Reg { ptr @.str.11, ptr @math_exp }, %struct.luaL_Reg { ptr @.str.12, ptr @math_toint }, %struct.luaL_Reg { ptr @.str.13, ptr @math_floor }, %struct.luaL_Reg { ptr @.str.14, ptr @math_fmod }, %struct.luaL_Reg { ptr @.str.15, ptr @math_ult }, %struct.luaL_Reg { ptr @.str.16, ptr @math_log }, %struct.luaL_Reg { ptr @.str.17, ptr @math_max }, %struct.luaL_Reg { ptr @.str.18, ptr @math_min }, %struct.luaL_Reg { ptr @.str.19, ptr @math_modf }, %struct.luaL_Reg { ptr @.str.20, ptr @math_rad }, %struct.luaL_Reg { ptr @.str.21, ptr @math_sin }, %struct.luaL_Reg { ptr @.str.22, ptr @math_sqrt }, %struct.luaL_Reg { ptr @.str.23, ptr @math_tan }, %struct.luaL_Reg { ptr @.str.24, ptr @math_type }, %struct.luaL_Reg { ptr @.str.25, ptr null }, %struct.luaL_Reg { ptr @.str.26, ptr null }, %struct.luaL_Reg { ptr @.str, ptr null }, %struct.luaL_Reg { ptr @.str.1, ptr null }, %struct.luaL_Reg { ptr @.str.2, ptr null }, %struct.luaL_Reg { ptr @.str.3, ptr null }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [3 x i8] c"pi\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"huge\00", align 1
@.str.2 = private unnamed_addr constant [11 x i8] c"maxinteger\00", align 1
@.str.3 = private unnamed_addr constant [11 x i8] c"mininteger\00", align 1
@.str.4 = private unnamed_addr constant [4 x i8] c"abs\00", align 1
@.str.5 = private unnamed_addr constant [5 x i8] c"acos\00", align 1
@.str.6 = private unnamed_addr constant [5 x i8] c"asin\00", align 1
@.str.7 = private unnamed_addr constant [5 x i8] c"atan\00", align 1
@.str.8 = private unnamed_addr constant [5 x i8] c"ceil\00", align 1
@.str.9 = private unnamed_addr constant [4 x i8] c"cos\00", align 1
@.str.10 = private unnamed_addr constant [4 x i8] c"deg\00", align 1
@.str.11 = private unnamed_addr constant [4 x i8] c"exp\00", align 1
@.str.12 = private unnamed_addr constant [10 x i8] c"tointeger\00", align 1
@.str.13 = private unnamed_addr constant [6 x i8] c"floor\00", align 1
@.str.14 = private unnamed_addr constant [5 x i8] c"fmod\00", align 1
@.str.15 = private unnamed_addr constant [4 x i8] c"ult\00", align 1
@.str.16 = private unnamed_addr constant [4 x i8] c"log\00", align 1
@.str.17 = private unnamed_addr constant [4 x i8] c"max\00", align 1
@.str.18 = private unnamed_addr constant [4 x i8] c"min\00", align 1
@.str.19 = private unnamed_addr constant [5 x i8] c"modf\00", align 1
@.str.20 = private unnamed_addr constant [4 x i8] c"rad\00", align 1
@.str.21 = private unnamed_addr constant [4 x i8] c"sin\00", align 1
@.str.22 = private unnamed_addr constant [5 x i8] c"sqrt\00", align 1
@.str.23 = private unnamed_addr constant [4 x i8] c"tan\00", align 1
@.str.24 = private unnamed_addr constant [5 x i8] c"type\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"random\00", align 1
@.str.26 = private unnamed_addr constant [11 x i8] c"randomseed\00", align 1
@.str.27 = private unnamed_addr constant [5 x i8] c"zero\00", align 1
@.str.28 = private unnamed_addr constant [15 x i8] c"value expected\00", align 1
@.str.29 = private unnamed_addr constant [8 x i8] c"integer\00", align 1
@.str.30 = private unnamed_addr constant [6 x i8] c"float\00", align 1
@randfuncs = internal constant [3 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.25, ptr @math_random }, %struct.luaL_Reg { ptr @.str.26, ptr @math_randomseed }, %struct.luaL_Reg zeroinitializer], align 16
@.str.31 = private unnamed_addr constant [26 x i8] c"wrong number of arguments\00", align 1
@.str.32 = private unnamed_addr constant [18 x i8] c"interval is empty\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_math(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 27)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @mathlib, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @lua_pushnumber(ptr noundef %6, double noundef 0x400921FB54442D18)
  %7 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %7, i32 noundef -2, ptr noundef @.str)
  %8 = load ptr, ptr %2, align 8
  call void @lua_pushnumber(ptr noundef %8, double noundef 0x7FF0000000000000)
  %9 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %9, i32 noundef -2, ptr noundef @.str.1)
  %10 = load ptr, ptr %2, align 8
  call void @lua_pushinteger(ptr noundef %10, i64 noundef 9223372036854775807)
  %11 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %11, i32 noundef -2, ptr noundef @.str.2)
  %12 = load ptr, ptr %2, align 8
  call void @lua_pushinteger(ptr noundef %12, i64 noundef -9223372036854775808)
  %13 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %13, i32 noundef -2, ptr noundef @.str.3)
  %14 = load ptr, ptr %2, align 8
  call void @setrandfunc(ptr noundef %14)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

declare void @lua_pushnumber(ptr noundef, double noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setrandfunc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @lua_newuserdatauv(ptr noundef %4, i64 noundef 32, i32 noundef 0)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = load ptr, ptr %3, align 8
  call void @randseed(ptr noundef %6, ptr noundef %7)
  %8 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %8, i32 noundef -3)
  %9 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %9, ptr noundef @randfuncs, i32 noundef 1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_abs(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_isinteger(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %18

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = call i64 @lua_tointegerx(ptr noundef %8, i32 noundef 1, ptr noundef null)
  store i64 %9, ptr %3, align 8
  %10 = load i64, ptr %3, align 8
  %11 = icmp slt i64 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %7
  %13 = load i64, ptr %3, align 8
  %14 = sub i64 0, %13
  store i64 %14, ptr %3, align 8
  br label %15

15:                                               ; preds = %12, %7
  %16 = load ptr, ptr %2, align 8
  %17 = load i64, ptr %3, align 8
  call void @lua_pushinteger(ptr noundef %16, i64 noundef %17)
  br label %23

18:                                               ; preds = %1
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = call double @luaL_checknumber(ptr noundef %20, i32 noundef 1)
  %22 = call double @llvm.fabs.f64(double %21)
  call void @lua_pushnumber(ptr noundef %19, double noundef %22)
  br label %23

23:                                               ; preds = %18, %15
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_acos(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @acos(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_asin(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @asin(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_atan(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  %4 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call double @luaL_checknumber(ptr noundef %5, i32 noundef 1)
  store double %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call double @luaL_optnumber(ptr noundef %7, i32 noundef 2, double noundef 1.000000e+00)
  store double %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = load double, ptr %3, align 8
  %11 = load double, ptr %4, align 8
  %12 = call double @atan2(double noundef %10, double noundef %11) #4
  call void @lua_pushnumber(ptr noundef %9, double noundef %12)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_ceil(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_isinteger(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %9

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %8, i32 noundef 1)
  br label %15

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = call double @luaL_checknumber(ptr noundef %10, i32 noundef 1)
  %12 = call double @llvm.ceil.f64(double %11)
  store double %12, ptr %3, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = load double, ptr %3, align 8
  call void @pushnumint(ptr noundef %13, double noundef %14)
  br label %15

15:                                               ; preds = %9, %7
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_cos(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @cos(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_deg(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = fmul double %5, 0x404CA5DC1A63C1F8
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_exp(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @exp(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_toint(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i64 @lua_tointegerx(ptr noundef %5, i32 noundef 1, ptr noundef %3)
  store i64 %6, ptr %4, align 8
  %7 = load i32, ptr %3, align 4
  %8 = icmp ne i32 %7, 0
  %9 = zext i1 %8 to i32
  %10 = sext i32 %9 to i64
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %1
  %13 = load ptr, ptr %2, align 8
  %14 = load i64, ptr %4, align 8
  call void @lua_pushinteger(ptr noundef %13, i64 noundef %14)
  br label %18

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %16, i32 noundef 1)
  %17 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %17)
  br label %18

18:                                               ; preds = %15, %12
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_floor(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_isinteger(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %9

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %8, i32 noundef 1)
  br label %15

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = call double @luaL_checknumber(ptr noundef %10, i32 noundef 1)
  %12 = call double @llvm.floor.f64(double %11)
  store double %12, ptr %3, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = load double, ptr %3, align 8
  call void @pushnumint(ptr noundef %13, double noundef %14)
  br label %15

15:                                               ; preds = %9, %7
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_fmod(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_isinteger(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %40

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_isinteger(ptr noundef %8, i32 noundef 2)
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %40

11:                                               ; preds = %7
  %12 = load ptr, ptr %2, align 8
  %13 = call i64 @lua_tointegerx(ptr noundef %12, i32 noundef 2, ptr noundef null)
  store i64 %13, ptr %3, align 8
  %14 = load i64, ptr %3, align 8
  %15 = add i64 %14, 1
  %16 = icmp ule i64 %15, 1
  br i1 %16, label %17, label %33

17:                                               ; preds = %11
  %18 = load i64, ptr %3, align 8
  %19 = icmp ne i64 %18, 0
  %20 = zext i1 %19 to i32
  %21 = icmp ne i32 %20, 0
  %22 = zext i1 %21 to i32
  %23 = sext i32 %22 to i64
  %24 = icmp ne i64 %23, 0
  br i1 %24, label %29, label %25

25:                                               ; preds = %17
  %26 = load ptr, ptr %2, align 8
  %27 = call i32 @luaL_argerror(ptr noundef %26, i32 noundef 2, ptr noundef @.str.27)
  %28 = icmp ne i32 %27, 0
  br label %29

29:                                               ; preds = %25, %17
  %30 = phi i1 [ true, %17 ], [ %28, %25 ]
  %31 = zext i1 %30 to i32
  %32 = load ptr, ptr %2, align 8
  call void @lua_pushinteger(ptr noundef %32, i64 noundef 0)
  br label %39

33:                                               ; preds = %11
  %34 = load ptr, ptr %2, align 8
  %35 = load ptr, ptr %2, align 8
  %36 = call i64 @lua_tointegerx(ptr noundef %35, i32 noundef 1, ptr noundef null)
  %37 = load i64, ptr %3, align 8
  %38 = srem i64 %36, %37
  call void @lua_pushinteger(ptr noundef %34, i64 noundef %38)
  br label %39

39:                                               ; preds = %33, %29
  br label %47

40:                                               ; preds = %7, %1
  %41 = load ptr, ptr %2, align 8
  %42 = load ptr, ptr %2, align 8
  %43 = call double @luaL_checknumber(ptr noundef %42, i32 noundef 1)
  %44 = load ptr, ptr %2, align 8
  %45 = call double @luaL_checknumber(ptr noundef %44, i32 noundef 2)
  %46 = call double @fmod(double noundef %43, double noundef %45) #4
  call void @lua_pushnumber(ptr noundef %41, double noundef %46)
  br label %47

47:                                               ; preds = %40, %39
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_ult(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i64 @luaL_checkinteger(ptr noundef %5, i32 noundef 1)
  store i64 %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call i64 @luaL_checkinteger(ptr noundef %7, i32 noundef 2)
  store i64 %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = load i64, ptr %3, align 8
  %11 = load i64, ptr %4, align 8
  %12 = icmp ult i64 %10, %11
  %13 = zext i1 %12 to i32
  call void @lua_pushboolean(ptr noundef %9, i32 noundef %13)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_log(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  %4 = alloca double, align 8
  %5 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call double @luaL_checknumber(ptr noundef %6, i32 noundef 1)
  store double %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 2)
  %10 = icmp sle i32 %9, 0
  br i1 %10, label %11, label %14

11:                                               ; preds = %1
  %12 = load double, ptr %3, align 8
  %13 = call double @log(double noundef %12) #4
  store double %13, ptr %4, align 8
  br label %36

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = call double @luaL_checknumber(ptr noundef %15, i32 noundef 2)
  store double %16, ptr %5, align 8
  %17 = load double, ptr %5, align 8
  %18 = fcmp oeq double %17, 2.000000e+00
  br i1 %18, label %19, label %22

19:                                               ; preds = %14
  %20 = load double, ptr %3, align 8
  %21 = call double @log2(double noundef %20) #4
  store double %21, ptr %4, align 8
  br label %35

22:                                               ; preds = %14
  %23 = load double, ptr %5, align 8
  %24 = fcmp oeq double %23, 1.000000e+01
  br i1 %24, label %25, label %28

25:                                               ; preds = %22
  %26 = load double, ptr %3, align 8
  %27 = call double @log10(double noundef %26) #4
  store double %27, ptr %4, align 8
  br label %34

28:                                               ; preds = %22
  %29 = load double, ptr %3, align 8
  %30 = call double @log(double noundef %29) #4
  %31 = load double, ptr %5, align 8
  %32 = call double @log(double noundef %31) #4
  %33 = fdiv double %30, %32
  store double %33, ptr %4, align 8
  br label %34

34:                                               ; preds = %28, %25
  br label %35

35:                                               ; preds = %34, %19
  br label %36

36:                                               ; preds = %35, %11
  %37 = load ptr, ptr %2, align 8
  %38 = load double, ptr %4, align 8
  call void @lua_pushnumber(ptr noundef %37, double noundef %38)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_max(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_gettop(ptr noundef %6)
  store i32 %7, ptr %3, align 4
  store i32 1, ptr %4, align 4
  %8 = load i32, ptr %3, align 4
  %9 = icmp sge i32 %8, 1
  %10 = zext i1 %9 to i32
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %19, label %15

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 @luaL_argerror(ptr noundef %16, i32 noundef 1, ptr noundef @.str.28)
  %18 = icmp ne i32 %17, 0
  br label %19

19:                                               ; preds = %15, %1
  %20 = phi i1 [ true, %1 ], [ %18, %15 ]
  %21 = zext i1 %20 to i32
  store i32 2, ptr %5, align 4
  br label %22

22:                                               ; preds = %35, %19
  %23 = load i32, ptr %5, align 4
  %24 = load i32, ptr %3, align 4
  %25 = icmp sle i32 %23, %24
  br i1 %25, label %26, label %38

26:                                               ; preds = %22
  %27 = load ptr, ptr %2, align 8
  %28 = load i32, ptr %4, align 4
  %29 = load i32, ptr %5, align 4
  %30 = call i32 @lua_compare(ptr noundef %27, i32 noundef %28, i32 noundef %29, i32 noundef 1)
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %34

32:                                               ; preds = %26
  %33 = load i32, ptr %5, align 4
  store i32 %33, ptr %4, align 4
  br label %34

34:                                               ; preds = %32, %26
  br label %35

35:                                               ; preds = %34
  %36 = load i32, ptr %5, align 4
  %37 = add nsw i32 %36, 1
  store i32 %37, ptr %5, align 4
  br label %22, !llvm.loop !6

38:                                               ; preds = %22
  %39 = load ptr, ptr %2, align 8
  %40 = load i32, ptr %4, align 4
  call void @lua_pushvalue(ptr noundef %39, i32 noundef %40)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_min(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_gettop(ptr noundef %6)
  store i32 %7, ptr %3, align 4
  store i32 1, ptr %4, align 4
  %8 = load i32, ptr %3, align 4
  %9 = icmp sge i32 %8, 1
  %10 = zext i1 %9 to i32
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %19, label %15

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 @luaL_argerror(ptr noundef %16, i32 noundef 1, ptr noundef @.str.28)
  %18 = icmp ne i32 %17, 0
  br label %19

19:                                               ; preds = %15, %1
  %20 = phi i1 [ true, %1 ], [ %18, %15 ]
  %21 = zext i1 %20 to i32
  store i32 2, ptr %5, align 4
  br label %22

22:                                               ; preds = %35, %19
  %23 = load i32, ptr %5, align 4
  %24 = load i32, ptr %3, align 4
  %25 = icmp sle i32 %23, %24
  br i1 %25, label %26, label %38

26:                                               ; preds = %22
  %27 = load ptr, ptr %2, align 8
  %28 = load i32, ptr %5, align 4
  %29 = load i32, ptr %4, align 4
  %30 = call i32 @lua_compare(ptr noundef %27, i32 noundef %28, i32 noundef %29, i32 noundef 1)
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %34

32:                                               ; preds = %26
  %33 = load i32, ptr %5, align 4
  store i32 %33, ptr %4, align 4
  br label %34

34:                                               ; preds = %32, %26
  br label %35

35:                                               ; preds = %34
  %36 = load i32, ptr %5, align 4
  %37 = add nsw i32 %36, 1
  store i32 %37, ptr %5, align 4
  br label %22, !llvm.loop !8

38:                                               ; preds = %22
  %39 = load ptr, ptr %2, align 8
  %40 = load i32, ptr %4, align 4
  call void @lua_pushvalue(ptr noundef %39, i32 noundef %40)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_modf(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca double, align 8
  %4 = alloca double, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_isinteger(ptr noundef %5, i32 noundef 1)
  %7 = icmp ne i32 %6, 0
  br i1 %7, label %8, label %11

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %9, i32 noundef 1)
  %10 = load ptr, ptr %2, align 8
  call void @lua_pushnumber(ptr noundef %10, double noundef 0.000000e+00)
  br label %37

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  %13 = call double @luaL_checknumber(ptr noundef %12, i32 noundef 1)
  store double %13, ptr %3, align 8
  %14 = load double, ptr %3, align 8
  %15 = fcmp olt double %14, 0.000000e+00
  br i1 %15, label %16, label %19

16:                                               ; preds = %11
  %17 = load double, ptr %3, align 8
  %18 = call double @llvm.ceil.f64(double %17)
  br label %22

19:                                               ; preds = %11
  %20 = load double, ptr %3, align 8
  %21 = call double @llvm.floor.f64(double %20)
  br label %22

22:                                               ; preds = %19, %16
  %23 = phi double [ %18, %16 ], [ %21, %19 ]
  store double %23, ptr %4, align 8
  %24 = load ptr, ptr %2, align 8
  %25 = load double, ptr %4, align 8
  call void @pushnumint(ptr noundef %24, double noundef %25)
  %26 = load ptr, ptr %2, align 8
  %27 = load double, ptr %3, align 8
  %28 = load double, ptr %4, align 8
  %29 = fcmp oeq double %27, %28
  br i1 %29, label %30, label %31

30:                                               ; preds = %22
  br label %35

31:                                               ; preds = %22
  %32 = load double, ptr %3, align 8
  %33 = load double, ptr %4, align 8
  %34 = fsub double %32, %33
  br label %35

35:                                               ; preds = %31, %30
  %36 = phi double [ 0.000000e+00, %30 ], [ %34, %31 ]
  call void @lua_pushnumber(ptr noundef %26, double noundef %36)
  br label %37

37:                                               ; preds = %35, %8
  ret i32 2
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_rad(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = fmul double %5, 0x3F91DF46A2529D39
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_sin(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @sin(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_sqrt(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @sqrt(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_tan(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call double @luaL_checknumber(ptr noundef %4, i32 noundef 1)
  %6 = call double @tan(double noundef %5) #4
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_type(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @lua_type(ptr noundef %3, i32 noundef 1)
  %5 = icmp eq i32 %4, 3
  br i1 %5, label %6, label %14

6:                                                ; preds = %1
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_isinteger(ptr noundef %8, i32 noundef 1)
  %10 = icmp ne i32 %9, 0
  %11 = zext i1 %10 to i64
  %12 = select i1 %10, ptr @.str.29, ptr @.str.30
  %13 = call ptr @lua_pushstring(ptr noundef %7, ptr noundef %12)
  br label %17

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %15, i32 noundef 1)
  %16 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %16)
  br label %17

17:                                               ; preds = %14, %6
  ret i32 1
}

declare i32 @lua_isinteger(ptr noundef, i32 noundef) #1

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

declare double @luaL_checknumber(ptr noundef, i32 noundef) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.fabs.f64(double) #2

; Function Attrs: nounwind
declare double @acos(double noundef) #3

; Function Attrs: nounwind
declare double @asin(double noundef) #3

declare double @luaL_optnumber(ptr noundef, i32 noundef, double noundef) #1

; Function Attrs: nounwind
declare double @atan2(double noundef, double noundef) #3

declare void @lua_settop(ptr noundef, i32 noundef) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.ceil.f64(double) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pushnumint(ptr noundef %0, double noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca double, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store double %1, ptr %4, align 8
  %6 = load double, ptr %4, align 8
  %7 = fcmp oge double %6, 0xC3E0000000000000
  br i1 %7, label %8, label %17

8:                                                ; preds = %2
  %9 = load double, ptr %4, align 8
  %10 = fcmp olt double %9, 0x43E0000000000000
  br i1 %10, label %11, label %17

11:                                               ; preds = %8
  %12 = load double, ptr %4, align 8
  %13 = fptosi double %12 to i64
  store i64 %13, ptr %5, align 8
  br i1 true, label %14, label %17

14:                                               ; preds = %11
  %15 = load ptr, ptr %3, align 8
  %16 = load i64, ptr %5, align 8
  call void @lua_pushinteger(ptr noundef %15, i64 noundef %16)
  br label %20

17:                                               ; preds = %11, %8, %2
  %18 = load ptr, ptr %3, align 8
  %19 = load double, ptr %4, align 8
  call void @lua_pushnumber(ptr noundef %18, double noundef %19)
  br label %20

20:                                               ; preds = %17, %14
  ret void
}

; Function Attrs: nounwind
declare double @cos(double noundef) #3

; Function Attrs: nounwind
declare double @exp(double noundef) #3

declare void @luaL_checkany(ptr noundef, i32 noundef) #1

declare void @lua_pushnil(ptr noundef) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.floor.f64(double) #2

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: nounwind
declare double @fmod(double noundef, double noundef) #3

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind
declare double @log(double noundef) #3

; Function Attrs: nounwind
declare double @log2(double noundef) #3

; Function Attrs: nounwind
declare double @log10(double noundef) #3

declare i32 @lua_gettop(ptr noundef) #1

declare i32 @lua_compare(ptr noundef, i32 noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind
declare double @sin(double noundef) #3

; Function Attrs: nounwind
declare double @sqrt(double noundef) #3

; Function Attrs: nounwind
declare double @tan(double noundef) #3

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare ptr @lua_newuserdatauv(ptr noundef, i64 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @randseed(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = call i64 @time(ptr noundef null) #4
  store i64 %7, ptr %5, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = ptrtoint ptr %8 to i64
  store i64 %9, ptr %6, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.RanState, ptr %11, i32 0, i32 0
  %13 = getelementptr inbounds [4 x i64], ptr %12, i64 0, i64 0
  %14 = load i64, ptr %5, align 8
  %15 = load i64, ptr %6, align 8
  call void @setseed(ptr noundef %10, ptr noundef %13, i64 noundef %14, i64 noundef %15)
  ret void
}

; Function Attrs: nounwind
declare i64 @time(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setseed(ptr noundef %0, ptr noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  store i64 %3, ptr %8, align 8
  %10 = load i64, ptr %7, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds i64, ptr %11, i64 0
  store i64 %10, ptr %12, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = getelementptr inbounds i64, ptr %13, i64 1
  store i64 255, ptr %14, align 8
  %15 = load i64, ptr %8, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds i64, ptr %16, i64 2
  store i64 %15, ptr %17, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds i64, ptr %18, i64 3
  store i64 0, ptr %19, align 8
  store i32 0, ptr %9, align 4
  br label %20

20:                                               ; preds = %26, %4
  %21 = load i32, ptr %9, align 4
  %22 = icmp slt i32 %21, 16
  br i1 %22, label %23, label %29

23:                                               ; preds = %20
  %24 = load ptr, ptr %6, align 8
  %25 = call i64 @nextrand(ptr noundef %24)
  br label %26

26:                                               ; preds = %23
  %27 = load i32, ptr %9, align 4
  %28 = add nsw i32 %27, 1
  store i32 %28, ptr %9, align 4
  br label %20, !llvm.loop !9

29:                                               ; preds = %20
  %30 = load ptr, ptr %5, align 8
  %31 = load i64, ptr %7, align 8
  call void @lua_pushinteger(ptr noundef %30, i64 noundef %31)
  %32 = load ptr, ptr %5, align 8
  %33 = load i64, ptr %8, align 8
  call void @lua_pushinteger(ptr noundef %32, i64 noundef %33)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @nextrand(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds i64, ptr %8, i64 0
  %10 = load i64, ptr %9, align 8
  store i64 %10, ptr %3, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds i64, ptr %11, i64 1
  %13 = load i64, ptr %12, align 8
  store i64 %13, ptr %4, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds i64, ptr %14, i64 2
  %16 = load i64, ptr %15, align 8
  %17 = load i64, ptr %3, align 8
  %18 = xor i64 %16, %17
  store i64 %18, ptr %5, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds i64, ptr %19, i64 3
  %21 = load i64, ptr %20, align 8
  %22 = load i64, ptr %4, align 8
  %23 = xor i64 %21, %22
  store i64 %23, ptr %6, align 8
  %24 = load i64, ptr %4, align 8
  %25 = mul i64 %24, 5
  %26 = call i64 @rotl(i64 noundef %25, i32 noundef 7)
  %27 = mul i64 %26, 9
  store i64 %27, ptr %7, align 8
  %28 = load i64, ptr %3, align 8
  %29 = load i64, ptr %6, align 8
  %30 = xor i64 %28, %29
  %31 = load ptr, ptr %2, align 8
  %32 = getelementptr inbounds i64, ptr %31, i64 0
  store i64 %30, ptr %32, align 8
  %33 = load i64, ptr %4, align 8
  %34 = load i64, ptr %5, align 8
  %35 = xor i64 %33, %34
  %36 = load ptr, ptr %2, align 8
  %37 = getelementptr inbounds i64, ptr %36, i64 1
  store i64 %35, ptr %37, align 8
  %38 = load i64, ptr %5, align 8
  %39 = load i64, ptr %4, align 8
  %40 = shl i64 %39, 17
  %41 = xor i64 %38, %40
  %42 = load ptr, ptr %2, align 8
  %43 = getelementptr inbounds i64, ptr %42, i64 2
  store i64 %41, ptr %43, align 8
  %44 = load i64, ptr %6, align 8
  %45 = call i64 @rotl(i64 noundef %44, i32 noundef 45)
  %46 = load ptr, ptr %2, align 8
  %47 = getelementptr inbounds i64, ptr %46, i64 3
  store i64 %45, ptr %47, align 8
  %48 = load i64, ptr %7, align 8
  ret i64 %48
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @rotl(i64 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca i32, align 4
  store i64 %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i64, ptr %3, align 8
  %6 = load i32, ptr %4, align 4
  %7 = zext i32 %6 to i64
  %8 = shl i64 %5, %7
  %9 = load i64, ptr %3, align 8
  %10 = and i64 %9, -1
  %11 = load i32, ptr %4, align 4
  %12 = sub nsw i32 64, %11
  %13 = zext i32 %12 to i64
  %14 = lshr i64 %10, %13
  %15 = or i64 %8, %14
  ret i64 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_random(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @lua_touserdata(ptr noundef %9, i32 noundef -1001001)
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = getelementptr inbounds %struct.RanState, ptr %11, i32 0, i32 0
  %13 = getelementptr inbounds [4 x i64], ptr %12, i64 0, i64 0
  %14 = call i64 @nextrand(ptr noundef %13)
  store i64 %14, ptr %8, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = call i32 @lua_gettop(ptr noundef %15)
  switch i32 %16, label %36 [
    i32 0, label %17
    i32 1, label %21
    i32 2, label %31
  ]

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  %19 = load i64, ptr %8, align 8
  %20 = call double @I2d(i64 noundef %19)
  call void @lua_pushnumber(ptr noundef %18, double noundef %20)
  store i32 1, ptr %2, align 4
  br label %66

21:                                               ; preds = %1
  store i64 1, ptr %4, align 8
  %22 = load ptr, ptr %3, align 8
  %23 = call i64 @luaL_checkinteger(ptr noundef %22, i32 noundef 1)
  store i64 %23, ptr %5, align 8
  %24 = load i64, ptr %5, align 8
  %25 = icmp eq i64 %24, 0
  br i1 %25, label %26, label %30

26:                                               ; preds = %21
  %27 = load ptr, ptr %3, align 8
  %28 = load i64, ptr %8, align 8
  %29 = and i64 %28, -1
  call void @lua_pushinteger(ptr noundef %27, i64 noundef %29)
  store i32 1, ptr %2, align 4
  br label %66

30:                                               ; preds = %21
  br label %39

31:                                               ; preds = %1
  %32 = load ptr, ptr %3, align 8
  %33 = call i64 @luaL_checkinteger(ptr noundef %32, i32 noundef 1)
  store i64 %33, ptr %4, align 8
  %34 = load ptr, ptr %3, align 8
  %35 = call i64 @luaL_checkinteger(ptr noundef %34, i32 noundef 2)
  store i64 %35, ptr %5, align 8
  br label %39

36:                                               ; preds = %1
  %37 = load ptr, ptr %3, align 8
  %38 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %37, ptr noundef @.str.31)
  store i32 %38, ptr %2, align 4
  br label %66

39:                                               ; preds = %31, %30
  %40 = load i64, ptr %4, align 8
  %41 = load i64, ptr %5, align 8
  %42 = icmp sle i64 %40, %41
  %43 = zext i1 %42 to i32
  %44 = icmp ne i32 %43, 0
  %45 = zext i1 %44 to i32
  %46 = sext i32 %45 to i64
  %47 = icmp ne i64 %46, 0
  br i1 %47, label %52, label %48

48:                                               ; preds = %39
  %49 = load ptr, ptr %3, align 8
  %50 = call i32 @luaL_argerror(ptr noundef %49, i32 noundef 1, ptr noundef @.str.32)
  %51 = icmp ne i32 %50, 0
  br label %52

52:                                               ; preds = %48, %39
  %53 = phi i1 [ true, %39 ], [ %51, %48 ]
  %54 = zext i1 %53 to i32
  %55 = load i64, ptr %8, align 8
  %56 = and i64 %55, -1
  %57 = load i64, ptr %5, align 8
  %58 = load i64, ptr %4, align 8
  %59 = sub i64 %57, %58
  %60 = load ptr, ptr %7, align 8
  %61 = call i64 @project(i64 noundef %56, i64 noundef %59, ptr noundef %60)
  store i64 %61, ptr %6, align 8
  %62 = load ptr, ptr %3, align 8
  %63 = load i64, ptr %6, align 8
  %64 = load i64, ptr %4, align 8
  %65 = add i64 %63, %64
  call void @lua_pushinteger(ptr noundef %62, i64 noundef %65)
  store i32 1, ptr %2, align 4
  br label %66

66:                                               ; preds = %52, %36, %26, %17
  %67 = load i32, ptr %2, align 4
  ret i32 %67
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @math_randomseed(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call ptr @lua_touserdata(ptr noundef %6, i32 noundef -1001001)
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 1)
  %10 = icmp eq i32 %9, -1
  br i1 %10, label %11, label %14

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  %13 = load ptr, ptr %3, align 8
  call void @randseed(ptr noundef %12, ptr noundef %13)
  br label %25

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = call i64 @luaL_checkinteger(ptr noundef %15, i32 noundef 1)
  store i64 %16, ptr %4, align 8
  %17 = load ptr, ptr %2, align 8
  %18 = call i64 @luaL_optinteger(ptr noundef %17, i32 noundef 2, i64 noundef 0)
  store i64 %18, ptr %5, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.RanState, ptr %20, i32 0, i32 0
  %22 = getelementptr inbounds [4 x i64], ptr %21, i64 0, i64 0
  %23 = load i64, ptr %4, align 8
  %24 = load i64, ptr %5, align 8
  call void @setseed(ptr noundef %19, ptr noundef %22, i64 noundef %23, i64 noundef %24)
  br label %25

25:                                               ; preds = %14, %11
  ret i32 2
}

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal double @I2d(i64 noundef %0) #0 {
  %2 = alloca i64, align 8
  %3 = alloca i64, align 8
  %4 = alloca double, align 8
  store i64 %0, ptr %2, align 8
  %5 = load i64, ptr %2, align 8
  %6 = and i64 %5, -1
  %7 = lshr i64 %6, 11
  store i64 %7, ptr %3, align 8
  %8 = load i64, ptr %3, align 8
  %9 = sitofp i64 %8 to double
  %10 = fmul double %9, 0x3CA0000000000000
  store double %10, ptr %4, align 8
  %11 = load i64, ptr %3, align 8
  %12 = icmp slt i64 %11, 0
  br i1 %12, label %13, label %16

13:                                               ; preds = %1
  %14 = load double, ptr %4, align 8
  %15 = fadd double %14, 1.000000e+00
  store double %15, ptr %4, align 8
  br label %16

16:                                               ; preds = %13, %1
  %17 = load double, ptr %4, align 8
  ret double %17
}

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @project(i64 noundef %0, i64 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  store i64 %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %9 = load i64, ptr %6, align 8
  %10 = load i64, ptr %6, align 8
  %11 = add i64 %10, 1
  %12 = and i64 %9, %11
  %13 = icmp eq i64 %12, 0
  br i1 %13, label %14, label %18

14:                                               ; preds = %3
  %15 = load i64, ptr %5, align 8
  %16 = load i64, ptr %6, align 8
  %17 = and i64 %15, %16
  store i64 %17, ptr %4, align 8
  br label %58

18:                                               ; preds = %3
  %19 = load i64, ptr %6, align 8
  store i64 %19, ptr %8, align 8
  %20 = load i64, ptr %8, align 8
  %21 = lshr i64 %20, 1
  %22 = load i64, ptr %8, align 8
  %23 = or i64 %22, %21
  store i64 %23, ptr %8, align 8
  %24 = load i64, ptr %8, align 8
  %25 = lshr i64 %24, 2
  %26 = load i64, ptr %8, align 8
  %27 = or i64 %26, %25
  store i64 %27, ptr %8, align 8
  %28 = load i64, ptr %8, align 8
  %29 = lshr i64 %28, 4
  %30 = load i64, ptr %8, align 8
  %31 = or i64 %30, %29
  store i64 %31, ptr %8, align 8
  %32 = load i64, ptr %8, align 8
  %33 = lshr i64 %32, 8
  %34 = load i64, ptr %8, align 8
  %35 = or i64 %34, %33
  store i64 %35, ptr %8, align 8
  %36 = load i64, ptr %8, align 8
  %37 = lshr i64 %36, 16
  %38 = load i64, ptr %8, align 8
  %39 = or i64 %38, %37
  store i64 %39, ptr %8, align 8
  %40 = load i64, ptr %8, align 8
  %41 = lshr i64 %40, 32
  %42 = load i64, ptr %8, align 8
  %43 = or i64 %42, %41
  store i64 %43, ptr %8, align 8
  br label %44

44:                                               ; preds = %50, %18
  %45 = load i64, ptr %8, align 8
  %46 = load i64, ptr %5, align 8
  %47 = and i64 %46, %45
  store i64 %47, ptr %5, align 8
  %48 = load i64, ptr %6, align 8
  %49 = icmp ugt i64 %47, %48
  br i1 %49, label %50, label %56

50:                                               ; preds = %44
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.RanState, ptr %51, i32 0, i32 0
  %53 = getelementptr inbounds [4 x i64], ptr %52, i64 0, i64 0
  %54 = call i64 @nextrand(ptr noundef %53)
  %55 = and i64 %54, -1
  store i64 %55, ptr %5, align 8
  br label %44, !llvm.loop !10

56:                                               ; preds = %44
  %57 = load i64, ptr %5, align 8
  store i64 %57, ptr %4, align 8
  br label %58

58:                                               ; preds = %56, %14
  %59 = load i64, ptr %4, align 8
  ret i64 %59
}

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #3 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind }

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
