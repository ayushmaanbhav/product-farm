# Product-FARM Rule Engine (Rust)

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange?style=for-the-badge&logo=rust" alt="Rust"/>
  <img src="https://img.shields.io/badge/tests-234_passing-success?style=for-the-badge&logo=checkmarx" alt="Tests"/>
  <img src="https://img.shields.io/badge/coverage-85%25-brightgreen?style=for-the-badge" alt="Coverage"/>
  <img src="https://img.shields.io/badge/license-Apache_2.0-blue?style=for-the-badge" alt="License"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/gRPC-tonic_0.12-blue?style=for-the-badge" alt="gRPC"/>
  <img src="https://img.shields.io/badge/REST-axum_0.7-green?style=for-the-badge" alt="REST"/>
  <img src="https://img.shields.io/badge/performance-<1ms-blueviolet?style=for-the-badge" alt="Performance"/>
</p>

A high-performance rule engine written in Rust, designed for evaluating complex business rules with JSON Logic expressions and DAG-based dependency resolution.

## Features

- **JSON Logic Evaluation**: Full implementation of JSON Logic with extensions
- **Tiered Compilation**: Automatic promotion from AST (Tier 0) to bytecode VM (Tier 1) after 100 evaluations
- **DAG-Based Execution**: Automatic dependency resolution with topological sorting
- **Parallel Execution Levels**: Rules without dependencies can run in parallel
- **Rule Builders**: Fluent API for creating common rule patterns (calculations, conditions, signals)
- **Rule Validation**: Pre-execution validation with cycle detection and dependency analysis
- **Batch Evaluation**: Evaluate rules against multiple input datasets efficiently
- **Persistence Layer**: In-memory and file-based storage for products, rules, and attributes
- **gRPC API**: Full-featured gRPC service for remote rule evaluation
- **Generic Design**: No hardcoded domain types - fully configurable via rules

## Performance

Benchmarks on a typical development machine:

### Tiered Compilation
| Tier | Time | Speedup |
|------|------|---------|
| Tier 0 (AST interpretation) | ~1.15µs | baseline |
| Tier 1 (Bytecode VM) | ~330ns | **3.5x faster** |
| RuleCache (promoted) | ~366ns | ~3.1x faster |

### JSON Logic Operations
| Operation | Time |
|-----------|------|
| Simple arithmetic (cached) | ~300ns |
| Nested expression (depth 5) | ~3.2µs |
| Nested expression (depth 10) | ~10.9µs |
| RSI signal rule | ~2.2µs |
| Complex entry logic (5 conditions) | ~18µs |
| Insurance premium (5 rules) | ~7.8µs |

### Rule Engine (DAG Execution)
| Operation | Time |
|-----------|------|
| Full trading strategy | ~10µs |
| 10-rule chain execution | ~19µs |
| 25-rule chain execution | ~97µs |
| 50-rule chain execution | ~347µs |

## Quick Start

### As a Library

```rust
use product_farm_core::{ProductId, Rule};
use product_farm_api::ProductFarmService;
use serde_json::json;

fn main() {
    let mut service = ProductFarmService::new();

    let rules = vec![
        Rule::new("my-product", "CALC", json!({
            "*": [{"var": "x"}, 2]
        }))
        .with_inputs(["x"])
        .with_outputs(["doubled"]),
    ];

    let result = service.evaluate(
        &ProductId::new("my-product"),
        &rules,
        &json!({"x": 21}),
    ).unwrap();

    println!("Result: {}", result["doubled"]); // 42
}
```

### As a gRPC Server

```bash
# Start the server
cargo run -p product-farm-api

# Or with a custom port
cargo run -p product-farm-api -- 9000
```

## Project Structure

```
engine-rs/
├── crates/
│   ├── core/           # Domain types (Product, Rule, Attribute, Value)
│   ├── json-logic/     # JSON Logic parser, compiler, VM
│   ├── rule-engine/    # DAG executor, context management
│   ├── persistence/    # Storage layer (stubs)
│   └── api/            # gRPC server and services
├── Cargo.toml          # Workspace configuration
└── README.md
```

## Crates

### `product-farm-core`
Core domain types:
- `Product`: Container for rules and attributes
- `Rule`: JSON Logic expression with input/output mappings
- `Attribute`: Dynamic attribute definitions
- `Value`: Runtime value type with JSON interop

### `product-farm-json-logic`
JSON Logic implementation:
- Parser: JSON to AST conversion
- Compiler: AST to bytecode
- VM: Stack-based bytecode interpreter
- Evaluator: High-level evaluation API
- Tiered Compilation: Automatic promotion from AST to bytecode (3.5x speedup)
- Bytecode Persistence: Serialize/deserialize compiled rules via `PersistedRule`

Supported operations:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`, `===`, `!==`
- Logic: `and`, `or`, `!`, `!!`, `if`
- Arrays: `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`, `in`
- Strings: `cat`, `substr`, `in`
- Data: `var`, `missing`, `missing_some`

### `product-farm-rule-engine`
DAG-based execution engine:
- Automatic dependency detection from input/output attributes
- Topological sorting for correct execution order
- Cycle detection
- Execution context management

### `product-farm-persistence`
Storage layer with multiple backends:
- In-memory repositories for testing and development
- File-based JSON storage for simple deployments
- Compiled rule persistence (bytecode cache for warm starts)
- Repository traits: `ProductRepository`, `RuleRepository`, `AttributeRepository`, `CompiledRuleRepository`

### `product-farm-api`
gRPC service layer:
- `ProductFarmService`: Evaluate, ValidateRules, GetExecutionPlan, HealthCheck
- `ProductService`: CRUD for products
- `RuleService`: CRUD for rules
- Streaming evaluation support

## JSON Logic Examples

### Simple Calculation
```json
{"*": [{"var": "price"}, {"var": "quantity"}]}
```

### Conditional Logic
```json
{
  "if": [
    {">": [{"var": "age"}, 60]}, "senior",
    {">": [{"var": "age"}, 18]}, "adult",
    "minor"
  ]
}
```

### Complex Trading Rule
```json
{
  "if": [
    {"and": [
      {"<": [{"var": "rsi"}, 30]},
      {">": [{"var": "price"}, {"var": "sma_50"}]}
    ]}, "BUY",
    {"and": [
      {">": [{"var": "rsi"}, 70]},
      {"<": [{"var": "price"}, {"var": "sma_50"}]}
    ]}, "SELL",
    "HOLD"
  ]
}
```

### Array Operations
```json
{
  "reduce": [
    {"var": "items"},
    {"+": [{"var": "accumulator"}, {"var": "current"}]},
    0
  ]
}
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p product-farm-json-logic

# Integration tests
cargo test -p product-farm-api --test grpc_integration
```

## Running Benchmarks

```bash
# JSON Logic benchmarks
cargo bench -p product-farm-json-logic

# Rule engine benchmarks
cargo bench -p product-farm-rule-engine
```

## Examples

```bash
# Local evaluation (no server)
cargo run -p product-farm-api --example local_evaluation

# Comprehensive feature showcase
cargo run -p product-farm-api --example comprehensive

# Bytecode persistence and warm starts
cargo run -p product-farm-api --example bytecode_persistence

# gRPC client basics (requires server)
# First start the server: cargo run -p product-farm-api
cargo run -p product-farm-api --example grpc_client

# Trading strategy (requires server)
cargo run -p product-farm-api --example trading_strategy
```

## gRPC API

### Services

#### ProductFarmService
- `Evaluate`: Evaluate rules for a product
- `BatchEvaluate`: Evaluate rules against multiple input datasets
- `EvaluateStream`: Streaming evaluation
- `ValidateRules`: Validate rules without executing
- `GetExecutionPlan`: Get DAG visualization (DOT, Mermaid, ASCII formats)
- `HealthCheck`: Server health status

#### ProductService
- `CreateProduct`, `GetProduct`, `UpdateProduct`, `DeleteProduct`, `ListProducts`

#### RuleService
- `CreateRule`, `GetRule`, `UpdateRule`, `DeleteRule`, `ListRules`

### Example gRPC Client (using grpcurl)

```bash
# Health check
grpcurl -plaintext localhost:50051 product_farm.ProductFarmService/HealthCheck

# Create a product
grpcurl -plaintext -d '{
  "name": "My Strategy",
  "description": "Trading strategy",
  "template_type": "TRADING"
}' localhost:50051 product_farm.ProductService/CreateProduct

# Create a rule
grpcurl -plaintext -d '{
  "product_id": "<product-id>",
  "rule_type": "SIGNAL",
  "input_attributes": ["rsi"],
  "output_attributes": ["signal"],
  "expression_json": "{\"if\": [{\"<\": [{\"var\": \"rsi\"}, 30]}, \"BUY\", \"HOLD\"]}"
}' localhost:50051 product_farm.RuleService/CreateRule

# Evaluate
grpcurl -plaintext -d '{
  "product_id": "<product-id>",
  "input_data": {"rsi": {"int_value": 25}}
}' localhost:50051 product_farm.ProductFarmService/Evaluate

# Batch Evaluate (multiple inputs at once)
grpcurl -plaintext -d '{
  "product_id": "<product-id>",
  "inputs": [
    {"input_id": "customer_001", "data": {"age": {"int_value": 25}, "coverage": {"int_value": 100000}}},
    {"input_id": "customer_002", "data": {"age": {"int_value": 45}, "coverage": {"int_value": 250000}}},
    {"input_id": "customer_003", "data": {"age": {"int_value": 65}, "coverage": {"int_value": 500000}}}
  ]
}' localhost:50051 product_farm.ProductFarmService/BatchEvaluate

# Get execution plan with visualizations
grpcurl -plaintext -d '{
  "product_id": "<product-id>"
}' localhost:50051 product_farm.ProductFarmService/GetExecutionPlan
# Response includes: levels, dependencies, dot_graph, mermaid_graph, ascii_graph
```

## Configuration

### Server Configuration

```rust
use product_farm_api::{ProductFarmServer, ServerConfig};

let config = ServerConfig::default()
    .with_addr("0.0.0.0:9000".parse().unwrap())
    .with_max_message_size(16 * 1024 * 1024)  // 16MB
    .with_keepalive(30, 60);

ProductFarmServer::with_config(config).run().await?;
```

### Environment Variables

- `RUST_LOG`: Logging level (e.g., `product_farm_api=debug`)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        gRPC API                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │ProductFarm   │ │Product       │ │Rule          │         │
│  │Service       │ │Service       │ │Service       │         │
│  └──────────────┘ └──────────────┘ └──────────────┘         │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                     Rule Engine                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │DAG Builder   │ │Executor      │ │Context       │         │
│  │              │ │              │ │Manager       │         │
│  └──────────────┘ └──────────────┘ └──────────────┘         │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    JSON Logic                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │Parser        │ │Compiler      │ │VM            │         │
│  │(JSON→AST)    │ │(AST→Bytecode)│ │(Execution)   │         │
│  └──────────────┘ └──────────────┘ └──────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## License

Apache 2.0
