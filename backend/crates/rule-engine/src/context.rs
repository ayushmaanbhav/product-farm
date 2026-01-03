//! Execution context for rule evaluation
//!
//! Manages the data available during rule execution, including:
//! - Input data from the client
//! - Computed values from previously executed rules
//! - Attribute definitions and constraints

use hashbrown::HashMap;
use product_farm_core::Value;
use std::sync::Arc;

/// The execution context holds all data available during rule evaluation
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Input data provided by the client
    input: Arc<HashMap<String, Value>>,
    /// Computed values from executed rules
    computed: HashMap<String, Value>,
    /// Metadata about the execution
    metadata: HashMap<String, Value>,
}

impl ExecutionContext {
    /// Create a new execution context with input data
    pub fn new(input: HashMap<String, Value>) -> Self {
        Self {
            input: Arc::new(input),
            computed: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create from JSON value
    pub fn from_json(json: &serde_json::Value) -> Self {
        let input = match json {
            serde_json::Value::Object(map) => {
                map.iter()
                    .map(|(k, v)| (k.clone(), Value::from(v.clone())))
                    .collect()
            }
            _ => HashMap::new(),
        };
        Self::new(input)
    }

    /// Get a value from the context (checks computed first, then input)
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.computed.get(key).or_else(|| self.input.get(key))
    }

    /// Get a nested value using dot notation
    pub fn get_path(&self, path: &str) -> Option<Value> {
        if path.is_empty() {
            // Return entire context as an object
            let mut all = std::collections::HashMap::new();
            for (k, v) in self.input.iter() {
                all.insert(k.clone(), v.clone());
            }
            for (k, v) in self.computed.iter() {
                all.insert(k.clone(), v.clone());
            }
            return Some(Value::Object(all));
        }

        let segments: Vec<&str> = path.split('.').collect();
        let first = segments[0];

        let value = self.get(first)?;

        if segments.len() == 1 {
            return Some(value.clone());
        }

        // Navigate nested path
        let mut current = value.clone();
        for segment in &segments[1..] {
            current = match current {
                Value::Object(map) => map.get(*segment)?.clone(),
                Value::Array(arr) => {
                    let idx: usize = segment.parse().ok()?;
                    arr.get(idx)?.clone()
                }
                _ => return None,
            };
        }
        Some(current)
    }

    /// Set a computed value
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.computed.insert(key.into(), value);
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Get all computed values
    pub fn computed_values(&self) -> &HashMap<String, Value> {
        &self.computed
    }

    /// Get all input values
    pub fn input_values(&self) -> &HashMap<String, Value> {
        &self.input
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: &ExecutionContext) {
        for (k, v) in other.computed.iter() {
            self.computed.insert(k.clone(), v.clone());
        }
    }

    /// Check if a key exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.computed.contains_key(key) || self.input.contains_key(key)
    }

    /// Get the list of all available keys
    pub fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.input.keys().map(|s| s.as_str()).collect();
        for k in self.computed.keys() {
            if !self.input.contains_key(k) {
                keys.push(k.as_str());
            }
        }
        keys
    }

    /// Get all available input keys as a HashSet (for dependency validation)
    pub fn available_inputs(&self) -> hashbrown::HashSet<String> {
        let mut inputs = hashbrown::HashSet::new();
        for k in self.input.keys() {
            inputs.insert(k.clone());
        }
        for k in self.computed.keys() {
            inputs.insert(k.clone());
        }
        inputs
    }

    /// Convert context to Value for evaluation (avoids JSON intermediate step).
    ///
    /// This is more efficient than `to_json()` when the evaluator can work with Value directly.
    /// Converts flat dot-separated paths into nested structures for proper var navigation.
    pub fn to_value(&self) -> Value {
        let mut root = std::collections::HashMap::new();

        // Helper to insert a value at a dot-separated path into a nested structure
        fn insert_at_path(map: &mut std::collections::HashMap<String, Value>, path: &str, value: Value) {
            let segments: Vec<&str> = path.split('.').collect();

            if segments.is_empty() {
                return;
            }

            if segments.len() == 1 {
                // Simple key without dots
                map.insert(path.to_string(), value);
                return;
            }

            // Navigate/create nested structure
            let first = segments[0];

            // Get or create the nested object
            let nested = map.entry(first.to_string())
                .or_insert_with(|| Value::Object(std::collections::HashMap::new()));

            if let Value::Object(nested_map) = nested {
                // Recurse with remaining path segments
                let remaining_path = segments[1..].join(".");
                insert_at_path(nested_map, &remaining_path, value);
            }
        }

        // Add input values
        for (k, v) in self.input.iter() {
            insert_at_path(&mut root, k, v.clone());
        }

        // Add computed values (overwrite input if same key)
        for (k, v) in self.computed.iter() {
            insert_at_path(&mut root, k, v.clone());
        }

        Value::Object(root)
    }

    /// Convert context to JSON for evaluation
    /// Converts flat dot-separated paths like "loan.main.input-val" into nested JSON structure
    /// so that JSON Logic's var operator can navigate them correctly.
    pub fn to_json(&self) -> serde_json::Value {
        let mut root = serde_json::Map::new();

        // Helper to insert a value at a dot-separated path into a nested structure
        fn insert_at_path(map: &mut serde_json::Map<String, serde_json::Value>, path: &str, value: serde_json::Value) {
            let segments: Vec<&str> = path.split('.').collect();

            if segments.is_empty() {
                return;
            }

            if segments.len() == 1 {
                // Simple key without dots
                map.insert(path.to_string(), value);
                return;
            }

            // Navigate/create nested structure
            let first = segments[0];

            // Get or create the nested object
            let nested = map.entry(first.to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

            if let serde_json::Value::Object(nested_map) = nested {
                // Recurse with remaining path segments
                let remaining_path = segments[1..].join(".");
                insert_at_path(nested_map, &remaining_path, value);
            }
        }

        // Add input values
        for (k, v) in self.input.iter() {
            insert_at_path(&mut root, k, v.to_json());
        }

        // Add computed values (overwrite input if same key)
        for (k, v) in self.computed.iter() {
            insert_at_path(&mut root, k, v.to_json());
        }

        serde_json::Value::Object(root)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_get_set() {
        let mut ctx = ExecutionContext::new(HashMap::from([
            ("x".into(), Value::Int(10)),
            ("y".into(), Value::Int(20)),
        ]));

        assert_eq!(ctx.get("x"), Some(&Value::Int(10)));
        assert_eq!(ctx.get("z"), None);

        ctx.set("z", Value::Int(30));
        assert_eq!(ctx.get("z"), Some(&Value::Int(30)));
    }

    #[test]
    fn test_context_nested_path() {
        let ctx = ExecutionContext::new(HashMap::from([(
            "user".into(),
            Value::Object(std::collections::HashMap::from([
                ("name".into(), Value::String("Alice".into())),
                ("age".into(), Value::Int(25)),
            ])),
        )]));

        assert_eq!(ctx.get_path("user.name"), Some(Value::String("Alice".into())));
        assert_eq!(ctx.get_path("user.age"), Some(Value::Int(25)));
        assert_eq!(ctx.get_path("user.unknown"), None);
    }

    #[test]
    fn test_computed_overrides_input() {
        let mut ctx = ExecutionContext::new(HashMap::from([
            ("x".into(), Value::Int(10)),
        ]));

        ctx.set("x", Value::Int(100));
        assert_eq!(ctx.get("x"), Some(&Value::Int(100)));
    }
}
