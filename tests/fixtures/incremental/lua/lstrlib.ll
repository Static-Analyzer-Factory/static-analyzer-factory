; ModuleID = 'lstrlib.c'
source_filename = "lstrlib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%union.anon.0 = type { i32 }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }
%struct.str_Writer = type { i32, %struct.luaL_Buffer }
%struct.GMatchState = type { ptr, ptr, ptr, %struct.MatchState }
%struct.MatchState = type { ptr, ptr, ptr, ptr, i32, i8, [32 x %struct.anon] }
%struct.anon = type { ptr, i64 }
%struct.Header = type { ptr, i32, i32 }
%struct.lconv = type { ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8 }

@strlib = internal constant [18 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str, ptr @str_byte }, %struct.luaL_Reg { ptr @.str.1, ptr @str_char }, %struct.luaL_Reg { ptr @.str.2, ptr @str_dump }, %struct.luaL_Reg { ptr @.str.3, ptr @str_find }, %struct.luaL_Reg { ptr @.str.4, ptr @str_format }, %struct.luaL_Reg { ptr @.str.5, ptr @gmatch }, %struct.luaL_Reg { ptr @.str.6, ptr @str_gsub }, %struct.luaL_Reg { ptr @.str.7, ptr @str_len }, %struct.luaL_Reg { ptr @.str.8, ptr @str_lower }, %struct.luaL_Reg { ptr @.str.9, ptr @str_match }, %struct.luaL_Reg { ptr @.str.10, ptr @str_rep }, %struct.luaL_Reg { ptr @.str.11, ptr @str_reverse }, %struct.luaL_Reg { ptr @.str.12, ptr @str_sub }, %struct.luaL_Reg { ptr @.str.13, ptr @str_upper }, %struct.luaL_Reg { ptr @.str.14, ptr @str_pack }, %struct.luaL_Reg { ptr @.str.15, ptr @str_packsize }, %struct.luaL_Reg { ptr @.str.16, ptr @str_unpack }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [5 x i8] c"byte\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"char\00", align 1
@.str.2 = private unnamed_addr constant [5 x i8] c"dump\00", align 1
@.str.3 = private unnamed_addr constant [5 x i8] c"find\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"format\00", align 1
@.str.5 = private unnamed_addr constant [7 x i8] c"gmatch\00", align 1
@.str.6 = private unnamed_addr constant [5 x i8] c"gsub\00", align 1
@.str.7 = private unnamed_addr constant [4 x i8] c"len\00", align 1
@.str.8 = private unnamed_addr constant [6 x i8] c"lower\00", align 1
@.str.9 = private unnamed_addr constant [6 x i8] c"match\00", align 1
@.str.10 = private unnamed_addr constant [4 x i8] c"rep\00", align 1
@.str.11 = private unnamed_addr constant [8 x i8] c"reverse\00", align 1
@.str.12 = private unnamed_addr constant [4 x i8] c"sub\00", align 1
@.str.13 = private unnamed_addr constant [6 x i8] c"upper\00", align 1
@.str.14 = private unnamed_addr constant [5 x i8] c"pack\00", align 1
@.str.15 = private unnamed_addr constant [9 x i8] c"packsize\00", align 1
@.str.16 = private unnamed_addr constant [7 x i8] c"unpack\00", align 1
@.str.17 = private unnamed_addr constant [22 x i8] c"string slice too long\00", align 1
@.str.18 = private unnamed_addr constant [19 x i8] c"value out of range\00", align 1
@.str.19 = private unnamed_addr constant [30 x i8] c"unable to dump given function\00", align 1
@.str.20 = private unnamed_addr constant [11 x i8] c"^$*+?.([%-\00", align 1
@.str.21 = private unnamed_addr constant [20 x i8] c"pattern too complex\00", align 1
@.str.22 = private unnamed_addr constant [35 x i8] c"missing '[' after '%%f' in pattern\00", align 1
@.str.23 = private unnamed_addr constant [18 x i8] c"too many captures\00", align 1
@.str.24 = private unnamed_addr constant [24 x i8] c"invalid pattern capture\00", align 1
@.str.25 = private unnamed_addr constant [47 x i8] c"malformed pattern (missing arguments to '%%b')\00", align 1
@.str.26 = private unnamed_addr constant [35 x i8] c"malformed pattern (ends with '%%')\00", align 1
@.str.27 = private unnamed_addr constant [32 x i8] c"malformed pattern (missing ']')\00", align 1
@.str.28 = private unnamed_addr constant [27 x i8] c"invalid capture index %%%d\00", align 1
@.str.29 = private unnamed_addr constant [19 x i8] c"unfinished capture\00", align 1
@.str.30 = private unnamed_addr constant [9 x i8] c"no value\00", align 1
@.str.31 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@.str.32 = private unnamed_addr constant [5 x i8] c"-+0 \00", align 1
@.str.33 = private unnamed_addr constant [3 x i8] c"-0\00", align 1
@.str.34 = private unnamed_addr constant [4 x i8] c"-#0\00", align 1
@.str.35 = private unnamed_addr constant [3 x i8] c"ll\00", align 1
@.str.36 = private unnamed_addr constant [6 x i8] c"-+#0 \00", align 1
@.str.37 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.38 = private unnamed_addr constant [7 x i8] c"(null)\00", align 1
@.str.39 = private unnamed_addr constant [38 x i8] c"specifier '%%q' cannot have modifiers\00", align 1
@.str.40 = private unnamed_addr constant [22 x i8] c"string contains zeros\00", align 1
@.str.41 = private unnamed_addr constant [36 x i8] c"invalid conversion '%s' to 'format'\00", align 1
@.str.42 = private unnamed_addr constant [16 x i8] c"-+#0 123456789.\00", align 1
@.str.43 = private unnamed_addr constant [26 x i8] c"invalid format (too long)\00", align 1
@.str.44 = private unnamed_addr constant [39 x i8] c"invalid conversion specification: '%s'\00", align 1
@.str.45 = private unnamed_addr constant [7 x i8] c"0x%llx\00", align 1
@.str.46 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.str.47 = private unnamed_addr constant [26 x i8] c"value has no literal form\00", align 1
@.str.48 = private unnamed_addr constant [4 x i8] c"\\%d\00", align 1
@.str.49 = private unnamed_addr constant [6 x i8] c"\\%03d\00", align 1
@.str.50 = private unnamed_addr constant [7 x i8] c"1e9999\00", align 1
@.str.51 = private unnamed_addr constant [8 x i8] c"-1e9999\00", align 1
@.str.52 = private unnamed_addr constant [6 x i8] c"(0/0)\00", align 1
@.str.53 = private unnamed_addr constant [3 x i8] c"%a\00", align 1
@.str.54 = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.str.55 = private unnamed_addr constant [22 x i8] c"string/function/table\00", align 1
@.str.56 = private unnamed_addr constant [33 x i8] c"invalid replacement value (a %s)\00", align 1
@.str.57 = private unnamed_addr constant [42 x i8] c"invalid use of '%c' in replacement string\00", align 1
@.str.58 = private unnamed_addr constant [27 x i8] c"resulting string too large\00", align 1
@.str.59 = private unnamed_addr constant [17 x i8] c"integer overflow\00", align 1
@.str.60 = private unnamed_addr constant [18 x i8] c"unsigned overflow\00", align 1
@.str.61 = private unnamed_addr constant [30 x i8] c"string longer than given size\00", align 1
@.str.62 = private unnamed_addr constant [41 x i8] c"string length does not fit in given size\00", align 1
@nativeendian = internal constant %union.anon.0 { i32 1 }, align 4
@.str.63 = private unnamed_addr constant [35 x i8] c"invalid next option for option 'X'\00", align 1
@.str.64 = private unnamed_addr constant [41 x i8] c"format asks for alignment not power of 2\00", align 1
@.str.65 = private unnamed_addr constant [35 x i8] c"missing size for format option 'c'\00", align 1
@.str.66 = private unnamed_addr constant [27 x i8] c"invalid format option '%c'\00", align 1
@.str.67 = private unnamed_addr constant [40 x i8] c"integral size (%d) out of limits [1,%d]\00", align 1
@.str.68 = private unnamed_addr constant [23 x i8] c"variable-length format\00", align 1
@.str.69 = private unnamed_addr constant [24 x i8] c"format result too large\00", align 1
@.str.70 = private unnamed_addr constant [31 x i8] c"initial position out of string\00", align 1
@.str.71 = private unnamed_addr constant [22 x i8] c"data string too short\00", align 1
@.str.72 = private unnamed_addr constant [17 x i8] c"too many results\00", align 1
@.str.73 = private unnamed_addr constant [33 x i8] c"unfinished string for format 'z'\00", align 1
@.str.74 = private unnamed_addr constant [46 x i8] c"%d-byte integer does not fit into Lua Integer\00", align 1
@stringmetamethods = internal constant [10 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.76, ptr @arith_add }, %struct.luaL_Reg { ptr @.str.77, ptr @arith_sub }, %struct.luaL_Reg { ptr @.str.78, ptr @arith_mul }, %struct.luaL_Reg { ptr @.str.79, ptr @arith_mod }, %struct.luaL_Reg { ptr @.str.80, ptr @arith_pow }, %struct.luaL_Reg { ptr @.str.81, ptr @arith_div }, %struct.luaL_Reg { ptr @.str.82, ptr @arith_idiv }, %struct.luaL_Reg { ptr @.str.83, ptr @arith_unm }, %struct.luaL_Reg { ptr @.str.75, ptr null }, %struct.luaL_Reg zeroinitializer], align 16
@.str.75 = private unnamed_addr constant [8 x i8] c"__index\00", align 1
@.str.76 = private unnamed_addr constant [6 x i8] c"__add\00", align 1
@.str.77 = private unnamed_addr constant [6 x i8] c"__sub\00", align 1
@.str.78 = private unnamed_addr constant [6 x i8] c"__mul\00", align 1
@.str.79 = private unnamed_addr constant [6 x i8] c"__mod\00", align 1
@.str.80 = private unnamed_addr constant [6 x i8] c"__pow\00", align 1
@.str.81 = private unnamed_addr constant [6 x i8] c"__div\00", align 1
@.str.82 = private unnamed_addr constant [7 x i8] c"__idiv\00", align 1
@.str.83 = private unnamed_addr constant [6 x i8] c"__unm\00", align 1
@.str.84 = private unnamed_addr constant [33 x i8] c"attempt to %s a '%s' with a '%s'\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_string(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 17)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @strlib, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  call void @createmetatable(ptr noundef %6)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @createmetatable(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %3, i32 noundef 0, i32 noundef 9)
  %4 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %4, ptr noundef @stringmetamethods, i32 noundef 0)
  %5 = load ptr, ptr %2, align 8
  %6 = call ptr @lua_pushstring(ptr noundef %5, ptr noundef @.str.37)
  %7 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %7, i32 noundef -2)
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_setmetatable(ptr noundef %8, i32 noundef -2)
  %10 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %10, i32 noundef -2)
  %11 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %11, i32 noundef -2)
  %12 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %12, i32 noundef -2, ptr noundef @.str.75)
  %13 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %13, i32 noundef -2)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_byte(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = call ptr @luaL_checklstring(ptr noundef %11, i32 noundef 1, ptr noundef %4)
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = call i64 @luaL_optinteger(ptr noundef %13, i32 noundef 2, i64 noundef 1)
  store i64 %14, ptr %6, align 8
  %15 = load i64, ptr %6, align 8
  %16 = load i64, ptr %4, align 8
  %17 = call i64 @posrelatI(i64 noundef %15, i64 noundef %16)
  store i64 %17, ptr %7, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = load i64, ptr %6, align 8
  %20 = load i64, ptr %4, align 8
  %21 = call i64 @getendpos(ptr noundef %18, i32 noundef 3, i64 noundef %19, i64 noundef %20)
  store i64 %21, ptr %8, align 8
  %22 = load i64, ptr %7, align 8
  %23 = load i64, ptr %8, align 8
  %24 = icmp ugt i64 %22, %23
  br i1 %24, label %25, label %26

25:                                               ; preds = %1
  store i32 0, ptr %2, align 4
  br label %67

26:                                               ; preds = %1
  %27 = load i64, ptr %8, align 8
  %28 = load i64, ptr %7, align 8
  %29 = sub i64 %27, %28
  %30 = icmp uge i64 %29, 2147483647
  %31 = zext i1 %30 to i32
  %32 = icmp ne i32 %31, 0
  %33 = zext i1 %32 to i32
  %34 = sext i32 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %36, label %39

36:                                               ; preds = %26
  %37 = load ptr, ptr %3, align 8
  %38 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %37, ptr noundef @.str.17)
  store i32 %38, ptr %2, align 4
  br label %67

39:                                               ; preds = %26
  %40 = load i64, ptr %8, align 8
  %41 = load i64, ptr %7, align 8
  %42 = sub i64 %40, %41
  %43 = trunc i64 %42 to i32
  %44 = add nsw i32 %43, 1
  store i32 %44, ptr %9, align 4
  %45 = load ptr, ptr %3, align 8
  %46 = load i32, ptr %9, align 4
  call void @luaL_checkstack(ptr noundef %45, i32 noundef %46, ptr noundef @.str.17)
  store i32 0, ptr %10, align 4
  br label %47

47:                                               ; preds = %62, %39
  %48 = load i32, ptr %10, align 4
  %49 = load i32, ptr %9, align 4
  %50 = icmp slt i32 %48, %49
  br i1 %50, label %51, label %65

51:                                               ; preds = %47
  %52 = load ptr, ptr %3, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = load i64, ptr %7, align 8
  %55 = load i32, ptr %10, align 4
  %56 = sext i32 %55 to i64
  %57 = add i64 %54, %56
  %58 = sub i64 %57, 1
  %59 = getelementptr inbounds i8, ptr %53, i64 %58
  %60 = load i8, ptr %59, align 1
  %61 = zext i8 %60 to i64
  call void @lua_pushinteger(ptr noundef %52, i64 noundef %61)
  br label %62

62:                                               ; preds = %51
  %63 = load i32, ptr %10, align 4
  %64 = add nsw i32 %63, 1
  store i32 %64, ptr %10, align 4
  br label %47, !llvm.loop !6

65:                                               ; preds = %47
  %66 = load i32, ptr %9, align 4
  store i32 %66, ptr %2, align 4
  br label %67

67:                                               ; preds = %65, %36, %25
  %68 = load i32, ptr %2, align 4
  ret i32 %68
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_char(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca %struct.luaL_Buffer, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @lua_gettop(ptr noundef %8)
  store i32 %9, ptr %3, align 4
  %10 = load ptr, ptr %2, align 8
  %11 = load i32, ptr %3, align 4
  %12 = sext i32 %11 to i64
  %13 = call ptr @luaL_buffinitsize(ptr noundef %10, ptr noundef %5, i64 noundef %12)
  store ptr %13, ptr %6, align 8
  store i32 1, ptr %4, align 4
  br label %14

14:                                               ; preds = %44, %1
  %15 = load i32, ptr %4, align 4
  %16 = load i32, ptr %3, align 4
  %17 = icmp sle i32 %15, %16
  br i1 %17, label %18, label %47

18:                                               ; preds = %14
  %19 = load ptr, ptr %2, align 8
  %20 = load i32, ptr %4, align 4
  %21 = call i64 @luaL_checkinteger(ptr noundef %19, i32 noundef %20)
  store i64 %21, ptr %7, align 8
  %22 = load i64, ptr %7, align 8
  %23 = icmp ule i64 %22, 255
  %24 = zext i1 %23 to i32
  %25 = icmp ne i32 %24, 0
  %26 = zext i1 %25 to i32
  %27 = sext i32 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %34, label %29

29:                                               ; preds = %18
  %30 = load ptr, ptr %2, align 8
  %31 = load i32, ptr %4, align 4
  %32 = call i32 @luaL_argerror(ptr noundef %30, i32 noundef %31, ptr noundef @.str.18)
  %33 = icmp ne i32 %32, 0
  br label %34

34:                                               ; preds = %29, %18
  %35 = phi i1 [ true, %18 ], [ %33, %29 ]
  %36 = zext i1 %35 to i32
  %37 = load i64, ptr %7, align 8
  %38 = trunc i64 %37 to i8
  %39 = load ptr, ptr %6, align 8
  %40 = load i32, ptr %4, align 4
  %41 = sub nsw i32 %40, 1
  %42 = sext i32 %41 to i64
  %43 = getelementptr inbounds i8, ptr %39, i64 %42
  store i8 %38, ptr %43, align 1
  br label %44

44:                                               ; preds = %34
  %45 = load i32, ptr %4, align 4
  %46 = add nsw i32 %45, 1
  store i32 %46, ptr %4, align 4
  br label %14, !llvm.loop !8

47:                                               ; preds = %14
  %48 = load i32, ptr %3, align 4
  %49 = sext i32 %48 to i64
  call void @luaL_pushresultsize(ptr noundef %5, i64 noundef %49)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_dump(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca %struct.str_Writer, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_toboolean(ptr noundef %6, i32 noundef 2)
  store i32 %7, ptr %5, align 4
  %8 = load ptr, ptr %3, align 8
  call void @luaL_checktype(ptr noundef %8, i32 noundef 1, i32 noundef 6)
  %9 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %9, i32 noundef 1)
  %10 = getelementptr inbounds %struct.str_Writer, ptr %4, i32 0, i32 0
  store i32 0, ptr %10, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = load i32, ptr %5, align 4
  %13 = call i32 @lua_dump(ptr noundef %11, ptr noundef @writer, ptr noundef %4, i32 noundef %12)
  %14 = icmp ne i32 %13, 0
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %1
  %21 = load ptr, ptr %3, align 8
  %22 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %21, ptr noundef @.str.19)
  store i32 %22, ptr %2, align 4
  br label %25

23:                                               ; preds = %1
  %24 = getelementptr inbounds %struct.str_Writer, ptr %4, i32 0, i32 1
  call void @luaL_pushresult(ptr noundef %24)
  store i32 1, ptr %2, align 4
  br label %25

25:                                               ; preds = %23, %20
  %26 = load i32, ptr %2, align 4
  ret i32 %26
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_find(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @str_find_aux(ptr noundef %3, i32 noundef 1)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_format(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %struct.luaL_Buffer, align 8
  %11 = alloca [32 x i8], align 16
  %12 = alloca i32, align 4
  %13 = alloca ptr, align 8
  %14 = alloca i32, align 4
  %15 = alloca i64, align 8
  %16 = alloca double, align 8
  %17 = alloca ptr, align 8
  %18 = alloca i64, align 8
  %19 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = call i32 @lua_gettop(ptr noundef %20)
  store i32 %21, ptr %4, align 4
  store i32 1, ptr %5, align 4
  %22 = load ptr, ptr %3, align 8
  %23 = load i32, ptr %5, align 4
  %24 = call ptr @luaL_checklstring(ptr noundef %22, i32 noundef %23, ptr noundef %6)
  store ptr %24, ptr %7, align 8
  %25 = load ptr, ptr %7, align 8
  %26 = load i64, ptr %6, align 8
  %27 = getelementptr inbounds i8, ptr %25, i64 %26
  store ptr %27, ptr %8, align 8
  %28 = load ptr, ptr %3, align 8
  call void @luaL_buffinit(ptr noundef %28, ptr noundef %10)
  br label %29

29:                                               ; preds = %254, %1
  %30 = load ptr, ptr %7, align 8
  %31 = load ptr, ptr %8, align 8
  %32 = icmp ult ptr %30, %31
  br i1 %32, label %33, label %255

33:                                               ; preds = %29
  %34 = load ptr, ptr %7, align 8
  %35 = load i8, ptr %34, align 1
  %36 = sext i8 %35 to i32
  %37 = icmp ne i32 %36, 37
  br i1 %37, label %38, label %59

38:                                               ; preds = %33
  %39 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 2
  %40 = load i64, ptr %39, align 8
  %41 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 1
  %42 = load i64, ptr %41, align 8
  %43 = icmp ult i64 %40, %42
  br i1 %43, label %47, label %44

44:                                               ; preds = %38
  %45 = call ptr @luaL_prepbuffsize(ptr noundef %10, i64 noundef 1)
  %46 = icmp ne ptr %45, null
  br label %47

47:                                               ; preds = %44, %38
  %48 = phi i1 [ true, %38 ], [ %46, %44 ]
  %49 = zext i1 %48 to i32
  %50 = load ptr, ptr %7, align 8
  %51 = getelementptr inbounds i8, ptr %50, i32 1
  store ptr %51, ptr %7, align 8
  %52 = load i8, ptr %50, align 1
  %53 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 0
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 2
  %56 = load i64, ptr %55, align 8
  %57 = add i64 %56, 1
  store i64 %57, ptr %55, align 8
  %58 = getelementptr inbounds i8, ptr %54, i64 %56
  store i8 %52, ptr %58, align 1
  br label %254

59:                                               ; preds = %33
  %60 = load ptr, ptr %7, align 8
  %61 = getelementptr inbounds i8, ptr %60, i32 1
  store ptr %61, ptr %7, align 8
  %62 = load i8, ptr %61, align 1
  %63 = sext i8 %62 to i32
  %64 = icmp eq i32 %63, 37
  br i1 %64, label %65, label %86

65:                                               ; preds = %59
  %66 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 2
  %67 = load i64, ptr %66, align 8
  %68 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 1
  %69 = load i64, ptr %68, align 8
  %70 = icmp ult i64 %67, %69
  br i1 %70, label %74, label %71

71:                                               ; preds = %65
  %72 = call ptr @luaL_prepbuffsize(ptr noundef %10, i64 noundef 1)
  %73 = icmp ne ptr %72, null
  br label %74

74:                                               ; preds = %71, %65
  %75 = phi i1 [ true, %65 ], [ %73, %71 ]
  %76 = zext i1 %75 to i32
  %77 = load ptr, ptr %7, align 8
  %78 = getelementptr inbounds i8, ptr %77, i32 1
  store ptr %78, ptr %7, align 8
  %79 = load i8, ptr %77, align 1
  %80 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 0
  %81 = load ptr, ptr %80, align 8
  %82 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 2
  %83 = load i64, ptr %82, align 8
  %84 = add i64 %83, 1
  store i64 %84, ptr %82, align 8
  %85 = getelementptr inbounds i8, ptr %81, i64 %83
  store i8 %79, ptr %85, align 1
  br label %253

86:                                               ; preds = %59
  store i32 120, ptr %12, align 4
  %87 = load i32, ptr %12, align 4
  %88 = sext i32 %87 to i64
  %89 = call ptr @luaL_prepbuffsize(ptr noundef %10, i64 noundef %88)
  store ptr %89, ptr %13, align 8
  store i32 0, ptr %14, align 4
  %90 = load i32, ptr %5, align 4
  %91 = add nsw i32 %90, 1
  store i32 %91, ptr %5, align 4
  %92 = load i32, ptr %4, align 4
  %93 = icmp sgt i32 %91, %92
  br i1 %93, label %94, label %98

94:                                               ; preds = %86
  %95 = load ptr, ptr %3, align 8
  %96 = load i32, ptr %5, align 4
  %97 = call i32 @luaL_argerror(ptr noundef %95, i32 noundef %96, ptr noundef @.str.30)
  store i32 %97, ptr %2, align 4
  br label %256

98:                                               ; preds = %86
  %99 = load ptr, ptr %3, align 8
  %100 = load ptr, ptr %7, align 8
  %101 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %102 = call ptr @getformat(ptr noundef %99, ptr noundef %100, ptr noundef %101)
  store ptr %102, ptr %7, align 8
  %103 = load ptr, ptr %7, align 8
  %104 = getelementptr inbounds i8, ptr %103, i32 1
  store ptr %104, ptr %7, align 8
  %105 = load i8, ptr %103, align 1
  %106 = sext i8 %105 to i32
  switch i32 %106, label %243 [
    i32 99, label %107
    i32 100, label %119
    i32 105, label %119
    i32 117, label %120
    i32 111, label %121
    i32 120, label %121
    i32 88, label %121
    i32 97, label %136
    i32 65, label %136
    i32 102, label %149
    i32 101, label %153
    i32 69, label %153
    i32 103, label %153
    i32 71, label %153
    i32 112, label %166
    i32 113, label %186
    i32 115, label %197
  ]

107:                                              ; preds = %98
  %108 = load ptr, ptr %3, align 8
  %109 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @checkformat(ptr noundef %108, ptr noundef %109, ptr noundef @.str.31, i32 noundef 0)
  %110 = load ptr, ptr %13, align 8
  %111 = load i32, ptr %12, align 4
  %112 = sext i32 %111 to i64
  %113 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %114 = load ptr, ptr %3, align 8
  %115 = load i32, ptr %5, align 4
  %116 = call i64 @luaL_checkinteger(ptr noundef %114, i32 noundef %115)
  %117 = trunc i64 %116 to i32
  %118 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %110, i64 noundef %112, ptr noundef %113, i32 noundef %117) #6
  store i32 %118, ptr %14, align 4
  br label %247

119:                                              ; preds = %98, %98
  store ptr @.str.32, ptr %9, align 8
  br label %122

120:                                              ; preds = %98
  store ptr @.str.33, ptr %9, align 8
  br label %122

121:                                              ; preds = %98, %98, %98
  store ptr @.str.34, ptr %9, align 8
  br label %122

122:                                              ; preds = %121, %120, %119
  %123 = load ptr, ptr %3, align 8
  %124 = load i32, ptr %5, align 4
  %125 = call i64 @luaL_checkinteger(ptr noundef %123, i32 noundef %124)
  store i64 %125, ptr %15, align 8
  %126 = load ptr, ptr %3, align 8
  %127 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %128 = load ptr, ptr %9, align 8
  call void @checkformat(ptr noundef %126, ptr noundef %127, ptr noundef %128, i32 noundef 1)
  %129 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @addlenmod(ptr noundef %129, ptr noundef @.str.35)
  %130 = load ptr, ptr %13, align 8
  %131 = load i32, ptr %12, align 4
  %132 = sext i32 %131 to i64
  %133 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %134 = load i64, ptr %15, align 8
  %135 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %130, i64 noundef %132, ptr noundef %133, i64 noundef %134) #6
  store i32 %135, ptr %14, align 4
  br label %247

136:                                              ; preds = %98, %98
  %137 = load ptr, ptr %3, align 8
  %138 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @checkformat(ptr noundef %137, ptr noundef %138, ptr noundef @.str.36, i32 noundef 1)
  %139 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @addlenmod(ptr noundef %139, ptr noundef @.str.37)
  %140 = load ptr, ptr %3, align 8
  %141 = load ptr, ptr %13, align 8
  %142 = load i32, ptr %12, align 4
  %143 = sext i32 %142 to i64
  %144 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %145 = load ptr, ptr %3, align 8
  %146 = load i32, ptr %5, align 4
  %147 = call double @luaL_checknumber(ptr noundef %145, i32 noundef %146)
  %148 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %141, i64 noundef %143, ptr noundef %144, double noundef %147) #6
  store i32 %148, ptr %14, align 4
  br label %247

149:                                              ; preds = %98
  store i32 418, ptr %12, align 4
  %150 = load i32, ptr %12, align 4
  %151 = sext i32 %150 to i64
  %152 = call ptr @luaL_prepbuffsize(ptr noundef %10, i64 noundef %151)
  store ptr %152, ptr %13, align 8
  br label %153

153:                                              ; preds = %98, %98, %98, %98, %149
  %154 = load ptr, ptr %3, align 8
  %155 = load i32, ptr %5, align 4
  %156 = call double @luaL_checknumber(ptr noundef %154, i32 noundef %155)
  store double %156, ptr %16, align 8
  %157 = load ptr, ptr %3, align 8
  %158 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @checkformat(ptr noundef %157, ptr noundef %158, ptr noundef @.str.36, i32 noundef 1)
  %159 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @addlenmod(ptr noundef %159, ptr noundef @.str.37)
  %160 = load ptr, ptr %13, align 8
  %161 = load i32, ptr %12, align 4
  %162 = sext i32 %161 to i64
  %163 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %164 = load double, ptr %16, align 8
  %165 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %160, i64 noundef %162, ptr noundef %163, double noundef %164) #6
  store i32 %165, ptr %14, align 4
  br label %247

166:                                              ; preds = %98
  %167 = load ptr, ptr %3, align 8
  %168 = load i32, ptr %5, align 4
  %169 = call ptr @lua_topointer(ptr noundef %167, i32 noundef %168)
  store ptr %169, ptr %17, align 8
  %170 = load ptr, ptr %3, align 8
  %171 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @checkformat(ptr noundef %170, ptr noundef %171, ptr noundef @.str.31, i32 noundef 0)
  %172 = load ptr, ptr %17, align 8
  %173 = icmp eq ptr %172, null
  br i1 %173, label %174, label %179

174:                                              ; preds = %166
  store ptr @.str.38, ptr %17, align 8
  %175 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %176 = call i64 @strlen(ptr noundef %175) #7
  %177 = sub i64 %176, 1
  %178 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 %177
  store i8 115, ptr %178, align 1
  br label %179

179:                                              ; preds = %174, %166
  %180 = load ptr, ptr %13, align 8
  %181 = load i32, ptr %12, align 4
  %182 = sext i32 %181 to i64
  %183 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %184 = load ptr, ptr %17, align 8
  %185 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %180, i64 noundef %182, ptr noundef %183, ptr noundef %184) #6
  store i32 %185, ptr %14, align 4
  br label %247

186:                                              ; preds = %98
  %187 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 2
  %188 = load i8, ptr %187, align 2
  %189 = sext i8 %188 to i32
  %190 = icmp ne i32 %189, 0
  br i1 %190, label %191, label %194

191:                                              ; preds = %186
  %192 = load ptr, ptr %3, align 8
  %193 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %192, ptr noundef @.str.39)
  store i32 %193, ptr %2, align 4
  br label %256

194:                                              ; preds = %186
  %195 = load ptr, ptr %3, align 8
  %196 = load i32, ptr %5, align 4
  call void @addliteral(ptr noundef %195, ptr noundef %10, i32 noundef %196)
  br label %247

197:                                              ; preds = %98
  %198 = load ptr, ptr %3, align 8
  %199 = load i32, ptr %5, align 4
  %200 = call ptr @luaL_tolstring(ptr noundef %198, i32 noundef %199, ptr noundef %18)
  store ptr %200, ptr %19, align 8
  %201 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 2
  %202 = load i8, ptr %201, align 2
  %203 = sext i8 %202 to i32
  %204 = icmp eq i32 %203, 0
  br i1 %204, label %205, label %206

205:                                              ; preds = %197
  call void @luaL_addvalue(ptr noundef %10)
  br label %242

206:                                              ; preds = %197
  %207 = load i64, ptr %18, align 8
  %208 = load ptr, ptr %19, align 8
  %209 = call i64 @strlen(ptr noundef %208) #7
  %210 = icmp eq i64 %207, %209
  %211 = zext i1 %210 to i32
  %212 = icmp ne i32 %211, 0
  %213 = zext i1 %212 to i32
  %214 = sext i32 %213 to i64
  %215 = icmp ne i64 %214, 0
  br i1 %215, label %221, label %216

216:                                              ; preds = %206
  %217 = load ptr, ptr %3, align 8
  %218 = load i32, ptr %5, align 4
  %219 = call i32 @luaL_argerror(ptr noundef %217, i32 noundef %218, ptr noundef @.str.40)
  %220 = icmp ne i32 %219, 0
  br label %221

221:                                              ; preds = %216, %206
  %222 = phi i1 [ true, %206 ], [ %220, %216 ]
  %223 = zext i1 %222 to i32
  %224 = load ptr, ptr %3, align 8
  %225 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  call void @checkformat(ptr noundef %224, ptr noundef %225, ptr noundef @.str.31, i32 noundef 1)
  %226 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %227 = call ptr @strchr(ptr noundef %226, i32 noundef 46) #7
  %228 = icmp eq ptr %227, null
  br i1 %228, label %229, label %233

229:                                              ; preds = %221
  %230 = load i64, ptr %18, align 8
  %231 = icmp uge i64 %230, 100
  br i1 %231, label %232, label %233

232:                                              ; preds = %229
  call void @luaL_addvalue(ptr noundef %10)
  br label %241

233:                                              ; preds = %229, %221
  %234 = load ptr, ptr %13, align 8
  %235 = load i32, ptr %12, align 4
  %236 = sext i32 %235 to i64
  %237 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %238 = load ptr, ptr %19, align 8
  %239 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %234, i64 noundef %236, ptr noundef %237, ptr noundef %238) #6
  store i32 %239, ptr %14, align 4
  %240 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %240, i32 noundef -2)
  br label %241

241:                                              ; preds = %233, %232
  br label %242

242:                                              ; preds = %241, %205
  br label %247

243:                                              ; preds = %98
  %244 = load ptr, ptr %3, align 8
  %245 = getelementptr inbounds [32 x i8], ptr %11, i64 0, i64 0
  %246 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %244, ptr noundef @.str.41, ptr noundef %245)
  store i32 %246, ptr %2, align 4
  br label %256

247:                                              ; preds = %242, %194, %179, %153, %136, %122, %107
  %248 = load i32, ptr %14, align 4
  %249 = sext i32 %248 to i64
  %250 = getelementptr inbounds %struct.luaL_Buffer, ptr %10, i32 0, i32 2
  %251 = load i64, ptr %250, align 8
  %252 = add i64 %251, %249
  store i64 %252, ptr %250, align 8
  br label %253

253:                                              ; preds = %247, %74
  br label %254

254:                                              ; preds = %253, %47
  br label %29, !llvm.loop !9

255:                                              ; preds = %29
  call void @luaL_pushresult(ptr noundef %10)
  store i32 1, ptr %2, align 4
  br label %256

256:                                              ; preds = %255, %243, %191, %94
  %257 = load i32, ptr %2, align 4
  ret i32 %257
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @gmatch(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @luaL_checklstring(ptr noundef %9, i32 noundef 1, ptr noundef %3)
  store ptr %10, ptr %5, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = call ptr @luaL_checklstring(ptr noundef %11, i32 noundef 2, ptr noundef %4)
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = call i64 @luaL_optinteger(ptr noundef %13, i32 noundef 3, i64 noundef 1)
  %15 = load i64, ptr %3, align 8
  %16 = call i64 @posrelatI(i64 noundef %14, i64 noundef %15)
  %17 = sub i64 %16, 1
  store i64 %17, ptr %7, align 8
  %18 = load ptr, ptr %2, align 8
  call void @lua_settop(ptr noundef %18, i32 noundef 2)
  %19 = load ptr, ptr %2, align 8
  %20 = call ptr @lua_newuserdatauv(ptr noundef %19, i64 noundef 576, i32 noundef 0)
  store ptr %20, ptr %8, align 8
  %21 = load i64, ptr %7, align 8
  %22 = load i64, ptr %3, align 8
  %23 = icmp ugt i64 %21, %22
  br i1 %23, label %24, label %27

24:                                               ; preds = %1
  %25 = load i64, ptr %3, align 8
  %26 = add i64 %25, 1
  store i64 %26, ptr %7, align 8
  br label %27

27:                                               ; preds = %24, %1
  %28 = load ptr, ptr %8, align 8
  %29 = getelementptr inbounds %struct.GMatchState, ptr %28, i32 0, i32 3
  %30 = load ptr, ptr %2, align 8
  %31 = load ptr, ptr %5, align 8
  %32 = load i64, ptr %3, align 8
  %33 = load ptr, ptr %6, align 8
  %34 = load i64, ptr %4, align 8
  call void @prepstate(ptr noundef %29, ptr noundef %30, ptr noundef %31, i64 noundef %32, ptr noundef %33, i64 noundef %34)
  %35 = load ptr, ptr %5, align 8
  %36 = load i64, ptr %7, align 8
  %37 = getelementptr inbounds i8, ptr %35, i64 %36
  %38 = load ptr, ptr %8, align 8
  %39 = getelementptr inbounds %struct.GMatchState, ptr %38, i32 0, i32 0
  store ptr %37, ptr %39, align 8
  %40 = load ptr, ptr %6, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = getelementptr inbounds %struct.GMatchState, ptr %41, i32 0, i32 1
  store ptr %40, ptr %42, align 8
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds %struct.GMatchState, ptr %43, i32 0, i32 2
  store ptr null, ptr %44, align 8
  %45 = load ptr, ptr %2, align 8
  call void @lua_pushcclosure(ptr noundef %45, ptr noundef @gmatch_aux, i32 noundef 3)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_gsub(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i64, align 8
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca i32, align 4
  %13 = alloca %struct.MatchState, align 8
  %14 = alloca %struct.luaL_Buffer, align 8
  %15 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %16 = load ptr, ptr %2, align 8
  %17 = call ptr @luaL_checklstring(ptr noundef %16, i32 noundef 1, ptr noundef %3)
  store ptr %17, ptr %5, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = call ptr @luaL_checklstring(ptr noundef %18, i32 noundef 2, ptr noundef %4)
  store ptr %19, ptr %6, align 8
  store ptr null, ptr %7, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = call i32 @lua_type(ptr noundef %20, i32 noundef 3)
  store i32 %21, ptr %8, align 4
  %22 = load ptr, ptr %2, align 8
  %23 = load i64, ptr %3, align 8
  %24 = add i64 %23, 1
  %25 = call i64 @luaL_optinteger(ptr noundef %22, i32 noundef 4, i64 noundef %24)
  store i64 %25, ptr %9, align 8
  %26 = load ptr, ptr %6, align 8
  %27 = load i8, ptr %26, align 1
  %28 = sext i8 %27 to i32
  %29 = icmp eq i32 %28, 94
  %30 = zext i1 %29 to i32
  store i32 %30, ptr %10, align 4
  store i64 0, ptr %11, align 8
  store i32 0, ptr %12, align 4
  %31 = load i32, ptr %8, align 4
  %32 = icmp eq i32 %31, 3
  br i1 %32, label %42, label %33

33:                                               ; preds = %1
  %34 = load i32, ptr %8, align 4
  %35 = icmp eq i32 %34, 4
  br i1 %35, label %42, label %36

36:                                               ; preds = %33
  %37 = load i32, ptr %8, align 4
  %38 = icmp eq i32 %37, 6
  br i1 %38, label %42, label %39

39:                                               ; preds = %36
  %40 = load i32, ptr %8, align 4
  %41 = icmp eq i32 %40, 5
  br label %42

42:                                               ; preds = %39, %36, %33, %1
  %43 = phi i1 [ true, %36 ], [ true, %33 ], [ true, %1 ], [ %41, %39 ]
  %44 = zext i1 %43 to i32
  %45 = icmp ne i32 %44, 0
  %46 = zext i1 %45 to i32
  %47 = sext i32 %46 to i64
  %48 = icmp ne i64 %47, 0
  br i1 %48, label %53, label %49

49:                                               ; preds = %42
  %50 = load ptr, ptr %2, align 8
  %51 = call i32 @luaL_typeerror(ptr noundef %50, i32 noundef 3, ptr noundef @.str.55)
  %52 = icmp ne i32 %51, 0
  br label %53

53:                                               ; preds = %49, %42
  %54 = phi i1 [ true, %42 ], [ %52, %49 ]
  %55 = zext i1 %54 to i32
  %56 = load ptr, ptr %2, align 8
  call void @luaL_buffinit(ptr noundef %56, ptr noundef %14)
  %57 = load i32, ptr %10, align 4
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %64

59:                                               ; preds = %53
  %60 = load ptr, ptr %6, align 8
  %61 = getelementptr inbounds i8, ptr %60, i32 1
  store ptr %61, ptr %6, align 8
  %62 = load i64, ptr %4, align 8
  %63 = add i64 %62, -1
  store i64 %63, ptr %4, align 8
  br label %64

64:                                               ; preds = %59, %53
  %65 = load ptr, ptr %2, align 8
  %66 = load ptr, ptr %5, align 8
  %67 = load i64, ptr %3, align 8
  %68 = load ptr, ptr %6, align 8
  %69 = load i64, ptr %4, align 8
  call void @prepstate(ptr noundef %13, ptr noundef %65, ptr noundef %66, i64 noundef %67, ptr noundef %68, i64 noundef %69)
  br label %70

70:                                               ; preds = %125, %64
  %71 = load i64, ptr %11, align 8
  %72 = load i64, ptr %9, align 8
  %73 = icmp slt i64 %71, %72
  br i1 %73, label %74, label %126

74:                                               ; preds = %70
  call void @reprepstate(ptr noundef %13)
  %75 = load ptr, ptr %5, align 8
  %76 = load ptr, ptr %6, align 8
  %77 = call ptr @match(ptr noundef %13, ptr noundef %75, ptr noundef %76)
  store ptr %77, ptr %15, align 8
  %78 = icmp ne ptr %77, null
  br i1 %78, label %79, label %93

79:                                               ; preds = %74
  %80 = load ptr, ptr %15, align 8
  %81 = load ptr, ptr %7, align 8
  %82 = icmp ne ptr %80, %81
  br i1 %82, label %83, label %93

83:                                               ; preds = %79
  %84 = load i64, ptr %11, align 8
  %85 = add nsw i64 %84, 1
  store i64 %85, ptr %11, align 8
  %86 = load ptr, ptr %5, align 8
  %87 = load ptr, ptr %15, align 8
  %88 = load i32, ptr %8, align 4
  %89 = call i32 @add_value(ptr noundef %13, ptr noundef %14, ptr noundef %86, ptr noundef %87, i32 noundef %88)
  %90 = load i32, ptr %12, align 4
  %91 = or i32 %89, %90
  store i32 %91, ptr %12, align 4
  %92 = load ptr, ptr %15, align 8
  store ptr %92, ptr %7, align 8
  store ptr %92, ptr %5, align 8
  br label %121

93:                                               ; preds = %79, %74
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %struct.MatchState, ptr %13, i32 0, i32 1
  %96 = load ptr, ptr %95, align 8
  %97 = icmp ult ptr %94, %96
  br i1 %97, label %98, label %119

98:                                               ; preds = %93
  %99 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 2
  %100 = load i64, ptr %99, align 8
  %101 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 1
  %102 = load i64, ptr %101, align 8
  %103 = icmp ult i64 %100, %102
  br i1 %103, label %107, label %104

104:                                              ; preds = %98
  %105 = call ptr @luaL_prepbuffsize(ptr noundef %14, i64 noundef 1)
  %106 = icmp ne ptr %105, null
  br label %107

107:                                              ; preds = %104, %98
  %108 = phi i1 [ true, %98 ], [ %106, %104 ]
  %109 = zext i1 %108 to i32
  %110 = load ptr, ptr %5, align 8
  %111 = getelementptr inbounds i8, ptr %110, i32 1
  store ptr %111, ptr %5, align 8
  %112 = load i8, ptr %110, align 1
  %113 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 0
  %114 = load ptr, ptr %113, align 8
  %115 = getelementptr inbounds %struct.luaL_Buffer, ptr %14, i32 0, i32 2
  %116 = load i64, ptr %115, align 8
  %117 = add i64 %116, 1
  store i64 %117, ptr %115, align 8
  %118 = getelementptr inbounds i8, ptr %114, i64 %116
  store i8 %112, ptr %118, align 1
  br label %120

119:                                              ; preds = %93
  br label %126

120:                                              ; preds = %107
  br label %121

121:                                              ; preds = %120, %83
  %122 = load i32, ptr %10, align 4
  %123 = icmp ne i32 %122, 0
  br i1 %123, label %124, label %125

124:                                              ; preds = %121
  br label %126

125:                                              ; preds = %121
  br label %70, !llvm.loop !10

126:                                              ; preds = %124, %119, %70
  %127 = load i32, ptr %12, align 4
  %128 = icmp ne i32 %127, 0
  br i1 %128, label %131, label %129

129:                                              ; preds = %126
  %130 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %130, i32 noundef 1)
  br label %139

131:                                              ; preds = %126
  %132 = load ptr, ptr %5, align 8
  %133 = getelementptr inbounds %struct.MatchState, ptr %13, i32 0, i32 1
  %134 = load ptr, ptr %133, align 8
  %135 = load ptr, ptr %5, align 8
  %136 = ptrtoint ptr %134 to i64
  %137 = ptrtoint ptr %135 to i64
  %138 = sub i64 %136, %137
  call void @luaL_addlstring(ptr noundef %14, ptr noundef %132, i64 noundef %138)
  call void @luaL_pushresult(ptr noundef %14)
  br label %139

139:                                              ; preds = %131, %129
  %140 = load ptr, ptr %2, align 8
  %141 = load i64, ptr %11, align 8
  call void @lua_pushinteger(ptr noundef %140, i64 noundef %141)
  ret i32 2
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_len(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call ptr @luaL_checklstring(ptr noundef %4, i32 noundef 1, ptr noundef %3)
  %6 = load ptr, ptr %2, align 8
  %7 = load i64, ptr %3, align 8
  call void @lua_pushinteger(ptr noundef %6, i64 noundef %7)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_lower(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca %struct.luaL_Buffer, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 1, ptr noundef %3)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call ptr @luaL_buffinitsize(ptr noundef %10, ptr noundef %5, i64 noundef %11)
  store ptr %12, ptr %7, align 8
  store i64 0, ptr %4, align 8
  br label %13

13:                                               ; preds = %28, %1
  %14 = load i64, ptr %4, align 8
  %15 = load i64, ptr %3, align 8
  %16 = icmp ult i64 %14, %15
  br i1 %16, label %17, label %31

17:                                               ; preds = %13
  %18 = load ptr, ptr %6, align 8
  %19 = load i64, ptr %4, align 8
  %20 = getelementptr inbounds i8, ptr %18, i64 %19
  %21 = load i8, ptr %20, align 1
  %22 = zext i8 %21 to i32
  %23 = call i32 @tolower(i32 noundef %22) #7
  %24 = trunc i32 %23 to i8
  %25 = load ptr, ptr %7, align 8
  %26 = load i64, ptr %4, align 8
  %27 = getelementptr inbounds i8, ptr %25, i64 %26
  store i8 %24, ptr %27, align 1
  br label %28

28:                                               ; preds = %17
  %29 = load i64, ptr %4, align 8
  %30 = add i64 %29, 1
  store i64 %30, ptr %4, align 8
  br label %13, !llvm.loop !11

31:                                               ; preds = %13
  %32 = load i64, ptr %3, align 8
  call void @luaL_pushresultsize(ptr noundef %5, i64 noundef %32)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_match(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @str_find_aux(ptr noundef %3, i32 noundef 0)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_rep(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca %struct.luaL_Buffer, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @luaL_checklstring(ptr noundef %12, i32 noundef 1, ptr noundef %4)
  store ptr %13, ptr %6, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = call i64 @luaL_checkinteger(ptr noundef %14, i32 noundef 2)
  store i64 %15, ptr %7, align 8
  %16 = load ptr, ptr %3, align 8
  %17 = call ptr @luaL_optlstring(ptr noundef %16, i32 noundef 3, ptr noundef @.str.37, ptr noundef %5)
  store ptr %17, ptr %8, align 8
  %18 = load i64, ptr %7, align 8
  %19 = icmp sle i64 %18, 0
  br i1 %19, label %20, label %23

20:                                               ; preds = %1
  %21 = load ptr, ptr %3, align 8
  %22 = call ptr @lua_pushstring(ptr noundef %21, ptr noundef @.str.37)
  br label %88

23:                                               ; preds = %1
  %24 = load i64, ptr %4, align 8
  %25 = load i64, ptr %5, align 8
  %26 = add i64 %24, %25
  %27 = load i64, ptr %4, align 8
  %28 = icmp ult i64 %26, %27
  br i1 %28, label %36, label %29

29:                                               ; preds = %23
  %30 = load i64, ptr %4, align 8
  %31 = load i64, ptr %5, align 8
  %32 = add i64 %30, %31
  %33 = load i64, ptr %7, align 8
  %34 = udiv i64 2147483647, %33
  %35 = icmp ugt i64 %32, %34
  br label %36

36:                                               ; preds = %29, %23
  %37 = phi i1 [ true, %23 ], [ %35, %29 ]
  %38 = zext i1 %37 to i32
  %39 = icmp ne i32 %38, 0
  %40 = zext i1 %39 to i32
  %41 = sext i32 %40 to i64
  %42 = icmp ne i64 %41, 0
  br i1 %42, label %43, label %46

43:                                               ; preds = %36
  %44 = load ptr, ptr %3, align 8
  %45 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %44, ptr noundef @.str.58)
  store i32 %45, ptr %2, align 4
  br label %89

46:                                               ; preds = %36
  %47 = load i64, ptr %7, align 8
  %48 = load i64, ptr %4, align 8
  %49 = mul i64 %47, %48
  %50 = load i64, ptr %7, align 8
  %51 = sub nsw i64 %50, 1
  %52 = load i64, ptr %5, align 8
  %53 = mul i64 %51, %52
  %54 = add i64 %49, %53
  store i64 %54, ptr %9, align 8
  %55 = load ptr, ptr %3, align 8
  %56 = load i64, ptr %9, align 8
  %57 = call ptr @luaL_buffinitsize(ptr noundef %55, ptr noundef %10, i64 noundef %56)
  store ptr %57, ptr %11, align 8
  br label %58

58:                                               ; preds = %80, %46
  %59 = load i64, ptr %7, align 8
  %60 = add nsw i64 %59, -1
  store i64 %60, ptr %7, align 8
  %61 = icmp sgt i64 %59, 1
  br i1 %61, label %62, label %81

62:                                               ; preds = %58
  %63 = load ptr, ptr %11, align 8
  %64 = load ptr, ptr %6, align 8
  %65 = load i64, ptr %4, align 8
  %66 = mul i64 %65, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %63, ptr align 1 %64, i64 %66, i1 false)
  %67 = load i64, ptr %4, align 8
  %68 = load ptr, ptr %11, align 8
  %69 = getelementptr inbounds i8, ptr %68, i64 %67
  store ptr %69, ptr %11, align 8
  %70 = load i64, ptr %5, align 8
  %71 = icmp ugt i64 %70, 0
  br i1 %71, label %72, label %80

72:                                               ; preds = %62
  %73 = load ptr, ptr %11, align 8
  %74 = load ptr, ptr %8, align 8
  %75 = load i64, ptr %5, align 8
  %76 = mul i64 %75, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %73, ptr align 1 %74, i64 %76, i1 false)
  %77 = load i64, ptr %5, align 8
  %78 = load ptr, ptr %11, align 8
  %79 = getelementptr inbounds i8, ptr %78, i64 %77
  store ptr %79, ptr %11, align 8
  br label %80

80:                                               ; preds = %72, %62
  br label %58, !llvm.loop !12

81:                                               ; preds = %58
  %82 = load ptr, ptr %11, align 8
  %83 = load ptr, ptr %6, align 8
  %84 = load i64, ptr %4, align 8
  %85 = mul i64 %84, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %82, ptr align 1 %83, i64 %85, i1 false)
  %86 = load i64, ptr %9, align 8
  call void @luaL_pushresultsize(ptr noundef %10, i64 noundef %86)
  br label %87

87:                                               ; preds = %81
  br label %88

88:                                               ; preds = %87, %20
  store i32 1, ptr %2, align 4
  br label %89

89:                                               ; preds = %88, %43
  %90 = load i32, ptr %2, align 4
  ret i32 %90
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_reverse(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca %struct.luaL_Buffer, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 1, ptr noundef %3)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call ptr @luaL_buffinitsize(ptr noundef %10, ptr noundef %5, i64 noundef %11)
  store ptr %12, ptr %7, align 8
  store i64 0, ptr %4, align 8
  br label %13

13:                                               ; preds = %28, %1
  %14 = load i64, ptr %4, align 8
  %15 = load i64, ptr %3, align 8
  %16 = icmp ult i64 %14, %15
  br i1 %16, label %17, label %31

17:                                               ; preds = %13
  %18 = load ptr, ptr %6, align 8
  %19 = load i64, ptr %3, align 8
  %20 = load i64, ptr %4, align 8
  %21 = sub i64 %19, %20
  %22 = sub i64 %21, 1
  %23 = getelementptr inbounds i8, ptr %18, i64 %22
  %24 = load i8, ptr %23, align 1
  %25 = load ptr, ptr %7, align 8
  %26 = load i64, ptr %4, align 8
  %27 = getelementptr inbounds i8, ptr %25, i64 %26
  store i8 %24, ptr %27, align 1
  br label %28

28:                                               ; preds = %17
  %29 = load i64, ptr %4, align 8
  %30 = add i64 %29, 1
  store i64 %30, ptr %4, align 8
  br label %13, !llvm.loop !13

31:                                               ; preds = %13
  %32 = load i64, ptr %3, align 8
  call void @luaL_pushresultsize(ptr noundef %5, i64 noundef %32)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_sub(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 1, ptr noundef %3)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call i64 @luaL_checkinteger(ptr noundef %9, i32 noundef 2)
  %11 = load i64, ptr %3, align 8
  %12 = call i64 @posrelatI(i64 noundef %10, i64 noundef %11)
  store i64 %12, ptr %5, align 8
  %13 = load ptr, ptr %2, align 8
  %14 = load i64, ptr %3, align 8
  %15 = call i64 @getendpos(ptr noundef %13, i32 noundef 3, i64 noundef -1, i64 noundef %14)
  store i64 %15, ptr %6, align 8
  %16 = load i64, ptr %5, align 8
  %17 = load i64, ptr %6, align 8
  %18 = icmp ule i64 %16, %17
  br i1 %18, label %19, label %30

19:                                               ; preds = %1
  %20 = load ptr, ptr %2, align 8
  %21 = load ptr, ptr %4, align 8
  %22 = load i64, ptr %5, align 8
  %23 = getelementptr inbounds i8, ptr %21, i64 %22
  %24 = getelementptr inbounds i8, ptr %23, i64 -1
  %25 = load i64, ptr %6, align 8
  %26 = load i64, ptr %5, align 8
  %27 = sub i64 %25, %26
  %28 = add i64 %27, 1
  %29 = call ptr @lua_pushlstring(ptr noundef %20, ptr noundef %24, i64 noundef %28)
  br label %33

30:                                               ; preds = %1
  %31 = load ptr, ptr %2, align 8
  %32 = call ptr @lua_pushstring(ptr noundef %31, ptr noundef @.str.37)
  br label %33

33:                                               ; preds = %30, %19
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_upper(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca %struct.luaL_Buffer, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 1, ptr noundef %3)
  store ptr %9, ptr %6, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = load i64, ptr %3, align 8
  %12 = call ptr @luaL_buffinitsize(ptr noundef %10, ptr noundef %5, i64 noundef %11)
  store ptr %12, ptr %7, align 8
  store i64 0, ptr %4, align 8
  br label %13

13:                                               ; preds = %28, %1
  %14 = load i64, ptr %4, align 8
  %15 = load i64, ptr %3, align 8
  %16 = icmp ult i64 %14, %15
  br i1 %16, label %17, label %31

17:                                               ; preds = %13
  %18 = load ptr, ptr %6, align 8
  %19 = load i64, ptr %4, align 8
  %20 = getelementptr inbounds i8, ptr %18, i64 %19
  %21 = load i8, ptr %20, align 1
  %22 = zext i8 %21 to i32
  %23 = call i32 @toupper(i32 noundef %22) #7
  %24 = trunc i32 %23 to i8
  %25 = load ptr, ptr %7, align 8
  %26 = load i64, ptr %4, align 8
  %27 = getelementptr inbounds i8, ptr %25, i64 %26
  store i8 %24, ptr %27, align 1
  br label %28

28:                                               ; preds = %17
  %29 = load i64, ptr %4, align 8
  %30 = add i64 %29, 1
  store i64 %30, ptr %4, align 8
  br label %13, !llvm.loop !14

31:                                               ; preds = %13
  %32 = load i64, ptr %3, align 8
  call void @luaL_pushresultsize(ptr noundef %5, i64 noundef %32)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_pack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.luaL_Buffer, align 8
  %4 = alloca %struct.Header, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca i64, align 8
  %13 = alloca i64, align 8
  %14 = alloca float, align 4
  %15 = alloca ptr, align 8
  %16 = alloca double, align 8
  %17 = alloca ptr, align 8
  %18 = alloca double, align 8
  %19 = alloca ptr, align 8
  %20 = alloca i64, align 8
  %21 = alloca ptr, align 8
  %22 = alloca i64, align 8
  %23 = alloca ptr, align 8
  %24 = alloca i64, align 8
  %25 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %26 = load ptr, ptr %2, align 8
  %27 = call ptr @luaL_checklstring(ptr noundef %26, i32 noundef 1, ptr noundef null)
  store ptr %27, ptr %5, align 8
  store i32 1, ptr %6, align 4
  store i64 0, ptr %7, align 8
  %28 = load ptr, ptr %2, align 8
  call void @initheader(ptr noundef %28, ptr noundef %4)
  %29 = load ptr, ptr %2, align 8
  call void @lua_pushnil(ptr noundef %29)
  %30 = load ptr, ptr %2, align 8
  call void @luaL_buffinit(ptr noundef %30, ptr noundef %3)
  br label %31

31:                                               ; preds = %335, %1
  %32 = load ptr, ptr %5, align 8
  %33 = load i8, ptr %32, align 1
  %34 = sext i8 %33 to i32
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %36, label %336

36:                                               ; preds = %31
  %37 = load i64, ptr %7, align 8
  %38 = call i32 @getdetails(ptr noundef %4, i64 noundef %37, ptr noundef %5, ptr noundef %8, ptr noundef %9)
  store i32 %38, ptr %10, align 4
  %39 = load i32, ptr %9, align 4
  %40 = load i32, ptr %8, align 4
  %41 = add nsw i32 %39, %40
  %42 = sext i32 %41 to i64
  %43 = load i64, ptr %7, align 8
  %44 = add i64 %43, %42
  store i64 %44, ptr %7, align 8
  br label %45

45:                                               ; preds = %58, %36
  %46 = load i32, ptr %9, align 4
  %47 = add nsw i32 %46, -1
  store i32 %47, ptr %9, align 4
  %48 = icmp sgt i32 %46, 0
  br i1 %48, label %49, label %67

49:                                               ; preds = %45
  %50 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %51 = load i64, ptr %50, align 8
  %52 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 1
  %53 = load i64, ptr %52, align 8
  %54 = icmp ult i64 %51, %53
  br i1 %54, label %58, label %55

55:                                               ; preds = %49
  %56 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 1)
  %57 = icmp ne ptr %56, null
  br label %58

58:                                               ; preds = %55, %49
  %59 = phi i1 [ true, %49 ], [ %57, %55 ]
  %60 = zext i1 %59 to i32
  %61 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 0
  %62 = load ptr, ptr %61, align 8
  %63 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %64 = load i64, ptr %63, align 8
  %65 = add i64 %64, 1
  store i64 %65, ptr %63, align 8
  %66 = getelementptr inbounds i8, ptr %62, i64 %64
  store i8 0, ptr %66, align 1
  br label %45, !llvm.loop !15

67:                                               ; preds = %45
  %68 = load i32, ptr %6, align 4
  %69 = add nsw i32 %68, 1
  store i32 %69, ptr %6, align 4
  %70 = load i32, ptr %10, align 4
  switch i32 %70, label %335 [
    i32 0, label %71
    i32 1, label %114
    i32 2, label %145
    i32 3, label %159
    i32 4, label %172
    i32 5, label %185
    i32 6, label %233
    i32 7, label %270
    i32 8, label %314
    i32 9, label %332
    i32 10, label %332
  ]

71:                                               ; preds = %67
  %72 = load ptr, ptr %2, align 8
  %73 = load i32, ptr %6, align 4
  %74 = call i64 @luaL_checkinteger(ptr noundef %72, i32 noundef %73)
  store i64 %74, ptr %11, align 8
  %75 = load i32, ptr %8, align 4
  %76 = icmp slt i32 %75, 8
  br i1 %76, label %77, label %106

77:                                               ; preds = %71
  %78 = load i32, ptr %8, align 4
  %79 = mul nsw i32 %78, 8
  %80 = sub nsw i32 %79, 1
  %81 = zext i32 %80 to i64
  %82 = shl i64 1, %81
  store i64 %82, ptr %12, align 8
  %83 = load i64, ptr %12, align 8
  %84 = sub nsw i64 0, %83
  %85 = load i64, ptr %11, align 8
  %86 = icmp sle i64 %84, %85
  br i1 %86, label %87, label %91

87:                                               ; preds = %77
  %88 = load i64, ptr %11, align 8
  %89 = load i64, ptr %12, align 8
  %90 = icmp slt i64 %88, %89
  br label %91

91:                                               ; preds = %87, %77
  %92 = phi i1 [ false, %77 ], [ %90, %87 ]
  %93 = zext i1 %92 to i32
  %94 = icmp ne i32 %93, 0
  %95 = zext i1 %94 to i32
  %96 = sext i32 %95 to i64
  %97 = icmp ne i64 %96, 0
  br i1 %97, label %103, label %98

98:                                               ; preds = %91
  %99 = load ptr, ptr %2, align 8
  %100 = load i32, ptr %6, align 4
  %101 = call i32 @luaL_argerror(ptr noundef %99, i32 noundef %100, ptr noundef @.str.59)
  %102 = icmp ne i32 %101, 0
  br label %103

103:                                              ; preds = %98, %91
  %104 = phi i1 [ true, %91 ], [ %102, %98 ]
  %105 = zext i1 %104 to i32
  br label %106

106:                                              ; preds = %103, %71
  %107 = load i64, ptr %11, align 8
  %108 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %109 = load i32, ptr %108, align 8
  %110 = load i32, ptr %8, align 4
  %111 = load i64, ptr %11, align 8
  %112 = icmp slt i64 %111, 0
  %113 = zext i1 %112 to i32
  call void @packint(ptr noundef %3, i64 noundef %107, i32 noundef %109, i32 noundef %110, i32 noundef %113)
  br label %335

114:                                              ; preds = %67
  %115 = load ptr, ptr %2, align 8
  %116 = load i32, ptr %6, align 4
  %117 = call i64 @luaL_checkinteger(ptr noundef %115, i32 noundef %116)
  store i64 %117, ptr %13, align 8
  %118 = load i32, ptr %8, align 4
  %119 = icmp slt i32 %118, 8
  br i1 %119, label %120, label %140

120:                                              ; preds = %114
  %121 = load i64, ptr %13, align 8
  %122 = load i32, ptr %8, align 4
  %123 = mul nsw i32 %122, 8
  %124 = zext i32 %123 to i64
  %125 = shl i64 1, %124
  %126 = icmp ult i64 %121, %125
  %127 = zext i1 %126 to i32
  %128 = icmp ne i32 %127, 0
  %129 = zext i1 %128 to i32
  %130 = sext i32 %129 to i64
  %131 = icmp ne i64 %130, 0
  br i1 %131, label %137, label %132

132:                                              ; preds = %120
  %133 = load ptr, ptr %2, align 8
  %134 = load i32, ptr %6, align 4
  %135 = call i32 @luaL_argerror(ptr noundef %133, i32 noundef %134, ptr noundef @.str.60)
  %136 = icmp ne i32 %135, 0
  br label %137

137:                                              ; preds = %132, %120
  %138 = phi i1 [ true, %120 ], [ %136, %132 ]
  %139 = zext i1 %138 to i32
  br label %140

140:                                              ; preds = %137, %114
  %141 = load i64, ptr %13, align 8
  %142 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %143 = load i32, ptr %142, align 8
  %144 = load i32, ptr %8, align 4
  call void @packint(ptr noundef %3, i64 noundef %141, i32 noundef %143, i32 noundef %144, i32 noundef 0)
  br label %335

145:                                              ; preds = %67
  %146 = load ptr, ptr %2, align 8
  %147 = load i32, ptr %6, align 4
  %148 = call double @luaL_checknumber(ptr noundef %146, i32 noundef %147)
  %149 = fptrunc double %148 to float
  store float %149, ptr %14, align 4
  %150 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 4)
  store ptr %150, ptr %15, align 8
  %151 = load ptr, ptr %15, align 8
  %152 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %153 = load i32, ptr %152, align 8
  call void @copywithendian(ptr noundef %151, ptr noundef %14, i32 noundef 4, i32 noundef %153)
  %154 = load i32, ptr %8, align 4
  %155 = sext i32 %154 to i64
  %156 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %157 = load i64, ptr %156, align 8
  %158 = add i64 %157, %155
  store i64 %158, ptr %156, align 8
  br label %335

159:                                              ; preds = %67
  %160 = load ptr, ptr %2, align 8
  %161 = load i32, ptr %6, align 4
  %162 = call double @luaL_checknumber(ptr noundef %160, i32 noundef %161)
  store double %162, ptr %16, align 8
  %163 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 8)
  store ptr %163, ptr %17, align 8
  %164 = load ptr, ptr %17, align 8
  %165 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %166 = load i32, ptr %165, align 8
  call void @copywithendian(ptr noundef %164, ptr noundef %16, i32 noundef 8, i32 noundef %166)
  %167 = load i32, ptr %8, align 4
  %168 = sext i32 %167 to i64
  %169 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %170 = load i64, ptr %169, align 8
  %171 = add i64 %170, %168
  store i64 %171, ptr %169, align 8
  br label %335

172:                                              ; preds = %67
  %173 = load ptr, ptr %2, align 8
  %174 = load i32, ptr %6, align 4
  %175 = call double @luaL_checknumber(ptr noundef %173, i32 noundef %174)
  store double %175, ptr %18, align 8
  %176 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 8)
  store ptr %176, ptr %19, align 8
  %177 = load ptr, ptr %19, align 8
  %178 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %179 = load i32, ptr %178, align 8
  call void @copywithendian(ptr noundef %177, ptr noundef %18, i32 noundef 8, i32 noundef %179)
  %180 = load i32, ptr %8, align 4
  %181 = sext i32 %180 to i64
  %182 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %183 = load i64, ptr %182, align 8
  %184 = add i64 %183, %181
  store i64 %184, ptr %182, align 8
  br label %335

185:                                              ; preds = %67
  %186 = load ptr, ptr %2, align 8
  %187 = load i32, ptr %6, align 4
  %188 = call ptr @luaL_checklstring(ptr noundef %186, i32 noundef %187, ptr noundef %20)
  store ptr %188, ptr %21, align 8
  %189 = load i64, ptr %20, align 8
  %190 = load i32, ptr %8, align 4
  %191 = sext i32 %190 to i64
  %192 = icmp ule i64 %189, %191
  %193 = zext i1 %192 to i32
  %194 = icmp ne i32 %193, 0
  %195 = zext i1 %194 to i32
  %196 = sext i32 %195 to i64
  %197 = icmp ne i64 %196, 0
  br i1 %197, label %203, label %198

198:                                              ; preds = %185
  %199 = load ptr, ptr %2, align 8
  %200 = load i32, ptr %6, align 4
  %201 = call i32 @luaL_argerror(ptr noundef %199, i32 noundef %200, ptr noundef @.str.61)
  %202 = icmp ne i32 %201, 0
  br label %203

203:                                              ; preds = %198, %185
  %204 = phi i1 [ true, %185 ], [ %202, %198 ]
  %205 = zext i1 %204 to i32
  %206 = load ptr, ptr %21, align 8
  %207 = load i64, ptr %20, align 8
  call void @luaL_addlstring(ptr noundef %3, ptr noundef %206, i64 noundef %207)
  br label %208

208:                                              ; preds = %223, %203
  %209 = load i64, ptr %20, align 8
  %210 = add i64 %209, 1
  store i64 %210, ptr %20, align 8
  %211 = load i32, ptr %8, align 4
  %212 = sext i32 %211 to i64
  %213 = icmp ult i64 %209, %212
  br i1 %213, label %214, label %232

214:                                              ; preds = %208
  %215 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %216 = load i64, ptr %215, align 8
  %217 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 1
  %218 = load i64, ptr %217, align 8
  %219 = icmp ult i64 %216, %218
  br i1 %219, label %223, label %220

220:                                              ; preds = %214
  %221 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 1)
  %222 = icmp ne ptr %221, null
  br label %223

223:                                              ; preds = %220, %214
  %224 = phi i1 [ true, %214 ], [ %222, %220 ]
  %225 = zext i1 %224 to i32
  %226 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 0
  %227 = load ptr, ptr %226, align 8
  %228 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %229 = load i64, ptr %228, align 8
  %230 = add i64 %229, 1
  store i64 %230, ptr %228, align 8
  %231 = getelementptr inbounds i8, ptr %227, i64 %229
  store i8 0, ptr %231, align 1
  br label %208, !llvm.loop !16

232:                                              ; preds = %208
  br label %335

233:                                              ; preds = %67
  %234 = load ptr, ptr %2, align 8
  %235 = load i32, ptr %6, align 4
  %236 = call ptr @luaL_checklstring(ptr noundef %234, i32 noundef %235, ptr noundef %22)
  store ptr %236, ptr %23, align 8
  %237 = load i32, ptr %8, align 4
  %238 = icmp sge i32 %237, 8
  br i1 %238, label %246, label %239

239:                                              ; preds = %233
  %240 = load i64, ptr %22, align 8
  %241 = load i32, ptr %8, align 4
  %242 = mul nsw i32 %241, 8
  %243 = zext i32 %242 to i64
  %244 = shl i64 1, %243
  %245 = icmp ult i64 %240, %244
  br label %246

246:                                              ; preds = %239, %233
  %247 = phi i1 [ true, %233 ], [ %245, %239 ]
  %248 = zext i1 %247 to i32
  %249 = icmp ne i32 %248, 0
  %250 = zext i1 %249 to i32
  %251 = sext i32 %250 to i64
  %252 = icmp ne i64 %251, 0
  br i1 %252, label %258, label %253

253:                                              ; preds = %246
  %254 = load ptr, ptr %2, align 8
  %255 = load i32, ptr %6, align 4
  %256 = call i32 @luaL_argerror(ptr noundef %254, i32 noundef %255, ptr noundef @.str.62)
  %257 = icmp ne i32 %256, 0
  br label %258

258:                                              ; preds = %253, %246
  %259 = phi i1 [ true, %246 ], [ %257, %253 ]
  %260 = zext i1 %259 to i32
  %261 = load i64, ptr %22, align 8
  %262 = getelementptr inbounds %struct.Header, ptr %4, i32 0, i32 1
  %263 = load i32, ptr %262, align 8
  %264 = load i32, ptr %8, align 4
  call void @packint(ptr noundef %3, i64 noundef %261, i32 noundef %263, i32 noundef %264, i32 noundef 0)
  %265 = load ptr, ptr %23, align 8
  %266 = load i64, ptr %22, align 8
  call void @luaL_addlstring(ptr noundef %3, ptr noundef %265, i64 noundef %266)
  %267 = load i64, ptr %22, align 8
  %268 = load i64, ptr %7, align 8
  %269 = add i64 %268, %267
  store i64 %269, ptr %7, align 8
  br label %335

270:                                              ; preds = %67
  %271 = load ptr, ptr %2, align 8
  %272 = load i32, ptr %6, align 4
  %273 = call ptr @luaL_checklstring(ptr noundef %271, i32 noundef %272, ptr noundef %24)
  store ptr %273, ptr %25, align 8
  %274 = load ptr, ptr %25, align 8
  %275 = call i64 @strlen(ptr noundef %274) #7
  %276 = load i64, ptr %24, align 8
  %277 = icmp eq i64 %275, %276
  %278 = zext i1 %277 to i32
  %279 = icmp ne i32 %278, 0
  %280 = zext i1 %279 to i32
  %281 = sext i32 %280 to i64
  %282 = icmp ne i64 %281, 0
  br i1 %282, label %288, label %283

283:                                              ; preds = %270
  %284 = load ptr, ptr %2, align 8
  %285 = load i32, ptr %6, align 4
  %286 = call i32 @luaL_argerror(ptr noundef %284, i32 noundef %285, ptr noundef @.str.40)
  %287 = icmp ne i32 %286, 0
  br label %288

288:                                              ; preds = %283, %270
  %289 = phi i1 [ true, %270 ], [ %287, %283 ]
  %290 = zext i1 %289 to i32
  %291 = load ptr, ptr %25, align 8
  %292 = load i64, ptr %24, align 8
  call void @luaL_addlstring(ptr noundef %3, ptr noundef %291, i64 noundef %292)
  %293 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %294 = load i64, ptr %293, align 8
  %295 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 1
  %296 = load i64, ptr %295, align 8
  %297 = icmp ult i64 %294, %296
  br i1 %297, label %301, label %298

298:                                              ; preds = %288
  %299 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 1)
  %300 = icmp ne ptr %299, null
  br label %301

301:                                              ; preds = %298, %288
  %302 = phi i1 [ true, %288 ], [ %300, %298 ]
  %303 = zext i1 %302 to i32
  %304 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 0
  %305 = load ptr, ptr %304, align 8
  %306 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %307 = load i64, ptr %306, align 8
  %308 = add i64 %307, 1
  store i64 %308, ptr %306, align 8
  %309 = getelementptr inbounds i8, ptr %305, i64 %307
  store i8 0, ptr %309, align 1
  %310 = load i64, ptr %24, align 8
  %311 = add i64 %310, 1
  %312 = load i64, ptr %7, align 8
  %313 = add i64 %312, %311
  store i64 %313, ptr %7, align 8
  br label %335

314:                                              ; preds = %67
  %315 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %316 = load i64, ptr %315, align 8
  %317 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 1
  %318 = load i64, ptr %317, align 8
  %319 = icmp ult i64 %316, %318
  br i1 %319, label %323, label %320

320:                                              ; preds = %314
  %321 = call ptr @luaL_prepbuffsize(ptr noundef %3, i64 noundef 1)
  %322 = icmp ne ptr %321, null
  br label %323

323:                                              ; preds = %320, %314
  %324 = phi i1 [ true, %314 ], [ %322, %320 ]
  %325 = zext i1 %324 to i32
  %326 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 0
  %327 = load ptr, ptr %326, align 8
  %328 = getelementptr inbounds %struct.luaL_Buffer, ptr %3, i32 0, i32 2
  %329 = load i64, ptr %328, align 8
  %330 = add i64 %329, 1
  store i64 %330, ptr %328, align 8
  %331 = getelementptr inbounds i8, ptr %327, i64 %329
  store i8 0, ptr %331, align 1
  br label %332

332:                                              ; preds = %67, %67, %323
  %333 = load i32, ptr %6, align 4
  %334 = add nsw i32 %333, -1
  store i32 %334, ptr %6, align 4
  br label %335

335:                                              ; preds = %67, %332, %301, %258, %232, %172, %159, %145, %140, %106
  br label %31, !llvm.loop !17

336:                                              ; preds = %31
  call void @luaL_pushresult(ptr noundef %3)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_packsize(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.Header, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call ptr @luaL_checklstring(ptr noundef %9, i32 noundef 1, ptr noundef null)
  store ptr %10, ptr %4, align 8
  store i64 0, ptr %5, align 8
  %11 = load ptr, ptr %2, align 8
  call void @initheader(ptr noundef %11, ptr noundef %3)
  br label %12

12:                                               ; preds = %56, %1
  %13 = load ptr, ptr %4, align 8
  %14 = load i8, ptr %13, align 1
  %15 = sext i8 %14 to i32
  %16 = icmp ne i32 %15, 0
  br i1 %16, label %17, label %63

17:                                               ; preds = %12
  %18 = load i64, ptr %5, align 8
  %19 = call i32 @getdetails(ptr noundef %3, i64 noundef %18, ptr noundef %4, ptr noundef %6, ptr noundef %7)
  store i32 %19, ptr %8, align 4
  %20 = load i32, ptr %8, align 4
  %21 = icmp ne i32 %20, 6
  br i1 %21, label %22, label %25

22:                                               ; preds = %17
  %23 = load i32, ptr %8, align 4
  %24 = icmp ne i32 %23, 7
  br label %25

25:                                               ; preds = %22, %17
  %26 = phi i1 [ false, %17 ], [ %24, %22 ]
  %27 = zext i1 %26 to i32
  %28 = icmp ne i32 %27, 0
  %29 = zext i1 %28 to i32
  %30 = sext i32 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %36, label %32

32:                                               ; preds = %25
  %33 = load ptr, ptr %2, align 8
  %34 = call i32 @luaL_argerror(ptr noundef %33, i32 noundef 1, ptr noundef @.str.68)
  %35 = icmp ne i32 %34, 0
  br label %36

36:                                               ; preds = %32, %25
  %37 = phi i1 [ true, %25 ], [ %35, %32 ]
  %38 = zext i1 %37 to i32
  %39 = load i32, ptr %7, align 4
  %40 = load i32, ptr %6, align 4
  %41 = add nsw i32 %40, %39
  store i32 %41, ptr %6, align 4
  %42 = load i64, ptr %5, align 8
  %43 = load i32, ptr %6, align 4
  %44 = sext i32 %43 to i64
  %45 = sub i64 2147483647, %44
  %46 = icmp ule i64 %42, %45
  %47 = zext i1 %46 to i32
  %48 = icmp ne i32 %47, 0
  %49 = zext i1 %48 to i32
  %50 = sext i32 %49 to i64
  %51 = icmp ne i64 %50, 0
  br i1 %51, label %56, label %52

52:                                               ; preds = %36
  %53 = load ptr, ptr %2, align 8
  %54 = call i32 @luaL_argerror(ptr noundef %53, i32 noundef 1, ptr noundef @.str.69)
  %55 = icmp ne i32 %54, 0
  br label %56

56:                                               ; preds = %52, %36
  %57 = phi i1 [ true, %36 ], [ %55, %52 ]
  %58 = zext i1 %57 to i32
  %59 = load i32, ptr %6, align 4
  %60 = sext i32 %59 to i64
  %61 = load i64, ptr %5, align 8
  %62 = add i64 %61, %60
  store i64 %62, ptr %5, align 8
  br label %12, !llvm.loop !18

63:                                               ; preds = %12
  %64 = load ptr, ptr %2, align 8
  %65 = load i64, ptr %5, align 8
  call void @lua_pushinteger(ptr noundef %64, i64 noundef %65)
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_unpack(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca %struct.Header, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  %12 = alloca i64, align 8
  %13 = alloca float, align 4
  %14 = alloca double, align 8
  %15 = alloca double, align 8
  %16 = alloca i64, align 8
  %17 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = call ptr @luaL_checklstring(ptr noundef %18, i32 noundef 1, ptr noundef null)
  store ptr %19, ptr %4, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = call ptr @luaL_checklstring(ptr noundef %20, i32 noundef 2, ptr noundef %5)
  store ptr %21, ptr %6, align 8
  %22 = load ptr, ptr %2, align 8
  %23 = call i64 @luaL_optinteger(ptr noundef %22, i32 noundef 3, i64 noundef 1)
  %24 = load i64, ptr %5, align 8
  %25 = call i64 @posrelatI(i64 noundef %23, i64 noundef %24)
  %26 = sub i64 %25, 1
  store i64 %26, ptr %7, align 8
  store i32 0, ptr %8, align 4
  %27 = load i64, ptr %7, align 8
  %28 = load i64, ptr %5, align 8
  %29 = icmp ule i64 %27, %28
  %30 = zext i1 %29 to i32
  %31 = icmp ne i32 %30, 0
  %32 = zext i1 %31 to i32
  %33 = sext i32 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %39, label %35

35:                                               ; preds = %1
  %36 = load ptr, ptr %2, align 8
  %37 = call i32 @luaL_argerror(ptr noundef %36, i32 noundef 3, ptr noundef @.str.70)
  %38 = icmp ne i32 %37, 0
  br label %39

39:                                               ; preds = %35, %1
  %40 = phi i1 [ true, %1 ], [ %38, %35 ]
  %41 = zext i1 %40 to i32
  %42 = load ptr, ptr %2, align 8
  call void @initheader(ptr noundef %42, ptr noundef %3)
  br label %43

43:                                               ; preds = %203, %39
  %44 = load ptr, ptr %4, align 8
  %45 = load i8, ptr %44, align 1
  %46 = sext i8 %45 to i32
  %47 = icmp ne i32 %46, 0
  br i1 %47, label %48, label %208

48:                                               ; preds = %43
  %49 = load i64, ptr %7, align 8
  %50 = call i32 @getdetails(ptr noundef %3, i64 noundef %49, ptr noundef %4, ptr noundef %9, ptr noundef %10)
  store i32 %50, ptr %11, align 4
  %51 = load i32, ptr %10, align 4
  %52 = sext i32 %51 to i64
  %53 = load i32, ptr %9, align 4
  %54 = sext i32 %53 to i64
  %55 = add i64 %52, %54
  %56 = load i64, ptr %5, align 8
  %57 = load i64, ptr %7, align 8
  %58 = sub i64 %56, %57
  %59 = icmp ule i64 %55, %58
  %60 = zext i1 %59 to i32
  %61 = icmp ne i32 %60, 0
  %62 = zext i1 %61 to i32
  %63 = sext i32 %62 to i64
  %64 = icmp ne i64 %63, 0
  br i1 %64, label %69, label %65

65:                                               ; preds = %48
  %66 = load ptr, ptr %2, align 8
  %67 = call i32 @luaL_argerror(ptr noundef %66, i32 noundef 2, ptr noundef @.str.71)
  %68 = icmp ne i32 %67, 0
  br label %69

69:                                               ; preds = %65, %48
  %70 = phi i1 [ true, %48 ], [ %68, %65 ]
  %71 = zext i1 %70 to i32
  %72 = load i32, ptr %10, align 4
  %73 = sext i32 %72 to i64
  %74 = load i64, ptr %7, align 8
  %75 = add i64 %74, %73
  store i64 %75, ptr %7, align 8
  %76 = load ptr, ptr %2, align 8
  call void @luaL_checkstack(ptr noundef %76, i32 noundef 2, ptr noundef @.str.72)
  %77 = load i32, ptr %8, align 4
  %78 = add nsw i32 %77, 1
  store i32 %78, ptr %8, align 4
  %79 = load i32, ptr %11, align 4
  switch i32 %79, label %203 [
    i32 0, label %80
    i32 1, label %80
    i32 2, label %94
    i32 3, label %103
    i32 4, label %111
    i32 5, label %119
    i32 6, label %127
    i32 7, label %168
    i32 9, label %200
    i32 8, label %200
    i32 10, label %200
  ]

80:                                               ; preds = %69, %69
  %81 = load ptr, ptr %2, align 8
  %82 = load ptr, ptr %6, align 8
  %83 = load i64, ptr %7, align 8
  %84 = getelementptr inbounds i8, ptr %82, i64 %83
  %85 = getelementptr inbounds %struct.Header, ptr %3, i32 0, i32 1
  %86 = load i32, ptr %85, align 8
  %87 = load i32, ptr %9, align 4
  %88 = load i32, ptr %11, align 4
  %89 = icmp eq i32 %88, 0
  %90 = zext i1 %89 to i32
  %91 = call i64 @unpackint(ptr noundef %81, ptr noundef %84, i32 noundef %86, i32 noundef %87, i32 noundef %90)
  store i64 %91, ptr %12, align 8
  %92 = load ptr, ptr %2, align 8
  %93 = load i64, ptr %12, align 8
  call void @lua_pushinteger(ptr noundef %92, i64 noundef %93)
  br label %203

94:                                               ; preds = %69
  %95 = load ptr, ptr %6, align 8
  %96 = load i64, ptr %7, align 8
  %97 = getelementptr inbounds i8, ptr %95, i64 %96
  %98 = getelementptr inbounds %struct.Header, ptr %3, i32 0, i32 1
  %99 = load i32, ptr %98, align 8
  call void @copywithendian(ptr noundef %13, ptr noundef %97, i32 noundef 4, i32 noundef %99)
  %100 = load ptr, ptr %2, align 8
  %101 = load float, ptr %13, align 4
  %102 = fpext float %101 to double
  call void @lua_pushnumber(ptr noundef %100, double noundef %102)
  br label %203

103:                                              ; preds = %69
  %104 = load ptr, ptr %6, align 8
  %105 = load i64, ptr %7, align 8
  %106 = getelementptr inbounds i8, ptr %104, i64 %105
  %107 = getelementptr inbounds %struct.Header, ptr %3, i32 0, i32 1
  %108 = load i32, ptr %107, align 8
  call void @copywithendian(ptr noundef %14, ptr noundef %106, i32 noundef 8, i32 noundef %108)
  %109 = load ptr, ptr %2, align 8
  %110 = load double, ptr %14, align 8
  call void @lua_pushnumber(ptr noundef %109, double noundef %110)
  br label %203

111:                                              ; preds = %69
  %112 = load ptr, ptr %6, align 8
  %113 = load i64, ptr %7, align 8
  %114 = getelementptr inbounds i8, ptr %112, i64 %113
  %115 = getelementptr inbounds %struct.Header, ptr %3, i32 0, i32 1
  %116 = load i32, ptr %115, align 8
  call void @copywithendian(ptr noundef %15, ptr noundef %114, i32 noundef 8, i32 noundef %116)
  %117 = load ptr, ptr %2, align 8
  %118 = load double, ptr %15, align 8
  call void @lua_pushnumber(ptr noundef %117, double noundef %118)
  br label %203

119:                                              ; preds = %69
  %120 = load ptr, ptr %2, align 8
  %121 = load ptr, ptr %6, align 8
  %122 = load i64, ptr %7, align 8
  %123 = getelementptr inbounds i8, ptr %121, i64 %122
  %124 = load i32, ptr %9, align 4
  %125 = sext i32 %124 to i64
  %126 = call ptr @lua_pushlstring(ptr noundef %120, ptr noundef %123, i64 noundef %125)
  br label %203

127:                                              ; preds = %69
  %128 = load ptr, ptr %2, align 8
  %129 = load ptr, ptr %6, align 8
  %130 = load i64, ptr %7, align 8
  %131 = getelementptr inbounds i8, ptr %129, i64 %130
  %132 = getelementptr inbounds %struct.Header, ptr %3, i32 0, i32 1
  %133 = load i32, ptr %132, align 8
  %134 = load i32, ptr %9, align 4
  %135 = call i64 @unpackint(ptr noundef %128, ptr noundef %131, i32 noundef %133, i32 noundef %134, i32 noundef 0)
  store i64 %135, ptr %16, align 8
  %136 = load i64, ptr %16, align 8
  %137 = load i64, ptr %5, align 8
  %138 = load i64, ptr %7, align 8
  %139 = sub i64 %137, %138
  %140 = load i32, ptr %9, align 4
  %141 = sext i32 %140 to i64
  %142 = sub i64 %139, %141
  %143 = icmp ule i64 %136, %142
  %144 = zext i1 %143 to i32
  %145 = icmp ne i32 %144, 0
  %146 = zext i1 %145 to i32
  %147 = sext i32 %146 to i64
  %148 = icmp ne i64 %147, 0
  br i1 %148, label %153, label %149

149:                                              ; preds = %127
  %150 = load ptr, ptr %2, align 8
  %151 = call i32 @luaL_argerror(ptr noundef %150, i32 noundef 2, ptr noundef @.str.71)
  %152 = icmp ne i32 %151, 0
  br label %153

153:                                              ; preds = %149, %127
  %154 = phi i1 [ true, %127 ], [ %152, %149 ]
  %155 = zext i1 %154 to i32
  %156 = load ptr, ptr %2, align 8
  %157 = load ptr, ptr %6, align 8
  %158 = load i64, ptr %7, align 8
  %159 = getelementptr inbounds i8, ptr %157, i64 %158
  %160 = load i32, ptr %9, align 4
  %161 = sext i32 %160 to i64
  %162 = getelementptr inbounds i8, ptr %159, i64 %161
  %163 = load i64, ptr %16, align 8
  %164 = call ptr @lua_pushlstring(ptr noundef %156, ptr noundef %162, i64 noundef %163)
  %165 = load i64, ptr %16, align 8
  %166 = load i64, ptr %7, align 8
  %167 = add i64 %166, %165
  store i64 %167, ptr %7, align 8
  br label %203

168:                                              ; preds = %69
  %169 = load ptr, ptr %6, align 8
  %170 = load i64, ptr %7, align 8
  %171 = getelementptr inbounds i8, ptr %169, i64 %170
  %172 = call i64 @strlen(ptr noundef %171) #7
  store i64 %172, ptr %17, align 8
  %173 = load i64, ptr %7, align 8
  %174 = load i64, ptr %17, align 8
  %175 = add i64 %173, %174
  %176 = load i64, ptr %5, align 8
  %177 = icmp ult i64 %175, %176
  %178 = zext i1 %177 to i32
  %179 = icmp ne i32 %178, 0
  %180 = zext i1 %179 to i32
  %181 = sext i32 %180 to i64
  %182 = icmp ne i64 %181, 0
  br i1 %182, label %187, label %183

183:                                              ; preds = %168
  %184 = load ptr, ptr %2, align 8
  %185 = call i32 @luaL_argerror(ptr noundef %184, i32 noundef 2, ptr noundef @.str.73)
  %186 = icmp ne i32 %185, 0
  br label %187

187:                                              ; preds = %183, %168
  %188 = phi i1 [ true, %168 ], [ %186, %183 ]
  %189 = zext i1 %188 to i32
  %190 = load ptr, ptr %2, align 8
  %191 = load ptr, ptr %6, align 8
  %192 = load i64, ptr %7, align 8
  %193 = getelementptr inbounds i8, ptr %191, i64 %192
  %194 = load i64, ptr %17, align 8
  %195 = call ptr @lua_pushlstring(ptr noundef %190, ptr noundef %193, i64 noundef %194)
  %196 = load i64, ptr %17, align 8
  %197 = add i64 %196, 1
  %198 = load i64, ptr %7, align 8
  %199 = add i64 %198, %197
  store i64 %199, ptr %7, align 8
  br label %203

200:                                              ; preds = %69, %69, %69
  %201 = load i32, ptr %8, align 4
  %202 = add nsw i32 %201, -1
  store i32 %202, ptr %8, align 4
  br label %203

203:                                              ; preds = %69, %200, %187, %153, %119, %111, %103, %94, %80
  %204 = load i32, ptr %9, align 4
  %205 = sext i32 %204 to i64
  %206 = load i64, ptr %7, align 8
  %207 = add i64 %206, %205
  store i64 %207, ptr %7, align 8
  br label %43, !llvm.loop !19

208:                                              ; preds = %43
  %209 = load ptr, ptr %2, align 8
  %210 = load i64, ptr %7, align 8
  %211 = add i64 %210, 1
  call void @lua_pushinteger(ptr noundef %209, i64 noundef %211)
  %212 = load i32, ptr %8, align 4
  %213 = add nsw i32 %212, 1
  ret i32 %213
}

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @posrelatI(i64 noundef %0, i64 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  store i64 %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %6 = load i64, ptr %4, align 8
  %7 = icmp sgt i64 %6, 0
  br i1 %7, label %8, label %10

8:                                                ; preds = %2
  %9 = load i64, ptr %4, align 8
  store i64 %9, ptr %3, align 8
  br label %25

10:                                               ; preds = %2
  %11 = load i64, ptr %4, align 8
  %12 = icmp eq i64 %11, 0
  br i1 %12, label %13, label %14

13:                                               ; preds = %10
  store i64 1, ptr %3, align 8
  br label %25

14:                                               ; preds = %10
  %15 = load i64, ptr %4, align 8
  %16 = load i64, ptr %5, align 8
  %17 = sub nsw i64 0, %16
  %18 = icmp slt i64 %15, %17
  br i1 %18, label %19, label %20

19:                                               ; preds = %14
  store i64 1, ptr %3, align 8
  br label %25

20:                                               ; preds = %14
  %21 = load i64, ptr %5, align 8
  %22 = load i64, ptr %4, align 8
  %23 = add i64 %21, %22
  %24 = add i64 %23, 1
  store i64 %24, ptr %3, align 8
  br label %25

25:                                               ; preds = %20, %19, %13, %8
  %26 = load i64, ptr %3, align 8
  ret i64 %26
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @getendpos(ptr noundef %0, i32 noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i64 %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = load i32, ptr %7, align 4
  %13 = load i64, ptr %8, align 8
  %14 = call i64 @luaL_optinteger(ptr noundef %11, i32 noundef %12, i64 noundef %13)
  store i64 %14, ptr %10, align 8
  %15 = load i64, ptr %10, align 8
  %16 = load i64, ptr %9, align 8
  %17 = icmp sgt i64 %15, %16
  br i1 %17, label %18, label %20

18:                                               ; preds = %4
  %19 = load i64, ptr %9, align 8
  store i64 %19, ptr %5, align 8
  br label %36

20:                                               ; preds = %4
  %21 = load i64, ptr %10, align 8
  %22 = icmp sge i64 %21, 0
  br i1 %22, label %23, label %25

23:                                               ; preds = %20
  %24 = load i64, ptr %10, align 8
  store i64 %24, ptr %5, align 8
  br label %36

25:                                               ; preds = %20
  %26 = load i64, ptr %10, align 8
  %27 = load i64, ptr %9, align 8
  %28 = sub nsw i64 0, %27
  %29 = icmp slt i64 %26, %28
  br i1 %29, label %30, label %31

30:                                               ; preds = %25
  store i64 0, ptr %5, align 8
  br label %36

31:                                               ; preds = %25
  %32 = load i64, ptr %9, align 8
  %33 = load i64, ptr %10, align 8
  %34 = add i64 %32, %33
  %35 = add i64 %34, 1
  store i64 %35, ptr %5, align 8
  br label %36

36:                                               ; preds = %31, %30, %23, %18
  %37 = load i64, ptr %5, align 8
  ret i64 %37
}

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare void @luaL_checkstack(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare i32 @lua_gettop(ptr noundef) #1

declare ptr @luaL_buffinitsize(ptr noundef, ptr noundef, i64 noundef) #1

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_pushresultsize(ptr noundef, i64 noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

declare void @luaL_checktype(ptr noundef, i32 noundef, i32 noundef) #1

declare void @lua_settop(ptr noundef, i32 noundef) #1

declare i32 @lua_dump(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @writer(ptr noundef %0, ptr noundef %1, i64 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %10 = load ptr, ptr %8, align 8
  store ptr %10, ptr %9, align 8
  %11 = load ptr, ptr %9, align 8
  %12 = getelementptr inbounds %struct.str_Writer, ptr %11, i32 0, i32 0
  %13 = load i32, ptr %12, align 8
  %14 = icmp ne i32 %13, 0
  br i1 %14, label %21, label %15

15:                                               ; preds = %4
  %16 = load ptr, ptr %9, align 8
  %17 = getelementptr inbounds %struct.str_Writer, ptr %16, i32 0, i32 0
  store i32 1, ptr %17, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = load ptr, ptr %9, align 8
  %20 = getelementptr inbounds %struct.str_Writer, ptr %19, i32 0, i32 1
  call void @luaL_buffinit(ptr noundef %18, ptr noundef %20)
  br label %21

21:                                               ; preds = %15, %4
  %22 = load ptr, ptr %9, align 8
  %23 = getelementptr inbounds %struct.str_Writer, ptr %22, i32 0, i32 1
  %24 = load ptr, ptr %6, align 8
  %25 = load i64, ptr %7, align 8
  call void @luaL_addlstring(ptr noundef %23, ptr noundef %24, i64 noundef %25)
  ret i32 0
}

declare void @luaL_pushresult(ptr noundef) #1

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

declare void @luaL_addlstring(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @str_find_aux(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  %11 = alloca ptr, align 8
  %12 = alloca %struct.MatchState, align 8
  %13 = alloca ptr, align 8
  %14 = alloca i32, align 4
  %15 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %16 = load ptr, ptr %4, align 8
  %17 = call ptr @luaL_checklstring(ptr noundef %16, i32 noundef 1, ptr noundef %6)
  store ptr %17, ptr %8, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = call ptr @luaL_checklstring(ptr noundef %18, i32 noundef 2, ptr noundef %7)
  store ptr %19, ptr %9, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = call i64 @luaL_optinteger(ptr noundef %20, i32 noundef 3, i64 noundef 1)
  %22 = load i64, ptr %6, align 8
  %23 = call i64 @posrelatI(i64 noundef %21, i64 noundef %22)
  %24 = sub i64 %23, 1
  store i64 %24, ptr %10, align 8
  %25 = load i64, ptr %10, align 8
  %26 = load i64, ptr %6, align 8
  %27 = icmp ugt i64 %25, %26
  br i1 %27, label %28, label %30

28:                                               ; preds = %2
  %29 = load ptr, ptr %4, align 8
  call void @lua_pushnil(ptr noundef %29)
  store i32 1, ptr %3, align 4
  br label %137

30:                                               ; preds = %2
  %31 = load i32, ptr %5, align 4
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %71

33:                                               ; preds = %30
  %34 = load ptr, ptr %4, align 8
  %35 = call i32 @lua_toboolean(ptr noundef %34, i32 noundef 4)
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %42, label %37

37:                                               ; preds = %33
  %38 = load ptr, ptr %9, align 8
  %39 = load i64, ptr %7, align 8
  %40 = call i32 @nospecials(ptr noundef %38, i64 noundef %39)
  %41 = icmp ne i32 %40, 0
  br i1 %41, label %42, label %71

42:                                               ; preds = %37, %33
  %43 = load ptr, ptr %8, align 8
  %44 = load i64, ptr %10, align 8
  %45 = getelementptr inbounds i8, ptr %43, i64 %44
  %46 = load i64, ptr %6, align 8
  %47 = load i64, ptr %10, align 8
  %48 = sub i64 %46, %47
  %49 = load ptr, ptr %9, align 8
  %50 = load i64, ptr %7, align 8
  %51 = call ptr @lmemfind(ptr noundef %45, i64 noundef %48, ptr noundef %49, i64 noundef %50)
  store ptr %51, ptr %11, align 8
  %52 = load ptr, ptr %11, align 8
  %53 = icmp ne ptr %52, null
  br i1 %53, label %54, label %70

54:                                               ; preds = %42
  %55 = load ptr, ptr %4, align 8
  %56 = load ptr, ptr %11, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = ptrtoint ptr %56 to i64
  %59 = ptrtoint ptr %57 to i64
  %60 = sub i64 %58, %59
  %61 = add nsw i64 %60, 1
  call void @lua_pushinteger(ptr noundef %55, i64 noundef %61)
  %62 = load ptr, ptr %4, align 8
  %63 = load ptr, ptr %11, align 8
  %64 = load ptr, ptr %8, align 8
  %65 = ptrtoint ptr %63 to i64
  %66 = ptrtoint ptr %64 to i64
  %67 = sub i64 %65, %66
  %68 = load i64, ptr %7, align 8
  %69 = add i64 %67, %68
  call void @lua_pushinteger(ptr noundef %62, i64 noundef %69)
  store i32 2, ptr %3, align 4
  br label %137

70:                                               ; preds = %42
  br label %135

71:                                               ; preds = %37, %30
  %72 = load ptr, ptr %8, align 8
  %73 = load i64, ptr %10, align 8
  %74 = getelementptr inbounds i8, ptr %72, i64 %73
  store ptr %74, ptr %13, align 8
  %75 = load ptr, ptr %9, align 8
  %76 = load i8, ptr %75, align 1
  %77 = sext i8 %76 to i32
  %78 = icmp eq i32 %77, 94
  %79 = zext i1 %78 to i32
  store i32 %79, ptr %14, align 4
  %80 = load i32, ptr %14, align 4
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %87

82:                                               ; preds = %71
  %83 = load ptr, ptr %9, align 8
  %84 = getelementptr inbounds i8, ptr %83, i32 1
  store ptr %84, ptr %9, align 8
  %85 = load i64, ptr %7, align 8
  %86 = add i64 %85, -1
  store i64 %86, ptr %7, align 8
  br label %87

87:                                               ; preds = %82, %71
  %88 = load ptr, ptr %4, align 8
  %89 = load ptr, ptr %8, align 8
  %90 = load i64, ptr %6, align 8
  %91 = load ptr, ptr %9, align 8
  %92 = load i64, ptr %7, align 8
  call void @prepstate(ptr noundef %12, ptr noundef %88, ptr noundef %89, i64 noundef %90, ptr noundef %91, i64 noundef %92)
  br label %93

93:                                               ; preds = %132, %87
  call void @reprepstate(ptr noundef %12)
  %94 = load ptr, ptr %13, align 8
  %95 = load ptr, ptr %9, align 8
  %96 = call ptr @match(ptr noundef %12, ptr noundef %94, ptr noundef %95)
  store ptr %96, ptr %15, align 8
  %97 = icmp ne ptr %96, null
  br i1 %97, label %98, label %121

98:                                               ; preds = %93
  %99 = load i32, ptr %5, align 4
  %100 = icmp ne i32 %99, 0
  br i1 %100, label %101, label %117

101:                                              ; preds = %98
  %102 = load ptr, ptr %4, align 8
  %103 = load ptr, ptr %13, align 8
  %104 = load ptr, ptr %8, align 8
  %105 = ptrtoint ptr %103 to i64
  %106 = ptrtoint ptr %104 to i64
  %107 = sub i64 %105, %106
  %108 = add nsw i64 %107, 1
  call void @lua_pushinteger(ptr noundef %102, i64 noundef %108)
  %109 = load ptr, ptr %4, align 8
  %110 = load ptr, ptr %15, align 8
  %111 = load ptr, ptr %8, align 8
  %112 = ptrtoint ptr %110 to i64
  %113 = ptrtoint ptr %111 to i64
  %114 = sub i64 %112, %113
  call void @lua_pushinteger(ptr noundef %109, i64 noundef %114)
  %115 = call i32 @push_captures(ptr noundef %12, ptr noundef null, ptr noundef null)
  %116 = add nsw i32 %115, 2
  store i32 %116, ptr %3, align 4
  br label %137

117:                                              ; preds = %98
  %118 = load ptr, ptr %13, align 8
  %119 = load ptr, ptr %15, align 8
  %120 = call i32 @push_captures(ptr noundef %12, ptr noundef %118, ptr noundef %119)
  store i32 %120, ptr %3, align 4
  br label %137

121:                                              ; preds = %93
  br label %122

122:                                              ; preds = %121
  %123 = load ptr, ptr %13, align 8
  %124 = getelementptr inbounds i8, ptr %123, i32 1
  store ptr %124, ptr %13, align 8
  %125 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 1
  %126 = load ptr, ptr %125, align 8
  %127 = icmp ult ptr %123, %126
  br i1 %127, label %128, label %132

128:                                              ; preds = %122
  %129 = load i32, ptr %14, align 4
  %130 = icmp ne i32 %129, 0
  %131 = xor i1 %130, true
  br label %132

132:                                              ; preds = %128, %122
  %133 = phi i1 [ false, %122 ], [ %131, %128 ]
  br i1 %133, label %93, label %134, !llvm.loop !20

134:                                              ; preds = %132
  br label %135

135:                                              ; preds = %134, %70
  %136 = load ptr, ptr %4, align 8
  call void @lua_pushnil(ptr noundef %136)
  store i32 1, ptr %3, align 4
  br label %137

137:                                              ; preds = %135, %117, %101, %54, %28
  %138 = load i32, ptr %3, align 4
  ret i32 %138
}

declare void @lua_pushnil(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @nospecials(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  store i64 0, ptr %6, align 8
  br label %7

7:                                                ; preds = %22, %2
  %8 = load ptr, ptr %4, align 8
  %9 = load i64, ptr %6, align 8
  %10 = getelementptr inbounds i8, ptr %8, i64 %9
  %11 = call ptr @strpbrk(ptr noundef %10, ptr noundef @.str.20) #7
  %12 = icmp ne ptr %11, null
  br i1 %12, label %13, label %14

13:                                               ; preds = %7
  store i32 0, ptr %3, align 4
  br label %27

14:                                               ; preds = %7
  %15 = load ptr, ptr %4, align 8
  %16 = load i64, ptr %6, align 8
  %17 = getelementptr inbounds i8, ptr %15, i64 %16
  %18 = call i64 @strlen(ptr noundef %17) #7
  %19 = add i64 %18, 1
  %20 = load i64, ptr %6, align 8
  %21 = add i64 %20, %19
  store i64 %21, ptr %6, align 8
  br label %22

22:                                               ; preds = %14
  %23 = load i64, ptr %6, align 8
  %24 = load i64, ptr %5, align 8
  %25 = icmp ule i64 %23, %24
  br i1 %25, label %7, label %26, !llvm.loop !21

26:                                               ; preds = %22
  store i32 1, ptr %3, align 4
  br label %27

27:                                               ; preds = %26, %13
  %28 = load i32, ptr %3, align 4
  ret i32 %28
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @lmemfind(ptr noundef %0, i64 noundef %1, ptr noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i64 %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %11 = load i64, ptr %9, align 8
  %12 = icmp eq i64 %11, 0
  br i1 %12, label %13, label %15

13:                                               ; preds = %4
  %14 = load ptr, ptr %6, align 8
  store ptr %14, ptr %5, align 8
  br label %62

15:                                               ; preds = %4
  %16 = load i64, ptr %9, align 8
  %17 = load i64, ptr %7, align 8
  %18 = icmp ugt i64 %16, %17
  br i1 %18, label %19, label %20

19:                                               ; preds = %15
  store ptr null, ptr %5, align 8
  br label %62

20:                                               ; preds = %15
  %21 = load i64, ptr %9, align 8
  %22 = add i64 %21, -1
  store i64 %22, ptr %9, align 8
  %23 = load i64, ptr %7, align 8
  %24 = load i64, ptr %9, align 8
  %25 = sub i64 %23, %24
  store i64 %25, ptr %7, align 8
  br label %26

26:                                               ; preds = %60, %20
  %27 = load i64, ptr %7, align 8
  %28 = icmp ugt i64 %27, 0
  br i1 %28, label %29, label %37

29:                                               ; preds = %26
  %30 = load ptr, ptr %6, align 8
  %31 = load ptr, ptr %8, align 8
  %32 = load i8, ptr %31, align 1
  %33 = sext i8 %32 to i32
  %34 = load i64, ptr %7, align 8
  %35 = call ptr @memchr(ptr noundef %30, i32 noundef %33, i64 noundef %34) #7
  store ptr %35, ptr %10, align 8
  %36 = icmp ne ptr %35, null
  br label %37

37:                                               ; preds = %29, %26
  %38 = phi i1 [ false, %26 ], [ %36, %29 ]
  br i1 %38, label %39, label %61

39:                                               ; preds = %37
  %40 = load ptr, ptr %10, align 8
  %41 = getelementptr inbounds i8, ptr %40, i32 1
  store ptr %41, ptr %10, align 8
  %42 = load ptr, ptr %10, align 8
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds i8, ptr %43, i64 1
  %45 = load i64, ptr %9, align 8
  %46 = call i32 @memcmp(ptr noundef %42, ptr noundef %44, i64 noundef %45) #7
  %47 = icmp eq i32 %46, 0
  br i1 %47, label %48, label %51

48:                                               ; preds = %39
  %49 = load ptr, ptr %10, align 8
  %50 = getelementptr inbounds i8, ptr %49, i64 -1
  store ptr %50, ptr %5, align 8
  br label %62

51:                                               ; preds = %39
  %52 = load ptr, ptr %10, align 8
  %53 = load ptr, ptr %6, align 8
  %54 = ptrtoint ptr %52 to i64
  %55 = ptrtoint ptr %53 to i64
  %56 = sub i64 %54, %55
  %57 = load i64, ptr %7, align 8
  %58 = sub i64 %57, %56
  store i64 %58, ptr %7, align 8
  %59 = load ptr, ptr %10, align 8
  store ptr %59, ptr %6, align 8
  br label %60

60:                                               ; preds = %51
  br label %26, !llvm.loop !22

61:                                               ; preds = %37
  store ptr null, ptr %5, align 8
  br label %62

62:                                               ; preds = %61, %48, %19, %13
  %63 = load ptr, ptr %5, align 8
  ret ptr %63
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @prepstate(ptr noundef %0, ptr noundef %1, ptr noundef %2, i64 noundef %3, ptr noundef %4, i64 noundef %5) #0 {
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i64, align 8
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store i64 %3, ptr %10, align 8
  store ptr %4, ptr %11, align 8
  store i64 %5, ptr %12, align 8
  %13 = load ptr, ptr %8, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.MatchState, ptr %14, i32 0, i32 3
  store ptr %13, ptr %15, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = getelementptr inbounds %struct.MatchState, ptr %16, i32 0, i32 4
  store i32 200, ptr %17, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.MatchState, ptr %19, i32 0, i32 0
  store ptr %18, ptr %20, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = load i64, ptr %10, align 8
  %23 = getelementptr inbounds i8, ptr %21, i64 %22
  %24 = load ptr, ptr %7, align 8
  %25 = getelementptr inbounds %struct.MatchState, ptr %24, i32 0, i32 1
  store ptr %23, ptr %25, align 8
  %26 = load ptr, ptr %11, align 8
  %27 = load i64, ptr %12, align 8
  %28 = getelementptr inbounds i8, ptr %26, i64 %27
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds %struct.MatchState, ptr %29, i32 0, i32 2
  store ptr %28, ptr %30, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @reprepstate(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.MatchState, ptr %3, i32 0, i32 5
  store i8 0, ptr %4, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @match(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i8, align 1
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.MatchState, ptr %11, i32 0, i32 4
  %13 = load i32, ptr %12, align 8
  %14 = add nsw i32 %13, -1
  store i32 %14, ptr %12, align 8
  %15 = icmp eq i32 %13, 0
  %16 = zext i1 %15 to i32
  %17 = icmp ne i32 %16, 0
  %18 = zext i1 %17 to i32
  %19 = sext i32 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %21, label %26

21:                                               ; preds = %3
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.MatchState, ptr %22, i32 0, i32 3
  %24 = load ptr, ptr %23, align 8
  %25 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %24, ptr noundef @.str.21)
  br label %26

26:                                               ; preds = %21, %3
  br label %27

27:                                               ; preds = %232, %213, %194, %162, %149, %94, %26
  %28 = load ptr, ptr %6, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.MatchState, ptr %29, i32 0, i32 2
  %31 = load ptr, ptr %30, align 8
  %32 = icmp ne ptr %28, %31
  br i1 %32, label %33, label %239

33:                                               ; preds = %27
  %34 = load ptr, ptr %6, align 8
  %35 = load i8, ptr %34, align 1
  %36 = sext i8 %35 to i32
  switch i32 %36, label %168 [
    i32 40, label %37
    i32 41, label %56
    i32 36, label %62
    i32 37, label %81
  ]

37:                                               ; preds = %33
  %38 = load ptr, ptr %6, align 8
  %39 = getelementptr inbounds i8, ptr %38, i64 1
  %40 = load i8, ptr %39, align 1
  %41 = sext i8 %40 to i32
  %42 = icmp eq i32 %41, 41
  br i1 %42, label %43, label %49

43:                                               ; preds = %37
  %44 = load ptr, ptr %4, align 8
  %45 = load ptr, ptr %5, align 8
  %46 = load ptr, ptr %6, align 8
  %47 = getelementptr inbounds i8, ptr %46, i64 2
  %48 = call ptr @start_capture(ptr noundef %44, ptr noundef %45, ptr noundef %47, i32 noundef -2)
  store ptr %48, ptr %5, align 8
  br label %55

49:                                               ; preds = %37
  %50 = load ptr, ptr %4, align 8
  %51 = load ptr, ptr %5, align 8
  %52 = load ptr, ptr %6, align 8
  %53 = getelementptr inbounds i8, ptr %52, i64 1
  %54 = call ptr @start_capture(ptr noundef %50, ptr noundef %51, ptr noundef %53, i32 noundef -1)
  store ptr %54, ptr %5, align 8
  br label %55

55:                                               ; preds = %49, %43
  br label %238

56:                                               ; preds = %33
  %57 = load ptr, ptr %4, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = getelementptr inbounds i8, ptr %59, i64 1
  %61 = call ptr @end_capture(ptr noundef %57, ptr noundef %58, ptr noundef %60)
  store ptr %61, ptr %5, align 8
  br label %238

62:                                               ; preds = %33
  %63 = load ptr, ptr %6, align 8
  %64 = getelementptr inbounds i8, ptr %63, i64 1
  %65 = load ptr, ptr %4, align 8
  %66 = getelementptr inbounds %struct.MatchState, ptr %65, i32 0, i32 2
  %67 = load ptr, ptr %66, align 8
  %68 = icmp ne ptr %64, %67
  br i1 %68, label %69, label %70

69:                                               ; preds = %62
  br label %169

70:                                               ; preds = %62
  %71 = load ptr, ptr %5, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.MatchState, ptr %72, i32 0, i32 1
  %74 = load ptr, ptr %73, align 8
  %75 = icmp eq ptr %71, %74
  br i1 %75, label %76, label %78

76:                                               ; preds = %70
  %77 = load ptr, ptr %5, align 8
  br label %79

78:                                               ; preds = %70
  br label %79

79:                                               ; preds = %78, %76
  %80 = phi ptr [ %77, %76 ], [ null, %78 ]
  store ptr %80, ptr %5, align 8
  br label %238

81:                                               ; preds = %33
  %82 = load ptr, ptr %6, align 8
  %83 = getelementptr inbounds i8, ptr %82, i64 1
  %84 = load i8, ptr %83, align 1
  %85 = sext i8 %84 to i32
  switch i32 %85, label %166 [
    i32 98, label %86
    i32 102, label %98
    i32 48, label %152
    i32 49, label %152
    i32 50, label %152
    i32 51, label %152
    i32 52, label %152
    i32 53, label %152
    i32 54, label %152
    i32 55, label %152
    i32 56, label %152
    i32 57, label %152
  ]

86:                                               ; preds = %81
  %87 = load ptr, ptr %4, align 8
  %88 = load ptr, ptr %5, align 8
  %89 = load ptr, ptr %6, align 8
  %90 = getelementptr inbounds i8, ptr %89, i64 2
  %91 = call ptr @matchbalance(ptr noundef %87, ptr noundef %88, ptr noundef %90)
  store ptr %91, ptr %5, align 8
  %92 = load ptr, ptr %5, align 8
  %93 = icmp ne ptr %92, null
  br i1 %93, label %94, label %97

94:                                               ; preds = %86
  %95 = load ptr, ptr %6, align 8
  %96 = getelementptr inbounds i8, ptr %95, i64 4
  store ptr %96, ptr %6, align 8
  br label %27

97:                                               ; preds = %86
  br label %167

98:                                               ; preds = %81
  %99 = load ptr, ptr %6, align 8
  %100 = getelementptr inbounds i8, ptr %99, i64 2
  store ptr %100, ptr %6, align 8
  %101 = load ptr, ptr %6, align 8
  %102 = load i8, ptr %101, align 1
  %103 = sext i8 %102 to i32
  %104 = icmp ne i32 %103, 91
  %105 = zext i1 %104 to i32
  %106 = icmp ne i32 %105, 0
  %107 = zext i1 %106 to i32
  %108 = sext i32 %107 to i64
  %109 = icmp ne i64 %108, 0
  br i1 %109, label %110, label %115

110:                                              ; preds = %98
  %111 = load ptr, ptr %4, align 8
  %112 = getelementptr inbounds %struct.MatchState, ptr %111, i32 0, i32 3
  %113 = load ptr, ptr %112, align 8
  %114 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %113, ptr noundef @.str.22)
  br label %115

115:                                              ; preds = %110, %98
  %116 = load ptr, ptr %4, align 8
  %117 = load ptr, ptr %6, align 8
  %118 = call ptr @classend(ptr noundef %116, ptr noundef %117)
  store ptr %118, ptr %7, align 8
  %119 = load ptr, ptr %5, align 8
  %120 = load ptr, ptr %4, align 8
  %121 = getelementptr inbounds %struct.MatchState, ptr %120, i32 0, i32 0
  %122 = load ptr, ptr %121, align 8
  %123 = icmp eq ptr %119, %122
  br i1 %123, label %124, label %125

124:                                              ; preds = %115
  br label %130

125:                                              ; preds = %115
  %126 = load ptr, ptr %5, align 8
  %127 = getelementptr inbounds i8, ptr %126, i64 -1
  %128 = load i8, ptr %127, align 1
  %129 = sext i8 %128 to i32
  br label %130

130:                                              ; preds = %125, %124
  %131 = phi i32 [ 0, %124 ], [ %129, %125 ]
  %132 = trunc i32 %131 to i8
  store i8 %132, ptr %8, align 1
  %133 = load i8, ptr %8, align 1
  %134 = zext i8 %133 to i32
  %135 = load ptr, ptr %6, align 8
  %136 = load ptr, ptr %7, align 8
  %137 = getelementptr inbounds i8, ptr %136, i64 -1
  %138 = call i32 @matchbracketclass(i32 noundef %134, ptr noundef %135, ptr noundef %137)
  %139 = icmp ne i32 %138, 0
  br i1 %139, label %151, label %140

140:                                              ; preds = %130
  %141 = load ptr, ptr %5, align 8
  %142 = load i8, ptr %141, align 1
  %143 = zext i8 %142 to i32
  %144 = load ptr, ptr %6, align 8
  %145 = load ptr, ptr %7, align 8
  %146 = getelementptr inbounds i8, ptr %145, i64 -1
  %147 = call i32 @matchbracketclass(i32 noundef %143, ptr noundef %144, ptr noundef %146)
  %148 = icmp ne i32 %147, 0
  br i1 %148, label %149, label %151

149:                                              ; preds = %140
  %150 = load ptr, ptr %7, align 8
  store ptr %150, ptr %6, align 8
  br label %27

151:                                              ; preds = %140, %130
  store ptr null, ptr %5, align 8
  br label %167

152:                                              ; preds = %81, %81, %81, %81, %81, %81, %81, %81, %81, %81
  %153 = load ptr, ptr %4, align 8
  %154 = load ptr, ptr %5, align 8
  %155 = load ptr, ptr %6, align 8
  %156 = getelementptr inbounds i8, ptr %155, i64 1
  %157 = load i8, ptr %156, align 1
  %158 = zext i8 %157 to i32
  %159 = call ptr @match_capture(ptr noundef %153, ptr noundef %154, i32 noundef %158)
  store ptr %159, ptr %5, align 8
  %160 = load ptr, ptr %5, align 8
  %161 = icmp ne ptr %160, null
  br i1 %161, label %162, label %165

162:                                              ; preds = %152
  %163 = load ptr, ptr %6, align 8
  %164 = getelementptr inbounds i8, ptr %163, i64 2
  store ptr %164, ptr %6, align 8
  br label %27

165:                                              ; preds = %152
  br label %167

166:                                              ; preds = %81
  br label %169

167:                                              ; preds = %165, %151, %97
  br label %238

168:                                              ; preds = %33
  br label %169

169:                                              ; preds = %168, %166, %69
  %170 = load ptr, ptr %4, align 8
  %171 = load ptr, ptr %6, align 8
  %172 = call ptr @classend(ptr noundef %170, ptr noundef %171)
  store ptr %172, ptr %9, align 8
  %173 = load ptr, ptr %4, align 8
  %174 = load ptr, ptr %5, align 8
  %175 = load ptr, ptr %6, align 8
  %176 = load ptr, ptr %9, align 8
  %177 = call i32 @singlematch(ptr noundef %173, ptr noundef %174, ptr noundef %175, ptr noundef %176)
  %178 = icmp ne i32 %177, 0
  br i1 %178, label %199, label %179

179:                                              ; preds = %169
  %180 = load ptr, ptr %9, align 8
  %181 = load i8, ptr %180, align 1
  %182 = sext i8 %181 to i32
  %183 = icmp eq i32 %182, 42
  br i1 %183, label %194, label %184

184:                                              ; preds = %179
  %185 = load ptr, ptr %9, align 8
  %186 = load i8, ptr %185, align 1
  %187 = sext i8 %186 to i32
  %188 = icmp eq i32 %187, 63
  br i1 %188, label %194, label %189

189:                                              ; preds = %184
  %190 = load ptr, ptr %9, align 8
  %191 = load i8, ptr %190, align 1
  %192 = sext i8 %191 to i32
  %193 = icmp eq i32 %192, 45
  br i1 %193, label %194, label %197

194:                                              ; preds = %189, %184, %179
  %195 = load ptr, ptr %9, align 8
  %196 = getelementptr inbounds i8, ptr %195, i64 1
  store ptr %196, ptr %6, align 8
  br label %27

197:                                              ; preds = %189
  store ptr null, ptr %5, align 8
  br label %198

198:                                              ; preds = %197
  br label %237

199:                                              ; preds = %169
  %200 = load ptr, ptr %9, align 8
  %201 = load i8, ptr %200, align 1
  %202 = sext i8 %201 to i32
  switch i32 %202, label %232 [
    i32 63, label %203
    i32 43, label %217
    i32 42, label %220
    i32 45, label %226
  ]

203:                                              ; preds = %199
  %204 = load ptr, ptr %4, align 8
  %205 = load ptr, ptr %5, align 8
  %206 = getelementptr inbounds i8, ptr %205, i64 1
  %207 = load ptr, ptr %9, align 8
  %208 = getelementptr inbounds i8, ptr %207, i64 1
  %209 = call ptr @match(ptr noundef %204, ptr noundef %206, ptr noundef %208)
  store ptr %209, ptr %10, align 8
  %210 = icmp ne ptr %209, null
  br i1 %210, label %211, label %213

211:                                              ; preds = %203
  %212 = load ptr, ptr %10, align 8
  store ptr %212, ptr %5, align 8
  br label %216

213:                                              ; preds = %203
  %214 = load ptr, ptr %9, align 8
  %215 = getelementptr inbounds i8, ptr %214, i64 1
  store ptr %215, ptr %6, align 8
  br label %27

216:                                              ; preds = %211
  br label %236

217:                                              ; preds = %199
  %218 = load ptr, ptr %5, align 8
  %219 = getelementptr inbounds i8, ptr %218, i32 1
  store ptr %219, ptr %5, align 8
  br label %220

220:                                              ; preds = %199, %217
  %221 = load ptr, ptr %4, align 8
  %222 = load ptr, ptr %5, align 8
  %223 = load ptr, ptr %6, align 8
  %224 = load ptr, ptr %9, align 8
  %225 = call ptr @max_expand(ptr noundef %221, ptr noundef %222, ptr noundef %223, ptr noundef %224)
  store ptr %225, ptr %5, align 8
  br label %236

226:                                              ; preds = %199
  %227 = load ptr, ptr %4, align 8
  %228 = load ptr, ptr %5, align 8
  %229 = load ptr, ptr %6, align 8
  %230 = load ptr, ptr %9, align 8
  %231 = call ptr @min_expand(ptr noundef %227, ptr noundef %228, ptr noundef %229, ptr noundef %230)
  store ptr %231, ptr %5, align 8
  br label %236

232:                                              ; preds = %199
  %233 = load ptr, ptr %5, align 8
  %234 = getelementptr inbounds i8, ptr %233, i32 1
  store ptr %234, ptr %5, align 8
  %235 = load ptr, ptr %9, align 8
  store ptr %235, ptr %6, align 8
  br label %27

236:                                              ; preds = %226, %220, %216
  br label %237

237:                                              ; preds = %236, %198
  br label %238

238:                                              ; preds = %237, %167, %79, %56, %55
  br label %239

239:                                              ; preds = %238, %27
  %240 = load ptr, ptr %4, align 8
  %241 = getelementptr inbounds %struct.MatchState, ptr %240, i32 0, i32 4
  %242 = load i32, ptr %241, align 8
  %243 = add nsw i32 %242, 1
  store i32 %243, ptr %241, align 8
  %244 = load ptr, ptr %5, align 8
  ret ptr %244
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @push_captures(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.MatchState, ptr %9, i32 0, i32 5
  %11 = load i8, ptr %10, align 4
  %12 = zext i8 %11 to i32
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %14, label %18

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  %16 = icmp ne ptr %15, null
  br i1 %16, label %17, label %18

17:                                               ; preds = %14
  br label %23

18:                                               ; preds = %14, %3
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.MatchState, ptr %19, i32 0, i32 5
  %21 = load i8, ptr %20, align 4
  %22 = zext i8 %21 to i32
  br label %23

23:                                               ; preds = %18, %17
  %24 = phi i32 [ 1, %17 ], [ %22, %18 ]
  store i32 %24, ptr %8, align 4
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.MatchState, ptr %25, i32 0, i32 3
  %27 = load ptr, ptr %26, align 8
  %28 = load i32, ptr %8, align 4
  call void @luaL_checkstack(ptr noundef %27, i32 noundef %28, ptr noundef @.str.23)
  store i32 0, ptr %7, align 4
  br label %29

29:                                               ; preds = %38, %23
  %30 = load i32, ptr %7, align 4
  %31 = load i32, ptr %8, align 4
  %32 = icmp slt i32 %30, %31
  br i1 %32, label %33, label %41

33:                                               ; preds = %29
  %34 = load ptr, ptr %4, align 8
  %35 = load i32, ptr %7, align 4
  %36 = load ptr, ptr %5, align 8
  %37 = load ptr, ptr %6, align 8
  call void @push_onecapture(ptr noundef %34, i32 noundef %35, ptr noundef %36, ptr noundef %37)
  br label %38

38:                                               ; preds = %33
  %39 = load i32, ptr %7, align 4
  %40 = add nsw i32 %39, 1
  store i32 %40, ptr %7, align 4
  br label %29, !llvm.loop !23

41:                                               ; preds = %29
  %42 = load i32, ptr %8, align 4
  ret i32 %42
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strpbrk(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare ptr @memchr(ptr noundef, i32 noundef, i64 noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare i32 @memcmp(ptr noundef, ptr noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @start_capture(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = getelementptr inbounds %struct.MatchState, ptr %11, i32 0, i32 5
  %13 = load i8, ptr %12, align 4
  %14 = zext i8 %13 to i32
  store i32 %14, ptr %10, align 4
  %15 = load i32, ptr %10, align 4
  %16 = icmp sge i32 %15, 32
  br i1 %16, label %17, label %22

17:                                               ; preds = %4
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds %struct.MatchState, ptr %18, i32 0, i32 3
  %20 = load ptr, ptr %19, align 8
  %21 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %20, ptr noundef @.str.23)
  br label %22

22:                                               ; preds = %17, %4
  %23 = load ptr, ptr %6, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.MatchState, ptr %24, i32 0, i32 6
  %26 = load i32, ptr %10, align 4
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds [32 x %struct.anon], ptr %25, i64 0, i64 %27
  %29 = getelementptr inbounds %struct.anon, ptr %28, i32 0, i32 0
  store ptr %23, ptr %29, align 8
  %30 = load i32, ptr %8, align 4
  %31 = sext i32 %30 to i64
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds %struct.MatchState, ptr %32, i32 0, i32 6
  %34 = load i32, ptr %10, align 4
  %35 = sext i32 %34 to i64
  %36 = getelementptr inbounds [32 x %struct.anon], ptr %33, i64 0, i64 %35
  %37 = getelementptr inbounds %struct.anon, ptr %36, i32 0, i32 1
  store i64 %31, ptr %37, align 8
  %38 = load i32, ptr %10, align 4
  %39 = add nsw i32 %38, 1
  %40 = trunc i32 %39 to i8
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.MatchState, ptr %41, i32 0, i32 5
  store i8 %40, ptr %42, align 4
  %43 = load ptr, ptr %5, align 8
  %44 = load ptr, ptr %6, align 8
  %45 = load ptr, ptr %7, align 8
  %46 = call ptr @match(ptr noundef %43, ptr noundef %44, ptr noundef %45)
  store ptr %46, ptr %9, align 8
  %47 = icmp eq ptr %46, null
  br i1 %47, label %48, label %53

48:                                               ; preds = %22
  %49 = load ptr, ptr %5, align 8
  %50 = getelementptr inbounds %struct.MatchState, ptr %49, i32 0, i32 5
  %51 = load i8, ptr %50, align 4
  %52 = add i8 %51, -1
  store i8 %52, ptr %50, align 4
  br label %53

53:                                               ; preds = %48, %22
  %54 = load ptr, ptr %9, align 8
  ret ptr %54
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @end_capture(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = call i32 @capture_to_close(ptr noundef %9)
  store i32 %10, ptr %7, align 4
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 6
  %14 = load i32, ptr %7, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds [32 x %struct.anon], ptr %13, i64 0, i64 %15
  %17 = getelementptr inbounds %struct.anon, ptr %16, i32 0, i32 0
  %18 = load ptr, ptr %17, align 8
  %19 = ptrtoint ptr %11 to i64
  %20 = ptrtoint ptr %18 to i64
  %21 = sub i64 %19, %20
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.MatchState, ptr %22, i32 0, i32 6
  %24 = load i32, ptr %7, align 4
  %25 = sext i32 %24 to i64
  %26 = getelementptr inbounds [32 x %struct.anon], ptr %23, i64 0, i64 %25
  %27 = getelementptr inbounds %struct.anon, ptr %26, i32 0, i32 1
  store i64 %21, ptr %27, align 8
  %28 = load ptr, ptr %4, align 8
  %29 = load ptr, ptr %5, align 8
  %30 = load ptr, ptr %6, align 8
  %31 = call ptr @match(ptr noundef %28, ptr noundef %29, ptr noundef %30)
  store ptr %31, ptr %8, align 8
  %32 = icmp eq ptr %31, null
  br i1 %32, label %33, label %40

33:                                               ; preds = %3
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.MatchState, ptr %34, i32 0, i32 6
  %36 = load i32, ptr %7, align 4
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds [32 x %struct.anon], ptr %35, i64 0, i64 %37
  %39 = getelementptr inbounds %struct.anon, ptr %38, i32 0, i32 1
  store i64 -1, ptr %39, align 8
  br label %40

40:                                               ; preds = %33, %3
  %41 = load ptr, ptr %8, align 8
  ret ptr %41
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @matchbalance(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 2
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds i8, ptr %14, i64 -1
  %16 = icmp uge ptr %11, %15
  %17 = zext i1 %16 to i32
  %18 = icmp ne i32 %17, 0
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %22, label %27

22:                                               ; preds = %3
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %struct.MatchState, ptr %23, i32 0, i32 3
  %25 = load ptr, ptr %24, align 8
  %26 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %25, ptr noundef @.str.25)
  br label %27

27:                                               ; preds = %22, %3
  %28 = load ptr, ptr %6, align 8
  %29 = load i8, ptr %28, align 1
  %30 = sext i8 %29 to i32
  %31 = load ptr, ptr %7, align 8
  %32 = load i8, ptr %31, align 1
  %33 = sext i8 %32 to i32
  %34 = icmp ne i32 %30, %33
  br i1 %34, label %35, label %36

35:                                               ; preds = %27
  store ptr null, ptr %4, align 8
  br label %78

36:                                               ; preds = %27
  %37 = load ptr, ptr %7, align 8
  %38 = load i8, ptr %37, align 1
  %39 = sext i8 %38 to i32
  store i32 %39, ptr %8, align 4
  %40 = load ptr, ptr %7, align 8
  %41 = getelementptr inbounds i8, ptr %40, i64 1
  %42 = load i8, ptr %41, align 1
  %43 = sext i8 %42 to i32
  store i32 %43, ptr %9, align 4
  store i32 1, ptr %10, align 4
  br label %44

44:                                               ; preds = %75, %36
  %45 = load ptr, ptr %6, align 8
  %46 = getelementptr inbounds i8, ptr %45, i32 1
  store ptr %46, ptr %6, align 8
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %struct.MatchState, ptr %47, i32 0, i32 1
  %49 = load ptr, ptr %48, align 8
  %50 = icmp ult ptr %46, %49
  br i1 %50, label %51, label %76

51:                                               ; preds = %44
  %52 = load ptr, ptr %6, align 8
  %53 = load i8, ptr %52, align 1
  %54 = sext i8 %53 to i32
  %55 = load i32, ptr %9, align 4
  %56 = icmp eq i32 %54, %55
  br i1 %56, label %57, label %65

57:                                               ; preds = %51
  %58 = load i32, ptr %10, align 4
  %59 = add nsw i32 %58, -1
  store i32 %59, ptr %10, align 4
  %60 = icmp eq i32 %59, 0
  br i1 %60, label %61, label %64

61:                                               ; preds = %57
  %62 = load ptr, ptr %6, align 8
  %63 = getelementptr inbounds i8, ptr %62, i64 1
  store ptr %63, ptr %4, align 8
  br label %78

64:                                               ; preds = %57
  br label %75

65:                                               ; preds = %51
  %66 = load ptr, ptr %6, align 8
  %67 = load i8, ptr %66, align 1
  %68 = sext i8 %67 to i32
  %69 = load i32, ptr %8, align 4
  %70 = icmp eq i32 %68, %69
  br i1 %70, label %71, label %74

71:                                               ; preds = %65
  %72 = load i32, ptr %10, align 4
  %73 = add nsw i32 %72, 1
  store i32 %73, ptr %10, align 4
  br label %74

74:                                               ; preds = %71, %65
  br label %75

75:                                               ; preds = %74, %64
  br label %44, !llvm.loop !24

76:                                               ; preds = %44
  br label %77

77:                                               ; preds = %76
  store ptr null, ptr %4, align 8
  br label %78

78:                                               ; preds = %77, %61, %35
  %79 = load ptr, ptr %4, align 8
  ret ptr %79
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @classend(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %5, align 8
  %7 = getelementptr inbounds i8, ptr %6, i32 1
  store ptr %7, ptr %5, align 8
  %8 = load i8, ptr %6, align 1
  %9 = sext i8 %8 to i32
  switch i32 %9, label %78 [
    i32 37, label %10
    i32 91, label %29
  ]

10:                                               ; preds = %2
  %11 = load ptr, ptr %5, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 2
  %14 = load ptr, ptr %13, align 8
  %15 = icmp eq ptr %11, %14
  %16 = zext i1 %15 to i32
  %17 = icmp ne i32 %16, 0
  %18 = zext i1 %17 to i32
  %19 = sext i32 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %21, label %26

21:                                               ; preds = %10
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.MatchState, ptr %22, i32 0, i32 3
  %24 = load ptr, ptr %23, align 8
  %25 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %24, ptr noundef @.str.26)
  br label %26

26:                                               ; preds = %21, %10
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds i8, ptr %27, i64 1
  store ptr %28, ptr %3, align 8
  br label %80

29:                                               ; preds = %2
  %30 = load ptr, ptr %5, align 8
  %31 = load i8, ptr %30, align 1
  %32 = sext i8 %31 to i32
  %33 = icmp eq i32 %32, 94
  br i1 %33, label %34, label %37

34:                                               ; preds = %29
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds i8, ptr %35, i32 1
  store ptr %36, ptr %5, align 8
  br label %37

37:                                               ; preds = %34, %29
  br label %38

38:                                               ; preds = %70, %37
  %39 = load ptr, ptr %5, align 8
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.MatchState, ptr %40, i32 0, i32 2
  %42 = load ptr, ptr %41, align 8
  %43 = icmp eq ptr %39, %42
  %44 = zext i1 %43 to i32
  %45 = icmp ne i32 %44, 0
  %46 = zext i1 %45 to i32
  %47 = sext i32 %46 to i64
  %48 = icmp ne i64 %47, 0
  br i1 %48, label %49, label %54

49:                                               ; preds = %38
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.MatchState, ptr %50, i32 0, i32 3
  %52 = load ptr, ptr %51, align 8
  %53 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %52, ptr noundef @.str.27)
  br label %54

54:                                               ; preds = %49, %38
  %55 = load ptr, ptr %5, align 8
  %56 = getelementptr inbounds i8, ptr %55, i32 1
  store ptr %56, ptr %5, align 8
  %57 = load i8, ptr %55, align 1
  %58 = sext i8 %57 to i32
  %59 = icmp eq i32 %58, 37
  br i1 %59, label %60, label %69

60:                                               ; preds = %54
  %61 = load ptr, ptr %5, align 8
  %62 = load ptr, ptr %4, align 8
  %63 = getelementptr inbounds %struct.MatchState, ptr %62, i32 0, i32 2
  %64 = load ptr, ptr %63, align 8
  %65 = icmp ult ptr %61, %64
  br i1 %65, label %66, label %69

66:                                               ; preds = %60
  %67 = load ptr, ptr %5, align 8
  %68 = getelementptr inbounds i8, ptr %67, i32 1
  store ptr %68, ptr %5, align 8
  br label %69

69:                                               ; preds = %66, %60, %54
  br label %70

70:                                               ; preds = %69
  %71 = load ptr, ptr %5, align 8
  %72 = load i8, ptr %71, align 1
  %73 = sext i8 %72 to i32
  %74 = icmp ne i32 %73, 93
  br i1 %74, label %38, label %75, !llvm.loop !25

75:                                               ; preds = %70
  %76 = load ptr, ptr %5, align 8
  %77 = getelementptr inbounds i8, ptr %76, i64 1
  store ptr %77, ptr %3, align 8
  br label %80

78:                                               ; preds = %2
  %79 = load ptr, ptr %5, align 8
  store ptr %79, ptr %3, align 8
  br label %80

80:                                               ; preds = %78, %75, %26
  %81 = load ptr, ptr %3, align 8
  ret ptr %81
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @matchbracketclass(i32 noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store i32 %0, ptr %5, align 4
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 1, ptr %8, align 4
  %9 = load ptr, ptr %6, align 8
  %10 = getelementptr inbounds i8, ptr %9, i64 1
  %11 = load i8, ptr %10, align 1
  %12 = sext i8 %11 to i32
  %13 = icmp eq i32 %12, 94
  br i1 %13, label %14, label %17

14:                                               ; preds = %3
  store i32 0, ptr %8, align 4
  %15 = load ptr, ptr %6, align 8
  %16 = getelementptr inbounds i8, ptr %15, i32 1
  store ptr %16, ptr %6, align 8
  br label %17

17:                                               ; preds = %14, %3
  br label %18

18:                                               ; preds = %79, %17
  %19 = load ptr, ptr %6, align 8
  %20 = getelementptr inbounds i8, ptr %19, i32 1
  store ptr %20, ptr %6, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = icmp ult ptr %20, %21
  br i1 %22, label %23, label %80

23:                                               ; preds = %18
  %24 = load ptr, ptr %6, align 8
  %25 = load i8, ptr %24, align 1
  %26 = sext i8 %25 to i32
  %27 = icmp eq i32 %26, 37
  br i1 %27, label %28, label %40

28:                                               ; preds = %23
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds i8, ptr %29, i32 1
  store ptr %30, ptr %6, align 8
  %31 = load i32, ptr %5, align 4
  %32 = load ptr, ptr %6, align 8
  %33 = load i8, ptr %32, align 1
  %34 = zext i8 %33 to i32
  %35 = call i32 @match_class(i32 noundef %31, i32 noundef %34)
  %36 = icmp ne i32 %35, 0
  br i1 %36, label %37, label %39

37:                                               ; preds = %28
  %38 = load i32, ptr %8, align 4
  store i32 %38, ptr %4, align 4
  br label %85

39:                                               ; preds = %28
  br label %79

40:                                               ; preds = %23
  %41 = load ptr, ptr %6, align 8
  %42 = getelementptr inbounds i8, ptr %41, i64 1
  %43 = load i8, ptr %42, align 1
  %44 = sext i8 %43 to i32
  %45 = icmp eq i32 %44, 45
  br i1 %45, label %46, label %69

46:                                               ; preds = %40
  %47 = load ptr, ptr %6, align 8
  %48 = getelementptr inbounds i8, ptr %47, i64 2
  %49 = load ptr, ptr %7, align 8
  %50 = icmp ult ptr %48, %49
  br i1 %50, label %51, label %69

51:                                               ; preds = %46
  %52 = load ptr, ptr %6, align 8
  %53 = getelementptr inbounds i8, ptr %52, i64 2
  store ptr %53, ptr %6, align 8
  %54 = load ptr, ptr %6, align 8
  %55 = getelementptr inbounds i8, ptr %54, i64 -2
  %56 = load i8, ptr %55, align 1
  %57 = zext i8 %56 to i32
  %58 = load i32, ptr %5, align 4
  %59 = icmp sle i32 %57, %58
  br i1 %59, label %60, label %68

60:                                               ; preds = %51
  %61 = load i32, ptr %5, align 4
  %62 = load ptr, ptr %6, align 8
  %63 = load i8, ptr %62, align 1
  %64 = zext i8 %63 to i32
  %65 = icmp sle i32 %61, %64
  br i1 %65, label %66, label %68

66:                                               ; preds = %60
  %67 = load i32, ptr %8, align 4
  store i32 %67, ptr %4, align 4
  br label %85

68:                                               ; preds = %60, %51
  br label %78

69:                                               ; preds = %46, %40
  %70 = load ptr, ptr %6, align 8
  %71 = load i8, ptr %70, align 1
  %72 = zext i8 %71 to i32
  %73 = load i32, ptr %5, align 4
  %74 = icmp eq i32 %72, %73
  br i1 %74, label %75, label %77

75:                                               ; preds = %69
  %76 = load i32, ptr %8, align 4
  store i32 %76, ptr %4, align 4
  br label %85

77:                                               ; preds = %69
  br label %78

78:                                               ; preds = %77, %68
  br label %79

79:                                               ; preds = %78, %39
  br label %18, !llvm.loop !26

80:                                               ; preds = %18
  %81 = load i32, ptr %8, align 4
  %82 = icmp ne i32 %81, 0
  %83 = xor i1 %82, true
  %84 = zext i1 %83 to i32
  store i32 %84, ptr %4, align 4
  br label %85

85:                                               ; preds = %80, %75, %66, %37
  %86 = load i32, ptr %4, align 4
  ret i32 %86
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @match_capture(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = load i32, ptr %7, align 4
  %11 = call i32 @check_capture(ptr noundef %9, i32 noundef %10)
  store i32 %11, ptr %7, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 6
  %14 = load i32, ptr %7, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds [32 x %struct.anon], ptr %13, i64 0, i64 %15
  %17 = getelementptr inbounds %struct.anon, ptr %16, i32 0, i32 1
  %18 = load i64, ptr %17, align 8
  store i64 %18, ptr %8, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.MatchState, ptr %19, i32 0, i32 1
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %6, align 8
  %23 = ptrtoint ptr %21 to i64
  %24 = ptrtoint ptr %22 to i64
  %25 = sub i64 %23, %24
  %26 = load i64, ptr %8, align 8
  %27 = icmp uge i64 %25, %26
  br i1 %27, label %28, label %44

28:                                               ; preds = %3
  %29 = load ptr, ptr %5, align 8
  %30 = getelementptr inbounds %struct.MatchState, ptr %29, i32 0, i32 6
  %31 = load i32, ptr %7, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds [32 x %struct.anon], ptr %30, i64 0, i64 %32
  %34 = getelementptr inbounds %struct.anon, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %6, align 8
  %37 = load i64, ptr %8, align 8
  %38 = call i32 @memcmp(ptr noundef %35, ptr noundef %36, i64 noundef %37) #7
  %39 = icmp eq i32 %38, 0
  br i1 %39, label %40, label %44

40:                                               ; preds = %28
  %41 = load ptr, ptr %6, align 8
  %42 = load i64, ptr %8, align 8
  %43 = getelementptr inbounds i8, ptr %41, i64 %42
  store ptr %43, ptr %4, align 8
  br label %45

44:                                               ; preds = %28, %3
  store ptr null, ptr %4, align 8
  br label %45

45:                                               ; preds = %44, %40
  %46 = load ptr, ptr %4, align 8
  ret ptr %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @singlematch(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 1
  %14 = load ptr, ptr %13, align 8
  %15 = icmp uge ptr %11, %14
  br i1 %15, label %16, label %17

16:                                               ; preds = %4
  store i32 0, ptr %5, align 4
  br label %45

17:                                               ; preds = %4
  %18 = load ptr, ptr %7, align 8
  %19 = load i8, ptr %18, align 1
  %20 = zext i8 %19 to i32
  store i32 %20, ptr %10, align 4
  %21 = load ptr, ptr %8, align 8
  %22 = load i8, ptr %21, align 1
  %23 = sext i8 %22 to i32
  switch i32 %23, label %38 [
    i32 46, label %24
    i32 37, label %25
    i32 91, label %32
  ]

24:                                               ; preds = %17
  store i32 1, ptr %5, align 4
  br label %45

25:                                               ; preds = %17
  %26 = load i32, ptr %10, align 4
  %27 = load ptr, ptr %8, align 8
  %28 = getelementptr inbounds i8, ptr %27, i64 1
  %29 = load i8, ptr %28, align 1
  %30 = zext i8 %29 to i32
  %31 = call i32 @match_class(i32 noundef %26, i32 noundef %30)
  store i32 %31, ptr %5, align 4
  br label %45

32:                                               ; preds = %17
  %33 = load i32, ptr %10, align 4
  %34 = load ptr, ptr %8, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds i8, ptr %35, i64 -1
  %37 = call i32 @matchbracketclass(i32 noundef %33, ptr noundef %34, ptr noundef %36)
  store i32 %37, ptr %5, align 4
  br label %45

38:                                               ; preds = %17
  %39 = load ptr, ptr %8, align 8
  %40 = load i8, ptr %39, align 1
  %41 = zext i8 %40 to i32
  %42 = load i32, ptr %10, align 4
  %43 = icmp eq i32 %41, %42
  %44 = zext i1 %43 to i32
  store i32 %44, ptr %5, align 4
  br label %45

45:                                               ; preds = %38, %32, %25, %24, %16
  %46 = load i32, ptr %5, align 4
  ret i32 %46
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @max_expand(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i64 0, ptr %10, align 8
  br label %12

12:                                               ; preds = %21, %4
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = load i64, ptr %10, align 8
  %16 = getelementptr inbounds i8, ptr %14, i64 %15
  %17 = load ptr, ptr %8, align 8
  %18 = load ptr, ptr %9, align 8
  %19 = call i32 @singlematch(ptr noundef %13, ptr noundef %16, ptr noundef %17, ptr noundef %18)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %24

21:                                               ; preds = %12
  %22 = load i64, ptr %10, align 8
  %23 = add nsw i64 %22, 1
  store i64 %23, ptr %10, align 8
  br label %12, !llvm.loop !27

24:                                               ; preds = %12
  br label %25

25:                                               ; preds = %40, %24
  %26 = load i64, ptr %10, align 8
  %27 = icmp sge i64 %26, 0
  br i1 %27, label %28, label %43

28:                                               ; preds = %25
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %7, align 8
  %31 = load i64, ptr %10, align 8
  %32 = getelementptr inbounds i8, ptr %30, i64 %31
  %33 = load ptr, ptr %9, align 8
  %34 = getelementptr inbounds i8, ptr %33, i64 1
  %35 = call ptr @match(ptr noundef %29, ptr noundef %32, ptr noundef %34)
  store ptr %35, ptr %11, align 8
  %36 = load ptr, ptr %11, align 8
  %37 = icmp ne ptr %36, null
  br i1 %37, label %38, label %40

38:                                               ; preds = %28
  %39 = load ptr, ptr %11, align 8
  store ptr %39, ptr %5, align 8
  br label %44

40:                                               ; preds = %28
  %41 = load i64, ptr %10, align 8
  %42 = add nsw i64 %41, -1
  store i64 %42, ptr %10, align 8
  br label %25, !llvm.loop !28

43:                                               ; preds = %25
  store ptr null, ptr %5, align 8
  br label %44

44:                                               ; preds = %43, %38
  %45 = load ptr, ptr %5, align 8
  ret ptr %45
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @min_expand(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  br label %11

11:                                               ; preds = %33, %4
  %12 = load ptr, ptr %6, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = getelementptr inbounds i8, ptr %14, i64 1
  %16 = call ptr @match(ptr noundef %12, ptr noundef %13, ptr noundef %15)
  store ptr %16, ptr %10, align 8
  %17 = load ptr, ptr %10, align 8
  %18 = icmp ne ptr %17, null
  br i1 %18, label %19, label %21

19:                                               ; preds = %11
  %20 = load ptr, ptr %10, align 8
  store ptr %20, ptr %5, align 8
  br label %34

21:                                               ; preds = %11
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = load ptr, ptr %8, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = call i32 @singlematch(ptr noundef %22, ptr noundef %23, ptr noundef %24, ptr noundef %25)
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %31

28:                                               ; preds = %21
  %29 = load ptr, ptr %7, align 8
  %30 = getelementptr inbounds i8, ptr %29, i32 1
  store ptr %30, ptr %7, align 8
  br label %32

31:                                               ; preds = %21
  store ptr null, ptr %5, align 8
  br label %34

32:                                               ; preds = %28
  br label %33

33:                                               ; preds = %32
  br label %11

34:                                               ; preds = %31, %19
  %35 = load ptr, ptr %5, align 8
  ret ptr %35
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @capture_to_close(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %struct.MatchState, ptr %5, i32 0, i32 5
  %7 = load i8, ptr %6, align 4
  %8 = zext i8 %7 to i32
  store i32 %8, ptr %4, align 4
  %9 = load i32, ptr %4, align 4
  %10 = add nsw i32 %9, -1
  store i32 %10, ptr %4, align 4
  br label %11

11:                                               ; preds = %26, %1
  %12 = load i32, ptr %4, align 4
  %13 = icmp sge i32 %12, 0
  br i1 %13, label %14, label %29

14:                                               ; preds = %11
  %15 = load ptr, ptr %3, align 8
  %16 = getelementptr inbounds %struct.MatchState, ptr %15, i32 0, i32 6
  %17 = load i32, ptr %4, align 4
  %18 = sext i32 %17 to i64
  %19 = getelementptr inbounds [32 x %struct.anon], ptr %16, i64 0, i64 %18
  %20 = getelementptr inbounds %struct.anon, ptr %19, i32 0, i32 1
  %21 = load i64, ptr %20, align 8
  %22 = icmp eq i64 %21, -1
  br i1 %22, label %23, label %25

23:                                               ; preds = %14
  %24 = load i32, ptr %4, align 4
  store i32 %24, ptr %2, align 4
  br label %34

25:                                               ; preds = %14
  br label %26

26:                                               ; preds = %25
  %27 = load i32, ptr %4, align 4
  %28 = add nsw i32 %27, -1
  store i32 %28, ptr %4, align 4
  br label %11, !llvm.loop !29

29:                                               ; preds = %11
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.MatchState, ptr %30, i32 0, i32 3
  %32 = load ptr, ptr %31, align 8
  %33 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %32, ptr noundef @.str.24)
  store i32 %33, ptr %2, align 4
  br label %34

34:                                               ; preds = %29, %23
  %35 = load i32, ptr %2, align 4
  ret i32 %35
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @match_class(i32 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  %7 = load i32, ptr %5, align 4
  %8 = call i32 @tolower(i32 noundef %7) #7
  switch i32 %8, label %103 [
    i32 97, label %9
    i32 99, label %18
    i32 100, label %27
    i32 103, label %36
    i32 108, label %45
    i32 112, label %54
    i32 115, label %63
    i32 117, label %72
    i32 119, label %81
    i32 120, label %90
    i32 122, label %99
  ]

9:                                                ; preds = %2
  %10 = call ptr @__ctype_b_loc() #8
  %11 = load ptr, ptr %10, align 8
  %12 = load i32, ptr %4, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds i16, ptr %11, i64 %13
  %15 = load i16, ptr %14, align 2
  %16 = zext i16 %15 to i32
  %17 = and i32 %16, 1024
  store i32 %17, ptr %6, align 4
  br label %108

18:                                               ; preds = %2
  %19 = call ptr @__ctype_b_loc() #8
  %20 = load ptr, ptr %19, align 8
  %21 = load i32, ptr %4, align 4
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds i16, ptr %20, i64 %22
  %24 = load i16, ptr %23, align 2
  %25 = zext i16 %24 to i32
  %26 = and i32 %25, 2
  store i32 %26, ptr %6, align 4
  br label %108

27:                                               ; preds = %2
  %28 = call ptr @__ctype_b_loc() #8
  %29 = load ptr, ptr %28, align 8
  %30 = load i32, ptr %4, align 4
  %31 = sext i32 %30 to i64
  %32 = getelementptr inbounds i16, ptr %29, i64 %31
  %33 = load i16, ptr %32, align 2
  %34 = zext i16 %33 to i32
  %35 = and i32 %34, 2048
  store i32 %35, ptr %6, align 4
  br label %108

36:                                               ; preds = %2
  %37 = call ptr @__ctype_b_loc() #8
  %38 = load ptr, ptr %37, align 8
  %39 = load i32, ptr %4, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds i16, ptr %38, i64 %40
  %42 = load i16, ptr %41, align 2
  %43 = zext i16 %42 to i32
  %44 = and i32 %43, 32768
  store i32 %44, ptr %6, align 4
  br label %108

45:                                               ; preds = %2
  %46 = call ptr @__ctype_b_loc() #8
  %47 = load ptr, ptr %46, align 8
  %48 = load i32, ptr %4, align 4
  %49 = sext i32 %48 to i64
  %50 = getelementptr inbounds i16, ptr %47, i64 %49
  %51 = load i16, ptr %50, align 2
  %52 = zext i16 %51 to i32
  %53 = and i32 %52, 512
  store i32 %53, ptr %6, align 4
  br label %108

54:                                               ; preds = %2
  %55 = call ptr @__ctype_b_loc() #8
  %56 = load ptr, ptr %55, align 8
  %57 = load i32, ptr %4, align 4
  %58 = sext i32 %57 to i64
  %59 = getelementptr inbounds i16, ptr %56, i64 %58
  %60 = load i16, ptr %59, align 2
  %61 = zext i16 %60 to i32
  %62 = and i32 %61, 4
  store i32 %62, ptr %6, align 4
  br label %108

63:                                               ; preds = %2
  %64 = call ptr @__ctype_b_loc() #8
  %65 = load ptr, ptr %64, align 8
  %66 = load i32, ptr %4, align 4
  %67 = sext i32 %66 to i64
  %68 = getelementptr inbounds i16, ptr %65, i64 %67
  %69 = load i16, ptr %68, align 2
  %70 = zext i16 %69 to i32
  %71 = and i32 %70, 8192
  store i32 %71, ptr %6, align 4
  br label %108

72:                                               ; preds = %2
  %73 = call ptr @__ctype_b_loc() #8
  %74 = load ptr, ptr %73, align 8
  %75 = load i32, ptr %4, align 4
  %76 = sext i32 %75 to i64
  %77 = getelementptr inbounds i16, ptr %74, i64 %76
  %78 = load i16, ptr %77, align 2
  %79 = zext i16 %78 to i32
  %80 = and i32 %79, 256
  store i32 %80, ptr %6, align 4
  br label %108

81:                                               ; preds = %2
  %82 = call ptr @__ctype_b_loc() #8
  %83 = load ptr, ptr %82, align 8
  %84 = load i32, ptr %4, align 4
  %85 = sext i32 %84 to i64
  %86 = getelementptr inbounds i16, ptr %83, i64 %85
  %87 = load i16, ptr %86, align 2
  %88 = zext i16 %87 to i32
  %89 = and i32 %88, 8
  store i32 %89, ptr %6, align 4
  br label %108

90:                                               ; preds = %2
  %91 = call ptr @__ctype_b_loc() #8
  %92 = load ptr, ptr %91, align 8
  %93 = load i32, ptr %4, align 4
  %94 = sext i32 %93 to i64
  %95 = getelementptr inbounds i16, ptr %92, i64 %94
  %96 = load i16, ptr %95, align 2
  %97 = zext i16 %96 to i32
  %98 = and i32 %97, 4096
  store i32 %98, ptr %6, align 4
  br label %108

99:                                               ; preds = %2
  %100 = load i32, ptr %4, align 4
  %101 = icmp eq i32 %100, 0
  %102 = zext i1 %101 to i32
  store i32 %102, ptr %6, align 4
  br label %108

103:                                              ; preds = %2
  %104 = load i32, ptr %5, align 4
  %105 = load i32, ptr %4, align 4
  %106 = icmp eq i32 %104, %105
  %107 = zext i1 %106 to i32
  store i32 %107, ptr %3, align 4
  br label %127

108:                                              ; preds = %99, %90, %81, %72, %63, %54, %45, %36, %27, %18, %9
  %109 = call ptr @__ctype_b_loc() #8
  %110 = load ptr, ptr %109, align 8
  %111 = load i32, ptr %5, align 4
  %112 = sext i32 %111 to i64
  %113 = getelementptr inbounds i16, ptr %110, i64 %112
  %114 = load i16, ptr %113, align 2
  %115 = zext i16 %114 to i32
  %116 = and i32 %115, 512
  %117 = icmp ne i32 %116, 0
  br i1 %117, label %118, label %120

118:                                              ; preds = %108
  %119 = load i32, ptr %6, align 4
  br label %125

120:                                              ; preds = %108
  %121 = load i32, ptr %6, align 4
  %122 = icmp ne i32 %121, 0
  %123 = xor i1 %122, true
  %124 = zext i1 %123 to i32
  br label %125

125:                                              ; preds = %120, %118
  %126 = phi i32 [ %119, %118 ], [ %124, %120 ]
  store i32 %126, ptr %3, align 4
  br label %127

127:                                              ; preds = %125, %103
  %128 = load i32, ptr %3, align 4
  ret i32 %128
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @tolower(i32 noundef) #2

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__ctype_b_loc() #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @check_capture(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %6 = load i32, ptr %5, align 4
  %7 = sub nsw i32 %6, 49
  store i32 %7, ptr %5, align 4
  %8 = load i32, ptr %5, align 4
  %9 = icmp slt i32 %8, 0
  br i1 %9, label %26, label %10

10:                                               ; preds = %2
  %11 = load i32, ptr %5, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.MatchState, ptr %12, i32 0, i32 5
  %14 = load i8, ptr %13, align 4
  %15 = zext i8 %14 to i32
  %16 = icmp sge i32 %11, %15
  br i1 %16, label %26, label %17

17:                                               ; preds = %10
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.MatchState, ptr %18, i32 0, i32 6
  %20 = load i32, ptr %5, align 4
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds [32 x %struct.anon], ptr %19, i64 0, i64 %21
  %23 = getelementptr inbounds %struct.anon, ptr %22, i32 0, i32 1
  %24 = load i64, ptr %23, align 8
  %25 = icmp eq i64 %24, -1
  br label %26

26:                                               ; preds = %17, %10, %2
  %27 = phi i1 [ true, %10 ], [ true, %2 ], [ %25, %17 ]
  %28 = zext i1 %27 to i32
  %29 = icmp ne i32 %28, 0
  %30 = zext i1 %29 to i32
  %31 = sext i32 %30 to i64
  %32 = icmp ne i64 %31, 0
  br i1 %32, label %33, label %40

33:                                               ; preds = %26
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.MatchState, ptr %34, i32 0, i32 3
  %36 = load ptr, ptr %35, align 8
  %37 = load i32, ptr %5, align 4
  %38 = add nsw i32 %37, 1
  %39 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %36, ptr noundef @.str.28, i32 noundef %38)
  store i32 %39, ptr %3, align 4
  br label %42

40:                                               ; preds = %26
  %41 = load i32, ptr %5, align 4
  store i32 %41, ptr %3, align 4
  br label %42

42:                                               ; preds = %40, %33
  %43 = load i32, ptr %3, align 4
  ret i32 %43
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @push_onecapture(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i32 %1, ptr %6, align 4
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %11 = load ptr, ptr %5, align 8
  %12 = load i32, ptr %6, align 4
  %13 = load ptr, ptr %7, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = call i64 @get_onecapture(ptr noundef %11, i32 noundef %12, ptr noundef %13, ptr noundef %14, ptr noundef %9)
  store i64 %15, ptr %10, align 8
  %16 = load i64, ptr %10, align 8
  %17 = icmp ne i64 %16, -2
  br i1 %17, label %18, label %25

18:                                               ; preds = %4
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.MatchState, ptr %19, i32 0, i32 3
  %21 = load ptr, ptr %20, align 8
  %22 = load ptr, ptr %9, align 8
  %23 = load i64, ptr %10, align 8
  %24 = call ptr @lua_pushlstring(ptr noundef %21, ptr noundef %22, i64 noundef %23)
  br label %25

25:                                               ; preds = %18, %4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @get_onecapture(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i64, align 8
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store ptr %4, ptr %11, align 8
  %13 = load i32, ptr %8, align 4
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.MatchState, ptr %14, i32 0, i32 5
  %16 = load i8, ptr %15, align 4
  %17 = zext i8 %16 to i32
  %18 = icmp sge i32 %13, %17
  br i1 %18, label %19, label %42

19:                                               ; preds = %5
  %20 = load i32, ptr %8, align 4
  %21 = icmp ne i32 %20, 0
  %22 = zext i1 %21 to i32
  %23 = icmp ne i32 %22, 0
  %24 = zext i1 %23 to i32
  %25 = sext i32 %24 to i64
  %26 = icmp ne i64 %25, 0
  br i1 %26, label %27, label %34

27:                                               ; preds = %19
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.MatchState, ptr %28, i32 0, i32 3
  %30 = load ptr, ptr %29, align 8
  %31 = load i32, ptr %8, align 4
  %32 = add nsw i32 %31, 1
  %33 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %30, ptr noundef @.str.28, i32 noundef %32)
  br label %34

34:                                               ; preds = %27, %19
  %35 = load ptr, ptr %9, align 8
  %36 = load ptr, ptr %11, align 8
  store ptr %35, ptr %36, align 8
  %37 = load ptr, ptr %10, align 8
  %38 = load ptr, ptr %9, align 8
  %39 = ptrtoint ptr %37 to i64
  %40 = ptrtoint ptr %38 to i64
  %41 = sub i64 %39, %40
  store i64 %41, ptr %6, align 8
  br label %94

42:                                               ; preds = %5
  %43 = load ptr, ptr %7, align 8
  %44 = getelementptr inbounds %struct.MatchState, ptr %43, i32 0, i32 6
  %45 = load i32, ptr %8, align 4
  %46 = sext i32 %45 to i64
  %47 = getelementptr inbounds [32 x %struct.anon], ptr %44, i64 0, i64 %46
  %48 = getelementptr inbounds %struct.anon, ptr %47, i32 0, i32 1
  %49 = load i64, ptr %48, align 8
  store i64 %49, ptr %12, align 8
  %50 = load ptr, ptr %7, align 8
  %51 = getelementptr inbounds %struct.MatchState, ptr %50, i32 0, i32 6
  %52 = load i32, ptr %8, align 4
  %53 = sext i32 %52 to i64
  %54 = getelementptr inbounds [32 x %struct.anon], ptr %51, i64 0, i64 %53
  %55 = getelementptr inbounds %struct.anon, ptr %54, i32 0, i32 0
  %56 = load ptr, ptr %55, align 8
  %57 = load ptr, ptr %11, align 8
  store ptr %56, ptr %57, align 8
  %58 = load i64, ptr %12, align 8
  %59 = icmp eq i64 %58, -1
  %60 = zext i1 %59 to i32
  %61 = icmp ne i32 %60, 0
  %62 = zext i1 %61 to i32
  %63 = sext i32 %62 to i64
  %64 = icmp ne i64 %63, 0
  br i1 %64, label %65, label %70

65:                                               ; preds = %42
  %66 = load ptr, ptr %7, align 8
  %67 = getelementptr inbounds %struct.MatchState, ptr %66, i32 0, i32 3
  %68 = load ptr, ptr %67, align 8
  %69 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %68, ptr noundef @.str.29)
  br label %92

70:                                               ; preds = %42
  %71 = load i64, ptr %12, align 8
  %72 = icmp eq i64 %71, -2
  br i1 %72, label %73, label %91

73:                                               ; preds = %70
  %74 = load ptr, ptr %7, align 8
  %75 = getelementptr inbounds %struct.MatchState, ptr %74, i32 0, i32 3
  %76 = load ptr, ptr %75, align 8
  %77 = load ptr, ptr %7, align 8
  %78 = getelementptr inbounds %struct.MatchState, ptr %77, i32 0, i32 6
  %79 = load i32, ptr %8, align 4
  %80 = sext i32 %79 to i64
  %81 = getelementptr inbounds [32 x %struct.anon], ptr %78, i64 0, i64 %80
  %82 = getelementptr inbounds %struct.anon, ptr %81, i32 0, i32 0
  %83 = load ptr, ptr %82, align 8
  %84 = load ptr, ptr %7, align 8
  %85 = getelementptr inbounds %struct.MatchState, ptr %84, i32 0, i32 0
  %86 = load ptr, ptr %85, align 8
  %87 = ptrtoint ptr %83 to i64
  %88 = ptrtoint ptr %86 to i64
  %89 = sub i64 %87, %88
  %90 = add nsw i64 %89, 1
  call void @lua_pushinteger(ptr noundef %76, i64 noundef %90)
  br label %91

91:                                               ; preds = %73, %70
  br label %92

92:                                               ; preds = %91, %65
  %93 = load i64, ptr %12, align 8
  store i64 %93, ptr %6, align 8
  br label %94

94:                                               ; preds = %92, %34
  %95 = load i64, ptr %6, align 8
  ret i64 %95
}

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare ptr @luaL_prepbuffsize(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getformat(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = call i64 @strspn(ptr noundef %8, ptr noundef @.str.42) #7
  store i64 %9, ptr %7, align 8
  %10 = load i64, ptr %7, align 8
  %11 = add i64 %10, 1
  store i64 %11, ptr %7, align 8
  %12 = load i64, ptr %7, align 8
  %13 = icmp uge i64 %12, 22
  br i1 %13, label %14, label %17

14:                                               ; preds = %3
  %15 = load ptr, ptr %4, align 8
  %16 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %15, ptr noundef @.str.43)
  br label %17

17:                                               ; preds = %14, %3
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds i8, ptr %18, i32 1
  store ptr %19, ptr %6, align 8
  store i8 37, ptr %18, align 1
  %20 = load ptr, ptr %6, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = load i64, ptr %7, align 8
  %23 = mul i64 %22, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %20, ptr align 1 %21, i64 %23, i1 false)
  %24 = load ptr, ptr %6, align 8
  %25 = load i64, ptr %7, align 8
  %26 = getelementptr inbounds i8, ptr %24, i64 %25
  store i8 0, ptr %26, align 1
  %27 = load ptr, ptr %5, align 8
  %28 = load i64, ptr %7, align 8
  %29 = getelementptr inbounds i8, ptr %27, i64 %28
  %30 = getelementptr inbounds i8, ptr %29, i64 -1
  ret ptr %30
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @checkformat(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store i32 %3, ptr %8, align 4
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 1
  store ptr %11, ptr %9, align 8
  %12 = load ptr, ptr %9, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = call i64 @strspn(ptr noundef %12, ptr noundef %13) #7
  %15 = load ptr, ptr %9, align 8
  %16 = getelementptr inbounds i8, ptr %15, i64 %14
  store ptr %16, ptr %9, align 8
  %17 = load ptr, ptr %9, align 8
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp ne i32 %19, 48
  br i1 %20, label %21, label %37

21:                                               ; preds = %4
  %22 = load ptr, ptr %9, align 8
  %23 = call ptr @get2digits(ptr noundef %22)
  store ptr %23, ptr %9, align 8
  %24 = load ptr, ptr %9, align 8
  %25 = load i8, ptr %24, align 1
  %26 = sext i8 %25 to i32
  %27 = icmp eq i32 %26, 46
  br i1 %27, label %28, label %36

28:                                               ; preds = %21
  %29 = load i32, ptr %8, align 4
  %30 = icmp ne i32 %29, 0
  br i1 %30, label %31, label %36

31:                                               ; preds = %28
  %32 = load ptr, ptr %9, align 8
  %33 = getelementptr inbounds i8, ptr %32, i32 1
  store ptr %33, ptr %9, align 8
  %34 = load ptr, ptr %9, align 8
  %35 = call ptr @get2digits(ptr noundef %34)
  store ptr %35, ptr %9, align 8
  br label %36

36:                                               ; preds = %31, %28, %21
  br label %37

37:                                               ; preds = %36, %4
  %38 = call ptr @__ctype_b_loc() #8
  %39 = load ptr, ptr %38, align 8
  %40 = load ptr, ptr %9, align 8
  %41 = load i8, ptr %40, align 1
  %42 = zext i8 %41 to i32
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds i16, ptr %39, i64 %43
  %45 = load i16, ptr %44, align 2
  %46 = zext i16 %45 to i32
  %47 = and i32 %46, 1024
  %48 = icmp ne i32 %47, 0
  br i1 %48, label %53, label %49

49:                                               ; preds = %37
  %50 = load ptr, ptr %5, align 8
  %51 = load ptr, ptr %6, align 8
  %52 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %50, ptr noundef @.str.44, ptr noundef %51)
  br label %53

53:                                               ; preds = %49, %37
  ret void
}

; Function Attrs: nounwind
declare i32 @snprintf(ptr noundef, i64 noundef, ptr noundef, ...) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addlenmod(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca i8, align 1
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call i64 @strlen(ptr noundef %8) #7
  store i64 %9, ptr %5, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = call i64 @strlen(ptr noundef %10) #7
  store i64 %11, ptr %6, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = load i64, ptr %5, align 8
  %14 = sub i64 %13, 1
  %15 = getelementptr inbounds i8, ptr %12, i64 %14
  %16 = load i8, ptr %15, align 1
  store i8 %16, ptr %7, align 1
  %17 = load ptr, ptr %3, align 8
  %18 = load i64, ptr %5, align 8
  %19 = getelementptr inbounds i8, ptr %17, i64 %18
  %20 = getelementptr inbounds i8, ptr %19, i64 -1
  %21 = load ptr, ptr %4, align 8
  %22 = call ptr @strcpy(ptr noundef %20, ptr noundef %21) #6
  %23 = load i8, ptr %7, align 1
  %24 = load ptr, ptr %3, align 8
  %25 = load i64, ptr %5, align 8
  %26 = load i64, ptr %6, align 8
  %27 = add i64 %25, %26
  %28 = sub i64 %27, 1
  %29 = getelementptr inbounds i8, ptr %24, i64 %28
  store i8 %23, ptr %29, align 1
  %30 = load ptr, ptr %3, align 8
  %31 = load i64, ptr %5, align 8
  %32 = load i64, ptr %6, align 8
  %33 = add i64 %31, %32
  %34 = getelementptr inbounds i8, ptr %30, i64 %33
  store i8 0, ptr %34, align 1
  ret void
}

declare double @luaL_checknumber(ptr noundef, i32 noundef) #1

declare ptr @lua_topointer(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addliteral(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %6, align 4
  %15 = call i32 @lua_type(ptr noundef %13, i32 noundef %14)
  switch i32 %15, label %61 [
    i32 4, label %16
    i32 3, label %23
    i32 0, label %56
    i32 1, label %56
  ]

16:                                               ; preds = %3
  %17 = load ptr, ptr %4, align 8
  %18 = load i32, ptr %6, align 4
  %19 = call ptr @lua_tolstring(ptr noundef %17, i32 noundef %18, ptr noundef %7)
  store ptr %19, ptr %8, align 8
  %20 = load ptr, ptr %5, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = load i64, ptr %7, align 8
  call void @addquoted(ptr noundef %20, ptr noundef %21, i64 noundef %22)
  br label %65

23:                                               ; preds = %3
  %24 = load ptr, ptr %5, align 8
  %25 = call ptr @luaL_prepbuffsize(ptr noundef %24, i64 noundef 120)
  store ptr %25, ptr %9, align 8
  %26 = load ptr, ptr %4, align 8
  %27 = load i32, ptr %6, align 4
  %28 = call i32 @lua_isinteger(ptr noundef %26, i32 noundef %27)
  %29 = icmp ne i32 %28, 0
  br i1 %29, label %37, label %30

30:                                               ; preds = %23
  %31 = load ptr, ptr %4, align 8
  %32 = load ptr, ptr %9, align 8
  %33 = load ptr, ptr %4, align 8
  %34 = load i32, ptr %6, align 4
  %35 = call double @lua_tonumberx(ptr noundef %33, i32 noundef %34, ptr noundef null)
  %36 = call i32 @quotefloat(ptr noundef %31, ptr noundef %32, double noundef %35)
  store i32 %36, ptr %10, align 4
  br label %49

37:                                               ; preds = %23
  %38 = load ptr, ptr %4, align 8
  %39 = load i32, ptr %6, align 4
  %40 = call i64 @lua_tointegerx(ptr noundef %38, i32 noundef %39, ptr noundef null)
  store i64 %40, ptr %11, align 8
  %41 = load i64, ptr %11, align 8
  %42 = icmp eq i64 %41, -9223372036854775808
  %43 = zext i1 %42 to i64
  %44 = select i1 %42, ptr @.str.45, ptr @.str.46
  store ptr %44, ptr %12, align 8
  %45 = load ptr, ptr %9, align 8
  %46 = load ptr, ptr %12, align 8
  %47 = load i64, ptr %11, align 8
  %48 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %45, i64 noundef 120, ptr noundef %46, i64 noundef %47) #6
  store i32 %48, ptr %10, align 4
  br label %49

49:                                               ; preds = %37, %30
  %50 = load i32, ptr %10, align 4
  %51 = sext i32 %50 to i64
  %52 = load ptr, ptr %5, align 8
  %53 = getelementptr inbounds %struct.luaL_Buffer, ptr %52, i32 0, i32 2
  %54 = load i64, ptr %53, align 8
  %55 = add i64 %54, %51
  store i64 %55, ptr %53, align 8
  br label %65

56:                                               ; preds = %3, %3
  %57 = load ptr, ptr %4, align 8
  %58 = load i32, ptr %6, align 4
  %59 = call ptr @luaL_tolstring(ptr noundef %57, i32 noundef %58, ptr noundef null)
  %60 = load ptr, ptr %5, align 8
  call void @luaL_addvalue(ptr noundef %60)
  br label %65

61:                                               ; preds = %3
  %62 = load ptr, ptr %4, align 8
  %63 = load i32, ptr %6, align 4
  %64 = call i32 @luaL_argerror(ptr noundef %62, i32 noundef %63, ptr noundef @.str.47)
  br label %65

65:                                               ; preds = %61, %56, %49, %16
  ret void
}

declare ptr @luaL_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_addvalue(ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strspn(ptr noundef, ptr noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #5

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @get2digits(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = call ptr @__ctype_b_loc() #8
  %4 = load ptr, ptr %3, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = load i8, ptr %5, align 1
  %7 = zext i8 %6 to i32
  %8 = sext i32 %7 to i64
  %9 = getelementptr inbounds i16, ptr %4, i64 %8
  %10 = load i16, ptr %9, align 2
  %11 = zext i16 %10 to i32
  %12 = and i32 %11, 2048
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %32

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds i8, ptr %15, i32 1
  store ptr %16, ptr %2, align 8
  %17 = call ptr @__ctype_b_loc() #8
  %18 = load ptr, ptr %17, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i32
  %22 = sext i32 %21 to i64
  %23 = getelementptr inbounds i16, ptr %18, i64 %22
  %24 = load i16, ptr %23, align 2
  %25 = zext i16 %24 to i32
  %26 = and i32 %25, 2048
  %27 = icmp ne i32 %26, 0
  br i1 %27, label %28, label %31

28:                                               ; preds = %14
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds i8, ptr %29, i32 1
  store ptr %30, ptr %2, align 8
  br label %31

31:                                               ; preds = %28, %14
  br label %32

32:                                               ; preds = %31, %1
  %33 = load ptr, ptr %2, align 8
  ret ptr %33
}

; Function Attrs: nounwind
declare ptr @strcpy(ptr noundef, ptr noundef) #4

declare i32 @lua_type(ptr noundef, i32 noundef) #1

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addquoted(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca [10 x i8], align 1
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.luaL_Buffer, ptr %8, i32 0, i32 2
  %10 = load i64, ptr %9, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.luaL_Buffer, ptr %11, i32 0, i32 1
  %13 = load i64, ptr %12, align 8
  %14 = icmp ult i64 %10, %13
  br i1 %14, label %19, label %15

15:                                               ; preds = %3
  %16 = load ptr, ptr %4, align 8
  %17 = call ptr @luaL_prepbuffsize(ptr noundef %16, i64 noundef 1)
  %18 = icmp ne ptr %17, null
  br label %19

19:                                               ; preds = %15, %3
  %20 = phi i1 [ true, %3 ], [ %18, %15 ]
  %21 = zext i1 %20 to i32
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.luaL_Buffer, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.luaL_Buffer, ptr %25, i32 0, i32 2
  %27 = load i64, ptr %26, align 8
  %28 = add i64 %27, 1
  store i64 %28, ptr %26, align 8
  %29 = getelementptr inbounds i8, ptr %24, i64 %27
  store i8 34, ptr %29, align 1
  br label %30

30:                                               ; preds = %162, %19
  %31 = load i64, ptr %6, align 8
  %32 = add i64 %31, -1
  store i64 %32, ptr %6, align 8
  %33 = icmp ne i64 %31, 0
  br i1 %33, label %34, label %165

34:                                               ; preds = %30
  %35 = load ptr, ptr %5, align 8
  %36 = load i8, ptr %35, align 1
  %37 = sext i8 %36 to i32
  %38 = icmp eq i32 %37, 34
  br i1 %38, label %49, label %39

39:                                               ; preds = %34
  %40 = load ptr, ptr %5, align 8
  %41 = load i8, ptr %40, align 1
  %42 = sext i8 %41 to i32
  %43 = icmp eq i32 %42, 92
  br i1 %43, label %49, label %44

44:                                               ; preds = %39
  %45 = load ptr, ptr %5, align 8
  %46 = load i8, ptr %45, align 1
  %47 = sext i8 %46 to i32
  %48 = icmp eq i32 %47, 10
  br i1 %48, label %49, label %96

49:                                               ; preds = %44, %39, %34
  %50 = load ptr, ptr %4, align 8
  %51 = getelementptr inbounds %struct.luaL_Buffer, ptr %50, i32 0, i32 2
  %52 = load i64, ptr %51, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.luaL_Buffer, ptr %53, i32 0, i32 1
  %55 = load i64, ptr %54, align 8
  %56 = icmp ult i64 %52, %55
  br i1 %56, label %61, label %57

57:                                               ; preds = %49
  %58 = load ptr, ptr %4, align 8
  %59 = call ptr @luaL_prepbuffsize(ptr noundef %58, i64 noundef 1)
  %60 = icmp ne ptr %59, null
  br label %61

61:                                               ; preds = %57, %49
  %62 = phi i1 [ true, %49 ], [ %60, %57 ]
  %63 = zext i1 %62 to i32
  %64 = load ptr, ptr %4, align 8
  %65 = getelementptr inbounds %struct.luaL_Buffer, ptr %64, i32 0, i32 0
  %66 = load ptr, ptr %65, align 8
  %67 = load ptr, ptr %4, align 8
  %68 = getelementptr inbounds %struct.luaL_Buffer, ptr %67, i32 0, i32 2
  %69 = load i64, ptr %68, align 8
  %70 = add i64 %69, 1
  store i64 %70, ptr %68, align 8
  %71 = getelementptr inbounds i8, ptr %66, i64 %69
  store i8 92, ptr %71, align 1
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.luaL_Buffer, ptr %72, i32 0, i32 2
  %74 = load i64, ptr %73, align 8
  %75 = load ptr, ptr %4, align 8
  %76 = getelementptr inbounds %struct.luaL_Buffer, ptr %75, i32 0, i32 1
  %77 = load i64, ptr %76, align 8
  %78 = icmp ult i64 %74, %77
  br i1 %78, label %83, label %79

79:                                               ; preds = %61
  %80 = load ptr, ptr %4, align 8
  %81 = call ptr @luaL_prepbuffsize(ptr noundef %80, i64 noundef 1)
  %82 = icmp ne ptr %81, null
  br label %83

83:                                               ; preds = %79, %61
  %84 = phi i1 [ true, %61 ], [ %82, %79 ]
  %85 = zext i1 %84 to i32
  %86 = load ptr, ptr %5, align 8
  %87 = load i8, ptr %86, align 1
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds %struct.luaL_Buffer, ptr %88, i32 0, i32 0
  %90 = load ptr, ptr %89, align 8
  %91 = load ptr, ptr %4, align 8
  %92 = getelementptr inbounds %struct.luaL_Buffer, ptr %91, i32 0, i32 2
  %93 = load i64, ptr %92, align 8
  %94 = add i64 %93, 1
  store i64 %94, ptr %92, align 8
  %95 = getelementptr inbounds i8, ptr %90, i64 %93
  store i8 %87, ptr %95, align 1
  br label %162

96:                                               ; preds = %44
  %97 = call ptr @__ctype_b_loc() #8
  %98 = load ptr, ptr %97, align 8
  %99 = load ptr, ptr %5, align 8
  %100 = load i8, ptr %99, align 1
  %101 = zext i8 %100 to i32
  %102 = sext i32 %101 to i64
  %103 = getelementptr inbounds i16, ptr %98, i64 %102
  %104 = load i16, ptr %103, align 2
  %105 = zext i16 %104 to i32
  %106 = and i32 %105, 2
  %107 = icmp ne i32 %106, 0
  br i1 %107, label %108, label %136

108:                                              ; preds = %96
  %109 = call ptr @__ctype_b_loc() #8
  %110 = load ptr, ptr %109, align 8
  %111 = load ptr, ptr %5, align 8
  %112 = getelementptr inbounds i8, ptr %111, i64 1
  %113 = load i8, ptr %112, align 1
  %114 = zext i8 %113 to i32
  %115 = sext i32 %114 to i64
  %116 = getelementptr inbounds i16, ptr %110, i64 %115
  %117 = load i16, ptr %116, align 2
  %118 = zext i16 %117 to i32
  %119 = and i32 %118, 2048
  %120 = icmp ne i32 %119, 0
  br i1 %120, label %127, label %121

121:                                              ; preds = %108
  %122 = getelementptr inbounds [10 x i8], ptr %7, i64 0, i64 0
  %123 = load ptr, ptr %5, align 8
  %124 = load i8, ptr %123, align 1
  %125 = zext i8 %124 to i32
  %126 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %122, i64 noundef 10, ptr noundef @.str.48, i32 noundef %125) #6
  br label %133

127:                                              ; preds = %108
  %128 = getelementptr inbounds [10 x i8], ptr %7, i64 0, i64 0
  %129 = load ptr, ptr %5, align 8
  %130 = load i8, ptr %129, align 1
  %131 = zext i8 %130 to i32
  %132 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %128, i64 noundef 10, ptr noundef @.str.49, i32 noundef %131) #6
  br label %133

133:                                              ; preds = %127, %121
  %134 = load ptr, ptr %4, align 8
  %135 = getelementptr inbounds [10 x i8], ptr %7, i64 0, i64 0
  call void @luaL_addstring(ptr noundef %134, ptr noundef %135)
  br label %161

136:                                              ; preds = %96
  %137 = load ptr, ptr %4, align 8
  %138 = getelementptr inbounds %struct.luaL_Buffer, ptr %137, i32 0, i32 2
  %139 = load i64, ptr %138, align 8
  %140 = load ptr, ptr %4, align 8
  %141 = getelementptr inbounds %struct.luaL_Buffer, ptr %140, i32 0, i32 1
  %142 = load i64, ptr %141, align 8
  %143 = icmp ult i64 %139, %142
  br i1 %143, label %148, label %144

144:                                              ; preds = %136
  %145 = load ptr, ptr %4, align 8
  %146 = call ptr @luaL_prepbuffsize(ptr noundef %145, i64 noundef 1)
  %147 = icmp ne ptr %146, null
  br label %148

148:                                              ; preds = %144, %136
  %149 = phi i1 [ true, %136 ], [ %147, %144 ]
  %150 = zext i1 %149 to i32
  %151 = load ptr, ptr %5, align 8
  %152 = load i8, ptr %151, align 1
  %153 = load ptr, ptr %4, align 8
  %154 = getelementptr inbounds %struct.luaL_Buffer, ptr %153, i32 0, i32 0
  %155 = load ptr, ptr %154, align 8
  %156 = load ptr, ptr %4, align 8
  %157 = getelementptr inbounds %struct.luaL_Buffer, ptr %156, i32 0, i32 2
  %158 = load i64, ptr %157, align 8
  %159 = add i64 %158, 1
  store i64 %159, ptr %157, align 8
  %160 = getelementptr inbounds i8, ptr %155, i64 %158
  store i8 %152, ptr %160, align 1
  br label %161

161:                                              ; preds = %148, %133
  br label %162

162:                                              ; preds = %161, %83
  %163 = load ptr, ptr %5, align 8
  %164 = getelementptr inbounds i8, ptr %163, i32 1
  store ptr %164, ptr %5, align 8
  br label %30, !llvm.loop !30

165:                                              ; preds = %30
  %166 = load ptr, ptr %4, align 8
  %167 = getelementptr inbounds %struct.luaL_Buffer, ptr %166, i32 0, i32 2
  %168 = load i64, ptr %167, align 8
  %169 = load ptr, ptr %4, align 8
  %170 = getelementptr inbounds %struct.luaL_Buffer, ptr %169, i32 0, i32 1
  %171 = load i64, ptr %170, align 8
  %172 = icmp ult i64 %168, %171
  br i1 %172, label %177, label %173

173:                                              ; preds = %165
  %174 = load ptr, ptr %4, align 8
  %175 = call ptr @luaL_prepbuffsize(ptr noundef %174, i64 noundef 1)
  %176 = icmp ne ptr %175, null
  br label %177

177:                                              ; preds = %173, %165
  %178 = phi i1 [ true, %165 ], [ %176, %173 ]
  %179 = zext i1 %178 to i32
  %180 = load ptr, ptr %4, align 8
  %181 = getelementptr inbounds %struct.luaL_Buffer, ptr %180, i32 0, i32 0
  %182 = load ptr, ptr %181, align 8
  %183 = load ptr, ptr %4, align 8
  %184 = getelementptr inbounds %struct.luaL_Buffer, ptr %183, i32 0, i32 2
  %185 = load i64, ptr %184, align 8
  %186 = add i64 %185, 1
  store i64 %186, ptr %184, align 8
  %187 = getelementptr inbounds i8, ptr %182, i64 %185
  store i8 34, ptr %187, align 1
  ret void
}

declare i32 @lua_isinteger(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @quotefloat(ptr noundef %0, ptr noundef %1, double noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca double, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i8, align 1
  %11 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store double %2, ptr %7, align 8
  %12 = load double, ptr %7, align 8
  %13 = fcmp oeq double %12, 0x7FF0000000000000
  br i1 %13, label %14, label %15

14:                                               ; preds = %3
  store ptr @.str.50, ptr %8, align 8
  br label %55

15:                                               ; preds = %3
  %16 = load double, ptr %7, align 8
  %17 = fcmp oeq double %16, 0xFFF0000000000000
  br i1 %17, label %18, label %19

18:                                               ; preds = %15
  store ptr @.str.51, ptr %8, align 8
  br label %54

19:                                               ; preds = %15
  %20 = load double, ptr %7, align 8
  %21 = load double, ptr %7, align 8
  %22 = fcmp une double %20, %21
  br i1 %22, label %23, label %24

23:                                               ; preds = %19
  store ptr @.str.52, ptr %8, align 8
  br label %53

24:                                               ; preds = %19
  %25 = load ptr, ptr %5, align 8
  %26 = load ptr, ptr %6, align 8
  %27 = load double, ptr %7, align 8
  %28 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %26, i64 noundef 120, ptr noundef @.str.53, double noundef %27) #6
  store i32 %28, ptr %9, align 4
  %29 = load ptr, ptr %6, align 8
  %30 = load i32, ptr %9, align 4
  %31 = sext i32 %30 to i64
  %32 = call ptr @memchr(ptr noundef %29, i32 noundef 46, i64 noundef %31) #7
  %33 = icmp eq ptr %32, null
  br i1 %33, label %34, label %51

34:                                               ; preds = %24
  %35 = call ptr @localeconv() #6
  %36 = getelementptr inbounds %struct.lconv, ptr %35, i32 0, i32 0
  %37 = load ptr, ptr %36, align 8
  %38 = getelementptr inbounds i8, ptr %37, i64 0
  %39 = load i8, ptr %38, align 1
  store i8 %39, ptr %10, align 1
  %40 = load ptr, ptr %6, align 8
  %41 = load i8, ptr %10, align 1
  %42 = sext i8 %41 to i32
  %43 = load i32, ptr %9, align 4
  %44 = sext i32 %43 to i64
  %45 = call ptr @memchr(ptr noundef %40, i32 noundef %42, i64 noundef %44) #7
  store ptr %45, ptr %11, align 8
  %46 = load ptr, ptr %11, align 8
  %47 = icmp ne ptr %46, null
  br i1 %47, label %48, label %50

48:                                               ; preds = %34
  %49 = load ptr, ptr %11, align 8
  store i8 46, ptr %49, align 1
  br label %50

50:                                               ; preds = %48, %34
  br label %51

51:                                               ; preds = %50, %24
  %52 = load i32, ptr %9, align 4
  store i32 %52, ptr %4, align 4
  br label %59

53:                                               ; preds = %23
  br label %54

54:                                               ; preds = %53, %18
  br label %55

55:                                               ; preds = %54, %14
  %56 = load ptr, ptr %6, align 8
  %57 = load ptr, ptr %8, align 8
  %58 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %56, i64 noundef 120, ptr noundef @.str.54, ptr noundef %57) #6
  store i32 %58, ptr %4, align 4
  br label %59

59:                                               ; preds = %55, %51
  %60 = load i32, ptr %4, align 4
  ret i32 %60
}

declare double @lua_tonumberx(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

declare void @luaL_addstring(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare ptr @localeconv() #4

declare ptr @lua_newuserdatauv(ptr noundef, i64 noundef, i32 noundef) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @gmatch_aux(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call ptr @lua_touserdata(ptr noundef %7, i32 noundef -1001003)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.GMatchState, ptr %10, i32 0, i32 3
  %12 = getelementptr inbounds %struct.MatchState, ptr %11, i32 0, i32 3
  store ptr %9, ptr %12, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.GMatchState, ptr %13, i32 0, i32 0
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %5, align 8
  br label %16

16:                                               ; preds = %52, %1
  %17 = load ptr, ptr %5, align 8
  %18 = load ptr, ptr %4, align 8
  %19 = getelementptr inbounds %struct.GMatchState, ptr %18, i32 0, i32 3
  %20 = getelementptr inbounds %struct.MatchState, ptr %19, i32 0, i32 1
  %21 = load ptr, ptr %20, align 8
  %22 = icmp ule ptr %17, %21
  br i1 %22, label %23, label %55

23:                                               ; preds = %16
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.GMatchState, ptr %24, i32 0, i32 3
  call void @reprepstate(ptr noundef %25)
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds %struct.GMatchState, ptr %26, i32 0, i32 3
  %28 = load ptr, ptr %5, align 8
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.GMatchState, ptr %29, i32 0, i32 1
  %31 = load ptr, ptr %30, align 8
  %32 = call ptr @match(ptr noundef %27, ptr noundef %28, ptr noundef %31)
  store ptr %32, ptr %6, align 8
  %33 = icmp ne ptr %32, null
  br i1 %33, label %34, label %51

34:                                               ; preds = %23
  %35 = load ptr, ptr %6, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.GMatchState, ptr %36, i32 0, i32 2
  %38 = load ptr, ptr %37, align 8
  %39 = icmp ne ptr %35, %38
  br i1 %39, label %40, label %51

40:                                               ; preds = %34
  %41 = load ptr, ptr %6, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.GMatchState, ptr %42, i32 0, i32 2
  store ptr %41, ptr %43, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.GMatchState, ptr %44, i32 0, i32 0
  store ptr %41, ptr %45, align 8
  %46 = load ptr, ptr %4, align 8
  %47 = getelementptr inbounds %struct.GMatchState, ptr %46, i32 0, i32 3
  %48 = load ptr, ptr %5, align 8
  %49 = load ptr, ptr %6, align 8
  %50 = call i32 @push_captures(ptr noundef %47, ptr noundef %48, ptr noundef %49)
  store i32 %50, ptr %2, align 4
  br label %56

51:                                               ; preds = %34, %23
  br label %52

52:                                               ; preds = %51
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds i8, ptr %53, i32 1
  store ptr %54, ptr %5, align 8
  br label %16, !llvm.loop !31

55:                                               ; preds = %16
  store i32 0, ptr %2, align 4
  br label %56

56:                                               ; preds = %55, %40
  %57 = load i32, ptr %2, align 4
  ret i32 %57
}

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

declare i32 @luaL_typeerror(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @add_value(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #0 {
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  store ptr %0, ptr %7, align 8
  store ptr %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i32 %4, ptr %11, align 4
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.MatchState, ptr %14, i32 0, i32 3
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %12, align 8
  %17 = load i32, ptr %11, align 4
  switch i32 %17, label %32 [
    i32 6, label %18
    i32 5, label %26
  ]

18:                                               ; preds = %5
  %19 = load ptr, ptr %12, align 8
  call void @lua_pushvalue(ptr noundef %19, i32 noundef 3)
  %20 = load ptr, ptr %7, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = load ptr, ptr %10, align 8
  %23 = call i32 @push_captures(ptr noundef %20, ptr noundef %21, ptr noundef %22)
  store i32 %23, ptr %13, align 4
  %24 = load ptr, ptr %12, align 8
  %25 = load i32, ptr %13, align 4
  call void @lua_callk(ptr noundef %24, i32 noundef %25, i32 noundef 1, i64 noundef 0, ptr noundef null)
  br label %37

26:                                               ; preds = %5
  %27 = load ptr, ptr %7, align 8
  %28 = load ptr, ptr %9, align 8
  %29 = load ptr, ptr %10, align 8
  call void @push_onecapture(ptr noundef %27, i32 noundef 0, ptr noundef %28, ptr noundef %29)
  %30 = load ptr, ptr %12, align 8
  %31 = call i32 @lua_gettable(ptr noundef %30, i32 noundef 3)
  br label %37

32:                                               ; preds = %5
  %33 = load ptr, ptr %7, align 8
  %34 = load ptr, ptr %8, align 8
  %35 = load ptr, ptr %9, align 8
  %36 = load ptr, ptr %10, align 8
  call void @add_s(ptr noundef %33, ptr noundef %34, ptr noundef %35, ptr noundef %36)
  store i32 1, ptr %6, align 4
  br label %69

37:                                               ; preds = %26, %18
  %38 = load ptr, ptr %12, align 8
  %39 = call i32 @lua_toboolean(ptr noundef %38, i32 noundef -1)
  %40 = icmp ne i32 %39, 0
  br i1 %40, label %50, label %41

41:                                               ; preds = %37
  %42 = load ptr, ptr %12, align 8
  call void @lua_settop(ptr noundef %42, i32 noundef -2)
  %43 = load ptr, ptr %8, align 8
  %44 = load ptr, ptr %9, align 8
  %45 = load ptr, ptr %10, align 8
  %46 = load ptr, ptr %9, align 8
  %47 = ptrtoint ptr %45 to i64
  %48 = ptrtoint ptr %46 to i64
  %49 = sub i64 %47, %48
  call void @luaL_addlstring(ptr noundef %43, ptr noundef %44, i64 noundef %49)
  store i32 0, ptr %6, align 4
  br label %69

50:                                               ; preds = %37
  %51 = load ptr, ptr %12, align 8
  %52 = call i32 @lua_isstring(ptr noundef %51, i32 noundef -1)
  %53 = icmp ne i32 %52, 0
  %54 = xor i1 %53, true
  %55 = zext i1 %54 to i32
  %56 = icmp ne i32 %55, 0
  %57 = zext i1 %56 to i32
  %58 = sext i32 %57 to i64
  %59 = icmp ne i64 %58, 0
  br i1 %59, label %60, label %67

60:                                               ; preds = %50
  %61 = load ptr, ptr %12, align 8
  %62 = load ptr, ptr %12, align 8
  %63 = load ptr, ptr %12, align 8
  %64 = call i32 @lua_type(ptr noundef %63, i32 noundef -1)
  %65 = call ptr @lua_typename(ptr noundef %62, i32 noundef %64)
  %66 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %61, ptr noundef @.str.56, ptr noundef %65)
  store i32 %66, ptr %6, align 4
  br label %69

67:                                               ; preds = %50
  %68 = load ptr, ptr %8, align 8
  call void @luaL_addvalue(ptr noundef %68)
  store i32 1, ptr %6, align 4
  br label %69

69:                                               ; preds = %67, %60, %41, %32
  %70 = load i32, ptr %6, align 4
  ret i32 %70
}

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

declare void @lua_callk(ptr noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare i32 @lua_gettable(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @add_s(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds %struct.MatchState, ptr %15, i32 0, i32 3
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %10, align 8
  %18 = load ptr, ptr %10, align 8
  %19 = call ptr @lua_tolstring(ptr noundef %18, i32 noundef 3, ptr noundef %9)
  store ptr %19, ptr %11, align 8
  br label %20

20:                                               ; preds = %112, %4
  %21 = load ptr, ptr %11, align 8
  %22 = load i64, ptr %9, align 8
  %23 = call ptr @memchr(ptr noundef %21, i32 noundef 37, i64 noundef %22) #7
  store ptr %23, ptr %12, align 8
  %24 = icmp ne ptr %23, null
  br i1 %24, label %25, label %123

25:                                               ; preds = %20
  %26 = load ptr, ptr %6, align 8
  %27 = load ptr, ptr %11, align 8
  %28 = load ptr, ptr %12, align 8
  %29 = load ptr, ptr %11, align 8
  %30 = ptrtoint ptr %28 to i64
  %31 = ptrtoint ptr %29 to i64
  %32 = sub i64 %30, %31
  call void @luaL_addlstring(ptr noundef %26, ptr noundef %27, i64 noundef %32)
  %33 = load ptr, ptr %12, align 8
  %34 = getelementptr inbounds i8, ptr %33, i32 1
  store ptr %34, ptr %12, align 8
  %35 = load ptr, ptr %12, align 8
  %36 = load i8, ptr %35, align 1
  %37 = sext i8 %36 to i32
  %38 = icmp eq i32 %37, 37
  br i1 %38, label %39, label %64

39:                                               ; preds = %25
  %40 = load ptr, ptr %6, align 8
  %41 = getelementptr inbounds %struct.luaL_Buffer, ptr %40, i32 0, i32 2
  %42 = load i64, ptr %41, align 8
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.luaL_Buffer, ptr %43, i32 0, i32 1
  %45 = load i64, ptr %44, align 8
  %46 = icmp ult i64 %42, %45
  br i1 %46, label %51, label %47

47:                                               ; preds = %39
  %48 = load ptr, ptr %6, align 8
  %49 = call ptr @luaL_prepbuffsize(ptr noundef %48, i64 noundef 1)
  %50 = icmp ne ptr %49, null
  br label %51

51:                                               ; preds = %47, %39
  %52 = phi i1 [ true, %39 ], [ %50, %47 ]
  %53 = zext i1 %52 to i32
  %54 = load ptr, ptr %12, align 8
  %55 = load i8, ptr %54, align 1
  %56 = load ptr, ptr %6, align 8
  %57 = getelementptr inbounds %struct.luaL_Buffer, ptr %56, i32 0, i32 0
  %58 = load ptr, ptr %57, align 8
  %59 = load ptr, ptr %6, align 8
  %60 = getelementptr inbounds %struct.luaL_Buffer, ptr %59, i32 0, i32 2
  %61 = load i64, ptr %60, align 8
  %62 = add i64 %61, 1
  store i64 %62, ptr %60, align 8
  %63 = getelementptr inbounds i8, ptr %58, i64 %61
  store i8 %55, ptr %63, align 1
  br label %112

64:                                               ; preds = %25
  %65 = load ptr, ptr %12, align 8
  %66 = load i8, ptr %65, align 1
  %67 = sext i8 %66 to i32
  %68 = icmp eq i32 %67, 48
  br i1 %68, label %69, label %77

69:                                               ; preds = %64
  %70 = load ptr, ptr %6, align 8
  %71 = load ptr, ptr %7, align 8
  %72 = load ptr, ptr %8, align 8
  %73 = load ptr, ptr %7, align 8
  %74 = ptrtoint ptr %72 to i64
  %75 = ptrtoint ptr %73 to i64
  %76 = sub i64 %74, %75
  call void @luaL_addlstring(ptr noundef %70, ptr noundef %71, i64 noundef %76)
  br label %111

77:                                               ; preds = %64
  %78 = call ptr @__ctype_b_loc() #8
  %79 = load ptr, ptr %78, align 8
  %80 = load ptr, ptr %12, align 8
  %81 = load i8, ptr %80, align 1
  %82 = zext i8 %81 to i32
  %83 = sext i32 %82 to i64
  %84 = getelementptr inbounds i16, ptr %79, i64 %83
  %85 = load i16, ptr %84, align 2
  %86 = zext i16 %85 to i32
  %87 = and i32 %86, 2048
  %88 = icmp ne i32 %87, 0
  br i1 %88, label %89, label %107

89:                                               ; preds = %77
  %90 = load ptr, ptr %5, align 8
  %91 = load ptr, ptr %12, align 8
  %92 = load i8, ptr %91, align 1
  %93 = sext i8 %92 to i32
  %94 = sub nsw i32 %93, 49
  %95 = load ptr, ptr %7, align 8
  %96 = load ptr, ptr %8, align 8
  %97 = call i64 @get_onecapture(ptr noundef %90, i32 noundef %94, ptr noundef %95, ptr noundef %96, ptr noundef %13)
  store i64 %97, ptr %14, align 8
  %98 = load i64, ptr %14, align 8
  %99 = icmp eq i64 %98, -2
  br i1 %99, label %100, label %102

100:                                              ; preds = %89
  %101 = load ptr, ptr %6, align 8
  call void @luaL_addvalue(ptr noundef %101)
  br label %106

102:                                              ; preds = %89
  %103 = load ptr, ptr %6, align 8
  %104 = load ptr, ptr %13, align 8
  %105 = load i64, ptr %14, align 8
  call void @luaL_addlstring(ptr noundef %103, ptr noundef %104, i64 noundef %105)
  br label %106

106:                                              ; preds = %102, %100
  br label %110

107:                                              ; preds = %77
  %108 = load ptr, ptr %10, align 8
  %109 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %108, ptr noundef @.str.57, i32 noundef 37)
  br label %110

110:                                              ; preds = %107, %106
  br label %111

111:                                              ; preds = %110, %69
  br label %112

112:                                              ; preds = %111, %51
  %113 = load ptr, ptr %12, align 8
  %114 = getelementptr inbounds i8, ptr %113, i64 1
  %115 = load ptr, ptr %11, align 8
  %116 = ptrtoint ptr %114 to i64
  %117 = ptrtoint ptr %115 to i64
  %118 = sub i64 %116, %117
  %119 = load i64, ptr %9, align 8
  %120 = sub i64 %119, %118
  store i64 %120, ptr %9, align 8
  %121 = load ptr, ptr %12, align 8
  %122 = getelementptr inbounds i8, ptr %121, i64 1
  store ptr %122, ptr %11, align 8
  br label %20, !llvm.loop !32

123:                                              ; preds = %20
  %124 = load ptr, ptr %6, align 8
  %125 = load ptr, ptr %11, align 8
  %126 = load i64, ptr %9, align 8
  call void @luaL_addlstring(ptr noundef %124, ptr noundef %125, i64 noundef %126)
  ret void
}

declare i32 @lua_isstring(ptr noundef, i32 noundef) #1

declare ptr @lua_typename(ptr noundef, i32 noundef) #1

declare ptr @luaL_optlstring(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #1

declare ptr @lua_pushstring(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @toupper(i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @initheader(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.Header, ptr %6, i32 0, i32 0
  store ptr %5, ptr %7, align 8
  %8 = load i8, ptr @nativeendian, align 4
  %9 = sext i8 %8 to i32
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.Header, ptr %10, i32 0, i32 1
  store i32 %9, ptr %11, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.Header, ptr %12, i32 0, i32 2
  store i32 1, ptr %13, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getdetails(ptr noundef %0, i64 noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i64 %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = load ptr, ptr %8, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = call i32 @getoption(ptr noundef %13, ptr noundef %14, ptr noundef %15)
  store i32 %16, ptr %11, align 4
  %17 = load ptr, ptr %9, align 8
  %18 = load i32, ptr %17, align 4
  store i32 %18, ptr %12, align 4
  %19 = load i32, ptr %11, align 4
  %20 = icmp eq i32 %19, 9
  br i1 %20, label %21, label %41

21:                                               ; preds = %5
  %22 = load ptr, ptr %8, align 8
  %23 = load ptr, ptr %22, align 8
  %24 = load i8, ptr %23, align 1
  %25 = sext i8 %24 to i32
  %26 = icmp eq i32 %25, 0
  br i1 %26, label %35, label %27

27:                                               ; preds = %21
  %28 = load ptr, ptr %6, align 8
  %29 = load ptr, ptr %8, align 8
  %30 = call i32 @getoption(ptr noundef %28, ptr noundef %29, ptr noundef %12)
  %31 = icmp eq i32 %30, 5
  br i1 %31, label %35, label %32

32:                                               ; preds = %27
  %33 = load i32, ptr %12, align 4
  %34 = icmp eq i32 %33, 0
  br i1 %34, label %35, label %40

35:                                               ; preds = %32, %27, %21
  %36 = load ptr, ptr %6, align 8
  %37 = getelementptr inbounds %struct.Header, ptr %36, i32 0, i32 0
  %38 = load ptr, ptr %37, align 8
  %39 = call i32 @luaL_argerror(ptr noundef %38, i32 noundef 1, ptr noundef @.str.63)
  br label %40

40:                                               ; preds = %35, %32
  br label %41

41:                                               ; preds = %40, %5
  %42 = load i32, ptr %12, align 4
  %43 = icmp sle i32 %42, 1
  br i1 %43, label %47, label %44

44:                                               ; preds = %41
  %45 = load i32, ptr %11, align 4
  %46 = icmp eq i32 %45, 5
  br i1 %46, label %47, label %49

47:                                               ; preds = %44, %41
  %48 = load ptr, ptr %10, align 8
  store i32 0, ptr %48, align 4
  br label %88

49:                                               ; preds = %44
  %50 = load i32, ptr %12, align 4
  %51 = load ptr, ptr %6, align 8
  %52 = getelementptr inbounds %struct.Header, ptr %51, i32 0, i32 2
  %53 = load i32, ptr %52, align 4
  %54 = icmp sgt i32 %50, %53
  br i1 %54, label %55, label %59

55:                                               ; preds = %49
  %56 = load ptr, ptr %6, align 8
  %57 = getelementptr inbounds %struct.Header, ptr %56, i32 0, i32 2
  %58 = load i32, ptr %57, align 4
  store i32 %58, ptr %12, align 4
  br label %59

59:                                               ; preds = %55, %49
  %60 = load i32, ptr %12, align 4
  %61 = load i32, ptr %12, align 4
  %62 = sub nsw i32 %61, 1
  %63 = and i32 %60, %62
  %64 = icmp ne i32 %63, 0
  %65 = zext i1 %64 to i32
  %66 = icmp ne i32 %65, 0
  %67 = zext i1 %66 to i32
  %68 = sext i32 %67 to i64
  %69 = icmp ne i64 %68, 0
  br i1 %69, label %70, label %75

70:                                               ; preds = %59
  %71 = load ptr, ptr %6, align 8
  %72 = getelementptr inbounds %struct.Header, ptr %71, i32 0, i32 0
  %73 = load ptr, ptr %72, align 8
  %74 = call i32 @luaL_argerror(ptr noundef %73, i32 noundef 1, ptr noundef @.str.64)
  br label %75

75:                                               ; preds = %70, %59
  %76 = load i32, ptr %12, align 4
  %77 = load i64, ptr %7, align 8
  %78 = load i32, ptr %12, align 4
  %79 = sub nsw i32 %78, 1
  %80 = sext i32 %79 to i64
  %81 = and i64 %77, %80
  %82 = trunc i64 %81 to i32
  %83 = sub nsw i32 %76, %82
  %84 = load i32, ptr %12, align 4
  %85 = sub nsw i32 %84, 1
  %86 = and i32 %83, %85
  %87 = load ptr, ptr %10, align 8
  store i32 %86, ptr %87, align 4
  br label %88

88:                                               ; preds = %75, %47
  %89 = load i32, ptr %11, align 4
  ret i32 %89
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @packint(ptr noundef %0, i64 noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store i64 %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %13 = load ptr, ptr %6, align 8
  %14 = load i32, ptr %9, align 4
  %15 = sext i32 %14 to i64
  %16 = call ptr @luaL_prepbuffsize(ptr noundef %13, i64 noundef %15)
  store ptr %16, ptr %11, align 8
  %17 = load i64, ptr %7, align 8
  %18 = and i64 %17, 255
  %19 = trunc i64 %18 to i8
  %20 = load ptr, ptr %11, align 8
  %21 = load i32, ptr %8, align 4
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %5
  br label %27

24:                                               ; preds = %5
  %25 = load i32, ptr %9, align 4
  %26 = sub nsw i32 %25, 1
  br label %27

27:                                               ; preds = %24, %23
  %28 = phi i32 [ 0, %23 ], [ %26, %24 ]
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds i8, ptr %20, i64 %29
  store i8 %19, ptr %30, align 1
  store i32 1, ptr %12, align 4
  br label %31

31:                                               ; preds = %55, %27
  %32 = load i32, ptr %12, align 4
  %33 = load i32, ptr %9, align 4
  %34 = icmp slt i32 %32, %33
  br i1 %34, label %35, label %58

35:                                               ; preds = %31
  %36 = load i64, ptr %7, align 8
  %37 = lshr i64 %36, 8
  store i64 %37, ptr %7, align 8
  %38 = load i64, ptr %7, align 8
  %39 = and i64 %38, 255
  %40 = trunc i64 %39 to i8
  %41 = load ptr, ptr %11, align 8
  %42 = load i32, ptr %8, align 4
  %43 = icmp ne i32 %42, 0
  br i1 %43, label %44, label %46

44:                                               ; preds = %35
  %45 = load i32, ptr %12, align 4
  br label %51

46:                                               ; preds = %35
  %47 = load i32, ptr %9, align 4
  %48 = sub nsw i32 %47, 1
  %49 = load i32, ptr %12, align 4
  %50 = sub nsw i32 %48, %49
  br label %51

51:                                               ; preds = %46, %44
  %52 = phi i32 [ %45, %44 ], [ %50, %46 ]
  %53 = sext i32 %52 to i64
  %54 = getelementptr inbounds i8, ptr %41, i64 %53
  store i8 %40, ptr %54, align 1
  br label %55

55:                                               ; preds = %51
  %56 = load i32, ptr %12, align 4
  %57 = add nsw i32 %56, 1
  store i32 %57, ptr %12, align 4
  br label %31, !llvm.loop !33

58:                                               ; preds = %31
  %59 = load i32, ptr %10, align 4
  %60 = icmp ne i32 %59, 0
  br i1 %60, label %61, label %88

61:                                               ; preds = %58
  %62 = load i32, ptr %9, align 4
  %63 = icmp sgt i32 %62, 8
  br i1 %63, label %64, label %88

64:                                               ; preds = %61
  store i32 8, ptr %12, align 4
  br label %65

65:                                               ; preds = %84, %64
  %66 = load i32, ptr %12, align 4
  %67 = load i32, ptr %9, align 4
  %68 = icmp slt i32 %66, %67
  br i1 %68, label %69, label %87

69:                                               ; preds = %65
  %70 = load ptr, ptr %11, align 8
  %71 = load i32, ptr %8, align 4
  %72 = icmp ne i32 %71, 0
  br i1 %72, label %73, label %75

73:                                               ; preds = %69
  %74 = load i32, ptr %12, align 4
  br label %80

75:                                               ; preds = %69
  %76 = load i32, ptr %9, align 4
  %77 = sub nsw i32 %76, 1
  %78 = load i32, ptr %12, align 4
  %79 = sub nsw i32 %77, %78
  br label %80

80:                                               ; preds = %75, %73
  %81 = phi i32 [ %74, %73 ], [ %79, %75 ]
  %82 = sext i32 %81 to i64
  %83 = getelementptr inbounds i8, ptr %70, i64 %82
  store i8 -1, ptr %83, align 1
  br label %84

84:                                               ; preds = %80
  %85 = load i32, ptr %12, align 4
  %86 = add nsw i32 %85, 1
  store i32 %86, ptr %12, align 4
  br label %65, !llvm.loop !34

87:                                               ; preds = %65
  br label %88

88:                                               ; preds = %87, %61, %58
  %89 = load i32, ptr %9, align 4
  %90 = sext i32 %89 to i64
  %91 = load ptr, ptr %6, align 8
  %92 = getelementptr inbounds %struct.luaL_Buffer, ptr %91, i32 0, i32 2
  %93 = load i64, ptr %92, align 8
  %94 = add i64 %93, %90
  store i64 %94, ptr %92, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @copywithendian(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  store i32 %3, ptr %8, align 4
  %9 = load i32, ptr %8, align 4
  %10 = load i8, ptr @nativeendian, align 4
  %11 = sext i8 %10 to i32
  %12 = icmp eq i32 %9, %11
  br i1 %12, label %13, label %18

13:                                               ; preds = %4
  %14 = load ptr, ptr %5, align 8
  %15 = load ptr, ptr %6, align 8
  %16 = load i32, ptr %7, align 4
  %17 = sext i32 %16 to i64
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %14, ptr align 1 %15, i64 %17, i1 false)
  br label %35

18:                                               ; preds = %4
  %19 = load i32, ptr %7, align 4
  %20 = sub nsw i32 %19, 1
  %21 = load ptr, ptr %5, align 8
  %22 = sext i32 %20 to i64
  %23 = getelementptr inbounds i8, ptr %21, i64 %22
  store ptr %23, ptr %5, align 8
  br label %24

24:                                               ; preds = %28, %18
  %25 = load i32, ptr %7, align 4
  %26 = add nsw i32 %25, -1
  store i32 %26, ptr %7, align 4
  %27 = icmp ne i32 %25, 0
  br i1 %27, label %28, label %34

28:                                               ; preds = %24
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds i8, ptr %29, i32 1
  store ptr %30, ptr %6, align 8
  %31 = load i8, ptr %29, align 1
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds i8, ptr %32, i32 -1
  store ptr %33, ptr %5, align 8
  store i8 %31, ptr %32, align 1
  br label %24, !llvm.loop !35

34:                                               ; preds = %24
  br label %35

35:                                               ; preds = %34, %13
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getoption(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds i8, ptr %11, i32 1
  store ptr %12, ptr %10, align 8
  %13 = load i8, ptr %11, align 1
  %14 = sext i8 %13 to i32
  store i32 %14, ptr %8, align 4
  %15 = load ptr, ptr %7, align 8
  store i32 0, ptr %15, align 4
  %16 = load i32, ptr %8, align 4
  switch i32 %16, label %96 [
    i32 98, label %17
    i32 66, label %19
    i32 104, label %21
    i32 72, label %23
    i32 108, label %25
    i32 76, label %27
    i32 106, label %29
    i32 74, label %31
    i32 84, label %33
    i32 102, label %35
    i32 110, label %37
    i32 100, label %39
    i32 105, label %41
    i32 73, label %46
    i32 115, label %51
    i32 99, label %56
    i32 122, label %74
    i32 120, label %75
    i32 88, label %77
    i32 32, label %78
    i32 60, label %79
    i32 62, label %82
    i32 61, label %85
    i32 33, label %90
  ]

17:                                               ; preds = %3
  %18 = load ptr, ptr %7, align 8
  store i32 1, ptr %18, align 4
  store i32 0, ptr %4, align 4
  br label %103

19:                                               ; preds = %3
  %20 = load ptr, ptr %7, align 8
  store i32 1, ptr %20, align 4
  store i32 1, ptr %4, align 4
  br label %103

21:                                               ; preds = %3
  %22 = load ptr, ptr %7, align 8
  store i32 2, ptr %22, align 4
  store i32 0, ptr %4, align 4
  br label %103

23:                                               ; preds = %3
  %24 = load ptr, ptr %7, align 8
  store i32 2, ptr %24, align 4
  store i32 1, ptr %4, align 4
  br label %103

25:                                               ; preds = %3
  %26 = load ptr, ptr %7, align 8
  store i32 8, ptr %26, align 4
  store i32 0, ptr %4, align 4
  br label %103

27:                                               ; preds = %3
  %28 = load ptr, ptr %7, align 8
  store i32 8, ptr %28, align 4
  store i32 1, ptr %4, align 4
  br label %103

29:                                               ; preds = %3
  %30 = load ptr, ptr %7, align 8
  store i32 8, ptr %30, align 4
  store i32 0, ptr %4, align 4
  br label %103

31:                                               ; preds = %3
  %32 = load ptr, ptr %7, align 8
  store i32 8, ptr %32, align 4
  store i32 1, ptr %4, align 4
  br label %103

33:                                               ; preds = %3
  %34 = load ptr, ptr %7, align 8
  store i32 8, ptr %34, align 4
  store i32 1, ptr %4, align 4
  br label %103

35:                                               ; preds = %3
  %36 = load ptr, ptr %7, align 8
  store i32 4, ptr %36, align 4
  store i32 2, ptr %4, align 4
  br label %103

37:                                               ; preds = %3
  %38 = load ptr, ptr %7, align 8
  store i32 8, ptr %38, align 4
  store i32 3, ptr %4, align 4
  br label %103

39:                                               ; preds = %3
  %40 = load ptr, ptr %7, align 8
  store i32 8, ptr %40, align 4
  store i32 4, ptr %4, align 4
  br label %103

41:                                               ; preds = %3
  %42 = load ptr, ptr %5, align 8
  %43 = load ptr, ptr %6, align 8
  %44 = call i32 @getnumlimit(ptr noundef %42, ptr noundef %43, i32 noundef 4)
  %45 = load ptr, ptr %7, align 8
  store i32 %44, ptr %45, align 4
  store i32 0, ptr %4, align 4
  br label %103

46:                                               ; preds = %3
  %47 = load ptr, ptr %5, align 8
  %48 = load ptr, ptr %6, align 8
  %49 = call i32 @getnumlimit(ptr noundef %47, ptr noundef %48, i32 noundef 4)
  %50 = load ptr, ptr %7, align 8
  store i32 %49, ptr %50, align 4
  store i32 1, ptr %4, align 4
  br label %103

51:                                               ; preds = %3
  %52 = load ptr, ptr %5, align 8
  %53 = load ptr, ptr %6, align 8
  %54 = call i32 @getnumlimit(ptr noundef %52, ptr noundef %53, i32 noundef 8)
  %55 = load ptr, ptr %7, align 8
  store i32 %54, ptr %55, align 4
  store i32 6, ptr %4, align 4
  br label %103

56:                                               ; preds = %3
  %57 = load ptr, ptr %6, align 8
  %58 = call i32 @getnum(ptr noundef %57, i32 noundef -1)
  %59 = load ptr, ptr %7, align 8
  store i32 %58, ptr %59, align 4
  %60 = load ptr, ptr %7, align 8
  %61 = load i32, ptr %60, align 4
  %62 = icmp eq i32 %61, -1
  %63 = zext i1 %62 to i32
  %64 = icmp ne i32 %63, 0
  %65 = zext i1 %64 to i32
  %66 = sext i32 %65 to i64
  %67 = icmp ne i64 %66, 0
  br i1 %67, label %68, label %73

68:                                               ; preds = %56
  %69 = load ptr, ptr %5, align 8
  %70 = getelementptr inbounds %struct.Header, ptr %69, i32 0, i32 0
  %71 = load ptr, ptr %70, align 8
  %72 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %71, ptr noundef @.str.65)
  br label %73

73:                                               ; preds = %68, %56
  store i32 5, ptr %4, align 4
  br label %103

74:                                               ; preds = %3
  store i32 7, ptr %4, align 4
  br label %103

75:                                               ; preds = %3
  %76 = load ptr, ptr %7, align 8
  store i32 1, ptr %76, align 4
  store i32 8, ptr %4, align 4
  br label %103

77:                                               ; preds = %3
  store i32 9, ptr %4, align 4
  br label %103

78:                                               ; preds = %3
  br label %102

79:                                               ; preds = %3
  %80 = load ptr, ptr %5, align 8
  %81 = getelementptr inbounds %struct.Header, ptr %80, i32 0, i32 1
  store i32 1, ptr %81, align 8
  br label %102

82:                                               ; preds = %3
  %83 = load ptr, ptr %5, align 8
  %84 = getelementptr inbounds %struct.Header, ptr %83, i32 0, i32 1
  store i32 0, ptr %84, align 8
  br label %102

85:                                               ; preds = %3
  %86 = load i8, ptr @nativeendian, align 4
  %87 = sext i8 %86 to i32
  %88 = load ptr, ptr %5, align 8
  %89 = getelementptr inbounds %struct.Header, ptr %88, i32 0, i32 1
  store i32 %87, ptr %89, align 8
  br label %102

90:                                               ; preds = %3
  store i32 8, ptr %9, align 4
  %91 = load ptr, ptr %5, align 8
  %92 = load ptr, ptr %6, align 8
  %93 = call i32 @getnumlimit(ptr noundef %91, ptr noundef %92, i32 noundef 8)
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %struct.Header, ptr %94, i32 0, i32 2
  store i32 %93, ptr %95, align 4
  br label %102

96:                                               ; preds = %3
  %97 = load ptr, ptr %5, align 8
  %98 = getelementptr inbounds %struct.Header, ptr %97, i32 0, i32 0
  %99 = load ptr, ptr %98, align 8
  %100 = load i32, ptr %8, align 4
  %101 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %99, ptr noundef @.str.66, i32 noundef %100)
  br label %102

102:                                              ; preds = %96, %90, %85, %82, %79, %78
  store i32 10, ptr %4, align 4
  br label %103

103:                                              ; preds = %102, %77, %75, %74, %73, %51, %46, %41, %39, %37, %35, %33, %31, %29, %27, %25, %23, %21, %19, %17
  %104 = load i32, ptr %4, align 4
  ret i32 %104
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getnumlimit(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %9 = load ptr, ptr %6, align 8
  %10 = load i32, ptr %7, align 4
  %11 = call i32 @getnum(ptr noundef %9, i32 noundef %10)
  store i32 %11, ptr %8, align 4
  %12 = load i32, ptr %8, align 4
  %13 = icmp sgt i32 %12, 16
  br i1 %13, label %17, label %14

14:                                               ; preds = %3
  %15 = load i32, ptr %8, align 4
  %16 = icmp sle i32 %15, 0
  br label %17

17:                                               ; preds = %14, %3
  %18 = phi i1 [ true, %3 ], [ %16, %14 ]
  %19 = zext i1 %18 to i32
  %20 = icmp ne i32 %19, 0
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %24, label %30

24:                                               ; preds = %17
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.Header, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = load i32, ptr %8, align 4
  %29 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %27, ptr noundef @.str.67, i32 noundef %28, i32 noundef 16)
  store i32 %29, ptr %4, align 4
  br label %32

30:                                               ; preds = %17
  %31 = load i32, ptr %8, align 4
  store i32 %31, ptr %4, align 4
  br label %32

32:                                               ; preds = %30, %24
  %33 = load i32, ptr %4, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @getnum(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = load ptr, ptr %7, align 8
  %9 = load i8, ptr %8, align 1
  %10 = sext i8 %9 to i32
  %11 = call i32 @digit(i32 noundef %10)
  %12 = icmp ne i32 %11, 0
  br i1 %12, label %15, label %13

13:                                               ; preds = %2
  %14 = load i32, ptr %5, align 4
  store i32 %14, ptr %3, align 4
  br label %40

15:                                               ; preds = %2
  store i32 0, ptr %6, align 4
  br label %16

16:                                               ; preds = %36, %15
  %17 = load i32, ptr %6, align 4
  %18 = mul nsw i32 %17, 10
  %19 = load ptr, ptr %4, align 8
  %20 = load ptr, ptr %19, align 8
  %21 = getelementptr inbounds i8, ptr %20, i32 1
  store ptr %21, ptr %19, align 8
  %22 = load i8, ptr %20, align 1
  %23 = sext i8 %22 to i32
  %24 = sub nsw i32 %23, 48
  %25 = add nsw i32 %18, %24
  store i32 %25, ptr %6, align 4
  br label %26

26:                                               ; preds = %16
  %27 = load ptr, ptr %4, align 8
  %28 = load ptr, ptr %27, align 8
  %29 = load i8, ptr %28, align 1
  %30 = sext i8 %29 to i32
  %31 = call i32 @digit(i32 noundef %30)
  %32 = icmp ne i32 %31, 0
  br i1 %32, label %33, label %36

33:                                               ; preds = %26
  %34 = load i32, ptr %6, align 4
  %35 = icmp sle i32 %34, 214748363
  br label %36

36:                                               ; preds = %33, %26
  %37 = phi i1 [ false, %26 ], [ %35, %33 ]
  br i1 %37, label %16, label %38, !llvm.loop !36

38:                                               ; preds = %36
  %39 = load i32, ptr %6, align 4
  store i32 %39, ptr %3, align 4
  br label %40

40:                                               ; preds = %38, %13
  %41 = load i32, ptr %3, align 4
  ret i32 %41
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @digit(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  %3 = load i32, ptr %2, align 4
  %4 = icmp sle i32 48, %3
  br i1 %4, label %5, label %8

5:                                                ; preds = %1
  %6 = load i32, ptr %2, align 4
  %7 = icmp sle i32 %6, 57
  br label %8

8:                                                ; preds = %5, %1
  %9 = phi i1 [ false, %1 ], [ %7, %5 ]
  %10 = zext i1 %9 to i32
  ret i32 %10
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @unpackint(ptr noundef %0, ptr noundef %1, i32 noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca i32, align 4
  %13 = alloca i32, align 4
  %14 = alloca i64, align 8
  %15 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i32 %2, ptr %8, align 4
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  store i64 0, ptr %11, align 8
  %16 = load i32, ptr %9, align 4
  %17 = icmp sle i32 %16, 8
  br i1 %17, label %18, label %20

18:                                               ; preds = %5
  %19 = load i32, ptr %9, align 4
  br label %21

20:                                               ; preds = %5
  br label %21

21:                                               ; preds = %20, %18
  %22 = phi i32 [ %19, %18 ], [ 8, %20 ]
  store i32 %22, ptr %13, align 4
  %23 = load i32, ptr %13, align 4
  %24 = sub nsw i32 %23, 1
  store i32 %24, ptr %12, align 4
  br label %25

25:                                               ; preds = %49, %21
  %26 = load i32, ptr %12, align 4
  %27 = icmp sge i32 %26, 0
  br i1 %27, label %28, label %52

28:                                               ; preds = %25
  %29 = load i64, ptr %11, align 8
  %30 = shl i64 %29, 8
  store i64 %30, ptr %11, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = load i32, ptr %8, align 4
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %34, label %36

34:                                               ; preds = %28
  %35 = load i32, ptr %12, align 4
  br label %41

36:                                               ; preds = %28
  %37 = load i32, ptr %9, align 4
  %38 = sub nsw i32 %37, 1
  %39 = load i32, ptr %12, align 4
  %40 = sub nsw i32 %38, %39
  br label %41

41:                                               ; preds = %36, %34
  %42 = phi i32 [ %35, %34 ], [ %40, %36 ]
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds i8, ptr %31, i64 %43
  %45 = load i8, ptr %44, align 1
  %46 = zext i8 %45 to i64
  %47 = load i64, ptr %11, align 8
  %48 = or i64 %47, %46
  store i64 %48, ptr %11, align 8
  br label %49

49:                                               ; preds = %41
  %50 = load i32, ptr %12, align 4
  %51 = add nsw i32 %50, -1
  store i32 %51, ptr %12, align 4
  br label %25, !llvm.loop !37

52:                                               ; preds = %25
  %53 = load i32, ptr %9, align 4
  %54 = icmp slt i32 %53, 8
  br i1 %54, label %55, label %70

55:                                               ; preds = %52
  %56 = load i32, ptr %10, align 4
  %57 = icmp ne i32 %56, 0
  br i1 %57, label %58, label %69

58:                                               ; preds = %55
  %59 = load i32, ptr %9, align 4
  %60 = mul nsw i32 %59, 8
  %61 = sub nsw i32 %60, 1
  %62 = zext i32 %61 to i64
  %63 = shl i64 1, %62
  store i64 %63, ptr %14, align 8
  %64 = load i64, ptr %11, align 8
  %65 = load i64, ptr %14, align 8
  %66 = xor i64 %64, %65
  %67 = load i64, ptr %14, align 8
  %68 = sub i64 %66, %67
  store i64 %68, ptr %11, align 8
  br label %69

69:                                               ; preds = %58, %55
  br label %122

70:                                               ; preds = %52
  %71 = load i32, ptr %9, align 4
  %72 = icmp sgt i32 %71, 8
  br i1 %72, label %73, label %121

73:                                               ; preds = %70
  %74 = load i32, ptr %10, align 4
  %75 = icmp ne i32 %74, 0
  br i1 %75, label %76, label %79

76:                                               ; preds = %73
  %77 = load i64, ptr %11, align 8
  %78 = icmp sge i64 %77, 0
  br label %79

79:                                               ; preds = %76, %73
  %80 = phi i1 [ true, %73 ], [ %78, %76 ]
  %81 = zext i1 %80 to i64
  %82 = select i1 %80, i32 0, i32 255
  store i32 %82, ptr %15, align 4
  %83 = load i32, ptr %13, align 4
  store i32 %83, ptr %12, align 4
  br label %84

84:                                               ; preds = %117, %79
  %85 = load i32, ptr %12, align 4
  %86 = load i32, ptr %9, align 4
  %87 = icmp slt i32 %85, %86
  br i1 %87, label %88, label %120

88:                                               ; preds = %84
  %89 = load ptr, ptr %7, align 8
  %90 = load i32, ptr %8, align 4
  %91 = icmp ne i32 %90, 0
  br i1 %91, label %92, label %94

92:                                               ; preds = %88
  %93 = load i32, ptr %12, align 4
  br label %99

94:                                               ; preds = %88
  %95 = load i32, ptr %9, align 4
  %96 = sub nsw i32 %95, 1
  %97 = load i32, ptr %12, align 4
  %98 = sub nsw i32 %96, %97
  br label %99

99:                                               ; preds = %94, %92
  %100 = phi i32 [ %93, %92 ], [ %98, %94 ]
  %101 = sext i32 %100 to i64
  %102 = getelementptr inbounds i8, ptr %89, i64 %101
  %103 = load i8, ptr %102, align 1
  %104 = zext i8 %103 to i32
  %105 = load i32, ptr %15, align 4
  %106 = icmp ne i32 %104, %105
  %107 = zext i1 %106 to i32
  %108 = icmp ne i32 %107, 0
  %109 = zext i1 %108 to i32
  %110 = sext i32 %109 to i64
  %111 = icmp ne i64 %110, 0
  br i1 %111, label %112, label %116

112:                                              ; preds = %99
  %113 = load ptr, ptr %6, align 8
  %114 = load i32, ptr %9, align 4
  %115 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %113, ptr noundef @.str.74, i32 noundef %114)
  br label %116

116:                                              ; preds = %112, %99
  br label %117

117:                                              ; preds = %116
  %118 = load i32, ptr %12, align 4
  %119 = add nsw i32 %118, 1
  store i32 %119, ptr %12, align 4
  br label %84, !llvm.loop !38

120:                                              ; preds = %84
  br label %121

121:                                              ; preds = %120, %70
  br label %122

122:                                              ; preds = %121, %69
  %123 = load i64, ptr %11, align 8
  ret i64 %123
}

declare void @lua_pushnumber(ptr noundef, double noundef) #1

declare i32 @lua_setmetatable(ptr noundef, i32 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_add(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 0, ptr noundef @.str.76)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_sub(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 1, ptr noundef @.str.77)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_mul(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 2, ptr noundef @.str.78)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_mod(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 3, ptr noundef @.str.79)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_pow(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 4, ptr noundef @.str.80)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_div(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 5, ptr noundef @.str.81)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_idiv(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 6, ptr noundef @.str.82)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith_unm(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @arith(ptr noundef %3, i32 noundef 12, ptr noundef @.str.83)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @arith(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = call i32 @tonum(ptr noundef %7, i32 noundef 1)
  %9 = icmp ne i32 %8, 0
  br i1 %9, label %10, label %17

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = call i32 @tonum(ptr noundef %11, i32 noundef 2)
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %17

14:                                               ; preds = %10
  %15 = load ptr, ptr %4, align 8
  %16 = load i32, ptr %5, align 4
  call void @lua_arith(ptr noundef %15, i32 noundef %16)
  br label %20

17:                                               ; preds = %10, %3
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %6, align 8
  call void @trymt(ptr noundef %18, ptr noundef %19)
  br label %20

20:                                               ; preds = %17, %14
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tonum(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = load i32, ptr %5, align 4
  %10 = call i32 @lua_type(ptr noundef %8, i32 noundef %9)
  %11 = icmp eq i32 %10, 3
  br i1 %11, label %12, label %15

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = load i32, ptr %5, align 4
  call void @lua_pushvalue(ptr noundef %13, i32 noundef %14)
  store i32 1, ptr %3, align 4
  br label %31

15:                                               ; preds = %2
  %16 = load ptr, ptr %4, align 8
  %17 = load i32, ptr %5, align 4
  %18 = call ptr @lua_tolstring(ptr noundef %16, i32 noundef %17, ptr noundef %6)
  store ptr %18, ptr %7, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = icmp ne ptr %19, null
  br i1 %20, label %21, label %28

21:                                               ; preds = %15
  %22 = load ptr, ptr %4, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = call i64 @lua_stringtonumber(ptr noundef %22, ptr noundef %23)
  %25 = load i64, ptr %6, align 8
  %26 = add i64 %25, 1
  %27 = icmp eq i64 %24, %26
  br label %28

28:                                               ; preds = %21, %15
  %29 = phi i1 [ false, %15 ], [ %27, %21 ]
  %30 = zext i1 %29 to i32
  store i32 %30, ptr %3, align 4
  br label %31

31:                                               ; preds = %28, %12
  %32 = load i32, ptr %3, align 4
  ret i32 %32
}

declare void @lua_arith(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @trymt(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  call void @lua_settop(ptr noundef %5, i32 noundef 2)
  %6 = load ptr, ptr %3, align 8
  %7 = call i32 @lua_type(ptr noundef %6, i32 noundef 2)
  %8 = icmp eq i32 %7, 4
  br i1 %8, label %15, label %9

9:                                                ; preds = %2
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call i32 @luaL_getmetafield(ptr noundef %10, i32 noundef 2, ptr noundef %11)
  %13 = icmp ne i32 %12, 0
  %14 = xor i1 %13, true
  br label %15

15:                                               ; preds = %9, %2
  %16 = phi i1 [ true, %2 ], [ %14, %9 ]
  %17 = zext i1 %16 to i32
  %18 = icmp ne i32 %17, 0
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %22, label %35

22:                                               ; preds = %15
  %23 = load ptr, ptr %3, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds i8, ptr %24, i64 2
  %26 = load ptr, ptr %3, align 8
  %27 = load ptr, ptr %3, align 8
  %28 = call i32 @lua_type(ptr noundef %27, i32 noundef -2)
  %29 = call ptr @lua_typename(ptr noundef %26, i32 noundef %28)
  %30 = load ptr, ptr %3, align 8
  %31 = load ptr, ptr %3, align 8
  %32 = call i32 @lua_type(ptr noundef %31, i32 noundef -1)
  %33 = call ptr @lua_typename(ptr noundef %30, i32 noundef %32)
  %34 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %23, ptr noundef @.str.84, ptr noundef %25, ptr noundef %29, ptr noundef %33)
  br label %35

35:                                               ; preds = %22, %15
  %36 = load ptr, ptr %3, align 8
  call void @lua_rotate(ptr noundef %36, i32 noundef -3, i32 noundef 1)
  %37 = load ptr, ptr %3, align 8
  call void @lua_callk(ptr noundef %37, i32 noundef 2, i32 noundef 1, i64 noundef 0, ptr noundef null)
  ret void
}

declare i64 @lua_stringtonumber(ptr noundef, ptr noundef) #1

declare i32 @luaL_getmetafield(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_rotate(ptr noundef, i32 noundef, i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #6 = { nounwind }
attributes #7 = { nounwind willreturn memory(read) }
attributes #8 = { nounwind willreturn memory(none) }

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
!18 = distinct !{!18, !7}
!19 = distinct !{!19, !7}
!20 = distinct !{!20, !7}
!21 = distinct !{!21, !7}
!22 = distinct !{!22, !7}
!23 = distinct !{!23, !7}
!24 = distinct !{!24, !7}
!25 = distinct !{!25, !7}
!26 = distinct !{!26, !7}
!27 = distinct !{!27, !7}
!28 = distinct !{!28, !7}
!29 = distinct !{!29, !7}
!30 = distinct !{!30, !7}
!31 = distinct !{!31, !7}
!32 = distinct !{!32, !7}
!33 = distinct !{!33, !7}
!34 = distinct !{!34, !7}
!35 = distinct !{!35, !7}
!36 = distinct !{!36, !7}
!37 = distinct !{!37, !7}
!38 = distinct !{!38, !7}
