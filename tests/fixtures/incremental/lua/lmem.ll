; ModuleID = 'lmem.c'
source_filename = "lmem.c"
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

@.str = private unnamed_addr constant [26 x i8] c"too many %s (limit is %d)\00", align 1
@.str.1 = private unnamed_addr constant [39 x i8] c"memory allocation error: block too big\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaM_growaux_(ptr noundef %0, ptr noundef %1, i32 noundef %2, ptr noundef %3, i32 noundef %4, i32 noundef %5, ptr noundef %6) #0 {
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca i32, align 4
  %12 = alloca ptr, align 8
  %13 = alloca i32, align 4
  %14 = alloca i32, align 4
  %15 = alloca ptr, align 8
  %16 = alloca ptr, align 8
  %17 = alloca i32, align 4
  store ptr %0, ptr %9, align 8
  store ptr %1, ptr %10, align 8
  store i32 %2, ptr %11, align 4
  store ptr %3, ptr %12, align 8
  store i32 %4, ptr %13, align 4
  store i32 %5, ptr %14, align 4
  store ptr %6, ptr %15, align 8
  %18 = load ptr, ptr %12, align 8
  %19 = load i32, ptr %18, align 4
  store i32 %19, ptr %17, align 4
  %20 = load i32, ptr %11, align 4
  %21 = add nsw i32 %20, 1
  %22 = load i32, ptr %17, align 4
  %23 = icmp sle i32 %21, %22
  br i1 %23, label %24, label %26

24:                                               ; preds = %7
  %25 = load ptr, ptr %10, align 8
  store ptr %25, ptr %8, align 8
  br label %71

26:                                               ; preds = %7
  %27 = load i32, ptr %17, align 4
  %28 = load i32, ptr %14, align 4
  %29 = sdiv i32 %28, 2
  %30 = icmp sge i32 %27, %29
  br i1 %30, label %31, label %46

31:                                               ; preds = %26
  %32 = load i32, ptr %17, align 4
  %33 = load i32, ptr %14, align 4
  %34 = icmp sge i32 %32, %33
  %35 = zext i1 %34 to i32
  %36 = icmp ne i32 %35, 0
  %37 = zext i1 %36 to i32
  %38 = sext i32 %37 to i64
  %39 = icmp ne i64 %38, 0
  br i1 %39, label %40, label %44

40:                                               ; preds = %31
  %41 = load ptr, ptr %9, align 8
  %42 = load ptr, ptr %15, align 8
  %43 = load i32, ptr %14, align 4
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %41, ptr noundef @.str, ptr noundef %42, i32 noundef %43) #4
  unreachable

44:                                               ; preds = %31
  %45 = load i32, ptr %14, align 4
  store i32 %45, ptr %17, align 4
  br label %53

46:                                               ; preds = %26
  %47 = load i32, ptr %17, align 4
  %48 = mul nsw i32 %47, 2
  store i32 %48, ptr %17, align 4
  %49 = load i32, ptr %17, align 4
  %50 = icmp slt i32 %49, 4
  br i1 %50, label %51, label %52

51:                                               ; preds = %46
  store i32 4, ptr %17, align 4
  br label %52

52:                                               ; preds = %51, %46
  br label %53

53:                                               ; preds = %52, %44
  %54 = load ptr, ptr %9, align 8
  %55 = load ptr, ptr %10, align 8
  %56 = load ptr, ptr %12, align 8
  %57 = load i32, ptr %56, align 4
  %58 = sext i32 %57 to i64
  %59 = load i32, ptr %13, align 4
  %60 = sext i32 %59 to i64
  %61 = mul i64 %58, %60
  %62 = load i32, ptr %17, align 4
  %63 = sext i32 %62 to i64
  %64 = load i32, ptr %13, align 4
  %65 = sext i32 %64 to i64
  %66 = mul i64 %63, %65
  %67 = call ptr @luaM_saferealloc_(ptr noundef %54, ptr noundef %55, i64 noundef %61, i64 noundef %66)
  store ptr %67, ptr %16, align 8
  %68 = load i32, ptr %17, align 4
  %69 = load ptr, ptr %12, align 8
  store i32 %68, ptr %69, align 4
  %70 = load ptr, ptr %16, align 8
  store ptr %70, ptr %8, align 8
  br label %71

71:                                               ; preds = %53, %24
  %72 = load ptr, ptr %8, align 8
  ret ptr %72
}

; Function Attrs: noreturn
declare hidden void @luaG_runerror(ptr noundef, ptr noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaM_saferealloc_(ptr noundef %0, ptr noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  store i64 %3, ptr %8, align 8
  %10 = load ptr, ptr %5, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = load i64, ptr %7, align 8
  %13 = load i64, ptr %8, align 8
  %14 = call ptr @luaM_realloc_(ptr noundef %10, ptr noundef %11, i64 noundef %12, i64 noundef %13)
  store ptr %14, ptr %9, align 8
  %15 = load ptr, ptr %9, align 8
  %16 = icmp eq ptr %15, null
  br i1 %16, label %17, label %20

17:                                               ; preds = %4
  %18 = load i64, ptr %8, align 8
  %19 = icmp ugt i64 %18, 0
  br label %20

20:                                               ; preds = %17, %4
  %21 = phi i1 [ false, %4 ], [ %19, %17 ]
  %22 = zext i1 %21 to i32
  %23 = icmp ne i32 %22, 0
  %24 = zext i1 %23 to i32
  %25 = sext i32 %24 to i64
  %26 = icmp ne i64 %25, 0
  br i1 %26, label %27, label %29

27:                                               ; preds = %20
  %28 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %28, i32 noundef 4) #4
  unreachable

29:                                               ; preds = %20
  %30 = load ptr, ptr %9, align 8
  ret ptr %30
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaM_shrinkvector_(ptr noundef %0, ptr noundef %1, ptr noundef %2, i32 noundef %3, i32 noundef %4) #0 {
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  %11 = alloca ptr, align 8
  %12 = alloca i64, align 8
  %13 = alloca i64, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store ptr %2, ptr %8, align 8
  store i32 %3, ptr %9, align 4
  store i32 %4, ptr %10, align 4
  %14 = load ptr, ptr %8, align 8
  %15 = load i32, ptr %14, align 4
  %16 = load i32, ptr %10, align 4
  %17 = mul nsw i32 %15, %16
  %18 = sext i32 %17 to i64
  store i64 %18, ptr %12, align 8
  %19 = load i32, ptr %9, align 4
  %20 = load i32, ptr %10, align 4
  %21 = mul nsw i32 %19, %20
  %22 = sext i32 %21 to i64
  store i64 %22, ptr %13, align 8
  %23 = load ptr, ptr %6, align 8
  %24 = load ptr, ptr %7, align 8
  %25 = load i64, ptr %12, align 8
  %26 = load i64, ptr %13, align 8
  %27 = call ptr @luaM_saferealloc_(ptr noundef %23, ptr noundef %24, i64 noundef %25, i64 noundef %26)
  store ptr %27, ptr %11, align 8
  %28 = load i32, ptr %9, align 4
  %29 = load ptr, ptr %8, align 8
  store i32 %28, ptr %29, align 4
  %30 = load ptr, ptr %11, align 8
  ret ptr %30
}

; Function Attrs: noinline noreturn nounwind optnone uwtable
define hidden void @luaM_toobig(ptr noundef %0) #2 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8
  call void (ptr, ptr, ...) @luaG_runerror(ptr noundef %3, ptr noundef @.str.1) #4
  unreachable
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaM_free_(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  store ptr %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %8 = load ptr, ptr %4, align 8
  %9 = getelementptr inbounds %struct.lua_State, ptr %8, i32 0, i32 7
  %10 = load ptr, ptr %9, align 8
  store ptr %10, ptr %7, align 8
  %11 = load ptr, ptr %7, align 8
  %12 = getelementptr inbounds %struct.global_State, ptr %11, i32 0, i32 0
  %13 = load ptr, ptr %12, align 8
  %14 = load ptr, ptr %7, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 1
  %16 = load ptr, ptr %15, align 8
  %17 = load ptr, ptr %5, align 8
  %18 = load i64, ptr %6, align 8
  %19 = call ptr %13(ptr noundef %16, ptr noundef %17, i64 noundef %18, i64 noundef 0)
  %20 = load i64, ptr %6, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = getelementptr inbounds %struct.global_State, ptr %21, i32 0, i32 3
  %23 = load i64, ptr %22, align 8
  %24 = sub i64 %23, %20
  store i64 %24, ptr %22, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaM_realloc_(ptr noundef %0, ptr noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i64 %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %12 = load ptr, ptr %6, align 8
  %13 = getelementptr inbounds %struct.lua_State, ptr %12, i32 0, i32 7
  %14 = load ptr, ptr %13, align 8
  store ptr %14, ptr %11, align 8
  %15 = load ptr, ptr %11, align 8
  %16 = getelementptr inbounds %struct.global_State, ptr %15, i32 0, i32 0
  %17 = load ptr, ptr %16, align 8
  %18 = load ptr, ptr %11, align 8
  %19 = getelementptr inbounds %struct.global_State, ptr %18, i32 0, i32 1
  %20 = load ptr, ptr %19, align 8
  %21 = load ptr, ptr %7, align 8
  %22 = load i64, ptr %8, align 8
  %23 = load i64, ptr %9, align 8
  %24 = call ptr %17(ptr noundef %20, ptr noundef %21, i64 noundef %22, i64 noundef %23)
  store ptr %24, ptr %10, align 8
  %25 = load ptr, ptr %10, align 8
  %26 = icmp eq ptr %25, null
  br i1 %26, label %27, label %30

27:                                               ; preds = %4
  %28 = load i64, ptr %9, align 8
  %29 = icmp ugt i64 %28, 0
  br label %30

30:                                               ; preds = %27, %4
  %31 = phi i1 [ false, %4 ], [ %29, %27 ]
  %32 = zext i1 %31 to i32
  %33 = icmp ne i32 %32, 0
  %34 = zext i1 %33 to i32
  %35 = sext i32 %34 to i64
  %36 = icmp ne i64 %35, 0
  br i1 %36, label %37, label %47

37:                                               ; preds = %30
  %38 = load ptr, ptr %6, align 8
  %39 = load ptr, ptr %7, align 8
  %40 = load i64, ptr %8, align 8
  %41 = load i64, ptr %9, align 8
  %42 = call ptr @tryagain(ptr noundef %38, ptr noundef %39, i64 noundef %40, i64 noundef %41)
  store ptr %42, ptr %10, align 8
  %43 = load ptr, ptr %10, align 8
  %44 = icmp eq ptr %43, null
  br i1 %44, label %45, label %46

45:                                               ; preds = %37
  store ptr null, ptr %5, align 8
  br label %58

46:                                               ; preds = %37
  br label %47

47:                                               ; preds = %46, %30
  %48 = load ptr, ptr %11, align 8
  %49 = getelementptr inbounds %struct.global_State, ptr %48, i32 0, i32 3
  %50 = load i64, ptr %49, align 8
  %51 = load i64, ptr %9, align 8
  %52 = add i64 %50, %51
  %53 = load i64, ptr %8, align 8
  %54 = sub i64 %52, %53
  %55 = load ptr, ptr %11, align 8
  %56 = getelementptr inbounds %struct.global_State, ptr %55, i32 0, i32 3
  store i64 %54, ptr %56, align 8
  %57 = load ptr, ptr %10, align 8
  store ptr %57, ptr %5, align 8
  br label %58

58:                                               ; preds = %47, %45
  %59 = load ptr, ptr %5, align 8
  ret ptr %59
}

; Function Attrs: noinline nounwind optnone uwtable
define internal ptr @tryagain(ptr noundef %0, ptr noundef %1, i64 noundef %2, i64 noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca i64, align 8
  %9 = alloca i64, align 8
  %10 = alloca ptr, align 8
  store ptr %0, ptr %6, align 8
  store ptr %1, ptr %7, align 8
  store i64 %2, ptr %8, align 8
  store i64 %3, ptr %9, align 8
  %11 = load ptr, ptr %6, align 8
  %12 = getelementptr inbounds %struct.lua_State, ptr %11, i32 0, i32 7
  %13 = load ptr, ptr %12, align 8
  store ptr %13, ptr %10, align 8
  %14 = load ptr, ptr %10, align 8
  %15 = getelementptr inbounds %struct.global_State, ptr %14, i32 0, i32 8
  %16 = getelementptr inbounds %struct.TValue, ptr %15, i32 0, i32 1
  %17 = load i8, ptr %16, align 8
  %18 = zext i8 %17 to i32
  %19 = and i32 %18, 15
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %21, label %38

21:                                               ; preds = %4
  %22 = load ptr, ptr %10, align 8
  %23 = getelementptr inbounds %struct.global_State, ptr %22, i32 0, i32 13
  %24 = load i8, ptr %23, align 1
  %25 = icmp ne i8 %24, 0
  br i1 %25, label %38, label %26

26:                                               ; preds = %21
  %27 = load ptr, ptr %6, align 8
  call void @luaC_fullgc(ptr noundef %27, i32 noundef 1)
  %28 = load ptr, ptr %10, align 8
  %29 = getelementptr inbounds %struct.global_State, ptr %28, i32 0, i32 0
  %30 = load ptr, ptr %29, align 8
  %31 = load ptr, ptr %10, align 8
  %32 = getelementptr inbounds %struct.global_State, ptr %31, i32 0, i32 1
  %33 = load ptr, ptr %32, align 8
  %34 = load ptr, ptr %7, align 8
  %35 = load i64, ptr %8, align 8
  %36 = load i64, ptr %9, align 8
  %37 = call ptr %30(ptr noundef %33, ptr noundef %34, i64 noundef %35, i64 noundef %36)
  store ptr %37, ptr %5, align 8
  br label %39

38:                                               ; preds = %21, %4
  store ptr null, ptr %5, align 8
  br label %39

39:                                               ; preds = %38, %26
  %40 = load ptr, ptr %5, align 8
  ret ptr %40
}

; Function Attrs: noreturn
declare hidden void @luaD_throw(ptr noundef, i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define hidden ptr @luaM_malloc_(ptr noundef %0, i64 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store i64 %1, ptr %6, align 8
  store i32 %2, ptr %7, align 4
  %10 = load i64, ptr %6, align 8
  %11 = icmp eq i64 %10, 0
  br i1 %11, label %12, label %13

12:                                               ; preds = %3
  store ptr null, ptr %4, align 8
  br label %52

13:                                               ; preds = %3
  %14 = load ptr, ptr %5, align 8
  %15 = getelementptr inbounds %struct.lua_State, ptr %14, i32 0, i32 7
  %16 = load ptr, ptr %15, align 8
  store ptr %16, ptr %8, align 8
  %17 = load ptr, ptr %8, align 8
  %18 = getelementptr inbounds %struct.global_State, ptr %17, i32 0, i32 0
  %19 = load ptr, ptr %18, align 8
  %20 = load ptr, ptr %8, align 8
  %21 = getelementptr inbounds %struct.global_State, ptr %20, i32 0, i32 1
  %22 = load ptr, ptr %21, align 8
  %23 = load i32, ptr %7, align 4
  %24 = sext i32 %23 to i64
  %25 = load i64, ptr %6, align 8
  %26 = call ptr %19(ptr noundef %22, ptr noundef null, i64 noundef %24, i64 noundef %25)
  store ptr %26, ptr %9, align 8
  %27 = load ptr, ptr %9, align 8
  %28 = icmp eq ptr %27, null
  %29 = zext i1 %28 to i32
  %30 = icmp ne i32 %29, 0
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %34, label %45

34:                                               ; preds = %13
  %35 = load ptr, ptr %5, align 8
  %36 = load i32, ptr %7, align 4
  %37 = sext i32 %36 to i64
  %38 = load i64, ptr %6, align 8
  %39 = call ptr @tryagain(ptr noundef %35, ptr noundef null, i64 noundef %37, i64 noundef %38)
  store ptr %39, ptr %9, align 8
  %40 = load ptr, ptr %9, align 8
  %41 = icmp eq ptr %40, null
  br i1 %41, label %42, label %44

42:                                               ; preds = %34
  %43 = load ptr, ptr %5, align 8
  call void @luaD_throw(ptr noundef %43, i32 noundef 4) #4
  unreachable

44:                                               ; preds = %34
  br label %45

45:                                               ; preds = %44, %13
  %46 = load i64, ptr %6, align 8
  %47 = load ptr, ptr %8, align 8
  %48 = getelementptr inbounds %struct.global_State, ptr %47, i32 0, i32 3
  %49 = load i64, ptr %48, align 8
  %50 = add i64 %49, %46
  store i64 %50, ptr %48, align 8
  %51 = load ptr, ptr %9, align 8
  store ptr %51, ptr %4, align 8
  br label %52

52:                                               ; preds = %45, %12
  %53 = load ptr, ptr %4, align 8
  ret ptr %53
}

declare hidden void @luaC_fullgc(ptr noundef, i32 noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { noinline noreturn nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { noreturn }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
