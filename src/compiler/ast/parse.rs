use super::*;

use std::io::BufRead;

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    scanner: &'a mut scan::Scanner<T>,
    current_span: crate::Span,
    current_token: Option<Token>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(scanner: &'a mut scan::Scanner<T>) -> crate::Result<Self> {
        let current_span = scanner.create_span(scanner.next_index(), scanner.next_index());
        let mut new_instance = Self {
            scanner,
            current_span,
            current_token: None,
        };
        new_instance.scan_token()?;
        Ok(new_instance)
    }

    pub fn file_id(&self) -> usize {
        self.scanner.file_id()
    }

    pub fn scan_token(&mut self) -> crate::Result<()> {
        if let Some((span, token)) = self.scanner.next_token()? {
            self.current_span = span;
            self.current_token = Some(token);
        }
        else {
            self.current_span = self.scanner.create_span(self.scanner.next_index(), self.scanner.next_index());
            self.current_token = None;
        }
        Ok(())
    }

    pub fn current_span(&self) -> crate::Span {
        self.current_span.clone()
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn get_token(&self) -> crate::Result<&Token> {
        self.current_token().ok_or_else(|| Box::new(crate::Error::ExpectedToken { span: self.current_span() }))
    }

    pub fn expect_token<'b>(&self, allowed: &'b [Token]) -> crate::Result<&'b Token> {
        let current_token = self.get_token()?;
        allowed.iter()
            .find(|&token| token == current_token)
            .ok_or_else(|| Box::new(crate::Error::ExpectedTokenFromList {
                span: self.current_span(),
                got_token: current_token.clone(),
                allowed_tokens: allowed.to_vec(),
            }))
    }

    pub fn expect_identifier(&self) -> crate::Result<String> {
        match self.get_token()? {
            Token::Literal(Literal::Identifier(name)) => Ok(name.clone()),
            _ => Err(Box::new(crate::Error::ExpectedIdentifier { span: self.current_span() }))
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
                    operand = Box::new(Node::Type(self.parse_type(Some(&[Token::ParenRight]))?));
                    self.scan_token()?;
                },
                UnaryOperation::GetAlign => {
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    operand = Box::new(Node::Type(self.parse_type(Some(&[Token::ParenRight]))?));
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
                Token::SquareLeft => {
                    self.scan_token()?;
                    let mut items = Vec::new();
                    while !(matches!(self.current_token(), Some(Token::SquareRight))) {
                        let item = self.parse_expression(None, &[Token::Comma, Token::SquareRight])?;
                        items.push(item);

                        if let Some(Token::Comma) = self.current_token() {
                            self.scan_token()?;
                        }
                    }

                    Box::new(Node::ArrayLiteral {
                        items,
                    })
                },
                _ => {
                    return Err(Box::new(crate::Error::ExpectedOperand { span: self.current_span(), got_token: token.clone() }));
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
                        Box::new(Node::Type(self.parse_type(None)?))
                    },
                    BinaryOperation::Subscript => {
                        let expression = self.parse_expression(None, &[Token::SquareRight])?;
                        self.scan_token()?;
                        expression
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
                // Left parenthesis indicates a function call
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
            else if let Token::CurlyLeft = token {
                // Left curly brace indicates a structure literal
                if let Some(parent_precedence) = parent_precedence {
                    if parent_precedence >= Precedence::Postfix {
                        // Parent operation should be made into a subtree of the operation (which has postfix precedence)
                        // Usually the parent operation is a static member access, e.g. `my::Struct {}`
                        break;
                    }
                }

                self.scan_token()?;
                let mut members = Vec::new();
                while !(matches!(self.current_token(), Some(Token::CurlyRight))) {
                    let member_name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let member_value = self.parse_expression(None, &[Token::Comma, Token::CurlyRight])?;
                    members.push((member_name, member_value));

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                self.scan_token()?;

                lhs = Box::new(Node::StructureLiteral {
                    type_name: lhs,
                    members,
                });
            }
            else {
                return Err(Box::new(crate::Error::ExpectedOperation { span: self.current_span(), got_token: token.clone() }));
            }
        }

        if parent_precedence.is_none() {
            self.expect_token(allowed_ends)?;
        }

        Ok(lhs)
    }

    pub fn parse_type(&mut self, allowed_ends: Option<&[Token]>) -> crate::Result<TypeNode> {
        match self.get_token()? {
            Token::Literal(Literal::Identifier(name)) => {
                let name = name.clone();
                self.scan_token()?;
                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }

                Ok(TypeNode::Identified {
                    type_name: name,
                })
            },
            Token::Star => {
                self.scan_token()?;
                let semantics = match self.current_token() {
                    Some(Token::Mut) => {
                        self.scan_token()?;
                        PointerSemantics::Mutable
                    },
                    Some(Token::Own) => {
                        self.scan_token()?;
                        PointerSemantics::Owned
                    },
                    _ => {
                        PointerSemantics::Immutable
                    }
                };
                let pointee_type = Box::new(self.parse_type(allowed_ends)?);

                Ok(TypeNode::Pointer {
                    pointee_type,
                    semantics,
                })
            },
            Token::SquareLeft => {
                self.scan_token()?;
                let item_type = Box::new(self.parse_type(Some(&[Token::Semicolon, Token::SquareRight]))?);
                let length;
                if let Some(Token::Semicolon) = self.current_token() {
                    self.scan_token()?;
                    length = Some(self.parse_expression(None, &[Token::SquareRight])?);
                }
                else {
                    length = None;
                }
                self.scan_token()?;
                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }
                
                Ok(TypeNode::Array {
                    item_type,
                    length,
                })
            },
            Token::Mut | Token::Own => {
                Err(Box::new(crate::Error::UnexpectedQualifier { span: self.current_span(), got_token: self.get_token()?.clone() }))
            },
            got_token => {
                Err(Box::new(crate::Error::ExpectedType { span: self.current_span(), got_token: got_token.clone() }))
            }
        }
    }

    pub fn parse_statement(&mut self, is_global: bool, is_in_implement_block: bool, allow_empty: bool) -> crate::Result<Option<Box<Node>>> {
        match self.current_token() {
            Some(Token::Semicolon) if allow_empty => {
                self.scan_token()?;
                // Returning None would imply that the end of the file was reached, so recursively try to parse a statement instead
                self.parse_statement(is_global, is_in_implement_block, allow_empty)
            },
            Some(Token::Let) => {
                self.scan_token()?;
                if let Some(Token::Const) = self.current_token() {
                    self.scan_token()?;
                    let name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let value_type = self.parse_type(Some(&[Token::Equal]))?;
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
                    let is_mutable = if let Some(Token::Mut) = self.current_token() {
                        self.scan_token()?;
                        true
                    } else {
                        false
                    };

                    let name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let value_type = self.parse_type(Some(&[Token::Equal, Token::Semicolon]))?;
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
                        is_mutable,
                        value,
                    })))
                }
            },
            Some(Token::Function) if is_global => {
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

                    let is_mutable = if let Some(Token::Mut) = self.current_token() {
                        self.scan_token()?;
                        true
                    } else {
                        false
                    };

                    let parameter_name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let parameter_type = self.parse_type(Some(&[Token::Comma, Token::ParenRight]))?;
                    parameters.push(FunctionParameter {
                        name: parameter_name,
                        type_node: parameter_type,
                        is_mutable,
                    });

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }

                self.scan_token()?;
                // The function body must be enclosed by a scope, so expect a '{' or ';' token following the return type (if present)
                self.expect_token(&[Token::RightArrow, Token::CurlyLeft, Token::Semicolon])?;
                let return_type = if let Some(Token::RightArrow) = self.current_token() {
                    self.scan_token()?;
                    self.parse_type(Some(&[Token::CurlyLeft, Token::Semicolon]))?
                } else {
                    TypeNode::Identified {
                        type_name: "void".into(),
                    }
                };
                let body = if let Some(Token::CurlyLeft) = self.current_token() {
                    self.parse_statement(false, is_in_implement_block, false)?
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
            Some(Token::Struct) if is_global => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::CurlyLeft, Token::Semicolon])?;
                let members = if let Some(Token::CurlyLeft) = self.current_token() {
                    self.scan_token()?;

                    let mut members = Vec::new();
                    while !(matches!(self.current_token(), Some(Token::CurlyRight))) {
                        let member_name = self.expect_identifier()?;
                        self.scan_token()?;
                        self.expect_token(&[Token::Colon])?;
                        self.scan_token()?;
                        let member_type = self.parse_type(Some(&[Token::Comma, Token::CurlyRight]))?;
                        members.push((member_name, member_type));

                        if let Some(Token::Comma) = self.current_token() {
                            self.scan_token()?;
                        }
                    }

                    self.scan_token()?;
                    Some(members)
                }
                else {
                    self.scan_token()?;
                    None
                };

                Ok(Some(Box::new(Node::Structure {
                    name,
                    members,
                })))
            },
            Some(Token::Implement) if is_global && !is_in_implement_block => {
                self.scan_token()?;
                let self_type = self.parse_type(Some(&[Token::CurlyLeft]))?;
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
                            return Err(Box::new(crate::Error::ExpectedClosingBracket { span: self.current_span(), bracket: Token::CurlyRight }));
                        },
                        _ => {
                            let statement = self.parse_statement(is_global, true, false)?
                                .ok_or_else(|| Box::new(crate::Error::ExpectedClosingBracket { span: self.current_span(), bracket: Token::CurlyRight }))?;
                            statements.push(statement);
                        }
                    }
                }

                Ok(Some(Box::new(Node::Implement {
                    self_type,
                    statements,
                })))
            },
            Some(got_token) if is_global => {
                Err(Box::new(crate::Error::ExpectedTokenFromList {
                    span: self.current_span(),
                    got_token: got_token.clone(),
                    // Semicolon is technically allowed, but like... why would you do that
                    allowed_tokens: vec![Token::Let, Token::Function, Token::Struct, Token::Implement],
                }))
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
                            return Err(Box::new(crate::Error::ExpectedClosingBracket { span: self.current_span(), bracket: Token::CurlyRight }));
                        },
                        _ => {
                            let statement = self.parse_statement(is_global, is_in_implement_block, false)?
                                .ok_or_else(|| Box::new(crate::Error::ExpectedClosingBracket { span: self.current_span(), bracket: Token::CurlyRight }))?;
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
                let consequent = self.parse_statement(is_global, is_in_implement_block, false)?
                    .ok_or_else(|| Box::new(crate::Error::ExpectedStatement { span: self.current_span() }))?;
                let alternative;
                if let Some(Token::Else) = self.current_token() {
                    self.scan_token()?;
                    alternative = self.parse_statement(is_global, is_in_implement_block, false)?;
                    if alternative.is_none() {
                        return Err(Box::new(crate::Error::ExpectedStatement { span: self.current_span() }));
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
                Err(Box::new(crate::Error::UnexpectedElse { span: self.current_span() }))
            },
            Some(Token::While) => {
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;
                let condition = self.parse_expression(None, &[Token::ParenRight])?;
                self.scan_token()?;
                let body = self.parse_statement(is_global, is_in_implement_block, false)?
                    .ok_or_else(|| Box::new(crate::Error::ExpectedStatement { span: self.current_span() }))?;

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

    pub fn parse_top_level_statement(&mut self) -> crate::Result<Option<Box<Node>>> {
        self.parse_statement(true, false, true)
    }
}