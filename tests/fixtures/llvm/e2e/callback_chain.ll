; ModuleID = '/workspace/tests/programs/c/callback_chain.c'
source_filename = "/workspace/tests/programs/c/callback_chain.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @step_c(ptr noundef %0) #0 !dbg !10 {
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
define dso_local void @step_b(ptr noundef %0) #0 !dbg !23 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !24, metadata !DIExpression()), !dbg !25
  call void @llvm.dbg.declare(metadata ptr %3, metadata !26, metadata !DIExpression()), !dbg !29
  store ptr @step_c, ptr %3, align 8, !dbg !29
  %4 = load ptr, ptr %3, align 8, !dbg !30
  %5 = load ptr, ptr %2, align 8, !dbg !31
  call void %4(ptr noundef %5), !dbg !30
  ret void, !dbg !32
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @step_a(ptr noundef %0) #0 !dbg !33 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !34, metadata !DIExpression()), !dbg !35
  call void @llvm.dbg.declare(metadata ptr %3, metadata !36, metadata !DIExpression()), !dbg !37
  store ptr @step_b, ptr %3, align 8, !dbg !37
  %4 = load ptr, ptr %3, align 8, !dbg !38
  %5 = load ptr, ptr %2, align 8, !dbg !39
  call void %4(ptr noundef %5), !dbg !38
  ret void, !dbg !40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !41 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !47, metadata !DIExpression()), !dbg !48
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !49, metadata !DIExpression()), !dbg !50
  %7 = load i32, ptr %4, align 4, !dbg !51
  %8 = icmp slt i32 %7, 2, !dbg !53
  br i1 %8, label %9, label %10, !dbg !54

9:                                                ; preds = %2
  store i32 1, ptr %3, align 4, !dbg !55
  br label %15, !dbg !55

10:                                               ; preds = %2
  call void @llvm.dbg.declare(metadata ptr %6, metadata !56, metadata !DIExpression()), !dbg !57
  %11 = load ptr, ptr %5, align 8, !dbg !58
  %12 = getelementptr inbounds ptr, ptr %11, i64 1, !dbg !58
  %13 = load ptr, ptr %12, align 8, !dbg !58
  store ptr %13, ptr %6, align 8, !dbg !57
  %14 = load ptr, ptr %6, align 8, !dbg !59
  call void @step_a(ptr noundef %14), !dbg !60
  store i32 0, ptr %3, align 4, !dbg !61
  br label %15, !dbg !61

15:                                               ; preds = %10, %9
  %16 = load i32, ptr %3, align 4, !dbg !62
  ret i32 %16, !dbg !62
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/callback_chain.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "487765235d1dacd02834a5031203725a")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "step_c", scope: !11, file: !11, line: 11, type: !12, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!11 = !DIFile(filename: "c/callback_chain.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "487765235d1dacd02834a5031203725a")
!12 = !DISubroutineType(types: !13)
!13 = !{null, !14}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !15, size: 64)
!15 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !16)
!16 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!17 = !{}
!18 = !DILocalVariable(name: "data", arg: 1, scope: !10, file: !11, line: 11, type: !14)
!19 = !DILocation(line: 11, column: 25, scope: !10)
!20 = !DILocation(line: 12, column: 12, scope: !10)
!21 = !DILocation(line: 12, column: 5, scope: !10)
!22 = !DILocation(line: 13, column: 1, scope: !10)
!23 = distinct !DISubprogram(name: "step_b", scope: !11, file: !11, line: 15, type: !12, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!24 = !DILocalVariable(name: "data", arg: 1, scope: !23, file: !11, line: 15, type: !14)
!25 = !DILocation(line: 15, column: 25, scope: !23)
!26 = !DILocalVariable(name: "next", scope: !23, file: !11, line: 16, type: !27)
!27 = !DIDerivedType(tag: DW_TAG_typedef, name: "handler_t", file: !11, line: 9, baseType: !28)
!28 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !12, size: 64)
!29 = !DILocation(line: 16, column: 15, scope: !23)
!30 = !DILocation(line: 17, column: 5, scope: !23)
!31 = !DILocation(line: 17, column: 10, scope: !23)
!32 = !DILocation(line: 18, column: 1, scope: !23)
!33 = distinct !DISubprogram(name: "step_a", scope: !11, file: !11, line: 20, type: !12, scopeLine: 20, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!34 = !DILocalVariable(name: "input", arg: 1, scope: !33, file: !11, line: 20, type: !14)
!35 = !DILocation(line: 20, column: 25, scope: !33)
!36 = !DILocalVariable(name: "handler", scope: !33, file: !11, line: 21, type: !27)
!37 = !DILocation(line: 21, column: 15, scope: !33)
!38 = !DILocation(line: 22, column: 5, scope: !33)
!39 = !DILocation(line: 22, column: 13, scope: !33)
!40 = !DILocation(line: 23, column: 1, scope: !33)
!41 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 25, type: !42, scopeLine: 25, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !17)
!42 = !DISubroutineType(types: !43)
!43 = !{!44, !44, !45}
!44 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!45 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !46, size: 64)
!46 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !16, size: 64)
!47 = !DILocalVariable(name: "argc", arg: 1, scope: !41, file: !11, line: 25, type: !44)
!48 = !DILocation(line: 25, column: 14, scope: !41)
!49 = !DILocalVariable(name: "argv", arg: 2, scope: !41, file: !11, line: 25, type: !45)
!50 = !DILocation(line: 25, column: 26, scope: !41)
!51 = !DILocation(line: 26, column: 9, scope: !52)
!52 = distinct !DILexicalBlock(scope: !41, file: !11, line: 26, column: 9)
!53 = !DILocation(line: 26, column: 14, scope: !52)
!54 = !DILocation(line: 26, column: 9, scope: !41)
!55 = !DILocation(line: 26, column: 19, scope: !52)
!56 = !DILocalVariable(name: "user_data", scope: !41, file: !11, line: 27, type: !46)
!57 = !DILocation(line: 27, column: 11, scope: !41)
!58 = !DILocation(line: 27, column: 23, scope: !41)
!59 = !DILocation(line: 28, column: 12, scope: !41)
!60 = !DILocation(line: 28, column: 5, scope: !41)
!61 = !DILocation(line: 29, column: 5, scope: !41)
!62 = !DILocation(line: 30, column: 1, scope: !41)
