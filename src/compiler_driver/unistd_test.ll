; module_id = 0
source_filename = "./src/compiler_driver/unistd_test.txt"

define dso_local i32 @main() {
.block.0:
	%parent_pid = alloca i32
	%0 = call i32() @getpid()
	store i32 %0, i32* %parent_pid
	%1 = load i32, i32* %parent_pid
	%2 = call i32(i8*, ...) @printf(i8* noundef bitcast ([31 x i8]* @.const.0 to i8*), i32 noundef %1)
	%child_pid = alloca i32
	%3 = call i32() @fork()
	store i32 %3, i32* %child_pid
	%4 = load i32, i32* %child_pid
	%5 = icmp eq i32 %4, 0
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = call i32() @getpid()
	store i32 %6, i32* %child_pid
	%7 = load i32, i32* %child_pid
	%8 = call i32(i8*, ...) @printf(i8* noundef bitcast ([26 x i8]* @.const.1 to i8*), i32 noundef %7)
	%9 = load i32, i32* %parent_pid
	%10 = call i32(i8*, ...) @printf(i8* noundef bitcast ([23 x i8]* @.const.2 to i8*), i32 noundef %9)
	br label %.block.3
.block.2:
	%11 = call i32(i32*) @wait(i32* noundef null)
	%12 = load i32, i32* %child_pid
	%13 = call i32(i8*, ...) @printf(i8* noundef bitcast ([26 x i8]* @.const.3 to i8*), i32 noundef %12)
	br label %.block.3
.block.3:
	ret i32 0
}

@.const.0 = private unnamed_addr constant [31 x i8] c"1. Parent (P) is having ID %d\0A\00"
@.const.1 = private unnamed_addr constant [26 x i8] c"2. Child is having ID %d\0A\00"
@.const.2 = private unnamed_addr constant [23 x i8] c"3. My Parent ID is %d\0A\00"
@.const.3 = private unnamed_addr constant [26 x i8] c"4. ID of P's Child is %d\0A\00"

declare i32 @fork()

declare i32 @getpid()

declare i32 @getppid()

declare i32 @printf(i8* noundef, ...)

declare i32 @wait(i32* noundef)

!llvm.module.flags = !{ !0, !1, !2, !3, !4 }
!llvm.ident = !{ !5 }
!0 = !{ i32 1, !"wchar_size", i32 4 }
!1 = !{ i32 7, !"PIC Level", i32 2 }
!2 = !{ i32 7, !"PIE Level", i32 2 }
!3 = !{ i32 7, !"uwtable", i32 1 }
!4 = !{ i32 7, !"frame-pointer", i32 2 }
!5 = !{ !"xarkenz compiler" }
