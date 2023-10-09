use std::fmt;

pub trait Error : std::error::Error {
    fn filename(&self) -> &str;

    fn line(&self) -> Option<usize> {
        None
    }

    fn into_boxed(self) -> Box<dyn Error> where Self: Sized + 'static {
        Box::new(self)
    }

    fn write_preamble(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line() {
            write!(f, "{}:{line}: ", self.filename())
        } else {
            write!(f, "{}: ", self.filename())
        }
    }
}

#[derive(Debug)]
pub struct RawError {
    message: String,
}

impl RawError {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RawError {}

impl Error for RawError {
    fn filename(&self) -> &str {
        "<unknown>"
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct FileError {
    filename: String,
    line: Option<usize>,
    cause: std::io::Error,
}

impl FileError {
    pub fn new(filename: String, line: Option<usize>, cause: std::io::Error) -> Self {
        Self {
            filename,
            line,
            cause,
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_preamble(f)?;
        write!(f, "{}", self.cause)
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl Error for FileError {
    fn filename(&self) -> &str {
        &self.filename
    }

    fn line(&self) -> Option<usize> {
        self.line
    }
}

#[derive(Debug)]
pub struct SyntaxError {
    filename: String,
    line: usize,
    message: String,
}

impl SyntaxError {
    pub fn new(filename: String, line: usize, message: String) -> Self {
        Self {
            filename,
            line,
            message,
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_preamble(f)?;
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxError {}

impl Error for SyntaxError {
    fn filename(&self) -> &str {
        &self.filename
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }
}
