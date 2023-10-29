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

    pub fn parse_operand(&mut self) -> crate::Result<Box<Node>> {
        let token = self.get_token()?;

        if let Some(operation) = UnaryOperation::from_prefix_token(token) {
            self.scan_token()?;
            let operand = self.parse_operand()?;

            Ok(Box::new(Node::Unary {
                operation,
                operand,
            }))
        }
        else {
            let mut operand = match token {
                Token::ParenLeft => {
                    self.scan_token()?;
                    self.parse_expression(None, &[Token::ParenRight])?
                },
                Token::Literal(literal) => {
                    Box::new(Node::Literal(literal.clone()))
                },
                _ => return Err(self.scanner.syntax_error(format!("expected an operand, got {token}")))
            };
            self.scan_token()?;

            while let Some(operation) = UnaryOperation::from_postfix_token(self.get_token()?) {
                operand = Box::new(Node::Unary {
                    operation,
                    operand,
                });
                self.scan_token()?;
            }

            Ok(operand)
        }
    }

    pub fn parse_expression(&mut self, parent_precedence: Option<Precedence>, allowed_ends: &[Token]) -> crate::Result<Box<Node>> {
        let mut lhs = self.parse_operand()?;

        while let Some(token) = self.current_token() {
            if allowed_ends.contains(token) {
                break;
            }
            else if let Some(operation) = BinaryOperation::from_token(token) {
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
                let rhs = self.parse_expression(Some(precedence), allowed_ends)?;

                lhs = Box::new(Node::Binary {
                    operation,
                    lhs,
                    rhs,
                });
            }
            else {
                return Err(self.scanner.syntax_error(format!("expected an operation, got {token}")));
            }
        }

        if parent_precedence.is_none() {
            self.expect_token(allowed_ends)?;
        }

        Ok(lhs)
    }

    pub fn parse_statement(&mut self) -> crate::Result<Option<Box<Node>>> {
        if let Some(token) = self.current_token() {
            match token {
                Token::Semicolon => {
                    self.scan_token()?;
                    self.parse_statement()
                },
                Token::Let => {
                    self.scan_token()?;
                    let identifier = self.parse_expression(None, &[Token::Colon])?;
                    self.scan_token()?;
                    let value_type = self.parse_expression(None, &[Token::Equal, Token::Semicolon])?;
                    let value;
                    if let Some(Token::Equal) = self.current_token() {
                        self.scan_token()?;
                        value = Some(self.parse_expression(None, &[Token::Semicolon])?);
                    }
                    else {
                        value = None;
                    }
                    self.scan_token()?;
                    Ok(Some(Box::new(Node::Let {
                        identifier,
                        value_type,
                        value,
                    })))
                },
                Token::Print => {
                    self.scan_token()?;
                    let value = self.parse_expression(None, &[Token::Semicolon])?;
                    self.scan_token()?;
                    Ok(Some(Box::new(Node::Print {
                        value,
                    })))
                },
                _ => {
                    let expression = self.parse_expression(None, &[Token::Semicolon])?;
                    self.scan_token()?;
                    Ok(Some(expression))
                }
            }
        }
        else {
            Ok(None)
        }
    }
}