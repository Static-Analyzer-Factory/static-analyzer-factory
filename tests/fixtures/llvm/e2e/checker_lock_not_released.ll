; ModuleID = '/tmp/checker_lock_not_released_raw.ll'
source_filename = "tests/programs/c/checker_lock_not_released.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%union.pthread_mutex_t = type { %struct.__pthread_mutex_s, [8 x i8] }
%struct.__pthread_mutex_s = type { i32, i32, i32, i32, i32, i32, %struct.__pthread_internal_list }
%struct.__pthread_internal_list = type { ptr, ptr }

@shared_data = dso_local global i32 0, align 4, !dbg !0
@mtx = dso_local global %union.pthread_mutex_t zeroinitializer, align 8, !dbg !5

; Function Attrs: noinline nounwind uwtable
define dso_local void @process() #0 !dbg !46 {
  %1 = call i32 @pthread_mutex_lock(ptr noundef @mtx) #2, !dbg !49
  %2 = load i32, ptr @shared_data, align 4, !dbg !50
  %3 = add nsw i32 %2, 1, !dbg !50
  store i32 %3, ptr @shared_data, align 4, !dbg !50
  ret void, !dbg !51
}

; Function Attrs: nounwind
declare i32 @pthread_mutex_lock(ptr noundef) #1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !52 {
  %1 = call i32 @pthread_mutex_init(ptr noundef @mtx, ptr noundef null) #2, !dbg !55
  call void @process(), !dbg !56
  %2 = call i32 @pthread_mutex_destroy(ptr noundef @mtx) #2, !dbg !57
  ret i32 0, !dbg !58
}

; Function Attrs: nounwind
declare i32 @pthread_mutex_init(ptr noundef, ptr noundef) #1

; Function Attrs: nounwind
declare i32 @pthread_mutex_destroy(ptr noundef) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { nounwind }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!38, !39, !40, !41, !42, !43, !44}
!llvm.ident = !{!45}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "shared_data", scope: !2, file: !3, line: 4, type: !16, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "tests/programs/c/checker_lock_not_released.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "33d3bcbea699f773b2c8f14e8fa867f0")
!4 = !{!0, !5}
!5 = !DIGlobalVariableExpression(var: !6, expr: !DIExpression())
!6 = distinct !DIGlobalVariable(name: "mtx", scope: !2, file: !3, line: 3, type: !7, isLocal: false, isDefinition: true)
!7 = !DIDerivedType(tag: DW_TAG_typedef, name: "pthread_mutex_t", file: !8, line: 72, baseType: !9)
!8 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/pthreadtypes.h", directory: "", checksumkind: CSK_MD5, checksum: "8a5acdbeec491eca11cf81cb1ef77ea7")
!9 = distinct !DICompositeType(tag: DW_TAG_union_type, file: !8, line: 67, size: 384, elements: !10)
!10 = !{!11, !31, !36}
!11 = !DIDerivedType(tag: DW_TAG_member, name: "__data", scope: !9, file: !8, line: 69, baseType: !12, size: 320)
!12 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "__pthread_mutex_s", file: !13, line: 27, size: 320, elements: !14)
!13 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/struct_mutex.h", directory: "", checksumkind: CSK_MD5, checksum: "7621bb083e29284124f6172bbd38533d")
!14 = !{!15, !17, !19, !20, !21, !22, !23}
!15 = !DIDerivedType(tag: DW_TAG_member, name: "__lock", scope: !12, file: !13, line: 29, baseType: !16, size: 32)
!16 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!17 = !DIDerivedType(tag: DW_TAG_member, name: "__count", scope: !12, file: !13, line: 30, baseType: !18, size: 32, offset: 32)
!18 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!19 = !DIDerivedType(tag: DW_TAG_member, name: "__owner", scope: !12, file: !13, line: 31, baseType: !16, size: 32, offset: 64)
!20 = !DIDerivedType(tag: DW_TAG_member, name: "__nusers", scope: !12, file: !13, line: 33, baseType: !18, size: 32, offset: 96)
!21 = !DIDerivedType(tag: DW_TAG_member, name: "__kind", scope: !12, file: !13, line: 58, baseType: !16, size: 32, offset: 128)
!22 = !DIDerivedType(tag: DW_TAG_member, name: "__spins", scope: !12, file: !13, line: 63, baseType: !16, size: 32, offset: 160)
!23 = !DIDerivedType(tag: DW_TAG_member, name: "__list", scope: !12, file: !13, line: 64, baseType: !24, size: 128, offset: 192)
!24 = !DIDerivedType(tag: DW_TAG_typedef, name: "__pthread_list_t", file: !25, line: 55, baseType: !26)
!25 = !DIFile(filename: "/usr/include/aarch64-linux-gnu/bits/thread-shared-types.h", directory: "", checksumkind: CSK_MD5, checksum: "b9a7199822bce372686baacd32a9f4f3")
!26 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "__pthread_internal_list", file: !25, line: 51, size: 128, elements: !27)
!27 = !{!28, !30}
!28 = !DIDerivedType(tag: DW_TAG_member, name: "__prev", scope: !26, file: !25, line: 53, baseType: !29, size: 64)
!29 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !26, size: 64)
!30 = !DIDerivedType(tag: DW_TAG_member, name: "__next", scope: !26, file: !25, line: 54, baseType: !29, size: 64, offset: 64)
!31 = !DIDerivedType(tag: DW_TAG_member, name: "__size", scope: !9, file: !8, line: 70, baseType: !32, size: 384)
!32 = !DICompositeType(tag: DW_TAG_array_type, baseType: !33, size: 384, elements: !34)
!33 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!34 = !{!35}
!35 = !DISubrange(count: 48)
!36 = !DIDerivedType(tag: DW_TAG_member, name: "__align", scope: !9, file: !8, line: 71, baseType: !37, size: 64)
!37 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!38 = !{i32 7, !"Dwarf Version", i32 5}
!39 = !{i32 2, !"Debug Info Version", i32 3}
!40 = !{i32 1, !"wchar_size", i32 4}
!41 = !{i32 8, !"PIC Level", i32 2}
!42 = !{i32 7, !"PIE Level", i32 2}
!43 = !{i32 7, !"uwtable", i32 2}
!44 = !{i32 7, !"frame-pointer", i32 1}
!45 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!46 = distinct !DISubprogram(name: "process", scope: !3, file: !3, line: 6, type: !47, scopeLine: 6, spFlags: DISPFlagDefinition, unit: !2)
!47 = !DISubroutineType(types: !48)
!48 = !{null}
!49 = !DILocation(line: 7, column: 5, scope: !46)
!50 = !DILocation(line: 8, column: 16, scope: !46)
!51 = !DILocation(line: 10, column: 1, scope: !46)
!52 = distinct !DISubprogram(name: "main", scope: !3, file: !3, line: 12, type: !53, scopeLine: 12, spFlags: DISPFlagDefinition, unit: !2)
!53 = !DISubroutineType(types: !54)
!54 = !{!16}
!55 = !DILocation(line: 13, column: 5, scope: !52)
!56 = !DILocation(line: 14, column: 5, scope: !52)
!57 = !DILocation(line: 15, column: 5, scope: !52)
!58 = !DILocation(line: 16, column: 5, scope: !52)
