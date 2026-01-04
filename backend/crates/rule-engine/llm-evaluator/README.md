# product-farm-llm-evaluator

LLM-powered rule evaluation for complex business logic that's hard to express in traditional rules.

[![Crates.io](https://img.shields.io/crates/v/product-farm-llm-evaluator.svg)](https://crates.io/crates/product-farm-llm-evaluator)
[![Documentation](https://docs.rs/product-farm-llm-evaluator/badge.svg)](https://docs.rs/product-farm-llm-evaluator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Some business rules are too complex or nuanced to express in traditional rule engines. `product-farm-llm-evaluator` bridges this gap by using Large Language Models to evaluate rules described in natural language.

**Example use cases:**
- "Approve if the customer's explanation sounds reasonable"
- "Flag if the transaction pattern looks suspicious"
- "Categorize the product based on its description"

## Features

- **Multiple providers** - Claude (Anthropic) and Ollama support
- **Structured output** - Returns typed values (numbers, booleans, strings)
- **Prompt engineering** - Built-in templates for consistent results
- **Rate limiting** - Configurable concurrency and throttling
- **Fallback handling** - Graceful degradation when LLM unavailable
- **Caching** - Optional response caching for identical inputs

## Part of Product-FARM

This crate is part of [Product-FARM](https://ayushmaanbhav.github.io/Product-FARM/), an enterprise-grade rule engine featuring:

- **Hybrid evaluation** - Combine deterministic rules with LLM judgment
- **Natural language YAML** - Define products in human-readable format
- **3M+ evaluations/sec** for deterministic rules
- **DAG execution** - Automatic dependency resolution

## Installation

```toml
[dependencies]
product-farm-llm-evaluator = { version = "0.2", features = ["anthropic"] }
# or
product-farm-llm-evaluator = { version = "0.2", features = ["ollama"] }
# or both
product-farm-llm-evaluator = { version = "0.2", features = ["all-providers"] }
```

## Usage

### With Claude (Anthropic)

```rust
use product_farm_llm_evaluator::{ClaudeEvaluator, RuleEvaluationContext};

let evaluator = ClaudeEvaluator::new(
    "your-api-key",
    "claude-sonnet-4-20250514",
)?;

let context = RuleEvaluationContext {
    rule_description: "Determine if the refund request is reasonable".into(),
    input_data: serde_json::json!({
        "purchase_date": "2024-01-15",
        "request_date": "2024-01-20",
        "reason": "Product arrived damaged, photos attached",
        "amount": 49.99
    }),
    expected_output_type: OutputType::Boolean,
    constraints: vec!["Consider standard 30-day return policy".into()],
};

let result = evaluator.evaluate(&context).await?;
// result.value = true (refund approved)
// result.confidence = 0.95
// result.reasoning = "Request is within policy window and damage is documented"
```

### With Ollama (Local)

```rust
use product_farm_llm_evaluator::{OllamaEvaluator, RuleEvaluationContext};

let evaluator = OllamaEvaluator::new(
    "http://localhost:11434",
    "llama3.2",
)?;

let result = evaluator.evaluate(&context).await?;
```

## Environment Configuration

```bash
# Anthropic
export RULE_ENGINE_LLM_PROVIDER=anthropic
export RULE_ENGINE_ANTHROPIC_API_KEY=your-key
export RULE_ENGINE_ANTHROPIC_MODEL=claude-sonnet-4-20250514

# Ollama
export RULE_ENGINE_LLM_PROVIDER=ollama
export RULE_ENGINE_OLLAMA_BASE_URL=http://localhost:11434
export RULE_ENGINE_OLLAMA_MODEL=llama3.2

# Shared settings
export RULE_ENGINE_LLM_MAX_CONCURRENCY=5
export RULE_ENGINE_LLM_TIMEOUT_SECS=30
```

## Output Types

| Type | Description | Example |
|------|-------------|---------|
| `Boolean` | Yes/no decisions | Approve refund? |
| `Number` | Numeric scores | Risk score 0-100 |
| `String` | Categories/labels | Sentiment: positive |
| `Json` | Structured data | Extracted entities |

## Best Practices

1. **Be specific** - Clear rule descriptions yield better results
2. **Provide context** - Include relevant data in the input
3. **Set constraints** - Guide the LLM with business rules
4. **Use caching** - Cache identical requests to reduce costs
5. **Have fallbacks** - Default values when LLM is unavailable

## Related Crates

| Crate | Description |
|-------|-------------|
| [product-farm-core](https://crates.io/crates/product-farm-core) | Core domain types |
| [product-farm-rule-engine](https://crates.io/crates/product-farm-rule-engine) | DAG executor |
| [product-farm-yaml-loader](https://crates.io/crates/product-farm-yaml-loader) | YAML definitions with LLM rules |

## Documentation

- [Product-FARM Documentation](https://ayushmaanbhav.github.io/Product-FARM/)
- [API Reference](https://docs.rs/product-farm-llm-evaluator)
- [LLM Integration Guide](https://ayushmaanbhav.github.io/Product-FARM/LLM_INTEGRATION)

## License

MIT License - see [LICENSE](LICENSE) for details.
