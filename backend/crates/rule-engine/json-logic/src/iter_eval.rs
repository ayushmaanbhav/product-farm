//! Loop-based (iterative) evaluator for JSON Logic
//!
//! This module provides a non-recursive evaluator that processes expressions
//! using explicit work tracking instead of call stack recursion. This prevents
//! stack overflow on deeply nested expressions.
//!
//! # Design
//!
//! Instead of recursive `eval_expr()` calls, we use:
//! - `work`: Vec of pending operations to process
//! - `values`: Vec of computed intermediate values
//!
//! The evaluator runs a loop that processes work items until complete.
//!
//! # Safety
//!
//! This implementation avoids unsafe code by using proper lifetime bounds.
//! The work queue only lives for the duration of a single `evaluate()` call,
//! and all references in it are valid for that duration.

use product_farm_core::Value;
use std::collections::HashMap;

use crate::ast::Expr;
use crate::config::Config;
use crate::error::{JsonLogicError, JsonLogicResult};

/// A pending operation in the work queue
#[derive(Debug)]
enum WorkItem<'a> {
    /// Evaluate an expression and push result to value stack
    Eval(&'a Expr),

    /// Combine N values from stack using an operation
    Combine(CombineOp, usize),

    /// Special handling for short-circuit AND
    /// (remaining_exprs, total_count)
    AndNext(&'a [Expr], usize),

    /// Special handling for short-circuit OR
    OrNext(&'a [Expr], usize),

    /// If/else chain: check condition result, then branch or continue
    /// (then_expr, remaining_exprs after then)
    IfConditionCheck(&'a Expr, &'a [Expr]),

    /// Check if condition for ternary, then evaluate appropriate branch
    TernaryCheck(&'a Expr, &'a Expr),

    /// Chain comparison: check if prev < curr, continue or short-circuit
    /// (remaining_exprs, comparison_fn_id)
    ChainCompareNext(&'a [Expr], ChainCmpOp),

    /// Array map: process next item or finish
    /// (mapper_expr, remaining_items, collected_results)
    MapNext(&'a Expr, Vec<Value>, Vec<Value>),

    /// Array filter: process next item or finish
    /// (predicate_expr, remaining_items, collected_results)
    FilterNext(&'a Expr, Vec<Value>, Vec<Value>),

    /// Filter check: if truthy, keep the item
    FilterCheck(&'a Expr, Value, Vec<Value>, Vec<Value>),

    /// Array reduce: process next item or finish
    /// (reducer_expr, remaining_items, accumulator)
    ReduceNext(&'a Expr, Vec<Value>),

    /// All: check result and continue or short-circuit
    AllNext(&'a Expr, Vec<Value>),

    /// Some: check result and continue or short-circuit
    SomeNext(&'a Expr, Vec<Value>),

    /// None: check result and continue or short-circuit
    NoneNext(&'a Expr, Vec<Value>),

    /// Substr: compute the substring after evaluating args
    SubstrCompute(bool), // has_length_arg

    /// Missing: check each key
    MissingNext(&'a [Expr], Vec<Value>),

    /// MissingSome: check keys and track found count
    MissingSomeNext(&'a [Expr], usize, usize, Vec<Value>),

    /// Division: check for zero and compute
    DivCheck,

    /// Modulo: check for zero and compute
    ModCheck,

    /// Negate the top value (for unary minus)
    Negate,

    /// Convert to boolean
    ToBool,

    /// Logical NOT
    Not,

    /// Log and pass through
    Log,
}

/// Operations that combine multiple values
#[derive(Debug, Clone, Copy)]
enum CombineOp {
    Eq,
    StrictEq,
    Ne,
    StrictNe,
    Add,
    Sub,
    Mul,
    Min,
    Max,
    Cat,
    Merge,
    In,
}

/// Chain comparison operations
#[derive(Debug, Clone, Copy)]
enum ChainCmpOp {
    Lt,
    Le,
    Gt,
    Ge,
}

impl ChainCmpOp {
    fn compare(&self, a: f64, b: f64) -> bool {
        match self {
            ChainCmpOp::Lt => a < b,
            ChainCmpOp::Le => a <= b,
            ChainCmpOp::Gt => a > b,
            ChainCmpOp::Ge => a >= b,
        }
    }
}

/// Loop-based JSON Logic evaluator
///
/// This evaluator uses an explicit work queue instead of recursion,
/// preventing stack overflow on deeply nested expressions.
#[derive(Default)]
pub struct IterativeEvaluator {
    // No fields needed - work and values are local to evaluate()
    _private: (),
}

impl IterativeEvaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Evaluate an expression against data
    ///
    /// This is safe because all references in the work queue are bounded
    /// by the lifetime of `expr` and `data`, which outlive this function call.
    pub fn evaluate(&mut self, expr: &Expr, data: &Value) -> JsonLogicResult<Value> {
        // Work queue and values are local to this function call
        // All references in work are valid for lifetime 'a where expr: &'a Expr
        let mut work: Vec<WorkItem<'_>> = Vec::with_capacity(64);
        let mut values: Vec<Value> = Vec::with_capacity(32);

        work.push(WorkItem::Eval(expr));

        let mut iterations = 0;
        while let Some(item) = work.pop() {
            iterations += 1;
            if iterations > Config::global().eval_max_steps {
                return Err(JsonLogicError::StackOverflow);
            }
            if work.len() > Config::global().eval_work_queue_limit {
                return Err(JsonLogicError::StackOverflow);
            }

            process_item(item, data, &mut work, &mut values)?;
        }

        values.pop().ok_or(JsonLogicError::RuntimeError(
            "No result value after evaluation".into(),
        ))
    }
}

/// Process a single work item
fn process_item<'a>(
    item: WorkItem<'a>,
    data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    match item {
        WorkItem::Eval(expr) => eval_expr(expr, data, work, values)?,
        WorkItem::Combine(op, count) => combine(op, count, values)?,
        WorkItem::AndNext(exprs, total) => and_next(exprs, total, work, values)?,
        WorkItem::OrNext(exprs, total) => or_next(exprs, total, work, values)?,
        WorkItem::IfConditionCheck(then_expr, remaining) => {
            if_condition_check(then_expr, remaining, work, values)?
        }
        WorkItem::TernaryCheck(then_expr, else_expr) => {
            ternary_check(then_expr, else_expr, work, values)?
        }
        WorkItem::ChainCompareNext(exprs, op) => chain_compare_next(exprs, op, work, values)?,
        WorkItem::MapNext(mapper, remaining, results) => {
            map_next(mapper, remaining, results, data, work, values)?
        }
        WorkItem::FilterNext(pred, remaining, results) => {
            filter_next(pred, remaining, results, data, work, values)?
        }
        WorkItem::FilterCheck(pred, item_val, remaining, results) => {
            filter_check(pred, item_val, remaining, results, work, values)?
        }
        WorkItem::ReduceNext(reducer, remaining) => {
            reduce_next(reducer, remaining, data, work, values)?
        }
        WorkItem::AllNext(pred, remaining) => all_next(pred, remaining, data, work, values)?,
        WorkItem::SomeNext(pred, remaining) => some_next(pred, remaining, data, work, values)?,
        WorkItem::NoneNext(pred, remaining) => none_next(pred, remaining, data, work, values)?,
        WorkItem::SubstrCompute(has_len) => substr_compute(has_len, values)?,
        WorkItem::MissingNext(keys, missing) => missing_next(keys, missing, data, work, values)?,
        WorkItem::MissingSomeNext(keys, min, found, missing) => {
            missing_some_next(keys, min, found, missing, data, work, values)?
        }
        WorkItem::DivCheck => div_check(values)?,
        WorkItem::ModCheck => mod_check(values)?,
        WorkItem::Negate => negate(values)?,
        WorkItem::ToBool => to_bool(values)?,
        WorkItem::Not => not(values)?,
        WorkItem::Log => log_value(values)?,
    }
    Ok(())
}

/// Start evaluating an expression
fn eval_expr<'a>(
    expr: &'a Expr,
    data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    match expr {
        Expr::Literal(v) => {
            values.push(v.clone());
        }

        Expr::Var(var) => {
            let value = get_variable(&var.path, data)
                .or_else(|| var.default.clone())
                .ok_or_else(|| JsonLogicError::VariableNotFound(var.path.clone()))?;
            values.push(value);
        }

        Expr::Eq(a, b) => {
            work.push(WorkItem::Combine(CombineOp::Eq, 2));
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::StrictEq(a, b) => {
            work.push(WorkItem::Combine(CombineOp::StrictEq, 2));
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::Ne(a, b) => {
            work.push(WorkItem::Combine(CombineOp::Ne, 2));
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::StrictNe(a, b) => {
            work.push(WorkItem::Combine(CombineOp::StrictNe, 2));
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::Lt(exprs) => start_chain_compare(exprs, ChainCmpOp::Lt, work, values)?,
        Expr::Le(exprs) => start_chain_compare(exprs, ChainCmpOp::Le, work, values)?,
        Expr::Gt(exprs) => start_chain_compare(exprs, ChainCmpOp::Gt, work, values)?,
        Expr::Ge(exprs) => start_chain_compare(exprs, ChainCmpOp::Ge, work, values)?,

        Expr::Not(a) => {
            work.push(WorkItem::Not);
            work.push(WorkItem::Eval(a));
        }

        Expr::ToBool(a) => {
            work.push(WorkItem::ToBool);
            work.push(WorkItem::Eval(a));
        }

        Expr::And(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Bool(true));
            } else {
                work.push(WorkItem::AndNext(&exprs[1..], exprs.len()));
                work.push(WorkItem::Eval(&exprs[0]));
            }
        }

        Expr::Or(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Bool(false));
            } else {
                work.push(WorkItem::OrNext(&exprs[1..], exprs.len()));
                work.push(WorkItem::Eval(&exprs[0]));
            }
        }

        Expr::If(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Null);
            } else if exprs.len() == 1 {
                // Single element is the else clause
                work.push(WorkItem::Eval(&exprs[0]));
            } else {
                // exprs[0] = condition, exprs[1] = then, exprs[2..] = remaining
                let remaining = if exprs.len() > 2 { &exprs[2..] } else { &[] as &[Expr] };
                work.push(WorkItem::IfConditionCheck(&exprs[1], remaining));
                work.push(WorkItem::Eval(&exprs[0]));
            }
        }

        Expr::Ternary(cond, then_expr, else_expr) => {
            work.push(WorkItem::TernaryCheck(then_expr, else_expr));
            work.push(WorkItem::Eval(cond));
        }

        Expr::Add(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Float(0.0));
            } else {
                work.push(WorkItem::Combine(CombineOp::Add, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Sub(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Float(0.0));
            } else if exprs.len() == 1 {
                // Unary negation
                work.push(WorkItem::Negate);
                work.push(WorkItem::Eval(&exprs[0]));
            } else {
                work.push(WorkItem::Combine(CombineOp::Sub, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Mul(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Float(1.0));
            } else {
                work.push(WorkItem::Combine(CombineOp::Mul, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Div(a, b) => {
            work.push(WorkItem::DivCheck);
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::Mod(a, b) => {
            work.push(WorkItem::ModCheck);
            work.push(WorkItem::Eval(b));
            work.push(WorkItem::Eval(a));
        }

        Expr::Min(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Float(f64::INFINITY));
            } else {
                work.push(WorkItem::Combine(CombineOp::Min, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Max(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Float(f64::NEG_INFINITY));
            } else {
                work.push(WorkItem::Combine(CombineOp::Max, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Cat(exprs) => {
            if exprs.is_empty() {
                values.push(Value::String(String::new()));
            } else {
                work.push(WorkItem::Combine(CombineOp::Cat, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::Substr(s, start, len) => {
            let has_len = len.is_some();
            work.push(WorkItem::SubstrCompute(has_len));
            if let Some(len_expr) = len {
                work.push(WorkItem::Eval(len_expr.as_ref()));
            }
            work.push(WorkItem::Eval(start));
            work.push(WorkItem::Eval(s));
        }

        Expr::Map(arr, mapper) => {
            // First evaluate the array synchronously
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => vec![],
            };
            if items.is_empty() {
                values.push(Value::Array(vec![]));
            } else {
                work.push(WorkItem::MapNext(mapper.as_ref(), items, vec![]));
            }
        }

        Expr::Filter(arr, predicate) => {
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => vec![],
            };
            if items.is_empty() {
                values.push(Value::Array(vec![]));
            } else {
                work.push(WorkItem::FilterNext(predicate.as_ref(), items, vec![]));
            }
        }

        Expr::Reduce(arr, reducer, initial) => {
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => {
                    // Not an array, return initial value
                    work.push(WorkItem::Eval(initial));
                    return Ok(());
                }
            };

            // Evaluate initial value synchronously
            let init_val = eval_to_value(initial, data, work, values)?;
            values.push(init_val);

            if !items.is_empty() {
                work.push(WorkItem::ReduceNext(reducer.as_ref(), items));
            }
        }

        Expr::All(arr, predicate) => {
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => {
                    values.push(Value::Bool(false));
                    return Ok(());
                }
            };
            if items.is_empty() {
                values.push(Value::Bool(false));
            } else {
                work.push(WorkItem::AllNext(predicate.as_ref(), items));
            }
        }

        Expr::Some(arr, predicate) => {
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => {
                    values.push(Value::Bool(false));
                    return Ok(());
                }
            };
            if items.is_empty() {
                values.push(Value::Bool(false));
            } else {
                work.push(WorkItem::SomeNext(predicate.as_ref(), items));
            }
        }

        Expr::None(arr, predicate) => {
            let arr_val = eval_to_value(arr, data, work, values)?;
            let items = match arr_val {
                Value::Array(items) => items,
                _ => {
                    values.push(Value::Bool(true));
                    return Ok(());
                }
            };
            if items.is_empty() {
                values.push(Value::Bool(true));
            } else {
                work.push(WorkItem::NoneNext(predicate.as_ref(), items));
            }
        }

        Expr::Merge(exprs) => {
            if exprs.is_empty() {
                values.push(Value::Array(vec![]));
            } else {
                work.push(WorkItem::Combine(CombineOp::Merge, exprs.len()));
                for expr in exprs.iter().rev() {
                    work.push(WorkItem::Eval(expr));
                }
            }
        }

        Expr::In(needle, haystack) => {
            work.push(WorkItem::Combine(CombineOp::In, 2));
            work.push(WorkItem::Eval(haystack));
            work.push(WorkItem::Eval(needle));
        }

        Expr::Missing(keys) => {
            if keys.is_empty() {
                values.push(Value::Array(vec![]));
            } else {
                work.push(WorkItem::MissingNext(&keys[..], vec![]));
            }
        }

        Expr::MissingSome(min_required, keys) => {
            let min_val = eval_to_value(min_required, data, work, values)?;
            let min = min_val.to_number() as usize;

            if keys.is_empty() {
                values.push(Value::Array(vec![]));
            } else {
                work.push(WorkItem::MissingSomeNext(&keys[..], min, 0, vec![]));
            }
        }

        Expr::Log(inner) => {
            work.push(WorkItem::Log);
            work.push(WorkItem::Eval(inner));
        }
    }
    Ok(())
}

/// Helper: evaluate expression synchronously (for array source evaluation)
/// This uses a separate local work queue to avoid lifetime issues
fn eval_to_value<'a>(
    expr: &'a Expr,
    data: &Value,
    _outer_work: &mut Vec<WorkItem<'a>>,
    _outer_values: &mut Vec<Value>,
) -> JsonLogicResult<Value> {
    // Use a fresh local work queue for sub-evaluation
    let mut work: Vec<WorkItem<'a>> = Vec::with_capacity(16);
    let mut values: Vec<Value> = Vec::with_capacity(8);

    work.push(WorkItem::Eval(expr));

    let mut iterations = 0;
    while let Some(item) = work.pop() {
        iterations += 1;
        if iterations > Config::global().eval_max_steps {
            return Err(JsonLogicError::StackOverflow);
        }
        process_item(item, data, &mut work, &mut values)?;
    }

    values.pop().ok_or(JsonLogicError::RuntimeError(
        "No result from sub-evaluation".into(),
    ))
}

/// Start a chain comparison
fn start_chain_compare<'a>(
    exprs: &'a [Expr],
    op: ChainCmpOp,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if exprs.len() < 2 {
        values.push(Value::Bool(false));
        return Ok(());
    }

    // Evaluate first two, then continue chain
    work.push(WorkItem::ChainCompareNext(&exprs[2..], op));
    work.push(WorkItem::Eval(&exprs[1]));
    work.push(WorkItem::Eval(&exprs[0]));
    Ok(())
}

/// Continue chain comparison
fn chain_compare_next<'a>(
    remaining: &'a [Expr],
    op: ChainCmpOp,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let curr = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    let prev = values.pop().ok_or(JsonLogicError::StackOverflow)?;

    let prev_num = prev.to_number();
    let curr_num = curr.to_number();

    if !op.compare(prev_num, curr_num) {
        // Short-circuit: comparison failed
        values.push(Value::Bool(false));
        return Ok(());
    }

    if remaining.is_empty() {
        // All comparisons passed
        values.push(Value::Bool(true));
    } else {
        // Continue with next comparison
        // Push current value back as "prev" for next comparison
        values.push(curr);
        work.push(WorkItem::ChainCompareNext(&remaining[1..], op));
        work.push(WorkItem::Eval(&remaining[0]));
    }
    Ok(())
}

/// Continue AND evaluation with short-circuit
fn and_next<'a>(
    remaining: &'a [Expr],
    _total: usize,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let val = values.last().ok_or(JsonLogicError::StackOverflow)?;

    if !val.is_truthy() {
        // Short-circuit: keep the falsy value as result
        return Ok(());
    }

    if remaining.is_empty() {
        // All truthy, last value is result (already on stack)
        return Ok(());
    }

    // Pop the truthy intermediate, evaluate next
    values.pop();
    work.push(WorkItem::AndNext(&remaining[1..], _total));
    work.push(WorkItem::Eval(&remaining[0]));
    Ok(())
}

/// Continue OR evaluation with short-circuit
fn or_next<'a>(
    remaining: &'a [Expr],
    _total: usize,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let val = values.last().ok_or(JsonLogicError::StackOverflow)?;

    if val.is_truthy() {
        // Short-circuit: keep the truthy value as result
        return Ok(());
    }

    if remaining.is_empty() {
        // All falsy, last value is result (already on stack)
        return Ok(());
    }

    // Pop the falsy intermediate, evaluate next
    values.pop();
    work.push(WorkItem::OrNext(&remaining[1..], _total));
    work.push(WorkItem::Eval(&remaining[0]));
    Ok(())
}

/// Check if condition result and branch accordingly
fn if_condition_check<'a>(
    then_expr: &'a Expr,
    remaining: &'a [Expr],
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let cond = values.pop().ok_or(JsonLogicError::StackOverflow)?;

    if cond.is_truthy() {
        // Condition is true, evaluate then branch
        work.push(WorkItem::Eval(then_expr));
    } else if remaining.is_empty() {
        // No more conditions, no else clause - return null
        values.push(Value::Null);
    } else if remaining.len() == 1 {
        // Single remaining element is the else clause
        work.push(WorkItem::Eval(&remaining[0]));
    } else {
        // More condition/then pairs: remaining[0] = next condition, remaining[1] = next then
        let next_remaining = if remaining.len() > 2 { &remaining[2..] } else { &[] as &[Expr] };
        work.push(WorkItem::IfConditionCheck(&remaining[1], next_remaining));
        work.push(WorkItem::Eval(&remaining[0]));
    }
    Ok(())
}

/// Check ternary condition and evaluate appropriate branch
fn ternary_check<'a>(
    then_expr: &'a Expr,
    else_expr: &'a Expr,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let cond = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    if cond.is_truthy() {
        work.push(WorkItem::Eval(then_expr));
    } else {
        work.push(WorkItem::Eval(else_expr));
    }
    Ok(())
}

/// Map: process next item
fn map_next<'a>(
    mapper: &'a Expr,
    mut remaining: Vec<Value>,
    mut results: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        values.push(Value::Array(results));
        return Ok(());
    }

    let item = remaining.remove(0);
    // Evaluate mapper with item as data - use a nested evaluation
    let result = eval_to_value(mapper, &item, work, values)?;
    results.push(result);

    // Continue with remaining
    work.push(WorkItem::MapNext(mapper, remaining, results));
    Ok(())
}

/// Filter: process next item
fn filter_next<'a>(
    predicate: &'a Expr,
    mut remaining: Vec<Value>,
    results: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        values.push(Value::Array(results));
        return Ok(());
    }

    let item = remaining.remove(0);
    // Evaluate predicate with item as data
    let pred_result = eval_to_value(predicate, &item, work, values)?;
    values.push(pred_result);
    work.push(WorkItem::FilterCheck(predicate, item, remaining, results));
    Ok(())
}

/// Filter: check predicate result
fn filter_check<'a>(
    predicate: &'a Expr,
    item: Value,
    remaining: Vec<Value>,
    mut results: Vec<Value>,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    let keep = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    if keep.is_truthy() {
        results.push(item);
    }
    work.push(WorkItem::FilterNext(predicate, remaining, results));
    Ok(())
}

/// Reduce: process next item
fn reduce_next<'a>(
    reducer: &'a Expr,
    mut remaining: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        // Accumulator is already on stack
        return Ok(());
    }

    let accumulator = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    let current = remaining.remove(0);

    // Build reduce context
    let reduce_data = Value::Object(HashMap::from([
        ("current".to_string(), current),
        ("accumulator".to_string(), accumulator),
    ]));

    // Evaluate reducer with context
    let new_acc = eval_to_value(reducer, &reduce_data, work, values)?;
    values.push(new_acc);

    if !remaining.is_empty() {
        work.push(WorkItem::ReduceNext(reducer, remaining));
    }
    Ok(())
}

/// All: check result and continue
fn all_next<'a>(
    predicate: &'a Expr,
    mut remaining: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        values.push(Value::Bool(true));
        return Ok(());
    }

    let item = remaining.remove(0);
    let result = eval_to_value(predicate, &item, work, values)?;

    if !result.is_truthy() {
        // Short-circuit
        values.push(Value::Bool(false));
        return Ok(());
    }

    work.push(WorkItem::AllNext(predicate, remaining));
    Ok(())
}

/// Some: check result and continue
fn some_next<'a>(
    predicate: &'a Expr,
    mut remaining: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        values.push(Value::Bool(false));
        return Ok(());
    }

    let item = remaining.remove(0);
    let result = eval_to_value(predicate, &item, work, values)?;

    if result.is_truthy() {
        // Short-circuit
        values.push(Value::Bool(true));
        return Ok(());
    }

    work.push(WorkItem::SomeNext(predicate, remaining));
    Ok(())
}

/// None: check result and continue
fn none_next<'a>(
    predicate: &'a Expr,
    mut remaining: Vec<Value>,
    _data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if remaining.is_empty() {
        values.push(Value::Bool(true));
        return Ok(());
    }

    let item = remaining.remove(0);
    let result = eval_to_value(predicate, &item, work, values)?;

    if result.is_truthy() {
        // Short-circuit
        values.push(Value::Bool(false));
        return Ok(());
    }

    work.push(WorkItem::NoneNext(predicate, remaining));
    Ok(())
}

/// Compute substring after args are evaluated
fn substr_compute(has_len: bool, values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let (s_val, start_val, len_opt) = if has_len {
        let len_val = values.pop().ok_or(JsonLogicError::StackOverflow)?;
        let start = values.pop().ok_or(JsonLogicError::StackOverflow)?;
        let s = values.pop().ok_or(JsonLogicError::StackOverflow)?;
        (s, start, Some(len_val))
    } else {
        let start = values.pop().ok_or(JsonLogicError::StackOverflow)?;
        let s = values.pop().ok_or(JsonLogicError::StackOverflow)?;
        (s, start, None)
    };

    let s_str = s_val.to_display_string();
    let start_num = start_val.to_number() as i64;

    let chars: Vec<char> = s_str.chars().collect();
    let len_chars = chars.len() as i64;

    let actual_start = if start_num < 0 {
        (len_chars + start_num).max(0) as usize
    } else {
        start_num.min(len_chars) as usize
    };

    let result = if let Some(length_val) = len_opt {
        let length = length_val.to_number() as i64;
        if length < 0 {
            let end = (len_chars + length).max(actual_start as i64) as usize;
            chars[actual_start..end].iter().collect()
        } else {
            let end = (actual_start + length as usize).min(chars.len());
            chars[actual_start..end].iter().collect()
        }
    } else {
        chars[actual_start..].iter().collect()
    };

    values.push(Value::String(result));
    Ok(())
}

/// Missing: check next key
fn missing_next<'a>(
    keys: &'a [Expr],
    mut missing: Vec<Value>,
    data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if keys.is_empty() {
        values.push(Value::Array(missing));
        return Ok(());
    }

    let key_val = eval_to_value(&keys[0], data, work, values)?;
    let key_str = key_val.to_display_string();

    if get_variable_ref(&key_str, data).is_none() {
        missing.push(Value::String(key_str));
    }

    work.push(WorkItem::MissingNext(&keys[1..], missing));
    Ok(())
}

/// MissingSome: check keys and track found
fn missing_some_next<'a>(
    keys: &'a [Expr],
    min: usize,
    mut found: usize,
    mut missing: Vec<Value>,
    data: &Value,
    work: &mut Vec<WorkItem<'a>>,
    values: &mut Vec<Value>,
) -> JsonLogicResult<()> {
    if keys.is_empty() {
        if found >= min {
            values.push(Value::Array(vec![]));
        } else {
            values.push(Value::Array(missing));
        }
        return Ok(());
    }

    let key_val = eval_to_value(&keys[0], data, work, values)?;
    let key_str = key_val.to_display_string();

    if get_variable_ref(&key_str, data).is_some() {
        found += 1;
    } else {
        missing.push(Value::String(key_str));
    }

    work.push(WorkItem::MissingSomeNext(&keys[1..], min, found, missing));
    Ok(())
}

/// Division with zero check
fn div_check(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let b = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    let a = values.pop().ok_or(JsonLogicError::StackOverflow)?;

    let b_num = b.to_number();
    if b_num == 0.0 {
        return Err(JsonLogicError::DivisionByZero);
    }

    values.push(Value::Float(a.to_number() / b_num));
    Ok(())
}

/// Modulo with zero check
fn mod_check(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let b = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    let a = values.pop().ok_or(JsonLogicError::StackOverflow)?;

    let b_num = b.to_number();
    if b_num == 0.0 {
        return Err(JsonLogicError::DivisionByZero);
    }

    values.push(Value::Float(a.to_number() % b_num));
    Ok(())
}

/// Negate top value
fn negate(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let val = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    values.push(Value::Float(-val.to_number()));
    Ok(())
}

/// Convert to boolean
fn to_bool(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let val = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    values.push(Value::Bool(val.is_truthy()));
    Ok(())
}

/// Logical NOT
fn not(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let val = values.pop().ok_or(JsonLogicError::StackOverflow)?;
    values.push(Value::Bool(!val.is_truthy()));
    Ok(())
}

/// Log and pass through
#[allow(unused_variables)]
fn log_value(values: &mut Vec<Value>) -> JsonLogicResult<()> {
    let val = values.last().ok_or(JsonLogicError::StackOverflow)?;
    #[cfg(debug_assertions)]
    eprintln!("[JSON Logic log] {:?}", val);
    Ok(())
}

/// Combine multiple values using an operation
fn combine(op: CombineOp, count: usize, values: &mut Vec<Value>) -> JsonLogicResult<()> {
    if count == 0 {
        return Err(JsonLogicError::RuntimeError("Combine with zero count".into()));
    }

    // Pop values (they were pushed in reverse order, so they come out in correct order)
    let mut vals = Vec::with_capacity(count);
    for _ in 0..count {
        vals.push(values.pop().ok_or(JsonLogicError::StackOverflow)?);
    }
    vals.reverse();

    let result = match op {
        CombineOp::Eq => {
            Value::Bool(vals[0].loose_equals(&vals[1]))
        }
        CombineOp::StrictEq => {
            Value::Bool(vals[0] == vals[1])
        }
        CombineOp::Ne => {
            Value::Bool(!vals[0].loose_equals(&vals[1]))
        }
        CombineOp::StrictNe => {
            Value::Bool(vals[0] != vals[1])
        }
        CombineOp::Add => {
            let sum: f64 = vals.iter().map(|v| v.to_number()).sum();
            Value::Float(sum)
        }
        CombineOp::Sub => {
            // N-ary subtraction: a - b - c - d = ((a - b) - c) - d
            let mut result = vals[0].to_number();
            for v in &vals[1..] {
                result -= v.to_number();
            }
            Value::Float(result)
        }
        CombineOp::Mul => {
            let product: f64 = vals.iter().map(|v| v.to_number()).product();
            Value::Float(product)
        }
        CombineOp::Min => {
            let min = vals.iter().map(|v| v.to_number()).fold(f64::INFINITY, f64::min);
            Value::Float(min)
        }
        CombineOp::Max => {
            let max = vals.iter().map(|v| v.to_number()).fold(f64::NEG_INFINITY, f64::max);
            Value::Float(max)
        }
        CombineOp::Cat => {
            let s: String = vals.iter().map(|v| v.to_display_string()).collect();
            Value::String(s)
        }
        CombineOp::Merge => {
            let mut result = Vec::new();
            for v in vals {
                match v {
                    Value::Array(items) => result.extend(items),
                    other => result.push(other),
                }
            }
            Value::Array(result)
        }
        CombineOp::In => {
            let needle = &vals[0];
            let haystack = &vals[1];
            let found = match haystack {
                Value::Array(items) => items.iter().any(|item| needle.loose_equals(item)),
                Value::String(s) => {
                    let needle_str = needle.to_display_string();
                    s.contains(&needle_str)
                }
                _ => false,
            };
            Value::Bool(found)
        }
    };

    values.push(result);
    Ok(())
}

/// Get a variable from data by path (returns reference to avoid cloning)
fn get_variable_ref<'a>(path: &str, data: &'a Value) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(data);
    }

    let mut current = data;

    // Use iterator instead of collecting to Vec to avoid allocation
    for segment in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(segment)?,
            Value::Array(arr) => {
                let idx: usize = segment.parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }
    Some(current)
}

/// Get a variable from data by path (clones the result for ownership)
#[inline]
fn get_variable(path: &str, data: &Value) -> Option<Value> {
    get_variable_ref(path, data).cloned()
}

/// Convenience function for one-shot evaluation using iterative evaluator
pub fn evaluate_iterative(expr: &Expr, data: &Value) -> JsonLogicResult<Value> {
    IterativeEvaluator::new().evaluate(expr, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use serde_json::json;

    fn eval(rule: serde_json::Value, data: serde_json::Value) -> Value {
        let expr = parse(&rule).unwrap();
        let data_val = Value::from_json(&data);
        evaluate_iterative(&expr, &data_val).unwrap()
    }

    #[test]
    fn test_literal() {
        assert_eq!(eval(json!(42), json!({})), Value::Int(42));
        assert_eq!(eval(json!("hello"), json!({})), Value::String("hello".into()));
        assert_eq!(eval(json!(true), json!({})), Value::Bool(true));
    }

    #[test]
    fn test_var() {
        assert_eq!(eval(json!({"var": "x"}), json!({"x": 10})), Value::Int(10));
        assert_eq!(
            eval(json!({"var": "user.name"}), json!({"user": {"name": "Alice"}})),
            Value::String("Alice".into())
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval(json!({"+": [1, 2, 3]}), json!({})).to_number(), 6.0);
        assert_eq!(eval(json!({"-": [10, 3]}), json!({})).to_number(), 7.0);
        assert_eq!(eval(json!({"-": [10, 3, 2, 1]}), json!({})).to_number(), 4.0);
        assert_eq!(eval(json!({"*": [2, 3, 4]}), json!({})).to_number(), 24.0);
        assert_eq!(eval(json!({"/": [20, 4]}), json!({})).to_number(), 5.0);
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval(json!({"==": [1, 1]}), json!({})), Value::Bool(true));
        assert_eq!(eval(json!({"!=": [1, 2]}), json!({})), Value::Bool(true));
        assert_eq!(eval(json!({">": [5, 3]}), json!({})), Value::Bool(true));
        assert_eq!(eval(json!({"<": [1, 5, 10]}), json!({})), Value::Bool(true));
        assert_eq!(eval(json!({"<": [1, 5, 3]}), json!({})), Value::Bool(false));
    }

    #[test]
    fn test_logical() {
        assert_eq!(eval(json!({"and": [true, true]}), json!({})).is_truthy(), true);
        assert_eq!(eval(json!({"and": [true, false]}), json!({})).is_truthy(), false);
        assert_eq!(eval(json!({"or": [false, true]}), json!({})).is_truthy(), true);
        assert_eq!(eval(json!({"!": [true]}), json!({})), Value::Bool(false));
    }

    #[test]
    fn test_if() {
        assert_eq!(
            eval(json!({"if": [true, "yes", "no"]}), json!({})),
            Value::String("yes".into())
        );
        assert_eq!(
            eval(json!({"if": [false, "yes", "no"]}), json!({})),
            Value::String("no".into())
        );
    }

    #[test]
    fn test_map() {
        let result = eval(
            json!({"map": [{"var": "nums"}, {"*": [{"var": ""}, 2]}]}),
            json!({"nums": [1, 2, 3]}),
        );
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0].to_number(), 2.0);
                assert_eq!(arr[1].to_number(), 4.0);
                assert_eq!(arr[2].to_number(), 6.0);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_filter() {
        let result = eval(
            json!({"filter": [{"var": "nums"}, {">": [{"var": ""}, 2]}]}),
            json!({"nums": [1, 2, 3, 4, 5]}),
        );
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0].to_number(), 3.0);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_reduce() {
        let result = eval(
            json!({"reduce": [
                {"var": "nums"},
                {"+": [{"var": "accumulator"}, {"var": "current"}]},
                0
            ]}),
            json!({"nums": [1, 2, 3, 4, 5]}),
        );
        assert_eq!(result.to_number(), 15.0);
    }

    #[test]
    fn test_all_some_none() {
        // All > 0
        assert_eq!(
            eval(
                json!({"all": [{"var": "nums"}, {">": [{"var": ""}, 0]}]}),
                json!({"nums": [1, 2, 3]})
            ),
            Value::Bool(true)
        );

        // Some > 5
        assert_eq!(
            eval(
                json!({"some": [{"var": "nums"}, {">": [{"var": ""}, 5]}]}),
                json!({"nums": [1, 2, 10]})
            ),
            Value::Bool(true)
        );

        // None < 0
        assert_eq!(
            eval(
                json!({"none": [{"var": "nums"}, {"<": [{"var": ""}, 0]}]}),
                json!({"nums": [1, 2, 3]})
            ),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_deeply_nested() {
        // This would overflow with deep recursion, but loops handle it fine
        let rule = json!({
            "/": [
                {"-": [
                    {"*": [
                        {"+": [{"var": "x"}, 1]},
                        2
                    ]},
                    3
                ]},
                2
            ]
        });
        let result = eval(rule, json!({"x": 5}));
        assert_eq!(result.to_number(), 4.5);
    }

    #[test]
    fn test_multi_branch_if() {
        // Test multi-branch if/else
        let rule = json!({
            "if": [
                {">": [{"var": "age"}, 60]}, "senior",
                {">": [{"var": "age"}, 18]}, "adult",
                "minor"
            ]
        });

        assert_eq!(eval(rule.clone(), json!({"age": 70})), Value::String("senior".into()));
        assert_eq!(eval(rule.clone(), json!({"age": 30})), Value::String("adult".into()));
        assert_eq!(eval(rule.clone(), json!({"age": 15})), Value::String("minor".into()));
    }

    #[test]
    fn test_substr() {
        assert_eq!(
            eval(json!({"substr": ["Hello World", 0, 5]}), json!({})),
            Value::String("Hello".into())
        );
        assert_eq!(
            eval(json!({"substr": ["Hello World", -5]}), json!({})),
            Value::String("World".into())
        );
    }

    #[test]
    fn test_cat() {
        assert_eq!(
            eval(json!({"cat": ["Hello", " ", "World"]}), json!({})),
            Value::String("Hello World".into())
        );
    }

    #[test]
    fn test_merge() {
        let result = eval(json!({"merge": [[1, 2], [3, 4]]}), json!({}));
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 4);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_in() {
        assert_eq!(eval(json!({"in": [3, [1, 2, 3, 4]]}), json!({})), Value::Bool(true));
        assert_eq!(eval(json!({"in": [5, [1, 2, 3, 4]]}), json!({})), Value::Bool(false));
        assert_eq!(eval(json!({"in": ["ell", "Hello"]}), json!({})), Value::Bool(true));
    }

    #[test]
    fn test_missing() {
        let result = eval(json!({"missing": ["a", "b", "c"]}), json!({"a": 1, "c": 3}));
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0], Value::String("b".into()));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_min_max() {
        assert_eq!(eval(json!({"min": [5, 2, 8, 1]}), json!({})).to_number(), 1.0);
        assert_eq!(eval(json!({"max": [5, 2, 8, 1]}), json!({})).to_number(), 8.0);
    }
}

/// Comparison tests between iterative and recursive evaluators
#[cfg(test)]
mod consistency_tests {
    use super::*;
    use crate::evaluator::Evaluator;
    use crate::parser::parse;
    use serde_json::json;

    fn compare_results(ast: &Value, iter: &Value) -> bool {
        match (ast, iter) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-10,
            (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
                ((*a as f64) - b).abs() < 1e-10
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| compare_results(x, y))
            }
            _ => false,
        }
    }

    fn test_both(rule: serde_json::Value, data: serde_json::Value) {
        let expr = parse(&rule).unwrap();
        let data_val = Value::from_json(&data);

        // Recursive evaluator
        let recursive = Evaluator::new().evaluate_ast(&expr, &data).unwrap();

        // Iterative evaluator
        let iterative = IterativeEvaluator::new().evaluate(&expr, &data_val).unwrap();

        assert!(
            compare_results(&recursive, &iterative),
            "Mismatch for {:?}:\n  recursive: {:?}\n  iterative: {:?}",
            rule, recursive, iterative
        );
    }

    #[test]
    fn test_arithmetic_consistency() {
        test_both(json!({"+": [1, 2, 3]}), json!({}));
        test_both(json!({"-": [10, 3, 2, 1]}), json!({}));
        test_both(json!({"*": [2, 3, 4]}), json!({}));
        test_both(json!({"/": [20, 4]}), json!({}));
        test_both(json!({"%": [17, 5]}), json!({}));
    }

    #[test]
    fn test_comparison_consistency() {
        test_both(json!({"<": [1, 5, 10]}), json!({}));
        test_both(json!({"<=": [5, 5, 10]}), json!({}));
        test_both(json!({">": [10, 5, 1]}), json!({}));
        test_both(json!({">=": [10, 5, 5]}), json!({}));
    }

    #[test]
    fn test_logical_consistency() {
        test_both(json!({"and": [true, true, false]}), json!({}));
        test_both(json!({"or": [false, false, true]}), json!({}));
        test_both(json!({"!": [false]}), json!({}));
        test_both(json!({"!!": [0]}), json!({}));
    }

    #[test]
    fn test_if_consistency() {
        test_both(json!({"if": [true, "yes", "no"]}), json!({}));
        test_both(json!({"if": [false, "yes", "no"]}), json!({}));
        test_both(json!({
            "if": [
                {">": [{"var": "x"}, 10]}, "big",
                {">": [{"var": "x"}, 5]}, "medium",
                "small"
            ]
        }), json!({"x": 15}));
        test_both(json!({
            "if": [
                {">": [{"var": "x"}, 10]}, "big",
                {">": [{"var": "x"}, 5]}, "medium",
                "small"
            ]
        }), json!({"x": 3}));
    }

    #[test]
    fn test_array_ops_consistency() {
        test_both(
            json!({"map": [{"var": "nums"}, {"*": [{"var": ""}, 2]}]}),
            json!({"nums": [1, 2, 3]})
        );
        test_both(
            json!({"filter": [{"var": "nums"}, {">": [{"var": ""}, 2]}]}),
            json!({"nums": [1, 2, 3, 4, 5]})
        );
        test_both(
            json!({"reduce": [{"var": "nums"}, {"+": [{"var": "accumulator"}, {"var": "current"}]}, 0]}),
            json!({"nums": [1, 2, 3, 4, 5]})
        );
    }

    #[test]
    fn test_complex_expression_consistency() {
        // Insurance premium calculation
        test_both(json!({
            "*": [
                {"var": "base"},
                {"if": [
                    {">": [{"var": "age"}, 60]}, 1.5,
                    {">": [{"var": "age"}, 40]}, 1.2,
                    1.0
                ]}
            ]
        }), json!({"base": 100, "age": 45}));
    }
}
