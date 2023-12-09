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

define dso_local i1 @is_digit(i8 noundef %.arg.ch) #0 {
	%ch = alloca i8
	store i8 %.arg.ch, i8* %ch
	%1 = load i8, i8* %ch
	%2 = icmp uge i8 %1, 48
	br i1 %2, label %.label.0, label %.label.1
.label.0:
	%3 = load i8, i8* %ch
	%4 = icmp ule i8 %3, 57
	ret i1 %4
.label.1:
	ret i1 false
}

define dso_local i64 @get_str_len(i8* noundef %.arg.str) #0 {
	%str = alloca i8*
	store i8* %.arg.str, i8** %str
	%index = alloca i64
	store i64 0, i64* %index
	br label %.label.0
.label.0:
	%1 = load i64, i64* %index
	%2 = load i8*, i8** %str
	%3 = getelementptr inbounds i8, i8* %2, i64 %1
	%4 = load i8, i8* %3
	%5 = icmp ne i8 %4, 0
	br i1 %5, label %.label.1, label %.label.2
.label.1:
	%6 = load i64, i64* %index
	%7 = add nuw i64 %6, 1
	store i64 %7, i64* %index
	br label %.label.0
.label.2:
	%8 = load i64, i64* %index
	ret i64 %8
}

@.const.0 = private unnamed_addr constant [42 x i8] c"1abc2\0Apqr3stu8vwx\0Aa1b2c3d4e5f\0Atreb7uchet\0A\00"
@input = dso_local global i8* bitcast ([42 x i8]* @.const.0 to i8*)

define dso_local void @aoc_01_p1() #0 {
	%1 = load i8*, i8** @input
	%2 = call i32(i8*, ...) @printf(i8* noundef %1)
	%input_len = alloca i64
	%3 = load i8*, i8** @input
	%4 = call i64(i8*) @get_str_len(i8* noundef %3)
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
	%13 = call i1(i8) @is_digit(i8 noundef %12)
	%14 = xor i1 %13, true
	br i1 %14, label %.label.4, label %.label.5
.label.4:
	%15 = load i64, i64* %index
	%16 = add nuw i64 %15, 1
	store i64 %16, i64* %index
	br label %.label.3
.label.5:
	%calibration_value = alloca i32
	%17 = load i64, i64* %index
	%18 = load i8*, i8** @input
	%19 = getelementptr inbounds i8, i8* %18, i64 %17
	%20 = load i8, i8* %19
	%21 = sub nuw i8 %20, 48
	%22 = zext i8 %21 to i32
	store i32 %22, i32* %calibration_value
	br label %.label.6
.label.6:
	%23 = load i64, i64* %index
	%24 = load i8*, i8** @input
	%25 = getelementptr inbounds i8, i8* %24, i64 %23
	%26 = load i8, i8* %25
	%27 = icmp ne i8 %26, 10
	br i1 %27, label %.label.7, label %.label.8
.label.7:
	%28 = load i64, i64* %index
	%29 = add nuw i64 %28, 1
	store i64 %29, i64* %index
	br label %.label.6
.label.8:
	%30 = load i64, i64* %index
	%31 = add nuw i64 %30, 1
	store i64 %31, i64* %start_index
	%32 = load i64, i64* %index
	%33 = sub nuw i64 %32, 1
	store i64 %33, i64* %index
	br label %.label.9
.label.9:
	%34 = load i64, i64* %index
	%35 = load i8*, i8** @input
	%36 = getelementptr inbounds i8, i8* %35, i64 %34
	%37 = load i8, i8* %36
	%38 = call i1(i8) @is_digit(i8 noundef %37)
	%39 = xor i1 %38, true
	br i1 %39, label %.label.10, label %.label.11
.label.10:
	%40 = load i64, i64* %index
	%41 = sub nuw i64 %40, 1
	store i64 %41, i64* %index
	br label %.label.9
.label.11:
	%42 = load i32, i32* %calibration_value
	%43 = mul nuw i32 %42, 10
	%44 = load i64, i64* %index
	%45 = load i8*, i8** @input
	%46 = getelementptr inbounds i8, i8* %45, i64 %44
	%47 = load i8, i8* %46
	%48 = sub nuw i8 %47, 48
	%49 = zext i8 %48 to i32
	%50 = add nuw i32 %43, %49
	store i32 %50, i32* %calibration_value
	%51 = load i32, i32* %calibration_sum
	%52 = load i32, i32* %calibration_value
	%53 = add nuw i32 %51, %52
	store i32 %53, i32* %calibration_sum
	br label %.label.0
.label.2:
	%54 = load i32, i32* %calibration_sum
	%55 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.1 to i8*), i32 noundef %54)
	ret void
}

@.const.1 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

define dso_local i32 @main() #0 {
	%1 = load i8*, i8** @input
	%2 = getelementptr inbounds i8, i8* %1, i32 0
