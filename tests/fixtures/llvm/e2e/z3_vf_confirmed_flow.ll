; ModuleID = 'tests/programs/c/z3_vf_confirmed_flow.c'
source_filename = "tests/programs/c/z3_vf_confirmed_flow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"USER_CMD\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @execute() #0 !dbg !17 {
  %1 = alloca ptr, align 8
  %2 = alloca [512 x i8], align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !21, metadata !DIExpression()), !dbg !23
  %3 = call ptr @getenv(ptr noundef @.str) #4, !dbg !24
  store ptr %3, ptr %1, align 8, !dbg !23
  call void @llvm.dbg.declare(metadata ptr %2, metadata !25, metadata !DIExpression()), !dbg !29
  %4 = getelementptr inbounds [512 x i8], ptr %2, i64 0, i64 0, !dbg !30
  %5 = load ptr, ptr %1, align 8, !dbg !31
  %6 = call ptr @strncpy(ptr noundef %4, ptr noundef %5, i64 noundef 511) #4, !dbg !32
  %7 = getelementptr inbounds [512 x i8], ptr %2, i64 0, i64 511, !dbg !33
  store i8 0, ptr %7, align 1, !dbg !34
  %8 = getelementptr inbounds [512 x i8], ptr %2, i64 0, i64 0, !dbg !35
  %9 = call i32 @system(ptr noundef %8), !dbg !36
  ret void, !dbg !37
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

; Function Attrs: nounwind
declare ptr @strncpy(ptr noundef, ptr noundef, i64 noundef) #2

declare i32 @system(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !38 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @execute(), !dbg !42
  ret i32 0, !dbg !43
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
!2 = !DIFile(filename: "tests/programs/c/z3_vf_confirmed_flow.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "26c0d629bd60bd2ea863cfeed217f8cf")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 72, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 9)
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
!17 = distinct !DISubprogram(name: "execute", scope: !2, file: !2, line: 8, type: !18, scopeLine: 8, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !20)
!18 = !DISubroutineType(types: !19)
!19 = !{null}
!20 = !{}
!21 = !DILocalVariable(name: "cmd", scope: !17, file: !2, line: 9, type: !22)
!22 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!23 = !DILocation(line: 9, column: 11, scope: !17)
!24 = !DILocation(line: 9, column: 17, scope: !17)
!25 = !DILocalVariable(name: "buf", scope: !17, file: !2, line: 10, type: !26)
!26 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 4096, elements: !27)
!27 = !{!28}
!28 = !DISubrange(count: 512)
!29 = !DILocation(line: 10, column: 10, scope: !17)
!30 = !DILocation(line: 11, column: 13, scope: !17)
!31 = !DILocation(line: 11, column: 18, scope: !17)
!32 = !DILocation(line: 11, column: 5, scope: !17)
!33 = !DILocation(line: 12, column: 5, scope: !17)
!34 = !DILocation(line: 12, column: 14, scope: !17)
!35 = !DILocation(line: 14, column: 12, scope: !17)
!36 = !DILocation(line: 14, column: 5, scope: !17)
!37 = !DILocation(line: 15, column: 1, scope: !17)
!38 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 17, type: !39, scopeLine: 17, spFlags: DISPFlagDefinition, unit: !7)
!39 = !DISubroutineType(types: !40)
!40 = !{!41}
!41 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!42 = !DILocation(line: 18, column: 5, scope: !38)
!43 = !DILocation(line: 19, column: 5, scope: !38)
