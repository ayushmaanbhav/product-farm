//! Bytecode compiler for JSON Logic expressions
//!
//! Compiles AST to a stack-based bytecode for fast evaluation.

use product_farm_core::Value;
use std::collections::HashMap;

use crate::{Expr, JsonLogicError, JsonLogicResult, VarExpr};

/// Bytecode operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    /// No operation
    Nop = 0,

    // Stack operations
    /// Push constant from pool: [idx_lo, idx_hi]
    LoadConst = 1,
    /// Push variable by index: [idx_lo, idx_hi]
    LoadVar = 2,
    /// Duplicate top of stack
    Dup = 3,
    /// Pop top of stack
    Pop = 4,
    /// Push variable by index with default from constant pool: [var_idx_lo, var_idx_hi, default_idx_lo, default_idx_hi]
    LoadVarWithDefault = 5,

    // Comparison
    /// Equal (==)
    Eq = 10,
    /// Strict equal (===)
    StrictEq = 11,
    /// Not equal (!=)
    Ne = 12,
    /// Strict not equal (!==)
    StrictNe = 13,
    /// Less than (<)
    Lt = 14,
    /// Less than or equal (<=)
    Le = 15,
    /// Greater than (>)
    Gt = 16,
    /// Greater than or equal (>=)
    Ge = 17,

    // Logical
    /// Logical NOT
    Not = 20,
    /// To boolean
    ToBool = 21,
    /// Logical AND (short-circuit): [jump_offset_lo, jump_offset_hi]
    And = 22,
    /// Logical OR (short-circuit): [jump_offset_lo, jump_offset_hi]
    Or = 23,

    // Arithmetic
    /// Add two values
    Add = 30,
    /// Subtract
    Sub = 31,
    /// Multiply
    Mul = 32,
    /// Divide
    Div = 33,
    /// Modulo
    Mod = 34,
    /// Negate
    Neg = 35,
    /// Minimum of N values: [count]
    Min = 36,
    /// Maximum of N values: [count]
    Max = 37,

    // String
    /// Concatenate N strings: [count]
    Cat = 40,
    /// Substring: [has_length]
    Substr = 41,

    // Control flow
    /// Unconditional jump: [offset_lo, offset_hi]
    Jump = 50,
    /// Jump if false: [offset_lo, offset_hi]
    JumpIfFalse = 51,
    /// Jump if true: [offset_lo, offset_hi]
    JumpIfTrue = 52,

    // Array operations
    /// Check if value in array
    In = 60,

    // Special
    /// Return result
    Return = 255,
}

impl OpCode {
    /// Get the opcode from a byte
    #[inline]
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(OpCode::Nop),
            1 => Some(OpCode::LoadConst),
            2 => Some(OpCode::LoadVar),
            3 => Some(OpCode::Dup),
            4 => Some(OpCode::Pop),
            5 => Some(OpCode::LoadVarWithDefault),
            10 => Some(OpCode::Eq),
            11 => Some(OpCode::StrictEq),
            12 => Some(OpCode::Ne),
            13 => Some(OpCode::StrictNe),
            14 => Some(OpCode::Lt),
            15 => Some(OpCode::Le),
            16 => Some(OpCode::Gt),
            17 => Some(OpCode::Ge),
            20 => Some(OpCode::Not),
            21 => Some(OpCode::ToBool),
            22 => Some(OpCode::And),
            23 => Some(OpCode::Or),
            30 => Some(OpCode::Add),
            31 => Some(OpCode::Sub),
            32 => Some(OpCode::Mul),
            33 => Some(OpCode::Div),
            34 => Some(OpCode::Mod),
            35 => Some(OpCode::Neg),
            36 => Some(OpCode::Min),
            37 => Some(OpCode::Max),
            40 => Some(OpCode::Cat),
            41 => Some(OpCode::Substr),
            50 => Some(OpCode::Jump),
            51 => Some(OpCode::JumpIfFalse),
            52 => Some(OpCode::JumpIfTrue),
            60 => Some(OpCode::In),
            255 => Some(OpCode::Return),
            _ => None,
        }
    }
}

/// Compiled bytecode for a JSON Logic expression
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompiledBytecode {
    /// The bytecode instructions
    pub bytecode: Vec<u8>,
    /// Constant pool
    pub constants: Vec<Value>,
    /// Variable name to index mapping
    pub variable_map: HashMap<String, u16>,
    /// Variable names in order (index -> name)
    pub variable_names: Vec<String>,
}

impl CompiledBytecode {
    /// Get the size of the bytecode in bytes
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }

    /// Get the number of variables
    pub fn variable_count(&self) -> usize {
        self.variable_names.len()
    }

    /// Get the number of constants
    pub fn constant_count(&self) -> usize {
        self.constants.len()
    }
}

/// Bytecode compiler
pub struct Compiler {
    /// Current bytecode being built
    bytecode: Vec<u8>,
    /// Constant pool
    constants: Vec<Value>,
    /// Variable mapping (name -> index)
    variables: HashMap<String, u16>,
    /// Variable names in order
    variable_names: Vec<String>,
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            bytecode: Vec::with_capacity(256),
            constants: Vec::with_capacity(32),
            variables: HashMap::with_capacity(16),
            variable_names: Vec::with_capacity(16),
        }
    }

    /// Compile an expression to bytecode
    pub fn compile(&mut self, expr: &Expr) -> JsonLogicResult<CompiledBytecode> {
        self.bytecode.clear();
        self.constants.clear();
        self.variables.clear();
        self.variable_names.clear();

        // First pass: collect all variables
        self.collect_variables(expr);

        // Second pass: generate bytecode
        self.compile_expr(expr)?;

        // Add return instruction
        self.emit(OpCode::Return);

        Ok(CompiledBytecode {
            bytecode: std::mem::take(&mut self.bytecode),
            constants: std::mem::take(&mut self.constants),
            variable_map: std::mem::take(&mut self.variables),
            variable_names: std::mem::take(&mut self.variable_names),
        })
    }

    /// Collect all variables referenced in the expression
    fn collect_variables(&mut self, expr: &Expr) {
        for var_path in expr.collect_variables() {
            if !self.variables.contains_key(var_path) {
                let idx = self.variable_names.len() as u16;
                self.variables.insert(var_path.to_string(), idx);
                self.variable_names.push(var_path.to_string());
            }
        }
    }

    /// Emit a single opcode
    fn emit(&mut self, op: OpCode) {
        self.bytecode.push(op as u8);
    }

    /// Emit opcode with u16 argument
    fn emit_u16(&mut self, op: OpCode, arg: u16) {
        self.bytecode.push(op as u8);
        self.bytecode.push((arg & 0xFF) as u8);
        self.bytecode.push((arg >> 8) as u8);
    }

    /// Emit LoadVarWithDefault instruction with variable index and default constant index
    fn emit_var_with_default(&mut self, var_idx: u16, default_idx: u16) {
        self.bytecode.push(OpCode::LoadVarWithDefault as u8);
        self.bytecode.push((var_idx & 0xFF) as u8);
        self.bytecode.push((var_idx >> 8) as u8);
        self.bytecode.push((default_idx & 0xFF) as u8);
        self.bytecode.push((default_idx >> 8) as u8);
    }

    /// Emit opcode with u8 argument
    fn emit_u8(&mut self, op: OpCode, arg: u8) {
        self.bytecode.push(op as u8);
        self.bytecode.push(arg);
    }

    /// Get current bytecode position
    fn position(&self) -> usize {
        self.bytecode.len()
    }

    /// Add a constant and return its index
    fn add_constant(&mut self, value: Value) -> u16 {
        // Check if constant already exists
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return i as u16;
            }
        }

        let idx = self.constants.len() as u16;
        self.constants.push(value);
        idx
    }

    /// Get variable index
    fn get_variable_index(&self, path: &str) -> Option<u16> {
        self.variables.get(path).copied()
    }

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr) -> JsonLogicResult<()> {
        match expr {
            Expr::Literal(value) => {
                let idx = self.add_constant(value.clone());
                self.emit_u16(OpCode::LoadConst, idx);
            }

            Expr::Var(var) => {
                self.compile_var(var)?;
            }

            // Comparison
            Expr::Eq(a, b) => {
                self.compile_expr(a)?;
                self.compile_expr(b)?;
                self.emit(OpCode::Eq);
            }
            Expr::StrictEq(a, b) => {
                self.compile_expr(a)?;
                self.compile_expr(b)?;
                self.emit(OpCode::StrictEq);
            }
            Expr::Ne(a, b) => {
                self.compile_expr(a)?;
                self.compile_expr(b)?;
                self.emit(OpCode::Ne);
            }
            Expr::StrictNe(a, b) => {
                self.compile_expr(a)?;
                self.compile_expr(b)?;
                self.emit(OpCode::StrictNe);
            }
            Expr::Lt(exprs) => self.compile_chain_comparison(exprs, OpCode::Lt)?,
            Expr::Le(exprs) => self.compile_chain_comparison(exprs, OpCode::Le)?,
            Expr::Gt(exprs) => self.compile_chain_comparison(exprs, OpCode::Gt)?,
            Expr::Ge(exprs) => self.compile_chain_comparison(exprs, OpCode::Ge)?,

            // Logical
            Expr::Not(a) => {
                self.compile_expr(a)?;
                self.emit(OpCode::Not);
            }
            Expr::ToBool(a) => {
                self.compile_expr(a)?;
                self.emit(OpCode::ToBool);
            }
            Expr::And(exprs) => self.compile_and(exprs)?,
            Expr::Or(exprs) => self.compile_or(exprs)?,

            // Conditional
            Expr::If(exprs) => self.compile_if(exprs)?,
            Expr::Ternary(cond, then, else_) => {
                self.compile_if(&[(**cond).clone(), (**then).clone(), (**else_).clone()])?;
            }

            // Arithmetic
            Expr::Add(exprs) => self.compile_nary_op(exprs, OpCode::Add)?,
            Expr::Sub(exprs) => {
                if exprs.len() == 1 {
                    self.compile_expr(&exprs[0])?;
                    self.emit(OpCode::Neg);
                } else {
                    self.compile_binary_op(&exprs[0], &exprs[1], OpCode::Sub)?;
                }
            }
            Expr::Mul(exprs) => self.compile_nary_op(exprs, OpCode::Mul)?,
            Expr::Div(a, b) => self.compile_binary_op(a, b, OpCode::Div)?,
            Expr::Mod(a, b) => self.compile_binary_op(a, b, OpCode::Mod)?,
            Expr::Min(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                }
                self.emit_u8(OpCode::Min, exprs.len() as u8);
            }
            Expr::Max(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                }
                self.emit_u8(OpCode::Max, exprs.len() as u8);
            }

            // String
            Expr::Cat(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                }
                self.emit_u8(OpCode::Cat, exprs.len() as u8);
            }
            Expr::Substr(s, start, len) => {
                self.compile_expr(s)?;
                self.compile_expr(start)?;
                if let Some(l) = len {
                    self.compile_expr(l)?;
                    self.emit_u8(OpCode::Substr, 1);
                } else {
                    self.emit_u8(OpCode::Substr, 0);
                }
            }

            // Array
            Expr::In(val, arr) => {
                self.compile_expr(val)?;
                self.compile_expr(arr)?;
                self.emit(OpCode::In);
            }

            // Not yet supported in bytecode - fall back to interpreter
            Expr::Map(_, _)
            | Expr::Filter(_, _)
            | Expr::Reduce(_, _, _)
            | Expr::All(_, _)
            | Expr::Some(_, _)
            | Expr::None(_, _)
            | Expr::Merge(_)
            | Expr::Missing(_)
            | Expr::MissingSome(_, _)
            | Expr::Log(_) => {
                return Err(JsonLogicError::CompilationError(
                    "Operation not supported in bytecode mode".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Compile variable access
    fn compile_var(&mut self, var: &VarExpr) -> JsonLogicResult<()> {
        if var.path.is_empty() {
            // Empty path = entire data object (not commonly used)
            return Err(JsonLogicError::CompilationError(
                "Empty var path not supported in bytecode".to_string(),
            ));
        }

        if let Some(idx) = self.get_variable_index(&var.path) {
            // Handle default value if variable might be missing
            if let Some(default) = &var.default {
                // Use LoadVarWithDefault: if var is null, use default from constant pool
                let default_idx = self.add_constant(default.clone());
                self.emit_var_with_default(idx, default_idx);
            } else {
                self.emit_u16(OpCode::LoadVar, idx);
            }
        } else {
            return Err(JsonLogicError::VariableNotFound(var.path.clone()));
        }

        Ok(())
    }

    /// Compile binary operation
    fn compile_binary_op(&mut self, a: &Expr, b: &Expr, op: OpCode) -> JsonLogicResult<()> {
        self.compile_expr(a)?;
        self.compile_expr(b)?;
        self.emit(op);
        Ok(())
    }

    /// Compile n-ary operation (left-associative)
    fn compile_nary_op(&mut self, exprs: &[Expr], op: OpCode) -> JsonLogicResult<()> {
        if exprs.is_empty() {
            return Err(JsonLogicError::InvalidArgumentCount {
                op: format!("{:?}", op),
                expected: "at least 1".to_string(),
                actual: 0,
            });
        }

        self.compile_expr(&exprs[0])?;
        for expr in &exprs[1..] {
            self.compile_expr(expr)?;
            self.emit(op);
        }

        Ok(())
    }

    /// Compile chain comparison (a < b < c)
    fn compile_chain_comparison(&mut self, exprs: &[Expr], op: OpCode) -> JsonLogicResult<()> {
        if exprs.len() == 2 {
            // Simple case: a < b
            self.compile_expr(&exprs[0])?;
            self.compile_expr(&exprs[1])?;
            self.emit(op);
        } else {
            // Chain case: a < b < c => (a < b) && (b < c)
            // For simplicity, just compile as AND of pairwise comparisons
            for i in 0..exprs.len() - 1 {
                self.compile_expr(&exprs[i])?;
                self.compile_expr(&exprs[i + 1])?;
                self.emit(op);
                if i > 0 {
                    self.emit(OpCode::And);
                    // Note: This is simplified; proper short-circuit needs jump patching
                }
            }
        }

        Ok(())
    }

    /// Compile AND with short-circuit evaluation
    fn compile_and(&mut self, exprs: &[Expr]) -> JsonLogicResult<()> {
        if exprs.is_empty() {
            let idx = self.add_constant(Value::Bool(true));
            self.emit_u16(OpCode::LoadConst, idx);
            return Ok(());
        }

        let mut end_jumps = Vec::new();

        for (i, expr) in exprs.iter().enumerate() {
            self.compile_expr(expr)?;

            if i < exprs.len() - 1 {
                // If false, short-circuit to end
                self.emit(OpCode::Dup);
                self.emit(OpCode::ToBool);
                let jump_pos = self.position();
                self.emit_u16(OpCode::JumpIfFalse, 0); // Placeholder
                end_jumps.push(jump_pos);
                self.emit(OpCode::Pop); // Pop the duplicated value
            }
        }

        // Patch all jumps to point to current position
        let end_pos = self.position();
        for jump_pos in end_jumps {
            let offset = (end_pos - jump_pos - 3) as u16;
            self.bytecode[jump_pos + 1] = (offset & 0xFF) as u8;
            self.bytecode[jump_pos + 2] = (offset >> 8) as u8;
        }

        Ok(())
    }

    /// Compile OR with short-circuit evaluation
    fn compile_or(&mut self, exprs: &[Expr]) -> JsonLogicResult<()> {
        if exprs.is_empty() {
            let idx = self.add_constant(Value::Bool(false));
            self.emit_u16(OpCode::LoadConst, idx);
            return Ok(());
        }

        let mut end_jumps = Vec::new();

        for (i, expr) in exprs.iter().enumerate() {
            self.compile_expr(expr)?;

            if i < exprs.len() - 1 {
                // If true, short-circuit to end
                self.emit(OpCode::Dup);
                self.emit(OpCode::ToBool);
                let jump_pos = self.position();
                self.emit_u16(OpCode::JumpIfTrue, 0); // Placeholder
                end_jumps.push(jump_pos);
                self.emit(OpCode::Pop); // Pop the duplicated value
            }
        }

        // Patch all jumps to point to current position
        let end_pos = self.position();
        for jump_pos in end_jumps {
            let offset = (end_pos - jump_pos - 3) as u16;
            self.bytecode[jump_pos + 1] = (offset & 0xFF) as u8;
            self.bytecode[jump_pos + 2] = (offset >> 8) as u8;
        }

        Ok(())
    }

    /// Compile if-then-else
    fn compile_if(&mut self, exprs: &[Expr]) -> JsonLogicResult<()> {
        if exprs.is_empty() {
            let idx = self.add_constant(Value::Null);
            self.emit_u16(OpCode::LoadConst, idx);
            return Ok(());
        }

        if exprs.len() == 1 {
            // Just a condition, no then/else
            self.compile_expr(&exprs[0])?;
            return Ok(());
        }

        // Collect all end jumps (for patching)
        let mut end_jumps = Vec::new();

        // Process condition-then pairs
        let mut i = 0;
        while i < exprs.len() {
            if i + 1 < exprs.len() {
                // We have a condition and a then-branch
                self.compile_expr(&exprs[i])?;
                self.emit(OpCode::ToBool);

                let false_jump_pos = self.position();
                self.emit_u16(OpCode::JumpIfFalse, 0); // Placeholder

                // Then branch
                self.compile_expr(&exprs[i + 1])?;

                if i + 2 < exprs.len() {
                    // More branches follow, jump to end
                    let end_jump_pos = self.position();
                    self.emit_u16(OpCode::Jump, 0); // Placeholder
                    end_jumps.push(end_jump_pos);
                }

                // Patch false jump
                let current_pos = self.position();
                let offset = (current_pos - false_jump_pos - 3) as u16;
                self.bytecode[false_jump_pos + 1] = (offset & 0xFF) as u8;
                self.bytecode[false_jump_pos + 2] = (offset >> 8) as u8;

                i += 2;
            } else {
                // Final else branch (no condition)
                self.compile_expr(&exprs[i])?;
                i += 1;
            }
        }

        // If no else branch, push null
        if exprs.len().is_multiple_of(2) {
            // Even number = all condition-then pairs, no else
            // The last false jump already points here
        }

        // Patch all end jumps
        let end_pos = self.position();
        for jump_pos in end_jumps {
            let offset = (end_pos - jump_pos - 3) as u16;
            self.bytecode[jump_pos + 1] = (offset & 0xFF) as u8;
            self.bytecode[jump_pos + 2] = (offset >> 8) as u8;
        }

        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use serde_json::json;

    #[test]
    fn test_compile_literal() {
        let expr = parse(&json!(42)).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        assert!(bytecode.size() > 0);
        assert_eq!(bytecode.constant_count(), 1);
    }

    #[test]
    fn test_compile_var() {
        let expr = parse(&json!({"var": "age"})).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        assert_eq!(bytecode.variable_count(), 1);
        assert!(bytecode.variable_map.contains_key("age"));
    }

    #[test]
    fn test_compile_comparison() {
        let expr = parse(&json!({">": [{"var": "age"}, 60]})).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        assert_eq!(bytecode.variable_count(), 1);
        assert_eq!(bytecode.constant_count(), 1); // 60
    }

    #[test]
    fn test_compile_arithmetic() {
        let expr = parse(&json!({"*": [{"var": "base"}, 1.2]})).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        assert_eq!(bytecode.variable_count(), 1);
        assert_eq!(bytecode.constant_count(), 1); // 1.2
    }

    #[test]
    fn test_compile_if() {
        let expr = parse(&json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                {"*": [{"var": "base"}, 1.2]},
                {"var": "base"}
            ]
        }))
        .unwrap();

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        assert_eq!(bytecode.variable_count(), 2); // age, base
        assert!(bytecode.size() > 10); // Should have substantial bytecode
    }
}
