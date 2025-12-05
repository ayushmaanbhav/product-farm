//! JSON Logic Parser, AST, and Bytecode Compiler
//!
//! This crate provides:
//! - Parsing JSON Logic expressions into an optimized AST
//! - Bytecode compilation for fast evaluation
//! - Tiered execution (interpreted AST → bytecode VM)
//! - Automatic tier promotion based on evaluation count
//! - Bytecode persistence via `PersistedRule` serialization

pub mod ast;
pub mod parser;
pub mod compiler;
pub mod vm;
pub mod evaluator;
pub mod error;
pub mod operations;
pub mod tiered;

pub use ast::*;
pub use parser::*;
pub use compiler::*;
pub use vm::*;
pub use evaluator::*;
pub use error::*;
pub use tiered::*;
