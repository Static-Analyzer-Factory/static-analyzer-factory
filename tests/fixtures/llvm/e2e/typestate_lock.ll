; ModuleID = '/workspace/tests/programs/cpp/typestate_lock.cpp'
source_filename = "/workspace/tests/programs/cpp/typestate_lock.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%union.pthread_mutex_t = type { %struct.__pthread_mutex_s, [8 x i8] }
%struct.__pthread_mutex_s = type { i32, i32, i32, i32, i32, i32, %struct.__pthread_internal_list }
%struct.__pthread_internal_list = type { ptr, ptr }

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define dso_local void @_Z14lock_no_unlockv() #0 !dbg !10 {
  %1 = alloca %union.pthread_mutex_t, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !15, metadata !DIExpression()), !dbg !47
  %2 = call i32 @pthread_mutex_init(ptr noundef %1, ptr noundef null) #3, !dbg !48
  %3 = call i32 @pthread_mutex_lock(ptr noundef %1) #3, !dbg !49
  ret void, !dbg !50
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare i32 @pthread_mutex_init(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind
declare i32 @pthread_mutex_lock(ptr noundef) #2

attributes #0 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/cpp/typestate_lock.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "030d4d06f65b14c29839fce5be9fb843")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "lock_no_unlock", linkageName: "_Z14lock_no_unlockv", scope: !11, file: !11, line: 4, type: !12, scopeLine: 4, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DIFile(filename: "cpp/typestate_lock.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "030d4d06f65b14c29839fce5be9fb843")
!12 = !DISubroutineType(types: !13)
!13 = !{null}
!14 = !{}
!15 = !DILocalVariable(name: "mutex", scope: !10, file: !11, line: 5, type: !16)
!16 = !DIDerivedType(tag: DW_TAG_typedef, name: "pthread_mutex_t", file: !17, line: 72, baseType: !18)
!17 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/pthreadtypes.h", directory: "", checksumkind: CSK_MD5, checksum: "8a5acdbeec491eca11cf81cb1ef77ea7")
!18 = distinct !DICompositeType(tag: DW_TAG_union_type, file: !17, line: 67, size: 384, flags: DIFlagTypePassByValue, elements: !19, identifier: "_ZTS15pthread_mutex_t")
!19 = !{!20, !40, !45}
!20 = !DIDerivedType(tag: DW_TAG_member, name: "__data", scope: !18, file: !17, line: 69, baseType: !21, size: 320)
!21 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "__pthread_mutex_s", file: !22, line: 27, size: 320, flags: DIFlagTypePassByValue, elements: !23, identifier: "_ZTS17__pthread_mutex_s")
!22 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/struct_mutex.h", directory: "", checksumkind: CSK_MD5, checksum: "7621bb083e29284124f6172bbd38533d")
!23 = !{!24, !26, !28, !29, !30, !31, !32}
!24 = !DIDerivedType(tag: DW_TAG_member, name: "__lock", scope: !21, file: !22, line: 29, baseType: !25, size: 32)
!25 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!26 = !DIDerivedType(tag: DW_TAG_member, name: "__count", scope: !21, file: !22, line: 30, baseType: !27, size: 32, offset: 32)
!27 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!28 = !DIDerivedType(tag: DW_TAG_member, name: "__owner", scope: !21, file: !22, line: 31, baseType: !25, size: 32, offset: 64)
!29 = !DIDerivedType(tag: DW_TAG_member, name: "__nusers", scope: !21, file: !22, line: 33, baseType: !27, size: 32, offset: 96)
!30 = !DIDerivedType(tag: DW_TAG_member, name: "__kind", scope: !21, file: !22, line: 58, baseType: !25, size: 32, offset: 128)
!31 = !DIDerivedType(tag: DW_TAG_member, name: "__spins", scope: !21, file: !22, line: 63, baseType: !25, size: 32, offset: 160)
!32 = !DIDerivedType(tag: DW_TAG_member, name: "__list", scope: !21, file: !22, line: 64, baseType: !33, size: 128, offset: 192)
!33 = !DIDerivedType(tag: DW_TAG_typedef, name: "__pthread_list_t", file: !34, line: 55, baseType: !35)
!34 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/thread-shared-types.h", directory: "", checksumkind: CSK_MD5, checksum: "b9a7199822bce372686baacd32a9f4f3")
!35 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "__pthread_internal_list", file: !34, line: 51, size: 128, flags: DIFlagTypePassByValue, elements: !36, identifier: "_ZTS23__pthread_internal_list")
!36 = !{!37, !39}
!37 = !DIDerivedType(tag: DW_TAG_member, name: "__prev", scope: !35, file: !34, line: 53, baseType: !38, size: 64)
!38 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !35, size: 64)
!39 = !DIDerivedType(tag: DW_TAG_member, name: "__next", scope: !35, file: !34, line: 54, baseType: !38, size: 64, offset: 64)
!40 = !DIDerivedType(tag: DW_TAG_member, name: "__size", scope: !18, file: !17, line: 70, baseType: !41, size: 384)
!41 = !DICompositeType(tag: DW_TAG_array_type, baseType: !42, size: 384, elements: !43)
!42 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!43 = !{!44}
!44 = !DISubrange(count: 48)
!45 = !DIDerivedType(tag: DW_TAG_member, name: "__align", scope: !18, file: !17, line: 71, baseType: !46, size: 64)
!46 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!47 = !DILocation(line: 5, column: 21, scope: !10)
!48 = !DILocation(line: 6, column: 5, scope: !10)
!49 = !DILocation(line: 7, column: 5, scope: !10)
!50 = !DILocation(line: 9, column: 1, scope: !10)
