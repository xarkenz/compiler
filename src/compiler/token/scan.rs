use super::*;

use crate::Error;

use std::io::{BufRead, BufReader};
use std::fs::File;

use utf8_chars::BufReadCharsExt;

#[derive(Debug)]
pub struct Scanner<T: BufRead> {
    filename: String,
    line: usize,
    source: T,
    put_backs: Vec<char>,
}

impl Scanner<BufReader<File>> {
    pub fn from_filename(filename: String) -> crate::Result<Self> {
        File::open(filename.clone())
            .map(|file| Self::new(filename.clone(), BufReader::new(file)))
            .map_err(|cause| crate::FileError::new(filename.clone(), None, cause).into_boxed())
    }
}

impl<T: BufRead> Scanner<T> {
    pub fn new(filename: String, source: T) -> Self {
        Self {
            filename,
            line: 1,
            source,
            put_backs: Vec::new(),
        }
    }

    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn file_error(&self, cause: std::io::Error) -> Box<dyn Error> {
        crate::FileError::new(self.filename.to_owned(), Some(self.line), cause).into_boxed()
    }

    pub fn syntax_error(&self, message: String) -> Box<dyn Error> {
        crate::SyntaxError::new(self.filename.to_owned(), self.line, message).into_boxed()
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
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            if ch.is_ascii_digit() {
                content.push(ch);
            } else {
                self.put_back(ch);
                break;
            }
        }

        let mut value = 0;
        for digit in content.chars() {
            value = 10 * value + digit as u64 - '0' as u64;
        }

        Ok(Some(Token::Literal(Literal::Integer(value))))
    }

    fn scan_identifier_literal(&mut self) -> crate::Result<Option<Token>> {
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                content.push(ch);
            } else {
                self.put_back(ch);
                break;
            }
        }

        if let Some(keyword_token) = get_keyword_token_match(&content) {
            Ok(Some(keyword_token.clone()))
        }
        else if &content == "true" {
            Ok(Some(Token::Literal(Literal::Boolean(true))))
        }
        else if &content == "false" {
            Ok(Some(Token::Literal(Literal::Boolean(false))))
        }
        else if &content == "null" {
            Ok(Some(Token::Literal(Literal::NullPointer)))
        }
        else {
            Ok(Some(Token::Literal(Literal::Identifier(content))))
        }
    }

    fn scan_symbolic_literal(&mut self) -> crate::Result<Option<Token>> {
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            content.push(ch);
            let matches = get_symbolic_token_partial_matches(content.as_str());
            if matches.is_empty() {
                break;
            }
        }

        while let Some(ch) = content.pop() {
            self.put_back(ch);
            if let Some(symbolic_token) = get_symbolic_token_match(content.as_str()) {
                return Ok(Some(symbolic_token.clone()));
            }
        }

        Err(self.syntax_error(String::from("unrecognized token")))
    }

    fn scan_escaped_char(&mut self) -> crate::Result<Option<u8>> {
        if let Some(ch) = self.next_char()? {
            if ch == '\\' {
                match self.next_char()? {
                    Some('\\') => {
                        Ok(Some(b'\\'))
                    },
                    Some('\"') => {
                        Ok(Some(b'\"'))
                    },
                    Some('\'') => {
                        Ok(Some(b'\''))
                    },
                    Some('n') => {
                        Ok(Some(b'\n'))
                    },
                    Some('t') => {
                        Ok(Some(b'\t'))
                    },
                    Some('0') => {
                        Ok(Some(b'\0'))
                    },
                    Some('x') => {
                        let mut byte = 0;
                        for _ in 0..2 {
                            if let Some(ch) = self.next_char()? {
                                // why did i do this manually, you ask? idk man no real reason
                                byte *= 16;
                                byte += match ch {
                                    '0'..='9' => ch as u8 - b'0',
                                    'A'..='F' => ch as u8 - b'A' + 10,
                                    'a'..='f' => ch as u8 - b'a' + 10,
                                    _ => return Err(self.syntax_error(format!("invalid hexadecimal digit '{ch}'")))
                                };
                            }
                            else {
                                return Ok(None)
                            }
                        }
                        Ok(Some(byte))
                    },
                    Some(escape_ch) => {
                        Err(self.syntax_error(format!("unrecognized escape character '{escape_ch}'")))
                    },
                    None => {
                        Ok(None)
                    },
                }
            }
            else if ch.is_ascii() {
                Ok(Some(ch as u8))
            }
            else {
                Err(self.syntax_error(format!("non-ASCII character '{ch}' in literal")))
            }
        }
        else {
            Ok(None)
        }
    }

    fn scan_string_literal(&mut self) -> crate::Result<Option<Token>> {
        let mut bytes = Vec::new();

        while let Some(ch) = self.next_char()? {
            if ch == '"' {
                // Add the NUL byte at the end
                // TODO: syntax like r"hello" to opt out?
                bytes.push(0);
                return Ok(Some(Token::Literal(Literal::String(StringValue::new(bytes)))));
            }
            else {
                self.put_back(ch);
                bytes.push(self.scan_escaped_char()?
                    .ok_or_else(|| self.syntax_error(String::from("unclosed string literal")))?);
            }
        }

        Err(self.syntax_error(String::from("unclosed string literal")))
    }

    fn scan_character_literal(&mut self) -> crate::Result<Option<Token>> {
        let byte = self.scan_escaped_char()?
            .ok_or_else(|| self.syntax_error(String::from("unclosed character literal")))?;

        match self.next_char()? {
            Some('\'') => Ok(Some(Token::Literal(Literal::Integer(byte as u64)))),
            Some(_) => Err(self.syntax_error(String::from("expected single quote to close character literal"))),
            None => Err(self.syntax_error(String::from("unclosed character literal"))),
        }
    }

    fn skip_line_comment(&mut self) -> crate::Result<()> {
        let mut escape_next_newline = false;

        while let Some(ch) = self.next_char()? {
            if ch == '\n' {
                if escape_next_newline {
                    escape_next_newline = false;
                }
                else {
                    break;
                }
            }
            else if ch == '\\' {
                escape_next_newline = !escape_next_newline;
            }
            else if !ch.is_whitespace() {
                escape_next_newline = false;
            }
        }

        Ok(())
    }

    fn skip_block_comment(&mut self) -> crate::Result<()> {
        let mut escape_next_char = false;

        while let Some(ch) = self.next_char()? {
            if escape_next_char {
                escape_next_char = false;
            }
            else if ch == '*' {
                match self.next_char()? {
                    Some('/') => return Ok(()),
                    Some(next_ch) => self.put_back(next_ch),
                    None => break,
                }
            }
            else if ch == '\\' {
                escape_next_char = true;
            }
        }

        Err(self.syntax_error(String::from("unclosed block comment")))
    }

    pub fn next_token(&mut self) -> crate::Result<Option<Token>> {
        if let Some(ch) = self.next_non_space_char()? {
            if ch.is_ascii_digit() {
                self.put_back(ch);
                self.scan_integer_literal()
            }
            else if ch == '_' || ch.is_ascii_alphanumeric() {
                self.put_back(ch);
                self.scan_identifier_literal()
            }
            else {
                if ch == '/' {
                    match self.next_char()? {
                        Some('/') => {
                            self.skip_line_comment()?;
                            return self.next_token();
                        },
                        Some('*') => {
                            self.skip_block_comment()?;
                            return self.next_token();
                        },
                        Some(next_ch) => self.put_back(next_ch),
                        None => {},
                    }
                }
                else if ch == '"' {
                    return self.scan_string_literal();
                }
                else if ch == '\'' {
                    return self.scan_character_literal();
                }
                self.put_back(ch);
                self.scan_symbolic_literal()
            }
        }
        else {
            Ok(None)
        }
    }
}