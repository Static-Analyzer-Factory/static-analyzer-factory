; ModuleID = 'ifds_multi_hop_taint.c'
source_filename = "ifds_multi_hop_taint.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@.str = private unnamed_addr constant [9 x i8] c"DEEP_CMD\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @step_three(ptr noundef %data) #0 {
entry:
  %data.addr = alloca ptr, align 8
  store ptr %data, ptr %data.addr, align 8
  %0 = load ptr, ptr %data.addr, align 8
  ret ptr %0
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @step_two(ptr noundef %data) #0 {
entry:
  %data.addr = alloca ptr, align 8
  store ptr %data, ptr %data.addr, align 8
  %0 = load ptr, ptr %data.addr, align 8
  %call = call ptr @step_three(ptr noundef %0)
  ret ptr %call
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @step_one(ptr noundef %data) #0 {
entry:
  %data.addr = alloca ptr, align 8
  store ptr %data, ptr %data.addr, align 8
  %0 = load ptr, ptr %data.addr, align 8
  %call = call ptr @step_two(ptr noundef %0)
  ret ptr %call
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
entry:
  %retval = alloca i32, align 4
  %env = alloca ptr, align 8
  %result = alloca ptr, align 8
  store i32 0, ptr %retval, align 4
  %call = call ptr @getenv(ptr noundef @.str) #3
  store ptr %call, ptr %env, align 8
  %0 = load ptr, ptr %env, align 8
  %call1 = call ptr @step_one(ptr noundef %0)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  %call2 = call i32 @system(ptr noundef %1)
  ret i32 0
}

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #1

declare i32 @system(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
