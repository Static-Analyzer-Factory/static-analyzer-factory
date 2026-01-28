; ModuleID = 'dda_basic_query.c'
source_filename = "dda_basic_query.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !13 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !17, metadata !DIExpression()), !dbg !18
  %4 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !19
  store ptr %4, ptr %2, align 8, !dbg !18
  %5 = load ptr, ptr %2, align 8, !dbg !20
  store i32 42, ptr %5, align 4, !dbg !21
  call void @llvm.dbg.declare(metadata ptr %3, metadata !22, metadata !DIExpression()), !dbg !23
  %6 = load ptr, ptr %2, align 8, !dbg !24
  %7 = load i32, ptr %6, align 4, !dbg !25
  store i32 %7, ptr %3, align 4, !dbg !23
  %8 = load ptr, ptr %2, align 8, !dbg !26
  call void @free(ptr noundef %8) #5, !dbg !27
  %9 = load i32, ptr %3, align 4, !dbg !28
  ret i32 %9, !dbg !29
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_basic_query.c", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "525e31925ac3b6f3e469762460a84a35")
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
!13 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 7, type: !14, scopeLine: 7, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{!4}
!16 = !{}
!17 = !DILocalVariable(name: "p", scope: !13, file: !1, line: 8, type: !3)
!18 = !DILocation(line: 8, column: 10, scope: !13)
!19 = !DILocation(line: 8, column: 21, scope: !13)
!20 = !DILocation(line: 9, column: 6, scope: !13)
!21 = !DILocation(line: 9, column: 8, scope: !13)
!22 = !DILocalVariable(name: "x", scope: !13, file: !1, line: 10, type: !4)
!23 = !DILocation(line: 10, column: 9, scope: !13)
!24 = !DILocation(line: 10, column: 14, scope: !13)
!25 = !DILocation(line: 10, column: 13, scope: !13)
!26 = !DILocation(line: 11, column: 10, scope: !13)
!27 = !DILocation(line: 11, column: 5, scope: !13)
!28 = !DILocation(line: 12, column: 12, scope: !13)
!29 = !DILocation(line: 12, column: 5, scope: !13)
