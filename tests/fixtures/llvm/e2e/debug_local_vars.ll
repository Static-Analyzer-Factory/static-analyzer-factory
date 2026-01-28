; ModuleID = 'tests/programs/c/debug_local_vars.c'
source_filename = "tests/programs/c/debug_local_vars.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @add(i32 noundef %0, i32 noundef %1) #0 !dbg !13 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  call void @llvm.dbg.declare(metadata ptr %5, metadata !21, metadata !DIExpression()), !dbg !22
  %6 = load i32, ptr %3, align 4, !dbg !23
  %7 = load i32, ptr %4, align 4, !dbg !24
  %8 = add nsw i32 %6, %7, !dbg !25
  store i32 %8, ptr %5, align 4, !dbg !22
  %9 = load i32, ptr %5, align 4, !dbg !26
  ret i32 %9, !dbg !27
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !28 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !31, metadata !DIExpression()), !dbg !32
  store i32 10, ptr %2, align 4, !dbg !32
  call void @llvm.dbg.declare(metadata ptr %3, metadata !33, metadata !DIExpression()), !dbg !34
  store i32 20, ptr %3, align 4, !dbg !34
  call void @llvm.dbg.declare(metadata ptr %4, metadata !35, metadata !DIExpression()), !dbg !36
  %6 = call noalias ptr @malloc(i64 noundef 4) #4, !dbg !37
  store ptr %6, ptr %4, align 8, !dbg !36
  call void @llvm.dbg.declare(metadata ptr %5, metadata !38, metadata !DIExpression()), !dbg !39
  %7 = load i32, ptr %2, align 4, !dbg !40
  %8 = load i32, ptr %3, align 4, !dbg !41
  %9 = call i32 @add(i32 noundef %7, i32 noundef %8), !dbg !42
  store i32 %9, ptr %5, align 4, !dbg !39
  %10 = load i32, ptr %5, align 4, !dbg !43
  %11 = load ptr, ptr %4, align 8, !dbg !44
  store i32 %10, ptr %11, align 4, !dbg !45
  %12 = load ptr, ptr %4, align 8, !dbg !46
  call void @free(ptr noundef %12) #5, !dbg !47
  ret i32 0, !dbg !48
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

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
!1 = !DIFile(filename: "tests/programs/c/debug_local_vars.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "249c4c4e6f45ef1597ddf3afb7cd8b90")
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
!13 = distinct !DISubprogram(name: "add", scope: !1, file: !1, line: 10, type: !14, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{!4, !4, !4}
!16 = !{}
!17 = !DILocalVariable(name: "a", arg: 1, scope: !13, file: !1, line: 10, type: !4)
!18 = !DILocation(line: 10, column: 13, scope: !13)
!19 = !DILocalVariable(name: "b", arg: 2, scope: !13, file: !1, line: 10, type: !4)
!20 = !DILocation(line: 10, column: 20, scope: !13)
!21 = !DILocalVariable(name: "result", scope: !13, file: !1, line: 11, type: !4)
!22 = !DILocation(line: 11, column: 9, scope: !13)
!23 = !DILocation(line: 11, column: 18, scope: !13)
!24 = !DILocation(line: 11, column: 22, scope: !13)
!25 = !DILocation(line: 11, column: 20, scope: !13)
!26 = !DILocation(line: 12, column: 12, scope: !13)
!27 = !DILocation(line: 12, column: 5, scope: !13)
!28 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 15, type: !29, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!29 = !DISubroutineType(types: !30)
!30 = !{!4}
!31 = !DILocalVariable(name: "x", scope: !28, file: !1, line: 16, type: !4)
!32 = !DILocation(line: 16, column: 9, scope: !28)
!33 = !DILocalVariable(name: "y", scope: !28, file: !1, line: 17, type: !4)
!34 = !DILocation(line: 17, column: 9, scope: !28)
!35 = !DILocalVariable(name: "ptr", scope: !28, file: !1, line: 18, type: !3)
!36 = !DILocation(line: 18, column: 10, scope: !28)
!37 = !DILocation(line: 18, column: 23, scope: !28)
!38 = !DILocalVariable(name: "sum", scope: !28, file: !1, line: 19, type: !4)
!39 = !DILocation(line: 19, column: 9, scope: !28)
!40 = !DILocation(line: 19, column: 19, scope: !28)
!41 = !DILocation(line: 19, column: 22, scope: !28)
!42 = !DILocation(line: 19, column: 15, scope: !28)
!43 = !DILocation(line: 20, column: 12, scope: !28)
!44 = !DILocation(line: 20, column: 6, scope: !28)
!45 = !DILocation(line: 20, column: 10, scope: !28)
!46 = !DILocation(line: 21, column: 10, scope: !28)
!47 = !DILocation(line: 21, column: 5, scope: !28)
!48 = !DILocation(line: 22, column: 5, scope: !28)
