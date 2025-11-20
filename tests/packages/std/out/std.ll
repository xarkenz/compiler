source_filename = "\\\\?\\C:\\Users\\seane\\Projects\\compiler\\tests\\packages\\std\\main.cupr"

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

%"::std::string::Str" = type { i8*, i64 }

define { i8*, i64 } @"::std::string::Str::raw_parts"(%"::std::string::Str"* %0) {
.block.0:
	%self = alloca %"::std::string::Str"*
	store %"::std::string::Str"* %0, %"::std::string::Str"** %self
	%1 = load %"::std::string::Str"*, %"::std::string::Str"** %self
	%2 = getelementptr inbounds %"::std::string::Str", %"::std::string::Str"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = load %"::std::string::Str"*, %"::std::string::Str"** %self
	%5 = getelementptr inbounds %"::std::string::Str", %"::std::string::Str"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = alloca { i8*, i64 }
	%8 = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %7, i32 0, i32 0
	store i8* %3, i8** %8
	%9 = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %7, i32 0, i32 1
	store i64 %6, i64* %9
	%10 = load { i8*, i64 }, { i8*, i64 }* %7
	ret { i8*, i64 } %10
}

%"::std::string::MutStr" = type { i8*, i64 }

define %"::std::string::Str" @"::std::string::MutStr::as_str"(%"::std::string::MutStr"* %0) {
.block.0:
	%self = alloca %"::std::string::MutStr"*
	store %"::std::string::MutStr"* %0, %"::std::string::MutStr"** %self
	%1 = load %"::std::string::MutStr"*, %"::std::string::MutStr"** %self
	%2 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	%4 = load %"::std::string::MutStr"*, %"::std::string::MutStr"** %self
	%5 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = alloca %"::std::string::Str"
	%8 = getelementptr inbounds %"::std::string::Str", %"::std::string::Str"* %7, i32 0, i32 0
	store i8* %3, i8** %8
	%9 = getelementptr inbounds %"::std::string::Str", %"::std::string::Str"* %7, i32 0, i32 1
	store i64 %6, i64* %9
	%10 = load %"::std::string::Str", %"::std::string::Str"* %7
	ret %"::std::string::Str" %10
}

%"::std::string::String" = type { %"::std::string::MutStr", i64 }

define %"::std::string::String" @"::std::string::String::new"() {
.block.0:
	ret %"::std::string::String" { %"::std::string::MutStr" { i8* null, i64 0 }, i64 0 }
}

define void @"::std::string::String::del"(%"::std::string::String" %0) {
.block.0:
	%self = alloca %"::std::string::String"
	store %"::std::string::String" %0, %"::std::string::String"* %self
	%1 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %self, i32 0, i32 0
	%2 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %1, i32 0, i32 0
	%3 = load i8*, i8** %2
	call void(i8*) @free(i8* %3)
	ret void
}

define %"::std::string::Str" @"::std::string::String::as_str"(%"::std::string::String"* %0) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%1 = load %"::std::string::String"*, %"::std::string::String"** %self
	%2 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %1, i32 0, i32 0
	%3 = call %"::std::string::Str"(%"::std::string::MutStr"*) @"::std::string::MutStr::as_str"(%"::std::string::MutStr"* %2)
	ret %"::std::string::Str" %3
}

define %"::std::string::MutStr" @"::std::string::String::as_mut_str"(%"::std::string::String"* %0) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%1 = load %"::std::string::String"*, %"::std::string::String"** %self
	%2 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %1, i32 0, i32 0
	%3 = load %"::std::string::MutStr", %"::std::string::MutStr"* %2
	ret %"::std::string::MutStr" %3
}

define i64 @"::std::string::String::capacity"(%"::std::string::String"* %0) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%1 = load %"::std::string::String"*, %"::std::string::String"** %self
	%2 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %1, i32 0, i32 1
	%3 = load i64, i64* %2
	ret i64 %3
}

define void @"::std::string::String::grow_by"(%"::std::string::String"* %0, i64 %1) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%additional = alloca i64
	store i64 %1, i64* %additional
	%2 = load %"::std::string::String"*, %"::std::string::String"** %self
	%3 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %2, i32 0, i32 1
	%4 = load i64, i64* %3
	%5 = load i64, i64* %additional
	%6 = add nuw i64 %4, %5
	%required_capacity = alloca i64
	store i64 %6, i64* %required_capacity
	%7 = load %"::std::string::String"*, %"::std::string::String"** %self
	%8 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %7, i32 0, i32 1
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
	%17 = load %"::std::string::String"*, %"::std::string::String"** %self
	%18 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %17, i32 0, i32 0
	%19 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %18, i32 0, i32 0
	%20 = load i8*, i8** %19
	%21 = load %"::std::string::String"*, %"::std::string::String"** %self
	%22 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %21, i32 0, i32 0
	%23 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %22, i32 0, i32 1
	%24 = load i64, i64* %23
	%25 = call i8*(i8*, i8*, i64) @memcpy(i8* %16, i8* %20, i64 %24)
	%26 = load %"::std::string::String"*, %"::std::string::String"** %self
	%27 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %26, i32 0, i32 0
	%28 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %27, i32 0, i32 0
	%29 = load i8*, i8** %28
	call void(i8*) @free(i8* %29)
	%30 = load %"::std::string::String"*, %"::std::string::String"** %self
	%31 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %30, i32 0, i32 0
	%32 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %31, i32 0, i32 0
	%33 = load i8*, i8** %ptr
	store i8* %33, i8** %32
	%34 = load %"::std::string::String"*, %"::std::string::String"** %self
	%35 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %34, i32 0, i32 1
	%36 = load i64, i64* %capacity
	store i64 %36, i64* %35
	ret void
}

define void @"::std::string::String::push"(%"::std::string::String"* %0, i8 %1) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%ch = alloca i8
	store i8 %1, i8* %ch
	%2 = load %"::std::string::String"*, %"::std::string::String"** %self
	%3 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %2, i32 0, i32 0
	%4 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %3, i32 0, i32 1
	%5 = load i64, i64* %4
	%6 = load %"::std::string::String"*, %"::std::string::String"** %self
	%7 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %6, i32 0, i32 1
	%8 = load i64, i64* %7
	%9 = icmp eq i64 %5, %8
	br i1 %9, label %.block.1, label %.block.2
.block.1:
	%10 = load %"::std::string::String"*, %"::std::string::String"** %self
	call void(%"::std::string::String"*, i64) @"::std::string::String::grow_by"(%"::std::string::String"* %10, i64 1)
	br label %.block.2
.block.2:
	%11 = load %"::std::string::String"*, %"::std::string::String"** %self
	%12 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %11, i32 0, i32 0
	%13 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %12, i32 0, i32 0
	%14 = load %"::std::string::String"*, %"::std::string::String"** %self
	%15 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %14, i32 0, i32 0
	%16 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %15, i32 0, i32 1
	%17 = load i64, i64* %16
	%18 = load i8*, i8** %13
	%19 = getelementptr inbounds i8, i8* %18, i64 %17
	%20 = load i8, i8* %ch
	store i8 %20, i8* %19
	%21 = load %"::std::string::String"*, %"::std::string::String"** %self
	%22 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %21, i32 0, i32 0
	%23 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %22, i32 0, i32 1
	%24 = load i64, i64* %23
	%25 = add nuw i64 %24, 1
	store i64 %25, i64* %23
	ret void
}

define void @"::std::string::String::insert"(%"::std::string::String"* %0, i64 %1, i8 %2) {
.block.0:
	%self = alloca %"::std::string::String"*
	store %"::std::string::String"* %0, %"::std::string::String"** %self
	%index = alloca i64
	store i64 %1, i64* %index
	%ch = alloca i8
	store i8 %2, i8* %ch
	%3 = load %"::std::string::String"*, %"::std::string::String"** %self
	%4 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %3, i32 0, i32 0
	%5 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %4, i32 0, i32 1
	%6 = load i64, i64* %5
	%7 = load %"::std::string::String"*, %"::std::string::String"** %self
	%8 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %7, i32 0, i32 1
	%9 = load i64, i64* %8
	%10 = icmp eq i64 %6, %9
	br i1 %10, label %.block.1, label %.block.2
.block.1:
	%11 = load %"::std::string::String"*, %"::std::string::String"** %self
	call void(%"::std::string::String"*, i64) @"::std::string::String::grow_by"(%"::std::string::String"* %11, i64 1)
	br label %.block.2
.block.2:
	br label %.block.3
.block.3:
	%12 = load i64, i64* %index
	%13 = load %"::std::string::String"*, %"::std::string::String"** %self
	%14 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %13, i32 0, i32 0
	%15 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %14, i32 0, i32 1
	%16 = load i64, i64* %15
	%17 = icmp ult i64 %12, %16
	br i1 %17, label %.block.4, label %.block.5
.block.4:
	%18 = load %"::std::string::String"*, %"::std::string::String"** %self
	%19 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %18, i32 0, i32 0
	%20 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %19, i32 0, i32 0
	%21 = load i64, i64* %index
	%22 = load i8*, i8** %20
	%23 = getelementptr inbounds i8, i8* %22, i64 %21
	call void(i8*, i8*) @"<u8>::swap"(i8* %23, i8* %ch)
	%24 = load i64, i64* %index
	%25 = add nuw i64 %24, 1
	store i64 %25, i64* %index
	br label %.block.3
.block.5:
	%26 = load %"::std::string::String"*, %"::std::string::String"** %self
	%27 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %26, i32 0, i32 0
	%28 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %27, i32 0, i32 0
	%29 = load %"::std::string::String"*, %"::std::string::String"** %self
	%30 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %29, i32 0, i32 0
	%31 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %30, i32 0, i32 1
	%32 = load i64, i64* %31
	%33 = load i8*, i8** %28
	%34 = getelementptr inbounds i8, i8* %33, i64 %32
	%35 = load i8, i8* %ch
	store i8 %35, i8* %34
	%36 = load %"::std::string::String"*, %"::std::string::String"** %self
	%37 = getelementptr inbounds %"::std::string::String", %"::std::string::String"* %36, i32 0, i32 0
	%38 = getelementptr inbounds %"::std::string::MutStr", %"::std::string::MutStr"* %37, i32 0, i32 1
	%39 = load i64, i64* %38
	%40 = add nuw i64 %39, 1
	store i64 %40, i64* %38
	ret void
}

; ==== External definitions from other packages ====

declare void @free(i8*)

declare i8* @malloc(i64)

declare i8* @memcpy(i8*, i8*, i64)

