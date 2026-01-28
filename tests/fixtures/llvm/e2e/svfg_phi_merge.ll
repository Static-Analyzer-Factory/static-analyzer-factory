; ModuleID = '/workspace/tests/programs/c/svfg_phi_merge.c'
source_filename = "/workspace/tests/programs/c/svfg_phi_merge.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test(i32 noundef %0) #0 !dbg !10 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !16, metadata !DIExpression()), !dbg !17
  call void @llvm.dbg.declare(metadata ptr %3, metadata !18, metadata !DIExpression()), !dbg !19
  call void @llvm.dbg.declare(metadata ptr %4, metadata !20, metadata !DIExpression()), !dbg !22
  store ptr %3, ptr %4, align 8, !dbg !22
  %6 = load i32, ptr %2, align 4, !dbg !23
  %7 = icmp ne i32 %6, 0, !dbg !23
  br i1 %7, label %8, label %11, !dbg !25

8:                                                ; preds = %1
  %9 = call i32 @source(), !dbg !26
  %10 = load ptr, ptr %4, align 8, !dbg !28
  store i32 %9, ptr %10, align 4, !dbg !29
  br label %13, !dbg !30

11:                                               ; preds = %1
  %12 = load ptr, ptr %4, align 8, !dbg !31
  store i32 0, ptr %12, align 4, !dbg !33
  br label %13

13:                                               ; preds = %11, %8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !34, metadata !DIExpression()), !dbg !35
  %14 = load ptr, ptr %4, align 8, !dbg !36
  %15 = load i32, ptr %14, align 4, !dbg !37
  store i32 %15, ptr %5, align 4, !dbg !35
  %16 = load i32, ptr %5, align 4, !dbg !38
  call void @sink(i32 noundef %16), !dbg !39
  ret void, !dbg !40
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
!1 = !DIFile(filename: "/workspace/tests/programs/c/svfg_phi_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "802b7b600012ee9adc04a9997956a67d")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 8, type: !12, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DIFile(filename: "c/svfg_phi_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "802b7b600012ee9adc04a9997956a67d")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "cond", arg: 1, scope: !10, file: !11, line: 8, type: !14)
!17 = !DILocation(line: 8, column: 15, scope: !10)
!18 = !DILocalVariable(name: "x", scope: !10, file: !11, line: 9, type: !14)
!19 = !DILocation(line: 9, column: 9, scope: !10)
!20 = !DILocalVariable(name: "p", scope: !10, file: !11, line: 10, type: !21)
!21 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!22 = !DILocation(line: 10, column: 10, scope: !10)
!23 = !DILocation(line: 12, column: 9, scope: !24)
!24 = distinct !DILexicalBlock(scope: !10, file: !11, line: 12, column: 9)
!25 = !DILocation(line: 12, column: 9, scope: !10)
!26 = !DILocation(line: 13, column: 14, scope: !27)
!27 = distinct !DILexicalBlock(scope: !24, file: !11, line: 12, column: 15)
!28 = !DILocation(line: 13, column: 10, scope: !27)
!29 = !DILocation(line: 13, column: 12, scope: !27)
!30 = !DILocation(line: 14, column: 5, scope: !27)
!31 = !DILocation(line: 15, column: 10, scope: !32)
!32 = distinct !DILexicalBlock(scope: !24, file: !11, line: 14, column: 12)
!33 = !DILocation(line: 15, column: 12, scope: !32)
!34 = !DILocalVariable(name: "val", scope: !10, file: !11, line: 18, type: !14)
!35 = !DILocation(line: 18, column: 9, scope: !10)
!36 = !DILocation(line: 18, column: 16, scope: !10)
!37 = !DILocation(line: 18, column: 15, scope: !10)
!38 = !DILocation(line: 19, column: 10, scope: !10)
!39 = !DILocation(line: 19, column: 5, scope: !10)
!40 = !DILocation(line: 20, column: 1, scope: !10)
