; ModuleID = 'tests/programs/c/z3_reach_infeasible.c'
source_filename = "tests/programs/c/z3_reach_infeasible.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [16 x i8] c"x is large: %d\0A\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [16 x i8] c"x is small: %d\0A\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @process(i32 noundef %0) #0 !dbg !19 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !24, metadata !DIExpression()), !dbg !25
  %3 = load i32, ptr %2, align 4, !dbg !26
  %4 = icmp sgt i32 %3, 10, !dbg !28
  br i1 %4, label %5, label %8, !dbg !29

5:                                                ; preds = %1
  %6 = load i32, ptr %2, align 4, !dbg !30
  %7 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %6), !dbg !32
  br label %8, !dbg !33

8:                                                ; preds = %5, %1
  %9 = load i32, ptr %2, align 4, !dbg !34
  %10 = icmp slt i32 %9, 5, !dbg !36
  br i1 %10, label %11, label %14, !dbg !37

11:                                               ; preds = %8
  %12 = load i32, ptr %2, align 4, !dbg !38
  %13 = call i32 (ptr, ...) @printf(ptr noundef @.str.1, i32 noundef %12), !dbg !40
  br label %14, !dbg !41

14:                                               ; preds = %11, %8
  ret void, !dbg !42
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @printf(ptr noundef, ...) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !43 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @process(i32 noundef 7), !dbg !46
  ret i32 0, !dbg !47
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!9}
!llvm.module.flags = !{!11, !12, !13, !14, !15, !16, !17}
!llvm.ident = !{!18}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_reach_infeasible.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "a7b3b02544ac2ef198223e55429fc4e4")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 128, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 16)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !3, isLocal: true, isDefinition: true)
!9 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !10, splitDebugInlining: false, nameTableKind: None)
!10 = !{!0, !7}
!11 = !{i32 7, !"Dwarf Version", i32 5}
!12 = !{i32 2, !"Debug Info Version", i32 3}
!13 = !{i32 1, !"wchar_size", i32 4}
!14 = !{i32 8, !"PIC Level", i32 2}
!15 = !{i32 7, !"PIE Level", i32 2}
!16 = !{i32 7, !"uwtable", i32 2}
!17 = !{i32 7, !"frame-pointer", i32 1}
!18 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!19 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 5, type: !20, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !9, retainedNodes: !23)
!20 = !DISubroutineType(types: !21)
!21 = !{null, !22}
!22 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!23 = !{}
!24 = !DILocalVariable(name: "x", arg: 1, scope: !19, file: !2, line: 5, type: !22)
!25 = !DILocation(line: 5, column: 18, scope: !19)
!26 = !DILocation(line: 6, column: 9, scope: !27)
!27 = distinct !DILexicalBlock(scope: !19, file: !2, line: 6, column: 9)
!28 = !DILocation(line: 6, column: 11, scope: !27)
!29 = !DILocation(line: 6, column: 9, scope: !19)
!30 = !DILocation(line: 8, column: 36, scope: !31)
!31 = distinct !DILexicalBlock(scope: !27, file: !2, line: 6, column: 17)
!32 = !DILocation(line: 8, column: 9, scope: !31)
!33 = !DILocation(line: 9, column: 5, scope: !31)
!34 = !DILocation(line: 10, column: 9, scope: !35)
!35 = distinct !DILexicalBlock(scope: !19, file: !2, line: 10, column: 9)
!36 = !DILocation(line: 10, column: 11, scope: !35)
!37 = !DILocation(line: 10, column: 9, scope: !19)
!38 = !DILocation(line: 13, column: 36, scope: !39)
!39 = distinct !DILexicalBlock(scope: !35, file: !2, line: 10, column: 16)
!40 = !DILocation(line: 13, column: 9, scope: !39)
!41 = !DILocation(line: 14, column: 5, scope: !39)
!42 = !DILocation(line: 15, column: 1, scope: !19)
!43 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 17, type: !44, scopeLine: 17, spFlags: DISPFlagDefinition, unit: !9)
!44 = !DISubroutineType(types: !45)
!45 = !{!22}
!46 = !DILocation(line: 18, column: 5, scope: !43)
!47 = !DILocation(line: 19, column: 5, scope: !43)
