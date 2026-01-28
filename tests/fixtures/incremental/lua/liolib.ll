; ModuleID = 'liolib.c'
source_filename = "liolib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.luaL_Stream = type { ptr, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }
%struct.RN = type { ptr, i32, i32, [201 x i8] }
%struct.lconv = type { ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8 }

@iolib = internal constant [12 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.5, ptr @io_close }, %struct.luaL_Reg { ptr @.str.6, ptr @io_flush }, %struct.luaL_Reg { ptr @.str.7, ptr @io_input }, %struct.luaL_Reg { ptr @.str.8, ptr @io_lines }, %struct.luaL_Reg { ptr @.str.9, ptr @io_open }, %struct.luaL_Reg { ptr @.str.10, ptr @io_output }, %struct.luaL_Reg { ptr @.str.11, ptr @io_popen }, %struct.luaL_Reg { ptr @.str.12, ptr @io_read }, %struct.luaL_Reg { ptr @.str.13, ptr @io_tmpfile }, %struct.luaL_Reg { ptr @.str.14, ptr @io_type }, %struct.luaL_Reg { ptr @.str.15, ptr @io_write }, %struct.luaL_Reg zeroinitializer], align 16
@stdin = external global ptr, align 8
@.str = private unnamed_addr constant [10 x i8] c"_IO_input\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"stdin\00", align 1
@stdout = external global ptr, align 8
@.str.2 = private unnamed_addr constant [11 x i8] c"_IO_output\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"stdout\00", align 1
@stderr = external global ptr, align 8
@.str.4 = private unnamed_addr constant [7 x i8] c"stderr\00", align 1
@.str.5 = private unnamed_addr constant [6 x i8] c"close\00", align 1
@.str.6 = private unnamed_addr constant [6 x i8] c"flush\00", align 1
@.str.7 = private unnamed_addr constant [6 x i8] c"input\00", align 1
@.str.8 = private unnamed_addr constant [6 x i8] c"lines\00", align 1
@.str.9 = private unnamed_addr constant [5 x i8] c"open\00", align 1
@.str.10 = private unnamed_addr constant [7 x i8] c"output\00", align 1
@.str.11 = private unnamed_addr constant [6 x i8] c"popen\00", align 1
@.str.12 = private unnamed_addr constant [5 x i8] c"read\00", align 1
@.str.13 = private unnamed_addr constant [8 x i8] c"tmpfile\00", align 1
@.str.14 = private unnamed_addr constant [5 x i8] c"type\00", align 1
@.str.15 = private unnamed_addr constant [6 x i8] c"write\00", align 1
@.str.16 = private unnamed_addr constant [6 x i8] c"FILE*\00", align 1
@.str.17 = private unnamed_addr constant [29 x i8] c"attempt to use a closed file\00", align 1
@.str.18 = private unnamed_addr constant [26 x i8] c"default %s file is closed\00", align 1
@.str.19 = private unnamed_addr constant [2 x i8] c"r\00", align 1
@.str.20 = private unnamed_addr constant [27 x i8] c"cannot open file '%s' (%s)\00", align 1
@.str.21 = private unnamed_addr constant [19 x i8] c"too many arguments\00", align 1
@.str.22 = private unnamed_addr constant [23 x i8] c"file is already closed\00", align 1
@.str.23 = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.str.24 = private unnamed_addr constant [15 x i8] c"invalid format\00", align 1
@.str.25 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.26 = private unnamed_addr constant [3 x i8] c"-+\00", align 1
@.str.27 = private unnamed_addr constant [3 x i8] c"00\00", align 1
@.str.28 = private unnamed_addr constant [3 x i8] c"xX\00", align 1
@.str.29 = private unnamed_addr constant [3 x i8] c"pP\00", align 1
@.str.30 = private unnamed_addr constant [3 x i8] c"eE\00", align 1
@.str.31 = private unnamed_addr constant [13 x i8] c"invalid mode\00", align 1
@.str.32 = private unnamed_addr constant [4 x i8] c"rwa\00", align 1
@.str.33 = private unnamed_addr constant [2 x i8] c"b\00", align 1
@.str.34 = private unnamed_addr constant [2 x i8] c"w\00", align 1
@.str.35 = private unnamed_addr constant [22 x i8] c"'popen' not supported\00", align 1
@.str.36 = private unnamed_addr constant [12 x i8] c"closed file\00", align 1
@.str.37 = private unnamed_addr constant [5 x i8] c"file\00", align 1
@.str.38 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.str.39 = private unnamed_addr constant [6 x i8] c"%.14g\00", align 1
@metameth = internal constant [5 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.40, ptr null }, %struct.luaL_Reg { ptr @.str.41, ptr @f_gc }, %struct.luaL_Reg { ptr @.str.42, ptr @f_gc }, %struct.luaL_Reg { ptr @.str.43, ptr @f_tostring }, %struct.luaL_Reg zeroinitializer], align 16
@meth = internal constant [8 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.12, ptr @f_read }, %struct.luaL_Reg { ptr @.str.15, ptr @f_write }, %struct.luaL_Reg { ptr @.str.8, ptr @f_lines }, %struct.luaL_Reg { ptr @.str.6, ptr @f_flush }, %struct.luaL_Reg { ptr @.str.46, ptr @f_seek }, %struct.luaL_Reg { ptr @.str.5, ptr @f_close }, %struct.luaL_Reg { ptr @.str.47, ptr @f_setvbuf }, %struct.luaL_Reg zeroinitializer], align 16
@.str.40 = private unnamed_addr constant [8 x i8] c"__index\00", align 1
@.str.41 = private unnamed_addr constant [5 x i8] c"__gc\00", align 1
@.str.42 = private unnamed_addr constant [8 x i8] c"__close\00", align 1
@.str.43 = private unnamed_addr constant [11 x i8] c"__tostring\00", align 1
@.str.44 = private unnamed_addr constant [14 x i8] c"file (closed)\00", align 1
@.str.45 = private unnamed_addr constant [10 x i8] c"file (%p)\00", align 1
@.str.46 = private unnamed_addr constant [5 x i8] c"seek\00", align 1
@.str.47 = private unnamed_addr constant [8 x i8] c"setvbuf\00", align 1
@f_seek.mode = internal constant [3 x i32] [i32 0, i32 1, i32 2], align 4
@f_seek.modenames = internal constant [4 x ptr] [ptr @.str.48, ptr @.str.49, ptr @.str.50, ptr null], align 16
@.str.48 = private unnamed_addr constant [4 x i8] c"set\00", align 1
@.str.49 = private unnamed_addr constant [4 x i8] c"cur\00", align 1
@.str.50 = private unnamed_addr constant [4 x i8] c"end\00", align 1
@.str.51 = private unnamed_addr constant [31 x i8] c"not an integer in proper range\00", align 1
@f_setvbuf.mode = internal constant [3 x i32] [i32 2, i32 0, i32 1], align 4
@f_setvbuf.modenames = internal constant [4 x ptr] [ptr @.str.52, ptr @.str.53, ptr @.str.54, ptr null], align 16
@.str.52 = private unnamed_addr constant [3 x i8] c"no\00", align 1
@.str.53 = private unnamed_addr constant [5 x i8] c"full\00", align 1
@.str.54 = private unnamed_addr constant [5 x i8] c"line\00", align 1
@.str.55 = private unnamed_addr constant [27 x i8] c"cannot close standard file\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_io(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 11)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @iolib, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @createmeta(ptr noundef %6)
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr @stdin, align 8
  call void @createstdfile(ptr noundef %7, ptr noundef %8, ptr noundef @.str, ptr noundef @.str.1)
  %9 = load ptr, ptr %2, align 8
  %10 = load ptr, ptr @stdout, align 8
  call void @createstdfile(ptr noundef %9, ptr noundef %10, ptr noundef @.str.2, ptr noundef @.str.3)
  %11 = load ptr, ptr %2, align 8
  %12 = load ptr, ptr @stderr, align 8
  call void @createstdfile(ptr noundef %11, ptr noundef %12, ptr noundef null, ptr noundef @.str.4)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createmeta(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @luaL_newmetatable(ptr noundef %3, ptr noundef @.str.16)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @metameth, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %6, i32 noundef 0, i32 noundef 7)
  %7 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %7, ptr noundef @meth, i32 noundef 0)
  %8 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %8, i32 noundef -2, ptr noundef @.str.40)
  %9 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %9, i32 noundef -2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createstdfile(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
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
  %11 = call ptr @newprefile(ptr noundef %10)
  store ptr %11, ptr %9, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = load ptr, ptr %9, align 8
  %14 = getelementptr inbounds %struct.luaL_Stream, ptr %13, i32 0, i32 0
  store ptr %12, ptr %14, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = getelementptr inbounds %struct.luaL_Stream, ptr %15, i32 0, i32 1
  store ptr @io_noclose, ptr %16, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = icmp ne ptr %17, null
  br i1 %18, label %19, label %23

19:                                               ; preds = %4
  %20 = load ptr, ptr %5, align 8
  call void @lua_pushvalue(ptr noundef %20, i32 noundef -1)
  %21 = load ptr, ptr %5, align 8
  %22 = load ptr, ptr %7, align 8
  call void @lua_setfield(ptr noundef %21, i32 noundef -1001000, ptr noundef %22)
  br label %23

23:                                               ; preds = %19, %4
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %8, align 8
  call void @lua_setfield(ptr noundef %24, i32 noundef -2, ptr noundef %25)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_close(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @lua_type(ptr noundef %3, i32 noundef 1)
  %5 = icmp eq i32 %4, -1
  br i1 %5, label %6, label %9

6:                                                ; preds = %1
  %7 = load ptr, ptr %2, align 8
  %8 = call i32 @lua_getfield(ptr noundef %7, i32 noundef -1001000, ptr noundef @.str.2)
  br label %9

9:                                                ; preds = %6, %1
  %10 = load ptr, ptr %2, align 8
  %11 = call i32 @f_close(ptr noundef %10)
  ret i32 %11
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_flush(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @getiofile(ptr noundef %4, ptr noundef @.str.2)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #5
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @fflush(ptr noundef %8)
  %10 = icmp eq i32 %9, 0
  %11 = zext i1 %10 to i32
  %12 = call i32 @luaL_fileresult(ptr noundef %7, i32 noundef %11, ptr noundef null)
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_input(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @g_iofile(ptr noundef %3, ptr noundef @.str, ptr noundef @.str.19)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_lines(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_type(ptr noundef %6, i32 noundef 1)
  %8 = icmp eq i32 %7, -1
  br i1 %8, label %9, label %11

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %10)
  br label %11

11:                                               ; preds = %9, %1
  %12 = load ptr, ptr %3, align 8
  %13 = call i32 @lua_type(ptr noundef %12, i32 noundef 1)
  %14 = icmp eq i32 %13, 0
  br i1 %14, label %15, label %22

15:                                               ; preds = %11
  %16 = load ptr, ptr %3, align 8
  %17 = call i32 @lua_getfield(ptr noundef %16, i32 noundef -1001000, ptr noundef @.str)
  %18 = load ptr, ptr %3, align 8
  call void @lua_copy(ptr noundef %18, i32 noundef -1, i32 noundef 1)
  %19 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %19, i32 noundef -2)
  %20 = load ptr, ptr %3, align 8
  %21 = call ptr @tofile(ptr noundef %20)
  store i32 0, ptr %4, align 4
  br label %29

22:                                               ; preds = %11
  %23 = load ptr, ptr %3, align 8
  %24 = call ptr @luaL_checklstring(ptr noundef %23, i32 noundef 1, ptr noundef null)
  store ptr %24, ptr %5, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = load ptr, ptr %5, align 8
  call void @opencheck(ptr noundef %25, ptr noundef %26, ptr noundef @.str.19)
  %27 = load ptr, ptr %3, align 8
  call void @lua_copy(ptr noundef %27, i32 noundef -1, i32 noundef 1)
  %28 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %28, i32 noundef -2)
  store i32 1, ptr %4, align 4
  br label %29

29:                                               ; preds = %22, %15
  %30 = load ptr, ptr %3, align 8
  %31 = load i32, ptr %4, align 4
  call void @aux_lines(ptr noundef %30, i32 noundef %31)
  %32 = load i32, ptr %4, align 4
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %34, label %38

34:                                               ; preds = %29
  %35 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %35)
  %36 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %36)
  %37 = load ptr, ptr %3, align 8
  call void @lua_pushvalue(ptr noundef %37, i32 noundef 1)
  store i32 4, ptr %2, align 4
  br label %39

38:                                               ; preds = %29
  store i32 1, ptr %2, align 4
  br label %39

39:                                               ; preds = %38, %34
  %40 = load i32, ptr %2, align 4
  ret i32 %40
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_open(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 1, ptr noundef null)
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @luaL_optlstring(ptr noundef %9, i32 noundef 2, ptr noundef @.str.19, ptr noundef null)
  store ptr %10, ptr %4, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = call ptr @newfile(ptr noundef %11)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %4, align 8
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = call i32 @l_checkmode(ptr noundef %14)
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %24, label %20

20:                                               ; preds = %1
  %21 = load ptr, ptr %2, align 8
  %22 = call i32 @luaL_argerror(ptr noundef %21, i32 noundef 2, ptr noundef @.str.31)
  %23 = icmp ne i32 %22, 0
  br label %24

24:                                               ; preds = %20, %1
  %25 = phi i1 [ true, %1 ], [ %23, %20 ]
  %26 = zext i1 %25 to i32
  %27 = call ptr @__errno_location() #5
  store i32 0, ptr %27, align 4
  %28 = load ptr, ptr %3, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = call noalias ptr @fopen64(ptr noundef %28, ptr noundef %29)
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.luaL_Stream, ptr %31, i32 0, i32 0
  store ptr %30, ptr %32, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.luaL_Stream, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = icmp eq ptr %35, null
  br i1 %36, label %37, label %41

37:                                               ; preds = %24
  %38 = load ptr, ptr %2, align 8
  %39 = load ptr, ptr %3, align 8
  %40 = call i32 @luaL_fileresult(ptr noundef %38, i32 noundef 0, ptr noundef %39)
  br label %42

41:                                               ; preds = %24
  br label %42

42:                                               ; preds = %41, %37
  %43 = phi i32 [ %40, %37 ], [ 1, %41 ]
  ret i32 %43
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_output(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @g_iofile(ptr noundef %3, ptr noundef @.str.2, ptr noundef @.str.34)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_popen(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call ptr @luaL_checklstring(ptr noundef %6, i32 noundef 1, ptr noundef null)
  store ptr %7, ptr %3, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @luaL_optlstring(ptr noundef %8, i32 noundef 2, ptr noundef @.str.19, ptr noundef null)
  store ptr %9, ptr %4, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = call ptr @newprefile(ptr noundef %10)
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds i8, ptr %12, i64 0
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp eq i32 %15, 114
  br i1 %16, label %23, label %17

17:                                               ; preds = %1
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds i8, ptr %18, i64 0
  %20 = load i8, ptr %19, align 1
  %21 = sext i8 %20 to i32
  %22 = icmp eq i32 %21, 119
  br i1 %22, label %23, label %29

23:                                               ; preds = %17, %1
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds i8, ptr %24, i64 1
  %26 = load i8, ptr %25, align 1
  %27 = sext i8 %26 to i32
  %28 = icmp eq i32 %27, 0
  br label %29

29:                                               ; preds = %23, %17
  %30 = phi i1 [ false, %17 ], [ %28, %23 ]
  %31 = zext i1 %30 to i32
  %32 = icmp ne i32 %31, 0
  %33 = zext i1 %32 to i32
  %34 = sext i32 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %40, label %36

36:                                               ; preds = %29
  %37 = load ptr, ptr %2, align 8
  %38 = call i32 @luaL_argerror(ptr noundef %37, i32 noundef 2, ptr noundef @.str.31)
  %39 = icmp ne i32 %38, 0
  br label %40

40:                                               ; preds = %36, %29
  %41 = phi i1 [ true, %29 ], [ %39, %36 ]
  %42 = zext i1 %41 to i32
  %43 = call ptr @__errno_location() #5
  store i32 0, ptr %43, align 4
  %44 = load ptr, ptr %3, align 8
  %45 = load ptr, ptr %4, align 8
  %46 = load ptr, ptr %2, align 8
  %47 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %46, ptr noundef @.str.35)
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.luaL_Stream, ptr %48, i32 0, i32 0
  store ptr null, ptr %49, align 8
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %struct.luaL_Stream, ptr %50, i32 0, i32 1
  store ptr @io_pclose, ptr %51, align 8
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.luaL_Stream, ptr %52, i32 0, i32 0
  %54 = load ptr, ptr %53, align 8
  %55 = icmp eq ptr %54, null
  br i1 %55, label %56, label %60

56:                                               ; preds = %40
  %57 = load ptr, ptr %2, align 8
  %58 = load ptr, ptr %3, align 8
  %59 = call i32 @luaL_fileresult(ptr noundef %57, i32 noundef 0, ptr noundef %58)
  br label %61

60:                                               ; preds = %40
  br label %61

61:                                               ; preds = %60, %56
  %62 = phi i32 [ %59, %56 ], [ 1, %60 ]
  ret i32 %62
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_read(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @getiofile(ptr noundef %4, ptr noundef @.str)
  %6 = call i32 @g_read(ptr noundef %3, ptr noundef %5, i32 noundef 1)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_tmpfile(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @newfile(ptr noundef %4)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #5
  store i32 0, ptr %6, align 4
  %7 = call noalias ptr @tmpfile64()
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.luaL_Stream, ptr %8, i32 0, i32 0
  store ptr %7, ptr %9, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.luaL_Stream, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  %13 = icmp eq ptr %12, null
  br i1 %13, label %14, label %17

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = call i32 @luaL_fileresult(ptr noundef %15, i32 noundef 0, ptr noundef null)
  br label %18

17:                                               ; preds = %1
  br label %18

18:                                               ; preds = %17, %14
  %19 = phi i32 [ %16, %14 ], [ 1, %17 ]
  ret i32 %19
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_type(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  call void @luaL_checkany(ptr noundef %4, i32 noundef 1)
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaL_testudata(ptr noundef %5, i32 noundef 1, ptr noundef @.str.16)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = icmp eq ptr %7, null
  br i1 %8, label %9, label %11

9:                                                ; preds = %1
  %10 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %10)
  br label %23

11:                                               ; preds = %1
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.luaL_Stream, ptr %12, i32 0, i32 1
  %14 = load ptr, ptr %13, align 8
  %15 = icmp eq ptr %14, null
  br i1 %15, label %16, label %19

16:                                               ; preds = %11
  %17 = load ptr, ptr %2, align 8
  %18 = call ptr @lua_pushstring(ptr noundef %17, ptr noundef @.str.36)
  br label %22

19:                                               ; preds = %11
  %20 = load ptr, ptr %2, align 8
  %21 = call ptr @lua_pushstring(ptr noundef %20, ptr noundef @.str.37)
  br label %22

22:                                               ; preds = %19, %16
  br label %23

23:                                               ; preds = %22, %9
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_write(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @getiofile(ptr noundef %4, ptr noundef @.str.2)
  %6 = call i32 @g_write(ptr noundef %3, ptr noundef %5, i32 noundef 1)
  ret i32 %6
}

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare i32 @lua_getfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_close(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call ptr @tofile(ptr noundef %3)
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @aux_close(ptr noundef %5)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @tofile(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 1
  %8 = load ptr, ptr %7, align 8
  %9 = icmp eq ptr %8, null
  %10 = zext i1 %9 to i32
  %11 = icmp ne i32 %10, 0
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %16, ptr noundef @.str.17)
  br label %18

18:                                               ; preds = %15, %1
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.luaL_Stream, ptr %19, i32 0, i32 0
  %21 = load ptr, ptr %20, align 8
  ret ptr %21
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @aux_close(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @luaL_checkudata(ptr noundef %5, i32 noundef 1, ptr noundef @.str.16)
  store ptr %6, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.luaL_Stream, ptr %7, i32 0, i32 1
  %9 = load ptr, ptr %8, align 8
  store volatile ptr %9, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.luaL_Stream, ptr %10, i32 0, i32 1
  store ptr null, ptr %11, align 8
  %12 = load volatile ptr, ptr %4, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = call i32 %12(ptr noundef %13)
  ret i32 %14
}

declare ptr @luaL_checkudata(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getiofile(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @lua_getfield(ptr noundef %6, i32 noundef -1001000, ptr noundef %7)
  %9 = load ptr, ptr %3, align 8
  %10 = call ptr @lua_touserdata(ptr noundef %9, i32 noundef -1)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.luaL_Stream, ptr %11, i32 0, i32 1
  %13 = load ptr, ptr %12, align 8
  %14 = icmp eq ptr %13, null
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %2
  %21 = load ptr, ptr %3, align 8
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds i8, ptr %22, i64 4
  %24 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %21, ptr noundef @.str.18, ptr noundef %23)
  br label %25

25:                                               ; preds = %20, %2
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.luaL_Stream, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  ret ptr %28
}

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__errno_location() #2

declare i32 @luaL_fileresult(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @fflush(ptr noundef) #1

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @g_iofile(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call i32 @lua_type(ptr noundef %8, i32 noundef 1)
  %10 = icmp sle i32 %9, 0
  br i1 %10, label %27, label %11

11:                                               ; preds = %3
  %12 = load ptr, ptr %4, align 8
  %13 = call ptr @lua_tolstring(ptr noundef %12, i32 noundef 1, ptr noundef null)
  store ptr %13, ptr %7, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = icmp ne ptr %14, null
  br i1 %15, label %16, label %20

16:                                               ; preds = %11
  %17 = load ptr, ptr %4, align 8
  %18 = load ptr, ptr %7, align 8
  %19 = load ptr, ptr %6, align 8
  call void @opencheck(ptr noundef %17, ptr noundef %18, ptr noundef %19)
  br label %24

20:                                               ; preds = %11
  %21 = load ptr, ptr %4, align 8
  %22 = call ptr @tofile(ptr noundef %21)
  %23 = load ptr, ptr %4, align 8
  call void @lua_pushvalue(ptr noundef %23, i32 noundef 1)
  br label %24

24:                                               ; preds = %20, %16
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  call void @lua_setfield(ptr noundef %25, i32 noundef -1001000, ptr noundef %26)
  br label %27

27:                                               ; preds = %24, %3
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = call i32 @lua_getfield(ptr noundef %28, i32 noundef -1001000, ptr noundef %29)
  ret i32 1
}

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @opencheck(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = call ptr @newfile(ptr noundef %8)
  store ptr %9, ptr %7, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = call noalias ptr @fopen64(ptr noundef %10, ptr noundef %11)
  %13 = load ptr, ptr %7, align 8
  %14 = getelementptr inbounds %struct.luaL_Stream, ptr %13, i32 0, i32 0
  store ptr %12, ptr %14, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.luaL_Stream, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = icmp eq ptr %17, null
  %19 = zext i1 %18 to i32
  %20 = icmp ne i32 %19, 0
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %24, label %31

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  %26 = load ptr, ptr %5, align 8
  %27 = call ptr @__errno_location() #5
  %28 = load i32, ptr %27, align 4
  %29 = call ptr @strerror(i32 noundef %28) #6
  %30 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %25, ptr noundef @.str.20, ptr noundef %26, ptr noundef %29)
  br label %31

31:                                               ; preds = %24, %3
  ret void
}

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @newfile(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @newprefile(ptr noundef %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 0
  store ptr null, ptr %7, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.luaL_Stream, ptr %8, i32 0, i32 1
  store ptr @io_fclose, ptr %9, align 8
  %10 = load ptr, ptr %3, align 8
  ret ptr %10
}

declare noalias ptr @fopen64(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @strerror(i32 noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @newprefile(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @lua_newuserdatauv(ptr noundef %4, i64 noundef 16, i32 noundef 0)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 1
  store ptr null, ptr %7, align 8
  %8 = load ptr, ptr %2, align 8
  call void @luaL_setmetatable(ptr noundef %8, ptr noundef @.str.16)
  %9 = load ptr, ptr %3, align 8
  ret ptr %9
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_fclose(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #5
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = getelementptr inbounds %struct.luaL_Stream, ptr %8, i32 0, i32 0
  %10 = load ptr, ptr %9, align 8
  %11 = call i32 @fclose(ptr noundef %10)
  %12 = icmp eq i32 %11, 0
  %13 = zext i1 %12 to i32
  %14 = call i32 @luaL_fileresult(ptr noundef %7, i32 noundef %13, ptr noundef null)
  ret i32 %14
}

declare ptr @lua_newuserdatauv(ptr noundef, i64 noundef, i32 noundef) #1

declare void @luaL_setmetatable(ptr noundef, ptr noundef) #1

declare i32 @fclose(ptr noundef) #1

declare void @lua_pushnil(ptr noundef) #1

declare void @lua_copy(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @aux_lines(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_gettop(ptr noundef %6)
  %8 = sub nsw i32 %7, 1
  store i32 %8, ptr %5, align 4
  %9 = load i32, ptr %5, align 4
  %10 = icmp sle i32 %9, 250
  %11 = zext i1 %10 to i32
  %12 = icmp ne i32 %11, 0
  %13 = zext i1 %12 to i32
  %14 = sext i32 %13 to i64
  %15 = icmp ne i64 %14, 0
  br i1 %15, label %20, label %16

16:                                               ; preds = %2
  %17 = load ptr, ptr %3, align 8
  %18 = call i32 @luaL_argerror(ptr noundef %17, i32 noundef 252, ptr noundef @.str.21)
  %19 = icmp ne i32 %18, 0
  br label %20

20:                                               ; preds = %16, %2
  %21 = phi i1 [ true, %2 ], [ %19, %16 ]
  %22 = zext i1 %21 to i32
  %23 = load ptr, ptr %3, align 8
  call void @lua_pushvalue(ptr noundef %23, i32 noundef 1)
  %24 = load ptr, ptr %3, align 8
  %25 = load i32, ptr %5, align 4
  %26 = sext i32 %25 to i64
  call void @lua_pushinteger(ptr noundef %24, i64 noundef %26)
  %27 = load ptr, ptr %3, align 8
  %28 = load i32, ptr %4, align 4
  call void @lua_pushboolean(ptr noundef %27, i32 noundef %28)
  %29 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %29, i32 noundef 2, i32 noundef 3)
  %30 = load ptr, ptr %3, align 8
  %31 = load i32, ptr %5, align 4
  %32 = add nsw i32 3, %31
  call void @lua_pushcclosure(ptr noundef %30, ptr noundef @io_readline, i32 noundef %32)
  ret void
}

declare i32 @lua_gettop(ptr noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_pushboolean(ptr noundef, i32 noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_readline(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call ptr @lua_touserdata(ptr noundef %7, i32 noundef -1001001)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = call i64 @lua_tointegerx(ptr noundef %9, i32 noundef -1001002, ptr noundef null)
  %11 = trunc i64 %10 to i32
  store i32 %11, ptr %6, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.luaL_Stream, ptr %12, i32 0, i32 1
  %14 = load ptr, ptr %13, align 8
  %15 = icmp eq ptr %14, null
  br i1 %15, label %16, label %19

16:                                               ; preds = %1
  %17 = load ptr, ptr %3, align 8
  %18 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %17, ptr noundef @.str.22)
  store i32 %18, ptr %2, align 4
  br label %69

19:                                               ; preds = %1
  %20 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %20, i32 noundef 1)
  %21 = load ptr, ptr %3, align 8
  %22 = load i32, ptr %6, align 4
  call void @luaL_checkstack(ptr noundef %21, i32 noundef %22, ptr noundef @.str.21)
  store i32 1, ptr %5, align 4
  br label %23

23:                                               ; preds = %32, %19
  %24 = load i32, ptr %5, align 4
  %25 = load i32, ptr %6, align 4
  %26 = icmp sle i32 %24, %25
  br i1 %26, label %27, label %35

27:                                               ; preds = %23
  %28 = load ptr, ptr %3, align 8
  %29 = load i32, ptr %5, align 4
  %30 = add nsw i32 3, %29
  %31 = sub nsw i32 -1001000, %30
  call void @lua_pushvalue(ptr noundef %28, i32 noundef %31)
  br label %32

32:                                               ; preds = %27
  %33 = load i32, ptr %5, align 4
  %34 = add nsw i32 %33, 1
  store i32 %34, ptr %5, align 4
  br label %23, !llvm.loop !6

35:                                               ; preds = %23
  %36 = load ptr, ptr %3, align 8
  %37 = load ptr, ptr %4, align 8
  %38 = getelementptr inbounds %struct.luaL_Stream, ptr %37, i32 0, i32 0
  %39 = load ptr, ptr %38, align 8
  %40 = call i32 @g_read(ptr noundef %36, ptr noundef %39, i32 noundef 2)
  store i32 %40, ptr %6, align 4
  %41 = load ptr, ptr %3, align 8
  %42 = load i32, ptr %6, align 4
  %43 = sub nsw i32 0, %42
  %44 = call i32 @lua_toboolean(ptr noundef %41, i32 noundef %43)
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %46, label %48

46:                                               ; preds = %35
  %47 = load i32, ptr %6, align 4
  store i32 %47, ptr %2, align 4
  br label %69

48:                                               ; preds = %35
  %49 = load i32, ptr %6, align 4
  %50 = icmp sgt i32 %49, 1
  br i1 %50, label %51, label %59

51:                                               ; preds = %48
  %52 = load ptr, ptr %3, align 8
  %53 = load ptr, ptr %3, align 8
  %54 = load i32, ptr %6, align 4
  %55 = sub nsw i32 0, %54
  %56 = add nsw i32 %55, 1
  %57 = call ptr @lua_tolstring(ptr noundef %53, i32 noundef %56, ptr noundef null)
  %58 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %52, ptr noundef @.str.23, ptr noundef %57)
  store i32 %58, ptr %2, align 4
  br label %69

59:                                               ; preds = %48
  %60 = load ptr, ptr %3, align 8
  %61 = call i32 @lua_toboolean(ptr noundef %60, i32 noundef -1001003)
  %62 = icmp ne i32 %61, 0
  br i1 %62, label %63, label %68

63:                                               ; preds = %59
  %64 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %64, i32 noundef 0)
  %65 = load ptr, ptr %3, align 8
  call void @lua_pushvalue(ptr noundef %65, i32 noundef -1001001)
  %66 = load ptr, ptr %3, align 8
  %67 = call i32 @aux_close(ptr noundef %66)
  br label %68

68:                                               ; preds = %63, %59
  store i32 0, ptr %2, align 4
  br label %69

69:                                               ; preds = %68, %51, %46, %16
  %70 = load i32, ptr %2, align 4
  ret i32 %70
}

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_checkstack(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @g_read(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @lua_gettop(ptr noundef %13)
  %15 = sub nsw i32 %14, 1
  store i32 %15, ptr %8, align 4
  %16 = load ptr, ptr %6, align 8
  call void @clearerr(ptr noundef %16) #6
  %17 = call ptr @__errno_location() #5
  store i32 0, ptr %17, align 4
  %18 = load i32, ptr %8, align 4
  %19 = icmp eq i32 %18, 0
  br i1 %19, label %20, label %26

20:                                               ; preds = %3
  %21 = load ptr, ptr %5, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = call i32 @read_line(ptr noundef %21, ptr noundef %22, i32 noundef 1)
  store i32 %23, ptr %10, align 4
  %24 = load i32, ptr %7, align 4
  %25 = add nsw i32 %24, 1
  store i32 %25, ptr %9, align 4
  br label %102

26:                                               ; preds = %3
  %27 = load ptr, ptr %5, align 8
  %28 = load i32, ptr %8, align 4
  %29 = add nsw i32 %28, 20
  call void @luaL_checkstack(ptr noundef %27, i32 noundef %29, ptr noundef @.str.21)
  store i32 1, ptr %10, align 4
  %30 = load i32, ptr %7, align 4
  store i32 %30, ptr %9, align 4
  br label %31

31:                                               ; preds = %98, %26
  %32 = load i32, ptr %8, align 4
  %33 = add nsw i32 %32, -1
  store i32 %33, ptr %8, align 4
  %34 = icmp ne i32 %32, 0
  br i1 %34, label %35, label %38

35:                                               ; preds = %31
  %36 = load i32, ptr %10, align 4
  %37 = icmp ne i32 %36, 0
  br label %38

38:                                               ; preds = %35, %31
  %39 = phi i1 [ false, %31 ], [ %37, %35 ]
  br i1 %39, label %40, label %101

40:                                               ; preds = %38
  %41 = load ptr, ptr %5, align 8
  %42 = load i32, ptr %9, align 4
  %43 = call i32 @lua_type(ptr noundef %41, i32 noundef %42)
  %44 = icmp eq i32 %43, 3
  br i1 %44, label %45, label %62

45:                                               ; preds = %40
  %46 = load ptr, ptr %5, align 8
  %47 = load i32, ptr %9, align 4
  %48 = call i64 @luaL_checkinteger(ptr noundef %46, i32 noundef %47)
  store i64 %48, ptr %11, align 8
  %49 = load i64, ptr %11, align 8
  %50 = icmp eq i64 %49, 0
  br i1 %50, label %51, label %55

51:                                               ; preds = %45
  %52 = load ptr, ptr %5, align 8
  %53 = load ptr, ptr %6, align 8
  %54 = call i32 @test_eof(ptr noundef %52, ptr noundef %53)
  br label %60

55:                                               ; preds = %45
  %56 = load ptr, ptr %5, align 8
  %57 = load ptr, ptr %6, align 8
  %58 = load i64, ptr %11, align 8
  %59 = call i32 @read_chars(ptr noundef %56, ptr noundef %57, i64 noundef %58)
  br label %60

60:                                               ; preds = %55, %51
  %61 = phi i32 [ %54, %51 ], [ %59, %55 ]
  store i32 %61, ptr %10, align 4
  br label %97

62:                                               ; preds = %40
  %63 = load ptr, ptr %5, align 8
  %64 = load i32, ptr %9, align 4
  %65 = call ptr @luaL_checklstring(ptr noundef %63, i32 noundef %64, ptr noundef null)
  store ptr %65, ptr %12, align 8
  %66 = load ptr, ptr %12, align 8
  %67 = load i8, ptr %66, align 1
  %68 = sext i8 %67 to i32
  %69 = icmp eq i32 %68, 42
  br i1 %69, label %70, label %73

70:                                               ; preds = %62
  %71 = load ptr, ptr %12, align 8
  %72 = getelementptr inbounds i8, ptr %71, i32 1
  store ptr %72, ptr %12, align 8
  br label %73

73:                                               ; preds = %70, %62
  %74 = load ptr, ptr %12, align 8
  %75 = load i8, ptr %74, align 1
  %76 = sext i8 %75 to i32
  switch i32 %76, label %92 [
    i32 110, label %77
    i32 108, label %81
    i32 76, label %85
    i32 97, label %89
  ]

77:                                               ; preds = %73
  %78 = load ptr, ptr %5, align 8
  %79 = load ptr, ptr %6, align 8
  %80 = call i32 @read_number(ptr noundef %78, ptr noundef %79)
  store i32 %80, ptr %10, align 4
  br label %96

81:                                               ; preds = %73
  %82 = load ptr, ptr %5, align 8
  %83 = load ptr, ptr %6, align 8
  %84 = call i32 @read_line(ptr noundef %82, ptr noundef %83, i32 noundef 1)
  store i32 %84, ptr %10, align 4
  br label %96

85:                                               ; preds = %73
  %86 = load ptr, ptr %5, align 8
  %87 = load ptr, ptr %6, align 8
  %88 = call i32 @read_line(ptr noundef %86, ptr noundef %87, i32 noundef 0)
  store i32 %88, ptr %10, align 4
  br label %96

89:                                               ; preds = %73
  %90 = load ptr, ptr %5, align 8
  %91 = load ptr, ptr %6, align 8
  call void @read_all(ptr noundef %90, ptr noundef %91)
  store i32 1, ptr %10, align 4
  br label %96

92:                                               ; preds = %73
  %93 = load ptr, ptr %5, align 8
  %94 = load i32, ptr %9, align 4
  %95 = call i32 @luaL_argerror(ptr noundef %93, i32 noundef %94, ptr noundef @.str.24)
  store i32 %95, ptr %4, align 4
  br label %119

96:                                               ; preds = %89, %85, %81, %77
  br label %97

97:                                               ; preds = %96, %60
  br label %98

98:                                               ; preds = %97
  %99 = load i32, ptr %9, align 4
  %100 = add nsw i32 %99, 1
  store i32 %100, ptr %9, align 4
  br label %31, !llvm.loop !8

101:                                              ; preds = %38
  br label %102

102:                                              ; preds = %101, %20
  %103 = load ptr, ptr %6, align 8
  %104 = call i32 @ferror(ptr noundef %103) #6
  %105 = icmp ne i32 %104, 0
  br i1 %105, label %106, label %109

106:                                              ; preds = %102
  %107 = load ptr, ptr %5, align 8
  %108 = call i32 @luaL_fileresult(ptr noundef %107, i32 noundef 0, ptr noundef null)
  store i32 %108, ptr %4, align 4
  br label %119

109:                                              ; preds = %102
  %110 = load i32, ptr %10, align 4
  %111 = icmp ne i32 %110, 0
  br i1 %111, label %115, label %112

112:                                              ; preds = %109
  %113 = load ptr, ptr %5, align 8
  call void @lua_settop(ptr noundef %113, i32 noundef -2)
  %114 = load ptr, ptr %5, align 8
  call void @lua_pushnil(ptr noundef %114)
  br label %115

115:                                              ; preds = %112, %109
  %116 = load i32, ptr %9, align 4
  %117 = load i32, ptr %7, align 4
  %118 = sub nsw i32 %116, %117
  store i32 %118, ptr %4, align 4
  br label %119

119:                                              ; preds = %115, %106, %92
  %120 = load i32, ptr %4, align 4
  ret i32 %120
}

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

; Function Attrs: nounwind
declare void @clearerr(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @read_line(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.luaL_Buffer, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %11 = load ptr, ptr %4, align 8
  call void @luaL_buffinit(ptr noundef %11, ptr noundef %7)
  br label %12

12:                                               ; preds = %46, %3
  %13 = call ptr @luaL_prepbuffsize(ptr noundef %7, i64 noundef 1024)
  store ptr %13, ptr %9, align 8
  store i32 0, ptr %10, align 4
  br label %14

14:                                               ; preds = %26, %12
  %15 = load i32, ptr %10, align 4
  %16 = icmp slt i32 %15, 1024
  br i1 %16, label %17, label %24

17:                                               ; preds = %14
  %18 = load ptr, ptr %5, align 8
  %19 = call i32 @getc(ptr noundef %18)
  store i32 %19, ptr %8, align 4
  %20 = icmp ne i32 %19, -1
  br i1 %20, label %21, label %24

21:                                               ; preds = %17
  %22 = load i32, ptr %8, align 4
  %23 = icmp ne i32 %22, 10
  br label %24

24:                                               ; preds = %21, %17, %14
  %25 = phi i1 [ false, %17 ], [ false, %14 ], [ %23, %21 ]
  br i1 %25, label %26, label %34

26:                                               ; preds = %24
  %27 = load i32, ptr %8, align 4
  %28 = trunc i32 %27 to i8
  %29 = load ptr, ptr %9, align 8
  %30 = load i32, ptr %10, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, ptr %10, align 4
  %32 = sext i32 %30 to i64
  %33 = getelementptr inbounds i8, ptr %29, i64 %32
  store i8 %28, ptr %33, align 1
  br label %14, !llvm.loop !9

34:                                               ; preds = %24
  %35 = load i32, ptr %10, align 4
  %36 = sext i32 %35 to i64
  %37 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 2
  %38 = load i64, ptr %37, align 8
  %39 = add i64 %38, %36
  store i64 %39, ptr %37, align 8
  br label %40

40:                                               ; preds = %34
  %41 = load i32, ptr %8, align 4
  %42 = icmp ne i32 %41, -1
  br i1 %42, label %43, label %46

43:                                               ; preds = %40
  %44 = load i32, ptr %8, align 4
  %45 = icmp ne i32 %44, 10
  br label %46

46:                                               ; preds = %43, %40
  %47 = phi i1 [ false, %40 ], [ %45, %43 ]
  br i1 %47, label %12, label %48, !llvm.loop !10

48:                                               ; preds = %46
  %49 = load i32, ptr %6, align 4
  %50 = icmp ne i32 %49, 0
  br i1 %50, label %74, label %51

51:                                               ; preds = %48
  %52 = load i32, ptr %8, align 4
  %53 = icmp eq i32 %52, 10
  br i1 %53, label %54, label %74

54:                                               ; preds = %51
  %55 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 2
  %56 = load i64, ptr %55, align 8
  %57 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 1
  %58 = load i64, ptr %57, align 8
  %59 = icmp ult i64 %56, %58
  br i1 %59, label %63, label %60

60:                                               ; preds = %54
  %61 = call ptr @luaL_prepbuffsize(ptr noundef %7, i64 noundef 1)
  %62 = icmp ne ptr %61, null
  br label %63

63:                                               ; preds = %60, %54
  %64 = phi i1 [ true, %54 ], [ %62, %60 ]
  %65 = zext i1 %64 to i32
  %66 = load i32, ptr %8, align 4
  %67 = trunc i32 %66 to i8
  %68 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 0
  %69 = load ptr, ptr %68, align 8
  %70 = getelementptr inbounds %struct.luaL_Buffer, ptr %7, i32 0, i32 2
  %71 = load i64, ptr %70, align 8
  %72 = add i64 %71, 1
  store i64 %72, ptr %70, align 8
  %73 = getelementptr inbounds i8, ptr %69, i64 %71
  store i8 %67, ptr %73, align 1
  br label %74

74:                                               ; preds = %63, %51, %48
  call void @luaL_pushresult(ptr noundef %7)
  %75 = load i32, ptr %8, align 4
  %76 = icmp eq i32 %75, 10
  br i1 %76, label %81, label %77

77:                                               ; preds = %74
  %78 = load ptr, ptr %4, align 8
  %79 = call i64 @lua_rawlen(ptr noundef %78, i32 noundef -1)
  %80 = icmp ugt i64 %79, 0
  br label %81

81:                                               ; preds = %77, %74
  %82 = phi i1 [ true, %74 ], [ %80, %77 ]
  %83 = zext i1 %82 to i32
  ret i32 %83
}

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @test_eof(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = call i32 @getc(ptr noundef %6)
  store i32 %7, ptr %5, align 4
  %8 = load i32, ptr %5, align 4
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @ungetc(i32 noundef %8, ptr noundef %9)
  %11 = load ptr, ptr %3, align 8
  %12 = call ptr @lua_pushstring(ptr noundef %11, ptr noundef @.str.25)
  %13 = load i32, ptr %5, align 4
  %14 = icmp ne i32 %13, -1
  %15 = zext i1 %14 to i32
  ret i32 %15
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @read_chars(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  call void @luaL_buffinit(ptr noundef %10, ptr noundef %9)
  %11 = load i64, ptr %6, align 8
  %12 = call ptr @luaL_prepbuffsize(ptr noundef %9, i64 noundef %11)
  store ptr %12, ptr %8, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = load i64, ptr %6, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = call i64 @fread(ptr noundef %13, i64 noundef 1, i64 noundef %14, ptr noundef %15)
  store i64 %16, ptr %7, align 8
  %17 = load i64, ptr %7, align 8
  %18 = getelementptr inbounds %struct.luaL_Buffer, ptr %9, i32 0, i32 2
  %19 = load i64, ptr %18, align 8
  %20 = add i64 %19, %17
  store i64 %20, ptr %18, align 8
  call void @luaL_pushresult(ptr noundef %9)
  %21 = load i64, ptr %7, align 8
  %22 = icmp ugt i64 %21, 0
  %23 = zext i1 %22 to i32
  ret i32 %23
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @read_number(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.RN, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca [2 x i8], align 1
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 0, ptr %7, align 4
  store i32 0, ptr %8, align 4
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 0
  store ptr %10, ptr %11, align 8
  %12 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 2
  store i32 0, ptr %12, align 4
  %13 = call ptr @localeconv() #6
  %14 = getelementptr inbounds %struct.lconv, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  %16 = getelementptr inbounds i8, ptr %15, i64 0
  %17 = load i8, ptr %16, align 1
  %18 = getelementptr inbounds [2 x i8], ptr %9, i64 0, i64 0
  store i8 %17, ptr %18, align 1
  %19 = getelementptr inbounds [2 x i8], ptr %9, i64 0, i64 1
  store i8 46, ptr %19, align 1
  br label %20

20:                                               ; preds = %25, %2
  %21 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 0
  %22 = load ptr, ptr %21, align 8
  %23 = call i32 @getc(ptr noundef %22)
  %24 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 1
  store i32 %23, ptr %24, align 8
  br label %25

25:                                               ; preds = %20
  %26 = call ptr @__ctype_b_loc() #5
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 1
  %29 = load i32, ptr %28, align 8
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds i16, ptr %27, i64 %30
  %32 = load i16, ptr %31, align 2
  %33 = zext i16 %32 to i32
  %34 = and i32 %33, 8192
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %20, label %36, !llvm.loop !11

36:                                               ; preds = %25
  %37 = call i32 @test2(ptr noundef %6, ptr noundef @.str.26)
  %38 = call i32 @test2(ptr noundef %6, ptr noundef @.str.27)
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %46

40:                                               ; preds = %36
  %41 = call i32 @test2(ptr noundef %6, ptr noundef @.str.28)
  %42 = icmp ne i32 %41, 0
  br i1 %42, label %43, label %44

43:                                               ; preds = %40
  store i32 1, ptr %8, align 4
  br label %45

44:                                               ; preds = %40
  store i32 1, ptr %7, align 4
  br label %45

45:                                               ; preds = %44, %43
  br label %46

46:                                               ; preds = %45, %36
  %47 = load i32, ptr %8, align 4
  %48 = call i32 @readdigits(ptr noundef %6, i32 noundef %47)
  %49 = load i32, ptr %7, align 4
  %50 = add nsw i32 %49, %48
  store i32 %50, ptr %7, align 4
  %51 = getelementptr inbounds [2 x i8], ptr %9, i64 0, i64 0
  %52 = call i32 @test2(ptr noundef %6, ptr noundef %51)
  %53 = icmp ne i32 %52, 0
  br i1 %53, label %54, label %59

54:                                               ; preds = %46
  %55 = load i32, ptr %8, align 4
  %56 = call i32 @readdigits(ptr noundef %6, i32 noundef %55)
  %57 = load i32, ptr %7, align 4
  %58 = add nsw i32 %57, %56
  store i32 %58, ptr %7, align 4
  br label %59

59:                                               ; preds = %54, %46
  %60 = load i32, ptr %7, align 4
  %61 = icmp sgt i32 %60, 0
  br i1 %61, label %62, label %72

62:                                               ; preds = %59
  %63 = load i32, ptr %8, align 4
  %64 = icmp ne i32 %63, 0
  %65 = zext i1 %64 to i64
  %66 = select i1 %64, ptr @.str.29, ptr @.str.30
  %67 = call i32 @test2(ptr noundef %6, ptr noundef %66)
  %68 = icmp ne i32 %67, 0
  br i1 %68, label %69, label %72

69:                                               ; preds = %62
  %70 = call i32 @test2(ptr noundef %6, ptr noundef @.str.26)
  %71 = call i32 @readdigits(ptr noundef %6, i32 noundef 0)
  br label %72

72:                                               ; preds = %69, %62, %59
  %73 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 1
  %74 = load i32, ptr %73, align 8
  %75 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 0
  %76 = load ptr, ptr %75, align 8
  %77 = call i32 @ungetc(i32 noundef %74, ptr noundef %76)
  %78 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 3
  %79 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 2
  %80 = load i32, ptr %79, align 4
  %81 = sext i32 %80 to i64
  %82 = getelementptr inbounds [201 x i8], ptr %78, i64 0, i64 %81
  store i8 0, ptr %82, align 1
  %83 = load ptr, ptr %4, align 8
  %84 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 3
  %85 = getelementptr inbounds [201 x i8], ptr %84, i64 0, i64 0
  %86 = call i64 @lua_stringtonumber(ptr noundef %83, ptr noundef %85)
  %87 = icmp ne i64 %86, 0
  %88 = zext i1 %87 to i32
  %89 = sext i32 %88 to i64
  %90 = icmp ne i64 %89, 0
  br i1 %90, label %91, label %92

91:                                               ; preds = %72
  store i32 1, ptr %3, align 4
  br label %94

92:                                               ; preds = %72
  %93 = load ptr, ptr %4, align 8
  call void @lua_pushnil(ptr noundef %93)
  store i32 0, ptr %3, align 4
  br label %94

94:                                               ; preds = %92, %91
  %95 = load i32, ptr %3, align 4
  ret i32 %95
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @read_all(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca %struct.luaL_Buffer, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  call void @luaL_buffinit(ptr noundef %8, ptr noundef %6)
  br label %9

9:                                                ; preds = %18, %2
  %10 = call ptr @luaL_prepbuffsize(ptr noundef %6, i64 noundef 1024)
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = call i64 @fread(ptr noundef %11, i64 noundef 1, i64 noundef 1024, ptr noundef %12)
  store i64 %13, ptr %5, align 8
  %14 = load i64, ptr %5, align 8
  %15 = getelementptr inbounds %struct.luaL_Buffer, ptr %6, i32 0, i32 2
  %16 = load i64, ptr %15, align 8
  %17 = add i64 %16, %14
  store i64 %17, ptr %15, align 8
  br label %18

18:                                               ; preds = %9
  %19 = load i64, ptr %5, align 8
  %20 = icmp eq i64 %19, 1024
  br i1 %20, label %9, label %21, !llvm.loop !12

21:                                               ; preds = %18
  call void @luaL_pushresult(ptr noundef %6)
  ret void
}

; Function Attrs: nounwind
declare i32 @ferror(ptr noundef) #3

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

declare ptr @luaL_prepbuffsize(ptr noundef, i64 noundef) #1

declare i32 @getc(ptr noundef) #1

declare void @luaL_pushresult(ptr noundef) #1

declare i64 @lua_rawlen(ptr noundef, i32 noundef) #1

declare i32 @ungetc(i32 noundef, ptr noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

declare i64 @fread(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @localeconv() #3

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__ctype_b_loc() #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @test2(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.RN, ptr %6, i32 0, i32 1
  %8 = load i32, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds i8, ptr %9, i64 0
  %11 = load i8, ptr %10, align 1
  %12 = sext i8 %11 to i32
  %13 = icmp eq i32 %8, %12
  br i1 %13, label %23, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.RN, ptr %15, i32 0, i32 1
  %17 = load i32, ptr %16, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds i8, ptr %18, i64 1
  %20 = load i8, ptr %19, align 1
  %21 = sext i8 %20 to i32
  %22 = icmp eq i32 %17, %21
  br i1 %22, label %23, label %26

23:                                               ; preds = %14, %2
  %24 = load ptr, ptr %4, align 8
  %25 = call i32 @nextc(ptr noundef %24)
  store i32 %25, ptr %3, align 4
  br label %27

26:                                               ; preds = %14
  store i32 0, ptr %3, align 4
  br label %27

27:                                               ; preds = %26, %23
  %28 = load i32, ptr %3, align 4
  ret i32 %28
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @readdigits(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  store i32 0, ptr %5, align 4
  br label %6

6:                                                ; preds = %39, %2
  %7 = load i32, ptr %4, align 4
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %9, label %21

9:                                                ; preds = %6
  %10 = call ptr @__ctype_b_loc() #5
  %11 = load ptr, ptr %10, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds %struct.RN, ptr %12, i32 0, i32 1
  %14 = load i32, ptr %13, align 8
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds i16, ptr %11, i64 %15
  %17 = load i16, ptr %16, align 2
  %18 = zext i16 %17 to i32
  %19 = and i32 %18, 4096
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %33, label %37

21:                                               ; preds = %6
  %22 = call ptr @__ctype_b_loc() #5
  %23 = load ptr, ptr %22, align 8
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.RN, ptr %24, i32 0, i32 1
  %26 = load i32, ptr %25, align 8
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds i16, ptr %23, i64 %27
  %29 = load i16, ptr %28, align 2
  %30 = zext i16 %29 to i32
  %31 = and i32 %30, 2048
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %37

33:                                               ; preds = %21, %9
  %34 = load ptr, ptr %3, align 8
  %35 = call i32 @nextc(ptr noundef %34)
  %36 = icmp ne i32 %35, 0
  br label %37

37:                                               ; preds = %33, %21, %9
  %38 = phi i1 [ false, %21 ], [ false, %9 ], [ %36, %33 ]
  br i1 %38, label %39, label %42

39:                                               ; preds = %37
  %40 = load i32, ptr %5, align 4
  %41 = add nsw i32 %40, 1
  store i32 %41, ptr %5, align 4
  br label %6, !llvm.loop !13

42:                                               ; preds = %37
  %43 = load i32, ptr %5, align 4
  ret i32 %43
}

declare i64 @lua_stringtonumber(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @nextc(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = getelementptr inbounds %struct.RN, ptr %4, i32 0, i32 2
  %6 = load i32, ptr %5, align 4
  %7 = icmp sge i32 %6, 200
  %8 = zext i1 %7 to i32
  %9 = icmp ne i32 %8, 0
  %10 = zext i1 %9 to i32
  %11 = sext i32 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %13, label %17

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.RN, ptr %14, i32 0, i32 3
  %16 = getelementptr inbounds [201 x i8], ptr %15, i64 0, i64 0
  store i8 0, ptr %16, align 8
  store i32 0, ptr %2, align 4
  br label %36

17:                                               ; preds = %1
  %18 = load ptr, ptr %3, align 8
  %19 = getelementptr inbounds %struct.RN, ptr %18, i32 0, i32 1
  %20 = load i32, ptr %19, align 8
  %21 = trunc i32 %20 to i8
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.RN, ptr %22, i32 0, i32 3
  %24 = load ptr, ptr %3, align 8
  %25 = getelementptr inbounds %struct.RN, ptr %24, i32 0, i32 2
  %26 = load i32, ptr %25, align 4
  %27 = add nsw i32 %26, 1
  store i32 %27, ptr %25, align 4
  %28 = sext i32 %26 to i64
  %29 = getelementptr inbounds [201 x i8], ptr %23, i64 0, i64 %28
  store i8 %21, ptr %29, align 1
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.RN, ptr %30, i32 0, i32 0
  %32 = load ptr, ptr %31, align 8
  %33 = call i32 @getc(ptr noundef %32)
  %34 = load ptr, ptr %3, align 8
  %35 = getelementptr inbounds %struct.RN, ptr %34, i32 0, i32 1
  store i32 %33, ptr %35, align 8
  store i32 1, ptr %2, align 4
  br label %36

36:                                               ; preds = %17, %13
  %37 = load i32, ptr %2, align 4
  ret i32 %37
}

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @l_checkmode(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load i8, ptr %3, align 1
  %5 = sext i8 %4 to i32
  %6 = icmp ne i32 %5, 0
  br i1 %6, label %7, label %28

7:                                                ; preds = %1
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds i8, ptr %8, i32 1
  store ptr %9, ptr %2, align 8
  %10 = load i8, ptr %8, align 1
  %11 = sext i8 %10 to i32
  %12 = call ptr @strchr(ptr noundef @.str.32, i32 noundef %11) #7
  %13 = icmp ne ptr %12, null
  br i1 %13, label %14, label %28

14:                                               ; preds = %7
  %15 = load ptr, ptr %2, align 8
  %16 = load i8, ptr %15, align 1
  %17 = sext i8 %16 to i32
  %18 = icmp ne i32 %17, 43
  br i1 %18, label %22, label %19

19:                                               ; preds = %14
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds i8, ptr %20, i32 1
  store ptr %21, ptr %2, align 8
  br i1 true, label %22, label %28

22:                                               ; preds = %19, %14
  %23 = load ptr, ptr %2, align 8
  %24 = call i64 @strspn(ptr noundef %23, ptr noundef @.str.33) #7
  %25 = load ptr, ptr %2, align 8
  %26 = call i64 @strlen(ptr noundef %25) #7
  %27 = icmp eq i64 %24, %26
  br label %28

28:                                               ; preds = %22, %19, %7, %1
  %29 = phi i1 [ false, %19 ], [ false, %7 ], [ false, %1 ], [ %27, %22 ]
  %30 = zext i1 %29 to i32
  ret i32 %30
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #4

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strspn(ptr noundef, ptr noundef) #4

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_pclose(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #5
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %struct.luaL_Stream, ptr %9, i32 0, i32 0
  %11 = load ptr, ptr %10, align 8
  %12 = call i32 @luaL_execresult(ptr noundef %7, i32 noundef -1)
  ret i32 %12
}

declare i32 @luaL_execresult(ptr noundef, i32 noundef) #1

declare noalias ptr @tmpfile64() #1

declare void @luaL_checkany(ptr noundef, i32 noundef) #1

declare ptr @luaL_testudata(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @g_write(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %13 = load ptr, ptr %5, align 8
  %14 = call i32 @lua_gettop(ptr noundef %13)
  %15 = load i32, ptr %7, align 4
  %16 = sub nsw i32 %14, %15
  store i32 %16, ptr %8, align 4
  store i32 1, ptr %9, align 4
  %17 = call ptr @__errno_location() #5
  store i32 0, ptr %17, align 4
  br label %18

18:                                               ; preds = %71, %3
  %19 = load i32, ptr %8, align 4
  %20 = add nsw i32 %19, -1
  store i32 %20, ptr %8, align 4
  %21 = icmp ne i32 %19, 0
  br i1 %21, label %22, label %74

22:                                               ; preds = %18
  %23 = load ptr, ptr %5, align 8
  %24 = load i32, ptr %7, align 4
  %25 = call i32 @lua_type(ptr noundef %23, i32 noundef %24)
  %26 = icmp eq i32 %25, 3
  br i1 %26, label %27, label %54

27:                                               ; preds = %22
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %7, align 4
  %30 = call i32 @lua_isinteger(ptr noundef %28, i32 noundef %29)
  %31 = icmp ne i32 %30, 0
  br i1 %31, label %32, label %38

32:                                               ; preds = %27
  %33 = load ptr, ptr %6, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = load i32, ptr %7, align 4
  %36 = call i64 @lua_tointegerx(ptr noundef %34, i32 noundef %35, ptr noundef null)
  %37 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %33, ptr noundef @.str.38, i64 noundef %36)
  br label %44

38:                                               ; preds = %27
  %39 = load ptr, ptr %6, align 8
  %40 = load ptr, ptr %5, align 8
  %41 = load i32, ptr %7, align 4
  %42 = call double @lua_tonumberx(ptr noundef %40, i32 noundef %41, ptr noundef null)
  %43 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %39, ptr noundef @.str.39, double noundef %42)
  br label %44

44:                                               ; preds = %38, %32
  %45 = phi i32 [ %37, %32 ], [ %43, %38 ]
  store i32 %45, ptr %10, align 4
  %46 = load i32, ptr %9, align 4
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %48, label %51

48:                                               ; preds = %44
  %49 = load i32, ptr %10, align 4
  %50 = icmp sgt i32 %49, 0
  br label %51

51:                                               ; preds = %48, %44
  %52 = phi i1 [ false, %44 ], [ %50, %48 ]
  %53 = zext i1 %52 to i32
  store i32 %53, ptr %9, align 4
  br label %70

54:                                               ; preds = %22
  %55 = load ptr, ptr %5, align 8
  %56 = load i32, ptr %7, align 4
  %57 = call ptr @luaL_checklstring(ptr noundef %55, i32 noundef %56, ptr noundef %11)
  store ptr %57, ptr %12, align 8
  %58 = load i32, ptr %9, align 4
  %59 = icmp ne i32 %58, 0
  br i1 %59, label %60, label %67

60:                                               ; preds = %54
  %61 = load ptr, ptr %12, align 8
  %62 = load i64, ptr %11, align 8
  %63 = load ptr, ptr %6, align 8
  %64 = call i64 @fwrite(ptr noundef %61, i64 noundef 1, i64 noundef %62, ptr noundef %63)
  %65 = load i64, ptr %11, align 8
  %66 = icmp eq i64 %64, %65
  br label %67

67:                                               ; preds = %60, %54
  %68 = phi i1 [ false, %54 ], [ %66, %60 ]
  %69 = zext i1 %68 to i32
  store i32 %69, ptr %9, align 4
  br label %70

70:                                               ; preds = %67, %51
  br label %71

71:                                               ; preds = %70
  %72 = load i32, ptr %7, align 4
  %73 = add nsw i32 %72, 1
  store i32 %73, ptr %7, align 4
  br label %18, !llvm.loop !14

74:                                               ; preds = %18
  %75 = load i32, ptr %9, align 4
  %76 = icmp ne i32 %75, 0
  %77 = zext i1 %76 to i32
  %78 = sext i32 %77 to i64
  %79 = icmp ne i64 %78, 0
  br i1 %79, label %80, label %81

80:                                               ; preds = %74
  store i32 1, ptr %4, align 4
  br label %85

81:                                               ; preds = %74
  %82 = load ptr, ptr %5, align 8
  %83 = load i32, ptr %9, align 4
  %84 = call i32 @luaL_fileresult(ptr noundef %82, i32 noundef %83, ptr noundef null)
  store i32 %84, ptr %4, align 4
  br label %85

85:                                               ; preds = %81, %80
  %86 = load i32, ptr %4, align 4
  ret i32 %86
}

declare i32 @lua_isinteger(ptr noundef, i32 noundef) #1

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #1

declare double @lua_tonumberx(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @fwrite(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

declare i32 @luaL_newmetatable(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_gc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 1
  %8 = load ptr, ptr %7, align 8
  %9 = icmp eq ptr %8, null
  br i1 %9, label %18, label %10

10:                                               ; preds = %1
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds %struct.luaL_Stream, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %18

15:                                               ; preds = %10
  %16 = load ptr, ptr %2, align 8
  %17 = call i32 @aux_close(ptr noundef %16)
  br label %18

18:                                               ; preds = %15, %10, %1
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_tostring(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 1
  %8 = load ptr, ptr %7, align 8
  %9 = icmp eq ptr %8, null
  br i1 %9, label %10, label %13

10:                                               ; preds = %1
  %11 = load ptr, ptr %2, align 8
  %12 = call ptr @lua_pushstring(ptr noundef %11, ptr noundef @.str.44)
  br label %19

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.luaL_Stream, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %14, ptr noundef @.str.45, ptr noundef %17)
  br label %19

19:                                               ; preds = %13, %10
  ret i32 1
}

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_read(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @tofile(ptr noundef %4)
  %6 = call i32 @g_read(ptr noundef %3, ptr noundef %5, i32 noundef 2)
  ret i32 %6
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_write(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @tofile(ptr noundef %4)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %6, i32 noundef 1)
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @g_write(ptr noundef %7, ptr noundef %8, i32 noundef 2)
  ret i32 %9
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_lines(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call ptr @tofile(ptr noundef %3)
  %5 = load ptr, ptr %2, align 8
  call void @aux_lines(ptr noundef %5, i32 noundef 0)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_flush(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @tofile(ptr noundef %4)
  store ptr %5, ptr %3, align 8
  %6 = call ptr @__errno_location() #5
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i32 @fflush(ptr noundef %8)
  %10 = icmp eq i32 %9, 0
  %11 = zext i1 %10 to i32
  %12 = call i32 @luaL_fileresult(ptr noundef %7, i32 noundef %11, ptr noundef null)
  ret i32 %12
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_seek(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @tofile(ptr noundef %8)
  store ptr %9, ptr %4, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call i32 @luaL_checkoption(ptr noundef %10, i32 noundef 2, ptr noundef @.str.49, ptr noundef @f_seek.modenames)
  store i32 %11, ptr %5, align 4
  %12 = load ptr, ptr %3, align 8
  %13 = call i64 @luaL_optinteger(ptr noundef %12, i32 noundef 3, i64 noundef 0)
  store i64 %13, ptr %6, align 8
  %14 = load i64, ptr %6, align 8
  store i64 %14, ptr %7, align 8
  %15 = load i64, ptr %7, align 8
  %16 = load i64, ptr %6, align 8
  %17 = icmp eq i64 %15, %16
  %18 = zext i1 %17 to i32
  %19 = icmp ne i32 %18, 0
  %20 = zext i1 %19 to i32
  %21 = sext i32 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %27, label %23

23:                                               ; preds = %1
  %24 = load ptr, ptr %3, align 8
  %25 = call i32 @luaL_argerror(ptr noundef %24, i32 noundef 3, ptr noundef @.str.51)
  %26 = icmp ne i32 %25, 0
  br label %27

27:                                               ; preds = %23, %1
  %28 = phi i1 [ true, %1 ], [ %26, %23 ]
  %29 = zext i1 %28 to i32
  %30 = call ptr @__errno_location() #5
  store i32 0, ptr %30, align 4
  %31 = load ptr, ptr %4, align 8
  %32 = load i64, ptr %7, align 8
  %33 = load i32, ptr %5, align 4
  %34 = sext i32 %33 to i64
  %35 = getelementptr inbounds [3 x i32], ptr @f_seek.mode, i64 0, i64 %34
  %36 = load i32, ptr %35, align 4
  %37 = call i32 @fseek(ptr noundef %31, i64 noundef %32, i32 noundef %36)
  store i32 %37, ptr %5, align 4
  %38 = load i32, ptr %5, align 4
  %39 = icmp ne i32 %38, 0
  %40 = zext i1 %39 to i32
  %41 = sext i32 %40 to i64
  %42 = icmp ne i64 %41, 0
  br i1 %42, label %43, label %46

43:                                               ; preds = %27
  %44 = load ptr, ptr %3, align 8
  %45 = call i32 @luaL_fileresult(ptr noundef %44, i32 noundef 0, ptr noundef null)
  store i32 %45, ptr %2, align 4
  br label %50

46:                                               ; preds = %27
  %47 = load ptr, ptr %3, align 8
  %48 = load ptr, ptr %4, align 8
  %49 = call i64 @ftell(ptr noundef %48)
  call void @lua_pushinteger(ptr noundef %47, i64 noundef %49)
  store i32 1, ptr %2, align 4
  br label %50

50:                                               ; preds = %46, %43
  %51 = load i32, ptr %2, align 4
  ret i32 %51
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @f_setvbuf(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @tofile(ptr noundef %7)
  store ptr %8, ptr %3, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call i32 @luaL_checkoption(ptr noundef %9, i32 noundef 2, ptr noundef null, ptr noundef @f_setvbuf.modenames)
  store i32 %10, ptr %4, align 4
  %11 = load ptr, ptr %2, align 8
  %12 = call i64 @luaL_optinteger(ptr noundef %11, i32 noundef 3, i64 noundef 1024)
  store i64 %12, ptr %5, align 8
  %13 = call ptr @__errno_location() #5
  store i32 0, ptr %13, align 4
  %14 = load ptr, ptr %3, align 8
  %15 = load i32, ptr %4, align 4
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds [3 x i32], ptr @f_setvbuf.mode, i64 0, i64 %16
  %18 = load i32, ptr %17, align 4
  %19 = load i64, ptr %5, align 8
  %20 = call i32 @setvbuf(ptr noundef %14, ptr noundef null, i32 noundef %18, i64 noundef %19) #6
  store i32 %20, ptr %6, align 4
  %21 = load ptr, ptr %2, align 8
  %22 = load i32, ptr %6, align 4
  %23 = icmp eq i32 %22, 0
  %24 = zext i1 %23 to i32
  %25 = call i32 @luaL_fileresult(ptr noundef %21, i32 noundef %24, ptr noundef null)
  ret i32 %25
}

declare i32 @luaL_checkoption(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare i32 @fseek(ptr noundef, i64 noundef, i32 noundef) #1

declare i64 @ftell(ptr noundef) #1

; Function Attrs: nounwind
declare i32 @setvbuf(ptr noundef, ptr noundef, i32 noundef, i64 noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @io_noclose(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checkudata(ptr noundef %4, i32 noundef 1, ptr noundef @.str.16)
  store ptr %5, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.luaL_Stream, ptr %6, i32 0, i32 1
  store ptr @io_noclose, ptr %7, align 8
  %8 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %8)
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @lua_pushstring(ptr noundef %9, ptr noundef @.str.55)
  ret i32 2
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nounwind willreturn memory(none) }
attributes #6 = { nounwind }
attributes #7 = { nounwind willreturn memory(read) }

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
