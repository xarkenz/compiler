; module_id = 0
source_filename = "./src/compiler_driver/collections.txt"

%type.LinkedListNode = type { {}*, %type.LinkedListNode* }

%type.SinglyLinkedList = type { %type.LinkedListNode* }

define dso_local %type.SinglyLinkedList @"SinglyLinkedList::new"() {
.block.0:
	ret %type.SinglyLinkedList { %type.LinkedListNode* null }
}

define dso_local {}* @"SinglyLinkedList::front"(%type.SinglyLinkedList* noundef %.arg.self) {
.block.0:
	%self = alloca %type.SinglyLinkedList*
	store %type.SinglyLinkedList* %.arg.self, %type.SinglyLinkedList** %self
	%0 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%1 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %0, i32 0, i32 0
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %1
	%3 = icmp eq %type.LinkedListNode* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	ret {}* null
.block.2:
	%4 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%5 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	%7 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %6, i32 0, i32 0
	%8 = load {}*, {}** %7
	ret {}* %8
}

define dso_local void @"SinglyLinkedList::push_front"(%type.SinglyLinkedList* noundef %.arg.self, {}* noundef %.arg.value) {
.block.0:
	%self = alloca %type.SinglyLinkedList*
	store %type.SinglyLinkedList* %.arg.self, %type.SinglyLinkedList** %self
	%value = alloca {}*
	store {}* %.arg.value, {}** %value
	%new_node = alloca %type.LinkedListNode*
	%0 = call {}*(i64) @malloc(i64 noundef 16)
	%1 = bitcast {}* %0 to %type.LinkedListNode*
	store %type.LinkedListNode* %1, %type.LinkedListNode** %new_node
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %new_node
	%3 = load {}*, {}** %value
	%4 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%5 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	%7 = alloca %type.LinkedListNode
	store %type.LinkedListNode { {}* undef, %type.LinkedListNode* undef }, %type.LinkedListNode* %7
	%8 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 0
	store {}* %3, {}** %8
	%9 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 1
	store %type.LinkedListNode* %6, %type.LinkedListNode** %9
	%10 = load %type.LinkedListNode, %type.LinkedListNode* %7
	store %type.LinkedListNode %10, %type.LinkedListNode* %2
	%11 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%12 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %11, i32 0, i32 0
	%13 = load %type.LinkedListNode*, %type.LinkedListNode** %new_node
	store %type.LinkedListNode* %13, %type.LinkedListNode** %12
	ret void
}

define dso_local {}* @"SinglyLinkedList::pop_front"(%type.SinglyLinkedList* noundef %.arg.self) {
.block.0:
	%self = alloca %type.SinglyLinkedList*
	store %type.SinglyLinkedList* %.arg.self, %type.SinglyLinkedList** %self
	%0 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%1 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %0, i32 0, i32 0
	%2 = load %type.LinkedListNode*, %type.LinkedListNode** %1
	%3 = icmp eq %type.LinkedListNode* %2, null
	br i1 %3, label %.block.1, label %.block.2
.block.1:
	ret {}* null
.block.2:
	%front = alloca %type.LinkedListNode*
	%4 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%5 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %4, i32 0, i32 0
	%6 = load %type.LinkedListNode*, %type.LinkedListNode** %5
	store %type.LinkedListNode* %6, %type.LinkedListNode** %front
	%value = alloca {}*
	%7 = load %type.LinkedListNode*, %type.LinkedListNode** %front
	%8 = getelementptr inbounds %type.LinkedListNode, %type.LinkedListNode* %7, i32 0, i32 0
	%9 = load {}*, {}** %8
	store {}* %9, {}** %value
	%10 = load %type.SinglyLinkedList*, %type.SinglyLinkedList** %self
	%11 = getelementptr inbounds %type.SinglyLinkedList, %type.SinglyLinkedList* %10, i32 0, i32 0
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

%type.BinaryTreeNode = type { {}*, %type.BinaryTreeNode*, %type.BinaryTreeNode* }

%type.BinaryTree = type { %type.BinaryTreeNode*, i32({}*, {}*)* }

define dso_local %type.BinaryTree @"BinaryTree::new"(i32({}*, {}*)* noundef %.arg.comparator) {
.block.0:
	%comparator = alloca i32({}*, {}*)*
	store i32({}*, {}*)* %.arg.comparator, i32({}*, {}*)** %comparator
	%0 = load i32({}*, {}*)*, i32({}*, {}*)** %comparator
	%1 = alloca %type.BinaryTree
	store %type.BinaryTree { %type.BinaryTreeNode* null, i32({}*, {}*)* undef }, %type.BinaryTree* %1
	%2 = getelementptr inbounds %type.BinaryTree, %type.BinaryTree* %1, i32 0, i32 1
	store i32({}*, {}*)* %0, i32({}*, {}*)** %2
	%3 = load %type.BinaryTree, %type.BinaryTree* %1
	ret %type.BinaryTree %3
}

define dso_local {}* @"BinaryTree::insert"(%type.BinaryTree* noundef %.arg.self, {}* noundef %.arg.key) {
.block.0:
	%self = alloca %type.BinaryTree*
	store %type.BinaryTree* %.arg.self, %type.BinaryTree** %self
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%current_ptr = alloca %type.BinaryTreeNode**
	%0 = load %type.BinaryTree*, %type.BinaryTree** %self
	%1 = getelementptr inbounds %type.BinaryTree, %type.BinaryTree* %0, i32 0, i32 0
	store %type.BinaryTreeNode** %1, %type.BinaryTreeNode*** %current_ptr
	br label %.block.1
.block.1:
	%2 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%3 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %2
	%4 = icmp ne %type.BinaryTreeNode* %3, null
	br i1 %4, label %.block.2, label %.block.3
.block.2:
	%ordering = alloca i32
	%5 = load %type.BinaryTree*, %type.BinaryTree** %self
	%6 = getelementptr inbounds %type.BinaryTree, %type.BinaryTree* %5, i32 0, i32 1
	%7 = load i32({}*, {}*)*, i32({}*, {}*)** %6
	%8 = load {}*, {}** %key
	%9 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%10 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %9
	%11 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %10, i32 0, i32 0
	%12 = load {}*, {}** %11
	%13 = call i32({}*, {}*) %7({}* noundef %8, {}* noundef %12)
	store i32 %13, i32* %ordering
	%14 = load i32, i32* %ordering
	%15 = icmp slt i32 %14, 0
	br i1 %15, label %.block.4, label %.block.5
.block.4:
	%16 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%17 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %16
	%18 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %17, i32 0, i32 1
	store %type.BinaryTreeNode** %18, %type.BinaryTreeNode*** %current_ptr
	br label %.block.6
.block.5:
	%19 = load i32, i32* %ordering
	%20 = icmp sgt i32 %19, 0
	br i1 %20, label %.block.7, label %.block.8
.block.7:
	%21 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%22 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %21
	%23 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %22, i32 0, i32 2
	store %type.BinaryTreeNode** %23, %type.BinaryTreeNode*** %current_ptr
	br label %.block.9
.block.8:
	%replaced_key = alloca {}*
	%24 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%25 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %24
	%26 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %25, i32 0, i32 0
	%27 = load {}*, {}** %26
	store {}* %27, {}** %replaced_key
	%28 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%29 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %28
	%30 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %29, i32 0, i32 0
	%31 = load {}*, {}** %key
	store {}* %31, {}** %30
	%32 = load {}*, {}** %replaced_key
	ret {}* %32
.block.9:
	br label %.block.6
.block.6:
	br label %.block.1
.block.3:
	%new_node = alloca %type.BinaryTreeNode*
	%33 = call {}*(i64) @malloc(i64 noundef 24)
	%34 = bitcast {}* %33 to %type.BinaryTreeNode*
	store %type.BinaryTreeNode* %34, %type.BinaryTreeNode** %new_node
	%35 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %new_node
	%36 = load {}*, {}** %key
	%37 = alloca %type.BinaryTreeNode
	store %type.BinaryTreeNode { {}* undef, %type.BinaryTreeNode* null, %type.BinaryTreeNode* null }, %type.BinaryTreeNode* %37
	%38 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %37, i32 0, i32 0
	store {}* %36, {}** %38
	%39 = load %type.BinaryTreeNode, %type.BinaryTreeNode* %37
	store %type.BinaryTreeNode %39, %type.BinaryTreeNode* %35
	%40 = load %type.BinaryTreeNode**, %type.BinaryTreeNode*** %current_ptr
	%41 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %new_node
	store %type.BinaryTreeNode* %41, %type.BinaryTreeNode** %40
	ret {}* null
}

define dso_local i1 @"BinaryTree::contains"(%type.BinaryTree* noundef %.arg.self, {}* noundef %.arg.key) {
.block.0:
	%self = alloca %type.BinaryTree*
	store %type.BinaryTree* %.arg.self, %type.BinaryTree** %self
	%key = alloca {}*
	store {}* %.arg.key, {}** %key
	%current = alloca %type.BinaryTreeNode*
	%0 = load %type.BinaryTree*, %type.BinaryTree** %self
	%1 = getelementptr inbounds %type.BinaryTree, %type.BinaryTree* %0, i32 0, i32 0
	%2 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %1
	store %type.BinaryTreeNode* %2, %type.BinaryTreeNode** %current
	br label %.block.1
.block.1:
	%3 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %current
	%4 = icmp ne %type.BinaryTreeNode* %3, null
	br i1 %4, label %.block.2, label %.block.3
.block.2:
	%ordering = alloca i32
	%5 = load %type.BinaryTree*, %type.BinaryTree** %self
	%6 = getelementptr inbounds %type.BinaryTree, %type.BinaryTree* %5, i32 0, i32 1
	%7 = load i32({}*, {}*)*, i32({}*, {}*)** %6
	%8 = load {}*, {}** %key
	%9 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %current
	%10 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %9, i32 0, i32 0
	%11 = load {}*, {}** %10
	%12 = call i32({}*, {}*) %7({}* noundef %8, {}* noundef %11)
	store i32 %12, i32* %ordering
	%13 = load i32, i32* %ordering
	%14 = icmp slt i32 %13, 0
	br i1 %14, label %.block.4, label %.block.5
.block.4:
	%15 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %current
	%16 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %15, i32 0, i32 1
	%17 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %16
	store %type.BinaryTreeNode* %17, %type.BinaryTreeNode** %current
	br label %.block.6
.block.5:
	%18 = load i32, i32* %ordering
	%19 = icmp sgt i32 %18, 0
	br i1 %19, label %.block.7, label %.block.8
.block.7:
	%20 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %current
	%21 = getelementptr inbounds %type.BinaryTreeNode, %type.BinaryTreeNode* %20, i32 0, i32 2
	%22 = load %type.BinaryTreeNode*, %type.BinaryTreeNode** %21
	store %type.BinaryTreeNode* %22, %type.BinaryTreeNode** %current
	br label %.block.9
.block.8:
	ret i1 true
.block.9:
	br label %.block.6
.block.6:
	br label %.block.1
.block.3:
	ret i1 false
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

define dso_local i32 @main() {
.block.0:
	%list = alloca %type.SinglyLinkedList
	%0 = call %type.SinglyLinkedList() @"SinglyLinkedList::new"()
	store %type.SinglyLinkedList %0, %type.SinglyLinkedList* %list
	%i = alloca i32
	store i32 1, i32* %i
	br label %.block.1
.block.1:
	%1 = load i32, i32* %i
	%2 = icmp sle i32 %1, 5
	br i1 %2, label %.block.2, label %.block.3
.block.2:
	%value = alloca i32*
	%3 = call {}*(i64) @malloc(i64 noundef 4)
	%4 = bitcast {}* %3 to i32*
	store i32* %4, i32** %value
	%5 = load i32*, i32** %value
	%6 = load i32, i32* %i
	store i32 %6, i32* %5
	%7 = load i32*, i32** %value
	%8 = load i32, i32* %7
	%9 = call i32(i8*, ...) @printf(i8* noundef bitcast ([10 x i8]* @.const.0 to i8*), i32 noundef %8)
	%10 = load i32*, i32** %value
	%11 = bitcast i32* %10 to {}*
	call void(%type.SinglyLinkedList*, {}*) @"SinglyLinkedList::push_front"(%type.SinglyLinkedList* noundef %list, {}* noundef %11)
	%12 = load i32, i32* %i
	%13 = add nsw i32 %12, 1
	store i32 %13, i32* %i
	br label %.block.1
.block.3:
	%value-1 = alloca i32*
	br label %.block.4
.block.4:
	%14 = call {}*(%type.SinglyLinkedList*) @"SinglyLinkedList::pop_front"(%type.SinglyLinkedList* noundef %list)
	%15 = bitcast {}* %14 to i32*
	store i32* %15, i32** %value-1
	%16 = icmp ne i32* %15, null
	br i1 %16, label %.block.5, label %.block.6
.block.5:
	%17 = load i32*, i32** %value-1
	%18 = load i32, i32* %17
	%19 = call i32(i8*, ...) @printf(i8* noundef bitcast ([9 x i8]* @.const.1 to i8*), i32 noundef %18)
	br label %.block.4
.block.6:
	%bst = alloca %type.BinaryTree
	%20 = call %type.BinaryTree(i32({}*, {}*)*) @"BinaryTree::new"(i32({}*, {}*)* noundef bitcast (i32(i32*, i32*)* @"i32::cmp" to i32({}*, {}*)*))
	store %type.BinaryTree %20, %type.BinaryTree* %bst
	%i-1 = alloca i32
	store i32 0, i32* %i-1
	br label %.block.7
.block.7:
	%21 = load i32, i32* %i-1
	%22 = icmp slt i32 %21, 8
	br i1 %22, label %.block.8, label %.block.9
.block.8:
	%key = alloca i32*
	%23 = call {}*(i64) @malloc(i64 noundef 4)
	%24 = bitcast {}* %23 to i32*
	store i32* %24, i32** %key
	%25 = load i32*, i32** %key
	%26 = load i32, i32* %i-1
	%27 = mul nsw i32 %26, 7
	%28 = srem i32 %27, 11
	store i32 %28, i32* %25
	%29 = load i32*, i32** %key
	%30 = load i32, i32* %29
	%31 = call i32(i8*, ...) @printf(i8* noundef bitcast ([12 x i8]* @.const.2 to i8*), i32 noundef %30)
	%32 = load i32*, i32** %key
	%33 = bitcast i32* %32 to {}*
	%34 = call {}*(%type.BinaryTree*, {}*) @"BinaryTree::insert"(%type.BinaryTree* noundef %bst, {}* noundef %33)
	%35 = load i32, i32* %i-1
	%36 = add nsw i32 %35, 1
	store i32 %36, i32* %i-1
	br label %.block.7
.block.9:
	%i-2 = alloca i32
	store i32 0, i32* %i-2
	br label %.block.10
.block.10:
	%37 = load i32, i32* %i-2
	%38 = icmp slt i32 %37, 11
	br i1 %38, label %.block.11, label %.block.12
.block.11:
	%is_contained = alloca i8*
	%39 = bitcast i32* %i-2 to {}*
	%40 = call i1(%type.BinaryTree*, {}*) @"BinaryTree::contains"(%type.BinaryTree* noundef %bst, {}* noundef %39)
	br i1 %40, label %.block.13, label %.block.14
.block.13:
	store i8* bitcast ([4 x i8]* @.const.3 to i8*), i8** %is_contained
	br label %.block.15
.block.14:
	store i8* bitcast ([3 x i8]* @.const.4 to i8*), i8** %is_contained
	br label %.block.15
.block.15:
	%41 = load i32, i32* %i-2
	%42 = load i8*, i8** %is_contained
	%43 = call i32(i8*, ...) @printf(i8* noundef bitcast ([17 x i8]* @.const.5 to i8*), i32 noundef %41, i8* noundef %42)
	%44 = load i32, i32* %i-2
	%45 = add nsw i32 %44, 1
	store i32 %45, i32* %i-2
	br label %.block.10
.block.12:
	ret i32 0
}

@.const.0 = private unnamed_addr constant [10 x i8] c"push: %d\0A\00"
@.const.1 = private unnamed_addr constant [9 x i8] c"pop: %d\0A\00"
@.const.2 = private unnamed_addr constant [12 x i8] c"insert: %d\0A\00"
@.const.3 = private unnamed_addr constant [4 x i8] c"yes\00"
@.const.4 = private unnamed_addr constant [3 x i8] c"no\00"
@.const.5 = private unnamed_addr constant [17 x i8] c"contains %d: %s\0A\00"

declare void @free({}* noundef)

declare {}* @malloc(i64 noundef)

declare i32 @printf(i8* noundef, ...)

!llvm.module.flags = !{ !0, !1, !2, !3, !4 }
!llvm.ident = !{ !5 }
!0 = !{ i32 1, !"wchar_size", i32 4 }
!1 = !{ i32 7, !"PIC Level", i32 2 }
!2 = !{ i32 7, !"PIE Level", i32 2 }
!3 = !{ i32 7, !"uwtable", i32 1 }
!4 = !{ i32 7, !"frame-pointer", i32 2 }
!5 = !{ !"xarkenz compiler" }
