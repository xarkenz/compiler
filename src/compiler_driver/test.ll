; ModuleID = './src/compiler_driver/test.ll'
; source_filename = "./src/compiler_driver/test.ll"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@print_i64_fstring = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@print_u64_fstring = private unnamed_addr constant [6 x i8] c"%llu\0A\00", align 1
@print_ptr_fstring = private unnamed_addr constant [4 x i8] c"%p\0A\00", align 1

define dso_local i32 @main() #0 {
    %x = alloca i8, align 4
    %1 = mul nsw i8 5, 6
    %2 = add nsw i8 %1, 3
    store i8 %2, i8* %x
    %3 = mul nsw i8 6, 3
    %4 = add nsw i8 %3, 5
    store i8 %4, i8* %x
    %y = alloca i16, align 4
    %5 = load i8, i8* %x
    %6 = sub nsw i8 %5, 7
    %7 = sext i8 %6 to i16
    %8 = mul nsw i16 2, %7
    %9 = add nsw i16 9, %8
    store i16 %9, i16* %y
    %10 = load i8, i8* %x
    %11 = sext i8 %10 to i64
    %12 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 %11)
    %13 = load i16, i16* %y
    %14 = sext i16 %13 to i64
    %15 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 %14)
    %x0 = alloca i16, align 4
    %16 = load i8, i8* %x
    %17 = sext i8 %16 to i16
    store i16 %17, i16* %x0
    %18 = load i16, i16* %x0
    %19 = load i16, i16* %y
    %20 = icmp eq i16 %18, %19
    %21 = zext i1 %20 to i64
    %22 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 %21)
    %23 = load i16, i16* %x0
    %24 = load i16, i16* %y
    %25 = icmp ne i16 %23, %24
    %26 = zext i1 %25 to i64
    %27 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 %26)
    %28 = load i16, i16* %x0
    %29 = load i16, i16* %y
    %30 = icmp slt i16 %28, %29
    %31 = zext i1 %30 to i64
    %32 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 %31)
    %33 = load i16, i16* %x0
    %34 = load i16, i16* %y
    %35 = icmp sgt i16 %33, %34
    %36 = zext i1 %35 to i64
    %37 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 %36)
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
