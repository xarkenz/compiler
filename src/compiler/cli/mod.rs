use clap::Parser;
use crate::sema::GlobalContext;

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

pub fn invoke(args: &CompilerArgs) -> crate::Result<()> {
    for (file_id, source_filename) in args.source_paths().iter().enumerate() {
        println!("Compiling '{source_filename}':");

        let mut scanner = crate::token::scan::Scanner::from_filename(file_id, source_filename.clone())?;
        let mut parser = crate::ast::parse::Parser::new(&mut scanner)?;

        let output_filename = args.output_path();
        let context = GlobalContext::new();
        crate::gen::Generator::from_filename(output_filename.into(), context)?
            .generate(&mut parser, args.source_paths())?;
        
        println!("LLVM successfully written to '{output_filename}'.");
    }

    Ok(())
}
