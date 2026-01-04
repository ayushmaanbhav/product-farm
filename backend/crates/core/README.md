# product-farm-core

Core domain types and abstractions for the Product-FARM rule engine ecosystem.

[![Crates.io](https://img.shields.io/crates/v/product-farm-core.svg)](https://crates.io/crates/product-farm-core)
[![Documentation](https://docs.rs/product-farm-core/badge.svg)](https://docs.rs/product-farm-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`product-farm-core` provides the foundational types used across the Product-FARM rule engine:

- **Value** - A flexible, JSON-compatible value type optimized for rule evaluation
- **Attribute** - Schema definitions for product attributes with validation
- **Rule** - Rule definitions with expression-based computation
- **Product** - Product configurations with attribute schemas and rule sets
- **Error types** - Comprehensive error handling across the ecosystem

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), an enterprise-grade rule engine featuring:

- **3M+ evaluations/sec** single-threaded, 22M+ multi-threaded
- **Natural language YAML** - Define rules in human-readable YAML with LLM-powered evaluation
- **DAG-based execution** - Automatic dependency resolution and parallel execution
- **Bytecode compilation** - 3.5x faster than AST interpretation
- **1 million rules** tested at 96k rules/sec

## Installation

```toml
[dependencies]
product-farm-core = "0.2"
```

## Usage

```rust
use product_farm_core::{Value, Attribute, AttributeType};

// Create typed values
let price = Value::Number(99.99);
let name = Value::String("Premium Plan".into());
let features = Value::Array(vec![
    Value::String("Feature A".into()),
    Value::String("Feature B".into()),
]);

// Define attributes with validation
let attr = Attribute {
    id: "base_price".into(),
    name: "Base Price".into(),
    attribute_type: AttributeType::Number,
    default_value: Some(Value::Number(0.0)),
    constraints: vec![],
};
```

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-json-logic](https://crates.io/crates/product-farm-json-logic) | JSON Logic parser with bytecode VM |
| [product-farm-rule-engine](https://crates.io/crates/product-farm-rule-engine) | DAG executor with parallel evaluation |
| [product-farm-farmscript](https://crates.io/crates/product-farm-farmscript) | Human-friendly DSL for rules |
| [product-farm-llm-evaluator](https://crates.io/crates/product-farm-llm-evaluator) | LLM-powered rule evaluation |
| [product-farm-yaml-loader](https://crates.io/crates/product-farm-yaml-loader) | YAML-based product definitions |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-core)
- [Performance Benchmarks](https://ayushmaanbhav.github.io/Product-FARM/BENCHMARKS)

## License

MIT License - see [LICENSE](LICENSE) for details.
