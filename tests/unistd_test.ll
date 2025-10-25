; file_id = 0
source_filename = "tests/unistd_test.txt"

declare i32 @printf(i8*, ...)

declare i32 @getpid()

declare i32 @getppid()

declare i32 @fork()

declare i32 @wait(i32*)

define i32 @main() {
.block.0:
	%0 = call i32() @getpid()
	%parent_pid = alloca i32
	store i32 %0, i32* %parent_pid
	%1 = load i32, i32* %parent_pid
	%2 = call i32(i8*, ...) @printf(i8* bitcast ([31 x i8]* @.const.0 to i8*), i32 %1)
	%3 = call i32() @fork()
	%child_pid = alloca i32
	store i32 %3, i32* %child_pid
	%4 = load i32, i32* %child_pid
	%5 = icmp eq i32 %4, 0
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = call i32() @getpid()
	%child_pid-1 = alloca i32
	store i32 %6, i32* %child_pid-1
	%7 = load i32, i32* %child_pid-1
	%8 = call i32(i8*, ...) @printf(i8* bitcast ([26 x i8]* @.const.1 to i8*), i32 %7)
	%9 = load i32, i32* %parent_pid
	%10 = call i32(i8*, ...) @printf(i8* bitcast ([23 x i8]* @.const.2 to i8*), i32 %9)
	br label %.block.3
.block.2:
	%11 = call i32(i32*) @wait(i32* null)
	%12 = load i32, i32* %child_pid
	%13 = call i32(i8*, ...) @printf(i8* bitcast ([26 x i8]* @.const.3 to i8*), i32 %12)
	br label %.block.3
.block.3:
	ret i32 0
}

@.const.0 = private unnamed_addr constant [31 x i8] c"1. Parent (P) is having ID %d\0A\00"
@.const.1 = private unnamed_addr constant [26 x i8] c"2. Child is having ID %d\0A\00"
@.const.2 = private unnamed_addr constant [23 x i8] c"3. My Parent ID is %d\0A\00"
@.const.3 = private unnamed_addr constant [26 x i8] c"4. ID of P's Child is %d\0A\00"

