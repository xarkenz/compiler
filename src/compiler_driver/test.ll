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
@input = dso_local global i8* bitcast ([42 x i8]* @.const.0 to i8*)

define dso_local void @aoc_01_p1() #0 {
.block.0:
	%0 = load i8*, i8** @input
	%1 = call i32(i8*, ...) @printf(i8* noundef %0)
	%input_len = alloca i64
	%2 = load i8*, i8** @input
	%3 = call i64(i8*) @strlen(i8* noundef %2)
	store i64 %3, i64* %input_len
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%start_index = alloca i64
	store i64 0, i64* %start_index
	br label %.block.1
.block.1:
	%4 = load i64, i64* %start_index
	%5 = load i64, i64* %input_len
	%6 = icmp ult i64 %4, %5
	br i1 %6, label %.block.2, label %.block.3
.block.2:
	%index = alloca i64
	%7 = load i64, i64* %start_index
	store i64 %7, i64* %index
	br label %.block.4
.block.4:
	%8 = load i64, i64* %index
	%9 = load i8*, i8** @input
	%10 = getelementptr inbounds i8, i8* %9, i64 %8
	%11 = load i8, i8* %10
	%12 = zext i8 %11 to i32
	%13 = call i32(i32) @isdigit(i32 noundef %12)
	%14 = icmp eq i32 %13, 0
	br i1 %14, label %.block.5, label %.block.6
.block.5:
	%15 = load i64, i64* %index
	%16 = add nuw i64 %15, 1
	store i64 %16, i64* %index
	br label %.block.4
.block.6:
	%calibration_value = alloca i32
	%17 = load i64, i64* %index
	%18 = load i8*, i8** @input
	%19 = getelementptr inbounds i8, i8* %18, i64 %17
	%20 = load i8, i8* %19
	%21 = sub nuw i8 %20, 48
	%22 = zext i8 %21 to i32
	store i32 %22, i32* %calibration_value
	br label %.block.7
.block.7:
	%23 = load i64, i64* %index
	%24 = load i8*, i8** @input
	%25 = getelementptr inbounds i8, i8* %24, i64 %23
	%26 = load i8, i8* %25
	%27 = icmp ne i8 %26, 10
	br i1 %27, label %.block.8, label %.block.9
.block.8:
	%28 = load i64, i64* %index
	%29 = add nuw i64 %28, 1
	store i64 %29, i64* %index
	br label %.block.7
.block.9:
	%30 = load i64, i64* %index
	%31 = add nuw i64 %30, 1
	store i64 %31, i64* %start_index
	%32 = load i64, i64* %index
	%33 = sub nuw i64 %32, 1
	store i64 %33, i64* %index
	br label %.block.10
.block.10:
	%34 = load i64, i64* %index
	%35 = load i8*, i8** @input
	%36 = getelementptr inbounds i8, i8* %35, i64 %34
	%37 = load i8, i8* %36
	%38 = zext i8 %37 to i32
	%39 = call i32(i32) @isdigit(i32 noundef %38)
	%40 = icmp eq i32 %39, 0
	br i1 %40, label %.block.11, label %.block.12
.block.11:
	%41 = load i64, i64* %index
	%42 = sub nuw i64 %41, 1
	store i64 %42, i64* %index
	br label %.block.10
.block.12:
	%43 = load i32, i32* %calibration_value
	%44 = mul nuw i32 %43, 10
	%45 = load i64, i64* %index
	%46 = load i8*, i8** @input
	%47 = getelementptr inbounds i8, i8* %46, i64 %45
	%48 = load i8, i8* %47
	%49 = sub nuw i8 %48, 48
	%50 = zext i8 %49 to i32
	%51 = add nuw i32 %44, %50
	store i32 %51, i32* %calibration_value
	%52 = load i32, i32* %calibration_sum
	%53 = load i32, i32* %calibration_value
	%54 = add nuw i32 %52, %53
	store i32 %54, i32* %calibration_sum
	br label %.block.1
.block.3:
	%55 = load i32, i32* %calibration_sum
	%56 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.1 to i8*), i32 noundef %55)
	ret void
}

@.const.1 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

define dso_local i32 @main() #0 {
.block.0:
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
