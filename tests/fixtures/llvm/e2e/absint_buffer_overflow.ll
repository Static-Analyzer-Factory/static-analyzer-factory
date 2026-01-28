; ModuleID = '/workspace/tests/programs/c/absint_buffer_overflow.c'
source_filename = "/workspace/tests/programs/c/absint_buffer_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @safe_access(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !17, metadata !DIExpression()), !dbg !18
  call void @llvm.dbg.declare(metadata ptr %3, metadata !19, metadata !DIExpression()), !dbg !21
  store i32 0, ptr %3, align 4, !dbg !21
  br label %4, !dbg !22

4:                                                ; preds = %14, %1
  %5 = load i32, ptr %3, align 4, !dbg !23
  %6 = icmp slt i32 %5, 10, !dbg !25
  br i1 %6, label %7, label %17, !dbg !26

7:                                                ; preds = %4
  %8 = load i32, ptr %3, align 4, !dbg !27
  %9 = mul nsw i32 %8, 2, !dbg !29
  %10 = load ptr, ptr %2, align 8, !dbg !30
  %11 = load i32, ptr %3, align 4, !dbg !31
  %12 = sext i32 %11 to i64, !dbg !30
  %13 = getelementptr inbounds i32, ptr %10, i64 %12, !dbg !30
  store i32 %9, ptr %13, align 4, !dbg !32
  br label %14, !dbg !33

14:                                               ; preds = %7
  %15 = load i32, ptr %3, align 4, !dbg !34
  %16 = add nsw i32 %15, 1, !dbg !34
  store i32 %16, ptr %3, align 4, !dbg !34
  br label %4, !dbg !35, !llvm.loop !36

17:                                               ; preds = %4
  ret void, !dbg !39
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @off_by_one(ptr noundef %0) #0 !dbg !40 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !41, metadata !DIExpression()), !dbg !42
  call void @llvm.dbg.declare(metadata ptr %3, metadata !43, metadata !DIExpression()), !dbg !45
  store i32 0, ptr %3, align 4, !dbg !45
  br label %4, !dbg !46

4:                                                ; preds = %13, %1
  %5 = load i32, ptr %3, align 4, !dbg !47
  %6 = icmp sle i32 %5, 10, !dbg !49
  br i1 %6, label %7, label %16, !dbg !50

7:                                                ; preds = %4
  %8 = load i32, ptr %3, align 4, !dbg !51
  %9 = load ptr, ptr %2, align 8, !dbg !53
  %10 = load i32, ptr %3, align 4, !dbg !54
  %11 = sext i32 %10 to i64, !dbg !53
  %12 = getelementptr inbounds i32, ptr %9, i64 %11, !dbg !53
  store i32 %8, ptr %12, align 4, !dbg !55
  br label %13, !dbg !56

13:                                               ; preds = %7
  %14 = load i32, ptr %3, align 4, !dbg !57
  %15 = add nsw i32 %14, 1, !dbg !57
  store i32 %15, ptr %3, align 4, !dbg !57
  br label %4, !dbg !58, !llvm.loop !59

16:                                               ; preds = %4
  ret void, !dbg !61
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !62 {
  %1 = alloca i32, align 4
  %2 = alloca [10 x i32], align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !65, metadata !DIExpression()), !dbg !69
  %3 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !70
  call void @safe_access(ptr noundef %3), !dbg !71
  %4 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !72
  call void @off_by_one(ptr noundef %4), !dbg !73
  %5 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !74
  %6 = load i32, ptr %5, align 4, !dbg !74
  ret i32 %6, !dbg !75
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/absint_buffer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "6ee455596e68f45df0d6aa54f6b85816")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "safe_access", scope: !11, file: !11, line: 12, type: !12, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!11 = !DIFile(filename: "c/absint_buffer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "6ee455596e68f45df0d6aa54f6b85816")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!16 = !{}
!17 = !DILocalVariable(name: "buf", arg: 1, scope: !10, file: !11, line: 12, type: !14)
!18 = !DILocation(line: 12, column: 23, scope: !10)
!19 = !DILocalVariable(name: "i", scope: !20, file: !11, line: 14, type: !15)
!20 = distinct !DILexicalBlock(scope: !10, file: !11, line: 14, column: 5)
!21 = !DILocation(line: 14, column: 14, scope: !20)
!22 = !DILocation(line: 14, column: 10, scope: !20)
!23 = !DILocation(line: 14, column: 21, scope: !24)
!24 = distinct !DILexicalBlock(scope: !20, file: !11, line: 14, column: 5)
!25 = !DILocation(line: 14, column: 23, scope: !24)
!26 = !DILocation(line: 14, column: 5, scope: !20)
!27 = !DILocation(line: 15, column: 18, scope: !28)
!28 = distinct !DILexicalBlock(scope: !24, file: !11, line: 14, column: 40)
!29 = !DILocation(line: 15, column: 20, scope: !28)
!30 = !DILocation(line: 15, column: 9, scope: !28)
!31 = !DILocation(line: 15, column: 13, scope: !28)
!32 = !DILocation(line: 15, column: 16, scope: !28)
!33 = !DILocation(line: 16, column: 5, scope: !28)
!34 = !DILocation(line: 14, column: 36, scope: !24)
!35 = !DILocation(line: 14, column: 5, scope: !24)
!36 = distinct !{!36, !26, !37, !38}
!37 = !DILocation(line: 16, column: 5, scope: !20)
!38 = !{!"llvm.loop.mustprogress"}
!39 = !DILocation(line: 17, column: 1, scope: !10)
!40 = distinct !DISubprogram(name: "off_by_one", scope: !11, file: !11, line: 19, type: !12, scopeLine: 19, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!41 = !DILocalVariable(name: "buf", arg: 1, scope: !40, file: !11, line: 19, type: !14)
!42 = !DILocation(line: 19, column: 22, scope: !40)
!43 = !DILocalVariable(name: "i", scope: !44, file: !11, line: 21, type: !15)
!44 = distinct !DILexicalBlock(scope: !40, file: !11, line: 21, column: 5)
!45 = !DILocation(line: 21, column: 14, scope: !44)
!46 = !DILocation(line: 21, column: 10, scope: !44)
!47 = !DILocation(line: 21, column: 21, scope: !48)
!48 = distinct !DILexicalBlock(scope: !44, file: !11, line: 21, column: 5)
!49 = !DILocation(line: 21, column: 23, scope: !48)
!50 = !DILocation(line: 21, column: 5, scope: !44)
!51 = !DILocation(line: 22, column: 18, scope: !52)
!52 = distinct !DILexicalBlock(scope: !48, file: !11, line: 21, column: 41)
!53 = !DILocation(line: 22, column: 9, scope: !52)
!54 = !DILocation(line: 22, column: 13, scope: !52)
!55 = !DILocation(line: 22, column: 16, scope: !52)
!56 = !DILocation(line: 23, column: 5, scope: !52)
!57 = !DILocation(line: 21, column: 37, scope: !48)
!58 = !DILocation(line: 21, column: 5, scope: !48)
!59 = distinct !{!59, !50, !60, !38}
!60 = !DILocation(line: 23, column: 5, scope: !44)
!61 = !DILocation(line: 24, column: 1, scope: !40)
!62 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 26, type: !63, scopeLine: 26, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!63 = !DISubroutineType(types: !64)
!64 = !{!15}
!65 = !DILocalVariable(name: "buf", scope: !62, file: !11, line: 27, type: !66)
!66 = !DICompositeType(tag: DW_TAG_array_type, baseType: !15, size: 320, elements: !67)
!67 = !{!68}
!68 = !DISubrange(count: 10)
!69 = !DILocation(line: 27, column: 9, scope: !62)
!70 = !DILocation(line: 28, column: 17, scope: !62)
!71 = !DILocation(line: 28, column: 5, scope: !62)
!72 = !DILocation(line: 29, column: 16, scope: !62)
!73 = !DILocation(line: 29, column: 5, scope: !62)
!74 = !DILocation(line: 30, column: 12, scope: !62)
!75 = !DILocation(line: 30, column: 5, scope: !62)
