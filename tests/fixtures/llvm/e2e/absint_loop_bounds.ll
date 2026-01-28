; ModuleID = '/workspace/tests/programs/c/absint_loop_bounds.c'
source_filename = "/workspace/tests/programs/c/absint_loop_bounds.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @init_array(ptr noundef %0, i32 noundef %1) #0 !dbg !10 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  call void @llvm.dbg.declare(metadata ptr %5, metadata !21, metadata !DIExpression()), !dbg !23
  store i32 0, ptr %5, align 4, !dbg !23
  br label %6, !dbg !24

6:                                                ; preds = %15, %2
  %7 = load i32, ptr %5, align 4, !dbg !25
  %8 = load i32, ptr %4, align 4, !dbg !27
  %9 = icmp slt i32 %7, %8, !dbg !28
  br i1 %9, label %10, label %18, !dbg !29

10:                                               ; preds = %6
  %11 = load ptr, ptr %3, align 8, !dbg !30
  %12 = load i32, ptr %5, align 4, !dbg !32
  %13 = sext i32 %12 to i64, !dbg !30
  %14 = getelementptr inbounds i32, ptr %11, i64 %13, !dbg !30
  store i32 0, ptr %14, align 4, !dbg !33
  br label %15, !dbg !34

15:                                               ; preds = %10
  %16 = load i32, ptr %5, align 4, !dbg !35
  %17 = add nsw i32 %16, 1, !dbg !35
  store i32 %17, ptr %5, align 4, !dbg !35
  br label %6, !dbg !36, !llvm.loop !37

18:                                               ; preds = %6
  ret void, !dbg !40
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @sum_array(ptr noundef %0, i32 noundef %1) #0 !dbg !41 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !44, metadata !DIExpression()), !dbg !45
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !46, metadata !DIExpression()), !dbg !47
  call void @llvm.dbg.declare(metadata ptr %5, metadata !48, metadata !DIExpression()), !dbg !49
  store i32 0, ptr %5, align 4, !dbg !49
  call void @llvm.dbg.declare(metadata ptr %6, metadata !50, metadata !DIExpression()), !dbg !52
  store i32 0, ptr %6, align 4, !dbg !52
  br label %7, !dbg !53

7:                                                ; preds = %19, %2
  %8 = load i32, ptr %6, align 4, !dbg !54
  %9 = load i32, ptr %4, align 4, !dbg !56
  %10 = icmp slt i32 %8, %9, !dbg !57
  br i1 %10, label %11, label %22, !dbg !58

11:                                               ; preds = %7
  %12 = load i32, ptr %5, align 4, !dbg !59
  %13 = load ptr, ptr %3, align 8, !dbg !61
  %14 = load i32, ptr %6, align 4, !dbg !62
  %15 = sext i32 %14 to i64, !dbg !61
  %16 = getelementptr inbounds i32, ptr %13, i64 %15, !dbg !61
  %17 = load i32, ptr %16, align 4, !dbg !61
  %18 = add nsw i32 %12, %17, !dbg !63
  store i32 %18, ptr %5, align 4, !dbg !64
  br label %19, !dbg !65

19:                                               ; preds = %11
  %20 = load i32, ptr %6, align 4, !dbg !66
  %21 = add nsw i32 %20, 1, !dbg !66
  store i32 %21, ptr %6, align 4, !dbg !66
  br label %7, !dbg !67, !llvm.loop !68

22:                                               ; preds = %7
  %23 = load i32, ptr %5, align 4, !dbg !70
  ret i32 %23, !dbg !71
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @count_positive(ptr noundef %0, i32 noundef %1) #0 !dbg !72 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !73, metadata !DIExpression()), !dbg !74
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !75, metadata !DIExpression()), !dbg !76
  call void @llvm.dbg.declare(metadata ptr %5, metadata !77, metadata !DIExpression()), !dbg !78
  store i32 0, ptr %5, align 4, !dbg !78
  call void @llvm.dbg.declare(metadata ptr %6, metadata !79, metadata !DIExpression()), !dbg !81
  store i32 0, ptr %6, align 4, !dbg !81
  br label %7, !dbg !82

7:                                                ; preds = %22, %2
  %8 = load i32, ptr %6, align 4, !dbg !83
  %9 = load i32, ptr %4, align 4, !dbg !85
  %10 = icmp slt i32 %8, %9, !dbg !86
  br i1 %10, label %11, label %25, !dbg !87

11:                                               ; preds = %7
  %12 = load ptr, ptr %3, align 8, !dbg !88
  %13 = load i32, ptr %6, align 4, !dbg !91
  %14 = sext i32 %13 to i64, !dbg !88
  %15 = getelementptr inbounds i32, ptr %12, i64 %14, !dbg !88
  %16 = load i32, ptr %15, align 4, !dbg !88
  %17 = icmp sgt i32 %16, 0, !dbg !92
  br i1 %17, label %18, label %21, !dbg !93

18:                                               ; preds = %11
  %19 = load i32, ptr %5, align 4, !dbg !94
  %20 = add nsw i32 %19, 1, !dbg !96
  store i32 %20, ptr %5, align 4, !dbg !97
  br label %21, !dbg !98

21:                                               ; preds = %18, %11
  br label %22, !dbg !99

22:                                               ; preds = %21
  %23 = load i32, ptr %6, align 4, !dbg !100
  %24 = add nsw i32 %23, 1, !dbg !100
  store i32 %24, ptr %6, align 4, !dbg !100
  br label %7, !dbg !101, !llvm.loop !102

25:                                               ; preds = %7
  %26 = load i32, ptr %5, align 4, !dbg !104
  ret i32 %26, !dbg !105
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !106 {
  %1 = alloca i32, align 4
  %2 = alloca [100 x i32], align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !109, metadata !DIExpression()), !dbg !113
  %5 = getelementptr inbounds [100 x i32], ptr %2, i64 0, i64 0, !dbg !114
  call void @init_array(ptr noundef %5, i32 noundef 100), !dbg !115
  call void @llvm.dbg.declare(metadata ptr %3, metadata !116, metadata !DIExpression()), !dbg !117
  %6 = getelementptr inbounds [100 x i32], ptr %2, i64 0, i64 0, !dbg !118
  %7 = call i32 @sum_array(ptr noundef %6, i32 noundef 100), !dbg !119
  store i32 %7, ptr %3, align 4, !dbg !117
  call void @llvm.dbg.declare(metadata ptr %4, metadata !120, metadata !DIExpression()), !dbg !121
  %8 = getelementptr inbounds [100 x i32], ptr %2, i64 0, i64 0, !dbg !122
  %9 = call i32 @count_positive(ptr noundef %8, i32 noundef 100), !dbg !123
  store i32 %9, ptr %4, align 4, !dbg !121
  %10 = load i32, ptr %3, align 4, !dbg !124
  %11 = load i32, ptr %4, align 4, !dbg !125
  %12 = add nsw i32 %10, %11, !dbg !126
  ret i32 %12, !dbg !127
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/absint_loop_bounds.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3a8b0b47f93f014a0b4c77514483913e")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "init_array", scope: !11, file: !11, line: 9, type: !12, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!11 = !DIFile(filename: "c/absint_loop_bounds.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3a8b0b47f93f014a0b4c77514483913e")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14, !15}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!16 = !{}
!17 = !DILocalVariable(name: "arr", arg: 1, scope: !10, file: !11, line: 9, type: !14)
!18 = !DILocation(line: 9, column: 22, scope: !10)
!19 = !DILocalVariable(name: "size", arg: 2, scope: !10, file: !11, line: 9, type: !15)
!20 = !DILocation(line: 9, column: 31, scope: !10)
!21 = !DILocalVariable(name: "i", scope: !22, file: !11, line: 11, type: !15)
!22 = distinct !DILexicalBlock(scope: !10, file: !11, line: 11, column: 5)
!23 = !DILocation(line: 11, column: 14, scope: !22)
!24 = !DILocation(line: 11, column: 10, scope: !22)
!25 = !DILocation(line: 11, column: 21, scope: !26)
!26 = distinct !DILexicalBlock(scope: !22, file: !11, line: 11, column: 5)
!27 = !DILocation(line: 11, column: 25, scope: !26)
!28 = !DILocation(line: 11, column: 23, scope: !26)
!29 = !DILocation(line: 11, column: 5, scope: !22)
!30 = !DILocation(line: 12, column: 9, scope: !31)
!31 = distinct !DILexicalBlock(scope: !26, file: !11, line: 11, column: 36)
!32 = !DILocation(line: 12, column: 13, scope: !31)
!33 = !DILocation(line: 12, column: 16, scope: !31)
!34 = !DILocation(line: 13, column: 5, scope: !31)
!35 = !DILocation(line: 11, column: 32, scope: !26)
!36 = !DILocation(line: 11, column: 5, scope: !26)
!37 = distinct !{!37, !29, !38, !39}
!38 = !DILocation(line: 13, column: 5, scope: !22)
!39 = !{!"llvm.loop.mustprogress"}
!40 = !DILocation(line: 14, column: 1, scope: !10)
!41 = distinct !DISubprogram(name: "sum_array", scope: !11, file: !11, line: 16, type: !42, scopeLine: 16, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!42 = !DISubroutineType(types: !43)
!43 = !{!15, !14, !15}
!44 = !DILocalVariable(name: "arr", arg: 1, scope: !41, file: !11, line: 16, type: !14)
!45 = !DILocation(line: 16, column: 20, scope: !41)
!46 = !DILocalVariable(name: "size", arg: 2, scope: !41, file: !11, line: 16, type: !15)
!47 = !DILocation(line: 16, column: 29, scope: !41)
!48 = !DILocalVariable(name: "total", scope: !41, file: !11, line: 17, type: !15)
!49 = !DILocation(line: 17, column: 9, scope: !41)
!50 = !DILocalVariable(name: "i", scope: !51, file: !11, line: 18, type: !15)
!51 = distinct !DILexicalBlock(scope: !41, file: !11, line: 18, column: 5)
!52 = !DILocation(line: 18, column: 14, scope: !51)
!53 = !DILocation(line: 18, column: 10, scope: !51)
!54 = !DILocation(line: 18, column: 21, scope: !55)
!55 = distinct !DILexicalBlock(scope: !51, file: !11, line: 18, column: 5)
!56 = !DILocation(line: 18, column: 25, scope: !55)
!57 = !DILocation(line: 18, column: 23, scope: !55)
!58 = !DILocation(line: 18, column: 5, scope: !51)
!59 = !DILocation(line: 19, column: 17, scope: !60)
!60 = distinct !DILexicalBlock(scope: !55, file: !11, line: 18, column: 36)
!61 = !DILocation(line: 19, column: 25, scope: !60)
!62 = !DILocation(line: 19, column: 29, scope: !60)
!63 = !DILocation(line: 19, column: 23, scope: !60)
!64 = !DILocation(line: 19, column: 15, scope: !60)
!65 = !DILocation(line: 20, column: 5, scope: !60)
!66 = !DILocation(line: 18, column: 32, scope: !55)
!67 = !DILocation(line: 18, column: 5, scope: !55)
!68 = distinct !{!68, !58, !69, !39}
!69 = !DILocation(line: 20, column: 5, scope: !51)
!70 = !DILocation(line: 21, column: 12, scope: !41)
!71 = !DILocation(line: 21, column: 5, scope: !41)
!72 = distinct !DISubprogram(name: "count_positive", scope: !11, file: !11, line: 24, type: !42, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!73 = !DILocalVariable(name: "arr", arg: 1, scope: !72, file: !11, line: 24, type: !14)
!74 = !DILocation(line: 24, column: 25, scope: !72)
!75 = !DILocalVariable(name: "size", arg: 2, scope: !72, file: !11, line: 24, type: !15)
!76 = !DILocation(line: 24, column: 34, scope: !72)
!77 = !DILocalVariable(name: "count", scope: !72, file: !11, line: 25, type: !15)
!78 = !DILocation(line: 25, column: 9, scope: !72)
!79 = !DILocalVariable(name: "i", scope: !80, file: !11, line: 26, type: !15)
!80 = distinct !DILexicalBlock(scope: !72, file: !11, line: 26, column: 5)
!81 = !DILocation(line: 26, column: 14, scope: !80)
!82 = !DILocation(line: 26, column: 10, scope: !80)
!83 = !DILocation(line: 26, column: 21, scope: !84)
!84 = distinct !DILexicalBlock(scope: !80, file: !11, line: 26, column: 5)
!85 = !DILocation(line: 26, column: 25, scope: !84)
!86 = !DILocation(line: 26, column: 23, scope: !84)
!87 = !DILocation(line: 26, column: 5, scope: !80)
!88 = !DILocation(line: 27, column: 13, scope: !89)
!89 = distinct !DILexicalBlock(scope: !90, file: !11, line: 27, column: 13)
!90 = distinct !DILexicalBlock(scope: !84, file: !11, line: 26, column: 36)
!91 = !DILocation(line: 27, column: 17, scope: !89)
!92 = !DILocation(line: 27, column: 20, scope: !89)
!93 = !DILocation(line: 27, column: 13, scope: !90)
!94 = !DILocation(line: 28, column: 21, scope: !95)
!95 = distinct !DILexicalBlock(scope: !89, file: !11, line: 27, column: 25)
!96 = !DILocation(line: 28, column: 27, scope: !95)
!97 = !DILocation(line: 28, column: 19, scope: !95)
!98 = !DILocation(line: 29, column: 9, scope: !95)
!99 = !DILocation(line: 30, column: 5, scope: !90)
!100 = !DILocation(line: 26, column: 32, scope: !84)
!101 = !DILocation(line: 26, column: 5, scope: !84)
!102 = distinct !{!102, !87, !103, !39}
!103 = !DILocation(line: 30, column: 5, scope: !80)
!104 = !DILocation(line: 31, column: 12, scope: !72)
!105 = !DILocation(line: 31, column: 5, scope: !72)
!106 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 34, type: !107, scopeLine: 34, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!107 = !DISubroutineType(types: !108)
!108 = !{!15}
!109 = !DILocalVariable(name: "data", scope: !106, file: !11, line: 35, type: !110)
!110 = !DICompositeType(tag: DW_TAG_array_type, baseType: !15, size: 3200, elements: !111)
!111 = !{!112}
!112 = !DISubrange(count: 100)
!113 = !DILocation(line: 35, column: 9, scope: !106)
!114 = !DILocation(line: 36, column: 16, scope: !106)
!115 = !DILocation(line: 36, column: 5, scope: !106)
!116 = !DILocalVariable(name: "s", scope: !106, file: !11, line: 37, type: !15)
!117 = !DILocation(line: 37, column: 9, scope: !106)
!118 = !DILocation(line: 37, column: 23, scope: !106)
!119 = !DILocation(line: 37, column: 13, scope: !106)
!120 = !DILocalVariable(name: "c", scope: !106, file: !11, line: 38, type: !15)
!121 = !DILocation(line: 38, column: 9, scope: !106)
!122 = !DILocation(line: 38, column: 28, scope: !106)
!123 = !DILocation(line: 38, column: 13, scope: !106)
!124 = !DILocation(line: 39, column: 12, scope: !106)
!125 = !DILocation(line: 39, column: 16, scope: !106)
!126 = !DILocation(line: 39, column: 14, scope: !106)
!127 = !DILocation(line: 39, column: 5, scope: !106)
