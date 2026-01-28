; ModuleID = '/workspace/tests/programs/c/info_leak.c'
source_filename = "/workspace/tests/programs/c/info_leak.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [17 x i8] c"password=hunter2\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @handle_request(i32 noundef %0) #0 !dbg !18 {
  %2 = alloca i32, align 4
  %3 = alloca [64 x i8], align 1
  %4 = alloca [128 x i8], align 1
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !23, metadata !DIExpression()), !dbg !24
  call void @llvm.dbg.declare(metadata ptr %3, metadata !25, metadata !DIExpression()), !dbg !29
  %5 = getelementptr inbounds [64 x i8], ptr %3, i64 0, i64 0, !dbg !30
  %6 = call ptr @strcpy(ptr noundef %5, ptr noundef @.str) #5, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %4, metadata !32, metadata !DIExpression()), !dbg !36
  %7 = getelementptr inbounds [128 x i8], ptr %4, i64 0, i64 0, !dbg !37
  %8 = getelementptr inbounds [64 x i8], ptr %3, i64 0, i64 0, !dbg !37
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %7, ptr align 1 %8, i64 64, i1 false), !dbg !37
  %9 = load i32, ptr %2, align 4, !dbg !38
  %10 = getelementptr inbounds [128 x i8], ptr %4, i64 0, i64 0, !dbg !39
  %11 = call i64 @send(i32 noundef %9, ptr noundef %10, i64 noundef 128, i32 noundef 0), !dbg !40
  ret void, !dbg !41
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind
declare ptr @strcpy(ptr noundef, ptr noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #3

declare i64 @send(i32 noundef, ptr noundef, i64 noundef, i32 noundef) #4

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !42 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @handle_request(i32 noundef 3), !dbg !45
  ret i32 0, !dbg !46
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!10, !11, !12, !13, !14, !15, !16}
!llvm.ident = !{!17}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 10, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/info_leak.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "72075258ba1a76129c441df0b3446406")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 136, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 17)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !8, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !9, splitDebugInlining: false, nameTableKind: None)
!8 = !DIFile(filename: "/workspace/tests/programs/c/info_leak.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "72075258ba1a76129c441df0b3446406")
!9 = !{!0}
!10 = !{i32 7, !"Dwarf Version", i32 5}
!11 = !{i32 2, !"Debug Info Version", i32 3}
!12 = !{i32 1, !"wchar_size", i32 4}
!13 = !{i32 8, !"PIC Level", i32 2}
!14 = !{i32 7, !"PIE Level", i32 2}
!15 = !{i32 7, !"uwtable", i32 2}
!16 = !{i32 7, !"frame-pointer", i32 1}
!17 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!18 = distinct !DISubprogram(name: "handle_request", scope: !2, file: !2, line: 8, type: !19, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !22)
!19 = !DISubroutineType(types: !20)
!20 = !{null, !21}
!21 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!22 = !{}
!23 = !DILocalVariable(name: "sockfd", arg: 1, scope: !18, file: !2, line: 8, type: !21)
!24 = !DILocation(line: 8, column: 25, scope: !18)
!25 = !DILocalVariable(name: "secret", scope: !18, file: !2, line: 9, type: !26)
!26 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 512, elements: !27)
!27 = !{!28}
!28 = !DISubrange(count: 64)
!29 = !DILocation(line: 9, column: 10, scope: !18)
!30 = !DILocation(line: 10, column: 12, scope: !18)
!31 = !DILocation(line: 10, column: 5, scope: !18)
!32 = !DILocalVariable(name: "response", scope: !18, file: !2, line: 11, type: !33)
!33 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 1024, elements: !34)
!34 = !{!35}
!35 = !DISubrange(count: 128)
!36 = !DILocation(line: 11, column: 10, scope: !18)
!37 = !DILocation(line: 12, column: 5, scope: !18)
!38 = !DILocation(line: 13, column: 10, scope: !18)
!39 = !DILocation(line: 13, column: 18, scope: !18)
!40 = !DILocation(line: 13, column: 5, scope: !18)
!41 = !DILocation(line: 14, column: 1, scope: !18)
!42 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 16, type: !43, scopeLine: 16, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7)
!43 = !DISubroutineType(types: !44)
!44 = !{!21}
!45 = !DILocation(line: 17, column: 5, scope: !42)
!46 = !DILocation(line: 18, column: 5, scope: !42)
