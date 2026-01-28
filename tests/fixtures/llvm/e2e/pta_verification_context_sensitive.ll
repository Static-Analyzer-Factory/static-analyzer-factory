; ModuleID = 'tests/fixtures/pta_verification/context_sensitive.c'
source_filename = "tests/fixtures/pta_verification/context_sensitive.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Object = type { i32 }

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @identity(ptr noundef %0) #0 !dbg !17 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !22, metadata !DIExpression()), !dbg !23
  %3 = load ptr, ptr %2, align 8, !dbg !24
  ret ptr %3, !dbg !25
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_identity_wrapper() #0 !dbg !26 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !29, metadata !DIExpression()), !dbg !30
  call void @llvm.dbg.declare(metadata ptr %2, metadata !31, metadata !DIExpression()), !dbg !32
  call void @llvm.dbg.declare(metadata ptr %3, metadata !33, metadata !DIExpression()), !dbg !34
  %5 = call ptr @identity(ptr noundef %1), !dbg !35
  store ptr %5, ptr %3, align 8, !dbg !34
  call void @llvm.dbg.declare(metadata ptr %4, metadata !36, metadata !DIExpression()), !dbg !37
  %6 = call ptr @identity(ptr noundef %2), !dbg !38
  store ptr %6, ptr %4, align 8, !dbg !37
  %7 = load ptr, ptr %3, align 8, !dbg !39
  %8 = load ptr, ptr %4, align 8, !dbg !40
  ret void, !dbg !41
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrapper2(ptr noundef %0) #0 !dbg !42 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !43, metadata !DIExpression()), !dbg !44
  %3 = load ptr, ptr %2, align 8, !dbg !45
  %4 = call ptr @identity(ptr noundef %3), !dbg !46
  ret ptr %4, !dbg !47
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_nested_wrappers() #0 !dbg !48 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !49, metadata !DIExpression()), !dbg !50
  call void @llvm.dbg.declare(metadata ptr %2, metadata !51, metadata !DIExpression()), !dbg !52
  call void @llvm.dbg.declare(metadata ptr %3, metadata !53, metadata !DIExpression()), !dbg !54
  %5 = call ptr @wrapper2(ptr noundef %1), !dbg !55
  store ptr %5, ptr %3, align 8, !dbg !54
  call void @llvm.dbg.declare(metadata ptr %4, metadata !56, metadata !DIExpression()), !dbg !57
  %6 = call ptr @wrapper2(ptr noundef %2), !dbg !58
  store ptr %6, ptr %4, align 8, !dbg !57
  %7 = load ptr, ptr %3, align 8, !dbg !59
  %8 = load ptr, ptr %4, align 8, !dbg !60
  ret void, !dbg !61
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrapper3(ptr noundef %0) #0 !dbg !62 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !63, metadata !DIExpression()), !dbg !64
  %3 = load ptr, ptr %2, align 8, !dbg !65
  %4 = call ptr @wrapper2(ptr noundef %3), !dbg !66
  ret ptr %4, !dbg !67
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_triple_nesting() #0 !dbg !68 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !69, metadata !DIExpression()), !dbg !70
  call void @llvm.dbg.declare(metadata ptr %2, metadata !71, metadata !DIExpression()), !dbg !72
  call void @llvm.dbg.declare(metadata ptr %3, metadata !73, metadata !DIExpression()), !dbg !74
  %5 = call ptr @wrapper3(ptr noundef %1), !dbg !75
  store ptr %5, ptr %3, align 8, !dbg !74
  call void @llvm.dbg.declare(metadata ptr %4, metadata !76, metadata !DIExpression()), !dbg !77
  %6 = call ptr @wrapper3(ptr noundef %2), !dbg !78
  store ptr %6, ptr %4, align 8, !dbg !77
  %7 = load ptr, ptr %3, align 8, !dbg !79
  %8 = load ptr, ptr %4, align 8, !dbg !80
  ret void, !dbg !81
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @my_alloc(i64 noundef %0) #0 !dbg !82 {
  %2 = alloca i64, align 8
  store i64 %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !88, metadata !DIExpression()), !dbg !89
  %3 = load i64, ptr %2, align 8, !dbg !90
  %4 = call noalias ptr @malloc(i64 noundef %3) #3, !dbg !91
  ret ptr %4, !dbg !92
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_alloc_wrapper() #0 !dbg !93 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !94, metadata !DIExpression()), !dbg !95
  %3 = call ptr @my_alloc(i64 noundef 10), !dbg !96
  store ptr %3, ptr %1, align 8, !dbg !95
  call void @llvm.dbg.declare(metadata ptr %2, metadata !97, metadata !DIExpression()), !dbg !98
  %4 = call ptr @my_alloc(i64 noundef 20), !dbg !99
  store ptr %4, ptr %2, align 8, !dbg !98
  %5 = load ptr, ptr %1, align 8, !dbg !100
  %6 = load ptr, ptr %2, align 8, !dbg !101
  ret void, !dbg !102
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @make_object(i32 noundef %0) #0 !dbg !103 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !106, metadata !DIExpression()), !dbg !107
  call void @llvm.dbg.declare(metadata ptr %3, metadata !108, metadata !DIExpression()), !dbg !109
  %4 = call noalias ptr @malloc(i64 noundef 4) #3, !dbg !110
  store ptr %4, ptr %3, align 8, !dbg !109
  %5 = load i32, ptr %2, align 4, !dbg !111
  %6 = load ptr, ptr %3, align 8, !dbg !112
  %7 = getelementptr inbounds %struct.Object, ptr %6, i32 0, i32 0, !dbg !113
  store i32 %5, ptr %7, align 4, !dbg !114
  %8 = load ptr, ptr %3, align 8, !dbg !115
  ret ptr %8, !dbg !116
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_factory_pattern() #0 !dbg !117 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !118, metadata !DIExpression()), !dbg !119
  %3 = call ptr @make_object(i32 noundef 1), !dbg !120
  store ptr %3, ptr %1, align 8, !dbg !119
  call void @llvm.dbg.declare(metadata ptr %2, metadata !121, metadata !DIExpression()), !dbg !122
  %4 = call ptr @make_object(i32 noundef 2), !dbg !123
  store ptr %4, ptr %2, align 8, !dbg !122
  %5 = load ptr, ptr %1, align 8, !dbg !124
  %6 = load ptr, ptr %2, align 8, !dbg !125
  ret void, !dbg !126
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @pass_through_a(ptr noundef %0) #0 !dbg !127 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !128, metadata !DIExpression()), !dbg !129
  %3 = load ptr, ptr %2, align 8, !dbg !130
  %4 = call ptr @identity(ptr noundef %3), !dbg !131
  ret ptr %4, !dbg !132
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @pass_through_b(ptr noundef %0) #0 !dbg !133 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !134, metadata !DIExpression()), !dbg !135
  %3 = load ptr, ptr %2, align 8, !dbg !136
  %4 = call ptr @identity(ptr noundef %3), !dbg !137
  ret ptr %4, !dbg !138
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_different_callers() #0 !dbg !139 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !140, metadata !DIExpression()), !dbg !141
  call void @llvm.dbg.declare(metadata ptr %2, metadata !142, metadata !DIExpression()), !dbg !143
  %4 = call ptr @pass_through_a(ptr noundef %1), !dbg !144
  store ptr %4, ptr %2, align 8, !dbg !143
  call void @llvm.dbg.declare(metadata ptr %3, metadata !145, metadata !DIExpression()), !dbg !146
  %5 = call ptr @pass_through_b(ptr noundef %1), !dbg !147
  store ptr %5, ptr %3, align 8, !dbg !146
  %6 = load ptr, ptr %2, align 8, !dbg !148
  %7 = load ptr, ptr %3, align 8, !dbg !149
  ret void, !dbg !150
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @maybe_wrap(i32 noundef %0, ptr noundef %1) #0 !dbg !151 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !154, metadata !DIExpression()), !dbg !155
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !156, metadata !DIExpression()), !dbg !157
  %6 = load i32, ptr %4, align 4, !dbg !158
  %7 = icmp ne i32 %6, 0, !dbg !158
  br i1 %7, label %8, label %11, !dbg !160

8:                                                ; preds = %2
  %9 = load ptr, ptr %5, align 8, !dbg !161
  %10 = call ptr @identity(ptr noundef %9), !dbg !163
  store ptr %10, ptr %3, align 8, !dbg !164
  br label %13, !dbg !164

11:                                               ; preds = %2
  %12 = load ptr, ptr %5, align 8, !dbg !165
  store ptr %12, ptr %3, align 8, !dbg !166
  br label %13, !dbg !166

13:                                               ; preds = %11, %8
  %14 = load ptr, ptr %3, align 8, !dbg !167
  ret ptr %14, !dbg !167
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_context_with_branch() #0 !dbg !168 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !169, metadata !DIExpression()), !dbg !170
  call void @llvm.dbg.declare(metadata ptr %2, metadata !171, metadata !DIExpression()), !dbg !172
  %3 = call ptr @maybe_wrap(i32 noundef 1, ptr noundef %1), !dbg !173
  store ptr %3, ptr %2, align 8, !dbg !172
  %4 = load ptr, ptr %2, align 8, !dbg !174
  ret void, !dbg !175
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !176 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_identity_wrapper(), !dbg !179
  call void @test_nested_wrappers(), !dbg !180
  call void @test_triple_nesting(), !dbg !181
  call void @test_alloc_wrapper(), !dbg !182
  call void @test_factory_pattern(), !dbg !183
  call void @test_different_callers(), !dbg !184
  call void @test_context_with_branch(), !dbg !185
  ret i32 0, !dbg !186
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind allocsize(0) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/fixtures/pta_verification/context_sensitive.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "ff4f7d4152328e17f093f34bd2c3e5b6")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIDerivedType(tag: DW_TAG_typedef, name: "Object", file: !1, line: 70, baseType: !5)
!5 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !1, line: 70, size: 32, elements: !6)
!6 = !{!7}
!7 = !DIDerivedType(tag: DW_TAG_member, name: "data", scope: !5, file: !1, line: 70, baseType: !8, size: 32)
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "identity", scope: !1, file: !1, line: 9, type: !18, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!18 = !DISubroutineType(types: !19)
!19 = !{!20, !20}
!20 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!21 = !{}
!22 = !DILocalVariable(name: "p", arg: 1, scope: !17, file: !1, line: 9, type: !20)
!23 = !DILocation(line: 9, column: 22, scope: !17)
!24 = !DILocation(line: 10, column: 12, scope: !17)
!25 = !DILocation(line: 10, column: 5, scope: !17)
!26 = distinct !DISubprogram(name: "test_identity_wrapper", scope: !1, file: !1, line: 13, type: !27, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!27 = !DISubroutineType(types: !28)
!28 = !{null}
!29 = !DILocalVariable(name: "a", scope: !26, file: !1, line: 14, type: !8)
!30 = !DILocation(line: 14, column: 9, scope: !26)
!31 = !DILocalVariable(name: "b", scope: !26, file: !1, line: 14, type: !8)
!32 = !DILocation(line: 14, column: 12, scope: !26)
!33 = !DILocalVariable(name: "r1", scope: !26, file: !1, line: 15, type: !20)
!34 = !DILocation(line: 15, column: 11, scope: !26)
!35 = !DILocation(line: 15, column: 16, scope: !26)
!36 = !DILocalVariable(name: "r2", scope: !26, file: !1, line: 16, type: !20)
!37 = !DILocation(line: 16, column: 11, scope: !26)
!38 = !DILocation(line: 16, column: 16, scope: !26)
!39 = !DILocation(line: 20, column: 11, scope: !26)
!40 = !DILocation(line: 21, column: 11, scope: !26)
!41 = !DILocation(line: 22, column: 1, scope: !26)
!42 = distinct !DISubprogram(name: "wrapper2", scope: !1, file: !1, line: 25, type: !18, scopeLine: 25, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!43 = !DILocalVariable(name: "p", arg: 1, scope: !42, file: !1, line: 25, type: !20)
!44 = !DILocation(line: 25, column: 22, scope: !42)
!45 = !DILocation(line: 26, column: 21, scope: !42)
!46 = !DILocation(line: 26, column: 12, scope: !42)
!47 = !DILocation(line: 26, column: 5, scope: !42)
!48 = distinct !DISubprogram(name: "test_nested_wrappers", scope: !1, file: !1, line: 29, type: !27, scopeLine: 29, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!49 = !DILocalVariable(name: "x", scope: !48, file: !1, line: 30, type: !8)
!50 = !DILocation(line: 30, column: 9, scope: !48)
!51 = !DILocalVariable(name: "y", scope: !48, file: !1, line: 30, type: !8)
!52 = !DILocation(line: 30, column: 12, scope: !48)
!53 = !DILocalVariable(name: "r1", scope: !48, file: !1, line: 31, type: !20)
!54 = !DILocation(line: 31, column: 11, scope: !48)
!55 = !DILocation(line: 31, column: 16, scope: !48)
!56 = !DILocalVariable(name: "r2", scope: !48, file: !1, line: 32, type: !20)
!57 = !DILocation(line: 32, column: 11, scope: !48)
!58 = !DILocation(line: 32, column: 16, scope: !48)
!59 = !DILocation(line: 35, column: 11, scope: !48)
!60 = !DILocation(line: 36, column: 11, scope: !48)
!61 = !DILocation(line: 37, column: 1, scope: !48)
!62 = distinct !DISubprogram(name: "wrapper3", scope: !1, file: !1, line: 40, type: !18, scopeLine: 40, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!63 = !DILocalVariable(name: "p", arg: 1, scope: !62, file: !1, line: 40, type: !20)
!64 = !DILocation(line: 40, column: 22, scope: !62)
!65 = !DILocation(line: 41, column: 21, scope: !62)
!66 = !DILocation(line: 41, column: 12, scope: !62)
!67 = !DILocation(line: 41, column: 5, scope: !62)
!68 = distinct !DISubprogram(name: "test_triple_nesting", scope: !1, file: !1, line: 44, type: !27, scopeLine: 44, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!69 = !DILocalVariable(name: "a", scope: !68, file: !1, line: 45, type: !8)
!70 = !DILocation(line: 45, column: 9, scope: !68)
!71 = !DILocalVariable(name: "b", scope: !68, file: !1, line: 45, type: !8)
!72 = !DILocation(line: 45, column: 12, scope: !68)
!73 = !DILocalVariable(name: "r1", scope: !68, file: !1, line: 46, type: !20)
!74 = !DILocation(line: 46, column: 11, scope: !68)
!75 = !DILocation(line: 46, column: 16, scope: !68)
!76 = !DILocalVariable(name: "r2", scope: !68, file: !1, line: 47, type: !20)
!77 = !DILocation(line: 47, column: 11, scope: !68)
!78 = !DILocation(line: 47, column: 16, scope: !68)
!79 = !DILocation(line: 50, column: 11, scope: !68)
!80 = !DILocation(line: 51, column: 11, scope: !68)
!81 = !DILocation(line: 52, column: 1, scope: !68)
!82 = distinct !DISubprogram(name: "my_alloc", scope: !1, file: !1, line: 55, type: !83, scopeLine: 55, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!83 = !DISubroutineType(types: !84)
!84 = !{!20, !85}
!85 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !86, line: 18, baseType: !87)
!86 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!87 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!88 = !DILocalVariable(name: "size", arg: 1, scope: !82, file: !1, line: 55, type: !85)
!89 = !DILocation(line: 55, column: 23, scope: !82)
!90 = !DILocation(line: 56, column: 19, scope: !82)
!91 = !DILocation(line: 56, column: 12, scope: !82)
!92 = !DILocation(line: 56, column: 5, scope: !82)
!93 = distinct !DISubprogram(name: "test_alloc_wrapper", scope: !1, file: !1, line: 59, type: !27, scopeLine: 59, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!94 = !DILocalVariable(name: "p1", scope: !93, file: !1, line: 60, type: !20)
!95 = !DILocation(line: 60, column: 11, scope: !93)
!96 = !DILocation(line: 60, column: 16, scope: !93)
!97 = !DILocalVariable(name: "p2", scope: !93, file: !1, line: 61, type: !20)
!98 = !DILocation(line: 61, column: 11, scope: !93)
!99 = !DILocation(line: 61, column: 16, scope: !93)
!100 = !DILocation(line: 65, column: 11, scope: !93)
!101 = !DILocation(line: 66, column: 11, scope: !93)
!102 = !DILocation(line: 67, column: 1, scope: !93)
!103 = distinct !DISubprogram(name: "make_object", scope: !1, file: !1, line: 72, type: !104, scopeLine: 72, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!104 = !DISubroutineType(types: !105)
!105 = !{!3, !8}
!106 = !DILocalVariable(name: "value", arg: 1, scope: !103, file: !1, line: 72, type: !8)
!107 = !DILocation(line: 72, column: 25, scope: !103)
!108 = !DILocalVariable(name: "obj", scope: !103, file: !1, line: 73, type: !3)
!109 = !DILocation(line: 73, column: 13, scope: !103)
!110 = !DILocation(line: 73, column: 28, scope: !103)
!111 = !DILocation(line: 74, column: 17, scope: !103)
!112 = !DILocation(line: 74, column: 5, scope: !103)
!113 = !DILocation(line: 74, column: 10, scope: !103)
!114 = !DILocation(line: 74, column: 15, scope: !103)
!115 = !DILocation(line: 75, column: 12, scope: !103)
!116 = !DILocation(line: 75, column: 5, scope: !103)
!117 = distinct !DISubprogram(name: "test_factory_pattern", scope: !1, file: !1, line: 78, type: !27, scopeLine: 78, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!118 = !DILocalVariable(name: "o1", scope: !117, file: !1, line: 79, type: !3)
!119 = !DILocation(line: 79, column: 13, scope: !117)
!120 = !DILocation(line: 79, column: 18, scope: !117)
!121 = !DILocalVariable(name: "o2", scope: !117, file: !1, line: 80, type: !3)
!122 = !DILocation(line: 80, column: 13, scope: !117)
!123 = !DILocation(line: 80, column: 18, scope: !117)
!124 = !DILocation(line: 83, column: 11, scope: !117)
!125 = !DILocation(line: 84, column: 11, scope: !117)
!126 = !DILocation(line: 85, column: 1, scope: !117)
!127 = distinct !DISubprogram(name: "pass_through_a", scope: !1, file: !1, line: 88, type: !18, scopeLine: 88, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!128 = !DILocalVariable(name: "p", arg: 1, scope: !127, file: !1, line: 88, type: !20)
!129 = !DILocation(line: 88, column: 28, scope: !127)
!130 = !DILocation(line: 89, column: 21, scope: !127)
!131 = !DILocation(line: 89, column: 12, scope: !127)
!132 = !DILocation(line: 89, column: 5, scope: !127)
!133 = distinct !DISubprogram(name: "pass_through_b", scope: !1, file: !1, line: 92, type: !18, scopeLine: 92, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!134 = !DILocalVariable(name: "p", arg: 1, scope: !133, file: !1, line: 92, type: !20)
!135 = !DILocation(line: 92, column: 28, scope: !133)
!136 = !DILocation(line: 93, column: 21, scope: !133)
!137 = !DILocation(line: 93, column: 12, scope: !133)
!138 = !DILocation(line: 93, column: 5, scope: !133)
!139 = distinct !DISubprogram(name: "test_different_callers", scope: !1, file: !1, line: 96, type: !27, scopeLine: 96, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!140 = !DILocalVariable(name: "x", scope: !139, file: !1, line: 97, type: !8)
!141 = !DILocation(line: 97, column: 9, scope: !139)
!142 = !DILocalVariable(name: "r1", scope: !139, file: !1, line: 99, type: !20)
!143 = !DILocation(line: 99, column: 11, scope: !139)
!144 = !DILocation(line: 99, column: 16, scope: !139)
!145 = !DILocalVariable(name: "r2", scope: !139, file: !1, line: 100, type: !20)
!146 = !DILocation(line: 100, column: 11, scope: !139)
!147 = !DILocation(line: 100, column: 16, scope: !139)
!148 = !DILocation(line: 103, column: 11, scope: !139)
!149 = !DILocation(line: 104, column: 11, scope: !139)
!150 = !DILocation(line: 105, column: 1, scope: !139)
!151 = distinct !DISubprogram(name: "maybe_wrap", scope: !1, file: !1, line: 108, type: !152, scopeLine: 108, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!152 = !DISubroutineType(types: !153)
!153 = !{!20, !8, !20}
!154 = !DILocalVariable(name: "cond", arg: 1, scope: !151, file: !1, line: 108, type: !8)
!155 = !DILocation(line: 108, column: 22, scope: !151)
!156 = !DILocalVariable(name: "p", arg: 2, scope: !151, file: !1, line: 108, type: !20)
!157 = !DILocation(line: 108, column: 34, scope: !151)
!158 = !DILocation(line: 109, column: 9, scope: !159)
!159 = distinct !DILexicalBlock(scope: !151, file: !1, line: 109, column: 9)
!160 = !DILocation(line: 109, column: 9, scope: !151)
!161 = !DILocation(line: 110, column: 25, scope: !162)
!162 = distinct !DILexicalBlock(scope: !159, file: !1, line: 109, column: 15)
!163 = !DILocation(line: 110, column: 16, scope: !162)
!164 = !DILocation(line: 110, column: 9, scope: !162)
!165 = !DILocation(line: 112, column: 12, scope: !151)
!166 = !DILocation(line: 112, column: 5, scope: !151)
!167 = !DILocation(line: 113, column: 1, scope: !151)
!168 = distinct !DISubprogram(name: "test_context_with_branch", scope: !1, file: !1, line: 115, type: !27, scopeLine: 115, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!169 = !DILocalVariable(name: "a", scope: !168, file: !1, line: 116, type: !8)
!170 = !DILocation(line: 116, column: 9, scope: !168)
!171 = !DILocalVariable(name: "r", scope: !168, file: !1, line: 117, type: !20)
!172 = !DILocation(line: 117, column: 11, scope: !168)
!173 = !DILocation(line: 117, column: 15, scope: !168)
!174 = !DILocation(line: 118, column: 11, scope: !168)
!175 = !DILocation(line: 119, column: 1, scope: !168)
!176 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 121, type: !177, scopeLine: 121, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!177 = !DISubroutineType(types: !178)
!178 = !{!8}
!179 = !DILocation(line: 122, column: 5, scope: !176)
!180 = !DILocation(line: 123, column: 5, scope: !176)
!181 = !DILocation(line: 124, column: 5, scope: !176)
!182 = !DILocation(line: 125, column: 5, scope: !176)
!183 = !DILocation(line: 126, column: 5, scope: !176)
!184 = !DILocation(line: 127, column: 5, scope: !176)
!185 = !DILocation(line: 128, column: 5, scope: !176)
!186 = !DILocation(line: 129, column: 5, scope: !176)
