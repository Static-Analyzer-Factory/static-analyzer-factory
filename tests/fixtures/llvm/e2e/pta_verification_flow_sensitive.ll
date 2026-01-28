; ModuleID = 'tests/fixtures/pta_verification/flow_sensitive.c'
source_filename = "tests/fixtures/pta_verification/flow_sensitive.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.SimpleStruct = type { i32, i32 }
%struct.Outer = type { %struct.Inner, i32 }
%struct.Inner = type { i32 }

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_basic_strong_update() #0 !dbg !13 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !17, metadata !DIExpression()), !dbg !18
  store i32 1, ptr %1, align 4, !dbg !18
  call void @llvm.dbg.declare(metadata ptr %2, metadata !19, metadata !DIExpression()), !dbg !20
  store i32 2, ptr %2, align 4, !dbg !20
  call void @llvm.dbg.declare(metadata ptr %3, metadata !21, metadata !DIExpression()), !dbg !22
  store ptr %1, ptr %3, align 8, !dbg !22
  %6 = load ptr, ptr %3, align 8, !dbg !23
  store i32 10, ptr %6, align 4, !dbg !24
  call void @llvm.dbg.declare(metadata ptr %4, metadata !25, metadata !DIExpression()), !dbg !26
  store ptr %2, ptr %4, align 8, !dbg !26
  %7 = load ptr, ptr %4, align 8, !dbg !27
  store i32 20, ptr %7, align 4, !dbg !28
  call void @llvm.dbg.declare(metadata ptr %5, metadata !29, metadata !DIExpression()), !dbg !30
  %8 = load ptr, ptr %3, align 8, !dbg !31
  %9 = load i32, ptr %8, align 4, !dbg !32
  %10 = load ptr, ptr %4, align 8, !dbg !33
  %11 = load i32, ptr %10, align 4, !dbg !34
  %12 = add nsw i32 %9, %11, !dbg !35
  store i32 %12, ptr %5, align 4, !dbg !30
  %13 = load i32, ptr %5, align 4, !dbg !36
  ret void, !dbg !37
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_local_pointer_strong_update() #0 !dbg !38 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !39, metadata !DIExpression()), !dbg !40
  call void @llvm.dbg.declare(metadata ptr %2, metadata !41, metadata !DIExpression()), !dbg !42
  store ptr %1, ptr %2, align 8, !dbg !42
  %4 = load ptr, ptr %2, align 8, !dbg !43
  store i32 100, ptr %4, align 4, !dbg !44
  %5 = load ptr, ptr %2, align 8, !dbg !45
  store i32 200, ptr %5, align 4, !dbg !46
  call void @llvm.dbg.declare(metadata ptr %3, metadata !47, metadata !DIExpression()), !dbg !48
  %6 = load ptr, ptr %2, align 8, !dbg !49
  %7 = load i32, ptr %6, align 4, !dbg !50
  store i32 %7, ptr %3, align 4, !dbg !48
  %8 = load i32, ptr %3, align 4, !dbg !51
  ret void, !dbg !52
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_struct_field_strong_update() #0 !dbg !53 {
  %1 = alloca %struct.SimpleStruct, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !54, metadata !DIExpression()), !dbg !60
  call void @llvm.dbg.declare(metadata ptr %2, metadata !61, metadata !DIExpression()), !dbg !62
  %4 = getelementptr inbounds %struct.SimpleStruct, ptr %1, i32 0, i32 0, !dbg !63
  store ptr %4, ptr %2, align 8, !dbg !62
  %5 = load ptr, ptr %2, align 8, !dbg !64
  store i32 1, ptr %5, align 4, !dbg !65
  %6 = load ptr, ptr %2, align 8, !dbg !66
  store i32 2, ptr %6, align 4, !dbg !67
  call void @llvm.dbg.declare(metadata ptr %3, metadata !68, metadata !DIExpression()), !dbg !69
  %7 = load ptr, ptr %2, align 8, !dbg !70
  %8 = load i32, ptr %7, align 4, !dbg !71
  store i32 %8, ptr %3, align 4, !dbg !69
  %9 = load i32, ptr %3, align 4, !dbg !72
  ret void, !dbg !73
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_weak_update_non_singleton(i32 noundef %0) #0 !dbg !74 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !77, metadata !DIExpression()), !dbg !78
  call void @llvm.dbg.declare(metadata ptr %3, metadata !79, metadata !DIExpression()), !dbg !80
  call void @llvm.dbg.declare(metadata ptr %4, metadata !81, metadata !DIExpression()), !dbg !82
  call void @llvm.dbg.declare(metadata ptr %5, metadata !83, metadata !DIExpression()), !dbg !84
  %8 = load i32, ptr %2, align 4, !dbg !85
  %9 = icmp ne i32 %8, 0, !dbg !85
  br i1 %9, label %10, label %11, !dbg !87

10:                                               ; preds = %1
  store ptr %3, ptr %5, align 8, !dbg !88
  br label %12, !dbg !90

11:                                               ; preds = %1
  store ptr %4, ptr %5, align 8, !dbg !91
  br label %12

12:                                               ; preds = %11, %10
  %13 = load ptr, ptr %5, align 8, !dbg !93
  store i32 42, ptr %13, align 4, !dbg !94
  call void @llvm.dbg.declare(metadata ptr %6, metadata !95, metadata !DIExpression()), !dbg !96
  %14 = load i32, ptr %3, align 4, !dbg !97
  store i32 %14, ptr %6, align 4, !dbg !96
  call void @llvm.dbg.declare(metadata ptr %7, metadata !98, metadata !DIExpression()), !dbg !99
  %15 = load i32, ptr %4, align 4, !dbg !100
  store i32 %15, ptr %7, align 4, !dbg !99
  %16 = load i32, ptr %6, align 4, !dbg !101
  %17 = load i32, ptr %7, align 4, !dbg !102
  ret void, !dbg !103
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_weak_update_array(i32 noundef %0) #0 !dbg !104 {
  %2 = alloca i32, align 4
  %3 = alloca [10 x i32], align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !105, metadata !DIExpression()), !dbg !106
  call void @llvm.dbg.declare(metadata ptr %3, metadata !107, metadata !DIExpression()), !dbg !111
  call void @llvm.dbg.declare(metadata ptr %4, metadata !112, metadata !DIExpression()), !dbg !113
  %6 = load i32, ptr %2, align 4, !dbg !114
  %7 = sext i32 %6 to i64, !dbg !115
  %8 = getelementptr inbounds [10 x i32], ptr %3, i64 0, i64 %7, !dbg !115
  store ptr %8, ptr %4, align 8, !dbg !113
  %9 = load ptr, ptr %4, align 8, !dbg !116
  store i32 100, ptr %9, align 4, !dbg !117
  call void @llvm.dbg.declare(metadata ptr %5, metadata !118, metadata !DIExpression()), !dbg !119
  %10 = getelementptr inbounds [10 x i32], ptr %3, i64 0, i64 0, !dbg !120
  %11 = load i32, ptr %10, align 4, !dbg !120
  store i32 %11, ptr %5, align 4, !dbg !119
  %12 = load i32, ptr %5, align 4, !dbg !121
  ret void, !dbg !122
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_weak_update_heap_array(i32 noundef %0) #0 !dbg !123 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !124, metadata !DIExpression()), !dbg !125
  call void @llvm.dbg.declare(metadata ptr %3, metadata !126, metadata !DIExpression()), !dbg !127
  %6 = call noalias ptr @malloc(i64 noundef 40) #4, !dbg !128
  store ptr %6, ptr %3, align 8, !dbg !127
  call void @llvm.dbg.declare(metadata ptr %4, metadata !129, metadata !DIExpression()), !dbg !130
  %7 = load ptr, ptr %3, align 8, !dbg !131
  %8 = load i32, ptr %2, align 4, !dbg !132
  %9 = sext i32 %8 to i64, !dbg !133
  %10 = getelementptr inbounds i32, ptr %7, i64 %9, !dbg !133
  store ptr %10, ptr %4, align 8, !dbg !130
  %11 = load ptr, ptr %4, align 8, !dbg !134
  store i32 50, ptr %11, align 4, !dbg !135
  call void @llvm.dbg.declare(metadata ptr %5, metadata !136, metadata !DIExpression()), !dbg !137
  %12 = load ptr, ptr %3, align 8, !dbg !138
  %13 = getelementptr inbounds i32, ptr %12, i64 0, !dbg !138
  %14 = load i32, ptr %13, align 4, !dbg !138
  store i32 %14, ptr %5, align 4, !dbg !137
  %15 = load i32, ptr %5, align 4, !dbg !139
  %16 = load ptr, ptr %3, align 8, !dbg !140
  call void @free(ptr noundef %16) #5, !dbg !141
  ret void, !dbg !142
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: nounwind
declare void @free(ptr noundef) #3

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_kill_gen_sequence() #0 !dbg !143 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !144, metadata !DIExpression()), !dbg !145
  call void @llvm.dbg.declare(metadata ptr %2, metadata !146, metadata !DIExpression()), !dbg !147
  store ptr %1, ptr %2, align 8, !dbg !147
  %4 = load ptr, ptr %2, align 8, !dbg !148
  store i32 1, ptr %4, align 4, !dbg !149
  %5 = load ptr, ptr %2, align 8, !dbg !150
  store i32 2, ptr %5, align 4, !dbg !151
  %6 = load ptr, ptr %2, align 8, !dbg !152
  store i32 3, ptr %6, align 4, !dbg !153
  call void @llvm.dbg.declare(metadata ptr %3, metadata !154, metadata !DIExpression()), !dbg !155
  %7 = load ptr, ptr %2, align 8, !dbg !156
  %8 = load i32, ptr %7, align 4, !dbg !157
  store i32 %8, ptr %3, align 4, !dbg !155
  %9 = load i32, ptr %3, align 4, !dbg !158
  ret void, !dbg !159
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_flow_with_branches(i32 noundef %0) #0 !dbg !160 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !161, metadata !DIExpression()), !dbg !162
  call void @llvm.dbg.declare(metadata ptr %3, metadata !163, metadata !DIExpression()), !dbg !164
  store i32 0, ptr %3, align 4, !dbg !164
  call void @llvm.dbg.declare(metadata ptr %4, metadata !165, metadata !DIExpression()), !dbg !166
  store ptr %3, ptr %4, align 8, !dbg !166
  %6 = load i32, ptr %2, align 4, !dbg !167
  %7 = icmp ne i32 %6, 0, !dbg !167
  br i1 %7, label %8, label %10, !dbg !169

8:                                                ; preds = %1
  %9 = load ptr, ptr %4, align 8, !dbg !170
  store i32 10, ptr %9, align 4, !dbg !172
  br label %12, !dbg !173

10:                                               ; preds = %1
  %11 = load ptr, ptr %4, align 8, !dbg !174
  store i32 20, ptr %11, align 4, !dbg !176
  br label %12

12:                                               ; preds = %10, %8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !177, metadata !DIExpression()), !dbg !178
  %13 = load ptr, ptr %4, align 8, !dbg !179
  %14 = load i32, ptr %13, align 4, !dbg !180
  store i32 %14, ptr %5, align 4, !dbg !178
  %15 = load i32, ptr %5, align 4, !dbg !181
  ret void, !dbg !182
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_partial_kill(i32 noundef %0) #0 !dbg !183 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !184, metadata !DIExpression()), !dbg !185
  call void @llvm.dbg.declare(metadata ptr %3, metadata !186, metadata !DIExpression()), !dbg !187
  store i32 0, ptr %3, align 4, !dbg !187
  call void @llvm.dbg.declare(metadata ptr %4, metadata !188, metadata !DIExpression()), !dbg !189
  store ptr %3, ptr %4, align 8, !dbg !189
  %6 = load i32, ptr %2, align 4, !dbg !190
  %7 = icmp ne i32 %6, 0, !dbg !190
  br i1 %7, label %8, label %10, !dbg !192

8:                                                ; preds = %1
  %9 = load ptr, ptr %4, align 8, !dbg !193
  store i32 100, ptr %9, align 4, !dbg !195
  br label %10, !dbg !196

10:                                               ; preds = %8, %1
  call void @llvm.dbg.declare(metadata ptr %5, metadata !197, metadata !DIExpression()), !dbg !198
  %11 = load ptr, ptr %4, align 8, !dbg !199
  %12 = load i32, ptr %11, align 4, !dbg !200
  store i32 %12, ptr %5, align 4, !dbg !198
  %13 = load i32, ptr %5, align 4, !dbg !201
  ret void, !dbg !202
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_mixed_updates(i32 noundef %0) #0 !dbg !203 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca [5 x i32], align 4
  %6 = alloca ptr, align 8
  %7 = alloca ptr, align 8
  %8 = alloca ptr, align 8
  %9 = alloca i32, align 4
  %10 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !204, metadata !DIExpression()), !dbg !205
  call void @llvm.dbg.declare(metadata ptr %3, metadata !206, metadata !DIExpression()), !dbg !207
  call void @llvm.dbg.declare(metadata ptr %4, metadata !208, metadata !DIExpression()), !dbg !209
  call void @llvm.dbg.declare(metadata ptr %5, metadata !210, metadata !DIExpression()), !dbg !214
  call void @llvm.dbg.declare(metadata ptr %6, metadata !215, metadata !DIExpression()), !dbg !216
  store ptr %3, ptr %6, align 8, !dbg !216
  call void @llvm.dbg.declare(metadata ptr %7, metadata !217, metadata !DIExpression()), !dbg !218
  %11 = load i32, ptr %2, align 4, !dbg !219
  %12 = icmp ne i32 %11, 0, !dbg !219
  br i1 %12, label %13, label %14, !dbg !221

13:                                               ; preds = %1
  store ptr %3, ptr %7, align 8, !dbg !222
  br label %15, !dbg !224

14:                                               ; preds = %1
  store ptr %4, ptr %7, align 8, !dbg !225
  br label %15

15:                                               ; preds = %14, %13
  %16 = load ptr, ptr %6, align 8, !dbg !227
  store i32 1, ptr %16, align 4, !dbg !228
  %17 = load ptr, ptr %7, align 8, !dbg !229
  store i32 2, ptr %17, align 4, !dbg !230
  call void @llvm.dbg.declare(metadata ptr %8, metadata !231, metadata !DIExpression()), !dbg !232
  %18 = getelementptr inbounds [5 x i32], ptr %5, i64 0, i64 0, !dbg !233
  store ptr %18, ptr %8, align 8, !dbg !232
  %19 = load ptr, ptr %8, align 8, !dbg !234
  store i32 3, ptr %19, align 4, !dbg !235
  call void @llvm.dbg.declare(metadata ptr %9, metadata !236, metadata !DIExpression()), !dbg !237
  %20 = load i32, ptr %3, align 4, !dbg !238
  store i32 %20, ptr %9, align 4, !dbg !237
  call void @llvm.dbg.declare(metadata ptr %10, metadata !239, metadata !DIExpression()), !dbg !240
  %21 = load i32, ptr %4, align 4, !dbg !241
  store i32 %21, ptr %10, align 4, !dbg !240
  %22 = load i32, ptr %9, align 4, !dbg !242
  %23 = load i32, ptr %10, align 4, !dbg !243
  ret void, !dbg !244
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @modify_through_ptr(ptr noundef %0) #0 !dbg !245 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !248, metadata !DIExpression()), !dbg !249
  %3 = load ptr, ptr %2, align 8, !dbg !250
  store i32 999, ptr %3, align 4, !dbg !251
  ret void, !dbg !252
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_interprocedural_update() #0 !dbg !253 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !254, metadata !DIExpression()), !dbg !255
  store i32 0, ptr %1, align 4, !dbg !255
  call void @llvm.dbg.declare(metadata ptr %2, metadata !256, metadata !DIExpression()), !dbg !257
  store ptr %1, ptr %2, align 8, !dbg !257
  %4 = load ptr, ptr %2, align 8, !dbg !258
  store i32 1, ptr %4, align 4, !dbg !259
  %5 = load ptr, ptr %2, align 8, !dbg !260
  call void @modify_through_ptr(ptr noundef %5), !dbg !261
  call void @llvm.dbg.declare(metadata ptr %3, metadata !262, metadata !DIExpression()), !dbg !263
  %6 = load ptr, ptr %2, align 8, !dbg !264
  %7 = load i32, ptr %6, align 4, !dbg !265
  store i32 %7, ptr %3, align 4, !dbg !263
  %8 = load i32, ptr %3, align 4, !dbg !266
  ret void, !dbg !267
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_loop_update(i32 noundef %0) #0 !dbg !268 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !269, metadata !DIExpression()), !dbg !270
  call void @llvm.dbg.declare(metadata ptr %3, metadata !271, metadata !DIExpression()), !dbg !272
  store i32 0, ptr %3, align 4, !dbg !272
  call void @llvm.dbg.declare(metadata ptr %4, metadata !273, metadata !DIExpression()), !dbg !274
  store ptr %3, ptr %4, align 8, !dbg !274
  call void @llvm.dbg.declare(metadata ptr %5, metadata !275, metadata !DIExpression()), !dbg !277
  store i32 0, ptr %5, align 4, !dbg !277
  br label %7, !dbg !278

7:                                                ; preds = %14, %1
  %8 = load i32, ptr %5, align 4, !dbg !279
  %9 = load i32, ptr %2, align 4, !dbg !281
  %10 = icmp slt i32 %8, %9, !dbg !282
  br i1 %10, label %11, label %17, !dbg !283

11:                                               ; preds = %7
  %12 = load i32, ptr %5, align 4, !dbg !284
  %13 = load ptr, ptr %4, align 8, !dbg !286
  store i32 %12, ptr %13, align 4, !dbg !287
  br label %14, !dbg !288

14:                                               ; preds = %11
  %15 = load i32, ptr %5, align 4, !dbg !289
  %16 = add nsw i32 %15, 1, !dbg !289
  store i32 %16, ptr %5, align 4, !dbg !289
  br label %7, !dbg !290, !llvm.loop !291

17:                                               ; preds = %7
  call void @llvm.dbg.declare(metadata ptr %6, metadata !294, metadata !DIExpression()), !dbg !295
  %18 = load ptr, ptr %4, align 8, !dbg !296
  %19 = load i32, ptr %18, align 4, !dbg !297
  store i32 %19, ptr %6, align 4, !dbg !295
  %20 = load i32, ptr %6, align 4, !dbg !298
  ret void, !dbg !299
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_nested_struct_update() #0 !dbg !300 {
  %1 = alloca %struct.Outer, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !301, metadata !DIExpression()), !dbg !311
  call void @llvm.dbg.declare(metadata ptr %2, metadata !312, metadata !DIExpression()), !dbg !313
  %4 = getelementptr inbounds %struct.Outer, ptr %1, i32 0, i32 0, !dbg !314
  %5 = getelementptr inbounds %struct.Inner, ptr %4, i32 0, i32 0, !dbg !315
  store ptr %5, ptr %2, align 8, !dbg !313
  %6 = load ptr, ptr %2, align 8, !dbg !316
  store i32 42, ptr %6, align 4, !dbg !317
  call void @llvm.dbg.declare(metadata ptr %3, metadata !318, metadata !DIExpression()), !dbg !319
  %7 = getelementptr inbounds %struct.Outer, ptr %1, i32 0, i32 0, !dbg !320
  %8 = getelementptr inbounds %struct.Inner, ptr %7, i32 0, i32 0, !dbg !321
  %9 = load i32, ptr %8, align 4, !dbg !321
  store i32 %9, ptr %3, align 4, !dbg !319
  %10 = load i32, ptr %3, align 4, !dbg !322
  ret void, !dbg !323
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_aliased_pointers() #0 !dbg !324 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !325, metadata !DIExpression()), !dbg !326
  call void @llvm.dbg.declare(metadata ptr %2, metadata !327, metadata !DIExpression()), !dbg !328
  store ptr %1, ptr %2, align 8, !dbg !328
  call void @llvm.dbg.declare(metadata ptr %3, metadata !329, metadata !DIExpression()), !dbg !330
  store ptr %1, ptr %3, align 8, !dbg !330
  %5 = load ptr, ptr %2, align 8, !dbg !331
  store i32 1, ptr %5, align 4, !dbg !332
  %6 = load ptr, ptr %3, align 8, !dbg !333
  store i32 2, ptr %6, align 4, !dbg !334
  call void @llvm.dbg.declare(metadata ptr %4, metadata !335, metadata !DIExpression()), !dbg !336
  %7 = load ptr, ptr %2, align 8, !dbg !337
  %8 = load i32, ptr %7, align 4, !dbg !338
  store i32 %8, ptr %4, align 4, !dbg !336
  %9 = load i32, ptr %4, align 4, !dbg !339
  ret void, !dbg !340
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !341 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_basic_strong_update(), !dbg !344
  call void @test_local_pointer_strong_update(), !dbg !345
  call void @test_struct_field_strong_update(), !dbg !346
  call void @test_weak_update_non_singleton(i32 noundef 1), !dbg !347
  call void @test_weak_update_array(i32 noundef 0), !dbg !348
  call void @test_weak_update_heap_array(i32 noundef 0), !dbg !349
  call void @test_kill_gen_sequence(), !dbg !350
  call void @test_flow_with_branches(i32 noundef 1), !dbg !351
  call void @test_partial_kill(i32 noundef 1), !dbg !352
  call void @test_mixed_updates(i32 noundef 1), !dbg !353
  call void @test_interprocedural_update(), !dbg !354
  call void @test_loop_update(i32 noundef 5), !dbg !355
  call void @test_nested_struct_update(), !dbg !356
  call void @test_aliased_pointers(), !dbg !357
  ret i32 0, !dbg !358
}

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
!1 = !DIFile(filename: "tests/fixtures/pta_verification/flow_sensitive.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "842735b45534cac26da753f8595477c7")
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
!13 = distinct !DISubprogram(name: "test_basic_strong_update", scope: !1, file: !1, line: 14, type: !14, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!14 = !DISubroutineType(types: !15)
!15 = !{null}
!16 = !{}
!17 = !DILocalVariable(name: "x", scope: !13, file: !1, line: 15, type: !4)
!18 = !DILocation(line: 15, column: 9, scope: !13)
!19 = !DILocalVariable(name: "y", scope: !13, file: !1, line: 16, type: !4)
!20 = !DILocation(line: 16, column: 9, scope: !13)
!21 = !DILocalVariable(name: "p", scope: !13, file: !1, line: 17, type: !3)
!22 = !DILocation(line: 17, column: 10, scope: !13)
!23 = !DILocation(line: 20, column: 6, scope: !13)
!24 = !DILocation(line: 20, column: 8, scope: !13)
!25 = !DILocalVariable(name: "q", scope: !13, file: !1, line: 22, type: !3)
!26 = !DILocation(line: 22, column: 10, scope: !13)
!27 = !DILocation(line: 23, column: 6, scope: !13)
!28 = !DILocation(line: 23, column: 8, scope: !13)
!29 = !DILocalVariable(name: "result", scope: !13, file: !1, line: 25, type: !4)
!30 = !DILocation(line: 25, column: 9, scope: !13)
!31 = !DILocation(line: 25, column: 19, scope: !13)
!32 = !DILocation(line: 25, column: 18, scope: !13)
!33 = !DILocation(line: 25, column: 24, scope: !13)
!34 = !DILocation(line: 25, column: 23, scope: !13)
!35 = !DILocation(line: 25, column: 21, scope: !13)
!36 = !DILocation(line: 26, column: 11, scope: !13)
!37 = !DILocation(line: 27, column: 1, scope: !13)
!38 = distinct !DISubprogram(name: "test_local_pointer_strong_update", scope: !1, file: !1, line: 30, type: !14, scopeLine: 30, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!39 = !DILocalVariable(name: "local", scope: !38, file: !1, line: 31, type: !4)
!40 = !DILocation(line: 31, column: 9, scope: !38)
!41 = !DILocalVariable(name: "ptr", scope: !38, file: !1, line: 32, type: !3)
!42 = !DILocation(line: 32, column: 10, scope: !38)
!43 = !DILocation(line: 34, column: 6, scope: !38)
!44 = !DILocation(line: 34, column: 10, scope: !38)
!45 = !DILocation(line: 35, column: 6, scope: !38)
!46 = !DILocation(line: 35, column: 10, scope: !38)
!47 = !DILocalVariable(name: "val", scope: !38, file: !1, line: 37, type: !4)
!48 = !DILocation(line: 37, column: 9, scope: !38)
!49 = !DILocation(line: 37, column: 16, scope: !38)
!50 = !DILocation(line: 37, column: 15, scope: !38)
!51 = !DILocation(line: 38, column: 11, scope: !38)
!52 = !DILocation(line: 39, column: 1, scope: !38)
!53 = distinct !DISubprogram(name: "test_struct_field_strong_update", scope: !1, file: !1, line: 47, type: !14, scopeLine: 47, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!54 = !DILocalVariable(name: "s", scope: !53, file: !1, line: 48, type: !55)
!55 = !DIDerivedType(tag: DW_TAG_typedef, name: "SimpleStruct", file: !1, line: 45, baseType: !56)
!56 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !1, line: 42, size: 64, elements: !57)
!57 = !{!58, !59}
!58 = !DIDerivedType(tag: DW_TAG_member, name: "field1", scope: !56, file: !1, line: 43, baseType: !4, size: 32)
!59 = !DIDerivedType(tag: DW_TAG_member, name: "field2", scope: !56, file: !1, line: 44, baseType: !4, size: 32, offset: 32)
!60 = !DILocation(line: 48, column: 18, scope: !53)
!61 = !DILocalVariable(name: "p", scope: !53, file: !1, line: 49, type: !3)
!62 = !DILocation(line: 49, column: 10, scope: !53)
!63 = !DILocation(line: 49, column: 17, scope: !53)
!64 = !DILocation(line: 51, column: 6, scope: !53)
!65 = !DILocation(line: 51, column: 8, scope: !53)
!66 = !DILocation(line: 52, column: 6, scope: !53)
!67 = !DILocation(line: 52, column: 8, scope: !53)
!68 = !DILocalVariable(name: "v", scope: !53, file: !1, line: 54, type: !4)
!69 = !DILocation(line: 54, column: 9, scope: !53)
!70 = !DILocation(line: 54, column: 14, scope: !53)
!71 = !DILocation(line: 54, column: 13, scope: !53)
!72 = !DILocation(line: 55, column: 11, scope: !53)
!73 = !DILocation(line: 56, column: 1, scope: !53)
!74 = distinct !DISubprogram(name: "test_weak_update_non_singleton", scope: !1, file: !1, line: 63, type: !75, scopeLine: 63, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!75 = !DISubroutineType(types: !76)
!76 = !{null, !4}
!77 = !DILocalVariable(name: "cond", arg: 1, scope: !74, file: !1, line: 63, type: !4)
!78 = !DILocation(line: 63, column: 41, scope: !74)
!79 = !DILocalVariable(name: "x", scope: !74, file: !1, line: 64, type: !4)
!80 = !DILocation(line: 64, column: 9, scope: !74)
!81 = !DILocalVariable(name: "y", scope: !74, file: !1, line: 64, type: !4)
!82 = !DILocation(line: 64, column: 12, scope: !74)
!83 = !DILocalVariable(name: "p", scope: !74, file: !1, line: 65, type: !3)
!84 = !DILocation(line: 65, column: 10, scope: !74)
!85 = !DILocation(line: 67, column: 9, scope: !86)
!86 = distinct !DILexicalBlock(scope: !74, file: !1, line: 67, column: 9)
!87 = !DILocation(line: 67, column: 9, scope: !74)
!88 = !DILocation(line: 68, column: 11, scope: !89)
!89 = distinct !DILexicalBlock(scope: !86, file: !1, line: 67, column: 15)
!90 = !DILocation(line: 69, column: 5, scope: !89)
!91 = !DILocation(line: 70, column: 11, scope: !92)
!92 = distinct !DILexicalBlock(scope: !86, file: !1, line: 69, column: 12)
!93 = !DILocation(line: 74, column: 6, scope: !74)
!94 = !DILocation(line: 74, column: 8, scope: !74)
!95 = !DILocalVariable(name: "r1", scope: !74, file: !1, line: 77, type: !4)
!96 = !DILocation(line: 77, column: 9, scope: !74)
!97 = !DILocation(line: 77, column: 14, scope: !74)
!98 = !DILocalVariable(name: "r2", scope: !74, file: !1, line: 78, type: !4)
!99 = !DILocation(line: 78, column: 9, scope: !74)
!100 = !DILocation(line: 78, column: 14, scope: !74)
!101 = !DILocation(line: 79, column: 11, scope: !74)
!102 = !DILocation(line: 80, column: 11, scope: !74)
!103 = !DILocation(line: 81, column: 1, scope: !74)
!104 = distinct !DISubprogram(name: "test_weak_update_array", scope: !1, file: !1, line: 84, type: !75, scopeLine: 84, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!105 = !DILocalVariable(name: "idx", arg: 1, scope: !104, file: !1, line: 84, type: !4)
!106 = !DILocation(line: 84, column: 33, scope: !104)
!107 = !DILocalVariable(name: "arr", scope: !104, file: !1, line: 85, type: !108)
!108 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 320, elements: !109)
!109 = !{!110}
!110 = !DISubrange(count: 10)
!111 = !DILocation(line: 85, column: 9, scope: !104)
!112 = !DILocalVariable(name: "p", scope: !104, file: !1, line: 86, type: !3)
!113 = !DILocation(line: 86, column: 10, scope: !104)
!114 = !DILocation(line: 86, column: 19, scope: !104)
!115 = !DILocation(line: 86, column: 15, scope: !104)
!116 = !DILocation(line: 88, column: 6, scope: !104)
!117 = !DILocation(line: 88, column: 8, scope: !104)
!118 = !DILocalVariable(name: "v", scope: !104, file: !1, line: 90, type: !4)
!119 = !DILocation(line: 90, column: 9, scope: !104)
!120 = !DILocation(line: 90, column: 13, scope: !104)
!121 = !DILocation(line: 91, column: 11, scope: !104)
!122 = !DILocation(line: 92, column: 1, scope: !104)
!123 = distinct !DISubprogram(name: "test_weak_update_heap_array", scope: !1, file: !1, line: 95, type: !75, scopeLine: 95, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!124 = !DILocalVariable(name: "idx", arg: 1, scope: !123, file: !1, line: 95, type: !4)
!125 = !DILocation(line: 95, column: 38, scope: !123)
!126 = !DILocalVariable(name: "arr", scope: !123, file: !1, line: 96, type: !3)
!127 = !DILocation(line: 96, column: 10, scope: !123)
!128 = !DILocation(line: 96, column: 22, scope: !123)
!129 = !DILocalVariable(name: "p", scope: !123, file: !1, line: 97, type: !3)
!130 = !DILocation(line: 97, column: 10, scope: !123)
!131 = !DILocation(line: 97, column: 14, scope: !123)
!132 = !DILocation(line: 97, column: 20, scope: !123)
!133 = !DILocation(line: 97, column: 18, scope: !123)
!134 = !DILocation(line: 99, column: 6, scope: !123)
!135 = !DILocation(line: 99, column: 8, scope: !123)
!136 = !DILocalVariable(name: "v", scope: !123, file: !1, line: 101, type: !4)
!137 = !DILocation(line: 101, column: 9, scope: !123)
!138 = !DILocation(line: 101, column: 13, scope: !123)
!139 = !DILocation(line: 102, column: 11, scope: !123)
!140 = !DILocation(line: 103, column: 10, scope: !123)
!141 = !DILocation(line: 103, column: 5, scope: !123)
!142 = !DILocation(line: 104, column: 1, scope: !123)
!143 = distinct !DISubprogram(name: "test_kill_gen_sequence", scope: !1, file: !1, line: 111, type: !14, scopeLine: 111, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!144 = !DILocalVariable(name: "x", scope: !143, file: !1, line: 112, type: !4)
!145 = !DILocation(line: 112, column: 9, scope: !143)
!146 = !DILocalVariable(name: "p", scope: !143, file: !1, line: 113, type: !3)
!147 = !DILocation(line: 113, column: 10, scope: !143)
!148 = !DILocation(line: 115, column: 6, scope: !143)
!149 = !DILocation(line: 115, column: 8, scope: !143)
!150 = !DILocation(line: 118, column: 6, scope: !143)
!151 = !DILocation(line: 118, column: 8, scope: !143)
!152 = !DILocation(line: 121, column: 6, scope: !143)
!153 = !DILocation(line: 121, column: 8, scope: !143)
!154 = !DILocalVariable(name: "final", scope: !143, file: !1, line: 124, type: !4)
!155 = !DILocation(line: 124, column: 9, scope: !143)
!156 = !DILocation(line: 124, column: 18, scope: !143)
!157 = !DILocation(line: 124, column: 17, scope: !143)
!158 = !DILocation(line: 125, column: 11, scope: !143)
!159 = !DILocation(line: 126, column: 1, scope: !143)
!160 = distinct !DISubprogram(name: "test_flow_with_branches", scope: !1, file: !1, line: 129, type: !75, scopeLine: 129, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!161 = !DILocalVariable(name: "cond", arg: 1, scope: !160, file: !1, line: 129, type: !4)
!162 = !DILocation(line: 129, column: 34, scope: !160)
!163 = !DILocalVariable(name: "x", scope: !160, file: !1, line: 130, type: !4)
!164 = !DILocation(line: 130, column: 9, scope: !160)
!165 = !DILocalVariable(name: "p", scope: !160, file: !1, line: 131, type: !3)
!166 = !DILocation(line: 131, column: 10, scope: !160)
!167 = !DILocation(line: 133, column: 9, scope: !168)
!168 = distinct !DILexicalBlock(scope: !160, file: !1, line: 133, column: 9)
!169 = !DILocation(line: 133, column: 9, scope: !160)
!170 = !DILocation(line: 134, column: 10, scope: !171)
!171 = distinct !DILexicalBlock(scope: !168, file: !1, line: 133, column: 15)
!172 = !DILocation(line: 134, column: 12, scope: !171)
!173 = !DILocation(line: 135, column: 5, scope: !171)
!174 = !DILocation(line: 136, column: 10, scope: !175)
!175 = distinct !DILexicalBlock(scope: !168, file: !1, line: 135, column: 12)
!176 = !DILocation(line: 136, column: 12, scope: !175)
!177 = !DILocalVariable(name: "v", scope: !160, file: !1, line: 141, type: !4)
!178 = !DILocation(line: 141, column: 9, scope: !160)
!179 = !DILocation(line: 141, column: 14, scope: !160)
!180 = !DILocation(line: 141, column: 13, scope: !160)
!181 = !DILocation(line: 142, column: 11, scope: !160)
!182 = !DILocation(line: 143, column: 1, scope: !160)
!183 = distinct !DISubprogram(name: "test_partial_kill", scope: !1, file: !1, line: 146, type: !75, scopeLine: 146, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!184 = !DILocalVariable(name: "cond", arg: 1, scope: !183, file: !1, line: 146, type: !4)
!185 = !DILocation(line: 146, column: 28, scope: !183)
!186 = !DILocalVariable(name: "x", scope: !183, file: !1, line: 147, type: !4)
!187 = !DILocation(line: 147, column: 9, scope: !183)
!188 = !DILocalVariable(name: "p", scope: !183, file: !1, line: 148, type: !3)
!189 = !DILocation(line: 148, column: 10, scope: !183)
!190 = !DILocation(line: 150, column: 9, scope: !191)
!191 = distinct !DILexicalBlock(scope: !183, file: !1, line: 150, column: 9)
!192 = !DILocation(line: 150, column: 9, scope: !183)
!193 = !DILocation(line: 151, column: 10, scope: !194)
!194 = distinct !DILexicalBlock(scope: !191, file: !1, line: 150, column: 15)
!195 = !DILocation(line: 151, column: 12, scope: !194)
!196 = !DILocation(line: 152, column: 5, scope: !194)
!197 = !DILocalVariable(name: "v", scope: !183, file: !1, line: 156, type: !4)
!198 = !DILocation(line: 156, column: 9, scope: !183)
!199 = !DILocation(line: 156, column: 14, scope: !183)
!200 = !DILocation(line: 156, column: 13, scope: !183)
!201 = !DILocation(line: 157, column: 11, scope: !183)
!202 = !DILocation(line: 158, column: 1, scope: !183)
!203 = distinct !DISubprogram(name: "test_mixed_updates", scope: !1, file: !1, line: 165, type: !75, scopeLine: 165, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!204 = !DILocalVariable(name: "cond", arg: 1, scope: !203, file: !1, line: 165, type: !4)
!205 = !DILocation(line: 165, column: 29, scope: !203)
!206 = !DILocalVariable(name: "a", scope: !203, file: !1, line: 166, type: !4)
!207 = !DILocation(line: 166, column: 9, scope: !203)
!208 = !DILocalVariable(name: "b", scope: !203, file: !1, line: 166, type: !4)
!209 = !DILocation(line: 166, column: 12, scope: !203)
!210 = !DILocalVariable(name: "arr", scope: !203, file: !1, line: 167, type: !211)
!211 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 160, elements: !212)
!212 = !{!213}
!213 = !DISubrange(count: 5)
!214 = !DILocation(line: 167, column: 9, scope: !203)
!215 = !DILocalVariable(name: "p1", scope: !203, file: !1, line: 168, type: !3)
!216 = !DILocation(line: 168, column: 10, scope: !203)
!217 = !DILocalVariable(name: "p2", scope: !203, file: !1, line: 169, type: !3)
!218 = !DILocation(line: 169, column: 10, scope: !203)
!219 = !DILocation(line: 171, column: 9, scope: !220)
!220 = distinct !DILexicalBlock(scope: !203, file: !1, line: 171, column: 9)
!221 = !DILocation(line: 171, column: 9, scope: !203)
!222 = !DILocation(line: 172, column: 12, scope: !223)
!223 = distinct !DILexicalBlock(scope: !220, file: !1, line: 171, column: 15)
!224 = !DILocation(line: 173, column: 5, scope: !223)
!225 = !DILocation(line: 174, column: 12, scope: !226)
!226 = distinct !DILexicalBlock(scope: !220, file: !1, line: 173, column: 12)
!227 = !DILocation(line: 178, column: 6, scope: !203)
!228 = !DILocation(line: 178, column: 9, scope: !203)
!229 = !DILocation(line: 179, column: 6, scope: !203)
!230 = !DILocation(line: 179, column: 9, scope: !203)
!231 = !DILocalVariable(name: "parr", scope: !203, file: !1, line: 181, type: !3)
!232 = !DILocation(line: 181, column: 10, scope: !203)
!233 = !DILocation(line: 181, column: 18, scope: !203)
!234 = !DILocation(line: 182, column: 6, scope: !203)
!235 = !DILocation(line: 182, column: 11, scope: !203)
!236 = !DILocalVariable(name: "r1", scope: !203, file: !1, line: 184, type: !4)
!237 = !DILocation(line: 184, column: 9, scope: !203)
!238 = !DILocation(line: 184, column: 14, scope: !203)
!239 = !DILocalVariable(name: "r2", scope: !203, file: !1, line: 185, type: !4)
!240 = !DILocation(line: 185, column: 9, scope: !203)
!241 = !DILocation(line: 185, column: 14, scope: !203)
!242 = !DILocation(line: 186, column: 11, scope: !203)
!243 = !DILocation(line: 187, column: 11, scope: !203)
!244 = !DILocation(line: 188, column: 1, scope: !203)
!245 = distinct !DISubprogram(name: "modify_through_ptr", scope: !1, file: !1, line: 191, type: !246, scopeLine: 191, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!246 = !DISubroutineType(types: !247)
!247 = !{null, !3}
!248 = !DILocalVariable(name: "ptr", arg: 1, scope: !245, file: !1, line: 191, type: !3)
!249 = !DILocation(line: 191, column: 30, scope: !245)
!250 = !DILocation(line: 192, column: 6, scope: !245)
!251 = !DILocation(line: 192, column: 10, scope: !245)
!252 = !DILocation(line: 193, column: 1, scope: !245)
!253 = distinct !DISubprogram(name: "test_interprocedural_update", scope: !1, file: !1, line: 195, type: !14, scopeLine: 195, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!254 = !DILocalVariable(name: "x", scope: !253, file: !1, line: 196, type: !4)
!255 = !DILocation(line: 196, column: 9, scope: !253)
!256 = !DILocalVariable(name: "p", scope: !253, file: !1, line: 197, type: !3)
!257 = !DILocation(line: 197, column: 10, scope: !253)
!258 = !DILocation(line: 199, column: 6, scope: !253)
!259 = !DILocation(line: 199, column: 8, scope: !253)
!260 = !DILocation(line: 200, column: 24, scope: !253)
!261 = !DILocation(line: 200, column: 5, scope: !253)
!262 = !DILocalVariable(name: "v", scope: !253, file: !1, line: 203, type: !4)
!263 = !DILocation(line: 203, column: 9, scope: !253)
!264 = !DILocation(line: 203, column: 14, scope: !253)
!265 = !DILocation(line: 203, column: 13, scope: !253)
!266 = !DILocation(line: 204, column: 11, scope: !253)
!267 = !DILocation(line: 205, column: 1, scope: !253)
!268 = distinct !DISubprogram(name: "test_loop_update", scope: !1, file: !1, line: 208, type: !75, scopeLine: 208, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!269 = !DILocalVariable(name: "n", arg: 1, scope: !268, file: !1, line: 208, type: !4)
!270 = !DILocation(line: 208, column: 27, scope: !268)
!271 = !DILocalVariable(name: "x", scope: !268, file: !1, line: 209, type: !4)
!272 = !DILocation(line: 209, column: 9, scope: !268)
!273 = !DILocalVariable(name: "p", scope: !268, file: !1, line: 210, type: !3)
!274 = !DILocation(line: 210, column: 10, scope: !268)
!275 = !DILocalVariable(name: "i", scope: !276, file: !1, line: 212, type: !4)
!276 = distinct !DILexicalBlock(scope: !268, file: !1, line: 212, column: 5)
!277 = !DILocation(line: 212, column: 14, scope: !276)
!278 = !DILocation(line: 212, column: 10, scope: !276)
!279 = !DILocation(line: 212, column: 21, scope: !280)
!280 = distinct !DILexicalBlock(scope: !276, file: !1, line: 212, column: 5)
!281 = !DILocation(line: 212, column: 25, scope: !280)
!282 = !DILocation(line: 212, column: 23, scope: !280)
!283 = !DILocation(line: 212, column: 5, scope: !276)
!284 = !DILocation(line: 213, column: 14, scope: !285)
!285 = distinct !DILexicalBlock(scope: !280, file: !1, line: 212, column: 33)
!286 = !DILocation(line: 213, column: 10, scope: !285)
!287 = !DILocation(line: 213, column: 12, scope: !285)
!288 = !DILocation(line: 214, column: 5, scope: !285)
!289 = !DILocation(line: 212, column: 29, scope: !280)
!290 = !DILocation(line: 212, column: 5, scope: !280)
!291 = distinct !{!291, !283, !292, !293}
!292 = !DILocation(line: 214, column: 5, scope: !276)
!293 = !{!"llvm.loop.mustprogress"}
!294 = !DILocalVariable(name: "v", scope: !268, file: !1, line: 217, type: !4)
!295 = !DILocation(line: 217, column: 9, scope: !268)
!296 = !DILocation(line: 217, column: 14, scope: !268)
!297 = !DILocation(line: 217, column: 13, scope: !268)
!298 = !DILocation(line: 218, column: 11, scope: !268)
!299 = !DILocation(line: 219, column: 1, scope: !268)
!300 = distinct !DISubprogram(name: "test_nested_struct_update", scope: !1, file: !1, line: 231, type: !14, scopeLine: 231, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!301 = !DILocalVariable(name: "o", scope: !300, file: !1, line: 232, type: !302)
!302 = !DIDerivedType(tag: DW_TAG_typedef, name: "Outer", file: !1, line: 229, baseType: !303)
!303 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !1, line: 226, size: 64, elements: !304)
!304 = !{!305, !310}
!305 = !DIDerivedType(tag: DW_TAG_member, name: "inner", scope: !303, file: !1, line: 227, baseType: !306, size: 32)
!306 = !DIDerivedType(tag: DW_TAG_typedef, name: "Inner", file: !1, line: 224, baseType: !307)
!307 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !1, line: 222, size: 32, elements: !308)
!308 = !{!309}
!309 = !DIDerivedType(tag: DW_TAG_member, name: "val", scope: !307, file: !1, line: 223, baseType: !4, size: 32)
!310 = !DIDerivedType(tag: DW_TAG_member, name: "other", scope: !303, file: !1, line: 228, baseType: !4, size: 32, offset: 32)
!311 = !DILocation(line: 232, column: 11, scope: !300)
!312 = !DILocalVariable(name: "p", scope: !300, file: !1, line: 233, type: !3)
!313 = !DILocation(line: 233, column: 10, scope: !300)
!314 = !DILocation(line: 233, column: 17, scope: !300)
!315 = !DILocation(line: 233, column: 23, scope: !300)
!316 = !DILocation(line: 235, column: 6, scope: !300)
!317 = !DILocation(line: 235, column: 8, scope: !300)
!318 = !DILocalVariable(name: "v", scope: !300, file: !1, line: 237, type: !4)
!319 = !DILocation(line: 237, column: 9, scope: !300)
!320 = !DILocation(line: 237, column: 15, scope: !300)
!321 = !DILocation(line: 237, column: 21, scope: !300)
!322 = !DILocation(line: 238, column: 11, scope: !300)
!323 = !DILocation(line: 239, column: 1, scope: !300)
!324 = distinct !DISubprogram(name: "test_aliased_pointers", scope: !1, file: !1, line: 242, type: !14, scopeLine: 242, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!325 = !DILocalVariable(name: "x", scope: !324, file: !1, line: 243, type: !4)
!326 = !DILocation(line: 243, column: 9, scope: !324)
!327 = !DILocalVariable(name: "p", scope: !324, file: !1, line: 244, type: !3)
!328 = !DILocation(line: 244, column: 10, scope: !324)
!329 = !DILocalVariable(name: "q", scope: !324, file: !1, line: 245, type: !3)
!330 = !DILocation(line: 245, column: 10, scope: !324)
!331 = !DILocation(line: 247, column: 6, scope: !324)
!332 = !DILocation(line: 247, column: 8, scope: !324)
!333 = !DILocation(line: 250, column: 6, scope: !324)
!334 = !DILocation(line: 250, column: 8, scope: !324)
!335 = !DILocalVariable(name: "v", scope: !324, file: !1, line: 253, type: !4)
!336 = !DILocation(line: 253, column: 9, scope: !324)
!337 = !DILocation(line: 253, column: 14, scope: !324)
!338 = !DILocation(line: 253, column: 13, scope: !324)
!339 = !DILocation(line: 254, column: 11, scope: !324)
!340 = !DILocation(line: 255, column: 1, scope: !324)
!341 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 257, type: !342, scopeLine: 257, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!342 = !DISubroutineType(types: !343)
!343 = !{!4}
!344 = !DILocation(line: 258, column: 5, scope: !341)
!345 = !DILocation(line: 259, column: 5, scope: !341)
!346 = !DILocation(line: 260, column: 5, scope: !341)
!347 = !DILocation(line: 261, column: 5, scope: !341)
!348 = !DILocation(line: 262, column: 5, scope: !341)
!349 = !DILocation(line: 263, column: 5, scope: !341)
!350 = !DILocation(line: 264, column: 5, scope: !341)
!351 = !DILocation(line: 265, column: 5, scope: !341)
!352 = !DILocation(line: 266, column: 5, scope: !341)
!353 = !DILocation(line: 267, column: 5, scope: !341)
!354 = !DILocation(line: 268, column: 5, scope: !341)
!355 = !DILocation(line: 269, column: 5, scope: !341)
!356 = !DILocation(line: 270, column: 5, scope: !341)
!357 = !DILocation(line: 271, column: 5, scope: !341)
!358 = !DILocation(line: 272, column: 5, scope: !341)
