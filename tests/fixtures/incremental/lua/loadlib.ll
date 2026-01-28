; ModuleID = 'loadlib.c'
source_filename = "loadlib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }

@pk_funcs = internal constant [8 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.14, ptr @ll_loadlib }, %struct.luaL_Reg { ptr @.str.15, ptr @ll_searchpath }, %struct.luaL_Reg { ptr @.str.11, ptr null }, %struct.luaL_Reg { ptr @.str.3, ptr null }, %struct.luaL_Reg { ptr @.str, ptr null }, %struct.luaL_Reg { ptr @.str.16, ptr null }, %struct.luaL_Reg { ptr @.str.9, ptr null }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [5 x i8] c"path\00", align 1
@.str.1 = private unnamed_addr constant [9 x i8] c"LUA_PATH\00", align 1
@.str.2 = private unnamed_addr constant [151 x i8] c"/usr/local/share/lua/5.4/?.lua;/usr/local/share/lua/5.4/?/init.lua;/usr/local/lib/lua/5.4/?.lua;/usr/local/lib/lua/5.4/?/init.lua;./?.lua;./?/init.lua\00", align 1
@.str.3 = private unnamed_addr constant [6 x i8] c"cpath\00", align 1
@.str.4 = private unnamed_addr constant [10 x i8] c"LUA_CPATH\00", align 1
@.str.5 = private unnamed_addr constant [69 x i8] c"/usr/local/lib/lua/5.4/?.so;/usr/local/lib/lua/5.4/loadall.so;./?.so\00", align 1
@.str.6 = private unnamed_addr constant [11 x i8] c"/\0A;\0A?\0A!\0A-\0A\00", align 1
@.str.7 = private unnamed_addr constant [7 x i8] c"config\00", align 1
@.str.8 = private unnamed_addr constant [8 x i8] c"_LOADED\00", align 1
@.str.9 = private unnamed_addr constant [7 x i8] c"loaded\00", align 1
@.str.10 = private unnamed_addr constant [9 x i8] c"_PRELOAD\00", align 1
@.str.11 = private unnamed_addr constant [8 x i8] c"preload\00", align 1
@ll_funcs = internal constant [2 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.40, ptr @ll_require }, %struct.luaL_Reg zeroinitializer], align 16
@.str.12 = private unnamed_addr constant [7 x i8] c"_CLIBS\00", align 1
@.str.13 = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@.str.14 = private unnamed_addr constant [8 x i8] c"loadlib\00", align 1
@.str.15 = private unnamed_addr constant [11 x i8] c"searchpath\00", align 1
@.str.16 = private unnamed_addr constant [10 x i8] c"searchers\00", align 1
@.str.17 = private unnamed_addr constant [7 x i8] c"absent\00", align 1
@.str.18 = private unnamed_addr constant [5 x i8] c"init\00", align 1
@.str.19 = private unnamed_addr constant [59 x i8] c"dynamic libraries not enabled; check your Lua installation\00", align 1
@.str.20 = private unnamed_addr constant [2 x i8] c".\00", align 1
@.str.21 = private unnamed_addr constant [2 x i8] c"/\00", align 1
@.str.22 = private unnamed_addr constant [2 x i8] c"?\00", align 1
@.str.23 = private unnamed_addr constant [2 x i8] c";\00", align 1
@.str.24 = private unnamed_addr constant [2 x i8] c"r\00", align 1
@.str.25 = private unnamed_addr constant [10 x i8] c"no file '\00", align 1
@.str.26 = private unnamed_addr constant [13 x i8] c"'\0A\09no file '\00", align 1
@.str.27 = private unnamed_addr constant [2 x i8] c"'\00", align 1
@createsearcherstable.searchers = internal constant [5 x ptr] [ptr @searcher_preload, ptr @searcher_Lua, ptr @searcher_C, ptr @searcher_Croot, ptr null], align 16
@.str.28 = private unnamed_addr constant [31 x i8] c"no field package.preload['%s']\00", align 1
@.str.29 = private unnamed_addr constant [10 x i8] c":preload:\00", align 1
@.str.30 = private unnamed_addr constant [30 x i8] c"'package.%s' must be a string\00", align 1
@.str.31 = private unnamed_addr constant [46 x i8] c"error loading module '%s' from file '%s':\0A\09%s\00", align 1
@.str.32 = private unnamed_addr constant [2 x i8] c"_\00", align 1
@.str.33 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@.str.34 = private unnamed_addr constant [11 x i8] c"luaopen_%s\00", align 1
@.str.35 = private unnamed_addr constant [28 x i8] c"no module '%s' in file '%s'\00", align 1
@.str.36 = private unnamed_addr constant [5 x i8] c"%s%s\00", align 1
@.str.37 = private unnamed_addr constant [5 x i8] c"_5_4\00", align 1
@.str.38 = private unnamed_addr constant [3 x i8] c";;\00", align 1
@.str.39 = private unnamed_addr constant [10 x i8] c"LUA_NOENV\00", align 1
@.str.40 = private unnamed_addr constant [8 x i8] c"require\00", align 1
@.str.41 = private unnamed_addr constant [36 x i8] c"'package.searchers' must be a table\00", align 1
@.str.42 = private unnamed_addr constant [3 x i8] c"\0A\09\00", align 1
@.str.43 = private unnamed_addr constant [25 x i8] c"module '%s' not found:%s\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_package(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @createclibstable(ptr noundef %3)
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %4, double noundef 5.040000e+02, i64 noundef 136)
  %5 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %5, i32 noundef 0, i32 noundef 7)
  %6 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %6, ptr noundef @pk_funcs, i32 noundef 0)
  %7 = load ptr, ptr %2, align 8
  call void @createsearcherstable(ptr noundef %7)
  %8 = load ptr, ptr %2, align 8
  call void @setpath(ptr noundef %8, ptr noundef @.str, ptr noundef @.str.1, ptr noundef @.str.2)
  %9 = load ptr, ptr %2, align 8
  call void @setpath(ptr noundef %9, ptr noundef @.str.3, ptr noundef @.str.4, ptr noundef @.str.5)
  %10 = load ptr, ptr %2, align 8
  %11 = call ptr @lua_pushstring(ptr noundef %10, ptr noundef @.str.6)
  %12 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %12, i32 noundef -2, ptr noundef @.str.7)
  %13 = load ptr, ptr %2, align 8
  %14 = call i32 @luaL_getsubtable(ptr noundef %13, i32 noundef -1001000, ptr noundef @.str.8)
  %15 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %15, i32 noundef -2, ptr noundef @.str.9)
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 @luaL_getsubtable(ptr noundef %16, i32 noundef -1001000, ptr noundef @.str.10)
  %18 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %18, i32 noundef -2, ptr noundef @.str.11)
  %19 = load ptr, ptr %2, align 8
  %20 = call i32 @lua_rawgeti(ptr noundef %19, i32 noundef -1001000, i64 noundef 2)
  %21 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %21, i32 noundef -2)
  %22 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %22, ptr noundef @ll_funcs, i32 noundef 1)
  %23 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %23, i32 noundef -2)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createclibstable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @luaL_getsubtable(ptr noundef %3, i32 noundef -1001000, ptr noundef @.str.12)
  %5 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %5, i32 noundef 0, i32 noundef 1)
  %6 = load ptr, ptr %2, align 8
  call void @lua_pushcclosure(ptr noundef %6, ptr noundef @gctm, i32 noundef 0)
  %7 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %7, i32 noundef -2, ptr noundef @.str.13)
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_setmetatable(ptr noundef %8, i32 noundef -2)
  ret void
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createsearcherstable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 4, i32 noundef 0)
  store i32 0, ptr %3, align 4
  br label %5

5:                                                ; preds = %22, %1
  %6 = load i32, ptr %3, align 4
  %7 = sext i32 %6 to i64
  %8 = getelementptr inbounds [5 x ptr], ptr @createsearcherstable.searchers, i64 0, i64 %7
  %9 = load ptr, ptr %8, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %25

11:                                               ; preds = %5
  %12 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %12, i32 noundef -2)
  %13 = load ptr, ptr %2, align 8
  %14 = load i32, ptr %3, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds [5 x ptr], ptr @createsearcherstable.searchers, i64 0, i64 %15
  %17 = load ptr, ptr %16, align 8
  call void @lua_pushcclosure(ptr noundef %13, ptr noundef %17, i32 noundef 1)
  %18 = load ptr, ptr %2, align 8
  %19 = load i32, ptr %3, align 4
  %20 = add nsw i32 %19, 1
  %21 = sext i32 %20 to i64
  call void @lua_rawseti(ptr noundef %18, i32 noundef -2, i64 noundef %21)
  br label %22

22:                                               ; preds = %11
  %23 = load i32, ptr %3, align 4
  %24 = add nsw i32 %23, 1
  store i32 %24, ptr %3, align 4
  br label %5, !llvm.loop !6

25:                                               ; preds = %5
  %26 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %26, i32 noundef -2, ptr noundef @.str.16)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @setpath(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i64, align 8
  %13 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %14, ptr noundef @.str.36, ptr noundef %15, ptr noundef @.str.37)
  store ptr %16, ptr %10, align 8
  %17 = load ptr, ptr %10, align 8
  %18 = call ptr @getenv(ptr noundef %17) #4
  store ptr %18, ptr %11, align 8
  %19 = load ptr, ptr %11, align 8
  %20 = icmp eq ptr %19, null
  br i1 %20, label %21, label %24

21:                                               ; preds = %4
  %22 = load ptr, ptr %7, align 8
  %23 = call ptr @getenv(ptr noundef %22) #4
  store ptr %23, ptr %11, align 8
  br label %24

24:                                               ; preds = %21, %4
  %25 = load ptr, ptr %11, align 8
  %26 = icmp eq ptr %25, null
  br i1 %26, label %31, label %27

27:                                               ; preds = %24
  %28 = load ptr, ptr %5, align 8
  %29 = call i32 @noenv(ptr noundef %28)
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %35

31:                                               ; preds = %27, %24
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %8, align 8
  %34 = call ptr @lua_pushstring(ptr noundef %32, ptr noundef %33)
  br label %114

35:                                               ; preds = %27
  %36 = load ptr, ptr %11, align 8
  %37 = call ptr @strstr(ptr noundef %36, ptr noundef @.str.38) #5
  store ptr %37, ptr %9, align 8
  %38 = icmp eq ptr %37, null
  br i1 %38, label %39, label %43

39:                                               ; preds = %35
  %40 = load ptr, ptr %5, align 8
  %41 = load ptr, ptr %11, align 8
  %42 = call ptr @lua_pushstring(ptr noundef %40, ptr noundef %41)
  br label %113

43:                                               ; preds = %35
  %44 = load ptr, ptr %11, align 8
  %45 = call i64 @strlen(ptr noundef %44) #5
  store i64 %45, ptr %12, align 8
  %46 = load ptr, ptr %5, align 8
  call void @luaL_buffinit(ptr noundef %46, ptr noundef %13)
  %47 = load ptr, ptr %11, align 8
  %48 = load ptr, ptr %9, align 8
  %49 = icmp ult ptr %47, %48
  br i1 %49, label %50, label %75

50:                                               ; preds = %43
  %51 = load ptr, ptr %11, align 8
  %52 = load ptr, ptr %9, align 8
  %53 = load ptr, ptr %11, align 8
  %54 = ptrtoint ptr %52 to i64
  %55 = ptrtoint ptr %53 to i64
  %56 = sub i64 %54, %55
  call void @luaL_addlstring(ptr noundef %13, ptr noundef %51, i64 noundef %56)
  %57 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 2
  %58 = load i64, ptr %57, align 8
  %59 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 1
  %60 = load i64, ptr %59, align 8
  %61 = icmp ult i64 %58, %60
  br i1 %61, label %65, label %62

62:                                               ; preds = %50
  %63 = call ptr @luaL_prepbuffsize(ptr noundef %13, i64 noundef 1)
  %64 = icmp ne ptr %63, null
  br label %65

65:                                               ; preds = %62, %50
  %66 = phi i1 [ true, %50 ], [ %64, %62 ]
  %67 = zext i1 %66 to i32
  %68 = load i8, ptr @.str.23, align 1
  %69 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 0
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 2
  %72 = load i64, ptr %71, align 8
  %73 = add i64 %72, 1
  store i64 %73, ptr %71, align 8
  %74 = getelementptr inbounds i8, ptr %70, i64 %72
  store i8 %68, ptr %74, align 1
  br label %75

75:                                               ; preds = %65, %43
  %76 = load ptr, ptr %8, align 8
  call void @luaL_addstring(ptr noundef %13, ptr noundef %76)
  %77 = load ptr, ptr %9, align 8
  %78 = load ptr, ptr %11, align 8
  %79 = load i64, ptr %12, align 8
  %80 = getelementptr inbounds i8, ptr %78, i64 %79
  %81 = getelementptr inbounds i8, ptr %80, i64 -2
  %82 = icmp ult ptr %77, %81
  br i1 %82, label %83, label %112

83:                                               ; preds = %75
  %84 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 2
  %85 = load i64, ptr %84, align 8
  %86 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 1
  %87 = load i64, ptr %86, align 8
  %88 = icmp ult i64 %85, %87
  br i1 %88, label %92, label %89

89:                                               ; preds = %83
  %90 = call ptr @luaL_prepbuffsize(ptr noundef %13, i64 noundef 1)
  %91 = icmp ne ptr %90, null
  br label %92

92:                                               ; preds = %89, %83
  %93 = phi i1 [ true, %83 ], [ %91, %89 ]
  %94 = zext i1 %93 to i32
  %95 = load i8, ptr @.str.23, align 1
  %96 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 0
  %97 = load ptr, ptr %96, align 8
  %98 = getelementptr inbounds %struct.luaL_Buffer, ptr %13, i32 0, i32 2
  %99 = load i64, ptr %98, align 8
  %100 = add i64 %99, 1
  store i64 %100, ptr %98, align 8
  %101 = getelementptr inbounds i8, ptr %97, i64 %99
  store i8 %95, ptr %101, align 1
  %102 = load ptr, ptr %9, align 8
  %103 = getelementptr inbounds i8, ptr %102, i64 2
  %104 = load ptr, ptr %11, align 8
  %105 = load i64, ptr %12, align 8
  %106 = getelementptr inbounds i8, ptr %104, i64 %105
  %107 = getelementptr inbounds i8, ptr %106, i64 -2
  %108 = load ptr, ptr %9, align 8
  %109 = ptrtoint ptr %107 to i64
  %110 = ptrtoint ptr %108 to i64
  %111 = sub i64 %109, %110
  call void @luaL_addlstring(ptr noundef %13, ptr noundef %103, i64 noundef %111)
  br label %112

112:                                              ; preds = %92, %75
  call void @luaL_pushresult(ptr noundef %13)
  br label %113

113:                                              ; preds = %112, %39
  br label %114

114:                                              ; preds = %113, %31
  %115 = load ptr, ptr %5, align 8
  %116 = load ptr, ptr %6, align 8
  call void @lua_setfield(ptr noundef %115, i32 noundef -3, ptr noundef %116)
  %117 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %117, i32 noundef -2)
  ret void
}

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_getsubtable(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_rawgeti(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @gctm(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i64 @luaL_len(ptr noundef %4, i32 noundef 1)
  store i64 %5, ptr %3, align 8
  br label %6

6:                                                ; preds = %16, %1
  %7 = load i64, ptr %3, align 8
  %8 = icmp sge i64 %7, 1
  br i1 %8, label %9, label %19

9:                                                ; preds = %6
  %10 = load ptr, ptr %2, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call i32 @lua_rawgeti(ptr noundef %10, i32 noundef 1, i64 noundef %11)
  %13 = load ptr, ptr %2, align 8
  %14 = call ptr @lua_touserdata(ptr noundef %13, i32 noundef -1)
  call void @lsys_unloadlib(ptr noundef %14)
  %15 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %15, i32 noundef -2)
  br label %16

16:                                               ; preds = %9
  %17 = load i64, ptr %3, align 8
  %18 = add nsw i64 %17, -1
  store i64 %18, ptr %3, align 8
  br label %6, !llvm.loop !8

19:                                               ; preds = %6
  ret i32 0
}

declare i32 @lua_setmetatable(ptr noundef, i32 noundef) #1

declare i64 @luaL_len(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @lsys_unloadlib(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  ret void
}

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @ll_loadlib(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 1, ptr noundef null)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @luaL_checklstring(ptr noundef %9, i32 noundef 2, ptr noundef null)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @lookforfunc(ptr noundef %11, ptr noundef %12, ptr noundef %13)
  store i32 %14, ptr %6, align 4
  %15 = load i32, ptr %6, align 4
  %16 = icmp eq i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = icmp ne i32 %17, 0
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %22, label %23

22:                                               ; preds = %1
  store i32 1, ptr %2, align 4
  br label %32

23:                                               ; preds = %1
  %24 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %24)
  %25 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %25, i32 noundef -2, i32 noundef 1)
  %26 = load ptr, ptr %3, align 8
  %27 = load i32, ptr %6, align 4
  %28 = icmp eq i32 %27, 1
  %29 = zext i1 %28 to i64
  %30 = select i1 %28, ptr @.str.17, ptr @.str.18
  %31 = call ptr @lua_pushstring(ptr noundef %26, ptr noundef %30)
  store i32 3, ptr %2, align 4
  br label %32

32:                                               ; preds = %23, %22
  %33 = load i32, ptr %2, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @ll_searchpath(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @luaL_checklstring(ptr noundef %6, i32 noundef 1, ptr noundef null)
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 2, ptr noundef null)
  %10 = load ptr, ptr %3, align 8
  %11 = call ptr @luaL_optlstring(ptr noundef %10, i32 noundef 3, ptr noundef @.str.20, ptr noundef null)
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @luaL_optlstring(ptr noundef %12, i32 noundef 4, ptr noundef @.str.21, ptr noundef null)
  %14 = call ptr @searchpath(ptr noundef %5, ptr noundef %7, ptr noundef %9, ptr noundef %11, ptr noundef %13)
  store ptr %14, ptr %4, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = icmp ne ptr %15, null
  br i1 %16, label %17, label %18

17:                                               ; preds = %1
  store i32 1, ptr %2, align 4
  br label %21

18:                                               ; preds = %1
  %19 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %19)
  %20 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %20, i32 noundef -2, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %21

21:                                               ; preds = %18, %17
  %22 = load i32, ptr %2, align 4
  ret i32 %22
}

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @lookforfunc(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = call ptr @checkclib(ptr noundef %10, ptr noundef %11)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = icmp eq ptr %13, null
  br i1 %14, label %15, label %31

15:                                               ; preds = %3
  %16 = load ptr, ptr %5, align 8
  %17 = load ptr, ptr %6, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = load i8, ptr %18, align 1
  %20 = sext i8 %19 to i32
  %21 = icmp eq i32 %20, 42
  %22 = zext i1 %21 to i32
  %23 = call ptr @lsys_load(ptr noundef %16, ptr noundef %17, i32 noundef %22)
  store ptr %23, ptr %8, align 8
  %24 = load ptr, ptr %8, align 8
  %25 = icmp eq ptr %24, null
  br i1 %25, label %26, label %27

26:                                               ; preds = %15
  store i32 1, ptr %4, align 4
  br label %49

27:                                               ; preds = %15
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %8, align 8
  call void @addtoclib(ptr noundef %28, ptr noundef %29, ptr noundef %30)
  br label %31

31:                                               ; preds = %27, %3
  %32 = load ptr, ptr %7, align 8
  %33 = load i8, ptr %32, align 1
  %34 = sext i8 %33 to i32
  %35 = icmp eq i32 %34, 42
  br i1 %35, label %36, label %38

36:                                               ; preds = %31
  %37 = load ptr, ptr %5, align 8
  call void @lua_pushboolean(ptr noundef %37, i32 noundef 1)
  store i32 0, ptr %4, align 4
  br label %49

38:                                               ; preds = %31
  %39 = load ptr, ptr %5, align 8
  %40 = load ptr, ptr %8, align 8
  %41 = load ptr, ptr %7, align 8
  %42 = call ptr @lsys_sym(ptr noundef %39, ptr noundef %40, ptr noundef %41)
  store ptr %42, ptr %9, align 8
  %43 = load ptr, ptr %9, align 8
  %44 = icmp eq ptr %43, null
  br i1 %44, label %45, label %46

45:                                               ; preds = %38
  store i32 2, ptr %4, align 4
  br label %49

46:                                               ; preds = %38
  %47 = load ptr, ptr %5, align 8
  %48 = load ptr, ptr %9, align 8
  call void @lua_pushcclosure(ptr noundef %47, ptr noundef %48, i32 noundef 0)
  store i32 0, ptr %4, align 4
  br label %49

49:                                               ; preds = %46, %45, %36, %26
  %50 = load i32, ptr %4, align 4
  ret i32 %50
}

declare void @lua_pushnil(ptr noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @checkclib(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_getfield(ptr noundef %6, i32 noundef -1001000, ptr noundef @.str.12)
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @lua_getfield(ptr noundef %8, i32 noundef -1, ptr noundef %9)
  %11 = load ptr, ptr %3, align 8
  %12 = call ptr @lua_touserdata(ptr noundef %11, i32 noundef -1)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %13, i32 noundef -3)
  %14 = load ptr, ptr %5, align 8
  ret ptr %14
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @lsys_load(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %5, align 8
  %8 = load i32, ptr %6, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @lua_pushstring(ptr noundef %9, ptr noundef @.str.19)
  ret ptr null
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addtoclib(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @lua_getfield(ptr noundef %7, i32 noundef -1001000, ptr noundef @.str.12)
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %6, align 8
  call void @lua_pushlightuserdata(ptr noundef %9, ptr noundef %10)
  %11 = load ptr, ptr %4, align 8
  call void @lua_pushvalue(ptr noundef %11, i32 noundef -1)
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %12, i32 noundef -3, ptr noundef %13)
  %14 = load ptr, ptr %4, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = call i64 @luaL_len(ptr noundef %15, i32 noundef -2)
  %17 = add nsw i64 %16, 1
  call void @lua_rawseti(ptr noundef %14, i32 noundef -2, i64 noundef %17)
  %18 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %18, i32 noundef -2)
  ret void
}

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @lsys_sym(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %5, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @lua_pushstring(ptr noundef %9, ptr noundef @.str.19)
  ret ptr null
}

declare i32 @lua_getfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushlightuserdata(ptr noundef, ptr noundef) #1

declare void @lua_rawseti(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @searchpath(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca %struct.luaL_Buffer, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store ptr %4, ptr %11, align 8
  %16 = load ptr, ptr %10, align 8
  %17 = load i8, ptr %16, align 1
  %18 = sext i8 %17 to i32
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %33

20:                                               ; preds = %5
  %21 = load ptr, ptr %8, align 8
  %22 = load ptr, ptr %10, align 8
  %23 = load i8, ptr %22, align 1
  %24 = sext i8 %23 to i32
  %25 = call ptr @strchr(ptr noundef %21, i32 noundef %24) #5
  %26 = icmp ne ptr %25, null
  br i1 %26, label %27, label %33

27:                                               ; preds = %20
  %28 = load ptr, ptr %7, align 8
  %29 = load ptr, ptr %8, align 8
  %30 = load ptr, ptr %10, align 8
  %31 = load ptr, ptr %11, align 8
  %32 = call ptr @luaL_gsub(ptr noundef %28, ptr noundef %29, ptr noundef %30, ptr noundef %31)
  store ptr %32, ptr %8, align 8
  br label %33

33:                                               ; preds = %27, %20, %5
  %34 = load ptr, ptr %7, align 8
  call void @luaL_buffinit(ptr noundef %34, ptr noundef %12)
  %35 = load ptr, ptr %9, align 8
  %36 = load ptr, ptr %8, align 8
  call void @luaL_addgsub(ptr noundef %12, ptr noundef %35, ptr noundef @.str.22, ptr noundef %36)
  %37 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 2
  %38 = load i64, ptr %37, align 8
  %39 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 1
  %40 = load i64, ptr %39, align 8
  %41 = icmp ult i64 %38, %40
  br i1 %41, label %45, label %42

42:                                               ; preds = %33
  %43 = call ptr @luaL_prepbuffsize(ptr noundef %12, i64 noundef 1)
  %44 = icmp ne ptr %43, null
  br label %45

45:                                               ; preds = %42, %33
  %46 = phi i1 [ true, %33 ], [ %44, %42 ]
  %47 = zext i1 %46 to i32
  %48 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 2
  %51 = load i64, ptr %50, align 8
  %52 = add i64 %51, 1
  store i64 %52, ptr %50, align 8
  %53 = getelementptr inbounds i8, ptr %49, i64 %51
  store i8 0, ptr %53, align 1
  %54 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 0
  %55 = load ptr, ptr %54, align 8
  store ptr %55, ptr %13, align 8
  %56 = load ptr, ptr %13, align 8
  %57 = getelementptr inbounds %struct.luaL_Buffer, ptr %12, i32 0, i32 2
  %58 = load i64, ptr %57, align 8
  %59 = getelementptr inbounds i8, ptr %56, i64 %58
  %60 = getelementptr inbounds i8, ptr %59, i64 -1
  store ptr %60, ptr %14, align 8
  br label %61

61:                                               ; preds = %73, %45
  %62 = load ptr, ptr %14, align 8
  %63 = call ptr @getnextfilename(ptr noundef %13, ptr noundef %62)
  store ptr %63, ptr %15, align 8
  %64 = icmp ne ptr %63, null
  br i1 %64, label %65, label %74

65:                                               ; preds = %61
  %66 = load ptr, ptr %15, align 8
  %67 = call i32 @readable(ptr noundef %66)
  %68 = icmp ne i32 %67, 0
  br i1 %68, label %69, label %73

69:                                               ; preds = %65
  %70 = load ptr, ptr %7, align 8
  %71 = load ptr, ptr %15, align 8
  %72 = call ptr @lua_pushstring(ptr noundef %70, ptr noundef %71)
  store ptr %72, ptr %6, align 8
  br label %78

73:                                               ; preds = %65
  br label %61, !llvm.loop !9

74:                                               ; preds = %61
  call void @luaL_pushresult(ptr noundef %12)
  %75 = load ptr, ptr %7, align 8
  %76 = load ptr, ptr %7, align 8
  %77 = call ptr @lua_tolstring(ptr noundef %76, i32 noundef -1, ptr noundef null)
  call void @pusherrornotfound(ptr noundef %75, ptr noundef %77)
  store ptr null, ptr %6, align 8
  br label %78

78:                                               ; preds = %74, %69
  %79 = load ptr, ptr %6, align 8
  ret ptr %79
}

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #2

declare ptr @luaL_gsub(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

declare void @luaL_addgsub(ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

declare ptr @luaL_prepbuffsize(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getnextfilename(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %7, align 8
  %10 = load ptr, ptr %7, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %10, %11
  br i1 %12, label %13, label %14

13:                                               ; preds = %2
  store ptr null, ptr %3, align 8
  br label %39

14:                                               ; preds = %2
  %15 = load ptr, ptr %7, align 8
  %16 = load i8, ptr %15, align 1
  %17 = sext i8 %16 to i32
  %18 = icmp eq i32 %17, 0
  br i1 %18, label %19, label %24

19:                                               ; preds = %14
  %20 = load i8, ptr @.str.23, align 1
  %21 = load ptr, ptr %7, align 8
  store i8 %20, ptr %21, align 1
  %22 = load ptr, ptr %7, align 8
  %23 = getelementptr inbounds i8, ptr %22, i32 1
  store ptr %23, ptr %7, align 8
  br label %24

24:                                               ; preds = %19, %14
  br label %25

25:                                               ; preds = %24
  %26 = load ptr, ptr %7, align 8
  %27 = load i8, ptr @.str.23, align 1
  %28 = sext i8 %27 to i32
  %29 = call ptr @strchr(ptr noundef %26, i32 noundef %28) #5
  store ptr %29, ptr %6, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = icmp eq ptr %30, null
  br i1 %31, label %32, label %34

32:                                               ; preds = %25
  %33 = load ptr, ptr %5, align 8
  store ptr %33, ptr %6, align 8
  br label %34

34:                                               ; preds = %32, %25
  %35 = load ptr, ptr %6, align 8
  store i8 0, ptr %35, align 1
  %36 = load ptr, ptr %6, align 8
  %37 = load ptr, ptr %4, align 8
  store ptr %36, ptr %37, align 8
  %38 = load ptr, ptr %7, align 8
  store ptr %38, ptr %3, align 8
  br label %39

39:                                               ; preds = %34, %13
  %40 = load ptr, ptr %3, align 8
  ret ptr %40
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @readable(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call noalias ptr @fopen64(ptr noundef %5, ptr noundef @.str.24)
  store ptr %6, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = icmp eq ptr %7, null
  br i1 %8, label %9, label %10

9:                                                ; preds = %1
  store i32 0, ptr %2, align 4
  br label %13

10:                                               ; preds = %1
  %11 = load ptr, ptr %4, align 8
  %12 = call i32 @fclose(ptr noundef %11)
  store i32 1, ptr %2, align 4
  br label %13

13:                                               ; preds = %10, %9
  %14 = load i32, ptr %2, align 4
  ret i32 %14
}

declare void @luaL_pushresult(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pusherrornotfound(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  call void @luaL_buffinit(ptr noundef %6, ptr noundef %5)
  call void @luaL_addstring(ptr noundef %5, ptr noundef @.str.25)
  %7 = load ptr, ptr %4, align 8
  call void @luaL_addgsub(ptr noundef %5, ptr noundef %7, ptr noundef @.str.23, ptr noundef @.str.26)
  call void @luaL_addstring(ptr noundef %5, ptr noundef @.str.27)
  call void @luaL_pushresult(ptr noundef %5)
  ret void
}

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare noalias ptr @fopen64(ptr noundef, ptr noundef) #1

declare i32 @fclose(ptr noundef) #1

declare void @luaL_addstring(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searcher_preload(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call ptr @luaL_checklstring(ptr noundef %5, i32 noundef 1, ptr noundef null)
  store ptr %6, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @lua_getfield(ptr noundef %7, i32 noundef -1001000, ptr noundef @.str.10)
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = call i32 @lua_getfield(ptr noundef %9, i32 noundef -1, ptr noundef %10)
  %12 = icmp eq i32 %11, 0
  br i1 %12, label %13, label %17

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %14, ptr noundef @.str.28, ptr noundef %15)
  store i32 1, ptr %2, align 4
  br label %20

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  %19 = call ptr @lua_pushstring(ptr noundef %18, ptr noundef @.str.29)
  store i32 2, ptr %2, align 4
  br label %20

20:                                               ; preds = %17, %13
  %21 = load i32, ptr %2, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searcher_Lua(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @luaL_checklstring(ptr noundef %6, i32 noundef 1, ptr noundef null)
  store ptr %7, ptr %5, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = call ptr @findfile(ptr noundef %8, ptr noundef %9, ptr noundef @.str, ptr noundef @.str.21)
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %1
  store i32 1, ptr %2, align 4
  br label %23

14:                                               ; preds = %1
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %4, align 8
  %18 = call i32 @luaL_loadfilex(ptr noundef %16, ptr noundef %17, ptr noundef null)
  %19 = icmp eq i32 %18, 0
  %20 = zext i1 %19 to i32
  %21 = load ptr, ptr %4, align 8
  %22 = call i32 @checkload(ptr noundef %15, i32 noundef %20, ptr noundef %21)
  store i32 %22, ptr %2, align 4
  br label %23

23:                                               ; preds = %14, %13
  %24 = load i32, ptr %2, align 4
  ret i32 %24
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searcher_C(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call ptr @luaL_checklstring(ptr noundef %6, i32 noundef 1, ptr noundef null)
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call ptr @findfile(ptr noundef %8, ptr noundef %9, ptr noundef @.str.3, ptr noundef @.str.21)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %1
  store i32 1, ptr %2, align 4
  br label %24

14:                                               ; preds = %1
  %15 = load ptr, ptr %3, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = call i32 @loadfunc(ptr noundef %16, ptr noundef %17, ptr noundef %18)
  %20 = icmp eq i32 %19, 0
  %21 = zext i1 %20 to i32
  %22 = load ptr, ptr %5, align 8
  %23 = call i32 @checkload(ptr noundef %15, i32 noundef %21, ptr noundef %22)
  store i32 %23, ptr %2, align 4
  br label %24

24:                                               ; preds = %14, %13
  %25 = load i32, ptr %2, align 4
  ret i32 %25
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @searcher_Croot(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 1, ptr noundef null)
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call ptr @strchr(ptr noundef %10, i32 noundef 46) #5
  store ptr %11, ptr %6, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = icmp eq ptr %12, null
  br i1 %13, label %14, label %15

14:                                               ; preds = %1
  store i32 0, ptr %2, align 4
  br label %53

15:                                               ; preds = %1
  %16 = load ptr, ptr %3, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = ptrtoint ptr %18 to i64
  %21 = ptrtoint ptr %19 to i64
  %22 = sub i64 %20, %21
  %23 = call ptr @lua_pushlstring(ptr noundef %16, ptr noundef %17, i64 noundef %22)
  %24 = load ptr, ptr %3, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = call ptr @lua_tolstring(ptr noundef %25, i32 noundef -1, ptr noundef null)
  %27 = call ptr @findfile(ptr noundef %24, ptr noundef %26, ptr noundef @.str.3, ptr noundef @.str.21)
  store ptr %27, ptr %4, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = icmp eq ptr %28, null
  br i1 %29, label %30, label %31

30:                                               ; preds = %15
  store i32 1, ptr %2, align 4
  br label %53

31:                                               ; preds = %15
  %32 = load ptr, ptr %3, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = call i32 @loadfunc(ptr noundef %32, ptr noundef %33, ptr noundef %34)
  store i32 %35, ptr %7, align 4
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %37, label %49

37:                                               ; preds = %31
  %38 = load i32, ptr %7, align 4
  %39 = icmp ne i32 %38, 2
  br i1 %39, label %40, label %44

40:                                               ; preds = %37
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = call i32 @checkload(ptr noundef %41, i32 noundef 0, ptr noundef %42)
  store i32 %43, ptr %2, align 4
  br label %53

44:                                               ; preds = %37
  %45 = load ptr, ptr %3, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = load ptr, ptr %4, align 8
  %48 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %45, ptr noundef @.str.35, ptr noundef %46, ptr noundef %47)
  store i32 1, ptr %2, align 4
  br label %53

49:                                               ; preds = %31
  %50 = load ptr, ptr %3, align 8
  %51 = load ptr, ptr %4, align 8
  %52 = call ptr @lua_pushstring(ptr noundef %50, ptr noundef %51)
  store i32 2, ptr %2, align 4
  br label %53

53:                                               ; preds = %49, %44, %40, %30, %14
  %54 = load i32, ptr %2, align 4
  ret i32 %54
}

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @findfile(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = call i32 @lua_getfield(ptr noundef %10, i32 noundef -1001001, ptr noundef %11)
  %13 = load ptr, ptr %5, align 8
  %14 = call ptr @lua_tolstring(ptr noundef %13, i32 noundef -1, ptr noundef null)
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = icmp eq ptr %15, null
  %17 = zext i1 %16 to i32
  %18 = icmp ne i32 %17, 0
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %22, label %26

22:                                               ; preds = %4
  %23 = load ptr, ptr %5, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %23, ptr noundef @.str.30, ptr noundef %24)
  br label %26

26:                                               ; preds = %22, %4
  %27 = load ptr, ptr %5, align 8
  %28 = load ptr, ptr %6, align 8
  %29 = load ptr, ptr %9, align 8
  %30 = load ptr, ptr %8, align 8
  %31 = call ptr @searchpath(ptr noundef %27, ptr noundef %28, ptr noundef %29, ptr noundef @.str.20, ptr noundef %30)
  ret ptr %31
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @checkload(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  %8 = load i32, ptr %6, align 4
  %9 = icmp ne i32 %8, 0
  %10 = zext i1 %9 to i32
  %11 = sext i32 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %13, label %17

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = call ptr @lua_pushstring(ptr noundef %14, ptr noundef %15)
  store i32 2, ptr %4, align 4
  br label %25

17:                                               ; preds = %3
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = call ptr @lua_tolstring(ptr noundef %19, i32 noundef 1, ptr noundef null)
  %21 = load ptr, ptr %7, align 8
  %22 = load ptr, ptr %5, align 8
  %23 = call ptr @lua_tolstring(ptr noundef %22, i32 noundef -1, ptr noundef null)
  %24 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %18, ptr noundef @.str.31, ptr noundef %20, ptr noundef %21, ptr noundef %23)
  store i32 %24, ptr %4, align 4
  br label %25

25:                                               ; preds = %17, %13
  %26 = load i32, ptr %4, align 4
  ret i32 %26
}

declare i32 @luaL_loadfilex(ptr noundef, ptr noundef, ptr noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @loadfunc(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = call ptr @luaL_gsub(ptr noundef %11, ptr noundef %12, ptr noundef @.str.20, ptr noundef @.str.32)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load i8, ptr @.str.33, align 1
  %16 = sext i8 %15 to i32
  %17 = call ptr @strchr(ptr noundef %14, i32 noundef %16) #5
  store ptr %17, ptr %9, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = icmp ne ptr %18, null
  br i1 %19, label %20, label %43

20:                                               ; preds = %3
  %21 = load ptr, ptr %5, align 8
  %22 = load ptr, ptr %7, align 8
  %23 = load ptr, ptr %9, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = ptrtoint ptr %23 to i64
  %26 = ptrtoint ptr %24 to i64
  %27 = sub i64 %25, %26
  %28 = call ptr @lua_pushlstring(ptr noundef %21, ptr noundef %22, i64 noundef %27)
  store ptr %28, ptr %8, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %8, align 8
  %31 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %29, ptr noundef @.str.34, ptr noundef %30)
  store ptr %31, ptr %8, align 8
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %6, align 8
  %34 = load ptr, ptr %8, align 8
  %35 = call i32 @lookforfunc(ptr noundef %32, ptr noundef %33, ptr noundef %34)
  store i32 %35, ptr %10, align 4
  %36 = load i32, ptr %10, align 4
  %37 = icmp ne i32 %36, 2
  br i1 %37, label %38, label %40

38:                                               ; preds = %20
  %39 = load i32, ptr %10, align 4
  store i32 %39, ptr %4, align 4
  br label %51

40:                                               ; preds = %20
  %41 = load ptr, ptr %9, align 8
  %42 = getelementptr inbounds i8, ptr %41, i64 1
  store ptr %42, ptr %7, align 8
  br label %43

43:                                               ; preds = %40, %3
  %44 = load ptr, ptr %5, align 8
  %45 = load ptr, ptr %7, align 8
  %46 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %44, ptr noundef @.str.34, ptr noundef %45)
  store ptr %46, ptr %8, align 8
  %47 = load ptr, ptr %5, align 8
  %48 = load ptr, ptr %6, align 8
  %49 = load ptr, ptr %8, align 8
  %50 = call i32 @lookforfunc(ptr noundef %47, ptr noundef %48, ptr noundef %49)
  store i32 %50, ptr %4, align 4
  br label %51

51:                                               ; preds = %43, %38
  %52 = load i32, ptr %4, align 4
  ret i32 %52
}

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @noenv(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_getfield(ptr noundef %4, i32 noundef -1001000, ptr noundef @.str.39)
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_toboolean(ptr noundef %6, i32 noundef -1)
  store i32 %7, ptr %3, align 4
  %8 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %8, i32 noundef -2)
  %9 = load i32, ptr %3, align 4
  ret i32 %9
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strstr(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #2

declare void @luaL_addlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @ll_require(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call ptr @luaL_checklstring(ptr noundef %5, i32 noundef 1, ptr noundef null)
  store ptr %6, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %7, i32 noundef 1)
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @lua_getfield(ptr noundef %8, i32 noundef -1001000, ptr noundef @.str.8)
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call i32 @lua_getfield(ptr noundef %10, i32 noundef 2, ptr noundef %11)
  %13 = load ptr, ptr %3, align 8
  %14 = call i32 @lua_toboolean(ptr noundef %13, i32 noundef -1)
  %15 = icmp ne i32 %14, 0
  br i1 %15, label %16, label %17

16:                                               ; preds = %1
  store i32 1, ptr %2, align 4
  br label %45

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %18, i32 noundef -2)
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %4, align 8
  call void @findloader(ptr noundef %19, ptr noundef %20)
  %21 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %21, i32 noundef -2, i32 noundef 1)
  %22 = load ptr, ptr %3, align 8
  call void @lua_pushvalue(ptr noundef %22, i32 noundef 1)
  %23 = load ptr, ptr %3, align 8
  call void @lua_pushvalue(ptr noundef %23, i32 noundef -3)
  %24 = load ptr, ptr %3, align 8
  call void @lua_callk(ptr noundef %24, i32 noundef 2, i32 noundef 1, i64 noundef 0, ptr noundef null)
  %25 = load ptr, ptr %3, align 8
  %26 = call i32 @lua_type(ptr noundef %25, i32 noundef -1)
  %27 = icmp eq i32 %26, 0
  br i1 %27, label %31, label %28

28:                                               ; preds = %17
  %29 = load ptr, ptr %3, align 8
  %30 = load ptr, ptr %4, align 8
  call void @lua_setfield(ptr noundef %29, i32 noundef 2, ptr noundef %30)
  br label %33

31:                                               ; preds = %17
  %32 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %32, i32 noundef -2)
  br label %33

33:                                               ; preds = %31, %28
  %34 = load ptr, ptr %3, align 8
  %35 = load ptr, ptr %4, align 8
  %36 = call i32 @lua_getfield(ptr noundef %34, i32 noundef 2, ptr noundef %35)
  %37 = icmp eq i32 %36, 0
  br i1 %37, label %38, label %43

38:                                               ; preds = %33
  %39 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %39, i32 noundef 1)
  %40 = load ptr, ptr %3, align 8
  call void @lua_copy(ptr noundef %40, i32 noundef -1, i32 noundef -2)
  %41 = load ptr, ptr %3, align 8
  %42 = load ptr, ptr %4, align 8
  call void @lua_setfield(ptr noundef %41, i32 noundef 2, ptr noundef %42)
  br label %43

43:                                               ; preds = %38, %33
  %44 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %44, i32 noundef -2, i32 noundef 1)
  store i32 2, ptr %2, align 4
  br label %45

45:                                               ; preds = %43, %16
  %46 = load i32, ptr %2, align 4
  ret i32 %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @findloader(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 @lua_getfield(ptr noundef %7, i32 noundef -1001001, ptr noundef @.str.16)
  %9 = icmp ne i32 %8, 5
  %10 = zext i1 %9 to i32
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %2
  %16 = load ptr, ptr %3, align 8
  %17 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %16, ptr noundef @.str.41)
  br label %18

18:                                               ; preds = %15, %2
  %19 = load ptr, ptr %3, align 8
  call void @luaL_buffinit(ptr noundef %19, ptr noundef %6)
  store i32 1, ptr %5, align 4
  br label %20

20:                                               ; preds = %63, %18
  call void @luaL_addstring(ptr noundef %6, ptr noundef @.str.42)
  %21 = load ptr, ptr %3, align 8
  %22 = load i32, ptr %5, align 4
  %23 = sext i32 %22 to i64
  %24 = call i32 @lua_rawgeti(ptr noundef %21, i32 noundef 3, i64 noundef %23)
  %25 = icmp eq i32 %24, 0
  %26 = zext i1 %25 to i32
  %27 = icmp ne i32 %26, 0
  %28 = zext i1 %27 to i32
  %29 = sext i32 %28 to i64
  %30 = icmp ne i64 %29, 0
  br i1 %30, label %31, label %41

31:                                               ; preds = %20
  %32 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %32, i32 noundef -2)
  %33 = getelementptr inbounds %struct.luaL_Buffer, ptr %6, i32 0, i32 2
  %34 = load i64, ptr %33, align 8
  %35 = sub i64 %34, 2
  store i64 %35, ptr %33, align 8
  call void @luaL_pushresult(ptr noundef %6)
  %36 = load ptr, ptr %3, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = call ptr @lua_tolstring(ptr noundef %38, i32 noundef -1, ptr noundef null)
  %40 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %36, ptr noundef @.str.43, ptr noundef %37, ptr noundef %39)
  br label %41

41:                                               ; preds = %31, %20
  %42 = load ptr, ptr %3, align 8
  %43 = load ptr, ptr %4, align 8
  %44 = call ptr @lua_pushstring(ptr noundef %42, ptr noundef %43)
  %45 = load ptr, ptr %3, align 8
  call void @lua_callk(ptr noundef %45, i32 noundef 1, i32 noundef 2, i64 noundef 0, ptr noundef null)
  %46 = load ptr, ptr %3, align 8
  %47 = call i32 @lua_type(ptr noundef %46, i32 noundef -2)
  %48 = icmp eq i32 %47, 6
  br i1 %48, label %49, label %50

49:                                               ; preds = %41
  ret void

50:                                               ; preds = %41
  %51 = load ptr, ptr %3, align 8
  %52 = call i32 @lua_isstring(ptr noundef %51, i32 noundef -2)
  %53 = icmp ne i32 %52, 0
  br i1 %53, label %54, label %56

54:                                               ; preds = %50
  %55 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %55, i32 noundef -2)
  call void @luaL_addvalue(ptr noundef %6)
  br label %61

56:                                               ; preds = %50
  %57 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %57, i32 noundef -3)
  %58 = getelementptr inbounds %struct.luaL_Buffer, ptr %6, i32 0, i32 2
  %59 = load i64, ptr %58, align 8
  %60 = sub i64 %59, 2
  store i64 %60, ptr %58, align 8
  br label %61

61:                                               ; preds = %56, %54
  br label %62

62:                                               ; preds = %61
  br label %63

63:                                               ; preds = %62
  %64 = load i32, ptr %5, align 4
  %65 = add nsw i32 %64, 1
  store i32 %65, ptr %5, align 4
  br label %20
}

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare void @lua_copy(ptr noundef, i32 noundef, i32 noundef) #1

declare i32 @lua_isstring(ptr noundef, i32 noundef) #1

declare void @luaL_addvalue(ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind }
attributes #5 = { nounwind willreturn memory(read) }

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
