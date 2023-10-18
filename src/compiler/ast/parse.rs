use super::*;

use std::fmt::Write;
use std::io::BufRead;

fn expected_token_error_message(allowed: &[Token], got_token: &Token) -> String {
    let mut message = format!("expected {}", &allowed[0]);
    for token in &allowed[1..] {
        write!(&mut message, ", {token}").unwrap();
    }
    write!(&mut message, "; got {got_token}").unwrap();
    message
}

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    scanner: &'a mut scan::Scanner<'a, T>,
    current_token: Option<Token>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(scanner: &'a mut scan::Scanner<'a, T>) -> crate::Result<Self> {
        let mut new_instance = Self {
            scanner,
            current_token: None,
        };
        new_instance.scan_token()?;
        Ok(new_instance)
    }

    pub fn scan_token(&mut self) -> crate::Result<()> {
        self.current_token = self.scanner.next_token()?;
        Ok(())
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn get_token(&self) -> crate::Result<&Token> {
        self.current_token().ok_or_else(|| self.scanner.syntax_error(String::from("unexpected end of file")))
    }

    pub fn expect_token<'b>(&self, allowed: &'b [Token]) -> crate::Result<&'b Token> {
        let current_token = self.get_token()?;
        allowed.iter()
            .find(|token| token == &current_token)
            .ok_or_else(|| self.scanner.syntax_error(expected_token_error_message(allowed, current_token)))
    }

    pub fn parse_terminal_node(&mut self) -> crate::Result<Box<Node>> {
        match self.get_token()? {
            Token::Literal(literal) => {
                let node = Box::new(Node::Literal(literal.clone()));
                self.scan_token()?;
                Ok(node)
            },
            token => Err(self.scanner.syntax_error(format!("expected a terminal token, got {token:?}")))
        }
    }

    pub fn parse_binary_expression(&mut self, parent_operation: Option<BinaryOperation>, allowed_ends: &[Token]) -> crate::Result<Box<Node>> {
        let parent_precedence = parent_operation.map(|operation| operation.precedence());

        let mut lhs = self.parse_terminal_node()?;

        while let Some(token) = self.current_token() {
            let operation = match token {
                Token::Plus => BinaryOperation::Add,
                Token::Minus => BinaryOperation::Subtract,
                Token::Star => BinaryOperation::Multiply,
                Token::Slash => BinaryOperation::Divide,
                _ => break
            };
            let precedence = operation.precedence();

            if let Some(parent_precedence) = parent_precedence {
                if parent_precedence > precedence || (
                    parent_precedence == precedence && precedence.associativity() == Associativity::LeftToRight
                ) {
                    // Parent operation has a higher precedence and therefore must be made into a subtree of the next operation
                    // If the parent operation has the same precedence, only make it a subtree when there is left-to-right associativity
                    break;
                }
            }

            self.scan_token()?;
            let rhs = self.parse_binary_expression(Some(operation), allowed_ends)?;

            lhs = Box::new(Node::Binary {
                operation,
                lhs,
                rhs,
            });
        }

        if parent_operation.is_none() {
            self.expect_token(allowed_ends)?;
        }

        Ok(lhs)
    }

    pub fn parse_statement(&mut self) -> crate::Result<Option<Box<Node>>> {
        if self.current_token().is_none() {
            Ok(None)
        } else {
            self.expect_token(&[Token::Print])?;
            self.scan_token()?;
            let print_expression = self.parse_binary_expression(None, &[Token::Semicolon])?;
            self.scan_token()?;
            Ok(Some(print_expression))
        }
    }
}