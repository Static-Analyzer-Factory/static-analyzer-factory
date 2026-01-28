; ModuleID = '/workspace/tests/programs/c/mssa_loop.c'
source_filename = "/workspace/tests/programs/c/mssa_loop.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test(i32 noundef %0) #0 !dbg !10 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !16, metadata !DIExpression()), !dbg !17
  call void @llvm.dbg.declare(metadata ptr %3, metadata !18, metadata !DIExpression()), !dbg !19
  call void @llvm.dbg.declare(metadata ptr %4, metadata !20, metadata !DIExpression()), !dbg !22
  store ptr %3, ptr %4, align 8, !dbg !22
  %8 = load ptr, ptr %4, align 8, !dbg !23
  store i32 0, ptr %8, align 4, !dbg !24
  call void @llvm.dbg.declare(metadata ptr %5, metadata !25, metadata !DIExpression()), !dbg !27
  store i32 0, ptr %5, align 4, !dbg !27
  br label %9, !dbg !28

9:                                                ; preds = %19, %1
  %10 = load i32, ptr %5, align 4, !dbg !29
  %11 = load i32, ptr %2, align 4, !dbg !31
  %12 = icmp slt i32 %10, %11, !dbg !32
  br i1 %12, label %13, label %22, !dbg !33

13:                                               ; preds = %9
  call void @llvm.dbg.declare(metadata ptr %6, metadata !34, metadata !DIExpression()), !dbg !36
  %14 = load ptr, ptr %4, align 8, !dbg !37
  %15 = load i32, ptr %14, align 4, !dbg !38
  store i32 %15, ptr %6, align 4, !dbg !36
  %16 = load i32, ptr %6, align 4, !dbg !39
  %17 = add nsw i32 %16, 1, !dbg !40
  %18 = load ptr, ptr %4, align 8, !dbg !41
  store i32 %17, ptr %18, align 4, !dbg !42
  br label %19, !dbg !43

19:                                               ; preds = %13
  %20 = load i32, ptr %5, align 4, !dbg !44
  %21 = add nsw i32 %20, 1, !dbg !44
  store i32 %21, ptr %5, align 4, !dbg !44
  br label %9, !dbg !45, !llvm.loop !46

22:                                               ; preds = %9
  call void @llvm.dbg.declare(metadata ptr %7, metadata !49, metadata !DIExpression()), !dbg !50
  %23 = load ptr, ptr %4, align 8, !dbg !51
  %24 = load i32, ptr %23, align 4, !dbg !52
  store i32 %24, ptr %7, align 4, !dbg !50
  %25 = load i32, ptr %7, align 4, !dbg !53
  call void @sink(i32 noundef %25), !dbg !54
  ret void, !dbg !55
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare void @sink(i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !56 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test(i32 noundef 10), !dbg !59
  ret i32 0, !dbg !60
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/mssa_loop.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "a763c04206fb444641fa76c74e92d940")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 11, type: !12, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DIFile(filename: "c/mssa_loop.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "a763c04206fb444641fa76c74e92d940")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "n", arg: 1, scope: !10, file: !11, line: 11, type: !14)
!17 = !DILocation(line: 11, column: 15, scope: !10)
!18 = !DILocalVariable(name: "acc", scope: !10, file: !11, line: 12, type: !14)
!19 = !DILocation(line: 12, column: 9, scope: !10)
!20 = !DILocalVariable(name: "p", scope: !10, file: !11, line: 13, type: !21)
!21 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!22 = !DILocation(line: 13, column: 10, scope: !10)
!23 = !DILocation(line: 14, column: 6, scope: !10)
!24 = !DILocation(line: 14, column: 8, scope: !10)
!25 = !DILocalVariable(name: "i", scope: !26, file: !11, line: 15, type: !14)
!26 = distinct !DILexicalBlock(scope: !10, file: !11, line: 15, column: 5)
!27 = !DILocation(line: 15, column: 14, scope: !26)
!28 = !DILocation(line: 15, column: 10, scope: !26)
!29 = !DILocation(line: 15, column: 21, scope: !30)
!30 = distinct !DILexicalBlock(scope: !26, file: !11, line: 15, column: 5)
!31 = !DILocation(line: 15, column: 25, scope: !30)
!32 = !DILocation(line: 15, column: 23, scope: !30)
!33 = !DILocation(line: 15, column: 5, scope: !26)
!34 = !DILocalVariable(name: "x", scope: !35, file: !11, line: 16, type: !14)
!35 = distinct !DILexicalBlock(scope: !30, file: !11, line: 15, column: 33)
!36 = !DILocation(line: 16, column: 13, scope: !35)
!37 = !DILocation(line: 16, column: 18, scope: !35)
!38 = !DILocation(line: 16, column: 17, scope: !35)
!39 = !DILocation(line: 17, column: 14, scope: !35)
!40 = !DILocation(line: 17, column: 16, scope: !35)
!41 = !DILocation(line: 17, column: 10, scope: !35)
!42 = !DILocation(line: 17, column: 12, scope: !35)
!43 = !DILocation(line: 18, column: 5, scope: !35)
!44 = !DILocation(line: 15, column: 29, scope: !30)
!45 = !DILocation(line: 15, column: 5, scope: !30)
!46 = distinct !{!46, !33, !47, !48}
!47 = !DILocation(line: 18, column: 5, scope: !26)
!48 = !{!"llvm.loop.mustprogress"}
!49 = !DILocalVariable(name: "result", scope: !10, file: !11, line: 19, type: !14)
!50 = !DILocation(line: 19, column: 9, scope: !10)
!51 = !DILocation(line: 19, column: 19, scope: !10)
!52 = !DILocation(line: 19, column: 18, scope: !10)
!53 = !DILocation(line: 20, column: 10, scope: !10)
!54 = !DILocation(line: 20, column: 5, scope: !10)
!55 = !DILocation(line: 21, column: 1, scope: !10)
!56 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 23, type: !57, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!57 = !DISubroutineType(types: !58)
!58 = !{!14}
!59 = !DILocation(line: 24, column: 5, scope: !56)
!60 = !DILocation(line: 25, column: 5, scope: !56)
