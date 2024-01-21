; module_id = 0
source_filename = "./src/compiler_driver/test.txt"

target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

define dso_local i64 @u64_max(i64 noundef %.arg.self, i64 noundef %.arg.other) #0 {
.block.0:
	%self = alloca i64
	store i64 %.arg.self, i64* %self
	%other = alloca i64
	store i64 %.arg.other, i64* %other
	%0 = load i64, i64* %self
	%1 = load i64, i64* %other
	%2 = icmp ugt i64 %0, %1
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = load i64, i64* %self
	ret i64 %3
.block.2:
	%4 = load i64, i64* %other
	ret i64 %4
}

%type.Str = type { i8*, i64 }

%type.MutStr = type { i8*, i64 }

%type.String = type { %type.MutStr, i64 }

define dso_local %type.String @String_new() #0 {
.block.0:
	ret %type.String { %type.MutStr { i8* null, i64 0 }, i64 0 }
}

define dso_local void @String_del(%type.String noundef %.arg.self) #0 {
.block.0:
	%self = alloca %type.String
	store %type.String %.arg.self, %type.String* %self
	%0 = getelementptr inbounds %type.String, %type.String* %self, i32 0, i32 0
	%1 = getelementptr inbounds %type.MutStr, %type.MutStr* %0, i32 0, i32 0
	%2 = load i8*, i8** %1
	%3 = bitcast i8* %2 to i8*
	call void(i8*) @free(i8* noundef %3)
	ret void
}

define dso_local %type.Str @String_as_str(%type.String* noundef %.arg.self) #0 {
.block.0:
	%self = alloca %type.String*
	store %type.String* %.arg.self, %type.String** %self
	%0 = load %type.String*, %type.String** %self
	%1 = getelementptr inbounds %type.String, %type.String* %0, i32 0, i32 0
	%2 = getelementptr inbounds %type.MutStr, %type.MutStr* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = load %type.String*, %type.String** %self
	%5 = getelementptr inbounds %type.String, %type.String* %4, i32 0, i32 0
	%6 = getelementptr inbounds %type.MutStr, %type.MutStr* %5, i32 0, i32 1
	%7 = load i64, i64* %6
	%8 = alloca %type.Str
	store %type.Str { i8* undef, i64 undef }, %type.Str* %8
	%9 = getelementptr inbounds %type.Str, %type.Str* %8, i32 0, i32 0
	store i8* %3, i8** %9
	%10 = getelementptr inbounds %type.Str, %type.Str* %8, i32 0, i32 1
	store i64 %7, i64* %10
	%11 = load %type.Str, %type.Str* %8
	ret %type.Str %11
}

define dso_local void @String_grow_by(%type.String* noundef %.arg.self, i64 noundef %.arg.additional) #0 {
.block.0:
	%self = alloca %type.String*
	store %type.String* %.arg.self, %type.String** %self
	%additional = alloca i64
	store i64 %.arg.additional, i64* %additional
	%required_capacity = alloca i64
	%0 = load %type.String*, %type.String** %self
	%1 = getelementptr inbounds %type.String, %type.String* %0, i32 0, i32 1
	%2 = load i64, i64* %1
	%3 = load i64, i64* %additional
	%4 = add nuw i64 %2, %3
	store i64 %4, i64* %required_capacity
	%capacity = alloca i64
	%5 = load %type.String*, %type.String** %self
	%6 = getelementptr inbounds %type.String, %type.String* %5, i32 0, i32 1
	%7 = load i64, i64* %6
	%8 = mul nuw i64 %7, 2
	%9 = load i64, i64* %required_capacity
	%10 = call i64(i64, i64) @u64_max(i64 noundef %8, i64 noundef %9)
	store i64 %10, i64* %capacity
	%ptr = alloca i8*
	%11 = load i64, i64* %capacity
	%12 = mul nuw i64 1, %11
	%13 = call i8*(i64) @malloc(i64 noundef %12)
	%14 = bitcast i8* %13 to i8*
	store i8* %14, i8** %ptr
	%15 = load i8*, i8** %ptr
	%16 = bitcast i8* %15 to i8*
	%17 = load %type.String*, %type.String** %self
	%18 = getelementptr inbounds %type.String, %type.String* %17, i32 0, i32 0
	%19 = getelementptr inbounds %type.MutStr, %type.MutStr* %18, i32 0, i32 0
	%20 = load i8*, i8** %19
	%21 = bitcast i8* %20 to i8*
	%22 = load %type.String*, %type.String** %self
	%23 = getelementptr inbounds %type.String, %type.String* %22, i32 0, i32 0
	%24 = getelementptr inbounds %type.MutStr, %type.MutStr* %23, i32 0, i32 1
	%25 = load i64, i64* %24
	%26 = call i8*(i8*, i8*, i64) @memcpy(i8* noundef %16, i8* noundef %21, i64 noundef %25)
	%27 = load %type.String*, %type.String** %self
	%28 = getelementptr inbounds %type.String, %type.String* %27, i32 0, i32 0
	%29 = getelementptr inbounds %type.MutStr, %type.MutStr* %28, i32 0, i32 0
	%30 = load i8*, i8** %29
	%31 = bitcast i8* %30 to i8*
	call void(i8*) @free(i8* noundef %31)
	%32 = load %type.String*, %type.String** %self
	%33 = getelementptr inbounds %type.String, %type.String* %32, i32 0, i32 0
	%34 = getelementptr inbounds %type.MutStr, %type.MutStr* %33, i32 0, i32 0
	%35 = load i8*, i8** %ptr
	store i8* %35, i8** %34
	%36 = load %type.String*, %type.String** %self
	%37 = getelementptr inbounds %type.String, %type.String* %36, i32 0, i32 1
	%38 = load i64, i64* %capacity
	store i64 %38, i64* %37
	ret void
}

define dso_local void @String_push(%type.String* noundef %.arg.self, i8 noundef %.arg.ch) #0 {
.block.0:
	%self = alloca %type.String*
	store %type.String* %.arg.self, %type.String** %self
	%ch = alloca i8
	store i8 %.arg.ch, i8* %ch
	%0 = load %type.String*, %type.String** %self
	%1 = getelementptr inbounds %type.String, %type.String* %0, i32 0, i32 0
	%2 = getelementptr inbounds %type.MutStr, %type.MutStr* %1, i32 0, i32 1
	%3 = load i64, i64* %2
	%4 = load %type.String*, %type.String** %self
	%5 = getelementptr inbounds %type.String, %type.String* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = icmp eq i64 %3, %6
	br i1 %7, label %.block.1, label %.block.2
.block.1:
	%8 = load %type.String*, %type.String** %self
	call void(%type.String*, i64) @String_grow_by(%type.String* noundef %8, i64 noundef 1)
	br label %.block.2
.block.2:
	%9 = load %type.String*, %type.String** %self
	%10 = getelementptr inbounds %type.String, %type.String* %9, i32 0, i32 0
	%11 = getelementptr inbounds %type.MutStr, %type.MutStr* %10, i32 0, i32 0
	%12 = load %type.String*, %type.String** %self
	%13 = getelementptr inbounds %type.String, %type.String* %12, i32 0, i32 0
	%14 = getelementptr inbounds %type.MutStr, %type.MutStr* %13, i32 0, i32 1
	%15 = load i64, i64* %14
	%16 = load i8*, i8** %11
	%17 = getelementptr inbounds i8, i8* %16, i64 %15
	%18 = load i8, i8* %ch
	store i8 %18, i8* %17
	%19 = load %type.String*, %type.String** %self
	%20 = getelementptr inbounds %type.String, %type.String* %19, i32 0, i32 0
	%21 = getelementptr inbounds %type.MutStr, %type.MutStr* %20, i32 0, i32 1
	%22 = load i64, i64* %21
	%23 = add nuw i64 %22, 1
	store i64 %23, i64* %21
	ret void
}

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

define dso_local void @aoc_01_p1() #0 {
.block.0:
	%input = alloca %type.CFile*
	%0 = call %type.CFile*(i8*, i8*) @fopen(i8* noundef bitcast ([10 x i8]* @.const.0 to i8*), i8* noundef bitcast ([2 x i8]* @.const.1 to i8*))
	store %type.CFile* %0, %type.CFile** %input
	%1 = load %type.CFile*, %type.CFile** %input
	%2 = icmp eq %type.CFile* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = call i32(i8*, ...) @printf(i8* noundef bitcast ([27 x i8]* @.const.2 to i8*))
	ret void
.block.2:
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%line = alloca [100 x i8]
	br label %.block.3
.block.3:
	%4 = bitcast [100 x i8]* %line to i8*
	%5 = load %type.CFile*, %type.CFile** %input
	%6 = call i8*(i8*, i32, %type.CFile*) @fgets(i8* noundef %4, i32 noundef 100, %type.CFile* noundef %5)
	%7 = icmp ne i8* %6, null
	br i1 %7, label %.block.4, label %.block.5
.block.4:
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.6
.block.6:
	%8 = load i64, i64* %index
	%9 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %8
	%10 = load i8, i8* %9
	%11 = zext i8 %10 to i32
	%12 = call i32(i32) @isdigit(i32 noundef %11)
	%13 = icmp eq i32 %12, 0
	br i1 %13, label %.block.7, label %.block.8
.block.7:
	%14 = load i64, i64* %index
	%15 = add nuw i64 %14, 1
	store i64 %15, i64* %index
	br label %.block.6
.block.8:
	%calibration_value = alloca i32
	%16 = load i64, i64* %index
	%17 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %16
	%18 = load i8, i8* %17
	%19 = sub nuw i8 %18, 48
	%20 = zext i8 %19 to i32
	store i32 %20, i32* %calibration_value
	%21 = bitcast [100 x i8]* %line to i8*
	%22 = call i64(i8*) @strlen(i8* noundef %21)
	%23 = sub nuw i64 %22, 1
	store i64 %23, i64* %index
	br label %.block.9
.block.9:
	%24 = load i64, i64* %index
	%25 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %24
	%26 = load i8, i8* %25
	%27 = zext i8 %26 to i32
	%28 = call i32(i32) @isdigit(i32 noundef %27)
	%29 = icmp eq i32 %28, 0
	br i1 %29, label %.block.10, label %.block.11
.block.10:
	%30 = load i64, i64* %index
	%31 = sub nuw i64 %30, 1
	store i64 %31, i64* %index
	br label %.block.9
.block.11:
	%calibration_value-1 = alloca i32
	%32 = load i32, i32* %calibration_value
	%33 = mul nuw i32 %32, 10
	%34 = load i64, i64* %index
	%35 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %34
	%36 = load i8, i8* %35
	%37 = sub nuw i8 %36, 48
	%38 = zext i8 %37 to i32
	%39 = add nuw i32 %33, %38
	store i32 %39, i32* %calibration_value-1
	%40 = load i32, i32* %calibration_value-1
	%41 = load i32, i32* %calibration_sum
	%42 = add nuw i32 %41, %40
	store i32 %42, i32* %calibration_sum
	br label %.block.3
.block.5:
	%43 = load %type.CFile*, %type.CFile** %input
	%44 = call i32(%type.CFile*) @fclose(%type.CFile* noundef %43)
	%45 = load i32, i32* %calibration_sum
	%46 = call i32(i8*, ...) @printf(i8* noundef bitcast ([38 x i8]* @.const.3 to i8*), i32 noundef %45)
	ret void
}

@.const.0 = private unnamed_addr constant [10 x i8] c"day01.txt\00"
@.const.1 = private unnamed_addr constant [2 x i8] c"r\00"
@.const.2 = private unnamed_addr constant [27 x i8] c"unable to open input file\0A\00"
@.const.3 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

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
	%18 = call i32(i8*, ...) @printf(i8* noundef bitcast ([11 x i8]* @.const.4 to i8*))
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
	%25 = call i32(i8*, ...) @printf(i8* noundef bitcast ([4 x i8]* @.const.5 to i8*), i8* noundef %24)
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

@.const.4 = private unnamed_addr constant [11 x i8] c"Reversed:\0A\00"
@.const.5 = private unnamed_addr constant [4 x i8] c"%s\0A\00"

%type.Student = type { i8*, i32, [4 x i32] }

define dso_local void @student_stuff() #0 {
.block.0:
	%joe_age = alloca i32
	store i32 97, i32* %joe_age
	%joe_calculus_grade_before_curve = alloca i32
	store i32 47, i32* %joe_calculus_grade_before_curve
	%joe = alloca %type.Student
	%0 = load i32, i32* %joe_age
	%1 = load i32, i32* %joe_calculus_grade_before_curve
	%2 = add nuw i32 %1, 15
	%3 = alloca [4 x i32]
	store [4 x i32] [ i32 80, i32 100, i32 92, i32 undef ], [4 x i32]* %3
	%4 = getelementptr inbounds [4 x i32], [4 x i32]* %3, i32 0, i64 3
	store i32 %2, i32* %4
	%5 = load [4 x i32], [4 x i32]* %3
	%6 = alloca %type.Student
	store %type.Student { i8* bitcast ([9 x i8]* @.const.6 to i8*), i32 undef, [4 x i32] undef }, %type.Student* %6
	%7 = getelementptr inbounds %type.Student, %type.Student* %6, i32 0, i32 1
	store i32 %0, i32* %7
	%8 = getelementptr inbounds %type.Student, %type.Student* %6, i32 0, i32 2
	store [4 x i32] %5, [4 x i32]* %8
	%9 = load %type.Student, %type.Student* %6
	store %type.Student %9, %type.Student* %joe
	%10 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 0
	%11 = load i8*, i8** %10
	%12 = call i32(i8*, ...) @printf(i8* noundef bitcast ([10 x i8]* @.const.7 to i8*), i8* noundef %11)
	%13 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 1
	%14 = load i32, i32* %13
	%15 = call i32(i8*, ...) @printf(i8* noundef bitcast ([9 x i8]* @.const.8 to i8*), i32 noundef %14)
	%16 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%17 = getelementptr inbounds [4 x i32], [4 x i32]* %16, i32 0, i32 0
	%18 = load i32, i32* %17
	%19 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%20 = getelementptr inbounds [4 x i32], [4 x i32]* %19, i32 0, i32 1
	%21 = load i32, i32* %20
	%22 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%23 = getelementptr inbounds [4 x i32], [4 x i32]* %22, i32 0, i32 2
	%24 = load i32, i32* %23
	%25 = getelementptr inbounds %type.Student, %type.Student* %joe, i32 0, i32 2
	%26 = getelementptr inbounds [4 x i32], [4 x i32]* %25, i32 0, i32 3
	%27 = load i32, i32* %26
	%28 = call i32(i8*, ...) @printf(i8* noundef bitcast ([24 x i8]* @.const.9 to i8*), i32 noundef %18, i32 noundef %21, i32 noundef %24, i32 noundef %27)
	ret void
}

@.const.6 = private unnamed_addr constant [9 x i8] c"Joe Mama\00"
@.const.7 = private unnamed_addr constant [10 x i8] c"Name: %s\0A\00"
@.const.8 = private unnamed_addr constant [9 x i8] c"Age: %u\0A\00"
@.const.9 = private unnamed_addr constant [24 x i8] c"Grades: %u, %u, %u, %u\0A\00"

define dso_local i32 @main() #0 {
.block.0:
	call void() @aoc_01_p1()
	call void() @student_stuff()
	%values = alloca [4 x i8*]
	store [4 x i8*] [ i8* bitcast ([8 x i8]* @.const.10 to i8*), i8* bitcast ([8 x i8]* @.const.11 to i8*), i8* bitcast ([8 x i8]* @.const.12 to i8*), i8* bitcast ([8 x i8]* @.const.13 to i8*) ], [4 x i8*]* %values
	%0 = bitcast [4 x i8*]* %values to i8**
	call void(i8**, i64) @omg_linked_list(i8** noundef %0, i64 noundef 4)
	%string = alloca %type.String
	%1 = call %type.String() @String_new()
	store %type.String %1, %type.String* %string
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 72)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 101)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 108)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 108)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 111)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 32)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 119)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 111)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 114)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 108)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 100)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 33)
	call void(%type.String*, i8) @String_push(%type.String* noundef %string, i8 noundef 0)
	%str = alloca %type.Str
	%2 = call %type.Str(%type.String*) @String_as_str(%type.String* noundef %string)
	store %type.Str %2, %type.Str* %str
	%3 = getelementptr inbounds %type.Str, %type.Str* %str, i32 0, i32 0
	%4 = load i8*, i8** %3
	%5 = call i32(i8*, ...) @printf(i8* noundef bitcast ([4 x i8]* @.const.14 to i8*), i8* noundef %4)
	%6 = load %type.String, %type.String* %string
	call void(%type.String) @String_del(%type.String noundef %6)
	ret i32 0
}

@.const.10 = private unnamed_addr constant [8 x i8] c"Value 1\00"
@.const.11 = private unnamed_addr constant [8 x i8] c"Value 2\00"
@.const.12 = private unnamed_addr constant [8 x i8] c"Value 3\00"
@.const.13 = private unnamed_addr constant [8 x i8] c"Value 4\00"
@.const.14 = private unnamed_addr constant [4 x i8] c"%s\0A\00"

%type.CFile = type opaque

declare i32 @fclose(%type.CFile* noundef) #1

declare i32 @feof(%type.CFile* noundef) #1

declare i8* @fgets(i8* noundef, i32 noundef, %type.CFile* noundef) #1

declare %type.CFile* @fopen(i8* noundef, i8* noundef) #1

declare void @free(i8* noundef) #1

declare i32 @isdigit(i32 noundef) #1

declare i8* @malloc(i64 noundef) #1

declare i8* @memcpy(i8* noundef, i8* noundef, i64 noundef) #1

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
