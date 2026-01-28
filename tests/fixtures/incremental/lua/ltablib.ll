; ModuleID = 'ltablib.c'
source_filename = "ltablib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }

@tab_funcs = internal constant [8 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str, ptr @tconcat }, %struct.luaL_Reg { ptr @.str.1, ptr @tinsert }, %struct.luaL_Reg { ptr @.str.2, ptr @tpack }, %struct.luaL_Reg { ptr @.str.3, ptr @tunpack }, %struct.luaL_Reg { ptr @.str.4, ptr @tremove }, %struct.luaL_Reg { ptr @.str.5, ptr @tmove }, %struct.luaL_Reg { ptr @.str.6, ptr @sort }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [7 x i8] c"concat\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"insert\00", align 1
@.str.2 = private unnamed_addr constant [5 x i8] c"pack\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"unpack\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"remove\00", align 1
@.str.5 = private unnamed_addr constant [5 x i8] c"move\00", align 1
@.str.6 = private unnamed_addr constant [5 x i8] c"sort\00", align 1
@.str.7 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.8 = private unnamed_addr constant [8 x i8] c"__index\00", align 1
@.str.9 = private unnamed_addr constant [11 x i8] c"__newindex\00", align 1
@.str.10 = private unnamed_addr constant [6 x i8] c"__len\00", align 1
@.str.11 = private unnamed_addr constant [53 x i8] c"invalid value (%s) at index %I in table for 'concat'\00", align 1
@.str.12 = private unnamed_addr constant [23 x i8] c"position out of bounds\00", align 1
@.str.13 = private unnamed_addr constant [38 x i8] c"wrong number of arguments to 'insert'\00", align 1
@.str.14 = private unnamed_addr constant [2 x i8] c"n\00", align 1
@.str.15 = private unnamed_addr constant [27 x i8] c"too many results to unpack\00", align 1
@.str.16 = private unnamed_addr constant [26 x i8] c"too many elements to move\00", align 1
@.str.17 = private unnamed_addr constant [24 x i8] c"destination wrap around\00", align 1
@.str.18 = private unnamed_addr constant [14 x i8] c"array too big\00", align 1
@.str.19 = private unnamed_addr constant [35 x i8] c"invalid order function for sorting\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_table(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 7)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @tab_funcs, i32 noundef 0)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tconcat(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.luaL_Buffer, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  call void @checktab(ptr noundef %8, i32 noundef 1, i32 noundef 5)
  %9 = load ptr, ptr %2, align 8
  %10 = call i64 @luaL_len(ptr noundef %9, i32 noundef 1)
  store i64 %10, ptr %4, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = call ptr @luaL_optlstring(ptr noundef %11, i32 noundef 2, ptr noundef @.str.7, ptr noundef %5)
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = call i64 @luaL_optinteger(ptr noundef %13, i32 noundef 3, i64 noundef 1)
  store i64 %14, ptr %7, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = load i64, ptr %4, align 8
  %17 = call i64 @luaL_optinteger(ptr noundef %15, i32 noundef 4, i64 noundef %16)
  store i64 %17, ptr %4, align 8
  %18 = load ptr, ptr %2, align 8
  call void @luaL_buffinit(ptr noundef %18, ptr noundef %3)
  br label %19

19:                                               ; preds = %28, %1
  %20 = load i64, ptr %7, align 8
  %21 = load i64, ptr %4, align 8
  %22 = icmp slt i64 %20, %21
  br i1 %22, label %23, label %31

23:                                               ; preds = %19
  %24 = load ptr, ptr %2, align 8
  %25 = load i64, ptr %7, align 8
  call void @addfield(ptr noundef %24, ptr noundef %3, i64 noundef %25)
  %26 = load ptr, ptr %6, align 8
  %27 = load i64, ptr %5, align 8
  call void @luaL_addlstring(ptr noundef %3, ptr noundef %26, i64 noundef %27)
  br label %28

28:                                               ; preds = %23
  %29 = load i64, ptr %7, align 8
  %30 = add nsw i64 %29, 1
  store i64 %30, ptr %7, align 8
  br label %19, !llvm.loop !6

31:                                               ; preds = %19
  %32 = load i64, ptr %7, align 8
  %33 = load i64, ptr %4, align 8
  %34 = icmp eq i64 %32, %33
  br i1 %34, label %35, label %38

35:                                               ; preds = %31
  %36 = load ptr, ptr %2, align 8
  %37 = load i64, ptr %7, align 8
  call void @addfield(ptr noundef %36, ptr noundef %3, i64 noundef %37)
  br label %38

38:                                               ; preds = %35, %31
  call void @luaL_pushresult(ptr noundef %3)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tinsert(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  call void @checktab(ptr noundef %7, i32 noundef 1, i32 noundef 7)
  %8 = load ptr, ptr %3, align 8
  %9 = call i64 @luaL_len(ptr noundef %8, i32 noundef 1)
  store i64 %9, ptr %5, align 8
  %10 = load i64, ptr %5, align 8
  %11 = add i64 %10, 1
  store i64 %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = call i32 @lua_gettop(ptr noundef %12)
  switch i32 %13, label %51 [
    i32 2, label %14
    i32 3, label %16
  ]

14:                                               ; preds = %1
  %15 = load i64, ptr %5, align 8
  store i64 %15, ptr %4, align 8
  br label %54

16:                                               ; preds = %1
  %17 = load ptr, ptr %3, align 8
  %18 = call i64 @luaL_checkinteger(ptr noundef %17, i32 noundef 2)
  store i64 %18, ptr %4, align 8
  %19 = load i64, ptr %4, align 8
  %20 = sub i64 %19, 1
  %21 = load i64, ptr %5, align 8
  %22 = icmp ult i64 %20, %21
  %23 = zext i1 %22 to i32
  %24 = icmp ne i32 %23, 0
  %25 = zext i1 %24 to i32
  %26 = sext i32 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %32, label %28

28:                                               ; preds = %16
  %29 = load ptr, ptr %3, align 8
  %30 = call i32 @luaL_argerror(ptr noundef %29, i32 noundef 2, ptr noundef @.str.12)
  %31 = icmp ne i32 %30, 0
  br label %32

32:                                               ; preds = %28, %16
  %33 = phi i1 [ true, %16 ], [ %31, %28 ]
  %34 = zext i1 %33 to i32
  %35 = load i64, ptr %5, align 8
  store i64 %35, ptr %6, align 8
  br label %36

36:                                               ; preds = %47, %32
  %37 = load i64, ptr %6, align 8
  %38 = load i64, ptr %4, align 8
  %39 = icmp sgt i64 %37, %38
  br i1 %39, label %40, label %50

40:                                               ; preds = %36
  %41 = load ptr, ptr %3, align 8
  %42 = load i64, ptr %6, align 8
  %43 = sub nsw i64 %42, 1
  %44 = call i32 @lua_geti(ptr noundef %41, i32 noundef 1, i64 noundef %43)
  %45 = load ptr, ptr %3, align 8
  %46 = load i64, ptr %6, align 8
  call void @lua_seti(ptr noundef %45, i32 noundef 1, i64 noundef %46)
  br label %47

47:                                               ; preds = %40
  %48 = load i64, ptr %6, align 8
  %49 = add nsw i64 %48, -1
  store i64 %49, ptr %6, align 8
  br label %36, !llvm.loop !8

50:                                               ; preds = %36
  br label %54

51:                                               ; preds = %1
  %52 = load ptr, ptr %3, align 8
  %53 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %52, ptr noundef @.str.13)
  store i32 %53, ptr %2, align 4
  br label %57

54:                                               ; preds = %50, %14
  %55 = load ptr, ptr %3, align 8
  %56 = load i64, ptr %4, align 8
  call void @lua_seti(ptr noundef %55, i32 noundef 1, i64 noundef %56)
  store i32 0, ptr %2, align 4
  br label %57

57:                                               ; preds = %54, %51
  %58 = load i32, ptr %2, align 4
  ret i32 %58
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tpack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_gettop(ptr noundef %5)
  store i32 %6, ptr %4, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load i32, ptr %4, align 4
  call void @lua_createtable(ptr noundef %7, i32 noundef %8, i32 noundef 1)
  %9 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %9, i32 noundef 1, i32 noundef 1)
  %10 = load i32, ptr %4, align 4
  store i32 %10, ptr %3, align 4
  br label %11

11:                                               ; preds = %18, %1
  %12 = load i32, ptr %3, align 4
  %13 = icmp sge i32 %12, 1
  br i1 %13, label %14, label %21

14:                                               ; preds = %11
  %15 = load ptr, ptr %2, align 8
  %16 = load i32, ptr %3, align 4
  %17 = sext i32 %16 to i64
  call void @lua_seti(ptr noundef %15, i32 noundef 1, i64 noundef %17)
  br label %18

18:                                               ; preds = %14
  %19 = load i32, ptr %3, align 4
  %20 = add nsw i32 %19, -1
  store i32 %20, ptr %3, align 4
  br label %11, !llvm.loop !9

21:                                               ; preds = %11
  %22 = load ptr, ptr %2, align 8
  %23 = load i32, ptr %4, align 4
  %24 = sext i32 %23 to i64
  call void @lua_pushinteger(ptr noundef %22, i64 noundef %24)
  %25 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %25, i32 noundef 1, ptr noundef @.str.14)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tunpack(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i64 @luaL_optinteger(ptr noundef %7, i32 noundef 2, i64 noundef 1)
  store i64 %8, ptr %5, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call i32 @lua_type(ptr noundef %9, i32 noundef 3)
  %11 = icmp sle i32 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %1
  %13 = load ptr, ptr %3, align 8
  %14 = call i64 @luaL_len(ptr noundef %13, i32 noundef 1)
  br label %18

15:                                               ; preds = %1
  %16 = load ptr, ptr %3, align 8
  %17 = call i64 @luaL_checkinteger(ptr noundef %16, i32 noundef 3)
  br label %18

18:                                               ; preds = %15, %12
  %19 = phi i64 [ %14, %12 ], [ %17, %15 ]
  store i64 %19, ptr %6, align 8
  %20 = load i64, ptr %5, align 8
  %21 = load i64, ptr %6, align 8
  %22 = icmp sgt i64 %20, %21
  br i1 %22, label %23, label %24

23:                                               ; preds = %18
  store i32 0, ptr %2, align 4
  br label %66

24:                                               ; preds = %18
  %25 = load i64, ptr %6, align 8
  %26 = load i64, ptr %5, align 8
  %27 = sub i64 %25, %26
  store i64 %27, ptr %4, align 8
  %28 = load i64, ptr %4, align 8
  %29 = icmp uge i64 %28, 2147483647
  br i1 %29, label %38, label %30

30:                                               ; preds = %24
  %31 = load ptr, ptr %3, align 8
  %32 = load i64, ptr %4, align 8
  %33 = add i64 %32, 1
  store i64 %33, ptr %4, align 8
  %34 = trunc i64 %33 to i32
  %35 = call i32 @lua_checkstack(ptr noundef %31, i32 noundef %34)
  %36 = icmp ne i32 %35, 0
  %37 = xor i1 %36, true
  br label %38

38:                                               ; preds = %30, %24
  %39 = phi i1 [ true, %24 ], [ %37, %30 ]
  %40 = zext i1 %39 to i32
  %41 = icmp ne i32 %40, 0
  %42 = zext i1 %41 to i32
  %43 = sext i32 %42 to i64
  %44 = icmp ne i64 %43, 0
  br i1 %44, label %45, label %48

45:                                               ; preds = %38
  %46 = load ptr, ptr %3, align 8
  %47 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %46, ptr noundef @.str.15)
  store i32 %47, ptr %2, align 4
  br label %66

48:                                               ; preds = %38
  br label %49

49:                                               ; preds = %57, %48
  %50 = load i64, ptr %5, align 8
  %51 = load i64, ptr %6, align 8
  %52 = icmp slt i64 %50, %51
  br i1 %52, label %53, label %60

53:                                               ; preds = %49
  %54 = load ptr, ptr %3, align 8
  %55 = load i64, ptr %5, align 8
  %56 = call i32 @lua_geti(ptr noundef %54, i32 noundef 1, i64 noundef %55)
  br label %57

57:                                               ; preds = %53
  %58 = load i64, ptr %5, align 8
  %59 = add nsw i64 %58, 1
  store i64 %59, ptr %5, align 8
  br label %49, !llvm.loop !10

60:                                               ; preds = %49
  %61 = load ptr, ptr %3, align 8
  %62 = load i64, ptr %6, align 8
  %63 = call i32 @lua_geti(ptr noundef %61, i32 noundef 1, i64 noundef %62)
  %64 = load i64, ptr %4, align 8
  %65 = trunc i64 %64 to i32
  store i32 %65, ptr %2, align 4
  br label %66

66:                                               ; preds = %60, %45, %23
  %67 = load i32, ptr %2, align 4
  ret i32 %67
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tremove(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  call void @checktab(ptr noundef %5, i32 noundef 1, i32 noundef 7)
  %6 = load ptr, ptr %2, align 8
  %7 = call i64 @luaL_len(ptr noundef %6, i32 noundef 1)
  store i64 %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load i64, ptr %3, align 8
  %10 = call i64 @luaL_optinteger(ptr noundef %8, i32 noundef 2, i64 noundef %9)
  store i64 %10, ptr %4, align 8
  %11 = load i64, ptr %4, align 8
  %12 = load i64, ptr %3, align 8
  %13 = icmp ne i64 %11, %12
  br i1 %13, label %14, label %31

14:                                               ; preds = %1
  %15 = load i64, ptr %4, align 8
  %16 = sub i64 %15, 1
  %17 = load i64, ptr %3, align 8
  %18 = icmp ule i64 %16, %17
  %19 = zext i1 %18 to i32
  %20 = icmp ne i32 %19, 0
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %28, label %24

24:                                               ; preds = %14
  %25 = load ptr, ptr %2, align 8
  %26 = call i32 @luaL_argerror(ptr noundef %25, i32 noundef 2, ptr noundef @.str.12)
  %27 = icmp ne i32 %26, 0
  br label %28

28:                                               ; preds = %24, %14
  %29 = phi i1 [ true, %14 ], [ %27, %24 ]
  %30 = zext i1 %29 to i32
  br label %31

31:                                               ; preds = %28, %1
  %32 = load ptr, ptr %2, align 8
  %33 = load i64, ptr %4, align 8
  %34 = call i32 @lua_geti(ptr noundef %32, i32 noundef 1, i64 noundef %33)
  br label %35

35:                                               ; preds = %46, %31
  %36 = load i64, ptr %4, align 8
  %37 = load i64, ptr %3, align 8
  %38 = icmp slt i64 %36, %37
  br i1 %38, label %39, label %49

39:                                               ; preds = %35
  %40 = load ptr, ptr %2, align 8
  %41 = load i64, ptr %4, align 8
  %42 = add nsw i64 %41, 1
  %43 = call i32 @lua_geti(ptr noundef %40, i32 noundef 1, i64 noundef %42)
  %44 = load ptr, ptr %2, align 8
  %45 = load i64, ptr %4, align 8
  call void @lua_seti(ptr noundef %44, i32 noundef 1, i64 noundef %45)
  br label %46

46:                                               ; preds = %39
  %47 = load i64, ptr %4, align 8
  %48 = add nsw i64 %47, 1
  store i64 %48, ptr %4, align 8
  br label %35, !llvm.loop !11

49:                                               ; preds = %35
  %50 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %50)
  %51 = load ptr, ptr %2, align 8
  %52 = load i64, ptr %4, align 8
  call void @lua_seti(ptr noundef %51, i32 noundef 1, i64 noundef %52)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tmove(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call i64 @luaL_checkinteger(ptr noundef %9, i32 noundef 2)
  store i64 %10, ptr %3, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = call i64 @luaL_checkinteger(ptr noundef %11, i32 noundef 3)
  store i64 %12, ptr %4, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = call i64 @luaL_checkinteger(ptr noundef %13, i32 noundef 4)
  store i64 %14, ptr %5, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = call i32 @lua_type(ptr noundef %15, i32 noundef 5)
  %17 = icmp sle i32 %16, 0
  %18 = xor i1 %17, true
  %19 = zext i1 %18 to i64
  %20 = select i1 %18, i32 5, i32 1
  store i32 %20, ptr %6, align 4
  %21 = load ptr, ptr %2, align 8
  call void @checktab(ptr noundef %21, i32 noundef 1, i32 noundef 1)
  %22 = load ptr, ptr %2, align 8
  %23 = load i32, ptr %6, align 4
  call void @checktab(ptr noundef %22, i32 noundef %23, i32 noundef 2)
  %24 = load i64, ptr %4, align 8
  %25 = load i64, ptr %3, align 8
  %26 = icmp sge i64 %24, %25
  br i1 %26, label %27, label %127

27:                                               ; preds = %1
  %28 = load i64, ptr %3, align 8
  %29 = icmp sgt i64 %28, 0
  br i1 %29, label %35, label %30

30:                                               ; preds = %27
  %31 = load i64, ptr %4, align 8
  %32 = load i64, ptr %3, align 8
  %33 = add nsw i64 9223372036854775807, %32
  %34 = icmp slt i64 %31, %33
  br label %35

35:                                               ; preds = %30, %27
  %36 = phi i1 [ true, %27 ], [ %34, %30 ]
  %37 = zext i1 %36 to i32
  %38 = icmp ne i32 %37, 0
  %39 = zext i1 %38 to i32
  %40 = sext i32 %39 to i64
  %41 = icmp ne i64 %40, 0
  br i1 %41, label %46, label %42

42:                                               ; preds = %35
  %43 = load ptr, ptr %2, align 8
  %44 = call i32 @luaL_argerror(ptr noundef %43, i32 noundef 3, ptr noundef @.str.16)
  %45 = icmp ne i32 %44, 0
  br label %46

46:                                               ; preds = %42, %35
  %47 = phi i1 [ true, %35 ], [ %45, %42 ]
  %48 = zext i1 %47 to i32
  %49 = load i64, ptr %4, align 8
  %50 = load i64, ptr %3, align 8
  %51 = sub nsw i64 %49, %50
  %52 = add nsw i64 %51, 1
  store i64 %52, ptr %7, align 8
  %53 = load i64, ptr %5, align 8
  %54 = load i64, ptr %7, align 8
  %55 = sub nsw i64 9223372036854775807, %54
  %56 = add nsw i64 %55, 1
  %57 = icmp sle i64 %53, %56
  %58 = zext i1 %57 to i32
  %59 = icmp ne i32 %58, 0
  %60 = zext i1 %59 to i32
  %61 = sext i32 %60 to i64
  %62 = icmp ne i64 %61, 0
  br i1 %62, label %67, label %63

63:                                               ; preds = %46
  %64 = load ptr, ptr %2, align 8
  %65 = call i32 @luaL_argerror(ptr noundef %64, i32 noundef 4, ptr noundef @.str.17)
  %66 = icmp ne i32 %65, 0
  br label %67

67:                                               ; preds = %63, %46
  %68 = phi i1 [ true, %46 ], [ %66, %63 ]
  %69 = zext i1 %68 to i32
  %70 = load i64, ptr %5, align 8
  %71 = load i64, ptr %4, align 8
  %72 = icmp sgt i64 %70, %71
  br i1 %72, label %85, label %73

73:                                               ; preds = %67
  %74 = load i64, ptr %5, align 8
  %75 = load i64, ptr %3, align 8
  %76 = icmp sle i64 %74, %75
  br i1 %76, label %85, label %77

77:                                               ; preds = %73
  %78 = load i32, ptr %6, align 4
  %79 = icmp ne i32 %78, 1
  br i1 %79, label %80, label %105

80:                                               ; preds = %77
  %81 = load ptr, ptr %2, align 8
  %82 = load i32, ptr %6, align 4
  %83 = call i32 @lua_compare(ptr noundef %81, i32 noundef 1, i32 noundef %82, i32 noundef 0)
  %84 = icmp ne i32 %83, 0
  br i1 %84, label %105, label %85

85:                                               ; preds = %80, %73, %67
  store i64 0, ptr %8, align 8
  br label %86

86:                                               ; preds = %101, %85
  %87 = load i64, ptr %8, align 8
  %88 = load i64, ptr %7, align 8
  %89 = icmp slt i64 %87, %88
  br i1 %89, label %90, label %104

90:                                               ; preds = %86
  %91 = load ptr, ptr %2, align 8
  %92 = load i64, ptr %3, align 8
  %93 = load i64, ptr %8, align 8
  %94 = add nsw i64 %92, %93
  %95 = call i32 @lua_geti(ptr noundef %91, i32 noundef 1, i64 noundef %94)
  %96 = load ptr, ptr %2, align 8
  %97 = load i32, ptr %6, align 4
  %98 = load i64, ptr %5, align 8
  %99 = load i64, ptr %8, align 8
  %100 = add nsw i64 %98, %99
  call void @lua_seti(ptr noundef %96, i32 noundef %97, i64 noundef %100)
  br label %101

101:                                              ; preds = %90
  %102 = load i64, ptr %8, align 8
  %103 = add nsw i64 %102, 1
  store i64 %103, ptr %8, align 8
  br label %86, !llvm.loop !12

104:                                              ; preds = %86
  br label %126

105:                                              ; preds = %80, %77
  %106 = load i64, ptr %7, align 8
  %107 = sub nsw i64 %106, 1
  store i64 %107, ptr %8, align 8
  br label %108

108:                                              ; preds = %122, %105
  %109 = load i64, ptr %8, align 8
  %110 = icmp sge i64 %109, 0
  br i1 %110, label %111, label %125

111:                                              ; preds = %108
  %112 = load ptr, ptr %2, align 8
  %113 = load i64, ptr %3, align 8
  %114 = load i64, ptr %8, align 8
  %115 = add nsw i64 %113, %114
  %116 = call i32 @lua_geti(ptr noundef %112, i32 noundef 1, i64 noundef %115)
  %117 = load ptr, ptr %2, align 8
  %118 = load i32, ptr %6, align 4
  %119 = load i64, ptr %5, align 8
  %120 = load i64, ptr %8, align 8
  %121 = add nsw i64 %119, %120
  call void @lua_seti(ptr noundef %117, i32 noundef %118, i64 noundef %121)
  br label %122

122:                                              ; preds = %111
  %123 = load i64, ptr %8, align 8
  %124 = add nsw i64 %123, -1
  store i64 %124, ptr %8, align 8
  br label %108, !llvm.loop !13

125:                                              ; preds = %108
  br label %126

126:                                              ; preds = %125, %104
  br label %127

127:                                              ; preds = %126, %1
  %128 = load ptr, ptr %2, align 8
  %129 = load i32, ptr %6, align 4
  call void @lua_pushvalue(ptr noundef %128, i32 noundef %129)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @sort(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @checktab(ptr noundef %4, i32 noundef 1, i32 noundef 7)
  %5 = load ptr, ptr %2, align 8
  %6 = call i64 @luaL_len(ptr noundef %5, i32 noundef 1)
  store i64 %6, ptr %3, align 8
  %7 = load i64, ptr %3, align 8
  %8 = icmp sgt i64 %7, 1
  br i1 %8, label %9, label %34

9:                                                ; preds = %1
  %10 = load i64, ptr %3, align 8
  %11 = icmp slt i64 %10, 2147483647
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %21, label %17

17:                                               ; preds = %9
  %18 = load ptr, ptr %2, align 8
  %19 = call i32 @luaL_argerror(ptr noundef %18, i32 noundef 1, ptr noundef @.str.18)
  %20 = icmp ne i32 %19, 0
  br label %21

21:                                               ; preds = %17, %9
  %22 = phi i1 [ true, %9 ], [ %20, %17 ]
  %23 = zext i1 %22 to i32
  %24 = load ptr, ptr %2, align 8
  %25 = call i32 @lua_type(ptr noundef %24, i32 noundef 2)
  %26 = icmp sle i32 %25, 0
  br i1 %26, label %29, label %27

27:                                               ; preds = %21
  %28 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %28, i32 noundef 2, i32 noundef 6)
  br label %29

29:                                               ; preds = %27, %21
  %30 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %30, i32 noundef 2)
  %31 = load ptr, ptr %2, align 8
  %32 = load i64, ptr %3, align 8
  %33 = trunc i64 %32 to i32
  call void @auxsort(ptr noundef %31, i32 noundef 1, i32 noundef %33, i32 noundef 0)
  br label %34

34:                                               ; preds = %29, %1
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checktab(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call i32 @lua_type(ptr noundef %8, i32 noundef %9)
  %11 = icmp ne i32 %10, 5
  br i1 %11, label %12, label %56

12:                                               ; preds = %3
  store i32 1, ptr %7, align 4
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call i32 @lua_getmetatable(ptr noundef %13, i32 noundef %14)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %52

17:                                               ; preds = %12
  %18 = load i32, ptr %6, align 4
  %19 = and i32 %18, 1
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %27

21:                                               ; preds = %17
  %22 = load ptr, ptr %4, align 8
  %23 = load i32, ptr %7, align 4
  %24 = add nsw i32 %23, 1
  store i32 %24, ptr %7, align 4
  %25 = call i32 @checkfield(ptr noundef %22, ptr noundef @.str.8, i32 noundef %24)
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %52

27:                                               ; preds = %21, %17
  %28 = load i32, ptr %6, align 4
  %29 = and i32 %28, 2
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %37

31:                                               ; preds = %27
  %32 = load ptr, ptr %4, align 8
  %33 = load i32, ptr %7, align 4
  %34 = add nsw i32 %33, 1
  store i32 %34, ptr %7, align 4
  %35 = call i32 @checkfield(ptr noundef %32, ptr noundef @.str.9, i32 noundef %34)
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %37, label %52

37:                                               ; preds = %31, %27
  %38 = load i32, ptr %6, align 4
  %39 = and i32 %38, 4
  %40 = icmp ne i32 %39, 0
  br i1 %40, label %41, label %47

41:                                               ; preds = %37
  %42 = load ptr, ptr %4, align 8
  %43 = load i32, ptr %7, align 4
  %44 = add nsw i32 %43, 1
  store i32 %44, ptr %7, align 4
  %45 = call i32 @checkfield(ptr noundef %42, ptr noundef @.str.10, i32 noundef %44)
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %52

47:                                               ; preds = %41, %37
  %48 = load ptr, ptr %4, align 8
  %49 = load i32, ptr %7, align 4
  %50 = sub nsw i32 0, %49
  %51 = sub nsw i32 %50, 1
  call void @lua_settop(ptr noundef %48, i32 noundef %51)
  br label %55

52:                                               ; preds = %41, %31, %21, %12
  %53 = load ptr, ptr %4, align 8
  %54 = load i32, ptr %5, align 4
  call void @luaL_checktype(ptr noundef %53, i32 noundef %54, i32 noundef 5)
  br label %55

55:                                               ; preds = %52, %47
  br label %56

56:                                               ; preds = %55, %3
  ret void
}

declare i64 @luaL_len(ptr noundef, i32 noundef) #1

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addfield(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i64, ptr %6, align 8
  %9 = call i32 @lua_geti(ptr noundef %7, i32 noundef 1, i64 noundef %8)
  %10 = load ptr, ptr %4, align 8
  %11 = call i32 @lua_isstring(ptr noundef %10, i32 noundef -1)
  %12 = icmp ne i32 %11, 0
  %13 = xor i1 %12, true
  %14 = zext i1 %13 to i32
  %15 = icmp ne i32 %14, 0
  %16 = zext i1 %15 to i32
  %17 = sext i32 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %19, label %27

19:                                               ; preds = %3
  %20 = load ptr, ptr %4, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = call i32 @lua_type(ptr noundef %22, i32 noundef -1)
  %24 = call ptr @lua_typename(ptr noundef %21, i32 noundef %23)
  %25 = load i64, ptr %6, align 8
  %26 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %20, ptr noundef @.str.11, ptr noundef %24, i64 noundef %25)
  br label %27

27:                                               ; preds = %19, %3
  %28 = load ptr, ptr %5, align 8
  call void @luaL_addvalue(ptr noundef %28)
  ret void
}

declare void @luaL_addlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare void @luaL_pushresult(ptr noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare i32 @lua_getmetatable(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @checkfield(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = call ptr @lua_pushstring(ptr noundef %7, ptr noundef %8)
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %6, align 4
  %12 = sub nsw i32 0, %11
  %13 = call i32 @lua_rawget(ptr noundef %10, i32 noundef %12)
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  ret i32 %15
}

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare i32 @lua_rawget(ptr noundef, i32 noundef) #1

declare i32 @lua_geti(ptr noundef, i32 noundef, i64 noundef) #1

declare i32 @lua_isstring(ptr noundef, i32 noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare ptr @lua_typename(ptr noundef, i32 noundef) #1

declare void @luaL_addvalue(ptr noundef) #1

declare i32 @lua_gettop(ptr noundef) #1

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_seti(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_checkstack(ptr noundef, i32 noundef) #1

declare void @lua_pushnil(ptr noundef) #1

declare i32 @lua_compare(ptr noundef, i32 noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @auxsort(ptr noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  br label %11

11:                                               ; preds = %152, %4
  %12 = load i32, ptr %6, align 4
  %13 = load i32, ptr %7, align 4
  %14 = icmp ult i32 %12, %13
  br i1 %14, label %15, label %153

15:                                               ; preds = %11
  %16 = load ptr, ptr %5, align 8
  %17 = load i32, ptr %6, align 4
  %18 = zext i32 %17 to i64
  %19 = call i32 @lua_geti(ptr noundef %16, i32 noundef 1, i64 noundef %18)
  %20 = load ptr, ptr %5, align 8
  %21 = load i32, ptr %7, align 4
  %22 = zext i32 %21 to i64
  %23 = call i32 @lua_geti(ptr noundef %20, i32 noundef 1, i64 noundef %22)
  %24 = load ptr, ptr %5, align 8
  %25 = call i32 @sort_comp(ptr noundef %24, i32 noundef -1, i32 noundef -2)
  %26 = icmp ne i32 %25, 0
  br i1 %26, label %27, label %31

27:                                               ; preds = %15
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %6, align 4
  %30 = load i32, ptr %7, align 4
  call void @set2(ptr noundef %28, i32 noundef %29, i32 noundef %30)
  br label %33

31:                                               ; preds = %15
  %32 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %32, i32 noundef -3)
  br label %33

33:                                               ; preds = %31, %27
  %34 = load i32, ptr %7, align 4
  %35 = load i32, ptr %6, align 4
  %36 = sub i32 %34, %35
  %37 = icmp eq i32 %36, 1
  br i1 %37, label %38, label %39

38:                                               ; preds = %33
  br label %153

39:                                               ; preds = %33
  %40 = load i32, ptr %7, align 4
  %41 = load i32, ptr %6, align 4
  %42 = sub i32 %40, %41
  %43 = icmp ult i32 %42, 100
  br i1 %43, label %47, label %44

44:                                               ; preds = %39
  %45 = load i32, ptr %8, align 4
  %46 = icmp eq i32 %45, 0
  br i1 %46, label %47, label %52

47:                                               ; preds = %44, %39
  %48 = load i32, ptr %6, align 4
  %49 = load i32, ptr %7, align 4
  %50 = add i32 %48, %49
  %51 = udiv i32 %50, 2
  store i32 %51, ptr %9, align 4
  br label %57

52:                                               ; preds = %44
  %53 = load i32, ptr %6, align 4
  %54 = load i32, ptr %7, align 4
  %55 = load i32, ptr %8, align 4
  %56 = call i32 @choosePivot(i32 noundef %53, i32 noundef %54, i32 noundef %55)
  store i32 %56, ptr %9, align 4
  br label %57

57:                                               ; preds = %52, %47
  %58 = load ptr, ptr %5, align 8
  %59 = load i32, ptr %9, align 4
  %60 = zext i32 %59 to i64
  %61 = call i32 @lua_geti(ptr noundef %58, i32 noundef 1, i64 noundef %60)
  %62 = load ptr, ptr %5, align 8
  %63 = load i32, ptr %6, align 4
  %64 = zext i32 %63 to i64
  %65 = call i32 @lua_geti(ptr noundef %62, i32 noundef 1, i64 noundef %64)
  %66 = load ptr, ptr %5, align 8
  %67 = call i32 @sort_comp(ptr noundef %66, i32 noundef -2, i32 noundef -1)
  %68 = icmp ne i32 %67, 0
  br i1 %68, label %69, label %73

69:                                               ; preds = %57
  %70 = load ptr, ptr %5, align 8
  %71 = load i32, ptr %9, align 4
  %72 = load i32, ptr %6, align 4
  call void @set2(ptr noundef %70, i32 noundef %71, i32 noundef %72)
  br label %89

73:                                               ; preds = %57
  %74 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %74, i32 noundef -2)
  %75 = load ptr, ptr %5, align 8
  %76 = load i32, ptr %7, align 4
  %77 = zext i32 %76 to i64
  %78 = call i32 @lua_geti(ptr noundef %75, i32 noundef 1, i64 noundef %77)
  %79 = load ptr, ptr %5, align 8
  %80 = call i32 @sort_comp(ptr noundef %79, i32 noundef -1, i32 noundef -2)
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %86

82:                                               ; preds = %73
  %83 = load ptr, ptr %5, align 8
  %84 = load i32, ptr %9, align 4
  %85 = load i32, ptr %7, align 4
  call void @set2(ptr noundef %83, i32 noundef %84, i32 noundef %85)
  br label %88

86:                                               ; preds = %73
  %87 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %87, i32 noundef -3)
  br label %88

88:                                               ; preds = %86, %82
  br label %89

89:                                               ; preds = %88, %69
  %90 = load i32, ptr %7, align 4
  %91 = load i32, ptr %6, align 4
  %92 = sub i32 %90, %91
  %93 = icmp eq i32 %92, 2
  br i1 %93, label %94, label %95

94:                                               ; preds = %89
  br label %153

95:                                               ; preds = %89
  %96 = load ptr, ptr %5, align 8
  %97 = load i32, ptr %9, align 4
  %98 = zext i32 %97 to i64
  %99 = call i32 @lua_geti(ptr noundef %96, i32 noundef 1, i64 noundef %98)
  %100 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %100, i32 noundef -1)
  %101 = load ptr, ptr %5, align 8
  %102 = load i32, ptr %7, align 4
  %103 = sub i32 %102, 1
  %104 = zext i32 %103 to i64
  %105 = call i32 @lua_geti(ptr noundef %101, i32 noundef 1, i64 noundef %104)
  %106 = load ptr, ptr %5, align 8
  %107 = load i32, ptr %9, align 4
  %108 = load i32, ptr %7, align 4
  %109 = sub i32 %108, 1
  call void @set2(ptr noundef %106, i32 noundef %107, i32 noundef %109)
  %110 = load ptr, ptr %5, align 8
  %111 = load i32, ptr %6, align 4
  %112 = load i32, ptr %7, align 4
  %113 = call i32 @partition(ptr noundef %110, i32 noundef %111, i32 noundef %112)
  store i32 %113, ptr %9, align 4
  %114 = load i32, ptr %9, align 4
  %115 = load i32, ptr %6, align 4
  %116 = sub i32 %114, %115
  %117 = load i32, ptr %7, align 4
  %118 = load i32, ptr %9, align 4
  %119 = sub i32 %117, %118
  %120 = icmp ult i32 %116, %119
  br i1 %120, label %121, label %132

121:                                              ; preds = %95
  %122 = load ptr, ptr %5, align 8
  %123 = load i32, ptr %6, align 4
  %124 = load i32, ptr %9, align 4
  %125 = sub i32 %124, 1
  %126 = load i32, ptr %8, align 4
  call void @auxsort(ptr noundef %122, i32 noundef %123, i32 noundef %125, i32 noundef %126)
  %127 = load i32, ptr %9, align 4
  %128 = load i32, ptr %6, align 4
  %129 = sub i32 %127, %128
  store i32 %129, ptr %10, align 4
  %130 = load i32, ptr %9, align 4
  %131 = add i32 %130, 1
  store i32 %131, ptr %6, align 4
  br label %143

132:                                              ; preds = %95
  %133 = load ptr, ptr %5, align 8
  %134 = load i32, ptr %9, align 4
  %135 = add i32 %134, 1
  %136 = load i32, ptr %7, align 4
  %137 = load i32, ptr %8, align 4
  call void @auxsort(ptr noundef %133, i32 noundef %135, i32 noundef %136, i32 noundef %137)
  %138 = load i32, ptr %7, align 4
  %139 = load i32, ptr %9, align 4
  %140 = sub i32 %138, %139
  store i32 %140, ptr %10, align 4
  %141 = load i32, ptr %9, align 4
  %142 = sub i32 %141, 1
  store i32 %142, ptr %7, align 4
  br label %143

143:                                              ; preds = %132, %121
  %144 = load i32, ptr %7, align 4
  %145 = load i32, ptr %6, align 4
  %146 = sub i32 %144, %145
  %147 = udiv i32 %146, 128
  %148 = load i32, ptr %10, align 4
  %149 = icmp ugt i32 %147, %148
  br i1 %149, label %150, label %152

150:                                              ; preds = %143
  %151 = call i32 @l_randomizePivot()
  store i32 %151, ptr %8, align 4
  br label %152

152:                                              ; preds = %150, %143
  br label %11, !llvm.loop !14

153:                                              ; preds = %38, %94, %11
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @sort_comp(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = call i32 @lua_type(ptr noundef %9, i32 noundef 2)
  %11 = icmp eq i32 %10, 0
  br i1 %11, label %12, label %17

12:                                               ; preds = %3
  %13 = load ptr, ptr %5, align 8
  %14 = load i32, ptr %6, align 4
  %15 = load i32, ptr %7, align 4
  %16 = call i32 @lua_compare(ptr noundef %13, i32 noundef %14, i32 noundef %15, i32 noundef 1)
  store i32 %16, ptr %4, align 4
  br label %30

17:                                               ; preds = %3
  %18 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %18, i32 noundef 2)
  %19 = load ptr, ptr %5, align 8
  %20 = load i32, ptr %6, align 4
  %21 = sub nsw i32 %20, 1
  call void @lua_pushvalue(ptr noundef %19, i32 noundef %21)
  %22 = load ptr, ptr %5, align 8
  %23 = load i32, ptr %7, align 4
  %24 = sub nsw i32 %23, 2
  call void @lua_pushvalue(ptr noundef %22, i32 noundef %24)
  %25 = load ptr, ptr %5, align 8
  call void @lua_callk(ptr noundef %25, i32 noundef 2, i32 noundef 1, i64 noundef 0, ptr noundef null)
  %26 = load ptr, ptr %5, align 8
  %27 = call i32 @lua_toboolean(ptr noundef %26, i32 noundef -1)
  store i32 %27, ptr %8, align 4
  %28 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %28, i32 noundef -2)
  %29 = load i32, ptr %8, align 4
  store i32 %29, ptr %4, align 4
  br label %30

30:                                               ; preds = %17, %12
  %31 = load i32, ptr %4, align 4
  ret i32 %31
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @set2(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = zext i32 %8 to i64
  call void @lua_seti(ptr noundef %7, i32 noundef 1, i64 noundef %9)
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %6, align 4
  %12 = zext i32 %11 to i64
  call void @lua_seti(ptr noundef %10, i32 noundef 1, i64 noundef %12)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @choosePivot(i32 noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %9 = load i32, ptr %5, align 4
  %10 = load i32, ptr %4, align 4
  %11 = sub i32 %9, %10
  %12 = udiv i32 %11, 4
  store i32 %12, ptr %7, align 4
  %13 = load i32, ptr %6, align 4
  %14 = load i32, ptr %7, align 4
  %15 = mul i32 %14, 2
  %16 = urem i32 %13, %15
  %17 = load i32, ptr %4, align 4
  %18 = load i32, ptr %7, align 4
  %19 = add i32 %17, %18
  %20 = add i32 %16, %19
  store i32 %20, ptr %8, align 4
  %21 = load i32, ptr %8, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @partition(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %9 = load i32, ptr %5, align 4
  store i32 %9, ptr %7, align 4
  %10 = load i32, ptr %6, align 4
  %11 = sub i32 %10, 1
  store i32 %11, ptr %8, align 4
  br label %12

12:                                               ; preds = %72, %3
  br label %13

13:                                               ; preds = %35, %12
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %7, align 4
  %16 = add i32 %15, 1
  store i32 %16, ptr %7, align 4
  %17 = zext i32 %16 to i64
  %18 = call i32 @lua_geti(ptr noundef %14, i32 noundef 1, i64 noundef %17)
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @sort_comp(ptr noundef %19, i32 noundef -1, i32 noundef -2)
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %22, label %37

22:                                               ; preds = %13
  %23 = load i32, ptr %7, align 4
  %24 = load i32, ptr %6, align 4
  %25 = sub i32 %24, 1
  %26 = icmp eq i32 %23, %25
  %27 = zext i1 %26 to i32
  %28 = icmp ne i32 %27, 0
  %29 = zext i1 %28 to i32
  %30 = sext i32 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %32, label %35

32:                                               ; preds = %22
  %33 = load ptr, ptr %4, align 8
  %34 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %33, ptr noundef @.str.19)
  br label %35

35:                                               ; preds = %32, %22
  %36 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %36, i32 noundef -2)
  br label %13, !llvm.loop !15

37:                                               ; preds = %13
  br label %38

38:                                               ; preds = %59, %37
  %39 = load ptr, ptr %4, align 8
  %40 = load i32, ptr %8, align 4
  %41 = add i32 %40, -1
  store i32 %41, ptr %8, align 4
  %42 = zext i32 %41 to i64
  %43 = call i32 @lua_geti(ptr noundef %39, i32 noundef 1, i64 noundef %42)
  %44 = load ptr, ptr %4, align 8
  %45 = call i32 @sort_comp(ptr noundef %44, i32 noundef -3, i32 noundef -1)
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %61

47:                                               ; preds = %38
  %48 = load i32, ptr %8, align 4
  %49 = load i32, ptr %7, align 4
  %50 = icmp ult i32 %48, %49
  %51 = zext i1 %50 to i32
  %52 = icmp ne i32 %51, 0
  %53 = zext i1 %52 to i32
  %54 = sext i32 %53 to i64
  %55 = icmp ne i64 %54, 0
  br i1 %55, label %56, label %59

56:                                               ; preds = %47
  %57 = load ptr, ptr %4, align 8
  %58 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %57, ptr noundef @.str.19)
  br label %59

59:                                               ; preds = %56, %47
  %60 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %60, i32 noundef -2)
  br label %38, !llvm.loop !16

61:                                               ; preds = %38
  %62 = load i32, ptr %8, align 4
  %63 = load i32, ptr %7, align 4
  %64 = icmp ult i32 %62, %63
  br i1 %64, label %65, label %72

65:                                               ; preds = %61
  %66 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %66, i32 noundef -2)
  %67 = load ptr, ptr %4, align 8
  %68 = load i32, ptr %6, align 4
  %69 = sub i32 %68, 1
  %70 = load i32, ptr %7, align 4
  call void @set2(ptr noundef %67, i32 noundef %69, i32 noundef %70)
  %71 = load i32, ptr %7, align 4
  ret i32 %71

72:                                               ; preds = %61
  %73 = load ptr, ptr %4, align 8
  %74 = load i32, ptr %7, align 4
  %75 = load i32, ptr %8, align 4
  call void @set2(ptr noundef %73, i32 noundef %74, i32 noundef %75)
  br label %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @l_randomizePivot() #0 {
  %1 = alloca i64, align 8
  %2 = alloca i64, align 8
  %3 = alloca [4 x i32], align 16
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = call i64 @clock() #4
  store i64 %6, ptr %1, align 8
  %7 = call i64 @time(ptr noundef null) #4
  store i64 %7, ptr %2, align 8
  store i32 0, ptr %5, align 4
  %8 = getelementptr inbounds [4 x i32], ptr %3, i64 0, i64 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 16 %8, ptr align 8 %1, i64 8, i1 false)
  %9 = getelementptr inbounds [4 x i32], ptr %3, i64 0, i64 0
  %10 = getelementptr inbounds i32, ptr %9, i64 2
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %10, ptr align 8 %2, i64 8, i1 false)
  store i32 0, ptr %4, align 4
  br label %11

11:                                               ; preds = %22, %0
  %12 = load i32, ptr %4, align 4
  %13 = zext i32 %12 to i64
  %14 = icmp ult i64 %13, 4
  br i1 %14, label %15, label %25

15:                                               ; preds = %11
  %16 = load i32, ptr %4, align 4
  %17 = zext i32 %16 to i64
  %18 = getelementptr inbounds [4 x i32], ptr %3, i64 0, i64 %17
  %19 = load i32, ptr %18, align 4
  %20 = load i32, ptr %5, align 4
  %21 = add i32 %20, %19
  store i32 %21, ptr %5, align 4
  br label %22

22:                                               ; preds = %15
  %23 = load i32, ptr %4, align 4
  %24 = add i32 %23, 1
  store i32 %24, ptr %4, align 4
  br label %11, !llvm.loop !17

25:                                               ; preds = %11
  %26 = load i32, ptr %5, align 4
  ret i32 %26
}

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind
declare i64 @clock() #2

; Function Attrs: nounwind
declare i64 @time(ptr noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
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
!11 = distinct !{!11, !7}
!12 = distinct !{!12, !7}
!13 = distinct !{!13, !7}
!14 = distinct !{!14, !7}
!15 = distinct !{!15, !7}
!16 = distinct !{!16, !7}
!17 = distinct !{!17, !7}
