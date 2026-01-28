; ModuleID = '/tmp/checker_memory_leak_raw.ll'
source_filename = "tests/programs/c/checker_memory_leak.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local ptr @create_buffer(i32 noundef %0) #0 !dbg !13 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !17, metadata !DIExpression()), !dbg !18
  %2 = sext i32 %0 to i64, !dbg !19
  %3 = mul i64 %2, 4, !dbg !20
  %4 = call noalias ptr @malloc(i64 noundef %3) #3, !dbg !21
  tail call void @llvm.dbg.value(metadata ptr %4, metadata !22, metadata !DIExpression()), !dbg !18
  %5 = getelementptr inbounds i32, ptr %4, i64 0, !dbg !23
  store i32 42, ptr %5, align 4, !dbg !24
  ret ptr %4, !dbg !25
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind uwtable
define dso_local void @process() #0 !dbg !26 {
  %1 = call ptr @create_buffer(i32 noundef 10), !dbg !29
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !30, metadata !DIExpression()), !dbg !31
  %2 = getelementptr inbounds i32, ptr %1, i64 1, !dbg !32
  store i32 100, ptr %2, align 4, !dbg !33
  ret void, !dbg !34
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !35 {
  call void @process(), !dbg !38
  ret i32 0, !dbg !39
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind allocsize(0) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/checker_memory_leak.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "18e95b1e6c417de6d4f7ccda43f20f6e")
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
!13 = distinct !DISubprogram(name: "create_buffer", scope: !1, file: !1, line: 3, type: !14, scopeLine: 3, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{!3, !4}
!16 = !{}
!17 = !DILocalVariable(name: "size", arg: 1, scope: !13, file: !1, line: 3, type: !4)
!18 = !DILocation(line: 0, scope: !13)
!19 = !DILocation(line: 4, column: 28, scope: !13)
!20 = !DILocation(line: 4, column: 33, scope: !13)
!21 = !DILocation(line: 4, column: 21, scope: !13)
!22 = !DILocalVariable(name: "p", scope: !13, file: !1, line: 4, type: !3)
!23 = !DILocation(line: 5, column: 5, scope: !13)
!24 = !DILocation(line: 5, column: 10, scope: !13)
!25 = !DILocation(line: 6, column: 5, scope: !13)
!26 = distinct !DISubprogram(name: "process", scope: !1, file: !1, line: 9, type: !27, scopeLine: 9, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!27 = !DISubroutineType(types: !28)
!28 = !{null}
!29 = !DILocation(line: 10, column: 16, scope: !26)
!30 = !DILocalVariable(name: "buf", scope: !26, file: !1, line: 10, type: !3)
!31 = !DILocation(line: 0, scope: !26)
!32 = !DILocation(line: 11, column: 5, scope: !26)
!33 = !DILocation(line: 11, column: 12, scope: !26)
!34 = !DILocation(line: 13, column: 1, scope: !26)
!35 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 15, type: !36, scopeLine: 15, spFlags: DISPFlagDefinition, unit: !0)
!36 = !DISubroutineType(types: !37)
!37 = !{!4}
!38 = !DILocation(line: 16, column: 5, scope: !35)
!39 = !DILocation(line: 17, column: 5, scope: !35)
