source_filename = "./src/compiler_driver/test.txt"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

define dso_local i32 @fibonacci(i32 noundef %.arg.limit) #0 {
.block.0:
	%limit = alloca i32
	store i32 %.arg.limit, i32* %limit
	%a = alloca i32
	store i32 0, i32* %a
	%b = alloca i32
	store i32 1, i32* %b
	br label %.block.1
.block.1:
	%0 = load i32, i32* %b
	%1 = load i32, i32* %limit
	%2 = icmp slt i32 %0, %1
	br i1 %2, label %.block.2, label %.block.3
.block.2:
	%temp = alloca i32
	%3 = load i32, i32* %a
	%4 = load i32, i32* %b
	%5 = add nsw i32 %3, %4
	store i32 %5, i32* %temp
	%6 = load i32, i32* %b
	store i32 %6, i32* %a
	%7 = load i32, i32* %temp
	store i32 %7, i32* %b
	br label %.block.1
.block.3:
	%8 = load i32, i32* %a
	ret i32 %8
}

define dso_local i32 @gcd(i32 noundef %.arg.a, i32 noundef %.arg.b) #0 {
.block.0:
	%a = alloca i32
	store i32 %.arg.a, i32* %a
	%b = alloca i32
	store i32 %.arg.b, i32* %b
	br label %.block.1
.block.1:
	%0 = load i32, i32* %b
	%1 = icmp uge i32 %0, 1
	br i1 %1, label %.block.2, label %.block.3
.block.2:
	%temp = alloca i32
	%2 = load i32, i32* %a
	%3 = load i32, i32* %b
	%4 = urem i32 %2, %3
	store i32 %4, i32* %temp
	%5 = load i32, i32* %b
	store i32 %5, i32* %a
	%6 = load i32, i32* %temp
	store i32 %6, i32* %b
	br label %.block.1
.block.3:
	%7 = load i32, i32* %a
	ret i32 %7
}

@.const.0 = private unnamed_addr constant [42 x i8] c"1abc2\0Apqr3stu8vwx\0Aa1b2c3d4e5f\0Atreb7uchet\0A\00"
@sample_input = dso_local global i8* bitcast ([42 x i8]* @.const.0 to i8*)

define dso_local void @aoc_01_p1() #0 {
.block.0:
	%input = alloca i8*
	%0 = call i8*(i8*, i8*) @fopen(i8* noundef bitcast ([10 x i8]* @.const.1 to i8*), i8* noundef bitcast ([2 x i8]* @.const.2 to i8*))
	store i8* %0, i8** %input
	%1 = load i8*, i8** %input
	%2 = icmp eq i8* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = call i32(i8*, ...) @printf(i8* noundef bitcast ([27 x i8]* @.const.3 to i8*))
	ret void
	br label %.block.2
.block.2:
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%line = alloca [100 x i8]
	br label %.block.3
.block.3:
	%5 = bitcast [100 x i8]* %line to i8*
	%6 = load i8*, i8** %input
	%7 = call i8*(i8*, i32, i8*) @fgets(i8* noundef %5, i32 noundef 100, i8* noundef %6)
	%8 = icmp ne i8* %7, null
	br i1 %8, label %.block.4, label %.block.5
.block.4:
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.6
.block.6:
	%9 = load i64, i64* %index
	%10 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %9
	%11 = load i8, i8* %10
	%12 = zext i8 %11 to i32
	%13 = call i32(i32) @isdigit(i32 noundef %12)
	%14 = icmp eq i32 %13, 0
	br i1 %14, label %.block.7, label %.block.8
.block.7:
	%15 = load i64, i64* %index
	%16 = add nuw i64 %15, 1
	store i64 %16, i64* %index
	br label %.block.6
.block.8:
	%calibration_value = alloca i32
	%17 = load i64, i64* %index
	%18 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %17
	%19 = load i8, i8* %18
	%20 = sub nuw i8 %19, 48
	%21 = zext i8 %20 to i32
	store i32 %21, i32* %calibration_value
	%22 = bitcast [100 x i8]* %line to i8*
	%23 = call i64(i8*) @strlen(i8* noundef %22)
	%24 = sub nuw i64 %23, 1
	store i64 %24, i64* %index
	br label %.block.9
.block.9:
	%25 = load i64, i64* %index
	%26 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %25
	%27 = load i8, i8* %26
	%28 = zext i8 %27 to i32
	%29 = call i32(i32) @isdigit(i32 noundef %28)
	%30 = icmp eq i32 %29, 0
	br i1 %30, label %.block.10, label %.block.11
.block.10:
	%31 = load i64, i64* %index
	%32 = sub nuw i64 %31, 1
	store i64 %32, i64* %index
	br label %.block.9
.block.11:
	%33 = load i32, i32* %calibration_value
	%34 = mul nuw i32 %33, 10
	%35 = load i64, i64* %index
	%36 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %35
	%37 = load i8, i8* %36
	%38 = sub nuw i8 %37, 48
	%39 = zext i8 %38 to i32
	%40 = add nuw i32 %34, %39
	store i32 %40, i32* %calibration_value
	%41 = load i32, i32* %calibration_sum
	%42 = load i32, i32* %calibration_value
	%43 = add nuw i32 %41, %42
	store i32 %43, i32* %calibration_sum
	br label %.block.3
.block.5:
	%44 = load i8*, i8** %input
	%45 = call i32(i8*) @fclose(i8* noundef %44)
	%46 = load i32, i32* %calibration_sum
	%47 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.4 to i8*), i32 noundef %46)
	ret void
}

@.const.1 = private unnamed_addr constant [10 x i8] c"day01.txt\00"
@.const.2 = private unnamed_addr constant [2 x i8] c"r\00"
@.const.3 = private unnamed_addr constant [27 x i8] c"unable to open input file\0A\00"
@.const.4 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

define dso_local i32 @main() #0 {
.block.0:
	call void() @aoc_01_p1()
	ret i32 0
}

declare i8* @fopen(i8* noundef, i8* noundef) #1

declare i32 @fclose(i8* noundef) #1

declare i32 @feof(i8* noundef) #1

declare i8* @fgets(i8* noundef, i32 noundef, i8* noundef) #1

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
