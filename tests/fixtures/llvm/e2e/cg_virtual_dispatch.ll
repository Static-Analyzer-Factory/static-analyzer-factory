; ModuleID = '/workspace/tests/programs/cpp/cg_virtual_dispatch.cpp'
source_filename = "/workspace/tests/programs/cpp/cg_virtual_dispatch.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.UnsafeProcessor = type { %class.Processor }
%class.Processor = type { ptr }

$_ZN15UnsafeProcessorC2Ev = comdat any

$_ZN15UnsafeProcessorD2Ev = comdat any

$_ZN9ProcessorC2Ev = comdat any

$_ZN15UnsafeProcessor7processEPKc = comdat any

$_ZN15UnsafeProcessorD0Ev = comdat any

$_ZN9ProcessorD2Ev = comdat any

$_ZN9ProcessorD0Ev = comdat any

$_ZTV15UnsafeProcessor = comdat any

$_ZTS15UnsafeProcessor = comdat any

$_ZTS9Processor = comdat any

$_ZTI9Processor = comdat any

$_ZTI15UnsafeProcessor = comdat any

$_ZTV9Processor = comdat any

@.str = private unnamed_addr constant [4 x i8] c"CMD\00", align 1, !dbg !0
@_ZTV15UnsafeProcessor = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI15UnsafeProcessor, ptr @_ZN15UnsafeProcessor7processEPKc, ptr @_ZN15UnsafeProcessorD2Ev, ptr @_ZN15UnsafeProcessorD0Ev] }, comdat, align 8
@_ZTVN10__cxxabiv120__si_class_type_infoE = external global [0 x ptr]
@_ZTS15UnsafeProcessor = linkonce_odr dso_local constant [18 x i8] c"15UnsafeProcessor\00", comdat, align 1
@_ZTVN10__cxxabiv117__class_type_infoE = external global [0 x ptr]
@_ZTS9Processor = linkonce_odr dso_local constant [11 x i8] c"9Processor\00", comdat, align 1
@_ZTI9Processor = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS9Processor }, comdat, align 8
@_ZTI15UnsafeProcessor = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS15UnsafeProcessor, ptr @_ZTI9Processor }, comdat, align 8
@_ZTV9Processor = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI9Processor, ptr @__cxa_pure_virtual, ptr @_ZN9ProcessorD2Ev, ptr @_ZN9ProcessorD0Ev] }, comdat, align 8

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z13run_processorP9ProcessorPKc(ptr noundef %0, ptr noundef %1) #0 !dbg !228 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !245, metadata !DIExpression()), !dbg !246
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !247, metadata !DIExpression()), !dbg !248
  %5 = load ptr, ptr %3, align 8, !dbg !249
  %6 = load ptr, ptr %4, align 8, !dbg !250
  %7 = load ptr, ptr %5, align 8, !dbg !251
  %8 = getelementptr inbounds ptr, ptr %7, i64 0, !dbg !251
  %9 = load ptr, ptr %8, align 8, !dbg !251
  call void %9(ptr noundef nonnull align 8 dereferenceable(8) %5, ptr noundef %6), !dbg !251
  ret void, !dbg !252
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #2 personality ptr @__gxx_personality_v0 !dbg !253 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca %class.UnsafeProcessor, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !254, metadata !DIExpression()), !dbg !255
  %6 = call ptr @getenv(ptr noundef @.str) #8, !dbg !256
  store ptr %6, ptr %2, align 8, !dbg !255
  call void @llvm.dbg.declare(metadata ptr %3, metadata !257, metadata !DIExpression()), !dbg !265
  call void @_ZN15UnsafeProcessorC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !265
  %7 = load ptr, ptr %2, align 8, !dbg !266
  invoke void @_Z13run_processorP9ProcessorPKc(ptr noundef %3, ptr noundef %7)
          to label %8 unwind label %10, !dbg !267

8:                                                ; preds = %0
  store i32 0, ptr %1, align 4, !dbg !268
  call void @_ZN15UnsafeProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !269
  %9 = load i32, ptr %1, align 4, !dbg !269
  ret i32 %9, !dbg !269

10:                                               ; preds = %0
  %11 = landingpad { ptr, i32 }
          cleanup, !dbg !269
  %12 = extractvalue { ptr, i32 } %11, 0, !dbg !269
  store ptr %12, ptr %4, align 8, !dbg !269
  %13 = extractvalue { ptr, i32 } %11, 1, !dbg !269
  store i32 %13, ptr %5, align 4, !dbg !269
  call void @_ZN15UnsafeProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !269
  br label %14, !dbg !269

14:                                               ; preds = %10
  %15 = load ptr, ptr %4, align 8, !dbg !269
  %16 = load i32, ptr %5, align 4, !dbg !269
  %17 = insertvalue { ptr, i32 } poison, ptr %15, 0, !dbg !269
  %18 = insertvalue { ptr, i32 } %17, i32 %16, 1, !dbg !269
  resume { ptr, i32 } %18, !dbg !269
}

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #3

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN15UnsafeProcessorC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !270 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !274, metadata !DIExpression()), !dbg !276
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9ProcessorC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !277
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV15UnsafeProcessor, i32 0, i32 0, i32 2), ptr %3, align 8, !dbg !277
  ret void, !dbg !277
}

declare i32 @__gxx_personality_v0(...)

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN15UnsafeProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !278 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !280, metadata !DIExpression()), !dbg !281
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9ProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !282
  ret void, !dbg !284
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9ProcessorC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !285 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !287, metadata !DIExpression()), !dbg !288
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV9Processor, i32 0, i32 0, i32 2), ptr %3, align 8, !dbg !289
  ret void, !dbg !289
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN15UnsafeProcessor7processEPKc(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #0 comdat align 2 !dbg !290 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !291, metadata !DIExpression()), !dbg !292
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !293, metadata !DIExpression()), !dbg !294
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8, !dbg !295
  %7 = call i32 @system(ptr noundef %6), !dbg !296
  ret void, !dbg !297
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN15UnsafeProcessorD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !298 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !299, metadata !DIExpression()), !dbg !300
  %3 = load ptr, ptr %2, align 8
  call void @_ZN15UnsafeProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #8, !dbg !301
  call void @_ZdlPv(ptr noundef %3) #9, !dbg !301
  ret void, !dbg !301
}

declare void @__cxa_pure_virtual() unnamed_addr

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9ProcessorD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !302 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !303, metadata !DIExpression()), !dbg !304
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !305
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9ProcessorD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !306 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !307, metadata !DIExpression()), !dbg !308
  %3 = load ptr, ptr %2, align 8
  call void @llvm.trap() #10, !dbg !309
  unreachable, !dbg !309
}

; Function Attrs: cold noreturn nounwind memory(inaccessiblemem: write)
declare void @llvm.trap() #5

declare i32 @system(ptr noundef) #6

; Function Attrs: nobuiltin nounwind
declare void @_ZdlPv(ptr noundef) #7

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { cold noreturn nounwind memory(inaccessiblemem: write) }
attributes #6 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #7 = { nobuiltin nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #8 = { nounwind }
attributes #9 = { builtin nounwind }
attributes #10 = { noreturn nounwind }

!llvm.dbg.cu = !{!8}
!llvm.module.flags = !{!220, !221, !222, !223, !224, !225, !226}
!llvm.ident = !{!227}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 23, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "cpp/cg_virtual_dispatch.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "d1ffc3b212cf9579ec71bc565227932b")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !6)
!4 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !5)
!5 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!6 = !{!7}
!7 = !DISubrange(count: 4)
!8 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !9, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !10, imports: !11, splitDebugInlining: false, nameTableKind: None)
!9 = !DIFile(filename: "/workspace/tests/programs/cpp/cg_virtual_dispatch.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "d1ffc3b212cf9579ec71bc565227932b")
!10 = !{!0}
!11 = !{!12, !20, !24, !31, !35, !43, !48, !50, !56, !60, !64, !74, !76, !80, !84, !88, !93, !97, !101, !105, !109, !117, !121, !125, !127, !131, !135, !140, !146, !150, !154, !156, !164, !168, !176, !178, !182, !186, !190, !194, !199, !204, !209, !210, !211, !212, !214, !215, !216, !217, !218, !219}
!12 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !14, file: !19, line: 52)
!13 = !DINamespace(name: "std", scope: null)
!14 = !DISubprogram(name: "abs", scope: !15, file: !15, line: 980, type: !16, flags: DIFlagPrototyped, spFlags: 0)
!15 = !DIFile(filename: "/usr/include/stdlib.h", directory: "", checksumkind: CSK_MD5, checksum: "7fa2ecb2348a66f8b44ab9a15abd0b72")
!16 = !DISubroutineType(types: !17)
!17 = !{!18, !18}
!18 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!19 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/bits/std_abs.h", directory: "")
!20 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !21, file: !23, line: 131)
!21 = !DIDerivedType(tag: DW_TAG_typedef, name: "div_t", file: !15, line: 63, baseType: !22)
!22 = !DICompositeType(tag: DW_TAG_structure_type, file: !15, line: 59, size: 64, flags: DIFlagFwdDecl, identifier: "_ZTS5div_t")
!23 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdlib", directory: "")
!24 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !25, file: !23, line: 132)
!25 = !DIDerivedType(tag: DW_TAG_typedef, name: "ldiv_t", file: !15, line: 71, baseType: !26)
!26 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !15, line: 67, size: 128, flags: DIFlagTypePassByValue, elements: !27, identifier: "_ZTS6ldiv_t")
!27 = !{!28, !30}
!28 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !26, file: !15, line: 69, baseType: !29, size: 64)
!29 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!30 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !26, file: !15, line: 70, baseType: !29, size: 64, offset: 64)
!31 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !32, file: !23, line: 134)
!32 = !DISubprogram(name: "abort", scope: !15, file: !15, line: 730, type: !33, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!33 = !DISubroutineType(types: !34)
!34 = !{null}
!35 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !36, file: !23, line: 136)
!36 = !DISubprogram(name: "aligned_alloc", scope: !15, file: !15, line: 724, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!37 = !DISubroutineType(types: !38)
!38 = !{!39, !40, !40}
!39 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!40 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !41, line: 18, baseType: !42)
!41 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!42 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!43 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !44, file: !23, line: 138)
!44 = !DISubprogram(name: "atexit", scope: !15, file: !15, line: 734, type: !45, flags: DIFlagPrototyped, spFlags: 0)
!45 = !DISubroutineType(types: !46)
!46 = !{!18, !47}
!47 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !33, size: 64)
!48 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !49, file: !23, line: 141)
!49 = !DISubprogram(name: "at_quick_exit", scope: !15, file: !15, line: 739, type: !45, flags: DIFlagPrototyped, spFlags: 0)
!50 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !51, file: !23, line: 144)
!51 = !DISubprogram(name: "atof", scope: !15, file: !15, line: 102, type: !52, flags: DIFlagPrototyped, spFlags: 0)
!52 = !DISubroutineType(types: !53)
!53 = !{!54, !55}
!54 = !DIBasicType(name: "double", size: 64, encoding: DW_ATE_float)
!55 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!56 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !57, file: !23, line: 145)
!57 = !DISubprogram(name: "atoi", scope: !15, file: !15, line: 105, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!58 = !DISubroutineType(types: !59)
!59 = !{!18, !55}
!60 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !61, file: !23, line: 146)
!61 = !DISubprogram(name: "atol", scope: !15, file: !15, line: 108, type: !62, flags: DIFlagPrototyped, spFlags: 0)
!62 = !DISubroutineType(types: !63)
!63 = !{!29, !55}
!64 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !65, file: !23, line: 147)
!65 = !DISubprogram(name: "bsearch", scope: !15, file: !15, line: 960, type: !66, flags: DIFlagPrototyped, spFlags: 0)
!66 = !DISubroutineType(types: !67)
!67 = !{!39, !68, !68, !40, !40, !70}
!68 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !69, size: 64)
!69 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!70 = !DIDerivedType(tag: DW_TAG_typedef, name: "__compar_fn_t", file: !15, line: 948, baseType: !71)
!71 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !72, size: 64)
!72 = !DISubroutineType(types: !73)
!73 = !{!18, !68, !68}
!74 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !75, file: !23, line: 148)
!75 = !DISubprogram(name: "calloc", scope: !15, file: !15, line: 675, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!76 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !77, file: !23, line: 149)
!77 = !DISubprogram(name: "div", scope: !15, file: !15, line: 992, type: !78, flags: DIFlagPrototyped, spFlags: 0)
!78 = !DISubroutineType(types: !79)
!79 = !{!21, !18, !18}
!80 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !81, file: !23, line: 150)
!81 = !DISubprogram(name: "exit", scope: !15, file: !15, line: 756, type: !82, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!82 = !DISubroutineType(types: !83)
!83 = !{null, !18}
!84 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !85, file: !23, line: 151)
!85 = !DISubprogram(name: "free", scope: !15, file: !15, line: 687, type: !86, flags: DIFlagPrototyped, spFlags: 0)
!86 = !DISubroutineType(types: !87)
!87 = !{null, !39}
!88 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !89, file: !23, line: 152)
!89 = !DISubprogram(name: "getenv", scope: !15, file: !15, line: 773, type: !90, flags: DIFlagPrototyped, spFlags: 0)
!90 = !DISubroutineType(types: !91)
!91 = !{!92, !55}
!92 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!93 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !94, file: !23, line: 153)
!94 = !DISubprogram(name: "labs", scope: !15, file: !15, line: 981, type: !95, flags: DIFlagPrototyped, spFlags: 0)
!95 = !DISubroutineType(types: !96)
!96 = !{!29, !29}
!97 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !98, file: !23, line: 154)
!98 = !DISubprogram(name: "ldiv", scope: !15, file: !15, line: 994, type: !99, flags: DIFlagPrototyped, spFlags: 0)
!99 = !DISubroutineType(types: !100)
!100 = !{!25, !29, !29}
!101 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !102, file: !23, line: 155)
!102 = !DISubprogram(name: "malloc", scope: !15, file: !15, line: 672, type: !103, flags: DIFlagPrototyped, spFlags: 0)
!103 = !DISubroutineType(types: !104)
!104 = !{!39, !40}
!105 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !106, file: !23, line: 157)
!106 = !DISubprogram(name: "mblen", scope: !15, file: !15, line: 1062, type: !107, flags: DIFlagPrototyped, spFlags: 0)
!107 = !DISubroutineType(types: !108)
!108 = !{!18, !55, !40}
!109 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !110, file: !23, line: 158)
!110 = !DISubprogram(name: "mbstowcs", scope: !15, file: !15, line: 1073, type: !111, flags: DIFlagPrototyped, spFlags: 0)
!111 = !DISubroutineType(types: !112)
!112 = !{!40, !113, !116, !40}
!113 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !114)
!114 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !115, size: 64)
!115 = !DIBasicType(name: "wchar_t", size: 32, encoding: DW_ATE_unsigned)
!116 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !55)
!117 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !118, file: !23, line: 159)
!118 = !DISubprogram(name: "mbtowc", scope: !15, file: !15, line: 1065, type: !119, flags: DIFlagPrototyped, spFlags: 0)
!119 = !DISubroutineType(types: !120)
!120 = !{!18, !113, !116, !40}
!121 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !122, file: !23, line: 161)
!122 = !DISubprogram(name: "qsort", scope: !15, file: !15, line: 970, type: !123, flags: DIFlagPrototyped, spFlags: 0)
!123 = !DISubroutineType(types: !124)
!124 = !{null, !39, !40, !40, !70}
!125 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !126, file: !23, line: 164)
!126 = !DISubprogram(name: "quick_exit", scope: !15, file: !15, line: 762, type: !82, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!127 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !128, file: !23, line: 167)
!128 = !DISubprogram(name: "rand", scope: !15, file: !15, line: 573, type: !129, flags: DIFlagPrototyped, spFlags: 0)
!129 = !DISubroutineType(types: !130)
!130 = !{!18}
!131 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !132, file: !23, line: 168)
!132 = !DISubprogram(name: "realloc", scope: !15, file: !15, line: 683, type: !133, flags: DIFlagPrototyped, spFlags: 0)
!133 = !DISubroutineType(types: !134)
!134 = !{!39, !39, !40}
!135 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !136, file: !23, line: 169)
!136 = !DISubprogram(name: "srand", scope: !15, file: !15, line: 575, type: !137, flags: DIFlagPrototyped, spFlags: 0)
!137 = !DISubroutineType(types: !138)
!138 = !{null, !139}
!139 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!140 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !141, file: !23, line: 170)
!141 = !DISubprogram(name: "strtod", scope: !15, file: !15, line: 118, type: !142, flags: DIFlagPrototyped, spFlags: 0)
!142 = !DISubroutineType(types: !143)
!143 = !{!54, !116, !144}
!144 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !145)
!145 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !92, size: 64)
!146 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !147, file: !23, line: 171)
!147 = !DISubprogram(name: "strtol", linkageName: "__isoc23_strtol", scope: !15, file: !15, line: 215, type: !148, flags: DIFlagPrototyped, spFlags: 0)
!148 = !DISubroutineType(types: !149)
!149 = !{!29, !116, !144, !18}
!150 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !151, file: !23, line: 172)
!151 = !DISubprogram(name: "strtoul", linkageName: "__isoc23_strtoul", scope: !15, file: !15, line: 219, type: !152, flags: DIFlagPrototyped, spFlags: 0)
!152 = !DISubroutineType(types: !153)
!153 = !{!42, !116, !144, !18}
!154 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !155, file: !23, line: 173)
!155 = !DISubprogram(name: "system", scope: !15, file: !15, line: 923, type: !58, flags: DIFlagPrototyped, spFlags: 0)
!156 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !157, file: !23, line: 175)
!157 = !DISubprogram(name: "wcstombs", scope: !15, file: !15, line: 1077, type: !158, flags: DIFlagPrototyped, spFlags: 0)
!158 = !DISubroutineType(types: !159)
!159 = !{!40, !160, !161, !40}
!160 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !92)
!161 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !162)
!162 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !163, size: 64)
!163 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !115)
!164 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !165, file: !23, line: 176)
!165 = !DISubprogram(name: "wctomb", scope: !15, file: !15, line: 1069, type: !166, flags: DIFlagPrototyped, spFlags: 0)
!166 = !DISubroutineType(types: !167)
!167 = !{!18, !92, !115}
!168 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !170, file: !23, line: 204)
!169 = !DINamespace(name: "__gnu_cxx", scope: null)
!170 = !DIDerivedType(tag: DW_TAG_typedef, name: "lldiv_t", file: !15, line: 81, baseType: !171)
!171 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !15, line: 77, size: 128, flags: DIFlagTypePassByValue, elements: !172, identifier: "_ZTS7lldiv_t")
!172 = !{!173, !175}
!173 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !171, file: !15, line: 79, baseType: !174, size: 64)
!174 = !DIBasicType(name: "long long", size: 64, encoding: DW_ATE_signed)
!175 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !171, file: !15, line: 80, baseType: !174, size: 64, offset: 64)
!176 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !177, file: !23, line: 210)
!177 = !DISubprogram(name: "_Exit", scope: !15, file: !15, line: 768, type: !82, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!178 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !179, file: !23, line: 214)
!179 = !DISubprogram(name: "llabs", scope: !15, file: !15, line: 984, type: !180, flags: DIFlagPrototyped, spFlags: 0)
!180 = !DISubroutineType(types: !181)
!181 = !{!174, !174}
!182 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !183, file: !23, line: 220)
!183 = !DISubprogram(name: "lldiv", scope: !15, file: !15, line: 998, type: !184, flags: DIFlagPrototyped, spFlags: 0)
!184 = !DISubroutineType(types: !185)
!185 = !{!170, !174, !174}
!186 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !187, file: !23, line: 231)
!187 = !DISubprogram(name: "atoll", scope: !15, file: !15, line: 113, type: !188, flags: DIFlagPrototyped, spFlags: 0)
!188 = !DISubroutineType(types: !189)
!189 = !{!174, !55}
!190 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !191, file: !23, line: 232)
!191 = !DISubprogram(name: "strtoll", linkageName: "__isoc23_strtoll", scope: !15, file: !15, line: 238, type: !192, flags: DIFlagPrototyped, spFlags: 0)
!192 = !DISubroutineType(types: !193)
!193 = !{!174, !116, !144, !18}
!194 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !195, file: !23, line: 233)
!195 = !DISubprogram(name: "strtoull", linkageName: "__isoc23_strtoull", scope: !15, file: !15, line: 243, type: !196, flags: DIFlagPrototyped, spFlags: 0)
!196 = !DISubroutineType(types: !197)
!197 = !{!198, !116, !144, !18}
!198 = !DIBasicType(name: "unsigned long long", size: 64, encoding: DW_ATE_unsigned)
!199 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !200, file: !23, line: 235)
!200 = !DISubprogram(name: "strtof", scope: !15, file: !15, line: 124, type: !201, flags: DIFlagPrototyped, spFlags: 0)
!201 = !DISubroutineType(types: !202)
!202 = !{!203, !116, !144}
!203 = !DIBasicType(name: "float", size: 32, encoding: DW_ATE_float)
!204 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !169, entity: !205, file: !23, line: 236)
!205 = !DISubprogram(name: "strtold", scope: !15, file: !15, line: 127, type: !206, flags: DIFlagPrototyped, spFlags: 0)
!206 = !DISubroutineType(types: !207)
!207 = !{!208, !116, !144}
!208 = !DIBasicType(name: "long double", size: 128, encoding: DW_ATE_float)
!209 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !170, file: !23, line: 244)
!210 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !177, file: !23, line: 246)
!211 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !179, file: !23, line: 248)
!212 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !213, file: !23, line: 249)
!213 = !DISubprogram(name: "div", linkageName: "_ZN9__gnu_cxx3divExx", scope: !169, file: !23, line: 217, type: !184, flags: DIFlagPrototyped, spFlags: 0)
!214 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !183, file: !23, line: 250)
!215 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !187, file: !23, line: 252)
!216 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !200, file: !23, line: 253)
!217 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !191, file: !23, line: 254)
!218 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !195, file: !23, line: 255)
!219 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !13, entity: !205, file: !23, line: 256)
!220 = !{i32 7, !"Dwarf Version", i32 5}
!221 = !{i32 2, !"Debug Info Version", i32 3}
!222 = !{i32 1, !"wchar_size", i32 4}
!223 = !{i32 8, !"PIC Level", i32 2}
!224 = !{i32 7, !"PIE Level", i32 2}
!225 = !{i32 7, !"uwtable", i32 2}
!226 = !{i32 7, !"frame-pointer", i32 1}
!227 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!228 = distinct !DISubprogram(name: "run_processor", linkageName: "_Z13run_processorP9ProcessorPKc", scope: !2, file: !2, line: 18, type: !229, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, retainedNodes: !244)
!229 = !DISubroutineType(types: !230)
!230 = !{null, !231, !55}
!231 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !232, size: 64)
!232 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Processor", file: !2, line: 5, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !233, vtableHolder: !232, identifier: "_ZTS9Processor")
!233 = !{!234, !237, !241}
!234 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Processor", scope: !2, file: !2, baseType: !235, size: 64, flags: DIFlagArtificial)
!235 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !236, size: 64)
!236 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "__vtbl_ptr_type", baseType: !129, size: 64)
!237 = !DISubprogram(name: "process", linkageName: "_ZN9Processor7processEPKc", scope: !232, file: !2, line: 7, type: !238, scopeLine: 7, containingType: !232, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagPureVirtual)
!238 = !DISubroutineType(types: !239)
!239 = !{null, !240, !55}
!240 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !232, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!241 = !DISubprogram(name: "~Processor", scope: !232, file: !2, line: 8, type: !242, scopeLine: 8, containingType: !232, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!242 = !DISubroutineType(types: !243)
!243 = !{null, !240}
!244 = !{}
!245 = !DILocalVariable(name: "p", arg: 1, scope: !228, file: !2, line: 18, type: !231)
!246 = !DILocation(line: 18, column: 31, scope: !228)
!247 = !DILocalVariable(name: "data", arg: 2, scope: !228, file: !2, line: 18, type: !55)
!248 = !DILocation(line: 18, column: 46, scope: !228)
!249 = !DILocation(line: 19, column: 5, scope: !228)
!250 = !DILocation(line: 19, column: 16, scope: !228)
!251 = !DILocation(line: 19, column: 8, scope: !228)
!252 = !DILocation(line: 20, column: 1, scope: !228)
!253 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 22, type: !129, scopeLine: 22, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, retainedNodes: !244)
!254 = !DILocalVariable(name: "input", scope: !253, file: !2, line: 23, type: !55)
!255 = !DILocation(line: 23, column: 17, scope: !253)
!256 = !DILocation(line: 23, column: 25, scope: !253)
!257 = !DILocalVariable(name: "proc", scope: !253, file: !2, line: 24, type: !258)
!258 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "UnsafeProcessor", file: !2, line: 11, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !259, vtableHolder: !232, identifier: "_ZTS15UnsafeProcessor")
!259 = !{!260, !261}
!260 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !258, baseType: !232, flags: DIFlagPublic, extraData: i32 0)
!261 = !DISubprogram(name: "process", linkageName: "_ZN15UnsafeProcessor7processEPKc", scope: !258, file: !2, line: 13, type: !262, scopeLine: 13, containingType: !258, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!262 = !DISubroutineType(types: !263)
!263 = !{null, !264, !55}
!264 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !258, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!265 = !DILocation(line: 24, column: 21, scope: !253)
!266 = !DILocation(line: 25, column: 26, scope: !253)
!267 = !DILocation(line: 25, column: 5, scope: !253)
!268 = !DILocation(line: 26, column: 5, scope: !253)
!269 = !DILocation(line: 27, column: 1, scope: !253)
!270 = distinct !DISubprogram(name: "UnsafeProcessor", linkageName: "_ZN15UnsafeProcessorC2Ev", scope: !258, file: !2, line: 11, type: !271, scopeLine: 11, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !273, retainedNodes: !244)
!271 = !DISubroutineType(types: !272)
!272 = !{null, !264}
!273 = !DISubprogram(name: "UnsafeProcessor", scope: !258, type: !271, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!274 = !DILocalVariable(name: "this", arg: 1, scope: !270, type: !275, flags: DIFlagArtificial | DIFlagObjectPointer)
!275 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !258, size: 64)
!276 = !DILocation(line: 0, scope: !270)
!277 = !DILocation(line: 11, column: 7, scope: !270)
!278 = distinct !DISubprogram(name: "~UnsafeProcessor", linkageName: "_ZN15UnsafeProcessorD2Ev", scope: !258, file: !2, line: 11, type: !271, scopeLine: 11, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !279, retainedNodes: !244)
!279 = !DISubprogram(name: "~UnsafeProcessor", scope: !258, type: !271, containingType: !258, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!280 = !DILocalVariable(name: "this", arg: 1, scope: !278, type: !275, flags: DIFlagArtificial | DIFlagObjectPointer)
!281 = !DILocation(line: 0, scope: !278)
!282 = !DILocation(line: 11, column: 7, scope: !283)
!283 = distinct !DILexicalBlock(scope: !278, file: !2, line: 11, column: 7)
!284 = !DILocation(line: 11, column: 7, scope: !278)
!285 = distinct !DISubprogram(name: "Processor", linkageName: "_ZN9ProcessorC2Ev", scope: !232, file: !2, line: 5, type: !242, scopeLine: 5, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !286, retainedNodes: !244)
!286 = !DISubprogram(name: "Processor", scope: !232, type: !242, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!287 = !DILocalVariable(name: "this", arg: 1, scope: !285, type: !231, flags: DIFlagArtificial | DIFlagObjectPointer)
!288 = !DILocation(line: 0, scope: !285)
!289 = !DILocation(line: 5, column: 7, scope: !285)
!290 = distinct !DISubprogram(name: "process", linkageName: "_ZN15UnsafeProcessor7processEPKc", scope: !258, file: !2, line: 13, type: !262, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !261, retainedNodes: !244)
!291 = !DILocalVariable(name: "this", arg: 1, scope: !290, type: !275, flags: DIFlagArtificial | DIFlagObjectPointer)
!292 = !DILocation(line: 0, scope: !290)
!293 = !DILocalVariable(name: "data", arg: 2, scope: !290, file: !2, line: 13, type: !55)
!294 = !DILocation(line: 13, column: 30, scope: !290)
!295 = !DILocation(line: 14, column: 16, scope: !290)
!296 = !DILocation(line: 14, column: 9, scope: !290)
!297 = !DILocation(line: 15, column: 5, scope: !290)
!298 = distinct !DISubprogram(name: "~UnsafeProcessor", linkageName: "_ZN15UnsafeProcessorD0Ev", scope: !258, file: !2, line: 11, type: !271, scopeLine: 11, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !279, retainedNodes: !244)
!299 = !DILocalVariable(name: "this", arg: 1, scope: !298, type: !275, flags: DIFlagArtificial | DIFlagObjectPointer)
!300 = !DILocation(line: 0, scope: !298)
!301 = !DILocation(line: 11, column: 7, scope: !298)
!302 = distinct !DISubprogram(name: "~Processor", linkageName: "_ZN9ProcessorD2Ev", scope: !232, file: !2, line: 8, type: !242, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !241, retainedNodes: !244)
!303 = !DILocalVariable(name: "this", arg: 1, scope: !302, type: !231, flags: DIFlagArtificial | DIFlagObjectPointer)
!304 = !DILocation(line: 0, scope: !302)
!305 = !DILocation(line: 8, column: 34, scope: !302)
!306 = distinct !DISubprogram(name: "~Processor", linkageName: "_ZN9ProcessorD0Ev", scope: !232, file: !2, line: 8, type: !242, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !8, declaration: !241, retainedNodes: !244)
!307 = !DILocalVariable(name: "this", arg: 1, scope: !306, type: !231, flags: DIFlagArtificial | DIFlagObjectPointer)
!308 = !DILocation(line: 0, scope: !306)
!309 = !DILocation(line: 8, column: 34, scope: !306)
