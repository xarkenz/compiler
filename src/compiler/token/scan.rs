use super::*;

use std::io::{BufRead, BufReader};
use std::fs::File;

use utf8_chars::BufReadCharsExt;

#[derive(Debug)]
pub struct Scanner<T: BufRead> {
    file_id: usize,
    next_index: usize,
    line: usize,
    source: T,
    put_backs: Vec<char>,
}

impl Scanner<BufReader<File>> {
    pub fn from_filename(file_id: usize, filename: String) -> crate::Result<Self> {
        File::open(filename)
            .map(|file| Self::new(file_id, BufReader::new(file)))
            .map_err(|cause| Box::new(crate::Error::SourceFileOpen { file_id, cause }))
    }
}

impl<T: BufRead> Scanner<T> {
    pub fn new(file_id: usize, source: T) -> Self {
        Self {
            file_id,
            next_index: 0,
            line: 1,
            source,
            put_backs: Vec::new(),
        }
    }

    pub fn file_id(&self) -> usize {
        self.file_id
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn next_index(&self) -> usize {
        self.next_index
    }

    pub fn create_span(&self, start_index: usize, end_index: usize) -> crate::Span {
        crate::Span {
            file_id: self.file_id,
            start_index,
            length: end_index - start_index,
        }
    }

    fn next_char(&mut self) -> crate::Result<Option<char>> {
        if let Some(ch) = self.put_backs.pop() {
            self.next_index += 1;
            Ok(Some(ch))
        }
        else {
            let read = self.source.read_char()
                .map_err(|cause| Box::new(crate::Error::SourceFileRead { file_id: self.file_id, line: self.line, cause }))?;

            if let Some(ch) = read {
                if ch == '\n' {
                    self.line += 1;
                }
                self.next_index += 1;
                Ok(Some(ch))
            }
            else {
                Ok(None)
            }
        }
    }

    fn put_back(&mut self, ch: char) {
        self.put_backs.push(ch);
        self.next_index -= 1;
    }

    fn next_non_space_char(&mut self) -> crate::Result<Option<char>> {
        while let Some(ch) = self.next_char()? {
            if !ch.is_whitespace() {
                return Ok(Some(ch));
            }
        }
        
        Ok(None)
    }

    fn scan_integer_literal(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        let start_index = self.next_index;
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
            value = 10 * value + (digit as i128 - '0' as i128);
        }

        let span = self.create_span(start_index, self.next_index);
        Ok(Some((span, Token::Literal(Literal::Integer(value)))))
    }

    fn scan_identifier_literal(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        let start_index = self.next_index;
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                content.push(ch);
            }
            else {
                self.put_back(ch);
                break;
            }
        }

        let span = self.create_span(start_index, self.next_index);

        if let Some(keyword_token) = get_keyword_token_match(&content) {
            Ok(Some((span, keyword_token.clone())))
        }
        else {
            let literal = match content.as_str() {
                "true" => Literal::Boolean(true),
                "false" => Literal::Boolean(false),
                "null" => Literal::NullPointer,
                _ => Literal::Identifier(content)
            };

            Ok(Some((span, Token::Literal(literal))))
        }
    }

    fn scan_symbolic_literal(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        let start_index = self.next_index;
        let mut content = String::new();

        // Consume characters as long as the current sequence is a valid token prefix
        while let Some(ch) = self.next_char()? {
            content.push(ch);
            let matches = get_symbolic_token_partial_matches(content.as_str());
            if matches.is_empty() {
                let ch = content.pop().unwrap();
                self.put_back(ch);
                break;
            }
        }

        // Backtrack to find the longest exact token match
        while !content.is_empty() {
            if let Some(symbolic_token) = get_symbolic_token_match(content.as_str()) {
                let span = self.create_span(start_index, self.next_index);
                return Ok(Some((span, symbolic_token.clone())));
            }

            // Since no match was found, take away a character and try again
            let ch = content.pop().unwrap();
            self.put_back(ch);
        }

        let span = self.create_span(start_index, start_index);
        Err(Box::new(crate::Error::InvalidToken { span }))
    }

    fn scan_escaped_char(&mut self) -> crate::Result<Option<u8>> {
        let start_index = self.next_index;

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
                                    _ => {
                                        let span = self.create_span(start_index, start_index + 4);
                                        return Err(Box::new(crate::Error::InvalidHexEscapeDigit { span, what: ch }));
                                    }
                                };
                            }
                            else {
                                return Ok(None)
                            }
                        }
                        Ok(Some(byte))
                    },
                    Some(ch) => {
                        let span = self.create_span(start_index, self.next_index);
                        Err(Box::new(crate::Error::InvalidEscape { span, what: ch }))
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
                let span = self.create_span(start_index, start_index);
                Err(Box::new(crate::Error::NonAsciiCharacter { span, what: ch }))
            }
        }
        else {
            Ok(None)
        }
    }

    fn scan_string_literal(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        let start_index = self.next_index - 1;
        let mut bytes = Vec::new();

        while let Some(ch) = self.next_char()? {
            if ch == '"' {
                let span = self.create_span(start_index, self.next_index);
                // Add the NUL byte at the end
                // TODO: syntax like r"hello" to opt out?
                bytes.push(0);
                return Ok(Some((span, Token::Literal(Literal::String(crate::sema::StringValue::new(bytes))))));
            }
            else {
                self.put_back(ch);
                bytes.push(self.scan_escaped_char()?.ok_or_else(|| {
                    let span = self.create_span(start_index, start_index);
                    Box::new(crate::Error::UnclosedString { span })
                })?);
            }
        }

        let span = self.create_span(start_index, start_index);
        Err(Box::new(crate::Error::UnclosedString { span }))
    }

    fn scan_character_literal(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        let start_index = self.next_index - 1;
        let byte = self.scan_escaped_char()?.ok_or_else(|| {
            let span = self.create_span(self.next_index, self.next_index);
            Box::new(crate::Error::UnclosedCharacter { span })
        })?;

        if let Some('\'') = self.next_char()? {
            let span = self.create_span(start_index, self.next_index);
            Ok(Some((span, Token::Literal(Literal::Integer(byte as i128)))))
        }
        else {
            let span = self.create_span(self.next_index, self.next_index);
            Err(Box::new(crate::Error::UnclosedCharacter { span }))
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
        // TODO: recursive block comments
        let start_index = self.next_index - 2;
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

        let span = self.create_span(start_index, start_index + 2);
        Err(Box::new(crate::Error::UnclosedComment { span }))
    }

    pub fn next_token(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
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