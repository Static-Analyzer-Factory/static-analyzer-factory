; ModuleID = '/tmp/sc.pre.ll'
source_filename = "tests/programs/c/spike_counter.c"
target datalayout = "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128"
target triple = "i386-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"0\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [16 x i8] c"spike_counter.c\00", align 1, !dbg !7
@.str.2 = private unnamed_addr constant [12 x i8] c"reach_error\00", align 1, !dbg !12

; Function Attrs: noinline nounwind uwtable
define dso_local void @reach_error() #0 !dbg !28 {
  call void @__assert_fail(ptr noundef @.str, ptr noundef @.str.1, i32 noundef 3, ptr noundef @.str.2) #4, !dbg !31
  unreachable, !dbg !31
}

; Function Attrs: nocallback noreturn nounwind
declare void @__assert_fail(ptr noundef, ptr noundef, i32 noundef, ptr noundef) #1

; Function Attrs: noinline nounwind uwtable
define dso_local void @__VERIFIER_assert(i32 noundef %0) #0 !dbg !32 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !37, metadata !DIExpression()), !dbg !38
  %2 = icmp ne i32 %0, 0, !dbg !39
  br i1 %2, label %5, label %3, !dbg !41

3:                                                ; preds = %1
  br label %4, !dbg !42

4:                                                ; preds = %3
  call void @llvm.dbg.label(metadata !43), !dbg !45
  call void @reach_error(), !dbg !46
  call void @abort() #5, !dbg !48
  unreachable, !dbg !48

5:                                                ; preds = %1
  ret void, !dbg !49
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #2

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.label(metadata) #2

; Function Attrs: noreturn
declare void @abort() #3

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !50 {
  tail call void @llvm.dbg.value(metadata i32 0, metadata !53, metadata !DIExpression()), !dbg !55
  br label %1, !dbg !56

1:                                                ; preds = %3, %0
  %.0 = phi i32 [ 0, %0 ], [ %4, %3 ], !dbg !55
  tail call void @llvm.dbg.value(metadata i32 %.0, metadata !53, metadata !DIExpression()), !dbg !55
  %2 = icmp ult i32 %.0, 1000000, !dbg !57
  br i1 %2, label %3, label %5, !dbg !56

3:                                                ; preds = %1
  %4 = add i32 %.0, 1, !dbg !58
  tail call void @llvm.dbg.value(metadata i32 %4, metadata !53, metadata !DIExpression()), !dbg !55
  br label %1, !dbg !56, !llvm.loop !60

5:                                                ; preds = %1
  %6 = icmp eq i32 %.0, 1000000, !dbg !63
  %7 = zext i1 %6 to i32, !dbg !63
  call void @__VERIFIER_assert(i32 noundef %7), !dbg !64
  ret i32 0, !dbg !65
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #2

attributes #0 = { noinline nounwind uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="i686" "target-features"="+cmov,+cx8,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback noreturn nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="i686" "target-features"="+cmov,+cx8,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #3 = { noreturn "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="i686" "target-features"="+cmov,+cx8,+x87" "tune-cpu"="generic" }
attributes #4 = { nocallback noreturn nounwind }
attributes #5 = { noreturn }

!llvm.dbg.cu = !{!17}
!llvm.module.flags = !{!19, !20, !21, !22, !23, !24, !25, !26}
!llvm.ident = !{!27}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/spike_counter.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "aa5abcf1e4492177a21810ba698ee98d")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 16, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_signed_char)
!5 = !{!6}
!6 = !DISubrange(count: 2)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 128, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 16)
!12 = !DIGlobalVariableExpression(var: !13, expr: !DIExpression())
!13 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !14, isLocal: true, isDefinition: true)
!14 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 96, elements: !15)
!15 = !{!16}
!16 = !DISubrange(count: 12)
!17 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !18, splitDebugInlining: false, nameTableKind: None)
!18 = !{!0, !7, !12}
!19 = !{i32 1, !"NumRegisterParameters", i32 0}
!20 = !{i32 7, !"Dwarf Version", i32 5}
!21 = !{i32 2, !"Debug Info Version", i32 3}
!22 = !{i32 1, !"wchar_size", i32 4}
!23 = !{i32 8, !"PIC Level", i32 2}
!24 = !{i32 7, !"PIE Level", i32 2}
!25 = !{i32 7, !"uwtable", i32 2}
!26 = !{i32 7, !"frame-pointer", i32 2}
!27 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!28 = distinct !DISubprogram(name: "reach_error", scope: !2, file: !2, line: 13, type: !29, scopeLine: 13, spFlags: DISPFlagDefinition, unit: !17)
!29 = !DISubroutineType(types: !30)
!30 = !{null}
!31 = !DILocation(line: 13, column: 22, scope: !28)
!32 = distinct !DISubprogram(name: "__VERIFIER_assert", scope: !2, file: !2, line: 14, type: !33, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !17, retainedNodes: !36)
!33 = !DISubroutineType(types: !34)
!34 = !{null, !35}
!35 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!36 = !{}
!37 = !DILocalVariable(name: "cond", arg: 1, scope: !32, file: !2, line: 14, type: !35)
!38 = !DILocation(line: 0, scope: !32)
!39 = !DILocation(line: 15, column: 10, scope: !40)
!40 = distinct !DILexicalBlock(scope: !32, file: !2, line: 15, column: 9)
!41 = !DILocation(line: 15, column: 9, scope: !32)
!42 = !DILocation(line: 15, column: 18, scope: !40)
!43 = !DILabel(scope: !44, name: "ERROR", file: !2, line: 16)
!44 = distinct !DILexicalBlock(scope: !40, file: !2, line: 15, column: 18)
!45 = !DILocation(line: 16, column: 5, scope: !44)
!46 = !DILocation(line: 18, column: 13, scope: !47)
!47 = distinct !DILexicalBlock(scope: !44, file: !2, line: 17, column: 9)
!48 = !DILocation(line: 19, column: 13, scope: !47)
!49 = !DILocation(line: 22, column: 5, scope: !32)
!50 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 24, type: !51, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !17, retainedNodes: !36)
!51 = !DISubroutineType(types: !52)
!52 = !{!35}
!53 = !DILocalVariable(name: "i", scope: !50, file: !2, line: 25, type: !54)
!54 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!55 = !DILocation(line: 0, scope: !50)
!56 = !DILocation(line: 26, column: 5, scope: !50)
!57 = !DILocation(line: 26, column: 14, scope: !50)
!58 = !DILocation(line: 27, column: 10, scope: !59)
!59 = distinct !DILexicalBlock(scope: !50, file: !2, line: 26, column: 25)
!60 = distinct !{!60, !56, !61, !62}
!61 = !DILocation(line: 28, column: 5, scope: !50)
!62 = !{!"llvm.loop.mustprogress"}
!63 = !DILocation(line: 29, column: 25, scope: !50)
!64 = !DILocation(line: 29, column: 5, scope: !50)
!65 = !DILocation(line: 30, column: 5, scope: !50)
