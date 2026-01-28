; ModuleID = 'mssa_integration.cpp'
source_filename = "mssa_integration.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.Container = type { i32 }

$_ZN9ContainerC2Ei = comdat any

$_ZN9Container8setValueEi = comdat any

$_ZNK9Container8getValueEv = comdat any

$_ZN9Container9incrementEv = comdat any

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z17test_field_accessv() #0 {
entry:
  %c = alloca %class.Container, align 4
  %r = alloca i32, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 10)
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 20)
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  store i32 %call, ptr %r, align 4
  %0 = load i32, ptr %r, align 4
  call void @_Z4sinki(i32 noundef %0)
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %this, i32 noundef %v) unnamed_addr #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %v.addr = alloca i32, align 4
  store ptr %this, ptr %this.addr, align 8
  store i32 %v, ptr %v.addr, align 4
  %this1 = load ptr, ptr %this.addr, align 8
  %value = getelementptr inbounds %class.Container, ptr %this1, i32 0, i32 0
  %0 = load i32, ptr %v.addr, align 4
  store i32 %0, ptr %value, align 4
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %this, i32 noundef %v) #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %v.addr = alloca i32, align 4
  store ptr %this, ptr %this.addr, align 8
  store i32 %v, ptr %v.addr, align 4
  %this1 = load ptr, ptr %this.addr, align 8
  %0 = load i32, ptr %v.addr, align 4
  %value = getelementptr inbounds %class.Container, ptr %this1, i32 0, i32 0
  store i32 %0, ptr %value, align 4
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %this) #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %value = getelementptr inbounds %class.Container, ptr %this1, i32 0, i32 0
  %0 = load i32, ptr %value, align 4
  ret i32 %0
}

declare void @_Z4sinki(i32 noundef) #2

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z16test_constructorv() #0 {
entry:
  %c = alloca %class.Container, align 4
  %r = alloca i32, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 42)
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  store i32 %call, ptr %r, align 4
  %0 = load i32, ptr %r, align 4
  call void @_Z4sinki(i32 noundef %0)
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z17test_method_chainv() #0 {
entry:
  %c = alloca %class.Container, align 4
  %r = alloca i32, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 0)
  call void @_ZN9Container9incrementEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  call void @_ZN9Container9incrementEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  call void @_ZN9Container9incrementEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  store i32 %call, ptr %r, align 4
  %0 = load i32, ptr %r, align 4
  call void @_Z4sinki(i32 noundef %0)
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9Container9incrementEv(ptr noundef nonnull align 4 dereferenceable(4) %this) #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %value = getelementptr inbounds %class.Container, ptr %this1, i32 0, i32 0
  %0 = load i32, ptr %value, align 4
  %add = add nsw i32 %0, 1
  %value2 = getelementptr inbounds %class.Container, ptr %this1, i32 0, i32 0
  store i32 %add, ptr %value2, align 4
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z21test_multiple_objectsv() #0 {
entry:
  %c1 = alloca %class.Container, align 4
  %c2 = alloca %class.Container, align 4
  %r1 = alloca i32, align 4
  %r2 = alloca i32, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c1, i32 noundef 100)
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c2, i32 noundef 200)
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %c1, i32 noundef 111)
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %c2, i32 noundef 222)
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c1)
  store i32 %call, ptr %r1, align 4
  %call1 = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c2)
  store i32 %call1, ptr %r2, align 4
  %0 = load i32, ptr %r1, align 4
  call void @_Z4sinki(i32 noundef %0)
  %1 = load i32, ptr %r2, align 4
  call void @_Z4sinki(i32 noundef %1)
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z19test_object_pointerv() #0 {
entry:
  %c = alloca %class.Container, align 4
  %p = alloca ptr, align 8
  %r = alloca i32, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 0)
  store ptr %c, ptr %p, align 8
  %0 = load ptr, ptr %p, align 8
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %0, i32 noundef 50)
  %1 = load ptr, ptr %p, align 8
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %1)
  store i32 %call, ptr %r, align 4
  %2 = load i32, ptr %r, align 4
  call void @_Z4sinki(i32 noundef %2)
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z22test_conditional_fieldi(i32 noundef %cond) #0 {
entry:
  %cond.addr = alloca i32, align 4
  %c = alloca %class.Container, align 4
  %r = alloca i32, align 4
  store i32 %cond, ptr %cond.addr, align 4
  call void @_ZN9ContainerC2Ei(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 0)
  %0 = load i32, ptr %cond.addr, align 4
  %tobool = icmp ne i32 %0, 0
  br i1 %tobool, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 10)
  br label %if.end

if.else:                                          ; preds = %entry
  call void @_ZN9Container8setValueEi(ptr noundef nonnull align 4 dereferenceable(4) %c, i32 noundef 20)
  br label %if.end

if.end:                                           ; preds = %if.else, %if.then
  %call = call noundef i32 @_ZNK9Container8getValueEv(ptr noundef nonnull align 4 dereferenceable(4) %c)
  store i32 %call, ptr %r, align 4
  %1 = load i32, ptr %r, align 4
  call void @_Z4sinki(i32 noundef %1)
  ret void
}

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #3 {
entry:
  %retval = alloca i32, align 4
  store i32 0, ptr %retval, align 4
  call void @_Z17test_field_accessv()
  call void @_Z16test_constructorv()
  call void @_Z17test_method_chainv()
  call void @_Z21test_multiple_objectsv()
  call void @_Z19test_object_pointerv()
  call void @_Z22test_conditional_fieldi(i32 noundef 1)
  ret i32 0
}

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
