; ModuleID = 'tests/programs/c/z3_assertion_failing.c'
source_filename = "tests/programs/c/z3_assertion_failing.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [23 x i8] c"idx >= 0 && idx < size\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [40 x i8] c"tests/programs/c/z3_assertion_failing.c\00", align 1, !dbg !7
@__PRETTY_FUNCTION__.unchecked_index = private unnamed_addr constant [37 x i8] c"int unchecked_index(int *, int, int)\00", align 1, !dbg !12
@.str.2 = private unnamed_addr constant [3 x i8] c"42\00", align 1, !dbg !18

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @unchecked_index(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 !dbg !33 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !39, metadata !DIExpression()), !dbg !40
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !41, metadata !DIExpression()), !dbg !42
  store i32 %2, ptr %6, align 4
  call void @llvm.dbg.declare(metadata ptr %6, metadata !43, metadata !DIExpression()), !dbg !44
  %7 = load i32, ptr %5, align 4, !dbg !45
  %8 = icmp sge i32 %7, 0, !dbg !45
  br i1 %8, label %9, label %14, !dbg !45

9:                                                ; preds = %3
  %10 = load i32, ptr %5, align 4, !dbg !45
  %11 = load i32, ptr %6, align 4, !dbg !45
  %12 = icmp slt i32 %10, %11, !dbg !45
  br i1 %12, label %13, label %14, !dbg !48

13:                                               ; preds = %9
  br label %15, !dbg !48

14:                                               ; preds = %9, %3
  call void @__assert_fail(ptr noundef @.str, ptr noundef @.str.1, i32 noundef 8, ptr noundef @__PRETTY_FUNCTION__.unchecked_index) #4, !dbg !45
  unreachable, !dbg !45

15:                                               ; preds = %13
  %16 = load ptr, ptr %4, align 8, !dbg !49
  %17 = load i32, ptr %5, align 4, !dbg !50
  %18 = sext i32 %17 to i64, !dbg !49
  %19 = getelementptr inbounds i32, ptr %16, i64 %18, !dbg !49
  %20 = load i32, ptr %19, align 4, !dbg !49
  ret i32 %20, !dbg !51
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noreturn nounwind
declare void @__assert_fail(ptr noundef, ptr noundef, i32 noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !52 {
  %1 = alloca i32, align 4
  %2 = alloca [10 x i32], align 4
  %3 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !55, metadata !DIExpression()), !dbg !59
  call void @llvm.dbg.declare(metadata ptr %3, metadata !60, metadata !DIExpression()), !dbg !61
  %4 = call i32 @atoi(ptr noundef @.str.2) #5, !dbg !62
  store i32 %4, ptr %3, align 4, !dbg !61
  %5 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !63
  %6 = load i32, ptr %3, align 4, !dbg !64
  %7 = call i32 @unchecked_index(ptr noundef %5, i32 noundef %6, i32 noundef 10), !dbg !65
  ret i32 %7, !dbg !66
}

; Function Attrs: nounwind willreturn memory(read)
declare i32 @atoi(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { noreturn nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind willreturn memory(read) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { noreturn nounwind }
attributes #5 = { nounwind willreturn memory(read) }

!llvm.dbg.cu = !{!23}
!llvm.module.flags = !{!25, !26, !27, !28, !29, !30, !31}
!llvm.ident = !{!32}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_assertion_failing.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "2ed94232964be354941c9beeb051f22f")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 184, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 23)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 320, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 40)
!12 = !DIGlobalVariableExpression(var: !13, expr: !DIExpression())
!13 = distinct !DIGlobalVariable(scope: null, file: !2, line: 8, type: !14, isLocal: true, isDefinition: true)
!14 = !DICompositeType(tag: DW_TAG_array_type, baseType: !15, size: 296, elements: !16)
!15 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!16 = !{!17}
!17 = !DISubrange(count: 37)
!18 = !DIGlobalVariableExpression(var: !19, expr: !DIExpression())
!19 = distinct !DIGlobalVariable(scope: null, file: !2, line: 14, type: !20, isLocal: true, isDefinition: true)
!20 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 24, elements: !21)
!21 = !{!22}
!22 = !DISubrange(count: 3)
!23 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !24, splitDebugInlining: false, nameTableKind: None)
!24 = !{!0, !7, !12, !18}
!25 = !{i32 7, !"Dwarf Version", i32 5}
!26 = !{i32 2, !"Debug Info Version", i32 3}
!27 = !{i32 1, !"wchar_size", i32 4}
!28 = !{i32 8, !"PIC Level", i32 2}
!29 = !{i32 7, !"PIE Level", i32 2}
!30 = !{i32 7, !"uwtable", i32 2}
!31 = !{i32 7, !"frame-pointer", i32 1}
!32 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!33 = distinct !DISubprogram(name: "unchecked_index", scope: !2, file: !2, line: 6, type: !34, scopeLine: 6, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !23, retainedNodes: !38)
!34 = !DISubroutineType(types: !35)
!35 = !{!36, !37, !36, !36}
!36 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!37 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !36, size: 64)
!38 = !{}
!39 = !DILocalVariable(name: "buf", arg: 1, scope: !33, file: !2, line: 6, type: !37)
!40 = !DILocation(line: 6, column: 26, scope: !33)
!41 = !DILocalVariable(name: "idx", arg: 2, scope: !33, file: !2, line: 6, type: !36)
!42 = !DILocation(line: 6, column: 35, scope: !33)
!43 = !DILocalVariable(name: "size", arg: 3, scope: !33, file: !2, line: 6, type: !36)
!44 = !DILocation(line: 6, column: 44, scope: !33)
!45 = !DILocation(line: 8, column: 5, scope: !46)
!46 = distinct !DILexicalBlock(scope: !47, file: !2, line: 8, column: 5)
!47 = distinct !DILexicalBlock(scope: !33, file: !2, line: 8, column: 5)
!48 = !DILocation(line: 8, column: 5, scope: !47)
!49 = !DILocation(line: 9, column: 12, scope: !33)
!50 = !DILocation(line: 9, column: 16, scope: !33)
!51 = !DILocation(line: 9, column: 5, scope: !33)
!52 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 12, type: !53, scopeLine: 12, spFlags: DISPFlagDefinition, unit: !23, retainedNodes: !38)
!53 = !DISubroutineType(types: !54)
!54 = !{!36}
!55 = !DILocalVariable(name: "buf", scope: !52, file: !2, line: 13, type: !56)
!56 = !DICompositeType(tag: DW_TAG_array_type, baseType: !36, size: 320, elements: !57)
!57 = !{!58}
!58 = !DISubrange(count: 10)
!59 = !DILocation(line: 13, column: 9, scope: !52)
!60 = !DILocalVariable(name: "idx", scope: !52, file: !2, line: 14, type: !36)
!61 = !DILocation(line: 14, column: 9, scope: !52)
!62 = !DILocation(line: 14, column: 15, scope: !52)
!63 = !DILocation(line: 15, column: 28, scope: !52)
!64 = !DILocation(line: 15, column: 33, scope: !52)
!65 = !DILocation(line: 15, column: 12, scope: !52)
!66 = !DILocation(line: 15, column: 5, scope: !52)
