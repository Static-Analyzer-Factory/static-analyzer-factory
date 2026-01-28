; ModuleID = 'tests/fixtures/sources/checker_double_free_exclusive.c'
source_filename = "tests/fixtures/sources/checker_double_free_exclusive.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @exclusive_free(i32 noundef %0) #0 !dbg !13 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !17, metadata !DIExpression()), !dbg !18
  call void @llvm.dbg.declare(metadata ptr %3, metadata !19, metadata !DIExpression()), !dbg !20
  %4 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !21
  store ptr %4, ptr %3, align 8, !dbg !20
  %5 = load i32, ptr %2, align 4, !dbg !22
  %6 = icmp ne i32 %5, 0, !dbg !22
  br i1 %6, label %7, label %9, !dbg !24

7:                                                ; preds = %1
  %8 = load ptr, ptr %3, align 8, !dbg !25
  call void @free(ptr noundef %8) #5, !dbg !26
  br label %11, !dbg !26

9:                                                ; preds = %1
  %10 = load ptr, ptr %3, align 8, !dbg !27
  call void @free(ptr noundef %10) #5, !dbg !28
  br label %11

11:                                               ; preds = %9, %7
  ret void, !dbg !29
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !30 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @exclusive_free(i32 noundef 1), !dbg !33
  ret i32 0, !dbg !34
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #3 = { nounwind "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/fixtures/sources/checker_double_free_exclusive.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "268008275521f6d66acf404b6f830713")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 2}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "exclusive_free", scope: !1, file: !1, line: 5, type: !14, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{null, !4}
!16 = !{}
!17 = !DILocalVariable(name: "cond", arg: 1, scope: !13, file: !1, line: 5, type: !4)
!18 = !DILocation(line: 5, column: 25, scope: !13)
!19 = !DILocalVariable(name: "p", scope: !13, file: !1, line: 6, type: !3)
!20 = !DILocation(line: 6, column: 10, scope: !13)
!21 = !DILocation(line: 6, column: 21, scope: !13)
!22 = !DILocation(line: 7, column: 9, scope: !23)
!23 = distinct !DILexicalBlock(scope: !13, file: !1, line: 7, column: 9)
!24 = !DILocation(line: 7, column: 9, scope: !13)
!25 = !DILocation(line: 8, column: 14, scope: !23)
!26 = !DILocation(line: 8, column: 9, scope: !23)
!27 = !DILocation(line: 10, column: 14, scope: !23)
!28 = !DILocation(line: 10, column: 9, scope: !23)
!29 = !DILocation(line: 12, column: 1, scope: !13)
!30 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 14, type: !31, scopeLine: 14, spFlags: DISPFlagDefinition, unit: !0)
!31 = !DISubroutineType(types: !32)
!32 = !{!4}
!33 = !DILocation(line: 15, column: 5, scope: !30)
!34 = !DILocation(line: 16, column: 5, scope: !30)
