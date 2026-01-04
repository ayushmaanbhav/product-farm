# product-farm-rule-engine

High-performance DAG-based rule execution engine with automatic dependency resolution and parallel evaluation.

[![Crates.io](https://img.shields.io/crates/v/product-farm-rule-engine.svg)](https://crates.io/crates/product-farm-rule-engine)
[![Documentation](https://docs.rs/product-farm-rule-engine/badge.svg)](https://docs.rs/product-farm-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`product-farm-rule-engine` is an enterprise-grade rule execution engine featuring:

- **DAG-based execution** - Automatic dependency resolution between rules
- **Parallel evaluation** - Execute independent rules concurrently with Rayon
- **Bytecode compilation** - 3.5x faster than AST interpretation
- **1 million rules** - Tested at 96k rules/sec with complex dependency graphs
- **Zero-copy design** - Minimal allocations during evaluation

## Performance

### Large-Scale Benchmarks (16-core Linux)

| Test Pattern | Rules | Time | Throughput |
|--------------|-------|------|------------|
| 100k Chain | 100,000 | 907ms | 110k/sec |
| 10k Diamond | 10,002 | 5ms | 2M/sec |
| 100x1000 Lattice | 100,001 | 445ms | 225k/sec |
| Tree (depth 8) | 87,382 | 34ms | 2.6M/sec |
| **1M Combined** | **1,000,000** | **10.4s** | **96k/sec** |

### DAG Parallel Speedup

| DAG Levels | Rules | Sequential | Parallel | Speedup |
|------------|-------|------------|----------|---------|
| 3 levels | 10 | 11.5μs | 4.2μs | 2.7x |
| 5 levels | 25 | 28.8μs | 8.1μs | 3.6x |
| 7 levels | 50 | 57.5μs | 12.4μs | 4.6x |
| 10 levels | 100 | 115μs | 21.3μs | 5.4x |

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), featuring:

- **Natural language YAML** - Define products and rules in human-readable YAML
- **LLM-powered evaluation** - Use Claude or Ollama for complex rule interpretation
- **FarmScript DSL** - Write rules like `if age < 25 then base_rate * 1.5`
- **Visual rule builder** - Web UI for non-technical users

## Installation

```toml
[dependencies]
product-farm-rule-engine = "0.2"
```

## Usage

```rust
use product_farm_rule_engine::{RuleExecutor, ExecutionContext, Rule, RuleId};
use product_farm_core::Value;
use serde_json::json;

// Create rules with dependencies
let rules = vec![
    Rule {
        id: RuleId::new("base_premium"),
        expression: json!({"*": [{"var": "age"}, 10]}),
        output_attribute: "base_premium".into(),
        dependencies: vec![],
    },
    Rule {
        id: RuleId::new("risk_factor"),
        expression: json!({
            "if": [
                {"<": [{"var": "age"}, 25]}, 1.5,
                {"<": [{"var": "age"}, 35]}, 1.2,
                1.0
            ]
        }),
        output_attribute: "risk_factor".into(),
        dependencies: vec![],
    },
    Rule {
        id: RuleId::new("final_premium"),
        expression: json!({
            "*": [{"var": "base_premium"}, {"var": "risk_factor"}]
        }),
        output_attribute: "final_premium".into(),
        dependencies: vec![
            RuleId::new("base_premium"),
            RuleId::new("risk_factor"),
        ],
    },
];

// Execute with automatic dependency resolution
let executor = RuleExecutor::new();
let mut context = ExecutionContext::new();
context.set_variable("age", Value::Number(30.0));

let results = executor.execute(&rules, &context)?;
// Results computed in optimal order with parallel execution
```

## Architecture

```
Rules with Dependencies
         │
         ▼
   ┌───────────┐
   │ DAG Build │ ──► Topological Sort
   └───────────┘
         │
         ▼
   ┌───────────┐
   │  Levels   │ ──► Group by dependency depth
   └───────────┘
         │
         ▼
   ┌───────────┐
   │  Rayon    │ ──► Parallel execution per level
   └───────────┘
         │
         ▼
      Results
```

## Features

### Automatic Dependency Resolution
Rules are automatically sorted by dependencies. No manual ordering required.

### Parallel Execution
Independent rules execute concurrently using Rayon's work-stealing scheduler.

### Bytecode Compilation
Complex expressions are compiled to bytecode for 3.5x faster execution.

### Cycle Detection
Circular dependencies are detected at build time with clear error messages.

### Incremental Evaluation
Re-evaluate only affected rules when inputs change.

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-core](https://crates.io/crates/product-farm-core) | Core domain types |
| [product-farm-json-logic](https://crates.io/crates/product-farm-json-logic) | JSON Logic with bytecode VM |
| [product-farm-farmscript](https://crates.io/crates/product-farm-farmscript) | Human-friendly DSL |
| [product-farm-llm-evaluator](https://crates.io/crates/product-farm-llm-evaluator) | LLM-powered evaluation |
| [product-farm-yaml-loader](https://crates.io/crates/product-farm-yaml-loader) | YAML product definitions |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-rule-engine)
- [Performance Benchmarks](https://ayushmaanbhav.github.io/Product-FARM/BENCHMARKS)
- [Architecture Guide](https://ayushmaanbhav.github.io/Product-FARM/ARCHITECTURE)

## License

MIT License - see [LICENSE](LICENSE) for details.
