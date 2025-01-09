; file_id = 0
source_filename = "tests/test2.txt"

define i32 @"[i32; 2]::x"([2 x i32]* %0) {
.block.0:
	%self = alloca [2 x i32]*
	store [2 x i32]* %0, [2 x i32]** %self
	%1 = load [2 x i32]*, [2 x i32]** %self
	%2 = getelementptr inbounds [2 x i32], [2 x i32]* %1, i32 0, i32 0
	%3 = load i32, i32* %2
	ret i32 %3
}

define i32 @"[i32; 2]::y"([2 x i32]* %0) {
.block.0:
	%self = alloca [2 x i32]*
	store [2 x i32]* %0, [2 x i32]** %self
	%1 = load [2 x i32]*, [2 x i32]** %self
	%2 = getelementptr inbounds [2 x i32], [2 x i32]* %1, i32 0, i32 1
	%3 = load i32, i32* %2
	ret i32 %3
}

define i32 @main() {
.block.0:
	%vector = alloca [2 x i32]
	store [2 x i32] [ i32 4, i32 8 ], [2 x i32]* %vector
	%0 = call i32([2 x i32]*) @"[i32; 2]::x"([2 x i32]* %vector)
	%1 = call i32([2 x i32]*) @"[i32; 2]::y"([2 x i32]* %vector)
	%2 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.0 to i8*), i32 %0, i32 %1)
	ret i32 0
}

@.const.0 = private unnamed_addr constant [10 x i8] c"(%d, %d)\0A\00"

%"type.thing::Thing" = type {}

%"type.Test" = type { %"type.thing::Thing"* }

%"type.test::Test" = type { %"type.Test"*, %"type.thing::Thing"* }

%"type.test::test::Test" = type { %"type.Test"*, %"type.test::Test"*, %"type.thing::Thing"* }

declare void @free({}*)

declare {}* @malloc(i64)

declare i32 @printf(i8*, ...)

