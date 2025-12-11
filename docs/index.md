---
layout: default
title: Home
---

# Product-FARM

**High-Performance Domain-Agnostic Rule Engine with AI-Powered Configuration**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-19-blue?logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue?logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](https://github.com/ayushmaanbhav/product-farm/blob/master/LICENSE)

---

## What is Product-FARM?

Product-FARM is a powerful, flexible rule engine designed for building complex business logic without writing code. It combines:

- **Visual Rule Builder** - Create rules through an intuitive drag-and-drop interface
- **DAG Execution Engine** - Automatic dependency resolution with parallel execution
- **JSON Logic Expressions** - Industry-standard expression language
- **AI-Powered Assistance** - Natural language rule creation and explanation
- **Sub-millisecond Performance** - Tiered compilation with bytecode optimization

![Dashboard](screenshots/dashboard-populated.png)

---

## Key Features

### Visual Rule Management

Build complex business rules visually with the block-based rule builder. See your rules as a dependency graph that updates in real-time.

![Rules DAG](screenshots/rules-dag-full-view.png)

### Domain-Agnostic Design

Product-FARM works across any industry:
- **Insurance** - Premium calculations, risk assessment
- **Finance** - Loan eligibility, trading signals
- **E-commerce** - Dynamic pricing, inventory rules
- **Healthcare** - Risk scoring, eligibility checks

### High Performance

| Metric | Performance |
|--------|-------------|
| Rule Evaluation | ~330ns (bytecode) |
| Parallel Execution | Automatic DAG-based |
| Batch Processing | 100K+ evaluations/sec |

---

## Quick Links

| Resource | Description |
|----------|-------------|
| [Quick Start](QUICK_START) | Get up and running in 5 minutes |
| [Architecture](ARCHITECTURE) | System design and component overview |
| [Use Cases](USE_CASES) | Real-world examples across industries |
| [API Reference](API_REFERENCE) | Complete REST and gRPC documentation |
| [GitHub](https://github.com/ayushmaanbhav/product-farm) | Source code and issues |

---

## Technology Stack

### Backend (Rust)
- **Axum** - High-performance REST API
- **Tonic** - gRPC services
- **Tokio** - Async runtime
- **DGraph** - Graph database

### Frontend (React)
- **React 19** - UI framework
- **TypeScript 5.0+** - Type safety
- **TailwindCSS 4** - Styling
- **@xyflow/react** - DAG visualization
- **shadcn/ui** - Component library

---

## Screenshots

### Dashboard
![Dashboard](screenshots/dashboard-populated.png)

### Rule Builder
![Rule Builder](screenshots/rule-builder-expression.png)

### Rule Canvas with DAG
![Rule Canvas](screenshots/rules-canvas-with-rules.png)

### Attributes Management
![Attributes](screenshots/attributes-populated.png)

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/ayushmaanbhav/product-farm.git
cd product-farm

# Start all services
./start-all.sh

# Open http://localhost:5173
```

See the [Quick Start Guide](QUICK_START) for detailed instructions.

---

## Example: Insurance Premium Calculation

```json
{
  "rule_type": "CALCULATION",
  "display_expression": "final_premium = base_premium * age_factor * smoker_factor",
  "expression": {
    "*": [
      {"var": "base_premium"},
      {"var": "age_factor"},
      {"var": "smoker_factor"}
    ]
  }
}
```

**Input:**
```json
{
  "coverage": 250000,
  "customer_age": 65,
  "smoker": false
}
```

**Output:**
```json
{
  "base_premium": 5000,
  "age_factor": 1.2,
  "smoker_factor": 1.0,
  "final_premium": 6000
}
```

---

## License

MIT License - see [LICENSE](https://github.com/ayushmaanbhav/product-farm/blob/master/LICENSE) for details.

---

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the [GitHub repository](https://github.com/ayushmaanbhav/product-farm).
