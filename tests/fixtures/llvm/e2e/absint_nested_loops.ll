; ModuleID = '/workspace/tests/programs/c/absint_nested_loops.c'
source_filename = "/workspace/tests/programs/c/absint_nested_loops.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @init_matrix(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !20, metadata !DIExpression()), !dbg !21
  call void @llvm.dbg.declare(metadata ptr %3, metadata !22, metadata !DIExpression()), !dbg !24
  store i32 0, ptr %3, align 4, !dbg !24
  br label %5, !dbg !25

5:                                                ; preds = %28, %1
  %6 = load i32, ptr %3, align 4, !dbg !26
  %7 = icmp slt i32 %6, 8, !dbg !28
  br i1 %7, label %8, label %31, !dbg !29

8:                                                ; preds = %5
  call void @llvm.dbg.declare(metadata ptr %4, metadata !30, metadata !DIExpression()), !dbg !33
  store i32 0, ptr %4, align 4, !dbg !33
  br label %9, !dbg !34

9:                                                ; preds = %24, %8
  %10 = load i32, ptr %4, align 4, !dbg !35
  %11 = icmp slt i32 %10, 8, !dbg !37
  br i1 %11, label %12, label %27, !dbg !38

12:                                               ; preds = %9
  %13 = load i32, ptr %3, align 4, !dbg !39
  %14 = mul nsw i32 %13, 8, !dbg !41
  %15 = load i32, ptr %4, align 4, !dbg !42
  %16 = add nsw i32 %14, %15, !dbg !43
  %17 = load ptr, ptr %2, align 8, !dbg !44
  %18 = load i32, ptr %3, align 4, !dbg !45
  %19 = sext i32 %18 to i64, !dbg !44
  %20 = getelementptr inbounds [8 x i32], ptr %17, i64 %19, !dbg !44
  %21 = load i32, ptr %4, align 4, !dbg !46
  %22 = sext i32 %21 to i64, !dbg !44
  %23 = getelementptr inbounds [8 x i32], ptr %20, i64 0, i64 %22, !dbg !44
  store i32 %16, ptr %23, align 4, !dbg !47
  br label %24, !dbg !48

24:                                               ; preds = %12
  %25 = load i32, ptr %4, align 4, !dbg !49
  %26 = add nsw i32 %25, 1, !dbg !49
  store i32 %26, ptr %4, align 4, !dbg !49
  br label %9, !dbg !50, !llvm.loop !51

27:                                               ; preds = %9
  br label %28, !dbg !54

28:                                               ; preds = %27
  %29 = load i32, ptr %3, align 4, !dbg !55
  %30 = add nsw i32 %29, 1, !dbg !55
  store i32 %30, ptr %3, align 4, !dbg !55
  br label %5, !dbg !56, !llvm.loop !57

31:                                               ; preds = %5
  ret void, !dbg !59
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @sum_matrix(ptr noundef %0) #0 !dbg !60 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !63, metadata !DIExpression()), !dbg !64
  call void @llvm.dbg.declare(metadata ptr %3, metadata !65, metadata !DIExpression()), !dbg !66
  store i32 0, ptr %3, align 4, !dbg !66
  call void @llvm.dbg.declare(metadata ptr %4, metadata !67, metadata !DIExpression()), !dbg !69
  store i32 0, ptr %4, align 4, !dbg !69
  br label %6, !dbg !70

6:                                                ; preds = %28, %1
  %7 = load i32, ptr %4, align 4, !dbg !71
  %8 = icmp slt i32 %7, 8, !dbg !73
  br i1 %8, label %9, label %31, !dbg !74

9:                                                ; preds = %6
  call void @llvm.dbg.declare(metadata ptr %5, metadata !75, metadata !DIExpression()), !dbg !78
  store i32 0, ptr %5, align 4, !dbg !78
  br label %10, !dbg !79

10:                                               ; preds = %24, %9
  %11 = load i32, ptr %5, align 4, !dbg !80
  %12 = icmp slt i32 %11, 8, !dbg !82
  br i1 %12, label %13, label %27, !dbg !83

13:                                               ; preds = %10
  %14 = load i32, ptr %3, align 4, !dbg !84
  %15 = load ptr, ptr %2, align 8, !dbg !86
  %16 = load i32, ptr %4, align 4, !dbg !87
  %17 = sext i32 %16 to i64, !dbg !86
  %18 = getelementptr inbounds [8 x i32], ptr %15, i64 %17, !dbg !86
  %19 = load i32, ptr %5, align 4, !dbg !88
  %20 = sext i32 %19 to i64, !dbg !86
  %21 = getelementptr inbounds [8 x i32], ptr %18, i64 0, i64 %20, !dbg !86
  %22 = load i32, ptr %21, align 4, !dbg !86
  %23 = add nsw i32 %14, %22, !dbg !89
  store i32 %23, ptr %3, align 4, !dbg !90
  br label %24, !dbg !91

24:                                               ; preds = %13
  %25 = load i32, ptr %5, align 4, !dbg !92
  %26 = add nsw i32 %25, 1, !dbg !92
  store i32 %26, ptr %5, align 4, !dbg !92
  br label %10, !dbg !93, !llvm.loop !94

27:                                               ; preds = %10
  br label %28, !dbg !96

28:                                               ; preds = %27
  %29 = load i32, ptr %4, align 4, !dbg !97
  %30 = add nsw i32 %29, 1, !dbg !97
  store i32 %30, ptr %4, align 4, !dbg !97
  br label %6, !dbg !98, !llvm.loop !99

31:                                               ; preds = %6
  %32 = load i32, ptr %3, align 4, !dbg !101
  ret i32 %32, !dbg !102
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !103 {
  %1 = alloca i32, align 4
  %2 = alloca [8 x [8 x i32]], align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !106, metadata !DIExpression()), !dbg !109
  %3 = getelementptr inbounds [8 x [8 x i32]], ptr %2, i64 0, i64 0, !dbg !110
  call void @init_matrix(ptr noundef %3), !dbg !111
  %4 = getelementptr inbounds [8 x [8 x i32]], ptr %2, i64 0, i64 0, !dbg !112
  %5 = call i32 @sum_matrix(ptr noundef %4), !dbg !113
  ret i32 %5, !dbg !114
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/absint_nested_loops.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3f7389798d8dba254f9f9e3aa542f848")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "init_matrix", scope: !11, file: !11, line: 9, type: !12, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!11 = !DIFile(filename: "c/absint_nested_loops.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3f7389798d8dba254f9f9e3aa542f848")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DICompositeType(tag: DW_TAG_array_type, baseType: !16, size: 256, elements: !17)
!16 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!17 = !{!18}
!18 = !DISubrange(count: 8)
!19 = !{}
!20 = !DILocalVariable(name: "matrix", arg: 1, scope: !10, file: !11, line: 9, type: !14)
!21 = !DILocation(line: 9, column: 22, scope: !10)
!22 = !DILocalVariable(name: "i", scope: !23, file: !11, line: 10, type: !16)
!23 = distinct !DILexicalBlock(scope: !10, file: !11, line: 10, column: 5)
!24 = !DILocation(line: 10, column: 14, scope: !23)
!25 = !DILocation(line: 10, column: 10, scope: !23)
!26 = !DILocation(line: 10, column: 21, scope: !27)
!27 = distinct !DILexicalBlock(scope: !23, file: !11, line: 10, column: 5)
!28 = !DILocation(line: 10, column: 23, scope: !27)
!29 = !DILocation(line: 10, column: 5, scope: !23)
!30 = !DILocalVariable(name: "j", scope: !31, file: !11, line: 11, type: !16)
!31 = distinct !DILexicalBlock(scope: !32, file: !11, line: 11, column: 9)
!32 = distinct !DILexicalBlock(scope: !27, file: !11, line: 10, column: 36)
!33 = !DILocation(line: 11, column: 18, scope: !31)
!34 = !DILocation(line: 11, column: 14, scope: !31)
!35 = !DILocation(line: 11, column: 25, scope: !36)
!36 = distinct !DILexicalBlock(scope: !31, file: !11, line: 11, column: 9)
!37 = !DILocation(line: 11, column: 27, scope: !36)
!38 = !DILocation(line: 11, column: 9, scope: !31)
!39 = !DILocation(line: 12, column: 28, scope: !40)
!40 = distinct !DILexicalBlock(scope: !36, file: !11, line: 11, column: 40)
!41 = !DILocation(line: 12, column: 30, scope: !40)
!42 = !DILocation(line: 12, column: 39, scope: !40)
!43 = !DILocation(line: 12, column: 37, scope: !40)
!44 = !DILocation(line: 12, column: 13, scope: !40)
!45 = !DILocation(line: 12, column: 20, scope: !40)
!46 = !DILocation(line: 12, column: 23, scope: !40)
!47 = !DILocation(line: 12, column: 26, scope: !40)
!48 = !DILocation(line: 13, column: 9, scope: !40)
!49 = !DILocation(line: 11, column: 36, scope: !36)
!50 = !DILocation(line: 11, column: 9, scope: !36)
!51 = distinct !{!51, !38, !52, !53}
!52 = !DILocation(line: 13, column: 9, scope: !31)
!53 = !{!"llvm.loop.mustprogress"}
!54 = !DILocation(line: 14, column: 5, scope: !32)
!55 = !DILocation(line: 10, column: 32, scope: !27)
!56 = !DILocation(line: 10, column: 5, scope: !27)
!57 = distinct !{!57, !29, !58, !53}
!58 = !DILocation(line: 14, column: 5, scope: !23)
!59 = !DILocation(line: 15, column: 1, scope: !10)
!60 = distinct !DISubprogram(name: "sum_matrix", scope: !11, file: !11, line: 17, type: !61, scopeLine: 17, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!61 = !DISubroutineType(types: !62)
!62 = !{!16, !14}
!63 = !DILocalVariable(name: "matrix", arg: 1, scope: !60, file: !11, line: 17, type: !14)
!64 = !DILocation(line: 17, column: 20, scope: !60)
!65 = !DILocalVariable(name: "total", scope: !60, file: !11, line: 18, type: !16)
!66 = !DILocation(line: 18, column: 9, scope: !60)
!67 = !DILocalVariable(name: "i", scope: !68, file: !11, line: 19, type: !16)
!68 = distinct !DILexicalBlock(scope: !60, file: !11, line: 19, column: 5)
!69 = !DILocation(line: 19, column: 14, scope: !68)
!70 = !DILocation(line: 19, column: 10, scope: !68)
!71 = !DILocation(line: 19, column: 21, scope: !72)
!72 = distinct !DILexicalBlock(scope: !68, file: !11, line: 19, column: 5)
!73 = !DILocation(line: 19, column: 23, scope: !72)
!74 = !DILocation(line: 19, column: 5, scope: !68)
!75 = !DILocalVariable(name: "j", scope: !76, file: !11, line: 20, type: !16)
!76 = distinct !DILexicalBlock(scope: !77, file: !11, line: 20, column: 9)
!77 = distinct !DILexicalBlock(scope: !72, file: !11, line: 19, column: 36)
!78 = !DILocation(line: 20, column: 18, scope: !76)
!79 = !DILocation(line: 20, column: 14, scope: !76)
!80 = !DILocation(line: 20, column: 25, scope: !81)
!81 = distinct !DILexicalBlock(scope: !76, file: !11, line: 20, column: 9)
!82 = !DILocation(line: 20, column: 27, scope: !81)
!83 = !DILocation(line: 20, column: 9, scope: !76)
!84 = !DILocation(line: 21, column: 21, scope: !85)
!85 = distinct !DILexicalBlock(scope: !81, file: !11, line: 20, column: 40)
!86 = !DILocation(line: 21, column: 29, scope: !85)
!87 = !DILocation(line: 21, column: 36, scope: !85)
!88 = !DILocation(line: 21, column: 39, scope: !85)
!89 = !DILocation(line: 21, column: 27, scope: !85)
!90 = !DILocation(line: 21, column: 19, scope: !85)
!91 = !DILocation(line: 22, column: 9, scope: !85)
!92 = !DILocation(line: 20, column: 36, scope: !81)
!93 = !DILocation(line: 20, column: 9, scope: !81)
!94 = distinct !{!94, !83, !95, !53}
!95 = !DILocation(line: 22, column: 9, scope: !76)
!96 = !DILocation(line: 23, column: 5, scope: !77)
!97 = !DILocation(line: 19, column: 32, scope: !72)
!98 = !DILocation(line: 19, column: 5, scope: !72)
!99 = distinct !{!99, !74, !100, !53}
!100 = !DILocation(line: 23, column: 5, scope: !68)
!101 = !DILocation(line: 24, column: 12, scope: !60)
!102 = !DILocation(line: 24, column: 5, scope: !60)
!103 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 27, type: !104, scopeLine: 27, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!104 = !DISubroutineType(types: !105)
!105 = !{!16}
!106 = !DILocalVariable(name: "matrix", scope: !103, file: !11, line: 28, type: !107)
!107 = !DICompositeType(tag: DW_TAG_array_type, baseType: !16, size: 2048, elements: !108)
!108 = !{!18, !18}
!109 = !DILocation(line: 28, column: 9, scope: !103)
!110 = !DILocation(line: 29, column: 17, scope: !103)
!111 = !DILocation(line: 29, column: 5, scope: !103)
!112 = !DILocation(line: 30, column: 23, scope: !103)
!113 = !DILocation(line: 30, column: 12, scope: !103)
!114 = !DILocation(line: 30, column: 5, scope: !103)
