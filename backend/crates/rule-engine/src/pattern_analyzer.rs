//! Smart Pattern Analyzer for Rules and Attributes
//!
//! Uses natural language friendly pattern matching to detect, group, and tag
//! similar rules, attributes, functions, and components. Computes metrics and
//! extracts insights with a curious mindset.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal rule representation for pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub expression: serde_json::Value,
    pub output_attribute: String,
    pub dependencies: Vec<String>,
}

/// Pattern categories detected through natural language analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternCategory {
    /// Calculation patterns (arithmetic, aggregation)
    Calculation(CalculationType),
    /// Conditional logic patterns
    Conditional(ConditionalType),
    /// Data transformation patterns
    Transformation(TransformationType),
    /// Validation patterns
    Validation(ValidationType),
    /// Lookup/reference patterns
    Lookup(LookupType),
    /// Aggregation patterns
    Aggregation(AggregationType),
    /// Custom/unknown patterns
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CalculationType {
    Multiplication,    // base * factor
    Division,          // amount / count
    Addition,          // sum of values
    Subtraction,       // difference
    Percentage,        // value * 0.xx or value / 100
    Compound,          // multiple operations
    Rounding,          // round, floor, ceil
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConditionalType {
    ThresholdBased,    // if value < threshold
    RangeBased,        // if min <= value <= max
    EqualityCheck,     // if status == "active"
    MultiCondition,    // if A and B and C
    Tiered,            // if/else if/else if/else (multiple tiers)
    BooleanLogic,      // and/or combinations
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransformationType {
    TypeConversion,    // string to number, etc.
    Formatting,        // date format, number format
    Normalization,     // standardize values
    Mapping,           // map values to other values
    Concatenation,     // string joining
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValidationType {
    RangeCheck,        // value in bounds
    RequiredField,     // not null/empty
    PatternMatch,      // regex validation
    CrossField,        // field A depends on field B
    BusinessRule,      // complex validation logic
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LookupType {
    TableLookup,       // lookup from reference data
    VariableAccess,    // simple var access
    NestedAccess,      // object.property.subproperty
    ArrayIndex,        // array[index]
    DynamicKey,        // object[variable]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AggregationType {
    Sum,
    Average,
    Count,
    Min,
    Max,
    First,
    Last,
    All,               // all items match
    Some,              // any item matches
    None,              // no items match
}

/// Semantic grouping based on natural language analysis of names and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGroup {
    /// Human-readable group name
    pub name: String,
    /// Natural language description
    pub description: String,
    /// Keywords that triggered this grouping
    pub keywords: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Member rule IDs
    pub rule_ids: Vec<String>,
    /// Member attribute paths
    pub attribute_paths: Vec<String>,
    /// Detected patterns in this group
    pub patterns: Vec<PatternCategory>,
    /// Computed metrics for this group
    pub metrics: GroupMetrics,
}

/// Metrics computed for a semantic group
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMetrics {
    /// Total rules in group
    pub rule_count: usize,
    /// Total attributes in group
    pub attribute_count: usize,
    /// Average complexity (AST nodes)
    pub avg_complexity: f64,
    /// Max dependency depth
    pub max_depth: usize,
    /// Parallelization potential (0.0 - 1.0)
    pub parallelism_score: f64,
    /// Estimated execution time (ns)
    pub estimated_time_ns: u64,
    /// LLM rule count
    pub llm_rule_count: usize,
    /// Bytecode compilation rate
    pub bytecode_rate: f64,
}

/// Individual rule insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInsight {
    pub rule_id: String,
    /// Detected patterns
    pub patterns: Vec<PatternCategory>,
    /// Semantic tags (natural language)
    pub tags: Vec<String>,
    /// Similar rules (by pattern)
    pub similar_rules: Vec<SimilarityMatch>,
    /// Complexity breakdown
    pub complexity: ComplexityBreakdown,
    /// Natural language summary
    pub summary: String,
    /// Optimization suggestions
    pub suggestions: Vec<Suggestion>,
    /// Interesting facts
    pub facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub rule_id: String,
    pub similarity_score: f64, // 0.0 - 1.0
    pub matching_patterns: Vec<PatternCategory>,
    pub reason: String, // Natural language explanation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBreakdown {
    pub total_nodes: usize,
    pub max_depth: usize,
    pub variable_count: usize,
    pub operator_count: usize,
    pub condition_count: usize,
    pub loop_count: usize,
    pub cyclomatic_complexity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub priority: SuggestionPriority,
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub estimated_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuggestionPriority {
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Maintainability,
    Correctness,
    Simplification,
    Consolidation,
}

/// Natural language keyword patterns for detection
pub struct KeywordPatterns {
    /// Pricing/financial keywords
    pub pricing: Vec<&'static str>,
    /// Risk/assessment keywords
    pub risk: Vec<&'static str>,
    /// Customer/user keywords
    pub customer: Vec<&'static str>,
    /// Discount/promotion keywords
    pub discount: Vec<&'static str>,
    /// Validation keywords
    pub validation: Vec<&'static str>,
    /// Calculation keywords
    pub calculation: Vec<&'static str>,
    /// Eligibility keywords
    pub eligibility: Vec<&'static str>,
    /// Coverage keywords
    pub coverage: Vec<&'static str>,
}

impl Default for KeywordPatterns {
    fn default() -> Self {
        Self {
            pricing: vec![
                "price", "premium", "rate", "cost", "fee", "charge", "amount",
                "total", "subtotal", "tax", "commission", "margin", "markup",
            ],
            risk: vec![
                "risk", "score", "factor", "rating", "assessment", "level",
                "hazard", "exposure", "probability", "severity", "impact",
            ],
            customer: vec![
                "customer", "user", "client", "member", "subscriber", "account",
                "profile", "person", "individual", "applicant", "policyholder",
            ],
            discount: vec![
                "discount", "rebate", "reduction", "saving", "offer", "promotion",
                "coupon", "loyalty", "bonus", "credit", "waiver",
            ],
            validation: vec![
                "valid", "check", "verify", "validate", "ensure", "require",
                "mandatory", "eligible", "allowed", "permitted", "approved",
            ],
            calculation: vec![
                "calculate", "compute", "derive", "formula", "equation",
                "multiply", "divide", "sum", "average", "aggregate",
            ],
            eligibility: vec![
                "eligible", "qualify", "entitled", "applicable", "available",
                "permitted", "authorized", "approved", "meets", "satisfies",
            ],
            coverage: vec![
                "cover", "coverage", "insure", "protect", "benefit", "policy",
                "claim", "limit", "deductible", "excess", "exclusion",
            ],
        }
    }
}

/// Main pattern analyzer
pub struct PatternAnalyzer {
    keywords: KeywordPatterns,
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            keywords: KeywordPatterns::default(),
        }
    }

    /// Analyze a collection of rules and extract patterns, groups, and insights
    pub fn analyze_rules(&self, rules: &[Rule]) -> AnalysisResult {
        let mut result = AnalysisResult::default();

        // Phase 1: Extract patterns from each rule
        for rule in rules {
            let insight = self.analyze_single_rule(rule);
            result.rule_insights.insert(rule.id.clone(), insight);
        }

        // Phase 2: Detect semantic groups
        result.semantic_groups = self.detect_semantic_groups(rules);

        // Phase 3: Find similar rules
        self.find_similar_rules(&mut result, rules);

        // Phase 4: Compute global metrics
        result.global_metrics = self.compute_global_metrics(rules, &result);

        // Phase 5: Generate interesting facts
        result.interesting_facts = self.generate_facts(&result, rules);

        result
    }

    /// Analyze a single rule
    fn analyze_single_rule(&self, rule: &Rule) -> RuleInsight {
        let patterns = self.detect_patterns_from_expression(&rule.expression);
        let tags = self.extract_semantic_tags(&rule.id, &rule.output_attribute);
        let complexity = self.analyze_complexity(&rule.expression);
        let summary = self.generate_natural_summary(rule, &patterns);
        let suggestions = self.generate_suggestions(rule, &patterns, &complexity);
        let facts = self.generate_rule_facts(rule, &patterns, &complexity);

        RuleInsight {
            rule_id: rule.id.clone(),
            patterns,
            tags,
            similar_rules: vec![], // Filled in later
            complexity,
            summary,
            suggestions,
            facts,
        }
    }

    /// Detect patterns from a JSON Logic expression
    fn detect_patterns_from_expression(&self, expr: &serde_json::Value) -> Vec<PatternCategory> {
        let mut patterns = Vec::new();

        self.detect_patterns_recursive(expr, &mut patterns);

        // Deduplicate
        patterns.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        patterns.dedup();

        patterns
    }

    fn detect_patterns_recursive(
        &self,
        expr: &serde_json::Value,
        patterns: &mut Vec<PatternCategory>,
    ) {
        match expr {
            serde_json::Value::Object(map) => {
                for (op, args) in map {
                    match op.as_str() {
                        // Arithmetic operations
                        "*" => {
                            // Check if it's a percentage (multiply by 0.xx)
                            if self.is_percentage_pattern(args) {
                                patterns.push(PatternCategory::Calculation(
                                    CalculationType::Percentage,
                                ));
                            } else {
                                patterns.push(PatternCategory::Calculation(
                                    CalculationType::Multiplication,
                                ));
                            }
                        }
                        "/" => patterns.push(PatternCategory::Calculation(CalculationType::Division)),
                        "+" => patterns.push(PatternCategory::Calculation(CalculationType::Addition)),
                        "-" => patterns.push(PatternCategory::Calculation(CalculationType::Subtraction)),

                        // Comparison operations
                        "<" | "<=" | ">" | ">=" => {
                            patterns.push(PatternCategory::Conditional(ConditionalType::ThresholdBased));
                        }
                        "==" | "===" | "!=" | "!==" => {
                            patterns.push(PatternCategory::Conditional(ConditionalType::EqualityCheck));
                        }

                        // Logical operations
                        "and" => {
                            patterns.push(PatternCategory::Conditional(ConditionalType::MultiCondition));
                        }
                        "or" => {
                            patterns.push(PatternCategory::Conditional(ConditionalType::BooleanLogic));
                        }

                        // Conditional
                        "if" => {
                            if let serde_json::Value::Array(arr) = args {
                                if arr.len() > 3 {
                                    patterns.push(PatternCategory::Conditional(ConditionalType::Tiered));
                                } else {
                                    patterns.push(PatternCategory::Conditional(ConditionalType::ThresholdBased));
                                }
                            }
                        }

                        // Variable access
                        "var" => {
                            if let serde_json::Value::String(path) = args {
                                if path.contains('.') {
                                    patterns.push(PatternCategory::Lookup(LookupType::NestedAccess));
                                } else {
                                    patterns.push(PatternCategory::Lookup(LookupType::VariableAccess));
                                }
                            }
                        }

                        // Aggregation
                        "min" => patterns.push(PatternCategory::Aggregation(AggregationType::Min)),
                        "max" => patterns.push(PatternCategory::Aggregation(AggregationType::Max)),
                        "reduce" => patterns.push(PatternCategory::Aggregation(AggregationType::Sum)),
                        "all" => patterns.push(PatternCategory::Aggregation(AggregationType::All)),
                        "some" => patterns.push(PatternCategory::Aggregation(AggregationType::Some)),
                        "none" => patterns.push(PatternCategory::Aggregation(AggregationType::None)),
                        "filter" | "map" => {
                            patterns.push(PatternCategory::Transformation(TransformationType::Mapping));
                        }

                        // String operations
                        "cat" => {
                            patterns.push(PatternCategory::Transformation(TransformationType::Concatenation));
                        }
                        "substr" => {
                            patterns.push(PatternCategory::Transformation(TransformationType::Formatting));
                        }

                        // Membership
                        "in" => {
                            patterns.push(PatternCategory::Validation(ValidationType::BusinessRule));
                        }

                        _ => {}
                    }

                    // Recurse into args
                    self.detect_patterns_recursive(args, patterns);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.detect_patterns_recursive(item, patterns);
                }
            }
            _ => {}
        }
    }

    fn is_percentage_pattern(&self, args: &serde_json::Value) -> bool {
        if let serde_json::Value::Array(arr) = args {
            for item in arr {
                if let serde_json::Value::Number(n) = item {
                    if let Some(f) = n.as_f64() {
                        if (0.0..1.0).contains(&f) && f != 0.0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Extract semantic tags from rule name and output
    fn extract_semantic_tags(&self, rule_id: &str, output_attr: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let combined = format!("{} {}", rule_id, output_attr).to_lowercase();

        // Check each keyword category
        for &keyword in &self.keywords.pricing {
            if combined.contains(keyword) {
                tags.push("pricing".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.risk {
            if combined.contains(keyword) {
                tags.push("risk-assessment".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.customer {
            if combined.contains(keyword) {
                tags.push("customer-related".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.discount {
            if combined.contains(keyword) {
                tags.push("discount-promotion".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.validation {
            if combined.contains(keyword) {
                tags.push("validation".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.eligibility {
            if combined.contains(keyword) {
                tags.push("eligibility".to_string());
                break;
            }
        }
        for &keyword in &self.keywords.coverage {
            if combined.contains(keyword) {
                tags.push("coverage".to_string());
                break;
            }
        }

        // Also extract from snake_case/camelCase parts
        let parts: Vec<&str> = rule_id
            .split(|c: char| c == '_' || c == '-' || c.is_uppercase())
            .filter(|s| !s.is_empty() && s.len() > 2)
            .collect();

        for part in parts {
            let part_lower = part.to_lowercase();
            if !tags.contains(&part_lower) && part_lower.len() > 3 {
                tags.push(part_lower);
            }
        }

        tags.truncate(5); // Limit to 5 tags
        tags
    }

    /// Analyze expression complexity
    fn analyze_complexity(&self, expr: &serde_json::Value) -> ComplexityBreakdown {
        let mut breakdown = ComplexityBreakdown {
            total_nodes: 0,
            max_depth: 0,
            variable_count: 0,
            operator_count: 0,
            condition_count: 0,
            loop_count: 0,
            cyclomatic_complexity: 1, // Base complexity
        };

        self.count_complexity_recursive(expr, 0, &mut breakdown);

        breakdown
    }

    fn count_complexity_recursive(
        &self,
        expr: &serde_json::Value,
        depth: usize,
        breakdown: &mut ComplexityBreakdown,
    ) {
        breakdown.total_nodes += 1;
        breakdown.max_depth = breakdown.max_depth.max(depth);

        match expr {
            serde_json::Value::Object(map) => {
                for (op, args) in map {
                    breakdown.operator_count += 1;

                    match op.as_str() {
                        "var" => breakdown.variable_count += 1,
                        "if" | "?:" => {
                            breakdown.condition_count += 1;
                            breakdown.cyclomatic_complexity += 1;
                        }
                        "and" | "or" => {
                            breakdown.cyclomatic_complexity += 1;
                        }
                        "map" | "filter" | "reduce" | "all" | "some" | "none" => {
                            breakdown.loop_count += 1;
                            breakdown.cyclomatic_complexity += 1;
                        }
                        _ => {}
                    }

                    self.count_complexity_recursive(args, depth + 1, breakdown);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.count_complexity_recursive(item, depth + 1, breakdown);
                }
            }
            _ => {}
        }
    }

    /// Generate natural language summary
    fn generate_natural_summary(&self, rule: &Rule, patterns: &[PatternCategory]) -> String {
        let mut parts = Vec::new();

        // Describe what it computes
        parts.push(format!("Computes '{}'", rule.output_attribute));

        // Describe inputs
        if !rule.dependencies.is_empty() {
            let deps: Vec<String> = rule.dependencies.iter().take(3).cloned().collect();
            if rule.dependencies.len() > 3 {
                parts.push(format!("using {} and {} more inputs", deps.join(", "), rule.dependencies.len() - 3));
            } else {
                parts.push(format!("using {}", deps.join(", ")));
            }
        }

        // Describe detected patterns
        let pattern_desc = self.describe_patterns(patterns);
        if !pattern_desc.is_empty() {
            parts.push(format!("via {}", pattern_desc));
        }

        parts.join(" ")
    }

    fn describe_patterns(&self, patterns: &[PatternCategory]) -> String {
        let mut descs = Vec::new();

        for pattern in patterns.iter().take(2) {
            let desc = match pattern {
                PatternCategory::Calculation(calc) => match calc {
                    CalculationType::Multiplication => "multiplication",
                    CalculationType::Division => "division",
                    CalculationType::Addition => "addition",
                    CalculationType::Percentage => "percentage calculation",
                    CalculationType::Compound => "compound calculation",
                    _ => continue,
                },
                PatternCategory::Conditional(cond) => match cond {
                    ConditionalType::ThresholdBased => "threshold comparison",
                    ConditionalType::Tiered => "tiered logic",
                    ConditionalType::MultiCondition => "multiple conditions",
                    _ => continue,
                },
                PatternCategory::Aggregation(agg) => match agg {
                    AggregationType::Sum => "summation",
                    AggregationType::Min => "minimum selection",
                    AggregationType::Max => "maximum selection",
                    _ => continue,
                },
                _ => continue,
            };
            descs.push(desc);
        }

        descs.join(" and ")
    }

    /// Generate optimization suggestions
    fn generate_suggestions(
        &self,
        rule: &Rule,
        patterns: &[PatternCategory],
        complexity: &ComplexityBreakdown,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Complexity suggestions
        if complexity.cyclomatic_complexity > 10 {
            suggestions.push(Suggestion {
                priority: SuggestionPriority::High,
                category: SuggestionCategory::Maintainability,
                title: "High complexity detected".to_string(),
                description: format!(
                    "This rule has cyclomatic complexity of {}. Consider breaking it into smaller rules.",
                    complexity.cyclomatic_complexity
                ),
                estimated_impact: "Improves maintainability and testability".to_string(),
            });
        }

        // Deep nesting
        if complexity.max_depth > 5 {
            suggestions.push(Suggestion {
                priority: SuggestionPriority::Medium,
                category: SuggestionCategory::Simplification,
                title: "Deep nesting".to_string(),
                description: format!(
                    "Expression has {} levels of nesting. Flatten where possible.",
                    complexity.max_depth
                ),
                estimated_impact: "Easier to understand and debug".to_string(),
            });
        }

        // Many dependencies
        if rule.dependencies.len() > 10 {
            suggestions.push(Suggestion {
                priority: SuggestionPriority::Medium,
                category: SuggestionCategory::Maintainability,
                title: "Many dependencies".to_string(),
                description: format!(
                    "This rule depends on {} other values. Consider intermediate calculations.",
                    rule.dependencies.len()
                ),
                estimated_impact: "Reduces coupling and improves testability".to_string(),
            });
        }

        // Tiered logic that could be a lookup table
        if patterns.contains(&PatternCategory::Conditional(ConditionalType::Tiered)) {
            if complexity.condition_count > 5 {
                suggestions.push(Suggestion {
                    priority: SuggestionPriority::Low,
                    category: SuggestionCategory::Performance,
                    title: "Consider lookup table".to_string(),
                    description: "Multiple tiers could be replaced with a lookup table for clarity.".to_string(),
                    estimated_impact: "Cleaner configuration, easier updates".to_string(),
                });
            }
        }

        suggestions
    }

    /// Generate interesting facts about a rule
    fn generate_rule_facts(
        &self,
        rule: &Rule,
        patterns: &[PatternCategory],
        complexity: &ComplexityBreakdown,
    ) -> Vec<String> {
        let mut facts = Vec::new();

        if complexity.total_nodes == 1 {
            facts.push("Simplest possible rule - single value or variable".to_string());
        }

        if complexity.variable_count == 0 {
            facts.push("Constant rule - returns the same value every time".to_string());
        }

        if complexity.loop_count > 0 {
            facts.push(format!("Processes arrays with {} iteration(s)", complexity.loop_count));
        }

        if patterns.contains(&PatternCategory::Calculation(CalculationType::Percentage)) {
            facts.push("Applies percentage-based calculation".to_string());
        }

        if patterns.contains(&PatternCategory::Conditional(ConditionalType::Tiered)) {
            facts.push(format!("Uses tiered logic with {} conditions", complexity.condition_count));
        }

        if rule.dependencies.is_empty() {
            facts.push("Input rule - no dependencies on other rules".to_string());
        }

        facts
    }

    /// Detect semantic groups from rules
    fn detect_semantic_groups(&self, rules: &[Rule]) -> Vec<SemanticGroup> {
        let mut groups: HashMap<String, SemanticGroup> = HashMap::new();

        // Group by keyword detection
        for rule in rules {
            let rule_lower = format!("{} {}", rule.id, rule.output_attribute).to_lowercase();

            // Pricing group
            if self.keywords.pricing.iter().any(|k| rule_lower.contains(k)) {
                let group = groups.entry("Pricing & Calculations".to_string()).or_insert_with(|| {
                    SemanticGroup {
                        name: "Pricing & Calculations".to_string(),
                        description: "Rules that compute prices, premiums, rates, and financial amounts".to_string(),
                        keywords: self.keywords.pricing.iter().map(|s| s.to_string()).collect(),
                        confidence: 0.0,
                        rule_ids: vec![],
                        attribute_paths: vec![],
                        patterns: vec![],
                        metrics: GroupMetrics::default(),
                    }
                });
                group.rule_ids.push(rule.id.clone());
                group.attribute_paths.push(rule.output_attribute.clone());
            }

            // Risk group
            if self.keywords.risk.iter().any(|k| rule_lower.contains(k)) {
                let group = groups.entry("Risk Assessment".to_string()).or_insert_with(|| {
                    SemanticGroup {
                        name: "Risk Assessment".to_string(),
                        description: "Rules that evaluate risk scores, factors, and ratings".to_string(),
                        keywords: self.keywords.risk.iter().map(|s| s.to_string()).collect(),
                        confidence: 0.0,
                        rule_ids: vec![],
                        attribute_paths: vec![],
                        patterns: vec![],
                        metrics: GroupMetrics::default(),
                    }
                });
                group.rule_ids.push(rule.id.clone());
                group.attribute_paths.push(rule.output_attribute.clone());
            }

            // Discount group
            if self.keywords.discount.iter().any(|k| rule_lower.contains(k)) {
                let group = groups.entry("Discounts & Promotions".to_string()).or_insert_with(|| {
                    SemanticGroup {
                        name: "Discounts & Promotions".to_string(),
                        description: "Rules that apply discounts, rebates, and promotional offers".to_string(),
                        keywords: self.keywords.discount.iter().map(|s| s.to_string()).collect(),
                        confidence: 0.0,
                        rule_ids: vec![],
                        attribute_paths: vec![],
                        patterns: vec![],
                        metrics: GroupMetrics::default(),
                    }
                });
                group.rule_ids.push(rule.id.clone());
                group.attribute_paths.push(rule.output_attribute.clone());
            }

            // Eligibility group
            if self.keywords.eligibility.iter().any(|k| rule_lower.contains(k)) {
                let group = groups.entry("Eligibility & Validation".to_string()).or_insert_with(|| {
                    SemanticGroup {
                        name: "Eligibility & Validation".to_string(),
                        description: "Rules that determine eligibility, validation, and approval".to_string(),
                        keywords: self.keywords.eligibility.iter().map(|s| s.to_string()).collect(),
                        confidence: 0.0,
                        rule_ids: vec![],
                        attribute_paths: vec![],
                        patterns: vec![],
                        metrics: GroupMetrics::default(),
                    }
                });
                group.rule_ids.push(rule.id.clone());
                group.attribute_paths.push(rule.output_attribute.clone());
            }

            // Coverage group
            if self.keywords.coverage.iter().any(|k| rule_lower.contains(k)) {
                let group = groups.entry("Coverage & Benefits".to_string()).or_insert_with(|| {
                    SemanticGroup {
                        name: "Coverage & Benefits".to_string(),
                        description: "Rules related to insurance coverage, benefits, and policy details".to_string(),
                        keywords: self.keywords.coverage.iter().map(|s| s.to_string()).collect(),
                        confidence: 0.0,
                        rule_ids: vec![],
                        attribute_paths: vec![],
                        patterns: vec![],
                        metrics: GroupMetrics::default(),
                    }
                });
                group.rule_ids.push(rule.id.clone());
                group.attribute_paths.push(rule.output_attribute.clone());
            }
        }

        // Calculate confidence scores and dedupe rule_ids
        for group in groups.values_mut() {
            group.rule_ids.sort();
            group.rule_ids.dedup();
            group.attribute_paths.sort();
            group.attribute_paths.dedup();

            // Confidence based on number of matching rules
            group.confidence = (group.rule_ids.len() as f64 / rules.len() as f64).min(1.0);
            group.metrics.rule_count = group.rule_ids.len();
            group.metrics.attribute_count = group.attribute_paths.len();
        }

        groups.into_values().collect()
    }

    /// Find similar rules based on patterns
    fn find_similar_rules(&self, result: &mut AnalysisResult, _rules: &[Rule]) {
        let rule_patterns: HashMap<String, Vec<PatternCategory>> = result
            .rule_insights
            .iter()
            .map(|(id, insight)| (id.clone(), insight.patterns.clone()))
            .collect();

        for (rule_id, insight) in result.rule_insights.iter_mut() {
            let my_patterns = &insight.patterns;

            for (other_id, other_patterns) in &rule_patterns {
                if rule_id == other_id {
                    continue;
                }

                let matching: Vec<PatternCategory> = my_patterns
                    .iter()
                    .filter(|p| other_patterns.contains(p))
                    .cloned()
                    .collect();

                if !matching.is_empty() {
                    let score = matching.len() as f64 / my_patterns.len().max(1) as f64;
                    if score >= 0.5 {
                        insight.similar_rules.push(SimilarityMatch {
                            rule_id: other_id.clone(),
                            similarity_score: score,
                            matching_patterns: matching.clone(),
                            reason: format!(
                                "Both use {} pattern(s)",
                                matching.len()
                            ),
                        });
                    }
                }
            }

            // Sort by similarity score
            insight.similar_rules.sort_by(|a, b| {
                b.similarity_score.partial_cmp(&a.similarity_score).unwrap()
            });
            insight.similar_rules.truncate(5);
        }
    }

    /// Compute global metrics
    fn compute_global_metrics(&self, rules: &[Rule], result: &AnalysisResult) -> GlobalMetrics {
        let mut metrics = GlobalMetrics::default();

        metrics.total_rules = rules.len();

        // Pattern distribution
        for insight in result.rule_insights.values() {
            for pattern in &insight.patterns {
                *metrics.pattern_distribution.entry(format!("{:?}", pattern)).or_insert(0) += 1;
            }

            metrics.avg_complexity += insight.complexity.cyclomatic_complexity as f64;
            metrics.max_complexity = metrics.max_complexity.max(insight.complexity.cyclomatic_complexity);
            metrics.total_variables += insight.complexity.variable_count;
        }

        if !rules.is_empty() {
            metrics.avg_complexity /= rules.len() as f64;
        }

        // Dependency analysis
        let mut dependency_counts: Vec<usize> = rules.iter().map(|r| r.dependencies.len()).collect();
        dependency_counts.sort();
        if !dependency_counts.is_empty() {
            metrics.avg_dependencies = dependency_counts.iter().sum::<usize>() as f64 / dependency_counts.len() as f64;
            metrics.max_dependencies = *dependency_counts.last().unwrap_or(&0);
        }

        // Group metrics
        metrics.semantic_group_count = result.semantic_groups.len();

        metrics
    }

    /// Generate interesting facts about the entire ruleset
    fn generate_facts(&self, result: &AnalysisResult, rules: &[Rule]) -> Vec<String> {
        let mut facts = Vec::new();

        // Total rules
        facts.push(format!("Total of {} rules analyzed", rules.len()));

        // Most common pattern
        if let Some((pattern, count)) = result.global_metrics.pattern_distribution.iter().max_by_key(|(_, c)| *c) {
            facts.push(format!("Most common pattern: {} ({}x)", pattern, count));
        }

        // Complexity distribution
        let simple_count = result.rule_insights.values().filter(|i| i.complexity.cyclomatic_complexity <= 2).count();
        let complex_count = result.rule_insights.values().filter(|i| i.complexity.cyclomatic_complexity > 5).count();
        if simple_count > 0 {
            facts.push(format!("{} rules are simple (complexity ≤ 2)", simple_count));
        }
        if complex_count > 0 {
            facts.push(format!("{} rules are complex (complexity > 5)", complex_count));
        }

        // Semantic groups
        for group in &result.semantic_groups {
            if group.rule_ids.len() >= 3 {
                facts.push(format!(
                    "'{}' group contains {} related rules",
                    group.name, group.rule_ids.len()
                ));
            }
        }

        // Dependency insights
        let independent = rules.iter().filter(|r| r.dependencies.is_empty()).count();
        if independent > 0 {
            facts.push(format!("{} rules are input rules (no dependencies)", independent));
        }

        let highly_connected = rules.iter().filter(|r| r.dependencies.len() > 5).count();
        if highly_connected > 0 {
            facts.push(format!("{} rules have 5+ dependencies (potential bottlenecks)", highly_connected));
        }

        facts
    }
}

/// Complete analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisResult {
    /// Per-rule insights
    pub rule_insights: HashMap<String, RuleInsight>,
    /// Detected semantic groups
    pub semantic_groups: Vec<SemanticGroup>,
    /// Global metrics
    pub global_metrics: GlobalMetrics,
    /// Interesting facts about the ruleset
    pub interesting_facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalMetrics {
    pub total_rules: usize,
    pub avg_complexity: f64,
    pub max_complexity: usize,
    pub total_variables: usize,
    pub avg_dependencies: f64,
    pub max_dependencies: usize,
    pub semantic_group_count: usize,
    pub pattern_distribution: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_multiplication_pattern() {
        let analyzer = PatternAnalyzer::new();
        let expr = serde_json::json!({
            "*": [{"var": "base"}, {"var": "factor"}]
        });

        let patterns = analyzer.detect_patterns_from_expression(&expr);
        assert!(patterns.contains(&PatternCategory::Calculation(CalculationType::Multiplication)));
    }

    #[test]
    fn test_detect_percentage_pattern() {
        let analyzer = PatternAnalyzer::new();
        let expr = serde_json::json!({
            "*": [{"var": "price"}, 0.15]
        });

        let patterns = analyzer.detect_patterns_from_expression(&expr);
        assert!(patterns.contains(&PatternCategory::Calculation(CalculationType::Percentage)));
    }

    #[test]
    fn test_detect_tiered_pattern() {
        let analyzer = PatternAnalyzer::new();
        let expr = serde_json::json!({
            "if": [
                {"<": [{"var": "age"}, 25]}, 1.5,
                {"<": [{"var": "age"}, 35]}, 1.2,
                {"<": [{"var": "age"}, 50]}, 1.0,
                0.9
            ]
        });

        let patterns = analyzer.detect_patterns_from_expression(&expr);
        assert!(patterns.contains(&PatternCategory::Conditional(ConditionalType::Tiered)));
    }

    #[test]
    fn test_semantic_tags() {
        let analyzer = PatternAnalyzer::new();
        let tags = analyzer.extract_semantic_tags("calculate_premium", "base_premium");
        assert!(tags.contains(&"pricing".to_string()));
    }

    #[test]
    fn test_complexity_analysis() {
        let analyzer = PatternAnalyzer::new();
        let expr = serde_json::json!({
            "if": [
                {"<": [{"var": "x"}, 10]},
                {"*": [{"var": "x"}, 2]},
                {"var": "x"}
            ]
        });

        let complexity = analyzer.analyze_complexity(&expr);
        assert!(complexity.condition_count >= 1);
        assert!(complexity.variable_count >= 2);
    }
}
