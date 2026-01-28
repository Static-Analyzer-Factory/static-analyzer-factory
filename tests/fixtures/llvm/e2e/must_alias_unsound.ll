; ModuleID = 'must_alias_unsound.c'
source_filename = "must_alias_unsound.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrapper() #0 !dbg !13 {
  %1 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !17
  ret ptr %1, !dbg !18
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !19 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !23, metadata !DIExpression()), !dbg !24
  %5 = call ptr @wrapper(), !dbg !25
  store ptr %5, ptr %2, align 8, !dbg !24
  call void @llvm.dbg.declare(metadata ptr %3, metadata !26, metadata !DIExpression()), !dbg !27
  %6 = call ptr @wrapper(), !dbg !28
  store ptr %6, ptr %3, align 8, !dbg !27
  %7 = load ptr, ptr %2, align 8, !dbg !29
  %8 = load ptr, ptr %3, align 8, !dbg !30
  call void @MUSTALIAS(ptr noundef %7, ptr noundef %8), !dbg !31
  %9 = load ptr, ptr %2, align 8, !dbg !32
  %10 = load ptr, ptr %3, align 8, !dbg !33
  call void @MAYALIAS(ptr noundef %9, ptr noundef %10), !dbg !34
  call void @llvm.dbg.declare(metadata ptr %4, metadata !35, metadata !DIExpression()), !dbg !36
  store i32 0, ptr %4, align 4, !dbg !36
  %11 = load ptr, ptr %2, align 8, !dbg !37
  call void @NOALIAS(ptr noundef %4, ptr noundef %11), !dbg !38
  %12 = load ptr, ptr %2, align 8, !dbg !39
  store i32 42, ptr %12, align 4, !dbg !40
  %13 = load ptr, ptr %3, align 8, !dbg !41
  store i32 99, ptr %13, align 4, !dbg !42
  %14 = load ptr, ptr %2, align 8, !dbg !43
  %15 = load i32, ptr %14, align 4, !dbg !44
  ret i32 %15, !dbg !45
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #2

declare void @MUSTALIAS(ptr noundef, ptr noundef) #3

declare void @MAYALIAS(ptr noundef, ptr noundef) #3

declare void @NOALIAS(ptr noundef, ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nounwind allocsize(0) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #3 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #4 = { nounwind allocsize(0) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.8 (11~20.04.2)", isOptimized: false, flags: "/usr/lib/llvm-18/bin/clang -S -emit-llvm -O0 -g -o must_alias_unsound.ll must_alias_unsound.c", runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "must_alias_unsound.c", directory: "/home/weili/SAF-tools/saf-analysis-docs/soundness-issues/tests", checksumkind: CSK_MD5, checksum: "b6002021d93853add57135d3daabda94")
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
!12 = !{!"Ubuntu clang version 18.1.8 (11~20.04.2)"}
!13 = distinct !DISubprogram(name: "wrapper", scope: !1, file: !1, line: 46, type: !14, scopeLine: 46, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!14 = !DISubroutineType(types: !15)
!15 = !{!16}
!16 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!17 = !DILocation(line: 47, column: 12, scope: !13)
!18 = !DILocation(line: 47, column: 5, scope: !13)
!19 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 50, type: !20, scopeLine: 50, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!20 = !DISubroutineType(types: !21)
!21 = !{!4}
!22 = !{}
!23 = !DILocalVariable(name: "p", scope: !19, file: !1, line: 51, type: !3)
!24 = !DILocation(line: 51, column: 10, scope: !19)
!25 = !DILocation(line: 51, column: 21, scope: !19)
!26 = !DILocalVariable(name: "q", scope: !19, file: !1, line: 52, type: !3)
!27 = !DILocation(line: 52, column: 10, scope: !19)
!28 = !DILocation(line: 52, column: 21, scope: !19)
!29 = !DILocation(line: 70, column: 15, scope: !19)
!30 = !DILocation(line: 70, column: 18, scope: !19)
!31 = !DILocation(line: 70, column: 5, scope: !19)
!32 = !DILocation(line: 73, column: 14, scope: !19)
!33 = !DILocation(line: 73, column: 17, scope: !19)
!34 = !DILocation(line: 73, column: 5, scope: !19)
!35 = !DILocalVariable(name: "a", scope: !19, file: !1, line: 76, type: !4)
!36 = !DILocation(line: 76, column: 9, scope: !19)
!37 = !DILocation(line: 77, column: 17, scope: !19)
!38 = !DILocation(line: 77, column: 5, scope: !19)
!39 = !DILocation(line: 79, column: 6, scope: !19)
!40 = !DILocation(line: 79, column: 8, scope: !19)
!41 = !DILocation(line: 80, column: 6, scope: !19)
!42 = !DILocation(line: 80, column: 8, scope: !19)
!43 = !DILocation(line: 88, column: 13, scope: !19)
!44 = !DILocation(line: 88, column: 12, scope: !19)
!45 = !DILocation(line: 88, column: 5, scope: !19)
