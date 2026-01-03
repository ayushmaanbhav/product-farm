//! Lexer/Tokenizer for FarmScript
//!
//! Converts source text into a stream of tokens.
//!
//! Special handling:
//! - `/path/to/var` at token start = path-style variable
//! - `a / b` = division operator

use crate::token::{Token, TokenKind, Span, TemplatePart};

/// The lexer that tokenizes FarmScript source
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    /// Track if we just emitted a value (for division vs path disambiguation)
    last_was_value: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            last_was_value: false,
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.current_pos;

        let Some((_pos, ch)) = self.advance() else {
            return Token::new(TokenKind::Eof, Span::new(start, start), "");
        };

        let kind = match ch {
            // Single-character tokens
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,

            // Division or path-style variable
            '/' => self.lex_slash_or_path(start),

            // Comparison and equality
            '=' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        TokenKind::EqEqEq // ===
                    } else {
                        TokenKind::EqEq // ==
                    }
                } else if self.match_char('>') {
                    TokenKind::Arrow // =>
                } else {
                    TokenKind::Assign // = (assignment for let)
                }
            }

            '!' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        TokenKind::NotEqEq // !==
                    } else {
                        TokenKind::NotEq // !=
                    }
                } else {
                    TokenKind::Not // !
                }
            }

            '<' => {
                if self.match_char('=') {
                    TokenKind::LtEq // <=
                } else if self.match_char('>') {
                    TokenKind::LtGt // <>
                } else {
                    TokenKind::Lt // <
                }
            }

            '>' => {
                if self.match_char('=') {
                    TokenKind::GtEq // >=
                } else {
                    TokenKind::Gt // >
                }
            }

            '?' => {
                if self.match_char('?') {
                    TokenKind::QuestionQuestion // ??
                } else {
                    TokenKind::Question // ? (postfix truthy)
                }
            }

            '&' => {
                if self.match_char('&') {
                    TokenKind::And // &&
                } else {
                    TokenKind::Error("Unexpected '&'. Use '&&' for logical AND.".into())
                }
            }

            '|' => {
                if self.match_char('|') {
                    TokenKind::Or // ||
                } else {
                    TokenKind::Error("Unexpected '|'. Use '||' for logical OR.".into())
                }
            }

            // Strings
            '"' => self.lex_string('"'),
            '\'' => self.lex_string('\''),

            // Template strings
            '`' => self.lex_template_string(),

            // Numbers
            '0'..='9' => self.lex_number(ch, start),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(ch, start),

            // Comments
            '#' => {
                self.skip_line_comment();
                return self.next_token();
            }

            _ => TokenKind::Error(format!("Unexpected character: '{}'", ch)),
        };

        // Track if this was a value-producing token (for / disambiguation)
        self.last_was_value = matches!(
            &kind,
            TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::TemplateString(_)
                | TokenKind::Bool(_)
                | TokenKind::Null
                | TokenKind::Ident(_)
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::Question // x? is a value
                | TokenKind::True
                | TokenKind::False
        );

        let end = self.current_pos;
        let lexeme = &self.source[start..end];
        Token::new(kind, Span::new(start, end), lexeme)
    }

    /// Lex '/' - could be division, safe division, or path-style variable
    fn lex_slash_or_path(&mut self, start: usize) -> TokenKind {
        // Check for safe division operators first
        if self.match_char('?') {
            return TokenKind::SlashQuestion; // /?
        }
        if self.match_char('!') {
            return TokenKind::SlashBang; // /!
        }

        // If the last token was a value, this is division
        if self.last_was_value {
            return TokenKind::Slash;
        }

        // Check if this starts a path-style variable: /path/to/var
        // Path must start with / followed by alphanumeric or underscore
        if let Some(&(_, next_ch)) = self.chars.peek() {
            if next_ch.is_alphanumeric() || next_ch == '_' {
                // This is a path-style variable
                return self.lex_path_variable(start);
            }
        }

        // Otherwise it's division
        TokenKind::Slash
    }

    /// Lex a path-style variable like /users/count
    fn lex_path_variable(&mut self, _start: usize) -> TokenKind {
        let mut path = String::from("/");

        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '/' || ch == '-' {
                path.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Remove trailing slash if present
        if path.ends_with('/') && path.len() > 1 {
            path.pop();
        }

        TokenKind::Ident(path)
    }

    /// Lex a string literal
    fn lex_string(&mut self, quote: char) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Error("Unterminated string".into()),
                Some((_, ch)) if ch == quote => break,
                Some((_, '\\')) => {
                    // Escape sequence
                    match self.advance() {
                        None => return TokenKind::Error("Unterminated escape sequence".into()),
                        Some((_, 'n')) => value.push('\n'),
                        Some((_, 't')) => value.push('\t'),
                        Some((_, 'r')) => value.push('\r'),
                        Some((_, '\\')) => value.push('\\'),
                        Some((_, c)) if c == quote => value.push(c),
                        Some((_, c)) => {
                            value.push('\\');
                            value.push(c);
                        }
                    }
                }
                Some((_, ch)) => value.push(ch),
            }
        }

        TokenKind::String(value)
    }

    /// Lex a template string like `Hello {name}!`
    fn lex_template_string(&mut self) -> TokenKind {
        let mut parts = Vec::new();
        let mut current_literal = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Error("Unterminated template string".into()),
                Some((_, '`')) => {
                    // End of template string
                    if !current_literal.is_empty() {
                        parts.push(TemplatePart::Literal(current_literal));
                    }
                    break;
                }
                Some((_, '{')) => {
                    // Start of expression
                    if !current_literal.is_empty() {
                        parts.push(TemplatePart::Literal(current_literal));
                        current_literal = String::new();
                    }

                    // Read until matching }
                    let mut expr = String::new();
                    let mut brace_depth = 1;

                    loop {
                        match self.advance() {
                            None => return TokenKind::Error("Unterminated template expression".into()),
                            Some((_, '{')) => {
                                brace_depth += 1;
                                expr.push('{');
                            }
                            Some((_, '}')) => {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    break;
                                }
                                expr.push('}');
                            }
                            Some((_, ch)) => expr.push(ch),
                        }
                    }

                    parts.push(TemplatePart::Expr(expr.trim().to_string()));
                }
                Some((_, '\\')) => {
                    // Escape sequence
                    match self.advance() {
                        None => return TokenKind::Error("Unterminated escape sequence".into()),
                        Some((_, 'n')) => current_literal.push('\n'),
                        Some((_, 't')) => current_literal.push('\t'),
                        Some((_, 'r')) => current_literal.push('\r'),
                        Some((_, '\\')) => current_literal.push('\\'),
                        Some((_, '`')) => current_literal.push('`'),
                        Some((_, '{')) => current_literal.push('{'),
                        Some((_, c)) => {
                            current_literal.push('\\');
                            current_literal.push(c);
                        }
                    }
                }
                Some((_, ch)) => current_literal.push(ch),
            }
        }

        TokenKind::TemplateString(parts)
    }

    /// Lex a number (integer or float)
    fn lex_number(&mut self, first_digit: char, _start: usize) -> TokenKind {
        let mut num_str = String::from(first_digit);
        let mut is_float = false;

        // Integer part
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    num_str.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if let Some(&(_, '.')) = self.chars.peek() {
            // Look ahead to see if it's followed by a digit
            let chars_clone: Vec<_> = self.source[self.current_pos..].char_indices().collect();
            if chars_clone.len() > 1 && chars_clone[1].1.is_ascii_digit() {
                is_float = true;
                self.advance(); // consume '.'
                num_str.push('.');

                // Fractional part
                while let Some(&(_, ch)) = self.chars.peek() {
                    if ch.is_ascii_digit() || ch == '_' {
                        if ch != '_' {
                            num_str.push(ch);
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        // Check for exponent
        if let Some(&(_, ch)) = self.chars.peek() {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                num_str.push(ch);
                self.advance();

                // Optional sign
                if let Some(&(_, sign)) = self.chars.peek() {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }

                // Exponent digits
                while let Some(&(_, ch)) = self.chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        if is_float {
            match num_str.parse::<f64>() {
                Ok(n) => TokenKind::Float(n),
                Err(_) => TokenKind::Error(format!("Invalid float: {}", num_str)),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => TokenKind::Integer(n),
                Err(_) => TokenKind::Error(format!("Invalid integer: {}", num_str)),
            }
        }
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self, first_char: char, _start: usize) -> TokenKind {
        let mut ident = String::from(first_char);

        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        TokenKind::from_keyword(&ident).unwrap_or(TokenKind::Ident(ident))
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a line comment (# ...)
    fn skip_line_comment(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.current_pos = pos + ch.len_utf8();
        }
        result
    }

    /// Match and consume a character if it matches
    fn match_char(&mut self, expected: char) -> bool {
        if let Some(&(_, ch)) = self.chars.peek() {
            if ch == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Peek at the current character without consuming
    #[allow(dead_code)]
    fn peek(&self) -> Option<char> {
        self.chars.clone().peek().map(|&(_, ch)| ch)
    }

    /// Collect all tokens (for testing)
    pub fn collect_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if matches!(token.kind, TokenKind::Eof) {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .collect_tokens()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_simple_tokens() {
        assert_eq!(
            lex("+ - * / %"),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(
            lex("== === != !== < > <= >= <>"),
            vec![
                TokenKind::EqEq,
                TokenKind::EqEqEq,
                TokenKind::NotEq,
                TokenKind::NotEqEq,
                TokenKind::Lt,
                TokenKind::Gt,
                TokenKind::LtEq,
                TokenKind::GtEq,
                TokenKind::LtGt,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(
            lex("and or not && || !"),
            vec![
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Not,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Not,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            lex("if then else true false null"),
            vec![
                TokenKind::If,
                TokenKind::Then,
                TokenKind::Else,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Null,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_equality_synonyms() {
        assert_eq!(
            lex("is isnt is_not eq equals same_as not_eq"),
            vec![
                TokenKind::Is,
                TokenKind::Isnt,
                TokenKind::IsNot,
                TokenKind::Eq,
                TokenKind::Equals,
                TokenKind::SameAs,
                TokenKind::NotEqKw,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_integers() {
        assert_eq!(
            lex("0 42 1_000_000"),
            vec![
                TokenKind::Integer(0),
                TokenKind::Integer(42),
                TokenKind::Integer(1000000),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_floats() {
        assert_eq!(
            lex("3.14 0.5 1e10 2.5e-3"),
            vec![
                TokenKind::Float(3.14),
                TokenKind::Float(0.5),
                TokenKind::Float(1e10),
                TokenKind::Float(2.5e-3),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = lex(r#""hello" 'world' "with \"escape\"""#);
        assert_eq!(
            tokens,
            vec![
                TokenKind::String("hello".into()),
                TokenKind::String("world".into()),
                TokenKind::String("with \"escape\"".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(
            lex("foo bar_baz camelCase"),
            vec![
                TokenKind::Ident("foo".into()),
                TokenKind::Ident("bar_baz".into()),
                TokenKind::Ident("camelCase".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_path_variables() {
        // Single path variable at start
        assert_eq!(
            lex("/users/count"),
            vec![
                TokenKind::Ident("/users/count".into()),
                TokenKind::Eof,
            ]
        );

        // Path variable after operator
        assert_eq!(
            lex("/users/count + /api/endpoint"),
            vec![
                TokenKind::Ident("/users/count".into()),
                TokenKind::Plus,
                TokenKind::Ident("/api/endpoint".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_division_vs_path() {
        // After a value, / is division
        assert_eq!(
            lex("a / b"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::Slash,
                TokenKind::Ident("b".into()),
                TokenKind::Eof,
            ]
        );

        // At start or after operator, /foo is a path
        assert_eq!(
            lex("/foo + /bar"),
            vec![
                TokenKind::Ident("/foo".into()),
                TokenKind::Plus,
                TokenKind::Ident("/bar".into()),
                TokenKind::Eof,
            ]
        );

        // Mixed: a / b + /path
        assert_eq!(
            lex("a / b + /path"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::Slash,
                TokenKind::Ident("b".into()),
                TokenKind::Plus,
                TokenKind::Ident("/path".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_safe_division() {
        assert_eq!(
            lex("a /? b /! c"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::SlashQuestion,
                TokenKind::Ident("b".into()),
                TokenKind::SlashBang,
                TokenKind::Ident("c".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_template_string() {
        let tokens = lex("`Hello {name}!`");
        assert_eq!(tokens.len(), 2); // template + eof
        match &tokens[0] {
            TokenKind::TemplateString(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts[0], TemplatePart::Literal("Hello ".into()));
                assert_eq!(parts[1], TemplatePart::Expr("name".into()));
                assert_eq!(parts[2], TemplatePart::Literal("!".into()));
            }
            _ => panic!("Expected template string"),
        }
    }

    #[test]
    fn test_question_operators() {
        assert_eq!(
            lex("x? a ?? b"),
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Question,
                TokenKind::Ident("a".into()),
                TokenKind::QuestionQuestion,
                TokenKind::Ident("b".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_arrow() {
        assert_eq!(
            lex("x => x * 2"),
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Arrow,
                TokenKind::Ident("x".into()),
                TokenKind::Star,
                TokenKind::Integer(2),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_sql_keywords() {
        assert_eq!(
            lex("from items where x > 0 select y"),
            vec![
                TokenKind::From,
                TokenKind::Ident("items".into()),
                TokenKind::Where,
                TokenKind::Ident("x".into()),
                TokenKind::Gt,
                TokenKind::Integer(0),
                TokenKind::Select,
                TokenKind::Ident("y".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_array_operations() {
        assert_eq!(
            lex("map filter reduce all some merge"),
            vec![
                TokenKind::Map,
                TokenKind::Filter,
                TokenKind::Reduce,
                TokenKind::All,
                TokenKind::Some,
                TokenKind::Merge,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comments() {
        assert_eq!(
            lex("x + y # this is a comment\nz"),
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Plus,
                TokenKind::Ident("y".into()),
                TokenKind::Ident("z".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let tokens = lex("alert_acknowledged and time_since_alert_secs < 120");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("alert_acknowledged".into()),
                TokenKind::And,
                TokenKind::Ident("time_since_alert_secs".into()),
                TokenKind::Lt,
                TokenKind::Integer(120),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_clamp_expression() {
        let tokens = lex("clamp(0, 100, raw_score)");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("clamp".into()),
                TokenKind::LParen,
                TokenKind::Integer(0),
                TokenKind::Comma,
                TokenKind::Integer(100),
                TokenKind::Comma,
                TokenKind::Ident("raw_score".into()),
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }
}
