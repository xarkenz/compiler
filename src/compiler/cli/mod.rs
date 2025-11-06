use std::path::{Path, PathBuf};
use clap::Parser as ClapParser;
use crate::ast::parse::parse_module;
use crate::gen::Generator;
use crate::sema::GlobalContext;
use crate::target::TargetInfo;
use crate::token::scan::Scanner;

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
pub struct CompilerArgs {
    #[doc = "Read source code rooted at <src_path>"]
    #[arg(value_name = "src_path")]
    src: PathBuf,
    #[doc = "Write output to <path>"]
    #[arg(short, long, value_name = "path")]
    out: PathBuf,
}

impl CompilerArgs {
    pub fn source_path(&self) -> &Path {
        &self.src
    }

    pub fn output_path(&self) -> &Path {
        &self.out
    }
}

pub fn parse_command_line_args() -> CompilerArgs {
    CompilerArgs::parse()
}

pub fn invoke(args: &CompilerArgs) -> Result<(), Box<(crate::Error, Vec<PathBuf>)>> {
    let error = |context: &GlobalContext| {
        let source_paths = context.source_paths().to_vec();
        move |error: Box<crate::Error>| Box::new((*error, source_paths))
    };

    println!("--- Starting compiler ---");

    let root_file_path = args.source_path();
    let output_path = args.output_path();

    // Set up the global context for compilation
    let target = TargetInfo::new(size_of::<&()>() as u64);
    let mut context = GlobalContext::new(root_file_path, target);

    let mut parsed_modules = Vec::new();
    while let Some((source_id, namespace)) = context.prepare_next_source().map_err(error(&context))? {
        // Scanning, parsing, and outline pass simultaneously
        let source_path = &context.source_paths()[source_id];
        println!("Parsing '{}'...", source_path.display());

        let mut scanner = Scanner::from_path(source_id, source_path).map_err(error(&context))?;

        context.enter_module(namespace);
        parsed_modules.push(parse_module(&mut scanner, &mut context).map_err(error(&context))?);
        context.exit_module();
    }

    // Fill pass (must be done after outline pass is complete for all files)
    println!("Processing definitions...");
    context.process_all(&mut parsed_modules).map_err(error(&context))?;

    // Code generation and emission
    println!("Generating output...");
    let source_paths = context.source_paths().to_vec();
    Generator::from_path(output_path, context)
        .and_then(|generator| generator.generate_all(&parsed_modules))
        .map_err(|error| Box::new((*error, source_paths)))?;

    println!("LLVM IR successfully written to '{}'.", output_path.display());

    Ok(())
}
