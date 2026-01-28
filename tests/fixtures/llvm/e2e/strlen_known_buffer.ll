; ModuleID = 'strlen_known_buffer'
source_filename = "strlen_known_buffer.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i64 @strlen(ptr)

define i64 @test_strlen_known() {
entry:
  %buf = alloca [11 x i8], align 1
  %ptr = getelementptr inbounds [11 x i8], ptr %buf, i64 0, i64 0
  %len = call i64 @strlen(ptr %ptr)
  ret i64 %len
}
