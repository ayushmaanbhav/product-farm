//! Evaluation functionality for product rules.
//!
//! Provides the `evaluate` function for executing rules against state.

use hashbrown::HashMap;
use product_farm_core::Value;
use product_farm_rule_engine::ExecutionContext;

/// Evaluation state - flexible key-value store.
#[derive(Debug, Clone, Default)]
pub struct State {
    values: HashMap<String, Value>,
}

impl State {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value in the state.
    pub fn set<V: IntoValue>(&mut self, key: impl Into<String>, value: V) -> &mut Self {
        self.values.insert(key.into(), value.into_value());
        self
    }

    /// Get a value from the state.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get all keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    /// Get all values.
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.values.values()
    }

    /// Get the inner map.
    pub fn inner(&self) -> &HashMap<String, Value> {
        &self.values
    }

    /// Convert to hashbrown HashMap (zero-cost, returns clone of inner).
    pub fn to_hashbrown(&self) -> HashMap<String, Value> {
        self.values.clone()
    }

    /// Convert to std::collections::HashMap (for LLM evaluator compatibility).
    pub fn to_std_hashmap(&self) -> std::collections::HashMap<String, Value> {
        self.values.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Convert from JSON value.
    pub fn from_json(json: serde_json::Value) -> Self {
        let mut state = Self::new();
        if let serde_json::Value::Object(map) = json {
            for (key, value) in map {
                state.set(key, json_to_value(value));
            }
        }
        state
    }

    /// Convert to JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        let map: serde_json::Map<String, serde_json::Value> = self
            .values
            .iter()
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect();
        serde_json::Value::Object(map)
    }

    /// Merge another state into this one.
    pub fn merge(&mut self, other: State) {
        self.values.extend(other.values);
    }

    /// Create from context (internal conversion).
    pub(crate) fn from_context(context: &ExecutionContext) -> Self {
        let json = context.to_json();
        Self::from_json(json)
    }
}

/// Evaluation result.
#[derive(Debug)]
pub struct EvalResult {
    /// Updated state after evaluation.
    pub state: State,

    /// Computed output values.
    pub outputs: std::collections::HashMap<String, Value>,

    /// Names of rules that were executed.
    pub executed_rules: Vec<String>,

    /// Execution time in milliseconds.
    pub execution_time_ms: u64,
}


// =============================================================================
// Value Conversion Helpers
// =============================================================================

fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, json_to_value(v))).collect())
        }
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        Value::Decimal(d) => {
            serde_json::Value::String(d.to_string())
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::Object(obj) => {
            serde_json::Value::Object(
                obj.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect()
            )
        }
    }
}

// Helper trait for converting into Value
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::Float(self as f64)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_set_get() {
        let mut state = State::new();
        state.set("name", "Alice");
        state.set("age", 30);
        state.set("score", 95.5);
        state.set("active", true);

        assert_eq!(state.get("name"), Some(&Value::String("Alice".into())));
        assert_eq!(state.get("age"), Some(&Value::Int(30)));
        assert_eq!(state.get("score"), Some(&Value::Float(95.5)));
        assert_eq!(state.get("active"), Some(&Value::Bool(true)));
        assert_eq!(state.get("missing"), None);
    }

    #[test]
    fn test_state_json_roundtrip() {
        let mut state = State::new();
        state.set("name", "Bob");
        state.set("count", 42);

        let json = state.to_json();
        let restored = State::from_json(json);

        assert_eq!(restored.get("name"), state.get("name"));
        assert_eq!(restored.get("count"), state.get("count"));
    }

    #[test]
    fn test_state_merge() {
        let mut state1 = State::new();
        state1.set("a", 1);
        state1.set("b", 2);

        let mut state2 = State::new();
        state2.set("b", 3);  // Override
        state2.set("c", 4);

        state1.merge(state2);

        assert_eq!(state1.get("a"), Some(&Value::Int(1)));
        assert_eq!(state1.get("b"), Some(&Value::Int(3)));  // Overridden
        assert_eq!(state1.get("c"), Some(&Value::Int(4)));
    }
}
