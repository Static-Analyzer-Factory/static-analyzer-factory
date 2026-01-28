; ModuleID = 'lcorolib.c'
source_filename = "lcorolib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.lua_Debug = type { i32, ptr, ptr, ptr, ptr, i64, i32, i32, i32, i8, i8, i8, i8, i16, i16, [60 x i8], ptr }

@co_funcs = internal constant [9 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str, ptr @luaB_cocreate }, %struct.luaL_Reg { ptr @.str.1, ptr @luaB_coresume }, %struct.luaL_Reg { ptr @.str.2, ptr @luaB_corunning }, %struct.luaL_Reg { ptr @.str.3, ptr @luaB_costatus }, %struct.luaL_Reg { ptr @.str.4, ptr @luaB_cowrap }, %struct.luaL_Reg { ptr @.str.5, ptr @luaB_yield }, %struct.luaL_Reg { ptr @.str.6, ptr @luaB_yieldable }, %struct.luaL_Reg { ptr @.str.7, ptr @luaB_close }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [7 x i8] c"create\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"resume\00", align 1
@.str.2 = private unnamed_addr constant [8 x i8] c"running\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"status\00", align 1
@.str.4 = private unnamed_addr constant [5 x i8] c"wrap\00", align 1
@.str.5 = private unnamed_addr constant [6 x i8] c"yield\00", align 1
@.str.6 = private unnamed_addr constant [12 x i8] c"isyieldable\00", align 1
@.str.7 = private unnamed_addr constant [6 x i8] c"close\00", align 1
@.str.8 = private unnamed_addr constant [7 x i8] c"thread\00", align 1
@.str.9 = private unnamed_addr constant [29 x i8] c"too many arguments to resume\00", align 1
@.str.10 = private unnamed_addr constant [27 x i8] c"too many results to resume\00", align 1
@statname = internal constant [4 x ptr] [ptr @.str.2, ptr @.str.11, ptr @.str.12, ptr @.str.13], align 16
@.str.11 = private unnamed_addr constant [5 x i8] c"dead\00", align 1
@.str.12 = private unnamed_addr constant [10 x i8] c"suspended\00", align 1
@.str.13 = private unnamed_addr constant [7 x i8] c"normal\00", align 1
@.str.14 = private unnamed_addr constant [28 x i8] c"cannot close a %s coroutine\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_coroutine(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 8)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @co_funcs, i32 noundef 0)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_cocreate(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %4, i32 noundef 1, i32 noundef 6)
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @lua_newthread(ptr noundef %5)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %3, align 8
  call void @lua_xmove(ptr noundef %8, ptr noundef %9, i32 noundef 1)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_coresume(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @getco(ptr noundef %6)
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @lua_gettop(ptr noundef %10)
  %12 = sub nsw i32 %11, 1
  %13 = call i32 @auxresume(ptr noundef %8, ptr noundef %9, i32 noundef %12)
  store i32 %13, ptr %5, align 4
  %14 = load i32, ptr %5, align 4
  %15 = icmp slt i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = icmp ne i32 %16, 0
  %18 = zext i1 %17 to i32
  %19 = sext i32 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %21, label %24

21:                                               ; preds = %1
  %22 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %22, i32 noundef 0)
  %23 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %23, i32 noundef -2, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %32

24:                                               ; preds = %1
  %25 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %25, i32 noundef 1)
  %26 = load ptr, ptr %3, align 8
  %27 = load i32, ptr %5, align 4
  %28 = add nsw i32 %27, 1
  %29 = sub nsw i32 0, %28
  call void @lua_rotate(ptr noundef %26, i32 noundef %29, i32 noundef 1)
  %30 = load i32, ptr %5, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, ptr %2, align 4
  br label %32

32:                                               ; preds = %24, %21
  %33 = load i32, ptr %2, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_corunning(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_pushthread(ptr noundef %4)
  store i32 %5, ptr %3, align 4
  %6 = load ptr, ptr %2, align 8
  %7 = load i32, ptr %3, align 4
  call void @lua_pushboolean(ptr noundef %6, i32 noundef %7)
  ret i32 2
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_costatus(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @getco(ptr noundef %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @auxstatus(ptr noundef %7, ptr noundef %8)
  %10 = sext i32 %9 to i64
  %11 = getelementptr inbounds [4 x ptr], ptr @statname, i64 0, i64 %10
  %12 = load ptr, ptr %11, align 8
  %13 = call ptr @lua_pushstring(ptr noundef %6, ptr noundef %12)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_cowrap(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @luaB_cocreate(ptr noundef %3)
  %5 = load ptr, ptr %2, align 8
  call void @lua_pushcclosure(ptr noundef %5, ptr noundef @luaB_auxwrap, i32 noundef 1)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_yield(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_gettop(ptr noundef %4)
  %6 = call i32 @lua_yieldk(ptr noundef %3, i32 noundef %5, i64 noundef 0, ptr noundef null)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_yieldable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef 1)
  %6 = icmp eq i32 %5, -1
  br i1 %6, label %7, label %9

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  br label %12

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  %11 = call ptr @getco(ptr noundef %10)
  br label %12

12:                                               ; preds = %9, %7
  %13 = phi ptr [ %8, %7 ], [ %11, %9 ]
  store ptr %13, ptr %3, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = call i32 @lua_isyieldable(ptr noundef %15)
  call void @lua_pushboolean(ptr noundef %14, i32 noundef %16)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_close(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @getco(ptr noundef %6)
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @auxstatus(ptr noundef %8, ptr noundef %9)
  store i32 %10, ptr %5, align 4
  %11 = load i32, ptr %5, align 4
  switch i32 %11, label %24 [
    i32 1, label %12
    i32 2, label %12
  ]

12:                                               ; preds = %1, %1
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @lua_closethread(ptr noundef %13, ptr noundef %14)
  store i32 %15, ptr %5, align 4
  %16 = load i32, ptr %5, align 4
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %20

18:                                               ; preds = %12
  %19 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %19, i32 noundef 1)
  store i32 1, ptr %2, align 4
  br label %31

20:                                               ; preds = %12
  %21 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %21, i32 noundef 0)
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %3, align 8
  call void @lua_xmove(ptr noundef %22, ptr noundef %23, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %31

24:                                               ; preds = %1
  %25 = load ptr, ptr %3, align 8
  %26 = load i32, ptr %5, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds [4 x ptr], ptr @statname, i64 0, i64 %27
  %29 = load ptr, ptr %28, align 8
  %30 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %25, ptr noundef @.str.14, ptr noundef %29)
  store i32 %30, ptr %2, align 4
  br label %31

31:                                               ; preds = %24, %20, %18
  %32 = load i32, ptr %2, align 4
  ret i32 %32
}

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_newthread(ptr noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare void @lua_xmove(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getco(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @lua_tothread(ptr noundef %4, i32 noundef 1)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = icmp ne ptr %6, null
  %8 = zext i1 %7 to i32
  %9 = sext i32 %8 to i64
  %10 = icmp ne i64 %9, 0
  br i1 %10, label %15, label %11

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  %13 = call i32 @luaL_typeerror(ptr noundef %12, i32 noundef 1, ptr noundef @.str.8)
  %14 = icmp ne i32 %13, 0
  br label %15

15:                                               ; preds = %11, %1
  %16 = phi i1 [ true, %1 ], [ %14, %11 ]
  %17 = zext i1 %16 to i32
  %18 = load ptr, ptr %3, align 8
  ret ptr %18
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @auxresume(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %10 = load ptr, ptr %6, align 8
  %11 = load i32, ptr %7, align 4
  %12 = call i32 @lua_checkstack(ptr noundef %10, i32 noundef %11)
  %13 = icmp ne i32 %12, 0
  %14 = xor i1 %13, true
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %3
  %21 = load ptr, ptr %5, align 8
  %22 = call ptr @lua_pushstring(ptr noundef %21, ptr noundef @.str.9)
  store i32 -1, ptr %4, align 4
  br label %70

23:                                               ; preds = %3
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %6, align 8
  %26 = load i32, ptr %7, align 4
  call void @lua_xmove(ptr noundef %24, ptr noundef %25, i32 noundef %26)
  %27 = load ptr, ptr %6, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %7, align 4
  %30 = call i32 @lua_resume(ptr noundef %27, ptr noundef %28, i32 noundef %29, ptr noundef %9)
  store i32 %30, ptr %8, align 4
  %31 = load i32, ptr %8, align 4
  %32 = icmp eq i32 %31, 0
  br i1 %32, label %36, label %33

33:                                               ; preds = %23
  %34 = load i32, ptr %8, align 4
  %35 = icmp eq i32 %34, 1
  br label %36

36:                                               ; preds = %33, %23
  %37 = phi i1 [ true, %23 ], [ %35, %33 ]
  %38 = zext i1 %37 to i32
  %39 = icmp ne i32 %38, 0
  %40 = zext i1 %39 to i32
  %41 = sext i32 %40 to i64
  %42 = icmp ne i64 %41, 0
  br i1 %42, label %43, label %67

43:                                               ; preds = %36
  %44 = load ptr, ptr %5, align 8
  %45 = load i32, ptr %9, align 4
  %46 = add nsw i32 %45, 1
  %47 = call i32 @lua_checkstack(ptr noundef %44, i32 noundef %46)
  %48 = icmp ne i32 %47, 0
  %49 = xor i1 %48, true
  %50 = zext i1 %49 to i32
  %51 = icmp ne i32 %50, 0
  %52 = zext i1 %51 to i32
  %53 = sext i32 %52 to i64
  %54 = icmp ne i64 %53, 0
  br i1 %54, label %55, label %62

55:                                               ; preds = %43
  %56 = load ptr, ptr %6, align 8
  %57 = load i32, ptr %9, align 4
  %58 = sub nsw i32 0, %57
  %59 = sub nsw i32 %58, 1
  call void @lua_settop(ptr noundef %56, i32 noundef %59)
  %60 = load ptr, ptr %5, align 8
  %61 = call ptr @lua_pushstring(ptr noundef %60, ptr noundef @.str.10)
  store i32 -1, ptr %4, align 4
  br label %70

62:                                               ; preds = %43
  %63 = load ptr, ptr %6, align 8
  %64 = load ptr, ptr %5, align 8
  %65 = load i32, ptr %9, align 4
  call void @lua_xmove(ptr noundef %63, ptr noundef %64, i32 noundef %65)
  %66 = load i32, ptr %9, align 4
  store i32 %66, ptr %4, align 4
  br label %70

67:                                               ; preds = %36
  %68 = load ptr, ptr %6, align 8
  %69 = load ptr, ptr %5, align 8
  call void @lua_xmove(ptr noundef %68, ptr noundef %69, i32 noundef 1)
  store i32 -1, ptr %4, align 4
  br label %70

70:                                               ; preds = %67, %62, %55, %20
  %71 = load i32, ptr %4, align 4
  ret i32 %71
}

declare i32 @lua_gettop(ptr noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_tothread(ptr noundef, i32 noundef) #1

declare i32 @luaL_typeerror(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_checkstack(ptr noundef, i32 noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare i32 @lua_resume(ptr noundef, ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare i32 @lua_pushthread(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @auxstatus(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.lua_Debug, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = icmp eq ptr %7, %8
  br i1 %9, label %10, label %11

10:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %27

11:                                               ; preds = %2
  %12 = load ptr, ptr %5, align 8
  %13 = call i32 @lua_status(ptr noundef %12)
  switch i32 %13, label %26 [
    i32 1, label %14
    i32 0, label %15
  ]

14:                                               ; preds = %11
  store i32 2, ptr %3, align 4
  br label %27

15:                                               ; preds = %11
  %16 = load ptr, ptr %5, align 8
  %17 = call i32 @lua_getstack(ptr noundef %16, i32 noundef 0, ptr noundef %6)
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %20

19:                                               ; preds = %15
  store i32 3, ptr %3, align 4
  br label %27

20:                                               ; preds = %15
  %21 = load ptr, ptr %5, align 8
  %22 = call i32 @lua_gettop(ptr noundef %21)
  %23 = icmp eq i32 %22, 0
  br i1 %23, label %24, label %25

24:                                               ; preds = %20
  store i32 1, ptr %3, align 4
  br label %27

25:                                               ; preds = %20
  store i32 2, ptr %3, align 4
  br label %27

26:                                               ; preds = %11
  store i32 1, ptr %3, align 4
  br label %27

27:                                               ; preds = %26, %25, %24, %19, %14, %10
  %28 = load i32, ptr %3, align 4
  ret i32 %28
}

declare i32 @lua_status(ptr noundef) #1

declare i32 @lua_getstack(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_auxwrap(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call ptr @lua_tothread(ptr noundef %7, i32 noundef -1001001)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = call i32 @lua_gettop(ptr noundef %11)
  %13 = call i32 @auxresume(ptr noundef %9, ptr noundef %10, i32 noundef %12)
  store i32 %13, ptr %5, align 4
  %14 = load i32, ptr %5, align 4
  %15 = icmp slt i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = icmp ne i32 %16, 0
  %18 = zext i1 %17 to i32
  %19 = sext i32 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %21, label %49

21:                                               ; preds = %1
  %22 = load ptr, ptr %4, align 8
  %23 = call i32 @lua_status(ptr noundef %22)
  store i32 %23, ptr %6, align 4
  %24 = load i32, ptr %6, align 4
  %25 = icmp ne i32 %24, 0
  br i1 %25, label %26, label %35

26:                                               ; preds = %21
  %27 = load i32, ptr %6, align 4
  %28 = icmp ne i32 %27, 1
  br i1 %28, label %29, label %35

29:                                               ; preds = %26
  %30 = load ptr, ptr %4, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = call i32 @lua_closethread(ptr noundef %30, ptr noundef %31)
  store i32 %32, ptr %6, align 4
  %33 = load ptr, ptr %4, align 8
  %34 = load ptr, ptr %3, align 8
  call void @lua_xmove(ptr noundef %33, ptr noundef %34, i32 noundef 1)
  br label %35

35:                                               ; preds = %29, %26, %21
  %36 = load i32, ptr %6, align 4
  %37 = icmp ne i32 %36, 4
  br i1 %37, label %38, label %46

38:                                               ; preds = %35
  %39 = load ptr, ptr %3, align 8
  %40 = call i32 @lua_type(ptr noundef %39, i32 noundef -1)
  %41 = icmp eq i32 %40, 4
  br i1 %41, label %42, label %46

42:                                               ; preds = %38
  %43 = load ptr, ptr %3, align 8
  call void @luaL_where(ptr noundef %43, i32 noundef 1)
  %44 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %44, i32 noundef -2, i32 noundef 1)
  %45 = load ptr, ptr %3, align 8
  call void @lua_concat(ptr noundef %45, i32 noundef 2)
  br label %46

46:                                               ; preds = %42, %38, %35
  %47 = load ptr, ptr %3, align 8
  %48 = call i32 @lua_error(ptr noundef %47)
  store i32 %48, ptr %2, align 4
  br label %51

49:                                               ; preds = %1
  %50 = load i32, ptr %5, align 4
  store i32 %50, ptr %2, align 4
  br label %51

51:                                               ; preds = %49, %46
  %52 = load i32, ptr %2, align 4
  ret i32 %52
}

declare i32 @lua_closethread(ptr noundef, ptr noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare void @luaL_where(ptr noundef, i32 noundef) #1

declare void @lua_concat(ptr noundef, i32 noundef) #1

declare i32 @lua_error(ptr noundef) #1

declare i32 @lua_yieldk(ptr noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_isyieldable(ptr noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
