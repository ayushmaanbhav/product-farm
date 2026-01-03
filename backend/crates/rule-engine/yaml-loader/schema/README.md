# Product-FARM YAML Schema Guide

This directory contains the YAML schema definitions for Product-FARM assessment scenarios.

## Files

| File | Purpose | When to Use |
|------|---------|-------------|
| `llm-guide.yaml` | Complete comprehensive guide | Give to LLM for full scenario generation |
| `product-schema.yaml` | Product metadata schema | Understanding product.yaml structure |
| `entities-schema.yaml` | Entity definitions schema | Understanding entities.yaml structure |
| `functions-schema.yaml` | Function/rule definitions schema | Understanding functions.yaml structure |

## Usage

### Full Scenario Generation

For LLMs to generate complete assessment scenarios from client requirements:

```
Provide the full contents of llm-guide.yaml as system context, then give client requirements.
```

The `llm-guide.yaml` includes:
- All schema definitions
- FarmScript language reference
- Complete generation guidelines
- Full example scenario

### Targeted Reference

For specific questions about one file type, provide only that schema:
- Questions about product configuration → `product-schema.yaml`
- Questions about data entities → `entities-schema.yaml`
- Questions about evaluation rules → `functions-schema.yaml`

## Architecture

Product-FARM uses a 5-layer architecture:

| Layer | Name | Contents |
|-------|------|----------|
| **Layer 1** | Requirements | Client objectives, competencies, success criteria |
| **Layer 2** | Domain | Scenario spec, narrative, phases, personas |
| **Layer 3** | Backend | Events, detection rules, transitions, branches |
| **Layer 4** | Session UI | Candidate interface, state management |
| **Layer 5** | Portal | Reports, scores, analytics |

## Output Structure

A complete Product-FARM scenario consists of three YAML files:

```
my-product-v1/
├── product.yaml      # Metadata + layer configuration
├── entities.yaml     # Data type definitions
└── functions.yaml    # Evaluation rules (FarmScript + LLM)
```

## FarmScript Quick Reference

FarmScript is the expression language for deterministic rules:

```farmscript
# Comparisons
candidate.score >= 70

# Conditionals
if score > 85 then "excellent" else if score > 70 then "good" else "needs_work"

# Array operations
all(competencies, c => c.score >= c.threshold)
filter(signals, s => s.type == "positive")
reduce(items, (sum, i) => sum + i.value, 0)

# Null-safe access
(candidate.certifications ?? []).length > 0
```

## LLM Rule Quick Reference

For subjective evaluations, use LLM-based rules:

```yaml
detect-communication-quality:
  description: Evaluates clarity of candidate communication
  llm:
    prompt: |
      Rate this message's clarity on 0-100:
      {{message}}

      Respond with JSON: {"score": N, "issues": [...]}
    temperature: 0.3
    output_format: json
```
