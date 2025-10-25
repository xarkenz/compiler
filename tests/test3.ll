; file_id = 0
source_filename = "tests/test3.txt"

define i1 @"::a"() {
.block.0:
	ret i1 true
}

define i1 @"::b"() {
.block.0:
	ret i1 false
}

define i1 @"::c"() {
.block.0:
	ret i1 true
}

define i32 @main() {
.block.0:
	%0 = call i1() @"::a"()
	br i1 %0, label %.block.2, label %.block.1
.block.1:
	%1 = call i1() @"::b"()
	br i1 %1, label %.block.3, label %.block.4
.block.3:
	%2 = call i1() @"::c"()
	br label %.block.4
.block.4:
	%3 = phi i1 [ false, %.block.1 ], [ %2, %.block.3 ]
	br label %.block.2
.block.2:
	%4 = phi i1 [ true, %.block.0 ], [ %3, %.block.4 ]
	%x = alloca i1
	store i1 %4, i1* %x
	ret i32 0
}

