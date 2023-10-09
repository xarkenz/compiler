use super::*;

use utf8_chars::BufReadCharsExt;

#[derive(Debug)]
pub struct Parser<'a, T: BufReadCharsExt> {
    scanner: &'a mut scan::Scanner<'a, T>,
    current_token: Option<Token>,
}

impl<'a, T: BufReadCharsExt> Parser<'a, T> {
    pub fn new(scanner: &'a mut scan::Scanner<'a, T>) -> Self {
        Self {
            scanner,
            current_token: None,
        }
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

    pub fn parse_binary_expression(&mut self, parent_operation: Option<BinaryOperation>) -> crate::Result<Box<Node>> {
        let parent_precedence = parent_operation.map(|operation| operation.precedence());

        let mut lhs = self.parse_terminal_node()?;

        while let Some(token) = self.current_token() {
            let operation = match token {
                Token::Plus => BinaryOperation::Add,
                Token::Minus => BinaryOperation::Subtract,
                Token::Star => BinaryOperation::Multiply,
                Token::Slash => BinaryOperation::Divide,
                _ => return Err(self.scanner.syntax_error(format!("expected a binary operator, got {token:?}")))
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
            let rhs = self.parse_binary_expression(Some(operation))?;

            lhs = Box::new(Node::Binary {
                operation,
                lhs,
                rhs,
            });
        }

        Ok(lhs)
    }
}