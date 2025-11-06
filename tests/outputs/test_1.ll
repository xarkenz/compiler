source_filename = "tests/sources/test_1.main.cupr"

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
	call void(i8*) @free(i8* %3)
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

define %"type.::MutStr" @"::String::as_mut_str"(%"type.::String"* %0) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%1 = load %"type.::String"*, %"type.::String"** %self
	%2 = getelementptr inbounds %"type.::String", %"type.::String"* %1, i32 0, i32 0
	%3 = load %"type.::MutStr", %"type.::MutStr"* %2
	ret %"type.::MutStr" %3
}

define i64 @"::String::capacity"(%"type.::String"* %0) {
.block.0:
	%self = alloca %"type.::String"*
	store %"type.::String"* %0, %"type.::String"** %self
	%1 = load %"type.::String"*, %"type.::String"** %self
	%2 = getelementptr inbounds %"type.::String", %"type.::String"* %1, i32 0, i32 1
	%3 = load i64, i64* %2
	ret i64 %3
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
	%15 = call i8*(i64) @malloc(i64 %14)
	%ptr = alloca i8*
	store i8* %15, i8** %ptr
	%16 = load i8*, i8** %ptr
	%17 = load %"type.::String"*, %"type.::String"** %self
	%18 = getelementptr inbounds %"type.::String", %"type.::String"* %17, i32 0, i32 0
	%19 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %18, i32 0, i32 0
	%20 = load i8*, i8** %19
	%21 = load %"type.::String"*, %"type.::String"** %self
	%22 = getelementptr inbounds %"type.::String", %"type.::String"* %21, i32 0, i32 0
	%23 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %22, i32 0, i32 1
	%24 = load i64, i64* %23
	%25 = call i8*(i8*, i8*, i64) @memcpy(i8* %16, i8* %20, i64 %24)
	%26 = load %"type.::String"*, %"type.::String"** %self
	%27 = getelementptr inbounds %"type.::String", %"type.::String"* %26, i32 0, i32 0
	%28 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %27, i32 0, i32 0
	%29 = load i8*, i8** %28
	call void(i8*) @free(i8* %29)
	%30 = load %"type.::String"*, %"type.::String"** %self
	%31 = getelementptr inbounds %"type.::String", %"type.::String"* %30, i32 0, i32 0
	%32 = getelementptr inbounds %"type.::MutStr", %"type.::MutStr"* %31, i32 0, i32 0
	%33 = load i8*, i8** %ptr
	store i8* %33, i8** %32
	%34 = load %"type.::String"*, %"type.::String"** %self
	%35 = getelementptr inbounds %"type.::String", %"type.::String"* %34, i32 0, i32 1
	%36 = load i64, i64* %capacity
	store i64 %36, i64* %35
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
	%0 = call %"type.::libc::stdio::CFile"*(i8*, i8*) @fopen(i8* bitcast ([10 x i8]* @.const.0 to i8*), i8* bitcast ([2 x i8]* @.const.1 to i8*))
	%input = alloca %"type.::libc::stdio::CFile"*
	store %"type.::libc::stdio::CFile"* %0, %"type.::libc::stdio::CFile"** %input
	%1 = load %"type.::libc::stdio::CFile"*, %"type.::libc::stdio::CFile"** %input
	%2 = icmp eq %"type.::libc::stdio::CFile"* %1, null
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
	%5 = load %"type.::libc::stdio::CFile"*, %"type.::libc::stdio::CFile"** %input
	%6 = call i8*(i8*, i32, %"type.::libc::stdio::CFile"*) @fgets(i8* %4, i32 100, %"type.::libc::stdio::CFile"* %5)
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
	%43 = load %"type.::libc::stdio::CFile"*, %"type.::libc::stdio::CFile"** %input
	%44 = call i32(%"type.::libc::stdio::CFile"*) @fclose(%"type.::libc::stdio::CFile"* %43)
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
	%5 = call i8*(i64) @malloc(i64 16)
	%6 = bitcast i8* %5 to %"type.::Node"*
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
	%32 = bitcast %"type.::Node"* %31 to i8*
	%33 = load %"type.::Node"*, %"type.::Node"** %head
