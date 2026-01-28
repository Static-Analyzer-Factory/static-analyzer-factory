; ModuleID = '/workspace/tests/programs/c/format_string.c'
source_filename = "/workspace/tests/programs/c/format_string.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !10 {
  %1 = alloca i32, align 4
  %2 = alloca [256 x i8], align 1
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !16, metadata !DIExpression()), !dbg !21
  %3 = getelementptr inbounds [256 x i8], ptr %2, i64 0, i64 0, !dbg !22
  %4 = call ptr @gets(ptr noundef %3), !dbg !23
  %5 = getelementptr inbounds [256 x i8], ptr %2, i64 0, i64 0, !dbg !24
  %6 = call i32 (ptr, ...) @printf(ptr noundef %5), !dbg !25
  ret i32 0, !dbg !26
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare ptr @gets(ptr noundef) #2

declare i32 @printf(ptr noundef, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/format_string.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "20cf570ce2e065318f7ebb40aaaecb43")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 10, type: !12, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DIFile(filename: "c/format_string.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "20cf570ce2e065318f7ebb40aaaecb43")
!12 = !DISubroutineType(types: !13)
!13 = !{!14}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "buf", scope: !10, file: !11, line: 11, type: !17)
!17 = !DICompositeType(tag: DW_TAG_array_type, baseType: !18, size: 2048, elements: !19)
!18 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!19 = !{!20}
!20 = !DISubrange(count: 256)
!21 = !DILocation(line: 11, column: 10, scope: !10)
!22 = !DILocation(line: 12, column: 10, scope: !10)
!23 = !DILocation(line: 12, column: 5, scope: !10)
!24 = !DILocation(line: 13, column: 12, scope: !10)
!25 = !DILocation(line: 13, column: 5, scope: !10)
!26 = !DILocation(line: 14, column: 5, scope: !10)
