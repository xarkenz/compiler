use super::*;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span {
    pub source_id: usize,
    pub start_index: usize,
    pub length: usize,
}

impl Span {
    pub fn tail_point(mut self) -> Self {
        self.start_index += self.length;
        self.length = 0;
        self
    }

    pub fn expand_to(mut self, end_span: Self) -> Self {
        if self.source_id != end_span.source_id {
            panic!("source IDs do not match");
        }
        self.length = end_span.start_index.checked_add(end_span.length)
            .and_then(|end_index| end_index.checked_sub(self.start_index))
            .expect("end span comes before start span");
        self
    }

    pub fn context_to_string(&self, path: impl AsRef<Path>) -> std::io::Result<(usize, usize, String)> {
        let mut reader = BufReader::new(File::open(path)?);
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
                    context.extend(std::iter::repeat_n(
                        ' ',
                        self.start_index.saturating_sub(line_start_index),
                    ));
                    if self.length == 0 {
                        context.push('^');
                    }
                    else {
                        context.extend(std::iter::repeat_n(
                            '~',
                            line_end_index.min(self.start_index + self.length)
                                .saturating_sub(line_start_index.max(self.start_index))
                        ));
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

pub enum ErrorKind {
    PackageFile {
        cause: String,
    },
    SourceFileOpen {
        cause: std::io::Error,
    },
    SourceFileRead {
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
    InvalidToken,
    InvalidLiteralSuffix,
    NonAsciiCharacter {
        what: char,
    },
    InvalidEscape {
        what: char,
    },
    InvalidHexEscapeDigit {
        what: char,
    },
    UnclosedString,
    UnclosedCharacter,
    UnclosedComment,
    ExpectedToken,
    ExpectedTokenFromList {
        got_token: token::Token,
        allowed_tokens: Vec<token::Token>,
    },
    ExpectedIdentifier,
    ExpectedString,
    ExpectedTupleMember,
    TupleMemberOutOfRange {
        member: String,
        member_count: i32,
    },
    ExpectedOperand {
        got_token: token::Token,
    },
    ExpectedOperation {
        got_token: token::Token,
    },
    ExpectedType {
        got_token: token::Token,
    },
    UnexpectedQualifier {
        got_token: token::Token,
    },
    ExpectedClosingBracket {
        bracket: token::Token,
    },
    ExpectedStatement,
    UnexpectedElse,
    UnexpectedNoBreak,
    InvalidGlobPath,
    CannotMutateValue {
        type_name: String,
    },
    ExpectedLValue,
    UndefinedSymbol {
        name: String,
    },
    UndefinedGlobalSymbol {
        namespace: String,
        name: String,
    },
    GlobalSymbolConflict {
        namespace: String,
        name: String,
    },
    NonTypeSymbol {
        name: String,
    },
    InvalidSuper {
        namespace: String,
    },
    ExpectedNamespace {
        name: String,
    },
    RecursiveTypeDefinition {
        type_name: String,
    },
    UnknownTypeSize {
        type_name: String,
    },
    UnknownTypeAlignment {
        type_name: String,
    },
    NonConstantArrayLength,
    IncompatibleTypes {
        expected_type: String,
        got_type: String,
    },
    InconvertibleTypes {
        from_type: String,
        to_type: String,
    },
    UnexpectedExpression,
    InvalidBreak,
    InvalidContinue,
    ExpectedReturnValue {
        function_name: String,
    },
    UnexpectedReturnValue {
        function_name: String,
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
    UnknownArrayType,
    UnknownTupleType,
    CannotInferType,
    NoSuchMethod {
        type_name: String,
        method_name: String,
    },
    InvalidStructIdentifier,
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
    UndefinedMember {
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
    InvalidMemberAccess {
        type_name: String,
    },
    ExpectedFunction {
        type_name: String,
    },
    WrongFunctionArgumentCount {
        expected_count: usize,
        got_count: usize,
    },
    UnsupportedConstantExpression,
    NoSelfType,
    ExpectedSelfParameter,
    ImportAliasRequired {
        path: String,
    },
    AmbiguousSymbol {
        name: String,
        possible_paths: Vec<String>,
    },
    MustSpecifyTypeForGlobal {
        name: String,
    },
    MustSpecifyTypeForUninitialized {
        name: String,
    },
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PackageFile { cause } => write!(f, "error reading package: {cause}"),
            Self::SourceFileOpen { cause, .. } => write!(f, "unable to open file: {cause}"),
            Self::SourceFileRead { cause, .. } => write!(f, "error while reading file: {cause}"),
            Self::OutputFileOpen { cause, .. } => write!(f, "unable to create file: {cause}"),
            Self::OutputFileWrite { cause, .. } => write!(f, "error while writing file: {cause}"),
            Self::InvalidToken => write!(f, "unrecognized token"),
            Self::InvalidLiteralSuffix => write!(f, "unsupported literal suffix"),
            Self::NonAsciiCharacter { what } => write!(f, "non-ASCII character '{what}' in literal"),
            Self::InvalidEscape { what } => write!(f, "unrecognized escape '\\{what}'"),
            Self::InvalidHexEscapeDigit { what } => write!(f, "expected hexadecimal digit, got '{what}'"),
            Self::UnclosedString => write!(f, "unclosed string literal"),
            Self::UnclosedCharacter => write!(f, "expected single quote to close character literal"),
            Self::UnclosedComment => write!(f, "unclosed block comment"),
            Self::ExpectedToken => write!(f, "unexpected end of file"),
            Self::ExpectedTokenFromList { got_token, allowed_tokens } => {
                write!(f, "expected '{}'", &allowed_tokens[0])?;
                for token in &allowed_tokens[1..] {
                    write!(f, ", '{token}'")?;
                }
                write!(f, "; got '{got_token}'")
            }
            Self::ExpectedIdentifier => write!(f, "expected an identifier"),
            Self::ExpectedString => write!(f, "expected a quoted string"),
            Self::ExpectedTupleMember => write!(f, "expected a tuple member"),
            Self::TupleMemberOutOfRange { member, member_count } => write!(f, "member '{member}' is out of range for a tuple with {member_count} members"),
            Self::ExpectedOperand { got_token } => write!(f, "expected an operand, got '{got_token}'"),
            Self::ExpectedOperation { got_token } => write!(f, "expected an operation, got '{got_token}'"),
            Self::ExpectedType { got_token } => write!(f, "expected a type, got '{got_token}'"),
            Self::UnexpectedQualifier { got_token } => write!(f, "type qualifier '{got_token}' is not allowed here"),
            Self::ExpectedClosingBracket { bracket } => write!(f, "expected closing '{bracket}'"),
            Self::ExpectedStatement => write!(f, "expected a statement"),
            Self::UnexpectedElse => write!(f, "unexpected 'else' without previous 'if'"),
            Self::UnexpectedNoBreak => write!(f, "unexpected 'nobreak' without previous 'while'"),
            Self::InvalidGlobPath => write!(f, "the '*' for a glob path must be located at the end of the path"),
            Self::CannotMutateValue { type_name } => write!(f, "cannot mutate value of type '{type_name}' as it is not 'mut'"),
            Self::ExpectedLValue => write!(f, "expected an lvalue"),
            Self::UndefinedSymbol { name } => write!(f, "symbol '{name}' is not defined"),
            Self::UndefinedGlobalSymbol { namespace, name } => write!(f, "symbol '{name}' is not defined in namespace '{namespace}'"),
            Self::GlobalSymbolConflict { namespace, name } => write!(f, "symbol '{name}' is defined multiple times in namespace '{namespace}'"),
            Self::NonTypeSymbol { name } => write!(f, "'{name}' does not represent a type"),
            Self::InvalidSuper { namespace } => write!(f, "namespace '{namespace}' has no 'super'"),
            Self::ExpectedNamespace { name } => write!(f, "expected a module or type, got '{name}'"),
            Self::RecursiveTypeDefinition { type_name } => write!(f, "recursive type definition for {type_name} (did you mean to use a pointer?)"),
            Self::UnknownTypeSize { type_name } => write!(f, "cannot use type '{type_name}' here, as its size is not constant (did you mean to use a pointer?)"),
            Self::UnknownTypeAlignment { type_name } => write!(f, "type '{type_name}' does not have a defined alignment"),
            Self::NonConstantArrayLength => write!(f, "array length must be constant"),
            Self::IncompatibleTypes { expected_type, got_type } => write!(f, "expected a value of type '{expected_type}', got '{got_type}' instead"),
            Self::InconvertibleTypes { from_type, to_type } => write!(f, "cannot convert from '{from_type}' to '{to_type}'"),
            Self::UnexpectedExpression => write!(f, "unexpected expression type"),
            Self::InvalidBreak => write!(f, "unexpected 'break' outside loop"),
            Self::InvalidContinue => write!(f, "unexpected 'continue' outside loop"),
            Self::ExpectedReturnValue { function_name } => write!(f, "cannot return without a value from non-void function '{function_name}'"),
            Self::UnexpectedReturnValue { function_name } => write!(f, "cannot return a value from void function '{function_name}'"),
            Self::NonValueSymbol { name } => write!(f, "cannot use '{name}' as a value"),
            Self::NonConstantSymbol { name } => write!(f, "'{name}' is not constant and cannot be used in a constant expression"),
            Self::IncompatibleValueType { value, type_name } => write!(f, "'{value}' cannot be used as a value of type '{type_name}'"),
            Self::UnknownArrayType => write!(f, "unable to infer array type"),
            Self::UnknownTupleType => write!(f, "unable to infer tuple type"),
            Self::CannotInferType => write!(f, "unable to infer the type of this expression"),
            Self::NoSuchMethod { type_name, method_name } => write!(f, "{type_name} has no method '{method_name}' (to call a member, wrap it in parentheses)"),
            Self::InvalidStructIdentifier => write!(f, "invalid syntax for struct type"),
            Self::NonStructSymbol { name } => write!(f, "cannot use '{name}' as a struct type"),
            Self::NonStructType { type_name } => write!(f, "type '{type_name}' is not a struct type"),
            Self::MissingStructMembers { member_names, type_name } => {
                write!(f, "missing members in initializer of struct '{type_name}': {}", &member_names[0])?;
                for member_name in &member_names[1..] {
                    write!(f, ", {member_name}")?;
                }
                Ok(())
            }
            Self::ExtraStructMembers { member_names, type_name } => {
                write!(f, "extraneous members in initializer of struct '{type_name}': {}", &member_names[0])?;
                for member_name in &member_names[1..] {
                    write!(f, ", {member_name}")?;
                }
                Ok(())
            }
            Self::UndefinedMember { member_name, type_name } => write!(f, "type '{type_name}' has no member '{member_name}'"),
            Self::ExpectedPointer { type_name } => write!(f, "expected a pointer, got value of type '{type_name}'"),
            Self::ExpectedInteger { type_name } => write!(f, "expected an integer, got value of type '{type_name}'"),
            Self::ExpectedArray { type_name } => write!(f, "expected an array, got value of type '{type_name}'"),
            Self::InvalidMemberAccess { type_name } => write!(f, "expected a struct or tuple, got value of type '{type_name}'"),
            Self::ExpectedFunction { type_name } => write!(f, "expected a function, got value of type '{type_name}'"),
            Self::WrongFunctionArgumentCount { expected_count, got_count } => {
                write!(f, "too {} arguments for function (expected {expected_count}, got {got_count})", if got_count < expected_count { "few" } else { "many" })
            }
            Self::UnsupportedConstantExpression => write!(f, "unsupported feature in constant expression"),
            Self::NoSelfType => write!(f, "keyword 'Self' can only be used inside 'implement' blocks and 'struct' definitions"),
            Self::ExpectedSelfParameter => write!(f, "expected a first parameter of type 'Self', '*Self', or '*mut Self'"),
            Self::ImportAliasRequired { path } => write!(f, "import '{path}' must be renamed using the syntax 'import _ as <name>'"),
            Self::AmbiguousSymbol { name, possible_paths } => {
                write!(f, "'{name}' could refer to multiple imported items ({}", &possible_paths[0])?;
                for possible_path in &possible_paths[1..] {
                    write!(f, ", {possible_path}")?;
                }
                write!(f, "); try importing one of these paths directly")
            }
            Self::MustSpecifyTypeForGlobal { name } => write!(f, "must specify type for global variable '{name}'"),
            Self::MustSpecifyTypeForUninitialized { name } => write!(f, "must specify type for '{name}' if no initial value is given"),
        }
    }
}

impl std::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

pub struct Error {
    span: Option<Span>,
    kind: ErrorKind,
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

impl Error {
    pub fn new(span: Option<Span>, kind: ErrorKind) -> Self {
        Self {
            span,
            kind,
        }
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut ErrorKind {
        &mut self.kind
    }

    pub fn to_string_with_context(&self, paths: &[PathBuf]) -> String {
        if let Some(span) = self.span() {
            let path = &paths[span.source_id];
            let path_display = path.display();
            if let Ok((line_number, column_number, context)) = span.context_to_string(path) {
                format!("Error in '{path_display}':\nline {line_number}:{column_number}: {self}\n\n{context}")
            }
            else {
                format!("Error in '{path_display}':\n{self}")
            }
        }
        else {
            format!("Error:\n{self}")
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error({})", self.kind())
    }
}
