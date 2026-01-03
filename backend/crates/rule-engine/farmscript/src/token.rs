//! Token types for FarmScript lexer

use std::fmt;

/// Span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Token with kind and position
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            lexeme: lexeme.into(),
        }
    }
}

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    TemplateString(Vec<TemplatePart>), // `Hello {name}!`
    Bool(bool),
    Null,

    // Identifier (includes /path/style variables)
    Ident(String),

    // Arithmetic operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Caret,         // ^

    // Safe division operators
    SlashQuestion, // /? (safe div returns 0)
    SlashBang,     // /! (safe div returns null)

    // Assignment
    Assign,        // = (for let expressions)

    // Comparison operators
    EqEq,          // ==
    EqEqEq,        // ===
    NotEq,         // !=
    NotEqEq,       // !==
    LtGt,          // <>
    Lt,            // <
    Gt,            // >
    LtEq,          // <=
    GtEq,          // >=

    // Logical operators (keywords handled separately)
    And,           // and, &&
    Or,            // or, ||
    Not,           // not, !

    // Postfix operators
    Question,      // ? (truthy check: x?)

    // Null coalescing
    QuestionQuestion, // ??

    // Arrow for lambdas
    Arrow,         // =>

    // Delimiters
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    LBrace,        // {
    RBrace,        // }
    Comma,         // ,
    Dot,           // .
    Colon,         // :
    Semicolon,     // ;

    // Keywords - Control flow
    If,
    Then,
    Else,

    // Keywords - Bindings
    Let,
    In,

    // Keywords - Literals
    True,
    False,

    // Keywords - Equality synonyms
    Is,            // is (alias for ===)
    Isnt,          // isnt (alias for !==)
    IsNot,         // is_not (alias for !==)
    Eq,            // eq (alias for ===)
    Equals,        // equals (alias for ===)
    SameAs,        // same_as (alias for ===)
    NotEqKw,       // not_eq (alias for !==)

    // Keywords - SQL-like
    From,
    Where,
    Select,

    // Keywords - Array operations
    Map,
    Filter,
    Reduce,
    All,
    Some,
    None,
    Merge,

    // Keywords - Data operations
    Missing,
    MissingSome,

    // Keywords - Debug
    Log,

    // Keywords - Functions
    Contains,

    // Special
    Eof,
    Error(String),
}

/// Part of a template string
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Literal(String),
    Expr(String), // The expression inside {}
}

impl TokenKind {
    /// Try to match a keyword from an identifier
    pub fn from_keyword(s: &str) -> Option<TokenKind> {
        match s.to_lowercase().as_str() {
            // Control flow
            "if" => Some(TokenKind::If),
            "then" => Some(TokenKind::Then),
            "else" => Some(TokenKind::Else),
            "let" => Some(TokenKind::Let),
            "in" => Some(TokenKind::In),

            // Booleans
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "null" | "nil" | "none" => Some(TokenKind::Null),

            // Logical operators
            "and" | "AND" => Some(TokenKind::And),
            "or" | "OR" => Some(TokenKind::Or),
            "not" | "NOT" => Some(TokenKind::Not),

            // Equality synonyms (all map to strict equality)
            "is" => Some(TokenKind::Is),
            "isnt" => Some(TokenKind::Isnt),
            "is_not" => Some(TokenKind::IsNot),
            "eq" => Some(TokenKind::Eq),
            "equals" => Some(TokenKind::Equals),
            "same_as" => Some(TokenKind::SameAs),
            "not_eq" => Some(TokenKind::NotEqKw),

            // SQL-like
            "from" => Some(TokenKind::From),
            "where" => Some(TokenKind::Where),
            "select" => Some(TokenKind::Select),

            // Array operations
            "map" => Some(TokenKind::Map),
            "filter" => Some(TokenKind::Filter),
            "reduce" => Some(TokenKind::Reduce),
            "all" => Some(TokenKind::All),
            "some" => Some(TokenKind::Some),
            // "none" handled above with null
            "merge" => Some(TokenKind::Merge),

            // Data operations
            "missing" => Some(TokenKind::Missing),
            "missing_some" => Some(TokenKind::MissingSome),

            // Functions
            "contains" => Some(TokenKind::Contains),

            // Debug
            "log" => Some(TokenKind::Log),

            _ => None,
        }
    }

    /// Check if this token is an equality operator (including synonyms)
    pub fn is_equality(&self) -> bool {
        matches!(
            self,
            TokenKind::EqEq
                | TokenKind::EqEqEq
                | TokenKind::Is
                | TokenKind::Eq
                | TokenKind::Equals
                | TokenKind::SameAs
        )
    }

    /// Check if this token is an inequality operator (including synonyms)
    pub fn is_inequality(&self) -> bool {
        matches!(
            self,
            TokenKind::NotEq
                | TokenKind::NotEqEq
                | TokenKind::LtGt
                | TokenKind::Isnt
                | TokenKind::IsNot
                | TokenKind::NotEqKw
        )
    }

    /// Check if this is a comparison operator
    pub fn is_comparison(&self) -> bool {
        self.is_equality() || self.is_inequality() || matches!(
            self,
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq
        )
    }

    /// Check if this is an arithmetic operator
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Caret
                | TokenKind::SlashQuestion
                | TokenKind::SlashBang
        )
    }

    /// Check if this is a logical operator
    pub fn is_logical(&self) -> bool {
        matches!(self, TokenKind::And | TokenKind::Or | TokenKind::Not)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::TemplateString(_) => write!(f, "<template>"),
            TokenKind::Bool(b) => write!(f, "{}", b),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::SlashQuestion => write!(f, "/?"),
            TokenKind::SlashBang => write!(f, "/!"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::EqEqEq => write!(f, "==="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::NotEqEq => write!(f, "!=="),
            TokenKind::LtGt => write!(f, "<>"),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::QuestionQuestion => write!(f, "??"),
            TokenKind::Arrow => write!(f, "=>"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::In => write!(f, "in"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Isnt => write!(f, "isnt"),
            TokenKind::IsNot => write!(f, "is_not"),
            TokenKind::Eq => write!(f, "eq"),
            TokenKind::Equals => write!(f, "equals"),
            TokenKind::SameAs => write!(f, "same_as"),
            TokenKind::NotEqKw => write!(f, "not_eq"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Where => write!(f, "where"),
            TokenKind::Select => write!(f, "select"),
            TokenKind::Map => write!(f, "map"),
            TokenKind::Filter => write!(f, "filter"),
            TokenKind::Reduce => write!(f, "reduce"),
            TokenKind::All => write!(f, "all"),
            TokenKind::Some => write!(f, "some"),
            TokenKind::None => write!(f, "none"),
            TokenKind::Merge => write!(f, "merge"),
            TokenKind::Missing => write!(f, "missing"),
            TokenKind::MissingSome => write!(f, "missing_some"),
            TokenKind::Log => write!(f, "log"),
            TokenKind::Contains => write!(f, "contains"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(e) => write!(f, "Error: {}", e),
        }
    }
}
