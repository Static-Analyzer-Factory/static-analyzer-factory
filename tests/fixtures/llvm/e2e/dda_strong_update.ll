; ModuleID = 'dda_strong_update.c'
source_filename = "dda_strong_update.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !10 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !15, metadata !DIExpression()), !dbg !16
  store i32 1, ptr %2, align 4, !dbg !16
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 2, ptr %3, align 4, !dbg !18
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !21
  store ptr %2, ptr %4, align 8, !dbg !21
  store ptr %3, ptr %4, align 8, !dbg !22
  %5 = load ptr, ptr %4, align 8, !dbg !23
  %6 = load i32, ptr %5, align 4, !dbg !24
  ret i32 %6, !dbg !25
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_strong_update.c", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "33dd7b70faa0bab9e989d9d13fb36c32")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 7, type: !11, scopeLine: 7, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DISubroutineType(types: !12)
!12 = !{!13}
!13 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!14 = !{}
!15 = !DILocalVariable(name: "x", scope: !10, file: !1, line: 8, type: !13)
!16 = !DILocation(line: 8, column: 9, scope: !10)
!17 = !DILocalVariable(name: "y", scope: !10, file: !1, line: 9, type: !13)
!18 = !DILocation(line: 9, column: 9, scope: !10)
!19 = !DILocalVariable(name: "p", scope: !10, file: !1, line: 10, type: !20)
!20 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !13, size: 64)
!21 = !DILocation(line: 10, column: 10, scope: !10)
!22 = !DILocation(line: 13, column: 7, scope: !10)
!23 = !DILocation(line: 16, column: 13, scope: !10)
!24 = !DILocation(line: 16, column: 12, scope: !10)
!25 = !DILocation(line: 16, column: 5, scope: !10)
