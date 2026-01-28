; ModuleID = 'tests/programs/c/z3_vf_sanitized_path.c'
source_filename = "tests/programs/c/z3_vf_sanitized_path.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [8 x i8] c"REQUEST\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [10 x i8] c"echo safe\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @handle_request(i32 noundef %0) #0 !dbg !22 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca [256 x i8], align 1
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  call void @llvm.dbg.declare(metadata ptr %3, metadata !29, metadata !DIExpression()), !dbg !31
  %5 = call ptr @getenv(ptr noundef @.str) #4, !dbg !32
  store ptr %5, ptr %3, align 8, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %4, metadata !33, metadata !DIExpression()), !dbg !37
  %6 = load i32, ptr %2, align 4, !dbg !38
  %7 = icmp ne i32 %6, 0, !dbg !38
  br i1 %7, label %8, label %11, !dbg !40

8:                                                ; preds = %1
  %9 = getelementptr inbounds [256 x i8], ptr %4, i64 0, i64 0, !dbg !41
  %10 = call ptr @strcpy(ptr noundef %9, ptr noundef @.str.1) #4, !dbg !43
  br label %16, !dbg !44

11:                                               ; preds = %1
  %12 = getelementptr inbounds [256 x i8], ptr %4, i64 0, i64 0, !dbg !45
  %13 = load ptr, ptr %3, align 8, !dbg !47
  %14 = call ptr @strncpy(ptr noundef %12, ptr noundef %13, i64 noundef 255) #4, !dbg !48
  %15 = getelementptr inbounds [256 x i8], ptr %4, i64 0, i64 255, !dbg !49
  store i8 0, ptr %15, align 1, !dbg !50
  br label %16

16:                                               ; preds = %11, %8
  %17 = getelementptr inbounds [256 x i8], ptr %4, i64 0, i64 0, !dbg !51
  %18 = call i32 @system(ptr noundef %17), !dbg !52
  ret void, !dbg !53
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #2

; Function Attrs: nounwind
declare ptr @strcpy(ptr noundef, ptr noundef) #2

; Function Attrs: nounwind
declare ptr @strncpy(ptr noundef, ptr noundef, i64 noundef) #2

declare i32 @system(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !54 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @handle_request(i32 noundef 1), !dbg !57
  ret i32 0, !dbg !58
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!14, !15, !16, !17, !18, !19, !20}
!llvm.ident = !{!21}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 10, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "tests/programs/c/z3_vf_sanitized_path.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "638d0b7ae5dca458c93712e98f65225f")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 64, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 8)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 15, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 80, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 10)
!12 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !13, splitDebugInlining: false, nameTableKind: None)
!13 = !{!0, !7}
!14 = !{i32 7, !"Dwarf Version", i32 5}
!15 = !{i32 2, !"Debug Info Version", i32 3}
!16 = !{i32 1, !"wchar_size", i32 4}
!17 = !{i32 8, !"PIC Level", i32 2}
!18 = !{i32 7, !"PIE Level", i32 2}
!19 = !{i32 7, !"uwtable", i32 2}
!20 = !{i32 7, !"frame-pointer", i32 1}
!21 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!22 = distinct !DISubprogram(name: "handle_request", scope: !2, file: !2, line: 9, type: !23, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !26)
!23 = !DISubroutineType(types: !24)
!24 = !{null, !25}
!25 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!26 = !{}
!27 = !DILocalVariable(name: "trusted", arg: 1, scope: !22, file: !2, line: 9, type: !25)
!28 = !DILocation(line: 9, column: 25, scope: !22)
!29 = !DILocalVariable(name: "input", scope: !22, file: !2, line: 10, type: !30)
!30 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!31 = !DILocation(line: 10, column: 11, scope: !22)
!32 = !DILocation(line: 10, column: 19, scope: !22)
!33 = !DILocalVariable(name: "buf", scope: !22, file: !2, line: 11, type: !34)
!34 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 2048, elements: !35)
!35 = !{!36}
!36 = !DISubrange(count: 256)
!37 = !DILocation(line: 11, column: 10, scope: !22)
!38 = !DILocation(line: 13, column: 9, scope: !39)
!39 = distinct !DILexicalBlock(scope: !22, file: !2, line: 13, column: 9)
!40 = !DILocation(line: 13, column: 9, scope: !22)
!41 = !DILocation(line: 15, column: 16, scope: !42)
!42 = distinct !DILexicalBlock(scope: !39, file: !2, line: 13, column: 18)
!43 = !DILocation(line: 15, column: 9, scope: !42)
!44 = !DILocation(line: 16, column: 5, scope: !42)
!45 = !DILocation(line: 18, column: 17, scope: !46)
!46 = distinct !DILexicalBlock(scope: !39, file: !2, line: 16, column: 12)
!47 = !DILocation(line: 18, column: 22, scope: !46)
!48 = !DILocation(line: 18, column: 9, scope: !46)
!49 = !DILocation(line: 19, column: 9, scope: !46)
!50 = !DILocation(line: 19, column: 18, scope: !46)
!51 = !DILocation(line: 24, column: 12, scope: !22)
!52 = !DILocation(line: 24, column: 5, scope: !22)
!53 = !DILocation(line: 25, column: 1, scope: !22)
!54 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 27, type: !55, scopeLine: 27, spFlags: DISPFlagDefinition, unit: !12)
!55 = !DISubroutineType(types: !56)
!56 = !{!25}
!57 = !DILocation(line: 28, column: 5, scope: !54)
!58 = !DILocation(line: 29, column: 5, scope: !54)
