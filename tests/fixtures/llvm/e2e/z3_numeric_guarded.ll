; ModuleID = 'tests/programs/c/z3_numeric_guarded.c'
source_filename = "tests/programs/c/z3_numeric_guarded.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @fill_array(ptr noundef %0, i32 noundef %1) #0 !dbg !10 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !16, metadata !DIExpression()), !dbg !17
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !18, metadata !DIExpression()), !dbg !19
  call void @llvm.dbg.declare(metadata ptr %5, metadata !20, metadata !DIExpression()), !dbg !22
  store i32 0, ptr %5, align 4, !dbg !22
  br label %6, !dbg !23

6:                                                ; preds = %21, %2
  %7 = load i32, ptr %5, align 4, !dbg !24
  %8 = load i32, ptr %4, align 4, !dbg !26
  %9 = icmp slt i32 %7, %8, !dbg !27
  br i1 %9, label %10, label %24, !dbg !28

10:                                               ; preds = %6
  %11 = load i32, ptr %5, align 4, !dbg !29
  %12 = icmp slt i32 %11, 10, !dbg !32
  br i1 %12, label %13, label %20, !dbg !33

13:                                               ; preds = %10
  %14 = load i32, ptr %5, align 4, !dbg !34
  %15 = mul nsw i32 %14, 2, !dbg !36
  %16 = load ptr, ptr %3, align 8, !dbg !37
  %17 = load i32, ptr %5, align 4, !dbg !38
  %18 = sext i32 %17 to i64, !dbg !37
  %19 = getelementptr inbounds i32, ptr %16, i64 %18, !dbg !37
  store i32 %15, ptr %19, align 4, !dbg !39
  br label %20, !dbg !40

20:                                               ; preds = %13, %10
  br label %21, !dbg !41

21:                                               ; preds = %20
  %22 = load i32, ptr %5, align 4, !dbg !42
  %23 = add nsw i32 %22, 1, !dbg !42
  store i32 %23, ptr %5, align 4, !dbg !42
  br label %6, !dbg !43, !llvm.loop !44

24:                                               ; preds = %6
  ret void, !dbg !47
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !48 {
  %1 = alloca i32, align 4
  %2 = alloca [10 x i32], align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !51, metadata !DIExpression()), !dbg !55
  %3 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !56
  call void @fill_array(ptr noundef %3, i32 noundef 10), !dbg !57
  %4 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !58
  %5 = load i32, ptr %4, align 4, !dbg !58
  ret i32 %5, !dbg !59
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/z3_numeric_guarded.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "7b6a855a8fd396d4157b6183a3b065e4")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "fill_array", scope: !1, file: !1, line: 5, type: !11, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DISubroutineType(types: !12)
!12 = !{null, !13, !14}
!13 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "buf", arg: 1, scope: !10, file: !1, line: 5, type: !13)
!17 = !DILocation(line: 5, column: 22, scope: !10)
!18 = !DILocalVariable(name: "n", arg: 2, scope: !10, file: !1, line: 5, type: !14)
!19 = !DILocation(line: 5, column: 31, scope: !10)
!20 = !DILocalVariable(name: "i", scope: !21, file: !1, line: 6, type: !14)
!21 = distinct !DILexicalBlock(scope: !10, file: !1, line: 6, column: 5)
!22 = !DILocation(line: 6, column: 14, scope: !21)
!23 = !DILocation(line: 6, column: 10, scope: !21)
!24 = !DILocation(line: 6, column: 21, scope: !25)
!25 = distinct !DILexicalBlock(scope: !21, file: !1, line: 6, column: 5)
!26 = !DILocation(line: 6, column: 25, scope: !25)
!27 = !DILocation(line: 6, column: 23, scope: !25)
!28 = !DILocation(line: 6, column: 5, scope: !21)
!29 = !DILocation(line: 9, column: 13, scope: !30)
!30 = distinct !DILexicalBlock(scope: !31, file: !1, line: 9, column: 13)
!31 = distinct !DILexicalBlock(scope: !25, file: !1, line: 6, column: 33)
!32 = !DILocation(line: 9, column: 15, scope: !30)
!33 = !DILocation(line: 9, column: 13, scope: !31)
!34 = !DILocation(line: 10, column: 22, scope: !35)
!35 = distinct !DILexicalBlock(scope: !30, file: !1, line: 9, column: 21)
!36 = !DILocation(line: 10, column: 24, scope: !35)
!37 = !DILocation(line: 10, column: 13, scope: !35)
!38 = !DILocation(line: 10, column: 17, scope: !35)
!39 = !DILocation(line: 10, column: 20, scope: !35)
!40 = !DILocation(line: 11, column: 9, scope: !35)
!41 = !DILocation(line: 12, column: 5, scope: !31)
!42 = !DILocation(line: 6, column: 29, scope: !25)
!43 = !DILocation(line: 6, column: 5, scope: !25)
!44 = distinct !{!44, !28, !45, !46}
!45 = !DILocation(line: 12, column: 5, scope: !21)
!46 = !{!"llvm.loop.mustprogress"}
!47 = !DILocation(line: 13, column: 1, scope: !10)
!48 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 15, type: !49, scopeLine: 15, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!49 = !DISubroutineType(types: !50)
!50 = !{!14}
!51 = !DILocalVariable(name: "buf", scope: !48, file: !1, line: 16, type: !52)
!52 = !DICompositeType(tag: DW_TAG_array_type, baseType: !14, size: 320, elements: !53)
!53 = !{!54}
!54 = !DISubrange(count: 10)
!55 = !DILocation(line: 16, column: 9, scope: !48)
!56 = !DILocation(line: 17, column: 16, scope: !48)
!57 = !DILocation(line: 17, column: 5, scope: !48)
!58 = !DILocation(line: 18, column: 12, scope: !48)
!59 = !DILocation(line: 18, column: 5, scope: !48)
