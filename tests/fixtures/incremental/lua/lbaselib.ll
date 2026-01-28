; ModuleID = 'lbaselib.c'
source_filename = "lbaselib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }

@base_funcs = internal constant [26 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.3, ptr @luaB_assert }, %struct.luaL_Reg { ptr @.str.4, ptr @luaB_collectgarbage }, %struct.luaL_Reg { ptr @.str.5, ptr @luaB_dofile }, %struct.luaL_Reg { ptr @.str.6, ptr @luaB_error }, %struct.luaL_Reg { ptr @.str.7, ptr @luaB_getmetatable }, %struct.luaL_Reg { ptr @.str.8, ptr @luaB_ipairs }, %struct.luaL_Reg { ptr @.str.9, ptr @luaB_loadfile }, %struct.luaL_Reg { ptr @.str.10, ptr @luaB_load }, %struct.luaL_Reg { ptr @.str.11, ptr @luaB_next }, %struct.luaL_Reg { ptr @.str.12, ptr @luaB_pairs }, %struct.luaL_Reg { ptr @.str.13, ptr @luaB_pcall }, %struct.luaL_Reg { ptr @.str.14, ptr @luaB_print }, %struct.luaL_Reg { ptr @.str.15, ptr @luaB_warn }, %struct.luaL_Reg { ptr @.str.16, ptr @luaB_rawequal }, %struct.luaL_Reg { ptr @.str.17, ptr @luaB_rawlen }, %struct.luaL_Reg { ptr @.str.18, ptr @luaB_rawget }, %struct.luaL_Reg { ptr @.str.19, ptr @luaB_rawset }, %struct.luaL_Reg { ptr @.str.20, ptr @luaB_select }, %struct.luaL_Reg { ptr @.str.21, ptr @luaB_setmetatable }, %struct.luaL_Reg { ptr @.str.22, ptr @luaB_tonumber }, %struct.luaL_Reg { ptr @.str.23, ptr @luaB_tostring }, %struct.luaL_Reg { ptr @.str.24, ptr @luaB_type }, %struct.luaL_Reg { ptr @.str.25, ptr @luaB_xpcall }, %struct.luaL_Reg { ptr @.str, ptr null }, %struct.luaL_Reg { ptr @.str.2, ptr null }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [3 x i8] c"_G\00", align 1
@.str.1 = private unnamed_addr constant [8 x i8] c"Lua 5.4\00", align 1
@.str.2 = private unnamed_addr constant [9 x i8] c"_VERSION\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"assert\00", align 1
@.str.4 = private unnamed_addr constant [15 x i8] c"collectgarbage\00", align 1
@.str.5 = private unnamed_addr constant [7 x i8] c"dofile\00", align 1
@.str.6 = private unnamed_addr constant [6 x i8] c"error\00", align 1
@.str.7 = private unnamed_addr constant [13 x i8] c"getmetatable\00", align 1
@.str.8 = private unnamed_addr constant [7 x i8] c"ipairs\00", align 1
@.str.9 = private unnamed_addr constant [9 x i8] c"loadfile\00", align 1
@.str.10 = private unnamed_addr constant [5 x i8] c"load\00", align 1
@.str.11 = private unnamed_addr constant [5 x i8] c"next\00", align 1
@.str.12 = private unnamed_addr constant [6 x i8] c"pairs\00", align 1
@.str.13 = private unnamed_addr constant [6 x i8] c"pcall\00", align 1
@.str.14 = private unnamed_addr constant [6 x i8] c"print\00", align 1
@.str.15 = private unnamed_addr constant [5 x i8] c"warn\00", align 1
@.str.16 = private unnamed_addr constant [9 x i8] c"rawequal\00", align 1
@.str.17 = private unnamed_addr constant [7 x i8] c"rawlen\00", align 1
@.str.18 = private unnamed_addr constant [7 x i8] c"rawget\00", align 1
@.str.19 = private unnamed_addr constant [7 x i8] c"rawset\00", align 1
@.str.20 = private unnamed_addr constant [7 x i8] c"select\00", align 1
@.str.21 = private unnamed_addr constant [13 x i8] c"setmetatable\00", align 1
@.str.22 = private unnamed_addr constant [9 x i8] c"tonumber\00", align 1
@.str.23 = private unnamed_addr constant [9 x i8] c"tostring\00", align 1
@.str.24 = private unnamed_addr constant [5 x i8] c"type\00", align 1
@.str.25 = private unnamed_addr constant [7 x i8] c"xpcall\00", align 1
@.str.26 = private unnamed_addr constant [18 x i8] c"assertion failed!\00", align 1
@luaB_collectgarbage.opts = internal constant [11 x ptr] [ptr @.str.27, ptr @.str.28, ptr @.str.29, ptr @.str.30, ptr @.str.31, ptr @.str.32, ptr @.str.33, ptr @.str.34, ptr @.str.35, ptr @.str.36, ptr null], align 16
@.str.27 = private unnamed_addr constant [5 x i8] c"stop\00", align 1
@.str.28 = private unnamed_addr constant [8 x i8] c"restart\00", align 1
@.str.29 = private unnamed_addr constant [8 x i8] c"collect\00", align 1
@.str.30 = private unnamed_addr constant [6 x i8] c"count\00", align 1
@.str.31 = private unnamed_addr constant [5 x i8] c"step\00", align 1
@.str.32 = private unnamed_addr constant [9 x i8] c"setpause\00", align 1
@.str.33 = private unnamed_addr constant [11 x i8] c"setstepmul\00", align 1
@.str.34 = private unnamed_addr constant [10 x i8] c"isrunning\00", align 1
@.str.35 = private unnamed_addr constant [13 x i8] c"generational\00", align 1
@.str.36 = private unnamed_addr constant [12 x i8] c"incremental\00", align 1
@luaB_collectgarbage.optsnum = internal constant [10 x i32] [i32 0, i32 1, i32 2, i32 3, i32 5, i32 6, i32 7, i32 9, i32 10, i32 11], align 16
@.str.37 = private unnamed_addr constant [12 x i8] c"__metatable\00", align 1
@.str.38 = private unnamed_addr constant [3 x i8] c"bt\00", align 1
@.str.39 = private unnamed_addr constant [8 x i8] c"=(load)\00", align 1
@.str.40 = private unnamed_addr constant [26 x i8] c"too many nested functions\00", align 1
@.str.41 = private unnamed_addr constant [37 x i8] c"reader function must return a string\00", align 1
@.str.42 = private unnamed_addr constant [8 x i8] c"__pairs\00", align 1
@.str.43 = private unnamed_addr constant [2 x i8] c"\09\00", align 1
@stdout = external global ptr, align 8
@.str.44 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@.str.45 = private unnamed_addr constant [16 x i8] c"table or string\00", align 1
@.str.46 = private unnamed_addr constant [19 x i8] c"index out of range\00", align 1
@.str.47 = private unnamed_addr constant [13 x i8] c"nil or table\00", align 1
@.str.48 = private unnamed_addr constant [36 x i8] c"cannot change a protected metatable\00", align 1
@.str.49 = private unnamed_addr constant [18 x i8] c"base out of range\00", align 1
@.str.50 = private unnamed_addr constant [7 x i8] c" \0C\0A\0D\09\0B\00", align 1
@.str.51 = private unnamed_addr constant [15 x i8] c"value expected\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_base(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @lua_rawgeti(ptr noundef %3, i32 noundef -1001000, i64 noundef 2)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @base_funcs, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %6, i32 noundef -1)
  %7 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %7, i32 noundef -2, ptr noundef @.str)
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @lua_pushstring(ptr noundef %8, ptr noundef @.str.1)
  %10 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %10, i32 noundef -2, ptr noundef @.str.2)
  ret i32 1
}

declare i32 @lua_rawgeti(ptr noundef, i32 noundef, i64 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_assert(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = call i32 @lua_toboolean(ptr noundef %4, i32 noundef 1)
  %6 = icmp ne i32 %5, 0
  %7 = zext i1 %6 to i32
  %8 = sext i32 %7 to i64
  %9 = icmp ne i64 %8, 0
  br i1 %9, label %10, label %13

10:                                               ; preds = %1
  %11 = load ptr, ptr %3, align 8
  %12 = call i32 @lua_gettop(ptr noundef %11)
  store i32 %12, ptr %2, align 4
  br label %22

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  call void @luaL_checkany(ptr noundef %14, i32 noundef 1)
  %15 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %15, i32 noundef 1, i32 noundef -1)
  %16 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %16, i32 noundef -2)
  %17 = load ptr, ptr %3, align 8
  %18 = call ptr @lua_pushstring(ptr noundef %17, ptr noundef @.str.26)
  %19 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %19, i32 noundef 1)
  %20 = load ptr, ptr %3, align 8
  %21 = call i32 @luaB_error(ptr noundef %20)
  store i32 %21, ptr %2, align 4
  br label %22

22:                                               ; preds = %13, %10
  %23 = load i32, ptr %2, align 4
  ret i32 %23
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_collectgarbage(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca i32, align 4
  %16 = alloca i32, align 4
  %17 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = call i32 @luaL_checkoption(ptr noundef %18, i32 noundef 1, ptr noundef @.str.29, ptr noundef @luaB_collectgarbage.opts)
  %20 = sext i32 %19 to i64
  %21 = getelementptr inbounds [10 x i32], ptr @luaB_collectgarbage.optsnum, i64 0, i64 %20
  %22 = load i32, ptr %21, align 4
  store i32 %22, ptr %4, align 4
  %23 = load i32, ptr %4, align 4
  switch i32 %23, label %112 [
    i32 3, label %24
    i32 5, label %41
    i32 6, label %55
    i32 7, label %55
    i32 9, label %70
    i32 10, label %80
    i32 11, label %94
  ]

24:                                               ; preds = %1
  %25 = load ptr, ptr %3, align 8
  %26 = load i32, ptr %4, align 4
  %27 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %25, i32 noundef %26)
  store i32 %27, ptr %5, align 4
  %28 = load ptr, ptr %3, align 8
  %29 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %28, i32 noundef 4)
  store i32 %29, ptr %6, align 4
  %30 = load i32, ptr %5, align 4
  %31 = icmp eq i32 %30, -1
  br i1 %31, label %32, label %33

32:                                               ; preds = %24
  br label %123

33:                                               ; preds = %24
  %34 = load ptr, ptr %3, align 8
  %35 = load i32, ptr %5, align 4
  %36 = sitofp i32 %35 to double
  %37 = load i32, ptr %6, align 4
  %38 = sitofp i32 %37 to double
  %39 = fdiv double %38, 1.024000e+03
  %40 = fadd double %36, %39
  call void @lua_pushnumber(ptr noundef %34, double noundef %40)
  store i32 1, ptr %2, align 4
  br label %125

41:                                               ; preds = %1
  %42 = load ptr, ptr %3, align 8
  %43 = call i64 @luaL_optinteger(ptr noundef %42, i32 noundef 2, i64 noundef 0)
  %44 = trunc i64 %43 to i32
  store i32 %44, ptr %7, align 4
  %45 = load ptr, ptr %3, align 8
  %46 = load i32, ptr %4, align 4
  %47 = load i32, ptr %7, align 4
  %48 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %45, i32 noundef %46, i32 noundef %47)
  store i32 %48, ptr %8, align 4
  %49 = load i32, ptr %8, align 4
  %50 = icmp eq i32 %49, -1
  br i1 %50, label %51, label %52

51:                                               ; preds = %41
  br label %123

52:                                               ; preds = %41
  %53 = load ptr, ptr %3, align 8
  %54 = load i32, ptr %8, align 4
  call void @lua_pushboolean(ptr noundef %53, i32 noundef %54)
  store i32 1, ptr %2, align 4
  br label %125

55:                                               ; preds = %1, %1
  %56 = load ptr, ptr %3, align 8
  %57 = call i64 @luaL_optinteger(ptr noundef %56, i32 noundef 2, i64 noundef 0)
  %58 = trunc i64 %57 to i32
  store i32 %58, ptr %9, align 4
  %59 = load ptr, ptr %3, align 8
  %60 = load i32, ptr %4, align 4
  %61 = load i32, ptr %9, align 4
  %62 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %59, i32 noundef %60, i32 noundef %61)
  store i32 %62, ptr %10, align 4
  %63 = load i32, ptr %10, align 4
  %64 = icmp eq i32 %63, -1
  br i1 %64, label %65, label %66

65:                                               ; preds = %55
  br label %123

66:                                               ; preds = %55
  %67 = load ptr, ptr %3, align 8
  %68 = load i32, ptr %10, align 4
  %69 = sext i32 %68 to i64
  call void @lua_pushinteger(ptr noundef %67, i64 noundef %69)
  store i32 1, ptr %2, align 4
  br label %125

70:                                               ; preds = %1
  %71 = load ptr, ptr %3, align 8
  %72 = load i32, ptr %4, align 4
  %73 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %71, i32 noundef %72)
  store i32 %73, ptr %11, align 4
  %74 = load i32, ptr %11, align 4
  %75 = icmp eq i32 %74, -1
  br i1 %75, label %76, label %77

76:                                               ; preds = %70
  br label %123

77:                                               ; preds = %70
  %78 = load ptr, ptr %3, align 8
  %79 = load i32, ptr %11, align 4
  call void @lua_pushboolean(ptr noundef %78, i32 noundef %79)
  store i32 1, ptr %2, align 4
  br label %125

80:                                               ; preds = %1
  %81 = load ptr, ptr %3, align 8
  %82 = call i64 @luaL_optinteger(ptr noundef %81, i32 noundef 2, i64 noundef 0)
  %83 = trunc i64 %82 to i32
  store i32 %83, ptr %12, align 4
  %84 = load ptr, ptr %3, align 8
  %85 = call i64 @luaL_optinteger(ptr noundef %84, i32 noundef 3, i64 noundef 0)
  %86 = trunc i64 %85 to i32
  store i32 %86, ptr %13, align 4
  %87 = load ptr, ptr %3, align 8
  %88 = load ptr, ptr %3, align 8
  %89 = load i32, ptr %4, align 4
  %90 = load i32, ptr %12, align 4
  %91 = load i32, ptr %13, align 4
  %92 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %88, i32 noundef %89, i32 noundef %90, i32 noundef %91)
  %93 = call i32 @pushmode(ptr noundef %87, i32 noundef %92)
  store i32 %93, ptr %2, align 4
  br label %125

94:                                               ; preds = %1
  %95 = load ptr, ptr %3, align 8
  %96 = call i64 @luaL_optinteger(ptr noundef %95, i32 noundef 2, i64 noundef 0)
  %97 = trunc i64 %96 to i32
  store i32 %97, ptr %14, align 4
  %98 = load ptr, ptr %3, align 8
  %99 = call i64 @luaL_optinteger(ptr noundef %98, i32 noundef 3, i64 noundef 0)
  %100 = trunc i64 %99 to i32
  store i32 %100, ptr %15, align 4
  %101 = load ptr, ptr %3, align 8
  %102 = call i64 @luaL_optinteger(ptr noundef %101, i32 noundef 4, i64 noundef 0)
  %103 = trunc i64 %102 to i32
  store i32 %103, ptr %16, align 4
  %104 = load ptr, ptr %3, align 8
  %105 = load ptr, ptr %3, align 8
  %106 = load i32, ptr %4, align 4
  %107 = load i32, ptr %14, align 4
  %108 = load i32, ptr %15, align 4
  %109 = load i32, ptr %16, align 4
  %110 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %105, i32 noundef %106, i32 noundef %107, i32 noundef %108, i32 noundef %109)
  %111 = call i32 @pushmode(ptr noundef %104, i32 noundef %110)
  store i32 %111, ptr %2, align 4
  br label %125

112:                                              ; preds = %1
  %113 = load ptr, ptr %3, align 8
  %114 = load i32, ptr %4, align 4
  %115 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %113, i32 noundef %114)
  store i32 %115, ptr %17, align 4
  %116 = load i32, ptr %17, align 4
  %117 = icmp eq i32 %116, -1
  br i1 %117, label %118, label %119

118:                                              ; preds = %112
  br label %123

119:                                              ; preds = %112
  %120 = load ptr, ptr %3, align 8
  %121 = load i32, ptr %17, align 4
  %122 = sext i32 %121 to i64
  call void @lua_pushinteger(ptr noundef %120, i64 noundef %122)
  store i32 1, ptr %2, align 4
  br label %125

123:                                              ; preds = %118, %76, %65, %51, %32
  %124 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %124)
  store i32 1, ptr %2, align 4
  br label %125

125:                                              ; preds = %123, %119, %94, %80, %77, %66, %52, %33
  %126 = load i32, ptr %2, align 4
  ret i32 %126
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_dofile(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call ptr @luaL_optlstring(ptr noundef %5, i32 noundef 1, ptr noundef null, ptr noundef null)
  store ptr %6, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @luaL_loadfilex(ptr noundef %8, ptr noundef %9, ptr noundef null)
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %20

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  %19 = call i32 @lua_error(ptr noundef %18)
  store i32 %19, ptr %2, align 4
  br label %24

20:                                               ; preds = %1
  %21 = load ptr, ptr %3, align 8
  call void @lua_callk(ptr noundef %21, i32 noundef 0, i32 noundef -1, i64 noundef 0, ptr noundef @dofilecont)
  %22 = load ptr, ptr %3, align 8
  %23 = call i32 @dofilecont(ptr noundef %22, i32 noundef 0, i64 noundef 0)
  store i32 %23, ptr %2, align 4
  br label %24

24:                                               ; preds = %20, %17
  %25 = load i32, ptr %2, align 4
  ret i32 %25
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_error(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i64 @luaL_optinteger(ptr noundef %4, i32 noundef 2, i64 noundef 1)
  %6 = trunc i64 %5 to i32
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 1)
  %10 = icmp eq i32 %9, 4
  br i1 %10, label %11, label %19

11:                                               ; preds = %1
  %12 = load i32, ptr %3, align 4
  %13 = icmp sgt i32 %12, 0
  br i1 %13, label %14, label %19

14:                                               ; preds = %11
  %15 = load ptr, ptr %2, align 8
  %16 = load i32, ptr %3, align 4
  call void @luaL_where(ptr noundef %15, i32 noundef %16)
  %17 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %17, i32 noundef 1)
  %18 = load ptr, ptr %2, align 8
  call void @lua_concat(ptr noundef %18, i32 noundef 2)
  br label %19

19:                                               ; preds = %14, %11, %1
  %20 = load ptr, ptr %2, align 8
  %21 = call i32 @lua_error(ptr noundef %20)
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_getmetatable(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 1)
  %5 = load ptr, ptr %3, align 8
  %6 = call i32 @lua_getmetatable(ptr noundef %5, i32 noundef 1)
  %7 = icmp ne i32 %6, 0
  br i1 %7, label %10, label %8

8:                                                ; preds = %1
  %9 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %9)
  store i32 1, ptr %2, align 4
  br label %13

10:                                               ; preds = %1
  %11 = load ptr, ptr %3, align 8
  %12 = call i32 @luaL_getmetafield(ptr noundef %11, i32 noundef 1, ptr noundef @.str.37)
  store i32 1, ptr %2, align 4
  br label %13

13:                                               ; preds = %10, %8
  %14 = load i32, ptr %2, align 4
  ret i32 %14
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_ipairs(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 1)
  %4 = load ptr, ptr %2, align 8
  call void @lua_pushcclosure(ptr noundef %4, ptr noundef @ipairsaux, i32 noundef 0)
  %5 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %5, i32 noundef 1)
  %6 = load ptr, ptr %2, align 8
  call void @lua_pushinteger(ptr noundef %6, i64 noundef 0)
  ret i32 3
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_loadfile(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_optlstring(ptr noundef %7, i32 noundef 1, ptr noundef null, ptr noundef null)
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @luaL_optlstring(ptr noundef %9, i32 noundef 2, ptr noundef null, ptr noundef null)
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = call i32 @lua_type(ptr noundef %11, i32 noundef 3)
  %13 = icmp eq i32 %12, -1
  %14 = xor i1 %13, true
  %15 = zext i1 %14 to i64
  %16 = select i1 %14, i32 3, i32 0
  store i32 %16, ptr %5, align 4
  %17 = load ptr, ptr %2, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @luaL_loadfilex(ptr noundef %17, ptr noundef %18, ptr noundef %19)
  store i32 %20, ptr %6, align 4
  %21 = load ptr, ptr %2, align 8
  %22 = load i32, ptr %6, align 4
  %23 = load i32, ptr %5, align 4
  %24 = call i32 @load_aux(ptr noundef %21, i32 noundef %22, i32 noundef %23)
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_load(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = call ptr @lua_tolstring(ptr noundef %10, i32 noundef 1, ptr noundef %4)
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %2, align 8
  %13 = call ptr @luaL_optlstring(ptr noundef %12, i32 noundef 3, ptr noundef @.str.38, ptr noundef null)
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = call i32 @lua_type(ptr noundef %14, i32 noundef 4)
  %16 = icmp eq i32 %15, -1
  %17 = xor i1 %16, true
  %18 = zext i1 %17 to i64
  %19 = select i1 %17, i32 4, i32 0
  store i32 %19, ptr %7, align 4
  %20 = load ptr, ptr %5, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %32

22:                                               ; preds = %1
  %23 = load ptr, ptr %2, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = call ptr @luaL_optlstring(ptr noundef %23, i32 noundef 2, ptr noundef %24, ptr noundef null)
  store ptr %25, ptr %8, align 8
  %26 = load ptr, ptr %2, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = load i64, ptr %4, align 8
  %29 = load ptr, ptr %8, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = call i32 @luaL_loadbufferx(ptr noundef %26, ptr noundef %27, i64 noundef %28, ptr noundef %29, ptr noundef %30)
  store i32 %31, ptr %3, align 4
  br label %41

32:                                               ; preds = %1
  %33 = load ptr, ptr %2, align 8
  %34 = call ptr @luaL_optlstring(ptr noundef %33, i32 noundef 2, ptr noundef @.str.39, ptr noundef null)
  store ptr %34, ptr %9, align 8
  %35 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %35, i32 noundef 1, i32 noundef 6)
  %36 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %36, i32 noundef 5)
  %37 = load ptr, ptr %2, align 8
  %38 = load ptr, ptr %9, align 8
  %39 = load ptr, ptr %6, align 8
  %40 = call i32 @lua_load(ptr noundef %37, ptr noundef @generic_reader, ptr noundef null, ptr noundef %38, ptr noundef %39)
  store i32 %40, ptr %3, align 4
  br label %41

41:                                               ; preds = %32, %22
  %42 = load ptr, ptr %2, align 8
  %43 = load i32, ptr %3, align 4
  %44 = load i32, ptr %7, align 4
  %45 = call i32 @load_aux(ptr noundef %42, i32 noundef %43, i32 noundef %44)
  ret i32 %45
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_next(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  call void @luaL_checktype(ptr noundef %4, i32 noundef 1, i32 noundef 5)
  %5 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %5, i32 noundef 2)
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_next(ptr noundef %6, i32 noundef 1)
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %10

9:                                                ; preds = %1
  store i32 2, ptr %2, align 4
  br label %12

10:                                               ; preds = %1
  %11 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %11)
  store i32 1, ptr %2, align 4
  br label %12

12:                                               ; preds = %10, %9
  %13 = load i32, ptr %2, align 4
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_pairs(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 1)
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @luaL_getmetafield(ptr noundef %4, i32 noundef 1, ptr noundef @.str.42)
  %6 = icmp eq i32 %5, 0
  br i1 %6, label %7, label %11

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  call void @lua_pushcclosure(ptr noundef %8, ptr noundef @luaB_next, i32 noundef 0)
  %9 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %9, i32 noundef 1)
  %10 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %10)
  br label %14

11:                                               ; preds = %1
  %12 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %12, i32 noundef 1)
  %13 = load ptr, ptr %2, align 8
  call void @lua_callk(ptr noundef %13, i32 noundef 1, i32 noundef 3, i64 noundef 0, ptr noundef @pairscont)
  br label %14

14:                                               ; preds = %11, %7
  ret i32 3
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_pcall(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 1)
  %5 = load ptr, ptr %2, align 8
  call void @lua_pushboolean(ptr noundef %5, i32 noundef 1)
  %6 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %6, i32 noundef 1, i32 noundef 1)
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_gettop(ptr noundef %8)
  %10 = sub nsw i32 %9, 2
  %11 = call i32 @lua_pcallk(ptr noundef %7, i32 noundef %10, i32 noundef -1, i32 noundef 0, i64 noundef 0, ptr noundef @finishpcall)
  store i32 %11, ptr %3, align 4
  %12 = load ptr, ptr %2, align 8
  %13 = load i32, ptr %3, align 4
  %14 = call i32 @finishpcall(ptr noundef %12, i32 noundef %13, i64 noundef 0)
  ret i32 %14
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_print(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call i32 @lua_gettop(ptr noundef %7)
  store i32 %8, ptr %3, align 4
  store i32 1, ptr %4, align 4
  br label %9

9:                                                ; preds = %28, %1
  %10 = load i32, ptr %4, align 4
  %11 = load i32, ptr %3, align 4
  %12 = icmp sle i32 %10, %11
  br i1 %12, label %13, label %31

13:                                               ; preds = %9
  %14 = load ptr, ptr %2, align 8
  %15 = load i32, ptr %4, align 4
  %16 = call ptr @luaL_tolstring(ptr noundef %14, i32 noundef %15, ptr noundef %5)
  store ptr %16, ptr %6, align 8
  %17 = load i32, ptr %4, align 4
  %18 = icmp sgt i32 %17, 1
  br i1 %18, label %19, label %22

19:                                               ; preds = %13
  %20 = load ptr, ptr @stdout, align 8
  %21 = call i64 @fwrite(ptr noundef @.str.43, i64 noundef 1, i64 noundef 1, ptr noundef %20)
  br label %22

22:                                               ; preds = %19, %13
  %23 = load ptr, ptr %6, align 8
  %24 = load i64, ptr %5, align 8
  %25 = load ptr, ptr @stdout, align 8
  %26 = call i64 @fwrite(ptr noundef %23, i64 noundef 1, i64 noundef %24, ptr noundef %25)
  %27 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %27, i32 noundef -2)
  br label %28

28:                                               ; preds = %22
  %29 = load i32, ptr %4, align 4
  %30 = add nsw i32 %29, 1
  store i32 %30, ptr %4, align 4
  br label %9, !llvm.loop !6

31:                                               ; preds = %9
  %32 = load ptr, ptr @stdout, align 8
  %33 = call i64 @fwrite(ptr noundef @.str.44, i64 noundef 1, i64 noundef 1, ptr noundef %32)
  %34 = load ptr, ptr @stdout, align 8
  %35 = call i32 @fflush(ptr noundef %34)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_warn(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_gettop(ptr noundef %5)
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 1, ptr noundef null)
  store i32 2, ptr %4, align 4
  br label %9

9:                                                ; preds = %17, %1
  %10 = load i32, ptr %4, align 4
  %11 = load i32, ptr %3, align 4
  %12 = icmp sle i32 %10, %11
  br i1 %12, label %13, label %20

13:                                               ; preds = %9
  %14 = load ptr, ptr %2, align 8
  %15 = load i32, ptr %4, align 4
  %16 = call ptr @luaL_checklstring(ptr noundef %14, i32 noundef %15, ptr noundef null)
  br label %17

17:                                               ; preds = %13
  %18 = load i32, ptr %4, align 4
  %19 = add nsw i32 %18, 1
  store i32 %19, ptr %4, align 4
  br label %9, !llvm.loop !8

20:                                               ; preds = %9
  store i32 1, ptr %4, align 4
  br label %21

21:                                               ; preds = %30, %20
  %22 = load i32, ptr %4, align 4
  %23 = load i32, ptr %3, align 4
  %24 = icmp slt i32 %22, %23
  br i1 %24, label %25, label %33

25:                                               ; preds = %21
  %26 = load ptr, ptr %2, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = load i32, ptr %4, align 4
  %29 = call ptr @lua_tolstring(ptr noundef %27, i32 noundef %28, ptr noundef null)
  call void @lua_warning(ptr noundef %26, ptr noundef %29, i32 noundef 1)
  br label %30

30:                                               ; preds = %25
  %31 = load i32, ptr %4, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %4, align 4
  br label %21, !llvm.loop !9

33:                                               ; preds = %21
  %34 = load ptr, ptr %2, align 8
  %35 = load ptr, ptr %2, align 8
  %36 = load i32, ptr %3, align 4
  %37 = call ptr @lua_tolstring(ptr noundef %35, i32 noundef %36, ptr noundef null)
  call void @lua_warning(ptr noundef %34, ptr noundef %37, i32 noundef 0)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_rawequal(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 1)
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 2)
  %5 = load ptr, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_rawequal(ptr noundef %6, i32 noundef 1, i32 noundef 2)
  call void @lua_pushboolean(ptr noundef %5, i32 noundef %7)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_rawlen(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef 1)
  store i32 %5, ptr %3, align 4
  %6 = load i32, ptr %3, align 4
  %7 = icmp eq i32 %6, 5
  br i1 %7, label %11, label %8

8:                                                ; preds = %1
  %9 = load i32, ptr %3, align 4
  %10 = icmp eq i32 %9, 4
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
  %20 = call i32 @luaL_typeerror(ptr noundef %19, i32 noundef 1, ptr noundef @.str.45)
  %21 = icmp ne i32 %20, 0
  br label %22

22:                                               ; preds = %18, %11
  %23 = phi i1 [ true, %11 ], [ %21, %18 ]
  %24 = zext i1 %23 to i32
  %25 = load ptr, ptr %2, align 8
  %26 = load ptr, ptr %2, align 8
  %27 = call i64 @lua_rawlen(ptr noundef %26, i32 noundef 1)
  call void @lua_pushinteger(ptr noundef %25, i64 noundef %27)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_rawget(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %3, i32 noundef 1, i32 noundef 5)
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 2)
  %5 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %5, i32 noundef 2)
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_rawget(ptr noundef %6, i32 noundef 1)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_rawset(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %3, i32 noundef 1, i32 noundef 5)
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 2)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %5, i32 noundef 3)
  %6 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %6, i32 noundef 3)
  %7 = load ptr, ptr %2, align 8
  call void @lua_rawset(ptr noundef %7, i32 noundef 1)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_select(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_gettop(ptr noundef %6)
  store i32 %7, ptr %4, align 4
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 1)
  %10 = icmp eq i32 %9, 4
  br i1 %10, label %11, label %22

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @lua_tolstring(ptr noundef %12, i32 noundef 1, ptr noundef null)
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp eq i32 %15, 35
  br i1 %16, label %17, label %22

17:                                               ; preds = %11
  %18 = load ptr, ptr %3, align 8
  %19 = load i32, ptr %4, align 4
  %20 = sub nsw i32 %19, 1
  %21 = sext i32 %20 to i64
  call void @lua_pushinteger(ptr noundef %18, i64 noundef %21)
  store i32 1, ptr %2, align 4
  br label %60

22:                                               ; preds = %11, %1
  %23 = load ptr, ptr %3, align 8
  %24 = call i64 @luaL_checkinteger(ptr noundef %23, i32 noundef 1)
  store i64 %24, ptr %5, align 8
  %25 = load i64, ptr %5, align 8
  %26 = icmp slt i64 %25, 0
  br i1 %26, label %27, label %32

27:                                               ; preds = %22
  %28 = load i32, ptr %4, align 4
  %29 = sext i32 %28 to i64
  %30 = load i64, ptr %5, align 8
  %31 = add nsw i64 %29, %30
  store i64 %31, ptr %5, align 8
  br label %41

32:                                               ; preds = %22
  %33 = load i64, ptr %5, align 8
  %34 = load i32, ptr %4, align 4
  %35 = sext i32 %34 to i64
  %36 = icmp sgt i64 %33, %35
  br i1 %36, label %37, label %40

37:                                               ; preds = %32
  %38 = load i32, ptr %4, align 4
  %39 = sext i32 %38 to i64
  store i64 %39, ptr %5, align 8
  br label %40

40:                                               ; preds = %37, %32
  br label %41

41:                                               ; preds = %40, %27
  %42 = load i64, ptr %5, align 8
  %43 = icmp sle i64 1, %42
  %44 = zext i1 %43 to i32
  %45 = icmp ne i32 %44, 0
  %46 = zext i1 %45 to i32
  %47 = sext i32 %46 to i64
  %48 = icmp ne i64 %47, 0
  br i1 %48, label %53, label %49

49:                                               ; preds = %41
  %50 = load ptr, ptr %3, align 8
  %51 = call i32 @luaL_argerror(ptr noundef %50, i32 noundef 1, ptr noundef @.str.46)
  %52 = icmp ne i32 %51, 0
  br label %53

53:                                               ; preds = %49, %41
  %54 = phi i1 [ true, %41 ], [ %52, %49 ]
  %55 = zext i1 %54 to i32
  %56 = load i32, ptr %4, align 4
  %57 = load i64, ptr %5, align 8
  %58 = trunc i64 %57 to i32
  %59 = sub nsw i32 %56, %58
  store i32 %59, ptr %2, align 4
  br label %60

60:                                               ; preds = %53, %17
  %61 = load i32, ptr %2, align 4
  ret i32 %61
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_setmetatable(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call i32 @lua_type(ptr noundef %5, i32 noundef 2)
  store i32 %6, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  call void @luaL_checktype(ptr noundef %7, i32 noundef 1, i32 noundef 5)
  %8 = load i32, ptr %4, align 4
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %13, label %10

10:                                               ; preds = %1
  %11 = load i32, ptr %4, align 4
  %12 = icmp eq i32 %11, 5
  br label %13

13:                                               ; preds = %10, %1
  %14 = phi i1 [ true, %1 ], [ %12, %10 ]
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %24, label %20

20:                                               ; preds = %13
  %21 = load ptr, ptr %3, align 8
  %22 = call i32 @luaL_typeerror(ptr noundef %21, i32 noundef 2, ptr noundef @.str.47)
  %23 = icmp ne i32 %22, 0
  br label %24

24:                                               ; preds = %20, %13
  %25 = phi i1 [ true, %13 ], [ %23, %20 ]
  %26 = zext i1 %25 to i32
  %27 = load ptr, ptr %3, align 8
  %28 = call i32 @luaL_getmetafield(ptr noundef %27, i32 noundef 1, ptr noundef @.str.37)
  %29 = icmp ne i32 %28, 0
  %30 = zext i1 %29 to i32
  %31 = icmp ne i32 %30, 0
  %32 = zext i1 %31 to i32
  %33 = sext i32 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %35, label %38

35:                                               ; preds = %24
  %36 = load ptr, ptr %3, align 8
  %37 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %36, ptr noundef @.str.48)
  store i32 %37, ptr %2, align 4
  br label %42

38:                                               ; preds = %24
  %39 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %39, i32 noundef 2)
  %40 = load ptr, ptr %3, align 8
  %41 = call i32 @lua_setmetatable(ptr noundef %40, i32 noundef 1)
  store i32 1, ptr %2, align 4
  br label %42

42:                                               ; preds = %38, %35
  %43 = load i32, ptr %2, align 4
  ret i32 %43
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_tonumber(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @lua_type(ptr noundef %10, i32 noundef 2)
  %12 = icmp sle i32 %11, 0
  br i1 %12, label %13, label %35

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @lua_type(ptr noundef %14, i32 noundef 1)
  %16 = icmp eq i32 %15, 3
  br i1 %16, label %17, label %19

17:                                               ; preds = %13
  %18 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %18, i32 noundef 1)
  store i32 1, ptr %2, align 4
  br label %74

19:                                               ; preds = %13
  %20 = load ptr, ptr %3, align 8
  %21 = call ptr @lua_tolstring(ptr noundef %20, i32 noundef 1, ptr noundef %4)
  store ptr %21, ptr %5, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = icmp ne ptr %22, null
  br i1 %23, label %24, label %32

24:                                               ; preds = %19
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = call i64 @lua_stringtonumber(ptr noundef %25, ptr noundef %26)
  %28 = load i64, ptr %4, align 8
  %29 = add i64 %28, 1
  %30 = icmp eq i64 %27, %29
  br i1 %30, label %31, label %32

31:                                               ; preds = %24
  store i32 1, ptr %2, align 4
  br label %74

32:                                               ; preds = %24, %19
  %33 = load ptr, ptr %3, align 8
  call void @luaL_checkany(ptr noundef %33, i32 noundef 1)
  br label %34

34:                                               ; preds = %32
  br label %72

35:                                               ; preds = %1
  store i64 0, ptr %8, align 8
  %36 = load ptr, ptr %3, align 8
  %37 = call i64 @luaL_checkinteger(ptr noundef %36, i32 noundef 2)
  store i64 %37, ptr %9, align 8
  %38 = load ptr, ptr %3, align 8
  call void @luaL_checktype(ptr noundef %38, i32 noundef 1, i32 noundef 4)
  %39 = load ptr, ptr %3, align 8
  %40 = call ptr @lua_tolstring(ptr noundef %39, i32 noundef 1, ptr noundef %6)
  store ptr %40, ptr %7, align 8
  %41 = load i64, ptr %9, align 8
  %42 = icmp sle i64 2, %41
  br i1 %42, label %43, label %46

43:                                               ; preds = %35
  %44 = load i64, ptr %9, align 8
  %45 = icmp sle i64 %44, 36
  br label %46

46:                                               ; preds = %43, %35
  %47 = phi i1 [ false, %35 ], [ %45, %43 ]
  %48 = zext i1 %47 to i32
  %49 = icmp ne i32 %48, 0
  %50 = zext i1 %49 to i32
  %51 = sext i32 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %57, label %53

53:                                               ; preds = %46
  %54 = load ptr, ptr %3, align 8
  %55 = call i32 @luaL_argerror(ptr noundef %54, i32 noundef 2, ptr noundef @.str.49)
  %56 = icmp ne i32 %55, 0
  br label %57

57:                                               ; preds = %53, %46
  %58 = phi i1 [ true, %46 ], [ %56, %53 ]
  %59 = zext i1 %58 to i32
  %60 = load ptr, ptr %7, align 8
  %61 = load i64, ptr %9, align 8
  %62 = trunc i64 %61 to i32
  %63 = call ptr @b_str2int(ptr noundef %60, i32 noundef %62, ptr noundef %8)
  %64 = load ptr, ptr %7, align 8
  %65 = load i64, ptr %6, align 8
  %66 = getelementptr inbounds i8, ptr %64, i64 %65
  %67 = icmp eq ptr %63, %66
  br i1 %67, label %68, label %71

68:                                               ; preds = %57
  %69 = load ptr, ptr %3, align 8
  %70 = load i64, ptr %8, align 8
  call void @lua_pushinteger(ptr noundef %69, i64 noundef %70)
  store i32 1, ptr %2, align 4
  br label %74

71:                                               ; preds = %57
  br label %72

72:                                               ; preds = %71, %34
  %73 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %73)
  store i32 1, ptr %2, align 4
  br label %74

74:                                               ; preds = %72, %68, %31, %17
  %75 = load i32, ptr %2, align 4
  ret i32 %75
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_tostring(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %3, i32 noundef 1)
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_tolstring(ptr noundef %4, i32 noundef 1, ptr noundef null)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_type(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_type(ptr noundef %4, i32 noundef 1)
  store i32 %5, ptr %3, align 4
  %6 = load i32, ptr %3, align 4
  %7 = icmp ne i32 %6, -1
  %8 = zext i1 %7 to i32
  %9 = icmp ne i32 %8, 0
  %10 = zext i1 %9 to i32
  %11 = sext i32 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %17, label %13

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  %15 = call i32 @luaL_argerror(ptr noundef %14, i32 noundef 1, ptr noundef @.str.51)
  %16 = icmp ne i32 %15, 0
  br label %17

17:                                               ; preds = %13, %1
  %18 = phi i1 [ true, %1 ], [ %16, %13 ]
  %19 = zext i1 %18 to i32
  %20 = load ptr, ptr %2, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = load i32, ptr %3, align 4
  %23 = call ptr @lua_typename(ptr noundef %21, i32 noundef %22)
  %24 = call ptr @lua_pushstring(ptr noundef %20, ptr noundef %23)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @luaB_xpcall(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_gettop(ptr noundef %5)
  store i32 %6, ptr %4, align 4
  %7 = load ptr, ptr %2, align 8
  call void @luaL_checktype(ptr noundef %7, i32 noundef 2, i32 noundef 6)
  %8 = load ptr, ptr %2, align 8
  call void @lua_pushboolean(ptr noundef %8, i32 noundef 1)
  %9 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %9, i32 noundef 1)
  %10 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %10, i32 noundef 3, i32 noundef 2)
  %11 = load ptr, ptr %2, align 8
  %12 = load i32, ptr %4, align 4
  %13 = sub nsw i32 %12, 2
  %14 = call i32 @lua_pcallk(ptr noundef %11, i32 noundef %13, i32 noundef -1, i32 noundef 2, i64 noundef 2, ptr noundef @finishpcall)
  store i32 %14, ptr %3, align 4
  %15 = load ptr, ptr %2, align 8
  %16 = load i32, ptr %3, align 4
  %17 = call i32 @finishpcall(ptr noundef %15, i32 noundef %16, i64 noundef 2)
  ret i32 %17
}

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

declare i32 @lua_gettop(ptr noundef) #1

declare void @luaL_checkany(ptr noundef, i32 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare i32 @luaL_checkoption(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i32 @lua_gc(ptr noundef, i32 noundef, ...) #1

declare void @lua_pushnumber(ptr noundef, double noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pushmode(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = icmp eq i32 %5, -1
  br i1 %6, label %7, label %9

7:                                                ; preds = %2
  %8 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %8)
  br label %16

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load i32, ptr %4, align 4
  %12 = icmp eq i32 %11, 11
  %13 = zext i1 %12 to i64
  %14 = select i1 %12, ptr @.str.36, ptr @.str.35
  %15 = call ptr @lua_pushstring(ptr noundef %10, ptr noundef %14)
  br label %16

16:                                               ; preds = %9, %7
  ret i32 1
}

declare void @lua_pushnil(ptr noundef) #1

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i32 @luaL_loadfilex(ptr noundef, ptr noundef, ptr noundef) #1

declare i32 @lua_error(ptr noundef) #1

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @dofilecont(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %7 = load i32, ptr %5, align 4
  %8 = load i64, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @lua_gettop(ptr noundef %9)
  %11 = sub nsw i32 %10, 1
  ret i32 %11
}

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare void @luaL_where(ptr noundef, i32 noundef) #1

declare void @lua_concat(ptr noundef, i32 noundef) #1

declare i32 @lua_getmetatable(ptr noundef, i32 noundef) #1

declare i32 @luaL_getmetafield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @ipairsaux(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i64 @luaL_checkinteger(ptr noundef %4, i32 noundef 2)
  store i64 %5, ptr %3, align 8
  %6 = load i64, ptr %3, align 8
  %7 = add i64 %6, 1
  store i64 %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load i64, ptr %3, align 8
  call void @lua_pushinteger(ptr noundef %8, i64 noundef %9)
  %10 = load ptr, ptr %2, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call i32 @lua_geti(ptr noundef %10, i32 noundef 1, i64 noundef %11)
  %13 = icmp eq i32 %12, 0
  %14 = zext i1 %13 to i64
  %15 = select i1 %13, i32 1, i32 2
  ret i32 %15
}

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare i32 @lua_geti(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @load_aux(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i32 %2, ptr %7, align 4
  %8 = load i32, ptr %6, align 4
  %9 = icmp eq i32 %8, 0
  %10 = zext i1 %9 to i32
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %15, label %28

15:                                               ; preds = %3
  %16 = load i32, ptr %7, align 4
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %27

18:                                               ; preds = %15
  %19 = load ptr, ptr %5, align 8
  %20 = load i32, ptr %7, align 4
  call void @lua_pushvalue(ptr noundef %19, i32 noundef %20)
  %21 = load ptr, ptr %5, align 8
  %22 = call ptr @lua_setupvalue(ptr noundef %21, i32 noundef -2, i32 noundef 1)
  %23 = icmp ne ptr %22, null
  br i1 %23, label %26, label %24

24:                                               ; preds = %18
  %25 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %25, i32 noundef -2)
  br label %26

26:                                               ; preds = %24, %18
  br label %27

27:                                               ; preds = %26, %15
  store i32 1, ptr %4, align 4
  br label %31

28:                                               ; preds = %3
  %29 = load ptr, ptr %5, align 8
  call void @lua_pushnil(ptr noundef %29)
  %30 = load ptr, ptr %5, align 8
  call void @lua_rotate(ptr noundef %30, i32 noundef -2, i32 noundef 1)
  store i32 2, ptr %4, align 4
  br label %31

31:                                               ; preds = %28, %27
  %32 = load i32, ptr %4, align 4
  ret i32 %32
}

declare ptr @lua_setupvalue(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_loadbufferx(ptr noundef, ptr noundef, i64 noundef, ptr noundef, ptr noundef) #1

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare i32 @lua_load(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @generic_reader(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = load ptr, ptr %5, align 8
  call void @luaL_checkstack(ptr noundef %9, i32 noundef 2, ptr noundef @.str.40)
  %10 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %10, i32 noundef 1)
  %11 = load ptr, ptr %5, align 8
  call void @lua_callk(ptr noundef %11, i32 noundef 0, i32 noundef 1, i64 noundef 0, ptr noundef null)
  %12 = load ptr, ptr %5, align 8
  %13 = call i32 @lua_type(ptr noundef %12, i32 noundef -1)
  %14 = icmp eq i32 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %3
  %16 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %16, i32 noundef -2)
  %17 = load ptr, ptr %7, align 8
  store i64 0, ptr %17, align 8
  store ptr null, ptr %4, align 8
  br label %38

18:                                               ; preds = %3
  %19 = load ptr, ptr %5, align 8
  %20 = call i32 @lua_isstring(ptr noundef %19, i32 noundef -1)
  %21 = icmp ne i32 %20, 0
  %22 = xor i1 %21, true
  %23 = zext i1 %22 to i32
  %24 = icmp ne i32 %23, 0
  %25 = zext i1 %24 to i32
  %26 = sext i32 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %28, label %31

28:                                               ; preds = %18
  %29 = load ptr, ptr %5, align 8
  %30 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %29, ptr noundef @.str.41)
  br label %31

31:                                               ; preds = %28, %18
  br label %32

32:                                               ; preds = %31
  %33 = load ptr, ptr %5, align 8
  call void @lua_copy(ptr noundef %33, i32 noundef -1, i32 noundef 5)
  %34 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %34, i32 noundef -2)
  %35 = load ptr, ptr %5, align 8
  %36 = load ptr, ptr %7, align 8
  %37 = call ptr @lua_tolstring(ptr noundef %35, i32 noundef 5, ptr noundef %36)
  store ptr %37, ptr %4, align 8
  br label %38

38:                                               ; preds = %32, %15
  %39 = load ptr, ptr %4, align 8
  ret ptr %39
}

declare void @luaL_checkstack(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_isstring(ptr noundef, i32 noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare void @lua_copy(ptr noundef, i32 noundef, i32 noundef) #1

declare i32 @lua_next(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pairscont(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i64 %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = load i64, ptr %6, align 8
  ret i32 3
}

declare i32 @lua_pcallk(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @finishpcall(ptr noundef %0, i32 noundef %1, i64 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store i64 %2, ptr %7, align 8
  %8 = load i32, ptr %6, align 4
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %13

10:                                               ; preds = %3
  %11 = load i32, ptr %6, align 4
  %12 = icmp ne i32 %11, 1
  br label %13

13:                                               ; preds = %10, %3
  %14 = phi i1 [ false, %3 ], [ %12, %10 ]
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %13
  %21 = load ptr, ptr %5, align 8
  call void @lua_pushboolean(ptr noundef %21, i32 noundef 0)
  %22 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %22, i32 noundef -2)
  store i32 2, ptr %4, align 4
  br label %29

23:                                               ; preds = %13
  %24 = load ptr, ptr %5, align 8
  %25 = call i32 @lua_gettop(ptr noundef %24)
  %26 = load i64, ptr %7, align 8
  %27 = trunc i64 %26 to i32
  %28 = sub nsw i32 %25, %27
  store i32 %28, ptr %4, align 4
  br label %29

29:                                               ; preds = %23, %20
  %30 = load i32, ptr %4, align 4
  ret i32 %30
}

declare ptr @luaL_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @fwrite(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

declare i32 @fflush(ptr noundef) #1

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_warning(ptr noundef, ptr noundef, i32 noundef) #1

declare i32 @lua_rawequal(ptr noundef, i32 noundef, i32 noundef) #1

declare i32 @luaL_typeerror(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @lua_rawlen(ptr noundef, i32 noundef) #1

declare i32 @lua_rawget(ptr noundef, i32 noundef) #1

declare void @lua_rawset(ptr noundef, i32 noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_setmetatable(ptr noundef, i32 noundef) #1

declare i64 @lua_stringtonumber(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @b_str2int(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store i64 0, ptr %8, align 8
  store i32 0, ptr %9, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = call i64 @strspn(ptr noundef %11, ptr noundef @.str.50) #4
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds i8, ptr %13, i64 %12
  store ptr %14, ptr %5, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = load i8, ptr %15, align 1
  %17 = sext i8 %16 to i32
  %18 = icmp eq i32 %17, 45
  br i1 %18, label %19, label %22

19:                                               ; preds = %3
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds i8, ptr %20, i32 1
  store ptr %21, ptr %5, align 8
  store i32 1, ptr %9, align 4
  br label %31

22:                                               ; preds = %3
  %23 = load ptr, ptr %5, align 8
  %24 = load i8, ptr %23, align 1
  %25 = sext i8 %24 to i32
  %26 = icmp eq i32 %25, 43
  br i1 %26, label %27, label %30

27:                                               ; preds = %22
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds i8, ptr %28, i32 1
  store ptr %29, ptr %5, align 8
  br label %30

30:                                               ; preds = %27, %22
  br label %31

31:                                               ; preds = %30, %19
  %32 = call ptr @__ctype_b_loc() #5
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds i16, ptr %33, i64 %37
  %39 = load i16, ptr %38, align 2
  %40 = zext i16 %39 to i32
  %41 = and i32 %40, 8
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %44, label %43

43:                                               ; preds = %31
  store ptr null, ptr %4, align 8
  br label %113

44:                                               ; preds = %31
  br label %45

45:                                               ; preds = %85, %44
  %46 = call ptr @__ctype_b_loc() #5
  %47 = load ptr, ptr %46, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = load i8, ptr %48, align 1
  %50 = zext i8 %49 to i32
  %51 = sext i32 %50 to i64
  %52 = getelementptr inbounds i16, ptr %47, i64 %51
  %53 = load i16, ptr %52, align 2
  %54 = zext i16 %53 to i32
  %55 = and i32 %54, 2048
  %56 = icmp ne i32 %55, 0
  br i1 %56, label %57, label %62

57:                                               ; preds = %45
  %58 = load ptr, ptr %5, align 8
  %59 = load i8, ptr %58, align 1
  %60 = sext i8 %59 to i32
  %61 = sub nsw i32 %60, 48
  br label %69

62:                                               ; preds = %45
  %63 = load ptr, ptr %5, align 8
  %64 = load i8, ptr %63, align 1
  %65 = zext i8 %64 to i32
  %66 = call i32 @toupper(i32 noundef %65) #4
  %67 = sub nsw i32 %66, 65
  %68 = add nsw i32 %67, 10
  br label %69

69:                                               ; preds = %62, %57
  %70 = phi i32 [ %61, %57 ], [ %68, %62 ]
  store i32 %70, ptr %10, align 4
  %71 = load i32, ptr %10, align 4
  %72 = load i32, ptr %6, align 4
  %73 = icmp sge i32 %71, %72
  br i1 %73, label %74, label %75

74:                                               ; preds = %69
  store ptr null, ptr %4, align 8
  br label %113

75:                                               ; preds = %69
  %76 = load i64, ptr %8, align 8
  %77 = load i32, ptr %6, align 4
  %78 = sext i32 %77 to i64
  %79 = mul i64 %76, %78
  %80 = load i32, ptr %10, align 4
  %81 = sext i32 %80 to i64
  %82 = add i64 %79, %81
  store i64 %82, ptr %8, align 8
  %83 = load ptr, ptr %5, align 8
  %84 = getelementptr inbounds i8, ptr %83, i32 1
  store ptr %84, ptr %5, align 8
  br label %85

85:                                               ; preds = %75
  %86 = call ptr @__ctype_b_loc() #5
  %87 = load ptr, ptr %86, align 8
  %88 = load ptr, ptr %5, align 8
  %89 = load i8, ptr %88, align 1
  %90 = zext i8 %89 to i32
  %91 = sext i32 %90 to i64
  %92 = getelementptr inbounds i16, ptr %87, i64 %91
  %93 = load i16, ptr %92, align 2
  %94 = zext i16 %93 to i32
  %95 = and i32 %94, 8
  %96 = icmp ne i32 %95, 0
  br i1 %96, label %45, label %97, !llvm.loop !10

97:                                               ; preds = %85
  %98 = load ptr, ptr %5, align 8
  %99 = call i64 @strspn(ptr noundef %98, ptr noundef @.str.50) #4
  %100 = load ptr, ptr %5, align 8
  %101 = getelementptr inbounds i8, ptr %100, i64 %99
  store ptr %101, ptr %5, align 8
  %102 = load i32, ptr %9, align 4
  %103 = icmp ne i32 %102, 0
  br i1 %103, label %104, label %107

104:                                              ; preds = %97
  %105 = load i64, ptr %8, align 8
  %106 = sub i64 0, %105
  br label %109

107:                                              ; preds = %97
  %108 = load i64, ptr %8, align 8
  br label %109

109:                                              ; preds = %107, %104
  %110 = phi i64 [ %106, %104 ], [ %108, %107 ]
  %111 = load ptr, ptr %7, align 8
  store i64 %110, ptr %111, align 8
  %112 = load ptr, ptr %5, align 8
  store ptr %112, ptr %4, align 8
  br label %113

113:                                              ; preds = %109, %74, %43
  %114 = load ptr, ptr %4, align 8
  ret ptr %114
}

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strspn(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__ctype_b_loc() #3

; Function Attrs: nounwind willreturn memory(read)
declare i32 @toupper(i32 noundef) #2

declare ptr @lua_typename(ptr noundef, i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind willreturn memory(read) }
attributes #5 = { nounwind willreturn memory(none) }

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
