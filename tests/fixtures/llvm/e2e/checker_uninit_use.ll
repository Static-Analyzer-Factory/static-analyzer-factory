; ModuleID = '/tmp/raw.ll'
source_filename = "tests/programs/c/checker_uninit_use.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @process() #0 !dbg !13 {
  %1 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !17
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !18, metadata !DIExpression()), !dbg !19
  %2 = load i32, ptr %1, align 4, !dbg !20
  tail call void @llvm.dbg.value(metadata i32 %2, metadata !21, metadata !DIExpression()), !dbg !19
  call void @free(ptr noundef %1) #5, !dbg !22
  ret i32 %2, !dbg !23
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !24 {
  %1 = call i32 @process(), !dbg !25
  ret i32 %1, !dbg !26
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/checker_uninit_use.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "a05c4d11d22c954d2f68a766bb173c15")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 1}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "process", scope: !1, file: !1, line: 3, type: !14, scopeLine: 3, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{!4}
!16 = !{}
!17 = !DILocation(line: 4, column: 21, scope: !13)
!18 = !DILocalVariable(name: "p", scope: !13, file: !1, line: 4, type: !3)
!19 = !DILocation(line: 0, scope: !13)
!20 = !DILocation(line: 7, column: 15, scope: !13)
!21 = !DILocalVariable(name: "val", scope: !13, file: !1, line: 7, type: !4)
!22 = !DILocation(line: 9, column: 5, scope: !13)
!23 = !DILocation(line: 10, column: 5, scope: !13)
!24 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 13, type: !14, scopeLine: 13, spFlags: DISPFlagDefinition, unit: !0)
!25 = !DILocation(line: 14, column: 12, scope: !24)
!26 = !DILocation(line: 14, column: 5, scope: !24)
