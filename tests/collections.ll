; module_id = 0
source_filename = "./src/compiler_driver/collections.txt"

define dso_local i32 @"i32::max"(i32 noundef %.arg.self, i32 noundef %.arg.other) {
.block.0:
	%self = alloca i32
	store i32 %.arg.self, i32* %self
	%other = alloca i32
	store i32 %.arg.other, i32* %other
	%0 = load i32, i32* %self
	%1 = load i32, i32* %other
	%2 = icmp sgt i32 %0, %1
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = load i32, i32* %self
	ret i32 %3
.block.2:
	%4 = load i32, i32* %other
	ret i32 %4
}

define dso_local i32 @"i32::cmp"(i32* noundef %.arg.self, i32* noundef %.arg.other) {
.block.0:
	%self = alloca i32*
	store i32* %.arg.self, i32** %self
	%other = alloca i32*
	store i32* %.arg.other, i32** %other
	%0 = load i32*, i32** %self
	%1 = load i32, i32* %0
	%2 = load i32*, i32** %other
	%3 = load i32, i32* %2
	%4 = icmp slt i32 %1, %3
	br i1 %4, label %.block.1, label %.block.2
.block.1:
	%5 = sub nsw i32 0, 1
	ret i32 %5
.block.2:
	%6 = load i32*, i32** %self
	%7 = load i32, i32* %6
	%8 = load i32*, i32** %other
	%9 = load i32, i32* %8
	%10 = icmp sgt i32 %7, %9
	br i1 %10, label %.block.3, label %.block.4
.block.3:
	ret i32 1
.block.4:
	ret i32 0
}

define dso_local void @"i32::print"(i32* noundef %.arg.self) {
.block.0:
	%self = alloca i32*
	store i32* %.arg.self, i32** %self
	%0 = load i32*, i32** %self
	%1 = load i32, i32* %0
	%2 = call i32(i8*, ...) @printf(i8* noundef bitcast ([3 x i8]* @.const.0 to i8*), i32 noundef %1)
	ret void
}

@.const.0 = private unnamed_addr constant [3 x i8] c"%d\00"

%type.LinkedListNode = type { {}*, %type.LinkedListNode* }

%type.LinkedList = type { %type.LinkedListNode* }

define dso_local %type.LinkedList @"LinkedList::new"() {
.block.0:
	ret %type.LinkedList { %type.LinkedListNode* null }
}

define dso_local {}* @"LinkedList::front"(%type.LinkedList* noundef %.arg.self) {
.block.0:
	%self = alloca %type.LinkedList*
	store %type.LinkedList* %.arg.self, %type.LinkedList** %self
	%0 = load %type.LinkedList*, %type.LinkedList** %self
	%1 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %0, i32 0, i32 0
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %1
	%3 = icmp eq %type.LinkedListNode* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	ret {}* null
.block.2:
	%4 = load %type.LinkedList*, %type.LinkedList** %self
	%5 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	%7 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %6, i32 0, i32 0
	%8 = load {}*, {}** %7
	ret {}* %8
}

define dso_local void @"LinkedList::push_front"(%type.LinkedList* noundef %.arg.self, {}* noundef %.arg.value) {
.block.0:
	%self = alloca %type.LinkedList*
	store %type.LinkedList* %.arg.self, %type.LinkedList** %self
	%value = alloca {}*
	store {}* %.arg.value, {}** %value
	%new_node = alloca %type.LinkedListNode*
	%0 = call {}*(i64) @malloc(i64 noundef 16)
	%1 = bitcast {}* %0 to %type.LinkedListNode*
	store %type.LinkedListNode* %1, %type.LinkedListNode** %new_node
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %new_node
	%3 = load {}*, {}** %value
	%4 = load %type.LinkedList*, %type.LinkedList** %self
	%5 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	%7 = alloca %type.LinkedListNode
	store %type.LinkedListNode { {}* undef, %type.LinkedListNode* undef }, %type.LinkedListNode* %7
	%8 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 0
	store {}* %3, {}** %8
	%9 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 1
	store %type.LinkedListNode* %6, %type.LinkedListNode** %9
	%10 = load %type.LinkedListNode, %type.LinkedListNode* %7
	store %type.LinkedListNode %10, %type.LinkedListNode* %2
	%11 = load %type.LinkedList*, %type.LinkedList** %self
	%12 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %11, i32 0, i32 0
	%13 = load %type.LinkedListNode*, %type.LinkedListNode** %new_node
	store %type.LinkedListNode* %13, %type.LinkedListNode** %12
	ret void
}

define dso_local {}* @"LinkedList::pop_front"(%type.LinkedList* noundef %.arg.self) {
.block.0:
	%self = alloca %type.LinkedList*
	store %type.LinkedList* %.arg.self, %type.LinkedList** %self
	%0 = load %type.LinkedList*, %type.LinkedList** %self
	%1 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %0, i32 0, i32 0
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %1
	%3 = icmp eq %type.LinkedListNode* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	ret {}* null
.block.2:
	%front = alloca %type.LinkedListNode*
	%4 = load %type.LinkedList*, %type.LinkedList** %self
	%5 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	store %type.LinkedListNode* %6, %type.LinkedListNode** %front
	%value = alloca {}*
	%7 = load %type.LinkedListNode*, %type.LinkedListNode** %front
	%8 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 0
	%9 = load {}*, {}** %8
	store {}* %9, {}** %value
	%10 = load %type.LinkedList*, %type.LinkedList** %self
	%11 = getelementptr inbounds %type.LinkedList, %type.LinkedList* %10, i32 0, i32 0
	%12 = load %type.LinkedListNode*, %type.LinkedListNode** %front
	%13 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %12, i32 0, i32 1
	%14 = load %type.LinkedListNode*, %type.LinkedListNode** %13
	store %type.LinkedListNode* %14, %type.LinkedListNode** %11
	%15 = load %type.LinkedListNode*, %type.LinkedListNode** %front
	%16 = bitcast %type.LinkedListNode* %15 to {}*
	call void({}*) @free({}* noundef %16)
	%17 = load {}*, {}** %value
	ret {}* %17
}

%type.AVLTreeNode = type { {}*, %type.AVLTreeNode*, %type.AVLTreeNode*, i32 }

define dso_local %type.AVLTreeNode* @"AVLTreeNode::alloc"({}* noundef %.arg.key) {
.block.0:
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%alloc = alloca %type.AVLTreeNode*
	%0 = call {}*(i64) @malloc(i64 noundef 32)
	%1 = bitcast {}* %0 to %type.AVLTreeNode*
	store %type.AVLTreeNode* %1, %type.AVLTreeNode** %alloc
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %alloc
	%3 = load {}*, {}** %key
	%4 = alloca %type.AVLTreeNode
	store %type.AVLTreeNode { {}* undef, %type.AVLTreeNode* null, %type.AVLTreeNode* null, i32 0 }, %type.AVLTreeNode* %4
	%5 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %4, i32 0, i32 0
	store {}* %3, {}** %5
	%6 = load %type.AVLTreeNode, %type.AVLTreeNode* %4
	store %type.AVLTreeNode %6, %type.AVLTreeNode* %2
	%7 = load %type.AVLTreeNode*, %type.AVLTreeNode** %alloc
	ret %type.AVLTreeNode* %7
}

define dso_local i32 @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %.arg.self) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = icmp eq %type.AVLTreeNode* %0, null
	br i1 %1, label %.block.1, label %.block.2
.block.1:
	%2 = sub nsw i32 0, 1
	ret i32 %2
.block.2:
	%3 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%4 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %3, i32 0, i32 3
	%5 = load i32, i32* %4
	ret i32 %5
}

define dso_local void @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %.arg.self) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %0, i32 0, i32 3
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%3 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %2, i32 0, i32 1
	%4 = load %type.AVLTreeNode*, %type.AVLTreeNode** %3
	%5 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %4)
	%6 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%7 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %6, i32 0, i32 2
	%8 = load %type.AVLTreeNode*, %type.AVLTreeNode** %7
	%9 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %8)
	%10 = call i32(i32, i32) @"i32::max"(i32 noundef %5, i32 noundef %9)
	%11 = add nsw i32 1, %10
	store i32 %11, i32* %1
	ret void
}

define dso_local %type.AVLTreeNode* @"AVLTreeNode::rotate_right"(%type.AVLTreeNode* noundef %.arg.self) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%new_root = alloca %type.AVLTreeNode*
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %0, i32 0, i32 1
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %1
	store %type.AVLTreeNode* %2, %type.AVLTreeNode** %new_root
	%3 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%4 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %3, i32 0, i32 1
	%5 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	%6 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %5, i32 0, i32 2
	%7 = load %type.AVLTreeNode*, %type.AVLTreeNode** %6
	store %type.AVLTreeNode* %7, %type.AVLTreeNode** %4
	%8 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	%9 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %8, i32 0, i32 2
	%10 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	store %type.AVLTreeNode* %10, %type.AVLTreeNode** %9
	%11 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	call void(%type.AVLTreeNode*) @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %11)
	%12 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	call void(%type.AVLTreeNode*) @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %12)
	%13 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	ret %type.AVLTreeNode* %13
}

define dso_local %type.AVLTreeNode* @"AVLTreeNode::rotate_left"(%type.AVLTreeNode* noundef %.arg.self) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%new_root = alloca %type.AVLTreeNode*
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %0, i32 0, i32 2
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %1
	store %type.AVLTreeNode* %2, %type.AVLTreeNode** %new_root
	%3 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%4 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %3, i32 0, i32 2
	%5 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	%6 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %5, i32 0, i32 1
	%7 = load %type.AVLTreeNode*, %type.AVLTreeNode** %6
	store %type.AVLTreeNode* %7, %type.AVLTreeNode** %4
	%8 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	%9 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %8, i32 0, i32 1
	%10 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	store %type.AVLTreeNode* %10, %type.AVLTreeNode** %9
	%11 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	call void(%type.AVLTreeNode*) @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %11)
	%12 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	call void(%type.AVLTreeNode*) @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %12)
	%13 = load %type.AVLTreeNode*, %type.AVLTreeNode** %new_root
	ret %type.AVLTreeNode* %13
}

define dso_local %type.AVLTreeNode* @"AVLTreeNode::balance"(%type.AVLTreeNode* noundef %.arg.self) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = icmp eq %type.AVLTreeNode* %0, null
	br i1 %1, label %.block.1, label %.block.2
.block.1:
	ret %type.AVLTreeNode* null
.block.2:
	%imbalance = alloca i32
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%3 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %2, i32 0, i32 1
	%4 = load %type.AVLTreeNode*, %type.AVLTreeNode** %3
	%5 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %4)
	%6 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%7 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %6, i32 0, i32 2
	%8 = load %type.AVLTreeNode*, %type.AVLTreeNode** %7
	%9 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %8)
	%10 = sub nsw i32 %5, %9
	store i32 %10, i32* %imbalance
	%11 = load i32, i32* %imbalance
	%12 = icmp sgt i32 %11, 1
	br i1 %12, label %.block.3, label %.block.4
.block.3:
	%13 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%14 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %13, i32 0, i32 1
	%15 = load %type.AVLTreeNode*, %type.AVLTreeNode** %14
	%16 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %15, i32 0, i32 2
	%17 = load %type.AVLTreeNode*, %type.AVLTreeNode** %16
	%18 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %17)
	%19 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%20 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %19, i32 0, i32 1
	%21 = load %type.AVLTreeNode*, %type.AVLTreeNode** %20
	%22 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %21, i32 0, i32 1
	%23 = load %type.AVLTreeNode*, %type.AVLTreeNode** %22
	%24 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %23)
	%25 = icmp sgt i32 %18, %24
	br i1 %25, label %.block.5, label %.block.6
.block.5:
	%26 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%27 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %26, i32 0, i32 1
	%28 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%29 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %28, i32 0, i32 1
	%30 = load %type.AVLTreeNode*, %type.AVLTreeNode** %29
	%31 = bitcast %type.AVLTreeNode* %30 to %type.AVLTreeNode*
	%32 = call %type.AVLTreeNode*(%type.AVLTreeNode*) @"AVLTreeNode::rotate_left"(%type.AVLTreeNode* noundef %31)
	store %type.AVLTreeNode* %32, %type.AVLTreeNode** %27
	br label %.block.6
.block.6:
	%33 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%34 = bitcast %type.AVLTreeNode* %33 to %type.AVLTreeNode*
	%35 = call %type.AVLTreeNode*(%type.AVLTreeNode*) @"AVLTreeNode::rotate_right"(%type.AVLTreeNode* noundef %34)
	store %type.AVLTreeNode* %35, %type.AVLTreeNode** %self
	br label %.block.7
.block.4:
	%36 = load i32, i32* %imbalance
	%37 = sub nsw i32 0, 1
	%38 = icmp slt i32 %36, %37
	br i1 %38, label %.block.8, label %.block.9
.block.8:
	%39 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%40 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %39, i32 0, i32 2
	%41 = load %type.AVLTreeNode*, %type.AVLTreeNode** %40
	%42 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %41, i32 0, i32 1
	%43 = load %type.AVLTreeNode*, %type.AVLTreeNode** %42
	%44 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %43)
	%45 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%46 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %45, i32 0, i32 2
	%47 = load %type.AVLTreeNode*, %type.AVLTreeNode** %46
	%48 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %47, i32 0, i32 2
	%49 = load %type.AVLTreeNode*, %type.AVLTreeNode** %48
	%50 = call i32(%type.AVLTreeNode*) @"AVLTreeNode::get_height"(%type.AVLTreeNode* noundef %49)
	%51 = icmp sgt i32 %44, %50
	br i1 %51, label %.block.10, label %.block.11
.block.10:
	%52 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%53 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %52, i32 0, i32 2
	%54 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%55 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %54, i32 0, i32 2
	%56 = load %type.AVLTreeNode*, %type.AVLTreeNode** %55
	%57 = bitcast %type.AVLTreeNode* %56 to %type.AVLTreeNode*
	%58 = call %type.AVLTreeNode*(%type.AVLTreeNode*) @"AVLTreeNode::rotate_right"(%type.AVLTreeNode* noundef %57)
	store %type.AVLTreeNode* %58, %type.AVLTreeNode** %53
	br label %.block.11
.block.11:
	%59 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%60 = bitcast %type.AVLTreeNode* %59 to %type.AVLTreeNode*
	%61 = call %type.AVLTreeNode*(%type.AVLTreeNode*) @"AVLTreeNode::rotate_left"(%type.AVLTreeNode* noundef %60)
	store %type.AVLTreeNode* %61, %type.AVLTreeNode** %self
	br label %.block.12
.block.9:
	%62 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	call void(%type.AVLTreeNode*) @"AVLTreeNode::recompute_height"(%type.AVLTreeNode* noundef %62)
	br label %.block.12
.block.12:
	br label %.block.7
.block.7:
	%63 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	ret %type.AVLTreeNode* %63
}

define dso_local void @"AVLTreeNode::print"(%type.AVLTreeNode* noundef %.arg.self, void({}*)* noundef %.arg.printer) {
.block.0:
	%self = alloca %type.AVLTreeNode*
	store %type.AVLTreeNode* %.arg.self, %type.AVLTreeNode** %self
	%printer = alloca void({}*)*
	store void({}*)* %.arg.printer, void({}*)** %printer
	%0 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%1 = icmp eq %type.AVLTreeNode* %0, null
	br i1 %1, label %.block.1, label %.block.2
.block.1:
	%2 = call i32(i8*, ...) @printf(i8* noundef bitcast ([2 x i8]* @.const.1 to i8*))
	br label %.block.3
.block.2:
	%3 = call i32(i8*, ...) @printf(i8* noundef bitcast ([2 x i8]* @.const.2 to i8*))
	%4 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%5 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %4, i32 0, i32 1
	%6 = load %type.AVLTreeNode*, %type.AVLTreeNode** %5
	%7 = load void({}*)*, void({}*)** %printer
	call void(%type.AVLTreeNode*, void({}*)*) @"AVLTreeNode::print"(%type.AVLTreeNode* noundef %6, void({}*)* noundef %7)
	%8 = call i32(i8*, ...) @printf(i8* noundef bitcast ([3 x i8]* @.const.3 to i8*))
	%9 = load %type.AVLTreeNode*, %type.AVLTreeNode** %self
	%10 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %9, i32 0, i32 2
	%11 = load %type.AVLTreeNode*, %type.AVLTreeNode** %10
	%12 = load void({}*)*, void({}*)** %printer
	call void(%type.AVLTreeNode*, void({}*)*) @"AVLTreeNode::print"(%type.AVLTreeNode* noundef %11, void({}*)* noundef %12)
	%13 = call i32(i8*, ...) @printf(i8* noundef bitcast ([2 x i8]* @.const.4 to i8*))
	br label %.block.3
.block.3:
	ret void
}

@.const.1 = private unnamed_addr constant [2 x i8] c"_\00"
@.const.2 = private unnamed_addr constant [2 x i8] c"(\00"
@.const.3 = private unnamed_addr constant [3 x i8] c", \00"
@.const.4 = private unnamed_addr constant [2 x i8] c")\00"

%type.AVLTree = type { %type.AVLTreeNode*, i32({}*, {}*)* }

define dso_local %type.AVLTree @"AVLTree::new"(i32({}*, {}*)* noundef %.arg.comparator) {
.block.0:
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %.arg.comparator, i32({}*, {}*)** %comparator
	%0 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%1 = alloca %type.AVLTree
	store %type.AVLTree { %type.AVLTreeNode* null, i32({}*, {}*)* undef }, %type.AVLTree* %1
	%2 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %1, i32 0, i32 1
	store i32({}*, {}*)* %0, i32({}*, {}*)** %2
	%3 = load %type.AVLTree, %type.AVLTree* %1
	ret %type.AVLTree %3
}

define dso_local {}* @"AVLTree::get"(%type.AVLTree* noundef %.arg.self, {}* noundef %.arg.key) {
.block.0:
	%self = alloca %type.AVLTree*
	store %type.AVLTree* %.arg.self, %type.AVLTree** %self
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%node = alloca %type.AVLTreeNode*
	%0 = load %type.AVLTree*, %type.AVLTree** %self
	%1 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %0, i32 0, i32 0
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %1
	store %type.AVLTreeNode* %2, %type.AVLTreeNode** %node
	br label %.block.1
.block.1:
	%3 = load %type.AVLTreeNode*, %type.AVLTreeNode** %node
	%4 = icmp ne %type.AVLTreeNode* %3, null
	br i1 %4, label %.block.2, label %.block.3
.block.2:
	%ordering = alloca i32
	%5 = load %type.AVLTree*, %type.AVLTree** %self
	%6 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %5, i32 0, i32 1
	%7 = load i32({}*, {}*)*, i32({}*, {}*)** %6
	%8 = load {}*, {}** %key
	%9 = load %type.AVLTreeNode*, %type.AVLTreeNode** %node
	%10 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %9, i32 0, i32 0
	%11 = load {}*, {}** %10
	%12 = call i32({}*, {}*) %7({}* noundef %8, {}* noundef %11)
	store i32 %12, i32* %ordering
	%13 = load i32, i32* %ordering
	%14 = icmp slt i32 %13, 0
	br i1 %14, label %.block.4, label %.block.5
.block.4:
	%15 = load %type.AVLTreeNode*, %type.AVLTreeNode** %node
	%16 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %15, i32 0, i32 1
	%17 = load %type.AVLTreeNode*, %type.AVLTreeNode** %16
	store %type.AVLTreeNode* %17, %type.AVLTreeNode** %node
	br label %.block.6
.block.5:
	%18 = load i32, i32* %ordering
	%19 = icmp sgt i32 %18, 0
	br i1 %19, label %.block.7, label %.block.8
.block.7:
	%20 = load %type.AVLTreeNode*, %type.AVLTreeNode** %node
	%21 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %20, i32 0, i32 2
	%22 = load %type.AVLTreeNode*, %type.AVLTreeNode** %21
	store %type.AVLTreeNode* %22, %type.AVLTreeNode** %node
	br label %.block.9
.block.8:
	%23 = load %type.AVLTreeNode*, %type.AVLTreeNode** %node
	%24 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %23, i32 0, i32 0
	%25 = load {}*, {}** %24
	ret {}* %25
.block.9:
	br label %.block.6
.block.6:
	br label %.block.1
.block.3:
	ret {}* null
}

define dso_local {}* @"AVLTree::insert_subtree"(%type.AVLTree* noundef %.arg.self, %type.AVLTreeNode** noundef %.arg.node_ref, {}* noundef %.arg.key) {
.block.0:
	%self = alloca %type.AVLTree*
	store %type.AVLTree* %.arg.self, %type.AVLTree** %self
	%node_ref = alloca %type.AVLTreeNode**
	store %type.AVLTreeNode** %.arg.node_ref, %type.AVLTreeNode*** %node_ref
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%0 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%1 = load %type.AVLTreeNode*, %type.AVLTreeNode** %0
	%2 = icmp eq %type.AVLTreeNode* %1, null
	br i1 %2, label %.block.1, label %.block.2
.block.1:
	%3 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%4 = load {}*, {}** %key
	%5 = call %type.AVLTreeNode*({}*) @"AVLTreeNode::alloc"({}* noundef %4)
	store %type.AVLTreeNode* %5, %type.AVLTreeNode** %3
	ret {}* null
.block.2:
	%ordering = alloca i32
	%6 = load %type.AVLTree*, %type.AVLTree** %self
	%7 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %6, i32 0, i32 1
	%8 = load i32({}*, {}*)*, i32({}*, {}*)** %7
	%9 = load {}*, {}** %key
	%10 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%11 = load %type.AVLTreeNode*, %type.AVLTreeNode** %10
	%12 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %11, i32 0, i32 0
	%13 = load {}*, {}** %12
	%14 = call i32({}*, {}*) %8({}* noundef %9, {}* noundef %13)
	store i32 %14, i32* %ordering
	%15 = load i32, i32* %ordering
	%16 = icmp slt i32 %15, 0
	br i1 %16, label %.block.3, label %.block.4
.block.3:
	%17 = load %type.AVLTree*, %type.AVLTree** %self
	%18 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%19 = load %type.AVLTreeNode*, %type.AVLTreeNode** %18
	%20 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %19, i32 0, i32 1
	%21 = load {}*, {}** %key
	%22 = call {}*(%type.AVLTree*, %type.AVLTreeNode**, {}*) @"AVLTree::insert_subtree"(%type.AVLTree* noundef %17, %type.AVLTreeNode** noundef %20, {}* noundef %21)
	br label %.block.5
.block.4:
	%23 = load i32, i32* %ordering
	%24 = icmp sgt i32 %23, 0
	br i1 %24, label %.block.6, label %.block.7
.block.6:
	%25 = load %type.AVLTree*, %type.AVLTree** %self
	%26 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%27 = load %type.AVLTreeNode*, %type.AVLTreeNode** %26
	%28 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %27, i32 0, i32 2
	%29 = load {}*, {}** %key
	%30 = call {}*(%type.AVLTree*, %type.AVLTreeNode**, {}*) @"AVLTree::insert_subtree"(%type.AVLTree* noundef %25, %type.AVLTreeNode** noundef %28, {}* noundef %29)
	br label %.block.8
.block.7:
	%replaced_key = alloca {}*
	%31 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%32 = load %type.AVLTreeNode*, %type.AVLTreeNode** %31
	%33 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %32, i32 0, i32 0
	%34 = load {}*, {}** %33
	store {}* %34, {}** %replaced_key
	%35 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%36 = load %type.AVLTreeNode*, %type.AVLTreeNode** %35
	%37 = getelementptr inbounds %type.AVLTreeNode, %type.AVLTreeNode* %36, i32 0, i32 0
	%38 = load {}*, {}** %key
	store {}* %38, {}** %37
	%39 = load {}*, {}** %replaced_key
	ret {}* %39
.block.8:
	br label %.block.5
.block.5:
	%40 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%41 = load %type.AVLTreeNode**, %type.AVLTreeNode*** %node_ref
	%42 = load %type.AVLTreeNode*, %type.AVLTreeNode** %41
	%43 = bitcast %type.AVLTreeNode* %42 to %type.AVLTreeNode*
	%44 = call %type.AVLTreeNode*(%type.AVLTreeNode*) @"AVLTreeNode::balance"(%type.AVLTreeNode* noundef %43)
	store %type.AVLTreeNode* %44, %type.AVLTreeNode** %40
	ret {}* null
}

define dso_local {}* @"AVLTree::insert"(%type.AVLTree* noundef %.arg.self, {}* noundef %.arg.key) {
.block.0:
	%self = alloca %type.AVLTree*
	store %type.AVLTree* %.arg.self, %type.AVLTree** %self
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%0 = load %type.AVLTree*, %type.AVLTree** %self
	%1 = load %type.AVLTree*, %type.AVLTree** %self
	%2 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %1, i32 0, i32 0
	%3 = load {}*, {}** %key
	%4 = call {}*(%type.AVLTree*, %type.AVLTreeNode**, {}*) @"AVLTree::insert_subtree"(%type.AVLTree* noundef %0, %type.AVLTreeNode** noundef %2, {}* noundef %3)
	ret {}* %4
}

define dso_local void @"AVLTree::print"(%type.AVLTree* noundef %.arg.self, void({}*)* noundef %.arg.printer) {
.block.0:
	%self = alloca %type.AVLTree*
	store %type.AVLTree* %.arg.self, %type.AVLTree** %self
	%printer = alloca void({}*)*
	store void({}*)* %.arg.printer, void({}*)** %printer
	%0 = load %type.AVLTree*, %type.AVLTree** %self
	%1 = getelementptr inbounds %type.AVLTree, %type.AVLTree* %0, i32 0, i32 0
	%2 = load %type.AVLTreeNode*, %type.AVLTreeNode** %1
	%3 = load void({}*)*, void({}*)** %printer
	call void(%type.AVLTreeNode*, void({}*)*) @"AVLTreeNode::print"(%type.AVLTreeNode* noundef %2, void({}*)* noundef %3)
	%4 = call i32(i8*, ...) @printf(i8* noundef bitcast ([2 x i8]* @.const.5 to i8*))
	ret void
}

@.const.5 = private unnamed_addr constant [2 x i8] c"\0A\00"

%type.BTreeNodeKey = type { {}*, %type.BTreeNode* }

%type.BTreeNode = type { i1, i64, %type.BTreeNodeKey*, %type.BTreeNode* }

%type.BTreeLeaf = type { i1, i64, {}** }

define dso_local %type.BTreeLeaf* @"BTreeLeaf::alloc"(i64 noundef %.arg.l_order, {}* noundef %.arg.first_element) {
.block.0:
	%l_order = alloca i64
	store i64 %.arg.l_order, i64* %l_order
	%first_element = alloca {}*
	store {}* %.arg.first_element, {}** %first_element
	%elements = alloca {}**
	%0 = load i64, i64* %l_order
	%1 = mul nuw i64 8, %0
	%2 = call {}*(i64) @malloc(i64 noundef %1)
	%3 = bitcast {}* %2 to {}**
	store {}** %3, {}*** %elements
	%4 = load {}**, {}*** %elements
	%5 = getelementptr inbounds {}*, {}** %4, i32 0
	%6 = load {}*, {}** %first_element
	store {}* %6, {}** %5
	%alloc = alloca %type.BTreeLeaf*
	%7 = call {}*(i64) @malloc(i64 noundef 24)
	%8 = bitcast {}* %7 to %type.BTreeLeaf*
	store %type.BTreeLeaf* %8, %type.BTreeLeaf** %alloc
	%9 = load %type.BTreeLeaf*, %type.BTreeLeaf** %alloc
