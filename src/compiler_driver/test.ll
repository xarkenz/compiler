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

define dso_local void @do_some_pointing() #0 {
	%a = alloca i32
	%b = alloca i32
	%c = alloca i32
	store i32 3, i32* %c
	store i32 3, i32* %b
	store i32 3, i32* %a
	%1 = load i32, i32* %a
	%2 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([7 x i8], [7 x i8]* @.const.0, i32 0, i32 0), i32 noundef %1)
	%3 = load i32, i32* %b
	%4 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([7 x i8], [7 x i8]* @.const.1, i32 0, i32 0), i32 noundef %3)
	%5 = load i32, i32* %c
	%6 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([7 x i8], [7 x i8]* @.const.2, i32 0, i32 0), i32 noundef %5)
	%x = alloca i32
	store i32 0, i32* %x
	%7 = load i32, i32* %x
	%8 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([7 x i8], [7 x i8]* @.const.3, i32 0, i32 0), i32 noundef %7)
	%y = alloca i32*
	store i32* %x, i32** %y
	%9 = load i32*, i32** %y
	store i32 1, i32* %9
	%10 = load i32, i32* %x
	%11 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([8 x i8], [8 x i8]* @.const.4, i32 0, i32 0), i32 noundef %10)
	%z = alloca i32**
	store i32** %y, i32*** %z
	%12 = load i32**, i32*** %z
	%13 = load i32*, i32** %12
	store i32 2, i32* %13
	%14 = load i32, i32* %x
	%15 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([9 x i8], [9 x i8]* @.const.5, i32 0, i32 0), i32 noundef %14)
	ret void
}

@.const.0 = private unnamed_addr constant [7 x i8] c"a: %d\0A\00"

@.const.1 = private unnamed_addr constant [7 x i8] c"b: %d\0A\00"

@.const.2 = private unnamed_addr constant [7 x i8] c"c: %d\0A\00"

@.const.3 = private unnamed_addr constant [7 x i8] c"x: %d\0A\00"

@.const.4 = private unnamed_addr constant [8 x i8] c"*y: %d\0A\00"

@.const.5 = private unnamed_addr constant [9 x i8] c"**z: %d\0A\00"

define dso_local i32 @main() #0 {
	%1 = call i32(i32) @fibonacci(i32 noundef 1000)
	%2 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([15 x i8], [15 x i8]* @.const.6, i32 0, i32 0), i32 noundef %1)
	%3 = call i32(i32, i32) @gcd(i32 noundef 18, i32 noundef 45)
	%4 = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([9 x i8], [9 x i8]* @.const.7, i32 0, i32 0), i32 noundef %3)
	call void() @do_some_pointing()
	ret i32 0
}

@.const.6 = private unnamed_addr constant [15 x i8] c"fibonacci: %d\0A\00"

@.const.7 = private unnamed_addr constant [9 x i8] c"gcd: %u\0A\00"

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

!llvm.module.flags = !{ !0, !1, !2, !3, !4 }
!llvm.ident = !{ !5 }
!0 = !{ i32 1, !"wchar_size", i32 4 }
!1 = !{ i32 7, !"PIC Level", i32 2 }
!2 = !{ i32 7, !"PIE Level", i32 2 }
!3 = !{ i32 7, !"uwtable", i32 1 }
!4 = !{ i32 7, !"frame-pointer", i32 2 }
!5 = !{ !"Ubuntu clang version 14.0.0-1ubuntu1.1" }
