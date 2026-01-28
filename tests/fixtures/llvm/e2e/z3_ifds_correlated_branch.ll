; ModuleID = 'tests/programs/c/z3_ifds_correlated_branch.c'
source_filename = "tests/programs/c/z3_ifds_correlated_branch.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [11 x i8] c"USER_INPUT\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [13 x i8] c"safe_default\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @dispatch(i32 noundef %0) #0 !dbg !22 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %3, metadata !29, metadata !DIExpression()), !dbg !31
  %4 = load i32, ptr %2, align 4, !dbg !32
  %5 = icmp eq i32 %4, 1, !dbg !34
  br i1 %5, label %6, label %8, !dbg !35

6:                                                ; preds = %1
  %7 = call ptr @getenv(ptr noundef @.str) #4, !dbg !36
  store ptr %7, ptr %3, align 8, !dbg !38
  br label %9, !dbg !39

8:                                                ; preds = %1
  store ptr @.str.1, ptr %3, align 8, !dbg !40
  br label %9

9:                                                ; preds = %8, %6
  %10 = load i32, ptr %2, align 4, !dbg !42
  %11 = icmp eq i32 %10, 1, !dbg !44
  br i1 %11, label %12, label %15, !dbg !45

12:                                               ; preds = %9
  %13 = load ptr, ptr %3, align 8, !dbg !46
  %14 = call i32 @system(ptr noundef %13), !dbg !48
  br label %15, !dbg !49

15:                                               ; preds = %12, %9
  ret void, !dbg !50
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

declare i32 @system(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !51 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @dispatch(i32 noundef 1), !dbg !54
  ret i32 0, !dbg !55
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!14, !15, !16, !17, !18, !19, !20}
!llvm.ident = !{!21}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 14, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_ifds_correlated_branch.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "2e0a8310b60755b48dc7470cd7cba2bf")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 88, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 11)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 16, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 104, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 13)
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
!22 = distinct !DISubprogram(name: "dispatch", scope: !2, file: !2, line: 10, type: !23, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !26)
!23 = !DISubroutineType(types: !24)
!24 = !{null, !25}
!25 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!26 = !{}
!27 = !DILocalVariable(name: "mode", arg: 1, scope: !22, file: !2, line: 10, type: !25)
!28 = !DILocation(line: 10, column: 19, scope: !22)
!29 = !DILocalVariable(name: "data", scope: !22, file: !2, line: 11, type: !30)
!30 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!31 = !DILocation(line: 11, column: 11, scope: !22)
!32 = !DILocation(line: 12, column: 9, scope: !33)
!33 = distinct !DILexicalBlock(scope: !22, file: !2, line: 12, column: 9)
!34 = !DILocation(line: 12, column: 14, scope: !33)
!35 = !DILocation(line: 12, column: 9, scope: !22)
!36 = !DILocation(line: 14, column: 16, scope: !37)
!37 = distinct !DILexicalBlock(scope: !33, file: !2, line: 12, column: 20)
!38 = !DILocation(line: 14, column: 14, scope: !37)
!39 = !DILocation(line: 15, column: 5, scope: !37)
!40 = !DILocation(line: 16, column: 14, scope: !41)
!41 = distinct !DILexicalBlock(scope: !33, file: !2, line: 15, column: 12)
!42 = !DILocation(line: 19, column: 9, scope: !43)
!43 = distinct !DILexicalBlock(scope: !22, file: !2, line: 19, column: 9)
!44 = !DILocation(line: 19, column: 14, scope: !43)
!45 = !DILocation(line: 19, column: 9, scope: !22)
!46 = !DILocation(line: 22, column: 16, scope: !47)
!47 = distinct !DILexicalBlock(scope: !43, file: !2, line: 19, column: 20)
!48 = !DILocation(line: 22, column: 9, scope: !47)
!49 = !DILocation(line: 23, column: 5, scope: !47)
!50 = !DILocation(line: 26, column: 1, scope: !22)
!51 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 28, type: !52, scopeLine: 28, spFlags: DISPFlagDefinition, unit: !12)
!52 = !DISubroutineType(types: !53)
!53 = !{!25}
!54 = !DILocation(line: 29, column: 5, scope: !51)
!55 = !DILocation(line: 30, column: 5, scope: !51)
