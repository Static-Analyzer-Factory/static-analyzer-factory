; ModuleID = 'ifds_cpp_class_taint.cpp'
source_filename = "ifds_cpp_class_taint.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.CommandWrapper = type { ptr }

$_ZN14CommandWrapperC2EPc = comdat any

$_ZN14CommandWrapper3getEv = comdat any

@.str = private unnamed_addr constant [4 x i8] c"CMD\00", align 1

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #0 {
entry:
  %retval = alloca i32, align 4
  %env = alloca ptr, align 8
  %wrapper = alloca %class.CommandWrapper, align 8
  %result = alloca ptr, align 8
  store i32 0, ptr %retval, align 4
  %call = call ptr @getenv(ptr noundef @.str) #4
  store ptr %call, ptr %env, align 8
  %0 = load ptr, ptr %env, align 8
  call void @_ZN14CommandWrapperC2EPc(ptr noundef nonnull align 8 dereferenceable(8) %wrapper, ptr noundef %0)
  %call1 = call noundef ptr @_ZN14CommandWrapper3getEv(ptr noundef nonnull align 8 dereferenceable(8) %wrapper)
  store ptr %call1, ptr %result, align 8
  %1 = load ptr, ptr %result, align 8
  %call2 = call i32 @system(ptr noundef %1)
  ret i32 0
}

; Function Attrs: nounwind
declare ptr @getenv(ptr noundef) #1

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN14CommandWrapperC2EPc(ptr noundef nonnull align 8 dereferenceable(8) %this, ptr noundef %input) unnamed_addr #2 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %input.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  store ptr %input, ptr %input.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %cmd = getelementptr inbounds %class.CommandWrapper, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %input.addr, align 8
  store ptr %0, ptr %cmd, align 8
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef ptr @_ZN14CommandWrapper3getEv(ptr noundef nonnull align 8 dereferenceable(8) %this) #2 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %cmd = getelementptr inbounds %class.CommandWrapper, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %cmd, align 8
  ret ptr %0
}

declare i32 @system(ptr noundef) #3

attributes #0 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { nounwind }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
