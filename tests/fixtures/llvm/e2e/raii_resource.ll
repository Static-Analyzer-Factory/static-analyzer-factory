; ModuleID = '/workspace/tests/programs/cpp/raii_resource.cpp'
source_filename = "/workspace/tests/programs/cpp/raii_resource.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.FileHandle = type { ptr }

$_ZN10FileHandleC2EPKc = comdat any

$_ZN10FileHandle9read_byteEv = comdat any

$_ZN10FileHandleD2Ev = comdat any

$__clang_call_terminate = comdat any

@.str = private unnamed_addr constant [14 x i8] c"/etc/hostname\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [16 x i8] c"first byte: %d\0A\00", align 1, !dbg !8
@.str.2 = private unnamed_addr constant [2 x i8] c"r\00", align 1, !dbg !13

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #0 personality ptr @__gxx_personality_v0 !dbg !241 {
  %1 = alloca i32, align 4
  %2 = alloca %class.FileHandle, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !243, metadata !DIExpression()), !dbg !244
  call void @_ZN10FileHandleC2EPKc(ptr noundef nonnull align 8 dereferenceable(8) %2, ptr noundef @.str), !dbg !244
  call void @llvm.dbg.declare(metadata ptr %3, metadata !245, metadata !DIExpression()), !dbg !246
  %6 = invoke noundef i32 @_ZN10FileHandle9read_byteEv(ptr noundef nonnull align 8 dereferenceable(8) %2)
          to label %7 unwind label %12, !dbg !247

7:                                                ; preds = %0
  store i32 %6, ptr %3, align 4, !dbg !246
  %8 = load i32, ptr %3, align 4, !dbg !248
  %9 = invoke i32 (ptr, ...) @printf(ptr noundef @.str.1, i32 noundef %8)
          to label %10 unwind label %12, !dbg !249

10:                                               ; preds = %7
  store i32 0, ptr %1, align 4, !dbg !250
  call void @_ZN10FileHandleD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %2) #6, !dbg !251
  %11 = load i32, ptr %1, align 4, !dbg !251
  ret i32 %11, !dbg !251

12:                                               ; preds = %7, %0
  %13 = landingpad { ptr, i32 }
          cleanup, !dbg !251
  %14 = extractvalue { ptr, i32 } %13, 0, !dbg !251
  store ptr %14, ptr %4, align 8, !dbg !251
  %15 = extractvalue { ptr, i32 } %13, 1, !dbg !251
  store i32 %15, ptr %5, align 4, !dbg !251
  call void @_ZN10FileHandleD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %2) #6, !dbg !251
  br label %16, !dbg !251

16:                                               ; preds = %12
  %17 = load ptr, ptr %4, align 8, !dbg !251
  %18 = load i32, ptr %5, align 4, !dbg !251
  %19 = insertvalue { ptr, i32 } poison, ptr %17, 0, !dbg !251
  %20 = insertvalue { ptr, i32 } %19, i32 %18, 1, !dbg !251
  resume { ptr, i32 } %20, !dbg !251
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN10FileHandleC2EPKc(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #2 comdat align 2 !dbg !252 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !253, metadata !DIExpression()), !dbg !255
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !256, metadata !DIExpression()), !dbg !257
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8, !dbg !258
  %7 = call noalias ptr @fopen(ptr noundef %6, ptr noundef @.str.2), !dbg !260
  %8 = getelementptr inbounds %class.FileHandle, ptr %5, i32 0, i32 0, !dbg !261
  store ptr %7, ptr %8, align 8, !dbg !262
  ret void, !dbg !263
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN10FileHandle9read_byteEv(ptr noundef nonnull align 8 dereferenceable(8) %0) #2 comdat align 2 !dbg !264 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !265, metadata !DIExpression()), !dbg !266
  %4 = load ptr, ptr %3, align 8
  %5 = getelementptr inbounds %class.FileHandle, ptr %4, i32 0, i32 0, !dbg !267
  %6 = load ptr, ptr %5, align 8, !dbg !267
  %7 = icmp ne ptr %6, null, !dbg !267
  br i1 %7, label %8, label %12, !dbg !269

8:                                                ; preds = %1
  %9 = getelementptr inbounds %class.FileHandle, ptr %4, i32 0, i32 0, !dbg !270
  %10 = load ptr, ptr %9, align 8, !dbg !270
  %11 = call i32 @fgetc(ptr noundef %10), !dbg !271
  store i32 %11, ptr %2, align 4, !dbg !272
  br label %13, !dbg !272

12:                                               ; preds = %1
  store i32 -1, ptr %2, align 4, !dbg !273
  br label %13, !dbg !273

13:                                               ; preds = %12, %8
  %14 = load i32, ptr %2, align 4, !dbg !274
  ret i32 %14, !dbg !274
}

declare i32 @__gxx_personality_v0(...)

declare i32 @printf(ptr noundef, ...) #3

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN10FileHandleD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 personality ptr @__gxx_personality_v0 !dbg !275 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !276, metadata !DIExpression()), !dbg !277
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.FileHandle, ptr %3, i32 0, i32 0, !dbg !278
  %5 = load ptr, ptr %4, align 8, !dbg !278
  %6 = icmp ne ptr %5, null, !dbg !278
  br i1 %6, label %7, label %12, !dbg !281

7:                                                ; preds = %1
  %8 = getelementptr inbounds %class.FileHandle, ptr %3, i32 0, i32 0, !dbg !282
  %9 = load ptr, ptr %8, align 8, !dbg !282
  %10 = invoke i32 @fclose(ptr noundef %9)
          to label %11 unwind label %13, !dbg !283

11:                                               ; preds = %7
  br label %12, !dbg !283

12:                                               ; preds = %11, %1
  ret void, !dbg !284

13:                                               ; preds = %7
  %14 = landingpad { ptr, i32 }
          catch ptr null, !dbg !283
  %15 = extractvalue { ptr, i32 } %14, 0, !dbg !283
  call void @__clang_call_terminate(ptr %15) #7, !dbg !283
  unreachable, !dbg !283
}

declare noalias ptr @fopen(ptr noundef, ptr noundef) #3

declare i32 @fgetc(ptr noundef) #3

declare i32 @fclose(ptr noundef) #3

; Function Attrs: noinline noreturn nounwind uwtable
define linkonce_odr hidden void @__clang_call_terminate(ptr noundef %0) #5 comdat {
  %2 = call ptr @__cxa_begin_catch(ptr %0) #6
  call void @_ZSt9terminatev() #7
  unreachable
}

declare ptr @__cxa_begin_catch(ptr)

declare void @_ZSt9terminatev()

attributes #0 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { noinline noreturn nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #6 = { nounwind }
attributes #7 = { noreturn nounwind }

!llvm.dbg.cu = !{!18}
!llvm.module.flags = !{!233, !234, !235, !236, !237, !238, !239}
!llvm.ident = !{!240}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 25, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "cpp/raii_resource.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bd73f29c6731733dd5d5e0e82bff1343")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 112, elements: !6)
!4 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !5)
!5 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!6 = !{!7}
!7 = !DISubrange(count: 14)
!8 = !DIGlobalVariableExpression(var: !9, expr: !DIExpression())
!9 = distinct !DIGlobalVariable(scope: null, file: !2, line: 27, type: !10, isLocal: true, isDefinition: true)
!10 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 128, elements: !11)
!11 = !{!12}
!12 = !DISubrange(count: 16)
!13 = !DIGlobalVariableExpression(var: !14, expr: !DIExpression())
!14 = distinct !DIGlobalVariable(scope: null, file: !2, line: 13, type: !15, isLocal: true, isDefinition: true)
!15 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 16, elements: !16)
!16 = !{!17}
!17 = !DISubrange(count: 2)
!18 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !19, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !20, globals: !41, imports: !42, splitDebugInlining: false, nameTableKind: None)
!19 = !DIFile(filename: "/workspace/tests/programs/cpp/raii_resource.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bd73f29c6731733dd5d5e0e82bff1343")
!20 = !{!21}
!21 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "FileHandle", file: !2, line: 9, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !22, identifier: "_ZTS10FileHandle")
!22 = !{!23, !29, !34, !38}
!23 = !DIDerivedType(tag: DW_TAG_member, name: "fp", scope: !21, file: !2, line: 10, baseType: !24, size: 64)
!24 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !25, size: 64)
!25 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !26, line: 7, baseType: !27)
!26 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!27 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !28, line: 49, size: 1728, flags: DIFlagFwdDecl, identifier: "_ZTS8_IO_FILE")
!28 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!29 = !DISubprogram(name: "FileHandle", scope: !21, file: !2, line: 12, type: !30, scopeLine: 12, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!30 = !DISubroutineType(types: !31)
!31 = !{null, !32, !33}
!32 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !21, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!33 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!34 = !DISubprogram(name: "read_byte", linkageName: "_ZN10FileHandle9read_byteEv", scope: !21, file: !2, line: 15, type: !35, scopeLine: 15, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!35 = !DISubroutineType(types: !36)
!36 = !{!37, !32}
!37 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!38 = !DISubprogram(name: "~FileHandle", scope: !21, file: !2, line: 19, type: !39, scopeLine: 19, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!39 = !DISubroutineType(types: !40)
!40 = !{null, !32}
!41 = !{!0, !8, !13}
!42 = !{!43, !46, !52, !56, !60, !62, !64, !66, !68, !75, !81, !86, !90, !94, !98, !107, !111, !113, !118, !124, !128, !135, !137, !141, !145, !149, !151, !155, !159, !161, !165, !167, !169, !173, !177, !181, !185, !189, !193, !195, !203, !207, !211, !216, !218, !220, !224, !228, !229, !230, !231, !232}
!43 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !25, file: !45, line: 98)
!44 = !DINamespace(name: "std", scope: null)
!45 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdio", directory: "")
!46 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !47, file: !45, line: 99)
!47 = !DIDerivedType(tag: DW_TAG_typedef, name: "fpos_t", file: !48, line: 85, baseType: !49)
!48 = !DIFile(filename: "/usr/include/stdio.h", directory: "", checksumkind: CSK_MD5, checksum: "1e435c46987a169d9f9186f63a512303")
!49 = !DIDerivedType(tag: DW_TAG_typedef, name: "__fpos_t", file: !50, line: 14, baseType: !51)
!50 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/__fpos_t.h", directory: "", checksumkind: CSK_MD5, checksum: "32de8bdaf3551a6c0a9394f9af4389ce")
!51 = !DICompositeType(tag: DW_TAG_structure_type, name: "_G_fpos_t", file: !50, line: 10, size: 128, flags: DIFlagFwdDecl, identifier: "_ZTS9_G_fpos_t")
!52 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !53, file: !45, line: 101)
!53 = !DISubprogram(name: "clearerr", scope: !48, file: !48, line: 860, type: !54, flags: DIFlagPrototyped, spFlags: 0)
!54 = !DISubroutineType(types: !55)
!55 = !{null, !24}
!56 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !57, file: !45, line: 102)
!57 = !DISubprogram(name: "fclose", scope: !48, file: !48, line: 184, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!58 = !DISubroutineType(types: !59)
!59 = !{!37, !24}
!60 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !61, file: !45, line: 103)
!61 = !DISubprogram(name: "feof", scope: !48, file: !48, line: 862, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!62 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !63, file: !45, line: 104)
!63 = !DISubprogram(name: "ferror", scope: !48, file: !48, line: 864, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!64 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !65, file: !45, line: 105)
!65 = !DISubprogram(name: "fflush", scope: !48, file: !48, line: 236, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!66 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !67, file: !45, line: 106)
!67 = !DISubprogram(name: "fgetc", scope: !48, file: !48, line: 575, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!68 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !69, file: !45, line: 107)
!69 = !DISubprogram(name: "fgetpos", scope: !48, file: !48, line: 829, type: !70, flags: DIFlagPrototyped, spFlags: 0)
!70 = !DISubroutineType(types: !71)
!71 = !{!37, !72, !73}
!72 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !24)
!73 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !74)
!74 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !47, size: 64)
!75 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !76, file: !45, line: 108)
!76 = !DISubprogram(name: "fgets", scope: !48, file: !48, line: 654, type: !77, flags: DIFlagPrototyped, spFlags: 0)
!77 = !DISubroutineType(types: !78)
!78 = !{!79, !80, !37, !72}
!79 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!80 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !79)
!81 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !82, file: !45, line: 109)
!82 = !DISubprogram(name: "fopen", scope: !48, file: !48, line: 264, type: !83, flags: DIFlagPrototyped, spFlags: 0)
!83 = !DISubroutineType(types: !84)
!84 = !{!24, !85, !85}
!85 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !33)
!86 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !87, file: !45, line: 110)
!87 = !DISubprogram(name: "fprintf", scope: !48, file: !48, line: 357, type: !88, flags: DIFlagPrototyped, spFlags: 0)
!88 = !DISubroutineType(types: !89)
!89 = !{!37, !72, !85, null}
!90 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !91, file: !45, line: 111)
!91 = !DISubprogram(name: "fputc", scope: !48, file: !48, line: 611, type: !92, flags: DIFlagPrototyped, spFlags: 0)
!92 = !DISubroutineType(types: !93)
!93 = !{!37, !37, !24}
!94 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !95, file: !45, line: 112)
!95 = !DISubprogram(name: "fputs", scope: !48, file: !48, line: 717, type: !96, flags: DIFlagPrototyped, spFlags: 0)
!96 = !DISubroutineType(types: !97)
!97 = !{!37, !85, !72}
!98 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !99, file: !45, line: 113)
!99 = !DISubprogram(name: "fread", scope: !48, file: !48, line: 738, type: !100, flags: DIFlagPrototyped, spFlags: 0)
!100 = !DISubroutineType(types: !101)
!101 = !{!102, !105, !102, !102, !72}
!102 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !103, line: 18, baseType: !104)
!103 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!104 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!105 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !106)
!106 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!107 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !108, file: !45, line: 114)
!108 = !DISubprogram(name: "freopen", scope: !48, file: !48, line: 271, type: !109, flags: DIFlagPrototyped, spFlags: 0)
!109 = !DISubroutineType(types: !110)
!110 = !{!24, !85, !85, !72}
!111 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !112, file: !45, line: 115)
!112 = !DISubprogram(name: "fscanf", linkageName: "__isoc23_fscanf", scope: !48, file: !48, line: 442, type: !88, flags: DIFlagPrototyped, spFlags: 0)
!113 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !114, file: !45, line: 116)
!114 = !DISubprogram(name: "fseek", scope: !48, file: !48, line: 779, type: !115, flags: DIFlagPrototyped, spFlags: 0)
!115 = !DISubroutineType(types: !116)
!116 = !{!37, !24, !117, !37}
!117 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!118 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !119, file: !45, line: 117)
!119 = !DISubprogram(name: "fsetpos", scope: !48, file: !48, line: 835, type: !120, flags: DIFlagPrototyped, spFlags: 0)
!120 = !DISubroutineType(types: !121)
!121 = !{!37, !24, !122}
!122 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !123, size: 64)
!123 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !47)
!124 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !125, file: !45, line: 118)
!125 = !DISubprogram(name: "ftell", scope: !48, file: !48, line: 785, type: !126, flags: DIFlagPrototyped, spFlags: 0)
!126 = !DISubroutineType(types: !127)
!127 = !{!117, !24}
!128 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !129, file: !45, line: 119)
!129 = !DISubprogram(name: "fwrite", scope: !48, file: !48, line: 745, type: !130, flags: DIFlagPrototyped, spFlags: 0)
!130 = !DISubroutineType(types: !131)
!131 = !{!102, !132, !102, !102, !72}
!132 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !133)
!133 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !134, size: 64)
!134 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!135 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !136, file: !45, line: 120)
!136 = !DISubprogram(name: "getc", scope: !48, file: !48, line: 576, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!137 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !138, file: !45, line: 121)
!138 = !DISubprogram(name: "getchar", scope: !48, file: !48, line: 582, type: !139, flags: DIFlagPrototyped, spFlags: 0)
!139 = !DISubroutineType(types: !140)
!140 = !{!37}
!141 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !142, file: !45, line: 126)
!142 = !DISubprogram(name: "perror", scope: !48, file: !48, line: 878, type: !143, flags: DIFlagPrototyped, spFlags: 0)
!143 = !DISubroutineType(types: !144)
!144 = !{null, !33}
!145 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !146, file: !45, line: 127)
!146 = !DISubprogram(name: "printf", scope: !48, file: !48, line: 363, type: !147, flags: DIFlagPrototyped, spFlags: 0)
!147 = !DISubroutineType(types: !148)
!148 = !{!37, !85, null}
!149 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !150, file: !45, line: 128)
!150 = !DISubprogram(name: "putc", scope: !48, file: !48, line: 612, type: !92, flags: DIFlagPrototyped, spFlags: 0)
!151 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !152, file: !45, line: 129)
!152 = !DISubprogram(name: "putchar", scope: !48, file: !48, line: 618, type: !153, flags: DIFlagPrototyped, spFlags: 0)
!153 = !DISubroutineType(types: !154)
!154 = !{!37, !37}
!155 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !156, file: !45, line: 130)
!156 = !DISubprogram(name: "puts", scope: !48, file: !48, line: 724, type: !157, flags: DIFlagPrototyped, spFlags: 0)
!157 = !DISubroutineType(types: !158)
!158 = !{!37, !33}
!159 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !160, file: !45, line: 131)
!160 = !DISubprogram(name: "remove", scope: !48, file: !48, line: 158, type: !157, flags: DIFlagPrototyped, spFlags: 0)
!161 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !162, file: !45, line: 132)
!162 = !DISubprogram(name: "rename", scope: !48, file: !48, line: 160, type: !163, flags: DIFlagPrototyped, spFlags: 0)
!163 = !DISubroutineType(types: !164)
!164 = !{!37, !33, !33}
!165 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !166, file: !45, line: 133)
!166 = !DISubprogram(name: "rewind", scope: !48, file: !48, line: 790, type: !54, flags: DIFlagPrototyped, spFlags: 0)
!167 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !168, file: !45, line: 134)
!168 = !DISubprogram(name: "scanf", linkageName: "__isoc23_scanf", scope: !48, file: !48, line: 445, type: !147, flags: DIFlagPrototyped, spFlags: 0)
!169 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !170, file: !45, line: 135)
!170 = !DISubprogram(name: "setbuf", scope: !48, file: !48, line: 334, type: !171, flags: DIFlagPrototyped, spFlags: 0)
!171 = !DISubroutineType(types: !172)
!172 = !{null, !72, !80}
!173 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !174, file: !45, line: 136)
!174 = !DISubprogram(name: "setvbuf", scope: !48, file: !48, line: 339, type: !175, flags: DIFlagPrototyped, spFlags: 0)
!175 = !DISubroutineType(types: !176)
!176 = !{!37, !72, !80, !37, !102}
!177 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !178, file: !45, line: 137)
!178 = !DISubprogram(name: "sprintf", scope: !48, file: !48, line: 365, type: !179, flags: DIFlagPrototyped, spFlags: 0)
!179 = !DISubroutineType(types: !180)
!180 = !{!37, !80, !85, null}
!181 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !182, file: !45, line: 138)
!182 = !DISubprogram(name: "sscanf", linkageName: "__isoc23_sscanf", scope: !48, file: !48, line: 447, type: !183, flags: DIFlagPrototyped, spFlags: 0)
!183 = !DISubroutineType(types: !184)
!184 = !{!37, !85, !85, null}
!185 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !186, file: !45, line: 139)
!186 = !DISubprogram(name: "tmpfile", scope: !48, file: !48, line: 194, type: !187, flags: DIFlagPrototyped, spFlags: 0)
!187 = !DISubroutineType(types: !188)
!188 = !{!24}
!189 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !190, file: !45, line: 141)
!190 = !DISubprogram(name: "tmpnam", scope: !48, file: !48, line: 211, type: !191, flags: DIFlagPrototyped, spFlags: 0)
!191 = !DISubroutineType(types: !192)
!192 = !{!79, !79}
!193 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !194, file: !45, line: 143)
!194 = !DISubprogram(name: "ungetc", scope: !48, file: !48, line: 731, type: !92, flags: DIFlagPrototyped, spFlags: 0)
!195 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !196, file: !45, line: 144)
!196 = !DISubprogram(name: "vfprintf", scope: !48, file: !48, line: 372, type: !197, flags: DIFlagPrototyped, spFlags: 0)
!197 = !DISubroutineType(types: !198)
!198 = !{!37, !72, !85, !199}
!199 = !DIDerivedType(tag: DW_TAG_typedef, name: "__gnuc_va_list", file: !200, line: 12, baseType: !201)
!200 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stdarg___gnuc_va_list.h", directory: "", checksumkind: CSK_MD5, checksum: "edb3f2eab991638e4dc94f6e55e3530f")
!201 = !DIDerivedType(tag: DW_TAG_typedef, name: "__builtin_va_list", file: !2, baseType: !202)
!202 = !DICompositeType(tag: DW_TAG_structure_type, name: "__va_list", scope: !44, file: !2, size: 256, flags: DIFlagFwdDecl, identifier: "_ZTSSt9__va_list")
!203 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !204, file: !45, line: 145)
!204 = !DISubprogram(name: "vprintf", scope: !48, file: !48, line: 378, type: !205, flags: DIFlagPrototyped, spFlags: 0)
!205 = !DISubroutineType(types: !206)
!206 = !{!37, !85, !199}
!207 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !208, file: !45, line: 146)
!208 = !DISubprogram(name: "vsprintf", scope: !48, file: !48, line: 380, type: !209, flags: DIFlagPrototyped, spFlags: 0)
!209 = !DISubroutineType(types: !210)
!210 = !{!37, !80, !85, !199}
!211 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !212, entity: !213, file: !45, line: 175)
!212 = !DINamespace(name: "__gnu_cxx", scope: null)
!213 = !DISubprogram(name: "snprintf", scope: !48, file: !48, line: 385, type: !214, flags: DIFlagPrototyped, spFlags: 0)
!214 = !DISubroutineType(types: !215)
!215 = !{!37, !80, !102, !85, null}
!216 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !212, entity: !217, file: !45, line: 176)
!217 = !DISubprogram(name: "vfscanf", linkageName: "__isoc23_vfscanf", scope: !48, file: !48, line: 511, type: !197, flags: DIFlagPrototyped, spFlags: 0)
!218 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !212, entity: !219, file: !45, line: 177)
!219 = !DISubprogram(name: "vscanf", linkageName: "__isoc23_vscanf", scope: !48, file: !48, line: 516, type: !205, flags: DIFlagPrototyped, spFlags: 0)
!220 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !212, entity: !221, file: !45, line: 178)
!221 = !DISubprogram(name: "vsnprintf", scope: !48, file: !48, line: 389, type: !222, flags: DIFlagPrototyped, spFlags: 0)
!222 = !DISubroutineType(types: !223)
!223 = !{!37, !80, !102, !85, !199}
!224 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !212, entity: !225, file: !45, line: 179)
!225 = !DISubprogram(name: "vsscanf", linkageName: "__isoc23_vsscanf", scope: !48, file: !48, line: 519, type: !226, flags: DIFlagPrototyped, spFlags: 0)
!226 = !DISubroutineType(types: !227)
!227 = !{!37, !85, !85, !199}
!228 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !213, file: !45, line: 185)
!229 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !217, file: !45, line: 186)
!230 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !219, file: !45, line: 187)
!231 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !221, file: !45, line: 188)
!232 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !44, entity: !225, file: !45, line: 189)
!233 = !{i32 7, !"Dwarf Version", i32 5}
!234 = !{i32 2, !"Debug Info Version", i32 3}
!235 = !{i32 1, !"wchar_size", i32 4}
!236 = !{i32 8, !"PIC Level", i32 2}
!237 = !{i32 7, !"PIE Level", i32 2}
!238 = !{i32 7, !"uwtable", i32 2}
!239 = !{i32 7, !"frame-pointer", i32 1}
!240 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!241 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 24, type: !139, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !18, retainedNodes: !242)
!242 = !{}
!243 = !DILocalVariable(name: "fh", scope: !241, file: !2, line: 25, type: !21)
!244 = !DILocation(line: 25, column: 16, scope: !241)
!245 = !DILocalVariable(name: "ch", scope: !241, file: !2, line: 26, type: !37)
!246 = !DILocation(line: 26, column: 9, scope: !241)
!247 = !DILocation(line: 26, column: 17, scope: !241)
!248 = !DILocation(line: 27, column: 32, scope: !241)
!249 = !DILocation(line: 27, column: 5, scope: !241)
!250 = !DILocation(line: 28, column: 5, scope: !241)
!251 = !DILocation(line: 30, column: 1, scope: !241)
!252 = distinct !DISubprogram(name: "FileHandle", linkageName: "_ZN10FileHandleC2EPKc", scope: !21, file: !2, line: 12, type: !30, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !18, declaration: !29, retainedNodes: !242)
!253 = !DILocalVariable(name: "this", arg: 1, scope: !252, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!254 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !21, size: 64)
!255 = !DILocation(line: 0, scope: !252)
!256 = !DILocalVariable(name: "path", arg: 2, scope: !252, file: !2, line: 12, type: !33)
!257 = !DILocation(line: 12, column: 28, scope: !252)
!258 = !DILocation(line: 13, column: 20, scope: !259)
!259 = distinct !DILexicalBlock(scope: !252, file: !2, line: 12, column: 34)
!260 = !DILocation(line: 13, column: 14, scope: !259)
!261 = !DILocation(line: 13, column: 9, scope: !259)
!262 = !DILocation(line: 13, column: 12, scope: !259)
!263 = !DILocation(line: 14, column: 5, scope: !252)
!264 = distinct !DISubprogram(name: "read_byte", linkageName: "_ZN10FileHandle9read_byteEv", scope: !21, file: !2, line: 15, type: !35, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !18, declaration: !34, retainedNodes: !242)
!265 = !DILocalVariable(name: "this", arg: 1, scope: !264, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!266 = !DILocation(line: 0, scope: !264)
!267 = !DILocation(line: 16, column: 13, scope: !268)
!268 = distinct !DILexicalBlock(scope: !264, file: !2, line: 16, column: 13)
!269 = !DILocation(line: 16, column: 13, scope: !264)
!270 = !DILocation(line: 16, column: 30, scope: !268)
!271 = !DILocation(line: 16, column: 24, scope: !268)
!272 = !DILocation(line: 16, column: 17, scope: !268)
!273 = !DILocation(line: 17, column: 9, scope: !264)
!274 = !DILocation(line: 18, column: 5, scope: !264)
!275 = distinct !DISubprogram(name: "~FileHandle", linkageName: "_ZN10FileHandleD2Ev", scope: !21, file: !2, line: 19, type: !39, scopeLine: 19, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !18, declaration: !38, retainedNodes: !242)
!276 = !DILocalVariable(name: "this", arg: 1, scope: !275, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!277 = !DILocation(line: 0, scope: !275)
!278 = !DILocation(line: 20, column: 13, scope: !279)
!279 = distinct !DILexicalBlock(scope: !280, file: !2, line: 20, column: 13)
!280 = distinct !DILexicalBlock(scope: !275, file: !2, line: 19, column: 19)
!281 = !DILocation(line: 20, column: 13, scope: !280)
!282 = !DILocation(line: 20, column: 24, scope: !279)
!283 = !DILocation(line: 20, column: 17, scope: !279)
!284 = !DILocation(line: 21, column: 5, scope: !275)
