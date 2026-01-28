; ModuleID = '/workspace/tests/programs/c/mssa_interproc.c'
source_filename = "/workspace/tests/programs/c/mssa_interproc.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @modify(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !17, metadata !DIExpression()), !dbg !18
  %3 = load ptr, ptr %2, align 8, !dbg !19
  store i32 100, ptr %3, align 4, !dbg !20
  ret void, !dbg !21
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test() #0 !dbg !22 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !25, metadata !DIExpression()), !dbg !26
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  store ptr %1, ptr %2, align 8, !dbg !28
  %4 = call i32 @source(), !dbg !29
  %5 = load ptr, ptr %2, align 8, !dbg !30
  store i32 %4, ptr %5, align 4, !dbg !31
  %6 = load ptr, ptr %2, align 8, !dbg !32
  call void @modify(ptr noundef %6), !dbg !33
  call void @llvm.dbg.declare(metadata ptr %3, metadata !34, metadata !DIExpression()), !dbg !35
  %7 = load ptr, ptr %2, align 8, !dbg !36
  %8 = load i32, ptr %7, align 4, !dbg !37
  store i32 %8, ptr %3, align 4, !dbg !35
  %9 = load i32, ptr %3, align 4, !dbg !38
  call void @sink(i32 noundef %9), !dbg !39
  ret void, !dbg !40
}

declare i32 @source() #2

declare void @sink(i32 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !41 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test(), !dbg !44
  ret i32 0, !dbg !45
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/mssa_interproc.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "7efe7dcfc505c4d40a060b9cda5512e9")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "modify", scope: !11, file: !11, line: 12, type: !12, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!11 = !DIFile(filename: "c/mssa_interproc.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "7efe7dcfc505c4d40a060b9cda5512e9")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!16 = !{}
!17 = !DILocalVariable(name: "p", arg: 1, scope: !10, file: !11, line: 12, type: !14)
!18 = !DILocation(line: 12, column: 18, scope: !10)
!19 = !DILocation(line: 13, column: 6, scope: !10)
!20 = !DILocation(line: 13, column: 8, scope: !10)
!21 = !DILocation(line: 14, column: 1, scope: !10)
!22 = distinct !DISubprogram(name: "test", scope: !11, file: !11, line: 16, type: !23, scopeLine: 16, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!23 = !DISubroutineType(types: !24)
!24 = !{null}
!25 = !DILocalVariable(name: "a", scope: !22, file: !11, line: 17, type: !15)
!26 = !DILocation(line: 17, column: 9, scope: !22)
!27 = !DILocalVariable(name: "p", scope: !22, file: !11, line: 18, type: !14)
!28 = !DILocation(line: 18, column: 10, scope: !22)
!29 = !DILocation(line: 19, column: 10, scope: !22)
!30 = !DILocation(line: 19, column: 6, scope: !22)
!31 = !DILocation(line: 19, column: 8, scope: !22)
!32 = !DILocation(line: 20, column: 12, scope: !22)
!33 = !DILocation(line: 20, column: 5, scope: !22)
!34 = !DILocalVariable(name: "x", scope: !22, file: !11, line: 21, type: !15)
!35 = !DILocation(line: 21, column: 9, scope: !22)
!36 = !DILocation(line: 21, column: 14, scope: !22)
!37 = !DILocation(line: 21, column: 13, scope: !22)
!38 = !DILocation(line: 22, column: 10, scope: !22)
!39 = !DILocation(line: 22, column: 5, scope: !22)
!40 = !DILocation(line: 23, column: 1, scope: !22)
!41 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 25, type: !42, scopeLine: 25, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!42 = !DISubroutineType(types: !43)
!43 = !{!15}
!44 = !DILocation(line: 26, column: 5, scope: !41)
!45 = !DILocation(line: 27, column: 5, scope: !41)
