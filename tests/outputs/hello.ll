; file_id = 0
source_filename = "tests/sources/hello.cu"

declare i32 @puts(i8*)

define i32 @main() {
.block.0:
	%0 = call i32(i8*) @puts(i8* bitcast ([13 x i8]* @.const.0 to i8*))
	ret i32 0
}

@.const.0 = private unnamed_addr constant [13 x i8] c"Hello world!\00"

