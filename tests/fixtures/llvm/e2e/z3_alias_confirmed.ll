; ModuleID = 'tests/programs/c/z3_alias_confirmed.c'
source_filename = "tests/programs/c/z3_alias_confirmed.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !10 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !15, metadata !DIExpression()), !dbg !16
  store i32 42, ptr %2, align 4, !dbg !16
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !19
  store ptr %2, ptr %3, align 8, !dbg !19
  call void @llvm.dbg.declare(metadata ptr %4, metadata !20, metadata !DIExpression()), !dbg !21
  store ptr %2, ptr %4, align 8, !dbg !21
  %5 = load ptr, ptr %3, align 8, !dbg !22
  %6 = load i32, ptr %5, align 4, !dbg !23
  %7 = load ptr, ptr %4, align 8, !dbg !24
  %8 = load i32, ptr %7, align 4, !dbg !25
  %9 = add nsw i32 %6, %8, !dbg !26
  ret i32 %9, !dbg !27
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/z3_alias_confirmed.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "4932e6a0c45c51f434c7099027011ac9")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 5, type: !11, scopeLine: 5, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DISubroutineType(types: !12)
!12 = !{!13}
!13 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!14 = !{}
!15 = !DILocalVariable(name: "x", scope: !10, file: !1, line: 6, type: !13)
!16 = !DILocation(line: 6, column: 9, scope: !10)
!17 = !DILocalVariable(name: "p", scope: !10, file: !1, line: 7, type: !18)
!18 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !13, size: 64)
!19 = !DILocation(line: 7, column: 10, scope: !10)
!20 = !DILocalVariable(name: "q", scope: !10, file: !1, line: 8, type: !18)
!21 = !DILocation(line: 8, column: 10, scope: !10)
!22 = !DILocation(line: 10, column: 13, scope: !10)
!23 = !DILocation(line: 10, column: 12, scope: !10)
!24 = !DILocation(line: 10, column: 18, scope: !10)
!25 = !DILocation(line: 10, column: 17, scope: !10)
!26 = !DILocation(line: 10, column: 15, scope: !10)
!27 = !DILocation(line: 10, column: 5, scope: !10)
