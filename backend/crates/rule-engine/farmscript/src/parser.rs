//! Parser for FarmScript
//!
//! Uses Pratt parsing for operator precedence handling.

use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind, Span, TemplatePart};
use thiserror::Error;

/// Parse error
#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at position {position}")]
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    #[error("Lexer error: {0}")]
    LexerError(String),
}

/// The FarmScript parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current = lexer.next_token();
        Self {
            lexer,
            current: current.clone(),
            previous: current,
        }
    }

    /// Parse the entire expression
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expression()?;

        if !self.is_at_end() {
            return Err(ParseError::UnexpectedToken {
                expected: "end of expression".into(),
                found: format!("{}", self.current.kind),
                position: self.current.span.start,
            });
        }

        Ok(expr)
    }

    /// Parse an expression with minimum precedence
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_null_coalesce()
    }

    /// Parse null coalescing: a ?? b
    fn parse_null_coalesce(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_or()?;

        while self.match_token(TokenKind::QuestionQuestion) {
            let right = self.parse_or()?;
            expr = Expr::NullCoalesce(Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    /// Parse OR: a or b
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and()?;

        while self.match_token(TokenKind::Or) {
            let right = self.parse_and()?;
            expr = Expr::binary(expr, BinaryOp::Or, right);
        }

        Ok(expr)
    }

    /// Parse AND: a and b
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenKind::And) {
            let right = self.parse_equality()?;
            expr = Expr::binary(expr, BinaryOp::And, right);
        }

        Ok(expr)
    }

    /// Parse equality: ==, ===, !=, !==, is, isnt, etc.
    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = if self.match_token(TokenKind::EqEq) {
                BinaryOp::Eq
            } else if self.match_token(TokenKind::EqEqEq)
                || self.match_token(TokenKind::Is)
                || self.match_token(TokenKind::Eq)
                || self.match_token(TokenKind::Equals)
                || self.match_token(TokenKind::SameAs)
            {
                BinaryOp::StrictEq
            } else if self.match_token(TokenKind::NotEq) || self.match_token(TokenKind::LtGt) {
                BinaryOp::NotEq
            } else if self.match_token(TokenKind::NotEqEq)
                || self.match_token(TokenKind::Isnt)
                || self.match_token(TokenKind::IsNot)
                || self.match_token(TokenKind::NotEqKw)
            {
                BinaryOp::StrictNotEq
            } else {
                break;
            };

            let right = self.parse_comparison()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    /// Parse comparison: <, >, <=, >=, in
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = if self.match_token(TokenKind::Lt) {
                BinaryOp::Lt
            } else if self.match_token(TokenKind::Gt) {
                BinaryOp::Gt
            } else if self.match_token(TokenKind::LtEq) {
                BinaryOp::LtEq
            } else if self.match_token(TokenKind::GtEq) {
                BinaryOp::GtEq
            } else if self.check(TokenKind::In) {
                // Check for 'in' keyword used as operator
                self.advance();
                BinaryOp::In
            } else {
                break;
            };

            let right = self.parse_additive()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    /// Parse additive: +, -
    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = if self.match_token(TokenKind::Plus) {
                BinaryOp::Add
            } else if self.match_token(TokenKind::Minus) {
                BinaryOp::Sub
            } else {
                break;
            };

            let right = self.parse_multiplicative()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    /// Parse multiplicative: *, /, %, /?, /!
    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_power()?;

        loop {
            let op = if self.match_token(TokenKind::Star) {
                BinaryOp::Mul
            } else if self.match_token(TokenKind::Slash) {
                BinaryOp::Div
            } else if self.match_token(TokenKind::Percent) {
                BinaryOp::Mod
            } else if self.match_token(TokenKind::SlashQuestion) {
                BinaryOp::SafeDivZero
            } else if self.match_token(TokenKind::SlashBang) {
                BinaryOp::SafeDivNull
            } else {
                break;
            };

            let right = self.parse_power()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    /// Parse power: ^ (right-associative)
    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_unary()?;

        if self.match_token(TokenKind::Caret) {
            let right = self.parse_power()?; // Right-associative
            return Ok(Expr::binary(expr, BinaryOp::Pow, right));
        }

        Ok(expr)
    }

    /// Parse unary: not, !, -, +
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenKind::Not) {
            let expr = self.parse_unary()?;
            return Ok(Expr::unary(UnaryOp::Not, expr));
        }

        if self.match_token(TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::unary(UnaryOp::Neg, expr));
        }

        if self.match_token(TokenKind::Plus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::unary(UnaryOp::Plus, expr));
        }

        self.parse_postfix()
    }

    /// Parse postfix: x?, method calls, property access, indexing
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenKind::Question) {
                // Truthy postfix: x?
                expr = Expr::Postfix(Box::new(PostfixExpr {
                    expr,
                    op: PostfixOp::Truthy,
                    span: self.previous.span,
                }));
            } else if self.match_token(TokenKind::Dot) {
                // Property access or method call
                let name = self.expect_identifier()?;

                if self.match_token(TokenKind::LParen) {
                    // Method call: obj.method(args...)
                    let args = self.parse_args()?;
                    expr = Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: name,
                        args,
                        span: self.previous.span,
                    }));
                } else {
                    // Property access: obj.prop
                    expr = Expr::Property(Box::new(PropertyExpr {
                        object: expr,
                        property: name,
                        span: self.previous.span,
                    }));
                }
            } else if self.match_token(TokenKind::LBracket) {
                // Index access: arr[idx]
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expr::Index(Box::new(IndexExpr {
                    object: expr,
                    index,
                    span: self.previous.span,
                }));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse primary expressions
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        // Literals
        if let TokenKind::Integer(n) = self.current.kind {
            self.advance();
            return Ok(Expr::int(n));
        }

        if let TokenKind::Float(n) = self.current.kind {
            self.advance();
            return Ok(Expr::float(n));
        }

        if let TokenKind::String(ref s) = self.current.kind {
            let s = s.clone();
            self.advance();
            return Ok(Expr::string(s));
        }

        if let TokenKind::TemplateString(ref parts) = self.current.kind {
            let parts = parts.clone();
            self.advance();
            return self.parse_template_parts(parts);
        }

        if self.match_token(TokenKind::True) {
            return Ok(Expr::bool(true));
        }

        if self.match_token(TokenKind::False) {
            return Ok(Expr::bool(false));
        }

        if self.match_token(TokenKind::Null) {
            return Ok(Expr::null());
        }

        // If expression
        if self.match_token(TokenKind::If) {
            return self.parse_if_expression();
        }

        // Let expression
        if self.match_token(TokenKind::Let) {
            return self.parse_let_expression();
        }

        // SQL-like query
        if self.match_token(TokenKind::From) {
            return self.parse_query_expression();
        }

        // Array literal
        if self.match_token(TokenKind::LBracket) {
            return self.parse_array_literal();
        }

        // Grouped expression or lambda
        if self.match_token(TokenKind::LParen) {
            return self.parse_grouped_or_lambda();
        }

        // Identifier or function call
        if let TokenKind::Ident(ref name) = self.current.kind {
            let name = name.clone();
            let span = self.current.span;
            self.advance();

            // Check for lambda: x => ...
            if self.match_token(TokenKind::Arrow) {
                let body = self.parse_expression()?;
                return Ok(Expr::Lambda(Box::new(LambdaExpr {
                    params: vec![name],
                    body,
                    span,
                })));
            }

            // Check for function call
            if self.match_token(TokenKind::LParen) {
                let args = self.parse_args()?;
                return Ok(Expr::Call(Box::new(CallExpr {
                    function: name,
                    args,
                    span,
                })));
            }

            // Simple variable reference
            return Ok(Expr::Var(VarExpr { name, span }));
        }

        // Handle keyword-based function names (map, filter, etc.)
        if self.is_array_function_keyword() {
            let name = format!("{}", self.current.kind);
            let span = self.current.span;
            self.advance();

            if self.match_token(TokenKind::LParen) {
                let args = self.parse_args()?;
                return Ok(Expr::Call(Box::new(CallExpr {
                    function: name,
                    args,
                    span,
                })));
            }

            return Err(ParseError::InvalidExpression(
                format!("'{}' requires arguments", name),
            ));
        }

        Err(ParseError::UnexpectedToken {
            expected: "expression".into(),
            found: format!("{}", self.current.kind),
            position: self.current.span.start,
        })
    }

    /// Check if current token is an array function keyword
    fn is_array_function_keyword(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::Map
                | TokenKind::Filter
                | TokenKind::Reduce
                | TokenKind::All
                | TokenKind::Some
                | TokenKind::Merge
                | TokenKind::Missing
                | TokenKind::MissingSome
                | TokenKind::Log
                | TokenKind::Contains
        )
    }

    /// Parse template string parts
    fn parse_template_parts(&mut self, parts: Vec<TemplatePart>) -> Result<Expr, ParseError> {
        let mut exprs = Vec::new();

        for part in parts {
            match part {
                TemplatePart::Literal(s) => {
                    exprs.push(TemplateExpr::Literal(s));
                }
                TemplatePart::Expr(expr_str) => {
                    // Parse the expression string
                    let mut inner_parser = Parser::new(Lexer::new(&expr_str));
                    let expr = inner_parser.parse()?;
                    exprs.push(TemplateExpr::Expr(expr));
                }
            }
        }

        Ok(Expr::Template(exprs))
    }

    /// Parse if expression: if cond then a else b
    fn parse_if_expression(&mut self) -> Result<Expr, ParseError> {
        let span = self.previous.span;
        let condition = self.parse_expression()?;

        self.expect(TokenKind::Then)?;
        let then_branch = self.parse_expression()?;

        let mut else_ifs = Vec::new();
        let mut else_branch = None;

        // Parse else if / else chains
        while self.match_token(TokenKind::Else) {
            if self.match_token(TokenKind::If) {
                // else if
                let cond = self.parse_expression()?;
                self.expect(TokenKind::Then)?;
                let then = self.parse_expression()?;
                else_ifs.push((cond, then));
            } else {
                // final else
                else_branch = Some(self.parse_expression()?);
                break;
            }
        }

        Ok(Expr::If(Box::new(IfExpr {
            condition,
            then_branch,
            else_branch,
            else_ifs,
            span,
        })))
    }

    /// Parse let expression: let x = expr in body
    /// Note: For complex expressions with comparison/in operators, use parentheses
    fn parse_let_expression(&mut self) -> Result<Expr, ParseError> {
        let span = self.previous.span;
        let name = self.expect_identifier()?;

        self.expect_kind(TokenKind::Assign)?; // Use = for assignment

        // Parse value - use parse_additive to stop before 'in' keyword
        // For comparisons in value, use parentheses: let x = (a > b) in ...
        let value = self.parse_additive()?;

        // 'in' is required to separate value from body
        self.expect_kind(TokenKind::In)?;

        let body = self.parse_expression()?;

        Ok(Expr::Let(Box::new(LetExpr {
            name,
            value,
            body,
            span,
        })))
    }

    /// Parse SQL-like query: from items where cond select expr
    fn parse_query_expression(&mut self) -> Result<Expr, ParseError> {
        let span = self.previous.span;
        let source = self.parse_primary()?; // Source is a simple expression

        let filter = if self.match_token(TokenKind::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(TokenKind::Select)?;
        let projection = self.parse_expression()?;

        Ok(Expr::Query(Box::new(QueryExpr {
            source,
            filter,
            projection,
            span,
        })))
    }

    /// Parse array literal: [a, b, c]
    fn parse_array_literal(&mut self) -> Result<Expr, ParseError> {
        let mut items = Vec::new();

        if !self.check(TokenKind::RBracket) {
            loop {
                items.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(TokenKind::RBracket) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RBracket)?;
        Ok(Expr::Array(items))
    }

    /// Parse grouped expression or lambda
    fn parse_grouped_or_lambda(&mut self) -> Result<Expr, ParseError> {
        // Check if this is a lambda with multiple params: (x, y) => ...
        if self.check_ident() {
            let first_name = self.expect_identifier()?;

            if self.match_token(TokenKind::Comma) {
                // Multiple params lambda
                let mut params = vec![first_name];

                loop {
                    params.push(self.expect_identifier()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }

                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_expression()?;

                return Ok(Expr::Lambda(Box::new(LambdaExpr {
                    params,
                    body,
                    span: Span::new(0, 0),
                })));
            }

            // Check for single-param lambda with parens: (x) => ...
            if self.match_token(TokenKind::RParen) {
                if self.match_token(TokenKind::Arrow) {
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda(Box::new(LambdaExpr {
                        params: vec![first_name],
                        body,
                        span: Span::new(0, 0),
                    })));
                }

                // Just a grouped identifier
                return Ok(Expr::Var(VarExpr {
                    name: first_name,
                    span: Span::new(0, 0),
                }));
            }

            // It's a grouped expression starting with an identifier
            // We need to "unread" the identifier...
            // This is tricky. Let's just parse a fresh expression.
            let var = Expr::Var(VarExpr {
                name: first_name,
                span: Span::new(0, 0),
            });

            // Continue parsing the rest of the expression
            let expr = self.continue_expression(var)?;
            self.expect(TokenKind::RParen)?;
            return Ok(expr);
        }

        // Regular grouped expression
        let expr = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;
        Ok(expr)
    }

    /// Continue parsing an expression after we've already parsed part of it
    fn continue_expression(&mut self, left: Expr) -> Result<Expr, ParseError> {
        // This is called when we've consumed an identifier inside parens
        // and need to continue parsing as if it was a normal expression
        let mut expr = left;

        // Handle postfix operators on the initial expression
        loop {
            if self.match_token(TokenKind::Question) {
                expr = Expr::Postfix(Box::new(PostfixExpr {
                    expr,
                    op: PostfixOp::Truthy,
                    span: self.previous.span,
                }));
            } else if self.match_token(TokenKind::Dot) {
                let name = self.expect_identifier()?;
                if self.match_token(TokenKind::LParen) {
                    let args = self.parse_args()?;
                    expr = Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: name,
                        args,
                        span: self.previous.span,
                    }));
                } else {
                    expr = Expr::Property(Box::new(PropertyExpr {
                        object: expr,
                        property: name,
                        span: self.previous.span,
                    }));
                }
            } else if self.match_token(TokenKind::LBracket) {
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expr::Index(Box::new(IndexExpr {
                    object: expr,
                    index,
                    span: self.previous.span,
                }));
            } else {
                break;
            }
        }

        // Now handle binary operators
        // We need to handle all precedence levels
        expr = self.parse_binary_rest(expr, 0)?;

        Ok(expr)
    }

    /// Parse the rest of a binary expression given a left operand
    fn parse_binary_rest(&mut self, mut left: Expr, min_prec: u8) -> Result<Expr, ParseError> {
        loop {
            let (op, prec) = match &self.current.kind {
                TokenKind::Or => (BinaryOp::Or, 1),
                TokenKind::And => (BinaryOp::And, 2),
                TokenKind::EqEq => (BinaryOp::Eq, 3),
                TokenKind::EqEqEq | TokenKind::Is | TokenKind::Eq
                | TokenKind::Equals | TokenKind::SameAs => (BinaryOp::StrictEq, 3),
                TokenKind::NotEq | TokenKind::LtGt => (BinaryOp::NotEq, 3),
                TokenKind::NotEqEq | TokenKind::Isnt
                | TokenKind::IsNot | TokenKind::NotEqKw => (BinaryOp::StrictNotEq, 3),
                TokenKind::Lt => (BinaryOp::Lt, 4),
                TokenKind::Gt => (BinaryOp::Gt, 4),
                TokenKind::LtEq => (BinaryOp::LtEq, 4),
                TokenKind::GtEq => (BinaryOp::GtEq, 4),
                TokenKind::Plus => (BinaryOp::Add, 5),
                TokenKind::Minus => (BinaryOp::Sub, 5),
                TokenKind::Star => (BinaryOp::Mul, 6),
                TokenKind::Slash => (BinaryOp::Div, 6),
                TokenKind::Percent => (BinaryOp::Mod, 6),
                TokenKind::SlashQuestion => (BinaryOp::SafeDivZero, 6),
                TokenKind::SlashBang => (BinaryOp::SafeDivNull, 6),
                TokenKind::Caret => (BinaryOp::Pow, 7),
                _ => break,
            };

            if prec < min_prec {
                break;
            }

            self.advance();
            let right = self.parse_unary()?;
            let next_min = if op.is_right_assoc() { prec } else { prec + 1 };
            let right = self.parse_binary_rest(right, next_min)?;
            left = Expr::binary(left, op, right);
        }

        Ok(left)
    }

    /// Parse function arguments
    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    /// Expect a specific token kind
    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", kind),
                found: format!("{}", self.current.kind),
                position: self.current.span.start,
            })
        }
    }

    /// Expect a specific token kind (with owned kind)
    fn expect_kind(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        self.expect(kind)
    }

    /// Expect an identifier and return its name
    /// Also accepts keywords that can be used as method names (filter, map, etc.)
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        // First check for actual identifier
        if let TokenKind::Ident(name) = &self.current.kind {
            let name = name.clone();
            self.advance();
            return Ok(name);
        }

        // Allow keywords as method/property names
        let name = match &self.current.kind {
            TokenKind::Map => "map",
            TokenKind::Filter => "filter",
            TokenKind::Reduce => "reduce",
            TokenKind::All => "all",
            TokenKind::Some => "some",
            TokenKind::Merge => "merge",
            TokenKind::Contains => "contains",
            TokenKind::Missing => "missing",
            TokenKind::MissingSome => "missing_some",
            TokenKind::Log => "log",
            TokenKind::In => "in",
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".into(),
                    found: format!("{}", self.current.kind),
                    position: self.current.span.start,
                });
            }
        };

        self.advance();
        Ok(name.to_string())
    }

    /// Check if current token is an identifier
    fn check_ident(&self) -> bool {
        matches!(self.current.kind, TokenKind::Ident(_))
    }

    /// Check if current token matches the expected kind
    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(&kind)
    }

    /// Match and consume if current token matches
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }

    /// Check if at end of input
    fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Expr, ParseError> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        parser.parse()
    }

    #[test]
    fn test_simple_literal() {
        assert_eq!(parse("42").unwrap(), Expr::int(42));
        assert_eq!(parse("3.14").unwrap(), Expr::float(3.14));
        assert_eq!(parse("true").unwrap(), Expr::bool(true));
        assert_eq!(parse("null").unwrap(), Expr::null());
    }

    #[test]
    fn test_variable() {
        let expr = parse("foo").unwrap();
        match expr {
            Expr::Var(v) => assert_eq!(v.name, "foo"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_path_variable() {
        let expr = parse("/users/count").unwrap();
        match expr {
            Expr::Var(v) => assert_eq!(v.name, "/users/count"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_binary_operators() {
        let expr = parse("a + b").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        // a + b * c should parse as a + (b * c)
        let expr = parse("a + b * c").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
                match &b.right {
                    Expr::Binary(inner) => {
                        assert_eq!(inner.op, BinaryOp::Mul);
                    }
                    _ => panic!("Expected nested binary"),
                }
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_comparison() {
        let expr = parse("x < 10").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Lt);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_logical_and() {
        let expr = parse("a and b").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::And);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_equality_synonyms() {
        // All should parse to StrictEq
        for source in &["a === b", "a is b", "a eq b", "a equals b", "a same_as b"] {
            let expr = parse(source).unwrap();
            match expr {
                Expr::Binary(b) => {
                    assert_eq!(b.op, BinaryOp::StrictEq, "Failed for: {}", source);
                }
                _ => panic!("Expected binary for: {}", source),
            }
        }
    }

    #[test]
    fn test_if_expression() {
        let expr = parse("if x > 0 then 1 else 0").unwrap();
        match expr {
            Expr::If(i) => {
                assert!(i.else_branch.is_some());
            }
            _ => panic!("Expected if"),
        }
    }

    #[test]
    fn test_function_call() {
        let expr = parse("max(a, b, c)").unwrap();
        match expr {
            Expr::Call(c) => {
                assert_eq!(c.function, "max");
                assert_eq!(c.args.len(), 3);
            }
            _ => panic!("Expected call"),
        }
    }

    #[test]
    fn test_method_call() {
        let expr = parse("items.filter(x => x > 0)").unwrap();
        match expr {
            Expr::MethodCall(m) => {
                assert_eq!(m.method, "filter");
            }
            _ => panic!("Expected method call"),
        }
    }

    #[test]
    fn test_lambda() {
        let expr = parse("x => x * 2").unwrap();
        match expr {
            Expr::Lambda(l) => {
                assert_eq!(l.params, vec!["x"]);
            }
            _ => panic!("Expected lambda"),
        }
    }

    #[test]
    fn test_array_literal() {
        let expr = parse("[1, 2, 3]").unwrap();
        match expr {
            Expr::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_in_operator() {
        let expr = parse("x in [1, 2, 3]").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::In);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_safe_division() {
        let expr = parse("a /? b").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::SafeDivZero);
            }
            _ => panic!("Expected binary"),
        }

        let expr = parse("a /! b").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::SafeDivNull);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_truthy_postfix() {
        let expr = parse("x?").unwrap();
        match expr {
            Expr::Postfix(p) => {
                assert_eq!(p.op, PostfixOp::Truthy);
            }
            _ => panic!("Expected postfix"),
        }
    }

    #[test]
    fn test_null_coalesce() {
        let expr = parse("a ?? b").unwrap();
        match expr {
            Expr::NullCoalesce(_, _) => {}
            _ => panic!("Expected null coalesce"),
        }
    }

    #[test]
    fn test_complex_expression() {
        // This is a real expression from the fixtures
        let expr = parse("alert_acknowledged and time_since_alert_secs < 120").unwrap();
        match expr {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::And);
            }
            _ => panic!("Expected binary"),
        }
    }

    #[test]
    fn test_clamp_expression() {
        let expr = parse("clamp(0, 100, raw_score)").unwrap();
        match expr {
            Expr::Call(c) => {
                assert_eq!(c.function, "clamp");
                assert_eq!(c.args.len(), 3);
            }
            _ => panic!("Expected call"),
        }
    }
}
