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

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  React 19 + TypeScript + Vite + TailwindCSS + shadcn/ui             │   │
│  │  • Visual Rule Builder    • DAG Canvas (@xyflow)                     │   │
│  │  • Simulation Panel       • AI Chat Assistant                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────┬────────────────────────────────────────┘
                                     │ HTTP/REST + gRPC
┌────────────────────────────────────▼────────────────────────────────────────┐
│                               API LAYER                                      │
│  ┌───────────────────────┐       ┌───────────────────────┐                  │
│  │  REST API (Axum)      │       │  gRPC API (Tonic)     │                  │
│  │  Port: 8081           │       │  Port: 50051          │                  │
│  │  • Products           │       │  • Evaluation         │                  │
│  │  • Rules              │       │  • Streaming          │                  │
│  │  • Attributes         │       │  • Batch Processing   │                  │
│  │  • Datatypes          │       │                       │                  │
│  │  • Enumerations       │       │                       │                  │
│  └───────────┬───────────┘       └───────────┬───────────┘                  │
│              │                               │                               │
│  ┌───────────▼───────────────────────────────▼───────────┐                  │
│  │                   Service Layer                        │                  │
│  │  ProductService │ RuleService │ AttributeService       │                  │
│  │  DatatypeService │ EnumerationService │ EvalService    │                  │
│  └───────────────────────────┬───────────────────────────┘                  │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────────────────┐
│                            CORE ENGINE                                       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐          │
│  │  JSON Logic      │  │  Rule Engine     │  │  AI Agent        │          │
│  │  ┌────────────┐  │  │  ┌────────────┐  │  │  ┌────────────┐  │          │
│  │  │ Parser     │  │  │  │ DAG Builder│  │  │  │ Translator │  │          │
│  │  │ Compiler   │  │  │  │ Topo Sort  │  │  │  │ Explainer  │  │          │
│  │  │ Bytecode VM│  │  │  │ Executor   │  │  │  │ Validator  │  │          │
│  │  │ Evaluator  │  │  │  │ Context    │  │  │  │ Visualizer │  │          │
│  │  └────────────┘  │  │  └────────────┘  │  │  └────────────┘  │          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘          │
└──────────────────────────────┬──────────────────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────────────────┐
│                         PERSISTENCE LAYER                                    │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐          │
│  │  DGraph          │  │  LRU Cache       │  │  File Storage    │          │
│  │  (Graph DB)      │  │  (Hot Data)      │  │  (Development)   │          │
│  │  Port: 9080      │  │                  │  │                  │          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘          │
└─────────────────────────────────────────────────────────────────────────────┘
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

```
                    JSON Logic Expression
                            │
                            ▼
                    ┌───────────────┐
                    │    Parser     │───────────────┐
                    │  (JSON → AST) │               │
                    └───────┬───────┘               │
                            │                       │
                            ▼                       ▼
                    ┌───────────────┐       ┌─────────────┐
                    │   Evaluator   │       │  Validator  │
                    │  (High-level) │       │ (Syntax OK) │
                    └───────┬───────┘       └─────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
        ┌─────────┐   ┌─────────┐   ┌─────────┐
        │ Tier 0  │   │ Tier 1  │   │ Tier 2  │
        │  (AST)  │   │(Bytecode│   │  (JIT)  │
        │~1.15µs  │──►│  ~330ns │   │ Future  │
        └─────────┘   └─────────┘   └─────────┘
              │             │
              └──── Auto-promote after 100 evals
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

```
Rules Input
      │
      ▼
┌─────────────────┐
│   DAG Builder   │
│  • Parse rules  │
│  • Extract deps │
│  • Build graph  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Cycle Check    │
│  (Tarjan's SCC) │
└────────┬────────┘
         │ No cycles
         ▼
┌─────────────────┐
│ Topological     │
│ Sort (Kahn's)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Level Assignment│
│  (BFS by depth) │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│       Parallel Execution         │
│  Level 0: [Rule1, Rule2, Rule3] │ ──► Parallel
│  Level 1: [Rule4, Rule5]        │ ──► Parallel
│  Level 2: [Rule6]               │ ──► Sequential
└────────────────┬────────────────┘
                 │
                 ▼
          ExecutionResult
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

```
┌─────────────────────────────────────────────────────────────┐
│                      API Layer                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐        │
│  │    REST (Axum)      │    │   gRPC (Tonic)      │        │
│  │                     │    │                     │        │
│  │  /api/products      │    │  ProductService     │        │
│  │  /api/rules         │    │  RuleService        │        │
│  │  /api/attributes    │    │  EvaluationService  │        │
│  │  /api/datatypes     │    │  AttributeService   │        │
│  │  /api/enumerations  │    │  DatatypeService    │        │
│  │  /api/evaluate      │    │                     │        │
│  └──────────┬──────────┘    └──────────┬──────────┘        │
│             │                          │                    │
│  ┌──────────▼──────────────────────────▼──────────┐        │
│  │              Shared Service Layer               │        │
│  │  ProductService │ RuleService │ EvalService     │        │
│  └─────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
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

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DATA MODEL                                         │
└─────────────────────────────────────────────────────────────────────────────┘

                              ┌──────────────┐
                              │   Product    │
                              │              │
                              │ id           │
                              │ name         │
                              │ status       │
                              │ template_type│
                              └──────┬───────┘
                                     │
           ┌─────────────────────────┼─────────────────────────┐
           │                         │                         │
           ▼                         ▼                         ▼
    ┌──────────────┐          ┌──────────────┐          ┌──────────────┐
    │    Rule      │          │   Abstract   │          │ Functionality│
    │              │          │  Attribute   │          │              │
    │ id           │          │              │          │ id           │
    │ rule_type    │          │ abstract_path│          │ name         │
    │ expression   │          │ datatype_id  │──────────│ required_    │
    │ inputs[]     │──────────│ tags[]       │          │ attributes[] │
    │ outputs[]    │          │ immutable    │          │ status       │
    └──────────────┘          └──────┬───────┘          └──────────────┘
                                     │
                              ┌──────▼───────┐
                              │  DataType    │
                              │              │
                              │ id           │
                              │ primitive    │
                              │ constraints  │
                              └──────────────┘
                                     │
                              ┌──────▼───────┐
                              │ Enumeration  │
                              │              │
                              │ name         │
                              │ values[]     │
                              │ template_type│
                              └──────────────┘
```

### Product Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PRODUCT LIFECYCLE                                     │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────┐     submit      ┌─────────────────┐
    │        │ ──────────────► │                 │
    │ DRAFT  │                 │ PENDING_APPROVAL│
    │        │ ◄────────────── │                 │
    └────┬───┘     reject      └────────┬────────┘
         │                              │
         │                              │ approve
         │                              ▼
         │                       ┌──────────────┐
         │                       │              │
         │                       │    ACTIVE    │
         │                       │  (immutable) │
         │                       │              │
         │                       └──────┬───────┘
         │                              │
         │                              │ discontinue
         │                              ▼
         │                       ┌──────────────┐
         │                       │              │
         │                       │ DISCONTINUED │
         │                       │              │
         │                       └──────────────┘
         │
         │ clone
         ▼
    ┌────────┐
    │  NEW   │
    │ DRAFT  │
    └────────┘
```

---

## Rule Engine

### JSON Logic Processing Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     JSON LOGIC PROCESSING PIPELINE                           │
└─────────────────────────────────────────────────────────────────────────────┘

Input: {"if": [{">": [{"var": "age"}, 60]}, 1.2, 1.0]}

Step 1: PARSING
┌─────────────────────────────────────────────────────────────┐
│  JSON String ──► serde_json::from_str ──► JsonValue         │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
Step 2: AST CONSTRUCTION
┌─────────────────────────────────────────────────────────────┐
│  JsonValue ──► Expression::parse ──► Expression AST         │
│                                                              │
│  Expression::If {                                            │
│      condition: Expression::Comparison {                     │
│          op: GreaterThan,                                    │
│          left: Expression::Variable("age"),                  │
│          right: Expression::Literal(60),                     │
│      },                                                      │
│      then_branch: Expression::Literal(1.2),                  │
│      else_branch: Expression::Literal(1.0),                  │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
Step 3: COMPILATION (if promoted to Tier 1)
┌─────────────────────────────────────────────────────────────┐
│  Expression AST ──► Compiler ──► Bytecode                    │
│                                                              │
│  [LoadVar(0), LoadConst(60), Gt, JumpIfFalse(3),             │
│   LoadConst(1.2), Jump(2), LoadConst(1.0), Return]           │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
Step 4: EXECUTION
┌─────────────────────────────────────────────────────────────┐
│  Context: {age: 65}                                          │
│                                                              │
│  Stack VM:                                                   │
│  • LoadVar(0)       → stack: [65]                           │
│  • LoadConst(60)    → stack: [65, 60]                       │
│  • Gt               → stack: [true]                         │
│  • JumpIfFalse(3)   → no jump (true)                        │
│  • LoadConst(1.2)   → stack: [1.2]                          │
│  • Return           → result: 1.2                           │
└─────────────────────────────────────────────────────────────┘
```

### DAG Execution

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          DAG EXECUTION FLOW                                  │
└─────────────────────────────────────────────────────────────────────────────┘

Rules:
  R1: base_premium = coverage * 0.02
  R2: age_factor = if(age > 60) 1.2 else 1.0
  R3: smoker_factor = if(smoker) 1.5 else 1.0
  R4: premium = base_premium * age_factor * smoker_factor

Step 1: Build DAG
┌─────────────────────────────────────────────────────────────┐
│                                                              │
│  Inputs: coverage, age, smoker                              │
│           │         │      │                                 │
│           ▼         ▼      ▼                                 │
│        ┌────┐   ┌────┐   ┌────┐                             │
│        │ R1 │   │ R2 │   │ R3 │   ◄── Level 0 (parallel)    │
│        └──┬─┘   └──┬─┘   └──┬─┘                             │
│           │        │        │                                │
│           └────────┼────────┘                                │
│                    │                                         │
│                    ▼                                         │
│                 ┌────┐                                       │
│                 │ R4 │           ◄── Level 1                 │
│                 └──┬─┘                                       │
│                    │                                         │
│                    ▼                                         │
│                 premium                                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘

Step 2: Execute by Level
┌─────────────────────────────────────────────────────────────┐
│  Level 0: Execute R1, R2, R3 in parallel                    │
│           • R1: 250000 * 0.02 = 5000                        │
│           • R2: 65 > 60 → 1.2                               │
│           • R3: false → 1.0                                 │
│                                                              │
│  Level 1: Execute R4                                         │
│           • R4: 5000 * 1.2 * 1.0 = 6000                     │
└─────────────────────────────────────────────────────────────┘
```

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

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STORAGE BACKENDS                                     │
└─────────────────────────────────────────────────────────────────────────────┘

┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│   InMemory    │     │     File      │     │    DGraph     │
│               │     │               │     │               │
│ • HashMap     │     │ • JSON files  │     │ • Graph DB    │
│ • Fast        │     │ • Simple      │     │ • Scalable    │
│ • No persist  │     │ • Portable    │     │ • GraphQL     │
│ • Testing     │     │ • Development │     │ • Production  │
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │  Repository Trait │
                    │                   │
                    │ save()            │
                    │ find_by_id()      │
                    │ find_all()        │
                    │ delete()          │
                    └───────────────────┘
```

### Hybrid Storage (Production)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HYBRID STORAGE                                       │
└─────────────────────────────────────────────────────────────────────────────┘

                     Request
                        │
                        ▼
               ┌────────────────┐
               │   LRU Cache    │
               │   (Hot Data)   │
               └───────┬────────┘
                       │
           ┌───────────┴───────────┐
           │ Cache Hit             │ Cache Miss
           ▼                       ▼
    ┌──────────────┐        ┌──────────────┐
    │   Return     │        │   DGraph     │
    │   Cached     │        │   Query      │
    └──────────────┘        └──────┬───────┘
                                   │
                                   ▼
                            ┌──────────────┐
                            │ Update Cache │
                            │ & Return     │
                            └──────────────┘
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

---

## Scalability

### Horizontal Scaling

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HORIZONTAL SCALING                                   │
└─────────────────────────────────────────────────────────────────────────────┘

                         Load Balancer
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
           ▼                  ▼                  ▼
    ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
    │   API Node   │   │   API Node   │   │   API Node   │
    │   (Rust)     │   │   (Rust)     │   │   (Rust)     │
    └──────┬───────┘   └──────┬───────┘   └──────┬───────┘
           │                  │                  │
           └──────────────────┼──────────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │  DGraph Cluster   │
                    │  (Alpha + Zero)   │
                    └───────────────────┘
```

### Performance Optimizations

1. **Tiered Compilation**: Auto-promote hot rules to bytecode
2. **LRU Caching**: Cache frequently accessed data
3. **Parallel Execution**: Rules without dependencies run concurrently
4. **Batch Evaluation**: Process multiple inputs efficiently
5. **Connection Pooling**: Reuse database connections

---

## Summary

Product-FARM's architecture is designed for:

- **Performance**: Sub-millisecond rule evaluation with tiered compilation
- **Scalability**: Horizontal scaling with stateless API nodes
- **Flexibility**: Domain-agnostic design supports any business domain
- **Maintainability**: Clean separation of concerns across crates
- **Extensibility**: Pluggable storage backends and API protocols
