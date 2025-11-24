source_filename = "\\\\?\\C:\\Users\\seane\\Projects\\compiler\\tests\\packages\\hello\\main.cupr"

declare i32 @puts(i8*)

@.const.hello.0 = private unnamed_addr constant [13 x i8] c"Hello world!\00"

define i32 @main() {
.block.0:
	%0 = call i32(i8*) @puts(i8* bitcast ([13 x i8]* @.const.hello.0 to i8*))
	ret i32 0
}

