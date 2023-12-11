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
	%2 = icmp ult i32 %0, %1
	br i1 %2, label %.block.2, label %.block.3
.block.2:
	%temp = alloca i32
	%3 = load i32, i32* %a
	%4 = load i32, i32* %b
	%5 = add nuw i32 %3, %4
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

