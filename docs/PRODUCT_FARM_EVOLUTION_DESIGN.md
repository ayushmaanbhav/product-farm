# Product-FARM Evolution: Comprehensive Design & Requirements Document

**Author:** Analysis based on existing codebase and spec
**Date:** December 2024
**Status:** ✅ **IMPLEMENTED** - The Rust migration is complete!

---

## Implementation Status (December 2024)

> **Note:** This document was originally written as a design proposal for migrating Product-FARM from Kotlin to Rust. **The migration has been successfully completed.** The sections below reflect the original design decisions, many of which have now been implemented.

### What Has Been Built

| Component | Status | Details |
|-----------|--------|---------|
| **Rust Backend** | ✅ Complete | ~20,710 lines across 6 crates |
| **React Frontend** | ✅ Complete | ~17,009 lines with full UI |
| **JSON Logic VM** | ✅ Complete | Tiered compilation (AST → Bytecode, 3.5x faster) |
| **Rule Engine** | ✅ Complete | DAG execution, topological sort, cycle detection |
| **REST API** | ✅ Complete | Axum-based on port 8081 |
| **gRPC API** | ✅ Complete | Tonic-based on port 50051 |
| **DGraph Integration** | ✅ Complete | Graph database for persistence |
| **Visual DAG Editor** | ✅ Complete | @xyflow/react integration |
| **AI Agent Tools** | ✅ Complete | Rule translation, explanation, validation |
| **E2E Tests** | ✅ Complete | 9 Playwright test suites |
| **Backend Tests** | ✅ Complete | 234 passing tests |

### Current Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PRESENTATION LAYER                                 │
│  React 19 + TypeScript + Vite + TailwindCSS + shadcn/ui + @xyflow          │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │ Dashboard  │ │ Rule Canvas│ │ Simulation │ │ AI Chat    │              │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘              │
└─────────────────────────────┬───────────────────────────────────────────────┘
                              │ HTTP (Port 8081) / gRPC (Port 50051)
┌─────────────────────────────▼───────────────────────────────────────────────┐
│                          RUST BACKEND                                        │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐          │
│  │ product-farm-api │  │ rule-engine      │  │ json-logic       │          │
│  │ (Axum + Tonic)   │  │ (DAG Executor)   │  │ (Parser/VM)      │          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐          │
│  │ core             │  │ persistence      │  │ ai-agent         │          │
│  │ (Domain Types)   │  │ (DGraph + Cache) │  │ (LLM Tools)      │          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘          │
└─────────────────────────────┬───────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────────────────┐
│                          PERSISTENCE LAYER                                   │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    DGraph (Port 9080)                                 │   │
│  │  - Products, Rules, Attributes as graph nodes                        │   │
│  │  - Dependencies as edges                                              │   │
│  │  - LRU Cache for hot data                                            │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Performance Achieved

| Metric | Target | Achieved |
|--------|--------|----------|
| Rule evaluation (simple) | <1ms | **~330ns** (Tier 1 bytecode) |
| Rule evaluation (complex) | <10ms | **~7.8µs** (5 rules) |
| Compilation speedup | 2x+ | **3.5x** (AST → Bytecode) |
| Test coverage | 80%+ | **~85%** |

For current documentation, see:
- [README.md](../README.md) - Main documentation
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [QUICK_START.md](./QUICK_START.md) - Getting started guide
- [API_REFERENCE.md](./API_REFERENCE.md) - API documentation
- [USE_CASES.md](./USE_CASES.md) - Example use cases

---

## Original Design Document

The sections below contain the original design proposal that guided the implementation.

---

## Executive Summary

Product-FARM (Product Functionality, Attribute and Rule Management System) is a **well-architected foundation** with solid domain modeling for rule-based product configuration. After thorough analysis of the codebase (~17,000 lines of Kotlin across 368 files) and the original spec document, this document evaluates whether to continue building on this foundation and outlines a comprehensive evolution path.

**Verdict: YES, this is worth continuing** - but with significant architectural evolution rather than incremental improvements.

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Gap Analysis: Spec vs Implementation](#2-gap-analysis)
3. [Feasibility Assessment](#3-feasibility-assessment)
4. [Evolution Architecture](#4-evolution-architecture)
5. [Technology Stack Decisions](#5-technology-stack-decisions)
   - 5.1 [Database: Why Graph? (DGraph vs IndraDB vs SurrealDB)](#51-database-why-graph)
   - 5.2 [Core Engine: Why Rust?](#52-core-engine-why-rust)
   - 5.3 [Rule Execution: JSON Logic Performance Analysis](#53-rule-execution-json-logic-performance-analysis) *(NEW)*
   - 5.4 [Migration Strategy: Kotlin → Rust](#54-migration-strategy-kotlin--rust)
6. [AI-Powered Rule Management](#6-ai-powered-rule-management)
7. [UX/UI Design Philosophy](#7-uxui-design-philosophy)
8. [Stock Trading Use Case](#8-stock-trading-use-case)
9. [Implementation Roadmap](#9-implementation-roadmap)
10. [What NOT To Do](#10-what-not-to-do)
11. [Assumptions & Risks](#11-assumptions--risks)

---

## 1. Current State Analysis

### 1.1 What's Built (Strengths)

| Component | Status | Quality |
|-----------|--------|---------|
| **Domain Model** | Complete | Excellent - Rich entity hierarchy with proper JPA patterns |
| **Rule Engine** | Complete | Good - DAG-based dependency resolution, topological sort |
| **JSON Logic Evaluator** | Complete | Good - 103 source files, comprehensive operations |
| **REST API Layer** | Complete | Good - 6 controllers, versioned endpoints |
| **Database Schema** | Complete | Good - 10 Liquibase migrations |
| **Service Layer** | Partial | Missing clone operations (TODO markers) |
| **Tests** | Partial | Good for json-logic/rule-framework, missing for API layer |

### 1.2 Core Concepts Successfully Modeled

```
Product (lifecycle: DRAFT → PENDING_APPROVAL → ACTIVE → DISCONTINUED)
    ├── AbstractAttribute (templates with datatypes, constraints, tags)
    │   └── Attribute (concrete instances with values or rules)
    ├── ProductFunctionality (capabilities requiring specific attributes)
    └── Rule (JSON Logic expressions with input/output dependencies)
```

### 1.3 Technical Debt & Incomplete Areas

1. **Clone Operations** - `AbstractAttributeService.clone()` has `TODO()` markers
2. **No API Tests** - Main module lacks integration test coverage
3. **kv-store Module** - Declared but not implemented
4. **No Search/Filtering** - Only basic repository queries
5. **No Authentication** - Security layer missing
6. **No Audit Trail** - Product version history not tracked

---

## 2. Gap Analysis

### 2.1 Spec vs Implementation Matrix

| Spec Requirement | Implementation Status | Gap Severity |
|------------------|----------------------|--------------|
| Product CRUD with lifecycle | ✅ Complete | None |
| Abstract/Concrete Attributes | ✅ Complete | None |
| Rule with input/output mapping | ✅ Complete | None |
| Functionality constraints | ✅ Complete | None |
| Product cloning | ⚠️ Partial | Medium |
| Approval workflow | ✅ Complete | None |
| Display expressions | ✅ Complete | None |
| Cycle detection in rules | ✅ Complete | None |
| UI for configuration | ❌ Not started | Critical |
| Sheet parsing/generation | ❌ Not started | High |
| Ops portal dashboards | ❌ Not started | Critical |

### 2.2 Original Vision vs Current Reality

**Original Vision (from spec):**
> "We need to have a UI in order to manage the attributes. As attributes have interdependencies and rules can be complex, we need to have an interactive UI for configuration management."

**Current Reality:**
- API-only backend with no UI
- No visual representation of rule dependencies
- No sheet import/export capability
- No user-friendly way for product teams to configure rules

---

## 3. Feasibility Assessment

### 3.1 Is This Worth Continuing?

**YES**, for the following reasons:

| Factor | Assessment |
|--------|------------|
| **Domain Model Maturity** | The core concepts (Product, Attribute, Rule, Functionality) are well-modeled and align with real-world rule-based systems |
| **Rule Engine Quality** | DAG-based execution with cycle detection is non-trivial and correctly implemented |
| **Extensibility** | Component-based architecture allows extension to non-insurance domains |
| **Code Quality** | Clean Kotlin code with proper patterns (Repository, Service, Transformer) |
| **Foundation Reusability** | Even with Rust rewrite, the domain knowledge and API contracts are valuable |

### 3.2 What Makes This Valuable

1. **Generic Rule Engine** - Not insurance-specific, can power any rule-based system
2. **Attribute Dependency Graph** - Critical for complex product configurations
3. **Templatization** - Abstract → Concrete attribute pattern enables reuse
4. **JSON Logic** - Industry-standard expression format, portable

### 3.3 What Needs Fundamental Change

| Current Approach | Problem | Better Approach |
|------------------|---------|-----------------|
| PostgreSQL with JSONB | Not optimized for graph traversal | Graph database (Neo4j, SurrealDB, or EdgeDB) |
| Kotlin/JVM | Memory overhead, GC pauses | Rust for core engine |
| No UI | Product teams can't self-serve | AI-assisted visual interface |
| Batch evaluation | Latency for real-time use cases | Streaming with incremental updates |

---

## 4. Evolution Architecture

### 4.1 Target Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PRESENTATION LAYER                                 │
├─────────────────────┬─────────────────────┬─────────────────────────────────┤
│    Web Dashboard    │   AI Chat Interface │      GraphQL/REST API           │
│   (React + D3.js)   │  (Natural Language) │   (for external integrations)   │
└─────────┬───────────┴──────────┬──────────┴───────────────┬─────────────────┘
          │                      │                          │
          ▼                      ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ORCHESTRATION LAYER                                  │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │  AI Agent Hub   │  │ Rule Compiler   │  │  Event Router   │              │
│  │ (LLM + Tools)   │  │ (NL → JSON)     │  │  (Pub/Sub)      │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
          │                      │                          │
          ▼                      ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CORE ENGINE (RUST)                                  │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │  Rule Evaluator │  │   DAG Executor  │  │  Cache Layer    │              │
│  │  (json-logic)   │  │  (Topo Sort)    │  │  (In-Memory)    │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │  Type System    │  │  Validation     │  │   gRPC Server   │              │
│  │  (Constraints)  │  │  (Constraints)  │  │   (Tonic)       │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
          │                      │                          │
          ▼                      ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PERSISTENCE LAYER                                   │
│                                                                              │
│  ┌─────────────────────────────────┐  ┌─────────────────────────────────┐   │
│  │     Graph Database              │  │      Time-Series DB             │   │
│  │  (SurrealDB / EdgeDB / Neo4j)   │  │  (QuestDB / TimescaleDB)        │   │
│  │                                 │  │  (for trading signals)          │   │
│  │  - Products as Nodes            │  │                                 │   │
│  │  - Attributes as Properties     │  │  - Price history                │   │
│  │  - Rules as Edges               │  │  - Signal events                │   │
│  │  - Dependencies as Graph        │  │  - Rule trigger logs            │   │
│  └─────────────────────────────────┘  └─────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Domain Model Evolution

```
                    CURRENT (Relational)

    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
    │   Product   │──────│  Attribute  │──────│    Rule     │
    └─────────────┘      └─────────────┘      └─────────────┘
           │                    │                    │
           │             FK relations          FK relations
           │                    │                    │
    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
    │Functionality│──────│  Abstract   │──────│  DataType   │
    └─────────────┘      │  Attribute  │      └─────────────┘
                         └─────────────┘

                                 ▼

                     EVOLVED (Graph-Native)

              ┌──────────────────────────────────────┐
              │              Product                 │
              │  (Node with embedded properties)     │
              └──────────────────┬───────────────────┘
                                 │
           ┌─────────────────────┼─────────────────────┐
           │                     │                     │
           ▼                     ▼                     ▼
    ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
    │  Attribute  │◄─────►│  Attribute  │◄─────►│  Attribute  │
    │   (Node)    │       │   (Node)    │       │   (Node)    │
    └──────┬──────┘       └──────┬──────┘       └──────┬──────┘
           │                     │                     │
           │   DEPENDS_ON        │   COMPUTES          │
           │     (Edge)          │    (Edge)           │
           ▼                     ▼                     ▼
    ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
    │    Rule     │       │    Rule     │       │    Rule     │
    │   (Node)    │       │   (Node)    │       │   (Node)    │
    └─────────────┘       └─────────────┘       └─────────────┘
```

---

## 5. Technology Stack Decisions

### 5.1 Database: Why Graph?

**Current Problem with PostgreSQL:**
- Rule dependencies require recursive CTEs or multiple queries
- JSONB queries for nested attribute paths are slow
- No native cycle detection in query language
- Scaling graph traversals requires application-level code

**Graph Database Comparison (Updated December 2024):**

| Database | Language | Strengths | Weaknesses | Best For |
|----------|----------|-----------|------------|----------|
| **DGraph** | Go | Production-proven (Fortune 500), 38% perf improvement in v24.1, Native GraphQL, 10k QPS 1-hop queries, BadgerDB storage | Not Rust-native (FFI overhead), Licensing complexity, Heavier deployment | **Scale & Reliability** |
| **IndraDB** | Rust | Native Rust (zero FFI), Embeddable, Facebook TAO-inspired, Pluggable backends (Postgres/Sled/RocksDB) | Limited query language, Smaller community, Less mature, No native GraphQL | **Single Binary Embed** |
| **SurrealDB** | Rust | Multi-model (graph+SQL+doc), Native Rust, Rich query language, Embeddable, Built-in auth | Newer (less battle-tested), Smaller community, Performance unproven at scale | **Flexibility & Features** |
| **Neo4j** | Java | Most mature, Large community, Cypher is powerful, Enterprise features | JVM overhead, Complex licensing, Not Rust-native | **Enterprise Legacy** |

**Detailed Analysis:**

#### DGraph (Recommended for Production)
- **Performance**: Benchmarks show 10k QPS for 1-hop queries at ~300ms p99, ~1000 QPS for 2-hop at ~0.95s p95
- **v24.1 Improvements**: Up to 3x faster count queries, 50% faster mutations with indexes, 99.5% faster count index insertions
- **Architecture**: Written in Go with BadgerDB (optimized for SSD), concurrent caching via Ristretto
- **Scale**: Used at terabyte-scale in production at Fortune 500 companies, Intuit Katlas, VMware Purser
- **Rust Integration**: Has `dgraph-rs` client crate - DB doesn't need to be in Rust
- **Meituan Benchmark**: Showed good performance but slightly behind NebulaGraph for certain relationship patterns

#### IndraDB (Recommended for Embedding)
- **Design**: Inspired by Facebook's TAO (graph datastore serving billions of requests)
- **Storage**: Pluggable - can use Sled (pure Rust), RocksDB, or PostgreSQL
- **Query Model**: Simple typed edges/vertices, JSON properties, multi-hop queries
- **Limitation**: No rich query language like Cypher/GraphQL - better for programmatic access
- **Best Use**: Embedded in Rust binary for low-latency, single-deployment scenarios

#### SurrealDB (Recommended for Flexibility)
- **Features**: Most feature-rich - SQL + Graph + Document + Events + Auth in one
- **Query Language**: Very expressive, supports complex traversals
- **Risk**: Newer project, less production validation at scale
- **Best Use**: When you need multi-model flexibility and can tolerate some risk

**Final Recommendation: DGraph** because:
1. **Production reliability** is critical for trading use case - can't afford data loss or downtime
2. **Native GraphQL** integrates well with AI tools and modern frontends
3. **Proven at terabyte scale** with sub-second query latency
4. **Rust client available** (`dgraph-rs`) - you get Rust-native client without DB being in Rust
5. **Active development** - v24.1 shows significant continued investment (38% perf improvement)
6. **Horizontal scaling** built-in for future growth

**Fallback Options:**
- **IndraDB** if single-binary deployment is critical (embed with Sled backend)
- **SurrealDB** if you want to experiment with multi-model and can accept newer tech risk

### 5.2 Core Engine: Why Rust?

**Current Kotlin/JVM Issues:**
- ~200ms cold start for Lambda/serverless
- GC pauses affect real-time rule evaluation
- Memory overhead (~500MB+ baseline)
- Not ideal for high-frequency trading signals

**Rust Benefits for Rule Engine:**

| Aspect | Improvement |
|--------|-------------|
| **Latency** | Sub-millisecond rule evaluation (vs 10-50ms JVM) |
| **Memory** | ~10-50MB for equivalent functionality |
| **Startup** | <10ms cold start |
| **Concurrency** | Zero-cost async with Tokio |
| **Safety** | Compile-time guarantees for rule graph integrity |

**Rust Ecosystem for This Project:**

```rust
// Core dependencies
dgraph-rs = "0.5"      // DGraph client (or indradb for embedded)
serde_json = "1.0"     // JSON handling
petgraph = "0.6"       // Graph algorithms (DAG, topo sort)
tonic = "0.10"         // gRPC server
tokio = "1.0"          // Async runtime
axum = "0.7"           // HTTP server (for REST fallback)

// Rule compilation (see Section 5.3)
cranelift = "0.104"    // JIT compiler for native code
cranelift-jit = "0.104"// JIT runtime

// Precision arithmetic (generic - used by any numeric rules)
rust_decimal = "1.32"  // Precise decimal arithmetic for financial calculations

// Note: No trading-specific crates - trading is configured via
// datatypes, enums, and rules like any other product template
```

### 5.3 Rule Execution: JSON Logic Performance Analysis

**CRITICAL INSIGHT**: JSON Logic is excellent for *storage* but problematic for *execution*.

#### The Problem with Interpreted JSON Logic

JSON Logic evaluation involves:

```
              INTERPRETED EXECUTION (Current - Slow)

JSON String: {"if": [{">": [{"var": "age"}, 60]}, ...]}
    │
    ▼ (every evaluation)
┌─────────────────────────────────────────────────────┐
│  1. Parse JSON string → AST nodes                   │  ~100ns
│  2. Match operation ("if", ">", "var", etc.)        │  ~50ns per op
│  3. Lookup variable in HashMap by string key        │  ~30ns per var
│  4. Dynamic type checking at each operation         │  ~20ns per check
│  5. Box/unbox values (heap allocation)              │  ~50ns per value
│  6. Recursive descent through nested structures     │  ~10ns per level
└─────────────────────────────────────────────────────┘
    │
    ▼
Total: ~500-5000ns per evaluation (depends on complexity)
```

**Why This Matters:**
- **Trading signals**: Need <1μs latency for high-frequency decisions
- **Bulk evaluation**: 10,000 policies × 50 rules each = 500K evaluations
- **Real-time quotes**: Premium calculation on every user interaction

#### The Solution: Compile on Load, Execute Native

```
              COMPILED EXECUTION (Proposed - Fast)

JSON Logic (Storage/Transport)
    │
    ▼ (on rule load - ONCE)
┌─────────────────────────────────────────────────────┐
│                   COMPILATION PHASE                  │
│                                                      │
│  1. Parse JSON Logic → Internal AST                 │
│  2. Type inference and validation                    │
│  3. Optimize AST (constant folding, dead code)      │
│  4. Generate bytecode OR native code                 │
│  5. Cache compiled form in memory                    │
└─────────────────────────────────────────────────────┘
    │
    ▼ (cached)
┌─────────────────────────────────────────────────────┐
│               EXECUTION PHASE (Fast)                 │
│                                                      │
│  - Direct variable access (no string lookup)        │
│  - No dynamic dispatch                               │
│  - No parsing overhead                               │
│  - CPU-cache friendly linear execution              │
└─────────────────────────────────────────────────────┘
    │
    ▼
Total: ~10-100ns per evaluation (10-100x faster)
```

#### Implementation Options Comparison

| Approach | Compilation Time | Execution Time | Complexity | Recommendation |
|----------|------------------|----------------|------------|----------------|
| **Optimized AST** | ~1μs | ~100-500ns | Low | MVP / Phase 1 |
| **Custom Bytecode VM** | ~10μs | ~50-100ns | Medium | Production / Phase 2 |
| **Cranelift JIT** | ~100μs | ~2-10ns | High | Trading / Phase 3 |

#### Recommended Approach: Tiered Compilation

```
                    TIERED COMPILATION STRATEGY

                         JSON Logic Input
                               │
                               ▼
                    ┌─────────────────────┐
                    │    Parse & Validate │
                    │    (shared step)    │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
       ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
       │   Tier 0    │  │   Tier 1    │  │   Tier 2    │
       │ Optimized   │  │  Bytecode   │  │ Cranelift   │
       │    AST      │  │     VM      │  │    JIT      │
       └─────────────┘  └─────────────┘  └─────────────┘
              │                │                │
              ▼                ▼                ▼
         ~100-500ns        ~50-100ns         ~2-10ns
         (cold rules)    (warm rules)    (hot rules)
```

**Tier Promotion Strategy:**
- **Tier 0 (Optimized AST)**: Default for all new rules, immediate availability
- **Tier 1 (Bytecode)**: Promote after 100 evaluations or explicit flag
- **Tier 2 (Cranelift JIT)**: Promote after 10,000 evaluations OR for trading rules

#### Bytecode VM Design

```rust
// Custom bytecode instruction set for rule evaluation
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    LoadConst(u16),      // Push constant from pool
    LoadVar(u16),        // Push variable by index (not string!)
    Pop,                  // Discard top of stack

    // Arithmetic
    Add, Sub, Mul, Div, Mod,

    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,

    // Logic
    And, Or, Not,

    // Control flow
    JumpIf(i16),         // Conditional jump (relative offset)
    Jump(i16),           // Unconditional jump

    // Special
    Call(u16),           // Call compiled sub-expression
    Return,              // Return result
}

// Compiled rule representation
pub struct CompiledRule {
    pub id: RuleId,
    pub bytecode: Vec<u8>,           // Compact bytecode
    pub constants: Vec<Value>,       // Constant pool
    pub variable_map: Vec<String>,   // Index → variable name
    pub tier: CompilationTier,
}

// Fast evaluation loop
impl CompiledRule {
    pub fn evaluate(&self, context: &EvalContext) -> Result<Value, EvalError> {
        let mut stack: SmallVec<[Value; 16]> = SmallVec::new();
        let mut pc: usize = 0;

        loop {
            match self.bytecode[pc] {
                OpCode::LoadVar(idx) => {
                    // Direct index access - no string hashing!
                    stack.push(context.vars[idx as usize].clone());
                    pc += 3;
                }
                OpCode::Gt => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(Value::Bool(a > b));
                    pc += 1;
                }
                OpCode::Return => return Ok(stack.pop().unwrap()),
                // ... other operations
            }
        }
    }
}
```

#### Cranelift JIT for Hot Rules (Trading)

For rules evaluated millions of times (trading signals), use Cranelift to compile to native machine code:

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};

pub struct JitCompiledRule {
    // Function pointer to native code
    pub func: fn(&[f64], &mut [f64]) -> bool,
    // Keep module alive
    _module: JITModule,
}

impl JitCompiledRule {
    pub fn compile(ast: &RuleAst) -> Result<Self, CompileError> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let mut module = JITModule::new(builder);

        // Translate AST to Cranelift IR
        let mut ctx = module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        // ... IR generation from AST ...

        // Compile to native code
        let func_id = module.declare_function("rule", Linkage::Local, &ctx.func.signature)?;
        module.define_function(func_id, &mut ctx)?;
        module.finalize_definitions()?;

        let code_ptr = module.get_finalized_function(func_id);
        let func: fn(&[f64], &mut [f64]) -> bool = unsafe { std::mem::transmute(code_ptr) };

        Ok(Self { func, _module: module })
    }

    #[inline(always)]
    pub fn evaluate(&self, inputs: &[f64], outputs: &mut [f64]) -> bool {
        // Direct native function call - nanosecond latency
        (self.func)(inputs, outputs)
    }
}
```

**Cranelift Performance Reference:**
- Compilation: ~72μs (vs LLVM's ~821μs) - 11x faster compilation
- Execution: ~2-3ns per simple operation
- Suitable for rules evaluated >10,000 times

#### JSON Logic Remains the Storage Format

**Why Keep JSON Logic:**
1. **Portability**: Can be evaluated by any language/platform
2. **Readability**: Human-understandable (with display_expression)
3. **Tooling**: AI can generate/manipulate it easily
4. **Serialization**: Easy to store in database, transfer via API
5. **Validation**: Well-defined semantics for testing

**The Hybrid Model:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RULE LIFECYCLE                                        │
│                                                                              │
│  ┌─────────┐    ┌─────────────┐    ┌────────────┐    ┌─────────────┐        │
│  │   AI    │───►│ JSON Logic  │───►│  Compile   │───►│   Execute   │        │
│  │ Generate│    │   (Store)   │    │ (On Load)  │    │  (Cached)   │        │
│  └─────────┘    └─────────────┘    └────────────┘    └─────────────┘        │
│                        │                                    │               │
│                        │                                    │               │
│                        ▼                                    ▼               │
│               ┌─────────────┐                      ┌─────────────┐          │
│               │   DGraph    │                      │  In-Memory  │          │
│               │  (persist)  │                      │   (cache)   │          │
│               └─────────────┘                      └─────────────┘          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.4 Migration Strategy: Kotlin → Rust

**Phase 1: Coexistence**
```
┌────────────────────────────────────────────────────────────┐
│                    Existing Kotlin API                      │
│                   (product-farm-api)                        │
└────────────────────────────┬───────────────────────────────┘
                             │ gRPC
                             ▼
┌────────────────────────────────────────────────────────────┐
│                  New Rust Rule Engine                       │
│              (rule-engine-rs service)                       │
└────────────────────────────────────────────────────────────┘
```

**Phase 2: Gradual Replacement**
- Port `json-logic` module to Rust first (pure computation)
- Port `rule-framework` module (DAG execution)
- Port API layer last (after UI is built)

**Phase 3: Full Rust**
- Single Rust binary with embedded SurrealDB
- Kotlin codebase archived as reference

---

## 6. AI-Powered Rule Management

### 6.1 Why AI for Rules?

The spec document identified a critical pain point:
> "Creating some other rules which have complex conditions... will not be feasible through the sheet. We need to have a UI."

**The insight:** Even a good UI is complex for non-technical users. Natural language is the ultimate interface.

### 6.2 AI Agent Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AI RULE MANAGEMENT AGENT                             │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        LLM (Claude/GPT-4)                           │    │
│  │                                                                      │    │
│  │  System Prompt:                                                      │    │
│  │  "You are a rule configuration expert. You help users create,       │    │
│  │   modify, and understand product rules. You have access to tools    │    │
│  │   for reading existing rules, creating new rules, validating        │    │
│  │   rule logic, and visualizing dependencies."                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                           TOOL SET                                   │    │
│  │                                                                      │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │    │
│  │  │ list_rules  │  │ create_rule │  │validate_rule│  │ visualize  │  │    │
│  │  │             │  │             │  │             │  │ _graph     │  │    │
│  │  │ Get rules   │  │ NL→JSON    │  │ Check cycles│  │ Generate   │  │    │
│  │  │ for product │  │ Logic       │  │ and types   │  │ Mermaid    │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │    │
│  │                                                                      │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │    │
│  │  │explain_rule │  │ clone_rule  │  │ test_rule   │  │ suggest    │  │    │
│  │  │             │  │             │  │             │  │ _optimize  │  │    │
│  │  │ JSON→NL     │  │ Copy with   │  │ Run with    │  │ Performance│  │    │
│  │  │ explanation │  │ modifications│ │ test inputs │  │ tips       │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Natural Language → JSON Logic Translation

**Example Interaction:**

```
User: "Create a rule that calculates premium as base_rate times coverage_amount,
       but if the customer age is over 60, add a 20% loading"

AI Agent: I'll create this rule for you. Let me break it down:

1. Base calculation: base_rate × coverage_amount
2. Condition: if age > 60, apply 1.2 multiplier

Here's the rule I'll create:

{
  "type": "PREMIUM_CALCULATION",
  "displayExpression": "IF age > 60 THEN base_rate × coverage_amount × 1.2
                        ELSE base_rate × coverage_amount",
  "expression": {
    "if": [
      {">": [{"var": "age"}, 60]},
      {"*": [{"var": "base_rate"}, {"var": "coverage_amount"}, 1.2]},
      {"*": [{"var": "base_rate"}, {"var": "coverage_amount"}]}
    ]
  },
  "inputAttributes": ["age", "base_rate", "coverage_amount"],
  "outputAttributes": ["premium"]
}

Shall I:
1. Validate this rule against your existing product?
2. Show how it connects to other rules in the dependency graph?
3. Test it with sample values?
```

### 6.4 AI Tools Implementation

```rust
// Tool definitions for the AI agent
pub enum RuleTool {
    ListRules { product_id: String },
    CreateRule {
        product_id: String,
        natural_language: String,
        rule_type: String,
    },
    ValidateRule { rule: Rule },
    ExplainRule { rule_id: String },
    TestRule {
        rule_id: String,
        test_inputs: HashMap<String, Value>,
    },
    VisualizeGraph {
        product_id: String,
        format: GraphFormat, // Mermaid, D3, DOT
    },
    SuggestOptimizations { product_id: String },
}

impl RuleTool {
    pub async fn execute(&self, engine: &RuleEngine) -> ToolResult {
        match self {
            Self::CreateRule { natural_language, .. } => {
                // Use LLM to translate NL to JSON Logic
                // Validate the generated rule
                // Return structured result
            }
            // ... other tools
        }
    }
}
```

### 6.5 AI-Assisted Features

| Feature | Description | Value |
|---------|-------------|-------|
| **Rule Explanation** | "Explain this rule in plain English" | Non-technical users can understand existing rules |
| **Impact Analysis** | "What happens if I change this attribute?" | Prevents breaking changes |
| **Suggestion Engine** | "Suggest rules for calculating X" | Accelerates configuration |
| **Anomaly Detection** | "This rule might create a cycle" | Proactive error prevention |
| **Documentation Gen** | Auto-generate rule documentation | Audit and compliance |

---

## 7. UX/UI Design Philosophy

### 7.1 Design Principles

1. **Progressive Disclosure** - Simple by default, powerful when needed
2. **Visual Rule Building** - Drag-and-drop nodes, not code editors
3. **Conversational Fallback** - Chat when UI gets complex
4. **Real-time Feedback** - Validate as you configure
5. **Graph-First Visualization** - Show dependencies, not tables

### 7.2 UI Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              RULE STUDIO                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐  ┌────────────────────────────────────────────────────┐   │
│  │              │  │                                                    │   │
│  │   PRODUCT    │  │              DEPENDENCY GRAPH VIEW                 │   │
│  │   SIDEBAR    │  │                                                    │   │
│  │              │  │       ┌───────┐                                    │   │
│  │  ▼ Products  │  │       │ age   │                                    │   │
│  │    ├ Health  │  │       └───┬───┘                                    │   │
│  │    ├ Motor   │  │           │                                        │   │
│  │    └ Term    │  │           ▼                                        │   │
│  │              │  │  ┌────────────────┐      ┌────────────────┐        │   │
│  │  ▼ Attributes│  │  │ age_loading    │─────►│ final_premium  │        │   │
│  │              │  │  └────────────────┘      └────────────────┘        │   │
│  │  ▼ Rules     │  │           ▲                      ▲                 │   │
│  │              │  │           │                      │                 │   │
│  │  ▼ Functions │  │  ┌────────┴───────┐    ┌────────┴───────┐         │   │
│  │              │  │  │ base_rate      │    │ coverage_amount│         │   │
│  │              │  │  └────────────────┘    └────────────────┘         │   │
│  └──────────────┘  └────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                         RULE EDITOR PANEL                             │   │
│  │                                                                        │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │   │
│  │  │   Visual    │  │    Code     │  │    Chat     │  │    Test     │   │   │
│  │  │   Editor    │  │   Editor    │  │  Assistant  │  │   Runner    │   │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │   │
│  │                                                                        │   │
│  │  IF  [age] > [60]  THEN                                               │   │
│  │      [base_premium] × [1.2]                                           │   │
│  │  ELSE                                                                  │   │
│  │      [base_premium]                                                    │   │
│  │                                                                        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  💬 Ask AI: "What if I add a discount for non-smokers?"              │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.3 Technology Stack for UI

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Framework** | React 18 + TypeScript | Industry standard, component ecosystem |
| **State** | Zustand or Jotai | Lightweight, graph-friendly state |
| **Graph Viz** | React Flow or D3.js | Interactive node-based editing |
| **Charts** | Recharts or Victory | Trading signals visualization |
| **Styling** | Tailwind CSS + shadcn/ui | Rapid, consistent UI |
| **Chat UI** | Vercel AI SDK | Streaming LLM responses |
| **Real-time** | Socket.io or WebSockets | Live rule updates |

### 7.4 Key UI Screens

1. **Dashboard** - Product overview, health metrics, recent changes
2. **Product Studio** - Configure attributes, rules, functionalities
3. **Graph Explorer** - Visual dependency navigation
4. **Rule Builder** - Visual + Code + Chat modes
5. **Test Playground** - Input values, see outputs
6. **Version History** - Diff rules, rollback changes
7. **Trading Console** - (for stock trading) Signal monitor, position tracker

---

## 8. Stock Trading Use Case

### 8.1 Mapping Trading to Product-FARM Concepts

| Trading Concept | Product-FARM Mapping |
|-----------------|---------------------|
| **Trading Strategy** | Product |
| **Market Data** | Input Attributes (price, volume, indicators) |
| **Trading Signal** | Output Attribute (buy/sell/hold) |
| **Entry/Exit Rules** | Rules with JSON Logic |
| **Risk Parameters** | Functionality constraints |
| **Position Size** | Computed Attribute |

### 8.2 Trading Configuration (Generic - No Custom Code)

**Philosophy:** Trading is just another product template. All trading concepts are defined through the existing generic system - no custom Rust types needed.

#### Step 1: Define Trading DataTypes (via API/UI)

```json
// POST /datatype - Define trading-specific datatypes dynamically
[
  { "name": "price", "type": "decimal", "description": "Price with 4 decimal precision" },
  { "name": "quantity", "type": "decimal", "description": "Share quantity (fractional ok)" },
  { "name": "percentage", "type": "decimal", "description": "0-100 percentage value" },
  { "name": "signal", "type": "enum", "description": "Trading signal type" },
  { "name": "timestamp", "type": "datetime", "description": "Market timestamp" }
]
```

#### Step 2: Define Trading Enumerations (via API/UI)

```json
// POST /productTemplate/TRADING/enum
[
  { "name": "SignalType", "value": ["BUY", "SELL", "HOLD", "SCALE_IN", "SCALE_OUT"] },
  { "name": "IndicatorType", "value": ["RSI", "MACD", "SMA", "EMA", "BOLLINGER", "ATR"] },
  { "name": "RuleCategory", "value": ["ENTRY", "EXIT", "STOP_LOSS", "TAKE_PROFIT", "POSITION_SIZE", "RISK"] },
  { "name": "TimeFrame", "value": ["1M", "5M", "15M", "1H", "4H", "1D", "1W"] }
]
```

#### Step 3: Create Trading Strategy as Product

```json
// PUT /product
{
  "productId": "momentum-strategy-v1",
  "templateType": "TRADING",
  "description": "RSI-based momentum trading strategy",
  "effectiveFrom": "2024-01-01T00:00:00Z",
  "expiryAt": "2025-12-31T23:59:59Z"
}
```

#### Step 4: Define Abstract Attributes (Reusable Templates)

```json
// PUT /product/momentum-strategy-v1/abstractAttribute
[
  // Market Data Inputs
  {
    "displayName": "current_price",
    "componentType": "MARKET_DATA",
    "datatype": "price",
    "tag": ["input", "realtime"],
    "description": "Current market price"
  },
  {
    "displayName": "rsi_14",
    "componentType": "INDICATOR",
    "datatype": "percentage",
    "tag": ["input", "indicator", "momentum"],
    "description": "14-period RSI value"
  },
  // Position State
  {
    "displayName": "entry_price",
    "componentType": "POSITION",
    "datatype": "price",
    "tag": ["state", "position"],
    "description": "Entry price of current position"
  },
  {
    "displayName": "holding_days",
    "componentType": "POSITION",
    "datatype": "int",
    "tag": ["state", "position"],
    "description": "Days since position opened"
  },
  // Computed Outputs
  {
    "displayName": "entry_signal",
    "componentType": "SIGNAL",
    "datatype": "signal",
    "enum": "SignalType",
    "tag": ["output", "signal"],
    "description": "Entry decision signal"
  },
  {
    "displayName": "exit_signal",
    "componentType": "SIGNAL",
    "datatype": "signal",
    "enum": "SignalType",
    "tag": ["output", "signal"],
    "description": "Exit decision signal"
  }
]
```

#### Step 5: Define Functionalities (Rule Categories)

```json
// PUT /product/momentum-strategy-v1/functionality
[
  {
    "name": "ENTRY_LOGIC",
    "description": "When to enter a position",
    "requiredAttributes": [
      { "abstractPath": "SIGNAL.entry_signal" }
    ]
  },
  {
    "name": "EXIT_LOGIC",
    "description": "When to exit a position",
    "requiredAttributes": [
      { "abstractPath": "SIGNAL.exit_signal" }
    ]
  },
  {
    "name": "RISK_MANAGEMENT",
    "description": "Stop loss and position sizing",
    "requiredAttributes": [
      { "abstractPath": "SIGNAL.stop_loss_triggered" },
      { "abstractPath": "POSITION.position_size" }
    ]
  }
]
```

#### Step 6: Create Rules via Generic System

Rules use the same JSON Logic as insurance - no trading-specific code:

```json
// PUT /product/momentum-strategy-v1/attribute (with rule)
{
  "displayName": "entry_signal",
  "rule": {
    "type": "ENTRY_LOGIC",
    "displayExpression": "BUY when RSI crosses above 30 from oversold",
    "inputAttribute": ["rsi_14", "rsi_14_prev"],
    "outputAttribute": ["entry_signal"],
    "description": "RSI oversold bounce entry"
  }
}
```

The rule's JSON Logic expression:
```json
{
  "if": [
    {"and": [
      {"<": [{"var": "rsi_14_prev"}, 30]},
      {">=": [{"var": "rsi_14"}, 30]}
    ]},
    "BUY",
    "HOLD"
  ]
}
```

### 8.3 Example: Stop-Loss Rule (Generic)

**Natural Language:**
> "If the current price drops 5% below my entry price, sell immediately.
> But if I've held for more than 30 days and price is above entry, use a trailing stop of 3%."

**AI generates rule using the generic system:**

```json
// PUT /product/momentum-strategy-v1/attribute
{
  "displayName": "stop_loss_signal",
  "rule": {
    "type": "RISK_MANAGEMENT",
    "displayExpression": "Complex stop-loss with time-based trailing",
    "inputAttribute": [
      "current_price", "entry_price", "holding_days", "highest_since_entry"
    ],
    "outputAttribute": ["stop_loss_signal"],
    "description": "Adaptive stop-loss: 5% fixed or 3% trailing after 30 days profit"
  }
}
```

**The JSON Logic expression (same engine as insurance):**

```json
{
  "if": [
    {"and": [
      {">": [{"var": "holding_days"}, 30]},
      {">": [{"var": "current_price"}, {"var": "entry_price"}]}
    ]},
    {"if": [
      {"<": [
        {"var": "current_price"},
        {"*": [{"var": "highest_since_entry"}, 0.97]}
      ]},
      "SELL",
      "HOLD"
    ]},
    {"if": [
      {"<": [
        {"var": "current_price"},
        {"*": [{"var": "entry_price"}, 0.95]}
      ]},
      "SELL",
      "HOLD"
    ]}
  ]
}
```

**Key Point:** This uses the exact same rule engine as insurance premium calculation. The only difference is the `templateType: "TRADING"` and the dynamically-defined datatypes/enums.

### 8.4 Real-Time Trading Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TRADING SIGNAL PIPELINE                               │
│                                                                              │
│   Market Data Feed          Rule Engine           Action Executor            │
│                                                                              │
│   ┌─────────────┐      ┌─────────────────┐      ┌─────────────────┐         │
│   │   Alpaca    │      │                 │      │                 │         │
│   │   Polygon   │─────►│   Rust Engine   │─────►│  Order Router   │         │
│   │   Yahoo     │      │                 │      │                 │         │
│   │   Binance   │      │  < 1ms eval     │      │  Paper/Live     │         │
│   └─────────────┘      └─────────────────┘      └─────────────────┘         │
│         │                      │                        │                    │
│         │                      │                        │                    │
│         ▼                      ▼                        ▼                    │
│   ┌─────────────┐      ┌─────────────────┐      ┌─────────────────┐         │
│   │ TimeSeries  │      │  Signal Log     │      │  Position       │         │
│   │ Database    │      │  (QuestDB)      │      │  Tracker        │         │
│   └─────────────┘      └─────────────────┘      └─────────────────┘         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.5 Trading UI Additions

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TRADING CONSOLE                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────┐  ┌─────────────────────────────┐   │
│  │         PRICE CHART                 │  │     ACTIVE SIGNALS          │   │
│  │                                     │  │                             │   │
│  │    📈 AAPL 189.45 (+1.2%)          │  │  ▲ BUY  AAPL  Entry: 185   │   │
│  │   ╱╲   ╱╲                          │  │  ▼ SELL MSFT  Stop hit      │   │
│  │  ╱  ╲ ╱  ╲                         │  │  ● HOLD GOOGL Trailing 3%   │   │
│  │ ╱    ╳    ╲___                     │  │                             │   │
│  │                                     │  │                             │   │
│  │  Entry ──────────────────          │  │                             │   │
│  │  Stop  ─ ─ ─ ─ ─ ─ ─ ─ ─          │  │                             │   │
│  └─────────────────────────────────────┘  └─────────────────────────────┘   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    RULE PERFORMANCE                                  │    │
│  │                                                                      │    │
│  │  Stop-Loss Rule:  Triggered 12x | Saved $2,340 | Avg loss: -4.2%    │    │
│  │  Entry Rule:      Triggered 8x  | Win rate: 62% | Avg gain: +8.1%   │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  💬 "Create a rule that buys when RSI crosses above 30 from oversold"       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Roadmap

### Phase 1: Foundation (Rust Core Engine)

**Goal:** Replace Kotlin rule engine with Rust, prove performance

| Task | Description | Priority |
|------|-------------|----------|
| 1.1 | Set up Rust project structure with Cargo workspace | P0 |
| 1.2 | Implement JSON Logic evaluator in Rust | P0 |
| 1.3 | Port DAG builder and topological sort | P0 |
| 1.4 | Create gRPC service definition (.proto) | P0 |
| 1.5 | Build gRPC server with Tonic | P0 |
| 1.6 | Benchmark against Kotlin implementation | P0 |
| 1.7 | Create Kotlin client adapter for gRPC | P1 |

**Deliverable:** Rust rule engine service, callable from existing Kotlin API

### Phase 2: Database Migration (DGraph)

**Goal:** Replace PostgreSQL with graph-native database

| Task | Description | Priority |
|------|-------------|----------|
| 2.1 | Design DGraph schema (GraphQL types, edges) | P0 |
| 2.2 | Write migration scripts from PostgreSQL | P0 |
| 2.3 | Implement Rust repository layer using dgraph-rs | P0 |
| 2.4 | Add graph traversal queries (dependencies, impact) | P0 |
| 2.5 | Implement real-time subscriptions | P1 |
| 2.6 | Performance testing with large graphs | P1 |
| 2.7 | Add bytecode persistence (see Section 5.3) | P1 |

**Deliverable:** Full data layer on DGraph with graph queries and compiled rule storage

### Phase 3: AI Agent Integration

**Goal:** Natural language rule management

| Task | Description | Priority |
|------|-------------|----------|
| 3.1 | Design AI tool specifications | P0 |
| 3.2 | Implement NL → JSON Logic translator | P0 |
| 3.3 | Build rule explanation generator | P0 |
| 3.4 | Create validation and testing tools | P0 |
| 3.5 | Integrate with Claude/OpenAI API | P0 |
| 3.6 | Build conversation history and context | P1 |

**Deliverable:** AI agent that can create, explain, and test rules

### Phase 4: Web UI (React Dashboard)

**Goal:** Beautiful, graph-first UI

| Task | Description | Priority |
|------|-------------|----------|
| 4.1 | Set up React + TypeScript + Vite project | P0 |
| 4.2 | Implement product/attribute/rule CRUD | P0 |
| 4.3 | Build graph visualization with React Flow | P0 |
| 4.4 | Create visual rule builder | P1 |
| 4.5 | Add AI chat interface | P0 |
| 4.6 | Implement test playground | P1 |
| 4.7 | Add version history and diff view | P2 |

**Deliverable:** Fully functional web dashboard

### Phase 5: Trading Use Case (Generic Configuration)

**Goal:** Configure trading as a product template - NO custom code

| Task | Description | Priority |
|------|-------------|----------|
| 5.1 | Create TRADING product template with datatypes/enums (via API) | P0 |
| 5.2 | Build market data adapter (Alpaca/Polygon → attribute values) | P0 |
| 5.3 | Add time-series storage for signals (QuestDB/ScyllaDB) | P1 |
| 5.4 | Build trading console UI (generic, template-driven) | P1 |
| 5.5 | Add paper trading adapter (maps signals → broker API) | P1 |
| 5.6 | Implement backtesting framework (replay historical data) | P2 |

**Deliverable:** Trading configured via generic system, no trading-specific engine code

**Note:** All trading concepts (signals, indicators, stop-loss) are defined as:
- DataTypes → via `/datatype` API
- Enums → via `/productTemplate/TRADING/enum` API
- Attributes → via `/abstractAttribute` API
- Rules → via standard JSON Logic rules

### Phase 6: Production Hardening

**Goal:** Enterprise-ready system

| Task | Description | Priority |
|------|-------------|----------|
| 6.1 | Add authentication (OAuth2/OIDC) | P0 |
| 6.2 | Implement audit logging | P0 |
| 6.3 | Add rate limiting and quotas | P1 |
| 6.4 | Set up monitoring (Prometheus/Grafana) | P0 |
| 6.5 | Create deployment automation (Docker/K8s) | P1 |
| 6.6 | Write comprehensive documentation | P1 |

**Deliverable:** Production-deployable system

---

## 10. What NOT To Do

### 10.1 Anti-Patterns to Avoid

| Don't | Why | Do Instead |
|-------|-----|------------|
| **Don't** try to "fix" the Kotlin code incrementally | Sunk cost; Rust rewrite is cleaner | Use Kotlin as reference only |
| **Don't** use PostgreSQL for graph queries | Will hit performance walls | Adopt graph database from start |
| **Don't** build UI before engine is solid | UI will need rewrites | Engine first, UI second |
| **Don't** over-engineer the AI integration | Start simple, iterate | Basic tools first, then sophistication |
| **Don't** hardcode domain-specific types (TradingSignal, InsurancePremium) | Defeats the purpose of generic system | Define all types via datatype/enum APIs |
| **Don't** rewrite JSON Logic parsing | Well-tested; focus on execution speed | Parse with serde, compile to bytecode |
| **Don't** try to support every trading platform | Feature creep | Start with one (Alpaca) |
| **Don't** skip the test playground | Users need to experiment safely | Build it in Phase 4 |

### 10.2 Scope Limitations

**In Scope:**
- Rule definition and evaluation
- Dependency graph management
- AI-assisted configuration
- Trading signal generation
- Web-based UI

**Out of Scope (for now):**
- Order execution (use existing brokers)
- Portfolio management beyond signals
- Multi-tenant SaaS (can add later)
- Mobile app
- Real-time collaborative editing

---

## 11. Assumptions & Risks

### 11.1 Assumptions

| Assumption | Impact if Wrong | Mitigation |
|------------|-----------------|------------|
| SurrealDB is stable enough for production | Would need to switch DBs | Monitor SurrealDB releases, have Neo4j backup plan |
| Rust JSON Logic libs are complete | Would need custom implementation | Evaluate `json-logic-rs` thoroughly first |
| Users will adopt AI chat interface | UI investment wasted | Build visual editor as fallback |
| Sub-millisecond latency is achievable | Trading use case compromised | Benchmark early in Phase 1 |
| LLM can reliably generate JSON Logic | AI features less useful | Use fine-tuned models, add validation |

### 11.2 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| SurrealDB performance at scale | Medium | High | Benchmark with 100K+ rules early |
| LLM hallucination in rule generation | High | Medium | Always validate, human review |
| Rust learning curve | Medium | Medium | Start with experienced Rust dev or contractor |
| Graph visualization performance | Medium | Low | Virtualize large graphs, paginate |
| Real-time trading latency | Low | High | Use dedicated infrastructure |

### 11.3 Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep | High | High | Strict phase gates, MVP focus |
| Single developer bottleneck | Medium | High | Document everything, modular design |
| Changing requirements | Medium | Medium | Flexible architecture, iterative delivery |

---

## 12. Success Metrics

### 12.1 Technical Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Rule evaluation latency | < 1ms p99 | Prometheus histogram |
| Graph query latency | < 10ms p99 | Database metrics |
| System startup time | < 1 second | Startup benchmark |
| Memory usage | < 100MB baseline | Runtime monitoring |
| Test coverage | > 80% | CI/CD reports |

### 12.2 User Experience Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Rule creation time | < 5 minutes | User sessions |
| AI assistance accuracy | > 90% valid rules | Validation pass rate |
| User task completion | > 95% success | Analytics |
| Learning curve | Productive in < 1 hour | User studies |

### 12.3 Trading Metrics (if applicable)

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Signal generation latency | < 100ms from data | End-to-end timing |
| Rule trigger accuracy | > 99% | Audit logs vs expected |
| Backtest throughput | 1M candles/second | Benchmark |

---

## Appendix A: Existing Code to Preserve

These components from the current Kotlin codebase should be referenced:

1. **Domain Model Definitions** - `product-farm-api/src/.../entity/` - Entity relationships
2. **JSON Logic Operations** - `json-logic/src/` - Operation implementations
3. **DAG Algorithms** - `rule-framework/src/.../DependencyGraph.kt` - Graph logic
4. **API Contracts** - `product-farm-api/src/.../api/` - Request/Response DTOs
5. **Validation Logic** - `product-farm-api/src/.../validator/` - Business rules

---

## Appendix B: Database Schema (SurrealDB)

```surql
-- Product node
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD id ON product TYPE string;
DEFINE FIELD status ON product TYPE string;
DEFINE FIELD effective_from ON product TYPE datetime;
DEFINE FIELD expiry_at ON product TYPE datetime;
DEFINE FIELD template_type ON product TYPE string;
DEFINE FIELD description ON product TYPE option<string>;
DEFINE INDEX product_id ON product FIELDS id UNIQUE;

-- Attribute node
DEFINE TABLE attribute SCHEMAFULL;
DEFINE FIELD path ON attribute TYPE string;
DEFINE FIELD abstract_path ON attribute TYPE string;
DEFINE FIELD value ON attribute TYPE option<object>;
DEFINE FIELD type ON attribute TYPE string; -- STATIC or DYNAMIC
DEFINE INDEX attr_path ON attribute FIELDS path UNIQUE;

-- Rule node
DEFINE TABLE rule SCHEMAFULL;
DEFINE FIELD id ON rule TYPE string;
DEFINE FIELD type ON rule TYPE string;
DEFINE FIELD display_expression ON rule TYPE string;
DEFINE FIELD expression ON rule TYPE object; -- JSON Logic
DEFINE FIELD description ON rule TYPE option<string>;
DEFINE INDEX rule_id ON rule FIELDS id UNIQUE;

-- Edges
DEFINE TABLE belongs_to SCHEMAFULL; -- attribute -> product
DEFINE TABLE computes SCHEMAFULL;   -- rule -> attribute (output)
DEFINE TABLE depends_on SCHEMAFULL; -- rule -> attribute (input)
DEFINE TABLE derived_from SCHEMAFULL; -- attribute -> abstract_attribute
```

---

## Appendix C: Rust Project Structure

```
product-farm-rs/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── core/                     # Domain types, traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── product.rs
│   │       ├── attribute.rs
│   │       ├── rule.rs
│   │       └── types.rs
│   ├── json-logic/               # JSON Logic evaluator
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── evaluator.rs
│   │       └── operations/
│   ├── rule-engine/              # DAG execution
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dag.rs
│   │       ├── executor.rs
│   │       └── cache.rs
│   ├── persistence/              # SurrealDB layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository.rs
│   │       └── migrations.rs
│   ├── api/                      # gRPC + REST server
│   │   ├── Cargo.toml
│   │   ├── proto/
│   │   │   └── rule_engine.proto
│   │   └── src/
│   │       ├── main.rs
│   │       ├── grpc.rs
│   │       └── rest.rs
│   └── trading/                  # Trading extensions
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── market_data.rs
│           ├── signals.rs
│           └── indicators.rs
├── web/                          # React frontend
│   ├── package.json
│   └── src/
└── ai-agent/                     # AI tools (Python/Node)
    └── ...
```

---

**Document Version:** 1.0
**Last Updated:** December 2024
**Next Review:** After Phase 1 completion
