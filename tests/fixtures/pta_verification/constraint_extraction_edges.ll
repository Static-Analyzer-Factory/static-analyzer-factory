; ModuleID = 'tests/fixtures/pta_verification/constraint_extraction_edges.c'
source_filename = "tests/fixtures/pta_verification/constraint_extraction_edges.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Point = type { i32, i32 }

@global_x = dso_local global i32 0, align 4, !dbg !0
@global_ptr = dso_local global ptr null, align 8, !dbg !9

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @target_func1() #0 !dbg !19 {
  ret void, !dbg !22
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @target_func2() #0 !dbg !23 {
  ret void, !dbg !24
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_basic_store_load() #0 !dbg !25 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !27, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %2, metadata !29, metadata !DIExpression()), !dbg !30
  store ptr %1, ptr %2, align 8, !dbg !30
  %4 = load ptr, ptr %2, align 8, !dbg !31
  store i32 42, ptr %4, align 4, !dbg !32
  call void @llvm.dbg.declare(metadata ptr %3, metadata !33, metadata !DIExpression()), !dbg !34
  %5 = load ptr, ptr %2, align 8, !dbg !35
  %6 = load i32, ptr %5, align 4, !dbg !36
  store i32 %6, ptr %3, align 4, !dbg !34
  %7 = load i32, ptr %3, align 4, !dbg !37
  ret void, !dbg !38
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_copy_chain() #0 !dbg !39 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !40, metadata !DIExpression()), !dbg !41
  call void @llvm.dbg.declare(metadata ptr %2, metadata !42, metadata !DIExpression()), !dbg !43
  store ptr %1, ptr %2, align 8, !dbg !43
  call void @llvm.dbg.declare(metadata ptr %3, metadata !44, metadata !DIExpression()), !dbg !45
  %6 = load ptr, ptr %2, align 8, !dbg !46
  store ptr %6, ptr %3, align 8, !dbg !45
  call void @llvm.dbg.declare(metadata ptr %4, metadata !47, metadata !DIExpression()), !dbg !48
  %7 = load ptr, ptr %3, align 8, !dbg !49
  store ptr %7, ptr %4, align 8, !dbg !48
  call void @llvm.dbg.declare(metadata ptr %5, metadata !50, metadata !DIExpression()), !dbg !51
  %8 = load ptr, ptr %4, align 8, !dbg !52
  store ptr %8, ptr %5, align 8, !dbg !51
  %9 = load ptr, ptr %5, align 8, !dbg !53
  ret void, !dbg !54
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_phi_merge(i32 noundef %0) #0 !dbg !55 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !58, metadata !DIExpression()), !dbg !59
  call void @llvm.dbg.declare(metadata ptr %3, metadata !60, metadata !DIExpression()), !dbg !61
  call void @llvm.dbg.declare(metadata ptr %4, metadata !62, metadata !DIExpression()), !dbg !63
  call void @llvm.dbg.declare(metadata ptr %5, metadata !64, metadata !DIExpression()), !dbg !65
  %7 = load i32, ptr %2, align 4, !dbg !66
  %8 = icmp ne i32 %7, 0, !dbg !66
  br i1 %8, label %9, label %10, !dbg !68

9:                                                ; preds = %1
  store ptr %3, ptr %5, align 8, !dbg !69
  br label %11, !dbg !71

10:                                               ; preds = %1
  store ptr %4, ptr %5, align 8, !dbg !72
  br label %11

11:                                               ; preds = %10, %9
  call void @llvm.dbg.declare(metadata ptr %6, metadata !74, metadata !DIExpression()), !dbg !75
  %12 = load ptr, ptr %5, align 8, !dbg !76
  %13 = load i32, ptr %12, align 4, !dbg !77
  store i32 %13, ptr %6, align 4, !dbg !75
  %14 = load i32, ptr %6, align 4, !dbg !78
  ret void, !dbg !79
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_select(i32 noundef %0) #0 !dbg !80 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !81, metadata !DIExpression()), !dbg !82
  call void @llvm.dbg.declare(metadata ptr %3, metadata !83, metadata !DIExpression()), !dbg !84
  call void @llvm.dbg.declare(metadata ptr %4, metadata !85, metadata !DIExpression()), !dbg !86
  call void @llvm.dbg.declare(metadata ptr %5, metadata !87, metadata !DIExpression()), !dbg !88
  %7 = load i32, ptr %2, align 4, !dbg !89
  %8 = icmp ne i32 %7, 0, !dbg !89
  br i1 %8, label %9, label %10, !dbg !89

9:                                                ; preds = %1
  br label %11, !dbg !89

10:                                               ; preds = %1
  br label %11, !dbg !89

11:                                               ; preds = %10, %9
  %12 = phi ptr [ %3, %9 ], [ %4, %10 ], !dbg !89
  store ptr %12, ptr %5, align 8, !dbg !88
  call void @llvm.dbg.declare(metadata ptr %6, metadata !90, metadata !DIExpression()), !dbg !91
  %13 = load ptr, ptr %5, align 8, !dbg !92
  %14 = load i32, ptr %13, align 4, !dbg !93
  store i32 %14, ptr %6, align 4, !dbg !91
  %15 = load i32, ptr %6, align 4, !dbg !94
  ret void, !dbg !95
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_gep_field() #0 !dbg !96 {
  %1 = alloca %struct.Point, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !97, metadata !DIExpression()), !dbg !102
  call void @llvm.dbg.declare(metadata ptr %2, metadata !103, metadata !DIExpression()), !dbg !104
  %4 = getelementptr inbounds %struct.Point, ptr %1, i32 0, i32 0, !dbg !105
  store ptr %4, ptr %2, align 8, !dbg !104
  call void @llvm.dbg.declare(metadata ptr %3, metadata !106, metadata !DIExpression()), !dbg !107
  %5 = getelementptr inbounds %struct.Point, ptr %1, i32 0, i32 1, !dbg !108
  store ptr %5, ptr %3, align 8, !dbg !107
  %6 = load ptr, ptr %2, align 8, !dbg !109
  store i32 1, ptr %6, align 4, !dbg !110
  %7 = load ptr, ptr %3, align 8, !dbg !111
  store i32 2, ptr %7, align 4, !dbg !112
  ret void, !dbg !113
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_gep_array() #0 !dbg !114 {
  %1 = alloca [10 x i32], align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !115, metadata !DIExpression()), !dbg !119
  call void @llvm.dbg.declare(metadata ptr %2, metadata !120, metadata !DIExpression()), !dbg !121
  %4 = getelementptr inbounds [10 x i32], ptr %1, i64 0, i64 0, !dbg !122
  store ptr %4, ptr %2, align 8, !dbg !121
  call void @llvm.dbg.declare(metadata ptr %3, metadata !123, metadata !DIExpression()), !dbg !124
  %5 = getelementptr inbounds [10 x i32], ptr %1, i64 0, i64 5, !dbg !125
  store ptr %5, ptr %3, align 8, !dbg !124
  %6 = load ptr, ptr %2, align 8, !dbg !126
  store i32 1, ptr %6, align 4, !dbg !127
  %7 = load ptr, ptr %3, align 8, !dbg !128
  store i32 2, ptr %7, align 4, !dbg !129
  ret void, !dbg !130
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_heap_alloc() #0 !dbg !131 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !132, metadata !DIExpression()), !dbg !133
  %2 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !134
  store ptr %2, ptr %1, align 8, !dbg !133
  %3 = load ptr, ptr %1, align 8, !dbg !135
  store i32 42, ptr %3, align 4, !dbg !136
  %4 = load ptr, ptr %1, align 8, !dbg !137
  call void @free(ptr noundef %4) #5, !dbg !138
  ret void, !dbg !139
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_global() #0 !dbg !140 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !141, metadata !DIExpression()), !dbg !142
  store ptr @global_x, ptr %1, align 8, !dbg !142
  %3 = load ptr, ptr %1, align 8, !dbg !143
  store ptr %3, ptr @global_ptr, align 8, !dbg !144
  call void @llvm.dbg.declare(metadata ptr %2, metadata !145, metadata !DIExpression()), !dbg !146
  %4 = load ptr, ptr @global_ptr, align 8, !dbg !147
  store ptr %4, ptr %2, align 8, !dbg !146
  %5 = load ptr, ptr %2, align 8, !dbg !148
  ret void, !dbg !149
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_cast() #0 !dbg !150 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !151, metadata !DIExpression()), !dbg !152
  call void @llvm.dbg.declare(metadata ptr %2, metadata !153, metadata !DIExpression()), !dbg !154
  store ptr %1, ptr %2, align 8, !dbg !154
  call void @llvm.dbg.declare(metadata ptr %3, metadata !155, metadata !DIExpression()), !dbg !156
  %4 = load ptr, ptr %2, align 8, !dbg !157
  store ptr %4, ptr %3, align 8, !dbg !156
  %5 = load ptr, ptr %3, align 8, !dbg !158
  store i32 42, ptr %5, align 4, !dbg !159
  ret void, !dbg !160
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @identity(ptr noundef %0) #0 !dbg !161 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !164, metadata !DIExpression()), !dbg !165
  %3 = load ptr, ptr %2, align 8, !dbg !166
  ret ptr %3, !dbg !167
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_interprocedural() #0 !dbg !168 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !169, metadata !DIExpression()), !dbg !170
  call void @llvm.dbg.declare(metadata ptr %2, metadata !171, metadata !DIExpression()), !dbg !172
  store ptr %1, ptr %2, align 8, !dbg !172
  call void @llvm.dbg.declare(metadata ptr %3, metadata !173, metadata !DIExpression()), !dbg !174
  %4 = load ptr, ptr %2, align 8, !dbg !175
  %5 = call ptr @identity(ptr noundef %4), !dbg !176
  store ptr %5, ptr %3, align 8, !dbg !174
  %6 = load ptr, ptr %3, align 8, !dbg !177
  ret void, !dbg !178
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_func_ptr(ptr noundef %0) #0 !dbg !179 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !184, metadata !DIExpression()), !dbg !185
  %3 = load ptr, ptr %2, align 8, !dbg !186
  call void %3(), !dbg !186
  ret void, !dbg !187
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !188 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_basic_store_load(), !dbg !191
  call void @test_copy_chain(), !dbg !192
  call void @test_phi_merge(i32 noundef 1), !dbg !193
  call void @test_select(i32 noundef 1), !dbg !194
  call void @test_gep_field(), !dbg !195
  call void @test_gep_array(), !dbg !196
  call void @test_heap_alloc(), !dbg !197
  call void @test_global(), !dbg !198
  call void @test_cast(), !dbg !199
  call void @test_interprocedural(), !dbg !200
  call void @test_func_ptr(ptr noundef @target_func1), !dbg !201
  ret i32 0, !dbg !202
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!11, !12, !13, !14, !15, !16, !17}
!llvm.ident = !{!18}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "global_x", scope: !2, file: !3, line: 7, type: !6, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !4, globals: !8, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "tests/fixtures/pta_verification/constraint_extraction_edges.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "31169644551d95180f7fbfb90359bf45")
!4 = !{!5, !7}
!5 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !6, size: 64)
!6 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!7 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!8 = !{!0, !9}
!9 = !DIGlobalVariableExpression(var: !10, expr: !DIExpression())
!10 = distinct !DIGlobalVariable(name: "global_ptr", scope: !2, file: !3, line: 8, type: !5, isLocal: false, isDefinition: true)
!11 = !{i32 7, !"Dwarf Version", i32 5}
!12 = !{i32 2, !"Debug Info Version", i32 3}
!13 = !{i32 1, !"wchar_size", i32 4}
!14 = !{i32 8, !"PIC Level", i32 2}
!15 = !{i32 7, !"PIE Level", i32 2}
!16 = !{i32 7, !"uwtable", i32 2}
!17 = !{i32 7, !"frame-pointer", i32 1}
!18 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!19 = distinct !DISubprogram(name: "target_func1", scope: !3, file: !3, line: 13, type: !20, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!20 = !DISubroutineType(types: !21)
!21 = !{null}
!22 = !DILocation(line: 13, column: 26, scope: !19)
!23 = distinct !DISubprogram(name: "target_func2", scope: !3, file: !3, line: 14, type: !20, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!24 = !DILocation(line: 14, column: 26, scope: !23)
!25 = distinct !DISubprogram(name: "test_basic_store_load", scope: !3, file: !3, line: 23, type: !20, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!26 = !{}
!27 = !DILocalVariable(name: "x", scope: !25, file: !3, line: 24, type: !6)
!28 = !DILocation(line: 24, column: 9, scope: !25)
!29 = !DILocalVariable(name: "p", scope: !25, file: !3, line: 25, type: !5)
!30 = !DILocation(line: 25, column: 10, scope: !25)
!31 = !DILocation(line: 26, column: 6, scope: !25)
!32 = !DILocation(line: 26, column: 8, scope: !25)
!33 = !DILocalVariable(name: "y", scope: !25, file: !3, line: 27, type: !6)
!34 = !DILocation(line: 27, column: 9, scope: !25)
!35 = !DILocation(line: 27, column: 14, scope: !25)
!36 = !DILocation(line: 27, column: 13, scope: !25)
!37 = !DILocation(line: 28, column: 11, scope: !25)
!38 = !DILocation(line: 29, column: 1, scope: !25)
!39 = distinct !DISubprogram(name: "test_copy_chain", scope: !3, file: !3, line: 32, type: !20, scopeLine: 32, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!40 = !DILocalVariable(name: "x", scope: !39, file: !3, line: 33, type: !6)
!41 = !DILocation(line: 33, column: 9, scope: !39)
!42 = !DILocalVariable(name: "a", scope: !39, file: !3, line: 34, type: !5)
!43 = !DILocation(line: 34, column: 10, scope: !39)
!44 = !DILocalVariable(name: "b", scope: !39, file: !3, line: 35, type: !5)
!45 = !DILocation(line: 35, column: 10, scope: !39)
!46 = !DILocation(line: 35, column: 14, scope: !39)
!47 = !DILocalVariable(name: "c", scope: !39, file: !3, line: 36, type: !5)
!48 = !DILocation(line: 36, column: 10, scope: !39)
!49 = !DILocation(line: 36, column: 14, scope: !39)
!50 = !DILocalVariable(name: "d", scope: !39, file: !3, line: 37, type: !5)
!51 = !DILocation(line: 37, column: 10, scope: !39)
!52 = !DILocation(line: 37, column: 14, scope: !39)
!53 = !DILocation(line: 38, column: 11, scope: !39)
!54 = !DILocation(line: 39, column: 1, scope: !39)
!55 = distinct !DISubprogram(name: "test_phi_merge", scope: !3, file: !3, line: 42, type: !56, scopeLine: 42, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!56 = !DISubroutineType(types: !57)
!57 = !{null, !6}
!58 = !DILocalVariable(name: "cond", arg: 1, scope: !55, file: !3, line: 42, type: !6)
!59 = !DILocation(line: 42, column: 25, scope: !55)
!60 = !DILocalVariable(name: "x", scope: !55, file: !3, line: 43, type: !6)
!61 = !DILocation(line: 43, column: 9, scope: !55)
!62 = !DILocalVariable(name: "y", scope: !55, file: !3, line: 43, type: !6)
!63 = !DILocation(line: 43, column: 12, scope: !55)
!64 = !DILocalVariable(name: "p", scope: !55, file: !3, line: 44, type: !5)
!65 = !DILocation(line: 44, column: 10, scope: !55)
!66 = !DILocation(line: 45, column: 9, scope: !67)
!67 = distinct !DILexicalBlock(scope: !55, file: !3, line: 45, column: 9)
!68 = !DILocation(line: 45, column: 9, scope: !55)
!69 = !DILocation(line: 46, column: 11, scope: !70)
!70 = distinct !DILexicalBlock(scope: !67, file: !3, line: 45, column: 15)
!71 = !DILocation(line: 47, column: 5, scope: !70)
!72 = !DILocation(line: 48, column: 11, scope: !73)
!73 = distinct !DILexicalBlock(scope: !67, file: !3, line: 47, column: 12)
!74 = !DILocalVariable(name: "z", scope: !55, file: !3, line: 51, type: !6)
!75 = !DILocation(line: 51, column: 9, scope: !55)
!76 = !DILocation(line: 51, column: 14, scope: !55)
!77 = !DILocation(line: 51, column: 13, scope: !55)
!78 = !DILocation(line: 52, column: 11, scope: !55)
!79 = !DILocation(line: 53, column: 1, scope: !55)
!80 = distinct !DISubprogram(name: "test_select", scope: !3, file: !3, line: 56, type: !56, scopeLine: 56, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!81 = !DILocalVariable(name: "cond", arg: 1, scope: !80, file: !3, line: 56, type: !6)
!82 = !DILocation(line: 56, column: 22, scope: !80)
!83 = !DILocalVariable(name: "x", scope: !80, file: !3, line: 57, type: !6)
!84 = !DILocation(line: 57, column: 9, scope: !80)
!85 = !DILocalVariable(name: "y", scope: !80, file: !3, line: 57, type: !6)
!86 = !DILocation(line: 57, column: 12, scope: !80)
!87 = !DILocalVariable(name: "p", scope: !80, file: !3, line: 58, type: !5)
!88 = !DILocation(line: 58, column: 10, scope: !80)
!89 = !DILocation(line: 58, column: 14, scope: !80)
!90 = !DILocalVariable(name: "z", scope: !80, file: !3, line: 59, type: !6)
!91 = !DILocation(line: 59, column: 9, scope: !80)
!92 = !DILocation(line: 59, column: 14, scope: !80)
!93 = !DILocation(line: 59, column: 13, scope: !80)
!94 = !DILocation(line: 60, column: 11, scope: !80)
!95 = !DILocation(line: 61, column: 1, scope: !80)
!96 = distinct !DISubprogram(name: "test_gep_field", scope: !3, file: !3, line: 69, type: !20, scopeLine: 69, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!97 = !DILocalVariable(name: "pt", scope: !96, file: !3, line: 70, type: !98)
!98 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Point", file: !3, line: 64, size: 64, elements: !99)
!99 = !{!100, !101}
!100 = !DIDerivedType(tag: DW_TAG_member, name: "x", scope: !98, file: !3, line: 65, baseType: !6, size: 32)
!101 = !DIDerivedType(tag: DW_TAG_member, name: "y", scope: !98, file: !3, line: 66, baseType: !6, size: 32, offset: 32)
!102 = !DILocation(line: 70, column: 18, scope: !96)
!103 = !DILocalVariable(name: "px", scope: !96, file: !3, line: 71, type: !5)
!104 = !DILocation(line: 71, column: 10, scope: !96)
!105 = !DILocation(line: 71, column: 19, scope: !96)
!106 = !DILocalVariable(name: "py", scope: !96, file: !3, line: 72, type: !5)
!107 = !DILocation(line: 72, column: 10, scope: !96)
!108 = !DILocation(line: 72, column: 19, scope: !96)
!109 = !DILocation(line: 73, column: 6, scope: !96)
!110 = !DILocation(line: 73, column: 9, scope: !96)
!111 = !DILocation(line: 74, column: 6, scope: !96)
!112 = !DILocation(line: 74, column: 9, scope: !96)
!113 = !DILocation(line: 75, column: 1, scope: !96)
!114 = distinct !DISubprogram(name: "test_gep_array", scope: !3, file: !3, line: 78, type: !20, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!115 = !DILocalVariable(name: "arr", scope: !114, file: !3, line: 79, type: !116)
!116 = !DICompositeType(tag: DW_TAG_array_type, baseType: !6, size: 320, elements: !117)
!117 = !{!118}
!118 = !DISubrange(count: 10)
!119 = !DILocation(line: 79, column: 9, scope: !114)
!120 = !DILocalVariable(name: "p", scope: !114, file: !3, line: 80, type: !5)
!121 = !DILocation(line: 80, column: 10, scope: !114)
!122 = !DILocation(line: 80, column: 15, scope: !114)
!123 = !DILocalVariable(name: "q", scope: !114, file: !3, line: 81, type: !5)
!124 = !DILocation(line: 81, column: 10, scope: !114)
!125 = !DILocation(line: 81, column: 15, scope: !114)
!126 = !DILocation(line: 82, column: 6, scope: !114)
!127 = !DILocation(line: 82, column: 8, scope: !114)
!128 = !DILocation(line: 83, column: 6, scope: !114)
!129 = !DILocation(line: 83, column: 8, scope: !114)
!130 = !DILocation(line: 84, column: 1, scope: !114)
!131 = distinct !DISubprogram(name: "test_heap_alloc", scope: !3, file: !3, line: 87, type: !20, scopeLine: 87, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!132 = !DILocalVariable(name: "p", scope: !131, file: !3, line: 88, type: !5)
!133 = !DILocation(line: 88, column: 10, scope: !131)
!134 = !DILocation(line: 88, column: 20, scope: !131)
!135 = !DILocation(line: 89, column: 6, scope: !131)
!136 = !DILocation(line: 89, column: 8, scope: !131)
!137 = !DILocation(line: 90, column: 10, scope: !131)
!138 = !DILocation(line: 90, column: 5, scope: !131)
!139 = !DILocation(line: 91, column: 1, scope: !131)
!140 = distinct !DISubprogram(name: "test_global", scope: !3, file: !3, line: 94, type: !20, scopeLine: 94, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!141 = !DILocalVariable(name: "p", scope: !140, file: !3, line: 95, type: !5)
!142 = !DILocation(line: 95, column: 10, scope: !140)
!143 = !DILocation(line: 96, column: 18, scope: !140)
!144 = !DILocation(line: 96, column: 16, scope: !140)
!145 = !DILocalVariable(name: "q", scope: !140, file: !3, line: 97, type: !5)
!146 = !DILocation(line: 97, column: 10, scope: !140)
!147 = !DILocation(line: 97, column: 14, scope: !140)
!148 = !DILocation(line: 98, column: 11, scope: !140)
!149 = !DILocation(line: 99, column: 1, scope: !140)
!150 = distinct !DISubprogram(name: "test_cast", scope: !3, file: !3, line: 102, type: !20, scopeLine: 102, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!151 = !DILocalVariable(name: "x", scope: !150, file: !3, line: 103, type: !6)
!152 = !DILocation(line: 103, column: 9, scope: !150)
!153 = !DILocalVariable(name: "v", scope: !150, file: !3, line: 104, type: !7)
!154 = !DILocation(line: 104, column: 11, scope: !150)
!155 = !DILocalVariable(name: "p", scope: !150, file: !3, line: 105, type: !5)
!156 = !DILocation(line: 105, column: 10, scope: !150)
!157 = !DILocation(line: 105, column: 20, scope: !150)
!158 = !DILocation(line: 106, column: 6, scope: !150)
!159 = !DILocation(line: 106, column: 8, scope: !150)
!160 = !DILocation(line: 107, column: 1, scope: !150)
!161 = distinct !DISubprogram(name: "identity", scope: !3, file: !3, line: 110, type: !162, scopeLine: 110, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!162 = !DISubroutineType(types: !163)
!163 = !{!5, !5}
!164 = !DILocalVariable(name: "p", arg: 1, scope: !161, file: !3, line: 110, type: !5)
!165 = !DILocation(line: 110, column: 20, scope: !161)
!166 = !DILocation(line: 111, column: 12, scope: !161)
!167 = !DILocation(line: 111, column: 5, scope: !161)
!168 = distinct !DISubprogram(name: "test_interprocedural", scope: !3, file: !3, line: 114, type: !20, scopeLine: 114, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!169 = !DILocalVariable(name: "x", scope: !168, file: !3, line: 115, type: !6)
!170 = !DILocation(line: 115, column: 9, scope: !168)
!171 = !DILocalVariable(name: "a", scope: !168, file: !3, line: 116, type: !5)
!172 = !DILocation(line: 116, column: 10, scope: !168)
!173 = !DILocalVariable(name: "b", scope: !168, file: !3, line: 117, type: !5)
!174 = !DILocation(line: 117, column: 10, scope: !168)
!175 = !DILocation(line: 117, column: 23, scope: !168)
!176 = !DILocation(line: 117, column: 14, scope: !168)
!177 = !DILocation(line: 119, column: 11, scope: !168)
!178 = !DILocation(line: 120, column: 1, scope: !168)
!179 = distinct !DISubprogram(name: "test_func_ptr", scope: !3, file: !3, line: 123, type: !180, scopeLine: 123, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !26)
!180 = !DISubroutineType(types: !181)
!181 = !{null, !182}
!182 = !DIDerivedType(tag: DW_TAG_typedef, name: "func_ptr_t", file: !3, line: 11, baseType: !183)
!183 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !20, size: 64)
!184 = !DILocalVariable(name: "fp", arg: 1, scope: !179, file: !3, line: 123, type: !182)
!185 = !DILocation(line: 123, column: 31, scope: !179)
!186 = !DILocation(line: 124, column: 5, scope: !179)
!187 = !DILocation(line: 125, column: 1, scope: !179)
!188 = distinct !DISubprogram(name: "main", scope: !3, file: !3, line: 127, type: !189, scopeLine: 127, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!189 = !DISubroutineType(types: !190)
!190 = !{!6}
!191 = !DILocation(line: 128, column: 5, scope: !188)
!192 = !DILocation(line: 129, column: 5, scope: !188)
!193 = !DILocation(line: 130, column: 5, scope: !188)
!194 = !DILocation(line: 131, column: 5, scope: !188)
!195 = !DILocation(line: 132, column: 5, scope: !188)
!196 = !DILocation(line: 133, column: 5, scope: !188)
!197 = !DILocation(line: 134, column: 5, scope: !188)
!198 = !DILocation(line: 135, column: 5, scope: !188)
!199 = !DILocation(line: 136, column: 5, scope: !188)
!200 = !DILocation(line: 137, column: 5, scope: !188)
!201 = !DILocation(line: 138, column: 5, scope: !188)
!202 = !DILocation(line: 139, column: 5, scope: !188)
