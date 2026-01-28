; ModuleID = '/workspace/tests/programs/c/cross_module_taint.c'
source_filename = "/workspace/tests/programs/c/cross_module_taint.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @module_a_get_input(i32 noundef %0, ptr noundef %1) #0 !dbg !10 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !21, metadata !DIExpression()), !dbg !22
  %6 = load i32, ptr %4, align 4, !dbg !23
  %7 = icmp slt i32 %6, 2, !dbg !25
  br i1 %7, label %8, label %9, !dbg !26

8:                                                ; preds = %2
  store ptr null, ptr %3, align 8, !dbg !27
  br label %13, !dbg !27

9:                                                ; preds = %2
  %10 = load ptr, ptr %5, align 8, !dbg !28
  %11 = getelementptr inbounds ptr, ptr %10, i64 1, !dbg !28
  %12 = load ptr, ptr %11, align 8, !dbg !28
  store ptr %12, ptr %3, align 8, !dbg !29
  br label %13, !dbg !29

13:                                               ; preds = %9, %8
  %14 = load ptr, ptr %3, align 8, !dbg !30
  ret ptr %14, !dbg !30
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @module_b_execute(ptr noundef %0) #0 !dbg !31 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !36, metadata !DIExpression()), !dbg !37
  %3 = load ptr, ptr %2, align 8, !dbg !38
  %4 = call i32 @system(ptr noundef %3), !dbg !39
  ret void, !dbg !40
}

declare i32 @system(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !41 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !44, metadata !DIExpression()), !dbg !45
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !46, metadata !DIExpression()), !dbg !47
  call void @llvm.dbg.declare(metadata ptr %6, metadata !48, metadata !DIExpression()), !dbg !49
  %7 = load i32, ptr %4, align 4, !dbg !50
  %8 = load ptr, ptr %5, align 8, !dbg !51
  %9 = call ptr @module_a_get_input(i32 noundef %7, ptr noundef %8), !dbg !52
  store ptr %9, ptr %6, align 8, !dbg !49
  %10 = load ptr, ptr %6, align 8, !dbg !53
  %11 = icmp ne ptr %10, null, !dbg !53
  br i1 %11, label %12, label %14, !dbg !55

12:                                               ; preds = %2
  %13 = load ptr, ptr %6, align 8, !dbg !56
  call void @module_b_execute(ptr noundef %13), !dbg !58
  br label %14, !dbg !59

14:                                               ; preds = %12, %2
  ret i32 0, !dbg !60
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/cross_module_taint.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bef5f44e092132b38a349f82d6835053")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "module_a_get_input", scope: !11, file: !11, line: 11, type: !12, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!11 = !DIFile(filename: "c/cross_module_taint.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "bef5f44e092132b38a349f82d6835053")
!12 = !DISubroutineType(types: !13)
!13 = !{!14, !16, !17}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!16 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!17 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!18 = !{}
!19 = !DILocalVariable(name: "argc", arg: 1, scope: !10, file: !11, line: 11, type: !16)
!20 = !DILocation(line: 11, column: 30, scope: !10)
!21 = !DILocalVariable(name: "argv", arg: 2, scope: !10, file: !11, line: 11, type: !17)
!22 = !DILocation(line: 11, column: 42, scope: !10)
!23 = !DILocation(line: 12, column: 9, scope: !24)
!24 = distinct !DILexicalBlock(scope: !10, file: !11, line: 12, column: 9)
!25 = !DILocation(line: 12, column: 14, scope: !24)
!26 = !DILocation(line: 12, column: 9, scope: !10)
!27 = !DILocation(line: 12, column: 19, scope: !24)
!28 = !DILocation(line: 13, column: 12, scope: !10)
!29 = !DILocation(line: 13, column: 5, scope: !10)
!30 = !DILocation(line: 14, column: 1, scope: !10)
!31 = distinct !DISubprogram(name: "module_b_execute", scope: !11, file: !11, line: 17, type: !32, scopeLine: 17, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!32 = !DISubroutineType(types: !33)
!33 = !{null, !34}
!34 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !35, size: 64)
!35 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !15)
!36 = !DILocalVariable(name: "cmd", arg: 1, scope: !31, file: !11, line: 17, type: !34)
!37 = !DILocation(line: 17, column: 35, scope: !31)
!38 = !DILocation(line: 18, column: 12, scope: !31)
!39 = !DILocation(line: 18, column: 5, scope: !31)
!40 = !DILocation(line: 19, column: 1, scope: !31)
!41 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 21, type: !42, scopeLine: 21, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!42 = !DISubroutineType(types: !43)
!43 = !{!16, !16, !17}
!44 = !DILocalVariable(name: "argc", arg: 1, scope: !41, file: !11, line: 21, type: !16)
!45 = !DILocation(line: 21, column: 14, scope: !41)
!46 = !DILocalVariable(name: "argv", arg: 2, scope: !41, file: !11, line: 21, type: !17)
!47 = !DILocation(line: 21, column: 26, scope: !41)
!48 = !DILocalVariable(name: "input", scope: !41, file: !11, line: 22, type: !14)
!49 = !DILocation(line: 22, column: 11, scope: !41)
!50 = !DILocation(line: 22, column: 38, scope: !41)
!51 = !DILocation(line: 22, column: 44, scope: !41)
!52 = !DILocation(line: 22, column: 19, scope: !41)
!53 = !DILocation(line: 23, column: 9, scope: !54)
!54 = distinct !DILexicalBlock(scope: !41, file: !11, line: 23, column: 9)
!55 = !DILocation(line: 23, column: 9, scope: !41)
!56 = !DILocation(line: 24, column: 26, scope: !57)
!57 = distinct !DILexicalBlock(scope: !54, file: !11, line: 23, column: 16)
!58 = !DILocation(line: 24, column: 9, scope: !57)
!59 = !DILocation(line: 25, column: 5, scope: !57)
!60 = !DILocation(line: 26, column: 5, scope: !41)
