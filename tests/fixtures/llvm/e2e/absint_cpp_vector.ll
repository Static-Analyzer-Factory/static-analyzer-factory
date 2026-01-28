; ModuleID = '/workspace/tests/programs/cpp/absint_cpp_vector.cpp'
source_filename = "/workspace/tests/programs/cpp/absint_cpp_vector.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.SimpleVector = type { ptr, i32, i32 }

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local noundef ptr @_Z10vec_createi(i32 noundef %0) #0 !dbg !230 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !234, metadata !DIExpression()), !dbg !235
  call void @llvm.dbg.declare(metadata ptr %4, metadata !236, metadata !DIExpression()), !dbg !237
  %5 = call noalias ptr @malloc(i64 noundef 16) #5, !dbg !238
  store ptr %5, ptr %4, align 8, !dbg !237
  %6 = load ptr, ptr %4, align 8, !dbg !239
  %7 = icmp ne ptr %6, null, !dbg !239
  br i1 %7, label %9, label %8, !dbg !241

8:                                                ; preds = %1
  store ptr null, ptr %2, align 8, !dbg !242
  br label %22, !dbg !242

9:                                                ; preds = %1
  %10 = load i32, ptr %3, align 4, !dbg !243
  %11 = sext i32 %10 to i64, !dbg !243
  %12 = mul i64 %11, 4, !dbg !244
  %13 = call noalias ptr @malloc(i64 noundef %12) #5, !dbg !245
  %14 = load ptr, ptr %4, align 8, !dbg !246
  %15 = getelementptr inbounds %struct.SimpleVector, ptr %14, i32 0, i32 0, !dbg !247
  store ptr %13, ptr %15, align 8, !dbg !248
  %16 = load ptr, ptr %4, align 8, !dbg !249
  %17 = getelementptr inbounds %struct.SimpleVector, ptr %16, i32 0, i32 1, !dbg !250
  store i32 0, ptr %17, align 8, !dbg !251
  %18 = load i32, ptr %3, align 4, !dbg !252
  %19 = load ptr, ptr %4, align 8, !dbg !253
  %20 = getelementptr inbounds %struct.SimpleVector, ptr %19, i32 0, i32 2, !dbg !254
  store i32 %18, ptr %20, align 4, !dbg !255
  %21 = load ptr, ptr %4, align 8, !dbg !256
  store ptr %21, ptr %2, align 8, !dbg !257
  br label %22, !dbg !257

22:                                               ; preds = %9, %8
  %23 = load ptr, ptr %2, align 8, !dbg !258
  ret ptr %23, !dbg !258
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local void @_Z8vec_pushP12SimpleVectori(ptr noundef %0, i32 noundef %1) #0 !dbg !259 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !262, metadata !DIExpression()), !dbg !263
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !264, metadata !DIExpression()), !dbg !265
  %5 = load ptr, ptr %3, align 8, !dbg !266
  %6 = getelementptr inbounds %struct.SimpleVector, ptr %5, i32 0, i32 1, !dbg !268
  %7 = load i32, ptr %6, align 8, !dbg !268
  %8 = load ptr, ptr %3, align 8, !dbg !269
  %9 = getelementptr inbounds %struct.SimpleVector, ptr %8, i32 0, i32 2, !dbg !270
  %10 = load i32, ptr %9, align 4, !dbg !270
  %11 = icmp slt i32 %7, %10, !dbg !271
  br i1 %11, label %12, label %28, !dbg !272

12:                                               ; preds = %2
  %13 = load i32, ptr %4, align 4, !dbg !273
  %14 = load ptr, ptr %3, align 8, !dbg !275
  %15 = getelementptr inbounds %struct.SimpleVector, ptr %14, i32 0, i32 0, !dbg !276
  %16 = load ptr, ptr %15, align 8, !dbg !276
  %17 = load ptr, ptr %3, align 8, !dbg !277
  %18 = getelementptr inbounds %struct.SimpleVector, ptr %17, i32 0, i32 1, !dbg !278
  %19 = load i32, ptr %18, align 8, !dbg !278
  %20 = sext i32 %19 to i64, !dbg !275
  %21 = getelementptr inbounds i32, ptr %16, i64 %20, !dbg !275
  store i32 %13, ptr %21, align 4, !dbg !279
  %22 = load ptr, ptr %3, align 8, !dbg !280
  %23 = getelementptr inbounds %struct.SimpleVector, ptr %22, i32 0, i32 1, !dbg !281
  %24 = load i32, ptr %23, align 8, !dbg !281
  %25 = add nsw i32 %24, 1, !dbg !282
  %26 = load ptr, ptr %3, align 8, !dbg !283
  %27 = getelementptr inbounds %struct.SimpleVector, ptr %26, i32 0, i32 1, !dbg !284
  store i32 %25, ptr %27, align 8, !dbg !285
  br label %28, !dbg !286

28:                                               ; preds = %12, %2
  ret void, !dbg !287
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local noundef i32 @_Z7vec_getP12SimpleVectori(ptr noundef %0, i32 noundef %1) #0 !dbg !288 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !291, metadata !DIExpression()), !dbg !292
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !293, metadata !DIExpression()), !dbg !294
  %5 = load ptr, ptr %3, align 8, !dbg !295
  %6 = getelementptr inbounds %struct.SimpleVector, ptr %5, i32 0, i32 0, !dbg !296
  %7 = load ptr, ptr %6, align 8, !dbg !296
  %8 = load i32, ptr %4, align 4, !dbg !297
  %9 = sext i32 %8 to i64, !dbg !295
  %10 = getelementptr inbounds i32, ptr %7, i64 %9, !dbg !295
  %11 = load i32, ptr %10, align 4, !dbg !295
  ret i32 %11, !dbg !298
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local noundef i32 @_Z12vec_get_safeP12SimpleVectori(ptr noundef %0, i32 noundef %1) #0 !dbg !299 {
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !300, metadata !DIExpression()), !dbg !301
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !302, metadata !DIExpression()), !dbg !303
  %6 = load i32, ptr %5, align 4, !dbg !304
  %7 = icmp sge i32 %6, 0, !dbg !306
  br i1 %7, label %8, label %22, !dbg !307

8:                                                ; preds = %2
  %9 = load i32, ptr %5, align 4, !dbg !308
  %10 = load ptr, ptr %4, align 8, !dbg !309
  %11 = getelementptr inbounds %struct.SimpleVector, ptr %10, i32 0, i32 1, !dbg !310
  %12 = load i32, ptr %11, align 8, !dbg !310
  %13 = icmp slt i32 %9, %12, !dbg !311
  br i1 %13, label %14, label %22, !dbg !312

14:                                               ; preds = %8
  %15 = load ptr, ptr %4, align 8, !dbg !313
  %16 = getelementptr inbounds %struct.SimpleVector, ptr %15, i32 0, i32 0, !dbg !315
  %17 = load ptr, ptr %16, align 8, !dbg !315
  %18 = load i32, ptr %5, align 4, !dbg !316
  %19 = sext i32 %18 to i64, !dbg !313
  %20 = getelementptr inbounds i32, ptr %17, i64 %19, !dbg !313
  %21 = load i32, ptr %20, align 4, !dbg !313
  store i32 %21, ptr %3, align 4, !dbg !317
  br label %23, !dbg !317

22:                                               ; preds = %8, %2
  store i32 -1, ptr %3, align 4, !dbg !318
  br label %23, !dbg !318

23:                                               ; preds = %22, %14
  %24 = load i32, ptr %3, align 4, !dbg !319
  ret i32 %24, !dbg !319
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local void @_Z8vec_freeP12SimpleVector(ptr noundef %0) #0 !dbg !320 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !323, metadata !DIExpression()), !dbg !324
  %3 = load ptr, ptr %2, align 8, !dbg !325
  %4 = icmp ne ptr %3, null, !dbg !325
  br i1 %4, label %5, label %10, !dbg !327

5:                                                ; preds = %1
  %6 = load ptr, ptr %2, align 8, !dbg !328
  %7 = getelementptr inbounds %struct.SimpleVector, ptr %6, i32 0, i32 0, !dbg !330
  %8 = load ptr, ptr %7, align 8, !dbg !330
  call void @free(ptr noundef %8) #6, !dbg !331
  %9 = load ptr, ptr %2, align 8, !dbg !332
  call void @free(ptr noundef %9) #6, !dbg !333
  br label %10, !dbg !334

10:                                               ; preds = %5, %1
  ret void, !dbg !335
}

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: mustprogress noinline norecurse nounwind optnone uwtable
define dso_local noundef i32 @main() #4 !dbg !336 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !337, metadata !DIExpression()), !dbg !338
  %6 = call noundef ptr @_Z10vec_createi(i32 noundef 16), !dbg !339
  store ptr %6, ptr %2, align 8, !dbg !338
  %7 = load ptr, ptr %2, align 8, !dbg !340
  %8 = icmp ne ptr %7, null, !dbg !340
  br i1 %8, label %10, label %9, !dbg !342

9:                                                ; preds = %0
  store i32 1, ptr %1, align 4, !dbg !343
  br label %30, !dbg !343

10:                                               ; preds = %0
  call void @llvm.dbg.declare(metadata ptr %3, metadata !344, metadata !DIExpression()), !dbg !346
  store i32 0, ptr %3, align 4, !dbg !346
  br label %11, !dbg !347

11:                                               ; preds = %18, %10
  %12 = load i32, ptr %3, align 4, !dbg !348
  %13 = icmp slt i32 %12, 10, !dbg !350
  br i1 %13, label %14, label %21, !dbg !351

14:                                               ; preds = %11
  %15 = load ptr, ptr %2, align 8, !dbg !352
  %16 = load i32, ptr %3, align 4, !dbg !354
  %17 = mul nsw i32 %16, 3, !dbg !355
  call void @_Z8vec_pushP12SimpleVectori(ptr noundef %15, i32 noundef %17), !dbg !356
  br label %18, !dbg !357

18:                                               ; preds = %14
  %19 = load i32, ptr %3, align 4, !dbg !358
  %20 = add nsw i32 %19, 1, !dbg !358
  store i32 %20, ptr %3, align 4, !dbg !358
  br label %11, !dbg !359, !llvm.loop !360

21:                                               ; preds = %11
  call void @llvm.dbg.declare(metadata ptr %4, metadata !363, metadata !DIExpression()), !dbg !364
  %22 = load ptr, ptr %2, align 8, !dbg !365
  %23 = call noundef i32 @_Z12vec_get_safeP12SimpleVectori(ptr noundef %22, i32 noundef 5), !dbg !366
  store i32 %23, ptr %4, align 4, !dbg !364
  call void @llvm.dbg.declare(metadata ptr %5, metadata !367, metadata !DIExpression()), !dbg !368
  %24 = load ptr, ptr %2, align 8, !dbg !369
  %25 = call noundef i32 @_Z7vec_getP12SimpleVectori(ptr noundef %24, i32 noundef 20), !dbg !370
  store i32 %25, ptr %5, align 4, !dbg !368
  %26 = load ptr, ptr %2, align 8, !dbg !371
  call void @_Z8vec_freeP12SimpleVector(ptr noundef %26), !dbg !372
  %27 = load i32, ptr %4, align 4, !dbg !373
  %28 = load i32, ptr %5, align 4, !dbg !374
  %29 = add nsw i32 %27, %28, !dbg !375
  store i32 %29, ptr %1, align 4, !dbg !376
  br label %30, !dbg !376

30:                                               ; preds = %21, %9
  %31 = load i32, ptr %1, align 4, !dbg !377
  ret i32 %31, !dbg !377
}

attributes #0 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { mustprogress noinline norecurse nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { nounwind allocsize(0) }
attributes #6 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!222, !223, !224, !225, !226, !227, !228}
!llvm.ident = !{!229}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, imports: !12, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/cpp/absint_cpp_vector.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "21b4224276fc07f6c79eee4d4a7036a3")
!2 = !{!3, !8}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "SimpleVector", file: !5, line: 6, size: 128, flags: DIFlagTypePassByValue, elements: !6, identifier: "_ZTS12SimpleVector")
!5 = !DIFile(filename: "cpp/absint_cpp_vector.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "21b4224276fc07f6c79eee4d4a7036a3")
!6 = !{!7, !10, !11}
!7 = !DIDerivedType(tag: DW_TAG_member, name: "data", scope: !4, file: !5, line: 7, baseType: !8, size: 64)
!8 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !9, size: 64)
!9 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!10 = !DIDerivedType(tag: DW_TAG_member, name: "size", scope: !4, file: !5, line: 8, baseType: !9, size: 32, offset: 64)
!11 = !DIDerivedType(tag: DW_TAG_member, name: "capacity", scope: !4, file: !5, line: 9, baseType: !9, size: 32, offset: 96)
!12 = !{!13, !20, !24, !31, !35, !43, !48, !50, !58, !62, !66, !76, !78, !82, !86, !90, !95, !99, !103, !107, !111, !119, !123, !127, !129, !133, !137, !142, !148, !152, !156, !158, !166, !170, !178, !180, !184, !188, !192, !196, !201, !206, !211, !212, !213, !214, !216, !217, !218, !219, !220, !221}
!13 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !15, file: !19, line: 52)
!14 = !DINamespace(name: "std", scope: null)
!15 = !DISubprogram(name: "abs", scope: !16, file: !16, line: 980, type: !17, flags: DIFlagPrototyped, spFlags: 0)
!16 = !DIFile(filename: "/usr/include/stdlib.h", directory: "", checksumkind: CSK_MD5, checksum: "7fa2ecb2348a66f8b44ab9a15abd0b72")
!17 = !DISubroutineType(types: !18)
!18 = !{!9, !9}
!19 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/bits/std_abs.h", directory: "")
!20 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !21, file: !23, line: 131)
!21 = !DIDerivedType(tag: DW_TAG_typedef, name: "div_t", file: !16, line: 63, baseType: !22)
!22 = !DICompositeType(tag: DW_TAG_structure_type, file: !16, line: 59, size: 64, flags: DIFlagFwdDecl, identifier: "_ZTS5div_t")
!23 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdlib", directory: "")
!24 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !25, file: !23, line: 132)
!25 = !DIDerivedType(tag: DW_TAG_typedef, name: "ldiv_t", file: !16, line: 71, baseType: !26)
!26 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !16, line: 67, size: 128, flags: DIFlagTypePassByValue, elements: !27, identifier: "_ZTS6ldiv_t")
!27 = !{!28, !30}
!28 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !26, file: !16, line: 69, baseType: !29, size: 64)
!29 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!30 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !26, file: !16, line: 70, baseType: !29, size: 64, offset: 64)
!31 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !32, file: !23, line: 134)
!32 = !DISubprogram(name: "abort", scope: !16, file: !16, line: 730, type: !33, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!33 = !DISubroutineType(types: !34)
!34 = !{null}
!35 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !36, file: !23, line: 136)
!36 = !DISubprogram(name: "aligned_alloc", scope: !16, file: !16, line: 724, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!37 = !DISubroutineType(types: !38)
!38 = !{!39, !40, !40}
!39 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!40 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !41, line: 18, baseType: !42)
!41 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!42 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!43 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !44, file: !23, line: 138)
!44 = !DISubprogram(name: "atexit", scope: !16, file: !16, line: 734, type: !45, flags: DIFlagPrototyped, spFlags: 0)
!45 = !DISubroutineType(types: !46)
!46 = !{!9, !47}
!47 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !33, size: 64)
!48 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !49, file: !23, line: 141)
!49 = !DISubprogram(name: "at_quick_exit", scope: !16, file: !16, line: 739, type: !45, flags: DIFlagPrototyped, spFlags: 0)
!50 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !51, file: !23, line: 144)
!51 = !DISubprogram(name: "atof", scope: !16, file: !16, line: 102, type: !52, flags: DIFlagPrototyped, spFlags: 0)
!52 = !DISubroutineType(types: !53)
!53 = !{!54, !55}
!54 = !DIBasicType(name: "double", size: 64, encoding: DW_ATE_float)
!55 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !56, size: 64)
!56 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !57)
!57 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!58 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !59, file: !23, line: 145)
!59 = !DISubprogram(name: "atoi", scope: !16, file: !16, line: 105, type: !60, flags: DIFlagPrototyped, spFlags: 0)
!60 = !DISubroutineType(types: !61)
!61 = !{!9, !55}
!62 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !63, file: !23, line: 146)
!63 = !DISubprogram(name: "atol", scope: !16, file: !16, line: 108, type: !64, flags: DIFlagPrototyped, spFlags: 0)
!64 = !DISubroutineType(types: !65)
!65 = !{!29, !55}
!66 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !67, file: !23, line: 147)
!67 = !DISubprogram(name: "bsearch", scope: !16, file: !16, line: 960, type: !68, flags: DIFlagPrototyped, spFlags: 0)
!68 = !DISubroutineType(types: !69)
!69 = !{!39, !70, !70, !40, !40, !72}
!70 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !71, size: 64)
!71 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!72 = !DIDerivedType(tag: DW_TAG_typedef, name: "__compar_fn_t", file: !16, line: 948, baseType: !73)
!73 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !74, size: 64)
!74 = !DISubroutineType(types: !75)
!75 = !{!9, !70, !70}
!76 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !77, file: !23, line: 148)
!77 = !DISubprogram(name: "calloc", scope: !16, file: !16, line: 675, type: !37, flags: DIFlagPrototyped, spFlags: 0)
!78 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !79, file: !23, line: 149)
!79 = !DISubprogram(name: "div", scope: !16, file: !16, line: 992, type: !80, flags: DIFlagPrototyped, spFlags: 0)
!80 = !DISubroutineType(types: !81)
!81 = !{!21, !9, !9}
!82 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !83, file: !23, line: 150)
!83 = !DISubprogram(name: "exit", scope: !16, file: !16, line: 756, type: !84, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!84 = !DISubroutineType(types: !85)
!85 = !{null, !9}
!86 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !87, file: !23, line: 151)
!87 = !DISubprogram(name: "free", scope: !16, file: !16, line: 687, type: !88, flags: DIFlagPrototyped, spFlags: 0)
!88 = !DISubroutineType(types: !89)
!89 = !{null, !39}
!90 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !91, file: !23, line: 152)
!91 = !DISubprogram(name: "getenv", scope: !16, file: !16, line: 773, type: !92, flags: DIFlagPrototyped, spFlags: 0)
!92 = !DISubroutineType(types: !93)
!93 = !{!94, !55}
!94 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !57, size: 64)
!95 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !96, file: !23, line: 153)
!96 = !DISubprogram(name: "labs", scope: !16, file: !16, line: 981, type: !97, flags: DIFlagPrototyped, spFlags: 0)
!97 = !DISubroutineType(types: !98)
!98 = !{!29, !29}
!99 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !100, file: !23, line: 154)
!100 = !DISubprogram(name: "ldiv", scope: !16, file: !16, line: 994, type: !101, flags: DIFlagPrototyped, spFlags: 0)
!101 = !DISubroutineType(types: !102)
!102 = !{!25, !29, !29}
!103 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !104, file: !23, line: 155)
!104 = !DISubprogram(name: "malloc", scope: !16, file: !16, line: 672, type: !105, flags: DIFlagPrototyped, spFlags: 0)
!105 = !DISubroutineType(types: !106)
!106 = !{!39, !40}
!107 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !108, file: !23, line: 157)
!108 = !DISubprogram(name: "mblen", scope: !16, file: !16, line: 1062, type: !109, flags: DIFlagPrototyped, spFlags: 0)
!109 = !DISubroutineType(types: !110)
!110 = !{!9, !55, !40}
!111 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !112, file: !23, line: 158)
!112 = !DISubprogram(name: "mbstowcs", scope: !16, file: !16, line: 1073, type: !113, flags: DIFlagPrototyped, spFlags: 0)
!113 = !DISubroutineType(types: !114)
!114 = !{!40, !115, !118, !40}
!115 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !116)
!116 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !117, size: 64)
!117 = !DIBasicType(name: "wchar_t", size: 32, encoding: DW_ATE_unsigned)
!118 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !55)
!119 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !120, file: !23, line: 159)
!120 = !DISubprogram(name: "mbtowc", scope: !16, file: !16, line: 1065, type: !121, flags: DIFlagPrototyped, spFlags: 0)
!121 = !DISubroutineType(types: !122)
!122 = !{!9, !115, !118, !40}
!123 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !124, file: !23, line: 161)
!124 = !DISubprogram(name: "qsort", scope: !16, file: !16, line: 970, type: !125, flags: DIFlagPrototyped, spFlags: 0)
!125 = !DISubroutineType(types: !126)
!126 = !{null, !39, !40, !40, !72}
!127 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !128, file: !23, line: 164)
!128 = !DISubprogram(name: "quick_exit", scope: !16, file: !16, line: 762, type: !84, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!129 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !130, file: !23, line: 167)
!130 = !DISubprogram(name: "rand", scope: !16, file: !16, line: 573, type: !131, flags: DIFlagPrototyped, spFlags: 0)
!131 = !DISubroutineType(types: !132)
!132 = !{!9}
!133 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !134, file: !23, line: 168)
!134 = !DISubprogram(name: "realloc", scope: !16, file: !16, line: 683, type: !135, flags: DIFlagPrototyped, spFlags: 0)
!135 = !DISubroutineType(types: !136)
!136 = !{!39, !39, !40}
!137 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !138, file: !23, line: 169)
!138 = !DISubprogram(name: "srand", scope: !16, file: !16, line: 575, type: !139, flags: DIFlagPrototyped, spFlags: 0)
!139 = !DISubroutineType(types: !140)
!140 = !{null, !141}
!141 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!142 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !143, file: !23, line: 170)
!143 = !DISubprogram(name: "strtod", scope: !16, file: !16, line: 118, type: !144, flags: DIFlagPrototyped, spFlags: 0)
!144 = !DISubroutineType(types: !145)
!145 = !{!54, !118, !146}
!146 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !147)
!147 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !94, size: 64)
!148 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !149, file: !23, line: 171)
!149 = !DISubprogram(name: "strtol", linkageName: "__isoc23_strtol", scope: !16, file: !16, line: 215, type: !150, flags: DIFlagPrototyped, spFlags: 0)
!150 = !DISubroutineType(types: !151)
!151 = !{!29, !118, !146, !9}
!152 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !153, file: !23, line: 172)
!153 = !DISubprogram(name: "strtoul", linkageName: "__isoc23_strtoul", scope: !16, file: !16, line: 219, type: !154, flags: DIFlagPrototyped, spFlags: 0)
!154 = !DISubroutineType(types: !155)
!155 = !{!42, !118, !146, !9}
!156 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !157, file: !23, line: 173)
!157 = !DISubprogram(name: "system", scope: !16, file: !16, line: 923, type: !60, flags: DIFlagPrototyped, spFlags: 0)
!158 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !159, file: !23, line: 175)
!159 = !DISubprogram(name: "wcstombs", scope: !16, file: !16, line: 1077, type: !160, flags: DIFlagPrototyped, spFlags: 0)
!160 = !DISubroutineType(types: !161)
!161 = !{!40, !162, !163, !40}
!162 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !94)
!163 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !164)
!164 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !165, size: 64)
!165 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !117)
!166 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !167, file: !23, line: 176)
!167 = !DISubprogram(name: "wctomb", scope: !16, file: !16, line: 1069, type: !168, flags: DIFlagPrototyped, spFlags: 0)
!168 = !DISubroutineType(types: !169)
!169 = !{!9, !94, !117}
!170 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !172, file: !23, line: 204)
!171 = !DINamespace(name: "__gnu_cxx", scope: null)
!172 = !DIDerivedType(tag: DW_TAG_typedef, name: "lldiv_t", file: !16, line: 81, baseType: !173)
!173 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !16, line: 77, size: 128, flags: DIFlagTypePassByValue, elements: !174, identifier: "_ZTS7lldiv_t")
!174 = !{!175, !177}
!175 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !173, file: !16, line: 79, baseType: !176, size: 64)
!176 = !DIBasicType(name: "long long", size: 64, encoding: DW_ATE_signed)
!177 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !173, file: !16, line: 80, baseType: !176, size: 64, offset: 64)
!178 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !179, file: !23, line: 210)
!179 = !DISubprogram(name: "_Exit", scope: !16, file: !16, line: 768, type: !84, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!180 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !181, file: !23, line: 214)
!181 = !DISubprogram(name: "llabs", scope: !16, file: !16, line: 984, type: !182, flags: DIFlagPrototyped, spFlags: 0)
!182 = !DISubroutineType(types: !183)
!183 = !{!176, !176}
!184 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !185, file: !23, line: 220)
!185 = !DISubprogram(name: "lldiv", scope: !16, file: !16, line: 998, type: !186, flags: DIFlagPrototyped, spFlags: 0)
!186 = !DISubroutineType(types: !187)
!187 = !{!172, !176, !176}
!188 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !189, file: !23, line: 231)
!189 = !DISubprogram(name: "atoll", scope: !16, file: !16, line: 113, type: !190, flags: DIFlagPrototyped, spFlags: 0)
!190 = !DISubroutineType(types: !191)
!191 = !{!176, !55}
!192 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !193, file: !23, line: 232)
!193 = !DISubprogram(name: "strtoll", linkageName: "__isoc23_strtoll", scope: !16, file: !16, line: 238, type: !194, flags: DIFlagPrototyped, spFlags: 0)
!194 = !DISubroutineType(types: !195)
!195 = !{!176, !118, !146, !9}
!196 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !197, file: !23, line: 233)
!197 = !DISubprogram(name: "strtoull", linkageName: "__isoc23_strtoull", scope: !16, file: !16, line: 243, type: !198, flags: DIFlagPrototyped, spFlags: 0)
!198 = !DISubroutineType(types: !199)
!199 = !{!200, !118, !146, !9}
!200 = !DIBasicType(name: "unsigned long long", size: 64, encoding: DW_ATE_unsigned)
!201 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !202, file: !23, line: 235)
!202 = !DISubprogram(name: "strtof", scope: !16, file: !16, line: 124, type: !203, flags: DIFlagPrototyped, spFlags: 0)
!203 = !DISubroutineType(types: !204)
!204 = !{!205, !118, !146}
!205 = !DIBasicType(name: "float", size: 32, encoding: DW_ATE_float)
!206 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !171, entity: !207, file: !23, line: 236)
!207 = !DISubprogram(name: "strtold", scope: !16, file: !16, line: 127, type: !208, flags: DIFlagPrototyped, spFlags: 0)
!208 = !DISubroutineType(types: !209)
!209 = !{!210, !118, !146}
!210 = !DIBasicType(name: "long double", size: 128, encoding: DW_ATE_float)
!211 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !172, file: !23, line: 244)
!212 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !179, file: !23, line: 246)
!213 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !181, file: !23, line: 248)
!214 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !215, file: !23, line: 249)
!215 = !DISubprogram(name: "div", linkageName: "_ZN9__gnu_cxx3divExx", scope: !171, file: !23, line: 217, type: !186, flags: DIFlagPrototyped, spFlags: 0)
!216 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !185, file: !23, line: 250)
!217 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !189, file: !23, line: 252)
!218 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !202, file: !23, line: 253)
!219 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !193, file: !23, line: 254)
!220 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !197, file: !23, line: 255)
!221 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !14, entity: !207, file: !23, line: 256)
!222 = !{i32 7, !"Dwarf Version", i32 5}
!223 = !{i32 2, !"Debug Info Version", i32 3}
!224 = !{i32 1, !"wchar_size", i32 4}
!225 = !{i32 8, !"PIC Level", i32 2}
!226 = !{i32 7, !"PIE Level", i32 2}
!227 = !{i32 7, !"uwtable", i32 2}
!228 = !{i32 7, !"frame-pointer", i32 1}
!229 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!230 = distinct !DISubprogram(name: "vec_create", linkageName: "_Z10vec_createi", scope: !5, file: !5, line: 12, type: !231, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!231 = !DISubroutineType(types: !232)
!232 = !{!3, !9}
!233 = !{}
!234 = !DILocalVariable(name: "cap", arg: 1, scope: !230, file: !5, line: 12, type: !9)
!235 = !DILocation(line: 12, column: 30, scope: !230)
!236 = !DILocalVariable(name: "v", scope: !230, file: !5, line: 13, type: !3)
!237 = !DILocation(line: 13, column: 19, scope: !230)
!238 = !DILocation(line: 13, column: 38, scope: !230)
!239 = !DILocation(line: 14, column: 10, scope: !240)
!240 = distinct !DILexicalBlock(scope: !230, file: !5, line: 14, column: 9)
!241 = !DILocation(line: 14, column: 9, scope: !230)
!242 = !DILocation(line: 14, column: 13, scope: !240)
!243 = !DILocation(line: 15, column: 28, scope: !230)
!244 = !DILocation(line: 15, column: 32, scope: !230)
!245 = !DILocation(line: 15, column: 21, scope: !230)
!246 = !DILocation(line: 15, column: 5, scope: !230)
!247 = !DILocation(line: 15, column: 8, scope: !230)
!248 = !DILocation(line: 15, column: 13, scope: !230)
!249 = !DILocation(line: 16, column: 5, scope: !230)
!250 = !DILocation(line: 16, column: 8, scope: !230)
!251 = !DILocation(line: 16, column: 13, scope: !230)
!252 = !DILocation(line: 17, column: 19, scope: !230)
!253 = !DILocation(line: 17, column: 5, scope: !230)
!254 = !DILocation(line: 17, column: 8, scope: !230)
!255 = !DILocation(line: 17, column: 17, scope: !230)
!256 = !DILocation(line: 18, column: 12, scope: !230)
!257 = !DILocation(line: 18, column: 5, scope: !230)
!258 = !DILocation(line: 19, column: 1, scope: !230)
!259 = distinct !DISubprogram(name: "vec_push", linkageName: "_Z8vec_pushP12SimpleVectori", scope: !5, file: !5, line: 21, type: !260, scopeLine: 21, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!260 = !DISubroutineType(types: !261)
!261 = !{null, !3, !9}
!262 = !DILocalVariable(name: "v", arg: 1, scope: !259, file: !5, line: 21, type: !3)
!263 = !DILocation(line: 21, column: 29, scope: !259)
!264 = !DILocalVariable(name: "value", arg: 2, scope: !259, file: !5, line: 21, type: !9)
!265 = !DILocation(line: 21, column: 36, scope: !259)
!266 = !DILocation(line: 22, column: 9, scope: !267)
!267 = distinct !DILexicalBlock(scope: !259, file: !5, line: 22, column: 9)
!268 = !DILocation(line: 22, column: 12, scope: !267)
!269 = !DILocation(line: 22, column: 19, scope: !267)
!270 = !DILocation(line: 22, column: 22, scope: !267)
!271 = !DILocation(line: 22, column: 17, scope: !267)
!272 = !DILocation(line: 22, column: 9, scope: !259)
!273 = !DILocation(line: 23, column: 28, scope: !274)
!274 = distinct !DILexicalBlock(scope: !267, file: !5, line: 22, column: 32)
!275 = !DILocation(line: 23, column: 9, scope: !274)
!276 = !DILocation(line: 23, column: 12, scope: !274)
!277 = !DILocation(line: 23, column: 17, scope: !274)
!278 = !DILocation(line: 23, column: 20, scope: !274)
!279 = !DILocation(line: 23, column: 26, scope: !274)
!280 = !DILocation(line: 24, column: 19, scope: !274)
!281 = !DILocation(line: 24, column: 22, scope: !274)
!282 = !DILocation(line: 24, column: 27, scope: !274)
!283 = !DILocation(line: 24, column: 9, scope: !274)
!284 = !DILocation(line: 24, column: 12, scope: !274)
!285 = !DILocation(line: 24, column: 17, scope: !274)
!286 = !DILocation(line: 25, column: 5, scope: !274)
!287 = !DILocation(line: 27, column: 1, scope: !259)
!288 = distinct !DISubprogram(name: "vec_get", linkageName: "_Z7vec_getP12SimpleVectori", scope: !5, file: !5, line: 29, type: !289, scopeLine: 29, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!289 = !DISubroutineType(types: !290)
!290 = !{!9, !3, !9}
!291 = !DILocalVariable(name: "v", arg: 1, scope: !288, file: !5, line: 29, type: !3)
!292 = !DILocation(line: 29, column: 27, scope: !288)
!293 = !DILocalVariable(name: "index", arg: 2, scope: !288, file: !5, line: 29, type: !9)
!294 = !DILocation(line: 29, column: 34, scope: !288)
!295 = !DILocation(line: 31, column: 12, scope: !288)
!296 = !DILocation(line: 31, column: 15, scope: !288)
!297 = !DILocation(line: 31, column: 20, scope: !288)
!298 = !DILocation(line: 31, column: 5, scope: !288)
!299 = distinct !DISubprogram(name: "vec_get_safe", linkageName: "_Z12vec_get_safeP12SimpleVectori", scope: !5, file: !5, line: 34, type: !289, scopeLine: 34, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!300 = !DILocalVariable(name: "v", arg: 1, scope: !299, file: !5, line: 34, type: !3)
!301 = !DILocation(line: 34, column: 32, scope: !299)
!302 = !DILocalVariable(name: "index", arg: 2, scope: !299, file: !5, line: 34, type: !9)
!303 = !DILocation(line: 34, column: 39, scope: !299)
!304 = !DILocation(line: 35, column: 9, scope: !305)
!305 = distinct !DILexicalBlock(scope: !299, file: !5, line: 35, column: 9)
!306 = !DILocation(line: 35, column: 15, scope: !305)
!307 = !DILocation(line: 35, column: 20, scope: !305)
!308 = !DILocation(line: 35, column: 23, scope: !305)
!309 = !DILocation(line: 35, column: 31, scope: !305)
!310 = !DILocation(line: 35, column: 34, scope: !305)
!311 = !DILocation(line: 35, column: 29, scope: !305)
!312 = !DILocation(line: 35, column: 9, scope: !299)
!313 = !DILocation(line: 36, column: 16, scope: !314)
!314 = distinct !DILexicalBlock(scope: !305, file: !5, line: 35, column: 40)
!315 = !DILocation(line: 36, column: 19, scope: !314)
!316 = !DILocation(line: 36, column: 24, scope: !314)
!317 = !DILocation(line: 36, column: 9, scope: !314)
!318 = !DILocation(line: 38, column: 5, scope: !299)
!319 = !DILocation(line: 39, column: 1, scope: !299)
!320 = distinct !DISubprogram(name: "vec_free", linkageName: "_Z8vec_freeP12SimpleVector", scope: !5, file: !5, line: 41, type: !321, scopeLine: 41, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!321 = !DISubroutineType(types: !322)
!322 = !{null, !3}
!323 = !DILocalVariable(name: "v", arg: 1, scope: !320, file: !5, line: 41, type: !3)
!324 = !DILocation(line: 41, column: 29, scope: !320)
!325 = !DILocation(line: 42, column: 9, scope: !326)
!326 = distinct !DILexicalBlock(scope: !320, file: !5, line: 42, column: 9)
!327 = !DILocation(line: 42, column: 9, scope: !320)
!328 = !DILocation(line: 43, column: 14, scope: !329)
!329 = distinct !DILexicalBlock(scope: !326, file: !5, line: 42, column: 12)
!330 = !DILocation(line: 43, column: 17, scope: !329)
!331 = !DILocation(line: 43, column: 9, scope: !329)
!332 = !DILocation(line: 44, column: 14, scope: !329)
!333 = !DILocation(line: 44, column: 9, scope: !329)
!334 = !DILocation(line: 45, column: 5, scope: !329)
!335 = !DILocation(line: 46, column: 1, scope: !320)
!336 = distinct !DISubprogram(name: "main", scope: !5, file: !5, line: 48, type: !131, scopeLine: 48, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !233)
!337 = !DILocalVariable(name: "v", scope: !336, file: !5, line: 49, type: !3)
!338 = !DILocation(line: 49, column: 19, scope: !336)
!339 = !DILocation(line: 49, column: 23, scope: !336)
!340 = !DILocation(line: 50, column: 10, scope: !341)
!341 = distinct !DILexicalBlock(scope: !336, file: !5, line: 50, column: 9)
!342 = !DILocation(line: 50, column: 9, scope: !336)
!343 = !DILocation(line: 50, column: 13, scope: !341)
!344 = !DILocalVariable(name: "i", scope: !345, file: !5, line: 52, type: !9)
!345 = distinct !DILexicalBlock(scope: !336, file: !5, line: 52, column: 5)
!346 = !DILocation(line: 52, column: 14, scope: !345)
!347 = !DILocation(line: 52, column: 10, scope: !345)
!348 = !DILocation(line: 52, column: 21, scope: !349)
!349 = distinct !DILexicalBlock(scope: !345, file: !5, line: 52, column: 5)
!350 = !DILocation(line: 52, column: 23, scope: !349)
!351 = !DILocation(line: 52, column: 5, scope: !345)
!352 = !DILocation(line: 53, column: 18, scope: !353)
!353 = distinct !DILexicalBlock(scope: !349, file: !5, line: 52, column: 34)
!354 = !DILocation(line: 53, column: 21, scope: !353)
!355 = !DILocation(line: 53, column: 23, scope: !353)
!356 = !DILocation(line: 53, column: 9, scope: !353)
!357 = !DILocation(line: 54, column: 5, scope: !353)
!358 = !DILocation(line: 52, column: 30, scope: !349)
!359 = !DILocation(line: 52, column: 5, scope: !349)
!360 = distinct !{!360, !351, !361, !362}
!361 = !DILocation(line: 54, column: 5, scope: !345)
!362 = !{!"llvm.loop.mustprogress"}
!363 = !DILocalVariable(name: "val1", scope: !336, file: !5, line: 56, type: !9)
!364 = !DILocation(line: 56, column: 9, scope: !336)
!365 = !DILocation(line: 56, column: 29, scope: !336)
!366 = !DILocation(line: 56, column: 16, scope: !336)
!367 = !DILocalVariable(name: "val2", scope: !336, file: !5, line: 57, type: !9)
!368 = !DILocation(line: 57, column: 9, scope: !336)
!369 = !DILocation(line: 57, column: 24, scope: !336)
!370 = !DILocation(line: 57, column: 16, scope: !336)
!371 = !DILocation(line: 59, column: 14, scope: !336)
!372 = !DILocation(line: 59, column: 5, scope: !336)
!373 = !DILocation(line: 60, column: 12, scope: !336)
!374 = !DILocation(line: 60, column: 19, scope: !336)
!375 = !DILocation(line: 60, column: 17, scope: !336)
!376 = !DILocation(line: 60, column: 5, scope: !336)
!377 = !DILocation(line: 61, column: 1, scope: !336)
