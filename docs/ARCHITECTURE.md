---
layout: default
title: Architecture
---

# Product-FARM Architecture

This document provides a comprehensive overview of Product-FARM's system architecture, component design, and data flow.

## Table of Contents

- [System Overview](#system-overview)
- [Component Architecture](#component-architecture)
- [Data Model](#data-model)
- [Rule Engine](#rule-engine)
- [API Layer](#api-layer)
- [Persistence Layer](#persistence-layer)
- [Frontend Architecture](#frontend-architecture)
- [Security Considerations](#security-considerations)
- [Scalability](#scalability)

---

## System Overview

Product-FARM is built as a layered architecture with clear separation of concerns:

```mermaid
flowchart TB
    subgraph Client["CLIENT LAYER"]
        direction TB
        UI["React 19 + TypeScript + Vite"]
        UI --> RB["Visual Rule Builder"]
        UI --> DAG["DAG Canvas (@xyflow)"]
        UI --> SIM["Simulation Panel"]
        UI --> AI["AI Chat Assistant"]
    end

    subgraph API["API LAYER"]
        direction TB
        REST["REST API (Axum)\nPort: 8081"]
        gRPC["gRPC API (Tonic)\nPort: 50051"]
        REST --> Services
        gRPC --> Services
        subgraph Services["Service Layer"]
            PS["ProductService"]
            RS["RuleService"]
            AS["AttributeService"]
            ES["EvalService"]
        end
    end

    subgraph Core["CORE ENGINE"]
        direction LR
        subgraph JL["JSON Logic"]
            Parser
            Compiler
            BytecodeVM["Bytecode VM"]
        end
        subgraph RE["Rule Engine"]
            DAGBuilder["DAG Builder"]
            TopoSort["Topo Sort"]
            Executor
        end
        subgraph AIA["AI Agent"]
            Translator
            Explainer
            Validator
        end
    end

    subgraph Persistence["PERSISTENCE LAYER"]
        direction LR
        DGraph["DGraph\n(Graph DB)\nPort: 9080"]
        Cache["LRU Cache\n(Hot Data)"]
        File["File Storage\n(Development)"]
    end

    Client -->|"HTTP/REST + gRPC"| API
    API --> Core
    Core --> Persistence

    style Client fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style API fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Core fill:#1e3a5f,stroke:#8b5cf6,color:#fff
    style Persistence fill:#1e3a5f,stroke:#10b981,color:#fff
```

---

## Component Architecture

### Backend Crates

The Rust backend is organized as a Cargo workspace with the following crates:

```
backend/crates/
├── core/           # Domain types and business logic
├── json-logic/     # JSON Logic expression engine
├── rule-engine/    # DAG execution engine
├── persistence/    # Storage abstraction layer
├── api/            # REST + gRPC services
└── ai-agent/       # AI-powered rule management tools
```

#### product-farm-core

Core domain types that are shared across all crates:

```rust
// Product with lifecycle management
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub status: ProductStatus,  // Draft → PendingApproval → Active → Discontinued
    pub template_type: String,
    pub effective_from: DateTime,
    pub expiry_at: Option<DateTime>,
    pub version: u32,
}

// Rule with JSON Logic expression
pub struct Rule {
    pub id: RuleId,
    pub product_id: ProductId,
    pub rule_type: String,
    pub input_attributes: Vec<RuleInputAttribute>,
    pub output_attributes: Vec<RuleOutputAttribute>,
    pub compiled_expression: String,  // JSON Logic
    pub display_expression: String,   // Human-readable
    pub order_index: i32,
    pub enabled: bool,
}

// Abstract attribute template
pub struct AbstractAttribute {
    pub abstract_path: String,
    pub product_id: ProductId,
    pub component_type: String,
    pub component_id: Option<String>,
    pub datatype_id: String,
    pub display_names: Vec<DisplayName>,
    pub tags: Vec<Tag>,
    pub immutable: bool,
}

// Dynamic value type
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Decimal(Decimal),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

#### product-farm-json-logic

High-performance JSON Logic implementation with tiered compilation:

```mermaid
flowchart TB
    Input["JSON Logic Expression"] --> Parser["Parser\n(JSON → AST)"]
    Parser --> Validator["Validator\n(Syntax OK)"]
    Parser --> Evaluator["Evaluator\n(High-level)"]

    Evaluator --> Tier0["Tier 0 (AST)\n~1.15µs"]
    Evaluator --> Tier1["Tier 1 (Bytecode)\n~330ns"]
    Evaluator --> Tier2["Tier 2 (JIT)\nFuture"]

    Tier0 -->|"Auto-promote after 100 evals"| Tier1

    style Input fill:#312e81,stroke:#6366f1,color:#fff
    style Parser fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Validator fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Evaluator fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Tier0 fill:#fef3c7,stroke:#f59e0b,color:#000
    style Tier1 fill:#d1fae5,stroke:#10b981,color:#000
    style Tier2 fill:#e5e7eb,stroke:#6b7280,color:#000
```

**Supported Operations:**

| Category | Operations |
|----------|------------|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `min`, `max` |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=`, `===`, `!==` |
| Logic | `and`, `or`, `!`, `!!`, `if` |
| Arrays | `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`, `in` |
| Strings | `cat`, `substr`, `in` |
| Data | `var`, `missing`, `missing_some` |

#### product-farm-rule-engine

DAG-based execution engine:

```rust
pub struct RuleDag {
    graph: DiGraph<RuleNode, ()>,
    nodes: HashMap<RuleId, NodeIndex>,
}

impl RuleDag {
    // Build DAG from rules
    pub fn build(rules: &[Rule]) -> Result<Self, DagError>;

    // Detect cycles
    pub fn has_cycles(&self) -> bool;

    // Get topologically sorted execution order
    pub fn execution_order(&self) -> Vec<RuleId>;

    // Get parallel execution levels
    pub fn execution_levels(&self) -> Vec<Vec<RuleId>>;

    // Generate visualization
    pub fn to_dot(&self) -> String;
    pub fn to_mermaid(&self) -> String;
    pub fn to_ascii(&self) -> String;
}
```

**Execution Flow:**

```mermaid
flowchart TB
    Input["Rules Input"] --> DAGBuilder["DAG Builder\n• Parse rules\n• Extract deps\n• Build graph"]
    DAGBuilder --> CycleCheck["Cycle Check\n(Tarjan's SCC)"]
    CycleCheck -->|"No cycles"| TopoSort["Topological Sort\n(Kahn's Algorithm)"]
    TopoSort --> LevelAssign["Level Assignment\n(BFS by depth)"]

    LevelAssign --> L0["Level 0: [Rule1, Rule2, Rule3]\n⚡ Parallel"]
    LevelAssign --> L1["Level 1: [Rule4, Rule5]\n⚡ Parallel"]
    LevelAssign --> L2["Level 2: [Rule6]\n→ Sequential"]

    L0 --> Result["ExecutionResult"]
    L1 --> Result
    L2 --> Result

    style Input fill:#312e81,stroke:#6366f1,color:#fff
    style DAGBuilder fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style CycleCheck fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style TopoSort fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style LevelAssign fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style L0 fill:#d1fae5,stroke:#10b981,color:#000
    style L1 fill:#d1fae5,stroke:#10b981,color:#000
    style L2 fill:#fef3c7,stroke:#f59e0b,color:#000
    style Result fill:#8b5cf6,stroke:#6366f1,color:#fff
```

#### product-farm-persistence

Multi-backend storage abstraction:

```rust
// Repository traits
#[async_trait]
pub trait ProductRepository {
    async fn save(&self, product: &Product) -> Result<Product>;
    async fn find_by_id(&self, id: &ProductId) -> Result<Option<Product>>;
    async fn find_all(&self) -> Result<Vec<Product>>;
    async fn delete(&self, id: &ProductId) -> Result<()>;
}

// Similar traits for Rule, Attribute, DataType, etc.

// Storage backends
pub enum StorageBackend {
    InMemory,       // HashMap-based, for testing
    File,           // JSON files in directory
    DGraph,         // Graph database
    Hybrid,         // DGraph + LRU cache
}
```

#### product-farm-api

Dual-protocol API layer:

```mermaid
flowchart TB
    subgraph API["API Layer"]
        direction TB
        subgraph Protocols["Protocol Handlers"]
            REST["REST (Axum)\n/api/products\n/api/rules\n/api/attributes\n/api/datatypes\n/api/evaluate"]
            gRPC["gRPC (Tonic)\nProductService\nRuleService\nEvaluationService\nAttributeService"]
        end

        subgraph Shared["Shared Service Layer"]
            PS["ProductService"]
            RS["RuleService"]
            ES["EvalService"]
        end

        REST --> Shared
        gRPC --> Shared
    end

    style API fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Protocols fill:#1e293b,stroke:#475569,color:#fff
    style Shared fill:#1e293b,stroke:#8b5cf6,color:#fff
```

#### product-farm-ai-agent

AI-powered rule management tools:

```rust
pub struct RuleAgent {
    translator: RuleTranslator,   // NL → JSON Logic
    explainer: RuleExplainer,     // JSON Logic → NL
    validator: RuleValidator,     // Syntax + semantic validation
    visualizer: RuleVisualizer,   // DAG visualization
}

impl RuleAgent {
    // Translate natural language to rule
    pub async fn create_rule(&self, description: &str) -> Result<Rule>;

    // Explain rule in plain English
    pub fn explain_rule(&self, rule: &Rule) -> String;

    // Validate rule integrity
    pub fn validate_rule(&self, rule: &Rule) -> ValidationResult;

    // Test rule with sample inputs
    pub fn test_rule(&self, rule: &Rule, inputs: &Value) -> TestResult;
}
```

---

## Data Model

### Entity Relationships

```mermaid
erDiagram
    PRODUCT ||--o{ RULE : contains
    PRODUCT ||--o{ ABSTRACT_ATTRIBUTE : has
    PRODUCT ||--o{ FUNCTIONALITY : defines

    ABSTRACT_ATTRIBUTE }|--|| DATATYPE : typed_by
    ABSTRACT_ATTRIBUTE }o--o{ TAG : tagged_with

    DATATYPE ||--o| ENUMERATION : may_reference

    FUNCTIONALITY ||--o{ ABSTRACT_ATTRIBUTE : requires

    RULE }|--|{ ABSTRACT_ATTRIBUTE : "inputs/outputs"

    PRODUCT {
        string id PK
        string name
        enum status
        string template_type
    }

    RULE {
        string id PK
        string rule_type
        json expression
        array inputs
        array outputs
    }

    ABSTRACT_ATTRIBUTE {
        string abstract_path PK
        string datatype_id FK
        array tags
        boolean immutable
    }

    FUNCTIONALITY {
        string id PK
        string name
        array required_attributes
        enum status
    }

    DATATYPE {
        string id PK
        string primitive
        json constraints
    }

    ENUMERATION {
        string name PK
        array values
        string template_type
    }

    TAG {
        string name PK
        int order_index
    }
```

### Product Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Draft

    Draft --> PendingApproval: submit()
    PendingApproval --> Draft: reject()
    PendingApproval --> Active: approve()

    Active --> Discontinued: discontinue()

    Draft --> Draft: clone()
    Active --> Draft: clone()

    note right of Active: Immutable\n(read-only)
    note right of Discontinued: Preserved for\naudit purposes
```

---

## Rule Engine

### JSON Logic Processing Pipeline

```mermaid
sequenceDiagram
    participant Input as JSON Logic Input
    participant Parser as Parser
    participant AST as AST Builder
    participant Compiler as Compiler
    participant VM as Stack VM

    Note over Input: {"if": [{">": [{"var": "age"}, 60]}, 1.2, 1.0]}

    Input->>Parser: Step 1: PARSING
    Parser->>Parser: serde_json::from_str
    Parser->>AST: JsonValue

    AST->>AST: Step 2: AST CONSTRUCTION
    Note over AST: Expression::If {<br/>condition: Comparison(>,age,60)<br/>then: Literal(1.2)<br/>else: Literal(1.0)}

    AST->>Compiler: Step 3: COMPILATION (if promoted)
    Note over Compiler: Bytecode:<br/>[LoadVar, LoadConst(60),<br/>Gt, JumpIfFalse, ...]

    Compiler->>VM: Step 4: EXECUTION
    Note over VM: Context: {age: 65}<br/>Stack: [65] → [65,60] → [true]<br/>Result: 1.2
```

**Detailed Execution Flow:**

| Step | Instruction | Stack State | Notes |
|------|-------------|-------------|-------|
| 1 | `LoadVar(0)` | `[65]` | Load `age` from context |
| 2 | `LoadConst(60)` | `[65, 60]` | Push constant 60 |
| 3 | `Gt` | `[true]` | 65 > 60 = true |
| 4 | `JumpIfFalse(3)` | `[true]` | No jump (condition true) |
| 5 | `LoadConst(1.2)` | `[1.2]` | Push result |
| 6 | `Return` | `[]` | Return 1.2 |

### DAG Execution

**Rules:**
- R1: `base_premium = coverage * 0.02`
- R2: `age_factor = if(age > 60) 1.2 else 1.0`
- R3: `smoker_factor = if(smoker) 1.5 else 1.0`
- R4: `premium = base_premium * age_factor * smoker_factor`

```mermaid
flowchart TB
    subgraph Inputs["Input Variables"]
        coverage["coverage\n250000"]
        age["age\n65"]
        smoker["smoker\nfalse"]
    end

    subgraph Level0["Level 0 (Parallel Execution ⚡)"]
        R1["R1: base_premium\ncoverage × 0.02\n= 5000"]
        R2["R2: age_factor\nif(65 > 60) 1.2\n= 1.2"]
        R3["R3: smoker_factor\nif(false) 1.5 else 1.0\n= 1.0"]
    end

    subgraph Level1["Level 1 (Sequential)"]
        R4["R4: premium\n5000 × 1.2 × 1.0\n= 6000"]
    end

    coverage --> R1
    age --> R2
    smoker --> R3

    R1 --> R4
    R2 --> R4
    R3 --> R4

    R4 --> Output["premium = 6000"]

    style Inputs fill:#312e81,stroke:#6366f1,color:#fff
    style Level0 fill:#065f46,stroke:#10b981,color:#fff
    style Level1 fill:#7c2d12,stroke:#f59e0b,color:#fff
    style Output fill:#8b5cf6,stroke:#6366f1,color:#fff
```

**Execution Timeline:**

| Level | Rules | Execution Mode | Results |
|-------|-------|----------------|---------|
| 0 | R1, R2, R3 | **Parallel** | 5000, 1.2, 1.0 |
| 1 | R4 | Sequential | 6000 |

---

## API Layer

### REST API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/products` | GET | List all products |
| `/api/products` | POST | Create product |
| `/api/products/{id}` | GET | Get product by ID |
| `/api/products/{id}` | PUT | Update product |
| `/api/products/{id}` | DELETE | Delete product |
| `/api/products/{id}/clone` | POST | Clone product |
| `/api/products/{id}/submit` | POST | Submit for approval |
| `/api/products/{id}/approve` | POST | Approve product |
| `/api/products/{id}/reject` | POST | Reject product |
| `/api/products/{id}/rules` | GET | List product rules |
| `/api/products/{id}/rules` | POST | Create rule |
| `/api/products/{id}/evaluate` | POST | Evaluate rules |
| `/api/products/{id}/batch-evaluate` | POST | Batch evaluation |
| `/api/abstract-attributes` | GET, POST | Attribute templates |
| `/api/datatypes` | GET, POST | Custom datatypes |
| `/api/enumerations` | GET, POST | Enumerations |

### gRPC Services

```protobuf
// Evaluation Service
service ProductFarmService {
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  rpc BatchEvaluate(BatchEvaluateRequest) returns (BatchEvaluateResponse);
  rpc EvaluateStream(stream EvaluateRequest) returns (stream EvaluateResponse);
  rpc ValidateRules(ValidateRulesRequest) returns (ValidateRulesResponse);
  rpc GetExecutionPlan(GetExecutionPlanRequest) returns (ExecutionPlanResponse);
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

// Product Management
service ProductService {
  rpc CreateProduct(CreateProductRequest) returns (Product);
  rpc GetProduct(GetProductRequest) returns (Product);
  rpc UpdateProduct(UpdateProductRequest) returns (Product);
  rpc DeleteProduct(DeleteProductRequest) returns (Empty);
  rpc ListProducts(ListProductsRequest) returns (ListProductsResponse);
  rpc CloneProduct(CloneProductRequest) returns (Product);
  rpc SubmitProduct(SubmitProductRequest) returns (Product);
  rpc ApproveProduct(ApproveProductRequest) returns (Product);
  rpc RejectProduct(RejectProductRequest) returns (Product);
}

// Rule Management
service RuleService {
  rpc CreateRule(CreateRuleRequest) returns (Rule);
  rpc GetRule(GetRuleRequest) returns (Rule);
  rpc UpdateRule(UpdateRuleRequest) returns (Rule);
  rpc DeleteRule(DeleteRuleRequest) returns (Empty);
  rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
}
```

---

## Persistence Layer

### Storage Backends

```mermaid
flowchart TB
    subgraph Backends["Storage Backend Options"]
        direction LR
        InMemory["InMemory\n• HashMap\n• Fast\n• No persist\n• Testing"]
        File["File\n• JSON files\n• Simple\n• Portable\n• Development"]
        DGraph["DGraph\n• Graph DB\n• Scalable\n• GraphQL\n• Production"]
    end

    InMemory --> Trait
    File --> Trait
    DGraph --> Trait

    subgraph Trait["Repository Trait"]
        save["save()"]
        find["find_by_id()"]
        findAll["find_all()"]
        delete["delete()"]
    end

    style Backends fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Trait fill:#1e293b,stroke:#8b5cf6,color:#fff
    style InMemory fill:#fef3c7,stroke:#f59e0b,color:#000
    style File fill:#dbeafe,stroke:#3b82f6,color:#000
    style DGraph fill:#d1fae5,stroke:#10b981,color:#000
```

### Hybrid Storage (Production)

```mermaid
flowchart TB
    Request["Request"] --> Cache{"LRU Cache\n(Hot Data)"}

    Cache -->|"Cache Hit\n~1µs"| Return["Return Cached"]
    Cache -->|"Cache Miss"| DGraph["DGraph Query\n~1-5ms"]

    DGraph --> Update["Update Cache\n& Return"]

    style Request fill:#312e81,stroke:#6366f1,color:#fff
    style Cache fill:#fef3c7,stroke:#f59e0b,color:#000
    style Return fill:#d1fae5,stroke:#10b981,color:#000
    style DGraph fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style Update fill:#d1fae5,stroke:#10b981,color:#000
```

---

## Frontend Architecture

### Component Structure

```
frontend/src/
├── components/
│   ├── RuleBuilder.tsx        # Block-based rule editor
│   ├── RuleCanvas.tsx         # DAG visualization (@xyflow)
│   ├── RuleValidator.tsx      # Rule validation UI
│   ├── SimulationPanel.tsx    # Rule testing interface
│   ├── BatchEvaluator.tsx     # Batch evaluation UI
│   ├── AIChat.tsx             # AI assistant chat
│   ├── AttributeExplorer.tsx  # Attribute browser
│   └── ProductCreationWizard.tsx
│
├── pages/
│   ├── Dashboard.tsx          # Overview page
│   ├── Products.tsx           # Product management
│   ├── Rules.tsx              # Rule management
│   ├── Attributes.tsx         # Attribute management
│   ├── Datatypes.tsx          # Datatype management
│   └── Enumerations.tsx       # Enumeration management
│
├── services/
│   └── api.ts                 # REST API client
│
├── store/
│   └── index.ts               # Zustand state management
│
└── types/
    └── index.ts               # TypeScript type definitions
```

### State Management (Zustand)

```typescript
interface AppState {
  // Products
  products: Product[];
  selectedProduct: Product | null;

  // Rules
  rules: Rule[];
  selectedRule: Rule | null;

  // Attributes
  abstractAttributes: AbstractAttribute[];

  // UI State
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchProducts: () => Promise<void>;
  createProduct: (product: CreateProductRequest) => Promise<void>;
  evaluateRules: (inputs: Record<string, any>) => Promise<EvaluationResult>;
}
```

---

## Security Considerations

### Input Validation

- All API inputs validated with regex patterns
- Product IDs: alphanumeric, no leading digits
- Rule types: strict pattern matching
- JSON Logic expressions: syntax validation before execution

### Product Lifecycle Protection

- Active products are immutable (read-only)
- Modification requires cloning to new Draft
- Version tracking for optimistic locking

### API Security (Future)

- [ ] OAuth2/OIDC authentication
- [ ] Role-based access control
- [ ] API rate limiting
- [ ] Audit logging

<div class="callout callout-warning">
<strong>Security Note:</strong> In production deployments, always enable authentication, validate all inputs at API boundaries, and use TLS for all connections. Product-FARM validates JSON Logic expressions before execution to prevent injection attacks.
</div>

---

## Scalability

### Horizontal Scaling

```mermaid
flowchart TB
    LB["Load Balancer"] --> API1["API Node 1\n(Rust)"]
    LB --> API2["API Node 2\n(Rust)"]
    LB --> API3["API Node 3\n(Rust)"]

    API1 --> DGraph
    API2 --> DGraph
    API3 --> DGraph

    subgraph DGraph["DGraph Cluster"]
        Alpha1["Alpha 1"]
        Alpha2["Alpha 2"]
        Zero["Zero\n(Coordinator)"]
    end

    style LB fill:#8b5cf6,stroke:#6366f1,color:#fff
    style API1 fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style API2 fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style API3 fill:#1e3a5f,stroke:#3b82f6,color:#fff
    style DGraph fill:#065f46,stroke:#10b981,color:#fff
```

### Performance Optimizations

1. **Tiered Compilation**: Auto-promote hot rules to bytecode
2. **LRU Caching**: Cache frequently accessed data
3. **Parallel Execution**: Rules without dependencies run concurrently
4. **Batch Evaluation**: Process multiple inputs efficiently
5. **Connection Pooling**: Reuse database connections

<div class="callout callout-performance">
<strong>Production Deployment:</strong> For maximum throughput, deploy multiple stateless API nodes behind a load balancer. Each node maintains its own LRU cache, with DGraph providing the shared source of truth.
</div>

---

## Summary

Product-FARM's architecture is designed for:

- **Performance**: Sub-millisecond rule evaluation with tiered compilation
- **Scalability**: Horizontal scaling with stateless API nodes
- **Flexibility**: Domain-agnostic design supports any business domain
- **Maintainability**: Clean separation of concerns across crates
- **Extensibility**: Pluggable storage backends and API protocols
