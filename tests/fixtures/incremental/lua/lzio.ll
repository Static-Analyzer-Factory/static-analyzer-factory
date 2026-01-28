; ModuleID = 'lzio.c'
source_filename = "lzio.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.Zio = type { i64, ptr, ptr, ptr, ptr }

; Function Attrs: noinline nounwind optnone uwtable
define hidden i32 @luaZ_fill(ptr noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  %7 = load ptr, ptr %3, align 8
  %8 = getelementptr inbounds %struct.Zio, ptr %7, i32 0, i32 4
  %9 = load ptr, ptr %8, align 8
  store ptr %9, ptr %5, align 8
  %10 = load ptr, ptr %3, align 8
  %11 = getelementptr inbounds %struct.Zio, ptr %10, i32 0, i32 2
  %12 = load ptr, ptr %11, align 8
  %13 = load ptr, ptr %5, align 8
  %14 = load ptr, ptr %3, align 8
  %15 = getelementptr inbounds %struct.Zio, ptr %14, i32 0, i32 3
  %16 = load ptr, ptr %15, align 8
  %17 = call ptr %12(ptr noundef %13, ptr noundef %16, ptr noundef %4)
  store ptr %17, ptr %6, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = icmp eq ptr %18, null
  br i1 %19, label %23, label %20

20:                                               ; preds = %1
  %21 = load i64, ptr %4, align 8
  %22 = icmp eq i64 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %20, %1
  store i32 -1, ptr %2, align 4
  br label %38

24:                                               ; preds = %20
  %25 = load i64, ptr %4, align 8
  %26 = sub i64 %25, 1
  %27 = load ptr, ptr %3, align 8
  %28 = getelementptr inbounds %struct.Zio, ptr %27, i32 0, i32 0
  store i64 %26, ptr %28, align 8
  %29 = load ptr, ptr %6, align 8
  %30 = load ptr, ptr %3, align 8
  %31 = getelementptr inbounds %struct.Zio, ptr %30, i32 0, i32 1
  store ptr %29, ptr %31, align 8
  %32 = load ptr, ptr %3, align 8
  %33 = getelementptr inbounds %struct.Zio, ptr %32, i32 0, i32 1
  %34 = load ptr, ptr %33, align 8
  %35 = getelementptr inbounds i8, ptr %34, i32 1
  store ptr %35, ptr %33, align 8
  %36 = load i8, ptr %34, align 1
  %37 = zext i8 %36 to i32
  store i32 %37, ptr %2, align 4
  br label %38

38:                                               ; preds = %24, %23
  %39 = load i32, ptr %2, align 4
  ret i32 %39
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden void @luaZ_init(ptr noundef %0, ptr noundef %1, ptr noundef %2, ptr noundef %3) #0 {
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store ptr %2, ptr %7, align 8
  store ptr %3, ptr %8, align 8
  %9 = load ptr, ptr %5, align 8
  %10 = load ptr, ptr %6, align 8
  %11 = getelementptr inbounds %struct.Zio, ptr %10, i32 0, i32 4
  store ptr %9, ptr %11, align 8
  %12 = load ptr, ptr %7, align 8
  %13 = load ptr, ptr %6, align 8
  %14 = getelementptr inbounds %struct.Zio, ptr %13, i32 0, i32 2
  store ptr %12, ptr %14, align 8
  %15 = load ptr, ptr %8, align 8
  %16 = load ptr, ptr %6, align 8
  %17 = getelementptr inbounds %struct.Zio, ptr %16, i32 0, i32 3
  store ptr %15, ptr %17, align 8
  %18 = load ptr, ptr %6, align 8
  %19 = getelementptr inbounds %struct.Zio, ptr %18, i32 0, i32 0
  store i64 0, ptr %19, align 8
  %20 = load ptr, ptr %6, align 8
  %21 = getelementptr inbounds %struct.Zio, ptr %20, i32 0, i32 1
  store ptr null, ptr %21, align 8
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define hidden i64 @luaZ_read(ptr noundef %0, ptr noundef %1, i64 noundef %2) #0 {
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca i64, align 8
  %8 = alloca i64, align 8
  store ptr %0, ptr %5, align 8
  store ptr %1, ptr %6, align 8
  store i64 %2, ptr %7, align 8
  br label %9

9:                                                ; preds = %45, %3
  %10 = load i64, ptr %7, align 8
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %12, label %68

12:                                               ; preds = %9
  %13 = load ptr, ptr %5, align 8
  %14 = getelementptr inbounds %struct.Zio, ptr %13, i32 0, i32 0
  %15 = load i64, ptr %14, align 8
  %16 = icmp eq i64 %15, 0
  br i1 %16, label %17, label %33

17:                                               ; preds = %12
  %18 = load ptr, ptr %5, align 8
  %19 = call i32 @luaZ_fill(ptr noundef %18)
  %20 = icmp eq i32 %19, -1
  br i1 %20, label %21, label %23

21:                                               ; preds = %17
  %22 = load i64, ptr %7, align 8
  store i64 %22, ptr %4, align 8
  br label %69

23:                                               ; preds = %17
  %24 = load ptr, ptr %5, align 8
  %25 = getelementptr inbounds %struct.Zio, ptr %24, i32 0, i32 0
  %26 = load i64, ptr %25, align 8
  %27 = add i64 %26, 1
  store i64 %27, ptr %25, align 8
  %28 = load ptr, ptr %5, align 8
  %29 = getelementptr inbounds %struct.Zio, ptr %28, i32 0, i32 1
  %30 = load ptr, ptr %29, align 8
  %31 = getelementptr inbounds i8, ptr %30, i32 -1
  store ptr %31, ptr %29, align 8
  br label %32

32:                                               ; preds = %23
  br label %33

33:                                               ; preds = %32, %12
  %34 = load i64, ptr %7, align 8
  %35 = load ptr, ptr %5, align 8
  %36 = getelementptr inbounds %struct.Zio, ptr %35, i32 0, i32 0
  %37 = load i64, ptr %36, align 8
  %38 = icmp ule i64 %34, %37
  br i1 %38, label %39, label %41

39:                                               ; preds = %33
  %40 = load i64, ptr %7, align 8
  br label %45

41:                                               ; preds = %33
  %42 = load ptr, ptr %5, align 8
  %43 = getelementptr inbounds %struct.Zio, ptr %42, i32 0, i32 0
  %44 = load i64, ptr %43, align 8
  br label %45

45:                                               ; preds = %41, %39
  %46 = phi i64 [ %40, %39 ], [ %44, %41 ]
  store i64 %46, ptr %8, align 8
  %47 = load ptr, ptr %6, align 8
  %48 = load ptr, ptr %5, align 8
  %49 = getelementptr inbounds %struct.Zio, ptr %48, i32 0, i32 1
  %50 = load ptr, ptr %49, align 8
  %51 = load i64, ptr %8, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %47, ptr align 1 %50, i64 %51, i1 false)
  %52 = load i64, ptr %8, align 8
  %53 = load ptr, ptr %5, align 8
  %54 = getelementptr inbounds %struct.Zio, ptr %53, i32 0, i32 0
  %55 = load i64, ptr %54, align 8
  %56 = sub i64 %55, %52
  store i64 %56, ptr %54, align 8
  %57 = load i64, ptr %8, align 8
  %58 = load ptr, ptr %5, align 8
  %59 = getelementptr inbounds %struct.Zio, ptr %58, i32 0, i32 1
  %60 = load ptr, ptr %59, align 8
  %61 = getelementptr inbounds i8, ptr %60, i64 %57
  store ptr %61, ptr %59, align 8
  %62 = load ptr, ptr %6, align 8
  %63 = load i64, ptr %8, align 8
  %64 = getelementptr inbounds i8, ptr %62, i64 %63
  store ptr %64, ptr %6, align 8
  %65 = load i64, ptr %8, align 8
  %66 = load i64, ptr %7, align 8
  %67 = sub i64 %66, %65
  store i64 %67, ptr %7, align 8
  br label %9, !llvm.loop !6

68:                                               ; preds = %9
  store i64 0, ptr %4, align 8
  br label %69

69:                                               ; preds = %68, %21
  %70 = load i64, ptr %4, align 8
  ret i64 %70
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }

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
