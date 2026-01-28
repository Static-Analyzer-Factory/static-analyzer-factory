; ModuleID = 'wrapped_arithmetic.c'
source_filename = "wrapped_arithmetic.c"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @i8_overflow() #0 {
entry:
  %x = alloca i8, align 1
  %y = alloca i8, align 1
  store i8 127, ptr %x, align 1
  %0 = load i8, ptr %x, align 1
  %conv = sext i8 %0 to i32
  %add = add nsw i32 %conv, 1
  %conv1 = trunc i32 %add to i8
  store i8 %conv1, ptr %y, align 1
  %1 = load i8, ptr %y, align 1
  call void @sink_i8(i8 noundef %1)
  ret void
}

declare void @sink_i8(i8 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @u8_wrap() #0 {
entry:
  %x = alloca i8, align 1
  %y = alloca i8, align 1
  store i8 -1, ptr %x, align 1
  %0 = load i8, ptr %x, align 1
  %conv = zext i8 %0 to i32
  %add = add nsw i32 %conv, 1
  %conv1 = trunc i32 %add to i8
  store i8 %conv1, ptr %y, align 1
  %1 = load i8, ptr %y, align 1
  call void @sink_u8(i8 noundef %1)
  ret void
}

declare void @sink_u8(i8 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @i32_overflow() #0 {
entry:
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  store i32 2147483647, ptr %x, align 4
  %0 = load i32, ptr %x, align 4
  %add = add nsw i32 %0, 1
  store i32 %add, ptr %y, align 4
  %1 = load i32, ptr %y, align 4
  call void @sink_i32(i32 noundef %1)
  ret void
}

declare void @sink_i32(i32 noundef) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @mul_overflow() #0 {
entry:
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  store i32 100000, ptr %x, align 4
  %0 = load i32, ptr %x, align 4
  %mul = mul nsw i32 %0, 100000
  store i32 %mul, ptr %y, align 4
  %1 = load i32, ptr %y, align 4
  call void @sink_i32(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @sub_underflow() #0 {
entry:
  %x = alloca i8, align 1
  %y = alloca i8, align 1
  store i8 -128, ptr %x, align 1
  %0 = load i8, ptr %x, align 1
  %conv = sext i8 %0 to i32
  %sub = sub nsw i32 %conv, 1
  %conv1 = trunc i32 %sub to i8
  store i8 %conv1, ptr %y, align 1
  %1 = load i8, ptr %y, align 1
  call void @sink_i8(i8 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @shift_overflow() #0 {
entry:
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  store i32 1, ptr %x, align 4
  %0 = load i32, ptr %x, align 4
  %shl = shl i32 %0, 31
  store i32 %shl, ptr %y, align 4
  %1 = load i32, ptr %y, align 4
  call void @sink_i32(i32 noundef %1)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @division_by_range(i32 noundef %x) #0 {
entry:
  %retval = alloca i32, align 4
  %x.addr = alloca i32, align 4
  store i32 %x, ptr %x.addr, align 4
  %0 = load i32, ptr %x.addr, align 4
  %cmp = icmp sgt i32 %0, 0
  br i1 %cmp, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %1 = load i32, ptr %x.addr, align 4
  %cmp1 = icmp slt i32 %1, 10
  br i1 %cmp1, label %if.then, label %if.end

if.then:                                          ; preds = %land.lhs.true
  %2 = load i32, ptr %x.addr, align 4
  %div = sdiv i32 100, %2
  store i32 %div, ptr %retval, align 4
  br label %return

if.end:                                           ; preds = %land.lhs.true, %entry
  store i32 0, ptr %retval, align 4
  br label %return

return:                                           ; preds = %if.end, %if.then
  %3 = load i32, ptr %retval, align 4
  ret i32 %3
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @signed_unsigned_mix() #0 {
entry:
  %s = alloca i32, align 4
  %u = alloca i32, align 4
  store i32 -1, ptr %s, align 4
  %0 = load i32, ptr %s, align 4
  store i32 %0, ptr %u, align 4
  %1 = load i32, ptr %u, align 4
  call void @sink_u32(i32 noundef %1)
  ret void
}

declare void @sink_u32(i32 noundef) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
