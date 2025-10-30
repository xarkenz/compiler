; file_id = 0
source_filename = "tests/sources/test_1.cupr"

%"type.::CFile" = type opaque

declare %"type.::CFile"* @fopen(i8*, i8*)

declare i32 @fclose(%"type.::CFile"*)

declare i32 @feof(%"type.::CFile"*)

declare i8* @fgets(i8*, i32, %"type.::CFile"*)

declare i32 @printf(i8*, ...)

declare {}* @malloc(i64)

declare void @free({}*)

declare {}* @memcpy({}*, {}*, i64)

declare i64 @strlen(i8*)

declare i32 @isdigit(i32)

define void @"<u8>::swap"(i8* %0, i8* %1) {
.block.0:
	%self = alloca i8*
	store i8* %0, i8** %self
	%other = alloca i8*
	store i8* %1, i8** %other
	%2 = load i8*, i8** %self
	%3 = load i8, i8* %2
	%temp = alloca i8
	store i8 %3, i8* %temp
	%4 = load i8*, i8** %self
	%5 = load i8*, i8** %other
	%6 = load i8, i8* %5
	store i8 %6, i8* %4
	%7 = load i8*, i8** %other
	%8 = load i8, i8* %temp
	store i8 %8, i8* %7
	ret void
}

define i64 @"<usize>::max"(i64 %0, i64 %1) {
.block.0:
	%self = alloca i64
	store i64 %0, i64* %self
	%other = alloca i64
	store i64 %1, i64* %other
	%2 = load i64, i64* %self
	%3 = load i64, i64* %other
	%4 = icmp ugt i64 %2, %3
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	%5 = load i64, i64* %self
	br label %.block.3
.block.2:
	%6 = load i64, i64* %other
	br label %.block.3
.block.3:
	%7 = phi i64 [ %5, %.block.1 ], [ %6, %.block.2 ]
	ret i64 %7
}

%"type.::Str" = type { i8*, i64 }

define { i8*, i64 } @"::Str::raw_parts"(%"type.::Str"* %0) {
.block.0:
	%self = alloca %"type.::Str"*
	store %"type.::Str"* %0, %"type.::Str"** %self
	%1 = load %"type.::Str"*, %"type.::Str"** %self
	%2 = getelementptr inbounds %"type.::Str", %"type.::Str"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = load %"type.::Str"*, %"type.::Str"** %self
	%5 = getelementptr inbounds %"type.::Str", %"type.::Str"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = alloca { i8*, i64 }
	%8 = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %7, i32 0, i32 0
	store i8* %3, i8** %8
	%9 = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %7, i32 0, i32 1
	store i64 %6, i64* %9
	%10 = load { i8*, i64 }, { i8*, i64 }* %7
	ret { i8*, i64 } %10
}

%"type.::MutStr" = type { i8*, i64 }

define %"type.::Str" @"::MutStr::as_str"(%"type.::MutStr"* %0) {
.block.0:
	%self = alloca %"type.::MutStr"*
	store %"type.::MutStr"* %0, %"type.::MutStr"** %self
	%1 = load %"type.::MutStr"*, %"type.::MutStr"** %self
	%2 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = load %"type.::MutStr"*, %"type.::MutStr"** %self
	%5 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = alloca %"type.::Str"
	%8 = getelementptr inbounds %"type.::Str", %"type.::Str"* %7, i32 0, i32 0
	store i8* %3, i8** %8
	%9 = getelementptr inbounds %"type.::Str", %"type.::Str"* %7, i32 0, i32 1
	store i64 %6, i64* %9
	%10 = load %"type.::Str", %"type.::Str"* %7
	ret %"type.::Str" %10
}

%"type.::String" = type { %"type.::MutStr", i64 }

define %"type.::String" @"::String::new"() {
.block.0:
	ret %"type.::String" { %"type.::MutStr" { i8* null, i64 0 }, i64 0 }
}

define void @"::String::del"(%"type.::String" %0) {
.block.0:
	%self = alloca %"type.::String"
	store %"type.::String" %0, %"type.::String"* %self
	%1 = getelementptr inbounds %"type.::String", %"type.::String"* %self, i32 0, i32 0
	%2 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = bitcast i8* %3 to {}*
	call void({}*) @free({}* %4)
	ret void
}

define %"type.::Str" @"::String::as_str"(%"type.::String"* %0) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%1 = load %"type.::String"*, %"type.::String"** %self
	%2 = getelementptr inbounds %"type.::String", %"type.::String"* %1, i32 0, i32 0
	%3 = call %"type.::Str"(%"type.::MutStr"*) @"::MutStr::as_str"(%"type.::MutStr"* %2)
	ret %"type.::Str" %3
}

define void @"::String::grow_by"(%"type.::String"* %0, i64 %1) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%additional = alloca i64
	store i64 %1, i64* %additional
	%2 = load %"type.::String"*, %"type.::String"** %self
	%3 = getelementptr inbounds %"type.::String", %"type.::String"* %2, i32 0, i32 1
	%4 = load i64, i64* %3
	%5 = load i64, i64* %additional
	%6 = add nuw i64 %4, %5
	%required_capacity = alloca i64
	store i64 %6, i64* %required_capacity
	%7 = load %"type.::String"*, %"type.::String"** %self
	%8 = getelementptr inbounds %"type.::String", %"type.::String"* %7, i32 0, i32 1
	%9 = load i64, i64* %8
	%10 = mul nuw i64 %9, 2
	%11 = load i64, i64* %required_capacity
	%12 = call i64(i64, i64) @"<usize>::max"(i64 %10, i64 %11)
	%capacity = alloca i64
	store i64 %12, i64* %capacity
	%13 = load i64, i64* %capacity
	%14 = mul nuw i64 1, %13
	%15 = call {}*(i64) @malloc(i64 %14)
	%16 = bitcast {}* %15 to i8*
	%ptr = alloca i8*
	store i8* %16, i8** %ptr
	%17 = load i8*, i8** %ptr
	%18 = bitcast i8* %17 to {}*
	%19 = load %"type.::String"*, %"type.::String"** %self
	%20 = getelementptr inbounds %"type.::String", %"type.::String"* %19, i32 0, i32 0
	%21 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %20, i32 0, i32 0
	%22 = load i8*, i8** %21
	%23 = bitcast i8* %22 to {}*
	%24 = load %"type.::String"*, %"type.::String"** %self
	%25 = getelementptr inbounds %"type.::String", %"type.::String"* %24, i32 0, i32 0
	%26 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %25, i32 0, i32 1
	%27 = load i64, i64* %26
	%28 = call {}*({}*, {}*, i64) @memcpy({}* %18, {}* %23, i64 %27)
	%29 = load %"type.::String"*, %"type.::String"** %self
	%30 = getelementptr inbounds %"type.::String", %"type.::String"* %29, i32 0, i32 0
	%31 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %30, i32 0, i32 0
	%32 = load i8*, i8** %31
	%33 = bitcast i8* %32 to {}*
	call void({}*) @free({}* %33)
	%34 = load %"type.::String"*, %"type.::String"** %self
	%35 = getelementptr inbounds %"type.::String", %"type.::String"* %34, i32 0, i32 0
	%36 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %35, i32 0, i32 0
	%37 = load i8*, i8** %ptr
	store i8* %37, i8** %36
	%38 = load %"type.::String"*, %"type.::String"** %self
	%39 = getelementptr inbounds %"type.::String", %"type.::String"* %38, i32 0, i32 1
	%40 = load i64, i64* %capacity
	store i64 %40, i64* %39
	ret void
}

define void @"::String::push"(%"type.::String"* %0, i8 %1) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%ch = alloca i8
	store i8 %1, i8* %ch
	%2 = load %"type.::String"*, %"type.::String"** %self
	%3 = getelementptr inbounds %"type.::String", %"type.::String"* %2, i32 0, i32 0
	%4 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %3, i32 0, i32 1
	%5 = load i64, i64* %4
	%6 = load %"type.::String"*, %"type.::String"** %self
	%7 = getelementptr inbounds %"type.::String", %"type.::String"* %6, i32 0, i32 1
	%8 = load i64, i64* %7
	%9 = icmp eq i64 %5, %8
	br i1 %9, label %.block.1, label %.block.2
.block.1:
	%10 = load %"type.::String"*, %"type.::String"** %self
	call void(%"type.::String"*, i64) @"::String::grow_by"(%"type.::String"* %10, i64 1)
	br label %.block.2
.block.2:
	%11 = load %"type.::String"*, %"type.::String"** %self
	%12 = getelementptr inbounds %"type.::String", %"type.::String"* %11, i32 0, i32 0
	%13 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %12, i32 0, i32 0
	%14 = load %"type.::String"*, %"type.::String"** %self
	%15 = getelementptr inbounds %"type.::String", %"type.::String"* %14, i32 0, i32 0
	%16 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %15, i32 0, i32 1
	%17 = load i64, i64* %16
	%18 = load i8*, i8** %13
	%19 = getelementptr inbounds i8, i8* %18, i64 %17
	%20 = load i8, i8* %ch
	store i8 %20, i8* %19
	%21 = load %"type.::String"*, %"type.::String"** %self
	%22 = getelementptr inbounds %"type.::String", %"type.::String"* %21, i32 0, i32 0
	%23 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %22, i32 0, i32 1
	%24 = load i64, i64* %23
	%25 = add nuw i64 %24, 1
	store i64 %25, i64* %23
	ret void
}

define void @"::String::insert"(%"type.::String"* %0, i64 %1, i8 %2) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%index = alloca i64
	store i64 %1, i64* %index
	%ch = alloca i8
	store i8 %2, i8* %ch
	%3 = load %"type.::String"*, %"type.::String"** %self
	%4 = getelementptr inbounds %"type.::String", %"type.::String"* %3, i32 0, i32 0
	%5 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = load %"type.::String"*, %"type.::String"** %self
	%8 = getelementptr inbounds %"type.::String", %"type.::String"* %7, i32 0, i32 1
	%9 = load i64, i64* %8
	%10 = icmp eq i64 %6, %9
	br i1 %10, label %.block.1, label %.block.2
.block.1:
	%11 = load %"type.::String"*, %"type.::String"** %self
	call void(%"type.::String"*, i64) @"::String::grow_by"(%"type.::String"* %11, i64 1)
	br label %.block.2
.block.2:
	br label %.block.3
.block.3:
	%12 = load i64, i64* %index
	%13 = load %"type.::String"*, %"type.::String"** %self
	%14 = getelementptr inbounds %"type.::String", %"type.::String"* %13, i32 0, i32 0
	%15 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %14, i32 0, i32 1
	%16 = load i64, i64* %15
	%17 = icmp ult i64 %12, %16
	br i1 %17, label %.block.4, label %.block.5
.block.4:
	%18 = load %"type.::String"*, %"type.::String"** %self
	%19 = getelementptr inbounds %"type.::String", %"type.::String"* %18, i32 0, i32 0
	%20 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %19, i32 0, i32 0
	%21 = load i64, i64* %index
	%22 = load i8*, i8** %20
	%23 = getelementptr inbounds i8, i8* %22, i64 %21
	call void(i8*, i8*) @"<u8>::swap"(i8* %23, i8* %ch)
	%24 = load i64, i64* %index
	%25 = add nuw i64 %24, 1
	store i64 %25, i64* %index
	br label %.block.3
.block.5:
	%26 = load %"type.::String"*, %"type.::String"** %self
	%27 = getelementptr inbounds %"type.::String", %"type.::String"* %26, i32 0, i32 0
	%28 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %27, i32 0, i32 0
	%29 = load %"type.::String"*, %"type.::String"** %self
	%30 = getelementptr inbounds %"type.::String", %"type.::String"* %29, i32 0, i32 0
	%31 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %30, i32 0, i32 1
	%32 = load i64, i64* %31
	%33 = load i8*, i8** %28
	%34 = getelementptr inbounds i8, i8* %33, i64 %32
	%35 = load i8, i8* %ch
	store i8 %35, i8* %34
	%36 = load %"type.::String"*, %"type.::String"** %self
	%37 = getelementptr inbounds %"type.::String", %"type.::String"* %36, i32 0, i32 0
	%38 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %37, i32 0, i32 1
	%39 = load i64, i64* %38
	%40 = add nuw i64 %39, 1
	store i64 %40, i64* %38
	ret void
}

define i32 @"::fibonacci"(i32 %0) {
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

define i32 @"::gcd"(i32 %0, i32 %1) {
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

define void @"::aoc_01_p1"() {
.block.0:
	%0 = call %"type.::CFile"*(i8*, i8*) @fopen(i8* bitcast ([10 x i8]* @.const.0 to i8*), i8* bitcast ([2 x i8]* @.const.1 to i8*))
	%input = alloca %"type.::CFile"*
	store %"type.::CFile"* %0, %"type.::CFile"** %input
	%1 = load %"type.::CFile"*, %"type.::CFile"** %input
	%2 = icmp eq %"type.::CFile"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = call i32(i8*, ...) @printf(i8* bitcast ([27 x i8]* @.const.2 to i8*))
	ret void
.block.2:
	%calibration_sum = alloca i32
	store i32 0, i32* %calibration_sum
	%line = alloca [100 x i8]
	br label %.block.3
.block.3:
	%4 = bitcast [100 x i8]* %line to i8*
	%5 = load %"type.::CFile"*, %"type.::CFile"** %input
	%6 = call i8*(i8*, i32, %"type.::CFile"*) @fgets(i8* %4, i32 100, %"type.::CFile"* %5)
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
	%43 = load %"type.::CFile"*, %"type.::CFile"** %input
	%44 = call i32(%"type.::CFile"*) @fclose(%"type.::CFile"* %43)
	%45 = load i32, i32* %calibration_sum
	%46 = call i32(i8*, ...) @printf(i8* bitcast ([38 x i8]* @.const.3 to i8*), i32 %45)
	ret void
}

@.const.0 = private unnamed_addr constant [10 x i8] c"day01.txt\00"
@.const.1 = private unnamed_addr constant [2 x i8] c"r\00"
@.const.2 = private unnamed_addr constant [27 x i8] c"unable to open input file\0A\00"
@.const.3 = private unnamed_addr constant [38 x i8] c"[01p1] Sum of calibration values: %d\0A\00"

%"type.::Node" = type { i8*, %"type.::Node"* }

define void @"::omg_linked_list"(i8** %0, i64 %1) {
.block.0:
	%values = alloca i8**
	store i8** %0, i8*** %values
	%value_count = alloca i64
	store i64 %1, i64* %value_count
	%head = alloca %"type.::Node"*
	store %"type.::Node"* null, %"type.::Node"** %head
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.1
.block.1:
	%2 = load i64, i64* %index
	%3 = load i64, i64* %value_count
	%4 = icmp ult i64 %2, %3
	br i1 %4, label %.block.2, label %.block.3
.block.2:
	%5 = call {}*(i64) @malloc(i64 16)
	%6 = bitcast {}* %5 to %"type.::Node"*
	%node = alloca %"type.::Node"*
	store %"type.::Node"* %6, %"type.::Node"** %node
	%7 = load %"type.::Node"*, %"type.::Node"** %node
	%8 = load i64, i64* %index
	%9 = load i8**, i8*** %values
	%10 = getelementptr inbounds i8*, i8** %9, i64 %8
	%11 = load i8*, i8** %10
	%12 = load %"type.::Node"*, %"type.::Node"** %head
	%13 = alloca %"type.::Node"
	%14 = getelementptr inbounds %"type.::Node", %"type.::Node"* %13, i32 0, i32 0
	store i8* %11, i8** %14
	%15 = getelementptr inbounds %"type.::Node", %"type.::Node"* %13, i32 0, i32 1
	store %"type.::Node"* %12, %"type.::Node"** %15
	%16 = load %"type.::Node", %"type.::Node"* %13
	store %"type.::Node" %16, %"type.::Node"* %7
	%17 = load %"type.::Node"*, %"type.::Node"** %node
	store %"type.::Node"* %17, %"type.::Node"** %head
	%18 = load i64, i64* %index
	%19 = add nuw i64 %18, 1
	store i64 %19, i64* %index
	br label %.block.1
.block.3:
	%20 = call i32(i8*, ...) @printf(i8* bitcast ([11 x i8]* @.const.4 to i8*))
	br label %.block.4
.block.4:
	%21 = load %"type.::Node"*, %"type.::Node"** %head
	%22 = icmp ne %"type.::Node"* %21, null
	br i1 %22, label %.block.5, label %.block.6
.block.5:
	%23 = load %"type.::Node"*, %"type.::Node"** %head
	%node-1 = alloca %"type.::Node"*
	store %"type.::Node"* %23, %"type.::Node"** %node-1
	%24 = load %"type.::Node"*, %"type.::Node"** %node-1
	%25 = getelementptr inbounds %"type.::Node", %"type.::Node"* %24, i32 0, i32 0
	%26 = load i8*, i8** %25
	%27 = call i32(i8*, ...) @printf(i8* bitcast ([4 x i8]* @.const.5 to i8*), i8* %26)
	%28 = load %"type.::Node"*, %"type.::Node"** %node-1
	%29 = getelementptr inbounds %"type.::Node", %"type.::Node"* %28, i32 0, i32 1
	%30 = load %"type.::Node"*, %"type.::Node"** %29
	store %"type.::Node"* %30, %"type.::Node"** %head
	%31 = load %"type.::Node"*, %"type.::Node"** %node-1
	%32 = bitcast %"type.::Node"* %31 to {}*
	call void({}*) @free({}* %32)
	br label %.block.4
.block.6:
	ret void
}

@.const.4 = private unnamed_addr constant [11 x i8] c"Reversed:\0A\00"
@.const.5 = private unnamed_addr constant [4 x i8] c"%s\0A\00"

%"type.::Student" = type { i8*, i32, [4 x i32] }

define void @"::student_stuff"() {
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
	%6 = alloca %"type.::Student"
	store %"type.::Student" { i8* bitcast ([9 x i8]* @.const.6 to i8*), i32 undef, [4 x i32] undef }, %"type.::Student"* %6
	%7 = getelementptr inbounds %"type.::Student", %"type.::Student"* %6, i32 0, i32 1
	store i32 %0, i32* %7
	%8 = getelementptr inbounds %"type.::Student", %"type.::Student"* %6, i32 0, i32 2
	store [4 x i32] %5, [4 x i32]* %8
	%9 = load %"type.::Student", %"type.::Student"* %6
	%joe = alloca %"type.::Student"
	store %"type.::Student" %9, %"type.::Student"* %joe
	%10 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 0
	%11 = load i8*, i8** %10
	%12 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.7 to i8*), i8* %11)
	%13 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 1
	%14 = load i32, i32* %13
	%15 = call i32(i8*, ...) @printf(i8* bitcast ([9 x i8]* @.const.8 to i8*), i32 %14)
	%16 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 2
	%17 = getelementptr inbounds [4 x i32], [4 x i32]* %16, i32 0, i32 0
	%18 = load i32, i32* %17
	%19 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 2
	%20 = getelementptr inbounds [4 x i32], [4 x i32]* %19, i32 0, i32 1
	%21 = load i32, i32* %20
	%22 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 2
	%23 = getelementptr inbounds [4 x i32], [4 x i32]* %22, i32 0, i32 2
	%24 = load i32, i32* %23
	%25 = getelementptr inbounds %"type.::Student", %"type.::Student"* %joe, i32 0, i32 2
	%26 = getelementptr inbounds [4 x i32], [4 x i32]* %25, i32 0, i32 3
	%27 = load i32, i32* %26
	%28 = call i32(i8*, ...) @printf(i8* bitcast ([24 x i8]* @.const.9 to i8*), i32 %18, i32 %21, i32 %24, i32 %27)
	ret void
}

@.const.6 = private unnamed_addr constant [9 x i8] c"Joe Mama\00"
@.const.7 = private unnamed_addr constant [10 x i8] c"Name: %s\0A\00"
@.const.8 = private unnamed_addr constant [9 x i8] c"Age: %u\0A\00"
@.const.9 = private unnamed_addr constant [24 x i8] c"Grades: %u, %u, %u, %u\0A\00"

define %"type.::String" @"<i64>::to_string"(i64 %0) {
.block.0:
	%self = alloca i64
	store i64 %0, i64* %self
	%1 = call %"type.::String"() @"::String::new"()
	%string = alloca %"type.::String"
	store %"type.::String" %1, %"type.::String"* %string
	%2 = load i64, i64* %self
	%3 = icmp eq i64 %2, 0
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 48)
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
	call void(%"type.::String"*, i64, i8) @"::String::insert"(%"type.::String"* %string, i64 0, i8 %14)
	%15 = load i64, i64* %self
	%16 = sdiv i64 %15, 10
	store i64 %16, i64* %self
	br label %.block.6
.block.8:
	%17 = load i1, i1* %is_negative
	br i1 %17, label %.block.9, label %.block.10
.block.9:
	call void(%"type.::String"*, i64, i8) @"::String::insert"(%"type.::String"* %string, i64 0, i8 45)
	br label %.block.10
.block.10:
	br label %.block.3
.block.3:
	%18 = load %"type.::String", %"type.::String"* %string
	ret %"type.::String" %18
}

@"::global_var" = global i32 5

@.const.10 = private unnamed_addr constant [14 x i8] c"I am a string\00"
@"::global_const" = constant i8* bitcast ([14 x i8]* @.const.10 to i8*)

define i32 @main() {
.block.0:
	call void() @"::aoc_01_p1"()
	call void() @"::student_stuff"()
	%values = alloca [4 x i8*]
	store [4 x i8*] [ i8* bitcast ([8 x i8]* @.const.11 to i8*), i8* bitcast ([8 x i8]* @.const.12 to i8*), i8* bitcast ([8 x i8]* @.const.13 to i8*), i8* bitcast ([8 x i8]* @.const.14 to i8*) ], [4 x i8*]* %values
	%0 = bitcast [4 x i8*]* %values to i8**
	call void(i8**, i64) @"::omg_linked_list"(i8** %0, i64 4)
	%1 = call %"type.::String"() @"::String::new"()
	%string = alloca %"type.::String"
	store %"type.::String" %1, %"type.::String"* %string
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 72)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 101)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 108)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 108)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 111)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 32)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 119)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 111)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 114)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 108)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 100)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 33)
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %string, i8 0)
	%2 = call %"type.::Str"(%"type.::String"*) @"::String::as_str"(%"type.::String"* %string)
	%str = alloca %"type.::Str"
	store %"type.::Str" %2, %"type.::Str"* %str
	%3 = getelementptr inbounds %"type.::Str", %"type.::Str"* %str, i32 0, i32 0
	%4 = load i8*, i8** %3
	%5 = call i32(i8*, ...) @printf(i8* bitcast ([4 x i8]* @.const.15 to i8*), i8* %4)
	%6 = load %"type.::String", %"type.::String"* %string
	call void(%"type.::String") @"::String::del"(%"type.::String" %6)
	%7 = sub nsw i64 0, 12345
	%8 = call %"type.::String"(i64) @"<i64>::to_string"(i64 %7)
	%number_string = alloca %"type.::String"
	store %"type.::String" %8, %"type.::String"* %number_string
	call void(%"type.::String"*, i8) @"::String::push"(%"type.::String"* %number_string, i8 0)
	%9 = getelementptr inbounds %"type.::String", %"type.::String"* %number_string, i32 0, i32 0
	%10 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %9, i32 0, i32 0
	%11 = load i8*, i8** %10
	%12 = call i32(i8*, ...) @printf(i8* bitcast ([22 x i8]* @.const.16 to i8*), i8* %11)
	%13 = load %"type.::String", %"type.::String"* %number_string
	call void(%"type.::String") @"::String::del"(%"type.::String" %13)
	ret i32 0
}

@.const.11 = private unnamed_addr constant [8 x i8] c"Value 1\00"
@.const.12 = private unnamed_addr constant [8 x i8] c"Value 2\00"
@.const.13 = private unnamed_addr constant [8 x i8] c"Value 3\00"
@.const.14 = private unnamed_addr constant [8 x i8] c"Value 4\00"
@.const.15 = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.const.16 = private unnamed_addr constant [22 x i8] c"i64::to_string: \22%s\22\0A\00"

