; ModuleID = '/workspace/tests/programs/c/integer_overflow.c'
source_filename = "/workspace/tests/programs/c/integer_overflow.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @alloc_array(i32 noundef %0, i32 noundef %1) #0 !dbg !10 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !19, metadata !DIExpression()), !dbg !20
  call void @llvm.dbg.declare(metadata ptr %5, metadata !21, metadata !DIExpression()), !dbg !22
  %7 = load i32, ptr %3, align 4, !dbg !23
  %8 = load i32, ptr %4, align 4, !dbg !24
  %9 = mul i32 %7, %8, !dbg !25
  store i32 %9, ptr %5, align 4, !dbg !22
  call void @llvm.dbg.declare(metadata ptr %6, metadata !26, metadata !DIExpression()), !dbg !27
  %10 = load i32, ptr %5, align 4, !dbg !28
  %11 = zext i32 %10 to i64, !dbg !28
  %12 = call noalias ptr @malloc(i64 noundef %11) #5, !dbg !29
  store ptr %12, ptr %6, align 8, !dbg !27
  %13 = load ptr, ptr %6, align 8, !dbg !30
  %14 = icmp ne ptr %13, null, !dbg !30
  br i1 %14, label %15, label %21, !dbg !32

15:                                               ; preds = %2
  %16 = load ptr, ptr %6, align 8, !dbg !33
  %17 = load i32, ptr %3, align 4, !dbg !35
  %18 = load i32, ptr %4, align 4, !dbg !36
  %19 = mul i32 %17, %18, !dbg !37
  %20 = zext i32 %19 to i64, !dbg !35
  call void @llvm.memset.p0.i64(ptr align 1 %16, i8 0, i64 %20, i1 false), !dbg !38
  br label %21, !dbg !39

21:                                               ; preds = %15, %2
  %22 = load ptr, ptr %6, align 8, !dbg !40
  ret ptr %22, !dbg !41
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !42 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !46, metadata !DIExpression()), !dbg !47
  %3 = call ptr @alloc_array(i32 noundef 1073741825, i32 noundef 4), !dbg !48
  store ptr %3, ptr %2, align 8, !dbg !47
  %4 = load ptr, ptr %2, align 8, !dbg !49
  call void @free(ptr noundef %4) #6, !dbg !50
  ret i32 0, !dbg !51
}

; Function Attrs: nounwind
declare void @free(ptr noundef) #4

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #4 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { nounwind allocsize(0) }
attributes #6 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/integer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "c5c9480d94c84087483e9b452a4669ce")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "alloc_array", scope: !11, file: !11, line: 9, type: !12, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!11 = !DIFile(filename: "c/integer_overflow.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "c5c9480d94c84087483e9b452a4669ce")
!12 = !DISubroutineType(types: !13)
!13 = !{!14, !15, !15}
!14 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!15 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!16 = !{}
!17 = !DILocalVariable(name: "count", arg: 1, scope: !10, file: !11, line: 9, type: !15)
!18 = !DILocation(line: 9, column: 32, scope: !10)
!19 = !DILocalVariable(name: "size", arg: 2, scope: !10, file: !11, line: 9, type: !15)
!20 = !DILocation(line: 9, column: 52, scope: !10)
!21 = !DILocalVariable(name: "total", scope: !10, file: !11, line: 10, type: !15)
!22 = !DILocation(line: 10, column: 18, scope: !10)
!23 = !DILocation(line: 10, column: 26, scope: !10)
!24 = !DILocation(line: 10, column: 34, scope: !10)
!25 = !DILocation(line: 10, column: 32, scope: !10)
!26 = !DILocalVariable(name: "buf", scope: !10, file: !11, line: 11, type: !14)
!27 = !DILocation(line: 11, column: 11, scope: !10)
!28 = !DILocation(line: 11, column: 24, scope: !10)
!29 = !DILocation(line: 11, column: 17, scope: !10)
!30 = !DILocation(line: 12, column: 9, scope: !31)
!31 = distinct !DILexicalBlock(scope: !10, file: !11, line: 12, column: 9)
!32 = !DILocation(line: 12, column: 9, scope: !10)
!33 = !DILocation(line: 13, column: 16, scope: !34)
!34 = distinct !DILexicalBlock(scope: !31, file: !11, line: 12, column: 14)
!35 = !DILocation(line: 13, column: 24, scope: !34)
!36 = !DILocation(line: 13, column: 32, scope: !34)
!37 = !DILocation(line: 13, column: 30, scope: !34)
!38 = !DILocation(line: 13, column: 9, scope: !34)
!39 = !DILocation(line: 14, column: 5, scope: !34)
!40 = !DILocation(line: 15, column: 12, scope: !10)
!41 = !DILocation(line: 15, column: 5, scope: !10)
!42 = distinct !DISubprogram(name: "main", scope: !11, file: !11, line: 18, type: !43, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!43 = !DISubroutineType(types: !44)
!44 = !{!45}
!45 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!46 = !DILocalVariable(name: "p", scope: !42, file: !11, line: 19, type: !14)
!47 = !DILocation(line: 19, column: 11, scope: !42)
!48 = !DILocation(line: 19, column: 15, scope: !42)
!49 = !DILocation(line: 20, column: 10, scope: !42)
!50 = !DILocation(line: 20, column: 5, scope: !42)
!51 = !DILocation(line: 21, column: 5, scope: !42)
