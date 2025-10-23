; file_id = 0
source_filename = "tests/type_path_problem.txt"

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

define i32 @main() {
.block.0:
	%arr = alloca [2 x i32]
	store [2 x i32] [ i32 5, i32 7 ], [2 x i32]* %arr
	%0 = call i32([2 x i32]*) @"<[i32; 2]>::x"([2 x i32]* %arr)
	%1 = call i32([2 x i32]*) @"<[i32; 2]>::y"([2 x i32]* %arr)
	%2 = call i32(i8*, ...) @printf(i8* bitcast ([9 x i8]* @.const.0 to i8*), i32 %0, i32 %1)
	ret i32 0
}

@.const.0 = private unnamed_addr constant [9 x i8] c"(%d, %d)\00"

declare i32 @printf(i8*, ...)

