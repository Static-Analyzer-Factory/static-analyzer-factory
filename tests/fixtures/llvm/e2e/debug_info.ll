; ModuleID = '/tmp/test_debug.c'
source_filename = "/tmp/test_debug.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @add(i32 noundef %0, i32 noundef %1) #0 !dbg !10 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !16, metadata !DIExpression()), !dbg !17
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !18, metadata !DIExpression()), !dbg !19
  %5 = load i32, ptr %3, align 4, !dbg !20
  %6 = load i32, ptr %4, align 4, !dbg !21
  %7 = add nsw i32 %5, %6, !dbg !22
  ret i32 %7, !dbg !23
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !24 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %2 = call i32 @add(i32 noundef 1, i32 noundef 2), !dbg !27
  ret i32 %2, !dbg !28
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/tmp/test_debug.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "ea9626ead9a64b05ab8d8dcaa9801c3d")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "add", scope: !11, file: !11, line: 1, type: !12, scopeLine: 1, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DIFile(filename: "/tmp/test_debug.c", directory: "", checksumkind: CSK_MD5, checksum: "ea9626ead9a64b05ab8d8dcaa9801c3d")
!12 = !DISubroutineType(types: !13)
!13 = !{!14, !14, !14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "a", arg: 1, scope: !10, file: !11, line: 1, type: !14)
!17 = !DILocation(line: 1, column: 13, scope: !10)
!18 = !DILocalVariable(name: "b", arg: 2, scope: !10, file: !11, line: 1, type: !14)
!19 = !DILocation(line: 1, column: 20, scope: !10)
!20 = !DILocation(line: 1, column: 32, scope: !10)
!21 = !DILocation(line: 1, column: 36, scope: !10)
!22 = !DILocation(line: 1, column: 34, scope: !10)
!23 = !DILocation(line: 1, column: 25, scope: !10)
!24 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 2, type: !25, scopeLine: 2, spFlags: DISPFlagDefinition, unit: !0)
!25 = !DISubroutineType(types: !26)
!26 = !{!14}
!27 = !DILocation(line: 2, column: 21, scope: !24)
!28 = !DILocation(line: 2, column: 14, scope: !24)
