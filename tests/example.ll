; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @my_func(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([4 x i8], [4 x i8]* @.str, i64 0, i64 0), i32 noundef %3)
  ret void
}

declare i32 @printf(i8* noundef, ...) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @do_call(void (i32)* noundef %0, i32 noundef %1) #0 {
  %3 = alloca void (i32)*, align 8
  %4 = alloca i32, align 4
  store void (i32)* %0, void (i32)** %3, align 8
  store i32 %1, i32* %4, align 4
  %5 = load void (i32)*, void (i32)** %3, align 8
  %6 = load i32, i32* %4, align 4
  call void %5(i32 noundef %6)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local zeroext i1 @func_a() #0 {
  ret i1 true
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local zeroext i1 @func_b() #0 {
  ret i1 false
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i8, align 1
  %3 = alloca i8, align 1
  store i32 0, i32* %1, align 4
  call void @do_call(void (i32)* noundef @my_func, i32 noundef 8)
  %4 = call zeroext i1 @func_a()
  br i1 %4, label %5, label %9

5:                                                ; preds = %0
  %6 = call zeroext i1 @func_b()
  br i1 %6, label %7, label %9

7:                                                ; preds = %5
  %8 = call zeroext i1 @func_a()
  br label %9

9:                                                ; preds = %7, %5, %0
  %10 = phi i1 [ false, %5 ], [ false, %0 ], [ %8, %7 ]
  %11 = zext i1 %10 to i8
  store i8 %11, i8* %2, align 1
  %12 = call zeroext i1 @func_a()
  br i1 %12, label %15, label %13

13:                                               ; preds = %9
  %14 = call zeroext i1 @func_b()
  br label %15

15:                                               ; preds = %13, %9
  %16 = phi i1 [ true, %9 ], [ %14, %13 ]
  %17 = zext i1 %16 to i8
  store i8 %17, i8* %3, align 1
  %18 = call zeroext i1 @func_a()
  br i1 %18, label %19, label %22

19:                                               ; preds = %15
  %20 = call zeroext i1 @func_b()
  br i1 %20, label %21, label %22

21:                                               ; preds = %19
  call void @my_func(i32 noundef 1)
  br label %22

22:                                               ; preds = %21, %19, %15
  %23 = call zeroext i1 @func_a()
  br i1 %23, label %26, label %24

24:                                               ; preds = %22
  %25 = call zeroext i1 @func_b()
  br i1 %25, label %26, label %27

26:                                               ; preds = %24, %22
  call void @my_func(i32 noundef 2)
  br label %27

27:                                               ; preds = %26, %24
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
