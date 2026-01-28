; ModuleID = '/workspace/tests/programs/c/cspta_nested_wrappers.c'
source_filename = "/workspace/tests/programs/c/cspta_nested_wrappers.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @safe_malloc(i32 noundef %0) #0 !dbg !14 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !20, metadata !DIExpression()), !dbg !21
  call void @llvm.dbg.declare(metadata ptr %4, metadata !22, metadata !DIExpression()), !dbg !23
  %5 = load i32, ptr %3, align 4, !dbg !24
  %6 = sext i32 %5 to i64, !dbg !24
  %7 = call noalias ptr @malloc(i64 noundef %6) #4, !dbg !25
  store ptr %7, ptr %4, align 8, !dbg !23
  %8 = load ptr, ptr %4, align 8, !dbg !26
  %9 = icmp ne ptr %8, null, !dbg !26
  br i1 %9, label %11, label %10, !dbg !28

10:                                               ; preds = %1
  store ptr null, ptr %2, align 8, !dbg !29
  br label %13, !dbg !29

11:                                               ; preds = %1
  %12 = load ptr, ptr %4, align 8, !dbg !31
  store ptr %12, ptr %2, align 8, !dbg !32
  br label %13, !dbg !32

13:                                               ; preds = %11, %10
  %14 = load ptr, ptr %2, align 8, !dbg !33
  ret ptr %14, !dbg !33
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @alloc_buffer(i32 noundef %0) #0 !dbg !34 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !35, metadata !DIExpression()), !dbg !36
  %3 = load i32, ptr %2, align 4, !dbg !37
  %4 = call ptr @safe_malloc(i32 noundef %3), !dbg !38
  ret ptr %4, !dbg !39
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !40 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !43, metadata !DIExpression()), !dbg !44
  %4 = call ptr @alloc_buffer(i32 noundef 64), !dbg !45
  store ptr %4, ptr %2, align 8, !dbg !44
  call void @llvm.dbg.declare(metadata ptr %3, metadata !46, metadata !DIExpression()), !dbg !47
  %5 = call ptr @alloc_buffer(i32 noundef 128), !dbg !48
  store ptr %5, ptr %3, align 8, !dbg !47
  %6 = load ptr, ptr %2, align 8, !dbg !49
  %7 = icmp ne ptr %6, null, !dbg !49
  br i1 %7, label %8, label %11, !dbg !51

8:                                                ; preds = %0
  %9 = load ptr, ptr %2, align 8, !dbg !52
  %10 = getelementptr inbounds i8, ptr %9, i64 0, !dbg !54
  store i8 65, ptr %10, align 1, !dbg !55
  br label %11, !dbg !56

11:                                               ; preds = %8, %0
  %12 = load ptr, ptr %3, align 8, !dbg !57
  %13 = icmp ne ptr %12, null, !dbg !57
  br i1 %13, label %14, label %17, !dbg !59

14:                                               ; preds = %11
  %15 = load ptr, ptr %3, align 8, !dbg !60
  %16 = getelementptr inbounds i8, ptr %15, i64 0, !dbg !62
  store i8 66, ptr %16, align 1, !dbg !63
  br label %17, !dbg !64

17:                                               ; preds = %14, %11
  %18 = load ptr, ptr %2, align 8, !dbg !65
  call void @free(ptr noundef %18) #5, !dbg !66
  %19 = load ptr, ptr %3, align 8, !dbg !67
  call void @free(ptr noundef %19) #5, !dbg !68
  ret i32 0, !dbg !69
}

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind allocsize(0) }
attributes #5 = { nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!6, !7, !8, !9, !10, !11, !12}
!llvm.ident = !{!13}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/cspta_nested_wrappers.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "7ccc3c35993433c0f5dc94a6d938fbe4")
!2 = !{!3, !4}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!4 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!5 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!6 = !{i32 7, !"Dwarf Version", i32 5}
!7 = !{i32 2, !"Debug Info Version", i32 3}
!8 = !{i32 1, !"wchar_size", i32 4}
!9 = !{i32 8, !"PIC Level", i32 2}
!10 = !{i32 7, !"PIE Level", i32 2}
!11 = !{i32 7, !"uwtable", i32 2}
!12 = !{i32 7, !"frame-pointer", i32 1}
!13 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!14 = distinct !DISubprogram(name: "safe_malloc", scope: !15, file: !15, line: 10, type: !16, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!15 = !DIFile(filename: "c/cspta_nested_wrappers.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "7ccc3c35993433c0f5dc94a6d938fbe4")
!16 = !DISubroutineType(types: !17)
!17 = !{!3, !18}
!18 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!19 = !{}
!20 = !DILocalVariable(name: "size", arg: 1, scope: !14, file: !15, line: 10, type: !18)
!21 = !DILocation(line: 10, column: 23, scope: !14)
!22 = !DILocalVariable(name: "p", scope: !14, file: !15, line: 11, type: !3)
!23 = !DILocation(line: 11, column: 11, scope: !14)
!24 = !DILocation(line: 11, column: 22, scope: !14)
!25 = !DILocation(line: 11, column: 15, scope: !14)
!26 = !DILocation(line: 12, column: 10, scope: !27)
!27 = distinct !DILexicalBlock(scope: !14, file: !15, line: 12, column: 9)
!28 = !DILocation(line: 12, column: 9, scope: !14)
!29 = !DILocation(line: 13, column: 9, scope: !30)
!30 = distinct !DILexicalBlock(scope: !27, file: !15, line: 12, column: 13)
!31 = !DILocation(line: 15, column: 12, scope: !14)
!32 = !DILocation(line: 15, column: 5, scope: !14)
!33 = !DILocation(line: 16, column: 1, scope: !14)
!34 = distinct !DISubprogram(name: "alloc_buffer", scope: !15, file: !15, line: 18, type: !16, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!35 = !DILocalVariable(name: "size", arg: 1, scope: !34, file: !15, line: 18, type: !18)
!36 = !DILocation(line: 18, column: 24, scope: !34)
!37 = !DILocation(line: 19, column: 24, scope: !34)
!38 = !DILocation(line: 19, column: 12, scope: !34)
!39 = !DILocation(line: 19, column: 5, scope: !34)
!40 = distinct !DISubprogram(name: "main", scope: !15, file: !15, line: 22, type: !41, scopeLine: 22, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!41 = !DISubroutineType(types: !42)
!42 = !{!18}
!43 = !DILocalVariable(name: "buf1", scope: !40, file: !15, line: 23, type: !3)
!44 = !DILocation(line: 23, column: 11, scope: !40)
!45 = !DILocation(line: 23, column: 18, scope: !40)
!46 = !DILocalVariable(name: "buf2", scope: !40, file: !15, line: 24, type: !3)
!47 = !DILocation(line: 24, column: 11, scope: !40)
!48 = !DILocation(line: 24, column: 18, scope: !40)
!49 = !DILocation(line: 28, column: 9, scope: !50)
!50 = distinct !DILexicalBlock(scope: !40, file: !15, line: 28, column: 9)
!51 = !DILocation(line: 28, column: 9, scope: !40)
!52 = !DILocation(line: 29, column: 18, scope: !53)
!53 = distinct !DILexicalBlock(scope: !50, file: !15, line: 28, column: 15)
!54 = !DILocation(line: 29, column: 9, scope: !53)
!55 = !DILocation(line: 29, column: 27, scope: !53)
!56 = !DILocation(line: 30, column: 5, scope: !53)
!57 = !DILocation(line: 31, column: 9, scope: !58)
!58 = distinct !DILexicalBlock(scope: !40, file: !15, line: 31, column: 9)
!59 = !DILocation(line: 31, column: 9, scope: !40)
!60 = !DILocation(line: 32, column: 18, scope: !61)
!61 = distinct !DILexicalBlock(scope: !58, file: !15, line: 31, column: 15)
!62 = !DILocation(line: 32, column: 9, scope: !61)
!63 = !DILocation(line: 32, column: 27, scope: !61)
!64 = !DILocation(line: 33, column: 5, scope: !61)
!65 = !DILocation(line: 35, column: 10, scope: !40)
!66 = !DILocation(line: 35, column: 5, scope: !40)
!67 = !DILocation(line: 36, column: 10, scope: !40)
!68 = !DILocation(line: 36, column: 5, scope: !40)
!69 = !DILocation(line: 37, column: 5, scope: !40)
