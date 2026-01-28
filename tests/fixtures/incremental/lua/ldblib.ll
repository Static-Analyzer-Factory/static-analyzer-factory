; ModuleID = 'ldblib.c'
source_filename = "ldblib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.lua_Debug = type { i32, ptr, ptr, ptr, ptr, i64, i32, i32, i32, i8, i8, i8, i8, i16, i16, [60 x i8], ptr }

@dblib = internal constant [18 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str, ptr @db_debug }, %struct.luaL_Reg { ptr @.str.1, ptr @db_getuservalue }, %struct.luaL_Reg { ptr @.str.2, ptr @db_gethook }, %struct.luaL_Reg { ptr @.str.3, ptr @db_getinfo }, %struct.luaL_Reg { ptr @.str.4, ptr @db_getlocal }, %struct.luaL_Reg { ptr @.str.5, ptr @db_getregistry }, %struct.luaL_Reg { ptr @.str.6, ptr @db_getmetatable }, %struct.luaL_Reg { ptr @.str.7, ptr @db_getupvalue }, %struct.luaL_Reg { ptr @.str.8, ptr @db_upvaluejoin }, %struct.luaL_Reg { ptr @.str.9, ptr @db_upvalueid }, %struct.luaL_Reg { ptr @.str.10, ptr @db_setuservalue }, %struct.luaL_Reg { ptr @.str.11, ptr @db_sethook }, %struct.luaL_Reg { ptr @.str.12, ptr @db_setlocal }, %struct.luaL_Reg { ptr @.str.13, ptr @db_setmetatable }, %struct.luaL_Reg { ptr @.str.14, ptr @db_setupvalue }, %struct.luaL_Reg { ptr @.str.15, ptr @db_traceback }, %struct.luaL_Reg { ptr @.str.16, ptr @db_setcstacklimit }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [6 x i8] c"debug\00", align 1
@.str.1 = private unnamed_addr constant [13 x i8] c"getuservalue\00", align 1
@.str.2 = private unnamed_addr constant [8 x i8] c"gethook\00", align 1
@.str.3 = private unnamed_addr constant [8 x i8] c"getinfo\00", align 1
@.str.4 = private unnamed_addr constant [9 x i8] c"getlocal\00", align 1
@.str.5 = private unnamed_addr constant [12 x i8] c"getregistry\00", align 1
@.str.6 = private unnamed_addr constant [13 x i8] c"getmetatable\00", align 1
@.str.7 = private unnamed_addr constant [11 x i8] c"getupvalue\00", align 1
@.str.8 = private unnamed_addr constant [12 x i8] c"upvaluejoin\00", align 1
@.str.9 = private unnamed_addr constant [10 x i8] c"upvalueid\00", align 1
@.str.10 = private unnamed_addr constant [13 x i8] c"setuservalue\00", align 1
@.str.11 = private unnamed_addr constant [8 x i8] c"sethook\00", align 1
@.str.12 = private unnamed_addr constant [9 x i8] c"setlocal\00", align 1
@.str.13 = private unnamed_addr constant [13 x i8] c"setmetatable\00", align 1
@.str.14 = private unnamed_addr constant [11 x i8] c"setupvalue\00", align 1
@.str.15 = private unnamed_addr constant [10 x i8] c"traceback\00", align 1
@.str.16 = private unnamed_addr constant [15 x i8] c"setcstacklimit\00", align 1
@stderr = external global ptr, align 8
@.str.17 = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.str.18 = private unnamed_addr constant [12 x i8] c"lua_debug> \00", align 1
@stdin = external global ptr, align 8
@.str.19 = private unnamed_addr constant [6 x i8] c"cont\0A\00", align 1
@.str.20 = private unnamed_addr constant [17 x i8] c"=(debug command)\00", align 1
@.str.21 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.str.22 = private unnamed_addr constant [14 x i8] c"external hook\00", align 1
@.str.23 = private unnamed_addr constant [9 x i8] c"_HOOKKEY\00", align 1
@hookf.hooknames = internal constant [5 x ptr] [ptr @.str.24, ptr @.str.25, ptr @.str.26, ptr @.str.27, ptr @.str.28], align 16
@.str.24 = private unnamed_addr constant [5 x i8] c"call\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"return\00", align 1
@.str.26 = private unnamed_addr constant [5 x i8] c"line\00", align 1
@.str.27 = private unnamed_addr constant [6 x i8] c"count\00", align 1
@.str.28 = private unnamed_addr constant [10 x i8] c"tail call\00", align 1
@.str.29 = private unnamed_addr constant [15 x i8] c"stack overflow\00", align 1
@.str.30 = private unnamed_addr constant [8 x i8] c"flnSrtu\00", align 1
@.str.31 = private unnamed_addr constant [19 x i8] c"invalid option '>'\00", align 1
@.str.32 = private unnamed_addr constant [4 x i8] c">%s\00", align 1
@.str.33 = private unnamed_addr constant [15 x i8] c"invalid option\00", align 1
@.str.34 = private unnamed_addr constant [7 x i8] c"source\00", align 1
@.str.35 = private unnamed_addr constant [10 x i8] c"short_src\00", align 1
@.str.36 = private unnamed_addr constant [12 x i8] c"linedefined\00", align 1
@.str.37 = private unnamed_addr constant [16 x i8] c"lastlinedefined\00", align 1
@.str.38 = private unnamed_addr constant [5 x i8] c"what\00", align 1
@.str.39 = private unnamed_addr constant [12 x i8] c"currentline\00", align 1
@.str.40 = private unnamed_addr constant [5 x i8] c"nups\00", align 1
@.str.41 = private unnamed_addr constant [8 x i8] c"nparams\00", align 1
@.str.42 = private unnamed_addr constant [9 x i8] c"isvararg\00", align 1
@.str.43 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.44 = private unnamed_addr constant [9 x i8] c"namewhat\00", align 1
@.str.45 = private unnamed_addr constant [10 x i8] c"ftransfer\00", align 1
@.str.46 = private unnamed_addr constant [10 x i8] c"ntransfer\00", align 1
@.str.47 = private unnamed_addr constant [11 x i8] c"istailcall\00", align 1
@.str.48 = private unnamed_addr constant [12 x i8] c"activelines\00", align 1
@.str.49 = private unnamed_addr constant [5 x i8] c"func\00", align 1
@.str.50 = private unnamed_addr constant [19 x i8] c"level out of range\00", align 1
@.str.51 = private unnamed_addr constant [22 x i8] c"Lua function expected\00", align 1
@.str.52 = private unnamed_addr constant [22 x i8] c"invalid upvalue index\00", align 1
@.str.53 = private unnamed_addr constant [2 x i8] c"k\00", align 1
@.str.54 = private unnamed_addr constant [7 x i8] c"__mode\00", align 1
@.str.55 = private unnamed_addr constant [13 x i8] c"nil or table\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_debug(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 17)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @dblib, i32 noundef 0)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_debug(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca [250 x i8], align 16
  store ptr %0, ptr %2, align 8
  br label %4

4:                                                ; preds = %36, %1
  %5 = load ptr, ptr @stderr, align 8
  %6 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %5, ptr noundef @.str.17, ptr noundef @.str.18)
  %7 = load ptr, ptr @stderr, align 8
  %8 = call i32 @fflush(ptr noundef %7)
  %9 = getelementptr inbounds [250 x i8], ptr %3, i64 0, i64 0
  %10 = load ptr, ptr @stdin, align 8
  %11 = call ptr @fgets(ptr noundef %9, i32 noundef 250, ptr noundef %10)
  %12 = icmp eq ptr %11, null
  br i1 %12, label %17, label %13

13:                                               ; preds = %4
  %14 = getelementptr inbounds [250 x i8], ptr %3, i64 0, i64 0
  %15 = call i32 @strcmp(ptr noundef %14, ptr noundef @.str.19) #3
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %18

17:                                               ; preds = %13, %4
  ret i32 0

18:                                               ; preds = %13
  %19 = load ptr, ptr %2, align 8
  %20 = getelementptr inbounds [250 x i8], ptr %3, i64 0, i64 0
  %21 = getelementptr inbounds [250 x i8], ptr %3, i64 0, i64 0
  %22 = call i64 @strlen(ptr noundef %21) #3
  %23 = call i32 @luaL_loadbufferx(ptr noundef %19, ptr noundef %20, i64 noundef %22, ptr noundef @.str.20, ptr noundef null)
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %29, label %25

25:                                               ; preds = %18
  %26 = load ptr, ptr %2, align 8
  %27 = call i32 @lua_pcallk(ptr noundef %26, i32 noundef 0, i32 noundef 0, i32 noundef 0, i64 noundef 0, ptr noundef null)
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %36

29:                                               ; preds = %25, %18
  %30 = load ptr, ptr @stderr, align 8
  %31 = load ptr, ptr %2, align 8
  %32 = call ptr @luaL_tolstring(ptr noundef %31, i32 noundef -1, ptr noundef null)
  %33 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %30, ptr noundef @.str.21, ptr noundef %32)
  %34 = load ptr, ptr @stderr, align 8
  %35 = call i32 @fflush(ptr noundef %34)
  br label %36

36:                                               ; preds = %29, %25
  %37 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %37, i32 noundef 0)
  br label %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getuservalue(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call i64 @luaL_optinteger(ptr noundef %5, i32 noundef 2, i64 noundef 1)
  %7 = trunc i64 %6 to i32
  store i32 %7, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 1)
  %10 = icmp ne i32 %9, 7
  br i1 %10, label %11, label %13

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %12)
  br label %21

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = load i32, ptr %4, align 4
  %16 = call i32 @lua_getiuservalue(ptr noundef %14, i32 noundef 1, i32 noundef %15)
  %17 = icmp ne i32 %16, -1
  br i1 %17, label %18, label %20

18:                                               ; preds = %13
  %19 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %19, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %22

20:                                               ; preds = %13
  br label %21

21:                                               ; preds = %20, %11
  store i32 1, ptr %2, align 4
  br label %22

22:                                               ; preds = %21, %18
  %23 = load i32, ptr %2, align 4
  ret i32 %23
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_gethook(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca [5 x i8], align 1
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @getthread(ptr noundef %9, ptr noundef %4)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = call i32 @lua_gethookmask(ptr noundef %11)
  store i32 %12, ptr %7, align 4
  %13 = load ptr, ptr %5, align 8
  %14 = call ptr @lua_gethook(ptr noundef %13)
  store ptr %14, ptr %8, align 8
  %15 = load ptr, ptr %8, align 8
  %16 = icmp eq ptr %15, null
  br i1 %16, label %17, label %19

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %18)
  store i32 1, ptr %2, align 4
  br label %49

19:                                               ; preds = %1
  %20 = load ptr, ptr %8, align 8
  %21 = icmp ne ptr %20, @hookf
  br i1 %21, label %22, label %25

22:                                               ; preds = %19
  %23 = load ptr, ptr %3, align 8
  %24 = call ptr @lua_pushstring(ptr noundef %23, ptr noundef @.str.22)
  br label %38

25:                                               ; preds = %19
  %26 = load ptr, ptr %3, align 8
  %27 = call i32 @lua_getfield(ptr noundef %26, i32 noundef -1001000, ptr noundef @.str.23)
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %5, align 8
  call void @checkstack(ptr noundef %28, ptr noundef %29, i32 noundef 1)
  %30 = load ptr, ptr %5, align 8
  %31 = call i32 @lua_pushthread(ptr noundef %30)
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %3, align 8
  call void @lua_xmove(ptr noundef %32, ptr noundef %33, i32 noundef 1)
  %34 = load ptr, ptr %3, align 8
  %35 = call i32 @lua_rawget(ptr noundef %34, i32 noundef -2)
  %36 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %36, i32 noundef -2, i32 noundef -1)
  %37 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %37, i32 noundef -2)
  br label %38

38:                                               ; preds = %25, %22
  br label %39

39:                                               ; preds = %38
  %40 = load ptr, ptr %3, align 8
  %41 = load i32, ptr %7, align 4
  %42 = getelementptr inbounds [5 x i8], ptr %6, i64 0, i64 0
  %43 = call ptr @unmakemask(i32 noundef %41, ptr noundef %42)
  %44 = call ptr @lua_pushstring(ptr noundef %40, ptr noundef %43)
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = call i32 @lua_gethookcount(ptr noundef %46)
  %48 = sext i32 %47 to i64
  call void @lua_pushinteger(ptr noundef %45, i64 noundef %48)
  store i32 3, ptr %2, align 4
  br label %49

49:                                               ; preds = %39, %17
  %50 = load i32, ptr %2, align 4
  ret i32 %50
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getinfo(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca %struct.lua_Debug, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @getthread(ptr noundef %8, ptr noundef %5)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %5, align 4
  %12 = add nsw i32 %11, 2
  %13 = call ptr @luaL_optlstring(ptr noundef %10, i32 noundef %12, ptr noundef @.str.30, ptr noundef null)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %6, align 8
  call void @checkstack(ptr noundef %14, ptr noundef %15, i32 noundef 3)
  %16 = load ptr, ptr %7, align 8
  %17 = getelementptr inbounds i8, ptr %16, i64 0
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp ne i32 %19, 62
  %21 = zext i1 %20 to i32
  %22 = icmp ne i32 %21, 0
  %23 = zext i1 %22 to i32
  %24 = sext i32 %23 to i64
  %25 = icmp ne i64 %24, 0
  br i1 %25, label %32, label %26

26:                                               ; preds = %1
  %27 = load ptr, ptr %3, align 8
  %28 = load i32, ptr %5, align 4
  %29 = add nsw i32 %28, 2
  %30 = call i32 @luaL_argerror(ptr noundef %27, i32 noundef %29, ptr noundef @.str.31)
  %31 = icmp ne i32 %30, 0
  br label %32

32:                                               ; preds = %26, %1
  %33 = phi i1 [ true, %1 ], [ %31, %26 ]
  %34 = zext i1 %33 to i32
  %35 = load ptr, ptr %3, align 8
  %36 = load i32, ptr %5, align 4
  %37 = add nsw i32 %36, 1
  %38 = call i32 @lua_type(ptr noundef %35, i32 noundef %37)
  %39 = icmp eq i32 %38, 6
  br i1 %39, label %40, label %49

40:                                               ; preds = %32
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %41, ptr noundef @.str.32, ptr noundef %42)
  store ptr %43, ptr %7, align 8
  %44 = load ptr, ptr %3, align 8
  %45 = load i32, ptr %5, align 4
  %46 = add nsw i32 %45, 1
  call void @lua_pushvalue(ptr noundef %44, i32 noundef %46)
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %6, align 8
  call void @lua_xmove(ptr noundef %47, ptr noundef %48, i32 noundef 1)
  br label %61

49:                                               ; preds = %32
  %50 = load ptr, ptr %6, align 8
  %51 = load ptr, ptr %3, align 8
  %52 = load i32, ptr %5, align 4
  %53 = add nsw i32 %52, 1
  %54 = call i64 @luaL_checkinteger(ptr noundef %51, i32 noundef %53)
  %55 = trunc i64 %54 to i32
  %56 = call i32 @lua_getstack(ptr noundef %50, i32 noundef %55, ptr noundef %4)
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %60, label %58

58:                                               ; preds = %49
  %59 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %59)
  store i32 1, ptr %2, align 4
  br label %169

60:                                               ; preds = %49
  br label %61

61:                                               ; preds = %60, %40
  %62 = load ptr, ptr %6, align 8
  %63 = load ptr, ptr %7, align 8
  %64 = call i32 @lua_getinfo(ptr noundef %62, ptr noundef %63, ptr noundef %4)
  %65 = icmp ne i32 %64, 0
  br i1 %65, label %71, label %66

66:                                               ; preds = %61
  %67 = load ptr, ptr %3, align 8
  %68 = load i32, ptr %5, align 4
  %69 = add nsw i32 %68, 2
  %70 = call i32 @luaL_argerror(ptr noundef %67, i32 noundef %69, ptr noundef @.str.33)
  store i32 %70, ptr %2, align 4
  br label %169

71:                                               ; preds = %61
  %72 = load ptr, ptr %3, align 8
  call void @lua_createtable(ptr noundef %72, i32 noundef 0, i32 noundef 0)
  %73 = load ptr, ptr %7, align 8
  %74 = call ptr @strchr(ptr noundef %73, i32 noundef 83) #3
  %75 = icmp ne ptr %74, null
  br i1 %75, label %76, label %96

76:                                               ; preds = %71
  %77 = load ptr, ptr %3, align 8
  %78 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 4
  %79 = load ptr, ptr %78, align 8
  %80 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 5
  %81 = load i64, ptr %80, align 8
  %82 = call ptr @lua_pushlstring(ptr noundef %77, ptr noundef %79, i64 noundef %81)
  %83 = load ptr, ptr %3, align 8
  call void @lua_setfield(ptr noundef %83, i32 noundef -2, ptr noundef @.str.34)
  %84 = load ptr, ptr %3, align 8
  %85 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 15
  %86 = getelementptr inbounds [60 x i8], ptr %85, i64 0, i64 0
  call void @settabss(ptr noundef %84, ptr noundef @.str.35, ptr noundef %86)
  %87 = load ptr, ptr %3, align 8
  %88 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 7
  %89 = load i32, ptr %88, align 4
  call void @settabsi(ptr noundef %87, ptr noundef @.str.36, i32 noundef %89)
  %90 = load ptr, ptr %3, align 8
  %91 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 8
  %92 = load i32, ptr %91, align 8
  call void @settabsi(ptr noundef %90, ptr noundef @.str.37, i32 noundef %92)
  %93 = load ptr, ptr %3, align 8
  %94 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 3
  %95 = load ptr, ptr %94, align 8
  call void @settabss(ptr noundef %93, ptr noundef @.str.38, ptr noundef %95)
  br label %96

96:                                               ; preds = %76, %71
  %97 = load ptr, ptr %7, align 8
  %98 = call ptr @strchr(ptr noundef %97, i32 noundef 108) #3
  %99 = icmp ne ptr %98, null
  br i1 %99, label %100, label %104

100:                                              ; preds = %96
  %101 = load ptr, ptr %3, align 8
  %102 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 6
  %103 = load i32, ptr %102, align 8
  call void @settabsi(ptr noundef %101, ptr noundef @.str.39, i32 noundef %103)
  br label %104

104:                                              ; preds = %100, %96
  %105 = load ptr, ptr %7, align 8
  %106 = call ptr @strchr(ptr noundef %105, i32 noundef 117) #3
  %107 = icmp ne ptr %106, null
  br i1 %107, label %108, label %121

108:                                              ; preds = %104
  %109 = load ptr, ptr %3, align 8
  %110 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 9
  %111 = load i8, ptr %110, align 4
  %112 = zext i8 %111 to i32
  call void @settabsi(ptr noundef %109, ptr noundef @.str.40, i32 noundef %112)
  %113 = load ptr, ptr %3, align 8
  %114 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 10
  %115 = load i8, ptr %114, align 1
  %116 = zext i8 %115 to i32
  call void @settabsi(ptr noundef %113, ptr noundef @.str.41, i32 noundef %116)
  %117 = load ptr, ptr %3, align 8
  %118 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 11
  %119 = load i8, ptr %118, align 2
  %120 = sext i8 %119 to i32
  call void @settabsb(ptr noundef %117, ptr noundef @.str.42, i32 noundef %120)
  br label %121

121:                                              ; preds = %108, %104
  %122 = load ptr, ptr %7, align 8
  %123 = call ptr @strchr(ptr noundef %122, i32 noundef 110) #3
  %124 = icmp ne ptr %123, null
  br i1 %124, label %125, label %132

125:                                              ; preds = %121
  %126 = load ptr, ptr %3, align 8
  %127 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 1
  %128 = load ptr, ptr %127, align 8
  call void @settabss(ptr noundef %126, ptr noundef @.str.43, ptr noundef %128)
  %129 = load ptr, ptr %3, align 8
  %130 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 2
  %131 = load ptr, ptr %130, align 8
  call void @settabss(ptr noundef %129, ptr noundef @.str.44, ptr noundef %131)
  br label %132

132:                                              ; preds = %125, %121
  %133 = load ptr, ptr %7, align 8
  %134 = call ptr @strchr(ptr noundef %133, i32 noundef 114) #3
  %135 = icmp ne ptr %134, null
  br i1 %135, label %136, label %145

136:                                              ; preds = %132
  %137 = load ptr, ptr %3, align 8
  %138 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 13
  %139 = load i16, ptr %138, align 8
  %140 = zext i16 %139 to i32
  call void @settabsi(ptr noundef %137, ptr noundef @.str.45, i32 noundef %140)
  %141 = load ptr, ptr %3, align 8
  %142 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 14
  %143 = load i16, ptr %142, align 2
  %144 = zext i16 %143 to i32
  call void @settabsi(ptr noundef %141, ptr noundef @.str.46, i32 noundef %144)
  br label %145

145:                                              ; preds = %136, %132
  %146 = load ptr, ptr %7, align 8
  %147 = call ptr @strchr(ptr noundef %146, i32 noundef 116) #3
  %148 = icmp ne ptr %147, null
  br i1 %148, label %149, label %154

149:                                              ; preds = %145
  %150 = load ptr, ptr %3, align 8
  %151 = getelementptr inbounds %struct.lua_Debug, ptr %4, i32 0, i32 12
  %152 = load i8, ptr %151, align 1
  %153 = sext i8 %152 to i32
  call void @settabsb(ptr noundef %150, ptr noundef @.str.47, i32 noundef %153)
  br label %154

154:                                              ; preds = %149, %145
  %155 = load ptr, ptr %7, align 8
  %156 = call ptr @strchr(ptr noundef %155, i32 noundef 76) #3
  %157 = icmp ne ptr %156, null
  br i1 %157, label %158, label %161

158:                                              ; preds = %154
  %159 = load ptr, ptr %3, align 8
  %160 = load ptr, ptr %6, align 8
  call void @treatstackoption(ptr noundef %159, ptr noundef %160, ptr noundef @.str.48)
  br label %161

161:                                              ; preds = %158, %154
  %162 = load ptr, ptr %7, align 8
  %163 = call ptr @strchr(ptr noundef %162, i32 noundef 102) #3
  %164 = icmp ne ptr %163, null
  br i1 %164, label %165, label %168

165:                                              ; preds = %161
  %166 = load ptr, ptr %3, align 8
  %167 = load ptr, ptr %6, align 8
  call void @treatstackoption(ptr noundef %166, ptr noundef %167, ptr noundef @.str.49)
  br label %168

168:                                              ; preds = %165, %161
  store i32 1, ptr %2, align 4
  br label %169

169:                                              ; preds = %168, %66, %58
  %170 = load i32, ptr %2, align 4
  ret i32 %170
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getlocal(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.lua_Debug, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @getthread(ptr noundef %10, ptr noundef %4)
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = load i32, ptr %4, align 4
  %14 = add nsw i32 %13, 2
  %15 = call i64 @luaL_checkinteger(ptr noundef %12, i32 noundef %14)
  %16 = trunc i64 %15 to i32
  store i32 %16, ptr %6, align 4
  %17 = load ptr, ptr %3, align 8
  %18 = load i32, ptr %4, align 4
  %19 = add nsw i32 %18, 1
  %20 = call i32 @lua_type(ptr noundef %17, i32 noundef %19)
  %21 = icmp eq i32 %20, 6
  br i1 %21, label %22, label %31

22:                                               ; preds = %1
  %23 = load ptr, ptr %3, align 8
  %24 = load i32, ptr %4, align 4
  %25 = add nsw i32 %24, 1
  call void @lua_pushvalue(ptr noundef %23, i32 noundef %25)
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = load i32, ptr %6, align 4
  %29 = call ptr @lua_getlocal(ptr noundef %27, ptr noundef null, i32 noundef %28)
  %30 = call ptr @lua_pushstring(ptr noundef %26, ptr noundef %29)
  store i32 1, ptr %2, align 4
  br label %69

31:                                               ; preds = %1
  %32 = load ptr, ptr %3, align 8
  %33 = load i32, ptr %4, align 4
  %34 = add nsw i32 %33, 1
  %35 = call i64 @luaL_checkinteger(ptr noundef %32, i32 noundef %34)
  %36 = trunc i64 %35 to i32
  store i32 %36, ptr %9, align 4
  %37 = load ptr, ptr %5, align 8
  %38 = load i32, ptr %9, align 4
  %39 = call i32 @lua_getstack(ptr noundef %37, i32 noundef %38, ptr noundef %7)
  %40 = icmp ne i32 %39, 0
  %41 = xor i1 %40, true
  %42 = zext i1 %41 to i32
  %43 = icmp ne i32 %42, 0
  %44 = zext i1 %43 to i32
  %45 = sext i32 %44 to i64
  %46 = icmp ne i64 %45, 0
  br i1 %46, label %47, label %52

47:                                               ; preds = %31
  %48 = load ptr, ptr %3, align 8
  %49 = load i32, ptr %4, align 4
  %50 = add nsw i32 %49, 1
  %51 = call i32 @luaL_argerror(ptr noundef %48, i32 noundef %50, ptr noundef @.str.50)
  store i32 %51, ptr %2, align 4
  br label %69

52:                                               ; preds = %31
  %53 = load ptr, ptr %3, align 8
  %54 = load ptr, ptr %5, align 8
  call void @checkstack(ptr noundef %53, ptr noundef %54, i32 noundef 1)
  %55 = load ptr, ptr %5, align 8
  %56 = load i32, ptr %6, align 4
  %57 = call ptr @lua_getlocal(ptr noundef %55, ptr noundef %7, i32 noundef %56)
  store ptr %57, ptr %8, align 8
  %58 = load ptr, ptr %8, align 8
  %59 = icmp ne ptr %58, null
  br i1 %59, label %60, label %67

60:                                               ; preds = %52
  %61 = load ptr, ptr %5, align 8
  %62 = load ptr, ptr %3, align 8
  call void @lua_xmove(ptr noundef %61, ptr noundef %62, i32 noundef 1)
  %63 = load ptr, ptr %3, align 8
  %64 = load ptr, ptr %8, align 8
  %65 = call ptr @lua_pushstring(ptr noundef %63, ptr noundef %64)
  %66 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %66, i32 noundef -2, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %69

67:                                               ; preds = %52
  %68 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %68)
  store i32 1, ptr %2, align 4
  br label %69

69:                                               ; preds = %67, %60, %47, %22
  %70 = load i32, ptr %2, align 4
  ret i32 %70
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getregistry(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %3, i32 noundef -1001000)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getmetatable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 1)
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_getmetatable(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %9, label %7

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %8)
  br label %9

9:                                                ; preds = %7, %1
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_getupvalue(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @auxupvalue(ptr noundef %3, i32 noundef 1)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_upvaluejoin(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @checkupval(ptr noundef %5, i32 noundef 1, i32 noundef 2, ptr noundef %3)
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @checkupval(ptr noundef %7, i32 noundef 3, i32 noundef 4, ptr noundef %4)
  %9 = load ptr, ptr %2, align 8
  %10 = call i32 @lua_iscfunction(ptr noundef %9, i32 noundef 1)
  %11 = icmp ne i32 %10, 0
  %12 = xor i1 %11, true
  %13 = zext i1 %12 to i32
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = sext i32 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %22, label %18

18:                                               ; preds = %1
  %19 = load ptr, ptr %2, align 8
  %20 = call i32 @luaL_argerror(ptr noundef %19, i32 noundef 1, ptr noundef @.str.51)
  %21 = icmp ne i32 %20, 0
  br label %22

22:                                               ; preds = %18, %1
  %23 = phi i1 [ true, %1 ], [ %21, %18 ]
  %24 = zext i1 %23 to i32
  %25 = load ptr, ptr %2, align 8
  %26 = call i32 @lua_iscfunction(ptr noundef %25, i32 noundef 3)
  %27 = icmp ne i32 %26, 0
  %28 = xor i1 %27, true
  %29 = zext i1 %28 to i32
  %30 = icmp ne i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %38, label %34

34:                                               ; preds = %22
  %35 = load ptr, ptr %2, align 8
  %36 = call i32 @luaL_argerror(ptr noundef %35, i32 noundef 3, ptr noundef @.str.51)
  %37 = icmp ne i32 %36, 0
  br label %38

38:                                               ; preds = %34, %22
  %39 = phi i1 [ true, %22 ], [ %37, %34 ]
  %40 = zext i1 %39 to i32
  %41 = load ptr, ptr %2, align 8
  %42 = load i32, ptr %3, align 4
  %43 = load i32, ptr %4, align 4
  call void @lua_upvaluejoin(ptr noundef %41, i32 noundef 1, i32 noundef %42, i32 noundef 3, i32 noundef %43)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_upvalueid(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @checkupval(ptr noundef %4, i32 noundef 1, i32 noundef 2, ptr noundef null)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = icmp ne ptr %6, null
  br i1 %7, label %8, label %11

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  %10 = load ptr, ptr %3, align 8
  call void @lua_pushlightuserdata(ptr noundef %9, ptr noundef %10)
  br label %13

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %12)
  br label %13

13:                                               ; preds = %11, %8
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_setuservalue(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i64 @luaL_optinteger(ptr noundef %4, i32 noundef 3, i64 noundef 1)
  %6 = trunc i64 %5 to i32
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %7, i32 noundef 1, i32 noundef 7)
  %8 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %8, i32 noundef 2)
  %9 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %9, i32 noundef 2)
  %10 = load ptr, ptr %2, align 8
  %11 = load i32, ptr %3, align 4
  %12 = call i32 @lua_setiuservalue(ptr noundef %10, i32 noundef 1, i32 noundef %11)
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %16, label %14

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %15)
  br label %16

16:                                               ; preds = %14, %1
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_sethook(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @getthread(ptr noundef %9, ptr noundef %3)
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = load i32, ptr %3, align 4
  %13 = add nsw i32 %12, 1
  %14 = call i32 @lua_type(ptr noundef %11, i32 noundef %13)
  %15 = icmp sle i32 %14, 0
  br i1 %15, label %16, label %20

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = load i32, ptr %3, align 4
  %19 = add nsw i32 %18, 1
  call void @lua_settop(ptr noundef %17, i32 noundef %19)
  store ptr null, ptr %6, align 8
  store i32 0, ptr %4, align 4
  store i32 0, ptr %5, align 4
  br label %36

20:                                               ; preds = %1
  %21 = load ptr, ptr %2, align 8
  %22 = load i32, ptr %3, align 4
  %23 = add nsw i32 %22, 2
  %24 = call ptr @luaL_checklstring(ptr noundef %21, i32 noundef %23, ptr noundef null)
  store ptr %24, ptr %8, align 8
  %25 = load ptr, ptr %2, align 8
  %26 = load i32, ptr %3, align 4
  %27 = add nsw i32 %26, 1
  call void @luaL_checktype(ptr noundef %25, i32 noundef %27, i32 noundef 6)
  %28 = load ptr, ptr %2, align 8
  %29 = load i32, ptr %3, align 4
  %30 = add nsw i32 %29, 3
  %31 = call i64 @luaL_optinteger(ptr noundef %28, i32 noundef %30, i64 noundef 0)
  %32 = trunc i64 %31 to i32
  store i32 %32, ptr %5, align 4
  store ptr @hookf, ptr %6, align 8
  %33 = load ptr, ptr %8, align 8
  %34 = load i32, ptr %5, align 4
  %35 = call i32 @makemask(ptr noundef %33, i32 noundef %34)
  store i32 %35, ptr %4, align 4
  br label %36

36:                                               ; preds = %20, %16
  %37 = load ptr, ptr %2, align 8
  %38 = call i32 @luaL_getsubtable(ptr noundef %37, i32 noundef -1001000, ptr noundef @.str.23)
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %47, label %40

40:                                               ; preds = %36
  %41 = load ptr, ptr %2, align 8
  %42 = call ptr @lua_pushstring(ptr noundef %41, ptr noundef @.str.53)
  %43 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %43, i32 noundef -2, ptr noundef @.str.54)
  %44 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %44, i32 noundef -1)
  %45 = load ptr, ptr %2, align 8
  %46 = call i32 @lua_setmetatable(ptr noundef %45, i32 noundef -2)
  br label %47

47:                                               ; preds = %40, %36
  %48 = load ptr, ptr %2, align 8
  %49 = load ptr, ptr %7, align 8
  call void @checkstack(ptr noundef %48, ptr noundef %49, i32 noundef 1)
  %50 = load ptr, ptr %7, align 8
  %51 = call i32 @lua_pushthread(ptr noundef %50)
  %52 = load ptr, ptr %7, align 8
  %53 = load ptr, ptr %2, align 8
  call void @lua_xmove(ptr noundef %52, ptr noundef %53, i32 noundef 1)
  %54 = load ptr, ptr %2, align 8
  %55 = load i32, ptr %3, align 4
  %56 = add nsw i32 %55, 1
  call void @lua_pushvalue(ptr noundef %54, i32 noundef %56)
  %57 = load ptr, ptr %2, align 8
  call void @lua_rawset(ptr noundef %57, i32 noundef -3)
  %58 = load ptr, ptr %7, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = load i32, ptr %4, align 4
  %61 = load i32, ptr %5, align 4
  call void @lua_sethook(ptr noundef %58, ptr noundef %59, i32 noundef %60, i32 noundef %61)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_setlocal(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.lua_Debug, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @getthread(ptr noundef %10, ptr noundef %4)
  store ptr %11, ptr %6, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = load i32, ptr %4, align 4
  %14 = add nsw i32 %13, 1
  %15 = call i64 @luaL_checkinteger(ptr noundef %12, i32 noundef %14)
  %16 = trunc i64 %15 to i32
  store i32 %16, ptr %8, align 4
  %17 = load ptr, ptr %3, align 8
  %18 = load i32, ptr %4, align 4
  %19 = add nsw i32 %18, 2
  %20 = call i64 @luaL_checkinteger(ptr noundef %17, i32 noundef %19)
  %21 = trunc i64 %20 to i32
  store i32 %21, ptr %9, align 4
  %22 = load ptr, ptr %6, align 8
  %23 = load i32, ptr %8, align 4
  %24 = call i32 @lua_getstack(ptr noundef %22, i32 noundef %23, ptr noundef %7)
  %25 = icmp ne i32 %24, 0
  %26 = xor i1 %25, true
  %27 = zext i1 %26 to i32
  %28 = icmp ne i32 %27, 0
  %29 = zext i1 %28 to i32
  %30 = sext i32 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %32, label %37

32:                                               ; preds = %1
  %33 = load ptr, ptr %3, align 8
  %34 = load i32, ptr %4, align 4
  %35 = add nsw i32 %34, 1
  %36 = call i32 @luaL_argerror(ptr noundef %33, i32 noundef %35, ptr noundef @.str.50)
  store i32 %36, ptr %2, align 4
  br label %59

37:                                               ; preds = %1
  %38 = load ptr, ptr %3, align 8
  %39 = load i32, ptr %4, align 4
  %40 = add nsw i32 %39, 3
  call void @luaL_checkany(ptr noundef %38, i32 noundef %40)
  %41 = load ptr, ptr %3, align 8
  %42 = load i32, ptr %4, align 4
  %43 = add nsw i32 %42, 3
  call void @lua_settop(ptr noundef %41, i32 noundef %43)
  %44 = load ptr, ptr %3, align 8
  %45 = load ptr, ptr %6, align 8
  call void @checkstack(ptr noundef %44, ptr noundef %45, i32 noundef 1)
  %46 = load ptr, ptr %3, align 8
  %47 = load ptr, ptr %6, align 8
  call void @lua_xmove(ptr noundef %46, ptr noundef %47, i32 noundef 1)
  %48 = load ptr, ptr %6, align 8
  %49 = load i32, ptr %9, align 4
  %50 = call ptr @lua_setlocal(ptr noundef %48, ptr noundef %7, i32 noundef %49)
  store ptr %50, ptr %5, align 8
  %51 = load ptr, ptr %5, align 8
  %52 = icmp eq ptr %51, null
  br i1 %52, label %53, label %55

53:                                               ; preds = %37
  %54 = load ptr, ptr %6, align 8
  call void @lua_settop(ptr noundef %54, i32 noundef -2)
  br label %55

55:                                               ; preds = %53, %37
  %56 = load ptr, ptr %3, align 8
  %57 = load ptr, ptr %5, align 8
  %58 = call ptr @lua_pushstring(ptr noundef %56, ptr noundef %57)
  store i32 1, ptr %2, align 4
  br label %59

59:                                               ; preds = %55, %32
  %60 = load i32, ptr %2, align 4
  ret i32 %60
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_setmetatable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef 2)
  store i32 %5, ptr %3, align 4
  %6 = load i32, ptr %3, align 4
  %7 = icmp eq i32 %6, 0
  br i1 %7, label %11, label %8

8:                                                ; preds = %1
  %9 = load i32, ptr %3, align 4
  %10 = icmp eq i32 %9, 5
  br label %11

11:                                               ; preds = %8, %1
  %12 = phi i1 [ true, %1 ], [ %10, %8 ]
  %13 = zext i1 %12 to i32
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = sext i32 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %22, label %18

18:                                               ; preds = %11
  %19 = load ptr, ptr %2, align 8
  %20 = call i32 @luaL_typeerror(ptr noundef %19, i32 noundef 2, ptr noundef @.str.55)
  %21 = icmp ne i32 %20, 0
  br label %22

22:                                               ; preds = %18, %11
  %23 = phi i1 [ true, %11 ], [ %21, %18 ]
  %24 = zext i1 %23 to i32
  %25 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %25, i32 noundef 2)
  %26 = load ptr, ptr %2, align 8
  %27 = call i32 @lua_setmetatable(ptr noundef %26, i32 noundef 1)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_setupvalue(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 3)
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @auxupvalue(ptr noundef %4, i32 noundef 0)
  ret i32 %5
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_traceback(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @getthread(ptr noundef %7, ptr noundef %3)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = load i32, ptr %3, align 4
  %11 = add nsw i32 %10, 1
  %12 = call ptr @lua_tolstring(ptr noundef %9, i32 noundef %11, ptr noundef null)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = icmp eq ptr %13, null
  br i1 %14, label %15, label %25

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = load i32, ptr %3, align 4
  %18 = add nsw i32 %17, 1
  %19 = call i32 @lua_type(ptr noundef %16, i32 noundef %18)
  %20 = icmp sle i32 %19, 0
  br i1 %20, label %25, label %21

21:                                               ; preds = %15
  %22 = load ptr, ptr %2, align 8
  %23 = load i32, ptr %3, align 4
  %24 = add nsw i32 %23, 1
  call void @lua_pushvalue(ptr noundef %22, i32 noundef %24)
  br label %41

25:                                               ; preds = %15, %1
  %26 = load ptr, ptr %2, align 8
  %27 = load i32, ptr %3, align 4
  %28 = add nsw i32 %27, 2
  %29 = load ptr, ptr %2, align 8
  %30 = load ptr, ptr %4, align 8
  %31 = icmp eq ptr %29, %30
  %32 = zext i1 %31 to i64
  %33 = select i1 %31, i32 1, i32 0
  %34 = sext i32 %33 to i64
  %35 = call i64 @luaL_optinteger(ptr noundef %26, i32 noundef %28, i64 noundef %34)
  %36 = trunc i64 %35 to i32
  store i32 %36, ptr %6, align 4
  %37 = load ptr, ptr %2, align 8
  %38 = load ptr, ptr %4, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = load i32, ptr %6, align 4
  call void @luaL_traceback(ptr noundef %37, ptr noundef %38, ptr noundef %39, i32 noundef %40)
  br label %41

41:                                               ; preds = %25, %21
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @db_setcstacklimit(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i64 @luaL_checkinteger(ptr noundef %5, i32 noundef 1)
  %7 = trunc i64 %6 to i32
  store i32 %7, ptr %3, align 4
  %8 = load ptr, ptr %2, align 8
  %9 = load i32, ptr %3, align 4
  %10 = call i32 @lua_setcstacklimit(ptr noundef %8, i32 noundef %9)
  store i32 %10, ptr %4, align 4
  %11 = load ptr, ptr %2, align 8
  %12 = load i32, ptr %4, align 4
  %13 = sext i32 %12 to i64
  call void @lua_pushinteger(ptr noundef %11, i64 noundef %13)
  ret i32 1
}

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #1

declare i32 @fflush(ptr noundef) #1

declare ptr @fgets(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #2

declare i32 @luaL_loadbufferx(ptr noundef, ptr noundef, i64 noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #2

declare i32 @lua_pcallk(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare ptr @luaL_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare void @lua_pushnil(ptr noundef) #1

declare i32 @lua_getiuservalue(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getthread(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = call i32 @lua_type(ptr noundef %6, i32 noundef 1)
  %8 = icmp eq i32 %7, 8
  br i1 %8, label %9, label %13

9:                                                ; preds = %2
  %10 = load ptr, ptr %5, align 8
  store i32 1, ptr %10, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @lua_tothread(ptr noundef %11, i32 noundef 1)
  store ptr %12, ptr %3, align 8
  br label %16

13:                                               ; preds = %2
  %14 = load ptr, ptr %5, align 8
  store i32 0, ptr %14, align 4
  %15 = load ptr, ptr %4, align 8
  store ptr %15, ptr %3, align 8
  br label %16

16:                                               ; preds = %13, %9
  %17 = load ptr, ptr %3, align 8
  ret ptr %17
}

declare i32 @lua_gethookmask(ptr noundef) #1

declare ptr @lua_gethook(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @hookf(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call i32 @lua_getfield(ptr noundef %5, i32 noundef -1001000, ptr noundef @.str.23)
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @lua_pushthread(ptr noundef %7)
  %9 = load ptr, ptr %3, align 8
  %10 = call i32 @lua_rawget(ptr noundef %9, i32 noundef -2)
  %11 = icmp eq i32 %10, 6
  br i1 %11, label %12, label %35

12:                                               ; preds = %2
  %13 = load ptr, ptr %3, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.lua_Debug, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds [5 x ptr], ptr @hookf.hooknames, i64 0, i64 %17
  %19 = load ptr, ptr %18, align 8
  %20 = call ptr @lua_pushstring(ptr noundef %13, ptr noundef %19)
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.lua_Debug, ptr %21, i32 0, i32 6
  %23 = load i32, ptr %22, align 8
  %24 = icmp sge i32 %23, 0
  br i1 %24, label %25, label %31

25:                                               ; preds = %12
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.lua_Debug, ptr %27, i32 0, i32 6
  %29 = load i32, ptr %28, align 8
  %30 = sext i32 %29 to i64
  call void @lua_pushinteger(ptr noundef %26, i64 noundef %30)
  br label %33

31:                                               ; preds = %12
  %32 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %32)
  br label %33

33:                                               ; preds = %31, %25
  %34 = load ptr, ptr %3, align 8
  call void @lua_callk(ptr noundef %34, i32 noundef 2, i32 noundef 0, i64 noundef 0, ptr noundef null)
  br label %35

35:                                               ; preds = %33, %2
  ret void
}

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare i32 @lua_getfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkstack(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = icmp ne ptr %7, %8
  br i1 %9, label %10, label %16

10:                                               ; preds = %3
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %6, align 4
  %13 = call i32 @lua_checkstack(ptr noundef %11, i32 noundef %12)
  %14 = icmp ne i32 %13, 0
  %15 = xor i1 %14, true
  br label %16

16:                                               ; preds = %10, %3
  %17 = phi i1 [ false, %3 ], [ %15, %10 ]
  %18 = zext i1 %17 to i32
  %19 = icmp ne i32 %18, 0
  %20 = zext i1 %19 to i32
  %21 = sext i32 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %23, label %26

23:                                               ; preds = %16
  %24 = load ptr, ptr %4, align 8
  %25 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %24, ptr noundef @.str.29)
  br label %26

26:                                               ; preds = %23, %16
  ret void
}

declare i32 @lua_pushthread(ptr noundef) #1

declare void @lua_xmove(ptr noundef, ptr noundef, i32 noundef) #1

declare i32 @lua_rawget(ptr noundef, i32 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @unmakemask(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %5, align 4
  %6 = load i32, ptr %3, align 4
  %7 = and i32 %6, 1
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %15

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8
  %11 = load i32, ptr %5, align 4
  %12 = add nsw i32 %11, 1
  store i32 %12, ptr %5, align 4
  %13 = sext i32 %11 to i64
  %14 = getelementptr inbounds i8, ptr %10, i64 %13
  store i8 99, ptr %14, align 1
  br label %15

15:                                               ; preds = %9, %2
  %16 = load i32, ptr %3, align 4
  %17 = and i32 %16, 2
  %18 = icmp ne i32 %17, 0
  br i1 %18, label %19, label %25

19:                                               ; preds = %15
  %20 = load ptr, ptr %4, align 8
  %21 = load i32, ptr %5, align 4
  %22 = add nsw i32 %21, 1
  store i32 %22, ptr %5, align 4
  %23 = sext i32 %21 to i64
  %24 = getelementptr inbounds i8, ptr %20, i64 %23
  store i8 114, ptr %24, align 1
  br label %25

25:                                               ; preds = %19, %15
  %26 = load i32, ptr %3, align 4
  %27 = and i32 %26, 4
  %28 = icmp ne i32 %27, 0
  br i1 %28, label %29, label %35

29:                                               ; preds = %25
  %30 = load ptr, ptr %4, align 8
  %31 = load i32, ptr %5, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %5, align 4
  %33 = sext i32 %31 to i64
  %34 = getelementptr inbounds i8, ptr %30, i64 %33
  store i8 108, ptr %34, align 1
  br label %35

35:                                               ; preds = %29, %25
  %36 = load ptr, ptr %4, align 8
  %37 = load i32, ptr %5, align 4
  %38 = sext i32 %37 to i64
  %39 = getelementptr inbounds i8, ptr %36, i64 %38
  store i8 0, ptr %39, align 1
  %40 = load ptr, ptr %4, align 8
  ret ptr %40
}

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare i32 @lua_gethookcount(ptr noundef) #1

declare ptr @lua_tothread(ptr noundef, i32 noundef) #1

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_checkstack(ptr noundef, i32 noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare i32 @lua_getstack(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare i32 @lua_getinfo(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #2

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @settabss(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = call ptr @lua_pushstring(ptr noundef %7, ptr noundef %8)
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %10, i32 noundef -2, ptr noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @settabsi(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %6, align 4
  %9 = sext i32 %8 to i64
  call void @lua_pushinteger(ptr noundef %7, i64 noundef %9)
  %10 = load ptr, ptr %4, align 8
  %11 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %10, i32 noundef -2, ptr noundef %11)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @settabsb(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %6, align 4
  call void @lua_pushboolean(ptr noundef %7, i32 noundef %8)
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %9, i32 noundef -2, ptr noundef %10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @treatstackoption(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = icmp eq ptr %7, %8
  br i1 %9, label %10, label %12

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  call void @lua_rotate(ptr noundef %11, i32 noundef -2, i32 noundef 1)
  br label %15

12:                                               ; preds = %3
  %13 = load ptr, ptr %5, align 8
  %14 = load ptr, ptr %4, align 8
  call void @lua_xmove(ptr noundef %13, ptr noundef %14, i32 noundef 1)
  br label %15

15:                                               ; preds = %12, %10
  %16 = load ptr, ptr %4, align 8
  %17 = load ptr, ptr %6, align 8
  call void @lua_setfield(ptr noundef %16, i32 noundef -2, ptr noundef %17)
  ret void
}

declare ptr @lua_getlocal(ptr noundef, ptr noundef, i32 noundef) #1

declare void @luaL_checkany(ptr noundef, i32 noundef) #1

declare i32 @lua_getmetatable(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @auxupvalue(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = call i64 @luaL_checkinteger(ptr noundef %8, i32 noundef 2)
  %10 = trunc i64 %9 to i32
  store i32 %10, ptr %7, align 4
  %11 = load ptr, ptr %4, align 8
  call void @luaL_checktype(ptr noundef %11, i32 noundef 1, i32 noundef 6)
  %12 = load i32, ptr %5, align 4
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %18

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = load i32, ptr %7, align 4
  %17 = call ptr @lua_getupvalue(ptr noundef %15, i32 noundef 1, i32 noundef %16)
  br label %22

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = load i32, ptr %7, align 4
  %21 = call ptr @lua_setupvalue(ptr noundef %19, i32 noundef 1, i32 noundef %20)
  br label %22

22:                                               ; preds = %18, %14
  %23 = phi ptr [ %17, %14 ], [ %21, %18 ]
  store ptr %23, ptr %6, align 8
  %24 = load ptr, ptr %6, align 8
  %25 = icmp eq ptr %24, null
  br i1 %25, label %26, label %27

26:                                               ; preds = %22
  store i32 0, ptr %3, align 4
  br label %37

27:                                               ; preds = %22
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = call ptr @lua_pushstring(ptr noundef %28, ptr noundef %29)
  %31 = load ptr, ptr %4, align 8
  %32 = load i32, ptr %5, align 4
  %33 = add nsw i32 %32, 1
  %34 = sub nsw i32 0, %33
  call void @lua_rotate(ptr noundef %31, i32 noundef %34, i32 noundef 1)
  %35 = load i32, ptr %5, align 4
  %36 = add nsw i32 %35, 1
  store i32 %36, ptr %3, align 4
  br label %37

37:                                               ; preds = %27, %26
  %38 = load i32, ptr %3, align 4
  ret i32 %38
}

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_getupvalue(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_setupvalue(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @checkupval(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  store ptr %3, ptr %8, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %7, align 4
  %13 = call i64 @luaL_checkinteger(ptr noundef %11, i32 noundef %12)
  %14 = trunc i64 %13 to i32
  store i32 %14, ptr %10, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %6, align 4
  call void @luaL_checktype(ptr noundef %15, i32 noundef %16, i32 noundef 6)
  %17 = load ptr, ptr %5, align 8
  %18 = load i32, ptr %6, align 4
  %19 = load i32, ptr %10, align 4
  %20 = call ptr @lua_upvalueid(ptr noundef %17, i32 noundef %18, i32 noundef %19)
  store ptr %20, ptr %9, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = icmp ne ptr %21, null
  br i1 %22, label %23, label %41

23:                                               ; preds = %4
  %24 = load ptr, ptr %9, align 8
  %25 = icmp ne ptr %24, null
  %26 = zext i1 %25 to i32
  %27 = icmp ne i32 %26, 0
  %28 = zext i1 %27 to i32
  %29 = sext i32 %28 to i64
  %30 = icmp ne i64 %29, 0
  br i1 %30, label %36, label %31

31:                                               ; preds = %23
  %32 = load ptr, ptr %5, align 8
  %33 = load i32, ptr %7, align 4
  %34 = call i32 @luaL_argerror(ptr noundef %32, i32 noundef %33, ptr noundef @.str.52)
  %35 = icmp ne i32 %34, 0
  br label %36

36:                                               ; preds = %31, %23
  %37 = phi i1 [ true, %23 ], [ %35, %31 ]
  %38 = zext i1 %37 to i32
  %39 = load i32, ptr %10, align 4
  %40 = load ptr, ptr %8, align 8
  store i32 %39, ptr %40, align 4
  br label %41

41:                                               ; preds = %36, %4
  %42 = load ptr, ptr %9, align 8
  ret ptr %42
}

declare i32 @lua_iscfunction(ptr noundef, i32 noundef) #1

declare void @lua_upvaluejoin(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_upvalueid(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushlightuserdata(ptr noundef, ptr noundef) #1

declare i32 @lua_setiuservalue(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @makemask(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  store i32 0, ptr %5, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @strchr(ptr noundef %6, i32 noundef 99) #3
  %8 = icmp ne ptr %7, null
  br i1 %8, label %9, label %12

9:                                                ; preds = %2
  %10 = load i32, ptr %5, align 4
  %11 = or i32 %10, 1
  store i32 %11, ptr %5, align 4
  br label %12

12:                                               ; preds = %9, %2
  %13 = load ptr, ptr %3, align 8
  %14 = call ptr @strchr(ptr noundef %13, i32 noundef 114) #3
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %19

16:                                               ; preds = %12
  %17 = load i32, ptr %5, align 4
  %18 = or i32 %17, 2
  store i32 %18, ptr %5, align 4
  br label %19

19:                                               ; preds = %16, %12
  %20 = load ptr, ptr %3, align 8
  %21 = call ptr @strchr(ptr noundef %20, i32 noundef 108) #3
  %22 = icmp ne ptr %21, null
  br i1 %22, label %23, label %26

23:                                               ; preds = %19
  %24 = load i32, ptr %5, align 4
  %25 = or i32 %24, 4
  store i32 %25, ptr %5, align 4
  br label %26

26:                                               ; preds = %23, %19
  %27 = load i32, ptr %4, align 4
  %28 = icmp sgt i32 %27, 0
  br i1 %28, label %29, label %32

29:                                               ; preds = %26
  %30 = load i32, ptr %5, align 4
  %31 = or i32 %30, 8
  store i32 %31, ptr %5, align 4
  br label %32

32:                                               ; preds = %29, %26
  %33 = load i32, ptr %5, align 4
  ret i32 %33
}

declare i32 @luaL_getsubtable(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_setmetatable(ptr noundef, i32 noundef) #1

declare void @lua_rawset(ptr noundef, i32 noundef) #1

declare void @lua_sethook(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_setlocal(ptr noundef, ptr noundef, i32 noundef) #1

declare i32 @luaL_typeerror(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_traceback(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

declare i32 @lua_setcstacklimit(ptr noundef, i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(read) }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
