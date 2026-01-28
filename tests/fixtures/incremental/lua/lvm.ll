; ModuleID = 'lvm.c'
source_filename = "lvm.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.TValue = type { %union.Value, i8 }
%union.Value = type { ptr }
%struct.TString = type { ptr, i8, i8, i8, i8, i32, %union.anon, [1 x i8] }
%union.anon = type { i64 }
%struct.Table = type { ptr, i8, i8, i8, i8, i32, ptr, ptr, ptr, ptr, ptr }
%struct.lua_State = type { ptr, i8, i8, i8, i8, i16, %union.StkIdRel, ptr, ptr, %union.StkIdRel, %union.StkIdRel, ptr, %union.StkIdRel, ptr, ptr, ptr, %struct.CallInfo, ptr, i64, i32, i32, i32, i32, i32 }
%union.StkIdRel = type { ptr }
%struct.CallInfo = type { %union.StkIdRel, %union.StkIdRel, ptr, ptr, %union.anon.0, %union.anon.2, i16, i16 }
%union.anon.0 = type { %struct.anon.1 }
%struct.anon.1 = type { ptr, i64, i64 }
%union.anon.2 = type { i32 }
%struct.global_State = type { ptr, ptr, i64, i64, i64, i64, %struct.stringtable, %struct.TValue, %struct.TValue, i32, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, [25 x ptr], [9 x ptr], [53 x [2 x ptr]], ptr, ptr }
%struct.stringtable = type { ptr, i32, i32 }
%struct.GCObject = type { ptr, i8, i8 }
%struct.Udata = type { ptr, i8, i8, i16, i64, ptr, ptr, [1 x %union.UValue] }
%union.UValue = type { %struct.TValue }
%union.StackValue = type { %struct.TValue }
%struct.anon = type { ptr, i32, i32 }
%struct.LClosure = type { ptr, i8, i8, i8, ptr, ptr, [1 x ptr] }
%struct.Proto = type { ptr, i8, i8, i8, i8, i8, i32, i32, i32, i32, i32, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }
%struct.UpVal = type { ptr, i8, i8, %union.anon.4, %union.anon.5 }
%union.anon.4 = type { ptr }
%union.anon.5 = type { %struct.anon.6 }
%struct.anon.6 = type { ptr, ptr }
%struct.Upvaldesc = type { ptr, i8, i8, i8 }

@.str = private unnamed_addr constant [6 x i8] c"index\00", align 1
@.str.1 = private unnamed_addr constant [40 x i8] c"'__index' chain too long; possible loop\00", align 1
@.str.2 = private unnamed_addr constant [43 x i8] c"'__newindex' chain too long; possible loop\00", align 1
@.str.3 = private unnamed_addr constant [23 x i8] c"string length overflow\00", align 1
@.str.4 = private unnamed_addr constant [14 x i8] c"get length of\00", align 1
@.str.5 = private unnamed_addr constant [26 x i8] c"attempt to divide by zero\00", align 1
@.str.6 = private unnamed_addr constant [26 x i8] c"attempt to perform 'n%%0'\00", align 1
@luaV_execute.disptab = internal constant [83 x ptr] [ptr blockaddress(@luaV_execute, %487), ptr blockaddress(@luaV_execute, %534), ptr blockaddress(@luaV_execute, %575), ptr blockaddress(@luaV_execute, %616), ptr blockaddress(@luaV_execute, %664), ptr blockaddress(@luaV_execute, %715), ptr blockaddress(@luaV_execute, %747), ptr blockaddress(@luaV_execute, %781), ptr blockaddress(@luaV_execute, %813), ptr blockaddress(@luaV_execute, %855), ptr blockaddress(@luaV_execute, %907), ptr blockaddress(@luaV_execute, %991), ptr blockaddress(@luaV_execute, %1092), ptr blockaddress(@luaV_execute, %1240), ptr blockaddress(@luaV_execute, %1360), ptr blockaddress(@luaV_execute, %1457), ptr blockaddress(@luaV_execute, %1604), ptr blockaddress(@luaV_execute, %1798), ptr blockaddress(@luaV_execute, %1964), ptr blockaddress(@luaV_execute, %2107), ptr blockaddress(@luaV_execute, %2210), ptr blockaddress(@luaV_execute, %2333), ptr blockaddress(@luaV_execute, %2417), ptr blockaddress(@luaV_execute, %2544), ptr blockaddress(@luaV_execute, %2671), ptr blockaddress(@luaV_execute, %2798), ptr blockaddress(@luaV_execute, %2936), ptr blockaddress(@luaV_execute, %3043), ptr blockaddress(@luaV_execute, %3140), ptr blockaddress(@luaV_execute, %3279), ptr blockaddress(@luaV_execute, %3354), ptr blockaddress(@luaV_execute, %3429), ptr blockaddress(@luaV_execute, %3504), ptr blockaddress(@luaV_execute, %3576), ptr blockaddress(@luaV_execute, %3647), ptr blockaddress(@luaV_execute, %3774), ptr blockaddress(@luaV_execute, %3901), ptr blockaddress(@luaV_execute, %4028), ptr blockaddress(@luaV_execute, %4166), ptr blockaddress(@luaV_execute, %4273), ptr blockaddress(@luaV_execute, %4370), ptr blockaddress(@luaV_execute, %4509), ptr blockaddress(@luaV_execute, %4600), ptr blockaddress(@luaV_execute, %4691), ptr blockaddress(@luaV_execute, %4874), ptr blockaddress(@luaV_execute, %4782), ptr blockaddress(@luaV_execute, %4965), ptr blockaddress(@luaV_execute, %5031), ptr blockaddress(@luaV_execute, %5100), ptr blockaddress(@luaV_execute, %5170), ptr blockaddress(@luaV_execute, %5272), ptr blockaddress(@luaV_execute, %5353), ptr blockaddress(@luaV_execute, %5408), ptr blockaddress(@luaV_execute, %5459), ptr blockaddress(@luaV_execute, %5530), ptr blockaddress(@luaV_execute, %5576), ptr blockaddress(@luaV_execute, %5617), ptr blockaddress(@luaV_execute, %5653), ptr blockaddress(@luaV_execute, %5730), ptr blockaddress(@luaV_execute, %5850), ptr blockaddress(@luaV_execute, %5970), ptr blockaddress(@luaV_execute, %6033), ptr blockaddress(@luaV_execute, %6121), ptr blockaddress(@luaV_execute, %6232), ptr blockaddress(@luaV_execute, %6343), ptr blockaddress(@luaV_execute, %6454), ptr blockaddress(@luaV_execute, %6565), ptr blockaddress(@luaV_execute, %6635), ptr blockaddress(@luaV_execute, %6721), ptr blockaddress(@luaV_execute, %6785), ptr blockaddress(@luaV_execute, %6864), ptr blockaddress(@luaV_execute, %6970), ptr blockaddress(@luaV_execute, %7027), ptr blockaddress(@luaV_execute, %7126), ptr blockaddress(@luaV_execute, %7223), ptr blockaddress(@luaV_execute, %7275), ptr blockaddress(@luaV_execute, %7303), ptr blockaddress(@luaV_execute, %7353), ptr blockaddress(@luaV_execute, %7414), ptr blockaddress(@luaV_execute, %7560), ptr blockaddress(@luaV_execute, %7638), ptr blockaddress(@luaV_execute, %7689), ptr blockaddress(@luaV_execute, %7744)], align 16
@.str.7 = private unnamed_addr constant [19 x i8] c"'for' step is zero\00", align 1
@.str.8 = private unnamed_addr constant [6 x i8] c"limit\00", align 1
@.str.9 = private unnamed_addr constant [5 x i8] c"step\00", align 1
@.str.10 = private unnamed_addr constant [14 x i8] c"initial value\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_tonumber_(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %struct.TValue, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.TValue, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  %11 = icmp eq i32 %10, 3
  br i1 %11, label %12, label %18

12:                                               ; preds = %2
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.TValue, ptr %13, i32 0, i32 0
  %15 = load i64, ptr %14, align 8
  %16 = sitofp i64 %15 to double
  %17 = load ptr, ptr %5, align 8
  store double %16, ptr %17, align 8
  store i32 1, ptr %3, align 4
  br label %38

18:                                               ; preds = %2
  %19 = load ptr, ptr %4, align 8
  %20 = call i32 @l_strton(ptr noundef %19, ptr noundef %6)
  %21 = icmp ne i32 %20, 0
  br i1 %21, label %22, label %37

22:                                               ; preds = %18
  %23 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = icmp eq i32 %25, 3
  br i1 %26, label %27, label %31

27:                                               ; preds = %22
  %28 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 0
  %29 = load i64, ptr %28, align 8
  %30 = sitofp i64 %29 to double
  br label %34

31:                                               ; preds = %22
  %32 = getelementptr inbounds %struct.TValue, ptr %6, i32 0, i32 0
  %33 = load double, ptr %32, align 8
  br label %34

34:                                               ; preds = %31, %27
  %35 = phi double [ %30, %27 ], [ %33, %31 ]
  %36 = load ptr, ptr %5, align 8
  store double %35, ptr %36, align 8
  store i32 1, ptr %3, align 4
  br label %38

37:                                               ; preds = %18
  store i32 0, ptr %3, align 4
  br label %38

38:                                               ; preds = %37, %34, %12
  %39 = load i32, ptr %3, align 4
  ret i32 %39
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @l_strton(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %7 = load ptr, ptr %4, align 8
  %8 = getelementptr inbounds %struct.TValue, ptr %7, i32 0, i32 1
  %9 = load i8, ptr %8, align 8
  %10 = zext i8 %9 to i32
  %11 = and i32 %10, 15
  %12 = icmp eq i32 %11, 4
  br i1 %12, label %14, label %13

13:                                               ; preds = %2
  store i32 0, ptr %3, align 4
  br label %42

14:                                               ; preds = %2
  %15 = load ptr, ptr %4, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %6, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.TString, ptr %18, i32 0, i32 7
  %20 = getelementptr inbounds [1 x i8], ptr %19, i64 0, i64 0
  %21 = load ptr, ptr %5, align 8
  %22 = call i64 @luaO_str2num(ptr noundef %20, ptr noundef %21)
  %23 = load ptr, ptr %6, align 8
  %24 = getelementptr inbounds %struct.TString, ptr %23, i32 0, i32 4
  %25 = load i8, ptr %24, align 1
  %26 = zext i8 %25 to i32
  %27 = icmp ne i32 %26, 255
  br i1 %27, label %28, label %33

28:                                               ; preds = %14
  %29 = load ptr, ptr %6, align 8
  %30 = getelementptr inbounds %struct.TString, ptr %29, i32 0, i32 4
  %31 = load i8, ptr %30, align 1
  %32 = zext i8 %31 to i64
  br label %37

33:                                               ; preds = %14
  %34 = load ptr, ptr %6, align 8
  %35 = getelementptr inbounds %struct.TString, ptr %34, i32 0, i32 6
  %36 = load i64, ptr %35, align 8
  br label %37

37:                                               ; preds = %33, %28
  %38 = phi i64 [ %32, %28 ], [ %36, %33 ]
  %39 = add i64 %38, 1
  %40 = icmp eq i64 %22, %39
  %41 = zext i1 %40 to i32
  store i32 %41, ptr %3, align 4
  br label %42

42:                                               ; preds = %37, %13
  %43 = load i32, ptr %3, align 4
  ret i32 %43
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_flttointeger(double noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca double, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  %8 = alloca double, align 8
  store double %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %9 = load double, ptr %5, align 8
  %10 = call double @llvm.floor.f64(double %9)
  store double %10, ptr %8, align 8
  %11 = load double, ptr %5, align 8
  %12 = load double, ptr %8, align 8
  %13 = fcmp une double %11, %12
  br i1 %13, label %14, label %26

14:                                               ; preds = %3
  %15 = load i32, ptr %7, align 4
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %18

17:                                               ; preds = %14
  store i32 0, ptr %4, align 4
  br label %39

18:                                               ; preds = %14
  %19 = load i32, ptr %7, align 4
  %20 = icmp eq i32 %19, 2
  br i1 %20, label %21, label %24

21:                                               ; preds = %18
  %22 = load double, ptr %8, align 8
  %23 = fadd double %22, 1.000000e+00
  store double %23, ptr %8, align 8
  br label %24

24:                                               ; preds = %21, %18
  br label %25

25:                                               ; preds = %24
  br label %26

26:                                               ; preds = %25, %3
  %27 = load double, ptr %8, align 8
  %28 = fcmp oge double %27, 0xC3E0000000000000
  br i1 %28, label %29, label %36

29:                                               ; preds = %26
  %30 = load double, ptr %8, align 8
  %31 = fcmp olt double %30, 0x43E0000000000000
  br i1 %31, label %32, label %36

32:                                               ; preds = %29
  %33 = load double, ptr %8, align 8
  %34 = fptosi double %33 to i64
  %35 = load ptr, ptr %6, align 8
  store i64 %34, ptr %35, align 8
  br label %36

36:                                               ; preds = %32, %29, %26
  %37 = phi i1 [ false, %29 ], [ false, %26 ], [ true, %32 ]
  %38 = zext i1 %37 to i32
  store i32 %38, ptr %4, align 4
  br label %39

39:                                               ; preds = %36, %17
  %40 = load i32, ptr %4, align 4
  ret i32 %40
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.floor.f64(double) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_tointegerns(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %8 = load ptr, ptr %5, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = icmp eq i32 %11, 19
  br i1 %12, label %13, label %20

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load double, ptr %15, align 8
  %17 = load ptr, ptr %6, align 8
  %18 = load i32, ptr %7, align 4
  %19 = call i32 @luaV_flttointeger(double noundef %16, ptr noundef %17, i32 noundef %18)
  store i32 %19, ptr %4, align 4
  br label %32

20:                                               ; preds = %3
  %21 = load ptr, ptr %5, align 8
  %22 = getelementptr inbounds %struct.TValue, ptr %21, i32 0, i32 1
  %23 = load i8, ptr %22, align 8
  %24 = zext i8 %23 to i32
  %25 = icmp eq i32 %24, 3
  br i1 %25, label %26, label %31

26:                                               ; preds = %20
  %27 = load ptr, ptr %5, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 0
  %29 = load i64, ptr %28, align 8
  %30 = load ptr, ptr %6, align 8
  store i64 %29, ptr %30, align 8
  store i32 1, ptr %4, align 4
  br label %32

31:                                               ; preds = %20
  store i32 0, ptr %4, align 4
  br label %32

32:                                               ; preds = %31, %26, %13
  %33 = load i32, ptr %4, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_tointeger(ptr noundef %0, ptr noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.TValue, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i32 %2, ptr %6, align 4
  %8 = load ptr, ptr %4, align 8
  %9 = call i32 @l_strton(ptr noundef %8, ptr noundef %7)
  %10 = icmp ne i32 %9, 0
  br i1 %10, label %11, label %12

11:                                               ; preds = %3
  store ptr %7, ptr %4, align 8
  br label %12

12:                                               ; preds = %11, %3
  %13 = load ptr, ptr %4, align 8
  %14 = load ptr, ptr %5, align 8
  %15 = load i32, ptr %6, align 4
  %16 = call i32 @luaV_tointegerns(ptr noundef %13, ptr noundef %14, i32 noundef %15)
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_finishget(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  store i32 0, ptr %11, align 4
  br label %15

15:                                               ; preds = %131, %5
  %16 = load i32, ptr %11, align 4
  %17 = icmp slt i32 %16, 2000
  br i1 %17, label %18, label %134

18:                                               ; preds = %15
  %19 = load ptr, ptr %10, align 8
  %20 = icmp eq ptr %19, null
  br i1 %20, label %21, label %40

21:                                               ; preds = %18
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = call ptr @luaT_gettmbyobj(ptr noundef %22, ptr noundef %23, i32 noundef 0)
  store ptr %24, ptr %12, align 8
  %25 = load ptr, ptr %12, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 1
  %27 = load i8, ptr %26, align 8
  %28 = zext i8 %27 to i32
  %29 = and i32 %28, 15
  %30 = icmp eq i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = icmp ne i32 %31, 0
  %33 = zext i1 %32 to i32
  %34 = sext i32 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %36, label %39

36:                                               ; preds = %21
  %37 = load ptr, ptr %6, align 8
  %38 = load ptr, ptr %7, align 8
  call void @luaG_typeerror(ptr noundef %37, ptr noundef %38, ptr noundef @.str) #7
  unreachable

39:                                               ; preds = %21
  br label %83

40:                                               ; preds = %18
  %41 = load ptr, ptr %7, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  %43 = load ptr, ptr %42, align 8
  %44 = getelementptr inbounds %struct.Table, ptr %43, i32 0, i32 9
  %45 = load ptr, ptr %44, align 8
  %46 = icmp eq ptr %45, null
  br i1 %46, label %47, label %48

47:                                               ; preds = %40
  br label %75

48:                                               ; preds = %40
  %49 = load ptr, ptr %7, align 8
  %50 = getelementptr inbounds %struct.TValue, ptr %49, i32 0, i32 0
  %51 = load ptr, ptr %50, align 8
  %52 = getelementptr inbounds %struct.Table, ptr %51, i32 0, i32 9
  %53 = load ptr, ptr %52, align 8
  %54 = getelementptr inbounds %struct.Table, ptr %53, i32 0, i32 3
  %55 = load i8, ptr %54, align 2
  %56 = zext i8 %55 to i32
  %57 = and i32 %56, 1
  %58 = icmp ne i32 %57, 0
  br i1 %58, label %59, label %60

59:                                               ; preds = %48
  br label %73

60:                                               ; preds = %48
  %61 = load ptr, ptr %7, align 8
  %62 = getelementptr inbounds %struct.TValue, ptr %61, i32 0, i32 0
  %63 = load ptr, ptr %62, align 8
  %64 = getelementptr inbounds %struct.Table, ptr %63, i32 0, i32 9
  %65 = load ptr, ptr %64, align 8
  %66 = load ptr, ptr %6, align 8
  %67 = getelementptr inbounds %struct.lua_State, ptr %66, i32 0, i32 7
  %68 = load ptr, ptr %67, align 8
  %69 = getelementptr inbounds %struct.global_State, ptr %68, i32 0, i32 42
  %70 = getelementptr inbounds [25 x ptr], ptr %69, i64 0, i64 0
  %71 = load ptr, ptr %70, align 8
  %72 = call ptr @luaT_gettm(ptr noundef %65, i32 noundef 0, ptr noundef %71)
  br label %73

73:                                               ; preds = %60, %59
  %74 = phi ptr [ null, %59 ], [ %72, %60 ]
  br label %75

75:                                               ; preds = %73, %47
  %76 = phi ptr [ null, %47 ], [ %74, %73 ]
  store ptr %76, ptr %12, align 8
  %77 = load ptr, ptr %12, align 8
  %78 = icmp eq ptr %77, null
  br i1 %78, label %79, label %82

79:                                               ; preds = %75
  %80 = load ptr, ptr %9, align 8
  %81 = getelementptr inbounds %struct.TValue, ptr %80, i32 0, i32 1
  store i8 0, ptr %81, align 8
  br label %136

82:                                               ; preds = %75
  br label %83

83:                                               ; preds = %82, %39
  %84 = load ptr, ptr %12, align 8
  %85 = getelementptr inbounds %struct.TValue, ptr %84, i32 0, i32 1
  %86 = load i8, ptr %85, align 8
  %87 = zext i8 %86 to i32
  %88 = and i32 %87, 15
  %89 = icmp eq i32 %88, 6
  br i1 %89, label %90, label %96

90:                                               ; preds = %83
  %91 = load ptr, ptr %6, align 8
  %92 = load ptr, ptr %12, align 8
  %93 = load ptr, ptr %7, align 8
  %94 = load ptr, ptr %8, align 8
  %95 = load ptr, ptr %9, align 8
  call void @luaT_callTMres(ptr noundef %91, ptr noundef %92, ptr noundef %93, ptr noundef %94, ptr noundef %95)
  br label %136

96:                                               ; preds = %83
  %97 = load ptr, ptr %12, align 8
  store ptr %97, ptr %7, align 8
  %98 = load ptr, ptr %7, align 8
  %99 = getelementptr inbounds %struct.TValue, ptr %98, i32 0, i32 1
  %100 = load i8, ptr %99, align 8
  %101 = zext i8 %100 to i32
  %102 = icmp eq i32 %101, 69
  br i1 %102, label %104, label %103

103:                                              ; preds = %96
  store ptr null, ptr %10, align 8
  br i1 false, label %117, label %130

104:                                              ; preds = %96
  %105 = load ptr, ptr %7, align 8
  %106 = getelementptr inbounds %struct.TValue, ptr %105, i32 0, i32 0
  %107 = load ptr, ptr %106, align 8
  %108 = load ptr, ptr %8, align 8
  %109 = call ptr @luaH_get(ptr noundef %107, ptr noundef %108)
  store ptr %109, ptr %10, align 8
  %110 = load ptr, ptr %10, align 8
  %111 = getelementptr inbounds %struct.TValue, ptr %110, i32 0, i32 1
  %112 = load i8, ptr %111, align 8
  %113 = zext i8 %112 to i32
  %114 = and i32 %113, 15
  %115 = icmp eq i32 %114, 0
  %116 = xor i1 %115, true
  br i1 %116, label %117, label %130

117:                                              ; preds = %104, %103
  %118 = load ptr, ptr %9, align 8
  store ptr %118, ptr %13, align 8
  %119 = load ptr, ptr %10, align 8
  store ptr %119, ptr %14, align 8
  %120 = load ptr, ptr %13, align 8
  %121 = getelementptr inbounds %struct.TValue, ptr %120, i32 0, i32 0
  %122 = load ptr, ptr %14, align 8
  %123 = getelementptr inbounds %struct.TValue, ptr %122, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %121, ptr align 8 %123, i64 8, i1 false)
  %124 = load ptr, ptr %14, align 8
  %125 = getelementptr inbounds %struct.TValue, ptr %124, i32 0, i32 1
  %126 = load i8, ptr %125, align 8
  %127 = load ptr, ptr %13, align 8
  %128 = getelementptr inbounds %struct.TValue, ptr %127, i32 0, i32 1
  store i8 %126, ptr %128, align 8
  %129 = load ptr, ptr %6, align 8
  br label %136

130:                                              ; preds = %104, %103
  br label %131

131:                                              ; preds = %130
  %132 = load i32, ptr %11, align 4
  %133 = add nsw i32 %132, 1
  store i32 %133, ptr %11, align 4
  br label %15, !llvm.loop !6

134:                                              ; preds = %15
  %135 = load ptr, ptr %6, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %135, ptr noundef @.str.1) #7
  unreachable

136:                                              ; preds = %117, %90, %79
  ret void
}

declare hidden ptr @luaT_gettmbyobj(ptr noundef, ptr noundef, i32 noundef) #2

; Function Attrs: noreturn
declare hidden void @luaG_typeerror(ptr noundef, ptr noundef, ptr noundef) #3

declare hidden ptr @luaT_gettm(ptr noundef, i32 noundef, ptr noundef) #2

declare hidden void @luaT_callTMres(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #2

declare hidden ptr @luaH_get(ptr noundef, ptr noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #4

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #3

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_finishset(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  store i32 0, ptr %11, align 4
  br label %16

16:                                               ; preds = %200, %5
  %17 = load i32, ptr %11, align 4
  %18 = icmp slt i32 %17, 2000
  br i1 %18, label %19, label %203

19:                                               ; preds = %16
  %20 = load ptr, ptr %10, align 8
  %21 = icmp ne ptr %20, null
  br i1 %21, label %22, label %100

22:                                               ; preds = %19
  %23 = load ptr, ptr %7, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 0
  %25 = load ptr, ptr %24, align 8
  store ptr %25, ptr %13, align 8
  %26 = load ptr, ptr %13, align 8
  %27 = getelementptr inbounds %struct.Table, ptr %26, i32 0, i32 9
  %28 = load ptr, ptr %27, align 8
  %29 = icmp eq ptr %28, null
  br i1 %29, label %30, label %31

30:                                               ; preds = %22
  br label %54

31:                                               ; preds = %22
  %32 = load ptr, ptr %13, align 8
  %33 = getelementptr inbounds %struct.Table, ptr %32, i32 0, i32 9
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds %struct.Table, ptr %34, i32 0, i32 3
  %36 = load i8, ptr %35, align 2
  %37 = zext i8 %36 to i32
  %38 = and i32 %37, 2
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %41

40:                                               ; preds = %31
  br label %52

41:                                               ; preds = %31
  %42 = load ptr, ptr %13, align 8
  %43 = getelementptr inbounds %struct.Table, ptr %42, i32 0, i32 9
  %44 = load ptr, ptr %43, align 8
  %45 = load ptr, ptr %6, align 8
  %46 = getelementptr inbounds %struct.lua_State, ptr %45, i32 0, i32 7
  %47 = load ptr, ptr %46, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 42
  %49 = getelementptr inbounds [25 x ptr], ptr %48, i64 0, i64 1
  %50 = load ptr, ptr %49, align 8
  %51 = call ptr @luaT_gettm(ptr noundef %44, i32 noundef 1, ptr noundef %50)
  br label %52

52:                                               ; preds = %41, %40
  %53 = phi ptr [ null, %40 ], [ %51, %41 ]
  br label %54

54:                                               ; preds = %52, %30
  %55 = phi ptr [ null, %30 ], [ %53, %52 ]
  store ptr %55, ptr %12, align 8
  %56 = load ptr, ptr %12, align 8
  %57 = icmp eq ptr %56, null
  br i1 %57, label %58, label %99

58:                                               ; preds = %54
  %59 = load ptr, ptr %6, align 8
  %60 = load ptr, ptr %13, align 8
  %61 = load ptr, ptr %8, align 8
  %62 = load ptr, ptr %10, align 8
  %63 = load ptr, ptr %9, align 8
  call void @luaH_finishset(ptr noundef %59, ptr noundef %60, ptr noundef %61, ptr noundef %62, ptr noundef %63)
  %64 = load ptr, ptr %13, align 8
  %65 = getelementptr inbounds %struct.Table, ptr %64, i32 0, i32 3
  %66 = load i8, ptr %65, align 2
  %67 = zext i8 %66 to i32
  %68 = and i32 %67, -64
  %69 = trunc i32 %68 to i8
  store i8 %69, ptr %65, align 2
  %70 = load ptr, ptr %9, align 8
  %71 = getelementptr inbounds %struct.TValue, ptr %70, i32 0, i32 1
  %72 = load i8, ptr %71, align 8
  %73 = zext i8 %72 to i32
  %74 = and i32 %73, 64
  %75 = icmp ne i32 %74, 0
  br i1 %75, label %76, label %97

76:                                               ; preds = %58
  %77 = load ptr, ptr %13, align 8
  %78 = getelementptr inbounds %struct.GCObject, ptr %77, i32 0, i32 2
  %79 = load i8, ptr %78, align 1
  %80 = zext i8 %79 to i32
  %81 = and i32 %80, 32
  %82 = icmp ne i32 %81, 0
  br i1 %82, label %83, label %95

83:                                               ; preds = %76
  %84 = load ptr, ptr %9, align 8
  %85 = getelementptr inbounds %struct.TValue, ptr %84, i32 0, i32 0
  %86 = load ptr, ptr %85, align 8
  %87 = getelementptr inbounds %struct.GCObject, ptr %86, i32 0, i32 2
  %88 = load i8, ptr %87, align 1
  %89 = zext i8 %88 to i32
  %90 = and i32 %89, 24
  %91 = icmp ne i32 %90, 0
  br i1 %91, label %92, label %95

92:                                               ; preds = %83
  %93 = load ptr, ptr %6, align 8
  %94 = load ptr, ptr %13, align 8
  call void @luaC_barrierback_(ptr noundef %93, ptr noundef %94)
  br label %96

95:                                               ; preds = %83, %76
  br label %96

96:                                               ; preds = %95, %92
  br label %98

97:                                               ; preds = %58
  br label %98

98:                                               ; preds = %97, %96
  br label %205

99:                                               ; preds = %54
  br label %119

100:                                              ; preds = %19
  %101 = load ptr, ptr %6, align 8
  %102 = load ptr, ptr %7, align 8
  %103 = call ptr @luaT_gettmbyobj(ptr noundef %101, ptr noundef %102, i32 noundef 1)
  store ptr %103, ptr %12, align 8
  %104 = load ptr, ptr %12, align 8
  %105 = getelementptr inbounds %struct.TValue, ptr %104, i32 0, i32 1
  %106 = load i8, ptr %105, align 8
  %107 = zext i8 %106 to i32
  %108 = and i32 %107, 15
  %109 = icmp eq i32 %108, 0
  %110 = zext i1 %109 to i32
  %111 = icmp ne i32 %110, 0
  %112 = zext i1 %111 to i32
  %113 = sext i32 %112 to i64
  %114 = icmp ne i64 %113, 0
  br i1 %114, label %115, label %118

115:                                              ; preds = %100
  %116 = load ptr, ptr %6, align 8
  %117 = load ptr, ptr %7, align 8
  call void @luaG_typeerror(ptr noundef %116, ptr noundef %117, ptr noundef @.str) #7
  unreachable

118:                                              ; preds = %100
  br label %119

119:                                              ; preds = %118, %99
  %120 = load ptr, ptr %12, align 8
  %121 = getelementptr inbounds %struct.TValue, ptr %120, i32 0, i32 1
  %122 = load i8, ptr %121, align 8
  %123 = zext i8 %122 to i32
  %124 = and i32 %123, 15
  %125 = icmp eq i32 %124, 6
  br i1 %125, label %126, label %132

126:                                              ; preds = %119
  %127 = load ptr, ptr %6, align 8
  %128 = load ptr, ptr %12, align 8
  %129 = load ptr, ptr %7, align 8
  %130 = load ptr, ptr %8, align 8
  %131 = load ptr, ptr %9, align 8
  call void @luaT_callTM(ptr noundef %127, ptr noundef %128, ptr noundef %129, ptr noundef %130, ptr noundef %131)
  br label %205

132:                                              ; preds = %119
  %133 = load ptr, ptr %12, align 8
  store ptr %133, ptr %7, align 8
  %134 = load ptr, ptr %7, align 8
  %135 = getelementptr inbounds %struct.TValue, ptr %134, i32 0, i32 1
  %136 = load i8, ptr %135, align 8
  %137 = zext i8 %136 to i32
  %138 = icmp eq i32 %137, 69
  br i1 %138, label %140, label %139

139:                                              ; preds = %132
  store ptr null, ptr %10, align 8
  br i1 false, label %153, label %199

140:                                              ; preds = %132
  %141 = load ptr, ptr %7, align 8
  %142 = getelementptr inbounds %struct.TValue, ptr %141, i32 0, i32 0
  %143 = load ptr, ptr %142, align 8
  %144 = load ptr, ptr %8, align 8
  %145 = call ptr @luaH_get(ptr noundef %143, ptr noundef %144)
  store ptr %145, ptr %10, align 8
  %146 = load ptr, ptr %10, align 8
  %147 = getelementptr inbounds %struct.TValue, ptr %146, i32 0, i32 1
  %148 = load i8, ptr %147, align 8
  %149 = zext i8 %148 to i32
  %150 = and i32 %149, 15
  %151 = icmp eq i32 %150, 0
  %152 = xor i1 %151, true
  br i1 %152, label %153, label %199

153:                                              ; preds = %140, %139
  %154 = load ptr, ptr %10, align 8
  store ptr %154, ptr %14, align 8
  %155 = load ptr, ptr %9, align 8
  store ptr %155, ptr %15, align 8
  %156 = load ptr, ptr %14, align 8
  %157 = getelementptr inbounds %struct.TValue, ptr %156, i32 0, i32 0
  %158 = load ptr, ptr %15, align 8
  %159 = getelementptr inbounds %struct.TValue, ptr %158, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %157, ptr align 8 %159, i64 8, i1 false)
  %160 = load ptr, ptr %15, align 8
  %161 = getelementptr inbounds %struct.TValue, ptr %160, i32 0, i32 1
  %162 = load i8, ptr %161, align 8
  %163 = load ptr, ptr %14, align 8
  %164 = getelementptr inbounds %struct.TValue, ptr %163, i32 0, i32 1
  store i8 %162, ptr %164, align 8
  %165 = load ptr, ptr %6, align 8
  %166 = load ptr, ptr %9, align 8
  %167 = getelementptr inbounds %struct.TValue, ptr %166, i32 0, i32 1
  %168 = load i8, ptr %167, align 8
  %169 = zext i8 %168 to i32
  %170 = and i32 %169, 64
  %171 = icmp ne i32 %170, 0
  br i1 %171, label %172, label %197

172:                                              ; preds = %153
  %173 = load ptr, ptr %7, align 8
  %174 = getelementptr inbounds %struct.TValue, ptr %173, i32 0, i32 0
  %175 = load ptr, ptr %174, align 8
  %176 = getelementptr inbounds %struct.GCObject, ptr %175, i32 0, i32 2
  %177 = load i8, ptr %176, align 1
  %178 = zext i8 %177 to i32
  %179 = and i32 %178, 32
  %180 = icmp ne i32 %179, 0
  br i1 %180, label %181, label %195

181:                                              ; preds = %172
  %182 = load ptr, ptr %9, align 8
  %183 = getelementptr inbounds %struct.TValue, ptr %182, i32 0, i32 0
  %184 = load ptr, ptr %183, align 8
  %185 = getelementptr inbounds %struct.GCObject, ptr %184, i32 0, i32 2
  %186 = load i8, ptr %185, align 1
  %187 = zext i8 %186 to i32
  %188 = and i32 %187, 24
  %189 = icmp ne i32 %188, 0
  br i1 %189, label %190, label %195

190:                                              ; preds = %181
  %191 = load ptr, ptr %6, align 8
  %192 = load ptr, ptr %7, align 8
  %193 = getelementptr inbounds %struct.TValue, ptr %192, i32 0, i32 0
  %194 = load ptr, ptr %193, align 8
  call void @luaC_barrierback_(ptr noundef %191, ptr noundef %194)
  br label %196

195:                                              ; preds = %181, %172
  br label %196

196:                                              ; preds = %195, %190
  br label %198

197:                                              ; preds = %153
  br label %198

198:                                              ; preds = %197, %196
  br label %205

199:                                              ; preds = %140, %139
  br label %200

200:                                              ; preds = %199
  %201 = load i32, ptr %11, align 4
  %202 = add nsw i32 %201, 1
  store i32 %202, ptr %11, align 4
  br label %16, !llvm.loop !8

203:                                              ; preds = %16
  %204 = load ptr, ptr %6, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %204, ptr noundef @.str.2) #7
  unreachable

205:                                              ; preds = %198, %126, %98
  ret void
}

declare hidden void @luaH_finishset(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #2

declare hidden void @luaC_barrierback_(ptr noundef, ptr noundef) #2

declare hidden void @luaT_callTM(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_lessthan(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  %13 = icmp eq i32 %12, 3
  br i1 %13, label %14, label %25

14:                                               ; preds = %3
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 3
  br i1 %20, label %21, label %25

21:                                               ; preds = %14
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = call i32 @LTnum(ptr noundef %22, ptr noundef %23)
  store i32 %24, ptr %4, align 4
  br label %30

25:                                               ; preds = %14, %3
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = call i32 @lessthanothers(ptr noundef %26, ptr noundef %27, ptr noundef %28)
  store i32 %29, ptr %4, align 4
  br label %30

30:                                               ; preds = %25, %21
  %31 = load i32, ptr %4, align 4
  ret i32 %31
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LTnum(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = icmp eq i32 %11, 3
  br i1 %12, label %13, label %35

13:                                               ; preds = %2
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load i64, ptr %15, align 8
  store i64 %16, ptr %6, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 3
  br i1 %21, label %22, label %29

22:                                               ; preds = %13
  %23 = load i64, ptr %6, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  %26 = load i64, ptr %25, align 8
  %27 = icmp slt i64 %23, %26
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %57

29:                                               ; preds = %13
  %30 = load i64, ptr %6, align 8
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  %33 = load double, ptr %32, align 8
  %34 = call i32 @LTintfloat(i64 noundef %30, double noundef %33)
  store i32 %34, ptr %3, align 4
  br label %57

35:                                               ; preds = %2
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  %38 = load double, ptr %37, align 8
  store double %38, ptr %7, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 1
  %41 = load i8, ptr %40, align 8
  %42 = zext i8 %41 to i32
  %43 = icmp eq i32 %42, 19
  br i1 %43, label %44, label %51

44:                                               ; preds = %35
  %45 = load double, ptr %7, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load double, ptr %47, align 8
  %49 = fcmp olt double %45, %48
  %50 = zext i1 %49 to i32
  store i32 %50, ptr %3, align 4
  br label %57

51:                                               ; preds = %35
  %52 = load double, ptr %7, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  %55 = load i64, ptr %54, align 8
  %56 = call i32 @LTfloatint(double noundef %52, i64 noundef %55)
  store i32 %56, ptr %3, align 4
  br label %57

57:                                               ; preds = %51, %44, %29, %22
  %58 = load i32, ptr %3, align 4
  ret i32 %58
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @lessthanothers(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  %13 = icmp eq i32 %12, 4
  br i1 %13, label %14, label %31

14:                                               ; preds = %3
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 4
  br i1 %20, label %21, label %31

21:                                               ; preds = %14
  %22 = load ptr, ptr %6, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = call i32 @l_strcmp(ptr noundef %24, ptr noundef %27)
  %29 = icmp slt i32 %28, 0
  %30 = zext i1 %29 to i32
  store i32 %30, ptr %4, align 4
  br label %36

31:                                               ; preds = %14, %3
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %6, align 8
  %34 = load ptr, ptr %7, align 8
  %35 = call i32 @luaT_callorderTM(ptr noundef %32, ptr noundef %33, ptr noundef %34, i32 noundef 20)
  store i32 %35, ptr %4, align 4
  br label %36

36:                                               ; preds = %31, %21
  %37 = load i32, ptr %4, align 4
  ret i32 %37
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_lessequal(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  %13 = icmp eq i32 %12, 3
  br i1 %13, label %14, label %25

14:                                               ; preds = %3
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 3
  br i1 %20, label %21, label %25

21:                                               ; preds = %14
  %22 = load ptr, ptr %6, align 8
  %23 = load ptr, ptr %7, align 8
  %24 = call i32 @LEnum(ptr noundef %22, ptr noundef %23)
  store i32 %24, ptr %4, align 4
  br label %30

25:                                               ; preds = %14, %3
  %26 = load ptr, ptr %5, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = load ptr, ptr %7, align 8
  %29 = call i32 @lessequalothers(ptr noundef %26, ptr noundef %27, ptr noundef %28)
  store i32 %29, ptr %4, align 4
  br label %30

30:                                               ; preds = %25, %21
  %31 = load i32, ptr %4, align 4
  ret i32 %31
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LEnum(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = icmp eq i32 %11, 3
  br i1 %12, label %13, label %35

13:                                               ; preds = %2
  %14 = load ptr, ptr %4, align 8
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load i64, ptr %15, align 8
  store i64 %16, ptr %6, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 1
  %19 = load i8, ptr %18, align 8
  %20 = zext i8 %19 to i32
  %21 = icmp eq i32 %20, 3
  br i1 %21, label %22, label %29

22:                                               ; preds = %13
  %23 = load i64, ptr %6, align 8
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 0
  %26 = load i64, ptr %25, align 8
  %27 = icmp sle i64 %23, %26
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %57

29:                                               ; preds = %13
  %30 = load i64, ptr %6, align 8
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %struct.TValue, ptr %31, i32 0, i32 0
  %33 = load double, ptr %32, align 8
  %34 = call i32 @LEintfloat(i64 noundef %30, double noundef %33)
  store i32 %34, ptr %3, align 4
  br label %57

35:                                               ; preds = %2
  %36 = load ptr, ptr %4, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  %38 = load double, ptr %37, align 8
  store double %38, ptr %7, align 8
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 1
  %41 = load i8, ptr %40, align 8
  %42 = zext i8 %41 to i32
  %43 = icmp eq i32 %42, 19
  br i1 %43, label %44, label %51

44:                                               ; preds = %35
  %45 = load double, ptr %7, align 8
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.TValue, ptr %46, i32 0, i32 0
  %48 = load double, ptr %47, align 8
  %49 = fcmp ole double %45, %48
  %50 = zext i1 %49 to i32
  store i32 %50, ptr %3, align 4
  br label %57

51:                                               ; preds = %35
  %52 = load double, ptr %7, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  %55 = load i64, ptr %54, align 8
  %56 = call i32 @LEfloatint(double noundef %52, i64 noundef %55)
  store i32 %56, ptr %3, align 4
  br label %57

57:                                               ; preds = %51, %44, %29, %22
  %58 = load i32, ptr %3, align 4
  ret i32 %58
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @lessequalothers(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %8 = load ptr, ptr %6, align 8
  %9 = getelementptr inbounds %struct.TValue, ptr %8, i32 0, i32 1
  %10 = load i8, ptr %9, align 8
  %11 = zext i8 %10 to i32
  %12 = and i32 %11, 15
  %13 = icmp eq i32 %12, 4
  br i1 %13, label %14, label %31

14:                                               ; preds = %3
  %15 = load ptr, ptr %7, align 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 4
  br i1 %20, label %21, label %31

21:                                               ; preds = %14
  %22 = load ptr, ptr %6, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 0
  %24 = load ptr, ptr %23, align 8
  %25 = load ptr, ptr %7, align 8
  %26 = getelementptr inbounds %struct.TValue, ptr %25, i32 0, i32 0
  %27 = load ptr, ptr %26, align 8
  %28 = call i32 @l_strcmp(ptr noundef %24, ptr noundef %27)
  %29 = icmp sle i32 %28, 0
  %30 = zext i1 %29 to i32
  store i32 %30, ptr %4, align 4
  br label %36

31:                                               ; preds = %14, %3
  %32 = load ptr, ptr %5, align 8
  %33 = load ptr, ptr %6, align 8
  %34 = load ptr, ptr %7, align 8
  %35 = call i32 @luaT_callorderTM(ptr noundef %32, ptr noundef %33, ptr noundef %34, i32 noundef 21)
  store i32 %35, ptr %4, align 4
  br label %36

36:                                               ; preds = %31, %21
  %37 = load i32, ptr %4, align 4
  ret i32 %37
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaV_equalobj(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds %struct.TValue, ptr %11, i32 0, i32 1
  %13 = load i8, ptr %12, align 8
  %14 = zext i8 %13 to i32
  %15 = and i32 %14, 63
  %16 = load ptr, ptr %7, align 8
  %17 = getelementptr inbounds %struct.TValue, ptr %16, i32 0, i32 1
  %18 = load i8, ptr %17, align 8
  %19 = zext i8 %18 to i32
  %20 = and i32 %19, 63
  %21 = icmp ne i32 %15, %20
  br i1 %21, label %22, label %57

22:                                               ; preds = %3
  %23 = load ptr, ptr %6, align 8
  %24 = getelementptr inbounds %struct.TValue, ptr %23, i32 0, i32 1
  %25 = load i8, ptr %24, align 8
  %26 = zext i8 %25 to i32
  %27 = and i32 %26, 15
  %28 = load ptr, ptr %7, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 1
  %30 = load i8, ptr %29, align 8
  %31 = zext i8 %30 to i32
  %32 = and i32 %31, 15
  %33 = icmp ne i32 %27, %32
  br i1 %33, label %41, label %34

34:                                               ; preds = %22
  %35 = load ptr, ptr %6, align 8
  %36 = getelementptr inbounds %struct.TValue, ptr %35, i32 0, i32 1
  %37 = load i8, ptr %36, align 8
  %38 = zext i8 %37 to i32
  %39 = and i32 %38, 15
  %40 = icmp ne i32 %39, 3
  br i1 %40, label %41, label %42

41:                                               ; preds = %34, %22
  store i32 0, ptr %4, align 4
  br label %340

42:                                               ; preds = %34
  %43 = load ptr, ptr %6, align 8
  %44 = call i32 @luaV_tointegerns(ptr noundef %43, ptr noundef %9, i32 noundef 0)
  %45 = icmp ne i32 %44, 0
  br i1 %45, label %46, label %54

46:                                               ; preds = %42
  %47 = load ptr, ptr %7, align 8
  %48 = call i32 @luaV_tointegerns(ptr noundef %47, ptr noundef %10, i32 noundef 0)
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %54

50:                                               ; preds = %46
  %51 = load i64, ptr %9, align 8
  %52 = load i64, ptr %10, align 8
  %53 = icmp eq i64 %51, %52
  br label %54

54:                                               ; preds = %50, %46, %42
  %55 = phi i1 [ false, %46 ], [ false, %42 ], [ %53, %50 ]
  %56 = zext i1 %55 to i32
  store i32 %56, ptr %4, align 4
  br label %340

57:                                               ; preds = %3
  %58 = load ptr, ptr %6, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 1
  %60 = load i8, ptr %59, align 8
  %61 = zext i8 %60 to i32
  %62 = and i32 %61, 63
  switch i32 %62, label %299 [
    i32 0, label %63
    i32 1, label %63
    i32 17, label %63
    i32 3, label %64
    i32 19, label %73
    i32 2, label %82
    i32 22, label %91
    i32 4, label %100
    i32 20, label %109
    i32 7, label %117
    i32 5, label %208
  ]

63:                                               ; preds = %57, %57, %57
  store i32 1, ptr %4, align 4
  br label %340

64:                                               ; preds = %57
  %65 = load ptr, ptr %6, align 8
  %66 = getelementptr inbounds %struct.TValue, ptr %65, i32 0, i32 0
  %67 = load i64, ptr %66, align 8
  %68 = load ptr, ptr %7, align 8
  %69 = getelementptr inbounds %struct.TValue, ptr %68, i32 0, i32 0
  %70 = load i64, ptr %69, align 8
  %71 = icmp eq i64 %67, %70
  %72 = zext i1 %71 to i32
  store i32 %72, ptr %4, align 4
  br label %340

73:                                               ; preds = %57
  %74 = load ptr, ptr %6, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  %76 = load double, ptr %75, align 8
  %77 = load ptr, ptr %7, align 8
  %78 = getelementptr inbounds %struct.TValue, ptr %77, i32 0, i32 0
  %79 = load double, ptr %78, align 8
  %80 = fcmp oeq double %76, %79
  %81 = zext i1 %80 to i32
  store i32 %81, ptr %4, align 4
  br label %340

82:                                               ; preds = %57
  %83 = load ptr, ptr %6, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 0
  %85 = load ptr, ptr %84, align 8
  %86 = load ptr, ptr %7, align 8
  %87 = getelementptr inbounds %struct.TValue, ptr %86, i32 0, i32 0
  %88 = load ptr, ptr %87, align 8
  %89 = icmp eq ptr %85, %88
  %90 = zext i1 %89 to i32
  store i32 %90, ptr %4, align 4
  br label %340

91:                                               ; preds = %57
  %92 = load ptr, ptr %6, align 8
  %93 = getelementptr inbounds %struct.TValue, ptr %92, i32 0, i32 0
  %94 = load ptr, ptr %93, align 8
  %95 = load ptr, ptr %7, align 8
  %96 = getelementptr inbounds %struct.TValue, ptr %95, i32 0, i32 0
  %97 = load ptr, ptr %96, align 8
  %98 = icmp eq ptr %94, %97
  %99 = zext i1 %98 to i32
  store i32 %99, ptr %4, align 4
  br label %340

100:                                              ; preds = %57
  %101 = load ptr, ptr %6, align 8
  %102 = getelementptr inbounds %struct.TValue, ptr %101, i32 0, i32 0
  %103 = load ptr, ptr %102, align 8
  %104 = load ptr, ptr %7, align 8
  %105 = getelementptr inbounds %struct.TValue, ptr %104, i32 0, i32 0
  %106 = load ptr, ptr %105, align 8
  %107 = icmp eq ptr %103, %106
  %108 = zext i1 %107 to i32
  store i32 %108, ptr %4, align 4
  br label %340

109:                                              ; preds = %57
  %110 = load ptr, ptr %6, align 8
  %111 = getelementptr inbounds %struct.TValue, ptr %110, i32 0, i32 0
  %112 = load ptr, ptr %111, align 8
  %113 = load ptr, ptr %7, align 8
  %114 = getelementptr inbounds %struct.TValue, ptr %113, i32 0, i32 0
  %115 = load ptr, ptr %114, align 8
  %116 = call i32 @luaS_eqlngstr(ptr noundef %112, ptr noundef %115)
  store i32 %116, ptr %4, align 4
  br label %340

117:                                              ; preds = %57
  %118 = load ptr, ptr %6, align 8
  %119 = getelementptr inbounds %struct.TValue, ptr %118, i32 0, i32 0
  %120 = load ptr, ptr %119, align 8
  %121 = load ptr, ptr %7, align 8
  %122 = getelementptr inbounds %struct.TValue, ptr %121, i32 0, i32 0
  %123 = load ptr, ptr %122, align 8
  %124 = icmp eq ptr %120, %123
  br i1 %124, label %125, label %126

125:                                              ; preds = %117
  store i32 1, ptr %4, align 4
  br label %340

126:                                              ; preds = %117
  %127 = load ptr, ptr %5, align 8
  %128 = icmp eq ptr %127, null
  br i1 %128, label %129, label %130

129:                                              ; preds = %126
  store i32 0, ptr %4, align 4
  br label %340

130:                                              ; preds = %126
  br label %131

131:                                              ; preds = %130
  %132 = load ptr, ptr %6, align 8
  %133 = getelementptr inbounds %struct.TValue, ptr %132, i32 0, i32 0
  %134 = load ptr, ptr %133, align 8
  %135 = getelementptr inbounds %struct.Udata, ptr %134, i32 0, i32 5
  %136 = load ptr, ptr %135, align 8
  %137 = icmp eq ptr %136, null
  br i1 %137, label %138, label %139

138:                                              ; preds = %131
  br label %166

139:                                              ; preds = %131
  %140 = load ptr, ptr %6, align 8
  %141 = getelementptr inbounds %struct.TValue, ptr %140, i32 0, i32 0
  %142 = load ptr, ptr %141, align 8
  %143 = getelementptr inbounds %struct.Udata, ptr %142, i32 0, i32 5
  %144 = load ptr, ptr %143, align 8
  %145 = getelementptr inbounds %struct.Table, ptr %144, i32 0, i32 3
  %146 = load i8, ptr %145, align 2
  %147 = zext i8 %146 to i32
  %148 = and i32 %147, 32
  %149 = icmp ne i32 %148, 0
  br i1 %149, label %150, label %151

150:                                              ; preds = %139
  br label %164

151:                                              ; preds = %139
  %152 = load ptr, ptr %6, align 8
  %153 = getelementptr inbounds %struct.TValue, ptr %152, i32 0, i32 0
  %154 = load ptr, ptr %153, align 8
  %155 = getelementptr inbounds %struct.Udata, ptr %154, i32 0, i32 5
  %156 = load ptr, ptr %155, align 8
  %157 = load ptr, ptr %5, align 8
  %158 = getelementptr inbounds %struct.lua_State, ptr %157, i32 0, i32 7
  %159 = load ptr, ptr %158, align 8
  %160 = getelementptr inbounds %struct.global_State, ptr %159, i32 0, i32 42
  %161 = getelementptr inbounds [25 x ptr], ptr %160, i64 0, i64 5
  %162 = load ptr, ptr %161, align 8
  %163 = call ptr @luaT_gettm(ptr noundef %156, i32 noundef 5, ptr noundef %162)
  br label %164

164:                                              ; preds = %151, %150
  %165 = phi ptr [ null, %150 ], [ %163, %151 ]
  br label %166

166:                                              ; preds = %164, %138
  %167 = phi ptr [ null, %138 ], [ %165, %164 ]
  store ptr %167, ptr %8, align 8
  %168 = load ptr, ptr %8, align 8
  %169 = icmp eq ptr %168, null
  br i1 %169, label %170, label %207

170:                                              ; preds = %166
  %171 = load ptr, ptr %7, align 8
  %172 = getelementptr inbounds %struct.TValue, ptr %171, i32 0, i32 0
  %173 = load ptr, ptr %172, align 8
  %174 = getelementptr inbounds %struct.Udata, ptr %173, i32 0, i32 5
  %175 = load ptr, ptr %174, align 8
  %176 = icmp eq ptr %175, null
  br i1 %176, label %177, label %178

177:                                              ; preds = %170
  br label %205

178:                                              ; preds = %170
  %179 = load ptr, ptr %7, align 8
  %180 = getelementptr inbounds %struct.TValue, ptr %179, i32 0, i32 0
  %181 = load ptr, ptr %180, align 8
  %182 = getelementptr inbounds %struct.Udata, ptr %181, i32 0, i32 5
  %183 = load ptr, ptr %182, align 8
  %184 = getelementptr inbounds %struct.Table, ptr %183, i32 0, i32 3
  %185 = load i8, ptr %184, align 2
  %186 = zext i8 %185 to i32
  %187 = and i32 %186, 32
  %188 = icmp ne i32 %187, 0
  br i1 %188, label %189, label %190

189:                                              ; preds = %178
  br label %203

190:                                              ; preds = %178
  %191 = load ptr, ptr %7, align 8
  %192 = getelementptr inbounds %struct.TValue, ptr %191, i32 0, i32 0
  %193 = load ptr, ptr %192, align 8
  %194 = getelementptr inbounds %struct.Udata, ptr %193, i32 0, i32 5
  %195 = load ptr, ptr %194, align 8
  %196 = load ptr, ptr %5, align 8
  %197 = getelementptr inbounds %struct.lua_State, ptr %196, i32 0, i32 7
  %198 = load ptr, ptr %197, align 8
  %199 = getelementptr inbounds %struct.global_State, ptr %198, i32 0, i32 42
  %200 = getelementptr inbounds [25 x ptr], ptr %199, i64 0, i64 5
  %201 = load ptr, ptr %200, align 8
  %202 = call ptr @luaT_gettm(ptr noundef %195, i32 noundef 5, ptr noundef %201)
  br label %203

203:                                              ; preds = %190, %189
  %204 = phi ptr [ null, %189 ], [ %202, %190 ]
  br label %205

205:                                              ; preds = %203, %177
  %206 = phi ptr [ null, %177 ], [ %204, %203 ]
  store ptr %206, ptr %8, align 8
  br label %207

207:                                              ; preds = %205, %166
  br label %308

208:                                              ; preds = %57
  %209 = load ptr, ptr %6, align 8
  %210 = getelementptr inbounds %struct.TValue, ptr %209, i32 0, i32 0
  %211 = load ptr, ptr %210, align 8
  %212 = load ptr, ptr %7, align 8
  %213 = getelementptr inbounds %struct.TValue, ptr %212, i32 0, i32 0
  %214 = load ptr, ptr %213, align 8
  %215 = icmp eq ptr %211, %214
  br i1 %215, label %216, label %217

216:                                              ; preds = %208
  store i32 1, ptr %4, align 4
  br label %340

217:                                              ; preds = %208
  %218 = load ptr, ptr %5, align 8
  %219 = icmp eq ptr %218, null
  br i1 %219, label %220, label %221

220:                                              ; preds = %217
  store i32 0, ptr %4, align 4
  br label %340

221:                                              ; preds = %217
  br label %222

222:                                              ; preds = %221
  %223 = load ptr, ptr %6, align 8
  %224 = getelementptr inbounds %struct.TValue, ptr %223, i32 0, i32 0
  %225 = load ptr, ptr %224, align 8
  %226 = getelementptr inbounds %struct.Table, ptr %225, i32 0, i32 9
  %227 = load ptr, ptr %226, align 8
  %228 = icmp eq ptr %227, null
  br i1 %228, label %229, label %230

229:                                              ; preds = %222
  br label %257

230:                                              ; preds = %222
  %231 = load ptr, ptr %6, align 8
  %232 = getelementptr inbounds %struct.TValue, ptr %231, i32 0, i32 0
  %233 = load ptr, ptr %232, align 8
  %234 = getelementptr inbounds %struct.Table, ptr %233, i32 0, i32 9
  %235 = load ptr, ptr %234, align 8
  %236 = getelementptr inbounds %struct.Table, ptr %235, i32 0, i32 3
  %237 = load i8, ptr %236, align 2
  %238 = zext i8 %237 to i32
  %239 = and i32 %238, 32
  %240 = icmp ne i32 %239, 0
  br i1 %240, label %241, label %242

241:                                              ; preds = %230
  br label %255

242:                                              ; preds = %230
  %243 = load ptr, ptr %6, align 8
  %244 = getelementptr inbounds %struct.TValue, ptr %243, i32 0, i32 0
  %245 = load ptr, ptr %244, align 8
  %246 = getelementptr inbounds %struct.Table, ptr %245, i32 0, i32 9
  %247 = load ptr, ptr %246, align 8
  %248 = load ptr, ptr %5, align 8
  %249 = getelementptr inbounds %struct.lua_State, ptr %248, i32 0, i32 7
  %250 = load ptr, ptr %249, align 8
  %251 = getelementptr inbounds %struct.global_State, ptr %250, i32 0, i32 42
  %252 = getelementptr inbounds [25 x ptr], ptr %251, i64 0, i64 5
  %253 = load ptr, ptr %252, align 8
  %254 = call ptr @luaT_gettm(ptr noundef %247, i32 noundef 5, ptr noundef %253)
  br label %255

255:                                              ; preds = %242, %241
  %256 = phi ptr [ null, %241 ], [ %254, %242 ]
  br label %257

257:                                              ; preds = %255, %229
  %258 = phi ptr [ null, %229 ], [ %256, %255 ]
  store ptr %258, ptr %8, align 8
  %259 = load ptr, ptr %8, align 8
  %260 = icmp eq ptr %259, null
  br i1 %260, label %261, label %298

261:                                              ; preds = %257
  %262 = load ptr, ptr %7, align 8
  %263 = getelementptr inbounds %struct.TValue, ptr %262, i32 0, i32 0
  %264 = load ptr, ptr %263, align 8
  %265 = getelementptr inbounds %struct.Table, ptr %264, i32 0, i32 9
  %266 = load ptr, ptr %265, align 8
  %267 = icmp eq ptr %266, null
  br i1 %267, label %268, label %269

268:                                              ; preds = %261
  br label %296

269:                                              ; preds = %261
  %270 = load ptr, ptr %7, align 8
  %271 = getelementptr inbounds %struct.TValue, ptr %270, i32 0, i32 0
  %272 = load ptr, ptr %271, align 8
  %273 = getelementptr inbounds %struct.Table, ptr %272, i32 0, i32 9
  %274 = load ptr, ptr %273, align 8
  %275 = getelementptr inbounds %struct.Table, ptr %274, i32 0, i32 3
  %276 = load i8, ptr %275, align 2
  %277 = zext i8 %276 to i32
  %278 = and i32 %277, 32
  %279 = icmp ne i32 %278, 0
  br i1 %279, label %280, label %281

280:                                              ; preds = %269
  br label %294

281:                                              ; preds = %269
  %282 = load ptr, ptr %7, align 8
  %283 = getelementptr inbounds %struct.TValue, ptr %282, i32 0, i32 0
  %284 = load ptr, ptr %283, align 8
  %285 = getelementptr inbounds %struct.Table, ptr %284, i32 0, i32 9
  %286 = load ptr, ptr %285, align 8
  %287 = load ptr, ptr %5, align 8
  %288 = getelementptr inbounds %struct.lua_State, ptr %287, i32 0, i32 7
  %289 = load ptr, ptr %288, align 8
  %290 = getelementptr inbounds %struct.global_State, ptr %289, i32 0, i32 42
  %291 = getelementptr inbounds [25 x ptr], ptr %290, i64 0, i64 5
  %292 = load ptr, ptr %291, align 8
  %293 = call ptr @luaT_gettm(ptr noundef %286, i32 noundef 5, ptr noundef %292)
  br label %294

294:                                              ; preds = %281, %280
  %295 = phi ptr [ null, %280 ], [ %293, %281 ]
  br label %296

296:                                              ; preds = %294, %268
  %297 = phi ptr [ null, %268 ], [ %295, %294 ]
  store ptr %297, ptr %8, align 8
  br label %298

298:                                              ; preds = %296, %257
  br label %308

299:                                              ; preds = %57
  %300 = load ptr, ptr %6, align 8
  %301 = getelementptr inbounds %struct.TValue, ptr %300, i32 0, i32 0
  %302 = load ptr, ptr %301, align 8
  %303 = load ptr, ptr %7, align 8
  %304 = getelementptr inbounds %struct.TValue, ptr %303, i32 0, i32 0
  %305 = load ptr, ptr %304, align 8
  %306 = icmp eq ptr %302, %305
  %307 = zext i1 %306 to i32
  store i32 %307, ptr %4, align 4
  br label %340

308:                                              ; preds = %298, %207
  %309 = load ptr, ptr %8, align 8
  %310 = icmp eq ptr %309, null
  br i1 %310, label %311, label %312

311:                                              ; preds = %308
  store i32 0, ptr %4, align 4
  br label %340

312:                                              ; preds = %308
  %313 = load ptr, ptr %5, align 8
  %314 = load ptr, ptr %8, align 8
  %315 = load ptr, ptr %6, align 8
  %316 = load ptr, ptr %7, align 8
  %317 = load ptr, ptr %5, align 8
  %318 = getelementptr inbounds %struct.lua_State, ptr %317, i32 0, i32 6
  %319 = load ptr, ptr %318, align 8
  call void @luaT_callTMres(ptr noundef %313, ptr noundef %314, ptr noundef %315, ptr noundef %316, ptr noundef %319)
  %320 = load ptr, ptr %5, align 8
  %321 = getelementptr inbounds %struct.lua_State, ptr %320, i32 0, i32 6
  %322 = load ptr, ptr %321, align 8
  %323 = getelementptr inbounds %struct.TValue, ptr %322, i32 0, i32 1
  %324 = load i8, ptr %323, align 8
  %325 = zext i8 %324 to i32
  %326 = icmp eq i32 %325, 1
  br i1 %326, label %336, label %327

327:                                              ; preds = %312
  %328 = load ptr, ptr %5, align 8
  %329 = getelementptr inbounds %struct.lua_State, ptr %328, i32 0, i32 6
  %330 = load ptr, ptr %329, align 8
  %331 = getelementptr inbounds %struct.TValue, ptr %330, i32 0, i32 1
  %332 = load i8, ptr %331, align 8
  %333 = zext i8 %332 to i32
  %334 = and i32 %333, 15
  %335 = icmp eq i32 %334, 0
  br label %336

336:                                              ; preds = %327, %312
  %337 = phi i1 [ true, %312 ], [ %335, %327 ]
  %338 = xor i1 %337, true
  %339 = zext i1 %338 to i32
  store i32 %339, ptr %4, align 4
  br label %340

340:                                              ; preds = %336, %311, %299, %220, %216, %129, %125, %109, %100, %91, %82, %73, %64, %63, %54, %41
  %341 = load i32, ptr %4, align 4
  ret i32 %341
}

declare hidden i32 @luaS_eqlngstr(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_concat(ptr noundef %0, i32 noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i64, align 8
  %12 = alloca [40 x i8], align 16
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store i32 %1, ptr %4, align 4
  %15 = load i32, ptr %4, align 4
  %16 = icmp eq i32 %15, 1
  br i1 %16, label %17, label %18

17:                                               ; preds = %2
  br label %320

18:                                               ; preds = %2
  br label %19

19:                                               ; preds = %317, %18
  %20 = load ptr, ptr %3, align 8
  %21 = getelementptr inbounds %struct.lua_State, ptr %20, i32 0, i32 6
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr %5, align 8
  store i32 2, ptr %6, align 4
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %union.StackValue, ptr %23, i64 -2
  %25 = getelementptr inbounds %struct.TValue, ptr %24, i32 0, i32 1
  %26 = load i8, ptr %25, align 8
  %27 = zext i8 %26 to i32
  %28 = and i32 %27, 15
  %29 = icmp eq i32 %28, 4
  br i1 %29, label %38, label %30

30:                                               ; preds = %19
  %31 = load ptr, ptr %5, align 8
  %32 = getelementptr inbounds %union.StackValue, ptr %31, i64 -2
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 1
  %34 = load i8, ptr %33, align 8
  %35 = zext i8 %34 to i32
  %36 = and i32 %35, 15
  %37 = icmp eq i32 %36, 3
  br i1 %37, label %38, label %58

38:                                               ; preds = %30, %19
  %39 = load ptr, ptr %5, align 8
  %40 = getelementptr inbounds %union.StackValue, ptr %39, i64 -1
  %41 = getelementptr inbounds %struct.TValue, ptr %40, i32 0, i32 1
  %42 = load i8, ptr %41, align 8
  %43 = zext i8 %42 to i32
  %44 = and i32 %43, 15
  %45 = icmp eq i32 %44, 4
  br i1 %45, label %60, label %46

46:                                               ; preds = %38
  %47 = load ptr, ptr %5, align 8
  %48 = getelementptr inbounds %union.StackValue, ptr %47, i64 -1
  %49 = getelementptr inbounds %struct.TValue, ptr %48, i32 0, i32 1
  %50 = load i8, ptr %49, align 8
  %51 = zext i8 %50 to i32
  %52 = and i32 %51, 15
  %53 = icmp eq i32 %52, 3
  br i1 %53, label %54, label %58

54:                                               ; preds = %46
  %55 = load ptr, ptr %3, align 8
  %56 = load ptr, ptr %5, align 8
  %57 = getelementptr inbounds %union.StackValue, ptr %56, i64 -1
  call void @luaO_tostring(ptr noundef %55, ptr noundef %57)
  br i1 true, label %60, label %58

58:                                               ; preds = %54, %46, %30
  %59 = load ptr, ptr %3, align 8
  call void @luaT_tryconcatTM(ptr noundef %59)
  br label %304

60:                                               ; preds = %54, %38
  %61 = load ptr, ptr %5, align 8
  %62 = getelementptr inbounds %union.StackValue, ptr %61, i64 -1
  %63 = getelementptr inbounds %struct.TValue, ptr %62, i32 0, i32 1
  %64 = load i8, ptr %63, align 8
  %65 = zext i8 %64 to i32
  %66 = icmp eq i32 %65, 68
  br i1 %66, label %67, label %101

67:                                               ; preds = %60
  %68 = load ptr, ptr %5, align 8
  %69 = getelementptr inbounds %union.StackValue, ptr %68, i64 -1
  %70 = getelementptr inbounds %struct.TValue, ptr %69, i32 0, i32 0
  %71 = load ptr, ptr %70, align 8
  %72 = getelementptr inbounds %struct.TString, ptr %71, i32 0, i32 4
  %73 = load i8, ptr %72, align 1
  %74 = zext i8 %73 to i32
  %75 = icmp eq i32 %74, 0
  br i1 %75, label %76, label %101

76:                                               ; preds = %67
  %77 = load ptr, ptr %5, align 8
  %78 = getelementptr inbounds %union.StackValue, ptr %77, i64 -2
  %79 = getelementptr inbounds %struct.TValue, ptr %78, i32 0, i32 1
  %80 = load i8, ptr %79, align 8
  %81 = zext i8 %80 to i32
  %82 = and i32 %81, 15
  %83 = icmp eq i32 %82, 4
  br i1 %83, label %98, label %84

84:                                               ; preds = %76
  %85 = load ptr, ptr %5, align 8
  %86 = getelementptr inbounds %union.StackValue, ptr %85, i64 -2
  %87 = getelementptr inbounds %struct.TValue, ptr %86, i32 0, i32 1
  %88 = load i8, ptr %87, align 8
  %89 = zext i8 %88 to i32
  %90 = and i32 %89, 15
  %91 = icmp eq i32 %90, 3
  br i1 %91, label %92, label %96

92:                                               ; preds = %84
  %93 = load ptr, ptr %3, align 8
  %94 = load ptr, ptr %5, align 8
  %95 = getelementptr inbounds %union.StackValue, ptr %94, i64 -2
  call void @luaO_tostring(ptr noundef %93, ptr noundef %95)
  br label %96

96:                                               ; preds = %92, %84
  %97 = phi i1 [ false, %84 ], [ true, %92 ]
  br label %98

98:                                               ; preds = %96, %76
  %99 = phi i1 [ true, %76 ], [ %97, %96 ]
  %100 = zext i1 %99 to i32
  br label %303

101:                                              ; preds = %67, %60
  %102 = load ptr, ptr %5, align 8
  %103 = getelementptr inbounds %union.StackValue, ptr %102, i64 -2
  %104 = getelementptr inbounds %struct.TValue, ptr %103, i32 0, i32 1
  %105 = load i8, ptr %104, align 8
  %106 = zext i8 %105 to i32
  %107 = icmp eq i32 %106, 68
  br i1 %107, label %108, label %132

108:                                              ; preds = %101
  %109 = load ptr, ptr %5, align 8
  %110 = getelementptr inbounds %union.StackValue, ptr %109, i64 -2
  %111 = getelementptr inbounds %struct.TValue, ptr %110, i32 0, i32 0
  %112 = load ptr, ptr %111, align 8
  %113 = getelementptr inbounds %struct.TString, ptr %112, i32 0, i32 4
  %114 = load i8, ptr %113, align 1
  %115 = zext i8 %114 to i32
  %116 = icmp eq i32 %115, 0
  br i1 %116, label %117, label %132

117:                                              ; preds = %108
  %118 = load ptr, ptr %5, align 8
  %119 = getelementptr inbounds %union.StackValue, ptr %118, i64 -2
  store ptr %119, ptr %7, align 8
  %120 = load ptr, ptr %5, align 8
  %121 = getelementptr inbounds %union.StackValue, ptr %120, i64 -1
  store ptr %121, ptr %8, align 8
  %122 = load ptr, ptr %7, align 8
  %123 = getelementptr inbounds %struct.TValue, ptr %122, i32 0, i32 0
  %124 = load ptr, ptr %8, align 8
  %125 = getelementptr inbounds %struct.TValue, ptr %124, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %123, ptr align 8 %125, i64 8, i1 false)
  %126 = load ptr, ptr %8, align 8
  %127 = getelementptr inbounds %struct.TValue, ptr %126, i32 0, i32 1
  %128 = load i8, ptr %127, align 8
  %129 = load ptr, ptr %7, align 8
  %130 = getelementptr inbounds %struct.TValue, ptr %129, i32 0, i32 1
  store i8 %128, ptr %130, align 8
  %131 = load ptr, ptr %3, align 8
  br label %302

132:                                              ; preds = %108, %101
  %133 = load ptr, ptr %5, align 8
  %134 = getelementptr inbounds %union.StackValue, ptr %133, i64 -1
  %135 = getelementptr inbounds %struct.TValue, ptr %134, i32 0, i32 0
  %136 = load ptr, ptr %135, align 8
  %137 = getelementptr inbounds %struct.TString, ptr %136, i32 0, i32 4
  %138 = load i8, ptr %137, align 1
  %139 = zext i8 %138 to i32
  %140 = icmp ne i32 %139, 255
  br i1 %140, label %141, label %149

141:                                              ; preds = %132
  %142 = load ptr, ptr %5, align 8
  %143 = getelementptr inbounds %union.StackValue, ptr %142, i64 -1
  %144 = getelementptr inbounds %struct.TValue, ptr %143, i32 0, i32 0
  %145 = load ptr, ptr %144, align 8
  %146 = getelementptr inbounds %struct.TString, ptr %145, i32 0, i32 4
  %147 = load i8, ptr %146, align 1
  %148 = zext i8 %147 to i64
  br label %156

149:                                              ; preds = %132
  %150 = load ptr, ptr %5, align 8
  %151 = getelementptr inbounds %union.StackValue, ptr %150, i64 -1
  %152 = getelementptr inbounds %struct.TValue, ptr %151, i32 0, i32 0
  %153 = load ptr, ptr %152, align 8
  %154 = getelementptr inbounds %struct.TString, ptr %153, i32 0, i32 6
  %155 = load i64, ptr %154, align 8
  br label %156

156:                                              ; preds = %149, %141
  %157 = phi i64 [ %148, %141 ], [ %155, %149 ]
  store i64 %157, ptr %9, align 8
  store i32 1, ptr %6, align 4
  br label %158

158:                                              ; preds = %260, %156
  %159 = load i32, ptr %6, align 4
  %160 = load i32, ptr %4, align 4
  %161 = icmp slt i32 %159, %160
  br i1 %161, label %162, label %198

162:                                              ; preds = %158
  %163 = load ptr, ptr %5, align 8
  %164 = load i32, ptr %6, align 4
  %165 = sext i32 %164 to i64
  %166 = sub i64 0, %165
  %167 = getelementptr inbounds %union.StackValue, ptr %163, i64 %166
  %168 = getelementptr inbounds %union.StackValue, ptr %167, i64 -1
  %169 = getelementptr inbounds %struct.TValue, ptr %168, i32 0, i32 1
  %170 = load i8, ptr %169, align 8
  %171 = zext i8 %170 to i32
  %172 = and i32 %171, 15
  %173 = icmp eq i32 %172, 4
  br i1 %173, label %196, label %174

174:                                              ; preds = %162
  %175 = load ptr, ptr %5, align 8
  %176 = load i32, ptr %6, align 4
  %177 = sext i32 %176 to i64
  %178 = sub i64 0, %177
  %179 = getelementptr inbounds %union.StackValue, ptr %175, i64 %178
  %180 = getelementptr inbounds %union.StackValue, ptr %179, i64 -1
  %181 = getelementptr inbounds %struct.TValue, ptr %180, i32 0, i32 1
  %182 = load i8, ptr %181, align 8
  %183 = zext i8 %182 to i32
  %184 = and i32 %183, 15
  %185 = icmp eq i32 %184, 3
  br i1 %185, label %186, label %194

186:                                              ; preds = %174
  %187 = load ptr, ptr %3, align 8
  %188 = load ptr, ptr %5, align 8
  %189 = load i32, ptr %6, align 4
  %190 = sext i32 %189 to i64
  %191 = sub i64 0, %190
  %192 = getelementptr inbounds %union.StackValue, ptr %188, i64 %191
  %193 = getelementptr inbounds %union.StackValue, ptr %192, i64 -1
  call void @luaO_tostring(ptr noundef %187, ptr noundef %193)
  br label %194

194:                                              ; preds = %186, %174
  %195 = phi i1 [ false, %174 ], [ true, %186 ]
  br label %196

196:                                              ; preds = %194, %162
  %197 = phi i1 [ true, %162 ], [ %195, %194 ]
  br label %198

198:                                              ; preds = %196, %158
  %199 = phi i1 [ false, %158 ], [ %197, %196 ]
  br i1 %199, label %200, label %263

200:                                              ; preds = %198
  %201 = load ptr, ptr %5, align 8
  %202 = load i32, ptr %6, align 4
  %203 = sext i32 %202 to i64
  %204 = sub i64 0, %203
  %205 = getelementptr inbounds %union.StackValue, ptr %201, i64 %204
  %206 = getelementptr inbounds %union.StackValue, ptr %205, i64 -1
  %207 = getelementptr inbounds %struct.TValue, ptr %206, i32 0, i32 0
  %208 = load ptr, ptr %207, align 8
  %209 = getelementptr inbounds %struct.TString, ptr %208, i32 0, i32 4
  %210 = load i8, ptr %209, align 1
  %211 = zext i8 %210 to i32
  %212 = icmp ne i32 %211, 255
  br i1 %212, label %213, label %225

213:                                              ; preds = %200
  %214 = load ptr, ptr %5, align 8
  %215 = load i32, ptr %6, align 4
  %216 = sext i32 %215 to i64
  %217 = sub i64 0, %216
  %218 = getelementptr inbounds %union.StackValue, ptr %214, i64 %217
  %219 = getelementptr inbounds %union.StackValue, ptr %218, i64 -1
  %220 = getelementptr inbounds %struct.TValue, ptr %219, i32 0, i32 0
  %221 = load ptr, ptr %220, align 8
  %222 = getelementptr inbounds %struct.TString, ptr %221, i32 0, i32 4
  %223 = load i8, ptr %222, align 1
  %224 = zext i8 %223 to i64
  br label %236

225:                                              ; preds = %200
  %226 = load ptr, ptr %5, align 8
  %227 = load i32, ptr %6, align 4
  %228 = sext i32 %227 to i64
  %229 = sub i64 0, %228
  %230 = getelementptr inbounds %union.StackValue, ptr %226, i64 %229
  %231 = getelementptr inbounds %union.StackValue, ptr %230, i64 -1
  %232 = getelementptr inbounds %struct.TValue, ptr %231, i32 0, i32 0
  %233 = load ptr, ptr %232, align 8
  %234 = getelementptr inbounds %struct.TString, ptr %233, i32 0, i32 6
  %235 = load i64, ptr %234, align 8
  br label %236

236:                                              ; preds = %225, %213
  %237 = phi i64 [ %224, %213 ], [ %235, %225 ]
  store i64 %237, ptr %11, align 8
  %238 = load i64, ptr %11, align 8
  %239 = load i64, ptr %9, align 8
  %240 = sub i64 9223372036854775775, %239
  %241 = icmp uge i64 %238, %240
  %242 = zext i1 %241 to i32
  %243 = icmp ne i32 %242, 0
  %244 = zext i1 %243 to i32
  %245 = sext i32 %244 to i64
  %246 = icmp ne i64 %245, 0
  br i1 %246, label %247, label %256

247:                                              ; preds = %236
  %248 = load ptr, ptr %5, align 8
  %249 = load i32, ptr %4, align 4
  %250 = sext i32 %249 to i64
  %251 = sub i64 0, %250
  %252 = getelementptr inbounds %union.StackValue, ptr %248, i64 %251
  %253 = load ptr, ptr %3, align 8
  %254 = getelementptr inbounds %struct.lua_State, ptr %253, i32 0, i32 6
  store ptr %252, ptr %254, align 8
  %255 = load ptr, ptr %3, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %255, ptr noundef @.str.3) #7
  unreachable

256:                                              ; preds = %236
  %257 = load i64, ptr %11, align 8
  %258 = load i64, ptr %9, align 8
  %259 = add i64 %258, %257
  store i64 %259, ptr %9, align 8
  br label %260

260:                                              ; preds = %256
  %261 = load i32, ptr %6, align 4
  %262 = add nsw i32 %261, 1
  store i32 %262, ptr %6, align 4
  br label %158, !llvm.loop !9

263:                                              ; preds = %198
  %264 = load i64, ptr %9, align 8
  %265 = icmp ule i64 %264, 40
  br i1 %265, label %266, label %274

266:                                              ; preds = %263
  %267 = load ptr, ptr %5, align 8
  %268 = load i32, ptr %6, align 4
  %269 = getelementptr inbounds [40 x i8], ptr %12, i64 0, i64 0
  call void @copy2buff(ptr noundef %267, i32 noundef %268, ptr noundef %269)
  %270 = load ptr, ptr %3, align 8
  %271 = getelementptr inbounds [40 x i8], ptr %12, i64 0, i64 0
  %272 = load i64, ptr %9, align 8
  %273 = call ptr @luaS_newlstr(ptr noundef %270, ptr noundef %271, i64 noundef %272)
  store ptr %273, ptr %10, align 8
  br label %283

274:                                              ; preds = %263
  %275 = load ptr, ptr %3, align 8
  %276 = load i64, ptr %9, align 8
  %277 = call ptr @luaS_createlngstrobj(ptr noundef %275, i64 noundef %276)
  store ptr %277, ptr %10, align 8
  %278 = load ptr, ptr %5, align 8
  %279 = load i32, ptr %6, align 4
  %280 = load ptr, ptr %10, align 8
  %281 = getelementptr inbounds %struct.TString, ptr %280, i32 0, i32 7
  %282 = getelementptr inbounds [1 x i8], ptr %281, i64 0, i64 0
  call void @copy2buff(ptr noundef %278, i32 noundef %279, ptr noundef %282)
  br label %283

283:                                              ; preds = %274, %266
  %284 = load ptr, ptr %5, align 8
  %285 = load i32, ptr %6, align 4
  %286 = sext i32 %285 to i64
  %287 = sub i64 0, %286
  %288 = getelementptr inbounds %union.StackValue, ptr %284, i64 %287
  store ptr %288, ptr %13, align 8
  %289 = load ptr, ptr %10, align 8
  store ptr %289, ptr %14, align 8
  %290 = load ptr, ptr %14, align 8
  %291 = load ptr, ptr %13, align 8
  %292 = getelementptr inbounds %struct.TValue, ptr %291, i32 0, i32 0
  store ptr %290, ptr %292, align 8
  %293 = load ptr, ptr %14, align 8
  %294 = getelementptr inbounds %struct.TString, ptr %293, i32 0, i32 1
  %295 = load i8, ptr %294, align 8
  %296 = zext i8 %295 to i32
  %297 = or i32 %296, 64
  %298 = trunc i32 %297 to i8
  %299 = load ptr, ptr %13, align 8
  %300 = getelementptr inbounds %struct.TValue, ptr %299, i32 0, i32 1
  store i8 %298, ptr %300, align 8
  %301 = load ptr, ptr %3, align 8
  br label %302

302:                                              ; preds = %283, %117
  br label %303

303:                                              ; preds = %302, %98
  br label %304

304:                                              ; preds = %303, %58
  %305 = load i32, ptr %6, align 4
  %306 = sub nsw i32 %305, 1
  %307 = load i32, ptr %4, align 4
  %308 = sub nsw i32 %307, %306
  store i32 %308, ptr %4, align 4
  %309 = load i32, ptr %6, align 4
  %310 = sub nsw i32 %309, 1
  %311 = load ptr, ptr %3, align 8
  %312 = getelementptr inbounds %struct.lua_State, ptr %311, i32 0, i32 6
  %313 = load ptr, ptr %312, align 8
  %314 = sext i32 %310 to i64
  %315 = sub i64 0, %314
  %316 = getelementptr inbounds %union.StackValue, ptr %313, i64 %315
  store ptr %316, ptr %312, align 8
  br label %317

317:                                              ; preds = %304
  %318 = load i32, ptr %4, align 4
  %319 = icmp sgt i32 %318, 1
  br i1 %319, label %19, label %320, !llvm.loop !10

320:                                              ; preds = %17, %317
  ret void
}

declare hidden void @luaO_tostring(ptr noundef, ptr noundef) #2

declare hidden void @luaT_tryconcatTM(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @copy2buff(ptr noundef %0, i32 noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store ptr %2, ptr %6, align 8
  store i64 0, ptr %7, align 8
  br label %10

10:                                               ; preds = %45, %3
  %11 = load ptr, ptr %4, align 8
  %12 = load i32, ptr %5, align 4
  %13 = sext i32 %12 to i64
  %14 = sub i64 0, %13
  %15 = getelementptr inbounds %union.StackValue, ptr %11, i64 %14
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  store ptr %17, ptr %8, align 8
  %18 = load ptr, ptr %8, align 8
  %19 = getelementptr inbounds %struct.TString, ptr %18, i32 0, i32 4
  %20 = load i8, ptr %19, align 1
  %21 = zext i8 %20 to i32
  %22 = icmp ne i32 %21, 255
  br i1 %22, label %23, label %28

23:                                               ; preds = %10
  %24 = load ptr, ptr %8, align 8
  %25 = getelementptr inbounds %struct.TString, ptr %24, i32 0, i32 4
  %26 = load i8, ptr %25, align 1
  %27 = zext i8 %26 to i64
  br label %32

28:                                               ; preds = %10
  %29 = load ptr, ptr %8, align 8
  %30 = getelementptr inbounds %struct.TString, ptr %29, i32 0, i32 6
  %31 = load i64, ptr %30, align 8
  br label %32

32:                                               ; preds = %28, %23
  %33 = phi i64 [ %27, %23 ], [ %31, %28 ]
  store i64 %33, ptr %9, align 8
  %34 = load ptr, ptr %6, align 8
  %35 = load i64, ptr %7, align 8
  %36 = getelementptr inbounds i8, ptr %34, i64 %35
  %37 = load ptr, ptr %8, align 8
  %38 = getelementptr inbounds %struct.TString, ptr %37, i32 0, i32 7
  %39 = getelementptr inbounds [1 x i8], ptr %38, i64 0, i64 0
  %40 = load i64, ptr %9, align 8
  %41 = mul i64 %40, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %36, ptr align 8 %39, i64 %41, i1 false)
  %42 = load i64, ptr %9, align 8
  %43 = load i64, ptr %7, align 8
  %44 = add i64 %43, %42
  store i64 %44, ptr %7, align 8
  br label %45

45:                                               ; preds = %32
  %46 = load i32, ptr %5, align 4
  %47 = add nsw i32 %46, -1
  store i32 %47, ptr %5, align 4
  %48 = icmp sgt i32 %47, 0
  br i1 %48, label %10, label %49, !llvm.loop !11

49:                                               ; preds = %45
  ret void
}

declare hidden ptr @luaS_newlstr(ptr noundef, ptr noundef, i64 noundef) #2

declare hidden ptr @luaS_createlngstrobj(ptr noundef, i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_objlen(ptr noundef %0, ptr noundef %1, ptr noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store ptr %2, ptr %6, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.TValue, ptr %12, i32 0, i32 1
  %14 = load i8, ptr %13, align 8
  %15 = zext i8 %14 to i32
  %16 = and i32 %15, 63
  switch i32 %16, label %85 [
    i32 5, label %17
    i32 4, label %62
    i32 20, label %74
  ]

17:                                               ; preds = %3
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.TValue, ptr %18, i32 0, i32 0
  %20 = load ptr, ptr %19, align 8
  store ptr %20, ptr %8, align 8
  %21 = load ptr, ptr %8, align 8
  %22 = getelementptr inbounds %struct.Table, ptr %21, i32 0, i32 9
  %23 = load ptr, ptr %22, align 8
  %24 = icmp eq ptr %23, null
  br i1 %24, label %25, label %26

25:                                               ; preds = %17
  br label %49

26:                                               ; preds = %17
  %27 = load ptr, ptr %8, align 8
  %28 = getelementptr inbounds %struct.Table, ptr %27, i32 0, i32 9
  %29 = load ptr, ptr %28, align 8
  %30 = getelementptr inbounds %struct.Table, ptr %29, i32 0, i32 3
  %31 = load i8, ptr %30, align 2
  %32 = zext i8 %31 to i32
  %33 = and i32 %32, 16
  %34 = icmp ne i32 %33, 0
  br i1 %34, label %35, label %36

35:                                               ; preds = %26
  br label %47

36:                                               ; preds = %26
  %37 = load ptr, ptr %8, align 8
  %38 = getelementptr inbounds %struct.Table, ptr %37, i32 0, i32 9
  %39 = load ptr, ptr %38, align 8
  %40 = load ptr, ptr %4, align 8
  %41 = getelementptr inbounds %struct.lua_State, ptr %40, i32 0, i32 7
  %42 = load ptr, ptr %41, align 8
  %43 = getelementptr inbounds %struct.global_State, ptr %42, i32 0, i32 42
  %44 = getelementptr inbounds [25 x ptr], ptr %43, i64 0, i64 4
  %45 = load ptr, ptr %44, align 8
  %46 = call ptr @luaT_gettm(ptr noundef %39, i32 noundef 4, ptr noundef %45)
  br label %47

47:                                               ; preds = %36, %35
  %48 = phi ptr [ null, %35 ], [ %46, %36 ]
  br label %49

49:                                               ; preds = %47, %25
  %50 = phi ptr [ null, %25 ], [ %48, %47 ]
  store ptr %50, ptr %7, align 8
  %51 = load ptr, ptr %7, align 8
  %52 = icmp ne ptr %51, null
  br i1 %52, label %53, label %54

53:                                               ; preds = %49
  br label %104

54:                                               ; preds = %49
  %55 = load ptr, ptr %5, align 8
  store ptr %55, ptr %9, align 8
  %56 = load ptr, ptr %8, align 8
  %57 = call i64 @luaH_getn(ptr noundef %56)
  %58 = load ptr, ptr %9, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 0
  store i64 %57, ptr %59, align 8
  %60 = load ptr, ptr %9, align 8
  %61 = getelementptr inbounds %struct.TValue, ptr %60, i32 0, i32 1
  store i8 3, ptr %61, align 8
  br label %110

62:                                               ; preds = %3
  %63 = load ptr, ptr %5, align 8
  store ptr %63, ptr %10, align 8
  %64 = load ptr, ptr %6, align 8
  %65 = getelementptr inbounds %struct.TValue, ptr %64, i32 0, i32 0
  %66 = load ptr, ptr %65, align 8
  %67 = getelementptr inbounds %struct.TString, ptr %66, i32 0, i32 4
  %68 = load i8, ptr %67, align 1
  %69 = zext i8 %68 to i64
  %70 = load ptr, ptr %10, align 8
  %71 = getelementptr inbounds %struct.TValue, ptr %70, i32 0, i32 0
  store i64 %69, ptr %71, align 8
  %72 = load ptr, ptr %10, align 8
  %73 = getelementptr inbounds %struct.TValue, ptr %72, i32 0, i32 1
  store i8 3, ptr %73, align 8
  br label %110

74:                                               ; preds = %3
  %75 = load ptr, ptr %5, align 8
  store ptr %75, ptr %11, align 8
  %76 = load ptr, ptr %6, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 0
  %78 = load ptr, ptr %77, align 8
  %79 = getelementptr inbounds %struct.TString, ptr %78, i32 0, i32 6
  %80 = load i64, ptr %79, align 8
  %81 = load ptr, ptr %11, align 8
  %82 = getelementptr inbounds %struct.TValue, ptr %81, i32 0, i32 0
  store i64 %80, ptr %82, align 8
  %83 = load ptr, ptr %11, align 8
  %84 = getelementptr inbounds %struct.TValue, ptr %83, i32 0, i32 1
  store i8 3, ptr %84, align 8
  br label %110

85:                                               ; preds = %3
  %86 = load ptr, ptr %4, align 8
  %87 = load ptr, ptr %6, align 8
  %88 = call ptr @luaT_gettmbyobj(ptr noundef %86, ptr noundef %87, i32 noundef 4)
  store ptr %88, ptr %7, align 8
  %89 = load ptr, ptr %7, align 8
  %90 = getelementptr inbounds %struct.TValue, ptr %89, i32 0, i32 1
  %91 = load i8, ptr %90, align 8
  %92 = zext i8 %91 to i32
  %93 = and i32 %92, 15
  %94 = icmp eq i32 %93, 0
  %95 = zext i1 %94 to i32
  %96 = icmp ne i32 %95, 0
  %97 = zext i1 %96 to i32
  %98 = sext i32 %97 to i64
  %99 = icmp ne i64 %98, 0
  br i1 %99, label %100, label %103

100:                                              ; preds = %85
  %101 = load ptr, ptr %4, align 8
  %102 = load ptr, ptr %6, align 8
  call void @luaG_typeerror(ptr noundef %101, ptr noundef %102, ptr noundef @.str.4) #7
  unreachable

103:                                              ; preds = %85
  br label %104

104:                                              ; preds = %103, %53
  %105 = load ptr, ptr %4, align 8
  %106 = load ptr, ptr %7, align 8
  %107 = load ptr, ptr %6, align 8
  %108 = load ptr, ptr %6, align 8
  %109 = load ptr, ptr %5, align 8
  call void @luaT_callTMres(ptr noundef %105, ptr noundef %106, ptr noundef %107, ptr noundef %108, ptr noundef %109)
  br label %110

110:                                              ; preds = %104, %74, %62, %54
  ret void
}

declare hidden i64 @luaH_getn(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaV_idiv(ptr noundef %0, i64 noundef %1, i64 noundef %2) #0 {
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  %9 = load i64, ptr %7, align 8
  %10 = add i64 %9, 1
  %11 = icmp ule i64 %10, 1
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %25

17:                                               ; preds = %3
  %18 = load i64, ptr %7, align 8
  %19 = icmp eq i64 %18, 0
  br i1 %19, label %20, label %22

20:                                               ; preds = %17
  %21 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %21, ptr noundef @.str.5) #7
  unreachable

22:                                               ; preds = %17
  %23 = load i64, ptr %6, align 8
  %24 = sub i64 0, %23
  store i64 %24, ptr %4, align 8
  br label %43

25:                                               ; preds = %3
  %26 = load i64, ptr %6, align 8
  %27 = load i64, ptr %7, align 8
  %28 = sdiv i64 %26, %27
  store i64 %28, ptr %8, align 8
  %29 = load i64, ptr %6, align 8
  %30 = load i64, ptr %7, align 8
  %31 = xor i64 %29, %30
  %32 = icmp slt i64 %31, 0
  br i1 %32, label %33, label %41

33:                                               ; preds = %25
  %34 = load i64, ptr %6, align 8
  %35 = load i64, ptr %7, align 8
  %36 = srem i64 %34, %35
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %38, label %41

38:                                               ; preds = %33
  %39 = load i64, ptr %8, align 8
  %40 = sub nsw i64 %39, 1
  store i64 %40, ptr %8, align 8
  br label %41

41:                                               ; preds = %38, %33, %25
  %42 = load i64, ptr %8, align 8
  store i64 %42, ptr %4, align 8
  br label %43

43:                                               ; preds = %41, %22
  %44 = load i64, ptr %4, align 8
  ret i64 %44
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaV_mod(ptr noundef %0, i64 noundef %1, i64 noundef %2) #0 {
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  %9 = load i64, ptr %7, align 8
  %10 = add i64 %9, 1
  %11 = icmp ule i64 %10, 1
  %12 = zext i1 %11 to i32
  %13 = icmp ne i32 %12, 0
  %14 = zext i1 %13 to i32
  %15 = sext i32 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %17, label %23

17:                                               ; preds = %3
  %18 = load i64, ptr %7, align 8
  %19 = icmp eq i64 %18, 0
  br i1 %19, label %20, label %22

20:                                               ; preds = %17
  %21 = load ptr, ptr %5, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %21, ptr noundef @.str.6) #7
  unreachable

22:                                               ; preds = %17
  store i64 0, ptr %4, align 8
  br label %40

23:                                               ; preds = %3
  %24 = load i64, ptr %6, align 8
  %25 = load i64, ptr %7, align 8
  %26 = srem i64 %24, %25
  store i64 %26, ptr %8, align 8
  %27 = load i64, ptr %8, align 8
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %29, label %38

29:                                               ; preds = %23
  %30 = load i64, ptr %8, align 8
  %31 = load i64, ptr %7, align 8
  %32 = xor i64 %30, %31
  %33 = icmp slt i64 %32, 0
  br i1 %33, label %34, label %38

34:                                               ; preds = %29
  %35 = load i64, ptr %7, align 8
  %36 = load i64, ptr %8, align 8
  %37 = add nsw i64 %36, %35
  store i64 %37, ptr %8, align 8
  br label %38

38:                                               ; preds = %34, %29, %23
  %39 = load i64, ptr %8, align 8
  store i64 %39, ptr %4, align 8
  br label %40

40:                                               ; preds = %38, %22
  %41 = load i64, ptr %4, align 8
  ret i64 %41
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden double @luaV_modf(ptr noundef %0, double noundef %1, double noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca double, align 8
  %6 = alloca double, align 8
  %7 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  store double %1, ptr %5, align 8
  store double %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = load double, ptr %5, align 8
  %10 = load double, ptr %6, align 8
  %11 = call double @fmod(double noundef %9, double noundef %10) #8
  store double %11, ptr %7, align 8
  %12 = load double, ptr %7, align 8
  %13 = fcmp ogt double %12, 0.000000e+00
  br i1 %13, label %14, label %17

14:                                               ; preds = %3
  %15 = load double, ptr %6, align 8
  %16 = fcmp olt double %15, 0.000000e+00
  br i1 %16, label %23, label %27

17:                                               ; preds = %3
  %18 = load double, ptr %7, align 8
  %19 = fcmp olt double %18, 0.000000e+00
  br i1 %19, label %20, label %27

20:                                               ; preds = %17
  %21 = load double, ptr %6, align 8
  %22 = fcmp ogt double %21, 0.000000e+00
  br i1 %22, label %23, label %27

23:                                               ; preds = %20, %14
  %24 = load double, ptr %6, align 8
  %25 = load double, ptr %7, align 8
  %26 = fadd double %25, %24
  store double %26, ptr %7, align 8
  br label %27

27:                                               ; preds = %23, %20, %17, %14
  %28 = load double, ptr %7, align 8
  ret double %28
}

; Function Attrs: nounwind
declare double @fmod(double noundef, double noundef) #5

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaV_shiftl(i64 noundef %0, i64 noundef %1) #0 {
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  store i64 %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %6 = load i64, ptr %5, align 8
  %7 = icmp slt i64 %6, 0
  br i1 %7, label %8, label %17

8:                                                ; preds = %2
  %9 = load i64, ptr %5, align 8
  %10 = icmp sle i64 %9, -64
  br i1 %10, label %11, label %12

11:                                               ; preds = %8
  store i64 0, ptr %3, align 8
  br label %25

12:                                               ; preds = %8
  %13 = load i64, ptr %4, align 8
  %14 = load i64, ptr %5, align 8
  %15 = sub nsw i64 0, %14
  %16 = lshr i64 %13, %15
  store i64 %16, ptr %3, align 8
  br label %25

17:                                               ; preds = %2
  %18 = load i64, ptr %5, align 8
  %19 = icmp sge i64 %18, 64
  br i1 %19, label %20, label %21

20:                                               ; preds = %17
  store i64 0, ptr %3, align 8
  br label %25

21:                                               ; preds = %17
  %22 = load i64, ptr %4, align 8
  %23 = load i64, ptr %5, align 8
  %24 = shl i64 %22, %23
  store i64 %24, ptr %3, align 8
  br label %25

25:                                               ; preds = %21, %20, %12, %11
  %26 = load i64, ptr %3, align 8
  ret i64 %26
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_finishOp(ptr noundef %0) #0 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %18 = load ptr, ptr %2, align 8
  %19 = getelementptr inbounds %struct.lua_State, ptr %18, i32 0, i32 8
  %20 = load ptr, ptr %19, align 8
  store ptr %20, ptr %3, align 8
  %21 = load ptr, ptr %3, align 8
  %22 = getelementptr inbounds %struct.CallInfo, ptr %21, i32 0, i32 0
  %23 = load ptr, ptr %22, align 8
  %24 = getelementptr inbounds %union.StackValue, ptr %23, i64 1
  store ptr %24, ptr %4, align 8
  %25 = load ptr, ptr %3, align 8
  %26 = getelementptr inbounds %struct.CallInfo, ptr %25, i32 0, i32 4
  %27 = getelementptr inbounds %struct.anon, ptr %26, i32 0, i32 0
  %28 = load ptr, ptr %27, align 8
  %29 = getelementptr inbounds i32, ptr %28, i64 -1
  %30 = load i32, ptr %29, align 4
  store i32 %30, ptr %5, align 4
  %31 = load i32, ptr %5, align 4
  %32 = lshr i32 %31, 0
  %33 = and i32 %32, 127
  store i32 %33, ptr %6, align 4
  %34 = load i32, ptr %6, align 4
  switch i32 %34, label %185 [
    i32 46, label %35
    i32 47, label %35
    i32 48, label %35
    i32 49, label %61
    i32 50, label %61
    i32 52, label %61
    i32 11, label %61
    i32 12, label %61
    i32 13, label %61
    i32 14, label %61
    i32 20, label %61
    i32 58, label %82
    i32 59, label %82
    i32 62, label %82
    i32 63, label %82
    i32 64, label %82
    i32 65, label %82
    i32 57, label %82
    i32 53, label %121
    i32 54, label %159
    i32 70, label %165
  ]

35:                                               ; preds = %1, %1, %1
  %36 = load ptr, ptr %4, align 8
  %37 = load ptr, ptr %3, align 8
  %38 = getelementptr inbounds %struct.CallInfo, ptr %37, i32 0, i32 4
  %39 = getelementptr inbounds %struct.anon, ptr %38, i32 0, i32 0
  %40 = load ptr, ptr %39, align 8
  %41 = getelementptr inbounds i32, ptr %40, i64 -2
  %42 = load i32, ptr %41, align 4
  %43 = lshr i32 %42, 7
  %44 = and i32 %43, 255
  %45 = sext i32 %44 to i64
  %46 = getelementptr inbounds %union.StackValue, ptr %36, i64 %45
  store ptr %46, ptr %7, align 8
  %47 = load ptr, ptr %2, align 8
  %48 = getelementptr inbounds %struct.lua_State, ptr %47, i32 0, i32 6
  %49 = load ptr, ptr %48, align 8
  %50 = getelementptr inbounds %union.StackValue, ptr %49, i32 -1
  store ptr %50, ptr %48, align 8
  store ptr %50, ptr %8, align 8
  %51 = load ptr, ptr %7, align 8
  %52 = getelementptr inbounds %struct.TValue, ptr %51, i32 0, i32 0
  %53 = load ptr, ptr %8, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %52, ptr align 8 %54, i64 8, i1 false)
  %55 = load ptr, ptr %8, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  %57 = load i8, ptr %56, align 8
  %58 = load ptr, ptr %7, align 8
  %59 = getelementptr inbounds %struct.TValue, ptr %58, i32 0, i32 1
  store i8 %57, ptr %59, align 8
  %60 = load ptr, ptr %2, align 8
  br label %186

61:                                               ; preds = %1, %1, %1, %1, %1, %1, %1, %1
  %62 = load ptr, ptr %4, align 8
  %63 = load i32, ptr %5, align 4
  %64 = lshr i32 %63, 7
  %65 = and i32 %64, 255
  %66 = sext i32 %65 to i64
  %67 = getelementptr inbounds %union.StackValue, ptr %62, i64 %66
  store ptr %67, ptr %9, align 8
  %68 = load ptr, ptr %2, align 8
  %69 = getelementptr inbounds %struct.lua_State, ptr %68, i32 0, i32 6
  %70 = load ptr, ptr %69, align 8
  %71 = getelementptr inbounds %union.StackValue, ptr %70, i32 -1
  store ptr %71, ptr %69, align 8
  store ptr %71, ptr %10, align 8
  %72 = load ptr, ptr %9, align 8
  %73 = getelementptr inbounds %struct.TValue, ptr %72, i32 0, i32 0
  %74 = load ptr, ptr %10, align 8
  %75 = getelementptr inbounds %struct.TValue, ptr %74, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %73, ptr align 8 %75, i64 8, i1 false)
  %76 = load ptr, ptr %10, align 8
  %77 = getelementptr inbounds %struct.TValue, ptr %76, i32 0, i32 1
  %78 = load i8, ptr %77, align 8
  %79 = load ptr, ptr %9, align 8
  %80 = getelementptr inbounds %struct.TValue, ptr %79, i32 0, i32 1
  store i8 %78, ptr %80, align 8
  %81 = load ptr, ptr %2, align 8
  br label %186

82:                                               ; preds = %1, %1, %1, %1, %1, %1, %1
  %83 = load ptr, ptr %2, align 8
  %84 = getelementptr inbounds %struct.lua_State, ptr %83, i32 0, i32 6
  %85 = load ptr, ptr %84, align 8
  %86 = getelementptr inbounds %union.StackValue, ptr %85, i64 -1
  %87 = getelementptr inbounds %struct.TValue, ptr %86, i32 0, i32 1
  %88 = load i8, ptr %87, align 8
  %89 = zext i8 %88 to i32
  %90 = icmp eq i32 %89, 1
  br i1 %90, label %101, label %91

91:                                               ; preds = %82
  %92 = load ptr, ptr %2, align 8
  %93 = getelementptr inbounds %struct.lua_State, ptr %92, i32 0, i32 6
  %94 = load ptr, ptr %93, align 8
  %95 = getelementptr inbounds %union.StackValue, ptr %94, i64 -1
  %96 = getelementptr inbounds %struct.TValue, ptr %95, i32 0, i32 1
  %97 = load i8, ptr %96, align 8
  %98 = zext i8 %97 to i32
  %99 = and i32 %98, 15
  %100 = icmp eq i32 %99, 0
  br label %101

101:                                              ; preds = %91, %82
  %102 = phi i1 [ true, %82 ], [ %100, %91 ]
  %103 = xor i1 %102, true
  %104 = zext i1 %103 to i32
  store i32 %104, ptr %11, align 4
  %105 = load ptr, ptr %2, align 8
  %106 = getelementptr inbounds %struct.lua_State, ptr %105, i32 0, i32 6
  %107 = load ptr, ptr %106, align 8
  %108 = getelementptr inbounds %union.StackValue, ptr %107, i32 -1
  store ptr %108, ptr %106, align 8
  %109 = load i32, ptr %11, align 4
  %110 = load i32, ptr %5, align 4
  %111 = lshr i32 %110, 15
  %112 = and i32 %111, 1
  %113 = icmp ne i32 %109, %112
  br i1 %113, label %114, label %120

114:                                              ; preds = %101
  %115 = load ptr, ptr %3, align 8
  %116 = getelementptr inbounds %struct.CallInfo, ptr %115, i32 0, i32 4
  %117 = getelementptr inbounds %struct.anon, ptr %116, i32 0, i32 0
  %118 = load ptr, ptr %117, align 8
  %119 = getelementptr inbounds i32, ptr %118, i32 1
  store ptr %119, ptr %117, align 8
  br label %120

120:                                              ; preds = %114, %101
  br label %186

121:                                              ; preds = %1
  %122 = load ptr, ptr %2, align 8
  %123 = getelementptr inbounds %struct.lua_State, ptr %122, i32 0, i32 6
  %124 = load ptr, ptr %123, align 8
  %125 = getelementptr inbounds %union.StackValue, ptr %124, i64 -1
  store ptr %125, ptr %12, align 8
  %126 = load i32, ptr %5, align 4
  %127 = lshr i32 %126, 7
  %128 = and i32 %127, 255
  store i32 %128, ptr %13, align 4
  %129 = load ptr, ptr %12, align 8
  %130 = getelementptr inbounds %union.StackValue, ptr %129, i64 -1
  %131 = load ptr, ptr %4, align 8
  %132 = load i32, ptr %13, align 4
  %133 = sext i32 %132 to i64
  %134 = getelementptr inbounds %union.StackValue, ptr %131, i64 %133
  %135 = ptrtoint ptr %130 to i64
  %136 = ptrtoint ptr %134 to i64
  %137 = sub i64 %135, %136
  %138 = sdiv exact i64 %137, 16
  %139 = trunc i64 %138 to i32
  store i32 %139, ptr %14, align 4
  %140 = load ptr, ptr %12, align 8
  %141 = getelementptr inbounds %union.StackValue, ptr %140, i64 -2
  store ptr %141, ptr %15, align 8
  %142 = load ptr, ptr %12, align 8
  store ptr %142, ptr %16, align 8
  %143 = load ptr, ptr %15, align 8
  %144 = getelementptr inbounds %struct.TValue, ptr %143, i32 0, i32 0
  %145 = load ptr, ptr %16, align 8
  %146 = getelementptr inbounds %struct.TValue, ptr %145, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %144, ptr align 8 %146, i64 8, i1 false)
  %147 = load ptr, ptr %16, align 8
  %148 = getelementptr inbounds %struct.TValue, ptr %147, i32 0, i32 1
  %149 = load i8, ptr %148, align 8
  %150 = load ptr, ptr %15, align 8
  %151 = getelementptr inbounds %struct.TValue, ptr %150, i32 0, i32 1
  store i8 %149, ptr %151, align 8
  %152 = load ptr, ptr %2, align 8
  %153 = load ptr, ptr %12, align 8
  %154 = getelementptr inbounds %union.StackValue, ptr %153, i64 -1
  %155 = load ptr, ptr %2, align 8
  %156 = getelementptr inbounds %struct.lua_State, ptr %155, i32 0, i32 6
  store ptr %154, ptr %156, align 8
  %157 = load ptr, ptr %2, align 8
  %158 = load i32, ptr %14, align 4
  call void @luaV_concat(ptr noundef %157, i32 noundef %158)
  br label %186

159:                                              ; preds = %1
  %160 = load ptr, ptr %3, align 8
  %161 = getelementptr inbounds %struct.CallInfo, ptr %160, i32 0, i32 4
  %162 = getelementptr inbounds %struct.anon, ptr %161, i32 0, i32 0
  %163 = load ptr, ptr %162, align 8
  %164 = getelementptr inbounds i32, ptr %163, i32 -1
  store ptr %164, ptr %162, align 8
  br label %186

165:                                              ; preds = %1
  %166 = load ptr, ptr %4, align 8
  %167 = load i32, ptr %5, align 4
  %168 = lshr i32 %167, 7
  %169 = and i32 %168, 255
  %170 = sext i32 %169 to i64
  %171 = getelementptr inbounds %union.StackValue, ptr %166, i64 %170
  store ptr %171, ptr %17, align 8
  %172 = load ptr, ptr %17, align 8
  %173 = load ptr, ptr %3, align 8
  %174 = getelementptr inbounds %struct.CallInfo, ptr %173, i32 0, i32 5
  %175 = load i32, ptr %174, align 8
  %176 = sext i32 %175 to i64
  %177 = getelementptr inbounds %union.StackValue, ptr %172, i64 %176
  %178 = load ptr, ptr %2, align 8
  %179 = getelementptr inbounds %struct.lua_State, ptr %178, i32 0, i32 6
  store ptr %177, ptr %179, align 8
  %180 = load ptr, ptr %3, align 8
  %181 = getelementptr inbounds %struct.CallInfo, ptr %180, i32 0, i32 4
  %182 = getelementptr inbounds %struct.anon, ptr %181, i32 0, i32 0
  %183 = load ptr, ptr %182, align 8
  %184 = getelementptr inbounds i32, ptr %183, i32 -1
  store ptr %184, ptr %182, align 8
  br label %186

185:                                              ; preds = %1
  br label %186

186:                                              ; preds = %185, %165, %159, %121, %120, %61, %35
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaV_execute(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca ptr, align 8
  %13 = alloca ptr, align 8
  %14 = alloca ptr, align 8
  %15 = alloca i64, align 8
  %16 = alloca ptr, align 8
  %17 = alloca ptr, align 8
  %18 = alloca i32, align 4
  %19 = alloca ptr, align 8
  %20 = alloca ptr, align 8
  %21 = alloca ptr, align 8
  %22 = alloca ptr, align 8
  %23 = alloca ptr, align 8
  %24 = alloca ptr, align 8
  %25 = alloca ptr, align 8
  %26 = alloca ptr, align 8
  %27 = alloca ptr, align 8
  %28 = alloca ptr, align 8
  %29 = alloca ptr, align 8
  %30 = alloca ptr, align 8
  %31 = alloca ptr, align 8
  %32 = alloca i32, align 4
  %33 = alloca ptr, align 8
  %34 = alloca i32, align 4
  %35 = alloca ptr, align 8
  %36 = alloca ptr, align 8
  %37 = alloca ptr, align 8
  %38 = alloca ptr, align 8
  %39 = alloca ptr, align 8
  %40 = alloca ptr, align 8
  %41 = alloca ptr, align 8
  %42 = alloca ptr, align 8
  %43 = alloca ptr, align 8
  %44 = alloca ptr, align 8
  %45 = alloca ptr, align 8
  %46 = alloca ptr, align 8
  %47 = alloca ptr, align 8
  %48 = alloca ptr, align 8
  %49 = alloca ptr, align 8
  %50 = alloca ptr, align 8
  %51 = alloca ptr, align 8
  %52 = alloca i64, align 8
  %53 = alloca ptr, align 8
  %54 = alloca ptr, align 8
  %55 = alloca ptr, align 8
  %56 = alloca ptr, align 8
  %57 = alloca ptr, align 8
  %58 = alloca i32, align 4
  %59 = alloca ptr, align 8
  %60 = alloca ptr, align 8
  %61 = alloca %struct.TValue, align 8
  %62 = alloca ptr, align 8
  %63 = alloca ptr, align 8
  %64 = alloca ptr, align 8
  %65 = alloca ptr, align 8
  %66 = alloca ptr, align 8
  %67 = alloca ptr, align 8
  %68 = alloca ptr, align 8
  %69 = alloca ptr, align 8
  %70 = alloca ptr, align 8
  %71 = alloca ptr, align 8
  %72 = alloca ptr, align 8
  %73 = alloca ptr, align 8
  %74 = alloca ptr, align 8
  %75 = alloca ptr, align 8
  %76 = alloca ptr, align 8
  %77 = alloca ptr, align 8
  %78 = alloca ptr, align 8
  %79 = alloca ptr, align 8
  %80 = alloca ptr, align 8
  %81 = alloca i64, align 8
  %82 = alloca ptr, align 8
  %83 = alloca ptr, align 8
  %84 = alloca ptr, align 8
  %85 = alloca ptr, align 8
  %86 = alloca i32, align 4
  %87 = alloca ptr, align 8
  %88 = alloca ptr, align 8
  %89 = alloca ptr, align 8
  %90 = alloca %struct.TValue, align 8
  %91 = alloca ptr, align 8
  %92 = alloca ptr, align 8
  %93 = alloca ptr, align 8
  %94 = alloca ptr, align 8
  %95 = alloca ptr, align 8
  %96 = alloca ptr, align 8
  %97 = alloca ptr, align 8
  %98 = alloca ptr, align 8
  %99 = alloca ptr, align 8
  %100 = alloca i32, align 4
  %101 = alloca i32, align 4
  %102 = alloca ptr, align 8
  %103 = alloca ptr, align 8
  %104 = alloca ptr, align 8
  %105 = alloca ptr, align 8
  %106 = alloca ptr, align 8
  %107 = alloca ptr, align 8
  %108 = alloca ptr, align 8
  %109 = alloca ptr, align 8
  %110 = alloca ptr, align 8
  %111 = alloca ptr, align 8
  %112 = alloca ptr, align 8
  %113 = alloca ptr, align 8
  %114 = alloca ptr, align 8
  %115 = alloca ptr, align 8
  %116 = alloca i32, align 4
  %117 = alloca i64, align 8
  %118 = alloca ptr, align 8
  %119 = alloca double, align 8
  %120 = alloca double, align 8
  %121 = alloca ptr, align 8
  %122 = alloca ptr, align 8
  %123 = alloca ptr, align 8
  %124 = alloca ptr, align 8
  %125 = alloca i64, align 8
  %126 = alloca i64, align 8
  %127 = alloca ptr, align 8
  %128 = alloca double, align 8
  %129 = alloca double, align 8
  %130 = alloca ptr, align 8
  %131 = alloca ptr, align 8
  %132 = alloca ptr, align 8
  %133 = alloca ptr, align 8
  %134 = alloca i64, align 8
  %135 = alloca i64, align 8
  %136 = alloca ptr, align 8
  %137 = alloca double, align 8
  %138 = alloca double, align 8
  %139 = alloca ptr, align 8
  %140 = alloca ptr, align 8
  %141 = alloca ptr, align 8
  %142 = alloca ptr, align 8
  %143 = alloca i64, align 8
  %144 = alloca i64, align 8
  %145 = alloca ptr, align 8
  %146 = alloca double, align 8
  %147 = alloca double, align 8
  %148 = alloca ptr, align 8
  %149 = alloca ptr, align 8
  %150 = alloca ptr, align 8
  %151 = alloca ptr, align 8
  %152 = alloca i64, align 8
  %153 = alloca i64, align 8
  %154 = alloca ptr, align 8
  %155 = alloca double, align 8
  %156 = alloca double, align 8
  %157 = alloca ptr, align 8
  %158 = alloca ptr, align 8
  %159 = alloca ptr, align 8
  %160 = alloca ptr, align 8
  %161 = alloca double, align 8
  %162 = alloca double, align 8
  %163 = alloca ptr, align 8
  %164 = alloca ptr, align 8
  %165 = alloca ptr, align 8
  %166 = alloca ptr, align 8
  %167 = alloca double, align 8
  %168 = alloca double, align 8
  %169 = alloca ptr, align 8
  %170 = alloca ptr, align 8
  %171 = alloca ptr, align 8
  %172 = alloca ptr, align 8
  %173 = alloca i64, align 8
  %174 = alloca i64, align 8
  %175 = alloca ptr, align 8
  %176 = alloca double, align 8
  %177 = alloca double, align 8
  %178 = alloca ptr, align 8
  %179 = alloca ptr, align 8
  %180 = alloca ptr, align 8
  %181 = alloca ptr, align 8
  %182 = alloca i64, align 8
  %183 = alloca i64, align 8
  %184 = alloca ptr, align 8
  %185 = alloca ptr, align 8
  %186 = alloca ptr, align 8
  %187 = alloca ptr, align 8
  %188 = alloca i64, align 8
  %189 = alloca i64, align 8
  %190 = alloca ptr, align 8
  %191 = alloca ptr, align 8
  %192 = alloca ptr, align 8
  %193 = alloca ptr, align 8
  %194 = alloca i64, align 8
  %195 = alloca i64, align 8
  %196 = alloca ptr, align 8
  %197 = alloca ptr, align 8
  %198 = alloca ptr, align 8
  %199 = alloca i32, align 4
  %200 = alloca i64, align 8
  %201 = alloca ptr, align 8
  %202 = alloca ptr, align 8
  %203 = alloca ptr, align 8
  %204 = alloca i32, align 4
  %205 = alloca i64, align 8
  %206 = alloca ptr, align 8
  %207 = alloca ptr, align 8
  %208 = alloca ptr, align 8
  %209 = alloca ptr, align 8
  %210 = alloca i64, align 8
  %211 = alloca i64, align 8
  %212 = alloca ptr, align 8
  %213 = alloca double, align 8
  %214 = alloca double, align 8
  %215 = alloca ptr, align 8
  %216 = alloca ptr, align 8
  %217 = alloca ptr, align 8
  %218 = alloca ptr, align 8
  %219 = alloca i64, align 8
  %220 = alloca i64, align 8
  %221 = alloca ptr, align 8
  %222 = alloca double, align 8
  %223 = alloca double, align 8
  %224 = alloca ptr, align 8
  %225 = alloca ptr, align 8
  %226 = alloca ptr, align 8
  %227 = alloca ptr, align 8
  %228 = alloca i64, align 8
  %229 = alloca i64, align 8
  %230 = alloca ptr, align 8
  %231 = alloca double, align 8
  %232 = alloca double, align 8
  %233 = alloca ptr, align 8
  %234 = alloca ptr, align 8
  %235 = alloca ptr, align 8
  %236 = alloca ptr, align 8
  %237 = alloca i64, align 8
  %238 = alloca i64, align 8
  %239 = alloca ptr, align 8
  %240 = alloca double, align 8
  %241 = alloca double, align 8
  %242 = alloca ptr, align 8
  %243 = alloca ptr, align 8
  %244 = alloca ptr, align 8
  %245 = alloca ptr, align 8
  %246 = alloca double, align 8
  %247 = alloca double, align 8
  %248 = alloca ptr, align 8
  %249 = alloca ptr, align 8
  %250 = alloca ptr, align 8
  %251 = alloca ptr, align 8
  %252 = alloca double, align 8
  %253 = alloca double, align 8
  %254 = alloca ptr, align 8
  %255 = alloca ptr, align 8
  %256 = alloca ptr, align 8
  %257 = alloca ptr, align 8
  %258 = alloca i64, align 8
  %259 = alloca i64, align 8
  %260 = alloca ptr, align 8
  %261 = alloca double, align 8
  %262 = alloca double, align 8
  %263 = alloca ptr, align 8
  %264 = alloca ptr, align 8
  %265 = alloca ptr, align 8
  %266 = alloca ptr, align 8
  %267 = alloca i64, align 8
  %268 = alloca i64, align 8
  %269 = alloca ptr, align 8
  %270 = alloca ptr, align 8
  %271 = alloca ptr, align 8
  %272 = alloca ptr, align 8
  %273 = alloca i64, align 8
  %274 = alloca i64, align 8
  %275 = alloca ptr, align 8
  %276 = alloca ptr, align 8
  %277 = alloca ptr, align 8
  %278 = alloca ptr, align 8
  %279 = alloca i64, align 8
  %280 = alloca i64, align 8
  %281 = alloca ptr, align 8
  %282 = alloca ptr, align 8
  %283 = alloca ptr, align 8
  %284 = alloca ptr, align 8
  %285 = alloca i64, align 8
  %286 = alloca i64, align 8
  %287 = alloca ptr, align 8
  %288 = alloca ptr, align 8
  %289 = alloca ptr, align 8
  %290 = alloca ptr, align 8
  %291 = alloca i64, align 8
  %292 = alloca i64, align 8
  %293 = alloca ptr, align 8
  %294 = alloca ptr, align 8
  %295 = alloca i32, align 4
  %296 = alloca ptr, align 8
  %297 = alloca i32, align 4
  %298 = alloca ptr, align 8
  %299 = alloca ptr, align 8
  %300 = alloca i32, align 4
  %301 = alloca i32, align 4
  %302 = alloca i32, align 4
  %303 = alloca i32, align 4
  %304 = alloca ptr, align 8
  %305 = alloca ptr, align 8
  %306 = alloca i32, align 4
  %307 = alloca ptr, align 8
  %308 = alloca i32, align 4
  %309 = alloca i32, align 4
  %310 = alloca ptr, align 8
  %311 = alloca ptr, align 8
  %312 = alloca ptr, align 8
  %313 = alloca double, align 8
  %314 = alloca i64, align 8
  %315 = alloca ptr, align 8
  %316 = alloca ptr, align 8
  %317 = alloca ptr, align 8
  %318 = alloca ptr, align 8
  %319 = alloca i64, align 8
  %320 = alloca ptr, align 8
  %321 = alloca ptr, align 8
  %322 = alloca ptr, align 8
  %323 = alloca ptr, align 8
  %324 = alloca ptr, align 8
  %325 = alloca i32, align 4
  %326 = alloca ptr, align 8
  %327 = alloca ptr, align 8
  %328 = alloca ptr, align 8
  %329 = alloca i32, align 4
  %330 = alloca ptr, align 8
  %331 = alloca i32, align 4
  %332 = alloca ptr, align 8
  %333 = alloca i32, align 4
  %334 = alloca ptr, align 8
  %335 = alloca i64, align 8
  %336 = alloca i64, align 8
  %337 = alloca i32, align 4
  %338 = alloca ptr, align 8
  %339 = alloca i32, align 4
  %340 = alloca ptr, align 8
  %341 = alloca i64, align 8
  %342 = alloca i64, align 8
  %343 = alloca i32, align 4
  %344 = alloca ptr, align 8
  %345 = alloca ptr, align 8
  %346 = alloca i32, align 4
  %347 = alloca i32, align 4
  %348 = alloca ptr, align 8
  %349 = alloca i32, align 4
  %350 = alloca i32, align 4
  %351 = alloca i32, align 4
  %352 = alloca ptr, align 8
  %353 = alloca i32, align 4
  %354 = alloca i32, align 4
  %355 = alloca double, align 8
  %356 = alloca double, align 8
  %357 = alloca i32, align 4
  %358 = alloca i32, align 4
  %359 = alloca ptr, align 8
  %360 = alloca i32, align 4
  %361 = alloca i32, align 4
  %362 = alloca double, align 8
  %363 = alloca double, align 8
  %364 = alloca i32, align 4
  %365 = alloca i32, align 4
  %366 = alloca ptr, align 8
  %367 = alloca i32, align 4
  %368 = alloca i32, align 4
  %369 = alloca double, align 8
  %370 = alloca double, align 8
  %371 = alloca i32, align 4
  %372 = alloca i32, align 4
  %373 = alloca ptr, align 8
  %374 = alloca i32, align 4
  %375 = alloca i32, align 4
  %376 = alloca double, align 8
  %377 = alloca double, align 8
  %378 = alloca i32, align 4
  %379 = alloca i32, align 4
  %380 = alloca ptr, align 8
  %381 = alloca i32, align 4
  %382 = alloca i32, align 4
  %383 = alloca ptr, align 8
  %384 = alloca ptr, align 8
  %385 = alloca ptr, align 8
  %386 = alloca ptr, align 8
  %387 = alloca i32, align 4
  %388 = alloca ptr, align 8
  %389 = alloca ptr, align 8
  %390 = alloca i32, align 4
  %391 = alloca i32, align 4
  %392 = alloca ptr, align 8
  %393 = alloca i32, align 4
  %394 = alloca i32, align 4
  %395 = alloca i32, align 4
  %396 = alloca i32, align 4
  %397 = alloca ptr, align 8
  %398 = alloca i32, align 4
  %399 = alloca i32, align 4
  %400 = alloca ptr, align 8
  %401 = alloca i32, align 4
  %402 = alloca ptr, align 8
  %403 = alloca i32, align 4
  %404 = alloca ptr, align 8
  %405 = alloca ptr, align 8
  %406 = alloca ptr, align 8
  %407 = alloca ptr, align 8
  %408 = alloca i64, align 8
  %409 = alloca i64, align 8
  %410 = alloca i64, align 8
  %411 = alloca ptr, align 8
  %412 = alloca ptr, align 8
  %413 = alloca ptr, align 8
  %414 = alloca ptr, align 8
  %415 = alloca ptr, align 8
  %416 = alloca ptr, align 8
  %417 = alloca ptr, align 8
  %418 = alloca ptr, align 8
  %419 = alloca ptr, align 8
  %420 = alloca ptr, align 8
  %421 = alloca i32, align 4
  %422 = alloca i32, align 4
  %423 = alloca ptr, align 8
  %424 = alloca ptr, align 8
  %425 = alloca ptr, align 8
  %426 = alloca ptr, align 8
  %427 = alloca ptr, align 8
  %428 = alloca ptr, align 8
  %429 = alloca ptr, align 8
  %430 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  br label %431

431:                                              ; preds = %6848, %6759, %2
  %432 = load ptr, ptr %3, align 8
  %433 = getelementptr inbounds %struct.lua_State, ptr %432, i32 0, i32 23
  %434 = load volatile i32, ptr %433, align 8
  store i32 %434, ptr %9, align 4
  br label %435

435:                                              ; preds = %7122, %431
  %436 = load ptr, ptr %4, align 8
  %437 = getelementptr inbounds %struct.CallInfo, ptr %436, i32 0, i32 0
  %438 = load ptr, ptr %437, align 8
  %439 = getelementptr inbounds %struct.TValue, ptr %438, i32 0, i32 0
  %440 = load ptr, ptr %439, align 8
  store ptr %440, ptr %5, align 8
  %441 = load ptr, ptr %5, align 8
  %442 = getelementptr inbounds %struct.LClosure, ptr %441, i32 0, i32 5
  %443 = load ptr, ptr %442, align 8
  %444 = getelementptr inbounds %struct.Proto, ptr %443, i32 0, i32 15
  %445 = load ptr, ptr %444, align 8
  store ptr %445, ptr %6, align 8
  %446 = load ptr, ptr %4, align 8
  %447 = getelementptr inbounds %struct.CallInfo, ptr %446, i32 0, i32 4
  %448 = getelementptr inbounds %struct.anon, ptr %447, i32 0, i32 0
  %449 = load ptr, ptr %448, align 8
  store ptr %449, ptr %8, align 8
  %450 = load i32, ptr %9, align 4
  %451 = icmp ne i32 %450, 0
  %452 = zext i1 %451 to i32
  %453 = sext i32 %452 to i64
  %454 = icmp ne i64 %453, 0
  br i1 %454, label %455, label %458

455:                                              ; preds = %435
  %456 = load ptr, ptr %3, align 8
  %457 = call i32 @luaG_tracecall(ptr noundef %456)
  store i32 %457, ptr %9, align 4
  br label %458

458:                                              ; preds = %455, %435
  %459 = load ptr, ptr %4, align 8
  %460 = getelementptr inbounds %struct.CallInfo, ptr %459, i32 0, i32 0
  %461 = load ptr, ptr %460, align 8
  %462 = getelementptr inbounds %union.StackValue, ptr %461, i64 1
  store ptr %462, ptr %7, align 8
  br label %463

463:                                              ; preds = %458
  %464 = load i32, ptr %9, align 4
  %465 = icmp ne i32 %464, 0
  %466 = zext i1 %465 to i32
  %467 = sext i32 %466 to i64
  %468 = icmp ne i64 %467, 0
  br i1 %468, label %469, label %477

469:                                              ; preds = %463
  %470 = load ptr, ptr %3, align 8
  %471 = load ptr, ptr %8, align 8
  %472 = call i32 @luaG_traceexec(ptr noundef %470, ptr noundef %471)
  store i32 %472, ptr %9, align 4
  %473 = load ptr, ptr %4, align 8
  %474 = getelementptr inbounds %struct.CallInfo, ptr %473, i32 0, i32 0
  %475 = load ptr, ptr %474, align 8
  %476 = getelementptr inbounds %union.StackValue, ptr %475, i64 1
  store ptr %476, ptr %7, align 8
  br label %477

477:                                              ; preds = %469, %463
  %478 = load ptr, ptr %8, align 8
  %479 = getelementptr inbounds i32, ptr %478, i32 1
  store ptr %479, ptr %8, align 8
  %480 = load i32, ptr %478, align 4
  store i32 %480, ptr %10, align 4
  %481 = load i32, ptr %10, align 4
  %482 = lshr i32 %481, 0
  %483 = and i32 %482, 127
  %484 = zext i32 %483 to i64
  %485 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %484
  %486 = load ptr, ptr %485, align 8
  br label %7120

487:                                              ; preds = %7120
  %488 = load ptr, ptr %7, align 8
  %489 = load i32, ptr %10, align 4
  %490 = lshr i32 %489, 7
  %491 = and i32 %490, 255
  %492 = sext i32 %491 to i64
  %493 = getelementptr inbounds %union.StackValue, ptr %488, i64 %492
  store ptr %493, ptr %11, align 8
  %494 = load ptr, ptr %11, align 8
  store ptr %494, ptr %12, align 8
  %495 = load ptr, ptr %7, align 8
  %496 = load i32, ptr %10, align 4
  %497 = lshr i32 %496, 16
  %498 = and i32 %497, 255
  %499 = sext i32 %498 to i64
  %500 = getelementptr inbounds %union.StackValue, ptr %495, i64 %499
  store ptr %500, ptr %13, align 8
  %501 = load ptr, ptr %12, align 8
  %502 = getelementptr inbounds %struct.TValue, ptr %501, i32 0, i32 0
  %503 = load ptr, ptr %13, align 8
  %504 = getelementptr inbounds %struct.TValue, ptr %503, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %502, ptr align 8 %504, i64 8, i1 false)
  %505 = load ptr, ptr %13, align 8
  %506 = getelementptr inbounds %struct.TValue, ptr %505, i32 0, i32 1
  %507 = load i8, ptr %506, align 8
  %508 = load ptr, ptr %12, align 8
  %509 = getelementptr inbounds %struct.TValue, ptr %508, i32 0, i32 1
  store i8 %507, ptr %509, align 8
  %510 = load ptr, ptr %3, align 8
  %511 = load i32, ptr %9, align 4
  %512 = icmp ne i32 %511, 0
  %513 = zext i1 %512 to i32
  %514 = sext i32 %513 to i64
  %515 = icmp ne i64 %514, 0
  br i1 %515, label %516, label %524

516:                                              ; preds = %487
  %517 = load ptr, ptr %3, align 8
  %518 = load ptr, ptr %8, align 8
  %519 = call i32 @luaG_traceexec(ptr noundef %517, ptr noundef %518)
  store i32 %519, ptr %9, align 4
  %520 = load ptr, ptr %4, align 8
  %521 = getelementptr inbounds %struct.CallInfo, ptr %520, i32 0, i32 0
  %522 = load ptr, ptr %521, align 8
  %523 = getelementptr inbounds %union.StackValue, ptr %522, i64 1
  store ptr %523, ptr %7, align 8
  br label %524

524:                                              ; preds = %516, %487
  %525 = load ptr, ptr %8, align 8
  %526 = getelementptr inbounds i32, ptr %525, i32 1
  store ptr %526, ptr %8, align 8
  %527 = load i32, ptr %525, align 4
  store i32 %527, ptr %10, align 4
  %528 = load i32, ptr %10, align 4
  %529 = lshr i32 %528, 0
  %530 = and i32 %529, 127
  %531 = zext i32 %530 to i64
  %532 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %531
  %533 = load ptr, ptr %532, align 8
  br label %7120

534:                                              ; preds = %7120
  %535 = load ptr, ptr %7, align 8
  %536 = load i32, ptr %10, align 4
  %537 = lshr i32 %536, 7
  %538 = and i32 %537, 255
  %539 = sext i32 %538 to i64
  %540 = getelementptr inbounds %union.StackValue, ptr %535, i64 %539
  store ptr %540, ptr %14, align 8
  %541 = load i32, ptr %10, align 4
  %542 = lshr i32 %541, 15
  %543 = and i32 %542, 131071
  %544 = sub nsw i32 %543, 65535
  %545 = sext i32 %544 to i64
  store i64 %545, ptr %15, align 8
  %546 = load ptr, ptr %14, align 8
  store ptr %546, ptr %16, align 8
  %547 = load i64, ptr %15, align 8
  %548 = load ptr, ptr %16, align 8
  %549 = getelementptr inbounds %struct.TValue, ptr %548, i32 0, i32 0
  store i64 %547, ptr %549, align 8
  %550 = load ptr, ptr %16, align 8
  %551 = getelementptr inbounds %struct.TValue, ptr %550, i32 0, i32 1
  store i8 3, ptr %551, align 8
  %552 = load i32, ptr %9, align 4
  %553 = icmp ne i32 %552, 0
  %554 = zext i1 %553 to i32
  %555 = sext i32 %554 to i64
  %556 = icmp ne i64 %555, 0
  br i1 %556, label %557, label %565

557:                                              ; preds = %534
  %558 = load ptr, ptr %3, align 8
  %559 = load ptr, ptr %8, align 8
  %560 = call i32 @luaG_traceexec(ptr noundef %558, ptr noundef %559)
  store i32 %560, ptr %9, align 4
  %561 = load ptr, ptr %4, align 8
  %562 = getelementptr inbounds %struct.CallInfo, ptr %561, i32 0, i32 0
  %563 = load ptr, ptr %562, align 8
  %564 = getelementptr inbounds %union.StackValue, ptr %563, i64 1
  store ptr %564, ptr %7, align 8
  br label %565

565:                                              ; preds = %557, %534
  %566 = load ptr, ptr %8, align 8
  %567 = getelementptr inbounds i32, ptr %566, i32 1
  store ptr %567, ptr %8, align 8
  %568 = load i32, ptr %566, align 4
  store i32 %568, ptr %10, align 4
  %569 = load i32, ptr %10, align 4
  %570 = lshr i32 %569, 0
  %571 = and i32 %570, 127
  %572 = zext i32 %571 to i64
  %573 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %572
  %574 = load ptr, ptr %573, align 8
  br label %7120

575:                                              ; preds = %7120
  %576 = load ptr, ptr %7, align 8
  %577 = load i32, ptr %10, align 4
  %578 = lshr i32 %577, 7
  %579 = and i32 %578, 255
  %580 = sext i32 %579 to i64
  %581 = getelementptr inbounds %union.StackValue, ptr %576, i64 %580
  store ptr %581, ptr %17, align 8
  %582 = load i32, ptr %10, align 4
  %583 = lshr i32 %582, 15
  %584 = and i32 %583, 131071
  %585 = sub nsw i32 %584, 65535
  store i32 %585, ptr %18, align 4
  %586 = load ptr, ptr %17, align 8
  store ptr %586, ptr %19, align 8
  %587 = load i32, ptr %18, align 4
  %588 = sitofp i32 %587 to double
  %589 = load ptr, ptr %19, align 8
  %590 = getelementptr inbounds %struct.TValue, ptr %589, i32 0, i32 0
  store double %588, ptr %590, align 8
  %591 = load ptr, ptr %19, align 8
  %592 = getelementptr inbounds %struct.TValue, ptr %591, i32 0, i32 1
  store i8 19, ptr %592, align 8
  %593 = load i32, ptr %9, align 4
  %594 = icmp ne i32 %593, 0
  %595 = zext i1 %594 to i32
  %596 = sext i32 %595 to i64
  %597 = icmp ne i64 %596, 0
  br i1 %597, label %598, label %606

598:                                              ; preds = %575
  %599 = load ptr, ptr %3, align 8
  %600 = load ptr, ptr %8, align 8
  %601 = call i32 @luaG_traceexec(ptr noundef %599, ptr noundef %600)
  store i32 %601, ptr %9, align 4
  %602 = load ptr, ptr %4, align 8
  %603 = getelementptr inbounds %struct.CallInfo, ptr %602, i32 0, i32 0
  %604 = load ptr, ptr %603, align 8
  %605 = getelementptr inbounds %union.StackValue, ptr %604, i64 1
  store ptr %605, ptr %7, align 8
  br label %606

606:                                              ; preds = %598, %575
  %607 = load ptr, ptr %8, align 8
  %608 = getelementptr inbounds i32, ptr %607, i32 1
  store ptr %608, ptr %8, align 8
  %609 = load i32, ptr %607, align 4
  store i32 %609, ptr %10, align 4
  %610 = load i32, ptr %10, align 4
  %611 = lshr i32 %610, 0
  %612 = and i32 %611, 127
  %613 = zext i32 %612 to i64
  %614 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %613
  %615 = load ptr, ptr %614, align 8
  br label %7120

616:                                              ; preds = %7120
  %617 = load ptr, ptr %7, align 8
  %618 = load i32, ptr %10, align 4
  %619 = lshr i32 %618, 7
  %620 = and i32 %619, 255
  %621 = sext i32 %620 to i64
  %622 = getelementptr inbounds %union.StackValue, ptr %617, i64 %621
  store ptr %622, ptr %20, align 8
  %623 = load ptr, ptr %6, align 8
  %624 = load i32, ptr %10, align 4
  %625 = lshr i32 %624, 15
  %626 = and i32 %625, 131071
  %627 = sext i32 %626 to i64
  %628 = getelementptr inbounds %struct.TValue, ptr %623, i64 %627
  store ptr %628, ptr %21, align 8
  %629 = load ptr, ptr %20, align 8
  store ptr %629, ptr %22, align 8
  %630 = load ptr, ptr %21, align 8
  store ptr %630, ptr %23, align 8
  %631 = load ptr, ptr %22, align 8
  %632 = getelementptr inbounds %struct.TValue, ptr %631, i32 0, i32 0
  %633 = load ptr, ptr %23, align 8
  %634 = getelementptr inbounds %struct.TValue, ptr %633, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %632, ptr align 8 %634, i64 8, i1 false)
  %635 = load ptr, ptr %23, align 8
  %636 = getelementptr inbounds %struct.TValue, ptr %635, i32 0, i32 1
  %637 = load i8, ptr %636, align 8
  %638 = load ptr, ptr %22, align 8
  %639 = getelementptr inbounds %struct.TValue, ptr %638, i32 0, i32 1
  store i8 %637, ptr %639, align 8
  %640 = load ptr, ptr %3, align 8
  %641 = load i32, ptr %9, align 4
  %642 = icmp ne i32 %641, 0
  %643 = zext i1 %642 to i32
  %644 = sext i32 %643 to i64
  %645 = icmp ne i64 %644, 0
  br i1 %645, label %646, label %654

646:                                              ; preds = %616
  %647 = load ptr, ptr %3, align 8
  %648 = load ptr, ptr %8, align 8
  %649 = call i32 @luaG_traceexec(ptr noundef %647, ptr noundef %648)
  store i32 %649, ptr %9, align 4
  %650 = load ptr, ptr %4, align 8
  %651 = getelementptr inbounds %struct.CallInfo, ptr %650, i32 0, i32 0
  %652 = load ptr, ptr %651, align 8
  %653 = getelementptr inbounds %union.StackValue, ptr %652, i64 1
  store ptr %653, ptr %7, align 8
  br label %654

654:                                              ; preds = %646, %616
  %655 = load ptr, ptr %8, align 8
  %656 = getelementptr inbounds i32, ptr %655, i32 1
  store ptr %656, ptr %8, align 8
  %657 = load i32, ptr %655, align 4
  store i32 %657, ptr %10, align 4
  %658 = load i32, ptr %10, align 4
  %659 = lshr i32 %658, 0
  %660 = and i32 %659, 127
  %661 = zext i32 %660 to i64
  %662 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %661
  %663 = load ptr, ptr %662, align 8
  br label %7120

664:                                              ; preds = %7120
  %665 = load ptr, ptr %7, align 8
  %666 = load i32, ptr %10, align 4
  %667 = lshr i32 %666, 7
  %668 = and i32 %667, 255
  %669 = sext i32 %668 to i64
  %670 = getelementptr inbounds %union.StackValue, ptr %665, i64 %669
  store ptr %670, ptr %24, align 8
  %671 = load ptr, ptr %6, align 8
  %672 = load ptr, ptr %8, align 8
  %673 = load i32, ptr %672, align 4
  %674 = lshr i32 %673, 7
  %675 = and i32 %674, 33554431
  %676 = sext i32 %675 to i64
  %677 = getelementptr inbounds %struct.TValue, ptr %671, i64 %676
  store ptr %677, ptr %25, align 8
  %678 = load ptr, ptr %8, align 8
  %679 = getelementptr inbounds i32, ptr %678, i32 1
  store ptr %679, ptr %8, align 8
  %680 = load ptr, ptr %24, align 8
  store ptr %680, ptr %26, align 8
  %681 = load ptr, ptr %25, align 8
  store ptr %681, ptr %27, align 8
  %682 = load ptr, ptr %26, align 8
  %683 = getelementptr inbounds %struct.TValue, ptr %682, i32 0, i32 0
  %684 = load ptr, ptr %27, align 8
  %685 = getelementptr inbounds %struct.TValue, ptr %684, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %683, ptr align 8 %685, i64 8, i1 false)
  %686 = load ptr, ptr %27, align 8
  %687 = getelementptr inbounds %struct.TValue, ptr %686, i32 0, i32 1
  %688 = load i8, ptr %687, align 8
  %689 = load ptr, ptr %26, align 8
  %690 = getelementptr inbounds %struct.TValue, ptr %689, i32 0, i32 1
  store i8 %688, ptr %690, align 8
  %691 = load ptr, ptr %3, align 8
  %692 = load i32, ptr %9, align 4
  %693 = icmp ne i32 %692, 0
  %694 = zext i1 %693 to i32
  %695 = sext i32 %694 to i64
  %696 = icmp ne i64 %695, 0
  br i1 %696, label %697, label %705

697:                                              ; preds = %664
  %698 = load ptr, ptr %3, align 8
  %699 = load ptr, ptr %8, align 8
  %700 = call i32 @luaG_traceexec(ptr noundef %698, ptr noundef %699)
  store i32 %700, ptr %9, align 4
  %701 = load ptr, ptr %4, align 8
  %702 = getelementptr inbounds %struct.CallInfo, ptr %701, i32 0, i32 0
  %703 = load ptr, ptr %702, align 8
  %704 = getelementptr inbounds %union.StackValue, ptr %703, i64 1
  store ptr %704, ptr %7, align 8
  br label %705

705:                                              ; preds = %697, %664
  %706 = load ptr, ptr %8, align 8
  %707 = getelementptr inbounds i32, ptr %706, i32 1
  store ptr %707, ptr %8, align 8
  %708 = load i32, ptr %706, align 4
  store i32 %708, ptr %10, align 4
  %709 = load i32, ptr %10, align 4
  %710 = lshr i32 %709, 0
  %711 = and i32 %710, 127
  %712 = zext i32 %711 to i64
  %713 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %712
  %714 = load ptr, ptr %713, align 8
  br label %7120

715:                                              ; preds = %7120
  %716 = load ptr, ptr %7, align 8
  %717 = load i32, ptr %10, align 4
  %718 = lshr i32 %717, 7
  %719 = and i32 %718, 255
  %720 = sext i32 %719 to i64
  %721 = getelementptr inbounds %union.StackValue, ptr %716, i64 %720
  store ptr %721, ptr %28, align 8
  %722 = load ptr, ptr %28, align 8
  %723 = getelementptr inbounds %struct.TValue, ptr %722, i32 0, i32 1
  store i8 1, ptr %723, align 8
  %724 = load i32, ptr %9, align 4
  %725 = icmp ne i32 %724, 0
  %726 = zext i1 %725 to i32
  %727 = sext i32 %726 to i64
  %728 = icmp ne i64 %727, 0
  br i1 %728, label %729, label %737

729:                                              ; preds = %715
  %730 = load ptr, ptr %3, align 8
  %731 = load ptr, ptr %8, align 8
  %732 = call i32 @luaG_traceexec(ptr noundef %730, ptr noundef %731)
  store i32 %732, ptr %9, align 4
  %733 = load ptr, ptr %4, align 8
  %734 = getelementptr inbounds %struct.CallInfo, ptr %733, i32 0, i32 0
  %735 = load ptr, ptr %734, align 8
  %736 = getelementptr inbounds %union.StackValue, ptr %735, i64 1
  store ptr %736, ptr %7, align 8
  br label %737

737:                                              ; preds = %729, %715
  %738 = load ptr, ptr %8, align 8
  %739 = getelementptr inbounds i32, ptr %738, i32 1
  store ptr %739, ptr %8, align 8
  %740 = load i32, ptr %738, align 4
  store i32 %740, ptr %10, align 4
  %741 = load i32, ptr %10, align 4
  %742 = lshr i32 %741, 0
  %743 = and i32 %742, 127
  %744 = zext i32 %743 to i64
  %745 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %744
  %746 = load ptr, ptr %745, align 8
  br label %7120

747:                                              ; preds = %7120
  %748 = load ptr, ptr %7, align 8
  %749 = load i32, ptr %10, align 4
  %750 = lshr i32 %749, 7
  %751 = and i32 %750, 255
  %752 = sext i32 %751 to i64
  %753 = getelementptr inbounds %union.StackValue, ptr %748, i64 %752
  store ptr %753, ptr %29, align 8
  %754 = load ptr, ptr %29, align 8
  %755 = getelementptr inbounds %struct.TValue, ptr %754, i32 0, i32 1
  store i8 1, ptr %755, align 8
  %756 = load ptr, ptr %8, align 8
  %757 = getelementptr inbounds i32, ptr %756, i32 1
  store ptr %757, ptr %8, align 8
  %758 = load i32, ptr %9, align 4
  %759 = icmp ne i32 %758, 0
  %760 = zext i1 %759 to i32
  %761 = sext i32 %760 to i64
  %762 = icmp ne i64 %761, 0
  br i1 %762, label %763, label %771

763:                                              ; preds = %747
  %764 = load ptr, ptr %3, align 8
  %765 = load ptr, ptr %8, align 8
  %766 = call i32 @luaG_traceexec(ptr noundef %764, ptr noundef %765)
  store i32 %766, ptr %9, align 4
  %767 = load ptr, ptr %4, align 8
  %768 = getelementptr inbounds %struct.CallInfo, ptr %767, i32 0, i32 0
  %769 = load ptr, ptr %768, align 8
  %770 = getelementptr inbounds %union.StackValue, ptr %769, i64 1
  store ptr %770, ptr %7, align 8
  br label %771

771:                                              ; preds = %763, %747
  %772 = load ptr, ptr %8, align 8
  %773 = getelementptr inbounds i32, ptr %772, i32 1
  store ptr %773, ptr %8, align 8
  %774 = load i32, ptr %772, align 4
  store i32 %774, ptr %10, align 4
  %775 = load i32, ptr %10, align 4
  %776 = lshr i32 %775, 0
  %777 = and i32 %776, 127
  %778 = zext i32 %777 to i64
  %779 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %778
  %780 = load ptr, ptr %779, align 8
  br label %7120

781:                                              ; preds = %7120
  %782 = load ptr, ptr %7, align 8
  %783 = load i32, ptr %10, align 4
  %784 = lshr i32 %783, 7
  %785 = and i32 %784, 255
  %786 = sext i32 %785 to i64
  %787 = getelementptr inbounds %union.StackValue, ptr %782, i64 %786
  store ptr %787, ptr %30, align 8
  %788 = load ptr, ptr %30, align 8
  %789 = getelementptr inbounds %struct.TValue, ptr %788, i32 0, i32 1
  store i8 17, ptr %789, align 8
  %790 = load i32, ptr %9, align 4
  %791 = icmp ne i32 %790, 0
  %792 = zext i1 %791 to i32
  %793 = sext i32 %792 to i64
  %794 = icmp ne i64 %793, 0
  br i1 %794, label %795, label %803

795:                                              ; preds = %781
  %796 = load ptr, ptr %3, align 8
  %797 = load ptr, ptr %8, align 8
  %798 = call i32 @luaG_traceexec(ptr noundef %796, ptr noundef %797)
  store i32 %798, ptr %9, align 4
  %799 = load ptr, ptr %4, align 8
  %800 = getelementptr inbounds %struct.CallInfo, ptr %799, i32 0, i32 0
  %801 = load ptr, ptr %800, align 8
  %802 = getelementptr inbounds %union.StackValue, ptr %801, i64 1
  store ptr %802, ptr %7, align 8
  br label %803

803:                                              ; preds = %795, %781
  %804 = load ptr, ptr %8, align 8
  %805 = getelementptr inbounds i32, ptr %804, i32 1
  store ptr %805, ptr %8, align 8
  %806 = load i32, ptr %804, align 4
  store i32 %806, ptr %10, align 4
  %807 = load i32, ptr %10, align 4
  %808 = lshr i32 %807, 0
  %809 = and i32 %808, 127
  %810 = zext i32 %809 to i64
  %811 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %810
  %812 = load ptr, ptr %811, align 8
  br label %7120

813:                                              ; preds = %7120
  %814 = load ptr, ptr %7, align 8
  %815 = load i32, ptr %10, align 4
  %816 = lshr i32 %815, 7
  %817 = and i32 %816, 255
  %818 = sext i32 %817 to i64
  %819 = getelementptr inbounds %union.StackValue, ptr %814, i64 %818
  store ptr %819, ptr %31, align 8
  %820 = load i32, ptr %10, align 4
  %821 = lshr i32 %820, 16
  %822 = and i32 %821, 255
  store i32 %822, ptr %32, align 4
  br label %823

823:                                              ; preds = %827, %813
  %824 = load ptr, ptr %31, align 8
  %825 = getelementptr inbounds %union.StackValue, ptr %824, i32 1
  store ptr %825, ptr %31, align 8
  %826 = getelementptr inbounds %struct.TValue, ptr %824, i32 0, i32 1
  store i8 0, ptr %826, align 8
  br label %827

827:                                              ; preds = %823
  %828 = load i32, ptr %32, align 4
  %829 = add nsw i32 %828, -1
  store i32 %829, ptr %32, align 4
  %830 = icmp ne i32 %828, 0
  br i1 %830, label %823, label %831, !llvm.loop !12

831:                                              ; preds = %827
  %832 = load i32, ptr %9, align 4
  %833 = icmp ne i32 %832, 0
  %834 = zext i1 %833 to i32
  %835 = sext i32 %834 to i64
  %836 = icmp ne i64 %835, 0
  br i1 %836, label %837, label %845

837:                                              ; preds = %831
  %838 = load ptr, ptr %3, align 8
  %839 = load ptr, ptr %8, align 8
  %840 = call i32 @luaG_traceexec(ptr noundef %838, ptr noundef %839)
  store i32 %840, ptr %9, align 4
  %841 = load ptr, ptr %4, align 8
  %842 = getelementptr inbounds %struct.CallInfo, ptr %841, i32 0, i32 0
  %843 = load ptr, ptr %842, align 8
  %844 = getelementptr inbounds %union.StackValue, ptr %843, i64 1
  store ptr %844, ptr %7, align 8
  br label %845

845:                                              ; preds = %837, %831
  %846 = load ptr, ptr %8, align 8
  %847 = getelementptr inbounds i32, ptr %846, i32 1
  store ptr %847, ptr %8, align 8
  %848 = load i32, ptr %846, align 4
  store i32 %848, ptr %10, align 4
  %849 = load i32, ptr %10, align 4
  %850 = lshr i32 %849, 0
  %851 = and i32 %850, 127
  %852 = zext i32 %851 to i64
  %853 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %852
  %854 = load ptr, ptr %853, align 8
  br label %7120

855:                                              ; preds = %7120
  %856 = load ptr, ptr %7, align 8
  %857 = load i32, ptr %10, align 4
  %858 = lshr i32 %857, 7
  %859 = and i32 %858, 255
  %860 = sext i32 %859 to i64
  %861 = getelementptr inbounds %union.StackValue, ptr %856, i64 %860
  store ptr %861, ptr %33, align 8
  %862 = load i32, ptr %10, align 4
  %863 = lshr i32 %862, 16
  %864 = and i32 %863, 255
  store i32 %864, ptr %34, align 4
  %865 = load ptr, ptr %33, align 8
  store ptr %865, ptr %35, align 8
  %866 = load ptr, ptr %5, align 8
  %867 = getelementptr inbounds %struct.LClosure, ptr %866, i32 0, i32 6
  %868 = load i32, ptr %34, align 4
  %869 = sext i32 %868 to i64
  %870 = getelementptr inbounds [1 x ptr], ptr %867, i64 0, i64 %869
  %871 = load ptr, ptr %870, align 8
  %872 = getelementptr inbounds %struct.UpVal, ptr %871, i32 0, i32 3
  %873 = load ptr, ptr %872, align 8
  store ptr %873, ptr %36, align 8
  %874 = load ptr, ptr %35, align 8
  %875 = getelementptr inbounds %struct.TValue, ptr %874, i32 0, i32 0
  %876 = load ptr, ptr %36, align 8
  %877 = getelementptr inbounds %struct.TValue, ptr %876, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %875, ptr align 8 %877, i64 8, i1 false)
  %878 = load ptr, ptr %36, align 8
  %879 = getelementptr inbounds %struct.TValue, ptr %878, i32 0, i32 1
  %880 = load i8, ptr %879, align 8
  %881 = load ptr, ptr %35, align 8
  %882 = getelementptr inbounds %struct.TValue, ptr %881, i32 0, i32 1
  store i8 %880, ptr %882, align 8
  %883 = load ptr, ptr %3, align 8
  %884 = load i32, ptr %9, align 4
  %885 = icmp ne i32 %884, 0
  %886 = zext i1 %885 to i32
  %887 = sext i32 %886 to i64
  %888 = icmp ne i64 %887, 0
  br i1 %888, label %889, label %897

889:                                              ; preds = %855
  %890 = load ptr, ptr %3, align 8
  %891 = load ptr, ptr %8, align 8
  %892 = call i32 @luaG_traceexec(ptr noundef %890, ptr noundef %891)
  store i32 %892, ptr %9, align 4
  %893 = load ptr, ptr %4, align 8
  %894 = getelementptr inbounds %struct.CallInfo, ptr %893, i32 0, i32 0
  %895 = load ptr, ptr %894, align 8
  %896 = getelementptr inbounds %union.StackValue, ptr %895, i64 1
  store ptr %896, ptr %7, align 8
  br label %897

897:                                              ; preds = %889, %855
  %898 = load ptr, ptr %8, align 8
  %899 = getelementptr inbounds i32, ptr %898, i32 1
  store ptr %899, ptr %8, align 8
  %900 = load i32, ptr %898, align 4
  store i32 %900, ptr %10, align 4
  %901 = load i32, ptr %10, align 4
  %902 = lshr i32 %901, 0
  %903 = and i32 %902, 127
  %904 = zext i32 %903 to i64
  %905 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %904
  %906 = load ptr, ptr %905, align 8
  br label %7120

907:                                              ; preds = %7120
  %908 = load ptr, ptr %7, align 8
  %909 = load i32, ptr %10, align 4
  %910 = lshr i32 %909, 7
  %911 = and i32 %910, 255
  %912 = sext i32 %911 to i64
  %913 = getelementptr inbounds %union.StackValue, ptr %908, i64 %912
  store ptr %913, ptr %37, align 8
  %914 = load ptr, ptr %5, align 8
  %915 = getelementptr inbounds %struct.LClosure, ptr %914, i32 0, i32 6
  %916 = load i32, ptr %10, align 4
  %917 = lshr i32 %916, 16
  %918 = and i32 %917, 255
  %919 = sext i32 %918 to i64
  %920 = getelementptr inbounds [1 x ptr], ptr %915, i64 0, i64 %919
  %921 = load ptr, ptr %920, align 8
  store ptr %921, ptr %38, align 8
  %922 = load ptr, ptr %38, align 8
  %923 = getelementptr inbounds %struct.UpVal, ptr %922, i32 0, i32 3
  %924 = load ptr, ptr %923, align 8
  store ptr %924, ptr %39, align 8
  %925 = load ptr, ptr %37, align 8
  store ptr %925, ptr %40, align 8
  %926 = load ptr, ptr %39, align 8
  %927 = getelementptr inbounds %struct.TValue, ptr %926, i32 0, i32 0
  %928 = load ptr, ptr %40, align 8
  %929 = getelementptr inbounds %struct.TValue, ptr %928, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %927, ptr align 8 %929, i64 8, i1 false)
  %930 = load ptr, ptr %40, align 8
  %931 = getelementptr inbounds %struct.TValue, ptr %930, i32 0, i32 1
  %932 = load i8, ptr %931, align 8
  %933 = load ptr, ptr %39, align 8
  %934 = getelementptr inbounds %struct.TValue, ptr %933, i32 0, i32 1
  store i8 %932, ptr %934, align 8
  %935 = load ptr, ptr %3, align 8
  %936 = load ptr, ptr %37, align 8
  %937 = getelementptr inbounds %struct.TValue, ptr %936, i32 0, i32 1
  %938 = load i8, ptr %937, align 8
  %939 = zext i8 %938 to i32
  %940 = and i32 %939, 64
  %941 = icmp ne i32 %940, 0
  br i1 %941, label %942, label %966

942:                                              ; preds = %907
  %943 = load ptr, ptr %38, align 8
  %944 = getelementptr inbounds %struct.UpVal, ptr %943, i32 0, i32 2
  %945 = load i8, ptr %944, align 1
  %946 = zext i8 %945 to i32
  %947 = and i32 %946, 32
  %948 = icmp ne i32 %947, 0
  br i1 %948, label %949, label %964

949:                                              ; preds = %942
  %950 = load ptr, ptr %37, align 8
  %951 = getelementptr inbounds %struct.TValue, ptr %950, i32 0, i32 0
  %952 = load ptr, ptr %951, align 8
  %953 = getelementptr inbounds %struct.GCObject, ptr %952, i32 0, i32 2
  %954 = load i8, ptr %953, align 1
  %955 = zext i8 %954 to i32
  %956 = and i32 %955, 24
  %957 = icmp ne i32 %956, 0
  br i1 %957, label %958, label %964

958:                                              ; preds = %949
  %959 = load ptr, ptr %3, align 8
  %960 = load ptr, ptr %38, align 8
  %961 = load ptr, ptr %37, align 8
  %962 = getelementptr inbounds %struct.TValue, ptr %961, i32 0, i32 0
  %963 = load ptr, ptr %962, align 8
  call void @luaC_barrier_(ptr noundef %959, ptr noundef %960, ptr noundef %963)
  br label %965

964:                                              ; preds = %949, %942
  br label %965

965:                                              ; preds = %964, %958
  br label %967

966:                                              ; preds = %907
  br label %967

967:                                              ; preds = %966, %965
  %968 = load i32, ptr %9, align 4
  %969 = icmp ne i32 %968, 0
  %970 = zext i1 %969 to i32
  %971 = sext i32 %970 to i64
  %972 = icmp ne i64 %971, 0
  br i1 %972, label %973, label %981

973:                                              ; preds = %967
  %974 = load ptr, ptr %3, align 8
  %975 = load ptr, ptr %8, align 8
  %976 = call i32 @luaG_traceexec(ptr noundef %974, ptr noundef %975)
  store i32 %976, ptr %9, align 4
  %977 = load ptr, ptr %4, align 8
  %978 = getelementptr inbounds %struct.CallInfo, ptr %977, i32 0, i32 0
  %979 = load ptr, ptr %978, align 8
  %980 = getelementptr inbounds %union.StackValue, ptr %979, i64 1
  store ptr %980, ptr %7, align 8
  br label %981

981:                                              ; preds = %973, %967
  %982 = load ptr, ptr %8, align 8
  %983 = getelementptr inbounds i32, ptr %982, i32 1
  store ptr %983, ptr %8, align 8
  %984 = load i32, ptr %982, align 4
  store i32 %984, ptr %10, align 4
  %985 = load i32, ptr %10, align 4
  %986 = lshr i32 %985, 0
  %987 = and i32 %986, 127
  %988 = zext i32 %987 to i64
  %989 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %988
  %990 = load ptr, ptr %989, align 8
  br label %7120

991:                                              ; preds = %7120
  %992 = load ptr, ptr %7, align 8
  %993 = load i32, ptr %10, align 4
  %994 = lshr i32 %993, 7
  %995 = and i32 %994, 255
  %996 = sext i32 %995 to i64
  %997 = getelementptr inbounds %union.StackValue, ptr %992, i64 %996
  store ptr %997, ptr %41, align 8
  %998 = load ptr, ptr %5, align 8
  %999 = getelementptr inbounds %struct.LClosure, ptr %998, i32 0, i32 6
  %1000 = load i32, ptr %10, align 4
  %1001 = lshr i32 %1000, 16
  %1002 = and i32 %1001, 255
  %1003 = sext i32 %1002 to i64
  %1004 = getelementptr inbounds [1 x ptr], ptr %999, i64 0, i64 %1003
  %1005 = load ptr, ptr %1004, align 8
  %1006 = getelementptr inbounds %struct.UpVal, ptr %1005, i32 0, i32 3
  %1007 = load ptr, ptr %1006, align 8
  store ptr %1007, ptr %43, align 8
  %1008 = load ptr, ptr %6, align 8
  %1009 = load i32, ptr %10, align 4
  %1010 = lshr i32 %1009, 24
  %1011 = and i32 %1010, 255
  %1012 = sext i32 %1011 to i64
  %1013 = getelementptr inbounds %struct.TValue, ptr %1008, i64 %1012
  store ptr %1013, ptr %44, align 8
  %1014 = load ptr, ptr %44, align 8
  %1015 = getelementptr inbounds %struct.TValue, ptr %1014, i32 0, i32 0
  %1016 = load ptr, ptr %1015, align 8
  store ptr %1016, ptr %45, align 8
  %1017 = load ptr, ptr %43, align 8
  %1018 = getelementptr inbounds %struct.TValue, ptr %1017, i32 0, i32 1
  %1019 = load i8, ptr %1018, align 8
  %1020 = zext i8 %1019 to i32
  %1021 = icmp eq i32 %1020, 69
  br i1 %1021, label %1023, label %1022

1022:                                             ; preds = %991
  store ptr null, ptr %42, align 8
  br i1 false, label %1036, label %1049

1023:                                             ; preds = %991
  %1024 = load ptr, ptr %43, align 8
  %1025 = getelementptr inbounds %struct.TValue, ptr %1024, i32 0, i32 0
  %1026 = load ptr, ptr %1025, align 8
  %1027 = load ptr, ptr %45, align 8
  %1028 = call ptr @luaH_getshortstr(ptr noundef %1026, ptr noundef %1027)
  store ptr %1028, ptr %42, align 8
  %1029 = load ptr, ptr %42, align 8
  %1030 = getelementptr inbounds %struct.TValue, ptr %1029, i32 0, i32 1
  %1031 = load i8, ptr %1030, align 8
  %1032 = zext i8 %1031 to i32
  %1033 = and i32 %1032, 15
  %1034 = icmp eq i32 %1033, 0
  %1035 = xor i1 %1034, true
  br i1 %1035, label %1036, label %1049

1036:                                             ; preds = %1023, %1022
  %1037 = load ptr, ptr %41, align 8
  store ptr %1037, ptr %46, align 8
  %1038 = load ptr, ptr %42, align 8
  store ptr %1038, ptr %47, align 8
  %1039 = load ptr, ptr %46, align 8
  %1040 = getelementptr inbounds %struct.TValue, ptr %1039, i32 0, i32 0
  %1041 = load ptr, ptr %47, align 8
  %1042 = getelementptr inbounds %struct.TValue, ptr %1041, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1040, ptr align 8 %1042, i64 8, i1 false)
  %1043 = load ptr, ptr %47, align 8
  %1044 = getelementptr inbounds %struct.TValue, ptr %1043, i32 0, i32 1
  %1045 = load i8, ptr %1044, align 8
  %1046 = load ptr, ptr %46, align 8
  %1047 = getelementptr inbounds %struct.TValue, ptr %1046, i32 0, i32 1
  store i8 %1045, ptr %1047, align 8
  %1048 = load ptr, ptr %3, align 8
  br label %1068

1049:                                             ; preds = %1023, %1022
  %1050 = load ptr, ptr %8, align 8
  %1051 = load ptr, ptr %4, align 8
  %1052 = getelementptr inbounds %struct.CallInfo, ptr %1051, i32 0, i32 4
  %1053 = getelementptr inbounds %struct.anon, ptr %1052, i32 0, i32 0
  store ptr %1050, ptr %1053, align 8
  %1054 = load ptr, ptr %4, align 8
  %1055 = getelementptr inbounds %struct.CallInfo, ptr %1054, i32 0, i32 1
  %1056 = load ptr, ptr %1055, align 8
  %1057 = load ptr, ptr %3, align 8
  %1058 = getelementptr inbounds %struct.lua_State, ptr %1057, i32 0, i32 6
  store ptr %1056, ptr %1058, align 8
  %1059 = load ptr, ptr %3, align 8
  %1060 = load ptr, ptr %43, align 8
  %1061 = load ptr, ptr %44, align 8
  %1062 = load ptr, ptr %41, align 8
  %1063 = load ptr, ptr %42, align 8
  call void @luaV_finishget(ptr noundef %1059, ptr noundef %1060, ptr noundef %1061, ptr noundef %1062, ptr noundef %1063)
  %1064 = load ptr, ptr %4, align 8
  %1065 = getelementptr inbounds %struct.CallInfo, ptr %1064, i32 0, i32 4
  %1066 = getelementptr inbounds %struct.anon, ptr %1065, i32 0, i32 1
  %1067 = load volatile i32, ptr %1066, align 8
  store i32 %1067, ptr %9, align 4
  br label %1068

1068:                                             ; preds = %1049, %1036
  %1069 = load i32, ptr %9, align 4
  %1070 = icmp ne i32 %1069, 0
  %1071 = zext i1 %1070 to i32
  %1072 = sext i32 %1071 to i64
  %1073 = icmp ne i64 %1072, 0
  br i1 %1073, label %1074, label %1082

1074:                                             ; preds = %1068
  %1075 = load ptr, ptr %3, align 8
  %1076 = load ptr, ptr %8, align 8
  %1077 = call i32 @luaG_traceexec(ptr noundef %1075, ptr noundef %1076)
  store i32 %1077, ptr %9, align 4
  %1078 = load ptr, ptr %4, align 8
  %1079 = getelementptr inbounds %struct.CallInfo, ptr %1078, i32 0, i32 0
  %1080 = load ptr, ptr %1079, align 8
  %1081 = getelementptr inbounds %union.StackValue, ptr %1080, i64 1
  store ptr %1081, ptr %7, align 8
  br label %1082

1082:                                             ; preds = %1074, %1068
  %1083 = load ptr, ptr %8, align 8
  %1084 = getelementptr inbounds i32, ptr %1083, i32 1
  store ptr %1084, ptr %8, align 8
  %1085 = load i32, ptr %1083, align 4
  store i32 %1085, ptr %10, align 4
  %1086 = load i32, ptr %10, align 4
  %1087 = lshr i32 %1086, 0
  %1088 = and i32 %1087, 127
  %1089 = zext i32 %1088 to i64
  %1090 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1089
  %1091 = load ptr, ptr %1090, align 8
  br label %7120

1092:                                             ; preds = %7120
  %1093 = load ptr, ptr %7, align 8
  %1094 = load i32, ptr %10, align 4
  %1095 = lshr i32 %1094, 7
  %1096 = and i32 %1095, 255
  %1097 = sext i32 %1096 to i64
  %1098 = getelementptr inbounds %union.StackValue, ptr %1093, i64 %1097
  store ptr %1098, ptr %48, align 8
  %1099 = load ptr, ptr %7, align 8
  %1100 = load i32, ptr %10, align 4
  %1101 = lshr i32 %1100, 16
  %1102 = and i32 %1101, 255
  %1103 = sext i32 %1102 to i64
  %1104 = getelementptr inbounds %union.StackValue, ptr %1099, i64 %1103
  store ptr %1104, ptr %50, align 8
  %1105 = load ptr, ptr %7, align 8
  %1106 = load i32, ptr %10, align 4
  %1107 = lshr i32 %1106, 24
  %1108 = and i32 %1107, 255
  %1109 = sext i32 %1108 to i64
  %1110 = getelementptr inbounds %union.StackValue, ptr %1105, i64 %1109
  store ptr %1110, ptr %51, align 8
  %1111 = load ptr, ptr %51, align 8
  %1112 = getelementptr inbounds %struct.TValue, ptr %1111, i32 0, i32 1
  %1113 = load i8, ptr %1112, align 8
  %1114 = zext i8 %1113 to i32
  %1115 = icmp eq i32 %1114, 3
  br i1 %1115, label %1116, label %1164

1116:                                             ; preds = %1092
  %1117 = load ptr, ptr %51, align 8
  %1118 = getelementptr inbounds %struct.TValue, ptr %1117, i32 0, i32 0
  %1119 = load i64, ptr %1118, align 8
  store i64 %1119, ptr %52, align 8
  %1120 = load ptr, ptr %50, align 8
  %1121 = getelementptr inbounds %struct.TValue, ptr %1120, i32 0, i32 1
  %1122 = load i8, ptr %1121, align 8
  %1123 = zext i8 %1122 to i32
  %1124 = icmp eq i32 %1123, 69
  br i1 %1124, label %1126, label %1125

1125:                                             ; preds = %1116
  store ptr null, ptr %49, align 8
  br label %1161

1126:                                             ; preds = %1116
  %1127 = load i64, ptr %52, align 8
  %1128 = sub i64 %1127, 1
  %1129 = load ptr, ptr %50, align 8
  %1130 = getelementptr inbounds %struct.TValue, ptr %1129, i32 0, i32 0
  %1131 = load ptr, ptr %1130, align 8
  %1132 = getelementptr inbounds %struct.Table, ptr %1131, i32 0, i32 5
  %1133 = load i32, ptr %1132, align 4
  %1134 = zext i32 %1133 to i64
  %1135 = icmp ult i64 %1128, %1134
  br i1 %1135, label %1136, label %1145

1136:                                             ; preds = %1126
  %1137 = load ptr, ptr %50, align 8
  %1138 = getelementptr inbounds %struct.TValue, ptr %1137, i32 0, i32 0
  %1139 = load ptr, ptr %1138, align 8
  %1140 = getelementptr inbounds %struct.Table, ptr %1139, i32 0, i32 6
  %1141 = load ptr, ptr %1140, align 8
  %1142 = load i64, ptr %52, align 8
  %1143 = sub i64 %1142, 1
  %1144 = getelementptr inbounds %struct.TValue, ptr %1141, i64 %1143
  br label %1151

1145:                                             ; preds = %1126
  %1146 = load ptr, ptr %50, align 8
  %1147 = getelementptr inbounds %struct.TValue, ptr %1146, i32 0, i32 0
  %1148 = load ptr, ptr %1147, align 8
  %1149 = load i64, ptr %52, align 8
  %1150 = call ptr @luaH_getint(ptr noundef %1148, i64 noundef %1149)
  br label %1151

1151:                                             ; preds = %1145, %1136
  %1152 = phi ptr [ %1144, %1136 ], [ %1150, %1145 ]
  store ptr %1152, ptr %49, align 8
  %1153 = load ptr, ptr %49, align 8
  %1154 = getelementptr inbounds %struct.TValue, ptr %1153, i32 0, i32 1
  %1155 = load i8, ptr %1154, align 8
  %1156 = zext i8 %1155 to i32
  %1157 = and i32 %1156, 15
  %1158 = icmp eq i32 %1157, 0
  %1159 = xor i1 %1158, true
  %1160 = zext i1 %1159 to i32
  br label %1161

1161:                                             ; preds = %1151, %1125
  %1162 = phi i32 [ 0, %1125 ], [ %1160, %1151 ]
  %1163 = icmp ne i32 %1162, 0
  br i1 %1163, label %1184, label %1197

1164:                                             ; preds = %1092
  %1165 = load ptr, ptr %50, align 8
  %1166 = getelementptr inbounds %struct.TValue, ptr %1165, i32 0, i32 1
  %1167 = load i8, ptr %1166, align 8
  %1168 = zext i8 %1167 to i32
  %1169 = icmp eq i32 %1168, 69
  br i1 %1169, label %1171, label %1170

1170:                                             ; preds = %1164
  store ptr null, ptr %49, align 8
  br i1 false, label %1184, label %1197

1171:                                             ; preds = %1164
  %1172 = load ptr, ptr %50, align 8
  %1173 = getelementptr inbounds %struct.TValue, ptr %1172, i32 0, i32 0
  %1174 = load ptr, ptr %1173, align 8
  %1175 = load ptr, ptr %51, align 8
  %1176 = call ptr @luaH_get(ptr noundef %1174, ptr noundef %1175)
  store ptr %1176, ptr %49, align 8
  %1177 = load ptr, ptr %49, align 8
  %1178 = getelementptr inbounds %struct.TValue, ptr %1177, i32 0, i32 1
  %1179 = load i8, ptr %1178, align 8
  %1180 = zext i8 %1179 to i32
  %1181 = and i32 %1180, 15
  %1182 = icmp eq i32 %1181, 0
  %1183 = xor i1 %1182, true
  br i1 %1183, label %1184, label %1197

1184:                                             ; preds = %1171, %1170, %1161
  %1185 = load ptr, ptr %48, align 8
  store ptr %1185, ptr %53, align 8
  %1186 = load ptr, ptr %49, align 8
  store ptr %1186, ptr %54, align 8
  %1187 = load ptr, ptr %53, align 8
  %1188 = getelementptr inbounds %struct.TValue, ptr %1187, i32 0, i32 0
  %1189 = load ptr, ptr %54, align 8
  %1190 = getelementptr inbounds %struct.TValue, ptr %1189, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1188, ptr align 8 %1190, i64 8, i1 false)
  %1191 = load ptr, ptr %54, align 8
  %1192 = getelementptr inbounds %struct.TValue, ptr %1191, i32 0, i32 1
  %1193 = load i8, ptr %1192, align 8
  %1194 = load ptr, ptr %53, align 8
  %1195 = getelementptr inbounds %struct.TValue, ptr %1194, i32 0, i32 1
  store i8 %1193, ptr %1195, align 8
  %1196 = load ptr, ptr %3, align 8
  br label %1216

1197:                                             ; preds = %1171, %1170, %1161
  %1198 = load ptr, ptr %8, align 8
  %1199 = load ptr, ptr %4, align 8
  %1200 = getelementptr inbounds %struct.CallInfo, ptr %1199, i32 0, i32 4
  %1201 = getelementptr inbounds %struct.anon, ptr %1200, i32 0, i32 0
  store ptr %1198, ptr %1201, align 8
  %1202 = load ptr, ptr %4, align 8
  %1203 = getelementptr inbounds %struct.CallInfo, ptr %1202, i32 0, i32 1
  %1204 = load ptr, ptr %1203, align 8
  %1205 = load ptr, ptr %3, align 8
  %1206 = getelementptr inbounds %struct.lua_State, ptr %1205, i32 0, i32 6
  store ptr %1204, ptr %1206, align 8
  %1207 = load ptr, ptr %3, align 8
  %1208 = load ptr, ptr %50, align 8
  %1209 = load ptr, ptr %51, align 8
  %1210 = load ptr, ptr %48, align 8
  %1211 = load ptr, ptr %49, align 8
  call void @luaV_finishget(ptr noundef %1207, ptr noundef %1208, ptr noundef %1209, ptr noundef %1210, ptr noundef %1211)
  %1212 = load ptr, ptr %4, align 8
  %1213 = getelementptr inbounds %struct.CallInfo, ptr %1212, i32 0, i32 4
  %1214 = getelementptr inbounds %struct.anon, ptr %1213, i32 0, i32 1
  %1215 = load volatile i32, ptr %1214, align 8
  store i32 %1215, ptr %9, align 4
  br label %1216

1216:                                             ; preds = %1197, %1184
  %1217 = load i32, ptr %9, align 4
  %1218 = icmp ne i32 %1217, 0
  %1219 = zext i1 %1218 to i32
  %1220 = sext i32 %1219 to i64
  %1221 = icmp ne i64 %1220, 0
  br i1 %1221, label %1222, label %1230

1222:                                             ; preds = %1216
  %1223 = load ptr, ptr %3, align 8
  %1224 = load ptr, ptr %8, align 8
  %1225 = call i32 @luaG_traceexec(ptr noundef %1223, ptr noundef %1224)
  store i32 %1225, ptr %9, align 4
  %1226 = load ptr, ptr %4, align 8
  %1227 = getelementptr inbounds %struct.CallInfo, ptr %1226, i32 0, i32 0
  %1228 = load ptr, ptr %1227, align 8
  %1229 = getelementptr inbounds %union.StackValue, ptr %1228, i64 1
  store ptr %1229, ptr %7, align 8
  br label %1230

1230:                                             ; preds = %1222, %1216
  %1231 = load ptr, ptr %8, align 8
  %1232 = getelementptr inbounds i32, ptr %1231, i32 1
  store ptr %1232, ptr %8, align 8
  %1233 = load i32, ptr %1231, align 4
  store i32 %1233, ptr %10, align 4
  %1234 = load i32, ptr %10, align 4
  %1235 = lshr i32 %1234, 0
  %1236 = and i32 %1235, 127
  %1237 = zext i32 %1236 to i64
  %1238 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1237
  %1239 = load ptr, ptr %1238, align 8
  br label %7120

1240:                                             ; preds = %7120
  %1241 = load ptr, ptr %7, align 8
  %1242 = load i32, ptr %10, align 4
  %1243 = lshr i32 %1242, 7
  %1244 = and i32 %1243, 255
  %1245 = sext i32 %1244 to i64
  %1246 = getelementptr inbounds %union.StackValue, ptr %1241, i64 %1245
  store ptr %1246, ptr %55, align 8
  %1247 = load ptr, ptr %7, align 8
  %1248 = load i32, ptr %10, align 4
  %1249 = lshr i32 %1248, 16
  %1250 = and i32 %1249, 255
  %1251 = sext i32 %1250 to i64
  %1252 = getelementptr inbounds %union.StackValue, ptr %1247, i64 %1251
  store ptr %1252, ptr %57, align 8
  %1253 = load i32, ptr %10, align 4
  %1254 = lshr i32 %1253, 24
  %1255 = and i32 %1254, 255
  store i32 %1255, ptr %58, align 4
  %1256 = load ptr, ptr %57, align 8
  %1257 = getelementptr inbounds %struct.TValue, ptr %1256, i32 0, i32 1
  %1258 = load i8, ptr %1257, align 8
  %1259 = zext i8 %1258 to i32
  %1260 = icmp eq i32 %1259, 69
  br i1 %1260, label %1262, label %1261

1261:                                             ; preds = %1240
  store ptr null, ptr %56, align 8
  br i1 false, label %1299, label %1312

1262:                                             ; preds = %1240
  %1263 = load i32, ptr %58, align 4
  %1264 = sext i32 %1263 to i64
  %1265 = sub i64 %1264, 1
  %1266 = load ptr, ptr %57, align 8
  %1267 = getelementptr inbounds %struct.TValue, ptr %1266, i32 0, i32 0
  %1268 = load ptr, ptr %1267, align 8
  %1269 = getelementptr inbounds %struct.Table, ptr %1268, i32 0, i32 5
  %1270 = load i32, ptr %1269, align 4
  %1271 = zext i32 %1270 to i64
  %1272 = icmp ult i64 %1265, %1271
  br i1 %1272, label %1273, label %1283

1273:                                             ; preds = %1262
  %1274 = load ptr, ptr %57, align 8
  %1275 = getelementptr inbounds %struct.TValue, ptr %1274, i32 0, i32 0
  %1276 = load ptr, ptr %1275, align 8
  %1277 = getelementptr inbounds %struct.Table, ptr %1276, i32 0, i32 6
  %1278 = load ptr, ptr %1277, align 8
  %1279 = load i32, ptr %58, align 4
  %1280 = sub nsw i32 %1279, 1
  %1281 = sext i32 %1280 to i64
  %1282 = getelementptr inbounds %struct.TValue, ptr %1278, i64 %1281
  br label %1290

1283:                                             ; preds = %1262
  %1284 = load ptr, ptr %57, align 8
  %1285 = getelementptr inbounds %struct.TValue, ptr %1284, i32 0, i32 0
  %1286 = load ptr, ptr %1285, align 8
  %1287 = load i32, ptr %58, align 4
  %1288 = sext i32 %1287 to i64
  %1289 = call ptr @luaH_getint(ptr noundef %1286, i64 noundef %1288)
  br label %1290

1290:                                             ; preds = %1283, %1273
  %1291 = phi ptr [ %1282, %1273 ], [ %1289, %1283 ]
  store ptr %1291, ptr %56, align 8
  %1292 = load ptr, ptr %56, align 8
  %1293 = getelementptr inbounds %struct.TValue, ptr %1292, i32 0, i32 1
  %1294 = load i8, ptr %1293, align 8
  %1295 = zext i8 %1294 to i32
  %1296 = and i32 %1295, 15
  %1297 = icmp eq i32 %1296, 0
  %1298 = xor i1 %1297, true
  br i1 %1298, label %1299, label %1312

1299:                                             ; preds = %1290, %1261
  %1300 = load ptr, ptr %55, align 8
  store ptr %1300, ptr %59, align 8
  %1301 = load ptr, ptr %56, align 8
  store ptr %1301, ptr %60, align 8
  %1302 = load ptr, ptr %59, align 8
  %1303 = getelementptr inbounds %struct.TValue, ptr %1302, i32 0, i32 0
  %1304 = load ptr, ptr %60, align 8
  %1305 = getelementptr inbounds %struct.TValue, ptr %1304, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1303, ptr align 8 %1305, i64 8, i1 false)
  %1306 = load ptr, ptr %60, align 8
  %1307 = getelementptr inbounds %struct.TValue, ptr %1306, i32 0, i32 1
  %1308 = load i8, ptr %1307, align 8
  %1309 = load ptr, ptr %59, align 8
  %1310 = getelementptr inbounds %struct.TValue, ptr %1309, i32 0, i32 1
  store i8 %1308, ptr %1310, align 8
  %1311 = load ptr, ptr %3, align 8
  br label %1336

1312:                                             ; preds = %1290, %1261
  store ptr %61, ptr %62, align 8
  %1313 = load i32, ptr %58, align 4
  %1314 = sext i32 %1313 to i64
  %1315 = load ptr, ptr %62, align 8
  %1316 = getelementptr inbounds %struct.TValue, ptr %1315, i32 0, i32 0
  store i64 %1314, ptr %1316, align 8
  %1317 = load ptr, ptr %62, align 8
  %1318 = getelementptr inbounds %struct.TValue, ptr %1317, i32 0, i32 1
  store i8 3, ptr %1318, align 8
  %1319 = load ptr, ptr %8, align 8
  %1320 = load ptr, ptr %4, align 8
  %1321 = getelementptr inbounds %struct.CallInfo, ptr %1320, i32 0, i32 4
  %1322 = getelementptr inbounds %struct.anon, ptr %1321, i32 0, i32 0
  store ptr %1319, ptr %1322, align 8
  %1323 = load ptr, ptr %4, align 8
  %1324 = getelementptr inbounds %struct.CallInfo, ptr %1323, i32 0, i32 1
  %1325 = load ptr, ptr %1324, align 8
  %1326 = load ptr, ptr %3, align 8
  %1327 = getelementptr inbounds %struct.lua_State, ptr %1326, i32 0, i32 6
  store ptr %1325, ptr %1327, align 8
  %1328 = load ptr, ptr %3, align 8
  %1329 = load ptr, ptr %57, align 8
  %1330 = load ptr, ptr %55, align 8
  %1331 = load ptr, ptr %56, align 8
  call void @luaV_finishget(ptr noundef %1328, ptr noundef %1329, ptr noundef %61, ptr noundef %1330, ptr noundef %1331)
  %1332 = load ptr, ptr %4, align 8
  %1333 = getelementptr inbounds %struct.CallInfo, ptr %1332, i32 0, i32 4
  %1334 = getelementptr inbounds %struct.anon, ptr %1333, i32 0, i32 1
  %1335 = load volatile i32, ptr %1334, align 8
  store i32 %1335, ptr %9, align 4
  br label %1336

1336:                                             ; preds = %1312, %1299
  %1337 = load i32, ptr %9, align 4
  %1338 = icmp ne i32 %1337, 0
  %1339 = zext i1 %1338 to i32
  %1340 = sext i32 %1339 to i64
  %1341 = icmp ne i64 %1340, 0
  br i1 %1341, label %1342, label %1350

1342:                                             ; preds = %1336
  %1343 = load ptr, ptr %3, align 8
  %1344 = load ptr, ptr %8, align 8
  %1345 = call i32 @luaG_traceexec(ptr noundef %1343, ptr noundef %1344)
  store i32 %1345, ptr %9, align 4
  %1346 = load ptr, ptr %4, align 8
  %1347 = getelementptr inbounds %struct.CallInfo, ptr %1346, i32 0, i32 0
  %1348 = load ptr, ptr %1347, align 8
  %1349 = getelementptr inbounds %union.StackValue, ptr %1348, i64 1
  store ptr %1349, ptr %7, align 8
  br label %1350

1350:                                             ; preds = %1342, %1336
  %1351 = load ptr, ptr %8, align 8
  %1352 = getelementptr inbounds i32, ptr %1351, i32 1
  store ptr %1352, ptr %8, align 8
  %1353 = load i32, ptr %1351, align 4
  store i32 %1353, ptr %10, align 4
  %1354 = load i32, ptr %10, align 4
  %1355 = lshr i32 %1354, 0
  %1356 = and i32 %1355, 127
  %1357 = zext i32 %1356 to i64
  %1358 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1357
  %1359 = load ptr, ptr %1358, align 8
  br label %7120

1360:                                             ; preds = %7120
  %1361 = load ptr, ptr %7, align 8
  %1362 = load i32, ptr %10, align 4
  %1363 = lshr i32 %1362, 7
  %1364 = and i32 %1363, 255
  %1365 = sext i32 %1364 to i64
  %1366 = getelementptr inbounds %union.StackValue, ptr %1361, i64 %1365
  store ptr %1366, ptr %63, align 8
  %1367 = load ptr, ptr %7, align 8
  %1368 = load i32, ptr %10, align 4
  %1369 = lshr i32 %1368, 16
  %1370 = and i32 %1369, 255
  %1371 = sext i32 %1370 to i64
  %1372 = getelementptr inbounds %union.StackValue, ptr %1367, i64 %1371
  store ptr %1372, ptr %65, align 8
  %1373 = load ptr, ptr %6, align 8
  %1374 = load i32, ptr %10, align 4
  %1375 = lshr i32 %1374, 24
  %1376 = and i32 %1375, 255
  %1377 = sext i32 %1376 to i64
  %1378 = getelementptr inbounds %struct.TValue, ptr %1373, i64 %1377
  store ptr %1378, ptr %66, align 8
  %1379 = load ptr, ptr %66, align 8
  %1380 = getelementptr inbounds %struct.TValue, ptr %1379, i32 0, i32 0
  %1381 = load ptr, ptr %1380, align 8
  store ptr %1381, ptr %67, align 8
  %1382 = load ptr, ptr %65, align 8
  %1383 = getelementptr inbounds %struct.TValue, ptr %1382, i32 0, i32 1
  %1384 = load i8, ptr %1383, align 8
  %1385 = zext i8 %1384 to i32
  %1386 = icmp eq i32 %1385, 69
  br i1 %1386, label %1388, label %1387

1387:                                             ; preds = %1360
  store ptr null, ptr %64, align 8
  br i1 false, label %1401, label %1414

1388:                                             ; preds = %1360
  %1389 = load ptr, ptr %65, align 8
  %1390 = getelementptr inbounds %struct.TValue, ptr %1389, i32 0, i32 0
  %1391 = load ptr, ptr %1390, align 8
  %1392 = load ptr, ptr %67, align 8
  %1393 = call ptr @luaH_getshortstr(ptr noundef %1391, ptr noundef %1392)
  store ptr %1393, ptr %64, align 8
  %1394 = load ptr, ptr %64, align 8
  %1395 = getelementptr inbounds %struct.TValue, ptr %1394, i32 0, i32 1
  %1396 = load i8, ptr %1395, align 8
  %1397 = zext i8 %1396 to i32
  %1398 = and i32 %1397, 15
  %1399 = icmp eq i32 %1398, 0
  %1400 = xor i1 %1399, true
  br i1 %1400, label %1401, label %1414

1401:                                             ; preds = %1388, %1387
  %1402 = load ptr, ptr %63, align 8
  store ptr %1402, ptr %68, align 8
  %1403 = load ptr, ptr %64, align 8
  store ptr %1403, ptr %69, align 8
  %1404 = load ptr, ptr %68, align 8
  %1405 = getelementptr inbounds %struct.TValue, ptr %1404, i32 0, i32 0
  %1406 = load ptr, ptr %69, align 8
  %1407 = getelementptr inbounds %struct.TValue, ptr %1406, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1405, ptr align 8 %1407, i64 8, i1 false)
  %1408 = load ptr, ptr %69, align 8
  %1409 = getelementptr inbounds %struct.TValue, ptr %1408, i32 0, i32 1
  %1410 = load i8, ptr %1409, align 8
  %1411 = load ptr, ptr %68, align 8
  %1412 = getelementptr inbounds %struct.TValue, ptr %1411, i32 0, i32 1
  store i8 %1410, ptr %1412, align 8
  %1413 = load ptr, ptr %3, align 8
  br label %1433

1414:                                             ; preds = %1388, %1387
  %1415 = load ptr, ptr %8, align 8
  %1416 = load ptr, ptr %4, align 8
  %1417 = getelementptr inbounds %struct.CallInfo, ptr %1416, i32 0, i32 4
  %1418 = getelementptr inbounds %struct.anon, ptr %1417, i32 0, i32 0
  store ptr %1415, ptr %1418, align 8
  %1419 = load ptr, ptr %4, align 8
  %1420 = getelementptr inbounds %struct.CallInfo, ptr %1419, i32 0, i32 1
  %1421 = load ptr, ptr %1420, align 8
  %1422 = load ptr, ptr %3, align 8
  %1423 = getelementptr inbounds %struct.lua_State, ptr %1422, i32 0, i32 6
  store ptr %1421, ptr %1423, align 8
  %1424 = load ptr, ptr %3, align 8
  %1425 = load ptr, ptr %65, align 8
  %1426 = load ptr, ptr %66, align 8
  %1427 = load ptr, ptr %63, align 8
  %1428 = load ptr, ptr %64, align 8
  call void @luaV_finishget(ptr noundef %1424, ptr noundef %1425, ptr noundef %1426, ptr noundef %1427, ptr noundef %1428)
  %1429 = load ptr, ptr %4, align 8
  %1430 = getelementptr inbounds %struct.CallInfo, ptr %1429, i32 0, i32 4
  %1431 = getelementptr inbounds %struct.anon, ptr %1430, i32 0, i32 1
  %1432 = load volatile i32, ptr %1431, align 8
  store i32 %1432, ptr %9, align 4
  br label %1433

1433:                                             ; preds = %1414, %1401
  %1434 = load i32, ptr %9, align 4
  %1435 = icmp ne i32 %1434, 0
  %1436 = zext i1 %1435 to i32
  %1437 = sext i32 %1436 to i64
  %1438 = icmp ne i64 %1437, 0
  br i1 %1438, label %1439, label %1447

1439:                                             ; preds = %1433
  %1440 = load ptr, ptr %3, align 8
  %1441 = load ptr, ptr %8, align 8
  %1442 = call i32 @luaG_traceexec(ptr noundef %1440, ptr noundef %1441)
  store i32 %1442, ptr %9, align 4
  %1443 = load ptr, ptr %4, align 8
  %1444 = getelementptr inbounds %struct.CallInfo, ptr %1443, i32 0, i32 0
  %1445 = load ptr, ptr %1444, align 8
  %1446 = getelementptr inbounds %union.StackValue, ptr %1445, i64 1
  store ptr %1446, ptr %7, align 8
  br label %1447

1447:                                             ; preds = %1439, %1433
  %1448 = load ptr, ptr %8, align 8
  %1449 = getelementptr inbounds i32, ptr %1448, i32 1
  store ptr %1449, ptr %8, align 8
  %1450 = load i32, ptr %1448, align 4
  store i32 %1450, ptr %10, align 4
  %1451 = load i32, ptr %10, align 4
  %1452 = lshr i32 %1451, 0
  %1453 = and i32 %1452, 127
  %1454 = zext i32 %1453 to i64
  %1455 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1454
  %1456 = load ptr, ptr %1455, align 8
  br label %7120

1457:                                             ; preds = %7120
  %1458 = load ptr, ptr %5, align 8
  %1459 = getelementptr inbounds %struct.LClosure, ptr %1458, i32 0, i32 6
  %1460 = load i32, ptr %10, align 4
  %1461 = lshr i32 %1460, 7
  %1462 = and i32 %1461, 255
  %1463 = sext i32 %1462 to i64
  %1464 = getelementptr inbounds [1 x ptr], ptr %1459, i64 0, i64 %1463
  %1465 = load ptr, ptr %1464, align 8
  %1466 = getelementptr inbounds %struct.UpVal, ptr %1465, i32 0, i32 3
  %1467 = load ptr, ptr %1466, align 8
  store ptr %1467, ptr %71, align 8
  %1468 = load ptr, ptr %6, align 8
  %1469 = load i32, ptr %10, align 4
  %1470 = lshr i32 %1469, 16
  %1471 = and i32 %1470, 255
  %1472 = sext i32 %1471 to i64
  %1473 = getelementptr inbounds %struct.TValue, ptr %1468, i64 %1472
  store ptr %1473, ptr %72, align 8
  %1474 = load i32, ptr %10, align 4
  %1475 = and i32 %1474, 32768
  %1476 = icmp ne i32 %1475, 0
  br i1 %1476, label %1477, label %1484

1477:                                             ; preds = %1457
  %1478 = load ptr, ptr %6, align 8
  %1479 = load i32, ptr %10, align 4
  %1480 = lshr i32 %1479, 24
  %1481 = and i32 %1480, 255
  %1482 = sext i32 %1481 to i64
  %1483 = getelementptr inbounds %struct.TValue, ptr %1478, i64 %1482
  br label %1491

1484:                                             ; preds = %1457
  %1485 = load ptr, ptr %7, align 8
  %1486 = load i32, ptr %10, align 4
  %1487 = lshr i32 %1486, 24
  %1488 = and i32 %1487, 255
  %1489 = sext i32 %1488 to i64
  %1490 = getelementptr inbounds %union.StackValue, ptr %1485, i64 %1489
  br label %1491

1491:                                             ; preds = %1484, %1477
  %1492 = phi ptr [ %1483, %1477 ], [ %1490, %1484 ]
  store ptr %1492, ptr %73, align 8
  %1493 = load ptr, ptr %72, align 8
  %1494 = getelementptr inbounds %struct.TValue, ptr %1493, i32 0, i32 0
  %1495 = load ptr, ptr %1494, align 8
  store ptr %1495, ptr %74, align 8
  %1496 = load ptr, ptr %71, align 8
  %1497 = getelementptr inbounds %struct.TValue, ptr %1496, i32 0, i32 1
  %1498 = load i8, ptr %1497, align 8
  %1499 = zext i8 %1498 to i32
  %1500 = icmp eq i32 %1499, 69
  br i1 %1500, label %1502, label %1501

1501:                                             ; preds = %1491
  store ptr null, ptr %70, align 8
  br i1 false, label %1515, label %1561

1502:                                             ; preds = %1491
  %1503 = load ptr, ptr %71, align 8
  %1504 = getelementptr inbounds %struct.TValue, ptr %1503, i32 0, i32 0
  %1505 = load ptr, ptr %1504, align 8
  %1506 = load ptr, ptr %74, align 8
  %1507 = call ptr @luaH_getshortstr(ptr noundef %1505, ptr noundef %1506)
  store ptr %1507, ptr %70, align 8
  %1508 = load ptr, ptr %70, align 8
  %1509 = getelementptr inbounds %struct.TValue, ptr %1508, i32 0, i32 1
  %1510 = load i8, ptr %1509, align 8
  %1511 = zext i8 %1510 to i32
  %1512 = and i32 %1511, 15
  %1513 = icmp eq i32 %1512, 0
  %1514 = xor i1 %1513, true
  br i1 %1514, label %1515, label %1561

1515:                                             ; preds = %1502, %1501
  %1516 = load ptr, ptr %70, align 8
  store ptr %1516, ptr %75, align 8
  %1517 = load ptr, ptr %73, align 8
  store ptr %1517, ptr %76, align 8
  %1518 = load ptr, ptr %75, align 8
  %1519 = getelementptr inbounds %struct.TValue, ptr %1518, i32 0, i32 0
  %1520 = load ptr, ptr %76, align 8
  %1521 = getelementptr inbounds %struct.TValue, ptr %1520, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1519, ptr align 8 %1521, i64 8, i1 false)
  %1522 = load ptr, ptr %76, align 8
  %1523 = getelementptr inbounds %struct.TValue, ptr %1522, i32 0, i32 1
  %1524 = load i8, ptr %1523, align 8
  %1525 = load ptr, ptr %75, align 8
  %1526 = getelementptr inbounds %struct.TValue, ptr %1525, i32 0, i32 1
  store i8 %1524, ptr %1526, align 8
  %1527 = load ptr, ptr %3, align 8
  %1528 = load ptr, ptr %73, align 8
  %1529 = getelementptr inbounds %struct.TValue, ptr %1528, i32 0, i32 1
  %1530 = load i8, ptr %1529, align 8
  %1531 = zext i8 %1530 to i32
  %1532 = and i32 %1531, 64
  %1533 = icmp ne i32 %1532, 0
  br i1 %1533, label %1534, label %1559

1534:                                             ; preds = %1515
  %1535 = load ptr, ptr %71, align 8
  %1536 = getelementptr inbounds %struct.TValue, ptr %1535, i32 0, i32 0
  %1537 = load ptr, ptr %1536, align 8
  %1538 = getelementptr inbounds %struct.GCObject, ptr %1537, i32 0, i32 2
  %1539 = load i8, ptr %1538, align 1
  %1540 = zext i8 %1539 to i32
  %1541 = and i32 %1540, 32
  %1542 = icmp ne i32 %1541, 0
  br i1 %1542, label %1543, label %1557

1543:                                             ; preds = %1534
  %1544 = load ptr, ptr %73, align 8
  %1545 = getelementptr inbounds %struct.TValue, ptr %1544, i32 0, i32 0
  %1546 = load ptr, ptr %1545, align 8
  %1547 = getelementptr inbounds %struct.GCObject, ptr %1546, i32 0, i32 2
  %1548 = load i8, ptr %1547, align 1
  %1549 = zext i8 %1548 to i32
  %1550 = and i32 %1549, 24
  %1551 = icmp ne i32 %1550, 0
  br i1 %1551, label %1552, label %1557

1552:                                             ; preds = %1543
  %1553 = load ptr, ptr %3, align 8
  %1554 = load ptr, ptr %71, align 8
  %1555 = getelementptr inbounds %struct.TValue, ptr %1554, i32 0, i32 0
  %1556 = load ptr, ptr %1555, align 8
  call void @luaC_barrierback_(ptr noundef %1553, ptr noundef %1556)
  br label %1558

1557:                                             ; preds = %1543, %1534
  br label %1558

1558:                                             ; preds = %1557, %1552
  br label %1560

1559:                                             ; preds = %1515
  br label %1560

1560:                                             ; preds = %1559, %1558
  br label %1580

1561:                                             ; preds = %1502, %1501
  %1562 = load ptr, ptr %8, align 8
  %1563 = load ptr, ptr %4, align 8
  %1564 = getelementptr inbounds %struct.CallInfo, ptr %1563, i32 0, i32 4
  %1565 = getelementptr inbounds %struct.anon, ptr %1564, i32 0, i32 0
  store ptr %1562, ptr %1565, align 8
  %1566 = load ptr, ptr %4, align 8
  %1567 = getelementptr inbounds %struct.CallInfo, ptr %1566, i32 0, i32 1
  %1568 = load ptr, ptr %1567, align 8
  %1569 = load ptr, ptr %3, align 8
  %1570 = getelementptr inbounds %struct.lua_State, ptr %1569, i32 0, i32 6
  store ptr %1568, ptr %1570, align 8
  %1571 = load ptr, ptr %3, align 8
  %1572 = load ptr, ptr %71, align 8
  %1573 = load ptr, ptr %72, align 8
  %1574 = load ptr, ptr %73, align 8
  %1575 = load ptr, ptr %70, align 8
  call void @luaV_finishset(ptr noundef %1571, ptr noundef %1572, ptr noundef %1573, ptr noundef %1574, ptr noundef %1575)
  %1576 = load ptr, ptr %4, align 8
  %1577 = getelementptr inbounds %struct.CallInfo, ptr %1576, i32 0, i32 4
  %1578 = getelementptr inbounds %struct.anon, ptr %1577, i32 0, i32 1
  %1579 = load volatile i32, ptr %1578, align 8
  store i32 %1579, ptr %9, align 4
  br label %1580

1580:                                             ; preds = %1561, %1560
  %1581 = load i32, ptr %9, align 4
  %1582 = icmp ne i32 %1581, 0
  %1583 = zext i1 %1582 to i32
  %1584 = sext i32 %1583 to i64
  %1585 = icmp ne i64 %1584, 0
  br i1 %1585, label %1586, label %1594

1586:                                             ; preds = %1580
  %1587 = load ptr, ptr %3, align 8
  %1588 = load ptr, ptr %8, align 8
  %1589 = call i32 @luaG_traceexec(ptr noundef %1587, ptr noundef %1588)
  store i32 %1589, ptr %9, align 4
  %1590 = load ptr, ptr %4, align 8
  %1591 = getelementptr inbounds %struct.CallInfo, ptr %1590, i32 0, i32 0
  %1592 = load ptr, ptr %1591, align 8
  %1593 = getelementptr inbounds %union.StackValue, ptr %1592, i64 1
  store ptr %1593, ptr %7, align 8
  br label %1594

1594:                                             ; preds = %1586, %1580
  %1595 = load ptr, ptr %8, align 8
  %1596 = getelementptr inbounds i32, ptr %1595, i32 1
  store ptr %1596, ptr %8, align 8
  %1597 = load i32, ptr %1595, align 4
  store i32 %1597, ptr %10, align 4
  %1598 = load i32, ptr %10, align 4
  %1599 = lshr i32 %1598, 0
  %1600 = and i32 %1599, 127
  %1601 = zext i32 %1600 to i64
  %1602 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1601
  %1603 = load ptr, ptr %1602, align 8
  br label %7120

1604:                                             ; preds = %7120
  %1605 = load ptr, ptr %7, align 8
  %1606 = load i32, ptr %10, align 4
  %1607 = lshr i32 %1606, 7
  %1608 = and i32 %1607, 255
  %1609 = sext i32 %1608 to i64
  %1610 = getelementptr inbounds %union.StackValue, ptr %1605, i64 %1609
  store ptr %1610, ptr %77, align 8
  %1611 = load ptr, ptr %7, align 8
  %1612 = load i32, ptr %10, align 4
  %1613 = lshr i32 %1612, 16
  %1614 = and i32 %1613, 255
  %1615 = sext i32 %1614 to i64
  %1616 = getelementptr inbounds %union.StackValue, ptr %1611, i64 %1615
  store ptr %1616, ptr %79, align 8
  %1617 = load i32, ptr %10, align 4
  %1618 = and i32 %1617, 32768
  %1619 = icmp ne i32 %1618, 0
  br i1 %1619, label %1620, label %1627

1620:                                             ; preds = %1604
  %1621 = load ptr, ptr %6, align 8
  %1622 = load i32, ptr %10, align 4
  %1623 = lshr i32 %1622, 24
  %1624 = and i32 %1623, 255
  %1625 = sext i32 %1624 to i64
  %1626 = getelementptr inbounds %struct.TValue, ptr %1621, i64 %1625
  br label %1634

1627:                                             ; preds = %1604
  %1628 = load ptr, ptr %7, align 8
  %1629 = load i32, ptr %10, align 4
  %1630 = lshr i32 %1629, 24
  %1631 = and i32 %1630, 255
  %1632 = sext i32 %1631 to i64
  %1633 = getelementptr inbounds %union.StackValue, ptr %1628, i64 %1632
  br label %1634

1634:                                             ; preds = %1627, %1620
  %1635 = phi ptr [ %1626, %1620 ], [ %1633, %1627 ]
  store ptr %1635, ptr %80, align 8
  %1636 = load ptr, ptr %79, align 8
  %1637 = getelementptr inbounds %struct.TValue, ptr %1636, i32 0, i32 1
  %1638 = load i8, ptr %1637, align 8
  %1639 = zext i8 %1638 to i32
  %1640 = icmp eq i32 %1639, 3
  br i1 %1640, label %1641, label %1689

1641:                                             ; preds = %1634
  %1642 = load ptr, ptr %79, align 8
  %1643 = getelementptr inbounds %struct.TValue, ptr %1642, i32 0, i32 0
  %1644 = load i64, ptr %1643, align 8
  store i64 %1644, ptr %81, align 8
  %1645 = load ptr, ptr %77, align 8
  %1646 = getelementptr inbounds %struct.TValue, ptr %1645, i32 0, i32 1
  %1647 = load i8, ptr %1646, align 8
  %1648 = zext i8 %1647 to i32
  %1649 = icmp eq i32 %1648, 69
  br i1 %1649, label %1651, label %1650

1650:                                             ; preds = %1641
  store ptr null, ptr %78, align 8
  br label %1686

1651:                                             ; preds = %1641
  %1652 = load i64, ptr %81, align 8
  %1653 = sub i64 %1652, 1
  %1654 = load ptr, ptr %77, align 8
  %1655 = getelementptr inbounds %struct.TValue, ptr %1654, i32 0, i32 0
  %1656 = load ptr, ptr %1655, align 8
  %1657 = getelementptr inbounds %struct.Table, ptr %1656, i32 0, i32 5
  %1658 = load i32, ptr %1657, align 4
  %1659 = zext i32 %1658 to i64
  %1660 = icmp ult i64 %1653, %1659
  br i1 %1660, label %1661, label %1670

1661:                                             ; preds = %1651
  %1662 = load ptr, ptr %77, align 8
  %1663 = getelementptr inbounds %struct.TValue, ptr %1662, i32 0, i32 0
  %1664 = load ptr, ptr %1663, align 8
  %1665 = getelementptr inbounds %struct.Table, ptr %1664, i32 0, i32 6
  %1666 = load ptr, ptr %1665, align 8
  %1667 = load i64, ptr %81, align 8
  %1668 = sub i64 %1667, 1
  %1669 = getelementptr inbounds %struct.TValue, ptr %1666, i64 %1668
  br label %1676

1670:                                             ; preds = %1651
  %1671 = load ptr, ptr %77, align 8
  %1672 = getelementptr inbounds %struct.TValue, ptr %1671, i32 0, i32 0
  %1673 = load ptr, ptr %1672, align 8
  %1674 = load i64, ptr %81, align 8
  %1675 = call ptr @luaH_getint(ptr noundef %1673, i64 noundef %1674)
  br label %1676

1676:                                             ; preds = %1670, %1661
  %1677 = phi ptr [ %1669, %1661 ], [ %1675, %1670 ]
  store ptr %1677, ptr %78, align 8
  %1678 = load ptr, ptr %78, align 8
  %1679 = getelementptr inbounds %struct.TValue, ptr %1678, i32 0, i32 1
  %1680 = load i8, ptr %1679, align 8
  %1681 = zext i8 %1680 to i32
  %1682 = and i32 %1681, 15
  %1683 = icmp eq i32 %1682, 0
  %1684 = xor i1 %1683, true
  %1685 = zext i1 %1684 to i32
  br label %1686

1686:                                             ; preds = %1676, %1650
  %1687 = phi i32 [ 0, %1650 ], [ %1685, %1676 ]
  %1688 = icmp ne i32 %1687, 0
  br i1 %1688, label %1709, label %1755

1689:                                             ; preds = %1634
  %1690 = load ptr, ptr %77, align 8
  %1691 = getelementptr inbounds %struct.TValue, ptr %1690, i32 0, i32 1
  %1692 = load i8, ptr %1691, align 8
  %1693 = zext i8 %1692 to i32
  %1694 = icmp eq i32 %1693, 69
  br i1 %1694, label %1696, label %1695

1695:                                             ; preds = %1689
  store ptr null, ptr %78, align 8
  br i1 false, label %1709, label %1755

1696:                                             ; preds = %1689
  %1697 = load ptr, ptr %77, align 8
  %1698 = getelementptr inbounds %struct.TValue, ptr %1697, i32 0, i32 0
  %1699 = load ptr, ptr %1698, align 8
  %1700 = load ptr, ptr %79, align 8
  %1701 = call ptr @luaH_get(ptr noundef %1699, ptr noundef %1700)
  store ptr %1701, ptr %78, align 8
  %1702 = load ptr, ptr %78, align 8
  %1703 = getelementptr inbounds %struct.TValue, ptr %1702, i32 0, i32 1
  %1704 = load i8, ptr %1703, align 8
  %1705 = zext i8 %1704 to i32
  %1706 = and i32 %1705, 15
  %1707 = icmp eq i32 %1706, 0
  %1708 = xor i1 %1707, true
  br i1 %1708, label %1709, label %1755

1709:                                             ; preds = %1696, %1695, %1686
  %1710 = load ptr, ptr %78, align 8
  store ptr %1710, ptr %82, align 8
  %1711 = load ptr, ptr %80, align 8
  store ptr %1711, ptr %83, align 8
  %1712 = load ptr, ptr %82, align 8
  %1713 = getelementptr inbounds %struct.TValue, ptr %1712, i32 0, i32 0
  %1714 = load ptr, ptr %83, align 8
  %1715 = getelementptr inbounds %struct.TValue, ptr %1714, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1713, ptr align 8 %1715, i64 8, i1 false)
  %1716 = load ptr, ptr %83, align 8
  %1717 = getelementptr inbounds %struct.TValue, ptr %1716, i32 0, i32 1
  %1718 = load i8, ptr %1717, align 8
  %1719 = load ptr, ptr %82, align 8
  %1720 = getelementptr inbounds %struct.TValue, ptr %1719, i32 0, i32 1
  store i8 %1718, ptr %1720, align 8
  %1721 = load ptr, ptr %3, align 8
  %1722 = load ptr, ptr %80, align 8
  %1723 = getelementptr inbounds %struct.TValue, ptr %1722, i32 0, i32 1
  %1724 = load i8, ptr %1723, align 8
  %1725 = zext i8 %1724 to i32
  %1726 = and i32 %1725, 64
  %1727 = icmp ne i32 %1726, 0
  br i1 %1727, label %1728, label %1753

1728:                                             ; preds = %1709
  %1729 = load ptr, ptr %77, align 8
  %1730 = getelementptr inbounds %struct.TValue, ptr %1729, i32 0, i32 0
  %1731 = load ptr, ptr %1730, align 8
  %1732 = getelementptr inbounds %struct.GCObject, ptr %1731, i32 0, i32 2
  %1733 = load i8, ptr %1732, align 1
  %1734 = zext i8 %1733 to i32
  %1735 = and i32 %1734, 32
  %1736 = icmp ne i32 %1735, 0
  br i1 %1736, label %1737, label %1751

1737:                                             ; preds = %1728
  %1738 = load ptr, ptr %80, align 8
  %1739 = getelementptr inbounds %struct.TValue, ptr %1738, i32 0, i32 0
  %1740 = load ptr, ptr %1739, align 8
  %1741 = getelementptr inbounds %struct.GCObject, ptr %1740, i32 0, i32 2
  %1742 = load i8, ptr %1741, align 1
  %1743 = zext i8 %1742 to i32
  %1744 = and i32 %1743, 24
  %1745 = icmp ne i32 %1744, 0
  br i1 %1745, label %1746, label %1751

1746:                                             ; preds = %1737
  %1747 = load ptr, ptr %3, align 8
  %1748 = load ptr, ptr %77, align 8
  %1749 = getelementptr inbounds %struct.TValue, ptr %1748, i32 0, i32 0
  %1750 = load ptr, ptr %1749, align 8
  call void @luaC_barrierback_(ptr noundef %1747, ptr noundef %1750)
  br label %1752

1751:                                             ; preds = %1737, %1728
  br label %1752

1752:                                             ; preds = %1751, %1746
  br label %1754

1753:                                             ; preds = %1709
  br label %1754

1754:                                             ; preds = %1753, %1752
  br label %1774

1755:                                             ; preds = %1696, %1695, %1686
  %1756 = load ptr, ptr %8, align 8
  %1757 = load ptr, ptr %4, align 8
  %1758 = getelementptr inbounds %struct.CallInfo, ptr %1757, i32 0, i32 4
  %1759 = getelementptr inbounds %struct.anon, ptr %1758, i32 0, i32 0
  store ptr %1756, ptr %1759, align 8
  %1760 = load ptr, ptr %4, align 8
  %1761 = getelementptr inbounds %struct.CallInfo, ptr %1760, i32 0, i32 1
  %1762 = load ptr, ptr %1761, align 8
  %1763 = load ptr, ptr %3, align 8
  %1764 = getelementptr inbounds %struct.lua_State, ptr %1763, i32 0, i32 6
  store ptr %1762, ptr %1764, align 8
  %1765 = load ptr, ptr %3, align 8
  %1766 = load ptr, ptr %77, align 8
  %1767 = load ptr, ptr %79, align 8
  %1768 = load ptr, ptr %80, align 8
  %1769 = load ptr, ptr %78, align 8
  call void @luaV_finishset(ptr noundef %1765, ptr noundef %1766, ptr noundef %1767, ptr noundef %1768, ptr noundef %1769)
  %1770 = load ptr, ptr %4, align 8
  %1771 = getelementptr inbounds %struct.CallInfo, ptr %1770, i32 0, i32 4
  %1772 = getelementptr inbounds %struct.anon, ptr %1771, i32 0, i32 1
  %1773 = load volatile i32, ptr %1772, align 8
  store i32 %1773, ptr %9, align 4
  br label %1774

1774:                                             ; preds = %1755, %1754
  %1775 = load i32, ptr %9, align 4
  %1776 = icmp ne i32 %1775, 0
  %1777 = zext i1 %1776 to i32
  %1778 = sext i32 %1777 to i64
  %1779 = icmp ne i64 %1778, 0
  br i1 %1779, label %1780, label %1788

1780:                                             ; preds = %1774
  %1781 = load ptr, ptr %3, align 8
  %1782 = load ptr, ptr %8, align 8
  %1783 = call i32 @luaG_traceexec(ptr noundef %1781, ptr noundef %1782)
  store i32 %1783, ptr %9, align 4
  %1784 = load ptr, ptr %4, align 8
  %1785 = getelementptr inbounds %struct.CallInfo, ptr %1784, i32 0, i32 0
  %1786 = load ptr, ptr %1785, align 8
  %1787 = getelementptr inbounds %union.StackValue, ptr %1786, i64 1
  store ptr %1787, ptr %7, align 8
  br label %1788

1788:                                             ; preds = %1780, %1774
  %1789 = load ptr, ptr %8, align 8
  %1790 = getelementptr inbounds i32, ptr %1789, i32 1
  store ptr %1790, ptr %8, align 8
  %1791 = load i32, ptr %1789, align 4
  store i32 %1791, ptr %10, align 4
  %1792 = load i32, ptr %10, align 4
  %1793 = lshr i32 %1792, 0
  %1794 = and i32 %1793, 127
  %1795 = zext i32 %1794 to i64
  %1796 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1795
  %1797 = load ptr, ptr %1796, align 8
  br label %7120

1798:                                             ; preds = %7120
  %1799 = load ptr, ptr %7, align 8
  %1800 = load i32, ptr %10, align 4
  %1801 = lshr i32 %1800, 7
  %1802 = and i32 %1801, 255
  %1803 = sext i32 %1802 to i64
  %1804 = getelementptr inbounds %union.StackValue, ptr %1799, i64 %1803
  store ptr %1804, ptr %84, align 8
  %1805 = load i32, ptr %10, align 4
  %1806 = lshr i32 %1805, 16
  %1807 = and i32 %1806, 255
  store i32 %1807, ptr %86, align 4
  %1808 = load i32, ptr %10, align 4
  %1809 = and i32 %1808, 32768
  %1810 = icmp ne i32 %1809, 0
  br i1 %1810, label %1811, label %1818

1811:                                             ; preds = %1798
  %1812 = load ptr, ptr %6, align 8
  %1813 = load i32, ptr %10, align 4
  %1814 = lshr i32 %1813, 24
  %1815 = and i32 %1814, 255
  %1816 = sext i32 %1815 to i64
  %1817 = getelementptr inbounds %struct.TValue, ptr %1812, i64 %1816
  br label %1825

1818:                                             ; preds = %1798
  %1819 = load ptr, ptr %7, align 8
  %1820 = load i32, ptr %10, align 4
  %1821 = lshr i32 %1820, 24
  %1822 = and i32 %1821, 255
  %1823 = sext i32 %1822 to i64
  %1824 = getelementptr inbounds %union.StackValue, ptr %1819, i64 %1823
  br label %1825

1825:                                             ; preds = %1818, %1811
  %1826 = phi ptr [ %1817, %1811 ], [ %1824, %1818 ]
  store ptr %1826, ptr %87, align 8
  %1827 = load ptr, ptr %84, align 8
  %1828 = getelementptr inbounds %struct.TValue, ptr %1827, i32 0, i32 1
  %1829 = load i8, ptr %1828, align 8
  %1830 = zext i8 %1829 to i32
  %1831 = icmp eq i32 %1830, 69
  br i1 %1831, label %1833, label %1832

1832:                                             ; preds = %1825
  store ptr null, ptr %85, align 8
  br i1 false, label %1870, label %1916

1833:                                             ; preds = %1825
  %1834 = load i32, ptr %86, align 4
  %1835 = sext i32 %1834 to i64
  %1836 = sub i64 %1835, 1
  %1837 = load ptr, ptr %84, align 8
  %1838 = getelementptr inbounds %struct.TValue, ptr %1837, i32 0, i32 0
  %1839 = load ptr, ptr %1838, align 8
  %1840 = getelementptr inbounds %struct.Table, ptr %1839, i32 0, i32 5
  %1841 = load i32, ptr %1840, align 4
  %1842 = zext i32 %1841 to i64
  %1843 = icmp ult i64 %1836, %1842
  br i1 %1843, label %1844, label %1854

1844:                                             ; preds = %1833
  %1845 = load ptr, ptr %84, align 8
  %1846 = getelementptr inbounds %struct.TValue, ptr %1845, i32 0, i32 0
  %1847 = load ptr, ptr %1846, align 8
  %1848 = getelementptr inbounds %struct.Table, ptr %1847, i32 0, i32 6
  %1849 = load ptr, ptr %1848, align 8
  %1850 = load i32, ptr %86, align 4
  %1851 = sub nsw i32 %1850, 1
  %1852 = sext i32 %1851 to i64
  %1853 = getelementptr inbounds %struct.TValue, ptr %1849, i64 %1852
  br label %1861

1854:                                             ; preds = %1833
  %1855 = load ptr, ptr %84, align 8
  %1856 = getelementptr inbounds %struct.TValue, ptr %1855, i32 0, i32 0
  %1857 = load ptr, ptr %1856, align 8
  %1858 = load i32, ptr %86, align 4
  %1859 = sext i32 %1858 to i64
  %1860 = call ptr @luaH_getint(ptr noundef %1857, i64 noundef %1859)
  br label %1861

1861:                                             ; preds = %1854, %1844
  %1862 = phi ptr [ %1853, %1844 ], [ %1860, %1854 ]
  store ptr %1862, ptr %85, align 8
  %1863 = load ptr, ptr %85, align 8
  %1864 = getelementptr inbounds %struct.TValue, ptr %1863, i32 0, i32 1
  %1865 = load i8, ptr %1864, align 8
  %1866 = zext i8 %1865 to i32
  %1867 = and i32 %1866, 15
  %1868 = icmp eq i32 %1867, 0
  %1869 = xor i1 %1868, true
  br i1 %1869, label %1870, label %1916

1870:                                             ; preds = %1861, %1832
  %1871 = load ptr, ptr %85, align 8
  store ptr %1871, ptr %88, align 8
  %1872 = load ptr, ptr %87, align 8
  store ptr %1872, ptr %89, align 8
  %1873 = load ptr, ptr %88, align 8
  %1874 = getelementptr inbounds %struct.TValue, ptr %1873, i32 0, i32 0
  %1875 = load ptr, ptr %89, align 8
  %1876 = getelementptr inbounds %struct.TValue, ptr %1875, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %1874, ptr align 8 %1876, i64 8, i1 false)
  %1877 = load ptr, ptr %89, align 8
  %1878 = getelementptr inbounds %struct.TValue, ptr %1877, i32 0, i32 1
  %1879 = load i8, ptr %1878, align 8
  %1880 = load ptr, ptr %88, align 8
  %1881 = getelementptr inbounds %struct.TValue, ptr %1880, i32 0, i32 1
  store i8 %1879, ptr %1881, align 8
  %1882 = load ptr, ptr %3, align 8
  %1883 = load ptr, ptr %87, align 8
  %1884 = getelementptr inbounds %struct.TValue, ptr %1883, i32 0, i32 1
  %1885 = load i8, ptr %1884, align 8
  %1886 = zext i8 %1885 to i32
  %1887 = and i32 %1886, 64
  %1888 = icmp ne i32 %1887, 0
  br i1 %1888, label %1889, label %1914

1889:                                             ; preds = %1870
  %1890 = load ptr, ptr %84, align 8
  %1891 = getelementptr inbounds %struct.TValue, ptr %1890, i32 0, i32 0
  %1892 = load ptr, ptr %1891, align 8
  %1893 = getelementptr inbounds %struct.GCObject, ptr %1892, i32 0, i32 2
  %1894 = load i8, ptr %1893, align 1
  %1895 = zext i8 %1894 to i32
  %1896 = and i32 %1895, 32
  %1897 = icmp ne i32 %1896, 0
  br i1 %1897, label %1898, label %1912

1898:                                             ; preds = %1889
  %1899 = load ptr, ptr %87, align 8
  %1900 = getelementptr inbounds %struct.TValue, ptr %1899, i32 0, i32 0
  %1901 = load ptr, ptr %1900, align 8
  %1902 = getelementptr inbounds %struct.GCObject, ptr %1901, i32 0, i32 2
  %1903 = load i8, ptr %1902, align 1
  %1904 = zext i8 %1903 to i32
  %1905 = and i32 %1904, 24
  %1906 = icmp ne i32 %1905, 0
  br i1 %1906, label %1907, label %1912

1907:                                             ; preds = %1898
  %1908 = load ptr, ptr %3, align 8
  %1909 = load ptr, ptr %84, align 8
  %1910 = getelementptr inbounds %struct.TValue, ptr %1909, i32 0, i32 0
  %1911 = load ptr, ptr %1910, align 8
  call void @luaC_barrierback_(ptr noundef %1908, ptr noundef %1911)
  br label %1913

1912:                                             ; preds = %1898, %1889
  br label %1913

1913:                                             ; preds = %1912, %1907
  br label %1915

1914:                                             ; preds = %1870
  br label %1915

1915:                                             ; preds = %1914, %1913
  br label %1940

1916:                                             ; preds = %1861, %1832
  store ptr %90, ptr %91, align 8
  %1917 = load i32, ptr %86, align 4
  %1918 = sext i32 %1917 to i64
  %1919 = load ptr, ptr %91, align 8
  %1920 = getelementptr inbounds %struct.TValue, ptr %1919, i32 0, i32 0
  store i64 %1918, ptr %1920, align 8
  %1921 = load ptr, ptr %91, align 8
  %1922 = getelementptr inbounds %struct.TValue, ptr %1921, i32 0, i32 1
  store i8 3, ptr %1922, align 8
  %1923 = load ptr, ptr %8, align 8
  %1924 = load ptr, ptr %4, align 8
  %1925 = getelementptr inbounds %struct.CallInfo, ptr %1924, i32 0, i32 4
  %1926 = getelementptr inbounds %struct.anon, ptr %1925, i32 0, i32 0
  store ptr %1923, ptr %1926, align 8
  %1927 = load ptr, ptr %4, align 8
  %1928 = getelementptr inbounds %struct.CallInfo, ptr %1927, i32 0, i32 1
  %1929 = load ptr, ptr %1928, align 8
  %1930 = load ptr, ptr %3, align 8
  %1931 = getelementptr inbounds %struct.lua_State, ptr %1930, i32 0, i32 6
  store ptr %1929, ptr %1931, align 8
  %1932 = load ptr, ptr %3, align 8
  %1933 = load ptr, ptr %84, align 8
  %1934 = load ptr, ptr %87, align 8
  %1935 = load ptr, ptr %85, align 8
  call void @luaV_finishset(ptr noundef %1932, ptr noundef %1933, ptr noundef %90, ptr noundef %1934, ptr noundef %1935)
  %1936 = load ptr, ptr %4, align 8
  %1937 = getelementptr inbounds %struct.CallInfo, ptr %1936, i32 0, i32 4
  %1938 = getelementptr inbounds %struct.anon, ptr %1937, i32 0, i32 1
  %1939 = load volatile i32, ptr %1938, align 8
  store i32 %1939, ptr %9, align 4
  br label %1940

1940:                                             ; preds = %1916, %1915
  %1941 = load i32, ptr %9, align 4
  %1942 = icmp ne i32 %1941, 0
  %1943 = zext i1 %1942 to i32
  %1944 = sext i32 %1943 to i64
  %1945 = icmp ne i64 %1944, 0
  br i1 %1945, label %1946, label %1954

1946:                                             ; preds = %1940
  %1947 = load ptr, ptr %3, align 8
  %1948 = load ptr, ptr %8, align 8
  %1949 = call i32 @luaG_traceexec(ptr noundef %1947, ptr noundef %1948)
  store i32 %1949, ptr %9, align 4
  %1950 = load ptr, ptr %4, align 8
  %1951 = getelementptr inbounds %struct.CallInfo, ptr %1950, i32 0, i32 0
  %1952 = load ptr, ptr %1951, align 8
  %1953 = getelementptr inbounds %union.StackValue, ptr %1952, i64 1
  store ptr %1953, ptr %7, align 8
  br label %1954

1954:                                             ; preds = %1946, %1940
  %1955 = load ptr, ptr %8, align 8
  %1956 = getelementptr inbounds i32, ptr %1955, i32 1
  store ptr %1956, ptr %8, align 8
  %1957 = load i32, ptr %1955, align 4
  store i32 %1957, ptr %10, align 4
  %1958 = load i32, ptr %10, align 4
  %1959 = lshr i32 %1958, 0
  %1960 = and i32 %1959, 127
  %1961 = zext i32 %1960 to i64
  %1962 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %1961
  %1963 = load ptr, ptr %1962, align 8
  br label %7120

1964:                                             ; preds = %7120
  %1965 = load ptr, ptr %7, align 8
  %1966 = load i32, ptr %10, align 4
  %1967 = lshr i32 %1966, 7
  %1968 = and i32 %1967, 255
  %1969 = sext i32 %1968 to i64
  %1970 = getelementptr inbounds %union.StackValue, ptr %1965, i64 %1969
  store ptr %1970, ptr %92, align 8
  %1971 = load ptr, ptr %6, align 8
  %1972 = load i32, ptr %10, align 4
  %1973 = lshr i32 %1972, 16
  %1974 = and i32 %1973, 255
  %1975 = sext i32 %1974 to i64
  %1976 = getelementptr inbounds %struct.TValue, ptr %1971, i64 %1975
  store ptr %1976, ptr %94, align 8
  %1977 = load i32, ptr %10, align 4
  %1978 = and i32 %1977, 32768
  %1979 = icmp ne i32 %1978, 0
  br i1 %1979, label %1980, label %1987

1980:                                             ; preds = %1964
  %1981 = load ptr, ptr %6, align 8
  %1982 = load i32, ptr %10, align 4
  %1983 = lshr i32 %1982, 24
  %1984 = and i32 %1983, 255
  %1985 = sext i32 %1984 to i64
  %1986 = getelementptr inbounds %struct.TValue, ptr %1981, i64 %1985
  br label %1994

1987:                                             ; preds = %1964
  %1988 = load ptr, ptr %7, align 8
  %1989 = load i32, ptr %10, align 4
  %1990 = lshr i32 %1989, 24
  %1991 = and i32 %1990, 255
  %1992 = sext i32 %1991 to i64
  %1993 = getelementptr inbounds %union.StackValue, ptr %1988, i64 %1992
  br label %1994

1994:                                             ; preds = %1987, %1980
  %1995 = phi ptr [ %1986, %1980 ], [ %1993, %1987 ]
  store ptr %1995, ptr %95, align 8
  %1996 = load ptr, ptr %94, align 8
  %1997 = getelementptr inbounds %struct.TValue, ptr %1996, i32 0, i32 0
  %1998 = load ptr, ptr %1997, align 8
  store ptr %1998, ptr %96, align 8
  %1999 = load ptr, ptr %92, align 8
  %2000 = getelementptr inbounds %struct.TValue, ptr %1999, i32 0, i32 1
  %2001 = load i8, ptr %2000, align 8
  %2002 = zext i8 %2001 to i32
  %2003 = icmp eq i32 %2002, 69
  br i1 %2003, label %2005, label %2004

2004:                                             ; preds = %1994
  store ptr null, ptr %93, align 8
  br i1 false, label %2018, label %2064

2005:                                             ; preds = %1994
  %2006 = load ptr, ptr %92, align 8
  %2007 = getelementptr inbounds %struct.TValue, ptr %2006, i32 0, i32 0
  %2008 = load ptr, ptr %2007, align 8
  %2009 = load ptr, ptr %96, align 8
  %2010 = call ptr @luaH_getshortstr(ptr noundef %2008, ptr noundef %2009)
  store ptr %2010, ptr %93, align 8
  %2011 = load ptr, ptr %93, align 8
  %2012 = getelementptr inbounds %struct.TValue, ptr %2011, i32 0, i32 1
  %2013 = load i8, ptr %2012, align 8
  %2014 = zext i8 %2013 to i32
  %2015 = and i32 %2014, 15
  %2016 = icmp eq i32 %2015, 0
  %2017 = xor i1 %2016, true
  br i1 %2017, label %2018, label %2064

2018:                                             ; preds = %2005, %2004
  %2019 = load ptr, ptr %93, align 8
  store ptr %2019, ptr %97, align 8
  %2020 = load ptr, ptr %95, align 8
  store ptr %2020, ptr %98, align 8
  %2021 = load ptr, ptr %97, align 8
  %2022 = getelementptr inbounds %struct.TValue, ptr %2021, i32 0, i32 0
  %2023 = load ptr, ptr %98, align 8
  %2024 = getelementptr inbounds %struct.TValue, ptr %2023, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %2022, ptr align 8 %2024, i64 8, i1 false)
  %2025 = load ptr, ptr %98, align 8
  %2026 = getelementptr inbounds %struct.TValue, ptr %2025, i32 0, i32 1
  %2027 = load i8, ptr %2026, align 8
  %2028 = load ptr, ptr %97, align 8
  %2029 = getelementptr inbounds %struct.TValue, ptr %2028, i32 0, i32 1
  store i8 %2027, ptr %2029, align 8
  %2030 = load ptr, ptr %3, align 8
  %2031 = load ptr, ptr %95, align 8
  %2032 = getelementptr inbounds %struct.TValue, ptr %2031, i32 0, i32 1
  %2033 = load i8, ptr %2032, align 8
  %2034 = zext i8 %2033 to i32
  %2035 = and i32 %2034, 64
  %2036 = icmp ne i32 %2035, 0
  br i1 %2036, label %2037, label %2062

2037:                                             ; preds = %2018
  %2038 = load ptr, ptr %92, align 8
  %2039 = getelementptr inbounds %struct.TValue, ptr %2038, i32 0, i32 0
  %2040 = load ptr, ptr %2039, align 8
  %2041 = getelementptr inbounds %struct.GCObject, ptr %2040, i32 0, i32 2
  %2042 = load i8, ptr %2041, align 1
  %2043 = zext i8 %2042 to i32
  %2044 = and i32 %2043, 32
  %2045 = icmp ne i32 %2044, 0
  br i1 %2045, label %2046, label %2060

2046:                                             ; preds = %2037
  %2047 = load ptr, ptr %95, align 8
  %2048 = getelementptr inbounds %struct.TValue, ptr %2047, i32 0, i32 0
  %2049 = load ptr, ptr %2048, align 8
  %2050 = getelementptr inbounds %struct.GCObject, ptr %2049, i32 0, i32 2
  %2051 = load i8, ptr %2050, align 1
  %2052 = zext i8 %2051 to i32
  %2053 = and i32 %2052, 24
  %2054 = icmp ne i32 %2053, 0
  br i1 %2054, label %2055, label %2060

2055:                                             ; preds = %2046
  %2056 = load ptr, ptr %3, align 8
  %2057 = load ptr, ptr %92, align 8
  %2058 = getelementptr inbounds %struct.TValue, ptr %2057, i32 0, i32 0
  %2059 = load ptr, ptr %2058, align 8
  call void @luaC_barrierback_(ptr noundef %2056, ptr noundef %2059)
  br label %2061

2060:                                             ; preds = %2046, %2037
  br label %2061

2061:                                             ; preds = %2060, %2055
  br label %2063

2062:                                             ; preds = %2018
  br label %2063

2063:                                             ; preds = %2062, %2061
  br label %2083

2064:                                             ; preds = %2005, %2004
  %2065 = load ptr, ptr %8, align 8
  %2066 = load ptr, ptr %4, align 8
  %2067 = getelementptr inbounds %struct.CallInfo, ptr %2066, i32 0, i32 4
  %2068 = getelementptr inbounds %struct.anon, ptr %2067, i32 0, i32 0
  store ptr %2065, ptr %2068, align 8
  %2069 = load ptr, ptr %4, align 8
  %2070 = getelementptr inbounds %struct.CallInfo, ptr %2069, i32 0, i32 1
  %2071 = load ptr, ptr %2070, align 8
  %2072 = load ptr, ptr %3, align 8
  %2073 = getelementptr inbounds %struct.lua_State, ptr %2072, i32 0, i32 6
  store ptr %2071, ptr %2073, align 8
  %2074 = load ptr, ptr %3, align 8
  %2075 = load ptr, ptr %92, align 8
  %2076 = load ptr, ptr %94, align 8
  %2077 = load ptr, ptr %95, align 8
  %2078 = load ptr, ptr %93, align 8
  call void @luaV_finishset(ptr noundef %2074, ptr noundef %2075, ptr noundef %2076, ptr noundef %2077, ptr noundef %2078)
  %2079 = load ptr, ptr %4, align 8
  %2080 = getelementptr inbounds %struct.CallInfo, ptr %2079, i32 0, i32 4
  %2081 = getelementptr inbounds %struct.anon, ptr %2080, i32 0, i32 1
  %2082 = load volatile i32, ptr %2081, align 8
  store i32 %2082, ptr %9, align 4
  br label %2083

2083:                                             ; preds = %2064, %2063
  %2084 = load i32, ptr %9, align 4
  %2085 = icmp ne i32 %2084, 0
  %2086 = zext i1 %2085 to i32
  %2087 = sext i32 %2086 to i64
  %2088 = icmp ne i64 %2087, 0
  br i1 %2088, label %2089, label %2097

2089:                                             ; preds = %2083
  %2090 = load ptr, ptr %3, align 8
  %2091 = load ptr, ptr %8, align 8
  %2092 = call i32 @luaG_traceexec(ptr noundef %2090, ptr noundef %2091)
  store i32 %2092, ptr %9, align 4
  %2093 = load ptr, ptr %4, align 8
  %2094 = getelementptr inbounds %struct.CallInfo, ptr %2093, i32 0, i32 0
  %2095 = load ptr, ptr %2094, align 8
  %2096 = getelementptr inbounds %union.StackValue, ptr %2095, i64 1
  store ptr %2096, ptr %7, align 8
  br label %2097

2097:                                             ; preds = %2089, %2083
  %2098 = load ptr, ptr %8, align 8
  %2099 = getelementptr inbounds i32, ptr %2098, i32 1
  store ptr %2099, ptr %8, align 8
  %2100 = load i32, ptr %2098, align 4
  store i32 %2100, ptr %10, align 4
  %2101 = load i32, ptr %10, align 4
  %2102 = lshr i32 %2101, 0
  %2103 = and i32 %2102, 127
  %2104 = zext i32 %2103 to i64
  %2105 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2104
  %2106 = load ptr, ptr %2105, align 8
  br label %7120

2107:                                             ; preds = %7120
  %2108 = load ptr, ptr %7, align 8
  %2109 = load i32, ptr %10, align 4
  %2110 = lshr i32 %2109, 7
  %2111 = and i32 %2110, 255
  %2112 = sext i32 %2111 to i64
  %2113 = getelementptr inbounds %union.StackValue, ptr %2108, i64 %2112
  store ptr %2113, ptr %99, align 8
  %2114 = load i32, ptr %10, align 4
  %2115 = lshr i32 %2114, 16
  %2116 = and i32 %2115, 255
  store i32 %2116, ptr %100, align 4
  %2117 = load i32, ptr %10, align 4
  %2118 = lshr i32 %2117, 24
  %2119 = and i32 %2118, 255
  store i32 %2119, ptr %101, align 4
  %2120 = load i32, ptr %100, align 4
  %2121 = icmp sgt i32 %2120, 0
  br i1 %2121, label %2122, label %2126

2122:                                             ; preds = %2107
  %2123 = load i32, ptr %100, align 4
  %2124 = sub nsw i32 %2123, 1
  %2125 = shl i32 1, %2124
  store i32 %2125, ptr %100, align 4
  br label %2126

2126:                                             ; preds = %2122, %2107
  %2127 = load i32, ptr %10, align 4
  %2128 = and i32 %2127, 32768
  %2129 = icmp ne i32 %2128, 0
  br i1 %2129, label %2130, label %2138

2130:                                             ; preds = %2126
  %2131 = load ptr, ptr %8, align 8
  %2132 = load i32, ptr %2131, align 4
  %2133 = lshr i32 %2132, 7
  %2134 = and i32 %2133, 33554431
  %2135 = mul nsw i32 %2134, 256
  %2136 = load i32, ptr %101, align 4
  %2137 = add nsw i32 %2136, %2135
  store i32 %2137, ptr %101, align 4
  br label %2138

2138:                                             ; preds = %2130, %2126
  %2139 = load ptr, ptr %8, align 8
  %2140 = getelementptr inbounds i32, ptr %2139, i32 1
  store ptr %2140, ptr %8, align 8
  %2141 = load ptr, ptr %99, align 8
  %2142 = getelementptr inbounds %union.StackValue, ptr %2141, i64 1
  %2143 = load ptr, ptr %3, align 8
  %2144 = getelementptr inbounds %struct.lua_State, ptr %2143, i32 0, i32 6
  store ptr %2142, ptr %2144, align 8
  %2145 = load ptr, ptr %3, align 8
  %2146 = call ptr @luaH_new(ptr noundef %2145)
  store ptr %2146, ptr %102, align 8
  %2147 = load ptr, ptr %99, align 8
  store ptr %2147, ptr %103, align 8
  %2148 = load ptr, ptr %102, align 8
  store ptr %2148, ptr %104, align 8
  %2149 = load ptr, ptr %104, align 8
  %2150 = load ptr, ptr %103, align 8
  %2151 = getelementptr inbounds %struct.TValue, ptr %2150, i32 0, i32 0
  store ptr %2149, ptr %2151, align 8
  %2152 = load ptr, ptr %103, align 8
  %2153 = getelementptr inbounds %struct.TValue, ptr %2152, i32 0, i32 1
  store i8 69, ptr %2153, align 8
  %2154 = load ptr, ptr %3, align 8
  %2155 = load i32, ptr %100, align 4
  %2156 = icmp ne i32 %2155, 0
  br i1 %2156, label %2160, label %2157

2157:                                             ; preds = %2138
  %2158 = load i32, ptr %101, align 4
  %2159 = icmp ne i32 %2158, 0
  br i1 %2159, label %2160, label %2165

2160:                                             ; preds = %2157, %2138
  %2161 = load ptr, ptr %3, align 8
  %2162 = load ptr, ptr %102, align 8
  %2163 = load i32, ptr %101, align 4
  %2164 = load i32, ptr %100, align 4
  call void @luaH_resize(ptr noundef %2161, ptr noundef %2162, i32 noundef %2163, i32 noundef %2164)
  br label %2165

2165:                                             ; preds = %2160, %2157
  %2166 = load ptr, ptr %3, align 8
  %2167 = getelementptr inbounds %struct.lua_State, ptr %2166, i32 0, i32 7
  %2168 = load ptr, ptr %2167, align 8
  %2169 = getelementptr inbounds %struct.global_State, ptr %2168, i32 0, i32 3
  %2170 = load i64, ptr %2169, align 8
  %2171 = icmp sgt i64 %2170, 0
  br i1 %2171, label %2172, label %2186

2172:                                             ; preds = %2165
  %2173 = load ptr, ptr %8, align 8
  %2174 = load ptr, ptr %4, align 8
  %2175 = getelementptr inbounds %struct.CallInfo, ptr %2174, i32 0, i32 4
  %2176 = getelementptr inbounds %struct.anon, ptr %2175, i32 0, i32 0
  store ptr %2173, ptr %2176, align 8
  %2177 = load ptr, ptr %99, align 8
  %2178 = getelementptr inbounds %union.StackValue, ptr %2177, i64 1
  %2179 = load ptr, ptr %3, align 8
  %2180 = getelementptr inbounds %struct.lua_State, ptr %2179, i32 0, i32 6
  store ptr %2178, ptr %2180, align 8
  %2181 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %2181)
  %2182 = load ptr, ptr %4, align 8
  %2183 = getelementptr inbounds %struct.CallInfo, ptr %2182, i32 0, i32 4
  %2184 = getelementptr inbounds %struct.anon, ptr %2183, i32 0, i32 1
  %2185 = load volatile i32, ptr %2184, align 8
  store i32 %2185, ptr %9, align 4
  br label %2186

2186:                                             ; preds = %2172, %2165
  %2187 = load i32, ptr %9, align 4
  %2188 = icmp ne i32 %2187, 0
  %2189 = zext i1 %2188 to i32
  %2190 = sext i32 %2189 to i64
  %2191 = icmp ne i64 %2190, 0
  br i1 %2191, label %2192, label %2200

2192:                                             ; preds = %2186
  %2193 = load ptr, ptr %3, align 8
  %2194 = load ptr, ptr %8, align 8
  %2195 = call i32 @luaG_traceexec(ptr noundef %2193, ptr noundef %2194)
  store i32 %2195, ptr %9, align 4
  %2196 = load ptr, ptr %4, align 8
  %2197 = getelementptr inbounds %struct.CallInfo, ptr %2196, i32 0, i32 0
  %2198 = load ptr, ptr %2197, align 8
  %2199 = getelementptr inbounds %union.StackValue, ptr %2198, i64 1
  store ptr %2199, ptr %7, align 8
  br label %2200

2200:                                             ; preds = %2192, %2186
  %2201 = load ptr, ptr %8, align 8
  %2202 = getelementptr inbounds i32, ptr %2201, i32 1
  store ptr %2202, ptr %8, align 8
  %2203 = load i32, ptr %2201, align 4
  store i32 %2203, ptr %10, align 4
  %2204 = load i32, ptr %10, align 4
  %2205 = lshr i32 %2204, 0
  %2206 = and i32 %2205, 127
  %2207 = zext i32 %2206 to i64
  %2208 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2207
  %2209 = load ptr, ptr %2208, align 8
  br label %7120

2210:                                             ; preds = %7120
  %2211 = load ptr, ptr %7, align 8
  %2212 = load i32, ptr %10, align 4
  %2213 = lshr i32 %2212, 7
  %2214 = and i32 %2213, 255
  %2215 = sext i32 %2214 to i64
  %2216 = getelementptr inbounds %union.StackValue, ptr %2211, i64 %2215
  store ptr %2216, ptr %105, align 8
  %2217 = load ptr, ptr %7, align 8
  %2218 = load i32, ptr %10, align 4
  %2219 = lshr i32 %2218, 16
  %2220 = and i32 %2219, 255
  %2221 = sext i32 %2220 to i64
  %2222 = getelementptr inbounds %union.StackValue, ptr %2217, i64 %2221
  store ptr %2222, ptr %107, align 8
  %2223 = load i32, ptr %10, align 4
  %2224 = and i32 %2223, 32768
  %2225 = icmp ne i32 %2224, 0
  br i1 %2225, label %2226, label %2233

2226:                                             ; preds = %2210
  %2227 = load ptr, ptr %6, align 8
  %2228 = load i32, ptr %10, align 4
  %2229 = lshr i32 %2228, 24
  %2230 = and i32 %2229, 255
  %2231 = sext i32 %2230 to i64
  %2232 = getelementptr inbounds %struct.TValue, ptr %2227, i64 %2231
  br label %2240

2233:                                             ; preds = %2210
  %2234 = load ptr, ptr %7, align 8
  %2235 = load i32, ptr %10, align 4
  %2236 = lshr i32 %2235, 24
  %2237 = and i32 %2236, 255
  %2238 = sext i32 %2237 to i64
  %2239 = getelementptr inbounds %union.StackValue, ptr %2234, i64 %2238
  br label %2240

2240:                                             ; preds = %2233, %2226
  %2241 = phi ptr [ %2232, %2226 ], [ %2239, %2233 ]
  store ptr %2241, ptr %108, align 8
  %2242 = load ptr, ptr %108, align 8
  %2243 = getelementptr inbounds %struct.TValue, ptr %2242, i32 0, i32 0
  %2244 = load ptr, ptr %2243, align 8
  store ptr %2244, ptr %109, align 8
  %2245 = load ptr, ptr %105, align 8
  %2246 = getelementptr inbounds %union.StackValue, ptr %2245, i64 1
  store ptr %2246, ptr %110, align 8
  %2247 = load ptr, ptr %107, align 8
  store ptr %2247, ptr %111, align 8
  %2248 = load ptr, ptr %110, align 8
  %2249 = getelementptr inbounds %struct.TValue, ptr %2248, i32 0, i32 0
  %2250 = load ptr, ptr %111, align 8
  %2251 = getelementptr inbounds %struct.TValue, ptr %2250, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %2249, ptr align 8 %2251, i64 8, i1 false)
  %2252 = load ptr, ptr %111, align 8
  %2253 = getelementptr inbounds %struct.TValue, ptr %2252, i32 0, i32 1
  %2254 = load i8, ptr %2253, align 8
  %2255 = load ptr, ptr %110, align 8
  %2256 = getelementptr inbounds %struct.TValue, ptr %2255, i32 0, i32 1
  store i8 %2254, ptr %2256, align 8
  %2257 = load ptr, ptr %3, align 8
  %2258 = load ptr, ptr %107, align 8
  %2259 = getelementptr inbounds %struct.TValue, ptr %2258, i32 0, i32 1
  %2260 = load i8, ptr %2259, align 8
  %2261 = zext i8 %2260 to i32
  %2262 = icmp eq i32 %2261, 69
  br i1 %2262, label %2264, label %2263

2263:                                             ; preds = %2240
  store ptr null, ptr %106, align 8
  br i1 false, label %2277, label %2290

2264:                                             ; preds = %2240
  %2265 = load ptr, ptr %107, align 8
  %2266 = getelementptr inbounds %struct.TValue, ptr %2265, i32 0, i32 0
  %2267 = load ptr, ptr %2266, align 8
  %2268 = load ptr, ptr %109, align 8
  %2269 = call ptr @luaH_getstr(ptr noundef %2267, ptr noundef %2268)
  store ptr %2269, ptr %106, align 8
  %2270 = load ptr, ptr %106, align 8
  %2271 = getelementptr inbounds %struct.TValue, ptr %2270, i32 0, i32 1
  %2272 = load i8, ptr %2271, align 8
  %2273 = zext i8 %2272 to i32
  %2274 = and i32 %2273, 15
  %2275 = icmp eq i32 %2274, 0
  %2276 = xor i1 %2275, true
  br i1 %2276, label %2277, label %2290

2277:                                             ; preds = %2264, %2263
  %2278 = load ptr, ptr %105, align 8
  store ptr %2278, ptr %112, align 8
  %2279 = load ptr, ptr %106, align 8
  store ptr %2279, ptr %113, align 8
  %2280 = load ptr, ptr %112, align 8
  %2281 = getelementptr inbounds %struct.TValue, ptr %2280, i32 0, i32 0
  %2282 = load ptr, ptr %113, align 8
  %2283 = getelementptr inbounds %struct.TValue, ptr %2282, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %2281, ptr align 8 %2283, i64 8, i1 false)
  %2284 = load ptr, ptr %113, align 8
  %2285 = getelementptr inbounds %struct.TValue, ptr %2284, i32 0, i32 1
  %2286 = load i8, ptr %2285, align 8
  %2287 = load ptr, ptr %112, align 8
  %2288 = getelementptr inbounds %struct.TValue, ptr %2287, i32 0, i32 1
  store i8 %2286, ptr %2288, align 8
  %2289 = load ptr, ptr %3, align 8
  br label %2309

2290:                                             ; preds = %2264, %2263
  %2291 = load ptr, ptr %8, align 8
  %2292 = load ptr, ptr %4, align 8
  %2293 = getelementptr inbounds %struct.CallInfo, ptr %2292, i32 0, i32 4
  %2294 = getelementptr inbounds %struct.anon, ptr %2293, i32 0, i32 0
  store ptr %2291, ptr %2294, align 8
  %2295 = load ptr, ptr %4, align 8
  %2296 = getelementptr inbounds %struct.CallInfo, ptr %2295, i32 0, i32 1
  %2297 = load ptr, ptr %2296, align 8
  %2298 = load ptr, ptr %3, align 8
  %2299 = getelementptr inbounds %struct.lua_State, ptr %2298, i32 0, i32 6
  store ptr %2297, ptr %2299, align 8
  %2300 = load ptr, ptr %3, align 8
  %2301 = load ptr, ptr %107, align 8
  %2302 = load ptr, ptr %108, align 8
  %2303 = load ptr, ptr %105, align 8
  %2304 = load ptr, ptr %106, align 8
  call void @luaV_finishget(ptr noundef %2300, ptr noundef %2301, ptr noundef %2302, ptr noundef %2303, ptr noundef %2304)
  %2305 = load ptr, ptr %4, align 8
  %2306 = getelementptr inbounds %struct.CallInfo, ptr %2305, i32 0, i32 4
  %2307 = getelementptr inbounds %struct.anon, ptr %2306, i32 0, i32 1
  %2308 = load volatile i32, ptr %2307, align 8
  store i32 %2308, ptr %9, align 4
  br label %2309

2309:                                             ; preds = %2290, %2277
  %2310 = load i32, ptr %9, align 4
  %2311 = icmp ne i32 %2310, 0
  %2312 = zext i1 %2311 to i32
  %2313 = sext i32 %2312 to i64
  %2314 = icmp ne i64 %2313, 0
  br i1 %2314, label %2315, label %2323

2315:                                             ; preds = %2309
  %2316 = load ptr, ptr %3, align 8
  %2317 = load ptr, ptr %8, align 8
  %2318 = call i32 @luaG_traceexec(ptr noundef %2316, ptr noundef %2317)
  store i32 %2318, ptr %9, align 4
  %2319 = load ptr, ptr %4, align 8
  %2320 = getelementptr inbounds %struct.CallInfo, ptr %2319, i32 0, i32 0
  %2321 = load ptr, ptr %2320, align 8
  %2322 = getelementptr inbounds %union.StackValue, ptr %2321, i64 1
  store ptr %2322, ptr %7, align 8
  br label %2323

2323:                                             ; preds = %2315, %2309
  %2324 = load ptr, ptr %8, align 8
  %2325 = getelementptr inbounds i32, ptr %2324, i32 1
  store ptr %2325, ptr %8, align 8
  %2326 = load i32, ptr %2324, align 4
  store i32 %2326, ptr %10, align 4
  %2327 = load i32, ptr %10, align 4
  %2328 = lshr i32 %2327, 0
  %2329 = and i32 %2328, 127
  %2330 = zext i32 %2329 to i64
  %2331 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2330
  %2332 = load ptr, ptr %2331, align 8
  br label %7120

2333:                                             ; preds = %7120
  %2334 = load ptr, ptr %7, align 8
  %2335 = load i32, ptr %10, align 4
  %2336 = lshr i32 %2335, 7
  %2337 = and i32 %2336, 255
  %2338 = sext i32 %2337 to i64
  %2339 = getelementptr inbounds %union.StackValue, ptr %2334, i64 %2338
  store ptr %2339, ptr %114, align 8
  %2340 = load ptr, ptr %7, align 8
  %2341 = load i32, ptr %10, align 4
  %2342 = lshr i32 %2341, 16
  %2343 = and i32 %2342, 255
  %2344 = sext i32 %2343 to i64
  %2345 = getelementptr inbounds %union.StackValue, ptr %2340, i64 %2344
  store ptr %2345, ptr %115, align 8
  %2346 = load i32, ptr %10, align 4
  %2347 = lshr i32 %2346, 24
  %2348 = and i32 %2347, 255
  %2349 = sub nsw i32 %2348, 127
  store i32 %2349, ptr %116, align 4
  %2350 = load ptr, ptr %115, align 8
  %2351 = getelementptr inbounds %struct.TValue, ptr %2350, i32 0, i32 1
  %2352 = load i8, ptr %2351, align 8
  %2353 = zext i8 %2352 to i32
  %2354 = icmp eq i32 %2353, 3
  br i1 %2354, label %2355, label %2370

2355:                                             ; preds = %2333
  %2356 = load ptr, ptr %115, align 8
  %2357 = getelementptr inbounds %struct.TValue, ptr %2356, i32 0, i32 0
  %2358 = load i64, ptr %2357, align 8
  store i64 %2358, ptr %117, align 8
  %2359 = load ptr, ptr %8, align 8
  %2360 = getelementptr inbounds i32, ptr %2359, i32 1
  store ptr %2360, ptr %8, align 8
  %2361 = load ptr, ptr %114, align 8
  store ptr %2361, ptr %118, align 8
  %2362 = load i64, ptr %117, align 8
  %2363 = load i32, ptr %116, align 4
  %2364 = sext i32 %2363 to i64
  %2365 = add i64 %2362, %2364
  %2366 = load ptr, ptr %118, align 8
  %2367 = getelementptr inbounds %struct.TValue, ptr %2366, i32 0, i32 0
  store i64 %2365, ptr %2367, align 8
  %2368 = load ptr, ptr %118, align 8
  %2369 = getelementptr inbounds %struct.TValue, ptr %2368, i32 0, i32 1
  store i8 3, ptr %2369, align 8
  br label %2393

2370:                                             ; preds = %2333
  %2371 = load ptr, ptr %115, align 8
  %2372 = getelementptr inbounds %struct.TValue, ptr %2371, i32 0, i32 1
  %2373 = load i8, ptr %2372, align 8
  %2374 = zext i8 %2373 to i32
  %2375 = icmp eq i32 %2374, 19
  br i1 %2375, label %2376, label %2392

2376:                                             ; preds = %2370
  %2377 = load ptr, ptr %115, align 8
  %2378 = getelementptr inbounds %struct.TValue, ptr %2377, i32 0, i32 0
  %2379 = load double, ptr %2378, align 8
  store double %2379, ptr %119, align 8
  %2380 = load i32, ptr %116, align 4
  %2381 = sitofp i32 %2380 to double
  store double %2381, ptr %120, align 8
  %2382 = load ptr, ptr %8, align 8
  %2383 = getelementptr inbounds i32, ptr %2382, i32 1
  store ptr %2383, ptr %8, align 8
  %2384 = load ptr, ptr %114, align 8
  store ptr %2384, ptr %121, align 8
  %2385 = load double, ptr %119, align 8
  %2386 = load double, ptr %120, align 8
  %2387 = fadd double %2385, %2386
  %2388 = load ptr, ptr %121, align 8
  %2389 = getelementptr inbounds %struct.TValue, ptr %2388, i32 0, i32 0
  store double %2387, ptr %2389, align 8
  %2390 = load ptr, ptr %121, align 8
  %2391 = getelementptr inbounds %struct.TValue, ptr %2390, i32 0, i32 1
  store i8 19, ptr %2391, align 8
  br label %2392

2392:                                             ; preds = %2376, %2370
  br label %2393

2393:                                             ; preds = %2392, %2355
  %2394 = load i32, ptr %9, align 4
  %2395 = icmp ne i32 %2394, 0
  %2396 = zext i1 %2395 to i32
  %2397 = sext i32 %2396 to i64
  %2398 = icmp ne i64 %2397, 0
  br i1 %2398, label %2399, label %2407

2399:                                             ; preds = %2393
  %2400 = load ptr, ptr %3, align 8
  %2401 = load ptr, ptr %8, align 8
  %2402 = call i32 @luaG_traceexec(ptr noundef %2400, ptr noundef %2401)
  store i32 %2402, ptr %9, align 4
  %2403 = load ptr, ptr %4, align 8
  %2404 = getelementptr inbounds %struct.CallInfo, ptr %2403, i32 0, i32 0
  %2405 = load ptr, ptr %2404, align 8
  %2406 = getelementptr inbounds %union.StackValue, ptr %2405, i64 1
  store ptr %2406, ptr %7, align 8
  br label %2407

2407:                                             ; preds = %2399, %2393
  %2408 = load ptr, ptr %8, align 8
  %2409 = getelementptr inbounds i32, ptr %2408, i32 1
  store ptr %2409, ptr %8, align 8
  %2410 = load i32, ptr %2408, align 4
  store i32 %2410, ptr %10, align 4
  %2411 = load i32, ptr %10, align 4
  %2412 = lshr i32 %2411, 0
  %2413 = and i32 %2412, 127
  %2414 = zext i32 %2413 to i64
  %2415 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2414
  %2416 = load ptr, ptr %2415, align 8
  br label %7120

2417:                                             ; preds = %7120
  %2418 = load ptr, ptr %7, align 8
  %2419 = load i32, ptr %10, align 4
  %2420 = lshr i32 %2419, 16
  %2421 = and i32 %2420, 255
  %2422 = sext i32 %2421 to i64
  %2423 = getelementptr inbounds %union.StackValue, ptr %2418, i64 %2422
  store ptr %2423, ptr %122, align 8
  %2424 = load ptr, ptr %6, align 8
  %2425 = load i32, ptr %10, align 4
  %2426 = lshr i32 %2425, 24
  %2427 = and i32 %2426, 255
  %2428 = sext i32 %2427 to i64
  %2429 = getelementptr inbounds %struct.TValue, ptr %2424, i64 %2428
  store ptr %2429, ptr %123, align 8
  %2430 = load ptr, ptr %7, align 8
  %2431 = load i32, ptr %10, align 4
  %2432 = lshr i32 %2431, 7
  %2433 = and i32 %2432, 255
  %2434 = sext i32 %2433 to i64
  %2435 = getelementptr inbounds %union.StackValue, ptr %2430, i64 %2434
  store ptr %2435, ptr %124, align 8
  %2436 = load ptr, ptr %122, align 8
  %2437 = getelementptr inbounds %struct.TValue, ptr %2436, i32 0, i32 1
  %2438 = load i8, ptr %2437, align 8
  %2439 = zext i8 %2438 to i32
  %2440 = icmp eq i32 %2439, 3
  br i1 %2440, label %2441, label %2464

2441:                                             ; preds = %2417
  %2442 = load ptr, ptr %123, align 8
  %2443 = getelementptr inbounds %struct.TValue, ptr %2442, i32 0, i32 1
  %2444 = load i8, ptr %2443, align 8
  %2445 = zext i8 %2444 to i32
  %2446 = icmp eq i32 %2445, 3
  br i1 %2446, label %2447, label %2464

2447:                                             ; preds = %2441
  %2448 = load ptr, ptr %122, align 8
  %2449 = getelementptr inbounds %struct.TValue, ptr %2448, i32 0, i32 0
  %2450 = load i64, ptr %2449, align 8
  store i64 %2450, ptr %125, align 8
  %2451 = load ptr, ptr %123, align 8
  %2452 = getelementptr inbounds %struct.TValue, ptr %2451, i32 0, i32 0
  %2453 = load i64, ptr %2452, align 8
  store i64 %2453, ptr %126, align 8
  %2454 = load ptr, ptr %8, align 8
  %2455 = getelementptr inbounds i32, ptr %2454, i32 1
  store ptr %2455, ptr %8, align 8
  %2456 = load ptr, ptr %124, align 8
  store ptr %2456, ptr %127, align 8
  %2457 = load i64, ptr %125, align 8
  %2458 = load i64, ptr %126, align 8
  %2459 = add i64 %2457, %2458
  %2460 = load ptr, ptr %127, align 8
  %2461 = getelementptr inbounds %struct.TValue, ptr %2460, i32 0, i32 0
  store i64 %2459, ptr %2461, align 8
  %2462 = load ptr, ptr %127, align 8
  %2463 = getelementptr inbounds %struct.TValue, ptr %2462, i32 0, i32 1
  store i8 3, ptr %2463, align 8
  br label %2520

2464:                                             ; preds = %2441, %2417
  %2465 = load ptr, ptr %122, align 8
  %2466 = getelementptr inbounds %struct.TValue, ptr %2465, i32 0, i32 1
  %2467 = load i8, ptr %2466, align 8
  %2468 = zext i8 %2467 to i32
  %2469 = icmp eq i32 %2468, 19
  br i1 %2469, label %2470, label %2474

2470:                                             ; preds = %2464
  %2471 = load ptr, ptr %122, align 8
  %2472 = getelementptr inbounds %struct.TValue, ptr %2471, i32 0, i32 0
  %2473 = load double, ptr %2472, align 8
  store double %2473, ptr %128, align 8
  br i1 true, label %2486, label %2519

2474:                                             ; preds = %2464
  %2475 = load ptr, ptr %122, align 8
  %2476 = getelementptr inbounds %struct.TValue, ptr %2475, i32 0, i32 1
  %2477 = load i8, ptr %2476, align 8
  %2478 = zext i8 %2477 to i32
  %2479 = icmp eq i32 %2478, 3
  br i1 %2479, label %2480, label %2485

2480:                                             ; preds = %2474
  %2481 = load ptr, ptr %122, align 8
  %2482 = getelementptr inbounds %struct.TValue, ptr %2481, i32 0, i32 0
  %2483 = load i64, ptr %2482, align 8
  %2484 = sitofp i64 %2483 to double
  store double %2484, ptr %128, align 8
  br i1 true, label %2486, label %2519

2485:                                             ; preds = %2474
  br i1 false, label %2486, label %2519

2486:                                             ; preds = %2485, %2480, %2470
  %2487 = load ptr, ptr %123, align 8
  %2488 = getelementptr inbounds %struct.TValue, ptr %2487, i32 0, i32 1
  %2489 = load i8, ptr %2488, align 8
  %2490 = zext i8 %2489 to i32
  %2491 = icmp eq i32 %2490, 19
  br i1 %2491, label %2492, label %2496

2492:                                             ; preds = %2486
  %2493 = load ptr, ptr %123, align 8
  %2494 = getelementptr inbounds %struct.TValue, ptr %2493, i32 0, i32 0
  %2495 = load double, ptr %2494, align 8
  store double %2495, ptr %129, align 8
  br i1 true, label %2508, label %2519

2496:                                             ; preds = %2486
  %2497 = load ptr, ptr %123, align 8
  %2498 = getelementptr inbounds %struct.TValue, ptr %2497, i32 0, i32 1
  %2499 = load i8, ptr %2498, align 8
  %2500 = zext i8 %2499 to i32
  %2501 = icmp eq i32 %2500, 3
  br i1 %2501, label %2502, label %2507

2502:                                             ; preds = %2496
  %2503 = load ptr, ptr %123, align 8
  %2504 = getelementptr inbounds %struct.TValue, ptr %2503, i32 0, i32 0
  %2505 = load i64, ptr %2504, align 8
  %2506 = sitofp i64 %2505 to double
  store double %2506, ptr %129, align 8
  br i1 true, label %2508, label %2519

2507:                                             ; preds = %2496
  br i1 false, label %2508, label %2519

2508:                                             ; preds = %2507, %2502, %2492
  %2509 = load ptr, ptr %8, align 8
  %2510 = getelementptr inbounds i32, ptr %2509, i32 1
  store ptr %2510, ptr %8, align 8
  %2511 = load ptr, ptr %124, align 8
  store ptr %2511, ptr %130, align 8
  %2512 = load double, ptr %128, align 8
  %2513 = load double, ptr %129, align 8
  %2514 = fadd double %2512, %2513
  %2515 = load ptr, ptr %130, align 8
  %2516 = getelementptr inbounds %struct.TValue, ptr %2515, i32 0, i32 0
  store double %2514, ptr %2516, align 8
  %2517 = load ptr, ptr %130, align 8
  %2518 = getelementptr inbounds %struct.TValue, ptr %2517, i32 0, i32 1
  store i8 19, ptr %2518, align 8
  br label %2519

2519:                                             ; preds = %2508, %2507, %2502, %2492, %2485, %2480, %2470
  br label %2520

2520:                                             ; preds = %2519, %2447
  %2521 = load i32, ptr %9, align 4
  %2522 = icmp ne i32 %2521, 0
  %2523 = zext i1 %2522 to i32
  %2524 = sext i32 %2523 to i64
  %2525 = icmp ne i64 %2524, 0
  br i1 %2525, label %2526, label %2534

2526:                                             ; preds = %2520
  %2527 = load ptr, ptr %3, align 8
  %2528 = load ptr, ptr %8, align 8
  %2529 = call i32 @luaG_traceexec(ptr noundef %2527, ptr noundef %2528)
  store i32 %2529, ptr %9, align 4
  %2530 = load ptr, ptr %4, align 8
  %2531 = getelementptr inbounds %struct.CallInfo, ptr %2530, i32 0, i32 0
  %2532 = load ptr, ptr %2531, align 8
  %2533 = getelementptr inbounds %union.StackValue, ptr %2532, i64 1
  store ptr %2533, ptr %7, align 8
  br label %2534

2534:                                             ; preds = %2526, %2520
  %2535 = load ptr, ptr %8, align 8
  %2536 = getelementptr inbounds i32, ptr %2535, i32 1
  store ptr %2536, ptr %8, align 8
  %2537 = load i32, ptr %2535, align 4
  store i32 %2537, ptr %10, align 4
  %2538 = load i32, ptr %10, align 4
  %2539 = lshr i32 %2538, 0
  %2540 = and i32 %2539, 127
  %2541 = zext i32 %2540 to i64
  %2542 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2541
  %2543 = load ptr, ptr %2542, align 8
  br label %7120

2544:                                             ; preds = %7120
  %2545 = load ptr, ptr %7, align 8
  %2546 = load i32, ptr %10, align 4
  %2547 = lshr i32 %2546, 16
  %2548 = and i32 %2547, 255
  %2549 = sext i32 %2548 to i64
  %2550 = getelementptr inbounds %union.StackValue, ptr %2545, i64 %2549
  store ptr %2550, ptr %131, align 8
  %2551 = load ptr, ptr %6, align 8
  %2552 = load i32, ptr %10, align 4
  %2553 = lshr i32 %2552, 24
  %2554 = and i32 %2553, 255
  %2555 = sext i32 %2554 to i64
  %2556 = getelementptr inbounds %struct.TValue, ptr %2551, i64 %2555
  store ptr %2556, ptr %132, align 8
  %2557 = load ptr, ptr %7, align 8
  %2558 = load i32, ptr %10, align 4
  %2559 = lshr i32 %2558, 7
  %2560 = and i32 %2559, 255
  %2561 = sext i32 %2560 to i64
  %2562 = getelementptr inbounds %union.StackValue, ptr %2557, i64 %2561
  store ptr %2562, ptr %133, align 8
  %2563 = load ptr, ptr %131, align 8
  %2564 = getelementptr inbounds %struct.TValue, ptr %2563, i32 0, i32 1
  %2565 = load i8, ptr %2564, align 8
  %2566 = zext i8 %2565 to i32
  %2567 = icmp eq i32 %2566, 3
  br i1 %2567, label %2568, label %2591

2568:                                             ; preds = %2544
  %2569 = load ptr, ptr %132, align 8
  %2570 = getelementptr inbounds %struct.TValue, ptr %2569, i32 0, i32 1
  %2571 = load i8, ptr %2570, align 8
  %2572 = zext i8 %2571 to i32
  %2573 = icmp eq i32 %2572, 3
  br i1 %2573, label %2574, label %2591

2574:                                             ; preds = %2568
  %2575 = load ptr, ptr %131, align 8
  %2576 = getelementptr inbounds %struct.TValue, ptr %2575, i32 0, i32 0
  %2577 = load i64, ptr %2576, align 8
  store i64 %2577, ptr %134, align 8
  %2578 = load ptr, ptr %132, align 8
  %2579 = getelementptr inbounds %struct.TValue, ptr %2578, i32 0, i32 0
  %2580 = load i64, ptr %2579, align 8
  store i64 %2580, ptr %135, align 8
  %2581 = load ptr, ptr %8, align 8
  %2582 = getelementptr inbounds i32, ptr %2581, i32 1
  store ptr %2582, ptr %8, align 8
  %2583 = load ptr, ptr %133, align 8
  store ptr %2583, ptr %136, align 8
  %2584 = load i64, ptr %134, align 8
  %2585 = load i64, ptr %135, align 8
  %2586 = sub i64 %2584, %2585
  %2587 = load ptr, ptr %136, align 8
  %2588 = getelementptr inbounds %struct.TValue, ptr %2587, i32 0, i32 0
  store i64 %2586, ptr %2588, align 8
  %2589 = load ptr, ptr %136, align 8
  %2590 = getelementptr inbounds %struct.TValue, ptr %2589, i32 0, i32 1
  store i8 3, ptr %2590, align 8
  br label %2647

2591:                                             ; preds = %2568, %2544
  %2592 = load ptr, ptr %131, align 8
  %2593 = getelementptr inbounds %struct.TValue, ptr %2592, i32 0, i32 1
  %2594 = load i8, ptr %2593, align 8
  %2595 = zext i8 %2594 to i32
  %2596 = icmp eq i32 %2595, 19
  br i1 %2596, label %2597, label %2601

2597:                                             ; preds = %2591
  %2598 = load ptr, ptr %131, align 8
  %2599 = getelementptr inbounds %struct.TValue, ptr %2598, i32 0, i32 0
  %2600 = load double, ptr %2599, align 8
  store double %2600, ptr %137, align 8
  br i1 true, label %2613, label %2646

2601:                                             ; preds = %2591
  %2602 = load ptr, ptr %131, align 8
  %2603 = getelementptr inbounds %struct.TValue, ptr %2602, i32 0, i32 1
  %2604 = load i8, ptr %2603, align 8
  %2605 = zext i8 %2604 to i32
  %2606 = icmp eq i32 %2605, 3
  br i1 %2606, label %2607, label %2612

2607:                                             ; preds = %2601
  %2608 = load ptr, ptr %131, align 8
  %2609 = getelementptr inbounds %struct.TValue, ptr %2608, i32 0, i32 0
  %2610 = load i64, ptr %2609, align 8
  %2611 = sitofp i64 %2610 to double
  store double %2611, ptr %137, align 8
  br i1 true, label %2613, label %2646

2612:                                             ; preds = %2601
  br i1 false, label %2613, label %2646

2613:                                             ; preds = %2612, %2607, %2597
  %2614 = load ptr, ptr %132, align 8
  %2615 = getelementptr inbounds %struct.TValue, ptr %2614, i32 0, i32 1
  %2616 = load i8, ptr %2615, align 8
  %2617 = zext i8 %2616 to i32
  %2618 = icmp eq i32 %2617, 19
  br i1 %2618, label %2619, label %2623

2619:                                             ; preds = %2613
  %2620 = load ptr, ptr %132, align 8
  %2621 = getelementptr inbounds %struct.TValue, ptr %2620, i32 0, i32 0
  %2622 = load double, ptr %2621, align 8
  store double %2622, ptr %138, align 8
  br i1 true, label %2635, label %2646

2623:                                             ; preds = %2613
  %2624 = load ptr, ptr %132, align 8
  %2625 = getelementptr inbounds %struct.TValue, ptr %2624, i32 0, i32 1
  %2626 = load i8, ptr %2625, align 8
  %2627 = zext i8 %2626 to i32
  %2628 = icmp eq i32 %2627, 3
  br i1 %2628, label %2629, label %2634

2629:                                             ; preds = %2623
  %2630 = load ptr, ptr %132, align 8
  %2631 = getelementptr inbounds %struct.TValue, ptr %2630, i32 0, i32 0
  %2632 = load i64, ptr %2631, align 8
  %2633 = sitofp i64 %2632 to double
  store double %2633, ptr %138, align 8
  br i1 true, label %2635, label %2646

2634:                                             ; preds = %2623
  br i1 false, label %2635, label %2646

2635:                                             ; preds = %2634, %2629, %2619
  %2636 = load ptr, ptr %8, align 8
  %2637 = getelementptr inbounds i32, ptr %2636, i32 1
  store ptr %2637, ptr %8, align 8
  %2638 = load ptr, ptr %133, align 8
  store ptr %2638, ptr %139, align 8
  %2639 = load double, ptr %137, align 8
  %2640 = load double, ptr %138, align 8
  %2641 = fsub double %2639, %2640
  %2642 = load ptr, ptr %139, align 8
  %2643 = getelementptr inbounds %struct.TValue, ptr %2642, i32 0, i32 0
  store double %2641, ptr %2643, align 8
  %2644 = load ptr, ptr %139, align 8
  %2645 = getelementptr inbounds %struct.TValue, ptr %2644, i32 0, i32 1
  store i8 19, ptr %2645, align 8
  br label %2646

2646:                                             ; preds = %2635, %2634, %2629, %2619, %2612, %2607, %2597
  br label %2647

2647:                                             ; preds = %2646, %2574
  %2648 = load i32, ptr %9, align 4
  %2649 = icmp ne i32 %2648, 0
  %2650 = zext i1 %2649 to i32
  %2651 = sext i32 %2650 to i64
  %2652 = icmp ne i64 %2651, 0
  br i1 %2652, label %2653, label %2661

2653:                                             ; preds = %2647
  %2654 = load ptr, ptr %3, align 8
  %2655 = load ptr, ptr %8, align 8
  %2656 = call i32 @luaG_traceexec(ptr noundef %2654, ptr noundef %2655)
  store i32 %2656, ptr %9, align 4
  %2657 = load ptr, ptr %4, align 8
  %2658 = getelementptr inbounds %struct.CallInfo, ptr %2657, i32 0, i32 0
  %2659 = load ptr, ptr %2658, align 8
  %2660 = getelementptr inbounds %union.StackValue, ptr %2659, i64 1
  store ptr %2660, ptr %7, align 8
  br label %2661

2661:                                             ; preds = %2653, %2647
  %2662 = load ptr, ptr %8, align 8
  %2663 = getelementptr inbounds i32, ptr %2662, i32 1
  store ptr %2663, ptr %8, align 8
  %2664 = load i32, ptr %2662, align 4
  store i32 %2664, ptr %10, align 4
  %2665 = load i32, ptr %10, align 4
  %2666 = lshr i32 %2665, 0
  %2667 = and i32 %2666, 127
  %2668 = zext i32 %2667 to i64
  %2669 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2668
  %2670 = load ptr, ptr %2669, align 8
  br label %7120

2671:                                             ; preds = %7120
  %2672 = load ptr, ptr %7, align 8
  %2673 = load i32, ptr %10, align 4
  %2674 = lshr i32 %2673, 16
  %2675 = and i32 %2674, 255
  %2676 = sext i32 %2675 to i64
  %2677 = getelementptr inbounds %union.StackValue, ptr %2672, i64 %2676
  store ptr %2677, ptr %140, align 8
  %2678 = load ptr, ptr %6, align 8
  %2679 = load i32, ptr %10, align 4
  %2680 = lshr i32 %2679, 24
  %2681 = and i32 %2680, 255
  %2682 = sext i32 %2681 to i64
  %2683 = getelementptr inbounds %struct.TValue, ptr %2678, i64 %2682
  store ptr %2683, ptr %141, align 8
  %2684 = load ptr, ptr %7, align 8
  %2685 = load i32, ptr %10, align 4
  %2686 = lshr i32 %2685, 7
  %2687 = and i32 %2686, 255
  %2688 = sext i32 %2687 to i64
  %2689 = getelementptr inbounds %union.StackValue, ptr %2684, i64 %2688
  store ptr %2689, ptr %142, align 8
  %2690 = load ptr, ptr %140, align 8
  %2691 = getelementptr inbounds %struct.TValue, ptr %2690, i32 0, i32 1
  %2692 = load i8, ptr %2691, align 8
  %2693 = zext i8 %2692 to i32
  %2694 = icmp eq i32 %2693, 3
  br i1 %2694, label %2695, label %2718

2695:                                             ; preds = %2671
  %2696 = load ptr, ptr %141, align 8
  %2697 = getelementptr inbounds %struct.TValue, ptr %2696, i32 0, i32 1
  %2698 = load i8, ptr %2697, align 8
  %2699 = zext i8 %2698 to i32
  %2700 = icmp eq i32 %2699, 3
  br i1 %2700, label %2701, label %2718

2701:                                             ; preds = %2695
  %2702 = load ptr, ptr %140, align 8
  %2703 = getelementptr inbounds %struct.TValue, ptr %2702, i32 0, i32 0
  %2704 = load i64, ptr %2703, align 8
  store i64 %2704, ptr %143, align 8
  %2705 = load ptr, ptr %141, align 8
  %2706 = getelementptr inbounds %struct.TValue, ptr %2705, i32 0, i32 0
  %2707 = load i64, ptr %2706, align 8
  store i64 %2707, ptr %144, align 8
  %2708 = load ptr, ptr %8, align 8
  %2709 = getelementptr inbounds i32, ptr %2708, i32 1
  store ptr %2709, ptr %8, align 8
  %2710 = load ptr, ptr %142, align 8
  store ptr %2710, ptr %145, align 8
  %2711 = load i64, ptr %143, align 8
  %2712 = load i64, ptr %144, align 8
  %2713 = mul i64 %2711, %2712
  %2714 = load ptr, ptr %145, align 8
  %2715 = getelementptr inbounds %struct.TValue, ptr %2714, i32 0, i32 0
  store i64 %2713, ptr %2715, align 8
  %2716 = load ptr, ptr %145, align 8
  %2717 = getelementptr inbounds %struct.TValue, ptr %2716, i32 0, i32 1
  store i8 3, ptr %2717, align 8
  br label %2774

2718:                                             ; preds = %2695, %2671
  %2719 = load ptr, ptr %140, align 8
  %2720 = getelementptr inbounds %struct.TValue, ptr %2719, i32 0, i32 1
  %2721 = load i8, ptr %2720, align 8
  %2722 = zext i8 %2721 to i32
  %2723 = icmp eq i32 %2722, 19
  br i1 %2723, label %2724, label %2728

2724:                                             ; preds = %2718
  %2725 = load ptr, ptr %140, align 8
  %2726 = getelementptr inbounds %struct.TValue, ptr %2725, i32 0, i32 0
  %2727 = load double, ptr %2726, align 8
  store double %2727, ptr %146, align 8
  br i1 true, label %2740, label %2773

2728:                                             ; preds = %2718
  %2729 = load ptr, ptr %140, align 8
  %2730 = getelementptr inbounds %struct.TValue, ptr %2729, i32 0, i32 1
  %2731 = load i8, ptr %2730, align 8
  %2732 = zext i8 %2731 to i32
  %2733 = icmp eq i32 %2732, 3
  br i1 %2733, label %2734, label %2739

2734:                                             ; preds = %2728
  %2735 = load ptr, ptr %140, align 8
  %2736 = getelementptr inbounds %struct.TValue, ptr %2735, i32 0, i32 0
  %2737 = load i64, ptr %2736, align 8
  %2738 = sitofp i64 %2737 to double
  store double %2738, ptr %146, align 8
  br i1 true, label %2740, label %2773

2739:                                             ; preds = %2728
  br i1 false, label %2740, label %2773

2740:                                             ; preds = %2739, %2734, %2724
  %2741 = load ptr, ptr %141, align 8
  %2742 = getelementptr inbounds %struct.TValue, ptr %2741, i32 0, i32 1
  %2743 = load i8, ptr %2742, align 8
  %2744 = zext i8 %2743 to i32
  %2745 = icmp eq i32 %2744, 19
  br i1 %2745, label %2746, label %2750

2746:                                             ; preds = %2740
  %2747 = load ptr, ptr %141, align 8
  %2748 = getelementptr inbounds %struct.TValue, ptr %2747, i32 0, i32 0
  %2749 = load double, ptr %2748, align 8
  store double %2749, ptr %147, align 8
  br i1 true, label %2762, label %2773

2750:                                             ; preds = %2740
  %2751 = load ptr, ptr %141, align 8
  %2752 = getelementptr inbounds %struct.TValue, ptr %2751, i32 0, i32 1
  %2753 = load i8, ptr %2752, align 8
  %2754 = zext i8 %2753 to i32
  %2755 = icmp eq i32 %2754, 3
  br i1 %2755, label %2756, label %2761

2756:                                             ; preds = %2750
  %2757 = load ptr, ptr %141, align 8
  %2758 = getelementptr inbounds %struct.TValue, ptr %2757, i32 0, i32 0
  %2759 = load i64, ptr %2758, align 8
  %2760 = sitofp i64 %2759 to double
  store double %2760, ptr %147, align 8
  br i1 true, label %2762, label %2773

2761:                                             ; preds = %2750
  br i1 false, label %2762, label %2773

2762:                                             ; preds = %2761, %2756, %2746
  %2763 = load ptr, ptr %8, align 8
  %2764 = getelementptr inbounds i32, ptr %2763, i32 1
  store ptr %2764, ptr %8, align 8
  %2765 = load ptr, ptr %142, align 8
  store ptr %2765, ptr %148, align 8
  %2766 = load double, ptr %146, align 8
  %2767 = load double, ptr %147, align 8
  %2768 = fmul double %2766, %2767
  %2769 = load ptr, ptr %148, align 8
  %2770 = getelementptr inbounds %struct.TValue, ptr %2769, i32 0, i32 0
  store double %2768, ptr %2770, align 8
  %2771 = load ptr, ptr %148, align 8
  %2772 = getelementptr inbounds %struct.TValue, ptr %2771, i32 0, i32 1
  store i8 19, ptr %2772, align 8
  br label %2773

2773:                                             ; preds = %2762, %2761, %2756, %2746, %2739, %2734, %2724
  br label %2774

2774:                                             ; preds = %2773, %2701
  %2775 = load i32, ptr %9, align 4
  %2776 = icmp ne i32 %2775, 0
  %2777 = zext i1 %2776 to i32
  %2778 = sext i32 %2777 to i64
  %2779 = icmp ne i64 %2778, 0
  br i1 %2779, label %2780, label %2788

2780:                                             ; preds = %2774
  %2781 = load ptr, ptr %3, align 8
  %2782 = load ptr, ptr %8, align 8
  %2783 = call i32 @luaG_traceexec(ptr noundef %2781, ptr noundef %2782)
  store i32 %2783, ptr %9, align 4
  %2784 = load ptr, ptr %4, align 8
  %2785 = getelementptr inbounds %struct.CallInfo, ptr %2784, i32 0, i32 0
  %2786 = load ptr, ptr %2785, align 8
  %2787 = getelementptr inbounds %union.StackValue, ptr %2786, i64 1
  store ptr %2787, ptr %7, align 8
  br label %2788

2788:                                             ; preds = %2780, %2774
  %2789 = load ptr, ptr %8, align 8
  %2790 = getelementptr inbounds i32, ptr %2789, i32 1
  store ptr %2790, ptr %8, align 8
  %2791 = load i32, ptr %2789, align 4
  store i32 %2791, ptr %10, align 4
  %2792 = load i32, ptr %10, align 4
  %2793 = lshr i32 %2792, 0
  %2794 = and i32 %2793, 127
  %2795 = zext i32 %2794 to i64
  %2796 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2795
  %2797 = load ptr, ptr %2796, align 8
  br label %7120

2798:                                             ; preds = %7120
  %2799 = load ptr, ptr %8, align 8
  %2800 = load ptr, ptr %4, align 8
  %2801 = getelementptr inbounds %struct.CallInfo, ptr %2800, i32 0, i32 4
  %2802 = getelementptr inbounds %struct.anon, ptr %2801, i32 0, i32 0
  store ptr %2799, ptr %2802, align 8
  %2803 = load ptr, ptr %4, align 8
  %2804 = getelementptr inbounds %struct.CallInfo, ptr %2803, i32 0, i32 1
  %2805 = load ptr, ptr %2804, align 8
  %2806 = load ptr, ptr %3, align 8
  %2807 = getelementptr inbounds %struct.lua_State, ptr %2806, i32 0, i32 6
  store ptr %2805, ptr %2807, align 8
  %2808 = load ptr, ptr %7, align 8
  %2809 = load i32, ptr %10, align 4
  %2810 = lshr i32 %2809, 16
  %2811 = and i32 %2810, 255
  %2812 = sext i32 %2811 to i64
  %2813 = getelementptr inbounds %union.StackValue, ptr %2808, i64 %2812
  store ptr %2813, ptr %149, align 8
  %2814 = load ptr, ptr %6, align 8
  %2815 = load i32, ptr %10, align 4
  %2816 = lshr i32 %2815, 24
  %2817 = and i32 %2816, 255
  %2818 = sext i32 %2817 to i64
  %2819 = getelementptr inbounds %struct.TValue, ptr %2814, i64 %2818
  store ptr %2819, ptr %150, align 8
  %2820 = load ptr, ptr %7, align 8
  %2821 = load i32, ptr %10, align 4
  %2822 = lshr i32 %2821, 7
  %2823 = and i32 %2822, 255
  %2824 = sext i32 %2823 to i64
  %2825 = getelementptr inbounds %union.StackValue, ptr %2820, i64 %2824
  store ptr %2825, ptr %151, align 8
  %2826 = load ptr, ptr %149, align 8
  %2827 = getelementptr inbounds %struct.TValue, ptr %2826, i32 0, i32 1
  %2828 = load i8, ptr %2827, align 8
  %2829 = zext i8 %2828 to i32
  %2830 = icmp eq i32 %2829, 3
  br i1 %2830, label %2831, label %2855

2831:                                             ; preds = %2798
  %2832 = load ptr, ptr %150, align 8
  %2833 = getelementptr inbounds %struct.TValue, ptr %2832, i32 0, i32 1
  %2834 = load i8, ptr %2833, align 8
  %2835 = zext i8 %2834 to i32
  %2836 = icmp eq i32 %2835, 3
  br i1 %2836, label %2837, label %2855

2837:                                             ; preds = %2831
  %2838 = load ptr, ptr %149, align 8
  %2839 = getelementptr inbounds %struct.TValue, ptr %2838, i32 0, i32 0
  %2840 = load i64, ptr %2839, align 8
  store i64 %2840, ptr %152, align 8
  %2841 = load ptr, ptr %150, align 8
  %2842 = getelementptr inbounds %struct.TValue, ptr %2841, i32 0, i32 0
  %2843 = load i64, ptr %2842, align 8
  store i64 %2843, ptr %153, align 8
  %2844 = load ptr, ptr %8, align 8
  %2845 = getelementptr inbounds i32, ptr %2844, i32 1
  store ptr %2845, ptr %8, align 8
  %2846 = load ptr, ptr %151, align 8
  store ptr %2846, ptr %154, align 8
  %2847 = load ptr, ptr %3, align 8
  %2848 = load i64, ptr %152, align 8
  %2849 = load i64, ptr %153, align 8
  %2850 = call i64 @luaV_mod(ptr noundef %2847, i64 noundef %2848, i64 noundef %2849)
  %2851 = load ptr, ptr %154, align 8
  %2852 = getelementptr inbounds %struct.TValue, ptr %2851, i32 0, i32 0
  store i64 %2850, ptr %2852, align 8
  %2853 = load ptr, ptr %154, align 8
  %2854 = getelementptr inbounds %struct.TValue, ptr %2853, i32 0, i32 1
  store i8 3, ptr %2854, align 8
  br label %2912

2855:                                             ; preds = %2831, %2798
  %2856 = load ptr, ptr %149, align 8
  %2857 = getelementptr inbounds %struct.TValue, ptr %2856, i32 0, i32 1
  %2858 = load i8, ptr %2857, align 8
  %2859 = zext i8 %2858 to i32
  %2860 = icmp eq i32 %2859, 19
  br i1 %2860, label %2861, label %2865

2861:                                             ; preds = %2855
  %2862 = load ptr, ptr %149, align 8
  %2863 = getelementptr inbounds %struct.TValue, ptr %2862, i32 0, i32 0
  %2864 = load double, ptr %2863, align 8
  store double %2864, ptr %155, align 8
  br i1 true, label %2877, label %2911

2865:                                             ; preds = %2855
  %2866 = load ptr, ptr %149, align 8
  %2867 = getelementptr inbounds %struct.TValue, ptr %2866, i32 0, i32 1
  %2868 = load i8, ptr %2867, align 8
  %2869 = zext i8 %2868 to i32
  %2870 = icmp eq i32 %2869, 3
  br i1 %2870, label %2871, label %2876

2871:                                             ; preds = %2865
  %2872 = load ptr, ptr %149, align 8
  %2873 = getelementptr inbounds %struct.TValue, ptr %2872, i32 0, i32 0
  %2874 = load i64, ptr %2873, align 8
  %2875 = sitofp i64 %2874 to double
  store double %2875, ptr %155, align 8
  br i1 true, label %2877, label %2911

2876:                                             ; preds = %2865
  br i1 false, label %2877, label %2911

2877:                                             ; preds = %2876, %2871, %2861
  %2878 = load ptr, ptr %150, align 8
  %2879 = getelementptr inbounds %struct.TValue, ptr %2878, i32 0, i32 1
  %2880 = load i8, ptr %2879, align 8
  %2881 = zext i8 %2880 to i32
  %2882 = icmp eq i32 %2881, 19
  br i1 %2882, label %2883, label %2887

2883:                                             ; preds = %2877
  %2884 = load ptr, ptr %150, align 8
  %2885 = getelementptr inbounds %struct.TValue, ptr %2884, i32 0, i32 0
  %2886 = load double, ptr %2885, align 8
  store double %2886, ptr %156, align 8
  br i1 true, label %2899, label %2911

2887:                                             ; preds = %2877
  %2888 = load ptr, ptr %150, align 8
  %2889 = getelementptr inbounds %struct.TValue, ptr %2888, i32 0, i32 1
  %2890 = load i8, ptr %2889, align 8
  %2891 = zext i8 %2890 to i32
  %2892 = icmp eq i32 %2891, 3
  br i1 %2892, label %2893, label %2898

2893:                                             ; preds = %2887
  %2894 = load ptr, ptr %150, align 8
  %2895 = getelementptr inbounds %struct.TValue, ptr %2894, i32 0, i32 0
  %2896 = load i64, ptr %2895, align 8
  %2897 = sitofp i64 %2896 to double
  store double %2897, ptr %156, align 8
  br i1 true, label %2899, label %2911

2898:                                             ; preds = %2887
  br i1 false, label %2899, label %2911

2899:                                             ; preds = %2898, %2893, %2883
  %2900 = load ptr, ptr %8, align 8
  %2901 = getelementptr inbounds i32, ptr %2900, i32 1
  store ptr %2901, ptr %8, align 8
  %2902 = load ptr, ptr %151, align 8
  store ptr %2902, ptr %157, align 8
  %2903 = load ptr, ptr %3, align 8
  %2904 = load double, ptr %155, align 8
  %2905 = load double, ptr %156, align 8
  %2906 = call double @luaV_modf(ptr noundef %2903, double noundef %2904, double noundef %2905)
  %2907 = load ptr, ptr %157, align 8
  %2908 = getelementptr inbounds %struct.TValue, ptr %2907, i32 0, i32 0
  store double %2906, ptr %2908, align 8
  %2909 = load ptr, ptr %157, align 8
  %2910 = getelementptr inbounds %struct.TValue, ptr %2909, i32 0, i32 1
  store i8 19, ptr %2910, align 8
  br label %2911

2911:                                             ; preds = %2899, %2898, %2893, %2883, %2876, %2871, %2861
  br label %2912

2912:                                             ; preds = %2911, %2837
  %2913 = load i32, ptr %9, align 4
  %2914 = icmp ne i32 %2913, 0
  %2915 = zext i1 %2914 to i32
  %2916 = sext i32 %2915 to i64
  %2917 = icmp ne i64 %2916, 0
  br i1 %2917, label %2918, label %2926

2918:                                             ; preds = %2912
  %2919 = load ptr, ptr %3, align 8
  %2920 = load ptr, ptr %8, align 8
  %2921 = call i32 @luaG_traceexec(ptr noundef %2919, ptr noundef %2920)
  store i32 %2921, ptr %9, align 4
  %2922 = load ptr, ptr %4, align 8
  %2923 = getelementptr inbounds %struct.CallInfo, ptr %2922, i32 0, i32 0
  %2924 = load ptr, ptr %2923, align 8
  %2925 = getelementptr inbounds %union.StackValue, ptr %2924, i64 1
  store ptr %2925, ptr %7, align 8
  br label %2926

2926:                                             ; preds = %2918, %2912
  %2927 = load ptr, ptr %8, align 8
  %2928 = getelementptr inbounds i32, ptr %2927, i32 1
  store ptr %2928, ptr %8, align 8
  %2929 = load i32, ptr %2927, align 4
  store i32 %2929, ptr %10, align 4
  %2930 = load i32, ptr %10, align 4
  %2931 = lshr i32 %2930, 0
  %2932 = and i32 %2931, 127
  %2933 = zext i32 %2932 to i64
  %2934 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %2933
  %2935 = load ptr, ptr %2934, align 8
  br label %7120

2936:                                             ; preds = %7120
  %2937 = load ptr, ptr %7, align 8
  %2938 = load i32, ptr %10, align 4
  %2939 = lshr i32 %2938, 7
  %2940 = and i32 %2939, 255
  %2941 = sext i32 %2940 to i64
  %2942 = getelementptr inbounds %union.StackValue, ptr %2937, i64 %2941
  store ptr %2942, ptr %158, align 8
  %2943 = load ptr, ptr %7, align 8
  %2944 = load i32, ptr %10, align 4
  %2945 = lshr i32 %2944, 16
  %2946 = and i32 %2945, 255
  %2947 = sext i32 %2946 to i64
  %2948 = getelementptr inbounds %union.StackValue, ptr %2943, i64 %2947
  store ptr %2948, ptr %159, align 8
  %2949 = load ptr, ptr %6, align 8
  %2950 = load i32, ptr %10, align 4
  %2951 = lshr i32 %2950, 24
  %2952 = and i32 %2951, 255
  %2953 = sext i32 %2952 to i64
  %2954 = getelementptr inbounds %struct.TValue, ptr %2949, i64 %2953
  store ptr %2954, ptr %160, align 8
  %2955 = load ptr, ptr %159, align 8
  %2956 = getelementptr inbounds %struct.TValue, ptr %2955, i32 0, i32 1
  %2957 = load i8, ptr %2956, align 8
  %2958 = zext i8 %2957 to i32
  %2959 = icmp eq i32 %2958, 19
  br i1 %2959, label %2960, label %2964

2960:                                             ; preds = %2936
  %2961 = load ptr, ptr %159, align 8
  %2962 = getelementptr inbounds %struct.TValue, ptr %2961, i32 0, i32 0
  %2963 = load double, ptr %2962, align 8
  store double %2963, ptr %161, align 8
  br i1 true, label %2976, label %3019

2964:                                             ; preds = %2936
  %2965 = load ptr, ptr %159, align 8
  %2966 = getelementptr inbounds %struct.TValue, ptr %2965, i32 0, i32 1
  %2967 = load i8, ptr %2966, align 8
  %2968 = zext i8 %2967 to i32
  %2969 = icmp eq i32 %2968, 3
  br i1 %2969, label %2970, label %2975

2970:                                             ; preds = %2964
  %2971 = load ptr, ptr %159, align 8
  %2972 = getelementptr inbounds %struct.TValue, ptr %2971, i32 0, i32 0
  %2973 = load i64, ptr %2972, align 8
  %2974 = sitofp i64 %2973 to double
  store double %2974, ptr %161, align 8
  br i1 true, label %2976, label %3019

2975:                                             ; preds = %2964
  br i1 false, label %2976, label %3019

2976:                                             ; preds = %2975, %2970, %2960
  %2977 = load ptr, ptr %160, align 8
  %2978 = getelementptr inbounds %struct.TValue, ptr %2977, i32 0, i32 1
  %2979 = load i8, ptr %2978, align 8
  %2980 = zext i8 %2979 to i32
  %2981 = icmp eq i32 %2980, 19
  br i1 %2981, label %2982, label %2986

2982:                                             ; preds = %2976
  %2983 = load ptr, ptr %160, align 8
  %2984 = getelementptr inbounds %struct.TValue, ptr %2983, i32 0, i32 0
  %2985 = load double, ptr %2984, align 8
  store double %2985, ptr %162, align 8
  br i1 true, label %2998, label %3019

2986:                                             ; preds = %2976
  %2987 = load ptr, ptr %160, align 8
  %2988 = getelementptr inbounds %struct.TValue, ptr %2987, i32 0, i32 1
  %2989 = load i8, ptr %2988, align 8
  %2990 = zext i8 %2989 to i32
  %2991 = icmp eq i32 %2990, 3
  br i1 %2991, label %2992, label %2997

2992:                                             ; preds = %2986
  %2993 = load ptr, ptr %160, align 8
  %2994 = getelementptr inbounds %struct.TValue, ptr %2993, i32 0, i32 0
  %2995 = load i64, ptr %2994, align 8
  %2996 = sitofp i64 %2995 to double
  store double %2996, ptr %162, align 8
  br i1 true, label %2998, label %3019

2997:                                             ; preds = %2986
  br i1 false, label %2998, label %3019

2998:                                             ; preds = %2997, %2992, %2982
  %2999 = load ptr, ptr %8, align 8
  %3000 = getelementptr inbounds i32, ptr %2999, i32 1
  store ptr %3000, ptr %8, align 8
  %3001 = load ptr, ptr %158, align 8
  store ptr %3001, ptr %163, align 8
  %3002 = load ptr, ptr %3, align 8
  %3003 = load double, ptr %162, align 8
  %3004 = fcmp oeq double %3003, 2.000000e+00
  br i1 %3004, label %3005, label %3009

3005:                                             ; preds = %2998
  %3006 = load double, ptr %161, align 8
  %3007 = load double, ptr %161, align 8
  %3008 = fmul double %3006, %3007
  br label %3013

3009:                                             ; preds = %2998
  %3010 = load double, ptr %161, align 8
  %3011 = load double, ptr %162, align 8
  %3012 = call double @pow(double noundef %3010, double noundef %3011) #8
  br label %3013

3013:                                             ; preds = %3009, %3005
  %3014 = phi double [ %3008, %3005 ], [ %3012, %3009 ]
  %3015 = load ptr, ptr %163, align 8
  %3016 = getelementptr inbounds %struct.TValue, ptr %3015, i32 0, i32 0
  store double %3014, ptr %3016, align 8
  %3017 = load ptr, ptr %163, align 8
  %3018 = getelementptr inbounds %struct.TValue, ptr %3017, i32 0, i32 1
  store i8 19, ptr %3018, align 8
  br label %3019

3019:                                             ; preds = %3013, %2997, %2992, %2982, %2975, %2970, %2960
  %3020 = load i32, ptr %9, align 4
  %3021 = icmp ne i32 %3020, 0
  %3022 = zext i1 %3021 to i32
  %3023 = sext i32 %3022 to i64
  %3024 = icmp ne i64 %3023, 0
  br i1 %3024, label %3025, label %3033

3025:                                             ; preds = %3019
  %3026 = load ptr, ptr %3, align 8
  %3027 = load ptr, ptr %8, align 8
  %3028 = call i32 @luaG_traceexec(ptr noundef %3026, ptr noundef %3027)
  store i32 %3028, ptr %9, align 4
  %3029 = load ptr, ptr %4, align 8
  %3030 = getelementptr inbounds %struct.CallInfo, ptr %3029, i32 0, i32 0
  %3031 = load ptr, ptr %3030, align 8
  %3032 = getelementptr inbounds %union.StackValue, ptr %3031, i64 1
  store ptr %3032, ptr %7, align 8
  br label %3033

3033:                                             ; preds = %3025, %3019
  %3034 = load ptr, ptr %8, align 8
  %3035 = getelementptr inbounds i32, ptr %3034, i32 1
  store ptr %3035, ptr %8, align 8
  %3036 = load i32, ptr %3034, align 4
  store i32 %3036, ptr %10, align 4
  %3037 = load i32, ptr %10, align 4
  %3038 = lshr i32 %3037, 0
  %3039 = and i32 %3038, 127
  %3040 = zext i32 %3039 to i64
  %3041 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3040
  %3042 = load ptr, ptr %3041, align 8
  br label %7120

3043:                                             ; preds = %7120
  %3044 = load ptr, ptr %7, align 8
  %3045 = load i32, ptr %10, align 4
  %3046 = lshr i32 %3045, 7
  %3047 = and i32 %3046, 255
  %3048 = sext i32 %3047 to i64
  %3049 = getelementptr inbounds %union.StackValue, ptr %3044, i64 %3048
  store ptr %3049, ptr %164, align 8
  %3050 = load ptr, ptr %7, align 8
  %3051 = load i32, ptr %10, align 4
  %3052 = lshr i32 %3051, 16
  %3053 = and i32 %3052, 255
  %3054 = sext i32 %3053 to i64
  %3055 = getelementptr inbounds %union.StackValue, ptr %3050, i64 %3054
  store ptr %3055, ptr %165, align 8
  %3056 = load ptr, ptr %6, align 8
  %3057 = load i32, ptr %10, align 4
  %3058 = lshr i32 %3057, 24
  %3059 = and i32 %3058, 255
  %3060 = sext i32 %3059 to i64
  %3061 = getelementptr inbounds %struct.TValue, ptr %3056, i64 %3060
  store ptr %3061, ptr %166, align 8
  %3062 = load ptr, ptr %165, align 8
  %3063 = getelementptr inbounds %struct.TValue, ptr %3062, i32 0, i32 1
  %3064 = load i8, ptr %3063, align 8
  %3065 = zext i8 %3064 to i32
  %3066 = icmp eq i32 %3065, 19
  br i1 %3066, label %3067, label %3071

3067:                                             ; preds = %3043
  %3068 = load ptr, ptr %165, align 8
  %3069 = getelementptr inbounds %struct.TValue, ptr %3068, i32 0, i32 0
  %3070 = load double, ptr %3069, align 8
  store double %3070, ptr %167, align 8
  br i1 true, label %3083, label %3116

3071:                                             ; preds = %3043
  %3072 = load ptr, ptr %165, align 8
  %3073 = getelementptr inbounds %struct.TValue, ptr %3072, i32 0, i32 1
  %3074 = load i8, ptr %3073, align 8
  %3075 = zext i8 %3074 to i32
  %3076 = icmp eq i32 %3075, 3
  br i1 %3076, label %3077, label %3082

3077:                                             ; preds = %3071
  %3078 = load ptr, ptr %165, align 8
  %3079 = getelementptr inbounds %struct.TValue, ptr %3078, i32 0, i32 0
  %3080 = load i64, ptr %3079, align 8
  %3081 = sitofp i64 %3080 to double
  store double %3081, ptr %167, align 8
  br i1 true, label %3083, label %3116

3082:                                             ; preds = %3071
  br i1 false, label %3083, label %3116

3083:                                             ; preds = %3082, %3077, %3067
  %3084 = load ptr, ptr %166, align 8
  %3085 = getelementptr inbounds %struct.TValue, ptr %3084, i32 0, i32 1
  %3086 = load i8, ptr %3085, align 8
  %3087 = zext i8 %3086 to i32
  %3088 = icmp eq i32 %3087, 19
  br i1 %3088, label %3089, label %3093

3089:                                             ; preds = %3083
  %3090 = load ptr, ptr %166, align 8
  %3091 = getelementptr inbounds %struct.TValue, ptr %3090, i32 0, i32 0
  %3092 = load double, ptr %3091, align 8
  store double %3092, ptr %168, align 8
  br i1 true, label %3105, label %3116

3093:                                             ; preds = %3083
  %3094 = load ptr, ptr %166, align 8
  %3095 = getelementptr inbounds %struct.TValue, ptr %3094, i32 0, i32 1
  %3096 = load i8, ptr %3095, align 8
  %3097 = zext i8 %3096 to i32
  %3098 = icmp eq i32 %3097, 3
  br i1 %3098, label %3099, label %3104

3099:                                             ; preds = %3093
  %3100 = load ptr, ptr %166, align 8
  %3101 = getelementptr inbounds %struct.TValue, ptr %3100, i32 0, i32 0
  %3102 = load i64, ptr %3101, align 8
  %3103 = sitofp i64 %3102 to double
  store double %3103, ptr %168, align 8
  br i1 true, label %3105, label %3116

3104:                                             ; preds = %3093
  br i1 false, label %3105, label %3116

3105:                                             ; preds = %3104, %3099, %3089
  %3106 = load ptr, ptr %8, align 8
  %3107 = getelementptr inbounds i32, ptr %3106, i32 1
  store ptr %3107, ptr %8, align 8
  %3108 = load ptr, ptr %164, align 8
  store ptr %3108, ptr %169, align 8
  %3109 = load double, ptr %167, align 8
  %3110 = load double, ptr %168, align 8
  %3111 = fdiv double %3109, %3110
  %3112 = load ptr, ptr %169, align 8
  %3113 = getelementptr inbounds %struct.TValue, ptr %3112, i32 0, i32 0
  store double %3111, ptr %3113, align 8
  %3114 = load ptr, ptr %169, align 8
  %3115 = getelementptr inbounds %struct.TValue, ptr %3114, i32 0, i32 1
  store i8 19, ptr %3115, align 8
  br label %3116

3116:                                             ; preds = %3105, %3104, %3099, %3089, %3082, %3077, %3067
  %3117 = load i32, ptr %9, align 4
  %3118 = icmp ne i32 %3117, 0
  %3119 = zext i1 %3118 to i32
  %3120 = sext i32 %3119 to i64
  %3121 = icmp ne i64 %3120, 0
  br i1 %3121, label %3122, label %3130

3122:                                             ; preds = %3116
  %3123 = load ptr, ptr %3, align 8
  %3124 = load ptr, ptr %8, align 8
  %3125 = call i32 @luaG_traceexec(ptr noundef %3123, ptr noundef %3124)
  store i32 %3125, ptr %9, align 4
  %3126 = load ptr, ptr %4, align 8
  %3127 = getelementptr inbounds %struct.CallInfo, ptr %3126, i32 0, i32 0
  %3128 = load ptr, ptr %3127, align 8
  %3129 = getelementptr inbounds %union.StackValue, ptr %3128, i64 1
  store ptr %3129, ptr %7, align 8
  br label %3130

3130:                                             ; preds = %3122, %3116
  %3131 = load ptr, ptr %8, align 8
  %3132 = getelementptr inbounds i32, ptr %3131, i32 1
  store ptr %3132, ptr %8, align 8
  %3133 = load i32, ptr %3131, align 4
  store i32 %3133, ptr %10, align 4
  %3134 = load i32, ptr %10, align 4
  %3135 = lshr i32 %3134, 0
  %3136 = and i32 %3135, 127
  %3137 = zext i32 %3136 to i64
  %3138 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3137
  %3139 = load ptr, ptr %3138, align 8
  br label %7120

3140:                                             ; preds = %7120
  %3141 = load ptr, ptr %8, align 8
  %3142 = load ptr, ptr %4, align 8
  %3143 = getelementptr inbounds %struct.CallInfo, ptr %3142, i32 0, i32 4
  %3144 = getelementptr inbounds %struct.anon, ptr %3143, i32 0, i32 0
  store ptr %3141, ptr %3144, align 8
  %3145 = load ptr, ptr %4, align 8
  %3146 = getelementptr inbounds %struct.CallInfo, ptr %3145, i32 0, i32 1
  %3147 = load ptr, ptr %3146, align 8
  %3148 = load ptr, ptr %3, align 8
  %3149 = getelementptr inbounds %struct.lua_State, ptr %3148, i32 0, i32 6
  store ptr %3147, ptr %3149, align 8
  %3150 = load ptr, ptr %7, align 8
  %3151 = load i32, ptr %10, align 4
  %3152 = lshr i32 %3151, 16
  %3153 = and i32 %3152, 255
  %3154 = sext i32 %3153 to i64
  %3155 = getelementptr inbounds %union.StackValue, ptr %3150, i64 %3154
  store ptr %3155, ptr %170, align 8
  %3156 = load ptr, ptr %6, align 8
  %3157 = load i32, ptr %10, align 4
  %3158 = lshr i32 %3157, 24
  %3159 = and i32 %3158, 255
  %3160 = sext i32 %3159 to i64
  %3161 = getelementptr inbounds %struct.TValue, ptr %3156, i64 %3160
  store ptr %3161, ptr %171, align 8
  %3162 = load ptr, ptr %7, align 8
  %3163 = load i32, ptr %10, align 4
  %3164 = lshr i32 %3163, 7
  %3165 = and i32 %3164, 255
  %3166 = sext i32 %3165 to i64
  %3167 = getelementptr inbounds %union.StackValue, ptr %3162, i64 %3166
  store ptr %3167, ptr %172, align 8
  %3168 = load ptr, ptr %170, align 8
  %3169 = getelementptr inbounds %struct.TValue, ptr %3168, i32 0, i32 1
  %3170 = load i8, ptr %3169, align 8
  %3171 = zext i8 %3170 to i32
  %3172 = icmp eq i32 %3171, 3
  br i1 %3172, label %3173, label %3197

3173:                                             ; preds = %3140
  %3174 = load ptr, ptr %171, align 8
  %3175 = getelementptr inbounds %struct.TValue, ptr %3174, i32 0, i32 1
  %3176 = load i8, ptr %3175, align 8
  %3177 = zext i8 %3176 to i32
  %3178 = icmp eq i32 %3177, 3
  br i1 %3178, label %3179, label %3197

3179:                                             ; preds = %3173
  %3180 = load ptr, ptr %170, align 8
  %3181 = getelementptr inbounds %struct.TValue, ptr %3180, i32 0, i32 0
  %3182 = load i64, ptr %3181, align 8
  store i64 %3182, ptr %173, align 8
  %3183 = load ptr, ptr %171, align 8
  %3184 = getelementptr inbounds %struct.TValue, ptr %3183, i32 0, i32 0
  %3185 = load i64, ptr %3184, align 8
  store i64 %3185, ptr %174, align 8
  %3186 = load ptr, ptr %8, align 8
  %3187 = getelementptr inbounds i32, ptr %3186, i32 1
  store ptr %3187, ptr %8, align 8
  %3188 = load ptr, ptr %172, align 8
  store ptr %3188, ptr %175, align 8
  %3189 = load ptr, ptr %3, align 8
  %3190 = load i64, ptr %173, align 8
  %3191 = load i64, ptr %174, align 8
  %3192 = call i64 @luaV_idiv(ptr noundef %3189, i64 noundef %3190, i64 noundef %3191)
  %3193 = load ptr, ptr %175, align 8
  %3194 = getelementptr inbounds %struct.TValue, ptr %3193, i32 0, i32 0
  store i64 %3192, ptr %3194, align 8
  %3195 = load ptr, ptr %175, align 8
  %3196 = getelementptr inbounds %struct.TValue, ptr %3195, i32 0, i32 1
  store i8 3, ptr %3196, align 8
  br label %3255

3197:                                             ; preds = %3173, %3140
  %3198 = load ptr, ptr %170, align 8
  %3199 = getelementptr inbounds %struct.TValue, ptr %3198, i32 0, i32 1
  %3200 = load i8, ptr %3199, align 8
  %3201 = zext i8 %3200 to i32
  %3202 = icmp eq i32 %3201, 19
  br i1 %3202, label %3203, label %3207

3203:                                             ; preds = %3197
  %3204 = load ptr, ptr %170, align 8
  %3205 = getelementptr inbounds %struct.TValue, ptr %3204, i32 0, i32 0
  %3206 = load double, ptr %3205, align 8
  store double %3206, ptr %176, align 8
  br i1 true, label %3219, label %3254

3207:                                             ; preds = %3197
  %3208 = load ptr, ptr %170, align 8
  %3209 = getelementptr inbounds %struct.TValue, ptr %3208, i32 0, i32 1
  %3210 = load i8, ptr %3209, align 8
  %3211 = zext i8 %3210 to i32
  %3212 = icmp eq i32 %3211, 3
  br i1 %3212, label %3213, label %3218

3213:                                             ; preds = %3207
  %3214 = load ptr, ptr %170, align 8
  %3215 = getelementptr inbounds %struct.TValue, ptr %3214, i32 0, i32 0
  %3216 = load i64, ptr %3215, align 8
  %3217 = sitofp i64 %3216 to double
  store double %3217, ptr %176, align 8
  br i1 true, label %3219, label %3254

3218:                                             ; preds = %3207
  br i1 false, label %3219, label %3254

3219:                                             ; preds = %3218, %3213, %3203
  %3220 = load ptr, ptr %171, align 8
  %3221 = getelementptr inbounds %struct.TValue, ptr %3220, i32 0, i32 1
  %3222 = load i8, ptr %3221, align 8
  %3223 = zext i8 %3222 to i32
  %3224 = icmp eq i32 %3223, 19
  br i1 %3224, label %3225, label %3229

3225:                                             ; preds = %3219
  %3226 = load ptr, ptr %171, align 8
  %3227 = getelementptr inbounds %struct.TValue, ptr %3226, i32 0, i32 0
  %3228 = load double, ptr %3227, align 8
  store double %3228, ptr %177, align 8
  br i1 true, label %3241, label %3254

3229:                                             ; preds = %3219
  %3230 = load ptr, ptr %171, align 8
  %3231 = getelementptr inbounds %struct.TValue, ptr %3230, i32 0, i32 1
  %3232 = load i8, ptr %3231, align 8
  %3233 = zext i8 %3232 to i32
  %3234 = icmp eq i32 %3233, 3
  br i1 %3234, label %3235, label %3240

3235:                                             ; preds = %3229
  %3236 = load ptr, ptr %171, align 8
  %3237 = getelementptr inbounds %struct.TValue, ptr %3236, i32 0, i32 0
  %3238 = load i64, ptr %3237, align 8
  %3239 = sitofp i64 %3238 to double
  store double %3239, ptr %177, align 8
  br i1 true, label %3241, label %3254

3240:                                             ; preds = %3229
  br i1 false, label %3241, label %3254

3241:                                             ; preds = %3240, %3235, %3225
  %3242 = load ptr, ptr %8, align 8
  %3243 = getelementptr inbounds i32, ptr %3242, i32 1
  store ptr %3243, ptr %8, align 8
  %3244 = load ptr, ptr %172, align 8
  store ptr %3244, ptr %178, align 8
  %3245 = load ptr, ptr %3, align 8
  %3246 = load double, ptr %176, align 8
  %3247 = load double, ptr %177, align 8
  %3248 = fdiv double %3246, %3247
  %3249 = call double @llvm.floor.f64(double %3248)
  %3250 = load ptr, ptr %178, align 8
  %3251 = getelementptr inbounds %struct.TValue, ptr %3250, i32 0, i32 0
  store double %3249, ptr %3251, align 8
  %3252 = load ptr, ptr %178, align 8
  %3253 = getelementptr inbounds %struct.TValue, ptr %3252, i32 0, i32 1
  store i8 19, ptr %3253, align 8
  br label %3254

3254:                                             ; preds = %3241, %3240, %3235, %3225, %3218, %3213, %3203
  br label %3255

3255:                                             ; preds = %3254, %3179
  %3256 = load i32, ptr %9, align 4
  %3257 = icmp ne i32 %3256, 0
  %3258 = zext i1 %3257 to i32
  %3259 = sext i32 %3258 to i64
  %3260 = icmp ne i64 %3259, 0
  br i1 %3260, label %3261, label %3269

3261:                                             ; preds = %3255
  %3262 = load ptr, ptr %3, align 8
  %3263 = load ptr, ptr %8, align 8
  %3264 = call i32 @luaG_traceexec(ptr noundef %3262, ptr noundef %3263)
  store i32 %3264, ptr %9, align 4
  %3265 = load ptr, ptr %4, align 8
  %3266 = getelementptr inbounds %struct.CallInfo, ptr %3265, i32 0, i32 0
  %3267 = load ptr, ptr %3266, align 8
  %3268 = getelementptr inbounds %union.StackValue, ptr %3267, i64 1
  store ptr %3268, ptr %7, align 8
  br label %3269

3269:                                             ; preds = %3261, %3255
  %3270 = load ptr, ptr %8, align 8
  %3271 = getelementptr inbounds i32, ptr %3270, i32 1
  store ptr %3271, ptr %8, align 8
  %3272 = load i32, ptr %3270, align 4
  store i32 %3272, ptr %10, align 4
  %3273 = load i32, ptr %10, align 4
  %3274 = lshr i32 %3273, 0
  %3275 = and i32 %3274, 127
  %3276 = zext i32 %3275 to i64
  %3277 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3276
  %3278 = load ptr, ptr %3277, align 8
  br label %7120

3279:                                             ; preds = %7120
  %3280 = load ptr, ptr %7, align 8
  %3281 = load i32, ptr %10, align 4
  %3282 = lshr i32 %3281, 7
  %3283 = and i32 %3282, 255
  %3284 = sext i32 %3283 to i64
  %3285 = getelementptr inbounds %union.StackValue, ptr %3280, i64 %3284
  store ptr %3285, ptr %179, align 8
  %3286 = load ptr, ptr %7, align 8
  %3287 = load i32, ptr %10, align 4
  %3288 = lshr i32 %3287, 16
  %3289 = and i32 %3288, 255
  %3290 = sext i32 %3289 to i64
  %3291 = getelementptr inbounds %union.StackValue, ptr %3286, i64 %3290
  store ptr %3291, ptr %180, align 8
  %3292 = load ptr, ptr %6, align 8
  %3293 = load i32, ptr %10, align 4
  %3294 = lshr i32 %3293, 24
  %3295 = and i32 %3294, 255
  %3296 = sext i32 %3295 to i64
  %3297 = getelementptr inbounds %struct.TValue, ptr %3292, i64 %3296
  store ptr %3297, ptr %181, align 8
  %3298 = load ptr, ptr %181, align 8
  %3299 = getelementptr inbounds %struct.TValue, ptr %3298, i32 0, i32 0
  %3300 = load i64, ptr %3299, align 8
  store i64 %3300, ptr %183, align 8
  %3301 = load ptr, ptr %180, align 8
  %3302 = getelementptr inbounds %struct.TValue, ptr %3301, i32 0, i32 1
  %3303 = load i8, ptr %3302, align 8
  %3304 = zext i8 %3303 to i32
  %3305 = icmp eq i32 %3304, 3
  %3306 = zext i1 %3305 to i32
  %3307 = icmp ne i32 %3306, 0
  %3308 = zext i1 %3307 to i32
  %3309 = sext i32 %3308 to i64
  %3310 = icmp ne i64 %3309, 0
  br i1 %3310, label %3311, label %3315

3311:                                             ; preds = %3279
  %3312 = load ptr, ptr %180, align 8
  %3313 = getelementptr inbounds %struct.TValue, ptr %3312, i32 0, i32 0
  %3314 = load i64, ptr %3313, align 8
  store i64 %3314, ptr %182, align 8
  br i1 true, label %3319, label %3330

3315:                                             ; preds = %3279
  %3316 = load ptr, ptr %180, align 8
  %3317 = call i32 @luaV_tointegerns(ptr noundef %3316, ptr noundef %182, i32 noundef 0)
  %3318 = icmp ne i32 %3317, 0
  br i1 %3318, label %3319, label %3330

3319:                                             ; preds = %3315, %3311
  %3320 = load ptr, ptr %8, align 8
  %3321 = getelementptr inbounds i32, ptr %3320, i32 1
  store ptr %3321, ptr %8, align 8
  %3322 = load ptr, ptr %179, align 8
  store ptr %3322, ptr %184, align 8
  %3323 = load i64, ptr %182, align 8
  %3324 = load i64, ptr %183, align 8
  %3325 = and i64 %3323, %3324
  %3326 = load ptr, ptr %184, align 8
  %3327 = getelementptr inbounds %struct.TValue, ptr %3326, i32 0, i32 0
  store i64 %3325, ptr %3327, align 8
  %3328 = load ptr, ptr %184, align 8
  %3329 = getelementptr inbounds %struct.TValue, ptr %3328, i32 0, i32 1
  store i8 3, ptr %3329, align 8
  br label %3330

3330:                                             ; preds = %3319, %3315, %3311
  %3331 = load i32, ptr %9, align 4
  %3332 = icmp ne i32 %3331, 0
  %3333 = zext i1 %3332 to i32
  %3334 = sext i32 %3333 to i64
  %3335 = icmp ne i64 %3334, 0
  br i1 %3335, label %3336, label %3344

3336:                                             ; preds = %3330
  %3337 = load ptr, ptr %3, align 8
  %3338 = load ptr, ptr %8, align 8
  %3339 = call i32 @luaG_traceexec(ptr noundef %3337, ptr noundef %3338)
  store i32 %3339, ptr %9, align 4
  %3340 = load ptr, ptr %4, align 8
  %3341 = getelementptr inbounds %struct.CallInfo, ptr %3340, i32 0, i32 0
  %3342 = load ptr, ptr %3341, align 8
  %3343 = getelementptr inbounds %union.StackValue, ptr %3342, i64 1
  store ptr %3343, ptr %7, align 8
  br label %3344

3344:                                             ; preds = %3336, %3330
  %3345 = load ptr, ptr %8, align 8
  %3346 = getelementptr inbounds i32, ptr %3345, i32 1
  store ptr %3346, ptr %8, align 8
  %3347 = load i32, ptr %3345, align 4
  store i32 %3347, ptr %10, align 4
  %3348 = load i32, ptr %10, align 4
  %3349 = lshr i32 %3348, 0
  %3350 = and i32 %3349, 127
  %3351 = zext i32 %3350 to i64
  %3352 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3351
  %3353 = load ptr, ptr %3352, align 8
  br label %7120

3354:                                             ; preds = %7120
  %3355 = load ptr, ptr %7, align 8
  %3356 = load i32, ptr %10, align 4
  %3357 = lshr i32 %3356, 7
  %3358 = and i32 %3357, 255
  %3359 = sext i32 %3358 to i64
  %3360 = getelementptr inbounds %union.StackValue, ptr %3355, i64 %3359
  store ptr %3360, ptr %185, align 8
  %3361 = load ptr, ptr %7, align 8
  %3362 = load i32, ptr %10, align 4
  %3363 = lshr i32 %3362, 16
  %3364 = and i32 %3363, 255
  %3365 = sext i32 %3364 to i64
  %3366 = getelementptr inbounds %union.StackValue, ptr %3361, i64 %3365
  store ptr %3366, ptr %186, align 8
  %3367 = load ptr, ptr %6, align 8
  %3368 = load i32, ptr %10, align 4
  %3369 = lshr i32 %3368, 24
  %3370 = and i32 %3369, 255
  %3371 = sext i32 %3370 to i64
  %3372 = getelementptr inbounds %struct.TValue, ptr %3367, i64 %3371
  store ptr %3372, ptr %187, align 8
  %3373 = load ptr, ptr %187, align 8
  %3374 = getelementptr inbounds %struct.TValue, ptr %3373, i32 0, i32 0
  %3375 = load i64, ptr %3374, align 8
  store i64 %3375, ptr %189, align 8
  %3376 = load ptr, ptr %186, align 8
  %3377 = getelementptr inbounds %struct.TValue, ptr %3376, i32 0, i32 1
  %3378 = load i8, ptr %3377, align 8
  %3379 = zext i8 %3378 to i32
  %3380 = icmp eq i32 %3379, 3
  %3381 = zext i1 %3380 to i32
  %3382 = icmp ne i32 %3381, 0
  %3383 = zext i1 %3382 to i32
  %3384 = sext i32 %3383 to i64
  %3385 = icmp ne i64 %3384, 0
  br i1 %3385, label %3386, label %3390

3386:                                             ; preds = %3354
  %3387 = load ptr, ptr %186, align 8
  %3388 = getelementptr inbounds %struct.TValue, ptr %3387, i32 0, i32 0
  %3389 = load i64, ptr %3388, align 8
  store i64 %3389, ptr %188, align 8
  br i1 true, label %3394, label %3405

3390:                                             ; preds = %3354
  %3391 = load ptr, ptr %186, align 8
  %3392 = call i32 @luaV_tointegerns(ptr noundef %3391, ptr noundef %188, i32 noundef 0)
  %3393 = icmp ne i32 %3392, 0
  br i1 %3393, label %3394, label %3405

3394:                                             ; preds = %3390, %3386
  %3395 = load ptr, ptr %8, align 8
  %3396 = getelementptr inbounds i32, ptr %3395, i32 1
  store ptr %3396, ptr %8, align 8
  %3397 = load ptr, ptr %185, align 8
  store ptr %3397, ptr %190, align 8
  %3398 = load i64, ptr %188, align 8
  %3399 = load i64, ptr %189, align 8
  %3400 = or i64 %3398, %3399
  %3401 = load ptr, ptr %190, align 8
  %3402 = getelementptr inbounds %struct.TValue, ptr %3401, i32 0, i32 0
  store i64 %3400, ptr %3402, align 8
  %3403 = load ptr, ptr %190, align 8
  %3404 = getelementptr inbounds %struct.TValue, ptr %3403, i32 0, i32 1
  store i8 3, ptr %3404, align 8
  br label %3405

3405:                                             ; preds = %3394, %3390, %3386
  %3406 = load i32, ptr %9, align 4
  %3407 = icmp ne i32 %3406, 0
  %3408 = zext i1 %3407 to i32
  %3409 = sext i32 %3408 to i64
  %3410 = icmp ne i64 %3409, 0
  br i1 %3410, label %3411, label %3419

3411:                                             ; preds = %3405
  %3412 = load ptr, ptr %3, align 8
  %3413 = load ptr, ptr %8, align 8
  %3414 = call i32 @luaG_traceexec(ptr noundef %3412, ptr noundef %3413)
  store i32 %3414, ptr %9, align 4
  %3415 = load ptr, ptr %4, align 8
  %3416 = getelementptr inbounds %struct.CallInfo, ptr %3415, i32 0, i32 0
  %3417 = load ptr, ptr %3416, align 8
  %3418 = getelementptr inbounds %union.StackValue, ptr %3417, i64 1
  store ptr %3418, ptr %7, align 8
  br label %3419

3419:                                             ; preds = %3411, %3405
  %3420 = load ptr, ptr %8, align 8
  %3421 = getelementptr inbounds i32, ptr %3420, i32 1
  store ptr %3421, ptr %8, align 8
  %3422 = load i32, ptr %3420, align 4
  store i32 %3422, ptr %10, align 4
  %3423 = load i32, ptr %10, align 4
  %3424 = lshr i32 %3423, 0
  %3425 = and i32 %3424, 127
  %3426 = zext i32 %3425 to i64
  %3427 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3426
  %3428 = load ptr, ptr %3427, align 8
  br label %7120

3429:                                             ; preds = %7120
  %3430 = load ptr, ptr %7, align 8
  %3431 = load i32, ptr %10, align 4
  %3432 = lshr i32 %3431, 7
  %3433 = and i32 %3432, 255
  %3434 = sext i32 %3433 to i64
  %3435 = getelementptr inbounds %union.StackValue, ptr %3430, i64 %3434
  store ptr %3435, ptr %191, align 8
  %3436 = load ptr, ptr %7, align 8
  %3437 = load i32, ptr %10, align 4
  %3438 = lshr i32 %3437, 16
  %3439 = and i32 %3438, 255
  %3440 = sext i32 %3439 to i64
  %3441 = getelementptr inbounds %union.StackValue, ptr %3436, i64 %3440
  store ptr %3441, ptr %192, align 8
  %3442 = load ptr, ptr %6, align 8
  %3443 = load i32, ptr %10, align 4
  %3444 = lshr i32 %3443, 24
  %3445 = and i32 %3444, 255
  %3446 = sext i32 %3445 to i64
  %3447 = getelementptr inbounds %struct.TValue, ptr %3442, i64 %3446
  store ptr %3447, ptr %193, align 8
  %3448 = load ptr, ptr %193, align 8
  %3449 = getelementptr inbounds %struct.TValue, ptr %3448, i32 0, i32 0
  %3450 = load i64, ptr %3449, align 8
  store i64 %3450, ptr %195, align 8
  %3451 = load ptr, ptr %192, align 8
  %3452 = getelementptr inbounds %struct.TValue, ptr %3451, i32 0, i32 1
  %3453 = load i8, ptr %3452, align 8
  %3454 = zext i8 %3453 to i32
  %3455 = icmp eq i32 %3454, 3
  %3456 = zext i1 %3455 to i32
  %3457 = icmp ne i32 %3456, 0
  %3458 = zext i1 %3457 to i32
  %3459 = sext i32 %3458 to i64
  %3460 = icmp ne i64 %3459, 0
  br i1 %3460, label %3461, label %3465

3461:                                             ; preds = %3429
  %3462 = load ptr, ptr %192, align 8
  %3463 = getelementptr inbounds %struct.TValue, ptr %3462, i32 0, i32 0
  %3464 = load i64, ptr %3463, align 8
  store i64 %3464, ptr %194, align 8
  br i1 true, label %3469, label %3480

3465:                                             ; preds = %3429
  %3466 = load ptr, ptr %192, align 8
  %3467 = call i32 @luaV_tointegerns(ptr noundef %3466, ptr noundef %194, i32 noundef 0)
  %3468 = icmp ne i32 %3467, 0
  br i1 %3468, label %3469, label %3480

3469:                                             ; preds = %3465, %3461
  %3470 = load ptr, ptr %8, align 8
  %3471 = getelementptr inbounds i32, ptr %3470, i32 1
  store ptr %3471, ptr %8, align 8
  %3472 = load ptr, ptr %191, align 8
  store ptr %3472, ptr %196, align 8
  %3473 = load i64, ptr %194, align 8
  %3474 = load i64, ptr %195, align 8
  %3475 = xor i64 %3473, %3474
  %3476 = load ptr, ptr %196, align 8
  %3477 = getelementptr inbounds %struct.TValue, ptr %3476, i32 0, i32 0
  store i64 %3475, ptr %3477, align 8
  %3478 = load ptr, ptr %196, align 8
  %3479 = getelementptr inbounds %struct.TValue, ptr %3478, i32 0, i32 1
  store i8 3, ptr %3479, align 8
  br label %3480

3480:                                             ; preds = %3469, %3465, %3461
  %3481 = load i32, ptr %9, align 4
  %3482 = icmp ne i32 %3481, 0
  %3483 = zext i1 %3482 to i32
  %3484 = sext i32 %3483 to i64
  %3485 = icmp ne i64 %3484, 0
  br i1 %3485, label %3486, label %3494

3486:                                             ; preds = %3480
  %3487 = load ptr, ptr %3, align 8
  %3488 = load ptr, ptr %8, align 8
  %3489 = call i32 @luaG_traceexec(ptr noundef %3487, ptr noundef %3488)
  store i32 %3489, ptr %9, align 4
  %3490 = load ptr, ptr %4, align 8
  %3491 = getelementptr inbounds %struct.CallInfo, ptr %3490, i32 0, i32 0
  %3492 = load ptr, ptr %3491, align 8
  %3493 = getelementptr inbounds %union.StackValue, ptr %3492, i64 1
  store ptr %3493, ptr %7, align 8
  br label %3494

3494:                                             ; preds = %3486, %3480
  %3495 = load ptr, ptr %8, align 8
  %3496 = getelementptr inbounds i32, ptr %3495, i32 1
  store ptr %3496, ptr %8, align 8
  %3497 = load i32, ptr %3495, align 4
  store i32 %3497, ptr %10, align 4
  %3498 = load i32, ptr %10, align 4
  %3499 = lshr i32 %3498, 0
  %3500 = and i32 %3499, 127
  %3501 = zext i32 %3500 to i64
  %3502 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3501
  %3503 = load ptr, ptr %3502, align 8
  br label %7120

3504:                                             ; preds = %7120
  %3505 = load ptr, ptr %7, align 8
  %3506 = load i32, ptr %10, align 4
  %3507 = lshr i32 %3506, 7
  %3508 = and i32 %3507, 255
  %3509 = sext i32 %3508 to i64
  %3510 = getelementptr inbounds %union.StackValue, ptr %3505, i64 %3509
  store ptr %3510, ptr %197, align 8
  %3511 = load ptr, ptr %7, align 8
  %3512 = load i32, ptr %10, align 4
  %3513 = lshr i32 %3512, 16
  %3514 = and i32 %3513, 255
  %3515 = sext i32 %3514 to i64
  %3516 = getelementptr inbounds %union.StackValue, ptr %3511, i64 %3515
  store ptr %3516, ptr %198, align 8
  %3517 = load i32, ptr %10, align 4
  %3518 = lshr i32 %3517, 24
  %3519 = and i32 %3518, 255
  %3520 = sub nsw i32 %3519, 127
  store i32 %3520, ptr %199, align 4
  %3521 = load ptr, ptr %198, align 8
  %3522 = getelementptr inbounds %struct.TValue, ptr %3521, i32 0, i32 1
  %3523 = load i8, ptr %3522, align 8
  %3524 = zext i8 %3523 to i32
  %3525 = icmp eq i32 %3524, 3
  %3526 = zext i1 %3525 to i32
  %3527 = icmp ne i32 %3526, 0
  %3528 = zext i1 %3527 to i32
  %3529 = sext i32 %3528 to i64
  %3530 = icmp ne i64 %3529, 0
  br i1 %3530, label %3531, label %3535

3531:                                             ; preds = %3504
  %3532 = load ptr, ptr %198, align 8
  %3533 = getelementptr inbounds %struct.TValue, ptr %3532, i32 0, i32 0
  %3534 = load i64, ptr %3533, align 8
  store i64 %3534, ptr %200, align 8
  br i1 true, label %3539, label %3552

3535:                                             ; preds = %3504
  %3536 = load ptr, ptr %198, align 8
  %3537 = call i32 @luaV_tointegerns(ptr noundef %3536, ptr noundef %200, i32 noundef 0)
  %3538 = icmp ne i32 %3537, 0
  br i1 %3538, label %3539, label %3552

3539:                                             ; preds = %3535, %3531
  %3540 = load ptr, ptr %8, align 8
  %3541 = getelementptr inbounds i32, ptr %3540, i32 1
  store ptr %3541, ptr %8, align 8
  %3542 = load ptr, ptr %197, align 8
  store ptr %3542, ptr %201, align 8
  %3543 = load i64, ptr %200, align 8
  %3544 = load i32, ptr %199, align 4
  %3545 = sub nsw i32 0, %3544
  %3546 = sext i32 %3545 to i64
  %3547 = call i64 @luaV_shiftl(i64 noundef %3543, i64 noundef %3546)
  %3548 = load ptr, ptr %201, align 8
  %3549 = getelementptr inbounds %struct.TValue, ptr %3548, i32 0, i32 0
  store i64 %3547, ptr %3549, align 8
  %3550 = load ptr, ptr %201, align 8
  %3551 = getelementptr inbounds %struct.TValue, ptr %3550, i32 0, i32 1
  store i8 3, ptr %3551, align 8
  br label %3552

3552:                                             ; preds = %3539, %3535, %3531
  %3553 = load i32, ptr %9, align 4
  %3554 = icmp ne i32 %3553, 0
  %3555 = zext i1 %3554 to i32
  %3556 = sext i32 %3555 to i64
  %3557 = icmp ne i64 %3556, 0
  br i1 %3557, label %3558, label %3566

3558:                                             ; preds = %3552
  %3559 = load ptr, ptr %3, align 8
  %3560 = load ptr, ptr %8, align 8
  %3561 = call i32 @luaG_traceexec(ptr noundef %3559, ptr noundef %3560)
  store i32 %3561, ptr %9, align 4
  %3562 = load ptr, ptr %4, align 8
  %3563 = getelementptr inbounds %struct.CallInfo, ptr %3562, i32 0, i32 0
  %3564 = load ptr, ptr %3563, align 8
  %3565 = getelementptr inbounds %union.StackValue, ptr %3564, i64 1
  store ptr %3565, ptr %7, align 8
  br label %3566

3566:                                             ; preds = %3558, %3552
  %3567 = load ptr, ptr %8, align 8
  %3568 = getelementptr inbounds i32, ptr %3567, i32 1
  store ptr %3568, ptr %8, align 8
  %3569 = load i32, ptr %3567, align 4
  store i32 %3569, ptr %10, align 4
  %3570 = load i32, ptr %10, align 4
  %3571 = lshr i32 %3570, 0
  %3572 = and i32 %3571, 127
  %3573 = zext i32 %3572 to i64
  %3574 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3573
  %3575 = load ptr, ptr %3574, align 8
  br label %7120

3576:                                             ; preds = %7120
  %3577 = load ptr, ptr %7, align 8
  %3578 = load i32, ptr %10, align 4
  %3579 = lshr i32 %3578, 7
  %3580 = and i32 %3579, 255
  %3581 = sext i32 %3580 to i64
  %3582 = getelementptr inbounds %union.StackValue, ptr %3577, i64 %3581
  store ptr %3582, ptr %202, align 8
  %3583 = load ptr, ptr %7, align 8
  %3584 = load i32, ptr %10, align 4
  %3585 = lshr i32 %3584, 16
  %3586 = and i32 %3585, 255
  %3587 = sext i32 %3586 to i64
  %3588 = getelementptr inbounds %union.StackValue, ptr %3583, i64 %3587
  store ptr %3588, ptr %203, align 8
  %3589 = load i32, ptr %10, align 4
  %3590 = lshr i32 %3589, 24
  %3591 = and i32 %3590, 255
  %3592 = sub nsw i32 %3591, 127
  store i32 %3592, ptr %204, align 4
  %3593 = load ptr, ptr %203, align 8
  %3594 = getelementptr inbounds %struct.TValue, ptr %3593, i32 0, i32 1
  %3595 = load i8, ptr %3594, align 8
  %3596 = zext i8 %3595 to i32
  %3597 = icmp eq i32 %3596, 3
  %3598 = zext i1 %3597 to i32
  %3599 = icmp ne i32 %3598, 0
  %3600 = zext i1 %3599 to i32
  %3601 = sext i32 %3600 to i64
  %3602 = icmp ne i64 %3601, 0
  br i1 %3602, label %3603, label %3607

3603:                                             ; preds = %3576
  %3604 = load ptr, ptr %203, align 8
  %3605 = getelementptr inbounds %struct.TValue, ptr %3604, i32 0, i32 0
  %3606 = load i64, ptr %3605, align 8
  store i64 %3606, ptr %205, align 8
  br i1 true, label %3611, label %3623

3607:                                             ; preds = %3576
  %3608 = load ptr, ptr %203, align 8
  %3609 = call i32 @luaV_tointegerns(ptr noundef %3608, ptr noundef %205, i32 noundef 0)
  %3610 = icmp ne i32 %3609, 0
  br i1 %3610, label %3611, label %3623

3611:                                             ; preds = %3607, %3603
  %3612 = load ptr, ptr %8, align 8
  %3613 = getelementptr inbounds i32, ptr %3612, i32 1
  store ptr %3613, ptr %8, align 8
  %3614 = load ptr, ptr %202, align 8
  store ptr %3614, ptr %206, align 8
  %3615 = load i32, ptr %204, align 4
  %3616 = sext i32 %3615 to i64
  %3617 = load i64, ptr %205, align 8
  %3618 = call i64 @luaV_shiftl(i64 noundef %3616, i64 noundef %3617)
  %3619 = load ptr, ptr %206, align 8
  %3620 = getelementptr inbounds %struct.TValue, ptr %3619, i32 0, i32 0
  store i64 %3618, ptr %3620, align 8
  %3621 = load ptr, ptr %206, align 8
  %3622 = getelementptr inbounds %struct.TValue, ptr %3621, i32 0, i32 1
  store i8 3, ptr %3622, align 8
  br label %3623

3623:                                             ; preds = %3611, %3607, %3603
  %3624 = load i32, ptr %9, align 4
  %3625 = icmp ne i32 %3624, 0
  %3626 = zext i1 %3625 to i32
  %3627 = sext i32 %3626 to i64
  %3628 = icmp ne i64 %3627, 0
  br i1 %3628, label %3629, label %3637

3629:                                             ; preds = %3623
  %3630 = load ptr, ptr %3, align 8
  %3631 = load ptr, ptr %8, align 8
  %3632 = call i32 @luaG_traceexec(ptr noundef %3630, ptr noundef %3631)
  store i32 %3632, ptr %9, align 4
  %3633 = load ptr, ptr %4, align 8
  %3634 = getelementptr inbounds %struct.CallInfo, ptr %3633, i32 0, i32 0
  %3635 = load ptr, ptr %3634, align 8
  %3636 = getelementptr inbounds %union.StackValue, ptr %3635, i64 1
  store ptr %3636, ptr %7, align 8
  br label %3637

3637:                                             ; preds = %3629, %3623
  %3638 = load ptr, ptr %8, align 8
  %3639 = getelementptr inbounds i32, ptr %3638, i32 1
  store ptr %3639, ptr %8, align 8
  %3640 = load i32, ptr %3638, align 4
  store i32 %3640, ptr %10, align 4
  %3641 = load i32, ptr %10, align 4
  %3642 = lshr i32 %3641, 0
  %3643 = and i32 %3642, 127
  %3644 = zext i32 %3643 to i64
  %3645 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3644
  %3646 = load ptr, ptr %3645, align 8
  br label %7120

3647:                                             ; preds = %7120
  %3648 = load ptr, ptr %7, align 8
  %3649 = load i32, ptr %10, align 4
  %3650 = lshr i32 %3649, 16
  %3651 = and i32 %3650, 255
  %3652 = sext i32 %3651 to i64
  %3653 = getelementptr inbounds %union.StackValue, ptr %3648, i64 %3652
  store ptr %3653, ptr %207, align 8
  %3654 = load ptr, ptr %7, align 8
  %3655 = load i32, ptr %10, align 4
  %3656 = lshr i32 %3655, 24
  %3657 = and i32 %3656, 255
  %3658 = sext i32 %3657 to i64
  %3659 = getelementptr inbounds %union.StackValue, ptr %3654, i64 %3658
  store ptr %3659, ptr %208, align 8
  %3660 = load ptr, ptr %7, align 8
  %3661 = load i32, ptr %10, align 4
  %3662 = lshr i32 %3661, 7
  %3663 = and i32 %3662, 255
  %3664 = sext i32 %3663 to i64
  %3665 = getelementptr inbounds %union.StackValue, ptr %3660, i64 %3664
  store ptr %3665, ptr %209, align 8
  %3666 = load ptr, ptr %207, align 8
  %3667 = getelementptr inbounds %struct.TValue, ptr %3666, i32 0, i32 1
  %3668 = load i8, ptr %3667, align 8
  %3669 = zext i8 %3668 to i32
  %3670 = icmp eq i32 %3669, 3
  br i1 %3670, label %3671, label %3694

3671:                                             ; preds = %3647
  %3672 = load ptr, ptr %208, align 8
  %3673 = getelementptr inbounds %struct.TValue, ptr %3672, i32 0, i32 1
  %3674 = load i8, ptr %3673, align 8
  %3675 = zext i8 %3674 to i32
  %3676 = icmp eq i32 %3675, 3
  br i1 %3676, label %3677, label %3694

3677:                                             ; preds = %3671
  %3678 = load ptr, ptr %207, align 8
  %3679 = getelementptr inbounds %struct.TValue, ptr %3678, i32 0, i32 0
  %3680 = load i64, ptr %3679, align 8
  store i64 %3680, ptr %210, align 8
  %3681 = load ptr, ptr %208, align 8
  %3682 = getelementptr inbounds %struct.TValue, ptr %3681, i32 0, i32 0
  %3683 = load i64, ptr %3682, align 8
  store i64 %3683, ptr %211, align 8
  %3684 = load ptr, ptr %8, align 8
  %3685 = getelementptr inbounds i32, ptr %3684, i32 1
  store ptr %3685, ptr %8, align 8
  %3686 = load ptr, ptr %209, align 8
  store ptr %3686, ptr %212, align 8
  %3687 = load i64, ptr %210, align 8
  %3688 = load i64, ptr %211, align 8
  %3689 = add i64 %3687, %3688
  %3690 = load ptr, ptr %212, align 8
  %3691 = getelementptr inbounds %struct.TValue, ptr %3690, i32 0, i32 0
  store i64 %3689, ptr %3691, align 8
  %3692 = load ptr, ptr %212, align 8
  %3693 = getelementptr inbounds %struct.TValue, ptr %3692, i32 0, i32 1
  store i8 3, ptr %3693, align 8
  br label %3750

3694:                                             ; preds = %3671, %3647
  %3695 = load ptr, ptr %207, align 8
  %3696 = getelementptr inbounds %struct.TValue, ptr %3695, i32 0, i32 1
  %3697 = load i8, ptr %3696, align 8
  %3698 = zext i8 %3697 to i32
  %3699 = icmp eq i32 %3698, 19
  br i1 %3699, label %3700, label %3704

3700:                                             ; preds = %3694
  %3701 = load ptr, ptr %207, align 8
  %3702 = getelementptr inbounds %struct.TValue, ptr %3701, i32 0, i32 0
  %3703 = load double, ptr %3702, align 8
  store double %3703, ptr %213, align 8
  br i1 true, label %3716, label %3749

3704:                                             ; preds = %3694
  %3705 = load ptr, ptr %207, align 8
  %3706 = getelementptr inbounds %struct.TValue, ptr %3705, i32 0, i32 1
  %3707 = load i8, ptr %3706, align 8
  %3708 = zext i8 %3707 to i32
  %3709 = icmp eq i32 %3708, 3
  br i1 %3709, label %3710, label %3715

3710:                                             ; preds = %3704
  %3711 = load ptr, ptr %207, align 8
  %3712 = getelementptr inbounds %struct.TValue, ptr %3711, i32 0, i32 0
  %3713 = load i64, ptr %3712, align 8
  %3714 = sitofp i64 %3713 to double
  store double %3714, ptr %213, align 8
  br i1 true, label %3716, label %3749

3715:                                             ; preds = %3704
  br i1 false, label %3716, label %3749

3716:                                             ; preds = %3715, %3710, %3700
  %3717 = load ptr, ptr %208, align 8
  %3718 = getelementptr inbounds %struct.TValue, ptr %3717, i32 0, i32 1
  %3719 = load i8, ptr %3718, align 8
  %3720 = zext i8 %3719 to i32
  %3721 = icmp eq i32 %3720, 19
  br i1 %3721, label %3722, label %3726

3722:                                             ; preds = %3716
  %3723 = load ptr, ptr %208, align 8
  %3724 = getelementptr inbounds %struct.TValue, ptr %3723, i32 0, i32 0
  %3725 = load double, ptr %3724, align 8
  store double %3725, ptr %214, align 8
  br i1 true, label %3738, label %3749

3726:                                             ; preds = %3716
  %3727 = load ptr, ptr %208, align 8
  %3728 = getelementptr inbounds %struct.TValue, ptr %3727, i32 0, i32 1
  %3729 = load i8, ptr %3728, align 8
  %3730 = zext i8 %3729 to i32
  %3731 = icmp eq i32 %3730, 3
  br i1 %3731, label %3732, label %3737

3732:                                             ; preds = %3726
  %3733 = load ptr, ptr %208, align 8
  %3734 = getelementptr inbounds %struct.TValue, ptr %3733, i32 0, i32 0
  %3735 = load i64, ptr %3734, align 8
  %3736 = sitofp i64 %3735 to double
  store double %3736, ptr %214, align 8
  br i1 true, label %3738, label %3749

3737:                                             ; preds = %3726
  br i1 false, label %3738, label %3749

3738:                                             ; preds = %3737, %3732, %3722
  %3739 = load ptr, ptr %8, align 8
  %3740 = getelementptr inbounds i32, ptr %3739, i32 1
  store ptr %3740, ptr %8, align 8
  %3741 = load ptr, ptr %209, align 8
  store ptr %3741, ptr %215, align 8
  %3742 = load double, ptr %213, align 8
  %3743 = load double, ptr %214, align 8
  %3744 = fadd double %3742, %3743
  %3745 = load ptr, ptr %215, align 8
  %3746 = getelementptr inbounds %struct.TValue, ptr %3745, i32 0, i32 0
  store double %3744, ptr %3746, align 8
  %3747 = load ptr, ptr %215, align 8
  %3748 = getelementptr inbounds %struct.TValue, ptr %3747, i32 0, i32 1
  store i8 19, ptr %3748, align 8
  br label %3749

3749:                                             ; preds = %3738, %3737, %3732, %3722, %3715, %3710, %3700
  br label %3750

3750:                                             ; preds = %3749, %3677
  %3751 = load i32, ptr %9, align 4
  %3752 = icmp ne i32 %3751, 0
  %3753 = zext i1 %3752 to i32
  %3754 = sext i32 %3753 to i64
  %3755 = icmp ne i64 %3754, 0
  br i1 %3755, label %3756, label %3764

3756:                                             ; preds = %3750
  %3757 = load ptr, ptr %3, align 8
  %3758 = load ptr, ptr %8, align 8
  %3759 = call i32 @luaG_traceexec(ptr noundef %3757, ptr noundef %3758)
  store i32 %3759, ptr %9, align 4
  %3760 = load ptr, ptr %4, align 8
  %3761 = getelementptr inbounds %struct.CallInfo, ptr %3760, i32 0, i32 0
  %3762 = load ptr, ptr %3761, align 8
  %3763 = getelementptr inbounds %union.StackValue, ptr %3762, i64 1
  store ptr %3763, ptr %7, align 8
  br label %3764

3764:                                             ; preds = %3756, %3750
  %3765 = load ptr, ptr %8, align 8
  %3766 = getelementptr inbounds i32, ptr %3765, i32 1
  store ptr %3766, ptr %8, align 8
  %3767 = load i32, ptr %3765, align 4
  store i32 %3767, ptr %10, align 4
  %3768 = load i32, ptr %10, align 4
  %3769 = lshr i32 %3768, 0
  %3770 = and i32 %3769, 127
  %3771 = zext i32 %3770 to i64
  %3772 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3771
  %3773 = load ptr, ptr %3772, align 8
  br label %7120

3774:                                             ; preds = %7120
  %3775 = load ptr, ptr %7, align 8
  %3776 = load i32, ptr %10, align 4
  %3777 = lshr i32 %3776, 16
  %3778 = and i32 %3777, 255
  %3779 = sext i32 %3778 to i64
  %3780 = getelementptr inbounds %union.StackValue, ptr %3775, i64 %3779
  store ptr %3780, ptr %216, align 8
  %3781 = load ptr, ptr %7, align 8
  %3782 = load i32, ptr %10, align 4
  %3783 = lshr i32 %3782, 24
  %3784 = and i32 %3783, 255
  %3785 = sext i32 %3784 to i64
  %3786 = getelementptr inbounds %union.StackValue, ptr %3781, i64 %3785
  store ptr %3786, ptr %217, align 8
  %3787 = load ptr, ptr %7, align 8
  %3788 = load i32, ptr %10, align 4
  %3789 = lshr i32 %3788, 7
  %3790 = and i32 %3789, 255
  %3791 = sext i32 %3790 to i64
  %3792 = getelementptr inbounds %union.StackValue, ptr %3787, i64 %3791
  store ptr %3792, ptr %218, align 8
  %3793 = load ptr, ptr %216, align 8
  %3794 = getelementptr inbounds %struct.TValue, ptr %3793, i32 0, i32 1
  %3795 = load i8, ptr %3794, align 8
  %3796 = zext i8 %3795 to i32
  %3797 = icmp eq i32 %3796, 3
  br i1 %3797, label %3798, label %3821

3798:                                             ; preds = %3774
  %3799 = load ptr, ptr %217, align 8
  %3800 = getelementptr inbounds %struct.TValue, ptr %3799, i32 0, i32 1
  %3801 = load i8, ptr %3800, align 8
  %3802 = zext i8 %3801 to i32
  %3803 = icmp eq i32 %3802, 3
  br i1 %3803, label %3804, label %3821

3804:                                             ; preds = %3798
  %3805 = load ptr, ptr %216, align 8
  %3806 = getelementptr inbounds %struct.TValue, ptr %3805, i32 0, i32 0
  %3807 = load i64, ptr %3806, align 8
  store i64 %3807, ptr %219, align 8
  %3808 = load ptr, ptr %217, align 8
  %3809 = getelementptr inbounds %struct.TValue, ptr %3808, i32 0, i32 0
  %3810 = load i64, ptr %3809, align 8
  store i64 %3810, ptr %220, align 8
  %3811 = load ptr, ptr %8, align 8
  %3812 = getelementptr inbounds i32, ptr %3811, i32 1
  store ptr %3812, ptr %8, align 8
  %3813 = load ptr, ptr %218, align 8
  store ptr %3813, ptr %221, align 8
  %3814 = load i64, ptr %219, align 8
  %3815 = load i64, ptr %220, align 8
  %3816 = sub i64 %3814, %3815
  %3817 = load ptr, ptr %221, align 8
  %3818 = getelementptr inbounds %struct.TValue, ptr %3817, i32 0, i32 0
  store i64 %3816, ptr %3818, align 8
  %3819 = load ptr, ptr %221, align 8
  %3820 = getelementptr inbounds %struct.TValue, ptr %3819, i32 0, i32 1
  store i8 3, ptr %3820, align 8
  br label %3877

3821:                                             ; preds = %3798, %3774
  %3822 = load ptr, ptr %216, align 8
  %3823 = getelementptr inbounds %struct.TValue, ptr %3822, i32 0, i32 1
  %3824 = load i8, ptr %3823, align 8
  %3825 = zext i8 %3824 to i32
  %3826 = icmp eq i32 %3825, 19
  br i1 %3826, label %3827, label %3831

3827:                                             ; preds = %3821
  %3828 = load ptr, ptr %216, align 8
  %3829 = getelementptr inbounds %struct.TValue, ptr %3828, i32 0, i32 0
  %3830 = load double, ptr %3829, align 8
  store double %3830, ptr %222, align 8
  br i1 true, label %3843, label %3876

3831:                                             ; preds = %3821
  %3832 = load ptr, ptr %216, align 8
  %3833 = getelementptr inbounds %struct.TValue, ptr %3832, i32 0, i32 1
  %3834 = load i8, ptr %3833, align 8
  %3835 = zext i8 %3834 to i32
  %3836 = icmp eq i32 %3835, 3
  br i1 %3836, label %3837, label %3842

3837:                                             ; preds = %3831
  %3838 = load ptr, ptr %216, align 8
  %3839 = getelementptr inbounds %struct.TValue, ptr %3838, i32 0, i32 0
  %3840 = load i64, ptr %3839, align 8
  %3841 = sitofp i64 %3840 to double
  store double %3841, ptr %222, align 8
  br i1 true, label %3843, label %3876

3842:                                             ; preds = %3831
  br i1 false, label %3843, label %3876

3843:                                             ; preds = %3842, %3837, %3827
  %3844 = load ptr, ptr %217, align 8
  %3845 = getelementptr inbounds %struct.TValue, ptr %3844, i32 0, i32 1
  %3846 = load i8, ptr %3845, align 8
  %3847 = zext i8 %3846 to i32
  %3848 = icmp eq i32 %3847, 19
  br i1 %3848, label %3849, label %3853

3849:                                             ; preds = %3843
  %3850 = load ptr, ptr %217, align 8
  %3851 = getelementptr inbounds %struct.TValue, ptr %3850, i32 0, i32 0
  %3852 = load double, ptr %3851, align 8
  store double %3852, ptr %223, align 8
  br i1 true, label %3865, label %3876

3853:                                             ; preds = %3843
  %3854 = load ptr, ptr %217, align 8
  %3855 = getelementptr inbounds %struct.TValue, ptr %3854, i32 0, i32 1
  %3856 = load i8, ptr %3855, align 8
  %3857 = zext i8 %3856 to i32
  %3858 = icmp eq i32 %3857, 3
  br i1 %3858, label %3859, label %3864

3859:                                             ; preds = %3853
  %3860 = load ptr, ptr %217, align 8
  %3861 = getelementptr inbounds %struct.TValue, ptr %3860, i32 0, i32 0
  %3862 = load i64, ptr %3861, align 8
  %3863 = sitofp i64 %3862 to double
  store double %3863, ptr %223, align 8
  br i1 true, label %3865, label %3876

3864:                                             ; preds = %3853
  br i1 false, label %3865, label %3876

3865:                                             ; preds = %3864, %3859, %3849
  %3866 = load ptr, ptr %8, align 8
  %3867 = getelementptr inbounds i32, ptr %3866, i32 1
  store ptr %3867, ptr %8, align 8
  %3868 = load ptr, ptr %218, align 8
  store ptr %3868, ptr %224, align 8
  %3869 = load double, ptr %222, align 8
  %3870 = load double, ptr %223, align 8
  %3871 = fsub double %3869, %3870
  %3872 = load ptr, ptr %224, align 8
  %3873 = getelementptr inbounds %struct.TValue, ptr %3872, i32 0, i32 0
  store double %3871, ptr %3873, align 8
  %3874 = load ptr, ptr %224, align 8
  %3875 = getelementptr inbounds %struct.TValue, ptr %3874, i32 0, i32 1
  store i8 19, ptr %3875, align 8
  br label %3876

3876:                                             ; preds = %3865, %3864, %3859, %3849, %3842, %3837, %3827
  br label %3877

3877:                                             ; preds = %3876, %3804
  %3878 = load i32, ptr %9, align 4
  %3879 = icmp ne i32 %3878, 0
  %3880 = zext i1 %3879 to i32
  %3881 = sext i32 %3880 to i64
  %3882 = icmp ne i64 %3881, 0
  br i1 %3882, label %3883, label %3891

3883:                                             ; preds = %3877
  %3884 = load ptr, ptr %3, align 8
  %3885 = load ptr, ptr %8, align 8
  %3886 = call i32 @luaG_traceexec(ptr noundef %3884, ptr noundef %3885)
  store i32 %3886, ptr %9, align 4
  %3887 = load ptr, ptr %4, align 8
  %3888 = getelementptr inbounds %struct.CallInfo, ptr %3887, i32 0, i32 0
  %3889 = load ptr, ptr %3888, align 8
  %3890 = getelementptr inbounds %union.StackValue, ptr %3889, i64 1
  store ptr %3890, ptr %7, align 8
  br label %3891

3891:                                             ; preds = %3883, %3877
  %3892 = load ptr, ptr %8, align 8
  %3893 = getelementptr inbounds i32, ptr %3892, i32 1
  store ptr %3893, ptr %8, align 8
  %3894 = load i32, ptr %3892, align 4
  store i32 %3894, ptr %10, align 4
  %3895 = load i32, ptr %10, align 4
  %3896 = lshr i32 %3895, 0
  %3897 = and i32 %3896, 127
  %3898 = zext i32 %3897 to i64
  %3899 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %3898
  %3900 = load ptr, ptr %3899, align 8
  br label %7120

3901:                                             ; preds = %7120
  %3902 = load ptr, ptr %7, align 8
  %3903 = load i32, ptr %10, align 4
  %3904 = lshr i32 %3903, 16
  %3905 = and i32 %3904, 255
  %3906 = sext i32 %3905 to i64
  %3907 = getelementptr inbounds %union.StackValue, ptr %3902, i64 %3906
  store ptr %3907, ptr %225, align 8
  %3908 = load ptr, ptr %7, align 8
  %3909 = load i32, ptr %10, align 4
  %3910 = lshr i32 %3909, 24
  %3911 = and i32 %3910, 255
  %3912 = sext i32 %3911 to i64
  %3913 = getelementptr inbounds %union.StackValue, ptr %3908, i64 %3912
  store ptr %3913, ptr %226, align 8
  %3914 = load ptr, ptr %7, align 8
  %3915 = load i32, ptr %10, align 4
  %3916 = lshr i32 %3915, 7
  %3917 = and i32 %3916, 255
  %3918 = sext i32 %3917 to i64
  %3919 = getelementptr inbounds %union.StackValue, ptr %3914, i64 %3918
  store ptr %3919, ptr %227, align 8
  %3920 = load ptr, ptr %225, align 8
  %3921 = getelementptr inbounds %struct.TValue, ptr %3920, i32 0, i32 1
  %3922 = load i8, ptr %3921, align 8
  %3923 = zext i8 %3922 to i32
  %3924 = icmp eq i32 %3923, 3
  br i1 %3924, label %3925, label %3948

3925:                                             ; preds = %3901
  %3926 = load ptr, ptr %226, align 8
  %3927 = getelementptr inbounds %struct.TValue, ptr %3926, i32 0, i32 1
  %3928 = load i8, ptr %3927, align 8
  %3929 = zext i8 %3928 to i32
  %3930 = icmp eq i32 %3929, 3
  br i1 %3930, label %3931, label %3948

3931:                                             ; preds = %3925
  %3932 = load ptr, ptr %225, align 8
  %3933 = getelementptr inbounds %struct.TValue, ptr %3932, i32 0, i32 0
  %3934 = load i64, ptr %3933, align 8
  store i64 %3934, ptr %228, align 8
  %3935 = load ptr, ptr %226, align 8
  %3936 = getelementptr inbounds %struct.TValue, ptr %3935, i32 0, i32 0
  %3937 = load i64, ptr %3936, align 8
  store i64 %3937, ptr %229, align 8
  %3938 = load ptr, ptr %8, align 8
  %3939 = getelementptr inbounds i32, ptr %3938, i32 1
  store ptr %3939, ptr %8, align 8
  %3940 = load ptr, ptr %227, align 8
  store ptr %3940, ptr %230, align 8
  %3941 = load i64, ptr %228, align 8
  %3942 = load i64, ptr %229, align 8
  %3943 = mul i64 %3941, %3942
  %3944 = load ptr, ptr %230, align 8
  %3945 = getelementptr inbounds %struct.TValue, ptr %3944, i32 0, i32 0
  store i64 %3943, ptr %3945, align 8
  %3946 = load ptr, ptr %230, align 8
  %3947 = getelementptr inbounds %struct.TValue, ptr %3946, i32 0, i32 1
  store i8 3, ptr %3947, align 8
  br label %4004

3948:                                             ; preds = %3925, %3901
  %3949 = load ptr, ptr %225, align 8
  %3950 = getelementptr inbounds %struct.TValue, ptr %3949, i32 0, i32 1
  %3951 = load i8, ptr %3950, align 8
  %3952 = zext i8 %3951 to i32
  %3953 = icmp eq i32 %3952, 19
  br i1 %3953, label %3954, label %3958

3954:                                             ; preds = %3948
  %3955 = load ptr, ptr %225, align 8
  %3956 = getelementptr inbounds %struct.TValue, ptr %3955, i32 0, i32 0
  %3957 = load double, ptr %3956, align 8
  store double %3957, ptr %231, align 8
  br i1 true, label %3970, label %4003

3958:                                             ; preds = %3948
  %3959 = load ptr, ptr %225, align 8
  %3960 = getelementptr inbounds %struct.TValue, ptr %3959, i32 0, i32 1
  %3961 = load i8, ptr %3960, align 8
  %3962 = zext i8 %3961 to i32
  %3963 = icmp eq i32 %3962, 3
  br i1 %3963, label %3964, label %3969

3964:                                             ; preds = %3958
  %3965 = load ptr, ptr %225, align 8
  %3966 = getelementptr inbounds %struct.TValue, ptr %3965, i32 0, i32 0
  %3967 = load i64, ptr %3966, align 8
  %3968 = sitofp i64 %3967 to double
  store double %3968, ptr %231, align 8
  br i1 true, label %3970, label %4003

3969:                                             ; preds = %3958
  br i1 false, label %3970, label %4003

3970:                                             ; preds = %3969, %3964, %3954
  %3971 = load ptr, ptr %226, align 8
  %3972 = getelementptr inbounds %struct.TValue, ptr %3971, i32 0, i32 1
  %3973 = load i8, ptr %3972, align 8
  %3974 = zext i8 %3973 to i32
  %3975 = icmp eq i32 %3974, 19
  br i1 %3975, label %3976, label %3980

3976:                                             ; preds = %3970
  %3977 = load ptr, ptr %226, align 8
  %3978 = getelementptr inbounds %struct.TValue, ptr %3977, i32 0, i32 0
  %3979 = load double, ptr %3978, align 8
  store double %3979, ptr %232, align 8
  br i1 true, label %3992, label %4003

3980:                                             ; preds = %3970
  %3981 = load ptr, ptr %226, align 8
  %3982 = getelementptr inbounds %struct.TValue, ptr %3981, i32 0, i32 1
  %3983 = load i8, ptr %3982, align 8
  %3984 = zext i8 %3983 to i32
  %3985 = icmp eq i32 %3984, 3
  br i1 %3985, label %3986, label %3991

3986:                                             ; preds = %3980
  %3987 = load ptr, ptr %226, align 8
  %3988 = getelementptr inbounds %struct.TValue, ptr %3987, i32 0, i32 0
  %3989 = load i64, ptr %3988, align 8
  %3990 = sitofp i64 %3989 to double
  store double %3990, ptr %232, align 8
  br i1 true, label %3992, label %4003

3991:                                             ; preds = %3980
  br i1 false, label %3992, label %4003

3992:                                             ; preds = %3991, %3986, %3976
  %3993 = load ptr, ptr %8, align 8
  %3994 = getelementptr inbounds i32, ptr %3993, i32 1
  store ptr %3994, ptr %8, align 8
  %3995 = load ptr, ptr %227, align 8
  store ptr %3995, ptr %233, align 8
  %3996 = load double, ptr %231, align 8
  %3997 = load double, ptr %232, align 8
  %3998 = fmul double %3996, %3997
  %3999 = load ptr, ptr %233, align 8
  %4000 = getelementptr inbounds %struct.TValue, ptr %3999, i32 0, i32 0
  store double %3998, ptr %4000, align 8
  %4001 = load ptr, ptr %233, align 8
  %4002 = getelementptr inbounds %struct.TValue, ptr %4001, i32 0, i32 1
  store i8 19, ptr %4002, align 8
  br label %4003

4003:                                             ; preds = %3992, %3991, %3986, %3976, %3969, %3964, %3954
  br label %4004

4004:                                             ; preds = %4003, %3931
  %4005 = load i32, ptr %9, align 4
  %4006 = icmp ne i32 %4005, 0
  %4007 = zext i1 %4006 to i32
  %4008 = sext i32 %4007 to i64
  %4009 = icmp ne i64 %4008, 0
  br i1 %4009, label %4010, label %4018

4010:                                             ; preds = %4004
  %4011 = load ptr, ptr %3, align 8
  %4012 = load ptr, ptr %8, align 8
  %4013 = call i32 @luaG_traceexec(ptr noundef %4011, ptr noundef %4012)
  store i32 %4013, ptr %9, align 4
  %4014 = load ptr, ptr %4, align 8
  %4015 = getelementptr inbounds %struct.CallInfo, ptr %4014, i32 0, i32 0
  %4016 = load ptr, ptr %4015, align 8
  %4017 = getelementptr inbounds %union.StackValue, ptr %4016, i64 1
  store ptr %4017, ptr %7, align 8
  br label %4018

4018:                                             ; preds = %4010, %4004
  %4019 = load ptr, ptr %8, align 8
  %4020 = getelementptr inbounds i32, ptr %4019, i32 1
  store ptr %4020, ptr %8, align 8
  %4021 = load i32, ptr %4019, align 4
  store i32 %4021, ptr %10, align 4
  %4022 = load i32, ptr %10, align 4
  %4023 = lshr i32 %4022, 0
  %4024 = and i32 %4023, 127
  %4025 = zext i32 %4024 to i64
  %4026 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4025
  %4027 = load ptr, ptr %4026, align 8
  br label %7120

4028:                                             ; preds = %7120
  %4029 = load ptr, ptr %8, align 8
  %4030 = load ptr, ptr %4, align 8
  %4031 = getelementptr inbounds %struct.CallInfo, ptr %4030, i32 0, i32 4
  %4032 = getelementptr inbounds %struct.anon, ptr %4031, i32 0, i32 0
  store ptr %4029, ptr %4032, align 8
  %4033 = load ptr, ptr %4, align 8
  %4034 = getelementptr inbounds %struct.CallInfo, ptr %4033, i32 0, i32 1
  %4035 = load ptr, ptr %4034, align 8
  %4036 = load ptr, ptr %3, align 8
  %4037 = getelementptr inbounds %struct.lua_State, ptr %4036, i32 0, i32 6
  store ptr %4035, ptr %4037, align 8
  %4038 = load ptr, ptr %7, align 8
  %4039 = load i32, ptr %10, align 4
  %4040 = lshr i32 %4039, 16
  %4041 = and i32 %4040, 255
  %4042 = sext i32 %4041 to i64
  %4043 = getelementptr inbounds %union.StackValue, ptr %4038, i64 %4042
  store ptr %4043, ptr %234, align 8
  %4044 = load ptr, ptr %7, align 8
  %4045 = load i32, ptr %10, align 4
  %4046 = lshr i32 %4045, 24
  %4047 = and i32 %4046, 255
  %4048 = sext i32 %4047 to i64
  %4049 = getelementptr inbounds %union.StackValue, ptr %4044, i64 %4048
  store ptr %4049, ptr %235, align 8
  %4050 = load ptr, ptr %7, align 8
  %4051 = load i32, ptr %10, align 4
  %4052 = lshr i32 %4051, 7
  %4053 = and i32 %4052, 255
  %4054 = sext i32 %4053 to i64
  %4055 = getelementptr inbounds %union.StackValue, ptr %4050, i64 %4054
  store ptr %4055, ptr %236, align 8
  %4056 = load ptr, ptr %234, align 8
  %4057 = getelementptr inbounds %struct.TValue, ptr %4056, i32 0, i32 1
  %4058 = load i8, ptr %4057, align 8
  %4059 = zext i8 %4058 to i32
  %4060 = icmp eq i32 %4059, 3
  br i1 %4060, label %4061, label %4085

4061:                                             ; preds = %4028
  %4062 = load ptr, ptr %235, align 8
  %4063 = getelementptr inbounds %struct.TValue, ptr %4062, i32 0, i32 1
  %4064 = load i8, ptr %4063, align 8
  %4065 = zext i8 %4064 to i32
  %4066 = icmp eq i32 %4065, 3
  br i1 %4066, label %4067, label %4085

4067:                                             ; preds = %4061
  %4068 = load ptr, ptr %234, align 8
  %4069 = getelementptr inbounds %struct.TValue, ptr %4068, i32 0, i32 0
  %4070 = load i64, ptr %4069, align 8
  store i64 %4070, ptr %237, align 8
  %4071 = load ptr, ptr %235, align 8
  %4072 = getelementptr inbounds %struct.TValue, ptr %4071, i32 0, i32 0
  %4073 = load i64, ptr %4072, align 8
  store i64 %4073, ptr %238, align 8
  %4074 = load ptr, ptr %8, align 8
  %4075 = getelementptr inbounds i32, ptr %4074, i32 1
  store ptr %4075, ptr %8, align 8
  %4076 = load ptr, ptr %236, align 8
  store ptr %4076, ptr %239, align 8
  %4077 = load ptr, ptr %3, align 8
  %4078 = load i64, ptr %237, align 8
  %4079 = load i64, ptr %238, align 8
  %4080 = call i64 @luaV_mod(ptr noundef %4077, i64 noundef %4078, i64 noundef %4079)
  %4081 = load ptr, ptr %239, align 8
  %4082 = getelementptr inbounds %struct.TValue, ptr %4081, i32 0, i32 0
  store i64 %4080, ptr %4082, align 8
  %4083 = load ptr, ptr %239, align 8
  %4084 = getelementptr inbounds %struct.TValue, ptr %4083, i32 0, i32 1
  store i8 3, ptr %4084, align 8
  br label %4142

4085:                                             ; preds = %4061, %4028
  %4086 = load ptr, ptr %234, align 8
  %4087 = getelementptr inbounds %struct.TValue, ptr %4086, i32 0, i32 1
  %4088 = load i8, ptr %4087, align 8
  %4089 = zext i8 %4088 to i32
  %4090 = icmp eq i32 %4089, 19
  br i1 %4090, label %4091, label %4095

4091:                                             ; preds = %4085
  %4092 = load ptr, ptr %234, align 8
  %4093 = getelementptr inbounds %struct.TValue, ptr %4092, i32 0, i32 0
  %4094 = load double, ptr %4093, align 8
  store double %4094, ptr %240, align 8
  br i1 true, label %4107, label %4141

4095:                                             ; preds = %4085
  %4096 = load ptr, ptr %234, align 8
  %4097 = getelementptr inbounds %struct.TValue, ptr %4096, i32 0, i32 1
  %4098 = load i8, ptr %4097, align 8
  %4099 = zext i8 %4098 to i32
  %4100 = icmp eq i32 %4099, 3
  br i1 %4100, label %4101, label %4106

4101:                                             ; preds = %4095
  %4102 = load ptr, ptr %234, align 8
  %4103 = getelementptr inbounds %struct.TValue, ptr %4102, i32 0, i32 0
  %4104 = load i64, ptr %4103, align 8
  %4105 = sitofp i64 %4104 to double
  store double %4105, ptr %240, align 8
  br i1 true, label %4107, label %4141

4106:                                             ; preds = %4095
  br i1 false, label %4107, label %4141

4107:                                             ; preds = %4106, %4101, %4091
  %4108 = load ptr, ptr %235, align 8
  %4109 = getelementptr inbounds %struct.TValue, ptr %4108, i32 0, i32 1
  %4110 = load i8, ptr %4109, align 8
  %4111 = zext i8 %4110 to i32
  %4112 = icmp eq i32 %4111, 19
  br i1 %4112, label %4113, label %4117

4113:                                             ; preds = %4107
  %4114 = load ptr, ptr %235, align 8
  %4115 = getelementptr inbounds %struct.TValue, ptr %4114, i32 0, i32 0
  %4116 = load double, ptr %4115, align 8
  store double %4116, ptr %241, align 8
  br i1 true, label %4129, label %4141

4117:                                             ; preds = %4107
  %4118 = load ptr, ptr %235, align 8
  %4119 = getelementptr inbounds %struct.TValue, ptr %4118, i32 0, i32 1
  %4120 = load i8, ptr %4119, align 8
  %4121 = zext i8 %4120 to i32
  %4122 = icmp eq i32 %4121, 3
  br i1 %4122, label %4123, label %4128

4123:                                             ; preds = %4117
  %4124 = load ptr, ptr %235, align 8
  %4125 = getelementptr inbounds %struct.TValue, ptr %4124, i32 0, i32 0
  %4126 = load i64, ptr %4125, align 8
  %4127 = sitofp i64 %4126 to double
  store double %4127, ptr %241, align 8
  br i1 true, label %4129, label %4141

4128:                                             ; preds = %4117
  br i1 false, label %4129, label %4141

4129:                                             ; preds = %4128, %4123, %4113
  %4130 = load ptr, ptr %8, align 8
  %4131 = getelementptr inbounds i32, ptr %4130, i32 1
  store ptr %4131, ptr %8, align 8
  %4132 = load ptr, ptr %236, align 8
  store ptr %4132, ptr %242, align 8
  %4133 = load ptr, ptr %3, align 8
  %4134 = load double, ptr %240, align 8
  %4135 = load double, ptr %241, align 8
  %4136 = call double @luaV_modf(ptr noundef %4133, double noundef %4134, double noundef %4135)
  %4137 = load ptr, ptr %242, align 8
  %4138 = getelementptr inbounds %struct.TValue, ptr %4137, i32 0, i32 0
  store double %4136, ptr %4138, align 8
  %4139 = load ptr, ptr %242, align 8
  %4140 = getelementptr inbounds %struct.TValue, ptr %4139, i32 0, i32 1
  store i8 19, ptr %4140, align 8
  br label %4141

4141:                                             ; preds = %4129, %4128, %4123, %4113, %4106, %4101, %4091
  br label %4142

4142:                                             ; preds = %4141, %4067
  %4143 = load i32, ptr %9, align 4
  %4144 = icmp ne i32 %4143, 0
  %4145 = zext i1 %4144 to i32
  %4146 = sext i32 %4145 to i64
  %4147 = icmp ne i64 %4146, 0
  br i1 %4147, label %4148, label %4156

4148:                                             ; preds = %4142
  %4149 = load ptr, ptr %3, align 8
  %4150 = load ptr, ptr %8, align 8
  %4151 = call i32 @luaG_traceexec(ptr noundef %4149, ptr noundef %4150)
  store i32 %4151, ptr %9, align 4
  %4152 = load ptr, ptr %4, align 8
  %4153 = getelementptr inbounds %struct.CallInfo, ptr %4152, i32 0, i32 0
  %4154 = load ptr, ptr %4153, align 8
  %4155 = getelementptr inbounds %union.StackValue, ptr %4154, i64 1
  store ptr %4155, ptr %7, align 8
  br label %4156

4156:                                             ; preds = %4148, %4142
  %4157 = load ptr, ptr %8, align 8
  %4158 = getelementptr inbounds i32, ptr %4157, i32 1
  store ptr %4158, ptr %8, align 8
  %4159 = load i32, ptr %4157, align 4
  store i32 %4159, ptr %10, align 4
  %4160 = load i32, ptr %10, align 4
  %4161 = lshr i32 %4160, 0
  %4162 = and i32 %4161, 127
  %4163 = zext i32 %4162 to i64
  %4164 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4163
  %4165 = load ptr, ptr %4164, align 8
  br label %7120

4166:                                             ; preds = %7120
  %4167 = load ptr, ptr %7, align 8
  %4168 = load i32, ptr %10, align 4
  %4169 = lshr i32 %4168, 7
  %4170 = and i32 %4169, 255
  %4171 = sext i32 %4170 to i64
  %4172 = getelementptr inbounds %union.StackValue, ptr %4167, i64 %4171
  store ptr %4172, ptr %243, align 8
  %4173 = load ptr, ptr %7, align 8
  %4174 = load i32, ptr %10, align 4
  %4175 = lshr i32 %4174, 16
  %4176 = and i32 %4175, 255
  %4177 = sext i32 %4176 to i64
  %4178 = getelementptr inbounds %union.StackValue, ptr %4173, i64 %4177
  store ptr %4178, ptr %244, align 8
  %4179 = load ptr, ptr %7, align 8
  %4180 = load i32, ptr %10, align 4
  %4181 = lshr i32 %4180, 24
  %4182 = and i32 %4181, 255
  %4183 = sext i32 %4182 to i64
  %4184 = getelementptr inbounds %union.StackValue, ptr %4179, i64 %4183
  store ptr %4184, ptr %245, align 8
  %4185 = load ptr, ptr %244, align 8
  %4186 = getelementptr inbounds %struct.TValue, ptr %4185, i32 0, i32 1
  %4187 = load i8, ptr %4186, align 8
  %4188 = zext i8 %4187 to i32
  %4189 = icmp eq i32 %4188, 19
  br i1 %4189, label %4190, label %4194

4190:                                             ; preds = %4166
  %4191 = load ptr, ptr %244, align 8
  %4192 = getelementptr inbounds %struct.TValue, ptr %4191, i32 0, i32 0
  %4193 = load double, ptr %4192, align 8
  store double %4193, ptr %246, align 8
  br i1 true, label %4206, label %4249

4194:                                             ; preds = %4166
  %4195 = load ptr, ptr %244, align 8
  %4196 = getelementptr inbounds %struct.TValue, ptr %4195, i32 0, i32 1
  %4197 = load i8, ptr %4196, align 8
  %4198 = zext i8 %4197 to i32
  %4199 = icmp eq i32 %4198, 3
  br i1 %4199, label %4200, label %4205

4200:                                             ; preds = %4194
  %4201 = load ptr, ptr %244, align 8
  %4202 = getelementptr inbounds %struct.TValue, ptr %4201, i32 0, i32 0
  %4203 = load i64, ptr %4202, align 8
  %4204 = sitofp i64 %4203 to double
  store double %4204, ptr %246, align 8
  br i1 true, label %4206, label %4249

4205:                                             ; preds = %4194
  br i1 false, label %4206, label %4249

4206:                                             ; preds = %4205, %4200, %4190
  %4207 = load ptr, ptr %245, align 8
  %4208 = getelementptr inbounds %struct.TValue, ptr %4207, i32 0, i32 1
  %4209 = load i8, ptr %4208, align 8
  %4210 = zext i8 %4209 to i32
  %4211 = icmp eq i32 %4210, 19
  br i1 %4211, label %4212, label %4216

4212:                                             ; preds = %4206
  %4213 = load ptr, ptr %245, align 8
  %4214 = getelementptr inbounds %struct.TValue, ptr %4213, i32 0, i32 0
  %4215 = load double, ptr %4214, align 8
  store double %4215, ptr %247, align 8
  br i1 true, label %4228, label %4249

4216:                                             ; preds = %4206
  %4217 = load ptr, ptr %245, align 8
  %4218 = getelementptr inbounds %struct.TValue, ptr %4217, i32 0, i32 1
  %4219 = load i8, ptr %4218, align 8
  %4220 = zext i8 %4219 to i32
  %4221 = icmp eq i32 %4220, 3
  br i1 %4221, label %4222, label %4227

4222:                                             ; preds = %4216
  %4223 = load ptr, ptr %245, align 8
  %4224 = getelementptr inbounds %struct.TValue, ptr %4223, i32 0, i32 0
  %4225 = load i64, ptr %4224, align 8
  %4226 = sitofp i64 %4225 to double
  store double %4226, ptr %247, align 8
  br i1 true, label %4228, label %4249

4227:                                             ; preds = %4216
  br i1 false, label %4228, label %4249

4228:                                             ; preds = %4227, %4222, %4212
  %4229 = load ptr, ptr %8, align 8
  %4230 = getelementptr inbounds i32, ptr %4229, i32 1
  store ptr %4230, ptr %8, align 8
  %4231 = load ptr, ptr %243, align 8
  store ptr %4231, ptr %248, align 8
  %4232 = load ptr, ptr %3, align 8
  %4233 = load double, ptr %247, align 8
  %4234 = fcmp oeq double %4233, 2.000000e+00
  br i1 %4234, label %4235, label %4239

4235:                                             ; preds = %4228
  %4236 = load double, ptr %246, align 8
  %4237 = load double, ptr %246, align 8
  %4238 = fmul double %4236, %4237
  br label %4243

4239:                                             ; preds = %4228
  %4240 = load double, ptr %246, align 8
  %4241 = load double, ptr %247, align 8
  %4242 = call double @pow(double noundef %4240, double noundef %4241) #8
  br label %4243

4243:                                             ; preds = %4239, %4235
  %4244 = phi double [ %4238, %4235 ], [ %4242, %4239 ]
  %4245 = load ptr, ptr %248, align 8
  %4246 = getelementptr inbounds %struct.TValue, ptr %4245, i32 0, i32 0
  store double %4244, ptr %4246, align 8
  %4247 = load ptr, ptr %248, align 8
  %4248 = getelementptr inbounds %struct.TValue, ptr %4247, i32 0, i32 1
  store i8 19, ptr %4248, align 8
  br label %4249

4249:                                             ; preds = %4243, %4227, %4222, %4212, %4205, %4200, %4190
  %4250 = load i32, ptr %9, align 4
  %4251 = icmp ne i32 %4250, 0
  %4252 = zext i1 %4251 to i32
  %4253 = sext i32 %4252 to i64
  %4254 = icmp ne i64 %4253, 0
  br i1 %4254, label %4255, label %4263

4255:                                             ; preds = %4249
  %4256 = load ptr, ptr %3, align 8
  %4257 = load ptr, ptr %8, align 8
  %4258 = call i32 @luaG_traceexec(ptr noundef %4256, ptr noundef %4257)
  store i32 %4258, ptr %9, align 4
  %4259 = load ptr, ptr %4, align 8
  %4260 = getelementptr inbounds %struct.CallInfo, ptr %4259, i32 0, i32 0
  %4261 = load ptr, ptr %4260, align 8
  %4262 = getelementptr inbounds %union.StackValue, ptr %4261, i64 1
  store ptr %4262, ptr %7, align 8
  br label %4263

4263:                                             ; preds = %4255, %4249
  %4264 = load ptr, ptr %8, align 8
  %4265 = getelementptr inbounds i32, ptr %4264, i32 1
  store ptr %4265, ptr %8, align 8
  %4266 = load i32, ptr %4264, align 4
  store i32 %4266, ptr %10, align 4
  %4267 = load i32, ptr %10, align 4
  %4268 = lshr i32 %4267, 0
  %4269 = and i32 %4268, 127
  %4270 = zext i32 %4269 to i64
  %4271 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4270
  %4272 = load ptr, ptr %4271, align 8
  br label %7120

4273:                                             ; preds = %7120
  %4274 = load ptr, ptr %7, align 8
  %4275 = load i32, ptr %10, align 4
  %4276 = lshr i32 %4275, 7
  %4277 = and i32 %4276, 255
  %4278 = sext i32 %4277 to i64
  %4279 = getelementptr inbounds %union.StackValue, ptr %4274, i64 %4278
  store ptr %4279, ptr %249, align 8
  %4280 = load ptr, ptr %7, align 8
  %4281 = load i32, ptr %10, align 4
  %4282 = lshr i32 %4281, 16
  %4283 = and i32 %4282, 255
  %4284 = sext i32 %4283 to i64
  %4285 = getelementptr inbounds %union.StackValue, ptr %4280, i64 %4284
  store ptr %4285, ptr %250, align 8
  %4286 = load ptr, ptr %7, align 8
  %4287 = load i32, ptr %10, align 4
  %4288 = lshr i32 %4287, 24
  %4289 = and i32 %4288, 255
  %4290 = sext i32 %4289 to i64
  %4291 = getelementptr inbounds %union.StackValue, ptr %4286, i64 %4290
  store ptr %4291, ptr %251, align 8
  %4292 = load ptr, ptr %250, align 8
  %4293 = getelementptr inbounds %struct.TValue, ptr %4292, i32 0, i32 1
  %4294 = load i8, ptr %4293, align 8
  %4295 = zext i8 %4294 to i32
  %4296 = icmp eq i32 %4295, 19
  br i1 %4296, label %4297, label %4301

4297:                                             ; preds = %4273
  %4298 = load ptr, ptr %250, align 8
  %4299 = getelementptr inbounds %struct.TValue, ptr %4298, i32 0, i32 0
  %4300 = load double, ptr %4299, align 8
  store double %4300, ptr %252, align 8
  br i1 true, label %4313, label %4346

4301:                                             ; preds = %4273
  %4302 = load ptr, ptr %250, align 8
  %4303 = getelementptr inbounds %struct.TValue, ptr %4302, i32 0, i32 1
  %4304 = load i8, ptr %4303, align 8
  %4305 = zext i8 %4304 to i32
  %4306 = icmp eq i32 %4305, 3
  br i1 %4306, label %4307, label %4312

4307:                                             ; preds = %4301
  %4308 = load ptr, ptr %250, align 8
  %4309 = getelementptr inbounds %struct.TValue, ptr %4308, i32 0, i32 0
  %4310 = load i64, ptr %4309, align 8
  %4311 = sitofp i64 %4310 to double
  store double %4311, ptr %252, align 8
  br i1 true, label %4313, label %4346

4312:                                             ; preds = %4301
  br i1 false, label %4313, label %4346

4313:                                             ; preds = %4312, %4307, %4297
  %4314 = load ptr, ptr %251, align 8
  %4315 = getelementptr inbounds %struct.TValue, ptr %4314, i32 0, i32 1
  %4316 = load i8, ptr %4315, align 8
  %4317 = zext i8 %4316 to i32
  %4318 = icmp eq i32 %4317, 19
  br i1 %4318, label %4319, label %4323

4319:                                             ; preds = %4313
  %4320 = load ptr, ptr %251, align 8
  %4321 = getelementptr inbounds %struct.TValue, ptr %4320, i32 0, i32 0
  %4322 = load double, ptr %4321, align 8
  store double %4322, ptr %253, align 8
  br i1 true, label %4335, label %4346

4323:                                             ; preds = %4313
  %4324 = load ptr, ptr %251, align 8
  %4325 = getelementptr inbounds %struct.TValue, ptr %4324, i32 0, i32 1
  %4326 = load i8, ptr %4325, align 8
  %4327 = zext i8 %4326 to i32
  %4328 = icmp eq i32 %4327, 3
  br i1 %4328, label %4329, label %4334

4329:                                             ; preds = %4323
  %4330 = load ptr, ptr %251, align 8
  %4331 = getelementptr inbounds %struct.TValue, ptr %4330, i32 0, i32 0
  %4332 = load i64, ptr %4331, align 8
  %4333 = sitofp i64 %4332 to double
  store double %4333, ptr %253, align 8
  br i1 true, label %4335, label %4346

4334:                                             ; preds = %4323
  br i1 false, label %4335, label %4346

4335:                                             ; preds = %4334, %4329, %4319
  %4336 = load ptr, ptr %8, align 8
  %4337 = getelementptr inbounds i32, ptr %4336, i32 1
  store ptr %4337, ptr %8, align 8
  %4338 = load ptr, ptr %249, align 8
  store ptr %4338, ptr %254, align 8
  %4339 = load double, ptr %252, align 8
  %4340 = load double, ptr %253, align 8
  %4341 = fdiv double %4339, %4340
  %4342 = load ptr, ptr %254, align 8
  %4343 = getelementptr inbounds %struct.TValue, ptr %4342, i32 0, i32 0
  store double %4341, ptr %4343, align 8
  %4344 = load ptr, ptr %254, align 8
  %4345 = getelementptr inbounds %struct.TValue, ptr %4344, i32 0, i32 1
  store i8 19, ptr %4345, align 8
  br label %4346

4346:                                             ; preds = %4335, %4334, %4329, %4319, %4312, %4307, %4297
  %4347 = load i32, ptr %9, align 4
  %4348 = icmp ne i32 %4347, 0
  %4349 = zext i1 %4348 to i32
  %4350 = sext i32 %4349 to i64
  %4351 = icmp ne i64 %4350, 0
  br i1 %4351, label %4352, label %4360

4352:                                             ; preds = %4346
  %4353 = load ptr, ptr %3, align 8
  %4354 = load ptr, ptr %8, align 8
  %4355 = call i32 @luaG_traceexec(ptr noundef %4353, ptr noundef %4354)
  store i32 %4355, ptr %9, align 4
  %4356 = load ptr, ptr %4, align 8
  %4357 = getelementptr inbounds %struct.CallInfo, ptr %4356, i32 0, i32 0
  %4358 = load ptr, ptr %4357, align 8
  %4359 = getelementptr inbounds %union.StackValue, ptr %4358, i64 1
  store ptr %4359, ptr %7, align 8
  br label %4360

4360:                                             ; preds = %4352, %4346
  %4361 = load ptr, ptr %8, align 8
  %4362 = getelementptr inbounds i32, ptr %4361, i32 1
  store ptr %4362, ptr %8, align 8
  %4363 = load i32, ptr %4361, align 4
  store i32 %4363, ptr %10, align 4
  %4364 = load i32, ptr %10, align 4
  %4365 = lshr i32 %4364, 0
  %4366 = and i32 %4365, 127
  %4367 = zext i32 %4366 to i64
  %4368 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4367
  %4369 = load ptr, ptr %4368, align 8
  br label %7120

4370:                                             ; preds = %7120
  %4371 = load ptr, ptr %8, align 8
  %4372 = load ptr, ptr %4, align 8
  %4373 = getelementptr inbounds %struct.CallInfo, ptr %4372, i32 0, i32 4
  %4374 = getelementptr inbounds %struct.anon, ptr %4373, i32 0, i32 0
  store ptr %4371, ptr %4374, align 8
  %4375 = load ptr, ptr %4, align 8
  %4376 = getelementptr inbounds %struct.CallInfo, ptr %4375, i32 0, i32 1
  %4377 = load ptr, ptr %4376, align 8
  %4378 = load ptr, ptr %3, align 8
  %4379 = getelementptr inbounds %struct.lua_State, ptr %4378, i32 0, i32 6
  store ptr %4377, ptr %4379, align 8
  %4380 = load ptr, ptr %7, align 8
  %4381 = load i32, ptr %10, align 4
  %4382 = lshr i32 %4381, 16
  %4383 = and i32 %4382, 255
  %4384 = sext i32 %4383 to i64
  %4385 = getelementptr inbounds %union.StackValue, ptr %4380, i64 %4384
  store ptr %4385, ptr %255, align 8
  %4386 = load ptr, ptr %7, align 8
  %4387 = load i32, ptr %10, align 4
  %4388 = lshr i32 %4387, 24
  %4389 = and i32 %4388, 255
  %4390 = sext i32 %4389 to i64
  %4391 = getelementptr inbounds %union.StackValue, ptr %4386, i64 %4390
  store ptr %4391, ptr %256, align 8
  %4392 = load ptr, ptr %7, align 8
  %4393 = load i32, ptr %10, align 4
  %4394 = lshr i32 %4393, 7
  %4395 = and i32 %4394, 255
  %4396 = sext i32 %4395 to i64
  %4397 = getelementptr inbounds %union.StackValue, ptr %4392, i64 %4396
  store ptr %4397, ptr %257, align 8
  %4398 = load ptr, ptr %255, align 8
  %4399 = getelementptr inbounds %struct.TValue, ptr %4398, i32 0, i32 1
  %4400 = load i8, ptr %4399, align 8
  %4401 = zext i8 %4400 to i32
  %4402 = icmp eq i32 %4401, 3
  br i1 %4402, label %4403, label %4427

4403:                                             ; preds = %4370
  %4404 = load ptr, ptr %256, align 8
  %4405 = getelementptr inbounds %struct.TValue, ptr %4404, i32 0, i32 1
  %4406 = load i8, ptr %4405, align 8
  %4407 = zext i8 %4406 to i32
  %4408 = icmp eq i32 %4407, 3
  br i1 %4408, label %4409, label %4427

4409:                                             ; preds = %4403
  %4410 = load ptr, ptr %255, align 8
  %4411 = getelementptr inbounds %struct.TValue, ptr %4410, i32 0, i32 0
  %4412 = load i64, ptr %4411, align 8
  store i64 %4412, ptr %258, align 8
  %4413 = load ptr, ptr %256, align 8
  %4414 = getelementptr inbounds %struct.TValue, ptr %4413, i32 0, i32 0
  %4415 = load i64, ptr %4414, align 8
  store i64 %4415, ptr %259, align 8
  %4416 = load ptr, ptr %8, align 8
  %4417 = getelementptr inbounds i32, ptr %4416, i32 1
  store ptr %4417, ptr %8, align 8
  %4418 = load ptr, ptr %257, align 8
  store ptr %4418, ptr %260, align 8
  %4419 = load ptr, ptr %3, align 8
  %4420 = load i64, ptr %258, align 8
  %4421 = load i64, ptr %259, align 8
  %4422 = call i64 @luaV_idiv(ptr noundef %4419, i64 noundef %4420, i64 noundef %4421)
  %4423 = load ptr, ptr %260, align 8
  %4424 = getelementptr inbounds %struct.TValue, ptr %4423, i32 0, i32 0
  store i64 %4422, ptr %4424, align 8
  %4425 = load ptr, ptr %260, align 8
  %4426 = getelementptr inbounds %struct.TValue, ptr %4425, i32 0, i32 1
  store i8 3, ptr %4426, align 8
  br label %4485

4427:                                             ; preds = %4403, %4370
  %4428 = load ptr, ptr %255, align 8
  %4429 = getelementptr inbounds %struct.TValue, ptr %4428, i32 0, i32 1
  %4430 = load i8, ptr %4429, align 8
  %4431 = zext i8 %4430 to i32
  %4432 = icmp eq i32 %4431, 19
  br i1 %4432, label %4433, label %4437

4433:                                             ; preds = %4427
  %4434 = load ptr, ptr %255, align 8
  %4435 = getelementptr inbounds %struct.TValue, ptr %4434, i32 0, i32 0
  %4436 = load double, ptr %4435, align 8
  store double %4436, ptr %261, align 8
  br i1 true, label %4449, label %4484

4437:                                             ; preds = %4427
  %4438 = load ptr, ptr %255, align 8
  %4439 = getelementptr inbounds %struct.TValue, ptr %4438, i32 0, i32 1
  %4440 = load i8, ptr %4439, align 8
  %4441 = zext i8 %4440 to i32
  %4442 = icmp eq i32 %4441, 3
  br i1 %4442, label %4443, label %4448

4443:                                             ; preds = %4437
  %4444 = load ptr, ptr %255, align 8
  %4445 = getelementptr inbounds %struct.TValue, ptr %4444, i32 0, i32 0
  %4446 = load i64, ptr %4445, align 8
  %4447 = sitofp i64 %4446 to double
  store double %4447, ptr %261, align 8
  br i1 true, label %4449, label %4484

4448:                                             ; preds = %4437
  br i1 false, label %4449, label %4484

4449:                                             ; preds = %4448, %4443, %4433
  %4450 = load ptr, ptr %256, align 8
  %4451 = getelementptr inbounds %struct.TValue, ptr %4450, i32 0, i32 1
  %4452 = load i8, ptr %4451, align 8
  %4453 = zext i8 %4452 to i32
  %4454 = icmp eq i32 %4453, 19
  br i1 %4454, label %4455, label %4459

4455:                                             ; preds = %4449
  %4456 = load ptr, ptr %256, align 8
  %4457 = getelementptr inbounds %struct.TValue, ptr %4456, i32 0, i32 0
  %4458 = load double, ptr %4457, align 8
  store double %4458, ptr %262, align 8
  br i1 true, label %4471, label %4484

4459:                                             ; preds = %4449
  %4460 = load ptr, ptr %256, align 8
  %4461 = getelementptr inbounds %struct.TValue, ptr %4460, i32 0, i32 1
  %4462 = load i8, ptr %4461, align 8
  %4463 = zext i8 %4462 to i32
  %4464 = icmp eq i32 %4463, 3
  br i1 %4464, label %4465, label %4470

4465:                                             ; preds = %4459
  %4466 = load ptr, ptr %256, align 8
  %4467 = getelementptr inbounds %struct.TValue, ptr %4466, i32 0, i32 0
  %4468 = load i64, ptr %4467, align 8
  %4469 = sitofp i64 %4468 to double
  store double %4469, ptr %262, align 8
  br i1 true, label %4471, label %4484

4470:                                             ; preds = %4459
  br i1 false, label %4471, label %4484

4471:                                             ; preds = %4470, %4465, %4455
  %4472 = load ptr, ptr %8, align 8
  %4473 = getelementptr inbounds i32, ptr %4472, i32 1
  store ptr %4473, ptr %8, align 8
  %4474 = load ptr, ptr %257, align 8
  store ptr %4474, ptr %263, align 8
  %4475 = load ptr, ptr %3, align 8
  %4476 = load double, ptr %261, align 8
  %4477 = load double, ptr %262, align 8
  %4478 = fdiv double %4476, %4477
  %4479 = call double @llvm.floor.f64(double %4478)
  %4480 = load ptr, ptr %263, align 8
  %4481 = getelementptr inbounds %struct.TValue, ptr %4480, i32 0, i32 0
  store double %4479, ptr %4481, align 8
  %4482 = load ptr, ptr %263, align 8
  %4483 = getelementptr inbounds %struct.TValue, ptr %4482, i32 0, i32 1
  store i8 19, ptr %4483, align 8
  br label %4484

4484:                                             ; preds = %4471, %4470, %4465, %4455, %4448, %4443, %4433
  br label %4485

4485:                                             ; preds = %4484, %4409
  %4486 = load i32, ptr %9, align 4
  %4487 = icmp ne i32 %4486, 0
  %4488 = zext i1 %4487 to i32
  %4489 = sext i32 %4488 to i64
  %4490 = icmp ne i64 %4489, 0
  br i1 %4490, label %4491, label %4499

4491:                                             ; preds = %4485
  %4492 = load ptr, ptr %3, align 8
  %4493 = load ptr, ptr %8, align 8
  %4494 = call i32 @luaG_traceexec(ptr noundef %4492, ptr noundef %4493)
  store i32 %4494, ptr %9, align 4
  %4495 = load ptr, ptr %4, align 8
  %4496 = getelementptr inbounds %struct.CallInfo, ptr %4495, i32 0, i32 0
  %4497 = load ptr, ptr %4496, align 8
  %4498 = getelementptr inbounds %union.StackValue, ptr %4497, i64 1
  store ptr %4498, ptr %7, align 8
  br label %4499

4499:                                             ; preds = %4491, %4485
  %4500 = load ptr, ptr %8, align 8
  %4501 = getelementptr inbounds i32, ptr %4500, i32 1
  store ptr %4501, ptr %8, align 8
  %4502 = load i32, ptr %4500, align 4
  store i32 %4502, ptr %10, align 4
  %4503 = load i32, ptr %10, align 4
  %4504 = lshr i32 %4503, 0
  %4505 = and i32 %4504, 127
  %4506 = zext i32 %4505 to i64
  %4507 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4506
  %4508 = load ptr, ptr %4507, align 8
  br label %7120

4509:                                             ; preds = %7120
  %4510 = load ptr, ptr %7, align 8
  %4511 = load i32, ptr %10, align 4
  %4512 = lshr i32 %4511, 7
  %4513 = and i32 %4512, 255
  %4514 = sext i32 %4513 to i64
  %4515 = getelementptr inbounds %union.StackValue, ptr %4510, i64 %4514
  store ptr %4515, ptr %264, align 8
  %4516 = load ptr, ptr %7, align 8
  %4517 = load i32, ptr %10, align 4
  %4518 = lshr i32 %4517, 16
  %4519 = and i32 %4518, 255
  %4520 = sext i32 %4519 to i64
  %4521 = getelementptr inbounds %union.StackValue, ptr %4516, i64 %4520
  store ptr %4521, ptr %265, align 8
  %4522 = load ptr, ptr %7, align 8
  %4523 = load i32, ptr %10, align 4
  %4524 = lshr i32 %4523, 24
  %4525 = and i32 %4524, 255
  %4526 = sext i32 %4525 to i64
  %4527 = getelementptr inbounds %union.StackValue, ptr %4522, i64 %4526
  store ptr %4527, ptr %266, align 8
  %4528 = load ptr, ptr %265, align 8
  %4529 = getelementptr inbounds %struct.TValue, ptr %4528, i32 0, i32 1
  %4530 = load i8, ptr %4529, align 8
  %4531 = zext i8 %4530 to i32
  %4532 = icmp eq i32 %4531, 3
  %4533 = zext i1 %4532 to i32
  %4534 = icmp ne i32 %4533, 0
  %4535 = zext i1 %4534 to i32
  %4536 = sext i32 %4535 to i64
  %4537 = icmp ne i64 %4536, 0
  br i1 %4537, label %4538, label %4542

4538:                                             ; preds = %4509
  %4539 = load ptr, ptr %265, align 8
  %4540 = getelementptr inbounds %struct.TValue, ptr %4539, i32 0, i32 0
  %4541 = load i64, ptr %4540, align 8
  store i64 %4541, ptr %267, align 8
  br i1 true, label %4546, label %4576

4542:                                             ; preds = %4509
  %4543 = load ptr, ptr %265, align 8
  %4544 = call i32 @luaV_tointegerns(ptr noundef %4543, ptr noundef %267, i32 noundef 0)
  %4545 = icmp ne i32 %4544, 0
  br i1 %4545, label %4546, label %4576

4546:                                             ; preds = %4542, %4538
  %4547 = load ptr, ptr %266, align 8
  %4548 = getelementptr inbounds %struct.TValue, ptr %4547, i32 0, i32 1
  %4549 = load i8, ptr %4548, align 8
  %4550 = zext i8 %4549 to i32
  %4551 = icmp eq i32 %4550, 3
  %4552 = zext i1 %4551 to i32
  %4553 = icmp ne i32 %4552, 0
  %4554 = zext i1 %4553 to i32
  %4555 = sext i32 %4554 to i64
  %4556 = icmp ne i64 %4555, 0
  br i1 %4556, label %4557, label %4561

4557:                                             ; preds = %4546
  %4558 = load ptr, ptr %266, align 8
  %4559 = getelementptr inbounds %struct.TValue, ptr %4558, i32 0, i32 0
  %4560 = load i64, ptr %4559, align 8
  store i64 %4560, ptr %268, align 8
  br i1 true, label %4565, label %4576

4561:                                             ; preds = %4546
  %4562 = load ptr, ptr %266, align 8
  %4563 = call i32 @luaV_tointegerns(ptr noundef %4562, ptr noundef %268, i32 noundef 0)
  %4564 = icmp ne i32 %4563, 0
  br i1 %4564, label %4565, label %4576

4565:                                             ; preds = %4561, %4557
  %4566 = load ptr, ptr %8, align 8
  %4567 = getelementptr inbounds i32, ptr %4566, i32 1
  store ptr %4567, ptr %8, align 8
  %4568 = load ptr, ptr %264, align 8
  store ptr %4568, ptr %269, align 8
  %4569 = load i64, ptr %267, align 8
  %4570 = load i64, ptr %268, align 8
  %4571 = and i64 %4569, %4570
  %4572 = load ptr, ptr %269, align 8
  %4573 = getelementptr inbounds %struct.TValue, ptr %4572, i32 0, i32 0
  store i64 %4571, ptr %4573, align 8
  %4574 = load ptr, ptr %269, align 8
  %4575 = getelementptr inbounds %struct.TValue, ptr %4574, i32 0, i32 1
  store i8 3, ptr %4575, align 8
  br label %4576

4576:                                             ; preds = %4565, %4561, %4557, %4542, %4538
  %4577 = load i32, ptr %9, align 4
  %4578 = icmp ne i32 %4577, 0
  %4579 = zext i1 %4578 to i32
  %4580 = sext i32 %4579 to i64
  %4581 = icmp ne i64 %4580, 0
  br i1 %4581, label %4582, label %4590

4582:                                             ; preds = %4576
  %4583 = load ptr, ptr %3, align 8
  %4584 = load ptr, ptr %8, align 8
  %4585 = call i32 @luaG_traceexec(ptr noundef %4583, ptr noundef %4584)
  store i32 %4585, ptr %9, align 4
  %4586 = load ptr, ptr %4, align 8
  %4587 = getelementptr inbounds %struct.CallInfo, ptr %4586, i32 0, i32 0
  %4588 = load ptr, ptr %4587, align 8
  %4589 = getelementptr inbounds %union.StackValue, ptr %4588, i64 1
  store ptr %4589, ptr %7, align 8
  br label %4590

4590:                                             ; preds = %4582, %4576
  %4591 = load ptr, ptr %8, align 8
  %4592 = getelementptr inbounds i32, ptr %4591, i32 1
  store ptr %4592, ptr %8, align 8
  %4593 = load i32, ptr %4591, align 4
  store i32 %4593, ptr %10, align 4
  %4594 = load i32, ptr %10, align 4
  %4595 = lshr i32 %4594, 0
  %4596 = and i32 %4595, 127
  %4597 = zext i32 %4596 to i64
  %4598 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4597
  %4599 = load ptr, ptr %4598, align 8
  br label %7120

4600:                                             ; preds = %7120
  %4601 = load ptr, ptr %7, align 8
  %4602 = load i32, ptr %10, align 4
  %4603 = lshr i32 %4602, 7
  %4604 = and i32 %4603, 255
  %4605 = sext i32 %4604 to i64
  %4606 = getelementptr inbounds %union.StackValue, ptr %4601, i64 %4605
  store ptr %4606, ptr %270, align 8
  %4607 = load ptr, ptr %7, align 8
  %4608 = load i32, ptr %10, align 4
  %4609 = lshr i32 %4608, 16
  %4610 = and i32 %4609, 255
  %4611 = sext i32 %4610 to i64
  %4612 = getelementptr inbounds %union.StackValue, ptr %4607, i64 %4611
  store ptr %4612, ptr %271, align 8
  %4613 = load ptr, ptr %7, align 8
  %4614 = load i32, ptr %10, align 4
  %4615 = lshr i32 %4614, 24
  %4616 = and i32 %4615, 255
  %4617 = sext i32 %4616 to i64
  %4618 = getelementptr inbounds %union.StackValue, ptr %4613, i64 %4617
  store ptr %4618, ptr %272, align 8
  %4619 = load ptr, ptr %271, align 8
  %4620 = getelementptr inbounds %struct.TValue, ptr %4619, i32 0, i32 1
  %4621 = load i8, ptr %4620, align 8
  %4622 = zext i8 %4621 to i32
  %4623 = icmp eq i32 %4622, 3
  %4624 = zext i1 %4623 to i32
  %4625 = icmp ne i32 %4624, 0
  %4626 = zext i1 %4625 to i32
  %4627 = sext i32 %4626 to i64
  %4628 = icmp ne i64 %4627, 0
  br i1 %4628, label %4629, label %4633

4629:                                             ; preds = %4600
  %4630 = load ptr, ptr %271, align 8
  %4631 = getelementptr inbounds %struct.TValue, ptr %4630, i32 0, i32 0
  %4632 = load i64, ptr %4631, align 8
  store i64 %4632, ptr %273, align 8
  br i1 true, label %4637, label %4667

4633:                                             ; preds = %4600
  %4634 = load ptr, ptr %271, align 8
  %4635 = call i32 @luaV_tointegerns(ptr noundef %4634, ptr noundef %273, i32 noundef 0)
  %4636 = icmp ne i32 %4635, 0
  br i1 %4636, label %4637, label %4667

4637:                                             ; preds = %4633, %4629
  %4638 = load ptr, ptr %272, align 8
  %4639 = getelementptr inbounds %struct.TValue, ptr %4638, i32 0, i32 1
  %4640 = load i8, ptr %4639, align 8
  %4641 = zext i8 %4640 to i32
  %4642 = icmp eq i32 %4641, 3
  %4643 = zext i1 %4642 to i32
  %4644 = icmp ne i32 %4643, 0
  %4645 = zext i1 %4644 to i32
  %4646 = sext i32 %4645 to i64
  %4647 = icmp ne i64 %4646, 0
  br i1 %4647, label %4648, label %4652

4648:                                             ; preds = %4637
  %4649 = load ptr, ptr %272, align 8
  %4650 = getelementptr inbounds %struct.TValue, ptr %4649, i32 0, i32 0
  %4651 = load i64, ptr %4650, align 8
  store i64 %4651, ptr %274, align 8
  br i1 true, label %4656, label %4667

4652:                                             ; preds = %4637
  %4653 = load ptr, ptr %272, align 8
  %4654 = call i32 @luaV_tointegerns(ptr noundef %4653, ptr noundef %274, i32 noundef 0)
  %4655 = icmp ne i32 %4654, 0
  br i1 %4655, label %4656, label %4667

4656:                                             ; preds = %4652, %4648
  %4657 = load ptr, ptr %8, align 8
  %4658 = getelementptr inbounds i32, ptr %4657, i32 1
  store ptr %4658, ptr %8, align 8
  %4659 = load ptr, ptr %270, align 8
  store ptr %4659, ptr %275, align 8
  %4660 = load i64, ptr %273, align 8
  %4661 = load i64, ptr %274, align 8
  %4662 = or i64 %4660, %4661
  %4663 = load ptr, ptr %275, align 8
  %4664 = getelementptr inbounds %struct.TValue, ptr %4663, i32 0, i32 0
  store i64 %4662, ptr %4664, align 8
  %4665 = load ptr, ptr %275, align 8
  %4666 = getelementptr inbounds %struct.TValue, ptr %4665, i32 0, i32 1
  store i8 3, ptr %4666, align 8
  br label %4667

4667:                                             ; preds = %4656, %4652, %4648, %4633, %4629
  %4668 = load i32, ptr %9, align 4
  %4669 = icmp ne i32 %4668, 0
  %4670 = zext i1 %4669 to i32
  %4671 = sext i32 %4670 to i64
  %4672 = icmp ne i64 %4671, 0
  br i1 %4672, label %4673, label %4681

4673:                                             ; preds = %4667
  %4674 = load ptr, ptr %3, align 8
  %4675 = load ptr, ptr %8, align 8
  %4676 = call i32 @luaG_traceexec(ptr noundef %4674, ptr noundef %4675)
  store i32 %4676, ptr %9, align 4
  %4677 = load ptr, ptr %4, align 8
  %4678 = getelementptr inbounds %struct.CallInfo, ptr %4677, i32 0, i32 0
  %4679 = load ptr, ptr %4678, align 8
  %4680 = getelementptr inbounds %union.StackValue, ptr %4679, i64 1
  store ptr %4680, ptr %7, align 8
  br label %4681

4681:                                             ; preds = %4673, %4667
  %4682 = load ptr, ptr %8, align 8
  %4683 = getelementptr inbounds i32, ptr %4682, i32 1
  store ptr %4683, ptr %8, align 8
  %4684 = load i32, ptr %4682, align 4
  store i32 %4684, ptr %10, align 4
  %4685 = load i32, ptr %10, align 4
  %4686 = lshr i32 %4685, 0
  %4687 = and i32 %4686, 127
  %4688 = zext i32 %4687 to i64
  %4689 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4688
  %4690 = load ptr, ptr %4689, align 8
  br label %7120

4691:                                             ; preds = %7120
  %4692 = load ptr, ptr %7, align 8
  %4693 = load i32, ptr %10, align 4
  %4694 = lshr i32 %4693, 7
  %4695 = and i32 %4694, 255
  %4696 = sext i32 %4695 to i64
  %4697 = getelementptr inbounds %union.StackValue, ptr %4692, i64 %4696
  store ptr %4697, ptr %276, align 8
  %4698 = load ptr, ptr %7, align 8
  %4699 = load i32, ptr %10, align 4
  %4700 = lshr i32 %4699, 16
  %4701 = and i32 %4700, 255
  %4702 = sext i32 %4701 to i64
  %4703 = getelementptr inbounds %union.StackValue, ptr %4698, i64 %4702
  store ptr %4703, ptr %277, align 8
  %4704 = load ptr, ptr %7, align 8
  %4705 = load i32, ptr %10, align 4
  %4706 = lshr i32 %4705, 24
  %4707 = and i32 %4706, 255
  %4708 = sext i32 %4707 to i64
  %4709 = getelementptr inbounds %union.StackValue, ptr %4704, i64 %4708
  store ptr %4709, ptr %278, align 8
  %4710 = load ptr, ptr %277, align 8
  %4711 = getelementptr inbounds %struct.TValue, ptr %4710, i32 0, i32 1
  %4712 = load i8, ptr %4711, align 8
  %4713 = zext i8 %4712 to i32
  %4714 = icmp eq i32 %4713, 3
  %4715 = zext i1 %4714 to i32
  %4716 = icmp ne i32 %4715, 0
  %4717 = zext i1 %4716 to i32
  %4718 = sext i32 %4717 to i64
  %4719 = icmp ne i64 %4718, 0
  br i1 %4719, label %4720, label %4724

4720:                                             ; preds = %4691
  %4721 = load ptr, ptr %277, align 8
  %4722 = getelementptr inbounds %struct.TValue, ptr %4721, i32 0, i32 0
  %4723 = load i64, ptr %4722, align 8
  store i64 %4723, ptr %279, align 8
  br i1 true, label %4728, label %4758

4724:                                             ; preds = %4691
  %4725 = load ptr, ptr %277, align 8
  %4726 = call i32 @luaV_tointegerns(ptr noundef %4725, ptr noundef %279, i32 noundef 0)
  %4727 = icmp ne i32 %4726, 0
  br i1 %4727, label %4728, label %4758

4728:                                             ; preds = %4724, %4720
  %4729 = load ptr, ptr %278, align 8
  %4730 = getelementptr inbounds %struct.TValue, ptr %4729, i32 0, i32 1
  %4731 = load i8, ptr %4730, align 8
  %4732 = zext i8 %4731 to i32
  %4733 = icmp eq i32 %4732, 3
  %4734 = zext i1 %4733 to i32
  %4735 = icmp ne i32 %4734, 0
  %4736 = zext i1 %4735 to i32
  %4737 = sext i32 %4736 to i64
  %4738 = icmp ne i64 %4737, 0
  br i1 %4738, label %4739, label %4743

4739:                                             ; preds = %4728
  %4740 = load ptr, ptr %278, align 8
  %4741 = getelementptr inbounds %struct.TValue, ptr %4740, i32 0, i32 0
  %4742 = load i64, ptr %4741, align 8
  store i64 %4742, ptr %280, align 8
  br i1 true, label %4747, label %4758

4743:                                             ; preds = %4728
  %4744 = load ptr, ptr %278, align 8
  %4745 = call i32 @luaV_tointegerns(ptr noundef %4744, ptr noundef %280, i32 noundef 0)
  %4746 = icmp ne i32 %4745, 0
  br i1 %4746, label %4747, label %4758

4747:                                             ; preds = %4743, %4739
  %4748 = load ptr, ptr %8, align 8
  %4749 = getelementptr inbounds i32, ptr %4748, i32 1
  store ptr %4749, ptr %8, align 8
  %4750 = load ptr, ptr %276, align 8
  store ptr %4750, ptr %281, align 8
  %4751 = load i64, ptr %279, align 8
  %4752 = load i64, ptr %280, align 8
  %4753 = xor i64 %4751, %4752
  %4754 = load ptr, ptr %281, align 8
  %4755 = getelementptr inbounds %struct.TValue, ptr %4754, i32 0, i32 0
  store i64 %4753, ptr %4755, align 8
  %4756 = load ptr, ptr %281, align 8
  %4757 = getelementptr inbounds %struct.TValue, ptr %4756, i32 0, i32 1
  store i8 3, ptr %4757, align 8
  br label %4758

4758:                                             ; preds = %4747, %4743, %4739, %4724, %4720
  %4759 = load i32, ptr %9, align 4
  %4760 = icmp ne i32 %4759, 0
  %4761 = zext i1 %4760 to i32
  %4762 = sext i32 %4761 to i64
  %4763 = icmp ne i64 %4762, 0
  br i1 %4763, label %4764, label %4772

4764:                                             ; preds = %4758
  %4765 = load ptr, ptr %3, align 8
  %4766 = load ptr, ptr %8, align 8
  %4767 = call i32 @luaG_traceexec(ptr noundef %4765, ptr noundef %4766)
  store i32 %4767, ptr %9, align 4
  %4768 = load ptr, ptr %4, align 8
  %4769 = getelementptr inbounds %struct.CallInfo, ptr %4768, i32 0, i32 0
  %4770 = load ptr, ptr %4769, align 8
  %4771 = getelementptr inbounds %union.StackValue, ptr %4770, i64 1
  store ptr %4771, ptr %7, align 8
  br label %4772

4772:                                             ; preds = %4764, %4758
  %4773 = load ptr, ptr %8, align 8
  %4774 = getelementptr inbounds i32, ptr %4773, i32 1
  store ptr %4774, ptr %8, align 8
  %4775 = load i32, ptr %4773, align 4
  store i32 %4775, ptr %10, align 4
  %4776 = load i32, ptr %10, align 4
  %4777 = lshr i32 %4776, 0
  %4778 = and i32 %4777, 127
  %4779 = zext i32 %4778 to i64
  %4780 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4779
  %4781 = load ptr, ptr %4780, align 8
  br label %7120

4782:                                             ; preds = %7120
  %4783 = load ptr, ptr %7, align 8
  %4784 = load i32, ptr %10, align 4
  %4785 = lshr i32 %4784, 7
  %4786 = and i32 %4785, 255
  %4787 = sext i32 %4786 to i64
  %4788 = getelementptr inbounds %union.StackValue, ptr %4783, i64 %4787
  store ptr %4788, ptr %282, align 8
  %4789 = load ptr, ptr %7, align 8
  %4790 = load i32, ptr %10, align 4
  %4791 = lshr i32 %4790, 16
  %4792 = and i32 %4791, 255
  %4793 = sext i32 %4792 to i64
  %4794 = getelementptr inbounds %union.StackValue, ptr %4789, i64 %4793
  store ptr %4794, ptr %283, align 8
  %4795 = load ptr, ptr %7, align 8
  %4796 = load i32, ptr %10, align 4
  %4797 = lshr i32 %4796, 24
  %4798 = and i32 %4797, 255
  %4799 = sext i32 %4798 to i64
  %4800 = getelementptr inbounds %union.StackValue, ptr %4795, i64 %4799
  store ptr %4800, ptr %284, align 8
  %4801 = load ptr, ptr %283, align 8
  %4802 = getelementptr inbounds %struct.TValue, ptr %4801, i32 0, i32 1
  %4803 = load i8, ptr %4802, align 8
  %4804 = zext i8 %4803 to i32
  %4805 = icmp eq i32 %4804, 3
  %4806 = zext i1 %4805 to i32
  %4807 = icmp ne i32 %4806, 0
  %4808 = zext i1 %4807 to i32
  %4809 = sext i32 %4808 to i64
  %4810 = icmp ne i64 %4809, 0
  br i1 %4810, label %4811, label %4815

4811:                                             ; preds = %4782
  %4812 = load ptr, ptr %283, align 8
  %4813 = getelementptr inbounds %struct.TValue, ptr %4812, i32 0, i32 0
  %4814 = load i64, ptr %4813, align 8
  store i64 %4814, ptr %285, align 8
  br i1 true, label %4819, label %4850

4815:                                             ; preds = %4782
  %4816 = load ptr, ptr %283, align 8
  %4817 = call i32 @luaV_tointegerns(ptr noundef %4816, ptr noundef %285, i32 noundef 0)
  %4818 = icmp ne i32 %4817, 0
  br i1 %4818, label %4819, label %4850

4819:                                             ; preds = %4815, %4811
  %4820 = load ptr, ptr %284, align 8
  %4821 = getelementptr inbounds %struct.TValue, ptr %4820, i32 0, i32 1
  %4822 = load i8, ptr %4821, align 8
  %4823 = zext i8 %4822 to i32
  %4824 = icmp eq i32 %4823, 3
  %4825 = zext i1 %4824 to i32
  %4826 = icmp ne i32 %4825, 0
  %4827 = zext i1 %4826 to i32
  %4828 = sext i32 %4827 to i64
  %4829 = icmp ne i64 %4828, 0
  br i1 %4829, label %4830, label %4834

4830:                                             ; preds = %4819
  %4831 = load ptr, ptr %284, align 8
  %4832 = getelementptr inbounds %struct.TValue, ptr %4831, i32 0, i32 0
  %4833 = load i64, ptr %4832, align 8
  store i64 %4833, ptr %286, align 8
  br i1 true, label %4838, label %4850

4834:                                             ; preds = %4819
  %4835 = load ptr, ptr %284, align 8
  %4836 = call i32 @luaV_tointegerns(ptr noundef %4835, ptr noundef %286, i32 noundef 0)
  %4837 = icmp ne i32 %4836, 0
  br i1 %4837, label %4838, label %4850

4838:                                             ; preds = %4834, %4830
  %4839 = load ptr, ptr %8, align 8
  %4840 = getelementptr inbounds i32, ptr %4839, i32 1
  store ptr %4840, ptr %8, align 8
  %4841 = load ptr, ptr %282, align 8
  store ptr %4841, ptr %287, align 8
  %4842 = load i64, ptr %285, align 8
  %4843 = load i64, ptr %286, align 8
  %4844 = sub i64 0, %4843
  %4845 = call i64 @luaV_shiftl(i64 noundef %4842, i64 noundef %4844)
  %4846 = load ptr, ptr %287, align 8
  %4847 = getelementptr inbounds %struct.TValue, ptr %4846, i32 0, i32 0
  store i64 %4845, ptr %4847, align 8
  %4848 = load ptr, ptr %287, align 8
  %4849 = getelementptr inbounds %struct.TValue, ptr %4848, i32 0, i32 1
  store i8 3, ptr %4849, align 8
  br label %4850

4850:                                             ; preds = %4838, %4834, %4830, %4815, %4811
  %4851 = load i32, ptr %9, align 4
  %4852 = icmp ne i32 %4851, 0
  %4853 = zext i1 %4852 to i32
  %4854 = sext i32 %4853 to i64
  %4855 = icmp ne i64 %4854, 0
  br i1 %4855, label %4856, label %4864

4856:                                             ; preds = %4850
  %4857 = load ptr, ptr %3, align 8
  %4858 = load ptr, ptr %8, align 8
  %4859 = call i32 @luaG_traceexec(ptr noundef %4857, ptr noundef %4858)
  store i32 %4859, ptr %9, align 4
  %4860 = load ptr, ptr %4, align 8
  %4861 = getelementptr inbounds %struct.CallInfo, ptr %4860, i32 0, i32 0
  %4862 = load ptr, ptr %4861, align 8
  %4863 = getelementptr inbounds %union.StackValue, ptr %4862, i64 1
  store ptr %4863, ptr %7, align 8
  br label %4864

4864:                                             ; preds = %4856, %4850
  %4865 = load ptr, ptr %8, align 8
  %4866 = getelementptr inbounds i32, ptr %4865, i32 1
  store ptr %4866, ptr %8, align 8
  %4867 = load i32, ptr %4865, align 4
  store i32 %4867, ptr %10, align 4
  %4868 = load i32, ptr %10, align 4
  %4869 = lshr i32 %4868, 0
  %4870 = and i32 %4869, 127
  %4871 = zext i32 %4870 to i64
  %4872 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4871
  %4873 = load ptr, ptr %4872, align 8
  br label %7120

4874:                                             ; preds = %7120
  %4875 = load ptr, ptr %7, align 8
  %4876 = load i32, ptr %10, align 4
  %4877 = lshr i32 %4876, 7
  %4878 = and i32 %4877, 255
  %4879 = sext i32 %4878 to i64
  %4880 = getelementptr inbounds %union.StackValue, ptr %4875, i64 %4879
  store ptr %4880, ptr %288, align 8
  %4881 = load ptr, ptr %7, align 8
  %4882 = load i32, ptr %10, align 4
  %4883 = lshr i32 %4882, 16
  %4884 = and i32 %4883, 255
  %4885 = sext i32 %4884 to i64
  %4886 = getelementptr inbounds %union.StackValue, ptr %4881, i64 %4885
  store ptr %4886, ptr %289, align 8
  %4887 = load ptr, ptr %7, align 8
  %4888 = load i32, ptr %10, align 4
  %4889 = lshr i32 %4888, 24
  %4890 = and i32 %4889, 255
  %4891 = sext i32 %4890 to i64
  %4892 = getelementptr inbounds %union.StackValue, ptr %4887, i64 %4891
  store ptr %4892, ptr %290, align 8
  %4893 = load ptr, ptr %289, align 8
  %4894 = getelementptr inbounds %struct.TValue, ptr %4893, i32 0, i32 1
  %4895 = load i8, ptr %4894, align 8
  %4896 = zext i8 %4895 to i32
  %4897 = icmp eq i32 %4896, 3
  %4898 = zext i1 %4897 to i32
  %4899 = icmp ne i32 %4898, 0
  %4900 = zext i1 %4899 to i32
  %4901 = sext i32 %4900 to i64
  %4902 = icmp ne i64 %4901, 0
  br i1 %4902, label %4903, label %4907

4903:                                             ; preds = %4874
  %4904 = load ptr, ptr %289, align 8
  %4905 = getelementptr inbounds %struct.TValue, ptr %4904, i32 0, i32 0
  %4906 = load i64, ptr %4905, align 8
  store i64 %4906, ptr %291, align 8
  br i1 true, label %4911, label %4941

4907:                                             ; preds = %4874
  %4908 = load ptr, ptr %289, align 8
  %4909 = call i32 @luaV_tointegerns(ptr noundef %4908, ptr noundef %291, i32 noundef 0)
  %4910 = icmp ne i32 %4909, 0
  br i1 %4910, label %4911, label %4941

4911:                                             ; preds = %4907, %4903
  %4912 = load ptr, ptr %290, align 8
  %4913 = getelementptr inbounds %struct.TValue, ptr %4912, i32 0, i32 1
  %4914 = load i8, ptr %4913, align 8
  %4915 = zext i8 %4914 to i32
  %4916 = icmp eq i32 %4915, 3
  %4917 = zext i1 %4916 to i32
  %4918 = icmp ne i32 %4917, 0
  %4919 = zext i1 %4918 to i32
  %4920 = sext i32 %4919 to i64
  %4921 = icmp ne i64 %4920, 0
  br i1 %4921, label %4922, label %4926

4922:                                             ; preds = %4911
  %4923 = load ptr, ptr %290, align 8
  %4924 = getelementptr inbounds %struct.TValue, ptr %4923, i32 0, i32 0
  %4925 = load i64, ptr %4924, align 8
  store i64 %4925, ptr %292, align 8
  br i1 true, label %4930, label %4941

4926:                                             ; preds = %4911
  %4927 = load ptr, ptr %290, align 8
  %4928 = call i32 @luaV_tointegerns(ptr noundef %4927, ptr noundef %292, i32 noundef 0)
  %4929 = icmp ne i32 %4928, 0
  br i1 %4929, label %4930, label %4941

4930:                                             ; preds = %4926, %4922
  %4931 = load ptr, ptr %8, align 8
  %4932 = getelementptr inbounds i32, ptr %4931, i32 1
  store ptr %4932, ptr %8, align 8
  %4933 = load ptr, ptr %288, align 8
  store ptr %4933, ptr %293, align 8
  %4934 = load i64, ptr %291, align 8
  %4935 = load i64, ptr %292, align 8
  %4936 = call i64 @luaV_shiftl(i64 noundef %4934, i64 noundef %4935)
  %4937 = load ptr, ptr %293, align 8
  %4938 = getelementptr inbounds %struct.TValue, ptr %4937, i32 0, i32 0
  store i64 %4936, ptr %4938, align 8
  %4939 = load ptr, ptr %293, align 8
  %4940 = getelementptr inbounds %struct.TValue, ptr %4939, i32 0, i32 1
  store i8 3, ptr %4940, align 8
  br label %4941

4941:                                             ; preds = %4930, %4926, %4922, %4907, %4903
  %4942 = load i32, ptr %9, align 4
  %4943 = icmp ne i32 %4942, 0
  %4944 = zext i1 %4943 to i32
  %4945 = sext i32 %4944 to i64
  %4946 = icmp ne i64 %4945, 0
  br i1 %4946, label %4947, label %4955

4947:                                             ; preds = %4941
  %4948 = load ptr, ptr %3, align 8
  %4949 = load ptr, ptr %8, align 8
  %4950 = call i32 @luaG_traceexec(ptr noundef %4948, ptr noundef %4949)
  store i32 %4950, ptr %9, align 4
  %4951 = load ptr, ptr %4, align 8
  %4952 = getelementptr inbounds %struct.CallInfo, ptr %4951, i32 0, i32 0
  %4953 = load ptr, ptr %4952, align 8
  %4954 = getelementptr inbounds %union.StackValue, ptr %4953, i64 1
  store ptr %4954, ptr %7, align 8
  br label %4955

4955:                                             ; preds = %4947, %4941
  %4956 = load ptr, ptr %8, align 8
  %4957 = getelementptr inbounds i32, ptr %4956, i32 1
  store ptr %4957, ptr %8, align 8
  %4958 = load i32, ptr %4956, align 4
  store i32 %4958, ptr %10, align 4
  %4959 = load i32, ptr %10, align 4
  %4960 = lshr i32 %4959, 0
  %4961 = and i32 %4960, 127
  %4962 = zext i32 %4961 to i64
  %4963 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %4962
  %4964 = load ptr, ptr %4963, align 8
  br label %7120

4965:                                             ; preds = %7120
  %4966 = load ptr, ptr %7, align 8
  %4967 = load i32, ptr %10, align 4
  %4968 = lshr i32 %4967, 7
  %4969 = and i32 %4968, 255
  %4970 = sext i32 %4969 to i64
  %4971 = getelementptr inbounds %union.StackValue, ptr %4966, i64 %4970
  store ptr %4971, ptr %294, align 8
  %4972 = load ptr, ptr %8, align 8
  %4973 = getelementptr inbounds i32, ptr %4972, i64 -2
  %4974 = load i32, ptr %4973, align 4
  store i32 %4974, ptr %295, align 4
  %4975 = load ptr, ptr %7, align 8
  %4976 = load i32, ptr %10, align 4
  %4977 = lshr i32 %4976, 16
  %4978 = and i32 %4977, 255
  %4979 = sext i32 %4978 to i64
  %4980 = getelementptr inbounds %union.StackValue, ptr %4975, i64 %4979
  store ptr %4980, ptr %296, align 8
  %4981 = load i32, ptr %10, align 4
  %4982 = lshr i32 %4981, 24
  %4983 = and i32 %4982, 255
  store i32 %4983, ptr %297, align 4
  %4984 = load ptr, ptr %7, align 8
  %4985 = load i32, ptr %295, align 4
  %4986 = lshr i32 %4985, 7
  %4987 = and i32 %4986, 255
  %4988 = sext i32 %4987 to i64
  %4989 = getelementptr inbounds %union.StackValue, ptr %4984, i64 %4988
  store ptr %4989, ptr %298, align 8
  %4990 = load ptr, ptr %8, align 8
  %4991 = load ptr, ptr %4, align 8
  %4992 = getelementptr inbounds %struct.CallInfo, ptr %4991, i32 0, i32 4
  %4993 = getelementptr inbounds %struct.anon, ptr %4992, i32 0, i32 0
  store ptr %4990, ptr %4993, align 8
  %4994 = load ptr, ptr %4, align 8
  %4995 = getelementptr inbounds %struct.CallInfo, ptr %4994, i32 0, i32 1
  %4996 = load ptr, ptr %4995, align 8
  %4997 = load ptr, ptr %3, align 8
  %4998 = getelementptr inbounds %struct.lua_State, ptr %4997, i32 0, i32 6
  store ptr %4996, ptr %4998, align 8
  %4999 = load ptr, ptr %3, align 8
  %5000 = load ptr, ptr %294, align 8
  %5001 = load ptr, ptr %296, align 8
  %5002 = load ptr, ptr %298, align 8
  %5003 = load i32, ptr %297, align 4
  call void @luaT_trybinTM(ptr noundef %4999, ptr noundef %5000, ptr noundef %5001, ptr noundef %5002, i32 noundef %5003)
  %5004 = load ptr, ptr %4, align 8
  %5005 = getelementptr inbounds %struct.CallInfo, ptr %5004, i32 0, i32 4
  %5006 = getelementptr inbounds %struct.anon, ptr %5005, i32 0, i32 1
  %5007 = load volatile i32, ptr %5006, align 8
  store i32 %5007, ptr %9, align 4
  %5008 = load i32, ptr %9, align 4
  %5009 = icmp ne i32 %5008, 0
  %5010 = zext i1 %5009 to i32
  %5011 = sext i32 %5010 to i64
  %5012 = icmp ne i64 %5011, 0
  br i1 %5012, label %5013, label %5021

5013:                                             ; preds = %4965
  %5014 = load ptr, ptr %3, align 8
  %5015 = load ptr, ptr %8, align 8
  %5016 = call i32 @luaG_traceexec(ptr noundef %5014, ptr noundef %5015)
  store i32 %5016, ptr %9, align 4
  %5017 = load ptr, ptr %4, align 8
  %5018 = getelementptr inbounds %struct.CallInfo, ptr %5017, i32 0, i32 0
  %5019 = load ptr, ptr %5018, align 8
  %5020 = getelementptr inbounds %union.StackValue, ptr %5019, i64 1
  store ptr %5020, ptr %7, align 8
  br label %5021

5021:                                             ; preds = %5013, %4965
  %5022 = load ptr, ptr %8, align 8
  %5023 = getelementptr inbounds i32, ptr %5022, i32 1
  store ptr %5023, ptr %8, align 8
  %5024 = load i32, ptr %5022, align 4
  store i32 %5024, ptr %10, align 4
  %5025 = load i32, ptr %10, align 4
  %5026 = lshr i32 %5025, 0
  %5027 = and i32 %5026, 127
  %5028 = zext i32 %5027 to i64
  %5029 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5028
  %5030 = load ptr, ptr %5029, align 8
  br label %7120

5031:                                             ; preds = %7120
  %5032 = load ptr, ptr %7, align 8
  %5033 = load i32, ptr %10, align 4
  %5034 = lshr i32 %5033, 7
  %5035 = and i32 %5034, 255
  %5036 = sext i32 %5035 to i64
  %5037 = getelementptr inbounds %union.StackValue, ptr %5032, i64 %5036
  store ptr %5037, ptr %299, align 8
  %5038 = load ptr, ptr %8, align 8
  %5039 = getelementptr inbounds i32, ptr %5038, i64 -2
  %5040 = load i32, ptr %5039, align 4
  store i32 %5040, ptr %300, align 4
  %5041 = load i32, ptr %10, align 4
  %5042 = lshr i32 %5041, 16
  %5043 = and i32 %5042, 255
  %5044 = sub nsw i32 %5043, 127
  store i32 %5044, ptr %301, align 4
  %5045 = load i32, ptr %10, align 4
  %5046 = lshr i32 %5045, 24
  %5047 = and i32 %5046, 255
  store i32 %5047, ptr %302, align 4
  %5048 = load i32, ptr %10, align 4
  %5049 = lshr i32 %5048, 15
  %5050 = and i32 %5049, 1
  store i32 %5050, ptr %303, align 4
  %5051 = load ptr, ptr %7, align 8
  %5052 = load i32, ptr %300, align 4
  %5053 = lshr i32 %5052, 7
  %5054 = and i32 %5053, 255
  %5055 = sext i32 %5054 to i64
  %5056 = getelementptr inbounds %union.StackValue, ptr %5051, i64 %5055
  store ptr %5056, ptr %304, align 8
  %5057 = load ptr, ptr %8, align 8
  %5058 = load ptr, ptr %4, align 8
  %5059 = getelementptr inbounds %struct.CallInfo, ptr %5058, i32 0, i32 4
  %5060 = getelementptr inbounds %struct.anon, ptr %5059, i32 0, i32 0
  store ptr %5057, ptr %5060, align 8
  %5061 = load ptr, ptr %4, align 8
  %5062 = getelementptr inbounds %struct.CallInfo, ptr %5061, i32 0, i32 1
  %5063 = load ptr, ptr %5062, align 8
  %5064 = load ptr, ptr %3, align 8
  %5065 = getelementptr inbounds %struct.lua_State, ptr %5064, i32 0, i32 6
  store ptr %5063, ptr %5065, align 8
  %5066 = load ptr, ptr %3, align 8
  %5067 = load ptr, ptr %299, align 8
  %5068 = load i32, ptr %301, align 4
  %5069 = sext i32 %5068 to i64
  %5070 = load i32, ptr %303, align 4
  %5071 = load ptr, ptr %304, align 8
  %5072 = load i32, ptr %302, align 4
  call void @luaT_trybiniTM(ptr noundef %5066, ptr noundef %5067, i64 noundef %5069, i32 noundef %5070, ptr noundef %5071, i32 noundef %5072)
  %5073 = load ptr, ptr %4, align 8
  %5074 = getelementptr inbounds %struct.CallInfo, ptr %5073, i32 0, i32 4
  %5075 = getelementptr inbounds %struct.anon, ptr %5074, i32 0, i32 1
  %5076 = load volatile i32, ptr %5075, align 8
  store i32 %5076, ptr %9, align 4
  %5077 = load i32, ptr %9, align 4
  %5078 = icmp ne i32 %5077, 0
  %5079 = zext i1 %5078 to i32
  %5080 = sext i32 %5079 to i64
  %5081 = icmp ne i64 %5080, 0
  br i1 %5081, label %5082, label %5090

5082:                                             ; preds = %5031
  %5083 = load ptr, ptr %3, align 8
  %5084 = load ptr, ptr %8, align 8
  %5085 = call i32 @luaG_traceexec(ptr noundef %5083, ptr noundef %5084)
  store i32 %5085, ptr %9, align 4
  %5086 = load ptr, ptr %4, align 8
  %5087 = getelementptr inbounds %struct.CallInfo, ptr %5086, i32 0, i32 0
  %5088 = load ptr, ptr %5087, align 8
  %5089 = getelementptr inbounds %union.StackValue, ptr %5088, i64 1
  store ptr %5089, ptr %7, align 8
  br label %5090

5090:                                             ; preds = %5082, %5031
  %5091 = load ptr, ptr %8, align 8
  %5092 = getelementptr inbounds i32, ptr %5091, i32 1
  store ptr %5092, ptr %8, align 8
  %5093 = load i32, ptr %5091, align 4
  store i32 %5093, ptr %10, align 4
  %5094 = load i32, ptr %10, align 4
  %5095 = lshr i32 %5094, 0
  %5096 = and i32 %5095, 127
  %5097 = zext i32 %5096 to i64
  %5098 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5097
  %5099 = load ptr, ptr %5098, align 8
  br label %7120

5100:                                             ; preds = %7120
  %5101 = load ptr, ptr %7, align 8
  %5102 = load i32, ptr %10, align 4
  %5103 = lshr i32 %5102, 7
  %5104 = and i32 %5103, 255
  %5105 = sext i32 %5104 to i64
  %5106 = getelementptr inbounds %union.StackValue, ptr %5101, i64 %5105
  store ptr %5106, ptr %305, align 8
  %5107 = load ptr, ptr %8, align 8
  %5108 = getelementptr inbounds i32, ptr %5107, i64 -2
  %5109 = load i32, ptr %5108, align 4
  store i32 %5109, ptr %306, align 4
  %5110 = load ptr, ptr %6, align 8
  %5111 = load i32, ptr %10, align 4
  %5112 = lshr i32 %5111, 16
  %5113 = and i32 %5112, 255
  %5114 = sext i32 %5113 to i64
  %5115 = getelementptr inbounds %struct.TValue, ptr %5110, i64 %5114
  store ptr %5115, ptr %307, align 8
  %5116 = load i32, ptr %10, align 4
  %5117 = lshr i32 %5116, 24
  %5118 = and i32 %5117, 255
  store i32 %5118, ptr %308, align 4
  %5119 = load i32, ptr %10, align 4
  %5120 = lshr i32 %5119, 15
  %5121 = and i32 %5120, 1
  store i32 %5121, ptr %309, align 4
  %5122 = load ptr, ptr %7, align 8
  %5123 = load i32, ptr %306, align 4
  %5124 = lshr i32 %5123, 7
  %5125 = and i32 %5124, 255
  %5126 = sext i32 %5125 to i64
  %5127 = getelementptr inbounds %union.StackValue, ptr %5122, i64 %5126
  store ptr %5127, ptr %310, align 8
  %5128 = load ptr, ptr %8, align 8
  %5129 = load ptr, ptr %4, align 8
  %5130 = getelementptr inbounds %struct.CallInfo, ptr %5129, i32 0, i32 4
  %5131 = getelementptr inbounds %struct.anon, ptr %5130, i32 0, i32 0
  store ptr %5128, ptr %5131, align 8
  %5132 = load ptr, ptr %4, align 8
  %5133 = getelementptr inbounds %struct.CallInfo, ptr %5132, i32 0, i32 1
  %5134 = load ptr, ptr %5133, align 8
  %5135 = load ptr, ptr %3, align 8
  %5136 = getelementptr inbounds %struct.lua_State, ptr %5135, i32 0, i32 6
  store ptr %5134, ptr %5136, align 8
  %5137 = load ptr, ptr %3, align 8
  %5138 = load ptr, ptr %305, align 8
  %5139 = load ptr, ptr %307, align 8
  %5140 = load i32, ptr %309, align 4
  %5141 = load ptr, ptr %310, align 8
  %5142 = load i32, ptr %308, align 4
  call void @luaT_trybinassocTM(ptr noundef %5137, ptr noundef %5138, ptr noundef %5139, i32 noundef %5140, ptr noundef %5141, i32 noundef %5142)
  %5143 = load ptr, ptr %4, align 8
  %5144 = getelementptr inbounds %struct.CallInfo, ptr %5143, i32 0, i32 4
  %5145 = getelementptr inbounds %struct.anon, ptr %5144, i32 0, i32 1
  %5146 = load volatile i32, ptr %5145, align 8
  store i32 %5146, ptr %9, align 4
  %5147 = load i32, ptr %9, align 4
  %5148 = icmp ne i32 %5147, 0
  %5149 = zext i1 %5148 to i32
  %5150 = sext i32 %5149 to i64
  %5151 = icmp ne i64 %5150, 0
  br i1 %5151, label %5152, label %5160

5152:                                             ; preds = %5100
  %5153 = load ptr, ptr %3, align 8
  %5154 = load ptr, ptr %8, align 8
  %5155 = call i32 @luaG_traceexec(ptr noundef %5153, ptr noundef %5154)
  store i32 %5155, ptr %9, align 4
  %5156 = load ptr, ptr %4, align 8
  %5157 = getelementptr inbounds %struct.CallInfo, ptr %5156, i32 0, i32 0
  %5158 = load ptr, ptr %5157, align 8
  %5159 = getelementptr inbounds %union.StackValue, ptr %5158, i64 1
  store ptr %5159, ptr %7, align 8
  br label %5160

5160:                                             ; preds = %5152, %5100
  %5161 = load ptr, ptr %8, align 8
  %5162 = getelementptr inbounds i32, ptr %5161, i32 1
  store ptr %5162, ptr %8, align 8
  %5163 = load i32, ptr %5161, align 4
  store i32 %5163, ptr %10, align 4
  %5164 = load i32, ptr %10, align 4
  %5165 = lshr i32 %5164, 0
  %5166 = and i32 %5165, 127
  %5167 = zext i32 %5166 to i64
  %5168 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5167
  %5169 = load ptr, ptr %5168, align 8
  br label %7120

5170:                                             ; preds = %7120
  %5171 = load ptr, ptr %7, align 8
  %5172 = load i32, ptr %10, align 4
  %5173 = lshr i32 %5172, 7
  %5174 = and i32 %5173, 255
  %5175 = sext i32 %5174 to i64
  %5176 = getelementptr inbounds %union.StackValue, ptr %5171, i64 %5175
  store ptr %5176, ptr %311, align 8
  %5177 = load ptr, ptr %7, align 8
  %5178 = load i32, ptr %10, align 4
  %5179 = lshr i32 %5178, 16
  %5180 = and i32 %5179, 255
  %5181 = sext i32 %5180 to i64
  %5182 = getelementptr inbounds %union.StackValue, ptr %5177, i64 %5181
  store ptr %5182, ptr %312, align 8
  %5183 = load ptr, ptr %312, align 8
  %5184 = getelementptr inbounds %struct.TValue, ptr %5183, i32 0, i32 1
  %5185 = load i8, ptr %5184, align 8
  %5186 = zext i8 %5185 to i32
  %5187 = icmp eq i32 %5186, 3
  br i1 %5187, label %5188, label %5199

5188:                                             ; preds = %5170
  %5189 = load ptr, ptr %312, align 8
  %5190 = getelementptr inbounds %struct.TValue, ptr %5189, i32 0, i32 0
  %5191 = load i64, ptr %5190, align 8
  store i64 %5191, ptr %314, align 8
  %5192 = load ptr, ptr %311, align 8
  store ptr %5192, ptr %315, align 8
  %5193 = load i64, ptr %314, align 8
  %5194 = sub i64 0, %5193
  %5195 = load ptr, ptr %315, align 8
  %5196 = getelementptr inbounds %struct.TValue, ptr %5195, i32 0, i32 0
  store i64 %5194, ptr %5196, align 8
  %5197 = load ptr, ptr %315, align 8
  %5198 = getelementptr inbounds %struct.TValue, ptr %5197, i32 0, i32 1
  store i8 3, ptr %5198, align 8
  br label %5248

5199:                                             ; preds = %5170
  %5200 = load ptr, ptr %312, align 8
  %5201 = getelementptr inbounds %struct.TValue, ptr %5200, i32 0, i32 1
  %5202 = load i8, ptr %5201, align 8
  %5203 = zext i8 %5202 to i32
  %5204 = icmp eq i32 %5203, 19
  br i1 %5204, label %5205, label %5209

5205:                                             ; preds = %5199
  %5206 = load ptr, ptr %312, align 8
  %5207 = getelementptr inbounds %struct.TValue, ptr %5206, i32 0, i32 0
  %5208 = load double, ptr %5207, align 8
  store double %5208, ptr %313, align 8
  br i1 true, label %5221, label %5229

5209:                                             ; preds = %5199
  %5210 = load ptr, ptr %312, align 8
  %5211 = getelementptr inbounds %struct.TValue, ptr %5210, i32 0, i32 1
  %5212 = load i8, ptr %5211, align 8
  %5213 = zext i8 %5212 to i32
  %5214 = icmp eq i32 %5213, 3
  br i1 %5214, label %5215, label %5220

5215:                                             ; preds = %5209
  %5216 = load ptr, ptr %312, align 8
  %5217 = getelementptr inbounds %struct.TValue, ptr %5216, i32 0, i32 0
  %5218 = load i64, ptr %5217, align 8
  %5219 = sitofp i64 %5218 to double
  store double %5219, ptr %313, align 8
  br i1 true, label %5221, label %5229

5220:                                             ; preds = %5209
  br i1 false, label %5221, label %5229

5221:                                             ; preds = %5220, %5215, %5205
  %5222 = load ptr, ptr %311, align 8
  store ptr %5222, ptr %316, align 8
  %5223 = load double, ptr %313, align 8
  %5224 = fneg double %5223
  %5225 = load ptr, ptr %316, align 8
  %5226 = getelementptr inbounds %struct.TValue, ptr %5225, i32 0, i32 0
  store double %5224, ptr %5226, align 8
  %5227 = load ptr, ptr %316, align 8
  %5228 = getelementptr inbounds %struct.TValue, ptr %5227, i32 0, i32 1
  store i8 19, ptr %5228, align 8
  br label %5247

5229:                                             ; preds = %5220, %5215, %5205
  %5230 = load ptr, ptr %8, align 8
  %5231 = load ptr, ptr %4, align 8
  %5232 = getelementptr inbounds %struct.CallInfo, ptr %5231, i32 0, i32 4
  %5233 = getelementptr inbounds %struct.anon, ptr %5232, i32 0, i32 0
  store ptr %5230, ptr %5233, align 8
  %5234 = load ptr, ptr %4, align 8
  %5235 = getelementptr inbounds %struct.CallInfo, ptr %5234, i32 0, i32 1
  %5236 = load ptr, ptr %5235, align 8
  %5237 = load ptr, ptr %3, align 8
  %5238 = getelementptr inbounds %struct.lua_State, ptr %5237, i32 0, i32 6
  store ptr %5236, ptr %5238, align 8
  %5239 = load ptr, ptr %3, align 8
  %5240 = load ptr, ptr %312, align 8
  %5241 = load ptr, ptr %312, align 8
  %5242 = load ptr, ptr %311, align 8
  call void @luaT_trybinTM(ptr noundef %5239, ptr noundef %5240, ptr noundef %5241, ptr noundef %5242, i32 noundef 18)
  %5243 = load ptr, ptr %4, align 8
  %5244 = getelementptr inbounds %struct.CallInfo, ptr %5243, i32 0, i32 4
  %5245 = getelementptr inbounds %struct.anon, ptr %5244, i32 0, i32 1
  %5246 = load volatile i32, ptr %5245, align 8
  store i32 %5246, ptr %9, align 4
  br label %5247

5247:                                             ; preds = %5229, %5221
  br label %5248

5248:                                             ; preds = %5247, %5188
  %5249 = load i32, ptr %9, align 4
  %5250 = icmp ne i32 %5249, 0
  %5251 = zext i1 %5250 to i32
  %5252 = sext i32 %5251 to i64
  %5253 = icmp ne i64 %5252, 0
  br i1 %5253, label %5254, label %5262

5254:                                             ; preds = %5248
  %5255 = load ptr, ptr %3, align 8
  %5256 = load ptr, ptr %8, align 8
  %5257 = call i32 @luaG_traceexec(ptr noundef %5255, ptr noundef %5256)
  store i32 %5257, ptr %9, align 4
  %5258 = load ptr, ptr %4, align 8
  %5259 = getelementptr inbounds %struct.CallInfo, ptr %5258, i32 0, i32 0
  %5260 = load ptr, ptr %5259, align 8
  %5261 = getelementptr inbounds %union.StackValue, ptr %5260, i64 1
  store ptr %5261, ptr %7, align 8
  br label %5262

5262:                                             ; preds = %5254, %5248
  %5263 = load ptr, ptr %8, align 8
  %5264 = getelementptr inbounds i32, ptr %5263, i32 1
  store ptr %5264, ptr %8, align 8
  %5265 = load i32, ptr %5263, align 4
  store i32 %5265, ptr %10, align 4
  %5266 = load i32, ptr %10, align 4
  %5267 = lshr i32 %5266, 0
  %5268 = and i32 %5267, 127
  %5269 = zext i32 %5268 to i64
  %5270 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5269
  %5271 = load ptr, ptr %5270, align 8
  br label %7120

5272:                                             ; preds = %7120
  %5273 = load ptr, ptr %7, align 8
  %5274 = load i32, ptr %10, align 4
  %5275 = lshr i32 %5274, 7
  %5276 = and i32 %5275, 255
  %5277 = sext i32 %5276 to i64
  %5278 = getelementptr inbounds %union.StackValue, ptr %5273, i64 %5277
  store ptr %5278, ptr %317, align 8
  %5279 = load ptr, ptr %7, align 8
  %5280 = load i32, ptr %10, align 4
  %5281 = lshr i32 %5280, 16
  %5282 = and i32 %5281, 255
  %5283 = sext i32 %5282 to i64
  %5284 = getelementptr inbounds %union.StackValue, ptr %5279, i64 %5283
  store ptr %5284, ptr %318, align 8
  %5285 = load ptr, ptr %318, align 8
  %5286 = getelementptr inbounds %struct.TValue, ptr %5285, i32 0, i32 1
  %5287 = load i8, ptr %5286, align 8
  %5288 = zext i8 %5287 to i32
  %5289 = icmp eq i32 %5288, 3
  %5290 = zext i1 %5289 to i32
  %5291 = icmp ne i32 %5290, 0
  %5292 = zext i1 %5291 to i32
  %5293 = sext i32 %5292 to i64
  %5294 = icmp ne i64 %5293, 0
  br i1 %5294, label %5295, label %5299

5295:                                             ; preds = %5272
  %5296 = load ptr, ptr %318, align 8
  %5297 = getelementptr inbounds %struct.TValue, ptr %5296, i32 0, i32 0
  %5298 = load i64, ptr %5297, align 8
  store i64 %5298, ptr %319, align 8
  br i1 true, label %5303, label %5311

5299:                                             ; preds = %5272
  %5300 = load ptr, ptr %318, align 8
  %5301 = call i32 @luaV_tointegerns(ptr noundef %5300, ptr noundef %319, i32 noundef 0)
  %5302 = icmp ne i32 %5301, 0
  br i1 %5302, label %5303, label %5311

5303:                                             ; preds = %5299, %5295
  %5304 = load ptr, ptr %317, align 8
  store ptr %5304, ptr %320, align 8
  %5305 = load i64, ptr %319, align 8
  %5306 = xor i64 -1, %5305
  %5307 = load ptr, ptr %320, align 8
  %5308 = getelementptr inbounds %struct.TValue, ptr %5307, i32 0, i32 0
  store i64 %5306, ptr %5308, align 8
  %5309 = load ptr, ptr %320, align 8
  %5310 = getelementptr inbounds %struct.TValue, ptr %5309, i32 0, i32 1
  store i8 3, ptr %5310, align 8
  br label %5329

5311:                                             ; preds = %5299, %5295
  %5312 = load ptr, ptr %8, align 8
  %5313 = load ptr, ptr %4, align 8
  %5314 = getelementptr inbounds %struct.CallInfo, ptr %5313, i32 0, i32 4
  %5315 = getelementptr inbounds %struct.anon, ptr %5314, i32 0, i32 0
  store ptr %5312, ptr %5315, align 8
  %5316 = load ptr, ptr %4, align 8
  %5317 = getelementptr inbounds %struct.CallInfo, ptr %5316, i32 0, i32 1
  %5318 = load ptr, ptr %5317, align 8
  %5319 = load ptr, ptr %3, align 8
  %5320 = getelementptr inbounds %struct.lua_State, ptr %5319, i32 0, i32 6
  store ptr %5318, ptr %5320, align 8
  %5321 = load ptr, ptr %3, align 8
  %5322 = load ptr, ptr %318, align 8
  %5323 = load ptr, ptr %318, align 8
  %5324 = load ptr, ptr %317, align 8
  call void @luaT_trybinTM(ptr noundef %5321, ptr noundef %5322, ptr noundef %5323, ptr noundef %5324, i32 noundef 19)
  %5325 = load ptr, ptr %4, align 8
  %5326 = getelementptr inbounds %struct.CallInfo, ptr %5325, i32 0, i32 4
  %5327 = getelementptr inbounds %struct.anon, ptr %5326, i32 0, i32 1
  %5328 = load volatile i32, ptr %5327, align 8
  store i32 %5328, ptr %9, align 4
  br label %5329

5329:                                             ; preds = %5311, %5303
  %5330 = load i32, ptr %9, align 4
  %5331 = icmp ne i32 %5330, 0
  %5332 = zext i1 %5331 to i32
  %5333 = sext i32 %5332 to i64
  %5334 = icmp ne i64 %5333, 0
  br i1 %5334, label %5335, label %5343

5335:                                             ; preds = %5329
  %5336 = load ptr, ptr %3, align 8
  %5337 = load ptr, ptr %8, align 8
  %5338 = call i32 @luaG_traceexec(ptr noundef %5336, ptr noundef %5337)
  store i32 %5338, ptr %9, align 4
  %5339 = load ptr, ptr %4, align 8
  %5340 = getelementptr inbounds %struct.CallInfo, ptr %5339, i32 0, i32 0
  %5341 = load ptr, ptr %5340, align 8
  %5342 = getelementptr inbounds %union.StackValue, ptr %5341, i64 1
  store ptr %5342, ptr %7, align 8
  br label %5343

5343:                                             ; preds = %5335, %5329
  %5344 = load ptr, ptr %8, align 8
  %5345 = getelementptr inbounds i32, ptr %5344, i32 1
  store ptr %5345, ptr %8, align 8
  %5346 = load i32, ptr %5344, align 4
  store i32 %5346, ptr %10, align 4
  %5347 = load i32, ptr %10, align 4
  %5348 = lshr i32 %5347, 0
  %5349 = and i32 %5348, 127
  %5350 = zext i32 %5349 to i64
  %5351 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5350
  %5352 = load ptr, ptr %5351, align 8
  br label %7120

5353:                                             ; preds = %7120
  %5354 = load ptr, ptr %7, align 8
  %5355 = load i32, ptr %10, align 4
  %5356 = lshr i32 %5355, 7
  %5357 = and i32 %5356, 255
  %5358 = sext i32 %5357 to i64
  %5359 = getelementptr inbounds %union.StackValue, ptr %5354, i64 %5358
  store ptr %5359, ptr %321, align 8
  %5360 = load ptr, ptr %7, align 8
  %5361 = load i32, ptr %10, align 4
  %5362 = lshr i32 %5361, 16
  %5363 = and i32 %5362, 255
  %5364 = sext i32 %5363 to i64
  %5365 = getelementptr inbounds %union.StackValue, ptr %5360, i64 %5364
  store ptr %5365, ptr %322, align 8
  %5366 = load ptr, ptr %322, align 8
  %5367 = getelementptr inbounds %struct.TValue, ptr %5366, i32 0, i32 1
  %5368 = load i8, ptr %5367, align 8
  %5369 = zext i8 %5368 to i32
  %5370 = icmp eq i32 %5369, 1
  br i1 %5370, label %5378, label %5371

5371:                                             ; preds = %5353
  %5372 = load ptr, ptr %322, align 8
  %5373 = getelementptr inbounds %struct.TValue, ptr %5372, i32 0, i32 1
  %5374 = load i8, ptr %5373, align 8
  %5375 = zext i8 %5374 to i32
  %5376 = and i32 %5375, 15
  %5377 = icmp eq i32 %5376, 0
  br i1 %5377, label %5378, label %5381

5378:                                             ; preds = %5371, %5353
  %5379 = load ptr, ptr %321, align 8
  %5380 = getelementptr inbounds %struct.TValue, ptr %5379, i32 0, i32 1
  store i8 17, ptr %5380, align 8
  br label %5384

5381:                                             ; preds = %5371
  %5382 = load ptr, ptr %321, align 8
  %5383 = getelementptr inbounds %struct.TValue, ptr %5382, i32 0, i32 1
  store i8 1, ptr %5383, align 8
  br label %5384

5384:                                             ; preds = %5381, %5378
  %5385 = load i32, ptr %9, align 4
  %5386 = icmp ne i32 %5385, 0
  %5387 = zext i1 %5386 to i32
  %5388 = sext i32 %5387 to i64
  %5389 = icmp ne i64 %5388, 0
  br i1 %5389, label %5390, label %5398

5390:                                             ; preds = %5384
  %5391 = load ptr, ptr %3, align 8
  %5392 = load ptr, ptr %8, align 8
  %5393 = call i32 @luaG_traceexec(ptr noundef %5391, ptr noundef %5392)
  store i32 %5393, ptr %9, align 4
  %5394 = load ptr, ptr %4, align 8
  %5395 = getelementptr inbounds %struct.CallInfo, ptr %5394, i32 0, i32 0
  %5396 = load ptr, ptr %5395, align 8
  %5397 = getelementptr inbounds %union.StackValue, ptr %5396, i64 1
  store ptr %5397, ptr %7, align 8
  br label %5398

5398:                                             ; preds = %5390, %5384
  %5399 = load ptr, ptr %8, align 8
  %5400 = getelementptr inbounds i32, ptr %5399, i32 1
  store ptr %5400, ptr %8, align 8
  %5401 = load i32, ptr %5399, align 4
  store i32 %5401, ptr %10, align 4
  %5402 = load i32, ptr %10, align 4
  %5403 = lshr i32 %5402, 0
  %5404 = and i32 %5403, 127
  %5405 = zext i32 %5404 to i64
  %5406 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5405
  %5407 = load ptr, ptr %5406, align 8
  br label %7120

5408:                                             ; preds = %7120
  %5409 = load ptr, ptr %7, align 8
  %5410 = load i32, ptr %10, align 4
  %5411 = lshr i32 %5410, 7
  %5412 = and i32 %5411, 255
  %5413 = sext i32 %5412 to i64
  %5414 = getelementptr inbounds %union.StackValue, ptr %5409, i64 %5413
  store ptr %5414, ptr %323, align 8
  %5415 = load ptr, ptr %8, align 8
  %5416 = load ptr, ptr %4, align 8
  %5417 = getelementptr inbounds %struct.CallInfo, ptr %5416, i32 0, i32 4
  %5418 = getelementptr inbounds %struct.anon, ptr %5417, i32 0, i32 0
  store ptr %5415, ptr %5418, align 8
  %5419 = load ptr, ptr %4, align 8
  %5420 = getelementptr inbounds %struct.CallInfo, ptr %5419, i32 0, i32 1
  %5421 = load ptr, ptr %5420, align 8
  %5422 = load ptr, ptr %3, align 8
  %5423 = getelementptr inbounds %struct.lua_State, ptr %5422, i32 0, i32 6
  store ptr %5421, ptr %5423, align 8
  %5424 = load ptr, ptr %3, align 8
  %5425 = load ptr, ptr %323, align 8
  %5426 = load ptr, ptr %7, align 8
  %5427 = load i32, ptr %10, align 4
  %5428 = lshr i32 %5427, 16
  %5429 = and i32 %5428, 255
  %5430 = sext i32 %5429 to i64
  %5431 = getelementptr inbounds %union.StackValue, ptr %5426, i64 %5430
  call void @luaV_objlen(ptr noundef %5424, ptr noundef %5425, ptr noundef %5431)
  %5432 = load ptr, ptr %4, align 8
  %5433 = getelementptr inbounds %struct.CallInfo, ptr %5432, i32 0, i32 4
  %5434 = getelementptr inbounds %struct.anon, ptr %5433, i32 0, i32 1
  %5435 = load volatile i32, ptr %5434, align 8
  store i32 %5435, ptr %9, align 4
  %5436 = load i32, ptr %9, align 4
  %5437 = icmp ne i32 %5436, 0
  %5438 = zext i1 %5437 to i32
  %5439 = sext i32 %5438 to i64
  %5440 = icmp ne i64 %5439, 0
  br i1 %5440, label %5441, label %5449

5441:                                             ; preds = %5408
  %5442 = load ptr, ptr %3, align 8
  %5443 = load ptr, ptr %8, align 8
  %5444 = call i32 @luaG_traceexec(ptr noundef %5442, ptr noundef %5443)
  store i32 %5444, ptr %9, align 4
  %5445 = load ptr, ptr %4, align 8
  %5446 = getelementptr inbounds %struct.CallInfo, ptr %5445, i32 0, i32 0
  %5447 = load ptr, ptr %5446, align 8
  %5448 = getelementptr inbounds %union.StackValue, ptr %5447, i64 1
  store ptr %5448, ptr %7, align 8
  br label %5449

5449:                                             ; preds = %5441, %5408
  %5450 = load ptr, ptr %8, align 8
  %5451 = getelementptr inbounds i32, ptr %5450, i32 1
  store ptr %5451, ptr %8, align 8
  %5452 = load i32, ptr %5450, align 4
  store i32 %5452, ptr %10, align 4
  %5453 = load i32, ptr %10, align 4
  %5454 = lshr i32 %5453, 0
  %5455 = and i32 %5454, 127
  %5456 = zext i32 %5455 to i64
  %5457 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5456
  %5458 = load ptr, ptr %5457, align 8
  br label %7120

5459:                                             ; preds = %7120
  %5460 = load ptr, ptr %7, align 8
  %5461 = load i32, ptr %10, align 4
  %5462 = lshr i32 %5461, 7
  %5463 = and i32 %5462, 255
  %5464 = sext i32 %5463 to i64
  %5465 = getelementptr inbounds %union.StackValue, ptr %5460, i64 %5464
  store ptr %5465, ptr %324, align 8
  %5466 = load i32, ptr %10, align 4
  %5467 = lshr i32 %5466, 16
  %5468 = and i32 %5467, 255
  store i32 %5468, ptr %325, align 4
  %5469 = load ptr, ptr %324, align 8
  %5470 = load i32, ptr %325, align 4
  %5471 = sext i32 %5470 to i64
  %5472 = getelementptr inbounds %union.StackValue, ptr %5469, i64 %5471
  %5473 = load ptr, ptr %3, align 8
  %5474 = getelementptr inbounds %struct.lua_State, ptr %5473, i32 0, i32 6
  store ptr %5472, ptr %5474, align 8
  %5475 = load ptr, ptr %8, align 8
  %5476 = load ptr, ptr %4, align 8
  %5477 = getelementptr inbounds %struct.CallInfo, ptr %5476, i32 0, i32 4
  %5478 = getelementptr inbounds %struct.anon, ptr %5477, i32 0, i32 0
  store ptr %5475, ptr %5478, align 8
  %5479 = load ptr, ptr %3, align 8
  %5480 = load i32, ptr %325, align 4
  call void @luaV_concat(ptr noundef %5479, i32 noundef %5480)
  %5481 = load ptr, ptr %4, align 8
  %5482 = getelementptr inbounds %struct.CallInfo, ptr %5481, i32 0, i32 4
  %5483 = getelementptr inbounds %struct.anon, ptr %5482, i32 0, i32 1
  %5484 = load volatile i32, ptr %5483, align 8
  store i32 %5484, ptr %9, align 4
  %5485 = load ptr, ptr %3, align 8
  %5486 = getelementptr inbounds %struct.lua_State, ptr %5485, i32 0, i32 7
  %5487 = load ptr, ptr %5486, align 8
  %5488 = getelementptr inbounds %struct.global_State, ptr %5487, i32 0, i32 3
  %5489 = load i64, ptr %5488, align 8
  %5490 = icmp sgt i64 %5489, 0
  br i1 %5490, label %5491, label %5506

5491:                                             ; preds = %5459
  %5492 = load ptr, ptr %8, align 8
  %5493 = load ptr, ptr %4, align 8
  %5494 = getelementptr inbounds %struct.CallInfo, ptr %5493, i32 0, i32 4
  %5495 = getelementptr inbounds %struct.anon, ptr %5494, i32 0, i32 0
  store ptr %5492, ptr %5495, align 8
  %5496 = load ptr, ptr %3, align 8
  %5497 = getelementptr inbounds %struct.lua_State, ptr %5496, i32 0, i32 6
  %5498 = load ptr, ptr %5497, align 8
  %5499 = load ptr, ptr %3, align 8
  %5500 = getelementptr inbounds %struct.lua_State, ptr %5499, i32 0, i32 6
  store ptr %5498, ptr %5500, align 8
  %5501 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %5501)
  %5502 = load ptr, ptr %4, align 8
  %5503 = getelementptr inbounds %struct.CallInfo, ptr %5502, i32 0, i32 4
  %5504 = getelementptr inbounds %struct.anon, ptr %5503, i32 0, i32 1
  %5505 = load volatile i32, ptr %5504, align 8
  store i32 %5505, ptr %9, align 4
  br label %5506

5506:                                             ; preds = %5491, %5459
  %5507 = load i32, ptr %9, align 4
  %5508 = icmp ne i32 %5507, 0
  %5509 = zext i1 %5508 to i32
  %5510 = sext i32 %5509 to i64
  %5511 = icmp ne i64 %5510, 0
  br i1 %5511, label %5512, label %5520

5512:                                             ; preds = %5506
  %5513 = load ptr, ptr %3, align 8
  %5514 = load ptr, ptr %8, align 8
  %5515 = call i32 @luaG_traceexec(ptr noundef %5513, ptr noundef %5514)
  store i32 %5515, ptr %9, align 4
  %5516 = load ptr, ptr %4, align 8
  %5517 = getelementptr inbounds %struct.CallInfo, ptr %5516, i32 0, i32 0
  %5518 = load ptr, ptr %5517, align 8
  %5519 = getelementptr inbounds %union.StackValue, ptr %5518, i64 1
  store ptr %5519, ptr %7, align 8
  br label %5520

5520:                                             ; preds = %5512, %5506
  %5521 = load ptr, ptr %8, align 8
  %5522 = getelementptr inbounds i32, ptr %5521, i32 1
  store ptr %5522, ptr %8, align 8
  %5523 = load i32, ptr %5521, align 4
  store i32 %5523, ptr %10, align 4
  %5524 = load i32, ptr %10, align 4
  %5525 = lshr i32 %5524, 0
  %5526 = and i32 %5525, 127
  %5527 = zext i32 %5526 to i64
  %5528 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5527
  %5529 = load ptr, ptr %5528, align 8
  br label %7120

5530:                                             ; preds = %7120
  %5531 = load ptr, ptr %7, align 8
  %5532 = load i32, ptr %10, align 4
  %5533 = lshr i32 %5532, 7
  %5534 = and i32 %5533, 255
  %5535 = sext i32 %5534 to i64
  %5536 = getelementptr inbounds %union.StackValue, ptr %5531, i64 %5535
  store ptr %5536, ptr %326, align 8
  %5537 = load ptr, ptr %8, align 8
  %5538 = load ptr, ptr %4, align 8
  %5539 = getelementptr inbounds %struct.CallInfo, ptr %5538, i32 0, i32 4
  %5540 = getelementptr inbounds %struct.anon, ptr %5539, i32 0, i32 0
  store ptr %5537, ptr %5540, align 8
  %5541 = load ptr, ptr %4, align 8
  %5542 = getelementptr inbounds %struct.CallInfo, ptr %5541, i32 0, i32 1
  %5543 = load ptr, ptr %5542, align 8
  %5544 = load ptr, ptr %3, align 8
  %5545 = getelementptr inbounds %struct.lua_State, ptr %5544, i32 0, i32 6
  store ptr %5543, ptr %5545, align 8
  %5546 = load ptr, ptr %3, align 8
  %5547 = load ptr, ptr %326, align 8
  %5548 = call ptr @luaF_close(ptr noundef %5546, ptr noundef %5547, i32 noundef 0, i32 noundef 1)
  %5549 = load ptr, ptr %4, align 8
  %5550 = getelementptr inbounds %struct.CallInfo, ptr %5549, i32 0, i32 4
  %5551 = getelementptr inbounds %struct.anon, ptr %5550, i32 0, i32 1
  %5552 = load volatile i32, ptr %5551, align 8
  store i32 %5552, ptr %9, align 4
  %5553 = load i32, ptr %9, align 4
  %5554 = icmp ne i32 %5553, 0
  %5555 = zext i1 %5554 to i32
  %5556 = sext i32 %5555 to i64
  %5557 = icmp ne i64 %5556, 0
  br i1 %5557, label %5558, label %5566

5558:                                             ; preds = %5530
  %5559 = load ptr, ptr %3, align 8
  %5560 = load ptr, ptr %8, align 8
  %5561 = call i32 @luaG_traceexec(ptr noundef %5559, ptr noundef %5560)
  store i32 %5561, ptr %9, align 4
  %5562 = load ptr, ptr %4, align 8
  %5563 = getelementptr inbounds %struct.CallInfo, ptr %5562, i32 0, i32 0
  %5564 = load ptr, ptr %5563, align 8
  %5565 = getelementptr inbounds %union.StackValue, ptr %5564, i64 1
  store ptr %5565, ptr %7, align 8
  br label %5566

5566:                                             ; preds = %5558, %5530
  %5567 = load ptr, ptr %8, align 8
  %5568 = getelementptr inbounds i32, ptr %5567, i32 1
  store ptr %5568, ptr %8, align 8
  %5569 = load i32, ptr %5567, align 4
  store i32 %5569, ptr %10, align 4
  %5570 = load i32, ptr %10, align 4
  %5571 = lshr i32 %5570, 0
  %5572 = and i32 %5571, 127
  %5573 = zext i32 %5572 to i64
  %5574 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5573
  %5575 = load ptr, ptr %5574, align 8
  br label %7120

5576:                                             ; preds = %7120
  %5577 = load ptr, ptr %7, align 8
  %5578 = load i32, ptr %10, align 4
  %5579 = lshr i32 %5578, 7
  %5580 = and i32 %5579, 255
  %5581 = sext i32 %5580 to i64
  %5582 = getelementptr inbounds %union.StackValue, ptr %5577, i64 %5581
  store ptr %5582, ptr %327, align 8
  %5583 = load ptr, ptr %8, align 8
  %5584 = load ptr, ptr %4, align 8
  %5585 = getelementptr inbounds %struct.CallInfo, ptr %5584, i32 0, i32 4
  %5586 = getelementptr inbounds %struct.anon, ptr %5585, i32 0, i32 0
  store ptr %5583, ptr %5586, align 8
  %5587 = load ptr, ptr %4, align 8
  %5588 = getelementptr inbounds %struct.CallInfo, ptr %5587, i32 0, i32 1
  %5589 = load ptr, ptr %5588, align 8
  %5590 = load ptr, ptr %3, align 8
  %5591 = getelementptr inbounds %struct.lua_State, ptr %5590, i32 0, i32 6
  store ptr %5589, ptr %5591, align 8
  %5592 = load ptr, ptr %3, align 8
  %5593 = load ptr, ptr %327, align 8
  call void @luaF_newtbcupval(ptr noundef %5592, ptr noundef %5593)
  %5594 = load i32, ptr %9, align 4
  %5595 = icmp ne i32 %5594, 0
  %5596 = zext i1 %5595 to i32
  %5597 = sext i32 %5596 to i64
  %5598 = icmp ne i64 %5597, 0
  br i1 %5598, label %5599, label %5607

5599:                                             ; preds = %5576
  %5600 = load ptr, ptr %3, align 8
  %5601 = load ptr, ptr %8, align 8
  %5602 = call i32 @luaG_traceexec(ptr noundef %5600, ptr noundef %5601)
  store i32 %5602, ptr %9, align 4
  %5603 = load ptr, ptr %4, align 8
  %5604 = getelementptr inbounds %struct.CallInfo, ptr %5603, i32 0, i32 0
  %5605 = load ptr, ptr %5604, align 8
  %5606 = getelementptr inbounds %union.StackValue, ptr %5605, i64 1
  store ptr %5606, ptr %7, align 8
  br label %5607

5607:                                             ; preds = %5599, %5576
  %5608 = load ptr, ptr %8, align 8
  %5609 = getelementptr inbounds i32, ptr %5608, i32 1
  store ptr %5609, ptr %8, align 8
  %5610 = load i32, ptr %5608, align 4
  store i32 %5610, ptr %10, align 4
  %5611 = load i32, ptr %10, align 4
  %5612 = lshr i32 %5611, 0
  %5613 = and i32 %5612, 127
  %5614 = zext i32 %5613 to i64
  %5615 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5614
  %5616 = load ptr, ptr %5615, align 8
  br label %7120

5617:                                             ; preds = %7120
  %5618 = load i32, ptr %10, align 4
  %5619 = lshr i32 %5618, 7
  %5620 = and i32 %5619, 33554431
  %5621 = sub nsw i32 %5620, 16777215
  %5622 = add nsw i32 %5621, 0
  %5623 = load ptr, ptr %8, align 8
  %5624 = sext i32 %5622 to i64
  %5625 = getelementptr inbounds i32, ptr %5623, i64 %5624
  store ptr %5625, ptr %8, align 8
  %5626 = load ptr, ptr %4, align 8
  %5627 = getelementptr inbounds %struct.CallInfo, ptr %5626, i32 0, i32 4
  %5628 = getelementptr inbounds %struct.anon, ptr %5627, i32 0, i32 1
  %5629 = load volatile i32, ptr %5628, align 8
  store i32 %5629, ptr %9, align 4
  %5630 = load i32, ptr %9, align 4
  %5631 = icmp ne i32 %5630, 0
  %5632 = zext i1 %5631 to i32
  %5633 = sext i32 %5632 to i64
  %5634 = icmp ne i64 %5633, 0
  br i1 %5634, label %5635, label %5643

5635:                                             ; preds = %5617
  %5636 = load ptr, ptr %3, align 8
  %5637 = load ptr, ptr %8, align 8
  %5638 = call i32 @luaG_traceexec(ptr noundef %5636, ptr noundef %5637)
  store i32 %5638, ptr %9, align 4
  %5639 = load ptr, ptr %4, align 8
  %5640 = getelementptr inbounds %struct.CallInfo, ptr %5639, i32 0, i32 0
  %5641 = load ptr, ptr %5640, align 8
  %5642 = getelementptr inbounds %union.StackValue, ptr %5641, i64 1
  store ptr %5642, ptr %7, align 8
  br label %5643

5643:                                             ; preds = %5635, %5617
  %5644 = load ptr, ptr %8, align 8
  %5645 = getelementptr inbounds i32, ptr %5644, i32 1
  store ptr %5645, ptr %8, align 8
  %5646 = load i32, ptr %5644, align 4
  store i32 %5646, ptr %10, align 4
  %5647 = load i32, ptr %10, align 4
  %5648 = lshr i32 %5647, 0
  %5649 = and i32 %5648, 127
  %5650 = zext i32 %5649 to i64
  %5651 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5650
  %5652 = load ptr, ptr %5651, align 8
  br label %7120

5653:                                             ; preds = %7120
  %5654 = load ptr, ptr %7, align 8
  %5655 = load i32, ptr %10, align 4
  %5656 = lshr i32 %5655, 7
  %5657 = and i32 %5656, 255
  %5658 = sext i32 %5657 to i64
  %5659 = getelementptr inbounds %union.StackValue, ptr %5654, i64 %5658
  store ptr %5659, ptr %328, align 8
  %5660 = load ptr, ptr %7, align 8
  %5661 = load i32, ptr %10, align 4
  %5662 = lshr i32 %5661, 16
  %5663 = and i32 %5662, 255
  %5664 = sext i32 %5663 to i64
  %5665 = getelementptr inbounds %union.StackValue, ptr %5660, i64 %5664
  store ptr %5665, ptr %330, align 8
  %5666 = load ptr, ptr %8, align 8
  %5667 = load ptr, ptr %4, align 8
  %5668 = getelementptr inbounds %struct.CallInfo, ptr %5667, i32 0, i32 4
  %5669 = getelementptr inbounds %struct.anon, ptr %5668, i32 0, i32 0
  store ptr %5666, ptr %5669, align 8
  %5670 = load ptr, ptr %4, align 8
  %5671 = getelementptr inbounds %struct.CallInfo, ptr %5670, i32 0, i32 1
  %5672 = load ptr, ptr %5671, align 8
  %5673 = load ptr, ptr %3, align 8
  %5674 = getelementptr inbounds %struct.lua_State, ptr %5673, i32 0, i32 6
  store ptr %5672, ptr %5674, align 8
  %5675 = load ptr, ptr %3, align 8
  %5676 = load ptr, ptr %328, align 8
  %5677 = load ptr, ptr %330, align 8
  %5678 = call i32 @luaV_equalobj(ptr noundef %5675, ptr noundef %5676, ptr noundef %5677)
  store i32 %5678, ptr %329, align 4
  %5679 = load ptr, ptr %4, align 8
  %5680 = getelementptr inbounds %struct.CallInfo, ptr %5679, i32 0, i32 4
  %5681 = getelementptr inbounds %struct.anon, ptr %5680, i32 0, i32 1
  %5682 = load volatile i32, ptr %5681, align 8
  store i32 %5682, ptr %9, align 4
  %5683 = load i32, ptr %329, align 4
  %5684 = load i32, ptr %10, align 4
  %5685 = lshr i32 %5684, 15
  %5686 = and i32 %5685, 1
  %5687 = icmp ne i32 %5683, %5686
  br i1 %5687, label %5688, label %5691

5688:                                             ; preds = %5653
  %5689 = load ptr, ptr %8, align 8
  %5690 = getelementptr inbounds i32, ptr %5689, i32 1
  store ptr %5690, ptr %8, align 8
  br label %5706

5691:                                             ; preds = %5653
  %5692 = load ptr, ptr %8, align 8
  %5693 = load i32, ptr %5692, align 4
  store i32 %5693, ptr %331, align 4
  %5694 = load i32, ptr %331, align 4
  %5695 = lshr i32 %5694, 7
  %5696 = and i32 %5695, 33554431
  %5697 = sub nsw i32 %5696, 16777215
  %5698 = add nsw i32 %5697, 1
  %5699 = load ptr, ptr %8, align 8
  %5700 = sext i32 %5698 to i64
  %5701 = getelementptr inbounds i32, ptr %5699, i64 %5700
  store ptr %5701, ptr %8, align 8
  %5702 = load ptr, ptr %4, align 8
  %5703 = getelementptr inbounds %struct.CallInfo, ptr %5702, i32 0, i32 4
  %5704 = getelementptr inbounds %struct.anon, ptr %5703, i32 0, i32 1
  %5705 = load volatile i32, ptr %5704, align 8
  store i32 %5705, ptr %9, align 4
  br label %5706

5706:                                             ; preds = %5691, %5688
  %5707 = load i32, ptr %9, align 4
  %5708 = icmp ne i32 %5707, 0
  %5709 = zext i1 %5708 to i32
  %5710 = sext i32 %5709 to i64
  %5711 = icmp ne i64 %5710, 0
  br i1 %5711, label %5712, label %5720

5712:                                             ; preds = %5706
  %5713 = load ptr, ptr %3, align 8
  %5714 = load ptr, ptr %8, align 8
  %5715 = call i32 @luaG_traceexec(ptr noundef %5713, ptr noundef %5714)
  store i32 %5715, ptr %9, align 4
  %5716 = load ptr, ptr %4, align 8
  %5717 = getelementptr inbounds %struct.CallInfo, ptr %5716, i32 0, i32 0
  %5718 = load ptr, ptr %5717, align 8
  %5719 = getelementptr inbounds %union.StackValue, ptr %5718, i64 1
  store ptr %5719, ptr %7, align 8
  br label %5720

5720:                                             ; preds = %5712, %5706
  %5721 = load ptr, ptr %8, align 8
  %5722 = getelementptr inbounds i32, ptr %5721, i32 1
  store ptr %5722, ptr %8, align 8
  %5723 = load i32, ptr %5721, align 4
  store i32 %5723, ptr %10, align 4
  %5724 = load i32, ptr %10, align 4
  %5725 = lshr i32 %5724, 0
  %5726 = and i32 %5725, 127
  %5727 = zext i32 %5726 to i64
  %5728 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5727
  %5729 = load ptr, ptr %5728, align 8
  br label %7120

5730:                                             ; preds = %7120
  %5731 = load ptr, ptr %7, align 8
  %5732 = load i32, ptr %10, align 4
  %5733 = lshr i32 %5732, 7
  %5734 = and i32 %5733, 255
  %5735 = sext i32 %5734 to i64
  %5736 = getelementptr inbounds %union.StackValue, ptr %5731, i64 %5735
  store ptr %5736, ptr %332, align 8
  %5737 = load ptr, ptr %7, align 8
  %5738 = load i32, ptr %10, align 4
  %5739 = lshr i32 %5738, 16
  %5740 = and i32 %5739, 255
  %5741 = sext i32 %5740 to i64
  %5742 = getelementptr inbounds %union.StackValue, ptr %5737, i64 %5741
  store ptr %5742, ptr %334, align 8
  %5743 = load ptr, ptr %332, align 8
  %5744 = getelementptr inbounds %struct.TValue, ptr %5743, i32 0, i32 1
  %5745 = load i8, ptr %5744, align 8
  %5746 = zext i8 %5745 to i32
  %5747 = icmp eq i32 %5746, 3
  br i1 %5747, label %5748, label %5765

5748:                                             ; preds = %5730
  %5749 = load ptr, ptr %334, align 8
  %5750 = getelementptr inbounds %struct.TValue, ptr %5749, i32 0, i32 1
  %5751 = load i8, ptr %5750, align 8
  %5752 = zext i8 %5751 to i32
  %5753 = icmp eq i32 %5752, 3
  br i1 %5753, label %5754, label %5765

5754:                                             ; preds = %5748
  %5755 = load ptr, ptr %332, align 8
  %5756 = getelementptr inbounds %struct.TValue, ptr %5755, i32 0, i32 0
  %5757 = load i64, ptr %5756, align 8
  store i64 %5757, ptr %335, align 8
  %5758 = load ptr, ptr %334, align 8
  %5759 = getelementptr inbounds %struct.TValue, ptr %5758, i32 0, i32 0
  %5760 = load i64, ptr %5759, align 8
  store i64 %5760, ptr %336, align 8
  %5761 = load i64, ptr %335, align 8
  %5762 = load i64, ptr %336, align 8
  %5763 = icmp slt i64 %5761, %5762
  %5764 = zext i1 %5763 to i32
  store i32 %5764, ptr %333, align 4
  br label %5802

5765:                                             ; preds = %5748, %5730
  %5766 = load ptr, ptr %332, align 8
  %5767 = getelementptr inbounds %struct.TValue, ptr %5766, i32 0, i32 1
  %5768 = load i8, ptr %5767, align 8
  %5769 = zext i8 %5768 to i32
  %5770 = and i32 %5769, 15
  %5771 = icmp eq i32 %5770, 3
  br i1 %5771, label %5772, label %5783

5772:                                             ; preds = %5765
  %5773 = load ptr, ptr %334, align 8
  %5774 = getelementptr inbounds %struct.TValue, ptr %5773, i32 0, i32 1
  %5775 = load i8, ptr %5774, align 8
  %5776 = zext i8 %5775 to i32
  %5777 = and i32 %5776, 15
  %5778 = icmp eq i32 %5777, 3
  br i1 %5778, label %5779, label %5783

5779:                                             ; preds = %5772
  %5780 = load ptr, ptr %332, align 8
  %5781 = load ptr, ptr %334, align 8
  %5782 = call i32 @LTnum(ptr noundef %5780, ptr noundef %5781)
  store i32 %5782, ptr %333, align 4
  br label %5801

5783:                                             ; preds = %5772, %5765
  %5784 = load ptr, ptr %8, align 8
  %5785 = load ptr, ptr %4, align 8
  %5786 = getelementptr inbounds %struct.CallInfo, ptr %5785, i32 0, i32 4
  %5787 = getelementptr inbounds %struct.anon, ptr %5786, i32 0, i32 0
  store ptr %5784, ptr %5787, align 8
  %5788 = load ptr, ptr %4, align 8
  %5789 = getelementptr inbounds %struct.CallInfo, ptr %5788, i32 0, i32 1
  %5790 = load ptr, ptr %5789, align 8
  %5791 = load ptr, ptr %3, align 8
  %5792 = getelementptr inbounds %struct.lua_State, ptr %5791, i32 0, i32 6
  store ptr %5790, ptr %5792, align 8
  %5793 = load ptr, ptr %3, align 8
  %5794 = load ptr, ptr %332, align 8
  %5795 = load ptr, ptr %334, align 8
  %5796 = call i32 @lessthanothers(ptr noundef %5793, ptr noundef %5794, ptr noundef %5795)
  store i32 %5796, ptr %333, align 4
  %5797 = load ptr, ptr %4, align 8
  %5798 = getelementptr inbounds %struct.CallInfo, ptr %5797, i32 0, i32 4
  %5799 = getelementptr inbounds %struct.anon, ptr %5798, i32 0, i32 1
  %5800 = load volatile i32, ptr %5799, align 8
  store i32 %5800, ptr %9, align 4
  br label %5801

5801:                                             ; preds = %5783, %5779
  br label %5802

5802:                                             ; preds = %5801, %5754
  %5803 = load i32, ptr %333, align 4
  %5804 = load i32, ptr %10, align 4
  %5805 = lshr i32 %5804, 15
  %5806 = and i32 %5805, 1
  %5807 = icmp ne i32 %5803, %5806
  br i1 %5807, label %5808, label %5811

5808:                                             ; preds = %5802
  %5809 = load ptr, ptr %8, align 8
  %5810 = getelementptr inbounds i32, ptr %5809, i32 1
  store ptr %5810, ptr %8, align 8
  br label %5826

5811:                                             ; preds = %5802
  %5812 = load ptr, ptr %8, align 8
  %5813 = load i32, ptr %5812, align 4
  store i32 %5813, ptr %337, align 4
  %5814 = load i32, ptr %337, align 4
  %5815 = lshr i32 %5814, 7
  %5816 = and i32 %5815, 33554431
  %5817 = sub nsw i32 %5816, 16777215
  %5818 = add nsw i32 %5817, 1
  %5819 = load ptr, ptr %8, align 8
  %5820 = sext i32 %5818 to i64
  %5821 = getelementptr inbounds i32, ptr %5819, i64 %5820
  store ptr %5821, ptr %8, align 8
  %5822 = load ptr, ptr %4, align 8
  %5823 = getelementptr inbounds %struct.CallInfo, ptr %5822, i32 0, i32 4
  %5824 = getelementptr inbounds %struct.anon, ptr %5823, i32 0, i32 1
  %5825 = load volatile i32, ptr %5824, align 8
  store i32 %5825, ptr %9, align 4
  br label %5826

5826:                                             ; preds = %5811, %5808
  %5827 = load i32, ptr %9, align 4
  %5828 = icmp ne i32 %5827, 0
  %5829 = zext i1 %5828 to i32
  %5830 = sext i32 %5829 to i64
  %5831 = icmp ne i64 %5830, 0
  br i1 %5831, label %5832, label %5840

5832:                                             ; preds = %5826
  %5833 = load ptr, ptr %3, align 8
  %5834 = load ptr, ptr %8, align 8
  %5835 = call i32 @luaG_traceexec(ptr noundef %5833, ptr noundef %5834)
  store i32 %5835, ptr %9, align 4
  %5836 = load ptr, ptr %4, align 8
  %5837 = getelementptr inbounds %struct.CallInfo, ptr %5836, i32 0, i32 0
  %5838 = load ptr, ptr %5837, align 8
  %5839 = getelementptr inbounds %union.StackValue, ptr %5838, i64 1
  store ptr %5839, ptr %7, align 8
  br label %5840

5840:                                             ; preds = %5832, %5826
  %5841 = load ptr, ptr %8, align 8
  %5842 = getelementptr inbounds i32, ptr %5841, i32 1
  store ptr %5842, ptr %8, align 8
  %5843 = load i32, ptr %5841, align 4
  store i32 %5843, ptr %10, align 4
  %5844 = load i32, ptr %10, align 4
  %5845 = lshr i32 %5844, 0
  %5846 = and i32 %5845, 127
  %5847 = zext i32 %5846 to i64
  %5848 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5847
  %5849 = load ptr, ptr %5848, align 8
  br label %7120

5850:                                             ; preds = %7120
  %5851 = load ptr, ptr %7, align 8
  %5852 = load i32, ptr %10, align 4
  %5853 = lshr i32 %5852, 7
  %5854 = and i32 %5853, 255
  %5855 = sext i32 %5854 to i64
  %5856 = getelementptr inbounds %union.StackValue, ptr %5851, i64 %5855
  store ptr %5856, ptr %338, align 8
  %5857 = load ptr, ptr %7, align 8
  %5858 = load i32, ptr %10, align 4
  %5859 = lshr i32 %5858, 16
  %5860 = and i32 %5859, 255
  %5861 = sext i32 %5860 to i64
  %5862 = getelementptr inbounds %union.StackValue, ptr %5857, i64 %5861
  store ptr %5862, ptr %340, align 8
  %5863 = load ptr, ptr %338, align 8
  %5864 = getelementptr inbounds %struct.TValue, ptr %5863, i32 0, i32 1
  %5865 = load i8, ptr %5864, align 8
  %5866 = zext i8 %5865 to i32
  %5867 = icmp eq i32 %5866, 3
  br i1 %5867, label %5868, label %5885

5868:                                             ; preds = %5850
  %5869 = load ptr, ptr %340, align 8
  %5870 = getelementptr inbounds %struct.TValue, ptr %5869, i32 0, i32 1
  %5871 = load i8, ptr %5870, align 8
  %5872 = zext i8 %5871 to i32
  %5873 = icmp eq i32 %5872, 3
  br i1 %5873, label %5874, label %5885

5874:                                             ; preds = %5868
  %5875 = load ptr, ptr %338, align 8
  %5876 = getelementptr inbounds %struct.TValue, ptr %5875, i32 0, i32 0
  %5877 = load i64, ptr %5876, align 8
  store i64 %5877, ptr %341, align 8
  %5878 = load ptr, ptr %340, align 8
  %5879 = getelementptr inbounds %struct.TValue, ptr %5878, i32 0, i32 0
  %5880 = load i64, ptr %5879, align 8
  store i64 %5880, ptr %342, align 8
  %5881 = load i64, ptr %341, align 8
  %5882 = load i64, ptr %342, align 8
  %5883 = icmp sle i64 %5881, %5882
  %5884 = zext i1 %5883 to i32
  store i32 %5884, ptr %339, align 4
  br label %5922

5885:                                             ; preds = %5868, %5850
  %5886 = load ptr, ptr %338, align 8
  %5887 = getelementptr inbounds %struct.TValue, ptr %5886, i32 0, i32 1
  %5888 = load i8, ptr %5887, align 8
  %5889 = zext i8 %5888 to i32
  %5890 = and i32 %5889, 15
  %5891 = icmp eq i32 %5890, 3
  br i1 %5891, label %5892, label %5903

5892:                                             ; preds = %5885
  %5893 = load ptr, ptr %340, align 8
  %5894 = getelementptr inbounds %struct.TValue, ptr %5893, i32 0, i32 1
  %5895 = load i8, ptr %5894, align 8
  %5896 = zext i8 %5895 to i32
  %5897 = and i32 %5896, 15
  %5898 = icmp eq i32 %5897, 3
  br i1 %5898, label %5899, label %5903

5899:                                             ; preds = %5892
  %5900 = load ptr, ptr %338, align 8
  %5901 = load ptr, ptr %340, align 8
  %5902 = call i32 @LEnum(ptr noundef %5900, ptr noundef %5901)
  store i32 %5902, ptr %339, align 4
  br label %5921

5903:                                             ; preds = %5892, %5885
  %5904 = load ptr, ptr %8, align 8
  %5905 = load ptr, ptr %4, align 8
  %5906 = getelementptr inbounds %struct.CallInfo, ptr %5905, i32 0, i32 4
  %5907 = getelementptr inbounds %struct.anon, ptr %5906, i32 0, i32 0
  store ptr %5904, ptr %5907, align 8
  %5908 = load ptr, ptr %4, align 8
  %5909 = getelementptr inbounds %struct.CallInfo, ptr %5908, i32 0, i32 1
  %5910 = load ptr, ptr %5909, align 8
  %5911 = load ptr, ptr %3, align 8
  %5912 = getelementptr inbounds %struct.lua_State, ptr %5911, i32 0, i32 6
  store ptr %5910, ptr %5912, align 8
  %5913 = load ptr, ptr %3, align 8
  %5914 = load ptr, ptr %338, align 8
  %5915 = load ptr, ptr %340, align 8
  %5916 = call i32 @lessequalothers(ptr noundef %5913, ptr noundef %5914, ptr noundef %5915)
  store i32 %5916, ptr %339, align 4
  %5917 = load ptr, ptr %4, align 8
  %5918 = getelementptr inbounds %struct.CallInfo, ptr %5917, i32 0, i32 4
  %5919 = getelementptr inbounds %struct.anon, ptr %5918, i32 0, i32 1
  %5920 = load volatile i32, ptr %5919, align 8
  store i32 %5920, ptr %9, align 4
  br label %5921

5921:                                             ; preds = %5903, %5899
  br label %5922

5922:                                             ; preds = %5921, %5874
  %5923 = load i32, ptr %339, align 4
  %5924 = load i32, ptr %10, align 4
  %5925 = lshr i32 %5924, 15
  %5926 = and i32 %5925, 1
  %5927 = icmp ne i32 %5923, %5926
  br i1 %5927, label %5928, label %5931

5928:                                             ; preds = %5922
  %5929 = load ptr, ptr %8, align 8
  %5930 = getelementptr inbounds i32, ptr %5929, i32 1
  store ptr %5930, ptr %8, align 8
  br label %5946

5931:                                             ; preds = %5922
  %5932 = load ptr, ptr %8, align 8
  %5933 = load i32, ptr %5932, align 4
  store i32 %5933, ptr %343, align 4
  %5934 = load i32, ptr %343, align 4
  %5935 = lshr i32 %5934, 7
  %5936 = and i32 %5935, 33554431
  %5937 = sub nsw i32 %5936, 16777215
  %5938 = add nsw i32 %5937, 1
  %5939 = load ptr, ptr %8, align 8
  %5940 = sext i32 %5938 to i64
  %5941 = getelementptr inbounds i32, ptr %5939, i64 %5940
  store ptr %5941, ptr %8, align 8
  %5942 = load ptr, ptr %4, align 8
  %5943 = getelementptr inbounds %struct.CallInfo, ptr %5942, i32 0, i32 4
  %5944 = getelementptr inbounds %struct.anon, ptr %5943, i32 0, i32 1
  %5945 = load volatile i32, ptr %5944, align 8
  store i32 %5945, ptr %9, align 4
  br label %5946

5946:                                             ; preds = %5931, %5928
  %5947 = load i32, ptr %9, align 4
  %5948 = icmp ne i32 %5947, 0
  %5949 = zext i1 %5948 to i32
  %5950 = sext i32 %5949 to i64
  %5951 = icmp ne i64 %5950, 0
  br i1 %5951, label %5952, label %5960

5952:                                             ; preds = %5946
  %5953 = load ptr, ptr %3, align 8
  %5954 = load ptr, ptr %8, align 8
  %5955 = call i32 @luaG_traceexec(ptr noundef %5953, ptr noundef %5954)
  store i32 %5955, ptr %9, align 4
  %5956 = load ptr, ptr %4, align 8
  %5957 = getelementptr inbounds %struct.CallInfo, ptr %5956, i32 0, i32 0
  %5958 = load ptr, ptr %5957, align 8
  %5959 = getelementptr inbounds %union.StackValue, ptr %5958, i64 1
  store ptr %5959, ptr %7, align 8
  br label %5960

5960:                                             ; preds = %5952, %5946
  %5961 = load ptr, ptr %8, align 8
  %5962 = getelementptr inbounds i32, ptr %5961, i32 1
  store ptr %5962, ptr %8, align 8
  %5963 = load i32, ptr %5961, align 4
  store i32 %5963, ptr %10, align 4
  %5964 = load i32, ptr %10, align 4
  %5965 = lshr i32 %5964, 0
  %5966 = and i32 %5965, 127
  %5967 = zext i32 %5966 to i64
  %5968 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %5967
  %5969 = load ptr, ptr %5968, align 8
  br label %7120

5970:                                             ; preds = %7120
  %5971 = load ptr, ptr %7, align 8
  %5972 = load i32, ptr %10, align 4
  %5973 = lshr i32 %5972, 7
  %5974 = and i32 %5973, 255
  %5975 = sext i32 %5974 to i64
  %5976 = getelementptr inbounds %union.StackValue, ptr %5971, i64 %5975
  store ptr %5976, ptr %344, align 8
  %5977 = load ptr, ptr %6, align 8
  %5978 = load i32, ptr %10, align 4
  %5979 = lshr i32 %5978, 16
  %5980 = and i32 %5979, 255
  %5981 = sext i32 %5980 to i64
  %5982 = getelementptr inbounds %struct.TValue, ptr %5977, i64 %5981
  store ptr %5982, ptr %345, align 8
  %5983 = load ptr, ptr %344, align 8
  %5984 = load ptr, ptr %345, align 8
  %5985 = call i32 @luaV_equalobj(ptr noundef null, ptr noundef %5983, ptr noundef %5984)
  store i32 %5985, ptr %346, align 4
  %5986 = load i32, ptr %346, align 4
  %5987 = load i32, ptr %10, align 4
  %5988 = lshr i32 %5987, 15
  %5989 = and i32 %5988, 1
  %5990 = icmp ne i32 %5986, %5989
  br i1 %5990, label %5991, label %5994

5991:                                             ; preds = %5970
  %5992 = load ptr, ptr %8, align 8
  %5993 = getelementptr inbounds i32, ptr %5992, i32 1
  store ptr %5993, ptr %8, align 8
  br label %6009

5994:                                             ; preds = %5970
  %5995 = load ptr, ptr %8, align 8
  %5996 = load i32, ptr %5995, align 4
  store i32 %5996, ptr %347, align 4
  %5997 = load i32, ptr %347, align 4
  %5998 = lshr i32 %5997, 7
  %5999 = and i32 %5998, 33554431
  %6000 = sub nsw i32 %5999, 16777215
  %6001 = add nsw i32 %6000, 1
  %6002 = load ptr, ptr %8, align 8
  %6003 = sext i32 %6001 to i64
  %6004 = getelementptr inbounds i32, ptr %6002, i64 %6003
  store ptr %6004, ptr %8, align 8
  %6005 = load ptr, ptr %4, align 8
  %6006 = getelementptr inbounds %struct.CallInfo, ptr %6005, i32 0, i32 4
  %6007 = getelementptr inbounds %struct.anon, ptr %6006, i32 0, i32 1
  %6008 = load volatile i32, ptr %6007, align 8
  store i32 %6008, ptr %9, align 4
  br label %6009

6009:                                             ; preds = %5994, %5991
  %6010 = load i32, ptr %9, align 4
  %6011 = icmp ne i32 %6010, 0
  %6012 = zext i1 %6011 to i32
  %6013 = sext i32 %6012 to i64
  %6014 = icmp ne i64 %6013, 0
  br i1 %6014, label %6015, label %6023

6015:                                             ; preds = %6009
  %6016 = load ptr, ptr %3, align 8
  %6017 = load ptr, ptr %8, align 8
  %6018 = call i32 @luaG_traceexec(ptr noundef %6016, ptr noundef %6017)
  store i32 %6018, ptr %9, align 4
  %6019 = load ptr, ptr %4, align 8
  %6020 = getelementptr inbounds %struct.CallInfo, ptr %6019, i32 0, i32 0
  %6021 = load ptr, ptr %6020, align 8
  %6022 = getelementptr inbounds %union.StackValue, ptr %6021, i64 1
  store ptr %6022, ptr %7, align 8
  br label %6023

6023:                                             ; preds = %6015, %6009
  %6024 = load ptr, ptr %8, align 8
  %6025 = getelementptr inbounds i32, ptr %6024, i32 1
  store ptr %6025, ptr %8, align 8
  %6026 = load i32, ptr %6024, align 4
  store i32 %6026, ptr %10, align 4
  %6027 = load i32, ptr %10, align 4
  %6028 = lshr i32 %6027, 0
  %6029 = and i32 %6028, 127
  %6030 = zext i32 %6029 to i64
  %6031 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6030
  %6032 = load ptr, ptr %6031, align 8
  br label %7120

6033:                                             ; preds = %7120
  %6034 = load ptr, ptr %7, align 8
  %6035 = load i32, ptr %10, align 4
  %6036 = lshr i32 %6035, 7
  %6037 = and i32 %6036, 255
  %6038 = sext i32 %6037 to i64
  %6039 = getelementptr inbounds %union.StackValue, ptr %6034, i64 %6038
  store ptr %6039, ptr %348, align 8
  %6040 = load i32, ptr %10, align 4
  %6041 = lshr i32 %6040, 16
  %6042 = and i32 %6041, 255
  %6043 = sub nsw i32 %6042, 127
  store i32 %6043, ptr %350, align 4
  %6044 = load ptr, ptr %348, align 8
  %6045 = getelementptr inbounds %struct.TValue, ptr %6044, i32 0, i32 1
  %6046 = load i8, ptr %6045, align 8
  %6047 = zext i8 %6046 to i32
  %6048 = icmp eq i32 %6047, 3
  br i1 %6048, label %6049, label %6057

6049:                                             ; preds = %6033
  %6050 = load ptr, ptr %348, align 8
  %6051 = getelementptr inbounds %struct.TValue, ptr %6050, i32 0, i32 0
  %6052 = load i64, ptr %6051, align 8
  %6053 = load i32, ptr %350, align 4
  %6054 = sext i32 %6053 to i64
  %6055 = icmp eq i64 %6052, %6054
  %6056 = zext i1 %6055 to i32
  store i32 %6056, ptr %349, align 4
  br label %6073

6057:                                             ; preds = %6033
  %6058 = load ptr, ptr %348, align 8
  %6059 = getelementptr inbounds %struct.TValue, ptr %6058, i32 0, i32 1
  %6060 = load i8, ptr %6059, align 8
  %6061 = zext i8 %6060 to i32
  %6062 = icmp eq i32 %6061, 19
  br i1 %6062, label %6063, label %6071

6063:                                             ; preds = %6057
  %6064 = load ptr, ptr %348, align 8
  %6065 = getelementptr inbounds %struct.TValue, ptr %6064, i32 0, i32 0
  %6066 = load double, ptr %6065, align 8
  %6067 = load i32, ptr %350, align 4
  %6068 = sitofp i32 %6067 to double
  %6069 = fcmp oeq double %6066, %6068
  %6070 = zext i1 %6069 to i32
  store i32 %6070, ptr %349, align 4
  br label %6072

6071:                                             ; preds = %6057
  store i32 0, ptr %349, align 4
  br label %6072

6072:                                             ; preds = %6071, %6063
  br label %6073

6073:                                             ; preds = %6072, %6049
  %6074 = load i32, ptr %349, align 4
  %6075 = load i32, ptr %10, align 4
  %6076 = lshr i32 %6075, 15
  %6077 = and i32 %6076, 1
  %6078 = icmp ne i32 %6074, %6077
  br i1 %6078, label %6079, label %6082

6079:                                             ; preds = %6073
  %6080 = load ptr, ptr %8, align 8
  %6081 = getelementptr inbounds i32, ptr %6080, i32 1
  store ptr %6081, ptr %8, align 8
  br label %6097

6082:                                             ; preds = %6073
  %6083 = load ptr, ptr %8, align 8
  %6084 = load i32, ptr %6083, align 4
  store i32 %6084, ptr %351, align 4
  %6085 = load i32, ptr %351, align 4
  %6086 = lshr i32 %6085, 7
  %6087 = and i32 %6086, 33554431
  %6088 = sub nsw i32 %6087, 16777215
  %6089 = add nsw i32 %6088, 1
  %6090 = load ptr, ptr %8, align 8
  %6091 = sext i32 %6089 to i64
  %6092 = getelementptr inbounds i32, ptr %6090, i64 %6091
  store ptr %6092, ptr %8, align 8
  %6093 = load ptr, ptr %4, align 8
  %6094 = getelementptr inbounds %struct.CallInfo, ptr %6093, i32 0, i32 4
  %6095 = getelementptr inbounds %struct.anon, ptr %6094, i32 0, i32 1
  %6096 = load volatile i32, ptr %6095, align 8
  store i32 %6096, ptr %9, align 4
  br label %6097

6097:                                             ; preds = %6082, %6079
  %6098 = load i32, ptr %9, align 4
  %6099 = icmp ne i32 %6098, 0
  %6100 = zext i1 %6099 to i32
  %6101 = sext i32 %6100 to i64
  %6102 = icmp ne i64 %6101, 0
  br i1 %6102, label %6103, label %6111

6103:                                             ; preds = %6097
  %6104 = load ptr, ptr %3, align 8
  %6105 = load ptr, ptr %8, align 8
  %6106 = call i32 @luaG_traceexec(ptr noundef %6104, ptr noundef %6105)
  store i32 %6106, ptr %9, align 4
  %6107 = load ptr, ptr %4, align 8
  %6108 = getelementptr inbounds %struct.CallInfo, ptr %6107, i32 0, i32 0
  %6109 = load ptr, ptr %6108, align 8
  %6110 = getelementptr inbounds %union.StackValue, ptr %6109, i64 1
  store ptr %6110, ptr %7, align 8
  br label %6111

6111:                                             ; preds = %6103, %6097
  %6112 = load ptr, ptr %8, align 8
  %6113 = getelementptr inbounds i32, ptr %6112, i32 1
  store ptr %6113, ptr %8, align 8
  %6114 = load i32, ptr %6112, align 4
  store i32 %6114, ptr %10, align 4
  %6115 = load i32, ptr %10, align 4
  %6116 = lshr i32 %6115, 0
  %6117 = and i32 %6116, 127
  %6118 = zext i32 %6117 to i64
  %6119 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6118
  %6120 = load ptr, ptr %6119, align 8
  br label %7120

6121:                                             ; preds = %7120
  %6122 = load ptr, ptr %7, align 8
  %6123 = load i32, ptr %10, align 4
  %6124 = lshr i32 %6123, 7
  %6125 = and i32 %6124, 255
  %6126 = sext i32 %6125 to i64
  %6127 = getelementptr inbounds %union.StackValue, ptr %6122, i64 %6126
  store ptr %6127, ptr %352, align 8
  %6128 = load i32, ptr %10, align 4
  %6129 = lshr i32 %6128, 16
  %6130 = and i32 %6129, 255
  %6131 = sub nsw i32 %6130, 127
  store i32 %6131, ptr %354, align 4
  %6132 = load ptr, ptr %352, align 8
  %6133 = getelementptr inbounds %struct.TValue, ptr %6132, i32 0, i32 1
  %6134 = load i8, ptr %6133, align 8
  %6135 = zext i8 %6134 to i32
  %6136 = icmp eq i32 %6135, 3
  br i1 %6136, label %6137, label %6145

6137:                                             ; preds = %6121
  %6138 = load ptr, ptr %352, align 8
  %6139 = getelementptr inbounds %struct.TValue, ptr %6138, i32 0, i32 0
  %6140 = load i64, ptr %6139, align 8
  %6141 = load i32, ptr %354, align 4
  %6142 = sext i32 %6141 to i64
  %6143 = icmp slt i64 %6140, %6142
  %6144 = zext i1 %6143 to i32
  store i32 %6144, ptr %353, align 4
  br label %6184

6145:                                             ; preds = %6121
  %6146 = load ptr, ptr %352, align 8
  %6147 = getelementptr inbounds %struct.TValue, ptr %6146, i32 0, i32 1
  %6148 = load i8, ptr %6147, align 8
  %6149 = zext i8 %6148 to i32
  %6150 = icmp eq i32 %6149, 19
  br i1 %6150, label %6151, label %6161

6151:                                             ; preds = %6145
  %6152 = load ptr, ptr %352, align 8
  %6153 = getelementptr inbounds %struct.TValue, ptr %6152, i32 0, i32 0
  %6154 = load double, ptr %6153, align 8
  store double %6154, ptr %355, align 8
  %6155 = load i32, ptr %354, align 4
  %6156 = sitofp i32 %6155 to double
  store double %6156, ptr %356, align 8
  %6157 = load double, ptr %355, align 8
  %6158 = load double, ptr %356, align 8
  %6159 = fcmp olt double %6157, %6158
  %6160 = zext i1 %6159 to i32
  store i32 %6160, ptr %353, align 4
  br label %6183

6161:                                             ; preds = %6145
  %6162 = load i32, ptr %10, align 4
  %6163 = lshr i32 %6162, 24
  %6164 = and i32 %6163, 255
  store i32 %6164, ptr %357, align 4
  %6165 = load ptr, ptr %8, align 8
  %6166 = load ptr, ptr %4, align 8
  %6167 = getelementptr inbounds %struct.CallInfo, ptr %6166, i32 0, i32 4
  %6168 = getelementptr inbounds %struct.anon, ptr %6167, i32 0, i32 0
  store ptr %6165, ptr %6168, align 8
  %6169 = load ptr, ptr %4, align 8
  %6170 = getelementptr inbounds %struct.CallInfo, ptr %6169, i32 0, i32 1
  %6171 = load ptr, ptr %6170, align 8
  %6172 = load ptr, ptr %3, align 8
  %6173 = getelementptr inbounds %struct.lua_State, ptr %6172, i32 0, i32 6
  store ptr %6171, ptr %6173, align 8
  %6174 = load ptr, ptr %3, align 8
  %6175 = load ptr, ptr %352, align 8
  %6176 = load i32, ptr %354, align 4
  %6177 = load i32, ptr %357, align 4
  %6178 = call i32 @luaT_callorderiTM(ptr noundef %6174, ptr noundef %6175, i32 noundef %6176, i32 noundef 0, i32 noundef %6177, i32 noundef 20)
  store i32 %6178, ptr %353, align 4
  %6179 = load ptr, ptr %4, align 8
  %6180 = getelementptr inbounds %struct.CallInfo, ptr %6179, i32 0, i32 4
  %6181 = getelementptr inbounds %struct.anon, ptr %6180, i32 0, i32 1
  %6182 = load volatile i32, ptr %6181, align 8
  store i32 %6182, ptr %9, align 4
  br label %6183

6183:                                             ; preds = %6161, %6151
  br label %6184

6184:                                             ; preds = %6183, %6137
  %6185 = load i32, ptr %353, align 4
  %6186 = load i32, ptr %10, align 4
  %6187 = lshr i32 %6186, 15
  %6188 = and i32 %6187, 1
  %6189 = icmp ne i32 %6185, %6188
  br i1 %6189, label %6190, label %6193

6190:                                             ; preds = %6184
  %6191 = load ptr, ptr %8, align 8
  %6192 = getelementptr inbounds i32, ptr %6191, i32 1
  store ptr %6192, ptr %8, align 8
  br label %6208

6193:                                             ; preds = %6184
  %6194 = load ptr, ptr %8, align 8
  %6195 = load i32, ptr %6194, align 4
  store i32 %6195, ptr %358, align 4
  %6196 = load i32, ptr %358, align 4
  %6197 = lshr i32 %6196, 7
  %6198 = and i32 %6197, 33554431
  %6199 = sub nsw i32 %6198, 16777215
  %6200 = add nsw i32 %6199, 1
  %6201 = load ptr, ptr %8, align 8
  %6202 = sext i32 %6200 to i64
  %6203 = getelementptr inbounds i32, ptr %6201, i64 %6202
  store ptr %6203, ptr %8, align 8
  %6204 = load ptr, ptr %4, align 8
  %6205 = getelementptr inbounds %struct.CallInfo, ptr %6204, i32 0, i32 4
  %6206 = getelementptr inbounds %struct.anon, ptr %6205, i32 0, i32 1
  %6207 = load volatile i32, ptr %6206, align 8
  store i32 %6207, ptr %9, align 4
  br label %6208

6208:                                             ; preds = %6193, %6190
  %6209 = load i32, ptr %9, align 4
  %6210 = icmp ne i32 %6209, 0
  %6211 = zext i1 %6210 to i32
  %6212 = sext i32 %6211 to i64
  %6213 = icmp ne i64 %6212, 0
  br i1 %6213, label %6214, label %6222

6214:                                             ; preds = %6208
  %6215 = load ptr, ptr %3, align 8
  %6216 = load ptr, ptr %8, align 8
  %6217 = call i32 @luaG_traceexec(ptr noundef %6215, ptr noundef %6216)
  store i32 %6217, ptr %9, align 4
  %6218 = load ptr, ptr %4, align 8
  %6219 = getelementptr inbounds %struct.CallInfo, ptr %6218, i32 0, i32 0
  %6220 = load ptr, ptr %6219, align 8
  %6221 = getelementptr inbounds %union.StackValue, ptr %6220, i64 1
  store ptr %6221, ptr %7, align 8
  br label %6222

6222:                                             ; preds = %6214, %6208
  %6223 = load ptr, ptr %8, align 8
  %6224 = getelementptr inbounds i32, ptr %6223, i32 1
  store ptr %6224, ptr %8, align 8
  %6225 = load i32, ptr %6223, align 4
  store i32 %6225, ptr %10, align 4
  %6226 = load i32, ptr %10, align 4
  %6227 = lshr i32 %6226, 0
  %6228 = and i32 %6227, 127
  %6229 = zext i32 %6228 to i64
  %6230 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6229
  %6231 = load ptr, ptr %6230, align 8
  br label %7120

6232:                                             ; preds = %7120
  %6233 = load ptr, ptr %7, align 8
  %6234 = load i32, ptr %10, align 4
  %6235 = lshr i32 %6234, 7
  %6236 = and i32 %6235, 255
  %6237 = sext i32 %6236 to i64
  %6238 = getelementptr inbounds %union.StackValue, ptr %6233, i64 %6237
  store ptr %6238, ptr %359, align 8
  %6239 = load i32, ptr %10, align 4
  %6240 = lshr i32 %6239, 16
  %6241 = and i32 %6240, 255
  %6242 = sub nsw i32 %6241, 127
  store i32 %6242, ptr %361, align 4
  %6243 = load ptr, ptr %359, align 8
  %6244 = getelementptr inbounds %struct.TValue, ptr %6243, i32 0, i32 1
  %6245 = load i8, ptr %6244, align 8
  %6246 = zext i8 %6245 to i32
  %6247 = icmp eq i32 %6246, 3
  br i1 %6247, label %6248, label %6256

6248:                                             ; preds = %6232
  %6249 = load ptr, ptr %359, align 8
  %6250 = getelementptr inbounds %struct.TValue, ptr %6249, i32 0, i32 0
  %6251 = load i64, ptr %6250, align 8
  %6252 = load i32, ptr %361, align 4
  %6253 = sext i32 %6252 to i64
  %6254 = icmp sle i64 %6251, %6253
  %6255 = zext i1 %6254 to i32
  store i32 %6255, ptr %360, align 4
  br label %6295

6256:                                             ; preds = %6232
  %6257 = load ptr, ptr %359, align 8
  %6258 = getelementptr inbounds %struct.TValue, ptr %6257, i32 0, i32 1
  %6259 = load i8, ptr %6258, align 8
  %6260 = zext i8 %6259 to i32
  %6261 = icmp eq i32 %6260, 19
  br i1 %6261, label %6262, label %6272

6262:                                             ; preds = %6256
  %6263 = load ptr, ptr %359, align 8
  %6264 = getelementptr inbounds %struct.TValue, ptr %6263, i32 0, i32 0
  %6265 = load double, ptr %6264, align 8
  store double %6265, ptr %362, align 8
  %6266 = load i32, ptr %361, align 4
  %6267 = sitofp i32 %6266 to double
  store double %6267, ptr %363, align 8
  %6268 = load double, ptr %362, align 8
  %6269 = load double, ptr %363, align 8
  %6270 = fcmp ole double %6268, %6269
  %6271 = zext i1 %6270 to i32
  store i32 %6271, ptr %360, align 4
  br label %6294

6272:                                             ; preds = %6256
  %6273 = load i32, ptr %10, align 4
  %6274 = lshr i32 %6273, 24
  %6275 = and i32 %6274, 255
  store i32 %6275, ptr %364, align 4
  %6276 = load ptr, ptr %8, align 8
  %6277 = load ptr, ptr %4, align 8
  %6278 = getelementptr inbounds %struct.CallInfo, ptr %6277, i32 0, i32 4
  %6279 = getelementptr inbounds %struct.anon, ptr %6278, i32 0, i32 0
  store ptr %6276, ptr %6279, align 8
  %6280 = load ptr, ptr %4, align 8
  %6281 = getelementptr inbounds %struct.CallInfo, ptr %6280, i32 0, i32 1
  %6282 = load ptr, ptr %6281, align 8
  %6283 = load ptr, ptr %3, align 8
  %6284 = getelementptr inbounds %struct.lua_State, ptr %6283, i32 0, i32 6
  store ptr %6282, ptr %6284, align 8
  %6285 = load ptr, ptr %3, align 8
  %6286 = load ptr, ptr %359, align 8
  %6287 = load i32, ptr %361, align 4
  %6288 = load i32, ptr %364, align 4
  %6289 = call i32 @luaT_callorderiTM(ptr noundef %6285, ptr noundef %6286, i32 noundef %6287, i32 noundef 0, i32 noundef %6288, i32 noundef 21)
  store i32 %6289, ptr %360, align 4
  %6290 = load ptr, ptr %4, align 8
  %6291 = getelementptr inbounds %struct.CallInfo, ptr %6290, i32 0, i32 4
  %6292 = getelementptr inbounds %struct.anon, ptr %6291, i32 0, i32 1
  %6293 = load volatile i32, ptr %6292, align 8
  store i32 %6293, ptr %9, align 4
  br label %6294

6294:                                             ; preds = %6272, %6262
  br label %6295

6295:                                             ; preds = %6294, %6248
  %6296 = load i32, ptr %360, align 4
  %6297 = load i32, ptr %10, align 4
  %6298 = lshr i32 %6297, 15
  %6299 = and i32 %6298, 1
  %6300 = icmp ne i32 %6296, %6299
  br i1 %6300, label %6301, label %6304

6301:                                             ; preds = %6295
  %6302 = load ptr, ptr %8, align 8
  %6303 = getelementptr inbounds i32, ptr %6302, i32 1
  store ptr %6303, ptr %8, align 8
  br label %6319

6304:                                             ; preds = %6295
  %6305 = load ptr, ptr %8, align 8
  %6306 = load i32, ptr %6305, align 4
  store i32 %6306, ptr %365, align 4
  %6307 = load i32, ptr %365, align 4
  %6308 = lshr i32 %6307, 7
  %6309 = and i32 %6308, 33554431
  %6310 = sub nsw i32 %6309, 16777215
  %6311 = add nsw i32 %6310, 1
  %6312 = load ptr, ptr %8, align 8
  %6313 = sext i32 %6311 to i64
  %6314 = getelementptr inbounds i32, ptr %6312, i64 %6313
  store ptr %6314, ptr %8, align 8
  %6315 = load ptr, ptr %4, align 8
  %6316 = getelementptr inbounds %struct.CallInfo, ptr %6315, i32 0, i32 4
  %6317 = getelementptr inbounds %struct.anon, ptr %6316, i32 0, i32 1
  %6318 = load volatile i32, ptr %6317, align 8
  store i32 %6318, ptr %9, align 4
  br label %6319

6319:                                             ; preds = %6304, %6301
  %6320 = load i32, ptr %9, align 4
  %6321 = icmp ne i32 %6320, 0
  %6322 = zext i1 %6321 to i32
  %6323 = sext i32 %6322 to i64
  %6324 = icmp ne i64 %6323, 0
  br i1 %6324, label %6325, label %6333

6325:                                             ; preds = %6319
  %6326 = load ptr, ptr %3, align 8
  %6327 = load ptr, ptr %8, align 8
  %6328 = call i32 @luaG_traceexec(ptr noundef %6326, ptr noundef %6327)
  store i32 %6328, ptr %9, align 4
  %6329 = load ptr, ptr %4, align 8
  %6330 = getelementptr inbounds %struct.CallInfo, ptr %6329, i32 0, i32 0
  %6331 = load ptr, ptr %6330, align 8
  %6332 = getelementptr inbounds %union.StackValue, ptr %6331, i64 1
  store ptr %6332, ptr %7, align 8
  br label %6333

6333:                                             ; preds = %6325, %6319
  %6334 = load ptr, ptr %8, align 8
  %6335 = getelementptr inbounds i32, ptr %6334, i32 1
  store ptr %6335, ptr %8, align 8
  %6336 = load i32, ptr %6334, align 4
  store i32 %6336, ptr %10, align 4
  %6337 = load i32, ptr %10, align 4
  %6338 = lshr i32 %6337, 0
  %6339 = and i32 %6338, 127
  %6340 = zext i32 %6339 to i64
  %6341 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6340
  %6342 = load ptr, ptr %6341, align 8
  br label %7120

6343:                                             ; preds = %7120
  %6344 = load ptr, ptr %7, align 8
  %6345 = load i32, ptr %10, align 4
  %6346 = lshr i32 %6345, 7
  %6347 = and i32 %6346, 255
  %6348 = sext i32 %6347 to i64
  %6349 = getelementptr inbounds %union.StackValue, ptr %6344, i64 %6348
  store ptr %6349, ptr %366, align 8
  %6350 = load i32, ptr %10, align 4
  %6351 = lshr i32 %6350, 16
  %6352 = and i32 %6351, 255
  %6353 = sub nsw i32 %6352, 127
  store i32 %6353, ptr %368, align 4
  %6354 = load ptr, ptr %366, align 8
  %6355 = getelementptr inbounds %struct.TValue, ptr %6354, i32 0, i32 1
  %6356 = load i8, ptr %6355, align 8
  %6357 = zext i8 %6356 to i32
  %6358 = icmp eq i32 %6357, 3
  br i1 %6358, label %6359, label %6367

6359:                                             ; preds = %6343
  %6360 = load ptr, ptr %366, align 8
  %6361 = getelementptr inbounds %struct.TValue, ptr %6360, i32 0, i32 0
  %6362 = load i64, ptr %6361, align 8
  %6363 = load i32, ptr %368, align 4
  %6364 = sext i32 %6363 to i64
  %6365 = icmp sgt i64 %6362, %6364
  %6366 = zext i1 %6365 to i32
  store i32 %6366, ptr %367, align 4
  br label %6406

6367:                                             ; preds = %6343
  %6368 = load ptr, ptr %366, align 8
  %6369 = getelementptr inbounds %struct.TValue, ptr %6368, i32 0, i32 1
  %6370 = load i8, ptr %6369, align 8
  %6371 = zext i8 %6370 to i32
  %6372 = icmp eq i32 %6371, 19
  br i1 %6372, label %6373, label %6383

6373:                                             ; preds = %6367
  %6374 = load ptr, ptr %366, align 8
  %6375 = getelementptr inbounds %struct.TValue, ptr %6374, i32 0, i32 0
  %6376 = load double, ptr %6375, align 8
  store double %6376, ptr %369, align 8
  %6377 = load i32, ptr %368, align 4
  %6378 = sitofp i32 %6377 to double
  store double %6378, ptr %370, align 8
  %6379 = load double, ptr %369, align 8
  %6380 = load double, ptr %370, align 8
  %6381 = fcmp ogt double %6379, %6380
  %6382 = zext i1 %6381 to i32
  store i32 %6382, ptr %367, align 4
  br label %6405

6383:                                             ; preds = %6367
  %6384 = load i32, ptr %10, align 4
  %6385 = lshr i32 %6384, 24
  %6386 = and i32 %6385, 255
  store i32 %6386, ptr %371, align 4
  %6387 = load ptr, ptr %8, align 8
  %6388 = load ptr, ptr %4, align 8
  %6389 = getelementptr inbounds %struct.CallInfo, ptr %6388, i32 0, i32 4
  %6390 = getelementptr inbounds %struct.anon, ptr %6389, i32 0, i32 0
  store ptr %6387, ptr %6390, align 8
  %6391 = load ptr, ptr %4, align 8
  %6392 = getelementptr inbounds %struct.CallInfo, ptr %6391, i32 0, i32 1
  %6393 = load ptr, ptr %6392, align 8
  %6394 = load ptr, ptr %3, align 8
  %6395 = getelementptr inbounds %struct.lua_State, ptr %6394, i32 0, i32 6
  store ptr %6393, ptr %6395, align 8
  %6396 = load ptr, ptr %3, align 8
  %6397 = load ptr, ptr %366, align 8
  %6398 = load i32, ptr %368, align 4
  %6399 = load i32, ptr %371, align 4
  %6400 = call i32 @luaT_callorderiTM(ptr noundef %6396, ptr noundef %6397, i32 noundef %6398, i32 noundef 1, i32 noundef %6399, i32 noundef 20)
  store i32 %6400, ptr %367, align 4
  %6401 = load ptr, ptr %4, align 8
  %6402 = getelementptr inbounds %struct.CallInfo, ptr %6401, i32 0, i32 4
  %6403 = getelementptr inbounds %struct.anon, ptr %6402, i32 0, i32 1
  %6404 = load volatile i32, ptr %6403, align 8
  store i32 %6404, ptr %9, align 4
  br label %6405

6405:                                             ; preds = %6383, %6373
  br label %6406

6406:                                             ; preds = %6405, %6359
  %6407 = load i32, ptr %367, align 4
  %6408 = load i32, ptr %10, align 4
  %6409 = lshr i32 %6408, 15
  %6410 = and i32 %6409, 1
  %6411 = icmp ne i32 %6407, %6410
  br i1 %6411, label %6412, label %6415

6412:                                             ; preds = %6406
  %6413 = load ptr, ptr %8, align 8
  %6414 = getelementptr inbounds i32, ptr %6413, i32 1
  store ptr %6414, ptr %8, align 8
  br label %6430

6415:                                             ; preds = %6406
  %6416 = load ptr, ptr %8, align 8
  %6417 = load i32, ptr %6416, align 4
  store i32 %6417, ptr %372, align 4
  %6418 = load i32, ptr %372, align 4
  %6419 = lshr i32 %6418, 7
  %6420 = and i32 %6419, 33554431
  %6421 = sub nsw i32 %6420, 16777215
  %6422 = add nsw i32 %6421, 1
  %6423 = load ptr, ptr %8, align 8
  %6424 = sext i32 %6422 to i64
  %6425 = getelementptr inbounds i32, ptr %6423, i64 %6424
  store ptr %6425, ptr %8, align 8
  %6426 = load ptr, ptr %4, align 8
  %6427 = getelementptr inbounds %struct.CallInfo, ptr %6426, i32 0, i32 4
  %6428 = getelementptr inbounds %struct.anon, ptr %6427, i32 0, i32 1
  %6429 = load volatile i32, ptr %6428, align 8
  store i32 %6429, ptr %9, align 4
  br label %6430

6430:                                             ; preds = %6415, %6412
  %6431 = load i32, ptr %9, align 4
  %6432 = icmp ne i32 %6431, 0
  %6433 = zext i1 %6432 to i32
  %6434 = sext i32 %6433 to i64
  %6435 = icmp ne i64 %6434, 0
  br i1 %6435, label %6436, label %6444

6436:                                             ; preds = %6430
  %6437 = load ptr, ptr %3, align 8
  %6438 = load ptr, ptr %8, align 8
  %6439 = call i32 @luaG_traceexec(ptr noundef %6437, ptr noundef %6438)
  store i32 %6439, ptr %9, align 4
  %6440 = load ptr, ptr %4, align 8
  %6441 = getelementptr inbounds %struct.CallInfo, ptr %6440, i32 0, i32 0
  %6442 = load ptr, ptr %6441, align 8
  %6443 = getelementptr inbounds %union.StackValue, ptr %6442, i64 1
  store ptr %6443, ptr %7, align 8
  br label %6444

6444:                                             ; preds = %6436, %6430
  %6445 = load ptr, ptr %8, align 8
  %6446 = getelementptr inbounds i32, ptr %6445, i32 1
  store ptr %6446, ptr %8, align 8
  %6447 = load i32, ptr %6445, align 4
  store i32 %6447, ptr %10, align 4
  %6448 = load i32, ptr %10, align 4
  %6449 = lshr i32 %6448, 0
  %6450 = and i32 %6449, 127
  %6451 = zext i32 %6450 to i64
  %6452 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6451
  %6453 = load ptr, ptr %6452, align 8
  br label %7120

6454:                                             ; preds = %7120
  %6455 = load ptr, ptr %7, align 8
  %6456 = load i32, ptr %10, align 4
  %6457 = lshr i32 %6456, 7
  %6458 = and i32 %6457, 255
  %6459 = sext i32 %6458 to i64
  %6460 = getelementptr inbounds %union.StackValue, ptr %6455, i64 %6459
  store ptr %6460, ptr %373, align 8
  %6461 = load i32, ptr %10, align 4
  %6462 = lshr i32 %6461, 16
  %6463 = and i32 %6462, 255
  %6464 = sub nsw i32 %6463, 127
  store i32 %6464, ptr %375, align 4
  %6465 = load ptr, ptr %373, align 8
  %6466 = getelementptr inbounds %struct.TValue, ptr %6465, i32 0, i32 1
  %6467 = load i8, ptr %6466, align 8
  %6468 = zext i8 %6467 to i32
  %6469 = icmp eq i32 %6468, 3
  br i1 %6469, label %6470, label %6478

6470:                                             ; preds = %6454
  %6471 = load ptr, ptr %373, align 8
  %6472 = getelementptr inbounds %struct.TValue, ptr %6471, i32 0, i32 0
  %6473 = load i64, ptr %6472, align 8
  %6474 = load i32, ptr %375, align 4
  %6475 = sext i32 %6474 to i64
  %6476 = icmp sge i64 %6473, %6475
  %6477 = zext i1 %6476 to i32
  store i32 %6477, ptr %374, align 4
  br label %6517

6478:                                             ; preds = %6454
  %6479 = load ptr, ptr %373, align 8
  %6480 = getelementptr inbounds %struct.TValue, ptr %6479, i32 0, i32 1
  %6481 = load i8, ptr %6480, align 8
  %6482 = zext i8 %6481 to i32
  %6483 = icmp eq i32 %6482, 19
  br i1 %6483, label %6484, label %6494

6484:                                             ; preds = %6478
  %6485 = load ptr, ptr %373, align 8
  %6486 = getelementptr inbounds %struct.TValue, ptr %6485, i32 0, i32 0
  %6487 = load double, ptr %6486, align 8
  store double %6487, ptr %376, align 8
  %6488 = load i32, ptr %375, align 4
  %6489 = sitofp i32 %6488 to double
  store double %6489, ptr %377, align 8
  %6490 = load double, ptr %376, align 8
  %6491 = load double, ptr %377, align 8
  %6492 = fcmp oge double %6490, %6491
  %6493 = zext i1 %6492 to i32
  store i32 %6493, ptr %374, align 4
  br label %6516

6494:                                             ; preds = %6478
  %6495 = load i32, ptr %10, align 4
  %6496 = lshr i32 %6495, 24
  %6497 = and i32 %6496, 255
  store i32 %6497, ptr %378, align 4
  %6498 = load ptr, ptr %8, align 8
  %6499 = load ptr, ptr %4, align 8
  %6500 = getelementptr inbounds %struct.CallInfo, ptr %6499, i32 0, i32 4
  %6501 = getelementptr inbounds %struct.anon, ptr %6500, i32 0, i32 0
  store ptr %6498, ptr %6501, align 8
  %6502 = load ptr, ptr %4, align 8
  %6503 = getelementptr inbounds %struct.CallInfo, ptr %6502, i32 0, i32 1
  %6504 = load ptr, ptr %6503, align 8
  %6505 = load ptr, ptr %3, align 8
  %6506 = getelementptr inbounds %struct.lua_State, ptr %6505, i32 0, i32 6
  store ptr %6504, ptr %6506, align 8
  %6507 = load ptr, ptr %3, align 8
  %6508 = load ptr, ptr %373, align 8
  %6509 = load i32, ptr %375, align 4
  %6510 = load i32, ptr %378, align 4
  %6511 = call i32 @luaT_callorderiTM(ptr noundef %6507, ptr noundef %6508, i32 noundef %6509, i32 noundef 1, i32 noundef %6510, i32 noundef 21)
  store i32 %6511, ptr %374, align 4
  %6512 = load ptr, ptr %4, align 8
  %6513 = getelementptr inbounds %struct.CallInfo, ptr %6512, i32 0, i32 4
  %6514 = getelementptr inbounds %struct.anon, ptr %6513, i32 0, i32 1
  %6515 = load volatile i32, ptr %6514, align 8
  store i32 %6515, ptr %9, align 4
  br label %6516

6516:                                             ; preds = %6494, %6484
  br label %6517

6517:                                             ; preds = %6516, %6470
  %6518 = load i32, ptr %374, align 4
  %6519 = load i32, ptr %10, align 4
  %6520 = lshr i32 %6519, 15
  %6521 = and i32 %6520, 1
  %6522 = icmp ne i32 %6518, %6521
  br i1 %6522, label %6523, label %6526

6523:                                             ; preds = %6517
  %6524 = load ptr, ptr %8, align 8
  %6525 = getelementptr inbounds i32, ptr %6524, i32 1
  store ptr %6525, ptr %8, align 8
  br label %6541

6526:                                             ; preds = %6517
  %6527 = load ptr, ptr %8, align 8
  %6528 = load i32, ptr %6527, align 4
  store i32 %6528, ptr %379, align 4
  %6529 = load i32, ptr %379, align 4
  %6530 = lshr i32 %6529, 7
  %6531 = and i32 %6530, 33554431
  %6532 = sub nsw i32 %6531, 16777215
  %6533 = add nsw i32 %6532, 1
  %6534 = load ptr, ptr %8, align 8
  %6535 = sext i32 %6533 to i64
  %6536 = getelementptr inbounds i32, ptr %6534, i64 %6535
  store ptr %6536, ptr %8, align 8
  %6537 = load ptr, ptr %4, align 8
  %6538 = getelementptr inbounds %struct.CallInfo, ptr %6537, i32 0, i32 4
  %6539 = getelementptr inbounds %struct.anon, ptr %6538, i32 0, i32 1
  %6540 = load volatile i32, ptr %6539, align 8
  store i32 %6540, ptr %9, align 4
  br label %6541

6541:                                             ; preds = %6526, %6523
  %6542 = load i32, ptr %9, align 4
  %6543 = icmp ne i32 %6542, 0
  %6544 = zext i1 %6543 to i32
  %6545 = sext i32 %6544 to i64
  %6546 = icmp ne i64 %6545, 0
  br i1 %6546, label %6547, label %6555

6547:                                             ; preds = %6541
  %6548 = load ptr, ptr %3, align 8
  %6549 = load ptr, ptr %8, align 8
  %6550 = call i32 @luaG_traceexec(ptr noundef %6548, ptr noundef %6549)
  store i32 %6550, ptr %9, align 4
  %6551 = load ptr, ptr %4, align 8
  %6552 = getelementptr inbounds %struct.CallInfo, ptr %6551, i32 0, i32 0
  %6553 = load ptr, ptr %6552, align 8
  %6554 = getelementptr inbounds %union.StackValue, ptr %6553, i64 1
  store ptr %6554, ptr %7, align 8
  br label %6555

6555:                                             ; preds = %6547, %6541
  %6556 = load ptr, ptr %8, align 8
  %6557 = getelementptr inbounds i32, ptr %6556, i32 1
  store ptr %6557, ptr %8, align 8
  %6558 = load i32, ptr %6556, align 4
  store i32 %6558, ptr %10, align 4
  %6559 = load i32, ptr %10, align 4
  %6560 = lshr i32 %6559, 0
  %6561 = and i32 %6560, 127
  %6562 = zext i32 %6561 to i64
  %6563 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6562
  %6564 = load ptr, ptr %6563, align 8
  br label %7120

6565:                                             ; preds = %7120
  %6566 = load ptr, ptr %7, align 8
  %6567 = load i32, ptr %10, align 4
  %6568 = lshr i32 %6567, 7
  %6569 = and i32 %6568, 255
  %6570 = sext i32 %6569 to i64
  %6571 = getelementptr inbounds %union.StackValue, ptr %6566, i64 %6570
  store ptr %6571, ptr %380, align 8
  %6572 = load ptr, ptr %380, align 8
  %6573 = getelementptr inbounds %struct.TValue, ptr %6572, i32 0, i32 1
  %6574 = load i8, ptr %6573, align 8
  %6575 = zext i8 %6574 to i32
  %6576 = icmp eq i32 %6575, 1
  br i1 %6576, label %6584, label %6577

6577:                                             ; preds = %6565
  %6578 = load ptr, ptr %380, align 8
  %6579 = getelementptr inbounds %struct.TValue, ptr %6578, i32 0, i32 1
  %6580 = load i8, ptr %6579, align 8
  %6581 = zext i8 %6580 to i32
  %6582 = and i32 %6581, 15
  %6583 = icmp eq i32 %6582, 0
  br label %6584

6584:                                             ; preds = %6577, %6565
  %6585 = phi i1 [ true, %6565 ], [ %6583, %6577 ]
  %6586 = xor i1 %6585, true
  %6587 = zext i1 %6586 to i32
  store i32 %6587, ptr %381, align 4
  %6588 = load i32, ptr %381, align 4
  %6589 = load i32, ptr %10, align 4
  %6590 = lshr i32 %6589, 15
  %6591 = and i32 %6590, 1
  %6592 = icmp ne i32 %6588, %6591
  br i1 %6592, label %6593, label %6596

6593:                                             ; preds = %6584
  %6594 = load ptr, ptr %8, align 8
  %6595 = getelementptr inbounds i32, ptr %6594, i32 1
  store ptr %6595, ptr %8, align 8
  br label %6611

6596:                                             ; preds = %6584
  %6597 = load ptr, ptr %8, align 8
  %6598 = load i32, ptr %6597, align 4
  store i32 %6598, ptr %382, align 4
  %6599 = load i32, ptr %382, align 4
  %6600 = lshr i32 %6599, 7
  %6601 = and i32 %6600, 33554431
  %6602 = sub nsw i32 %6601, 16777215
  %6603 = add nsw i32 %6602, 1
  %6604 = load ptr, ptr %8, align 8
  %6605 = sext i32 %6603 to i64
  %6606 = getelementptr inbounds i32, ptr %6604, i64 %6605
  store ptr %6606, ptr %8, align 8
  %6607 = load ptr, ptr %4, align 8
  %6608 = getelementptr inbounds %struct.CallInfo, ptr %6607, i32 0, i32 4
  %6609 = getelementptr inbounds %struct.anon, ptr %6608, i32 0, i32 1
  %6610 = load volatile i32, ptr %6609, align 8
  store i32 %6610, ptr %9, align 4
  br label %6611

6611:                                             ; preds = %6596, %6593
  %6612 = load i32, ptr %9, align 4
  %6613 = icmp ne i32 %6612, 0
  %6614 = zext i1 %6613 to i32
  %6615 = sext i32 %6614 to i64
  %6616 = icmp ne i64 %6615, 0
  br i1 %6616, label %6617, label %6625

6617:                                             ; preds = %6611
  %6618 = load ptr, ptr %3, align 8
  %6619 = load ptr, ptr %8, align 8
  %6620 = call i32 @luaG_traceexec(ptr noundef %6618, ptr noundef %6619)
  store i32 %6620, ptr %9, align 4
  %6621 = load ptr, ptr %4, align 8
  %6622 = getelementptr inbounds %struct.CallInfo, ptr %6621, i32 0, i32 0
  %6623 = load ptr, ptr %6622, align 8
  %6624 = getelementptr inbounds %union.StackValue, ptr %6623, i64 1
  store ptr %6624, ptr %7, align 8
  br label %6625

6625:                                             ; preds = %6617, %6611
  %6626 = load ptr, ptr %8, align 8
  %6627 = getelementptr inbounds i32, ptr %6626, i32 1
  store ptr %6627, ptr %8, align 8
  %6628 = load i32, ptr %6626, align 4
  store i32 %6628, ptr %10, align 4
  %6629 = load i32, ptr %10, align 4
  %6630 = lshr i32 %6629, 0
  %6631 = and i32 %6630, 127
  %6632 = zext i32 %6631 to i64
  %6633 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6632
  %6634 = load ptr, ptr %6633, align 8
  br label %7120

6635:                                             ; preds = %7120
  %6636 = load ptr, ptr %7, align 8
  %6637 = load i32, ptr %10, align 4
  %6638 = lshr i32 %6637, 7
  %6639 = and i32 %6638, 255
  %6640 = sext i32 %6639 to i64
  %6641 = getelementptr inbounds %union.StackValue, ptr %6636, i64 %6640
  store ptr %6641, ptr %383, align 8
  %6642 = load ptr, ptr %7, align 8
  %6643 = load i32, ptr %10, align 4
  %6644 = lshr i32 %6643, 16
  %6645 = and i32 %6644, 255
  %6646 = sext i32 %6645 to i64
  %6647 = getelementptr inbounds %union.StackValue, ptr %6642, i64 %6646
  store ptr %6647, ptr %384, align 8
  %6648 = load ptr, ptr %384, align 8
  %6649 = getelementptr inbounds %struct.TValue, ptr %6648, i32 0, i32 1
  %6650 = load i8, ptr %6649, align 8
  %6651 = zext i8 %6650 to i32
  %6652 = icmp eq i32 %6651, 1
  br i1 %6652, label %6660, label %6653

6653:                                             ; preds = %6635
  %6654 = load ptr, ptr %384, align 8
  %6655 = getelementptr inbounds %struct.TValue, ptr %6654, i32 0, i32 1
  %6656 = load i8, ptr %6655, align 8
  %6657 = zext i8 %6656 to i32
  %6658 = and i32 %6657, 15
  %6659 = icmp eq i32 %6658, 0
  br label %6660

6660:                                             ; preds = %6653, %6635
  %6661 = phi i1 [ true, %6635 ], [ %6659, %6653 ]
  %6662 = zext i1 %6661 to i32
  %6663 = load i32, ptr %10, align 4
  %6664 = lshr i32 %6663, 15
  %6665 = and i32 %6664, 1
  %6666 = icmp eq i32 %6662, %6665
  br i1 %6666, label %6667, label %6670

6667:                                             ; preds = %6660
  %6668 = load ptr, ptr %8, align 8
  %6669 = getelementptr inbounds i32, ptr %6668, i32 1
  store ptr %6669, ptr %8, align 8
  br label %6697

6670:                                             ; preds = %6660
  %6671 = load ptr, ptr %383, align 8
  store ptr %6671, ptr %385, align 8
  %6672 = load ptr, ptr %384, align 8
  store ptr %6672, ptr %386, align 8
  %6673 = load ptr, ptr %385, align 8
  %6674 = getelementptr inbounds %struct.TValue, ptr %6673, i32 0, i32 0
  %6675 = load ptr, ptr %386, align 8
  %6676 = getelementptr inbounds %struct.TValue, ptr %6675, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %6674, ptr align 8 %6676, i64 8, i1 false)
  %6677 = load ptr, ptr %386, align 8
  %6678 = getelementptr inbounds %struct.TValue, ptr %6677, i32 0, i32 1
  %6679 = load i8, ptr %6678, align 8
  %6680 = load ptr, ptr %385, align 8
  %6681 = getelementptr inbounds %struct.TValue, ptr %6680, i32 0, i32 1
  store i8 %6679, ptr %6681, align 8
  %6682 = load ptr, ptr %3, align 8
  %6683 = load ptr, ptr %8, align 8
  %6684 = load i32, ptr %6683, align 4
  store i32 %6684, ptr %387, align 4
  %6685 = load i32, ptr %387, align 4
  %6686 = lshr i32 %6685, 7
  %6687 = and i32 %6686, 33554431
  %6688 = sub nsw i32 %6687, 16777215
  %6689 = add nsw i32 %6688, 1
  %6690 = load ptr, ptr %8, align 8
  %6691 = sext i32 %6689 to i64
  %6692 = getelementptr inbounds i32, ptr %6690, i64 %6691
  store ptr %6692, ptr %8, align 8
  %6693 = load ptr, ptr %4, align 8
  %6694 = getelementptr inbounds %struct.CallInfo, ptr %6693, i32 0, i32 4
  %6695 = getelementptr inbounds %struct.anon, ptr %6694, i32 0, i32 1
  %6696 = load volatile i32, ptr %6695, align 8
  store i32 %6696, ptr %9, align 4
  br label %6697

6697:                                             ; preds = %6670, %6667
  %6698 = load i32, ptr %9, align 4
  %6699 = icmp ne i32 %6698, 0
  %6700 = zext i1 %6699 to i32
  %6701 = sext i32 %6700 to i64
  %6702 = icmp ne i64 %6701, 0
  br i1 %6702, label %6703, label %6711

6703:                                             ; preds = %6697
  %6704 = load ptr, ptr %3, align 8
  %6705 = load ptr, ptr %8, align 8
  %6706 = call i32 @luaG_traceexec(ptr noundef %6704, ptr noundef %6705)
  store i32 %6706, ptr %9, align 4
  %6707 = load ptr, ptr %4, align 8
  %6708 = getelementptr inbounds %struct.CallInfo, ptr %6707, i32 0, i32 0
  %6709 = load ptr, ptr %6708, align 8
  %6710 = getelementptr inbounds %union.StackValue, ptr %6709, i64 1
  store ptr %6710, ptr %7, align 8
  br label %6711

6711:                                             ; preds = %6703, %6697
  %6712 = load ptr, ptr %8, align 8
  %6713 = getelementptr inbounds i32, ptr %6712, i32 1
  store ptr %6713, ptr %8, align 8
  %6714 = load i32, ptr %6712, align 4
  store i32 %6714, ptr %10, align 4
  %6715 = load i32, ptr %10, align 4
  %6716 = lshr i32 %6715, 0
  %6717 = and i32 %6716, 127
  %6718 = zext i32 %6717 to i64
  %6719 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6718
  %6720 = load ptr, ptr %6719, align 8
  br label %7120

6721:                                             ; preds = %7120
  %6722 = load ptr, ptr %7, align 8
  %6723 = load i32, ptr %10, align 4
  %6724 = lshr i32 %6723, 7
  %6725 = and i32 %6724, 255
  %6726 = sext i32 %6725 to i64
  %6727 = getelementptr inbounds %union.StackValue, ptr %6722, i64 %6726
  store ptr %6727, ptr %388, align 8
  %6728 = load i32, ptr %10, align 4
  %6729 = lshr i32 %6728, 16
  %6730 = and i32 %6729, 255
  store i32 %6730, ptr %390, align 4
  %6731 = load i32, ptr %10, align 4
  %6732 = lshr i32 %6731, 24
  %6733 = and i32 %6732, 255
  %6734 = sub nsw i32 %6733, 1
  store i32 %6734, ptr %391, align 4
  %6735 = load i32, ptr %390, align 4
  %6736 = icmp ne i32 %6735, 0
  br i1 %6736, label %6737, label %6744

6737:                                             ; preds = %6721
  %6738 = load ptr, ptr %388, align 8
  %6739 = load i32, ptr %390, align 4
  %6740 = sext i32 %6739 to i64
  %6741 = getelementptr inbounds %union.StackValue, ptr %6738, i64 %6740
  %6742 = load ptr, ptr %3, align 8
  %6743 = getelementptr inbounds %struct.lua_State, ptr %6742, i32 0, i32 6
  store ptr %6741, ptr %6743, align 8
  br label %6744

6744:                                             ; preds = %6737, %6721
  %6745 = load ptr, ptr %8, align 8
  %6746 = load ptr, ptr %4, align 8
  %6747 = getelementptr inbounds %struct.CallInfo, ptr %6746, i32 0, i32 4
  %6748 = getelementptr inbounds %struct.anon, ptr %6747, i32 0, i32 0
  store ptr %6745, ptr %6748, align 8
  %6749 = load ptr, ptr %3, align 8
  %6750 = load ptr, ptr %388, align 8
  %6751 = load i32, ptr %391, align 4
  %6752 = call ptr @luaD_precall(ptr noundef %6749, ptr noundef %6750, i32 noundef %6751)
  store ptr %6752, ptr %389, align 8
  %6753 = icmp eq ptr %6752, null
  br i1 %6753, label %6754, label %6759

6754:                                             ; preds = %6744
  %6755 = load ptr, ptr %4, align 8
  %6756 = getelementptr inbounds %struct.CallInfo, ptr %6755, i32 0, i32 4
  %6757 = getelementptr inbounds %struct.anon, ptr %6756, i32 0, i32 1
  %6758 = load volatile i32, ptr %6757, align 8
  store i32 %6758, ptr %9, align 4
  br label %6761

6759:                                             ; preds = %6744
  %6760 = load ptr, ptr %389, align 8
  store ptr %6760, ptr %4, align 8
  br label %431

6761:                                             ; preds = %6754
  %6762 = load i32, ptr %9, align 4
  %6763 = icmp ne i32 %6762, 0
  %6764 = zext i1 %6763 to i32
  %6765 = sext i32 %6764 to i64
  %6766 = icmp ne i64 %6765, 0
  br i1 %6766, label %6767, label %6775

6767:                                             ; preds = %6761
  %6768 = load ptr, ptr %3, align 8
  %6769 = load ptr, ptr %8, align 8
  %6770 = call i32 @luaG_traceexec(ptr noundef %6768, ptr noundef %6769)
  store i32 %6770, ptr %9, align 4
  %6771 = load ptr, ptr %4, align 8
  %6772 = getelementptr inbounds %struct.CallInfo, ptr %6771, i32 0, i32 0
  %6773 = load ptr, ptr %6772, align 8
  %6774 = getelementptr inbounds %union.StackValue, ptr %6773, i64 1
  store ptr %6774, ptr %7, align 8
  br label %6775

6775:                                             ; preds = %6767, %6761
  %6776 = load ptr, ptr %8, align 8
  %6777 = getelementptr inbounds i32, ptr %6776, i32 1
  store ptr %6777, ptr %8, align 8
  %6778 = load i32, ptr %6776, align 4
  store i32 %6778, ptr %10, align 4
  %6779 = load i32, ptr %10, align 4
  %6780 = lshr i32 %6779, 0
  %6781 = and i32 %6780, 127
  %6782 = zext i32 %6781 to i64
  %6783 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %6782
  %6784 = load ptr, ptr %6783, align 8
  br label %7120

6785:                                             ; preds = %7120
  %6786 = load ptr, ptr %7, align 8
  %6787 = load i32, ptr %10, align 4
  %6788 = lshr i32 %6787, 7
  %6789 = and i32 %6788, 255
  %6790 = sext i32 %6789 to i64
  %6791 = getelementptr inbounds %union.StackValue, ptr %6786, i64 %6790
  store ptr %6791, ptr %392, align 8
  %6792 = load i32, ptr %10, align 4
  %6793 = lshr i32 %6792, 16
  %6794 = and i32 %6793, 255
  store i32 %6794, ptr %393, align 4
  %6795 = load i32, ptr %10, align 4
  %6796 = lshr i32 %6795, 24
  %6797 = and i32 %6796, 255
  store i32 %6797, ptr %395, align 4
  %6798 = load i32, ptr %395, align 4
  %6799 = icmp ne i32 %6798, 0
  br i1 %6799, label %6800, label %6807

6800:                                             ; preds = %6785
  %6801 = load ptr, ptr %4, align 8
  %6802 = getelementptr inbounds %struct.CallInfo, ptr %6801, i32 0, i32 4
  %6803 = getelementptr inbounds %struct.anon, ptr %6802, i32 0, i32 2
  %6804 = load i32, ptr %6803, align 4
  %6805 = load i32, ptr %395, align 4
  %6806 = add nsw i32 %6804, %6805
  br label %6808

6807:                                             ; preds = %6785
  br label %6808

6808:                                             ; preds = %6807, %6800
  %6809 = phi i32 [ %6806, %6800 ], [ 0, %6807 ]
  store i32 %6809, ptr %396, align 4
  %6810 = load i32, ptr %393, align 4
  %6811 = icmp ne i32 %6810, 0
  br i1 %6811, label %6812, label %6819

6812:                                             ; preds = %6808
  %6813 = load ptr, ptr %392, align 8
  %6814 = load i32, ptr %393, align 4
  %6815 = sext i32 %6814 to i64
  %6816 = getelementptr inbounds %union.StackValue, ptr %6813, i64 %6815
  %6817 = load ptr, ptr %3, align 8
  %6818 = getelementptr inbounds %struct.lua_State, ptr %6817, i32 0, i32 6
  store ptr %6816, ptr %6818, align 8
  br label %6829

6819:                                             ; preds = %6808
  %6820 = load ptr, ptr %3, align 8
  %6821 = getelementptr inbounds %struct.lua_State, ptr %6820, i32 0, i32 6
  %6822 = load ptr, ptr %6821, align 8
  %6823 = load ptr, ptr %392, align 8
  %6824 = ptrtoint ptr %6822 to i64
  %6825 = ptrtoint ptr %6823 to i64
  %6826 = sub i64 %6824, %6825
  %6827 = sdiv exact i64 %6826, 16
  %6828 = trunc i64 %6827 to i32
  store i32 %6828, ptr %393, align 4
  br label %6829

6829:                                             ; preds = %6819, %6812
  %6830 = load ptr, ptr %8, align 8
  %6831 = load ptr, ptr %4, align 8
  %6832 = getelementptr inbounds %struct.CallInfo, ptr %6831, i32 0, i32 4
  %6833 = getelementptr inbounds %struct.anon, ptr %6832, i32 0, i32 0
  store ptr %6830, ptr %6833, align 8
  %6834 = load i32, ptr %10, align 4
  %6835 = and i32 %6834, 32768
  %6836 = icmp ne i32 %6835, 0
  br i1 %6836, label %6837, label %6840

6837:                                             ; preds = %6829
  %6838 = load ptr, ptr %3, align 8
  %6839 = load ptr, ptr %7, align 8
  call void @luaF_closeupval(ptr noundef %6838, ptr noundef %6839)
  br label %6840

6840:                                             ; preds = %6837, %6829
  %6841 = load ptr, ptr %3, align 8
  %6842 = load ptr, ptr %4, align 8
  %6843 = load ptr, ptr %392, align 8
  %6844 = load i32, ptr %393, align 4
  %6845 = load i32, ptr %396, align 4
  %6846 = call i32 @luaD_pretailcall(ptr noundef %6841, ptr noundef %6842, ptr noundef %6843, i32 noundef %6844, i32 noundef %6845)
  store i32 %6846, ptr %394, align 4
  %6847 = icmp slt i32 %6846, 0
  br i1 %6847, label %6848, label %6849

6848:                                             ; preds = %6840
  br label %431

6849:                                             ; preds = %6840
  %6850 = load i32, ptr %396, align 4
  %6851 = load ptr, ptr %4, align 8
  %6852 = getelementptr inbounds %struct.CallInfo, ptr %6851, i32 0, i32 0
  %6853 = load ptr, ptr %6852, align 8
  %6854 = sext i32 %6850 to i64
  %6855 = sub i64 0, %6854
  %6856 = getelementptr inbounds %union.StackValue, ptr %6853, i64 %6855
  store ptr %6856, ptr %6852, align 8
  %6857 = load ptr, ptr %3, align 8
  %6858 = load ptr, ptr %4, align 8
  %6859 = load i32, ptr %394, align 4
  call void @luaD_poscall(ptr noundef %6857, ptr noundef %6858, i32 noundef %6859)
  %6860 = load ptr, ptr %4, align 8
  %6861 = getelementptr inbounds %struct.CallInfo, ptr %6860, i32 0, i32 4
  %6862 = getelementptr inbounds %struct.anon, ptr %6861, i32 0, i32 1
  %6863 = load volatile i32, ptr %6862, align 8
  store i32 %6863, ptr %9, align 4
  br label %7112

6864:                                             ; preds = %7120
  %6865 = load ptr, ptr %7, align 8
  %6866 = load i32, ptr %10, align 4
  %6867 = lshr i32 %6866, 7
  %6868 = and i32 %6867, 255
  %6869 = sext i32 %6868 to i64
  %6870 = getelementptr inbounds %union.StackValue, ptr %6865, i64 %6869
  store ptr %6870, ptr %397, align 8
  %6871 = load i32, ptr %10, align 4
  %6872 = lshr i32 %6871, 16
  %6873 = and i32 %6872, 255
  %6874 = sub nsw i32 %6873, 1
  store i32 %6874, ptr %398, align 4
  %6875 = load i32, ptr %10, align 4
  %6876 = lshr i32 %6875, 24
  %6877 = and i32 %6876, 255
  store i32 %6877, ptr %399, align 4
  %6878 = load i32, ptr %398, align 4
  %6879 = icmp slt i32 %6878, 0
  br i1 %6879, label %6880, label %6890

6880:                                             ; preds = %6864
  %6881 = load ptr, ptr %3, align 8
  %6882 = getelementptr inbounds %struct.lua_State, ptr %6881, i32 0, i32 6
  %6883 = load ptr, ptr %6882, align 8
  %6884 = load ptr, ptr %397, align 8
  %6885 = ptrtoint ptr %6883 to i64
  %6886 = ptrtoint ptr %6884 to i64
  %6887 = sub i64 %6885, %6886
  %6888 = sdiv exact i64 %6887, 16
  %6889 = trunc i64 %6888 to i32
  store i32 %6889, ptr %398, align 4
  br label %6890

6890:                                             ; preds = %6880, %6864
  %6891 = load ptr, ptr %8, align 8
  %6892 = load ptr, ptr %4, align 8
  %6893 = getelementptr inbounds %struct.CallInfo, ptr %6892, i32 0, i32 4
  %6894 = getelementptr inbounds %struct.anon, ptr %6893, i32 0, i32 0
  store ptr %6891, ptr %6894, align 8
  %6895 = load i32, ptr %10, align 4
  %6896 = and i32 %6895, 32768
  %6897 = icmp ne i32 %6896, 0
  br i1 %6897, label %6898, label %6940

6898:                                             ; preds = %6890
  %6899 = load i32, ptr %398, align 4
  %6900 = load ptr, ptr %4, align 8
  %6901 = getelementptr inbounds %struct.CallInfo, ptr %6900, i32 0, i32 5
  store i32 %6899, ptr %6901, align 8
  %6902 = load ptr, ptr %3, align 8
  %6903 = getelementptr inbounds %struct.lua_State, ptr %6902, i32 0, i32 6
  %6904 = load ptr, ptr %6903, align 8
  %6905 = load ptr, ptr %4, align 8
  %6906 = getelementptr inbounds %struct.CallInfo, ptr %6905, i32 0, i32 1
  %6907 = load ptr, ptr %6906, align 8
  %6908 = icmp ult ptr %6904, %6907
  br i1 %6908, label %6909, label %6915

6909:                                             ; preds = %6898
  %6910 = load ptr, ptr %4, align 8
  %6911 = getelementptr inbounds %struct.CallInfo, ptr %6910, i32 0, i32 1
  %6912 = load ptr, ptr %6911, align 8
  %6913 = load ptr, ptr %3, align 8
  %6914 = getelementptr inbounds %struct.lua_State, ptr %6913, i32 0, i32 6
  store ptr %6912, ptr %6914, align 8
  br label %6915

6915:                                             ; preds = %6909, %6898
  %6916 = load ptr, ptr %3, align 8
  %6917 = load ptr, ptr %7, align 8
  %6918 = call ptr @luaF_close(ptr noundef %6916, ptr noundef %6917, i32 noundef -1, i32 noundef 1)
  %6919 = load ptr, ptr %4, align 8
  %6920 = getelementptr inbounds %struct.CallInfo, ptr %6919, i32 0, i32 4
  %6921 = getelementptr inbounds %struct.anon, ptr %6920, i32 0, i32 1
  %6922 = load volatile i32, ptr %6921, align 8
  store i32 %6922, ptr %9, align 4
  %6923 = load i32, ptr %9, align 4
  %6924 = icmp ne i32 %6923, 0
  %6925 = zext i1 %6924 to i32
  %6926 = sext i32 %6925 to i64
  %6927 = icmp ne i64 %6926, 0
  br i1 %6927, label %6928, label %6939

6928:                                             ; preds = %6915
  %6929 = load ptr, ptr %4, align 8
  %6930 = getelementptr inbounds %struct.CallInfo, ptr %6929, i32 0, i32 0
  %6931 = load ptr, ptr %6930, align 8
  %6932 = getelementptr inbounds %union.StackValue, ptr %6931, i64 1
  store ptr %6932, ptr %7, align 8
  %6933 = load ptr, ptr %7, align 8
  %6934 = load i32, ptr %10, align 4
  %6935 = lshr i32 %6934, 7
  %6936 = and i32 %6935, 255
  %6937 = sext i32 %6936 to i64
  %6938 = getelementptr inbounds %union.StackValue, ptr %6933, i64 %6937
  store ptr %6938, ptr %397, align 8
  br label %6939

6939:                                             ; preds = %6928, %6915
  br label %6940

6940:                                             ; preds = %6939, %6890
  %6941 = load i32, ptr %399, align 4
  %6942 = icmp ne i32 %6941, 0
  br i1 %6942, label %6943, label %6956

6943:                                             ; preds = %6940
  %6944 = load ptr, ptr %4, align 8
  %6945 = getelementptr inbounds %struct.CallInfo, ptr %6944, i32 0, i32 4
  %6946 = getelementptr inbounds %struct.anon, ptr %6945, i32 0, i32 2
  %6947 = load i32, ptr %6946, align 4
  %6948 = load i32, ptr %399, align 4
  %6949 = add nsw i32 %6947, %6948
  %6950 = load ptr, ptr %4, align 8
  %6951 = getelementptr inbounds %struct.CallInfo, ptr %6950, i32 0, i32 0
  %6952 = load ptr, ptr %6951, align 8
  %6953 = sext i32 %6949 to i64
  %6954 = sub i64 0, %6953
  %6955 = getelementptr inbounds %union.StackValue, ptr %6952, i64 %6954
  store ptr %6955, ptr %6951, align 8
  br label %6956

6956:                                             ; preds = %6943, %6940
  %6957 = load ptr, ptr %397, align 8
  %6958 = load i32, ptr %398, align 4
  %6959 = sext i32 %6958 to i64
  %6960 = getelementptr inbounds %union.StackValue, ptr %6957, i64 %6959
  %6961 = load ptr, ptr %3, align 8
  %6962 = getelementptr inbounds %struct.lua_State, ptr %6961, i32 0, i32 6
  store ptr %6960, ptr %6962, align 8
  %6963 = load ptr, ptr %3, align 8
  %6964 = load ptr, ptr %4, align 8
  %6965 = load i32, ptr %398, align 4
  call void @luaD_poscall(ptr noundef %6963, ptr noundef %6964, i32 noundef %6965)
  %6966 = load ptr, ptr %4, align 8
  %6967 = getelementptr inbounds %struct.CallInfo, ptr %6966, i32 0, i32 4
  %6968 = getelementptr inbounds %struct.anon, ptr %6967, i32 0, i32 1
  %6969 = load volatile i32, ptr %6968, align 8
  store i32 %6969, ptr %9, align 4
  br label %7112

6970:                                             ; preds = %7120
  %6971 = load ptr, ptr %3, align 8
  %6972 = getelementptr inbounds %struct.lua_State, ptr %6971, i32 0, i32 23
  %6973 = load volatile i32, ptr %6972, align 8
  %6974 = icmp ne i32 %6973, 0
  %6975 = zext i1 %6974 to i32
  %6976 = sext i32 %6975 to i64
  %6977 = icmp ne i64 %6976, 0
  br i1 %6977, label %6978, label %6994

6978:                                             ; preds = %6970
  %6979 = load ptr, ptr %7, align 8
  %6980 = load i32, ptr %10, align 4
  %6981 = lshr i32 %6980, 7
  %6982 = and i32 %6981, 255
  %6983 = sext i32 %6982 to i64
  %6984 = getelementptr inbounds %union.StackValue, ptr %6979, i64 %6983
  store ptr %6984, ptr %400, align 8
  %6985 = load ptr, ptr %400, align 8
  %6986 = load ptr, ptr %3, align 8
  %6987 = getelementptr inbounds %struct.lua_State, ptr %6986, i32 0, i32 6
  store ptr %6985, ptr %6987, align 8
  %6988 = load ptr, ptr %8, align 8
  %6989 = load ptr, ptr %4, align 8
  %6990 = getelementptr inbounds %struct.CallInfo, ptr %6989, i32 0, i32 4
  %6991 = getelementptr inbounds %struct.anon, ptr %6990, i32 0, i32 0
  store ptr %6988, ptr %6991, align 8
  %6992 = load ptr, ptr %3, align 8
  %6993 = load ptr, ptr %4, align 8
  call void @luaD_poscall(ptr noundef %6992, ptr noundef %6993, i32 noundef 0)
  store i32 1, ptr %9, align 4
  br label %7026

6994:                                             ; preds = %6970
  %6995 = load ptr, ptr %4, align 8
  %6996 = getelementptr inbounds %struct.CallInfo, ptr %6995, i32 0, i32 2
  %6997 = load ptr, ptr %6996, align 8
  %6998 = load ptr, ptr %3, align 8
  %6999 = getelementptr inbounds %struct.lua_State, ptr %6998, i32 0, i32 8
  store ptr %6997, ptr %6999, align 8
  %7000 = load ptr, ptr %7, align 8
  %7001 = getelementptr inbounds %union.StackValue, ptr %7000, i64 -1
  %7002 = load ptr, ptr %3, align 8
  %7003 = getelementptr inbounds %struct.lua_State, ptr %7002, i32 0, i32 6
  store ptr %7001, ptr %7003, align 8
  %7004 = load ptr, ptr %4, align 8
  %7005 = getelementptr inbounds %struct.CallInfo, ptr %7004, i32 0, i32 6
  %7006 = load i16, ptr %7005, align 4
  %7007 = sext i16 %7006 to i32
  store i32 %7007, ptr %401, align 4
  br label %7008

7008:                                             ; preds = %7022, %6994
  %7009 = load i32, ptr %401, align 4
  %7010 = icmp sgt i32 %7009, 0
  %7011 = zext i1 %7010 to i32
  %7012 = icmp ne i32 %7011, 0
  %7013 = zext i1 %7012 to i32
  %7014 = sext i32 %7013 to i64
  %7015 = icmp ne i64 %7014, 0
  br i1 %7015, label %7016, label %7025

7016:                                             ; preds = %7008
  %7017 = load ptr, ptr %3, align 8
  %7018 = getelementptr inbounds %struct.lua_State, ptr %7017, i32 0, i32 6
  %7019 = load ptr, ptr %7018, align 8
  %7020 = getelementptr inbounds %union.StackValue, ptr %7019, i32 1
  store ptr %7020, ptr %7018, align 8
  %7021 = getelementptr inbounds %struct.TValue, ptr %7019, i32 0, i32 1
  store i8 0, ptr %7021, align 8
  br label %7022

7022:                                             ; preds = %7016
  %7023 = load i32, ptr %401, align 4
  %7024 = add nsw i32 %7023, -1
  store i32 %7024, ptr %401, align 4
  br label %7008, !llvm.loop !13

7025:                                             ; preds = %7008
  br label %7026

7026:                                             ; preds = %7025, %6978
  br label %7112

7027:                                             ; preds = %7120
  %7028 = load ptr, ptr %3, align 8
  %7029 = getelementptr inbounds %struct.lua_State, ptr %7028, i32 0, i32 23
  %7030 = load volatile i32, ptr %7029, align 8
  %7031 = icmp ne i32 %7030, 0
  %7032 = zext i1 %7031 to i32
  %7033 = sext i32 %7032 to i64
  %7034 = icmp ne i64 %7033, 0
  br i1 %7034, label %7035, label %7052

7035:                                             ; preds = %7027
  %7036 = load ptr, ptr %7, align 8
  %7037 = load i32, ptr %10, align 4
  %7038 = lshr i32 %7037, 7
  %7039 = and i32 %7038, 255
  %7040 = sext i32 %7039 to i64
  %7041 = getelementptr inbounds %union.StackValue, ptr %7036, i64 %7040
  store ptr %7041, ptr %402, align 8
  %7042 = load ptr, ptr %402, align 8
  %7043 = getelementptr inbounds %union.StackValue, ptr %7042, i64 1
  %7044 = load ptr, ptr %3, align 8
  %7045 = getelementptr inbounds %struct.lua_State, ptr %7044, i32 0, i32 6
  store ptr %7043, ptr %7045, align 8
  %7046 = load ptr, ptr %8, align 8
  %7047 = load ptr, ptr %4, align 8
  %7048 = getelementptr inbounds %struct.CallInfo, ptr %7047, i32 0, i32 4
  %7049 = getelementptr inbounds %struct.anon, ptr %7048, i32 0, i32 0
  store ptr %7046, ptr %7049, align 8
  %7050 = load ptr, ptr %3, align 8
  %7051 = load ptr, ptr %4, align 8
  call void @luaD_poscall(ptr noundef %7050, ptr noundef %7051, i32 noundef 1)
  store i32 1, ptr %9, align 4
  br label %7111

7052:                                             ; preds = %7027
  %7053 = load ptr, ptr %4, align 8
  %7054 = getelementptr inbounds %struct.CallInfo, ptr %7053, i32 0, i32 6
  %7055 = load i16, ptr %7054, align 4
  %7056 = sext i16 %7055 to i32
  store i32 %7056, ptr %403, align 4
  %7057 = load ptr, ptr %4, align 8
  %7058 = getelementptr inbounds %struct.CallInfo, ptr %7057, i32 0, i32 2
  %7059 = load ptr, ptr %7058, align 8
  %7060 = load ptr, ptr %3, align 8
  %7061 = getelementptr inbounds %struct.lua_State, ptr %7060, i32 0, i32 8
  store ptr %7059, ptr %7061, align 8
  %7062 = load i32, ptr %403, align 4
  %7063 = icmp eq i32 %7062, 0
  br i1 %7063, label %7064, label %7069

7064:                                             ; preds = %7052
  %7065 = load ptr, ptr %7, align 8
  %7066 = getelementptr inbounds %union.StackValue, ptr %7065, i64 -1
  %7067 = load ptr, ptr %3, align 8
  %7068 = getelementptr inbounds %struct.lua_State, ptr %7067, i32 0, i32 6
  store ptr %7066, ptr %7068, align 8
  br label %7110

7069:                                             ; preds = %7052
  %7070 = load ptr, ptr %7, align 8
  %7071 = load i32, ptr %10, align 4
  %7072 = lshr i32 %7071, 7
  %7073 = and i32 %7072, 255
  %7074 = sext i32 %7073 to i64
  %7075 = getelementptr inbounds %union.StackValue, ptr %7070, i64 %7074
  store ptr %7075, ptr %404, align 8
  %7076 = load ptr, ptr %7, align 8
  %7077 = getelementptr inbounds %union.StackValue, ptr %7076, i64 -1
  store ptr %7077, ptr %405, align 8
  %7078 = load ptr, ptr %404, align 8
  store ptr %7078, ptr %406, align 8
  %7079 = load ptr, ptr %405, align 8
  %7080 = getelementptr inbounds %struct.TValue, ptr %7079, i32 0, i32 0
  %7081 = load ptr, ptr %406, align 8
  %7082 = getelementptr inbounds %struct.TValue, ptr %7081, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7080, ptr align 8 %7082, i64 8, i1 false)
  %7083 = load ptr, ptr %406, align 8
  %7084 = getelementptr inbounds %struct.TValue, ptr %7083, i32 0, i32 1
  %7085 = load i8, ptr %7084, align 8
  %7086 = load ptr, ptr %405, align 8
  %7087 = getelementptr inbounds %struct.TValue, ptr %7086, i32 0, i32 1
  store i8 %7085, ptr %7087, align 8
  %7088 = load ptr, ptr %3, align 8
  %7089 = load ptr, ptr %7, align 8
  %7090 = load ptr, ptr %3, align 8
  %7091 = getelementptr inbounds %struct.lua_State, ptr %7090, i32 0, i32 6
  store ptr %7089, ptr %7091, align 8
  br label %7092

7092:                                             ; preds = %7106, %7069
  %7093 = load i32, ptr %403, align 4
  %7094 = icmp sgt i32 %7093, 1
  %7095 = zext i1 %7094 to i32
  %7096 = icmp ne i32 %7095, 0
  %7097 = zext i1 %7096 to i32
  %7098 = sext i32 %7097 to i64
  %7099 = icmp ne i64 %7098, 0
  br i1 %7099, label %7100, label %7109

7100:                                             ; preds = %7092
  %7101 = load ptr, ptr %3, align 8
  %7102 = getelementptr inbounds %struct.lua_State, ptr %7101, i32 0, i32 6
  %7103 = load ptr, ptr %7102, align 8
  %7104 = getelementptr inbounds %union.StackValue, ptr %7103, i32 1
  store ptr %7104, ptr %7102, align 8
  %7105 = getelementptr inbounds %struct.TValue, ptr %7103, i32 0, i32 1
  store i8 0, ptr %7105, align 8
  br label %7106

7106:                                             ; preds = %7100
  %7107 = load i32, ptr %403, align 4
  %7108 = add nsw i32 %7107, -1
  store i32 %7108, ptr %403, align 4
  br label %7092, !llvm.loop !14

7109:                                             ; preds = %7092
  br label %7110

7110:                                             ; preds = %7109, %7064
  br label %7111

7111:                                             ; preds = %7110, %7035
  br label %7112

7112:                                             ; preds = %7111, %7026, %6956, %6849
  %7113 = load ptr, ptr %4, align 8
  %7114 = getelementptr inbounds %struct.CallInfo, ptr %7113, i32 0, i32 7
  %7115 = load i16, ptr %7114, align 2
  %7116 = zext i16 %7115 to i32
  %7117 = and i32 %7116, 4
  %7118 = icmp ne i32 %7117, 0
  br i1 %7118, label %7119, label %7122

7119:                                             ; preds = %7112
  ret void

7120:                                             ; preds = %7758, %7734, %7679, %7628, %7550, %7404, %7265, %7213, %6775, %6711, %6625, %6555, %6444, %6333, %6222, %6111, %6023, %5960, %5840, %5720, %5643, %5607, %5566, %5520, %5449, %5398, %5343, %5262, %5160, %5090, %5021, %4955, %4864, %4772, %4681, %4590, %4499, %4360, %4263, %4156, %4018, %3891, %3764, %3637, %3566, %3494, %3419, %3344, %3269, %3130, %3033, %2926, %2788, %2661, %2534, %2407, %2323, %2200, %2097, %1954, %1788, %1594, %1447, %1350, %1230, %1082, %981, %897, %845, %803, %771, %737, %705, %654, %606, %565, %524, %477
  %7121 = phi ptr [ %486, %477 ], [ %533, %524 ], [ %574, %565 ], [ %615, %606 ], [ %663, %654 ], [ %714, %705 ], [ %746, %737 ], [ %780, %771 ], [ %812, %803 ], [ %854, %845 ], [ %906, %897 ], [ %990, %981 ], [ %1091, %1082 ], [ %1239, %1230 ], [ %1359, %1350 ], [ %1456, %1447 ], [ %1603, %1594 ], [ %1797, %1788 ], [ %1963, %1954 ], [ %2106, %2097 ], [ %2209, %2200 ], [ %2332, %2323 ], [ %2416, %2407 ], [ %2543, %2534 ], [ %2670, %2661 ], [ %2797, %2788 ], [ %2935, %2926 ], [ %3042, %3033 ], [ %3139, %3130 ], [ %3278, %3269 ], [ %3353, %3344 ], [ %3428, %3419 ], [ %3503, %3494 ], [ %3575, %3566 ], [ %3646, %3637 ], [ %3773, %3764 ], [ %3900, %3891 ], [ %4027, %4018 ], [ %4165, %4156 ], [ %4272, %4263 ], [ %4369, %4360 ], [ %4508, %4499 ], [ %4599, %4590 ], [ %4690, %4681 ], [ %4781, %4772 ], [ %4873, %4864 ], [ %4964, %4955 ], [ %5030, %5021 ], [ %5099, %5090 ], [ %5169, %5160 ], [ %5271, %5262 ], [ %5352, %5343 ], [ %5407, %5398 ], [ %5458, %5449 ], [ %5529, %5520 ], [ %5575, %5566 ], [ %5616, %5607 ], [ %5652, %5643 ], [ %5729, %5720 ], [ %5849, %5840 ], [ %5969, %5960 ], [ %6032, %6023 ], [ %6120, %6111 ], [ %6231, %6222 ], [ %6342, %6333 ], [ %6453, %6444 ], [ %6564, %6555 ], [ %6634, %6625 ], [ %6720, %6711 ], [ %6784, %6775 ], [ %7222, %7213 ], [ %7274, %7265 ], [ %7413, %7404 ], [ %7559, %7550 ], [ %7637, %7628 ], [ %7688, %7679 ], [ %7743, %7734 ], [ %7767, %7758 ]
  indirectbr ptr %7121, [label %487, label %534, label %575, label %616, label %664, label %715, label %747, label %781, label %813, label %855, label %907, label %991, label %1092, label %1240, label %1360, label %1457, label %1604, label %1798, label %1964, label %2107, label %2210, label %2333, label %2417, label %2544, label %2671, label %2798, label %2936, label %3043, label %3140, label %3279, label %3354, label %3429, label %3504, label %3576, label %3647, label %3774, label %3901, label %4028, label %4166, label %4273, label %4370, label %4509, label %4600, label %4691, label %4874, label %4782, label %4965, label %5031, label %5100, label %5170, label %5272, label %5353, label %5408, label %5459, label %5530, label %5576, label %5617, label %5653, label %5730, label %5850, label %5970, label %6033, label %6121, label %6232, label %6343, label %6454, label %6565, label %6635, label %6721, label %6785, label %6864, label %6970, label %7027, label %7126, label %7223, label %7275, label %7303, label %7353, label %7414, label %7560, label %7638, label %7689, label %7744]

7122:                                             ; preds = %7112
  %7123 = load ptr, ptr %4, align 8
  %7124 = getelementptr inbounds %struct.CallInfo, ptr %7123, i32 0, i32 2
  %7125 = load ptr, ptr %7124, align 8
  store ptr %7125, ptr %4, align 8
  br label %435

7126:                                             ; preds = %7120
  %7127 = load ptr, ptr %7, align 8
  %7128 = load i32, ptr %10, align 4
  %7129 = lshr i32 %7128, 7
  %7130 = and i32 %7129, 255
  %7131 = sext i32 %7130 to i64
  %7132 = getelementptr inbounds %union.StackValue, ptr %7127, i64 %7131
  store ptr %7132, ptr %407, align 8
  %7133 = load ptr, ptr %407, align 8
  %7134 = getelementptr inbounds %union.StackValue, ptr %7133, i64 2
  %7135 = getelementptr inbounds %struct.TValue, ptr %7134, i32 0, i32 1
  %7136 = load i8, ptr %7135, align 8
  %7137 = zext i8 %7136 to i32
  %7138 = icmp eq i32 %7137, 3
  br i1 %7138, label %7139, label %7182

7139:                                             ; preds = %7126
  %7140 = load ptr, ptr %407, align 8
  %7141 = getelementptr inbounds %union.StackValue, ptr %7140, i64 1
  %7142 = getelementptr inbounds %struct.TValue, ptr %7141, i32 0, i32 0
  %7143 = load i64, ptr %7142, align 8
  store i64 %7143, ptr %408, align 8
  %7144 = load i64, ptr %408, align 8
  %7145 = icmp ugt i64 %7144, 0
  br i1 %7145, label %7146, label %7181

7146:                                             ; preds = %7139
  %7147 = load ptr, ptr %407, align 8
  %7148 = getelementptr inbounds %union.StackValue, ptr %7147, i64 2
  %7149 = getelementptr inbounds %struct.TValue, ptr %7148, i32 0, i32 0
  %7150 = load i64, ptr %7149, align 8
  store i64 %7150, ptr %409, align 8
  %7151 = load ptr, ptr %407, align 8
  %7152 = getelementptr inbounds %struct.TValue, ptr %7151, i32 0, i32 0
  %7153 = load i64, ptr %7152, align 8
  store i64 %7153, ptr %410, align 8
  %7154 = load ptr, ptr %407, align 8
  %7155 = getelementptr inbounds %union.StackValue, ptr %7154, i64 1
  store ptr %7155, ptr %411, align 8
  %7156 = load i64, ptr %408, align 8
  %7157 = sub i64 %7156, 1
  %7158 = load ptr, ptr %411, align 8
  %7159 = getelementptr inbounds %struct.TValue, ptr %7158, i32 0, i32 0
  store i64 %7157, ptr %7159, align 8
  %7160 = load i64, ptr %410, align 8
  %7161 = load i64, ptr %409, align 8
  %7162 = add i64 %7160, %7161
  store i64 %7162, ptr %410, align 8
  %7163 = load ptr, ptr %407, align 8
  store ptr %7163, ptr %412, align 8
  %7164 = load i64, ptr %410, align 8
  %7165 = load ptr, ptr %412, align 8
  %7166 = getelementptr inbounds %struct.TValue, ptr %7165, i32 0, i32 0
  store i64 %7164, ptr %7166, align 8
  %7167 = load ptr, ptr %407, align 8
  %7168 = getelementptr inbounds %union.StackValue, ptr %7167, i64 3
  store ptr %7168, ptr %413, align 8
  %7169 = load i64, ptr %410, align 8
  %7170 = load ptr, ptr %413, align 8
  %7171 = getelementptr inbounds %struct.TValue, ptr %7170, i32 0, i32 0
  store i64 %7169, ptr %7171, align 8
  %7172 = load ptr, ptr %413, align 8
  %7173 = getelementptr inbounds %struct.TValue, ptr %7172, i32 0, i32 1
  store i8 3, ptr %7173, align 8
  %7174 = load i32, ptr %10, align 4
  %7175 = lshr i32 %7174, 15
  %7176 = and i32 %7175, 131071
  %7177 = load ptr, ptr %8, align 8
  %7178 = sext i32 %7176 to i64
  %7179 = sub i64 0, %7178
  %7180 = getelementptr inbounds i32, ptr %7177, i64 %7179
  store ptr %7180, ptr %8, align 8
  br label %7181

7181:                                             ; preds = %7146, %7139
  br label %7195

7182:                                             ; preds = %7126
  %7183 = load ptr, ptr %407, align 8
  %7184 = call i32 @floatforloop(ptr noundef %7183)
  %7185 = icmp ne i32 %7184, 0
  br i1 %7185, label %7186, label %7194

7186:                                             ; preds = %7182
  %7187 = load i32, ptr %10, align 4
  %7188 = lshr i32 %7187, 15
  %7189 = and i32 %7188, 131071
  %7190 = load ptr, ptr %8, align 8
  %7191 = sext i32 %7189 to i64
  %7192 = sub i64 0, %7191
  %7193 = getelementptr inbounds i32, ptr %7190, i64 %7192
  store ptr %7193, ptr %8, align 8
  br label %7194

7194:                                             ; preds = %7186, %7182
  br label %7195

7195:                                             ; preds = %7194, %7181
  %7196 = load ptr, ptr %4, align 8
  %7197 = getelementptr inbounds %struct.CallInfo, ptr %7196, i32 0, i32 4
  %7198 = getelementptr inbounds %struct.anon, ptr %7197, i32 0, i32 1
  %7199 = load volatile i32, ptr %7198, align 8
  store i32 %7199, ptr %9, align 4
  %7200 = load i32, ptr %9, align 4
  %7201 = icmp ne i32 %7200, 0
  %7202 = zext i1 %7201 to i32
  %7203 = sext i32 %7202 to i64
  %7204 = icmp ne i64 %7203, 0
  br i1 %7204, label %7205, label %7213

7205:                                             ; preds = %7195
  %7206 = load ptr, ptr %3, align 8
  %7207 = load ptr, ptr %8, align 8
  %7208 = call i32 @luaG_traceexec(ptr noundef %7206, ptr noundef %7207)
  store i32 %7208, ptr %9, align 4
  %7209 = load ptr, ptr %4, align 8
  %7210 = getelementptr inbounds %struct.CallInfo, ptr %7209, i32 0, i32 0
  %7211 = load ptr, ptr %7210, align 8
  %7212 = getelementptr inbounds %union.StackValue, ptr %7211, i64 1
  store ptr %7212, ptr %7, align 8
  br label %7213

7213:                                             ; preds = %7205, %7195
  %7214 = load ptr, ptr %8, align 8
  %7215 = getelementptr inbounds i32, ptr %7214, i32 1
  store ptr %7215, ptr %8, align 8
  %7216 = load i32, ptr %7214, align 4
  store i32 %7216, ptr %10, align 4
  %7217 = load i32, ptr %10, align 4
  %7218 = lshr i32 %7217, 0
  %7219 = and i32 %7218, 127
  %7220 = zext i32 %7219 to i64
  %7221 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7220
  %7222 = load ptr, ptr %7221, align 8
  br label %7120

7223:                                             ; preds = %7120
  %7224 = load ptr, ptr %7, align 8
  %7225 = load i32, ptr %10, align 4
  %7226 = lshr i32 %7225, 7
  %7227 = and i32 %7226, 255
  %7228 = sext i32 %7227 to i64
  %7229 = getelementptr inbounds %union.StackValue, ptr %7224, i64 %7228
  store ptr %7229, ptr %414, align 8
  %7230 = load ptr, ptr %8, align 8
  %7231 = load ptr, ptr %4, align 8
  %7232 = getelementptr inbounds %struct.CallInfo, ptr %7231, i32 0, i32 4
  %7233 = getelementptr inbounds %struct.anon, ptr %7232, i32 0, i32 0
  store ptr %7230, ptr %7233, align 8
  %7234 = load ptr, ptr %4, align 8
  %7235 = getelementptr inbounds %struct.CallInfo, ptr %7234, i32 0, i32 1
  %7236 = load ptr, ptr %7235, align 8
  %7237 = load ptr, ptr %3, align 8
  %7238 = getelementptr inbounds %struct.lua_State, ptr %7237, i32 0, i32 6
  store ptr %7236, ptr %7238, align 8
  %7239 = load ptr, ptr %3, align 8
  %7240 = load ptr, ptr %414, align 8
  %7241 = call i32 @forprep(ptr noundef %7239, ptr noundef %7240)
  %7242 = icmp ne i32 %7241, 0
  br i1 %7242, label %7243, label %7251

7243:                                             ; preds = %7223
  %7244 = load i32, ptr %10, align 4
  %7245 = lshr i32 %7244, 15
  %7246 = and i32 %7245, 131071
  %7247 = add nsw i32 %7246, 1
  %7248 = load ptr, ptr %8, align 8
  %7249 = sext i32 %7247 to i64
  %7250 = getelementptr inbounds i32, ptr %7248, i64 %7249
  store ptr %7250, ptr %8, align 8
  br label %7251

7251:                                             ; preds = %7243, %7223
  %7252 = load i32, ptr %9, align 4
  %7253 = icmp ne i32 %7252, 0
  %7254 = zext i1 %7253 to i32
  %7255 = sext i32 %7254 to i64
  %7256 = icmp ne i64 %7255, 0
  br i1 %7256, label %7257, label %7265

7257:                                             ; preds = %7251
  %7258 = load ptr, ptr %3, align 8
  %7259 = load ptr, ptr %8, align 8
  %7260 = call i32 @luaG_traceexec(ptr noundef %7258, ptr noundef %7259)
  store i32 %7260, ptr %9, align 4
  %7261 = load ptr, ptr %4, align 8
  %7262 = getelementptr inbounds %struct.CallInfo, ptr %7261, i32 0, i32 0
  %7263 = load ptr, ptr %7262, align 8
  %7264 = getelementptr inbounds %union.StackValue, ptr %7263, i64 1
  store ptr %7264, ptr %7, align 8
  br label %7265

7265:                                             ; preds = %7257, %7251
  %7266 = load ptr, ptr %8, align 8
  %7267 = getelementptr inbounds i32, ptr %7266, i32 1
  store ptr %7267, ptr %8, align 8
  %7268 = load i32, ptr %7266, align 4
  store i32 %7268, ptr %10, align 4
  %7269 = load i32, ptr %10, align 4
  %7270 = lshr i32 %7269, 0
  %7271 = and i32 %7270, 127
  %7272 = zext i32 %7271 to i64
  %7273 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7272
  %7274 = load ptr, ptr %7273, align 8
  br label %7120

7275:                                             ; preds = %7120
  %7276 = load ptr, ptr %7, align 8
  %7277 = load i32, ptr %10, align 4
  %7278 = lshr i32 %7277, 7
  %7279 = and i32 %7278, 255
  %7280 = sext i32 %7279 to i64
  %7281 = getelementptr inbounds %union.StackValue, ptr %7276, i64 %7280
  store ptr %7281, ptr %415, align 8
  %7282 = load ptr, ptr %8, align 8
  %7283 = load ptr, ptr %4, align 8
  %7284 = getelementptr inbounds %struct.CallInfo, ptr %7283, i32 0, i32 4
  %7285 = getelementptr inbounds %struct.anon, ptr %7284, i32 0, i32 0
  store ptr %7282, ptr %7285, align 8
  %7286 = load ptr, ptr %4, align 8
  %7287 = getelementptr inbounds %struct.CallInfo, ptr %7286, i32 0, i32 1
  %7288 = load ptr, ptr %7287, align 8
  %7289 = load ptr, ptr %3, align 8
  %7290 = getelementptr inbounds %struct.lua_State, ptr %7289, i32 0, i32 6
  store ptr %7288, ptr %7290, align 8
  %7291 = load ptr, ptr %3, align 8
  %7292 = load ptr, ptr %415, align 8
  %7293 = getelementptr inbounds %union.StackValue, ptr %7292, i64 3
  call void @luaF_newtbcupval(ptr noundef %7291, ptr noundef %7293)
  %7294 = load i32, ptr %10, align 4
  %7295 = lshr i32 %7294, 15
  %7296 = and i32 %7295, 131071
  %7297 = load ptr, ptr %8, align 8
  %7298 = sext i32 %7296 to i64
  %7299 = getelementptr inbounds i32, ptr %7297, i64 %7298
  store ptr %7299, ptr %8, align 8
  %7300 = load ptr, ptr %8, align 8
  %7301 = getelementptr inbounds i32, ptr %7300, i32 1
  store ptr %7301, ptr %8, align 8
  %7302 = load i32, ptr %7300, align 4
  store i32 %7302, ptr %10, align 4
  br label %7304

7303:                                             ; preds = %7120
  br label %7304

7304:                                             ; preds = %7303, %7275
  %7305 = load ptr, ptr %7, align 8
  %7306 = load i32, ptr %10, align 4
  %7307 = lshr i32 %7306, 7
  %7308 = and i32 %7307, 255
  %7309 = sext i32 %7308 to i64
  %7310 = getelementptr inbounds %union.StackValue, ptr %7305, i64 %7309
  store ptr %7310, ptr %416, align 8
  %7311 = load ptr, ptr %416, align 8
  %7312 = getelementptr inbounds %union.StackValue, ptr %7311, i64 4
  %7313 = load ptr, ptr %416, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7312, ptr align 8 %7313, i64 48, i1 false)
  %7314 = load ptr, ptr %416, align 8
  %7315 = getelementptr inbounds %union.StackValue, ptr %7314, i64 4
  %7316 = getelementptr inbounds %union.StackValue, ptr %7315, i64 3
  %7317 = load ptr, ptr %3, align 8
  %7318 = getelementptr inbounds %struct.lua_State, ptr %7317, i32 0, i32 6
  store ptr %7316, ptr %7318, align 8
  %7319 = load ptr, ptr %8, align 8
  %7320 = load ptr, ptr %4, align 8
  %7321 = getelementptr inbounds %struct.CallInfo, ptr %7320, i32 0, i32 4
  %7322 = getelementptr inbounds %struct.anon, ptr %7321, i32 0, i32 0
  store ptr %7319, ptr %7322, align 8
  %7323 = load ptr, ptr %3, align 8
  %7324 = load ptr, ptr %416, align 8
  %7325 = getelementptr inbounds %union.StackValue, ptr %7324, i64 4
  %7326 = load i32, ptr %10, align 4
  %7327 = lshr i32 %7326, 24
  %7328 = and i32 %7327, 255
  call void @luaD_call(ptr noundef %7323, ptr noundef %7325, i32 noundef %7328)
  %7329 = load ptr, ptr %4, align 8
  %7330 = getelementptr inbounds %struct.CallInfo, ptr %7329, i32 0, i32 4
  %7331 = getelementptr inbounds %struct.anon, ptr %7330, i32 0, i32 1
  %7332 = load volatile i32, ptr %7331, align 8
  store i32 %7332, ptr %9, align 4
  %7333 = load i32, ptr %9, align 4
  %7334 = icmp ne i32 %7333, 0
  %7335 = zext i1 %7334 to i32
  %7336 = sext i32 %7335 to i64
  %7337 = icmp ne i64 %7336, 0
  br i1 %7337, label %7338, label %7349

7338:                                             ; preds = %7304
  %7339 = load ptr, ptr %4, align 8
  %7340 = getelementptr inbounds %struct.CallInfo, ptr %7339, i32 0, i32 0
  %7341 = load ptr, ptr %7340, align 8
  %7342 = getelementptr inbounds %union.StackValue, ptr %7341, i64 1
  store ptr %7342, ptr %7, align 8
  %7343 = load ptr, ptr %7, align 8
  %7344 = load i32, ptr %10, align 4
  %7345 = lshr i32 %7344, 7
  %7346 = and i32 %7345, 255
  %7347 = sext i32 %7346 to i64
  %7348 = getelementptr inbounds %union.StackValue, ptr %7343, i64 %7347
  store ptr %7348, ptr %416, align 8
  br label %7349

7349:                                             ; preds = %7338, %7304
  %7350 = load ptr, ptr %8, align 8
  %7351 = getelementptr inbounds i32, ptr %7350, i32 1
  store ptr %7351, ptr %8, align 8
  %7352 = load i32, ptr %7350, align 4
  store i32 %7352, ptr %10, align 4
  br label %7354

7353:                                             ; preds = %7120
  br label %7354

7354:                                             ; preds = %7353, %7349
  %7355 = load ptr, ptr %7, align 8
  %7356 = load i32, ptr %10, align 4
  %7357 = lshr i32 %7356, 7
  %7358 = and i32 %7357, 255
  %7359 = sext i32 %7358 to i64
  %7360 = getelementptr inbounds %union.StackValue, ptr %7355, i64 %7359
  store ptr %7360, ptr %417, align 8
  %7361 = load ptr, ptr %417, align 8
  %7362 = getelementptr inbounds %union.StackValue, ptr %7361, i64 4
  %7363 = getelementptr inbounds %struct.TValue, ptr %7362, i32 0, i32 1
  %7364 = load i8, ptr %7363, align 8
  %7365 = zext i8 %7364 to i32
  %7366 = and i32 %7365, 15
  %7367 = icmp eq i32 %7366, 0
  br i1 %7367, label %7390, label %7368

7368:                                             ; preds = %7354
  %7369 = load ptr, ptr %417, align 8
  %7370 = getelementptr inbounds %union.StackValue, ptr %7369, i64 2
  store ptr %7370, ptr %418, align 8
  %7371 = load ptr, ptr %417, align 8
  %7372 = getelementptr inbounds %union.StackValue, ptr %7371, i64 4
  store ptr %7372, ptr %419, align 8
  %7373 = load ptr, ptr %418, align 8
  %7374 = getelementptr inbounds %struct.TValue, ptr %7373, i32 0, i32 0
  %7375 = load ptr, ptr %419, align 8
  %7376 = getelementptr inbounds %struct.TValue, ptr %7375, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7374, ptr align 8 %7376, i64 8, i1 false)
  %7377 = load ptr, ptr %419, align 8
  %7378 = getelementptr inbounds %struct.TValue, ptr %7377, i32 0, i32 1
  %7379 = load i8, ptr %7378, align 8
  %7380 = load ptr, ptr %418, align 8
  %7381 = getelementptr inbounds %struct.TValue, ptr %7380, i32 0, i32 1
  store i8 %7379, ptr %7381, align 8
  %7382 = load ptr, ptr %3, align 8
  %7383 = load i32, ptr %10, align 4
  %7384 = lshr i32 %7383, 15
  %7385 = and i32 %7384, 131071
  %7386 = load ptr, ptr %8, align 8
  %7387 = sext i32 %7385 to i64
  %7388 = sub i64 0, %7387
  %7389 = getelementptr inbounds i32, ptr %7386, i64 %7388
  store ptr %7389, ptr %8, align 8
  br label %7390

7390:                                             ; preds = %7368, %7354
  %7391 = load i32, ptr %9, align 4
  %7392 = icmp ne i32 %7391, 0
  %7393 = zext i1 %7392 to i32
  %7394 = sext i32 %7393 to i64
  %7395 = icmp ne i64 %7394, 0
  br i1 %7395, label %7396, label %7404

7396:                                             ; preds = %7390
  %7397 = load ptr, ptr %3, align 8
  %7398 = load ptr, ptr %8, align 8
  %7399 = call i32 @luaG_traceexec(ptr noundef %7397, ptr noundef %7398)
  store i32 %7399, ptr %9, align 4
  %7400 = load ptr, ptr %4, align 8
  %7401 = getelementptr inbounds %struct.CallInfo, ptr %7400, i32 0, i32 0
  %7402 = load ptr, ptr %7401, align 8
  %7403 = getelementptr inbounds %union.StackValue, ptr %7402, i64 1
  store ptr %7403, ptr %7, align 8
  br label %7404

7404:                                             ; preds = %7396, %7390
  %7405 = load ptr, ptr %8, align 8
  %7406 = getelementptr inbounds i32, ptr %7405, i32 1
  store ptr %7406, ptr %8, align 8
  %7407 = load i32, ptr %7405, align 4
  store i32 %7407, ptr %10, align 4
  %7408 = load i32, ptr %10, align 4
  %7409 = lshr i32 %7408, 0
  %7410 = and i32 %7409, 127
  %7411 = zext i32 %7410 to i64
  %7412 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7411
  %7413 = load ptr, ptr %7412, align 8
  br label %7120

7414:                                             ; preds = %7120
  %7415 = load ptr, ptr %7, align 8
  %7416 = load i32, ptr %10, align 4
  %7417 = lshr i32 %7416, 7
  %7418 = and i32 %7417, 255
  %7419 = sext i32 %7418 to i64
  %7420 = getelementptr inbounds %union.StackValue, ptr %7415, i64 %7419
  store ptr %7420, ptr %420, align 8
  %7421 = load i32, ptr %10, align 4
  %7422 = lshr i32 %7421, 16
  %7423 = and i32 %7422, 255
  store i32 %7423, ptr %421, align 4
  %7424 = load i32, ptr %10, align 4
  %7425 = lshr i32 %7424, 24
  %7426 = and i32 %7425, 255
  store i32 %7426, ptr %422, align 4
  %7427 = load ptr, ptr %420, align 8
  %7428 = getelementptr inbounds %struct.TValue, ptr %7427, i32 0, i32 0
  %7429 = load ptr, ptr %7428, align 8
  store ptr %7429, ptr %423, align 8
  %7430 = load i32, ptr %421, align 4
  %7431 = icmp eq i32 %7430, 0
  br i1 %7431, label %7432, label %7443

7432:                                             ; preds = %7414
  %7433 = load ptr, ptr %3, align 8
  %7434 = getelementptr inbounds %struct.lua_State, ptr %7433, i32 0, i32 6
  %7435 = load ptr, ptr %7434, align 8
  %7436 = load ptr, ptr %420, align 8
  %7437 = ptrtoint ptr %7435 to i64
  %7438 = ptrtoint ptr %7436 to i64
  %7439 = sub i64 %7437, %7438
  %7440 = sdiv exact i64 %7439, 16
  %7441 = trunc i64 %7440 to i32
  %7442 = sub nsw i32 %7441, 1
  store i32 %7442, ptr %421, align 4
  br label %7449

7443:                                             ; preds = %7414
  %7444 = load ptr, ptr %4, align 8
  %7445 = getelementptr inbounds %struct.CallInfo, ptr %7444, i32 0, i32 1
  %7446 = load ptr, ptr %7445, align 8
  %7447 = load ptr, ptr %3, align 8
  %7448 = getelementptr inbounds %struct.lua_State, ptr %7447, i32 0, i32 6
  store ptr %7446, ptr %7448, align 8
  br label %7449

7449:                                             ; preds = %7443, %7432
  %7450 = load i32, ptr %421, align 4
  %7451 = load i32, ptr %422, align 4
  %7452 = add i32 %7451, %7450
  store i32 %7452, ptr %422, align 4
  %7453 = load i32, ptr %10, align 4
  %7454 = and i32 %7453, 32768
  %7455 = icmp ne i32 %7454, 0
  br i1 %7455, label %7456, label %7466

7456:                                             ; preds = %7449
  %7457 = load ptr, ptr %8, align 8
  %7458 = load i32, ptr %7457, align 4
  %7459 = lshr i32 %7458, 7
  %7460 = and i32 %7459, 33554431
  %7461 = mul nsw i32 %7460, 256
  %7462 = load i32, ptr %422, align 4
  %7463 = add i32 %7462, %7461
  store i32 %7463, ptr %422, align 4
  %7464 = load ptr, ptr %8, align 8
  %7465 = getelementptr inbounds i32, ptr %7464, i32 1
  store ptr %7465, ptr %8, align 8
  br label %7466

7466:                                             ; preds = %7456, %7449
  %7467 = load i32, ptr %422, align 4
  %7468 = load ptr, ptr %423, align 8
  %7469 = call i32 @luaH_realasize(ptr noundef %7468)
  %7470 = icmp ugt i32 %7467, %7469
  br i1 %7470, label %7471, label %7475

7471:                                             ; preds = %7466
  %7472 = load ptr, ptr %3, align 8
  %7473 = load ptr, ptr %423, align 8
  %7474 = load i32, ptr %422, align 4
  call void @luaH_resizearray(ptr noundef %7472, ptr noundef %7473, i32 noundef %7474)
  br label %7475

7475:                                             ; preds = %7471, %7466
  br label %7476

7476:                                             ; preds = %7533, %7475
  %7477 = load i32, ptr %421, align 4
  %7478 = icmp sgt i32 %7477, 0
  br i1 %7478, label %7479, label %7536

7479:                                             ; preds = %7476
  %7480 = load ptr, ptr %420, align 8
  %7481 = load i32, ptr %421, align 4
  %7482 = sext i32 %7481 to i64
  %7483 = getelementptr inbounds %union.StackValue, ptr %7480, i64 %7482
  store ptr %7483, ptr %424, align 8
  %7484 = load ptr, ptr %423, align 8
  %7485 = getelementptr inbounds %struct.Table, ptr %7484, i32 0, i32 6
  %7486 = load ptr, ptr %7485, align 8
  %7487 = load i32, ptr %422, align 4
  %7488 = sub i32 %7487, 1
  %7489 = zext i32 %7488 to i64
  %7490 = getelementptr inbounds %struct.TValue, ptr %7486, i64 %7489
  store ptr %7490, ptr %425, align 8
  %7491 = load ptr, ptr %424, align 8
  store ptr %7491, ptr %426, align 8
  %7492 = load ptr, ptr %425, align 8
  %7493 = getelementptr inbounds %struct.TValue, ptr %7492, i32 0, i32 0
  %7494 = load ptr, ptr %426, align 8
  %7495 = getelementptr inbounds %struct.TValue, ptr %7494, i32 0, i32 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %7493, ptr align 8 %7495, i64 8, i1 false)
  %7496 = load ptr, ptr %426, align 8
  %7497 = getelementptr inbounds %struct.TValue, ptr %7496, i32 0, i32 1
  %7498 = load i8, ptr %7497, align 8
  %7499 = load ptr, ptr %425, align 8
  %7500 = getelementptr inbounds %struct.TValue, ptr %7499, i32 0, i32 1
  store i8 %7498, ptr %7500, align 8
  %7501 = load ptr, ptr %3, align 8
  %7502 = load i32, ptr %422, align 4
  %7503 = add i32 %7502, -1
  store i32 %7503, ptr %422, align 4
  %7504 = load ptr, ptr %424, align 8
  %7505 = getelementptr inbounds %struct.TValue, ptr %7504, i32 0, i32 1
  %7506 = load i8, ptr %7505, align 8
  %7507 = zext i8 %7506 to i32
  %7508 = and i32 %7507, 64
  %7509 = icmp ne i32 %7508, 0
  br i1 %7509, label %7510, label %7531

7510:                                             ; preds = %7479
  %7511 = load ptr, ptr %423, align 8
  %7512 = getelementptr inbounds %struct.GCObject, ptr %7511, i32 0, i32 2
  %7513 = load i8, ptr %7512, align 1
  %7514 = zext i8 %7513 to i32
  %7515 = and i32 %7514, 32
  %7516 = icmp ne i32 %7515, 0
  br i1 %7516, label %7517, label %7529

7517:                                             ; preds = %7510
  %7518 = load ptr, ptr %424, align 8
  %7519 = getelementptr inbounds %struct.TValue, ptr %7518, i32 0, i32 0
  %7520 = load ptr, ptr %7519, align 8
  %7521 = getelementptr inbounds %struct.GCObject, ptr %7520, i32 0, i32 2
  %7522 = load i8, ptr %7521, align 1
  %7523 = zext i8 %7522 to i32
  %7524 = and i32 %7523, 24
  %7525 = icmp ne i32 %7524, 0
  br i1 %7525, label %7526, label %7529

7526:                                             ; preds = %7517
  %7527 = load ptr, ptr %3, align 8
  %7528 = load ptr, ptr %423, align 8
  call void @luaC_barrierback_(ptr noundef %7527, ptr noundef %7528)
  br label %7530

7529:                                             ; preds = %7517, %7510
  br label %7530

7530:                                             ; preds = %7529, %7526
  br label %7532

7531:                                             ; preds = %7479
  br label %7532

7532:                                             ; preds = %7531, %7530
  br label %7533

7533:                                             ; preds = %7532
  %7534 = load i32, ptr %421, align 4
  %7535 = add nsw i32 %7534, -1
  store i32 %7535, ptr %421, align 4
  br label %7476, !llvm.loop !15

7536:                                             ; preds = %7476
  %7537 = load i32, ptr %9, align 4
  %7538 = icmp ne i32 %7537, 0
  %7539 = zext i1 %7538 to i32
  %7540 = sext i32 %7539 to i64
  %7541 = icmp ne i64 %7540, 0
  br i1 %7541, label %7542, label %7550

7542:                                             ; preds = %7536
  %7543 = load ptr, ptr %3, align 8
  %7544 = load ptr, ptr %8, align 8
  %7545 = call i32 @luaG_traceexec(ptr noundef %7543, ptr noundef %7544)
  store i32 %7545, ptr %9, align 4
  %7546 = load ptr, ptr %4, align 8
  %7547 = getelementptr inbounds %struct.CallInfo, ptr %7546, i32 0, i32 0
  %7548 = load ptr, ptr %7547, align 8
  %7549 = getelementptr inbounds %union.StackValue, ptr %7548, i64 1
  store ptr %7549, ptr %7, align 8
  br label %7550

7550:                                             ; preds = %7542, %7536
  %7551 = load ptr, ptr %8, align 8
  %7552 = getelementptr inbounds i32, ptr %7551, i32 1
  store ptr %7552, ptr %8, align 8
  %7553 = load i32, ptr %7551, align 4
  store i32 %7553, ptr %10, align 4
  %7554 = load i32, ptr %10, align 4
  %7555 = lshr i32 %7554, 0
  %7556 = and i32 %7555, 127
  %7557 = zext i32 %7556 to i64
  %7558 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7557
  %7559 = load ptr, ptr %7558, align 8
  br label %7120

7560:                                             ; preds = %7120
  %7561 = load ptr, ptr %7, align 8
  %7562 = load i32, ptr %10, align 4
  %7563 = lshr i32 %7562, 7
  %7564 = and i32 %7563, 255
  %7565 = sext i32 %7564 to i64
  %7566 = getelementptr inbounds %union.StackValue, ptr %7561, i64 %7565
  store ptr %7566, ptr %427, align 8
  %7567 = load ptr, ptr %5, align 8
  %7568 = getelementptr inbounds %struct.LClosure, ptr %7567, i32 0, i32 5
  %7569 = load ptr, ptr %7568, align 8
  %7570 = getelementptr inbounds %struct.Proto, ptr %7569, i32 0, i32 17
  %7571 = load ptr, ptr %7570, align 8
  %7572 = load i32, ptr %10, align 4
  %7573 = lshr i32 %7572, 15
  %7574 = and i32 %7573, 131071
  %7575 = sext i32 %7574 to i64
  %7576 = getelementptr inbounds ptr, ptr %7571, i64 %7575
  %7577 = load ptr, ptr %7576, align 8
  store ptr %7577, ptr %428, align 8
  %7578 = load ptr, ptr %8, align 8
  %7579 = load ptr, ptr %4, align 8
  %7580 = getelementptr inbounds %struct.CallInfo, ptr %7579, i32 0, i32 4
  %7581 = getelementptr inbounds %struct.anon, ptr %7580, i32 0, i32 0
  store ptr %7578, ptr %7581, align 8
  %7582 = load ptr, ptr %4, align 8
  %7583 = getelementptr inbounds %struct.CallInfo, ptr %7582, i32 0, i32 1
  %7584 = load ptr, ptr %7583, align 8
  %7585 = load ptr, ptr %3, align 8
  %7586 = getelementptr inbounds %struct.lua_State, ptr %7585, i32 0, i32 6
  store ptr %7584, ptr %7586, align 8
  %7587 = load ptr, ptr %3, align 8
  %7588 = load ptr, ptr %428, align 8
  %7589 = load ptr, ptr %5, align 8
  %7590 = getelementptr inbounds %struct.LClosure, ptr %7589, i32 0, i32 6
  %7591 = getelementptr inbounds [1 x ptr], ptr %7590, i64 0, i64 0
  %7592 = load ptr, ptr %7, align 8
  %7593 = load ptr, ptr %427, align 8
  call void @pushclosure(ptr noundef %7587, ptr noundef %7588, ptr noundef %7591, ptr noundef %7592, ptr noundef %7593)
  %7594 = load ptr, ptr %3, align 8
  %7595 = getelementptr inbounds %struct.lua_State, ptr %7594, i32 0, i32 7
  %7596 = load ptr, ptr %7595, align 8
  %7597 = getelementptr inbounds %struct.global_State, ptr %7596, i32 0, i32 3
  %7598 = load i64, ptr %7597, align 8
  %7599 = icmp sgt i64 %7598, 0
  br i1 %7599, label %7600, label %7614

7600:                                             ; preds = %7560
  %7601 = load ptr, ptr %8, align 8
  %7602 = load ptr, ptr %4, align 8
  %7603 = getelementptr inbounds %struct.CallInfo, ptr %7602, i32 0, i32 4
  %7604 = getelementptr inbounds %struct.anon, ptr %7603, i32 0, i32 0
  store ptr %7601, ptr %7604, align 8
  %7605 = load ptr, ptr %427, align 8
  %7606 = getelementptr inbounds %union.StackValue, ptr %7605, i64 1
  %7607 = load ptr, ptr %3, align 8
  %7608 = getelementptr inbounds %struct.lua_State, ptr %7607, i32 0, i32 6
  store ptr %7606, ptr %7608, align 8
  %7609 = load ptr, ptr %3, align 8
  call void @luaC_step(ptr noundef %7609)
  %7610 = load ptr, ptr %4, align 8
  %7611 = getelementptr inbounds %struct.CallInfo, ptr %7610, i32 0, i32 4
  %7612 = getelementptr inbounds %struct.anon, ptr %7611, i32 0, i32 1
  %7613 = load volatile i32, ptr %7612, align 8
  store i32 %7613, ptr %9, align 4
  br label %7614

7614:                                             ; preds = %7600, %7560
  %7615 = load i32, ptr %9, align 4
  %7616 = icmp ne i32 %7615, 0
  %7617 = zext i1 %7616 to i32
  %7618 = sext i32 %7617 to i64
  %7619 = icmp ne i64 %7618, 0
  br i1 %7619, label %7620, label %7628

7620:                                             ; preds = %7614
  %7621 = load ptr, ptr %3, align 8
  %7622 = load ptr, ptr %8, align 8
  %7623 = call i32 @luaG_traceexec(ptr noundef %7621, ptr noundef %7622)
  store i32 %7623, ptr %9, align 4
  %7624 = load ptr, ptr %4, align 8
  %7625 = getelementptr inbounds %struct.CallInfo, ptr %7624, i32 0, i32 0
  %7626 = load ptr, ptr %7625, align 8
  %7627 = getelementptr inbounds %union.StackValue, ptr %7626, i64 1
  store ptr %7627, ptr %7, align 8
  br label %7628

7628:                                             ; preds = %7620, %7614
  %7629 = load ptr, ptr %8, align 8
  %7630 = getelementptr inbounds i32, ptr %7629, i32 1
  store ptr %7630, ptr %8, align 8
  %7631 = load i32, ptr %7629, align 4
  store i32 %7631, ptr %10, align 4
  %7632 = load i32, ptr %10, align 4
  %7633 = lshr i32 %7632, 0
  %7634 = and i32 %7633, 127
  %7635 = zext i32 %7634 to i64
  %7636 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7635
  %7637 = load ptr, ptr %7636, align 8
  br label %7120

7638:                                             ; preds = %7120
  %7639 = load ptr, ptr %7, align 8
  %7640 = load i32, ptr %10, align 4
  %7641 = lshr i32 %7640, 7
  %7642 = and i32 %7641, 255
  %7643 = sext i32 %7642 to i64
  %7644 = getelementptr inbounds %union.StackValue, ptr %7639, i64 %7643
  store ptr %7644, ptr %429, align 8
  %7645 = load i32, ptr %10, align 4
  %7646 = lshr i32 %7645, 24
  %7647 = and i32 %7646, 255
  %7648 = sub nsw i32 %7647, 1
  store i32 %7648, ptr %430, align 4
  %7649 = load ptr, ptr %8, align 8
  %7650 = load ptr, ptr %4, align 8
  %7651 = getelementptr inbounds %struct.CallInfo, ptr %7650, i32 0, i32 4
  %7652 = getelementptr inbounds %struct.anon, ptr %7651, i32 0, i32 0
  store ptr %7649, ptr %7652, align 8
  %7653 = load ptr, ptr %4, align 8
  %7654 = getelementptr inbounds %struct.CallInfo, ptr %7653, i32 0, i32 1
  %7655 = load ptr, ptr %7654, align 8
  %7656 = load ptr, ptr %3, align 8
  %7657 = getelementptr inbounds %struct.lua_State, ptr %7656, i32 0, i32 6
  store ptr %7655, ptr %7657, align 8
  %7658 = load ptr, ptr %3, align 8
  %7659 = load ptr, ptr %4, align 8
  %7660 = load ptr, ptr %429, align 8
  %7661 = load i32, ptr %430, align 4
  call void @luaT_getvarargs(ptr noundef %7658, ptr noundef %7659, ptr noundef %7660, i32 noundef %7661)
  %7662 = load ptr, ptr %4, align 8
  %7663 = getelementptr inbounds %struct.CallInfo, ptr %7662, i32 0, i32 4
  %7664 = getelementptr inbounds %struct.anon, ptr %7663, i32 0, i32 1
  %7665 = load volatile i32, ptr %7664, align 8
  store i32 %7665, ptr %9, align 4
  %7666 = load i32, ptr %9, align 4
  %7667 = icmp ne i32 %7666, 0
  %7668 = zext i1 %7667 to i32
  %7669 = sext i32 %7668 to i64
  %7670 = icmp ne i64 %7669, 0
  br i1 %7670, label %7671, label %7679

7671:                                             ; preds = %7638
  %7672 = load ptr, ptr %3, align 8
  %7673 = load ptr, ptr %8, align 8
  %7674 = call i32 @luaG_traceexec(ptr noundef %7672, ptr noundef %7673)
  store i32 %7674, ptr %9, align 4
  %7675 = load ptr, ptr %4, align 8
  %7676 = getelementptr inbounds %struct.CallInfo, ptr %7675, i32 0, i32 0
  %7677 = load ptr, ptr %7676, align 8
  %7678 = getelementptr inbounds %union.StackValue, ptr %7677, i64 1
  store ptr %7678, ptr %7, align 8
  br label %7679

7679:                                             ; preds = %7671, %7638
  %7680 = load ptr, ptr %8, align 8
  %7681 = getelementptr inbounds i32, ptr %7680, i32 1
  store ptr %7681, ptr %8, align 8
  %7682 = load i32, ptr %7680, align 4
  store i32 %7682, ptr %10, align 4
  %7683 = load i32, ptr %10, align 4
  %7684 = lshr i32 %7683, 0
  %7685 = and i32 %7684, 127
  %7686 = zext i32 %7685 to i64
  %7687 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7686
  %7688 = load ptr, ptr %7687, align 8
  br label %7120

7689:                                             ; preds = %7120
  %7690 = load ptr, ptr %8, align 8
  %7691 = load ptr, ptr %4, align 8
  %7692 = getelementptr inbounds %struct.CallInfo, ptr %7691, i32 0, i32 4
  %7693 = getelementptr inbounds %struct.anon, ptr %7692, i32 0, i32 0
  store ptr %7690, ptr %7693, align 8
  %7694 = load ptr, ptr %3, align 8
  %7695 = load i32, ptr %10, align 4
  %7696 = lshr i32 %7695, 7
  %7697 = and i32 %7696, 255
  %7698 = load ptr, ptr %4, align 8
  %7699 = load ptr, ptr %5, align 8
  %7700 = getelementptr inbounds %struct.LClosure, ptr %7699, i32 0, i32 5
  %7701 = load ptr, ptr %7700, align 8
  call void @luaT_adjustvarargs(ptr noundef %7694, i32 noundef %7697, ptr noundef %7698, ptr noundef %7701)
  %7702 = load ptr, ptr %4, align 8
  %7703 = getelementptr inbounds %struct.CallInfo, ptr %7702, i32 0, i32 4
  %7704 = getelementptr inbounds %struct.anon, ptr %7703, i32 0, i32 1
  %7705 = load volatile i32, ptr %7704, align 8
  store i32 %7705, ptr %9, align 4
  %7706 = load i32, ptr %9, align 4
  %7707 = icmp ne i32 %7706, 0
  %7708 = zext i1 %7707 to i32
  %7709 = sext i32 %7708 to i64
  %7710 = icmp ne i64 %7709, 0
  br i1 %7710, label %7711, label %7716

7711:                                             ; preds = %7689
  %7712 = load ptr, ptr %3, align 8
  %7713 = load ptr, ptr %4, align 8
  call void @luaD_hookcall(ptr noundef %7712, ptr noundef %7713)
  %7714 = load ptr, ptr %3, align 8
  %7715 = getelementptr inbounds %struct.lua_State, ptr %7714, i32 0, i32 20
  store i32 1, ptr %7715, align 4
  br label %7716

7716:                                             ; preds = %7711, %7689
  %7717 = load ptr, ptr %4, align 8
  %7718 = getelementptr inbounds %struct.CallInfo, ptr %7717, i32 0, i32 0
  %7719 = load ptr, ptr %7718, align 8
  %7720 = getelementptr inbounds %union.StackValue, ptr %7719, i64 1
  store ptr %7720, ptr %7, align 8
  %7721 = load i32, ptr %9, align 4
  %7722 = icmp ne i32 %7721, 0
  %7723 = zext i1 %7722 to i32
  %7724 = sext i32 %7723 to i64
  %7725 = icmp ne i64 %7724, 0
  br i1 %7725, label %7726, label %7734

7726:                                             ; preds = %7716
  %7727 = load ptr, ptr %3, align 8
  %7728 = load ptr, ptr %8, align 8
  %7729 = call i32 @luaG_traceexec(ptr noundef %7727, ptr noundef %7728)
  store i32 %7729, ptr %9, align 4
  %7730 = load ptr, ptr %4, align 8
  %7731 = getelementptr inbounds %struct.CallInfo, ptr %7730, i32 0, i32 0
  %7732 = load ptr, ptr %7731, align 8
  %7733 = getelementptr inbounds %union.StackValue, ptr %7732, i64 1
  store ptr %7733, ptr %7, align 8
  br label %7734

7734:                                             ; preds = %7726, %7716
  %7735 = load ptr, ptr %8, align 8
  %7736 = getelementptr inbounds i32, ptr %7735, i32 1
  store ptr %7736, ptr %8, align 8
  %7737 = load i32, ptr %7735, align 4
  store i32 %7737, ptr %10, align 4
  %7738 = load i32, ptr %10, align 4
  %7739 = lshr i32 %7738, 0
  %7740 = and i32 %7739, 127
  %7741 = zext i32 %7740 to i64
  %7742 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7741
  %7743 = load ptr, ptr %7742, align 8
  br label %7120

7744:                                             ; preds = %7120
  %7745 = load i32, ptr %9, align 4
  %7746 = icmp ne i32 %7745, 0
  %7747 = zext i1 %7746 to i32
  %7748 = sext i32 %7747 to i64
  %7749 = icmp ne i64 %7748, 0
  br i1 %7749, label %7750, label %7758

7750:                                             ; preds = %7744
  %7751 = load ptr, ptr %3, align 8
  %7752 = load ptr, ptr %8, align 8
  %7753 = call i32 @luaG_traceexec(ptr noundef %7751, ptr noundef %7752)
  store i32 %7753, ptr %9, align 4
  %7754 = load ptr, ptr %4, align 8
  %7755 = getelementptr inbounds %struct.CallInfo, ptr %7754, i32 0, i32 0
  %7756 = load ptr, ptr %7755, align 8
  %7757 = getelementptr inbounds %union.StackValue, ptr %7756, i64 1
  store ptr %7757, ptr %7, align 8
  br label %7758

7758:                                             ; preds = %7750, %7744
  %7759 = load ptr, ptr %8, align 8
  %7760 = getelementptr inbounds i32, ptr %7759, i32 1
  store ptr %7760, ptr %8, align 8
  %7761 = load i32, ptr %7759, align 4
  store i32 %7761, ptr %10, align 4
  %7762 = load i32, ptr %10, align 4
  %7763 = lshr i32 %7762, 0
  %7764 = and i32 %7763, 127
  %7765 = zext i32 %7764 to i64
  %7766 = getelementptr inbounds [83 x ptr], ptr @luaV_execute.disptab, i64 0, i64 %7765
  %7767 = load ptr, ptr %7766, align 8
  br label %7120
}

declare hidden i32 @luaG_tracecall(ptr noundef) #2

declare hidden i32 @luaG_traceexec(ptr noundef, ptr noundef) #2

declare hidden void @luaC_barrier_(ptr noundef, ptr noundef, ptr noundef) #2

declare hidden ptr @luaH_getshortstr(ptr noundef, ptr noundef) #2

declare hidden ptr @luaH_getint(ptr noundef, i64 noundef) #2

declare hidden ptr @luaH_new(ptr noundef) #2

declare hidden void @luaH_resize(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #2

declare hidden void @luaC_step(ptr noundef) #2

declare hidden ptr @luaH_getstr(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind
declare double @pow(double noundef, double noundef) #5

declare hidden void @luaT_trybinTM(ptr noundef, ptr noundef, ptr noundef, ptr noundef, i32 noundef) #2

declare hidden void @luaT_trybiniTM(ptr noundef, ptr noundef, i64 noundef, i32 noundef, ptr noundef, i32 noundef) #2

declare hidden void @luaT_trybinassocTM(ptr noundef, ptr noundef, ptr noundef, i32 noundef, ptr noundef, i32 noundef) #2

declare hidden ptr @luaF_close(ptr noundef, ptr noundef, i32 noundef, i32 noundef) #2

declare hidden void @luaF_newtbcupval(ptr noundef, ptr noundef) #2

declare hidden i32 @luaT_callorderiTM(ptr noundef, ptr noundef, i32 noundef, i32 noundef, i32 noundef, i32 noundef) #2

declare hidden ptr @luaD_precall(ptr noundef, ptr noundef, i32 noundef) #2

declare hidden void @luaF_closeupval(ptr noundef, ptr noundef) #2

declare hidden i32 @luaD_pretailcall(ptr noundef, ptr noundef, ptr noundef, i32 noundef, i32 noundef) #2

declare hidden void @luaD_poscall(ptr noundef, ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @floatforloop(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca double, align 8
  %5 = alloca double, align 8
  %6 = alloca double, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %9 = load ptr, ptr %3, align 8
  %10 = getelementptr inbounds %union.StackValue, ptr %9, i64 2
  %11 = getelementptr inbounds %struct.TValue, ptr %10, i32 0, i32 0
  %12 = load double, ptr %11, align 8
  store double %12, ptr %4, align 8
  %13 = load ptr, ptr %3, align 8
  %14 = getelementptr inbounds %union.StackValue, ptr %13, i64 1
  %15 = getelementptr inbounds %struct.TValue, ptr %14, i32 0, i32 0
  %16 = load double, ptr %15, align 8
  store double %16, ptr %5, align 8
  %17 = load ptr, ptr %3, align 8
  %18 = getelementptr inbounds %struct.TValue, ptr %17, i32 0, i32 0
  %19 = load double, ptr %18, align 8
  store double %19, ptr %6, align 8
  %20 = load double, ptr %6, align 8
  %21 = load double, ptr %4, align 8
  %22 = fadd double %20, %21
  store double %22, ptr %6, align 8
  %23 = load double, ptr %4, align 8
  %24 = fcmp olt double 0.000000e+00, %23
  br i1 %24, label %25, label %29

25:                                               ; preds = %1
  %26 = load double, ptr %6, align 8
  %27 = load double, ptr %5, align 8
  %28 = fcmp ole double %26, %27
  br i1 %28, label %33, label %45

29:                                               ; preds = %1
  %30 = load double, ptr %5, align 8
  %31 = load double, ptr %6, align 8
  %32 = fcmp ole double %30, %31
  br i1 %32, label %33, label %45

33:                                               ; preds = %29, %25
  %34 = load ptr, ptr %3, align 8
  store ptr %34, ptr %7, align 8
  %35 = load double, ptr %6, align 8
  %36 = load ptr, ptr %7, align 8
  %37 = getelementptr inbounds %struct.TValue, ptr %36, i32 0, i32 0
  store double %35, ptr %37, align 8
  %38 = load ptr, ptr %3, align 8
  %39 = getelementptr inbounds %union.StackValue, ptr %38, i64 3
  store ptr %39, ptr %8, align 8
  %40 = load double, ptr %6, align 8
  %41 = load ptr, ptr %8, align 8
  %42 = getelementptr inbounds %struct.TValue, ptr %41, i32 0, i32 0
  store double %40, ptr %42, align 8
  %43 = load ptr, ptr %8, align 8
  %44 = getelementptr inbounds %struct.TValue, ptr %43, i32 0, i32 1
  store i8 19, ptr %44, align 8
  store i32 1, ptr %2, align 4
  br label %46

45:                                               ; preds = %29, %25
  store i32 0, ptr %2, align 4
  br label %46

46:                                               ; preds = %45, %33
  %47 = load i32, ptr %2, align 4
  ret i32 %47
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @forprep(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca i64, align 8
  %11 = alloca i64, align 8
  %12 = alloca ptr, align 8
  %13 = alloca i64, align 8
  %14 = alloca ptr, align 8
  %15 = alloca double, align 8
  %16 = alloca double, align 8
  %17 = alloca double, align 8
  %18 = alloca ptr, align 8
  %19 = alloca ptr, align 8
  %20 = alloca ptr, align 8
  %21 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %22 = load ptr, ptr %5, align 8
  store ptr %22, ptr %6, align 8
  %23 = load ptr, ptr %5, align 8
  %24 = getelementptr inbounds %union.StackValue, ptr %23, i64 1
  store ptr %24, ptr %7, align 8
  %25 = load ptr, ptr %5, align 8
  %26 = getelementptr inbounds %union.StackValue, ptr %25, i64 2
  store ptr %26, ptr %8, align 8
  %27 = load ptr, ptr %6, align 8
  %28 = getelementptr inbounds %struct.TValue, ptr %27, i32 0, i32 1
  %29 = load i8, ptr %28, align 8
  %30 = zext i8 %29 to i32
  %31 = icmp eq i32 %30, 3
  br i1 %31, label %32, label %96

32:                                               ; preds = %2
  %33 = load ptr, ptr %8, align 8
  %34 = getelementptr inbounds %struct.TValue, ptr %33, i32 0, i32 1
  %35 = load i8, ptr %34, align 8
  %36 = zext i8 %35 to i32
  %37 = icmp eq i32 %36, 3
  br i1 %37, label %38, label %96

38:                                               ; preds = %32
  %39 = load ptr, ptr %6, align 8
  %40 = getelementptr inbounds %struct.TValue, ptr %39, i32 0, i32 0
  %41 = load i64, ptr %40, align 8
  store i64 %41, ptr %9, align 8
  %42 = load ptr, ptr %8, align 8
  %43 = getelementptr inbounds %struct.TValue, ptr %42, i32 0, i32 0
  %44 = load i64, ptr %43, align 8
  store i64 %44, ptr %10, align 8
  %45 = load i64, ptr %10, align 8
  %46 = icmp eq i64 %45, 0
  br i1 %46, label %47, label %49

47:                                               ; preds = %38
  %48 = load ptr, ptr %4, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %48, ptr noundef @.str.7) #7
  unreachable

49:                                               ; preds = %38
  %50 = load ptr, ptr %5, align 8
  %51 = getelementptr inbounds %union.StackValue, ptr %50, i64 3
  store ptr %51, ptr %12, align 8
  %52 = load i64, ptr %9, align 8
  %53 = load ptr, ptr %12, align 8
  %54 = getelementptr inbounds %struct.TValue, ptr %53, i32 0, i32 0
  store i64 %52, ptr %54, align 8
  %55 = load ptr, ptr %12, align 8
  %56 = getelementptr inbounds %struct.TValue, ptr %55, i32 0, i32 1
  store i8 3, ptr %56, align 8
  %57 = load ptr, ptr %4, align 8
  %58 = load i64, ptr %9, align 8
  %59 = load ptr, ptr %7, align 8
  %60 = load i64, ptr %10, align 8
  %61 = call i32 @forlimit(ptr noundef %57, i64 noundef %58, ptr noundef %59, ptr noundef %11, i64 noundef %60)
  %62 = icmp ne i32 %61, 0
  br i1 %62, label %63, label %64

63:                                               ; preds = %49
  store i32 1, ptr %3, align 4
  br label %216

64:                                               ; preds = %49
  %65 = load i64, ptr %10, align 8
  %66 = icmp sgt i64 %65, 0
  br i1 %66, label %67, label %78

67:                                               ; preds = %64
  %68 = load i64, ptr %11, align 8
  %69 = load i64, ptr %9, align 8
  %70 = sub i64 %68, %69
  store i64 %70, ptr %13, align 8
  %71 = load i64, ptr %10, align 8
  %72 = icmp ne i64 %71, 1
  br i1 %72, label %73, label %77

73:                                               ; preds = %67
  %74 = load i64, ptr %10, align 8
  %75 = load i64, ptr %13, align 8
  %76 = udiv i64 %75, %74
  store i64 %76, ptr %13, align 8
  br label %77

77:                                               ; preds = %73, %67
  br label %88

78:                                               ; preds = %64
  %79 = load i64, ptr %9, align 8
  %80 = load i64, ptr %11, align 8
  %81 = sub i64 %79, %80
  store i64 %81, ptr %13, align 8
  %82 = load i64, ptr %10, align 8
  %83 = add nsw i64 %82, 1
  %84 = sub nsw i64 0, %83
  %85 = add i64 %84, 1
  %86 = load i64, ptr %13, align 8
  %87 = udiv i64 %86, %85
  store i64 %87, ptr %13, align 8
  br label %88

88:                                               ; preds = %78, %77
  %89 = load ptr, ptr %7, align 8
  store ptr %89, ptr %14, align 8
  %90 = load i64, ptr %13, align 8
  %91 = load ptr, ptr %14, align 8
  %92 = getelementptr inbounds %struct.TValue, ptr %91, i32 0, i32 0
  store i64 %90, ptr %92, align 8
  %93 = load ptr, ptr %14, align 8
  %94 = getelementptr inbounds %struct.TValue, ptr %93, i32 0, i32 1
  store i8 3, ptr %94, align 8
  br label %95

95:                                               ; preds = %88
  br label %215

96:                                               ; preds = %32, %2
  %97 = load ptr, ptr %7, align 8
  %98 = getelementptr inbounds %struct.TValue, ptr %97, i32 0, i32 1
  %99 = load i8, ptr %98, align 8
  %100 = zext i8 %99 to i32
  %101 = icmp eq i32 %100, 19
  br i1 %101, label %102, label %106

102:                                              ; preds = %96
  %103 = load ptr, ptr %7, align 8
  %104 = getelementptr inbounds %struct.TValue, ptr %103, i32 0, i32 0
  %105 = load double, ptr %104, align 8
  store double %105, ptr %16, align 8
  br label %109

106:                                              ; preds = %96
  %107 = load ptr, ptr %7, align 8
  %108 = call i32 @luaV_tonumber_(ptr noundef %107, ptr noundef %16)
  br label %109

109:                                              ; preds = %106, %102
  %110 = phi i32 [ 1, %102 ], [ %108, %106 ]
  %111 = icmp ne i32 %110, 0
  %112 = xor i1 %111, true
  %113 = zext i1 %112 to i32
  %114 = icmp ne i32 %113, 0
  %115 = zext i1 %114 to i32
  %116 = sext i32 %115 to i64
  %117 = icmp ne i64 %116, 0
  br i1 %117, label %118, label %121

118:                                              ; preds = %109
  %119 = load ptr, ptr %4, align 8
  %120 = load ptr, ptr %7, align 8
  call void @luaG_forerror(ptr noundef %119, ptr noundef %120, ptr noundef @.str.8) #7
  unreachable

121:                                              ; preds = %109
  %122 = load ptr, ptr %8, align 8
  %123 = getelementptr inbounds %struct.TValue, ptr %122, i32 0, i32 1
  %124 = load i8, ptr %123, align 8
  %125 = zext i8 %124 to i32
  %126 = icmp eq i32 %125, 19
  br i1 %126, label %127, label %131

127:                                              ; preds = %121
  %128 = load ptr, ptr %8, align 8
  %129 = getelementptr inbounds %struct.TValue, ptr %128, i32 0, i32 0
  %130 = load double, ptr %129, align 8
  store double %130, ptr %17, align 8
  br label %134

131:                                              ; preds = %121
  %132 = load ptr, ptr %8, align 8
  %133 = call i32 @luaV_tonumber_(ptr noundef %132, ptr noundef %17)
  br label %134

134:                                              ; preds = %131, %127
  %135 = phi i32 [ 1, %127 ], [ %133, %131 ]
  %136 = icmp ne i32 %135, 0
  %137 = xor i1 %136, true
  %138 = zext i1 %137 to i32
  %139 = icmp ne i32 %138, 0
  %140 = zext i1 %139 to i32
  %141 = sext i32 %140 to i64
  %142 = icmp ne i64 %141, 0
  br i1 %142, label %143, label %146

143:                                              ; preds = %134
  %144 = load ptr, ptr %4, align 8
  %145 = load ptr, ptr %8, align 8
  call void @luaG_forerror(ptr noundef %144, ptr noundef %145, ptr noundef @.str.9) #7
  unreachable

146:                                              ; preds = %134
  %147 = load ptr, ptr %6, align 8
  %148 = getelementptr inbounds %struct.TValue, ptr %147, i32 0, i32 1
  %149 = load i8, ptr %148, align 8
  %150 = zext i8 %149 to i32
  %151 = icmp eq i32 %150, 19
  br i1 %151, label %152, label %156

152:                                              ; preds = %146
  %153 = load ptr, ptr %6, align 8
  %154 = getelementptr inbounds %struct.TValue, ptr %153, i32 0, i32 0
  %155 = load double, ptr %154, align 8
  store double %155, ptr %15, align 8
  br label %159

156:                                              ; preds = %146
  %157 = load ptr, ptr %6, align 8
  %158 = call i32 @luaV_tonumber_(ptr noundef %157, ptr noundef %15)
  br label %159

159:                                              ; preds = %156, %152
  %160 = phi i32 [ 1, %152 ], [ %158, %156 ]
  %161 = icmp ne i32 %160, 0
  %162 = xor i1 %161, true
  %163 = zext i1 %162 to i32
  %164 = icmp ne i32 %163, 0
  %165 = zext i1 %164 to i32
  %166 = sext i32 %165 to i64
  %167 = icmp ne i64 %166, 0
  br i1 %167, label %168, label %171

168:                                              ; preds = %159
  %169 = load ptr, ptr %4, align 8
  %170 = load ptr, ptr %6, align 8
  call void @luaG_forerror(ptr noundef %169, ptr noundef %170, ptr noundef @.str.10) #7
  unreachable

171:                                              ; preds = %159
  %172 = load double, ptr %17, align 8
  %173 = fcmp oeq double %172, 0.000000e+00
  br i1 %173, label %174, label %176

174:                                              ; preds = %171
  %175 = load ptr, ptr %4, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %175, ptr noundef @.str.7) #7
  unreachable

176:                                              ; preds = %171
  %177 = load double, ptr %17, align 8
  %178 = fcmp olt double 0.000000e+00, %177
  br i1 %178, label %179, label %183

179:                                              ; preds = %176
  %180 = load double, ptr %16, align 8
  %181 = load double, ptr %15, align 8
  %182 = fcmp olt double %180, %181
  br i1 %182, label %187, label %188

183:                                              ; preds = %176
  %184 = load double, ptr %15, align 8
  %185 = load double, ptr %16, align 8
  %186 = fcmp olt double %184, %185
  br i1 %186, label %187, label %188

187:                                              ; preds = %183, %179
  store i32 1, ptr %3, align 4
  br label %216

188:                                              ; preds = %183, %179
  %189 = load ptr, ptr %7, align 8
  store ptr %189, ptr %18, align 8
  %190 = load double, ptr %16, align 8
  %191 = load ptr, ptr %18, align 8
  %192 = getelementptr inbounds %struct.TValue, ptr %191, i32 0, i32 0
  store double %190, ptr %192, align 8
  %193 = load ptr, ptr %18, align 8
  %194 = getelementptr inbounds %struct.TValue, ptr %193, i32 0, i32 1
  store i8 19, ptr %194, align 8
  %195 = load ptr, ptr %8, align 8
  store ptr %195, ptr %19, align 8
  %196 = load double, ptr %17, align 8
  %197 = load ptr, ptr %19, align 8
  %198 = getelementptr inbounds %struct.TValue, ptr %197, i32 0, i32 0
  store double %196, ptr %198, align 8
  %199 = load ptr, ptr %19, align 8
  %200 = getelementptr inbounds %struct.TValue, ptr %199, i32 0, i32 1
  store i8 19, ptr %200, align 8
  %201 = load ptr, ptr %5, align 8
  store ptr %201, ptr %20, align 8
  %202 = load double, ptr %15, align 8
  %203 = load ptr, ptr %20, align 8
  %204 = getelementptr inbounds %struct.TValue, ptr %203, i32 0, i32 0
  store double %202, ptr %204, align 8
  %205 = load ptr, ptr %20, align 8
  %206 = getelementptr inbounds %struct.TValue, ptr %205, i32 0, i32 1
  store i8 19, ptr %206, align 8
  %207 = load ptr, ptr %5, align 8
  %208 = getelementptr inbounds %union.StackValue, ptr %207, i64 3
  store ptr %208, ptr %21, align 8
  %209 = load double, ptr %15, align 8
  %210 = load ptr, ptr %21, align 8
  %211 = getelementptr inbounds %struct.TValue, ptr %210, i32 0, i32 0
  store double %209, ptr %211, align 8
  %212 = load ptr, ptr %21, align 8
  %213 = getelementptr inbounds %struct.TValue, ptr %212, i32 0, i32 1
  store i8 19, ptr %213, align 8
  br label %214

214:                                              ; preds = %188
  br label %215

215:                                              ; preds = %214, %95
  store i32 0, ptr %3, align 4
  br label %216

216:                                              ; preds = %215, %187, %63
  %217 = load i32, ptr %3, align 4
  ret i32 %217
}

declare hidden void @luaD_call(ptr noundef, ptr noundef, i32 noundef) #2

declare hidden i32 @luaH_realasize(ptr noundef) #2

declare hidden void @luaH_resizearray(ptr noundef, ptr noundef, i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal void @pushclosure(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3, ptr noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca ptr, align 8
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store ptr %3, ptr %9, align 8
  store ptr %4, ptr %10, align 8
  %17 = load ptr, ptr %7, align 8
  %18 = getelementptr inbounds %struct.Proto, ptr %17, i32 0, i32 6
  %19 = load i32, ptr %18, align 8
  store i32 %19, ptr %11, align 4
  %20 = load ptr, ptr %7, align 8
  %21 = getelementptr inbounds %struct.Proto, ptr %20, i32 0, i32 18
  %22 = load ptr, ptr %21, align 8
  store ptr %22, ptr %12, align 8
  %23 = load ptr, ptr %6, align 8
  %24 = load i32, ptr %11, align 4
  %25 = call ptr @luaF_newLclosure(ptr noundef %23, i32 noundef %24)
  store ptr %25, ptr %14, align 8
  %26 = load ptr, ptr %7, align 8
  %27 = load ptr, ptr %14, align 8
  %28 = getelementptr inbounds %struct.LClosure, ptr %27, i32 0, i32 5
  store ptr %26, ptr %28, align 8
  %29 = load ptr, ptr %10, align 8
  store ptr %29, ptr %15, align 8
  %30 = load ptr, ptr %14, align 8
  store ptr %30, ptr %16, align 8
  %31 = load ptr, ptr %16, align 8
  %32 = load ptr, ptr %15, align 8
  %33 = getelementptr inbounds %struct.TValue, ptr %32, i32 0, i32 0
  store ptr %31, ptr %33, align 8
  %34 = load ptr, ptr %15, align 8
  %35 = getelementptr inbounds %struct.TValue, ptr %34, i32 0, i32 1
  store i8 70, ptr %35, align 8
  %36 = load ptr, ptr %6, align 8
  store i32 0, ptr %13, align 4
  br label %37

37:                                               ; preds = %113, %5
  %38 = load i32, ptr %13, align 4
  %39 = load i32, ptr %11, align 4
  %40 = icmp slt i32 %38, %39
  br i1 %40, label %41, label %116

41:                                               ; preds = %37
  %42 = load ptr, ptr %12, align 8
  %43 = load i32, ptr %13, align 4
  %44 = sext i32 %43 to i64
  %45 = getelementptr inbounds %struct.Upvaldesc, ptr %42, i64 %44
  %46 = getelementptr inbounds %struct.Upvaldesc, ptr %45, i32 0, i32 1
  %47 = load i8, ptr %46, align 8
  %48 = icmp ne i8 %47, 0
  br i1 %48, label %49, label %67

49:                                               ; preds = %41
  %50 = load ptr, ptr %6, align 8
  %51 = load ptr, ptr %9, align 8
  %52 = load ptr, ptr %12, align 8
  %53 = load i32, ptr %13, align 4
  %54 = sext i32 %53 to i64
  %55 = getelementptr inbounds %struct.Upvaldesc, ptr %52, i64 %54
  %56 = getelementptr inbounds %struct.Upvaldesc, ptr %55, i32 0, i32 2
  %57 = load i8, ptr %56, align 1
  %58 = zext i8 %57 to i32
  %59 = sext i32 %58 to i64
  %60 = getelementptr inbounds %union.StackValue, ptr %51, i64 %59
  %61 = call ptr @luaF_findupval(ptr noundef %50, ptr noundef %60)
  %62 = load ptr, ptr %14, align 8
  %63 = getelementptr inbounds %struct.LClosure, ptr %62, i32 0, i32 6
  %64 = load i32, ptr %13, align 4
  %65 = sext i32 %64 to i64
  %66 = getelementptr inbounds [1 x ptr], ptr %63, i64 0, i64 %65
  store ptr %61, ptr %66, align 8
  br label %83

67:                                               ; preds = %41
  %68 = load ptr, ptr %8, align 8
  %69 = load ptr, ptr %12, align 8
  %70 = load i32, ptr %13, align 4
  %71 = sext i32 %70 to i64
  %72 = getelementptr inbounds %struct.Upvaldesc, ptr %69, i64 %71
  %73 = getelementptr inbounds %struct.Upvaldesc, ptr %72, i32 0, i32 2
  %74 = load i8, ptr %73, align 1
  %75 = zext i8 %74 to i64
  %76 = getelementptr inbounds ptr, ptr %68, i64 %75
  %77 = load ptr, ptr %76, align 8
  %78 = load ptr, ptr %14, align 8
  %79 = getelementptr inbounds %struct.LClosure, ptr %78, i32 0, i32 6
  %80 = load i32, ptr %13, align 4
  %81 = sext i32 %80 to i64
  %82 = getelementptr inbounds [1 x ptr], ptr %79, i64 0, i64 %81
  store ptr %77, ptr %82, align 8
  br label %83

83:                                               ; preds = %67, %49
  %84 = load ptr, ptr %14, align 8
  %85 = getelementptr inbounds %struct.LClosure, ptr %84, i32 0, i32 2
  %86 = load i8, ptr %85, align 1
  %87 = zext i8 %86 to i32
  %88 = and i32 %87, 32
  %89 = icmp ne i32 %88, 0
  br i1 %89, label %90, label %111

90:                                               ; preds = %83
  %91 = load ptr, ptr %14, align 8
  %92 = getelementptr inbounds %struct.LClosure, ptr %91, i32 0, i32 6
  %93 = load i32, ptr %13, align 4
  %94 = sext i32 %93 to i64
  %95 = getelementptr inbounds [1 x ptr], ptr %92, i64 0, i64 %94
  %96 = load ptr, ptr %95, align 8
  %97 = getelementptr inbounds %struct.UpVal, ptr %96, i32 0, i32 2
  %98 = load i8, ptr %97, align 1
  %99 = zext i8 %98 to i32
  %100 = and i32 %99, 24
  %101 = icmp ne i32 %100, 0
  br i1 %101, label %102, label %111

102:                                              ; preds = %90
  %103 = load ptr, ptr %6, align 8
  %104 = load ptr, ptr %14, align 8
  %105 = load ptr, ptr %14, align 8
  %106 = getelementptr inbounds %struct.LClosure, ptr %105, i32 0, i32 6
  %107 = load i32, ptr %13, align 4
  %108 = sext i32 %107 to i64
  %109 = getelementptr inbounds [1 x ptr], ptr %106, i64 0, i64 %108
  %110 = load ptr, ptr %109, align 8
  call void @luaC_barrier_(ptr noundef %103, ptr noundef %104, ptr noundef %110)
  br label %112

111:                                              ; preds = %90, %83
  br label %112

112:                                              ; preds = %111, %102
  br label %113

113:                                              ; preds = %112
  %114 = load i32, ptr %13, align 4
  %115 = add nsw i32 %114, 1
  store i32 %115, ptr %13, align 4
  br label %37, !llvm.loop !16

116:                                              ; preds = %37
  ret void
}

declare hidden void @luaT_getvarargs(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #2

declare hidden void @luaT_adjustvarargs(ptr noundef, i32 noundef, ptr noundef, ptr noundef) #2

declare hidden void @luaD_hookcall(ptr noundef, ptr noundef) #2

declare hidden i64 @luaO_str2num(ptr noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LTintfloat(i64 noundef %0, double noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  %5 = alloca double, align 8
  %6 = alloca i64, align 8
  store i64 %0, ptr %4, align 8
  store double %1, ptr %5, align 8
  %7 = load i64, ptr %4, align 8
  %8 = add i64 9007199254740992, %7
  %9 = icmp ule i64 %8, 18014398509481984
  br i1 %9, label %10, label %16

10:                                               ; preds = %2
  %11 = load i64, ptr %4, align 8
  %12 = sitofp i64 %11 to double
  %13 = load double, ptr %5, align 8
  %14 = fcmp olt double %12, %13
  %15 = zext i1 %14 to i32
  store i32 %15, ptr %3, align 4
  br label %29

16:                                               ; preds = %2
  %17 = load double, ptr %5, align 8
  %18 = call i32 @luaV_flttointeger(double noundef %17, ptr noundef %6, i32 noundef 2)
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %16
  %21 = load i64, ptr %4, align 8
  %22 = load i64, ptr %6, align 8
  %23 = icmp slt i64 %21, %22
  %24 = zext i1 %23 to i32
  store i32 %24, ptr %3, align 4
  br label %29

25:                                               ; preds = %16
  %26 = load double, ptr %5, align 8
  %27 = fcmp ogt double %26, 0.000000e+00
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %29

29:                                               ; preds = %25, %20, %10
  %30 = load i32, ptr %3, align 4
  ret i32 %30
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LTfloatint(double noundef %0, i64 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca double, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store double %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %7 = load i64, ptr %5, align 8
  %8 = add i64 9007199254740992, %7
  %9 = icmp ule i64 %8, 18014398509481984
  br i1 %9, label %10, label %16

10:                                               ; preds = %2
  %11 = load double, ptr %4, align 8
  %12 = load i64, ptr %5, align 8
  %13 = sitofp i64 %12 to double
  %14 = fcmp olt double %11, %13
  %15 = zext i1 %14 to i32
  store i32 %15, ptr %3, align 4
  br label %29

16:                                               ; preds = %2
  %17 = load double, ptr %4, align 8
  %18 = call i32 @luaV_flttointeger(double noundef %17, ptr noundef %6, i32 noundef 1)
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %16
  %21 = load i64, ptr %6, align 8
  %22 = load i64, ptr %5, align 8
  %23 = icmp slt i64 %21, %22
  %24 = zext i1 %23 to i32
  store i32 %24, ptr %3, align 4
  br label %29

25:                                               ; preds = %16
  %26 = load double, ptr %4, align 8
  %27 = fcmp olt double %26, 0.000000e+00
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %29

29:                                               ; preds = %25, %20, %10
  %30 = load i32, ptr %3, align 4
  ret i32 %30
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @l_strcmp(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i64, align 8
  %10 = alloca i32, align 4
  %11 = alloca i64, align 8
  %12 = alloca i64, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  %13 = load ptr, ptr %4, align 8
  %14 = getelementptr inbounds %struct.TString, ptr %13, i32 0, i32 7
  %15 = getelementptr inbounds [1 x i8], ptr %14, i64 0, i64 0
  store ptr %15, ptr %6, align 8
  %16 = load ptr, ptr %4, align 8
  %17 = getelementptr inbounds %struct.TString, ptr %16, i32 0, i32 4
  %18 = load i8, ptr %17, align 1
  %19 = zext i8 %18 to i32
  %20 = icmp ne i32 %19, 255
  br i1 %20, label %21, label %26

21:                                               ; preds = %2
  %22 = load ptr, ptr %4, align 8
  %23 = getelementptr inbounds %struct.TString, ptr %22, i32 0, i32 4
  %24 = load i8, ptr %23, align 1
  %25 = zext i8 %24 to i64
  br label %30

26:                                               ; preds = %2
  %27 = load ptr, ptr %4, align 8
  %28 = getelementptr inbounds %struct.TString, ptr %27, i32 0, i32 6
  %29 = load i64, ptr %28, align 8
  br label %30

30:                                               ; preds = %26, %21
  %31 = phi i64 [ %25, %21 ], [ %29, %26 ]
  store i64 %31, ptr %7, align 8
  %32 = load ptr, ptr %5, align 8
  %33 = getelementptr inbounds %struct.TString, ptr %32, i32 0, i32 7
  %34 = getelementptr inbounds [1 x i8], ptr %33, i64 0, i64 0
  store ptr %34, ptr %8, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.TString, ptr %35, i32 0, i32 4
  %37 = load i8, ptr %36, align 1
  %38 = zext i8 %37 to i32
  %39 = icmp ne i32 %38, 255
  br i1 %39, label %40, label %45

40:                                               ; preds = %30
  %41 = load ptr, ptr %5, align 8
  %42 = getelementptr inbounds %struct.TString, ptr %41, i32 0, i32 4
  %43 = load i8, ptr %42, align 1
  %44 = zext i8 %43 to i64
  br label %49

45:                                               ; preds = %30
  %46 = load ptr, ptr %5, align 8
  %47 = getelementptr inbounds %struct.TString, ptr %46, i32 0, i32 6
  %48 = load i64, ptr %47, align 8
  br label %49

49:                                               ; preds = %45, %40
  %50 = phi i64 [ %44, %40 ], [ %48, %45 ]
  store i64 %50, ptr %9, align 8
  br label %51

51:                                               ; preds = %96, %49
  %52 = load ptr, ptr %6, align 8
  %53 = load ptr, ptr %8, align 8
  %54 = call i32 @strcoll(ptr noundef %52, ptr noundef %53) #9
  store i32 %54, ptr %10, align 4
  %55 = load i32, ptr %10, align 4
  %56 = icmp ne i32 %55, 0
  br i1 %56, label %57, label %59

57:                                               ; preds = %51
  %58 = load i32, ptr %10, align 4
  store i32 %58, ptr %3, align 4
  br label %97

59:                                               ; preds = %51
  %60 = load ptr, ptr %6, align 8
  %61 = call i64 @strlen(ptr noundef %60) #9
  store i64 %61, ptr %11, align 8
  %62 = load ptr, ptr %8, align 8
  %63 = call i64 @strlen(ptr noundef %62) #9
  store i64 %63, ptr %12, align 8
  %64 = load i64, ptr %12, align 8
  %65 = load i64, ptr %9, align 8
  %66 = icmp eq i64 %64, %65
  br i1 %66, label %67, label %73

67:                                               ; preds = %59
  %68 = load i64, ptr %11, align 8
  %69 = load i64, ptr %7, align 8
  %70 = icmp eq i64 %68, %69
  %71 = zext i1 %70 to i64
  %72 = select i1 %70, i32 0, i32 1
  store i32 %72, ptr %3, align 4
  br label %97

73:                                               ; preds = %59
  %74 = load i64, ptr %11, align 8
  %75 = load i64, ptr %7, align 8
  %76 = icmp eq i64 %74, %75
  br i1 %76, label %77, label %78

77:                                               ; preds = %73
  store i32 -1, ptr %3, align 4
  br label %97

78:                                               ; preds = %73
  br label %79

79:                                               ; preds = %78
  %80 = load i64, ptr %11, align 8
  %81 = add i64 %80, 1
  store i64 %81, ptr %11, align 8
  %82 = load i64, ptr %12, align 8
  %83 = add i64 %82, 1
  store i64 %83, ptr %12, align 8
  %84 = load i64, ptr %11, align 8
  %85 = load ptr, ptr %6, align 8
  %86 = getelementptr inbounds i8, ptr %85, i64 %84
  store ptr %86, ptr %6, align 8
  %87 = load i64, ptr %11, align 8
  %88 = load i64, ptr %7, align 8
  %89 = sub i64 %88, %87
  store i64 %89, ptr %7, align 8
  %90 = load i64, ptr %12, align 8
  %91 = load ptr, ptr %8, align 8
  %92 = getelementptr inbounds i8, ptr %91, i64 %90
  store ptr %92, ptr %8, align 8
  %93 = load i64, ptr %12, align 8
  %94 = load i64, ptr %9, align 8
  %95 = sub i64 %94, %93
  store i64 %95, ptr %9, align 8
  br label %96

96:                                               ; preds = %79
  br label %51

97:                                               ; preds = %77, %67, %57
  %98 = load i32, ptr %3, align 4
  ret i32 %98
}

declare hidden i32 @luaT_callorderTM(ptr noundef, ptr noundef, ptr noundef, i32 noundef) #2

; Function Attrs: nounwind willreturn memory(read)
declare i32 @strcoll(ptr noundef, ptr noundef) #6

; Function Attrs: nounwind willreturn memory(read)
declare i64 @strlen(ptr noundef) #6

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LEintfloat(i64 noundef %0, double noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  %5 = alloca double, align 8
  %6 = alloca i64, align 8
  store i64 %0, ptr %4, align 8
  store double %1, ptr %5, align 8
  %7 = load i64, ptr %4, align 8
  %8 = add i64 9007199254740992, %7
  %9 = icmp ule i64 %8, 18014398509481984
  br i1 %9, label %10, label %16

10:                                               ; preds = %2
  %11 = load i64, ptr %4, align 8
  %12 = sitofp i64 %11 to double
  %13 = load double, ptr %5, align 8
  %14 = fcmp ole double %12, %13
  %15 = zext i1 %14 to i32
  store i32 %15, ptr %3, align 4
  br label %29

16:                                               ; preds = %2
  %17 = load double, ptr %5, align 8
  %18 = call i32 @luaV_flttointeger(double noundef %17, ptr noundef %6, i32 noundef 1)
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %16
  %21 = load i64, ptr %4, align 8
  %22 = load i64, ptr %6, align 8
  %23 = icmp sle i64 %21, %22
  %24 = zext i1 %23 to i32
  store i32 %24, ptr %3, align 4
  br label %29

25:                                               ; preds = %16
  %26 = load double, ptr %5, align 8
  %27 = fcmp ogt double %26, 0.000000e+00
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %29

29:                                               ; preds = %25, %20, %10
  %30 = load i32, ptr %3, align 4
  ret i32 %30
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @LEfloatint(double noundef %0, i64 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca double, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store double %0, ptr %4, align 8
  store i64 %1, ptr %5, align 8
  %7 = load i64, ptr %5, align 8
  %8 = add i64 9007199254740992, %7
  %9 = icmp ule i64 %8, 18014398509481984
  br i1 %9, label %10, label %16

10:                                               ; preds = %2
  %11 = load double, ptr %4, align 8
  %12 = load i64, ptr %5, align 8
  %13 = sitofp i64 %12 to double
  %14 = fcmp ole double %11, %13
  %15 = zext i1 %14 to i32
  store i32 %15, ptr %3, align 4
  br label %29

16:                                               ; preds = %2
  %17 = load double, ptr %4, align 8
  %18 = call i32 @luaV_flttointeger(double noundef %17, ptr noundef %6, i32 noundef 2)
  %19 = icmp ne i32 %18, 0
  br i1 %19, label %20, label %25

20:                                               ; preds = %16
  %21 = load i64, ptr %6, align 8
  %22 = load i64, ptr %5, align 8
  %23 = icmp sle i64 %21, %22
  %24 = zext i1 %23 to i32
  store i32 %24, ptr %3, align 4
  br label %29

25:                                               ; preds = %16
  %26 = load double, ptr %4, align 8
  %27 = fcmp olt double %26, 0.000000e+00
  %28 = zext i1 %27 to i32
  store i32 %28, ptr %3, align 4
  br label %29

29:                                               ; preds = %25, %20, %10
  %30 = load i32, ptr %3, align 4
  ret i32 %30
}

; Function Attrs: noinline nounwind optnone uwtable
define internal i32 @forlimit(ptr noundef %0, i64 noundef %1, ptr noundef %2, ptr noundef %3, i64 noundef %4) #0 {
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i64, align 8
  %12 = alloca double, align 8
  store ptr %0, ptr %7, align 8
  store i64 %1, ptr %8, align 8
  store ptr %2, ptr %9, align 8
  store ptr %3, ptr %10, align 8
  store i64 %4, ptr %11, align 8
  %13 = load ptr, ptr %9, align 8
  %14 = load ptr, ptr %10, align 8
  %15 = load i64, ptr %11, align 8
  %16 = icmp slt i64 %15, 0
  %17 = zext i1 %16 to i64
  %18 = select i1 %16, i32 2, i32 1
  %19 = call i32 @luaV_tointeger(ptr noundef %13, ptr noundef %14, i32 noundef %18)
  %20 = icmp ne i32 %19, 0
  br i1 %20, label %54, label %21

21:                                               ; preds = %5
  %22 = load ptr, ptr %9, align 8
  %23 = getelementptr inbounds %struct.TValue, ptr %22, i32 0, i32 1
  %24 = load i8, ptr %23, align 8
  %25 = zext i8 %24 to i32
  %26 = icmp eq i32 %25, 19
  br i1 %26, label %27, label %31

27:                                               ; preds = %21
  %28 = load ptr, ptr %9, align 8
  %29 = getelementptr inbounds %struct.TValue, ptr %28, i32 0, i32 0
  %30 = load double, ptr %29, align 8
  store double %30, ptr %12, align 8
  br i1 true, label %38, label %35

31:                                               ; preds = %21
  %32 = load ptr, ptr %9, align 8
  %33 = call i32 @luaV_tonumber_(ptr noundef %32, ptr noundef %12)
  %34 = icmp ne i32 %33, 0
  br i1 %34, label %38, label %35

35:                                               ; preds = %31, %27
  %36 = load ptr, ptr %7, align 8
  %37 = load ptr, ptr %9, align 8
  call void @luaG_forerror(ptr noundef %36, ptr noundef %37, ptr noundef @.str.8) #7
  unreachable

38:                                               ; preds = %31, %27
  %39 = load double, ptr %12, align 8
  %40 = fcmp olt double 0.000000e+00, %39
  br i1 %40, label %41, label %47

41:                                               ; preds = %38
  %42 = load i64, ptr %11, align 8
  %43 = icmp slt i64 %42, 0
  br i1 %43, label %44, label %45

44:                                               ; preds = %41
  store i32 1, ptr %6, align 4
  br label %71

45:                                               ; preds = %41
  %46 = load ptr, ptr %10, align 8
  store i64 9223372036854775807, ptr %46, align 8
  br label %53

47:                                               ; preds = %38
  %48 = load i64, ptr %11, align 8
  %49 = icmp sgt i64 %48, 0
  br i1 %49, label %50, label %51

50:                                               ; preds = %47
  store i32 1, ptr %6, align 4
  br label %71

51:                                               ; preds = %47
  %52 = load ptr, ptr %10, align 8
  store i64 -9223372036854775808, ptr %52, align 8
  br label %53

53:                                               ; preds = %51, %45
  br label %54

54:                                               ; preds = %53, %5
  %55 = load i64, ptr %11, align 8
  %56 = icmp sgt i64 %55, 0
  br i1 %56, label %57, label %63

57:                                               ; preds = %54
  %58 = load i64, ptr %8, align 8
  %59 = load ptr, ptr %10, align 8
  %60 = load i64, ptr %59, align 8
  %61 = icmp sgt i64 %58, %60
  %62 = zext i1 %61 to i32
  br label %69

63:                                               ; preds = %54
  %64 = load i64, ptr %8, align 8
  %65 = load ptr, ptr %10, align 8
  %66 = load i64, ptr %65, align 8
  %67 = icmp slt i64 %64, %66
  %68 = zext i1 %67 to i32
  br label %69

69:                                               ; preds = %63, %57
  %70 = phi i32 [ %62, %57 ], [ %68, %63 ]
  store i32 %70, ptr %6, align 4
  br label %71

71:                                               ; preds = %69, %50, %44
  %72 = load i32, ptr %6, align 4
  ret i32 %72
}

; Function Attrs: noreturn
declare hidden void @luaG_forerror(ptr noundef, ptr noundef, ptr noundef) #3

declare hidden ptr @luaF_newLclosure(ptr noundef, i32 noundef) #2

declare hidden ptr @luaF_findupval(ptr noundef, ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #6 = { nounwind willreturn memory(read) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #7 = { noreturn }
attributes #8 = { nounwind }
attributes #9 = { nounwind willreturn memory(read) }

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
