mod common;

#[test]
fn hello() {
    common::test_compile_package("hello");
}

#[test]
fn test_1() {
    common::test_compile_package("test_1");
}

#[test]
fn test_2() {
    common::test_compile_package("test_2");
}

#[test]
fn test_3() {
    common::test_compile_package("test_3");
}

#[test]
fn test_unix() {
    common::test_compile_package("test_unix");
}

#[test]
fn test_collections() {
    common::test_compile_package("test_collections");
}
