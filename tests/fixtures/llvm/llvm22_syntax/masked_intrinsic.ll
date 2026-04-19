; ModuleID = '/workspace/tests/programs/c/llvm22_syntax/masked_intrinsic.c'
source_filename = "/workspace/tests/programs/c/llvm22_syntax/masked_intrinsic.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: readwrite) uwtable
define dso_local void @conditional_store(ptr noalias noundef writeonly captures(none) %0, ptr noalias noundef readonly captures(none) %1, i64 noundef %2) local_unnamed_addr #0 {
  %4 = icmp eq i64 %2, 0
  br i1 %4, label %51, label %5

5:                                                ; preds = %3
  %6 = icmp ult i64 %2, 8
  br i1 %6, label %49, label %7

7:                                                ; preds = %5
  %8 = icmp ult i64 %2, 32
  br i1 %8, label %36, label %9

9:                                                ; preds = %7
  %10 = and i64 %2, 24
  %11 = and i64 %2, -32
  br label %12

12:                                               ; preds = %12, %9
  %13 = phi i64 [ 0, %9 ], [ %30, %12 ]
  %14 = getelementptr inbounds nuw i32, ptr %1, i64 %13
  %15 = getelementptr inbounds nuw i8, ptr %14, i64 32
  %16 = getelementptr inbounds nuw i8, ptr %14, i64 64
  %17 = getelementptr inbounds nuw i8, ptr %14, i64 96
  %18 = load <8 x i32>, ptr %14, align 4, !tbaa !5
  %19 = load <8 x i32>, ptr %15, align 4, !tbaa !5
  %20 = load <8 x i32>, ptr %16, align 4, !tbaa !5
  %21 = load <8 x i32>, ptr %17, align 4, !tbaa !5
  %22 = icmp ne <8 x i32> %18, zeroinitializer
  %23 = icmp ne <8 x i32> %19, zeroinitializer
  %24 = icmp ne <8 x i32> %20, zeroinitializer
  %25 = icmp ne <8 x i32> %21, zeroinitializer
  %26 = getelementptr float, ptr %0, i64 %13
  %27 = getelementptr i8, ptr %26, i64 32
  %28 = getelementptr i8, ptr %26, i64 64
  %29 = getelementptr i8, ptr %26, i64 96
  tail call void @llvm.masked.store.v8f32.p0(<8 x float> splat (float 1.000000e+00), ptr align 4 %26, <8 x i1> %22), !tbaa !9
  tail call void @llvm.masked.store.v8f32.p0(<8 x float> splat (float 1.000000e+00), ptr align 4 %27, <8 x i1> %23), !tbaa !9
  tail call void @llvm.masked.store.v8f32.p0(<8 x float> splat (float 1.000000e+00), ptr align 4 %28, <8 x i1> %24), !tbaa !9
  tail call void @llvm.masked.store.v8f32.p0(<8 x float> splat (float 1.000000e+00), ptr align 4 %29, <8 x i1> %25), !tbaa !9
  %30 = add nuw i64 %13, 32
  %31 = icmp eq i64 %30, %11
  br i1 %31, label %32, label %12, !llvm.loop !11

32:                                               ; preds = %12
  %33 = icmp eq i64 %2, %11
  br i1 %33, label %51, label %34

34:                                               ; preds = %32
  %35 = icmp eq i64 %10, 0
  br i1 %35, label %49, label %36, !prof !15

36:                                               ; preds = %7, %34
  %37 = phi i64 [ %11, %34 ], [ 0, %7 ]
  %38 = and i64 %2, -8
  br label %39

39:                                               ; preds = %39, %36
  %40 = phi i64 [ %37, %36 ], [ %45, %39 ]
  %41 = getelementptr inbounds nuw i32, ptr %1, i64 %40
  %42 = load <8 x i32>, ptr %41, align 4, !tbaa !5
  %43 = icmp ne <8 x i32> %42, zeroinitializer
  %44 = getelementptr float, ptr %0, i64 %40
  tail call void @llvm.masked.store.v8f32.p0(<8 x float> splat (float 1.000000e+00), ptr align 4 %44, <8 x i1> %43), !tbaa !9
  %45 = add nuw i64 %40, 8
  %46 = icmp eq i64 %45, %38
  br i1 %46, label %47, label %39, !llvm.loop !16

47:                                               ; preds = %39
  %48 = icmp eq i64 %2, %38
  br i1 %48, label %51, label %49

49:                                               ; preds = %5, %34, %47
  %50 = phi i64 [ 0, %5 ], [ %11, %34 ], [ %38, %47 ]
  br label %52

51:                                               ; preds = %59, %32, %47, %3
  ret void

52:                                               ; preds = %49, %59
  %53 = phi i64 [ %60, %59 ], [ %50, %49 ]
  %54 = getelementptr inbounds nuw i32, ptr %1, i64 %53
  %55 = load i32, ptr %54, align 4, !tbaa !5
  %56 = icmp eq i32 %55, 0
  br i1 %56, label %59, label %57

57:                                               ; preds = %52
  %58 = getelementptr inbounds nuw float, ptr %0, i64 %53
  store float 1.000000e+00, ptr %58, align 4, !tbaa !9
  br label %59

59:                                               ; preds = %52, %57
  %60 = add nuw i64 %53, 1
  %61 = icmp eq i64 %60, %2
  br i1 %61, label %51, label %52, !llvm.loop !17
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: write)
declare void @llvm.masked.store.v8f32.p0(<8 x float>, ptr captures(none), <8 x i1>) #1

attributes #0 = { nofree norecurse nosync nounwind memory(argmem: readwrite) uwtable "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+avx,+avx2,+cmov,+crc32,+cx8,+fxsr,+mmx,+popcnt,+sse,+sse2,+sse3,+sse4.1,+sse4.2,+ssse3,+x87,+xsave" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(argmem: write) }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}
!llvm.errno.tbaa = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{!"Ubuntu clang version 22.1.3 (++20260402073256+4250a0fc5de9-1~exp1~20260402073413.57)"}
!5 = !{!6, !6, i64 0}
!6 = !{!"int", !7, i64 0}
!7 = !{!"omnipotent char", !8, i64 0}
!8 = !{!"Simple C/C++ TBAA"}
!9 = !{!10, !10, i64 0}
!10 = !{!"float", !7, i64 0}
!11 = distinct !{!11, !12, !13, !14}
!12 = !{!"llvm.loop.mustprogress"}
!13 = !{!"llvm.loop.isvectorized", i32 1}
!14 = !{!"llvm.loop.unroll.runtime.disable"}
!15 = !{!"branch_weights", i32 8, i32 24}
!16 = distinct !{!16, !12, !13, !14}
!17 = distinct !{!17, !12, !14, !13}
