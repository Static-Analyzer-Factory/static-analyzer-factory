; ModuleID = '/workspace/tests/programs/c/sql_injection.c'
source_filename = "/workspace/tests/programs/c/sql_injection.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [6 x i8] c"QUERY\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !21 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !26, metadata !DIExpression()), !dbg !29
  %5 = call ptr @getenv(ptr noundef @.str) #4, !dbg !30
  store ptr %5, ptr %2, align 8, !dbg !29
  call void @llvm.dbg.declare(metadata ptr %3, metadata !31, metadata !DIExpression()), !dbg !32
  %6 = load ptr, ptr %2, align 8, !dbg !33
  store ptr %6, ptr %3, align 8, !dbg !32
  call void @llvm.dbg.declare(metadata ptr %4, metadata !34, metadata !DIExpression()), !dbg !38
  store ptr null, ptr %4, align 8, !dbg !38
  %7 = load ptr, ptr %4, align 8, !dbg !39
  %8 = load ptr, ptr %3, align 8, !dbg !40
  %9 = call i32 @sqlite3_exec(ptr noundef %7, ptr noundef %8, ptr noundef null, ptr noundef null, ptr noundef null), !dbg !41
  ret i32 0, !dbg !42
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

declare i32 @sqlite3_exec(ptr noundef, ptr noundef, ptr noundef, ptr noundef, ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!13, !14, !15, !16, !17, !18, !19}
!llvm.ident = !{!20}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 14, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/sql_injection.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "e7a522d0079c1457a291d1926b7fd873")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 48, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 6)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !9, globals: !12, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/sql_injection.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "e7a522d0079c1457a291d1926b7fd873")
!9 = !{!10, !11}
!10 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!11 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!12 = !{!0}
!13 = !{i32 7, !"Dwarf Version", i32 5}
!14 = !{i32 2, !"Debug Info Version", i32 3}
!15 = !{i32 1, !"wchar_size", i32 4}
!16 = !{i32 8, !"PIC Level", i32 2}
!17 = !{i32 7, !"PIE Level", i32 2}
!18 = !{i32 7, !"uwtable", i32 2}
!19 = !{i32 7, !"frame-pointer", i32 1}
!20 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!21 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 13, type: !22, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !25)
!22 = !DISubroutineType(types: !23)
!23 = !{!24}
!24 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!25 = !{}
!26 = !DILocalVariable(name: "user_input", scope: !21, file: !2, line: 14, type: !27)
!27 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !28, size: 64)
!28 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!29 = !DILocation(line: 14, column: 17, scope: !21)
!30 = !DILocation(line: 14, column: 30, scope: !21)
!31 = !DILocalVariable(name: "query_str", scope: !21, file: !2, line: 15, type: !10)
!32 = !DILocation(line: 15, column: 11, scope: !21)
!33 = !DILocation(line: 15, column: 31, scope: !21)
!34 = !DILocalVariable(name: "db", scope: !21, file: !2, line: 16, type: !35)
!35 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !36, size: 64)
!36 = !DIDerivedType(tag: DW_TAG_typedef, name: "sqlite3", file: !2, line: 10, baseType: !37)
!37 = !DICompositeType(tag: DW_TAG_structure_type, name: "sqlite3", file: !2, line: 10, flags: DIFlagFwdDecl)
!38 = !DILocation(line: 16, column: 14, scope: !21)
!39 = !DILocation(line: 17, column: 18, scope: !21)
!40 = !DILocation(line: 17, column: 22, scope: !21)
!41 = !DILocation(line: 17, column: 5, scope: !21)
!42 = !DILocation(line: 18, column: 5, scope: !21)
