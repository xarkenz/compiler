source_filename = "\\\\?\\C:\\Users\\seane\\Projects\\compiler\\tests\\packages\\test_collections\\main.cupr"

%"::test_collections::BTree" = type { i64, i64, i32({}*, {}*)*, %"::test_collections::BTreeNode"* }

%"::test_collections::BTreeNodeKey" = type { {}*, %"::test_collections::BTreeNode"* }

%"::test_collections::LinkedList" = type { %"::test_collections::LinkedListNode"* }

%"::test_collections::AVLTree" = type { %"::test_collections::AVLTreeNode"*, i32({}*, {}*)* }

%"::test_collections::AVLTreeNode" = type { {}*, %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"*, i32 }

%"::test_collections::BTreeNode" = type { i1, i64, %"::test_collections::BTreeNodeKey"*, %"::test_collections::BTreeNode"* }

%"::test_collections::BTreeLeaf" = type { i1, i64, {}** }

%"::test_collections::LinkedListNode" = type { {}*, %"::test_collections::LinkedListNode"* }

declare i32 @printf(i8*, ...)

declare i8* @malloc(i64)

declare void @free(i8*)

declare i32 @llvm.smax.i32(i32, i32)

@.const.test_collections.0 = private unnamed_addr constant [3 x i8] c"%d\00"

@.const.test_collections.1 = private unnamed_addr constant [2 x i8] c"(\00"

@.const.test_collections.2 = private unnamed_addr constant [2 x i8] c" \00"

@.const.test_collections.3 = private unnamed_addr constant [2 x i8] c" \00"

@.const.test_collections.4 = private unnamed_addr constant [2 x i8] c")\00"

@.const.test_collections.5 = private unnamed_addr constant [2 x i8] c"\0A\00"

@.const.test_collections.6 = private unnamed_addr constant [3 x i8] c"[]\00"

@.const.test_collections.7 = private unnamed_addr constant [4 x i8] c"[%d\00"

@.const.test_collections.8 = private unnamed_addr constant [5 x i8] c", %d\00"

@.const.test_collections.9 = private unnamed_addr constant [2 x i8] c"]\00"

@.const.test_collections.10 = private unnamed_addr constant [10 x i8] c"push: %d\0A\00"

@.const.test_collections.11 = private unnamed_addr constant [9 x i8] c"pop: %d\0A\00"

@.const.test_collections.12 = private unnamed_addr constant [12 x i8] c"insert: %d\0A\00"

@.const.test_collections.13 = private unnamed_addr constant [4 x i8] c"yes\00"

@.const.test_collections.14 = private unnamed_addr constant [3 x i8] c"no\00"

@.const.test_collections.15 = private unnamed_addr constant [17 x i8] c"contains %d: %s\0A\00"

@.const.test_collections.16 = private unnamed_addr constant [11 x i8] c"unsorted: \00"

@.const.test_collections.17 = private unnamed_addr constant [2 x i8] c"\0A\00"

@.const.test_collections.18 = private unnamed_addr constant [11 x i8] c"heapsort: \00"

@.const.test_collections.19 = private unnamed_addr constant [2 x i8] c"\0A\00"

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
	%3 = call i32(i8*, ...) @printf(i8* bitcast ([3 x i8]* @.const.test_collections.0 to i8*), i32 %2)
	ret void
}

define %"::test_collections::LinkedList" @"::test_collections::LinkedList::new"() {
.block.0:
	ret %"::test_collections::LinkedList" { %"::test_collections::LinkedListNode"* null }
}

define {}* @"::test_collections::LinkedList::front"(%"::test_collections::LinkedList"* %0) {
.block.0:
	%self = alloca %"::test_collections::LinkedList"*
	store %"::test_collections::LinkedList"* %0, %"::test_collections::LinkedList"** %self
	%1 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%2 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %1, i32 0, i32 0
	%3 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %2
	%4 = icmp eq %"::test_collections::LinkedListNode"* %3, null
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	br label %.block.3
.block.2:
	%5 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%6 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %5, i32 0, i32 0
	%7 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %6
	%8 = getelementptr inbounds %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %7, i32 0, i32 0
	%9 = load {}*, {}** %8
	br label %.block.3
.block.3:
	%10 = phi {}* [ null, %.block.1 ], [ %9, %.block.2 ]
	ret {}* %10
}

define void @"::test_collections::LinkedList::push_front"(%"::test_collections::LinkedList"* %0, {}* %1) {
.block.0:
	%self = alloca %"::test_collections::LinkedList"*
	store %"::test_collections::LinkedList"* %0, %"::test_collections::LinkedList"** %self
	%value = alloca {}*
	store {}* %1, {}** %value
	%2 = call i8*(i64) @malloc(i64 16)
	%3 = bitcast i8* %2 to %"::test_collections::LinkedListNode"*
	%new_node = alloca %"::test_collections::LinkedListNode"*
	store %"::test_collections::LinkedListNode"* %3, %"::test_collections::LinkedListNode"** %new_node
	%4 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %new_node
	%5 = load {}*, {}** %value
	%6 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%7 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %6, i32 0, i32 0
	%8 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %7
	%9 = alloca %"::test_collections::LinkedListNode"
	%10 = getelementptr inbounds %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %9, i32 0, i32 0
	store {}* %5, {}** %10
	%11 = getelementptr inbounds %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %9, i32 0, i32 1
	store %"::test_collections::LinkedListNode"* %8, %"::test_collections::LinkedListNode"** %11
	%12 = load %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %9
	store %"::test_collections::LinkedListNode" %12, %"::test_collections::LinkedListNode"* %4
	%13 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%14 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %13, i32 0, i32 0
	%15 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %new_node
	store %"::test_collections::LinkedListNode"* %15, %"::test_collections::LinkedListNode"** %14
	ret void
}

define {}* @"::test_collections::LinkedList::pop_front"(%"::test_collections::LinkedList"* %0) {
.block.0:
	%self = alloca %"::test_collections::LinkedList"*
	store %"::test_collections::LinkedList"* %0, %"::test_collections::LinkedList"** %self
	%1 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%2 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %1, i32 0, i32 0
	%3 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %2
	%4 = icmp eq %"::test_collections::LinkedListNode"* %3, null
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	br label %.block.3
.block.2:
	%5 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%6 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %5, i32 0, i32 0
	%7 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %6
	%front = alloca %"::test_collections::LinkedListNode"*
	store %"::test_collections::LinkedListNode"* %7, %"::test_collections::LinkedListNode"** %front
	%8 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %front
	%9 = getelementptr inbounds %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %8, i32 0, i32 0
	%10 = load {}*, {}** %9
	%value = alloca {}*
	store {}* %10, {}** %value
	%11 = load %"::test_collections::LinkedList"*, %"::test_collections::LinkedList"** %self
	%12 = getelementptr inbounds %"::test_collections::LinkedList", %"::test_collections::LinkedList"* %11, i32 0, i32 0
	%13 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %front
	%14 = getelementptr inbounds %"::test_collections::LinkedListNode", %"::test_collections::LinkedListNode"* %13, i32 0, i32 1
	%15 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %14
	store %"::test_collections::LinkedListNode"* %15, %"::test_collections::LinkedListNode"** %12
	%16 = load %"::test_collections::LinkedListNode"*, %"::test_collections::LinkedListNode"** %front
	%17 = bitcast %"::test_collections::LinkedListNode"* %16 to i8*
	call void(i8*) @free(i8* %17)
	%18 = load {}*, {}** %value
	br label %.block.3
.block.3:
	%19 = phi {}* [ null, %.block.1 ], [ %18, %.block.2 ]
	ret {}* %19
}

define %"::test_collections::AVLTreeNode"* @"::test_collections::AVLTreeNode::alloc"({}* %0) {
.block.0:
	%key = alloca {}*
	store {}* %0, {}** %key
	%1 = call i8*(i64) @malloc(i64 32)
	%2 = bitcast i8* %1 to %"::test_collections::AVLTreeNode"*
	%alloc = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %2, %"::test_collections::AVLTreeNode"** %alloc
	%3 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %alloc
	%4 = load {}*, {}** %key
	%5 = alloca %"::test_collections::AVLTreeNode"
	store %"::test_collections::AVLTreeNode" { {}* undef, %"::test_collections::AVLTreeNode"* null, %"::test_collections::AVLTreeNode"* null, i32 0 }, %"::test_collections::AVLTreeNode"* %5
	%6 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %5, i32 0, i32 0
	store {}* %4, {}** %6
	%7 = load %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %5
	store %"::test_collections::AVLTreeNode" %7, %"::test_collections::AVLTreeNode"* %3
	%8 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %alloc
	ret %"::test_collections::AVLTreeNode"* %8
}

define i32 @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%1 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%2 = icmp eq %"::test_collections::AVLTreeNode"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = sub nsw i32 0, 1
	br label %.block.3
.block.2:
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %4, i32 0, i32 3
	%6 = load i32, i32* %5
	br label %.block.3
.block.3:
	%7 = phi i32 [ %3, %.block.1 ], [ %6, %.block.2 ]
	ret i32 %7
}

define void @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%1 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %1, i32 0, i32 3
	%3 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%4 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %3, i32 0, i32 1
	%5 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %4
	%6 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %5)
	%7 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%8 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %7, i32 0, i32 2
	%9 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %8
	%10 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %9)
	%11 = call i32(i32, i32) @llvm.smax.i32(i32 %6, i32 %10)
	%12 = add nsw i32 1, %11
	store i32 %12, i32* %2
	ret void
}

define %"::test_collections::AVLTreeNode"* @"::test_collections::AVLTreeNode::rotate_right"(%"::test_collections::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%1 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %1, i32 0, i32 1
	%3 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %2
	%new_root = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %3, %"::test_collections::AVLTreeNode"** %new_root
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %4, i32 0, i32 1
	%6 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	%7 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %6, i32 0, i32 2
	%8 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %7
	store %"::test_collections::AVLTreeNode"* %8, %"::test_collections::AVLTreeNode"** %5
	%9 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	%10 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %9, i32 0, i32 2
	%11 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	store %"::test_collections::AVLTreeNode"* %11, %"::test_collections::AVLTreeNode"** %10
	%12 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	call void(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %12)
	%13 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	call void(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %13)
	%14 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	ret %"::test_collections::AVLTreeNode"* %14
}

define %"::test_collections::AVLTreeNode"* @"::test_collections::AVLTreeNode::rotate_left"(%"::test_collections::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%1 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%2 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %1, i32 0, i32 2
	%3 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %2
	%new_root = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %3, %"::test_collections::AVLTreeNode"** %new_root
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%5 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %4, i32 0, i32 2
	%6 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	%7 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %6, i32 0, i32 1
	%8 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %7
	store %"::test_collections::AVLTreeNode"* %8, %"::test_collections::AVLTreeNode"** %5
	%9 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	%10 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %9, i32 0, i32 1
	%11 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	store %"::test_collections::AVLTreeNode"* %11, %"::test_collections::AVLTreeNode"** %10
	%12 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	call void(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %12)
	%13 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	call void(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %13)
	%14 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %new_root
	ret %"::test_collections::AVLTreeNode"* %14
}

define %"::test_collections::AVLTreeNode"* @"::test_collections::AVLTreeNode::balance"(%"::test_collections::AVLTreeNode"* %0) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%1 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%2 = icmp eq %"::test_collections::AVLTreeNode"* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	ret %"::test_collections::AVLTreeNode"* null
.block.2:
	%3 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%4 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %3, i32 0, i32 1
	%5 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %4
	%6 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %5)
	%7 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%8 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %7, i32 0, i32 2
	%9 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %8
	%10 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %9)
	%11 = sub nsw i32 %6, %10
	%imbalance = alloca i32
	store i32 %11, i32* %imbalance
	%12 = load i32, i32* %imbalance
	%13 = icmp sgt i32 %12, 1
	br i1 %13, label %.block.3, label %.block.4
.block.3:
	%14 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%15 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %14, i32 0, i32 1
	%16 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %15
	%17 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %16, i32 0, i32 2
	%18 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %17
	%19 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %18)
	%20 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%21 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %20, i32 0, i32 1
	%22 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %21
	%23 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %22, i32 0, i32 1
	%24 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %23
	%25 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %24)
	%26 = icmp sgt i32 %19, %25
	br i1 %26, label %.block.5, label %.block.6
.block.5:
	%27 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%28 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %27, i32 0, i32 1
	%29 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%30 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %29, i32 0, i32 1
	%31 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %30
	%32 = call %"::test_collections::AVLTreeNode"*(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::rotate_left"(%"::test_collections::AVLTreeNode"* %31)
	store %"::test_collections::AVLTreeNode"* %32, %"::test_collections::AVLTreeNode"** %28
	br label %.block.6
.block.6:
	%33 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%34 = call %"::test_collections::AVLTreeNode"*(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::rotate_right"(%"::test_collections::AVLTreeNode"* %33)
	br label %.block.7
.block.4:
	%35 = load i32, i32* %imbalance
	%36 = sub nsw i32 0, 1
	%37 = icmp slt i32 %35, %36
	br i1 %37, label %.block.8, label %.block.9
.block.8:
	%38 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%39 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %38, i32 0, i32 2
	%40 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %39
	%41 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %40, i32 0, i32 1
	%42 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %41
	%43 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %42)
	%44 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%45 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %44, i32 0, i32 2
	%46 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %45
	%47 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %46, i32 0, i32 2
	%48 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %47
	%49 = call i32(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::get_height"(%"::test_collections::AVLTreeNode"* %48)
	%50 = icmp sgt i32 %43, %49
	br i1 %50, label %.block.10, label %.block.11
.block.10:
	%51 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%52 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %51, i32 0, i32 2
	%53 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%54 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %53, i32 0, i32 2
	%55 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %54
	%56 = call %"::test_collections::AVLTreeNode"*(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::rotate_right"(%"::test_collections::AVLTreeNode"* %55)
	store %"::test_collections::AVLTreeNode"* %56, %"::test_collections::AVLTreeNode"** %52
	br label %.block.11
.block.11:
	%57 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%58 = call %"::test_collections::AVLTreeNode"*(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::rotate_left"(%"::test_collections::AVLTreeNode"* %57)
	br label %.block.12
.block.9:
	%59 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	call void(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::recompute_height"(%"::test_collections::AVLTreeNode"* %59)
	%60 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	br label %.block.12
.block.12:
	%61 = phi %"::test_collections::AVLTreeNode"* [ %58, %.block.11 ], [ %60, %.block.9 ]
	br label %.block.7
.block.7:
	%62 = phi %"::test_collections::AVLTreeNode"* [ %34, %.block.6 ], [ %61, %.block.12 ]
	ret %"::test_collections::AVLTreeNode"* %62
}

define void @"::test_collections::AVLTreeNode::print"(%"::test_collections::AVLTreeNode"* %0, void({}*)* %1) {
.block.0:
	%self = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %0, %"::test_collections::AVLTreeNode"** %self
	%printer = alloca void({}*)*
	store void({}*)* %1, void({}*)** %printer
	%2 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%3 = icmp ne %"::test_collections::AVLTreeNode"* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	%4 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.1 to i8*))
	%5 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%6 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %5, i32 0, i32 1
	%7 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %6
	%8 = load void({}*)*, void({}*)** %printer
	call void(%"::test_collections::AVLTreeNode"*, void({}*)*) @"::test_collections::AVLTreeNode::print"(%"::test_collections::AVLTreeNode"* %7, void({}*)* %8)
	%9 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.2 to i8*))
	%10 = load void({}*)*, void({}*)** %printer
	%11 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%12 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %11, i32 0, i32 0
	%13 = load {}*, {}** %12
	call void({}*) %10({}* %13)
	%14 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.3 to i8*))
	%15 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %self
	%16 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %15, i32 0, i32 2
	%17 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %16
	%18 = load void({}*)*, void({}*)** %printer
	call void(%"::test_collections::AVLTreeNode"*, void({}*)*) @"::test_collections::AVLTreeNode::print"(%"::test_collections::AVLTreeNode"* %17, void({}*)* %18)
	%19 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.4 to i8*))
	br label %.block.2
.block.2:
	ret void
}

define %"::test_collections::AVLTree" @"::test_collections::AVLTree::new"(i32({}*, {}*)* %0) {
.block.0:
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %0, i32({}*, {}*)** %comparator
	%1 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%2 = alloca %"::test_collections::AVLTree"
	store %"::test_collections::AVLTree" { %"::test_collections::AVLTreeNode"* null, i32({}*, {}*)* undef }, %"::test_collections::AVLTree"* %2
	%3 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %2, i32 0, i32 1
	store i32({}*, {}*)* %1, i32({}*, {}*)** %3
	%4 = load %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %2
	ret %"::test_collections::AVLTree" %4
}

define {}* @"::test_collections::AVLTree::get"(%"::test_collections::AVLTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"::test_collections::AVLTree"*
	store %"::test_collections::AVLTree"* %0, %"::test_collections::AVLTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%3 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %2, i32 0, i32 0
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %3
	%node = alloca %"::test_collections::AVLTreeNode"*
	store %"::test_collections::AVLTreeNode"* %4, %"::test_collections::AVLTreeNode"** %node
	br label %.block.1
.block.1:
	%5 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %node
	%6 = icmp ne %"::test_collections::AVLTreeNode"* %5, null
	br i1 %6, label %.block.2, label %.block.3
.block.2:
	%7 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%8 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %7, i32 0, i32 1
	%9 = load i32({}*, {}*)*, i32({}*, {}*)** %8
	%10 = load {}*, {}** %key
	%11 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %node
	%12 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %11, i32 0, i32 0
	%13 = load {}*, {}** %12
	%14 = call i32({}*, {}*) %9({}* %10, {}* %13)
	%ordering = alloca i32
	store i32 %14, i32* %ordering
	%15 = load i32, i32* %ordering
	%16 = icmp slt i32 %15, 0
	br i1 %16, label %.block.4, label %.block.5
.block.4:
	%17 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %node
	%18 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %17, i32 0, i32 1
	%19 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %18
	store %"::test_collections::AVLTreeNode"* %19, %"::test_collections::AVLTreeNode"** %node
	br label %.block.6
.block.5:
	%20 = load i32, i32* %ordering
	%21 = icmp sgt i32 %20, 0
	br i1 %21, label %.block.7, label %.block.8
.block.7:
	%22 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %node
	%23 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %22, i32 0, i32 2
	%24 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %23
	store %"::test_collections::AVLTreeNode"* %24, %"::test_collections::AVLTreeNode"** %node
	br label %.block.9
.block.8:
	%25 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %node
	%26 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %25, i32 0, i32 0
	%27 = load {}*, {}** %26
	ret {}* %27
.block.9:
	br label %.block.6
.block.6:
	br label %.block.1
.block.3:
	ret {}* null
}

define {}* @"::test_collections::AVLTree::insert_subtree"(%"::test_collections::AVLTree"* %0, %"::test_collections::AVLTreeNode"** %1, {}* %2) {
.block.0:
	%self = alloca %"::test_collections::AVLTree"*
	store %"::test_collections::AVLTree"* %0, %"::test_collections::AVLTree"** %self
	%node_ref = alloca %"::test_collections::AVLTreeNode"**
	store %"::test_collections::AVLTreeNode"** %1, %"::test_collections::AVLTreeNode"*** %node_ref
	%key = alloca {}*
	store {}* %2, {}** %key
	%3 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %3
	%5 = icmp eq %"::test_collections::AVLTreeNode"* %4, null
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%7 = load {}*, {}** %key
	%8 = call %"::test_collections::AVLTreeNode"*({}*) @"::test_collections::AVLTreeNode::alloc"({}* %7)
	store %"::test_collections::AVLTreeNode"* %8, %"::test_collections::AVLTreeNode"** %6
	ret {}* null
.block.2:
	%9 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%10 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %9, i32 0, i32 1
	%11 = load i32({}*, {}*)*, i32({}*, {}*)** %10
	%12 = load {}*, {}** %key
	%13 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%14 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %13
	%15 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %14, i32 0, i32 0
	%16 = load {}*, {}** %15
	%17 = call i32({}*, {}*) %11({}* %12, {}* %16)
	%ordering = alloca i32
	store i32 %17, i32* %ordering
	%18 = load i32, i32* %ordering
	%19 = icmp slt i32 %18, 0
	br i1 %19, label %.block.3, label %.block.4
.block.3:
	%20 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%21 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%22 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %21
	%23 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %22, i32 0, i32 1
	%24 = load {}*, {}** %key
	%25 = call {}*(%"::test_collections::AVLTree"*, %"::test_collections::AVLTreeNode"**, {}*) @"::test_collections::AVLTree::insert_subtree"(%"::test_collections::AVLTree"* %20, %"::test_collections::AVLTreeNode"** %23, {}* %24)
	br label %.block.5
.block.4:
	%26 = load i32, i32* %ordering
	%27 = icmp sgt i32 %26, 0
	br i1 %27, label %.block.6, label %.block.7
.block.6:
	%28 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%29 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%30 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %29
	%31 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %30, i32 0, i32 2
	%32 = load {}*, {}** %key
	%33 = call {}*(%"::test_collections::AVLTree"*, %"::test_collections::AVLTreeNode"**, {}*) @"::test_collections::AVLTree::insert_subtree"(%"::test_collections::AVLTree"* %28, %"::test_collections::AVLTreeNode"** %31, {}* %32)
	br label %.block.8
.block.7:
	%34 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%35 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %34
	%36 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %35, i32 0, i32 0
	%37 = load {}*, {}** %36
	%replaced_key = alloca {}*
	store {}* %37, {}** %replaced_key
	%38 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%39 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %38
	%40 = getelementptr inbounds %"::test_collections::AVLTreeNode", %"::test_collections::AVLTreeNode"* %39, i32 0, i32 0
	%41 = load {}*, {}** %key
	store {}* %41, {}** %40
	%42 = load {}*, {}** %replaced_key
	ret {}* %42
.block.8:
	br label %.block.5
.block.5:
	%43 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%44 = load %"::test_collections::AVLTreeNode"**, %"::test_collections::AVLTreeNode"*** %node_ref
	%45 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %44
	%46 = call %"::test_collections::AVLTreeNode"*(%"::test_collections::AVLTreeNode"*) @"::test_collections::AVLTreeNode::balance"(%"::test_collections::AVLTreeNode"* %45)
	store %"::test_collections::AVLTreeNode"* %46, %"::test_collections::AVLTreeNode"** %43
	ret {}* null
}

define {}* @"::test_collections::AVLTree::insert"(%"::test_collections::AVLTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"::test_collections::AVLTree"*
	store %"::test_collections::AVLTree"* %0, %"::test_collections::AVLTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%3 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%4 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %3, i32 0, i32 0
	%5 = load {}*, {}** %key
	%6 = call {}*(%"::test_collections::AVLTree"*, %"::test_collections::AVLTreeNode"**, {}*) @"::test_collections::AVLTree::insert_subtree"(%"::test_collections::AVLTree"* %2, %"::test_collections::AVLTreeNode"** %4, {}* %5)
	ret {}* %6
}

define void @"::test_collections::AVLTree::print"(%"::test_collections::AVLTree"* %0, void({}*)* %1) {
.block.0:
	%self = alloca %"::test_collections::AVLTree"*
	store %"::test_collections::AVLTree"* %0, %"::test_collections::AVLTree"** %self
	%printer = alloca void({}*)*
	store void({}*)* %1, void({}*)** %printer
	%2 = load %"::test_collections::AVLTree"*, %"::test_collections::AVLTree"** %self
	%3 = getelementptr inbounds %"::test_collections::AVLTree", %"::test_collections::AVLTree"* %2, i32 0, i32 0
	%4 = load %"::test_collections::AVLTreeNode"*, %"::test_collections::AVLTreeNode"** %3
	%5 = load void({}*)*, void({}*)** %printer
	call void(%"::test_collections::AVLTreeNode"*, void({}*)*) @"::test_collections::AVLTreeNode::print"(%"::test_collections::AVLTreeNode"* %4, void({}*)* %5)
	%6 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.5 to i8*))
	ret void
}

define %"::test_collections::BTreeLeaf"* @"::test_collections::BTreeLeaf::alloc"(i64 %0, {}* %1) {
.block.0:
	%l_order = alloca i64
	store i64 %0, i64* %l_order
	%first_element = alloca {}*
	store {}* %1, {}** %first_element
	%2 = load i64, i64* %l_order
	%3 = mul nuw i64 8, %2
	%4 = call i8*(i64) @malloc(i64 %3)
	%5 = bitcast i8* %4 to {}**
	%elements = alloca {}**
	store {}** %5, {}*** %elements
	%6 = load {}**, {}*** %elements
	%7 = getelementptr inbounds {}*, {}** %6, i32 0
	%8 = load {}*, {}** %first_element
	store {}* %8, {}** %7
	%9 = call i8*(i64) @malloc(i64 24)
	%10 = bitcast i8* %9 to %"::test_collections::BTreeLeaf"*
	%alloc = alloca %"::test_collections::BTreeLeaf"*
	store %"::test_collections::BTreeLeaf"* %10, %"::test_collections::BTreeLeaf"** %alloc
	%11 = load %"::test_collections::BTreeLeaf"*, %"::test_collections::BTreeLeaf"** %alloc
	%12 = load {}**, {}*** %elements
	%13 = alloca %"::test_collections::BTreeLeaf"
	store %"::test_collections::BTreeLeaf" { i1 true, i64 1, {}** undef }, %"::test_collections::BTreeLeaf"* %13
	%14 = getelementptr inbounds %"::test_collections::BTreeLeaf", %"::test_collections::BTreeLeaf"* %13, i32 0, i32 2
	store {}** %12, {}*** %14
	%15 = load %"::test_collections::BTreeLeaf", %"::test_collections::BTreeLeaf"* %13
	store %"::test_collections::BTreeLeaf" %15, %"::test_collections::BTreeLeaf"* %11
	%16 = load %"::test_collections::BTreeLeaf"*, %"::test_collections::BTreeLeaf"** %alloc
	ret %"::test_collections::BTreeLeaf"* %16
}

define %"::test_collections::BTree" @"::test_collections::BTree::new"(i64 %0, i64 %1, i32({}*, {}*)* %2) {
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
	%6 = alloca %"::test_collections::BTree"
	store %"::test_collections::BTree" { i64 undef, i64 undef, i32({}*, {}*)* undef, %"::test_collections::BTreeNode"* null }, %"::test_collections::BTree"* %6
	%7 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %6, i32 0, i32 0
	store i64 %3, i64* %7
	%8 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %6, i32 0, i32 1
	store i64 %4, i64* %8
	%9 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %6, i32 0, i32 2
	store i32({}*, {}*)* %5, i32({}*, {}*)** %9
	%10 = load %"::test_collections::BTree", %"::test_collections::BTree"* %6
	ret %"::test_collections::BTree" %10
}

define {}* @"::test_collections::BTree::insert"(%"::test_collections::BTree"* %0, {}* %1) {
.block.0:
	%self = alloca %"::test_collections::BTree"*
	store %"::test_collections::BTree"* %0, %"::test_collections::BTree"** %self
	%key = alloca {}*
	store {}* %1, {}** %key
	%2 = load %"::test_collections::BTree"*, %"::test_collections::BTree"** %self
	%3 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %2, i32 0, i32 3
	%4 = load %"::test_collections::BTreeNode"*, %"::test_collections::BTreeNode"** %3
	%5 = icmp eq %"::test_collections::BTreeNode"* %4, null
	br i1 %5, label %.block.1, label %.block.2
.block.1:
	%6 = load %"::test_collections::BTree"*, %"::test_collections::BTree"** %self
	%7 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %6, i32 0, i32 3
	%8 = load %"::test_collections::BTree"*, %"::test_collections::BTree"** %self
	%9 = getelementptr inbounds %"::test_collections::BTree", %"::test_collections::BTree"* %8, i32 0, i32 1
	%10 = load i64, i64* %9
	%11 = load {}*, {}** %key
	%12 = call %"::test_collections::BTreeLeaf"*(i64, {}*) @"::test_collections::BTreeLeaf::alloc"(i64 %10, {}* %11)
	%13 = bitcast %"::test_collections::BTreeLeaf"* %12 to %"::test_collections::BTreeNode"*
	store %"::test_collections::BTreeNode"* %13, %"::test_collections::BTreeNode"** %7
	ret {}* null
.block.2:
	ret {}* null
}

define void @"::test_collections::max_percolate_down"({}** %0, i64 %1, i32({}*, {}*)* %2, i64 %3) {
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

define void @"::test_collections::heap_sort"({}** %0, i64 %1, i32({}*, {}*)* %2) {
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
	call void({}**, i64, i32({}*, {}*)*, i64) @"::test_collections::max_percolate_down"({}** %9, i64 %10, i32({}*, {}*)* %11, i64 %12)
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
	call void({}**, i64, i32({}*, {}*)*, i64) @"::test_collections::max_percolate_down"({}** %31, i64 %32, i32({}*, {}*)* %33, i64 0)
	br label %.block.4
.block.6:
	ret void
}

define void @"::test_collections::print_i32_ptr_array"(i32** %0, i64 %1) {
.block.0:
	%array = alloca i32**
	store i32** %0, i32*** %array
	%length = alloca i64
	store i64 %1, i64* %length
	%2 = load i64, i64* %length
	%3 = icmp eq i64 %2, 0
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	%4 = call i32(i8*, ...) @printf(i8* bitcast ([3 x i8]* @.const.test_collections.6 to i8*))
	ret void
.block.2:
	%5 = load i32**, i32*** %array
	%6 = getelementptr inbounds i32*, i32** %5, i32 0
	%7 = load i32*, i32** %6
	%8 = load i32, i32* %7
	%9 = call i32(i8*, ...) @printf(i8* bitcast ([4 x i8]* @.const.test_collections.7 to i8*), i32 %8)
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
	%18 = call i32(i8*, ...) @printf(i8* bitcast ([5 x i8]* @.const.test_collections.8 to i8*), i32 %17)
	%19 = load i64, i64* %index
	%20 = add nuw i64 %19, 1
	store i64 %20, i64* %index
	br label %.block.3
.block.5:
	%21 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.9 to i8*))
	ret void
}

define i32 @main() {
.block.0:
	%keys = alloca [15 x i32]
	store [15 x i32] [ i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15 ], [15 x i32]* %keys
	%0 = call %"::test_collections::LinkedList"() @"::test_collections::LinkedList::new"()
	%list = alloca %"::test_collections::LinkedList"
	store %"::test_collections::LinkedList" %0, %"::test_collections::LinkedList"* %list
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
	%6 = call i32(i8*, ...) @printf(i8* bitcast ([10 x i8]* @.const.test_collections.10 to i8*), i32 %5)
	%7 = load i64, i64* %i
	%8 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %7
	%9 = bitcast i32* %8 to {}*
	call void(%"::test_collections::LinkedList"*, {}*) @"::test_collections::LinkedList::push_front"(%"::test_collections::LinkedList"* %list, {}* %9)
	%10 = load i64, i64* %i
	%11 = add nuw i64 %10, 1
	store i64 %11, i64* %i
	br label %.block.1
.block.3:
	%value = alloca i32*
	br label %.block.4
.block.4:
	%12 = call {}*(%"::test_collections::LinkedList"*) @"::test_collections::LinkedList::pop_front"(%"::test_collections::LinkedList"* %list)
	%13 = bitcast {}* %12 to i32*
	store i32* %13, i32** %value
	%14 = icmp ne i32* %13, null
	br i1 %14, label %.block.5, label %.block.6
.block.5:
	%15 = load i32*, i32** %value
	%16 = load i32, i32* %15
	%17 = call i32(i8*, ...) @printf(i8* bitcast ([9 x i8]* @.const.test_collections.11 to i8*), i32 %16)
	br label %.block.4
.block.6:
	%18 = call %"::test_collections::AVLTree"(i32({}*, {}*)*) @"::test_collections::AVLTree::new"(i32({}*, {}*)* bitcast (i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*))
	%tree = alloca %"::test_collections::AVLTree"
	store %"::test_collections::AVLTree" %18, %"::test_collections::AVLTree"* %tree
	%i-1 = alloca i64
	store i64 0, i64* %i-1
	br label %.block.7
.block.7:
	%19 = load i64, i64* %i-1
	%20 = icmp ult i64 %19, 7
	br i1 %20, label %.block.8, label %.block.9
.block.8:
	%21 = load i64, i64* %i-1
	%22 = mul nuw i64 %21, 7
	%23 = urem i64 %22, 10
	%idx = alloca i64
	store i64 %23, i64* %idx
	%24 = load i64, i64* %idx
	%25 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %24
	%26 = load i32, i32* %25
	%27 = call i32(i8*, ...) @printf(i8* bitcast ([12 x i8]* @.const.test_collections.12 to i8*), i32 %26)
	%28 = load i64, i64* %idx
	%29 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %28
	%30 = bitcast i32* %29 to {}*
	%31 = call {}*(%"::test_collections::AVLTree"*, {}*) @"::test_collections::AVLTree::insert"(%"::test_collections::AVLTree"* %tree, {}* %30)
	%32 = load i64, i64* %i-1
	%33 = add nuw i64 %32, 1
	store i64 %33, i64* %i-1
	br label %.block.7
.block.9:
	%i-2 = alloca i64
	store i64 0, i64* %i-2
	br label %.block.10
.block.10:
	%34 = load i64, i64* %i-2
	%35 = icmp ult i64 %34, 10
	br i1 %35, label %.block.11, label %.block.12
.block.11:
	%36 = load i64, i64* %i-2
	%37 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %36
	%38 = bitcast i32* %37 to {}*
	%39 = call {}*(%"::test_collections::AVLTree"*, {}*) @"::test_collections::AVLTree::get"(%"::test_collections::AVLTree"* %tree, {}* %38)
	%key = alloca {}*
	store {}* %39, {}** %key
	%40 = load {}*, {}** %key
	%41 = icmp ne {}* %40, null
	br i1 %41, label %.block.13, label %.block.14
.block.13:
	br label %.block.15
.block.14:
	br label %.block.15
.block.15:
	%42 = phi i8* [ bitcast ([4 x i8]* @.const.test_collections.13 to i8*), %.block.13 ], [ bitcast ([3 x i8]* @.const.test_collections.14 to i8*), %.block.14 ]
	%is_contained = alloca i8*
	store i8* %42, i8** %is_contained
	%43 = load i64, i64* %i-2
	%44 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %43
	%45 = load i32, i32* %44
	%46 = load i8*, i8** %is_contained
	%47 = call i32(i8*, ...) @printf(i8* bitcast ([17 x i8]* @.const.test_collections.15 to i8*), i32 %45, i8* %46)
	%48 = load i64, i64* %i-2
	%49 = add nuw i64 %48, 1
	store i64 %49, i64* %i-2
	br label %.block.10
.block.12:
	call void(%"::test_collections::AVLTree"*, void({}*)*) @"::test_collections::AVLTree::print"(%"::test_collections::AVLTree"* %tree, void({}*)* bitcast (void(i32*)* @"<i32>::print" to void({}*)*))
	%50 = call %"::test_collections::BTree"(i64, i64, i32({}*, {}*)*) @"::test_collections::BTree::new"(i64 3, i64 2, i32({}*, {}*)* bitcast (i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*))
	%b_tree = alloca %"::test_collections::BTree"
	store %"::test_collections::BTree" %50, %"::test_collections::BTree"* %b_tree
	%51 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i32 0
	%52 = bitcast i32* %51 to {}*
	%53 = call {}*(%"::test_collections::BTree"*, {}*) @"::test_collections::BTree::insert"(%"::test_collections::BTree"* %b_tree, {}* %52)
	%heap_sort_test = alloca [15 x i32*]
	%index = alloca i64
	store i64 0, i64* %index
	br label %.block.16
.block.16:
	%54 = load i64, i64* %index
	%55 = icmp ult i64 %54, 15
	br i1 %55, label %.block.17, label %.block.18
.block.17:
	%56 = load i64, i64* %index
	%57 = add nuw i64 %56, 7
	%58 = mul nuw i64 %57, 7
	%59 = urem i64 %58, 15
	%60 = getelementptr inbounds [15 x i32], [15 x i32]* %keys, i32 0, i64 %59
	%key-1 = alloca i32*
	store i32* %60, i32** %key-1
	%61 = load i64, i64* %index
	%62 = getelementptr inbounds [15 x i32*], [15 x i32*]* %heap_sort_test, i32 0, i64 %61
	%63 = load i32*, i32** %key-1
	store i32* %63, i32** %62
	%64 = load i64, i64* %index
	%65 = add nuw i64 %64, 1
	store i64 %65, i64* %index
	br label %.block.16
.block.18:
	%66 = call i32(i8*, ...) @printf(i8* bitcast ([11 x i8]* @.const.test_collections.16 to i8*))
	%67 = bitcast [15 x i32*]* %heap_sort_test to i32**
	call void(i32**, i64) @"::test_collections::print_i32_ptr_array"(i32** %67, i64 15)
	%68 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.17 to i8*))
	%69 = bitcast [15 x i32*]* %heap_sort_test to {}**
	call void({}**, i64, i32({}*, {}*)*) @"::test_collections::heap_sort"({}** %69, i64 15, i32({}*, {}*)* bitcast (i32(i32*, i32*)* @"<i32>::cmp" to i32({}*, {}*)*))
	%70 = call i32(i8*, ...) @printf(i8* bitcast ([11 x i8]* @.const.test_collections.18 to i8*))
	%71 = bitcast [15 x i32*]* %heap_sort_test to i32**
	call void(i32**, i64) @"::test_collections::print_i32_ptr_array"(i32** %71, i64 15)
	%72 = call i32(i8*, ...) @printf(i8* bitcast ([2 x i8]* @.const.test_collections.19 to i8*))
	ret i32 0
}

