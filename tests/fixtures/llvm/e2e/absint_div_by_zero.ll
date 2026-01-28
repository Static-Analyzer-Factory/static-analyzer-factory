; ModuleID = 'tests/programs/c/absint_div_by_zero.c'
source_filename = "tests/programs/c/absint_div_by_zero.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @definite_div_zero(i32 noundef %0) #0 !dbg !10 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !15, metadata !DIExpression()), !dbg !16
  %3 = load i32, ptr %2, align 4, !dbg !17
  %4 = sdiv i32 %3, 0, !dbg !18
  ret i32 %4, !dbg !19
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @possible_div_zero(i32 noundef %0, i32 noundef %1) #0 !dbg !20 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !23, metadata !DIExpression()), !dbg !24
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !25, metadata !DIExpression()), !dbg !26
  %6 = load i32, ptr %5, align 4, !dbg !27
  %7 = icmp sge i32 %6, 0, !dbg !29
  br i1 %7, label %8, label %15, !dbg !30

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !31
  %10 = icmp sle i32 %9, 10, !dbg !32
  br i1 %10, label %11, label %15, !dbg !33

11:                                               ; preds = %8
  %12 = load i32, ptr %4, align 4, !dbg !34
  %13 = load i32, ptr %5, align 4, !dbg !36
  %14 = sdiv i32 %12, %13, !dbg !37
  store i32 %14, ptr %3, align 4, !dbg !38
  br label %16, !dbg !38

15:                                               ; preds = %8, %2
  store i32 0, ptr %3, align 4, !dbg !39
  br label %16, !dbg !39

16:                                               ; preds = %15, %11
  %17 = load i32, ptr %3, align 4, !dbg !40
  ret i32 %17, !dbg !40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @safe_div(i32 noundef %0, i32 noundef %1) #0 !dbg !41 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !42, metadata !DIExpression()), !dbg !43
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !44, metadata !DIExpression()), !dbg !45
  %6 = load i32, ptr %5, align 4, !dbg !46
  %7 = icmp ne i32 %6, 0, !dbg !48
  br i1 %7, label %8, label %12, !dbg !49

8:                                                ; preds = %2
  %9 = load i32, ptr %4, align 4, !dbg !50
  %10 = load i32, ptr %5, align 4, !dbg !52
  %11 = sdiv i32 %9, %10, !dbg !53
  store i32 %11, ptr %3, align 4, !dbg !54
  br label %13, !dbg !54

12:                                               ; preds = %2
  store i32 0, ptr %3, align 4, !dbg !55
  br label %13, !dbg !55

13:                                               ; preds = %12, %8
  %14 = load i32, ptr %3, align 4, !dbg !56
  ret i32 %14, !dbg !56
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @rem_by_zero(i32 noundef %0) #0 !dbg !57 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !58, metadata !DIExpression()), !dbg !59
  %3 = load i32, ptr %2, align 4, !dbg !60
  %4 = srem i32 %3, 0, !dbg !61
  ret i32 %4, !dbg !62
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @unsigned_div_zero(i32 noundef %0) #0 !dbg !63 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !71, metadata !DIExpression()), !dbg !72
  %3 = load i32, ptr %2, align 4, !dbg !73
  %4 = udiv i32 %3, 0, !dbg !74
  ret i32 %4, !dbg !75
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @safe_after_check(i32 noundef %0, i32 noundef %1) #0 !dbg !76 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !77, metadata !DIExpression()), !dbg !78
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !79, metadata !DIExpression()), !dbg !80
  %6 = load i32, ptr %5, align 4, !dbg !81
  %7 = icmp sgt i32 %6, 0, !dbg !83
  br i1 %7, label %8, label %12, !dbg !84

8:                                                ; preds = %2
  %9 = load i32, ptr %4, align 4, !dbg !85
  %10 = load i32, ptr %5, align 4, !dbg !87
  %11 = sdiv i32 %9, %10, !dbg !88
  store i32 %11, ptr %3, align 4, !dbg !89
  br label %13, !dbg !89

12:                                               ; preds = %2
  store i32 1, ptr %3, align 4, !dbg !90
  br label %13, !dbg !90

13:                                               ; preds = %12, %8
  %14 = load i32, ptr %3, align 4, !dbg !91
  ret i32 %14, !dbg !91
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @unsigned_rem_zero(i32 noundef %0) #0 !dbg !92 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !93, metadata !DIExpression()), !dbg !94
  %3 = load i32, ptr %2, align 4, !dbg !95
  %4 = urem i32 %3, 0, !dbg !96
  ret i32 %4, !dbg !97
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/absint_div_by_zero.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "844d88b1c483f1d81f022e0c76ebcd7e")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "definite_div_zero", scope: !1, file: !1, line: 4, type: !11, scopeLine: 4, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DISubroutineType(types: !12)
!12 = !{!13, !13}
!13 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!14 = !{}
!15 = !DILocalVariable(name: "x", arg: 1, scope: !10, file: !1, line: 4, type: !13)
!16 = !DILocation(line: 4, column: 27, scope: !10)
!17 = !DILocation(line: 5, column: 12, scope: !10)
!18 = !DILocation(line: 5, column: 14, scope: !10)
!19 = !DILocation(line: 5, column: 5, scope: !10)
!20 = distinct !DISubprogram(name: "possible_div_zero", scope: !1, file: !1, line: 9, type: !21, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!21 = !DISubroutineType(types: !22)
!22 = !{!13, !13, !13}
!23 = !DILocalVariable(name: "x", arg: 1, scope: !20, file: !1, line: 9, type: !13)
!24 = !DILocation(line: 9, column: 27, scope: !20)
!25 = !DILocalVariable(name: "y", arg: 2, scope: !20, file: !1, line: 9, type: !13)
!26 = !DILocation(line: 9, column: 34, scope: !20)
!27 = !DILocation(line: 11, column: 9, scope: !28)
!28 = distinct !DILexicalBlock(scope: !20, file: !1, line: 11, column: 9)
!29 = !DILocation(line: 11, column: 11, scope: !28)
!30 = !DILocation(line: 11, column: 16, scope: !28)
!31 = !DILocation(line: 11, column: 19, scope: !28)
!32 = !DILocation(line: 11, column: 21, scope: !28)
!33 = !DILocation(line: 11, column: 9, scope: !20)
!34 = !DILocation(line: 12, column: 16, scope: !35)
!35 = distinct !DILexicalBlock(scope: !28, file: !1, line: 11, column: 28)
!36 = !DILocation(line: 12, column: 20, scope: !35)
!37 = !DILocation(line: 12, column: 18, scope: !35)
!38 = !DILocation(line: 12, column: 9, scope: !35)
!39 = !DILocation(line: 14, column: 5, scope: !20)
!40 = !DILocation(line: 15, column: 1, scope: !20)
!41 = distinct !DISubprogram(name: "safe_div", scope: !1, file: !1, line: 18, type: !21, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!42 = !DILocalVariable(name: "x", arg: 1, scope: !41, file: !1, line: 18, type: !13)
!43 = !DILocation(line: 18, column: 18, scope: !41)
!44 = !DILocalVariable(name: "y", arg: 2, scope: !41, file: !1, line: 18, type: !13)
!45 = !DILocation(line: 18, column: 25, scope: !41)
!46 = !DILocation(line: 19, column: 9, scope: !47)
!47 = distinct !DILexicalBlock(scope: !41, file: !1, line: 19, column: 9)
!48 = !DILocation(line: 19, column: 11, scope: !47)
!49 = !DILocation(line: 19, column: 9, scope: !41)
!50 = !DILocation(line: 20, column: 16, scope: !51)
!51 = distinct !DILexicalBlock(scope: !47, file: !1, line: 19, column: 17)
!52 = !DILocation(line: 20, column: 20, scope: !51)
!53 = !DILocation(line: 20, column: 18, scope: !51)
!54 = !DILocation(line: 20, column: 9, scope: !51)
!55 = !DILocation(line: 22, column: 5, scope: !41)
!56 = !DILocation(line: 23, column: 1, scope: !41)
!57 = distinct !DISubprogram(name: "rem_by_zero", scope: !1, file: !1, line: 26, type: !11, scopeLine: 26, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!58 = !DILocalVariable(name: "x", arg: 1, scope: !57, file: !1, line: 26, type: !13)
!59 = !DILocation(line: 26, column: 21, scope: !57)
!60 = !DILocation(line: 27, column: 12, scope: !57)
!61 = !DILocation(line: 27, column: 14, scope: !57)
!62 = !DILocation(line: 27, column: 5, scope: !57)
!63 = distinct !DISubprogram(name: "unsigned_div_zero", scope: !1, file: !1, line: 31, type: !64, scopeLine: 31, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!64 = !DISubroutineType(types: !65)
!65 = !{!66, !66}
!66 = !DIDerivedType(tag: DW_TAG_typedef, name: "uint32_t", file: !67, line: 26, baseType: !68)
!67 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/stdint-uintn.h", directory: "", checksumkind: CSK_MD5, checksum: "256fcabbefa27ca8cf5e6d37525e6e16")
!68 = !DIDerivedType(tag: DW_TAG_typedef, name: "__uint32_t", file: !69, line: 42, baseType: !70)
!69 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types.h", directory: "", checksumkind: CSK_MD5, checksum: "e1865d9fe29fe1b5ced550b7ba458f9e")
!70 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!71 = !DILocalVariable(name: "x", arg: 1, scope: !63, file: !1, line: 31, type: !66)
!72 = !DILocation(line: 31, column: 37, scope: !63)
!73 = !DILocation(line: 32, column: 12, scope: !63)
!74 = !DILocation(line: 32, column: 14, scope: !63)
!75 = !DILocation(line: 32, column: 5, scope: !63)
!76 = distinct !DISubprogram(name: "safe_after_check", scope: !1, file: !1, line: 36, type: !21, scopeLine: 36, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!77 = !DILocalVariable(name: "x", arg: 1, scope: !76, file: !1, line: 36, type: !13)
!78 = !DILocation(line: 36, column: 26, scope: !76)
!79 = !DILocalVariable(name: "y", arg: 2, scope: !76, file: !1, line: 36, type: !13)
!80 = !DILocation(line: 36, column: 33, scope: !76)
!81 = !DILocation(line: 37, column: 9, scope: !82)
!82 = distinct !DILexicalBlock(scope: !76, file: !1, line: 37, column: 9)
!83 = !DILocation(line: 37, column: 11, scope: !82)
!84 = !DILocation(line: 37, column: 9, scope: !76)
!85 = !DILocation(line: 38, column: 16, scope: !86)
!86 = distinct !DILexicalBlock(scope: !82, file: !1, line: 37, column: 16)
!87 = !DILocation(line: 38, column: 20, scope: !86)
!88 = !DILocation(line: 38, column: 18, scope: !86)
!89 = !DILocation(line: 38, column: 9, scope: !86)
!90 = !DILocation(line: 40, column: 5, scope: !76)
!91 = !DILocation(line: 41, column: 1, scope: !76)
!92 = distinct !DISubprogram(name: "unsigned_rem_zero", scope: !1, file: !1, line: 44, type: !64, scopeLine: 44, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!93 = !DILocalVariable(name: "x", arg: 1, scope: !92, file: !1, line: 44, type: !66)
!94 = !DILocation(line: 44, column: 37, scope: !92)
!95 = !DILocation(line: 45, column: 12, scope: !92)
!96 = !DILocation(line: 45, column: 14, scope: !92)
!97 = !DILocation(line: 45, column: 5, scope: !92)
