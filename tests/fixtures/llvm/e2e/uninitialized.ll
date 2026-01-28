; ModuleID = '/workspace/tests/programs/c/uninitialized.c'
source_filename = "/workspace/tests/programs/c/uninitialized.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"ok\0A\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !18 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !25, metadata !DIExpression()), !dbg !26
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !27, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %6, metadata !29, metadata !DIExpression()), !dbg !30
  %7 = load i32, ptr %4, align 4, !dbg !31
  %8 = icmp sgt i32 %7, 2, !dbg !33
  br i1 %8, label %9, label %10, !dbg !34

9:                                                ; preds = %2
  store i32 0, ptr %6, align 4, !dbg !35
  br label %10, !dbg !37

10:                                               ; preds = %9, %2
  %11 = load i32, ptr %6, align 4, !dbg !38
  %12 = icmp eq i32 %11, 0, !dbg !40
  br i1 %12, label %13, label %15, !dbg !41

13:                                               ; preds = %10
  %14 = call i32 (ptr, ...) @printf(ptr noundef @.str), !dbg !42
  br label %15, !dbg !44

15:                                               ; preds = %13, %10
  ret i32 0, !dbg !45
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @printf(ptr noundef, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!10, !11, !12, !13, !14, !15, !16}
!llvm.ident = !{!17}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 14, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/uninitialized.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "781fd5d3a654381a95b3103aae66d866")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 4)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !9, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/uninitialized.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "781fd5d3a654381a95b3103aae66d866")
!9 = !{!0}
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 7, type: !19, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !24)
!19 = !DISubroutineType(types: !20)
!20 = !{!21, !21, !22}
!21 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!22 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !23, size: 64)
!23 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!24 = !{}
!25 = !DILocalVariable(name: "argc", arg: 1, scope: !18, file: !2, line: 7, type: !21)
!26 = !DILocation(line: 7, column: 14, scope: !18)
!27 = !DILocalVariable(name: "argv", arg: 2, scope: !18, file: !2, line: 7, type: !22)
!28 = !DILocation(line: 7, column: 26, scope: !18)
!29 = !DILocalVariable(name: "status", scope: !18, file: !2, line: 8, type: !21)
!30 = !DILocation(line: 8, column: 9, scope: !18)
!31 = !DILocation(line: 9, column: 9, scope: !32)
!32 = distinct !DILexicalBlock(scope: !18, file: !2, line: 9, column: 9)
!33 = !DILocation(line: 9, column: 14, scope: !32)
!34 = !DILocation(line: 9, column: 9, scope: !18)
!35 = !DILocation(line: 10, column: 16, scope: !36)
!36 = distinct !DILexicalBlock(scope: !32, file: !2, line: 9, column: 19)
!37 = !DILocation(line: 11, column: 5, scope: !36)
!38 = !DILocation(line: 13, column: 9, scope: !39)
!39 = distinct !DILexicalBlock(scope: !18, file: !2, line: 13, column: 9)
!40 = !DILocation(line: 13, column: 16, scope: !39)
!41 = !DILocation(line: 13, column: 9, scope: !18)
!42 = !DILocation(line: 14, column: 9, scope: !43)
!43 = distinct !DILexicalBlock(scope: !39, file: !2, line: 13, column: 22)
!44 = !DILocation(line: 15, column: 5, scope: !43)
!45 = !DILocation(line: 16, column: 5, scope: !18)
