; ModuleID = './src/compiler_driver/test.txt'
; source_filename = "./src/compiler_driver/test.txt"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@print_i64_fstring = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@print_u64_fstring = private unnamed_addr constant [6 x i8] c"%llu\0A\00", align 1
@print_ptr_fstring = private unnamed_addr constant [4 x i8] c"%p\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @fibonacci(i32 noundef %-arg-limit) #0 {
    %limit = alloca i32, align 4
    store i32 %-arg-limit, i32* %limit, align 4
    %a = alloca i32, align 4
    store i32 0, i32* %a, align 4
    %b = alloca i32, align 4
    store i32 1, i32* %b, align 4
    br label %-L0
-L0:
    %1 = load i32, i32* %b, align 4
    %2 = load i32, i32* %limit, align 4
    %3 = icmp slt i32 %1, %2
    br i1 %3, label %-L1, label %-L2
-L1:
    %temp = alloca i32, align 4
    %4 = load i32, i32* %a, align 4
    %5 = load i32, i32* %b, align 4
    %6 = add nsw i32 %4, %5
    store i32 %6, i32* %temp, align 4
    %7 = load i32, i32* %b, align 4
    store i32 %7, i32* %a, align 4
    %8 = load i32, i32* %temp, align 4
    store i32 %8, i32* %b, align 4
    br label %-L0
-L2:
    %9 = load i32, i32* %a, align 4
    ret i32 %9
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @gcd(i32 noundef %-arg-a, i32 noundef %-arg-b) #0 {
    %a = alloca i32, align 4
    store i32 %-arg-a, i32* %a, align 4
    %b = alloca i32, align 4
    store i32 %-arg-b, i32* %b, align 4
    br label %-L0
-L0:
    %1 = load i32, i32* %b, align 4
    %2 = icmp uge i32 %1, 1
    br i1 %2, label %-L1, label %-L2
-L1:
    %temp = alloca i32, align 4
    %3 = load i32, i32* %a, align 4
    %4 = load i32, i32* %b, align 4
    %5 = urem i32 %3, %4
    store i32 %5, i32* %temp, align 4
    %6 = load i32, i32* %b, align 4
    store i32 %6, i32* %a, align 4
    %7 = load i32, i32* %temp, align 4
    store i32 %7, i32* %b, align 4
    br label %-L0
-L2:
    %8 = load i32, i32* %a, align 4
    ret i32 %8
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
    %1 = call i32(i32) @fibonacci(i32 noundef 1000)
    %2 = sext i32 %1 to i64
    %3 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 noundef %2)
    %4 = call i32(i32, i32) @gcd(i32 noundef 18, i32 noundef 45)
    %5 = zext i32 %4 to i64
    %6 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 noundef %5)
    ret i32 0
}

declare i32 @printf(i8* noundef, ...) #1

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
!5 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
