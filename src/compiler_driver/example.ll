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
  %4 = alloca i8, align 1
  %5 = alloca i64, align 8
  %6 = alloca %struct.MyStruct, align 8
  store i32 0, i32* %1, align 4
  store i32 3, i32* %2, align 4
  store i32 5, i32* %3, align 4
  %7 = load i32, i32* %2, align 4
  %8 = icmp eq i32 %7, 3
  br i1 %8, label %9, label %12

9:                                                ; preds = %0
  %10 = load i32, i32* %3, align 4
  %11 = icmp eq i32 %10, 5
  br label %12

12:                                               ; preds = %9, %0
  %13 = phi i1 [ false, %0 ], [ %11, %9 ]
  %14 = zext i1 %13 to i8
  store i8 %14, i8* %4, align 1
  %15 = load i32, i32* %2, align 4
  %16 = icmp sgt i32 %15, 2
  br i1 %16, label %20, label %17

17:                                               ; preds = %12
  %18 = load i32, i32* %3, align 4
  %19 = icmp sgt i32 %18, 8
  br label %20

20:                                               ; preds = %17, %12
  %21 = phi i1 [ true, %12 ], [ %19, %17 ]
  %22 = zext i1 %21 to i8
  store i8 %22, i8* %4, align 1
  %23 = load i32, i32* %2, align 4
  %24 = icmp slt i32 %23, 4
  br i1 %24, label %25, label %33

25:                                               ; preds = %20
  %26 = load i32, i32* %3, align 4
  %27 = icmp slt i32 %26, 2
  br i1 %27, label %31, label %28

28:                                               ; preds = %25
  %29 = load i32, i32* %3, align 4
  %30 = icmp sgt i32 %29, 3
  br label %31

31:                                               ; preds = %28, %25
  %32 = phi i1 [ true, %25 ], [ %30, %28 ]
  br label %33

33:                                               ; preds = %31, %20
  %34 = phi i1 [ false, %20 ], [ %32, %31 ]
  %35 = zext i1 %34 to i8
  store i8 %35, i8* %4, align 1
  store i64 34, i64* %5, align 8
  %36 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %6, i32 0, i32 0
  store i32 12, i32* %36, align 8
  %37 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %6, i32 0, i32 1
  %38 = load i64, i64* %5, align 8
  store i64 %38, i64* %37, align 8
  %39 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %6, i32 0, i32 2
  store i8 56, i8* %39, align 8
  %40 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %6, i32 0, i32 0
  store i32 5, i32* %40, align 8
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
