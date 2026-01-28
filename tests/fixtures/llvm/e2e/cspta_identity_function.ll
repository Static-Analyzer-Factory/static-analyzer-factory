; ModuleID = '/workspace/tests/programs/c/cspta_identity_function.c'
source_filename = "/workspace/tests/programs/c/cspta_identity_function.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @id(ptr noundef %0) #0 !dbg !13 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !19, metadata !DIExpression()), !dbg !20
  %3 = load ptr, ptr %2, align 8, !dbg !21
  ret ptr %3, !dbg !22
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !23 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !26, metadata !DIExpression()), !dbg !27
  store i32 10, ptr %2, align 4, !dbg !27
  call void @llvm.dbg.declare(metadata ptr %3, metadata !28, metadata !DIExpression()), !dbg !29
  store i32 20, ptr %3, align 4, !dbg !29
  call void @llvm.dbg.declare(metadata ptr %4, metadata !30, metadata !DIExpression()), !dbg !31
  %6 = call ptr @id(ptr noundef %2), !dbg !32
  store ptr %6, ptr %4, align 8, !dbg !31
  call void @llvm.dbg.declare(metadata ptr %5, metadata !33, metadata !DIExpression()), !dbg !34
  %7 = call ptr @id(ptr noundef %3), !dbg !35
  store ptr %7, ptr %5, align 8, !dbg !34
  %8 = load ptr, ptr %4, align 8, !dbg !36
  store i32 100, ptr %8, align 4, !dbg !37
  %9 = load ptr, ptr %5, align 8, !dbg !38
  store i32 200, ptr %9, align 4, !dbg !39
  %10 = load i32, ptr %2, align 4, !dbg !40
  %11 = load i32, ptr %3, align 4, !dbg !41
  %12 = add nsw i32 %10, %11, !dbg !42
  ret i32 %12, !dbg !43
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!5, !6, !7, !8, !9, !10, !11}
!llvm.ident = !{!12}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/workspace/tests/programs/c/cspta_identity_function.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "10abf8637821261a97c94602912598c6")
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
!13 = distinct !DISubprogram(name: "id", scope: !14, file: !14, line: 9, type: !15, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!14 = !DIFile(filename: "c/cspta_identity_function.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "10abf8637821261a97c94602912598c6")
!15 = !DISubroutineType(types: !16)
!16 = !{!17, !17}
!17 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!18 = !{}
!19 = !DILocalVariable(name: "p", arg: 1, scope: !13, file: !14, line: 9, type: !17)
!20 = !DILocation(line: 9, column: 16, scope: !13)
!21 = !DILocation(line: 10, column: 12, scope: !13)
!22 = !DILocation(line: 10, column: 5, scope: !13)
!23 = distinct !DISubprogram(name: "main", scope: !14, file: !14, line: 13, type: !24, scopeLine: 13, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !18)
!24 = !DISubroutineType(types: !25)
!25 = !{!4}
!26 = !DILocalVariable(name: "x", scope: !23, file: !14, line: 14, type: !4)
!27 = !DILocation(line: 14, column: 9, scope: !23)
!28 = !DILocalVariable(name: "y", scope: !23, file: !14, line: 15, type: !4)
!29 = !DILocation(line: 15, column: 9, scope: !23)
!30 = !DILocalVariable(name: "px", scope: !23, file: !14, line: 17, type: !17)
!31 = !DILocation(line: 17, column: 11, scope: !23)
!32 = !DILocation(line: 17, column: 16, scope: !23)
!33 = !DILocalVariable(name: "py", scope: !23, file: !14, line: 18, type: !17)
!34 = !DILocation(line: 18, column: 11, scope: !23)
!35 = !DILocation(line: 18, column: 16, scope: !23)
!36 = !DILocation(line: 22, column: 13, scope: !23)
!37 = !DILocation(line: 22, column: 16, scope: !23)
!38 = !DILocation(line: 23, column: 13, scope: !23)
!39 = !DILocation(line: 23, column: 16, scope: !23)
!40 = !DILocation(line: 25, column: 12, scope: !23)
!41 = !DILocation(line: 25, column: 16, scope: !23)
!42 = !DILocation(line: 25, column: 14, scope: !23)
!43 = !DILocation(line: 25, column: 5, scope: !23)
