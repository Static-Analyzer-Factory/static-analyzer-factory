; ModuleID = '/workspace/tests/programs/c/fspta_loop_weak_update.c'
source_filename = "/workspace/tests/programs/c/fspta_loop_weak_update.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@g_a = dso_local global i32 0, align 4, !dbg !0
@g_b = dso_local global i32 0, align 4, !dbg !5
@g_c = dso_local global i32 0, align 4, !dbg !9

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_loop_weak_update() #0 !dbg !19 {
  %1 = alloca [3 x ptr], align 8
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !23, metadata !DIExpression()), !dbg !28
  %3 = getelementptr inbounds [3 x ptr], ptr %1, i64 0, i64 0, !dbg !29
  store ptr @g_a, ptr %3, align 8, !dbg !30
  %4 = getelementptr inbounds [3 x ptr], ptr %1, i64 0, i64 1, !dbg !31
  store ptr @g_b, ptr %4, align 8, !dbg !32
  %5 = getelementptr inbounds [3 x ptr], ptr %1, i64 0, i64 2, !dbg !33
  store ptr @g_c, ptr %5, align 8, !dbg !34
  call void @llvm.dbg.declare(metadata ptr %2, metadata !35, metadata !DIExpression()), !dbg !37
  store i32 0, ptr %2, align 4, !dbg !37
  br label %6, !dbg !38

6:                                                ; preds = %16, %0
  %7 = load i32, ptr %2, align 4, !dbg !39
  %8 = icmp slt i32 %7, 3, !dbg !41
  br i1 %8, label %9, label %19, !dbg !42

9:                                                ; preds = %6
  %10 = load i32, ptr %2, align 4, !dbg !43
  %11 = mul nsw i32 %10, 10, !dbg !45
  %12 = load i32, ptr %2, align 4, !dbg !46
  %13 = sext i32 %12 to i64, !dbg !47
  %14 = getelementptr inbounds [3 x ptr], ptr %1, i64 0, i64 %13, !dbg !47
  %15 = load ptr, ptr %14, align 8, !dbg !47
  store i32 %11, ptr %15, align 4, !dbg !48
  br label %16, !dbg !49

16:                                               ; preds = %9
  %17 = load i32, ptr %2, align 4, !dbg !50
  %18 = add nsw i32 %17, 1, !dbg !50
  store i32 %18, ptr %2, align 4, !dbg !50
  br label %6, !dbg !51, !llvm.loop !52

19:                                               ; preds = %6
  ret void, !dbg !55
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !56 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_loop_weak_update(), !dbg !59
  ret i32 0, !dbg !60
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!11, !12, !13, !14, !15, !16, !17}
!llvm.ident = !{!18}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "g_a", scope: !2, file: !7, line: 9, type: !8, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "/workspace/tests/programs/c/fspta_loop_weak_update.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4476b16371caa555d8834079b4e44745")
!4 = !{!0, !5, !9}
!5 = !DIGlobalVariableExpression(var: !6, expr: !DIExpression())
!6 = distinct !DIGlobalVariable(name: "g_b", scope: !2, file: !7, line: 9, type: !8, isLocal: false, isDefinition: true)
!7 = !DIFile(filename: "c/fspta_loop_weak_update.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4476b16371caa555d8834079b4e44745")
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !DIGlobalVariableExpression(var: !10, expr: !DIExpression())
!10 = distinct !DIGlobalVariable(name: "g_c", scope: !2, file: !7, line: 9, type: !8, isLocal: false, isDefinition: true)
!11 = !{i32 7, !"Dwarf Version", i32 5}
!12 = !{i32 2, !"Debug Info Version", i32 3}
!13 = !{i32 1, !"wchar_size", i32 4}
!14 = !{i32 8, !"PIC Level", i32 2}
!15 = !{i32 7, !"PIE Level", i32 2}
!16 = !{i32 7, !"uwtable", i32 2}
!17 = !{i32 7, !"frame-pointer", i32 1}
!18 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!19 = distinct !DISubprogram(name: "test_loop_weak_update", scope: !7, file: !7, line: 11, type: !20, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !22)
!20 = !DISubroutineType(types: !21)
!21 = !{null}
!22 = !{}
!23 = !DILocalVariable(name: "arr", scope: !19, file: !7, line: 12, type: !24)
!24 = !DICompositeType(tag: DW_TAG_array_type, baseType: !25, size: 192, elements: !26)
!25 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !8, size: 64)
!26 = !{!27}
!27 = !DISubrange(count: 3)
!28 = !DILocation(line: 12, column: 10, scope: !19)
!29 = !DILocation(line: 13, column: 5, scope: !19)
!30 = !DILocation(line: 13, column: 12, scope: !19)
!31 = !DILocation(line: 14, column: 5, scope: !19)
!32 = !DILocation(line: 14, column: 12, scope: !19)
!33 = !DILocation(line: 15, column: 5, scope: !19)
!34 = !DILocation(line: 15, column: 12, scope: !19)
!35 = !DILocalVariable(name: "i", scope: !36, file: !7, line: 17, type: !8)
!36 = distinct !DILexicalBlock(scope: !19, file: !7, line: 17, column: 5)
!37 = !DILocation(line: 17, column: 14, scope: !36)
!38 = !DILocation(line: 17, column: 10, scope: !36)
!39 = !DILocation(line: 17, column: 21, scope: !40)
!40 = distinct !DILexicalBlock(scope: !36, file: !7, line: 17, column: 5)
!41 = !DILocation(line: 17, column: 23, scope: !40)
!42 = !DILocation(line: 17, column: 5, scope: !36)
!43 = !DILocation(line: 20, column: 19, scope: !44)
!44 = distinct !DILexicalBlock(scope: !40, file: !7, line: 17, column: 33)
!45 = !DILocation(line: 20, column: 21, scope: !44)
!46 = !DILocation(line: 20, column: 14, scope: !44)
!47 = !DILocation(line: 20, column: 10, scope: !44)
!48 = !DILocation(line: 20, column: 17, scope: !44)
!49 = !DILocation(line: 21, column: 5, scope: !44)
!50 = !DILocation(line: 17, column: 29, scope: !40)
!51 = !DILocation(line: 17, column: 5, scope: !40)
!52 = distinct !{!52, !42, !53, !54}
!53 = !DILocation(line: 21, column: 5, scope: !36)
!54 = !{!"llvm.loop.mustprogress"}
!55 = !DILocation(line: 22, column: 1, scope: !19)
!56 = distinct !DISubprogram(name: "main", scope: !7, file: !7, line: 24, type: !57, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!57 = !DISubroutineType(types: !58)
!58 = !{!8}
!59 = !DILocation(line: 25, column: 5, scope: !56)
!60 = !DILocation(line: 26, column: 5, scope: !56)
