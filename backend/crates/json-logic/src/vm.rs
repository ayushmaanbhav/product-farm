//! Bytecode Virtual Machine for JSON Logic
//!
//! A fast stack-based VM for executing compiled JSON Logic expressions.

use product_farm_core::Value;
use rust_decimal::Decimal;
use smallvec::SmallVec;

use crate::{CompiledBytecode, JsonLogicError, JsonLogicResult, OpCode};

/// Maximum stack depth to prevent overflow
const MAX_STACK_DEPTH: usize = 1024;

/// Execution context containing variable values
pub struct EvalContext {
    /// Variable values by index (matching bytecode.variable_names order)
    pub values: Vec<Value>,
}

impl EvalContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Create context with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    /// Build context from a map of variable names to values
    pub fn from_map(
        map: &std::collections::HashMap<String, Value>,
        bytecode: &CompiledBytecode,
    ) -> Self {
        let mut values = vec![Value::Null; bytecode.variable_names.len()];
        for (name, idx) in &bytecode.variable_map {
            if let Some(value) = map.get(name) {
                values[*idx as usize] = value.clone();
            }
        }
        Self { values }
    }

    /// Build context from JSON value (for nested access)
    pub fn from_json(
        data: &serde_json::Value,
        bytecode: &CompiledBytecode,
    ) -> Self {
        let mut values = vec![Value::Null; bytecode.variable_names.len()];

        for (name, idx) in &bytecode.variable_map {
            if let Some(value) = get_json_path(data, name) {
                values[*idx as usize] = Value::from_json(&value);
            }
        }

        Self { values }
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a value from JSON by dot-separated path
fn get_json_path(data: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    if path.is_empty() {
        return Some(data.clone());
    }

    let mut current = data;
    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(segment)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = segment.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }

    Some(current.clone())
}

/// Bytecode Virtual Machine
#[derive(Debug)]
pub struct VM {
    /// Execution stack
    stack: SmallVec<[Value; 32]>,
}

impl VM {
    /// Create a new VM
    pub fn new() -> Self {
        Self {
            stack: SmallVec::new(),
        }
    }

    /// Execute bytecode with the given context
    pub fn execute(
        &mut self,
        bytecode: &CompiledBytecode,
        context: &EvalContext,
    ) -> JsonLogicResult<Value> {
        self.stack.clear();

        let code = &bytecode.bytecode;
        let constants = &bytecode.constants;
        let mut pc: usize = 0;

        while pc < code.len() {
            let op = OpCode::from_byte(code[pc])
                .ok_or(JsonLogicError::InvalidBytecode(pc))?;

            match op {
                OpCode::Nop => {
                    pc += 1;
                }

                OpCode::LoadConst => {
                    let idx = read_u16(code, pc + 1) as usize;
                    if idx >= constants.len() {
                        return Err(JsonLogicError::InvalidBytecode(pc));
                    }
                    self.push(constants[idx].clone())?;
                    pc += 3;
                }

                OpCode::LoadVar => {
                    let idx = read_u16(code, pc + 1) as usize;
                    if idx >= context.values.len() {
                        return Err(JsonLogicError::VariableNotFound(format!("index {}", idx)));
                    }
                    self.push(context.values[idx].clone())?;
                    pc += 3;
                }

                OpCode::LoadVarWithDefault => {
                    let var_idx = read_u16(code, pc + 1) as usize;
                    let default_idx = read_u16(code, pc + 3) as usize;

                    let value = if var_idx < context.values.len() {
                        let v = &context.values[var_idx];
                        if matches!(v, Value::Null) {
                            // Variable is null, use default
                            if default_idx >= constants.len() {
                                return Err(JsonLogicError::InvalidBytecode(pc));
                            }
                            constants[default_idx].clone()
                        } else {
                            v.clone()
                        }
                    } else {
                        // Variable not found, use default
                        if default_idx >= constants.len() {
                            return Err(JsonLogicError::InvalidBytecode(pc));
                        }
                        constants[default_idx].clone()
                    };

                    self.push(value)?;
                    pc += 5;
                }

                OpCode::Dup => {
                    let top = self.peek()?.clone();
                    self.push(top)?;
                    pc += 1;
                }

                OpCode::Pop => {
                    self.pop()?;
                    pc += 1;
                }

                // Comparison operations
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(loose_equals(&a, &b)))?;
                    pc += 1;
                }

                OpCode::StrictEq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b))?;
                    pc += 1;
                }

                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(!loose_equals(&a, &b)))?;
                    pc += 1;
                }

                OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a != b))?;
                    pc += 1;
                }

                OpCode::Lt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(compare_values(&a, &b) == Some(std::cmp::Ordering::Less)))?;
                    pc += 1;
                }

                OpCode::Le => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = compare_values(&a, &b)
                        .map(|o| o != std::cmp::Ordering::Greater)
                        .unwrap_or(false);
                    self.push(Value::Bool(result))?;
                    pc += 1;
                }

                OpCode::Gt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(compare_values(&a, &b) == Some(std::cmp::Ordering::Greater)))?;
                    pc += 1;
                }

                OpCode::Ge => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = compare_values(&a, &b)
                        .map(|o| o != std::cmp::Ordering::Less)
                        .unwrap_or(false);
                    self.push(Value::Bool(result))?;
                    pc += 1;
                }

                // Logical operations
                OpCode::Not => {
                    let a = self.pop()?;
                    self.push(Value::Bool(!a.is_truthy()))?;
                    pc += 1;
                }

                OpCode::ToBool => {
                    let a = self.pop()?;
                    self.push(Value::Bool(a.is_truthy()))?;
                    pc += 1;
                }

                OpCode::And | OpCode::Or => {
                    // These should have been compiled away into jumps
                    // but if we see them, just AND/OR the top two values
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = if op == OpCode::And {
                        a.is_truthy() && b.is_truthy()
                    } else {
                        a.is_truthy() || b.is_truthy()
                    };
                    self.push(Value::Bool(result))?;
                    pc += 1;
                }

                // Arithmetic
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(add_values(&a, &b))?;
                    pc += 1;
                }

                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(sub_values(&a, &b))?;
                    pc += 1;
                }

                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(mul_values(&a, &b))?;
                    pc += 1;
                }

                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(div_values(&a, &b)?)?;
                    pc += 1;
                }

                OpCode::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(mod_values(&a, &b)?)?;
                    pc += 1;
                }

                OpCode::Neg => {
                    let a = self.pop()?;
                    self.push(negate_value(&a))?;
                    pc += 1;
                }

                OpCode::Min => {
                    let count = code[pc + 1] as usize;
                    let mut min_val = self.pop()?;
                    for _ in 1..count {
                        let val = self.pop()?;
                        if compare_values(&val, &min_val) == Some(std::cmp::Ordering::Less) {
                            min_val = val;
                        }
                    }
                    self.push(min_val)?;
                    pc += 2;
                }

                OpCode::Max => {
                    let count = code[pc + 1] as usize;
                    let mut max_val = self.pop()?;
                    for _ in 1..count {
                        let val = self.pop()?;
                        if compare_values(&val, &max_val) == Some(std::cmp::Ordering::Greater) {
                            max_val = val;
                        }
                    }
                    self.push(max_val)?;
                    pc += 2;
                }

                // String
                OpCode::Cat => {
                    let count = code[pc + 1] as usize;
                    let mut result = String::new();
                    let mut values: Vec<Value> = Vec::with_capacity(count);
                    for _ in 0..count {
                        values.push(self.pop()?);
                    }
                    values.reverse();
                    for val in values {
                        result.push_str(&value_to_string(&val));
                    }
                    self.push(Value::String(result))?;
                    pc += 2;
                }

                OpCode::Substr => {
                    let has_length = code[pc + 1] != 0;
                    let (s, start, length) = if has_length {
                        let l = self.pop()?;
                        let st = self.pop()?;
                        let s = self.pop()?;
                        (s, st, Some(l))
                    } else {
                        let st = self.pop()?;
                        let s = self.pop()?;
                        (s, st, None)
                    };

                    let s = value_to_string(&s);
                    let start_idx = start.as_int().unwrap_or(0) as usize;
                    let chars: Vec<char> = s.chars().collect();

                    let result = if let Some(len) = length {
                        let len = len.as_int().unwrap_or(chars.len() as i64) as usize;
                        chars
                            .into_iter()
                            .skip(start_idx)
                            .take(len)
                            .collect::<String>()
                    } else {
                        chars.into_iter().skip(start_idx).collect::<String>()
                    };

                    self.push(Value::String(result))?;
                    pc += 2;
                }

                // Control flow
                OpCode::Jump => {
                    let offset = read_u16(code, pc + 1) as usize;
                    pc = pc + 3 + offset;
                }

                OpCode::JumpIfFalse => {
                    let cond = self.pop()?;
                    if !cond.is_truthy() {
                        let offset = read_u16(code, pc + 1) as usize;
                        pc = pc + 3 + offset;
                    } else {
                        pc += 3;
                    }
                }

                OpCode::JumpIfTrue => {
                    let cond = self.pop()?;
                    if cond.is_truthy() {
                        let offset = read_u16(code, pc + 1) as usize;
                        pc = pc + 3 + offset;
                    } else {
                        pc += 3;
                    }
                }

                // Array
                OpCode::In => {
                    let arr = self.pop()?;
                    let val = self.pop()?;

                    let result = match &arr {
                        Value::Array(items) => items.iter().any(|item| loose_equals(&val, item)),
                        Value::String(s) => {
                            if let Value::String(needle) = &val {
                                s.contains(needle.as_str())
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    self.push(Value::Bool(result))?;
                    pc += 1;
                }

                OpCode::Return => {
                    return self.pop();
                }
            }
        }

        // If we reach here without Return, return top of stack or null
        if self.stack.is_empty() {
            Ok(Value::Null)
        } else {
            self.pop()
        }
    }

    /// Push value onto stack
    #[inline(always)]
    fn push(&mut self, value: Value) -> JsonLogicResult<()> {
        if self.stack.len() >= MAX_STACK_DEPTH {
            return Err(JsonLogicError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    #[inline(always)]
    fn pop(&mut self) -> JsonLogicResult<Value> {
        self.stack.pop().ok_or(JsonLogicError::StackUnderflow)
    }

    /// Peek at top of stack
    #[inline(always)]
    fn peek(&self) -> JsonLogicResult<&Value> {
        self.stack.last().ok_or(JsonLogicError::StackUnderflow)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

/// Read u16 from bytecode (little-endian)
#[inline(always)]
fn read_u16(code: &[u8], offset: usize) -> u16 {
    let lo = code.get(offset).copied().unwrap_or(0) as u16;
    let hi = code.get(offset + 1).copied().unwrap_or(0) as u16;
    lo | (hi << 8)
}

/// Loose equality comparison (JavaScript-style coercion)
#[inline]
fn loose_equals(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Decimal(a), Value::Decimal(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,

        // Cross-type numeric comparisons
        (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
        (Value::Int(a), Value::Decimal(b)) => Decimal::from(*a) == *b,
        (Value::Decimal(a), Value::Int(b)) => *a == Decimal::from(*b),

        // String to number coercion
        (Value::String(s), Value::Int(i)) => s.parse::<i64>().ok() == Some(*i),
        (Value::Int(i), Value::String(s)) => Some(*i) == s.parse::<i64>().ok(),

        _ => false,
    }
}

/// Compare values (returns ordering)
#[inline]
fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
        (Value::Decimal(a), Value::Decimal(b)) => a.partial_cmp(b),
        (Value::String(a), Value::String(b)) => a.partial_cmp(b),

        // Cross-type numeric
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
        (Value::Int(a), Value::Decimal(b)) => Decimal::from(*a).partial_cmp(b),
        (Value::Decimal(a), Value::Int(b)) => a.partial_cmp(&Decimal::from(*b)),

        _ => None,
    }
}

/// Add two values
fn add_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
        (Value::Decimal(a), Value::Decimal(b)) => Value::Decimal(*a + *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
        (Value::Int(a), Value::Decimal(b)) => Value::Decimal(Decimal::from(*a) + *b),
        (Value::Decimal(a), Value::Int(b)) => Value::Decimal(*a + Decimal::from(*b)),
        (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
        _ => Value::Null,
    }
}

/// Subtract two values
fn sub_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (Value::Decimal(a), Value::Decimal(b)) => Value::Decimal(*a - *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
        _ => Value::Null,
    }
}

/// Multiply two values
fn mul_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (Value::Decimal(a), Value::Decimal(b)) => Value::Decimal(*a * *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a * *b as f64),
        _ => Value::Null,
    }
}

/// Divide two values
fn div_values(a: &Value, b: &Value) -> JsonLogicResult<Value> {
    let divisor = b.as_float().unwrap_or(0.0);
    if divisor == 0.0 {
        return Err(JsonLogicError::DivisionByZero);
    }

    Ok(match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Float(*a as f64 / *b as f64),
        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a / *b as f64),
        _ => Value::Null,
    })
}

/// Modulo two values
fn mod_values(a: &Value, b: &Value) -> JsonLogicResult<Value> {
    let divisor = b.as_int().unwrap_or(0);
    if divisor == 0 {
        return Err(JsonLogicError::DivisionByZero);
    }

    Ok(match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
        _ => Value::Null,
    })
}

/// Negate a value
fn negate_value(a: &Value) -> Value {
    match a {
        Value::Int(a) => Value::Int(-a),
        Value::Float(a) => Value::Float(-a),
        Value::Decimal(a) => Value::Decimal(-*a),
        _ => Value::Null,
    }
}

/// Convert value to string
fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Decimal(d) => d.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) => "[array]".to_string(),
        Value::Object(_) => "[object]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, Compiler};
    use serde_json::json;

    fn eval(json_logic: serde_json::Value, data: serde_json::Value) -> Value {
        let expr = parse(&json_logic).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();
        let context = EvalContext::from_json(&data, &bytecode);
        let mut vm = VM::new();
        vm.execute(&bytecode, &context).unwrap()
    }

    #[test]
    fn test_vm_literal() {
        let result = eval(json!(42), json!({}));
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_var() {
        let result = eval(json!({"var": "age"}), json!({"age": 25}));
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_vm_comparison() {
        let result = eval(json!({">": [{"var": "age"}, 18]}), json!({"age": 25}));
        assert_eq!(result, Value::Bool(true));

        let result = eval(json!({">": [{"var": "age"}, 18]}), json!({"age": 15}));
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_arithmetic() {
        let result = eval(json!({"*": [{"var": "base"}, 1.2]}), json!({"base": 100}));
        assert_eq!(result, Value::Float(120.0));

        let result = eval(json!({"+": [1, 2, 3]}), json!({}));
        assert_eq!(result, Value::Int(6));
    }

    #[test]
    fn test_vm_if() {
        // age > 60 ? base * 1.2 : base
        let result = eval(
            json!({
                "if": [
                    {">": [{"var": "age"}, 60]},
                    {"*": [{"var": "base"}, 1.2]},
                    {"var": "base"}
                ]
            }),
            json!({"age": 65, "base": 100}),
        );
        assert_eq!(result, Value::Float(120.0));

        let result = eval(
            json!({
                "if": [
                    {">": [{"var": "age"}, 60]},
                    {"*": [{"var": "base"}, 1.2]},
                    {"var": "base"}
                ]
            }),
            json!({"age": 50, "base": 100}),
        );
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_vm_and_or() {
        let result = eval(
            json!({"and": [true, {"var": "active"}]}),
            json!({"active": true}),
        );
        assert_eq!(result, Value::Bool(true));

        let result = eval(
            json!({"or": [false, {"var": "fallback"}]}),
            json!({"fallback": "default"}),
        );
        assert_eq!(result, Value::String("default".to_string()));
    }

    #[test]
    fn test_vm_nested_path() {
        let result = eval(
            json!({"var": "user.profile.age"}),
            json!({"user": {"profile": {"age": 30}}}),
        );
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_vm_var_with_default() {
        // Variable exists - should use value
        let result = eval(
            json!({"var": ["name", "unknown"]}),
            json!({"name": "Alice"}),
        );
        assert_eq!(result, Value::String("Alice".to_string()));

        // Variable missing - should use default
        let result = eval(
            json!({"var": ["missing", "default_value"]}),
            json!({"other": "data"}),
        );
        assert_eq!(result, Value::String("default_value".to_string()));

        // Variable is null - should use default
        let result = eval(
            json!({"var": ["nullable", 42]}),
            json!({"nullable": null}),
        );
        assert_eq!(result, Value::Int(42));

        // Default in arithmetic
        let result = eval(
            json!({"+": [{"var": ["x", 10]}, {"var": ["y", 20]}]}),
            json!({"x": 5}),  // y is missing
        );
        assert_eq!(result, Value::Int(25));  // 5 + 20
    }
}
