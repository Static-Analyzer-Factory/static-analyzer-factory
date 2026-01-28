; ModuleID = '/workspace/tests/programs/c/callback_fn_ptr.c'
source_filename = "/workspace/tests/programs/c/callback_fn_ptr.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [16 x i8] c"processing: %d\0A\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @process(i32 noundef %0) #0 !dbg !18 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !23, metadata !DIExpression()), !dbg !24
  %3 = load i32, ptr %2, align 4, !dbg !25
  %4 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %3), !dbg !26
  ret void, !dbg !27
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @printf(ptr noundef, ...) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @invoke(ptr noundef %0, i32 noundef %1) #0 !dbg !28 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !33, metadata !DIExpression()), !dbg !34
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !35, metadata !DIExpression()), !dbg !36
  %5 = load ptr, ptr %3, align 8, !dbg !37
  %6 = load i32, ptr %4, align 4, !dbg !38
  call void %5(i32 noundef %6), !dbg !37
  ret void, !dbg !39
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !40 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !43, metadata !DIExpression()), !dbg !44
  store ptr @process, ptr %2, align 8, !dbg !44
  %3 = load ptr, ptr %2, align 8, !dbg !45
  call void @invoke(ptr noundef %3, i32 noundef 42), !dbg !46
  ret i32 0, !dbg !47
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!10, !11, !12, !13, !14, !15, !16}
!llvm.ident = !{!17}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 11, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/callback_fn_ptr.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "f375d6daca452135237d8a19144b463f")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 128, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 16)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !9, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/callback_fn_ptr.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "f375d6daca452135237d8a19144b463f")
!9 = !{!0}
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "process", scope: !2, file: !2, line: 10, type: !19, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !22)
!19 = !DISubroutineType(types: !20)
!20 = !{null, !21}
!21 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!22 = !{}
!23 = !DILocalVariable(name: "x", arg: 1, scope: !18, file: !2, line: 10, type: !21)
!24 = !DILocation(line: 10, column: 18, scope: !18)
!25 = !DILocation(line: 11, column: 32, scope: !18)
!26 = !DILocation(line: 11, column: 5, scope: !18)
!27 = !DILocation(line: 12, column: 1, scope: !18)
!28 = distinct !DISubprogram(name: "invoke", scope: !2, file: !2, line: 14, type: !29, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !22)
!29 = !DISubroutineType(types: !30)
!30 = !{null, !31, !21}
!31 = !DIDerivedType(tag: DW_TAG_typedef, name: "callback_t", file: !2, line: 8, baseType: !32)
!32 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !19, size: 64)
!33 = !DILocalVariable(name: "fp", arg: 1, scope: !28, file: !2, line: 14, type: !31)
!34 = !DILocation(line: 14, column: 24, scope: !28)
!35 = !DILocalVariable(name: "val", arg: 2, scope: !28, file: !2, line: 14, type: !21)
!36 = !DILocation(line: 14, column: 32, scope: !28)
!37 = !DILocation(line: 15, column: 5, scope: !28)
!38 = !DILocation(line: 15, column: 8, scope: !28)
!39 = !DILocation(line: 16, column: 1, scope: !28)
!40 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 18, type: !41, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !22)
!41 = !DISubroutineType(types: !42)
!42 = !{!21}
!43 = !DILocalVariable(name: "cb", scope: !40, file: !2, line: 19, type: !31)
!44 = !DILocation(line: 19, column: 16, scope: !40)
!45 = !DILocation(line: 20, column: 12, scope: !40)
!46 = !DILocation(line: 20, column: 5, scope: !40)
!47 = !DILocation(line: 21, column: 5, scope: !40)
