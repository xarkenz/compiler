mod common;

#[test]
fn hello() {
    common::test_compile(
        "hello.main.cupr",
        "hello.ll",
    );
}

#[test]
fn test_1() {
    common::test_compile(
        "test_1.main.cupr",
        "test_1.ll",
    );
}

#[test]
fn test_2() {
    common::test_compile(
        "test_2.main.cupr",
        "test_2.ll",
    );
}

#[test]
fn test_3() {
    common::test_compile(
        "test_3.main.cupr",
        "test_3.ll",
    );
}

#[test]
fn test_unix() {
    common::test_compile(
        "test_unix.main.cupr",
        "test_unix.ll",
    );
}

#[test]
fn test_collections() {
    common::test_compile(
        "test_collections.main.cupr",
        "test_collections.ll",
    );
}
