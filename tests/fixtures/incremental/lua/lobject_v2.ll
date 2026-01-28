; ModuleID = 'lobject.c'
source_filename = "lobject.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.lconv = type { ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8 }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.BuffFS = type { ptr, i32, i32, [199 x i8] }
%struct.__va_list_tag = type { i32, i32, ptr, ptr }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon.0, %union.anon.3, i16, i16 }
%union.anon.0 = type { %struct.anon.2 }
%struct.anon.2 = type { ptr, i64, i64 }
%union.anon.3 = type { i32 }
%union.StackValue = type { %struct.TValue }

@luaO_ceillog2.log_2 = internal constant [256 x i8] c"\00\01\02\02\03\03\03\03\04\04\04\04\04\04\04\04\05\05\05\05\05\05\05\05\05\05\05\05\05\05\05\05\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\06\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\07\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08\08", align 16
@luai_ctype_ = external hidden constant [257 x i8], align 16
@luaO_str2num.hook = internal global ptr null, align 8
@.str = private unnamed_addr constant [7 x i8] c"(null)\00", align 1
@.str.1 = private unnamed_addr constant [3 x i8] c"%p\00", align 1
@.str.2 = private unnamed_addr constant [2 x i8] c"%\00", align 1
@.str.3 = private unnamed_addr constant [43 x i8] c"invalid option '%%%c' to 'lua_pushfstring'\00", align 1
@.str.4 = private unnamed_addr constant [4 x i8] c"...\00", align 1
@.str.5 = private unnamed_addr constant [10 x i8] c"[string \22\00", align 1
@.str.6 = private unnamed_addr constant [3 x i8] c"\22]\00", align 1
@.str.7 = private unnamed_addr constant [6 x i8] c".xXnN\00", align 1
@.str.8 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.str.9 = private unnamed_addr constant [6 x i8] c"%.14g\00", align 1
@.str.10 = private unnamed_addr constant [12 x i8] c"-0123456789\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaO_ceillog2(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  store i32 0, ptr %3, align 4
  %4 = load i32, ptr %2, align 4
  %5 = add i32 %4, -1
  store i32 %5, ptr %2, align 4
  br label %6

6:                                                ; preds = %9, %1
  %7 = load i32, ptr %2, align 4
  %8 = icmp uge i32 %7, 256
  br i1 %8, label %9, label %14

9:                                                ; preds = %6
  %10 = load i32, ptr %3, align 4
  %11 = add nsw i32 %10, 8
  store i32 %11, ptr %3, align 4
  %12 = load i32, ptr %2, align 4
  %13 = lshr i32 %12, 8
  store i32 %13, ptr %2, align 4
  br label %6, !llvm.loop !6

14:                                               ; preds = %6
  %15 = load i32, ptr %3, align 4
  %16 = load i32, ptr %2, align 4
  %17 = zext i32 %16 to i64
  %18 = getelementptr inbounds [256 x i8], ptr @luaO_ceillog2.log_2, i64 0, i64 %17
  %19 = load i8, ptr %18, align 1
  %20 = zext i8 %19 to i32
  %21 = add nsw i32 %15, %20
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaO_rawarith(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  %12 = alloca i64, align 8
  %13 = alloca i64, align 8
  %14 = alloca ptr, align 8
  %15 = alloca double, align 8
  %16 = alloca double, align 8
  %17 = alloca ptr, align 8
  %18 = alloca double, align 8
  %19 = alloca double, align 8
  %20 = alloca ptr, align 8
  %21 = alloca ptr, align 8
  store ptr %0, ptr %7, align 8
  store i32 %1, ptr %8, align 4
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store ptr %4, ptr %11, align 8
  %22 = load i32, ptr %8, align 4
  switch i32 %22, label %129 [
    i32 7, label %23
    i32 8, label %23
    i32 9, label %23
    i32 10, label %23
    i32 11, label %23
    i32 13, label %23
    i32 5, label %73
    i32 4, label %73
  ]

23:                                               ; preds = %5, %5, %5, %5, %5, %5
  %24 = load ptr, ptr %9, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = zext i8 %26 to i32
  %28 = icmp eq i32 %27, 3
  %29 = zext i1 %28 to i32
  %30 = icmp ne i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %34, label %38

34:                                               ; preds = %23
  %35 = load ptr, ptr %9, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 0
  %37 = load i64, ptr %36, align 8
  store i64 %37, ptr %12, align 8
  br i1 true, label %42, label %72

38:                                               ; preds = %23
  %39 = load ptr, ptr %9, align 8
  %40 = call i32 @luaV_tointegerns(ptr noundef %39, ptr noundef %12, i32 noundef 0)
  %41 = icmp ne i32 %40, 0
  br i1 %41, label %42, label %72

42:                                               ; preds = %38, %34
  %43 = load ptr, ptr %10, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  %45 = load i8, ptr %44, align 8
  %46 = zext i8 %45 to i32
  %47 = icmp eq i32 %46, 3
  %48 = zext i1 %47 to i32
  %49 = icmp ne i32 %48, 0
  %50 = zext i1 %49 to i32
  %51 = sext i32 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %53, label %57

53:                                               ; preds = %42
  %54 = load ptr, ptr %10, align 8
  %55 = getelementptr inbounds %struct.TValue, ptr %54, i32 0, i32 0
  %56 = load i64, ptr %55, align 8
  store i64 %56, ptr %13, align 8
  br i1 true, label %61, label %72

57:                                               ; preds = %42
  %58 = load ptr, ptr %10, align 8
  %59 = call i32 @luaV_tointegerns(ptr noundef %58, ptr noundef %13, i32 noundef 0)
  %60 = icmp ne i32 %59, 0
  br i1 %60, label %61, label %72

61:                                               ; preds = %57, %53
  %62 = load ptr, ptr %11, align 8
  store ptr %62, ptr %14, align 8
  %63 = load ptr, ptr %7, align 8
  %64 = load i32, ptr %8, align 4
  %65 = load i64, ptr %12, align 8
  %66 = load i64, ptr %13, align 8
  %67 = call i64 @intarith(ptr noundef %63, i32 noundef %64, i64 noundef %65, i64 noundef %66)
  %68 = load ptr, ptr %14, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  store i64 %67, ptr %69, align 8
  %70 = load ptr, ptr %14, align 8
  %71 = getelementptr inbounds %struct.TValue, ptr %70, i32 0, i32 1
  store i8 3, ptr %71, align 8
  store i32 1, ptr %6, align 4
  br label %212

72:                                               ; preds = %57, %53, %38, %34
  store i32 0, ptr %6, align 4
  br label %212

73:                                               ; preds = %5, %5
  %74 = load ptr, ptr %9, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 1
  %76 = load i8, ptr %75, align 8
  %77 = zext i8 %76 to i32
  %78 = icmp eq i32 %77, 19
  br i1 %78, label %79, label %83

79:                                               ; preds = %73
  %80 = load ptr, ptr %9, align 8
  %81 = getelementptr inbounds %struct.TValue, ptr %80, i32 0, i32 0
  %82 = load double, ptr %81, align 8
  store double %82, ptr %15, align 8
  br i1 true, label %95, label %128

83:                                               ; preds = %73
  %84 = load ptr, ptr %9, align 8
  %85 = getelementptr inbounds %struct.TValue, ptr %84, i32 0, i32 1
  %86 = load i8, ptr %85, align 8
  %87 = zext i8 %86 to i32
  %88 = icmp eq i32 %87, 3
  br i1 %88, label %89, label %94

89:                                               ; preds = %83
  %90 = load ptr, ptr %9, align 8
  %91 = getelementptr inbounds %struct.TValue, ptr %90, i32 0, i32 0
  %92 = load i64, ptr %91, align 8
  %93 = sitofp i64 %92 to double
  store double %93, ptr %15, align 8
  br i1 true, label %95, label %128

94:                                               ; preds = %83
  br i1 false, label %95, label %128

95:                                               ; preds = %94, %89, %79
  %96 = load ptr, ptr %10, align 8
  %97 = getelementptr inbounds %struct.TValue, ptr %96, i32 0, i32 1
  %98 = load i8, ptr %97, align 8
  %99 = zext i8 %98 to i32
  %100 = icmp eq i32 %99, 19
  br i1 %100, label %101, label %105

101:                                              ; preds = %95
  %102 = load ptr, ptr %10, align 8
  %103 = getelementptr inbounds %struct.TValue, ptr %102, i32 0, i32 0
  %104 = load double, ptr %103, align 8
  store double %104, ptr %16, align 8
  br i1 true, label %117, label %128

105:                                              ; preds = %95
  %106 = load ptr, ptr %10, align 8
  %107 = getelementptr inbounds %struct.TValue, ptr %106, i32 0, i32 1
  %108 = load i8, ptr %107, align 8
  %109 = zext i8 %108 to i32
  %110 = icmp eq i32 %109, 3
  br i1 %110, label %111, label %116

111:                                              ; preds = %105
  %112 = load ptr, ptr %10, align 8
  %113 = getelementptr inbounds %struct.TValue, ptr %112, i32 0, i32 0
  %114 = load i64, ptr %113, align 8
  %115 = sitofp i64 %114 to double
  store double %115, ptr %16, align 8
  br i1 true, label %117, label %128

116:                                              ; preds = %105
  br i1 false, label %117, label %128

117:                                              ; preds = %116, %111, %101
  %118 = load ptr, ptr %11, align 8
  store ptr %118, ptr %17, align 8
  %119 = load ptr, ptr %7, align 8
  %120 = load i32, ptr %8, align 4
  %121 = load double, ptr %15, align 8
  %122 = load double, ptr %16, align 8
  %123 = call double @numarith(ptr noundef %119, i32 noundef %120, double noundef %121, double noundef %122)
  %124 = load ptr, ptr %17, align 8
  %125 = getelementptr inbounds %struct.TValue, ptr %124, i32 0, i32 0
  store double %123, ptr %125, align 8
  %126 = load ptr, ptr %17, align 8
  %127 = getelementptr inbounds %struct.TValue, ptr %126, i32 0, i32 1
  store i8 19, ptr %127, align 8
  store i32 1, ptr %6, align 4
  br label %212

128:                                              ; preds = %116, %111, %101, %94, %89, %79
  store i32 0, ptr %6, align 4
  br label %212

129:                                              ; preds = %5
  %130 = load ptr, ptr %9, align 8
  %131 = getelementptr inbounds %struct.TValue, ptr %130, i32 0, i32 1
  %132 = load i8, ptr %131, align 8
  %133 = zext i8 %132 to i32
  %134 = icmp eq i32 %133, 3
  br i1 %134, label %135, label %156

135:                                              ; preds = %129
  %136 = load ptr, ptr %10, align 8
  %137 = getelementptr inbounds %struct.TValue, ptr %136, i32 0, i32 1
  %138 = load i8, ptr %137, align 8
  %139 = zext i8 %138 to i32
  %140 = icmp eq i32 %139, 3
  br i1 %140, label %141, label %156

141:                                              ; preds = %135
  %142 = load ptr, ptr %11, align 8
  store ptr %142, ptr %20, align 8
  %143 = load ptr, ptr %7, align 8
  %144 = load i32, ptr %8, align 4
  %145 = load ptr, ptr %9, align 8
  %146 = getelementptr inbounds %struct.TValue, ptr %145, i32 0, i32 0
  %147 = load i64, ptr %146, align 8
  %148 = load ptr, ptr %10, align 8
  %149 = getelementptr inbounds %struct.TValue, ptr %148, i32 0, i32 0
  %150 = load i64, ptr %149, align 8
  %151 = call i64 @intarith(ptr noundef %143, i32 noundef %144, i64 noundef %147, i64 noundef %150)
  %152 = load ptr, ptr %20, align 8
  %153 = getelementptr inbounds %struct.TValue, ptr %152, i32 0, i32 0
  store i64 %151, ptr %153, align 8
  %154 = load ptr, ptr %20, align 8
  %155 = getelementptr inbounds %struct.TValue, ptr %154, i32 0, i32 1
  store i8 3, ptr %155, align 8
  store i32 1, ptr %6, align 4
  br label %212

156:                                              ; preds = %135, %129
  %157 = load ptr, ptr %9, align 8
  %158 = getelementptr inbounds %struct.TValue, ptr %157, i32 0, i32 1
  %159 = load i8, ptr %158, align 8
  %160 = zext i8 %159 to i32
  %161 = icmp eq i32 %160, 19
  br i1 %161, label %162, label %166

162:                                              ; preds = %156
  %163 = load ptr, ptr %9, align 8
  %164 = getelementptr inbounds %struct.TValue, ptr %163, i32 0, i32 0
  %165 = load double, ptr %164, align 8
  store double %165, ptr %18, align 8
  br i1 true, label %178, label %211

166:                                              ; preds = %156
  %167 = load ptr, ptr %9, align 8
  %168 = getelementptr inbounds %struct.TValue, ptr %167, i32 0, i32 1
  %169 = load i8, ptr %168, align 8
  %170 = zext i8 %169 to i32
  %171 = icmp eq i32 %170, 3
  br i1 %171, label %172, label %177

172:                                              ; preds = %166
  %173 = load ptr, ptr %9, align 8
  %174 = getelementptr inbounds %struct.TValue, ptr %173, i32 0, i32 0
  %175 = load i64, ptr %174, align 8
  %176 = sitofp i64 %175 to double
  store double %176, ptr %18, align 8
  br i1 true, label %178, label %211

177:                                              ; preds = %166
  br i1 false, label %178, label %211

178:                                              ; preds = %177, %172, %162
  %179 = load ptr, ptr %10, align 8
  %180 = getelementptr inbounds %struct.TValue, ptr %179, i32 0, i32 1
  %181 = load i8, ptr %180, align 8
  %182 = zext i8 %181 to i32
  %183 = icmp eq i32 %182, 19
  br i1 %183, label %184, label %188

184:                                              ; preds = %178
  %185 = load ptr, ptr %10, align 8
  %186 = getelementptr inbounds %struct.TValue, ptr %185, i32 0, i32 0
  %187 = load double, ptr %186, align 8
  store double %187, ptr %19, align 8
  br i1 true, label %200, label %211

188:                                              ; preds = %178
  %189 = load ptr, ptr %10, align 8
  %190 = getelementptr inbounds %struct.TValue, ptr %189, i32 0, i32 1
  %191 = load i8, ptr %190, align 8
  %192 = zext i8 %191 to i32
  %193 = icmp eq i32 %192, 3
  br i1 %193, label %194, label %199

194:                                              ; preds = %188
  %195 = load ptr, ptr %10, align 8
  %196 = getelementptr inbounds %struct.TValue, ptr %195, i32 0, i32 0
  %197 = load i64, ptr %196, align 8
  %198 = sitofp i64 %197 to double
  store double %198, ptr %19, align 8
  br i1 true, label %200, label %211

199:                                              ; preds = %188
  br i1 false, label %200, label %211

200:                                              ; preds = %199, %194, %184
  %201 = load ptr, ptr %11, align 8
  store ptr %201, ptr %21, align 8
  %202 = load ptr, ptr %7, align 8
  %203 = load i32, ptr %8, align 4
  %204 = load double, ptr %18, align 8
  %205 = load double, ptr %19, align 8
  %206 = call double @numarith(ptr noundef %202, i32 noundef %203, double noundef %204, double noundef %205)
  %207 = load ptr, ptr %21, align 8
  %208 = getelementptr inbounds %struct.TValue, ptr %207, i32 0, i32 0
  store double %206, ptr %208, align 8
  %209 = load ptr, ptr %21, align 8
  %210 = getelementptr inbounds %struct.TValue, ptr %209, i32 0, i32 1
  store i8 19, ptr %210, align 8
  store i32 1, ptr %6, align 4
  br label %212

211:                                              ; preds = %199, %194, %184, %177, %172, %162
  store i32 0, ptr %6, align 4
  br label %212

212:                                              ; preds = %211, %200, %141, %128, %117, %72, %61
  %213 = load i32, ptr %6, align 4
  ret i32 %213
}

declare hidden i32 @luaV_tointegerns(ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i64 @intarith(ptr noundef %0, i32 noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca i64, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store i64 %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %10 = load i32, ptr %7, align 4
  switch i32 %10, label %60 [
    i32 0, label %11
    i32 1, label %15
    i32 2, label %19
    i32 3, label %23
    i32 6, label %28
    i32 7, label %33
    i32 8, label %37
    i32 9, label %41
    i32 10, label %45
    i32 11, label %49
    i32 12, label %54
    i32 13, label %57
  ]

11:                                               ; preds = %4
  %12 = load i64, ptr %8, align 8
  %13 = load i64, ptr %9, align 8
  %14 = add i64 %12, %13
  store i64 %14, ptr %5, align 8
  br label %61

15:                                               ; preds = %4
  %16 = load i64, ptr %8, align 8
  %17 = load i64, ptr %9, align 8
  %18 = sub i64 %16, %17
  store i64 %18, ptr %5, align 8
  br label %61

19:                                               ; preds = %4
  %20 = load i64, ptr %8, align 8
  %21 = load i64, ptr %9, align 8
  %22 = mul i64 %20, %21
  store i64 %22, ptr %5, align 8
  br label %61

23:                                               ; preds = %4
  %24 = load ptr, ptr %6, align 8
  %25 = load i64, ptr %8, align 8
  %26 = load i64, ptr %9, align 8
  %27 = call i64 @luaV_mod(ptr noundef %24, i64 noundef %25, i64 noundef %26)
  store i64 %27, ptr %5, align 8
  br label %61

28:                                               ; preds = %4
  %29 = load ptr, ptr %6, align 8
  %30 = load i64, ptr %8, align 8
  %31 = load i64, ptr %9, align 8
  %32 = call i64 @luaV_idiv(ptr noundef %29, i64 noundef %30, i64 noundef %31)
  store i64 %32, ptr %5, align 8
  br label %61

33:                                               ; preds = %4
  %34 = load i64, ptr %8, align 8
  %35 = load i64, ptr %9, align 8
  %36 = and i64 %34, %35
  store i64 %36, ptr %5, align 8
  br label %61

37:                                               ; preds = %4
  %38 = load i64, ptr %8, align 8
  %39 = load i64, ptr %9, align 8
  %40 = or i64 %38, %39
  store i64 %40, ptr %5, align 8
  br label %61

41:                                               ; preds = %4
  %42 = load i64, ptr %8, align 8
  %43 = load i64, ptr %9, align 8
  %44 = xor i64 %42, %43
  store i64 %44, ptr %5, align 8
  br label %61

45:                                               ; preds = %4
  %46 = load i64, ptr %8, align 8
  %47 = load i64, ptr %9, align 8
  %48 = call i64 @luaV_shiftl(i64 noundef %46, i64 noundef %47)
  store i64 %48, ptr %5, align 8
  br label %61

49:                                               ; preds = %4
  %50 = load i64, ptr %8, align 8
  %51 = load i64, ptr %9, align 8
  %52 = sub i64 0, %51
  %53 = call i64 @luaV_shiftl(i64 noundef %50, i64 noundef %52)
  store i64 %53, ptr %5, align 8
  br label %61

54:                                               ; preds = %4
  %55 = load i64, ptr %8, align 8
  %56 = sub i64 0, %55
  store i64 %56, ptr %5, align 8
  br label %61

57:                                               ; preds = %4
  %58 = load i64, ptr %8, align 8
  %59 = xor i64 -1, %58
  store i64 %59, ptr %5, align 8
  br label %61

60:                                               ; preds = %4
  store i64 0, ptr %5, align 8
  br label %61

61:                                               ; preds = %60, %57, %54, %49, %45, %41, %37, %33, %28, %23, %19, %15, %11
  %62 = load i64, ptr %5, align 8
  ret i64 %62
}

; Function Attrs: noinline nounwind optnone uwtable
define internal double @numarith(ptr noundef %0, i32 noundef %1, double noundef %2, double noundef %3) #0 {
  %5 = alloca double, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca double, align 8
  %9 = alloca double, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store double %2, ptr %8, align 8
  store double %3, ptr %9, align 8
  %10 = load i32, ptr %7, align 4
  switch i32 %10, label %55 [
    i32 0, label %11
    i32 1, label %15
    i32 2, label %19
    i32 5, label %23
    i32 4, label %27
    i32 6, label %41
    i32 12, label %47
    i32 3, label %50
  ]

11:                                               ; preds = %4
  %12 = load double, ptr %8, align 8
  %13 = load double, ptr %9, align 8
  %14 = fadd double %12, %13
  store double %14, ptr %5, align 8
  br label %56

15:                                               ; preds = %4
  %16 = load double, ptr %8, align 8
  %17 = load double, ptr %9, align 8
  %18 = fsub double %16, %17
  store double %18, ptr %5, align 8
  br label %56

19:                                               ; preds = %4
  %20 = load double, ptr %8, align 8
  %21 = load double, ptr %9, align 8
  %22 = fmul double %20, %21
  store double %22, ptr %5, align 8
  br label %56

23:                                               ; preds = %4
  %24 = load double, ptr %8, align 8
  %25 = load double, ptr %9, align 8
  %26 = fdiv double %24, %25
  store double %26, ptr %5, align 8
  br label %56

27:                                               ; preds = %4
  %28 = load ptr, ptr %6, align 8
  %29 = load double, ptr %9, align 8
  %30 = fcmp oeq double %29, 2.000000e+00
  br i1 %30, label %31, label %35

31:                                               ; preds = %27
  %32 = load double, ptr %8, align 8
  %33 = load double, ptr %8, align 8
  %34 = fmul double %32, %33
  br label %39

35:                                               ; preds = %27
  %36 = load double, ptr %8, align 8
  %37 = load double, ptr %9, align 8
  %38 = call double @pow(double noundef %36, double noundef %37) #8
  br label %39

39:                                               ; preds = %35, %31
  %40 = phi double [ %34, %31 ], [ %38, %35 ]
  store double %40, ptr %5, align 8
  br label %56

41:                                               ; preds = %4
  %42 = load ptr, ptr %6, align 8
  %43 = load double, ptr %8, align 8
  %44 = load double, ptr %9, align 8
  %45 = fdiv double %43, %44
  %46 = call double @llvm.floor.f64(double %45)
  store double %46, ptr %5, align 8
  br label %56

47:                                               ; preds = %4
  %48 = load double, ptr %8, align 8
  %49 = fneg double %48
  store double %49, ptr %5, align 8
  br label %56

50:                                               ; preds = %4
  %51 = load ptr, ptr %6, align 8
  %52 = load double, ptr %8, align 8
  %53 = load double, ptr %9, align 8
  %54 = call double @luaV_modf(ptr noundef %51, double noundef %52, double noundef %53)
  store double %54, ptr %5, align 8
  br label %56

55:                                               ; preds = %4
  store double 0.000000e+00, ptr %5, align 8
  br label %56

56:                                               ; preds = %55, %50, %47, %41, %39, %23, %19, %15, %11
  %57 = load double, ptr %5, align 8
  ret double %57
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaO_arith(ptr noundef %0, i32 noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store i32 %1, ptr %7, align 4
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = load i32, ptr %7, align 4
  %13 = load ptr, ptr %8, align 8
  %14 = load ptr, ptr %9, align 8
  %15 = load ptr, ptr %10, align 8
  %16 = call i32 @luaO_rawarith(ptr noundef %11, i32 noundef %12, ptr noundef %13, ptr noundef %14, ptr noundef %15)
  %17 = icmp ne i32 %16, 0
  br i1 %17, label %26, label %18

18:                                               ; preds = %5
  %19 = load ptr, ptr %6, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = load ptr, ptr %9, align 8
  %22 = load ptr, ptr %10, align 8
  %23 = load i32, ptr %7, align 4
  %24 = sub nsw i32 %23, 0
  %25 = add nsw i32 %24, 6
  call void @luaT_trybinTM(ptr noundef %19, ptr noundef %20, ptr noundef %21, ptr noundef %22, i32 noundef %25)
  br label %26

26:                                               ; preds = %18, %5
  ret void
}

declare hidden void @luaT_trybinTM(ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaO_hexavalue(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  %4 = load i32, ptr %3, align 4
  %5 = add nsw i32 %4, 1
  %6 = sext i32 %5 to i64
  %7 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %6
  %8 = load i8, ptr %7, align 1
  %9 = zext i8 %8 to i32
  %10 = and i32 %9, 2
  %11 = icmp ne i32 %10, 0
  br i1 %11, label %12, label %15

12:                                               ; preds = %1
  %13 = load i32, ptr %3, align 4
  %14 = sub nsw i32 %13, 48
  store i32 %14, ptr %2, align 4
  br label %20

15:                                               ; preds = %1
  %16 = load i32, ptr %3, align 4
  %17 = or i32 %16, 32
  %18 = sub nsw i32 %17, 97
  %19 = add nsw i32 %18, 10
  store i32 %19, ptr %2, align 4
  br label %20

20:                                               ; preds = %15, %12
  %21 = load i32, ptr %2, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaO_str2num(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca double, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %11 = load ptr, ptr @luaO_str2num.hook, align 8
  %12 = icmp ne ptr %11, null
  br i1 %12, label %13, label %18

13:                                               ; preds = %2
  %14 = load ptr, ptr @luaO_str2num.hook, align 8
  %15 = load ptr, ptr %4, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = call i64 %14(ptr noundef %15, ptr noundef %16)
  store i64 %17, ptr %3, align 8
  br label %49

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = call ptr @l_str2int(ptr noundef %19, ptr noundef %6)
  store ptr %20, ptr %8, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %29

22:                                               ; preds = %18
  %23 = load ptr, ptr %5, align 8
  store ptr %23, ptr %9, align 8
  %24 = load i64, ptr %6, align 8
  %25 = load ptr, ptr %9, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  store i64 %24, ptr %26, align 8
  %27 = load ptr, ptr %9, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  store i8 3, ptr %28, align 8
  br label %42

29:                                               ; preds = %18
  %30 = load ptr, ptr %4, align 8
  %31 = call ptr @l_str2d(ptr noundef %30, ptr noundef %7)
  store ptr %31, ptr %8, align 8
  %32 = icmp ne ptr %31, null
  br i1 %32, label %33, label %40

33:                                               ; preds = %29
  %34 = load ptr, ptr %5, align 8
  store ptr %34, ptr %10, align 8
  %35 = load double, ptr %7, align 8
  %36 = load ptr, ptr %10, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  store double %35, ptr %37, align 8
  %38 = load ptr, ptr %10, align 8
  %39 = getelementptr inbounds %struct.TValue, ptr %38, i32 0, i32 1
  store i8 19, ptr %39, align 8
  br label %41

40:                                               ; preds = %29
  store i64 0, ptr %3, align 8
  br label %49

41:                                               ; preds = %33
  br label %42

42:                                               ; preds = %41, %22
  %43 = load ptr, ptr %8, align 8
  %44 = load ptr, ptr %4, align 8
  %45 = ptrtoint ptr %43 to i64
  %46 = ptrtoint ptr %44 to i64
  %47 = sub i64 %45, %46
  %48 = add nsw i64 %47, 1
  store i64 %48, ptr %3, align 8
  br label %49

49:                                               ; preds = %42, %40, %13
  %50 = load i64, ptr %3, align 8
  ret i64 %50
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @l_str2int(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 0, ptr %6, align 8
  store i32 1, ptr %7, align 4
  br label %10

10:                                               ; preds = %21, %2
  %11 = load ptr, ptr %4, align 8
  %12 = load i8, ptr %11, align 1
  %13 = zext i8 %12 to i32
  %14 = add nsw i32 %13, 1
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %15
  %17 = load i8, ptr %16, align 1
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 8
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %21, label %24

21:                                               ; preds = %10
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds i8, ptr %22, i32 1
  store ptr %23, ptr %4, align 8
  br label %10, !llvm.loop !8

24:                                               ; preds = %10
  %25 = call i32 @isneg(ptr noundef %4)
  store i32 %25, ptr %8, align 4
  %26 = load ptr, ptr %4, align 8
  %27 = getelementptr inbounds i8, ptr %26, i64 0
  %28 = load i8, ptr %27, align 1
  %29 = sext i8 %28 to i32
  %30 = icmp eq i32 %29, 48
  br i1 %30, label %31, label %70

31:                                               ; preds = %24
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds i8, ptr %32, i64 1
  %34 = load i8, ptr %33, align 1
  %35 = sext i8 %34 to i32
  %36 = icmp eq i32 %35, 120
  br i1 %36, label %43, label %37

37:                                               ; preds = %31
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds i8, ptr %38, i64 1
  %40 = load i8, ptr %39, align 1
  %41 = sext i8 %40 to i32
  %42 = icmp eq i32 %41, 88
  br i1 %42, label %43, label %70

43:                                               ; preds = %37, %31
  %44 = load ptr, ptr %4, align 8
  %45 = getelementptr inbounds i8, ptr %44, i64 2
  store ptr %45, ptr %4, align 8
  br label %46

46:                                               ; preds = %66, %43
  %47 = load ptr, ptr %4, align 8
  %48 = load i8, ptr %47, align 1
  %49 = zext i8 %48 to i32
  %50 = add nsw i32 %49, 1
  %51 = sext i32 %50 to i64
  %52 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %51
  %53 = load i8, ptr %52, align 1
  %54 = zext i8 %53 to i32
  %55 = and i32 %54, 16
  %56 = icmp ne i32 %55, 0
  br i1 %56, label %57, label %69

57:                                               ; preds = %46
  %58 = load i64, ptr %6, align 8
  %59 = mul i64 %58, 16
  %60 = load ptr, ptr %4, align 8
  %61 = load i8, ptr %60, align 1
  %62 = sext i8 %61 to i32
  %63 = call i32 @luaO_hexavalue(i32 noundef %62)
  %64 = sext i32 %63 to i64
  %65 = add i64 %59, %64
  store i64 %65, ptr %6, align 8
  store i32 0, ptr %7, align 4
  br label %66

66:                                               ; preds = %57
  %67 = load ptr, ptr %4, align 8
  %68 = getelementptr inbounds i8, ptr %67, i32 1
  store ptr %68, ptr %4, align 8
  br label %46, !llvm.loop !9

69:                                               ; preds = %46
  br label %108

70:                                               ; preds = %37, %24
  br label %71

71:                                               ; preds = %104, %70
  %72 = load ptr, ptr %4, align 8
  %73 = load i8, ptr %72, align 1
  %74 = zext i8 %73 to i32
  %75 = add nsw i32 %74, 1
  %76 = sext i32 %75 to i64
  %77 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %76
  %78 = load i8, ptr %77, align 1
  %79 = zext i8 %78 to i32
  %80 = and i32 %79, 2
  %81 = icmp ne i32 %80, 0
  br i1 %81, label %82, label %107

82:                                               ; preds = %71
  %83 = load ptr, ptr %4, align 8
  %84 = load i8, ptr %83, align 1
  %85 = sext i8 %84 to i32
  %86 = sub nsw i32 %85, 48
  store i32 %86, ptr %9, align 4
  %87 = load i64, ptr %6, align 8
  %88 = icmp uge i64 %87, 922337203685477580
  br i1 %88, label %89, label %98

89:                                               ; preds = %82
  %90 = load i64, ptr %6, align 8
  %91 = icmp ugt i64 %90, 922337203685477580
  br i1 %91, label %97, label %92

92:                                               ; preds = %89
  %93 = load i32, ptr %9, align 4
  %94 = load i32, ptr %8, align 4
  %95 = add nsw i32 7, %94
  %96 = icmp sgt i32 %93, %95
  br i1 %96, label %97, label %98

97:                                               ; preds = %92, %89
  store ptr null, ptr %3, align 8
  br label %144

98:                                               ; preds = %92, %82
  %99 = load i64, ptr %6, align 8
  %100 = mul i64 %99, 10
  %101 = load i32, ptr %9, align 4
  %102 = sext i32 %101 to i64
  %103 = add i64 %100, %102
  store i64 %103, ptr %6, align 8
  store i32 0, ptr %7, align 4
  br label %104

104:                                              ; preds = %98
  %105 = load ptr, ptr %4, align 8
  %106 = getelementptr inbounds i8, ptr %105, i32 1
  store ptr %106, ptr %4, align 8
  br label %71, !llvm.loop !10

107:                                              ; preds = %71
  br label %108

108:                                              ; preds = %107, %69
  br label %109

109:                                              ; preds = %120, %108
  %110 = load ptr, ptr %4, align 8
  %111 = load i8, ptr %110, align 1
  %112 = zext i8 %111 to i32
  %113 = add nsw i32 %112, 1
  %114 = sext i32 %113 to i64
  %115 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %114
  %116 = load i8, ptr %115, align 1
  %117 = zext i8 %116 to i32
  %118 = and i32 %117, 8
  %119 = icmp ne i32 %118, 0
  br i1 %119, label %120, label %123

120:                                              ; preds = %109
  %121 = load ptr, ptr %4, align 8
  %122 = getelementptr inbounds i8, ptr %121, i32 1
  store ptr %122, ptr %4, align 8
  br label %109, !llvm.loop !11

123:                                              ; preds = %109
  %124 = load i32, ptr %7, align 4
  %125 = icmp ne i32 %124, 0
  br i1 %125, label %131, label %126

126:                                              ; preds = %123
  %127 = load ptr, ptr %4, align 8
  %128 = load i8, ptr %127, align 1
  %129 = sext i8 %128 to i32
  %130 = icmp ne i32 %129, 0
  br i1 %130, label %131, label %132

131:                                              ; preds = %126, %123
  store ptr null, ptr %3, align 8
  br label %144

132:                                              ; preds = %126
  %133 = load i32, ptr %8, align 4
  %134 = icmp ne i32 %133, 0
  br i1 %134, label %135, label %138

135:                                              ; preds = %132
  %136 = load i64, ptr %6, align 8
  %137 = sub i64 0, %136
  br label %140

138:                                              ; preds = %132
  %139 = load i64, ptr %6, align 8
  br label %140

140:                                              ; preds = %138, %135
  %141 = phi i64 [ %137, %135 ], [ %139, %138 ]
  %142 = load ptr, ptr %5, align 8
  store i64 %141, ptr %142, align 8
  %143 = load ptr, ptr %4, align 8
  store ptr %143, ptr %3, align 8
  br label %144

144:                                              ; preds = %140, %131, %97
  %145 = load ptr, ptr %3, align 8
  ret ptr %145
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @l_str2d(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca [201 x i8], align 16
  %10 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %11 = load ptr, ptr %4, align 8
  %12 = call ptr @strpbrk(ptr noundef %11, ptr noundef @.str.7) #9
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = icmp ne ptr %13, null
  br i1 %14, label %15, label %20

15:                                               ; preds = %2
  %16 = load ptr, ptr %7, align 8
  %17 = load i8, ptr %16, align 1
  %18 = zext i8 %17 to i32
  %19 = or i32 %18, 32
  br label %21

20:                                               ; preds = %2
  br label %21

21:                                               ; preds = %20, %15
  %22 = phi i32 [ %19, %15 ], [ 0, %20 ]
  store i32 %22, ptr %8, align 4
  %23 = load i32, ptr %8, align 4
  %24 = icmp eq i32 %23, 110
  br i1 %24, label %25, label %26

25:                                               ; preds = %21
  store ptr null, ptr %3, align 8
  br label %75

26:                                               ; preds = %21
  %27 = load ptr, ptr %4, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = load i32, ptr %8, align 4
  %30 = call ptr @l_str2dloc(ptr noundef %27, ptr noundef %28, i32 noundef %29)
  store ptr %30, ptr %6, align 8
  %31 = load ptr, ptr %6, align 8
  %32 = icmp eq ptr %31, null
  br i1 %32, label %33, label %73

33:                                               ; preds = %26
  %34 = load ptr, ptr %4, align 8
  %35 = call ptr @strchr(ptr noundef %34, i32 noundef 46) #9
  store ptr %35, ptr %10, align 8
  %36 = load ptr, ptr %10, align 8
  %37 = icmp eq ptr %36, null
  br i1 %37, label %42, label %38

38:                                               ; preds = %33
  %39 = load ptr, ptr %4, align 8
  %40 = call i64 @strlen(ptr noundef %39) #9
  %41 = icmp ugt i64 %40, 200
  br i1 %41, label %42, label %43

42:                                               ; preds = %38, %33
  store ptr null, ptr %3, align 8
  br label %75

43:                                               ; preds = %38
  %44 = getelementptr inbounds [201 x i8], ptr %9, i64 0, i64 0
  %45 = load ptr, ptr %4, align 8
  %46 = call ptr @strcpy(ptr noundef %44, ptr noundef %45) #8
  %47 = call ptr @localeconv() #8
  %48 = getelementptr inbounds %struct.lconv, ptr %47, i32 0, i32 0
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds i8, ptr %49, i64 0
  %51 = load i8, ptr %50, align 1
  %52 = load ptr, ptr %10, align 8
  %53 = load ptr, ptr %4, align 8
  %54 = ptrtoint ptr %52 to i64
  %55 = ptrtoint ptr %53 to i64
  %56 = sub i64 %54, %55
  %57 = getelementptr inbounds [201 x i8], ptr %9, i64 0, i64 %56
  store i8 %51, ptr %57, align 1
  %58 = getelementptr inbounds [201 x i8], ptr %9, i64 0, i64 0
  %59 = load ptr, ptr %5, align 8
  %60 = load i32, ptr %8, align 4
  %61 = call ptr @l_str2dloc(ptr noundef %58, ptr noundef %59, i32 noundef %60)
  store ptr %61, ptr %6, align 8
  %62 = load ptr, ptr %6, align 8
  %63 = icmp ne ptr %62, null
  br i1 %63, label %64, label %72

64:                                               ; preds = %43
  %65 = load ptr, ptr %4, align 8
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds [201 x i8], ptr %9, i64 0, i64 0
  %68 = ptrtoint ptr %66 to i64
  %69 = ptrtoint ptr %67 to i64
  %70 = sub i64 %68, %69
  %71 = getelementptr inbounds i8, ptr %65, i64 %70
  store ptr %71, ptr %6, align 8
  br label %72

72:                                               ; preds = %64, %43
  br label %73

73:                                               ; preds = %72, %26
  %74 = load ptr, ptr %6, align 8
  store ptr %74, ptr %3, align 8
  br label %75

75:                                               ; preds = %73, %42, %25
  %76 = load ptr, ptr %3, align 8
  ret ptr %76
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaO_utf8esc(ptr noundef %0, i64 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 8
  store i32 1, ptr %5, align 4
  %7 = load i64, ptr %4, align 8
  %8 = icmp ult i64 %7, 128
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load i64, ptr %4, align 8
  %11 = trunc i64 %10 to i8
  %12 = load ptr, ptr %3, align 8
  %13 = getelementptr inbounds i8, ptr %12, i64 7
  store i8 %11, ptr %13, align 1
  br label %48

14:                                               ; preds = %2
  store i32 63, ptr %6, align 4
  br label %15

15:                                               ; preds = %30, %14
  %16 = load i64, ptr %4, align 8
  %17 = and i64 %16, 63
  %18 = or i64 128, %17
  %19 = trunc i64 %18 to i8
  %20 = load ptr, ptr %3, align 8
  %21 = load i32, ptr %5, align 4
  %22 = add nsw i32 %21, 1
  store i32 %22, ptr %5, align 4
  %23 = sub nsw i32 8, %21
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds i8, ptr %20, i64 %24
  store i8 %19, ptr %25, align 1
  %26 = load i64, ptr %4, align 8
  %27 = lshr i64 %26, 6
  store i64 %27, ptr %4, align 8
  %28 = load i32, ptr %6, align 4
  %29 = lshr i32 %28, 1
  store i32 %29, ptr %6, align 4
  br label %30

30:                                               ; preds = %15
  %31 = load i64, ptr %4, align 8
  %32 = load i32, ptr %6, align 4
  %33 = zext i32 %32 to i64
  %34 = icmp ugt i64 %31, %33
  br i1 %34, label %15, label %35, !llvm.loop !12

35:                                               ; preds = %30
  %36 = load i32, ptr %6, align 4
  %37 = xor i32 %36, -1
  %38 = shl i32 %37, 1
  %39 = zext i32 %38 to i64
  %40 = load i64, ptr %4, align 8
  %41 = or i64 %39, %40
  %42 = trunc i64 %41 to i8
  %43 = load ptr, ptr %3, align 8
  %44 = load i32, ptr %5, align 4
  %45 = sub nsw i32 8, %44
  %46 = sext i32 %45 to i64
  %47 = getelementptr inbounds i8, ptr %43, i64 %46
  store i8 %42, ptr %47, align 1
  br label %48

48:                                               ; preds = %35, %9
  %49 = load i32, ptr %5, align 4
  ret i32 %49
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaO_tostring(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca [44 x i8], align 16
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds [44 x i8], ptr %5, i64 0, i64 0
  %11 = call i32 @tostringbuff(ptr noundef %9, ptr noundef %10)
  store i32 %11, ptr %6, align 4
  %12 = load ptr, ptr %4, align 8
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds [44 x i8], ptr %5, i64 0, i64 0
  %15 = load i32, ptr %6, align 4
  %16 = sext i32 %15 to i64
  %17 = call ptr @luaS_newlstr(ptr noundef %13, ptr noundef %14, i64 noundef %16)
  store ptr %17, ptr %8, align 8
  %18 = load ptr, ptr %8, align 8
  %19 = load ptr, ptr %7, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  store ptr %18, ptr %20, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = getelementptr inbounds %struct.TString, ptr %21, i32 0, i32 1
  %23 = load i8, ptr %22, align 8
  %24 = zext i8 %23 to i32
  %25 = or i32 %24, 64
  %26 = trunc i32 %25 to i8
  %27 = load ptr, ptr %7, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  store i8 %26, ptr %28, align 8
  %29 = load ptr, ptr %3, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @tostringbuff(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 1
  %8 = load i8, ptr %7, align 8
  %9 = zext i8 %8 to i32
  %10 = icmp eq i32 %9, 3
  br i1 %10, label %11, label %17

11:                                               ; preds = %2
  %12 = load ptr, ptr %4, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 0
  %15 = load i64, ptr %14, align 8
  %16 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %12, i64 noundef 44, ptr noundef @.str.8, i64 noundef %15) #8
  store i32 %16, ptr %5, align 4
  br label %47

17:                                               ; preds = %2
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %3, align 8
  %20 = getelementptr inbounds %struct.TValue, ptr %19, i32 0, i32 0
  %21 = load double, ptr %20, align 8
  %22 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %18, i64 noundef 44, ptr noundef @.str.9, double noundef %21) #8
  store i32 %22, ptr %5, align 4
  %23 = load ptr, ptr %4, align 8
  %24 = load ptr, ptr %4, align 8
  %25 = call i64 @strspn(ptr noundef %24, ptr noundef @.str.10) #9
  %26 = getelementptr inbounds i8, ptr %23, i64 %25
  %27 = load i8, ptr %26, align 1
  %28 = sext i8 %27 to i32
  %29 = icmp eq i32 %28, 0
  br i1 %29, label %30, label %46

30:                                               ; preds = %17
  %31 = call ptr @localeconv() #8
  %32 = getelementptr inbounds %struct.lconv, ptr %31, i32 0, i32 0
  %33 = load ptr, ptr %32, align 8
  %34 = getelementptr inbounds i8, ptr %33, i64 0
  %35 = load i8, ptr %34, align 1
  %36 = load ptr, ptr %4, align 8
  %37 = load i32, ptr %5, align 4
  %38 = add nsw i32 %37, 1
  store i32 %38, ptr %5, align 4
  %39 = sext i32 %37 to i64
  %40 = getelementptr inbounds i8, ptr %36, i64 %39
  store i8 %35, ptr %40, align 1
  %41 = load ptr, ptr %4, align 8
  %42 = load i32, ptr %5, align 4
  %43 = add nsw i32 %42, 1
  store i32 %43, ptr %5, align 4
  %44 = sext i32 %42 to i64
  %45 = getelementptr inbounds i8, ptr %41, i64 %44
  store i8 48, ptr %45, align 1
  br label %46

46:                                               ; preds = %30, %17
  br label %47

47:                                               ; preds = %46, %11
  %48 = load i32, ptr %5, align 4
  ret i32 %48
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaO_pushvfstring(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %struct.BuffFS, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca i8, align 1
  %11 = alloca %struct.TValue, align 8
  %12 = alloca ptr, align 8
  %13 = alloca %struct.TValue, align 8
  %14 = alloca ptr, align 8
  %15 = alloca %struct.TValue, align 8
  %16 = alloca ptr, align 8
  %17 = alloca i32, align 4
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  %20 = alloca i32, align 4
  %21 = alloca [8 x i8], align 1
  %22 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %23 = getelementptr inbounds %struct.BuffFS, ptr %7, i32 0, i32 2
  store i32 0, ptr %23, align 4
  %24 = getelementptr inbounds %struct.BuffFS, ptr %7, i32 0, i32 1
  store i32 0, ptr %24, align 8
  %25 = load ptr, ptr %4, align 8
  %26 = getelementptr inbounds %struct.BuffFS, ptr %7, i32 0, i32 0
  store ptr %25, ptr %26, align 8
  br label %27

27:                                               ; preds = %207, %3
  %28 = load ptr, ptr %5, align 8
  %29 = call ptr @strchr(ptr noundef %28, i32 noundef 37) #9
  store ptr %29, ptr %8, align 8
  %30 = icmp ne ptr %29, null
  br i1 %30, label %31, label %210

31:                                               ; preds = %27
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %8, align 8
  %34 = load ptr, ptr %5, align 8
  %35 = ptrtoint ptr %33 to i64
  %36 = ptrtoint ptr %34 to i64
  %37 = sub i64 %35, %36
  call void @addstr2buff(ptr noundef %7, ptr noundef %32, i64 noundef %37)
  %38 = load ptr, ptr %8, align 8
  %39 = getelementptr inbounds i8, ptr %38, i64 1
  %40 = load i8, ptr %39, align 1
  %41 = sext i8 %40 to i32
  switch i32 %41, label %201 [
    i32 115, label %42
    i32 99, label %66
    i32 100, label %84
    i32 73, label %106
    i32 102, label %127
    i32 112, label %148
    i32 85, label %173
    i32 37, label %200
  ]

42:                                               ; preds = %31
  %43 = load ptr, ptr %6, align 8
  %44 = getelementptr inbounds %struct.__va_list_tag, ptr %43, i32 0, i32 0
  %45 = load i32, ptr %44, align 8
  %46 = icmp ule i32 %45, 40
  br i1 %46, label %47, label %52

47:                                               ; preds = %42
  %48 = getelementptr inbounds %struct.__va_list_tag, ptr %43, i32 0, i32 3
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr i8, ptr %49, i32 %45
  %51 = add i32 %45, 8
  store i32 %51, ptr %44, align 8
  br label %56

52:                                               ; preds = %42
  %53 = getelementptr inbounds %struct.__va_list_tag, ptr %43, i32 0, i32 2
  %54 = load ptr, ptr %53, align 8
  %55 = getelementptr i8, ptr %54, i32 8
  store ptr %55, ptr %53, align 8
  br label %56

56:                                               ; preds = %52, %47
  %57 = phi ptr [ %50, %47 ], [ %54, %52 ]
  %58 = load ptr, ptr %57, align 8
  store ptr %58, ptr %9, align 8
  %59 = load ptr, ptr %9, align 8
  %60 = icmp eq ptr %59, null
  br i1 %60, label %61, label %62

61:                                               ; preds = %56
  store ptr @.str, ptr %9, align 8
  br label %62

62:                                               ; preds = %61, %56
  %63 = load ptr, ptr %9, align 8
  %64 = load ptr, ptr %9, align 8
  %65 = call i64 @strlen(ptr noundef %64) #9
  call void @addstr2buff(ptr noundef %7, ptr noundef %63, i64 noundef %65)
  br label %207

66:                                               ; preds = %31
  %67 = load ptr, ptr %6, align 8
  %68 = getelementptr inbounds %struct.__va_list_tag, ptr %67, i32 0, i32 0
  %69 = load i32, ptr %68, align 8
  %70 = icmp ule i32 %69, 40
  br i1 %70, label %71, label %76

71:                                               ; preds = %66
  %72 = getelementptr inbounds %struct.__va_list_tag, ptr %67, i32 0, i32 3
  %73 = load ptr, ptr %72, align 8
  %74 = getelementptr i8, ptr %73, i32 %69
  %75 = add i32 %69, 8
  store i32 %75, ptr %68, align 8
  br label %80

76:                                               ; preds = %66
  %77 = getelementptr inbounds %struct.__va_list_tag, ptr %67, i32 0, i32 2
  %78 = load ptr, ptr %77, align 8
  %79 = getelementptr i8, ptr %78, i32 8
  store ptr %79, ptr %77, align 8
  br label %80

80:                                               ; preds = %76, %71
  %81 = phi ptr [ %74, %71 ], [ %78, %76 ]
  %82 = load i32, ptr %81, align 4
  %83 = trunc i32 %82 to i8
  store i8 %83, ptr %10, align 1
  call void @addstr2buff(ptr noundef %7, ptr noundef %10, i64 noundef 1)
  br label %207

84:                                               ; preds = %31
  store ptr %11, ptr %12, align 8
  %85 = load ptr, ptr %6, align 8
  %86 = getelementptr inbounds %struct.__va_list_tag, ptr %85, i32 0, i32 0
  %87 = load i32, ptr %86, align 8
  %88 = icmp ule i32 %87, 40
  br i1 %88, label %89, label %94

89:                                               ; preds = %84
  %90 = getelementptr inbounds %struct.__va_list_tag, ptr %85, i32 0, i32 3
  %91 = load ptr, ptr %90, align 8
  %92 = getelementptr i8, ptr %91, i32 %87
  %93 = add i32 %87, 8
  store i32 %93, ptr %86, align 8
  br label %98

94:                                               ; preds = %84
  %95 = getelementptr inbounds %struct.__va_list_tag, ptr %85, i32 0, i32 2
  %96 = load ptr, ptr %95, align 8
  %97 = getelementptr i8, ptr %96, i32 8
  store ptr %97, ptr %95, align 8
  br label %98

98:                                               ; preds = %94, %89
  %99 = phi ptr [ %92, %89 ], [ %96, %94 ]
  %100 = load i32, ptr %99, align 4
  %101 = sext i32 %100 to i64
  %102 = load ptr, ptr %12, align 8
  %103 = getelementptr inbounds %struct.TValue, ptr %102, i32 0, i32 0
  store i64 %101, ptr %103, align 8
  %104 = load ptr, ptr %12, align 8
  %105 = getelementptr inbounds %struct.TValue, ptr %104, i32 0, i32 1
  store i8 3, ptr %105, align 8
  call void @addnum2buff(ptr noundef %7, ptr noundef %11)
  br label %207

106:                                              ; preds = %31
  store ptr %13, ptr %14, align 8
  %107 = load ptr, ptr %6, align 8
  %108 = getelementptr inbounds %struct.__va_list_tag, ptr %107, i32 0, i32 0
  %109 = load i32, ptr %108, align 8
  %110 = icmp ule i32 %109, 40
  br i1 %110, label %111, label %116

111:                                              ; preds = %106
  %112 = getelementptr inbounds %struct.__va_list_tag, ptr %107, i32 0, i32 3
  %113 = load ptr, ptr %112, align 8
  %114 = getelementptr i8, ptr %113, i32 %109
  %115 = add i32 %109, 8
  store i32 %115, ptr %108, align 8
  br label %120

116:                                              ; preds = %106
  %117 = getelementptr inbounds %struct.__va_list_tag, ptr %107, i32 0, i32 2
  %118 = load ptr, ptr %117, align 8
  %119 = getelementptr i8, ptr %118, i32 8
  store ptr %119, ptr %117, align 8
  br label %120

120:                                              ; preds = %116, %111
  %121 = phi ptr [ %114, %111 ], [ %118, %116 ]
  %122 = load i64, ptr %121, align 8
  %123 = load ptr, ptr %14, align 8
  %124 = getelementptr inbounds %struct.TValue, ptr %123, i32 0, i32 0
  store i64 %122, ptr %124, align 8
  %125 = load ptr, ptr %14, align 8
  %126 = getelementptr inbounds %struct.TValue, ptr %125, i32 0, i32 1
  store i8 3, ptr %126, align 8
  call void @addnum2buff(ptr noundef %7, ptr noundef %13)
  br label %207

127:                                              ; preds = %31
  store ptr %15, ptr %16, align 8
  %128 = load ptr, ptr %6, align 8
  %129 = getelementptr inbounds %struct.__va_list_tag, ptr %128, i32 0, i32 1
  %130 = load i32, ptr %129, align 4
  %131 = icmp ule i32 %130, 160
  br i1 %131, label %132, label %137

132:                                              ; preds = %127
  %133 = getelementptr inbounds %struct.__va_list_tag, ptr %128, i32 0, i32 3
  %134 = load ptr, ptr %133, align 8
  %135 = getelementptr i8, ptr %134, i32 %130
  %136 = add i32 %130, 16
  store i32 %136, ptr %129, align 4
  br label %141

137:                                              ; preds = %127
  %138 = getelementptr inbounds %struct.__va_list_tag, ptr %128, i32 0, i32 2
  %139 = load ptr, ptr %138, align 8
  %140 = getelementptr i8, ptr %139, i32 8
  store ptr %140, ptr %138, align 8
  br label %141

141:                                              ; preds = %137, %132
  %142 = phi ptr [ %135, %132 ], [ %139, %137 ]
  %143 = load double, ptr %142, align 8
  %144 = load ptr, ptr %16, align 8
  %145 = getelementptr inbounds %struct.TValue, ptr %144, i32 0, i32 0
  store double %143, ptr %145, align 8
  %146 = load ptr, ptr %16, align 8
  %147 = getelementptr inbounds %struct.TValue, ptr %146, i32 0, i32 1
  store i8 19, ptr %147, align 8
  call void @addnum2buff(ptr noundef %7, ptr noundef %15)
  br label %207

148:                                              ; preds = %31
  store i32 32, ptr %17, align 4
  %149 = call ptr @getbuff(ptr noundef %7, i32 noundef 32)
  store ptr %149, ptr %18, align 8
  %150 = load ptr, ptr %6, align 8
  %151 = getelementptr inbounds %struct.__va_list_tag, ptr %150, i32 0, i32 0
  %152 = load i32, ptr %151, align 8
  %153 = icmp ule i32 %152, 40
  br i1 %153, label %154, label %159

154:                                              ; preds = %148
  %155 = getelementptr inbounds %struct.__va_list_tag, ptr %150, i32 0, i32 3
  %156 = load ptr, ptr %155, align 8
  %157 = getelementptr i8, ptr %156, i32 %152
  %158 = add i32 %152, 8
  store i32 %158, ptr %151, align 8
  br label %163

159:                                              ; preds = %148
  %160 = getelementptr inbounds %struct.__va_list_tag, ptr %150, i32 0, i32 2
  %161 = load ptr, ptr %160, align 8
  %162 = getelementptr i8, ptr %161, i32 8
  store ptr %162, ptr %160, align 8
  br label %163

163:                                              ; preds = %159, %154
  %164 = phi ptr [ %157, %154 ], [ %161, %159 ]
  %165 = load ptr, ptr %164, align 8
  store ptr %165, ptr %19, align 8
  %166 = load ptr, ptr %18, align 8
  %167 = load ptr, ptr %19, align 8
  %168 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr noundef %166, i64 noundef 32, ptr noundef @.str.1, ptr noundef %167) #8
  store i32 %168, ptr %20, align 4
  %169 = load i32, ptr %20, align 4
  %170 = getelementptr inbounds %struct.BuffFS, ptr %7, i32 0, i32 2
  %171 = load i32, ptr %170, align 4
  %172 = add nsw i32 %171, %169
  store i32 %172, ptr %170, align 4
  br label %207

173:                                              ; preds = %31
  %174 = getelementptr inbounds [8 x i8], ptr %21, i64 0, i64 0
  %175 = load ptr, ptr %6, align 8
  %176 = getelementptr inbounds %struct.__va_list_tag, ptr %175, i32 0, i32 0
  %177 = load i32, ptr %176, align 8
  %178 = icmp ule i32 %177, 40
  br i1 %178, label %179, label %184

179:                                              ; preds = %173
  %180 = getelementptr inbounds %struct.__va_list_tag, ptr %175, i32 0, i32 3
  %181 = load ptr, ptr %180, align 8
  %182 = getelementptr i8, ptr %181, i32 %177
  %183 = add i32 %177, 8
  store i32 %183, ptr %176, align 8
  br label %188

184:                                              ; preds = %173
  %185 = getelementptr inbounds %struct.__va_list_tag, ptr %175, i32 0, i32 2
  %186 = load ptr, ptr %185, align 8
  %187 = getelementptr i8, ptr %186, i32 8
  store ptr %187, ptr %185, align 8
  br label %188

188:                                              ; preds = %184, %179
  %189 = phi ptr [ %182, %179 ], [ %186, %184 ]
  %190 = load i64, ptr %189, align 8
  %191 = call i32 @luaO_utf8esc(ptr noundef %174, i64 noundef %190)
  store i32 %191, ptr %22, align 4
  %192 = getelementptr inbounds [8 x i8], ptr %21, i64 0, i64 0
  %193 = getelementptr inbounds i8, ptr %192, i64 8
  %194 = load i32, ptr %22, align 4
  %195 = sext i32 %194 to i64
  %196 = sub i64 0, %195
  %197 = getelementptr inbounds i8, ptr %193, i64 %196
  %198 = load i32, ptr %22, align 4
  %199 = sext i32 %198 to i64
  call void @addstr2buff(ptr noundef %7, ptr noundef %197, i64 noundef %199)
  br label %207

200:                                              ; preds = %31
  call void @addstr2buff(ptr noundef %7, ptr noundef @.str.2, i64 noundef 1)
  br label %207

201:                                              ; preds = %31
  %202 = load ptr, ptr %4, align 8
  %203 = load ptr, ptr %8, align 8
  %204 = getelementptr inbounds i8, ptr %203, i64 1
  %205 = load i8, ptr %204, align 1
  %206 = sext i8 %205 to i32
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %202, ptr noundef @.str.3, i32 noundef %206) #10
  unreachable

207:                                              ; preds = %200, %188, %163, %141, %120, %98, %80, %62
  %208 = load ptr, ptr %8, align 8
  %209 = getelementptr inbounds i8, ptr %208, i64 2
  store ptr %209, ptr %5, align 8
  br label %27, !llvm.loop !13

210:                                              ; preds = %27
  %211 = load ptr, ptr %5, align 8
  %212 = load ptr, ptr %5, align 8
  %213 = call i64 @strlen(ptr noundef %212) #9
  call void @addstr2buff(ptr noundef %7, ptr noundef %211, i64 noundef %213)
  call void @clearbuff(ptr noundef %7)
  %214 = load ptr, ptr %4, align 8
  %215 = getelementptr inbounds %struct.lua_State, ptr %214, i32 0, i32 6
  %216 = load ptr, ptr %215, align 8
  %217 = getelementptr inbounds %union.StackValue, ptr %216, i64 -1
  %218 = getelementptr inbounds %struct.TValue, ptr %217, i32 0, i32 0
  %219 = load ptr, ptr %218, align 8
  %220 = getelementptr inbounds %struct.TString, ptr %219, i32 0, i32 7
  %221 = getelementptr inbounds [1 x i8], ptr %220, i64 0, i64 0
  ret ptr %221
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strchr(ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addstr2buff(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %8 = load i64, ptr %6, align 8
  %9 = icmp ule i64 %8, 199
  br i1 %9, label %10, label %24

10:                                               ; preds = %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i64, ptr %6, align 8
  %13 = trunc i64 %12 to i32
  %14 = call ptr @getbuff(ptr noundef %11, i32 noundef %13)
  store ptr %14, ptr %7, align 8
  %15 = load ptr, ptr %7, align 8
  %16 = load ptr, ptr %5, align 8
  %17 = load i64, ptr %6, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %15, ptr align 1 %16, i64 %17, i1 false)
  %18 = load i64, ptr %6, align 8
  %19 = trunc i64 %18 to i32
  %20 = load ptr, ptr %4, align 8
  %21 = getelementptr inbounds %struct.BuffFS, ptr %20, i32 0, i32 2
  %22 = load i32, ptr %21, align 4
  %23 = add nsw i32 %22, %19
  store i32 %23, ptr %21, align 4
  br label %29

24:                                               ; preds = %3
  %25 = load ptr, ptr %4, align 8
  call void @clearbuff(ptr noundef %25)
  %26 = load ptr, ptr %4, align 8
  %27 = load ptr, ptr %5, align 8
  %28 = load i64, ptr %6, align 8
  call void @pushstr(ptr noundef %26, ptr noundef %27, i64 noundef %28)
  br label %29

29:                                               ; preds = %24, %10
  ret void
}

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @addnum2buff(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = call ptr @getbuff(ptr noundef %7, i32 noundef 44)
  store ptr %8, ptr %5, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = call i32 @tostringbuff(ptr noundef %9, ptr noundef %10)
  store i32 %11, ptr %6, align 4
  %12 = load i32, ptr %6, align 4
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %struct.BuffFS, ptr %13, i32 0, i32 2
  %15 = load i32, ptr %14, align 4
  %16 = add nsw i32 %15, %12
  store i32 %16, ptr %14, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @getbuff(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %5 = load i32, ptr %4, align 4
  %6 = load ptr, ptr %3, align 8
  %7 = getelementptr inbounds %struct.BuffFS, ptr %6, i32 0, i32 2
  %8 = load i32, ptr %7, align 4
  %9 = sub nsw i32 199, %8
  %10 = icmp sgt i32 %5, %9
  br i1 %10, label %11, label %13

11:                                               ; preds = %2
  %12 = load ptr, ptr %3, align 8
  call void @clearbuff(ptr noundef %12)
  br label %13

13:                                               ; preds = %11, %2
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.BuffFS, ptr %14, i32 0, i32 3
  %16 = getelementptr inbounds [199 x i8], ptr %15, i64 0, i64 0
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.BuffFS, ptr %17, i32 0, i32 2
  %19 = load i32, ptr %18, align 4
  %20 = sext i32 %19 to i64
  %21 = getelementptr inbounds i8, ptr %16, i64 %20
  ret ptr %21
}

; Function Attrs: nounwind
declare i32 @snprintf(ptr noundef, i64 noundef, ptr noundef, ...) #3

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #4

; Function Attrs: noinline nounwind optnone uwtable
define internal void @clearbuff(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  %4 = load ptr, ptr %2, align 8
  %5 = getelementptr inbounds %struct.BuffFS, ptr %4, i32 0, i32 3
  %6 = getelementptr inbounds [199 x i8], ptr %5, i64 0, i64 0
  %7 = load ptr, ptr %2, align 8
  %8 = getelementptr inbounds %struct.BuffFS, ptr %7, i32 0, i32 2
  %9 = load i32, ptr %8, align 4
  %10 = sext i32 %9 to i64
  call void @pushstr(ptr noundef %3, ptr noundef %6, i64 noundef %10)
  %11 = load ptr, ptr %2, align 8
  %12 = getelementptr inbounds %struct.BuffFS, ptr %11, i32 0, i32 2
  store i32 0, ptr %12, align 4
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaO_pushfstring(ptr noundef %0, ptr noundef %1, ...) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca [1 x %struct.__va_list_tag], align 16
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  %7 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_start(ptr %7)
  %8 = load ptr, ptr %3, align 8
  %9 = load ptr, ptr %4, align 8
  %10 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  %11 = call ptr @luaO_pushvfstring(ptr noundef %8, ptr noundef %9, ptr noundef %10)
  store ptr %11, ptr %5, align 8
  %12 = getelementptr inbounds [1 x %struct.__va_list_tag], ptr %6, i64 0, i64 0
  call void @llvm.va_end(ptr %12)
  %13 = load ptr, ptr %5, align 8
  ret ptr %13
}

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_start(ptr) #5

; Function Attrs: nocallback nofree nosync nounwind willreturn
declare void @llvm.va_end(ptr) #5

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaO_chunkid(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  store i64 60, ptr %7, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load i8, ptr %9, align 1
  %11 = sext i8 %10 to i32
  %12 = icmp eq i32 %11, 61
  br i1 %12, label %13, label %36

13:                                               ; preds = %3
  %14 = load i64, ptr %6, align 8
  %15 = load i64, ptr %7, align 8
  %16 = icmp ule i64 %14, %15
  br i1 %16, label %17, label %23

17:                                               ; preds = %13
  %18 = load ptr, ptr %4, align 8
  %19 = load ptr, ptr %5, align 8
  %20 = getelementptr inbounds i8, ptr %19, i64 1
  %21 = load i64, ptr %6, align 8
  %22 = mul i64 %21, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %18, ptr align 1 %20, i64 %22, i1 false)
  br label %35

23:                                               ; preds = %13
  %24 = load ptr, ptr %4, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds i8, ptr %25, i64 1
  %27 = load i64, ptr %7, align 8
  %28 = sub i64 %27, 1
  %29 = mul i64 %28, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %24, ptr align 1 %26, i64 %29, i1 false)
  %30 = load i64, ptr %7, align 8
  %31 = sub i64 %30, 1
  %32 = load ptr, ptr %4, align 8
  %33 = getelementptr inbounds i8, ptr %32, i64 %31
  store ptr %33, ptr %4, align 8
  %34 = load ptr, ptr %4, align 8
  store i8 0, ptr %34, align 1
  br label %35

35:                                               ; preds = %23, %17
  br label %119

36:                                               ; preds = %3
  %37 = load ptr, ptr %5, align 8
  %38 = load i8, ptr %37, align 1
  %39 = sext i8 %38 to i32
  %40 = icmp eq i32 %39, 64
  br i1 %40, label %41, label %68

41:                                               ; preds = %36
  %42 = load i64, ptr %6, align 8
  %43 = load i64, ptr %7, align 8
  %44 = icmp ule i64 %42, %43
  br i1 %44, label %45, label %51

45:                                               ; preds = %41
  %46 = load ptr, ptr %4, align 8
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds i8, ptr %47, i64 1
  %49 = load i64, ptr %6, align 8
  %50 = mul i64 %49, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %46, ptr align 1 %48, i64 %50, i1 false)
  br label %67

51:                                               ; preds = %41
  %52 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %52, ptr align 1 @.str.4, i64 3, i1 false)
  %53 = load ptr, ptr %4, align 8
  %54 = getelementptr inbounds i8, ptr %53, i64 3
  store ptr %54, ptr %4, align 8
  %55 = load i64, ptr %7, align 8
  %56 = sub i64 %55, 3
  store i64 %56, ptr %7, align 8
  %57 = load ptr, ptr %4, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds i8, ptr %58, i64 1
  %60 = load i64, ptr %6, align 8
  %61 = getelementptr inbounds i8, ptr %59, i64 %60
  %62 = load i64, ptr %7, align 8
  %63 = sub i64 0, %62
  %64 = getelementptr inbounds i8, ptr %61, i64 %63
  %65 = load i64, ptr %7, align 8
  %66 = mul i64 %65, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %57, ptr align 1 %64, i64 %66, i1 false)
  br label %67

67:                                               ; preds = %51, %45
  br label %118

68:                                               ; preds = %36
  %69 = load ptr, ptr %5, align 8
  %70 = call ptr @strchr(ptr noundef %69, i32 noundef 10) #9
  store ptr %70, ptr %8, align 8
  %71 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %71, ptr align 1 @.str.5, i64 9, i1 false)
  %72 = load ptr, ptr %4, align 8
  %73 = getelementptr inbounds i8, ptr %72, i64 9
  store ptr %73, ptr %4, align 8
  %74 = load i64, ptr %7, align 8
  %75 = sub i64 %74, 15
  store i64 %75, ptr %7, align 8
  %76 = load i64, ptr %6, align 8
  %77 = load i64, ptr %7, align 8
  %78 = icmp ult i64 %76, %77
  br i1 %78, label %79, label %90

79:                                               ; preds = %68
  %80 = load ptr, ptr %8, align 8
  %81 = icmp eq ptr %80, null
  br i1 %81, label %82, label %90

82:                                               ; preds = %79
  %83 = load ptr, ptr %4, align 8
  %84 = load ptr, ptr %5, align 8
  %85 = load i64, ptr %6, align 8
  %86 = mul i64 %85, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %83, ptr align 1 %84, i64 %86, i1 false)
  %87 = load i64, ptr %6, align 8
  %88 = load ptr, ptr %4, align 8
  %89 = getelementptr inbounds i8, ptr %88, i64 %87
  store ptr %89, ptr %4, align 8
  br label %116

90:                                               ; preds = %79, %68
  %91 = load ptr, ptr %8, align 8
  %92 = icmp ne ptr %91, null
  br i1 %92, label %93, label %99

93:                                               ; preds = %90
  %94 = load ptr, ptr %8, align 8
  %95 = load ptr, ptr %5, align 8
  %96 = ptrtoint ptr %94 to i64
  %97 = ptrtoint ptr %95 to i64
  %98 = sub i64 %96, %97
  store i64 %98, ptr %6, align 8
  br label %99

99:                                               ; preds = %93, %90
  %100 = load i64, ptr %6, align 8
  %101 = load i64, ptr %7, align 8
  %102 = icmp ugt i64 %100, %101
  br i1 %102, label %103, label %105

103:                                              ; preds = %99
  %104 = load i64, ptr %7, align 8
  store i64 %104, ptr %6, align 8
  br label %105

105:                                              ; preds = %103, %99
  %106 = load ptr, ptr %4, align 8
  %107 = load ptr, ptr %5, align 8
  %108 = load i64, ptr %6, align 8
  %109 = mul i64 %108, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %106, ptr align 1 %107, i64 %109, i1 false)
  %110 = load i64, ptr %6, align 8
  %111 = load ptr, ptr %4, align 8
  %112 = getelementptr inbounds i8, ptr %111, i64 %110
  store ptr %112, ptr %4, align 8
  %113 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %113, ptr align 1 @.str.4, i64 3, i1 false)
  %114 = load ptr, ptr %4, align 8
  %115 = getelementptr inbounds i8, ptr %114, i64 3
  store ptr %115, ptr %4, align 8
  br label %116

116:                                              ; preds = %105, %82
  %117 = load ptr, ptr %4, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %117, ptr align 1 @.str.6, i64 3, i1 false)
  br label %118

118:                                              ; preds = %116, %67
  br label %119

119:                                              ; preds = %118, %35
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #6

declare hidden i64 @luaV_mod(ptr noundef, i64 noundef, i64 noundef) #1

declare hidden i64 @luaV_idiv(ptr noundef, i64 noundef, i64 noundef) #1

declare hidden i64 @luaV_shiftl(i64 noundef, i64 noundef) #1

; Function Attrs: nounwind
declare double @pow(double noundef, double noundef) #3

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.floor.f64(double) #7

declare hidden double @luaV_modf(ptr noundef, double noundef, double noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @isneg(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %4 = load ptr, ptr %3, align 8
  %5 = load ptr, ptr %4, align 8
  %6 = load i8, ptr %5, align 1
  %7 = sext i8 %6 to i32
  %8 = icmp eq i32 %7, 45
  br i1 %8, label %9, label %13

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8
  %11 = load ptr, ptr %10, align 8
  %12 = getelementptr inbounds i8, ptr %11, i32 1
  store ptr %12, ptr %10, align 8
  store i32 1, ptr %2, align 4
  br label %25

13:                                               ; preds = %1
  %14 = load ptr, ptr %3, align 8
  %15 = load ptr, ptr %14, align 8
  %16 = load i8, ptr %15, align 1
  %17 = sext i8 %16 to i32
  %18 = icmp eq i32 %17, 43
  br i1 %18, label %19, label %23

19:                                               ; preds = %13
  %20 = load ptr, ptr %3, align 8
  %21 = load ptr, ptr %20, align 8
  %22 = getelementptr inbounds i8, ptr %21, i32 1
  store ptr %22, ptr %20, align 8
  br label %23

23:                                               ; preds = %19, %13
  br label %24

24:                                               ; preds = %23
  store i32 0, ptr %2, align 4
  br label %25

25:                                               ; preds = %24, %9
  %26 = load i32, ptr %2, align 4
  ret i32 %26
}

; Function Attrs: nounwind willreturn memory(read)
declare ptr @strpbrk(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @l_str2dloc(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %9 = load i32, ptr %7, align 4
  %10 = icmp eq i32 %9, 120
  br i1 %10, label %11, label %14

11:                                               ; preds = %3
  %12 = load ptr, ptr %5, align 8
  %13 = call double @strtod(ptr noundef %12, ptr noundef %8) #8
  br label %17

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8
  %16 = call double @strtod(ptr noundef %15, ptr noundef %8) #8
  br label %17

17:                                               ; preds = %14, %11
  %18 = phi double [ %13, %11 ], [ %16, %14 ]
  %19 = load ptr, ptr %6, align 8
  store double %18, ptr %19, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = load ptr, ptr %5, align 8
  %22 = icmp eq ptr %20, %21
  br i1 %22, label %23, label %24

23:                                               ; preds = %17
  store ptr null, ptr %4, align 8
  br label %49

24:                                               ; preds = %17
  br label %25

25:                                               ; preds = %36, %24
  %26 = load ptr, ptr %8, align 8
  %27 = load i8, ptr %26, align 1
  %28 = zext i8 %27 to i32
  %29 = add nsw i32 %28, 1
  %30 = sext i32 %29 to i64
  %31 = getelementptr inbounds [257 x i8], ptr @luai_ctype_, i64 0, i64 %30
  %32 = load i8, ptr %31, align 1
  %33 = zext i8 %32 to i32
  %34 = and i32 %33, 8
  %35 = icmp ne i32 %34, 0
  br i1 %35, label %36, label %39

36:                                               ; preds = %25
  %37 = load ptr, ptr %8, align 8
  %38 = getelementptr inbounds i8, ptr %37, i32 1
  store ptr %38, ptr %8, align 8
  br label %25, !llvm.loop !14

39:                                               ; preds = %25
  %40 = load ptr, ptr %8, align 8
  %41 = load i8, ptr %40, align 1
  %42 = sext i8 %41 to i32
  %43 = icmp eq i32 %42, 0
  br i1 %43, label %44, label %46

44:                                               ; preds = %39
  %45 = load ptr, ptr %8, align 8
  br label %47

46:                                               ; preds = %39
  br label %47

47:                                               ; preds = %46, %44
  %48 = phi ptr [ %45, %44 ], [ null, %46 ]
  store ptr %48, ptr %4, align 8
  br label %49

49:                                               ; preds = %47, %23
  %50 = load ptr, ptr %4, align 8
  ret ptr %50
}

; Function Attrs: nounwind
declare ptr @strcpy(ptr noundef, ptr noundef) #3

; Function Attrs: nounwind
declare ptr @localeconv() #3

; Function Attrs: nounwind
declare double @strtod(ptr noundef, ptr noundef) #3

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strspn(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pushstr(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds %struct.BuffFS, ptr %10, i32 0, i32 0
  %12 = load ptr, ptr %11, align 8
  store ptr %12, ptr %7, align 8
  %13 = load ptr, ptr %7, align 8
  %14 = getelementptr inbounds %struct.lua_State, ptr %13, i32 0, i32 6
  %15 = load ptr, ptr %14, align 8
  store ptr %15, ptr %8, align 8
  %16 = load ptr, ptr %7, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load i64, ptr %6, align 8
  %19 = call ptr @luaS_newlstr(ptr noundef %16, ptr noundef %17, i64 noundef %18)
  store ptr %19, ptr %9, align 8
  %20 = load ptr, ptr %9, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 0
  store ptr %20, ptr %22, align 8
  %23 = load ptr, ptr %9, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  %27 = or i32 %26, 64
  %28 = trunc i32 %27 to i8
  %29 = load ptr, ptr %8, align 8
  %30 = getelementptr inbounds %struct.TValue, ptr %29, i32 0, i32 1
  store i8 %28, ptr %30, align 8
  %31 = load ptr, ptr %7, align 8
  %32 = load ptr, ptr %7, align 8
  %33 = getelementptr inbounds %struct.lua_State, ptr %32, i32 0, i32 6
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %union.StackValue, ptr %34, i32 1
  store ptr %35, ptr %33, align 8
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.BuffFS, ptr %36, i32 0, i32 1
  %38 = load i32, ptr %37, align 8
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %43, label %40

40:                                               ; preds = %3
  %41 = load ptr, ptr %4, align 8
  %42 = getelementptr inbounds %struct.BuffFS, ptr %41, i32 0, i32 1
  store i32 1, ptr %42, align 8
  br label %45

43:                                               ; preds = %3
  %44 = load ptr, ptr %7, align 8
  call void @luaV_concat(ptr noundef %44, i32 noundef 2)
  br label %45

45:                                               ; preds = %43, %40
  ret void
}

declare hidden void @luaV_concat(ptr noundef, i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #5 = { nocallback nofree nosync nounwind willreturn }
attributes #6 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #7 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #8 = { nounwind }
attributes #9 = { nounwind willreturn memory(read) }
attributes #10 = { noreturn }

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
