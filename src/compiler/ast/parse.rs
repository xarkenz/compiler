use super::*;

use std::io::BufRead;
use crate::sema::{GlobalContext, PrimitiveType, Symbol};
use crate::token::scan::Scanner;

pub fn parse_module<T: BufRead>(scanner: &mut Scanner<T>, context: &mut GlobalContext, namespace: NamespaceHandle) -> crate::Result<ParsedModule> {
    let mut parser = Parser::new(scanner)?;

    let previous_module = context.replace_current_module(namespace);

    let mut statements = Vec::new();
    while let Some(statement) = parser.parse_top_level_statement(context)? {
        statements.push(*statement);
    }

    context.replace_current_module(previous_module);

    Ok(ParsedModule {
        statements,
        namespace,
    })
}

pub struct ParsedModule {
    statements: Vec<Node>,
    namespace: NamespaceHandle,
}

impl ParsedModule {
    pub fn statements(&self) -> &[Node] {
        &self.statements
    }

    pub fn statements_mut(&mut self) -> &mut [Node] {
        &mut self.statements
    }

    pub fn namespace(&self) -> NamespaceHandle {
        self.namespace
    }
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

    pub fn source_id(&self) -> usize {
        self.scanner.source_id()
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
        self.current_span
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn get_token(&self) -> crate::Result<&Token> {
        self.current_token().ok_or_else(|| Box::new(crate::Error::new(
            Some(self.current_span()),
            crate::ErrorKind::ExpectedToken,
        )))
    }

    pub fn expect_token<'b>(&self, allowed: &'b [Token]) -> crate::Result<&'b Token> {
        let current_token = self.get_token()?;
        allowed.iter()
            .find(|&token| token == current_token)
            .ok_or_else(|| Box::new(crate::Error::new(
                Some(self.current_span()),
                crate::ErrorKind::ExpectedTokenFromList {
                    got_token: current_token.clone(),
                    allowed_tokens: allowed.to_vec(),
                },
            )))
    }

    pub fn expect_identifier(&self) -> crate::Result<Box<str>> {
        match self.get_token()? {
            Token::Literal(Literal::Name(name)) => Ok(name.clone()),
            _ => Err(Box::new(crate::Error::new(
                Some(self.current_span()),
                crate::ErrorKind::ExpectedIdentifier,
            )))
        }
    }

    pub fn parse_path(&mut self, first_segment: Option<(crate::Span, PathSegment)>, allow_glob: bool) -> crate::Result<(crate::Span, Box<[PathSegment]>, bool)> {
        let (start_span, mut segments) = match first_segment {
            Some((start_span, first_segment)) => (start_span, vec![first_segment]),
            None => (self.current_span(), Vec::new()),
        };

        loop {
            let mut is_glob = false;

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
                    segments.push(PathSegment::Type(type_node));
                }
                Token::Star if allow_glob => {
                    is_glob = true;
                }
                got_token => return Err(Box::new(crate::Error::new(
                    Some(self.current_span()),
                    crate::ErrorKind::ExpectedType {
                        got_token: got_token.clone(),
                    },
                )))
            }
            let end_span = self.current_span();

            self.scan_token()?;
            let Some(Token::Colon2) = self.current_token() else {
                break Ok((
                    start_span.expand_to(end_span),
                    segments.into_boxed_slice(),
                    is_glob,
                ));
            };
            if is_glob {
                // The "*" for a glob path must be the last segment
                break Err(Box::new(crate::Error::new(
                    Some(start_span.expand_to(self.current_span())),
                    crate::ErrorKind::InvalidGlobPath,
                )));
            }
            self.scan_token()?;
        }
    }

    pub fn parse_operand(&mut self, allowed_ends: &[Token], strict_ends: bool) -> crate::Result<Box<Node>> {
        let start_span = self.current_span();
        let token = self.get_token()?;

        if let Some(operation) = UnaryOperation::from_prefix_token(token) {
            self.scan_token()?;
            let operand;
            match operation {
                UnaryOperation::GetSize | UnaryOperation::GetAlign => {
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    let type_node = self.parse_type(Some(&[Token::ParenRight]))?;
                    self.scan_token()?;
                    operand = Box::new(Node::new(type_node.span(), NodeKind::Type(type_node)));
                }
                _ => {
                    operand = self.parse_expression(Some(Precedence::Prefix), allowed_ends, strict_ends)?;
                }
            };

            Ok(Box::new(Node::new(
                start_span.expand_to(operand.span()),
                NodeKind::Unary {
                    operation,
                    operand,
                },
            )))
        }
        else {
            let mut operand = match token {
                Token::Literal(literal) => {
                    // Token literal
                    let literal = literal.clone();
                    self.scan_token()?;

                    if let Some(Token::Colon2) = self.current_token() {
                        let first_segment = match literal {
                            Literal::Name(name) => PathSegment::Name(name),
                            Literal::PrimitiveType(primitive_type) => PathSegment::PrimitiveType(primitive_type),
                            _ => return Err(Box::new(crate::Error::new(
                                Some(start_span),
                                crate::ErrorKind::ExpectedIdentifier,
                            )))
                        };
                        self.scan_token()?;

                        let (span, segments, _) = self.parse_path(Some((start_span, first_segment)), false)?;

                        Box::new(Node::new(
                            span,
                            NodeKind::Path {
                                segments,
                            },
                        ))
                    }
                    else {
                        Box::new(Node::new(
                            start_span,
                            NodeKind::Literal(literal),
                        ))
                    }
                }
                Token::SquareLeft => {
                    // Array literal
                    self.scan_token()?;
                    let mut items = Vec::new();
                    while !matches!(self.current_token(), Some(Token::SquareRight)) {
                        let item = self.parse_expression(None, &[Token::Comma, Token::SquareRight], true)?;
                        items.push(*item);

                        if let Some(Token::Comma) = self.current_token() {
                            self.scan_token()?;
                        }
                    }
                    let span = start_span.expand_to(self.current_span());
                    self.scan_token()?;

                    Box::new(Node::new(
                        span,
                        NodeKind::ArrayLiteral {
                            items: items.into_boxed_slice(),
                        },
                    ))
                }
                Token::ParenLeft => {
                    self.scan_token()?;
                    if let Some(Token::ParenRight) = self.current_token() {
                        // Unit type
                        let span = start_span.expand_to(self.current_span());
                        self.scan_token()?;

                        Box::new(Node::new(
                            span,
                            NodeKind::TupleLiteral {
                                items: Box::new([]),
                            },
                        ))
                    }
                    else {
                        let first_item = self.parse_expression(None, &[Token::Comma, Token::ParenRight], true)?;
                        if let Some(Token::Comma) = self.current_token() {
                            // Tuple literal
                            self.scan_token()?;
                            let mut items = vec![*first_item];
                            while !matches!(self.current_token(), Some(Token::ParenRight)) {
                                let item = self.parse_expression(None, &[Token::Comma, Token::ParenRight], true)?;
                                items.push(*item);

                                if let Some(Token::Comma) = self.current_token() {
                                    self.scan_token()?;
                                }
                            }
                            let span = start_span.expand_to(self.current_span());
                            self.scan_token()?;

                            Box::new(Node::new(
                                span,
                                NodeKind::TupleLiteral {
                                    items: items.into_boxed_slice(),
                                },
                            ))
                        }
                        else {
                            // Grouping parentheses
                            let span = start_span.expand_to(self.current_span());
                            self.scan_token()?;

                            Box::new(Node::new(
                                span,
                                NodeKind::Grouping {
                                    content: first_item,
                                },
                            ))
                        }
                    }
                }
                Token::Colon2 | Token::Super | Token::Module | Token::SelfType | Token::AngleLeft => {
                    // Path literal
                    let (span, segments, _) = self.parse_path(None, false)?;

                    Box::new(Node::new(
                        span,
                        NodeKind::Path {
                            segments,
                        },
                    ))
                }
                Token::CurlyLeft => {
                    // Scope expression
                    self.scan_token()?;
                    self.parse_scope(start_span)?
                }
                Token::If => {
                    // Conditional expression
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

                    let end_span;
                    let alternative;
                    if let Some(Token::Else) = self.current_token() {
                        self.scan_token()?;
                        let expression = self.parse_expression(None, allowed_ends, strict_ends)?;
                        end_span = expression.span();
                        alternative = Some(expression);
                    }
                    else {
                        end_span = consequent.span();
                        alternative = None;
                    }

                    Box::new(Node::new(
                        start_span.expand_to(end_span),
                        NodeKind::Conditional {
                            condition,
                            consequent,
                            alternative,
                        },
                    ))
                }
                Token::Else => {
                    return Err(Box::new(crate::Error::new(
                        Some(start_span),
                        crate::ErrorKind::UnexpectedElse,
                    )));
                }
                Token::While => {
                    // While loop expression
                    self.scan_token()?;
                    self.expect_token(&[Token::ParenLeft])?;
                    self.scan_token()?;
                    let condition = self.parse_expression(None, &[Token::ParenRight], true)?;
                    self.scan_token()?;
                    let body = self.parse_expression(None, allowed_ends, strict_ends)?;

                    Box::new(Node::new(
                        start_span.expand_to(body.span()),
                        NodeKind::While {
                            condition,
                            body,
                        },
                    ))
                }
                Token::Break => {
                    // Break expression
                    self.scan_token()?;
                    Box::new(Node::new(
                        start_span,
                        NodeKind::Break,
                    ))
                }
                Token::Continue => {
                    // Continue expression
                    self.scan_token()?;
                    Box::new(Node::new(
                        start_span,
                        NodeKind::Continue,
                    ))
                }
                Token::Return => {
                    // Return expression
                    self.scan_token()?;
                    let span;
                    let value;
                    if allowed_ends.contains(self.get_token()?) {
                        span = start_span;
                        value = None;
                    }
                    else {
                        let expression = self.parse_expression(None, allowed_ends, strict_ends)?;
                        span = start_span.expand_to(expression.span());
                        value = Some(expression);
                    }

                    Box::new(Node::new(
                        span,
                        NodeKind::Return {
                            value,
                        },
                    ))
                }
                _ => {
                    return Err(Box::new(crate::Error::new(
                        Some(self.current_span()),
                        crate::ErrorKind::ExpectedOperand {
                            got_token: token.clone(),
                        },
                    )));
                }
            };

            // Greedily parse postfix operators, if any
            while let Some(operation) = UnaryOperation::from_postfix_token(self.get_token()?) {
                operand = Box::new(Node::new(
                    start_span.expand_to(self.current_span()),
                    NodeKind::Unary {
                        operation,
                        operand,
                    },
                ));
                self.scan_token()?;
            }

            Ok(operand)
        }
    }

    fn parse_scope(&mut self, start_span: crate::Span) -> crate::Result<Box<Node>> {
        let mut statements = Vec::new();
        let (span, tail) = loop {
            while let Some(Token::Semicolon) = self.current_token() {
                self.scan_token()?;
            }

            match self.current_token() {
                Some(Token::CurlyRight) => {
                    let span = start_span.expand_to(self.current_span());
                    self.scan_token()?;
                    break (span, None);
                }
                Some(Token::Let) => {
                    let let_start_span = self.current_span();
                    self.scan_token()?;
                    statements.push(*self.parse_let_statement(let_start_span)?);
                }
                Some(..) => {
                    let statement = self.parse_expression(None, &[Token::Semicolon, Token::CurlyRight], false)?;
                    if let Some(Token::CurlyRight) = self.current_token() {
                        let span = start_span.expand_to(self.current_span());
                        self.scan_token()?;
                        break (span, Some(statement));
                    }
                    else {
                        statements.push(*statement);
                    }
                }
                None => {
                    return Err(Box::new(crate::Error::new(
                        Some(self.current_span()),
                        crate::ErrorKind::ExpectedClosingBracket {
                            bracket: Token::CurlyRight,
                        },
                    )));
                }
            }
        };

        Ok(Box::new(Node::new(
            span,
            NodeKind::Scope {
                statements: statements.into_boxed_slice(),
                tail,
            },
        )))
    }

    pub fn parse_expression(&mut self, parent_precedence: Option<Precedence>, allowed_ends: &[Token], strict_ends: bool) -> crate::Result<Box<Node>> {
        let start_span = self.current_span();
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
                let (span, rhs) = match operation {
                    BinaryOperation::Convert => {
                        let type_node = self.parse_type(None)?;
                        let rhs = Box::new(Node::new(
                            start_span.expand_to(type_node.span()),
                            NodeKind::Type(type_node),
                        ));
                        (start_span.expand_to(rhs.span()), rhs)
                    }
                    BinaryOperation::Subscript => {
                        let expression = self.parse_expression(None, &[Token::SquareRight], true)?;
                        let span = start_span.expand_to(self.current_span());
                        self.scan_token()?;
                        (span, expression)
                    }
                    _ => {
                        let rhs = self.parse_expression(Some(precedence), allowed_ends, strict_ends)?;
                        (start_span.expand_to(rhs.span()), rhs)
                    }
                };

                lhs = Box::new(Node::new(
                    span,
                    NodeKind::Binary {
                        operation,
                        lhs,
                        rhs,
                    },
                ));
            }
            else if let Token::ParenLeft = token {
                // Left parenthesis indicates a function call
                if let Some(parent_precedence) = parent_precedence {
                    if parent_precedence >= Precedence::Postfix {
                        // Parent operation should be made into a subtree of the call operation
                        // (which has postfix precedence). Usually the parent operation is some
                        // sort of access operation, e.g. `a.b()` or `a::b()`
                        break;
                    }
                }

                self.scan_token()?;
                let mut arguments = Vec::new();
                while !matches!(self.current_token(), Some(Token::ParenRight)) {
                    let argument = self.parse_expression(None, &[Token::Comma, Token::ParenRight], true)?;
                    arguments.push(*argument);

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                let span = start_span.expand_to(self.current_span());
                self.scan_token()?;

                lhs = Box::new(Node::new(
                    span,
                    NodeKind::Call {
                        callee: lhs,
                        arguments: arguments.into_boxed_slice(),
                    },
                ));
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
                    members.push((member_name, *member_value));

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                let span = start_span.expand_to(self.current_span());
                self.scan_token()?;

                lhs = Box::new(Node::new(
                    span,
                    NodeKind::StructureLiteral {
                        structure_type: lhs,
                        members: members.into_boxed_slice(),
                    },
                ));
            }
            else {
                return Err(Box::new(crate::Error::new(
                    Some(self.current_span()),
                    crate::ErrorKind::ExpectedOperation {
                        got_token: token.clone(),
                    },
                )));
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

    pub fn parse_type(&mut self, allowed_ends: Option<&[Token]>) -> crate::Result<Box<TypeNode>> {
        let start_span = self.current_span();

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
                let pointee_type = self.parse_type(allowed_ends)?;

                Ok(Box::new(TypeNode::new(
                    start_span.expand_to(pointee_type.span()),
                    TypeNodeKind::Pointer {
                        pointee_type,
                        semantics,
                    },
                )))
            }
            Token::SquareLeft => {
                self.scan_token()?;
                let item_type = self.parse_type(Some(&[Token::Semicolon, Token::SquareRight]))?;
                let length;
                if let Some(Token::Semicolon) = self.current_token() {
                    self.scan_token()?;
                    length = Some(self.parse_expression(None, &[Token::SquareRight], true)?);
                }
                else {
                    length = None;
                }
                let span = start_span.expand_to(self.current_span());
                self.scan_token()?;

                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }

                Ok(Box::new(TypeNode::new(
                    span,
                    TypeNodeKind::Array {
                        item_type,
                        length,
                    },
                )))
            }
            Token::ParenLeft => {
                self.scan_token()?;
                let result;
                if let Some(Token::ParenRight) = self.current_token() {
                    // Unit type
                    result = Box::new(TypeNode::new(
                        start_span.expand_to(self.current_span()),
                        TypeNodeKind::Tuple {
                            item_types: Box::new([]),
                        },
                    ));
                    self.scan_token()?;
                }
                else {
                    let first_item_type = self.parse_type(Some(&[Token::Comma, Token::ParenRight]))?;
                    if let Some(Token::Comma) = self.current_token() {
                        // Tuple type
                        self.scan_token()?;

                        let mut item_types = vec![*first_item_type];
                        while !matches!(self.current_token(), Some(Token::ParenRight)) {
                            item_types.push(*self.parse_type(Some(&[Token::Comma, Token::ParenRight]))?);

                            if let Some(Token::Comma) = self.current_token() {
                                self.scan_token()?;
                            }
                        }

                        result = Box::new(TypeNode::new(
                            start_span.expand_to(self.current_span()),
                            TypeNodeKind::Tuple {
                                item_types: item_types.into_boxed_slice(),
                            },
                        ));
                        self.scan_token()?;
                    }
                    else {
                        // Grouping parentheses
                        result = Box::new(TypeNode::new(
                            start_span.expand_to(self.current_span()),
                            TypeNodeKind::Grouping {
                                content: first_item_type,
                            },
                        ));
                        self.scan_token()?;
                    }
                }

                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }

                Ok(result)
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

                    parameter_types.push(*self.parse_type(Some(&[Token::Comma, Token::ParenRight]))?);

                    if let Some(Token::Comma) = self.current_token() {
                        self.scan_token()?;
                    }
                }
                let right_paren_span = self.current_span();

                let return_type;
                self.scan_token()?;
                if let Some(Token::RightArrow) = self.current_token() {
                    self.scan_token()?;
                    return_type = self.parse_type(allowed_ends)?;
                }
                else {
                    return_type = Box::new(TypeNode::new(
                        right_paren_span.tail_point(),
                        TypeNodeKind::Path {
                            segments: Box::new([PathSegment::PrimitiveType(crate::sema::PrimitiveType {
                                name: "void",
                                handle: crate::sema::TypeHandle::VOID,
                            })]),
                        },
                    ));

                    if let Some(allowed_ends) = allowed_ends {
                        self.expect_token(allowed_ends)?;
                    }
                }

                Ok(Box::new(TypeNode::new(
                    start_span.expand_to(return_type.span()),
                    TypeNodeKind::Function {
                        parameter_types: parameter_types.into_boxed_slice(),
                        is_variadic: is_varargs,
                        return_type,
                    },
                )))
            }
            Token::Mut => {
                Err(Box::new(crate::Error::new(
                    Some(self.current_span()),
                    crate::ErrorKind::UnexpectedQualifier {
                        got_token: Token::Mut,
                    },
                )))
            }
            _ => {
                let (span, segments, _) = self.parse_path(None, false)?;

                if let Some(allowed_ends) = allowed_ends {
                    self.expect_token(allowed_ends)?;
                }

                Ok(Box::new(TypeNode::new(
                    span,
                    TypeNodeKind::Path {
                        segments,
                    },
                )))
            }
        }
    }

    fn parse_let_statement(&mut self, start_span: crate::Span) -> crate::Result<Box<Node>> {
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

            Ok(Box::new(Node::new(
                start_span.expand_to(value.span()),
                NodeKind::Constant {
                    name,
                    value_type,
                    value,
                    global_register: None,
                },
            )))
        }
        else {
            let is_mutable = if let Some(Token::Mut) = self.current_token() {
                self.scan_token()?;
                true
            } else {
                false
            };
            let name = self.expect_identifier()?;
            let mut end_span = self.current_span();
            self.scan_token()?;
            self.expect_token(&[Token::Colon, Token::Equal, Token::Semicolon])?;

            let value_type = if let Some(Token::Colon) = self.current_token() {
                self.scan_token()?;
                let type_node = self.parse_type(Some(&[Token::Equal, Token::Semicolon]))?;
                end_span = type_node.span();
                Some(type_node)
            } else {
                None
            };

            let value = if let Some(Token::Equal) = self.current_token() {
                self.scan_token()?;
                let expression = self.parse_expression(None, &[Token::Semicolon], true)?;
                end_span = expression.span();
                Some(expression)
            } else {
                None
            };
            self.scan_token()?;

            Ok(Box::new(Node::new(
                start_span.expand_to(end_span),
                NodeKind::Let {
                    name,
                    value_type,
                    is_mutable,
                    value,
                    global_register: None,
                },
            )))
        }
    }

    fn parse_function_definition(&mut self, start_span: crate::Span, is_foreign: bool) -> crate::Result<Box<Node>> {
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
            let parameter_start_span = self.current_span();

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
                span: parameter_start_span.expand_to(parameter_type.span()),
                name: parameter_name,
                type_node: parameter_type,
                is_mutable,
            });

            if let Some(Token::Comma) = self.current_token() {
                self.scan_token()?;
            }
        }
        let mut end_span = self.current_span();

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
            Box::new(TypeNode::new(
                end_span.tail_point(),
                TypeNodeKind::Path {
                    segments: Box::new([PathSegment::PrimitiveType(PrimitiveType {
                        name: "void",
                        handle: TypeHandle::VOID
                    })]),
                },
            ))
        };

        let body = if let Some(Token::CurlyLeft) = self.current_token() {
            let body_start_span = self.current_span();
            self.scan_token()?;
            let body = self.parse_scope(body_start_span)?;
            end_span = body.span();
            Some(body)
        } else {
            self.scan_token()?;
            end_span = return_type.span();
            None
        };

        Ok(Box::new(Node::new(
            start_span.expand_to(end_span),
            NodeKind::Function {
                name,
                is_foreign,
                parameters: parameters.into_boxed_slice(),
                is_variadic,
                return_type,
                body,
                global_register: None,
            },
        )))
    }

    fn parse_structure_definition(&mut self, context: &mut GlobalContext, start_span: crate::Span, is_foreign: bool) -> crate::Result<Box<Node>> {
        let name = self.expect_identifier()?;
        let name_span = self.current_span();

        let self_type = context.outline_structure_type(name.clone())?;
        context.set_self_type(self_type);

        self.scan_token()?;
        if is_foreign {
            self.expect_token(&[Token::CurlyLeft, Token::Semicolon])?;
        }
        else {
            self.expect_token(&[Token::CurlyLeft])?;
        }
        let (end_span, members) = if let Some(Token::CurlyLeft) = self.current_token() {
            self.scan_token()?;

            let mut members = Vec::new();
            while !matches!(self.current_token(), Some(Token::CurlyRight)) {
                let member_name = self.expect_identifier()?;
                let member_start_span = self.current_span();
                self.scan_token()?;
                self.expect_token(&[Token::Colon])?;
                self.scan_token()?;
                let member_type = self.parse_type(Some(&[Token::Comma, Token::CurlyRight]))?;

                members.push(StructureMemberNode {
                    span: member_start_span.expand_to(member_type.span()),
                    name: member_name,
                    type_node: member_type,
                });

                if let Some(Token::Comma) = self.current_token() {
                    self.scan_token()?;
                }
            }
            let end_span = self.current_span();

            self.scan_token()?;
            (end_span, Some(members))
        } else {
            self.scan_token()?;
            (name_span, None)
        };

        context.unset_self_type();

        Ok(Box::new(Node::new(
            start_span.expand_to(end_span),
            NodeKind::Structure {
                name,
                is_foreign,
                members: members.map(Vec::into_boxed_slice),
                self_type,
            },
        )))
    }

    pub fn parse_statement(&mut self, global_context: &mut GlobalContext, is_implementation: bool, allow_empty: bool) -> crate::Result<Option<Box<Node>>> {
        let start_span = self.current_span();

        // Most statement types can be detected simply by the first token
        match self.current_token() {
            Some(Token::Semicolon) if allow_empty => {
                self.scan_token()?;
                // Returning None would imply that the end of the file was reached,
                // so recursively try to parse a statement instead
                self.parse_statement(global_context, is_implementation, allow_empty)
            }
            Some(Token::Let) => {
                self.scan_token()?;
                self.parse_let_statement(start_span).map(Some)
            }
            Some(Token::Function) => {
                self.scan_token()?;
                self.parse_function_definition(start_span, false).map(Some)
            }
            Some(Token::Struct) if !is_implementation => {
                self.scan_token()?;
                self.parse_structure_definition(global_context, start_span, false).map(Some)
            }
            Some(Token::Implement) if !is_implementation => {
                self.scan_token()?;
                let self_type = self.parse_type(Some(&[Token::CurlyLeft]))?;
                self.scan_token()?;

                let mut statements = Vec::new();
                let span = loop {
                    while let Some(Token::Semicolon) = self.current_token() {
                        self.scan_token()?;
                    }

                    match self.current_token() {
                        Some(Token::CurlyRight) => {
                            let span = start_span.expand_to(self.current_span());
                            self.scan_token()?;
                            break span;
                        }
                        None => {
                            return Err(Box::new(crate::Error::new(
                                Some(self.current_span()),
                                crate::ErrorKind::ExpectedClosingBracket {
                                    bracket: Token::CurlyRight,
                                },
                            )));
                        }
                        _ => {
                            let statement = self.parse_statement(global_context, true, true)?
                                .ok_or_else(|| Box::new(crate::Error::new(
                                    Some(self.current_span()),
                                    crate::ErrorKind::ExpectedClosingBracket {
                                        bracket: Token::CurlyRight,
                                    },
                                )))?;
                            statements.push(*statement);
                        }
                    }
                };

                Ok(Some(Box::new(Node::new(
                    span,
                    NodeKind::Implement {
                        self_type,
                        statements: statements.into_boxed_slice(),
                    },
                ))))
            }
            Some(Token::Module) if !is_implementation => {
                self.scan_token()?;
                let name = self.expect_identifier()?;
                let name_span = self.current_span();
                self.scan_token()?;

                if let Token::Semicolon = self.expect_token(&[Token::Semicolon, Token::CurlyLeft])? {
                    self.scan_token()?;

                    global_context.queue_module_file(name.clone());

                    return Ok(Some(Box::new(Node::new(
                        start_span.expand_to(name_span),
                        NodeKind::ModuleFile {
                            name,
                        },
                    ))));
                }
                self.scan_token()?;

                let namespace = global_context.get_or_create_module(
                    global_context.current_module(),
                    &name,
                )?;
                let parent_module = global_context.replace_current_module(namespace);

                let mut statements = Vec::new();
                let span = loop {
                    while let Some(Token::Semicolon) = self.current_token() {
                        self.scan_token()?;
                    }

                    match self.current_token() {
                        Some(Token::CurlyRight) => {
                            let span = start_span.expand_to(self.current_span());
                            self.scan_token()?;
                            break span;
                        }
                        None => {
                            return Err(Box::new(crate::Error::new(
                                Some(self.current_span()),
                                crate::ErrorKind::ExpectedClosingBracket {
                                    bracket: Token::CurlyRight,
                                },
                            )));
                        }
                        _ => {
                            let statement = self.parse_statement(global_context, false, true)?
                                .ok_or_else(|| Box::new(crate::Error::new(
                                    Some(self.current_span()),
                                    crate::ErrorKind::ExpectedClosingBracket {
                                        bracket: Token::CurlyRight,
                                    },
                                )))?;
                            statements.push(*statement);
                        }
                    }
                };

                global_context.replace_current_module(parent_module);

                Ok(Some(Box::new(Node::new(
                    span,
                    NodeKind::Module {
                        name,
                        statements: statements.into_boxed_slice(),
                        namespace,
                    },
                ))))
            }
            Some(Token::Import) if !is_implementation => {
                self.scan_token()?;
                let (path_span, segments, is_glob_path) = self.parse_path(None, true)?;

                if is_glob_path {
                    self.expect_token(&[Token::Semicolon])?;
                    self.scan_token()?;

                    let path = global_context.get_absolute_path(path_span, &segments)?;
                    global_context.current_module_info_mut().add_glob_import(path);

                    Ok(Some(Box::new(Node::new(
                        start_span.expand_to(path_span),
                        NodeKind::GlobImport {
                            segments,
                        },
                    ))))
                }
                else {
                    self.expect_token(&[Token::As, Token::Semicolon])?;

                    let alias;
                    let end_span;
                    if let Some(Token::As) = self.current_token() {
                        self.scan_token()?;
                        alias = Some(self.expect_identifier()?);
                        end_span = self.current_span();
                        self.scan_token()?;
                        self.expect_token(&[Token::Semicolon])?;
                    }
                    else {
                        alias = None;
                        end_span = path_span;
                    }
                    self.scan_token()?;

                    let path = global_context.get_absolute_path(path_span, &segments)?;
                    let import_name = match alias.as_ref() {
                        Some(alias) => alias.as_ref(),
                        None => match path.tail_name() {
                            Some(name) => name,
                            None => {
                                return Err(Box::new(crate::Error::new(
                                    Some(end_span),
                                    crate::ErrorKind::ImportAliasRequired {
                                        path: path.to_string(),
                                    },
                                )));
                            }
                        }
                    };
                    // Establish an alias symbol in the current module corresponding to this import
                    global_context.current_module_info_mut().define(
                        import_name,
                        Symbol::Alias(path.clone()),
                    )?;

                    Ok(Some(Box::new(Node::new(
                        start_span.expand_to(end_span),
                        NodeKind::Import {
                            segments,
                            alias,
                        },
                    ))))
                }
            }
            Some(Token::Foreign) => {
                self.scan_token()?;
                match self.get_token()? {
                    Token::Function => {
                        self.scan_token()?;
                        self.parse_function_definition(start_span, true).map(Some)
                    }
                    Token::Struct if !is_implementation => {
                        self.scan_token()?;
                        self.parse_structure_definition(global_context, start_span, true).map(Some)
                    }
                    got_token => {
                        let mut allowed_tokens = vec![Token::Function];
                        if !is_implementation {
                            allowed_tokens.push(Token::Struct);
                        }
                        Err(Box::new(crate::Error::new(
                            Some(self.current_span()),
                            crate::ErrorKind::ExpectedTokenFromList {
                                got_token: got_token.clone(),
                                allowed_tokens,
                            },
                        )))
                    }
                }
            }
            Some(got_token) => {
                Err(Box::new(crate::Error::new(
                    Some(self.current_span()),
                    crate::ErrorKind::ExpectedTokenFromList {
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
                    },
                )))
            }
            None => Ok(None),
        }
    }

    pub fn parse_top_level_statement(&mut self, context: &mut GlobalContext) -> crate::Result<Option<Box<Node>>> {
        self.parse_statement(context, false, true)
    }
}
