; ModuleID = 'tests/fixtures/e2e/source/ps_cpp_raii_guard.cpp'
source_filename = "tests/fixtures/e2e/source/ps_cpp_raii_guard.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.Resource = type <{ ptr, i8, [7 x i8] }>

$_ZN8ResourceC2Ev = comdat any

$_ZN8Resource8use_dataEv = comdat any

$_ZN8Resource7releaseEv = comdat any

$_ZN8ResourceD2Ev = comdat any

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #0 personality ptr @__gxx_personality_v0 {
entry:
  %retval = alloca i32, align 4
  %r = alloca %class.Resource, align 8
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  store i32 0, ptr %retval, align 4
  call void @_ZN8ResourceC2Ev(ptr noundef nonnull align 8 dereferenceable(9) %r)
  invoke void @_ZN8Resource8use_dataEv(ptr noundef nonnull align 8 dereferenceable(9) %r)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %entry
  invoke void @_ZN8Resource7releaseEv(ptr noundef nonnull align 8 dereferenceable(9) %r)
          to label %invoke.cont1 unwind label %lpad

invoke.cont1:                                     ; preds = %invoke.cont
  store i32 0, ptr %retval, align 4
  call void @_ZN8ResourceD2Ev(ptr noundef nonnull align 8 dereferenceable(9) %r) #6
  %0 = load i32, ptr %retval, align 4
  ret i32 %0

lpad:                                             ; preds = %invoke.cont, %entry
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  store ptr %2, ptr %exn.slot, align 8
  %3 = extractvalue { ptr, i32 } %1, 1
  store i32 %3, ptr %ehselector.slot, align 4
  call void @_ZN8ResourceD2Ev(ptr noundef nonnull align 8 dereferenceable(9) %r) #6
  br label %eh.resume

eh.resume:                                        ; preds = %lpad
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val2 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val2
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8ResourceC2Ev(ptr noundef nonnull align 8 dereferenceable(9) %this) unnamed_addr #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %data = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %call = call noalias ptr @malloc(i64 noundef 40) #7
  store ptr %call, ptr %data, align 8
  %released = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 1
  store i8 0, ptr %released, align 8
  %data2 = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %data2, align 8
  %tobool = icmp ne ptr %0, null
  br i1 %tobool, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  %data3 = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %1 = load ptr, ptr %data3, align 8
  store i32 0, ptr %1, align 4
  br label %if.end

if.end:                                           ; preds = %if.then, %entry
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN8Resource8use_dataEv(ptr noundef nonnull align 8 dereferenceable(9) %this) #2 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %data = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %data, align 8
  %tobool = icmp ne ptr %0, null
  br i1 %tobool, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %released = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 1
  %1 = load i8, ptr %released, align 8
  %tobool2 = trunc i8 %1 to i1
  br i1 %tobool2, label %if.end, label %if.then

if.then:                                          ; preds = %land.lhs.true
  %data3 = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %2 = load ptr, ptr %data3, align 8
  call void @use_ptr(ptr noundef %2)
  br label %if.end

if.end:                                           ; preds = %if.then, %land.lhs.true, %entry
  ret void
}

declare i32 @__gxx_personality_v0(...)

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8Resource7releaseEv(ptr noundef nonnull align 8 dereferenceable(9) %this) #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %data = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %data, align 8
  %tobool = icmp ne ptr %0, null
  br i1 %tobool, label %land.lhs.true, label %if.end

land.lhs.true:                                    ; preds = %entry
  %released = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 1
  %1 = load i8, ptr %released, align 8
  %tobool2 = trunc i8 %1 to i1
  br i1 %tobool2, label %if.end, label %if.then

if.then:                                          ; preds = %land.lhs.true
  %data3 = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 0
  %2 = load ptr, ptr %data3, align 8
  call void @free(ptr noundef %2) #6
  %released4 = getelementptr inbounds %class.Resource, ptr %this1, i32 0, i32 1
  store i8 1, ptr %released4, align 8
  br label %if.end

if.end:                                           ; preds = %if.then, %land.lhs.true, %entry
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8ResourceD2Ev(ptr noundef nonnull align 8 dereferenceable(9) %this) unnamed_addr #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  call void @_ZN8Resource7releaseEv(ptr noundef nonnull align 8 dereferenceable(9) %this1)
  ret void
}

; Function Attrs: nounwind allocsize(0)
declare noalias ptr @malloc(i64 noundef) #3

declare void @use_ptr(ptr noundef) #4

; Function Attrs: nounwind
declare void @free(ptr noundef) #5

attributes #0 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nounwind allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #4 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #6 = { nounwind }
attributes #7 = { nounwind allocsize(0) }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
