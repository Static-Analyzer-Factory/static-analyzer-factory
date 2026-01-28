; ModuleID = '/workspace/tests/programs/c/buffer_overflow.c'
source_filename = "/workspace/tests/programs/c/buffer_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !13 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  %4 = call noalias ptr @malloc(i64 noundef 20) #4, !dbg !20
  store ptr %4, ptr %2, align 8, !dbg !19
  call void @llvm.dbg.declare(metadata ptr %3, metadata !21, metadata !DIExpression()), !dbg !23
  store i32 0, ptr %3, align 4, !dbg !23
  br label %5, !dbg !24

5:                                                ; preds = %14, %0
  %6 = load i32, ptr %3, align 4, !dbg !25
  %7 = icmp slt i32 %6, 10, !dbg !27
  br i1 %7, label %8, label %17, !dbg !28

8:                                                ; preds = %5
  %9 = load i32, ptr %3, align 4, !dbg !29
  %10 = load ptr, ptr %2, align 8, !dbg !31
  %11 = load i32, ptr %3, align 4, !dbg !32
  %12 = sext i32 %11 to i64, !dbg !31
  %13 = getelementptr inbounds i32, ptr %10, i64 %12, !dbg !31
  store i32 %9, ptr %13, align 4, !dbg !33
  br label %14, !dbg !34

14:                                               ; preds = %8
  %15 = load i32, ptr %3, align 4, !dbg !35
  %16 = add nsw i32 %15, 1, !dbg !35
  store i32 %16, ptr %3, align 4, !dbg !35
  br label %5, !dbg !36, !llvm.loop !37

17:                                               ; preds = %5
  %18 = load ptr, ptr %2, align 8, !dbg !40
  call void @free(ptr noundef %18) #5, !dbg !41
  ret i32 0, !dbg !42
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/buffer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3aacef2445f5ad3a3deb6f3fb96ef65b")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 1}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "main", scope: !14, file: !14, line: 7, type: !15, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!14 = !DIFile(filename: "c/buffer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "3aacef2445f5ad3a3deb6f3fb96ef65b")
!15 = !DISubroutineType(types: !16)
!16 = !{!4}
!17 = !{}
!18 = !DILocalVariable(name: "buf", scope: !13, file: !14, line: 8, type: !3)
!19 = !DILocation(line: 8, column: 10, scope: !13)
!20 = !DILocation(line: 8, column: 23, scope: !13)
!21 = !DILocalVariable(name: "i", scope: !22, file: !14, line: 9, type: !4)
!22 = distinct !DILexicalBlock(scope: !13, file: !14, line: 9, column: 5)
!23 = !DILocation(line: 9, column: 14, scope: !22)
!24 = !DILocation(line: 9, column: 10, scope: !22)
!25 = !DILocation(line: 9, column: 21, scope: !26)
!26 = distinct !DILexicalBlock(scope: !22, file: !14, line: 9, column: 5)
!27 = !DILocation(line: 9, column: 23, scope: !26)
!28 = !DILocation(line: 9, column: 5, scope: !22)
!29 = !DILocation(line: 10, column: 18, scope: !30)
!30 = distinct !DILexicalBlock(scope: !26, file: !14, line: 9, column: 34)
!31 = !DILocation(line: 10, column: 9, scope: !30)
!32 = !DILocation(line: 10, column: 13, scope: !30)
!33 = !DILocation(line: 10, column: 16, scope: !30)
!34 = !DILocation(line: 11, column: 5, scope: !30)
!35 = !DILocation(line: 9, column: 30, scope: !26)
!36 = !DILocation(line: 9, column: 5, scope: !26)
!37 = distinct !{!37, !28, !38, !39}
!38 = !DILocation(line: 11, column: 5, scope: !22)
!39 = !{!"llvm.loop.mustprogress"}
!40 = !DILocation(line: 12, column: 10, scope: !13)
!41 = !DILocation(line: 12, column: 5, scope: !13)
!42 = !DILocation(line: 13, column: 5, scope: !13)
