; ModuleID = 'tests/fixtures/pta_verification/class_hierarchy.cpp'
source_filename = "tests/fixtures/pta_verification/class_hierarchy.cpp"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%class.Circle = type { %class.AbstractShape, double }
%class.AbstractShape = type { ptr }
%class.Rectangle = type { %class.AbstractShape, double, double }
%class.CRTPDerived = type { i8 }

$_ZN7DerivedC2Ev = comdat any

$_ZN6Level3C2Ev = comdat any

$_ZN3DogC2Ev = comdat any

$_ZN3CatC2Ev = comdat any

$_ZN4BirdC2Ev = comdat any

$_ZN8DocumentC2Ev = comdat any

$_ZN13DiamondBottomC1Ev = comdat any

$_ZN6CircleC2Ed = comdat any

$_ZN9RectangleC2Edd = comdat any

$_ZN17ConcreteObserverAC2Ev = comdat any

$_ZN17ConcreteObserverBC2Ev = comdat any

$_ZN8CRTPBaseI11CRTPDerivedE9interfaceEv = comdat any

$_ZN18ConcretePrototype1C2Ev = comdat any

$_ZN16CloneableDerivedC2Ev = comdat any

$_ZN4BaseC2Ev = comdat any

$_ZN7DerivedD2Ev = comdat any

$_ZN7DerivedD0Ev = comdat any

$_ZN7Derived3fooEv = comdat any

$_ZN4Base3barEv = comdat any

$_ZN4BaseD2Ev = comdat any

$_ZN4BaseD0Ev = comdat any

$_ZN4Base3fooEv = comdat any

$_ZN6Level2C2Ev = comdat any

$_ZN6Level3D2Ev = comdat any

$_ZN6Level3D0Ev = comdat any

$_ZN6Level36methodEv = comdat any

$_ZN6Level1C2Ev = comdat any

$_ZN6Level2D2Ev = comdat any

$_ZN6Level2D0Ev = comdat any

$_ZN6Level26methodEv = comdat any

$_ZN6Level1D2Ev = comdat any

$_ZN6Level1D0Ev = comdat any

$_ZN6Level16methodEv = comdat any

$_ZN6AnimalC2Ev = comdat any

$_ZN3DogD2Ev = comdat any

$_ZN3DogD0Ev = comdat any

$_ZN3Dog5speakEv = comdat any

$_ZN6AnimalD2Ev = comdat any

$_ZN6AnimalD0Ev = comdat any

$_ZN6Animal5speakEv = comdat any

$_ZN3CatD2Ev = comdat any

$_ZN3CatD0Ev = comdat any

$_ZN3Cat5speakEv = comdat any

$_ZN4BirdD2Ev = comdat any

$_ZN4BirdD0Ev = comdat any

$_ZN4Bird5speakEv = comdat any

$_ZN9PrintableC2Ev = comdat any

$_ZN12SerializableC2Ev = comdat any

$_ZN8DocumentD2Ev = comdat any

$_ZN8DocumentD0Ev = comdat any

$_ZN8Document5printEv = comdat any

$_ZN8Document9serializeEv = comdat any

$_ZThn8_N8DocumentD1Ev = comdat any

$_ZThn8_N8DocumentD0Ev = comdat any

$_ZThn8_N8Document9serializeEv = comdat any

$_ZN9PrintableD2Ev = comdat any

$_ZN9PrintableD0Ev = comdat any

$_ZN9Printable5printEv = comdat any

$_ZN12SerializableD2Ev = comdat any

$_ZN12SerializableD0Ev = comdat any

$_ZN12Serializable9serializeEv = comdat any

$_ZN11DiamondBaseC2Ev = comdat any

$_ZN11DiamondLeftC2Ev = comdat any

$_ZN12DiamondRightC2Ev = comdat any

$_ZN11DiamondLeftD1Ev = comdat any

$_ZN11DiamondLeftD0Ev = comdat any

$_ZN11DiamondLeft5valueEv = comdat any

$_ZN12DiamondRightD1Ev = comdat any

$_ZN12DiamondRightD0Ev = comdat any

$_ZN12DiamondRight5valueEv = comdat any

$_ZTv0_n24_N12DiamondRightD1Ev = comdat any

$_ZTv0_n24_N12DiamondRightD0Ev = comdat any

$_ZTv0_n32_N12DiamondRight5valueEv = comdat any

$_ZN13DiamondBottomD1Ev = comdat any

$_ZN13DiamondBottomD0Ev = comdat any

$_ZN13DiamondBottom5valueEv = comdat any

$_ZThn8_N13DiamondBottomD1Ev = comdat any

$_ZThn8_N13DiamondBottomD0Ev = comdat any

$_ZThn8_N13DiamondBottom5valueEv = comdat any

$_ZN11DiamondBaseD2Ev = comdat any

$_ZN11DiamondBaseD0Ev = comdat any

$_ZN11DiamondBase5valueEv = comdat any

$_ZN11DiamondLeftD2Ev = comdat any

$_ZTv0_n24_N11DiamondLeftD1Ev = comdat any

$_ZTv0_n24_N11DiamondLeftD0Ev = comdat any

$_ZTv0_n32_N11DiamondLeft5valueEv = comdat any

$_ZN12DiamondRightD2Ev = comdat any

$_ZN13DiamondBottomD2Ev = comdat any

$_ZTv0_n24_N13DiamondBottomD1Ev = comdat any

$_ZTv0_n24_N13DiamondBottomD0Ev = comdat any

$_ZTv0_n32_N13DiamondBottom5valueEv = comdat any

$_ZN13AbstractShapeC2Ev = comdat any

$_ZN6CircleD2Ev = comdat any

$_ZN6CircleD0Ev = comdat any

$_ZN6Circle4areaEv = comdat any

$_ZN6Circle9perimeterEv = comdat any

$_ZN13AbstractShapeD2Ev = comdat any

$_ZN13AbstractShapeD0Ev = comdat any

$_ZN9RectangleD2Ev = comdat any

$_ZN9RectangleD0Ev = comdat any

$_ZN9Rectangle4areaEv = comdat any

$_ZN9Rectangle9perimeterEv = comdat any

$_ZN9IObserverC2Ev = comdat any

$_ZN17ConcreteObserverAD2Ev = comdat any

$_ZN17ConcreteObserverAD0Ev = comdat any

$_ZN17ConcreteObserverA6updateEi = comdat any

$_ZN9IObserverD2Ev = comdat any

$_ZN9IObserverD0Ev = comdat any

$_ZN17ConcreteObserverBD2Ev = comdat any

$_ZN17ConcreteObserverBD0Ev = comdat any

$_ZN17ConcreteObserverB6updateEi = comdat any

$_ZN9PrototypeC2Ev = comdat any

$_ZN18ConcretePrototype1D2Ev = comdat any

$_ZN18ConcretePrototype1D0Ev = comdat any

$_ZN18ConcretePrototype15cloneEv = comdat any

$_ZN9PrototypeD2Ev = comdat any

$_ZN9PrototypeD0Ev = comdat any

$_ZN18ConcretePrototype1C2ERKS_ = comdat any

$_ZN9PrototypeC2ERKS_ = comdat any

$_ZN13CloneableBaseC2Ev = comdat any

$_ZN16CloneableDerivedD2Ev = comdat any

$_ZN16CloneableDerivedD0Ev = comdat any

$_ZN16CloneableDerived10clone_selfEv = comdat any

$_ZN13CloneableBaseD2Ev = comdat any

$_ZN13CloneableBaseD0Ev = comdat any

$_ZN13CloneableBase10clone_selfEv = comdat any

$_ZN13CloneableBaseC2ERKS_ = comdat any

$_ZN16CloneableDerivedC2ERKS_ = comdat any

$_ZN11CRTPDerived14implementationEv = comdat any

$_ZTV7Derived = comdat any

$_ZTS7Derived = comdat any

$_ZTS4Base = comdat any

$_ZTI4Base = comdat any

$_ZTI7Derived = comdat any

$_ZTV4Base = comdat any

$_ZTV6Level3 = comdat any

$_ZTS6Level3 = comdat any

$_ZTS6Level2 = comdat any

$_ZTS6Level1 = comdat any

$_ZTI6Level1 = comdat any

$_ZTI6Level2 = comdat any

$_ZTI6Level3 = comdat any

$_ZTV6Level2 = comdat any

$_ZTV6Level1 = comdat any

$_ZTV3Dog = comdat any

$_ZTS3Dog = comdat any

$_ZTS6Animal = comdat any

$_ZTI6Animal = comdat any

$_ZTI3Dog = comdat any

$_ZTV6Animal = comdat any

$_ZTV3Cat = comdat any

$_ZTS3Cat = comdat any

$_ZTI3Cat = comdat any

$_ZTV4Bird = comdat any

$_ZTS4Bird = comdat any

$_ZTI4Bird = comdat any

$_ZTV8Document = comdat any

$_ZTS8Document = comdat any

$_ZTS9Printable = comdat any

$_ZTI9Printable = comdat any

$_ZTS12Serializable = comdat any

$_ZTI12Serializable = comdat any

$_ZTI8Document = comdat any

$_ZTV9Printable = comdat any

$_ZTV12Serializable = comdat any

$_ZTV13DiamondBottom = comdat any

$_ZTT13DiamondBottom = comdat any

$_ZTC13DiamondBottom0_11DiamondLeft = comdat any

$_ZTS11DiamondLeft = comdat any

$_ZTS11DiamondBase = comdat any

$_ZTI11DiamondBase = comdat any

$_ZTI11DiamondLeft = comdat any

$_ZTC13DiamondBottom8_12DiamondRight = comdat any

$_ZTS12DiamondRight = comdat any

$_ZTI12DiamondRight = comdat any

$_ZTS13DiamondBottom = comdat any

$_ZTI13DiamondBottom = comdat any

$_ZTV11DiamondBase = comdat any

$_ZTV11DiamondLeft = comdat any

$_ZTT11DiamondLeft = comdat any

$_ZTV12DiamondRight = comdat any

$_ZTT12DiamondRight = comdat any

$_ZTV6Circle = comdat any

$_ZTS6Circle = comdat any

$_ZTS13AbstractShape = comdat any

$_ZTI13AbstractShape = comdat any

$_ZTI6Circle = comdat any

$_ZTV13AbstractShape = comdat any

$_ZTV9Rectangle = comdat any

$_ZTS9Rectangle = comdat any

$_ZTI9Rectangle = comdat any

$_ZTV17ConcreteObserverA = comdat any

$_ZTS17ConcreteObserverA = comdat any

$_ZTS9IObserver = comdat any

$_ZTI9IObserver = comdat any

$_ZTI17ConcreteObserverA = comdat any

$_ZTV9IObserver = comdat any

$_ZTV17ConcreteObserverB = comdat any

$_ZTS17ConcreteObserverB = comdat any

$_ZTI17ConcreteObserverB = comdat any

$_ZTV18ConcretePrototype1 = comdat any

$_ZTS18ConcretePrototype1 = comdat any

$_ZTS9Prototype = comdat any

$_ZTI9Prototype = comdat any

$_ZTI18ConcretePrototype1 = comdat any

$_ZTV9Prototype = comdat any

$_ZTV16CloneableDerived = comdat any

$_ZTS16CloneableDerived = comdat any

$_ZTS13CloneableBase = comdat any

$_ZTI13CloneableBase = comdat any

$_ZTI16CloneableDerived = comdat any

$_ZTV13CloneableBase = comdat any

@_ZTV7Derived = linkonce_odr dso_local unnamed_addr constant { [6 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI7Derived, ptr @_ZN7DerivedD2Ev, ptr @_ZN7DerivedD0Ev, ptr @_ZN7Derived3fooEv, ptr @_ZN4Base3barEv] }, comdat, align 8
@_ZTVN10__cxxabiv120__si_class_type_infoE = external global [0 x ptr]
@_ZTS7Derived = linkonce_odr dso_local constant [9 x i8] c"7Derived\00", comdat, align 1
@_ZTVN10__cxxabiv117__class_type_infoE = external global [0 x ptr]
@_ZTS4Base = linkonce_odr dso_local constant [6 x i8] c"4Base\00", comdat, align 1
@_ZTI4Base = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS4Base }, comdat, align 8
@_ZTI7Derived = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS7Derived, ptr @_ZTI4Base }, comdat, align 8
@_ZTV4Base = linkonce_odr dso_local unnamed_addr constant { [6 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI4Base, ptr @_ZN4BaseD2Ev, ptr @_ZN4BaseD0Ev, ptr @_ZN4Base3fooEv, ptr @_ZN4Base3barEv] }, comdat, align 8
@_ZTV6Level3 = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI6Level3, ptr @_ZN6Level3D2Ev, ptr @_ZN6Level3D0Ev, ptr @_ZN6Level36methodEv] }, comdat, align 8
@_ZTS6Level3 = linkonce_odr dso_local constant [8 x i8] c"6Level3\00", comdat, align 1
@_ZTS6Level2 = linkonce_odr dso_local constant [8 x i8] c"6Level2\00", comdat, align 1
@_ZTS6Level1 = linkonce_odr dso_local constant [8 x i8] c"6Level1\00", comdat, align 1
@_ZTI6Level1 = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS6Level1 }, comdat, align 8
@_ZTI6Level2 = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS6Level2, ptr @_ZTI6Level1 }, comdat, align 8
@_ZTI6Level3 = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS6Level3, ptr @_ZTI6Level2 }, comdat, align 8
@_ZTV6Level2 = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI6Level2, ptr @_ZN6Level2D2Ev, ptr @_ZN6Level2D0Ev, ptr @_ZN6Level26methodEv] }, comdat, align 8
@_ZTV6Level1 = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI6Level1, ptr @_ZN6Level1D2Ev, ptr @_ZN6Level1D0Ev, ptr @_ZN6Level16methodEv] }, comdat, align 8
@_ZTV3Dog = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI3Dog, ptr @_ZN3DogD2Ev, ptr @_ZN3DogD0Ev, ptr @_ZN3Dog5speakEv] }, comdat, align 8
@_ZTS3Dog = linkonce_odr dso_local constant [5 x i8] c"3Dog\00", comdat, align 1
@_ZTS6Animal = linkonce_odr dso_local constant [8 x i8] c"6Animal\00", comdat, align 1
@_ZTI6Animal = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS6Animal }, comdat, align 8
@_ZTI3Dog = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS3Dog, ptr @_ZTI6Animal }, comdat, align 8
@_ZTV6Animal = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI6Animal, ptr @_ZN6AnimalD2Ev, ptr @_ZN6AnimalD0Ev, ptr @_ZN6Animal5speakEv] }, comdat, align 8
@_ZTV3Cat = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI3Cat, ptr @_ZN3CatD2Ev, ptr @_ZN3CatD0Ev, ptr @_ZN3Cat5speakEv] }, comdat, align 8
@_ZTS3Cat = linkonce_odr dso_local constant [5 x i8] c"3Cat\00", comdat, align 1
@_ZTI3Cat = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS3Cat, ptr @_ZTI6Animal }, comdat, align 8
@_ZTV4Bird = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI4Bird, ptr @_ZN4BirdD2Ev, ptr @_ZN4BirdD0Ev, ptr @_ZN4Bird5speakEv] }, comdat, align 8
@_ZTS4Bird = linkonce_odr dso_local constant [6 x i8] c"4Bird\00", comdat, align 1
@_ZTI4Bird = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS4Bird, ptr @_ZTI6Animal }, comdat, align 8
@_ZTV8Document = linkonce_odr dso_local unnamed_addr constant { [6 x ptr], [5 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI8Document, ptr @_ZN8DocumentD2Ev, ptr @_ZN8DocumentD0Ev, ptr @_ZN8Document5printEv, ptr @_ZN8Document9serializeEv], [5 x ptr] [ptr inttoptr (i64 -8 to ptr), ptr @_ZTI8Document, ptr @_ZThn8_N8DocumentD1Ev, ptr @_ZThn8_N8DocumentD0Ev, ptr @_ZThn8_N8Document9serializeEv] }, comdat, align 8
@_ZTVN10__cxxabiv121__vmi_class_type_infoE = external global [0 x ptr]
@_ZTS8Document = linkonce_odr dso_local constant [10 x i8] c"8Document\00", comdat, align 1
@_ZTS9Printable = linkonce_odr dso_local constant [11 x i8] c"9Printable\00", comdat, align 1
@_ZTI9Printable = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS9Printable }, comdat, align 8
@_ZTS12Serializable = linkonce_odr dso_local constant [15 x i8] c"12Serializable\00", comdat, align 1
@_ZTI12Serializable = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS12Serializable }, comdat, align 8
@_ZTI8Document = linkonce_odr dso_local constant { ptr, ptr, i32, i32, ptr, i64, ptr, i64 } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv121__vmi_class_type_infoE, i64 2), ptr @_ZTS8Document, i32 0, i32 2, ptr @_ZTI9Printable, i64 2, ptr @_ZTI12Serializable, i64 2050 }, comdat, align 8
@_ZTV9Printable = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI9Printable, ptr @_ZN9PrintableD2Ev, ptr @_ZN9PrintableD0Ev, ptr @_ZN9Printable5printEv] }, comdat, align 8
@_ZTV12Serializable = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI12Serializable, ptr @_ZN12SerializableD2Ev, ptr @_ZN12SerializableD0Ev, ptr @_ZN12Serializable9serializeEv] }, comdat, align 8
@_ZTV13DiamondBottom = linkonce_odr dso_local unnamed_addr constant { [8 x ptr], [8 x ptr] } { [8 x ptr] [ptr null, ptr null, ptr null, ptr null, ptr @_ZTI13DiamondBottom, ptr @_ZN13DiamondBottomD1Ev, ptr @_ZN13DiamondBottomD0Ev, ptr @_ZN13DiamondBottom5valueEv], [8 x ptr] [ptr inttoptr (i64 -8 to ptr), ptr inttoptr (i64 -8 to ptr), ptr inttoptr (i64 -8 to ptr), ptr inttoptr (i64 -8 to ptr), ptr @_ZTI13DiamondBottom, ptr @_ZThn8_N13DiamondBottomD1Ev, ptr @_ZThn8_N13DiamondBottomD0Ev, ptr @_ZThn8_N13DiamondBottom5valueEv] }, comdat, align 8
@_ZTT13DiamondBottom = linkonce_odr dso_local unnamed_addr constant [7 x ptr] [ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTC13DiamondBottom0_11DiamondLeft, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTC13DiamondBottom0_11DiamondLeft, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr], [7 x ptr] }, ptr @_ZTC13DiamondBottom8_12DiamondRight, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr], [7 x ptr] }, ptr @_ZTC13DiamondBottom8_12DiamondRight, i32 0, inrange i32 1, i32 4), ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 1, i32 5)], comdat, align 8
@_ZTC13DiamondBottom0_11DiamondLeft = linkonce_odr dso_local unnamed_addr constant { [8 x ptr] } { [8 x ptr] [ptr null, ptr null, ptr null, ptr null, ptr @_ZTI11DiamondLeft, ptr @_ZN11DiamondLeftD1Ev, ptr @_ZN11DiamondLeftD0Ev, ptr @_ZN11DiamondLeft5valueEv] }, comdat, align 8
@_ZTS11DiamondLeft = linkonce_odr dso_local constant [14 x i8] c"11DiamondLeft\00", comdat, align 1
@_ZTS11DiamondBase = linkonce_odr dso_local constant [14 x i8] c"11DiamondBase\00", comdat, align 1
@_ZTI11DiamondBase = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS11DiamondBase }, comdat, align 8
@_ZTI11DiamondLeft = linkonce_odr dso_local constant { ptr, ptr, i32, i32, ptr, i64 } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv121__vmi_class_type_infoE, i64 2), ptr @_ZTS11DiamondLeft, i32 0, i32 1, ptr @_ZTI11DiamondBase, i64 -10237 }, comdat, align 8
@_ZTC13DiamondBottom8_12DiamondRight = linkonce_odr dso_local unnamed_addr constant { [8 x ptr], [7 x ptr] } { [8 x ptr] [ptr inttoptr (i64 -8 to ptr), ptr null, ptr null, ptr null, ptr @_ZTI12DiamondRight, ptr @_ZN12DiamondRightD1Ev, ptr @_ZN12DiamondRightD0Ev, ptr @_ZN12DiamondRight5valueEv], [7 x ptr] [ptr inttoptr (i64 8 to ptr), ptr inttoptr (i64 8 to ptr), ptr inttoptr (i64 8 to ptr), ptr @_ZTI12DiamondRight, ptr @_ZTv0_n24_N12DiamondRightD1Ev, ptr @_ZTv0_n24_N12DiamondRightD0Ev, ptr @_ZTv0_n32_N12DiamondRight5valueEv] }, comdat, align 8
@_ZTS12DiamondRight = linkonce_odr dso_local constant [15 x i8] c"12DiamondRight\00", comdat, align 1
@_ZTI12DiamondRight = linkonce_odr dso_local constant { ptr, ptr, i32, i32, ptr, i64 } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv121__vmi_class_type_infoE, i64 2), ptr @_ZTS12DiamondRight, i32 0, i32 1, ptr @_ZTI11DiamondBase, i64 -10237 }, comdat, align 8
@_ZTS13DiamondBottom = linkonce_odr dso_local constant [16 x i8] c"13DiamondBottom\00", comdat, align 1
@_ZTI13DiamondBottom = linkonce_odr dso_local constant { ptr, ptr, i32, i32, ptr, i64, ptr, i64 } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv121__vmi_class_type_infoE, i64 2), ptr @_ZTS13DiamondBottom, i32 2, i32 2, ptr @_ZTI11DiamondLeft, i64 2, ptr @_ZTI12DiamondRight, i64 2050 }, comdat, align 8
@_ZTV11DiamondBase = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI11DiamondBase, ptr @_ZN11DiamondBaseD2Ev, ptr @_ZN11DiamondBaseD0Ev, ptr @_ZN11DiamondBase5valueEv] }, comdat, align 8
@_ZTV11DiamondLeft = linkonce_odr dso_local unnamed_addr constant { [8 x ptr] } { [8 x ptr] [ptr null, ptr null, ptr null, ptr null, ptr @_ZTI11DiamondLeft, ptr @_ZN11DiamondLeftD1Ev, ptr @_ZN11DiamondLeftD0Ev, ptr @_ZN11DiamondLeft5valueEv] }, comdat, align 8
@_ZTT11DiamondLeft = linkonce_odr dso_local unnamed_addr constant [2 x ptr] [ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTV11DiamondLeft, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTV11DiamondLeft, i32 0, inrange i32 0, i32 5)], comdat, align 8
@_ZTV12DiamondRight = linkonce_odr dso_local unnamed_addr constant { [8 x ptr] } { [8 x ptr] [ptr null, ptr null, ptr null, ptr null, ptr @_ZTI12DiamondRight, ptr @_ZN12DiamondRightD1Ev, ptr @_ZN12DiamondRightD0Ev, ptr @_ZN12DiamondRight5valueEv] }, comdat, align 8
@_ZTT12DiamondRight = linkonce_odr dso_local unnamed_addr constant [2 x ptr] [ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTV12DiamondRight, i32 0, inrange i32 0, i32 5), ptr getelementptr inbounds ({ [8 x ptr] }, ptr @_ZTV12DiamondRight, i32 0, inrange i32 0, i32 5)], comdat, align 8
@_ZTV6Circle = linkonce_odr dso_local unnamed_addr constant { [6 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI6Circle, ptr @_ZN6CircleD2Ev, ptr @_ZN6CircleD0Ev, ptr @_ZN6Circle4areaEv, ptr @_ZN6Circle9perimeterEv] }, comdat, align 8
@_ZTS6Circle = linkonce_odr dso_local constant [8 x i8] c"6Circle\00", comdat, align 1
@_ZTS13AbstractShape = linkonce_odr dso_local constant [16 x i8] c"13AbstractShape\00", comdat, align 1
@_ZTI13AbstractShape = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS13AbstractShape }, comdat, align 8
@_ZTI6Circle = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS6Circle, ptr @_ZTI13AbstractShape }, comdat, align 8
@_ZTV13AbstractShape = linkonce_odr dso_local unnamed_addr constant { [6 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI13AbstractShape, ptr @_ZN13AbstractShapeD2Ev, ptr @_ZN13AbstractShapeD0Ev, ptr @__cxa_pure_virtual, ptr @__cxa_pure_virtual] }, comdat, align 8
@_ZTV9Rectangle = linkonce_odr dso_local unnamed_addr constant { [6 x ptr] } { [6 x ptr] [ptr null, ptr @_ZTI9Rectangle, ptr @_ZN9RectangleD2Ev, ptr @_ZN9RectangleD0Ev, ptr @_ZN9Rectangle4areaEv, ptr @_ZN9Rectangle9perimeterEv] }, comdat, align 8
@_ZTS9Rectangle = linkonce_odr dso_local constant [11 x i8] c"9Rectangle\00", comdat, align 1
@_ZTI9Rectangle = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS9Rectangle, ptr @_ZTI13AbstractShape }, comdat, align 8
@_ZTV17ConcreteObserverA = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI17ConcreteObserverA, ptr @_ZN17ConcreteObserverAD2Ev, ptr @_ZN17ConcreteObserverAD0Ev, ptr @_ZN17ConcreteObserverA6updateEi] }, comdat, align 8
@_ZTS17ConcreteObserverA = linkonce_odr dso_local constant [20 x i8] c"17ConcreteObserverA\00", comdat, align 1
@_ZTS9IObserver = linkonce_odr dso_local constant [11 x i8] c"9IObserver\00", comdat, align 1
@_ZTI9IObserver = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS9IObserver }, comdat, align 8
@_ZTI17ConcreteObserverA = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS17ConcreteObserverA, ptr @_ZTI9IObserver }, comdat, align 8
@_ZTV9IObserver = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI9IObserver, ptr @_ZN9IObserverD2Ev, ptr @_ZN9IObserverD0Ev, ptr @__cxa_pure_virtual] }, comdat, align 8
@_ZTV17ConcreteObserverB = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI17ConcreteObserverB, ptr @_ZN17ConcreteObserverBD2Ev, ptr @_ZN17ConcreteObserverBD0Ev, ptr @_ZN17ConcreteObserverB6updateEi] }, comdat, align 8
@_ZTS17ConcreteObserverB = linkonce_odr dso_local constant [20 x i8] c"17ConcreteObserverB\00", comdat, align 1
@_ZTI17ConcreteObserverB = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS17ConcreteObserverB, ptr @_ZTI9IObserver }, comdat, align 8
@_ZTV18ConcretePrototype1 = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI18ConcretePrototype1, ptr @_ZN18ConcretePrototype1D2Ev, ptr @_ZN18ConcretePrototype1D0Ev, ptr @_ZN18ConcretePrototype15cloneEv] }, comdat, align 8
@_ZTS18ConcretePrototype1 = linkonce_odr dso_local constant [21 x i8] c"18ConcretePrototype1\00", comdat, align 1
@_ZTS9Prototype = linkonce_odr dso_local constant [11 x i8] c"9Prototype\00", comdat, align 1
@_ZTI9Prototype = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS9Prototype }, comdat, align 8
@_ZTI18ConcretePrototype1 = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS18ConcretePrototype1, ptr @_ZTI9Prototype }, comdat, align 8
@_ZTV9Prototype = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI9Prototype, ptr @_ZN9PrototypeD2Ev, ptr @_ZN9PrototypeD0Ev, ptr @__cxa_pure_virtual] }, comdat, align 8
@_ZTV16CloneableDerived = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI16CloneableDerived, ptr @_ZN16CloneableDerivedD2Ev, ptr @_ZN16CloneableDerivedD0Ev, ptr @_ZN16CloneableDerived10clone_selfEv] }, comdat, align 8
@_ZTS16CloneableDerived = linkonce_odr dso_local constant [19 x i8] c"16CloneableDerived\00", comdat, align 1
@_ZTS13CloneableBase = linkonce_odr dso_local constant [16 x i8] c"13CloneableBase\00", comdat, align 1
@_ZTI13CloneableBase = linkonce_odr dso_local constant { ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv117__class_type_infoE, i64 2), ptr @_ZTS13CloneableBase }, comdat, align 8
@_ZTI16CloneableDerived = linkonce_odr dso_local constant { ptr, ptr, ptr } { ptr getelementptr inbounds (ptr, ptr @_ZTVN10__cxxabiv120__si_class_type_infoE, i64 2), ptr @_ZTS16CloneableDerived, ptr @_ZTI13CloneableBase }, comdat, align 8
@_ZTV13CloneableBase = linkonce_odr dso_local unnamed_addr constant { [5 x ptr] } { [5 x ptr] [ptr null, ptr @_ZTI13CloneableBase, ptr @_ZN13CloneableBaseD2Ev, ptr @_ZN13CloneableBaseD0Ev, ptr @_ZN13CloneableBase10clone_selfEv] }, comdat, align 8

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z23test_single_inheritancev() #0 !dbg !239 {
  %1 = alloca ptr, align 8
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !241, metadata !DIExpression()), !dbg !256
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !257, !heapallocsite !258
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !265
  call void @_ZN7DerivedC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !265
  store ptr %3, ptr %1, align 8, !dbg !256
  call void @llvm.dbg.declare(metadata ptr %2, metadata !266, metadata !DIExpression()), !dbg !267
  %4 = load ptr, ptr %1, align 8, !dbg !268
  %5 = load ptr, ptr %4, align 8, !dbg !269
  %6 = getelementptr inbounds ptr, ptr %5, i64 2, !dbg !269
  %7 = load ptr, ptr %6, align 8, !dbg !269
  %8 = call noundef i32 %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !269
  store i32 %8, ptr %2, align 4, !dbg !267
  %9 = load ptr, ptr %1, align 8, !dbg !270
  %10 = icmp eq ptr %9, null, !dbg !271
  br i1 %10, label %15, label %11, !dbg !271

11:                                               ; preds = %0
  %12 = load ptr, ptr %9, align 8, !dbg !271
  %13 = getelementptr inbounds ptr, ptr %12, i64 1, !dbg !271
  %14 = load ptr, ptr %13, align 8, !dbg !271
  call void %14(ptr noundef nonnull align 8 dereferenceable(8) %9) #11, !dbg !271
  br label %15, !dbg !271

15:                                               ; preds = %11, %0
  ret void, !dbg !272
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: nobuiltin allocsize(0)
declare noundef nonnull ptr @_Znwm(i64 noundef) #2

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #3

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN7DerivedC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !273 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !277, metadata !DIExpression()), !dbg !279
  %3 = load ptr, ptr %2, align 8
  call void @_ZN4BaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !280
  store ptr getelementptr inbounds ({ [6 x ptr] }, ptr @_ZTV7Derived, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !280
  ret void, !dbg !280
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z21test_deep_inheritancev() #0 !dbg !281 {
  %1 = alloca ptr, align 8
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !282, metadata !DIExpression()), !dbg !294
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !295, !heapallocsite !296
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !310
  call void @_ZN6Level3C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !310
  store ptr %3, ptr %1, align 8, !dbg !294
  call void @llvm.dbg.declare(metadata ptr %2, metadata !311, metadata !DIExpression()), !dbg !312
  %4 = load ptr, ptr %1, align 8, !dbg !313
  %5 = load ptr, ptr %4, align 8, !dbg !314
  %6 = getelementptr inbounds ptr, ptr %5, i64 2, !dbg !314
  %7 = load ptr, ptr %6, align 8, !dbg !314
  %8 = call noundef i32 %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !314
  store i32 %8, ptr %2, align 4, !dbg !312
  %9 = load ptr, ptr %1, align 8, !dbg !315
  %10 = icmp eq ptr %9, null, !dbg !316
  br i1 %10, label %15, label %11, !dbg !316

11:                                               ; preds = %0
  %12 = load ptr, ptr %9, align 8, !dbg !316
  %13 = getelementptr inbounds ptr, ptr %12, i64 1, !dbg !316
  %14 = load ptr, ptr %13, align 8, !dbg !316
  call void %14(ptr noundef nonnull align 8 dereferenceable(8) %9) #11, !dbg !316
  br label %15, !dbg !316

15:                                               ; preds = %11, %0
  ret void, !dbg !317
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level3C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !318 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !322, metadata !DIExpression()), !dbg !324
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level2C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !325
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV6Level3, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !325
  ret void, !dbg !325
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z21test_multiple_derivedi(i32 noundef %0) #0 !dbg !326 {
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  store i32 %0, ptr %2, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !327, metadata !DIExpression()), !dbg !328
  call void @llvm.dbg.declare(metadata ptr %3, metadata !329, metadata !DIExpression()), !dbg !339
  %4 = load i32, ptr %2, align 4, !dbg !340
  switch i32 %4, label %9 [
    i32 0, label %5
    i32 1, label %7
  ], !dbg !341

5:                                                ; preds = %1
  %6 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !342, !heapallocsite !344
  call void @llvm.memset.p0.i64(ptr align 8 %6, i8 0, i64 8, i1 false), !dbg !351
  call void @_ZN3DogC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %6) #11, !dbg !351
  store ptr %6, ptr %3, align 8, !dbg !352
  br label %11, !dbg !353

7:                                                ; preds = %1
  %8 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !354, !heapallocsite !355
  call void @llvm.memset.p0.i64(ptr align 8 %8, i8 0, i64 8, i1 false), !dbg !362
  call void @_ZN3CatC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %8) #11, !dbg !362
  store ptr %8, ptr %3, align 8, !dbg !363
  br label %11, !dbg !364

9:                                                ; preds = %1
  %10 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !365, !heapallocsite !366
  call void @llvm.memset.p0.i64(ptr align 8 %10, i8 0, i64 8, i1 false), !dbg !373
  call void @_ZN4BirdC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %10) #11, !dbg !373
  store ptr %10, ptr %3, align 8, !dbg !374
  br label %11, !dbg !375

11:                                               ; preds = %9, %7, %5
  %12 = load ptr, ptr %3, align 8, !dbg !376
  %13 = load ptr, ptr %12, align 8, !dbg !377
  %14 = getelementptr inbounds ptr, ptr %13, i64 2, !dbg !377
  %15 = load ptr, ptr %14, align 8, !dbg !377
  call void %15(ptr noundef nonnull align 8 dereferenceable(8) %12), !dbg !377
  %16 = load ptr, ptr %3, align 8, !dbg !378
  %17 = icmp eq ptr %16, null, !dbg !379
  br i1 %17, label %22, label %18, !dbg !379

18:                                               ; preds = %11
  %19 = load ptr, ptr %16, align 8, !dbg !379
  %20 = getelementptr inbounds ptr, ptr %19, i64 1, !dbg !379
  %21 = load ptr, ptr %20, align 8, !dbg !379
  call void %21(ptr noundef nonnull align 8 dereferenceable(8) %16) #11, !dbg !379
  br label %22, !dbg !379

22:                                               ; preds = %18, %11
  ret void, !dbg !380
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !381 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !383, metadata !DIExpression()), !dbg !385
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !386
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV3Dog, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !386
  ret void, !dbg !386
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3CatC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !387 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !389, metadata !DIExpression()), !dbg !391
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !392
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV3Cat, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !392
  ret void, !dbg !392
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BirdC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !393 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !395, metadata !DIExpression()), !dbg !397
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !398
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV4Bird, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !398
  ret void, !dbg !398
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z25test_multiple_inheritancev() #0 !dbg !399 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !400, metadata !DIExpression()), !dbg !410
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #10, !dbg !411, !heapallocsite !412
  call void @llvm.memset.p0.i64(ptr align 16 %3, i8 0, i64 16, i1 false), !dbg !429
  call void @_ZN8DocumentC2Ev(ptr noundef nonnull align 8 dereferenceable(16) %3) #11, !dbg !429
  store ptr %3, ptr %1, align 8, !dbg !410
  %4 = load ptr, ptr %1, align 8, !dbg !430
  %5 = load ptr, ptr %4, align 8, !dbg !431
  %6 = getelementptr inbounds ptr, ptr %5, i64 2, !dbg !431
  %7 = load ptr, ptr %6, align 8, !dbg !431
  call void %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !431
  %8 = load ptr, ptr %1, align 8, !dbg !432
  %9 = icmp eq ptr %8, null, !dbg !433
  br i1 %9, label %14, label %10, !dbg !433

10:                                               ; preds = %0
  %11 = load ptr, ptr %8, align 8, !dbg !433
  %12 = getelementptr inbounds ptr, ptr %11, i64 1, !dbg !433
  %13 = load ptr, ptr %12, align 8, !dbg !433
  call void %13(ptr noundef nonnull align 8 dereferenceable(8) %8) #11, !dbg !433
  br label %14, !dbg !433

14:                                               ; preds = %10, %0
  call void @llvm.dbg.declare(metadata ptr %2, metadata !434, metadata !DIExpression()), !dbg !436
  %15 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #10, !dbg !437, !heapallocsite !412
  call void @llvm.memset.p0.i64(ptr align 16 %15, i8 0, i64 16, i1 false), !dbg !438
  call void @_ZN8DocumentC2Ev(ptr noundef nonnull align 8 dereferenceable(16) %15) #11, !dbg !438
  %16 = icmp eq ptr %15, null, !dbg !437
  br i1 %16, label %19, label %17, !dbg !437

17:                                               ; preds = %14
  %18 = getelementptr inbounds i8, ptr %15, i64 8, !dbg !437
  br label %19, !dbg !437

19:                                               ; preds = %17, %14
  %20 = phi ptr [ %18, %17 ], [ null, %14 ], !dbg !437
  store ptr %20, ptr %2, align 8, !dbg !436
  %21 = load ptr, ptr %2, align 8, !dbg !439
  %22 = load ptr, ptr %21, align 8, !dbg !440
  %23 = getelementptr inbounds ptr, ptr %22, i64 2, !dbg !440
  %24 = load ptr, ptr %23, align 8, !dbg !440
  call void %24(ptr noundef nonnull align 8 dereferenceable(8) %21), !dbg !440
  %25 = load ptr, ptr %2, align 8, !dbg !441
  %26 = icmp eq ptr %25, null, !dbg !442
  br i1 %26, label %31, label %27, !dbg !442

27:                                               ; preds = %19
  %28 = load ptr, ptr %25, align 8, !dbg !442
  %29 = getelementptr inbounds ptr, ptr %28, i64 1, !dbg !442
  %30 = load ptr, ptr %29, align 8, !dbg !442
  call void %30(ptr noundef nonnull align 8 dereferenceable(8) %25) #11, !dbg !442
  br label %31, !dbg !442

31:                                               ; preds = %27, %19
  ret void, !dbg !443
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8DocumentC2Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !444 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !446, metadata !DIExpression()), !dbg !448
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9PrintableC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !449
  %4 = getelementptr inbounds i8, ptr %3, i64 8, !dbg !449
  call void @_ZN12SerializableC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %4) #11, !dbg !449
  store ptr getelementptr inbounds ({ [6 x ptr], [5 x ptr] }, ptr @_ZTV8Document, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !449
  %5 = getelementptr inbounds i8, ptr %3, i64 8, !dbg !449
  store ptr getelementptr inbounds ({ [6 x ptr], [5 x ptr] }, ptr @_ZTV8Document, i32 0, inrange i32 1, i32 2), ptr %5, align 8, !dbg !449
  ret void, !dbg !449
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z24test_diamond_inheritancev() #0 !dbg !450 {
  %1 = alloca ptr, align 8
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !451, metadata !DIExpression()), !dbg !463
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #10, !dbg !464, !heapallocsite !465
  call void @llvm.memset.p0.i64(ptr align 16 %3, i8 0, i64 16, i1 false), !dbg !487
  call void @_ZN13DiamondBottomC1Ev(ptr noundef nonnull align 8 dereferenceable(16) %3) #11, !dbg !487
  %4 = icmp eq ptr %3, null, !dbg !464
  br i1 %4, label %10, label %5, !dbg !464

5:                                                ; preds = %0
  %6 = load ptr, ptr %3, align 8, !dbg !464
  %7 = getelementptr i8, ptr %6, i64 -40, !dbg !464
  %8 = load i64, ptr %7, align 8, !dbg !464
  %9 = getelementptr inbounds i8, ptr %3, i64 %8, !dbg !464
  br label %10, !dbg !464

10:                                               ; preds = %5, %0
  %11 = phi ptr [ %9, %5 ], [ null, %0 ], !dbg !464
  store ptr %11, ptr %1, align 8, !dbg !463
  call void @llvm.dbg.declare(metadata ptr %2, metadata !488, metadata !DIExpression()), !dbg !489
  %12 = load ptr, ptr %1, align 8, !dbg !490
  %13 = load ptr, ptr %12, align 8, !dbg !491
  %14 = getelementptr inbounds ptr, ptr %13, i64 2, !dbg !491
  %15 = load ptr, ptr %14, align 8, !dbg !491
  %16 = call noundef i32 %15(ptr noundef nonnull align 8 dereferenceable(8) %12), !dbg !491
  store i32 %16, ptr %2, align 4, !dbg !489
  %17 = load ptr, ptr %1, align 8, !dbg !492
  %18 = icmp eq ptr %17, null, !dbg !493
  br i1 %18, label %23, label %19, !dbg !493

19:                                               ; preds = %10
  %20 = load ptr, ptr %17, align 8, !dbg !493
  %21 = getelementptr inbounds ptr, ptr %20, i64 1, !dbg !493
  %22 = load ptr, ptr %21, align 8, !dbg !493
  call void %22(ptr noundef nonnull align 8 dereferenceable(8) %17) #11, !dbg !493
  br label %23, !dbg !493

23:                                               ; preds = %19, %10
  ret void, !dbg !494
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13DiamondBottomC1Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !495 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !499, metadata !DIExpression()), !dbg !501
  %3 = load ptr, ptr %2, align 8
  call void @_ZN11DiamondBaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !502
  call void @_ZN11DiamondLeftC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3, ptr noundef getelementptr inbounds ([7 x ptr], ptr @_ZTT13DiamondBottom, i64 0, i64 1)) #11, !dbg !502
  %4 = getelementptr inbounds i8, ptr %3, i64 8, !dbg !502
  call void @_ZN12DiamondRightC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %4, ptr noundef getelementptr inbounds ([7 x ptr], ptr @_ZTT13DiamondBottom, i64 0, i64 3)) #11, !dbg !502
  store ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 0, i32 5), ptr %3, align 8, !dbg !502
  store ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 0, i32 5), ptr %3, align 8, !dbg !502
  %5 = getelementptr inbounds i8, ptr %3, i64 8, !dbg !502
  store ptr getelementptr inbounds ({ [8 x ptr], [8 x ptr] }, ptr @_ZTV13DiamondBottom, i32 0, inrange i32 1, i32 5), ptr %5, align 8, !dbg !502
  ret void, !dbg !502
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z19test_abstract_classv() #0 personality ptr @__gxx_personality_v0 !dbg !503 {
  %1 = alloca [2 x ptr], align 8
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca double, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !504, metadata !DIExpression()), !dbg !520
  %6 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 16) #10, !dbg !521, !heapallocsite !522
  invoke void @_ZN6CircleC2Ed(ptr noundef nonnull align 8 dereferenceable(16) %6, double noundef 5.000000e+00)
          to label %7 unwind label %27, !dbg !534

7:                                                ; preds = %0
  %8 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 0, !dbg !535
  store ptr %6, ptr %8, align 8, !dbg !536
  %9 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 24) #10, !dbg !537, !heapallocsite !538
  invoke void @_ZN9RectangleC2Edd(ptr noundef nonnull align 8 dereferenceable(24) %9, double noundef 3.000000e+00, double noundef 4.000000e+00)
          to label %10 unwind label %31, !dbg !551

10:                                               ; preds = %7
  %11 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 1, !dbg !552
  store ptr %9, ptr %11, align 8, !dbg !553
  call void @llvm.dbg.declare(metadata ptr %4, metadata !554, metadata !DIExpression()), !dbg !556
  store i32 0, ptr %4, align 4, !dbg !556
  br label %12, !dbg !557

12:                                               ; preds = %24, %10
  %13 = load i32, ptr %4, align 4, !dbg !558
  %14 = icmp slt i32 %13, 2, !dbg !560
  br i1 %14, label %15, label %35, !dbg !561

15:                                               ; preds = %12
  call void @llvm.dbg.declare(metadata ptr %5, metadata !562, metadata !DIExpression()), !dbg !564
  %16 = load i32, ptr %4, align 4, !dbg !565
  %17 = sext i32 %16 to i64, !dbg !566
  %18 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 %17, !dbg !566
  %19 = load ptr, ptr %18, align 8, !dbg !566
  %20 = load ptr, ptr %19, align 8, !dbg !567
  %21 = getelementptr inbounds ptr, ptr %20, i64 2, !dbg !567
  %22 = load ptr, ptr %21, align 8, !dbg !567
  %23 = call noundef double %22(ptr noundef nonnull align 8 dereferenceable(8) %19), !dbg !567
  store double %23, ptr %5, align 8, !dbg !564
  br label %24, !dbg !568

24:                                               ; preds = %15
  %25 = load i32, ptr %4, align 4, !dbg !569
  %26 = add nsw i32 %25, 1, !dbg !569
  store i32 %26, ptr %4, align 4, !dbg !569
  br label %12, !dbg !570, !llvm.loop !571

27:                                               ; preds = %0
  %28 = landingpad { ptr, i32 }
          cleanup, !dbg !574
  %29 = extractvalue { ptr, i32 } %28, 0, !dbg !574
  store ptr %29, ptr %2, align 8, !dbg !574
  %30 = extractvalue { ptr, i32 } %28, 1, !dbg !574
  store i32 %30, ptr %3, align 4, !dbg !574
  call void @_ZdlPv(ptr noundef %6) #12, !dbg !521
  br label %52, !dbg !521

31:                                               ; preds = %7
  %32 = landingpad { ptr, i32 }
          cleanup, !dbg !574
  %33 = extractvalue { ptr, i32 } %32, 0, !dbg !574
  store ptr %33, ptr %2, align 8, !dbg !574
  %34 = extractvalue { ptr, i32 } %32, 1, !dbg !574
  store i32 %34, ptr %3, align 4, !dbg !574
  call void @_ZdlPv(ptr noundef %9) #12, !dbg !537
  br label %52, !dbg !537

35:                                               ; preds = %12
  %36 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 0, !dbg !575
  %37 = load ptr, ptr %36, align 8, !dbg !575
  %38 = icmp eq ptr %37, null, !dbg !576
  br i1 %38, label %43, label %39, !dbg !576

39:                                               ; preds = %35
  %40 = load ptr, ptr %37, align 8, !dbg !576
  %41 = getelementptr inbounds ptr, ptr %40, i64 1, !dbg !576
  %42 = load ptr, ptr %41, align 8, !dbg !576
  call void %42(ptr noundef nonnull align 8 dereferenceable(8) %37) #11, !dbg !576
  br label %43, !dbg !576

43:                                               ; preds = %39, %35
  %44 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 1, !dbg !577
  %45 = load ptr, ptr %44, align 8, !dbg !577
  %46 = icmp eq ptr %45, null, !dbg !578
  br i1 %46, label %51, label %47, !dbg !578

47:                                               ; preds = %43
  %48 = load ptr, ptr %45, align 8, !dbg !578
  %49 = getelementptr inbounds ptr, ptr %48, i64 1, !dbg !578
  %50 = load ptr, ptr %49, align 8, !dbg !578
  call void %50(ptr noundef nonnull align 8 dereferenceable(8) %45) #11, !dbg !578
  br label %51, !dbg !578

51:                                               ; preds = %47, %43
  ret void, !dbg !574

52:                                               ; preds = %31, %27
  %53 = load ptr, ptr %2, align 8, !dbg !521
  %54 = load i32, ptr %3, align 4, !dbg !521
  %55 = insertvalue { ptr, i32 } poison, ptr %53, 0, !dbg !521
  %56 = insertvalue { ptr, i32 } %55, i32 %54, 1, !dbg !521
  resume { ptr, i32 } %56, !dbg !521
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6CircleC2Ed(ptr noundef nonnull align 8 dereferenceable(16) %0, double noundef %1) unnamed_addr #4 comdat align 2 !dbg !579 {
  %3 = alloca ptr, align 8
  %4 = alloca double, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !580, metadata !DIExpression()), !dbg !582
  store double %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !583, metadata !DIExpression()), !dbg !584
  %5 = load ptr, ptr %3, align 8
  call void @_ZN13AbstractShapeC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %5) #11, !dbg !585
  store ptr getelementptr inbounds ({ [6 x ptr] }, ptr @_ZTV6Circle, i32 0, inrange i32 0, i32 2), ptr %5, align 8, !dbg !586
  %6 = getelementptr inbounds %class.Circle, ptr %5, i32 0, i32 1, !dbg !587
  %7 = load double, ptr %4, align 8, !dbg !588
  store double %7, ptr %6, align 8, !dbg !587
  ret void, !dbg !589
}

declare i32 @__gxx_personality_v0(...)

; Function Attrs: nobuiltin nounwind
declare void @_ZdlPv(ptr noundef) #5

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9RectangleC2Edd(ptr noundef nonnull align 8 dereferenceable(24) %0, double noundef %1, double noundef %2) unnamed_addr #4 comdat align 2 !dbg !590 {
  %4 = alloca ptr, align 8
  %5 = alloca double, align 8
  %6 = alloca double, align 8
  store ptr %0, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !591, metadata !DIExpression()), !dbg !593
  store double %1, ptr %5, align 8
  call void @llvm.dbg.declare(metadata ptr %5, metadata !594, metadata !DIExpression()), !dbg !595
  store double %2, ptr %6, align 8
  call void @llvm.dbg.declare(metadata ptr %6, metadata !596, metadata !DIExpression()), !dbg !597
  %7 = load ptr, ptr %4, align 8
  call void @_ZN13AbstractShapeC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %7) #11, !dbg !598
  store ptr getelementptr inbounds ({ [6 x ptr] }, ptr @_ZTV9Rectangle, i32 0, inrange i32 0, i32 2), ptr %7, align 8, !dbg !599
  %8 = getelementptr inbounds %class.Rectangle, ptr %7, i32 0, i32 1, !dbg !600
  %9 = load double, ptr %5, align 8, !dbg !601
  store double %9, ptr %8, align 8, !dbg !600
  %10 = getelementptr inbounds %class.Rectangle, ptr %7, i32 0, i32 2, !dbg !602
  %11 = load double, ptr %6, align 8, !dbg !603
  store double %11, ptr %10, align 8, !dbg !602
  ret void, !dbg !604
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z22test_interface_patternv() #0 !dbg !605 {
  %1 = alloca [2 x ptr], align 8
  %2 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !606, metadata !DIExpression()), !dbg !619
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !620, !heapallocsite !621
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !628
  call void @_ZN17ConcreteObserverAC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !628
  %4 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 0, !dbg !629
  store ptr %3, ptr %4, align 8, !dbg !630
  %5 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !631, !heapallocsite !632
  call void @llvm.memset.p0.i64(ptr align 8 %5, i8 0, i64 8, i1 false), !dbg !639
  call void @_ZN17ConcreteObserverBC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %5) #11, !dbg !639
  %6 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 1, !dbg !640
  store ptr %5, ptr %6, align 8, !dbg !641
  call void @llvm.dbg.declare(metadata ptr %2, metadata !642, metadata !DIExpression()), !dbg !644
  store i32 0, ptr %2, align 4, !dbg !644
  br label %7, !dbg !645

7:                                                ; preds = %18, %0
  %8 = load i32, ptr %2, align 4, !dbg !646
  %9 = icmp slt i32 %8, 2, !dbg !648
  br i1 %9, label %10, label %21, !dbg !649

10:                                               ; preds = %7
  %11 = load i32, ptr %2, align 4, !dbg !650
  %12 = sext i32 %11 to i64, !dbg !652
  %13 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 %12, !dbg !652
  %14 = load ptr, ptr %13, align 8, !dbg !652
  %15 = load ptr, ptr %14, align 8, !dbg !653
  %16 = getelementptr inbounds ptr, ptr %15, i64 2, !dbg !653
  %17 = load ptr, ptr %16, align 8, !dbg !653
  call void %17(ptr noundef nonnull align 8 dereferenceable(8) %14, i32 noundef 42), !dbg !653
  br label %18, !dbg !654

18:                                               ; preds = %10
  %19 = load i32, ptr %2, align 4, !dbg !655
  %20 = add nsw i32 %19, 1, !dbg !655
  store i32 %20, ptr %2, align 4, !dbg !655
  br label %7, !dbg !656, !llvm.loop !657

21:                                               ; preds = %7
  %22 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 0, !dbg !659
  %23 = load ptr, ptr %22, align 8, !dbg !659
  %24 = icmp eq ptr %23, null, !dbg !660
  br i1 %24, label %29, label %25, !dbg !660

25:                                               ; preds = %21
  %26 = load ptr, ptr %23, align 8, !dbg !660
  %27 = getelementptr inbounds ptr, ptr %26, i64 1, !dbg !660
  %28 = load ptr, ptr %27, align 8, !dbg !660
  call void %28(ptr noundef nonnull align 8 dereferenceable(8) %23) #11, !dbg !660
  br label %29, !dbg !660

29:                                               ; preds = %25, %21
  %30 = getelementptr inbounds [2 x ptr], ptr %1, i64 0, i64 1, !dbg !661
  %31 = load ptr, ptr %30, align 8, !dbg !661
  %32 = icmp eq ptr %31, null, !dbg !662
  br i1 %32, label %37, label %33, !dbg !662

33:                                               ; preds = %29
  %34 = load ptr, ptr %31, align 8, !dbg !662
  %35 = getelementptr inbounds ptr, ptr %34, i64 1, !dbg !662
  %36 = load ptr, ptr %35, align 8, !dbg !662
  call void %36(ptr noundef nonnull align 8 dereferenceable(8) %31) #11, !dbg !662
  br label %37, !dbg !662

37:                                               ; preds = %33, %29
  ret void, !dbg !663
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverAC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !664 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !668, metadata !DIExpression()), !dbg !670
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9IObserverC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !671
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV17ConcreteObserverA, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !671
  ret void, !dbg !671
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverBC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !672 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !676, metadata !DIExpression()), !dbg !678
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9IObserverC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !679
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV17ConcreteObserverB, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !679
  ret void, !dbg !679
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z9test_crtpv() #0 !dbg !680 {
  %1 = alloca %class.CRTPDerived, align 1
  call void @llvm.dbg.declare(metadata ptr %1, metadata !681, metadata !DIExpression()), !dbg !682
  call void @_ZN8CRTPBaseI11CRTPDerivedE9interfaceEv(ptr noundef nonnull align 1 dereferenceable(1) %1), !dbg !683
  ret void, !dbg !684
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local void @_ZN8CRTPBaseI11CRTPDerivedE9interfaceEv(ptr noundef nonnull align 1 dereferenceable(1) %0) #0 comdat align 2 !dbg !685 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !686, metadata !DIExpression()), !dbg !688
  %3 = load ptr, ptr %2, align 8
  call void @_ZN11CRTPDerived14implementationEv(ptr noundef nonnull align 1 dereferenceable(1) %3), !dbg !689
  ret void, !dbg !690
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z22test_prototype_patternv() #0 !dbg !691 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !692, metadata !DIExpression()), !dbg !704
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !705, !heapallocsite !706
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !713
  call void @_ZN18ConcretePrototype1C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !713
  store ptr %3, ptr %1, align 8, !dbg !704
  call void @llvm.dbg.declare(metadata ptr %2, metadata !714, metadata !DIExpression()), !dbg !715
  %4 = load ptr, ptr %1, align 8, !dbg !716
  %5 = load ptr, ptr %4, align 8, !dbg !717
  %6 = getelementptr inbounds ptr, ptr %5, i64 2, !dbg !717
  %7 = load ptr, ptr %6, align 8, !dbg !717
  %8 = call noundef ptr %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !717
  store ptr %8, ptr %2, align 8, !dbg !715
  %9 = load ptr, ptr %1, align 8, !dbg !718
  %10 = icmp eq ptr %9, null, !dbg !719
  br i1 %10, label %15, label %11, !dbg !719

11:                                               ; preds = %0
  %12 = load ptr, ptr %9, align 8, !dbg !719
  %13 = getelementptr inbounds ptr, ptr %12, i64 1, !dbg !719
  %14 = load ptr, ptr %13, align 8, !dbg !719
  call void %14(ptr noundef nonnull align 8 dereferenceable(8) %9) #11, !dbg !719
  br label %15, !dbg !719

15:                                               ; preds = %11, %0
  %16 = load ptr, ptr %2, align 8, !dbg !720
  %17 = icmp eq ptr %16, null, !dbg !721
  br i1 %17, label %22, label %18, !dbg !721

18:                                               ; preds = %15
  %19 = load ptr, ptr %16, align 8, !dbg !721
  %20 = getelementptr inbounds ptr, ptr %19, i64 1, !dbg !721
  %21 = load ptr, ptr %20, align 8, !dbg !721
  call void %21(ptr noundef nonnull align 8 dereferenceable(8) %16) #11, !dbg !721
  br label %22, !dbg !721

22:                                               ; preds = %18, %15
  ret void, !dbg !722
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN18ConcretePrototype1C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !723 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !727, metadata !DIExpression()), !dbg !729
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9PrototypeC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !730
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV18ConcretePrototype1, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !730
  ret void, !dbg !730
}

; Function Attrs: mustprogress noinline optnone uwtable
define dso_local void @_Z21test_covariant_returnv() #0 !dbg !731 {
  %1 = alloca ptr, align 8
  %2 = alloca ptr, align 8
  call void @llvm.dbg.declare(metadata ptr %1, metadata !732, metadata !DIExpression()), !dbg !744
  %3 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !745, !heapallocsite !746
  call void @llvm.memset.p0.i64(ptr align 8 %3, i8 0, i64 8, i1 false), !dbg !754
  call void @_ZN16CloneableDerivedC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !754
  store ptr %3, ptr %1, align 8, !dbg !744
  call void @llvm.dbg.declare(metadata ptr %2, metadata !755, metadata !DIExpression()), !dbg !756
  %4 = load ptr, ptr %1, align 8, !dbg !757
  %5 = load ptr, ptr %4, align 8, !dbg !758
  %6 = getelementptr inbounds ptr, ptr %5, i64 2, !dbg !758
  %7 = load ptr, ptr %6, align 8, !dbg !758
  %8 = call noundef ptr %7(ptr noundef nonnull align 8 dereferenceable(8) %4), !dbg !758
  store ptr %8, ptr %2, align 8, !dbg !756
  %9 = load ptr, ptr %1, align 8, !dbg !759
  %10 = icmp eq ptr %9, null, !dbg !760
  br i1 %10, label %15, label %11, !dbg !760

11:                                               ; preds = %0
  %12 = load ptr, ptr %9, align 8, !dbg !760
  %13 = getelementptr inbounds ptr, ptr %12, i64 1, !dbg !760
  %14 = load ptr, ptr %13, align 8, !dbg !760
  call void %14(ptr noundef nonnull align 8 dereferenceable(8) %9) #11, !dbg !760
  br label %15, !dbg !760

15:                                               ; preds = %11, %0
  %16 = load ptr, ptr %2, align 8, !dbg !761
  %17 = icmp eq ptr %16, null, !dbg !762
  br i1 %17, label %22, label %18, !dbg !762

18:                                               ; preds = %15
  %19 = load ptr, ptr %16, align 8, !dbg !762
  %20 = getelementptr inbounds ptr, ptr %19, i64 1, !dbg !762
  %21 = load ptr, ptr %20, align 8, !dbg !762
  call void %21(ptr noundef nonnull align 8 dereferenceable(8) %16) #11, !dbg !762
  br label %22, !dbg !762

22:                                               ; preds = %18, %15
  ret void, !dbg !763
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN16CloneableDerivedC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !764 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !768, metadata !DIExpression()), !dbg !769
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13CloneableBaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !770
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV16CloneableDerived, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !770
  ret void, !dbg !770
}

; Function Attrs: mustprogress noinline norecurse optnone uwtable
define dso_local noundef i32 @main() #6 !dbg !771 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  call void @_Z23test_single_inheritancev(), !dbg !772
  call void @_Z21test_deep_inheritancev(), !dbg !773
  call void @_Z21test_multiple_derivedi(i32 noundef 0), !dbg !774
  call void @_Z25test_multiple_inheritancev(), !dbg !775
  call void @_Z24test_diamond_inheritancev(), !dbg !776
  call void @_Z19test_abstract_classv(), !dbg !777
  call void @_Z22test_interface_patternv(), !dbg !778
  call void @_Z9test_crtpv(), !dbg !779
  call void @_Z22test_prototype_patternv(), !dbg !780
  call void @_Z21test_covariant_returnv(), !dbg !781
  ret i32 0, !dbg !782
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !783 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !785, metadata !DIExpression()), !dbg !786
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [6 x ptr] }, ptr @_ZTV4Base, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !787
  ret void, !dbg !787
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN7DerivedD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !788 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !790, metadata !DIExpression()), !dbg !791
  %3 = load ptr, ptr %2, align 8
  call void @_ZN4BaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !792
  ret void, !dbg !794
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN7DerivedD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !795 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !796, metadata !DIExpression()), !dbg !797
  %3 = load ptr, ptr %2, align 8
  call void @_ZN7DerivedD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !798
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !798
  ret void, !dbg !798
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN7Derived3fooEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !799 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !800, metadata !DIExpression()), !dbg !801
  %3 = load ptr, ptr %2, align 8
  ret i32 2, !dbg !802
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN4Base3barEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !803 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !804, metadata !DIExpression()), !dbg !805
  %3 = load ptr, ptr %2, align 8
  ret i32 10, !dbg !806
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !807 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !808, metadata !DIExpression()), !dbg !809
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !810
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BaseD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !811 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !812, metadata !DIExpression()), !dbg !813
  %3 = load ptr, ptr %2, align 8
  call void @_ZN4BaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !814
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !814
  ret void, !dbg !814
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN4Base3fooEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !815 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !816, metadata !DIExpression()), !dbg !817
  %3 = load ptr, ptr %2, align 8
  ret i32 1, !dbg !818
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level2C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !819 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !823, metadata !DIExpression()), !dbg !825
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level1C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !826
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV6Level2, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !826
  ret void, !dbg !826
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level3D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !827 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !829, metadata !DIExpression()), !dbg !830
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level2D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !831
  ret void, !dbg !833
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level3D0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !834 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !835, metadata !DIExpression()), !dbg !836
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level3D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !837
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !837
  ret void, !dbg !837
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN6Level36methodEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !838 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !839, metadata !DIExpression()), !dbg !840
  %3 = load ptr, ptr %2, align 8
  ret i32 300, !dbg !841
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level1C2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !842 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !844, metadata !DIExpression()), !dbg !845
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV6Level1, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !846
  ret void, !dbg !846
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level2D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !847 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !849, metadata !DIExpression()), !dbg !850
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level1D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !851
  ret void, !dbg !853
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level2D0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !854 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !855, metadata !DIExpression()), !dbg !856
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level2D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !857
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !857
  ret void, !dbg !857
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN6Level26methodEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !858 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !859, metadata !DIExpression()), !dbg !860
  %3 = load ptr, ptr %2, align 8
  ret i32 200, !dbg !861
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level1D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !862 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !863, metadata !DIExpression()), !dbg !864
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !865
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Level1D0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !866 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !867, metadata !DIExpression()), !dbg !868
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6Level1D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !869
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !869
  ret void, !dbg !869
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN6Level16methodEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !870 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !871, metadata !DIExpression()), !dbg !872
  %3 = load ptr, ptr %2, align 8
  ret i32 100, !dbg !873
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !874 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !876, metadata !DIExpression()), !dbg !877
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV6Animal, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !878
  ret void, !dbg !878
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !879 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !881, metadata !DIExpression()), !dbg !882
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !883
  ret void, !dbg !885
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3DogD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !886 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !887, metadata !DIExpression()), !dbg !888
  %3 = load ptr, ptr %2, align 8
  call void @_ZN3DogD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !889
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !889
  ret void, !dbg !889
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3Dog5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !890 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !891, metadata !DIExpression()), !dbg !892
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !893
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !894 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !895, metadata !DIExpression()), !dbg !896
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !897
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6AnimalD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !898 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !899, metadata !DIExpression()), !dbg !900
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !901
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !901
  ret void, !dbg !901
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6Animal5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !902 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !903, metadata !DIExpression()), !dbg !904
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !905
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3CatD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !906 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !908, metadata !DIExpression()), !dbg !909
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !910
  ret void, !dbg !912
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3CatD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !913 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !914, metadata !DIExpression()), !dbg !915
  %3 = load ptr, ptr %2, align 8
  call void @_ZN3CatD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !916
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !916
  ret void, !dbg !916
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN3Cat5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !917 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !918, metadata !DIExpression()), !dbg !919
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !920
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BirdD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !921 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !923, metadata !DIExpression()), !dbg !924
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6AnimalD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !925
  ret void, !dbg !927
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4BirdD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !928 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !929, metadata !DIExpression()), !dbg !930
  %3 = load ptr, ptr %2, align 8
  call void @_ZN4BirdD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !931
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !931
  ret void, !dbg !931
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN4Bird5speakEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !932 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !933, metadata !DIExpression()), !dbg !934
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !935
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrintableC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !936 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !938, metadata !DIExpression()), !dbg !939
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV9Printable, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !940
  ret void, !dbg !940
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12SerializableC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !941 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !943, metadata !DIExpression()), !dbg !944
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV12Serializable, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !945
  ret void, !dbg !945
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8DocumentD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !946 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !948, metadata !DIExpression()), !dbg !949
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds i8, ptr %3, i64 8, !dbg !950
  call void @_ZN12SerializableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %4) #11, !dbg !950
  call void @_ZN9PrintableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !950
  ret void, !dbg !952
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8DocumentD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !953 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !954, metadata !DIExpression()), !dbg !955
  %3 = load ptr, ptr %2, align 8
  call void @_ZN8DocumentD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %3) #11, !dbg !956
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !956
  ret void, !dbg !956
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8Document5printEv(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !957 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !958, metadata !DIExpression()), !dbg !959
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !960
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN8Document9serializeEv(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !961 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !962, metadata !DIExpression()), !dbg !963
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !964
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZThn8_N8DocumentD1Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !965 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !967
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !967
  tail call void @_ZN8DocumentD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %4) #11, !dbg !967
  ret void, !dbg !967
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZThn8_N8DocumentD0Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !968 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !969
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !969
  tail call void @_ZN8DocumentD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %4) #11, !dbg !969
  ret void, !dbg !969
}

; Function Attrs: noinline optnone uwtable
define linkonce_odr dso_local void @_ZThn8_N8Document9serializeEv(ptr noundef %0) unnamed_addr #8 comdat align 2 !dbg !970 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !971
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !971
  tail call void @_ZN8Document9serializeEv(ptr noundef nonnull align 8 dereferenceable(16) %4), !dbg !971
  ret void, !dbg !971
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrintableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !972 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !973, metadata !DIExpression()), !dbg !974
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !975
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrintableD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !976 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !977, metadata !DIExpression()), !dbg !978
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9PrintableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !979
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !979
  ret void, !dbg !979
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9Printable5printEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !980 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !981, metadata !DIExpression()), !dbg !982
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !983
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12SerializableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !984 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !985, metadata !DIExpression()), !dbg !986
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !987
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12SerializableD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !988 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !989, metadata !DIExpression()), !dbg !990
  %3 = load ptr, ptr %2, align 8
  call void @_ZN12SerializableD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !991
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !991
  ret void, !dbg !991
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12Serializable9serializeEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !992 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !993, metadata !DIExpression()), !dbg !994
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !995
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondBaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !996 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !998, metadata !DIExpression()), !dbg !999
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV11DiamondBase, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !1000
  ret void, !dbg !1000
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondLeftC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #4 comdat align 2 !dbg !1001 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1005, metadata !DIExpression()), !dbg !1007
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1008, metadata !DIExpression()), !dbg !1007
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %6, align 8, !dbg !1010
  store ptr %7, ptr %5, align 8, !dbg !1010
  %8 = getelementptr inbounds ptr, ptr %6, i64 1, !dbg !1010
  %9 = load ptr, ptr %8, align 8, !dbg !1010
  %10 = load ptr, ptr %5, align 8, !dbg !1010
  %11 = getelementptr i8, ptr %10, i64 -40, !dbg !1010
  %12 = load i64, ptr %11, align 8, !dbg !1010
  %13 = getelementptr inbounds i8, ptr %5, i64 %12, !dbg !1010
  store ptr %9, ptr %13, align 8, !dbg !1010
  ret void, !dbg !1010
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12DiamondRightC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #4 comdat align 2 !dbg !1011 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1015, metadata !DIExpression()), !dbg !1017
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1018, metadata !DIExpression()), !dbg !1017
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = load ptr, ptr %6, align 8, !dbg !1019
  store ptr %7, ptr %5, align 8, !dbg !1019
  %8 = getelementptr inbounds ptr, ptr %6, i64 1, !dbg !1019
  %9 = load ptr, ptr %8, align 8, !dbg !1019
  %10 = load ptr, ptr %5, align 8, !dbg !1019
  %11 = getelementptr i8, ptr %10, i64 -40, !dbg !1019
  %12 = load i64, ptr %11, align 8, !dbg !1019
  %13 = getelementptr inbounds i8, ptr %5, i64 %12, !dbg !1019
  store ptr %9, ptr %13, align 8, !dbg !1019
  ret void, !dbg !1019
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondLeftD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1020 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1022, metadata !DIExpression()), !dbg !1023
  %3 = load ptr, ptr %2, align 8
  call void @_ZN11DiamondLeftD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3, ptr noundef @_ZTT11DiamondLeft) #11, !dbg !1024
  call void @_ZN11DiamondBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1024
  ret void, !dbg !1024
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondLeftD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1025 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1026, metadata !DIExpression()), !dbg !1027
  %3 = load ptr, ptr %2, align 8
  call void @_ZN11DiamondLeftD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1028
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1028
  ret void, !dbg !1028
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN11DiamondLeft5valueEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1029 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1030, metadata !DIExpression()), !dbg !1031
  %3 = load ptr, ptr %2, align 8
  ret i32 1, !dbg !1032
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12DiamondRightD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1033 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1035, metadata !DIExpression()), !dbg !1036
  %3 = load ptr, ptr %2, align 8
  call void @_ZN12DiamondRightD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3, ptr noundef @_ZTT12DiamondRight) #11, !dbg !1037
  call void @_ZN11DiamondBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1037
  ret void, !dbg !1037
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12DiamondRightD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1038 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1039, metadata !DIExpression()), !dbg !1040
  %3 = load ptr, ptr %2, align 8
  call void @_ZN12DiamondRightD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1041
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1041
  ret void, !dbg !1041
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN12DiamondRight5valueEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1042 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1043, metadata !DIExpression()), !dbg !1044
  %3 = load ptr, ptr %2, align 8
  ret i32 2, !dbg !1045
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N12DiamondRightD1Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1046 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1047
  %4 = load ptr, ptr %3, align 8, !dbg !1047
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1047
  %6 = load i64, ptr %5, align 8, !dbg !1047
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1047
  tail call void @_ZN12DiamondRightD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %7) #11, !dbg !1047
  ret void, !dbg !1047
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N12DiamondRightD0Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1048 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1049
  %4 = load ptr, ptr %3, align 8, !dbg !1049
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1049
  %6 = load i64, ptr %5, align 8, !dbg !1049
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1049
  tail call void @_ZN12DiamondRightD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %7) #11, !dbg !1049
  ret void, !dbg !1049
}

; Function Attrs: noinline optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZTv0_n32_N12DiamondRight5valueEv(ptr noundef %0) unnamed_addr #8 comdat align 2 !dbg !1050 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1051
  %4 = load ptr, ptr %3, align 8, !dbg !1051
  %5 = getelementptr inbounds i8, ptr %4, i64 -32, !dbg !1051
  %6 = load i64, ptr %5, align 8, !dbg !1051
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1051
  %8 = tail call noundef i32 @_ZN12DiamondRight5valueEv(ptr noundef nonnull align 8 dereferenceable(8) %7), !dbg !1051
  ret i32 %8, !dbg !1051
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13DiamondBottomD1Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1052 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1054, metadata !DIExpression()), !dbg !1055
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13DiamondBottomD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %3, ptr noundef @_ZTT13DiamondBottom) #11, !dbg !1056
  call void @_ZN11DiamondBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1056
  ret void, !dbg !1056
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13DiamondBottomD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1057 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1058, metadata !DIExpression()), !dbg !1059
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13DiamondBottomD1Ev(ptr noundef nonnull align 8 dereferenceable(16) %3) #11, !dbg !1060
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1060
  ret void, !dbg !1060
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN13DiamondBottom5valueEv(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1061 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1062, metadata !DIExpression()), !dbg !1063
  %3 = load ptr, ptr %2, align 8
  ret i32 3, !dbg !1064
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZThn8_N13DiamondBottomD1Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1065 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1066
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !1066
  tail call void @_ZN13DiamondBottomD1Ev(ptr noundef nonnull align 8 dereferenceable(16) %4) #11, !dbg !1066
  ret void, !dbg !1066
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZThn8_N13DiamondBottomD0Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1067 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1068
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !1068
  tail call void @_ZN13DiamondBottomD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %4) #11, !dbg !1068
  ret void, !dbg !1068
}

; Function Attrs: noinline optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZThn8_N13DiamondBottom5valueEv(ptr noundef %0) unnamed_addr #8 comdat align 2 !dbg !1069 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1070
  %4 = getelementptr inbounds i8, ptr %3, i64 -8, !dbg !1070
  %5 = tail call noundef i32 @_ZN13DiamondBottom5valueEv(ptr noundef nonnull align 8 dereferenceable(16) %4), !dbg !1070
  ret i32 %5, !dbg !1070
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1071 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1072, metadata !DIExpression()), !dbg !1073
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1074
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondBaseD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1075 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1076, metadata !DIExpression()), !dbg !1077
  %3 = load ptr, ptr %2, align 8
  call void @_ZN11DiamondBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1078
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1078
  ret void, !dbg !1078
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZN11DiamondBase5valueEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1079 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1080, metadata !DIExpression()), !dbg !1081
  %3 = load ptr, ptr %2, align 8
  ret i32 0, !dbg !1082
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11DiamondLeftD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #4 comdat align 2 !dbg !1083 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1084, metadata !DIExpression()), !dbg !1085
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1086, metadata !DIExpression()), !dbg !1085
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  ret void, !dbg !1087
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N11DiamondLeftD1Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1088 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1089
  %4 = load ptr, ptr %3, align 8, !dbg !1089
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1089
  %6 = load i64, ptr %5, align 8, !dbg !1089
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1089
  tail call void @_ZN11DiamondLeftD1Ev(ptr noundef nonnull align 8 dereferenceable(8) %7) #11, !dbg !1089
  ret void, !dbg !1089
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N11DiamondLeftD0Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1090 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1091
  %4 = load ptr, ptr %3, align 8, !dbg !1091
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1091
  %6 = load i64, ptr %5, align 8, !dbg !1091
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1091
  tail call void @_ZN11DiamondLeftD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %7) #11, !dbg !1091
  ret void, !dbg !1091
}

; Function Attrs: noinline optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZTv0_n32_N11DiamondLeft5valueEv(ptr noundef %0) unnamed_addr #8 comdat align 2 !dbg !1092 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1093
  %4 = load ptr, ptr %3, align 8, !dbg !1093
  %5 = getelementptr inbounds i8, ptr %4, i64 -32, !dbg !1093
  %6 = load i64, ptr %5, align 8, !dbg !1093
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1093
  %8 = tail call noundef i32 @_ZN11DiamondLeft5valueEv(ptr noundef nonnull align 8 dereferenceable(8) %7), !dbg !1093
  ret i32 %8, !dbg !1093
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN12DiamondRightD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef %1) unnamed_addr #4 comdat align 2 !dbg !1094 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1095, metadata !DIExpression()), !dbg !1096
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1097, metadata !DIExpression()), !dbg !1096
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  ret void, !dbg !1098
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13DiamondBottomD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %0, ptr noundef %1) unnamed_addr #4 comdat align 2 !dbg !1099 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1100, metadata !DIExpression()), !dbg !1101
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1102, metadata !DIExpression()), !dbg !1101
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8
  %7 = getelementptr inbounds i8, ptr %5, i64 8, !dbg !1103
  %8 = getelementptr inbounds ptr, ptr %6, i64 3, !dbg !1103
  call void @_ZN12DiamondRightD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %7, ptr noundef %8) #11, !dbg !1103
  %9 = getelementptr inbounds ptr, ptr %6, i64 1, !dbg !1103
  call void @_ZN11DiamondLeftD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %5, ptr noundef %9) #11, !dbg !1103
  ret void, !dbg !1105
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N13DiamondBottomD1Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1106 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1107
  %4 = load ptr, ptr %3, align 8, !dbg !1107
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1107
  %6 = load i64, ptr %5, align 8, !dbg !1107
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1107
  tail call void @_ZN13DiamondBottomD1Ev(ptr noundef nonnull align 8 dereferenceable(16) %7) #11, !dbg !1107
  ret void, !dbg !1107
}

; Function Attrs: noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZTv0_n24_N13DiamondBottomD0Ev(ptr noundef %0) unnamed_addr #7 comdat align 2 !dbg !1108 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1109
  %4 = load ptr, ptr %3, align 8, !dbg !1109
  %5 = getelementptr inbounds i8, ptr %4, i64 -24, !dbg !1109
  %6 = load i64, ptr %5, align 8, !dbg !1109
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1109
  tail call void @_ZN13DiamondBottomD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %7) #11, !dbg !1109
  ret void, !dbg !1109
}

; Function Attrs: noinline optnone uwtable
define linkonce_odr dso_local noundef i32 @_ZTv0_n32_N13DiamondBottom5valueEv(ptr noundef %0) unnamed_addr #8 comdat align 2 !dbg !1110 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  %3 = load ptr, ptr %2, align 8, !dbg !1111
  %4 = load ptr, ptr %3, align 8, !dbg !1111
  %5 = getelementptr inbounds i8, ptr %4, i64 -32, !dbg !1111
  %6 = load i64, ptr %5, align 8, !dbg !1111
  %7 = getelementptr inbounds i8, ptr %3, i64 %6, !dbg !1111
  %8 = tail call noundef i32 @_ZN13DiamondBottom5valueEv(ptr noundef nonnull align 8 dereferenceable(16) %7), !dbg !1111
  ret i32 %8, !dbg !1111
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13AbstractShapeC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1112 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1114, metadata !DIExpression()), !dbg !1115
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [6 x ptr] }, ptr @_ZTV13AbstractShape, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !1116
  ret void, !dbg !1116
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6CircleD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1117 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1121, metadata !DIExpression()), !dbg !1122
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13AbstractShapeD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1123
  ret void, !dbg !1125
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN6CircleD0Ev(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1126 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1127, metadata !DIExpression()), !dbg !1128
  %3 = load ptr, ptr %2, align 8
  call void @_ZN6CircleD2Ev(ptr noundef nonnull align 8 dereferenceable(16) %3) #11, !dbg !1129
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1129
  ret void, !dbg !1129
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef double @_ZN6Circle4areaEv(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1130 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1131, metadata !DIExpression()), !dbg !1132
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Circle, ptr %3, i32 0, i32 1, !dbg !1133
  %5 = load double, ptr %4, align 8, !dbg !1133
  %6 = fmul double 3.141590e+00, %5, !dbg !1134
  %7 = getelementptr inbounds %class.Circle, ptr %3, i32 0, i32 1, !dbg !1135
  %8 = load double, ptr %7, align 8, !dbg !1135
  %9 = fmul double %6, %8, !dbg !1136
  ret double %9, !dbg !1137
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef double @_ZN6Circle9perimeterEv(ptr noundef nonnull align 8 dereferenceable(16) %0) unnamed_addr #4 comdat align 2 !dbg !1138 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1139, metadata !DIExpression()), !dbg !1140
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Circle, ptr %3, i32 0, i32 1, !dbg !1141
  %5 = load double, ptr %4, align 8, !dbg !1141
  %6 = fmul double 6.283180e+00, %5, !dbg !1142
  ret double %6, !dbg !1143
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13AbstractShapeD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1144 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1145, metadata !DIExpression()), !dbg !1146
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1147
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13AbstractShapeD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1148 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1149, metadata !DIExpression()), !dbg !1150
  %3 = load ptr, ptr %2, align 8
  call void @llvm.trap() #13, !dbg !1151
  unreachable, !dbg !1151
}

declare void @__cxa_pure_virtual() unnamed_addr

; Function Attrs: cold noreturn nounwind memory(inaccessiblemem: write)
declare void @llvm.trap() #9

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9RectangleD2Ev(ptr noundef nonnull align 8 dereferenceable(24) %0) unnamed_addr #4 comdat align 2 !dbg !1152 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1156, metadata !DIExpression()), !dbg !1157
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13AbstractShapeD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1158
  ret void, !dbg !1160
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9RectangleD0Ev(ptr noundef nonnull align 8 dereferenceable(24) %0) unnamed_addr #4 comdat align 2 !dbg !1161 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1162, metadata !DIExpression()), !dbg !1163
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9RectangleD2Ev(ptr noundef nonnull align 8 dereferenceable(24) %3) #11, !dbg !1164
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1164
  ret void, !dbg !1164
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef double @_ZN9Rectangle4areaEv(ptr noundef nonnull align 8 dereferenceable(24) %0) unnamed_addr #4 comdat align 2 !dbg !1165 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1166, metadata !DIExpression()), !dbg !1167
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Rectangle, ptr %3, i32 0, i32 1, !dbg !1168
  %5 = load double, ptr %4, align 8, !dbg !1168
  %6 = getelementptr inbounds %class.Rectangle, ptr %3, i32 0, i32 2, !dbg !1169
  %7 = load double, ptr %6, align 8, !dbg !1169
  %8 = fmul double %5, %7, !dbg !1170
  ret double %8, !dbg !1171
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local noundef double @_ZN9Rectangle9perimeterEv(ptr noundef nonnull align 8 dereferenceable(24) %0) unnamed_addr #4 comdat align 2 !dbg !1172 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1173, metadata !DIExpression()), !dbg !1174
  %3 = load ptr, ptr %2, align 8
  %4 = getelementptr inbounds %class.Rectangle, ptr %3, i32 0, i32 1, !dbg !1175
  %5 = load double, ptr %4, align 8, !dbg !1175
  %6 = getelementptr inbounds %class.Rectangle, ptr %3, i32 0, i32 2, !dbg !1176
  %7 = load double, ptr %6, align 8, !dbg !1176
  %8 = fadd double %5, %7, !dbg !1177
  %9 = fmul double 2.000000e+00, %8, !dbg !1178
  ret double %9, !dbg !1179
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9IObserverC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1180 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1182, metadata !DIExpression()), !dbg !1183
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV9IObserver, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !1184
  ret void, !dbg !1184
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverAD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1185 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1187, metadata !DIExpression()), !dbg !1188
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9IObserverD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1189
  ret void, !dbg !1191
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverAD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1192 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1193, metadata !DIExpression()), !dbg !1194
  %3 = load ptr, ptr %2, align 8
  call void @_ZN17ConcreteObserverAD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1195
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1195
  ret void, !dbg !1195
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverA6updateEi(ptr noundef nonnull align 8 dereferenceable(8) %0, i32 noundef %1) unnamed_addr #4 comdat align 2 !dbg !1196 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1197, metadata !DIExpression()), !dbg !1198
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1199, metadata !DIExpression()), !dbg !1200
  %5 = load ptr, ptr %3, align 8
  ret void, !dbg !1201
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9IObserverD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1202 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1203, metadata !DIExpression()), !dbg !1204
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1205
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9IObserverD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1206 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1207, metadata !DIExpression()), !dbg !1208
  %3 = load ptr, ptr %2, align 8
  call void @llvm.trap() #13, !dbg !1209
  unreachable, !dbg !1209
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverBD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1210 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1212, metadata !DIExpression()), !dbg !1213
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9IObserverD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1214
  ret void, !dbg !1216
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverBD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1217 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1218, metadata !DIExpression()), !dbg !1219
  %3 = load ptr, ptr %2, align 8
  call void @_ZN17ConcreteObserverBD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1220
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1220
  ret void, !dbg !1220
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN17ConcreteObserverB6updateEi(ptr noundef nonnull align 8 dereferenceable(8) %0, i32 noundef %1) unnamed_addr #4 comdat align 2 !dbg !1221 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1222, metadata !DIExpression()), !dbg !1223
  store i32 %1, ptr %4, align 4
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1224, metadata !DIExpression()), !dbg !1225
  %5 = load ptr, ptr %3, align 8
  ret void, !dbg !1226
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrototypeC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1227 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1229, metadata !DIExpression()), !dbg !1230
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV9Prototype, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !1231
  ret void, !dbg !1231
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN18ConcretePrototype1D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1232 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1234, metadata !DIExpression()), !dbg !1235
  %3 = load ptr, ptr %2, align 8
  call void @_ZN9PrototypeD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1236
  ret void, !dbg !1238
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN18ConcretePrototype1D0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1239 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1240, metadata !DIExpression()), !dbg !1241
  %3 = load ptr, ptr %2, align 8
  call void @_ZN18ConcretePrototype1D2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1242
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1242
  ret void, !dbg !1242
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local noundef ptr @_ZN18ConcretePrototype15cloneEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #0 comdat align 2 !dbg !1243 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1244, metadata !DIExpression()), !dbg !1245
  %3 = load ptr, ptr %2, align 8
  %4 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !1246, !heapallocsite !706
  call void @_ZN18ConcretePrototype1C2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %4, ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1247
  ret ptr %4, !dbg !1248
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrototypeD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1249 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1250, metadata !DIExpression()), !dbg !1251
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1252
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrototypeD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1253 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1254, metadata !DIExpression()), !dbg !1255
  %3 = load ptr, ptr %2, align 8
  call void @llvm.trap() #13, !dbg !1256
  unreachable, !dbg !1256
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN18ConcretePrototype1C2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef nonnull align 8 dereferenceable(8) %1) unnamed_addr #4 comdat align 2 !dbg !1257 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1263, metadata !DIExpression()), !dbg !1264
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1265, metadata !DIExpression()), !dbg !1264
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8, !dbg !1266
  call void @_ZN9PrototypeC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %5, ptr noundef nonnull align 8 dereferenceable(8) %6) #11, !dbg !1266
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV18ConcretePrototype1, i32 0, inrange i32 0, i32 2), ptr %5, align 8, !dbg !1266
  ret void, !dbg !1266
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN9PrototypeC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef nonnull align 8 dereferenceable(8) %1) unnamed_addr #4 comdat align 2 !dbg !1267 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1273, metadata !DIExpression()), !dbg !1274
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1275, metadata !DIExpression()), !dbg !1274
  %5 = load ptr, ptr %3, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV9Prototype, i32 0, inrange i32 0, i32 2), ptr %5, align 8, !dbg !1276
  ret void, !dbg !1276
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13CloneableBaseC2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1277 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1279, metadata !DIExpression()), !dbg !1280
  %3 = load ptr, ptr %2, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV13CloneableBase, i32 0, inrange i32 0, i32 2), ptr %3, align 8, !dbg !1281
  ret void, !dbg !1281
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN16CloneableDerivedD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1282 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1284, metadata !DIExpression()), !dbg !1285
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13CloneableBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1286
  ret void, !dbg !1288
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN16CloneableDerivedD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1289 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1290, metadata !DIExpression()), !dbg !1291
  %3 = load ptr, ptr %2, align 8
  call void @_ZN16CloneableDerivedD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1292
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1292
  ret void, !dbg !1292
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local noundef ptr @_ZN16CloneableDerived10clone_selfEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #0 comdat align 2 !dbg !1293 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1294, metadata !DIExpression()), !dbg !1295
  %3 = load ptr, ptr %2, align 8
  %4 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !1296, !heapallocsite !746
  call void @_ZN16CloneableDerivedC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %4, ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1297
  ret ptr %4, !dbg !1298
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13CloneableBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1299 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1300, metadata !DIExpression()), !dbg !1301
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1302
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13CloneableBaseD0Ev(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #4 comdat align 2 !dbg !1303 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1304, metadata !DIExpression()), !dbg !1305
  %3 = load ptr, ptr %2, align 8
  call void @_ZN13CloneableBaseD2Ev(ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1306
  call void @_ZdlPv(ptr noundef %3) #12, !dbg !1306
  ret void, !dbg !1306
}

; Function Attrs: mustprogress noinline optnone uwtable
define linkonce_odr dso_local noundef ptr @_ZN13CloneableBase10clone_selfEv(ptr noundef nonnull align 8 dereferenceable(8) %0) unnamed_addr #0 comdat align 2 !dbg !1307 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1308, metadata !DIExpression()), !dbg !1309
  %3 = load ptr, ptr %2, align 8
  %4 = call noalias noundef nonnull ptr @_Znwm(i64 noundef 8) #10, !dbg !1310, !heapallocsite !734
  call void @_ZN13CloneableBaseC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %4, ptr noundef nonnull align 8 dereferenceable(8) %3) #11, !dbg !1311
  ret ptr %4, !dbg !1312
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN13CloneableBaseC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef nonnull align 8 dereferenceable(8) %1) unnamed_addr #4 comdat align 2 !dbg !1313 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1319, metadata !DIExpression()), !dbg !1320
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1321, metadata !DIExpression()), !dbg !1320
  %5 = load ptr, ptr %3, align 8
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV13CloneableBase, i32 0, inrange i32 0, i32 2), ptr %5, align 8, !dbg !1322
  ret void, !dbg !1322
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN16CloneableDerivedC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %0, ptr noundef nonnull align 8 dereferenceable(8) %1) unnamed_addr #4 comdat align 2 !dbg !1323 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  call void @llvm.dbg.declare(metadata ptr %3, metadata !1329, metadata !DIExpression()), !dbg !1330
  store ptr %1, ptr %4, align 8
  call void @llvm.dbg.declare(metadata ptr %4, metadata !1331, metadata !DIExpression()), !dbg !1330
  %5 = load ptr, ptr %3, align 8
  %6 = load ptr, ptr %4, align 8, !dbg !1332
  call void @_ZN13CloneableBaseC2ERKS_(ptr noundef nonnull align 8 dereferenceable(8) %5, ptr noundef nonnull align 8 dereferenceable(8) %6) #11, !dbg !1332
  store ptr getelementptr inbounds ({ [5 x ptr] }, ptr @_ZTV16CloneableDerived, i32 0, inrange i32 0, i32 2), ptr %5, align 8, !dbg !1332
  ret void, !dbg !1332
}

; Function Attrs: mustprogress noinline nounwind optnone uwtable
define linkonce_odr dso_local void @_ZN11CRTPDerived14implementationEv(ptr noundef nonnull align 1 dereferenceable(1) %0) #4 comdat align 2 !dbg !1333 {
  %2 = alloca ptr, align 8
  store ptr %0, ptr %2, align 8
  call void @llvm.dbg.declare(metadata ptr %2, metadata !1334, metadata !DIExpression()), !dbg !1335
  %3 = load ptr, ptr %2, align 8
  ret void, !dbg !1336
}

attributes #0 = { mustprogress noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nobuiltin allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #3 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #4 = { mustprogress noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #5 = { nobuiltin nounwind "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #6 = { mustprogress noinline norecurse optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #7 = { noinline nounwind optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #8 = { noinline optnone uwtable "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="generic" "target-features"="+fp-armv8,+neon,+outline-atomics,+v8a,-fmv" }
attributes #9 = { cold noreturn nounwind memory(inaccessiblemem: write) }
attributes #10 = { builtin allocsize(0) }
attributes #11 = { nounwind }
attributes #12 = { builtin nounwind }
attributes #13 = { noreturn nounwind }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!231, !232, !233, !234, !235, !236, !237}
!llvm.ident = !{!238}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, retainedTypes: !2, imports: !20, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "tests/fixtures/pta_verification/class_hierarchy.cpp", directory: "/workspace", checksumkind: CSK_MD5, checksum: "94661677b83df5db74871c1dadab0f48")
!2 = !{!3}
!3 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64)
!4 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "CRTPDerived", file: !1, line: 242, size: 8, flags: DIFlagTypePassByValue, elements: !5, identifier: "_ZTS11CRTPDerived")
!5 = !{!6, !16}
!6 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !4, baseType: !7, flags: DIFlagPublic, extraData: i32 0)
!7 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "CRTPBase<CRTPDerived>", file: !1, line: 233, size: 8, flags: DIFlagTypePassByValue, elements: !8, templateParams: !14, identifier: "_ZTS8CRTPBaseI11CRTPDerivedE")
!8 = !{!9, !13}
!9 = !DISubprogram(name: "interface", linkageName: "_ZN8CRTPBaseI11CRTPDerivedE9interfaceEv", scope: !7, file: !1, line: 235, type: !10, scopeLine: 235, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!10 = !DISubroutineType(types: !11)
!11 = !{null, !12}
!12 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !7, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!13 = !DISubprogram(name: "implementation", linkageName: "_ZN8CRTPBaseI11CRTPDerivedE14implementationEv", scope: !7, file: !1, line: 239, type: !10, scopeLine: 239, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!14 = !{!15}
!15 = !DITemplateTypeParameter(name: "Derived", type: !4)
!16 = !DISubprogram(name: "implementation", linkageName: "_ZN11CRTPDerived14implementationEv", scope: !4, file: !1, line: 244, type: !17, scopeLine: 244, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!17 = !DISubroutineType(types: !18)
!18 = !{null, !19}
!19 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !4, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!20 = !{!21, !29, !33, !40, !44, !52, !57, !59, !67, !71, !75, !85, !87, !91, !95, !99, !104, !108, !112, !116, !120, !128, !132, !136, !138, !142, !146, !151, !157, !161, !165, !167, !175, !179, !187, !189, !193, !197, !201, !205, !210, !215, !220, !221, !222, !223, !225, !226, !227, !228, !229, !230}
!21 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !23, file: !28, line: 52)
!22 = !DINamespace(name: "std", scope: null)
!23 = !DISubprogram(name: "abs", scope: !24, file: !24, line: 980, type: !25, flags: DIFlagPrototyped, spFlags: 0)
!24 = !DIFile(filename: "/usr/include/stdlib.h", directory: "", checksumkind: CSK_MD5, checksum: "7fa2ecb2348a66f8b44ab9a15abd0b72")
!25 = !DISubroutineType(types: !26)
!26 = !{!27, !27}
!27 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!28 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/bits/std_abs.h", directory: "")
!29 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !30, file: !32, line: 131)
!30 = !DIDerivedType(tag: DW_TAG_typedef, name: "div_t", file: !24, line: 63, baseType: !31)
!31 = !DICompositeType(tag: DW_TAG_structure_type, file: !24, line: 59, size: 64, flags: DIFlagFwdDecl, identifier: "_ZTS5div_t")
!32 = !DIFile(filename: "/usr/bin/../lib/gcc/aarch64-linux-gnu/13/../../../../include/c++/13/cstdlib", directory: "")
!33 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !34, file: !32, line: 132)
!34 = !DIDerivedType(tag: DW_TAG_typedef, name: "ldiv_t", file: !24, line: 71, baseType: !35)
!35 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !24, line: 67, size: 128, flags: DIFlagTypePassByValue, elements: !36, identifier: "_ZTS6ldiv_t")
!36 = !{!37, !39}
!37 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !35, file: !24, line: 69, baseType: !38, size: 64)
!38 = !DIBasicType(name: "long", size: 64, encoding: DW_ATE_signed)
!39 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !35, file: !24, line: 70, baseType: !38, size: 64, offset: 64)
!40 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !41, file: !32, line: 134)
!41 = !DISubprogram(name: "abort", scope: !24, file: !24, line: 730, type: !42, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!42 = !DISubroutineType(types: !43)
!43 = !{null}
!44 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !45, file: !32, line: 136)
!45 = !DISubprogram(name: "aligned_alloc", scope: !24, file: !24, line: 724, type: !46, flags: DIFlagPrototyped, spFlags: 0)
!46 = !DISubroutineType(types: !47)
!47 = !{!48, !49, !49}
!48 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64)
!49 = !DIDerivedType(tag: DW_TAG_typedef, name: "size_t", file: !50, line: 18, baseType: !51)
!50 = !DIFile(filename: "/usr/lib/llvm-18/lib/clang/18/include/__stddef_size_t.h", directory: "", checksumkind: CSK_MD5, checksum: "2c44e821a2b1951cde2eb0fb2e656867")
!51 = !DIBasicType(name: "unsigned long", size: 64, encoding: DW_ATE_unsigned)
!52 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !53, file: !32, line: 138)
!53 = !DISubprogram(name: "atexit", scope: !24, file: !24, line: 734, type: !54, flags: DIFlagPrototyped, spFlags: 0)
!54 = !DISubroutineType(types: !55)
!55 = !{!27, !56}
!56 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !42, size: 64)
!57 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !58, file: !32, line: 141)
!58 = !DISubprogram(name: "at_quick_exit", scope: !24, file: !24, line: 739, type: !54, flags: DIFlagPrototyped, spFlags: 0)
!59 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !60, file: !32, line: 144)
!60 = !DISubprogram(name: "atof", scope: !24, file: !24, line: 102, type: !61, flags: DIFlagPrototyped, spFlags: 0)
!61 = !DISubroutineType(types: !62)
!62 = !{!63, !64}
!63 = !DIBasicType(name: "double", size: 64, encoding: DW_ATE_float)
!64 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !65, size: 64)
!65 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !66)
!66 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_unsigned_char)
!67 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !68, file: !32, line: 145)
!68 = !DISubprogram(name: "atoi", scope: !24, file: !24, line: 105, type: !69, flags: DIFlagPrototyped, spFlags: 0)
!69 = !DISubroutineType(types: !70)
!70 = !{!27, !64}
!71 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !72, file: !32, line: 146)
!72 = !DISubprogram(name: "atol", scope: !24, file: !24, line: 108, type: !73, flags: DIFlagPrototyped, spFlags: 0)
!73 = !DISubroutineType(types: !74)
!74 = !{!38, !64}
!75 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !76, file: !32, line: 147)
!76 = !DISubprogram(name: "bsearch", scope: !24, file: !24, line: 960, type: !77, flags: DIFlagPrototyped, spFlags: 0)
!77 = !DISubroutineType(types: !78)
!78 = !{!48, !79, !79, !49, !49, !81}
!79 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !80, size: 64)
!80 = !DIDerivedType(tag: DW_TAG_const_type, baseType: null)
!81 = !DIDerivedType(tag: DW_TAG_typedef, name: "__compar_fn_t", file: !24, line: 948, baseType: !82)
!82 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !83, size: 64)
!83 = !DISubroutineType(types: !84)
!84 = !{!27, !79, !79}
!85 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !86, file: !32, line: 148)
!86 = !DISubprogram(name: "calloc", scope: !24, file: !24, line: 675, type: !46, flags: DIFlagPrototyped, spFlags: 0)
!87 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !88, file: !32, line: 149)
!88 = !DISubprogram(name: "div", scope: !24, file: !24, line: 992, type: !89, flags: DIFlagPrototyped, spFlags: 0)
!89 = !DISubroutineType(types: !90)
!90 = !{!30, !27, !27}
!91 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !92, file: !32, line: 150)
!92 = !DISubprogram(name: "exit", scope: !24, file: !24, line: 756, type: !93, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!93 = !DISubroutineType(types: !94)
!94 = !{null, !27}
!95 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !96, file: !32, line: 151)
!96 = !DISubprogram(name: "free", scope: !24, file: !24, line: 687, type: !97, flags: DIFlagPrototyped, spFlags: 0)
!97 = !DISubroutineType(types: !98)
!98 = !{null, !48}
!99 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !100, file: !32, line: 152)
!100 = !DISubprogram(name: "getenv", scope: !24, file: !24, line: 773, type: !101, flags: DIFlagPrototyped, spFlags: 0)
!101 = !DISubroutineType(types: !102)
!102 = !{!103, !64}
!103 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !66, size: 64)
!104 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !105, file: !32, line: 153)
!105 = !DISubprogram(name: "labs", scope: !24, file: !24, line: 981, type: !106, flags: DIFlagPrototyped, spFlags: 0)
!106 = !DISubroutineType(types: !107)
!107 = !{!38, !38}
!108 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !109, file: !32, line: 154)
!109 = !DISubprogram(name: "ldiv", scope: !24, file: !24, line: 994, type: !110, flags: DIFlagPrototyped, spFlags: 0)
!110 = !DISubroutineType(types: !111)
!111 = !{!34, !38, !38}
!112 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !113, file: !32, line: 155)
!113 = !DISubprogram(name: "malloc", scope: !24, file: !24, line: 672, type: !114, flags: DIFlagPrototyped, spFlags: 0)
!114 = !DISubroutineType(types: !115)
!115 = !{!48, !49}
!116 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !117, file: !32, line: 157)
!117 = !DISubprogram(name: "mblen", scope: !24, file: !24, line: 1062, type: !118, flags: DIFlagPrototyped, spFlags: 0)
!118 = !DISubroutineType(types: !119)
!119 = !{!27, !64, !49}
!120 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !121, file: !32, line: 158)
!121 = !DISubprogram(name: "mbstowcs", scope: !24, file: !24, line: 1073, type: !122, flags: DIFlagPrototyped, spFlags: 0)
!122 = !DISubroutineType(types: !123)
!123 = !{!49, !124, !127, !49}
!124 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !125)
!125 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !126, size: 64)
!126 = !DIBasicType(name: "wchar_t", size: 32, encoding: DW_ATE_unsigned)
!127 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !64)
!128 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !129, file: !32, line: 159)
!129 = !DISubprogram(name: "mbtowc", scope: !24, file: !24, line: 1065, type: !130, flags: DIFlagPrototyped, spFlags: 0)
!130 = !DISubroutineType(types: !131)
!131 = !{!27, !124, !127, !49}
!132 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !133, file: !32, line: 161)
!133 = !DISubprogram(name: "qsort", scope: !24, file: !24, line: 970, type: !134, flags: DIFlagPrototyped, spFlags: 0)
!134 = !DISubroutineType(types: !135)
!135 = !{null, !48, !49, !49, !81}
!136 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !137, file: !32, line: 164)
!137 = !DISubprogram(name: "quick_exit", scope: !24, file: !24, line: 762, type: !93, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!138 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !139, file: !32, line: 167)
!139 = !DISubprogram(name: "rand", scope: !24, file: !24, line: 573, type: !140, flags: DIFlagPrototyped, spFlags: 0)
!140 = !DISubroutineType(types: !141)
!141 = !{!27}
!142 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !143, file: !32, line: 168)
!143 = !DISubprogram(name: "realloc", scope: !24, file: !24, line: 683, type: !144, flags: DIFlagPrototyped, spFlags: 0)
!144 = !DISubroutineType(types: !145)
!145 = !{!48, !48, !49}
!146 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !147, file: !32, line: 169)
!147 = !DISubprogram(name: "srand", scope: !24, file: !24, line: 575, type: !148, flags: DIFlagPrototyped, spFlags: 0)
!148 = !DISubroutineType(types: !149)
!149 = !{null, !150}
!150 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!151 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !152, file: !32, line: 170)
!152 = !DISubprogram(name: "strtod", scope: !24, file: !24, line: 118, type: !153, flags: DIFlagPrototyped, spFlags: 0)
!153 = !DISubroutineType(types: !154)
!154 = !{!63, !127, !155}
!155 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !156)
!156 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !103, size: 64)
!157 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !158, file: !32, line: 171)
!158 = !DISubprogram(name: "strtol", linkageName: "__isoc23_strtol", scope: !24, file: !24, line: 215, type: !159, flags: DIFlagPrototyped, spFlags: 0)
!159 = !DISubroutineType(types: !160)
!160 = !{!38, !127, !155, !27}
!161 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !162, file: !32, line: 172)
!162 = !DISubprogram(name: "strtoul", linkageName: "__isoc23_strtoul", scope: !24, file: !24, line: 219, type: !163, flags: DIFlagPrototyped, spFlags: 0)
!163 = !DISubroutineType(types: !164)
!164 = !{!51, !127, !155, !27}
!165 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !166, file: !32, line: 173)
!166 = !DISubprogram(name: "system", scope: !24, file: !24, line: 923, type: !69, flags: DIFlagPrototyped, spFlags: 0)
!167 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !168, file: !32, line: 175)
!168 = !DISubprogram(name: "wcstombs", scope: !24, file: !24, line: 1077, type: !169, flags: DIFlagPrototyped, spFlags: 0)
!169 = !DISubroutineType(types: !170)
!170 = !{!49, !171, !172, !49}
!171 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !103)
!172 = !DIDerivedType(tag: DW_TAG_restrict_type, baseType: !173)
!173 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !174, size: 64)
!174 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !126)
!175 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !176, file: !32, line: 176)
!176 = !DISubprogram(name: "wctomb", scope: !24, file: !24, line: 1069, type: !177, flags: DIFlagPrototyped, spFlags: 0)
!177 = !DISubroutineType(types: !178)
!178 = !{!27, !103, !126}
!179 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !181, file: !32, line: 204)
!180 = !DINamespace(name: "__gnu_cxx", scope: null)
!181 = !DIDerivedType(tag: DW_TAG_typedef, name: "lldiv_t", file: !24, line: 81, baseType: !182)
!182 = distinct !DICompositeType(tag: DW_TAG_structure_type, file: !24, line: 77, size: 128, flags: DIFlagTypePassByValue, elements: !183, identifier: "_ZTS7lldiv_t")
!183 = !{!184, !186}
!184 = !DIDerivedType(tag: DW_TAG_member, name: "quot", scope: !182, file: !24, line: 79, baseType: !185, size: 64)
!185 = !DIBasicType(name: "long long", size: 64, encoding: DW_ATE_signed)
!186 = !DIDerivedType(tag: DW_TAG_member, name: "rem", scope: !182, file: !24, line: 80, baseType: !185, size: 64, offset: 64)
!187 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !188, file: !32, line: 210)
!188 = !DISubprogram(name: "_Exit", scope: !24, file: !24, line: 768, type: !93, flags: DIFlagPrototyped | DIFlagNoReturn, spFlags: 0)
!189 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !190, file: !32, line: 214)
!190 = !DISubprogram(name: "llabs", scope: !24, file: !24, line: 984, type: !191, flags: DIFlagPrototyped, spFlags: 0)
!191 = !DISubroutineType(types: !192)
!192 = !{!185, !185}
!193 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !194, file: !32, line: 220)
!194 = !DISubprogram(name: "lldiv", scope: !24, file: !24, line: 998, type: !195, flags: DIFlagPrototyped, spFlags: 0)
!195 = !DISubroutineType(types: !196)
!196 = !{!181, !185, !185}
!197 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !198, file: !32, line: 231)
!198 = !DISubprogram(name: "atoll", scope: !24, file: !24, line: 113, type: !199, flags: DIFlagPrototyped, spFlags: 0)
!199 = !DISubroutineType(types: !200)
!200 = !{!185, !64}
!201 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !202, file: !32, line: 232)
!202 = !DISubprogram(name: "strtoll", linkageName: "__isoc23_strtoll", scope: !24, file: !24, line: 238, type: !203, flags: DIFlagPrototyped, spFlags: 0)
!203 = !DISubroutineType(types: !204)
!204 = !{!185, !127, !155, !27}
!205 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !206, file: !32, line: 233)
!206 = !DISubprogram(name: "strtoull", linkageName: "__isoc23_strtoull", scope: !24, file: !24, line: 243, type: !207, flags: DIFlagPrototyped, spFlags: 0)
!207 = !DISubroutineType(types: !208)
!208 = !{!209, !127, !155, !27}
!209 = !DIBasicType(name: "unsigned long long", size: 64, encoding: DW_ATE_unsigned)
!210 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !211, file: !32, line: 235)
!211 = !DISubprogram(name: "strtof", scope: !24, file: !24, line: 124, type: !212, flags: DIFlagPrototyped, spFlags: 0)
!212 = !DISubroutineType(types: !213)
!213 = !{!214, !127, !155}
!214 = !DIBasicType(name: "float", size: 32, encoding: DW_ATE_float)
!215 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !180, entity: !216, file: !32, line: 236)
!216 = !DISubprogram(name: "strtold", scope: !24, file: !24, line: 127, type: !217, flags: DIFlagPrototyped, spFlags: 0)
!217 = !DISubroutineType(types: !218)
!218 = !{!219, !127, !155}
!219 = !DIBasicType(name: "long double", size: 128, encoding: DW_ATE_float)
!220 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !181, file: !32, line: 244)
!221 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !188, file: !32, line: 246)
!222 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !190, file: !32, line: 248)
!223 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !224, file: !32, line: 249)
!224 = !DISubprogram(name: "div", linkageName: "_ZN9__gnu_cxx3divExx", scope: !180, file: !32, line: 217, type: !195, flags: DIFlagPrototyped, spFlags: 0)
!225 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !194, file: !32, line: 250)
!226 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !198, file: !32, line: 252)
!227 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !211, file: !32, line: 253)
!228 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !202, file: !32, line: 254)
!229 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !206, file: !32, line: 255)
!230 = !DIImportedEntity(tag: DW_TAG_imported_declaration, scope: !22, entity: !216, file: !32, line: 256)
!231 = !{i32 7, !"Dwarf Version", i32 5}
!232 = !{i32 2, !"Debug Info Version", i32 3}
!233 = !{i32 1, !"wchar_size", i32 4}
!234 = !{i32 8, !"PIC Level", i32 2}
!235 = !{i32 7, !"PIE Level", i32 2}
!236 = !{i32 7, !"uwtable", i32 2}
!237 = !{i32 7, !"frame-pointer", i32 1}
!238 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!239 = distinct !DISubprogram(name: "test_single_inheritance", linkageName: "_Z23test_single_inheritancev", scope: !1, file: !1, line: 24, type: !42, scopeLine: 24, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!240 = !{}
!241 = !DILocalVariable(name: "b", scope: !239, file: !1, line: 25, type: !242)
!242 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !243, size: 64)
!243 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Base", file: !1, line: 11, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !244, vtableHolder: !243, identifier: "_ZTS4Base")
!244 = !{!245, !248, !252, !255}
!245 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Base", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!246 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !247, size: 64)
!247 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "__vtbl_ptr_type", baseType: !140, size: 64)
!248 = !DISubprogram(name: "~Base", scope: !243, file: !1, line: 13, type: !249, scopeLine: 13, containingType: !243, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!249 = !DISubroutineType(types: !250)
!250 = !{null, !251}
!251 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !243, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!252 = !DISubprogram(name: "foo", linkageName: "_ZN4Base3fooEv", scope: !243, file: !1, line: 14, type: !253, scopeLine: 14, containingType: !243, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!253 = !DISubroutineType(types: !254)
!254 = !{!27, !251}
!255 = !DISubprogram(name: "bar", linkageName: "_ZN4Base3barEv", scope: !243, file: !1, line: 15, type: !253, scopeLine: 15, containingType: !243, virtualIndex: 3, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!256 = !DILocation(line: 25, column: 11, scope: !239)
!257 = !DILocation(line: 25, column: 15, scope: !239)
!258 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Derived", file: !1, line: 18, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !259, vtableHolder: !243, identifier: "_ZTS7Derived")
!259 = !{!260, !261}
!260 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !258, baseType: !243, flags: DIFlagPublic, extraData: i32 0)
!261 = !DISubprogram(name: "foo", linkageName: "_ZN7Derived3fooEv", scope: !258, file: !1, line: 20, type: !262, scopeLine: 20, containingType: !258, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!262 = !DISubroutineType(types: !263)
!263 = !{!27, !264}
!264 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !258, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!265 = !DILocation(line: 25, column: 19, scope: !239)
!266 = !DILocalVariable(name: "r", scope: !239, file: !1, line: 26, type: !27)
!267 = !DILocation(line: 26, column: 9, scope: !239)
!268 = !DILocation(line: 26, column: 13, scope: !239)
!269 = !DILocation(line: 26, column: 16, scope: !239)
!270 = !DILocation(line: 28, column: 12, scope: !239)
!271 = !DILocation(line: 28, column: 5, scope: !239)
!272 = !DILocation(line: 29, column: 1, scope: !239)
!273 = distinct !DISubprogram(name: "Derived", linkageName: "_ZN7DerivedC2Ev", scope: !258, file: !1, line: 18, type: !274, scopeLine: 18, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !276, retainedNodes: !240)
!274 = !DISubroutineType(types: !275)
!275 = !{null, !264}
!276 = !DISubprogram(name: "Derived", scope: !258, type: !274, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!277 = !DILocalVariable(name: "this", arg: 1, scope: !273, type: !278, flags: DIFlagArtificial | DIFlagObjectPointer)
!278 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !258, size: 64)
!279 = !DILocation(line: 0, scope: !273)
!280 = !DILocation(line: 18, column: 7, scope: !273)
!281 = distinct !DISubprogram(name: "test_deep_inheritance", linkageName: "_Z21test_deep_inheritancev", scope: !1, file: !1, line: 48, type: !42, scopeLine: 48, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!282 = !DILocalVariable(name: "p", scope: !281, file: !1, line: 49, type: !283)
!283 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !284, size: 64)
!284 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Level1", file: !1, line: 32, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !285, vtableHolder: !284, identifier: "_ZTS6Level1")
!285 = !{!286, !287, !291}
!286 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Level1", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!287 = !DISubprogram(name: "~Level1", scope: !284, file: !1, line: 34, type: !288, scopeLine: 34, containingType: !284, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!288 = !DISubroutineType(types: !289)
!289 = !{null, !290}
!290 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !284, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!291 = !DISubprogram(name: "method", linkageName: "_ZN6Level16methodEv", scope: !284, file: !1, line: 35, type: !292, scopeLine: 35, containingType: !284, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!292 = !DISubroutineType(types: !293)
!293 = !{!27, !290}
!294 = !DILocation(line: 49, column: 13, scope: !281)
!295 = !DILocation(line: 49, column: 17, scope: !281)
!296 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Level3", file: !1, line: 43, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !297, vtableHolder: !284, identifier: "_ZTS6Level3")
!297 = !{!298, !306}
!298 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !296, baseType: !299, flags: DIFlagPublic, extraData: i32 0)
!299 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Level2", file: !1, line: 38, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !300, vtableHolder: !284, identifier: "_ZTS6Level2")
!300 = !{!301, !302}
!301 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !299, baseType: !284, flags: DIFlagPublic, extraData: i32 0)
!302 = !DISubprogram(name: "method", linkageName: "_ZN6Level26methodEv", scope: !299, file: !1, line: 40, type: !303, scopeLine: 40, containingType: !299, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!303 = !DISubroutineType(types: !304)
!304 = !{!27, !305}
!305 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !299, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!306 = !DISubprogram(name: "method", linkageName: "_ZN6Level36methodEv", scope: !296, file: !1, line: 45, type: !307, scopeLine: 45, containingType: !296, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!307 = !DISubroutineType(types: !308)
!308 = !{!27, !309}
!309 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !296, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!310 = !DILocation(line: 49, column: 21, scope: !281)
!311 = !DILocalVariable(name: "r", scope: !281, file: !1, line: 50, type: !27)
!312 = !DILocation(line: 50, column: 9, scope: !281)
!313 = !DILocation(line: 50, column: 13, scope: !281)
!314 = !DILocation(line: 50, column: 16, scope: !281)
!315 = !DILocation(line: 52, column: 12, scope: !281)
!316 = !DILocation(line: 52, column: 5, scope: !281)
!317 = !DILocation(line: 53, column: 1, scope: !281)
!318 = distinct !DISubprogram(name: "Level3", linkageName: "_ZN6Level3C2Ev", scope: !296, file: !1, line: 43, type: !319, scopeLine: 43, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !321, retainedNodes: !240)
!319 = !DISubroutineType(types: !320)
!320 = !{null, !309}
!321 = !DISubprogram(name: "Level3", scope: !296, type: !319, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!322 = !DILocalVariable(name: "this", arg: 1, scope: !318, type: !323, flags: DIFlagArtificial | DIFlagObjectPointer)
!323 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !296, size: 64)
!324 = !DILocation(line: 0, scope: !318)
!325 = !DILocation(line: 43, column: 7, scope: !318)
!326 = distinct !DISubprogram(name: "test_multiple_derived", linkageName: "_Z21test_multiple_derivedi", scope: !1, file: !1, line: 77, type: !93, scopeLine: 77, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!327 = !DILocalVariable(name: "choice", arg: 1, scope: !326, file: !1, line: 77, type: !27)
!328 = !DILocation(line: 77, column: 32, scope: !326)
!329 = !DILocalVariable(name: "a", scope: !326, file: !1, line: 78, type: !330)
!330 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !331, size: 64)
!331 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Animal", file: !1, line: 56, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !332, vtableHolder: !331, identifier: "_ZTS6Animal")
!332 = !{!333, !334, !338}
!333 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Animal", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!334 = !DISubprogram(name: "~Animal", scope: !331, file: !1, line: 58, type: !335, scopeLine: 58, containingType: !331, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!335 = !DISubroutineType(types: !336)
!336 = !{null, !337}
!337 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !331, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!338 = !DISubprogram(name: "speak", linkageName: "_ZN6Animal5speakEv", scope: !331, file: !1, line: 59, type: !335, scopeLine: 59, containingType: !331, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!339 = !DILocation(line: 78, column: 13, scope: !326)
!340 = !DILocation(line: 79, column: 13, scope: !326)
!341 = !DILocation(line: 79, column: 5, scope: !326)
!342 = !DILocation(line: 80, column: 21, scope: !343)
!343 = distinct !DILexicalBlock(scope: !326, file: !1, line: 79, column: 21)
!344 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Dog", file: !1, line: 62, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !345, vtableHolder: !331, identifier: "_ZTS3Dog")
!345 = !{!346, !347}
!346 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !344, baseType: !331, flags: DIFlagPublic, extraData: i32 0)
!347 = !DISubprogram(name: "speak", linkageName: "_ZN3Dog5speakEv", scope: !344, file: !1, line: 64, type: !348, scopeLine: 64, containingType: !344, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!348 = !DISubroutineType(types: !349)
!349 = !{null, !350}
!350 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !344, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!351 = !DILocation(line: 80, column: 25, scope: !343)
!352 = !DILocation(line: 80, column: 19, scope: !343)
!353 = !DILocation(line: 80, column: 32, scope: !343)
!354 = !DILocation(line: 81, column: 21, scope: !343)
!355 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Cat", file: !1, line: 67, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !356, vtableHolder: !331, identifier: "_ZTS3Cat")
!356 = !{!357, !358}
!357 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !355, baseType: !331, flags: DIFlagPublic, extraData: i32 0)
!358 = !DISubprogram(name: "speak", linkageName: "_ZN3Cat5speakEv", scope: !355, file: !1, line: 69, type: !359, scopeLine: 69, containingType: !355, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!359 = !DISubroutineType(types: !360)
!360 = !{null, !361}
!361 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !355, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!362 = !DILocation(line: 81, column: 25, scope: !343)
!363 = !DILocation(line: 81, column: 19, scope: !343)
!364 = !DILocation(line: 81, column: 32, scope: !343)
!365 = !DILocation(line: 82, column: 22, scope: !343)
!366 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Bird", file: !1, line: 72, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !367, vtableHolder: !331, identifier: "_ZTS4Bird")
!367 = !{!368, !369}
!368 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !366, baseType: !331, flags: DIFlagPublic, extraData: i32 0)
!369 = !DISubprogram(name: "speak", linkageName: "_ZN4Bird5speakEv", scope: !366, file: !1, line: 74, type: !370, scopeLine: 74, containingType: !366, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!370 = !DISubroutineType(types: !371)
!371 = !{null, !372}
!372 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !366, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!373 = !DILocation(line: 82, column: 26, scope: !343)
!374 = !DILocation(line: 82, column: 20, scope: !343)
!375 = !DILocation(line: 82, column: 34, scope: !343)
!376 = !DILocation(line: 84, column: 5, scope: !326)
!377 = !DILocation(line: 84, column: 8, scope: !326)
!378 = !DILocation(line: 85, column: 12, scope: !326)
!379 = !DILocation(line: 85, column: 5, scope: !326)
!380 = !DILocation(line: 86, column: 1, scope: !326)
!381 = distinct !DISubprogram(name: "Dog", linkageName: "_ZN3DogC2Ev", scope: !344, file: !1, line: 62, type: !348, scopeLine: 62, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !382, retainedNodes: !240)
!382 = !DISubprogram(name: "Dog", scope: !344, type: !348, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!383 = !DILocalVariable(name: "this", arg: 1, scope: !381, type: !384, flags: DIFlagArtificial | DIFlagObjectPointer)
!384 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !344, size: 64)
!385 = !DILocation(line: 0, scope: !381)
!386 = !DILocation(line: 62, column: 7, scope: !381)
!387 = distinct !DISubprogram(name: "Cat", linkageName: "_ZN3CatC2Ev", scope: !355, file: !1, line: 67, type: !359, scopeLine: 67, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !388, retainedNodes: !240)
!388 = !DISubprogram(name: "Cat", scope: !355, type: !359, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!389 = !DILocalVariable(name: "this", arg: 1, scope: !387, type: !390, flags: DIFlagArtificial | DIFlagObjectPointer)
!390 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !355, size: 64)
!391 = !DILocation(line: 0, scope: !387)
!392 = !DILocation(line: 67, column: 7, scope: !387)
!393 = distinct !DISubprogram(name: "Bird", linkageName: "_ZN4BirdC2Ev", scope: !366, file: !1, line: 72, type: !370, scopeLine: 72, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !394, retainedNodes: !240)
!394 = !DISubprogram(name: "Bird", scope: !366, type: !370, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!395 = !DILocalVariable(name: "this", arg: 1, scope: !393, type: !396, flags: DIFlagArtificial | DIFlagObjectPointer)
!396 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !366, size: 64)
!397 = !DILocation(line: 0, scope: !393)
!398 = !DILocation(line: 72, column: 7, scope: !393)
!399 = distinct !DISubprogram(name: "test_multiple_inheritance", linkageName: "_Z25test_multiple_inheritancev", scope: !1, file: !1, line: 111, type: !42, scopeLine: 111, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!400 = !DILocalVariable(name: "p", scope: !399, file: !1, line: 112, type: !401)
!401 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !402, size: 64)
!402 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Printable", file: !1, line: 93, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !403, vtableHolder: !402, identifier: "_ZTS9Printable")
!403 = !{!404, !405, !409}
!404 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Printable", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!405 = !DISubprogram(name: "~Printable", scope: !402, file: !1, line: 95, type: !406, scopeLine: 95, containingType: !402, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!406 = !DISubroutineType(types: !407)
!407 = !{null, !408}
!408 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !402, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!409 = !DISubprogram(name: "print", linkageName: "_ZN9Printable5printEv", scope: !402, file: !1, line: 96, type: !406, scopeLine: 96, containingType: !402, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!410 = !DILocation(line: 112, column: 16, scope: !399)
!411 = !DILocation(line: 112, column: 20, scope: !399)
!412 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Document", file: !1, line: 105, size: 128, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !413, vtableHolder: !402, identifier: "_ZTS8Document")
!413 = !{!414, !415, !424, !428}
!414 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !412, baseType: !402, flags: DIFlagPublic, extraData: i32 0)
!415 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !412, baseType: !416, offset: 64, flags: DIFlagPublic, extraData: i32 0)
!416 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Serializable", file: !1, line: 99, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !417, vtableHolder: !416, identifier: "_ZTS12Serializable")
!417 = !{!418, !419, !423}
!418 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Serializable", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!419 = !DISubprogram(name: "~Serializable", scope: !416, file: !1, line: 101, type: !420, scopeLine: 101, containingType: !416, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!420 = !DISubroutineType(types: !421)
!421 = !{null, !422}
!422 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !416, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!423 = !DISubprogram(name: "serialize", linkageName: "_ZN12Serializable9serializeEv", scope: !416, file: !1, line: 102, type: !420, scopeLine: 102, containingType: !416, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!424 = !DISubprogram(name: "print", linkageName: "_ZN8Document5printEv", scope: !412, file: !1, line: 107, type: !425, scopeLine: 107, containingType: !412, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!425 = !DISubroutineType(types: !426)
!426 = !{null, !427}
!427 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !412, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!428 = !DISubprogram(name: "serialize", linkageName: "_ZN8Document9serializeEv", scope: !412, file: !1, line: 108, type: !425, scopeLine: 108, containingType: !412, virtualIndex: 3, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!429 = !DILocation(line: 112, column: 24, scope: !399)
!430 = !DILocation(line: 113, column: 5, scope: !399)
!431 = !DILocation(line: 113, column: 8, scope: !399)
!432 = !DILocation(line: 114, column: 12, scope: !399)
!433 = !DILocation(line: 114, column: 5, scope: !399)
!434 = !DILocalVariable(name: "s", scope: !399, file: !1, line: 116, type: !435)
!435 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !416, size: 64)
!436 = !DILocation(line: 116, column: 19, scope: !399)
!437 = !DILocation(line: 116, column: 23, scope: !399)
!438 = !DILocation(line: 116, column: 27, scope: !399)
!439 = !DILocation(line: 117, column: 5, scope: !399)
!440 = !DILocation(line: 117, column: 8, scope: !399)
!441 = !DILocation(line: 118, column: 12, scope: !399)
!442 = !DILocation(line: 118, column: 5, scope: !399)
!443 = !DILocation(line: 119, column: 1, scope: !399)
!444 = distinct !DISubprogram(name: "Document", linkageName: "_ZN8DocumentC2Ev", scope: !412, file: !1, line: 105, type: !425, scopeLine: 105, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !445, retainedNodes: !240)
!445 = !DISubprogram(name: "Document", scope: !412, type: !425, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!446 = !DILocalVariable(name: "this", arg: 1, scope: !444, type: !447, flags: DIFlagArtificial | DIFlagObjectPointer)
!447 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !412, size: 64)
!448 = !DILocation(line: 0, scope: !444)
!449 = !DILocation(line: 105, column: 7, scope: !444)
!450 = distinct !DISubprogram(name: "test_diamond_inheritance", linkageName: "_Z24test_diamond_inheritancev", scope: !1, file: !1, line: 144, type: !42, scopeLine: 144, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!451 = !DILocalVariable(name: "b", scope: !450, file: !1, line: 145, type: !452)
!452 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !453, size: 64)
!453 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "DiamondBase", file: !1, line: 122, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !454, vtableHolder: !453, identifier: "_ZTS11DiamondBase")
!454 = !{!455, !456, !460}
!455 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$DiamondBase", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!456 = !DISubprogram(name: "~DiamondBase", scope: !453, file: !1, line: 124, type: !457, scopeLine: 124, containingType: !453, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!457 = !DISubroutineType(types: !458)
!458 = !{null, !459}
!459 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !453, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!460 = !DISubprogram(name: "value", linkageName: "_ZN11DiamondBase5valueEv", scope: !453, file: !1, line: 125, type: !461, scopeLine: 125, containingType: !453, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!461 = !DISubroutineType(types: !462)
!462 = !{!27, !459}
!463 = !DILocation(line: 145, column: 18, scope: !450)
!464 = !DILocation(line: 145, column: 22, scope: !450)
!465 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "DiamondBottom", file: !1, line: 138, size: 128, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !466, vtableHolder: !468, identifier: "_ZTS13DiamondBottom")
!466 = !{!467, !475, !483}
!467 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !465, baseType: !468, flags: DIFlagPublic, extraData: i32 0)
!468 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "DiamondLeft", file: !1, line: 128, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !469, vtableHolder: !453, identifier: "_ZTS11DiamondLeft")
!469 = !{!470, !471}
!470 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !468, baseType: !453, offset: 40, flags: DIFlagPublic | DIFlagVirtual, extraData: i32 0)
!471 = !DISubprogram(name: "value", linkageName: "_ZN11DiamondLeft5valueEv", scope: !468, file: !1, line: 130, type: !472, scopeLine: 130, containingType: !468, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!472 = !DISubroutineType(types: !473)
!473 = !{!27, !474}
!474 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !468, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!475 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !465, baseType: !476, offset: 64, flags: DIFlagPublic, extraData: i32 0)
!476 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "DiamondRight", file: !1, line: 133, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !477, vtableHolder: !453, identifier: "_ZTS12DiamondRight")
!477 = !{!478, !479}
!478 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !476, baseType: !453, offset: 40, flags: DIFlagPublic | DIFlagVirtual, extraData: i32 0)
!479 = !DISubprogram(name: "value", linkageName: "_ZN12DiamondRight5valueEv", scope: !476, file: !1, line: 135, type: !480, scopeLine: 135, containingType: !476, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!480 = !DISubroutineType(types: !481)
!481 = !{!27, !482}
!482 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !476, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!483 = !DISubprogram(name: "value", linkageName: "_ZN13DiamondBottom5valueEv", scope: !465, file: !1, line: 141, type: !484, scopeLine: 141, containingType: !465, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!484 = !DISubroutineType(types: !485)
!485 = !{!27, !486}
!486 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !465, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!487 = !DILocation(line: 145, column: 26, scope: !450)
!488 = !DILocalVariable(name: "r", scope: !450, file: !1, line: 146, type: !27)
!489 = !DILocation(line: 146, column: 9, scope: !450)
!490 = !DILocation(line: 146, column: 13, scope: !450)
!491 = !DILocation(line: 146, column: 16, scope: !450)
!492 = !DILocation(line: 148, column: 12, scope: !450)
!493 = !DILocation(line: 148, column: 5, scope: !450)
!494 = !DILocation(line: 149, column: 1, scope: !450)
!495 = distinct !DISubprogram(name: "DiamondBottom", linkageName: "_ZN13DiamondBottomC1Ev", scope: !465, file: !1, line: 138, type: !496, scopeLine: 138, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !498, retainedNodes: !240)
!496 = !DISubroutineType(types: !497)
!497 = !{null, !486}
!498 = !DISubprogram(name: "DiamondBottom", scope: !465, type: !496, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!499 = !DILocalVariable(name: "this", arg: 1, scope: !495, type: !500, flags: DIFlagArtificial | DIFlagObjectPointer)
!500 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !465, size: 64)
!501 = !DILocation(line: 0, scope: !495)
!502 = !DILocation(line: 138, column: 7, scope: !495)
!503 = distinct !DISubprogram(name: "test_abstract_class", linkageName: "_Z19test_abstract_classv", scope: !1, file: !1, line: 179, type: !42, scopeLine: 179, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!504 = !DILocalVariable(name: "shapes", scope: !503, file: !1, line: 180, type: !505)
!505 = !DICompositeType(tag: DW_TAG_array_type, baseType: !506, size: 128, elements: !518)
!506 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !507, size: 64)
!507 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "AbstractShape", file: !1, line: 156, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !508, vtableHolder: !507, identifier: "_ZTS13AbstractShape")
!508 = !{!509, !510, !514, !517}
!509 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$AbstractShape", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!510 = !DISubprogram(name: "~AbstractShape", scope: !507, file: !1, line: 158, type: !511, scopeLine: 158, containingType: !507, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!511 = !DISubroutineType(types: !512)
!512 = !{null, !513}
!513 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !507, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!514 = !DISubprogram(name: "area", linkageName: "_ZN13AbstractShape4areaEv", scope: !507, file: !1, line: 159, type: !515, scopeLine: 159, containingType: !507, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagPureVirtual)
!515 = !DISubroutineType(types: !516)
!516 = !{!63, !513}
!517 = !DISubprogram(name: "perimeter", linkageName: "_ZN13AbstractShape9perimeterEv", scope: !507, file: !1, line: 160, type: !515, scopeLine: 160, containingType: !507, virtualIndex: 3, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagPureVirtual)
!518 = !{!519}
!519 = !DISubrange(count: 2)
!520 = !DILocation(line: 180, column: 20, scope: !503)
!521 = !DILocation(line: 181, column: 17, scope: !503)
!522 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Circle", file: !1, line: 163, size: 128, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !523, vtableHolder: !507, identifier: "_ZTS6Circle")
!523 = !{!524, !525, !526, !530, !533}
!524 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !522, baseType: !507, flags: DIFlagPublic, extraData: i32 0)
!525 = !DIDerivedType(tag: DW_TAG_member, name: "radius", scope: !522, file: !1, line: 164, baseType: !63, size: 64, offset: 64)
!526 = !DISubprogram(name: "Circle", scope: !522, file: !1, line: 166, type: !527, scopeLine: 166, flags: DIFlagPublic | DIFlagExplicit | DIFlagPrototyped, spFlags: 0)
!527 = !DISubroutineType(types: !528)
!528 = !{null, !529, !63}
!529 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !522, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!530 = !DISubprogram(name: "area", linkageName: "_ZN6Circle4areaEv", scope: !522, file: !1, line: 167, type: !531, scopeLine: 167, containingType: !522, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!531 = !DISubroutineType(types: !532)
!532 = !{!63, !529}
!533 = !DISubprogram(name: "perimeter", linkageName: "_ZN6Circle9perimeterEv", scope: !522, file: !1, line: 168, type: !531, scopeLine: 168, containingType: !522, virtualIndex: 3, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!534 = !DILocation(line: 181, column: 21, scope: !503)
!535 = !DILocation(line: 181, column: 5, scope: !503)
!536 = !DILocation(line: 181, column: 15, scope: !503)
!537 = !DILocation(line: 182, column: 17, scope: !503)
!538 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Rectangle", file: !1, line: 171, size: 192, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !539, vtableHolder: !507, identifier: "_ZTS9Rectangle")
!539 = !{!540, !541, !542, !543, !547, !550}
!540 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !538, baseType: !507, flags: DIFlagPublic, extraData: i32 0)
!541 = !DIDerivedType(tag: DW_TAG_member, name: "width", scope: !538, file: !1, line: 172, baseType: !63, size: 64, offset: 64)
!542 = !DIDerivedType(tag: DW_TAG_member, name: "height", scope: !538, file: !1, line: 172, baseType: !63, size: 64, offset: 128)
!543 = !DISubprogram(name: "Rectangle", scope: !538, file: !1, line: 174, type: !544, scopeLine: 174, flags: DIFlagPublic | DIFlagPrototyped, spFlags: 0)
!544 = !DISubroutineType(types: !545)
!545 = !{null, !546, !63, !63}
!546 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !538, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!547 = !DISubprogram(name: "area", linkageName: "_ZN9Rectangle4areaEv", scope: !538, file: !1, line: 175, type: !548, scopeLine: 175, containingType: !538, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!548 = !DISubroutineType(types: !549)
!549 = !{!63, !546}
!550 = !DISubprogram(name: "perimeter", linkageName: "_ZN9Rectangle9perimeterEv", scope: !538, file: !1, line: 176, type: !548, scopeLine: 176, containingType: !538, virtualIndex: 3, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!551 = !DILocation(line: 182, column: 21, scope: !503)
!552 = !DILocation(line: 182, column: 5, scope: !503)
!553 = !DILocation(line: 182, column: 15, scope: !503)
!554 = !DILocalVariable(name: "i", scope: !555, file: !1, line: 184, type: !27)
!555 = distinct !DILexicalBlock(scope: !503, file: !1, line: 184, column: 5)
!556 = !DILocation(line: 184, column: 14, scope: !555)
!557 = !DILocation(line: 184, column: 10, scope: !555)
!558 = !DILocation(line: 184, column: 21, scope: !559)
!559 = distinct !DILexicalBlock(scope: !555, file: !1, line: 184, column: 5)
!560 = !DILocation(line: 184, column: 23, scope: !559)
!561 = !DILocation(line: 184, column: 5, scope: !555)
!562 = !DILocalVariable(name: "a", scope: !563, file: !1, line: 185, type: !63)
!563 = distinct !DILexicalBlock(scope: !559, file: !1, line: 184, column: 33)
!564 = !DILocation(line: 185, column: 16, scope: !563)
!565 = !DILocation(line: 185, column: 27, scope: !563)
!566 = !DILocation(line: 185, column: 20, scope: !563)
!567 = !DILocation(line: 185, column: 31, scope: !563)
!568 = !DILocation(line: 187, column: 5, scope: !563)
!569 = !DILocation(line: 184, column: 29, scope: !559)
!570 = !DILocation(line: 184, column: 5, scope: !559)
!571 = distinct !{!571, !561, !572, !573}
!572 = !DILocation(line: 187, column: 5, scope: !555)
!573 = !{!"llvm.loop.mustprogress"}
!574 = !DILocation(line: 191, column: 1, scope: !503)
!575 = !DILocation(line: 189, column: 12, scope: !503)
!576 = !DILocation(line: 189, column: 5, scope: !503)
!577 = !DILocation(line: 190, column: 12, scope: !503)
!578 = !DILocation(line: 190, column: 5, scope: !503)
!579 = distinct !DISubprogram(name: "Circle", linkageName: "_ZN6CircleC2Ed", scope: !522, file: !1, line: 166, type: !527, scopeLine: 166, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !526, retainedNodes: !240)
!580 = !DILocalVariable(name: "this", arg: 1, scope: !579, type: !581, flags: DIFlagArtificial | DIFlagObjectPointer)
!581 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !522, size: 64)
!582 = !DILocation(line: 0, scope: !579)
!583 = !DILocalVariable(name: "r", arg: 2, scope: !579, file: !1, line: 166, type: !63)
!584 = !DILocation(line: 166, column: 28, scope: !579)
!585 = !DILocation(line: 166, column: 14, scope: !579)
!586 = !DILocation(line: 166, column: 43, scope: !579)
!587 = !DILocation(line: 166, column: 33, scope: !579)
!588 = !DILocation(line: 166, column: 40, scope: !579)
!589 = !DILocation(line: 166, column: 44, scope: !579)
!590 = distinct !DISubprogram(name: "Rectangle", linkageName: "_ZN9RectangleC2Edd", scope: !538, file: !1, line: 174, type: !544, scopeLine: 174, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !543, retainedNodes: !240)
!591 = !DILocalVariable(name: "this", arg: 1, scope: !590, type: !592, flags: DIFlagArtificial | DIFlagObjectPointer)
!592 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !538, size: 64)
!593 = !DILocation(line: 0, scope: !590)
!594 = !DILocalVariable(name: "w", arg: 2, scope: !590, file: !1, line: 174, type: !63)
!595 = !DILocation(line: 174, column: 22, scope: !590)
!596 = !DILocalVariable(name: "h", arg: 3, scope: !590, file: !1, line: 174, type: !63)
!597 = !DILocation(line: 174, column: 32, scope: !590)
!598 = !DILocation(line: 174, column: 5, scope: !590)
!599 = !DILocation(line: 174, column: 57, scope: !590)
!600 = !DILocation(line: 174, column: 37, scope: !590)
!601 = !DILocation(line: 174, column: 43, scope: !590)
!602 = !DILocation(line: 174, column: 47, scope: !590)
!603 = !DILocation(line: 174, column: 54, scope: !590)
!604 = !DILocation(line: 174, column: 58, scope: !590)
!605 = distinct !DISubprogram(name: "test_interface_pattern", linkageName: "_Z22test_interface_patternv", scope: !1, file: !1, line: 214, type: !42, scopeLine: 214, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!606 = !DILocalVariable(name: "observers", scope: !605, file: !1, line: 215, type: !607)
!607 = !DICompositeType(tag: DW_TAG_array_type, baseType: !608, size: 128, elements: !518)
!608 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !609, size: 64)
!609 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "IObserver", file: !1, line: 198, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !610, vtableHolder: !609, identifier: "_ZTS9IObserver")
!610 = !{!611, !612, !616}
!611 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$IObserver", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!612 = !DISubprogram(name: "~IObserver", scope: !609, file: !1, line: 200, type: !613, scopeLine: 200, containingType: !609, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!613 = !DISubroutineType(types: !614)
!614 = !{null, !615}
!615 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !609, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!616 = !DISubprogram(name: "update", linkageName: "_ZN9IObserver6updateEi", scope: !609, file: !1, line: 201, type: !617, scopeLine: 201, containingType: !609, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagPureVirtual)
!617 = !DISubroutineType(types: !618)
!618 = !{null, !615, !27}
!619 = !DILocation(line: 215, column: 16, scope: !605)
!620 = !DILocation(line: 216, column: 20, scope: !605)
!621 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "ConcreteObserverA", file: !1, line: 204, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !622, vtableHolder: !609, identifier: "_ZTS17ConcreteObserverA")
!622 = !{!623, !624}
!623 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !621, baseType: !609, flags: DIFlagPublic, extraData: i32 0)
!624 = !DISubprogram(name: "update", linkageName: "_ZN17ConcreteObserverA6updateEi", scope: !621, file: !1, line: 206, type: !625, scopeLine: 206, containingType: !621, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!625 = !DISubroutineType(types: !626)
!626 = !{null, !627, !27}
!627 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !621, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!628 = !DILocation(line: 216, column: 24, scope: !605)
!629 = !DILocation(line: 216, column: 5, scope: !605)
!630 = !DILocation(line: 216, column: 18, scope: !605)
!631 = !DILocation(line: 217, column: 20, scope: !605)
!632 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "ConcreteObserverB", file: !1, line: 209, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !633, vtableHolder: !609, identifier: "_ZTS17ConcreteObserverB")
!633 = !{!634, !635}
!634 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !632, baseType: !609, flags: DIFlagPublic, extraData: i32 0)
!635 = !DISubprogram(name: "update", linkageName: "_ZN17ConcreteObserverB6updateEi", scope: !632, file: !1, line: 211, type: !636, scopeLine: 211, containingType: !632, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!636 = !DISubroutineType(types: !637)
!637 = !{null, !638, !27}
!638 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !632, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!639 = !DILocation(line: 217, column: 24, scope: !605)
!640 = !DILocation(line: 217, column: 5, scope: !605)
!641 = !DILocation(line: 217, column: 18, scope: !605)
!642 = !DILocalVariable(name: "i", scope: !643, file: !1, line: 219, type: !27)
!643 = distinct !DILexicalBlock(scope: !605, file: !1, line: 219, column: 5)
!644 = !DILocation(line: 219, column: 14, scope: !643)
!645 = !DILocation(line: 219, column: 10, scope: !643)
!646 = !DILocation(line: 219, column: 21, scope: !647)
!647 = distinct !DILexicalBlock(scope: !643, file: !1, line: 219, column: 5)
!648 = !DILocation(line: 219, column: 23, scope: !647)
!649 = !DILocation(line: 219, column: 5, scope: !643)
!650 = !DILocation(line: 220, column: 19, scope: !651)
!651 = distinct !DILexicalBlock(scope: !647, file: !1, line: 219, column: 33)
!652 = !DILocation(line: 220, column: 9, scope: !651)
!653 = !DILocation(line: 220, column: 23, scope: !651)
!654 = !DILocation(line: 221, column: 5, scope: !651)
!655 = !DILocation(line: 219, column: 29, scope: !647)
!656 = !DILocation(line: 219, column: 5, scope: !647)
!657 = distinct !{!657, !649, !658, !573}
!658 = !DILocation(line: 221, column: 5, scope: !643)
!659 = !DILocation(line: 223, column: 12, scope: !605)
!660 = !DILocation(line: 223, column: 5, scope: !605)
!661 = !DILocation(line: 224, column: 12, scope: !605)
!662 = !DILocation(line: 224, column: 5, scope: !605)
!663 = !DILocation(line: 225, column: 1, scope: !605)
!664 = distinct !DISubprogram(name: "ConcreteObserverA", linkageName: "_ZN17ConcreteObserverAC2Ev", scope: !621, file: !1, line: 204, type: !665, scopeLine: 204, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !667, retainedNodes: !240)
!665 = !DISubroutineType(types: !666)
!666 = !{null, !627}
!667 = !DISubprogram(name: "ConcreteObserverA", scope: !621, type: !665, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!668 = !DILocalVariable(name: "this", arg: 1, scope: !664, type: !669, flags: DIFlagArtificial | DIFlagObjectPointer)
!669 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !621, size: 64)
!670 = !DILocation(line: 0, scope: !664)
!671 = !DILocation(line: 204, column: 7, scope: !664)
!672 = distinct !DISubprogram(name: "ConcreteObserverB", linkageName: "_ZN17ConcreteObserverBC2Ev", scope: !632, file: !1, line: 209, type: !673, scopeLine: 209, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !675, retainedNodes: !240)
!673 = !DISubroutineType(types: !674)
!674 = !{null, !638}
!675 = !DISubprogram(name: "ConcreteObserverB", scope: !632, type: !673, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!676 = !DILocalVariable(name: "this", arg: 1, scope: !672, type: !677, flags: DIFlagArtificial | DIFlagObjectPointer)
!677 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !632, size: 64)
!678 = !DILocation(line: 0, scope: !672)
!679 = !DILocation(line: 209, column: 7, scope: !672)
!680 = distinct !DISubprogram(name: "test_crtp", linkageName: "_Z9test_crtpv", scope: !1, file: !1, line: 247, type: !42, scopeLine: 247, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!681 = !DILocalVariable(name: "d", scope: !680, file: !1, line: 248, type: !4)
!682 = !DILocation(line: 248, column: 17, scope: !680)
!683 = !DILocation(line: 249, column: 7, scope: !680)
!684 = !DILocation(line: 250, column: 1, scope: !680)
!685 = distinct !DISubprogram(name: "interface", linkageName: "_ZN8CRTPBaseI11CRTPDerivedE9interfaceEv", scope: !7, file: !1, line: 235, type: !10, scopeLine: 235, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !9, retainedNodes: !240)
!686 = !DILocalVariable(name: "this", arg: 1, scope: !685, type: !687, flags: DIFlagArtificial | DIFlagObjectPointer)
!687 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !7, size: 64)
!688 = !DILocation(line: 0, scope: !685)
!689 = !DILocation(line: 236, column: 38, scope: !685)
!690 = !DILocation(line: 237, column: 5, scope: !685)
!691 = distinct !DISubprogram(name: "test_prototype_pattern", linkageName: "_Z22test_prototype_patternv", scope: !1, file: !1, line: 273, type: !42, scopeLine: 273, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!692 = !DILocalVariable(name: "p", scope: !691, file: !1, line: 274, type: !693)
!693 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !694, size: 64)
!694 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "Prototype", file: !1, line: 257, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !695, vtableHolder: !694, identifier: "_ZTS9Prototype")
!695 = !{!696, !697, !701}
!696 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$Prototype", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!697 = !DISubprogram(name: "~Prototype", scope: !694, file: !1, line: 259, type: !698, scopeLine: 259, containingType: !694, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!698 = !DISubroutineType(types: !699)
!699 = !{null, !700}
!700 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !694, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!701 = !DISubprogram(name: "clone", linkageName: "_ZN9Prototype5cloneEv", scope: !694, file: !1, line: 260, type: !702, scopeLine: 260, containingType: !694, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagPureVirtual)
!702 = !DISubroutineType(types: !703)
!703 = !{!693, !700}
!704 = !DILocation(line: 274, column: 16, scope: !691)
!705 = !DILocation(line: 274, column: 20, scope: !691)
!706 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "ConcretePrototype1", file: !1, line: 263, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !707, vtableHolder: !694, identifier: "_ZTS18ConcretePrototype1")
!707 = !{!708, !709}
!708 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !706, baseType: !694, flags: DIFlagPublic, extraData: i32 0)
!709 = !DISubprogram(name: "clone", linkageName: "_ZN18ConcretePrototype15cloneEv", scope: !706, file: !1, line: 265, type: !710, scopeLine: 265, containingType: !706, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!710 = !DISubroutineType(types: !711)
!711 = !{!693, !712}
!712 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !706, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!713 = !DILocation(line: 274, column: 24, scope: !691)
!714 = !DILocalVariable(name: "copy", scope: !691, file: !1, line: 275, type: !693)
!715 = !DILocation(line: 275, column: 16, scope: !691)
!716 = !DILocation(line: 275, column: 23, scope: !691)
!717 = !DILocation(line: 275, column: 26, scope: !691)
!718 = !DILocation(line: 276, column: 12, scope: !691)
!719 = !DILocation(line: 276, column: 5, scope: !691)
!720 = !DILocation(line: 277, column: 12, scope: !691)
!721 = !DILocation(line: 277, column: 5, scope: !691)
!722 = !DILocation(line: 278, column: 1, scope: !691)
!723 = distinct !DISubprogram(name: "ConcretePrototype1", linkageName: "_ZN18ConcretePrototype1C2Ev", scope: !706, file: !1, line: 263, type: !724, scopeLine: 263, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !726, retainedNodes: !240)
!724 = !DISubroutineType(types: !725)
!725 = !{null, !712}
!726 = !DISubprogram(name: "ConcretePrototype1", scope: !706, type: !724, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!727 = !DILocalVariable(name: "this", arg: 1, scope: !723, type: !728, flags: DIFlagArtificial | DIFlagObjectPointer)
!728 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !706, size: 64)
!729 = !DILocation(line: 0, scope: !723)
!730 = !DILocation(line: 263, column: 7, scope: !723)
!731 = distinct !DISubprogram(name: "test_covariant_return", linkageName: "_Z21test_covariant_returnv", scope: !1, file: !1, line: 297, type: !42, scopeLine: 297, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !240)
!732 = !DILocalVariable(name: "b", scope: !731, file: !1, line: 298, type: !733)
!733 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !734, size: 64)
!734 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "CloneableBase", file: !1, line: 285, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !735, vtableHolder: !734, identifier: "_ZTS13CloneableBase")
!735 = !{!736, !737, !741}
!736 = !DIDerivedType(tag: DW_TAG_member, name: "_vptr$CloneableBase", scope: !1, file: !1, baseType: !246, size: 64, flags: DIFlagArtificial)
!737 = !DISubprogram(name: "~CloneableBase", scope: !734, file: !1, line: 287, type: !738, scopeLine: 287, containingType: !734, virtualIndex: 0, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!738 = !DISubroutineType(types: !739)
!739 = !{null, !740}
!740 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !734, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!741 = !DISubprogram(name: "clone_self", linkageName: "_ZN13CloneableBase10clone_selfEv", scope: !734, file: !1, line: 288, type: !742, scopeLine: 288, containingType: !734, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!742 = !DISubroutineType(types: !743)
!743 = !{!733, !740}
!744 = !DILocation(line: 298, column: 20, scope: !731)
!745 = !DILocation(line: 298, column: 24, scope: !731)
!746 = distinct !DICompositeType(tag: DW_TAG_class_type, name: "CloneableDerived", file: !1, line: 291, size: 64, flags: DIFlagTypePassByReference | DIFlagNonTrivial, elements: !747, vtableHolder: !734, identifier: "_ZTS16CloneableDerived")
!747 = !{!748, !749}
!748 = !DIDerivedType(tag: DW_TAG_inheritance, scope: !746, baseType: !734, flags: DIFlagPublic, extraData: i32 0)
!749 = !DISubprogram(name: "clone_self", linkageName: "_ZN16CloneableDerived10clone_selfEv", scope: !746, file: !1, line: 294, type: !750, scopeLine: 294, containingType: !746, virtualIndex: 2, flags: DIFlagPublic | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!750 = !DISubroutineType(types: !751)
!751 = !{!752, !753}
!752 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !746, size: 64)
!753 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !746, size: 64, flags: DIFlagArtificial | DIFlagObjectPointer)
!754 = !DILocation(line: 298, column: 28, scope: !731)
!755 = !DILocalVariable(name: "copy", scope: !731, file: !1, line: 299, type: !733)
!756 = !DILocation(line: 299, column: 20, scope: !731)
!757 = !DILocation(line: 299, column: 27, scope: !731)
!758 = !DILocation(line: 299, column: 30, scope: !731)
!759 = !DILocation(line: 300, column: 12, scope: !731)
!760 = !DILocation(line: 300, column: 5, scope: !731)
!761 = !DILocation(line: 301, column: 12, scope: !731)
!762 = !DILocation(line: 301, column: 5, scope: !731)
!763 = !DILocation(line: 302, column: 1, scope: !731)
!764 = distinct !DISubprogram(name: "CloneableDerived", linkageName: "_ZN16CloneableDerivedC2Ev", scope: !746, file: !1, line: 291, type: !765, scopeLine: 291, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !767, retainedNodes: !240)
!765 = !DISubroutineType(types: !766)
!766 = !{null, !753}
!767 = !DISubprogram(name: "CloneableDerived", scope: !746, type: !765, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!768 = !DILocalVariable(name: "this", arg: 1, scope: !764, type: !752, flags: DIFlagArtificial | DIFlagObjectPointer)
!769 = !DILocation(line: 0, scope: !764)
!770 = !DILocation(line: 291, column: 7, scope: !764)
!771 = distinct !DISubprogram(name: "main", scope: !1, file: !1, line: 304, type: !140, scopeLine: 304, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)
!772 = !DILocation(line: 305, column: 5, scope: !771)
!773 = !DILocation(line: 306, column: 5, scope: !771)
!774 = !DILocation(line: 307, column: 5, scope: !771)
!775 = !DILocation(line: 308, column: 5, scope: !771)
!776 = !DILocation(line: 309, column: 5, scope: !771)
!777 = !DILocation(line: 310, column: 5, scope: !771)
!778 = !DILocation(line: 311, column: 5, scope: !771)
!779 = !DILocation(line: 312, column: 5, scope: !771)
!780 = !DILocation(line: 313, column: 5, scope: !771)
!781 = !DILocation(line: 314, column: 5, scope: !771)
!782 = !DILocation(line: 315, column: 5, scope: !771)
!783 = distinct !DISubprogram(name: "Base", linkageName: "_ZN4BaseC2Ev", scope: !243, file: !1, line: 11, type: !249, scopeLine: 11, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !784, retainedNodes: !240)
!784 = !DISubprogram(name: "Base", scope: !243, type: !249, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!785 = !DILocalVariable(name: "this", arg: 1, scope: !783, type: !242, flags: DIFlagArtificial | DIFlagObjectPointer)
!786 = !DILocation(line: 0, scope: !783)
!787 = !DILocation(line: 11, column: 7, scope: !783)
!788 = distinct !DISubprogram(name: "~Derived", linkageName: "_ZN7DerivedD2Ev", scope: !258, file: !1, line: 18, type: !274, scopeLine: 18, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !789, retainedNodes: !240)
!789 = !DISubprogram(name: "~Derived", scope: !258, type: !274, containingType: !258, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!790 = !DILocalVariable(name: "this", arg: 1, scope: !788, type: !278, flags: DIFlagArtificial | DIFlagObjectPointer)
!791 = !DILocation(line: 0, scope: !788)
!792 = !DILocation(line: 18, column: 7, scope: !793)
!793 = distinct !DILexicalBlock(scope: !788, file: !1, line: 18, column: 7)
!794 = !DILocation(line: 18, column: 7, scope: !788)
!795 = distinct !DISubprogram(name: "~Derived", linkageName: "_ZN7DerivedD0Ev", scope: !258, file: !1, line: 18, type: !274, scopeLine: 18, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !789, retainedNodes: !240)
!796 = !DILocalVariable(name: "this", arg: 1, scope: !795, type: !278, flags: DIFlagArtificial | DIFlagObjectPointer)
!797 = !DILocation(line: 0, scope: !795)
!798 = !DILocation(line: 18, column: 7, scope: !795)
!799 = distinct !DISubprogram(name: "foo", linkageName: "_ZN7Derived3fooEv", scope: !258, file: !1, line: 20, type: !262, scopeLine: 20, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !261, retainedNodes: !240)
!800 = !DILocalVariable(name: "this", arg: 1, scope: !799, type: !278, flags: DIFlagArtificial | DIFlagObjectPointer)
!801 = !DILocation(line: 0, scope: !799)
!802 = !DILocation(line: 20, column: 26, scope: !799)
!803 = distinct !DISubprogram(name: "bar", linkageName: "_ZN4Base3barEv", scope: !243, file: !1, line: 15, type: !253, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !255, retainedNodes: !240)
!804 = !DILocalVariable(name: "this", arg: 1, scope: !803, type: !242, flags: DIFlagArtificial | DIFlagObjectPointer)
!805 = !DILocation(line: 0, scope: !803)
!806 = !DILocation(line: 15, column: 25, scope: !803)
!807 = distinct !DISubprogram(name: "~Base", linkageName: "_ZN4BaseD2Ev", scope: !243, file: !1, line: 13, type: !249, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !248, retainedNodes: !240)
!808 = !DILocalVariable(name: "this", arg: 1, scope: !807, type: !242, flags: DIFlagArtificial | DIFlagObjectPointer)
!809 = !DILocation(line: 0, scope: !807)
!810 = !DILocation(line: 13, column: 29, scope: !807)
!811 = distinct !DISubprogram(name: "~Base", linkageName: "_ZN4BaseD0Ev", scope: !243, file: !1, line: 13, type: !249, scopeLine: 13, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !248, retainedNodes: !240)
!812 = !DILocalVariable(name: "this", arg: 1, scope: !811, type: !242, flags: DIFlagArtificial | DIFlagObjectPointer)
!813 = !DILocation(line: 0, scope: !811)
!814 = !DILocation(line: 13, column: 29, scope: !811)
!815 = distinct !DISubprogram(name: "foo", linkageName: "_ZN4Base3fooEv", scope: !243, file: !1, line: 14, type: !253, scopeLine: 14, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !252, retainedNodes: !240)
!816 = !DILocalVariable(name: "this", arg: 1, scope: !815, type: !242, flags: DIFlagArtificial | DIFlagObjectPointer)
!817 = !DILocation(line: 0, scope: !815)
!818 = !DILocation(line: 14, column: 25, scope: !815)
!819 = distinct !DISubprogram(name: "Level2", linkageName: "_ZN6Level2C2Ev", scope: !299, file: !1, line: 38, type: !820, scopeLine: 38, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !822, retainedNodes: !240)
!820 = !DISubroutineType(types: !821)
!821 = !{null, !305}
!822 = !DISubprogram(name: "Level2", scope: !299, type: !820, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!823 = !DILocalVariable(name: "this", arg: 1, scope: !819, type: !824, flags: DIFlagArtificial | DIFlagObjectPointer)
!824 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !299, size: 64)
!825 = !DILocation(line: 0, scope: !819)
!826 = !DILocation(line: 38, column: 7, scope: !819)
!827 = distinct !DISubprogram(name: "~Level3", linkageName: "_ZN6Level3D2Ev", scope: !296, file: !1, line: 43, type: !319, scopeLine: 43, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !828, retainedNodes: !240)
!828 = !DISubprogram(name: "~Level3", scope: !296, type: !319, containingType: !296, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!829 = !DILocalVariable(name: "this", arg: 1, scope: !827, type: !323, flags: DIFlagArtificial | DIFlagObjectPointer)
!830 = !DILocation(line: 0, scope: !827)
!831 = !DILocation(line: 43, column: 7, scope: !832)
!832 = distinct !DILexicalBlock(scope: !827, file: !1, line: 43, column: 7)
!833 = !DILocation(line: 43, column: 7, scope: !827)
!834 = distinct !DISubprogram(name: "~Level3", linkageName: "_ZN6Level3D0Ev", scope: !296, file: !1, line: 43, type: !319, scopeLine: 43, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !828, retainedNodes: !240)
!835 = !DILocalVariable(name: "this", arg: 1, scope: !834, type: !323, flags: DIFlagArtificial | DIFlagObjectPointer)
!836 = !DILocation(line: 0, scope: !834)
!837 = !DILocation(line: 43, column: 7, scope: !834)
!838 = distinct !DISubprogram(name: "method", linkageName: "_ZN6Level36methodEv", scope: !296, file: !1, line: 45, type: !307, scopeLine: 45, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !306, retainedNodes: !240)
!839 = !DILocalVariable(name: "this", arg: 1, scope: !838, type: !323, flags: DIFlagArtificial | DIFlagObjectPointer)
!840 = !DILocation(line: 0, scope: !838)
!841 = !DILocation(line: 45, column: 29, scope: !838)
!842 = distinct !DISubprogram(name: "Level1", linkageName: "_ZN6Level1C2Ev", scope: !284, file: !1, line: 32, type: !288, scopeLine: 32, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !843, retainedNodes: !240)
!843 = !DISubprogram(name: "Level1", scope: !284, type: !288, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!844 = !DILocalVariable(name: "this", arg: 1, scope: !842, type: !283, flags: DIFlagArtificial | DIFlagObjectPointer)
!845 = !DILocation(line: 0, scope: !842)
!846 = !DILocation(line: 32, column: 7, scope: !842)
!847 = distinct !DISubprogram(name: "~Level2", linkageName: "_ZN6Level2D2Ev", scope: !299, file: !1, line: 38, type: !820, scopeLine: 38, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !848, retainedNodes: !240)
!848 = !DISubprogram(name: "~Level2", scope: !299, type: !820, containingType: !299, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!849 = !DILocalVariable(name: "this", arg: 1, scope: !847, type: !824, flags: DIFlagArtificial | DIFlagObjectPointer)
!850 = !DILocation(line: 0, scope: !847)
!851 = !DILocation(line: 38, column: 7, scope: !852)
!852 = distinct !DILexicalBlock(scope: !847, file: !1, line: 38, column: 7)
!853 = !DILocation(line: 38, column: 7, scope: !847)
!854 = distinct !DISubprogram(name: "~Level2", linkageName: "_ZN6Level2D0Ev", scope: !299, file: !1, line: 38, type: !820, scopeLine: 38, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !848, retainedNodes: !240)
!855 = !DILocalVariable(name: "this", arg: 1, scope: !854, type: !824, flags: DIFlagArtificial | DIFlagObjectPointer)
!856 = !DILocation(line: 0, scope: !854)
!857 = !DILocation(line: 38, column: 7, scope: !854)
!858 = distinct !DISubprogram(name: "method", linkageName: "_ZN6Level26methodEv", scope: !299, file: !1, line: 40, type: !303, scopeLine: 40, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !302, retainedNodes: !240)
!859 = !DILocalVariable(name: "this", arg: 1, scope: !858, type: !824, flags: DIFlagArtificial | DIFlagObjectPointer)
!860 = !DILocation(line: 0, scope: !858)
!861 = !DILocation(line: 40, column: 29, scope: !858)
!862 = distinct !DISubprogram(name: "~Level1", linkageName: "_ZN6Level1D2Ev", scope: !284, file: !1, line: 34, type: !288, scopeLine: 34, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !287, retainedNodes: !240)
!863 = !DILocalVariable(name: "this", arg: 1, scope: !862, type: !283, flags: DIFlagArtificial | DIFlagObjectPointer)
!864 = !DILocation(line: 0, scope: !862)
!865 = !DILocation(line: 34, column: 31, scope: !862)
!866 = distinct !DISubprogram(name: "~Level1", linkageName: "_ZN6Level1D0Ev", scope: !284, file: !1, line: 34, type: !288, scopeLine: 34, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !287, retainedNodes: !240)
!867 = !DILocalVariable(name: "this", arg: 1, scope: !866, type: !283, flags: DIFlagArtificial | DIFlagObjectPointer)
!868 = !DILocation(line: 0, scope: !866)
!869 = !DILocation(line: 34, column: 31, scope: !866)
!870 = distinct !DISubprogram(name: "method", linkageName: "_ZN6Level16methodEv", scope: !284, file: !1, line: 35, type: !292, scopeLine: 35, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !291, retainedNodes: !240)
!871 = !DILocalVariable(name: "this", arg: 1, scope: !870, type: !283, flags: DIFlagArtificial | DIFlagObjectPointer)
!872 = !DILocation(line: 0, scope: !870)
!873 = !DILocation(line: 35, column: 28, scope: !870)
!874 = distinct !DISubprogram(name: "Animal", linkageName: "_ZN6AnimalC2Ev", scope: !331, file: !1, line: 56, type: !335, scopeLine: 56, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !875, retainedNodes: !240)
!875 = !DISubprogram(name: "Animal", scope: !331, type: !335, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!876 = !DILocalVariable(name: "this", arg: 1, scope: !874, type: !330, flags: DIFlagArtificial | DIFlagObjectPointer)
!877 = !DILocation(line: 0, scope: !874)
!878 = !DILocation(line: 56, column: 7, scope: !874)
!879 = distinct !DISubprogram(name: "~Dog", linkageName: "_ZN3DogD2Ev", scope: !344, file: !1, line: 62, type: !348, scopeLine: 62, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !880, retainedNodes: !240)
!880 = !DISubprogram(name: "~Dog", scope: !344, type: !348, containingType: !344, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!881 = !DILocalVariable(name: "this", arg: 1, scope: !879, type: !384, flags: DIFlagArtificial | DIFlagObjectPointer)
!882 = !DILocation(line: 0, scope: !879)
!883 = !DILocation(line: 62, column: 7, scope: !884)
!884 = distinct !DILexicalBlock(scope: !879, file: !1, line: 62, column: 7)
!885 = !DILocation(line: 62, column: 7, scope: !879)
!886 = distinct !DISubprogram(name: "~Dog", linkageName: "_ZN3DogD0Ev", scope: !344, file: !1, line: 62, type: !348, scopeLine: 62, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !880, retainedNodes: !240)
!887 = !DILocalVariable(name: "this", arg: 1, scope: !886, type: !384, flags: DIFlagArtificial | DIFlagObjectPointer)
!888 = !DILocation(line: 0, scope: !886)
!889 = !DILocation(line: 62, column: 7, scope: !886)
!890 = distinct !DISubprogram(name: "speak", linkageName: "_ZN3Dog5speakEv", scope: !344, file: !1, line: 64, type: !348, scopeLine: 64, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !347, retainedNodes: !240)
!891 = !DILocalVariable(name: "this", arg: 1, scope: !890, type: !384, flags: DIFlagArtificial | DIFlagObjectPointer)
!892 = !DILocation(line: 0, scope: !890)
!893 = !DILocation(line: 64, column: 28, scope: !890)
!894 = distinct !DISubprogram(name: "~Animal", linkageName: "_ZN6AnimalD2Ev", scope: !331, file: !1, line: 58, type: !335, scopeLine: 58, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !334, retainedNodes: !240)
!895 = !DILocalVariable(name: "this", arg: 1, scope: !894, type: !330, flags: DIFlagArtificial | DIFlagObjectPointer)
!896 = !DILocation(line: 0, scope: !894)
!897 = !DILocation(line: 58, column: 31, scope: !894)
!898 = distinct !DISubprogram(name: "~Animal", linkageName: "_ZN6AnimalD0Ev", scope: !331, file: !1, line: 58, type: !335, scopeLine: 58, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !334, retainedNodes: !240)
!899 = !DILocalVariable(name: "this", arg: 1, scope: !898, type: !330, flags: DIFlagArtificial | DIFlagObjectPointer)
!900 = !DILocation(line: 0, scope: !898)
!901 = !DILocation(line: 58, column: 31, scope: !898)
!902 = distinct !DISubprogram(name: "speak", linkageName: "_ZN6Animal5speakEv", scope: !331, file: !1, line: 59, type: !335, scopeLine: 59, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !338, retainedNodes: !240)
!903 = !DILocalVariable(name: "this", arg: 1, scope: !902, type: !330, flags: DIFlagArtificial | DIFlagObjectPointer)
!904 = !DILocation(line: 0, scope: !902)
!905 = !DILocation(line: 59, column: 27, scope: !902)
!906 = distinct !DISubprogram(name: "~Cat", linkageName: "_ZN3CatD2Ev", scope: !355, file: !1, line: 67, type: !359, scopeLine: 67, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !907, retainedNodes: !240)
!907 = !DISubprogram(name: "~Cat", scope: !355, type: !359, containingType: !355, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!908 = !DILocalVariable(name: "this", arg: 1, scope: !906, type: !390, flags: DIFlagArtificial | DIFlagObjectPointer)
!909 = !DILocation(line: 0, scope: !906)
!910 = !DILocation(line: 67, column: 7, scope: !911)
!911 = distinct !DILexicalBlock(scope: !906, file: !1, line: 67, column: 7)
!912 = !DILocation(line: 67, column: 7, scope: !906)
!913 = distinct !DISubprogram(name: "~Cat", linkageName: "_ZN3CatD0Ev", scope: !355, file: !1, line: 67, type: !359, scopeLine: 67, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !907, retainedNodes: !240)
!914 = !DILocalVariable(name: "this", arg: 1, scope: !913, type: !390, flags: DIFlagArtificial | DIFlagObjectPointer)
!915 = !DILocation(line: 0, scope: !913)
!916 = !DILocation(line: 67, column: 7, scope: !913)
!917 = distinct !DISubprogram(name: "speak", linkageName: "_ZN3Cat5speakEv", scope: !355, file: !1, line: 69, type: !359, scopeLine: 69, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !358, retainedNodes: !240)
!918 = !DILocalVariable(name: "this", arg: 1, scope: !917, type: !390, flags: DIFlagArtificial | DIFlagObjectPointer)
!919 = !DILocation(line: 0, scope: !917)
!920 = !DILocation(line: 69, column: 28, scope: !917)
!921 = distinct !DISubprogram(name: "~Bird", linkageName: "_ZN4BirdD2Ev", scope: !366, file: !1, line: 72, type: !370, scopeLine: 72, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !922, retainedNodes: !240)
!922 = !DISubprogram(name: "~Bird", scope: !366, type: !370, containingType: !366, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!923 = !DILocalVariable(name: "this", arg: 1, scope: !921, type: !396, flags: DIFlagArtificial | DIFlagObjectPointer)
!924 = !DILocation(line: 0, scope: !921)
!925 = !DILocation(line: 72, column: 7, scope: !926)
!926 = distinct !DILexicalBlock(scope: !921, file: !1, line: 72, column: 7)
!927 = !DILocation(line: 72, column: 7, scope: !921)
!928 = distinct !DISubprogram(name: "~Bird", linkageName: "_ZN4BirdD0Ev", scope: !366, file: !1, line: 72, type: !370, scopeLine: 72, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !922, retainedNodes: !240)
!929 = !DILocalVariable(name: "this", arg: 1, scope: !928, type: !396, flags: DIFlagArtificial | DIFlagObjectPointer)
!930 = !DILocation(line: 0, scope: !928)
!931 = !DILocation(line: 72, column: 7, scope: !928)
!932 = distinct !DISubprogram(name: "speak", linkageName: "_ZN4Bird5speakEv", scope: !366, file: !1, line: 74, type: !370, scopeLine: 74, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !369, retainedNodes: !240)
!933 = !DILocalVariable(name: "this", arg: 1, scope: !932, type: !396, flags: DIFlagArtificial | DIFlagObjectPointer)
!934 = !DILocation(line: 0, scope: !932)
!935 = !DILocation(line: 74, column: 28, scope: !932)
!936 = distinct !DISubprogram(name: "Printable", linkageName: "_ZN9PrintableC2Ev", scope: !402, file: !1, line: 93, type: !406, scopeLine: 93, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !937, retainedNodes: !240)
!937 = !DISubprogram(name: "Printable", scope: !402, type: !406, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!938 = !DILocalVariable(name: "this", arg: 1, scope: !936, type: !401, flags: DIFlagArtificial | DIFlagObjectPointer)
!939 = !DILocation(line: 0, scope: !936)
!940 = !DILocation(line: 93, column: 7, scope: !936)
!941 = distinct !DISubprogram(name: "Serializable", linkageName: "_ZN12SerializableC2Ev", scope: !416, file: !1, line: 99, type: !420, scopeLine: 99, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !942, retainedNodes: !240)
!942 = !DISubprogram(name: "Serializable", scope: !416, type: !420, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!943 = !DILocalVariable(name: "this", arg: 1, scope: !941, type: !435, flags: DIFlagArtificial | DIFlagObjectPointer)
!944 = !DILocation(line: 0, scope: !941)
!945 = !DILocation(line: 99, column: 7, scope: !941)
!946 = distinct !DISubprogram(name: "~Document", linkageName: "_ZN8DocumentD2Ev", scope: !412, file: !1, line: 105, type: !425, scopeLine: 105, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !947, retainedNodes: !240)
!947 = !DISubprogram(name: "~Document", scope: !412, type: !425, containingType: !412, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!948 = !DILocalVariable(name: "this", arg: 1, scope: !946, type: !447, flags: DIFlagArtificial | DIFlagObjectPointer)
!949 = !DILocation(line: 0, scope: !946)
!950 = !DILocation(line: 105, column: 7, scope: !951)
!951 = distinct !DILexicalBlock(scope: !946, file: !1, line: 105, column: 7)
!952 = !DILocation(line: 105, column: 7, scope: !946)
!953 = distinct !DISubprogram(name: "~Document", linkageName: "_ZN8DocumentD0Ev", scope: !412, file: !1, line: 105, type: !425, scopeLine: 105, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !947, retainedNodes: !240)
!954 = !DILocalVariable(name: "this", arg: 1, scope: !953, type: !447, flags: DIFlagArtificial | DIFlagObjectPointer)
!955 = !DILocation(line: 0, scope: !953)
!956 = !DILocation(line: 105, column: 7, scope: !953)
!957 = distinct !DISubprogram(name: "print", linkageName: "_ZN8Document5printEv", scope: !412, file: !1, line: 107, type: !425, scopeLine: 107, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !424, retainedNodes: !240)
!958 = !DILocalVariable(name: "this", arg: 1, scope: !957, type: !447, flags: DIFlagArtificial | DIFlagObjectPointer)
!959 = !DILocation(line: 0, scope: !957)
!960 = !DILocation(line: 107, column: 28, scope: !957)
!961 = distinct !DISubprogram(name: "serialize", linkageName: "_ZN8Document9serializeEv", scope: !412, file: !1, line: 108, type: !425, scopeLine: 108, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !428, retainedNodes: !240)
!962 = !DILocalVariable(name: "this", arg: 1, scope: !961, type: !447, flags: DIFlagArtificial | DIFlagObjectPointer)
!963 = !DILocation(line: 0, scope: !961)
!964 = !DILocation(line: 108, column: 32, scope: !961)
!965 = distinct !DISubprogram(linkageName: "_ZThn8_N8DocumentD1Ev", scope: !1, file: !1, line: 105, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!966 = !DISubroutineType(types: !240)
!967 = !DILocation(line: 0, scope: !965)
!968 = distinct !DISubprogram(linkageName: "_ZThn8_N8DocumentD0Ev", scope: !1, file: !1, line: 105, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!969 = !DILocation(line: 0, scope: !968)
!970 = distinct !DISubprogram(linkageName: "_ZThn8_N8Document9serializeEv", scope: !1, file: !1, line: 108, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!971 = !DILocation(line: 0, scope: !970)
!972 = distinct !DISubprogram(name: "~Printable", linkageName: "_ZN9PrintableD2Ev", scope: !402, file: !1, line: 95, type: !406, scopeLine: 95, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !405, retainedNodes: !240)
!973 = !DILocalVariable(name: "this", arg: 1, scope: !972, type: !401, flags: DIFlagArtificial | DIFlagObjectPointer)
!974 = !DILocation(line: 0, scope: !972)
!975 = !DILocation(line: 95, column: 34, scope: !972)
!976 = distinct !DISubprogram(name: "~Printable", linkageName: "_ZN9PrintableD0Ev", scope: !402, file: !1, line: 95, type: !406, scopeLine: 95, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !405, retainedNodes: !240)
!977 = !DILocalVariable(name: "this", arg: 1, scope: !976, type: !401, flags: DIFlagArtificial | DIFlagObjectPointer)
!978 = !DILocation(line: 0, scope: !976)
!979 = !DILocation(line: 95, column: 34, scope: !976)
!980 = distinct !DISubprogram(name: "print", linkageName: "_ZN9Printable5printEv", scope: !402, file: !1, line: 96, type: !406, scopeLine: 96, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !409, retainedNodes: !240)
!981 = !DILocalVariable(name: "this", arg: 1, scope: !980, type: !401, flags: DIFlagArtificial | DIFlagObjectPointer)
!982 = !DILocation(line: 0, scope: !980)
!983 = !DILocation(line: 96, column: 27, scope: !980)
!984 = distinct !DISubprogram(name: "~Serializable", linkageName: "_ZN12SerializableD2Ev", scope: !416, file: !1, line: 101, type: !420, scopeLine: 101, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !419, retainedNodes: !240)
!985 = !DILocalVariable(name: "this", arg: 1, scope: !984, type: !435, flags: DIFlagArtificial | DIFlagObjectPointer)
!986 = !DILocation(line: 0, scope: !984)
!987 = !DILocation(line: 101, column: 37, scope: !984)
!988 = distinct !DISubprogram(name: "~Serializable", linkageName: "_ZN12SerializableD0Ev", scope: !416, file: !1, line: 101, type: !420, scopeLine: 101, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !419, retainedNodes: !240)
!989 = !DILocalVariable(name: "this", arg: 1, scope: !988, type: !435, flags: DIFlagArtificial | DIFlagObjectPointer)
!990 = !DILocation(line: 0, scope: !988)
!991 = !DILocation(line: 101, column: 37, scope: !988)
!992 = distinct !DISubprogram(name: "serialize", linkageName: "_ZN12Serializable9serializeEv", scope: !416, file: !1, line: 102, type: !420, scopeLine: 102, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !423, retainedNodes: !240)
!993 = !DILocalVariable(name: "this", arg: 1, scope: !992, type: !435, flags: DIFlagArtificial | DIFlagObjectPointer)
!994 = !DILocation(line: 0, scope: !992)
!995 = !DILocation(line: 102, column: 31, scope: !992)
!996 = distinct !DISubprogram(name: "DiamondBase", linkageName: "_ZN11DiamondBaseC2Ev", scope: !453, file: !1, line: 122, type: !457, scopeLine: 122, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !997, retainedNodes: !240)
!997 = !DISubprogram(name: "DiamondBase", scope: !453, type: !457, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!998 = !DILocalVariable(name: "this", arg: 1, scope: !996, type: !452, flags: DIFlagArtificial | DIFlagObjectPointer)
!999 = !DILocation(line: 0, scope: !996)
!1000 = !DILocation(line: 122, column: 7, scope: !996)
!1001 = distinct !DISubprogram(name: "DiamondLeft", linkageName: "_ZN11DiamondLeftC2Ev", scope: !468, file: !1, line: 128, type: !1002, scopeLine: 128, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1004, retainedNodes: !240)
!1002 = !DISubroutineType(types: !1003)
!1003 = !{null, !474}
!1004 = !DISubprogram(name: "DiamondLeft", scope: !468, type: !1002, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1005 = !DILocalVariable(name: "this", arg: 1, scope: !1001, type: !1006, flags: DIFlagArtificial | DIFlagObjectPointer)
!1006 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !468, size: 64)
!1007 = !DILocation(line: 0, scope: !1001)
!1008 = !DILocalVariable(name: "vtt", arg: 2, scope: !1001, type: !1009, flags: DIFlagArtificial)
!1009 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !48, size: 64)
!1010 = !DILocation(line: 128, column: 7, scope: !1001)
!1011 = distinct !DISubprogram(name: "DiamondRight", linkageName: "_ZN12DiamondRightC2Ev", scope: !476, file: !1, line: 133, type: !1012, scopeLine: 133, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1014, retainedNodes: !240)
!1012 = !DISubroutineType(types: !1013)
!1013 = !{null, !482}
!1014 = !DISubprogram(name: "DiamondRight", scope: !476, type: !1012, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1015 = !DILocalVariable(name: "this", arg: 1, scope: !1011, type: !1016, flags: DIFlagArtificial | DIFlagObjectPointer)
!1016 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !476, size: 64)
!1017 = !DILocation(line: 0, scope: !1011)
!1018 = !DILocalVariable(name: "vtt", arg: 2, scope: !1011, type: !1009, flags: DIFlagArtificial)
!1019 = !DILocation(line: 133, column: 7, scope: !1011)
!1020 = distinct !DISubprogram(name: "~DiamondLeft", linkageName: "_ZN11DiamondLeftD1Ev", scope: !468, file: !1, line: 128, type: !1002, scopeLine: 128, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1021, retainedNodes: !240)
!1021 = !DISubprogram(name: "~DiamondLeft", scope: !468, type: !1002, containingType: !468, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1022 = !DILocalVariable(name: "this", arg: 1, scope: !1020, type: !1006, flags: DIFlagArtificial | DIFlagObjectPointer)
!1023 = !DILocation(line: 0, scope: !1020)
!1024 = !DILocation(line: 128, column: 7, scope: !1020)
!1025 = distinct !DISubprogram(name: "~DiamondLeft", linkageName: "_ZN11DiamondLeftD0Ev", scope: !468, file: !1, line: 128, type: !1002, scopeLine: 128, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1021, retainedNodes: !240)
!1026 = !DILocalVariable(name: "this", arg: 1, scope: !1025, type: !1006, flags: DIFlagArtificial | DIFlagObjectPointer)
!1027 = !DILocation(line: 0, scope: !1025)
!1028 = !DILocation(line: 128, column: 7, scope: !1025)
!1029 = distinct !DISubprogram(name: "value", linkageName: "_ZN11DiamondLeft5valueEv", scope: !468, file: !1, line: 130, type: !472, scopeLine: 130, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !471, retainedNodes: !240)
!1030 = !DILocalVariable(name: "this", arg: 1, scope: !1029, type: !1006, flags: DIFlagArtificial | DIFlagObjectPointer)
!1031 = !DILocation(line: 0, scope: !1029)
!1032 = !DILocation(line: 130, column: 28, scope: !1029)
!1033 = distinct !DISubprogram(name: "~DiamondRight", linkageName: "_ZN12DiamondRightD1Ev", scope: !476, file: !1, line: 133, type: !1012, scopeLine: 133, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1034, retainedNodes: !240)
!1034 = !DISubprogram(name: "~DiamondRight", scope: !476, type: !1012, containingType: !476, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1035 = !DILocalVariable(name: "this", arg: 1, scope: !1033, type: !1016, flags: DIFlagArtificial | DIFlagObjectPointer)
!1036 = !DILocation(line: 0, scope: !1033)
!1037 = !DILocation(line: 133, column: 7, scope: !1033)
!1038 = distinct !DISubprogram(name: "~DiamondRight", linkageName: "_ZN12DiamondRightD0Ev", scope: !476, file: !1, line: 133, type: !1012, scopeLine: 133, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1034, retainedNodes: !240)
!1039 = !DILocalVariable(name: "this", arg: 1, scope: !1038, type: !1016, flags: DIFlagArtificial | DIFlagObjectPointer)
!1040 = !DILocation(line: 0, scope: !1038)
!1041 = !DILocation(line: 133, column: 7, scope: !1038)
!1042 = distinct !DISubprogram(name: "value", linkageName: "_ZN12DiamondRight5valueEv", scope: !476, file: !1, line: 135, type: !480, scopeLine: 135, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !479, retainedNodes: !240)
!1043 = !DILocalVariable(name: "this", arg: 1, scope: !1042, type: !1016, flags: DIFlagArtificial | DIFlagObjectPointer)
!1044 = !DILocation(line: 0, scope: !1042)
!1045 = !DILocation(line: 135, column: 28, scope: !1042)
!1046 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N12DiamondRightD1Ev", scope: !1, file: !1, line: 133, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1047 = !DILocation(line: 0, scope: !1046)
!1048 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N12DiamondRightD0Ev", scope: !1, file: !1, line: 133, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1049 = !DILocation(line: 0, scope: !1048)
!1050 = distinct !DISubprogram(linkageName: "_ZTv0_n32_N12DiamondRight5valueEv", scope: !1, file: !1, line: 135, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1051 = !DILocation(line: 0, scope: !1050)
!1052 = distinct !DISubprogram(name: "~DiamondBottom", linkageName: "_ZN13DiamondBottomD1Ev", scope: !465, file: !1, line: 138, type: !496, scopeLine: 138, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1053, retainedNodes: !240)
!1053 = !DISubprogram(name: "~DiamondBottom", scope: !465, type: !496, containingType: !465, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1054 = !DILocalVariable(name: "this", arg: 1, scope: !1052, type: !500, flags: DIFlagArtificial | DIFlagObjectPointer)
!1055 = !DILocation(line: 0, scope: !1052)
!1056 = !DILocation(line: 138, column: 7, scope: !1052)
!1057 = distinct !DISubprogram(name: "~DiamondBottom", linkageName: "_ZN13DiamondBottomD0Ev", scope: !465, file: !1, line: 138, type: !496, scopeLine: 138, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1053, retainedNodes: !240)
!1058 = !DILocalVariable(name: "this", arg: 1, scope: !1057, type: !500, flags: DIFlagArtificial | DIFlagObjectPointer)
!1059 = !DILocation(line: 0, scope: !1057)
!1060 = !DILocation(line: 138, column: 7, scope: !1057)
!1061 = distinct !DISubprogram(name: "value", linkageName: "_ZN13DiamondBottom5valueEv", scope: !465, file: !1, line: 141, type: !484, scopeLine: 141, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !483, retainedNodes: !240)
!1062 = !DILocalVariable(name: "this", arg: 1, scope: !1061, type: !500, flags: DIFlagArtificial | DIFlagObjectPointer)
!1063 = !DILocation(line: 0, scope: !1061)
!1064 = !DILocation(line: 141, column: 28, scope: !1061)
!1065 = distinct !DISubprogram(linkageName: "_ZThn8_N13DiamondBottomD1Ev", scope: !1, file: !1, line: 138, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1066 = !DILocation(line: 0, scope: !1065)
!1067 = distinct !DISubprogram(linkageName: "_ZThn8_N13DiamondBottomD0Ev", scope: !1, file: !1, line: 138, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1068 = !DILocation(line: 0, scope: !1067)
!1069 = distinct !DISubprogram(linkageName: "_ZThn8_N13DiamondBottom5valueEv", scope: !1, file: !1, line: 141, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1070 = !DILocation(line: 0, scope: !1069)
!1071 = distinct !DISubprogram(name: "~DiamondBase", linkageName: "_ZN11DiamondBaseD2Ev", scope: !453, file: !1, line: 124, type: !457, scopeLine: 124, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !456, retainedNodes: !240)
!1072 = !DILocalVariable(name: "this", arg: 1, scope: !1071, type: !452, flags: DIFlagArtificial | DIFlagObjectPointer)
!1073 = !DILocation(line: 0, scope: !1071)
!1074 = !DILocation(line: 124, column: 36, scope: !1071)
!1075 = distinct !DISubprogram(name: "~DiamondBase", linkageName: "_ZN11DiamondBaseD0Ev", scope: !453, file: !1, line: 124, type: !457, scopeLine: 124, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !456, retainedNodes: !240)
!1076 = !DILocalVariable(name: "this", arg: 1, scope: !1075, type: !452, flags: DIFlagArtificial | DIFlagObjectPointer)
!1077 = !DILocation(line: 0, scope: !1075)
!1078 = !DILocation(line: 124, column: 36, scope: !1075)
!1079 = distinct !DISubprogram(name: "value", linkageName: "_ZN11DiamondBase5valueEv", scope: !453, file: !1, line: 125, type: !461, scopeLine: 125, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !460, retainedNodes: !240)
!1080 = !DILocalVariable(name: "this", arg: 1, scope: !1079, type: !452, flags: DIFlagArtificial | DIFlagObjectPointer)
!1081 = !DILocation(line: 0, scope: !1079)
!1082 = !DILocation(line: 125, column: 27, scope: !1079)
!1083 = distinct !DISubprogram(name: "~DiamondLeft", linkageName: "_ZN11DiamondLeftD2Ev", scope: !468, file: !1, line: 128, type: !1002, scopeLine: 128, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1021, retainedNodes: !240)
!1084 = !DILocalVariable(name: "this", arg: 1, scope: !1083, type: !1006, flags: DIFlagArtificial | DIFlagObjectPointer)
!1085 = !DILocation(line: 0, scope: !1083)
!1086 = !DILocalVariable(name: "vtt", arg: 2, scope: !1083, type: !1009, flags: DIFlagArtificial)
!1087 = !DILocation(line: 128, column: 7, scope: !1083)
!1088 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N11DiamondLeftD1Ev", scope: !1, file: !1, line: 128, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1089 = !DILocation(line: 0, scope: !1088)
!1090 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N11DiamondLeftD0Ev", scope: !1, file: !1, line: 128, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1091 = !DILocation(line: 0, scope: !1090)
!1092 = distinct !DISubprogram(linkageName: "_ZTv0_n32_N11DiamondLeft5valueEv", scope: !1, file: !1, line: 130, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1093 = !DILocation(line: 0, scope: !1092)
!1094 = distinct !DISubprogram(name: "~DiamondRight", linkageName: "_ZN12DiamondRightD2Ev", scope: !476, file: !1, line: 133, type: !1012, scopeLine: 133, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1034, retainedNodes: !240)
!1095 = !DILocalVariable(name: "this", arg: 1, scope: !1094, type: !1016, flags: DIFlagArtificial | DIFlagObjectPointer)
!1096 = !DILocation(line: 0, scope: !1094)
!1097 = !DILocalVariable(name: "vtt", arg: 2, scope: !1094, type: !1009, flags: DIFlagArtificial)
!1098 = !DILocation(line: 133, column: 7, scope: !1094)
!1099 = distinct !DISubprogram(name: "~DiamondBottom", linkageName: "_ZN13DiamondBottomD2Ev", scope: !465, file: !1, line: 138, type: !496, scopeLine: 138, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1053, retainedNodes: !240)
!1100 = !DILocalVariable(name: "this", arg: 1, scope: !1099, type: !500, flags: DIFlagArtificial | DIFlagObjectPointer)
!1101 = !DILocation(line: 0, scope: !1099)
!1102 = !DILocalVariable(name: "vtt", arg: 2, scope: !1099, type: !1009, flags: DIFlagArtificial)
!1103 = !DILocation(line: 138, column: 7, scope: !1104)
!1104 = distinct !DILexicalBlock(scope: !1099, file: !1, line: 138, column: 7)
!1105 = !DILocation(line: 138, column: 7, scope: !1099)
!1106 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N13DiamondBottomD1Ev", scope: !1, file: !1, line: 138, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1107 = !DILocation(line: 0, scope: !1106)
!1108 = distinct !DISubprogram(linkageName: "_ZTv0_n24_N13DiamondBottomD0Ev", scope: !1, file: !1, line: 138, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1109 = !DILocation(line: 0, scope: !1108)
!1110 = distinct !DISubprogram(linkageName: "_ZTv0_n32_N13DiamondBottom5valueEv", scope: !1, file: !1, line: 141, type: !966, flags: DIFlagArtificial | DIFlagThunk, spFlags: DISPFlagDefinition, unit: !0)
!1111 = !DILocation(line: 0, scope: !1110)
!1112 = distinct !DISubprogram(name: "AbstractShape", linkageName: "_ZN13AbstractShapeC2Ev", scope: !507, file: !1, line: 156, type: !511, scopeLine: 156, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1113, retainedNodes: !240)
!1113 = !DISubprogram(name: "AbstractShape", scope: !507, type: !511, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1114 = !DILocalVariable(name: "this", arg: 1, scope: !1112, type: !506, flags: DIFlagArtificial | DIFlagObjectPointer)
!1115 = !DILocation(line: 0, scope: !1112)
!1116 = !DILocation(line: 156, column: 7, scope: !1112)
!1117 = distinct !DISubprogram(name: "~Circle", linkageName: "_ZN6CircleD2Ev", scope: !522, file: !1, line: 163, type: !1118, scopeLine: 163, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1120, retainedNodes: !240)
!1118 = !DISubroutineType(types: !1119)
!1119 = !{null, !529}
!1120 = !DISubprogram(name: "~Circle", scope: !522, type: !1118, containingType: !522, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1121 = !DILocalVariable(name: "this", arg: 1, scope: !1117, type: !581, flags: DIFlagArtificial | DIFlagObjectPointer)
!1122 = !DILocation(line: 0, scope: !1117)
!1123 = !DILocation(line: 163, column: 7, scope: !1124)
!1124 = distinct !DILexicalBlock(scope: !1117, file: !1, line: 163, column: 7)
!1125 = !DILocation(line: 163, column: 7, scope: !1117)
!1126 = distinct !DISubprogram(name: "~Circle", linkageName: "_ZN6CircleD0Ev", scope: !522, file: !1, line: 163, type: !1118, scopeLine: 163, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1120, retainedNodes: !240)
!1127 = !DILocalVariable(name: "this", arg: 1, scope: !1126, type: !581, flags: DIFlagArtificial | DIFlagObjectPointer)
!1128 = !DILocation(line: 0, scope: !1126)
!1129 = !DILocation(line: 163, column: 7, scope: !1126)
!1130 = distinct !DISubprogram(name: "area", linkageName: "_ZN6Circle4areaEv", scope: !522, file: !1, line: 167, type: !531, scopeLine: 167, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !530, retainedNodes: !240)
!1131 = !DILocalVariable(name: "this", arg: 1, scope: !1130, type: !581, flags: DIFlagArtificial | DIFlagObjectPointer)
!1132 = !DILocation(line: 0, scope: !1130)
!1133 = !DILocation(line: 167, column: 47, scope: !1130)
!1134 = !DILocation(line: 167, column: 45, scope: !1130)
!1135 = !DILocation(line: 167, column: 56, scope: !1130)
!1136 = !DILocation(line: 167, column: 54, scope: !1130)
!1137 = !DILocation(line: 167, column: 30, scope: !1130)
!1138 = distinct !DISubprogram(name: "perimeter", linkageName: "_ZN6Circle9perimeterEv", scope: !522, file: !1, line: 168, type: !531, scopeLine: 168, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !533, retainedNodes: !240)
!1139 = !DILocalVariable(name: "this", arg: 1, scope: !1138, type: !581, flags: DIFlagArtificial | DIFlagObjectPointer)
!1140 = !DILocation(line: 0, scope: !1138)
!1141 = !DILocation(line: 168, column: 56, scope: !1138)
!1142 = !DILocation(line: 168, column: 54, scope: !1138)
!1143 = !DILocation(line: 168, column: 35, scope: !1138)
!1144 = distinct !DISubprogram(name: "~AbstractShape", linkageName: "_ZN13AbstractShapeD2Ev", scope: !507, file: !1, line: 158, type: !511, scopeLine: 158, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !510, retainedNodes: !240)
!1145 = !DILocalVariable(name: "this", arg: 1, scope: !1144, type: !506, flags: DIFlagArtificial | DIFlagObjectPointer)
!1146 = !DILocation(line: 0, scope: !1144)
!1147 = !DILocation(line: 158, column: 38, scope: !1144)
!1148 = distinct !DISubprogram(name: "~AbstractShape", linkageName: "_ZN13AbstractShapeD0Ev", scope: !507, file: !1, line: 158, type: !511, scopeLine: 158, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !510, retainedNodes: !240)
!1149 = !DILocalVariable(name: "this", arg: 1, scope: !1148, type: !506, flags: DIFlagArtificial | DIFlagObjectPointer)
!1150 = !DILocation(line: 0, scope: !1148)
!1151 = !DILocation(line: 158, column: 38, scope: !1148)
!1152 = distinct !DISubprogram(name: "~Rectangle", linkageName: "_ZN9RectangleD2Ev", scope: !538, file: !1, line: 171, type: !1153, scopeLine: 171, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1155, retainedNodes: !240)
!1153 = !DISubroutineType(types: !1154)
!1154 = !{null, !546}
!1155 = !DISubprogram(name: "~Rectangle", scope: !538, type: !1153, containingType: !538, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1156 = !DILocalVariable(name: "this", arg: 1, scope: !1152, type: !592, flags: DIFlagArtificial | DIFlagObjectPointer)
!1157 = !DILocation(line: 0, scope: !1152)
!1158 = !DILocation(line: 171, column: 7, scope: !1159)
!1159 = distinct !DILexicalBlock(scope: !1152, file: !1, line: 171, column: 7)
!1160 = !DILocation(line: 171, column: 7, scope: !1152)
!1161 = distinct !DISubprogram(name: "~Rectangle", linkageName: "_ZN9RectangleD0Ev", scope: !538, file: !1, line: 171, type: !1153, scopeLine: 171, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1155, retainedNodes: !240)
!1162 = !DILocalVariable(name: "this", arg: 1, scope: !1161, type: !592, flags: DIFlagArtificial | DIFlagObjectPointer)
!1163 = !DILocation(line: 0, scope: !1161)
!1164 = !DILocation(line: 171, column: 7, scope: !1161)
!1165 = distinct !DISubprogram(name: "area", linkageName: "_ZN9Rectangle4areaEv", scope: !538, file: !1, line: 175, type: !548, scopeLine: 175, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !547, retainedNodes: !240)
!1166 = !DILocalVariable(name: "this", arg: 1, scope: !1165, type: !592, flags: DIFlagArtificial | DIFlagObjectPointer)
!1167 = !DILocation(line: 0, scope: !1165)
!1168 = !DILocation(line: 175, column: 37, scope: !1165)
!1169 = !DILocation(line: 175, column: 45, scope: !1165)
!1170 = !DILocation(line: 175, column: 43, scope: !1165)
!1171 = !DILocation(line: 175, column: 30, scope: !1165)
!1172 = distinct !DISubprogram(name: "perimeter", linkageName: "_ZN9Rectangle9perimeterEv", scope: !538, file: !1, line: 176, type: !548, scopeLine: 176, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !550, retainedNodes: !240)
!1173 = !DILocalVariable(name: "this", arg: 1, scope: !1172, type: !592, flags: DIFlagArtificial | DIFlagObjectPointer)
!1174 = !DILocation(line: 0, scope: !1172)
!1175 = !DILocation(line: 176, column: 47, scope: !1172)
!1176 = !DILocation(line: 176, column: 55, scope: !1172)
!1177 = !DILocation(line: 176, column: 53, scope: !1172)
!1178 = !DILocation(line: 176, column: 44, scope: !1172)
!1179 = !DILocation(line: 176, column: 35, scope: !1172)
!1180 = distinct !DISubprogram(name: "IObserver", linkageName: "_ZN9IObserverC2Ev", scope: !609, file: !1, line: 198, type: !613, scopeLine: 198, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1181, retainedNodes: !240)
!1181 = !DISubprogram(name: "IObserver", scope: !609, type: !613, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1182 = !DILocalVariable(name: "this", arg: 1, scope: !1180, type: !608, flags: DIFlagArtificial | DIFlagObjectPointer)
!1183 = !DILocation(line: 0, scope: !1180)
!1184 = !DILocation(line: 198, column: 7, scope: !1180)
!1185 = distinct !DISubprogram(name: "~ConcreteObserverA", linkageName: "_ZN17ConcreteObserverAD2Ev", scope: !621, file: !1, line: 204, type: !665, scopeLine: 204, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1186, retainedNodes: !240)
!1186 = !DISubprogram(name: "~ConcreteObserverA", scope: !621, type: !665, containingType: !621, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1187 = !DILocalVariable(name: "this", arg: 1, scope: !1185, type: !669, flags: DIFlagArtificial | DIFlagObjectPointer)
!1188 = !DILocation(line: 0, scope: !1185)
!1189 = !DILocation(line: 204, column: 7, scope: !1190)
!1190 = distinct !DILexicalBlock(scope: !1185, file: !1, line: 204, column: 7)
!1191 = !DILocation(line: 204, column: 7, scope: !1185)
!1192 = distinct !DISubprogram(name: "~ConcreteObserverA", linkageName: "_ZN17ConcreteObserverAD0Ev", scope: !621, file: !1, line: 204, type: !665, scopeLine: 204, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1186, retainedNodes: !240)
!1193 = !DILocalVariable(name: "this", arg: 1, scope: !1192, type: !669, flags: DIFlagArtificial | DIFlagObjectPointer)
!1194 = !DILocation(line: 0, scope: !1192)
!1195 = !DILocation(line: 204, column: 7, scope: !1192)
!1196 = distinct !DISubprogram(name: "update", linkageName: "_ZN17ConcreteObserverA6updateEi", scope: !621, file: !1, line: 206, type: !625, scopeLine: 206, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !624, retainedNodes: !240)
!1197 = !DILocalVariable(name: "this", arg: 1, scope: !1196, type: !669, flags: DIFlagArtificial | DIFlagObjectPointer)
!1198 = !DILocation(line: 0, scope: !1196)
!1199 = !DILocalVariable(name: "value", arg: 2, scope: !1196, file: !1, line: 206, type: !27)
!1200 = !DILocation(line: 206, column: 21, scope: !1196)
!1201 = !DILocation(line: 206, column: 52, scope: !1196)
!1202 = distinct !DISubprogram(name: "~IObserver", linkageName: "_ZN9IObserverD2Ev", scope: !609, file: !1, line: 200, type: !613, scopeLine: 200, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !612, retainedNodes: !240)
!1203 = !DILocalVariable(name: "this", arg: 1, scope: !1202, type: !608, flags: DIFlagArtificial | DIFlagObjectPointer)
!1204 = !DILocation(line: 0, scope: !1202)
!1205 = !DILocation(line: 200, column: 34, scope: !1202)
!1206 = distinct !DISubprogram(name: "~IObserver", linkageName: "_ZN9IObserverD0Ev", scope: !609, file: !1, line: 200, type: !613, scopeLine: 200, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !612, retainedNodes: !240)
!1207 = !DILocalVariable(name: "this", arg: 1, scope: !1206, type: !608, flags: DIFlagArtificial | DIFlagObjectPointer)
!1208 = !DILocation(line: 0, scope: !1206)
!1209 = !DILocation(line: 200, column: 34, scope: !1206)
!1210 = distinct !DISubprogram(name: "~ConcreteObserverB", linkageName: "_ZN17ConcreteObserverBD2Ev", scope: !632, file: !1, line: 209, type: !673, scopeLine: 209, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1211, retainedNodes: !240)
!1211 = !DISubprogram(name: "~ConcreteObserverB", scope: !632, type: !673, containingType: !632, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1212 = !DILocalVariable(name: "this", arg: 1, scope: !1210, type: !677, flags: DIFlagArtificial | DIFlagObjectPointer)
!1213 = !DILocation(line: 0, scope: !1210)
!1214 = !DILocation(line: 209, column: 7, scope: !1215)
!1215 = distinct !DILexicalBlock(scope: !1210, file: !1, line: 209, column: 7)
!1216 = !DILocation(line: 209, column: 7, scope: !1210)
!1217 = distinct !DISubprogram(name: "~ConcreteObserverB", linkageName: "_ZN17ConcreteObserverBD0Ev", scope: !632, file: !1, line: 209, type: !673, scopeLine: 209, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1211, retainedNodes: !240)
!1218 = !DILocalVariable(name: "this", arg: 1, scope: !1217, type: !677, flags: DIFlagArtificial | DIFlagObjectPointer)
!1219 = !DILocation(line: 0, scope: !1217)
!1220 = !DILocation(line: 209, column: 7, scope: !1217)
!1221 = distinct !DISubprogram(name: "update", linkageName: "_ZN17ConcreteObserverB6updateEi", scope: !632, file: !1, line: 211, type: !636, scopeLine: 211, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !635, retainedNodes: !240)
!1222 = !DILocalVariable(name: "this", arg: 1, scope: !1221, type: !677, flags: DIFlagArtificial | DIFlagObjectPointer)
!1223 = !DILocation(line: 0, scope: !1221)
!1224 = !DILocalVariable(name: "value", arg: 2, scope: !1221, file: !1, line: 211, type: !27)
!1225 = !DILocation(line: 211, column: 21, scope: !1221)
!1226 = !DILocation(line: 211, column: 52, scope: !1221)
!1227 = distinct !DISubprogram(name: "Prototype", linkageName: "_ZN9PrototypeC2Ev", scope: !694, file: !1, line: 257, type: !698, scopeLine: 257, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1228, retainedNodes: !240)
!1228 = !DISubprogram(name: "Prototype", scope: !694, type: !698, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1229 = !DILocalVariable(name: "this", arg: 1, scope: !1227, type: !693, flags: DIFlagArtificial | DIFlagObjectPointer)
!1230 = !DILocation(line: 0, scope: !1227)
!1231 = !DILocation(line: 257, column: 7, scope: !1227)
!1232 = distinct !DISubprogram(name: "~ConcretePrototype1", linkageName: "_ZN18ConcretePrototype1D2Ev", scope: !706, file: !1, line: 263, type: !724, scopeLine: 263, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1233, retainedNodes: !240)
!1233 = !DISubprogram(name: "~ConcretePrototype1", scope: !706, type: !724, containingType: !706, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1234 = !DILocalVariable(name: "this", arg: 1, scope: !1232, type: !728, flags: DIFlagArtificial | DIFlagObjectPointer)
!1235 = !DILocation(line: 0, scope: !1232)
!1236 = !DILocation(line: 263, column: 7, scope: !1237)
!1237 = distinct !DILexicalBlock(scope: !1232, file: !1, line: 263, column: 7)
!1238 = !DILocation(line: 263, column: 7, scope: !1232)
!1239 = distinct !DISubprogram(name: "~ConcretePrototype1", linkageName: "_ZN18ConcretePrototype1D0Ev", scope: !706, file: !1, line: 263, type: !724, scopeLine: 263, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1233, retainedNodes: !240)
!1240 = !DILocalVariable(name: "this", arg: 1, scope: !1239, type: !728, flags: DIFlagArtificial | DIFlagObjectPointer)
!1241 = !DILocation(line: 0, scope: !1239)
!1242 = !DILocation(line: 263, column: 7, scope: !1239)
!1243 = distinct !DISubprogram(name: "clone", linkageName: "_ZN18ConcretePrototype15cloneEv", scope: !706, file: !1, line: 265, type: !710, scopeLine: 265, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !709, retainedNodes: !240)
!1244 = !DILocalVariable(name: "this", arg: 1, scope: !1243, type: !728, flags: DIFlagArtificial | DIFlagObjectPointer)
!1245 = !DILocation(line: 0, scope: !1243)
!1246 = !DILocation(line: 265, column: 42, scope: !1243)
!1247 = !DILocation(line: 265, column: 46, scope: !1243)
!1248 = !DILocation(line: 265, column: 35, scope: !1243)
!1249 = distinct !DISubprogram(name: "~Prototype", linkageName: "_ZN9PrototypeD2Ev", scope: !694, file: !1, line: 259, type: !698, scopeLine: 259, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !697, retainedNodes: !240)
!1250 = !DILocalVariable(name: "this", arg: 1, scope: !1249, type: !693, flags: DIFlagArtificial | DIFlagObjectPointer)
!1251 = !DILocation(line: 0, scope: !1249)
!1252 = !DILocation(line: 259, column: 34, scope: !1249)
!1253 = distinct !DISubprogram(name: "~Prototype", linkageName: "_ZN9PrototypeD0Ev", scope: !694, file: !1, line: 259, type: !698, scopeLine: 259, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !697, retainedNodes: !240)
!1254 = !DILocalVariable(name: "this", arg: 1, scope: !1253, type: !693, flags: DIFlagArtificial | DIFlagObjectPointer)
!1255 = !DILocation(line: 0, scope: !1253)
!1256 = !DILocation(line: 259, column: 34, scope: !1253)
!1257 = distinct !DISubprogram(name: "ConcretePrototype1", linkageName: "_ZN18ConcretePrototype1C2ERKS_", scope: !706, file: !1, line: 263, type: !1258, scopeLine: 263, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1262, retainedNodes: !240)
!1258 = !DISubroutineType(types: !1259)
!1259 = !{null, !712, !1260}
!1260 = !DIDerivedType(tag: DW_TAG_reference_type, baseType: !1261, size: 64)
!1261 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !706)
!1262 = !DISubprogram(name: "ConcretePrototype1", scope: !706, type: !1258, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1263 = !DILocalVariable(name: "this", arg: 1, scope: !1257, type: !728, flags: DIFlagArtificial | DIFlagObjectPointer)
!1264 = !DILocation(line: 0, scope: !1257)
!1265 = !DILocalVariable(arg: 2, scope: !1257, type: !1260)
!1266 = !DILocation(line: 263, column: 7, scope: !1257)
!1267 = distinct !DISubprogram(name: "Prototype", linkageName: "_ZN9PrototypeC2ERKS_", scope: !694, file: !1, line: 257, type: !1268, scopeLine: 257, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1272, retainedNodes: !240)
!1268 = !DISubroutineType(types: !1269)
!1269 = !{null, !700, !1270}
!1270 = !DIDerivedType(tag: DW_TAG_reference_type, baseType: !1271, size: 64)
!1271 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !694)
!1272 = !DISubprogram(name: "Prototype", scope: !694, type: !1268, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1273 = !DILocalVariable(name: "this", arg: 1, scope: !1267, type: !693, flags: DIFlagArtificial | DIFlagObjectPointer)
!1274 = !DILocation(line: 0, scope: !1267)
!1275 = !DILocalVariable(arg: 2, scope: !1267, type: !1270)
!1276 = !DILocation(line: 257, column: 7, scope: !1267)
!1277 = distinct !DISubprogram(name: "CloneableBase", linkageName: "_ZN13CloneableBaseC2Ev", scope: !734, file: !1, line: 285, type: !738, scopeLine: 285, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1278, retainedNodes: !240)
!1278 = !DISubprogram(name: "CloneableBase", scope: !734, type: !738, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1279 = !DILocalVariable(name: "this", arg: 1, scope: !1277, type: !733, flags: DIFlagArtificial | DIFlagObjectPointer)
!1280 = !DILocation(line: 0, scope: !1277)
!1281 = !DILocation(line: 285, column: 7, scope: !1277)
!1282 = distinct !DISubprogram(name: "~CloneableDerived", linkageName: "_ZN16CloneableDerivedD2Ev", scope: !746, file: !1, line: 291, type: !765, scopeLine: 291, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1283, retainedNodes: !240)
!1283 = !DISubprogram(name: "~CloneableDerived", scope: !746, type: !765, containingType: !746, virtualIndex: 0, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagVirtual)
!1284 = !DILocalVariable(name: "this", arg: 1, scope: !1282, type: !752, flags: DIFlagArtificial | DIFlagObjectPointer)
!1285 = !DILocation(line: 0, scope: !1282)
!1286 = !DILocation(line: 291, column: 7, scope: !1287)
!1287 = distinct !DILexicalBlock(scope: !1282, file: !1, line: 291, column: 7)
!1288 = !DILocation(line: 291, column: 7, scope: !1282)
!1289 = distinct !DISubprogram(name: "~CloneableDerived", linkageName: "_ZN16CloneableDerivedD0Ev", scope: !746, file: !1, line: 291, type: !765, scopeLine: 291, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1283, retainedNodes: !240)
!1290 = !DILocalVariable(name: "this", arg: 1, scope: !1289, type: !752, flags: DIFlagArtificial | DIFlagObjectPointer)
!1291 = !DILocation(line: 0, scope: !1289)
!1292 = !DILocation(line: 291, column: 7, scope: !1289)
!1293 = distinct !DISubprogram(name: "clone_self", linkageName: "_ZN16CloneableDerived10clone_selfEv", scope: !746, file: !1, line: 294, type: !750, scopeLine: 294, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !749, retainedNodes: !240)
!1294 = !DILocalVariable(name: "this", arg: 1, scope: !1293, type: !752, flags: DIFlagArtificial | DIFlagObjectPointer)
!1295 = !DILocation(line: 0, scope: !1293)
!1296 = !DILocation(line: 294, column: 54, scope: !1293)
!1297 = !DILocation(line: 294, column: 58, scope: !1293)
!1298 = !DILocation(line: 294, column: 47, scope: !1293)
!1299 = distinct !DISubprogram(name: "~CloneableBase", linkageName: "_ZN13CloneableBaseD2Ev", scope: !734, file: !1, line: 287, type: !738, scopeLine: 287, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !737, retainedNodes: !240)
!1300 = !DILocalVariable(name: "this", arg: 1, scope: !1299, type: !733, flags: DIFlagArtificial | DIFlagObjectPointer)
!1301 = !DILocation(line: 0, scope: !1299)
!1302 = !DILocation(line: 287, column: 38, scope: !1299)
!1303 = distinct !DISubprogram(name: "~CloneableBase", linkageName: "_ZN13CloneableBaseD0Ev", scope: !734, file: !1, line: 287, type: !738, scopeLine: 287, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !737, retainedNodes: !240)
!1304 = !DILocalVariable(name: "this", arg: 1, scope: !1303, type: !733, flags: DIFlagArtificial | DIFlagObjectPointer)
!1305 = !DILocation(line: 0, scope: !1303)
!1306 = !DILocation(line: 287, column: 38, scope: !1303)
!1307 = distinct !DISubprogram(name: "clone_self", linkageName: "_ZN13CloneableBase10clone_selfEv", scope: !734, file: !1, line: 288, type: !742, scopeLine: 288, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !741, retainedNodes: !240)
!1308 = !DILocalVariable(name: "this", arg: 1, scope: !1307, type: !733, flags: DIFlagArtificial | DIFlagObjectPointer)
!1309 = !DILocation(line: 0, scope: !1307)
!1310 = !DILocation(line: 288, column: 50, scope: !1307)
!1311 = !DILocation(line: 288, column: 54, scope: !1307)
!1312 = !DILocation(line: 288, column: 43, scope: !1307)
!1313 = distinct !DISubprogram(name: "CloneableBase", linkageName: "_ZN13CloneableBaseC2ERKS_", scope: !734, file: !1, line: 285, type: !1314, scopeLine: 285, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1318, retainedNodes: !240)
!1314 = !DISubroutineType(types: !1315)
!1315 = !{null, !740, !1316}
!1316 = !DIDerivedType(tag: DW_TAG_reference_type, baseType: !1317, size: 64)
!1317 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !734)
!1318 = !DISubprogram(name: "CloneableBase", scope: !734, type: !1314, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1319 = !DILocalVariable(name: "this", arg: 1, scope: !1313, type: !733, flags: DIFlagArtificial | DIFlagObjectPointer)
!1320 = !DILocation(line: 0, scope: !1313)
!1321 = !DILocalVariable(arg: 2, scope: !1313, type: !1316)
!1322 = !DILocation(line: 285, column: 7, scope: !1313)
!1323 = distinct !DISubprogram(name: "CloneableDerived", linkageName: "_ZN16CloneableDerivedC2ERKS_", scope: !746, file: !1, line: 291, type: !1324, scopeLine: 291, flags: DIFlagArtificial | DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !1328, retainedNodes: !240)
!1324 = !DISubroutineType(types: !1325)
!1325 = !{null, !753, !1326}
!1326 = !DIDerivedType(tag: DW_TAG_reference_type, baseType: !1327, size: 64)
!1327 = !DIDerivedType(tag: DW_TAG_const_type, baseType: !746)
!1328 = !DISubprogram(name: "CloneableDerived", scope: !746, type: !1324, flags: DIFlagPublic | DIFlagArtificial | DIFlagPrototyped, spFlags: 0)
!1329 = !DILocalVariable(name: "this", arg: 1, scope: !1323, type: !752, flags: DIFlagArtificial | DIFlagObjectPointer)
!1330 = !DILocation(line: 0, scope: !1323)
!1331 = !DILocalVariable(arg: 2, scope: !1323, type: !1326)
!1332 = !DILocation(line: 291, column: 7, scope: !1323)
!1333 = distinct !DISubprogram(name: "implementation", linkageName: "_ZN11CRTPDerived14implementationEv", scope: !4, file: !1, line: 244, type: !17, scopeLine: 244, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, declaration: !16, retainedNodes: !240)
!1334 = !DILocalVariable(name: "this", arg: 1, scope: !1333, type: !3, flags: DIFlagArtificial | DIFlagObjectPointer)
!1335 = !DILocation(line: 0, scope: !1333)
!1336 = !DILocation(line: 244, column: 28, scope: !1333)
