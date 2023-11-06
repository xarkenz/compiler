use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CompilerArgs {
    #[arg(short, long)]
    src: Vec<String>,
    #[arg(short, long)]
    out: String,
    #[arg(long)]
    debug: bool,
}

impl CompilerArgs {
    pub fn source_paths(&self) -> &[String] {
        &self.src
    }

    pub fn output_path(&self) -> &str {
        &self.out
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

pub fn parse_command_line_args() -> CompilerArgs {
    CompilerArgs::parse()
}

pub fn invoke(args: CompilerArgs) -> crate::Result<()> {
    for source_filename in args.source_paths() {
        println!("Compiling '{source_filename}':");

        let mut scanner = crate::token::scan::Scanner::from_file(source_filename)?;
        let mut parser = crate::ast::parse::Parser::new(&mut scanner)?;

        // while let Some(statement) = parser.parse_statement()? {
        //     println!("{statement}");
        // }

        let output_filename = args.output_path();
        crate::gen::Generator::from_filename(output_filename)?
            .generate(&mut parser)?;
        println!("LLVM successfully written to '{output_filename}'.");
    }

    Ok(())
}
