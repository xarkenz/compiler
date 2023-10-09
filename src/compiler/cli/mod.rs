use crate::{Error, RawError};

use std::path::PathBuf;
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CompilerArgs {
    #[arg(short, long, action = ArgAction::Append)]
    src: Vec<PathBuf>,
    #[arg(long)]
    debug: bool,
}

impl CompilerArgs {
    pub fn source_paths(&self) -> &[PathBuf] {
        &self.src
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

pub fn parse_cl_args() -> CompilerArgs {
    CompilerArgs::parse()
}

pub fn invoke(args: CompilerArgs) -> crate::Result<()> {
    for source_path in args.source_paths() {
        let filename = source_path.to_str().expect("invalid utf-8 sequence in filename");

        println!("Interpreting '{filename}':");

        let mut scanner = crate::token::scan::Scanner::from_file(filename)?;
        let mut parser = crate::ast::parse::Parser::new(&mut scanner);
        
        fn interpret_ast(root: &crate::ast::Node) -> crate::Result<u64> {
            match root {
                crate::ast::Node::Literal(crate::token::Literal::Integer(value)) => {
                    Ok(*value)
                },
                crate::ast::Node::Binary { operation, lhs, rhs } => {
                    let lhs = interpret_ast(lhs.as_ref())?;
                    let rhs = interpret_ast(rhs.as_ref())?;
                    use crate::ast::BinaryOperation;
                    match operation {
                        BinaryOperation::Add => Ok(lhs + rhs),
                        BinaryOperation::Subtract => Ok(lhs - rhs),
                        BinaryOperation::Multiply => Ok(lhs * rhs),
                        BinaryOperation::Divide => Ok(lhs / rhs),
                        _ => Err(RawError::new(format!("unexpected binary operation: {operation:?}")).into_boxed())
                    }
                },
                _ => Err(RawError::new(format!("unexpected node: {root:?}")).into_boxed())
            }
        }

        parser.scan_token()?;
        let parsed_ast = parser.parse_binary_expression(None)?;
        let result = interpret_ast(parsed_ast.as_ref())?;

        println!("Result: {result}");
        dbg!(&parsed_ast);
    }

    Ok(())
}
