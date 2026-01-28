; ModuleID = 'tests/programs/c/memcpy_overflow.c'
source_filename = "tests/programs/c/memcpy_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@__const.memmove_overflow_bad.src = private unnamed_addr constant [32 x i8] c"hello world this is a long text\00", align 1
@__const.memmove_overflow_good.src = private unnamed_addr constant [32 x i8] c"hello world this is a long text\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memcpy_overflow_bad() #0 !dbg !13 {
  %1 = alloca ptr, align 8
  %2 = alloca [10 x i32], align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !17, metadata !DIExpression()), !dbg !18
  %3 = call noalias ptr @malloc(i64 noundef 10) #6, !dbg !19
  store ptr %3, ptr %1, align 8, !dbg !18
  call void @llvm.dbg.declare(metadata ptr %2, metadata !20, metadata !DIExpression()), !dbg !24
  call void @llvm.memset.p0.i64(ptr align 4 %2, i8 0, i64 40, i1 false), !dbg !24
  %4 = load ptr, ptr %1, align 8, !dbg !25
  %5 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !26
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %4, ptr align 4 %5, i64 40, i1 false), !dbg !26
  %6 = load ptr, ptr %1, align 8, !dbg !27
  call void @free(ptr noundef %6) #7, !dbg !28
  ret void, !dbg !29
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #3

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #4

; Function Attrs: nounwind
declare void @free(ptr noundef) #5

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memcpy_overflow_good() #0 !dbg !30 {
  %1 = alloca ptr, align 8
  %2 = alloca [10 x i32], align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !31, metadata !DIExpression()), !dbg !32
  %3 = call noalias ptr @malloc(i64 noundef 40) #6, !dbg !33
  store ptr %3, ptr %1, align 8, !dbg !32
  call void @llvm.dbg.declare(metadata ptr %2, metadata !34, metadata !DIExpression()), !dbg !35
  call void @llvm.memset.p0.i64(ptr align 4 %2, i8 0, i64 40, i1 false), !dbg !35
  %4 = load ptr, ptr %1, align 8, !dbg !36
  %5 = getelementptr inbounds [10 x i32], ptr %2, i64 0, i64 0, !dbg !37
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %4, ptr align 4 %5, i64 40, i1 false), !dbg !37
  %6 = load ptr, ptr %1, align 8, !dbg !38
  call void @free(ptr noundef %6) #7, !dbg !39
  ret void, !dbg !40
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memmove_overflow_bad() #0 !dbg !41 {
  %1 = alloca [16 x i8], align 1
  %2 = alloca [32 x i8], align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !42, metadata !DIExpression()), !dbg !47
  call void @llvm.dbg.declare(metadata ptr %2, metadata !48, metadata !DIExpression()), !dbg !52
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %2, ptr align 1 @__const.memmove_overflow_bad.src, i64 32, i1 false), !dbg !52
  %3 = getelementptr inbounds [16 x i8], ptr %1, i64 0, i64 0, !dbg !53
  %4 = getelementptr inbounds [32 x i8], ptr %2, i64 0, i64 0, !dbg !53
  call void @llvm.memmove.p0.p0.i64(ptr align 1 %3, ptr align 1 %4, i64 32, i1 false), !dbg !53
  ret void, !dbg !54
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memmove.p0.p0.i64(ptr nocapture writeonly, ptr nocapture readonly, i64, i1 immarg) #4

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memmove_overflow_good() #0 !dbg !55 {
  %1 = alloca [32 x i8], align 1
  %2 = alloca [32 x i8], align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !56, metadata !DIExpression()), !dbg !57
  call void @llvm.dbg.declare(metadata ptr %2, metadata !58, metadata !DIExpression()), !dbg !59
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %2, ptr align 1 @__const.memmove_overflow_good.src, i64 32, i1 false), !dbg !59
  %3 = getelementptr inbounds [32 x i8], ptr %1, i64 0, i64 0, !dbg !60
  %4 = getelementptr inbounds [32 x i8], ptr %2, i64 0, i64 0, !dbg !60
  call void @llvm.memmove.p0.p0.i64(ptr align 1 %3, ptr align 1 %4, i64 32, i1 false), !dbg !60
  ret void, !dbg !61
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memset_overflow_bad() #0 !dbg !62 {
  %1 = alloca [10 x i8], align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !63, metadata !DIExpression()), !dbg !65
  %2 = getelementptr inbounds [10 x i8], ptr %1, i64 0, i64 0, !dbg !66
  call void @llvm.memset.p0.i64(ptr align 1 %2, i8 0, i64 20, i1 false), !dbg !66
  ret void, !dbg !67
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @memset_overflow_good() #0 !dbg !68 {
  %1 = alloca [20 x i8], align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !69, metadata !DIExpression()), !dbg !73
  %2 = getelementptr inbounds [20 x i8], ptr %1, i64 0, i64 0, !dbg !74
  call void @llvm.memset.p0.i64(ptr align 1 %2, i8 0, i64 20, i1 false), !dbg !74
  ret void, !dbg !75
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !76 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @memcpy_overflow_bad(), !dbg !79
  call void @memcpy_overflow_good(), !dbg !80
  call void @memmove_overflow_bad(), !dbg !81
  call void @memmove_overflow_good(), !dbg !82
  call void @memset_overflow_bad(), !dbg !83
  call void @memset_overflow_good(), !dbg !84
  ret i32 0, !dbg !85
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #6 = { nounwind allocsize(0) }
attributes #7 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/programs/c/memcpy_overflow.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "98d15fc2dea37fc18570ea0e31a71da0")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 1}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "memcpy_overflow_bad", scope: !1, file: !1, line: 10, type: !14, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{null}
!16 = !{}
!17 = !DILocalVariable(name: "data", scope: !13, file: !1, line: 11, type: !3)
!18 = !DILocation(line: 11, column: 10, scope: !13)
!19 = !DILocation(line: 11, column: 24, scope: !13)
!20 = !DILocalVariable(name: "source", scope: !13, file: !1, line: 12, type: !21)
!21 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 320, elements: !22)
!22 = !{!23}
!23 = !DISubrange(count: 10)
!24 = !DILocation(line: 12, column: 9, scope: !13)
!25 = !DILocation(line: 14, column: 12, scope: !13)
!26 = !DILocation(line: 14, column: 5, scope: !13)
!27 = !DILocation(line: 15, column: 10, scope: !13)
!28 = !DILocation(line: 15, column: 5, scope: !13)
!29 = !DILocation(line: 16, column: 1, scope: !13)
!30 = distinct !DISubprogram(name: "memcpy_overflow_good", scope: !1, file: !1, line: 19, type: !14, scopeLine: 19, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!31 = !DILocalVariable(name: "data", scope: !30, file: !1, line: 20, type: !3)
!32 = !DILocation(line: 20, column: 10, scope: !30)
!33 = !DILocation(line: 20, column: 24, scope: !30)
!34 = !DILocalVariable(name: "source", scope: !30, file: !1, line: 21, type: !21)
!35 = !DILocation(line: 21, column: 9, scope: !30)
!36 = !DILocation(line: 23, column: 12, scope: !30)
!37 = !DILocation(line: 23, column: 5, scope: !30)
!38 = !DILocation(line: 24, column: 10, scope: !30)
!39 = !DILocation(line: 24, column: 5, scope: !30)
!40 = !DILocation(line: 25, column: 1, scope: !30)
!41 = distinct !DISubprogram(name: "memmove_overflow_bad", scope: !1, file: !1, line: 28, type: !14, scopeLine: 28, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!42 = !DILocalVariable(name: "dest", scope: !41, file: !1, line: 29, type: !43)
!43 = !DICompositeType(tag: DW_TAG_array_type, baseType: !44, size: 128, elements: !45)
!44 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!45 = !{!46}
!46 = !DISubrange(count: 16)
!47 = !DILocation(line: 29, column: 10, scope: !41)
!48 = !DILocalVariable(name: "src", scope: !41, file: !1, line: 30, type: !49)
!49 = !DICompositeType(tag: DW_TAG_array_type, baseType: !44, size: 256, elements: !50)
!50 = !{!51}
!51 = !DISubrange(count: 32)
!52 = !DILocation(line: 30, column: 10, scope: !41)
!53 = !DILocation(line: 32, column: 5, scope: !41)
!54 = !DILocation(line: 33, column: 1, scope: !41)
!55 = distinct !DISubprogram(name: "memmove_overflow_good", scope: !1, file: !1, line: 36, type: !14, scopeLine: 36, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!56 = !DILocalVariable(name: "dest", scope: !55, file: !1, line: 37, type: !49)
!57 = !DILocation(line: 37, column: 10, scope: !55)
!58 = !DILocalVariable(name: "src", scope: !55, file: !1, line: 38, type: !49)
!59 = !DILocation(line: 38, column: 10, scope: !55)
!60 = !DILocation(line: 40, column: 5, scope: !55)
!61 = !DILocation(line: 41, column: 1, scope: !55)
!62 = distinct !DISubprogram(name: "memset_overflow_bad", scope: !1, file: !1, line: 44, type: !14, scopeLine: 44, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!63 = !DILocalVariable(name: "buffer", scope: !62, file: !1, line: 45, type: !64)
!64 = !DICompositeType(tag: DW_TAG_array_type, baseType: !44, size: 80, elements: !22)
!65 = !DILocation(line: 45, column: 10, scope: !62)
!66 = !DILocation(line: 47, column: 5, scope: !62)
!67 = !DILocation(line: 48, column: 1, scope: !62)
!68 = distinct !DISubprogram(name: "memset_overflow_good", scope: !1, file: !1, line: 51, type: !14, scopeLine: 51, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!69 = !DILocalVariable(name: "buffer", scope: !68, file: !1, line: 52, type: !70)
!70 = !DICompositeType(tag: DW_TAG_array_type, baseType: !44, size: 160, elements: !71)
!71 = !{!72}
!72 = !DISubrange(count: 20)
!73 = !DILocation(line: 52, column: 10, scope: !68)
!74 = !DILocation(line: 54, column: 5, scope: !68)
!75 = !DILocation(line: 55, column: 1, scope: !68)
!76 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 57, type: !77, scopeLine: 57, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!77 = !DISubroutineType(types: !78)
!78 = !{!4}
!79 = !DILocation(line: 58, column: 5, scope: !76)
!80 = !DILocation(line: 59, column: 5, scope: !76)
!81 = !DILocation(line: 60, column: 5, scope: !76)
!82 = !DILocation(line: 61, column: 5, scope: !76)
!83 = !DILocation(line: 62, column: 5, scope: !76)
!84 = !DILocation(line: 63, column: 5, scope: !76)
!85 = !DILocation(line: 64, column: 5, scope: !76)
