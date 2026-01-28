; ModuleID = '/workspace/tests/programs/c/cspta_wrapper_dispatch.c'
source_filename = "/workspace/tests/programs/c/cspta_wrapper_dispatch.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @my_alloc(i32 noundef %0) #0 !dbg !13 {
  %2 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !20, metadata !DIExpression()), !dbg !21
  %3 = load i32, ptr %2, align 4, !dbg !22
  %4 = sext i32 %3 to i64, !dbg !22
  %5 = call noalias ptr @malloc(i64 noundef %4) #4, !dbg !23
  ret ptr %5, !dbg !24
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @use_ptr(ptr noundef %0) #0 !dbg !25 {
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !28, metadata !DIExpression()), !dbg !29
  call void @llvm.dbg.declare(metadata ptr %3, metadata !30, metadata !DIExpression()), !dbg !33
  %4 = load ptr, ptr %2, align 8, !dbg !34
  store ptr %4, ptr %3, align 8, !dbg !33
  %5 = load ptr, ptr %3, align 8, !dbg !35
  store volatile i8 120, ptr %5, align 1, !dbg !36
  ret void, !dbg !37
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !38 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !41, metadata !DIExpression()), !dbg !42
  %4 = call ptr @my_alloc(i32 noundef 16), !dbg !43
  store ptr %4, ptr %2, align 8, !dbg !42
  call void @llvm.dbg.declare(metadata ptr %3, metadata !44, metadata !DIExpression()), !dbg !45
  %5 = call ptr @my_alloc(i32 noundef 32), !dbg !46
  store ptr %5, ptr %3, align 8, !dbg !45
  %6 = load ptr, ptr %2, align 8, !dbg !47
  call void @use_ptr(ptr noundef %6), !dbg !48
  %7 = load ptr, ptr %3, align 8, !dbg !49
  call void @use_ptr(ptr noundef %7), !dbg !50
  %8 = load ptr, ptr %2, align 8, !dbg !51
  call void @free(ptr noundef %8) #5, !dbg !52
  %9 = load ptr, ptr %3, align 8, !dbg !53
  call void @free(ptr noundef %9) #5, !dbg !54
  ret i32 0, !dbg !55
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
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/cspta_wrapper_dispatch.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "8e010e7ccc6aff20d1b565f459f7ca2a")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{i32 7, !"Dwarf Version", i32 5}
!6 = !{i32 2, !"Debug Info Version", i32 3}
!7 = !{i32 1, !"wchar_size", i32 4}
!8 = !{i32 8, !"PIC Level", i32 2}
!9 = !{i32 7, !"PIE Level", i32 2}
!10 = !{i32 7, !"uwtable", i32 2}
!11 = !{i32 7, !"frame-pointer", i32 1}
!12 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!13 = distinct !DISubprogram(name: "my_alloc", scope: !14, file: !14, line: 10, type: !15, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!14 = !DIFile(filename: "c/cspta_wrapper_dispatch.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "8e010e7ccc6aff20d1b565f459f7ca2a")
!15 = !DISubroutineType(types: !16)
!16 = !{!17, !18}
!17 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!18 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!19 = !{}
!20 = !DILocalVariable(name: "size", arg: 1, scope: !13, file: !14, line: 10, type: !18)
!21 = !DILocation(line: 10, column: 20, scope: !13)
!22 = !DILocation(line: 11, column: 19, scope: !13)
!23 = !DILocation(line: 11, column: 12, scope: !13)
!24 = !DILocation(line: 11, column: 5, scope: !13)
!25 = distinct !DISubprogram(name: "use_ptr", scope: !14, file: !14, line: 14, type: !26, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!26 = !DISubroutineType(types: !27)
!27 = !{null, !17}
!28 = !DILocalVariable(name: "p", arg: 1, scope: !25, file: !14, line: 14, type: !17)
!29 = !DILocation(line: 14, column: 20, scope: !25)
!30 = !DILocalVariable(name: "cp", scope: !25, file: !14, line: 16, type: !31)
!31 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !32, size: 64)
!32 = !DIDerivedType(tag: DW_TAG_volatile_type, baseType: !4)
!33 = !DILocation(line: 16, column: 20, scope: !25)
!34 = !DILocation(line: 16, column: 33, scope: !25)
!35 = !DILocation(line: 17, column: 6, scope: !25)
!36 = !DILocation(line: 17, column: 9, scope: !25)
!37 = !DILocation(line: 18, column: 1, scope: !25)
!38 = distinct !DISubprogram(name: "main", scope: !14, file: !14, line: 20, type: !39, scopeLine: 20, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !19)
!39 = !DISubroutineType(types: !40)
!40 = !{!18}
!41 = !DILocalVariable(name: "a", scope: !38, file: !14, line: 21, type: !17)
!42 = !DILocation(line: 21, column: 11, scope: !38)
!43 = !DILocation(line: 21, column: 15, scope: !38)
!44 = !DILocalVariable(name: "b", scope: !38, file: !14, line: 22, type: !17)
!45 = !DILocation(line: 22, column: 11, scope: !38)
!46 = !DILocation(line: 22, column: 15, scope: !38)
!47 = !DILocation(line: 26, column: 13, scope: !38)
!48 = !DILocation(line: 26, column: 5, scope: !38)
!49 = !DILocation(line: 27, column: 13, scope: !38)
!50 = !DILocation(line: 27, column: 5, scope: !38)
!51 = !DILocation(line: 29, column: 10, scope: !38)
!52 = !DILocation(line: 29, column: 5, scope: !38)
!53 = !DILocation(line: 30, column: 10, scope: !38)
!54 = !DILocation(line: 30, column: 5, scope: !38)
!55 = !DILocation(line: 31, column: 5, scope: !38)
