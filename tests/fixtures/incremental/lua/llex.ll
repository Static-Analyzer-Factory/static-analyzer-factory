; ModuleID = 'llex.c'
source_filename = "llex.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.LexState = type { i32, i32, i32, %struct.Token, %struct.Token, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Token = type { i32, %union.SemInfo }
%union.SemInfo = type { double }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.NodeKey = type { %union.Value, i8, i8, i32, %union.Value }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon.0, %union.anon.2, i16, i16 }
%union.anon.0 = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%union.StackValue = type { %struct.TValue }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.Mbuffer = type { ptr, i64, i64 }
%struct.Zio = type { i64, ptr, ptr, ptr, ptr }

@.str = private unnamed_addr constant [5 x i8] c"_ENV\00", align 1
@luaX_tokens = internal constant [37 x ptr] [ptr @.str.4, ptr @.str.5, ptr @.str.6, ptr @.str.7, ptr @.str.8, ptr @.str.9, ptr @.str.10, ptr @.str.11, ptr @.str.12, ptr @.str.13, ptr @.str.14, ptr @.str.15, ptr @.str.16, ptr @.str.17, ptr @.str.18, ptr @.str.19, ptr @.str.20, ptr @.str.21, ptr @.str.22, ptr @.str.23, ptr @.str.24, ptr @.str.25, ptr @.str.26, ptr @.str.27, ptr @.str.28, ptr @.str.29, ptr @.str.30, ptr @.str.31, ptr @.str.32, ptr @.str.33, ptr @.str.34, ptr @.str.35, ptr @.str.36, ptr @.str.37, ptr @.str.38, ptr @.str.39, ptr @.str.40], align 16
@luai_ctype_ = external hidden constant [257 x i8], align 16
@.str.1 = private unnamed_addr constant [5 x i8] c"'%c'\00", align 1
@.str.2 = private unnamed_addr constant [8 x i8] c"'<\\%d>'\00", align 1
@.str.3 = private unnamed_addr constant [5 x i8] c"'%s'\00", align 1
@.str.4 = private unnamed_addr constant [4 x i8] c"and\00", align 1
@.str.5 = private unnamed_addr constant [6 x i8] c"break\00", align 1
@.str.6 = private unnamed_addr constant [3 x i8] c"do\00", align 1
@.str.7 = private unnamed_addr constant [5 x i8] c"else\00", align 1
@.str.8 = private unnamed_addr constant [7 x i8] c"elseif\00", align 1
@.str.9 = private unnamed_addr constant [4 x i8] c"end\00", align 1
@.str.10 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.11 = private unnamed_addr constant [4 x i8] c"for\00", align 1
@.str.12 = private unnamed_addr constant [9 x i8] c"function\00", align 1
@.str.13 = private unnamed_addr constant [5 x i8] c"goto\00", align 1
@.str.14 = private unnamed_addr constant [3 x i8] c"if\00", align 1
@.str.15 = private unnamed_addr constant [3 x i8] c"in\00", align 1
@.str.16 = private unnamed_addr constant [6 x i8] c"local\00", align 1
@.str.17 = private unnamed_addr constant [4 x i8] c"nil\00", align 1
@.str.18 = private unnamed_addr constant [4 x i8] c"not\00", align 1
@.str.19 = private unnamed_addr constant [3 x i8] c"or\00", align 1
@.str.20 = private unnamed_addr constant [7 x i8] c"repeat\00", align 1
@.str.21 = private unnamed_addr constant [7 x i8] c"return\00", align 1
@.str.22 = private unnamed_addr constant [5 x i8] c"then\00", align 1
@.str.23 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.24 = private unnamed_addr constant [6 x i8] c"until\00", align 1
@.str.25 = private unnamed_addr constant [6 x i8] c"while\00", align 1
@.str.26 = private unnamed_addr constant [3 x i8] c"//\00", align 1
@.str.27 = private unnamed_addr constant [3 x i8] c"..\00", align 1
@.str.28 = private unnamed_addr constant [4 x i8] c"...\00", align 1
@.str.29 = private unnamed_addr constant [3 x i8] c"==\00", align 1
@.str.30 = private unnamed_addr constant [3 x i8] c">=\00", align 1
@.str.31 = private unnamed_addr constant [3 x i8] c"<=\00", align 1
@.str.32 = private unnamed_addr constant [3 x i8] c"~=\00", align 1
@.str.33 = private unnamed_addr constant [3 x i8] c"<<\00", align 1
@.str.34 = private unnamed_addr constant [3 x i8] c">>\00", align 1
@.str.35 = private unnamed_addr constant [3 x i8] c"::\00", align 1
@.str.36 = private unnamed_addr constant [6 x i8] c"<eof>\00", align 1
@.str.37 = private unnamed_addr constant [9 x i8] c"<number>\00", align 1
@.str.38 = private unnamed_addr constant [10 x i8] c"<integer>\00", align 1
@.str.39 = private unnamed_addr constant [7 x i8] c"<name>\00", align 1
@.str.40 = private unnamed_addr constant [9 x i8] c"<string>\00", align 1
@.str.41 = private unnamed_addr constant [11 x i8] c"%s near %s\00", align 1
@.str.42 = private unnamed_addr constant [25 x i8] c"lexical element too long\00", align 1
@.str.43 = private unnamed_addr constant [30 x i8] c"invalid long string delimiter\00", align 1
@.str.44 = private unnamed_addr constant [25 x i8] c"chunk has too many lines\00", align 1
@.str.45 = private unnamed_addr constant [7 x i8] c"string\00", align 1
@.str.46 = private unnamed_addr constant [8 x i8] c"comment\00", align 1
@.str.47 = private unnamed_addr constant [41 x i8] c"unfinished long %s (starting at line %d)\00", align 1
@.str.48 = private unnamed_addr constant [18 x i8] c"unfinished string\00", align 1
@.str.49 = private unnamed_addr constant [24 x i8] c"invalid escape sequence\00", align 1
@.str.50 = private unnamed_addr constant [27 x i8] c"hexadecimal digit expected\00", align 1
@.str.51 = private unnamed_addr constant [12 x i8] c"missing '{'\00", align 1
@.str.52 = private unnamed_addr constant [22 x i8] c"UTF-8 value too large\00", align 1
@.str.53 = private unnamed_addr constant [12 x i8] c"missing '}'\00", align 1
@.str.54 = private unnamed_addr constant [25 x i8] c"decimal escape too large\00", align 1
@.str.55 = private unnamed_addr constant [3 x i8] c"Ee\00", align 1
@.str.56 = private unnamed_addr constant [3 x i8] c"xX\00", align 1
@.str.57 = private unnamed_addr constant [3 x i8] c"Pp\00", align 1
@.str.58 = private unnamed_addr constant [3 x i8] c"-+\00", align 1
@.str.59 = private unnamed_addr constant [17 x i8] c"malformed number\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaX_init(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call ptr @luaS_newlstr(ptr noundef %6, ptr noundef @.str, i64 noundef 4)
  store ptr %7, ptr %4, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %4, align 8
  call void @luaC_fix(ptr noundef %8, ptr noundef %9)
  store i32 0, ptr %3, align 4
  br label %10

10:                                               ; preds = %27, %1
  %11 = load i32, ptr %3, align 4
  %12 = icmp slt i32 %11, 22
  br i1 %12, label %13, label %30

13:                                               ; preds = %10
  %14 = load ptr, ptr %2, align 8
  %15 = load i32, ptr %3, align 4
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds [37 x ptr], ptr @luaX_tokens, i64 0, i64 %16
  %18 = load ptr, ptr %17, align 8
  %19 = call ptr @luaS_new(ptr noundef %14, ptr noundef %18)
  store ptr %19, ptr %5, align 8
  %20 = load ptr, ptr %2, align 8
  %21 = load ptr, ptr %5, align 8
  call void @luaC_fix(ptr noundef %20, ptr noundef %21)
  %22 = load i32, ptr %3, align 4
  %23 = add nsw i32 %22, 1
  %24 = trunc i32 %23 to i8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %struct.TString, ptr %25, i32 0, i32 3
  store i8 %24, ptr %26, align 2
  br label %27

27:                                               ; preds = %13
  %28 = load i32, ptr %3, align 4
  %29 = add nsw i32 %28, 1
  store i32 %29, ptr %3, align 4
  br label %10, !llvm.loop !6

30:                                               ; preds = %10
  ret void
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

declare hidden void @luaC_fix(ptr noundef, ptr noundef) #1

declare hidden ptr @luaS_new(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaX_token2str(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %7 = load i32, ptr %5, align 4
  %8 = icmp slt i32 %7, 256
  br i1 %8, label %9, label %30

9:                                                ; preds = %2
  %10 = load i32, ptr %5, align 4
  %11 = add nsw i32 %10, 1
  %12 = sext i32 %11 to i64
  %13 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %12
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 4
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %24

18:                                               ; preds = %9
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 6
  %21 = load ptr, ptr %20, align 8
  %22 = load i32, ptr %5, align 4
  %23 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %21, ptr noundef @.str.1, i32 noundef %22)
  store ptr %23, ptr %3, align 8
  br label %46

24:                                               ; preds = %9
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.LexState, ptr %25, i32 0, i32 6
  %27 = load ptr, ptr %26, align 8
  %28 = load i32, ptr %5, align 4
  %29 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %27, ptr noundef @.str.2, i32 noundef %28)
  store ptr %29, ptr %3, align 8
  br label %46

30:                                               ; preds = %2
  %31 = load i32, ptr %5, align 4
  %32 = sub nsw i32 %31, 256
  %33 = sext i32 %32 to i64
  %34 = getelementptr inbounds [37 x ptr], ptr @luaX_tokens, i64 0, i64 %33
  %35 = load ptr, ptr %34, align 8
  store ptr %35, ptr %6, align 8
  %36 = load i32, ptr %5, align 4
  %37 = icmp slt i32 %36, 288
  br i1 %37, label %38, label %44

38:                                               ; preds = %30
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.LexState, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = load ptr, ptr %6, align 8
  %43 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %41, ptr noundef @.str.3, ptr noundef %42)
  store ptr %43, ptr %3, align 8
  br label %46

44:                                               ; preds = %30
  %45 = load ptr, ptr %6, align 8
  store ptr %45, ptr %3, align 8
  br label %46

46:                                               ; preds = %44, %38, %24, %18
  %47 = load ptr, ptr %3, align 8
  ret ptr %47
}

declare hidden ptr @luaO_pushfstring(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaX_syntaxerror(ptr noundef %0, ptr noundef %1) #2 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 3
  %9 = getelementptr inbounds %struct.Token, ptr %8, i32 0, i32 0
  %10 = load i32, ptr %9, align 8
  call void @lexerror(ptr noundef %5, ptr noundef %6, i32 noundef %10) #5
  unreachable
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define internal void @lexerror(ptr noundef %0, ptr noundef %1, i32 noundef %2) #2 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 6
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 11
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 1
  %16 = load i32, ptr %15, align 4
  %17 = call ptr @luaG_addinfo(ptr noundef %9, ptr noundef %10, ptr noundef %13, i32 noundef %16)
  store ptr %17, ptr %5, align 8
  %18 = load i32, ptr %6, align 4
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %29

20:                                               ; preds = %3
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.LexState, ptr %21, i32 0, i32 6
  %23 = load ptr, ptr %22, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = load i32, ptr %6, align 4
  %27 = call ptr @txtToken(ptr noundef %25, i32 noundef %26)
  %28 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %23, ptr noundef @.str.41, ptr noundef %24, ptr noundef %27)
  br label %29

29:                                               ; preds = %20, %3
  %30 = load ptr, ptr %4, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  call void @luaD_throw(ptr noundef %32, i32 noundef 3) #5
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaX_newstring(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %7, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load i64, ptr %6, align 8
  %19 = call ptr @luaS_newlstr(ptr noundef %16, ptr noundef %17, i64 noundef %18)
  store ptr %19, ptr %8, align 8
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 9
  %22 = load ptr, ptr %21, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = call ptr @luaH_getstr(ptr noundef %22, ptr noundef %23)
  store ptr %24, ptr %9, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  %27 = load i8, ptr %26, align 8
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 15
  %30 = icmp eq i32 %29, 0
  br i1 %30, label %35, label %31

31:                                               ; preds = %3
  %32 = load ptr, ptr %9, align 8
  %33 = getelementptr inbounds %struct.NodeKey, ptr %32, i32 0, i32 4
  %34 = load ptr, ptr %33, align 8
  store ptr %34, ptr %8, align 8
  br label %74

35:                                               ; preds = %3
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.lua_State, ptr %36, i32 0, i32 6
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %union.StackValue, ptr %38, i32 1
  store ptr %39, ptr %37, align 8
  store ptr %38, ptr %10, align 8
  %40 = load ptr, ptr %10, align 8
  store ptr %40, ptr %11, align 8
  %41 = load ptr, ptr %8, align 8
  store ptr %41, ptr %12, align 8
  %42 = load ptr, ptr %12, align 8
  %43 = load ptr, ptr %11, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 0
  store ptr %42, ptr %44, align 8
  %45 = load ptr, ptr %12, align 8
  %46 = getelementptr inbounds %struct.TString, ptr %45, i32 0, i32 1
  %47 = load i8, ptr %46, align 8
  %48 = zext i8 %47 to i32
  %49 = or i32 %48, 64
  %50 = trunc i32 %49 to i8
  %51 = load ptr, ptr %11, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 1
  store i8 %50, ptr %52, align 8
  %53 = load ptr, ptr %7, align 8
  %54 = load ptr, ptr %7, align 8
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 9
  %57 = load ptr, ptr %56, align 8
  %58 = load ptr, ptr %10, align 8
  %59 = load ptr, ptr %9, align 8
  %60 = load ptr, ptr %10, align 8
  call void @luaH_finishset(ptr noundef %54, ptr noundef %57, ptr noundef %58, ptr noundef %59, ptr noundef %60)
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.lua_State, ptr %61, i32 0, i32 7
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %struct.global_State, ptr %63, i32 0, i32 3
  %65 = load i64, ptr %64, align 8
  %66 = icmp sgt i64 %65, 0
  br i1 %66, label %67, label %69

67:                                               ; preds = %35
  %68 = load ptr, ptr %7, align 8
  call void @luaC_step(ptr noundef %68)
  br label %69

69:                                               ; preds = %67, %35
  %70 = load ptr, ptr %7, align 8
  %71 = getelementptr inbounds %struct.lua_State, ptr %70, i32 0, i32 6
  %72 = load ptr, ptr %71, align 8
  %73 = getelementptr inbounds %union.StackValue, ptr %72, i32 -1
  store ptr %73, ptr %71, align 8
  br label %74

74:                                               ; preds = %69, %31
  %75 = load ptr, ptr %8, align 8
  ret ptr %75
}

declare hidden ptr @luaH_getstr(ptr noundef, ptr noundef) #1

declare hidden void @luaH_finishset(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

declare hidden void @luaC_step(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaX_setinput(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i32, align 4
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store i32 %4, ptr %10, align 4
  %11 = load ptr, ptr %7, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 3
  %13 = getelementptr inbounds %struct.Token, ptr %12, i32 0, i32 0
  store i32 0, ptr %13, align 8
  %14 = load ptr, ptr %6, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 6
  store ptr %14, ptr %16, align 8
  %17 = load i32, ptr %10, align 4
  %18 = load ptr, ptr %7, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 0
  store i32 %17, ptr %19, align 8
  %20 = load ptr, ptr %7, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 4
  %22 = getelementptr inbounds %struct.Token, ptr %21, i32 0, i32 0
  store i32 288, ptr %22, align 8
  %23 = load ptr, ptr %8, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 7
  store ptr %23, ptr %25, align 8
  %26 = load ptr, ptr %7, align 8
  %27 = getelementptr inbounds %struct.LexState, ptr %26, i32 0, i32 5
  store ptr null, ptr %27, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.LexState, ptr %28, i32 0, i32 1
  store i32 1, ptr %29, align 4
  %30 = load ptr, ptr %7, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 2
  store i32 1, ptr %31, align 8
  %32 = load ptr, ptr %9, align 8
  %33 = load ptr, ptr %7, align 8
  %34 = getelementptr inbounds %struct.LexState, ptr %33, i32 0, i32 11
  store ptr %32, ptr %34, align 8
  %35 = load ptr, ptr %6, align 8
  %36 = call ptr @luaS_newlstr(ptr noundef %35, ptr noundef @.str, i64 noundef 4)
  %37 = load ptr, ptr %7, align 8
  %38 = getelementptr inbounds %struct.LexState, ptr %37, i32 0, i32 12
  store ptr %36, ptr %38, align 8
  %39 = load ptr, ptr %7, align 8
  %40 = getelementptr inbounds %struct.LexState, ptr %39, i32 0, i32 6
  %41 = load ptr, ptr %40, align 8
  %42 = load ptr, ptr %7, align 8
  %43 = getelementptr inbounds %struct.LexState, ptr %42, i32 0, i32 8
  %44 = load ptr, ptr %43, align 8
  %45 = getelementptr inbounds %struct.Mbuffer, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = getelementptr inbounds %struct.LexState, ptr %47, i32 0, i32 8
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.Mbuffer, ptr %49, i32 0, i32 2
  %51 = load i64, ptr %50, align 8
  %52 = mul i64 %51, 1
  %53 = call ptr @luaM_saferealloc_(ptr noundef %41, ptr noundef %46, i64 noundef %52, i64 noundef 32)
  %54 = load ptr, ptr %7, align 8
  %55 = getelementptr inbounds %struct.LexState, ptr %54, i32 0, i32 8
  %56 = load ptr, ptr %55, align 8
  %57 = getelementptr inbounds %struct.Mbuffer, ptr %56, i32 0, i32 0
  store ptr %53, ptr %57, align 8
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.LexState, ptr %58, i32 0, i32 8
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds %struct.Mbuffer, ptr %60, i32 0, i32 2
  store i64 32, ptr %61, align 8
  ret void
}

declare hidden ptr @luaM_saferealloc_(ptr noundef, ptr noundef, i64 noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaX_next(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %struct.LexState, ptr %3, i32 0, i32 1
  %5 = load i32, ptr %4, align 4
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 2
  store i32 %5, ptr %7, align 8
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 4
  %10 = getelementptr inbounds %struct.Token, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  %12 = icmp ne i32 %11, 288
  br i1 %12, label %13, label %21

13:                                               ; preds = %1
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 3
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %15, ptr align 8 %17, i64 16, i1 false)
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.LexState, ptr %18, i32 0, i32 4
  %20 = getelementptr inbounds %struct.Token, ptr %19, i32 0, i32 0
  store i32 288, ptr %20, align 8
  br label %30

21:                                               ; preds = %1
  %22 = load ptr, ptr %2, align 8
  %23 = load ptr, ptr %2, align 8
  %24 = getelementptr inbounds %struct.LexState, ptr %23, i32 0, i32 3
  %25 = getelementptr inbounds %struct.Token, ptr %24, i32 0, i32 1
  %26 = call i32 @llex(ptr noundef %22, ptr noundef %25)
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.LexState, ptr %27, i32 0, i32 3
  %29 = getelementptr inbounds %struct.Token, ptr %28, i32 0, i32 0
  store i32 %26, ptr %29, align 8
  br label %30

30:                                               ; preds = %21, %13
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @llex(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 8
  %12 = load ptr, ptr %11, align 8
  %13 = getelementptr inbounds %struct.Mbuffer, ptr %12, i32 0, i32 1
  store i64 0, ptr %13, align 8
  br label %14

14:                                               ; preds = %564, %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 0
  %17 = load i32, ptr %16, align 8
  switch i32 %17, label %447 [
    i32 10, label %18
    i32 13, label %18
    i32 32, label %20
    i32 12, label %20
    i32 9, label %20
    i32 11, label %20
    i32 45, label %46
    i32 91, label %169
    i32 61, label %185
    i32 60, label %216
    i32 62, label %252
    i32 47, label %288
    i32 126, label %319
    i32 58, label %350
    i32 34, label %381
    i32 39, label %381
    i32 46, label %387
    i32 48, label %442
    i32 49, label %442
    i32 50, label %442
    i32 51, label %442
    i32 52, label %442
    i32 53, label %442
    i32 54, label %442
    i32 55, label %442
    i32 56, label %442
    i32 57, label %442
    i32 -1, label %446
  ]

18:                                               ; preds = %14, %14
  %19 = load ptr, ptr %4, align 8
  call void @inclinenumber(ptr noundef %19)
  br label %564

20:                                               ; preds = %14, %14, %14, %14
  %21 = load ptr, ptr %4, align 8
  %22 = getelementptr inbounds %struct.LexState, ptr %21, i32 0, i32 7
  %23 = load ptr, ptr %22, align 8
  %24 = getelementptr inbounds %struct.Zio, ptr %23, i32 0, i32 0
  %25 = load i64, ptr %24, align 8
  %26 = add i64 %25, -1
  store i64 %26, ptr %24, align 8
  %27 = icmp ugt i64 %25, 0
  br i1 %27, label %28, label %37

28:                                               ; preds = %20
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.LexState, ptr %29, i32 0, i32 7
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds %struct.Zio, ptr %31, i32 0, i32 1
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds i8, ptr %33, i32 1
  store ptr %34, ptr %32, align 8
  %35 = load i8, ptr %33, align 1
  %36 = zext i8 %35 to i32
  br label %42

37:                                               ; preds = %20
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.LexState, ptr %38, i32 0, i32 7
  %40 = load ptr, ptr %39, align 8
  %41 = call i32 @luaZ_fill(ptr noundef %40)
  br label %42

42:                                               ; preds = %37, %28
  %43 = phi i32 [ %36, %28 ], [ %41, %37 ]
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.LexState, ptr %44, i32 0, i32 0
  store i32 %43, ptr %45, align 8
  br label %564

46:                                               ; preds = %14
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.LexState, ptr %47, i32 0, i32 7
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.Zio, ptr %49, i32 0, i32 0
  %51 = load i64, ptr %50, align 8
  %52 = add i64 %51, -1
  store i64 %52, ptr %50, align 8
  %53 = icmp ugt i64 %51, 0
  br i1 %53, label %54, label %63

54:                                               ; preds = %46
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 7
  %57 = load ptr, ptr %56, align 8
  %58 = getelementptr inbounds %struct.Zio, ptr %57, i32 0, i32 1
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds i8, ptr %59, i32 1
  store ptr %60, ptr %58, align 8
  %61 = load i8, ptr %59, align 1
  %62 = zext i8 %61 to i32
  br label %68

63:                                               ; preds = %46
  %64 = load ptr, ptr %4, align 8
  %65 = getelementptr inbounds %struct.LexState, ptr %64, i32 0, i32 7
  %66 = load ptr, ptr %65, align 8
  %67 = call i32 @luaZ_fill(ptr noundef %66)
  br label %68

68:                                               ; preds = %63, %54
  %69 = phi i32 [ %62, %54 ], [ %67, %63 ]
  %70 = load ptr, ptr %4, align 8
  %71 = getelementptr inbounds %struct.LexState, ptr %70, i32 0, i32 0
  store i32 %69, ptr %71, align 8
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds %struct.LexState, ptr %72, i32 0, i32 0
  %74 = load i32, ptr %73, align 8
  %75 = icmp ne i32 %74, 45
  br i1 %75, label %76, label %77

76:                                               ; preds = %68
  store i32 45, ptr %3, align 4
  br label %565

77:                                               ; preds = %68
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.LexState, ptr %78, i32 0, i32 7
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds %struct.Zio, ptr %80, i32 0, i32 0
  %82 = load i64, ptr %81, align 8
  %83 = add i64 %82, -1
  store i64 %83, ptr %81, align 8
  %84 = icmp ugt i64 %82, 0
  br i1 %84, label %85, label %94

85:                                               ; preds = %77
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.LexState, ptr %86, i32 0, i32 7
  %88 = load ptr, ptr %87, align 8
  %89 = getelementptr inbounds %struct.Zio, ptr %88, i32 0, i32 1
  %90 = load ptr, ptr %89, align 8
  %91 = getelementptr inbounds i8, ptr %90, i32 1
  store ptr %91, ptr %89, align 8
  %92 = load i8, ptr %90, align 1
  %93 = zext i8 %92 to i32
  br label %99

94:                                               ; preds = %77
  %95 = load ptr, ptr %4, align 8
  %96 = getelementptr inbounds %struct.LexState, ptr %95, i32 0, i32 7
  %97 = load ptr, ptr %96, align 8
  %98 = call i32 @luaZ_fill(ptr noundef %97)
  br label %99

99:                                               ; preds = %94, %85
  %100 = phi i32 [ %93, %85 ], [ %98, %94 ]
  %101 = load ptr, ptr %4, align 8
  %102 = getelementptr inbounds %struct.LexState, ptr %101, i32 0, i32 0
  store i32 %100, ptr %102, align 8
  %103 = load ptr, ptr %4, align 8
  %104 = getelementptr inbounds %struct.LexState, ptr %103, i32 0, i32 0
  %105 = load i32, ptr %104, align 8
  %106 = icmp eq i32 %105, 91
  br i1 %106, label %107, label %124

107:                                              ; preds = %99
  %108 = load ptr, ptr %4, align 8
  %109 = call i64 @skip_sep(ptr noundef %108)
  store i64 %109, ptr %6, align 8
  %110 = load ptr, ptr %4, align 8
  %111 = getelementptr inbounds %struct.LexState, ptr %110, i32 0, i32 8
  %112 = load ptr, ptr %111, align 8
  %113 = getelementptr inbounds %struct.Mbuffer, ptr %112, i32 0, i32 1
  store i64 0, ptr %113, align 8
  %114 = load i64, ptr %6, align 8
  %115 = icmp uge i64 %114, 2
  br i1 %115, label %116, label %123

116:                                              ; preds = %107
  %117 = load ptr, ptr %4, align 8
  %118 = load i64, ptr %6, align 8
  call void @read_long_string(ptr noundef %117, ptr noundef null, i64 noundef %118)
  %119 = load ptr, ptr %4, align 8
  %120 = getelementptr inbounds %struct.LexState, ptr %119, i32 0, i32 8
  %121 = load ptr, ptr %120, align 8
  %122 = getelementptr inbounds %struct.Mbuffer, ptr %121, i32 0, i32 1
  store i64 0, ptr %122, align 8
  br label %564

123:                                              ; preds = %107
  br label %124

124:                                              ; preds = %123, %99
  br label %125

125:                                              ; preds = %164, %124
  %126 = load ptr, ptr %4, align 8
  %127 = getelementptr inbounds %struct.LexState, ptr %126, i32 0, i32 0
  %128 = load i32, ptr %127, align 8
  %129 = icmp eq i32 %128, 10
  br i1 %129, label %140, label %130

130:                                              ; preds = %125
  %131 = load ptr, ptr %4, align 8
  %132 = getelementptr inbounds %struct.LexState, ptr %131, i32 0, i32 0
  %133 = load i32, ptr %132, align 8
  %134 = icmp eq i32 %133, 13
  br i1 %134, label %140, label %135

135:                                              ; preds = %130
  %136 = load ptr, ptr %4, align 8
  %137 = getelementptr inbounds %struct.LexState, ptr %136, i32 0, i32 0
  %138 = load i32, ptr %137, align 8
  %139 = icmp ne i32 %138, -1
  br label %140

140:                                              ; preds = %135, %130, %125
  %141 = phi i1 [ false, %130 ], [ false, %125 ], [ %139, %135 ]
  br i1 %141, label %142, label %168

142:                                              ; preds = %140
  %143 = load ptr, ptr %4, align 8
  %144 = getelementptr inbounds %struct.LexState, ptr %143, i32 0, i32 7
  %145 = load ptr, ptr %144, align 8
  %146 = getelementptr inbounds %struct.Zio, ptr %145, i32 0, i32 0
  %147 = load i64, ptr %146, align 8
  %148 = add i64 %147, -1
  store i64 %148, ptr %146, align 8
  %149 = icmp ugt i64 %147, 0
  br i1 %149, label %150, label %159

150:                                              ; preds = %142
  %151 = load ptr, ptr %4, align 8
  %152 = getelementptr inbounds %struct.LexState, ptr %151, i32 0, i32 7
  %153 = load ptr, ptr %152, align 8
  %154 = getelementptr inbounds %struct.Zio, ptr %153, i32 0, i32 1
  %155 = load ptr, ptr %154, align 8
  %156 = getelementptr inbounds i8, ptr %155, i32 1
  store ptr %156, ptr %154, align 8
  %157 = load i8, ptr %155, align 1
  %158 = zext i8 %157 to i32
  br label %164

159:                                              ; preds = %142
  %160 = load ptr, ptr %4, align 8
  %161 = getelementptr inbounds %struct.LexState, ptr %160, i32 0, i32 7
  %162 = load ptr, ptr %161, align 8
  %163 = call i32 @luaZ_fill(ptr noundef %162)
  br label %164

164:                                              ; preds = %159, %150
  %165 = phi i32 [ %158, %150 ], [ %163, %159 ]
  %166 = load ptr, ptr %4, align 8
  %167 = getelementptr inbounds %struct.LexState, ptr %166, i32 0, i32 0
  store i32 %165, ptr %167, align 8
  br label %125, !llvm.loop !8

168:                                              ; preds = %140
  br label %564

169:                                              ; preds = %14
  %170 = load ptr, ptr %4, align 8
  %171 = call i64 @skip_sep(ptr noundef %170)
  store i64 %171, ptr %7, align 8
  %172 = load i64, ptr %7, align 8
  %173 = icmp uge i64 %172, 2
  br i1 %173, label %174, label %178

174:                                              ; preds = %169
  %175 = load ptr, ptr %4, align 8
  %176 = load ptr, ptr %5, align 8
  %177 = load i64, ptr %7, align 8
  call void @read_long_string(ptr noundef %175, ptr noundef %176, i64 noundef %177)
  store i32 292, ptr %3, align 4
  br label %565

178:                                              ; preds = %169
  %179 = load i64, ptr %7, align 8
  %180 = icmp eq i64 %179, 0
  br i1 %180, label %181, label %183

181:                                              ; preds = %178
  %182 = load ptr, ptr %4, align 8
  call void @lexerror(ptr noundef %182, ptr noundef @.str.43, i32 noundef 292) #5
  unreachable

183:                                              ; preds = %178
  br label %184

184:                                              ; preds = %183
  store i32 91, ptr %3, align 4
  br label %565

185:                                              ; preds = %14
  %186 = load ptr, ptr %4, align 8
  %187 = getelementptr inbounds %struct.LexState, ptr %186, i32 0, i32 7
  %188 = load ptr, ptr %187, align 8
  %189 = getelementptr inbounds %struct.Zio, ptr %188, i32 0, i32 0
  %190 = load i64, ptr %189, align 8
  %191 = add i64 %190, -1
  store i64 %191, ptr %189, align 8
  %192 = icmp ugt i64 %190, 0
  br i1 %192, label %193, label %202

193:                                              ; preds = %185
  %194 = load ptr, ptr %4, align 8
  %195 = getelementptr inbounds %struct.LexState, ptr %194, i32 0, i32 7
  %196 = load ptr, ptr %195, align 8
  %197 = getelementptr inbounds %struct.Zio, ptr %196, i32 0, i32 1
  %198 = load ptr, ptr %197, align 8
  %199 = getelementptr inbounds i8, ptr %198, i32 1
  store ptr %199, ptr %197, align 8
  %200 = load i8, ptr %198, align 1
  %201 = zext i8 %200 to i32
  br label %207

202:                                              ; preds = %185
  %203 = load ptr, ptr %4, align 8
  %204 = getelementptr inbounds %struct.LexState, ptr %203, i32 0, i32 7
  %205 = load ptr, ptr %204, align 8
  %206 = call i32 @luaZ_fill(ptr noundef %205)
  br label %207

207:                                              ; preds = %202, %193
  %208 = phi i32 [ %201, %193 ], [ %206, %202 ]
  %209 = load ptr, ptr %4, align 8
  %210 = getelementptr inbounds %struct.LexState, ptr %209, i32 0, i32 0
  store i32 %208, ptr %210, align 8
  %211 = load ptr, ptr %4, align 8
  %212 = call i32 @check_next1(ptr noundef %211, i32 noundef 61)
  %213 = icmp ne i32 %212, 0
  br i1 %213, label %214, label %215

214:                                              ; preds = %207
  store i32 281, ptr %3, align 4
  br label %565

215:                                              ; preds = %207
  store i32 61, ptr %3, align 4
  br label %565

216:                                              ; preds = %14
  %217 = load ptr, ptr %4, align 8
  %218 = getelementptr inbounds %struct.LexState, ptr %217, i32 0, i32 7
  %219 = load ptr, ptr %218, align 8
  %220 = getelementptr inbounds %struct.Zio, ptr %219, i32 0, i32 0
  %221 = load i64, ptr %220, align 8
  %222 = add i64 %221, -1
  store i64 %222, ptr %220, align 8
  %223 = icmp ugt i64 %221, 0
  br i1 %223, label %224, label %233

224:                                              ; preds = %216
  %225 = load ptr, ptr %4, align 8
  %226 = getelementptr inbounds %struct.LexState, ptr %225, i32 0, i32 7
  %227 = load ptr, ptr %226, align 8
  %228 = getelementptr inbounds %struct.Zio, ptr %227, i32 0, i32 1
  %229 = load ptr, ptr %228, align 8
  %230 = getelementptr inbounds i8, ptr %229, i32 1
  store ptr %230, ptr %228, align 8
  %231 = load i8, ptr %229, align 1
  %232 = zext i8 %231 to i32
  br label %238

233:                                              ; preds = %216
  %234 = load ptr, ptr %4, align 8
  %235 = getelementptr inbounds %struct.LexState, ptr %234, i32 0, i32 7
  %236 = load ptr, ptr %235, align 8
  %237 = call i32 @luaZ_fill(ptr noundef %236)
  br label %238

238:                                              ; preds = %233, %224
  %239 = phi i32 [ %232, %224 ], [ %237, %233 ]
  %240 = load ptr, ptr %4, align 8
  %241 = getelementptr inbounds %struct.LexState, ptr %240, i32 0, i32 0
  store i32 %239, ptr %241, align 8
  %242 = load ptr, ptr %4, align 8
  %243 = call i32 @check_next1(ptr noundef %242, i32 noundef 61)
  %244 = icmp ne i32 %243, 0
  br i1 %244, label %245, label %246

245:                                              ; preds = %238
  store i32 283, ptr %3, align 4
  br label %565

246:                                              ; preds = %238
  %247 = load ptr, ptr %4, align 8
  %248 = call i32 @check_next1(ptr noundef %247, i32 noundef 60)
  %249 = icmp ne i32 %248, 0
  br i1 %249, label %250, label %251

250:                                              ; preds = %246
  store i32 285, ptr %3, align 4
  br label %565

251:                                              ; preds = %246
  store i32 60, ptr %3, align 4
  br label %565

252:                                              ; preds = %14
  %253 = load ptr, ptr %4, align 8
  %254 = getelementptr inbounds %struct.LexState, ptr %253, i32 0, i32 7
  %255 = load ptr, ptr %254, align 8
  %256 = getelementptr inbounds %struct.Zio, ptr %255, i32 0, i32 0
  %257 = load i64, ptr %256, align 8
  %258 = add i64 %257, -1
  store i64 %258, ptr %256, align 8
  %259 = icmp ugt i64 %257, 0
  br i1 %259, label %260, label %269

260:                                              ; preds = %252
  %261 = load ptr, ptr %4, align 8
  %262 = getelementptr inbounds %struct.LexState, ptr %261, i32 0, i32 7
  %263 = load ptr, ptr %262, align 8
  %264 = getelementptr inbounds %struct.Zio, ptr %263, i32 0, i32 1
  %265 = load ptr, ptr %264, align 8
  %266 = getelementptr inbounds i8, ptr %265, i32 1
  store ptr %266, ptr %264, align 8
  %267 = load i8, ptr %265, align 1
  %268 = zext i8 %267 to i32
  br label %274

269:                                              ; preds = %252
  %270 = load ptr, ptr %4, align 8
  %271 = getelementptr inbounds %struct.LexState, ptr %270, i32 0, i32 7
  %272 = load ptr, ptr %271, align 8
  %273 = call i32 @luaZ_fill(ptr noundef %272)
  br label %274

274:                                              ; preds = %269, %260
  %275 = phi i32 [ %268, %260 ], [ %273, %269 ]
  %276 = load ptr, ptr %4, align 8
  %277 = getelementptr inbounds %struct.LexState, ptr %276, i32 0, i32 0
  store i32 %275, ptr %277, align 8
  %278 = load ptr, ptr %4, align 8
  %279 = call i32 @check_next1(ptr noundef %278, i32 noundef 61)
  %280 = icmp ne i32 %279, 0
  br i1 %280, label %281, label %282

281:                                              ; preds = %274
  store i32 282, ptr %3, align 4
  br label %565

282:                                              ; preds = %274
  %283 = load ptr, ptr %4, align 8
  %284 = call i32 @check_next1(ptr noundef %283, i32 noundef 62)
  %285 = icmp ne i32 %284, 0
  br i1 %285, label %286, label %287

286:                                              ; preds = %282
  store i32 286, ptr %3, align 4
  br label %565

287:                                              ; preds = %282
  store i32 62, ptr %3, align 4
  br label %565

288:                                              ; preds = %14
  %289 = load ptr, ptr %4, align 8
  %290 = getelementptr inbounds %struct.LexState, ptr %289, i32 0, i32 7
  %291 = load ptr, ptr %290, align 8
  %292 = getelementptr inbounds %struct.Zio, ptr %291, i32 0, i32 0
  %293 = load i64, ptr %292, align 8
  %294 = add i64 %293, -1
  store i64 %294, ptr %292, align 8
  %295 = icmp ugt i64 %293, 0
  br i1 %295, label %296, label %305

296:                                              ; preds = %288
  %297 = load ptr, ptr %4, align 8
  %298 = getelementptr inbounds %struct.LexState, ptr %297, i32 0, i32 7
  %299 = load ptr, ptr %298, align 8
  %300 = getelementptr inbounds %struct.Zio, ptr %299, i32 0, i32 1
  %301 = load ptr, ptr %300, align 8
  %302 = getelementptr inbounds i8, ptr %301, i32 1
  store ptr %302, ptr %300, align 8
  %303 = load i8, ptr %301, align 1
  %304 = zext i8 %303 to i32
  br label %310

305:                                              ; preds = %288
  %306 = load ptr, ptr %4, align 8
  %307 = getelementptr inbounds %struct.LexState, ptr %306, i32 0, i32 7
  %308 = load ptr, ptr %307, align 8
  %309 = call i32 @luaZ_fill(ptr noundef %308)
  br label %310

310:                                              ; preds = %305, %296
  %311 = phi i32 [ %304, %296 ], [ %309, %305 ]
  %312 = load ptr, ptr %4, align 8
  %313 = getelementptr inbounds %struct.LexState, ptr %312, i32 0, i32 0
  store i32 %311, ptr %313, align 8
  %314 = load ptr, ptr %4, align 8
  %315 = call i32 @check_next1(ptr noundef %314, i32 noundef 47)
  %316 = icmp ne i32 %315, 0
  br i1 %316, label %317, label %318

317:                                              ; preds = %310
  store i32 278, ptr %3, align 4
  br label %565

318:                                              ; preds = %310
  store i32 47, ptr %3, align 4
  br label %565

319:                                              ; preds = %14
  %320 = load ptr, ptr %4, align 8
  %321 = getelementptr inbounds %struct.LexState, ptr %320, i32 0, i32 7
  %322 = load ptr, ptr %321, align 8
  %323 = getelementptr inbounds %struct.Zio, ptr %322, i32 0, i32 0
  %324 = load i64, ptr %323, align 8
  %325 = add i64 %324, -1
  store i64 %325, ptr %323, align 8
  %326 = icmp ugt i64 %324, 0
  br i1 %326, label %327, label %336

327:                                              ; preds = %319
  %328 = load ptr, ptr %4, align 8
  %329 = getelementptr inbounds %struct.LexState, ptr %328, i32 0, i32 7
  %330 = load ptr, ptr %329, align 8
  %331 = getelementptr inbounds %struct.Zio, ptr %330, i32 0, i32 1
  %332 = load ptr, ptr %331, align 8
  %333 = getelementptr inbounds i8, ptr %332, i32 1
  store ptr %333, ptr %331, align 8
  %334 = load i8, ptr %332, align 1
  %335 = zext i8 %334 to i32
  br label %341

336:                                              ; preds = %319
  %337 = load ptr, ptr %4, align 8
  %338 = getelementptr inbounds %struct.LexState, ptr %337, i32 0, i32 7
  %339 = load ptr, ptr %338, align 8
  %340 = call i32 @luaZ_fill(ptr noundef %339)
  br label %341

341:                                              ; preds = %336, %327
  %342 = phi i32 [ %335, %327 ], [ %340, %336 ]
  %343 = load ptr, ptr %4, align 8
  %344 = getelementptr inbounds %struct.LexState, ptr %343, i32 0, i32 0
  store i32 %342, ptr %344, align 8
  %345 = load ptr, ptr %4, align 8
  %346 = call i32 @check_next1(ptr noundef %345, i32 noundef 61)
  %347 = icmp ne i32 %346, 0
  br i1 %347, label %348, label %349

348:                                              ; preds = %341
  store i32 284, ptr %3, align 4
  br label %565

349:                                              ; preds = %341
  store i32 126, ptr %3, align 4
  br label %565

350:                                              ; preds = %14
  %351 = load ptr, ptr %4, align 8
  %352 = getelementptr inbounds %struct.LexState, ptr %351, i32 0, i32 7
  %353 = load ptr, ptr %352, align 8
  %354 = getelementptr inbounds %struct.Zio, ptr %353, i32 0, i32 0
  %355 = load i64, ptr %354, align 8
  %356 = add i64 %355, -1
  store i64 %356, ptr %354, align 8
  %357 = icmp ugt i64 %355, 0
  br i1 %357, label %358, label %367

358:                                              ; preds = %350
  %359 = load ptr, ptr %4, align 8
  %360 = getelementptr inbounds %struct.LexState, ptr %359, i32 0, i32 7
  %361 = load ptr, ptr %360, align 8
  %362 = getelementptr inbounds %struct.Zio, ptr %361, i32 0, i32 1
  %363 = load ptr, ptr %362, align 8
  %364 = getelementptr inbounds i8, ptr %363, i32 1
  store ptr %364, ptr %362, align 8
  %365 = load i8, ptr %363, align 1
  %366 = zext i8 %365 to i32
  br label %372

367:                                              ; preds = %350
  %368 = load ptr, ptr %4, align 8
  %369 = getelementptr inbounds %struct.LexState, ptr %368, i32 0, i32 7
  %370 = load ptr, ptr %369, align 8
  %371 = call i32 @luaZ_fill(ptr noundef %370)
  br label %372

372:                                              ; preds = %367, %358
  %373 = phi i32 [ %366, %358 ], [ %371, %367 ]
  %374 = load ptr, ptr %4, align 8
  %375 = getelementptr inbounds %struct.LexState, ptr %374, i32 0, i32 0
  store i32 %373, ptr %375, align 8
  %376 = load ptr, ptr %4, align 8
  %377 = call i32 @check_next1(ptr noundef %376, i32 noundef 58)
  %378 = icmp ne i32 %377, 0
  br i1 %378, label %379, label %380

379:                                              ; preds = %372
  store i32 287, ptr %3, align 4
  br label %565

380:                                              ; preds = %372
  store i32 58, ptr %3, align 4
  br label %565

381:                                              ; preds = %14, %14
  %382 = load ptr, ptr %4, align 8
  %383 = load ptr, ptr %4, align 8
  %384 = getelementptr inbounds %struct.LexState, ptr %383, i32 0, i32 0
  %385 = load i32, ptr %384, align 8
  %386 = load ptr, ptr %5, align 8
  call void @read_string(ptr noundef %382, i32 noundef %385, ptr noundef %386)
  store i32 292, ptr %3, align 4
  br label %565

387:                                              ; preds = %14
  %388 = load ptr, ptr %4, align 8
  %389 = load ptr, ptr %4, align 8
  %390 = getelementptr inbounds %struct.LexState, ptr %389, i32 0, i32 0
  %391 = load i32, ptr %390, align 8
  call void @save(ptr noundef %388, i32 noundef %391)
  %392 = load ptr, ptr %4, align 8
  %393 = getelementptr inbounds %struct.LexState, ptr %392, i32 0, i32 7
  %394 = load ptr, ptr %393, align 8
  %395 = getelementptr inbounds %struct.Zio, ptr %394, i32 0, i32 0
  %396 = load i64, ptr %395, align 8
  %397 = add i64 %396, -1
  store i64 %397, ptr %395, align 8
  %398 = icmp ugt i64 %396, 0
  br i1 %398, label %399, label %408

399:                                              ; preds = %387
  %400 = load ptr, ptr %4, align 8
  %401 = getelementptr inbounds %struct.LexState, ptr %400, i32 0, i32 7
  %402 = load ptr, ptr %401, align 8
  %403 = getelementptr inbounds %struct.Zio, ptr %402, i32 0, i32 1
  %404 = load ptr, ptr %403, align 8
  %405 = getelementptr inbounds i8, ptr %404, i32 1
  store ptr %405, ptr %403, align 8
  %406 = load i8, ptr %404, align 1
  %407 = zext i8 %406 to i32
  br label %413

408:                                              ; preds = %387
  %409 = load ptr, ptr %4, align 8
  %410 = getelementptr inbounds %struct.LexState, ptr %409, i32 0, i32 7
  %411 = load ptr, ptr %410, align 8
  %412 = call i32 @luaZ_fill(ptr noundef %411)
  br label %413

413:                                              ; preds = %408, %399
  %414 = phi i32 [ %407, %399 ], [ %412, %408 ]
  %415 = load ptr, ptr %4, align 8
  %416 = getelementptr inbounds %struct.LexState, ptr %415, i32 0, i32 0
  store i32 %414, ptr %416, align 8
  %417 = load ptr, ptr %4, align 8
  %418 = call i32 @check_next1(ptr noundef %417, i32 noundef 46)
  %419 = icmp ne i32 %418, 0
  br i1 %419, label %420, label %426

420:                                              ; preds = %413
  %421 = load ptr, ptr %4, align 8
  %422 = call i32 @check_next1(ptr noundef %421, i32 noundef 46)
  %423 = icmp ne i32 %422, 0
  br i1 %423, label %424, label %425

424:                                              ; preds = %420
  store i32 280, ptr %3, align 4
  br label %565

425:                                              ; preds = %420
  store i32 279, ptr %3, align 4
  br label %565

426:                                              ; preds = %413
  %427 = load ptr, ptr %4, align 8
  %428 = getelementptr inbounds %struct.LexState, ptr %427, i32 0, i32 0
  %429 = load i32, ptr %428, align 8
  %430 = add nsw i32 %429, 1
  %431 = sext i32 %430 to i64
  %432 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %431
  %433 = load i8, ptr %432, align 1
  %434 = zext i8 %433 to i32
  %435 = and i32 %434, 2
  %436 = icmp ne i32 %435, 0
  br i1 %436, label %438, label %437

437:                                              ; preds = %426
  store i32 46, ptr %3, align 4
  br label %565

438:                                              ; preds = %426
  %439 = load ptr, ptr %4, align 8
  %440 = load ptr, ptr %5, align 8
  %441 = call i32 @read_numeral(ptr noundef %439, ptr noundef %440)
  store i32 %441, ptr %3, align 4
  br label %565

442:                                              ; preds = %14, %14, %14, %14, %14, %14, %14, %14, %14, %14
  %443 = load ptr, ptr %4, align 8
  %444 = load ptr, ptr %5, align 8
  %445 = call i32 @read_numeral(ptr noundef %443, ptr noundef %444)
  store i32 %445, ptr %3, align 4
  br label %565

446:                                              ; preds = %14
  store i32 288, ptr %3, align 4
  br label %565

447:                                              ; preds = %14
  %448 = load ptr, ptr %4, align 8
  %449 = getelementptr inbounds %struct.LexState, ptr %448, i32 0, i32 0
  %450 = load i32, ptr %449, align 8
  %451 = add nsw i32 %450, 1
  %452 = sext i32 %451 to i64
  %453 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %452
  %454 = load i8, ptr %453, align 1
  %455 = zext i8 %454 to i32
  %456 = and i32 %455, 1
  %457 = icmp ne i32 %456, 0
  br i1 %457, label %458, label %534

458:                                              ; preds = %447
  br label %459

459:                                              ; preds = %489, %458
  %460 = load ptr, ptr %4, align 8
  %461 = load ptr, ptr %4, align 8
  %462 = getelementptr inbounds %struct.LexState, ptr %461, i32 0, i32 0
  %463 = load i32, ptr %462, align 8
  call void @save(ptr noundef %460, i32 noundef %463)
  %464 = load ptr, ptr %4, align 8
  %465 = getelementptr inbounds %struct.LexState, ptr %464, i32 0, i32 7
  %466 = load ptr, ptr %465, align 8
  %467 = getelementptr inbounds %struct.Zio, ptr %466, i32 0, i32 0
  %468 = load i64, ptr %467, align 8
  %469 = add i64 %468, -1
  store i64 %469, ptr %467, align 8
  %470 = icmp ugt i64 %468, 0
  br i1 %470, label %471, label %480

471:                                              ; preds = %459
  %472 = load ptr, ptr %4, align 8
  %473 = getelementptr inbounds %struct.LexState, ptr %472, i32 0, i32 7
  %474 = load ptr, ptr %473, align 8
  %475 = getelementptr inbounds %struct.Zio, ptr %474, i32 0, i32 1
  %476 = load ptr, ptr %475, align 8
  %477 = getelementptr inbounds i8, ptr %476, i32 1
  store ptr %477, ptr %475, align 8
  %478 = load i8, ptr %476, align 1
  %479 = zext i8 %478 to i32
  br label %485

480:                                              ; preds = %459
  %481 = load ptr, ptr %4, align 8
  %482 = getelementptr inbounds %struct.LexState, ptr %481, i32 0, i32 7
  %483 = load ptr, ptr %482, align 8
  %484 = call i32 @luaZ_fill(ptr noundef %483)
  br label %485

485:                                              ; preds = %480, %471
  %486 = phi i32 [ %479, %471 ], [ %484, %480 ]
  %487 = load ptr, ptr %4, align 8
  %488 = getelementptr inbounds %struct.LexState, ptr %487, i32 0, i32 0
  store i32 %486, ptr %488, align 8
  br label %489

489:                                              ; preds = %485
  %490 = load ptr, ptr %4, align 8
  %491 = getelementptr inbounds %struct.LexState, ptr %490, i32 0, i32 0
  %492 = load i32, ptr %491, align 8
  %493 = add nsw i32 %492, 1
  %494 = sext i32 %493 to i64
  %495 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %494
  %496 = load i8, ptr %495, align 1
  %497 = zext i8 %496 to i32
  %498 = and i32 %497, 3
  %499 = icmp ne i32 %498, 0
  br i1 %499, label %459, label %500, !llvm.loop !9

500:                                              ; preds = %489
  %501 = load ptr, ptr %4, align 8
  %502 = load ptr, ptr %4, align 8
  %503 = getelementptr inbounds %struct.LexState, ptr %502, i32 0, i32 8
  %504 = load ptr, ptr %503, align 8
  %505 = getelementptr inbounds %struct.Mbuffer, ptr %504, i32 0, i32 0
  %506 = load ptr, ptr %505, align 8
  %507 = load ptr, ptr %4, align 8
  %508 = getelementptr inbounds %struct.LexState, ptr %507, i32 0, i32 8
  %509 = load ptr, ptr %508, align 8
  %510 = getelementptr inbounds %struct.Mbuffer, ptr %509, i32 0, i32 1
  %511 = load i64, ptr %510, align 8
  %512 = call ptr @luaX_newstring(ptr noundef %501, ptr noundef %506, i64 noundef %511)
  store ptr %512, ptr %8, align 8
  %513 = load ptr, ptr %8, align 8
  %514 = load ptr, ptr %5, align 8
  store ptr %513, ptr %514, align 8
  %515 = load ptr, ptr %8, align 8
  %516 = getelementptr inbounds %struct.TString, ptr %515, i32 0, i32 1
  %517 = load i8, ptr %516, align 8
  %518 = zext i8 %517 to i32
  %519 = icmp eq i32 %518, 4
  br i1 %519, label %520, label %533

520:                                              ; preds = %500
  %521 = load ptr, ptr %8, align 8
  %522 = getelementptr inbounds %struct.TString, ptr %521, i32 0, i32 3
  %523 = load i8, ptr %522, align 2
  %524 = zext i8 %523 to i32
  %525 = icmp sgt i32 %524, 0
  br i1 %525, label %526, label %533

526:                                              ; preds = %520
  %527 = load ptr, ptr %8, align 8
  %528 = getelementptr inbounds %struct.TString, ptr %527, i32 0, i32 3
  %529 = load i8, ptr %528, align 2
  %530 = zext i8 %529 to i32
  %531 = sub nsw i32 %530, 1
  %532 = add nsw i32 %531, 256
  store i32 %532, ptr %3, align 4
  br label %565

533:                                              ; preds = %520, %500
  store i32 291, ptr %3, align 4
  br label %565

534:                                              ; preds = %447
  %535 = load ptr, ptr %4, align 8
  %536 = getelementptr inbounds %struct.LexState, ptr %535, i32 0, i32 0
  %537 = load i32, ptr %536, align 8
  store i32 %537, ptr %9, align 4
  %538 = load ptr, ptr %4, align 8
  %539 = getelementptr inbounds %struct.LexState, ptr %538, i32 0, i32 7
  %540 = load ptr, ptr %539, align 8
  %541 = getelementptr inbounds %struct.Zio, ptr %540, i32 0, i32 0
  %542 = load i64, ptr %541, align 8
  %543 = add i64 %542, -1
  store i64 %543, ptr %541, align 8
  %544 = icmp ugt i64 %542, 0
  br i1 %544, label %545, label %554

545:                                              ; preds = %534
  %546 = load ptr, ptr %4, align 8
  %547 = getelementptr inbounds %struct.LexState, ptr %546, i32 0, i32 7
  %548 = load ptr, ptr %547, align 8
  %549 = getelementptr inbounds %struct.Zio, ptr %548, i32 0, i32 1
  %550 = load ptr, ptr %549, align 8
  %551 = getelementptr inbounds i8, ptr %550, i32 1
  store ptr %551, ptr %549, align 8
  %552 = load i8, ptr %550, align 1
  %553 = zext i8 %552 to i32
  br label %559

554:                                              ; preds = %534
  %555 = load ptr, ptr %4, align 8
  %556 = getelementptr inbounds %struct.LexState, ptr %555, i32 0, i32 7
  %557 = load ptr, ptr %556, align 8
  %558 = call i32 @luaZ_fill(ptr noundef %557)
  br label %559

559:                                              ; preds = %554, %545
  %560 = phi i32 [ %553, %545 ], [ %558, %554 ]
  %561 = load ptr, ptr %4, align 8
  %562 = getelementptr inbounds %struct.LexState, ptr %561, i32 0, i32 0
  store i32 %560, ptr %562, align 8
  %563 = load i32, ptr %9, align 4
  store i32 %563, ptr %3, align 4
  br label %565

564:                                              ; preds = %168, %116, %42, %18
  br label %14

565:                                              ; preds = %559, %533, %526, %446, %442, %438, %437, %425, %424, %381, %380, %379, %349, %348, %318, %317, %287, %286, %281, %251, %250, %245, %215, %214, %184, %174, %76
  %566 = load i32, ptr %3, align 4
  ret i32 %566
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaX_lookahead(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LexState, ptr %4, i32 0, i32 4
  %6 = getelementptr inbounds %struct.Token, ptr %5, i32 0, i32 1
  %7 = call i32 @llex(ptr noundef %3, ptr noundef %6)
  %8 = load ptr, ptr %2, align 8
  %9 = getelementptr inbounds %struct.LexState, ptr %8, i32 0, i32 4
  %10 = getelementptr inbounds %struct.Token, ptr %9, i32 0, i32 0
  store i32 %7, ptr %10, align 8
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 4
  %13 = getelementptr inbounds %struct.Token, ptr %12, i32 0, i32 0
  %14 = load i32, ptr %13, align 8
  ret i32 %14
}

declare hidden ptr @luaG_addinfo(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @txtToken(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %6 = load i32, ptr %5, align 4
  switch i32 %6, label %18 [
    i32 291, label %7
    i32 292, label %7
    i32 289, label %7
    i32 290, label %7
  ]

7:                                                ; preds = %2, %2, %2, %2
  %8 = load ptr, ptr %4, align 8
  call void @save(ptr noundef %8, i32 noundef 0)
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 6
  %11 = load ptr, ptr %10, align 8
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 8
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Mbuffer, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %11, ptr noundef @.str.3, ptr noundef %16)
  store ptr %17, ptr %3, align 8
  br label %22

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = load i32, ptr %5, align 4
  %21 = call ptr @luaX_token2str(ptr noundef %19, i32 noundef %20)
  store ptr %21, ptr %3, align 8
  br label %22

22:                                               ; preds = %18, %7
  %23 = load ptr, ptr %3, align 8
  ret ptr %23
}

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @save(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 8
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds %struct.Mbuffer, ptr %10, i32 0, i32 1
  %12 = load i64, ptr %11, align 8
  %13 = add i64 %12, 1
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.Mbuffer, ptr %14, i32 0, i32 2
  %16 = load i64, ptr %15, align 8
  %17 = icmp ugt i64 %13, %16
  br i1 %17, label %18, label %48

18:                                               ; preds = %2
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds %struct.Mbuffer, ptr %19, i32 0, i32 2
  %21 = load i64, ptr %20, align 8
  %22 = icmp uge i64 %21, 4611686018427387903
  br i1 %22, label %23, label %25

23:                                               ; preds = %18
  %24 = load ptr, ptr %3, align 8
  call void @lexerror(ptr noundef %24, ptr noundef @.str.42, i32 noundef 0) #5
  unreachable

25:                                               ; preds = %18
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.Mbuffer, ptr %26, i32 0, i32 2
  %28 = load i64, ptr %27, align 8
  %29 = mul i64 %28, 2
  store i64 %29, ptr %6, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 6
  %32 = load ptr, ptr %31, align 8
  %33 = load ptr, ptr %5, align 8
  %34 = getelementptr inbounds %struct.Mbuffer, ptr %33, i32 0, i32 0
  %35 = load ptr, ptr %34, align 8
  %36 = load ptr, ptr %5, align 8
  %37 = getelementptr inbounds %struct.Mbuffer, ptr %36, i32 0, i32 2
  %38 = load i64, ptr %37, align 8
  %39 = mul i64 %38, 1
  %40 = load i64, ptr %6, align 8
  %41 = mul i64 %40, 1
  %42 = call ptr @luaM_saferealloc_(ptr noundef %32, ptr noundef %35, i64 noundef %39, i64 noundef %41)
  %43 = load ptr, ptr %5, align 8
  %44 = getelementptr inbounds %struct.Mbuffer, ptr %43, i32 0, i32 0
  store ptr %42, ptr %44, align 8
  %45 = load i64, ptr %6, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.Mbuffer, ptr %46, i32 0, i32 2
  store i64 %45, ptr %47, align 8
  br label %48

48:                                               ; preds = %25, %2
  %49 = load i32, ptr %4, align 4
  %50 = trunc i32 %49 to i8
  %51 = load ptr, ptr %5, align 8
  %52 = getelementptr inbounds %struct.Mbuffer, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %52, align 8
  %54 = load ptr, ptr %5, align 8
  %55 = getelementptr inbounds %struct.Mbuffer, ptr %54, i32 0, i32 1
  %56 = load i64, ptr %55, align 8
  %57 = add i64 %56, 1
  store i64 %57, ptr %55, align 8
  %58 = getelementptr inbounds i8, ptr %53, i64 %56
  store i8 %50, ptr %58, align 1
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @inclinenumber(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LexState, ptr %4, i32 0, i32 0
  %6 = load i32, ptr %5, align 8
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 7
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Zio, ptr %9, i32 0, i32 0
  %11 = load i64, ptr %10, align 8
  %12 = add i64 %11, -1
  store i64 %12, ptr %10, align 8
  %13 = icmp ugt i64 %11, 0
  br i1 %13, label %14, label %23

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 7
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.Zio, ptr %17, i32 0, i32 1
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds i8, ptr %19, i32 1
  store ptr %20, ptr %18, align 8
  %21 = load i8, ptr %19, align 1
  %22 = zext i8 %21 to i32
  br label %28

23:                                               ; preds = %1
  %24 = load ptr, ptr %2, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 7
  %26 = load ptr, ptr %25, align 8
  %27 = call i32 @luaZ_fill(ptr noundef %26)
  br label %28

28:                                               ; preds = %23, %14
  %29 = phi i32 [ %22, %14 ], [ %27, %23 ]
  %30 = load ptr, ptr %2, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 0
  store i32 %29, ptr %31, align 8
  %32 = load ptr, ptr %2, align 8
  %33 = getelementptr inbounds %struct.LexState, ptr %32, i32 0, i32 0
  %34 = load i32, ptr %33, align 8
  %35 = icmp eq i32 %34, 10
  br i1 %35, label %41, label %36

36:                                               ; preds = %28
  %37 = load ptr, ptr %2, align 8
  %38 = getelementptr inbounds %struct.LexState, ptr %37, i32 0, i32 0
  %39 = load i32, ptr %38, align 8
  %40 = icmp eq i32 %39, 13
  br i1 %40, label %41, label %73

41:                                               ; preds = %36, %28
  %42 = load ptr, ptr %2, align 8
  %43 = getelementptr inbounds %struct.LexState, ptr %42, i32 0, i32 0
  %44 = load i32, ptr %43, align 8
  %45 = load i32, ptr %3, align 4
  %46 = icmp ne i32 %44, %45
  br i1 %46, label %47, label %73

47:                                               ; preds = %41
  %48 = load ptr, ptr %2, align 8
  %49 = getelementptr inbounds %struct.LexState, ptr %48, i32 0, i32 7
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.Zio, ptr %50, i32 0, i32 0
  %52 = load i64, ptr %51, align 8
  %53 = add i64 %52, -1
  store i64 %53, ptr %51, align 8
  %54 = icmp ugt i64 %52, 0
  br i1 %54, label %55, label %64

55:                                               ; preds = %47
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds %struct.LexState, ptr %56, i32 0, i32 7
  %58 = load ptr, ptr %57, align 8
  %59 = getelementptr inbounds %struct.Zio, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds i8, ptr %60, i32 1
  store ptr %61, ptr %59, align 8
  %62 = load i8, ptr %60, align 1
  %63 = zext i8 %62 to i32
  br label %69

64:                                               ; preds = %47
  %65 = load ptr, ptr %2, align 8
  %66 = getelementptr inbounds %struct.LexState, ptr %65, i32 0, i32 7
  %67 = load ptr, ptr %66, align 8
  %68 = call i32 @luaZ_fill(ptr noundef %67)
  br label %69

69:                                               ; preds = %64, %55
  %70 = phi i32 [ %63, %55 ], [ %68, %64 ]
  %71 = load ptr, ptr %2, align 8
  %72 = getelementptr inbounds %struct.LexState, ptr %71, i32 0, i32 0
  store i32 %70, ptr %72, align 8
  br label %73

73:                                               ; preds = %69, %41, %36
  %74 = load ptr, ptr %2, align 8
  %75 = getelementptr inbounds %struct.LexState, ptr %74, i32 0, i32 1
  %76 = load i32, ptr %75, align 4
  %77 = add nsw i32 %76, 1
  store i32 %77, ptr %75, align 4
  %78 = icmp sge i32 %77, 2147483647
  br i1 %78, label %79, label %81

79:                                               ; preds = %73
  %80 = load ptr, ptr %2, align 8
  call void @lexerror(ptr noundef %80, ptr noundef @.str.44, i32 noundef 0) #5
  unreachable

81:                                               ; preds = %73
  ret void
}

declare hidden i32 @luaZ_fill(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @skip_sep(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i64 0, ptr %3, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.LexState, ptr %5, i32 0, i32 0
  %7 = load i32, ptr %6, align 8
  store i32 %7, ptr %4, align 4
  %8 = load ptr, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  call void @save(ptr noundef %8, i32 noundef %11)
  %12 = load ptr, ptr %2, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Zio, ptr %14, i32 0, i32 0
  %16 = load i64, ptr %15, align 8
  %17 = add i64 %16, -1
  store i64 %17, ptr %15, align 8
  %18 = icmp ugt i64 %16, 0
  br i1 %18, label %19, label %28

19:                                               ; preds = %1
  %20 = load ptr, ptr %2, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 7
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.Zio, ptr %22, i32 0, i32 1
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds i8, ptr %24, i32 1
  store ptr %25, ptr %23, align 8
  %26 = load i8, ptr %24, align 1
  %27 = zext i8 %26 to i32
  br label %33

28:                                               ; preds = %1
  %29 = load ptr, ptr %2, align 8
  %30 = getelementptr inbounds %struct.LexState, ptr %29, i32 0, i32 7
  %31 = load ptr, ptr %30, align 8
  %32 = call i32 @luaZ_fill(ptr noundef %31)
  br label %33

33:                                               ; preds = %28, %19
  %34 = phi i32 [ %27, %19 ], [ %32, %28 ]
  %35 = load ptr, ptr %2, align 8
  %36 = getelementptr inbounds %struct.LexState, ptr %35, i32 0, i32 0
  store i32 %34, ptr %36, align 8
  br label %37

37:                                               ; preds = %68, %33
  %38 = load ptr, ptr %2, align 8
  %39 = getelementptr inbounds %struct.LexState, ptr %38, i32 0, i32 0
  %40 = load i32, ptr %39, align 8
  %41 = icmp eq i32 %40, 61
  br i1 %41, label %42, label %74

42:                                               ; preds = %37
  %43 = load ptr, ptr %2, align 8
  %44 = load ptr, ptr %2, align 8
  %45 = getelementptr inbounds %struct.LexState, ptr %44, i32 0, i32 0
  %46 = load i32, ptr %45, align 8
  call void @save(ptr noundef %43, i32 noundef %46)
  %47 = load ptr, ptr %2, align 8
  %48 = getelementptr inbounds %struct.LexState, ptr %47, i32 0, i32 7
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %struct.Zio, ptr %49, i32 0, i32 0
  %51 = load i64, ptr %50, align 8
  %52 = add i64 %51, -1
  store i64 %52, ptr %50, align 8
  %53 = icmp ugt i64 %51, 0
  br i1 %53, label %54, label %63

54:                                               ; preds = %42
  %55 = load ptr, ptr %2, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 7
  %57 = load ptr, ptr %56, align 8
  %58 = getelementptr inbounds %struct.Zio, ptr %57, i32 0, i32 1
  %59 = load ptr, ptr %58, align 8
  %60 = getelementptr inbounds i8, ptr %59, i32 1
  store ptr %60, ptr %58, align 8
  %61 = load i8, ptr %59, align 1
  %62 = zext i8 %61 to i32
  br label %68

63:                                               ; preds = %42
  %64 = load ptr, ptr %2, align 8
  %65 = getelementptr inbounds %struct.LexState, ptr %64, i32 0, i32 7
  %66 = load ptr, ptr %65, align 8
  %67 = call i32 @luaZ_fill(ptr noundef %66)
  br label %68

68:                                               ; preds = %63, %54
  %69 = phi i32 [ %62, %54 ], [ %67, %63 ]
  %70 = load ptr, ptr %2, align 8
  %71 = getelementptr inbounds %struct.LexState, ptr %70, i32 0, i32 0
  store i32 %69, ptr %71, align 8
  %72 = load i64, ptr %3, align 8
  %73 = add i64 %72, 1
  store i64 %73, ptr %3, align 8
  br label %37, !llvm.loop !10

74:                                               ; preds = %37
  %75 = load ptr, ptr %2, align 8
  %76 = getelementptr inbounds %struct.LexState, ptr %75, i32 0, i32 0
  %77 = load i32, ptr %76, align 8
  %78 = load i32, ptr %4, align 4
  %79 = icmp eq i32 %77, %78
  br i1 %79, label %80, label %83

80:                                               ; preds = %74
  %81 = load i64, ptr %3, align 8
  %82 = add i64 %81, 2
  br label %89

83:                                               ; preds = %74
  %84 = load i64, ptr %3, align 8
  %85 = icmp eq i64 %84, 0
  %86 = zext i1 %85 to i64
  %87 = select i1 %85, i32 1, i32 0
  %88 = sext i32 %87 to i64
  br label %89

89:                                               ; preds = %83, %80
  %90 = phi i64 [ %82, %80 ], [ %88, %83 ]
  ret i64 %90
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @read_long_string(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 1
  %12 = load i32, ptr %11, align 4
  store i32 %12, ptr %7, align 4
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.LexState, ptr %14, i32 0, i32 0
  %16 = load i32, ptr %15, align 8
  call void @save(ptr noundef %13, i32 noundef %16)
  %17 = load ptr, ptr %4, align 8
  %18 = getelementptr inbounds %struct.LexState, ptr %17, i32 0, i32 7
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.Zio, ptr %19, i32 0, i32 0
  %21 = load i64, ptr %20, align 8
  %22 = add i64 %21, -1
  store i64 %22, ptr %20, align 8
  %23 = icmp ugt i64 %21, 0
  br i1 %23, label %24, label %33

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.LexState, ptr %25, i32 0, i32 7
  %27 = load ptr, ptr %26, align 8
  %28 = getelementptr inbounds %struct.Zio, ptr %27, i32 0, i32 1
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds i8, ptr %29, i32 1
  store ptr %30, ptr %28, align 8
  %31 = load i8, ptr %29, align 1
  %32 = zext i8 %31 to i32
  br label %38

33:                                               ; preds = %3
  %34 = load ptr, ptr %4, align 8
  %35 = getelementptr inbounds %struct.LexState, ptr %34, i32 0, i32 7
  %36 = load ptr, ptr %35, align 8
  %37 = call i32 @luaZ_fill(ptr noundef %36)
  br label %38

38:                                               ; preds = %33, %24
  %39 = phi i32 [ %32, %24 ], [ %37, %33 ]
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.LexState, ptr %40, i32 0, i32 0
  store i32 %39, ptr %41, align 8
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.LexState, ptr %42, i32 0, i32 0
  %44 = load i32, ptr %43, align 8
  %45 = icmp eq i32 %44, 10
  br i1 %45, label %51, label %46

46:                                               ; preds = %38
  %47 = load ptr, ptr %4, align 8
  %48 = getelementptr inbounds %struct.LexState, ptr %47, i32 0, i32 0
  %49 = load i32, ptr %48, align 8
  %50 = icmp eq i32 %49, 13
  br i1 %50, label %51, label %53

51:                                               ; preds = %46, %38
  %52 = load ptr, ptr %4, align 8
  call void @inclinenumber(ptr noundef %52)
  br label %53

53:                                               ; preds = %51, %46
  br label %54

54:                                               ; preds = %178, %53
  %55 = load ptr, ptr %4, align 8
  %56 = getelementptr inbounds %struct.LexState, ptr %55, i32 0, i32 0
  %57 = load i32, ptr %56, align 8
  switch i32 %57, label %118 [
    i32 -1, label %58
    i32 93, label %71
    i32 10, label %107
    i32 13, label %107
  ]

58:                                               ; preds = %54
  %59 = load ptr, ptr %5, align 8
  %60 = icmp ne ptr %59, null
  %61 = zext i1 %60 to i64
  %62 = select i1 %60, ptr @.str.45, ptr @.str.46
  store ptr %62, ptr %8, align 8
  %63 = load ptr, ptr %4, align 8
  %64 = getelementptr inbounds %struct.LexState, ptr %63, i32 0, i32 6
  %65 = load ptr, ptr %64, align 8
  %66 = load ptr, ptr %8, align 8
  %67 = load i32, ptr %7, align 4
  %68 = call ptr (ptr, ptr, ...) @luaO_pushfstring(ptr noundef %65, ptr noundef @.str.47, ptr noundef %66, i32 noundef %67)
  store ptr %68, ptr %9, align 8
  %69 = load ptr, ptr %4, align 8
  %70 = load ptr, ptr %9, align 8
  call void @lexerror(ptr noundef %69, ptr noundef %70, i32 noundef 288) #5
  unreachable

71:                                               ; preds = %54
  %72 = load ptr, ptr %4, align 8
  %73 = call i64 @skip_sep(ptr noundef %72)
  %74 = load i64, ptr %6, align 8
  %75 = icmp eq i64 %73, %74
  br i1 %75, label %76, label %106

76:                                               ; preds = %71
  %77 = load ptr, ptr %4, align 8
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.LexState, ptr %78, i32 0, i32 0
  %80 = load i32, ptr %79, align 8
  call void @save(ptr noundef %77, i32 noundef %80)
  %81 = load ptr, ptr %4, align 8
  %82 = getelementptr inbounds %struct.LexState, ptr %81, i32 0, i32 7
  %83 = load ptr, ptr %82, align 8
  %84 = getelementptr inbounds %struct.Zio, ptr %83, i32 0, i32 0
  %85 = load i64, ptr %84, align 8
  %86 = add i64 %85, -1
  store i64 %86, ptr %84, align 8
  %87 = icmp ugt i64 %85, 0
  br i1 %87, label %88, label %97

88:                                               ; preds = %76
  %89 = load ptr, ptr %4, align 8
  %90 = getelementptr inbounds %struct.LexState, ptr %89, i32 0, i32 7
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr inbounds %struct.Zio, ptr %91, i32 0, i32 1
  %93 = load ptr, ptr %92, align 8
  %94 = getelementptr inbounds i8, ptr %93, i32 1
  store ptr %94, ptr %92, align 8
  %95 = load i8, ptr %93, align 1
  %96 = zext i8 %95 to i32
  br label %102

97:                                               ; preds = %76
  %98 = load ptr, ptr %4, align 8
  %99 = getelementptr inbounds %struct.LexState, ptr %98, i32 0, i32 7
  %100 = load ptr, ptr %99, align 8
  %101 = call i32 @luaZ_fill(ptr noundef %100)
  br label %102

102:                                              ; preds = %97, %88
  %103 = phi i32 [ %96, %88 ], [ %101, %97 ]
  %104 = load ptr, ptr %4, align 8
  %105 = getelementptr inbounds %struct.LexState, ptr %104, i32 0, i32 0
  store i32 %103, ptr %105, align 8
  br label %179

106:                                              ; preds = %71
  br label %178

107:                                              ; preds = %54, %54
  %108 = load ptr, ptr %4, align 8
  call void @save(ptr noundef %108, i32 noundef 10)
  %109 = load ptr, ptr %4, align 8
  call void @inclinenumber(ptr noundef %109)
  %110 = load ptr, ptr %5, align 8
  %111 = icmp ne ptr %110, null
  br i1 %111, label %117, label %112

112:                                              ; preds = %107
  %113 = load ptr, ptr %4, align 8
  %114 = getelementptr inbounds %struct.LexState, ptr %113, i32 0, i32 8
  %115 = load ptr, ptr %114, align 8
  %116 = getelementptr inbounds %struct.Mbuffer, ptr %115, i32 0, i32 1
  store i64 0, ptr %116, align 8
  br label %117

117:                                              ; preds = %112, %107
  br label %178

118:                                              ; preds = %54
  %119 = load ptr, ptr %5, align 8
  %120 = icmp ne ptr %119, null
  br i1 %120, label %121, label %151

121:                                              ; preds = %118
  %122 = load ptr, ptr %4, align 8
  %123 = load ptr, ptr %4, align 8
  %124 = getelementptr inbounds %struct.LexState, ptr %123, i32 0, i32 0
  %125 = load i32, ptr %124, align 8
  call void @save(ptr noundef %122, i32 noundef %125)
  %126 = load ptr, ptr %4, align 8
  %127 = getelementptr inbounds %struct.LexState, ptr %126, i32 0, i32 7
  %128 = load ptr, ptr %127, align 8
  %129 = getelementptr inbounds %struct.Zio, ptr %128, i32 0, i32 0
  %130 = load i64, ptr %129, align 8
  %131 = add i64 %130, -1
  store i64 %131, ptr %129, align 8
  %132 = icmp ugt i64 %130, 0
  br i1 %132, label %133, label %142

133:                                              ; preds = %121
  %134 = load ptr, ptr %4, align 8
  %135 = getelementptr inbounds %struct.LexState, ptr %134, i32 0, i32 7
  %136 = load ptr, ptr %135, align 8
  %137 = getelementptr inbounds %struct.Zio, ptr %136, i32 0, i32 1
  %138 = load ptr, ptr %137, align 8
  %139 = getelementptr inbounds i8, ptr %138, i32 1
  store ptr %139, ptr %137, align 8
  %140 = load i8, ptr %138, align 1
  %141 = zext i8 %140 to i32
  br label %147

142:                                              ; preds = %121
  %143 = load ptr, ptr %4, align 8
  %144 = getelementptr inbounds %struct.LexState, ptr %143, i32 0, i32 7
  %145 = load ptr, ptr %144, align 8
  %146 = call i32 @luaZ_fill(ptr noundef %145)
  br label %147

147:                                              ; preds = %142, %133
  %148 = phi i32 [ %141, %133 ], [ %146, %142 ]
  %149 = load ptr, ptr %4, align 8
  %150 = getelementptr inbounds %struct.LexState, ptr %149, i32 0, i32 0
  store i32 %148, ptr %150, align 8
  br label %177

151:                                              ; preds = %118
  %152 = load ptr, ptr %4, align 8
  %153 = getelementptr inbounds %struct.LexState, ptr %152, i32 0, i32 7
  %154 = load ptr, ptr %153, align 8
  %155 = getelementptr inbounds %struct.Zio, ptr %154, i32 0, i32 0
  %156 = load i64, ptr %155, align 8
  %157 = add i64 %156, -1
  store i64 %157, ptr %155, align 8
  %158 = icmp ugt i64 %156, 0
  br i1 %158, label %159, label %168

159:                                              ; preds = %151
  %160 = load ptr, ptr %4, align 8
  %161 = getelementptr inbounds %struct.LexState, ptr %160, i32 0, i32 7
  %162 = load ptr, ptr %161, align 8
  %163 = getelementptr inbounds %struct.Zio, ptr %162, i32 0, i32 1
  %164 = load ptr, ptr %163, align 8
  %165 = getelementptr inbounds i8, ptr %164, i32 1
  store ptr %165, ptr %163, align 8
  %166 = load i8, ptr %164, align 1
  %167 = zext i8 %166 to i32
  br label %173

168:                                              ; preds = %151
  %169 = load ptr, ptr %4, align 8
  %170 = getelementptr inbounds %struct.LexState, ptr %169, i32 0, i32 7
  %171 = load ptr, ptr %170, align 8
  %172 = call i32 @luaZ_fill(ptr noundef %171)
  br label %173

173:                                              ; preds = %168, %159
  %174 = phi i32 [ %167, %159 ], [ %172, %168 ]
  %175 = load ptr, ptr %4, align 8
  %176 = getelementptr inbounds %struct.LexState, ptr %175, i32 0, i32 0
  store i32 %174, ptr %176, align 8
  br label %177

177:                                              ; preds = %173, %147
  br label %178

178:                                              ; preds = %177, %117, %106
  br label %54

179:                                              ; preds = %102
  %180 = load ptr, ptr %5, align 8
  %181 = icmp ne ptr %180, null
  br i1 %181, label %182, label %201

182:                                              ; preds = %179
  %183 = load ptr, ptr %4, align 8
  %184 = load ptr, ptr %4, align 8
  %185 = getelementptr inbounds %struct.LexState, ptr %184, i32 0, i32 8
  %186 = load ptr, ptr %185, align 8
  %187 = getelementptr inbounds %struct.Mbuffer, ptr %186, i32 0, i32 0
  %188 = load ptr, ptr %187, align 8
  %189 = load i64, ptr %6, align 8
  %190 = getelementptr inbounds i8, ptr %188, i64 %189
  %191 = load ptr, ptr %4, align 8
  %192 = getelementptr inbounds %struct.LexState, ptr %191, i32 0, i32 8
  %193 = load ptr, ptr %192, align 8
  %194 = getelementptr inbounds %struct.Mbuffer, ptr %193, i32 0, i32 1
  %195 = load i64, ptr %194, align 8
  %196 = load i64, ptr %6, align 8
  %197 = mul i64 2, %196
  %198 = sub i64 %195, %197
  %199 = call ptr @luaX_newstring(ptr noundef %183, ptr noundef %190, i64 noundef %198)
  %200 = load ptr, ptr %5, align 8
  store ptr %199, ptr %200, align 8
  br label %201

201:                                              ; preds = %182, %179
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @check_next1(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  %9 = load i32, ptr %5, align 4
  %10 = icmp eq i32 %8, %9
  br i1 %10, label %11, label %37

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Zio, ptr %14, i32 0, i32 0
  %16 = load i64, ptr %15, align 8
  %17 = add i64 %16, -1
  store i64 %17, ptr %15, align 8
  %18 = icmp ugt i64 %16, 0
  br i1 %18, label %19, label %28

19:                                               ; preds = %11
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 7
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.Zio, ptr %22, i32 0, i32 1
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds i8, ptr %24, i32 1
  store ptr %25, ptr %23, align 8
  %26 = load i8, ptr %24, align 1
  %27 = zext i8 %26 to i32
  br label %33

28:                                               ; preds = %11
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.LexState, ptr %29, i32 0, i32 7
  %31 = load ptr, ptr %30, align 8
  %32 = call i32 @luaZ_fill(ptr noundef %31)
  br label %33

33:                                               ; preds = %28, %19
  %34 = phi i32 [ %27, %19 ], [ %32, %28 ]
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.LexState, ptr %35, i32 0, i32 0
  store i32 %34, ptr %36, align 8
  store i32 1, ptr %3, align 4
  br label %38

37:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %38

38:                                               ; preds = %37, %33
  %39 = load i32, ptr %3, align 4
  ret i32 %39
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @read_string(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  call void @save(ptr noundef %8, i32 noundef %11)
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds %struct.LexState, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  %15 = getelementptr inbounds %struct.Zio, ptr %14, i32 0, i32 0
  %16 = load i64, ptr %15, align 8
  %17 = add i64 %16, -1
  store i64 %17, ptr %15, align 8
  %18 = icmp ugt i64 %16, 0
  br i1 %18, label %19, label %28

19:                                               ; preds = %3
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.LexState, ptr %20, i32 0, i32 7
  %22 = load ptr, ptr %21, align 8
  %23 = getelementptr inbounds %struct.Zio, ptr %22, i32 0, i32 1
  %24 = load ptr, ptr %23, align 8
  %25 = getelementptr inbounds i8, ptr %24, i32 1
  store ptr %25, ptr %23, align 8
  %26 = load i8, ptr %24, align 1
  %27 = zext i8 %26 to i32
  br label %33

28:                                               ; preds = %3
  %29 = load ptr, ptr %4, align 8
  %30 = getelementptr inbounds %struct.LexState, ptr %29, i32 0, i32 7
  %31 = load ptr, ptr %30, align 8
  %32 = call i32 @luaZ_fill(ptr noundef %31)
  br label %33

33:                                               ; preds = %28, %19
  %34 = phi i32 [ %27, %19 ], [ %32, %28 ]
  %35 = load ptr, ptr %4, align 8
  %36 = getelementptr inbounds %struct.LexState, ptr %35, i32 0, i32 0
  store i32 %34, ptr %36, align 8
  br label %37

37:                                               ; preds = %265, %33
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds %struct.LexState, ptr %38, i32 0, i32 0
  %40 = load i32, ptr %39, align 8
  %41 = load i32, ptr %5, align 4
  %42 = icmp ne i32 %40, %41
  br i1 %42, label %43, label %266

43:                                               ; preds = %37
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds %struct.LexState, ptr %44, i32 0, i32 0
  %46 = load i32, ptr %45, align 8
  switch i32 %46, label %235 [
    i32 -1, label %47
    i32 10, label %49
    i32 13, label %49
    i32 92, label %51
  ]

47:                                               ; preds = %43
  %48 = load ptr, ptr %4, align 8
  call void @lexerror(ptr noundef %48, ptr noundef @.str.48, i32 noundef 288) #5
  unreachable

49:                                               ; preds = %43, %43
  %50 = load ptr, ptr %4, align 8
  call void @lexerror(ptr noundef %50, ptr noundef @.str.48, i32 noundef 292) #5
  unreachable

51:                                               ; preds = %43
  %52 = load ptr, ptr %4, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds %struct.LexState, ptr %53, i32 0, i32 0
  %55 = load i32, ptr %54, align 8
  call void @save(ptr noundef %52, i32 noundef %55)
  %56 = load ptr, ptr %4, align 8
  %57 = getelementptr inbounds %struct.LexState, ptr %56, i32 0, i32 7
  %58 = load ptr, ptr %57, align 8
  %59 = getelementptr inbounds %struct.Zio, ptr %58, i32 0, i32 0
  %60 = load i64, ptr %59, align 8
  %61 = add i64 %60, -1
  store i64 %61, ptr %59, align 8
  %62 = icmp ugt i64 %60, 0
  br i1 %62, label %63, label %72

63:                                               ; preds = %51
  %64 = load ptr, ptr %4, align 8
  %65 = getelementptr inbounds %struct.LexState, ptr %64, i32 0, i32 7
  %66 = load ptr, ptr %65, align 8
  %67 = getelementptr inbounds %struct.Zio, ptr %66, i32 0, i32 1
  %68 = load ptr, ptr %67, align 8
  %69 = getelementptr inbounds i8, ptr %68, i32 1
  store ptr %69, ptr %67, align 8
  %70 = load i8, ptr %68, align 1
  %71 = zext i8 %70 to i32
  br label %77

72:                                               ; preds = %51
  %73 = load ptr, ptr %4, align 8
  %74 = getelementptr inbounds %struct.LexState, ptr %73, i32 0, i32 7
  %75 = load ptr, ptr %74, align 8
  %76 = call i32 @luaZ_fill(ptr noundef %75)
  br label %77

77:                                               ; preds = %72, %63
  %78 = phi i32 [ %71, %63 ], [ %76, %72 ]
  %79 = load ptr, ptr %4, align 8
  %80 = getelementptr inbounds %struct.LexState, ptr %79, i32 0, i32 0
  store i32 %78, ptr %80, align 8
  %81 = load ptr, ptr %4, align 8
  %82 = getelementptr inbounds %struct.LexState, ptr %81, i32 0, i32 0
  %83 = load i32, ptr %82, align 8
  switch i32 %83, label %186 [
    i32 97, label %84
    i32 98, label %85
    i32 102, label %86
    i32 110, label %87
    i32 114, label %88
    i32 116, label %89
    i32 118, label %90
    i32 120, label %91
    i32 117, label %94
    i32 10, label %96
    i32 13, label %96
    i32 92, label %98
    i32 34, label %98
    i32 39, label %98
    i32 -1, label %102
    i32 122, label %103
  ]

84:                                               ; preds = %77
  store i32 7, ptr %7, align 4
  br label %199

85:                                               ; preds = %77
  store i32 8, ptr %7, align 4
  br label %199

86:                                               ; preds = %77
  store i32 12, ptr %7, align 4
  br label %199

87:                                               ; preds = %77
  store i32 10, ptr %7, align 4
  br label %199

88:                                               ; preds = %77
  store i32 13, ptr %7, align 4
  br label %199

89:                                               ; preds = %77
  store i32 9, ptr %7, align 4
  br label %199

90:                                               ; preds = %77
  store i32 11, ptr %7, align 4
  br label %199

91:                                               ; preds = %77
  %92 = load ptr, ptr %4, align 8
  %93 = call i32 @readhexaesc(ptr noundef %92)
  store i32 %93, ptr %7, align 4
  br label %199

94:                                               ; preds = %77
  %95 = load ptr, ptr %4, align 8
  call void @utf8esc(ptr noundef %95)
  br label %234

96:                                               ; preds = %77, %77
  %97 = load ptr, ptr %4, align 8
  call void @inclinenumber(ptr noundef %97)
  store i32 10, ptr %7, align 4
  br label %225

98:                                               ; preds = %77, %77, %77
  %99 = load ptr, ptr %4, align 8
  %100 = getelementptr inbounds %struct.LexState, ptr %99, i32 0, i32 0
  %101 = load i32, ptr %100, align 8
  store i32 %101, ptr %7, align 4
  br label %199

102:                                              ; preds = %77
  br label %234

103:                                              ; preds = %77
  %104 = load ptr, ptr %4, align 8
  %105 = getelementptr inbounds %struct.LexState, ptr %104, i32 0, i32 8
  %106 = load ptr, ptr %105, align 8
  %107 = getelementptr inbounds %struct.Mbuffer, ptr %106, i32 0, i32 1
  %108 = load i64, ptr %107, align 8
  %109 = sub i64 %108, 1
  store i64 %109, ptr %107, align 8
  %110 = load ptr, ptr %4, align 8
  %111 = getelementptr inbounds %struct.LexState, ptr %110, i32 0, i32 7
  %112 = load ptr, ptr %111, align 8
  %113 = getelementptr inbounds %struct.Zio, ptr %112, i32 0, i32 0
  %114 = load i64, ptr %113, align 8
  %115 = add i64 %114, -1
  store i64 %115, ptr %113, align 8
  %116 = icmp ugt i64 %114, 0
  br i1 %116, label %117, label %126

117:                                              ; preds = %103
  %118 = load ptr, ptr %4, align 8
  %119 = getelementptr inbounds %struct.LexState, ptr %118, i32 0, i32 7
  %120 = load ptr, ptr %119, align 8
  %121 = getelementptr inbounds %struct.Zio, ptr %120, i32 0, i32 1
  %122 = load ptr, ptr %121, align 8
  %123 = getelementptr inbounds i8, ptr %122, i32 1
  store ptr %123, ptr %121, align 8
  %124 = load i8, ptr %122, align 1
  %125 = zext i8 %124 to i32
  br label %131

126:                                              ; preds = %103
  %127 = load ptr, ptr %4, align 8
  %128 = getelementptr inbounds %struct.LexState, ptr %127, i32 0, i32 7
  %129 = load ptr, ptr %128, align 8
  %130 = call i32 @luaZ_fill(ptr noundef %129)
  br label %131

131:                                              ; preds = %126, %117
  %132 = phi i32 [ %125, %117 ], [ %130, %126 ]
  %133 = load ptr, ptr %4, align 8
  %134 = getelementptr inbounds %struct.LexState, ptr %133, i32 0, i32 0
  store i32 %132, ptr %134, align 8
  br label %135

135:                                              ; preds = %184, %131
  %136 = load ptr, ptr %4, align 8
  %137 = getelementptr inbounds %struct.LexState, ptr %136, i32 0, i32 0
  %138 = load i32, ptr %137, align 8
  %139 = add nsw i32 %138, 1
  %140 = sext i32 %139 to i64
  %141 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %140
  %142 = load i8, ptr %141, align 1
  %143 = zext i8 %142 to i32
  %144 = and i32 %143, 8
  %145 = icmp ne i32 %144, 0
  br i1 %145, label %146, label %185

146:                                              ; preds = %135
  %147 = load ptr, ptr %4, align 8
  %148 = getelementptr inbounds %struct.LexState, ptr %147, i32 0, i32 0
  %149 = load i32, ptr %148, align 8
  %150 = icmp eq i32 %149, 10
  br i1 %150, label %156, label %151

151:                                              ; preds = %146
  %152 = load ptr, ptr %4, align 8
  %153 = getelementptr inbounds %struct.LexState, ptr %152, i32 0, i32 0
  %154 = load i32, ptr %153, align 8
  %155 = icmp eq i32 %154, 13
  br i1 %155, label %156, label %158

156:                                              ; preds = %151, %146
  %157 = load ptr, ptr %4, align 8
  call void @inclinenumber(ptr noundef %157)
  br label %184

158:                                              ; preds = %151
  %159 = load ptr, ptr %4, align 8
  %160 = getelementptr inbounds %struct.LexState, ptr %159, i32 0, i32 7
  %161 = load ptr, ptr %160, align 8
  %162 = getelementptr inbounds %struct.Zio, ptr %161, i32 0, i32 0
  %163 = load i64, ptr %162, align 8
  %164 = add i64 %163, -1
  store i64 %164, ptr %162, align 8
  %165 = icmp ugt i64 %163, 0
  br i1 %165, label %166, label %175

166:                                              ; preds = %158
  %167 = load ptr, ptr %4, align 8
  %168 = getelementptr inbounds %struct.LexState, ptr %167, i32 0, i32 7
  %169 = load ptr, ptr %168, align 8
  %170 = getelementptr inbounds %struct.Zio, ptr %169, i32 0, i32 1
  %171 = load ptr, ptr %170, align 8
  %172 = getelementptr inbounds i8, ptr %171, i32 1
  store ptr %172, ptr %170, align 8
  %173 = load i8, ptr %171, align 1
  %174 = zext i8 %173 to i32
  br label %180

175:                                              ; preds = %158
  %176 = load ptr, ptr %4, align 8
  %177 = getelementptr inbounds %struct.LexState, ptr %176, i32 0, i32 7
  %178 = load ptr, ptr %177, align 8
  %179 = call i32 @luaZ_fill(ptr noundef %178)
  br label %180

180:                                              ; preds = %175, %166
  %181 = phi i32 [ %174, %166 ], [ %179, %175 ]
  %182 = load ptr, ptr %4, align 8
  %183 = getelementptr inbounds %struct.LexState, ptr %182, i32 0, i32 0
  store i32 %181, ptr %183, align 8
  br label %184

184:                                              ; preds = %180, %156
  br label %135, !llvm.loop !11

185:                                              ; preds = %135
  br label %234

186:                                              ; preds = %77
  %187 = load ptr, ptr %4, align 8
  %188 = load ptr, ptr %4, align 8
  %189 = getelementptr inbounds %struct.LexState, ptr %188, i32 0, i32 0
  %190 = load i32, ptr %189, align 8
  %191 = add nsw i32 %190, 1
  %192 = sext i32 %191 to i64
  %193 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %192
  %194 = load i8, ptr %193, align 1
  %195 = zext i8 %194 to i32
  %196 = and i32 %195, 2
  call void @esccheck(ptr noundef %187, i32 noundef %196, ptr noundef @.str.49)
  %197 = load ptr, ptr %4, align 8
  %198 = call i32 @readdecesc(ptr noundef %197)
  store i32 %198, ptr %7, align 4
  br label %225

199:                                              ; preds = %98, %91, %90, %89, %88, %87, %86, %85, %84
  %200 = load ptr, ptr %4, align 8
  %201 = getelementptr inbounds %struct.LexState, ptr %200, i32 0, i32 7
  %202 = load ptr, ptr %201, align 8
  %203 = getelementptr inbounds %struct.Zio, ptr %202, i32 0, i32 0
  %204 = load i64, ptr %203, align 8
  %205 = add i64 %204, -1
  store i64 %205, ptr %203, align 8
  %206 = icmp ugt i64 %204, 0
  br i1 %206, label %207, label %216

207:                                              ; preds = %199
  %208 = load ptr, ptr %4, align 8
  %209 = getelementptr inbounds %struct.LexState, ptr %208, i32 0, i32 7
  %210 = load ptr, ptr %209, align 8
  %211 = getelementptr inbounds %struct.Zio, ptr %210, i32 0, i32 1
  %212 = load ptr, ptr %211, align 8
  %213 = getelementptr inbounds i8, ptr %212, i32 1
  store ptr %213, ptr %211, align 8
  %214 = load i8, ptr %212, align 1
  %215 = zext i8 %214 to i32
  br label %221

216:                                              ; preds = %199
  %217 = load ptr, ptr %4, align 8
  %218 = getelementptr inbounds %struct.LexState, ptr %217, i32 0, i32 7
  %219 = load ptr, ptr %218, align 8
  %220 = call i32 @luaZ_fill(ptr noundef %219)
  br label %221

221:                                              ; preds = %216, %207
  %222 = phi i32 [ %215, %207 ], [ %220, %216 ]
  %223 = load ptr, ptr %4, align 8
  %224 = getelementptr inbounds %struct.LexState, ptr %223, i32 0, i32 0
  store i32 %222, ptr %224, align 8
  br label %225

225:                                              ; preds = %221, %186, %96
  %226 = load ptr, ptr %4, align 8
  %227 = getelementptr inbounds %struct.LexState, ptr %226, i32 0, i32 8
  %228 = load ptr, ptr %227, align 8
  %229 = getelementptr inbounds %struct.Mbuffer, ptr %228, i32 0, i32 1
  %230 = load i64, ptr %229, align 8
  %231 = sub i64 %230, 1
  store i64 %231, ptr %229, align 8
  %232 = load ptr, ptr %4, align 8
  %233 = load i32, ptr %7, align 4
  call void @save(ptr noundef %232, i32 noundef %233)
  br label %234

234:                                              ; preds = %225, %185, %102, %94
  br label %265

235:                                              ; preds = %43
  %236 = load ptr, ptr %4, align 8
  %237 = load ptr, ptr %4, align 8
  %238 = getelementptr inbounds %struct.LexState, ptr %237, i32 0, i32 0
  %239 = load i32, ptr %238, align 8
  call void @save(ptr noundef %236, i32 noundef %239)
  %240 = load ptr, ptr %4, align 8
  %241 = getelementptr inbounds %struct.LexState, ptr %240, i32 0, i32 7
  %242 = load ptr, ptr %241, align 8
  %243 = getelementptr inbounds %struct.Zio, ptr %242, i32 0, i32 0
  %244 = load i64, ptr %243, align 8
  %245 = add i64 %244, -1
  store i64 %245, ptr %243, align 8
  %246 = icmp ugt i64 %244, 0
  br i1 %246, label %247, label %256

247:                                              ; preds = %235
  %248 = load ptr, ptr %4, align 8
  %249 = getelementptr inbounds %struct.LexState, ptr %248, i32 0, i32 7
  %250 = load ptr, ptr %249, align 8
  %251 = getelementptr inbounds %struct.Zio, ptr %250, i32 0, i32 1
  %252 = load ptr, ptr %251, align 8
  %253 = getelementptr inbounds i8, ptr %252, i32 1
  store ptr %253, ptr %251, align 8
  %254 = load i8, ptr %252, align 1
  %255 = zext i8 %254 to i32
  br label %261

256:                                              ; preds = %235
  %257 = load ptr, ptr %4, align 8
  %258 = getelementptr inbounds %struct.LexState, ptr %257, i32 0, i32 7
  %259 = load ptr, ptr %258, align 8
  %260 = call i32 @luaZ_fill(ptr noundef %259)
  br label %261

261:                                              ; preds = %256, %247
  %262 = phi i32 [ %255, %247 ], [ %260, %256 ]
  %263 = load ptr, ptr %4, align 8
  %264 = getelementptr inbounds %struct.LexState, ptr %263, i32 0, i32 0
  store i32 %262, ptr %264, align 8
  br label %265

265:                                              ; preds = %261, %234
  br label %37, !llvm.loop !12

266:                                              ; preds = %37
  %267 = load ptr, ptr %4, align 8
  %268 = load ptr, ptr %4, align 8
  %269 = getelementptr inbounds %struct.LexState, ptr %268, i32 0, i32 0
  %270 = load i32, ptr %269, align 8
  call void @save(ptr noundef %267, i32 noundef %270)
  %271 = load ptr, ptr %4, align 8
  %272 = getelementptr inbounds %struct.LexState, ptr %271, i32 0, i32 7
  %273 = load ptr, ptr %272, align 8
  %274 = getelementptr inbounds %struct.Zio, ptr %273, i32 0, i32 0
  %275 = load i64, ptr %274, align 8
  %276 = add i64 %275, -1
  store i64 %276, ptr %274, align 8
  %277 = icmp ugt i64 %275, 0
  br i1 %277, label %278, label %287

278:                                              ; preds = %266
  %279 = load ptr, ptr %4, align 8
  %280 = getelementptr inbounds %struct.LexState, ptr %279, i32 0, i32 7
  %281 = load ptr, ptr %280, align 8
  %282 = getelementptr inbounds %struct.Zio, ptr %281, i32 0, i32 1
  %283 = load ptr, ptr %282, align 8
  %284 = getelementptr inbounds i8, ptr %283, i32 1
  store ptr %284, ptr %282, align 8
  %285 = load i8, ptr %283, align 1
  %286 = zext i8 %285 to i32
  br label %292

287:                                              ; preds = %266
  %288 = load ptr, ptr %4, align 8
  %289 = getelementptr inbounds %struct.LexState, ptr %288, i32 0, i32 7
  %290 = load ptr, ptr %289, align 8
  %291 = call i32 @luaZ_fill(ptr noundef %290)
  br label %292

292:                                              ; preds = %287, %278
  %293 = phi i32 [ %286, %278 ], [ %291, %287 ]
  %294 = load ptr, ptr %4, align 8
  %295 = getelementptr inbounds %struct.LexState, ptr %294, i32 0, i32 0
  store i32 %293, ptr %295, align 8
  %296 = load ptr, ptr %4, align 8
  %297 = load ptr, ptr %4, align 8
  %298 = getelementptr inbounds %struct.LexState, ptr %297, i32 0, i32 8
  %299 = load ptr, ptr %298, align 8
  %300 = getelementptr inbounds %struct.Mbuffer, ptr %299, i32 0, i32 0
  %301 = load ptr, ptr %300, align 8
  %302 = getelementptr inbounds i8, ptr %301, i64 1
  %303 = load ptr, ptr %4, align 8
  %304 = getelementptr inbounds %struct.LexState, ptr %303, i32 0, i32 8
  %305 = load ptr, ptr %304, align 8
  %306 = getelementptr inbounds %struct.Mbuffer, ptr %305, i32 0, i32 1
  %307 = load i64, ptr %306, align 8
  %308 = sub i64 %307, 2
  %309 = call ptr @luaX_newstring(ptr noundef %296, ptr noundef %302, i64 noundef %308)
  %310 = load ptr, ptr %6, align 8
  store ptr %309, ptr %310, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @read_numeral(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.TValue, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr @.str.55, ptr %7, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  store i32 %11, ptr %8, align 4
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.LexState, ptr %13, i32 0, i32 0
  %15 = load i32, ptr %14, align 8
  call void @save(ptr noundef %12, i32 noundef %15)
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 7
  %18 = load ptr, ptr %17, align 8
  %19 = getelementptr inbounds %struct.Zio, ptr %18, i32 0, i32 0
  %20 = load i64, ptr %19, align 8
  %21 = add i64 %20, -1
  store i64 %21, ptr %19, align 8
  %22 = icmp ugt i64 %20, 0
  br i1 %22, label %23, label %32

23:                                               ; preds = %2
  %24 = load ptr, ptr %4, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 7
  %26 = load ptr, ptr %25, align 8
  %27 = getelementptr inbounds %struct.Zio, ptr %26, i32 0, i32 1
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds i8, ptr %28, i32 1
  store ptr %29, ptr %27, align 8
  %30 = load i8, ptr %28, align 1
  %31 = zext i8 %30 to i32
  br label %37

32:                                               ; preds = %2
  %33 = load ptr, ptr %4, align 8
  %34 = getelementptr inbounds %struct.LexState, ptr %33, i32 0, i32 7
  %35 = load ptr, ptr %34, align 8
  %36 = call i32 @luaZ_fill(ptr noundef %35)
  br label %37

37:                                               ; preds = %32, %23
  %38 = phi i32 [ %31, %23 ], [ %36, %32 ]
  %39 = load ptr, ptr %4, align 8
  %40 = getelementptr inbounds %struct.LexState, ptr %39, i32 0, i32 0
  store i32 %38, ptr %40, align 8
  %41 = load i32, ptr %8, align 4
  %42 = icmp eq i32 %41, 48
  br i1 %42, label %43, label %48

43:                                               ; preds = %37
  %44 = load ptr, ptr %4, align 8
  %45 = call i32 @check_next2(ptr noundef %44, ptr noundef @.str.56)
  %46 = icmp ne i32 %45, 0
  br i1 %46, label %47, label %48

47:                                               ; preds = %43
  store ptr @.str.57, ptr %7, align 8
  br label %48

48:                                               ; preds = %47, %43, %37
  br label %49

49:                                               ; preds = %105, %48
  %50 = load ptr, ptr %4, align 8
  %51 = load ptr, ptr %7, align 8
  %52 = call i32 @check_next2(ptr noundef %50, ptr noundef %51)
  %53 = icmp ne i32 %52, 0
  br i1 %53, label %54, label %57

54:                                               ; preds = %49
  %55 = load ptr, ptr %4, align 8
  %56 = call i32 @check_next2(ptr noundef %55, ptr noundef @.str.58)
  br label %105

57:                                               ; preds = %49
  %58 = load ptr, ptr %4, align 8
  %59 = getelementptr inbounds %struct.LexState, ptr %58, i32 0, i32 0
  %60 = load i32, ptr %59, align 8
  %61 = add nsw i32 %60, 1
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %62
  %64 = load i8, ptr %63, align 1
  %65 = zext i8 %64 to i32
  %66 = and i32 %65, 16
  %67 = icmp ne i32 %66, 0
  br i1 %67, label %73, label %68

68:                                               ; preds = %57
  %69 = load ptr, ptr %4, align 8
  %70 = getelementptr inbounds %struct.LexState, ptr %69, i32 0, i32 0
  %71 = load i32, ptr %70, align 8
  %72 = icmp eq i32 %71, 46
  br i1 %72, label %73, label %103

73:                                               ; preds = %68, %57
  %74 = load ptr, ptr %4, align 8
  %75 = load ptr, ptr %4, align 8
  %76 = getelementptr inbounds %struct.LexState, ptr %75, i32 0, i32 0
  %77 = load i32, ptr %76, align 8
  call void @save(ptr noundef %74, i32 noundef %77)
  %78 = load ptr, ptr %4, align 8
  %79 = getelementptr inbounds %struct.LexState, ptr %78, i32 0, i32 7
  %80 = load ptr, ptr %79, align 8
  %81 = getelementptr inbounds %struct.Zio, ptr %80, i32 0, i32 0
  %82 = load i64, ptr %81, align 8
  %83 = add i64 %82, -1
  store i64 %83, ptr %81, align 8
  %84 = icmp ugt i64 %82, 0
  br i1 %84, label %85, label %94

85:                                               ; preds = %73
  %86 = load ptr, ptr %4, align 8
  %87 = getelementptr inbounds %struct.LexState, ptr %86, i32 0, i32 7
  %88 = load ptr, ptr %87, align 8
  %89 = getelementptr inbounds %struct.Zio, ptr %88, i32 0, i32 1
  %90 = load ptr, ptr %89, align 8
  %91 = getelementptr inbounds i8, ptr %90, i32 1
  store ptr %91, ptr %89, align 8
  %92 = load i8, ptr %90, align 1
  %93 = zext i8 %92 to i32
  br label %99

94:                                               ; preds = %73
  %95 = load ptr, ptr %4, align 8
  %96 = getelementptr inbounds %struct.LexState, ptr %95, i32 0, i32 7
  %97 = load ptr, ptr %96, align 8
  %98 = call i32 @luaZ_fill(ptr noundef %97)
  br label %99

99:                                               ; preds = %94, %85
  %100 = phi i32 [ %93, %85 ], [ %98, %94 ]
  %101 = load ptr, ptr %4, align 8
  %102 = getelementptr inbounds %struct.LexState, ptr %101, i32 0, i32 0
  store i32 %100, ptr %102, align 8
  br label %104

103:                                              ; preds = %68
  br label %106

104:                                              ; preds = %99
  br label %105

105:                                              ; preds = %104, %54
  br label %49

106:                                              ; preds = %103
  %107 = load ptr, ptr %4, align 8
  %108 = getelementptr inbounds %struct.LexState, ptr %107, i32 0, i32 0
  %109 = load i32, ptr %108, align 8
  %110 = add nsw i32 %109, 1
  %111 = sext i32 %110 to i64
  %112 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %111
  %113 = load i8, ptr %112, align 1
  %114 = zext i8 %113 to i32
  %115 = and i32 %114, 1
  %116 = icmp ne i32 %115, 0
  br i1 %116, label %117, label %147

117:                                              ; preds = %106
  %118 = load ptr, ptr %4, align 8
  %119 = load ptr, ptr %4, align 8
  %120 = getelementptr inbounds %struct.LexState, ptr %119, i32 0, i32 0
  %121 = load i32, ptr %120, align 8
  call void @save(ptr noundef %118, i32 noundef %121)
  %122 = load ptr, ptr %4, align 8
  %123 = getelementptr inbounds %struct.LexState, ptr %122, i32 0, i32 7
  %124 = load ptr, ptr %123, align 8
  %125 = getelementptr inbounds %struct.Zio, ptr %124, i32 0, i32 0
  %126 = load i64, ptr %125, align 8
  %127 = add i64 %126, -1
  store i64 %127, ptr %125, align 8
  %128 = icmp ugt i64 %126, 0
  br i1 %128, label %129, label %138

129:                                              ; preds = %117
  %130 = load ptr, ptr %4, align 8
  %131 = getelementptr inbounds %struct.LexState, ptr %130, i32 0, i32 7
  %132 = load ptr, ptr %131, align 8
  %133 = getelementptr inbounds %struct.Zio, ptr %132, i32 0, i32 1
  %134 = load ptr, ptr %133, align 8
  %135 = getelementptr inbounds i8, ptr %134, i32 1
  store ptr %135, ptr %133, align 8
  %136 = load i8, ptr %134, align 1
  %137 = zext i8 %136 to i32
  br label %143

138:                                              ; preds = %117
  %139 = load ptr, ptr %4, align 8
  %140 = getelementptr inbounds %struct.LexState, ptr %139, i32 0, i32 7
  %141 = load ptr, ptr %140, align 8
  %142 = call i32 @luaZ_fill(ptr noundef %141)
  br label %143

143:                                              ; preds = %138, %129
  %144 = phi i32 [ %137, %129 ], [ %142, %138 ]
  %145 = load ptr, ptr %4, align 8
  %146 = getelementptr inbounds %struct.LexState, ptr %145, i32 0, i32 0
  store i32 %144, ptr %146, align 8
  br label %147

147:                                              ; preds = %143, %106
  %148 = load ptr, ptr %4, align 8
  call void @save(ptr noundef %148, i32 noundef 0)
  %149 = load ptr, ptr %4, align 8
  %150 = getelementptr inbounds %struct.LexState, ptr %149, i32 0, i32 8
  %151 = load ptr, ptr %150, align 8
  %152 = getelementptr inbounds %struct.Mbuffer, ptr %151, i32 0, i32 0
  %153 = load ptr, ptr %152, align 8
  %154 = call i64 @luaO_str2num(ptr noundef %153, ptr noundef %6)
  %155 = icmp eq i64 %154, 0
  br i1 %155, label %156, label %158

156:                                              ; preds = %147
  %157 = load ptr, ptr %4, align 8
  call void @lexerror(ptr noundef %157, ptr noundef @.str.59, i32 noundef 289) #5
  unreachable

158:                                              ; preds = %147
  %159 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 1
  %160 = load i8, ptr %159, align 8
  %161 = zext i8 %160 to i32
  %162 = icmp eq i32 %161, 3
  br i1 %162, label %163, label %167

163:                                              ; preds = %158
  %164 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 0
  %165 = load i64, ptr %164, align 8
  %166 = load ptr, ptr %5, align 8
  store i64 %165, ptr %166, align 8
  store i32 290, ptr %3, align 4
  br label %171

167:                                              ; preds = %158
  %168 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 0
  %169 = load double, ptr %168, align 8
  %170 = load ptr, ptr %5, align 8
  store double %169, ptr %170, align 8
  store i32 289, ptr %3, align 4
  br label %171

171:                                              ; preds = %167, %163
  %172 = load i32, ptr %3, align 4
  ret i32 %172
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @readhexaesc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = call i32 @gethexa(ptr noundef %4)
  store i32 %5, ptr %3, align 4
  %6 = load i32, ptr %3, align 4
  %7 = shl i32 %6, 4
  %8 = load ptr, ptr %2, align 8
  %9 = call i32 @gethexa(ptr noundef %8)
  %10 = add nsw i32 %7, %9
  store i32 %10, ptr %3, align 4
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.LexState, ptr %11, i32 0, i32 8
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %struct.Mbuffer, ptr %13, i32 0, i32 1
  %15 = load i64, ptr %14, align 8
  %16 = sub i64 %15, 2
  store i64 %16, ptr %14, align 8
  %17 = load i32, ptr %3, align 4
  ret i32 %17
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @utf8esc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca [8 x i8], align 1
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = getelementptr inbounds [8 x i8], ptr %3, i64 0, i64 0
  %6 = load ptr, ptr %2, align 8
  %7 = call i64 @readutf8esc(ptr noundef %6)
  %8 = call i32 @luaO_utf8esc(ptr noundef %5, i64 noundef %7)
  store i32 %8, ptr %4, align 4
  br label %9

9:                                                ; preds = %20, %1
  %10 = load i32, ptr %4, align 4
  %11 = icmp sgt i32 %10, 0
  br i1 %11, label %12, label %23

12:                                               ; preds = %9
  %13 = load ptr, ptr %2, align 8
  %14 = load i32, ptr %4, align 4
  %15 = sub nsw i32 8, %14
  %16 = sext i32 %15 to i64
  %17 = getelementptr inbounds [8 x i8], ptr %3, i64 0, i64 %16
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  call void @save(ptr noundef %13, i32 noundef %19)
  br label %20

20:                                               ; preds = %12
  %21 = load i32, ptr %4, align 4
  %22 = add nsw i32 %21, -1
  store i32 %22, ptr %4, align 4
  br label %9, !llvm.loop !13

23:                                               ; preds = %9
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @esccheck(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  %7 = load i32, ptr %5, align 4
  %8 = icmp ne i32 %7, 0
  br i1 %8, label %47, label %9

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.LexState, ptr %10, i32 0, i32 0
  %12 = load i32, ptr %11, align 8
  %13 = icmp ne i32 %12, -1
  br i1 %13, label %14, label %44

14:                                               ; preds = %9
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.LexState, ptr %16, i32 0, i32 0
  %18 = load i32, ptr %17, align 8
  call void @save(ptr noundef %15, i32 noundef %18)
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds %struct.LexState, ptr %19, i32 0, i32 7
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds %struct.Zio, ptr %21, i32 0, i32 0
  %23 = load i64, ptr %22, align 8
  %24 = add i64 %23, -1
  store i64 %24, ptr %22, align 8
  %25 = icmp ugt i64 %23, 0
  br i1 %25, label %26, label %35

26:                                               ; preds = %14
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.LexState, ptr %27, i32 0, i32 7
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.Zio, ptr %29, i32 0, i32 1
  %31 = load ptr, ptr %30, align 8
  %32 = getelementptr inbounds i8, ptr %31, i32 1
  store ptr %32, ptr %30, align 8
  %33 = load i8, ptr %31, align 1
  %34 = zext i8 %33 to i32
  br label %40

35:                                               ; preds = %14
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.LexState, ptr %36, i32 0, i32 7
  %38 = load ptr, ptr %37, align 8
  %39 = call i32 @luaZ_fill(ptr noundef %38)
  br label %40

40:                                               ; preds = %35, %26
  %41 = phi i32 [ %34, %26 ], [ %39, %35 ]
  %42 = load ptr, ptr %4, align 8
  %43 = getelementptr inbounds %struct.LexState, ptr %42, i32 0, i32 0
  store i32 %41, ptr %43, align 8
  br label %44

44:                                               ; preds = %40, %9
  %45 = load ptr, ptr %4, align 8
  %46 = load ptr, ptr %6, align 8
  call void @lexerror(ptr noundef %45, ptr noundef %46, i32 noundef 292) #5
  unreachable

47:                                               ; preds = %3
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @readdecesc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 0, ptr %4, align 4
  store i32 0, ptr %3, align 4
  br label %5

5:                                                ; preds = %58, %1
  %6 = load i32, ptr %3, align 4
  %7 = icmp slt i32 %6, 3
  br i1 %7, label %8, label %19

8:                                                ; preds = %5
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 0
  %11 = load i32, ptr %10, align 8
  %12 = add nsw i32 %11, 1
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %13
  %15 = load i8, ptr %14, align 1
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 2
  %18 = icmp ne i32 %17, 0
  br label %19

19:                                               ; preds = %8, %5
  %20 = phi i1 [ false, %5 ], [ %18, %8 ]
  br i1 %20, label %21, label %61

21:                                               ; preds = %19
  %22 = load i32, ptr %4, align 4
  %23 = mul nsw i32 10, %22
  %24 = load ptr, ptr %2, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 0
  %26 = load i32, ptr %25, align 8
  %27 = add nsw i32 %23, %26
  %28 = sub nsw i32 %27, 48
  store i32 %28, ptr %4, align 4
  %29 = load ptr, ptr %2, align 8
  %30 = load ptr, ptr %2, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 0
  %32 = load i32, ptr %31, align 8
  call void @save(ptr noundef %29, i32 noundef %32)
  %33 = load ptr, ptr %2, align 8
  %34 = getelementptr inbounds %struct.LexState, ptr %33, i32 0, i32 7
  %35 = load ptr, ptr %34, align 8
  %36 = getelementptr inbounds %struct.Zio, ptr %35, i32 0, i32 0
  %37 = load i64, ptr %36, align 8
  %38 = add i64 %37, -1
  store i64 %38, ptr %36, align 8
  %39 = icmp ugt i64 %37, 0
  br i1 %39, label %40, label %49

40:                                               ; preds = %21
  %41 = load ptr, ptr %2, align 8
  %42 = getelementptr inbounds %struct.LexState, ptr %41, i32 0, i32 7
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %struct.Zio, ptr %43, i32 0, i32 1
  %45 = load ptr, ptr %44, align 8
  %46 = getelementptr inbounds i8, ptr %45, i32 1
  store ptr %46, ptr %44, align 8
  %47 = load i8, ptr %45, align 1
  %48 = zext i8 %47 to i32
  br label %54

49:                                               ; preds = %21
  %50 = load ptr, ptr %2, align 8
  %51 = getelementptr inbounds %struct.LexState, ptr %50, i32 0, i32 7
  %52 = load ptr, ptr %51, align 8
  %53 = call i32 @luaZ_fill(ptr noundef %52)
  br label %54

54:                                               ; preds = %49, %40
  %55 = phi i32 [ %48, %40 ], [ %53, %49 ]
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds %struct.LexState, ptr %56, i32 0, i32 0
  store i32 %55, ptr %57, align 8
  br label %58

58:                                               ; preds = %54
  %59 = load i32, ptr %3, align 4
  %60 = add nsw i32 %59, 1
  store i32 %60, ptr %3, align 4
  br label %5, !llvm.loop !14

61:                                               ; preds = %19
  %62 = load ptr, ptr %2, align 8
  %63 = load i32, ptr %4, align 4
  %64 = icmp sle i32 %63, 255
  %65 = zext i1 %64 to i32
  call void @esccheck(ptr noundef %62, i32 noundef %65, ptr noundef @.str.54)
  %66 = load i32, ptr %3, align 4
  %67 = sext i32 %66 to i64
  %68 = load ptr, ptr %2, align 8
  %69 = getelementptr inbounds %struct.LexState, ptr %68, i32 0, i32 8
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %struct.Mbuffer, ptr %70, i32 0, i32 1
  %72 = load i64, ptr %71, align 8
  %73 = sub i64 %72, %67
  store i64 %73, ptr %71, align 8
  %74 = load i32, ptr %4, align 4
  ret i32 %74
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @gethexa(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.LexState, ptr %4, i32 0, i32 0
  %6 = load i32, ptr %5, align 8
  call void @save(ptr noundef %3, i32 noundef %6)
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.LexState, ptr %7, i32 0, i32 7
  %9 = load ptr, ptr %8, align 8
  %10 = getelementptr inbounds %struct.Zio, ptr %9, i32 0, i32 0
  %11 = load i64, ptr %10, align 8
  %12 = add i64 %11, -1
  store i64 %12, ptr %10, align 8
  %13 = icmp ugt i64 %11, 0
  br i1 %13, label %14, label %23

14:                                               ; preds = %1
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 7
  %17 = load ptr, ptr %16, align 8
  %18 = getelementptr inbounds %struct.Zio, ptr %17, i32 0, i32 1
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds i8, ptr %19, i32 1
  store ptr %20, ptr %18, align 8
  %21 = load i8, ptr %19, align 1
  %22 = zext i8 %21 to i32
  br label %28

23:                                               ; preds = %1
  %24 = load ptr, ptr %2, align 8
  %25 = getelementptr inbounds %struct.LexState, ptr %24, i32 0, i32 7
  %26 = load ptr, ptr %25, align 8
  %27 = call i32 @luaZ_fill(ptr noundef %26)
  br label %28

28:                                               ; preds = %23, %14
  %29 = phi i32 [ %22, %14 ], [ %27, %23 ]
  %30 = load ptr, ptr %2, align 8
  %31 = getelementptr inbounds %struct.LexState, ptr %30, i32 0, i32 0
  store i32 %29, ptr %31, align 8
  %32 = load ptr, ptr %2, align 8
  %33 = load ptr, ptr %2, align 8
  %34 = getelementptr inbounds %struct.LexState, ptr %33, i32 0, i32 0
  %35 = load i32, ptr %34, align 8
  %36 = add nsw i32 %35, 1
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %37
  %39 = load i8, ptr %38, align 1
  %40 = zext i8 %39 to i32
  %41 = and i32 %40, 16
  call void @esccheck(ptr noundef %32, i32 noundef %41, ptr noundef @.str.50)
  %42 = load ptr, ptr %2, align 8
  %43 = getelementptr inbounds %struct.LexState, ptr %42, i32 0, i32 0
  %44 = load i32, ptr %43, align 8
  %45 = call i32 @luaO_hexavalue(i32 noundef %44)
  ret i32 %45
}

declare hidden i32 @luaO_hexavalue(i32 noundef) #1

declare hidden i32 @luaO_utf8esc(ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @readutf8esc(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  store i32 4, ptr %4, align 4
  %5 = load ptr, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  call void @save(ptr noundef %5, i32 noundef %8)
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.LexState, ptr %9, i32 0, i32 7
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.Zio, ptr %11, i32 0, i32 0
  %13 = load i64, ptr %12, align 8
  %14 = add i64 %13, -1
  store i64 %14, ptr %12, align 8
  %15 = icmp ugt i64 %13, 0
  br i1 %15, label %16, label %25

16:                                               ; preds = %1
  %17 = load ptr, ptr %2, align 8
  %18 = getelementptr inbounds %struct.LexState, ptr %17, i32 0, i32 7
  %19 = load ptr, ptr %18, align 8
  %20 = getelementptr inbounds %struct.Zio, ptr %19, i32 0, i32 1
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds i8, ptr %21, i32 1
  store ptr %22, ptr %20, align 8
  %23 = load i8, ptr %21, align 1
  %24 = zext i8 %23 to i32
  br label %30

25:                                               ; preds = %1
  %26 = load ptr, ptr %2, align 8
  %27 = getelementptr inbounds %struct.LexState, ptr %26, i32 0, i32 7
  %28 = load ptr, ptr %27, align 8
  %29 = call i32 @luaZ_fill(ptr noundef %28)
  br label %30

30:                                               ; preds = %25, %16
  %31 = phi i32 [ %24, %16 ], [ %29, %25 ]
  %32 = load ptr, ptr %2, align 8
  %33 = getelementptr inbounds %struct.LexState, ptr %32, i32 0, i32 0
  store i32 %31, ptr %33, align 8
  %34 = load ptr, ptr %2, align 8
  %35 = load ptr, ptr %2, align 8
  %36 = getelementptr inbounds %struct.LexState, ptr %35, i32 0, i32 0
  %37 = load i32, ptr %36, align 8
  %38 = icmp eq i32 %37, 123
  %39 = zext i1 %38 to i32
  call void @esccheck(ptr noundef %34, i32 noundef %39, ptr noundef @.str.51)
  %40 = load ptr, ptr %2, align 8
  %41 = call i32 @gethexa(ptr noundef %40)
  %42 = sext i32 %41 to i64
  store i64 %42, ptr %3, align 8
  br label %43

43:                                               ; preds = %83, %30
  %44 = load ptr, ptr %2, align 8
  %45 = load ptr, ptr %2, align 8
  %46 = getelementptr inbounds %struct.LexState, ptr %45, i32 0, i32 0
  %47 = load i32, ptr %46, align 8
  call void @save(ptr noundef %44, i32 noundef %47)
  %48 = load ptr, ptr %2, align 8
  %49 = getelementptr inbounds %struct.LexState, ptr %48, i32 0, i32 7
  %50 = load ptr, ptr %49, align 8
  %51 = getelementptr inbounds %struct.Zio, ptr %50, i32 0, i32 0
  %52 = load i64, ptr %51, align 8
  %53 = add i64 %52, -1
  store i64 %53, ptr %51, align 8
  %54 = icmp ugt i64 %52, 0
  br i1 %54, label %55, label %64

55:                                               ; preds = %43
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds %struct.LexState, ptr %56, i32 0, i32 7
  %58 = load ptr, ptr %57, align 8
  %59 = getelementptr inbounds %struct.Zio, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds i8, ptr %60, i32 1
  store ptr %61, ptr %59, align 8
  %62 = load i8, ptr %60, align 1
  %63 = zext i8 %62 to i32
  br label %69

64:                                               ; preds = %43
  %65 = load ptr, ptr %2, align 8
  %66 = getelementptr inbounds %struct.LexState, ptr %65, i32 0, i32 7
  %67 = load ptr, ptr %66, align 8
  %68 = call i32 @luaZ_fill(ptr noundef %67)
  br label %69

69:                                               ; preds = %64, %55
  %70 = phi i32 [ %63, %55 ], [ %68, %64 ]
  %71 = load ptr, ptr %2, align 8
  %72 = getelementptr inbounds %struct.LexState, ptr %71, i32 0, i32 0
  store i32 %70, ptr %72, align 8
  %73 = load ptr, ptr %2, align 8
  %74 = getelementptr inbounds %struct.LexState, ptr %73, i32 0, i32 0
  %75 = load i32, ptr %74, align 8
  %76 = add nsw i32 %75, 1
  %77 = sext i32 %76 to i64
  %78 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %77
  %79 = load i8, ptr %78, align 1
  %80 = zext i8 %79 to i32
  %81 = and i32 %80, 16
  %82 = icmp ne i32 %81, 0
  br i1 %82, label %83, label %98

83:                                               ; preds = %69
  %84 = load i32, ptr %4, align 4
  %85 = add nsw i32 %84, 1
  store i32 %85, ptr %4, align 4
  %86 = load ptr, ptr %2, align 8
  %87 = load i64, ptr %3, align 8
  %88 = icmp ule i64 %87, 134217727
  %89 = zext i1 %88 to i32
  call void @esccheck(ptr noundef %86, i32 noundef %89, ptr noundef @.str.52)
  %90 = load i64, ptr %3, align 8
  %91 = shl i64 %90, 4
  %92 = load ptr, ptr %2, align 8
  %93 = getelementptr inbounds %struct.LexState, ptr %92, i32 0, i32 0
  %94 = load i32, ptr %93, align 8
  %95 = call i32 @luaO_hexavalue(i32 noundef %94)
  %96 = sext i32 %95 to i64
  %97 = add i64 %91, %96
  store i64 %97, ptr %3, align 8
  br label %43, !llvm.loop !15

98:                                               ; preds = %69
  %99 = load ptr, ptr %2, align 8
  %100 = load ptr, ptr %2, align 8
  %101 = getelementptr inbounds %struct.LexState, ptr %100, i32 0, i32 0
  %102 = load i32, ptr %101, align 8
  %103 = icmp eq i32 %102, 125
  %104 = zext i1 %103 to i32
  call void @esccheck(ptr noundef %99, i32 noundef %104, ptr noundef @.str.53)
  %105 = load ptr, ptr %2, align 8
  %106 = getelementptr inbounds %struct.LexState, ptr %105, i32 0, i32 7
  %107 = load ptr, ptr %106, align 8
  %108 = getelementptr inbounds %struct.Zio, ptr %107, i32 0, i32 0
  %109 = load i64, ptr %108, align 8
  %110 = add i64 %109, -1
  store i64 %110, ptr %108, align 8
  %111 = icmp ugt i64 %109, 0
  br i1 %111, label %112, label %121

112:                                              ; preds = %98
  %113 = load ptr, ptr %2, align 8
  %114 = getelementptr inbounds %struct.LexState, ptr %113, i32 0, i32 7
  %115 = load ptr, ptr %114, align 8
  %116 = getelementptr inbounds %struct.Zio, ptr %115, i32 0, i32 1
  %117 = load ptr, ptr %116, align 8
  %118 = getelementptr inbounds i8, ptr %117, i32 1
  store ptr %118, ptr %116, align 8
  %119 = load i8, ptr %117, align 1
  %120 = zext i8 %119 to i32
  br label %126

121:                                              ; preds = %98
  %122 = load ptr, ptr %2, align 8
  %123 = getelementptr inbounds %struct.LexState, ptr %122, i32 0, i32 7
  %124 = load ptr, ptr %123, align 8
  %125 = call i32 @luaZ_fill(ptr noundef %124)
  br label %126

126:                                              ; preds = %121, %112
  %127 = phi i32 [ %120, %112 ], [ %125, %121 ]
  %128 = load ptr, ptr %2, align 8
  %129 = getelementptr inbounds %struct.LexState, ptr %128, i32 0, i32 0
  store i32 %127, ptr %129, align 8
  %130 = load i32, ptr %4, align 4
  %131 = sext i32 %130 to i64
  %132 = load ptr, ptr %2, align 8
  %133 = getelementptr inbounds %struct.LexState, ptr %132, i32 0, i32 8
  %134 = load ptr, ptr %133, align 8
  %135 = getelementptr inbounds %struct.Mbuffer, ptr %134, i32 0, i32 1
  %136 = load i64, ptr %135, align 8
  %137 = sub i64 %136, %131
  store i64 %137, ptr %135, align 8
  %138 = load i64, ptr %3, align 8
  ret i64 %138
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @check_next2(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds %struct.LexState, ptr %6, i32 0, i32 0
  %8 = load i32, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = getelementptr inbounds i8, ptr %9, i64 0
  %11 = load i8, ptr %10, align 1
  %12 = sext i8 %11 to i32
  %13 = icmp eq i32 %8, %12
  br i1 %13, label %23, label %14

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.LexState, ptr %15, i32 0, i32 0
  %17 = load i32, ptr %16, align 8
  %18 = load ptr, ptr %5, align 8
  %19 = getelementptr inbounds i8, ptr %18, i64 1
  %20 = load i8, ptr %19, align 1
  %21 = sext i8 %20 to i32
  %22 = icmp eq i32 %17, %21
  br i1 %22, label %23, label %53

23:                                               ; preds = %14, %2
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.LexState, ptr %25, i32 0, i32 0
  %27 = load i32, ptr %26, align 8
  call void @save(ptr noundef %24, i32 noundef %27)
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.LexState, ptr %28, i32 0, i32 7
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %struct.Zio, ptr %30, i32 0, i32 0
  %32 = load i64, ptr %31, align 8
  %33 = add i64 %32, -1
  store i64 %33, ptr %31, align 8
  %34 = icmp ugt i64 %32, 0
  br i1 %34, label %35, label %44

35:                                               ; preds = %23
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.LexState, ptr %36, i32 0, i32 7
  %38 = load ptr, ptr %37, align 8
  %39 = getelementptr inbounds %struct.Zio, ptr %38, i32 0, i32 1
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds i8, ptr %40, i32 1
  store ptr %41, ptr %39, align 8
  %42 = load i8, ptr %40, align 1
  %43 = zext i8 %42 to i32
  br label %49

44:                                               ; preds = %23
  %45 = load ptr, ptr %4, align 8
  %46 = getelementptr inbounds %struct.LexState, ptr %45, i32 0, i32 7
  %47 = load ptr, ptr %46, align 8
  %48 = call i32 @luaZ_fill(ptr noundef %47)
  br label %49

49:                                               ; preds = %44, %35
  %50 = phi i32 [ %43, %35 ], [ %48, %44 ]
  %51 = load ptr, ptr %4, align 8
  %52 = getelementptr inbounds %struct.LexState, ptr %51, i32 0, i32 0
  store i32 %50, ptr %52, align 8
  store i32 1, ptr %3, align 4
  br label %54

53:                                               ; preds = %14
  store i32 0, ptr %3, align 4
  br label %54

54:                                               ; preds = %53, %49
  %55 = load i32, ptr %3, align 4
  ret i32 %55
}

declare hidden i64 @luaO_str2num(ptr noundef, ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { noreturn }

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
