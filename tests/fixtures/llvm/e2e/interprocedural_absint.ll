; ModuleID = '/workspace/tests/programs/c/interprocedural_absint.c'
source_filename = "/workspace/tests/programs/c/interprocedural_absint.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @return_constant() local_unnamed_addr #0 !dbg !11 {
  ret i32 42, !dbg !16
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @identity(i32 noundef returned %0) local_unnamed_addr #0 !dbg !17 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !21, metadata !DIExpression()), !dbg !22
  ret i32 %0, !dbg !23
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @add_one(i32 noundef %0) local_unnamed_addr #0 !dbg !24 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !26, metadata !DIExpression()), !dbg !27
  %2 = add nsw i32 %0, 1, !dbg !28
  ret i32 %2, !dbg !29
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @clamp_to_100(i32 noundef %0) local_unnamed_addr #0 !dbg !30 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !32, metadata !DIExpression()), !dbg !33
  %2 = tail call i32 @llvm.smin.i32(i32 %0, i32 100), !dbg !34
  %3 = tail call i32 @llvm.smax.i32(i32 %2, i32 0), !dbg !34
  ret i32 %3, !dbg !35
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @test_constant_return() local_unnamed_addr #0 !dbg !36 {
  tail call void @llvm.dbg.value(metadata i32 42, metadata !38, metadata !DIExpression()), !dbg !39
  ret i32 42, !dbg !40
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @test_identity_call() local_unnamed_addr #0 !dbg !41 {
  tail call void @llvm.dbg.value(metadata i32 10, metadata !43, metadata !DIExpression()), !dbg !44
  ret i32 10, !dbg !45
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @test_add_one_call() local_unnamed_addr #0 !dbg !46 {
  %1 = tail call i32 @add_one(i32 noundef 10), !dbg !49
  tail call void @llvm.dbg.value(metadata i32 %1, metadata !48, metadata !DIExpression()), !dbg !50
  ret i32 %1, !dbg !51
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @test_clamp_call(i32 noundef %0) local_unnamed_addr #0 !dbg !52 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !54, metadata !DIExpression()), !dbg !56
  %2 = tail call i32 @clamp_to_100(i32 noundef %0), !dbg !57, !range !58
  tail call void @llvm.dbg.value(metadata i32 %2, metadata !55, metadata !DIExpression()), !dbg !56
  ret i32 %2, !dbg !59
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @test_chained_calls() local_unnamed_addr #0 !dbg !60 {
  tail call void @llvm.dbg.value(metadata i32 42, metadata !62, metadata !DIExpression()), !dbg !65
  %1 = tail call i32 @add_one(i32 noundef 42), !dbg !66
  tail call void @llvm.dbg.value(metadata i32 %1, metadata !63, metadata !DIExpression()), !dbg !65
  tail call void @llvm.dbg.value(metadata i32 %1, metadata !64, metadata !DIExpression()), !dbg !65
  ret i32 %1, !dbg !67
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @multi_caller_callee(i32 noundef %0) local_unnamed_addr #0 !dbg !68 {
  tail call void @llvm.dbg.value(metadata i32 %0, metadata !70, metadata !DIExpression()), !dbg !71
  %2 = shl nsw i32 %0, 1, !dbg !72
  ret i32 %2, !dbg !73
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @test_multi_caller_1() local_unnamed_addr #0 !dbg !74 {
  %1 = tail call i32 @multi_caller_callee(i32 noundef 5), !dbg !75
  ret i32 %1, !dbg !76
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local i32 @test_multi_caller_2() local_unnamed_addr #0 !dbg !77 {
  %1 = tail call i32 @multi_caller_callee(i32 noundef 10), !dbg !78
  ret i32 %1, !dbg !79
}

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable
define dso_local noundef i32 @main() local_unnamed_addr #0 !dbg !80 {
  ret i32 0, !dbg !81
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.smin.i32(i32, i32) #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.smax.i32(i32, i32) #1

attributes #0 = { mustprogress nofree noinline norecurse nosync nounwind willreturn memory(none) uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8, !9}
!llvm.ident = !{!10}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/interprocedural_absint.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "f474642ccb50baa16c0a68c5bc7fe953")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{i32 7, !"debug-info-assignment-tracking", i1 true}
!10 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!11 = distinct !DISubprogram(name: "return_constant", scope: !12, file: !12, line: 6, type: !13, scopeLine: 6, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0)
!12 = !DIFile(filename: "tests/programs/c/interprocedural_absint.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "f474642ccb50baa16c0a68c5bc7fe953")
!13 = !DISubroutineType(types: !14)
!14 = !{!15}
!15 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!16 = !DILocation(line: 7, column: 5, scope: !11)
!17 = distinct !DISubprogram(name: "identity", scope: !12, file: !12, line: 11, type: !18, scopeLine: 11, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{!15, !15}
!20 = !{!21}
!21 = !DILocalVariable(name: "x", arg: 1, scope: !17, file: !12, line: 11, type: !15)
!22 = !DILocation(line: 0, scope: !17)
!23 = !DILocation(line: 12, column: 5, scope: !17)
!24 = distinct !DISubprogram(name: "add_one", scope: !12, file: !12, line: 16, type: !18, scopeLine: 16, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !25)
!25 = !{!26}
!26 = !DILocalVariable(name: "x", arg: 1, scope: !24, file: !12, line: 16, type: !15)
!27 = !DILocation(line: 0, scope: !24)
!28 = !DILocation(line: 17, column: 14, scope: !24)
!29 = !DILocation(line: 17, column: 5, scope: !24)
!30 = distinct !DISubprogram(name: "clamp_to_100", scope: !12, file: !12, line: 21, type: !18, scopeLine: 21, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !31)
!31 = !{!32}
!32 = !DILocalVariable(name: "x", arg: 1, scope: !30, file: !12, line: 21, type: !15)
!33 = !DILocation(line: 0, scope: !30)
!34 = !DILocation(line: 22, column: 9, scope: !30)
!35 = !DILocation(line: 25, column: 1, scope: !30)
!36 = distinct !DISubprogram(name: "test_constant_return", scope: !12, file: !12, line: 28, type: !13, scopeLine: 28, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !37)
!37 = !{!38}
!38 = !DILocalVariable(name: "v", scope: !36, file: !12, line: 29, type: !15)
!39 = !DILocation(line: 0, scope: !36)
!40 = !DILocation(line: 31, column: 5, scope: !36)
!41 = distinct !DISubprogram(name: "test_identity_call", scope: !12, file: !12, line: 35, type: !13, scopeLine: 35, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !42)
!42 = !{!43}
!43 = !DILocalVariable(name: "v", scope: !41, file: !12, line: 36, type: !15)
!44 = !DILocation(line: 0, scope: !41)
!45 = !DILocation(line: 38, column: 5, scope: !41)
!46 = distinct !DISubprogram(name: "test_add_one_call", scope: !12, file: !12, line: 42, type: !13, scopeLine: 42, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !47)
!47 = !{!48}
!48 = !DILocalVariable(name: "v", scope: !46, file: !12, line: 43, type: !15)
!49 = !DILocation(line: 43, column: 13, scope: !46)
!50 = !DILocation(line: 0, scope: !46)
!51 = !DILocation(line: 45, column: 5, scope: !46)
!52 = distinct !DISubprogram(name: "test_clamp_call", scope: !12, file: !12, line: 49, type: !18, scopeLine: 49, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !53)
!53 = !{!54, !55}
!54 = !DILocalVariable(name: "input", arg: 1, scope: !52, file: !12, line: 49, type: !15)
!55 = !DILocalVariable(name: "v", scope: !52, file: !12, line: 50, type: !15)
!56 = !DILocation(line: 0, scope: !52)
!57 = !DILocation(line: 50, column: 13, scope: !52)
!58 = !{i32 0, i32 101}
!59 = !DILocation(line: 52, column: 5, scope: !52)
!60 = distinct !DISubprogram(name: "test_chained_calls", scope: !12, file: !12, line: 56, type: !13, scopeLine: 56, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !61)
!61 = !{!62, !63, !64}
!62 = !DILocalVariable(name: "a", scope: !60, file: !12, line: 57, type: !15)
!63 = !DILocalVariable(name: "b", scope: !60, file: !12, line: 58, type: !15)
!64 = !DILocalVariable(name: "c", scope: !60, file: !12, line: 59, type: !15)
!65 = !DILocation(line: 0, scope: !60)
!66 = !DILocation(line: 58, column: 13, scope: !60)
!67 = !DILocation(line: 60, column: 5, scope: !60)
!68 = distinct !DISubprogram(name: "multi_caller_callee", scope: !12, file: !12, line: 64, type: !18, scopeLine: 64, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !69)
!69 = !{!70}
!70 = !DILocalVariable(name: "x", arg: 1, scope: !68, file: !12, line: 64, type: !15)
!71 = !DILocation(line: 0, scope: !68)
!72 = !DILocation(line: 65, column: 14, scope: !68)
!73 = !DILocation(line: 65, column: 5, scope: !68)
!74 = distinct !DISubprogram(name: "test_multi_caller_1", scope: !12, file: !12, line: 68, type: !13, scopeLine: 68, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0)
!75 = !DILocation(line: 69, column: 12, scope: !74)
!76 = !DILocation(line: 69, column: 5, scope: !74)
!77 = distinct !DISubprogram(name: "test_multi_caller_2", scope: !12, file: !12, line: 72, type: !13, scopeLine: 72, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0)
!78 = !DILocation(line: 73, column: 12, scope: !77)
!79 = !DILocation(line: 73, column: 5, scope: !77)
!80 = distinct !DISubprogram(name: "main", scope: !12, file: !12, line: 76, type: !13, scopeLine: 76, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0)
!81 = !DILocation(line: 84, column: 5, scope: !80)
