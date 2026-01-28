; ModuleID = '/workspace/tests/programs/cpp/fspta_cpp_field.cpp'
source_filename = "/workspace/tests/programs/cpp/fspta_cpp_field.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.Container = type { ptr }

$_ZN9Container5set_aEv = comdat any

$_ZN9Container5set_bEv = comdat any

$_ZN9Container4readEv = comdat any

@a_val = dso_local global i32 0, align 4, !dbg !0
@b_val = dso_local global i32 0, align 4, !dbg !5

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #0 !dbg !17 {
  %1 = alloca i32, align 4
  %2 = alloca %class.Container, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !34
  call void @_ZN9Container5set_aEv(ptr noundef nonnull align 8 dereferenceable(8) %2), !dbg !35
  call void @_ZN9Container5set_bEv(ptr noundef nonnull align 8 dereferenceable(8) %2), !dbg !36
  %3 = call noundef i32 @_ZN9Container4readEv(ptr noundef nonnull align 8 dereferenceable(8) %2), !dbg !37
  ret i32 %3, !dbg !38
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9Container5set_aEv(ptr noundef nonnull align 8 dereferenceable(8) %0) #2 comdat align 2 !dbg !39 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !40, metadata !DIExpression()), !dbg !42
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Container, ptr %3, i32 0, i32 0, !dbg !43
  store ptr @a_val, ptr %4, align 8, !dbg !44
  ret void, !dbg !45
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9Container5set_bEv(ptr noundef nonnull align 8 dereferenceable(8) %0) #2 comdat align 2 !dbg !46 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !47, metadata !DIExpression()), !dbg !48
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Container, ptr %3, i32 0, i32 0, !dbg !49
  store ptr @b_val, ptr %4, align 8, !dbg !50
  ret void, !dbg !51
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN9Container4readEv(ptr noundef nonnull align 8 dereferenceable(8) %0) #2 comdat align 2 !dbg !52 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !53, metadata !DIExpression()), !dbg !54
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Container, ptr %3, i32 0, i32 0, !dbg !55
  %5 = load ptr, ptr %4, align 8, !dbg !55
  %6 = load i32, ptr %5, align 4, !dbg !56
  ret i32 %6, !dbg !57
}

attributes #0 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "a_val", scope: !2, file: !7, line: 8, type: !8, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "/workspace/tests/programs/cpp/fspta_cpp_field.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "110e2b2e3b36b3aeca5a8eb7b59005b6")
!4 = !{!0, !5}
!5 = !DIGlobalVariableExpression(var: !6, expr: !DIExpression())
!6 = distinct !DIGlobalVariable(name: "b_val", scope: !2, file: !7, line: 8, type: !8, isLocal: false, isDefinition: true)
!7 = !DIFile(filename: "cpp/fspta_cpp_field.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "110e2b2e3b36b3aeca5a8eb7b59005b6")
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "main", scope: !7, file: !7, line: 27, type: !18, scopeLine: 27, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{!8}
!20 = !{}
!21 = !DILocalVariable(name: "c", scope: !17, file: !7, line: 28, type: !22)
!22 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Container", file: !7, line: 10, size: 64, flags: DIFlagTypePassByValue, elements: !23, identifier: "_ZTS9Container")
!23 = !{!24, !26, !30, !31}
!24 = !DIDerivedType(tag: DW_TAG_member, name: "data", scope: !22, file: !7, line: 12, baseType: !25, size: 64, flags: DIFlagPublic)
!25 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !8, size: 64)
!26 = !DISubprogram(name: "set_a", linkageName: "_ZN9Container5set_aEv", scope: !22, file: !7, line: 14, type: !27, scopeLine: 14, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!27 = !DISubroutineType(types: !28)
!28 = !{null, !29}
!29 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !22, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!30 = !DISubprogram(name: "set_b", linkageName: "_ZN9Container5set_bEv", scope: !22, file: !7, line: 18, type: !27, scopeLine: 18, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!31 = !DISubprogram(name: "read", linkageName: "_ZN9Container4readEv", scope: !22, file: !7, line: 22, type: !32, scopeLine: 22, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!32 = !DISubroutineType(types: !33)
!33 = !{!8, !29}
!34 = !DILocation(line: 28, column: 15, scope: !17)
!35 = !DILocation(line: 29, column: 7, scope: !17)
!36 = !DILocation(line: 30, column: 7, scope: !17)
!37 = !DILocation(line: 31, column: 14, scope: !17)
!38 = !DILocation(line: 31, column: 5, scope: !17)
!39 = distinct !DISubprogram(name: "set_a", linkageName: "_ZN9Container5set_aEv", scope: !22, file: !7, line: 14, type: !27, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, declaration: !26, retainedNodes: !20)
!40 = !DILocalVariable(name: "this", arg: 1, scope: !39, type: !41, flags: DIFlagArtificial | DIFlagObjectPointer)
!41 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !22, size: 64)
!42 = !DILocation(line: 0, scope: !39)
!43 = !DILocation(line: 15, column: 9, scope: !39)
!44 = !DILocation(line: 15, column: 14, scope: !39)
!45 = !DILocation(line: 16, column: 5, scope: !39)
!46 = distinct !DISubprogram(name: "set_b", linkageName: "_ZN9Container5set_bEv", scope: !22, file: !7, line: 18, type: !27, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, declaration: !30, retainedNodes: !20)
!47 = !DILocalVariable(name: "this", arg: 1, scope: !46, type: !41, flags: DIFlagArtificial | DIFlagObjectPointer)
!48 = !DILocation(line: 0, scope: !46)
!49 = !DILocation(line: 19, column: 9, scope: !46)
!50 = !DILocation(line: 19, column: 14, scope: !46)
!51 = !DILocation(line: 20, column: 5, scope: !46)
!52 = distinct !DISubprogram(name: "read", linkageName: "_ZN9Container4readEv", scope: !22, file: !7, line: 22, type: !32, scopeLine: 22, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, declaration: !31, retainedNodes: !20)
!53 = !DILocalVariable(name: "this", arg: 1, scope: !52, type: !41, flags: DIFlagArtificial | DIFlagObjectPointer)
!54 = !DILocation(line: 0, scope: !52)
!55 = !DILocation(line: 23, column: 17, scope: !52)
!56 = !DILocation(line: 23, column: 16, scope: !52)
!57 = !DILocation(line: 23, column: 9, scope: !52)
