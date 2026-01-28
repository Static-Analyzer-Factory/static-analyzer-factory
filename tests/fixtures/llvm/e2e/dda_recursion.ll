; ModuleID = 'dda_recursion.cpp'
source_filename = "dda_recursion.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Node = type { i32, ptr }

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local noundef i32 @_Z8sum_listP4Node(ptr noundef %0) #0 !dbg !221 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !230, metadata !DIExpression()), !dbg !231
  %4 = load ptr, ptr %3, align 8, !dbg !232
  %5 = icmp eq ptr %4, null, !dbg !234
  br i1 %5, label %6, label %7, !dbg !235

6:                                                ; preds = %1
  store i32 0, ptr %2, align 4, !dbg !236
  br label %16, !dbg !236

7:                                                ; preds = %1
  %8 = load ptr, ptr %3, align 8, !dbg !238
  %9 = getelementptr inbounds %struct.Node, ptr %8, i32 0, i32 0, !dbg !239
  %10 = load i32, ptr %9, align 8, !dbg !239
  %11 = load ptr, ptr %3, align 8, !dbg !240
  %12 = getelementptr inbounds %struct.Node, ptr %11, i32 0, i32 1, !dbg !241
  %13 = load ptr, ptr %12, align 8, !dbg !241
  %14 = call noundef i32 @_Z8sum_listP4Node(ptr noundef %13), !dbg !242
  %15 = add nsw i32 %10, %14, !dbg !243
  store i32 %15, ptr %2, align 4, !dbg !244
  br label %16, !dbg !244

16:                                               ; preds = %7, %6
  %17 = load i32, ptr %2, align 4, !dbg !245
  ret i32 %17, !dbg !245
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #2 !dbg !246 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !247, metadata !DIExpression()), !dbg !248
  %6 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #5, !dbg !249, !heapallocsite !225
  %7 = getelementptr inbounds %struct.Node, ptr %6, i32 0, i32 0, !dbg !250
  store i32 1, ptr %7, align 16, !dbg !250
  %8 = getelementptr inbounds %struct.Node, ptr %6, i32 0, i32 1, !dbg !250
  store ptr null, ptr %8, align 8, !dbg !250
  store ptr %6, ptr %2, align 8, !dbg !248
  call void @llvm.dbg.declare(metadata ptr %3, metadata !251, metadata !DIExpression()), !dbg !252
  %9 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #5, !dbg !253, !heapallocsite !225
  %10 = getelementptr inbounds %struct.Node, ptr %9, i32 0, i32 0, !dbg !254
  store i32 2, ptr %10, align 16, !dbg !254
  %11 = getelementptr inbounds %struct.Node, ptr %9, i32 0, i32 1, !dbg !254
  %12 = load ptr, ptr %2, align 8, !dbg !255
  store ptr %12, ptr %11, align 8, !dbg !254
  store ptr %9, ptr %3, align 8, !dbg !252
  call void @llvm.dbg.declare(metadata ptr %4, metadata !256, metadata !DIExpression()), !dbg !257
  %13 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #5, !dbg !258, !heapallocsite !225
  %14 = getelementptr inbounds %struct.Node, ptr %13, i32 0, i32 0, !dbg !259
  store i32 3, ptr %14, align 16, !dbg !259
  %15 = getelementptr inbounds %struct.Node, ptr %13, i32 0, i32 1, !dbg !259
  %16 = load ptr, ptr %3, align 8, !dbg !260
  store ptr %16, ptr %15, align 8, !dbg !259
  store ptr %13, ptr %4, align 8, !dbg !257
  call void @llvm.dbg.declare(metadata ptr %5, metadata !261, metadata !DIExpression()), !dbg !262
  %17 = load ptr, ptr %4, align 8, !dbg !263
  %18 = call noundef i32 @_Z8sum_listP4Node(ptr noundef %17), !dbg !264
  store i32 %18, ptr %5, align 4, !dbg !262
  %19 = load ptr, ptr %2, align 8, !dbg !265
  %20 = icmp eq ptr %19, null, !dbg !266
  br i1 %20, label %22, label %21, !dbg !266

21:                                               ; preds = %0
  call void @_ZdlPv(ptr noundef %19) #6, !dbg !266
  br label %22, !dbg !266

22:                                               ; preds = %21, %0
  %23 = load ptr, ptr %3, align 8, !dbg !267
  %24 = icmp eq ptr %23, null, !dbg !268
  br i1 %24, label %26, label %25, !dbg !268

25:                                               ; preds = %22
  call void @_ZdlPv(ptr noundef %23) #6, !dbg !268
  br label %26, !dbg !268

26:                                               ; preds = %25, %22
  %27 = load ptr, ptr %4, align 8, !dbg !269
  %28 = icmp eq ptr %27, null, !dbg !270
  br i1 %28, label %30, label %29, !dbg !270

29:                                               ; preds = %26
  call void @_ZdlPv(ptr noundef %27) #6, !dbg !270
  br label %30, !dbg !270

30:                                               ; preds = %29, %26
  %31 = load i32, ptr %5, align 4, !dbg !271
  ret i32 %31, !dbg !272
}

; Function Attrs: nobuiltin allocsize(0)
declare noundef nonnull ptr @_Znwm(i64 noundef) #3

; Function Attrs: nobuiltin nounwind
declare void @_ZdlPv(ptr noundef) #4

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nobuiltin allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nobuiltin nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { builtin allocsize(0) }
attributes #6 = { builtin nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!213, !214, !215, !216, !217, !218, !219}
!llvm.ident = !{!220}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, imports: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_recursion.cpp", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "f4a0ee3112734cba7bbaa0b6a09cfe87")
!2 = !{!3, !11, !15, !22, !26, !34, !39, !41, !49, !53, !57, !67, !69, !73, !77, !81, !86, !90, !94, !98, !102, !110, !114, !118, !120, !124, !128, !133, !139, !143, !147, !149, !157, !161, !169, !171, !175, !179, !183, !187, !192, !197, !202, !203, !204, !205, !207, !208, !209, !210, !211, !212}
!3 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !5, file: !10, line: 52)
!4 = !DINamespace(name: "std", scope: null)
!5 = !DISubprogram(name: "abs", scope: !6, file: !6, line: 980, type: !7, flags: DIFlagPrototyped, spFlags: 0)
!6 = !DIFile(filename: "/usr/include/stdlib.h", directory: "", checksumkind: CSK_MD5, checksum: "7fa2ecb2348a66f8b44ab9a15abd0b72")
!7 = !DISubroutineType(types: !8)
!8 = !{!9, !9}
!9 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!10 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/bits/std_abs.h", directory: "")
!11 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !12, file: !14, line: 131)
!12 = !DIDerivedType(tag: DW_TAG_typedef, name: "div_t", file: !6, line: 63, baseType: !13)
!13 = !DICompositeType(tag: DW_TAG_structure_type, file: !6, line: 59, size: 64, flags: DIFlagFwdDecl, identifier: "_ZTS5div_t")
!14 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdlib", directory: "")
!15 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !16, file: !14, line: 132)
!16 = !DIDerivedType(tag: DW_TAG_typedef, name: "ldiv_t", file: !6, line: 71, baseType: !17)
!17 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !6, line: 67, size: 128, flags: DIFlagTypePassByValue, elements: !18, identifier: "_ZTS6ldiv_t")
!18 = !{!19, !21}
!19 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !17, file: !6, line: 69, baseType: !20, size: 64)
!20 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!21 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !17, file: !6, line: 70, baseType: !20, size: 64, offset: 64)
!22 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !23, file: !14, line: 134)
!23 = !DISubprogram(name: "abort", scope: !6, file: !6, line: 730, type: !24, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!24 = !DISubroutineType(types: !25)
!25 = !{null}
!26 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !27, file: !14, line: 136)
!27 = !DISubprogram(name: "aligned_alloc", scope: !6, file: !6, line: 724, type: !28, flags: DIFlagPrototyped, spFlags: 0)
!28 = !DISubroutineType(types: !29)
!29 = !{!30, !31, !31}
!30 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!31 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !32, line: 18, baseType: !33)
!32 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!33 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!34 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !35, file: !14, line: 138)
!35 = !DISubprogram(name: "atexit", scope: !6, file: !6, line: 734, type: !36, flags: DIFlagPrototyped, spFlags: 0)
!36 = !DISubroutineType(types: !37)
!37 = !{!9, !38}
!38 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !24, size: 64)
!39 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !40, file: !14, line: 141)
!40 = !DISubprogram(name: "at_quick_exit", scope: !6, file: !6, line: 739, type: !36, flags: DIFlagPrototyped, spFlags: 0)
!41 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !42, file: !14, line: 144)
!42 = !DISubprogram(name: "atof", scope: !6, file: !6, line: 102, type: !43, flags: DIFlagPrototyped, spFlags: 0)
!43 = !DISubroutineType(types: !44)
!44 = !{!45, !46}
!45 = !DIBasicType(name: "double", size: 64, encoding: DW_ATE_float)
!46 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !47, size: 64)
!47 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !48)
!48 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!49 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !50, file: !14, line: 145)
!50 = !DISubprogram(name: "atoi", scope: !6, file: !6, line: 105, type: !51, flags: DIFlagPrototyped, spFlags: 0)
!51 = !DISubroutineType(types: !52)
!52 = !{!9, !46}
!53 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !54, file: !14, line: 146)
!54 = !DISubprogram(name: "atol", scope: !6, file: !6, line: 108, type: !55, flags: DIFlagPrototyped, spFlags: 0)
!55 = !DISubroutineType(types: !56)
!56 = !{!20, !46}
!57 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !58, file: !14, line: 147)
!58 = !DISubprogram(name: "bsearch", scope: !6, file: !6, line: 960, type: !59, flags: DIFlagPrototyped, spFlags: 0)
!59 = !DISubroutineType(types: !60)
!60 = !{!30, !61, !61, !31, !31, !63}
!61 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !62, size: 64)
!62 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!63 = !DIDerivedType(tag: DW_TAG_typedef, name: "__compar_fn_t", file: !6, line: 948, baseType: !64)
!64 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !65, size: 64)
!65 = !DISubroutineType(types: !66)
!66 = !{!9, !61, !61}
!67 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !68, file: !14, line: 148)
!68 = !DISubprogram(name: "calloc", scope: !6, file: !6, line: 675, type: !28, flags: DIFlagPrototyped, spFlags: 0)
!69 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !70, file: !14, line: 149)
!70 = !DISubprogram(name: "div", scope: !6, file: !6, line: 992, type: !71, flags: DIFlagPrototyped, spFlags: 0)
!71 = !DISubroutineType(types: !72)
!72 = !{!12, !9, !9}
!73 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !74, file: !14, line: 150)
!74 = !DISubprogram(name: "exit", scope: !6, file: !6, line: 756, type: !75, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!75 = !DISubroutineType(types: !76)
!76 = !{null, !9}
!77 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !78, file: !14, line: 151)
!78 = !DISubprogram(name: "free", scope: !6, file: !6, line: 687, type: !79, flags: DIFlagPrototyped, spFlags: 0)
!79 = !DISubroutineType(types: !80)
!80 = !{null, !30}
!81 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !82, file: !14, line: 152)
!82 = !DISubprogram(name: "getenv", scope: !6, file: !6, line: 773, type: !83, flags: DIFlagPrototyped, spFlags: 0)
!83 = !DISubroutineType(types: !84)
!84 = !{!85, !46}
!85 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !48, size: 64)
!86 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !87, file: !14, line: 153)
!87 = !DISubprogram(name: "labs", scope: !6, file: !6, line: 981, type: !88, flags: DIFlagPrototyped, spFlags: 0)
!88 = !DISubroutineType(types: !89)
!89 = !{!20, !20}
!90 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !91, file: !14, line: 154)
!91 = !DISubprogram(name: "ldiv", scope: !6, file: !6, line: 994, type: !92, flags: DIFlagPrototyped, spFlags: 0)
!92 = !DISubroutineType(types: !93)
!93 = !{!16, !20, !20}
!94 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !95, file: !14, line: 155)
!95 = !DISubprogram(name: "malloc", scope: !6, file: !6, line: 672, type: !96, flags: DIFlagPrototyped, spFlags: 0)
!96 = !DISubroutineType(types: !97)
!97 = !{!30, !31}
!98 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !99, file: !14, line: 157)
!99 = !DISubprogram(name: "mblen", scope: !6, file: !6, line: 1062, type: !100, flags: DIFlagPrototyped, spFlags: 0)
!100 = !DISubroutineType(types: !101)
!101 = !{!9, !46, !31}
!102 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !103, file: !14, line: 158)
!103 = !DISubprogram(name: "mbstowcs", scope: !6, file: !6, line: 1073, type: !104, flags: DIFlagPrototyped, spFlags: 0)
!104 = !DISubroutineType(types: !105)
!105 = !{!31, !106, !109, !31}
!106 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !107)
!107 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !108, size: 64)
!108 = !DIBasicType(name: "wchar_t", size: 32, encoding: DW_ATE_unsigned)
!109 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !46)
!110 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !111, file: !14, line: 159)
!111 = !DISubprogram(name: "mbtowc", scope: !6, file: !6, line: 1065, type: !112, flags: DIFlagPrototyped, spFlags: 0)
!112 = !DISubroutineType(types: !113)
!113 = !{!9, !106, !109, !31}
!114 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !115, file: !14, line: 161)
!115 = !DISubprogram(name: "qsort", scope: !6, file: !6, line: 970, type: !116, flags: DIFlagPrototyped, spFlags: 0)
!116 = !DISubroutineType(types: !117)
!117 = !{null, !30, !31, !31, !63}
!118 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !119, file: !14, line: 164)
!119 = !DISubprogram(name: "quick_exit", scope: !6, file: !6, line: 762, type: !75, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!120 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !121, file: !14, line: 167)
!121 = !DISubprogram(name: "rand", scope: !6, file: !6, line: 573, type: !122, flags: DIFlagPrototyped, spFlags: 0)
!122 = !DISubroutineType(types: !123)
!123 = !{!9}
!124 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !125, file: !14, line: 168)
!125 = !DISubprogram(name: "realloc", scope: !6, file: !6, line: 683, type: !126, flags: DIFlagPrototyped, spFlags: 0)
!126 = !DISubroutineType(types: !127)
!127 = !{!30, !30, !31}
!128 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !129, file: !14, line: 169)
!129 = !DISubprogram(name: "srand", scope: !6, file: !6, line: 575, type: !130, flags: DIFlagPrototyped, spFlags: 0)
!130 = !DISubroutineType(types: !131)
!131 = !{null, !132}
!132 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!133 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !134, file: !14, line: 170)
!134 = !DISubprogram(name: "strtod", scope: !6, file: !6, line: 118, type: !135, flags: DIFlagPrototyped, spFlags: 0)
!135 = !DISubroutineType(types: !136)
!136 = !{!45, !109, !137}
!137 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !138)
!138 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !85, size: 64)
!139 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !140, file: !14, line: 171)
!140 = !DISubprogram(name: "strtol", linkageName: "__isoc23_strtol", scope: !6, file: !6, line: 215, type: !141, flags: DIFlagPrototyped, spFlags: 0)
!141 = !DISubroutineType(types: !142)
!142 = !{!20, !109, !137, !9}
!143 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !144, file: !14, line: 172)
!144 = !DISubprogram(name: "strtoul", linkageName: "__isoc23_strtoul", scope: !6, file: !6, line: 219, type: !145, flags: DIFlagPrototyped, spFlags: 0)
!145 = !DISubroutineType(types: !146)
!146 = !{!33, !109, !137, !9}
!147 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !148, file: !14, line: 173)
!148 = !DISubprogram(name: "system", scope: !6, file: !6, line: 923, type: !51, flags: DIFlagPrototyped, spFlags: 0)
!149 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !150, file: !14, line: 175)
!150 = !DISubprogram(name: "wcstombs", scope: !6, file: !6, line: 1077, type: !151, flags: DIFlagPrototyped, spFlags: 0)
!151 = !DISubroutineType(types: !152)
!152 = !{!31, !153, !154, !31}
!153 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !85)
!154 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !155)
!155 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !156, size: 64)
!156 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !108)
!157 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !158, file: !14, line: 176)
!158 = !DISubprogram(name: "wctomb", scope: !6, file: !6, line: 1069, type: !159, flags: DIFlagPrototyped, spFlags: 0)
!159 = !DISubroutineType(types: !160)
!160 = !{!9, !85, !108}
!161 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !163, file: !14, line: 204)
!162 = !DINamespace(name: "__gnu_cxx", scope: null)
!163 = !DIDerivedType(tag: DW_TAG_typedef, name: "lldiv_t", file: !6, line: 81, baseType: !164)
!164 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !6, line: 77, size: 128, flags: DIFlagTypePassByValue, elements: !165, identifier: "_ZTS7lldiv_t")
!165 = !{!166, !168}
!166 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !164, file: !6, line: 79, baseType: !167, size: 64)
!167 = !DIBasicType(name: "long long", size: 64, encoding: DW_ATE_signed)
!168 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !164, file: !6, line: 80, baseType: !167, size: 64, offset: 64)
!169 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !170, file: !14, line: 210)
!170 = !DISubprogram(name: "_Exit", scope: !6, file: !6, line: 768, type: !75, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!171 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !172, file: !14, line: 214)
!172 = !DISubprogram(name: "llabs", scope: !6, file: !6, line: 984, type: !173, flags: DIFlagPrototyped, spFlags: 0)
!173 = !DISubroutineType(types: !174)
!174 = !{!167, !167}
!175 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !176, file: !14, line: 220)
!176 = !DISubprogram(name: "lldiv", scope: !6, file: !6, line: 998, type: !177, flags: DIFlagPrototyped, spFlags: 0)
!177 = !DISubroutineType(types: !178)
!178 = !{!163, !167, !167}
!179 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !180, file: !14, line: 231)
!180 = !DISubprogram(name: "atoll", scope: !6, file: !6, line: 113, type: !181, flags: DIFlagPrototyped, spFlags: 0)
!181 = !DISubroutineType(types: !182)
!182 = !{!167, !46}
!183 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !184, file: !14, line: 232)
!184 = !DISubprogram(name: "strtoll", linkageName: "__isoc23_strtoll", scope: !6, file: !6, line: 238, type: !185, flags: DIFlagPrototyped, spFlags: 0)
!185 = !DISubroutineType(types: !186)
!186 = !{!167, !109, !137, !9}
!187 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !188, file: !14, line: 233)
!188 = !DISubprogram(name: "strtoull", linkageName: "__isoc23_strtoull", scope: !6, file: !6, line: 243, type: !189, flags: DIFlagPrototyped, spFlags: 0)
!189 = !DISubroutineType(types: !190)
!190 = !{!191, !109, !137, !9}
!191 = !DIBasicType(name: "unsigned long long", size: 64, encoding: DW_ATE_unsigned)
!192 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !193, file: !14, line: 235)
!193 = !DISubprogram(name: "strtof", scope: !6, file: !6, line: 124, type: !194, flags: DIFlagPrototyped, spFlags: 0)
!194 = !DISubroutineType(types: !195)
!195 = !{!196, !109, !137}
!196 = !DIBasicType(name: "float", size: 32, encoding: DW_ATE_float)
!197 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !162, entity: !198, file: !14, line: 236)
!198 = !DISubprogram(name: "strtold", scope: !6, file: !6, line: 127, type: !199, flags: DIFlagPrototyped, spFlags: 0)
!199 = !DISubroutineType(types: !200)
!200 = !{!201, !109, !137}
!201 = !DIBasicType(name: "long double", size: 128, encoding: DW_ATE_float)
!202 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !163, file: !14, line: 244)
!203 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !170, file: !14, line: 246)
!204 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !172, file: !14, line: 248)
!205 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !206, file: !14, line: 249)
!206 = !DISubprogram(name: "div", linkageName: "_ZN9__gnu_cxx3divExx", scope: !162, file: !14, line: 217, type: !177, flags: DIFlagPrototyped, spFlags: 0)
!207 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !176, file: !14, line: 250)
!208 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !180, file: !14, line: 252)
!209 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !193, file: !14, line: 253)
!210 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !184, file: !14, line: 254)
!211 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !188, file: !14, line: 255)
!212 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !4, entity: !198, file: !14, line: 256)
!213 = !{i32 7, !"Dwarf Version", i32 5}
!214 = !{i32 2, !"Debug Info Version", i32 3}
!215 = !{i32 1, !"wchar_size", i32 4}
!216 = !{i32 8, !"PIC Level", i32 2}
!217 = !{i32 7, !"PIE Level", i32 2}
!218 = !{i32 7, !"uwtable", i32 2}
!219 = !{i32 7, !"frame-pointer", i32 1}
!220 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!221 = distinct !DISubprogram(name: "sum_list", linkageName: "_Z8sum_listP4Node", scope: !1, file: !1, line: 13, type: !222, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !229)
!222 = !DISubroutineType(types: !223)
!223 = !{!9, !224}
!224 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !225, size: 64)
!225 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Node", file: !1, line: 7, size: 128, flags: DIFlagTypePassByValue, elements: !226, identifier: "_ZTS4Node")
!226 = !{!227, !228}
!227 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !225, file: !1, line: 8, baseType: !9, size: 32)
!228 = !DIDerivedType(tag: DW_TAG_member, name: "next", scope: !225, file: !1, line: 9, baseType: !224, size: 64, offset: 64)
!229 = !{}
!230 = !DILocalVariable(name: "node", arg: 1, scope: !221, file: !1, line: 13, type: !224)
!231 = !DILocation(line: 13, column: 20, scope: !221)
!232 = !DILocation(line: 14, column: 9, scope: !233)
!233 = distinct !DILexicalBlock(scope: !221, file: !1, line: 14, column: 9)
!234 = !DILocation(line: 14, column: 14, scope: !233)
!235 = !DILocation(line: 14, column: 9, scope: !221)
!236 = !DILocation(line: 15, column: 9, scope: !237)
!237 = distinct !DILexicalBlock(scope: !233, file: !1, line: 14, column: 26)
!238 = !DILocation(line: 17, column: 12, scope: !221)
!239 = !DILocation(line: 17, column: 18, scope: !221)
!240 = !DILocation(line: 17, column: 35, scope: !221)
!241 = !DILocation(line: 17, column: 41, scope: !221)
!242 = !DILocation(line: 17, column: 26, scope: !221)
!243 = !DILocation(line: 17, column: 24, scope: !221)
!244 = !DILocation(line: 17, column: 5, scope: !221)
!245 = !DILocation(line: 18, column: 1, scope: !221)
!246 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 20, type: !122, scopeLine: 20, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !229)
!247 = !DILocalVariable(name: "n1", scope: !246, file: !1, line: 22, type: !224)
!248 = !DILocation(line: 22, column: 11, scope: !246)
!249 = !DILocation(line: 22, column: 16, scope: !246)
!250 = !DILocation(line: 22, column: 24, scope: !246)
!251 = !DILocalVariable(name: "n2", scope: !246, file: !1, line: 23, type: !224)
!252 = !DILocation(line: 23, column: 11, scope: !246)
!253 = !DILocation(line: 23, column: 16, scope: !246)
!254 = !DILocation(line: 23, column: 24, scope: !246)
!255 = !DILocation(line: 23, column: 28, scope: !246)
!256 = !DILocalVariable(name: "n3", scope: !246, file: !1, line: 24, type: !224)
!257 = !DILocation(line: 24, column: 11, scope: !246)
!258 = !DILocation(line: 24, column: 16, scope: !246)
!259 = !DILocation(line: 24, column: 24, scope: !246)
!260 = !DILocation(line: 24, column: 28, scope: !246)
!261 = !DILocalVariable(name: "total", scope: !246, file: !1, line: 27, type: !9)
!262 = !DILocation(line: 27, column: 9, scope: !246)
!263 = !DILocation(line: 27, column: 26, scope: !246)
!264 = !DILocation(line: 27, column: 17, scope: !246)
!265 = !DILocation(line: 29, column: 12, scope: !246)
!266 = !DILocation(line: 29, column: 5, scope: !246)
!267 = !DILocation(line: 30, column: 12, scope: !246)
!268 = !DILocation(line: 30, column: 5, scope: !246)
!269 = !DILocation(line: 31, column: 12, scope: !246)
!270 = !DILocation(line: 31, column: 5, scope: !246)
!271 = !DILocation(line: 33, column: 12, scope: !246)
!272 = !DILocation(line: 33, column: 5, scope: !246)
