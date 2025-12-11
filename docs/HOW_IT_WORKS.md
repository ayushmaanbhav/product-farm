---
layout: default
title: How It Works
---

# How It Works

This document provides a technical deep-dive into how Product-FARM evaluates rules, from the moment a request arrives to the final result being returned.

---

## Overview: The Evaluation Pipeline

When you call the evaluation API, here's what happens under the hood:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         EVALUATION PIPELINE                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. REQUEST      2. LOAD         3. BUILD        4. EXECUTE     5. RETURN  │
│   ────────────    ────────────    ────────────    ────────────   ────────── │
│                                                                              │
│   ┌─────────┐    ┌─────────┐     ┌─────────┐     ┌─────────┐    ┌─────────┐│
│   │ Client  │───►│  Fetch  │────►│  Build  │────►│ Execute │───►│ Format  ││
│   │ Request │    │  Rules  │     │   DAG   │     │  Rules  │    │ Response││
│   └─────────┘    └─────────┘     └─────────┘     └─────────┘    └─────────┘│
│                                                                              │
│   JSON/gRPC      DGraph          Dependency      Parallel       JSON/gRPC   │
│   payload        query           analysis        execution      response    │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Request Processing

### REST API Request

```bash
POST /api/products/insurance-premium-v1/evaluate
Content-Type: application/json

{
  "inputs": {
    "customer_age": 45,
    "coverage_amount": 250000,
    "smoker_status": "NON_SMOKER"
  },
  "functionality": "quote"
}
```

### gRPC Request

```protobuf
message EvaluateRequest {
  string product_id = 1;
  string functionality = 2;
  map<string, Value> input_data = 3;
}
```

### What Happens

1. **Parse Request**: Extract product ID, functionality, and input values
2. **Validate Inputs**: Check types match expected datatypes
3. **Load Product**: Retrieve product configuration from cache or database

---

## Step 2: Rule Loading

### DGraph Query

Product-FARM uses DGraph to store and query rules efficiently:

```graphql
query GetProductRules($productId: string) {
  product(func: eq(product_id, $productId)) {
    name
    rules {
      rule_id
      name
      expression
      inputs { name datatype }
      outputs { name datatype }
    }
    attributes {
      name
      datatype
      component
    }
  }
}
```

### Caching Strategy

```
┌──────────────────────────────────────────────────────────────────┐
│                       CACHING LAYERS                             │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Request ──► LRU Cache (Hot) ──► DGraph (Cold)                   │
│                   │                   │                          │
│               Hit: ~1µs           Miss: ~1-5ms                   │
│                                                                  │
│  Cache Policy:                                                   │
│  • Product rules: 5 minute TTL                                   │
│  • Compiled bytecode: Until product changes                      │
│  • Eviction: LRU with 1000 entry limit                           │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

<div class="callout callout-performance">
<strong>Cache Impact:</strong> Cache hits deliver ~1μs latency vs ~1-5ms for database queries—a 1000-5000x improvement. For production workloads, ensure cache sizes are tuned for your working set.
</div>

---

## Step 3: DAG Construction

### Dependency Analysis

The system analyzes rule inputs and outputs to build a **Directed Acyclic Graph (DAG)**:

```rust
// Pseudocode for DAG construction
fn build_dag(rules: Vec<Rule>) -> DAG {
    let mut dag = DAG::new();

    for rule in rules {
        dag.add_node(rule.id);

        // Find rules that produce our inputs
        for input in rule.inputs {
            if let Some(producer) = find_rule_producing(input) {
                dag.add_edge(producer, rule.id);
            }
        }
    }

    dag.topological_sort()
}
```

### Example DAG

For our insurance premium calculator:

```
                    INPUT LAYER
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
   ┌───────────┐  ┌───────────┐  ┌───────────┐
   │customer_  │  │coverage_  │  │smoker_    │
   │   age     │  │  amount   │  │  status   │
   └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
         │               │               │
         │               │               │
         ▼               ▼               ▼
                    LEVEL 0 (Parallel)
                         │
   ┌─────────────────────┼─────────────────────┐
   │                     │                     │
   ▼                     ▼                     ▼
┌────────────────┐ ┌────────────────┐ ┌────────────────┐
│calculate_age_  │ │calculate_base_ │ │calculate_      │
│   factor       │ │   premium      │ │smoker_factor   │
│                │ │                │ │                │
│IN: age         │ │IN: coverage    │ │IN: smoker_     │
│OUT: age_factor │ │OUT: base_prem  │ │    status      │
└───────┬────────┘ └───────┬────────┘ │OUT: smoker_    │
        │                  │          │    factor      │
        │                  │          └───────┬────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                           ▼
                    LEVEL 1 (Sequential)
                           │
                           ▼
              ┌────────────────────────┐
              │ calculate_final_       │
              │       premium          │
              │                        │
              │ IN: base_premium,      │
              │     age_factor,        │
              │     smoker_factor      │
              │ OUT: final_premium     │
              └───────────┬────────────┘
                          │
                          ▼
                    LEVEL 2 (Sequential)
                          │
                          ▼
              ┌────────────────────────┐
              │ calculate_monthly_     │
              │       payment          │
              │                        │
              │ IN: final_premium      │
              │ OUT: monthly_payment   │
              └───────────┬────────────┘
                          │
                          ▼
                    OUTPUT LAYER
```

### Execution Levels

| Level | Rules | Dependencies | Execution |
|-------|-------|--------------|-----------|
| 0 | age_factor, base_premium, smoker_factor | Only inputs | **Parallel** |
| 1 | final_premium | Level 0 outputs | **Sequential** after Level 0 |
| 2 | monthly_payment | Level 1 outputs | **Sequential** after Level 1 |

---

## Step 4: Rule Execution

### The JSON Logic Engine

Product-FARM uses a custom JSON Logic implementation with a **tiered compilation** strategy:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    TIERED COMPILATION PIPELINE                               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   JSON Logic          Parse           Tier Check        Execute              │
│   Expression          ─────►          ─────────►        ───────►             │
│                                                                              │
│   {"*": [...]}        AST             eval_count?       Result               │
│                        │                  │                                  │
│                        │         ┌────────┴────────┐                         │
│                        │         │                 │                         │
│                        ▼         ▼                 ▼                         │
│                    ┌───────┐  < 100           >= 100                         │
│                    │  AST  │    │                 │                          │
│                    │ Tree  │    │                 │                          │
│                    └───┬───┘    │                 │                          │
│                        │        │                 │                          │
│                        │        ▼                 ▼                          │
│                        │   ┌─────────┐      ┌─────────┐                      │
│                        │   │ Tier 0  │      │Compiler │                      │
│                        │   │AST Eval │      │         │                      │
│                        │   └────┬────┘      └────┬────┘                      │
│                        │        │                │                           │
│                        │        │                ▼                           │
│                        │        │          ┌─────────┐                       │
│                        │        │          │Bytecode │                       │
│                        │        │          └────┬────┘                       │
│                        │        │                │                           │
│                        │        │                ▼                           │
│                        │        │          ┌─────────┐                       │
│                        │        │          │ Tier 1  │                       │
│                        │        │          │ VM Exec │                       │
│                        │        │          └────┬────┘                       │
│                        │        │                │                           │
│                        │        └────────┬───────┘                           │
│                        │                 │                                   │
│                        ▼                 ▼                                   │
│                             ┌─────────────────┐                              │
│                             │     Result      │                              │
│                             └─────────────────┘                              │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Tier 0: AST Interpretation

- **How it works**: Walk the AST tree, evaluate each node recursively
- **Latency**: ~1.15 microseconds per evaluation
- **Use case**: Rules evaluated < 100 times (cold start, rare rules)

```rust
fn eval_ast(node: &AstNode, context: &Context) -> Value {
    match node {
        AstNode::Var(name) => context.get(name),
        AstNode::Literal(v) => v.clone(),
        AstNode::Op { op, args } => {
            let evaluated: Vec<Value> = args.iter()
                .map(|a| eval_ast(a, context))
                .collect();
            apply_operator(op, evaluated)
        }
    }
}
```

### Tier 1: Bytecode VM

- **How it works**: Compile AST to bytecode, execute on stack-based VM
- **Latency**: ~330 nanoseconds per evaluation (3.5x faster)
- **Use case**: Hot paths, frequently evaluated rules

```rust
// Bytecode instructions
enum Instruction {
    LoadVar(String),      // Push variable onto stack
    LoadConst(Value),     // Push constant onto stack
    Add,                  // Pop two, push sum
    Multiply,             // Pop two, push product
    Compare(CmpOp),       // Pop two, push boolean
    JumpIfFalse(usize),   // Conditional jump
    Return,               // Return top of stack
}

// Compiled bytecode for: base_premium = coverage_amount * 0.02
[
    LoadVar("coverage_amount"),  // stack: [250000]
    LoadConst(0.02),             // stack: [250000, 0.02]
    Multiply,                    // stack: [5000]
    Return,                      // result: 5000
]
```

### Performance Comparison

| Metric | Tier 0 (AST) | Tier 1 (Bytecode) |
|--------|--------------|-------------------|
| Latency | ~1.15µs | ~330ns |
| Throughput | ~870K/sec | ~3M/sec |
| Warmup | None | 100 evaluations |
| Memory | Lower | Higher (bytecode cache) |

<div class="callout callout-tip">
<strong>Optimization Tip:</strong> For production deployments with predictable workloads, configure eager compilation to pre-compile all rules during product activation, ensuring hot paths are ready immediately.
</div>

---

## Step 5: Execution Context

### Variable Resolution

The execution context holds all values needed during evaluation:

```rust
struct ExecutionContext {
    // Input values provided by caller
    inputs: HashMap<String, Value>,

    // Intermediate results from rule execution
    computed: HashMap<String, Value>,

    // Type information for validation
    types: HashMap<String, DataType>,
}

impl ExecutionContext {
    fn get(&self, name: &str) -> Value {
        // 1. Check computed values first
        if let Some(v) = self.computed.get(name) {
            return v.clone();
        }
        // 2. Fall back to inputs
        if let Some(v) = self.inputs.get(name) {
            return v.clone();
        }
        // 3. Error if not found
        panic!("Variable not found: {}", name);
    }

    fn set(&mut self, name: &str, value: Value) {
        self.computed.insert(name.to_string(), value);
    }
}
```

### Type Coercion

Product-FARM handles type coercion automatically:

| Input Type | Target Type | Coercion |
|------------|-------------|----------|
| `integer` | `decimal` | Automatic widening |
| `string` | `enum` | Validated against enum values |
| `decimal` | `currency` | Precision adjustment |
| `null` | any | Use default value |

```rust
fn coerce(value: Value, target: DataType) -> Result<Value> {
    match (value, target) {
        // Integer to Decimal: widen
        (Value::Int(i), DataType::Decimal) =>
            Ok(Value::Decimal(i as f64)),

        // String to Enum: validate
        (Value::String(s), DataType::Enum(values)) => {
            if values.contains(&s) {
                Ok(Value::Enum(s))
            } else {
                Err(Error::InvalidEnumValue(s))
            }
        }

        // Same type: no coercion needed
        (v, t) if v.type_matches(t) => Ok(v),

        // Incompatible types
        _ => Err(Error::TypeMismatch)
    }
}
```

---

## Step 6: Parallel Execution

### Level-Based Parallelism

Rules at the same execution level run in parallel:

```rust
async fn execute_dag(dag: &DAG, context: &mut Context) {
    for level in dag.levels() {
        // All rules in this level can run in parallel
        let futures: Vec<_> = level.rules
            .iter()
            .map(|rule| execute_rule(rule, context))
            .collect();

        // Wait for all rules in this level to complete
        let results = join_all(futures).await;

        // Store results in context for next level
        for (rule, result) in level.rules.iter().zip(results) {
            for output in &rule.outputs {
                context.set(output, result.get(output));
            }
        }
    }
}
```

### Execution Timeline

```
Time ──────────────────────────────────────────────────────────────►

Level 0:
  ┌──────────────┐
  │ age_factor   │  (0.8µs)
  └──────────────┘
  ┌──────────────┐
  │ base_premium │  (0.6µs)
  └──────────────┘
  ┌──────────────┐
  │smoker_factor │  (0.7µs)
  └──────────────┘
                   │
                   │ Synchronization barrier
                   │
Level 1:           ▼
                  ┌──────────────┐
                  │final_premium │  (0.4µs)
                  └──────────────┘
                                  │
                                  │ Synchronization barrier
                                  │
Level 2:                          ▼
                                 ┌────────────────┐
                                 │monthly_payment │  (0.3µs)
                                 └────────────────┘

Total time: max(L0) + L1 + L2 = 0.8 + 0.4 + 0.3 = 1.5µs

Without parallelism: 0.8 + 0.6 + 0.7 + 0.4 + 0.3 = 2.8µs
Speedup: 1.87x
```

<div class="callout callout-info">
<strong>Parallel Scaling:</strong> Speedup increases with DAG width. Products with many independent rules at each level see greater parallelization benefits—up to 5.4x for deep DAGs with 100+ rules.
</div>

---

## Step 7: Response Formatting

### Result Structure

```json
{
  "success": true,
  "product_id": "insurance-premium-v1",
  "functionality": "quote",
  "outputs": {
    "base_premium": {
      "value": 5000.00,
      "datatype": "currency"
    },
    "age_factor": {
      "value": 1.2,
      "datatype": "percentage"
    },
    "smoker_factor": {
      "value": 1.0,
      "datatype": "percentage"
    },
    "final_premium": {
      "value": 6000.00,
      "datatype": "currency"
    },
    "monthly_payment": {
      "value": 500.00,
      "datatype": "currency"
    }
  },
  "execution_stats": {
    "total_time_us": 1.5,
    "rules_executed": 5,
    "cache_hits": 3,
    "tier": "bytecode"
  }
}
```

### Error Handling

```json
{
  "success": false,
  "error": {
    "code": "MISSING_INPUT",
    "message": "Required input 'customer_age' was not provided",
    "details": {
      "missing_inputs": ["customer_age"],
      "provided_inputs": ["coverage_amount", "smoker_status"]
    }
  }
}
```

---

## Complete Walkthrough Example

Let's trace a complete evaluation:

### Input

```json
{
  "product_id": "insurance-premium-v1",
  "functionality": "quote",
  "inputs": {
    "customer_age": 45,
    "coverage_amount": 250000,
    "smoker_status": "NON_SMOKER"
  }
}
```

### Step-by-Step Trace

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 1: Parse Request                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ Product: insurance-premium-v1                                               │
│ Functionality: quote                                                        │
│ Inputs: {customer_age: 45, coverage_amount: 250000, smoker_status: ...}     │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 2: Load Rules (from cache)                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Rules loaded: 5                                                             │
│ - calculate_age_factor                                                      │
│ - calculate_base_premium                                                    │
│ - calculate_smoker_factor                                                   │
│ - calculate_final_premium                                                   │
│ - calculate_monthly_payment                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 3: Build DAG                                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Level 0: [age_factor, base_premium, smoker_factor] - run in parallel        │
│ Level 1: [final_premium] - depends on Level 0                               │
│ Level 2: [monthly_payment] - depends on Level 1                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 4: Execute Level 0 (Parallel)                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ calculate_age_factor:                                                       │
│   Expression: IF age > 60 THEN 1.5 ELSE IF age > 40 THEN 1.2 ELSE 1.0       │
│   Input: customer_age = 45                                                  │
│   Evaluation: 45 > 60? NO → 45 > 40? YES → 1.2                              │
│   Output: age_factor = 1.2                                                  │
│                                                                             │
│ calculate_base_premium:                                                     │
│   Expression: coverage_amount × 0.02                                        │
│   Input: coverage_amount = 250000                                           │
│   Evaluation: 250000 × 0.02 = 5000                                          │
│   Output: base_premium = 5000.00                                            │
│                                                                             │
│ calculate_smoker_factor:                                                    │
│   Expression: CASE smoker_status WHEN REGULAR 1.8 WHEN OCCASIONAL 1.3 ...   │
│   Input: smoker_status = NON_SMOKER                                         │
│   Evaluation: NON_SMOKER → default → 1.0                                    │
│   Output: smoker_factor = 1.0                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 5: Execute Level 1                                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ calculate_final_premium:                                                    │
│   Expression: base_premium × age_factor × smoker_factor                     │
│   Inputs: base_premium=5000, age_factor=1.2, smoker_factor=1.0              │
│   Evaluation: 5000 × 1.2 × 1.0 = 6000                                       │
│   Output: final_premium = 6000.00                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 6: Execute Level 2                                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ calculate_monthly_payment:                                                  │
│   Expression: final_premium ÷ 12                                            │
│   Input: final_premium = 6000                                               │
│   Evaluation: 6000 ÷ 12 = 500                                               │
│   Output: monthly_payment = 500.00                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 7: Return Response                                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ {                                                                           │
│   "outputs": {                                                              │
│     "base_premium": 5000.00,                                                │
│     "age_factor": 1.2,                                                      │
│     "smoker_factor": 1.0,                                                   │
│     "final_premium": 6000.00,                                               │
│     "monthly_payment": 500.00                                               │
│   }                                                                         │
│ }                                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Performance Benchmarks

### Single Rule Evaluation

| Complexity | Tier 0 (AST) | Tier 1 (Bytecode) |
|------------|--------------|-------------------|
| Simple arithmetic | 0.8µs | 0.25µs |
| Conditional (3 branches) | 1.2µs | 0.35µs |
| Complex (10+ operations) | 2.5µs | 0.6µs |

### Full Product Evaluation

| Scenario | Rules | Levels | Latency |
|----------|-------|--------|---------|
| Simple quote | 3 | 2 | 1.2µs |
| Standard quote | 6 | 3 | 2.1µs |
| Full underwriting | 15 | 5 | 4.5µs |
| Complex pricing | 30 | 7 | 8.2µs |

### Throughput

```
┌────────────────────────────────────────────────────────────┐
│               THROUGHPUT BENCHMARKS                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Single Thread:                                            │
│  ├── Tier 0: ~870,000 evaluations/second                   │
│  └── Tier 1: ~3,000,000 evaluations/second                 │
│                                                            │
│  Multi-Thread (8 cores):                                   │
│  ├── Tier 0: ~6,500,000 evaluations/second                 │
│  └── Tier 1: ~22,000,000 evaluations/second                │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## API Reference

### REST API

```bash
# Evaluate a functionality
POST /api/products/{product_id}/evaluate
Content-Type: application/json

{
  "functionality": "quote",
  "inputs": {
    "customer_age": 45,
    "coverage_amount": 250000
  }
}
```

### gRPC API

```protobuf
service ProductFarmService {
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  rpc EvaluateStream(stream EvaluateRequest) returns (stream EvaluateResponse);
}
```

### Batch Evaluation

```bash
POST /api/products/{product_id}/evaluate/batch
Content-Type: application/json

{
  "functionality": "quote",
  "batch": [
    {"customer_age": 25, "coverage_amount": 100000},
    {"customer_age": 45, "coverage_amount": 250000},
    {"customer_age": 65, "coverage_amount": 500000}
  ]
}
```

---

## Next Steps

- [Architecture](ARCHITECTURE) - System design and component details
- [API Reference](API_REFERENCE) - Complete API documentation
- [Quick Start](QUICK_START) - Build your first product
