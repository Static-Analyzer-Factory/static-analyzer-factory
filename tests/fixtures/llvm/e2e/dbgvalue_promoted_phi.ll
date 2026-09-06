; ModuleID = '/tmp/dp.pre.ll'
source_filename = "tests/programs/c/dbgvalue_promoted_phi.c"
target datalayout = "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128"
target triple = "i386-pc-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !11 {
  tail call void @llvm.dbg.value(metadata i32 0, metadata !16, metadata !DIExpression()), !dbg !17
  tail call void @llvm.dbg.value(metadata i32 0, metadata !18, metadata !DIExpression()), !dbg !20
  br label %1, !dbg !21

1:                                                ; preds = %5, %0
  %.01 = phi i32 [ 0, %0 ], [ %4, %5 ], !dbg !17
  %.0 = phi i32 [ 0, %0 ], [ %6, %5 ], !dbg !22
  tail call void @llvm.dbg.value(metadata i32 %.0, metadata !18, metadata !DIExpression()), !dbg !20
  tail call void @llvm.dbg.value(metadata i32 %.01, metadata !16, metadata !DIExpression()), !dbg !17
  %2 = icmp slt i32 %.0, 100, !dbg !23
  br i1 %2, label %3, label %7, !dbg !25

3:                                                ; preds = %1
  %4 = add nsw i32 %.01, %.0, !dbg !26
  tail call void @llvm.dbg.value(metadata i32 %4, metadata !16, metadata !DIExpression()), !dbg !17
  br label %5, !dbg !28

5:                                                ; preds = %3
  %6 = add nsw i32 %.0, 1, !dbg !29
  tail call void @llvm.dbg.value(metadata i32 %6, metadata !18, metadata !DIExpression()), !dbg !20
  br label %1, !dbg !30, !llvm.loop !31

7:                                                ; preds = %1
  ret i32 %.01, !dbg !34
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="i686" "target-features"="+cmov,+cx8,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8, !9}
!llvm.ident = !{!10}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/dbgvalue_promoted_phi.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "7b5b6f239a54220008d22113473bbbbd")
!2 = !{i32 1, !"NumRegisterParameters", i32 0}
!3 = !{i32 7, !"Dwarf Version", i32 5}
!4 = !{i32 2, !"Debug Info Version", i32 3}
!5 = !{i32 1, !"wchar_size", i32 4}
!6 = !{i32 8, !"PIC Level", i32 2}
!7 = !{i32 7, !"PIE Level", i32 2}
!8 = !{i32 7, !"uwtable", i32 2}
!9 = !{i32 7, !"frame-pointer", i32 2}
!10 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!11 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 12, type: !12, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!12 = !DISubroutineType(types: !13)
!13 = !{!14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "s", scope: !11, file: !1, line: 13, type: !14)
!17 = !DILocation(line: 0, scope: !11)
!18 = !DILocalVariable(name: "i", scope: !19, file: !1, line: 14, type: !14)
!19 = distinct !DILexicalBlock(scope: !11, file: !1, line: 14, column: 5)
!20 = !DILocation(line: 0, scope: !19)
!21 = !DILocation(line: 14, column: 10, scope: !19)
!22 = !DILocation(line: 14, scope: !19)
!23 = !DILocation(line: 14, column: 23, scope: !24)
!24 = distinct !DILexicalBlock(scope: !19, file: !1, line: 14, column: 5)
!25 = !DILocation(line: 14, column: 5, scope: !19)
!26 = !DILocation(line: 15, column: 11, scope: !27)
!27 = distinct !DILexicalBlock(scope: !24, file: !1, line: 14, column: 35)
!28 = !DILocation(line: 16, column: 5, scope: !27)
!29 = !DILocation(line: 14, column: 31, scope: !24)
!30 = !DILocation(line: 14, column: 5, scope: !24)
!31 = distinct !{!31, !25, !32, !33}
!32 = !DILocation(line: 16, column: 5, scope: !19)
!33 = !{!"llvm.loop.mustprogress"}
!34 = !DILocation(line: 17, column: 5, scope: !11)
