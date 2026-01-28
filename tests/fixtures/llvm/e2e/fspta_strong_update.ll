; ModuleID = '/workspace/tests/programs/c/fspta_strong_update.c'
source_filename = "/workspace/tests/programs/c/fspta_strong_update.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@a_val = dso_local global i32 0, align 4, !dbg !0
@b_val = dso_local global i32 0, align 4, !dbg !5

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_strong_update() #0 !dbg !17 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !21, metadata !DIExpression()), !dbg !23
  store ptr @a_val, ptr %1, align 8, !dbg !23
  %2 = load ptr, ptr %1, align 8, !dbg !24
  store i32 10, ptr %2, align 4, !dbg !25
  store ptr @b_val, ptr %1, align 8, !dbg !26
  %3 = load ptr, ptr %1, align 8, !dbg !27
  store i32 20, ptr %3, align 4, !dbg !28
  ret void, !dbg !29
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !30 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_strong_update(), !dbg !33
  ret i32 0, !dbg !34
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "a_val", scope: !2, file: !7, line: 9, type: !8, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "/workspace/tests/programs/c/fspta_strong_update.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "37c853f0a412c9326a80031e60f3c4e2")
!4 = !{!0, !5}
!5 = !DIGlobalVariableExpression(var: !6, expr: !DIExpression())
!6 = distinct !DIGlobalVariable(name: "b_val", scope: !2, file: !7, line: 9, type: !8, isLocal: false, isDefinition: true)
!7 = !DIFile(filename: "c/fspta_strong_update.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "37c853f0a412c9326a80031e60f3c4e2")
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "test_strong_update", scope: !7, file: !7, line: 11, type: !18, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{null}
!20 = !{}
!21 = !DILocalVariable(name: "p", scope: !17, file: !7, line: 12, type: !22)
!22 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !8, size: 64)
!23 = !DILocation(line: 12, column: 10, scope: !17)
!24 = !DILocation(line: 13, column: 6, scope: !17)
!25 = !DILocation(line: 13, column: 8, scope: !17)
!26 = !DILocation(line: 14, column: 7, scope: !17)
!27 = !DILocation(line: 15, column: 6, scope: !17)
!28 = !DILocation(line: 15, column: 8, scope: !17)
!29 = !DILocation(line: 18, column: 1, scope: !17)
!30 = distinct !DISubprogram(name: "main", scope: !7, file: !7, line: 20, type: !31, scopeLine: 20, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!31 = !DISubroutineType(types: !32)
!32 = !{!8}
!33 = !DILocation(line: 21, column: 5, scope: !30)
!34 = !DILocation(line: 22, column: 5, scope: !30)
