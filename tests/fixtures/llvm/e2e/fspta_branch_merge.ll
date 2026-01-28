; ModuleID = '/workspace/tests/programs/c/fspta_branch_merge.c'
source_filename = "/workspace/tests/programs/c/fspta_branch_merge.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@a_val = dso_local global i32 0, align 4, !dbg !0
@b_val = dso_local global i32 0, align 4, !dbg !5

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_branch_merge(i32 noundef %0) #0 !dbg !17 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !22
  call void @llvm.dbg.declare(metadata ptr %3, metadata !23, metadata !DIExpression()), !dbg !25
  %4 = load i32, ptr %2, align 4, !dbg !26
  %5 = icmp ne i32 %4, 0, !dbg !26
  br i1 %5, label %6, label %7, !dbg !28

6:                                                ; preds = %1
  store ptr @a_val, ptr %3, align 8, !dbg !29
  br label %8, !dbg !31

7:                                                ; preds = %1
  store ptr @b_val, ptr %3, align 8, !dbg !32
  br label %8

8:                                                ; preds = %7, %6
  %9 = load ptr, ptr %3, align 8, !dbg !34
  store i32 42, ptr %9, align 4, !dbg !35
  ret void, !dbg !36
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !37 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_branch_merge(i32 noundef 1), !dbg !40
  ret i32 0, !dbg !41
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "a_val", scope: !2, file: !7, line: 10, type: !8, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "/workspace/tests/programs/c/fspta_branch_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "0cb5b254593d03ba091e6a7072312820")
!4 = !{!0, !5}
!5 = !DIGlobalVariableExpression(var: !6, expr: !DIExpression())
!6 = distinct !DIGlobalVariable(name: "b_val", scope: !2, file: !7, line: 10, type: !8, isLocal: false, isDefinition: true)
!7 = !DIFile(filename: "c/fspta_branch_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "0cb5b254593d03ba091e6a7072312820")
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "test_branch_merge", scope: !7, file: !7, line: 12, type: !18, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{null, !8}
!20 = !{}
!21 = !DILocalVariable(name: "cond", arg: 1, scope: !17, file: !7, line: 12, type: !8)
!22 = !DILocation(line: 12, column: 28, scope: !17)
!23 = !DILocalVariable(name: "p", scope: !17, file: !7, line: 13, type: !24)
!24 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !8, size: 64)
!25 = !DILocation(line: 13, column: 10, scope: !17)
!26 = !DILocation(line: 14, column: 9, scope: !27)
!27 = distinct !DILexicalBlock(scope: !17, file: !7, line: 14, column: 9)
!28 = !DILocation(line: 14, column: 9, scope: !17)
!29 = !DILocation(line: 15, column: 11, scope: !30)
!30 = distinct !DILexicalBlock(scope: !27, file: !7, line: 14, column: 15)
!31 = !DILocation(line: 16, column: 5, scope: !30)
!32 = !DILocation(line: 17, column: 11, scope: !33)
!33 = distinct !DILexicalBlock(scope: !27, file: !7, line: 16, column: 12)
!34 = !DILocation(line: 20, column: 6, scope: !17)
!35 = !DILocation(line: 20, column: 8, scope: !17)
!36 = !DILocation(line: 21, column: 1, scope: !17)
!37 = distinct !DISubprogram(name: "main", scope: !7, file: !7, line: 23, type: !38, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!38 = !DISubroutineType(types: !39)
!39 = !{!8}
!40 = !DILocation(line: 24, column: 5, scope: !37)
!41 = !DILocation(line: 25, column: 5, scope: !37)
