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

@.const.0 = private unnamed_addr constant [42 x i8] c"1abc2\0Apqr3stu8vwx\0Aa1b2c3d4e5f\0Atreb7uchet\0A\00"
@sample_input = dso_local constant i8* bitcast ([42 x i8]* @.const.0 to i8*)

define dso_local void @aoc_01_p1() #0 {
.block.0:
	%input = alloca %type.CFile*
	%0 = call %type.CFile*(i8*, i8*) @fopen(i8* noundef bitcast ([10 x i8]* @.const.1 to i8*), i8* noundef bitcast ([2 x i8]* @.const.2 to i8*))
	store %type.CFile* %0, %type.CFile** %input
	%1 = load %type.CFile*, %type.CFile** %input
	%2 = icmp eq %type.CFile* %1, null
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
	%6 = load %type.CFile*, %type.CFile** %input
	%7 = call i8*(i8*, i32, %type.CFile*) @fgets(i8* noundef %5, i32 noundef 100, %type.CFile* noundef %6)
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
	%calibration_value-1 = alloca i32
	%33 = load i32, i32* %calibration_value
	%34 = mul nuw i32 %33, 10
	%35 = load i64, i64* %index
	%36 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %35
	%37 = load i8, i8* %36
	%38 = sub nuw i8 %37, 48
	%39 = zext i8 %38 to i32
	%40 = add nuw i32 %34, %39
	store i32 %40, i32* %calibration_value-1
	%41 = load i32, i32* %calibration_sum
	%42 = load i32, i32* %calibration_value-1
	%43 = add nuw i32 %41, %42
	store i32 %43, i32* %calibration_sum
	br label %.block.3
.block.5:
	%44 = load %type.CFile*, %type.CFile** %input
	%45 = call i32(%type.CFile*) @fclose(%type.CFile* noundef %44)
	%46 = load i32, i32* %calibration_sum
	%47 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.4 to i8*), i32 noundef %46)
	ret void
}

@.const.1 = private unnamed_addr constant [10 x i8] c"day01.txt\00"
@.const.2 = private unnamed_addr constant [2 x i8] c"r\00"
@.const.3 = private unnamed_addr constant [27 x i8] c"unable to open input file\0A\00"
@.const.4 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

%type.Node = type { i8*, %type.Node* }

define dso_local void @omg_linked_list(i8** noundef %.arg.values, i64 noundef %.arg.value_count) #0 {
.block.0:
	%values = alloca i8**
	store i8** %.arg.values, i8*** %values
	%value_count = alloca i64
	store i64 %.arg.value_count, i64* %value_count
	%head = alloca %type.Node*
	store %type.Node* null, %type.Node** %head
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.1
.block.1:
	%0 = load i64, i64* %index
	%1 = load i64, i64* %value_count
	%2 = icmp ult i64 %0, %1
	br i1 %2, label %.block.2, label %.block.3
.block.2:
	%node = alloca %type.Node*
	%3 = call i8*(i64) @malloc(i64 noundef 16)
	%4 = bitcast i8* %3 to %type.Node*
	store %type.Node* %4, %type.Node** %node
	%5 = load %type.Node*, %type.Node** %node
	%6 = load i64, i64* %index
	%7 = load i8**, i8*** %values
	%8 = getelementptr inbounds i8*, i8** %7, i64 %6
	%9 = load i8*, i8** %8
	%10 = load %type.Node*, %type.Node** %head
	%11 = alloca %type.Node
	store %type.Node { i8* undef, %type.Node* undef }, %type.Node* %11
	%12 = getelementptr inbounds %type.Node, %type.Node* %11, i32 0, i32 0
	store i8* %9, i8** %12
	%13 = getelementptr inbounds %type.Node, %type.Node* %11, i32 0, i32 1
	store %type.Node* %10, %type.Node** %13
	%14 = load %type.Node, %type.Node* %11
	store %type.Node %14, %type.Node* %5
	%15 = load %type.Node*, %type.Node** %node
	store %type.Node* %15, %type.Node** %head
	%16 = load i64, i64* %index
	%17 = add nuw i64 %16, 1
	store i64 %17, i64* %index
	br label %.block.1
.block.3:
	%18 = call i32(i8*, ...) @printf(i8* noundef bitcast ([11 x i8]* @.const.5 to i8*))
	br label %.block.4
.block.4:
	%19 = load %type.Node*, %type.Node** %head
	%20 = icmp ne %type.Node* %19, null
	br i1 %20, label %.block.5, label %.block.6
.block.5:
	%node-1 = alloca %type.Node*
	%21 = load %type.Node*, %type.Node** %head
	store %type.Node* %21, %type.Node** %node-1
	%22 = load %type.Node*, %type.Node** %node-1
	%23 = getelementptr inbounds %type.Node, %type.Node* %22, i32 0, i32 0
	%24 = load i8*, i8** %23
	%25 = call i32(i8*, ...) @printf(i8* noundef bitcast ([4 x i8]* @.const.6 to i8*), i8* noundef %24)
	%26 = load %type.Node*, %type.Node** %node-1
	%27 = getelementptr inbounds %type.Node, %type.Node* %26, i32 0, i32 1
	%28 = load %type.Node*, %type.Node** %27
	store %type.Node* %28, %type.Node** %head
	%29 = load %type.Node*, %type.Node** %node-1
	%30 = bitcast %type.Node* %29 to i8*
	call void(i8*) @free(i8* noundef %30)
	br label %.block.4
.block.6:
	ret void
}

@.const.5 = private unnamed_addr constant [11 x i8] c"Reversed:\0A\00"
@.const.6 = private unnamed_addr constant [4 x i8] c"%s\0A\00"

%type.Student = type { i8*, i32, [4 x i32] }

define dso_local i32 @main() #0 {
.block.0:
	call void() @aoc_01_p1()
	%x = alloca i32
	store i32 5, i32* %x
	%x-1 = alloca i32
	%0 = load i32, i32* %x
	store i32 %0, i32* %x-1
	%x-2 = alloca i32
	%1 = load i32, i32* %x-1
	store i32 %1, i32* %x-2
	%2 = load i32, i32* %x-2
	%3 = call i32(i8*, ...) @printf(i8* noundef bitcast ([4 x i8]* @.const.7 to i8*), i32 noundef %2)
	%joe_age = alloca i32
	store i32 97, i32* %joe_age
	%joe_calculus_grade_before_curve = alloca i32
	store i32 47, i32* %joe_calculus_grade_before_curve
	%joe = alloca %type.Student
	%4 = load i32, i32* %joe_age
	%5 = load i32, i32* %joe_calculus_grade_before_curve
	%6 = add nuw i32 %5, 15
	%7 = alloca [4 x i32]
	store [4 x i32] [ i32 80, i32 100, i32 92, i32 undef ], [4 x i32]* %7
	%8 = getelementptr inbounds [4 x i32], [4 x i32]* %7, i32 0, i64 3
	store i32 %6, i32* %8
	%9 = load [4 x i32], [4 x i32]* %7
	%10 = alloca %type.Student
	store %type.Student { i8* bitcast ([9 x i8]* @.const.8 to i8*), i32 undef, [4 x i32] undef }, %type.Student* %10
	%11 = getelementptr inbounds %type.Student, %type.Student* %10, i32 0, i32 1
	store i32 %4, i32* %11
	%12 = getelementptr inbounds %type.Student, %type.Student* %10, i32 0, i32 2
	store [4 x i32] %9, [4 x i32]* %12
	%13 = load %type.Student, %type.Student* %10
	store %type.Student %13, %type.Student* %joe
	%14 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 0
	%15 = load i8*, i8** %14
	%16 = call i32(i8*, ...) @printf(i8* noundef bitcast ([10 x i8]* @.const.9 to i8*), i8* noundef %15)
	%17 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 1
	%18 = load i32, i32* %17
	%19 = call i32(i8*, ...) @printf(i8* noundef bitcast ([9 x i8]* @.const.10 to i8*), i32 noundef %18)
	%20 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%21 = getelementptr inbounds [4 x i32], [4 x i32]* %20, i32 0, i32 0
	%22 = load i32, i32* %21
	%23 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%24 = getelementptr inbounds [4 x i32], [4 x i32]* %23, i32 0, i32 1
	%25 = load i32, i32* %24
	%26 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%27 = getelementptr inbounds [4 x i32], [4 x i32]* %26, i32 0, i32 2
	%28 = load i32, i32* %27
	%29 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%30 = getelementptr inbounds [4 x i32], [4 x i32]* %29, i32 0, i32 3
	%31 = load i32, i32* %30
	%32 = call i32(i8*, ...) @printf(i8* noundef bitcast ([24 x i8]* @.const.11 to i8*), i32 noundef %22, i32 noundef %25, i32 noundef %28, i32 noundef %31)
	%values = alloca [4 x i8*]
	store [4 x i8*] [ i8* bitcast ([8 x i8]* @.const.12 to i8*), i8* bitcast ([8 x i8]* @.const.13 to i8*), i8* bitcast ([8 x i8]* @.const.14 to i8*), i8* bitcast ([8 x i8]* @.const.15 to i8*) ], [4 x i8*]* %values
	%33 = bitcast [4 x i8*]* %values to i8**
	call void(i8**, i64) @omg_linked_list(i8** noundef %33, i64 noundef 4)
	ret i32 0
}

@.const.7 = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.const.8 = private unnamed_addr constant [9 x i8] c"Joe Mama\00"
@.const.9 = private unnamed_addr constant [10 x i8] c"Name: %s\0A\00"
@.const.10 = private unnamed_addr constant [9 x i8] c"Age: %u\0A\00"
@.const.11 = private unnamed_addr constant [24 x i8] c"Grades: %u, %u, %u, %u\0A\00"
@.const.12 = private unnamed_addr constant [8 x i8] c"Value 1\00"
@.const.13 = private unnamed_addr constant [8 x i8] c"Value 2\00"
@.const.14 = private unnamed_addr constant [8 x i8] c"Value 3\00"
@.const.15 = private unnamed_addr constant [8 x i8] c"Value 4\00"

%type.CFile = type opaque

declare i32 @fclose(%type.CFile* noundef) #1

declare i32 @feof(%type.CFile* noundef) #1

declare i8* @fgets(i8* noundef, i32 noundef, %type.CFile* noundef) #1

declare %type.CFile* @fopen(i8* noundef, i8* noundef) #1

declare void @free(i8* noundef) #1

declare i32 @isdigit(i32 noundef) #1

declare i8* @malloc(i64 noundef) #1

declare i32 @printf(i8* noundef, ...) #1

declare i64 @strlen(i8* noundef) #1

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
