; ModuleID = './src/compiler_driver/test.ll'
; source_filename = "./src/compiler_driver/test.ll"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@print_i64_fstring = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@print_u64_fstring = private unnamed_addr constant [6 x i8] c"%llu\0A\00", align 1
@print_ptr_fstring = private unnamed_addr constant [4 x i8] c"%p\0A\00", align 1

define dso_local i32 @main() #0 {
    %a = alloca i32, align 4
    store i32 0, i32* %a
    %b = alloca i32, align 4
    store i32 1, i32* %b
    br label %-L1
-L1:
    %1 = load i32, i32* %b
    %2 = icmp slt i32 %1, 1000
    br i1 %2, label %-L2, label %-L3
-L2:
    %3 = load i32, i32* %b
    %4 = sext i32 %3 to i64
    %5 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 %4)
    %temp = alloca i32, align 4
    %6 = load i32, i32* %a
    %7 = load i32, i32* %b
    %8 = add nsw i32 %6, %7
    store i32 %8, i32* %temp
    %9 = load i32, i32* %b
    store i32 %9, i32* %a
    %10 = load i32, i32* %temp
    store i32 %10, i32* %b
    br label %-L1
-L3:
    %11 = load i32, i32* %b
    %12 = icmp sgt i32 %11, 950
    br i1 %12, label %-L4, label %-L5
-L4:
    %13 = sext i32 0 to i64
    %14 = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 %13)
    br label %-L5
-L5:
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
