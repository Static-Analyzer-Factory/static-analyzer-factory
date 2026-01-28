; ModuleID = 'ide_typestate.cpp'
source_filename = "ide_typestate.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.File = type { i32 }
%class.DataProcessor = type { ptr }

$_ZN4FileC2EPKc = comdat any

$_ZN4File9read_dataEPci = comdat any

$_ZN4File10close_fileEv = comdat any

$_ZN4FileD2Ev = comdat any

$_ZN13DataProcessor7processEv = comdat any

$_ZN13DataProcessor6finishEv = comdat any

@.str = private unnamed_addr constant [9 x i8] c"test.txt\00", align 1
@.str.1 = private unnamed_addr constant [9 x i8] c"data.txt\00", align 1
@.str.2 = private unnamed_addr constant [10 x i8] c"input.txt\00", align 1
@.str.3 = private unnamed_addr constant [10 x i8] c"file1.txt\00", align 1
@.str.4 = private unnamed_addr constant [10 x i8] c"file2.txt\00", align 1
@.str.5 = private unnamed_addr constant [9 x i8] c"path.txt\00", align 1

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z20test_class_typestatev() #0 personality ptr @__gxx_personality_v0 {
entry:
  %f = alloca %class.File, align 4
  %buf = alloca [64 x i8], align 1
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  call void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef @.str)
  %arraydecay = getelementptr inbounds [64 x i8], ptr %buf, i64 0, i64 0
  invoke void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef %arraydecay, i32 noundef 64)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %entry
  invoke void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %f)
          to label %invoke.cont1 unwind label %lpad

invoke.cont1:                                     ; preds = %invoke.cont
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  ret void

lpad:                                             ; preds = %invoke.cont, %entry
  %0 = landingpad { ptr, i32 }
          cleanup
  %1 = extractvalue { ptr, i32 } %0, 0
  store ptr %1, ptr %exn.slot, align 8
  %2 = extractvalue { ptr, i32 } %0, 1
  store i32 %2, ptr %ehselector.slot, align 4
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  br label %eh.resume

eh.resume:                                        ; preds = %lpad
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val2 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val2
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %this, ptr noundef %path) unnamed_addr #0 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %path.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  store ptr %path, ptr %path.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %0 = load ptr, ptr %path.addr, align 8
  %call = call i32 @open(ptr noundef %0, i32 noundef 0)
  %fd = getelementptr inbounds %class.File, ptr %this1, i32 0, i32 0
  store i32 %call, ptr %fd, align 4
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %this, ptr noundef %buf, i32 noundef %size) #0 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %buf.addr = alloca ptr, align 8
  %size.addr = alloca i32, align 4
  store ptr %this, ptr %this.addr, align 8
  store ptr %buf, ptr %buf.addr, align 8
  store i32 %size, ptr %size.addr, align 4
  %this1 = load ptr, ptr %this.addr, align 8
  %fd = getelementptr inbounds %class.File, ptr %this1, i32 0, i32 0
  %0 = load i32, ptr %fd, align 4
  %1 = load ptr, ptr %buf.addr, align 8
  %2 = load i32, ptr %size.addr, align 4
  %call = call i32 @read(i32 noundef %0, ptr noundef %1, i32 noundef %2)
  ret void
}

declare i32 @__gxx_personality_v0(...)

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %this) #0 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %fd = getelementptr inbounds %class.File, ptr %this1, i32 0, i32 0
  %0 = load i32, ptr %fd, align 4
  %call = call i32 @close(i32 noundef %0)
  ret void
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %this) unnamed_addr #1 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z26test_class_use_after_closev() #0 personality ptr @__gxx_personality_v0 {
entry:
  %f = alloca %class.File, align 4
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  %buf = alloca [32 x i8], align 1
  call void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef @.str.1)
  invoke void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %f)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %entry
  %arraydecay = getelementptr inbounds [32 x i8], ptr %buf, i64 0, i64 0
  invoke void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef %arraydecay, i32 noundef 32)
          to label %invoke.cont1 unwind label %lpad

invoke.cont1:                                     ; preds = %invoke.cont
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  ret void

lpad:                                             ; preds = %invoke.cont, %entry
  %0 = landingpad { ptr, i32 }
          cleanup
  %1 = extractvalue { ptr, i32 } %0, 0
  store ptr %1, ptr %exn.slot, align 8
  %2 = extractvalue { ptr, i32 } %0, 1
  store i32 %2, ptr %ehselector.slot, align 4
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  br label %eh.resume

eh.resume:                                        ; preds = %lpad
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val2 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val2
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z17test_method_chainv() #0 personality ptr @__gxx_personality_v0 {
entry:
  %f = alloca %class.File, align 4
  %dp = alloca %class.DataProcessor, align 8
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  call void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef @.str.2)
  %file = getelementptr inbounds %class.DataProcessor, ptr %dp, i32 0, i32 0
  store ptr %f, ptr %file, align 8
  invoke void @_ZN13DataProcessor7processEv(ptr noundef nonnull align 8 dereferenceable(8) %dp)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %entry
  invoke void @_ZN13DataProcessor6finishEv(ptr noundef nonnull align 8 dereferenceable(8) %dp)
          to label %invoke.cont1 unwind label %lpad

invoke.cont1:                                     ; preds = %invoke.cont
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  ret void

lpad:                                             ; preds = %invoke.cont, %entry
  %0 = landingpad { ptr, i32 }
          cleanup
  %1 = extractvalue { ptr, i32 } %0, 0
  store ptr %1, ptr %exn.slot, align 8
  %2 = extractvalue { ptr, i32 } %0, 1
  store i32 %2, ptr %ehselector.slot, align 4
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  br label %eh.resume

eh.resume:                                        ; preds = %lpad
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val2 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val2
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN13DataProcessor7processEv(ptr noundef nonnull align 8 dereferenceable(8) %this) #0 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  %buf = alloca [128 x i8], align 1
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %file = getelementptr inbounds %class.DataProcessor, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %file, align 8
  %arraydecay = getelementptr inbounds [128 x i8], ptr %buf, i64 0, i64 0
  call void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %0, ptr noundef %arraydecay, i32 noundef 128)
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN13DataProcessor6finishEv(ptr noundef nonnull align 8 dereferenceable(8) %this) #0 comdat align 2 {
entry:
  %this.addr = alloca ptr, align 8
  store ptr %this, ptr %this.addr, align 8
  %this1 = load ptr, ptr %this.addr, align 8
  %file = getelementptr inbounds %class.DataProcessor, ptr %this1, i32 0, i32 0
  %0 = load ptr, ptr %file, align 8
  call void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %0)
  ret void
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z21test_multiple_handlesv() #0 personality ptr @__gxx_personality_v0 {
entry:
  %f1 = alloca %class.File, align 4
  %f2 = alloca %class.File, align 4
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  %buf = alloca [32 x i8], align 1
  call void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f1, ptr noundef @.str.3)
  invoke void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f2, ptr noundef @.str.4)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %entry
  %arraydecay = getelementptr inbounds [32 x i8], ptr %buf, i64 0, i64 0
  invoke void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %f1, ptr noundef %arraydecay, i32 noundef 32)
          to label %invoke.cont2 unwind label %lpad1

invoke.cont2:                                     ; preds = %invoke.cont
  invoke void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %f1)
          to label %invoke.cont3 unwind label %lpad1

invoke.cont3:                                     ; preds = %invoke.cont2
  %arraydecay4 = getelementptr inbounds [32 x i8], ptr %buf, i64 0, i64 0
  invoke void @_ZN4File9read_dataEPci(ptr noundef nonnull align 4 dereferenceable(4) %f2, ptr noundef %arraydecay4, i32 noundef 32)
          to label %invoke.cont5 unwind label %lpad1

invoke.cont5:                                     ; preds = %invoke.cont3
  invoke void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %f2)
          to label %invoke.cont6 unwind label %lpad1

invoke.cont6:                                     ; preds = %invoke.cont5
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f2) #3
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f1) #3
  ret void

lpad:                                             ; preds = %entry
  %0 = landingpad { ptr, i32 }
          cleanup
  %1 = extractvalue { ptr, i32 } %0, 0
  store ptr %1, ptr %exn.slot, align 8
  %2 = extractvalue { ptr, i32 } %0, 1
  store i32 %2, ptr %ehselector.slot, align 4
  br label %ehcleanup

lpad1:                                            ; preds = %invoke.cont5, %invoke.cont3, %invoke.cont2, %invoke.cont
  %3 = landingpad { ptr, i32 }
          cleanup
  %4 = extractvalue { ptr, i32 } %3, 0
  store ptr %4, ptr %exn.slot, align 8
  %5 = extractvalue { ptr, i32 } %3, 1
  store i32 %5, ptr %ehselector.slot, align 4
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f2) #3
  br label %ehcleanup

ehcleanup:                                        ; preds = %lpad1, %lpad
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f1) #3
  br label %eh.resume

eh.resume:                                        ; preds = %ehcleanup
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val7 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val7
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z19test_path_sensitiveb(i1 noundef %cond) #0 personality ptr @__gxx_personality_v0 {
entry:
  %cond.addr = alloca i8, align 1
  %f = alloca %class.File, align 4
  %exn.slot = alloca ptr, align 8
  %ehselector.slot = alloca i32, align 4
  %frombool = zext i1 %cond to i8
  store i8 %frombool, ptr %cond.addr, align 1
  call void @_ZN4FileC2EPKc(ptr noundef nonnull align 4 dereferenceable(4) %f, ptr noundef @.str.5)
  %0 = load i8, ptr %cond.addr, align 1
  %tobool = trunc i8 %0 to i1
  br i1 %tobool, label %if.then, label %if.end

if.then:                                          ; preds = %entry
  invoke void @_ZN4File10close_fileEv(ptr noundef nonnull align 4 dereferenceable(4) %f)
          to label %invoke.cont unwind label %lpad

invoke.cont:                                      ; preds = %if.then
  br label %if.end

lpad:                                             ; preds = %if.then
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  store ptr %2, ptr %exn.slot, align 8
  %3 = extractvalue { ptr, i32 } %1, 1
  store i32 %3, ptr %ehselector.slot, align 4
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  br label %eh.resume

if.end:                                           ; preds = %invoke.cont, %entry
  call void @_ZN4FileD2Ev(ptr noundef nonnull align 4 dereferenceable(4) %f) #3
  ret void

eh.resume:                                        ; preds = %lpad
  %exn = load ptr, ptr %exn.slot, align 8
  %sel = load i32, ptr %ehselector.slot, align 4
  %lpad.val = insertvalue { ptr, i32 } poison, ptr %exn, 0
  %lpad.val1 = insertvalue { ptr, i32 } %lpad.val, i32 %sel, 1
  resume { ptr, i32 } %lpad.val1
}

declare i32 @open(ptr noundef, i32 noundef) #2

declare i32 @read(i32 noundef, ptr noundef, i32 noundef) #2

declare i32 @close(i32 noundef) #2

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
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
