; ModuleID = 'lua.c'
source_filename = "lua.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [39 x i8] c"cannot create state: not enough memory\00", align 1
@stderr = external global ptr, align 8
@.str.1 = private unnamed_addr constant [5 x i8] c"%s: \00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.str.3 = private unnamed_addr constant [10 x i8] c"LUA_NOENV\00", align 1
@progname = internal global ptr @.str.4, align 8
@.str.4 = private unnamed_addr constant [4 x i8] c"lua\00", align 1
@.str.5 = private unnamed_addr constant [21 x i8] c"'%s' needs argument\0A\00", align 1
@.str.6 = private unnamed_addr constant [26 x i8] c"unrecognized option '%s'\0A\00", align 1
@.str.7 = private unnamed_addr constant [449 x i8] c"usage: %s [options] [script [args]]\0AAvailable options are:\0A  -e stat   execute string 'stat'\0A  -i        enter interactive mode after executing 'script'\0A  -l mod    require library 'mod' into global 'mod'\0A  -l g=mod  require library 'mod' into global 'g'\0A  -v        show version information\0A  -E        ignore environment variables\0A  -W        turn warnings on\0A  --        stop handling options\0A  -         stop handling options and execute stdin\0A\00", align 1
@.str.8 = private unnamed_addr constant [52 x i8] c"Lua 5.4.7  Copyright (C) 1994-2024 Lua.org, PUC-Rio\00", align 1
@stdout = external global ptr, align 8
@.str.9 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@.str.10 = private unnamed_addr constant [4 x i8] c"arg\00", align 1
@.str.11 = private unnamed_addr constant [14 x i8] c"=LUA_INIT_5_4\00", align 1
@.str.12 = private unnamed_addr constant [10 x i8] c"=LUA_INIT\00", align 1
@globalL = internal global ptr null, align 8
@.str.13 = private unnamed_addr constant [11 x i8] c"__tostring\00", align 1
@.str.14 = private unnamed_addr constant [29 x i8] c"(error object is a %s value)\00", align 1
@.str.15 = private unnamed_addr constant [13 x i8] c"interrupted!\00", align 1
@.str.16 = private unnamed_addr constant [16 x i8] c"=(command line)\00", align 1
@.str.17 = private unnamed_addr constant [4 x i8] c"@on\00", align 1
@.str.18 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@.str.19 = private unnamed_addr constant [8 x i8] c"require\00", align 1
@.str.20 = private unnamed_addr constant [3 x i8] c"--\00", align 1
@.str.21 = private unnamed_addr constant [21 x i8] c"'arg' is not a table\00", align 1
@.str.22 = private unnamed_addr constant [29 x i8] c"too many arguments to script\00", align 1
@stdin = external global ptr, align 8
@.str.23 = private unnamed_addr constant [10 x i8] c"return %s\00", align 1
@.str.24 = private unnamed_addr constant [8 x i8] c"_PROMPT\00", align 1
@.str.25 = private unnamed_addr constant [9 x i8] c"_PROMPT2\00", align 1
@.str.26 = private unnamed_addr constant [3 x i8] c"> \00", align 1
@.str.27 = private unnamed_addr constant [4 x i8] c">> \00", align 1
@.str.28 = private unnamed_addr constant [11 x i8] c"return %s;\00", align 1
@.str.29 = private unnamed_addr constant [7 x i8] c"=stdin\00", align 1
@.str.30 = private unnamed_addr constant [6 x i8] c"<eof>\00", align 1
@.str.31 = private unnamed_addr constant [26 x i8] c"too many results to print\00", align 1
@.str.32 = private unnamed_addr constant [6 x i8] c"print\00", align 1
@.str.33 = private unnamed_addr constant [27 x i8] c"error calling 'print' (%s)\00", align 1
@.str.34 = private unnamed_addr constant [29 x i8] c"(error message not a string)\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  store ptr %1, ptr %5, align 8
  %9 = call ptr @luaL_newstate()
  store ptr %9, ptr %8, align 8
  %10 = load ptr, ptr %8, align 8
  %11 = icmp eq ptr %10, null
  br i1 %11, label %12, label %16

12:                                               ; preds = %2
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds ptr, ptr %13, i64 0
  %15 = load ptr, ptr %14, align 8
  call void @l_message(ptr noundef %15, ptr noundef @.str)
  store i32 1, ptr %3, align 4
  br label %42

16:                                               ; preds = %2
  %17 = load ptr, ptr %8, align 8
  %18 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %17, i32 noundef 0)
  %19 = load ptr, ptr %8, align 8
  call void @lua_pushcclosure(ptr noundef %19, ptr noundef @pmain, i32 noundef 0)
  %20 = load ptr, ptr %8, align 8
  %21 = load i32, ptr %4, align 4
  %22 = sext i32 %21 to i64
  call void @lua_pushinteger(ptr noundef %20, i64 noundef %22)
  %23 = load ptr, ptr %8, align 8
  %24 = load ptr, ptr %5, align 8
  call void @lua_pushlightuserdata(ptr noundef %23, ptr noundef %24)
  %25 = load ptr, ptr %8, align 8
  %26 = call i32 @lua_pcallk(ptr noundef %25, i32 noundef 2, i32 noundef 1, i32 noundef 0, i64 noundef 0, ptr noundef null)
  store i32 %26, ptr %6, align 4
  %27 = load ptr, ptr %8, align 8
  %28 = call i32 @lua_toboolean(ptr noundef %27, i32 noundef -1)
  store i32 %28, ptr %7, align 4
  %29 = load ptr, ptr %8, align 8
  %30 = load i32, ptr %6, align 4
  %31 = call i32 @report(ptr noundef %29, i32 noundef %30)
  %32 = load ptr, ptr %8, align 8
  call void @lua_close(ptr noundef %32)
  %33 = load i32, ptr %7, align 4
  %34 = icmp ne i32 %33, 0
  br i1 %34, label %35, label %38

35:                                               ; preds = %16
  %36 = load i32, ptr %6, align 4
  %37 = icmp eq i32 %36, 0
  br label %38

38:                                               ; preds = %35, %16
  %39 = phi i1 [ false, %16 ], [ %37, %35 ]
  %40 = zext i1 %39 to i64
  %41 = select i1 %39, i32 0, i32 1
  store i32 %41, ptr %3, align 4
  br label %42

42:                                               ; preds = %38, %12
  %43 = load i32, ptr %3, align 4
  ret i32 %43
}

declare ptr @luaL_newstate() #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @l_message(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = icmp ne ptr %5, null
  br i1 %6, label %7, label %13

7:                                                ; preds = %2
  %8 = load ptr, ptr @stderr, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %8, ptr noundef @.str.1, ptr noundef %9)
  %11 = load ptr, ptr @stderr, align 8
  %12 = call i32 @fflush(ptr noundef %11)
  br label %13

13:                                               ; preds = %7, %2
  %14 = load ptr, ptr @stderr, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %14, ptr noundef @.str.2, ptr noundef %15)
  %17 = load ptr, ptr @stderr, align 8
  %18 = call i32 @fflush(ptr noundef %17)
  ret void
}

declare i32 @lua_gc(ptr noundef, i32 noundef, ...) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pmain(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call i64 @lua_tointegerx(ptr noundef %9, i32 noundef 1, ptr noundef null)
  %11 = trunc i64 %10 to i32
  store i32 %11, ptr %4, align 4
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @lua_touserdata(ptr noundef %12, i32 noundef 2)
  store ptr %13, ptr %5, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = call i32 @collectargs(ptr noundef %14, ptr noundef %6)
  store i32 %15, ptr %7, align 4
  %16 = load i32, ptr %6, align 4
  %17 = icmp sgt i32 %16, 0
  br i1 %17, label %18, label %20

18:                                               ; preds = %1
  %19 = load i32, ptr %6, align 4
  br label %22

20:                                               ; preds = %1
  %21 = load i32, ptr %4, align 4
  br label %22

22:                                               ; preds = %20, %18
  %23 = phi i32 [ %19, %18 ], [ %21, %20 ]
  store i32 %23, ptr %8, align 4
  %24 = load ptr, ptr %3, align 8
  call void @luaL_checkversion_(ptr noundef %24, double noundef 5.040000e+02, i64 noundef 136)
  %25 = load i32, ptr %7, align 4
  %26 = icmp eq i32 %25, 1
  br i1 %26, label %27, label %33

27:                                               ; preds = %22
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %6, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds ptr, ptr %28, i64 %30
  %32 = load ptr, ptr %31, align 8
  call void @print_usage(ptr noundef %32)
  store i32 0, ptr %2, align 4
  br label %102

33:                                               ; preds = %22
  %34 = load i32, ptr %7, align 4
  %35 = and i32 %34, 4
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %37, label %38

37:                                               ; preds = %33
  call void @print_version()
  br label %38

38:                                               ; preds = %37, %33
  %39 = load i32, ptr %7, align 4
  %40 = and i32 %39, 16
  %41 = icmp ne i32 %40, 0
  br i1 %41, label %42, label %45

42:                                               ; preds = %38
  %43 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %43, i32 noundef 1)
  %44 = load ptr, ptr %3, align 8
  call void @lua_setfield(ptr noundef %44, i32 noundef -1001000, ptr noundef @.str.3)
  br label %45

45:                                               ; preds = %42, %38
  %46 = load ptr, ptr %3, align 8
  call void @luaL_openlibs(ptr noundef %46)
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = load i32, ptr %4, align 4
  %50 = load i32, ptr %6, align 4
  call void @createargtable(ptr noundef %47, ptr noundef %48, i32 noundef %49, i32 noundef %50)
  %51 = load ptr, ptr %3, align 8
  %52 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %51, i32 noundef 1)
  %53 = load ptr, ptr %3, align 8
  %54 = call i32 (ptr, i32, ...) @lua_gc(ptr noundef %53, i32 noundef 10, i32 noundef 0, i32 noundef 0)
  %55 = load i32, ptr %7, align 4
  %56 = and i32 %55, 16
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %64, label %58

58:                                               ; preds = %45
  %59 = load ptr, ptr %3, align 8
  %60 = call i32 @handle_luainit(ptr noundef %59)
  %61 = icmp ne i32 %60, 0
  br i1 %61, label %62, label %63

62:                                               ; preds = %58
  store i32 0, ptr %2, align 4
  br label %102

63:                                               ; preds = %58
  br label %64

64:                                               ; preds = %63, %45
  %65 = load ptr, ptr %3, align 8
  %66 = load ptr, ptr %5, align 8
  %67 = load i32, ptr %8, align 4
  %68 = call i32 @runargs(ptr noundef %65, ptr noundef %66, i32 noundef %67)
  %69 = icmp ne i32 %68, 0
  br i1 %69, label %71, label %70

70:                                               ; preds = %64
  store i32 0, ptr %2, align 4
  br label %102

71:                                               ; preds = %64
  %72 = load i32, ptr %6, align 4
  %73 = icmp sgt i32 %72, 0
  br i1 %73, label %74, label %84

74:                                               ; preds = %71
  %75 = load ptr, ptr %3, align 8
  %76 = load ptr, ptr %5, align 8
  %77 = load i32, ptr %6, align 4
  %78 = sext i32 %77 to i64
  %79 = getelementptr inbounds ptr, ptr %76, i64 %78
  %80 = call i32 @handle_script(ptr noundef %75, ptr noundef %79)
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %83

82:                                               ; preds = %74
  store i32 0, ptr %2, align 4
  br label %102

83:                                               ; preds = %74
  br label %84

84:                                               ; preds = %83, %71
  %85 = load i32, ptr %7, align 4
  %86 = and i32 %85, 2
  %87 = icmp ne i32 %86, 0
  br i1 %87, label %88, label %90

88:                                               ; preds = %84
  %89 = load ptr, ptr %3, align 8
  call void @doREPL(ptr noundef %89)
  br label %100

90:                                               ; preds = %84
  %91 = load i32, ptr %6, align 4
  %92 = icmp slt i32 %91, 1
  br i1 %92, label %93, label %99

93:                                               ; preds = %90
  %94 = load i32, ptr %7, align 4
  %95 = and i32 %94, 12
  %96 = icmp ne i32 %95, 0
  br i1 %96, label %99, label %97

97:                                               ; preds = %93
  call void @print_version()
  %98 = load ptr, ptr %3, align 8
  call void @doREPL(ptr noundef %98)
  br label %99

99:                                               ; preds = %97, %93, %90
  br label %100

100:                                              ; preds = %99, %88
  %101 = load ptr, ptr %3, align 8
  call void @lua_pushboolean(ptr noundef %101, i32 noundef 1)
  store i32 1, ptr %2, align 4
  br label %102

102:                                              ; preds = %100, %82, %70, %62, %27
  %103 = load i32, ptr %2, align 4
  ret i32 %103
}

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_pushlightuserdata(ptr noundef, ptr noundef) #1

declare i32 @lua_pcallk(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @report(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load i32, ptr %4, align 4
  %7 = icmp ne i32 %6, 0
  br i1 %7, label %8, label %18

8:                                                ; preds = %2
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @lua_tolstring(ptr noundef %9, i32 noundef -1, ptr noundef null)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = icmp eq ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %8
  store ptr @.str.34, ptr %5, align 8
  br label %14

14:                                               ; preds = %13, %8
  %15 = load ptr, ptr @progname, align 8
  %16 = load ptr, ptr %5, align 8
  call void @l_message(ptr noundef %15, ptr noundef %16)
  %17 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %17, i32 noundef -2)
  br label %18

18:                                               ; preds = %14, %2
  %19 = load i32, ptr %4, align 4
  ret i32 %19
}

declare void @lua_close(ptr noundef) #1

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #1

declare i32 @fflush(ptr noundef) #1

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @collectargs(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 0, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds ptr, ptr %8, i64 0
  %10 = load ptr, ptr %9, align 8
  %11 = icmp ne ptr %10, null
  br i1 %11, label %12, label %24

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds ptr, ptr %13, i64 0
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds i8, ptr %15, i64 0
  %17 = load i8, ptr %16, align 1
  %18 = icmp ne i8 %17, 0
  br i1 %18, label %19, label %23

19:                                               ; preds = %12
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds ptr, ptr %20, i64 0
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr @progname, align 8
  br label %23

23:                                               ; preds = %19, %12
  br label %26

24:                                               ; preds = %2
  %25 = load ptr, ptr %5, align 8
  store i32 -1, ptr %25, align 4
  store i32 0, ptr %3, align 4
  br label %161

26:                                               ; preds = %23
  store i32 1, ptr %7, align 4
  br label %27

27:                                               ; preds = %155, %26
  %28 = load ptr, ptr %4, align 8
  %29 = load i32, ptr %7, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds ptr, ptr %28, i64 %30
  %32 = load ptr, ptr %31, align 8
  %33 = icmp ne ptr %32, null
  br i1 %33, label %34, label %158

34:                                               ; preds = %27
  %35 = load i32, ptr %7, align 4
  %36 = load ptr, ptr %5, align 8
  store i32 %35, ptr %36, align 4
  %37 = load ptr, ptr %4, align 8
  %38 = load i32, ptr %7, align 4
  %39 = sext i32 %38 to i64
  %40 = getelementptr inbounds ptr, ptr %37, i64 %39
  %41 = load ptr, ptr %40, align 8
  %42 = getelementptr inbounds i8, ptr %41, i64 0
  %43 = load i8, ptr %42, align 1
  %44 = sext i8 %43 to i32
  %45 = icmp ne i32 %44, 45
  br i1 %45, label %46, label %48

46:                                               ; preds = %34
  %47 = load i32, ptr %6, align 4
  store i32 %47, ptr %3, align 4
  br label %161

48:                                               ; preds = %34
  %49 = load ptr, ptr %4, align 8
  %50 = load i32, ptr %7, align 4
  %51 = sext i32 %50 to i64
  %52 = getelementptr inbounds ptr, ptr %49, i64 %51
  %53 = load ptr, ptr %52, align 8
  %54 = getelementptr inbounds i8, ptr %53, i64 1
  %55 = load i8, ptr %54, align 1
  %56 = sext i8 %55 to i32
  switch i32 %56, label %153 [
    i32 45, label %57
    i32 0, label %73
    i32 69, label %75
    i32 87, label %89
    i32 105, label %101
    i32 118, label %104
    i32 101, label %118
    i32 108, label %121
  ]

57:                                               ; preds = %48
  %58 = load ptr, ptr %4, align 8
  %59 = load i32, ptr %7, align 4
  %60 = sext i32 %59 to i64
  %61 = getelementptr inbounds ptr, ptr %58, i64 %60
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds i8, ptr %62, i64 2
  %64 = load i8, ptr %63, align 1
  %65 = sext i8 %64 to i32
  %66 = icmp ne i32 %65, 0
  br i1 %66, label %67, label %68

67:                                               ; preds = %57
  store i32 1, ptr %3, align 4
  br label %161

68:                                               ; preds = %57
  %69 = load i32, ptr %7, align 4
  %70 = add nsw i32 %69, 1
  %71 = load ptr, ptr %5, align 8
  store i32 %70, ptr %71, align 4
  %72 = load i32, ptr %6, align 4
  store i32 %72, ptr %3, align 4
  br label %161

73:                                               ; preds = %48
  %74 = load i32, ptr %6, align 4
  store i32 %74, ptr %3, align 4
  br label %161

75:                                               ; preds = %48
  %76 = load ptr, ptr %4, align 8
  %77 = load i32, ptr %7, align 4
  %78 = sext i32 %77 to i64
  %79 = getelementptr inbounds ptr, ptr %76, i64 %78
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds i8, ptr %80, i64 2
  %82 = load i8, ptr %81, align 1
  %83 = sext i8 %82 to i32
  %84 = icmp ne i32 %83, 0
  br i1 %84, label %85, label %86

85:                                               ; preds = %75
  store i32 1, ptr %3, align 4
  br label %161

86:                                               ; preds = %75
  %87 = load i32, ptr %6, align 4
  %88 = or i32 %87, 16
  store i32 %88, ptr %6, align 4
  br label %154

89:                                               ; preds = %48
  %90 = load ptr, ptr %4, align 8
  %91 = load i32, ptr %7, align 4
  %92 = sext i32 %91 to i64
  %93 = getelementptr inbounds ptr, ptr %90, i64 %92
  %94 = load ptr, ptr %93, align 8
  %95 = getelementptr inbounds i8, ptr %94, i64 2
  %96 = load i8, ptr %95, align 1
  %97 = sext i8 %96 to i32
  %98 = icmp ne i32 %97, 0
  br i1 %98, label %99, label %100

99:                                               ; preds = %89
  store i32 1, ptr %3, align 4
  br label %161

100:                                              ; preds = %89
  br label %154

101:                                              ; preds = %48
  %102 = load i32, ptr %6, align 4
  %103 = or i32 %102, 2
  store i32 %103, ptr %6, align 4
  br label %104

104:                                              ; preds = %48, %101
  %105 = load ptr, ptr %4, align 8
  %106 = load i32, ptr %7, align 4
  %107 = sext i32 %106 to i64
  %108 = getelementptr inbounds ptr, ptr %105, i64 %107
  %109 = load ptr, ptr %108, align 8
  %110 = getelementptr inbounds i8, ptr %109, i64 2
  %111 = load i8, ptr %110, align 1
  %112 = sext i8 %111 to i32
  %113 = icmp ne i32 %112, 0
  br i1 %113, label %114, label %115

114:                                              ; preds = %104
  store i32 1, ptr %3, align 4
  br label %161

115:                                              ; preds = %104
  %116 = load i32, ptr %6, align 4
  %117 = or i32 %116, 4
  store i32 %117, ptr %6, align 4
  br label %154

118:                                              ; preds = %48
  %119 = load i32, ptr %6, align 4
  %120 = or i32 %119, 8
  store i32 %120, ptr %6, align 4
  br label %121

121:                                              ; preds = %48, %118
  %122 = load ptr, ptr %4, align 8
  %123 = load i32, ptr %7, align 4
  %124 = sext i32 %123 to i64
  %125 = getelementptr inbounds ptr, ptr %122, i64 %124
  %126 = load ptr, ptr %125, align 8
  %127 = getelementptr inbounds i8, ptr %126, i64 2
  %128 = load i8, ptr %127, align 1
  %129 = sext i8 %128 to i32
  %130 = icmp eq i32 %129, 0
  br i1 %130, label %131, label %152

131:                                              ; preds = %121
  %132 = load i32, ptr %7, align 4
  %133 = add nsw i32 %132, 1
  store i32 %133, ptr %7, align 4
  %134 = load ptr, ptr %4, align 8
  %135 = load i32, ptr %7, align 4
  %136 = sext i32 %135 to i64
  %137 = getelementptr inbounds ptr, ptr %134, i64 %136
  %138 = load ptr, ptr %137, align 8
  %139 = icmp eq ptr %138, null
  br i1 %139, label %150, label %140

140:                                              ; preds = %131
  %141 = load ptr, ptr %4, align 8
  %142 = load i32, ptr %7, align 4
  %143 = sext i32 %142 to i64
  %144 = getelementptr inbounds ptr, ptr %141, i64 %143
  %145 = load ptr, ptr %144, align 8
  %146 = getelementptr inbounds i8, ptr %145, i64 0
  %147 = load i8, ptr %146, align 1
  %148 = sext i8 %147 to i32
  %149 = icmp eq i32 %148, 45
  br i1 %149, label %150, label %151

150:                                              ; preds = %140, %131
  store i32 1, ptr %3, align 4
  br label %161

151:                                              ; preds = %140
  br label %152

152:                                              ; preds = %151, %121
  br label %154

153:                                              ; preds = %48
  store i32 1, ptr %3, align 4
  br label %161

154:                                              ; preds = %152, %115, %100, %86
  br label %155

155:                                              ; preds = %154
  %156 = load i32, ptr %7, align 4
  %157 = add nsw i32 %156, 1
  store i32 %157, ptr %7, align 4
  br label %27, !llvm.loop !6

158:                                              ; preds = %27
  %159 = load ptr, ptr %5, align 8
  store i32 0, ptr %159, align 4
  %160 = load i32, ptr %6, align 4
  store i32 %160, ptr %3, align 4
  br label %161

161:                                              ; preds = %158, %153, %150, %114, %99, %85, %73, %68, %67, %46, %24
  %162 = load i32, ptr %3, align 4
  ret i32 %162
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @print_usage(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr @stderr, align 8
  %4 = load ptr, ptr @progname, align 8
  %5 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %3, ptr noundef @.str.1, ptr noundef %4)
  %6 = load ptr, ptr @stderr, align 8
  %7 = call i32 @fflush(ptr noundef %6)
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds i8, ptr %8, i64 1
  %10 = load i8, ptr %9, align 1
  %11 = sext i8 %10 to i32
  %12 = icmp eq i32 %11, 101
  br i1 %12, label %19, label %13

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds i8, ptr %14, i64 1
  %16 = load i8, ptr %15, align 1
  %17 = sext i8 %16 to i32
  %18 = icmp eq i32 %17, 108
  br i1 %18, label %19, label %25

19:                                               ; preds = %13, %1
  %20 = load ptr, ptr @stderr, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %20, ptr noundef @.str.5, ptr noundef %21)
  %23 = load ptr, ptr @stderr, align 8
  %24 = call i32 @fflush(ptr noundef %23)
  br label %31

25:                                               ; preds = %13
  %26 = load ptr, ptr @stderr, align 8
  %27 = load ptr, ptr %2, align 8
  %28 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %26, ptr noundef @.str.6, ptr noundef %27)
  %29 = load ptr, ptr @stderr, align 8
  %30 = call i32 @fflush(ptr noundef %29)
  br label %31

31:                                               ; preds = %25, %19
  %32 = load ptr, ptr @stderr, align 8
  %33 = load ptr, ptr @progname, align 8
  %34 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %32, ptr noundef @.str.7, ptr noundef %33)
  %35 = load ptr, ptr @stderr, align 8
  %36 = call i32 @fflush(ptr noundef %35)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @print_version() #0 {
  %1 = load ptr, ptr @stdout, align 8
  %2 = call i64 @fwrite(ptr noundef @.str.8, i64 noundef 1, i64 noundef 51, ptr noundef %1)
  %3 = load ptr, ptr @stdout, align 8
  %4 = call i64 @fwrite(ptr noundef @.str.9, i64 noundef 1, i64 noundef 1, ptr noundef %3)
  %5 = load ptr, ptr @stdout, align 8
  %6 = call i32 @fflush(ptr noundef %5)
  ret void
}

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_openlibs(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createargtable(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %11 = load i32, ptr %7, align 4
  %12 = load i32, ptr %8, align 4
  %13 = add nsw i32 %12, 1
  %14 = sub nsw i32 %11, %13
  store i32 %14, ptr %10, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = load i32, ptr %10, align 4
  %17 = load i32, ptr %8, align 4
  %18 = add nsw i32 %17, 1
  call void @lua_createtable(ptr noundef %15, i32 noundef %16, i32 noundef %18)
  store i32 0, ptr %9, align 4
  br label %19

19:                                               ; preds = %36, %4
  %20 = load i32, ptr %9, align 4
  %21 = load i32, ptr %7, align 4
  %22 = icmp slt i32 %20, %21
  br i1 %22, label %23, label %39

23:                                               ; preds = %19
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %6, align 8
  %26 = load i32, ptr %9, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds ptr, ptr %25, i64 %27
  %29 = load ptr, ptr %28, align 8
  %30 = call ptr @lua_pushstring(ptr noundef %24, ptr noundef %29)
  %31 = load ptr, ptr %5, align 8
  %32 = load i32, ptr %9, align 4
  %33 = load i32, ptr %8, align 4
  %34 = sub nsw i32 %32, %33
  %35 = sext i32 %34 to i64
  call void @lua_rawseti(ptr noundef %31, i32 noundef -2, i64 noundef %35)
  br label %36

36:                                               ; preds = %23
  %37 = load i32, ptr %9, align 4
  %38 = add nsw i32 %37, 1
  store i32 %38, ptr %9, align 4
  br label %19, !llvm.loop !8

39:                                               ; preds = %19
  %40 = load ptr, ptr %5, align 8
  call void @lua_setglobal(ptr noundef %40, ptr noundef @.str.10)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @handle_luainit(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr @.str.11, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds i8, ptr %6, i64 1
  %8 = call ptr @getenv(ptr noundef %7) #4
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = icmp eq ptr %9, null
  br i1 %10, label %11, label %15

11:                                               ; preds = %1
  store ptr @.str.12, ptr %4, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds i8, ptr %12, i64 1
  %14 = call ptr @getenv(ptr noundef %13) #4
  store ptr %14, ptr %5, align 8
  br label %15

15:                                               ; preds = %11, %1
  %16 = load ptr, ptr %5, align 8
  %17 = icmp eq ptr %16, null
  br i1 %17, label %18, label %19

18:                                               ; preds = %15
  store i32 0, ptr %2, align 4
  br label %35

19:                                               ; preds = %15
  %20 = load ptr, ptr %5, align 8
  %21 = getelementptr inbounds i8, ptr %20, i64 0
  %22 = load i8, ptr %21, align 1
  %23 = sext i8 %22 to i32
  %24 = icmp eq i32 %23, 64
  br i1 %24, label %25, label %30

25:                                               ; preds = %19
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds i8, ptr %27, i64 1
  %29 = call i32 @dofile(ptr noundef %26, ptr noundef %28)
  store i32 %29, ptr %2, align 4
  br label %35

30:                                               ; preds = %19
  %31 = load ptr, ptr %3, align 8
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = call i32 @dostring(ptr noundef %31, ptr noundef %32, ptr noundef %33)
  store i32 %34, ptr %2, align 4
  br label %35

35:                                               ; preds = %30, %25, %18
  %36 = load i32, ptr %2, align 4
  ret i32 %36
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @runargs(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 1, ptr %8, align 4
  br label %12

12:                                               ; preds = %64, %3
  %13 = load i32, ptr %8, align 4
  %14 = load i32, ptr %7, align 4
  %15 = icmp slt i32 %13, %14
  br i1 %15, label %16, label %67

16:                                               ; preds = %12
  %17 = load ptr, ptr %6, align 8
  %18 = load i32, ptr %8, align 4
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds ptr, ptr %17, i64 %19
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 1
  %23 = load i8, ptr %22, align 1
  %24 = sext i8 %23 to i32
  store i32 %24, ptr %9, align 4
  %25 = load i32, ptr %9, align 4
  switch i32 %25, label %63 [
    i32 101, label %26
    i32 108, label %26
    i32 87, label %61
  ]

26:                                               ; preds = %16, %16
  %27 = load ptr, ptr %6, align 8
  %28 = load i32, ptr %8, align 4
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds ptr, ptr %27, i64 %29
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds i8, ptr %31, i64 2
  store ptr %32, ptr %11, align 8
  %33 = load ptr, ptr %11, align 8
  %34 = load i8, ptr %33, align 1
  %35 = sext i8 %34 to i32
  %36 = icmp eq i32 %35, 0
  br i1 %36, label %37, label %44

37:                                               ; preds = %26
  %38 = load ptr, ptr %6, align 8
  %39 = load i32, ptr %8, align 4
  %40 = add nsw i32 %39, 1
  store i32 %40, ptr %8, align 4
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds ptr, ptr %38, i64 %41
  %43 = load ptr, ptr %42, align 8
  store ptr %43, ptr %11, align 8
  br label %44

44:                                               ; preds = %37, %26
  %45 = load i32, ptr %9, align 4
  %46 = icmp eq i32 %45, 101
  br i1 %46, label %47, label %51

47:                                               ; preds = %44
  %48 = load ptr, ptr %5, align 8
  %49 = load ptr, ptr %11, align 8
  %50 = call i32 @dostring(ptr noundef %48, ptr noundef %49, ptr noundef @.str.16)
  br label %55

51:                                               ; preds = %44
  %52 = load ptr, ptr %5, align 8
  %53 = load ptr, ptr %11, align 8
  %54 = call i32 @dolibrary(ptr noundef %52, ptr noundef %53)
  br label %55

55:                                               ; preds = %51, %47
  %56 = phi i32 [ %50, %47 ], [ %54, %51 ]
  store i32 %56, ptr %10, align 4
  %57 = load i32, ptr %10, align 4
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %60

59:                                               ; preds = %55
  store i32 0, ptr %4, align 4
  br label %68

60:                                               ; preds = %55
  br label %63

61:                                               ; preds = %16
  %62 = load ptr, ptr %5, align 8
  call void @lua_warning(ptr noundef %62, ptr noundef @.str.17, i32 noundef 0)
  br label %63

63:                                               ; preds = %16, %61, %60
  br label %64

64:                                               ; preds = %63
  %65 = load i32, ptr %8, align 4
  %66 = add nsw i32 %65, 1
  store i32 %66, ptr %8, align 4
  br label %12, !llvm.loop !9

67:                                               ; preds = %12
  store i32 1, ptr %4, align 4
  br label %68

68:                                               ; preds = %67, %59
  %69 = load i32, ptr %4, align 4
  ret i32 %69
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @handle_script(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds ptr, ptr %8, i64 0
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %6, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = call i32 @strcmp(ptr noundef %11, ptr noundef @.str.18) #5
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %14, label %21

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds ptr, ptr %15, i64 -1
  %17 = load ptr, ptr %16, align 8
  %18 = call i32 @strcmp(ptr noundef %17, ptr noundef @.str.20) #5
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %21

20:                                               ; preds = %14
  store ptr null, ptr %6, align 8
  br label %21

21:                                               ; preds = %20, %14, %2
  %22 = load ptr, ptr %3, align 8
  %23 = load ptr, ptr %6, align 8
  %24 = call i32 @luaL_loadfilex(ptr noundef %22, ptr noundef %23, ptr noundef null)
  store i32 %24, ptr %5, align 4
  %25 = load i32, ptr %5, align 4
  %26 = icmp eq i32 %25, 0
  br i1 %26, label %27, label %33

27:                                               ; preds = %21
  %28 = load ptr, ptr %3, align 8
  %29 = call i32 @pushargs(ptr noundef %28)
  store i32 %29, ptr %7, align 4
  %30 = load ptr, ptr %3, align 8
  %31 = load i32, ptr %7, align 4
  %32 = call i32 @docall(ptr noundef %30, i32 noundef %31, i32 noundef -1)
  store i32 %32, ptr %5, align 4
  br label %33

33:                                               ; preds = %27, %21
  %34 = load ptr, ptr %3, align 8
  %35 = load i32, ptr %5, align 4
  %36 = call i32 @report(ptr noundef %34, i32 noundef %35)
  ret i32 %36
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @doREPL(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr @progname, align 8
  store ptr %5, ptr %4, align 8
  store ptr null, ptr @progname, align 8
  %6 = load ptr, ptr %2, align 8
  br label %7

7:                                                ; preds = %26, %1
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @loadline(ptr noundef %8)
  store i32 %9, ptr %3, align 4
  %10 = icmp ne i32 %9, -1
  br i1 %10, label %11, label %27

11:                                               ; preds = %7
  %12 = load i32, ptr %3, align 4
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %14, label %17

14:                                               ; preds = %11
  %15 = load ptr, ptr %2, align 8
  %16 = call i32 @docall(ptr noundef %15, i32 noundef 0, i32 noundef -1)
  store i32 %16, ptr %3, align 4
  br label %17

17:                                               ; preds = %14, %11
  %18 = load i32, ptr %3, align 4
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %22

20:                                               ; preds = %17
  %21 = load ptr, ptr %2, align 8
  call void @l_print(ptr noundef %21)
  br label %26

22:                                               ; preds = %17
  %23 = load ptr, ptr %2, align 8
  %24 = load i32, ptr %3, align 4
  %25 = call i32 @report(ptr noundef %23, i32 noundef %24)
  br label %26

26:                                               ; preds = %22, %20
  br label %7, !llvm.loop !10

27:                                               ; preds = %7
  %28 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %28, i32 noundef 0)
  %29 = load ptr, ptr @stdout, align 8
  %30 = call i64 @fwrite(ptr noundef @.str.9, i64 noundef 1, i64 noundef 1, ptr noundef %29)
  %31 = load ptr, ptr @stdout, align 8
  %32 = call i32 @fflush(ptr noundef %31)
  %33 = load ptr, ptr %4, align 8
  store ptr %33, ptr @progname, align 8
  ret void
}

declare i64 @fwrite(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare void @lua_rawseti(ptr noundef, i32 noundef, i64 noundef) #1

declare void @lua_setglobal(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @dofile(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @luaL_loadfilex(ptr noundef %6, ptr noundef %7, ptr noundef null)
  %9 = call i32 @dochunk(ptr noundef %5, i32 noundef %8)
  ret i32 %9
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @dostring(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call i64 @strlen(ptr noundef %10) #5
  %12 = load ptr, ptr %6, align 8
  %13 = call i32 @luaL_loadbufferx(ptr noundef %8, ptr noundef %9, i64 noundef %11, ptr noundef %12, ptr noundef null)
  %14 = call i32 @dochunk(ptr noundef %7, i32 noundef %13)
  ret i32 %14
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @dochunk(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = icmp eq i32 %5, 0
  br i1 %6, label %7, label %10

7:                                                ; preds = %2
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @docall(ptr noundef %8, i32 noundef 0, i32 noundef 0)
  store i32 %9, ptr %4, align 4
  br label %10

10:                                               ; preds = %7, %2
  %11 = load ptr, ptr %3, align 8
  %12 = load i32, ptr %4, align 4
  %13 = call i32 @report(ptr noundef %11, i32 noundef %12)
  ret i32 %13
}

declare i32 @luaL_loadfilex(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @docall(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @lua_gettop(ptr noundef %9)
  %11 = load i32, ptr %5, align 4
  %12 = sub nsw i32 %10, %11
  store i32 %12, ptr %8, align 4
  %13 = load ptr, ptr %4, align 8
  call void @lua_pushcclosure(ptr noundef %13, ptr noundef @msghandler, i32 noundef 0)
  %14 = load ptr, ptr %4, align 8
  %15 = load i32, ptr %8, align 4
  call void @lua_rotate(ptr noundef %14, i32 noundef %15, i32 noundef 1)
  %16 = load ptr, ptr %4, align 8
  store ptr %16, ptr @globalL, align 8
  %17 = call ptr @__sysv_signal(i32 noundef 2, ptr noundef @laction) #4
  %18 = load ptr, ptr %4, align 8
  %19 = load i32, ptr %5, align 4
  %20 = load i32, ptr %6, align 4
  %21 = load i32, ptr %8, align 4
  %22 = call i32 @lua_pcallk(ptr noundef %18, i32 noundef %19, i32 noundef %20, i32 noundef %21, i64 noundef 0, ptr noundef null)
  store i32 %22, ptr %7, align 4
  %23 = call ptr @__sysv_signal(i32 noundef 2, ptr noundef null) #4
  %24 = load ptr, ptr %4, align 8
  %25 = load i32, ptr %8, align 4
  call void @lua_rotate(ptr noundef %24, i32 noundef %25, i32 noundef -1)
  %26 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %26, i32 noundef -2)
  %27 = load i32, ptr %7, align 4
  ret i32 %27
}

declare i32 @lua_gettop(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @msghandler(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = call ptr @lua_tolstring(ptr noundef %5, i32 noundef 1, ptr noundef null)
  store ptr %6, ptr %4, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = icmp eq ptr %7, null
  br i1 %8, label %9, label %26

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @luaL_callmeta(ptr noundef %10, i32 noundef 1, ptr noundef @.str.13)
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %13, label %18

13:                                               ; preds = %9
  %14 = load ptr, ptr %3, align 8
  %15 = call i32 @lua_type(ptr noundef %14, i32 noundef -1)
  %16 = icmp eq i32 %15, 4
  br i1 %16, label %17, label %18

17:                                               ; preds = %13
  store i32 1, ptr %2, align 4
  br label %30

18:                                               ; preds = %13, %9
  %19 = load ptr, ptr %3, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = call i32 @lua_type(ptr noundef %21, i32 noundef 1)
  %23 = call ptr @lua_typename(ptr noundef %20, i32 noundef %22)
  %24 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %19, ptr noundef @.str.14, ptr noundef %23)
  store ptr %24, ptr %4, align 8
  br label %25

25:                                               ; preds = %18
  br label %26

26:                                               ; preds = %25, %1
  %27 = load ptr, ptr %3, align 8
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %4, align 8
  call void @luaL_traceback(ptr noundef %27, ptr noundef %28, ptr noundef %29, i32 noundef 1)
  store i32 1, ptr %2, align 4
  br label %30

30:                                               ; preds = %26, %17
  %31 = load i32, ptr %2, align 4
  ret i32 %31
}

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: nounwind
declare ptr @__sysv_signal(i32 noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @laction(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  store i32 15, ptr %3, align 4
  %4 = load i32, ptr %2, align 4
  %5 = call ptr @__sysv_signal(i32 noundef %4, ptr noundef null) #4
  %6 = load ptr, ptr @globalL, align 8
  %7 = load i32, ptr %3, align 4
  call void @lua_sethook(ptr noundef %6, ptr noundef @lstop, i32 noundef %7, i32 noundef 1)
  ret void
}

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_callmeta(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

declare ptr @lua_typename(ptr noundef, i32 noundef) #1

declare void @luaL_traceback(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

declare void @lua_sethook(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @lstop(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  call void @lua_sethook(ptr noundef %6, ptr noundef null, i32 noundef 0, i32 noundef 0)
  %7 = load ptr, ptr %3, align 8
  %8 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %7, ptr noundef @.str.15)
  ret void
}

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare i32 @luaL_loadbufferx(ptr noundef, ptr noundef, i64 noundef, ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @dolibrary(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr null, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call ptr @strchr(ptr noundef %8, i32 noundef 61) #5
  store ptr %9, ptr %7, align 8
  %10 = load ptr, ptr %7, align 8
  %11 = icmp eq ptr %10, null
  br i1 %11, label %12, label %18

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load i8, ptr @.str.18, align 1
  %16 = sext i8 %15 to i32
  %17 = call ptr @strchr(ptr noundef %14, i32 noundef %16) #5
  store ptr %17, ptr %6, align 8
  br label %22

18:                                               ; preds = %2
  %19 = load ptr, ptr %7, align 8
  store i8 0, ptr %19, align 1
  %20 = load ptr, ptr %7, align 8
  %21 = getelementptr inbounds i8, ptr %20, i32 1
  store ptr %21, ptr %7, align 8
  br label %22

22:                                               ; preds = %18, %12
  %23 = load ptr, ptr %3, align 8
  %24 = call i32 @lua_getglobal(ptr noundef %23, ptr noundef @.str.19)
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %7, align 8
  %27 = call ptr @lua_pushstring(ptr noundef %25, ptr noundef %26)
  %28 = load ptr, ptr %3, align 8
  %29 = call i32 @docall(ptr noundef %28, i32 noundef 1, i32 noundef 1)
  store i32 %29, ptr %5, align 4
  %30 = load i32, ptr %5, align 4
  %31 = icmp eq i32 %30, 0
  br i1 %31, label %32, label %40

32:                                               ; preds = %22
  %33 = load ptr, ptr %6, align 8
  %34 = icmp ne ptr %33, null
  br i1 %34, label %35, label %37

35:                                               ; preds = %32
  %36 = load ptr, ptr %6, align 8
  store i8 0, ptr %36, align 1
  br label %37

37:                                               ; preds = %35, %32
  %38 = load ptr, ptr %3, align 8
  %39 = load ptr, ptr %4, align 8
  call void @lua_setglobal(ptr noundef %38, ptr noundef %39)
  br label %40

40:                                               ; preds = %37, %22
  %41 = load ptr, ptr %3, align 8
  %42 = load i32, ptr %5, align 4
  %43 = call i32 @report(ptr noundef %41, i32 noundef %42)
  ret i32 %43
}

declare void @lua_warning(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #3

declare i32 @lua_getglobal(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pushargs(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_getglobal(ptr noundef %5, ptr noundef @.str.10)
  %7 = icmp ne i32 %6, 5
  br i1 %7, label %8, label %11

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  %10 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %9, ptr noundef @.str.21)
  br label %11

11:                                               ; preds = %8, %1
  %12 = load ptr, ptr %2, align 8
  %13 = call i64 @luaL_len(ptr noundef %12, i32 noundef -1)
  %14 = trunc i64 %13 to i32
  store i32 %14, ptr %4, align 4
  %15 = load ptr, ptr %2, align 8
  %16 = load i32, ptr %4, align 4
  %17 = add nsw i32 %16, 3
  call void @luaL_checkstack(ptr noundef %15, i32 noundef %17, ptr noundef @.str.22)
  store i32 1, ptr %3, align 4
  br label %18

18:                                               ; preds = %29, %11
  %19 = load i32, ptr %3, align 4
  %20 = load i32, ptr %4, align 4
  %21 = icmp sle i32 %19, %20
  br i1 %21, label %22, label %32

22:                                               ; preds = %18
  %23 = load ptr, ptr %2, align 8
  %24 = load i32, ptr %3, align 4
  %25 = sub nsw i32 0, %24
  %26 = load i32, ptr %3, align 4
  %27 = sext i32 %26 to i64
  %28 = call i32 @lua_rawgeti(ptr noundef %23, i32 noundef %25, i64 noundef %27)
  br label %29

29:                                               ; preds = %22
  %30 = load i32, ptr %3, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, ptr %3, align 4
  br label %18, !llvm.loop !11

32:                                               ; preds = %18
  %33 = load ptr, ptr %2, align 8
  %34 = load i32, ptr %3, align 4
  %35 = sub nsw i32 0, %34
  call void @lua_rotate(ptr noundef %33, i32 noundef %35, i32 noundef -1)
  %36 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %36, i32 noundef -2)
  %37 = load i32, ptr %4, align 4
  ret i32 %37
}

declare i64 @luaL_len(ptr noundef, i32 noundef) #1

declare void @luaL_checkstack(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @lua_rawgeti(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @loadline(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %5, i32 noundef 0)
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @pushline(ptr noundef %6, i32 noundef 1)
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %10, label %9

9:                                                ; preds = %1
  store i32 -1, ptr %2, align 4
  br label %21

10:                                               ; preds = %1
  %11 = load ptr, ptr %3, align 8
  %12 = call i32 @addreturn(ptr noundef %11)
  store i32 %12, ptr %4, align 4
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %17

14:                                               ; preds = %10
  %15 = load ptr, ptr %3, align 8
  %16 = call i32 @multiline(ptr noundef %15)
  store i32 %16, ptr %4, align 4
  br label %17

17:                                               ; preds = %14, %10
  %18 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %18, i32 noundef 1, i32 noundef -1)
  %19 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %19, i32 noundef -2)
  %20 = load i32, ptr %4, align 4
  store i32 %20, ptr %2, align 4
  br label %21

21:                                               ; preds = %17, %9
  %22 = load i32, ptr %2, align 4
  ret i32 %22
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @l_print(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @lua_gettop(ptr noundef %4)
  store i32 %5, ptr %3, align 4
  %6 = load i32, ptr %3, align 4
  %7 = icmp sgt i32 %6, 0
  br i1 %7, label %8, label %24

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  call void @luaL_checkstack(ptr noundef %9, i32 noundef 20, ptr noundef @.str.31)
  %10 = load ptr, ptr %2, align 8
  %11 = call i32 @lua_getglobal(ptr noundef %10, ptr noundef @.str.32)
  %12 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %12, i32 noundef 1, i32 noundef 1)
  %13 = load ptr, ptr %2, align 8
  %14 = load i32, ptr %3, align 4
  %15 = call i32 @lua_pcallk(ptr noundef %13, i32 noundef %14, i32 noundef 0, i32 noundef 0, i64 noundef 0, ptr noundef null)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %23

17:                                               ; preds = %8
  %18 = load ptr, ptr @progname, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = call ptr @lua_tolstring(ptr noundef %20, i32 noundef -1, ptr noundef null)
  %22 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %19, ptr noundef @.str.33, ptr noundef %21)
  call void @l_message(ptr noundef %18, ptr noundef %22)
  br label %23

23:                                               ; preds = %17, %8
  br label %24

24:                                               ; preds = %23, %1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pushline(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca [512 x i8], align 16
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %11 = getelementptr inbounds [512 x i8], ptr %6, i64 0, i64 0
  store ptr %11, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load i32, ptr %5, align 4
  %14 = call ptr @get_prompt(ptr noundef %12, i32 noundef %13)
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %9, align 8
  %17 = load ptr, ptr @stdout, align 8
  %18 = call i32 @fputs(ptr noundef %16, ptr noundef %17)
  %19 = load ptr, ptr @stdout, align 8
  %20 = call i32 @fflush(ptr noundef %19)
  %21 = load ptr, ptr %7, align 8
  %22 = load ptr, ptr @stdin, align 8
  %23 = call ptr @fgets(ptr noundef %21, i32 noundef 512, ptr noundef %22)
  %24 = icmp ne ptr %23, null
  %25 = zext i1 %24 to i32
  store i32 %25, ptr %10, align 4
  %26 = load i32, ptr %10, align 4
  %27 = icmp eq i32 %26, 0
  br i1 %27, label %28, label %29

28:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %70

29:                                               ; preds = %2
  %30 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %30, i32 noundef -2)
  %31 = load ptr, ptr %7, align 8
  %32 = call i64 @strlen(ptr noundef %31) #5
  store i64 %32, ptr %8, align 8
  %33 = load i64, ptr %8, align 8
  %34 = icmp ugt i64 %33, 0
  br i1 %34, label %35, label %48

35:                                               ; preds = %29
  %36 = load ptr, ptr %7, align 8
  %37 = load i64, ptr %8, align 8
  %38 = sub i64 %37, 1
  %39 = getelementptr inbounds i8, ptr %36, i64 %38
  %40 = load i8, ptr %39, align 1
  %41 = sext i8 %40 to i32
  %42 = icmp eq i32 %41, 10
  br i1 %42, label %43, label %48

43:                                               ; preds = %35
  %44 = load ptr, ptr %7, align 8
  %45 = load i64, ptr %8, align 8
  %46 = add i64 %45, -1
  store i64 %46, ptr %8, align 8
  %47 = getelementptr inbounds i8, ptr %44, i64 %46
  store i8 0, ptr %47, align 1
  br label %48

48:                                               ; preds = %43, %35, %29
  %49 = load i32, ptr %5, align 4
  %50 = icmp ne i32 %49, 0
  br i1 %50, label %51, label %62

51:                                               ; preds = %48
  %52 = load ptr, ptr %7, align 8
  %53 = getelementptr inbounds i8, ptr %52, i64 0
  %54 = load i8, ptr %53, align 1
  %55 = sext i8 %54 to i32
  %56 = icmp eq i32 %55, 61
  br i1 %56, label %57, label %62

57:                                               ; preds = %51
  %58 = load ptr, ptr %4, align 8
  %59 = load ptr, ptr %7, align 8
  %60 = getelementptr inbounds i8, ptr %59, i64 1
  %61 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %58, ptr noundef @.str.23, ptr noundef %60)
  br label %67

62:                                               ; preds = %51, %48
  %63 = load ptr, ptr %4, align 8
  %64 = load ptr, ptr %7, align 8
  %65 = load i64, ptr %8, align 8
  %66 = call ptr @lua_pushlstring(ptr noundef %63, ptr noundef %64, i64 noundef %65)
  br label %67

67:                                               ; preds = %62, %57
  %68 = load ptr, ptr %4, align 8
  %69 = load ptr, ptr %7, align 8
  store i32 1, ptr %3, align 4
  br label %70

70:                                               ; preds = %67, %28
  %71 = load i32, ptr %3, align 4
  ret i32 %71
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @addreturn(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call ptr @lua_tolstring(ptr noundef %6, i32 noundef -1, ptr noundef null)
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %8, ptr noundef @.str.28, ptr noundef %9)
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = call i64 @strlen(ptr noundef %13) #5
  %15 = call i32 @luaL_loadbufferx(ptr noundef %11, ptr noundef %12, i64 noundef %14, ptr noundef @.str.29, ptr noundef null)
  store i32 %15, ptr %5, align 4
  %16 = load i32, ptr %5, align 4
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %30

18:                                               ; preds = %1
  %19 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %19, i32 noundef -2, i32 noundef -1)
  %20 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %20, i32 noundef -2)
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 0
  %23 = load i8, ptr %22, align 1
  %24 = sext i8 %23 to i32
  %25 = icmp ne i32 %24, 0
  br i1 %25, label %26, label %29

26:                                               ; preds = %18
  %27 = load ptr, ptr %2, align 8
  %28 = load ptr, ptr %3, align 8
  br label %29

29:                                               ; preds = %26, %18
  br label %32

30:                                               ; preds = %1
  %31 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %31, i32 noundef -3)
  br label %32

32:                                               ; preds = %30, %29
  %33 = load i32, ptr %5, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @multiline(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  br label %6

6:                                                ; preds = %25, %1
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @lua_tolstring(ptr noundef %7, i32 noundef 1, ptr noundef %3)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call i32 @luaL_loadbufferx(ptr noundef %9, ptr noundef %10, i64 noundef %11, ptr noundef @.str.29, ptr noundef null)
  store i32 %12, ptr %5, align 4
  %13 = load ptr, ptr %2, align 8
  %14 = load i32, ptr %5, align 4
  %15 = call i32 @incomplete(ptr noundef %13, i32 noundef %14)
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %21

17:                                               ; preds = %6
  %18 = load ptr, ptr %2, align 8
  %19 = call i32 @pushline(ptr noundef %18, i32 noundef 0)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %25, label %21

21:                                               ; preds = %17, %6
  %22 = load ptr, ptr %2, align 8
  %23 = load ptr, ptr %4, align 8
  %24 = load i32, ptr %5, align 4
  ret i32 %24

25:                                               ; preds = %17
  %26 = load ptr, ptr %2, align 8
  %27 = call ptr @lua_pushstring(ptr noundef %26, ptr noundef @.str.9)
  %28 = load ptr, ptr %2, align 8
  call void @lua_rotate(ptr noundef %28, i32 noundef -2, i32 noundef 1)
  %29 = load ptr, ptr %2, align 8
  call void @lua_concat(ptr noundef %29, i32 noundef 3)
  br label %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @get_prompt(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load i32, ptr %5, align 4
  %9 = icmp ne i32 %8, 0
  %10 = zext i1 %9 to i64
  %11 = select i1 %9, ptr @.str.24, ptr @.str.25
  %12 = call i32 @lua_getglobal(ptr noundef %7, ptr noundef %11)
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %14, label %19

14:                                               ; preds = %2
  %15 = load i32, ptr %5, align 4
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i64
  %18 = select i1 %16, ptr @.str.26, ptr @.str.27
  store ptr %18, ptr %3, align 8
  br label %25

19:                                               ; preds = %2
  %20 = load ptr, ptr %4, align 8
  %21 = call ptr @luaL_tolstring(ptr noundef %20, i32 noundef -1, ptr noundef null)
  store ptr %21, ptr %6, align 8
  %22 = load ptr, ptr %4, align 8
  call void @lua_rotate(ptr noundef %22, i32 noundef -2, i32 noundef -1)
  %23 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %23, i32 noundef -2)
  %24 = load ptr, ptr %6, align 8
  store ptr %24, ptr %3, align 8
  br label %25

25:                                               ; preds = %19, %14
  %26 = load ptr, ptr %3, align 8
  ret ptr %26
}

declare i32 @fputs(ptr noundef, ptr noundef) #1

declare ptr @fgets(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare ptr @luaL_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @incomplete(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load i32, ptr %5, align 4
  %9 = icmp eq i32 %8, 3
  br i1 %9, label %10, label %25

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @lua_tolstring(ptr noundef %11, i32 noundef -1, ptr noundef %6)
  store ptr %12, ptr %7, align 8
  %13 = load i64, ptr %6, align 8
  %14 = icmp uge i64 %13, 5
  br i1 %14, label %15, label %24

15:                                               ; preds = %10
  %16 = load ptr, ptr %7, align 8
  %17 = load i64, ptr %6, align 8
  %18 = getelementptr inbounds i8, ptr %16, i64 %17
  %19 = getelementptr inbounds i8, ptr %18, i64 -5
  %20 = call i32 @strcmp(ptr noundef %19, ptr noundef @.str.30) #5
  %21 = icmp eq i32 %20, 0
  br i1 %21, label %22, label %24

22:                                               ; preds = %15
  %23 = load ptr, ptr %4, align 8
  call void @lua_settop(ptr noundef %23, i32 noundef -2)
  store i32 1, ptr %3, align 4
  br label %26

24:                                               ; preds = %15, %10
  br label %25

25:                                               ; preds = %24, %2
  store i32 0, ptr %3, align 4
  br label %26

26:                                               ; preds = %25, %22
  %27 = load i32, ptr %3, align 4
  ret i32 %27
}

declare void @lua_concat(ptr noundef, i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
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
!10 = distinct !{!10, !7}
!11 = distinct !{!11, !7}
