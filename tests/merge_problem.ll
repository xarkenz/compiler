; file_id = 0
source_filename = "tests/merge_problem.txt"

%"type.::merge::first::First" = type { %"type.::merge::target::MyType" }

%"type.::merge::second::Second" = type { %"type.::merge::target::MyType" }

%"type.::merge::target::MyType" = type {}

define void @"::merge::target::MyType::my_method"() {
.block.0:
	ret void
}

