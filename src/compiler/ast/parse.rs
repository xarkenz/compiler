use super::*;

use std::fmt::Write;
use std::io::BufRead;

fn expected_token_error_message(allowed: &[Token], got_token: &Token) -> String {
    let mut message = format!("expected '{}'", &allowed[0]);
    for token in &allowed[1..] {
        write!(&mut message, ", '{token}'").unwrap();
    }
    write!(&mut message, "; got '{got_token}'").unwrap();
    message
}

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    scanner: &'a mut scan::Scanner<T>,
    current_token: Option<Token>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(scanner: &'a mut scan::Scanner<T>) -> crate::Result<Self> {
        let mut new_instance = Self {
            scanner,
            current_token: None,
        };
        new_instance.scan_token()?;
        Ok(new_instance)
    }

    pub fn filename(&'a self) -> &'a str {
        self.scanner.filename()
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

    pub fn expect_identifier(&self) -> crate::Result<String> {
        match self.get_token()? {
            Token::Literal(Literal::Identifier(name)) => Ok(name.clone()),
            _ => Err(self.scanner.syntax_error(String::from("expected an identifier")))
        }
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
            // Allowed ends are checked before operations, even if a valid operation can end the expression
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
                let rhs = match operation {
                    BinaryOperation::Convert => {
                        Box::new(Node::ValueType(self.parse_value_type(allowed_ends)?))
                    },
                    _ => {
                        self.parse_expression(Some(precedence), allowed_ends)?
                    }
                };

                lhs = Box::new(Node::Binary {
                    operation,
                    lhs,
                    rhs,
                });
            }
            else if let Token::ParenLeft = token {
                // Left parenthesis indicates the function call operation
                if let Some(parent_precedence) = parent_precedence {
                    if parent_precedence >= Precedence::Postfix {
                        // Parent operation should be made into a subtree of the call operation (which has postfix precedence)
                        // Usually the parent operation is some sort of member access, e.g. `a.b()`
                        break;
                    }
                }

                self.scan_token()?;
                let mut arguments = Vec::new();
                while !(matches!(self.current_token(), Some(Token::ParenRight))) {
                    let argument = self.parse_expression(None, &[Token::Comma, Token::ParenRight])?;
                    arguments.push(argument);

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                self.scan_token()?;

                lhs = Box::new(Node::Call {
                    callee: lhs,
                    arguments,
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

    pub fn parse_value_type(&mut self, allowed_ends: &[Token]) -> crate::Result<ValueType> {
        let value_type = match self.get_token()? {
            Token::Literal(Literal::Identifier(name)) => {
                let name = name.clone();
                self.scan_token()?;
                ValueType::Named(name)
            },
            token => {
                return Err(self.scanner.syntax_error(format!("expected a type, got '{token}'")));
            }
        };

        self.expect_token(allowed_ends)?;

        Ok(value_type)
    }

    pub fn parse_statement(&mut self, is_global: bool, allow_empty: bool) -> crate::Result<Option<Box<Node>>> {
        match self.current_token() {
            Some(Token::Semicolon) if allow_empty => {
                self.scan_token()?;
                // Returning None implies that the end of the file is reached, so recursively try to parse a statement instead
                self.parse_statement(is_global, allow_empty)
            },
            Some(Token::Let) => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::Colon])?;
                self.scan_token()?;
                let value_type = self.parse_value_type(&[Token::Equal, Token::Semicolon])?;
                let value = if let Some(Token::Equal) = self.current_token() {
                    self.scan_token()?;
                    Some(self.parse_expression(None, &[Token::Semicolon])?)
                } else {
                    None
                };
                self.scan_token()?;

                Ok(Some(Box::new(Node::Let {
                    name,
                    value_type,
                    value,
                })))
            },
            Some(Token::Function) => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;

                let mut parameters = Vec::new();
                while !(matches!(self.current_token(), Some(Token::ParenRight))) {
                    let parameter_name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let parameter_type = self.parse_value_type(&[Token::Comma, Token::ParenRight])?;
                    parameters.push(FunctionParameter {
                        name: parameter_name,
                        value_type: parameter_type,
                    });

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }

                self.scan_token()?;
                self.expect_token(&[Token::RightArrow])?;
                self.scan_token()?;
                // The function body must be enclosed by a scope, so expect a '{' token following the return type
                let return_type = self.parse_value_type(&[Token::CurlyLeft])?;
                let body = self.parse_statement(false, false)?.unwrap();

                Ok(Some(Box::new(Node::Function {
                    name,
                    parameters,
                    return_type,
                    body,
                })))
            },
            Some(got_token) if is_global => {
                Err(self.scanner.syntax_error(expected_token_error_message(&[Token::Semicolon, Token::Let, Token::Function], got_token)))
            },
            Some(Token::CurlyLeft) => {
                self.scan_token()?;
                let mut statements = Vec::new();
                loop {
                    while let Some(Token::Semicolon) = self.current_token() {
                        self.scan_token()?;
                    }

                    match self.current_token() {
                        Some(Token::CurlyRight) => {
                            self.scan_token()?;
                            break;
                        },
                        None => {
                            return Err(self.scanner.syntax_error(String::from("expected closing '}'")));
                        },
                        _ => {
                            let statement = self.parse_statement(false, false)?
                                .ok_or_else(|| self.scanner.syntax_error(String::from("expected closing '}'")))?;
                            statements.push(statement);
                        }
                    }
                }

                Ok(Some(Box::new(Node::Scope {
                    statements,
                })))
            },
            Some(Token::If) => {
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;
                let condition = self.parse_expression(None, &[Token::ParenRight])?;
                self.scan_token()?;
                let consequent = self.parse_statement(is_global, false)?
                    .ok_or_else(|| self.scanner.syntax_error(String::from("expected a statement after 'if (<condition>)'")))?;
                let alternative;
                if let Some(Token::Else) = self.current_token() {
                    self.scan_token()?;
                    alternative = self.parse_statement(is_global, false)?;
                    if alternative.is_none() {
                        return Err(self.scanner.syntax_error(String::from("expected a statement after 'else'")));
                    }
                }
                else {
                    alternative = None;
                }

                Ok(Some(Box::new(Node::Conditional {
                    condition,
                    consequent,
                    alternative,
                })))
            },
            Some(Token::Else) => {
                Err(self.scanner.syntax_error(String::from("unexpected 'else' without previous 'if'")))
            },
            Some(Token::While) => {
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;
                let condition = self.parse_expression(None, &[Token::ParenRight])?;
                self.scan_token()?;
                let body = self.parse_statement(is_global, false)?
                    .ok_or_else(|| self.scanner.syntax_error(String::from("expected a statement after 'if (<condition>)'")))?;

                Ok(Some(Box::new(Node::While {
                    condition,
                    body,
                })))
            },
            Some(Token::Break) => {
                self.scan_token()?;
                self.expect_token(&[Token::Semicolon])?;
                self.scan_token()?;

                Ok(Some(Box::new(Node::Break)))
            },
            Some(Token::Continue) => {
                self.scan_token()?;
                self.expect_token(&[Token::Semicolon])?;
                self.scan_token()?;

                Ok(Some(Box::new(Node::Continue)))
            },
            Some(Token::Return) => {
                self.scan_token()?;
                let value;
                if let Some(Token::Semicolon) = self.current_token() {
                    value = None;
                }
                else {
                    value = Some(self.parse_expression(None, &[Token::Semicolon])?);
                }
                self.scan_token()?;

                Ok(Some(Box::new(Node::Return {
                    value,
                })))
            },
            Some(Token::Print) => {
                self.scan_token()?;
                let value = self.parse_expression(None, &[Token::Semicolon])?;
                self.scan_token()?;

                Ok(Some(Box::new(Node::Print {
                    value,
                })))
            },
            Some(_) => {
                let expression = self.parse_expression(None, &[Token::Semicolon])?;
                self.scan_token()?;

                Ok(Some(expression))
            },
            None => Ok(None),
        }
    }
}