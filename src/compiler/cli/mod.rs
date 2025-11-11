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
    #[doc = "Compile the package inside directory <package_path>"]
    #[arg(value_name = "package_path")]
    package: PathBuf,
}

impl CompilerArgs {
    pub fn package_path(&self) -> &Path {
        &self.package
    }
}

pub fn parse_command_line_args() -> CompilerArgs {
    CompilerArgs::parse()
}

pub fn invoke(args: &CompilerArgs) -> Result<(), Box<(crate::Error, Vec<PathBuf>)>> {
    let error = |context: &GlobalContext| {
        let source_paths = context.package().source_paths().to_vec();
        move |error: Box<crate::Error>| Box::new((*error, source_paths))
    };

    let package_path = args.package_path();

    // Set up the global context for compilation
    let target = TargetInfo::new(size_of::<&()>() as u64);
    let mut context = GlobalContext::new(package_path, target)
        .map_err(|error| Box::new((*error, Vec::new())))?;

    loop {
        println!("--- Compiling package '{}' ---", context.package().info().name());

        let mut parsed_modules = Vec::new();
        while let Some((source_id, namespace)) = context
            .prepare_next_source()
            .map_err(error(&context))?
        {
            // Scanning, parsing, and outline pass simultaneously
            let source_path = &context.package().source_paths()[source_id];
            println!("Parsing '{}'...", source_path.display());

            let mut scanner = Scanner::from_path(source_id, source_path)
                .map_err(error(&context))?;

            let parsed_module = parse_module(&mut scanner, &mut context, namespace)
                .map_err(error(&context))?;

            parsed_modules.push(parsed_module);
        }

        // Fill pass (must be done after outline pass is complete for all files)
        println!("Processing definitions...");
        context.process_package(&mut parsed_modules).map_err(error(&context))?;

        // Code generation and emission
        let source_paths = context.package().source_paths().to_vec();
        let output_path = context.package().info().get_output_path();
        println!("Generating output at '{}'...", output_path.display());
        Generator::from_path(&output_path, &mut context)
            .and_then(|generator| generator.generate_package(&parsed_modules))
            .map_err(|error| Box::new((*error, source_paths)))?;

        println!("LLVM IR successfully written to '{}'.", output_path.display());

        if !context.start_next_package() {
            break;
        }
    }

    Ok(())
}
