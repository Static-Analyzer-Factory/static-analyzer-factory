; ModuleID = 'tests/programs/c/z3_numeric_genuine_overflow.c'
source_filename = "tests/programs/c/z3_numeric_genuine_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [8 x i8] c"1000000\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @compute(i32 noundef %0, i32 noundef %1) #0 !dbg !17 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !22, metadata !DIExpression()), !dbg !23
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !24, metadata !DIExpression()), !dbg !25
  call void @llvm.dbg.declare(metadata ptr %5, metadata !26, metadata !DIExpression()), !dbg !27
  %6 = load i32, ptr %3, align 4, !dbg !28
  %7 = load i32, ptr %4, align 4, !dbg !29
  %8 = mul nsw i32 %6, %7, !dbg !30
  store i32 %8, ptr %5, align 4, !dbg !27
  %9 = load i32, ptr %5, align 4, !dbg !31
  ret i32 %9, !dbg !32
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !33 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !36, metadata !DIExpression()), !dbg !37
  %4 = call i32 @atoi(ptr noundef @.str) #3, !dbg !38
  store i32 %4, ptr %2, align 4, !dbg !37
  call void @llvm.dbg.declare(metadata ptr %3, metadata !39, metadata !DIExpression()), !dbg !40
  %5 = call i32 @atoi(ptr noundef @.str) #3, !dbg !41
  store i32 %5, ptr %3, align 4, !dbg !40
  %6 = load i32, ptr %2, align 4, !dbg !42
  %7 = load i32, ptr %3, align 4, !dbg !43
  %8 = call i32 @compute(i32 noundef %6, i32 noundef %7), !dbg !44
  ret i32 %8, !dbg !45
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @atoi(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind willreturn memory(read) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind willreturn memory(read) }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 12, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_numeric_genuine_overflow.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "98b83683ef2870e82c83e6501539bfac")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 64, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 8)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !8, splitDebugInlining: false, nameTableKind: None)
!8 = !{!0}
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "compute", scope: !2, file: !2, line: 5, type: !18, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !21)
!18 = !DISubroutineType(types: !19)
!19 = !{!20, !20, !20}
!20 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!21 = !{}
!22 = !DILocalVariable(name: "a", arg: 1, scope: !17, file: !2, line: 5, type: !20)
!23 = !DILocation(line: 5, column: 17, scope: !17)
!24 = !DILocalVariable(name: "b", arg: 2, scope: !17, file: !2, line: 5, type: !20)
!25 = !DILocation(line: 5, column: 24, scope: !17)
!26 = !DILocalVariable(name: "result", scope: !17, file: !2, line: 7, type: !20)
!27 = !DILocation(line: 7, column: 9, scope: !17)
!28 = !DILocation(line: 7, column: 18, scope: !17)
!29 = !DILocation(line: 7, column: 22, scope: !17)
!30 = !DILocation(line: 7, column: 20, scope: !17)
!31 = !DILocation(line: 8, column: 12, scope: !17)
!32 = !DILocation(line: 8, column: 5, scope: !17)
!33 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 11, type: !34, scopeLine: 11, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !21)
!34 = !DISubroutineType(types: !35)
!35 = !{!20}
!36 = !DILocalVariable(name: "a", scope: !33, file: !2, line: 12, type: !20)
!37 = !DILocation(line: 12, column: 9, scope: !33)
!38 = !DILocation(line: 12, column: 13, scope: !33)
!39 = !DILocalVariable(name: "b", scope: !33, file: !2, line: 13, type: !20)
!40 = !DILocation(line: 13, column: 9, scope: !33)
!41 = !DILocation(line: 13, column: 13, scope: !33)
!42 = !DILocation(line: 14, column: 20, scope: !33)
!43 = !DILocation(line: 14, column: 23, scope: !33)
!44 = !DILocation(line: 14, column: 12, scope: !33)
!45 = !DILocation(line: 14, column: 5, scope: !33)
