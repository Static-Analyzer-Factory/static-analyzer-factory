; ModuleID = '/workspace/tests/programs/cpp/vtable_dispatch.cpp'
source_filename = "/workspace/tests/programs/cpp/vtable_dispatch.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

$_ZN3DogC2Ev = comdat any

$_ZN6AnimalC2Ev = comdat any

$_ZN3Dog5speakEv = comdat any

$_ZN3DogD2Ev = comdat any

$_ZN3DogD0Ev = comdat any

$_ZN6Animal5speakEv = comdat any

$_ZN6AnimalD2Ev = comdat any

$_ZN6AnimalD0Ev = comdat any

$_ZTV3Dog = comdat any

$_ZTS3Dog = comdat any

$_ZTS6Animal = comdat any

$_ZTI6Animal = comdat any

$_ZTI3Dog = comdat any

$_ZTV6Animal = comdat any

@_ZTV3Dog = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI3Dog, ptr @_ZN3Dog5speakEv, ptr @_ZN3DogD2Ev, ptr @_ZN3DogD0Ev] }, comdat, align 8
@_ZTVN10__cxxabiv120__si_class_type_infoE = external global [0 x ptr]
@_ZTS3Dog = linkonce_odr dso_local constant [5 x i8] c"3Dog\00", comdat, align 1
@_ZTVN10__cxxabiv117__class_type_infoE = external global [0 x ptr]
@_ZTS6Animal = linkonce_odr dso_local constant [8 x i8] c"6Animal\00", comdat, align 1
@_ZTI6Animal = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS6Animal }, comdat, align 8
@_ZTI3Dog = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS3Dog, ptr @_ZTI6Animal }, comdat, align 8
@_ZTV6Animal = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI6Animal, ptr @_ZN6Animal5speakEv, ptr @_ZN6AnimalD2Ev, ptr @_ZN6AnimalD0Ev] }, comdat, align 8
@.str = private unnamed_addr constant [5 x i8] c"...\0A\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [6 x i8] c"woof\0A\00", align 1, !dbg !8

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #0 !dbg !222 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !224, metadata !DIExpression()), !dbg !236
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #8, !dbg !237, !heapallocsite !238
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !245
  call void @_ZN3DogC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #9, !dbg !245
  store ptr %3, ptr %2, align 8, !dbg !236
  %4 = load ptr, ptr %2, align 8, !dbg !246
  %5 = load ptr, ptr %4, align 8, !dbg !247
  %6 = getelementptr inbounds ptr, ptr %5, i64 0, !dbg !247
  %7 = load ptr, ptr %6, align 8, !dbg !247
  call void %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !247
  %8 = load ptr, ptr %2, align 8, !dbg !248
  %9 = icmp eq ptr %8, null, !dbg !249
  br i1 %9, label %14, label %10, !dbg !249

10:                                               ; preds = %0
  %11 = load ptr, ptr %8, align 8, !dbg !249
  %12 = getelementptr inbounds ptr, ptr %11, i64 2, !dbg !249
  %13 = load ptr, ptr %12, align 8, !dbg !249
  call void %13(ptr noundef nonnull align 8 dereferenceable(8) %8) #9, !dbg !249
  br label %14, !dbg !249

14:                                               ; preds = %10, %0
  ret i32 0, !dbg !250
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nobuiltin allocsize(0)
declare noundef nonnull ptr @_Znwm(i64 noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #3

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !251 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !253, metadata !DIExpression()), !dbg !255
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #9, !dbg !256
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV3Dog, i32 0, i32 0, i32 2), ptr %3, align 8, !dbg !256
  ret void, !dbg !256
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !257 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !259, metadata !DIExpression()), !dbg !260
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV6Animal, i32 0, i32 0, i32 2), ptr %3, align 8, !dbg !261
  ret void, !dbg !261
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN3Dog5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #5 comdat align 2 !dbg !262 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !263, metadata !DIExpression()), !dbg !264
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 (ptr, ...) @printf(ptr noundef @.str.1), !dbg !265
  ret void, !dbg !266
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !267 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !269, metadata !DIExpression()), !dbg !270
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #9, !dbg !271
  ret void, !dbg !273
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !274 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !275, metadata !DIExpression()), !dbg !276
  %3 = load ptr, ptr %2, align 8
  call void @_ZN3DogD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #9, !dbg !277
  call void @_ZdlPv(ptr noundef %3) #10, !dbg !277
  ret void, !dbg !277
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN6Animal5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #5 comdat align 2 !dbg !278 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !279, metadata !DIExpression()), !dbg !280
  %3 = load ptr, ptr %2, align 8
  %4 = call i32 (ptr, ...) @printf(ptr noundef @.str), !dbg !281
  ret void, !dbg !282
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !283 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !284, metadata !DIExpression()), !dbg !285
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !286
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !287 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !288, metadata !DIExpression()), !dbg !289
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #9, !dbg !290
  call void @_ZdlPv(ptr noundef %3) #10, !dbg !290
  ret void, !dbg !290
}

declare i32 @printf(ptr noundef, ...) #6

; Function Attrs: nobuiltin nounwind
declare void @_ZdlPv(ptr noundef) #7

attributes #0 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nobuiltin allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #4 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #6 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #7 = { nobuiltin nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #8 = { builtin allocsize(0) }
attributes #9 = { nounwind }
attributes #10 = { builtin nounwind }

!llvm.dbg.cu = !{!13}
!llvm.module.flags = !{!214, !215, !216, !217, !218, !219, !220}
!llvm.ident = !{!221}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 11, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "cpp/vtable_dispatch.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4bab1650216ec88bcff693fea673a0b4")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 40, elements: !6)
!4 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !5)
!5 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!6 = !{!7}
!7 = !DISubrange(count: 5)
!8 = !DIGlobalVariableExpression(var: !9, expr: !DIExpression())
!9 = distinct !DIGlobalVariable(scope: null, file: !2, line: 19, type: !10, isLocal: true, isDefinition: true)
!10 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 48, elements: !11)
!11 = !{!12}
!12 = !DISubrange(count: 6)
!13 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !14, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !15, imports: !16, splitDebugInlining: false, nameTableKind: None)
!14 = !DIFile(filename: "/workspace/tests/programs/cpp/vtable_dispatch.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4bab1650216ec88bcff693fea673a0b4")
!15 = !{!0, !8}
!16 = !{!17, !24, !30, !35, !40, !42, !44, !46, !48, !55, !61, !67, !71, !75, !79, !88, !92, !94, !99, !105, !109, !116, !118, !122, !126, !130, !132, !136, !140, !142, !146, !148, !150, !154, !158, !162, !166, !170, !174, !176, !184, !188, !192, !197, !199, !201, !205, !209, !210, !211, !212, !213}
!17 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !19, file: !23, line: 98)
!18 = !DINamespace(name: "std", scope: null)
!19 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !20, line: 7, baseType: !21)
!20 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!21 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !22, line: 49, size: 1728, flags: DIFlagFwdDecl, identifier: "_ZTS8_IO_FILE")
!22 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!23 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdio", directory: "")
!24 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !25, file: !23, line: 99)
!25 = !DIDerivedType(tag: DW_TAG_typedef, name: "fpos_t", file: !26, line: 85, baseType: !27)
!26 = !DIFile(filename: "/usr/include/stdio.h", directory: "", checksumkind: CSK_MD5, checksum: "1e435c46987a169d9f9186f63a512303")
!27 = !DIDerivedType(tag: DW_TAG_typedef, name: "__fpos_t", file: !28, line: 14, baseType: !29)
!28 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/__fpos_t.h", directory: "", checksumkind: CSK_MD5, checksum: "32de8bdaf3551a6c0a9394f9af4389ce")
!29 = !DICompositeType(tag: DW_TAG_structure_type, name: "_G_fpos_t", file: !28, line: 10, size: 128, flags: DIFlagFwdDecl, identifier: "_ZTS9_G_fpos_t")
!30 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !31, file: !23, line: 101)
!31 = !DISubprogram(name: "clearerr", scope: !26, file: !26, line: 860, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!32 = !DISubroutineType(types: !33)
!33 = !{null, !34}
!34 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !19, size: 64)
!35 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !36, file: !23, line: 102)
!36 = !DISubprogram(name: "fclose", scope: !26, file: !26, line: 184, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!37 = !DISubroutineType(types: !38)
!38 = !{!39, !34}
!39 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!40 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !41, file: !23, line: 103)
!41 = !DISubprogram(name: "feof", scope: !26, file: !26, line: 862, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!42 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !43, file: !23, line: 104)
!43 = !DISubprogram(name: "ferror", scope: !26, file: !26, line: 864, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!44 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !45, file: !23, line: 105)
!45 = !DISubprogram(name: "fflush", scope: !26, file: !26, line: 236, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!46 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !47, file: !23, line: 106)
!47 = !DISubprogram(name: "fgetc", scope: !26, file: !26, line: 575, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!48 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !49, file: !23, line: 107)
!49 = !DISubprogram(name: "fgetpos", scope: !26, file: !26, line: 829, type: !50, flags: DIFlagPrototyped, spFlags: 0)
!50 = !DISubroutineType(types: !51)
!51 = !{!39, !52, !53}
!52 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !34)
!53 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !54)
!54 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !25, size: 64)
!55 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !56, file: !23, line: 108)
!56 = !DISubprogram(name: "fgets", scope: !26, file: !26, line: 654, type: !57, flags: DIFlagPrototyped, spFlags: 0)
!57 = !DISubroutineType(types: !58)
!58 = !{!59, !60, !39, !52}
!59 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!60 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !59)
!61 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !62, file: !23, line: 109)
!62 = !DISubprogram(name: "fopen", scope: !26, file: !26, line: 264, type: !63, flags: DIFlagPrototyped, spFlags: 0)
!63 = !DISubroutineType(types: !64)
!64 = !{!34, !65, !65}
!65 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !66)
!66 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!67 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !68, file: !23, line: 110)
!68 = !DISubprogram(name: "fprintf", scope: !26, file: !26, line: 357, type: !69, flags: DIFlagPrototyped, spFlags: 0)
!69 = !DISubroutineType(types: !70)
!70 = !{!39, !52, !65, null}
!71 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !72, file: !23, line: 111)
!72 = !DISubprogram(name: "fputc", scope: !26, file: !26, line: 611, type: !73, flags: DIFlagPrototyped, spFlags: 0)
!73 = !DISubroutineType(types: !74)
!74 = !{!39, !39, !34}
!75 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !76, file: !23, line: 112)
!76 = !DISubprogram(name: "fputs", scope: !26, file: !26, line: 717, type: !77, flags: DIFlagPrototyped, spFlags: 0)
!77 = !DISubroutineType(types: !78)
!78 = !{!39, !65, !52}
!79 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !80, file: !23, line: 113)
!80 = !DISubprogram(name: "fread", scope: !26, file: !26, line: 738, type: !81, flags: DIFlagPrototyped, spFlags: 0)
!81 = !DISubroutineType(types: !82)
!82 = !{!83, !86, !83, !83, !52}
!83 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !84, line: 18, baseType: !85)
!84 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!85 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!86 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !87)
!87 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!88 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !89, file: !23, line: 114)
!89 = !DISubprogram(name: "freopen", scope: !26, file: !26, line: 271, type: !90, flags: DIFlagPrototyped, spFlags: 0)
!90 = !DISubroutineType(types: !91)
!91 = !{!34, !65, !65, !52}
!92 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !93, file: !23, line: 115)
!93 = !DISubprogram(name: "fscanf", linkageName: "__isoc23_fscanf", scope: !26, file: !26, line: 442, type: !69, flags: DIFlagPrototyped, spFlags: 0)
!94 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !95, file: !23, line: 116)
!95 = !DISubprogram(name: "fseek", scope: !26, file: !26, line: 779, type: !96, flags: DIFlagPrototyped, spFlags: 0)
!96 = !DISubroutineType(types: !97)
!97 = !{!39, !34, !98, !39}
!98 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!99 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !100, file: !23, line: 117)
!100 = !DISubprogram(name: "fsetpos", scope: !26, file: !26, line: 835, type: !101, flags: DIFlagPrototyped, spFlags: 0)
!101 = !DISubroutineType(types: !102)
!102 = !{!39, !34, !103}
!103 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !104, size: 64)
!104 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !25)
!105 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !106, file: !23, line: 118)
!106 = !DISubprogram(name: "ftell", scope: !26, file: !26, line: 785, type: !107, flags: DIFlagPrototyped, spFlags: 0)
!107 = !DISubroutineType(types: !108)
!108 = !{!98, !34}
!109 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !110, file: !23, line: 119)
!110 = !DISubprogram(name: "fwrite", scope: !26, file: !26, line: 745, type: !111, flags: DIFlagPrototyped, spFlags: 0)
!111 = !DISubroutineType(types: !112)
!112 = !{!83, !113, !83, !83, !52}
!113 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !114)
!114 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !115, size: 64)
!115 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!116 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !117, file: !23, line: 120)
!117 = !DISubprogram(name: "getc", scope: !26, file: !26, line: 576, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!118 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !119, file: !23, line: 121)
!119 = !DISubprogram(name: "getchar", scope: !26, file: !26, line: 582, type: !120, flags: DIFlagPrototyped, spFlags: 0)
!120 = !DISubroutineType(types: !121)
!121 = !{!39}
!122 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !123, file: !23, line: 126)
!123 = !DISubprogram(name: "perror", scope: !26, file: !26, line: 878, type: !124, flags: DIFlagPrototyped, spFlags: 0)
!124 = !DISubroutineType(types: !125)
!125 = !{null, !66}
!126 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !127, file: !23, line: 127)
!127 = !DISubprogram(name: "printf", scope: !26, file: !26, line: 363, type: !128, flags: DIFlagPrototyped, spFlags: 0)
!128 = !DISubroutineType(types: !129)
!129 = !{!39, !65, null}
!130 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !131, file: !23, line: 128)
!131 = !DISubprogram(name: "putc", scope: !26, file: !26, line: 612, type: !73, flags: DIFlagPrototyped, spFlags: 0)
!132 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !133, file: !23, line: 129)
!133 = !DISubprogram(name: "putchar", scope: !26, file: !26, line: 618, type: !134, flags: DIFlagPrototyped, spFlags: 0)
!134 = !DISubroutineType(types: !135)
!135 = !{!39, !39}
!136 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !137, file: !23, line: 130)
!137 = !DISubprogram(name: "puts", scope: !26, file: !26, line: 724, type: !138, flags: DIFlagPrototyped, spFlags: 0)
!138 = !DISubroutineType(types: !139)
!139 = !{!39, !66}
!140 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !141, file: !23, line: 131)
!141 = !DISubprogram(name: "remove", scope: !26, file: !26, line: 158, type: !138, flags: DIFlagPrototyped, spFlags: 0)
!142 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !143, file: !23, line: 132)
!143 = !DISubprogram(name: "rename", scope: !26, file: !26, line: 160, type: !144, flags: DIFlagPrototyped, spFlags: 0)
!144 = !DISubroutineType(types: !145)
!145 = !{!39, !66, !66}
!146 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !147, file: !23, line: 133)
!147 = !DISubprogram(name: "rewind", scope: !26, file: !26, line: 790, type: !32, flags: DIFlagPrototyped, spFlags: 0)
!148 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !149, file: !23, line: 134)
!149 = !DISubprogram(name: "scanf", linkageName: "__isoc23_scanf", scope: !26, file: !26, line: 445, type: !128, flags: DIFlagPrototyped, spFlags: 0)
!150 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !151, file: !23, line: 135)
!151 = !DISubprogram(name: "setbuf", scope: !26, file: !26, line: 334, type: !152, flags: DIFlagPrototyped, spFlags: 0)
!152 = !DISubroutineType(types: !153)
!153 = !{null, !52, !60}
!154 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !155, file: !23, line: 136)
!155 = !DISubprogram(name: "setvbuf", scope: !26, file: !26, line: 339, type: !156, flags: DIFlagPrototyped, spFlags: 0)
!156 = !DISubroutineType(types: !157)
!157 = !{!39, !52, !60, !39, !83}
!158 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !159, file: !23, line: 137)
!159 = !DISubprogram(name: "sprintf", scope: !26, file: !26, line: 365, type: !160, flags: DIFlagPrototyped, spFlags: 0)
!160 = !DISubroutineType(types: !161)
!161 = !{!39, !60, !65, null}
!162 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !163, file: !23, line: 138)
!163 = !DISubprogram(name: "sscanf", linkageName: "__isoc23_sscanf", scope: !26, file: !26, line: 447, type: !164, flags: DIFlagPrototyped, spFlags: 0)
!164 = !DISubroutineType(types: !165)
!165 = !{!39, !65, !65, null}
!166 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !167, file: !23, line: 139)
!167 = !DISubprogram(name: "tmpfile", scope: !26, file: !26, line: 194, type: !168, flags: DIFlagPrototyped, spFlags: 0)
!168 = !DISubroutineType(types: !169)
!169 = !{!34}
!170 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !171, file: !23, line: 141)
!171 = !DISubprogram(name: "tmpnam", scope: !26, file: !26, line: 211, type: !172, flags: DIFlagPrototyped, spFlags: 0)
!172 = !DISubroutineType(types: !173)
!173 = !{!59, !59}
!174 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !175, file: !23, line: 143)
!175 = !DISubprogram(name: "ungetc", scope: !26, file: !26, line: 731, type: !73, flags: DIFlagPrototyped, spFlags: 0)
!176 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !177, file: !23, line: 144)
!177 = !DISubprogram(name: "vfprintf", scope: !26, file: !26, line: 372, type: !178, flags: DIFlagPrototyped, spFlags: 0)
!178 = !DISubroutineType(types: !179)
!179 = !{!39, !52, !65, !180}
!180 = !DIDerivedType(tag: DW_TAG_typedef, name: "__gnuc_va_list", file: !181, line: 12, baseType: !182)
!181 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stdarg___gnuc_va_list.h", directory: "", checksumkind: CSK_MD5, checksum: "edb3f2eab991638e4dc94f6e55e3530f")
!182 = !DIDerivedType(tag: DW_TAG_typedef, name: "__builtin_va_list", file: !2, baseType: !183)
!183 = !DICompositeType(tag: DW_TAG_structure_type, name: "__va_list", scope: !18, file: !2, size: 256, flags: DIFlagFwdDecl, identifier: "_ZTSSt9__va_list")
!184 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !185, file: !23, line: 145)
!185 = !DISubprogram(name: "vprintf", scope: !26, file: !26, line: 378, type: !186, flags: DIFlagPrototyped, spFlags: 0)
!186 = !DISubroutineType(types: !187)
!187 = !{!39, !65, !180}
!188 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !189, file: !23, line: 146)
!189 = !DISubprogram(name: "vsprintf", scope: !26, file: !26, line: 380, type: !190, flags: DIFlagPrototyped, spFlags: 0)
!190 = !DISubroutineType(types: !191)
!191 = !{!39, !60, !65, !180}
!192 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !193, entity: !194, file: !23, line: 175)
!193 = !DINamespace(name: "__gnu_cxx", scope: null)
!194 = !DISubprogram(name: "snprintf", scope: !26, file: !26, line: 385, type: !195, flags: DIFlagPrototyped, spFlags: 0)
!195 = !DISubroutineType(types: !196)
!196 = !{!39, !60, !83, !65, null}
!197 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !193, entity: !198, file: !23, line: 176)
!198 = !DISubprogram(name: "vfscanf", linkageName: "__isoc23_vfscanf", scope: !26, file: !26, line: 511, type: !178, flags: DIFlagPrototyped, spFlags: 0)
!199 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !193, entity: !200, file: !23, line: 177)
!200 = !DISubprogram(name: "vscanf", linkageName: "__isoc23_vscanf", scope: !26, file: !26, line: 516, type: !186, flags: DIFlagPrototyped, spFlags: 0)
!201 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !193, entity: !202, file: !23, line: 178)
!202 = !DISubprogram(name: "vsnprintf", scope: !26, file: !26, line: 389, type: !203, flags: DIFlagPrototyped, spFlags: 0)
!203 = !DISubroutineType(types: !204)
!204 = !{!39, !60, !83, !65, !180}
!205 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !193, entity: !206, file: !23, line: 179)
!206 = !DISubprogram(name: "vsscanf", linkageName: "__isoc23_vsscanf", scope: !26, file: !26, line: 519, type: !207, flags: DIFlagPrototyped, spFlags: 0)
!207 = !DISubroutineType(types: !208)
!208 = !{!39, !65, !65, !180}
!209 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !194, file: !23, line: 185)
!210 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !198, file: !23, line: 186)
!211 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !200, file: !23, line: 187)
!212 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !202, file: !23, line: 188)
!213 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !18, entity: !206, file: !23, line: 189)
!214 = !{i32 7, !"Dwarf Version", i32 5}
!215 = !{i32 2, !"Debug Info Version", i32 3}
!216 = !{i32 1, !"wchar_size", i32 4}
!217 = !{i32 8, !"PIC Level", i32 2}
!218 = !{i32 7, !"PIE Level", i32 2}
!219 = !{i32 7, !"uwtable", i32 2}
!220 = !{i32 7, !"frame-pointer", i32 1}
!221 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!222 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 23, type: !120, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, retainedNodes: !223)
!223 = !{}
!224 = !DILocalVariable(name: "a", scope: !222, file: !2, line: 24, type: !225)
!225 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !226, size: 64)
!226 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Animal", file: !2, line: 8, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !227, vtableHolder: !226, identifier: "_ZTS6Animal")
!227 = !{!228, !231, !235}
!228 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Animal", scope: !2, file: !2, baseType: !229, size: 64, flags: DIFlagArtificial)
!229 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !230, size: 64)
!230 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "__vtbl_ptr_type", baseType: !120, size: 64)
!231 = !DISubprogram(name: "speak", linkageName: "_ZN6Animal5speakEv", scope: !226, file: !2, line: 10, type: !232, scopeLine: 10, containingType: !226, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!232 = !DISubroutineType(types: !233)
!233 = !{null, !234}
!234 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !226, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!235 = !DISubprogram(name: "~Animal", scope: !226, file: !2, line: 13, type: !232, scopeLine: 13, containingType: !226, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!236 = !DILocation(line: 24, column: 13, scope: !222)
!237 = !DILocation(line: 24, column: 17, scope: !222)
!238 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Dog", file: !2, line: 16, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !239, vtableHolder: !226, identifier: "_ZTS3Dog")
!239 = !{!240, !241}
!240 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !238, baseType: !226, flags: DIFlagPublic, extraData: i32 0)
!241 = !DISubprogram(name: "speak", linkageName: "_ZN3Dog5speakEv", scope: !238, file: !2, line: 18, type: !242, scopeLine: 18, containingType: !238, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!242 = !DISubroutineType(types: !243)
!243 = !{null, !244}
!244 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !238, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!245 = !DILocation(line: 24, column: 21, scope: !222)
!246 = !DILocation(line: 25, column: 5, scope: !222)
!247 = !DILocation(line: 25, column: 8, scope: !222)
!248 = !DILocation(line: 26, column: 12, scope: !222)
!249 = !DILocation(line: 26, column: 5, scope: !222)
!250 = !DILocation(line: 27, column: 5, scope: !222)
!251 = distinct !DISubprogram(name: "Dog", linkageName: "_ZN3DogC2Ev", scope: !238, file: !2, line: 16, type: !242, scopeLine: 16, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !252, retainedNodes: !223)
!252 = !DISubprogram(name: "Dog", scope: !238, type: !242, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!253 = !DILocalVariable(name: "this", arg: 1, scope: !251, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!254 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !238, size: 64)
!255 = !DILocation(line: 0, scope: !251)
!256 = !DILocation(line: 16, column: 7, scope: !251)
!257 = distinct !DISubprogram(name: "Animal", linkageName: "_ZN6AnimalC2Ev", scope: !226, file: !2, line: 8, type: !232, scopeLine: 8, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !258, retainedNodes: !223)
!258 = !DISubprogram(name: "Animal", scope: !226, type: !232, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!259 = !DILocalVariable(name: "this", arg: 1, scope: !257, type: !225, flags: DIFlagArtificial | DIFlagObjectPointer)
!260 = !DILocation(line: 0, scope: !257)
!261 = !DILocation(line: 8, column: 7, scope: !257)
!262 = distinct !DISubprogram(name: "speak", linkageName: "_ZN3Dog5speakEv", scope: !238, file: !2, line: 18, type: !242, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !241, retainedNodes: !223)
!263 = !DILocalVariable(name: "this", arg: 1, scope: !262, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!264 = !DILocation(line: 0, scope: !262)
!265 = !DILocation(line: 19, column: 9, scope: !262)
!266 = !DILocation(line: 20, column: 5, scope: !262)
!267 = distinct !DISubprogram(name: "~Dog", linkageName: "_ZN3DogD2Ev", scope: !238, file: !2, line: 16, type: !242, scopeLine: 16, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !268, retainedNodes: !223)
!268 = !DISubprogram(name: "~Dog", scope: !238, type: !242, containingType: !238, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!269 = !DILocalVariable(name: "this", arg: 1, scope: !267, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!270 = !DILocation(line: 0, scope: !267)
!271 = !DILocation(line: 16, column: 7, scope: !272)
!272 = distinct !DILexicalBlock(scope: !267, file: !2, line: 16, column: 7)
!273 = !DILocation(line: 16, column: 7, scope: !267)
!274 = distinct !DISubprogram(name: "~Dog", linkageName: "_ZN3DogD0Ev", scope: !238, file: !2, line: 16, type: !242, scopeLine: 16, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !268, retainedNodes: !223)
!275 = !DILocalVariable(name: "this", arg: 1, scope: !274, type: !254, flags: DIFlagArtificial | DIFlagObjectPointer)
!276 = !DILocation(line: 0, scope: !274)
!277 = !DILocation(line: 16, column: 7, scope: !274)
!278 = distinct !DISubprogram(name: "speak", linkageName: "_ZN6Animal5speakEv", scope: !226, file: !2, line: 10, type: !232, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !231, retainedNodes: !223)
!279 = !DILocalVariable(name: "this", arg: 1, scope: !278, type: !225, flags: DIFlagArtificial | DIFlagObjectPointer)
!280 = !DILocation(line: 0, scope: !278)
!281 = !DILocation(line: 11, column: 9, scope: !278)
!282 = !DILocation(line: 12, column: 5, scope: !278)
!283 = distinct !DISubprogram(name: "~Animal", linkageName: "_ZN6AnimalD2Ev", scope: !226, file: !2, line: 13, type: !232, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !235, retainedNodes: !223)
!284 = !DILocalVariable(name: "this", arg: 1, scope: !283, type: !225, flags: DIFlagArtificial | DIFlagObjectPointer)
!285 = !DILocation(line: 0, scope: !283)
!286 = !DILocation(line: 13, column: 31, scope: !283)
!287 = distinct !DISubprogram(name: "~Animal", linkageName: "_ZN6AnimalD0Ev", scope: !226, file: !2, line: 13, type: !232, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !13, declaration: !235, retainedNodes: !223)
!288 = !DILocalVariable(name: "this", arg: 1, scope: !287, type: !225, flags: DIFlagArtificial | DIFlagObjectPointer)
!289 = !DILocation(line: 0, scope: !287)
!290 = !DILocation(line: 13, column: 31, scope: !287)
