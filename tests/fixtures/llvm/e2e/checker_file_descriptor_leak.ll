; ModuleID = '/tmp/raw.ll'
source_filename = "tests/programs/c/checker_file_descriptor_leak.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"r\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1, !dbg !7

; Function Attrs: noinline nounwind uwtable
define dso_local void @process(ptr noundef %0) #0 !dbg !22 {
  %2 = alloca [256 x i8], align 1
  tail call void @llvm.dbg.value(metadata ptr %0, metadata !28, metadata !DIExpression()), !dbg !29
  %3 = call noalias ptr @fopen(ptr noundef %0, ptr noundef @.str), !dbg !30
  tail call void @llvm.dbg.value(metadata ptr %3, metadata !31, metadata !DIExpression()), !dbg !29
  %4 = icmp ne ptr %3, null, !dbg !94
  br i1 %4, label %6, label %5, !dbg !96

5:                                                ; preds = %1
  br label %9, !dbg !97

6:                                                ; preds = %1
  call void @llvm.dbg.declare(metadata ptr %2, metadata !98, metadata !DIExpression()), !dbg !102
  %7 = getelementptr inbounds [256 x i8], ptr %2, i64 0, i64 0, !dbg !103
  %8 = call ptr @fgets(ptr noundef %7, i32 noundef 256, ptr noundef %3), !dbg !104
  br label %9, !dbg !105

9:                                                ; preds = %6, %5
  ret void, !dbg !105
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare noalias ptr @fopen(ptr noundef, ptr noundef) #2

declare ptr @fgets(ptr noundef, i32 noundef, ptr noundef) #2

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !106 {
  call void @process(ptr noundef @.str.1), !dbg !109
  ret i32 0, !dbg !110
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!14, !15, !16, !17, !18, !19, !20}
!llvm.ident = !{!21}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 4, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/checker_file_descriptor_leak.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "f1f09360c369c4fe603d5d64ddf7a40b")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 16, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 2)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 14, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 72, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 9)
!12 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !13, splitDebugInlining: false, nameTableKind: None)
!13 = !{!0, !7}
!14 = !{i32 7, !"Dwarf Version", i32 5}
!15 = !{i32 2, !"Debug Info Version", i32 3}
!16 = !{i32 1, !"wchar_size", i32 4}
!17 = !{i32 8, !"PIC Level", i32 2}
!18 = !{i32 7, !"PIE Level", i32 2}
!19 = !{i32 7, !"uwtable", i32 2}
!20 = !{i32 7, !"frame-pointer", i32 1}
!21 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!22 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 3, type: !23, scopeLine: 3, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !27)
!23 = !DISubroutineType(types: !24)
!24 = !{null, !25}
!25 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !26, size: 64)
!26 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!27 = !{}
!28 = !DILocalVariable(name: "path", arg: 1, scope: !22, file: !2, line: 3, type: !25)
!29 = !DILocation(line: 0, scope: !22)
!30 = !DILocation(line: 4, column: 15, scope: !22)
!31 = !DILocalVariable(name: "f", scope: !22, file: !2, line: 4, type: !32)
!32 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !33, size: 64)
!33 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !34, line: 7, baseType: !35)
!34 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!35 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !36, line: 49, size: 1728, elements: !37)
!36 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!37 = !{!38, !40, !42, !43, !44, !45, !46, !47, !48, !49, !50, !51, !52, !55, !57, !58, !59, !63, !65, !67, !71, !74, !76, !79, !82, !83, !85, !89, !90}
!38 = !DIDerivedType(tag: DW_TAG_member, name: "_flags", scope: !35, file: !36, line: 51, baseType: !39, size: 32)
!39 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!40 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_ptr", scope: !35, file: !36, line: 54, baseType: !41, size: 64, offset: 64)
!41 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!42 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_end", scope: !35, file: !36, line: 55, baseType: !41, size: 64, offset: 128)
!43 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_base", scope: !35, file: !36, line: 56, baseType: !41, size: 64, offset: 192)
!44 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_base", scope: !35, file: !36, line: 57, baseType: !41, size: 64, offset: 256)
!45 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_ptr", scope: !35, file: !36, line: 58, baseType: !41, size: 64, offset: 320)
!46 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_end", scope: !35, file: !36, line: 59, baseType: !41, size: 64, offset: 384)
!47 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_base", scope: !35, file: !36, line: 60, baseType: !41, size: 64, offset: 448)
!48 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_end", scope: !35, file: !36, line: 61, baseType: !41, size: 64, offset: 512)
!49 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_base", scope: !35, file: !36, line: 64, baseType: !41, size: 64, offset: 576)
!50 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_backup_base", scope: !35, file: !36, line: 65, baseType: !41, size: 64, offset: 640)
!51 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_end", scope: !35, file: !36, line: 66, baseType: !41, size: 64, offset: 704)
!52 = !DIDerivedType(tag: DW_TAG_member, name: "_markers", scope: !35, file: !36, line: 68, baseType: !53, size: 64, offset: 768)
!53 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !54, size: 64)
!54 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_marker", file: !36, line: 36, flags: DIFlagFwdDecl)
!55 = !DIDerivedType(tag: DW_TAG_member, name: "_chain", scope: !35, file: !36, line: 70, baseType: !56, size: 64, offset: 832)
!56 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !35, size: 64)
!57 = !DIDerivedType(tag: DW_TAG_member, name: "_fileno", scope: !35, file: !36, line: 72, baseType: !39, size: 32, offset: 896)
!58 = !DIDerivedType(tag: DW_TAG_member, name: "_flags2", scope: !35, file: !36, line: 73, baseType: !39, size: 32, offset: 928)
!59 = !DIDerivedType(tag: DW_TAG_member, name: "_old_offset", scope: !35, file: !36, line: 74, baseType: !60, size: 64, offset: 960)
!60 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off_t", file: !61, line: 152, baseType: !62)
!61 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types.h", directory: "", checksumkind: CSK_MD5, checksum: "e1865d9fe29fe1b5ced550b7ba458f9e")
!62 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!63 = !DIDerivedType(tag: DW_TAG_member, name: "_cur_column", scope: !35, file: !36, line: 77, baseType: !64, size: 16, offset: 1024)
!64 = !DIBasicType(name: "unsigned short", size: 16, encoding: DW_ATE_unsigned)
!65 = !DIDerivedType(tag: DW_TAG_member, name: "_vtable_offset", scope: !35, file: !36, line: 78, baseType: !66, size: 8, offset: 1040)
!66 = !DIBasicType(name: "signed char", size: 8, encoding: DW_ATE_signed_char)
!67 = !DIDerivedType(tag: DW_TAG_member, name: "_shortbuf", scope: !35, file: !36, line: 79, baseType: !68, size: 8, offset: 1048)
!68 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 8, elements: !69)
!69 = !{!70}
!70 = !DISubrange(count: 1)
!71 = !DIDerivedType(tag: DW_TAG_member, name: "_lock", scope: !35, file: !36, line: 81, baseType: !72, size: 64, offset: 1088)
!72 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !73, size: 64)
!73 = !DIDerivedType(tag: DW_TAG_typedef, name: "_IO_lock_t", file: !36, line: 43, baseType: null)
!74 = !DIDerivedType(tag: DW_TAG_member, name: "_offset", scope: !35, file: !36, line: 89, baseType: !75, size: 64, offset: 1152)
!75 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off64_t", file: !61, line: 153, baseType: !62)
!76 = !DIDerivedType(tag: DW_TAG_member, name: "_codecvt", scope: !35, file: !36, line: 91, baseType: !77, size: 64, offset: 1216)
!77 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !78, size: 64)
!78 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_codecvt", file: !36, line: 37, flags: DIFlagFwdDecl)
!79 = !DIDerivedType(tag: DW_TAG_member, name: "_wide_data", scope: !35, file: !36, line: 92, baseType: !80, size: 64, offset: 1280)
!80 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !81, size: 64)
!81 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_wide_data", file: !36, line: 38, flags: DIFlagFwdDecl)
!82 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_list", scope: !35, file: !36, line: 93, baseType: !56, size: 64, offset: 1344)
!83 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_buf", scope: !35, file: !36, line: 94, baseType: !84, size: 64, offset: 1408)
!84 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!85 = !DIDerivedType(tag: DW_TAG_member, name: "__pad5", scope: !35, file: !36, line: 95, baseType: !86, size: 64, offset: 1472)
!86 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !87, line: 18, baseType: !88)
!87 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!88 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!89 = !DIDerivedType(tag: DW_TAG_member, name: "_mode", scope: !35, file: !36, line: 96, baseType: !39, size: 32, offset: 1536)
!90 = !DIDerivedType(tag: DW_TAG_member, name: "_unused2", scope: !35, file: !36, line: 98, baseType: !91, size: 160, offset: 1568)
!91 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 160, elements: !92)
!92 = !{!93}
!93 = !DISubrange(count: 20)
!94 = !DILocation(line: 5, column: 10, scope: !95)
!95 = distinct !DILexicalBlock(scope: !22, file: !2, line: 5, column: 9)
!96 = !DILocation(line: 5, column: 9, scope: !22)
!97 = !DILocation(line: 5, column: 13, scope: !95)
!98 = !DILocalVariable(name: "buf", scope: !22, file: !2, line: 7, type: !99)
!99 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 2048, elements: !100)
!100 = !{!101}
!101 = !DISubrange(count: 256)
!102 = !DILocation(line: 7, column: 10, scope: !22)
!103 = !DILocation(line: 8, column: 11, scope: !22)
!104 = !DILocation(line: 8, column: 5, scope: !22)
!105 = !DILocation(line: 11, column: 1, scope: !22)
!106 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 13, type: !107, scopeLine: 13, spFlags: DISPFlagDefinition, unit: !12)
!107 = !DISubroutineType(types: !108)
!108 = !{!39}
!109 = !DILocation(line: 14, column: 5, scope: !106)
!110 = !DILocation(line: 15, column: 5, scope: !106)
