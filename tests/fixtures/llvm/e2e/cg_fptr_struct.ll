; ModuleID = '/workspace/tests/programs/c/cg_fptr_struct.c'
source_filename = "/workspace/tests/programs/c/cg_fptr_struct.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Plugin = type { ptr, ptr }

@.str = private unnamed_addr constant [7 x i8] c"danger\00", align 1, !dbg !0
@.str.1 = private unnamed_addr constant [6 x i8] c"INPUT\00", align 1, !dbg !7

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @dangerous_handler(ptr noundef %0) #0 !dbg !23 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !29, metadata !DIExpression()), !dbg !30
  %3 = load ptr, ptr %2, align 8, !dbg !31
  %4 = call i32 @system(ptr noundef %3), !dbg !32
  ret void, !dbg !33
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

declare i32 @system(ptr noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @invoke_plugin(ptr noundef %0, ptr noundef %1) #0 !dbg !34 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !44, metadata !DIExpression()), !dbg !45
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !46, metadata !DIExpression()), !dbg !47
  %5 = load ptr, ptr %3, align 8, !dbg !48
  %6 = getelementptr inbounds %struct.Plugin, ptr %5, i32 0, i32 0, !dbg !49
  %7 = load ptr, ptr %6, align 8, !dbg !49
  %8 = load ptr, ptr %4, align 8, !dbg !50
  call void %7(ptr noundef %8), !dbg !48
  ret void, !dbg !51
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !52 {
  %1 = alloca i32, align 4
  %2 = alloca %struct.Plugin, align 8
  %3 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !56, metadata !DIExpression()), !dbg !57
  %4 = getelementptr inbounds %struct.Plugin, ptr %2, i32 0, i32 0, !dbg !58
  store ptr @dangerous_handler, ptr %4, align 8, !dbg !59
  %5 = getelementptr inbounds %struct.Plugin, ptr %2, i32 0, i32 1, !dbg !60
  store ptr @.str, ptr %5, align 8, !dbg !61
  call void @llvm.dbg.declare(metadata ptr %3, metadata !62, metadata !DIExpression()), !dbg !63
  %6 = call ptr @getenv(ptr noundef @.str.1) #4, !dbg !64
  store ptr %6, ptr %3, align 8, !dbg !63
  %7 = load ptr, ptr %3, align 8, !dbg !65
  call void @invoke_plugin(ptr noundef %2, ptr noundef %7), !dbg !66
  ret i32 0, !dbg !67
}

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #3

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.dbg.cu = !{!12}
!llvm.module.flags = !{!15, !16, !17, !18, !19, !20, !21}
!llvm.ident = !{!22}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 21, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "c/cg_fptr_struct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "083f3bca3753a718a69bb75e99c0c9c7")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 56, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!5 = !{!6}
!6 = !DISubrange(count: 7)
!7 = !DIGlobalVariableExpression(var: !8, expr: !DIExpression())
!8 = distinct !DIGlobalVariable(scope: null, file: !2, line: 22, type: !9, isLocal: true, isDefinition: true)
!9 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 48, elements: !10)
!10 = !{!11}
!11 = !DISubrange(count: 6)
!12 = distinct !DICompileUnit(language: DW_LANG_C11, file: !13, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !14, splitDebugInlining: false, nameTableKind: None)
!13 = !DIFile(filename: "/workspace/tests/programs/c/cg_fptr_struct.c", directory: "/workspace/tests/programs", checksumkind: CSK_MD5, checksum: "083f3bca3753a718a69bb75e99c0c9c7")
!14 = !{!0, !7}
!15 = !{i32 7, !"Dwarf Version", i32 5}
!16 = !{i32 2, !"Debug Info Version", i32 3}
!17 = !{i32 1, !"wchar_size", i32 4}
!18 = !{i32 8, !"PIC Level", i32 2}
!19 = !{i32 7, !"PIE Level", i32 2}
!20 = !{i32 7, !"uwtable", i32 2}
!21 = !{i32 7, !"frame-pointer", i32 1}
!22 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!23 = distinct !DISubprogram(name: "dangerous_handler", scope: !2, file: !2, line: 12, type: !24, scopeLine: 12, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !28)
!24 = !DISubroutineType(types: !25)
!25 = !{null, !26}
!26 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !27, size: 64)
!27 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !4)
!28 = !{}
!29 = !DILocalVariable(name: "s", arg: 1, scope: !23, file: !2, line: 12, type: !26)
!30 = !DILocation(line: 12, column: 36, scope: !23)
!31 = !DILocation(line: 12, column: 48, scope: !23)
!32 = !DILocation(line: 12, column: 41, scope: !23)
!33 = !DILocation(line: 12, column: 52, scope: !23)
!34 = distinct !DISubprogram(name: "invoke_plugin", scope: !2, file: !2, line: 14, type: !35, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !28)
!35 = !DISubroutineType(types: !36)
!36 = !{null, !37, !26}
!37 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !38, size: 64)
!38 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Plugin", file: !2, line: 7, size: 128, elements: !39)
!39 = !{!40, !43}
!40 = !DIDerivedType(tag: DW_TAG_member, name: "handle", scope: !38, file: !2, line: 8, baseType: !41, size: 64)
!41 = !DIDerivedType(tag: DW_TAG_typedef, name: "handler_fn", file: !2, line: 5, baseType: !42)
!42 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !24, size: 64)
!43 = !DIDerivedType(tag: DW_TAG_member, name: "name", scope: !38, file: !2, line: 9, baseType: !26, size: 64, offset: 64)
!44 = !DILocalVariable(name: "p", arg: 1, scope: !34, file: !2, line: 14, type: !37)
!45 = !DILocation(line: 14, column: 35, scope: !34)
!46 = !DILocalVariable(name: "data", arg: 2, scope: !34, file: !2, line: 14, type: !26)
!47 = !DILocation(line: 14, column: 50, scope: !34)
!48 = !DILocation(line: 15, column: 5, scope: !34)
!49 = !DILocation(line: 15, column: 8, scope: !34)
!50 = !DILocation(line: 15, column: 15, scope: !34)
!51 = !DILocation(line: 16, column: 1, scope: !34)
!52 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 18, type: !53, scopeLine: 18, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !12, retainedNodes: !28)
!53 = !DISubroutineType(types: !54)
!54 = !{!55}
!55 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!56 = !DILocalVariable(name: "p", scope: !52, file: !2, line: 19, type: !38)
!57 = !DILocation(line: 19, column: 19, scope: !52)
!58 = !DILocation(line: 20, column: 7, scope: !52)
!59 = !DILocation(line: 20, column: 14, scope: !52)
!60 = !DILocation(line: 21, column: 7, scope: !52)
!61 = !DILocation(line: 21, column: 12, scope: !52)
!62 = !DILocalVariable(name: "input", scope: !52, file: !2, line: 22, type: !26)
!63 = !DILocation(line: 22, column: 17, scope: !52)
!64 = !DILocation(line: 22, column: 25, scope: !52)
!65 = !DILocation(line: 23, column: 23, scope: !52)
!66 = !DILocation(line: 23, column: 5, scope: !52)
!67 = !DILocation(line: 24, column: 5, scope: !52)
