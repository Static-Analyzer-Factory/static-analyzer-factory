; ModuleID = 'indirect_call_named_ssa.c'
source_filename = "tests/programs/c/indirect_call_named_ssa.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @double_it(i32 noundef %0) #0 !dbg !7 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !12, metadata !DIExpression()), !dbg !13
  %3 = load i32, ptr %2, align 4, !dbg !14
  %4 = mul nsw i32 %3, 2, !dbg !15
  ret i32 %4, !dbg !16
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @triple_it(i32 noundef %0) #0 !dbg !17 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  %3 = load i32, ptr %2, align 4, !dbg !20
  %4 = mul nsw i32 %3, 3, !dbg !21
  ret i32 %4, !dbg !22
}

; Function Attrs: noinline nounwind uwtable
define dso_local ptr @get_callback(i32 noundef %0) #0 !dbg !23 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !28, metadata !DIExpression()), !dbg !29
  %4 = load i32, ptr %3, align 4, !dbg !30
  %5 = icmp ne i32 %4, 0, !dbg !30
  br i1 %5, label %6, label %7, !dbg !32

6:                                                ; preds = %1
  store ptr @double_it, ptr %2, align 8, !dbg !33
  br label %8, !dbg !33

7:                                                ; preds = %1
  store ptr @triple_it, ptr %2, align 8, !dbg !34
  br label %8, !dbg !34

8:                                                ; preds = %7, %6
  %9 = load ptr, ptr %2, align 8, !dbg !35
  ret ptr %9, !dbg !35
}

; Function Attrs: noinline nounwind uwtable
; NOTE: Hand-edited to simulate mem2reg output — named SSA values (%call, %result)
; replace alloca/load/store. This triggers the bug where %call (a named local
; SSA value) was misclassified as a direct call to a function named "call".
define dso_local i32 @use_callback(i32 noundef %choice, i32 noundef %value) #0 !dbg !36 {
entry:
  %call = call ptr @get_callback(i32 noundef %choice), !dbg !46
  %result = call i32 %call(i32 noundef %value), !dbg !47
  ret i32 %result, !dbg !49
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 !dbg !50 {
  %1 = call i32 @use_callback(i32 noundef 1, i32 noundef 42), !dbg !53
  ret i32 %1, !dbg !54
}

attributes #0 = { noinline nounwind uwtable "frame-pointer"="non-leaf" "target-cpu"="generic" "target-features"="+v8a" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!3, !4, !5}
!llvm.ident = !{!6}

!0 = distinct !DICompileUnit(language: DW_LANG_C99, file: !1, producer: "clang version 18.1.8", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, enums: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/indirect_call_named_ssa.c", directory: "/workspace")
!2 = !{}
!3 = !{i32 7, !"Dwarf Version", i32 4}
!4 = !{i32 2, !"Debug Info Version", i32 3}
!5 = !{i32 1, !"wchar_size", i32 4}
!6 = !{!"clang version 18.1.8"}
!7 = distinct !DISubprogram(name: "double_it", scope: !8, file: !8, line: 7, type: !9, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !2)
!8 = !DIFile(filename: "tests/programs/c/indirect_call_named_ssa.c", directory: "/workspace")
!9 = !DISubroutineType(types: !10)
!10 = !{!11, !11}
!11 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!12 = !DILocalVariable(name: "x", arg: 1, scope: !7, file: !8, line: 7, type: !11)
!13 = !DILocation(line: 7, column: 19, scope: !7)
!14 = !DILocation(line: 7, column: 31, scope: !7)
!15 = !DILocation(line: 7, column: 33, scope: !7)
!16 = !DILocation(line: 7, column: 24, scope: !7)
!17 = distinct !DISubprogram(name: "triple_it", scope: !8, file: !8, line: 8, type: !9, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !2)
!18 = !DILocalVariable(name: "x", arg: 1, scope: !17, file: !8, line: 8, type: !11)
!19 = !DILocation(line: 8, column: 19, scope: !17)
!20 = !DILocation(line: 8, column: 31, scope: !17)
!21 = !DILocation(line: 8, column: 33, scope: !17)
!22 = !DILocation(line: 8, column: 24, scope: !17)
!23 = distinct !DISubprogram(name: "get_callback", scope: !8, file: !8, line: 11, type: !24, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !2)
!24 = !DISubroutineType(types: !25)
!25 = !{!26, !11}
!26 = !DIDerivedType(tag: DW_TAG_typedef, name: "callback_t", file: !8, line: 5, baseType: !27)
!27 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !9, size: 64)
!28 = !DILocalVariable(name: "choice", arg: 1, scope: !23, file: !8, line: 11, type: !11)
!29 = !DILocation(line: 11, column: 29, scope: !23)
!30 = !DILocation(line: 12, column: 9, scope: !31)
!31 = distinct !DILexicalBlock(scope: !23, file: !8, line: 12, column: 9)
!32 = !DILocation(line: 12, column: 9, scope: !23)
!33 = !DILocation(line: 13, column: 9, scope: !31)
!34 = !DILocation(line: 14, column: 5, scope: !23)
!35 = !DILocation(line: 15, column: 1, scope: !23)
!36 = distinct !DISubprogram(name: "use_callback", scope: !8, file: !8, line: 17, type: !37, scopeLine: 17, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !2)
!37 = !DISubroutineType(types: !38)
!38 = !{!11, !11, !11}
!39 = !DILocalVariable(name: "choice", arg: 1, scope: !36, file: !8, line: 17, type: !11)
!40 = !DILocation(line: 17, column: 22, scope: !36)
!41 = !DILocalVariable(name: "value", arg: 2, scope: !36, file: !8, line: 17, type: !11)
!42 = !DILocation(line: 17, column: 34, scope: !36)
!43 = !DILocalVariable(name: "cb", scope: !36, file: !8, line: 18, type: !26)
!44 = !DILocation(line: 18, column: 16, scope: !36)
!45 = !DILocation(line: 18, column: 34, scope: !36)
!46 = !DILocation(line: 18, column: 21, scope: !36)
!47 = !DILocation(line: 19, column: 12, scope: !36)
!48 = !DILocation(line: 19, column: 15, scope: !36)
!49 = !DILocation(line: 19, column: 5, scope: !36)
!50 = distinct !DISubprogram(name: "main", scope: !8, file: !8, line: 22, type: !51, scopeLine: 22, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !2)
!51 = !DISubroutineType(types: !52)
!52 = !{!11}
!53 = !DILocation(line: 23, column: 12, scope: !50)
!54 = !DILocation(line: 23, column: 5, scope: !50)
