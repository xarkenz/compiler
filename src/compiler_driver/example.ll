; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i8, align 1
  store i32 0, i32* %1, align 4
  store i32 3, i32* %2, align 4
  store i32 5, i32* %3, align 4
  %5 = load i32, i32* %2, align 4
  %6 = icmp eq i32 %5, 3
  br i1 %6, label %7, label %10

7:                                                ; preds = %0
  %8 = load i32, i32* %3, align 4
  %9 = icmp eq i32 %8, 5
  br label %10

10:                                               ; preds = %7, %0
  %11 = phi i1 [ false, %0 ], [ %9, %7 ]
  %12 = zext i1 %11 to i8
  store i8 %12, i8* %4, align 1
  %13 = load i32, i32* %2, align 4
  %14 = icmp sgt i32 %13, 2
  br i1 %14, label %18, label %15

15:                                               ; preds = %10
  %16 = load i32, i32* %3, align 4
  %17 = icmp sgt i32 %16, 8
  br label %18

18:                                               ; preds = %15, %10
  %19 = phi i1 [ true, %10 ], [ %17, %15 ]
  %20 = zext i1 %19 to i8
  store i8 %20, i8* %4, align 1
  %21 = load i32, i32* %2, align 4
  %22 = icmp slt i32 %21, 4
  br i1 %22, label %23, label %31

23:                                               ; preds = %18
  %24 = load i32, i32* %3, align 4
  %25 = icmp slt i32 %24, 2
  br i1 %25, label %29, label %26

26:                                               ; preds = %23
  %27 = load i32, i32* %3, align 4
  %28 = icmp sgt i32 %27, 3
  br label %29

29:                                               ; preds = %26, %23
  %30 = phi i1 [ true, %23 ], [ %28, %26 ]
  br label %31

31:                                               ; preds = %29, %18
  %32 = phi i1 [ false, %18 ], [ %30, %29 ]
  %33 = zext i1 %32 to i8
  store i8 %33, i8* %4, align 1
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
