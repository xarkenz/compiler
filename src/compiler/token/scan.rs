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
            .map_err(|cause| Box::new(crate::Error::SourceFileOpen {
                file_id,
                cause,
            }))
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

    pub fn next_token(&mut self) -> crate::Result<Option<(crate::Span, Token)>> {
        if let Some(ch) = self.next_non_space_char()? {
            if ch.is_ascii_digit() {
                self.put_back(ch);
                self.scan_numeric_literal().map(Some)
            }
            else if ch == '_' || ch.is_ascii_alphanumeric() {
                self.put_back(ch);
                self.scan_word_literal().map(Some)
            }
            else {
                if ch == '/' {
                    match self.next_char()? {
                        Some('/') => {
                            self.skip_line_comment()?;
                            return self.next_token();
                        }
                        Some('*') => {
                            self.skip_block_comment()?;
                            return self.next_token();
                        }
                        Some(next_ch) => {
                            self.put_back(next_ch);
                        }
                        None => {}
                    }
                }
                else if ch == '"' {
                    return self.scan_string_literal().map(Some);
                }
                else if ch == '\'' {
                    return self.scan_character_literal().map(Some);
                }
                self.put_back(ch);
                self.scan_symbolic_literal().map(Some)
            }
        }
        else {
            Ok(None)
        }
    }

    fn next_char(&mut self) -> crate::Result<Option<char>> {
        if let Some(ch) = self.put_backs.pop() {
            self.next_index += 1;
            Ok(Some(ch))
        }
        else {
            let read = self.source.read_char()
                .map_err(|cause| Box::new(crate::Error::SourceFileRead {
                    file_id: self.file_id,
                    line: self.line,
                    cause,
                }))?;

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

    fn scan_alphanumeric_word(&mut self) -> crate::Result<(crate::Span, String)> {
        let start_index = self.next_index;
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            match ch {
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => {
                    content.push(ch);
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        Ok((
            self.create_span(start_index, self.next_index),
            content,
        ))
    }

    fn scan_numeric_literal(&mut self) -> crate::Result<(crate::Span, Token)> {
        let start_index = self.next_index;
        let mut content = String::new();
        let mut suffix = None;

        while let Some(ch) = self.next_char()? {
            match ch {
                '_' => {}
                '0'..='9' => {
                    content.push(ch);
                }
                '.' => {
                    // A dot could either be a decimal point or an access operation. We'll
                    // only consider it to be a decimal point if the following character is a digit.
                    match self.next_char()? {
                        Some('0'..='9') => {
                            content.push(ch);
                            return self.scan_float_literal_end(start_index, content);
                        }
                        Some(non_digit) => {
                            self.put_back(non_digit);
                        }
                        None => {}
                    }
                    self.put_back(ch);
                    break;
                }
                'E' | 'e' => {
                    content.push(ch);
                    return self.scan_float_literal_end(start_index, content);
                }
                'f' => {
                    // This is a bit of a hack, but this detects the start of a float suffix.
                    // Since the result will be a float literal, just delegate to float scanning.
                    return self.scan_float_literal_end(start_index, content);
                }
                'A'..='Z' | 'a'..='z' => {
                    // The only valid integer suffixes start with 'i' and 'u', but detect any
                    // letter here to provide more graceful error handling.
                    self.put_back(ch);
                    suffix = Some(self.scan_integer_suffix()?);
                    break;
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        let value = content.chars().fold(0, |value, digit| {
            10 * value + (digit as i128 - '0' as i128)
        });

        Ok((
            self.create_span(start_index, self.next_index),
            Token::Literal(Literal::Integer(value, suffix)),
        ))
    }

    fn scan_float_literal_end(&mut self, start_index: usize, mut content: String) -> crate::Result<(crate::Span, Token)> {
        let mut suffix = None;

        while let Some(ch) = self.next_char()? {
            match ch {
                '_' => {}
                '0'..='9' => {
                    content.push(ch);
                }
                '+' | '-' if matches!(content.chars().last(), Some('E' | 'e')) => {
                    content.push(ch);
                }
                'A'..='Z' | 'a'..='z' => {
                    self.put_back(ch);
                    suffix = Some(self.scan_float_suffix()?);
                    break;
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        let value = content.parse::<f64>()
            .map_err(|_| Box::new(crate::Error::InvalidToken {
                span: self.create_span(start_index, self.next_index),
            }))?;

        Ok((
            self.create_span(start_index, self.next_index),
            Token::Literal(Literal::Float(value, suffix)),
        ))
    }

    fn scan_integer_suffix(&mut self) -> crate::Result<IntegerSuffix> {
        let (span, content) = self.scan_alphanumeric_word()?;

        IntegerSuffix::find(&content)
            .ok_or_else(|| Box::new(crate::Error::InvalidLiteralSuffix {
                span,
            }))
    }

    fn scan_float_suffix(&mut self) -> crate::Result<FloatSuffix> {
        let (span, content) = self.scan_alphanumeric_word()?;

        FloatSuffix::find(&content)
            .ok_or_else(|| Box::new(crate::Error::InvalidLiteralSuffix {
                span,
            }))
    }

    fn scan_word_literal(&mut self) -> crate::Result<(crate::Span, Token)> {
        let (span, content) = self.scan_alphanumeric_word()?;

        if let Some(keyword_token) = get_keyword_token_match(&content) {
            Ok((
                span,
                keyword_token.clone(),
            ))
        }
        else {
            let literal = match content.as_str() {
                "true" => Literal::Boolean(true),
                "false" => Literal::Boolean(false),
                "null" => Literal::NullPointer,
                _ => match crate::sema::TypeHandle::primitive(&content) {
                    Some(primitive_type) => Literal::PrimitiveType(primitive_type),
                    None => Literal::Name(content),
                }
            };

            Ok((
                span,
                Token::Literal(literal),
            ))
        }
    }

    fn scan_symbolic_literal(&mut self) -> crate::Result<(crate::Span, Token)> {
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
                return Ok((
                    self.create_span(start_index, self.next_index),
                    symbolic_token.clone(),
                ));
            }

            // Since no match was found, take away a character and try again
            let ch = content.pop().unwrap();
            self.put_back(ch);
        }

        Err(Box::new(crate::Error::InvalidToken {
            span: self.create_span(start_index, start_index),
        }))
    }

    fn scan_escaped_char(&mut self) -> crate::Result<Option<u8>> {
        let start_index = self.next_index;

        if let Some(ch) = self.next_char()? {
            if ch == '\\' {
                match self.next_char()? {
                    Some('\\') => {
                        Ok(Some(b'\\'))
                    }
                    Some('\"') => {
                        Ok(Some(b'\"'))
                    }
                    Some('\'') => {
                        Ok(Some(b'\''))
                    }
                    Some('n') => {
                        Ok(Some(b'\n'))
                    }
                    Some('t') => {
                        Ok(Some(b'\t'))
                    }
                    Some('0') => {
                        Ok(Some(b'\0'))
                    }
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
                                        return Err(Box::new(crate::Error::InvalidHexEscapeDigit {
                                            span: self.create_span(start_index, start_index + 4),
                                            what: ch,
                                        }));
                                    }
                                };
                            }
                            else {
                                return Ok(None);
                            }
                        }
                        Ok(Some(byte))
                    }
                    Some(ch) => {
                        Err(Box::new(crate::Error::InvalidEscape {
                            span: self.create_span(start_index, self.next_index),
                            what: ch,
                        }))
                    }
                    None => {
                        Ok(None)
                    }
                }
            }
            else if ch.is_ascii() {
                Ok(Some(ch as u8))
            }
            else {
                Err(Box::new(crate::Error::NonAsciiCharacter {
                    span: self.create_span(start_index, start_index),
                    what: ch,
                }))
            }
        }
        else {
            Ok(None)
        }
    }

    fn scan_string_literal(&mut self) -> crate::Result<(crate::Span, Token)> {
        let start_index = self.next_index - 1;
        let mut bytes = Vec::new();

        while let Some(ch) = self.next_char()? {
            if ch == '"' {
                // Add the NUL byte at the end
                // TODO: syntax like r"hello" to opt out?
                bytes.push(0);
                return Ok((
                    self.create_span(start_index, self.next_index),
                    Token::Literal(Literal::String(crate::sema::StringValue::new(bytes))),
                ));
            }
            else {
                self.put_back(ch);
                bytes.push(self.scan_escaped_char()?.ok_or_else(|| {
                    Box::new(crate::Error::UnclosedString {
                        span: self.create_span(start_index, start_index),
                    })
                })?);
            }
        }

        Err(Box::new(crate::Error::UnclosedString {
            span: self.create_span(start_index, start_index),
        }))
    }

    fn scan_character_literal(&mut self) -> crate::Result<(crate::Span, Token)> {
        let start_index = self.next_index - 1;
        let byte = self.scan_escaped_char()?.ok_or_else(|| {
            Box::new(crate::Error::UnclosedCharacter {
                span: self.create_span(self.next_index, self.next_index),
            })
        })?;

        if let Some('\'') = self.next_char()? {
            let suffix = match self.next_char()? {
                Some(ch @ ('A'..='Z' | 'a'..='z' | '_')) => {
                    self.put_back(ch);
                    Some(self.scan_integer_suffix()?)
                }
                Some(ch) => {
                    self.put_back(ch);
                    None
                }
                None => None,
            };

            Ok((
                self.create_span(start_index, self.next_index),
                Token::Literal(Literal::Integer(byte as i128, suffix)),
            ))
        }
        else {
            Err(Box::new(crate::Error::UnclosedCharacter {
                span: self.create_span(self.next_index, self.next_index),
            }))
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

        Err(Box::new(crate::Error::UnclosedComment {
            span: self.create_span(start_index, start_index + 2),
        }))
    }
}
