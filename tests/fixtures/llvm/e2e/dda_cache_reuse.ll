; ModuleID = 'dda_cache_reuse.c'
source_filename = "dda_cache_reuse.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @get_alloc() #0 !dbg !13 {
  %1 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !15
  ret ptr %1, !dbg !16
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !17 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !22
  %5 = call ptr @get_alloc(), !dbg !23
  store ptr %5, ptr %2, align 8, !dbg !22
  call void @llvm.dbg.declare(metadata ptr %3, metadata !24, metadata !DIExpression()), !dbg !25
  %6 = load ptr, ptr %2, align 8, !dbg !26
  store ptr %6, ptr %3, align 8, !dbg !25
  %7 = load ptr, ptr %2, align 8, !dbg !27
  store i32 1, ptr %7, align 4, !dbg !28
  %8 = load ptr, ptr %3, align 8, !dbg !29
  store i32 2, ptr %8, align 4, !dbg !30
  call void @llvm.dbg.declare(metadata ptr %4, metadata !31, metadata !DIExpression()), !dbg !32
  %9 = load ptr, ptr %2, align 8, !dbg !33
  %10 = load i32, ptr %9, align 4, !dbg !34
  %11 = load ptr, ptr %3, align 8, !dbg !35
  %12 = load i32, ptr %11, align 4, !dbg !36
  %13 = add nsw i32 %10, %12, !dbg !37
  store i32 %13, ptr %4, align 4, !dbg !32
  %14 = load ptr, ptr %2, align 8, !dbg !38
  call void @free(ptr noundef %14) #5, !dbg !39
  %15 = load i32, ptr %4, align 4, !dbg !40
  ret i32 %15, !dbg !41
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_cache_reuse.c", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "be32dab258725c13f78c8f5b8cd2cca6")
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
!13 = distinct !DISubprogram(name: "get_alloc", scope: !1, file: !1, line: 7, type: !14, scopeLine: 7, spFlags: DISPFlagDefinition, unit: !0)
!14 = !DISubroutineType(types: !2)
!15 = !DILocation(line: 8, column: 19, scope: !13)
!16 = !DILocation(line: 8, column: 5, scope: !13)
!17 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 11, type: !18, scopeLine: 11, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{!4}
!20 = !{}
!21 = !DILocalVariable(name: "p", scope: !17, file: !1, line: 12, type: !3)
!22 = !DILocation(line: 12, column: 10, scope: !17)
!23 = !DILocation(line: 12, column: 14, scope: !17)
!24 = !DILocalVariable(name: "q", scope: !17, file: !1, line: 13, type: !3)
!25 = !DILocation(line: 13, column: 10, scope: !17)
!26 = !DILocation(line: 13, column: 14, scope: !17)
!27 = !DILocation(line: 16, column: 6, scope: !17)
!28 = !DILocation(line: 16, column: 8, scope: !17)
!29 = !DILocation(line: 19, column: 6, scope: !17)
!30 = !DILocation(line: 19, column: 8, scope: !17)
!31 = !DILocalVariable(name: "result", scope: !17, file: !1, line: 21, type: !4)
!32 = !DILocation(line: 21, column: 9, scope: !17)
!33 = !DILocation(line: 21, column: 19, scope: !17)
!34 = !DILocation(line: 21, column: 18, scope: !17)
!35 = !DILocation(line: 21, column: 24, scope: !17)
!36 = !DILocation(line: 21, column: 23, scope: !17)
!37 = !DILocation(line: 21, column: 21, scope: !17)
!38 = !DILocation(line: 22, column: 10, scope: !17)
!39 = !DILocation(line: 22, column: 5, scope: !17)
!40 = !DILocation(line: 23, column: 12, scope: !17)
!41 = !DILocation(line: 23, column: 5, scope: !17)
