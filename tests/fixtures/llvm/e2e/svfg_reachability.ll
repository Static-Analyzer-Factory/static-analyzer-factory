; ModuleID = '/workspace/tests/programs/c/svfg_reachability.c'
source_filename = "/workspace/tests/programs/c/svfg_reachability.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test() #0 !dbg !10 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !15, metadata !DIExpression()), !dbg !17
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !20
  store ptr %1, ptr %2, align 8, !dbg !20
  call void @llvm.dbg.declare(metadata ptr %3, metadata !21, metadata !DIExpression()), !dbg !22
  %5 = call i32 @source(), !dbg !23
  store i32 %5, ptr %3, align 4, !dbg !22
  %6 = load i32, ptr %3, align 4, !dbg !24
  %7 = load ptr, ptr %2, align 8, !dbg !25
  store i32 %6, ptr %7, align 4, !dbg !26
  call void @llvm.dbg.declare(metadata ptr %4, metadata !27, metadata !DIExpression()), !dbg !28
  %8 = load ptr, ptr %2, align 8, !dbg !29
  %9 = load i32, ptr %8, align 4, !dbg !30
  store i32 %9, ptr %4, align 4, !dbg !28
  %10 = load i32, ptr %4, align 4, !dbg !31
  call void @sink(i32 noundef %10), !dbg !32
  ret void, !dbg !33
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @source() #2

declare void @sink(i32 noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/svfg_reachability.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "2c717ca40588065773a35c5de5440bce")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 7, type: !12, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DIFile(filename: "c/svfg_reachability.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "2c717ca40588065773a35c5de5440bce")
!12 = !DISubroutineType(types: !13)
!13 = !{null}
!14 = !{}
!15 = !DILocalVariable(name: "buf", scope: !10, file: !11, line: 8, type: !16)
!16 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!17 = !DILocation(line: 8, column: 9, scope: !10)
!18 = !DILocalVariable(name: "ptr", scope: !10, file: !11, line: 9, type: !19)
!19 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!20 = !DILocation(line: 9, column: 10, scope: !10)
!21 = !DILocalVariable(name: "tainted", scope: !10, file: !11, line: 11, type: !16)
!22 = !DILocation(line: 11, column: 9, scope: !10)
!23 = !DILocation(line: 11, column: 19, scope: !10)
!24 = !DILocation(line: 12, column: 12, scope: !10)
!25 = !DILocation(line: 12, column: 6, scope: !10)
!26 = !DILocation(line: 12, column: 10, scope: !10)
!27 = !DILocalVariable(name: "result", scope: !10, file: !11, line: 13, type: !16)
!28 = !DILocation(line: 13, column: 9, scope: !10)
!29 = !DILocation(line: 13, column: 19, scope: !10)
!30 = !DILocation(line: 13, column: 18, scope: !10)
!31 = !DILocation(line: 14, column: 10, scope: !10)
!32 = !DILocation(line: 14, column: 5, scope: !10)
!33 = !DILocation(line: 15, column: 1, scope: !10)
