; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@my_string_array = dso_local constant [26 x i8] c"assigning string to array\00", align 16
@.str = private unnamed_addr constant [28 x i8] c"assigning string to pointer\00", align 1
@my_string_ptr = dso_local constant i8* getelementptr inbounds ([28 x i8], [28 x i8]* @.str, i32 0, i32 0), align 8
@my_other_array = dso_local constant [5 x i32] [i32 104, i32 101, i32 108, i32 108, i32 111], align 16
@.compoundliteral = internal constant [5 x i32] [i32 119, i32 111, i32 114, i32 108, i32 100], align 4
@my_other_ptr = dso_local constant i32* getelementptr inbounds ([5 x i32], [5 x i32]* @.compoundliteral, i32 0, i32 0), align 8
@__const.main.my_local_array = private unnamed_addr constant [3 x i16] [i16 1, i16 2, i16 3], align 2
@__const.main.format = private unnamed_addr constant [10 x i8] c"test: %d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [3 x i16], align 2
  %3 = alloca [10 x i8], align 1
  store i32 0, i32* %1, align 4
  %4 = bitcast [3 x i16]* %2 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 2 %4, i8* align 2 bitcast ([3 x i16]* @__const.main.my_local_array to i8*), i64 6, i1 false)
  %5 = bitcast [10 x i8]* %3 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %5, i8* align 1 getelementptr inbounds ([10 x i8], [10 x i8]* @__const.main.format, i32 0, i32 0), i64 10, i1 false)
  %6 = getelementptr inbounds [10 x i8], [10 x i8]* %3, i64 0, i64 0
  %7 = call i32 (i8*, ...) @printf(i8* noundef %6, i32 noundef 42)
  ret i32 0
}

; Function Attrs: argmemonly nofree nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #1

declare i32 @printf(i8* noundef, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { argmemonly nofree nounwind willreturn }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
