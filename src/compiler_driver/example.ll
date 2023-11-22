; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [5 x i8] c"%ld\0A\00", align 1
@.str.1 = private unnamed_addr constant [5 x i8] c"%lu\0A\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%p\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @print_i64(i64 noundef %0) #0 {
  %2 = alloca i64, align 8
  store i64 %0, i64* %2, align 8
  %3 = load i64, i64* %2, align 8
  %4 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i64 noundef %3)
  ret void
}

declare i32 @printf(i8* noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @print_u64(i64 noundef %0) #0 {
  %2 = alloca i64, align 8
  store i64 %0, i64* %2, align 8
  %3 = load i64, i64* %2, align 8
  %4 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([5 x i8], [5 x i8]* @.str.1, i64 0, i64 0), i64 noundef %3)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @print_ptr(i8* noundef %0) #0 {
  %2 = alloca i8*, align 8
  store i8* %0, i8** %2, align 8
  %3 = load i8*, i8** %2, align 8
  %4 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i8* noundef %3)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @fibonacci(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  store i32 0, i32* %3, align 4
  store i32 1, i32* %4, align 4
  br label %6

6:                                                ; preds = %10, %1
  %7 = load i32, i32* %4, align 4
  %8 = load i32, i32* %2, align 4
  %9 = icmp slt i32 %7, %8
  br i1 %9, label %10, label %16

10:                                               ; preds = %6
  %11 = load i32, i32* %3, align 4
  %12 = load i32, i32* %4, align 4
  %13 = add nsw i32 %11, %12
  store i32 %13, i32* %5, align 4
  %14 = load i32, i32* %4, align 4
  store i32 %14, i32* %3, align 4
  %15 = load i32, i32* %5, align 4
  store i32 %15, i32* %4, align 4
  br label %6, !llvm.loop !6

16:                                               ; preds = %6
  %17 = load i32, i32* %3, align 4
  ret i32 %17
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @gcd(i32 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, i32* %3, align 4
  store i32 %1, i32* %4, align 4
  br label %6

6:                                                ; preds = %9, %2
  %7 = load i32, i32* %4, align 4
  %8 = icmp uge i32 %7, 1
  br i1 %8, label %9, label %15

9:                                                ; preds = %6
  %10 = load i32, i32* %3, align 4
  %11 = load i32, i32* %4, align 4
  %12 = urem i32 %10, %11
  store i32 %12, i32* %5, align 4
  %13 = load i32, i32* %4, align 4
  store i32 %13, i32* %3, align 4
  %14 = load i32, i32* %5, align 4
  store i32 %14, i32* %4, align 4
  br label %6, !llvm.loop !8

15:                                               ; preds = %6
  %16 = load i32, i32* %3, align 4
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i8, align 1
  %3 = alloca i8, align 1
  %4 = alloca i8, align 1
  %5 = alloca i8, align 1
  store i32 0, i32* %1, align 4
  store i8 1, i8* %2, align 1
  store i8 0, i8* %3, align 1
  store i8 1, i8* %4, align 1
  %6 = load i8, i8* %2, align 1
  %7 = trunc i8 %6 to i1
  br i1 %7, label %8, label %11

8:                                                ; preds = %0
  %9 = load i8, i8* %3, align 1
  %10 = trunc i8 %9 to i1
  br i1 %10, label %14, label %11

11:                                               ; preds = %8, %0
  %12 = load i8, i8* %4, align 1
  %13 = trunc i8 %12 to i1
  br label %14

14:                                               ; preds = %11, %8
  %15 = phi i1 [ true, %8 ], [ %13, %11 ]
  %16 = zext i1 %15 to i8
  store i8 %16, i8* %5, align 1
  %17 = load i8, i8* %2, align 1
  %18 = trunc i8 %17 to i1
  br i1 %18, label %19, label %22

19:                                               ; preds = %14
  %20 = load i8, i8* %3, align 1
  %21 = trunc i8 %20 to i1
  br i1 %21, label %25, label %22

22:                                               ; preds = %19, %14
  %23 = load i8, i8* %4, align 1
  %24 = trunc i8 %23 to i1
  br i1 %24, label %25, label %28

25:                                               ; preds = %22, %19
  %26 = call i32 @fibonacci(i32 noundef 1000)
  %27 = sext i32 %26 to i64
  call void @print_i64(i64 noundef %27)
  br label %31

28:                                               ; preds = %22
  %29 = call i32 @gcd(i32 noundef 18, i32 noundef 45)
  %30 = zext i32 %29 to i64
  call void @print_u64(i64 noundef %30)
  br label %31

31:                                               ; preds = %28, %25
  ret i32 0
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
!6 = distinct !{!6, !7}
!7 = !{!"llvm.loop.mustprogress"}
!8 = distinct !{!8, !7}
