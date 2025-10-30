use clap::Parser as ClapParser;
use crate::ast::parse::parse_all;
use crate::gen::Generator;
use crate::sema::GlobalContext;
use crate::target::TargetInfo;
use crate::token::scan::Scanner;

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
pub struct CompilerArgs {
    /// Read source code from <path>
    #[arg(short, long, value_name = "path")]
    src: Vec<String>,
    /// Write output to <path>
    #[arg(short, long, value_name = "path")]
    out: String,
}

impl CompilerArgs {
    pub fn source_paths(&self) -> &[String] {
        &self.src
    }

    pub fn output_path(&self) -> &str {
        &self.out
    }
}

pub fn parse_command_line_args() -> CompilerArgs {
    CompilerArgs::parse()
}

pub fn invoke(args: &CompilerArgs) -> crate::Result<()> {
    for (file_id, source_filename) in args.source_paths().iter().enumerate() {
        println!("Compiling '{source_filename}':");

        // Set up the global context for compilation
        let target = TargetInfo::new(size_of::<&()>() as u64);
        let mut context = GlobalContext::new(target);

        // Scanning, parsing, and outline pass simultaneously
        let mut scanner = Scanner::from_filename(file_id, source_filename.clone())?;
        let mut ast = parse_all(&mut scanner, &mut context)?;

        // Fill pass (must be done after outline pass is complete)
        context.process_global_statements(&mut ast)?;

        // Code generation and emission
        let output_filename = args.output_path();
        Generator::from_filename(output_filename.into(), context)?
            .generate_all(&ast, scanner.file_id(), args.source_paths())?;

        println!("LLVM successfully written to '{output_filename}'.");
    }

    Ok(())
}
