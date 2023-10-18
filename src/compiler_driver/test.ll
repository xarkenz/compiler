; ModuleID = './src/compiler_driver/test.ll'
; source_filename = "./src/compiler_driver/test.ll"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@print_int_fstring = private unnamed_addr constant
    [4 x i8] c"%d\0A\00", align 1

define dso_local i32 @main () #0 {
    %1 = alloca i32, align 4
    %2 = alloca i32, align 4
    %3 = alloca i32, align 4
    %4 = alloca i32, align 4
    %5 = alloca i32, align 4
    %6 = alloca i32, align 4
    %7 = alloca i32, align 4
    store i32 2, i32* %1
    store i32 3, i32* %2
    store i32 5, i32* %3
    %8 = load i32, i32* %2
    %9 = load i32, i32* %3
    %10 = mul nsw i32 %8, %9
    store i32 8, i32* %4
    %11 = load i32, i32* %4
    %12 = mul nsw i32 %10, %11
    %13 = load i32, i32* %1
    %14 = add nsw i32 %13, %12
    store i32 13, i32* %5
    %15 = load i32, i32* %5
    %16 = add nsw i32 %14, %15
    store i32 1, i32* %6
    store i32 4, i32* %7
    %17 = load i32, i32* %6
    %18 = load i32, i32* %7
    %19 = mul nsw i32 %17, %18
    %20 = add nsw i32 %16, %19
    %21 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), i32 %20)
    %22 = alloca i32, align 4
    %23 = alloca i32, align 4
    %24 = alloca i32, align 4
    %25 = alloca i32, align 4
    %26 = alloca i32, align 4
    %27 = alloca i32, align 4
    store i32 4, i32* %22
    store i32 80, i32* %23
    store i32 20, i32* %24
    %28 = load i32, i32* %23
    %29 = load i32, i32* %24
    %30 = sdiv i32 %28, %29
    store i32 3, i32* %25
    %31 = load i32, i32* %25
    %32 = sdiv i32 %30, %31
    store i32 7, i32* %26
    %33 = load i32, i32* %26
    %34 = mul nsw i32 %32, %33
    %35 = load i32, i32* %22
    %36 = add nsw i32 %35, %34
    store i32 15, i32* %27
    %37 = load i32, i32* %27
    %38 = sub nsw i32 %36, %37
    %39 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), i32 %38)
    %40 = alloca i32, align 4
    store i32 150395, i32* %40
    %41 = load i32, i32* %40
    %42 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), i32 %41)
    %43 = alloca i32, align 4
    %44 = alloca i32, align 4
    %45 = alloca i32, align 4
    %46 = alloca i32, align 4
    %47 = alloca i32, align 4
    store i32 5, i32* %43
    store i32 4, i32* %44
    %48 = load i32, i32* %43
    %49 = load i32, i32* %44
    %50 = mul nsw i32 %48, %49
    store i32 3, i32* %45
    %51 = load i32, i32* %45
    %52 = mul nsw i32 %50, %51
    store i32 2, i32* %46
    %53 = load i32, i32* %46
    %54 = mul nsw i32 %52, %53
    store i32 1, i32* %47
    %55 = load i32, i32* %47
    %56 = mul nsw i32 %54, %55
    %57 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), i32 %56)
    %58 = alloca i32, align 4
    %59 = alloca i32, align 4
    %60 = alloca i32, align 4
    store i32 1, i32* %58
    store i32 1, i32* %59
    %61 = load i32, i32* %58
    %62 = load i32, i32* %59
    %63 = sub nsw i32 %61, %62
    store i32 1, i32* %60
    %64 = load i32, i32* %60
    %65 = add nsw i32 %63, %64
    %66 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), i32 %65)
    ret i32 0
}

declare i32 @printf(i8*, ...) #1

attributes #0 = {
    noinline nounwind optnone uwtable
    "frame-pointer"="all"
    "min-legal-vector-width"="0"
    "no-trapping-math"="true"
    "stack-protector-buffer-size"="8"
    "target-cpu"="x86-64"
    "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87"
    "tune-cpu"="generic"
}
attributes #1 = {
    "frame-pointer"="all"
    "no-trapping-math"="true"
    "stack-protector-buffer-size"="8"
    "target-cpu"="x86-64"
    "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87"
    "tune-cpu"="generic"
}

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}
!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 10.0.0-4ubuntu1"}
