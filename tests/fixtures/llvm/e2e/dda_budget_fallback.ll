; ModuleID = 'dda_budget_fallback.c'
source_filename = "dda_budget_fallback.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap1(ptr noundef %0) #0 !dbg !10 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !16, metadata !DIExpression()), !dbg !17
  %3 = load ptr, ptr %2, align 8, !dbg !18
  ret ptr %3, !dbg !19
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap2(ptr noundef %0) #0 !dbg !20 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !21, metadata !DIExpression()), !dbg !22
  %3 = load ptr, ptr %2, align 8, !dbg !23
  %4 = call ptr @wrap1(ptr noundef %3), !dbg !24
  ret ptr %4, !dbg !25
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap3(ptr noundef %0) #0 !dbg !26 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !27, metadata !DIExpression()), !dbg !28
  %3 = load ptr, ptr %2, align 8, !dbg !29
  %4 = call ptr @wrap2(ptr noundef %3), !dbg !30
  ret ptr %4, !dbg !31
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap4(ptr noundef %0) #0 !dbg !32 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !33, metadata !DIExpression()), !dbg !34
  %3 = load ptr, ptr %2, align 8, !dbg !35
  %4 = call ptr @wrap3(ptr noundef %3), !dbg !36
  ret ptr %4, !dbg !37
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap5(ptr noundef %0) #0 !dbg !38 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !39, metadata !DIExpression()), !dbg !40
  %3 = load ptr, ptr %2, align 8, !dbg !41
  %4 = call ptr @wrap4(ptr noundef %3), !dbg !42
  ret ptr %4, !dbg !43
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap6(ptr noundef %0) #0 !dbg !44 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !45, metadata !DIExpression()), !dbg !46
  %3 = load ptr, ptr %2, align 8, !dbg !47
  %4 = call ptr @wrap5(ptr noundef %3), !dbg !48
  ret ptr %4, !dbg !49
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap7(ptr noundef %0) #0 !dbg !50 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !51, metadata !DIExpression()), !dbg !52
  %3 = load ptr, ptr %2, align 8, !dbg !53
  %4 = call ptr @wrap6(ptr noundef %3), !dbg !54
  ret ptr %4, !dbg !55
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap8(ptr noundef %0) #0 !dbg !56 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !57, metadata !DIExpression()), !dbg !58
  %3 = load ptr, ptr %2, align 8, !dbg !59
  %4 = call ptr @wrap7(ptr noundef %3), !dbg !60
  ret ptr %4, !dbg !61
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap9(ptr noundef %0) #0 !dbg !62 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !63, metadata !DIExpression()), !dbg !64
  %3 = load ptr, ptr %2, align 8, !dbg !65
  %4 = call ptr @wrap8(ptr noundef %3), !dbg !66
  ret ptr %4, !dbg !67
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrap10(ptr noundef %0) #0 !dbg !68 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !69, metadata !DIExpression()), !dbg !70
  %3 = load ptr, ptr %2, align 8, !dbg !71
  %4 = call ptr @wrap9(ptr noundef %3), !dbg !72
  ret ptr %4, !dbg !73
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !74 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !77, metadata !DIExpression()), !dbg !78
  store i32 42, ptr %2, align 4, !dbg !78
  call void @llvm.dbg.declare(metadata ptr %3, metadata !79, metadata !DIExpression()), !dbg !80
  store ptr %2, ptr %3, align 8, !dbg !80
  call void @llvm.dbg.declare(metadata ptr %4, metadata !81, metadata !DIExpression()), !dbg !82
  %5 = load ptr, ptr %3, align 8, !dbg !83
  %6 = call ptr @wrap10(ptr noundef %5), !dbg !84
  store ptr %6, ptr %4, align 8, !dbg !82
  %7 = load ptr, ptr %4, align 8, !dbg !85
  %8 = load i32, ptr %7, align 4, !dbg !86
  ret i32 %8, !dbg !87
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7, !8}
!llvm.ident = !{!9}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "dda_budget_fallback.c", directory: "/workspace/tests/fixtures/sources", checksumkind: CSK_MD5, checksum: "73b44bb6e66d873ad47ea0f2f8d163e8")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 1, !"wchar_size", i32 4}
!5 = !{i32 8, !"PIC Level", i32 2}
!6 = !{i32 7, !"PIE Level", i32 2}
!7 = !{i32 7, !"uwtable", i32 2}
!8 = !{i32 7, !"frame-pointer", i32 1}
!9 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!10 = distinct !DISubprogram(name: "wrap1", scope: !1, file: !1, line: 8, type: !11, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!11 = !DISubroutineType(types: !12)
!12 = !{!13, !13}
!13 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!14 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!15 = !{}
!16 = !DILocalVariable(name: "p", arg: 1, scope: !10, file: !1, line: 8, type: !13)
!17 = !DILocation(line: 8, column: 17, scope: !10)
!18 = !DILocation(line: 8, column: 29, scope: !10)
!19 = !DILocation(line: 8, column: 22, scope: !10)
!20 = distinct !DISubprogram(name: "wrap2", scope: !1, file: !1, line: 9, type: !11, scopeLine: 9, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!21 = !DILocalVariable(name: "p", arg: 1, scope: !20, file: !1, line: 9, type: !13)
!22 = !DILocation(line: 9, column: 17, scope: !20)
!23 = !DILocation(line: 9, column: 35, scope: !20)
!24 = !DILocation(line: 9, column: 29, scope: !20)
!25 = !DILocation(line: 9, column: 22, scope: !20)
!26 = distinct !DISubprogram(name: "wrap3", scope: !1, file: !1, line: 10, type: !11, scopeLine: 10, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!27 = !DILocalVariable(name: "p", arg: 1, scope: !26, file: !1, line: 10, type: !13)
!28 = !DILocation(line: 10, column: 17, scope: !26)
!29 = !DILocation(line: 10, column: 35, scope: !26)
!30 = !DILocation(line: 10, column: 29, scope: !26)
!31 = !DILocation(line: 10, column: 22, scope: !26)
!32 = distinct !DISubprogram(name: "wrap4", scope: !1, file: !1, line: 11, type: !11, scopeLine: 11, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!33 = !DILocalVariable(name: "p", arg: 1, scope: !32, file: !1, line: 11, type: !13)
!34 = !DILocation(line: 11, column: 17, scope: !32)
!35 = !DILocation(line: 11, column: 35, scope: !32)
!36 = !DILocation(line: 11, column: 29, scope: !32)
!37 = !DILocation(line: 11, column: 22, scope: !32)
!38 = distinct !DISubprogram(name: "wrap5", scope: !1, file: !1, line: 12, type: !11, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!39 = !DILocalVariable(name: "p", arg: 1, scope: !38, file: !1, line: 12, type: !13)
!40 = !DILocation(line: 12, column: 17, scope: !38)
!41 = !DILocation(line: 12, column: 35, scope: !38)
!42 = !DILocation(line: 12, column: 29, scope: !38)
!43 = !DILocation(line: 12, column: 22, scope: !38)
!44 = distinct !DISubprogram(name: "wrap6", scope: !1, file: !1, line: 13, type: !11, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!45 = !DILocalVariable(name: "p", arg: 1, scope: !44, file: !1, line: 13, type: !13)
!46 = !DILocation(line: 13, column: 17, scope: !44)
!47 = !DILocation(line: 13, column: 35, scope: !44)
!48 = !DILocation(line: 13, column: 29, scope: !44)
!49 = !DILocation(line: 13, column: 22, scope: !44)
!50 = distinct !DISubprogram(name: "wrap7", scope: !1, file: !1, line: 14, type: !11, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!51 = !DILocalVariable(name: "p", arg: 1, scope: !50, file: !1, line: 14, type: !13)
!52 = !DILocation(line: 14, column: 17, scope: !50)
!53 = !DILocation(line: 14, column: 35, scope: !50)
!54 = !DILocation(line: 14, column: 29, scope: !50)
!55 = !DILocation(line: 14, column: 22, scope: !50)
!56 = distinct !DISubprogram(name: "wrap8", scope: !1, file: !1, line: 15, type: !11, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!57 = !DILocalVariable(name: "p", arg: 1, scope: !56, file: !1, line: 15, type: !13)
!58 = !DILocation(line: 15, column: 17, scope: !56)
!59 = !DILocation(line: 15, column: 35, scope: !56)
!60 = !DILocation(line: 15, column: 29, scope: !56)
!61 = !DILocation(line: 15, column: 22, scope: !56)
!62 = distinct !DISubprogram(name: "wrap9", scope: !1, file: !1, line: 16, type: !11, scopeLine: 16, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!63 = !DILocalVariable(name: "p", arg: 1, scope: !62, file: !1, line: 16, type: !13)
!64 = !DILocation(line: 16, column: 17, scope: !62)
!65 = !DILocation(line: 16, column: 35, scope: !62)
!66 = !DILocation(line: 16, column: 29, scope: !62)
!67 = !DILocation(line: 16, column: 22, scope: !62)
!68 = distinct !DISubprogram(name: "wrap10", scope: !1, file: !1, line: 17, type: !11, scopeLine: 17, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!69 = !DILocalVariable(name: "p", arg: 1, scope: !68, file: !1, line: 17, type: !13)
!70 = !DILocation(line: 17, column: 18, scope: !68)
!71 = !DILocation(line: 17, column: 36, scope: !68)
!72 = !DILocation(line: 17, column: 30, scope: !68)
!73 = !DILocation(line: 17, column: 23, scope: !68)
!74 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 19, type: !75, scopeLine: 19, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !15)
!75 = !DISubroutineType(types: !76)
!76 = !{!14}
!77 = !DILocalVariable(name: "x", scope: !74, file: !1, line: 20, type: !14)
!78 = !DILocation(line: 20, column: 9, scope: !74)
!79 = !DILocalVariable(name: "p", scope: !74, file: !1, line: 21, type: !13)
!80 = !DILocation(line: 21, column: 10, scope: !74)
!81 = !DILocalVariable(name: "q", scope: !74, file: !1, line: 24, type: !13)
!82 = !DILocation(line: 24, column: 10, scope: !74)
!83 = !DILocation(line: 24, column: 21, scope: !74)
!84 = !DILocation(line: 24, column: 14, scope: !74)
!85 = !DILocation(line: 26, column: 13, scope: !74)
!86 = !DILocation(line: 26, column: 12, scope: !74)
!87 = !DILocation(line: 26, column: 5, scope: !74)
