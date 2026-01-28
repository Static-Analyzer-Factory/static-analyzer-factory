; ModuleID = '/workspace/tests/programs/c/taint_sanitized.c'
source_filename = "/workspace/tests/programs/c/taint_sanitized.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@sanitize_input.safe = internal global [256 x i8] zeroinitializer, align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @sanitize_input(ptr noundef %0) #0 !dbg !2 {
  %2 = alloca ptr, align 8
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !25, metadata !DIExpression()), !dbg !26
  call void @llvm.dbg.declare(metadata ptr %3, metadata !27, metadata !DIExpression()), !dbg !31
  store i64 0, ptr %3, align 8, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %4, metadata !32, metadata !DIExpression()), !dbg !34
  store i64 0, ptr %4, align 8, !dbg !34
  br label %5, !dbg !35

5:                                                ; preds = %68, %1
  %6 = load ptr, ptr %2, align 8, !dbg !36
  %7 = load i64, ptr %4, align 8, !dbg !38
  %8 = getelementptr inbounds i8, ptr %6, i64 %7, !dbg !36
  %9 = load i8, ptr %8, align 1, !dbg !36
  %10 = zext i8 %9 to i32, !dbg !36
  %11 = icmp ne i32 %10, 0, !dbg !36
  br i1 %11, label %12, label %15, !dbg !39

12:                                               ; preds = %5
  %13 = load i64, ptr %3, align 8, !dbg !40
  %14 = icmp ult i64 %13, 255, !dbg !41
  br label %15

15:                                               ; preds = %12, %5
  %16 = phi i1 [ false, %5 ], [ %14, %12 ], !dbg !42
  br i1 %16, label %17, label %71, !dbg !43

17:                                               ; preds = %15
  %18 = load ptr, ptr %2, align 8, !dbg !44
  %19 = load i64, ptr %4, align 8, !dbg !47
  %20 = getelementptr inbounds i8, ptr %18, i64 %19, !dbg !44
  %21 = load i8, ptr %20, align 1, !dbg !44
  %22 = zext i8 %21 to i32, !dbg !44
  %23 = icmp sge i32 %22, 97, !dbg !48
  br i1 %23, label %24, label %31, !dbg !49

24:                                               ; preds = %17
  %25 = load ptr, ptr %2, align 8, !dbg !50
  %26 = load i64, ptr %4, align 8, !dbg !51
  %27 = getelementptr inbounds i8, ptr %25, i64 %26, !dbg !50
  %28 = load i8, ptr %27, align 1, !dbg !50
  %29 = zext i8 %28 to i32, !dbg !50
  %30 = icmp sle i32 %29, 122, !dbg !52
  br i1 %30, label %59, label %31, !dbg !53

31:                                               ; preds = %24, %17
  %32 = load ptr, ptr %2, align 8, !dbg !54
  %33 = load i64, ptr %4, align 8, !dbg !55
  %34 = getelementptr inbounds i8, ptr %32, i64 %33, !dbg !54
  %35 = load i8, ptr %34, align 1, !dbg !54
  %36 = zext i8 %35 to i32, !dbg !54
  %37 = icmp sge i32 %36, 65, !dbg !56
  br i1 %37, label %38, label %45, !dbg !57

38:                                               ; preds = %31
  %39 = load ptr, ptr %2, align 8, !dbg !58
  %40 = load i64, ptr %4, align 8, !dbg !59
  %41 = getelementptr inbounds i8, ptr %39, i64 %40, !dbg !58
  %42 = load i8, ptr %41, align 1, !dbg !58
  %43 = zext i8 %42 to i32, !dbg !58
  %44 = icmp sle i32 %43, 90, !dbg !60
  br i1 %44, label %59, label %45, !dbg !61

45:                                               ; preds = %38, %31
  %46 = load ptr, ptr %2, align 8, !dbg !62
  %47 = load i64, ptr %4, align 8, !dbg !63
  %48 = getelementptr inbounds i8, ptr %46, i64 %47, !dbg !62
  %49 = load i8, ptr %48, align 1, !dbg !62
  %50 = zext i8 %49 to i32, !dbg !62
  %51 = icmp sge i32 %50, 48, !dbg !64
  br i1 %51, label %52, label %67, !dbg !65

52:                                               ; preds = %45
  %53 = load ptr, ptr %2, align 8, !dbg !66
  %54 = load i64, ptr %4, align 8, !dbg !67
  %55 = getelementptr inbounds i8, ptr %53, i64 %54, !dbg !66
  %56 = load i8, ptr %55, align 1, !dbg !66
  %57 = zext i8 %56 to i32, !dbg !66
  %58 = icmp sle i32 %57, 57, !dbg !68
  br i1 %58, label %59, label %67, !dbg !69

59:                                               ; preds = %52, %38, %24
  %60 = load ptr, ptr %2, align 8, !dbg !70
  %61 = load i64, ptr %4, align 8, !dbg !72
  %62 = getelementptr inbounds i8, ptr %60, i64 %61, !dbg !70
  %63 = load i8, ptr %62, align 1, !dbg !70
  %64 = load i64, ptr %3, align 8, !dbg !73
  %65 = add i64 %64, 1, !dbg !73
  store i64 %65, ptr %3, align 8, !dbg !73
  %66 = getelementptr inbounds [256 x i8], ptr @sanitize_input.safe, i64 0, i64 %64, !dbg !74
  store i8 %63, ptr %66, align 1, !dbg !75
  br label %67, !dbg !76

67:                                               ; preds = %59, %52, %45
  br label %68, !dbg !77

68:                                               ; preds = %67
  %69 = load i64, ptr %4, align 8, !dbg !78
  %70 = add i64 %69, 1, !dbg !78
  store i64 %70, ptr %4, align 8, !dbg !78
  br label %5, !dbg !79, !llvm.loop !80

71:                                               ; preds = %15
  %72 = load i64, ptr %3, align 8, !dbg !83
  %73 = getelementptr inbounds [256 x i8], ptr @sanitize_input.safe, i64 0, i64 %72, !dbg !84
  store i8 0, ptr %73, align 1, !dbg !85
  ret ptr @sanitize_input.safe, !dbg !86
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main(i32 noundef %0, ptr noundef %1) #0 !dbg !87 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !92, metadata !DIExpression()), !dbg !93
  store ptr %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !94, metadata !DIExpression()), !dbg !95
  %8 = load i32, ptr %4, align 4, !dbg !96
  %9 = icmp slt i32 %8, 2, !dbg !98
  br i1 %9, label %10, label %11, !dbg !99

10:                                               ; preds = %2
  store i32 1, ptr %3, align 4, !dbg !100
  br label %19, !dbg !100

11:                                               ; preds = %2
  call void @llvm.dbg.declare(metadata ptr %6, metadata !101, metadata !DIExpression()), !dbg !102
  %12 = load ptr, ptr %5, align 8, !dbg !103
  %13 = getelementptr inbounds ptr, ptr %12, i64 1, !dbg !103
  %14 = load ptr, ptr %13, align 8, !dbg !103
  store ptr %14, ptr %6, align 8, !dbg !102
  call void @llvm.dbg.declare(metadata ptr %7, metadata !104, metadata !DIExpression()), !dbg !105
  %15 = load ptr, ptr %6, align 8, !dbg !106
  %16 = call ptr @sanitize_input(ptr noundef %15), !dbg !107
  store ptr %16, ptr %7, align 8, !dbg !105
  %17 = load ptr, ptr %7, align 8, !dbg !108
  %18 = call i32 @system(ptr noundef %17), !dbg !109
  store i32 %18, ptr %3, align 4, !dbg !110
  br label %19, !dbg !110

19:                                               ; preds = %11, %10
  %20 = load i32, ptr %3, align 4, !dbg !111
  ret i32 %20, !dbg !111
}

declare i32 @system(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.dbg.cu = !{!10}
!llvm.module.flags = !{!17, !18, !19, !20, !21, !22, !23}
!llvm.ident = !{!24}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(name: "safe", scope: !2, file: !3, line: 11, type: !14, isLocal: true, isDefinition: true)
!2 = distinct !DISubprogram(name: "sanitize_input", scope: !3, file: !3, line: 10, type: !4, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !10, retainedNodes: !13)
!3 = !DIFile(filename: "c/taint_sanitized.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "2adca1093e2065d774f9415ca730f3e9")
!4 = !DISubroutineType(types: !5)
!5 = !{!6, !8}
!6 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !7, size: 64)
!7 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!8 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !9, size: 64)
!9 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !7)
!10 = distinct !DICompileUnit(language: DW_LANG_C11, file: !11, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !12, splitDebugInlining: false, nameTableKind: None)
!11 = !DIFile(filename: "/workspace/tests/programs/c/taint_sanitized.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "2adca1093e2065d774f9415ca730f3e9")
!12 = !{!0}
!13 = !{}
!14 = !DICompositeType(tag: DW_TAG_array_type, baseType: !7, size: 2048, elements: !15)
!15 = !{!16}
!16 = !DISubrange(count: 256)
!17 = !{i32 7, !"Dwarf Version", i32 5}
!18 = !{i32 2, !"Debug Info Version", i32 3}
!19 = !{i32 1, !"wchar_size", i32 4}
!20 = !{i32 8, !"PIC Level", i32 2}
!21 = !{i32 7, !"PIE Level", i32 2}
!22 = !{i32 7, !"uwtable", i32 2}
!23 = !{i32 7, !"frame-pointer", i32 1}
!24 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!25 = !DILocalVariable(name: "input", arg: 1, scope: !2, file: !3, line: 10, type: !8)
!26 = !DILocation(line: 10, column: 34, scope: !2)
!27 = !DILocalVariable(name: "j", scope: !2, file: !3, line: 12, type: !28)
!28 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !29, line: 18, baseType: !30)
!29 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!30 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!31 = !DILocation(line: 12, column: 12, scope: !2)
!32 = !DILocalVariable(name: "i", scope: !33, file: !3, line: 13, type: !28)
!33 = distinct !DILexicalBlock(scope: !2, file: !3, line: 13, column: 5)
!34 = !DILocation(line: 13, column: 17, scope: !33)
!35 = !DILocation(line: 13, column: 10, scope: !33)
!36 = !DILocation(line: 13, column: 24, scope: !37)
!37 = distinct !DILexicalBlock(scope: !33, file: !3, line: 13, column: 5)
!38 = !DILocation(line: 13, column: 30, scope: !37)
!39 = !DILocation(line: 13, column: 33, scope: !37)
!40 = !DILocation(line: 13, column: 36, scope: !37)
!41 = !DILocation(line: 13, column: 38, scope: !37)
!42 = !DILocation(line: 0, scope: !37)
!43 = !DILocation(line: 13, column: 5, scope: !33)
!44 = !DILocation(line: 15, column: 14, scope: !45)
!45 = distinct !DILexicalBlock(scope: !46, file: !3, line: 15, column: 13)
!46 = distinct !DILexicalBlock(scope: !37, file: !3, line: 13, column: 63)
!47 = !DILocation(line: 15, column: 20, scope: !45)
!48 = !DILocation(line: 15, column: 23, scope: !45)
!49 = !DILocation(line: 15, column: 30, scope: !45)
!50 = !DILocation(line: 15, column: 33, scope: !45)
!51 = !DILocation(line: 15, column: 39, scope: !45)
!52 = !DILocation(line: 15, column: 42, scope: !45)
!53 = !DILocation(line: 15, column: 50, scope: !45)
!54 = !DILocation(line: 16, column: 14, scope: !45)
!55 = !DILocation(line: 16, column: 20, scope: !45)
!56 = !DILocation(line: 16, column: 23, scope: !45)
!57 = !DILocation(line: 16, column: 30, scope: !45)
!58 = !DILocation(line: 16, column: 33, scope: !45)
!59 = !DILocation(line: 16, column: 39, scope: !45)
!60 = !DILocation(line: 16, column: 42, scope: !45)
!61 = !DILocation(line: 16, column: 50, scope: !45)
!62 = !DILocation(line: 17, column: 14, scope: !45)
!63 = !DILocation(line: 17, column: 20, scope: !45)
!64 = !DILocation(line: 17, column: 23, scope: !45)
!65 = !DILocation(line: 17, column: 30, scope: !45)
!66 = !DILocation(line: 17, column: 33, scope: !45)
!67 = !DILocation(line: 17, column: 39, scope: !45)
!68 = !DILocation(line: 17, column: 42, scope: !45)
!69 = !DILocation(line: 15, column: 13, scope: !46)
!70 = !DILocation(line: 18, column: 25, scope: !71)
!71 = distinct !DILexicalBlock(scope: !45, file: !3, line: 17, column: 51)
!72 = !DILocation(line: 18, column: 31, scope: !71)
!73 = !DILocation(line: 18, column: 19, scope: !71)
!74 = !DILocation(line: 18, column: 13, scope: !71)
!75 = !DILocation(line: 18, column: 23, scope: !71)
!76 = !DILocation(line: 19, column: 9, scope: !71)
!77 = !DILocation(line: 20, column: 5, scope: !46)
!78 = !DILocation(line: 13, column: 59, scope: !37)
!79 = !DILocation(line: 13, column: 5, scope: !37)
!80 = distinct !{!80, !43, !81, !82}
!81 = !DILocation(line: 20, column: 5, scope: !33)
!82 = !{!"llvm.loop.mustprogress"}
!83 = !DILocation(line: 21, column: 10, scope: !2)
!84 = !DILocation(line: 21, column: 5, scope: !2)
!85 = !DILocation(line: 21, column: 13, scope: !2)
!86 = !DILocation(line: 22, column: 5, scope: !2)
!87 = distinct !DISubprogram(name: "main", scope: !3, file: !3, line: 25, type: !88, scopeLine: 25, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !10, retainedNodes: !13)
!88 = !DISubroutineType(types: !89)
!89 = !{!90, !90, !91}
!90 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!91 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !6, size: 64)
!92 = !DILocalVariable(name: "argc", arg: 1, scope: !87, file: !3, line: 25, type: !90)
!93 = !DILocation(line: 25, column: 14, scope: !87)
!94 = !DILocalVariable(name: "argv", arg: 2, scope: !87, file: !3, line: 25, type: !91)
!95 = !DILocation(line: 25, column: 26, scope: !87)
!96 = !DILocation(line: 26, column: 9, scope: !97)
!97 = distinct !DILexicalBlock(scope: !87, file: !3, line: 26, column: 9)
!98 = !DILocation(line: 26, column: 14, scope: !97)
!99 = !DILocation(line: 26, column: 9, scope: !87)
!100 = !DILocation(line: 26, column: 19, scope: !97)
!101 = !DILocalVariable(name: "user_input", scope: !87, file: !3, line: 27, type: !6)
!102 = !DILocation(line: 27, column: 11, scope: !87)
!103 = !DILocation(line: 27, column: 24, scope: !87)
!104 = !DILocalVariable(name: "safe_cmd", scope: !87, file: !3, line: 28, type: !6)
!105 = !DILocation(line: 28, column: 11, scope: !87)
!106 = !DILocation(line: 28, column: 37, scope: !87)
!107 = !DILocation(line: 28, column: 22, scope: !87)
!108 = !DILocation(line: 29, column: 19, scope: !87)
!109 = !DILocation(line: 29, column: 12, scope: !87)
!110 = !DILocation(line: 29, column: 5, scope: !87)
!111 = !DILocation(line: 30, column: 1, scope: !87)
