# product-farm-json-logic

High-performance JSON Logic implementation with bytecode compilation and a register-based VM.

[![Crates.io](https://img.shields.io/crates/v/product-farm-json-logic.svg)](https://crates.io/crates/product-farm-json-logic)
[![Documentation](https://docs.rs/product-farm-json-logic/badge.svg)](https://docs.rs/product-farm-json-logic)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`product-farm-json-logic` is a blazing-fast JSON Logic evaluator featuring:

- **Tiered execution** - AST interpretation for simple rules, bytecode for complex ones
- **Bytecode compiler** - Compiles JSON Logic to optimized bytecode
- **Register-based VM** - Executes bytecode with minimal overhead
- **3.5x faster** than pure AST interpretation
- **~330ns** per evaluation for compiled rules

## Performance

| Mode | Time | Throughput |
|------|------|------------|
| AST Interpretation | ~1.15μs | 870K/sec |
| Bytecode Execution | ~330ns | 3M/sec |
| **Improvement** | **3.5x** | **3.5x** |

### Conditional Rules

| Complexity | AST | Bytecode | Speedup |
|------------|-----|----------|---------|
| Single if/else | 1.8μs | 450ns | 4.0x |
| Nested (3 levels) | 3.2μs | 680ns | 4.7x |
| Multiple (5 branches) | 4.1μs | 820ns | 5.0x |

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), an enterprise-grade rule engine featuring:

- **Natural language YAML** - Define rules in human-readable format
- **LLM-powered evaluation** - Use Claude or Ollama for complex rule interpretation
- **DAG execution** - Automatic dependency resolution
- **1 million rules** tested at 96k rules/sec

## Installation

```toml
[dependencies]
product-farm-json-logic = "0.2"
```

## Usage

```rust
use product_farm_json_logic::{JsonLogicEvaluator, CachedExpression};
use product_farm_core::Value;
use serde_json::json;

// Create an evaluator
let mut evaluator = JsonLogicEvaluator::new();

// Parse and cache an expression (auto-compiles to bytecode if complex)
let expr = CachedExpression::new(json!({
    "if": [
        {"<": [{"var": "age"}, 25]},
        {"*": [{"var": "base_rate"}, 1.5]},
        {"<": [{"var": "age"}, 35]},
        {"*": [{"var": "base_rate"}, 1.2]},
        {"var": "base_rate"}
    ]
}));

// Evaluate with data
let data = Value::Object(hashbrown::HashMap::from([
    ("age".into(), Value::Number(30.0)),
    ("base_rate".into(), Value::Number(100.0)),
]));

let result = evaluator.evaluate_cached_value(&expr, &data)?;
// result = 120.0 (base_rate * 1.2 for age 30)
```

## Supported Operations

### Arithmetic
`+`, `-`, `*`, `/`, `%`, `min`, `max`

### Comparison
`==`, `!=`, `<`, `<=`, `>`, `>=`, `===`, `!==`

### Logic
`and`, `or`, `!`, `!!`, `if`, `?:`

### String
`cat`, `substr`, `in`

### Array
`map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`

### Data Access
`var`, `missing`, `missing_some`

## Architecture

```
JSON Logic Expression
        │
        ▼
   ┌─────────┐
   │  Parser │ ──────► AST
   └─────────┘
        │
        ▼ (if nodes > 5)
   ┌──────────┐
   │ Compiler │ ──────► Bytecode
   └──────────┘
        │
        ▼
   ┌─────────┐
   │   VM    │ ──────► Result
   └─────────┘
```

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-core](https://crates.io/crates/product-farm-core) | Core domain types |
| [product-farm-rule-engine](https://crates.io/crates/product-farm-rule-engine) | DAG executor |
| [product-farm-farmscript](https://crates.io/crates/product-farm-farmscript) | Human-friendly DSL |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-json-logic)
- [JSON Logic Specification](https://jsonlogic.com/)

## License

MIT License - see [LICENSE](LICENSE) for details.
