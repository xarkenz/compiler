source_filename = "tests/sources/test_2.main.cupr"

define i32 @main() {
.block.0:
	%vector = alloca [2 x i32]
	store [2 x i32] [ i32 4, i32 8 ], [2 x i32]* %vector
	%0 = call i32([2 x i32]*) @"<[i32; 2]>::x"([2 x i32]* %vector)
	%1 = call i32([2 x i32]*) @"<[i32; 2]>::y"([2 x i32]* %vector)
	%2 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.0 to i8*), i32 %0, i32 %1)
	ret i32 0
}

@.const.0 = private unnamed_addr constant [10 x i8] c"(%d, %d)\0A\00"

define i32 @"<[i32; 2]>::x"([2 x i32]* %0) {
.block.0:
	%self = alloca [2 x i32]*
	store [2 x i32]* %0, [2 x i32]** %self
	%1 = load [2 x i32]*, [2 x i32]** %self
	%2 = getelementptr inbounds [2 x i32], [2 x i32]* %1, i32 0, i32 0
	%3 = load i32, i32* %2
	ret i32 %3
}

define i32 @"<[i32; 2]>::y"([2 x i32]* %0) {
.block.0:
	%self = alloca [2 x i32]*
	store [2 x i32]* %0, [2 x i32]** %self
	%1 = load [2 x i32]*, [2 x i32]** %self
	%2 = getelementptr inbounds [2 x i32], [2 x i32]* %1, i32 0, i32 1
	%3 = load i32, i32* %2
	ret i32 %3
}

%"type.::First" = type {}

%"type.::thing::Thing" = type { %"type.::Test"*, %"type.::First"*, %"type.::thing::Thing"* }

%"type.::Test" = type { %"type.::test::Test"*, %"type.::test::test::Test"*, %"type.::thing::Thing"* }

%"type.::test::Test" = type { %"type.::Test"*, %"type.::test::test::Test"*, %"type.::thing::Thing"* }

define i32 @"::test::test::Test::do_thing"(%"type.::test::test::Test"* %0, i32 %1) {
.block.0:
	%self = alloca %"type.::test::test::Test"*
	store %"type.::test::test::Test"* %0, %"type.::test::test::Test"** %self
	%x = alloca i32
	store i32 %1, i32* %x
	%2 = load i32, i32* %x
	%3 = load i32, i32* %x
	%4 = mul nsw i32 %2, %3
	ret i32 %4
}

%"type.::test::test::Test" = type { %"type.::Test"*, %"type.::test::Test"*, %"type.::thing::Thing"* }

declare i8* @malloc(i64)

declare i8* @calloc(i64, i64)

declare i8* @realloc(i8*, i64)

declare void @free(i8*)

declare i32 @rand()

declare void @srand(i32)

declare i32 @atexit(void()*)

declare void @exit(i32)

declare i32 @isalnum(i32)

declare i32 @isalpha(i32)

declare i32 @islower(i32)

declare i32 @isupper(i32)

declare i32 @isdigit(i32)

declare i32 @isxdigit(i32)

declare i32 @iscntrl(i32)

declare i32 @isgraph(i32)

declare i32 @isspace(i32)

declare i32 @isblank(i32)

declare i32 @isprint(i32)

declare i32 @ispunct(i32)

declare i32 @tolower(i32)

declare i32 @toupper(i32)

%"type.::libc::stdio::CFile" = type opaque

declare %"type.::libc::stdio::CFile"* @fopen(i8*, i8*)

declare i32 @fclose(%"type.::libc::stdio::CFile"*)

declare i32 @feof(%"type.::libc::stdio::CFile"*)

declare i8* @fgets(i8*, i32, %"type.::libc::stdio::CFile"*)

declare i32 @printf(i8*, ...)

declare i32 @puts(i8*)

declare i64 @strlen(i8*)

declare i8* @memcpy(i8*, i8*, i64)

