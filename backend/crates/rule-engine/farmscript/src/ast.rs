//! Abstract Syntax Tree for FarmScript
//!
//! Represents the parsed structure of FarmScript expressions.

use crate::token::Span;

/// A FarmScript expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Literal(Literal),

    // Variable reference
    Var(VarExpr),

    // Binary operations
    Binary(Box<BinaryExpr>),

    // Unary operations
    Unary(Box<UnaryExpr>),

    // Postfix operations (x?)
    Postfix(Box<PostfixExpr>),

    // Conditional: if cond then a else b
    If(Box<IfExpr>),

    // Let binding: let x = expr in body
    Let(Box<LetExpr>),

    // Function call: fn(args...)
    Call(Box<CallExpr>),

    // Method call: obj.method(args...)
    MethodCall(Box<MethodCallExpr>),

    // Property access: obj.prop
    Property(Box<PropertyExpr>),

    // Index access: arr[idx]
    Index(Box<IndexExpr>),

    // Array literal: [a, b, c]
    Array(Vec<Expr>),

    // Lambda: x => expr or (x, y) => expr
    Lambda(Box<LambdaExpr>),

    // SQL-like query: from items where cond select expr
    Query(Box<QueryExpr>),

    // Template string: `Hello {name}!`
    Template(Vec<TemplateExpr>),

    // Null coalescing: a ?? b
    NullCoalesce(Box<Expr>, Box<Expr>),
}

/// A literal value
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

/// Variable reference
#[derive(Debug, Clone, PartialEq)]
pub struct VarExpr {
    pub name: String,
    pub span: Span,
}

/// Binary expression
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Expr,
    pub op: BinaryOp,
    pub right: Expr,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    SafeDivZero,  // /? returns 0 on div by zero
    SafeDivNull,  // /! returns null on div by zero

    // Comparison
    Eq,          // == (loose)
    StrictEq,    // === (strict)
    NotEq,       // !=
    StrictNotEq, // !==
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,

    // Membership
    In,
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq
            | BinaryOp::StrictEq
            | BinaryOp::NotEq
            | BinaryOp::StrictNotEq => 3,
            BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq
            | BinaryOp::In => 4,
            BinaryOp::Add | BinaryOp::Sub => 5,
            BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::SafeDivZero
            | BinaryOp::SafeDivNull => 6,
            BinaryOp::Pow => 7,
        }
    }

    /// Is this operator right-associative?
    pub fn is_right_assoc(&self) -> bool {
        matches!(self, BinaryOp::Pow)
    }
}

/// Unary expression
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,    // not x, !x
    Neg,    // -x
    Plus,   // +x (no-op, for symmetry)
}

/// Postfix expression (x?)
#[derive(Debug, Clone, PartialEq)]
pub struct PostfixExpr {
    pub expr: Expr,
    pub op: PostfixOp,
    pub span: Span,
}

/// Postfix operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostfixOp {
    Truthy, // x? - convert to boolean
}

/// If expression
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Expr,
    pub then_branch: Expr,
    pub else_branch: Option<Expr>,
    pub else_ifs: Vec<(Expr, Expr)>, // (condition, then)
    pub span: Span,
}

/// Let binding expression
#[derive(Debug, Clone, PartialEq)]
pub struct LetExpr {
    pub name: String,
    pub value: Expr,
    pub body: Expr,
    pub span: Span,
}

/// Function call expression
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub function: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Method call expression
#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Property access expression
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyExpr {
    pub object: Expr,
    pub property: String,
    pub span: Span,
}

/// Index access expression
#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub object: Expr,
    pub index: Expr,
    pub span: Span,
}

/// Lambda expression
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub params: Vec<String>,
    pub body: Expr,
    pub span: Span,
}

/// SQL-like query expression
#[derive(Debug, Clone, PartialEq)]
pub struct QueryExpr {
    pub source: Expr,           // from <source>
    pub filter: Option<Expr>,   // where <condition>
    pub projection: Expr,       // select <expr>
    pub span: Span,
}

/// Template string part
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateExpr {
    Literal(String),
    Expr(Expr),
}

impl Expr {
    /// Create a null literal
    pub fn null() -> Self {
        Expr::Literal(Literal::Null)
    }

    /// Create a boolean literal
    pub fn bool(value: bool) -> Self {
        Expr::Literal(Literal::Bool(value))
    }

    /// Create an integer literal
    pub fn int(value: i64) -> Self {
        Expr::Literal(Literal::Integer(value))
    }

    /// Create a float literal
    pub fn float(value: f64) -> Self {
        Expr::Literal(Literal::Float(value))
    }

    /// Create a string literal
    pub fn string(value: impl Into<String>) -> Self {
        Expr::Literal(Literal::String(value.into()))
    }

    /// Create a variable reference
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(VarExpr {
            name: name.into(),
            span: Span::new(0, 0),
        })
    }

    /// Create a binary expression
    pub fn binary(left: Expr, op: BinaryOp, right: Expr) -> Self {
        Expr::Binary(Box::new(BinaryExpr {
            left,
            op,
            right,
            span: Span::new(0, 0),
        }))
    }

    /// Create a unary expression
    pub fn unary(op: UnaryOp, expr: Expr) -> Self {
        Expr::Unary(Box::new(UnaryExpr {
            op,
            expr,
            span: Span::new(0, 0),
        }))
    }

    /// Create a function call
    pub fn call(function: impl Into<String>, args: Vec<Expr>) -> Self {
        Expr::Call(Box::new(CallExpr {
            function: function.into(),
            args,
            span: Span::new(0, 0),
        }))
    }

    /// Create an if expression
    pub fn if_then_else(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Self {
        Expr::If(Box::new(IfExpr {
            condition,
            then_branch,
            else_branch,
            else_ifs: vec![],
            span: Span::new(0, 0),
        }))
    }

    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_) => Span::new(0, 0),
            Expr::Var(v) => v.span,
            Expr::Binary(b) => b.span,
            Expr::Unary(u) => u.span,
            Expr::Postfix(p) => p.span,
            Expr::If(i) => i.span,
            Expr::Let(l) => l.span,
            Expr::Call(c) => c.span,
            Expr::MethodCall(m) => m.span,
            Expr::Property(p) => p.span,
            Expr::Index(i) => i.span,
            Expr::Array(_) => Span::new(0, 0),
            Expr::Lambda(l) => l.span,
            Expr::Query(q) => q.span,
            Expr::Template(_) => Span::new(0, 0),
            Expr::NullCoalesce(_, _) => Span::new(0, 0),
        }
    }

    /// Collect all variable names referenced in this expression
    pub fn collect_variables(&self) -> Vec<&str> {
        let mut vars = Vec::new();
        self.collect_variables_into(&mut vars);
        vars
    }

    fn collect_variables_into<'a>(&'a self, vars: &mut Vec<&'a str>) {
        match self {
            Expr::Var(v) => vars.push(&v.name),
            Expr::Binary(b) => {
                b.left.collect_variables_into(vars);
                b.right.collect_variables_into(vars);
            }
            Expr::Unary(u) => u.expr.collect_variables_into(vars),
            Expr::Postfix(p) => p.expr.collect_variables_into(vars),
            Expr::If(i) => {
                i.condition.collect_variables_into(vars);
                i.then_branch.collect_variables_into(vars);
                if let Some(ref else_branch) = i.else_branch {
                    else_branch.collect_variables_into(vars);
                }
                for (cond, then) in &i.else_ifs {
                    cond.collect_variables_into(vars);
                    then.collect_variables_into(vars);
                }
            }
            Expr::Let(l) => {
                l.value.collect_variables_into(vars);
                l.body.collect_variables_into(vars);
            }
            Expr::Call(c) => {
                for arg in &c.args {
                    arg.collect_variables_into(vars);
                }
            }
            Expr::MethodCall(m) => {
                m.object.collect_variables_into(vars);
                for arg in &m.args {
                    arg.collect_variables_into(vars);
                }
            }
            Expr::Property(p) => p.object.collect_variables_into(vars),
            Expr::Index(i) => {
                i.object.collect_variables_into(vars);
                i.index.collect_variables_into(vars);
            }
            Expr::Array(items) => {
                for item in items {
                    item.collect_variables_into(vars);
                }
            }
            Expr::Lambda(l) => l.body.collect_variables_into(vars),
            Expr::Query(q) => {
                q.source.collect_variables_into(vars);
                if let Some(ref filter) = q.filter {
                    filter.collect_variables_into(vars);
                }
                q.projection.collect_variables_into(vars);
            }
            Expr::Template(parts) => {
                for part in parts {
                    if let TemplateExpr::Expr(e) = part {
                        e.collect_variables_into(vars);
                    }
                }
            }
            Expr::NullCoalesce(a, b) => {
                a.collect_variables_into(vars);
                b.collect_variables_into(vars);
            }
            Expr::Literal(_) => {}
        }
    }
}
