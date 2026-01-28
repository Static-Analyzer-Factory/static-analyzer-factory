; ModuleID = '/workspace/tests/programs/c/fspta_interproc.c'
source_filename = "/workspace/tests/programs/c/fspta_interproc.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@target_val = dso_local global i32 0, align 4, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @set_ptr(ptr noundef %0) #0 !dbg !15 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !22
  %3 = load ptr, ptr %2, align 8, !dbg !23
  store ptr @target_val, ptr %3, align 8, !dbg !24
  ret void, !dbg !25
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @test_interproc() #0 !dbg !26 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !29, metadata !DIExpression()), !dbg !30
  store ptr null, ptr %1, align 8, !dbg !30
  call void @set_ptr(ptr noundef %1), !dbg !31
  %2 = load ptr, ptr %1, align 8, !dbg !32
  %3 = load i32, ptr %2, align 4, !dbg !33
  ret i32 %3, !dbg !34
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !35 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %2 = call i32 @test_interproc(), !dbg !36
  ret i32 %2, !dbg !37
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!2}
!llvm.module.flags = !{!7, !8, !9, !10, !11, !12, !13}
!llvm.ident = !{!14}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "target_val", scope: !2, file: !5, line: 9, type: !6, isLocal: false, isDefinition: true)
!2 = distinct !DICompileUnit(language: DW_LANG_C11, file: !3, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !4, splitDebugInlining: false, nameTableKind: None)
!3 = !DIFile(filename: "/workspace/tests/programs/c/fspta_interproc.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4fc69a0f179270ffc69fc93a852439df")
!4 = !{!0}
!5 = !DIFile(filename: "c/fspta_interproc.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "4fc69a0f179270ffc69fc93a852439df")
!6 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!7 = !{i32 7, !"Dwarf Version", i32 5}
!8 = !{i32 2, !"Debug Info Version", i32 3}
!9 = !{i32 1, !"wchar_size", i32 4}
!10 = !{i32 8, !"PIC Level", i32 2}
!11 = !{i32 7, !"PIE Level", i32 2}
!12 = !{i32 7, !"uwtable", i32 2}
!13 = !{i32 7, !"frame-pointer", i32 1}
!14 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!15 = distinct !DISubprogram(name: "set_ptr", scope: !5, file: !5, line: 11, type: !16, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !20)
!16 = !DISubroutineType(types: !17)
!17 = !{null, !18}
!18 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !19, size: 64)
!19 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !6, size: 64)
!20 = !{}
!21 = !DILocalVariable(name: "pp", arg: 1, scope: !15, file: !5, line: 11, type: !18)
!22 = !DILocation(line: 11, column: 20, scope: !15)
!23 = !DILocation(line: 12, column: 6, scope: !15)
!24 = !DILocation(line: 12, column: 9, scope: !15)
!25 = !DILocation(line: 13, column: 1, scope: !15)
!26 = distinct !DISubprogram(name: "test_interproc", scope: !5, file: !5, line: 15, type: !27, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2, retainedNodes: !20)
!27 = !DISubroutineType(types: !28)
!28 = !{!6}
!29 = !DILocalVariable(name: "p", scope: !26, file: !5, line: 16, type: !19)
!30 = !DILocation(line: 16, column: 10, scope: !26)
!31 = !DILocation(line: 17, column: 5, scope: !26)
!32 = !DILocation(line: 19, column: 13, scope: !26)
!33 = !DILocation(line: 19, column: 12, scope: !26)
!34 = !DILocation(line: 19, column: 5, scope: !26)
!35 = distinct !DISubprogram(name: "main", scope: !5, file: !5, line: 22, type: !27, scopeLine: 22, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !2)
!36 = !DILocation(line: 23, column: 12, scope: !35)
!37 = !DILocation(line: 23, column: 5, scope: !35)
