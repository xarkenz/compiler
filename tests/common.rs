use clap::Parser;

pub fn test_compile_package(package_name: &str) {
    let args = cupric::cli::CompilerArgs::parse_from([
        "compiler".to_string(),
        format!("tests/packages/{package_name}"),
    ]);

    if let Err(error) = cupric::cli::invoke(&args) {
        let (error, source_paths) = *error;
        println!("\x1b[31m{}\x1b[0m", error.to_string_with_context(&source_paths));
        panic!("compile command failed")
    }
}
