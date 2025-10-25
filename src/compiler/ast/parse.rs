use super::*;

use std::io::BufRead;
use crate::sema::{GlobalContext, PrimitiveType, Symbol};
use crate::token::scan::Scanner;

pub fn parse_all<T: BufRead>(scanner: &mut Scanner<T>, context: &mut GlobalContext) -> crate::Result<Vec<Node>> {
    let mut parser = Parser::new(scanner)?;

    let mut top_level_statements = Vec::new();
    while let Some(statement) = parser.parse_top_level_statement(context)? {
        top_level_statements.push(*statement);
    }

    Ok(top_level_statements)
}

pub struct Parser<'a, T: BufRead> {
    scanner: &'a mut Scanner<T>,
    current_span: crate::Span,
    current_token: Option<Token>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(scanner: &'a mut Scanner<T>) -> crate::Result<Self> {
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
            Token::Literal(Literal::Name(name)) => Ok(name.clone()),
            _ => Err(Box::new(crate::Error::ExpectedIdentifier { span: self.current_span() }))
        }
    }

    pub fn parse_path(&mut self, first_segment: Option<PathSegment>, allow_glob: bool) -> crate::Result<(Vec<PathSegment>, bool)> {
        let mut segments = Vec::from_iter(first_segment);
        let mut is_glob = false;

        loop {
            match self.get_token()? {
                Token::Literal(Literal::Name(name)) => {
                    segments.push(PathSegment::Name(name.clone()));
                }
                Token::Colon2 if segments.is_empty() => {
                    segments.push(PathSegment::RootModule);
                    continue;
                }
                Token::Super => {
                    segments.push(PathSegment::SuperModule);
                }
                Token::Module => {
                    segments.push(PathSegment::SelfModule);
                }
                Token::SelfType if segments.is_empty() => {
                    segments.push(PathSegment::SelfType);
                }
                Token::Literal(Literal::PrimitiveType(primitive_type)) if segments.is_empty() => {
                    segments.push(PathSegment::PrimitiveType(*primitive_type));
                }
                Token::AngleLeft if segments.is_empty() => {
                    self.scan_token()?;
                    let type_node = self.parse_type(Some(&[Token::AngleRight]))?;
                    segments.push(PathSegment::Type(Box::new(type_node)));
                }
                Token::Star if allow_glob => {
                    is_glob = true;
                }
                got_token => return Err(Box::new(crate::Error::ExpectedType {
                    span: self.current_span(),
                    got_token: got_token.clone(),
                }))
            }

            self.scan_token()?;
            let Some(Token::Colon2) = self.current_token() else {
                break;
            };
            if is_glob {
                return Err(Box::new(crate::Error::InvalidGlobPath {
                    span: self.current_span(),
                }))
            }
            self.scan_token()?;
        }

        Ok((segments, is_glob))
    }

    pub fn parse_operand(&mut self, allowed_ends: &[Token], strict_ends: bool) -> crate::Result<Box<Node>> {
        let token = self.get_token()?;

        if let Some(operation) = UnaryOperation::from_prefix_token(token) {
            self.scan_token()?;
            let operand;
            match operation {
                UnaryOperation::GetSize | UnaryOperation::GetAlign => {
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    operand = Box::new(Node::Type(self.parse_type(Some(&[Token::ParenRight]))?));
                    self.scan_token()?;
                }
                _ => {
                    operand = self.parse_expression(Some(Precedence::Prefix), allowed_ends, strict_ends)?;
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
                    let content = self.parse_expression(None, &[Token::ParenRight], true)?;
                    self.scan_token()?;

                    Box::new(Node::Grouping {
                        content,
                    })
                }
                Token::Literal(literal) => {
                    let literal = literal.clone();
                    let literal_span = self.current_span();
                    self.scan_token()?;

                    if let Some(Token::Colon2) = self.current_token() {
                        let first_segment = match literal {
                            Literal::Name(name) => PathSegment::Name(name),
                            Literal::PrimitiveType(primitive_type) => PathSegment::PrimitiveType(primitive_type),
                            _ => return Err(Box::new(crate::Error::ExpectedIdentifier {
                                span: literal_span,
                            }))
                        };
                        self.scan_token()?;

                        Box::new(Node::Path {
                            segments: self.parse_path(Some(first_segment), false)?.0,
                        })
                    }
                    else {
                        Box::new(Node::Literal(literal))
                    }
                }
                Token::SquareLeft => {
                    self.scan_token()?;
                    let mut items = Vec::new();
                    while !matches!(self.current_token(), Some(Token::SquareRight)) {
                        let item = self.parse_expression(None, &[Token::Comma, Token::SquareRight], true)?;
                        items.push(item);

                        if let Some(Token::Comma) = self.current_token() {
                            self.scan_token()?;
                        }
                    }
                    self.scan_token()?;

                    Box::new(Node::ArrayLiteral {
                        items,
                    })
                }
                Token::Colon2 | Token::Super | Token::Module | Token::SelfType | Token::AngleLeft => {
                    Box::new(Node::Path {
                        segments: self.parse_path(None, false)?.0,
                    })
                }
                Token::CurlyLeft => {
                    self.scan_token()?;
                    self.parse_scope()?
                }
                Token::If => {
                    self.scan_token()?;
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    let condition = self.parse_expression(None, &[Token::ParenRight], true)?;
                    self.scan_token()?;
                    let consequent_ends: Vec<Token> = allowed_ends
                        .iter()
                        .cloned()
                        .chain(std::iter::once(Token::Else))
                        .collect();
                    let consequent = self.parse_expression(None, &consequent_ends, strict_ends)?;
                    let alternative = if let Some(Token::Else) = self.current_token() {
                        self.scan_token()?;
                        Some(self.parse_expression(None, allowed_ends, strict_ends)?)
                    } else {
                        None
                    };

                    Box::new(Node::Conditional {
                        condition,
                        consequent,
                        alternative,
                    })
                }
                Token::Else => {
                    return Err(Box::new(crate::Error::UnexpectedElse {
                        span: self.current_span(),
                    }));
                }
                Token::While => {
                    self.scan_token()?;
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    let condition = self.parse_expression(None, &[Token::ParenRight], true)?;
                    self.scan_token()?;
                    let body = self.parse_expression(None, allowed_ends, strict_ends)?;

                    Box::new(Node::While {
                        condition,
                        body,
                    })
                }
                Token::Break => {
                    self.scan_token()?;
                    Box::new(Node::Break)
                }
                Token::Continue => {
                    self.scan_token()?;
                    Box::new(Node::Continue)
                }
                Token::Return => {
                    self.scan_token()?;
                    let value = if allowed_ends.contains(self.get_token()?) {
                        None
                    } else {
                        Some(self.parse_expression(None, allowed_ends, strict_ends)?)
                    };
                    Box::new(Node::Return {
                        value,
                    })
                }
                _ => {
                    return Err(Box::new(crate::Error::ExpectedOperand {
                        span: self.current_span(),
                        got_token: token.clone(),
                    }));
                }
            };

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

    fn parse_scope(&mut self) -> crate::Result<Box<Node>> {
        let mut statements = Vec::new();
        let tail = loop {
            while let Some(Token::Semicolon) = self.current_token() {
                self.scan_token()?;
            }

            match self.current_token() {
                Some(Token::CurlyRight) => {
                    self.scan_token()?;
                    break None;
                }
                Some(Token::Let) => {
                    self.scan_token()?;
                    statements.push(self.parse_let_statement()?);
                }
                Some(..) => {
                    let statement = self.parse_expression(None, &[Token::Semicolon, Token::CurlyRight], false)?;
                    if let Some(Token::CurlyRight) = self.current_token() {
                        self.scan_token()?;
                        break Some(statement);
                    }
                    else {
                        statements.push(statement);
                    }
                }
                None => {
                    return Err(Box::new(crate::Error::ExpectedClosingBracket {
                        span: self.current_span(),
                        bracket: Token::CurlyRight,
                    }));
                }
            }
        };

        Ok(Box::new(Node::Scope {
            statements,
            tail,
        }))
    }

    pub fn parse_expression(&mut self, parent_precedence: Option<Precedence>, allowed_ends: &[Token], strict_ends: bool) -> crate::Result<Box<Node>> {
        let mut lhs = self.parse_operand(allowed_ends, strict_ends)?;

        while let Some(token) = self.current_token() {
            if allowed_ends.contains(token) || !lhs.requires_semicolon() {
                // Allowed ends are checked before operations, even if one of the allowed ends
                // doubles as a valid operator. Also, ensure that this isn't a location where an
                // implicit semicolon could go-- if it is, it's safer to insert a semicolon. To get
                // around this, the programmer could put the left-hand side inside parentheses.
                break;
            }
            else if let Some(operation) = BinaryOperation::from_token(token) {
                let precedence = operation.precedence();

                if parent_precedence.is_some_and(|parent_precedence| parent_precedence > precedence || (
                    parent_precedence == precedence && precedence.associativity() == Associativity::LeftToRight
                )) {
                    // Parent operation has a higher precedence and therefore must be made into a
                    // subtree of the next operation. If the parent operation has the same
                    // precedence, only make it a subtree when there is left-to-right associativity.
                    break;
                }

                self.scan_token()?;
                let rhs = match operation {
                    BinaryOperation::Convert => {
                        Box::new(Node::Type(self.parse_type(None)?))
                    },
                    BinaryOperation::Subscript => {
                        let expression = self.parse_expression(None, &[Token::SquareRight], true)?;
                        self.scan_token()?;
                        expression
                    },
                    _ => {
                        self.parse_expression(Some(precedence), allowed_ends, strict_ends)?
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
                        // Parent operation should be made into a subtree of the call operation
                        // (which has postfix precedence). Usually the parent operation is some
                        // sort of access operation for a method call, e.g. `a.b()`
                        break;
                    }
                }

                self.scan_token()?;
                let mut arguments = Vec::new();
                while !matches!(self.current_token(), Some(Token::ParenRight)) {
                    let argument = self.parse_expression(None, &[Token::Comma, Token::ParenRight], true)?;
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
                while !matches!(self.current_token(), Some(Token::CurlyRight)) {
                    let member_name = self.expect_identifier()?;
                    self.scan_token()?;
                    self.expect_token(&[Token::Colon])?;
                    self.scan_token()?;
                    let member_value = self.parse_expression(None, &[Token::Comma, Token::CurlyRight], true)?;
                    members.push((member_name, member_value));

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                self.scan_token()?;

                lhs = Box::new(Node::StructureLiteral {
                    structure_type: lhs,
                    members,
                });
            }
            else {
                return Err(Box::new(crate::Error::ExpectedOperation {
                    span: self.current_span(),
                    got_token: token.clone(),
                }));
            }
        }

        if parent_precedence.is_none() {
            // Check that we have arrived at one of the allowed ending tokens.
            // This doesn't apply to expressions with implicitly inserted semicolons, though.
            if let Err(error) = self.expect_token(allowed_ends) {
                if strict_ends || lhs.requires_semicolon() {
                    return Err(error);
                }
            }
        }

        Ok(lhs)
    }

    pub fn parse_type(&mut self, allowed_ends: Option<&[Token]>) -> crate::Result<TypeNode> {
        match self.get_token()? {
            Token::Star => {
                self.scan_token()?;
                let semantics = match self.current_token() {
                    Some(Token::Mut) => {
                        self.scan_token()?;
                        PointerSemantics::Mutable
                    }
                    _ => {
                        PointerSemantics::Immutable
                    }
                };
                let pointee_type = Box::new(self.parse_type(allowed_ends)?);

                Ok(TypeNode::Pointer {
                    pointee_type,
                    semantics,
                })
            }
            Token::SquareLeft => {
                self.scan_token()?;
                let item_type = Box::new(self.parse_type(Some(&[Token::Semicolon, Token::SquareRight]))?);
                let length;
                if let Some(Token::Semicolon) = self.current_token() {
                    self.scan_token()?;
                    length = Some(self.parse_expression(None, &[Token::SquareRight], true)?);
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
            }
            Token::Function => {
                self.scan_token()?;
                self.expect_token(&[Token::ParenLeft])?;
                self.scan_token()?;

                let mut parameter_types = Vec::new();
                let mut is_varargs = false;
                while !matches!(self.current_token(), Some(Token::ParenRight)) {
                    if let Some(Token::Dot2) = self.current_token() {
                        is_varargs = true;
                        self.scan_token()?;
                        // The '..' for variadic arguments must be the end of the function signature
                        self.expect_token(&[Token::ParenRight])?;
                        break;
                    }

                    parameter_types.push(self.parse_type(Some(&[Token::Comma, Token::ParenRight]))?);

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }

                let return_type;
                self.scan_token()?;
                if let Some(Token::RightArrow) = self.current_token() {
                    self.scan_token()?;
                    return_type = Box::new(self.parse_type(allowed_ends)?);
                }
                else {
                    return_type = Box::new(TypeNode::Path {
                        segments: vec![PathSegment::PrimitiveType(crate::sema::PrimitiveType {
                            name: "void",
                            handle: crate::sema::TypeHandle::VOID
                        })],
                    });

                    if let Some(allowed_ends) = allowed_ends {
                        self.expect_token(allowed_ends)?;
                    }
                }

                Ok(TypeNode::Function {
                    parameter_types,
                    is_variadic: is_varargs,
                    return_type,
                })
            }
            Token::Mut => {
                Err(Box::new(crate::Error::UnexpectedQualifier {
                    span: self.current_span(),
                    got_token: Token::Mut,
                }))
            }
            _ => {
                let segments = self.parse_path(None, false)?.0;

                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }

                Ok(TypeNode::Path {
                    segments,
                })
            }
        }
    }

    fn parse_let_statement(&mut self) -> crate::Result<Box<Node>> {
        if let Some(Token::Const) = self.current_token() {
            self.scan_token()?;
            let name = self.expect_identifier()?;
            self.scan_token()?;
            self.expect_token(&[Token::Colon])?;
            self.scan_token()?;
            let value_type = self.parse_type(Some(&[Token::Equal]))?;
            self.scan_token()?;
            let value = self.parse_expression(None, &[Token::Semicolon], true)?;
            self.scan_token()?;

            Ok(Box::new(Node::Constant {
                name,
                value_type,
                value,
                global_register: None,
            }))
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
            self.expect_token(&[Token::Colon, Token::Equal, Token::Semicolon])?;
            let value_type = if let Some(Token::Colon) = self.current_token() {
                self.scan_token()?;
                Some(self.parse_type(Some(&[Token::Equal, Token::Semicolon]))?)
            } else {
                None
            };
            let value = if let Some(Token::Equal) = self.current_token() {
                self.scan_token()?;
                Some(self.parse_expression(None, &[Token::Semicolon], true)?)
            } else {
                None
            };
            self.scan_token()?;

            Ok(Box::new(Node::Let {
                name,
                value_type,
                is_mutable,
                value,
                global_register: None,
            }))
        }
    }

    fn parse_function_definition(&mut self, is_foreign: bool) -> crate::Result<Box<Node>> {
        let name = self.expect_identifier()?;
        self.scan_token()?;
        self.expect_token(&[Token::ParenLeft])?;
        self.scan_token()?;

        let mut parameters = Vec::new();
        let mut is_variadic = false;
        while !matches!(self.current_token(), Some(Token::ParenRight)) {
            if let Some(Token::Dot2) = self.current_token() {
                is_variadic = true;
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

            parameters.push(FunctionParameterNode {
                name: parameter_name,
                type_node: parameter_type,
                is_mutable,
            });

            if let Some(Token::Comma) = self.current_token() {
                self.scan_token()?;
            }
        }

        self.scan_token()?;
        // The function body must be enclosed by a scope, so expect a '{' or ';' token following
        // the return type (if present)
        if is_foreign {
            self.expect_token(&[Token::RightArrow, Token::CurlyLeft, Token::Semicolon])?;
        }
        else {
            self.expect_token(&[Token::RightArrow, Token::CurlyLeft])?;
        }
        let return_type = if let Some(Token::RightArrow) = self.current_token() {
            self.scan_token()?;
            if is_foreign {
                self.parse_type(Some(&[Token::CurlyLeft, Token::Semicolon]))?
            }
            else {
                self.parse_type(Some(&[Token::CurlyLeft]))?
            }
        } else {
            TypeNode::Path {
                segments: vec![PathSegment::PrimitiveType(PrimitiveType {
                    name: "void",
                    handle: TypeHandle::VOID
                })],
            }
        };

        let body = if let Some(Token::CurlyLeft) = self.current_token() {
            self.scan_token()?;
            Some(self.parse_scope()?)
        } else {
            self.scan_token()?;
            None
        };

        Ok(Box::new(Node::Function {
            name,
            is_foreign,
            parameters,
            is_variadic,
            return_type,
            body,
            global_register: None,
        }))
    }

    fn parse_structure_definition(&mut self, context: &mut GlobalContext, is_foreign: bool) -> crate::Result<Box<Node>> {
        let name = self.expect_identifier()?;

        let self_type = context.outline_structure_type(name.clone())?;
        context.set_self_type(self_type);

        self.scan_token()?;
        if is_foreign {
            self.expect_token(&[Token::CurlyLeft, Token::Semicolon])?;
        }
        else {
            self.expect_token(&[Token::CurlyLeft])?;
        }
        let members = if let Some(Token::CurlyLeft) = self.current_token() {
            self.scan_token()?;

            let mut members = Vec::new();
            while !matches!(self.current_token(), Some(Token::CurlyRight)) {
                let member_name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::Colon])?;
                self.scan_token()?;
                let member_type = self.parse_type(Some(&[Token::Comma, Token::CurlyRight]))?;

                members.push(StructureMemberNode {
                    name: member_name,
                    type_node: member_type,
                });

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

        context.unset_self_type();

        Ok(Box::new(Node::Structure {
            name,
            is_foreign,
            members,
            self_type,
        }))
    }

    pub fn parse_global_statement(&mut self, global_context: &mut GlobalContext, is_implementation: bool, allow_empty: bool) -> crate::Result<Option<Box<Node>>> {
        // Most statement types can be detected simply by the first token
        match self.current_token() {
            Some(Token::Semicolon) if allow_empty => {
                self.scan_token()?;
                // Returning None would imply that the end of the file was reached,
                // so recursively try to parse a statement instead
                self.parse_global_statement(global_context, is_implementation, allow_empty)
            }
            Some(Token::Let) => {
                self.scan_token()?;
                self.parse_let_statement().map(Some)
            }
            Some(Token::Function) => {
                self.scan_token()?;
                self.parse_function_definition(false).map(Some)
            }
            Some(Token::Struct) if !is_implementation => {
                self.scan_token()?;
                self.parse_structure_definition(global_context, false).map(Some)
            }
            Some(Token::Implement) if !is_implementation => {
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
                        }
                        None => {
                            return Err(Box::new(crate::Error::ExpectedClosingBracket {
                                span: self.current_span(),
                                bracket: Token::CurlyRight,
                            }));
                        }
                        _ => {
                            let statement = self.parse_global_statement(global_context, true, true)?
                                .ok_or_else(|| Box::new(crate::Error::ExpectedClosingBracket {
                                    span: self.current_span(),
                                    bracket: Token::CurlyRight,
                                }))?;
                            statements.push(statement);
                        }
                    }
                }

                Ok(Some(Box::new(Node::Implement {
                    self_type,
                    statements,
                })))
            }
            Some(Token::Module) if !is_implementation => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                self.scan_token()?;
                self.expect_token(&[Token::CurlyLeft])?;
                self.scan_token()?;

                let namespace = global_context.enter_module_outline(&name)?;

                let mut statements = Vec::new();
                loop {
                    while let Some(Token::Semicolon) = self.current_token() {
                        self.scan_token()?;
                    }

                    match self.current_token() {
                        Some(Token::CurlyRight) => {
                            self.scan_token()?;
                            break;
                        }
                        None => {
                            return Err(Box::new(crate::Error::ExpectedClosingBracket {
                                span: self.current_span(),
                                bracket: Token::CurlyRight,
                            }));
                        }
                        _ => {
                            let statement = self.parse_global_statement(global_context, false, true)?
                                .ok_or_else(|| Box::new(crate::Error::ExpectedClosingBracket {
                                    span: self.current_span(),
                                    bracket: Token::CurlyRight,
                                }))?;
                            statements.push(statement);
                        }
                    }
                }

                global_context.exit_module();

                Ok(Some(Box::new(Node::Module {
                    name,
                    statements,
                    namespace,
                })))
            }
            Some(Token::Import) if !is_implementation => {
                self.scan_token()?;
                let (segments, is_glob_path) = self.parse_path(None, true)?;

                if is_glob_path {
                    self.expect_token(&[Token::Semicolon])?;
                    self.scan_token()?;

                    let path = global_context.get_absolute_path(&segments)?;
                    global_context.current_module_info_mut().add_glob_import(path);

                    Ok(Some(Box::new(Node::GlobImport {
                        segments,
                    })))
                }
                else {
                    self.expect_token(&[Token::As, Token::Semicolon])?;

                    let mut alias = None;
                    if let Some(Token::As) = self.current_token() {
                        self.scan_token()?;
                        alias = Some(self.expect_identifier()?);
                        self.scan_token()?;
                        self.expect_token(&[Token::Semicolon])?;
                    }
                    self.scan_token()?;

                    let path = global_context.get_absolute_path(&segments)?;
                    let import_name = match alias.as_ref() {
                        Some(alias) => alias.as_str(),
                        None => match path.tail_name() {
                            Some(name) => name,
                            None => {
                                return Err(Box::new(crate::Error::ImportAliasRequired {
                                    path: path.to_string(),
                                }));
                            }
                        }
                    };
                    // Establish an alias symbol in the current module corresponding to this import
                    global_context.current_module_info_mut().define(
                        import_name,
                        Symbol::Alias(path.clone()),
                    )?;

                    Ok(Some(Box::new(Node::Import {
                        segments,
                        alias,
                    })))
                }
            }
            Some(Token::Foreign) => {
                self.scan_token()?;
                match self.get_token()? {
                    Token::Function => {
                        self.scan_token()?;
                        self.parse_function_definition(true).map(Some)
                    }
                    Token::Struct if !is_implementation => {
                        self.scan_token()?;
                        self.parse_structure_definition(global_context, true).map(Some)
                    }
                    got_token => {
                        let mut allowed_tokens = vec![Token::Function];
                        if !is_implementation {
                            allowed_tokens.push(Token::Struct);
                        }
                        Err(Box::new(crate::Error::ExpectedTokenFromList {
                            span: self.current_span(),
                            got_token: got_token.clone(),
                            allowed_tokens,
                        }))
                    }
                }
            }
            Some(got_token) => {
                Err(Box::new(crate::Error::ExpectedTokenFromList {
                    span: self.current_span(),
                    got_token: got_token.clone(),
                    // Semicolon is technically allowed, but like... why would you do that
                    allowed_tokens: vec![
                        Token::Let,
                        Token::Function,
                        Token::Struct,
                        Token::Implement,
                        Token::Module,
                        Token::Import,
                        Token::Foreign,
                    ],
                }))
            }
            None => Ok(None),
        }
    }

    pub fn parse_top_level_statement(&mut self, context: &mut GlobalContext) -> crate::Result<Option<Box<Node>>> {
        self.parse_global_statement(context, false, true)
    }
}
