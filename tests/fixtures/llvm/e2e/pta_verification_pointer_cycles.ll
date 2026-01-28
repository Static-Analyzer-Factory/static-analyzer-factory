; ModuleID = 'tests/fixtures/pta_verification/pointer_cycles.c'
source_filename = "tests/fixtures/pta_verification/pointer_cycles.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Node = type { ptr, i32 }
%struct.SelfRef = type { ptr, i32 }

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_simple_cycle() #0 !dbg !12 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !16, metadata !DIExpression()), !dbg !17
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  store ptr %2, ptr %1, align 8, !dbg !20
  store ptr %1, ptr %2, align 8, !dbg !21
  call void @llvm.dbg.declare(metadata ptr %3, metadata !22, metadata !DIExpression()), !dbg !23
  %5 = load ptr, ptr %1, align 8, !dbg !24
  store ptr %5, ptr %3, align 8, !dbg !23
  call void @llvm.dbg.declare(metadata ptr %4, metadata !25, metadata !DIExpression()), !dbg !26
  %6 = load ptr, ptr %2, align 8, !dbg !27
  store ptr %6, ptr %4, align 8, !dbg !26
  %7 = load ptr, ptr %3, align 8, !dbg !28
  %8 = load ptr, ptr %4, align 8, !dbg !29
  ret void, !dbg !30
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_three_way_cycle() #0 !dbg !31 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !32, metadata !DIExpression()), !dbg !33
  call void @llvm.dbg.declare(metadata ptr %2, metadata !34, metadata !DIExpression()), !dbg !35
  call void @llvm.dbg.declare(metadata ptr %3, metadata !36, metadata !DIExpression()), !dbg !37
  store ptr %2, ptr %1, align 8, !dbg !38
  store ptr %3, ptr %2, align 8, !dbg !39
  store ptr %1, ptr %3, align 8, !dbg !40
  call void @llvm.dbg.declare(metadata ptr %4, metadata !41, metadata !DIExpression()), !dbg !42
  %5 = load ptr, ptr %1, align 8, !dbg !43
  store ptr %5, ptr %4, align 8, !dbg !42
  %6 = load ptr, ptr %4, align 8, !dbg !44
  ret void, !dbg !45
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_memory_cycle() #0 !dbg !46 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !47, metadata !DIExpression()), !dbg !48
  call void @llvm.dbg.declare(metadata ptr %2, metadata !49, metadata !DIExpression()), !dbg !51
  store ptr %1, ptr %2, align 8, !dbg !51
  %4 = load ptr, ptr %2, align 8, !dbg !52
  %5 = load ptr, ptr %2, align 8, !dbg !53
  store ptr %4, ptr %5, align 8, !dbg !54
  call void @llvm.dbg.declare(metadata ptr %3, metadata !55, metadata !DIExpression()), !dbg !56
  %6 = load ptr, ptr %2, align 8, !dbg !57
  %7 = load ptr, ptr %6, align 8, !dbg !58
  store ptr %7, ptr %3, align 8, !dbg !56
  %8 = load ptr, ptr %3, align 8, !dbg !59
  ret void, !dbg !60
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_linked_list_cycle() #0 !dbg !61 {
  %1 = alloca %struct.Node, align 8
  %2 = alloca %struct.Node, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !62, metadata !DIExpression()), !dbg !69
  call void @llvm.dbg.declare(metadata ptr %2, metadata !70, metadata !DIExpression()), !dbg !71
  %5 = getelementptr inbounds %struct.Node, ptr %1, i32 0, i32 0, !dbg !72
  store ptr %2, ptr %5, align 8, !dbg !73
  %6 = getelementptr inbounds %struct.Node, ptr %2, i32 0, i32 0, !dbg !74
  store ptr %1, ptr %6, align 8, !dbg !75
  call void @llvm.dbg.declare(metadata ptr %3, metadata !76, metadata !DIExpression()), !dbg !77
  %7 = getelementptr inbounds %struct.Node, ptr %1, i32 0, i32 0, !dbg !78
  %8 = load ptr, ptr %7, align 8, !dbg !78
  store ptr %8, ptr %3, align 8, !dbg !77
  call void @llvm.dbg.declare(metadata ptr %4, metadata !79, metadata !DIExpression()), !dbg !80
  %9 = load ptr, ptr %3, align 8, !dbg !81
  %10 = getelementptr inbounds %struct.Node, ptr %9, i32 0, i32 0, !dbg !82
  %11 = load ptr, ptr %10, align 8, !dbg !82
  store ptr %11, ptr %4, align 8, !dbg !80
  %12 = load ptr, ptr %4, align 8, !dbg !83
  ret void, !dbg !84
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_self_reference() #0 !dbg !85 {
  %1 = alloca %struct.SelfRef, align 8
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !86, metadata !DIExpression()), !dbg !92
  %4 = getelementptr inbounds %struct.SelfRef, ptr %1, i32 0, i32 0, !dbg !93
  store ptr %1, ptr %4, align 8, !dbg !94
  call void @llvm.dbg.declare(metadata ptr %2, metadata !95, metadata !DIExpression()), !dbg !96
  %5 = getelementptr inbounds %struct.SelfRef, ptr %1, i32 0, i32 0, !dbg !97
  %6 = load ptr, ptr %5, align 8, !dbg !97
  store ptr %6, ptr %2, align 8, !dbg !96
  call void @llvm.dbg.declare(metadata ptr %3, metadata !98, metadata !DIExpression()), !dbg !99
  %7 = load ptr, ptr %2, align 8, !dbg !100
  %8 = getelementptr inbounds %struct.SelfRef, ptr %7, i32 0, i32 0, !dbg !101
  %9 = load ptr, ptr %8, align 8, !dbg !101
  store ptr %9, ptr %3, align 8, !dbg !99
  %10 = load ptr, ptr %3, align 8, !dbg !102
  ret void, !dbg !103
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_convergence_wide() #0 !dbg !104 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !105, metadata !DIExpression()), !dbg !106
  call void @llvm.dbg.declare(metadata ptr %2, metadata !107, metadata !DIExpression()), !dbg !108
  call void @llvm.dbg.declare(metadata ptr %3, metadata !109, metadata !DIExpression()), !dbg !110
  call void @llvm.dbg.declare(metadata ptr %4, metadata !111, metadata !DIExpression()), !dbg !112
  call void @llvm.dbg.declare(metadata ptr %5, metadata !113, metadata !DIExpression()), !dbg !114
  call void @llvm.dbg.declare(metadata ptr %6, metadata !115, metadata !DIExpression()), !dbg !116
  call void @llvm.dbg.declare(metadata ptr %7, metadata !117, metadata !DIExpression()), !dbg !118
  call void @llvm.dbg.declare(metadata ptr %8, metadata !119, metadata !DIExpression()), !dbg !120
  call void @llvm.dbg.declare(metadata ptr %9, metadata !121, metadata !DIExpression()), !dbg !123
  store ptr %1, ptr %9, align 8, !dbg !124
  store ptr %2, ptr %9, align 8, !dbg !125
  store ptr %3, ptr %9, align 8, !dbg !126
  store ptr %4, ptr %9, align 8, !dbg !127
  store ptr %5, ptr %9, align 8, !dbg !128
  store ptr %6, ptr %9, align 8, !dbg !129
  store ptr %7, ptr %9, align 8, !dbg !130
  store ptr %8, ptr %9, align 8, !dbg !131
  call void @llvm.dbg.declare(metadata ptr %10, metadata !132, metadata !DIExpression()), !dbg !133
  %11 = load ptr, ptr %9, align 8, !dbg !134
  store ptr %11, ptr %10, align 8, !dbg !133
  %12 = load ptr, ptr %10, align 8, !dbg !135
  ret void, !dbg !136
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_deep_chain() #0 !dbg !137 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  %10 = alloca ptr, align 8
  %11 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !138, metadata !DIExpression()), !dbg !139
  call void @llvm.dbg.declare(metadata ptr %2, metadata !140, metadata !DIExpression()), !dbg !141
  store ptr %1, ptr %2, align 8, !dbg !141
  call void @llvm.dbg.declare(metadata ptr %3, metadata !142, metadata !DIExpression()), !dbg !143
  %12 = load ptr, ptr %2, align 8, !dbg !144
  store ptr %12, ptr %3, align 8, !dbg !143
  call void @llvm.dbg.declare(metadata ptr %4, metadata !145, metadata !DIExpression()), !dbg !146
  %13 = load ptr, ptr %3, align 8, !dbg !147
  store ptr %13, ptr %4, align 8, !dbg !146
  call void @llvm.dbg.declare(metadata ptr %5, metadata !148, metadata !DIExpression()), !dbg !149
  %14 = load ptr, ptr %4, align 8, !dbg !150
  store ptr %14, ptr %5, align 8, !dbg !149
  call void @llvm.dbg.declare(metadata ptr %6, metadata !151, metadata !DIExpression()), !dbg !152
  %15 = load ptr, ptr %5, align 8, !dbg !153
  store ptr %15, ptr %6, align 8, !dbg !152
  call void @llvm.dbg.declare(metadata ptr %7, metadata !154, metadata !DIExpression()), !dbg !155
  %16 = load ptr, ptr %6, align 8, !dbg !156
  store ptr %16, ptr %7, align 8, !dbg !155
  call void @llvm.dbg.declare(metadata ptr %8, metadata !157, metadata !DIExpression()), !dbg !158
  %17 = load ptr, ptr %7, align 8, !dbg !159
  store ptr %17, ptr %8, align 8, !dbg !158
  call void @llvm.dbg.declare(metadata ptr %9, metadata !160, metadata !DIExpression()), !dbg !161
  %18 = load ptr, ptr %8, align 8, !dbg !162
  store ptr %18, ptr %9, align 8, !dbg !161
  call void @llvm.dbg.declare(metadata ptr %10, metadata !163, metadata !DIExpression()), !dbg !164
  %19 = load ptr, ptr %9, align 8, !dbg !165
  store ptr %19, ptr %10, align 8, !dbg !164
  call void @llvm.dbg.declare(metadata ptr %11, metadata !166, metadata !DIExpression()), !dbg !167
  %20 = load ptr, ptr %10, align 8, !dbg !168
  store ptr %20, ptr %11, align 8, !dbg !167
  %21 = load ptr, ptr %11, align 8, !dbg !169
  ret void, !dbg !170
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_diamond(i32 noundef %0) #0 !dbg !171 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !174, metadata !DIExpression()), !dbg !175
  call void @llvm.dbg.declare(metadata ptr %3, metadata !176, metadata !DIExpression()), !dbg !177
  call void @llvm.dbg.declare(metadata ptr %4, metadata !178, metadata !DIExpression()), !dbg !179
  store ptr %3, ptr %4, align 8, !dbg !179
  call void @llvm.dbg.declare(metadata ptr %5, metadata !180, metadata !DIExpression()), !dbg !181
  call void @llvm.dbg.declare(metadata ptr %6, metadata !182, metadata !DIExpression()), !dbg !183
  call void @llvm.dbg.declare(metadata ptr %7, metadata !184, metadata !DIExpression()), !dbg !185
  %8 = load ptr, ptr %4, align 8, !dbg !186
  store ptr %8, ptr %5, align 8, !dbg !187
  %9 = load ptr, ptr %4, align 8, !dbg !188
  store ptr %9, ptr %6, align 8, !dbg !189
  %10 = load i32, ptr %2, align 4, !dbg !190
  %11 = icmp ne i32 %10, 0, !dbg !190
  br i1 %11, label %12, label %14, !dbg !192

12:                                               ; preds = %1
  %13 = load ptr, ptr %5, align 8, !dbg !193
  store ptr %13, ptr %7, align 8, !dbg !195
  br label %16, !dbg !196

14:                                               ; preds = %1
  %15 = load ptr, ptr %6, align 8, !dbg !197
  store ptr %15, ptr %7, align 8, !dbg !199
  br label %16

16:                                               ; preds = %14, %12
  %17 = load ptr, ptr %7, align 8, !dbg !200
  ret void, !dbg !201
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !202 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_simple_cycle(), !dbg !205
  call void @test_three_way_cycle(), !dbg !206
  call void @test_memory_cycle(), !dbg !207
  call void @test_linked_list_cycle(), !dbg !208
  call void @test_self_reference(), !dbg !209
  call void @test_convergence_wide(), !dbg !210
  call void @test_deep_chain(), !dbg !211
  call void @test_diamond(i32 noundef 1), !dbg !212
  ret i32 0, !dbg !213
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!4, !5, !6, !7, !8, !9, !10}
!llvm.ident = !{!11}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/fixtures/pta_verification/pointer_cycles.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "0735028e4ba255644cd22e4652d7a530")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!4 = !{i32 7, !"Dwarf Version", i32 5}
!5 = !{i32 2, !"Debug Info Version", i32 3}
!6 = !{i32 1, !"wchar_size", i32 4}
!7 = !{i32 8, !"PIC Level", i32 2}
!8 = !{i32 7, !"PIE Level", i32 2}
!9 = !{i32 7, !"uwtable", i32 2}
!10 = !{i32 7, !"frame-pointer", i32 1}
!11 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!12 = distinct !DISubprogram(name: "test_simple_cycle", scope: !1, file: !1, line: 7, type: !13, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!13 = !DISubroutineType(types: !14)
!14 = !{null}
!15 = !{}
!16 = !DILocalVariable(name: "p", scope: !12, file: !1, line: 8, type: !3)
!17 = !DILocation(line: 8, column: 11, scope: !12)
!18 = !DILocalVariable(name: "q", scope: !12, file: !1, line: 8, type: !3)
!19 = !DILocation(line: 8, column: 15, scope: !12)
!20 = !DILocation(line: 9, column: 7, scope: !12)
!21 = !DILocation(line: 10, column: 7, scope: !12)
!22 = !DILocalVariable(name: "r", scope: !12, file: !1, line: 12, type: !3)
!23 = !DILocation(line: 12, column: 11, scope: !12)
!24 = !DILocation(line: 12, column: 15, scope: !12)
!25 = !DILocalVariable(name: "s", scope: !12, file: !1, line: 13, type: !3)
!26 = !DILocation(line: 13, column: 11, scope: !12)
!27 = !DILocation(line: 13, column: 15, scope: !12)
!28 = !DILocation(line: 14, column: 11, scope: !12)
!29 = !DILocation(line: 15, column: 11, scope: !12)
!30 = !DILocation(line: 16, column: 1, scope: !12)
!31 = distinct !DISubprogram(name: "test_three_way_cycle", scope: !1, file: !1, line: 19, type: !13, scopeLine: 19, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!32 = !DILocalVariable(name: "a", scope: !31, file: !1, line: 20, type: !3)
!33 = !DILocation(line: 20, column: 11, scope: !31)
!34 = !DILocalVariable(name: "b", scope: !31, file: !1, line: 20, type: !3)
!35 = !DILocation(line: 20, column: 15, scope: !31)
!36 = !DILocalVariable(name: "c", scope: !31, file: !1, line: 20, type: !3)
!37 = !DILocation(line: 20, column: 19, scope: !31)
!38 = !DILocation(line: 21, column: 7, scope: !31)
!39 = !DILocation(line: 22, column: 7, scope: !31)
!40 = !DILocation(line: 23, column: 7, scope: !31)
!41 = !DILocalVariable(name: "x", scope: !31, file: !1, line: 26, type: !3)
!42 = !DILocation(line: 26, column: 11, scope: !31)
!43 = !DILocation(line: 26, column: 15, scope: !31)
!44 = !DILocation(line: 27, column: 11, scope: !31)
!45 = !DILocation(line: 28, column: 1, scope: !31)
!46 = distinct !DISubprogram(name: "test_memory_cycle", scope: !1, file: !1, line: 31, type: !13, scopeLine: 31, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!47 = !DILocalVariable(name: "p", scope: !46, file: !1, line: 32, type: !3)
!48 = !DILocation(line: 32, column: 11, scope: !46)
!49 = !DILocalVariable(name: "pp", scope: !46, file: !1, line: 33, type: !50)
!50 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !3, size: 64)
!51 = !DILocation(line: 33, column: 12, scope: !46)
!52 = !DILocation(line: 37, column: 18, scope: !46)
!53 = !DILocation(line: 37, column: 6, scope: !46)
!54 = !DILocation(line: 37, column: 9, scope: !46)
!55 = !DILocalVariable(name: "r", scope: !46, file: !1, line: 39, type: !3)
!56 = !DILocation(line: 39, column: 11, scope: !46)
!57 = !DILocation(line: 39, column: 16, scope: !46)
!58 = !DILocation(line: 39, column: 15, scope: !46)
!59 = !DILocation(line: 40, column: 11, scope: !46)
!60 = !DILocation(line: 41, column: 1, scope: !46)
!61 = distinct !DISubprogram(name: "test_linked_list_cycle", scope: !1, file: !1, line: 49, type: !13, scopeLine: 49, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!62 = !DILocalVariable(name: "head", scope: !61, file: !1, line: 50, type: !63)
!63 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Node", file: !1, line: 44, size: 128, elements: !64)
!64 = !{!65, !67}
!65 = !DIDerivedType(tag: DW_TAG_member, name: "next", scope: !63, file: !1, line: 45, baseType: !66, size: 64)
!66 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !63, size: 64)
!67 = !DIDerivedType(tag: DW_TAG_member, name: "data", scope: !63, file: !1, line: 46, baseType: !68, size: 32, offset: 64)
!68 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!69 = !DILocation(line: 50, column: 17, scope: !61)
!70 = !DILocalVariable(name: "node1", scope: !61, file: !1, line: 51, type: !63)
!71 = !DILocation(line: 51, column: 17, scope: !61)
!72 = !DILocation(line: 53, column: 10, scope: !61)
!73 = !DILocation(line: 53, column: 15, scope: !61)
!74 = !DILocation(line: 54, column: 11, scope: !61)
!75 = !DILocation(line: 54, column: 16, scope: !61)
!76 = !DILocalVariable(name: "p", scope: !61, file: !1, line: 56, type: !66)
!77 = !DILocation(line: 56, column: 18, scope: !61)
!78 = !DILocation(line: 56, column: 27, scope: !61)
!79 = !DILocalVariable(name: "q", scope: !61, file: !1, line: 57, type: !66)
!80 = !DILocation(line: 57, column: 18, scope: !61)
!81 = !DILocation(line: 57, column: 22, scope: !61)
!82 = !DILocation(line: 57, column: 25, scope: !61)
!83 = !DILocation(line: 58, column: 11, scope: !61)
!84 = !DILocation(line: 59, column: 1, scope: !61)
!85 = distinct !DISubprogram(name: "test_self_reference", scope: !1, file: !1, line: 67, type: !13, scopeLine: 67, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!86 = !DILocalVariable(name: "obj", scope: !85, file: !1, line: 68, type: !87)
!87 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "SelfRef", file: !1, line: 62, size: 128, elements: !88)
!88 = !{!89, !91}
!89 = !DIDerivedType(tag: DW_TAG_member, name: "self", scope: !87, file: !1, line: 63, baseType: !90, size: 64)
!90 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !87, size: 64)
!91 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !87, file: !1, line: 64, baseType: !68, size: 32, offset: 64)
!92 = !DILocation(line: 68, column: 20, scope: !85)
!93 = !DILocation(line: 69, column: 9, scope: !85)
!94 = !DILocation(line: 69, column: 14, scope: !85)
!95 = !DILocalVariable(name: "p", scope: !85, file: !1, line: 71, type: !90)
!96 = !DILocation(line: 71, column: 21, scope: !85)
!97 = !DILocation(line: 71, column: 29, scope: !85)
!98 = !DILocalVariable(name: "q", scope: !85, file: !1, line: 72, type: !90)
!99 = !DILocation(line: 72, column: 21, scope: !85)
!100 = !DILocation(line: 72, column: 25, scope: !85)
!101 = !DILocation(line: 72, column: 28, scope: !85)
!102 = !DILocation(line: 73, column: 11, scope: !85)
!103 = !DILocation(line: 74, column: 1, scope: !85)
!104 = distinct !DISubprogram(name: "test_convergence_wide", scope: !1, file: !1, line: 77, type: !13, scopeLine: 77, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!105 = !DILocalVariable(name: "a", scope: !104, file: !1, line: 78, type: !68)
!106 = !DILocation(line: 78, column: 9, scope: !104)
!107 = !DILocalVariable(name: "b", scope: !104, file: !1, line: 78, type: !68)
!108 = !DILocation(line: 78, column: 12, scope: !104)
!109 = !DILocalVariable(name: "c", scope: !104, file: !1, line: 78, type: !68)
!110 = !DILocation(line: 78, column: 15, scope: !104)
!111 = !DILocalVariable(name: "d", scope: !104, file: !1, line: 78, type: !68)
!112 = !DILocation(line: 78, column: 18, scope: !104)
!113 = !DILocalVariable(name: "e", scope: !104, file: !1, line: 78, type: !68)
!114 = !DILocation(line: 78, column: 21, scope: !104)
!115 = !DILocalVariable(name: "f", scope: !104, file: !1, line: 78, type: !68)
!116 = !DILocation(line: 78, column: 24, scope: !104)
!117 = !DILocalVariable(name: "g", scope: !104, file: !1, line: 78, type: !68)
!118 = !DILocation(line: 78, column: 27, scope: !104)
!119 = !DILocalVariable(name: "h", scope: !104, file: !1, line: 78, type: !68)
!120 = !DILocation(line: 78, column: 30, scope: !104)
!121 = !DILocalVariable(name: "p", scope: !104, file: !1, line: 79, type: !122)
!122 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !68, size: 64)
!123 = !DILocation(line: 79, column: 10, scope: !104)
!124 = !DILocation(line: 82, column: 7, scope: !104)
!125 = !DILocation(line: 83, column: 7, scope: !104)
!126 = !DILocation(line: 84, column: 7, scope: !104)
!127 = !DILocation(line: 85, column: 7, scope: !104)
!128 = !DILocation(line: 86, column: 7, scope: !104)
!129 = !DILocation(line: 87, column: 7, scope: !104)
!130 = !DILocation(line: 88, column: 7, scope: !104)
!131 = !DILocation(line: 89, column: 7, scope: !104)
!132 = !DILocalVariable(name: "q", scope: !104, file: !1, line: 92, type: !122)
!133 = !DILocation(line: 92, column: 10, scope: !104)
!134 = !DILocation(line: 92, column: 14, scope: !104)
!135 = !DILocation(line: 93, column: 11, scope: !104)
!136 = !DILocation(line: 94, column: 1, scope: !104)
!137 = distinct !DISubprogram(name: "test_deep_chain", scope: !1, file: !1, line: 97, type: !13, scopeLine: 97, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!138 = !DILocalVariable(name: "x", scope: !137, file: !1, line: 98, type: !68)
!139 = !DILocation(line: 98, column: 9, scope: !137)
!140 = !DILocalVariable(name: "p1", scope: !137, file: !1, line: 99, type: !122)
!141 = !DILocation(line: 99, column: 10, scope: !137)
!142 = !DILocalVariable(name: "p2", scope: !137, file: !1, line: 100, type: !122)
!143 = !DILocation(line: 100, column: 10, scope: !137)
!144 = !DILocation(line: 100, column: 15, scope: !137)
!145 = !DILocalVariable(name: "p3", scope: !137, file: !1, line: 101, type: !122)
!146 = !DILocation(line: 101, column: 10, scope: !137)
!147 = !DILocation(line: 101, column: 15, scope: !137)
!148 = !DILocalVariable(name: "p4", scope: !137, file: !1, line: 102, type: !122)
!149 = !DILocation(line: 102, column: 10, scope: !137)
!150 = !DILocation(line: 102, column: 15, scope: !137)
!151 = !DILocalVariable(name: "p5", scope: !137, file: !1, line: 103, type: !122)
!152 = !DILocation(line: 103, column: 10, scope: !137)
!153 = !DILocation(line: 103, column: 15, scope: !137)
!154 = !DILocalVariable(name: "p6", scope: !137, file: !1, line: 104, type: !122)
!155 = !DILocation(line: 104, column: 10, scope: !137)
!156 = !DILocation(line: 104, column: 15, scope: !137)
!157 = !DILocalVariable(name: "p7", scope: !137, file: !1, line: 105, type: !122)
!158 = !DILocation(line: 105, column: 10, scope: !137)
!159 = !DILocation(line: 105, column: 15, scope: !137)
!160 = !DILocalVariable(name: "p8", scope: !137, file: !1, line: 106, type: !122)
!161 = !DILocation(line: 106, column: 10, scope: !137)
!162 = !DILocation(line: 106, column: 15, scope: !137)
!163 = !DILocalVariable(name: "p9", scope: !137, file: !1, line: 107, type: !122)
!164 = !DILocation(line: 107, column: 10, scope: !137)
!165 = !DILocation(line: 107, column: 15, scope: !137)
!166 = !DILocalVariable(name: "p10", scope: !137, file: !1, line: 108, type: !122)
!167 = !DILocation(line: 108, column: 10, scope: !137)
!168 = !DILocation(line: 108, column: 16, scope: !137)
!169 = !DILocation(line: 111, column: 11, scope: !137)
!170 = !DILocation(line: 112, column: 1, scope: !137)
!171 = distinct !DISubprogram(name: "test_diamond", scope: !1, file: !1, line: 115, type: !172, scopeLine: 115, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!172 = !DISubroutineType(types: !173)
!173 = !{null, !68}
!174 = !DILocalVariable(name: "cond", arg: 1, scope: !171, file: !1, line: 115, type: !68)
!175 = !DILocation(line: 115, column: 23, scope: !171)
!176 = !DILocalVariable(name: "x", scope: !171, file: !1, line: 116, type: !68)
!177 = !DILocation(line: 116, column: 9, scope: !171)
!178 = !DILocalVariable(name: "a", scope: !171, file: !1, line: 117, type: !122)
!179 = !DILocation(line: 117, column: 10, scope: !171)
!180 = !DILocalVariable(name: "b", scope: !171, file: !1, line: 118, type: !122)
!181 = !DILocation(line: 118, column: 10, scope: !171)
!182 = !DILocalVariable(name: "c", scope: !171, file: !1, line: 118, type: !122)
!183 = !DILocation(line: 118, column: 14, scope: !171)
!184 = !DILocalVariable(name: "d", scope: !171, file: !1, line: 118, type: !122)
!185 = !DILocation(line: 118, column: 18, scope: !171)
!186 = !DILocation(line: 121, column: 9, scope: !171)
!187 = !DILocation(line: 121, column: 7, scope: !171)
!188 = !DILocation(line: 122, column: 9, scope: !171)
!189 = !DILocation(line: 122, column: 7, scope: !171)
!190 = !DILocation(line: 123, column: 9, scope: !191)
!191 = distinct !DILexicalBlock(scope: !171, file: !1, line: 123, column: 9)
!192 = !DILocation(line: 123, column: 9, scope: !171)
!193 = !DILocation(line: 124, column: 13, scope: !194)
!194 = distinct !DILexicalBlock(scope: !191, file: !1, line: 123, column: 15)
!195 = !DILocation(line: 124, column: 11, scope: !194)
!196 = !DILocation(line: 125, column: 5, scope: !194)
!197 = !DILocation(line: 126, column: 13, scope: !198)
!198 = distinct !DILexicalBlock(scope: !191, file: !1, line: 125, column: 12)
!199 = !DILocation(line: 126, column: 11, scope: !198)
!200 = !DILocation(line: 129, column: 11, scope: !171)
!201 = !DILocation(line: 130, column: 1, scope: !171)
!202 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 132, type: !203, scopeLine: 132, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!203 = !DISubroutineType(types: !204)
!204 = !{!68}
!205 = !DILocation(line: 133, column: 5, scope: !202)
!206 = !DILocation(line: 134, column: 5, scope: !202)
!207 = !DILocation(line: 135, column: 5, scope: !202)
!208 = !DILocation(line: 136, column: 5, scope: !202)
!209 = !DILocation(line: 137, column: 5, scope: !202)
!210 = !DILocation(line: 138, column: 5, scope: !202)
!211 = !DILocation(line: 139, column: 5, scope: !202)
!212 = !DILocation(line: 140, column: 5, scope: !202)
!213 = !DILocation(line: 141, column: 5, scope: !202)
