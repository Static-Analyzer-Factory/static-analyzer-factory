; ModuleID = '/workspace/tests/programs/c/library_wrapper.c'
source_filename = "/workspace/tests/programs/c/library_wrapper.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @safe_exec(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  %3 = load ptr, ptr %2, align 8, !dbg !20
  %4 = call i32 @system(ptr noundef %3), !dbg !21
  ret void, !dbg !22
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @system(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !23 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !29, metadata !DIExpression()), !dbg !30
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !31, metadata !DIExpression()), !dbg !32
  %7 = load i32, ptr %4, align 4, !dbg !33
  %8 = icmp slt i32 %7, 2, !dbg !35
  br i1 %8, label %9, label %10, !dbg !36

9:                                                ; preds = %2
  store i32 1, ptr %3, align 4, !dbg !37
  br label %15, !dbg !37

10:                                               ; preds = %2
  call void @llvm.dbg.declare(metadata ptr %6, metadata !38, metadata !DIExpression()), !dbg !39
  %11 = load ptr, ptr %5, align 8, !dbg !40
  %12 = getelementptr inbounds ptr, ptr %11, i64 1, !dbg !40
  %13 = load ptr, ptr %12, align 8, !dbg !40
  store ptr %13, ptr %6, align 8, !dbg !39
  %14 = load ptr, ptr %6, align 8, !dbg !41
  call void @safe_exec(ptr noundef %14), !dbg !42
  store i32 0, ptr %3, align 4, !dbg !43
  br label %15, !dbg !43

15:                                               ; preds = %10, %9
  %16 = load i32, ptr %3, align 4, !dbg !44
  ret i32 %16, !dbg !44
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/library_wrapper.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "6c1c9772c92ba9be94d9d85f0fed080a")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "safe_exec", scope: !11, file: !11, line: 9, type: !12, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!11 = !DIFile(filename: "c/library_wrapper.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "6c1c9772c92ba9be94d9d85f0fed080a")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !16)
!16 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!17 = !{}
!18 = !DILocalVariable(name: "cmd", arg: 1, scope: !10, file: !11, line: 9, type: !14)
!19 = !DILocation(line: 9, column: 28, scope: !10)
!20 = !DILocation(line: 11, column: 12, scope: !10)
!21 = !DILocation(line: 11, column: 5, scope: !10)
!22 = !DILocation(line: 12, column: 1, scope: !10)
!23 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 14, type: !24, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!24 = !DISubroutineType(types: !25)
!25 = !{!26, !26, !27}
!26 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!27 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !28, size: 64)
!28 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!29 = !DILocalVariable(name: "argc", arg: 1, scope: !23, file: !11, line: 14, type: !26)
!30 = !DILocation(line: 14, column: 14, scope: !23)
!31 = !DILocalVariable(name: "argv", arg: 2, scope: !23, file: !11, line: 14, type: !27)
!32 = !DILocation(line: 14, column: 26, scope: !23)
!33 = !DILocation(line: 15, column: 9, scope: !34)
!34 = distinct !DILexicalBlock(scope: !23, file: !11, line: 15, column: 9)
!35 = !DILocation(line: 15, column: 14, scope: !34)
!36 = !DILocation(line: 15, column: 9, scope: !23)
!37 = !DILocation(line: 15, column: 19, scope: !34)
!38 = !DILocalVariable(name: "user_input", scope: !23, file: !11, line: 16, type: !28)
!39 = !DILocation(line: 16, column: 11, scope: !23)
!40 = !DILocation(line: 16, column: 24, scope: !23)
!41 = !DILocation(line: 17, column: 15, scope: !23)
!42 = !DILocation(line: 17, column: 5, scope: !23)
!43 = !DILocation(line: 18, column: 5, scope: !23)
!44 = !DILocation(line: 19, column: 1, scope: !23)
