source_filename = "\\\\?\\C:\\Users\\seane\\Projects\\compiler\\tests\\packages\\test_1\\main.cupr"

%"::test_1::Student" = type { i8*, i32, [4 x i32] }

%"::std::string::Str" = type { i8*, i64 }

%"::test_1::Node" = type { i8*, %"::test_1::Node"* }

%"::std::string::MutStr" = type { i8*, i64 }

%"::std::string::String" = type { %"::std::string::MutStr", i64 }

%"::libc::stdio::CFile" = type opaque

declare %"::libc::stdio::CFile"* @fopen(i8*, i8*)

declare i32 @puts(i8*)

declare i8* @fgets(i8*, i32, %"::libc::stdio::CFile"*)

declare i32 @isdigit(i32)

declare i64 @strlen(i8*)

declare i32 @fclose(%"::libc::stdio::CFile"*)

declare i32 @printf(i8*, ...)

declare i8* @malloc(i64)

declare void @free(i8*)

declare %"::std::string::String" @"::std::string::String::new"()

declare void @"::std::string::String::push"(%"::std::string::String"*, i8)

declare void @"::std::string::String::insert"(%"::std::string::String"*, i64, i8)

declare %"::std::string::Str" @"::std::string::String::as_str"(%"::std::string::String"*)

declare void @"::std::string::String::del"(%"::std::string::String")

@.const.test_1.0 = private unnamed_addr constant [17 x i8] c"test_1/day01.txt\00"

@.const.test_1.1 = private unnamed_addr constant [2 x i8] c"r\00"

@.const.test_1.2 = private unnamed_addr constant [26 x i8] c"unable to open input file\00"

@.const.test_1.3 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

@.const.test_1.4 = private unnamed_addr constant [10 x i8] c"Reversed:\00"

@.const.test_1.5 = private unnamed_addr constant [4 x i8] c"%s\0A\00"

@.const.test_1.6 = private unnamed_addr constant [9 x i8] c"Joe Mama\00"

@.const.test_1.7 = private unnamed_addr constant [10 x i8] c"Name: %s\0A\00"

@.const.test_1.8 = private unnamed_addr constant [9 x i8] c"Age: %u\0A\00"

@.const.test_1.9 = private unnamed_addr constant [24 x i8] c"Grades: %u, %u, %u, %u\0A\00"

@"::test_1::static_mut_var" = global i32 5

@.const.test_1.10 = private unnamed_addr constant [14 x i8] c"I am a string\00"

@"::test_1::static_var" = constant i8* bitcast ([14 x i8]* @.const.test_1.10 to i8*)

@.const.test_1.11 = private unnamed_addr constant [8 x i8] c"Value 1\00"

@.const.test_1.12 = private unnamed_addr constant [8 x i8] c"Value 2\00"

@.const.test_1.13 = private unnamed_addr constant [8 x i8] c"Value 3\00"

@.const.test_1.14 = private unnamed_addr constant [8 x i8] c"Value 4\00"

@.const.test_1.15 = private unnamed_addr constant [22 x i8] c"i64::to_string: \22%s\22\0A\00"

define i32 @"::test_1::fibonacci"(i32 %0) {
.block.0:
	%limit = alloca i32
	store i32 %0, i32* %limit
	%a = alloca i32
	store i32 0, i32* %a
	%b = alloca i32
	store i32 1, i32* %b
	br label %.block.1
.block.1:
	%1 = load i32, i32* %b
	%2 = load i32, i32* %limit
	%3 = icmp slt i32 %1, %2
	br i1 %3, label %.block.2, label %.block.3
.block.2:
	%4 = load i32, i32* %a
	%5 = load i32, i32* %b
	%6 = add nsw i32 %4, %5
	%temp = alloca i32
	store i32 %6, i32* %temp
	%7 = load i32, i32* %b
	store i32 %7, i32* %a
	%8 = load i32, i32* %temp
	store i32 %8, i32* %b
	br label %.block.1
.block.3:
	%9 = load i32, i32* %a
	ret i32 %9
}

define i32 @"::test_1::gcd"(i32 %0, i32 %1) {
.block.0:
	%a = alloca i32
	store i32 %0, i32* %a
	%b = alloca i32
	store i32 %1, i32* %b
	br label %.block.1
.block.1:
	%2 = load i32, i32* %b
	%3 = icmp uge i32 %2, 1
	br i1 %3, label %.block.2, label %.block.3
.block.2:
	%4 = load i32, i32* %a
	%5 = load i32, i32* %b
	%6 = urem i32 %4, %5
	%temp = alloca i32
	store i32 %6, i32* %temp
	%7 = load i32, i32* %b
	store i32 %7, i32* %a
	%8 = load i32, i32* %temp
	store i32 %8, i32* %b
	br label %.block.1
.block.3:
	%9 = load i32, i32* %a
	ret i32 %9
}

define void @"::test_1::aoc_01_p1"() {
.block.0:
	%0 = call %"::libc::stdio::CFile"*(i8*, i8*) @fopen(i8* bitcast ([17 x i8]* @.const.test_1.0 to i8*), i8* bitcast ([2 x i8]* @.const.test_1.1 to i8*))
	%input = alloca %"::libc::stdio::CFile"*
	store %"::libc::stdio::CFile"* %0, %"::libc::stdio::CFile"** %input
	%1 = load %"::libc::stdio::CFile"*, %"::libc::stdio::CFile"** %input
	%2 = icmp eq %"::libc::stdio::CFile"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = call i32(i8*) @puts(i8* bitcast ([26 x i8]* @.const.test_1.2 to i8*))
	ret void
.block.2:
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%line = alloca [100 x i8]
	br label %.block.3
.block.3:
	%4 = bitcast [100 x i8]* %line to i8*
	%5 = load %"::libc::stdio::CFile"*, %"::libc::stdio::CFile"** %input
	%6 = call i8*(i8*, i32, %"::libc::stdio::CFile"*) @fgets(i8* %4, i32 100, %"::libc::stdio::CFile"* %5)
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
	%12 = call i32(i32) @isdigit(i32 %11)
	%13 = icmp eq i32 %12, 0
	br i1 %13, label %.block.7, label %.block.8
.block.7:
	%14 = load i64, i64* %index
	%15 = add nuw i64 %14, 1
	store i64 %15, i64* %index
	br label %.block.6
.block.8:
	%16 = load i64, i64* %index
	%17 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %16
	%18 = load i8, i8* %17
	%19 = sub nuw i8 %18, 48
	%20 = zext i8 %19 to i32
	%calibration_value = alloca i32
	store i32 %20, i32* %calibration_value
	%21 = bitcast [100 x i8]* %line to i8*
	%22 = call i64(i8*) @strlen(i8* %21)
	%23 = sub nuw i64 %22, 1
	store i64 %23, i64* %index
	br label %.block.9
.block.9:
	%24 = load i64, i64* %index
	%25 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %24
	%26 = load i8, i8* %25
	%27 = zext i8 %26 to i32
	%28 = call i32(i32) @isdigit(i32 %27)
	%29 = icmp eq i32 %28, 0
	br i1 %29, label %.block.10, label %.block.11
.block.10:
	%30 = load i64, i64* %index
	%31 = sub nuw i64 %30, 1
	store i64 %31, i64* %index
	br label %.block.9
.block.11:
	%32 = load i32, i32* %calibration_value
	%33 = mul nuw i32 %32, 10
	%34 = load i64, i64* %index
	%35 = getelementptr inbounds [100 x i8], [100 x i8]* %line, i32 0, i64 %34
	%36 = load i8, i8* %35
	%37 = sub nuw i8 %36, 48
	%38 = zext i8 %37 to i32
	%39 = add nuw i32 %33, %38
	%calibration_value-1 = alloca i32
	store i32 %39, i32* %calibration_value-1
	%40 = load i32, i32* %calibration_value-1
	%41 = load i32, i32* %calibration_sum
	%42 = add nuw i32 %41, %40
	store i32 %42, i32* %calibration_sum
	br label %.block.3
.block.5:
	%43 = load %"::libc::stdio::CFile"*, %"::libc::stdio::CFile"** %input
	%44 = call i32(%"::libc::stdio::CFile"*) @fclose(%"::libc::stdio::CFile"* %43)
	%45 = load i32, i32* %calibration_sum
	%46 = call i32(i8*, ...) @printf(i8* bitcast ([38 x i8]* @.const.test_1.3 to i8*), i32 %45)
	ret void
}

define void @"::test_1::omg_linked_list"(i8** %0, i64 %1) {
.block.0:
	%values = alloca i8**
	store i8** %0, i8*** %values
	%value_count = alloca i64
	store i64 %1, i64* %value_count
	%head = alloca %"::test_1::Node"*
	store %"::test_1::Node"* null, %"::test_1::Node"** %head
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.1
.block.1:
	%2 = load i64, i64* %index
	%3 = load i64, i64* %value_count
	%4 = icmp ult i64 %2, %3
	br i1 %4, label %.block.2, label %.block.3
.block.2:
	%5 = call i8*(i64) @malloc(i64 16)
	%6 = bitcast i8* %5 to %"::test_1::Node"*
	%node = alloca %"::test_1::Node"*
	store %"::test_1::Node"* %6, %"::test_1::Node"** %node
	%7 = load %"::test_1::Node"*, %"::test_1::Node"** %node
	%8 = load i64, i64* %index
	%9 = load i8**, i8*** %values
	%10 = getelementptr inbounds i8*, i8** %9, i64 %8
	%11 = load i8*, i8** %10
	%12 = load %"::test_1::Node"*, %"::test_1::Node"** %head
	%13 = alloca %"::test_1::Node"
	%14 = getelementptr inbounds %"::test_1::Node", %"::test_1::Node"* %13, i32 0, i32 0
	store i8* %11, i8** %14
	%15 = getelementptr inbounds %"::test_1::Node", %"::test_1::Node"* %13, i32 0, i32 1
	store %"::test_1::Node"* %12, %"::test_1::Node"** %15
	%16 = load %"::test_1::Node", %"::test_1::Node"* %13
	store %"::test_1::Node" %16, %"::test_1::Node"* %7
	%17 = load %"::test_1::Node"*, %"::test_1::Node"** %node
	store %"::test_1::Node"* %17, %"::test_1::Node"** %head
	%18 = load i64, i64* %index
	%19 = add nuw i64 %18, 1
	store i64 %19, i64* %index
	br label %.block.1
.block.3:
	%20 = call i32(i8*) @puts(i8* bitcast ([10 x i8]* @.const.test_1.4 to i8*))
	br label %.block.4
.block.4:
	%21 = load %"::test_1::Node"*, %"::test_1::Node"** %head
	%22 = icmp ne %"::test_1::Node"* %21, null
	br i1 %22, label %.block.5, label %.block.6
.block.5:
	%23 = load %"::test_1::Node"*, %"::test_1::Node"** %head
	%node-1 = alloca %"::test_1::Node"*
	store %"::test_1::Node"* %23, %"::test_1::Node"** %node-1
	%24 = load %"::test_1::Node"*, %"::test_1::Node"** %node-1
	%25 = getelementptr inbounds %"::test_1::Node", %"::test_1::Node"* %24, i32 0, i32 0
	%26 = load i8*, i8** %25
	%27 = call i32(i8*, ...) @printf(i8* bitcast ([4 x i8]* @.const.test_1.5 to i8*), i8* %26)
	%28 = load %"::test_1::Node"*, %"::test_1::Node"** %node-1
	%29 = getelementptr inbounds %"::test_1::Node", %"::test_1::Node"* %28, i32 0, i32 1
	%30 = load %"::test_1::Node"*, %"::test_1::Node"** %29
	store %"::test_1::Node"* %30, %"::test_1::Node"** %head
	%31 = load %"::test_1::Node"*, %"::test_1::Node"** %node-1
	%32 = bitcast %"::test_1::Node"* %31 to i8*
	call void(i8*) @free(i8* %32)
	br label %.block.4
.block.6:
	ret void
}

define void @"::test_1::student_stuff"() {
.block.0:
	%joe_age = alloca i32
	store i32 97, i32* %joe_age
	%joe_calculus_grade_before_curve = alloca i32
	store i32 47, i32* %joe_calculus_grade_before_curve
	%0 = load i32, i32* %joe_age
	%1 = load i32, i32* %joe_calculus_grade_before_curve
	%2 = add nuw i32 %1, 15
	%3 = alloca [4 x i32]
	store [4 x i32] [ i32 80, i32 100, i32 92, i32 undef ], [4 x i32]* %3
	%4 = getelementptr inbounds [4 x i32], [4 x i32]* %3, i32 0, i64 3
	store i32 %2, i32* %4
	%5 = load [4 x i32], [4 x i32]* %3
	%6 = alloca %"::test_1::Student"
	store %"::test_1::Student" { i8* bitcast ([9 x i8]* @.const.test_1.6 to i8*), i32 undef, [4 x i32] undef }, %"::test_1::Student"* %6
	%7 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %6, i32 0, i32 1
	store i32 %0, i32* %7
	%8 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %6, i32 0, i32 2
	store [4 x i32] %5, [4 x i32]* %8
	%9 = load %"::test_1::Student", %"::test_1::Student"* %6
	%joe = alloca %"::test_1::Student"
	store %"::test_1::Student" %9, %"::test_1::Student"* %joe
	%10 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 0
	%11 = load i8*, i8** %10
	%12 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.test_1.7 to i8*), i8* %11)
	%13 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 1
	%14 = load i32, i32* %13
	%15 = call i32(i8*, ...) @printf(i8* bitcast ([9 x i8]* @.const.test_1.8 to i8*), i32 %14)
	%16 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 2
	%17 = getelementptr inbounds [4 x i32], [4 x i32]* %16, i32 0, i32 0
	%18 = load i32, i32* %17
	%19 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 2
	%20 = getelementptr inbounds [4 x i32], [4 x i32]* %19, i32 0, i32 1
	%21 = load i32, i32* %20
	%22 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 2
	%23 = getelementptr inbounds [4 x i32], [4 x i32]* %22, i32 0, i32 2
	%24 = load i32, i32* %23
	%25 = getelementptr inbounds %"::test_1::Student", %"::test_1::Student"* %joe, i32 0, i32 2
	%26 = getelementptr inbounds [4 x i32], [4 x i32]* %25, i32 0, i32 3
	%27 = load i32, i32* %26
	%28 = call i32(i8*, ...) @printf(i8* bitcast ([24 x i8]* @.const.test_1.9 to i8*), i32 %18, i32 %21, i32 %24, i32 %27)
	ret void
}

define %"::std::string::String" @"<i64>::to_string"(i64 %0) {
.block.0:
	%self = alloca i64
	store i64 %0, i64* %self
	%1 = call %"::std::string::String"() @"::std::string::String::new"()
	%string = alloca %"::std::string::String"
	store %"::std::string::String" %1, %"::std::string::String"* %string
	%2 = load i64, i64* %self
	%3 = icmp eq i64 %2, 0
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 48)
	br label %.block.3
.block.2:
	%4 = load i64, i64* %self
	%5 = icmp slt i64 %4, 0
	%is_negative = alloca i1
	store i1 %5, i1* %is_negative
	%6 = load i1, i1* %is_negative
	br i1 %6, label %.block.4, label %.block.5
.block.4:
	%7 = load i64, i64* %self
	%8 = sub nsw i64 0, %7
	store i64 %8, i64* %self
	br label %.block.5
.block.5:
	br label %.block.6
.block.6:
	%9 = load i64, i64* %self
	%10 = icmp ne i64 %9, 0
	br i1 %10, label %.block.7, label %.block.8
.block.7:
	%11 = load i64, i64* %self
	%12 = srem i64 %11, 10
	%13 = trunc i64 %12 to i8
	%14 = add nuw i8 %13, 48
	call void(%"::std::string::String"*, i64, i8) @"::std::string::String::insert"(%"::std::string::String"* %string, i64 0, i8 %14)
	%15 = load i64, i64* %self
	%16 = sdiv i64 %15, 10
	store i64 %16, i64* %self
	br label %.block.6
.block.8:
	%17 = load i1, i1* %is_negative
	br i1 %17, label %.block.9, label %.block.10
.block.9:
	call void(%"::std::string::String"*, i64, i8) @"::std::string::String::insert"(%"::std::string::String"* %string, i64 0, i8 45)
	br label %.block.10
.block.10:
	br label %.block.3
.block.3:
	%18 = load %"::std::string::String", %"::std::string::String"* %string
	ret %"::std::string::String" %18
}

define i32 @main() {
.block.0:
	call void() @"::test_1::aoc_01_p1"()
	call void() @"::test_1::student_stuff"()
	%values = alloca [4 x i8*]
	store [4 x i8*] [ i8* bitcast ([8 x i8]* @.const.test_1.11 to i8*), i8* bitcast ([8 x i8]* @.const.test_1.12 to i8*), i8* bitcast ([8 x i8]* @.const.test_1.13 to i8*), i8* bitcast ([8 x i8]* @.const.test_1.14 to i8*) ], [4 x i8*]* %values
	%0 = bitcast [4 x i8*]* %values to i8**
	call void(i8**, i64) @"::test_1::omg_linked_list"(i8** %0, i64 4)
	%1 = call %"::std::string::String"() @"::std::string::String::new"()
	%string = alloca %"::std::string::String"
	store %"::std::string::String" %1, %"::std::string::String"* %string
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 72)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 101)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 108)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 108)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 111)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 32)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 119)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 111)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 114)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 108)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 100)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 33)
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %string, i8 0)
	%2 = call %"::std::string::Str"(%"::std::string::String"*) @"::std::string::String::as_str"(%"::std::string::String"* %string)
	%str = alloca %"::std::string::Str"
	store %"::std::string::Str" %2, %"::std::string::Str"* %str
	%3 = getelementptr inbounds %"::std::string::Str", %"::std::string::Str"* %str, i32 0, i32 0
	%4 = load i8*, i8** %3
	%5 = call i32(i8*) @puts(i8* %4)
	%6 = load %"::std::string::String", %"::std::string::String"* %string
	call void(%"::std::string::String") @"::std::string::String::del"(%"::std::string::String" %6)
	%7 = sub nsw i64 0, 12345
	%8 = call %"::std::string::String"(i64) @"<i64>::to_string"(i64 %7)
	%number_string = alloca %"::std::string::String"
	store %"::std::string::String" %8, %"::std::string::String"* %number_string
	call void(%"::std::string::String"*, i8) @"::std::string::String::push"(%"::std::string::String"* %number_string, i8 0)
	%9 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %number_string, i32 0, i32 0
	%10 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %9, i32 0, i32 0
	%11 = load i8*, i8** %10
	%12 = call i32(i8*, ...) @printf(i8* bitcast ([22 x i8]* @.const.test_1.15 to i8*), i8* %11)
	%13 = load %"::std::string::String", %"::std::string::String"* %number_string
	call void(%"::std::string::String") @"::std::string::String::del"(%"::std::string::String" %13)
	ret i32 0
}

