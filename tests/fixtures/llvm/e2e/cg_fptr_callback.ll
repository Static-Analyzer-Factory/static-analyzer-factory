; ModuleID = '/workspace/tests/programs/c/cg_fptr_callback.c'
source_filename = "/workspace/tests/programs/c/cg_fptr_callback.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"USER_CMD\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @dangerous_sink(ptr noundef %0) #0 !dbg !18 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !24, metadata !DIExpression()), !dbg !25
  %3 = load ptr, ptr %2, align 8, !dbg !26
  %4 = call i32 @system(ptr noundef %3), !dbg !27
  ret void, !dbg !28
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @system(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @dispatch(ptr noundef %0, ptr noundef %1) #0 !dbg !29 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !33, metadata !DIExpression()), !dbg !34
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !35, metadata !DIExpression()), !dbg !36
  %5 = load ptr, ptr %3, align 8, !dbg !37
  %6 = load ptr, ptr %4, align 8, !dbg !38
  call void %5(ptr noundef %6), !dbg !37
  ret void, !dbg !39
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !40 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !44, metadata !DIExpression()), !dbg !45
  %3 = call ptr @getenv(ptr noundef @.str) #4, !dbg !46
  store ptr %3, ptr %2, align 8, !dbg !45
  %4 = load ptr, ptr %2, align 8, !dbg !47
  call void @dispatch(ptr noundef @dangerous_sink, ptr noundef %4), !dbg !48
  ret i32 0, !dbg !49
}

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!10, !11, !12, !13, !14, !15, !16}
!llvm.ident = !{!17}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 12, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/cg_fptr_callback.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "de89350cb317e70f927d127158d4d9c0")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 72, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 9)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !9, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/cg_fptr_callback.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "de89350cb317e70f927d127158d4d9c0")
!9 = !{!0}
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "dangerous_sink", scope: !2, file: !2, line: 5, type: !19, scopeLine: 5, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!19 = !DISubroutineType(types: !20)
!20 = !{null, !21}
!21 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !22, size: 64)
!22 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!23 = !{}
!24 = !DILocalVariable(name: "cmd", arg: 1, scope: !18, file: !2, line: 5, type: !21)
!25 = !DILocation(line: 5, column: 33, scope: !18)
!26 = !DILocation(line: 5, column: 47, scope: !18)
!27 = !DILocation(line: 5, column: 40, scope: !18)
!28 = !DILocation(line: 5, column: 53, scope: !18)
!29 = distinct !DISubprogram(name: "dispatch", scope: !2, file: !2, line: 7, type: !30, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!30 = !DISubroutineType(types: !31)
!31 = !{null, !32, !21}
!32 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !19, size: 64)
!33 = !DILocalVariable(name: "handler", arg: 1, scope: !29, file: !2, line: 7, type: !32)
!34 = !DILocation(line: 7, column: 22, scope: !29)
!35 = !DILocalVariable(name: "data", arg: 2, scope: !29, file: !2, line: 7, type: !21)
!36 = !DILocation(line: 7, column: 58, scope: !29)
!37 = !DILocation(line: 8, column: 5, scope: !29)
!38 = !DILocation(line: 8, column: 13, scope: !29)
!39 = !DILocation(line: 9, column: 1, scope: !29)
!40 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 11, type: !41, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!41 = !DISubroutineType(types: !42)
!42 = !{!43}
!43 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!44 = !DILocalVariable(name: "input", scope: !40, file: !2, line: 12, type: !21)
!45 = !DILocation(line: 12, column: 17, scope: !40)
!46 = !DILocation(line: 12, column: 25, scope: !40)
!47 = !DILocation(line: 13, column: 30, scope: !40)
!48 = !DILocation(line: 13, column: 5, scope: !40)
!49 = !DILocation(line: 14, column: 5, scope: !40)
