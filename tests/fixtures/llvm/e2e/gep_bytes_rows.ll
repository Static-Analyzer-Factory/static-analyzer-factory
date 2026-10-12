; Hand-written fixture for the frontend's byte-arithmetic GEP extension and
; global layouts (plan 222 §2q). ILP32-style layout: a 2-D float table read
; through a decayed row pointer, and a struct with a padded field.
target datalayout = "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128"
target triple = "i386-pc-linux-gnu"

%struct.S = type { i8, i32, [2 x i16] }

@W = internal constant [3 x [2 x float]] [[2 x float] [float 1.0, float 2.0], [2 x float] [float 3.0, float 4.0], [2 x float] [float 5.0, float 6.0]], align 4
@s = internal global %struct.S zeroinitializer, align 4

define internal float @get(ptr %p, i32 %c, i32 %i) {
  %row = getelementptr inbounds [2 x float], ptr %p, i32 %c
  %e = getelementptr inbounds [2 x float], ptr %row, i32 0, i32 %i
  %v = load float, ptr %e, align 4
  ret float %v
}

define i32 @main() {
  %v = call float @get(ptr @W, i32 0, i32 1)
  %f = getelementptr inbounds %struct.S, ptr @s, i32 0, i32 2, i32 1
  store i16 7, ptr %f, align 2
  ret i32 0
}
