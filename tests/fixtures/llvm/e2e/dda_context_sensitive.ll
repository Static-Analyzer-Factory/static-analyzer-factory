; ModuleID = 'dda_context_sensitive.c'
source_filename = "dda_context_sensitive.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @get_ptr(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !16, metadata !DIExpression()), !dbg !17
  %3 = load ptr, ptr %2, align 8, !dbg !18
  ret ptr %3, !dbg !19
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !20 {
  %1 = alloca i32, align 4
  %2 = alloca [10 x i32], align 4
  %3 = alloca [10 x i32], align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !23, metadata !DIExpression()), !dbg !27
  call void @llvm.dbg.declare(metadata ptr %3, metadata !28, metadata !DIExpression()), !dbg !29
  call void @llvm.dbg.declare(metadata ptr %4, metadata !30, metadata !DIExpression()), !dbg !31
  %6 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !32
  %7 = call ptr @get_ptr(ptr noundef %6), !dbg !33
  store ptr %7, ptr %4, align 8, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %5, metadata !34, metadata !DIExpression()), !dbg !35
  %8 = getelementptr inbounds [10 x i32], ptr %3, i64 0, i64 0, !dbg !36
  %9 = call ptr @get_ptr(ptr noundef %8), !dbg !37
  store ptr %9, ptr %5, align 8, !dbg !35
  %10 = load ptr, ptr %4, align 8, !dbg !38
  store i32 1, ptr %10, align 4, !dbg !39
  %11 = load ptr, ptr %5, align 8, !dbg !40
  store i32 2, ptr %11, align 4, !dbg !41
  %12 = load ptr, ptr %4, align 8, !dbg !42
  %13 = load i32, ptr %12, align 4, !dbg !43
  %14 = load ptr, ptr %5, align 8, !dbg !44
  %15 = load i32, ptr %14, align 4, !dbg !45
  %16 = add nsw i32 %13, %15, !dbg !46
  ret i32 %16, !dbg !47
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_context_sensitive.c", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "fb6af08aa2381115ff2c709f734d46b5")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "get_ptr", scope: !1, file: !1, line: 7, type: !11, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DISubroutineType(types: !12)
!12 = !{!13, !13}
!13 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "arr", arg: 1, scope: !10, file: !1, line: 7, type: !13)
!17 = !DILocation(line: 7, column: 19, scope: !10)
!18 = !DILocation(line: 8, column: 12, scope: !10)
!19 = !DILocation(line: 8, column: 5, scope: !10)
!20 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 11, type: !21, scopeLine: 11, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!21 = !DISubroutineType(types: !22)
!22 = !{!14}
!23 = !DILocalVariable(name: "arr1", scope: !20, file: !1, line: 12, type: !24)
!24 = !DICompositeType(tag: DW_TAG_array_type, baseType: !14, size: 320, elements: !25)
!25 = !{!26}
!26 = !DISubrange(count: 10)
!27 = !DILocation(line: 12, column: 9, scope: !20)
!28 = !DILocalVariable(name: "arr2", scope: !20, file: !1, line: 13, type: !24)
!29 = !DILocation(line: 13, column: 9, scope: !20)
!30 = !DILocalVariable(name: "p1", scope: !20, file: !1, line: 15, type: !13)
!31 = !DILocation(line: 15, column: 10, scope: !20)
!32 = !DILocation(line: 15, column: 23, scope: !20)
!33 = !DILocation(line: 15, column: 15, scope: !20)
!34 = !DILocalVariable(name: "p2", scope: !20, file: !1, line: 16, type: !13)
!35 = !DILocation(line: 16, column: 10, scope: !20)
!36 = !DILocation(line: 16, column: 23, scope: !20)
!37 = !DILocation(line: 16, column: 15, scope: !20)
!38 = !DILocation(line: 18, column: 6, scope: !20)
!39 = !DILocation(line: 18, column: 9, scope: !20)
!40 = !DILocation(line: 19, column: 6, scope: !20)
!41 = !DILocation(line: 19, column: 9, scope: !20)
!42 = !DILocation(line: 21, column: 13, scope: !20)
!43 = !DILocation(line: 21, column: 12, scope: !20)
!44 = !DILocation(line: 21, column: 19, scope: !20)
!45 = !DILocation(line: 21, column: 18, scope: !20)
!46 = !DILocation(line: 21, column: 16, scope: !20)
!47 = !DILocation(line: 21, column: 5, scope: !20)
