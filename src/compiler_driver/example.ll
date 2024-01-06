; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.MyStruct = type { i32, i64, i8 }

@my_global_struct = dso_local global %struct.MyStruct zeroinitializer, align 8

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i8, align 1
  %6 = alloca i64, align 8
  %7 = alloca %struct.MyStruct, align 8
  store i32 0, i32* %1, align 4
  store i32 3, i32* %2, align 4
  store i32 2, i32* %3, align 4
  %8 = load i32, i32* %2, align 4
  %9 = load i32, i32* %3, align 4
  %10 = add nsw i32 %9, %8
  store i32 %10, i32* %3, align 4
  store i32 %10, i32* %4, align 4
  %11 = load i32, i32* %2, align 4
  %12 = icmp eq i32 %11, 3
  br i1 %12, label %13, label %16

13:                                               ; preds = %0
  %14 = load i32, i32* %3, align 4
  %15 = icmp eq i32 %14, 5
  br label %16

16:                                               ; preds = %13, %0
  %17 = phi i1 [ false, %0 ], [ %15, %13 ]
  %18 = zext i1 %17 to i8
  store i8 %18, i8* %5, align 1
  %19 = load i32, i32* %2, align 4
  %20 = icmp sgt i32 %19, 2
  br i1 %20, label %24, label %21

21:                                               ; preds = %16
  %22 = load i32, i32* %3, align 4
  %23 = icmp sgt i32 %22, 8
  br label %24

24:                                               ; preds = %21, %16
  %25 = phi i1 [ true, %16 ], [ %23, %21 ]
  %26 = zext i1 %25 to i8
  store i8 %26, i8* %5, align 1
  %27 = load i32, i32* %2, align 4
  %28 = icmp slt i32 %27, 4
  br i1 %28, label %29, label %37

29:                                               ; preds = %24
  %30 = load i32, i32* %3, align 4
  %31 = icmp slt i32 %30, 2
  br i1 %31, label %35, label %32

32:                                               ; preds = %29
  %33 = load i32, i32* %3, align 4
  %34 = icmp sgt i32 %33, 3
  br label %35

35:                                               ; preds = %32, %29
  %36 = phi i1 [ true, %29 ], [ %34, %32 ]
  br label %37

37:                                               ; preds = %35, %24
  %38 = phi i1 [ false, %24 ], [ %36, %35 ]
  %39 = zext i1 %38 to i8
  store i8 %39, i8* %5, align 1
  store i64 34, i64* %6, align 8
  %40 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %7, i32 0, i32 0
  store i32 12, i32* %40, align 8
  %41 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %7, i32 0, i32 1
  %42 = load i64, i64* %6, align 8
  store i64 %42, i64* %41, align 8
  %43 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %7, i32 0, i32 2
  store i8 56, i8* %43, align 8
  %44 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %7, i32 0, i32 0
  store i32 5, i32* %44, align 8
  ret i32 0
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
