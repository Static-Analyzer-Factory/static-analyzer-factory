; ModuleID = '/workspace/tests/programs/c/mssa_store_load_simple.c'
source_filename = "/workspace/tests/programs/c/mssa_store_load_simple.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test() #0 !dbg !10 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !15, metadata !DIExpression()), !dbg !17
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  call void @llvm.dbg.declare(metadata ptr %3, metadata !20, metadata !DIExpression()), !dbg !22
  store ptr %1, ptr %3, align 8, !dbg !22
  call void @llvm.dbg.declare(metadata ptr %4, metadata !23, metadata !DIExpression()), !dbg !24
  store ptr %2, ptr %4, align 8, !dbg !24
  %6 = call i32 @source(), !dbg !25
  %7 = load ptr, ptr %3, align 8, !dbg !26
  store i32 %6, ptr %7, align 4, !dbg !27
  %8 = load ptr, ptr %4, align 8, !dbg !28
  store i32 99, ptr %8, align 4, !dbg !29
  call void @llvm.dbg.declare(metadata ptr %5, metadata !30, metadata !DIExpression()), !dbg !31
  %9 = load ptr, ptr %3, align 8, !dbg !32
  %10 = load i32, ptr %9, align 4, !dbg !33
  store i32 %10, ptr %5, align 4, !dbg !31
  %11 = load i32, ptr %5, align 4, !dbg !34
  call void @sink(i32 noundef %11), !dbg !35
  ret void, !dbg !36
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @source() #2

declare void @sink(i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !37 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test(), !dbg !40
  ret i32 0, !dbg !41
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/mssa_store_load_simple.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "0e78ea3ae9d1ccc822c8ab223c6b3d0a")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 12, type: !12, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !14)
!11 = !DIFile(filename: "c/mssa_store_load_simple.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "0e78ea3ae9d1ccc822c8ab223c6b3d0a")
!12 = !DISubroutineType(types: !13)
!13 = !{null}
!14 = !{}
!15 = !DILocalVariable(name: "a", scope: !10, file: !11, line: 13, type: !16)
!16 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!17 = !DILocation(line: 13, column: 9, scope: !10)
!18 = !DILocalVariable(name: "b", scope: !10, file: !11, line: 13, type: !16)
!19 = !DILocation(line: 13, column: 12, scope: !10)
!20 = !DILocalVariable(name: "p", scope: !10, file: !11, line: 14, type: !21)
!21 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!22 = !DILocation(line: 14, column: 10, scope: !10)
!23 = !DILocalVariable(name: "q", scope: !10, file: !11, line: 15, type: !21)
!24 = !DILocation(line: 15, column: 10, scope: !10)
!25 = !DILocation(line: 16, column: 10, scope: !10)
!26 = !DILocation(line: 16, column: 6, scope: !10)
!27 = !DILocation(line: 16, column: 8, scope: !10)
!28 = !DILocation(line: 17, column: 6, scope: !10)
!29 = !DILocation(line: 17, column: 8, scope: !10)
!30 = !DILocalVariable(name: "x", scope: !10, file: !11, line: 18, type: !16)
!31 = !DILocation(line: 18, column: 9, scope: !10)
!32 = !DILocation(line: 18, column: 14, scope: !10)
!33 = !DILocation(line: 18, column: 13, scope: !10)
!34 = !DILocation(line: 19, column: 10, scope: !10)
!35 = !DILocation(line: 19, column: 5, scope: !10)
!36 = !DILocation(line: 20, column: 1, scope: !10)
!37 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 22, type: !38, scopeLine: 22, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!38 = !DISubroutineType(types: !39)
!39 = !{!16}
!40 = !DILocation(line: 23, column: 5, scope: !37)
!41 = !DILocation(line: 24, column: 5, scope: !37)
