; file_id = 0
source_filename = "tests/sources/test_3.cupr"

declare i32 @printf(i8*, ...)

define i32 @main() {
.block.0:
	%f1 = alloca float
	store float 0x4008000000000000, float* %f1
	%f2 = alloca float
	store float 0x4018000000000000, float* %f2
	%0 = load float, float* %f1
	%1 = load float, float* %f2
	%2 = fadd float %0, %1
	%f3 = alloca float
	store float %2, float* %f3
	%3 = load float, float* %f3
	%4 = fpext float %3 to double
	%5 = call i32(i8*, ...) @printf(i8* bitcast ([12 x i8]* @.const.0 to i8*), double %4)
	ret i32 0
}

@.const.0 = private unnamed_addr constant [12 x i8] c"Result: %f\0A\00"

