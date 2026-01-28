; ModuleID = 'tests/programs/c/z3_alias_disjoint.c'
source_filename = "tests/programs/c/z3_alias_disjoint.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !10 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca ptr, align 8
  %9 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !18, metadata !DIExpression()), !dbg !19
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !20, metadata !DIExpression()), !dbg !21
  call void @llvm.dbg.declare(metadata ptr %6, metadata !22, metadata !DIExpression()), !dbg !23
  store i32 1, ptr %6, align 4, !dbg !23
  call void @llvm.dbg.declare(metadata ptr %7, metadata !24, metadata !DIExpression()), !dbg !25
  store i32 2, ptr %7, align 4, !dbg !25
  call void @llvm.dbg.declare(metadata ptr %8, metadata !26, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %9, metadata !29, metadata !DIExpression()), !dbg !30
  %10 = load i32, ptr %4, align 4, !dbg !31
  %11 = icmp sgt i32 %10, 1, !dbg !33
  br i1 %11, label %12, label %13, !dbg !34

12:                                               ; preds = %2
  store ptr %6, ptr %8, align 8, !dbg !35
  store ptr %7, ptr %9, align 8, !dbg !37
  br label %14, !dbg !38

13:                                               ; preds = %2
  store ptr %7, ptr %8, align 8, !dbg !39
  store ptr %6, ptr %9, align 8, !dbg !41
  br label %14

14:                                               ; preds = %13, %12
  %15 = load ptr, ptr %8, align 8, !dbg !42
  %16 = load i32, ptr %15, align 4, !dbg !43
  %17 = load ptr, ptr %9, align 8, !dbg !44
  %18 = load i32, ptr %17, align 4, !dbg !45
  %19 = add nsw i32 %16, %18, !dbg !46
  ret i32 %19, !dbg !47
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/z3_alias_disjoint.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "b9f76a2793182b7a18426cf3bb1d6ed6")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 5, type: !11, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!11 = !DISubroutineType(types: !12)
!12 = !{!13, !13, !14}
!13 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!16 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!17 = !{}
!18 = !DILocalVariable(name: "argc", arg: 1, scope: !10, file: !1, line: 5, type: !13)
!19 = !DILocation(line: 5, column: 14, scope: !10)
!20 = !DILocalVariable(name: "argv", arg: 2, scope: !10, file: !1, line: 5, type: !14)
!21 = !DILocation(line: 5, column: 27, scope: !10)
!22 = !DILocalVariable(name: "a", scope: !10, file: !1, line: 6, type: !13)
!23 = !DILocation(line: 6, column: 9, scope: !10)
!24 = !DILocalVariable(name: "b", scope: !10, file: !1, line: 6, type: !13)
!25 = !DILocation(line: 6, column: 16, scope: !10)
!26 = !DILocalVariable(name: "p", scope: !10, file: !1, line: 7, type: !27)
!27 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !13, size: 64)
!28 = !DILocation(line: 7, column: 10, scope: !10)
!29 = !DILocalVariable(name: "q", scope: !10, file: !1, line: 7, type: !27)
!30 = !DILocation(line: 7, column: 14, scope: !10)
!31 = !DILocation(line: 9, column: 9, scope: !32)
!32 = distinct !DILexicalBlock(scope: !10, file: !1, line: 9, column: 9)
!33 = !DILocation(line: 9, column: 14, scope: !32)
!34 = !DILocation(line: 9, column: 9, scope: !10)
!35 = !DILocation(line: 10, column: 11, scope: !36)
!36 = distinct !DILexicalBlock(scope: !32, file: !1, line: 9, column: 19)
!37 = !DILocation(line: 11, column: 11, scope: !36)
!38 = !DILocation(line: 12, column: 5, scope: !36)
!39 = !DILocation(line: 13, column: 11, scope: !40)
!40 = distinct !DILexicalBlock(scope: !32, file: !1, line: 12, column: 12)
!41 = !DILocation(line: 14, column: 11, scope: !40)
!42 = !DILocation(line: 18, column: 13, scope: !10)
!43 = !DILocation(line: 18, column: 12, scope: !10)
!44 = !DILocation(line: 18, column: 18, scope: !10)
!45 = !DILocation(line: 18, column: 17, scope: !10)
!46 = !DILocation(line: 18, column: 15, scope: !10)
!47 = !DILocation(line: 18, column: 5, scope: !10)
