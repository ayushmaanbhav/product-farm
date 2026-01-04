# product-farm-yaml-loader

Flexible YAML-based product definition loader with schema inference and LLM integration.

[![Crates.io](https://img.shields.io/crates/v/product-farm-yaml-loader.svg)](https://crates.io/crates/product-farm-yaml-loader)
[![Documentation](https://docs.rs/product-farm-yaml-loader/badge.svg)](https://docs.rs/product-farm-yaml-loader)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Define complex products with rules in human-readable YAML. No programming required.

```yaml
product:
  name: Auto Insurance
  version: "2.0"

attributes:
  - name: driver_age
    type: number
    constraints:
      min: 16
      max: 100

  - name: vehicle_value
    type: number

  - name: risk_score
    type: number
    rule: |
      if driver_age < 25 then 1.5
      else if driver_age > 65 then 1.3
      else 1.0

  - name: base_premium
    type: number
    rule:
      expression: vehicle_value * 0.02 * risk_score
      depends_on: [vehicle_value, risk_score]

  - name: approval_decision
    type: boolean
    llm_rule:
      description: "Approve if the applicant's profile is acceptable"
      provider: claude
      constraints:
        - "Reject if risk_score > 2.0"
        - "Consider claims_history in decision"
```

## Features

- **Natural language rules** - Write rules in FarmScript or JSON Logic
- **Schema inference** - Auto-detect types from example data
- **LLM integration** - Use AI for complex decisions
- **Dependency resolution** - Automatic rule ordering
- **Hot reload** - Update products without restart
- **Validation** - Comprehensive schema and rule validation
- **Multi-file** - Split large products across files

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), an enterprise-grade rule engine featuring:

- **3M+ evaluations/sec** with bytecode compilation
- **1 million rules** tested at 96k rules/sec
- **DAG execution** - Parallel rule evaluation
- **Visual builder** - Web UI for non-technical users

## Installation

```toml
[dependencies]
product-farm-yaml-loader = "0.2"
```

## Usage

### Loading Products

```rust
use product_farm_yaml_loader::{ProductRegistry, YamlLoader};

// Load from directory
let registry = ProductRegistry::from_directory("./products")?;

// Or load single file
let loader = YamlLoader::new();
let product = loader.load_file("./products/insurance.yaml")?;

// Access product
let insurance = registry.get("auto-insurance")?;
```

### Evaluating Rules

```rust
use product_farm_yaml_loader::ProductEvaluator;
use serde_json::json;

let evaluator = ProductEvaluator::new(&product);

let inputs = json!({
    "driver_age": 22,
    "vehicle_value": 25000,
    "claims_history": []
});

let results = evaluator.evaluate(&inputs).await?;
// results["risk_score"] = 1.5
// results["base_premium"] = 750.0
// results["approval_decision"] = true
```

## YAML Schema

### Product Definition

```yaml
product:
  name: string          # Required
  version: string       # Semantic version
  description: string   # Optional
  tags: [string]        # For organization
```

### Attributes

```yaml
attributes:
  - name: attribute_name
    type: number | string | boolean | array | object
    description: "Human-readable description"
    default: value
    constraints:
      min: number
      max: number
      pattern: regex
      enum: [allowed, values]
```

### Rules

```yaml
# FarmScript (recommended)
rule: |
  if condition then result
  else default

# JSON Logic
rule:
  expression:
    "*": [{"var": "a"}, {"var": "b"}]
  depends_on: [a, b]

# LLM-powered
llm_rule:
  description: "Natural language description"
  provider: claude | ollama
  output_type: boolean | number | string
  constraints:
    - "Business rule 1"
    - "Business rule 2"
```

### Scenarios (for testing)

```yaml
scenarios:
  - name: "Young driver high-value car"
    inputs:
      driver_age: 20
      vehicle_value: 50000
    expected:
      risk_score: 1.5
      base_premium: 1500
```

## Directory Structure

```
products/
├── insurance/
│   ├── product.yaml      # Main definition
│   ├── attributes.yaml   # Attribute schemas
│   ├── rules.yaml        # Rule definitions
│   └── scenarios.yaml    # Test scenarios
└── banking/
    └── product.yaml
```

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-core](https://crates.io/crates/product-farm-core) | Core domain types |
| [product-farm-json-logic](https://crates.io/crates/product-farm-json-logic) | JSON Logic execution |
| [product-farm-rule-engine](https://crates.io/crates/product-farm-rule-engine) | DAG executor |
| [product-farm-farmscript](https://crates.io/crates/product-farm-farmscript) | Human-friendly DSL |
| [product-farm-llm-evaluator](https://crates.io/crates/product-farm-llm-evaluator) | LLM integration |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-yaml-loader)
- [YAML Schema Reference](https://ayushmaanbhav.github.io/Product-FARM/YAML_SCHEMA)
- [Examples](https://github.com/ayushmaanbhav/product-farm/tree/master/examples)

## License

MIT License - see [LICENSE](LICENSE) for details.
