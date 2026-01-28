; ModuleID = 'tests/fixtures/pta_verification/recursive_calls.c'
source_filename = "tests/fixtures/pta_verification/recursive_calls.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct.Node = type { i32, ptr }

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @factorial(i32 noundef %0) #0 !dbg !19 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !23, metadata !DIExpression()), !dbg !24
  %4 = load i32, ptr %3, align 4, !dbg !25
  %5 = icmp sle i32 %4, 1, !dbg !27
  br i1 %5, label %6, label %7, !dbg !28

6:                                                ; preds = %1
  store i32 1, ptr %2, align 4, !dbg !29
  br label %13, !dbg !29

7:                                                ; preds = %1
  %8 = load i32, ptr %3, align 4, !dbg !30
  %9 = load i32, ptr %3, align 4, !dbg !31
  %10 = sub nsw i32 %9, 1, !dbg !32
  %11 = call i32 @factorial(i32 noundef %10), !dbg !33
  %12 = mul nsw i32 %8, %11, !dbg !34
  store i32 %12, ptr %2, align 4, !dbg !35
  br label %13, !dbg !35

13:                                               ; preds = %7, %6
  %14 = load i32, ptr %2, align 4, !dbg !36
  ret i32 %14, !dbg !36
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_self_recursion() #0 !dbg !37 {
  %1 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !40, metadata !DIExpression()), !dbg !41
  %2 = call i32 @factorial(i32 noundef 5), !dbg !42
  store i32 %2, ptr %1, align 4, !dbg !41
  %3 = load i32, ptr %1, align 4, !dbg !43
  ret void, !dbg !44
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @is_even(i32 noundef %0) #0 !dbg !45 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !46, metadata !DIExpression()), !dbg !47
  %4 = load i32, ptr %3, align 4, !dbg !48
  %5 = icmp eq i32 %4, 0, !dbg !50
  br i1 %5, label %6, label %7, !dbg !51

6:                                                ; preds = %1
  store i32 1, ptr %2, align 4, !dbg !52
  br label %11, !dbg !52

7:                                                ; preds = %1
  %8 = load i32, ptr %3, align 4, !dbg !53
  %9 = sub nsw i32 %8, 1, !dbg !54
  %10 = call i32 @is_odd(i32 noundef %9), !dbg !55
  store i32 %10, ptr %2, align 4, !dbg !56
  br label %11, !dbg !56

11:                                               ; preds = %7, %6
  %12 = load i32, ptr %2, align 4, !dbg !57
  ret i32 %12, !dbg !57
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @is_odd(i32 noundef %0) #0 !dbg !58 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !59, metadata !DIExpression()), !dbg !60
  %4 = load i32, ptr %3, align 4, !dbg !61
  %5 = icmp eq i32 %4, 0, !dbg !63
  br i1 %5, label %6, label %7, !dbg !64

6:                                                ; preds = %1
  store i32 0, ptr %2, align 4, !dbg !65
  br label %11, !dbg !65

7:                                                ; preds = %1
  %8 = load i32, ptr %3, align 4, !dbg !66
  %9 = sub nsw i32 %8, 1, !dbg !67
  %10 = call i32 @is_even(i32 noundef %9), !dbg !68
  store i32 %10, ptr %2, align 4, !dbg !69
  br label %11, !dbg !69

11:                                               ; preds = %7, %6
  %12 = load i32, ptr %2, align 4, !dbg !70
  ret i32 %12, !dbg !70
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_mutual_recursion() #0 !dbg !71 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !72, metadata !DIExpression()), !dbg !73
  %3 = call i32 @is_even(i32 noundef 4), !dbg !74
  store i32 %3, ptr %1, align 4, !dbg !73
  call void @llvm.dbg.declare(metadata ptr %2, metadata !75, metadata !DIExpression()), !dbg !76
  %4 = call i32 @is_odd(i32 noundef 3), !dbg !77
  store i32 %4, ptr %2, align 4, !dbg !76
  %5 = load i32, ptr %1, align 4, !dbg !78
  %6 = load i32, ptr %2, align 4, !dbg !79
  ret void, !dbg !80
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @recursive_identity(ptr noundef %0, i32 noundef %1) #0 !dbg !81 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !85, metadata !DIExpression()), !dbg !86
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !87, metadata !DIExpression()), !dbg !88
  %6 = load i32, ptr %5, align 4, !dbg !89
  %7 = icmp sle i32 %6, 0, !dbg !91
  br i1 %7, label %8, label %10, !dbg !92

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !93
  store ptr %9, ptr %3, align 8, !dbg !94
  br label %15, !dbg !94

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !95
  %12 = load i32, ptr %5, align 4, !dbg !96
  %13 = sub nsw i32 %12, 1, !dbg !97
  %14 = call ptr @recursive_identity(ptr noundef %11, i32 noundef %13), !dbg !98
  store ptr %14, ptr %3, align 8, !dbg !99
  br label %15, !dbg !99

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !100
  ret ptr %16, !dbg !100
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_recursive_pointer() #0 !dbg !101 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !102, metadata !DIExpression()), !dbg !103
  call void @llvm.dbg.declare(metadata ptr %2, metadata !104, metadata !DIExpression()), !dbg !105
  %3 = call ptr @recursive_identity(ptr noundef %1, i32 noundef 5), !dbg !106
  store ptr %3, ptr %2, align 8, !dbg !105
  %4 = load ptr, ptr %2, align 8, !dbg !107
  ret void, !dbg !108
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @func_a(ptr noundef %0, i32 noundef %1) #0 !dbg !109 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !110, metadata !DIExpression()), !dbg !111
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !112, metadata !DIExpression()), !dbg !113
  %6 = load i32, ptr %5, align 4, !dbg !114
  %7 = icmp sle i32 %6, 0, !dbg !116
  br i1 %7, label %8, label %10, !dbg !117

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !118
  store ptr %9, ptr %3, align 8, !dbg !119
  br label %15, !dbg !119

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !120
  %12 = load i32, ptr %5, align 4, !dbg !121
  %13 = sub nsw i32 %12, 1, !dbg !122
  %14 = call ptr @func_b(ptr noundef %11, i32 noundef %13), !dbg !123
  store ptr %14, ptr %3, align 8, !dbg !124
  br label %15, !dbg !124

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !125
  ret ptr %16, !dbg !125
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @func_b(ptr noundef %0, i32 noundef %1) #0 !dbg !126 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !127, metadata !DIExpression()), !dbg !128
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !129, metadata !DIExpression()), !dbg !130
  %6 = load i32, ptr %5, align 4, !dbg !131
  %7 = icmp sle i32 %6, 0, !dbg !133
  br i1 %7, label %8, label %10, !dbg !134

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !135
  store ptr %9, ptr %3, align 8, !dbg !136
  br label %15, !dbg !136

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !137
  %12 = load i32, ptr %5, align 4, !dbg !138
  %13 = sub nsw i32 %12, 1, !dbg !139
  %14 = call ptr @func_c(ptr noundef %11, i32 noundef %13), !dbg !140
  store ptr %14, ptr %3, align 8, !dbg !141
  br label %15, !dbg !141

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !142
  ret ptr %16, !dbg !142
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @func_c(ptr noundef %0, i32 noundef %1) #0 !dbg !143 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !144, metadata !DIExpression()), !dbg !145
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !146, metadata !DIExpression()), !dbg !147
  %6 = load i32, ptr %5, align 4, !dbg !148
  %7 = icmp sle i32 %6, 0, !dbg !150
  br i1 %7, label %8, label %10, !dbg !151

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !152
  store ptr %9, ptr %3, align 8, !dbg !153
  br label %15, !dbg !153

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !154
  %12 = load i32, ptr %5, align 4, !dbg !155
  %13 = sub nsw i32 %12, 1, !dbg !156
  %14 = call ptr @func_a(ptr noundef %11, i32 noundef %13), !dbg !157
  store ptr %14, ptr %3, align 8, !dbg !158
  br label %15, !dbg !158

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !159
  ret ptr %16, !dbg !159
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_three_way_recursion() #0 !dbg !160 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !161, metadata !DIExpression()), !dbg !162
  call void @llvm.dbg.declare(metadata ptr %2, metadata !163, metadata !DIExpression()), !dbg !164
  %3 = call ptr @func_a(ptr noundef %1, i32 noundef 10), !dbg !165
  store ptr %3, ptr %2, align 8, !dbg !164
  %4 = load ptr, ptr %2, align 8, !dbg !166
  ret void, !dbg !167
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @build_list(i32 noundef %0) #0 !dbg !168 {
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca ptr, align 8
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !171, metadata !DIExpression()), !dbg !172
  %5 = load i32, ptr %3, align 4, !dbg !173
  %6 = icmp sle i32 %5, 0, !dbg !175
  br i1 %6, label %7, label %8, !dbg !176

7:                                                ; preds = %1
  store ptr null, ptr %2, align 8, !dbg !177
  br label %19, !dbg !177

8:                                                ; preds = %1
  call void @llvm.dbg.declare(metadata ptr %4, metadata !178, metadata !DIExpression()), !dbg !179
  %9 = call noalias ptr @malloc(i64 noundef 16) #3, !dbg !180
  store ptr %9, ptr %4, align 8, !dbg !179
  %10 = load i32, ptr %3, align 4, !dbg !181
  %11 = load ptr, ptr %4, align 8, !dbg !182
  %12 = getelementptr inbounds %struct.Node, ptr %11, i32 0, i32 0, !dbg !183
  store i32 %10, ptr %12, align 8, !dbg !184
  %13 = load i32, ptr %3, align 4, !dbg !185
  %14 = sub nsw i32 %13, 1, !dbg !186
  %15 = call ptr @build_list(i32 noundef %14), !dbg !187
  %16 = load ptr, ptr %4, align 8, !dbg !188
  %17 = getelementptr inbounds %struct.Node, ptr %16, i32 0, i32 1, !dbg !189
  store ptr %15, ptr %17, align 8, !dbg !190
  %18 = load ptr, ptr %4, align 8, !dbg !191
  store ptr %18, ptr %2, align 8, !dbg !192
  br label %19, !dbg !192

19:                                               ; preds = %8, %7
  %20 = load ptr, ptr %2, align 8, !dbg !193
  ret ptr %20, !dbg !193
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_recursive_allocation() #0 !dbg !194 {
  %1 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !195, metadata !DIExpression()), !dbg !196
  %2 = call ptr @build_list(i32 noundef 5), !dbg !197
  store ptr %2, ptr %1, align 8, !dbg !196
  %3 = load ptr, ptr %1, align 8, !dbg !198
  ret void, !dbg !199
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @indirect_recurse(ptr noundef %0, i32 noundef %1) #0 !dbg !200 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca ptr, align 8
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !201, metadata !DIExpression()), !dbg !202
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !203, metadata !DIExpression()), !dbg !204
  %7 = load i32, ptr %5, align 4, !dbg !205
  %8 = icmp sle i32 %7, 0, !dbg !207
  br i1 %8, label %9, label %11, !dbg !208

9:                                                ; preds = %2
  %10 = load ptr, ptr %4, align 8, !dbg !209
  store ptr %10, ptr %3, align 8, !dbg !210
  br label %17, !dbg !210

11:                                               ; preds = %2
  call void @llvm.dbg.declare(metadata ptr %6, metadata !211, metadata !DIExpression()), !dbg !214
  store ptr @indirect_recurse, ptr %6, align 8, !dbg !214
  %12 = load ptr, ptr %6, align 8, !dbg !215
  %13 = load ptr, ptr %4, align 8, !dbg !216
  %14 = load i32, ptr %5, align 4, !dbg !217
  %15 = sub nsw i32 %14, 1, !dbg !218
  %16 = call ptr %12(ptr noundef %13, i32 noundef %15), !dbg !215
  store ptr %16, ptr %3, align 8, !dbg !219
  br label %17, !dbg !219

17:                                               ; preds = %11, %9
  %18 = load ptr, ptr %3, align 8, !dbg !220
  ret ptr %18, !dbg !220
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_indirect_recursion() #0 !dbg !221 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !222, metadata !DIExpression()), !dbg !223
  call void @llvm.dbg.declare(metadata ptr %2, metadata !224, metadata !DIExpression()), !dbg !225
  %3 = call ptr @indirect_recurse(ptr noundef %1, i32 noundef 3), !dbg !226
  store ptr %3, ptr %2, align 8, !dbg !225
  %4 = load ptr, ptr %2, align 8, !dbg !227
  ret void, !dbg !228
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @wrapper_entry(ptr noundef %0) #0 !dbg !229 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !232, metadata !DIExpression()), !dbg !233
  %3 = load ptr, ptr %2, align 8, !dbg !234
  %4 = call ptr @helper_recursive(ptr noundef %3, i32 noundef 5), !dbg !235
  ret ptr %4, !dbg !236
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @helper_recursive(ptr noundef %0, i32 noundef %1) #0 !dbg !237 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !238, metadata !DIExpression()), !dbg !239
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !240, metadata !DIExpression()), !dbg !241
  %6 = load i32, ptr %5, align 4, !dbg !242
  %7 = icmp sle i32 %6, 0, !dbg !244
  br i1 %7, label %8, label %10, !dbg !245

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !246
  store ptr %9, ptr %3, align 8, !dbg !247
  br label %15, !dbg !247

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !248
  %12 = load i32, ptr %5, align 4, !dbg !249
  %13 = sub nsw i32 %12, 1, !dbg !250
  %14 = call ptr @helper_recursive(ptr noundef %11, i32 noundef %13), !dbg !251
  store ptr %14, ptr %3, align 8, !dbg !252
  br label %15, !dbg !252

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !253
  ret ptr %16, !dbg !253
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_recursive_wrapper() #0 !dbg !254 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !255, metadata !DIExpression()), !dbg !256
  call void @llvm.dbg.declare(metadata ptr %2, metadata !257, metadata !DIExpression()), !dbg !258
  call void @llvm.dbg.declare(metadata ptr %3, metadata !259, metadata !DIExpression()), !dbg !260
  %5 = call ptr @wrapper_entry(ptr noundef %1), !dbg !261
  store ptr %5, ptr %3, align 8, !dbg !260
  call void @llvm.dbg.declare(metadata ptr %4, metadata !262, metadata !DIExpression()), !dbg !263
  %6 = call ptr @wrapper_entry(ptr noundef %2), !dbg !264
  store ptr %6, ptr %4, align 8, !dbg !263
  %7 = load ptr, ptr %3, align 8, !dbg !265
  %8 = load ptr, ptr %4, align 8, !dbg !266
  ret void, !dbg !267
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @diamond_b(ptr noundef %0, i32 noundef %1) #0 !dbg !268 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !269, metadata !DIExpression()), !dbg !270
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !271, metadata !DIExpression()), !dbg !272
  %5 = load ptr, ptr %3, align 8, !dbg !273
  %6 = load i32, ptr %4, align 4, !dbg !274
  %7 = call ptr @diamond_d(ptr noundef %5, i32 noundef %6), !dbg !275
  ret ptr %7, !dbg !276
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @diamond_d(ptr noundef %0, i32 noundef %1) #0 !dbg !277 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !278, metadata !DIExpression()), !dbg !279
  store i32 %1, ptr %5, align 4
  call void @llvm.dbg.declare(metadata ptr %5, metadata !280, metadata !DIExpression()), !dbg !281
  %6 = load i32, ptr %5, align 4, !dbg !282
  %7 = icmp sle i32 %6, 0, !dbg !284
  br i1 %7, label %8, label %10, !dbg !285

8:                                                ; preds = %2
  %9 = load ptr, ptr %4, align 8, !dbg !286
  store ptr %9, ptr %3, align 8, !dbg !287
  br label %15, !dbg !287

10:                                               ; preds = %2
  %11 = load ptr, ptr %4, align 8, !dbg !288
  %12 = load i32, ptr %5, align 4, !dbg !289
  %13 = sub nsw i32 %12, 1, !dbg !290
  %14 = call ptr @diamond_d(ptr noundef %11, i32 noundef %13), !dbg !291
  store ptr %14, ptr %3, align 8, !dbg !292
  br label %15, !dbg !292

15:                                               ; preds = %10, %8
  %16 = load ptr, ptr %3, align 8, !dbg !293
  ret ptr %16, !dbg !293
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @diamond_c(ptr noundef %0, i32 noundef %1) #0 !dbg !294 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !295, metadata !DIExpression()), !dbg !296
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !297, metadata !DIExpression()), !dbg !298
  %5 = load ptr, ptr %3, align 8, !dbg !299
  %6 = load i32, ptr %4, align 4, !dbg !300
  %7 = call ptr @diamond_d(ptr noundef %5, i32 noundef %6), !dbg !301
  ret ptr %7, !dbg !302
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @diamond_a(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 !dbg !303 {
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !306, metadata !DIExpression()), !dbg !307
  store i32 %1, ptr %6, align 4
  call void @llvm.dbg.declare(metadata ptr %6, metadata !308, metadata !DIExpression()), !dbg !309
  store i32 %2, ptr %7, align 4
  call void @llvm.dbg.declare(metadata ptr %7, metadata !310, metadata !DIExpression()), !dbg !311
  %8 = load i32, ptr %6, align 4, !dbg !312
  %9 = icmp ne i32 %8, 0, !dbg !312
  br i1 %9, label %10, label %14, !dbg !314

10:                                               ; preds = %3
  %11 = load ptr, ptr %5, align 8, !dbg !315
  %12 = load i32, ptr %7, align 4, !dbg !317
  %13 = call ptr @diamond_b(ptr noundef %11, i32 noundef %12), !dbg !318
  store ptr %13, ptr %4, align 8, !dbg !319
  br label %18, !dbg !319

14:                                               ; preds = %3
  %15 = load ptr, ptr %5, align 8, !dbg !320
  %16 = load i32, ptr %7, align 4, !dbg !322
  %17 = call ptr @diamond_c(ptr noundef %15, i32 noundef %16), !dbg !323
  store ptr %17, ptr %4, align 8, !dbg !324
  br label %18, !dbg !324

18:                                               ; preds = %14, %10
  %19 = load ptr, ptr %4, align 8, !dbg !325
  ret ptr %19, !dbg !325
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @test_diamond_recursive() #0 !dbg !326 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !327, metadata !DIExpression()), !dbg !328
  call void @llvm.dbg.declare(metadata ptr %2, metadata !329, metadata !DIExpression()), !dbg !330
  %3 = call ptr @diamond_a(ptr noundef %1, i32 noundef 1, i32 noundef 5), !dbg !331
  store ptr %3, ptr %2, align 8, !dbg !330
  %4 = load ptr, ptr %2, align 8, !dbg !332
  ret void, !dbg !333
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !334 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @test_self_recursion(), !dbg !337
  call void @test_mutual_recursion(), !dbg !338
  call void @test_recursive_pointer(), !dbg !339
  call void @test_three_way_recursion(), !dbg !340
  call void @test_recursive_allocation(), !dbg !341
  call void @test_indirect_recursion(), !dbg !342
  call void @test_recursive_wrapper(), !dbg !343
  call void @test_diamond_recursive(), !dbg !344
  ret i32 0, !dbg !345
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind allocsize(0) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!11, !12, !13, !14, !15, !16, !17}
!llvm.ident = !{!18}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/fixtures/pta_verification/recursive_calls.c", directory: "/workspace", checksumkind: CSK_MD5, checksum: "63942e083d926c5ac931d0d9a391d15a")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = !DIDerivedType(tag: DW_TAG_typedef, name: "Node", file: !1, line: 84, baseType: !5)
!5 = distinct !DICompositeType(tag: DW_TAG_structure_type, name: "Node", file: !1, line: 81, size: 128, elements: !6)
!6 = !{!7, !9}
!7 = !DIDerivedType(tag: DW_TAG_member, name: "value", scope: !5, file: !1, line: 82, baseType: !8, size: 32)
!8 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!9 = !DIDerivedType(tag: DW_TAG_member, name: "next", scope: !5, file: !1, line: 83, baseType: !10, size: 64, offset: 64)
!10 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !5, size: 64)
!11 = !{i32 7, !"Dwarf Version", i32 5}
!12 = !{i32 2, !"Debug Info Version", i32 3}
!13 = !{i32 1, !"wchar_size", i32 4}
!14 = !{i32 8, !"PIC Level", i32 2}
!15 = !{i32 7, !"PIE Level", i32 2}
!16 = !{i32 7, !"uwtable", i32 2}
!17 = !{i32 7, !"frame-pointer", i32 1}
!18 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!19 = distinct !DISubprogram(name: "factorial", scope: !1, file: !1, line: 8, type: !20, scopeLine: 8, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!20 = !DISubroutineType(types: !21)
!21 = !{!8, !8}
!22 = !{}
!23 = !DILocalVariable(name: "n", arg: 1, scope: !19, file: !1, line: 8, type: !8)
!24 = !DILocation(line: 8, column: 19, scope: !19)
!25 = !DILocation(line: 9, column: 9, scope: !26)
!26 = distinct !DILexicalBlock(scope: !19, file: !1, line: 9, column: 9)
!27 = !DILocation(line: 9, column: 11, scope: !26)
!28 = !DILocation(line: 9, column: 9, scope: !19)
!29 = !DILocation(line: 9, column: 17, scope: !26)
!30 = !DILocation(line: 10, column: 12, scope: !19)
!31 = !DILocation(line: 10, column: 26, scope: !19)
!32 = !DILocation(line: 10, column: 28, scope: !19)
!33 = !DILocation(line: 10, column: 16, scope: !19)
!34 = !DILocation(line: 10, column: 14, scope: !19)
!35 = !DILocation(line: 10, column: 5, scope: !19)
!36 = !DILocation(line: 11, column: 1, scope: !19)
!37 = distinct !DISubprogram(name: "test_self_recursion", scope: !1, file: !1, line: 13, type: !38, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!38 = !DISubroutineType(types: !39)
!39 = !{null}
!40 = !DILocalVariable(name: "result", scope: !37, file: !1, line: 14, type: !8)
!41 = !DILocation(line: 14, column: 9, scope: !37)
!42 = !DILocation(line: 14, column: 18, scope: !37)
!43 = !DILocation(line: 15, column: 11, scope: !37)
!44 = !DILocation(line: 16, column: 1, scope: !37)
!45 = distinct !DISubprogram(name: "is_even", scope: !1, file: !1, line: 23, type: !20, scopeLine: 23, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!46 = !DILocalVariable(name: "n", arg: 1, scope: !45, file: !1, line: 23, type: !8)
!47 = !DILocation(line: 23, column: 17, scope: !45)
!48 = !DILocation(line: 24, column: 9, scope: !49)
!49 = distinct !DILexicalBlock(scope: !45, file: !1, line: 24, column: 9)
!50 = !DILocation(line: 24, column: 11, scope: !49)
!51 = !DILocation(line: 24, column: 9, scope: !45)
!52 = !DILocation(line: 24, column: 17, scope: !49)
!53 = !DILocation(line: 25, column: 19, scope: !45)
!54 = !DILocation(line: 25, column: 21, scope: !45)
!55 = !DILocation(line: 25, column: 12, scope: !45)
!56 = !DILocation(line: 25, column: 5, scope: !45)
!57 = !DILocation(line: 26, column: 1, scope: !45)
!58 = distinct !DISubprogram(name: "is_odd", scope: !1, file: !1, line: 28, type: !20, scopeLine: 28, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!59 = !DILocalVariable(name: "n", arg: 1, scope: !58, file: !1, line: 28, type: !8)
!60 = !DILocation(line: 28, column: 16, scope: !58)
!61 = !DILocation(line: 29, column: 9, scope: !62)
!62 = distinct !DILexicalBlock(scope: !58, file: !1, line: 29, column: 9)
!63 = !DILocation(line: 29, column: 11, scope: !62)
!64 = !DILocation(line: 29, column: 9, scope: !58)
!65 = !DILocation(line: 29, column: 17, scope: !62)
!66 = !DILocation(line: 30, column: 20, scope: !58)
!67 = !DILocation(line: 30, column: 22, scope: !58)
!68 = !DILocation(line: 30, column: 12, scope: !58)
!69 = !DILocation(line: 30, column: 5, scope: !58)
!70 = !DILocation(line: 31, column: 1, scope: !58)
!71 = distinct !DISubprogram(name: "test_mutual_recursion", scope: !1, file: !1, line: 33, type: !38, scopeLine: 33, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!72 = !DILocalVariable(name: "r1", scope: !71, file: !1, line: 34, type: !8)
!73 = !DILocation(line: 34, column: 9, scope: !71)
!74 = !DILocation(line: 34, column: 14, scope: !71)
!75 = !DILocalVariable(name: "r2", scope: !71, file: !1, line: 35, type: !8)
!76 = !DILocation(line: 35, column: 9, scope: !71)
!77 = !DILocation(line: 35, column: 14, scope: !71)
!78 = !DILocation(line: 36, column: 11, scope: !71)
!79 = !DILocation(line: 37, column: 11, scope: !71)
!80 = !DILocation(line: 38, column: 1, scope: !71)
!81 = distinct !DISubprogram(name: "recursive_identity", scope: !1, file: !1, line: 42, type: !82, scopeLine: 42, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!82 = !DISubroutineType(types: !83)
!83 = !{!84, !84, !8}
!84 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!85 = !DILocalVariable(name: "p", arg: 1, scope: !81, file: !1, line: 42, type: !84)
!86 = !DILocation(line: 42, column: 32, scope: !81)
!87 = !DILocalVariable(name: "depth", arg: 2, scope: !81, file: !1, line: 42, type: !8)
!88 = !DILocation(line: 42, column: 39, scope: !81)
!89 = !DILocation(line: 43, column: 9, scope: !90)
!90 = distinct !DILexicalBlock(scope: !81, file: !1, line: 43, column: 9)
!91 = !DILocation(line: 43, column: 15, scope: !90)
!92 = !DILocation(line: 43, column: 9, scope: !81)
!93 = !DILocation(line: 43, column: 28, scope: !90)
!94 = !DILocation(line: 43, column: 21, scope: !90)
!95 = !DILocation(line: 44, column: 31, scope: !81)
!96 = !DILocation(line: 44, column: 34, scope: !81)
!97 = !DILocation(line: 44, column: 40, scope: !81)
!98 = !DILocation(line: 44, column: 12, scope: !81)
!99 = !DILocation(line: 44, column: 5, scope: !81)
!100 = !DILocation(line: 45, column: 1, scope: !81)
!101 = distinct !DISubprogram(name: "test_recursive_pointer", scope: !1, file: !1, line: 47, type: !38, scopeLine: 47, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!102 = !DILocalVariable(name: "x", scope: !101, file: !1, line: 48, type: !8)
!103 = !DILocation(line: 48, column: 9, scope: !101)
!104 = !DILocalVariable(name: "r", scope: !101, file: !1, line: 49, type: !84)
!105 = !DILocation(line: 49, column: 11, scope: !101)
!106 = !DILocation(line: 49, column: 15, scope: !101)
!107 = !DILocation(line: 50, column: 11, scope: !101)
!108 = !DILocation(line: 51, column: 1, scope: !101)
!109 = distinct !DISubprogram(name: "func_a", scope: !1, file: !1, line: 59, type: !82, scopeLine: 59, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!110 = !DILocalVariable(name: "p", arg: 1, scope: !109, file: !1, line: 59, type: !84)
!111 = !DILocation(line: 59, column: 20, scope: !109)
!112 = !DILocalVariable(name: "n", arg: 2, scope: !109, file: !1, line: 59, type: !8)
!113 = !DILocation(line: 59, column: 27, scope: !109)
!114 = !DILocation(line: 60, column: 9, scope: !115)
!115 = distinct !DILexicalBlock(scope: !109, file: !1, line: 60, column: 9)
!116 = !DILocation(line: 60, column: 11, scope: !115)
!117 = !DILocation(line: 60, column: 9, scope: !109)
!118 = !DILocation(line: 60, column: 24, scope: !115)
!119 = !DILocation(line: 60, column: 17, scope: !115)
!120 = !DILocation(line: 61, column: 19, scope: !109)
!121 = !DILocation(line: 61, column: 22, scope: !109)
!122 = !DILocation(line: 61, column: 24, scope: !109)
!123 = !DILocation(line: 61, column: 12, scope: !109)
!124 = !DILocation(line: 61, column: 5, scope: !109)
!125 = !DILocation(line: 62, column: 1, scope: !109)
!126 = distinct !DISubprogram(name: "func_b", scope: !1, file: !1, line: 64, type: !82, scopeLine: 64, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!127 = !DILocalVariable(name: "p", arg: 1, scope: !126, file: !1, line: 64, type: !84)
!128 = !DILocation(line: 64, column: 20, scope: !126)
!129 = !DILocalVariable(name: "n", arg: 2, scope: !126, file: !1, line: 64, type: !8)
!130 = !DILocation(line: 64, column: 27, scope: !126)
!131 = !DILocation(line: 65, column: 9, scope: !132)
!132 = distinct !DILexicalBlock(scope: !126, file: !1, line: 65, column: 9)
!133 = !DILocation(line: 65, column: 11, scope: !132)
!134 = !DILocation(line: 65, column: 9, scope: !126)
!135 = !DILocation(line: 65, column: 24, scope: !132)
!136 = !DILocation(line: 65, column: 17, scope: !132)
!137 = !DILocation(line: 66, column: 19, scope: !126)
!138 = !DILocation(line: 66, column: 22, scope: !126)
!139 = !DILocation(line: 66, column: 24, scope: !126)
!140 = !DILocation(line: 66, column: 12, scope: !126)
!141 = !DILocation(line: 66, column: 5, scope: !126)
!142 = !DILocation(line: 67, column: 1, scope: !126)
!143 = distinct !DISubprogram(name: "func_c", scope: !1, file: !1, line: 69, type: !82, scopeLine: 69, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!144 = !DILocalVariable(name: "p", arg: 1, scope: !143, file: !1, line: 69, type: !84)
!145 = !DILocation(line: 69, column: 20, scope: !143)
!146 = !DILocalVariable(name: "n", arg: 2, scope: !143, file: !1, line: 69, type: !8)
!147 = !DILocation(line: 69, column: 27, scope: !143)
!148 = !DILocation(line: 70, column: 9, scope: !149)
!149 = distinct !DILexicalBlock(scope: !143, file: !1, line: 70, column: 9)
!150 = !DILocation(line: 70, column: 11, scope: !149)
!151 = !DILocation(line: 70, column: 9, scope: !143)
!152 = !DILocation(line: 70, column: 24, scope: !149)
!153 = !DILocation(line: 70, column: 17, scope: !149)
!154 = !DILocation(line: 71, column: 19, scope: !143)
!155 = !DILocation(line: 71, column: 22, scope: !143)
!156 = !DILocation(line: 71, column: 24, scope: !143)
!157 = !DILocation(line: 71, column: 12, scope: !143)
!158 = !DILocation(line: 71, column: 5, scope: !143)
!159 = !DILocation(line: 72, column: 1, scope: !143)
!160 = distinct !DISubprogram(name: "test_three_way_recursion", scope: !1, file: !1, line: 74, type: !38, scopeLine: 74, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!161 = !DILocalVariable(name: "val", scope: !160, file: !1, line: 75, type: !8)
!162 = !DILocation(line: 75, column: 9, scope: !160)
!163 = !DILocalVariable(name: "r", scope: !160, file: !1, line: 76, type: !84)
!164 = !DILocation(line: 76, column: 11, scope: !160)
!165 = !DILocation(line: 76, column: 15, scope: !160)
!166 = !DILocation(line: 77, column: 11, scope: !160)
!167 = !DILocation(line: 78, column: 1, scope: !160)
!168 = distinct !DISubprogram(name: "build_list", scope: !1, file: !1, line: 86, type: !169, scopeLine: 86, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!169 = !DISubroutineType(types: !170)
!170 = !{!3, !8}
!171 = !DILocalVariable(name: "n", arg: 1, scope: !168, file: !1, line: 86, type: !8)
!172 = !DILocation(line: 86, column: 22, scope: !168)
!173 = !DILocation(line: 87, column: 9, scope: !174)
!174 = distinct !DILexicalBlock(scope: !168, file: !1, line: 87, column: 9)
!175 = !DILocation(line: 87, column: 11, scope: !174)
!176 = !DILocation(line: 87, column: 9, scope: !168)
!177 = !DILocation(line: 87, column: 17, scope: !174)
!178 = !DILocalVariable(name: "node", scope: !168, file: !1, line: 88, type: !3)
!179 = !DILocation(line: 88, column: 11, scope: !168)
!180 = !DILocation(line: 88, column: 25, scope: !168)
!181 = !DILocation(line: 89, column: 19, scope: !168)
!182 = !DILocation(line: 89, column: 5, scope: !168)
!183 = !DILocation(line: 89, column: 11, scope: !168)
!184 = !DILocation(line: 89, column: 17, scope: !168)
!185 = !DILocation(line: 90, column: 29, scope: !168)
!186 = !DILocation(line: 90, column: 31, scope: !168)
!187 = !DILocation(line: 90, column: 18, scope: !168)
!188 = !DILocation(line: 90, column: 5, scope: !168)
!189 = !DILocation(line: 90, column: 11, scope: !168)
!190 = !DILocation(line: 90, column: 16, scope: !168)
!191 = !DILocation(line: 91, column: 12, scope: !168)
!192 = !DILocation(line: 91, column: 5, scope: !168)
!193 = !DILocation(line: 92, column: 1, scope: !168)
!194 = distinct !DISubprogram(name: "test_recursive_allocation", scope: !1, file: !1, line: 94, type: !38, scopeLine: 94, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!195 = !DILocalVariable(name: "list", scope: !194, file: !1, line: 95, type: !3)
!196 = !DILocation(line: 95, column: 11, scope: !194)
!197 = !DILocation(line: 95, column: 18, scope: !194)
!198 = !DILocation(line: 96, column: 11, scope: !194)
!199 = !DILocation(line: 97, column: 1, scope: !194)
!200 = distinct !DISubprogram(name: "indirect_recurse", scope: !1, file: !1, line: 102, type: !82, scopeLine: 102, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!201 = !DILocalVariable(name: "p", arg: 1, scope: !200, file: !1, line: 102, type: !84)
!202 = !DILocation(line: 102, column: 30, scope: !200)
!203 = !DILocalVariable(name: "n", arg: 2, scope: !200, file: !1, line: 102, type: !8)
!204 = !DILocation(line: 102, column: 37, scope: !200)
!205 = !DILocation(line: 103, column: 9, scope: !206)
!206 = distinct !DILexicalBlock(scope: !200, file: !1, line: 103, column: 9)
!207 = !DILocation(line: 103, column: 11, scope: !206)
!208 = !DILocation(line: 103, column: 9, scope: !200)
!209 = !DILocation(line: 103, column: 24, scope: !206)
!210 = !DILocation(line: 103, column: 17, scope: !206)
!211 = !DILocalVariable(name: "fn", scope: !200, file: !1, line: 104, type: !212)
!212 = !DIDerivedType(tag: DW_TAG_typedef, name: "RecFn", file: !1, line: 100, baseType: !213)
!213 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !82, size: 64)
!214 = !DILocation(line: 104, column: 11, scope: !200)
!215 = !DILocation(line: 105, column: 12, scope: !200)
!216 = !DILocation(line: 105, column: 15, scope: !200)
!217 = !DILocation(line: 105, column: 18, scope: !200)
!218 = !DILocation(line: 105, column: 20, scope: !200)
!219 = !DILocation(line: 105, column: 5, scope: !200)
!220 = !DILocation(line: 106, column: 1, scope: !200)
!221 = distinct !DISubprogram(name: "test_indirect_recursion", scope: !1, file: !1, line: 108, type: !38, scopeLine: 108, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!222 = !DILocalVariable(name: "x", scope: !221, file: !1, line: 109, type: !8)
!223 = !DILocation(line: 109, column: 9, scope: !221)
!224 = !DILocalVariable(name: "r", scope: !221, file: !1, line: 110, type: !84)
!225 = !DILocation(line: 110, column: 11, scope: !221)
!226 = !DILocation(line: 110, column: 15, scope: !221)
!227 = !DILocation(line: 111, column: 11, scope: !221)
!228 = !DILocation(line: 112, column: 1, scope: !221)
!229 = distinct !DISubprogram(name: "wrapper_entry", scope: !1, file: !1, line: 118, type: !230, scopeLine: 118, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!230 = !DISubroutineType(types: !231)
!231 = !{!84, !84}
!232 = !DILocalVariable(name: "p", arg: 1, scope: !229, file: !1, line: 118, type: !84)
!233 = !DILocation(line: 118, column: 27, scope: !229)
!234 = !DILocation(line: 119, column: 29, scope: !229)
!235 = !DILocation(line: 119, column: 12, scope: !229)
!236 = !DILocation(line: 119, column: 5, scope: !229)
!237 = distinct !DISubprogram(name: "helper_recursive", scope: !1, file: !1, line: 122, type: !82, scopeLine: 122, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!238 = !DILocalVariable(name: "p", arg: 1, scope: !237, file: !1, line: 122, type: !84)
!239 = !DILocation(line: 122, column: 30, scope: !237)
!240 = !DILocalVariable(name: "depth", arg: 2, scope: !237, file: !1, line: 122, type: !8)
!241 = !DILocation(line: 122, column: 37, scope: !237)
!242 = !DILocation(line: 123, column: 9, scope: !243)
!243 = distinct !DILexicalBlock(scope: !237, file: !1, line: 123, column: 9)
!244 = !DILocation(line: 123, column: 15, scope: !243)
!245 = !DILocation(line: 123, column: 9, scope: !237)
!246 = !DILocation(line: 123, column: 28, scope: !243)
!247 = !DILocation(line: 123, column: 21, scope: !243)
!248 = !DILocation(line: 124, column: 29, scope: !237)
!249 = !DILocation(line: 124, column: 32, scope: !237)
!250 = !DILocation(line: 124, column: 38, scope: !237)
!251 = !DILocation(line: 124, column: 12, scope: !237)
!252 = !DILocation(line: 124, column: 5, scope: !237)
!253 = !DILocation(line: 125, column: 1, scope: !237)
!254 = distinct !DISubprogram(name: "test_recursive_wrapper", scope: !1, file: !1, line: 127, type: !38, scopeLine: 127, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!255 = !DILocalVariable(name: "a", scope: !254, file: !1, line: 128, type: !8)
!256 = !DILocation(line: 128, column: 9, scope: !254)
!257 = !DILocalVariable(name: "b", scope: !254, file: !1, line: 128, type: !8)
!258 = !DILocation(line: 128, column: 12, scope: !254)
!259 = !DILocalVariable(name: "r1", scope: !254, file: !1, line: 131, type: !84)
!260 = !DILocation(line: 131, column: 11, scope: !254)
!261 = !DILocation(line: 131, column: 16, scope: !254)
!262 = !DILocalVariable(name: "r2", scope: !254, file: !1, line: 132, type: !84)
!263 = !DILocation(line: 132, column: 11, scope: !254)
!264 = !DILocation(line: 132, column: 16, scope: !254)
!265 = !DILocation(line: 133, column: 11, scope: !254)
!266 = !DILocation(line: 134, column: 11, scope: !254)
!267 = !DILocation(line: 135, column: 1, scope: !254)
!268 = distinct !DISubprogram(name: "diamond_b", scope: !1, file: !1, line: 142, type: !82, scopeLine: 142, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!269 = !DILocalVariable(name: "p", arg: 1, scope: !268, file: !1, line: 142, type: !84)
!270 = !DILocation(line: 142, column: 23, scope: !268)
!271 = !DILocalVariable(name: "n", arg: 2, scope: !268, file: !1, line: 142, type: !8)
!272 = !DILocation(line: 142, column: 30, scope: !268)
!273 = !DILocation(line: 143, column: 22, scope: !268)
!274 = !DILocation(line: 143, column: 25, scope: !268)
!275 = !DILocation(line: 143, column: 12, scope: !268)
!276 = !DILocation(line: 143, column: 5, scope: !268)
!277 = distinct !DISubprogram(name: "diamond_d", scope: !1, file: !1, line: 150, type: !82, scopeLine: 150, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!278 = !DILocalVariable(name: "p", arg: 1, scope: !277, file: !1, line: 150, type: !84)
!279 = !DILocation(line: 150, column: 23, scope: !277)
!280 = !DILocalVariable(name: "n", arg: 2, scope: !277, file: !1, line: 150, type: !8)
!281 = !DILocation(line: 150, column: 30, scope: !277)
!282 = !DILocation(line: 151, column: 9, scope: !283)
!283 = distinct !DILexicalBlock(scope: !277, file: !1, line: 151, column: 9)
!284 = !DILocation(line: 151, column: 11, scope: !283)
!285 = !DILocation(line: 151, column: 9, scope: !277)
!286 = !DILocation(line: 151, column: 24, scope: !283)
!287 = !DILocation(line: 151, column: 17, scope: !283)
!288 = !DILocation(line: 152, column: 22, scope: !277)
!289 = !DILocation(line: 152, column: 25, scope: !277)
!290 = !DILocation(line: 152, column: 27, scope: !277)
!291 = !DILocation(line: 152, column: 12, scope: !277)
!292 = !DILocation(line: 152, column: 5, scope: !277)
!293 = !DILocation(line: 153, column: 1, scope: !277)
!294 = distinct !DISubprogram(name: "diamond_c", scope: !1, file: !1, line: 146, type: !82, scopeLine: 146, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!295 = !DILocalVariable(name: "p", arg: 1, scope: !294, file: !1, line: 146, type: !84)
!296 = !DILocation(line: 146, column: 23, scope: !294)
!297 = !DILocalVariable(name: "n", arg: 2, scope: !294, file: !1, line: 146, type: !8)
!298 = !DILocation(line: 146, column: 30, scope: !294)
!299 = !DILocation(line: 147, column: 22, scope: !294)
!300 = !DILocation(line: 147, column: 25, scope: !294)
!301 = !DILocation(line: 147, column: 12, scope: !294)
!302 = !DILocation(line: 147, column: 5, scope: !294)
!303 = distinct !DISubprogram(name: "diamond_a", scope: !1, file: !1, line: 155, type: !304, scopeLine: 155, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!304 = !DISubroutineType(types: !305)
!305 = !{!84, !84, !8, !8}
!306 = !DILocalVariable(name: "p", arg: 1, scope: !303, file: !1, line: 155, type: !84)
!307 = !DILocation(line: 155, column: 23, scope: !303)
!308 = !DILocalVariable(name: "cond", arg: 2, scope: !303, file: !1, line: 155, type: !8)
!309 = !DILocation(line: 155, column: 30, scope: !303)
!310 = !DILocalVariable(name: "n", arg: 3, scope: !303, file: !1, line: 155, type: !8)
!311 = !DILocation(line: 155, column: 40, scope: !303)
!312 = !DILocation(line: 156, column: 9, scope: !313)
!313 = distinct !DILexicalBlock(scope: !303, file: !1, line: 156, column: 9)
!314 = !DILocation(line: 156, column: 9, scope: !303)
!315 = !DILocation(line: 157, column: 26, scope: !316)
!316 = distinct !DILexicalBlock(scope: !313, file: !1, line: 156, column: 15)
!317 = !DILocation(line: 157, column: 29, scope: !316)
!318 = !DILocation(line: 157, column: 16, scope: !316)
!319 = !DILocation(line: 157, column: 9, scope: !316)
!320 = !DILocation(line: 159, column: 26, scope: !321)
!321 = distinct !DILexicalBlock(scope: !313, file: !1, line: 158, column: 12)
!322 = !DILocation(line: 159, column: 29, scope: !321)
!323 = !DILocation(line: 159, column: 16, scope: !321)
!324 = !DILocation(line: 159, column: 9, scope: !321)
!325 = !DILocation(line: 161, column: 1, scope: !303)
!326 = distinct !DISubprogram(name: "test_diamond_recursive", scope: !1, file: !1, line: 163, type: !38, scopeLine: 163, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !22)
!327 = !DILocalVariable(name: "x", scope: !326, file: !1, line: 164, type: !8)
!328 = !DILocation(line: 164, column: 9, scope: !326)
!329 = !DILocalVariable(name: "r", scope: !326, file: !1, line: 165, type: !84)
!330 = !DILocation(line: 165, column: 11, scope: !326)
!331 = !DILocation(line: 165, column: 15, scope: !326)
!332 = !DILocation(line: 166, column: 11, scope: !326)
!333 = !DILocation(line: 167, column: 1, scope: !326)
!334 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 169, type: !335, scopeLine: 169, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!335 = !DISubroutineType(types: !336)
!336 = !{!8}
!337 = !DILocation(line: 170, column: 5, scope: !334)
!338 = !DILocation(line: 171, column: 5, scope: !334)
!339 = !DILocation(line: 172, column: 5, scope: !334)
!340 = !DILocation(line: 173, column: 5, scope: !334)
!341 = !DILocation(line: 174, column: 5, scope: !334)
!342 = !DILocation(line: 175, column: 5, scope: !334)
!343 = !DILocation(line: 176, column: 5, scope: !334)
!344 = !DILocation(line: 177, column: 5, scope: !334)
!345 = !DILocation(line: 178, column: 5, scope: !334)
