; ModuleID = 'luac.c'
source_filename = "luac.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon, %union.anon.1, i16, i16 }
%union.anon = type { %struct.anon.0 }
%struct.anon.0 = type { ptr, i64, i64 }
%union.anon.1 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%union.StackValue = type { %struct.TValue }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon.4, [1 x i8] }
%union.anon.4 = type { i64 }
%struct.LocVar = type { ptr, i32, i32 }

@.str = private unnamed_addr constant [21 x i8] c"no input files given\00", align 1
@.str.1 = private unnamed_addr constant [39 x i8] c"cannot create state: not enough memory\00", align 1
@progname = internal global ptr @.str.12, align 8
@.str.2 = private unnamed_addr constant [3 x i8] c"--\00", align 1
@.str.3 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@.str.4 = private unnamed_addr constant [3 x i8] c"-l\00", align 1
@listing = internal global i32 0, align 4
@.str.5 = private unnamed_addr constant [3 x i8] c"-o\00", align 1
@output = internal global ptr @Output, align 8
@.str.6 = private unnamed_addr constant [20 x i8] c"'-o' needs argument\00", align 1
@.str.7 = private unnamed_addr constant [3 x i8] c"-p\00", align 1
@dumping = internal global i32 1, align 4
@.str.8 = private unnamed_addr constant [3 x i8] c"-s\00", align 1
@stripping = internal global i32 0, align 4
@.str.9 = private unnamed_addr constant [3 x i8] c"-v\00", align 1
@Output = internal global [9 x i8] c"luac.out\00", align 1
@.str.10 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.str.11 = private unnamed_addr constant [52 x i8] c"Lua 5.4.7  Copyright (C) 1994-2024 Lua.org, PUC-Rio\00", align 1
@.str.12 = private unnamed_addr constant [5 x i8] c"luac\00", align 1
@stderr = external global ptr, align 8
@.str.13 = private unnamed_addr constant [30 x i8] c"%s: unrecognized option '%s'\0A\00", align 1
@.str.14 = private unnamed_addr constant [8 x i8] c"%s: %s\0A\00", align 1
@.str.15 = private unnamed_addr constant [329 x i8] c"usage: %s [options] [filenames]\0AAvailable options are:\0A  -l       list (use -l -l for full listing)\0A  -o name  output to file 'name' (default is \22%s\22)\0A  -p       parse only\0A  -s       strip debug information\0A  -v       show version information\0A  --       stop handling options\0A  -        stop handling options and process stdin\0A\00", align 1
@tmname = internal global ptr null, align 8
@.str.16 = private unnamed_addr constant [21 x i8] c"too many input files\00", align 1
@stdout = external global ptr, align 8
@.str.17 = private unnamed_addr constant [3 x i8] c"wb\00", align 1
@.str.18 = private unnamed_addr constant [5 x i8] c"open\00", align 1
@.str.19 = private unnamed_addr constant [6 x i8] c"write\00", align 1
@.str.20 = private unnamed_addr constant [6 x i8] c"close\00", align 1
@.str.21 = private unnamed_addr constant [8 x i8] c"=(luac)\00", align 1
@.str.22 = private unnamed_addr constant [20 x i8] c"(function()end)();\0A\00", align 1
@.str.23 = private unnamed_addr constant [3 x i8] c"=?\00", align 1
@.str.24 = private unnamed_addr constant [5 x i8] c"\1BLua\00", align 1
@.str.25 = private unnamed_addr constant [10 x i8] c"(bstring)\00", align 1
@.str.26 = private unnamed_addr constant [9 x i8] c"(string)\00", align 1
@.str.27 = private unnamed_addr constant [41 x i8] c"\0A%s <%s:%d,%d> (%d instruction%s at %p)\0A\00", align 1
@.str.28 = private unnamed_addr constant [5 x i8] c"main\00", align 1
@.str.29 = private unnamed_addr constant [9 x i8] c"function\00", align 1
@.str.30 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@.str.31 = private unnamed_addr constant [2 x i8] c"s\00", align 1
@.str.32 = private unnamed_addr constant [40 x i8] c"%d%s param%s, %d slot%s, %d upvalue%s, \00", align 1
@.str.33 = private unnamed_addr constant [2 x i8] c"+\00", align 1
@.str.34 = private unnamed_addr constant [42 x i8] c"%d local%s, %d constant%s, %d function%s\0A\00", align 1
@.str.35 = private unnamed_addr constant [5 x i8] c"\09%d\09\00", align 1
@.str.36 = private unnamed_addr constant [6 x i8] c"[%d]\09\00", align 1
@.str.37 = private unnamed_addr constant [5 x i8] c"[-]\09\00", align 1
@.str.38 = private unnamed_addr constant [6 x i8] c"%-9s\09\00", align 1
@opnames = internal constant [84 x ptr] [ptr @.str.61, ptr @.str.62, ptr @.str.63, ptr @.str.64, ptr @.str.65, ptr @.str.66, ptr @.str.67, ptr @.str.68, ptr @.str.69, ptr @.str.70, ptr @.str.71, ptr @.str.72, ptr @.str.73, ptr @.str.74, ptr @.str.75, ptr @.str.76, ptr @.str.77, ptr @.str.78, ptr @.str.79, ptr @.str.80, ptr @.str.81, ptr @.str.82, ptr @.str.83, ptr @.str.84, ptr @.str.85, ptr @.str.86, ptr @.str.87, ptr @.str.88, ptr @.str.89, ptr @.str.90, ptr @.str.91, ptr @.str.92, ptr @.str.93, ptr @.str.94, ptr @.str.95, ptr @.str.96, ptr @.str.97, ptr @.str.98, ptr @.str.99, ptr @.str.100, ptr @.str.101, ptr @.str.102, ptr @.str.103, ptr @.str.104, ptr @.str.105, ptr @.str.106, ptr @.str.107, ptr @.str.108, ptr @.str.109, ptr @.str.110, ptr @.str.111, ptr @.str.112, ptr @.str.113, ptr @.str.114, ptr @.str.115, ptr @.str.116, ptr @.str.117, ptr @.str.118, ptr @.str.119, ptr @.str.120, ptr @.str.121, ptr @.str.122, ptr @.str.123, ptr @.str.124, ptr @.str.125, ptr @.str.126, ptr @.str.127, ptr @.str.128, ptr @.str.129, ptr @.str.130, ptr @.str.131, ptr @.str.132, ptr @.str.133, ptr @.str.134, ptr @.str.135, ptr @.str.136, ptr @.str.137, ptr @.str.138, ptr @.str.139, ptr @.str.140, ptr @.str.141, ptr @.str.142, ptr @.str.143, ptr null], align 16
@.str.39 = private unnamed_addr constant [6 x i8] c"%d %d\00", align 1
@.str.40 = private unnamed_addr constant [4 x i8] c"\09; \00", align 1
@.str.41 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@.str.42 = private unnamed_addr constant [10 x i8] c"\09; %d out\00", align 1
@.str.43 = private unnamed_addr constant [6 x i8] c"\09; %s\00", align 1
@.str.44 = private unnamed_addr constant [9 x i8] c"%d %d %d\00", align 1
@.str.45 = private unnamed_addr constant [2 x i8] c" \00", align 1
@.str.46 = private unnamed_addr constant [11 x i8] c"%d %d %d%s\00", align 1
@.str.47 = private unnamed_addr constant [2 x i8] c"k\00", align 1
@.str.48 = private unnamed_addr constant [6 x i8] c"\09; %d\00", align 1
@.str.49 = private unnamed_addr constant [12 x i8] c"%d %d %d %d\00", align 1
@.str.50 = private unnamed_addr constant [6 x i8] c" flip\00", align 1
@.str.51 = private unnamed_addr constant [7 x i8] c"\09; %s \00", align 1
@.str.52 = private unnamed_addr constant [9 x i8] c"\09; to %d\00", align 1
@.str.53 = private unnamed_addr constant [8 x i8] c"all in \00", align 1
@.str.54 = private unnamed_addr constant [7 x i8] c"%d in \00", align 1
@.str.55 = private unnamed_addr constant [8 x i8] c"all out\00", align 1
@.str.56 = private unnamed_addr constant [7 x i8] c"%d out\00", align 1
@.str.57 = private unnamed_addr constant [9 x i8] c"\09; %d in\00", align 1
@.str.58 = private unnamed_addr constant [14 x i8] c"\09; exit to %d\00", align 1
@.str.59 = private unnamed_addr constant [6 x i8] c"\09; %p\00", align 1
@.str.60 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@.str.61 = private unnamed_addr constant [5 x i8] c"MOVE\00", align 1
@.str.62 = private unnamed_addr constant [6 x i8] c"LOADI\00", align 1
@.str.63 = private unnamed_addr constant [6 x i8] c"LOADF\00", align 1
@.str.64 = private unnamed_addr constant [6 x i8] c"LOADK\00", align 1
@.str.65 = private unnamed_addr constant [7 x i8] c"LOADKX\00", align 1
@.str.66 = private unnamed_addr constant [10 x i8] c"LOADFALSE\00", align 1
@.str.67 = private unnamed_addr constant [11 x i8] c"LFALSESKIP\00", align 1
@.str.68 = private unnamed_addr constant [9 x i8] c"LOADTRUE\00", align 1
@.str.69 = private unnamed_addr constant [8 x i8] c"LOADNIL\00", align 1
@.str.70 = private unnamed_addr constant [9 x i8] c"GETUPVAL\00", align 1
@.str.71 = private unnamed_addr constant [9 x i8] c"SETUPVAL\00", align 1
@.str.72 = private unnamed_addr constant [9 x i8] c"GETTABUP\00", align 1
@.str.73 = private unnamed_addr constant [9 x i8] c"GETTABLE\00", align 1
@.str.74 = private unnamed_addr constant [5 x i8] c"GETI\00", align 1
@.str.75 = private unnamed_addr constant [9 x i8] c"GETFIELD\00", align 1
@.str.76 = private unnamed_addr constant [9 x i8] c"SETTABUP\00", align 1
@.str.77 = private unnamed_addr constant [9 x i8] c"SETTABLE\00", align 1
@.str.78 = private unnamed_addr constant [5 x i8] c"SETI\00", align 1
@.str.79 = private unnamed_addr constant [9 x i8] c"SETFIELD\00", align 1
@.str.80 = private unnamed_addr constant [9 x i8] c"NEWTABLE\00", align 1
@.str.81 = private unnamed_addr constant [5 x i8] c"SELF\00", align 1
@.str.82 = private unnamed_addr constant [5 x i8] c"ADDI\00", align 1
@.str.83 = private unnamed_addr constant [5 x i8] c"ADDK\00", align 1
@.str.84 = private unnamed_addr constant [5 x i8] c"SUBK\00", align 1
@.str.85 = private unnamed_addr constant [5 x i8] c"MULK\00", align 1
@.str.86 = private unnamed_addr constant [5 x i8] c"MODK\00", align 1
@.str.87 = private unnamed_addr constant [5 x i8] c"POWK\00", align 1
@.str.88 = private unnamed_addr constant [5 x i8] c"DIVK\00", align 1
@.str.89 = private unnamed_addr constant [6 x i8] c"IDIVK\00", align 1
@.str.90 = private unnamed_addr constant [6 x i8] c"BANDK\00", align 1
@.str.91 = private unnamed_addr constant [5 x i8] c"BORK\00", align 1
@.str.92 = private unnamed_addr constant [6 x i8] c"BXORK\00", align 1
@.str.93 = private unnamed_addr constant [5 x i8] c"SHRI\00", align 1
@.str.94 = private unnamed_addr constant [5 x i8] c"SHLI\00", align 1
@.str.95 = private unnamed_addr constant [4 x i8] c"ADD\00", align 1
@.str.96 = private unnamed_addr constant [4 x i8] c"SUB\00", align 1
@.str.97 = private unnamed_addr constant [4 x i8] c"MUL\00", align 1
@.str.98 = private unnamed_addr constant [4 x i8] c"MOD\00", align 1
@.str.99 = private unnamed_addr constant [4 x i8] c"POW\00", align 1
@.str.100 = private unnamed_addr constant [4 x i8] c"DIV\00", align 1
@.str.101 = private unnamed_addr constant [5 x i8] c"IDIV\00", align 1
@.str.102 = private unnamed_addr constant [5 x i8] c"BAND\00", align 1
@.str.103 = private unnamed_addr constant [4 x i8] c"BOR\00", align 1
@.str.104 = private unnamed_addr constant [5 x i8] c"BXOR\00", align 1
@.str.105 = private unnamed_addr constant [4 x i8] c"SHL\00", align 1
@.str.106 = private unnamed_addr constant [4 x i8] c"SHR\00", align 1
@.str.107 = private unnamed_addr constant [6 x i8] c"MMBIN\00", align 1
@.str.108 = private unnamed_addr constant [7 x i8] c"MMBINI\00", align 1
@.str.109 = private unnamed_addr constant [7 x i8] c"MMBINK\00", align 1
@.str.110 = private unnamed_addr constant [4 x i8] c"UNM\00", align 1
@.str.111 = private unnamed_addr constant [5 x i8] c"BNOT\00", align 1
@.str.112 = private unnamed_addr constant [4 x i8] c"NOT\00", align 1
@.str.113 = private unnamed_addr constant [4 x i8] c"LEN\00", align 1
@.str.114 = private unnamed_addr constant [7 x i8] c"CONCAT\00", align 1
@.str.115 = private unnamed_addr constant [6 x i8] c"CLOSE\00", align 1
@.str.116 = private unnamed_addr constant [4 x i8] c"TBC\00", align 1
@.str.117 = private unnamed_addr constant [4 x i8] c"JMP\00", align 1
@.str.118 = private unnamed_addr constant [3 x i8] c"EQ\00", align 1
@.str.119 = private unnamed_addr constant [3 x i8] c"LT\00", align 1
@.str.120 = private unnamed_addr constant [3 x i8] c"LE\00", align 1
@.str.121 = private unnamed_addr constant [4 x i8] c"EQK\00", align 1
@.str.122 = private unnamed_addr constant [4 x i8] c"EQI\00", align 1
@.str.123 = private unnamed_addr constant [4 x i8] c"LTI\00", align 1
@.str.124 = private unnamed_addr constant [4 x i8] c"LEI\00", align 1
@.str.125 = private unnamed_addr constant [4 x i8] c"GTI\00", align 1
@.str.126 = private unnamed_addr constant [4 x i8] c"GEI\00", align 1
@.str.127 = private unnamed_addr constant [5 x i8] c"TEST\00", align 1
@.str.128 = private unnamed_addr constant [8 x i8] c"TESTSET\00", align 1
@.str.129 = private unnamed_addr constant [5 x i8] c"CALL\00", align 1
@.str.130 = private unnamed_addr constant [9 x i8] c"TAILCALL\00", align 1
@.str.131 = private unnamed_addr constant [7 x i8] c"RETURN\00", align 1
@.str.132 = private unnamed_addr constant [8 x i8] c"RETURN0\00", align 1
@.str.133 = private unnamed_addr constant [8 x i8] c"RETURN1\00", align 1
@.str.134 = private unnamed_addr constant [8 x i8] c"FORLOOP\00", align 1
@.str.135 = private unnamed_addr constant [8 x i8] c"FORPREP\00", align 1
@.str.136 = private unnamed_addr constant [9 x i8] c"TFORPREP\00", align 1
@.str.137 = private unnamed_addr constant [9 x i8] c"TFORCALL\00", align 1
@.str.138 = private unnamed_addr constant [9 x i8] c"TFORLOOP\00", align 1
@.str.139 = private unnamed_addr constant [8 x i8] c"SETLIST\00", align 1
@.str.140 = private unnamed_addr constant [8 x i8] c"CLOSURE\00", align 1
@.str.141 = private unnamed_addr constant [7 x i8] c"VARARG\00", align 1
@.str.142 = private unnamed_addr constant [11 x i8] c"VARARGPREP\00", align 1
@.str.143 = private unnamed_addr constant [9 x i8] c"EXTRAARG\00", align 1
@.str.144 = private unnamed_addr constant [4 x i8] c"nil\00", align 1
@.str.145 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.146 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.147 = private unnamed_addr constant [6 x i8] c"%.14g\00", align 1
@.str.148 = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.str.149 = private unnamed_addr constant [12 x i8] c"-0123456789\00", align 1
@.str.150 = private unnamed_addr constant [3 x i8] c".0\00", align 1
@.str.151 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.str.152 = private unnamed_addr constant [4 x i8] c"?%d\00", align 1
@.str.153 = private unnamed_addr constant [2 x i8] c"\22\00", align 1
@.str.154 = private unnamed_addr constant [3 x i8] c"\\\22\00", align 1
@.str.155 = private unnamed_addr constant [3 x i8] c"\\\\\00", align 1
@.str.156 = private unnamed_addr constant [3 x i8] c"\\a\00", align 1
@.str.157 = private unnamed_addr constant [3 x i8] c"\\b\00", align 1
@.str.158 = private unnamed_addr constant [3 x i8] c"\\f\00", align 1
@.str.159 = private unnamed_addr constant [3 x i8] c"\\n\00", align 1
@.str.160 = private unnamed_addr constant [3 x i8] c"\\r\00", align 1
@.str.161 = private unnamed_addr constant [3 x i8] c"\\t\00", align 1
@.str.162 = private unnamed_addr constant [3 x i8] c"\\v\00", align 1
@.str.163 = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@.str.164 = private unnamed_addr constant [6 x i8] c"\\%03d\00", align 1
@.str.165 = private unnamed_addr constant [24 x i8] c"constants (%d) for %p:\0A\00", align 1
@.str.166 = private unnamed_addr constant [21 x i8] c"locals (%d) for %p:\0A\00", align 1
@.str.167 = private unnamed_addr constant [14 x i8] c"\09%d\09%s\09%d\09%d\0A\00", align 1
@.str.168 = private unnamed_addr constant [23 x i8] c"upvalues (%d) for %p:\0A\00", align 1
@.str.169 = private unnamed_addr constant [2 x i8] c"N\00", align 1
@.str.170 = private unnamed_addr constant [2 x i8] c"B\00", align 1
@.str.171 = private unnamed_addr constant [2 x i8] c"F\00", align 1
@.str.172 = private unnamed_addr constant [2 x i8] c"I\00", align 1
@.str.173 = private unnamed_addr constant [2 x i8] c"S\00", align 1
@.str.174 = private unnamed_addr constant [2 x i8] c"\09\00", align 1
@.str.175 = private unnamed_addr constant [22 x i8] c"%s: cannot %s %s: %s\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  store ptr %1, ptr %5, align 8
  %8 = load i32, ptr %4, align 4
  %9 = load ptr, ptr %5, align 8
  %10 = call i32 @doargs(i32 noundef %8, ptr noundef %9)
  store i32 %10, ptr %7, align 4
  %11 = load i32, ptr %7, align 4
  %12 = load i32, ptr %4, align 4
  %13 = sub nsw i32 %12, %11
  store i32 %13, ptr %4, align 4
  %14 = load i32, ptr %7, align 4
  %15 = load ptr, ptr %5, align 8
  %16 = sext i32 %14 to i64
  %17 = getelementptr inbounds ptr, ptr %15, i64 %16
  store ptr %17, ptr %5, align 8
  %18 = load i32, ptr %4, align 4
  %19 = icmp sle i32 %18, 0
  br i1 %19, label %20, label %21

20:                                               ; preds = %2
  call void @usage(ptr noundef @.str)
  br label %21

21:                                               ; preds = %20, %2
  %22 = call ptr @luaL_newstate()
  store ptr %22, ptr %6, align 8
  %23 = load ptr, ptr %6, align 8
  %24 = icmp eq ptr %23, null
  br i1 %24, label %25, label %26

25:                                               ; preds = %21
  call void @fatal(ptr noundef @.str.1)
  br label %26

26:                                               ; preds = %25, %21
  %27 = load ptr, ptr %6, align 8
  call void @lua_pushcclosure(ptr noundef %27, ptr noundef @pmain, i32 noundef 0)
  %28 = load ptr, ptr %6, align 8
  %29 = load i32, ptr %4, align 4
  %30 = sext i32 %29 to i64
  call void @lua_pushinteger(ptr noundef %28, i64 noundef %30)
  %31 = load ptr, ptr %6, align 8
  %32 = load ptr, ptr %5, align 8
  call void @lua_pushlightuserdata(ptr noundef %31, ptr noundef %32)
  %33 = load ptr, ptr %6, align 8
  %34 = call i32 @lua_pcallk(ptr noundef %33, i32 noundef 2, i32 noundef 0, i32 noundef 0, i64 noundef 0, ptr noundef null)
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %36, label %39

36:                                               ; preds = %26
  %37 = load ptr, ptr %6, align 8
  %38 = call ptr @lua_tolstring(ptr noundef %37, i32 noundef -1, ptr noundef null)
  call void @fatal(ptr noundef %38)
  br label %39

39:                                               ; preds = %36, %26
  %40 = load ptr, ptr %6, align 8
  call void @lua_close(ptr noundef %40)
  ret i32 0
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @doargs(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  store ptr %1, ptr %4, align 8
  store i32 0, ptr %6, align 4
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds ptr, ptr %7, i64 0
  %9 = load ptr, ptr %8, align 8
  %10 = icmp ne ptr %9, null
  br i1 %10, label %11, label %22

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = getelementptr inbounds ptr, ptr %12, i64 0
  %14 = load ptr, ptr %13, align 8
  %15 = load i8, ptr %14, align 1
  %16 = sext i8 %15 to i32
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %18, label %22

18:                                               ; preds = %11
  %19 = load ptr, ptr %4, align 8
  %20 = getelementptr inbounds ptr, ptr %19, i64 0
  %21 = load ptr, ptr %20, align 8
  store ptr %21, ptr @progname, align 8
  br label %22

22:                                               ; preds = %18, %11, %2
  store i32 1, ptr %5, align 4
  br label %23

23:                                               ; preds = %161, %22
  %24 = load i32, ptr %5, align 4
  %25 = load i32, ptr %3, align 4
  %26 = icmp slt i32 %24, %25
  br i1 %26, label %27, label %164

27:                                               ; preds = %23
  %28 = load ptr, ptr %4, align 8
  %29 = load i32, ptr %5, align 4
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds ptr, ptr %28, i64 %30
  %32 = load ptr, ptr %31, align 8
  %33 = load i8, ptr %32, align 1
  %34 = sext i8 %33 to i32
  %35 = icmp ne i32 %34, 45
  br i1 %35, label %36, label %37

36:                                               ; preds = %27
  br label %164

37:                                               ; preds = %27
  %38 = load ptr, ptr %4, align 8
  %39 = load i32, ptr %5, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds ptr, ptr %38, i64 %40
  %42 = load ptr, ptr %41, align 8
  %43 = call i32 @strcmp(ptr noundef %42, ptr noundef @.str.2) #6
  %44 = icmp eq i32 %43, 0
  br i1 %44, label %45, label %54

45:                                               ; preds = %37
  %46 = load i32, ptr %5, align 4
  %47 = add nsw i32 %46, 1
  store i32 %47, ptr %5, align 4
  %48 = load i32, ptr %6, align 4
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %53

50:                                               ; preds = %45
  %51 = load i32, ptr %6, align 4
  %52 = add nsw i32 %51, 1
  store i32 %52, ptr %6, align 4
  br label %53

53:                                               ; preds = %50, %45
  br label %164

54:                                               ; preds = %37
  %55 = load ptr, ptr %4, align 8
  %56 = load i32, ptr %5, align 4
  %57 = sext i32 %56 to i64
  %58 = getelementptr inbounds ptr, ptr %55, i64 %57
  %59 = load ptr, ptr %58, align 8
  %60 = call i32 @strcmp(ptr noundef %59, ptr noundef @.str.3) #6
  %61 = icmp eq i32 %60, 0
  br i1 %61, label %62, label %63

62:                                               ; preds = %54
  br label %164

63:                                               ; preds = %54
  %64 = load ptr, ptr %4, align 8
  %65 = load i32, ptr %5, align 4
  %66 = sext i32 %65 to i64
  %67 = getelementptr inbounds ptr, ptr %64, i64 %66
  %68 = load ptr, ptr %67, align 8
  %69 = call i32 @strcmp(ptr noundef %68, ptr noundef @.str.4) #6
  %70 = icmp eq i32 %69, 0
  br i1 %70, label %71, label %74

71:                                               ; preds = %63
  %72 = load i32, ptr @listing, align 4
  %73 = add nsw i32 %72, 1
  store i32 %73, ptr @listing, align 4
  br label %157

74:                                               ; preds = %63
  %75 = load ptr, ptr %4, align 8
  %76 = load i32, ptr %5, align 4
  %77 = sext i32 %76 to i64
  %78 = getelementptr inbounds ptr, ptr %75, i64 %77
  %79 = load ptr, ptr %78, align 8
  %80 = call i32 @strcmp(ptr noundef %79, ptr noundef @.str.5) #6
  %81 = icmp eq i32 %80, 0
  br i1 %81, label %82, label %118

82:                                               ; preds = %74
  %83 = load ptr, ptr %4, align 8
  %84 = load i32, ptr %5, align 4
  %85 = add nsw i32 %84, 1
  store i32 %85, ptr %5, align 4
  %86 = sext i32 %85 to i64
  %87 = getelementptr inbounds ptr, ptr %83, i64 %86
  %88 = load ptr, ptr %87, align 8
  store ptr %88, ptr @output, align 8
  %89 = load ptr, ptr @output, align 8
  %90 = icmp eq ptr %89, null
  br i1 %90, label %107, label %91

91:                                               ; preds = %82
  %92 = load ptr, ptr @output, align 8
  %93 = load i8, ptr %92, align 1
  %94 = sext i8 %93 to i32
  %95 = icmp eq i32 %94, 0
  br i1 %95, label %107, label %96

96:                                               ; preds = %91
  %97 = load ptr, ptr @output, align 8
  %98 = load i8, ptr %97, align 1
  %99 = sext i8 %98 to i32
  %100 = icmp eq i32 %99, 45
  br i1 %100, label %101, label %108

101:                                              ; preds = %96
  %102 = load ptr, ptr @output, align 8
  %103 = getelementptr inbounds i8, ptr %102, i64 1
  %104 = load i8, ptr %103, align 1
  %105 = sext i8 %104 to i32
  %106 = icmp ne i32 %105, 0
  br i1 %106, label %107, label %108

107:                                              ; preds = %101, %91, %82
  call void @usage(ptr noundef @.str.6)
  br label %108

108:                                              ; preds = %107, %101, %96
  %109 = load ptr, ptr %4, align 8
  %110 = load i32, ptr %5, align 4
  %111 = sext i32 %110 to i64
  %112 = getelementptr inbounds ptr, ptr %109, i64 %111
  %113 = load ptr, ptr %112, align 8
  %114 = call i32 @strcmp(ptr noundef %113, ptr noundef @.str.3) #6
  %115 = icmp eq i32 %114, 0
  br i1 %115, label %116, label %117

116:                                              ; preds = %108
  store ptr null, ptr @output, align 8
  br label %117

117:                                              ; preds = %116, %108
  br label %156

118:                                              ; preds = %74
  %119 = load ptr, ptr %4, align 8
  %120 = load i32, ptr %5, align 4
  %121 = sext i32 %120 to i64
  %122 = getelementptr inbounds ptr, ptr %119, i64 %121
  %123 = load ptr, ptr %122, align 8
  %124 = call i32 @strcmp(ptr noundef %123, ptr noundef @.str.7) #6
  %125 = icmp eq i32 %124, 0
  br i1 %125, label %126, label %127

126:                                              ; preds = %118
  store i32 0, ptr @dumping, align 4
  br label %155

127:                                              ; preds = %118
  %128 = load ptr, ptr %4, align 8
  %129 = load i32, ptr %5, align 4
  %130 = sext i32 %129 to i64
  %131 = getelementptr inbounds ptr, ptr %128, i64 %130
  %132 = load ptr, ptr %131, align 8
  %133 = call i32 @strcmp(ptr noundef %132, ptr noundef @.str.8) #6
  %134 = icmp eq i32 %133, 0
  br i1 %134, label %135, label %136

135:                                              ; preds = %127
  store i32 1, ptr @stripping, align 4
  br label %154

136:                                              ; preds = %127
  %137 = load ptr, ptr %4, align 8
  %138 = load i32, ptr %5, align 4
  %139 = sext i32 %138 to i64
  %140 = getelementptr inbounds ptr, ptr %137, i64 %139
  %141 = load ptr, ptr %140, align 8
  %142 = call i32 @strcmp(ptr noundef %141, ptr noundef @.str.9) #6
  %143 = icmp eq i32 %142, 0
  br i1 %143, label %144, label %147

144:                                              ; preds = %136
  %145 = load i32, ptr %6, align 4
  %146 = add nsw i32 %145, 1
  store i32 %146, ptr %6, align 4
  br label %153

147:                                              ; preds = %136
  %148 = load ptr, ptr %4, align 8
  %149 = load i32, ptr %5, align 4
  %150 = sext i32 %149 to i64
  %151 = getelementptr inbounds ptr, ptr %148, i64 %150
  %152 = load ptr, ptr %151, align 8
  call void @usage(ptr noundef %152)
  br label %153

153:                                              ; preds = %147, %144
  br label %154

154:                                              ; preds = %153, %135
  br label %155

155:                                              ; preds = %154, %126
  br label %156

156:                                              ; preds = %155, %117
  br label %157

157:                                              ; preds = %156, %71
  br label %158

158:                                              ; preds = %157
  br label %159

159:                                              ; preds = %158
  br label %160

160:                                              ; preds = %159
  br label %161

161:                                              ; preds = %160
  %162 = load i32, ptr %5, align 4
  %163 = add nsw i32 %162, 1
  store i32 %163, ptr %5, align 4
  br label %23, !llvm.loop !6

164:                                              ; preds = %62, %53, %36, %23
  %165 = load i32, ptr %5, align 4
  %166 = load i32, ptr %3, align 4
  %167 = icmp eq i32 %165, %166
  br i1 %167, label %168, label %180

168:                                              ; preds = %164
  %169 = load i32, ptr @listing, align 4
  %170 = icmp ne i32 %169, 0
  br i1 %170, label %174, label %171

171:                                              ; preds = %168
  %172 = load i32, ptr @dumping, align 4
  %173 = icmp ne i32 %172, 0
  br i1 %173, label %180, label %174

174:                                              ; preds = %171, %168
  store i32 0, ptr @dumping, align 4
  %175 = load ptr, ptr %4, align 8
  %176 = load i32, ptr %5, align 4
  %177 = add nsw i32 %176, -1
  store i32 %177, ptr %5, align 4
  %178 = sext i32 %177 to i64
  %179 = getelementptr inbounds ptr, ptr %175, i64 %178
  store ptr @Output, ptr %179, align 8
  br label %180

180:                                              ; preds = %174, %171, %164
  %181 = load i32, ptr %6, align 4
  %182 = icmp ne i32 %181, 0
  br i1 %182, label %183, label %191

183:                                              ; preds = %180
  %184 = call i32 (ptr, ...) @printf(ptr noundef @.str.10, ptr noundef @.str.11)
  %185 = load i32, ptr %6, align 4
  %186 = load i32, ptr %3, align 4
  %187 = sub nsw i32 %186, 1
  %188 = icmp eq i32 %185, %187
  br i1 %188, label %189, label %190

189:                                              ; preds = %183
  call void @exit(i32 noundef 0) #7
  unreachable

190:                                              ; preds = %183
  br label %191

191:                                              ; preds = %190, %180
  %192 = load i32, ptr %5, align 4
  ret i32 %192
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @usage(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load i8, ptr %3, align 1
  %5 = sext i8 %4 to i32
  %6 = icmp eq i32 %5, 45
  br i1 %6, label %7, label %12

7:                                                ; preds = %1
  %8 = load ptr, ptr @stderr, align 8
  %9 = load ptr, ptr @progname, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %8, ptr noundef @.str.13, ptr noundef %9, ptr noundef %10)
  br label %17

12:                                               ; preds = %1
  %13 = load ptr, ptr @stderr, align 8
  %14 = load ptr, ptr @progname, align 8
  %15 = load ptr, ptr %2, align 8
  %16 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %13, ptr noundef @.str.14, ptr noundef %14, ptr noundef %15)
  br label %17

17:                                               ; preds = %12, %7
  %18 = load ptr, ptr @stderr, align 8
  %19 = load ptr, ptr @progname, align 8
  %20 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %18, ptr noundef @.str.15, ptr noundef %19, ptr noundef @Output)
  call void @exit(i32 noundef 1) #7
  unreachable
}

declare ptr @luaL_newstate() #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @fatal(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr @stderr, align 8
  %4 = load ptr, ptr @progname, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %3, ptr noundef @.str.14, ptr noundef %4, ptr noundef %5)
  call void @exit(i32 noundef 1) #7
  unreachable
}

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @pmain(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %9 = load ptr, ptr %2, align 8
  %10 = call i64 @lua_tointegerx(ptr noundef %9, i32 noundef 1, ptr noundef null)
  %11 = trunc i64 %10 to i32
  store i32 %11, ptr %3, align 4
  %12 = load ptr, ptr %2, align 8
  %13 = call ptr @lua_touserdata(ptr noundef %12, i32 noundef 2)
  store ptr %13, ptr %4, align 8
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 7
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.global_State, ptr %16, i32 0, i32 42
  %18 = getelementptr inbounds [25 x ptr], ptr %17, i64 0, i64 0
  store ptr %18, ptr @tmname, align 8
  %19 = load ptr, ptr %2, align 8
  %20 = load i32, ptr %3, align 4
  %21 = call i32 @lua_checkstack(ptr noundef %19, i32 noundef %20)
  %22 = icmp ne i32 %21, 0
  br i1 %22, label %24, label %23

23:                                               ; preds = %1
  call void @fatal(ptr noundef @.str.16)
  br label %24

24:                                               ; preds = %23, %1
  store i32 0, ptr %6, align 4
  br label %25

25:                                               ; preds = %54, %24
  %26 = load i32, ptr %6, align 4
  %27 = load i32, ptr %3, align 4
  %28 = icmp slt i32 %26, %27
  br i1 %28, label %29, label %57

29:                                               ; preds = %25
  %30 = load ptr, ptr %4, align 8
  %31 = load i32, ptr %6, align 4
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds ptr, ptr %30, i64 %32
  %34 = load ptr, ptr %33, align 8
  %35 = call i32 @strcmp(ptr noundef %34, ptr noundef @.str.3) #6
  %36 = icmp eq i32 %35, 0
  br i1 %36, label %37, label %38

37:                                               ; preds = %29
  br label %44

38:                                               ; preds = %29
  %39 = load ptr, ptr %4, align 8
  %40 = load i32, ptr %6, align 4
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds ptr, ptr %39, i64 %41
  %43 = load ptr, ptr %42, align 8
  br label %44

44:                                               ; preds = %38, %37
  %45 = phi ptr [ null, %37 ], [ %43, %38 ]
  store ptr %45, ptr %7, align 8
  %46 = load ptr, ptr %2, align 8
  %47 = load ptr, ptr %7, align 8
  %48 = call i32 @luaL_loadfilex(ptr noundef %46, ptr noundef %47, ptr noundef null)
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %53

50:                                               ; preds = %44
  %51 = load ptr, ptr %2, align 8
  %52 = call ptr @lua_tolstring(ptr noundef %51, i32 noundef -1, ptr noundef null)
  call void @fatal(ptr noundef %52)
  br label %53

53:                                               ; preds = %50, %44
  br label %54

54:                                               ; preds = %53
  %55 = load i32, ptr %6, align 4
  %56 = add nsw i32 %55, 1
  store i32 %56, ptr %6, align 4
  br label %25, !llvm.loop !8

57:                                               ; preds = %25
  %58 = load ptr, ptr %2, align 8
  %59 = load i32, ptr %3, align 4
  %60 = call ptr @combine(ptr noundef %58, i32 noundef %59)
  store ptr %60, ptr %5, align 8
  %61 = load i32, ptr @listing, align 4
  %62 = icmp ne i32 %61, 0
  br i1 %62, label %63, label %68

63:                                               ; preds = %57
  %64 = load ptr, ptr %5, align 8
  %65 = load i32, ptr @listing, align 4
  %66 = icmp sgt i32 %65, 1
  %67 = zext i1 %66 to i32
  call void @PrintFunction(ptr noundef %64, i32 noundef %67)
  br label %68

68:                                               ; preds = %63, %57
  %69 = load i32, ptr @dumping, align 4
  %70 = icmp ne i32 %69, 0
  br i1 %70, label %71, label %100

71:                                               ; preds = %68
  %72 = load ptr, ptr @output, align 8
  %73 = icmp eq ptr %72, null
  br i1 %73, label %74, label %76

74:                                               ; preds = %71
  %75 = load ptr, ptr @stdout, align 8
  br label %79

76:                                               ; preds = %71
  %77 = load ptr, ptr @output, align 8
  %78 = call noalias ptr @fopen64(ptr noundef %77, ptr noundef @.str.17)
  br label %79

79:                                               ; preds = %76, %74
  %80 = phi ptr [ %75, %74 ], [ %78, %76 ]
  store ptr %80, ptr %8, align 8
  %81 = load ptr, ptr %8, align 8
  %82 = icmp eq ptr %81, null
  br i1 %82, label %83, label %84

83:                                               ; preds = %79
  call void @cannot(ptr noundef @.str.18)
  br label %84

84:                                               ; preds = %83, %79
  %85 = load ptr, ptr %2, align 8
  %86 = load ptr, ptr %5, align 8
  %87 = load ptr, ptr %8, align 8
  %88 = load i32, ptr @stripping, align 4
  %89 = call i32 @luaU_dump(ptr noundef %85, ptr noundef %86, ptr noundef @writer, ptr noundef %87, i32 noundef %88)
  %90 = load ptr, ptr %8, align 8
  %91 = call i32 @ferror(ptr noundef %90) #8
  %92 = icmp ne i32 %91, 0
  br i1 %92, label %93, label %94

93:                                               ; preds = %84
  call void @cannot(ptr noundef @.str.19)
  br label %94

94:                                               ; preds = %93, %84
  %95 = load ptr, ptr %8, align 8
  %96 = call i32 @fclose(ptr noundef %95)
  %97 = icmp ne i32 %96, 0
  br i1 %97, label %98, label %99

98:                                               ; preds = %94
  call void @cannot(ptr noundef @.str.20)
  br label %99

99:                                               ; preds = %98, %94
  br label %100

100:                                              ; preds = %99, %68
  ret i32 0
}

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_pushlightuserdata(ptr noundef, ptr noundef) #1

declare i32 @lua_pcallk(ptr noundef, i32 noundef, i32 noundef, i32 noundef, i64 noundef, ptr noundef) #1

declare ptr @lua_tolstring(ptr noundef, i32 noundef, ptr noundef) #1

declare void @lua_close(ptr noundef) #1

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcmp(ptr noundef, ptr noundef) #2

declare i32 @printf(ptr noundef, ...) #1

; Function Attrs: noreturn nounwind
declare void @exit(i32 noundef) #3

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #1

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

declare ptr @lua_touserdata(ptr noundef, i32 noundef) #1

declare i32 @lua_checkstack(ptr noundef, i32 noundef) #1

declare i32 @luaL_loadfilex(ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @combine(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %8 = load i32, ptr %5, align 4
  %9 = icmp eq i32 %8, 1
  br i1 %9, label %10, label %19

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 6
  %13 = load ptr, ptr %12, align 8
  %14 = getelementptr inbounds %union.StackValue, ptr %13, i64 -1
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds %struct.LClosure, ptr %16, i32 0, i32 5
  %18 = load ptr, ptr %17, align 8
  store ptr %18, ptr %3, align 8
  br label %88

19:                                               ; preds = %2
  %20 = load i32, ptr %5, align 4
  store i32 %20, ptr %7, align 4
  %21 = load ptr, ptr %4, align 8
  %22 = call i32 @lua_load(ptr noundef %21, ptr noundef @reader, ptr noundef %7, ptr noundef @.str.21, ptr noundef null)
  %23 = icmp ne i32 %22, 0
  br i1 %23, label %24, label %27

24:                                               ; preds = %19
  %25 = load ptr, ptr %4, align 8
  %26 = call ptr @lua_tolstring(ptr noundef %25, i32 noundef -1, ptr noundef null)
  call void @fatal(ptr noundef %26)
  br label %27

27:                                               ; preds = %24, %19
  %28 = load ptr, ptr %4, align 8
  %29 = getelementptr inbounds %struct.lua_State, ptr %28, i32 0, i32 6
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds %union.StackValue, ptr %30, i64 -1
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds %struct.LClosure, ptr %33, i32 0, i32 5
  %35 = load ptr, ptr %34, align 8
  store ptr %35, ptr %6, align 8
  store i32 0, ptr %7, align 4
  br label %36

36:                                               ; preds = %83, %27
  %37 = load i32, ptr %7, align 4
  %38 = load i32, ptr %5, align 4
  %39 = icmp slt i32 %37, %38
  br i1 %39, label %40, label %86

40:                                               ; preds = %36
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.lua_State, ptr %41, i32 0, i32 6
  %43 = load ptr, ptr %42, align 8
  %44 = load i32, ptr %7, align 4
  %45 = load i32, ptr %5, align 4
  %46 = sub nsw i32 %44, %45
  %47 = sub nsw i32 %46, 1
  %48 = sext i32 %47 to i64
  %49 = getelementptr inbounds %union.StackValue, ptr %43, i64 %48
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 0
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.LClosure, ptr %51, i32 0, i32 5
  %53 = load ptr, ptr %52, align 8
  %54 = load ptr, ptr %6, align 8
  %55 = getelementptr inbounds %struct.Proto, ptr %54, i32 0, i32 17
  %56 = load ptr, ptr %55, align 8
  %57 = load i32, ptr %7, align 4
  %58 = sext i32 %57 to i64
  %59 = getelementptr inbounds ptr, ptr %56, i64 %58
  store ptr %53, ptr %59, align 8
  %60 = load ptr, ptr %6, align 8
  %61 = getelementptr inbounds %struct.Proto, ptr %60, i32 0, i32 17
  %62 = load ptr, ptr %61, align 8
  %63 = load i32, ptr %7, align 4
  %64 = sext i32 %63 to i64
  %65 = getelementptr inbounds ptr, ptr %62, i64 %64
  %66 = load ptr, ptr %65, align 8
  %67 = getelementptr inbounds %struct.Proto, ptr %66, i32 0, i32 6
  %68 = load i32, ptr %67, align 8
  %69 = icmp sgt i32 %68, 0
  br i1 %69, label %70, label %82

70:                                               ; preds = %40
  %71 = load ptr, ptr %6, align 8
  %72 = getelementptr inbounds %struct.Proto, ptr %71, i32 0, i32 17
  %73 = load ptr, ptr %72, align 8
  %74 = load i32, ptr %7, align 4
  %75 = sext i32 %74 to i64
  %76 = getelementptr inbounds ptr, ptr %73, i64 %75
  %77 = load ptr, ptr %76, align 8
  %78 = getelementptr inbounds %struct.Proto, ptr %77, i32 0, i32 18
  %79 = load ptr, ptr %78, align 8
  %80 = getelementptr inbounds %struct.Upvaldesc, ptr %79, i64 0
  %81 = getelementptr inbounds %struct.Upvaldesc, ptr %80, i32 0, i32 1
  store i8 0, ptr %81, align 8
  br label %82

82:                                               ; preds = %70, %40
  br label %83

83:                                               ; preds = %82
  %84 = load i32, ptr %7, align 4
  %85 = add nsw i32 %84, 1
  store i32 %85, ptr %7, align 4
  br label %36, !llvm.loop !9

86:                                               ; preds = %36
  %87 = load ptr, ptr %6, align 8
  store ptr %87, ptr %3, align 8
  br label %88

88:                                               ; preds = %86, %10
  %89 = load ptr, ptr %3, align 8
  ret ptr %89
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintFunction(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 10
  %9 = load i32, ptr %8, align 8
  store i32 %9, ptr %6, align 4
  %10 = load ptr, ptr %3, align 8
  call void @PrintHeader(ptr noundef %10)
  %11 = load ptr, ptr %3, align 8
  call void @PrintCode(ptr noundef %11)
  %12 = load i32, ptr %4, align 4
  %13 = icmp ne i32 %12, 0
  br i1 %13, label %14, label %16

14:                                               ; preds = %2
  %15 = load ptr, ptr %3, align 8
  call void @PrintDebug(ptr noundef %15)
  br label %16

16:                                               ; preds = %14, %2
  store i32 0, ptr %5, align 4
  br label %17

17:                                               ; preds = %30, %16
  %18 = load i32, ptr %5, align 4
  %19 = load i32, ptr %6, align 4
  %20 = icmp slt i32 %18, %19
  br i1 %20, label %21, label %33

21:                                               ; preds = %17
  %22 = load ptr, ptr %3, align 8
  %23 = getelementptr inbounds %struct.Proto, ptr %22, i32 0, i32 17
  %24 = load ptr, ptr %23, align 8
  %25 = load i32, ptr %5, align 4
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds ptr, ptr %24, i64 %26
  %28 = load ptr, ptr %27, align 8
  %29 = load i32, ptr %4, align 4
  call void @PrintFunction(ptr noundef %28, i32 noundef %29)
  br label %30

30:                                               ; preds = %21
  %31 = load i32, ptr %5, align 4
  %32 = add nsw i32 %31, 1
  store i32 %32, ptr %5, align 4
  br label %17, !llvm.loop !10

33:                                               ; preds = %17
  ret void
}

declare noalias ptr @fopen64(ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @cannot(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr @stderr, align 8
  %4 = load ptr, ptr @progname, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = load ptr, ptr @output, align 8
  %7 = call ptr @__errno_location() #9
  %8 = load i32, ptr %7, align 4
  %9 = call ptr @strerror(i32 noundef %8) #8
  %10 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %3, ptr noundef @.str.175, ptr noundef %4, ptr noundef %5, ptr noundef %6, ptr noundef %9)
  call void @exit(i32 noundef 1) #7
  unreachable
}

declare hidden i32 @luaU_dump(ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @writer(ptr noundef %0, ptr noundef %1, i64 noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = load i64, ptr %7, align 8
  %12 = load ptr, ptr %8, align 8
  %13 = call i64 @fwrite(ptr noundef %10, i64 noundef %11, i64 noundef 1, ptr noundef %12)
  %14 = icmp ne i64 %13, 1
  br i1 %14, label %15, label %18

15:                                               ; preds = %4
  %16 = load i64, ptr %7, align 8
  %17 = icmp ne i64 %16, 0
  br label %18

18:                                               ; preds = %15, %4
  %19 = phi i1 [ false, %4 ], [ %17, %15 ]
  %20 = zext i1 %19 to i32
  ret i32 %20
}

; Function Attrs: nounwind
declare i32 @ferror(ptr noundef) #4

declare i32 @fclose(ptr noundef) #1

declare i32 @lua_load(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @reader(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %5, align 8
  %9 = load ptr, ptr %6, align 8
  %10 = load i32, ptr %9, align 4
  %11 = add nsw i32 %10, -1
  store i32 %11, ptr %9, align 4
  %12 = icmp ne i32 %10, 0
  br i1 %12, label %13, label %15

13:                                               ; preds = %3
  %14 = load ptr, ptr %7, align 8
  store i64 19, ptr %14, align 8
  store ptr @.str.22, ptr %4, align 8
  br label %17

15:                                               ; preds = %3
  %16 = load ptr, ptr %7, align 8
  store i64 0, ptr %16, align 8
  store ptr null, ptr %4, align 8
  br label %17

17:                                               ; preds = %15, %13
  %18 = load ptr, ptr %4, align 8
  ret ptr %18
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintHeader(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.Proto, ptr %4, i32 0, i32 22
  %6 = load ptr, ptr %5, align 8
  %7 = icmp ne ptr %6, null
  br i1 %7, label %8, label %14

8:                                                ; preds = %1
  %9 = load ptr, ptr %2, align 8
  %10 = getelementptr inbounds %struct.Proto, ptr %9, i32 0, i32 22
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds %struct.TString, ptr %11, i32 0, i32 7
  %13 = getelementptr inbounds [1 x i8], ptr %12, i64 0, i64 0
  br label %15

14:                                               ; preds = %1
  br label %15

15:                                               ; preds = %14, %8
  %16 = phi ptr [ %13, %8 ], [ @.str.23, %14 ]
  store ptr %16, ptr %3, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = load i8, ptr %17, align 1
  %19 = sext i8 %18 to i32
  %20 = icmp eq i32 %19, 64
  br i1 %20, label %26, label %21

21:                                               ; preds = %15
  %22 = load ptr, ptr %3, align 8
  %23 = load i8, ptr %22, align 1
  %24 = sext i8 %23 to i32
  %25 = icmp eq i32 %24, 61
  br i1 %25, label %26, label %29

26:                                               ; preds = %21, %15
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds i8, ptr %27, i32 1
  store ptr %28, ptr %3, align 8
  br label %39

29:                                               ; preds = %21
  %30 = load ptr, ptr %3, align 8
  %31 = load i8, ptr %30, align 1
  %32 = sext i8 %31 to i32
  %33 = load i8, ptr @.str.24, align 1
  %34 = sext i8 %33 to i32
  %35 = icmp eq i32 %32, %34
  br i1 %35, label %36, label %37

36:                                               ; preds = %29
  store ptr @.str.25, ptr %3, align 8
  br label %38

37:                                               ; preds = %29
  store ptr @.str.26, ptr %3, align 8
  br label %38

38:                                               ; preds = %37, %36
  br label %39

39:                                               ; preds = %38, %26
  %40 = load ptr, ptr %2, align 8
  %41 = getelementptr inbounds %struct.Proto, ptr %40, i32 0, i32 13
  %42 = load i32, ptr %41, align 4
  %43 = icmp eq i32 %42, 0
  %44 = zext i1 %43 to i64
  %45 = select i1 %43, ptr @.str.28, ptr @.str.29
  %46 = load ptr, ptr %3, align 8
  %47 = load ptr, ptr %2, align 8
  %48 = getelementptr inbounds %struct.Proto, ptr %47, i32 0, i32 13
  %49 = load i32, ptr %48, align 4
  %50 = load ptr, ptr %2, align 8
  %51 = getelementptr inbounds %struct.Proto, ptr %50, i32 0, i32 14
  %52 = load i32, ptr %51, align 8
  %53 = load ptr, ptr %2, align 8
  %54 = getelementptr inbounds %struct.Proto, ptr %53, i32 0, i32 8
  %55 = load i32, ptr %54, align 8
  %56 = load ptr, ptr %2, align 8
  %57 = getelementptr inbounds %struct.Proto, ptr %56, i32 0, i32 8
  %58 = load i32, ptr %57, align 8
  %59 = icmp eq i32 %58, 1
  %60 = zext i1 %59 to i64
  %61 = select i1 %59, ptr @.str.30, ptr @.str.31
  %62 = load ptr, ptr %2, align 8
  %63 = call i32 (ptr, ...) @printf(ptr noundef @.str.27, ptr noundef %45, ptr noundef %46, i32 noundef %49, i32 noundef %52, i32 noundef %55, ptr noundef %61, ptr noundef %62)
  %64 = load ptr, ptr %2, align 8
  %65 = getelementptr inbounds %struct.Proto, ptr %64, i32 0, i32 3
  %66 = load i8, ptr %65, align 2
  %67 = zext i8 %66 to i32
  %68 = load ptr, ptr %2, align 8
  %69 = getelementptr inbounds %struct.Proto, ptr %68, i32 0, i32 4
  %70 = load i8, ptr %69, align 1
  %71 = zext i8 %70 to i32
  %72 = icmp ne i32 %71, 0
  %73 = zext i1 %72 to i64
  %74 = select i1 %72, ptr @.str.33, ptr @.str.30
  %75 = load ptr, ptr %2, align 8
  %76 = getelementptr inbounds %struct.Proto, ptr %75, i32 0, i32 3
  %77 = load i8, ptr %76, align 2
  %78 = zext i8 %77 to i32
  %79 = icmp eq i32 %78, 1
  %80 = zext i1 %79 to i64
  %81 = select i1 %79, ptr @.str.30, ptr @.str.31
  %82 = load ptr, ptr %2, align 8
  %83 = getelementptr inbounds %struct.Proto, ptr %82, i32 0, i32 5
  %84 = load i8, ptr %83, align 4
  %85 = zext i8 %84 to i32
  %86 = load ptr, ptr %2, align 8
  %87 = getelementptr inbounds %struct.Proto, ptr %86, i32 0, i32 5
  %88 = load i8, ptr %87, align 4
  %89 = zext i8 %88 to i32
  %90 = icmp eq i32 %89, 1
  %91 = zext i1 %90 to i64
  %92 = select i1 %90, ptr @.str.30, ptr @.str.31
  %93 = load ptr, ptr %2, align 8
  %94 = getelementptr inbounds %struct.Proto, ptr %93, i32 0, i32 6
  %95 = load i32, ptr %94, align 8
  %96 = load ptr, ptr %2, align 8
  %97 = getelementptr inbounds %struct.Proto, ptr %96, i32 0, i32 6
  %98 = load i32, ptr %97, align 8
  %99 = icmp eq i32 %98, 1
  %100 = zext i1 %99 to i64
  %101 = select i1 %99, ptr @.str.30, ptr @.str.31
  %102 = call i32 (ptr, ...) @printf(ptr noundef @.str.32, i32 noundef %67, ptr noundef %74, ptr noundef %81, i32 noundef %85, ptr noundef %92, i32 noundef %95, ptr noundef %101)
  %103 = load ptr, ptr %2, align 8
  %104 = getelementptr inbounds %struct.Proto, ptr %103, i32 0, i32 11
  %105 = load i32, ptr %104, align 4
  %106 = load ptr, ptr %2, align 8
  %107 = getelementptr inbounds %struct.Proto, ptr %106, i32 0, i32 11
  %108 = load i32, ptr %107, align 4
  %109 = icmp eq i32 %108, 1
  %110 = zext i1 %109 to i64
  %111 = select i1 %109, ptr @.str.30, ptr @.str.31
  %112 = load ptr, ptr %2, align 8
  %113 = getelementptr inbounds %struct.Proto, ptr %112, i32 0, i32 7
  %114 = load i32, ptr %113, align 4
  %115 = load ptr, ptr %2, align 8
  %116 = getelementptr inbounds %struct.Proto, ptr %115, i32 0, i32 7
  %117 = load i32, ptr %116, align 4
  %118 = icmp eq i32 %117, 1
  %119 = zext i1 %118 to i64
  %120 = select i1 %118, ptr @.str.30, ptr @.str.31
  %121 = load ptr, ptr %2, align 8
  %122 = getelementptr inbounds %struct.Proto, ptr %121, i32 0, i32 10
  %123 = load i32, ptr %122, align 8
  %124 = load ptr, ptr %2, align 8
  %125 = getelementptr inbounds %struct.Proto, ptr %124, i32 0, i32 10
  %126 = load i32, ptr %125, align 8
  %127 = icmp eq i32 %126, 1
  %128 = zext i1 %127 to i64
  %129 = select i1 %127, ptr @.str.30, ptr @.str.31
  %130 = call i32 (ptr, ...) @printf(ptr noundef @.str.34, i32 noundef %105, ptr noundef %111, i32 noundef %114, ptr noundef %120, i32 noundef %123, ptr noundef %129)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintCode(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
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
  store ptr %0, ptr %2, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.Proto, ptr %18, i32 0, i32 16
  %20 = load ptr, ptr %19, align 8
  store ptr %20, ptr %3, align 8
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.Proto, ptr %21, i32 0, i32 8
  %23 = load i32, ptr %22, align 8
  store i32 %23, ptr %5, align 4
  store i32 0, ptr %4, align 4
  br label %24

24:                                               ; preds = %827, %1
  %25 = load i32, ptr %4, align 4
  %26 = load i32, ptr %5, align 4
  %27 = icmp slt i32 %25, %26
  br i1 %27, label %28, label %830

28:                                               ; preds = %24
  %29 = load ptr, ptr %3, align 8
  %30 = load i32, ptr %4, align 4
  %31 = sext i32 %30 to i64
  %32 = getelementptr inbounds i32, ptr %29, i64 %31
  %33 = load i32, ptr %32, align 4
  store i32 %33, ptr %6, align 4
  %34 = load i32, ptr %6, align 4
  %35 = lshr i32 %34, 0
  %36 = and i32 %35, 127
  store i32 %36, ptr %7, align 4
  %37 = load i32, ptr %6, align 4
  %38 = lshr i32 %37, 7
  %39 = and i32 %38, 255
  store i32 %39, ptr %8, align 4
  %40 = load i32, ptr %6, align 4
  %41 = lshr i32 %40, 16
  %42 = and i32 %41, 255
  store i32 %42, ptr %9, align 4
  %43 = load i32, ptr %6, align 4
  %44 = lshr i32 %43, 24
  %45 = and i32 %44, 255
  store i32 %45, ptr %10, align 4
  %46 = load i32, ptr %6, align 4
  %47 = lshr i32 %46, 7
  %48 = and i32 %47, 33554431
  store i32 %48, ptr %11, align 4
  %49 = load i32, ptr %6, align 4
  %50 = lshr i32 %49, 15
  %51 = and i32 %50, 131071
  store i32 %51, ptr %12, align 4
  %52 = load i32, ptr %6, align 4
  %53 = lshr i32 %52, 16
  %54 = and i32 %53, 255
  %55 = sub nsw i32 %54, 127
  store i32 %55, ptr %13, align 4
  %56 = load i32, ptr %6, align 4
  %57 = lshr i32 %56, 24
  %58 = and i32 %57, 255
  %59 = sub nsw i32 %58, 127
  store i32 %59, ptr %14, align 4
  %60 = load i32, ptr %6, align 4
  %61 = lshr i32 %60, 15
  %62 = and i32 %61, 131071
  %63 = sub nsw i32 %62, 65535
  store i32 %63, ptr %15, align 4
  %64 = load i32, ptr %6, align 4
  %65 = lshr i32 %64, 15
  %66 = and i32 %65, 1
  store i32 %66, ptr %16, align 4
  %67 = load ptr, ptr %2, align 8
  %68 = load i32, ptr %4, align 4
  %69 = call i32 @luaG_getfuncline(ptr noundef %67, i32 noundef %68)
  store i32 %69, ptr %17, align 4
  %70 = load i32, ptr %4, align 4
  %71 = add nsw i32 %70, 1
  %72 = call i32 (ptr, ...) @printf(ptr noundef @.str.35, i32 noundef %71)
  %73 = load i32, ptr %17, align 4
  %74 = icmp sgt i32 %73, 0
  br i1 %74, label %75, label %78

75:                                               ; preds = %28
  %76 = load i32, ptr %17, align 4
  %77 = call i32 (ptr, ...) @printf(ptr noundef @.str.36, i32 noundef %76)
  br label %80

78:                                               ; preds = %28
  %79 = call i32 (ptr, ...) @printf(ptr noundef @.str.37)
  br label %80

80:                                               ; preds = %78, %75
  %81 = load i32, ptr %7, align 4
  %82 = zext i32 %81 to i64
  %83 = getelementptr inbounds [84 x ptr], ptr @opnames, i64 0, i64 %82
  %84 = load ptr, ptr %83, align 8
  %85 = call i32 (ptr, ...) @printf(ptr noundef @.str.38, ptr noundef %84)
  %86 = load i32, ptr %7, align 4
  switch i32 %86, label %825 [
    i32 0, label %87
    i32 1, label %91
    i32 2, label %95
    i32 3, label %99
    i32 4, label %106
    i32 5, label %119
    i32 6, label %122
    i32 7, label %125
    i32 8, label %128
    i32 9, label %135
    i32 10, label %163
    i32 11, label %191
    i32 12, label %223
    i32 13, label %228
    i32 14, label %233
    i32 15, label %241
    i32 16, label %284
    i32 17, label %300
    i32 18, label %316
    i32 19, label %335
    i32 20, label %352
    i32 21, label %368
    i32 22, label %373
    i32 23, label %381
    i32 24, label %389
    i32 25, label %397
    i32 26, label %405
    i32 27, label %413
    i32 28, label %421
    i32 29, label %429
    i32 30, label %437
    i32 31, label %445
    i32 32, label %453
    i32 33, label %458
    i32 34, label %463
    i32 35, label %468
    i32 36, label %473
    i32 37, label %478
    i32 38, label %483
    i32 39, label %488
    i32 40, label %493
    i32 41, label %498
    i32 42, label %503
    i32 43, label %508
    i32 44, label %513
    i32 45, label %518
    i32 46, label %523
    i32 47, label %536
    i32 48, label %555
    i32 49, label %576
    i32 50, label %580
    i32 51, label %584
    i32 52, label %588
    i32 53, label %592
    i32 54, label %596
    i32 55, label %599
    i32 56, label %602
    i32 57, label %616
    i32 58, label %621
    i32 59, label %626
    i32 60, label %631
    i32 61, label %639
    i32 62, label %644
    i32 63, label %649
    i32 64, label %654
    i32 65, label %659
    i32 66, label %664
    i32 67, label %668
    i32 68, label %673
    i32 69, label %697
    i32 70, label %709
    i32 71, label %728
    i32 72, label %729
    i32 73, label %732
    i32 74, label %741
    i32 75, label %750
    i32 76, label %759
    i32 77, label %763
    i32 78, label %772
    i32 79, label %793
    i32 80, label %805
    i32 81, label %819
    i32 82, label %822
  ]

87:                                               ; preds = %80
  %88 = load i32, ptr %8, align 4
  %89 = load i32, ptr %9, align 4
  %90 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %88, i32 noundef %89)
  br label %825

91:                                               ; preds = %80
  %92 = load i32, ptr %8, align 4
  %93 = load i32, ptr %15, align 4
  %94 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %92, i32 noundef %93)
  br label %825

95:                                               ; preds = %80
  %96 = load i32, ptr %8, align 4
  %97 = load i32, ptr %15, align 4
  %98 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %96, i32 noundef %97)
  br label %825

99:                                               ; preds = %80
  %100 = load i32, ptr %8, align 4
  %101 = load i32, ptr %12, align 4
  %102 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %100, i32 noundef %101)
  %103 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %104 = load ptr, ptr %2, align 8
  %105 = load i32, ptr %12, align 4
  call void @PrintConstant(ptr noundef %104, i32 noundef %105)
  br label %825

106:                                              ; preds = %80
  %107 = load i32, ptr %8, align 4
  %108 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %107)
  %109 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %110 = load ptr, ptr %2, align 8
  %111 = load ptr, ptr %3, align 8
  %112 = load i32, ptr %4, align 4
  %113 = add nsw i32 %112, 1
  %114 = sext i32 %113 to i64
  %115 = getelementptr inbounds i32, ptr %111, i64 %114
  %116 = load i32, ptr %115, align 4
  %117 = lshr i32 %116, 7
  %118 = and i32 %117, 33554431
  call void @PrintConstant(ptr noundef %110, i32 noundef %118)
  br label %825

119:                                              ; preds = %80
  %120 = load i32, ptr %8, align 4
  %121 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %120)
  br label %825

122:                                              ; preds = %80
  %123 = load i32, ptr %8, align 4
  %124 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %123)
  br label %825

125:                                              ; preds = %80
  %126 = load i32, ptr %8, align 4
  %127 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %126)
  br label %825

128:                                              ; preds = %80
  %129 = load i32, ptr %8, align 4
  %130 = load i32, ptr %9, align 4
  %131 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %129, i32 noundef %130)
  %132 = load i32, ptr %9, align 4
  %133 = add nsw i32 %132, 1
  %134 = call i32 (ptr, ...) @printf(ptr noundef @.str.42, i32 noundef %133)
  br label %825

135:                                              ; preds = %80
  %136 = load i32, ptr %8, align 4
  %137 = load i32, ptr %9, align 4
  %138 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %136, i32 noundef %137)
  %139 = load ptr, ptr %2, align 8
  %140 = getelementptr inbounds %struct.Proto, ptr %139, i32 0, i32 18
  %141 = load ptr, ptr %140, align 8
  %142 = load i32, ptr %9, align 4
  %143 = sext i32 %142 to i64
  %144 = getelementptr inbounds %struct.Upvaldesc, ptr %141, i64 %143
  %145 = getelementptr inbounds %struct.Upvaldesc, ptr %144, i32 0, i32 0
  %146 = load ptr, ptr %145, align 8
  %147 = icmp ne ptr %146, null
  br i1 %147, label %148, label %159

148:                                              ; preds = %135
  %149 = load ptr, ptr %2, align 8
  %150 = getelementptr inbounds %struct.Proto, ptr %149, i32 0, i32 18
  %151 = load ptr, ptr %150, align 8
  %152 = load i32, ptr %9, align 4
  %153 = sext i32 %152 to i64
  %154 = getelementptr inbounds %struct.Upvaldesc, ptr %151, i64 %153
  %155 = getelementptr inbounds %struct.Upvaldesc, ptr %154, i32 0, i32 0
  %156 = load ptr, ptr %155, align 8
  %157 = getelementptr inbounds %struct.TString, ptr %156, i32 0, i32 7
  %158 = getelementptr inbounds [1 x i8], ptr %157, i64 0, i64 0
  br label %160

159:                                              ; preds = %135
  br label %160

160:                                              ; preds = %159, %148
  %161 = phi ptr [ %158, %148 ], [ @.str.3, %159 ]
  %162 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %161)
  br label %825

163:                                              ; preds = %80
  %164 = load i32, ptr %8, align 4
  %165 = load i32, ptr %9, align 4
  %166 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %164, i32 noundef %165)
  %167 = load ptr, ptr %2, align 8
  %168 = getelementptr inbounds %struct.Proto, ptr %167, i32 0, i32 18
  %169 = load ptr, ptr %168, align 8
  %170 = load i32, ptr %9, align 4
  %171 = sext i32 %170 to i64
  %172 = getelementptr inbounds %struct.Upvaldesc, ptr %169, i64 %171
  %173 = getelementptr inbounds %struct.Upvaldesc, ptr %172, i32 0, i32 0
  %174 = load ptr, ptr %173, align 8
  %175 = icmp ne ptr %174, null
  br i1 %175, label %176, label %187

176:                                              ; preds = %163
  %177 = load ptr, ptr %2, align 8
  %178 = getelementptr inbounds %struct.Proto, ptr %177, i32 0, i32 18
  %179 = load ptr, ptr %178, align 8
  %180 = load i32, ptr %9, align 4
  %181 = sext i32 %180 to i64
  %182 = getelementptr inbounds %struct.Upvaldesc, ptr %179, i64 %181
  %183 = getelementptr inbounds %struct.Upvaldesc, ptr %182, i32 0, i32 0
  %184 = load ptr, ptr %183, align 8
  %185 = getelementptr inbounds %struct.TString, ptr %184, i32 0, i32 7
  %186 = getelementptr inbounds [1 x i8], ptr %185, i64 0, i64 0
  br label %188

187:                                              ; preds = %163
  br label %188

188:                                              ; preds = %187, %176
  %189 = phi ptr [ %186, %176 ], [ @.str.3, %187 ]
  %190 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %189)
  br label %825

191:                                              ; preds = %80
  %192 = load i32, ptr %8, align 4
  %193 = load i32, ptr %9, align 4
  %194 = load i32, ptr %10, align 4
  %195 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %192, i32 noundef %193, i32 noundef %194)
  %196 = load ptr, ptr %2, align 8
  %197 = getelementptr inbounds %struct.Proto, ptr %196, i32 0, i32 18
  %198 = load ptr, ptr %197, align 8
  %199 = load i32, ptr %9, align 4
  %200 = sext i32 %199 to i64
  %201 = getelementptr inbounds %struct.Upvaldesc, ptr %198, i64 %200
  %202 = getelementptr inbounds %struct.Upvaldesc, ptr %201, i32 0, i32 0
  %203 = load ptr, ptr %202, align 8
  %204 = icmp ne ptr %203, null
  br i1 %204, label %205, label %216

205:                                              ; preds = %191
  %206 = load ptr, ptr %2, align 8
  %207 = getelementptr inbounds %struct.Proto, ptr %206, i32 0, i32 18
  %208 = load ptr, ptr %207, align 8
  %209 = load i32, ptr %9, align 4
  %210 = sext i32 %209 to i64
  %211 = getelementptr inbounds %struct.Upvaldesc, ptr %208, i64 %210
  %212 = getelementptr inbounds %struct.Upvaldesc, ptr %211, i32 0, i32 0
  %213 = load ptr, ptr %212, align 8
  %214 = getelementptr inbounds %struct.TString, ptr %213, i32 0, i32 7
  %215 = getelementptr inbounds [1 x i8], ptr %214, i64 0, i64 0
  br label %217

216:                                              ; preds = %191
  br label %217

217:                                              ; preds = %216, %205
  %218 = phi ptr [ %215, %205 ], [ @.str.3, %216 ]
  %219 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %218)
  %220 = call i32 (ptr, ...) @printf(ptr noundef @.str.45)
  %221 = load ptr, ptr %2, align 8
  %222 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %221, i32 noundef %222)
  br label %825

223:                                              ; preds = %80
  %224 = load i32, ptr %8, align 4
  %225 = load i32, ptr %9, align 4
  %226 = load i32, ptr %10, align 4
  %227 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %224, i32 noundef %225, i32 noundef %226)
  br label %825

228:                                              ; preds = %80
  %229 = load i32, ptr %8, align 4
  %230 = load i32, ptr %9, align 4
  %231 = load i32, ptr %10, align 4
  %232 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %229, i32 noundef %230, i32 noundef %231)
  br label %825

233:                                              ; preds = %80
  %234 = load i32, ptr %8, align 4
  %235 = load i32, ptr %9, align 4
  %236 = load i32, ptr %10, align 4
  %237 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %234, i32 noundef %235, i32 noundef %236)
  %238 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %239 = load ptr, ptr %2, align 8
  %240 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %239, i32 noundef %240)
  br label %825

241:                                              ; preds = %80
  %242 = load i32, ptr %8, align 4
  %243 = load i32, ptr %9, align 4
  %244 = load i32, ptr %10, align 4
  %245 = load i32, ptr %16, align 4
  %246 = icmp ne i32 %245, 0
  %247 = zext i1 %246 to i64
  %248 = select i1 %246, ptr @.str.47, ptr @.str.30
  %249 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %242, i32 noundef %243, i32 noundef %244, ptr noundef %248)
  %250 = load ptr, ptr %2, align 8
  %251 = getelementptr inbounds %struct.Proto, ptr %250, i32 0, i32 18
  %252 = load ptr, ptr %251, align 8
  %253 = load i32, ptr %8, align 4
  %254 = sext i32 %253 to i64
  %255 = getelementptr inbounds %struct.Upvaldesc, ptr %252, i64 %254
  %256 = getelementptr inbounds %struct.Upvaldesc, ptr %255, i32 0, i32 0
  %257 = load ptr, ptr %256, align 8
  %258 = icmp ne ptr %257, null
  br i1 %258, label %259, label %270

259:                                              ; preds = %241
  %260 = load ptr, ptr %2, align 8
  %261 = getelementptr inbounds %struct.Proto, ptr %260, i32 0, i32 18
  %262 = load ptr, ptr %261, align 8
  %263 = load i32, ptr %8, align 4
  %264 = sext i32 %263 to i64
  %265 = getelementptr inbounds %struct.Upvaldesc, ptr %262, i64 %264
  %266 = getelementptr inbounds %struct.Upvaldesc, ptr %265, i32 0, i32 0
  %267 = load ptr, ptr %266, align 8
  %268 = getelementptr inbounds %struct.TString, ptr %267, i32 0, i32 7
  %269 = getelementptr inbounds [1 x i8], ptr %268, i64 0, i64 0
  br label %271

270:                                              ; preds = %241
  br label %271

271:                                              ; preds = %270, %259
  %272 = phi ptr [ %269, %259 ], [ @.str.3, %270 ]
  %273 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %272)
  %274 = call i32 (ptr, ...) @printf(ptr noundef @.str.45)
  %275 = load ptr, ptr %2, align 8
  %276 = load i32, ptr %9, align 4
  call void @PrintConstant(ptr noundef %275, i32 noundef %276)
  %277 = load i32, ptr %16, align 4
  %278 = icmp ne i32 %277, 0
  br i1 %278, label %279, label %283

279:                                              ; preds = %271
  %280 = call i32 (ptr, ...) @printf(ptr noundef @.str.45)
  %281 = load ptr, ptr %2, align 8
  %282 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %281, i32 noundef %282)
  br label %283

283:                                              ; preds = %279, %271
  br label %825

284:                                              ; preds = %80
  %285 = load i32, ptr %8, align 4
  %286 = load i32, ptr %9, align 4
  %287 = load i32, ptr %10, align 4
  %288 = load i32, ptr %16, align 4
  %289 = icmp ne i32 %288, 0
  %290 = zext i1 %289 to i64
  %291 = select i1 %289, ptr @.str.47, ptr @.str.30
  %292 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %285, i32 noundef %286, i32 noundef %287, ptr noundef %291)
  %293 = load i32, ptr %16, align 4
  %294 = icmp ne i32 %293, 0
  br i1 %294, label %295, label %299

295:                                              ; preds = %284
  %296 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %297 = load ptr, ptr %2, align 8
  %298 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %297, i32 noundef %298)
  br label %299

299:                                              ; preds = %295, %284
  br label %825

300:                                              ; preds = %80
  %301 = load i32, ptr %8, align 4
  %302 = load i32, ptr %9, align 4
  %303 = load i32, ptr %10, align 4
  %304 = load i32, ptr %16, align 4
  %305 = icmp ne i32 %304, 0
  %306 = zext i1 %305 to i64
  %307 = select i1 %305, ptr @.str.47, ptr @.str.30
  %308 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %301, i32 noundef %302, i32 noundef %303, ptr noundef %307)
  %309 = load i32, ptr %16, align 4
  %310 = icmp ne i32 %309, 0
  br i1 %310, label %311, label %315

311:                                              ; preds = %300
  %312 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %313 = load ptr, ptr %2, align 8
  %314 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %313, i32 noundef %314)
  br label %315

315:                                              ; preds = %311, %300
  br label %825

316:                                              ; preds = %80
  %317 = load i32, ptr %8, align 4
  %318 = load i32, ptr %9, align 4
  %319 = load i32, ptr %10, align 4
  %320 = load i32, ptr %16, align 4
  %321 = icmp ne i32 %320, 0
  %322 = zext i1 %321 to i64
  %323 = select i1 %321, ptr @.str.47, ptr @.str.30
  %324 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %317, i32 noundef %318, i32 noundef %319, ptr noundef %323)
  %325 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %326 = load ptr, ptr %2, align 8
  %327 = load i32, ptr %9, align 4
  call void @PrintConstant(ptr noundef %326, i32 noundef %327)
  %328 = load i32, ptr %16, align 4
  %329 = icmp ne i32 %328, 0
  br i1 %329, label %330, label %334

330:                                              ; preds = %316
  %331 = call i32 (ptr, ...) @printf(ptr noundef @.str.45)
  %332 = load ptr, ptr %2, align 8
  %333 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %332, i32 noundef %333)
  br label %334

334:                                              ; preds = %330, %316
  br label %825

335:                                              ; preds = %80
  %336 = load i32, ptr %8, align 4
  %337 = load i32, ptr %9, align 4
  %338 = load i32, ptr %10, align 4
  %339 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %336, i32 noundef %337, i32 noundef %338)
  %340 = load i32, ptr %10, align 4
  %341 = load ptr, ptr %3, align 8
  %342 = load i32, ptr %4, align 4
  %343 = add nsw i32 %342, 1
  %344 = sext i32 %343 to i64
  %345 = getelementptr inbounds i32, ptr %341, i64 %344
  %346 = load i32, ptr %345, align 4
  %347 = lshr i32 %346, 7
  %348 = and i32 %347, 33554431
  %349 = mul nsw i32 %348, 256
  %350 = add nsw i32 %340, %349
  %351 = call i32 (ptr, ...) @printf(ptr noundef @.str.48, i32 noundef %350)
  br label %825

352:                                              ; preds = %80
  %353 = load i32, ptr %8, align 4
  %354 = load i32, ptr %9, align 4
  %355 = load i32, ptr %10, align 4
  %356 = load i32, ptr %16, align 4
  %357 = icmp ne i32 %356, 0
  %358 = zext i1 %357 to i64
  %359 = select i1 %357, ptr @.str.47, ptr @.str.30
  %360 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %353, i32 noundef %354, i32 noundef %355, ptr noundef %359)
  %361 = load i32, ptr %16, align 4
  %362 = icmp ne i32 %361, 0
  br i1 %362, label %363, label %367

363:                                              ; preds = %352
  %364 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %365 = load ptr, ptr %2, align 8
  %366 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %365, i32 noundef %366)
  br label %367

367:                                              ; preds = %363, %352
  br label %825

368:                                              ; preds = %80
  %369 = load i32, ptr %8, align 4
  %370 = load i32, ptr %9, align 4
  %371 = load i32, ptr %14, align 4
  %372 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %369, i32 noundef %370, i32 noundef %371)
  br label %825

373:                                              ; preds = %80
  %374 = load i32, ptr %8, align 4
  %375 = load i32, ptr %9, align 4
  %376 = load i32, ptr %10, align 4
  %377 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %374, i32 noundef %375, i32 noundef %376)
  %378 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %379 = load ptr, ptr %2, align 8
  %380 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %379, i32 noundef %380)
  br label %825

381:                                              ; preds = %80
  %382 = load i32, ptr %8, align 4
  %383 = load i32, ptr %9, align 4
  %384 = load i32, ptr %10, align 4
  %385 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %382, i32 noundef %383, i32 noundef %384)
  %386 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %387 = load ptr, ptr %2, align 8
  %388 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %387, i32 noundef %388)
  br label %825

389:                                              ; preds = %80
  %390 = load i32, ptr %8, align 4
  %391 = load i32, ptr %9, align 4
  %392 = load i32, ptr %10, align 4
  %393 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %390, i32 noundef %391, i32 noundef %392)
  %394 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %395 = load ptr, ptr %2, align 8
  %396 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %395, i32 noundef %396)
  br label %825

397:                                              ; preds = %80
  %398 = load i32, ptr %8, align 4
  %399 = load i32, ptr %9, align 4
  %400 = load i32, ptr %10, align 4
  %401 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %398, i32 noundef %399, i32 noundef %400)
  %402 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %403 = load ptr, ptr %2, align 8
  %404 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %403, i32 noundef %404)
  br label %825

405:                                              ; preds = %80
  %406 = load i32, ptr %8, align 4
  %407 = load i32, ptr %9, align 4
  %408 = load i32, ptr %10, align 4
  %409 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %406, i32 noundef %407, i32 noundef %408)
  %410 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %411 = load ptr, ptr %2, align 8
  %412 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %411, i32 noundef %412)
  br label %825

413:                                              ; preds = %80
  %414 = load i32, ptr %8, align 4
  %415 = load i32, ptr %9, align 4
  %416 = load i32, ptr %10, align 4
  %417 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %414, i32 noundef %415, i32 noundef %416)
  %418 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %419 = load ptr, ptr %2, align 8
  %420 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %419, i32 noundef %420)
  br label %825

421:                                              ; preds = %80
  %422 = load i32, ptr %8, align 4
  %423 = load i32, ptr %9, align 4
  %424 = load i32, ptr %10, align 4
  %425 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %422, i32 noundef %423, i32 noundef %424)
  %426 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %427 = load ptr, ptr %2, align 8
  %428 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %427, i32 noundef %428)
  br label %825

429:                                              ; preds = %80
  %430 = load i32, ptr %8, align 4
  %431 = load i32, ptr %9, align 4
  %432 = load i32, ptr %10, align 4
  %433 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %430, i32 noundef %431, i32 noundef %432)
  %434 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %435 = load ptr, ptr %2, align 8
  %436 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %435, i32 noundef %436)
  br label %825

437:                                              ; preds = %80
  %438 = load i32, ptr %8, align 4
  %439 = load i32, ptr %9, align 4
  %440 = load i32, ptr %10, align 4
  %441 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %438, i32 noundef %439, i32 noundef %440)
  %442 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %443 = load ptr, ptr %2, align 8
  %444 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %443, i32 noundef %444)
  br label %825

445:                                              ; preds = %80
  %446 = load i32, ptr %8, align 4
  %447 = load i32, ptr %9, align 4
  %448 = load i32, ptr %10, align 4
  %449 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %446, i32 noundef %447, i32 noundef %448)
  %450 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %451 = load ptr, ptr %2, align 8
  %452 = load i32, ptr %10, align 4
  call void @PrintConstant(ptr noundef %451, i32 noundef %452)
  br label %825

453:                                              ; preds = %80
  %454 = load i32, ptr %8, align 4
  %455 = load i32, ptr %9, align 4
  %456 = load i32, ptr %14, align 4
  %457 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %454, i32 noundef %455, i32 noundef %456)
  br label %825

458:                                              ; preds = %80
  %459 = load i32, ptr %8, align 4
  %460 = load i32, ptr %9, align 4
  %461 = load i32, ptr %14, align 4
  %462 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %459, i32 noundef %460, i32 noundef %461)
  br label %825

463:                                              ; preds = %80
  %464 = load i32, ptr %8, align 4
  %465 = load i32, ptr %9, align 4
  %466 = load i32, ptr %10, align 4
  %467 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %464, i32 noundef %465, i32 noundef %466)
  br label %825

468:                                              ; preds = %80
  %469 = load i32, ptr %8, align 4
  %470 = load i32, ptr %9, align 4
  %471 = load i32, ptr %10, align 4
  %472 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %469, i32 noundef %470, i32 noundef %471)
  br label %825

473:                                              ; preds = %80
  %474 = load i32, ptr %8, align 4
  %475 = load i32, ptr %9, align 4
  %476 = load i32, ptr %10, align 4
  %477 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %474, i32 noundef %475, i32 noundef %476)
  br label %825

478:                                              ; preds = %80
  %479 = load i32, ptr %8, align 4
  %480 = load i32, ptr %9, align 4
  %481 = load i32, ptr %10, align 4
  %482 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %479, i32 noundef %480, i32 noundef %481)
  br label %825

483:                                              ; preds = %80
  %484 = load i32, ptr %8, align 4
  %485 = load i32, ptr %9, align 4
  %486 = load i32, ptr %10, align 4
  %487 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %484, i32 noundef %485, i32 noundef %486)
  br label %825

488:                                              ; preds = %80
  %489 = load i32, ptr %8, align 4
  %490 = load i32, ptr %9, align 4
  %491 = load i32, ptr %10, align 4
  %492 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %489, i32 noundef %490, i32 noundef %491)
  br label %825

493:                                              ; preds = %80
  %494 = load i32, ptr %8, align 4
  %495 = load i32, ptr %9, align 4
  %496 = load i32, ptr %10, align 4
  %497 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %494, i32 noundef %495, i32 noundef %496)
  br label %825

498:                                              ; preds = %80
  %499 = load i32, ptr %8, align 4
  %500 = load i32, ptr %9, align 4
  %501 = load i32, ptr %10, align 4
  %502 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %499, i32 noundef %500, i32 noundef %501)
  br label %825

503:                                              ; preds = %80
  %504 = load i32, ptr %8, align 4
  %505 = load i32, ptr %9, align 4
  %506 = load i32, ptr %10, align 4
  %507 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %504, i32 noundef %505, i32 noundef %506)
  br label %825

508:                                              ; preds = %80
  %509 = load i32, ptr %8, align 4
  %510 = load i32, ptr %9, align 4
  %511 = load i32, ptr %10, align 4
  %512 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %509, i32 noundef %510, i32 noundef %511)
  br label %825

513:                                              ; preds = %80
  %514 = load i32, ptr %8, align 4
  %515 = load i32, ptr %9, align 4
  %516 = load i32, ptr %10, align 4
  %517 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %514, i32 noundef %515, i32 noundef %516)
  br label %825

518:                                              ; preds = %80
  %519 = load i32, ptr %8, align 4
  %520 = load i32, ptr %9, align 4
  %521 = load i32, ptr %10, align 4
  %522 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %519, i32 noundef %520, i32 noundef %521)
  br label %825

523:                                              ; preds = %80
  %524 = load i32, ptr %8, align 4
  %525 = load i32, ptr %9, align 4
  %526 = load i32, ptr %10, align 4
  %527 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %524, i32 noundef %525, i32 noundef %526)
  %528 = load ptr, ptr @tmname, align 8
  %529 = load i32, ptr %10, align 4
  %530 = sext i32 %529 to i64
  %531 = getelementptr inbounds ptr, ptr %528, i64 %530
  %532 = load ptr, ptr %531, align 8
  %533 = getelementptr inbounds %struct.TString, ptr %532, i32 0, i32 7
  %534 = getelementptr inbounds [1 x i8], ptr %533, i64 0, i64 0
  %535 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %534)
  br label %825

536:                                              ; preds = %80
  %537 = load i32, ptr %8, align 4
  %538 = load i32, ptr %13, align 4
  %539 = load i32, ptr %10, align 4
  %540 = load i32, ptr %16, align 4
  %541 = call i32 (ptr, ...) @printf(ptr noundef @.str.49, i32 noundef %537, i32 noundef %538, i32 noundef %539, i32 noundef %540)
  %542 = load ptr, ptr @tmname, align 8
  %543 = load i32, ptr %10, align 4
  %544 = sext i32 %543 to i64
  %545 = getelementptr inbounds ptr, ptr %542, i64 %544
  %546 = load ptr, ptr %545, align 8
  %547 = getelementptr inbounds %struct.TString, ptr %546, i32 0, i32 7
  %548 = getelementptr inbounds [1 x i8], ptr %547, i64 0, i64 0
  %549 = call i32 (ptr, ...) @printf(ptr noundef @.str.43, ptr noundef %548)
  %550 = load i32, ptr %16, align 4
  %551 = icmp ne i32 %550, 0
  br i1 %551, label %552, label %554

552:                                              ; preds = %536
  %553 = call i32 (ptr, ...) @printf(ptr noundef @.str.50)
  br label %554

554:                                              ; preds = %552, %536
  br label %825

555:                                              ; preds = %80
  %556 = load i32, ptr %8, align 4
  %557 = load i32, ptr %9, align 4
  %558 = load i32, ptr %10, align 4
  %559 = load i32, ptr %16, align 4
  %560 = call i32 (ptr, ...) @printf(ptr noundef @.str.49, i32 noundef %556, i32 noundef %557, i32 noundef %558, i32 noundef %559)
  %561 = load ptr, ptr @tmname, align 8
  %562 = load i32, ptr %10, align 4
  %563 = sext i32 %562 to i64
  %564 = getelementptr inbounds ptr, ptr %561, i64 %563
  %565 = load ptr, ptr %564, align 8
  %566 = getelementptr inbounds %struct.TString, ptr %565, i32 0, i32 7
  %567 = getelementptr inbounds [1 x i8], ptr %566, i64 0, i64 0
  %568 = call i32 (ptr, ...) @printf(ptr noundef @.str.51, ptr noundef %567)
  %569 = load ptr, ptr %2, align 8
  %570 = load i32, ptr %9, align 4
  call void @PrintConstant(ptr noundef %569, i32 noundef %570)
  %571 = load i32, ptr %16, align 4
  %572 = icmp ne i32 %571, 0
  br i1 %572, label %573, label %575

573:                                              ; preds = %555
  %574 = call i32 (ptr, ...) @printf(ptr noundef @.str.50)
  br label %575

575:                                              ; preds = %573, %555
  br label %825

576:                                              ; preds = %80
  %577 = load i32, ptr %8, align 4
  %578 = load i32, ptr %9, align 4
  %579 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %577, i32 noundef %578)
  br label %825

580:                                              ; preds = %80
  %581 = load i32, ptr %8, align 4
  %582 = load i32, ptr %9, align 4
  %583 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %581, i32 noundef %582)
  br label %825

584:                                              ; preds = %80
  %585 = load i32, ptr %8, align 4
  %586 = load i32, ptr %9, align 4
  %587 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %585, i32 noundef %586)
  br label %825

588:                                              ; preds = %80
  %589 = load i32, ptr %8, align 4
  %590 = load i32, ptr %9, align 4
  %591 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %589, i32 noundef %590)
  br label %825

592:                                              ; preds = %80
  %593 = load i32, ptr %8, align 4
  %594 = load i32, ptr %9, align 4
  %595 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %593, i32 noundef %594)
  br label %825

596:                                              ; preds = %80
  %597 = load i32, ptr %8, align 4
  %598 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %597)
  br label %825

599:                                              ; preds = %80
  %600 = load i32, ptr %8, align 4
  %601 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %600)
  br label %825

602:                                              ; preds = %80
  %603 = load i32, ptr %6, align 4
  %604 = lshr i32 %603, 7
  %605 = and i32 %604, 33554431
  %606 = sub nsw i32 %605, 16777215
  %607 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %606)
  %608 = load i32, ptr %6, align 4
  %609 = lshr i32 %608, 7
  %610 = and i32 %609, 33554431
  %611 = sub nsw i32 %610, 16777215
  %612 = load i32, ptr %4, align 4
  %613 = add nsw i32 %611, %612
  %614 = add nsw i32 %613, 2
  %615 = call i32 (ptr, ...) @printf(ptr noundef @.str.52, i32 noundef %614)
  br label %825

616:                                              ; preds = %80
  %617 = load i32, ptr %8, align 4
  %618 = load i32, ptr %9, align 4
  %619 = load i32, ptr %16, align 4
  %620 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %617, i32 noundef %618, i32 noundef %619)
  br label %825

621:                                              ; preds = %80
  %622 = load i32, ptr %8, align 4
  %623 = load i32, ptr %9, align 4
  %624 = load i32, ptr %16, align 4
  %625 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %622, i32 noundef %623, i32 noundef %624)
  br label %825

626:                                              ; preds = %80
  %627 = load i32, ptr %8, align 4
  %628 = load i32, ptr %9, align 4
  %629 = load i32, ptr %16, align 4
  %630 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %627, i32 noundef %628, i32 noundef %629)
  br label %825

631:                                              ; preds = %80
  %632 = load i32, ptr %8, align 4
  %633 = load i32, ptr %9, align 4
  %634 = load i32, ptr %16, align 4
  %635 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %632, i32 noundef %633, i32 noundef %634)
  %636 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %637 = load ptr, ptr %2, align 8
  %638 = load i32, ptr %9, align 4
  call void @PrintConstant(ptr noundef %637, i32 noundef %638)
  br label %825

639:                                              ; preds = %80
  %640 = load i32, ptr %8, align 4
  %641 = load i32, ptr %13, align 4
  %642 = load i32, ptr %16, align 4
  %643 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %640, i32 noundef %641, i32 noundef %642)
  br label %825

644:                                              ; preds = %80
  %645 = load i32, ptr %8, align 4
  %646 = load i32, ptr %13, align 4
  %647 = load i32, ptr %16, align 4
  %648 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %645, i32 noundef %646, i32 noundef %647)
  br label %825

649:                                              ; preds = %80
  %650 = load i32, ptr %8, align 4
  %651 = load i32, ptr %13, align 4
  %652 = load i32, ptr %16, align 4
  %653 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %650, i32 noundef %651, i32 noundef %652)
  br label %825

654:                                              ; preds = %80
  %655 = load i32, ptr %8, align 4
  %656 = load i32, ptr %13, align 4
  %657 = load i32, ptr %16, align 4
  %658 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %655, i32 noundef %656, i32 noundef %657)
  br label %825

659:                                              ; preds = %80
  %660 = load i32, ptr %8, align 4
  %661 = load i32, ptr %13, align 4
  %662 = load i32, ptr %16, align 4
  %663 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %660, i32 noundef %661, i32 noundef %662)
  br label %825

664:                                              ; preds = %80
  %665 = load i32, ptr %8, align 4
  %666 = load i32, ptr %16, align 4
  %667 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %665, i32 noundef %666)
  br label %825

668:                                              ; preds = %80
  %669 = load i32, ptr %8, align 4
  %670 = load i32, ptr %9, align 4
  %671 = load i32, ptr %16, align 4
  %672 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %669, i32 noundef %670, i32 noundef %671)
  br label %825

673:                                              ; preds = %80
  %674 = load i32, ptr %8, align 4
  %675 = load i32, ptr %9, align 4
  %676 = load i32, ptr %10, align 4
  %677 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %674, i32 noundef %675, i32 noundef %676)
  %678 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %679 = load i32, ptr %9, align 4
  %680 = icmp eq i32 %679, 0
  br i1 %680, label %681, label %683

681:                                              ; preds = %673
  %682 = call i32 (ptr, ...) @printf(ptr noundef @.str.53)
  br label %687

683:                                              ; preds = %673
  %684 = load i32, ptr %9, align 4
  %685 = sub nsw i32 %684, 1
  %686 = call i32 (ptr, ...) @printf(ptr noundef @.str.54, i32 noundef %685)
  br label %687

687:                                              ; preds = %683, %681
  %688 = load i32, ptr %10, align 4
  %689 = icmp eq i32 %688, 0
  br i1 %689, label %690, label %692

690:                                              ; preds = %687
  %691 = call i32 (ptr, ...) @printf(ptr noundef @.str.55)
  br label %696

692:                                              ; preds = %687
  %693 = load i32, ptr %10, align 4
  %694 = sub nsw i32 %693, 1
  %695 = call i32 (ptr, ...) @printf(ptr noundef @.str.56, i32 noundef %694)
  br label %696

696:                                              ; preds = %692, %690
  br label %825

697:                                              ; preds = %80
  %698 = load i32, ptr %8, align 4
  %699 = load i32, ptr %9, align 4
  %700 = load i32, ptr %10, align 4
  %701 = load i32, ptr %16, align 4
  %702 = icmp ne i32 %701, 0
  %703 = zext i1 %702 to i64
  %704 = select i1 %702, ptr @.str.47, ptr @.str.30
  %705 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %698, i32 noundef %699, i32 noundef %700, ptr noundef %704)
  %706 = load i32, ptr %9, align 4
  %707 = sub nsw i32 %706, 1
  %708 = call i32 (ptr, ...) @printf(ptr noundef @.str.57, i32 noundef %707)
  br label %825

709:                                              ; preds = %80
  %710 = load i32, ptr %8, align 4
  %711 = load i32, ptr %9, align 4
  %712 = load i32, ptr %10, align 4
  %713 = load i32, ptr %16, align 4
  %714 = icmp ne i32 %713, 0
  %715 = zext i1 %714 to i64
  %716 = select i1 %714, ptr @.str.47, ptr @.str.30
  %717 = call i32 (ptr, ...) @printf(ptr noundef @.str.46, i32 noundef %710, i32 noundef %711, i32 noundef %712, ptr noundef %716)
  %718 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %719 = load i32, ptr %9, align 4
  %720 = icmp eq i32 %719, 0
  br i1 %720, label %721, label %723

721:                                              ; preds = %709
  %722 = call i32 (ptr, ...) @printf(ptr noundef @.str.55)
  br label %727

723:                                              ; preds = %709
  %724 = load i32, ptr %9, align 4
  %725 = sub nsw i32 %724, 1
  %726 = call i32 (ptr, ...) @printf(ptr noundef @.str.56, i32 noundef %725)
  br label %727

727:                                              ; preds = %723, %721
  br label %825

728:                                              ; preds = %80
  br label %825

729:                                              ; preds = %80
  %730 = load i32, ptr %8, align 4
  %731 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %730)
  br label %825

732:                                              ; preds = %80
  %733 = load i32, ptr %8, align 4
  %734 = load i32, ptr %12, align 4
  %735 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %733, i32 noundef %734)
  %736 = load i32, ptr %4, align 4
  %737 = load i32, ptr %12, align 4
  %738 = sub nsw i32 %736, %737
  %739 = add nsw i32 %738, 2
  %740 = call i32 (ptr, ...) @printf(ptr noundef @.str.52, i32 noundef %739)
  br label %825

741:                                              ; preds = %80
  %742 = load i32, ptr %8, align 4
  %743 = load i32, ptr %12, align 4
  %744 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %742, i32 noundef %743)
  %745 = load i32, ptr %4, align 4
  %746 = load i32, ptr %12, align 4
  %747 = add nsw i32 %745, %746
  %748 = add nsw i32 %747, 3
  %749 = call i32 (ptr, ...) @printf(ptr noundef @.str.58, i32 noundef %748)
  br label %825

750:                                              ; preds = %80
  %751 = load i32, ptr %8, align 4
  %752 = load i32, ptr %12, align 4
  %753 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %751, i32 noundef %752)
  %754 = load i32, ptr %4, align 4
  %755 = load i32, ptr %12, align 4
  %756 = add nsw i32 %754, %755
  %757 = add nsw i32 %756, 2
  %758 = call i32 (ptr, ...) @printf(ptr noundef @.str.52, i32 noundef %757)
  br label %825

759:                                              ; preds = %80
  %760 = load i32, ptr %8, align 4
  %761 = load i32, ptr %10, align 4
  %762 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %760, i32 noundef %761)
  br label %825

763:                                              ; preds = %80
  %764 = load i32, ptr %8, align 4
  %765 = load i32, ptr %12, align 4
  %766 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %764, i32 noundef %765)
  %767 = load i32, ptr %4, align 4
  %768 = load i32, ptr %12, align 4
  %769 = sub nsw i32 %767, %768
  %770 = add nsw i32 %769, 2
  %771 = call i32 (ptr, ...) @printf(ptr noundef @.str.52, i32 noundef %770)
  br label %825

772:                                              ; preds = %80
  %773 = load i32, ptr %8, align 4
  %774 = load i32, ptr %9, align 4
  %775 = load i32, ptr %10, align 4
  %776 = call i32 (ptr, ...) @printf(ptr noundef @.str.44, i32 noundef %773, i32 noundef %774, i32 noundef %775)
  %777 = load i32, ptr %16, align 4
  %778 = icmp ne i32 %777, 0
  br i1 %778, label %779, label %792

779:                                              ; preds = %772
  %780 = load i32, ptr %10, align 4
  %781 = load ptr, ptr %3, align 8
  %782 = load i32, ptr %4, align 4
  %783 = add nsw i32 %782, 1
  %784 = sext i32 %783 to i64
  %785 = getelementptr inbounds i32, ptr %781, i64 %784
  %786 = load i32, ptr %785, align 4
  %787 = lshr i32 %786, 7
  %788 = and i32 %787, 33554431
  %789 = mul nsw i32 %788, 256
  %790 = add nsw i32 %780, %789
  %791 = call i32 (ptr, ...) @printf(ptr noundef @.str.48, i32 noundef %790)
  br label %792

792:                                              ; preds = %779, %772
  br label %825

793:                                              ; preds = %80
  %794 = load i32, ptr %8, align 4
  %795 = load i32, ptr %12, align 4
  %796 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %794, i32 noundef %795)
  %797 = load ptr, ptr %2, align 8
  %798 = getelementptr inbounds %struct.Proto, ptr %797, i32 0, i32 17
  %799 = load ptr, ptr %798, align 8
  %800 = load i32, ptr %12, align 4
  %801 = sext i32 %800 to i64
  %802 = getelementptr inbounds ptr, ptr %799, i64 %801
  %803 = load ptr, ptr %802, align 8
  %804 = call i32 (ptr, ...) @printf(ptr noundef @.str.59, ptr noundef %803)
  br label %825

805:                                              ; preds = %80
  %806 = load i32, ptr %8, align 4
  %807 = load i32, ptr %10, align 4
  %808 = call i32 (ptr, ...) @printf(ptr noundef @.str.39, i32 noundef %806, i32 noundef %807)
  %809 = call i32 (ptr, ...) @printf(ptr noundef @.str.40)
  %810 = load i32, ptr %10, align 4
  %811 = icmp eq i32 %810, 0
  br i1 %811, label %812, label %814

812:                                              ; preds = %805
  %813 = call i32 (ptr, ...) @printf(ptr noundef @.str.55)
  br label %818

814:                                              ; preds = %805
  %815 = load i32, ptr %10, align 4
  %816 = sub nsw i32 %815, 1
  %817 = call i32 (ptr, ...) @printf(ptr noundef @.str.56, i32 noundef %816)
  br label %818

818:                                              ; preds = %814, %812
  br label %825

819:                                              ; preds = %80
  %820 = load i32, ptr %8, align 4
  %821 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %820)
  br label %825

822:                                              ; preds = %80
  %823 = load i32, ptr %11, align 4
  %824 = call i32 (ptr, ...) @printf(ptr noundef @.str.41, i32 noundef %823)
  br label %825

825:                                              ; preds = %80, %822, %819, %818, %793, %792, %763, %759, %750, %741, %732, %729, %728, %727, %697, %696, %668, %664, %659, %654, %649, %644, %639, %631, %626, %621, %616, %602, %599, %596, %592, %588, %584, %580, %576, %575, %554, %523, %518, %513, %508, %503, %498, %493, %488, %483, %478, %473, %468, %463, %458, %453, %445, %437, %429, %421, %413, %405, %397, %389, %381, %373, %368, %367, %335, %334, %315, %299, %283, %233, %228, %223, %217, %188, %160, %128, %125, %122, %119, %106, %99, %95, %91, %87
  %826 = call i32 (ptr, ...) @printf(ptr noundef @.str.60)
  br label %827

827:                                              ; preds = %825
  %828 = load i32, ptr %4, align 4
  %829 = add nsw i32 %828, 1
  store i32 %829, ptr %4, align 4
  br label %24, !llvm.loop !11

830:                                              ; preds = %24
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintDebug(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = getelementptr inbounds %struct.Proto, ptr %5, i32 0, i32 7
  %7 = load i32, ptr %6, align 4
  store i32 %7, ptr %4, align 4
  %8 = load i32, ptr %4, align 4
  %9 = load ptr, ptr %2, align 8
  %10 = call i32 (ptr, ...) @printf(ptr noundef @.str.165, i32 noundef %8, ptr noundef %9)
  store i32 0, ptr %3, align 4
  br label %11

11:                                               ; preds = %23, %1
  %12 = load i32, ptr %3, align 4
  %13 = load i32, ptr %4, align 4
  %14 = icmp slt i32 %12, %13
  br i1 %14, label %15, label %26

15:                                               ; preds = %11
  %16 = load i32, ptr %3, align 4
  %17 = call i32 (ptr, ...) @printf(ptr noundef @.str.35, i32 noundef %16)
  %18 = load ptr, ptr %2, align 8
  %19 = load i32, ptr %3, align 4
  call void @PrintType(ptr noundef %18, i32 noundef %19)
  %20 = load ptr, ptr %2, align 8
  %21 = load i32, ptr %3, align 4
  call void @PrintConstant(ptr noundef %20, i32 noundef %21)
  %22 = call i32 (ptr, ...) @printf(ptr noundef @.str.60)
  br label %23

23:                                               ; preds = %15
  %24 = load i32, ptr %3, align 4
  %25 = add nsw i32 %24, 1
  store i32 %25, ptr %3, align 4
  br label %11, !llvm.loop !12

26:                                               ; preds = %11
  %27 = load ptr, ptr %2, align 8
  %28 = getelementptr inbounds %struct.Proto, ptr %27, i32 0, i32 11
  %29 = load i32, ptr %28, align 4
  store i32 %29, ptr %4, align 4
  %30 = load i32, ptr %4, align 4
  %31 = load ptr, ptr %2, align 8
  %32 = call i32 (ptr, ...) @printf(ptr noundef @.str.166, i32 noundef %30, ptr noundef %31)
  store i32 0, ptr %3, align 4
  br label %33

33:                                               ; preds = %68, %26
  %34 = load i32, ptr %3, align 4
  %35 = load i32, ptr %4, align 4
  %36 = icmp slt i32 %34, %35
  br i1 %36, label %37, label %71

37:                                               ; preds = %33
  %38 = load i32, ptr %3, align 4
  %39 = load ptr, ptr %2, align 8
  %40 = getelementptr inbounds %struct.Proto, ptr %39, i32 0, i32 21
  %41 = load ptr, ptr %40, align 8
  %42 = load i32, ptr %3, align 4
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds %struct.LocVar, ptr %41, i64 %43
  %45 = getelementptr inbounds %struct.LocVar, ptr %44, i32 0, i32 0
  %46 = load ptr, ptr %45, align 8
  %47 = getelementptr inbounds %struct.TString, ptr %46, i32 0, i32 7
  %48 = getelementptr inbounds [1 x i8], ptr %47, i64 0, i64 0
  %49 = load ptr, ptr %2, align 8
  %50 = getelementptr inbounds %struct.Proto, ptr %49, i32 0, i32 21
  %51 = load ptr, ptr %50, align 8
  %52 = load i32, ptr %3, align 4
  %53 = sext i32 %52 to i64
  %54 = getelementptr inbounds %struct.LocVar, ptr %51, i64 %53
  %55 = getelementptr inbounds %struct.LocVar, ptr %54, i32 0, i32 1
  %56 = load i32, ptr %55, align 8
  %57 = add nsw i32 %56, 1
  %58 = load ptr, ptr %2, align 8
  %59 = getelementptr inbounds %struct.Proto, ptr %58, i32 0, i32 21
  %60 = load ptr, ptr %59, align 8
  %61 = load i32, ptr %3, align 4
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds %struct.LocVar, ptr %60, i64 %62
  %64 = getelementptr inbounds %struct.LocVar, ptr %63, i32 0, i32 2
  %65 = load i32, ptr %64, align 4
  %66 = add nsw i32 %65, 1
  %67 = call i32 (ptr, ...) @printf(ptr noundef @.str.167, i32 noundef %38, ptr noundef %48, i32 noundef %57, i32 noundef %66)
  br label %68

68:                                               ; preds = %37
  %69 = load i32, ptr %3, align 4
  %70 = add nsw i32 %69, 1
  store i32 %70, ptr %3, align 4
  br label %33, !llvm.loop !13

71:                                               ; preds = %33
  %72 = load ptr, ptr %2, align 8
  %73 = getelementptr inbounds %struct.Proto, ptr %72, i32 0, i32 6
  %74 = load i32, ptr %73, align 8
  store i32 %74, ptr %4, align 4
  %75 = load i32, ptr %4, align 4
  %76 = load ptr, ptr %2, align 8
  %77 = call i32 (ptr, ...) @printf(ptr noundef @.str.168, i32 noundef %75, ptr noundef %76)
  store i32 0, ptr %3, align 4
  br label %78

78:                                               ; preds = %126, %71
  %79 = load i32, ptr %3, align 4
  %80 = load i32, ptr %4, align 4
  %81 = icmp slt i32 %79, %80
  br i1 %81, label %82, label %129

82:                                               ; preds = %78
  %83 = load i32, ptr %3, align 4
  %84 = load ptr, ptr %2, align 8
  %85 = getelementptr inbounds %struct.Proto, ptr %84, i32 0, i32 18
  %86 = load ptr, ptr %85, align 8
  %87 = load i32, ptr %3, align 4
  %88 = sext i32 %87 to i64
  %89 = getelementptr inbounds %struct.Upvaldesc, ptr %86, i64 %88
  %90 = getelementptr inbounds %struct.Upvaldesc, ptr %89, i32 0, i32 0
  %91 = load ptr, ptr %90, align 8
  %92 = icmp ne ptr %91, null
  br i1 %92, label %93, label %104

93:                                               ; preds = %82
  %94 = load ptr, ptr %2, align 8
  %95 = getelementptr inbounds %struct.Proto, ptr %94, i32 0, i32 18
  %96 = load ptr, ptr %95, align 8
  %97 = load i32, ptr %3, align 4
  %98 = sext i32 %97 to i64
  %99 = getelementptr inbounds %struct.Upvaldesc, ptr %96, i64 %98
  %100 = getelementptr inbounds %struct.Upvaldesc, ptr %99, i32 0, i32 0
  %101 = load ptr, ptr %100, align 8
  %102 = getelementptr inbounds %struct.TString, ptr %101, i32 0, i32 7
  %103 = getelementptr inbounds [1 x i8], ptr %102, i64 0, i64 0
  br label %105

104:                                              ; preds = %82
  br label %105

105:                                              ; preds = %104, %93
  %106 = phi ptr [ %103, %93 ], [ @.str.3, %104 ]
  %107 = load ptr, ptr %2, align 8
  %108 = getelementptr inbounds %struct.Proto, ptr %107, i32 0, i32 18
  %109 = load ptr, ptr %108, align 8
  %110 = load i32, ptr %3, align 4
  %111 = sext i32 %110 to i64
  %112 = getelementptr inbounds %struct.Upvaldesc, ptr %109, i64 %111
  %113 = getelementptr inbounds %struct.Upvaldesc, ptr %112, i32 0, i32 1
  %114 = load i8, ptr %113, align 8
  %115 = zext i8 %114 to i32
  %116 = load ptr, ptr %2, align 8
  %117 = getelementptr inbounds %struct.Proto, ptr %116, i32 0, i32 18
  %118 = load ptr, ptr %117, align 8
  %119 = load i32, ptr %3, align 4
  %120 = sext i32 %119 to i64
  %121 = getelementptr inbounds %struct.Upvaldesc, ptr %118, i64 %120
  %122 = getelementptr inbounds %struct.Upvaldesc, ptr %121, i32 0, i32 2
  %123 = load i8, ptr %122, align 1
  %124 = zext i8 %123 to i32
  %125 = call i32 (ptr, ...) @printf(ptr noundef @.str.167, i32 noundef %83, ptr noundef %106, i32 noundef %115, i32 noundef %124)
  br label %126

126:                                              ; preds = %105
  %127 = load i32, ptr %3, align 4
  %128 = add nsw i32 %127, 1
  store i32 %128, ptr %3, align 4
  br label %78, !llvm.loop !14

129:                                              ; preds = %78
  ret void
}

declare hidden i32 @luaG_getfuncline(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintConstant(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca [100 x i8], align 16
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.Proto, ptr %7, i32 0, i32 15
  %9 = load ptr, ptr %8, align 8
  %10 = load i32, ptr %4, align 4
  %11 = sext i32 %10 to i64
  %12 = getelementptr inbounds %struct.TValue, ptr %9, i64 %11
  store ptr %12, ptr %5, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 1
  %15 = load i8, ptr %14, align 8
  %16 = zext i8 %15 to i32
  %17 = and i32 %16, 63
  switch i32 %17, label %50 [
    i32 0, label %18
    i32 1, label %20
    i32 17, label %22
    i32 19, label %24
    i32 3, label %41
    i32 4, label %46
    i32 20, label %46
  ]

18:                                               ; preds = %2
  %19 = call i32 (ptr, ...) @printf(ptr noundef @.str.144)
  br label %57

20:                                               ; preds = %2
  %21 = call i32 (ptr, ...) @printf(ptr noundef @.str.145)
  br label %57

22:                                               ; preds = %2
  %23 = call i32 (ptr, ...) @printf(ptr noundef @.str.146)
  br label %57

24:                                               ; preds = %2
  %25 = getelementptr inbounds [100 x i8], ptr %6, i64 0, i64 0
  %26 = load ptr, ptr %5, align 8
  %27 = getelementptr inbounds %struct.TValue, ptr %26, i32 0, i32 0
  %28 = load double, ptr %27, align 8
  %29 = call i32 (ptr, ptr, ...) @sprintf(ptr noundef %25, ptr noundef @.str.147, double noundef %28) #8
  %30 = getelementptr inbounds [100 x i8], ptr %6, i64 0, i64 0
  %31 = call i32 (ptr, ...) @printf(ptr noundef @.str.148, ptr noundef %30)
  %32 = getelementptr inbounds [100 x i8], ptr %6, i64 0, i64 0
  %33 = call i64 @strspn(ptr noundef %32, ptr noundef @.str.149) #6
  %34 = getelementptr inbounds [100 x i8], ptr %6, i64 0, i64 %33
  %35 = load i8, ptr %34, align 1
  %36 = sext i8 %35 to i32
  %37 = icmp eq i32 %36, 0
  br i1 %37, label %38, label %40

38:                                               ; preds = %24
  %39 = call i32 (ptr, ...) @printf(ptr noundef @.str.150)
  br label %40

40:                                               ; preds = %38, %24
  br label %57

41:                                               ; preds = %2
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load i64, ptr %43, align 8
  %45 = call i32 (ptr, ...) @printf(ptr noundef @.str.151, i64 noundef %44)
  br label %57

46:                                               ; preds = %2, %2
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %struct.TValue, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  call void @PrintString(ptr noundef %49)
  br label %57

50:                                               ; preds = %2
  %51 = load ptr, ptr %5, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 1
  %53 = load i8, ptr %52, align 8
  %54 = zext i8 %53 to i32
  %55 = and i32 %54, 63
  %56 = call i32 (ptr, ...) @printf(ptr noundef @.str.152, i32 noundef %55)
  br label %57

57:                                               ; preds = %50, %46, %41, %40, %22, %20, %18
  ret void
}

; Function Attrs: nounwind
declare i32 @sprintf(ptr noundef, ptr noundef, ...) #4

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strspn(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintString(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.TString, ptr %7, i32 0, i32 7
  %9 = getelementptr inbounds [1 x i8], ptr %8, i64 0, i64 0
  store ptr %9, ptr %3, align 8
  %10 = load ptr, ptr %2, align 8
  %11 = getelementptr inbounds %struct.TString, ptr %10, i32 0, i32 4
  %12 = load i8, ptr %11, align 1
  %13 = zext i8 %12 to i32
  %14 = icmp ne i32 %13, 255
  br i1 %14, label %15, label %20

15:                                               ; preds = %1
  %16 = load ptr, ptr %2, align 8
  %17 = getelementptr inbounds %struct.TString, ptr %16, i32 0, i32 4
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i64
  br label %24

20:                                               ; preds = %1
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds %struct.TString, ptr %21, i32 0, i32 6
  %23 = load i64, ptr %22, align 8
  br label %24

24:                                               ; preds = %20, %15
  %25 = phi i64 [ %19, %15 ], [ %23, %20 ]
  store i64 %25, ptr %5, align 8
  %26 = call i32 (ptr, ...) @printf(ptr noundef @.str.153)
  store i64 0, ptr %4, align 8
  br label %27

27:                                               ; preds = %74, %24
  %28 = load i64, ptr %4, align 8
  %29 = load i64, ptr %5, align 8
  %30 = icmp ult i64 %28, %29
  br i1 %30, label %31, label %77

31:                                               ; preds = %27
  %32 = load ptr, ptr %3, align 8
  %33 = load i64, ptr %4, align 8
  %34 = getelementptr inbounds i8, ptr %32, i64 %33
  %35 = load i8, ptr %34, align 1
  %36 = zext i8 %35 to i32
  store i32 %36, ptr %6, align 4
  %37 = load i32, ptr %6, align 4
  switch i32 %37, label %56 [
    i32 34, label %38
    i32 92, label %40
    i32 7, label %42
    i32 8, label %44
    i32 12, label %46
    i32 10, label %48
    i32 13, label %50
    i32 9, label %52
    i32 11, label %54
  ]

38:                                               ; preds = %31
  %39 = call i32 (ptr, ...) @printf(ptr noundef @.str.154)
  br label %73

40:                                               ; preds = %31
  %41 = call i32 (ptr, ...) @printf(ptr noundef @.str.155)
  br label %73

42:                                               ; preds = %31
  %43 = call i32 (ptr, ...) @printf(ptr noundef @.str.156)
  br label %73

44:                                               ; preds = %31
  %45 = call i32 (ptr, ...) @printf(ptr noundef @.str.157)
  br label %73

46:                                               ; preds = %31
  %47 = call i32 (ptr, ...) @printf(ptr noundef @.str.158)
  br label %73

48:                                               ; preds = %31
  %49 = call i32 (ptr, ...) @printf(ptr noundef @.str.159)
  br label %73

50:                                               ; preds = %31
  %51 = call i32 (ptr, ...) @printf(ptr noundef @.str.160)
  br label %73

52:                                               ; preds = %31
  %53 = call i32 (ptr, ...) @printf(ptr noundef @.str.161)
  br label %73

54:                                               ; preds = %31
  %55 = call i32 (ptr, ...) @printf(ptr noundef @.str.162)
  br label %73

56:                                               ; preds = %31
  %57 = call ptr @__ctype_b_loc() #9
  %58 = load ptr, ptr %57, align 8
  %59 = load i32, ptr %6, align 4
  %60 = sext i32 %59 to i64
  %61 = getelementptr inbounds i16, ptr %58, i64 %60
  %62 = load i16, ptr %61, align 2
  %63 = zext i16 %62 to i32
  %64 = and i32 %63, 16384
  %65 = icmp ne i32 %64, 0
  br i1 %65, label %66, label %69

66:                                               ; preds = %56
  %67 = load i32, ptr %6, align 4
  %68 = call i32 (ptr, ...) @printf(ptr noundef @.str.163, i32 noundef %67)
  br label %72

69:                                               ; preds = %56
  %70 = load i32, ptr %6, align 4
  %71 = call i32 (ptr, ...) @printf(ptr noundef @.str.164, i32 noundef %70)
  br label %72

72:                                               ; preds = %69, %66
  br label %73

73:                                               ; preds = %72, %54, %52, %50, %48, %46, %44, %42, %40, %38
  br label %74

74:                                               ; preds = %73
  %75 = load i64, ptr %4, align 8
  %76 = add i64 %75, 1
  store i64 %76, ptr %4, align 8
  br label %27, !llvm.loop !15

77:                                               ; preds = %27
  %78 = call i32 (ptr, ...) @printf(ptr noundef @.str.153)
  ret void
}

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__ctype_b_loc() #5

; Function Attrs: noinline nounwind optnone uwtable
define internal void @PrintType(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.Proto, ptr %6, i32 0, i32 15
  %8 = load ptr, ptr %7, align 8
  %9 = load i32, ptr %4, align 4
  %10 = sext i32 %9 to i64
  %11 = getelementptr inbounds %struct.TValue, ptr %8, i64 %10
  store ptr %11, ptr %5, align 8
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 63
  switch i32 %16, label %27 [
    i32 0, label %17
    i32 1, label %19
    i32 17, label %19
    i32 19, label %21
    i32 3, label %23
    i32 4, label %25
    i32 20, label %25
  ]

17:                                               ; preds = %2
  %18 = call i32 (ptr, ...) @printf(ptr noundef @.str.169)
  br label %34

19:                                               ; preds = %2, %2
  %20 = call i32 (ptr, ...) @printf(ptr noundef @.str.170)
  br label %34

21:                                               ; preds = %2
  %22 = call i32 (ptr, ...) @printf(ptr noundef @.str.171)
  br label %34

23:                                               ; preds = %2
  %24 = call i32 (ptr, ...) @printf(ptr noundef @.str.172)
  br label %34

25:                                               ; preds = %2, %2
  %26 = call i32 (ptr, ...) @printf(ptr noundef @.str.173)
  br label %34

27:                                               ; preds = %2
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  %30 = load i8, ptr %29, align 8
  %31 = zext i8 %30 to i32
  %32 = and i32 %31, 63
  %33 = call i32 (ptr, ...) @printf(ptr noundef @.str.152, i32 noundef %32)
  br label %34

34:                                               ; preds = %27, %25, %23, %21, %19, %17
  %35 = call i32 (ptr, ...) @printf(ptr noundef @.str.174)
  ret void
}

; Function Attrs: nounwind
declare ptr @strerror(i32 noundef) #4

; Function Attrs: nounwind willreturn memory(none)
declare ptr @__errno_location() #5

declare i64 @fwrite(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { noreturn nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nounwind willreturn memory(none) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { nounwind willreturn memory(read) }
attributes #7 = { noreturn nounwind }
attributes #8 = { nounwind }
attributes #9 = { nounwind willreturn memory(none) }

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
