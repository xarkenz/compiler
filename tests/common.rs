use clap::Parser;

pub fn test_compile<'a>(
    source_filenames: impl IntoIterator<Item = &'a str>,
    output_filename: &str,
) {
    let args_iter = std::iter::once("compiler".to_string())
        .chain(source_filenames
            .into_iter()
            .flat_map(|source_filename| [
                "-s".to_string(),
                format!("tests/sources/{source_filename}"),
            ]))
        .chain([
            "-o".to_string(),
            format!("tests/outputs/{output_filename}"),
        ]);
    let args = compiler::cli::CompilerArgs::parse_from(args_iter);

    if let Err(error) = compiler::cli::invoke(&args) {
        panic!("Error: {}", error.to_string_with_context(args.source_paths()))
    }
}
