use clap::Parser;

#[test]
fn general_test() {
    let args = compiler::cli::CompilerArgs::parse_from([
        "compiler",
        "-s", "tests/test3.txt",
        "-o", "tests/test3.ll",
    ]);
    if let Err(error) = compiler::cli::invoke(&args) {
        panic!("Error: {}", error.to_string_with_context(args.source_paths()))
    }
}
