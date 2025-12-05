//! Dynamic value types for attribute values and rule evaluation

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dynamic value that can hold any attribute value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Default)]
pub enum Value {
    /// Null/undefined value
    #[default]
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value (64-bit)
    Int(i64),
    /// Floating point value
    Float(f64),
    /// Decimal value for precise financial calculations
    #[serde(with = "rust_decimal::serde::str")]
    Decimal(Decimal),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<Value>),
    /// Object/map of values
    Object(HashMap<String, Value>),
}

impl Value {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if value is truthy (for boolean operations)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Decimal(d) => !d.is_zero(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }

    /// Try to get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => {
                // Check for NaN, Infinity, and out-of-range values
                if f.is_nan() || f.is_infinite() {
                    return None;
                }
                // Check if the float is within i64 range
                if *f < (i64::MIN as f64) || *f > (i64::MAX as f64) {
                    return None;
                }
                Some(*f as i64)
            }
            Value::Decimal(d) => d.to_string().parse().ok(),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::Decimal(d) => d.to_string().parse().ok(),
            _ => None,
        }
    }

    /// Try to get as decimal
    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Value::Int(i) => Some(Decimal::from(*i)),
            Value::Float(f) => Decimal::try_from(*f).ok(),
            Value::Decimal(d) => Some(*d),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try to get as string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to number (f64), coercing if possible
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            Value::Decimal(d) => d.to_string().parse().unwrap_or(0.0),
            Value::String(s) => s.parse().unwrap_or(0.0),
            Value::Array(_) => 0.0,
            Value::Object(_) => 0.0,
        }
    }

    /// Convert to display string
    pub fn to_display_string(&self) -> String {
        match self {
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

    /// Loose equality comparison (JavaScript-style coercion)
    pub fn loose_equals(&self, other: &Value) -> bool {
        match (self, other) {
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
            (Value::String(s), Value::Float(f)) => s.parse::<f64>().ok() == Some(*f),
            (Value::Float(f), Value::String(s)) => Some(*f) == s.parse::<f64>().ok(),

            _ => false,
        }
    }

    /// Try to get as array
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get as object
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => {
                serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Value::Decimal(d) => serde_json::Value::String(d.to_string()),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Array(a) => {
                serde_json::Value::Array(a.iter().map(|v| v.to_json()).collect())
            }
            Value::Object(o) => {
                serde_json::Value::Object(
                    o.iter().map(|(k, v)| (k.clone(), v.to_json())).collect(),
                )
            }
        }
    }

    /// Create from JSON value
    pub fn from_json(json: &serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Array(a) => {
                Value::Array(a.iter().map(Value::from_json).collect())
            }
            serde_json::Value::Object(o) => {
                Value::Object(
                    o.iter()
                        .map(|(k, v)| (k.clone(), Value::from_json(v)))
                        .collect(),
                )
            }
        }
    }
}


impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<Decimal> for Value {
    fn from(d: Decimal) -> Self {
        Value::Decimal(d)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

impl From<serde_json::Value> for Value {
    fn from(json: serde_json::Value) -> Self {
        Value::from_json(&json)
    }
}

/// Comparison result for Value types
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Decimal(a), Value::Decimal(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            // Cross-type numeric comparisons
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Int(a), Value::Decimal(b)) => Decimal::from(*a).partial_cmp(b),
            (Value::Decimal(a), Value::Int(b)) => a.partial_cmp(&Decimal::from(*b)),
            (Value::Float(a), Value::Decimal(b)) => {
                Decimal::try_from(*a).ok()?.partial_cmp(b)
            }
            (Value::Decimal(a), Value::Float(b)) => {
                a.partial_cmp(&Decimal::try_from(*b).ok()?)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_truthy() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::String("".to_string()).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
    }

    #[test]
    fn test_value_comparison() {
        assert!(Value::Int(5) > Value::Int(3));
        assert!(Value::Float(5.0) > Value::Int(3));
        assert!(Value::Int(5) > Value::Float(3.0));
        assert!(Value::String("b".to_string()) > Value::String("a".to_string()));
    }

    #[test]
    fn test_json_roundtrip() {
        let value = Value::Object(
            [
                ("name".to_string(), Value::String("test".to_string())),
                ("count".to_string(), Value::Int(42)),
                ("values".to_string(), Value::Array(vec![Value::Int(1), Value::Int(2)])),
            ]
            .into_iter()
            .collect(),
        );

        let json = value.to_json();
        let back = Value::from_json(&json);
        assert_eq!(value, back);
    }
}
