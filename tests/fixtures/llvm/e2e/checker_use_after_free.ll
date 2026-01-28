; ModuleID = '/tmp/raw.ll'
source_filename = "tests/programs/c/checker_use_after_free.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [25 x i8] c"Hello, allocated memory!\00", align 1, !dbg !0

; Function Attrs: noinline nounwind uwtable
define dso_local void @process() #0 !dbg !19 {
  %1 = call noalias ptr @malloc(i64 noundef 64) #4, !dbg !23
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !24, metadata !DIExpression()), !dbg !25
  %2 = call ptr @strcpy(ptr noundef %1, ptr noundef @.str) #5, !dbg !26
  call void @free(ptr noundef %1) #5, !dbg !27
  %3 = getelementptr inbounds i8, ptr %1, i64 0, !dbg !28
  %4 = load i8, ptr %3, align 1, !dbg !28
  tail call void @llvm.dbg.value(metadata i8 %4, metadata !29, metadata !DIExpression()), !dbg !25
  %5 = getelementptr inbounds i8, ptr %1, i64 0, !dbg !30
  store i8 88, ptr %5, align 1, !dbg !31
  ret void, !dbg !32
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare ptr @strcpy(ptr noundef, ptr noundef) #3

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !33 {
  call void @process(), !dbg !37
  ret i32 0, !dbg !38
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!11, !12, !13, !14, !15, !16, !17}
!llvm.ident = !{!18}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 6, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/checker_use_after_free.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "69a73921f923cc46268dae88208e0689")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 200, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 25)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !8, globals: !10, splitDebugInlining: false, nameTableKind: None)
!8 = !{!9}
!9 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!10 = !{!0}
!11 = !{i32 7, !"Dwarf Version", i32 5}
!12 = !{i32 2, !"Debug Info Version", i32 3}
!13 = !{i32 1, !"wchar_size", i32 4}
!14 = !{i32 8, !"PIC Level", i32 2}
!15 = !{i32 7, !"PIE Level", i32 2}
!16 = !{i32 7, !"uwtable", i32 2}
!17 = !{i32 7, !"frame-pointer", i32 1}
!18 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!19 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 4, type: !20, scopeLine: 4, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !22)
!20 = !DISubroutineType(types: !21)
!21 = !{null}
!22 = !{}
!23 = !DILocation(line: 5, column: 28, scope: !19)
!24 = !DILocalVariable(name: "buffer", scope: !19, file: !2, line: 5, type: !9)
!25 = !DILocation(line: 0, scope: !19)
!26 = !DILocation(line: 6, column: 5, scope: !19)
!27 = !DILocation(line: 9, column: 5, scope: !19)
!28 = !DILocation(line: 12, column: 14, scope: !19)
!29 = !DILocalVariable(name: "c", scope: !19, file: !2, line: 12, type: !4)
!30 = !DILocation(line: 15, column: 5, scope: !19)
!31 = !DILocation(line: 15, column: 15, scope: !19)
!32 = !DILocation(line: 16, column: 1, scope: !19)
!33 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 18, type: !34, scopeLine: 18, spFlags: DISPFlagDefinition, unit: !7)
!34 = !DISubroutineType(types: !35)
!35 = !{!36}
!36 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!37 = !DILocation(line: 19, column: 5, scope: !33)
!38 = !DILocation(line: 20, column: 5, scope: !33)
