; ModuleID = 'loslib.c'
source_filename = "loslib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.tm = type { i32, i32, i32, i32, i32, i32, i32, i32, i32, i64, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }

@syslib = internal constant [12 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str, ptr @os_clock }, %struct.luaL_Reg { ptr @.str.1, ptr @os_date }, %struct.luaL_Reg { ptr @.str.2, ptr @os_difftime }, %struct.luaL_Reg { ptr @.str.3, ptr @os_execute }, %struct.luaL_Reg { ptr @.str.4, ptr @os_exit }, %struct.luaL_Reg { ptr @.str.5, ptr @os_getenv }, %struct.luaL_Reg { ptr @.str.6, ptr @os_remove }, %struct.luaL_Reg { ptr @.str.7, ptr @os_rename }, %struct.luaL_Reg { ptr @.str.8, ptr @os_setlocale }, %struct.luaL_Reg { ptr @.str.9, ptr @os_time }, %struct.luaL_Reg { ptr @.str.10, ptr @os_tmpname }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [6 x i8] c"clock\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"date\00", align 1
@.str.2 = private unnamed_addr constant [9 x i8] c"difftime\00", align 1
@.str.3 = private unnamed_addr constant [8 x i8] c"execute\00", align 1
@.str.4 = private unnamed_addr constant [5 x i8] c"exit\00", align 1
@.str.5 = private unnamed_addr constant [7 x i8] c"getenv\00", align 1
@.str.6 = private unnamed_addr constant [7 x i8] c"remove\00", align 1
@.str.7 = private unnamed_addr constant [7 x i8] c"rename\00", align 1
@.str.8 = private unnamed_addr constant [10 x i8] c"setlocale\00", align 1
@.str.9 = private unnamed_addr constant [5 x i8] c"time\00", align 1
@.str.10 = private unnamed_addr constant [8 x i8] c"tmpname\00", align 1
@.str.11 = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@.str.12 = private unnamed_addr constant [55 x i8] c"date result cannot be represented in this installation\00", align 1
@.str.13 = private unnamed_addr constant [3 x i8] c"*t\00", align 1
@.str.14 = private unnamed_addr constant [19 x i8] c"time out-of-bounds\00", align 1
@.str.15 = private unnamed_addr constant [5 x i8] c"year\00", align 1
@.str.16 = private unnamed_addr constant [6 x i8] c"month\00", align 1
@.str.17 = private unnamed_addr constant [4 x i8] c"day\00", align 1
@.str.18 = private unnamed_addr constant [5 x i8] c"hour\00", align 1
@.str.19 = private unnamed_addr constant [4 x i8] c"min\00", align 1
@.str.20 = private unnamed_addr constant [4 x i8] c"sec\00", align 1
@.str.21 = private unnamed_addr constant [5 x i8] c"yday\00", align 1
@.str.22 = private unnamed_addr constant [5 x i8] c"wday\00", align 1
@.str.23 = private unnamed_addr constant [6 x i8] c"isdst\00", align 1
@.str.24 = private unnamed_addr constant [78 x i8] c"aAbBcCdDeFgGhHIjmMnprRStTuUVwWxXyYzZ%||EcECExEXEyEYOdOeOHOIOmOMOSOuOUOVOwOWOy\00", align 1
@.str.25 = private unnamed_addr constant [36 x i8] c"invalid conversion specifier '%%%s'\00", align 1
@os_setlocale.cat = internal constant [6 x i32] [i32 6, i32 3, i32 0, i32 4, i32 1, i32 2], align 16
@os_setlocale.catnames = internal constant [7 x ptr] [ptr @.str.26, ptr @.str.27, ptr @.str.28, ptr @.str.29, ptr @.str.30, ptr @.str.9, ptr null], align 16
@.str.26 = private unnamed_addr constant [4 x i8] c"all\00", align 1
@.str.27 = private unnamed_addr constant [8 x i8] c"collate\00", align 1
@.str.28 = private unnamed_addr constant [6 x i8] c"ctype\00", align 1
@.str.29 = private unnamed_addr constant [9 x i8] c"monetary\00", align 1
@.str.30 = private unnamed_addr constant [8 x i8] c"numeric\00", align 1
@.str.31 = private unnamed_addr constant [55 x i8] c"time result cannot be represented in this installation\00", align 1
@.str.32 = private unnamed_addr constant [29 x i8] c"field '%s' is not an integer\00", align 1
@.str.33 = private unnamed_addr constant [33 x i8] c"field '%s' missing in date table\00", align 1
@.str.34 = private unnamed_addr constant [27 x i8] c"field '%s' is out-of-bound\00", align 1
@.str.35 = private unnamed_addr constant [37 x i8] c"unable to generate a unique filename\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_os(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 11)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @syslib, i32 noundef 0)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_clock(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i64 @clock() #7
  %5 = sitofp i64 %4 to double
  %6 = fdiv double %5, 1.000000e+06
  call void @lua_pushnumber(ptr noundef %3, double noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_date(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %struct.tm, align 8
  %9 = alloca ptr, align 8
  %10 = alloca [4 x i8], align 1
  %11 = alloca %struct.luaL_Buffer, align 8
  %12 = alloca i64, align 8
  %13 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = call ptr @luaL_optlstring(ptr noundef %14, i32 noundef 1, ptr noundef @.str.11, ptr noundef %4)
  store ptr %15, ptr %5, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = call i32 @lua_type(ptr noundef %16, i32 noundef 2)
  %18 = icmp sle i32 %17, 0
  br i1 %18, label %19, label %21

19:                                               ; preds = %1
  %20 = call i64 @time(ptr noundef null) #7
  br label %24

21:                                               ; preds = %1
  %22 = load ptr, ptr %3, align 8
  %23 = call i64 @l_checktime(ptr noundef %22, i32 noundef 2)
  br label %24

24:                                               ; preds = %21, %19
  %25 = phi i64 [ %20, %19 ], [ %23, %21 ]
  store i64 %25, ptr %6, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = load i64, ptr %4, align 8
  %28 = getelementptr inbounds i8, ptr %26, i64 %27
  store ptr %28, ptr %7, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = load i8, ptr %29, align 1
  %31 = sext i8 %30 to i32
  %32 = icmp eq i32 %31, 33
  br i1 %32, label %33, label %39

33:                                               ; preds = %24
  %34 = getelementptr inbounds %struct.tm, ptr %8, i32 0, i32 0
  %35 = load i32, ptr %34, align 8
  %36 = call ptr @gmtime(ptr noundef %6) #7
  store ptr %36, ptr %9, align 8
  %37 = load ptr, ptr %5, align 8
  %38 = getelementptr inbounds i8, ptr %37, i32 1
  store ptr %38, ptr %5, align 8
  br label %43

39:                                               ; preds = %24
  %40 = getelementptr inbounds %struct.tm, ptr %8, i32 0, i32 0
  %41 = load i32, ptr %40, align 8
  %42 = call ptr @localtime(ptr noundef %6) #7
  store ptr %42, ptr %9, align 8
  br label %43

43:                                               ; preds = %39, %33
  %44 = load ptr, ptr %9, align 8
  %45 = icmp eq ptr %44, null
  br i1 %45, label %46, label %49

46:                                               ; preds = %43
  %47 = load ptr, ptr %3, align 8
  %48 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %47, ptr noundef @.str.12)
  store i32 %48, ptr %2, align 4
  br label %115

49:                                               ; preds = %43
  %50 = load ptr, ptr %5, align 8
  %51 = call i32 @strcmp(ptr noundef %50, ptr noundef @.str.13) #8
  %52 = icmp eq i32 %51, 0
  br i1 %52, label %53, label %57

53:                                               ; preds = %49
  %54 = load ptr, ptr %3, align 8
  call void @lua_createtable(ptr noundef %54, i32 noundef 0, i32 noundef 9)
  %55 = load ptr, ptr %3, align 8
  %56 = load ptr, ptr %9, align 8
  call void @setallfields(ptr noundef %55, ptr noundef %56)
  br label %114

57:                                               ; preds = %49
  %58 = getelementptr inbounds [4 x i8], ptr %10, i64 0, i64 0
  store i8 37, ptr %58, align 1
  %59 = load ptr, ptr %3, align 8
  call void @luaL_buffinit(ptr noundef %59, ptr noundef %11)
  br label %60

60:                                               ; preds = %112, %57
  %61 = load ptr, ptr %5, align 8
  %62 = load ptr, ptr %7, align 8
  %63 = icmp ult ptr %61, %62
  br i1 %63, label %64, label %113

64:                                               ; preds = %60
  %65 = load ptr, ptr %5, align 8
  %66 = load i8, ptr %65, align 1
  %67 = sext i8 %66 to i32
  %68 = icmp ne i32 %67, 37
  br i1 %68, label %69, label %90

69:                                               ; preds = %64
  %70 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 2
  %71 = load i64, ptr %70, align 8
  %72 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 1
  %73 = load i64, ptr %72, align 8
  %74 = icmp ult i64 %71, %73
  br i1 %74, label %78, label %75

75:                                               ; preds = %69
  %76 = call ptr @luaL_prepbuffsize(ptr noundef %11, i64 noundef 1)
  %77 = icmp ne ptr %76, null
  br label %78

78:                                               ; preds = %75, %69
  %79 = phi i1 [ true, %69 ], [ %77, %75 ]
  %80 = zext i1 %79 to i32
  %81 = load ptr, ptr %5, align 8
  %82 = getelementptr inbounds i8, ptr %81, i32 1
  store ptr %82, ptr %5, align 8
  %83 = load i8, ptr %81, align 1
  %84 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 2
  %87 = load i64, ptr %86, align 8
  %88 = add i64 %87, 1
  store i64 %88, ptr %86, align 8
  %89 = getelementptr inbounds i8, ptr %85, i64 %87
  store i8 %83, ptr %89, align 1
  br label %112

90:                                               ; preds = %64
  %91 = call ptr @luaL_prepbuffsize(ptr noundef %11, i64 noundef 250)
  store ptr %91, ptr %13, align 8
  %92 = load ptr, ptr %5, align 8
  %93 = getelementptr inbounds i8, ptr %92, i32 1
  store ptr %93, ptr %5, align 8
  %94 = load ptr, ptr %3, align 8
  %95 = load ptr, ptr %5, align 8
  %96 = load ptr, ptr %7, align 8
  %97 = load ptr, ptr %5, align 8
  %98 = ptrtoint ptr %96 to i64
  %99 = ptrtoint ptr %97 to i64
  %100 = sub i64 %98, %99
  %101 = getelementptr inbounds [4 x i8], ptr %10, i64 0, i64 0
  %102 = getelementptr inbounds i8, ptr %101, i64 1
  %103 = call ptr @checkoption(ptr noundef %94, ptr noundef %95, i64 noundef %100, ptr noundef %102)
  store ptr %103, ptr %5, align 8
  %104 = load ptr, ptr %13, align 8
  %105 = getelementptr inbounds [4 x i8], ptr %10, i64 0, i64 0
  %106 = load ptr, ptr %9, align 8
  %107 = call i64 @strftime(ptr noundef %104, i64 noundef 250, ptr noundef %105, ptr noundef %106) #7
  store i64 %107, ptr %12, align 8
  %108 = load i64, ptr %12, align 8
  %109 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 2
  %110 = load i64, ptr %109, align 8
  %111 = add i64 %110, %108
  store i64 %111, ptr %109, align 8
  br label %112

112:                                              ; preds = %90, %78
  br label %60, !llvm.loop !6

113:                                              ; preds = %60
  call void @luaL_pushresult(ptr noundef %11)
  br label %114

114:                                              ; preds = %113, %53
  store i32 1, ptr %2, align 4
  br label %115

115:                                              ; preds = %114, %46
  %116 = load i32, ptr %2, align 4
  ret i32 %116
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_difftime(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i64 @l_checktime(ptr noundef %5, i32 noundef 1)
  store i64 %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call i64 @l_checktime(ptr noundef %7, i32 noundef 2)
  store i64 %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = load i64, ptr %3, align 8
  %11 = load i64, ptr %4, align 8
  %12 = call double @difftime(i64 noundef %10, i64 noundef %11) #9
  call void @lua_pushnumber(ptr noundef %9, double noundef %12)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_execute(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @luaL_optlstring(ptr noundef %6, i32 noundef 1, ptr noundef null, ptr noundef null)
  store ptr %7, ptr %4, align 8
  %8 = call ptr @__errno_location() #9
  store i32 0, ptr %8, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @system(ptr noundef %9)
  store i32 %10, ptr %5, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = icmp ne ptr %11, null
  br i1 %12, label %13, label %17

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = load i32, ptr %5, align 4
  %16 = call i32 @luaL_execresult(ptr noundef %14, i32 noundef %15)
  store i32 %16, ptr %2, align 4
  br label %20

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  %19 = load i32, ptr %5, align 4
  call void @lua_pushboolean(ptr noundef %18, i32 noundef %19)
  store i32 1, ptr %2, align 4
  br label %20

20:                                               ; preds = %17, %13
  %21 = load i32, ptr %2, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_exit(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef 1)
  %6 = icmp eq i32 %5, 1
  br i1 %6, label %7, label %13

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_toboolean(ptr noundef %8, i32 noundef 1)
  %10 = icmp ne i32 %9, 0
  %11 = zext i1 %10 to i64
  %12 = select i1 %10, i32 0, i32 1
  store i32 %12, ptr %3, align 4
  br label %17

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  %15 = call i64 @luaL_optinteger(ptr noundef %14, i32 noundef 1, i64 noundef 0)
  %16 = trunc i64 %15 to i32
  store i32 %16, ptr %3, align 4
  br label %17

17:                                               ; preds = %13, %7
  %18 = load ptr, ptr %2, align 8
  %19 = call i32 @lua_toboolean(ptr noundef %18, i32 noundef 2)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %23

21:                                               ; preds = %17
  %22 = load ptr, ptr %2, align 8
  call void @lua_close(ptr noundef %22)
  br label %23

23:                                               ; preds = %21, %17
  %24 = load ptr, ptr %2, align 8
  %25 = icmp ne ptr %24, null
  br i1 %25, label %26, label %28

26:                                               ; preds = %23
  %27 = load i32, ptr %3, align 4
  call void @exit(i32 noundef %27) #10
  unreachable

28:                                               ; preds = %23
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_getenv(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checklstring(ptr noundef %4, i32 noundef 1, ptr noundef null)
  %6 = call ptr @getenv(ptr noundef %5) #7
  %7 = call ptr @lua_pushstring(ptr noundef %3, ptr noundef %6)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_remove(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checklstring(ptr noundef %4, i32 noundef 1, ptr noundef null)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #9
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @remove(ptr noundef %8) #7
  %10 = icmp eq i32 %9, 0
  %11 = zext i1 %10 to i32
  %12 = load ptr, ptr %3, align 8
  %13 = call i32 @luaL_fileresult(ptr noundef %7, i32 noundef %11, ptr noundef %12)
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_rename(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaL_checklstring(ptr noundef %5, i32 noundef 1, ptr noundef null)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 2, ptr noundef null)
  store ptr %8, ptr %4, align 8
  %9 = call ptr @__errno_location() #9
  store i32 0, ptr %9, align 4
  %10 = load ptr, ptr %2, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = call i32 @rename(ptr noundef %11, ptr noundef %12) #7
  %14 = icmp eq i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = call i32 @luaL_fileresult(ptr noundef %10, i32 noundef %15, ptr noundef null)
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_setlocale(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaL_optlstring(ptr noundef %5, i32 noundef 1, ptr noundef null, ptr noundef null)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call i32 @luaL_checkoption(ptr noundef %7, i32 noundef 2, ptr noundef @.str.26, ptr noundef @os_setlocale.catnames)
  store i32 %8, ptr %4, align 4
  %9 = load ptr, ptr %2, align 8
  %10 = load i32, ptr %4, align 4
  %11 = sext i32 %10 to i64
  %12 = getelementptr inbounds [6 x i32], ptr @os_setlocale.cat, i64 0, i64 %11
  %13 = load i32, ptr %12, align 4
  %14 = load ptr, ptr %3, align 8
  %15 = call ptr @setlocale(i32 noundef %13, ptr noundef %14) #7
  %16 = call ptr @lua_pushstring(ptr noundef %9, ptr noundef %15)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_time(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca %struct.tm, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_type(ptr noundef %6, i32 noundef 1)
  %8 = icmp sle i32 %7, 0
  br i1 %8, label %9, label %11

9:                                                ; preds = %1
  %10 = call i64 @time(ptr noundef null) #7
  store i64 %10, ptr %4, align 8
  br label %37

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  call void @luaL_checktype(ptr noundef %12, i32 noundef 1, i32 noundef 5)
  %13 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %13, i32 noundef 1)
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @getfield(ptr noundef %14, ptr noundef @.str.15, i32 noundef -1, i32 noundef 1900)
  %16 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 5
  store i32 %15, ptr %16, align 4
  %17 = load ptr, ptr %3, align 8
  %18 = call i32 @getfield(ptr noundef %17, ptr noundef @.str.16, i32 noundef -1, i32 noundef 1)
  %19 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 4
  store i32 %18, ptr %19, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = call i32 @getfield(ptr noundef %20, ptr noundef @.str.17, i32 noundef -1, i32 noundef 0)
  %22 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 3
  store i32 %21, ptr %22, align 4
  %23 = load ptr, ptr %3, align 8
  %24 = call i32 @getfield(ptr noundef %23, ptr noundef @.str.18, i32 noundef 12, i32 noundef 0)
  %25 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 2
  store i32 %24, ptr %25, align 8
  %26 = load ptr, ptr %3, align 8
  %27 = call i32 @getfield(ptr noundef %26, ptr noundef @.str.19, i32 noundef 0, i32 noundef 0)
  %28 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 1
  store i32 %27, ptr %28, align 4
  %29 = load ptr, ptr %3, align 8
  %30 = call i32 @getfield(ptr noundef %29, ptr noundef @.str.20, i32 noundef 0, i32 noundef 0)
  %31 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 0
  store i32 %30, ptr %31, align 8
  %32 = load ptr, ptr %3, align 8
  %33 = call i32 @getboolfield(ptr noundef %32, ptr noundef @.str.23)
  %34 = getelementptr inbounds %struct.tm, ptr %5, i32 0, i32 8
  store i32 %33, ptr %34, align 8
  %35 = call i64 @mktime(ptr noundef %5) #7
  store i64 %35, ptr %4, align 8
  %36 = load ptr, ptr %3, align 8
  call void @setallfields(ptr noundef %36, ptr noundef %5)
  br label %37

37:                                               ; preds = %11, %9
  %38 = load i64, ptr %4, align 8
  %39 = load i64, ptr %4, align 8
  %40 = icmp ne i64 %38, %39
  br i1 %40, label %44, label %41

41:                                               ; preds = %37
  %42 = load i64, ptr %4, align 8
  %43 = icmp eq i64 %42, -1
  br i1 %43, label %44, label %47

44:                                               ; preds = %41, %37
  %45 = load ptr, ptr %3, align 8
  %46 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %45, ptr noundef @.str.31)
  store i32 %46, ptr %2, align 4
  br label %50

47:                                               ; preds = %41
  %48 = load ptr, ptr %3, align 8
  %49 = load i64, ptr %4, align 8
  call void @lua_pushinteger(ptr noundef %48, i64 noundef %49)
  store i32 1, ptr %2, align 4
  br label %50

50:                                               ; preds = %47, %44
  %51 = load i32, ptr %2, align 4
  ret i32 %51
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @os_tmpname(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca [20 x i8], align 16
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %6 = getelementptr inbounds [20 x i8], ptr %4, i64 0, i64 0
  %7 = call ptr @tmpnam(ptr noundef %6) #7
  %8 = icmp eq ptr %7, null
  %9 = zext i1 %8 to i32
  store i32 %9, ptr %5, align 4
  %10 = load i32, ptr %5, align 4
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %1
  %16 = load ptr, ptr %3, align 8
  %17 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %16, ptr noundef @.str.35)
  store i32 %17, ptr %2, align 4
  br label %22

18:                                               ; preds = %1
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds [20 x i8], ptr %4, i64 0, i64 0
  %21 = call ptr @lua_pushstring(ptr noundef %19, ptr noundef %20)
  store i32 1, ptr %2, align 4
  br label %22

22:                                               ; preds = %18, %15
  %23 = load i32, ptr %2, align 4
  ret i32 %23
}

declare void @lua_pushnumber(ptr noundef, double noundef) #1

; Function Attrs: nounwind
declare i64 @clock() #2

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind
declare i64 @time(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @l_checktime(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = load i32, ptr %4, align 4
  %8 = call i64 @luaL_checkinteger(ptr noundef %6, i32 noundef %7)
  store i64 %8, ptr %5, align 8
  %9 = load i64, ptr %5, align 8
  %10 = load i64, ptr %5, align 8
  %11 = icmp eq i64 %9, %10
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %22, label %17

17:                                               ; preds = %2
  %18 = load ptr, ptr %3, align 8
  %19 = load i32, ptr %4, align 4
  %20 = call i32 @luaL_argerror(ptr noundef %18, i32 noundef %19, ptr noundef @.str.14)
  %21 = icmp ne i32 %20, 0
  br label %22

22:                                               ; preds = %17, %2
  %23 = phi i1 [ true, %2 ], [ %21, %17 ]
  %24 = zext i1 %23 to i32
  %25 = load i64, ptr %5, align 8
  ret i64 %25
}

; Function Attrs: nounwind
declare ptr @gmtime(ptr noundef) #2

; Function Attrs: nounwind
declare ptr @localtime(ptr noundef) #2

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setallfields(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.tm, ptr %6, i32 0, i32 5
  %8 = load i32, ptr %7, align 4
  call void @setfield(ptr noundef %5, ptr noundef @.str.15, i32 noundef %8, i32 noundef 1900)
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.tm, ptr %10, i32 0, i32 4
  %12 = load i32, ptr %11, align 8
  call void @setfield(ptr noundef %9, ptr noundef @.str.16, i32 noundef %12, i32 noundef 1)
  %13 = load ptr, ptr %3, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.tm, ptr %14, i32 0, i32 3
  %16 = load i32, ptr %15, align 4
  call void @setfield(ptr noundef %13, ptr noundef @.str.17, i32 noundef %16, i32 noundef 0)
  %17 = load ptr, ptr %3, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.tm, ptr %18, i32 0, i32 2
  %20 = load i32, ptr %19, align 8
  call void @setfield(ptr noundef %17, ptr noundef @.str.18, i32 noundef %20, i32 noundef 0)
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.tm, ptr %22, i32 0, i32 1
  %24 = load i32, ptr %23, align 4
  call void @setfield(ptr noundef %21, ptr noundef @.str.19, i32 noundef %24, i32 noundef 0)
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.tm, ptr %26, i32 0, i32 0
  %28 = load i32, ptr %27, align 8
  call void @setfield(ptr noundef %25, ptr noundef @.str.20, i32 noundef %28, i32 noundef 0)
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.tm, ptr %30, i32 0, i32 7
  %32 = load i32, ptr %31, align 4
  call void @setfield(ptr noundef %29, ptr noundef @.str.21, i32 noundef %32, i32 noundef 1)
  %33 = load ptr, ptr %3, align 8
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.tm, ptr %34, i32 0, i32 6
  %36 = load i32, ptr %35, align 8
  call void @setfield(ptr noundef %33, ptr noundef @.str.22, i32 noundef %36, i32 noundef 1)
  %37 = load ptr, ptr %3, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.tm, ptr %38, i32 0, i32 8
  %40 = load i32, ptr %39, align 8
  call void @setboolfield(ptr noundef %37, ptr noundef @.str.23, i32 noundef %40)
  ret void
}

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

declare ptr @luaL_prepbuffsize(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @checkoption(ptr noundef %0, ptr noundef %1, i64 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i64 %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr @.str.24, ptr %10, align 8
  store i32 1, ptr %11, align 4
  br label %12

12:                                               ; preds = %54, %4
  %13 = load ptr, ptr %10, align 8
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %22

17:                                               ; preds = %12
  %18 = load i32, ptr %11, align 4
  %19 = sext i32 %18 to i64
  %20 = load i64, ptr %8, align 8
  %21 = icmp sle i64 %19, %20
  br label %22

22:                                               ; preds = %17, %12
  %23 = phi i1 [ false, %12 ], [ %21, %17 ]
  br i1 %23, label %24, label %59

24:                                               ; preds = %22
  %25 = load ptr, ptr %10, align 8
  %26 = load i8, ptr %25, align 1
  %27 = sext i8 %26 to i32
  %28 = icmp eq i32 %27, 124
  br i1 %28, label %29, label %32

29:                                               ; preds = %24
  %30 = load i32, ptr %11, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, ptr %11, align 4
  br label %53

32:                                               ; preds = %24
  %33 = load ptr, ptr %7, align 8
  %34 = load ptr, ptr %10, align 8
  %35 = load i32, ptr %11, align 4
  %36 = sext i32 %35 to i64
  %37 = call i32 @memcmp(ptr noundef %33, ptr noundef %34, i64 noundef %36) #8
  %38 = icmp eq i32 %37, 0
  br i1 %38, label %39, label %52

39:                                               ; preds = %32
  %40 = load ptr, ptr %9, align 8
  %41 = load ptr, ptr %7, align 8
  %42 = load i32, ptr %11, align 4
  %43 = sext i32 %42 to i64
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %40, ptr align 1 %41, i64 %43, i1 false)
  %44 = load ptr, ptr %9, align 8
  %45 = load i32, ptr %11, align 4
  %46 = sext i32 %45 to i64
  %47 = getelementptr inbounds i8, ptr %44, i64 %46
  store i8 0, ptr %47, align 1
  %48 = load ptr, ptr %7, align 8
  %49 = load i32, ptr %11, align 4
  %50 = sext i32 %49 to i64
  %51 = getelementptr inbounds i8, ptr %48, i64 %50
  store ptr %51, ptr %5, align 8
  br label %66

52:                                               ; preds = %32
  br label %53

53:                                               ; preds = %52, %29
  br label %54

54:                                               ; preds = %53
  %55 = load i32, ptr %11, align 4
  %56 = load ptr, ptr %10, align 8
  %57 = sext i32 %55 to i64
  %58 = getelementptr inbounds i8, ptr %56, i64 %57
  store ptr %58, ptr %10, align 8
  br label %12, !llvm.loop !8

59:                                               ; preds = %22
  %60 = load ptr, ptr %6, align 8
  %61 = load ptr, ptr %6, align 8
  %62 = load ptr, ptr %7, align 8
  %63 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %61, ptr noundef @.str.25, ptr noundef %62)
  %64 = call i32 @luaL_argerror(ptr noundef %60, i32 noundef 1, ptr noundef %63)
  %65 = load ptr, ptr %7, align 8
  store ptr %65, ptr %5, align 8
  br label %66

66:                                               ; preds = %59, %39
  %67 = load ptr, ptr %5, align 8
  ret ptr %67
}

; Function Attrs: nounwind
declare i64 @strftime(ptr noundef, i64 noundef, ptr noundef, ptr noundef) #2

declare void @luaL_pushresult(ptr noundef) #1

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setfield(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %7, align 4
  %11 = sext i32 %10 to i64
  %12 = load i32, ptr %8, align 4
  %13 = sext i32 %12 to i64
  %14 = add nsw i64 %11, %13
  call void @lua_pushinteger(ptr noundef %9, i64 noundef %14)
  %15 = load ptr, ptr %5, align 8
  %16 = load ptr, ptr %6, align 8
  call void @lua_setfield(ptr noundef %15, i32 noundef -2, ptr noundef %16)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setboolfield(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load i32, ptr %6, align 4
  %8 = icmp slt i32 %7, 0
  br i1 %8, label %9, label %10

9:                                                ; preds = %3
  br label %15

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %6, align 4
  call void @lua_pushboolean(ptr noundef %11, i32 noundef %12)
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %13, i32 noundef -2, ptr noundef %14)
  br label %15

15:                                               ; preds = %10, %9
  ret void
}

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @memcmp(ptr noundef, ptr noundef, i64 noundef) #3

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #4

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: nounwind willreturn memory(none)
declare double @difftime(i64 noundef, i64 noundef) #5

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__errno_location() #5

declare i32 @system(ptr noundef) #1

declare i32 @luaL_execresult(ptr noundef, i32 noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_close(ptr noundef) #1

; Function Attrs: noreturn nounwind
declare void @exit(i32 noundef) #6

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_fileresult(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: nounwind
declare i32 @remove(ptr noundef) #2

; Function Attrs: nounwind
declare i32 @rename(ptr noundef, ptr noundef) #2

declare i32 @luaL_checkoption(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @setlocale(i32 noundef, ptr noundef) #2

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getfield(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = call i32 @lua_getfield(ptr noundef %13, i32 noundef -1, ptr noundef %14)
  store i32 %15, ptr %11, align 4
  %16 = load ptr, ptr %6, align 8
  %17 = call i64 @lua_tointegerx(ptr noundef %16, i32 noundef -1, ptr noundef %10)
  store i64 %17, ptr %12, align 8
  %18 = load i32, ptr %10, align 4
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %48, label %20

20:                                               ; preds = %4
  %21 = load i32, ptr %11, align 4
  %22 = icmp ne i32 %21, 0
  %23 = zext i1 %22 to i32
  %24 = icmp ne i32 %23, 0
  %25 = zext i1 %24 to i32
  %26 = sext i32 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %28, label %32

28:                                               ; preds = %20
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %7, align 8
  %31 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %29, ptr noundef @.str.32, ptr noundef %30)
  store i32 %31, ptr %5, align 4
  br label %76

32:                                               ; preds = %20
  %33 = load i32, ptr %8, align 4
  %34 = icmp slt i32 %33, 0
  %35 = zext i1 %34 to i32
  %36 = icmp ne i32 %35, 0
  %37 = zext i1 %36 to i32
  %38 = sext i32 %37 to i64
  %39 = icmp ne i64 %38, 0
  br i1 %39, label %40, label %44

40:                                               ; preds = %32
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %41, ptr noundef @.str.33, ptr noundef %42)
  store i32 %43, ptr %5, align 4
  br label %76

44:                                               ; preds = %32
  br label %45

45:                                               ; preds = %44
  %46 = load i32, ptr %8, align 4
  %47 = sext i32 %46 to i64
  store i64 %47, ptr %12, align 8
  br label %72

48:                                               ; preds = %4
  %49 = load i64, ptr %12, align 8
  %50 = icmp sge i64 %49, 0
  br i1 %50, label %51, label %57

51:                                               ; preds = %48
  %52 = load i64, ptr %12, align 8
  %53 = load i32, ptr %9, align 4
  %54 = sext i32 %53 to i64
  %55 = sub nsw i64 %52, %54
  %56 = icmp sle i64 %55, 2147483647
  br i1 %56, label %67, label %63

57:                                               ; preds = %48
  %58 = load i32, ptr %9, align 4
  %59 = add nsw i32 -2147483648, %58
  %60 = sext i32 %59 to i64
  %61 = load i64, ptr %12, align 8
  %62 = icmp sle i64 %60, %61
  br i1 %62, label %67, label %63

63:                                               ; preds = %57, %51
  %64 = load ptr, ptr %6, align 8
  %65 = load ptr, ptr %7, align 8
  %66 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %64, ptr noundef @.str.34, ptr noundef %65)
  store i32 %66, ptr %5, align 4
  br label %76

67:                                               ; preds = %57, %51
  %68 = load i32, ptr %9, align 4
  %69 = sext i32 %68 to i64
  %70 = load i64, ptr %12, align 8
  %71 = sub nsw i64 %70, %69
  store i64 %71, ptr %12, align 8
  br label %72

72:                                               ; preds = %67, %45
  %73 = load ptr, ptr %6, align 8
  call void @lua_settop(ptr noundef %73, i32 noundef -2)
  %74 = load i64, ptr %12, align 8
  %75 = trunc i64 %74 to i32
  store i32 %75, ptr %5, align 4
  br label %76

76:                                               ; preds = %72, %63, %40, %28
  %77 = load i32, ptr %5, align 4
  ret i32 %77
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getboolfield(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @lua_getfield(ptr noundef %6, i32 noundef -1, ptr noundef %7)
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %10, label %11

10:                                               ; preds = %2
  br label %14

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  %13 = call i32 @lua_toboolean(ptr noundef %12, i32 noundef -1)
  br label %14

14:                                               ; preds = %11, %10
  %15 = phi i32 [ -1, %10 ], [ %13, %11 ]
  store i32 %15, ptr %5, align 4
  %16 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %16, i32 noundef -2)
  %17 = load i32, ptr %5, align 4
  ret i32 %17
}

; Function Attrs: nounwind
declare i64 @mktime(ptr noundef) #2

declare i32 @lua_getfield(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @tmpnam(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { noreturn nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #7 = { nounwind }
attributes #8 = { nounwind willreturn memory(read) }
attributes #9 = { nounwind willreturn memory(none) }
attributes #10 = { noreturn nounwind }

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
