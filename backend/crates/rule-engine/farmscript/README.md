# product-farm-farmscript

Human-friendly DSL that compiles to JSON Logic for intuitive rule authoring.

[![Crates.io](https://img.shields.io/crates/v/product-farm-farmscript.svg)](https://crates.io/crates/product-farm-farmscript)
[![Documentation](https://docs.rs/product-farm-farmscript/badge.svg)](https://docs.rs/product-farm-farmscript)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

FarmScript is a domain-specific language designed for business users to write rules without learning JSON Logic syntax. It compiles to JSON Logic for execution.

**Write this:**
```
if age < 25 then base_rate * 1.5
else if age < 35 then base_rate * 1.2
else base_rate
```

**Instead of this:**
```json
{
  "if": [
    {"<": [{"var": "age"}, 25]}, {"*": [{"var": "base_rate"}, 1.5]},
    {"<": [{"var": "age"}, 35]}, {"*": [{"var": "base_rate"}, 1.2]},
    {"var": "base_rate"}
  ]
}
```

## Features

- **Natural syntax** - Reads like English, not code
- **Type inference** - Automatic type detection from context
- **Error messages** - Clear, actionable error descriptions
- **Zero runtime** - Compiles to JSON Logic at build time
- **IDE support** - Syntax highlighting and completion (coming soon)

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), an enterprise-grade rule engine featuring:

- **3M+ evaluations/sec** with bytecode compilation
- **Natural language YAML** - Define products in human-readable format
- **LLM-powered evaluation** - Use AI for complex rule interpretation
- **1 million rules** tested at 96k rules/sec

## Installation

```toml
[dependencies]
product-farm-farmscript = "0.2"
```

## Usage

```rust
use product_farm_farmscript::{Parser, compile_to_json_logic};

// Parse and compile FarmScript
let script = r#"
    if customer_type == "premium" then
        base_price * 0.9
    else if quantity > 100 then
        base_price * 0.95
    else
        base_price
"#;

let json_logic = compile_to_json_logic(script)?;
// Now execute with product-farm-json-logic or product-farm-rule-engine
```

## Syntax Reference

### Variables
```
age                    # Simple variable
customer.address.city  # Nested access
items[0].price         # Array access
```

### Operators
```
# Arithmetic
price + tax
total - discount
quantity * unit_price
amount / count

# Comparison
age < 25
status == "active"
score >= threshold

# Logical
is_member and has_discount
is_premium or is_vip
not is_blocked
```

### Conditionals
```
if condition then result

if condition then
    result_a
else
    result_b

if condition_1 then result_1
else if condition_2 then result_2
else default_result
```

### Functions
```
min(a, b, c)
max(price, floor_price)
round(calculated_value, 2)
```

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-core](https://crates.io/crates/product-farm-core) | Core domain types |
| [product-farm-json-logic](https://crates.io/crates/product-farm-json-logic) | JSON Logic execution |
| [product-farm-rule-engine](https://crates.io/crates/product-farm-rule-engine) | DAG executor |
| [product-farm-yaml-loader](https://crates.io/crates/product-farm-yaml-loader) | YAML definitions |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-farmscript)
- [FarmScript Guide](https://ayushmaanbhav.github.io/Product-FARM/FARMSCRIPT)

## License

MIT License - see [LICENSE](LICENSE) for details.
