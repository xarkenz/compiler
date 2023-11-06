fn main() {
    let args = compiler::cli::parse_command_line_args();
    match compiler::cli::invoke(args) {
        Err(error) => eprintln!("compiler_driver: {error}"),
        Ok(_) => println!("compiler_driver: success"),
    }
}
