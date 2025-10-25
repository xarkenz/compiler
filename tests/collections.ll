; file_id = 0
source_filename = "tests/collections.txt"

declare {}* @malloc(i64)

declare void @free({}*)

declare i32 @printf(i8*, ...)

define i32 @"<i32>::max"(i32 %0, i32 %1) {
.block.0:
	%self = alloca i32
	store i32 %0, i32* %self
	%other = alloca i32
	store i32 %1, i32* %other
	%2 = load i32, i32* %self
	%3 = load i32, i32* %other
	%4 = icmp sgt i32 %2, %3
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	%5 = load i32, i32* %self
	br label %.block.3
.block.2:
	%6 = load i32, i32* %other
	br label %.block.3
.block.3:
	%7 = phi i32 [ %5, %.block.1 ], [ %6, %.block.2 ]
	ret i32 %7
}

define i32 @"<i32>::cmp"(i32* %0, i32* %1) {
.block.0:
	%self = alloca i32*
	store i32* %0, i32** %self
	%other = alloca i32*
	store i32* %1, i32** %other
	%2 = load i32*, i32** %self
	%3 = load i32, i32* %2
	%4 = load i32*, i32** %other
	%5 = load i32, i32* %4
	%6 = icmp slt i32 %3, %5
	br i1 %6, label %.block.1, label %.block.2
.block.1:
	%7 = sub nsw i32 0, 1
	br label %.block.3
.block.2:
	%8 = load i32*, i32** %self
	%9 = load i32, i32* %8
	%10 = load i32*, i32** %other
	%11 = load i32, i32* %10
	%12 = icmp sgt i32 %9, %11
	br i1 %12, label %.block.4, label %.block.5
.block.4:
	br label %.block.6
.block.5:
	br label %.block.6
.block.6:
	%13 = phi i32 [ 1, %.block.4 ], [ 0, %.block.5 ]
	br label %.block.3
.block.3:
	%14 = phi i32 [ %7, %.block.1 ], [ %13, %.block.6 ]
	ret i32 %14
}

define void @"<i32>::print"(i32* %0) {
.block.0:
	%self = alloca i32*
	store i32* %0, i32** %self
	%1 = load i32*, i32** %self
	%2 = load i32, i32* %1
	%3 = call i32(i8*, ...) @printf(i8* bitcast ([3 x i8]* @.const.0 to i8*), i32 %2)
	ret void
}

@.const.0 = private unnamed_addr constant [3 x i8] c"%d\00"

%"type.::LinkedListNode" = type { {}*, %"type.::LinkedListNode"* }

%"type.::LinkedList" = type { %"type.::LinkedListNode"* }

define %"type.::LinkedList" @"::LinkedList::new"() {
.block.0:
	ret %"type.::LinkedList" { %"type.::LinkedListNode"* null }
}

define {}* @"::LinkedList::front"(%"type.::LinkedList"* %0) {
.block.0:
	%self = alloca %"type.::LinkedList"*
	store %"type.::LinkedList"* %0, %"type.::LinkedList"** %self
	%1 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%2 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %1, i32 0, i32 0
	%3 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %2
	%4 = icmp eq %"type.::LinkedListNode"* %3, null
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	br label %.block.3
.block.2:
	%5 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%6 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %5, i32 0, i32 0
	%7 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %6
	%8 = getelementptr inbounds %"type.::LinkedListNode", %"type.::LinkedListNode"* %7, i32 0, i32 0
	%9 = load {}*, {}** %8
	br label %.block.3
.block.3:
	%10 = phi {}* [ null, %.block.1 ], [ %9, %.block.2 ]
	ret {}* %10
}

define void @"::LinkedList::push_front"(%"type.::LinkedList"* %0, {}* %1) {
.block.0:
	%self = alloca %"type.::LinkedList"*
	store %"type.::LinkedList"* %0, %"type.::LinkedList"** %self
	%value = alloca {}*
	store {}* %1, {}** %value
	%2 = call {}*(i64) @malloc(i64 16)
	%3 = bitcast {}* %2 to %"type.::LinkedListNode"*
	%new_node = alloca %"type.::LinkedListNode"*
	store %"type.::LinkedListNode"* %3, %"type.::LinkedListNode"** %new_node
	%4 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %new_node
	%5 = load {}*, {}** %value
	%6 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%7 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %6, i32 0, i32 0
	%8 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %7
	%9 = alloca %"type.::LinkedListNode"
	store %"type.::LinkedListNode" { {}* undef, %"type.::LinkedListNode"* undef }, %"type.::LinkedListNode"* %9
	%10 = getelementptr inbounds %"type.::LinkedListNode", %"type.::LinkedListNode"* %9, i32 0, i32 0
	store {}* %5, {}** %10
	%11 = getelementptr inbounds %"type.::LinkedListNode", %"type.::LinkedListNode"* %9, i32 0, i32 1
	store %"type.::LinkedListNode"* %8, %"type.::LinkedListNode"** %11
	%12 = load %"type.::LinkedListNode", %"type.::LinkedListNode"* %9
	store %"type.::LinkedListNode" %12, %"type.::LinkedListNode"* %4
	%13 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%14 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %13, i32 0, i32 0
	%15 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %new_node
	store %"type.::LinkedListNode"* %15, %"type.::LinkedListNode"** %14
	ret void
}

define {}* @"::LinkedList::pop_front"(%"type.::LinkedList"* %0) {
.block.0:
	%self = alloca %"type.::LinkedList"*
	store %"type.::LinkedList"* %0, %"type.::LinkedList"** %self
	%1 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%2 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %1, i32 0, i32 0
	%3 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %2
	%4 = icmp eq %"type.::LinkedListNode"* %3, null
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	br label %.block.3
.block.2:
	%5 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%6 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %5, i32 0, i32 0
	%7 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %6
	%front = alloca %"type.::LinkedListNode"*
	store %"type.::LinkedListNode"* %7, %"type.::LinkedListNode"** %front
	%8 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %front
	%9 = getelementptr inbounds %"type.::LinkedListNode", %"type.::LinkedListNode"* %8, i32 0, i32 0
	%10 = load {}*, {}** %9
	%value = alloca {}*
	store {}* %10, {}** %value
	%11 = load %"type.::LinkedList"*, %"type.::LinkedList"** %self
	%12 = getelementptr inbounds %"type.::LinkedList", %"type.::LinkedList"* %11, i32 0, i32 0
	%13 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %front
	%14 = getelementptr inbounds %"type.::LinkedListNode", %"type.::LinkedListNode"* %13, i32 0, i32 1
	%15 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %14
	store %"type.::LinkedListNode"* %15, %"type.::LinkedListNode"** %12
	%16 = load %"type.::LinkedListNode"*, %"type.::LinkedListNode"** %front
	%17 = bitcast %"type.::LinkedListNode"* %16 to {}*
	call void({}*) @free({}* %17)
	%18 = load {}*, {}** %value
	br label %.block.3
.block.3:
	%19 = phi {}* [ null, %.block.1 ], [ %18, %.block.2 ]
	ret {}* %19
}

%"type.::AVLTreeNode" = type { {}*, %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"*, i32 }

define %"type.::AVLTreeNode"* @"::AVLTreeNode::alloc"({}* %0) {
.block.0:
	%key = alloca {}*
	store {}* %0, {}** %key
	%1 = call {}*(i64) @malloc(i64 32)
	%2 = bitcast {}* %1 to %"type.::AVLTreeNode"*
	%alloc = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %2, %"type.::AVLTreeNode"** %alloc
	%3 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %alloc
	%4 = load {}*, {}** %key
	%5 = alloca %"type.::AVLTreeNode"
	store %"type.::AVLTreeNode" { {}* undef, %"type.::AVLTreeNode"* null, %"type.::AVLTreeNode"* null, i32 0 }, %"type.::AVLTreeNode"* %5
	%6 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %5, i32 0, i32 0
	store {}* %4, {}** %6
	%7 = load %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %5
	store %"type.::AVLTreeNode" %7, %"type.::AVLTreeNode"* %3
	%8 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %alloc
	ret %"type.::AVLTreeNode"* %8
}

define i32 @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%1 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%2 = icmp eq %"type.::AVLTreeNode"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = sub nsw i32 0, 1
	br label %.block.3
.block.2:
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %4, i32 0, i32 3
	%6 = load i32, i32* %5
	br label %.block.3
.block.3:
	%7 = phi i32 [ %3, %.block.1 ], [ %6, %.block.2 ]
	ret i32 %7
}

define void @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%1 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %1, i32 0, i32 3
	%3 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%4 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %3, i32 0, i32 1
	%5 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %4
	%6 = bitcast %"type.::AVLTreeNode"* %5 to %"type.::AVLTreeNode"*
	%7 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %6)
	%8 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%9 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %8, i32 0, i32 2
	%10 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %9
	%11 = bitcast %"type.::AVLTreeNode"* %10 to %"type.::AVLTreeNode"*
	%12 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %11)
	%13 = call i32(i32, i32) @"<i32>::max"(i32 %7, i32 %12)
	%14 = add nsw i32 1, %13
	store i32 %14, i32* %2
	ret void
}

define %"type.::AVLTreeNode"* @"::AVLTreeNode::rotate_right"(%"type.::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%1 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %1, i32 0, i32 1
	%3 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %2
	%new_root = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %3, %"type.::AVLTreeNode"** %new_root
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %4, i32 0, i32 1
	%6 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	%7 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %6, i32 0, i32 2
	%8 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %7
	store %"type.::AVLTreeNode"* %8, %"type.::AVLTreeNode"** %5
	%9 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	%10 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %9, i32 0, i32 2
	%11 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	store %"type.::AVLTreeNode"* %11, %"type.::AVLTreeNode"** %10
	%12 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	call void(%"type.::AVLTreeNode"*) @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %12)
	%13 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	call void(%"type.::AVLTreeNode"*) @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %13)
	%14 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	ret %"type.::AVLTreeNode"* %14
}

define %"type.::AVLTreeNode"* @"::AVLTreeNode::rotate_left"(%"type.::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%1 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %1, i32 0, i32 2
	%3 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %2
	%new_root = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %3, %"type.::AVLTreeNode"** %new_root
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %4, i32 0, i32 2
	%6 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	%7 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %6, i32 0, i32 1
	%8 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %7
	store %"type.::AVLTreeNode"* %8, %"type.::AVLTreeNode"** %5
	%9 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	%10 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %9, i32 0, i32 1
	%11 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	store %"type.::AVLTreeNode"* %11, %"type.::AVLTreeNode"** %10
	%12 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	call void(%"type.::AVLTreeNode"*) @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %12)
	%13 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	call void(%"type.::AVLTreeNode"*) @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %13)
	%14 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %new_root
	ret %"type.::AVLTreeNode"* %14
}

define %"type.::AVLTreeNode"* @"::AVLTreeNode::balance"(%"type.::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%1 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%2 = icmp eq %"type.::AVLTreeNode"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	ret %"type.::AVLTreeNode"* null
.block.2:
	%3 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%4 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %3, i32 0, i32 1
	%5 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %4
	%6 = bitcast %"type.::AVLTreeNode"* %5 to %"type.::AVLTreeNode"*
	%7 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %6)
	%8 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%9 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %8, i32 0, i32 2
	%10 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %9
	%11 = bitcast %"type.::AVLTreeNode"* %10 to %"type.::AVLTreeNode"*
	%12 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %11)
	%13 = sub nsw i32 %7, %12
	%imbalance = alloca i32
	store i32 %13, i32* %imbalance
	%14 = load i32, i32* %imbalance
	%15 = icmp sgt i32 %14, 1
	br i1 %15, label %.block.3, label %.block.4
.block.3:
	%16 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%17 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %16, i32 0, i32 1
	%18 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %17
	%19 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %18, i32 0, i32 2
	%20 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %19
	%21 = bitcast %"type.::AVLTreeNode"* %20 to %"type.::AVLTreeNode"*
	%22 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %21)
	%23 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%24 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %23, i32 0, i32 1
	%25 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %24
	%26 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %25, i32 0, i32 1
	%27 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %26
	%28 = bitcast %"type.::AVLTreeNode"* %27 to %"type.::AVLTreeNode"*
	%29 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %28)
	%30 = icmp sgt i32 %22, %29
	br i1 %30, label %.block.5, label %.block.6
.block.5:
	%31 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%32 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %31, i32 0, i32 1
	%33 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%34 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %33, i32 0, i32 1
	%35 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %34
	%36 = call %"type.::AVLTreeNode"*(%"type.::AVLTreeNode"*) @"::AVLTreeNode::rotate_left"(%"type.::AVLTreeNode"* %35)
	store %"type.::AVLTreeNode"* %36, %"type.::AVLTreeNode"** %32
	br label %.block.6
.block.6:
	%37 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%38 = call %"type.::AVLTreeNode"*(%"type.::AVLTreeNode"*) @"::AVLTreeNode::rotate_right"(%"type.::AVLTreeNode"* %37)
	br label %.block.7
.block.4:
	%39 = load i32, i32* %imbalance
	%40 = sub nsw i32 0, 1
	%41 = icmp slt i32 %39, %40
	br i1 %41, label %.block.8, label %.block.9
.block.8:
	%42 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%43 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %42, i32 0, i32 2
	%44 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %43
	%45 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %44, i32 0, i32 1
	%46 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %45
	%47 = bitcast %"type.::AVLTreeNode"* %46 to %"type.::AVLTreeNode"*
	%48 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %47)
	%49 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%50 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %49, i32 0, i32 2
	%51 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %50
	%52 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %51, i32 0, i32 2
	%53 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %52
	%54 = bitcast %"type.::AVLTreeNode"* %53 to %"type.::AVLTreeNode"*
	%55 = call i32(%"type.::AVLTreeNode"*) @"::AVLTreeNode::get_height"(%"type.::AVLTreeNode"* %54)
	%56 = icmp sgt i32 %48, %55
	br i1 %56, label %.block.10, label %.block.11
.block.10:
	%57 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%58 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %57, i32 0, i32 2
	%59 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%60 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %59, i32 0, i32 2
	%61 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %60
	%62 = call %"type.::AVLTreeNode"*(%"type.::AVLTreeNode"*) @"::AVLTreeNode::rotate_right"(%"type.::AVLTreeNode"* %61)
	store %"type.::AVLTreeNode"* %62, %"type.::AVLTreeNode"** %58
	br label %.block.11
.block.11:
	%63 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%64 = call %"type.::AVLTreeNode"*(%"type.::AVLTreeNode"*) @"::AVLTreeNode::rotate_left"(%"type.::AVLTreeNode"* %63)
	br label %.block.12
.block.9:
	%65 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	call void(%"type.::AVLTreeNode"*) @"::AVLTreeNode::recompute_height"(%"type.::AVLTreeNode"* %65)
	%66 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	br label %.block.12
.block.12:
	%67 = phi %"type.::AVLTreeNode"* [ %64, %.block.11 ], [ %66, %.block.9 ]
	br label %.block.7
.block.7:
	%68 = phi %"type.::AVLTreeNode"* [ %38, %.block.6 ], [ %67, %.block.12 ]
	ret %"type.::AVLTreeNode"* %68
}

define void @"::AVLTreeNode::print"(%"type.::AVLTreeNode"* %0, void({}*)* %1) {
.block.0:
	%self = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %0, %"type.::AVLTreeNode"** %self
	%printer = alloca void({}*)*
	store void({}*)* %1, void({}*)** %printer
	%2 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%3 = icmp ne %"type.::AVLTreeNode"* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	%4 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.1 to i8*))
	%5 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%6 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %5, i32 0, i32 1
	%7 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %6
	%8 = load void({}*)*, void({}*)** %printer
	call void(%"type.::AVLTreeNode"*, void({}*)*) @"::AVLTreeNode::print"(%"type.::AVLTreeNode"* %7, void({}*)* %8)
	%9 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.2 to i8*))
	%10 = load void({}*)*, void({}*)** %printer
	%11 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%12 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %11, i32 0, i32 0
	%13 = load {}*, {}** %12
	call void({}*) %10({}* %13)
	%14 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.3 to i8*))
	%15 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %self
	%16 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %15, i32 0, i32 2
	%17 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %16
	%18 = load void({}*)*, void({}*)** %printer
	call void(%"type.::AVLTreeNode"*, void({}*)*) @"::AVLTreeNode::print"(%"type.::AVLTreeNode"* %17, void({}*)* %18)
	%19 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.4 to i8*))
	br label %.block.2
.block.2:
	ret void
}

@.const.1 = private unnamed_addr constant [2 x i8] c"(\00"
@.const.2 = private unnamed_addr constant [2 x i8] c" \00"
@.const.3 = private unnamed_addr constant [2 x i8] c" \00"
@.const.4 = private unnamed_addr constant [2 x i8] c")\00"

%"type.::AVLTree" = type { %"type.::AVLTreeNode"*, i32({}*, {}*)* }

define %"type.::AVLTree" @"::AVLTree::new"(i32({}*, {}*)* %0) {
.block.0:
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %0, i32({}*, {}*)** %comparator
	%1 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%2 = alloca %"type.::AVLTree"
	store %"type.::AVLTree" { %"type.::AVLTreeNode"* null, i32({}*, {}*)* undef }, %"type.::AVLTree"* %2
	%3 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %2, i32 0, i32 1
	store i32({}*, {}*)* %1, i32({}*, {}*)** %3
	%4 = load %"type.::AVLTree", %"type.::AVLTree"* %2
	ret %"type.::AVLTree" %4
}

define {}* @"::AVLTree::get"(%"type.::AVLTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"type.::AVLTree"*
	store %"type.::AVLTree"* %0, %"type.::AVLTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%3 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %2, i32 0, i32 0
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %3
	%node = alloca %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %4, %"type.::AVLTreeNode"** %node
	br label %.block.1
.block.1:
	%5 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %node
	%6 = icmp ne %"type.::AVLTreeNode"* %5, null
	br i1 %6, label %.block.2, label %.block.3
.block.2:
	%7 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%8 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %7, i32 0, i32 1
	%9 = load i32({}*, {}*)*, i32({}*, {}*)** %8
	%10 = load {}*, {}** %key
	%11 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %node
	%12 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %11, i32 0, i32 0
	%13 = load {}*, {}** %12
	%14 = call i32({}*, {}*) %9({}* %10, {}* %13)
	%ordering = alloca i32
	store i32 %14, i32* %ordering
	%15 = load i32, i32* %ordering
	%16 = icmp slt i32 %15, 0
	br i1 %16, label %.block.4, label %.block.5
.block.4:
	%17 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %node
	%18 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %17, i32 0, i32 1
	%19 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %18
	%20 = bitcast %"type.::AVLTreeNode"* %19 to %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %20, %"type.::AVLTreeNode"** %node
	br label %.block.6
.block.5:
	%21 = load i32, i32* %ordering
	%22 = icmp sgt i32 %21, 0
	br i1 %22, label %.block.7, label %.block.8
.block.7:
	%23 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %node
	%24 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %23, i32 0, i32 2
	%25 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %24
	%26 = bitcast %"type.::AVLTreeNode"* %25 to %"type.::AVLTreeNode"*
	store %"type.::AVLTreeNode"* %26, %"type.::AVLTreeNode"** %node
	br label %.block.9
.block.8:
	%27 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %node
	%28 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %27, i32 0, i32 0
	%29 = load {}*, {}** %28
	ret {}* %29
.block.9:
	br label %.block.6
.block.6:
	br label %.block.1
.block.3:
	ret {}* null
}

define {}* @"::AVLTree::insert_subtree"(%"type.::AVLTree"* %0, %"type.::AVLTreeNode"** %1, {}* %2) {
.block.0:
	%self = alloca %"type.::AVLTree"*
	store %"type.::AVLTree"* %0, %"type.::AVLTree"** %self
	%node_ref = alloca %"type.::AVLTreeNode"**
	store %"type.::AVLTreeNode"** %1, %"type.::AVLTreeNode"*** %node_ref
	%key = alloca {}*
	store {}* %2, {}** %key
	%3 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %3
	%5 = icmp eq %"type.::AVLTreeNode"* %4, null
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%7 = load {}*, {}** %key
	%8 = call %"type.::AVLTreeNode"*({}*) @"::AVLTreeNode::alloc"({}* %7)
	store %"type.::AVLTreeNode"* %8, %"type.::AVLTreeNode"** %6
	ret {}* null
.block.2:
	%9 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%10 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %9, i32 0, i32 1
	%11 = load i32({}*, {}*)*, i32({}*, {}*)** %10
	%12 = load {}*, {}** %key
	%13 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%14 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %13
	%15 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %14, i32 0, i32 0
	%16 = load {}*, {}** %15
	%17 = call i32({}*, {}*) %11({}* %12, {}* %16)
	%ordering = alloca i32
	store i32 %17, i32* %ordering
	%18 = load i32, i32* %ordering
	%19 = icmp slt i32 %18, 0
	br i1 %19, label %.block.3, label %.block.4
.block.3:
	%20 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%21 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%22 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %21
	%23 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %22, i32 0, i32 1
	%24 = load {}*, {}** %key
	%25 = call {}*(%"type.::AVLTree"*, %"type.::AVLTreeNode"**, {}*) @"::AVLTree::insert_subtree"(%"type.::AVLTree"* %20, %"type.::AVLTreeNode"** %23, {}* %24)
	br label %.block.5
.block.4:
	%26 = load i32, i32* %ordering
	%27 = icmp sgt i32 %26, 0
	br i1 %27, label %.block.6, label %.block.7
.block.6:
	%28 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%29 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%30 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %29
	%31 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %30, i32 0, i32 2
	%32 = load {}*, {}** %key
	%33 = call {}*(%"type.::AVLTree"*, %"type.::AVLTreeNode"**, {}*) @"::AVLTree::insert_subtree"(%"type.::AVLTree"* %28, %"type.::AVLTreeNode"** %31, {}* %32)
	br label %.block.8
.block.7:
	%34 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%35 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %34
	%36 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %35, i32 0, i32 0
	%37 = load {}*, {}** %36
	%replaced_key = alloca {}*
	store {}* %37, {}** %replaced_key
	%38 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%39 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %38
	%40 = getelementptr inbounds %"type.::AVLTreeNode", %"type.::AVLTreeNode"* %39, i32 0, i32 0
	%41 = load {}*, {}** %key
	store {}* %41, {}** %40
	%42 = load {}*, {}** %replaced_key
	ret {}* %42
.block.8:
	br label %.block.5
.block.5:
	%43 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%44 = load %"type.::AVLTreeNode"**, %"type.::AVLTreeNode"*** %node_ref
	%45 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %44
	%46 = call %"type.::AVLTreeNode"*(%"type.::AVLTreeNode"*) @"::AVLTreeNode::balance"(%"type.::AVLTreeNode"* %45)
	store %"type.::AVLTreeNode"* %46, %"type.::AVLTreeNode"** %43
	ret {}* null
}

define {}* @"::AVLTree::insert"(%"type.::AVLTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"type.::AVLTree"*
	store %"type.::AVLTree"* %0, %"type.::AVLTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%3 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%4 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %3, i32 0, i32 0
	%5 = load {}*, {}** %key
	%6 = call {}*(%"type.::AVLTree"*, %"type.::AVLTreeNode"**, {}*) @"::AVLTree::insert_subtree"(%"type.::AVLTree"* %2, %"type.::AVLTreeNode"** %4, {}* %5)
	ret {}* %6
}

define void @"::AVLTree::print"(%"type.::AVLTree"* %0, void({}*)* %1) {
.block.0:
	%self = alloca %"type.::AVLTree"*
	store %"type.::AVLTree"* %0, %"type.::AVLTree"** %self
	%printer = alloca void({}*)*
	store void({}*)* %1, void({}*)** %printer
	%2 = load %"type.::AVLTree"*, %"type.::AVLTree"** %self
	%3 = getelementptr inbounds %"type.::AVLTree", %"type.::AVLTree"* %2, i32 0, i32 0
	%4 = load %"type.::AVLTreeNode"*, %"type.::AVLTreeNode"** %3
	%5 = load void({}*)*, void({}*)** %printer
	call void(%"type.::AVLTreeNode"*, void({}*)*) @"::AVLTreeNode::print"(%"type.::AVLTreeNode"* %4, void({}*)* %5)
	%6 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.5 to i8*))
	ret void
}

@.const.5 = private unnamed_addr constant [2 x i8] c"\0A\00"

%"type.::BTreeNodeKey" = type { {}*, %"type.::BTreeNode"* }

%"type.::BTreeNode" = type { i1, i64, %"type.::BTreeNodeKey"*, %"type.::BTreeNode"* }

%"type.::BTreeLeaf" = type { i1, i64, {}** }

define %"type.::BTreeLeaf"* @"::BTreeLeaf::alloc"(i64 %0, {}* %1) {
.block.0:
	%l_order = alloca i64
	store i64 %0, i64* %l_order
	%first_element = alloca {}*
	store {}* %1, {}** %first_element
	%2 = load i64, i64* %l_order
	%3 = mul nuw i64 8, %2
	%4 = call {}*(i64) @malloc(i64 %3)
	%5 = bitcast {}* %4 to {}**
	%elements = alloca {}**
	store {}** %5, {}*** %elements
	%6 = load {}**, {}*** %elements
	%7 = getelementptr inbounds {}*, {}** %6, i32 0
	%8 = load {}*, {}** %first_element
	store {}* %8, {}** %7
	%9 = call {}*(i64) @malloc(i64 24)
	%10 = bitcast {}* %9 to %"type.::BTreeLeaf"*
	%alloc = alloca %"type.::BTreeLeaf"*
	store %"type.::BTreeLeaf"* %10, %"type.::BTreeLeaf"** %alloc
	%11 = load %"type.::BTreeLeaf"*, %"type.::BTreeLeaf"** %alloc
	%12 = load {}**, {}*** %elements
	%13 = alloca %"type.::BTreeLeaf"
	store %"type.::BTreeLeaf" { i1 true, i64 1, {}** undef }, %"type.::BTreeLeaf"* %13
	%14 = getelementptr inbounds %"type.::BTreeLeaf", %"type.::BTreeLeaf"* %13, i32 0, i32 2
	store {}** %12, {}*** %14
	%15 = load %"type.::BTreeLeaf", %"type.::BTreeLeaf"* %13
	store %"type.::BTreeLeaf" %15, %"type.::BTreeLeaf"* %11
	%16 = load %"type.::BTreeLeaf"*, %"type.::BTreeLeaf"** %alloc
	ret %"type.::BTreeLeaf"* %16
}

%"type.::BTree" = type { i64, i64, i32({}*, {}*)*, %"type.::BTreeNode"* }

define %"type.::BTree" @"::BTree::new"(i64 %0, i64 %1, i32({}*, {}*)* %2) {
.block.0:
	%m_order = alloca i64
	store i64 %0, i64* %m_order
	%l_order = alloca i64
	store i64 %1, i64* %l_order
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %2, i32({}*, {}*)** %comparator
	%3 = load i64, i64* %m_order
	%4 = load i64, i64* %l_order
	%5 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%6 = alloca %"type.::BTree"
	store %"type.::BTree" { i64 undef, i64 undef, i32({}*, {}*)* undef, %"type.::BTreeNode"* null }, %"type.::BTree"* %6
	%7 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %6, i32 0, i32 0
	store i64 %3, i64* %7
	%8 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %6, i32 0, i32 1
	store i64 %4, i64* %8
	%9 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %6, i32 0, i32 2
	store i32({}*, {}*)* %5, i32({}*, {}*)** %9
	%10 = load %"type.::BTree", %"type.::BTree"* %6
	ret %"type.::BTree" %10
}

define {}* @"::BTree::insert"(%"type.::BTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"type.::BTree"*
	store %"type.::BTree"* %0, %"type.::BTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"type.::BTree"*, %"type.::BTree"** %self
	%3 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %2, i32 0, i32 3
	%4 = load %"type.::BTreeNode"*, %"type.::BTreeNode"** %3
	%5 = icmp eq %"type.::BTreeNode"* %4, null
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = load %"type.::BTree"*, %"type.::BTree"** %self
	%7 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %6, i32 0, i32 3
	%8 = load %"type.::BTree"*, %"type.::BTree"** %self
	%9 = getelementptr inbounds %"type.::BTree", %"type.::BTree"* %8, i32 0, i32 1
	%10 = load i64, i64* %9
	%11 = load {}*, {}** %key
	%12 = call %"type.::BTreeLeaf"*(i64, {}*) @"::BTreeLeaf::alloc"(i64 %10, {}* %11)
	%13 = bitcast %"type.::BTreeLeaf"* %12 to %"type.::BTreeNode"*
	store %"type.::BTreeNode"* %13, %"type.::BTreeNode"** %7
	ret {}* null
.block.2:
	ret {}* null
}

define void @"::max_percolate_dmut"({}** %0, i64 %1, i32({}*, {}*)* %2, i64 %3) {
.block.0:
	%array = alloca {}**
	store {}** %0, {}*** %array
	%length = alloca i64
	store i64 %1, i64* %length
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %2, i32({}*, {}*)** %comparator
	%index = alloca i64
	store i64 %3, i64* %index
	%4 = load i64, i64* %index
	%5 = load {}**, {}*** %array
	%6 = getelementptr inbounds {}*, {}** %5, i64 %4
	%7 = load {}*, {}** %6
	%target = alloca {}*
	store {}* %7, {}** %target
	br label %.block.1
.block.1:
	br i1 true, label %.block.2, label %.block.3
.block.2:
	%8 = load i64, i64* %index
	%9 = add nuw i64 %8, 1
	%10 = mul nuw i64 %9, 2
	%11 = sub nuw i64 %10, 1
	%left = alloca i64
	store i64 %11, i64* %left
	%12 = load i64, i64* %left
	%13 = add nuw i64 %12, 1
	%right = alloca i64
	store i64 %13, i64* %right
	%14 = load i64, i64* %left
	%15 = load i64, i64* %length
	%16 = icmp uge i64 %14, %15
	br i1 %16, label %.block.4, label %.block.5
.block.4:
	br label %.block.3
.block.5:
	%17 = load i64, i64* %right
	%18 = load i64, i64* %length
	%19 = icmp uge i64 %17, %18
	br i1 %19, label %.block.7, label %.block.6
.block.6:
	%20 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%21 = load i64, i64* %left
	%22 = load {}**, {}*** %array
	%23 = getelementptr inbounds {}*, {}** %22, i64 %21
	%24 = load {}*, {}** %23
	%25 = load i64, i64* %right
	%26 = load {}**, {}*** %array
	%27 = getelementptr inbounds {}*, {}** %26, i64 %25
	%28 = load {}*, {}** %27
	%29 = call i32({}*, {}*) %20({}* %24, {}* %28)
	%30 = icmp sgt i32 %29, 0
	br label %.block.7
.block.7:
	%31 = phi i1 [ true, %.block.5 ], [ %30, %.block.6 ]
	br i1 %31, label %.block.8, label %.block.9
.block.8:
	%32 = load i64, i64* %left
	br label %.block.10
.block.9:
	%33 = load i64, i64* %right
	br label %.block.10
.block.10:
	%34 = phi i64 [ %32, %.block.8 ], [ %33, %.block.9 ]
	%max = alloca i64
	store i64 %34, i64* %max
	%35 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%36 = load i64, i64* %max
	%37 = load {}**, {}*** %array
	%38 = getelementptr inbounds {}*, {}** %37, i64 %36
	%39 = load {}*, {}** %38
	%40 = load {}*, {}** %target
	%41 = call i32({}*, {}*) %35({}* %39, {}* %40)
	%42 = icmp sgt i32 %41, 0
	br i1 %42, label %.block.11, label %.block.12
.block.11:
	%43 = load i64, i64* %index
	%44 = load {}**, {}*** %array
	%45 = getelementptr inbounds {}*, {}** %44, i64 %43
	%46 = load i64, i64* %max
	%47 = load {}**, {}*** %array
	%48 = getelementptr inbounds {}*, {}** %47, i64 %46
	%49 = load {}*, {}** %48
	store {}* %49, {}** %45
	%50 = load i64, i64* %max
	store i64 %50, i64* %index
	br label %.block.13
.block.12:
	br label %.block.3
.block.13:
	br label %.block.1
.block.3:
	%51 = load i64, i64* %index
	%52 = load {}**, {}*** %array
	%53 = getelementptr inbounds {}*, {}** %52, i64 %51
	%54 = load {}*, {}** %target
	store {}* %54, {}** %53
	ret void
}

define void @"::heap_sort"({}** %0, i64 %1, i32({}*, {}*)* %2) {
.block.0:
	%array = alloca {}**
	store {}** %0, {}*** %array
	%length = alloca i64
	store i64 %1, i64* %length
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %2, i32({}*, {}*)** %comparator
	%3 = load i64, i64* %length
	%4 = udiv i64 %3, 2
	%index = alloca i64
	store i64 %4, i64* %index
	br label %.block.1
.block.1:
	%5 = load i64, i64* %index
	%6 = icmp ugt i64 %5, 0
	br i1 %6, label %.block.2, label %.block.3
.block.2:
	%7 = load i64, i64* %index
	%8 = sub nuw i64 %7, 1
	store i64 %8, i64* %index
	%9 = load {}**, {}*** %array
	%10 = load i64, i64* %length
	%11 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%12 = load i64, i64* %index
	call void({}**, i64, i32({}*, {}*)*, i64) @"::max_percolate_dmut"({}** %9, i64 %10, i32({}*, {}*)* %11, i64 %12)
	br label %.block.1
.block.3:
	%13 = load i64, i64* %length
	store i64 %13, i64* %index
	br label %.block.4
.block.4:
	%14 = load i64, i64* %index
	%15 = icmp ugt i64 %14, 1
	br i1 %15, label %.block.5, label %.block.6
.block.5:
	%16 = load i64, i64* %index
	%17 = sub nuw i64 %16, 1
	store i64 %17, i64* %index
	%18 = load {}**, {}*** %array
	%19 = getelementptr inbounds {}*, {}** %18, i32 0
	%20 = load {}*, {}** %19
	%max_value = alloca {}*
	store {}* %20, {}** %max_value
	%21 = load {}**, {}*** %array
	%22 = getelementptr inbounds {}*, {}** %21, i32 0
	%23 = load i64, i64* %index
	%24 = load {}**, {}*** %array
	%25 = getelementptr inbounds {}*, {}** %24, i64 %23
	%26 = load {}*, {}** %25
	store {}* %26, {}** %22
	%27 = load i64, i64* %index
	%28 = load {}**, {}*** %array
	%29 = getelementptr inbounds {}*, {}** %28, i64 %27
	%30 = load {}*, {}** %max_value
	store {}* %30, {}** %29
	%31 = load {}**, {}*** %array
	%32 = load i64, i64* %index
	%33 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	call void({}**, i64, i32({}*, {}*)*, i64) @"::max_percolate_dmut"({}** %31, i64 %32, i32({}*, {}*)* %33, i64 0)
	br label %.block.4
.block.6:
	ret void
}

define void @"::print_i32_ptr_array"(i32** %0, i64 %1) {
.block.0:
	%array = alloca i32**
	store i32** %0, i32*** %array
	%length = alloca i64
	store i64 %1, i64* %length
	%2 = load i64, i64* %length
	%3 = icmp eq i64 %2, 0
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	%4 = call i32(i8*, ...) @printf(i8* bitcast ([3 x i8]* @.const.6 to i8*))
	ret void
.block.2:
	%5 = load i32**, i32*** %array
	%6 = getelementptr inbounds i32*, i32** %5, i32 0
	%7 = load i32*, i32** %6
	%8 = load i32, i32* %7
	%9 = call i32(i8*, ...) @printf(i8* bitcast ([4 x i8]* @.const.7 to i8*), i32 %8)
	%index = alloca i64
	store i64 1, i64* %index
	br label %.block.3
.block.3:
	%10 = load i64, i64* %index
	%11 = load i64, i64* %length
	%12 = icmp ult i64 %10, %11
	br i1 %12, label %.block.4, label %.block.5
.block.4:
	%13 = load i64, i64* %index
	%14 = load i32**, i32*** %array
	%15 = getelementptr inbounds i32*, i32** %14, i64 %13
	%16 = load i32*, i32** %15
	%17 = load i32, i32* %16
	%18 = call i32(i8*, ...) @printf(i8* bitcast ([5 x i8]* @.const.8 to i8*), i32 %17)
	%19 = load i64, i64* %index
	%20 = add nuw i64 %19, 1
	store i64 %20, i64* %index
	br label %.block.3
.block.5:
	%21 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.9 to i8*))
	ret void
}

@.const.6 = private unnamed_addr constant [3 x i8] c"[]\00"
@.const.7 = private unnamed_addr constant [4 x i8] c"[%d\00"
@.const.8 = private unnamed_addr constant [5 x i8] c", %d\00"
@.const.9 = private unnamed_addr constant [2 x i8] c"]\00"

define i32 @main() {
.block.0:
	%keys = alloca [15 x i32]
	store [15 x i32] [ i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15 ], [15 x i32]* %keys
	%0 = call %"type.::LinkedList"() @"::LinkedList::new"()
	%list = alloca %"type.::LinkedList"
	store %"type.::LinkedList" %0, %"type.::LinkedList"* %list
	%i = alloca i64
	store i64 0, i64* %i
	br label %.block.1
.block.1:
	%1 = load i64, i64* %i
	%2 = icmp ult i64 %1, 5
	br i1 %2, label %.block.2, label %.block.3
.block.2:
	%3 = load i64, i64* %i
	%4 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %3
	%5 = load i32, i32* %4
	%6 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.10 to i8*), i32 %5)
	%7 = load i64, i64* %i
	%8 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %7
	%9 = bitcast i32* %8 to {}*
	call void(%"type.::LinkedList"*, {}*) @"::LinkedList::push_front"(%"type.::LinkedList"* %list, {}* %9)
	%10 = load i64, i64* %i
	%11 = add nuw i64 %10, 1
	store i64 %11, i64* %i
	br label %.block.1
.block.3:
	%value = alloca i32*
	br label %.block.4
.block.4:
	%12 = call {}*(%"type.::LinkedList"*) @"::LinkedList::pop_front"(%"type.::LinkedList"* %list)
	%13 = bitcast {}* %12 to i32*
	store i32* %13, i32** %value
	%14 = icmp ne i32* %13, null
	br i1 %14, label %.block.5, label %.block.6
.block.5:
	%15 = load i32*, i32** %value
	%16 = load i32, i32* %15
	%17 = call i32(i8*, ...) @printf(i8* bitcast ([9 x i8]* @.const.11 to i8*), i32 %16)
	br label %.block.4
.block.6:
	%18 = bitcast i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*
	%19 = call %"type.::AVLTree"(i32({}*, {}*)*) @"::AVLTree::new"(i32({}*, {}*)* %18)
	%tree = alloca %"type.::AVLTree"
	store %"type.::AVLTree" %19, %"type.::AVLTree"* %tree
	%i-1 = alloca i64
	store i64 0, i64* %i-1
	br label %.block.7
.block.7:
	%20 = load i64, i64* %i-1
	%21 = icmp ult i64 %20, 7
	br i1 %21, label %.block.8, label %.block.9
.block.8:
	%22 = load i64, i64* %i-1
	%23 = mul nuw i64 %22, 7
	%24 = urem i64 %23, 10
	%idx = alloca i64
	store i64 %24, i64* %idx
	%25 = load i64, i64* %idx
	%26 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %25
	%27 = load i32, i32* %26
	%28 = call i32(i8*, ...) @printf(i8* bitcast ([12 x i8]* @.const.12 to i8*), i32 %27)
	%29 = load i64, i64* %idx
	%30 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %29
	%31 = bitcast i32* %30 to {}*
	%32 = call {}*(%"type.::AVLTree"*, {}*) @"::AVLTree::insert"(%"type.::AVLTree"* %tree, {}* %31)
	%33 = load i64, i64* %i-1
	%34 = add nuw i64 %33, 1
	store i64 %34, i64* %i-1
	br label %.block.7
.block.9:
	%i-2 = alloca i64
	store i64 0, i64* %i-2
	br label %.block.10
.block.10:
	%35 = load i64, i64* %i-2
	%36 = icmp ult i64 %35, 10
	br i1 %36, label %.block.11, label %.block.12
.block.11:
	%37 = bitcast %"type.::AVLTree"* %tree to %"type.::AVLTree"*
	%38 = load i64, i64* %i-2
	%39 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %38
	%40 = bitcast i32* %39 to {}*
	%41 = call {}*(%"type.::AVLTree"*, {}*) @"::AVLTree::get"(%"type.::AVLTree"* %37, {}* %40)
	%key = alloca {}*
	store {}* %41, {}** %key
	%42 = load {}*, {}** %key
	%43 = icmp ne {}* %42, null
	br i1 %43, label %.block.13, label %.block.14
.block.13:
	br label %.block.15
.block.14:
	br label %.block.15
.block.15:
	%44 = phi i8* [ bitcast ([4 x i8]* @.const.13 to i8*), %.block.13 ], [ bitcast ([3 x i8]* @.const.14 to i8*), %.block.14 ]
	%is_contained = alloca i8*
	store i8* %44, i8** %is_contained
	%45 = load i64, i64* %i-2
	%46 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %45
	%47 = load i32, i32* %46
	%48 = load i8*, i8** %is_contained
	%49 = call i32(i8*, ...) @printf(i8* bitcast ([17 x i8]* @.const.15 to i8*), i32 %47, i8* %48)
	%50 = load i64, i64* %i-2
	%51 = add nuw i64 %50, 1
	store i64 %51, i64* %i-2
	br label %.block.10
.block.12:
	%52 = bitcast %"type.::AVLTree"* %tree to %"type.::AVLTree"*
	%53 = bitcast void(i32*)* @"<i32>::print" to void({}*)*
	call void(%"type.::AVLTree"*, void({}*)*) @"::AVLTree::print"(%"type.::AVLTree"* %52, void({}*)* %53)
	%54 = bitcast i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*
	%55 = call %"type.::BTree"(i64, i64, i32({}*, {}*)*) @"::BTree::new"(i64 3, i64 2, i32({}*, {}*)* %54)
	%b_tree = alloca %"type.::BTree"
	store %"type.::BTree" %55, %"type.::BTree"* %b_tree
	%56 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i32 0
	%57 = bitcast i32* %56 to {}*
	%58 = call {}*(%"type.::BTree"*, {}*) @"::BTree::insert"(%"type.::BTree"* %b_tree, {}* %57)
	%heap_sort_test = alloca [15 x i32*]
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.16
.block.16:
	%59 = load i64, i64* %index
	%60 = icmp ult i64 %59, 15
	br i1 %60, label %.block.17, label %.block.18
.block.17:
	%61 = load i64, i64* %index
	%62 = add nuw i64 %61, 7
	%63 = mul nuw i64 %62, 7
	%64 = urem i64 %63, 15
	%65 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %64
	%key-1 = alloca i32*
	store i32* %65, i32** %key-1
	%66 = load i64, i64* %index
	%67 = getelementptr inbounds [15 x i32*], [15 x i32*]* %heap_sort_test, i32 0, i64 %66
	%68 = load i32*, i32** %key-1
	store i32* %68, i32** %67
	%69 = load i64, i64* %index
	%70 = add nuw i64 %69, 1
	store i64 %70, i64* %index
	br label %.block.16
.block.18:
	%71 = call i32(i8*, ...) @printf(i8* bitcast ([11 x i8]* @.const.16 to i8*))
	%72 = bitcast [15 x i32*]* %heap_sort_test to i32**
	call void(i32**, i64) @"::print_i32_ptr_array"(i32** %72, i64 15)
	%73 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.17 to i8*))
	%74 = bitcast [15 x i32*]* %heap_sort_test to {}**
	%75 = bitcast i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*
	call void({}**, i64, i32({}*, {}*)*) @"::heap_sort"({}** %74, i64 15, i32({}*, {}*)* %75)
	%76 = call i32(i8*, ...) @printf(i8* bitcast ([11 x i8]* @.const.18 to i8*))
	%77 = bitcast [15 x i32*]* %heap_sort_test to i32**
	call void(i32**, i64) @"::print_i32_ptr_array"(i32** %77, i64 15)
	%78 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.19 to i8*))
	ret i32 0
}

@.const.10 = private unnamed_addr constant [10 x i8] c"push: %d\0A\00"
@.const.11 = private unnamed_addr constant [9 x i8] c"pop: %d\0A\00"
@.const.12 = private unnamed_addr constant [12 x i8] c"insert: %d\0A\00"
@.const.13 = private unnamed_addr constant [4 x i8] c"yes\00"
@.const.14 = private unnamed_addr constant [3 x i8] c"no\00"
@.const.15 = private unnamed_addr constant [17 x i8] c"contains %d: %s\0A\00"
@.const.16 = private unnamed_addr constant [11 x i8] c"unsorted: \00"
@.const.17 = private unnamed_addr constant [2 x i8] c"\0A\00"
@.const.18 = private unnamed_addr constant [11 x i8] c"heapsort: \00"
@.const.19 = private unnamed_addr constant [2 x i8] c"\0A\00"

