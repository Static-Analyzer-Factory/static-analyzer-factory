; ModuleID = '/workspace/tests/programs/cpp/svfg_class_member.cpp'
source_filename = "/workspace/tests/programs/cpp/svfg_class_member.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.Container = type { i32 }

$_ZN9ContainerC2Ei = comdat any

$_ZNK9Container3getEv = comdat any

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z4testv() #0 !dbg !25 {
  %1 = alloca i32, align 4
  %2 = alloca %class.Container, align 4
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !29, metadata !DIExpression()), !dbg !30
  %4 = call i32 @source(), !dbg !31
  store i32 %4, ptr %1, align 4, !dbg !30
  call void @llvm.dbg.declare(metadata ptr %2, metadata !32, metadata !DIExpression()), !dbg !33
  %5 = load i32, ptr %1, align 4, !dbg !34
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %2, i32 noundef %5), !dbg !33
  call void @llvm.dbg.declare(metadata ptr %3, metadata !35, metadata !DIExpression()), !dbg !36
  %6 = call noundef i32 @_ZNK9Container3getEv(ptr noundef nonnull align 4 dereferenceable(4) %2), !dbg !37
  store i32 %6, ptr %3, align 4, !dbg !36
  %7 = load i32, ptr %3, align 4, !dbg !38
  call void @sink(i32 noundef %7), !dbg !39
  ret void, !dbg !40
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @source() #2

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %0, i32 noundef %1) unnamed_addr #3 comdat align 2 !dbg !41 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !42, metadata !DIExpression()), !dbg !44
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !45, metadata !DIExpression()), !dbg !46
  %5 = load ptr, ptr %3, align 8
  %6 = getelementptr inbounds %class.Container, ptr %5, i32 0, i32 0, !dbg !47
  %7 = load i32, ptr %4, align 4, !dbg !48
  store i32 %7, ptr %6, align 4, !dbg !47
  ret void, !dbg !49
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZNK9Container3getEv(ptr noundef nonnull align 4 dereferenceable(4) %0) #3 comdat align 2 !dbg !50 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !51, metadata !DIExpression()), !dbg !53
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Container, ptr %3, i32 0, i32 0, !dbg !54
  %5 = load i32, ptr %4, align 4, !dbg !54
  ret i32 %5, !dbg !55
}

declare void @sink(i32 noundef) #2

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!17, !18, !19, !20, !21, !22, !23}
!llvm.ident = !{!24}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/cpp/svfg_class_member.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "38ff6afb7ff228a2c5154d76531fe208")
!2 = !{!3}
!3 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Container", file: !4, line: 8, size: 32, flags: DIFlagTypePassByValue | DIFlagNonTrivial, elements: !5, identifier: "_ZTS9Container")
!4 = !DIFile(filename: "cpp/svfg_class_member.cpp", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "38ff6afb7ff228a2c5154d76531fe208")
!5 = !{!6, !8, !12}
!6 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !3, file: !4, line: 9, baseType: !7, size: 32)
!7 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!8 = !DISubprogram(name: "Container", scope: !3, file: !4, line: 11, type: !9, scopeLine: 11, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!9 = !DISubroutineType(types: !10)
!10 = !{null, !11, !7}
!11 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !3, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!12 = !DISubprogram(name: "get", linkageName: "_ZNK9Container3getEv", scope: !3, file: !4, line: 12, type: !13, scopeLine: 12, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!13 = !DISubroutineType(types: !14)
!14 = !{!7, !15}
!15 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!16 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !3)
!17 = !{i32 7, !"Dwarf Version", i32 5}
!18 = !{i32 2, !"Debug Info Version", i32 3}
!19 = !{i32 1, !"wchar_size", i32 4}
!20 = !{i32 8, !"PIC Level", i32 2}
!21 = !{i32 7, !"PIE Level", i32 2}
!22 = !{i32 7, !"uwtable", i32 2}
!23 = !{i32 7, !"frame-pointer", i32 1}
!24 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!25 = distinct !DISubprogram(name: "test", linkageName: "_Z4testv", scope: !4, file: !4, line: 15, type: !26, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !28)
!26 = !DISubroutineType(types: !27)
!27 = !{null}
!28 = !{}
!29 = !DILocalVariable(name: "tainted", scope: !25, file: !4, line: 16, type: !7)
!30 = !DILocation(line: 16, column: 9, scope: !25)
!31 = !DILocation(line: 16, column: 19, scope: !25)
!32 = !DILocalVariable(name: "c", scope: !25, file: !4, line: 17, type: !3)
!33 = !DILocation(line: 17, column: 15, scope: !25)
!34 = !DILocation(line: 17, column: 17, scope: !25)
!35 = !DILocalVariable(name: "val", scope: !25, file: !4, line: 18, type: !7)
!36 = !DILocation(line: 18, column: 9, scope: !25)
!37 = !DILocation(line: 18, column: 17, scope: !25)
!38 = !DILocation(line: 19, column: 10, scope: !25)
!39 = !DILocation(line: 19, column: 5, scope: !25)
!40 = !DILocation(line: 20, column: 1, scope: !25)
!41 = distinct !DISubprogram(name: "Container", linkageName: "_ZN9ContainerC2Ei", scope: !3, file: !4, line: 11, type: !9, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !8, retainedNodes: !28)
!42 = !DILocalVariable(name: "this", arg: 1, scope: !41, type: !43, flags: DIFlagArtificial | DIFlagObjectPointer)
!43 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !3, size: 64)
!44 = !DILocation(line: 0, scope: !41)
!45 = !DILocalVariable(name: "v", arg: 2, scope: !41, file: !4, line: 11, type: !7)
!46 = !DILocation(line: 11, column: 19, scope: !41)
!47 = !DILocation(line: 11, column: 24, scope: !41)
!48 = !DILocation(line: 11, column: 30, scope: !41)
!49 = !DILocation(line: 11, column: 34, scope: !41)
!50 = distinct !DISubprogram(name: "get", linkageName: "_ZNK9Container3getEv", scope: !3, file: !4, line: 12, type: !13, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !12, retainedNodes: !28)
!51 = !DILocalVariable(name: "this", arg: 1, scope: !50, type: !52, flags: DIFlagArtificial | DIFlagObjectPointer)
!52 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!53 = !DILocation(line: 0, scope: !50)
!54 = !DILocation(line: 12, column: 30, scope: !50)
!55 = !DILocation(line: 12, column: 23, scope: !50)
