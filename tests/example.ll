; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.1 = private unnamed_addr constant [12 x i8] c"Result: %f\0A\00", align 1

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
  %4 = alloca float, align 4
  %5 = alloca float, align 4
  %6 = alloca float, align 4
  store i32 0, i32* %1, align 4
  call void @do_call(void (i32)* noundef @my_func, i32 noundef 8)
  %7 = call zeroext i1 @func_a()
  br i1 %7, label %8, label %12

8:                                                ; preds = %0
  %9 = call zeroext i1 @func_b()
  br i1 %9, label %10, label %12

10:                                               ; preds = %8
  %11 = call zeroext i1 @func_a()
  br label %12

12:                                               ; preds = %10, %8, %0
  %13 = phi i1 [ false, %8 ], [ false, %0 ], [ %11, %10 ]
  %14 = zext i1 %13 to i8
  store i8 %14, i8* %2, align 1
  %15 = call zeroext i1 @func_a()
  br i1 %15, label %18, label %16

16:                                               ; preds = %12
  %17 = call zeroext i1 @func_b()
  br label %18

18:                                               ; preds = %16, %12
  %19 = phi i1 [ true, %12 ], [ %17, %16 ]
  %20 = zext i1 %19 to i8
  store i8 %20, i8* %3, align 1
  %21 = call zeroext i1 @func_a()
  br i1 %21, label %22, label %25

22:                                               ; preds = %18
  %23 = call zeroext i1 @func_b()
  br i1 %23, label %24, label %25

24:                                               ; preds = %22
  call void @my_func(i32 noundef 1)
  br label %25

25:                                               ; preds = %24, %22, %18
  %26 = call zeroext i1 @func_a()
  br i1 %26, label %29, label %27

27:                                               ; preds = %25
  %28 = call zeroext i1 @func_b()
  br i1 %28, label %29, label %30

29:                                               ; preds = %27, %25
  call void @my_func(i32 noundef 2)
  br label %30

30:                                               ; preds = %29, %27
  store float 3.000000e+00, float* %4, align 4
  store float 6.000000e+00, float* %5, align 4
  %31 = load float, float* %4, align 4
  %32 = load float, float* %5, align 4
  %33 = fadd float %31, %32
  store float %33, float* %6, align 4
  %34 = load float, float* %6, align 4
  %35 = fpext float %34 to double
  %36 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([12 x i8], [12 x i8]* @.str.1, i64 0, i64 0), double noundef %35)
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
