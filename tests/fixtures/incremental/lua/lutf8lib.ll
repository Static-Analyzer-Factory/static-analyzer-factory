; ModuleID = 'lutf8lib.c'
source_filename = "lutf8lib.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.luaL_Reg = type { ptr, ptr }
%struct.luaL_Buffer = type { ptr, i64, i64, ptr, %union.anon }
%union.anon = type { double, [1016 x i8] }

@funcs = internal constant [7 x %struct.luaL_Reg] [%struct.luaL_Reg { ptr @.str.2, ptr @byteoffset }, %struct.luaL_Reg { ptr @.str.3, ptr @codepoint }, %struct.luaL_Reg { ptr @.str.4, ptr @utfchar }, %struct.luaL_Reg { ptr @.str.5, ptr @utflen }, %struct.luaL_Reg { ptr @.str.6, ptr @iter_codes }, %struct.luaL_Reg { ptr @.str.1, ptr null }, %struct.luaL_Reg zeroinitializer], align 16
@.str = private unnamed_addr constant [15 x i8] c"[\00-\7F\C2-\FD][\80-\BF]*\00", align 1
@.str.1 = private unnamed_addr constant [12 x i8] c"charpattern\00", align 1
@.str.2 = private unnamed_addr constant [7 x i8] c"offset\00", align 1
@.str.3 = private unnamed_addr constant [10 x i8] c"codepoint\00", align 1
@.str.4 = private unnamed_addr constant [5 x i8] c"char\00", align 1
@.str.5 = private unnamed_addr constant [4 x i8] c"len\00", align 1
@.str.6 = private unnamed_addr constant [6 x i8] c"codes\00", align 1
@.str.7 = private unnamed_addr constant [23 x i8] c"position out of bounds\00", align 1
@.str.8 = private unnamed_addr constant [40 x i8] c"initial position is a continuation byte\00", align 1
@.str.9 = private unnamed_addr constant [14 x i8] c"out of bounds\00", align 1
@.str.10 = private unnamed_addr constant [22 x i8] c"string slice too long\00", align 1
@.str.11 = private unnamed_addr constant [19 x i8] c"invalid UTF-8 code\00", align 1
@utf8_decode.limits = internal constant [6 x i32] [i32 -1, i32 128, i32 2048, i32 65536, i32 2097152, i32 67108864], align 16
@.str.12 = private unnamed_addr constant [19 x i8] c"value out of range\00", align 1
@.str.13 = private unnamed_addr constant [3 x i8] c"%U\00", align 1
@.str.14 = private unnamed_addr constant [31 x i8] c"initial position out of bounds\00", align 1
@.str.15 = private unnamed_addr constant [29 x i8] c"final position out of bounds\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @luaopen_utf8(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void @luaL_checkversion_(ptr noundef %3, double noundef 5.040000e+02, i64 noundef 136)
  %4 = load ptr, ptr %2, align 8
  call void @lua_createtable(ptr noundef %4, i32 noundef 0, i32 noundef 6)
  %5 = load ptr, ptr %2, align 8
  call void @luaL_setfuncs(ptr noundef %5, ptr noundef @funcs, i32 noundef 0)
  %6 = load ptr, ptr %2, align 8
  %7 = call ptr @lua_pushlstring(ptr noundef %6, ptr noundef @.str, i64 noundef 14)
  %8 = load ptr, ptr %2, align 8
  call void @lua_setfield(ptr noundef %8, i32 noundef -2, ptr noundef @.str.1)
  ret i32 1
}

declare void @luaL_checkversion_(ptr noundef, double noundef, i64 noundef) #1

declare void @lua_createtable(ptr noundef, i32 noundef, i32 noundef) #1

declare void @luaL_setfuncs(ptr noundef, ptr noundef, i32 noundef) #1

declare ptr @lua_pushlstring(ptr noundef, ptr noundef, i64 noundef) #1

declare void @lua_setfield(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @byteoffset(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store ptr %0, ptr %3, align 8
  %8 = load ptr, ptr %3, align 8
  %9 = call ptr @luaL_checklstring(ptr noundef %8, i32 noundef 1, ptr noundef %4)
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = call i64 @luaL_checkinteger(ptr noundef %10, i32 noundef 2)
  store i64 %11, ptr %6, align 8
  %12 = load i64, ptr %6, align 8
  %13 = icmp sge i64 %12, 0
  br i1 %13, label %14, label %15

14:                                               ; preds = %1
  br label %18

15:                                               ; preds = %1
  %16 = load i64, ptr %4, align 8
  %17 = add i64 %16, 1
  br label %18

18:                                               ; preds = %15, %14
  %19 = phi i64 [ 1, %14 ], [ %17, %15 ]
  store i64 %19, ptr %7, align 8
  %20 = load ptr, ptr %3, align 8
  %21 = load i64, ptr %7, align 8
  %22 = call i64 @luaL_optinteger(ptr noundef %20, i32 noundef 3, i64 noundef %21)
  %23 = load i64, ptr %4, align 8
  %24 = call i64 @u_posrelat(i64 noundef %22, i64 noundef %23)
  store i64 %24, ptr %7, align 8
  %25 = load i64, ptr %7, align 8
  %26 = icmp sle i64 1, %25
  br i1 %26, label %27, label %32

27:                                               ; preds = %18
  %28 = load i64, ptr %7, align 8
  %29 = add nsw i64 %28, -1
  store i64 %29, ptr %7, align 8
  %30 = load i64, ptr %4, align 8
  %31 = icmp sle i64 %29, %30
  br label %32

32:                                               ; preds = %27, %18
  %33 = phi i1 [ false, %18 ], [ %31, %27 ]
  %34 = zext i1 %33 to i32
  %35 = icmp ne i32 %34, 0
  %36 = zext i1 %35 to i32
  %37 = sext i32 %36 to i64
  %38 = icmp ne i64 %37, 0
  br i1 %38, label %43, label %39

39:                                               ; preds = %32
  %40 = load ptr, ptr %3, align 8
  %41 = call i32 @luaL_argerror(ptr noundef %40, i32 noundef 3, ptr noundef @.str.7)
  %42 = icmp ne i32 %41, 0
  br label %43

43:                                               ; preds = %39, %32
  %44 = phi i1 [ true, %32 ], [ %42, %39 ]
  %45 = zext i1 %44 to i32
  %46 = load i64, ptr %6, align 8
  %47 = icmp eq i64 %46, 0
  br i1 %47, label %48, label %66

48:                                               ; preds = %43
  br label %49

49:                                               ; preds = %62, %48
  %50 = load i64, ptr %7, align 8
  %51 = icmp sgt i64 %50, 0
  br i1 %51, label %52, label %60

52:                                               ; preds = %49
  %53 = load ptr, ptr %5, align 8
  %54 = load i64, ptr %7, align 8
  %55 = getelementptr inbounds i8, ptr %53, i64 %54
  %56 = load i8, ptr %55, align 1
  %57 = sext i8 %56 to i32
  %58 = and i32 %57, 192
  %59 = icmp eq i32 %58, 128
  br label %60

60:                                               ; preds = %52, %49
  %61 = phi i1 [ false, %49 ], [ %59, %52 ]
  br i1 %61, label %62, label %65

62:                                               ; preds = %60
  %63 = load i64, ptr %7, align 8
  %64 = add nsw i64 %63, -1
  store i64 %64, ptr %7, align 8
  br label %49, !llvm.loop !6

65:                                               ; preds = %60
  br label %139

66:                                               ; preds = %43
  %67 = load ptr, ptr %5, align 8
  %68 = load i64, ptr %7, align 8
  %69 = getelementptr inbounds i8, ptr %67, i64 %68
  %70 = load i8, ptr %69, align 1
  %71 = sext i8 %70 to i32
  %72 = and i32 %71, 192
  %73 = icmp eq i32 %72, 128
  br i1 %73, label %74, label %77

74:                                               ; preds = %66
  %75 = load ptr, ptr %3, align 8
  %76 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %75, ptr noundef @.str.8)
  store i32 %76, ptr %2, align 4
  br label %149

77:                                               ; preds = %66
  %78 = load i64, ptr %6, align 8
  %79 = icmp slt i64 %78, 0
  br i1 %79, label %80, label %110

80:                                               ; preds = %77
  br label %81

81:                                               ; preds = %106, %80
  %82 = load i64, ptr %6, align 8
  %83 = icmp slt i64 %82, 0
  br i1 %83, label %84, label %87

84:                                               ; preds = %81
  %85 = load i64, ptr %7, align 8
  %86 = icmp sgt i64 %85, 0
  br label %87

87:                                               ; preds = %84, %81
  %88 = phi i1 [ false, %81 ], [ %86, %84 ]
  br i1 %88, label %89, label %109

89:                                               ; preds = %87
  br label %90

90:                                               ; preds = %104, %89
  %91 = load i64, ptr %7, align 8
  %92 = add nsw i64 %91, -1
  store i64 %92, ptr %7, align 8
  br label %93

93:                                               ; preds = %90
  %94 = load i64, ptr %7, align 8
  %95 = icmp sgt i64 %94, 0
  br i1 %95, label %96, label %104

96:                                               ; preds = %93
  %97 = load ptr, ptr %5, align 8
  %98 = load i64, ptr %7, align 8
  %99 = getelementptr inbounds i8, ptr %97, i64 %98
  %100 = load i8, ptr %99, align 1
  %101 = sext i8 %100 to i32
  %102 = and i32 %101, 192
  %103 = icmp eq i32 %102, 128
  br label %104

104:                                              ; preds = %96, %93
  %105 = phi i1 [ false, %93 ], [ %103, %96 ]
  br i1 %105, label %90, label %106, !llvm.loop !8

106:                                              ; preds = %104
  %107 = load i64, ptr %6, align 8
  %108 = add nsw i64 %107, 1
  store i64 %108, ptr %6, align 8
  br label %81, !llvm.loop !9

109:                                              ; preds = %87
  br label %138

110:                                              ; preds = %77
  %111 = load i64, ptr %6, align 8
  %112 = add nsw i64 %111, -1
  store i64 %112, ptr %6, align 8
  br label %113

113:                                              ; preds = %134, %110
  %114 = load i64, ptr %6, align 8
  %115 = icmp sgt i64 %114, 0
  br i1 %115, label %116, label %120

116:                                              ; preds = %113
  %117 = load i64, ptr %7, align 8
  %118 = load i64, ptr %4, align 8
  %119 = icmp slt i64 %117, %118
  br label %120

120:                                              ; preds = %116, %113
  %121 = phi i1 [ false, %113 ], [ %119, %116 ]
  br i1 %121, label %122, label %137

122:                                              ; preds = %120
  br label %123

123:                                              ; preds = %126, %122
  %124 = load i64, ptr %7, align 8
  %125 = add nsw i64 %124, 1
  store i64 %125, ptr %7, align 8
  br label %126

126:                                              ; preds = %123
  %127 = load ptr, ptr %5, align 8
  %128 = load i64, ptr %7, align 8
  %129 = getelementptr inbounds i8, ptr %127, i64 %128
  %130 = load i8, ptr %129, align 1
  %131 = sext i8 %130 to i32
  %132 = and i32 %131, 192
  %133 = icmp eq i32 %132, 128
  br i1 %133, label %123, label %134, !llvm.loop !10

134:                                              ; preds = %126
  %135 = load i64, ptr %6, align 8
  %136 = add nsw i64 %135, -1
  store i64 %136, ptr %6, align 8
  br label %113, !llvm.loop !11

137:                                              ; preds = %120
  br label %138

138:                                              ; preds = %137, %109
  br label %139

139:                                              ; preds = %138, %65
  %140 = load i64, ptr %6, align 8
  %141 = icmp eq i64 %140, 0
  br i1 %141, label %142, label %146

142:                                              ; preds = %139
  %143 = load ptr, ptr %3, align 8
  %144 = load i64, ptr %7, align 8
  %145 = add nsw i64 %144, 1
  call void @lua_pushinteger(ptr noundef %143, i64 noundef %145)
  br label %148

146:                                              ; preds = %139
  %147 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %147)
  br label %148

148:                                              ; preds = %146, %142
  store i32 1, ptr %2, align 4
  br label %149

149:                                              ; preds = %148, %74
  %150 = load i32, ptr %2, align 4
  ret i32 %150
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @codepoint(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  %12 = load ptr, ptr %3, align 8
  %13 = call ptr @luaL_checklstring(ptr noundef %12, i32 noundef 1, ptr noundef %4)
  store ptr %13, ptr %5, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = call i64 @luaL_optinteger(ptr noundef %14, i32 noundef 2, i64 noundef 1)
  %16 = load i64, ptr %4, align 8
  %17 = call i64 @u_posrelat(i64 noundef %15, i64 noundef %16)
  store i64 %17, ptr %6, align 8
  %18 = load ptr, ptr %3, align 8
  %19 = load i64, ptr %6, align 8
  %20 = call i64 @luaL_optinteger(ptr noundef %18, i32 noundef 3, i64 noundef %19)
  %21 = load i64, ptr %4, align 8
  %22 = call i64 @u_posrelat(i64 noundef %20, i64 noundef %21)
  store i64 %22, ptr %7, align 8
  %23 = load ptr, ptr %3, align 8
  %24 = call i32 @lua_toboolean(ptr noundef %23, i32 noundef 4)
  store i32 %24, ptr %8, align 4
  %25 = load i64, ptr %6, align 8
  %26 = icmp sge i64 %25, 1
  %27 = zext i1 %26 to i32
  %28 = icmp ne i32 %27, 0
  %29 = zext i1 %28 to i32
  %30 = sext i32 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %36, label %32

32:                                               ; preds = %1
  %33 = load ptr, ptr %3, align 8
  %34 = call i32 @luaL_argerror(ptr noundef %33, i32 noundef 2, ptr noundef @.str.9)
  %35 = icmp ne i32 %34, 0
  br label %36

36:                                               ; preds = %32, %1
  %37 = phi i1 [ true, %1 ], [ %35, %32 ]
  %38 = zext i1 %37 to i32
  %39 = load i64, ptr %7, align 8
  %40 = load i64, ptr %4, align 8
  %41 = icmp sle i64 %39, %40
  %42 = zext i1 %41 to i32
  %43 = icmp ne i32 %42, 0
  %44 = zext i1 %43 to i32
  %45 = sext i32 %44 to i64
  %46 = icmp ne i64 %45, 0
  br i1 %46, label %51, label %47

47:                                               ; preds = %36
  %48 = load ptr, ptr %3, align 8
  %49 = call i32 @luaL_argerror(ptr noundef %48, i32 noundef 3, ptr noundef @.str.9)
  %50 = icmp ne i32 %49, 0
  br label %51

51:                                               ; preds = %47, %36
  %52 = phi i1 [ true, %36 ], [ %50, %47 ]
  %53 = zext i1 %52 to i32
  %54 = load i64, ptr %6, align 8
  %55 = load i64, ptr %7, align 8
  %56 = icmp sgt i64 %54, %55
  br i1 %56, label %57, label %58

57:                                               ; preds = %51
  store i32 0, ptr %2, align 4
  br label %105

58:                                               ; preds = %51
  %59 = load i64, ptr %7, align 8
  %60 = load i64, ptr %6, align 8
  %61 = sub nsw i64 %59, %60
  %62 = icmp sge i64 %61, 2147483647
  br i1 %62, label %63, label %66

63:                                               ; preds = %58
  %64 = load ptr, ptr %3, align 8
  %65 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %64, ptr noundef @.str.10)
  store i32 %65, ptr %2, align 4
  br label %105

66:                                               ; preds = %58
  %67 = load i64, ptr %7, align 8
  %68 = load i64, ptr %6, align 8
  %69 = sub nsw i64 %67, %68
  %70 = trunc i64 %69 to i32
  %71 = add nsw i32 %70, 1
  store i32 %71, ptr %9, align 4
  %72 = load ptr, ptr %3, align 8
  %73 = load i32, ptr %9, align 4
  call void @luaL_checkstack(ptr noundef %72, i32 noundef %73, ptr noundef @.str.10)
  store i32 0, ptr %9, align 4
  %74 = load ptr, ptr %5, align 8
  %75 = load i64, ptr %7, align 8
  %76 = getelementptr inbounds i8, ptr %74, i64 %75
  store ptr %76, ptr %10, align 8
  %77 = load i64, ptr %6, align 8
  %78 = sub nsw i64 %77, 1
  %79 = load ptr, ptr %5, align 8
  %80 = getelementptr inbounds i8, ptr %79, i64 %78
  store ptr %80, ptr %5, align 8
  br label %81

81:                                               ; preds = %97, %66
  %82 = load ptr, ptr %5, align 8
  %83 = load ptr, ptr %10, align 8
  %84 = icmp ult ptr %82, %83
  br i1 %84, label %85, label %103

85:                                               ; preds = %81
  %86 = load ptr, ptr %5, align 8
  %87 = load i32, ptr %8, align 4
  %88 = icmp ne i32 %87, 0
  %89 = xor i1 %88, true
  %90 = zext i1 %89 to i32
  %91 = call ptr @utf8_decode(ptr noundef %86, ptr noundef %11, i32 noundef %90)
  store ptr %91, ptr %5, align 8
  %92 = load ptr, ptr %5, align 8
  %93 = icmp eq ptr %92, null
  br i1 %93, label %94, label %97

94:                                               ; preds = %85
  %95 = load ptr, ptr %3, align 8
  %96 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %95, ptr noundef @.str.11)
  store i32 %96, ptr %2, align 4
  br label %105

97:                                               ; preds = %85
  %98 = load ptr, ptr %3, align 8
  %99 = load i32, ptr %11, align 4
  %100 = zext i32 %99 to i64
  call void @lua_pushinteger(ptr noundef %98, i64 noundef %100)
  %101 = load i32, ptr %9, align 4
  %102 = add nsw i32 %101, 1
  store i32 %102, ptr %9, align 4
  br label %81, !llvm.loop !12

103:                                              ; preds = %81
  %104 = load i32, ptr %9, align 4
  store i32 %104, ptr %2, align 4
  br label %105

105:                                              ; preds = %103, %94, %63, %57
  %106 = load i32, ptr %2, align 4
  ret i32 %106
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @utfchar(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca %struct.luaL_Buffer, align 8
  store ptr %0, ptr %2, align 8
  %6 = load ptr, ptr %2, align 8
  %7 = call i32 @lua_gettop(ptr noundef %6)
  store i32 %7, ptr %3, align 4
  %8 = load i32, ptr %3, align 4
  %9 = icmp eq i32 %8, 1
  br i1 %9, label %10, label %12

10:                                               ; preds = %1
  %11 = load ptr, ptr %2, align 8
  call void @pushutfchar(ptr noundef %11, i32 noundef 1)
  br label %25

12:                                               ; preds = %1
  %13 = load ptr, ptr %2, align 8
  call void @luaL_buffinit(ptr noundef %13, ptr noundef %5)
  store i32 1, ptr %4, align 4
  br label %14

14:                                               ; preds = %21, %12
  %15 = load i32, ptr %4, align 4
  %16 = load i32, ptr %3, align 4
  %17 = icmp sle i32 %15, %16
  br i1 %17, label %18, label %24

18:                                               ; preds = %14
  %19 = load ptr, ptr %2, align 8
  %20 = load i32, ptr %4, align 4
  call void @pushutfchar(ptr noundef %19, i32 noundef %20)
  call void @luaL_addvalue(ptr noundef %5)
  br label %21

21:                                               ; preds = %18
  %22 = load i32, ptr %4, align 4
  %23 = add nsw i32 %22, 1
  store i32 %23, ptr %4, align 4
  br label %14, !llvm.loop !13

24:                                               ; preds = %14
  call void @luaL_pushresult(ptr noundef %5)
  br label %25

25:                                               ; preds = %24, %10
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @utflen(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i64 0, ptr %4, align 8
  %11 = load ptr, ptr %3, align 8
  %12 = call ptr @luaL_checklstring(ptr noundef %11, i32 noundef 1, ptr noundef %5)
  store ptr %12, ptr %6, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = call i64 @luaL_optinteger(ptr noundef %13, i32 noundef 2, i64 noundef 1)
  %15 = load i64, ptr %5, align 8
  %16 = call i64 @u_posrelat(i64 noundef %14, i64 noundef %15)
  store i64 %16, ptr %7, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = call i64 @luaL_optinteger(ptr noundef %17, i32 noundef 3, i64 noundef -1)
  %19 = load i64, ptr %5, align 8
  %20 = call i64 @u_posrelat(i64 noundef %18, i64 noundef %19)
  store i64 %20, ptr %8, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = call i32 @lua_toboolean(ptr noundef %21, i32 noundef 4)
  store i32 %22, ptr %9, align 4
  %23 = load i64, ptr %7, align 8
  %24 = icmp sle i64 1, %23
  br i1 %24, label %25, label %30

25:                                               ; preds = %1
  %26 = load i64, ptr %7, align 8
  %27 = add nsw i64 %26, -1
  store i64 %27, ptr %7, align 8
  %28 = load i64, ptr %5, align 8
  %29 = icmp sle i64 %27, %28
  br label %30

30:                                               ; preds = %25, %1
  %31 = phi i1 [ false, %1 ], [ %29, %25 ]
  %32 = zext i1 %31 to i32
  %33 = icmp ne i32 %32, 0
  %34 = zext i1 %33 to i32
  %35 = sext i32 %34 to i64
  %36 = icmp ne i64 %35, 0
  br i1 %36, label %41, label %37

37:                                               ; preds = %30
  %38 = load ptr, ptr %3, align 8
  %39 = call i32 @luaL_argerror(ptr noundef %38, i32 noundef 2, ptr noundef @.str.14)
  %40 = icmp ne i32 %39, 0
  br label %41

41:                                               ; preds = %37, %30
  %42 = phi i1 [ true, %30 ], [ %40, %37 ]
  %43 = zext i1 %42 to i32
  %44 = load i64, ptr %8, align 8
  %45 = add nsw i64 %44, -1
  store i64 %45, ptr %8, align 8
  %46 = load i64, ptr %5, align 8
  %47 = icmp slt i64 %45, %46
  %48 = zext i1 %47 to i32
  %49 = icmp ne i32 %48, 0
  %50 = zext i1 %49 to i32
  %51 = sext i32 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %57, label %53

53:                                               ; preds = %41
  %54 = load ptr, ptr %3, align 8
  %55 = call i32 @luaL_argerror(ptr noundef %54, i32 noundef 3, ptr noundef @.str.15)
  %56 = icmp ne i32 %55, 0
  br label %57

57:                                               ; preds = %53, %41
  %58 = phi i1 [ true, %41 ], [ %56, %53 ]
  %59 = zext i1 %58 to i32
  br label %60

60:                                               ; preds = %80, %57
  %61 = load i64, ptr %7, align 8
  %62 = load i64, ptr %8, align 8
  %63 = icmp sle i64 %61, %62
  br i1 %63, label %64, label %88

64:                                               ; preds = %60
  %65 = load ptr, ptr %6, align 8
  %66 = load i64, ptr %7, align 8
  %67 = getelementptr inbounds i8, ptr %65, i64 %66
  %68 = load i32, ptr %9, align 4
  %69 = icmp ne i32 %68, 0
  %70 = xor i1 %69, true
  %71 = zext i1 %70 to i32
  %72 = call ptr @utf8_decode(ptr noundef %67, ptr noundef null, i32 noundef %71)
  store ptr %72, ptr %10, align 8
  %73 = load ptr, ptr %10, align 8
  %74 = icmp eq ptr %73, null
  br i1 %74, label %75, label %80

75:                                               ; preds = %64
  %76 = load ptr, ptr %3, align 8
  call void @lua_pushnil(ptr noundef %76)
  %77 = load ptr, ptr %3, align 8
  %78 = load i64, ptr %7, align 8
  %79 = add nsw i64 %78, 1
  call void @lua_pushinteger(ptr noundef %77, i64 noundef %79)
  store i32 2, ptr %2, align 4
  br label %91

80:                                               ; preds = %64
  %81 = load ptr, ptr %10, align 8
  %82 = load ptr, ptr %6, align 8
  %83 = ptrtoint ptr %81 to i64
  %84 = ptrtoint ptr %82 to i64
  %85 = sub i64 %83, %84
  store i64 %85, ptr %7, align 8
  %86 = load i64, ptr %4, align 8
  %87 = add nsw i64 %86, 1
  store i64 %87, ptr %4, align 8
  br label %60, !llvm.loop !14

88:                                               ; preds = %60
  %89 = load ptr, ptr %3, align 8
  %90 = load i64, ptr %4, align 8
  call void @lua_pushinteger(ptr noundef %89, i64 noundef %90)
  store i32 1, ptr %2, align 4
  br label %91

91:                                               ; preds = %88, %75
  %92 = load i32, ptr %2, align 4
  ret i32 %92
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @iter_codes(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %5 = load ptr, ptr %2, align 8
  %6 = call i32 @lua_toboolean(ptr noundef %5, i32 noundef 2)
  store i32 %6, ptr %3, align 4
  %7 = load ptr, ptr %2, align 8
  %8 = call ptr @luaL_checklstring(ptr noundef %7, i32 noundef 1, ptr noundef null)
  store ptr %8, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load i8, ptr %9, align 1
  %11 = sext i8 %10 to i32
  %12 = and i32 %11, 192
  %13 = icmp eq i32 %12, 128
  %14 = xor i1 %13, true
  %15 = zext i1 %14 to i32
  %16 = icmp ne i32 %15, 0
  %17 = zext i1 %16 to i32
  %18 = sext i32 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %24, label %20

20:                                               ; preds = %1
  %21 = load ptr, ptr %2, align 8
  %22 = call i32 @luaL_argerror(ptr noundef %21, i32 noundef 1, ptr noundef @.str.11)
  %23 = icmp ne i32 %22, 0
  br label %24

24:                                               ; preds = %20, %1
  %25 = phi i1 [ true, %1 ], [ %23, %20 ]
  %26 = zext i1 %25 to i32
  %27 = load ptr, ptr %2, align 8
  %28 = load i32, ptr %3, align 4
  %29 = icmp ne i32 %28, 0
  %30 = zext i1 %29 to i64
  %31 = select i1 %29, ptr @iter_auxlax, ptr @iter_auxstrict
  call void @lua_pushcclosure(ptr noundef %27, ptr noundef %31, i32 noundef 0)
  %32 = load ptr, ptr %2, align 8
  call void @lua_pushvalue(ptr noundef %32, i32 noundef 1)
  %33 = load ptr, ptr %2, align 8
  call void @lua_pushinteger(ptr noundef %33, i64 noundef 0)
  ret i32 3
}

declare ptr @luaL_checklstring(ptr noundef, i32 noundef, ptr noundef) #1

declare i64 @luaL_checkinteger(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @u_posrelat(i64 noundef %0, i64 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  store i64 %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %6 = load i64, ptr %4, align 8
  %7 = icmp sge i64 %6, 0
  br i1 %7, label %8, label %10

8:                                                ; preds = %2
  %9 = load i64, ptr %4, align 8
  store i64 %9, ptr %3, align 8
  br label %21

10:                                               ; preds = %2
  %11 = load i64, ptr %4, align 8
  %12 = sub i64 0, %11
  %13 = load i64, ptr %5, align 8
  %14 = icmp ugt i64 %12, %13
  br i1 %14, label %15, label %16

15:                                               ; preds = %10
  store i64 0, ptr %3, align 8
  br label %21

16:                                               ; preds = %10
  %17 = load i64, ptr %5, align 8
  %18 = load i64, ptr %4, align 8
  %19 = add nsw i64 %17, %18
  %20 = add nsw i64 %19, 1
  store i64 %20, ptr %3, align 8
  br label %21

21:                                               ; preds = %16, %15, %8
  %22 = load i64, ptr %3, align 8
  ret i64 %22
}

declare i64 @luaL_optinteger(ptr noundef, i32 noundef, i64 noundef) #1

declare i32 @luaL_argerror(ptr noundef, i32 noundef, ptr noundef) #1

declare i32 @luaL_error(ptr noundef, ptr noundef, ...) #1

declare void @lua_pushinteger(ptr noundef, i64 noundef) #1

declare void @lua_pushnil(ptr noundef) #1

declare i32 @lua_toboolean(ptr noundef, i32 noundef) #1

declare void @luaL_checkstack(ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @utf8_decode(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds i8, ptr %12, i64 0
  %14 = load i8, ptr %13, align 1
  %15 = zext i8 %14 to i32
  store i32 %15, ptr %8, align 4
  store i32 0, ptr %9, align 4
  %16 = load i32, ptr %8, align 4
  %17 = icmp ult i32 %16, 128
  br i1 %17, label %18, label %20

18:                                               ; preds = %3
  %19 = load i32, ptr %8, align 4
  store i32 %19, ptr %9, align 4
  br label %72

20:                                               ; preds = %3
  store i32 0, ptr %10, align 4
  br label %21

21:                                               ; preds = %43, %20
  %22 = load i32, ptr %8, align 4
  %23 = and i32 %22, 64
  %24 = icmp ne i32 %23, 0
  br i1 %24, label %25, label %46

25:                                               ; preds = %21
  %26 = load ptr, ptr %5, align 8
  %27 = load i32, ptr %10, align 4
  %28 = add nsw i32 %27, 1
  store i32 %28, ptr %10, align 4
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds i8, ptr %26, i64 %29
  %31 = load i8, ptr %30, align 1
  %32 = zext i8 %31 to i32
  store i32 %32, ptr %11, align 4
  %33 = load i32, ptr %11, align 4
  %34 = and i32 %33, 192
  %35 = icmp eq i32 %34, 128
  br i1 %35, label %37, label %36

36:                                               ; preds = %25
  store ptr null, ptr %4, align 8
  br label %95

37:                                               ; preds = %25
  %38 = load i32, ptr %9, align 4
  %39 = shl i32 %38, 6
  %40 = load i32, ptr %11, align 4
  %41 = and i32 %40, 63
  %42 = or i32 %39, %41
  store i32 %42, ptr %9, align 4
  br label %43

43:                                               ; preds = %37
  %44 = load i32, ptr %8, align 4
  %45 = shl i32 %44, 1
  store i32 %45, ptr %8, align 4
  br label %21, !llvm.loop !15

46:                                               ; preds = %21
  %47 = load i32, ptr %8, align 4
  %48 = and i32 %47, 127
  %49 = load i32, ptr %10, align 4
  %50 = mul nsw i32 %49, 5
  %51 = shl i32 %48, %50
  %52 = load i32, ptr %9, align 4
  %53 = or i32 %52, %51
  store i32 %53, ptr %9, align 4
  %54 = load i32, ptr %10, align 4
  %55 = icmp sgt i32 %54, 5
  br i1 %55, label %66, label %56

56:                                               ; preds = %46
  %57 = load i32, ptr %9, align 4
  %58 = icmp ugt i32 %57, 2147483647
  br i1 %58, label %66, label %59

59:                                               ; preds = %56
  %60 = load i32, ptr %9, align 4
  %61 = load i32, ptr %10, align 4
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds [6 x i32], ptr @utf8_decode.limits, i64 0, i64 %62
  %64 = load i32, ptr %63, align 4
  %65 = icmp ult i32 %60, %64
  br i1 %65, label %66, label %67

66:                                               ; preds = %59, %56, %46
  store ptr null, ptr %4, align 8
  br label %95

67:                                               ; preds = %59
  %68 = load i32, ptr %10, align 4
  %69 = load ptr, ptr %5, align 8
  %70 = sext i32 %68 to i64
  %71 = getelementptr inbounds i8, ptr %69, i64 %70
  store ptr %71, ptr %5, align 8
  br label %72

72:                                               ; preds = %67, %18
  %73 = load i32, ptr %7, align 4
  %74 = icmp ne i32 %73, 0
  br i1 %74, label %75, label %86

75:                                               ; preds = %72
  %76 = load i32, ptr %9, align 4
  %77 = icmp ugt i32 %76, 1114111
  br i1 %77, label %84, label %78

78:                                               ; preds = %75
  %79 = load i32, ptr %9, align 4
  %80 = icmp ule i32 55296, %79
  br i1 %80, label %81, label %85

81:                                               ; preds = %78
  %82 = load i32, ptr %9, align 4
  %83 = icmp ule i32 %82, 57343
  br i1 %83, label %84, label %85

84:                                               ; preds = %81, %75
  store ptr null, ptr %4, align 8
  br label %95

85:                                               ; preds = %81, %78
  br label %86

86:                                               ; preds = %85, %72
  %87 = load ptr, ptr %6, align 8
  %88 = icmp ne ptr %87, null
  br i1 %88, label %89, label %92

89:                                               ; preds = %86
  %90 = load i32, ptr %9, align 4
  %91 = load ptr, ptr %6, align 8
  store i32 %90, ptr %91, align 4
  br label %92

92:                                               ; preds = %89, %86
  %93 = load ptr, ptr %5, align 8
  %94 = getelementptr inbounds i8, ptr %93, i64 1
  store ptr %94, ptr %4, align 8
  br label %95

95:                                               ; preds = %92, %84, %66, %36
  %96 = load ptr, ptr %4, align 8
  ret ptr %96
}

declare i32 @lua_gettop(ptr noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pushutfchar(ptr noundef %0, i32 noundef %1) #0 {
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
  %10 = icmp ule i64 %9, 2147483647
  %11 = zext i1 %10 to i32
  %12 = icmp ne i32 %11, 0
  %13 = zext i1 %12 to i32
  %14 = sext i32 %13 to i64
  %15 = icmp ne i64 %14, 0
  br i1 %15, label %21, label %16

16:                                               ; preds = %2
  %17 = load ptr, ptr %3, align 8
  %18 = load i32, ptr %4, align 4
  %19 = call i32 @luaL_argerror(ptr noundef %17, i32 noundef %18, ptr noundef @.str.12)
  %20 = icmp ne i32 %19, 0
  br label %21

21:                                               ; preds = %16, %2
  %22 = phi i1 [ true, %2 ], [ %20, %16 ]
  %23 = zext i1 %22 to i32
  %24 = load ptr, ptr %3, align 8
  %25 = load i64, ptr %5, align 8
  %26 = call ptr (ptr, ptr, ...) @lua_pushfstring(ptr noundef %24, ptr noundef @.str.13, i64 noundef %25)
  ret void
}

declare void @luaL_buffinit(ptr noundef, ptr noundef) #1

declare void @luaL_addvalue(ptr noundef) #1

declare void @luaL_pushresult(ptr noundef) #1

declare ptr @lua_pushfstring(ptr noundef, ptr noundef, ...) #1

declare void @lua_pushcclosure(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @iter_auxlax(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @iter_aux(ptr noundef %3, i32 noundef 0)
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @iter_auxstrict(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 @iter_aux(ptr noundef %3, i32 noundef 1)
  ret i32 %4
}

declare void @lua_pushvalue(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @iter_aux(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i32, align 4
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @luaL_checklstring(ptr noundef %11, i32 noundef 1, ptr noundef %6)
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = call i64 @lua_tointegerx(ptr noundef %13, i32 noundef 2, ptr noundef null)
  store i64 %14, ptr %8, align 8
  %15 = load i64, ptr %8, align 8
  %16 = load i64, ptr %6, align 8
  %17 = icmp ult i64 %15, %16
  br i1 %17, label %18, label %31

18:                                               ; preds = %2
  br label %19

19:                                               ; preds = %27, %18
  %20 = load ptr, ptr %7, align 8
  %21 = load i64, ptr %8, align 8
  %22 = getelementptr inbounds i8, ptr %20, i64 %21
  %23 = load i8, ptr %22, align 1
  %24 = sext i8 %23 to i32
  %25 = and i32 %24, 192
  %26 = icmp eq i32 %25, 128
  br i1 %26, label %27, label %30

27:                                               ; preds = %19
  %28 = load i64, ptr %8, align 8
  %29 = add i64 %28, 1
  store i64 %29, ptr %8, align 8
  br label %19, !llvm.loop !16

30:                                               ; preds = %19
  br label %31

31:                                               ; preds = %30, %2
  %32 = load i64, ptr %8, align 8
  %33 = load i64, ptr %6, align 8
  %34 = icmp uge i64 %32, %33
  br i1 %34, label %35, label %36

35:                                               ; preds = %31
  store i32 0, ptr %3, align 4
  br label %60

36:                                               ; preds = %31
  %37 = load ptr, ptr %7, align 8
  %38 = load i64, ptr %8, align 8
  %39 = getelementptr inbounds i8, ptr %37, i64 %38
  %40 = load i32, ptr %5, align 4
  %41 = call ptr @utf8_decode(ptr noundef %39, ptr noundef %9, i32 noundef %40)
  store ptr %41, ptr %10, align 8
  %42 = load ptr, ptr %10, align 8
  %43 = icmp eq ptr %42, null
  br i1 %43, label %50, label %44

44:                                               ; preds = %36
  %45 = load ptr, ptr %10, align 8
  %46 = load i8, ptr %45, align 1
  %47 = sext i8 %46 to i32
  %48 = and i32 %47, 192
  %49 = icmp eq i32 %48, 128
  br i1 %49, label %50, label %53

50:                                               ; preds = %44, %36
  %51 = load ptr, ptr %4, align 8
  %52 = call i32 (ptr, ptr, ...) @luaL_error(ptr noundef %51, ptr noundef @.str.11)
  store i32 %52, ptr %3, align 4
  br label %60

53:                                               ; preds = %44
  %54 = load ptr, ptr %4, align 8
  %55 = load i64, ptr %8, align 8
  %56 = add i64 %55, 1
  call void @lua_pushinteger(ptr noundef %54, i64 noundef %56)
  %57 = load ptr, ptr %4, align 8
  %58 = load i32, ptr %9, align 4
  %59 = zext i32 %58 to i64
  call void @lua_pushinteger(ptr noundef %57, i64 noundef %59)
  store i32 2, ptr %3, align 4
  br label %60

60:                                               ; preds = %53, %50, %35
  %61 = load i32, ptr %3, align 4
  ret i32 %61
}

declare i64 @lua_tointegerx(ptr noundef, i32 noundef, ptr noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

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
