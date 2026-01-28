; ModuleID = 'tests/programs/c/z3_ifds_genuine_taint.c'
source_filename = "tests/programs/c/z3_ifds_genuine_taint.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"CMD\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @run_command() #0 !dbg !17 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !21, metadata !DIExpression()), !dbg !23
  %2 = call ptr @getenv(ptr noundef @.str) #4, !dbg !24
  store ptr %2, ptr %1, align 8, !dbg !23
  %3 = load ptr, ptr %1, align 8, !dbg !25
  %4 = call i32 @system(ptr noundef %3), !dbg !26
  ret void, !dbg !27
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

declare i32 @system(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !28 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @run_command(), !dbg !32
  ret i32 0, !dbg !33
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 9, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_ifds_genuine_taint.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "c1b1815ababb53d2ae68d93206ca13ff")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 4)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !8, splitDebugInlining: false, nameTableKind: None)
!8 = !{!0}
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 1}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "run_command", scope: !2, file: !2, line: 8, type: !18, scopeLine: 8, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{null}
!20 = !{}
!21 = !DILocalVariable(name: "cmd", scope: !17, file: !2, line: 9, type: !22)
!22 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!23 = !DILocation(line: 9, column: 11, scope: !17)
!24 = !DILocation(line: 9, column: 17, scope: !17)
!25 = !DILocation(line: 11, column: 12, scope: !17)
!26 = !DILocation(line: 11, column: 5, scope: !17)
!27 = !DILocation(line: 12, column: 1, scope: !17)
!28 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 14, type: !29, scopeLine: 14, spFlags: DISPFlagDefinition, unit: !7)
!29 = !DISubroutineType(types: !30)
!30 = !{!31}
!31 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!32 = !DILocation(line: 15, column: 5, scope: !28)
!33 = !DILocation(line: 16, column: 5, scope: !28)
