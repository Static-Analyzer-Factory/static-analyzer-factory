; ModuleID = '/workspace/tests/programs/c/cg_iterative_resolve.c'
source_filename = "/workspace/tests/programs/c/cg_iterative_resolve.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"CMD\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @final_sink(ptr noundef %0) #0 !dbg !18 {
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
define dso_local void @trampoline(ptr noundef %0, ptr noundef %1) #0 !dbg !29 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !34, metadata !DIExpression()), !dbg !35
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !36, metadata !DIExpression()), !dbg !37
  %5 = load ptr, ptr %3, align 8, !dbg !38
  %6 = load ptr, ptr %4, align 8, !dbg !39
  call void %5(ptr noundef %6), !dbg !38
  ret void, !dbg !40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @setup(ptr noundef %0) #0 !dbg !41 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !47, metadata !DIExpression()), !dbg !48
  %3 = load ptr, ptr %2, align 8, !dbg !49
  store ptr @trampoline, ptr %3, align 8, !dbg !50
  ret void, !dbg !51
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !52 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !56, metadata !DIExpression()), !dbg !57
  call void @setup(ptr noundef %2), !dbg !58
  call void @llvm.dbg.declare(metadata ptr %3, metadata !59, metadata !DIExpression()), !dbg !60
  %4 = call ptr @getenv(ptr noundef @.str) #4, !dbg !61
  store ptr %4, ptr %3, align 8, !dbg !60
  %5 = load ptr, ptr %2, align 8, !dbg !62
  %6 = load ptr, ptr %3, align 8, !dbg !63
  call void %5(ptr noundef @final_sink, ptr noundef %6), !dbg !62
  ret i32 0, !dbg !64
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
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 22, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/cg_iterative_resolve.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "402b4c3ffbf92a846c00491584121488")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 4)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !9, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/cg_iterative_resolve.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "402b4c3ffbf92a846c00491584121488")
!9 = !{!0}
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "final_sink", scope: !2, file: !2, line: 7, type: !19, scopeLine: 7, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!19 = !DISubroutineType(types: !20)
!20 = !{null, !21}
!21 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !22, size: 64)
!22 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!23 = !{}
!24 = !DILocalVariable(name: "s", arg: 1, scope: !18, file: !2, line: 7, type: !21)
!25 = !DILocation(line: 7, column: 29, scope: !18)
!26 = !DILocation(line: 7, column: 41, scope: !18)
!27 = !DILocation(line: 7, column: 34, scope: !18)
!28 = !DILocation(line: 7, column: 45, scope: !18)
!29 = distinct !DISubprogram(name: "trampoline", scope: !2, file: !2, line: 9, type: !30, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!30 = !DISubroutineType(types: !31)
!31 = !{null, !32, !21}
!32 = !DIDerivedType(tag: DW_TAG_typedef, name: "sink_fn", file: !2, line: 5, baseType: !33)
!33 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !19, size: 64)
!34 = !DILocalVariable(name: "fn", arg: 1, scope: !29, file: !2, line: 9, type: !32)
!35 = !DILocation(line: 9, column: 25, scope: !29)
!36 = !DILocalVariable(name: "data", arg: 2, scope: !29, file: !2, line: 9, type: !21)
!37 = !DILocation(line: 9, column: 41, scope: !29)
!38 = !DILocation(line: 10, column: 5, scope: !29)
!39 = !DILocation(line: 10, column: 8, scope: !29)
!40 = !DILocation(line: 11, column: 1, scope: !29)
!41 = distinct !DISubprogram(name: "setup", scope: !2, file: !2, line: 15, type: !42, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!42 = !DISubroutineType(types: !43)
!43 = !{null, !44}
!44 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !45, size: 64)
!45 = !DIDerivedType(tag: DW_TAG_typedef, name: "dispatch_fn", file: !2, line: 13, baseType: !46)
!46 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !30, size: 64)
!47 = !DILocalVariable(name: "out", arg: 1, scope: !41, file: !2, line: 15, type: !44)
!48 = !DILocation(line: 15, column: 25, scope: !41)
!49 = !DILocation(line: 16, column: 6, scope: !41)
!50 = !DILocation(line: 16, column: 10, scope: !41)
!51 = !DILocation(line: 17, column: 1, scope: !41)
!52 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 19, type: !53, scopeLine: 19, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !23)
!53 = !DISubroutineType(types: !54)
!54 = !{!55}
!55 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!56 = !DILocalVariable(name: "f", scope: !52, file: !2, line: 20, type: !45)
!57 = !DILocation(line: 20, column: 17, scope: !52)
!58 = !DILocation(line: 21, column: 5, scope: !52)
!59 = !DILocalVariable(name: "input", scope: !52, file: !2, line: 22, type: !21)
!60 = !DILocation(line: 22, column: 17, scope: !52)
!61 = !DILocation(line: 22, column: 25, scope: !52)
!62 = !DILocation(line: 23, column: 5, scope: !52)
!63 = !DILocation(line: 23, column: 19, scope: !52)
!64 = !DILocation(line: 24, column: 5, scope: !52)
