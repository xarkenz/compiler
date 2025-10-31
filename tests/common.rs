use clap::Parser;

pub fn test_compile<'a>(
    source_filename: &str,
    output_filename: &str,
) {
    let args = compiler::cli::CompilerArgs::parse_from([
        "compiler".to_string(),
        "--out".to_string(),
        format!("tests/outputs/{output_filename}"),
        format!("tests/sources/{source_filename}"),
    ]);

    if let Err(error) = compiler::cli::invoke(&args) {
        let (error, source_paths) = *error;
        panic!("Error: {}", error.to_string_with_context(&source_paths))
    }
}
