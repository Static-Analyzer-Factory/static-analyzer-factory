; ModuleID = 'tests/programs/c/z3_reach_feasible.c'
source_filename = "tests/programs/c/z3_reach_feasible.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [14 x i8] c"positive: %d\0A\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [13 x i8] c"bounded: %d\0A\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @process(i32 noundef %0) #0 !dbg !22 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  %3 = load i32, ptr %2, align 4, !dbg !29
  %4 = icmp sgt i32 %3, 0, !dbg !31
  br i1 %4, label %5, label %8, !dbg !32

5:                                                ; preds = %1
  %6 = load i32, ptr %2, align 4, !dbg !33
  %7 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %6), !dbg !35
  br label %8, !dbg !36

8:                                                ; preds = %5, %1
  %9 = load i32, ptr %2, align 4, !dbg !37
  %10 = icmp slt i32 %9, 100, !dbg !39
  br i1 %10, label %11, label %14, !dbg !40

11:                                               ; preds = %8
  %12 = load i32, ptr %2, align 4, !dbg !41
  %13 = call i32 (ptr, ...) @printf(ptr noundef @.str.1, i32 noundef %12), !dbg !43
  br label %14, !dbg !44

14:                                               ; preds = %11, %8
  ret void, !dbg !45
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @printf(ptr noundef, ...) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !46 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @process(i32 noundef 50), !dbg !49
  ret i32 0, !dbg !50
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!14, !15, !16, !17, !18, !19, !20}
!llvm.ident = !{!21}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_reach_feasible.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "62913783d75b31ead44416cfe200ffd5")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 112, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 14)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 104, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 13)
!12 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !13, splitDebugInlining: false, nameTableKind: None)
!13 = !{!0, !7}
!14 = !{i32 7, !"Dwarf Version", i32 5}
!15 = !{i32 2, !"Debug Info Version", i32 3}
!16 = !{i32 1, !"wchar_size", i32 4}
!17 = !{i32 8, !"PIC Level", i32 2}
!18 = !{i32 7, !"PIE Level", i32 2}
!19 = !{i32 7, !"uwtable", i32 2}
!20 = !{i32 7, !"frame-pointer", i32 1}
!21 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!22 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 5, type: !23, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !26)
!23 = !DISubroutineType(types: !24)
!24 = !{null, !25}
!25 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!26 = !{}
!27 = !DILocalVariable(name: "x", arg: 1, scope: !22, file: !2, line: 5, type: !25)
!28 = !DILocation(line: 5, column: 18, scope: !22)
!29 = !DILocation(line: 6, column: 9, scope: !30)
!30 = distinct !DILexicalBlock(scope: !22, file: !2, line: 6, column: 9)
!31 = !DILocation(line: 6, column: 11, scope: !30)
!32 = !DILocation(line: 6, column: 9, scope: !22)
!33 = !DILocation(line: 8, column: 34, scope: !34)
!34 = distinct !DILexicalBlock(scope: !30, file: !2, line: 6, column: 16)
!35 = !DILocation(line: 8, column: 9, scope: !34)
!36 = !DILocation(line: 9, column: 5, scope: !34)
!37 = !DILocation(line: 10, column: 9, scope: !38)
!38 = distinct !DILexicalBlock(scope: !22, file: !2, line: 10, column: 9)
!39 = !DILocation(line: 10, column: 11, scope: !38)
!40 = !DILocation(line: 10, column: 9, scope: !22)
!41 = !DILocation(line: 13, column: 33, scope: !42)
!42 = distinct !DILexicalBlock(scope: !38, file: !2, line: 10, column: 18)
!43 = !DILocation(line: 13, column: 9, scope: !42)
!44 = !DILocation(line: 14, column: 5, scope: !42)
!45 = !DILocation(line: 15, column: 1, scope: !22)
!46 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 17, type: !47, scopeLine: 17, spFlags: DISPFlagDefinition, unit: !12)
!47 = !DISubroutineType(types: !48)
!48 = !{!25}
!49 = !DILocation(line: 18, column: 5, scope: !46)
!50 = !DILocation(line: 19, column: 5, scope: !46)
