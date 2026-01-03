//! Tiered Compilation for JSON Logic
//!
//! Implements automatic promotion from AST interpretation to bytecode VM
//! based on evaluation count. This optimizes frequently-used rules.
//!
//! Tiers:
//! - Tier 0 (AST): Default for all new rules, fastest startup
//! - Tier 1 (Bytecode): After PROMOTION_THRESHOLD evaluations, compiled to bytecode
//! - Tier 2 (JIT): Reserved for future Cranelift JIT compilation

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use product_farm_core::Value;

use crate::config::Config;
use crate::iter_eval::IterativeEvaluator;
use crate::{CompiledBytecode, Compiler, EvalContext, Expr, JsonLogicResult, VM};

/// Compilation tier for a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompilationTier {
    /// Tier 0: AST interpretation (default)
    Ast,
    /// Tier 1: Bytecode VM execution
    Bytecode,
    /// Tier 2: JIT compiled (future)
    Jit,
}

impl Default for CompilationTier {
    fn default() -> Self {
        Self::Ast
    }
}

/// A compiled rule with tier tracking
pub struct CompiledRule {
    /// The parsed AST (always available)
    pub ast: Expr,
    /// Compiled bytecode (populated after promotion to Tier 1)
    bytecode: Option<CompiledBytecode>,
    /// Current compilation tier
    tier: CompilationTier,
    /// Number of times this rule has been evaluated
    eval_count: AtomicU64,
}

impl CompiledRule {
    /// Create a new compiled rule at Tier 0 (AST)
    pub fn new(ast: Expr) -> Self {
        Self {
            ast,
            bytecode: None,
            tier: CompilationTier::Ast,
            eval_count: AtomicU64::new(0),
        }
    }

    /// Create a new compiled rule pre-compiled to Tier 1 (Bytecode)
    pub fn with_bytecode(ast: Expr, bytecode: CompiledBytecode) -> Self {
        Self {
            ast,
            bytecode: Some(bytecode),
            tier: CompilationTier::Bytecode,
            eval_count: AtomicU64::new(0),
        }
    }

    /// Get the current compilation tier
    pub fn tier(&self) -> CompilationTier {
        self.tier
    }

    /// Get the evaluation count
    pub fn eval_count(&self) -> u64 {
        self.eval_count.load(Ordering::Relaxed)
    }

    /// Check if this rule should be promoted to a higher tier (uses default threshold)
    pub fn should_promote(&self) -> bool {
        self.should_promote_at(Config::global().bytecode_promotion_threshold)
    }

    /// Check if this rule should be promoted with a custom threshold
    pub fn should_promote_at(&self, threshold: u64) -> bool {
        self.tier == CompilationTier::Ast
            && self.eval_count.load(Ordering::Relaxed) >= threshold
    }

    /// Promote to Tier 1 (Bytecode)
    pub fn promote_to_bytecode(&mut self) -> JsonLogicResult<()> {
        if self.tier != CompilationTier::Ast {
            return Ok(()); // Already promoted or higher
        }

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&self.ast)?;
        self.bytecode = Some(bytecode);
        self.tier = CompilationTier::Bytecode;
        Ok(())
    }

    /// Evaluate this rule with automatic tier selection
    pub fn evaluate(&self, data: &serde_json::Value) -> JsonLogicResult<Value> {
        // Increment evaluation count
        self.eval_count.fetch_add(1, Ordering::Relaxed);

        match self.tier {
            CompilationTier::Ast => {
                // Use loop-based evaluator (no recursion, prevents stack overflow)
                let data_val = Value::from_json(data);
                IterativeEvaluator::new().evaluate(&self.ast, &data_val)
            }
            CompilationTier::Bytecode | CompilationTier::Jit => {
                if let Some(ref bytecode) = self.bytecode {
                    let context = EvalContext::from_json(data, bytecode);
                    let mut vm = VM::new();
                    vm.execute(bytecode, &context)
                } else {
                    // Fallback to loop-based evaluator if bytecode not available
                    let data_val = Value::from_json(data);
                    IterativeEvaluator::new().evaluate(&self.ast, &data_val)
                }
            }
        }
    }

    /// Serialize to a persistable format
    pub fn to_persisted(&self) -> PersistedRule {
        PersistedRule {
            ast: self.ast.clone(),
            bytecode: self.bytecode.clone(),
            tier: self.tier,
            eval_count: self.eval_count.load(Ordering::Relaxed),
        }
    }

    /// Restore from a persisted format
    pub fn from_persisted(persisted: PersistedRule) -> Self {
        Self {
            ast: persisted.ast,
            bytecode: persisted.bytecode,
            tier: persisted.tier,
            eval_count: AtomicU64::new(persisted.eval_count),
        }
    }

    /// Get the bytecode if available
    pub fn bytecode(&self) -> Option<&CompiledBytecode> {
        self.bytecode.as_ref()
    }
}

/// Serializable version of CompiledRule for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedRule {
    /// The parsed AST
    pub ast: Expr,
    /// Compiled bytecode (if promoted to Tier 1)
    pub bytecode: Option<CompiledBytecode>,
    /// Current compilation tier
    pub tier: CompilationTier,
    /// Number of evaluations at time of persistence
    pub eval_count: u64,
}

impl PersistedRule {
    /// Serialize to JSON bytes
    pub fn to_json(&self) -> JsonLogicResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| crate::JsonLogicError::InvalidStructure(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn from_json(bytes: &[u8]) -> JsonLogicResult<Self> {
        serde_json::from_slice(bytes).map_err(|e| crate::JsonLogicError::InvalidStructure(e.to_string()))
    }
}

/// Cache for compiled rules with automatic tier promotion
pub struct RuleCache {
    /// Cached compiled rules by rule ID
    rules: RwLock<HashMap<String, Arc<RwLock<CompiledRule>>>>,
    /// Promotion threshold (configurable)
    promotion_threshold: u64,
}

impl RuleCache {
    /// Create a new rule cache with default promotion threshold
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            promotion_threshold: Config::global().bytecode_promotion_threshold,
        }
    }

    /// Create a new rule cache with custom promotion threshold
    pub fn with_threshold(threshold: u64) -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            promotion_threshold: threshold,
        }
    }

    /// Get or create a compiled rule for the given ID and expression
    pub fn get_or_compile(&self, rule_id: &str, expr: &Expr) -> Arc<RwLock<CompiledRule>> {
        // Check if rule exists
        {
            let rules = self.rules.read().unwrap();
            if let Some(rule) = rules.get(rule_id) {
                return Arc::clone(rule);
            }
        }

        // Create new rule
        let compiled = Arc::new(RwLock::new(CompiledRule::new(expr.clone())));

        // Insert into cache
        {
            let mut rules = self.rules.write().unwrap();
            rules.insert(rule_id.to_string(), Arc::clone(&compiled));
        }

        compiled
    }

    /// Evaluate a rule with automatic caching and tier promotion
    pub fn evaluate(
        &self,
        rule_id: &str,
        expr: &Expr,
        data: &serde_json::Value,
    ) -> JsonLogicResult<Value> {
        let compiled = self.get_or_compile(rule_id, expr);

        // Evaluate
        let result = {
            let rule = compiled.read().unwrap();
            rule.evaluate(data)
        };

        // Check for promotion after evaluation
        {
            let rule = compiled.read().unwrap();
            if rule.should_promote_at(self.promotion_threshold) {
                drop(rule); // Release read lock
                let mut rule = compiled.write().unwrap();
                if rule.should_promote_at(self.promotion_threshold) {
                    // Double-check after acquiring write lock
                    if let Err(e) = rule.promote_to_bytecode() {
                        // Log but don't fail - continue with AST
                        eprintln!("Warning: Failed to promote rule {} to bytecode: {}", rule_id, e);
                    }
                }
            }
        }

        result
    }

    /// Force compile a rule to bytecode (Tier 1)
    pub fn force_compile(&self, rule_id: &str, expr: &Expr) -> JsonLogicResult<()> {
        let compiled = self.get_or_compile(rule_id, expr);
        let mut rule = compiled.write().unwrap();
        rule.promote_to_bytecode()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let rules = self.rules.read().unwrap();
        let mut stats = CacheStats::default();

        for (_, rule) in rules.iter() {
            let rule = rule.read().unwrap();
            stats.total_rules += 1;
            match rule.tier() {
                CompilationTier::Ast => stats.tier0_rules += 1,
                CompilationTier::Bytecode => stats.tier1_rules += 1,
                CompilationTier::Jit => stats.tier2_rules += 1,
            }
            stats.total_evaluations += rule.eval_count();
        }

        stats
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut rules = self.rules.write().unwrap();
        rules.clear();
    }

    /// Remove a specific rule from the cache
    pub fn remove(&self, rule_id: &str) -> bool {
        let mut rules = self.rules.write().unwrap();
        rules.remove(rule_id).is_some()
    }

    /// Get the number of cached rules
    pub fn len(&self) -> usize {
        self.rules.read().unwrap().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.rules.read().unwrap().is_empty()
    }
}

impl Default for RuleCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the rule cache
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Total number of rules in cache
    pub total_rules: usize,
    /// Number of rules at Tier 0 (AST)
    pub tier0_rules: usize,
    /// Number of rules at Tier 1 (Bytecode)
    pub tier1_rules: usize,
    /// Number of rules at Tier 2 (JIT)
    pub tier2_rules: usize,
    /// Total number of evaluations across all rules
    pub total_evaluations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use serde_json::json;

    #[test]
    fn test_compiled_rule_starts_at_tier0() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let rule = CompiledRule::new(expr);

        assert_eq!(rule.tier(), CompilationTier::Ast);
        assert_eq!(rule.eval_count(), 0);
    }

    #[test]
    fn test_compiled_rule_evaluation_increments_count() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let rule = CompiledRule::new(expr);

        let result = rule.evaluate(&json!({"x": 21})).unwrap();
        assert_eq!(result, Value::Float(42.0));
        assert_eq!(rule.eval_count(), 1);
    }

    #[test]
    fn test_compiled_rule_manual_promotion() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let mut rule = CompiledRule::new(expr);

        assert_eq!(rule.tier(), CompilationTier::Ast);

        rule.promote_to_bytecode().unwrap();
        assert_eq!(rule.tier(), CompilationTier::Bytecode);

        // Should still work after promotion
        let result = rule.evaluate(&json!({"x": 21})).unwrap();
        assert_eq!(result.to_number(), 42.0);
    }

    #[test]
    fn test_rule_cache_basic() {
        let cache = RuleCache::new();
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();

        let result = cache.evaluate("rule1", &expr, &json!({"x": 21})).unwrap();
        assert_eq!(result, Value::Float(42.0));

        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_rule_cache_automatic_promotion() {
        let cache = RuleCache::with_threshold(5); // Low threshold for testing
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();

        // Evaluate 10 times to trigger promotion
        for i in 0..10 {
            let result = cache.evaluate("rule1", &expr, &json!({"x": i})).unwrap();
            assert_eq!(result.to_number(), (i * 2) as f64);
        }

        // Check that rule was promoted
        let stats = cache.stats();
        assert_eq!(stats.tier1_rules, 1);
        assert_eq!(stats.tier0_rules, 0);
    }

    #[test]
    fn test_rule_cache_force_compile() {
        let cache = RuleCache::new();
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();

        // Force compile without any evaluations
        cache.force_compile("rule1", &expr).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.tier1_rules, 1);
    }

    #[test]
    fn test_rule_cache_stats() {
        let cache = RuleCache::with_threshold(100);
        let expr1 = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let expr2 = parse(&json!({"+": [{"var": "a"}, {"var": "b"}]})).unwrap();

        // Add two rules
        cache.evaluate("rule1", &expr1, &json!({"x": 1})).unwrap();
        cache.evaluate("rule2", &expr2, &json!({"a": 1, "b": 2})).unwrap();

        // Force compile one
        cache.force_compile("rule1", &expr1).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_rules, 2);
        assert_eq!(stats.tier0_rules, 1);
        assert_eq!(stats.tier1_rules, 1);
        assert_eq!(stats.total_evaluations, 2);
    }

    #[test]
    fn test_rule_cache_remove() {
        let cache = RuleCache::new();
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();

        cache.evaluate("rule1", &expr, &json!({"x": 1})).unwrap();
        assert_eq!(cache.len(), 1);

        assert!(cache.remove("rule1"));
        assert_eq!(cache.len(), 0);

        assert!(!cache.remove("nonexistent"));
    }

    #[test]
    fn test_with_bytecode_constructor() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();

        let rule = CompiledRule::with_bytecode(expr, bytecode);
        assert_eq!(rule.tier(), CompilationTier::Bytecode);

        let result = rule.evaluate(&json!({"x": 21})).unwrap();
        assert_eq!(result.to_number(), 42.0);
    }

    #[test]
    fn test_persistence_roundtrip_tier0() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let rule = CompiledRule::new(expr);

        // Evaluate a few times
        for i in 0..5 {
            rule.evaluate(&json!({"x": i})).unwrap();
        }

        // Persist
        let persisted = rule.to_persisted();
        let bytes = persisted.to_json().unwrap();

        // Restore
        let restored_persisted = PersistedRule::from_json(&bytes).unwrap();
        let restored_rule = CompiledRule::from_persisted(restored_persisted);

        assert_eq!(restored_rule.tier(), CompilationTier::Ast);
        assert_eq!(restored_rule.eval_count(), 5);

        // Verify it still works
        let result = restored_rule.evaluate(&json!({"x": 21})).unwrap();
        assert_eq!(result.to_number(), 42.0);
    }

    #[test]
    fn test_persistence_roundtrip_tier1() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let mut rule = CompiledRule::new(expr);

        // Promote to bytecode
        rule.promote_to_bytecode().unwrap();
        assert_eq!(rule.tier(), CompilationTier::Bytecode);

        // Persist (including bytecode)
        let persisted = rule.to_persisted();
        assert!(persisted.bytecode.is_some());
        let bytes = persisted.to_json().unwrap();

        // Restore
        let restored_persisted = PersistedRule::from_json(&bytes).unwrap();
        let restored_rule = CompiledRule::from_persisted(restored_persisted);

        assert_eq!(restored_rule.tier(), CompilationTier::Bytecode);
        assert!(restored_rule.bytecode().is_some());

        // Verify it works with restored bytecode
        let result = restored_rule.evaluate(&json!({"x": 21})).unwrap();
        assert_eq!(result.to_number(), 42.0);
    }

    #[test]
    fn test_persistence_complex_expression() {
        let expr = parse(&json!({
            "if": [
                {">": [{"var": "score"}, 90]}, "A",
                {">": [{"var": "score"}, 80]}, "B",
                {">": [{"var": "score"}, 70]}, "C",
                "D"
            ]
        })).unwrap();
        let mut rule = CompiledRule::new(expr);
        rule.promote_to_bytecode().unwrap();

        // Persist and restore
        let persisted = rule.to_persisted();
        let bytes = persisted.to_json().unwrap();
        let restored = CompiledRule::from_persisted(PersistedRule::from_json(&bytes).unwrap());

        // Test various scores
        assert_eq!(restored.evaluate(&json!({"score": 95})).unwrap(), Value::String("A".into()));
        assert_eq!(restored.evaluate(&json!({"score": 85})).unwrap(), Value::String("B".into()));
        assert_eq!(restored.evaluate(&json!({"score": 75})).unwrap(), Value::String("C".into()));
        assert_eq!(restored.evaluate(&json!({"score": 65})).unwrap(), Value::String("D".into()));
    }

    #[test]
    fn test_persisted_rule_json_size() {
        let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
        let mut rule = CompiledRule::new(expr);
        rule.promote_to_bytecode().unwrap();

        let persisted = rule.to_persisted();
        let bytes = persisted.to_json().unwrap();

        // Should be reasonably compact
        assert!(bytes.len() < 1000, "JSON too large: {} bytes", bytes.len());

        // Verify JSON is valid
        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains("\"tier\":\"Bytecode\""));
        assert!(json_str.contains("\"bytecode\""));
    }
}
