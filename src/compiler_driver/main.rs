fn main() -> compiler::Result<()> {
    let args = compiler::cli::parse_cl_args();
    compiler::cli::invoke(args)
}
