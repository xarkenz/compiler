source_filename = "./src/compiler_driver/test.txt"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

define dso_local i32 @fibonacci(i32 noundef %.arg.limit) #0 {
	%limit = alloca i32
	store i32 %.arg.limit, i32* %limit
	%a = alloca i32
	store i32 0, i32* %a
	%b = alloca i32
	store i32 1, i32* %b
	br label %.label.0
.label.0:
	%1 = load i32, i32* %b
	%2 = load i32, i32* %limit
	%3 = icmp slt i32 %1, %2
	br i1 %3, label %.label.1, label %.label.2
.label.1:
	%temp = alloca i32
	%4 = load i32, i32* %a
	%5 = load i32, i32* %b
	%6 = add nsw i32 %4, %5
	store i32 %6, i32* %temp
	%7 = load i32, i32* %b
	store i32 %7, i32* %a
	%8 = load i32, i32* %temp
	store i32 %8, i32* %b
	br label %.label.0
.label.2:
	%9 = load i32, i32* %a
	ret i32 %9
}

define dso_local i32 @gcd(i32 noundef %.arg.a, i32 noundef %.arg.b) #0 {
	%a = alloca i32
	store i32 %.arg.a, i32* %a
	%b = alloca i32
	store i32 %.arg.b, i32* %b
	br label %.label.0
.label.0:
	%1 = load i32, i32* %b
	%2 = icmp uge i32 %1, 1
	br i1 %2, label %.label.1, label %.label.2
.label.1:
	%temp = alloca i32
	%3 = load i32, i32* %a
	%4 = load i32, i32* %b
	%5 = urem i32 %3, %4
	store i32 %5, i32* %temp
	%6 = load i32, i32* %b
	store i32 %6, i32* %a
	%7 = load i32, i32* %temp
	store i32 %7, i32* %b
	br label %.label.0
.label.2:
	%8 = load i32, i32* %a
	ret i32 %8
}

@.const.0 = private unnamed_addr constant [42 x i8] c"1abc2\0Apqr3stu8vwx\0Aa1b2c3d4e5f\0Atreb7uchet\0A\00"
@input = dso_local global i8* bitcast ([42 x i8]* @.const.0 to i8*)

define dso_local void @aoc_01_p1() #0 {
	%1 = load i8*, i8** @input
	%2 = call i32(i8*, ...) @printf(i8* noundef %1)
	%input_len = alloca i64
	%3 = load i8*, i8** @input
	%4 = call i64(i8*) @strlen(i8* noundef %3)
	store i64 %4, i64* %input_len
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%start_index = alloca i64
	store i64 0, i64* %start_index
	br label %.label.0
.label.0:
	%5 = load i64, i64* %start_index
	%6 = load i64, i64* %input_len
	%7 = icmp ult i64 %5, %6
	br i1 %7, label %.label.1, label %.label.2
.label.1:
	%index = alloca i64
	%8 = load i64, i64* %start_index
	store i64 %8, i64* %index
	br label %.label.3
.label.3:
	%9 = load i64, i64* %index
	%10 = load i8*, i8** @input
	%11 = getelementptr inbounds i8, i8* %10, i64 %9
	%12 = load i8, i8* %11
	%13 = zext i8 %12 to i32
	%14 = call i32(i32) @isdigit(i32 noundef %13)
	%15 = icmp eq i32 %14, 0
	br i1 %15, label %.label.4, label %.label.5
.label.4:
	%16 = load i64, i64* %index
	%17 = add nuw i64 %16, 1
	store i64 %17, i64* %index
	br label %.label.3
.label.5:
	%calibration_value = alloca i32
	%18 = load i64, i64* %index
	%19 = load i8*, i8** @input
	%20 = getelementptr inbounds i8, i8* %19, i64 %18
	%21 = load i8, i8* %20
	%22 = sub nuw i8 %21, 48
	%23 = zext i8 %22 to i32
	store i32 %23, i32* %calibration_value
	br label %.label.6
.label.6:
	%24 = load i64, i64* %index
	%25 = load i8*, i8** @input
	%26 = getelementptr inbounds i8, i8* %25, i64 %24
	%27 = load i8, i8* %26
	%28 = icmp ne i8 %27, 10
	br i1 %28, label %.label.7, label %.label.8
.label.7:
	%29 = load i64, i64* %index
	%30 = add nuw i64 %29, 1
	store i64 %30, i64* %index
	br label %.label.6
.label.8:
	%31 = load i64, i64* %index
	%32 = add nuw i64 %31, 1
	store i64 %32, i64* %start_index
	%33 = load i64, i64* %index
	%34 = sub nuw i64 %33, 1
	store i64 %34, i64* %index
	br label %.label.9
.label.9:
	%35 = load i64, i64* %index
	%36 = load i8*, i8** @input
	%37 = getelementptr inbounds i8, i8* %36, i64 %35
	%38 = load i8, i8* %37
	%39 = zext i8 %38 to i32
	%40 = call i32(i32) @isdigit(i32 noundef %39)
	%41 = icmp eq i32 %40, 0
	br i1 %41, label %.label.10, label %.label.11
.label.10:
	%42 = load i64, i64* %index
	%43 = sub nuw i64 %42, 1
	store i64 %43, i64* %index
	br label %.label.9
.label.11:
	%44 = load i32, i32* %calibration_value
	%45 = mul nuw i32 %44, 10
	%46 = load i64, i64* %index
	%47 = load i8*, i8** @input
	%48 = getelementptr inbounds i8, i8* %47, i64 %46
	%49 = load i8, i8* %48
	%50 = sub nuw i8 %49, 48
	%51 = zext i8 %50 to i32
	%52 = add nuw i32 %45, %51
	store i32 %52, i32* %calibration_value
	%53 = load i32, i32* %calibration_sum
	%54 = load i32, i32* %calibration_value
	%55 = add nuw i32 %53, %54
	store i32 %55, i32* %calibration_sum
	br label %.label.0
.label.2:
	%56 = load i32, i32* %calibration_sum
	%57 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.1 to i8*), i32 noundef %56)
	ret void
}

@.const.1 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

define dso_local i32 @main() #0 {
	call void() @aoc_01_p1()
	ret i32 0
}

declare i32 @printf(i8* noundef, ...) #1

declare i8* @malloc(i64 noundef) #1

declare void @free(i8* noundef) #1

declare i64 @strlen(i8* noundef) #1

declare i32 @isdigit(i32 noundef) #1

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

!llvm.module.flags = !{ !0, !1, !2, !3, !4 }
!llvm.ident = !{ !5 }
!0 = !{ i32 1, !"wchar_size", i32 4 }
!1 = !{ i32 7, !"PIC Level", i32 2 }
!2 = !{ i32 7, !"PIE Level", i32 2 }
!3 = !{ i32 7, !"uwtable", i32 1 }
!4 = !{ i32 7, !"frame-pointer", i32 2 }
!5 = !{ !"Ubuntu clang version 14.0.0-1ubuntu1.1" }
