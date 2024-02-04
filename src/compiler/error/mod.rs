use super::*;

use std::fmt;
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Clone, PartialEq, Debug)]
pub struct Span {
    pub file_id: usize,
    pub start_index: usize,
    pub length: usize,
}

impl Span {
    pub fn context_to_string(&self, filename: &str) -> std::io::Result<(usize, usize, String)> {
        let mut reader = BufReader::new(File::open(filename)?);
        let mut line_number = 0;
        let mut column_number = 0;
        let mut line_start_index = 0;
        let mut context = String::new();
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            let line_end_index = line_start_index + line.len();

            if line_end_index > self.start_index {
                if column_number == 0 {
                    column_number = self.start_index - line_start_index + 1;
                }

                if line_start_index < self.start_index + self.length {
                    let line_trim = line.trim_end();

                    // Write the line from the source file
                    context.push('\t');
                    context.push_str(line_trim);
                    context.push('\n');

                    // Write the span markers on the line below
                    context.push('\t');
                    for index in line_start_index .. line_start_index + line_trim.len() {
                        if index >= self.start_index + self.length {
                            break;
                        }
                        else if index >= self.start_index {
                            context.push('^');
                        }
                        else {
                            context.push(' ');
                        }
                    }
                    context.push('\n');
                }
                else {
                    break;
                }
            }

            line_number += 1;
            line_start_index = line_end_index;
            line.clear();
        }

        Ok((line_number, column_number, context))
    }
}

pub enum Error {
    SourceFileOpen {
        file_id: usize,
        cause: std::io::Error,
    },
    SourceFileRead {
        file_id: usize,
        line: usize,
        cause: std::io::Error,
    },
    OutputFileOpen {
        filename: String,
        cause: std::io::Error,
    },
    OutputFileWrite {
        filename: String,
        cause: std::io::Error,
    },
    InvalidToken {
        span: Span,
    },
    NonAsciiCharacter {
        span: Span,
        what: char,
    },
    InvalidEscape {
        span: Span,
        what: char,
    },
    InvalidHexEscapeDigit {
        span: Span,
        what: char,
    },
    UnclosedString {
        span: Span,
    },
    UnclosedCharacter {
        span: Span,
    },
    UnclosedComment {
        span: Span,
    },
    ExpectedToken {
        span: Span,
    },
    ExpectedTokenFromList {
        span: Span,
        got_token: token::Token,
        allowed_tokens: Vec<token::Token>,
    },
    ExpectedIdentifier {
        span: Span,
    },
    ExpectedOperand {
        span: Span,
        got_token: token::Token,
    },
    ExpectedOperation {
        span: Span,
        got_token: token::Token,
    },
    ExpectedType {
        span: Span,
        got_token: token::Token,
    },
    UnexpectedQualifier {
        span: Span,
        got_token: token::Token,
    },
    ExpectedClosingBracket {
        span: Span,
        bracket: token::Token,
    },
    ExpectedStatement {
        span: Span,
    },
    UnexpectedElse {
        span: Span,
    },
    UnknownTypeSize {
        type_name: String,
    },
    CannotMutateValue {
        type_name: String,
    },
    ExpectedLValue {
    },
    UndefinedSymbol {
        name: String,
    },
    PartialType {
        type_name: String,
    },
    UnknownType {
        type_name: String,
    },
    NonConstantArrayLength {
    },
    IncompatibleTypes {
        expected_type: String,
        got_type: String,
    },
    InconvertibleTypes {
        original_type: String,
        target_type: String,
    },
    UnexpectedExpression {
    },
    InvalidBreak {
    },
    InvalidContinue {
    },
    InvalidReturn {
    },
    ExpectedReturnValue {
    },
    UnexpectedReturnValue {
    },
    NonValueSymbol {
        name: String,
    },
    NonConstantSymbol {
        name: String,
    },
    IncompatibleValueType {
        value: String,
        type_name: String,
    },
    UnknownArrayType {
    },
    InvalidStructIdentifier {
    },
    NonStructSymbol {
        name: String,
    },
    NonStructType {
        type_name: String,
    },
    MissingStructMembers {
        member_names: Vec<String>,
        type_name: String,
    },
    ExtraStructMembers {
        member_names: Vec<String>,
        type_name: String,
    },
    UndefinedStructMember {
        member_name: String,
        type_name: String,
    },
    ExpectedPointer {
        type_name: String,
    },
    ExpectedInteger {
        type_name: String,
    },
    ExpectedArray {
        type_name: String,
    },
    ExpectedStruct {
        type_name: String,
    },
    ExpectedFunction {
        type_name: String,
    },
    MissingFunctionArguments {
        expected_count: usize,
        got_count: usize,
    },
    ExtraFunctionArguments {
        expected_count: usize,
        got_count: usize,
    },
    GlobalSymbolConflict {
        name: String,
    },
    FunctionSignatureConflict {
        function_name: String,
        old_type: String,
        new_type: String,
    },
    MultipleFunctionDefinition {
        function_name: String,
    },
    MissingReturnStatement {
        function_name: String,
    },
    UnsupportedConstantExpression {
    },
    InvalidStaticAccess {
    },
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

impl Error {
    pub fn span(&self) -> Option<&Span> {
        match self {
            Error::InvalidToken { span } => Some(span),
            Error::NonAsciiCharacter { span, .. } => Some(span),
            Error::InvalidEscape { span, .. } => Some(span),
            Error::InvalidHexEscapeDigit { span, .. } => Some(span),
            Error::UnclosedString { span } => Some(span),
            Error::UnclosedCharacter { span } => Some(span),
            Error::UnclosedComment { span } => Some(span),
            Error::ExpectedToken { span } => Some(span),
            Error::ExpectedTokenFromList { span, .. } => Some(span),
            Error::ExpectedIdentifier { span } => Some(span),
            Error::ExpectedOperand { span, .. } => Some(span),
            Error::ExpectedOperation { span, .. } => Some(span),
            Error::ExpectedType { span, .. } => Some(span),
            Error::UnexpectedQualifier { span, .. } => Some(span),
            Error::ExpectedClosingBracket { span, .. } => Some(span),
            Error::ExpectedStatement { span } => Some(span),
            Error::UnexpectedElse { span } => Some(span),
            _ => None
        }
    }

    pub fn to_string_with_context(&self, filenames: &[String]) -> String {
        if let Some(span) = self.span() {
            let filename = &filenames[span.file_id];
            if let Ok((line_number, column_number, context)) = span.context_to_string(filename) {
                format!("{filename}:{line_number}:{column_number}: {self}\n\n{context}")
            }
            else {
                format!("{filename}: {self}")
            }
        }
        else {
            format!("{self}")
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceFileOpen { cause, .. } => write!(f, "unable to open file: {cause}"),
            Self::SourceFileRead { cause, .. } => write!(f, "error while reading file: {cause}"),
            Self::OutputFileOpen { cause, .. } => write!(f, "unable to create file: {cause}"),
            Self::OutputFileWrite { cause, .. } => write!(f, "error while writing file: {cause}"),
            Self::InvalidToken { .. } => write!(f, "unrecognized token"),
            Self::NonAsciiCharacter { what, .. } => write!(f, "non-ASCII character '{what}' in literal"),
            Self::InvalidEscape { what, .. } => write!(f, "unrecognized escape '\\{what}'"),
            Self::InvalidHexEscapeDigit { what, .. } => write!(f, "expected hexadecimal digit, got '{what}'"),
            Self::UnclosedString { .. } => write!(f, "unclosed string literal"),
            Self::UnclosedCharacter { .. } => write!(f, "expected single quote to close character literal"),
            Self::UnclosedComment { .. } => write!(f, "unclosed block comment"),
            Self::ExpectedToken { .. } => write!(f, "unexpected end of file"),
            Self::ExpectedTokenFromList { got_token, allowed_tokens, .. } => {
                write!(f, "expected '{}'", &allowed_tokens[0])?;
                for token in &allowed_tokens[1..] {
                    write!(f, ", '{token}'")?;
                }
                write!(f, "; got '{got_token}'")
            },
            Self::ExpectedIdentifier { .. } => write!(f, "expected an identifier"),
            Self::ExpectedOperand { got_token, .. } => write!(f, "expected an operand, got '{got_token}'"),
            Self::ExpectedOperation { got_token, .. } => write!(f, "expected an operation, got '{got_token}'"),
            Self::ExpectedType { got_token, .. } => write!(f, "expected a type, got '{got_token}'"),
            Self::UnexpectedQualifier { got_token, .. } => write!(f, "type qualifier '{got_token}' is not allowed here"),
            Self::ExpectedClosingBracket { bracket, .. } => write!(f, "expected closing '{bracket}'"),
            Self::ExpectedStatement { .. } => write!(f, "expected a statement"),
            Self::UnexpectedElse { .. } => write!(f, "unexpected 'else' without previous 'if'"),
            Self::UnknownTypeSize { type_name } => write!(f, "cannot use type '{type_name}' here, as it does not have a known size at this time (did you mean to use a pointer?)"),
            Self::CannotMutateValue { type_name } => write!(f, "cannot mutate value of type '{type_name}' as it is not 'mut'"),
            Self::ExpectedLValue {} => write!(f, "expected an lvalue"),
            Self::UndefinedSymbol { name } => write!(f, "undefined symbol '{name}'"),
            Self::PartialType { type_name } => write!(f, "type '{type_name}' is declared but not defined"),
            Self::UnknownType { type_name } => write!(f, "unrecognized type name '{type_name}'"),
            Self::NonConstantArrayLength {} => write!(f, "array length must be constant"),
            Self::IncompatibleTypes { expected_type, got_type } => write!(f, "expected a value of type '{expected_type}', got '{got_type}' instead"),
            Self::InconvertibleTypes { original_type, target_type } => write!(f, "cannot convert from '{original_type}' to '{target_type}'"),
            Self::UnexpectedExpression {} => write!(f, "unexpected expression type"),
            Self::InvalidBreak {} => write!(f, "unexpected 'break' outside loop"),
            Self::InvalidContinue {} => write!(f, "unexpected 'continue' outside loop"),
            Self::InvalidReturn {} => write!(f, "unexpected 'return' outside function"),
            Self::ExpectedReturnValue {} => write!(f, "cannot return without a value from a non-void function"),
            Self::UnexpectedReturnValue {} => write!(f, "cannot return a value from a void function"),
            Self::NonValueSymbol { name } => write!(f, "cannot use '{name}' as a value"),
            Self::NonConstantSymbol { name } => write!(f, "'{name}' is not constant and cannot be used in a constant expression"),
            Self::IncompatibleValueType { value, type_name } => write!(f, "'{value}' cannot be used as a value of type '{type_name}'"),
            Self::UnknownArrayType {} => write!(f, "unable to infer array type"),
            Self::InvalidStructIdentifier {} => write!(f, "invalid syntax for struct type"),
            Self::NonStructSymbol { name } => write!(f, "cannot use '{name}' as a struct type"),
            Self::NonStructType { type_name } => write!(f, "type '{type_name}' is not a struct type"),
            Self::MissingStructMembers { member_names, type_name } => {
                write!(f, "missing members in initializer of struct '{type_name}': {}", &member_names[0])?;
                for member_name in &member_names[1..] {
                    write!(f, ", {member_name}")?;
                }
                Ok(())
            },
            Self::ExtraStructMembers { member_names, type_name } => {
                write!(f, "extraneous members in initializer of struct '{type_name}': {}", &member_names[0])?;
                for member_name in &member_names[1..] {
                    write!(f, ", {member_name}")?;
                }
                Ok(())
            },
            Self::UndefinedStructMember { member_name, type_name } => write!(f, "member '{member_name}' does not exist in struct '{type_name}'"),
            Self::ExpectedPointer { type_name } => write!(f, "expected a pointer, got value of type '{type_name}'"),
            Self::ExpectedInteger { type_name } => write!(f, "expected an integer, got value of type '{type_name}'"),
            Self::ExpectedArray { type_name } => write!(f, "expected an array or pointer to array, got value of type '{type_name}'"),
            Self::ExpectedStruct { type_name } => write!(f, "expected a struct, got value of type '{type_name}'"),
            Self::ExpectedFunction { type_name } => write!(f, "expected a function, got value of type '{type_name}'"),
            Self::MissingFunctionArguments { expected_count, got_count } => write!(f, "too few arguments (expected {expected_count}, got {got_count})"),
            Self::ExtraFunctionArguments { expected_count, got_count } => write!(f, "too many arguments (expected {expected_count}, got {got_count})"),
            Self::GlobalSymbolConflict { name } => write!(f, "global name '{name}' is already in use"),
            Self::FunctionSignatureConflict { function_name, old_type, new_type } => {
                writeln!(f, "conflicting signatures for function '{function_name}':")?;
                writeln!(f, "old: {old_type}")?;
                write!(f, "new: {new_type}")
            },
            Self::MultipleFunctionDefinition { function_name } => write!(f, "multiple definitions exist for function '{function_name}'"),
            Self::MissingReturnStatement { function_name } => write!(f, "non-void function '{function_name}' could finish without returning a value"),
            Self::UnsupportedConstantExpression {} => write!(f, "unsupported feature in constant expression"),
            Self::InvalidStaticAccess {} => write!(f, "invalid operands for '::'"),
        }
    }
}
