; ModuleID = '/workspace/tests/programs/cpp/dangling_ptr_cpp.cpp'
source_filename = "/workspace/tests/programs/cpp/dangling_ptr_cpp.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1, !dbg !0

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local noundef nonnull align 4 dereferenceable(4) ptr @_Z9get_valuev() #0 !dbg !217 {
  %1 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !222, metadata !DIExpression()), !dbg !223
  store i32 42, ptr %1, align 4, !dbg !223
  ret ptr %1, !dbg !224
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #2 !dbg !225 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !226, metadata !DIExpression()), !dbg !227
  %3 = call noundef nonnull align 4 dereferenceable(4) ptr @_Z9get_valuev(), !dbg !228
  store ptr %3, ptr %2, align 8, !dbg !227
  %4 = load ptr, ptr %2, align 8, !dbg !229
  %5 = load i32, ptr %4, align 4, !dbg !229
  %6 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %5), !dbg !230
  ret i32 0, !dbg !231
}

declare i32 @printf(ptr noundef, ...) #3

attributes #0 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!8}
!llvm.module.flags = !{!209, !210, !211, !212, !213, !214, !215}
!llvm.ident = !{!216}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 15, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "cpp/dangling_ptr_cpp.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "176d937a3457bcac6afcd3757bead17b")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !6)
!4 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !5)
!5 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!6 = !{!7}
!7 = !DISubrange(count: 4)
!8 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !9, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !10, imports: !11, splitDebugInlining: false, nameTableKind: None)
!9 = !DIFile(filename: "/workspace/tests/programs/cpp/dangling_ptr_cpp.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "176d937a3457bcac6afcd3757bead17b")
!10 = !{!0}
!11 = !{!12, !19, !25, !30, !35, !37, !39, !41, !43, !50, !56, !62, !66, !70, !74, !83, !87, !89, !94, !100, !104, !111, !113, !117, !121, !125, !127, !131, !135, !137, !141, !143, !145, !149, !153, !157, !161, !165, !169, !171, !179, !183, !187, !192, !194, !196, !200, !204, !205, !206, !207, !208}
!12 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !14, file: !18, line: 98)
!13 = !DINamespace(name: "std", scope: null)
!14 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !15, line: 7, baseType: !16)
!15 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!16 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !17, line: 49, size: 1728, flags: DIFlagFwdDecl, identifier: "_ZTS8_IO_FILE")
!17 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!18 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdio", directory: "")
!19 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !20, file: !18, line: 99)
!20 = !DIDerivedType(tag: DW_TAG_typedef, name: "fpos_t", file: !21, line: 85, baseType: !22)
!21 = !DIFile(filename: "/usr/include/stdio.h", directory: "", checksumkind: CSK_MD5, checksum: "1e435c46987a169d9f9186f63a512303")
!22 = !DIDerivedType(tag: DW_TAG_typedef, name: "__fpos_t", file: !23, line: 14, baseType: !24)
!23 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/__fpos_t.h", directory: "", checksumkind: CSK_MD5, checksum: "32de8bdaf3551a6c0a9394f9af4389ce")
!24 = !DICompositeType(tag: DW_TAG_structure_type, name: "_G_fpos_t", file: !23, line: 10, size: 128, flags: DIFlagFwdDecl, identifier: "_ZTS9_G_fpos_t")
!25 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !26, file: !18, line: 101)
!26 = !DISubprogram(name: "clearerr", scope: !21, file: !21, line: 860, type: !27, flags: DIFlagPrototyped, spFlags: 0)
!27 = !DISubroutineType(types: !28)
!28 = !{null, !29}
!29 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!30 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !31, file: !18, line: 102)
!31 = !DISubprogram(name: "fclose", scope: !21, file: !21, line: 184, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!32 = !DISubroutineType(types: !33)
!33 = !{!34, !29}
!34 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!35 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !36, file: !18, line: 103)
!36 = !DISubprogram(name: "feof", scope: !21, file: !21, line: 862, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!37 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !38, file: !18, line: 104)
!38 = !DISubprogram(name: "ferror", scope: !21, file: !21, line: 864, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!39 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !40, file: !18, line: 105)
!40 = !DISubprogram(name: "fflush", scope: !21, file: !21, line: 236, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!41 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !42, file: !18, line: 106)
!42 = !DISubprogram(name: "fgetc", scope: !21, file: !21, line: 575, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!43 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !44, file: !18, line: 107)
!44 = !DISubprogram(name: "fgetpos", scope: !21, file: !21, line: 829, type: !45, flags: DIFlagPrototyped, spFlags: 0)
!45 = !DISubroutineType(types: !46)
!46 = !{!34, !47, !48}
!47 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !29)
!48 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !49)
!49 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !20, size: 64)
!50 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !51, file: !18, line: 108)
!51 = !DISubprogram(name: "fgets", scope: !21, file: !21, line: 654, type: !52, flags: DIFlagPrototyped, spFlags: 0)
!52 = !DISubroutineType(types: !53)
!53 = !{!54, !55, !34, !47}
!54 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!55 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !54)
!56 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !57, file: !18, line: 109)
!57 = !DISubprogram(name: "fopen", scope: !21, file: !21, line: 264, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!58 = !DISubroutineType(types: !59)
!59 = !{!29, !60, !60}
!60 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !61)
!61 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!62 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !63, file: !18, line: 110)
!63 = !DISubprogram(name: "fprintf", scope: !21, file: !21, line: 357, type: !64, flags: DIFlagPrototyped, spFlags: 0)
!64 = !DISubroutineType(types: !65)
!65 = !{!34, !47, !60, null}
!66 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !67, file: !18, line: 111)
!67 = !DISubprogram(name: "fputc", scope: !21, file: !21, line: 611, type: !68, flags: DIFlagPrototyped, spFlags: 0)
!68 = !DISubroutineType(types: !69)
!69 = !{!34, !34, !29}
!70 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !71, file: !18, line: 112)
!71 = !DISubprogram(name: "fputs", scope: !21, file: !21, line: 717, type: !72, flags: DIFlagPrototyped, spFlags: 0)
!72 = !DISubroutineType(types: !73)
!73 = !{!34, !60, !47}
!74 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !75, file: !18, line: 113)
!75 = !DISubprogram(name: "fread", scope: !21, file: !21, line: 738, type: !76, flags: DIFlagPrototyped, spFlags: 0)
!76 = !DISubroutineType(types: !77)
!77 = !{!78, !81, !78, !78, !47}
!78 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !79, line: 18, baseType: !80)
!79 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!80 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!81 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !82)
!82 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!83 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !84, file: !18, line: 114)
!84 = !DISubprogram(name: "freopen", scope: !21, file: !21, line: 271, type: !85, flags: DIFlagPrototyped, spFlags: 0)
!85 = !DISubroutineType(types: !86)
!86 = !{!29, !60, !60, !47}
!87 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !88, file: !18, line: 115)
!88 = !DISubprogram(name: "fscanf", linkageName: "__isoc23_fscanf", scope: !21, file: !21, line: 442, type: !64, flags: DIFlagPrototyped, spFlags: 0)
!89 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !90, file: !18, line: 116)
!90 = !DISubprogram(name: "fseek", scope: !21, file: !21, line: 779, type: !91, flags: DIFlagPrototyped, spFlags: 0)
!91 = !DISubroutineType(types: !92)
!92 = !{!34, !29, !93, !34}
!93 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!94 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !95, file: !18, line: 117)
!95 = !DISubprogram(name: "fsetpos", scope: !21, file: !21, line: 835, type: !96, flags: DIFlagPrototyped, spFlags: 0)
!96 = !DISubroutineType(types: !97)
!97 = !{!34, !29, !98}
!98 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !99, size: 64)
!99 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !20)
!100 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !101, file: !18, line: 118)
!101 = !DISubprogram(name: "ftell", scope: !21, file: !21, line: 785, type: !102, flags: DIFlagPrototyped, spFlags: 0)
!102 = !DISubroutineType(types: !103)
!103 = !{!93, !29}
!104 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !105, file: !18, line: 119)
!105 = !DISubprogram(name: "fwrite", scope: !21, file: !21, line: 745, type: !106, flags: DIFlagPrototyped, spFlags: 0)
!106 = !DISubroutineType(types: !107)
!107 = !{!78, !108, !78, !78, !47}
!108 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !109)
!109 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !110, size: 64)
!110 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!111 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !112, file: !18, line: 120)
!112 = !DISubprogram(name: "getc", scope: !21, file: !21, line: 576, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!113 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !114, file: !18, line: 121)
!114 = !DISubprogram(name: "getchar", scope: !21, file: !21, line: 582, type: !115, flags: DIFlagPrototyped, spFlags: 0)
!115 = !DISubroutineType(types: !116)
!116 = !{!34}
!117 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !118, file: !18, line: 126)
!118 = !DISubprogram(name: "perror", scope: !21, file: !21, line: 878, type: !119, flags: DIFlagPrototyped, spFlags: 0)
!119 = !DISubroutineType(types: !120)
!120 = !{null, !61}
!121 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !122, file: !18, line: 127)
!122 = !DISubprogram(name: "printf", scope: !21, file: !21, line: 363, type: !123, flags: DIFlagPrototyped, spFlags: 0)
!123 = !DISubroutineType(types: !124)
!124 = !{!34, !60, null}
!125 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !126, file: !18, line: 128)
!126 = !DISubprogram(name: "putc", scope: !21, file: !21, line: 612, type: !68, flags: DIFlagPrototyped, spFlags: 0)
!127 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !128, file: !18, line: 129)
!128 = !DISubprogram(name: "putchar", scope: !21, file: !21, line: 618, type: !129, flags: DIFlagPrototyped, spFlags: 0)
!129 = !DISubroutineType(types: !130)
!130 = !{!34, !34}
!131 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !132, file: !18, line: 130)
!132 = !DISubprogram(name: "puts", scope: !21, file: !21, line: 724, type: !133, flags: DIFlagPrototyped, spFlags: 0)
!133 = !DISubroutineType(types: !134)
!134 = !{!34, !61}
!135 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !136, file: !18, line: 131)
!136 = !DISubprogram(name: "remove", scope: !21, file: !21, line: 158, type: !133, flags: DIFlagPrototyped, spFlags: 0)
!137 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !138, file: !18, line: 132)
!138 = !DISubprogram(name: "rename", scope: !21, file: !21, line: 160, type: !139, flags: DIFlagPrototyped, spFlags: 0)
!139 = !DISubroutineType(types: !140)
!140 = !{!34, !61, !61}
!141 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !142, file: !18, line: 133)
!142 = !DISubprogram(name: "rewind", scope: !21, file: !21, line: 790, type: !27, flags: DIFlagPrototyped, spFlags: 0)
!143 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !144, file: !18, line: 134)
!144 = !DISubprogram(name: "scanf", linkageName: "__isoc23_scanf", scope: !21, file: !21, line: 445, type: !123, flags: DIFlagPrototyped, spFlags: 0)
!145 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !146, file: !18, line: 135)
!146 = !DISubprogram(name: "setbuf", scope: !21, file: !21, line: 334, type: !147, flags: DIFlagPrototyped, spFlags: 0)
!147 = !DISubroutineType(types: !148)
!148 = !{null, !47, !55}
!149 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !150, file: !18, line: 136)
!150 = !DISubprogram(name: "setvbuf", scope: !21, file: !21, line: 339, type: !151, flags: DIFlagPrototyped, spFlags: 0)
!151 = !DISubroutineType(types: !152)
!152 = !{!34, !47, !55, !34, !78}
!153 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !154, file: !18, line: 137)
!154 = !DISubprogram(name: "sprintf", scope: !21, file: !21, line: 365, type: !155, flags: DIFlagPrototyped, spFlags: 0)
!155 = !DISubroutineType(types: !156)
!156 = !{!34, !55, !60, null}
!157 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !158, file: !18, line: 138)
!158 = !DISubprogram(name: "sscanf", linkageName: "__isoc23_sscanf", scope: !21, file: !21, line: 447, type: !159, flags: DIFlagPrototyped, spFlags: 0)
!159 = !DISubroutineType(types: !160)
!160 = !{!34, !60, !60, null}
!161 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !162, file: !18, line: 139)
!162 = !DISubprogram(name: "tmpfile", scope: !21, file: !21, line: 194, type: !163, flags: DIFlagPrototyped, spFlags: 0)
!163 = !DISubroutineType(types: !164)
!164 = !{!29}
!165 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !166, file: !18, line: 141)
!166 = !DISubprogram(name: "tmpnam", scope: !21, file: !21, line: 211, type: !167, flags: DIFlagPrototyped, spFlags: 0)
!167 = !DISubroutineType(types: !168)
!168 = !{!54, !54}
!169 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !170, file: !18, line: 143)
!170 = !DISubprogram(name: "ungetc", scope: !21, file: !21, line: 731, type: !68, flags: DIFlagPrototyped, spFlags: 0)
!171 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !172, file: !18, line: 144)
!172 = !DISubprogram(name: "vfprintf", scope: !21, file: !21, line: 372, type: !173, flags: DIFlagPrototyped, spFlags: 0)
!173 = !DISubroutineType(types: !174)
!174 = !{!34, !47, !60, !175}
!175 = !DIDerivedType(tag: DW_TAG_typedef, name: "__gnuc_va_list", file: !176, line: 12, baseType: !177)
!176 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stdarg___gnuc_va_list.h", directory: "", checksumkind: CSK_MD5, checksum: "edb3f2eab991638e4dc94f6e55e3530f")
!177 = !DIDerivedType(tag: DW_TAG_typedef, name: "__builtin_va_list", file: !2, baseType: !178)
!178 = !DICompositeType(tag: DW_TAG_structure_type, name: "__va_list", scope: !13, file: !2, size: 256, flags: DIFlagFwdDecl, identifier: "_ZTSSt9__va_list")
!179 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !180, file: !18, line: 145)
!180 = !DISubprogram(name: "vprintf", scope: !21, file: !21, line: 378, type: !181, flags: DIFlagPrototyped, spFlags: 0)
!181 = !DISubroutineType(types: !182)
!182 = !{!34, !60, !175}
!183 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !184, file: !18, line: 146)
!184 = !DISubprogram(name: "vsprintf", scope: !21, file: !21, line: 380, type: !185, flags: DIFlagPrototyped, spFlags: 0)
!185 = !DISubroutineType(types: !186)
!186 = !{!34, !55, !60, !175}
!187 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !188, entity: !189, file: !18, line: 175)
!188 = !DINamespace(name: "__gnu_cxx", scope: null)
!189 = !DISubprogram(name: "snprintf", scope: !21, file: !21, line: 385, type: !190, flags: DIFlagPrototyped, spFlags: 0)
!190 = !DISubroutineType(types: !191)
!191 = !{!34, !55, !78, !60, null}
!192 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !188, entity: !193, file: !18, line: 176)
!193 = !DISubprogram(name: "vfscanf", linkageName: "__isoc23_vfscanf", scope: !21, file: !21, line: 511, type: !173, flags: DIFlagPrototyped, spFlags: 0)
!194 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !188, entity: !195, file: !18, line: 177)
!195 = !DISubprogram(name: "vscanf", linkageName: "__isoc23_vscanf", scope: !21, file: !21, line: 516, type: !181, flags: DIFlagPrototyped, spFlags: 0)
!196 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !188, entity: !197, file: !18, line: 178)
!197 = !DISubprogram(name: "vsnprintf", scope: !21, file: !21, line: 389, type: !198, flags: DIFlagPrototyped, spFlags: 0)
!198 = !DISubroutineType(types: !199)
!199 = !{!34, !55, !78, !60, !175}
!200 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !188, entity: !201, file: !18, line: 179)
!201 = !DISubprogram(name: "vsscanf", linkageName: "__isoc23_vsscanf", scope: !21, file: !21, line: 519, type: !202, flags: DIFlagPrototyped, spFlags: 0)
!202 = !DISubroutineType(types: !203)
!203 = !{!34, !60, !60, !175}
!204 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !189, file: !18, line: 185)
!205 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !193, file: !18, line: 186)
!206 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !195, file: !18, line: 187)
!207 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !197, file: !18, line: 188)
!208 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !201, file: !18, line: 189)
!209 = !{i32 7, !"Dwarf Version", i32 5}
!210 = !{i32 2, !"Debug Info Version", i32 3}
!211 = !{i32 1, !"wchar_size", i32 4}
!212 = !{i32 8, !"PIC Level", i32 2}
!213 = !{i32 7, !"PIE Level", i32 2}
!214 = !{i32 7, !"uwtable", i32 2}
!215 = !{i32 7, !"frame-pointer", i32 1}
!216 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!217 = distinct !DISubprogram(name: "get_value", linkageName: "_Z9get_valuev", scope: !2, file: !2, line: 8, type: !218, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, retainedNodes: !221)
!218 = !DISubroutineType(types: !219)
!219 = !{!220}
!220 = !DIDerivedType(tag: DW_TAG_reference_type, baseType: !34, size: 64)
!221 = !{}
!222 = !DILocalVariable(name: "local", scope: !217, file: !2, line: 9, type: !34)
!223 = !DILocation(line: 9, column: 9, scope: !217)
!224 = !DILocation(line: 10, column: 5, scope: !217)
!225 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 13, type: !115, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, retainedNodes: !221)
!226 = !DILocalVariable(name: "ref", scope: !225, file: !2, line: 14, type: !220)
!227 = !DILocation(line: 14, column: 10, scope: !225)
!228 = !DILocation(line: 14, column: 16, scope: !225)
!229 = !DILocation(line: 15, column: 20, scope: !225)
!230 = !DILocation(line: 15, column: 5, scope: !225)
!231 = !DILocation(line: 16, column: 5, scope: !225)
