; ModuleID = '/workspace/tests/programs/c/svfg_interproc_direct.c'
source_filename = "/workspace/tests/programs/c/svfg_interproc_direct.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @get_tainted() #0 !dbg !10 {
  %1 = call i32 @source(), !dbg !15
  ret i32 %1, !dbg !16
}

declare i32 @source() #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test() #0 !dbg !17 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !21, metadata !DIExpression()), !dbg !22
  call void @llvm.dbg.declare(metadata ptr %2, metadata !23, metadata !DIExpression()), !dbg !25
  store ptr %1, ptr %2, align 8, !dbg !25
  call void @llvm.dbg.declare(metadata ptr %3, metadata !26, metadata !DIExpression()), !dbg !27
  %5 = call i32 @get_tainted(), !dbg !28
  store i32 %5, ptr %3, align 4, !dbg !27
  %6 = load i32, ptr %3, align 4, !dbg !29
  %7 = load ptr, ptr %2, align 8, !dbg !30
  store i32 %6, ptr %7, align 4, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %4, metadata !32, metadata !DIExpression()), !dbg !33
  %8 = load ptr, ptr %2, align 8, !dbg !34
  %9 = load i32, ptr %8, align 4, !dbg !35
  store i32 %9, ptr %4, align 4, !dbg !33
  %10 = load i32, ptr %4, align 4, !dbg !36
  call void @sink(i32 noundef %10), !dbg !37
  ret void, !dbg !38
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #2

declare void @sink(i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/svfg_interproc_direct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4aeea6622825fcaded35eb7d27d33b9d")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "get_tainted", scope: !11, file: !11, line: 8, type: !12, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!11 = !DIFile(filename: "c/svfg_interproc_direct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4aeea6622825fcaded35eb7d27d33b9d")
!12 = !DISubroutineType(types: !13)
!13 = !{!14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !DILocation(line: 9, column: 12, scope: !10)
!16 = !DILocation(line: 9, column: 5, scope: !10)
!17 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 12, type: !18, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{null}
!20 = !{}
!21 = !DILocalVariable(name: "x", scope: !17, file: !11, line: 13, type: !14)
!22 = !DILocation(line: 13, column: 9, scope: !17)
!23 = !DILocalVariable(name: "p", scope: !17, file: !11, line: 14, type: !24)
!24 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!25 = !DILocation(line: 14, column: 10, scope: !17)
!26 = !DILocalVariable(name: "val", scope: !17, file: !11, line: 16, type: !14)
!27 = !DILocation(line: 16, column: 9, scope: !17)
!28 = !DILocation(line: 16, column: 15, scope: !17)
!29 = !DILocation(line: 17, column: 10, scope: !17)
!30 = !DILocation(line: 17, column: 6, scope: !17)
!31 = !DILocation(line: 17, column: 8, scope: !17)
!32 = !DILocalVariable(name: "loaded", scope: !17, file: !11, line: 18, type: !14)
!33 = !DILocation(line: 18, column: 9, scope: !17)
!34 = !DILocation(line: 18, column: 19, scope: !17)
!35 = !DILocation(line: 18, column: 18, scope: !17)
!36 = !DILocation(line: 19, column: 10, scope: !17)
!37 = !DILocation(line: 19, column: 5, scope: !17)
!38 = !DILocation(line: 20, column: 1, scope: !17)
