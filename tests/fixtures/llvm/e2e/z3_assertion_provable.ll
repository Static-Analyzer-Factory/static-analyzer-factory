; ModuleID = 'tests/programs/c/z3_assertion_provable.c'
source_filename = "tests/programs/c/z3_assertion_provable.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [11 x i8] c"sum < 1000\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [41 x i8] c"tests/programs/c/z3_assertion_provable.c\00", align 1, !dbg !7
@__PRETTY_FUNCTION__.validated_add = private unnamed_addr constant [28 x i8] c"int validated_add(int, int)\00", align 1, !dbg !12

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @validated_add(i32 noundef %0, i32 noundef %1) #0 !dbg !28 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !33, metadata !DIExpression()), !dbg !34
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !35, metadata !DIExpression()), !dbg !36
  %7 = load i32, ptr %4, align 4, !dbg !37
  %8 = icmp slt i32 %7, 0, !dbg !39
  br i1 %8, label %12, label %9, !dbg !40

9:                                                ; preds = %2
  %10 = load i32, ptr %4, align 4, !dbg !41
  %11 = icmp sgt i32 %10, 100, !dbg !42
  br i1 %11, label %12, label %13, !dbg !43

12:                                               ; preds = %9, %2
  store i32 -1, ptr %3, align 4, !dbg !44
  br label %30, !dbg !44

13:                                               ; preds = %9
  %14 = load i32, ptr %5, align 4, !dbg !45
  %15 = icmp slt i32 %14, 0, !dbg !47
  br i1 %15, label %19, label %16, !dbg !48

16:                                               ; preds = %13
  %17 = load i32, ptr %5, align 4, !dbg !49
  %18 = icmp sgt i32 %17, 100, !dbg !50
  br i1 %18, label %19, label %20, !dbg !51

19:                                               ; preds = %16, %13
  store i32 -1, ptr %3, align 4, !dbg !52
  br label %30, !dbg !52

20:                                               ; preds = %16
  call void @llvm.dbg.declare(metadata ptr %6, metadata !53, metadata !DIExpression()), !dbg !54
  %21 = load i32, ptr %4, align 4, !dbg !55
  %22 = load i32, ptr %5, align 4, !dbg !56
  %23 = add nsw i32 %21, %22, !dbg !57
  store i32 %23, ptr %6, align 4, !dbg !54
  %24 = load i32, ptr %6, align 4, !dbg !58
  %25 = icmp slt i32 %24, 1000, !dbg !58
  br i1 %25, label %26, label %27, !dbg !61

26:                                               ; preds = %20
  br label %28, !dbg !61

27:                                               ; preds = %20
  call void @__assert_fail(ptr noundef @.str, ptr noundef @.str.1, i32 noundef 10, ptr noundef @__PRETTY_FUNCTION__.validated_add) #3, !dbg !58
  unreachable, !dbg !58

28:                                               ; preds = %26
  %29 = load i32, ptr %6, align 4, !dbg !62
  store i32 %29, ptr %3, align 4, !dbg !63
  br label %30, !dbg !63

30:                                               ; preds = %28, %19, %12
  %31 = load i32, ptr %3, align 4, !dbg !64
  ret i32 %31, !dbg !64
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noreturn nounwind
declare void @__assert_fail(ptr noundef, ptr noundef, i32 noundef, ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !65 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %2 = call i32 @validated_add(i32 noundef 10, i32 noundef 20), !dbg !68
  ret i32 %2, !dbg !69
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { noreturn nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { noreturn nounwind }

!llvm.dbg.cu = !{!18}
!llvm.module.flags = !{!20, !21, !22, !23, !24, !25, !26}
!llvm.ident = !{!27}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 10, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_assertion_provable.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "ba680f3d9e5141feb2da19ed9c326bf8")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 88, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 11)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 10, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 328, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 41)
!12 = !DIGlobalVariableExpression(var: !13, expr: !DIExpression())
!13 = distinct !DIGlobalVariable(scope: null, file: !2, line: 10, type: !14, isLocal: true, isDefinition: true)
!14 = !DICompositeType(tag: DW_TAG_array_type, baseType: !15, size: 224, elements: !16)
!15 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!16 = !{!17}
!17 = !DISubrange(count: 28)
!18 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !19, splitDebugInlining: false, nameTableKind: None)
!19 = !{!0, !7, !12}
!20 = !{i32 7, !"Dwarf Version", i32 5}
!21 = !{i32 2, !"Debug Info Version", i32 3}
!22 = !{i32 1, !"wchar_size", i32 4}
!23 = !{i32 8, !"PIC Level", i32 2}
!24 = !{i32 7, !"PIE Level", i32 2}
!25 = !{i32 7, !"uwtable", i32 2}
!26 = !{i32 7, !"frame-pointer", i32 1}
!27 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!28 = distinct !DISubprogram(name: "validated_add", scope: !2, file: !2, line: 5, type: !29, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !18, retainedNodes: !32)
!29 = !DISubroutineType(types: !30)
!30 = !{!31, !31, !31}
!31 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!32 = !{}
!33 = !DILocalVariable(name: "a", arg: 1, scope: !28, file: !2, line: 5, type: !31)
!34 = !DILocation(line: 5, column: 23, scope: !28)
!35 = !DILocalVariable(name: "b", arg: 2, scope: !28, file: !2, line: 5, type: !31)
!36 = !DILocation(line: 5, column: 30, scope: !28)
!37 = !DILocation(line: 6, column: 9, scope: !38)
!38 = distinct !DILexicalBlock(scope: !28, file: !2, line: 6, column: 9)
!39 = !DILocation(line: 6, column: 11, scope: !38)
!40 = !DILocation(line: 6, column: 15, scope: !38)
!41 = !DILocation(line: 6, column: 18, scope: !38)
!42 = !DILocation(line: 6, column: 20, scope: !38)
!43 = !DILocation(line: 6, column: 9, scope: !28)
!44 = !DILocation(line: 6, column: 27, scope: !38)
!45 = !DILocation(line: 7, column: 9, scope: !46)
!46 = distinct !DILexicalBlock(scope: !28, file: !2, line: 7, column: 9)
!47 = !DILocation(line: 7, column: 11, scope: !46)
!48 = !DILocation(line: 7, column: 15, scope: !46)
!49 = !DILocation(line: 7, column: 18, scope: !46)
!50 = !DILocation(line: 7, column: 20, scope: !46)
!51 = !DILocation(line: 7, column: 9, scope: !28)
!52 = !DILocation(line: 7, column: 27, scope: !46)
!53 = !DILocalVariable(name: "sum", scope: !28, file: !2, line: 8, type: !31)
!54 = !DILocation(line: 8, column: 9, scope: !28)
!55 = !DILocation(line: 8, column: 15, scope: !28)
!56 = !DILocation(line: 8, column: 19, scope: !28)
!57 = !DILocation(line: 8, column: 17, scope: !28)
!58 = !DILocation(line: 10, column: 5, scope: !59)
!59 = distinct !DILexicalBlock(scope: !60, file: !2, line: 10, column: 5)
!60 = distinct !DILexicalBlock(scope: !28, file: !2, line: 10, column: 5)
!61 = !DILocation(line: 10, column: 5, scope: !60)
!62 = !DILocation(line: 11, column: 12, scope: !28)
!63 = !DILocation(line: 11, column: 5, scope: !28)
!64 = !DILocation(line: 12, column: 1, scope: !28)
!65 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 14, type: !66, scopeLine: 14, spFlags: DISPFlagDefinition, unit: !18)
!66 = !DISubroutineType(types: !67)
!67 = !{!31}
!68 = !DILocation(line: 15, column: 12, scope: !65)
!69 = !DILocation(line: 15, column: 5, scope: !65)
