; ModuleID = '/workspace/tests/programs/c/absint_integer_overflow.c'
source_filename = "/workspace/tests/programs/c/absint_integer_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @safe_add(i32 noundef %0, i32 noundef %1) #0 !dbg !13 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !18, metadata !DIExpression()), !dbg !19
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !20, metadata !DIExpression()), !dbg !21
  %5 = load i32, ptr %3, align 4, !dbg !22
  %6 = load i32, ptr %4, align 4, !dbg !23
  %7 = add nsw i32 %5, %6, !dbg !24
  ret i32 %7, !dbg !25
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i64 @compute_area(i32 noundef %0, i32 noundef %1) #0 !dbg !26 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !29, metadata !DIExpression()), !dbg !30
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !31, metadata !DIExpression()), !dbg !32
  call void @llvm.dbg.declare(metadata ptr %5, metadata !33, metadata !DIExpression()), !dbg !34
  %6 = load i32, ptr %3, align 4, !dbg !35
  %7 = load i32, ptr %4, align 4, !dbg !36
  %8 = mul nsw i32 %6, %7, !dbg !37
  store i32 %8, ptr %5, align 4, !dbg !34
  %9 = load i32, ptr %5, align 4, !dbg !38
  %10 = sext i32 %9 to i64, !dbg !39
  ret i64 %10, !dbg !40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @accumulate(i32 noundef %0) #0 !dbg !41 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !44, metadata !DIExpression()), !dbg !45
  call void @llvm.dbg.declare(metadata ptr %3, metadata !46, metadata !DIExpression()), !dbg !47
  store i32 0, ptr %3, align 4, !dbg !47
  call void @llvm.dbg.declare(metadata ptr %4, metadata !48, metadata !DIExpression()), !dbg !50
  store i32 0, ptr %4, align 4, !dbg !50
  br label %5, !dbg !51

5:                                                ; preds = %13, %1
  %6 = load i32, ptr %4, align 4, !dbg !52
  %7 = load i32, ptr %2, align 4, !dbg !54
  %8 = icmp slt i32 %6, %7, !dbg !55
  br i1 %8, label %9, label %16, !dbg !56

9:                                                ; preds = %5
  %10 = load i32, ptr %3, align 4, !dbg !57
  %11 = load i32, ptr %4, align 4, !dbg !59
  %12 = add nsw i32 %10, %11, !dbg !60
  store i32 %12, ptr %3, align 4, !dbg !61
  br label %13, !dbg !62

13:                                               ; preds = %9
  %14 = load i32, ptr %4, align 4, !dbg !63
  %15 = add nsw i32 %14, 1, !dbg !63
  store i32 %15, ptr %4, align 4, !dbg !63
  br label %5, !dbg !64, !llvm.loop !65

16:                                               ; preds = %5
  %17 = load i32, ptr %3, align 4, !dbg !68
  ret i32 %17, !dbg !69
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !70 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i64, align 8
  %4 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !73, metadata !DIExpression()), !dbg !74
  %5 = call i32 @safe_add(i32 noundef 10, i32 noundef 20), !dbg !75
  store i32 %5, ptr %2, align 4, !dbg !74
  call void @llvm.dbg.declare(metadata ptr %3, metadata !76, metadata !DIExpression()), !dbg !77
  %6 = call i64 @compute_area(i32 noundef 640, i32 noundef 480), !dbg !78
  store i64 %6, ptr %3, align 8, !dbg !77
  call void @llvm.dbg.declare(metadata ptr %4, metadata !79, metadata !DIExpression()), !dbg !80
  %7 = call i32 @accumulate(i32 noundef 100), !dbg !81
  store i32 %7, ptr %4, align 4, !dbg !80
  %8 = load i32, ptr %2, align 4, !dbg !82
  %9 = sext i32 %8 to i64, !dbg !82
  %10 = load i64, ptr %3, align 8, !dbg !83
  %11 = add nsw i64 %9, %10, !dbg !84
  %12 = load i32, ptr %4, align 4, !dbg !85
  %13 = sext i32 %12 to i64, !dbg !85
  %14 = add nsw i64 %11, %13, !dbg !86
  %15 = trunc i64 %14 to i32, !dbg !87
  ret i32 %15, !dbg !88
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/absint_integer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "69361d15edad3c71a2eb7afca1ebfd4d")
!2 = !{!3, !4}
!3 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!4 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 1}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "safe_add", scope: !14, file: !14, line: 8, type: !15, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!14 = !DIFile(filename: "c/absint_integer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "69361d15edad3c71a2eb7afca1ebfd4d")
!15 = !DISubroutineType(types: !16)
!16 = !{!4, !4, !4}
!17 = !{}
!18 = !DILocalVariable(name: "a", arg: 1, scope: !13, file: !14, line: 8, type: !4)
!19 = !DILocation(line: 8, column: 18, scope: !13)
!20 = !DILocalVariable(name: "b", arg: 2, scope: !13, file: !14, line: 8, type: !4)
!21 = !DILocation(line: 8, column: 25, scope: !13)
!22 = !DILocation(line: 10, column: 12, scope: !13)
!23 = !DILocation(line: 10, column: 16, scope: !13)
!24 = !DILocation(line: 10, column: 14, scope: !13)
!25 = !DILocation(line: 10, column: 5, scope: !13)
!26 = distinct !DISubprogram(name: "compute_area", scope: !14, file: !14, line: 13, type: !27, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!27 = !DISubroutineType(types: !28)
!28 = !{!3, !4, !4}
!29 = !DILocalVariable(name: "width", arg: 1, scope: !26, file: !14, line: 13, type: !4)
!30 = !DILocation(line: 13, column: 23, scope: !26)
!31 = !DILocalVariable(name: "height", arg: 2, scope: !26, file: !14, line: 13, type: !4)
!32 = !DILocation(line: 13, column: 34, scope: !26)
!33 = !DILocalVariable(name: "area", scope: !26, file: !14, line: 16, type: !4)
!34 = !DILocation(line: 16, column: 9, scope: !26)
!35 = !DILocation(line: 16, column: 16, scope: !26)
!36 = !DILocation(line: 16, column: 24, scope: !26)
!37 = !DILocation(line: 16, column: 22, scope: !26)
!38 = !DILocation(line: 17, column: 18, scope: !26)
!39 = !DILocation(line: 17, column: 12, scope: !26)
!40 = !DILocation(line: 17, column: 5, scope: !26)
!41 = distinct !DISubprogram(name: "accumulate", scope: !14, file: !14, line: 20, type: !42, scopeLine: 20, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!42 = !DISubroutineType(types: !43)
!43 = !{!4, !4}
!44 = !DILocalVariable(name: "n", arg: 1, scope: !41, file: !14, line: 20, type: !4)
!45 = !DILocation(line: 20, column: 20, scope: !41)
!46 = !DILocalVariable(name: "sum", scope: !41, file: !14, line: 21, type: !4)
!47 = !DILocation(line: 21, column: 9, scope: !41)
!48 = !DILocalVariable(name: "i", scope: !49, file: !14, line: 22, type: !4)
!49 = distinct !DILexicalBlock(scope: !41, file: !14, line: 22, column: 5)
!50 = !DILocation(line: 22, column: 14, scope: !49)
!51 = !DILocation(line: 22, column: 10, scope: !49)
!52 = !DILocation(line: 22, column: 21, scope: !53)
!53 = distinct !DILexicalBlock(scope: !49, file: !14, line: 22, column: 5)
!54 = !DILocation(line: 22, column: 25, scope: !53)
!55 = !DILocation(line: 22, column: 23, scope: !53)
!56 = !DILocation(line: 22, column: 5, scope: !49)
!57 = !DILocation(line: 23, column: 15, scope: !58)
!58 = distinct !DILexicalBlock(scope: !53, file: !14, line: 22, column: 33)
!59 = !DILocation(line: 23, column: 21, scope: !58)
!60 = !DILocation(line: 23, column: 19, scope: !58)
!61 = !DILocation(line: 23, column: 13, scope: !58)
!62 = !DILocation(line: 24, column: 5, scope: !58)
!63 = !DILocation(line: 22, column: 29, scope: !53)
!64 = !DILocation(line: 22, column: 5, scope: !53)
!65 = distinct !{!65, !56, !66, !67}
!66 = !DILocation(line: 24, column: 5, scope: !49)
!67 = !{!"llvm.loop.mustprogress"}
!68 = !DILocation(line: 25, column: 12, scope: !41)
!69 = !DILocation(line: 25, column: 5, scope: !41)
!70 = distinct !DISubprogram(name: "main", scope: !14, file: !14, line: 28, type: !71, scopeLine: 28, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!71 = !DISubroutineType(types: !72)
!72 = !{!4}
!73 = !DILocalVariable(name: "a", scope: !70, file: !14, line: 29, type: !4)
!74 = !DILocation(line: 29, column: 9, scope: !70)
!75 = !DILocation(line: 29, column: 13, scope: !70)
!76 = !DILocalVariable(name: "area", scope: !70, file: !14, line: 30, type: !3)
!77 = !DILocation(line: 30, column: 10, scope: !70)
!78 = !DILocation(line: 30, column: 17, scope: !70)
!79 = !DILocalVariable(name: "sum", scope: !70, file: !14, line: 31, type: !4)
!80 = !DILocation(line: 31, column: 9, scope: !70)
!81 = !DILocation(line: 31, column: 15, scope: !70)
!82 = !DILocation(line: 32, column: 18, scope: !70)
!83 = !DILocation(line: 32, column: 22, scope: !70)
!84 = !DILocation(line: 32, column: 20, scope: !70)
!85 = !DILocation(line: 32, column: 29, scope: !70)
!86 = !DILocation(line: 32, column: 27, scope: !70)
!87 = !DILocation(line: 32, column: 12, scope: !70)
!88 = !DILocation(line: 32, column: 5, scope: !70)
