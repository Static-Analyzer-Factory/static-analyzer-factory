; ModuleID = '/workspace/tests/programs/c/typestate_correct.c'
source_filename = "/workspace/tests/programs/c/typestate_correct.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [2 x i8] c"r\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @correct_usage() #0 !dbg !25 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !29, metadata !DIExpression()), !dbg !91
  %2 = call noalias ptr @fopen(ptr noundef @.str, ptr noundef @.str.1), !dbg !92
  store ptr %2, ptr %1, align 8, !dbg !91
  %3 = load ptr, ptr %1, align 8, !dbg !93
  %4 = call i64 @fread(ptr noundef null, i64 noundef 1, i64 noundef 1, ptr noundef %3), !dbg !94
  %5 = load ptr, ptr %1, align 8, !dbg !95
  %6 = call i32 @fclose(ptr noundef %5), !dbg !96
  ret void, !dbg !97
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare noalias ptr @fopen(ptr noundef, ptr noundef) #2

declare i64 @fread(ptr noundef, i64 noundef, i64 noundef, ptr noundef) #2

declare i32 @fclose(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!17, !18, !19, !20, !21, !22, !23}
!llvm.ident = !{!24}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 5, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/typestate_correct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "f7bb0a2c792b1fc26d077ce0c0bd9135")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 72, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 9)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 5, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 16, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 2)
!12 = distinct !DICompileUnit(language: DW_LANG_C11, file: !13, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !14, globals: !16, splitDebugInlining: false, nameTableKind: None)
!13 = !DIFile(filename: "/workspace/tests/programs/c/typestate_correct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "f7bb0a2c792b1fc26d077ce0c0bd9135")
!14 = !{!15}
!15 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!16 = !{!0, !7}
!17 = !{i32 7, !"Dwarf Version", i32 5}
!18 = !{i32 2, !"Debug Info Version", i32 3}
!19 = !{i32 1, !"wchar_size", i32 4}
!20 = !{i32 8, !"PIC Level", i32 2}
!21 = !{i32 7, !"PIE Level", i32 2}
!22 = !{i32 7, !"uwtable", i32 2}
!23 = !{i32 7, !"frame-pointer", i32 1}
!24 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!25 = distinct !DISubprogram(name: "correct_usage", scope: !2, file: !2, line: 4, type: !26, scopeLine: 4, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !28)
!26 = !DISubroutineType(types: !27)
!27 = !{null}
!28 = !{}
!29 = !DILocalVariable(name: "fp", scope: !25, file: !2, line: 5, type: !30)
!30 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !31, size: 64)
!31 = !DIDerivedType(tag: DW_TAG_typedef, name: "FILE", file: !32, line: 7, baseType: !33)
!32 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "571f9fb6223c42439075fdde11a0de5d")
!33 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_FILE", file: !34, line: 49, size: 1728, elements: !35)
!34 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types/struct_FILE.h", directory: "", checksumkind: CSK_MD5, checksum: "7a6d4a00a37ee6b9a40cd04bd01f5d00")
!35 = !{!36, !38, !40, !41, !42, !43, !44, !45, !46, !47, !48, !49, !50, !53, !55, !56, !57, !61, !63, !65, !69, !72, !74, !77, !80, !81, !82, !86, !87}
!36 = !DIDerivedType(tag: DW_TAG_member, name: "_flags", scope: !33, file: !34, line: 51, baseType: !37, size: 32)
!37 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!38 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_ptr", scope: !33, file: !34, line: 54, baseType: !39, size: 64, offset: 64)
!39 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!40 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_end", scope: !33, file: !34, line: 55, baseType: !39, size: 64, offset: 128)
!41 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_read_base", scope: !33, file: !34, line: 56, baseType: !39, size: 64, offset: 192)
!42 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_base", scope: !33, file: !34, line: 57, baseType: !39, size: 64, offset: 256)
!43 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_ptr", scope: !33, file: !34, line: 58, baseType: !39, size: 64, offset: 320)
!44 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_write_end", scope: !33, file: !34, line: 59, baseType: !39, size: 64, offset: 384)
!45 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_base", scope: !33, file: !34, line: 60, baseType: !39, size: 64, offset: 448)
!46 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_buf_end", scope: !33, file: !34, line: 61, baseType: !39, size: 64, offset: 512)
!47 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_base", scope: !33, file: !34, line: 64, baseType: !39, size: 64, offset: 576)
!48 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_backup_base", scope: !33, file: !34, line: 65, baseType: !39, size: 64, offset: 640)
!49 = !DIDerivedType(tag: DW_TAG_member, name: "_IO_save_end", scope: !33, file: !34, line: 66, baseType: !39, size: 64, offset: 704)
!50 = !DIDerivedType(tag: DW_TAG_member, name: "_markers", scope: !33, file: !34, line: 68, baseType: !51, size: 64, offset: 768)
!51 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !52, size: 64)
!52 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_marker", file: !34, line: 36, flags: DIFlagFwdDecl)
!53 = !DIDerivedType(tag: DW_TAG_member, name: "_chain", scope: !33, file: !34, line: 70, baseType: !54, size: 64, offset: 832)
!54 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !33, size: 64)
!55 = !DIDerivedType(tag: DW_TAG_member, name: "_fileno", scope: !33, file: !34, line: 72, baseType: !37, size: 32, offset: 896)
!56 = !DIDerivedType(tag: DW_TAG_member, name: "_flags2", scope: !33, file: !34, line: 73, baseType: !37, size: 32, offset: 928)
!57 = !DIDerivedType(tag: DW_TAG_member, name: "_old_offset", scope: !33, file: !34, line: 74, baseType: !58, size: 64, offset: 960)
!58 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off_t", file: !59, line: 152, baseType: !60)
!59 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/types.h", directory: "", checksumkind: CSK_MD5, checksum: "e1865d9fe29fe1b5ced550b7ba458f9e")
!60 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!61 = !DIDerivedType(tag: DW_TAG_member, name: "_cur_column", scope: !33, file: !34, line: 77, baseType: !62, size: 16, offset: 1024)
!62 = !DIBasicType(name: "unsigned short", size: 16, encoding: DW_ATE_unsigned)
!63 = !DIDerivedType(tag: DW_TAG_member, name: "_vtable_offset", scope: !33, file: !34, line: 78, baseType: !64, size: 8, offset: 1040)
!64 = !DIBasicType(name: "signed char", size: 8, encoding: DW_ATE_signed_char)
!65 = !DIDerivedType(tag: DW_TAG_member, name: "_shortbuf", scope: !33, file: !34, line: 79, baseType: !66, size: 8, offset: 1048)
!66 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 8, elements: !67)
!67 = !{!68}
!68 = !DISubrange(count: 1)
!69 = !DIDerivedType(tag: DW_TAG_member, name: "_lock", scope: !33, file: !34, line: 81, baseType: !70, size: 64, offset: 1088)
!70 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !71, size: 64)
!71 = !DIDerivedType(tag: DW_TAG_typedef, name: "_IO_lock_t", file: !34, line: 43, baseType: null)
!72 = !DIDerivedType(tag: DW_TAG_member, name: "_offset", scope: !33, file: !34, line: 89, baseType: !73, size: 64, offset: 1152)
!73 = !DIDerivedType(tag: DW_TAG_typedef, name: "__off64_t", file: !59, line: 153, baseType: !60)
!74 = !DIDerivedType(tag: DW_TAG_member, name: "_codecvt", scope: !33, file: !34, line: 91, baseType: !75, size: 64, offset: 1216)
!75 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !76, size: 64)
!76 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_codecvt", file: !34, line: 37, flags: DIFlagFwdDecl)
!77 = !DIDerivedType(tag: DW_TAG_member, name: "_wide_data", scope: !33, file: !34, line: 92, baseType: !78, size: 64, offset: 1280)
!78 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !79, size: 64)
!79 = !DICompositeType(tag: DW_TAG_structure_type, name: "_IO_wide_data", file: !34, line: 38, flags: DIFlagFwdDecl)
!80 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_list", scope: !33, file: !34, line: 93, baseType: !54, size: 64, offset: 1344)
!81 = !DIDerivedType(tag: DW_TAG_member, name: "_freeres_buf", scope: !33, file: !34, line: 94, baseType: !15, size: 64, offset: 1408)
!82 = !DIDerivedType(tag: DW_TAG_member, name: "__pad5", scope: !33, file: !34, line: 95, baseType: !83, size: 64, offset: 1472)
!83 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !84, line: 18, baseType: !85)
!84 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!85 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!86 = !DIDerivedType(tag: DW_TAG_member, name: "_mode", scope: !33, file: !34, line: 96, baseType: !37, size: 32, offset: 1536)
!87 = !DIDerivedType(tag: DW_TAG_member, name: "_unused2", scope: !33, file: !34, line: 98, baseType: !88, size: 160, offset: 1568)
!88 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 160, elements: !89)
!89 = !{!90}
!90 = !DISubrange(count: 20)
!91 = !DILocation(line: 5, column: 11, scope: !25)
!92 = !DILocation(line: 5, column: 16, scope: !25)
!93 = !DILocation(line: 6, column: 23, scope: !25)
!94 = !DILocation(line: 6, column: 5, scope: !25)
!95 = !DILocation(line: 7, column: 12, scope: !25)
!96 = !DILocation(line: 7, column: 5, scope: !25)
!97 = !DILocation(line: 8, column: 1, scope: !25)
