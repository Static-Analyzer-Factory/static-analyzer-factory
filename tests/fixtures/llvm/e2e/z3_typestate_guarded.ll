; ModuleID = 'tests/programs/c/z3_typestate_guarded.c'
source_filename = "tests/programs/c/z3_typestate_guarded.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [2 x i8] c"r\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @process(i32 noundef %0) #0 !dbg !22 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %3, metadata !29, metadata !DIExpression()), !dbg !91
  store ptr null, ptr %3, align 8, !dbg !91
  %4 = load i32, ptr %2, align 4, !dbg !92
  %5 = icmp ne i32 %4, 0, !dbg !92
  br i1 %5, label %6, label %8, !dbg !94

6:                                                ; preds = %1
  %7 = call noalias ptr @fopen(ptr noundef @.str, ptr noundef @.str.1), !dbg !95
  store ptr %7, ptr %3, align 8, !dbg !97
  br label %8, !dbg !98

8:                                                ; preds = %6, %1
  %9 = load i32, ptr %2, align 4, !dbg !99
  %10 = icmp ne i32 %9, 0, !dbg !99
  br i1 %10, label %11, label %14, !dbg !101

11:                                               ; preds = %8
  %12 = load ptr, ptr %3, align 8, !dbg !102
  %13 = call i32 @fclose(ptr noundef %12), !dbg !104
  br label %14, !dbg !105

14:                                               ; preds = %11, %8
  ret void, !dbg !106
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare noalias ptr @fopen(ptr noundef, ptr noundef) #2

declare i32 @fclose(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !107 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @process(i32 noundef 1), !dbg !110
  call void @process(i32 noundef 0), !dbg !111
  ret i32 0, !dbg !112
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!14, !15, !16, !17, !18, !19, !20}
!llvm.ident = !{!21}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_typestate_guarded.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "8773cc28798ee98273e091813d42f5fa")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 72, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 9)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 16, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 2)
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
!22 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 5, type: !23, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !26)
!23 = !DISubroutineType(types: !24)
!24 = !{null, !25}
!25 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!26 = !{}
!27 = !DILocalVariable(name: "flag", arg: 1, scope: !22, file: !2, line: 5, type: !25)
!28 = !DILocation(line: 5, column: 18, scope: !22)
!29 = !DILocalVariable(name: "f", scope: !22, file: !2, line: 6, type: !30)
!30 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !31, size: 64)
!31 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !32, line: 7, baseType: !33)
!32 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!33 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !34, line: 49, size: 1728, elements: !35)
!34 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!35 = !{!36, !37, !39, !40, !41, !42, !43, !44, !45, !46, !47, !48, !49, !52, !54, !55, !56, !60, !62, !64, !68, !71, !73, !76, !79, !80, !82, !86, !87}
!36 = !DIDerivedType(tag: DW_TAG_member, name: "_flags", scope: !33, file: !34, line: 51, baseType: !25, size: 32)
!37 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_ptr", scope: !33, file: !34, line: 54, baseType: !38, size: 64, offset: 64)
!38 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!39 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_end", scope: !33, file: !34, line: 55, baseType: !38, size: 64, offset: 128)
!40 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_base", scope: !33, file: !34, line: 56, baseType: !38, size: 64, offset: 192)
!41 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_base", scope: !33, file: !34, line: 57, baseType: !38, size: 64, offset: 256)
!42 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_ptr", scope: !33, file: !34, line: 58, baseType: !38, size: 64, offset: 320)
!43 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_end", scope: !33, file: !34, line: 59, baseType: !38, size: 64, offset: 384)
!44 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_base", scope: !33, file: !34, line: 60, baseType: !38, size: 64, offset: 448)
!45 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_end", scope: !33, file: !34, line: 61, baseType: !38, size: 64, offset: 512)
!46 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_base", scope: !33, file: !34, line: 64, baseType: !38, size: 64, offset: 576)
!47 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_backup_base", scope: !33, file: !34, line: 65, baseType: !38, size: 64, offset: 640)
!48 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_end", scope: !33, file: !34, line: 66, baseType: !38, size: 64, offset: 704)
!49 = !DIDerivedType(tag: DW_TAG_member, name: "_markers", scope: !33, file: !34, line: 68, baseType: !50, size: 64, offset: 768)
!50 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !51, size: 64)
!51 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_marker", file: !34, line: 36, flags: DIFlagFwdDecl)
!52 = !DIDerivedType(tag: DW_TAG_member, name: "_chain", scope: !33, file: !34, line: 70, baseType: !53, size: 64, offset: 832)
!53 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !33, size: 64)
!54 = !DIDerivedType(tag: DW_TAG_member, name: "_fileno", scope: !33, file: !34, line: 72, baseType: !25, size: 32, offset: 896)
!55 = !DIDerivedType(tag: DW_TAG_member, name: "_flags2", scope: !33, file: !34, line: 73, baseType: !25, size: 32, offset: 928)
!56 = !DIDerivedType(tag: DW_TAG_member, name: "_old_offset", scope: !33, file: !34, line: 74, baseType: !57, size: 64, offset: 960)
!57 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off_t", file: !58, line: 152, baseType: !59)
!58 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types.h", directory: "", checksumkind: CSK_MD5, checksum: "e1865d9fe29fe1b5ced550b7ba458f9e")
!59 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!60 = !DIDerivedType(tag: DW_TAG_member, name: "_cur_column", scope: !33, file: !34, line: 77, baseType: !61, size: 16, offset: 1024)
!61 = !DIBasicType(name: "unsigned short", size: 16, encoding: DW_ATE_unsigned)
!62 = !DIDerivedType(tag: DW_TAG_member, name: "_vtable_offset", scope: !33, file: !34, line: 78, baseType: !63, size: 8, offset: 1040)
!63 = !DIBasicType(name: "signed char", size: 8, encoding: DW_ATE_signed_char)
!64 = !DIDerivedType(tag: DW_TAG_member, name: "_shortbuf", scope: !33, file: !34, line: 79, baseType: !65, size: 8, offset: 1048)
!65 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 8, elements: !66)
!66 = !{!67}
!67 = !DISubrange(count: 1)
!68 = !DIDerivedType(tag: DW_TAG_member, name: "_lock", scope: !33, file: !34, line: 81, baseType: !69, size: 64, offset: 1088)
!69 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !70, size: 64)
!70 = !DIDerivedType(tag: DW_TAG_typedef, name: "_IO_lock_t", file: !34, line: 43, baseType: null)
!71 = !DIDerivedType(tag: DW_TAG_member, name: "_offset", scope: !33, file: !34, line: 89, baseType: !72, size: 64, offset: 1152)
!72 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off64_t", file: !58, line: 153, baseType: !59)
!73 = !DIDerivedType(tag: DW_TAG_member, name: "_codecvt", scope: !33, file: !34, line: 91, baseType: !74, size: 64, offset: 1216)
!74 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !75, size: 64)
!75 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_codecvt", file: !34, line: 37, flags: DIFlagFwdDecl)
!76 = !DIDerivedType(tag: DW_TAG_member, name: "_wide_data", scope: !33, file: !34, line: 92, baseType: !77, size: 64, offset: 1280)
!77 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !78, size: 64)
!78 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_wide_data", file: !34, line: 38, flags: DIFlagFwdDecl)
!79 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_list", scope: !33, file: !34, line: 93, baseType: !53, size: 64, offset: 1344)
!80 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_buf", scope: !33, file: !34, line: 94, baseType: !81, size: 64, offset: 1408)
!81 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!82 = !DIDerivedType(tag: DW_TAG_member, name: "__pad5", scope: !33, file: !34, line: 95, baseType: !83, size: 64, offset: 1472)
!83 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !84, line: 18, baseType: !85)
!84 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!85 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!86 = !DIDerivedType(tag: DW_TAG_member, name: "_mode", scope: !33, file: !34, line: 96, baseType: !25, size: 32, offset: 1536)
!87 = !DIDerivedType(tag: DW_TAG_member, name: "_unused2", scope: !33, file: !34, line: 98, baseType: !88, size: 160, offset: 1568)
!88 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 160, elements: !89)
!89 = !{!90}
!90 = !DISubrange(count: 20)
!91 = !DILocation(line: 6, column: 11, scope: !22)
!92 = !DILocation(line: 7, column: 9, scope: !93)
!93 = distinct !DILexicalBlock(scope: !22, file: !2, line: 7, column: 9)
!94 = !DILocation(line: 7, column: 9, scope: !22)
!95 = !DILocation(line: 8, column: 13, scope: !96)
!96 = distinct !DILexicalBlock(scope: !93, file: !2, line: 7, column: 15)
!97 = !DILocation(line: 8, column: 11, scope: !96)
!98 = !DILocation(line: 9, column: 5, scope: !96)
!99 = !DILocation(line: 11, column: 9, scope: !100)
!100 = distinct !DILexicalBlock(scope: !22, file: !2, line: 11, column: 9)
!101 = !DILocation(line: 11, column: 9, scope: !22)
!102 = !DILocation(line: 13, column: 16, scope: !103)
!103 = distinct !DILexicalBlock(scope: !100, file: !2, line: 11, column: 15)
!104 = !DILocation(line: 13, column: 9, scope: !103)
!105 = !DILocation(line: 14, column: 5, scope: !103)
!106 = !DILocation(line: 17, column: 1, scope: !22)
!107 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 19, type: !108, scopeLine: 19, spFlags: DISPFlagDefinition, unit: !12)
!108 = !DISubroutineType(types: !109)
!109 = !{!25}
!110 = !DILocation(line: 20, column: 5, scope: !107)
!111 = !DILocation(line: 21, column: 5, scope: !107)
!112 = !DILocation(line: 22, column: 5, scope: !107)
