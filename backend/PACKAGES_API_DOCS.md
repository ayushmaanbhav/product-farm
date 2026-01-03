# Product-FARM Backend Packages API Documentation

## Overview

The backend consists of three core Rust crates that form a layered rule evaluation system:

```
┌─────────────────────────────────────────────────────────────┐
│                      rule-engine                            │
│         DAG-based execution, context management             │
├─────────────────────────────────────────────────────────────┤
│                      json-logic                             │
│      Parser → AST → Bytecode Compiler → VM Execution        │
├─────────────────────────────────────────────────────────────┤
│                         core                                │
│    Product, Rule, Attribute, Value, Path types              │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Core Package (`product-farm-core`)

**Location:** `/crates/core/`
**Purpose:** Domain types, value system, validation, and path abstractions

### Key Types

#### Value (Runtime Polymorphic Type)
```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Decimal(Decimal),          // rust_decimal for precision
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

// Key methods:
value.is_truthy() -> bool              // JavaScript-style truthiness
value.to_number() -> f64               // Coerce to number
value.loose_equals(&other) -> bool     // JS-style == comparison
value.to_json() -> JsonValue           // Convert to serde_json
Value::from_json(&JsonValue) -> Self   // Parse from JSON
```

#### Product (Root Entity)
```rust
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub status: ProductStatus,           // Draft → PendingApproval → Active → Discontinued
    pub template_type: TemplateType,
    pub parent_product_id: Option<ProductId>,
    pub effective_from: DateTime<Utc>,
    pub expiry_at: Option<DateTime<Utc>>,
    // ... timestamps, version
}

// Lifecycle methods:
product.submit() -> CoreResult<()>      // Draft → PendingApproval
product.approve() -> CoreResult<()>     // PendingApproval → Active
product.reject() -> CoreResult<()>      // → Draft
product.discontinue() -> CoreResult<()> // → Discontinued
product.is_editable() -> bool           // Only in Draft
```

#### Rule (JSON Logic Expression Container)
```rust
pub struct Rule {
    pub id: RuleId,
    pub product_id: ProductId,
    pub rule_type: String,               // "premium-calculation", "entry-logic", etc.
    pub input_attributes: Vec<RuleInputAttribute>,
    pub output_attributes: Vec<RuleOutputAttribute>,
    pub compiled_expression: String,     // JSON Logic as JSON string
    pub display_expression: String,
    pub enabled: bool,
    pub order_index: i32,
}

// Factory methods:
Rule::from_json_logic(product_id, rule_type, json!({...})) -> Self
rule.with_inputs(vec![...]) -> Self
rule.with_outputs(vec![...]) -> Self
rule.get_expression() -> CoreResult<JsonValue>
```

#### RuleBuilder (Fluent API)
```rust
RuleBuilder::new(product_id, rule_type)
    .input("path:to:input")
    .output("path:to:output")
    .expression(json!({"*": [{"var": "x"}, 2]}))
    .description("Doubles the input")
    .order(1)
    .enabled(true)
    .build() -> CoreResult<Rule>
```

#### Attribute (Concrete Value Instance)
```rust
pub struct Attribute {
    pub path: ConcretePath,
    pub abstract_path: AbstractPath,
    pub product_id: ProductId,
    pub value_type: AttributeValueType,  // FixedValue | RuleDriven | JustDefinition
    pub value: Option<Value>,            // Set if FixedValue
    pub rule_id: Option<RuleId>,         // Set if RuleDriven
}

// Constructors:
Attribute::new_fixed_value(path, abstract_path, product_id, value) -> Self
Attribute::new_rule_driven(path, abstract_path, product_id, rule_id) -> Self
Attribute::new_just_definition(path, abstract_path, product_id) -> Self
```

#### Path Types
```rust
// Abstract (template) path format:
// {productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}
AbstractPath::build(product_id, component_type, component_id, attr_name) -> Self

// Concrete (instance) path format:
// {productId}:{componentType}:{componentId}:{attributeName}
ConcretePath::build(product_id, component_type, component_id, attr_name) -> Self
concrete_path.to_abstract_path() -> AbstractPath
```

#### Error Types
```rust
pub enum CoreError {
    ProductNotFound(ProductId),
    RuleNotFound(RuleId),
    AttributeNotFound(AttributeId),
    InvalidStateTransition { from, to },
    ValidationError(String),
    ValidationFailed { field, message },
    CyclicDependency(String),
    TypeMismatch { expected, actual },
    // ...
}

pub type CoreResult<T> = Result<T, CoreError>;
```

---

## 2. JSON Logic Package (`product-farm-json-logic`)

**Location:** `/crates/json-logic/`
**Purpose:** Parse, compile, and evaluate JSON Logic expressions with tiered optimization

### Architecture

```
JSON Input → Parser → AST (Expr) → Compiler → Bytecode → VM Execution
                         │                                    │
                         └──── Direct AST Evaluation ─────────┘
                              (Tier 0, < 5 nodes)
```

### Supported Operations

| Category | Operations |
|----------|------------|
| **Arithmetic** | `+`, `-`, `*`, `/`, `%`, `min`, `max` |
| **Comparison** | `==`, `!=`, `<`, `<=`, `>`, `>=`, `===`, `!==` |
| **Logic** | `and`, `or`, `!`, `!!`, `if`, `?:` |
| **Array** | `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`, `in` |
| **String** | `cat`, `substr`, `in` |
| **Data** | `var`, `missing`, `missing_some`, `log` |

### Key Types

#### Expression AST
```rust
pub enum Expr {
    Literal(Value),
    Var(VarExpr),                              // {"var": "path.to.value"}

    // Comparison
    Eq(Box<Expr>, Box<Expr>),                  // {"==": [a, b]}
    Lt(Vec<Expr>),                             // {"<": [a, b]} or chain
    // ...

    // Logical
    And(Vec<Expr>),                            // {"and": [...]}
    Or(Vec<Expr>),                             // {"or": [...]}
    If(Vec<Expr>),                             // {"if": [cond, then, else...]}

    // Arithmetic
    Add(Vec<Expr>),                            // {"+": [a, b, c]}
    Mul(Vec<Expr>),                            // {"*": [...]}
    // ...

    // Array operations
    Map(Box<Expr>, Box<Expr>),                 // {"map": [array, expr]}
    Filter(Box<Expr>, Box<Expr>),              // {"filter": [array, expr]}
    Reduce(Box<Expr>, Box<Expr>, Box<Expr>),   // {"reduce": [array, expr, init]}
}

pub struct VarExpr {
    pub path: String,                          // "user.age" or "" for root
    pub default: Option<Value>,                // Default if path missing
}
```

#### Parser
```rust
pub fn parse(json: &JsonValue) -> JsonLogicResult<Expr>

// Example:
let expr = parse(&json!({"*": [{"var": "x"}, 2]}))?;
```

#### Evaluator (Main Entry Point)
```rust
pub struct Evaluator {
    vm: VM,
}

impl Evaluator {
    pub fn new() -> Self

    // One-shot evaluation
    pub fn evaluate(&mut self, rule: &JsonValue, data: &JsonValue)
        -> JsonLogicResult<Value>

    // Cached evaluation (faster for repeated calls)
    pub fn evaluate_cached(&mut self, cached: &CachedExpression, data: &JsonValue)
        -> JsonLogicResult<Value>
}

// Example:
let mut evaluator = Evaluator::new();
let result = evaluator.evaluate(
    &json!({"if": [{">": [{"var": "age"}, 60]}, 1.2, 1.0]}),
    &json!({"age": 65})
)?;
// result = Value::Float(1.2)
```

#### Cached Expression
```rust
pub struct CachedExpression {
    pub ast: Arc<Expr>,
    pub bytecode: Option<Arc<CompiledBytecode>>,
    pub variables: Vec<String>,              // Variables referenced
    pub node_count: usize,                   // Complexity metric
}

impl CachedExpression {
    pub fn from_json(json: &JsonValue) -> JsonLogicResult<Self>
    pub fn compile_bytecode(&mut self) -> JsonLogicResult<()>
    pub fn has_bytecode(&self) -> bool
}
```

#### Tiered Execution
```rust
pub enum CompilationTier {
    Ast,       // Tier 0: Direct AST interpretation (default)
    Bytecode,  // Tier 1: Stack-based bytecode VM (3.5x faster)
    Jit,       // Tier 2: Future JIT compilation
}

// Auto-promotion thresholds:
const BYTECODE_COMPILE_THRESHOLD: usize = 5;   // Compile if 5+ AST nodes
const PROMOTION_THRESHOLD: u64 = 100;          // Promote after 100 evals
```

#### Bytecode VM
```rust
pub struct VM {
    stack: SmallVec<[Value; 32]>,             // Execution stack
}

impl VM {
    pub fn new() -> Self
    pub fn execute(&mut self, bytecode: &CompiledBytecode, context: &EvalContext)
        -> JsonLogicResult<Value>
}

// Stack-based opcodes: LoadConst, LoadVar, Add, Mul, Eq, Lt, Jump, etc.
```

### Performance

| Mode | Time | Notes |
|------|------|-------|
| AST Interpretation | ~1.15µs | Default for small expressions |
| Bytecode VM | ~330ns | 3.5x faster, auto-promoted |

---

## 3. Rule Engine Package (`product-farm-rule-engine`)

**Location:** `/crates/rule-engine/`
**Purpose:** DAG-based rule execution with dependency resolution and parallel execution

### Architecture

```
Rules[] → DAG Builder → Topological Sort → Execution Levels → Sequential/Parallel Execution
              │                                    │
              ├── Cycle Detection                  └── Level N rules can run in parallel
              └── Dependency Resolution
```

### Key Types

#### Execution Context
```rust
pub struct ExecutionContext {
    input: Arc<HashMap<String, Value>>,      // Immutable input data
    computed: HashMap<String, Value>,         // Rule outputs (mutable)
    metadata: HashMap<String, Value>,         // Execution metadata
}

impl ExecutionContext {
    pub fn new(input: HashMap<String, Value>) -> Self
    pub fn from_json(json: &JsonValue) -> Self

    // Access
    pub fn get(&self, key: &str) -> Option<&Value>
    pub fn get_path(&self, path: &str) -> Option<Value>   // Dot-notation: "user.age"
    pub fn contains(&self, key: &str) -> bool

    // Mutation
    pub fn set(&mut self, key: impl Into<String>, value: Value)
    pub fn merge(&mut self, other: &ExecutionContext)

    // Export
    pub fn to_json(&self) -> JsonValue                    // Nested JSON output
    pub fn computed_values(&self) -> &HashMap<String, Value>
}
```

#### Rule DAG
```rust
pub struct RuleDag {
    graph: DiGraph<RuleNode, ()>,              // petgraph backing
    rule_to_node: HashMap<RuleId, NodeIndex>,
    output_to_rule: HashMap<String, RuleId>,   // Which rule produces which output
}

pub struct RuleNode {
    pub id: RuleId,
    pub inputs: Vec<String>,                   // Input attribute paths
    pub outputs: Vec<String>,                  // Output attribute paths
    pub order_index: i32,
}

impl RuleDag {
    pub fn from_rules(rules: &[Rule]) -> RuleEngineResult<Self>

    // Execution planning
    pub fn topological_order(&self) -> RuleEngineResult<Vec<RuleId>>
    pub fn execution_levels(&self) -> RuleEngineResult<Vec<Vec<RuleId>>>

    // Dependency queries
    pub fn dependencies(&self, id: &RuleId) -> Vec<RuleId>
    pub fn dependents(&self, id: &RuleId) -> Vec<RuleId>

    // Validation
    pub fn find_missing_inputs(&self, available: &HashSet<String>) -> Vec<(RuleId, String)>

    // Visualization
    pub fn to_dot(&self) -> String             // Graphviz format
    pub fn to_mermaid(&self) -> String         // Markdown diagram
    pub fn to_ascii(&self) -> String           // Terminal format
}
```

#### Execution Plan
```rust
pub struct ExecutionPlan {
    pub total_rules: usize,
    pub total_stages: usize,
    pub stages: Vec<ExecutionStage>,
}

pub struct ExecutionStage {
    pub level: usize,
    pub parallel: bool,                        // True if rules can run concurrently
    pub rules: Vec<ExecutionPlanRule>,
}

pub struct ExecutionPlanRule {
    pub id: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,
}
```

#### Rule Executor
```rust
pub struct RuleExecutor {
    evaluator: Evaluator,                      // JSON Logic evaluator
    compiled_rules: HashMap<RuleId, CompiledRule>,
}

pub struct CompiledRule {
    pub rule: Arc<Rule>,
    pub expression: Arc<CachedExpression>,
}

impl RuleExecutor {
    pub fn new() -> Self
    pub fn compile_rules(&mut self, rules: &[Rule]) -> RuleEngineResult<()>

    pub fn execute(&mut self, rules: &[Rule], context: &mut ExecutionContext)
        -> RuleEngineResult<ExecutionResult>

    pub fn stats(&self) -> ExecutorStats
    pub fn clear_cache(&mut self)
}
```

#### Execution Result
```rust
pub struct ExecutionResult {
    pub rule_results: Vec<RuleResult>,
    pub context: ExecutionContext,             // Final context with all computed values
    pub total_time_ns: u64,
    pub levels: Vec<Vec<RuleId>>,              // Execution levels for debugging
}

pub struct RuleResult {
    pub rule_id: RuleId,
    pub outputs: Vec<(String, Value)>,         // (output_path, computed_value)
    pub execution_time_ns: u64,
}

impl ExecutionResult {
    pub fn get_result(&self, rule_id: &RuleId) -> Option<&RuleResult>
    pub fn get_output(&self, output: &str) -> Option<&Value>
}
```

#### Executor Stats
```rust
pub struct ExecutorStats {
    pub compiled_rules: usize,
    pub total_ast_nodes: usize,
    pub rules_with_bytecode: usize,
}
```

### Execution Flow

```rust
// 1. Create executor
let mut executor = RuleExecutor::new();

// 2. Prepare context with input data
let mut context = ExecutionContext::from_json(&json!({
    "base_rate": 100.0,
    "age": 45,
    "risk_factor": 1.2
}));

// 3. Define rules
let rules = vec![
    Rule::from_json_logic("prod1", "calc-premium",
        json!({"*": [{"var": "base_rate"}, {"var": "age_factor"}]}))
        .with_inputs(vec!["base_rate", "age_factor"])
        .with_outputs(vec!["premium"]),
    // ...
];

// 4. Execute (builds DAG, resolves dependencies, runs in order)
let result = executor.execute(&rules, &mut context)?;

// 5. Access results
println!("Premium: {:?}", result.get_output("premium"));
println!("Execution time: {}ns", result.total_time_ns);
for rule_result in &result.rule_results {
    println!("Rule {} took {}ns", rule_result.rule_id, rule_result.execution_time_ns);
}
```

### Error Types
```rust
pub enum RuleEngineError {
    CyclicDependency(String),
    MissingDependency { rule: String, dependency: String },
    RuleNotFound(String),
    EvaluationError(String),
    InvalidConfiguration(String),
    JsonLogicError(JsonLogicError),
    ContextError(String),
}

pub type RuleEngineResult<T> = Result<T, RuleEngineError>;
```

---

## Usage Examples

### Example 1: Simple Calculation
```rust
use product_farm_json_logic::Evaluator;
use serde_json::json;

let mut evaluator = Evaluator::new();

// Calculate: x * 2 + 10
let result = evaluator.evaluate(
    &json!({"+": [{"*": [{"var": "x"}, 2]}, 10]}),
    &json!({"x": 5})
)?;
// result = Value::Int(20)
```

### Example 2: Conditional Logic
```rust
// Age-based pricing
let result = evaluator.evaluate(
    &json!({"if": [
        {">": [{"var": "age"}, 60]}, 1.5,
        {">": [{"var": "age"}, 40]}, 1.2,
        1.0
    ]}),
    &json!({"age": 45})
)?;
// result = Value::Float(1.2)
```

### Example 3: Multi-Rule DAG Execution
```rust
use product_farm_rule_engine::{RuleExecutor, ExecutionContext};
use product_farm_core::Rule;

// Rule 1: base_premium = rate * coverage
let rule1 = Rule::from_json_logic("ins", "base-premium",
    json!({"*": [{"var": "rate"}, {"var": "coverage"}]}))
    .with_inputs(vec!["rate", "coverage"])
    .with_outputs(vec!["base_premium"]);

// Rule 2: age_factor = if age > 60 then 1.5 else 1.0
let rule2 = Rule::from_json_logic("ins", "age-factor",
    json!({"if": [{">": [{"var": "age"}, 60]}, 1.5, 1.0]}))
    .with_inputs(vec!["age"])
    .with_outputs(vec!["age_factor"]);

// Rule 3: final_premium = base_premium * age_factor (depends on rules 1 & 2)
let rule3 = Rule::from_json_logic("ins", "final-premium",
    json!({"*": [{"var": "base_premium"}, {"var": "age_factor"}]}))
    .with_inputs(vec!["base_premium", "age_factor"])
    .with_outputs(vec!["final_premium"]);

let mut executor = RuleExecutor::new();
let mut context = ExecutionContext::from_json(&json!({
    "rate": 0.05,
    "coverage": 100000,
    "age": 65
}));

let result = executor.execute(&vec![rule1, rule2, rule3], &mut context)?;

// Execution order: [rule1, rule2] (parallel, level 0) → [rule3] (level 1)
// final_premium = 100000 * 0.05 * 1.5 = 7500
```

---

## Performance Characteristics

| Operation | Typical Time |
|-----------|-------------|
| JSON Logic parse | ~500ns |
| AST evaluation (simple) | ~1.15µs |
| Bytecode evaluation | ~330ns |
| 10-rule chain execution | ~19µs |
| 50-rule chain execution | ~347µs |
| DAG construction (10 rules) | ~2µs |

---

## Dependencies

| Crate | External Dependencies |
|-------|----------------------|
| **core** | serde, chrono, uuid, rust_decimal, thiserror, regex |
| **json-logic** | serde_json, smallvec, hashbrown |
| **rule-engine** | petgraph, tokio, tracing |

---

## File Locations

| Component | Path |
|-----------|------|
| Core types | `crates/core/src/` |
| JSON Logic | `crates/json-logic/src/` |
| Rule Engine | `crates/rule-engine/src/` |
| API Layer | `crates/api/src/` |
| Examples | `examples/` |
