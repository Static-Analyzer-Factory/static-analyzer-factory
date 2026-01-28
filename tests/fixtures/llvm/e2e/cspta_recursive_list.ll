; ModuleID = '/workspace/tests/programs/c/cspta_recursive_list.c'
source_filename = "/workspace/tests/programs/c/cspta_recursive_list.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Node = type { i32, ptr }

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @create_node(i32 noundef %0) #0 !dbg !18 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !22, metadata !DIExpression()), !dbg !23
  call void @llvm.dbg.declare(metadata ptr %3, metadata !24, metadata !DIExpression()), !dbg !25
  %4 = call noalias ptr @malloc(i64 noundef 16) #4, !dbg !26
  store ptr %4, ptr %3, align 8, !dbg !25
  %5 = load ptr, ptr %3, align 8, !dbg !27
  %6 = icmp ne ptr %5, null, !dbg !27
  br i1 %6, label %7, label %13, !dbg !29

7:                                                ; preds = %1
  %8 = load i32, ptr %2, align 4, !dbg !30
  %9 = load ptr, ptr %3, align 8, !dbg !32
  %10 = getelementptr inbounds %struct.Node, ptr %9, i32 0, i32 0, !dbg !33
  store i32 %8, ptr %10, align 8, !dbg !34
  %11 = load ptr, ptr %3, align 8, !dbg !35
  %12 = getelementptr inbounds %struct.Node, ptr %11, i32 0, i32 1, !dbg !36
  store ptr null, ptr %12, align 8, !dbg !37
  br label %13, !dbg !38

13:                                               ; preds = %7, %1
  %14 = load ptr, ptr %3, align 8, !dbg !39
  ret ptr %14, !dbg !40
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @traverse(ptr noundef %0) #0 !dbg !41 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !44, metadata !DIExpression()), !dbg !45
  %4 = load ptr, ptr %3, align 8, !dbg !46
  %5 = icmp ne ptr %4, null, !dbg !46
  br i1 %5, label %7, label %6, !dbg !48

6:                                                ; preds = %1
  store i32 0, ptr %2, align 4, !dbg !49
  br label %16, !dbg !49

7:                                                ; preds = %1
  %8 = load ptr, ptr %3, align 8, !dbg !50
  %9 = getelementptr inbounds %struct.Node, ptr %8, i32 0, i32 0, !dbg !51
  %10 = load i32, ptr %9, align 8, !dbg !51
  %11 = load ptr, ptr %3, align 8, !dbg !52
  %12 = getelementptr inbounds %struct.Node, ptr %11, i32 0, i32 1, !dbg !53
  %13 = load ptr, ptr %12, align 8, !dbg !53
  %14 = call i32 @traverse(ptr noundef %13), !dbg !54
  %15 = add nsw i32 %10, %14, !dbg !55
  store i32 %15, ptr %2, align 4, !dbg !56
  br label %16, !dbg !56

16:                                               ; preds = %7, %6
  %17 = load i32, ptr %2, align 4, !dbg !57
  ret i32 %17, !dbg !57
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !58 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !61, metadata !DIExpression()), !dbg !62
  %6 = call ptr @create_node(i32 noundef 1), !dbg !63
  store ptr %6, ptr %2, align 8, !dbg !62
  call void @llvm.dbg.declare(metadata ptr %3, metadata !64, metadata !DIExpression()), !dbg !65
  %7 = call ptr @create_node(i32 noundef 2), !dbg !66
  store ptr %7, ptr %3, align 8, !dbg !65
  call void @llvm.dbg.declare(metadata ptr %4, metadata !67, metadata !DIExpression()), !dbg !68
  %8 = call ptr @create_node(i32 noundef 3), !dbg !69
  store ptr %8, ptr %4, align 8, !dbg !68
  %9 = load ptr, ptr %3, align 8, !dbg !70
  %10 = load ptr, ptr %2, align 8, !dbg !71
  %11 = getelementptr inbounds %struct.Node, ptr %10, i32 0, i32 1, !dbg !72
  store ptr %9, ptr %11, align 8, !dbg !73
  %12 = load ptr, ptr %4, align 8, !dbg !74
  %13 = load ptr, ptr %3, align 8, !dbg !75
  %14 = getelementptr inbounds %struct.Node, ptr %13, i32 0, i32 1, !dbg !76
  store ptr %12, ptr %14, align 8, !dbg !77
  call void @llvm.dbg.declare(metadata ptr %5, metadata !78, metadata !DIExpression()), !dbg !79
  %15 = load ptr, ptr %2, align 8, !dbg !80
  %16 = call i32 @traverse(ptr noundef %15), !dbg !81
  store i32 %16, ptr %5, align 4, !dbg !79
  %17 = load ptr, ptr %2, align 8, !dbg !82
  call void @free(ptr noundef %17) #5, !dbg !83
  %18 = load ptr, ptr %3, align 8, !dbg !84
  call void @free(ptr noundef %18) #5, !dbg !85
  %19 = load ptr, ptr %4, align 8, !dbg !86
  call void @free(ptr noundef %19) #5, !dbg !87
  %20 = load i32, ptr %5, align 4, !dbg !88
  ret i32 %20, !dbg !89
}

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!10, !11, !12, !13, !14, !15, !16}
!llvm.ident = !{!17}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/cspta_recursive_list.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bd695340d5cae374d70fb04fb2219e4e")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Node", file: !5, line: 9, size: 128, elements: !6)
!5 = !DIFile(filename: "c/cspta_recursive_list.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bd695340d5cae374d70fb04fb2219e4e")
!6 = !{!7, !9}
!7 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !4, file: !5, line: 10, baseType: !8, size: 32)
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !DIDerivedType(tag: DW_TAG_member, name: "next", scope: !4, file: !5, line: 11, baseType: !3, size: 64, offset: 64)
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "create_node", scope: !5, file: !5, line: 14, type: !19, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!19 = !DISubroutineType(types: !20)
!20 = !{!3, !8}
!21 = !{}
!22 = !DILocalVariable(name: "val", arg: 1, scope: !18, file: !5, line: 14, type: !8)
!23 = !DILocation(line: 14, column: 30, scope: !18)
!24 = !DILocalVariable(name: "n", scope: !18, file: !5, line: 15, type: !3)
!25 = !DILocation(line: 15, column: 18, scope: !18)
!26 = !DILocation(line: 15, column: 37, scope: !18)
!27 = !DILocation(line: 16, column: 9, scope: !28)
!28 = distinct !DILexicalBlock(scope: !18, file: !5, line: 16, column: 9)
!29 = !DILocation(line: 16, column: 9, scope: !18)
!30 = !DILocation(line: 17, column: 20, scope: !31)
!31 = distinct !DILexicalBlock(scope: !28, file: !5, line: 16, column: 12)
!32 = !DILocation(line: 17, column: 9, scope: !31)
!33 = !DILocation(line: 17, column: 12, scope: !31)
!34 = !DILocation(line: 17, column: 18, scope: !31)
!35 = !DILocation(line: 18, column: 9, scope: !31)
!36 = !DILocation(line: 18, column: 12, scope: !31)
!37 = !DILocation(line: 18, column: 17, scope: !31)
!38 = !DILocation(line: 19, column: 5, scope: !31)
!39 = !DILocation(line: 20, column: 12, scope: !18)
!40 = !DILocation(line: 20, column: 5, scope: !18)
!41 = distinct !DISubprogram(name: "traverse", scope: !5, file: !5, line: 23, type: !42, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!42 = !DISubroutineType(types: !43)
!43 = !{!8, !3}
!44 = !DILocalVariable(name: "head", arg: 1, scope: !41, file: !5, line: 23, type: !3)
!45 = !DILocation(line: 23, column: 27, scope: !41)
!46 = !DILocation(line: 24, column: 10, scope: !47)
!47 = distinct !DILexicalBlock(scope: !41, file: !5, line: 24, column: 9)
!48 = !DILocation(line: 24, column: 9, scope: !41)
!49 = !DILocation(line: 24, column: 16, scope: !47)
!50 = !DILocation(line: 25, column: 12, scope: !41)
!51 = !DILocation(line: 25, column: 18, scope: !41)
!52 = !DILocation(line: 25, column: 35, scope: !41)
!53 = !DILocation(line: 25, column: 41, scope: !41)
!54 = !DILocation(line: 25, column: 26, scope: !41)
!55 = !DILocation(line: 25, column: 24, scope: !41)
!56 = !DILocation(line: 25, column: 5, scope: !41)
!57 = !DILocation(line: 26, column: 1, scope: !41)
!58 = distinct !DISubprogram(name: "main", scope: !5, file: !5, line: 28, type: !59, scopeLine: 28, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !21)
!59 = !DISubroutineType(types: !60)
!60 = !{!8}
!61 = !DILocalVariable(name: "a", scope: !58, file: !5, line: 29, type: !3)
!62 = !DILocation(line: 29, column: 18, scope: !58)
!63 = !DILocation(line: 29, column: 22, scope: !58)
!64 = !DILocalVariable(name: "b", scope: !58, file: !5, line: 30, type: !3)
!65 = !DILocation(line: 30, column: 18, scope: !58)
!66 = !DILocation(line: 30, column: 22, scope: !58)
!67 = !DILocalVariable(name: "c", scope: !58, file: !5, line: 31, type: !3)
!68 = !DILocation(line: 31, column: 18, scope: !58)
!69 = !DILocation(line: 31, column: 22, scope: !58)
!70 = !DILocation(line: 33, column: 15, scope: !58)
!71 = !DILocation(line: 33, column: 5, scope: !58)
!72 = !DILocation(line: 33, column: 8, scope: !58)
!73 = !DILocation(line: 33, column: 13, scope: !58)
!74 = !DILocation(line: 34, column: 15, scope: !58)
!75 = !DILocation(line: 34, column: 5, scope: !58)
!76 = !DILocation(line: 34, column: 8, scope: !58)
!77 = !DILocation(line: 34, column: 13, scope: !58)
!78 = !DILocalVariable(name: "sum", scope: !58, file: !5, line: 36, type: !8)
!79 = !DILocation(line: 36, column: 9, scope: !58)
!80 = !DILocation(line: 36, column: 24, scope: !58)
!81 = !DILocation(line: 36, column: 15, scope: !58)
!82 = !DILocation(line: 38, column: 10, scope: !58)
!83 = !DILocation(line: 38, column: 5, scope: !58)
!84 = !DILocation(line: 39, column: 10, scope: !58)
!85 = !DILocation(line: 39, column: 5, scope: !58)
!86 = !DILocation(line: 40, column: 10, scope: !58)
!87 = !DILocation(line: 40, column: 5, scope: !58)
!88 = !DILocation(line: 41, column: 12, scope: !58)
!89 = !DILocation(line: 41, column: 5, scope: !58)
