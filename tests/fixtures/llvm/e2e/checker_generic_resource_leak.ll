; ModuleID = '/tmp/raw.ll'
source_filename = "tests/programs/c/checker_generic_resource_leak.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Resource = type { i32, ptr }

; Function Attrs: noinline nounwind uwtable
define dso_local ptr @acquire_resource() #0 !dbg !20 {
  %1 = call noalias ptr @malloc(i64 noundef 16) #4, !dbg !24
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !25, metadata !DIExpression()), !dbg !26
  %2 = getelementptr inbounds %struct.Resource, ptr %1, i32 0, i32 0, !dbg !27
  store i32 1, ptr %2, align 8, !dbg !28
  %3 = call noalias ptr @malloc(i64 noundef 64) #4, !dbg !29
  %4 = getelementptr inbounds %struct.Resource, ptr %1, i32 0, i32 1, !dbg !30
  store ptr %3, ptr %4, align 8, !dbg !31
  ret ptr %1, !dbg !32
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind uwtable
define dso_local void @release_resource(ptr noundef %0) #0 !dbg !33 {
  tail call void @llvm.dbg.value(metadata ptr %0, metadata !36, metadata !DIExpression()), !dbg !37
  %2 = getelementptr inbounds %struct.Resource, ptr %0, i32 0, i32 1, !dbg !38
  %3 = load ptr, ptr %2, align 8, !dbg !38
  call void @free(ptr noundef %3) #5, !dbg !39
  call void @free(ptr noundef %0) #5, !dbg !40
  ret void, !dbg !41
}

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind uwtable
define dso_local void @process() #0 !dbg !42 {
  %1 = call ptr @acquire_resource(), !dbg !45
  tail call void @llvm.dbg.value(metadata ptr %1, metadata !46, metadata !DIExpression()), !dbg !47
  %2 = getelementptr inbounds %struct.Resource, ptr %1, i32 0, i32 0, !dbg !48
  store i32 42, ptr %2, align 8, !dbg !49
  ret void, !dbg !50
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !51 {
  call void @process(), !dbg !54
  ret i32 0, !dbg !55
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.value(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!12, !13, !14, !15, !16, !17, !18}
!llvm.ident = !{!19}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/checker_generic_resource_leak.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "54dbaeed085724fa1808fd4571f8af8d")
!2 = !{!3, !10}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIDerivedType(tag: DW_TAG_typedef, name: "Resource", file: !1, line: 7, baseType: !5)
!5 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !1, line: 4, size: 128, elements: !6)
!6 = !{!7, !9}
!7 = !DIDerivedType(tag: DW_TAG_member, name: "handle", scope: !5, file: !1, line: 5, baseType: !8, size: 32)
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !DIDerivedType(tag: DW_TAG_member, name: "name", scope: !5, file: !1, line: 6, baseType: !10, size: 64, offset: 64)
!10 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !11, size: 64)
!11 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!12 = !{i32 7, !"Dwarf Version", i32 5}
!13 = !{i32 2, !"Debug Info Version", i32 3}
!14 = !{i32 1, !"wchar_size", i32 4}
!15 = !{i32 8, !"PIC Level", i32 2}
!16 = !{i32 7, !"PIE Level", i32 2}
!17 = !{i32 7, !"uwtable", i32 2}
!18 = !{i32 7, !"frame-pointer", i32 1}
!19 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!20 = distinct !DISubprogram(name: "acquire_resource", scope: !1, file: !1, line: 9, type: !21, scopeLine: 9, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !23)
!21 = !DISubroutineType(types: !22)
!22 = !{!3}
!23 = !{}
!24 = !DILocation(line: 10, column: 31, scope: !20)
!25 = !DILocalVariable(name: "r", scope: !20, file: !1, line: 10, type: !3)
!26 = !DILocation(line: 0, scope: !20)
!27 = !DILocation(line: 11, column: 8, scope: !20)
!28 = !DILocation(line: 11, column: 15, scope: !20)
!29 = !DILocation(line: 12, column: 23, scope: !20)
!30 = !DILocation(line: 12, column: 8, scope: !20)
!31 = !DILocation(line: 12, column: 13, scope: !20)
!32 = !DILocation(line: 13, column: 5, scope: !20)
!33 = distinct !DISubprogram(name: "release_resource", scope: !1, file: !1, line: 16, type: !34, scopeLine: 16, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !23)
!34 = !DISubroutineType(types: !35)
!35 = !{null, !3}
!36 = !DILocalVariable(name: "r", arg: 1, scope: !33, file: !1, line: 16, type: !3)
!37 = !DILocation(line: 0, scope: !33)
!38 = !DILocation(line: 17, column: 13, scope: !33)
!39 = !DILocation(line: 17, column: 5, scope: !33)
!40 = !DILocation(line: 18, column: 5, scope: !33)
!41 = !DILocation(line: 19, column: 1, scope: !33)
!42 = distinct !DISubprogram(name: "process", scope: !1, file: !1, line: 21, type: !43, scopeLine: 21, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !23)
!43 = !DISubroutineType(types: !44)
!44 = !{null}
!45 = !DILocation(line: 22, column: 19, scope: !42)
!46 = !DILocalVariable(name: "r", scope: !42, file: !1, line: 22, type: !3)
!47 = !DILocation(line: 0, scope: !42)
!48 = !DILocation(line: 23, column: 8, scope: !42)
!49 = !DILocation(line: 23, column: 15, scope: !42)
!50 = !DILocation(line: 27, column: 1, scope: !42)
!51 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 29, type: !52, scopeLine: 29, spFlags: DISPFlagDefinition, unit: !0)
!52 = !DISubroutineType(types: !53)
!53 = !{!8}
!54 = !DILocation(line: 30, column: 5, scope: !51)
!55 = !DILocation(line: 31, column: 5, scope: !51)
