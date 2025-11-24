source_filename = "\\\\?\\C:\\Users\\seane\\Projects\\compiler\\tests\\packages\\test_2\\main.cupr"

%"::test_2::thing::Thing" = type { %"::test_2::Test"*, %"::test_2::First"*, %"::test_2::thing::Thing"* }

%"::test_2::test::Test" = type { %"::test_2::Test"*, %"::test_2::test::test::Test"*, %"::test_2::thing::Thing"* }

%"::test_2::First" = type {}

%"::test_2::test::test::Test" = type { %"::test_2::Test"*, %"::test_2::test::Test"*, %"::test_2::thing::Thing"* }

%"::test_2::Test" = type { %"::test_2::test::Test"*, %"::test_2::test::test::Test"*, %"::test_2::thing::Thing"* }

declare i32 @printf(i8*, ...)

@.const.test_2.0 = private unnamed_addr constant [10 x i8] c"(%d, %d)\0A\00"

define i32 @main() {
.block.0:
	%vector = alloca [2 x i32]
	store [2 x i32] [ i32 4, i32 8 ], [2 x i32]* %vector
	%0 = call i32([2 x i32]*) @"<[i32; 2]>::x"([2 x i32]* %vector)
	%1 = call i32([2 x i32]*) @"<[i32; 2]>::y"([2 x i32]* %vector)
	%2 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.test_2.0 to i8*), i32 %0, i32 %1)
	ret i32 0
}

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

define i32 @"::test_2::test::test::Test::do_thing"(%"::test_2::test::test::Test"* %0, i32 %1) {
.block.0:
	%self = alloca %"::test_2::test::test::Test"*
	store %"::test_2::test::test::Test"* %0, %"::test_2::test::test::Test"** %self
	%x = alloca i32
	store i32 %1, i32* %x
	%2 = load i32, i32* %x
	%3 = load i32, i32* %x
	%4 = mul nsw i32 %2, %3
	ret i32 %4
}

