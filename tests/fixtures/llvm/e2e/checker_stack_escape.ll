; ModuleID = '/tmp/raw.ll'
source_filename = "tests/programs/c/checker_stack_escape.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local ptr @get_value() #0 !dbg !10 {
  %1 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !16, metadata !DIExpression()), !dbg !17
  store i32 42, ptr %1, align 4, !dbg !17
  ret ptr %1, !dbg !18
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !19 {
  %1 = call ptr @get_value(), !dbg !22
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !23, metadata !DIExpression()), !dbg !24
  %2 = load i32, ptr %1, align 4, !dbg !25
  ret i32 %2, !dbg !26
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/checker_stack_escape.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "a44b52907f55e576cfe7f444d557de8c")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "get_value", scope: !1, file: !1, line: 1, type: !11, scopeLine: 1, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DISubroutineType(types: !12)
!12 = !{!13}
!13 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "x", scope: !10, file: !1, line: 2, type: !14)
!17 = !DILocation(line: 2, column: 9, scope: !10)
!18 = !DILocation(line: 4, column: 5, scope: !10)
!19 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 7, type: !20, scopeLine: 7, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!20 = !DISubroutineType(types: !21)
!21 = !{!14}
!22 = !DILocation(line: 8, column: 14, scope: !19)
!23 = !DILocalVariable(name: "p", scope: !19, file: !1, line: 8, type: !13)
!24 = !DILocation(line: 0, scope: !19)
!25 = !DILocation(line: 9, column: 12, scope: !19)
!26 = !DILocation(line: 9, column: 5, scope: !19)
