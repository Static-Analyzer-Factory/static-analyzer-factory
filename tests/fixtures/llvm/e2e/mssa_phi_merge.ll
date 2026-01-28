; ModuleID = '/workspace/tests/programs/c/mssa_phi_merge.c'
source_filename = "/workspace/tests/programs/c/mssa_phi_merge.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test(ptr noundef %0, i32 noundef %1) #0 !dbg !10 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  %6 = load i32, ptr %4, align 4, !dbg !21
  %7 = icmp ne i32 %6, 0, !dbg !21
  br i1 %7, label %8, label %10, !dbg !23

8:                                                ; preds = %2
  %9 = load ptr, ptr %3, align 8, !dbg !24
  store i32 1, ptr %9, align 4, !dbg !26
  br label %12, !dbg !27

10:                                               ; preds = %2
  %11 = load ptr, ptr %3, align 8, !dbg !28
  store i32 2, ptr %11, align 4, !dbg !30
  br label %12

12:                                               ; preds = %10, %8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !31, metadata !DIExpression()), !dbg !32
  %13 = load ptr, ptr %3, align 8, !dbg !33
  %14 = load i32, ptr %13, align 4, !dbg !34
  store i32 %14, ptr %5, align 4, !dbg !32
  %15 = load i32, ptr %5, align 4, !dbg !35
  call void @sink(i32 noundef %15), !dbg !36
  ret void, !dbg !37
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare void @sink(i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !38 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !41, metadata !DIExpression()), !dbg !42
  call void @test(ptr noundef %2, i32 noundef 1), !dbg !43
  ret i32 0, !dbg !44
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/mssa_phi_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "cd6b34bdb1b80fbd32ae7e7219e5bae9")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 11, type: !12, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!11 = !DIFile(filename: "c/mssa_phi_merge.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "cd6b34bdb1b80fbd32ae7e7219e5bae9")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14, !15}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!16 = !{}
!17 = !DILocalVariable(name: "p", arg: 1, scope: !10, file: !11, line: 11, type: !14)
!18 = !DILocation(line: 11, column: 16, scope: !10)
!19 = !DILocalVariable(name: "cond", arg: 2, scope: !10, file: !11, line: 11, type: !15)
!20 = !DILocation(line: 11, column: 23, scope: !10)
!21 = !DILocation(line: 12, column: 9, scope: !22)
!22 = distinct !DILexicalBlock(scope: !10, file: !11, line: 12, column: 9)
!23 = !DILocation(line: 12, column: 9, scope: !10)
!24 = !DILocation(line: 13, column: 10, scope: !25)
!25 = distinct !DILexicalBlock(scope: !22, file: !11, line: 12, column: 15)
!26 = !DILocation(line: 13, column: 12, scope: !25)
!27 = !DILocation(line: 14, column: 5, scope: !25)
!28 = !DILocation(line: 15, column: 10, scope: !29)
!29 = distinct !DILexicalBlock(scope: !22, file: !11, line: 14, column: 12)
!30 = !DILocation(line: 15, column: 12, scope: !29)
!31 = !DILocalVariable(name: "x", scope: !10, file: !11, line: 17, type: !15)
!32 = !DILocation(line: 17, column: 9, scope: !10)
!33 = !DILocation(line: 17, column: 14, scope: !10)
!34 = !DILocation(line: 17, column: 13, scope: !10)
!35 = !DILocation(line: 18, column: 10, scope: !10)
!36 = !DILocation(line: 18, column: 5, scope: !10)
!37 = !DILocation(line: 19, column: 1, scope: !10)
!38 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 21, type: !39, scopeLine: 21, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!39 = !DISubroutineType(types: !40)
!40 = !{!15}
!41 = !DILocalVariable(name: "v", scope: !38, file: !11, line: 22, type: !15)
!42 = !DILocation(line: 22, column: 9, scope: !38)
!43 = !DILocation(line: 23, column: 5, scope: !38)
!44 = !DILocation(line: 24, column: 5, scope: !38)
