; ModuleID = 'tests/programs/c/absint_shift_count.c'
source_filename = "tests/programs/c/absint_shift_count.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @negative_shift(i32 noundef %0) #0 !dbg !10 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !15, metadata !DIExpression()), !dbg !16
  %3 = load i32, ptr %2, align 4, !dbg !17
  %4 = shl i32 %3, -1, !dbg !18
  ret i32 %4, !dbg !19
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @overflow_shift(i32 noundef %0) #0 !dbg !20 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !22
  %3 = load i32, ptr %2, align 4, !dbg !23
  %4 = shl i32 %3, 32, !dbg !24
  ret i32 %4, !dbg !25
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @maybe_negative_shift(i32 noundef %0, i32 noundef %1) #0 !dbg !26 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !29, metadata !DIExpression()), !dbg !30
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !31, metadata !DIExpression()), !dbg !32
  %6 = load i32, ptr %5, align 4, !dbg !33
  %7 = icmp sge i32 %6, -5, !dbg !35
  br i1 %7, label %8, label %15, !dbg !36

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !37
  %10 = icmp sle i32 %9, 5, !dbg !38
  br i1 %10, label %11, label %15, !dbg !39

11:                                               ; preds = %8
  %12 = load i32, ptr %4, align 4, !dbg !40
  %13 = load i32, ptr %5, align 4, !dbg !42
  %14 = shl i32 %12, %13, !dbg !43
  store i32 %14, ptr %3, align 4, !dbg !44
  br label %16, !dbg !44

15:                                               ; preds = %8, %2
  store i32 0, ptr %3, align 4, !dbg !45
  br label %16, !dbg !45

16:                                               ; preds = %15, %11
  %17 = load i32, ptr %3, align 4, !dbg !46
  ret i32 %17, !dbg !46
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @maybe_overflow_shift(i32 noundef %0, i32 noundef %1) #0 !dbg !47 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !48, metadata !DIExpression()), !dbg !49
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !50, metadata !DIExpression()), !dbg !51
  %6 = load i32, ptr %5, align 4, !dbg !52
  %7 = icmp sge i32 %6, 28, !dbg !54
  br i1 %7, label %8, label %15, !dbg !55

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !56
  %10 = icmp sle i32 %9, 35, !dbg !57
  br i1 %10, label %11, label %15, !dbg !58

11:                                               ; preds = %8
  %12 = load i32, ptr %4, align 4, !dbg !59
  %13 = load i32, ptr %5, align 4, !dbg !61
  %14 = shl i32 %12, %13, !dbg !62
  store i32 %14, ptr %3, align 4, !dbg !63
  br label %16, !dbg !63

15:                                               ; preds = %8, %2
  store i32 0, ptr %3, align 4, !dbg !64
  br label %16, !dbg !64

16:                                               ; preds = %15, %11
  %17 = load i32, ptr %3, align 4, !dbg !65
  ret i32 %17, !dbg !65
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @safe_shift(i32 noundef %0, i32 noundef %1) #0 !dbg !66 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !67, metadata !DIExpression()), !dbg !68
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !69, metadata !DIExpression()), !dbg !70
  %6 = load i32, ptr %5, align 4, !dbg !71
  %7 = icmp sge i32 %6, 0, !dbg !73
  br i1 %7, label %8, label %15, !dbg !74

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !75
  %10 = icmp slt i32 %9, 32, !dbg !76
  br i1 %10, label %11, label %15, !dbg !77

11:                                               ; preds = %8
  %12 = load i32, ptr %4, align 4, !dbg !78
  %13 = load i32, ptr %5, align 4, !dbg !80
  %14 = shl i32 %12, %13, !dbg !81
  store i32 %14, ptr %3, align 4, !dbg !82
  br label %16, !dbg !82

15:                                               ; preds = %8, %2
  store i32 0, ptr %3, align 4, !dbg !83
  br label %16, !dbg !83

16:                                               ; preds = %15, %11
  %17 = load i32, ptr %3, align 4, !dbg !84
  ret i32 %17, !dbg !84
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @unsigned_right_shift_overflow(i32 noundef %0) #0 !dbg !85 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !93, metadata !DIExpression()), !dbg !94
  %3 = load i32, ptr %2, align 4, !dbg !95
  %4 = lshr i32 %3, 64, !dbg !96
  ret i32 %4, !dbg !97
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @arith_shift_negative(i32 noundef %0) #0 !dbg !98 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !104, metadata !DIExpression()), !dbg !105
  %3 = load i32, ptr %2, align 4, !dbg !106
  %4 = ashr i32 %3, -2, !dbg !107
  ret i32 %4, !dbg !108
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @safe_after_check(i32 noundef %0, i32 noundef %1) #0 !dbg !109 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !110, metadata !DIExpression()), !dbg !111
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !112, metadata !DIExpression()), !dbg !113
  %6 = load i32, ptr %5, align 4, !dbg !114
  %7 = icmp sgt i32 %6, 0, !dbg !116
  br i1 %7, label %8, label %15, !dbg !117

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !118
  %10 = icmp slt i32 %9, 31, !dbg !119
  br i1 %10, label %11, label %15, !dbg !120

11:                                               ; preds = %8
  %12 = load i32, ptr %4, align 4, !dbg !121
  %13 = load i32, ptr %5, align 4, !dbg !123
  %14 = shl i32 %12, %13, !dbg !124
  store i32 %14, ptr %3, align 4, !dbg !125
  br label %16, !dbg !125

15:                                               ; preds = %8, %2
  store i32 1, ptr %3, align 4, !dbg !126
  br label %16, !dbg !126

16:                                               ; preds = %15, %11
  %17 = load i32, ptr %3, align 4, !dbg !127
  ret i32 %17, !dbg !127
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/absint_shift_count.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "c5e114e80de67b6b9dc2d9f69c4ee448")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "negative_shift", scope: !1, file: !1, line: 4, type: !11, scopeLine: 4, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DISubroutineType(types: !12)
!12 = !{!13, !13}
!13 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!14 = !{}
!15 = !DILocalVariable(name: "x", arg: 1, scope: !10, file: !1, line: 4, type: !13)
!16 = !DILocation(line: 4, column: 24, scope: !10)
!17 = !DILocation(line: 5, column: 12, scope: !10)
!18 = !DILocation(line: 5, column: 14, scope: !10)
!19 = !DILocation(line: 5, column: 5, scope: !10)
!20 = distinct !DISubprogram(name: "overflow_shift", scope: !1, file: !1, line: 9, type: !11, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!21 = !DILocalVariable(name: "x", arg: 1, scope: !20, file: !1, line: 9, type: !13)
!22 = !DILocation(line: 9, column: 24, scope: !20)
!23 = !DILocation(line: 10, column: 12, scope: !20)
!24 = !DILocation(line: 10, column: 14, scope: !20)
!25 = !DILocation(line: 10, column: 5, scope: !20)
!26 = distinct !DISubprogram(name: "maybe_negative_shift", scope: !1, file: !1, line: 14, type: !27, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!27 = !DISubroutineType(types: !28)
!28 = !{!13, !13, !13}
!29 = !DILocalVariable(name: "x", arg: 1, scope: !26, file: !1, line: 14, type: !13)
!30 = !DILocation(line: 14, column: 30, scope: !26)
!31 = !DILocalVariable(name: "y", arg: 2, scope: !26, file: !1, line: 14, type: !13)
!32 = !DILocation(line: 14, column: 37, scope: !26)
!33 = !DILocation(line: 16, column: 9, scope: !34)
!34 = distinct !DILexicalBlock(scope: !26, file: !1, line: 16, column: 9)
!35 = !DILocation(line: 16, column: 11, scope: !34)
!36 = !DILocation(line: 16, column: 17, scope: !34)
!37 = !DILocation(line: 16, column: 20, scope: !34)
!38 = !DILocation(line: 16, column: 22, scope: !34)
!39 = !DILocation(line: 16, column: 9, scope: !26)
!40 = !DILocation(line: 17, column: 16, scope: !41)
!41 = distinct !DILexicalBlock(scope: !34, file: !1, line: 16, column: 28)
!42 = !DILocation(line: 17, column: 21, scope: !41)
!43 = !DILocation(line: 17, column: 18, scope: !41)
!44 = !DILocation(line: 17, column: 9, scope: !41)
!45 = !DILocation(line: 19, column: 5, scope: !26)
!46 = !DILocation(line: 20, column: 1, scope: !26)
!47 = distinct !DISubprogram(name: "maybe_overflow_shift", scope: !1, file: !1, line: 23, type: !27, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!48 = !DILocalVariable(name: "x", arg: 1, scope: !47, file: !1, line: 23, type: !13)
!49 = !DILocation(line: 23, column: 30, scope: !47)
!50 = !DILocalVariable(name: "y", arg: 2, scope: !47, file: !1, line: 23, type: !13)
!51 = !DILocation(line: 23, column: 37, scope: !47)
!52 = !DILocation(line: 25, column: 9, scope: !53)
!53 = distinct !DILexicalBlock(scope: !47, file: !1, line: 25, column: 9)
!54 = !DILocation(line: 25, column: 11, scope: !53)
!55 = !DILocation(line: 25, column: 17, scope: !53)
!56 = !DILocation(line: 25, column: 20, scope: !53)
!57 = !DILocation(line: 25, column: 22, scope: !53)
!58 = !DILocation(line: 25, column: 9, scope: !47)
!59 = !DILocation(line: 26, column: 16, scope: !60)
!60 = distinct !DILexicalBlock(scope: !53, file: !1, line: 25, column: 29)
!61 = !DILocation(line: 26, column: 21, scope: !60)
!62 = !DILocation(line: 26, column: 18, scope: !60)
!63 = !DILocation(line: 26, column: 9, scope: !60)
!64 = !DILocation(line: 28, column: 5, scope: !47)
!65 = !DILocation(line: 29, column: 1, scope: !47)
!66 = distinct !DISubprogram(name: "safe_shift", scope: !1, file: !1, line: 32, type: !27, scopeLine: 32, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!67 = !DILocalVariable(name: "x", arg: 1, scope: !66, file: !1, line: 32, type: !13)
!68 = !DILocation(line: 32, column: 20, scope: !66)
!69 = !DILocalVariable(name: "y", arg: 2, scope: !66, file: !1, line: 32, type: !13)
!70 = !DILocation(line: 32, column: 27, scope: !66)
!71 = !DILocation(line: 33, column: 9, scope: !72)
!72 = distinct !DILexicalBlock(scope: !66, file: !1, line: 33, column: 9)
!73 = !DILocation(line: 33, column: 11, scope: !72)
!74 = !DILocation(line: 33, column: 16, scope: !72)
!75 = !DILocation(line: 33, column: 19, scope: !72)
!76 = !DILocation(line: 33, column: 21, scope: !72)
!77 = !DILocation(line: 33, column: 9, scope: !66)
!78 = !DILocation(line: 34, column: 16, scope: !79)
!79 = distinct !DILexicalBlock(scope: !72, file: !1, line: 33, column: 27)
!80 = !DILocation(line: 34, column: 21, scope: !79)
!81 = !DILocation(line: 34, column: 18, scope: !79)
!82 = !DILocation(line: 34, column: 9, scope: !79)
!83 = !DILocation(line: 36, column: 5, scope: !66)
!84 = !DILocation(line: 37, column: 1, scope: !66)
!85 = distinct !DISubprogram(name: "unsigned_right_shift_overflow", scope: !1, file: !1, line: 40, type: !86, scopeLine: 40, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!86 = !DISubroutineType(types: !87)
!87 = !{!88, !88}
!88 = !DIDerivedType(tag: DW_TAG_typedef, name: "uint32_t", file: !89, line: 26, baseType: !90)
!89 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/stdint-uintn.h", directory: "", checksumkind: CSK_MD5, checksum: "256fcabbefa27ca8cf5e6d37525e6e16")
!90 = !DIDerivedType(tag: DW_TAG_typedef, name: "__uint32_t", file: !91, line: 42, baseType: !92)
!91 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types.h", directory: "", checksumkind: CSK_MD5, checksum: "e1865d9fe29fe1b5ced550b7ba458f9e")
!92 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!93 = !DILocalVariable(name: "x", arg: 1, scope: !85, file: !1, line: 40, type: !88)
!94 = !DILocation(line: 40, column: 49, scope: !85)
!95 = !DILocation(line: 41, column: 12, scope: !85)
!96 = !DILocation(line: 41, column: 14, scope: !85)
!97 = !DILocation(line: 41, column: 5, scope: !85)
!98 = distinct !DISubprogram(name: "arith_shift_negative", scope: !1, file: !1, line: 45, type: !99, scopeLine: 45, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!99 = !DISubroutineType(types: !100)
!100 = !{!101, !101}
!101 = !DIDerivedType(tag: DW_TAG_typedef, name: "int32_t", file: !102, line: 26, baseType: !103)
!102 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/stdint-intn.h", directory: "", checksumkind: CSK_MD5, checksum: "649b383a60bfa3eb90e85840b2b0be20")
!103 = !DIDerivedType(tag: DW_TAG_typedef, name: "__int32_t", file: !91, line: 41, baseType: !13)
!104 = !DILocalVariable(name: "x", arg: 1, scope: !98, file: !1, line: 45, type: !101)
!105 = !DILocation(line: 45, column: 38, scope: !98)
!106 = !DILocation(line: 46, column: 12, scope: !98)
!107 = !DILocation(line: 46, column: 14, scope: !98)
!108 = !DILocation(line: 46, column: 5, scope: !98)
!109 = distinct !DISubprogram(name: "safe_after_check", scope: !1, file: !1, line: 50, type: !27, scopeLine: 50, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!110 = !DILocalVariable(name: "x", arg: 1, scope: !109, file: !1, line: 50, type: !13)
!111 = !DILocation(line: 50, column: 26, scope: !109)
!112 = !DILocalVariable(name: "y", arg: 2, scope: !109, file: !1, line: 50, type: !13)
!113 = !DILocation(line: 50, column: 33, scope: !109)
!114 = !DILocation(line: 51, column: 9, scope: !115)
!115 = distinct !DILexicalBlock(scope: !109, file: !1, line: 51, column: 9)
!116 = !DILocation(line: 51, column: 11, scope: !115)
!117 = !DILocation(line: 51, column: 15, scope: !115)
!118 = !DILocation(line: 51, column: 18, scope: !115)
!119 = !DILocation(line: 51, column: 20, scope: !115)
!120 = !DILocation(line: 51, column: 9, scope: !109)
!121 = !DILocation(line: 52, column: 16, scope: !122)
!122 = distinct !DILexicalBlock(scope: !115, file: !1, line: 51, column: 26)
!123 = !DILocation(line: 52, column: 21, scope: !122)
!124 = !DILocation(line: 52, column: 18, scope: !122)
!125 = !DILocation(line: 52, column: 9, scope: !122)
!126 = !DILocation(line: 54, column: 5, scope: !109)
!127 = !DILocation(line: 55, column: 1, scope: !109)
