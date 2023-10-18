use super::*;

use crate::{Error, FileError, SyntaxError};

use std::io::{BufRead, BufReader};
use std::fs::File;

use utf8_chars::BufReadCharsExt;

#[derive(Debug)]
pub struct Scanner<'a, T: BufRead> {
    filename: &'a str,
    line: usize,
    source: T,
    put_backs: Vec<char>,
}

impl<'a> Scanner<'a, BufReader<File>> {
    pub fn from_file(filename: &'a str) -> crate::Result<Self> {
        let source = BufReader::new(File::open(filename)
            .map_err(|cause| FileError::new(filename.to_owned(), None, cause).into_boxed())?);
        Ok(Self::new(filename, source))
    }
}

impl<'a, T: BufRead> Scanner<'a, T> {
    pub fn new(filename: &'a str, source: T) -> Self {
        Self {
            filename,
            line: 1,
            source,
            put_backs: Vec::new(),
        }
    }

    pub fn filename(&self) -> &'a str {
        self.filename
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn file_error(&self, cause: std::io::Error) -> Box<dyn Error> {
        FileError::new(self.filename.to_owned(), Some(self.line), cause).into_boxed()
    }

    pub fn syntax_error(&self, message: String) -> Box<dyn Error> {
        SyntaxError::new(self.filename.to_owned(), self.line, message).into_boxed()
    }

    fn next_char(&mut self) -> crate::Result<Option<char>> {
        if let Some(ch) = self.put_backs.pop() {
            Ok(Some(ch))
        } else {
            let read = self.source.read_char()
                .map_err(|cause| self.file_error(cause))?;
            if let Some(ch) = read {
                if ch == '\n' {
                    self.line += 1;
                }
                Ok(Some(ch))
            } else {
                Ok(None)
            }
        }
    }

    fn put_back(&mut self, ch: char) {
        self.put_backs.push(ch);
    }

    fn next_non_space_char(&mut self) -> crate::Result<Option<char>> {
        while let Some(ch) = self.next_char()? {
            if !ch.is_whitespace() {
                return Ok(Some(ch));
            }
        }
        
        Ok(None)
    }

    fn scan_integer_literal(&mut self) -> crate::Result<Option<Token>> {
        let mut raw_literal = String::new();

        while let Some(ch) = self.next_char()? {
            if ch.is_ascii_digit() {
                raw_literal.push(ch);
            } else {
                self.put_back(ch);
                break;
            }
        }

        let mut value = 0;
        for digit in raw_literal.chars() {
            value = 10 * value + digit as u64 - '0' as u64;
        }

        Ok(Some(Token::Literal(Literal::Integer(value))))
    }

    fn scan_identifier_literal(&mut self) -> crate::Result<Option<Token>> {
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            if ch == '_' || ch.is_alphanumeric() {
                content.push(ch);
            } else {
                self.put_back(ch);
                break;
            }
        }

        if let Some(keyword_token) = get_keyword_token(&content) {
            Ok(Some(keyword_token.clone()))
        } else {
            Ok(Some(Token::Literal(Literal::Identifier(content))))
        }
    }

    pub fn next_token(&mut self) -> crate::Result<Option<Token>> {
        if let Some(ch) = self.next_non_space_char()? {
            let possible_tokens = get_symbolic_token_matches(&String::from(ch));
            if possible_tokens.is_empty() {
                if ch.is_ascii_digit() {
                    self.put_back(ch);
                    self.scan_integer_literal()
                } else if ch == '_' || ch.is_alphanumeric() {
                    self.put_back(ch);
                    self.scan_identifier_literal()
                } else {
                    // temporary
                    Ok(None)
                }
            } else {
                // temporary
                Ok(Some(possible_tokens[0].clone()))
            }
        } else {
            Ok(None)
        }
    }
}