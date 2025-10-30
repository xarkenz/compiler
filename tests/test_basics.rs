mod common;

#[test]
fn hello() {
    common::test_compile(
        ["hello.cu"],
        "hello.ll",
    );
}

#[test]
fn test_1() {
    common::test_compile(
        ["test_1.cu"],
        "test_1.ll",
    );
}

#[test]
fn test_2() {
    common::test_compile(
        ["test_2.cu"],
        "test_2.ll",
    );
}

#[test]
fn test_3() {
    common::test_compile(
        ["test_3.cu"],
        "test_3.ll",
    );
}

#[test]
fn test_unix() {
    common::test_compile(
        ["test_unix.cu"],
        "test_unix.ll",
    );
}

#[test]
fn test_collections() {
    common::test_compile(
        ["test_collections.cu"],
        "test_collections.ll",
    );
}
