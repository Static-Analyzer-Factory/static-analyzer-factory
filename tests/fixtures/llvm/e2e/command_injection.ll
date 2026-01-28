; ModuleID = '/workspace/tests/programs/c/command_injection.c'
source_filename = "/workspace/tests/programs/c/command_injection.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !10 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !21, metadata !DIExpression()), !dbg !22
  %7 = load i32, ptr %4, align 4, !dbg !23
  %8 = icmp slt i32 %7, 2, !dbg !25
  br i1 %8, label %9, label %10, !dbg !26

9:                                                ; preds = %2
  store i32 1, ptr %3, align 4, !dbg !27
  br label %16, !dbg !27

10:                                               ; preds = %2
  call void @llvm.dbg.declare(metadata ptr %6, metadata !28, metadata !DIExpression()), !dbg !29
  %11 = load ptr, ptr %5, align 8, !dbg !30
  %12 = getelementptr inbounds ptr, ptr %11, i64 1, !dbg !30
  %13 = load ptr, ptr %12, align 8, !dbg !30
  store ptr %13, ptr %6, align 8, !dbg !29
  %14 = load ptr, ptr %6, align 8, !dbg !31
  %15 = call i32 @system(ptr noundef %14), !dbg !32
  store i32 %15, ptr %3, align 4, !dbg !33
  br label %16, !dbg !33

16:                                               ; preds = %10, %9
  %17 = load i32, ptr %3, align 4, !dbg !34
  ret i32 %17, !dbg !34
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @system(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/command_injection.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "e9639a7830bf2ba6c2594b918d7fc83c")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 7, type: !12, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!11 = !DIFile(filename: "c/command_injection.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "e9639a7830bf2ba6c2594b918d7fc83c")
!12 = !DISubroutineType(types: !13)
!13 = !{!14, !14, !15}
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!16 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !17, size: 64)
!17 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!18 = !{}
!19 = !DILocalVariable(name: "argc", arg: 1, scope: !10, file: !11, line: 7, type: !14)
!20 = !DILocation(line: 7, column: 14, scope: !10)
!21 = !DILocalVariable(name: "argv", arg: 2, scope: !10, file: !11, line: 7, type: !15)
!22 = !DILocation(line: 7, column: 26, scope: !10)
!23 = !DILocation(line: 8, column: 9, scope: !24)
!24 = distinct !DILexicalBlock(scope: !10, file: !11, line: 8, column: 9)
!25 = !DILocation(line: 8, column: 14, scope: !24)
!26 = !DILocation(line: 8, column: 9, scope: !10)
!27 = !DILocation(line: 8, column: 19, scope: !24)
!28 = !DILocalVariable(name: "user_cmd", scope: !10, file: !11, line: 9, type: !16)
!29 = !DILocation(line: 9, column: 11, scope: !10)
!30 = !DILocation(line: 9, column: 22, scope: !10)
!31 = !DILocation(line: 10, column: 19, scope: !10)
!32 = !DILocation(line: 10, column: 12, scope: !10)
!33 = !DILocation(line: 10, column: 5, scope: !10)
!34 = !DILocation(line: 11, column: 1, scope: !10)
