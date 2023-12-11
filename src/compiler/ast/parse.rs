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

    pub fn parse_operand(&mut self, allowed_ends: &[Token]) -> crate::Result<Box<Node>> {
        let token = self.get_token()?;

        if let Some(operation) = UnaryOperation::from_prefix_token(token) {
            self.scan_token()?;
            let operand;
            match operation {
                UnaryOperation::GetSize => {
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    operand = Box::new(Node::Type(self.parse_type(&[Token::ParenRight])?));
                    self.scan_token()?;
                },
                UnaryOperation::GetAlign => {
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    operand = Box::new(Node::Type(self.parse_type(&[Token::ParenRight])?));
                    self.scan_token()?;
                },
                _ => {
                    operand = self.parse_expression(Some(Precedence::Prefix), allowed_ends)?;
                }
            };

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
                _ => {
                    return Err(self.scanner.syntax_error(format!("expected an operand, got {token}")));
                }
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
        let mut lhs = self.parse_operand(allowed_ends)?;

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
                        Box::new(Node::Type(self.parse_type(allowed_ends)?))
                    },
                    BinaryOperation::Subscript => {
                        let expression = self.parse_expression(None, &[Token::SquareRight])?;
                        self.scan_token()?;
                        expression
                    }
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

    pub fn parse_type(&mut self, allowed_ends: &[Token]) -> crate::Result<TypeNode> {
        match self.get_token()? {
            Token::Mut => {
                self.scan_token()?;
                let mutable_type = self.parse_unqualified_type(allowed_ends)?;
                Ok(TypeNode::Mutable(Box::new(mutable_type)))
            },
            _ => {
                self.parse_unqualified_type(allowed_ends)
            }
        }
    }

    pub fn parse_unqualified_type(&mut self, allowed_ends: &[Token]) -> crate::Result<TypeNode> {
        match self.get_token()? {
            Token::Literal(Literal::Identifier(name)) => {
                let name = name.clone();
                self.scan_token()?;
                self.expect_token(allowed_ends)?;

                Ok(TypeNode::Named(name))
            },
            Token::Star => {
                self.scan_token()?;
                let pointee_type = self.parse_type(allowed_ends)?;

                Ok(TypeNode::Pointer(Box::new(pointee_type)))
            },
            Token::SquareLeft => {
                self.scan_token()?;
                let item_type = Box::new(self.parse_unqualified_type(&[Token::Semicolon, Token::SquareRight])?);
                let length;
                if let Some(Token::Semicolon) = self.current_token() {
                    self.scan_token()?;
                    length = Some(self.parse_expression(None, &[Token::SquareRight])?);
                }
                else {
                    length = None;
                }
                self.scan_token()?;
                self.expect_token(allowed_ends)?;
                
                Ok(TypeNode::Array(item_type, length))
            },
            Token::Mut => {
                Err(self.scanner.syntax_error(format!("'mut' is not allowed here")))
            },
            token => {
                Err(self.scanner.syntax_error(format!("expected a type, got '{token}'")))
            }
        }
    }

    pub fn parse_statement(&mut self, is_global: bool, allow_empty: bool) -> crate::Result<Option<Box<Node>>> {
        match self.current_token() {
            Some(Token::Semicolon) if allow_empty => {
                self.scan_token()?;
                // Returning None would imply that the end of the file was reached, so recursively try to parse a statement instead
                self.parse_statement(is_global, allow_empty)
            },
            Some(Token::Let) => {
                self.scan_token()?;
                if let Some(Token::Const) = self.current_token() {
                    self.scan_token()?;
                    let name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let value_type = self.parse_type(&[Token::Equal])?;
                    self.scan_token()?;
                    let value = self.parse_expression(None, &[Token::Semicolon])?;
                    self.scan_token()?;

                    Ok(Some(Box::new(Node::Constant {
                        name,
                        value_type,
                        value,
                    })))
                }
                else {
                    let name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let value_type = self.parse_type(&[Token::Equal, Token::Semicolon])?;
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
                }
            },
            Some(Token::Function) => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;

                let mut parameters = Vec::new();
                let mut is_varargs = false;
                while !(matches!(self.current_token(), Some(Token::ParenRight))) {
                    if let Some(Token::Dot2) = self.current_token() {
                        is_varargs = true;
                        self.scan_token()?;
                        // The '..' for variadic arguments must be the end of the function signature
                        self.expect_token(&[Token::ParenRight])?;
                        break;
                    }

                    let parameter_name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let parameter_type = self.parse_type(&[Token::Comma, Token::ParenRight])?;
                    parameters.push((parameter_name, parameter_type));

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }

                self.scan_token()?;
                // The function body must be enclosed by a scope, so expect a '{' or ';' token following the return type (if present)
                self.expect_token(&[Token::RightArrow, Token::CurlyLeft, Token::Semicolon])?;
                let return_type = if let Some(Token::RightArrow) = self.current_token() {
                    self.scan_token()?;
                    self.parse_type(&[Token::CurlyLeft, Token::Semicolon])?
                } else {
                    TypeNode::Named(String::from("void"))
                };
                let body = if let Some(Token::CurlyLeft) = self.current_token() {
                    self.parse_statement(false, false)?
                } else {
                    self.scan_token()?;
                    None
                };

                Ok(Some(Box::new(Node::Function {
                    name,
                    parameters,
                    is_varargs,
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
            Some(_) => {
                let expression = self.parse_expression(None, &[Token::Semicolon])?;
                self.scan_token()?;

                Ok(Some(expression))
            },
            None => Ok(None),
        }
    }
}