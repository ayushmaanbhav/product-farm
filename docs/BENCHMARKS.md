---
layout: default
title: Performance Benchmarks
---

# Performance Benchmarks

Comprehensive performance data for Product-FARM's rule evaluation engine, demonstrating enterprise-grade throughput and latency characteristics.

---

## Executive Summary

| Metric | Value |
|--------|-------|
| **Rule Evaluation (Bytecode)** | ~330ns |
| **Single-thread Throughput** | 3M+ evals/sec |
| **Multi-thread Throughput** | 22M+ evals/sec |
| **Large-Scale Execution** | 96,000 rules/sec (1M rules) |
| **100k Chain Execution** | 907ms |
| **Compilation Speedup** | 3.5x |
| **Test Coverage** | 85% |
| **Backend Tests** | 551+ |
| **E2E Test Suites** | 9 |

<div class="callout callout-performance">
<strong>Production-Ready:</strong> These benchmarks demonstrate Product-FARM's readiness for high-throughput production workloads, from financial calculations to real-time pricing engines.
</div>

---

## Methodology

### Hardware Specifications

Benchmarks were conducted on:

| Component | Specification |
|-----------|--------------|
| CPU | AMD Ryzen 9 5900X (12 cores, 24 threads) |
| RAM | 64GB DDR4-3600 |
| Storage | NVMe SSD |
| OS | Ubuntu 22.04 LTS |
| Rust | 1.75+ (release build) |

### Test Environment

- **DGraph**: v23.x running locally
- **Connection**: Unix socket (minimal network overhead)
- **Cache**: Warm cache (preloaded data)
- **Compilation**: Release mode with LTO

### Measurement Approach

- **Tool**: Criterion.rs microbenchmark framework
- **Iterations**: 10,000+ per benchmark
- **Warmup**: 1,000 iterations before measurement
- **Statistics**: Mean, median, standard deviation reported

---

## Rule Evaluation Benchmarks

### Simple Rules

Single-operation rules (arithmetic, comparison):

| Mode | Time | Throughput | Memory |
|------|------|------------|--------|
| AST (Tier 0) | ~1.15μs | 870K/sec | 128 bytes |
| Bytecode (Tier 1) | ~330ns | 3M/sec | 64 bytes |
| **Improvement** | **3.5x** | **3.5x** | **50%** |

**Example Rule:**
```json
{
  "expression": { "*": [{"var": "base_rate"}, {"var": "factor"}] }
}
```

### Conditional Rules

Rules with branching logic:

| Complexity | AST | Bytecode | Improvement |
|------------|-----|----------|-------------|
| Single if/else | 1.8μs | 450ns | 4.0x |
| Nested conditions (3 levels) | 3.2μs | 680ns | 4.7x |
| Multiple conditions (5 branches) | 4.1μs | 820ns | 5.0x |

**Example Rule:**
```json
{
  "expression": {
    "if": [
      {"<": [{"var": "age"}, 25]}, {"*": [{"var": "base"}, 1.5]},
      {"<": [{"var": "age"}, 35]}, {"*": [{"var": "base"}, 1.2]},
      {"*": [{"var": "base"}, 1.0]}
    ]
  }
}
```

### Complex DAG Rules

Multi-level dependency chains:

| DAG Levels | Rules | Sequential | Parallel | Speedup |
|------------|-------|------------|----------|---------|
| 3 | 10 | 11.5μs | 4.2μs | 2.7x |
| 5 | 25 | 28.8μs | 8.1μs | 3.6x |
| 7 | 50 | 57.5μs | 12.4μs | 4.6x |
| 10 | 100 | 115μs | 21.3μs | 5.4x |

<div class="callout callout-info">
<strong>Parallel Scaling:</strong> Parallel execution speedup improves with DAG depth as more rules can execute concurrently within each level.
</div>

### Large-Scale Stress Tests

Production-scale benchmarks on **16-core Linux system** using rayon parallel execution:

| Test Pattern | Rules | Variables | Levels | Time | Throughput |
|--------------|-------|-----------|--------|------|------------|
| 100k Chain | 100,000 | 100,001 | 100,000 | 907ms | 110k/sec |
| 10k Diamond | 10,002 | 10,003 | 3 | 5ms | 2M/sec |
| 100×1000 Lattice | 100,001 | 100,002 | 1,000 | 445ms | 225k/sec |
| 10×10k Cascades | 100,000 | 100,010 | 10,000 | 650ms | 154k/sec |
| Tree (depth 8, branch 4) | 87,382 | 87,383 | 9 | 34ms | 2.6M/sec |
| 100 Mesh Groups (50 nodes) | 5,100 | 5,200 | 51 | 3ms | 1.7M/sec |
| Independent Rules | 597,515 | 598,515 | 1 | 445ms | 1.3M/sec |
| **Full Combined Test** | **1,000,000** | **1,001,114** | **100,000** | **10.4s** | **96k/sec** |

**Test Environment:**
- CPU: 16 cores (Linux 6.8.0)
- RAM: 64GB
- Rust: stable (release mode with LTO)
- Parallelism: rayon with 16 worker threads

**Total tests passing: 551 across workspace**

---

## Throughput Benchmarks

### Single-Threaded Performance

Continuous evaluation on a single thread:

| Mode | Throughput | Notes |
|------|-----------|-------|
| AST Interpretation | 870,000 evals/sec | No compilation overhead |
| Bytecode Execution | 3,030,000 evals/sec | Pre-compiled rules |
| Mixed (50% hot) | 1,850,000 evals/sec | Realistic workload |

### Multi-Threaded Performance

Parallel evaluation across CPU cores:

| Threads | AST | Bytecode | Efficiency |
|---------|-----|----------|------------|
| 1 | 870K/sec | 3.0M/sec | 100% |
| 4 | 3.4M/sec | 11.5M/sec | 96% |
| 8 | 6.5M/sec | 22.0M/sec | 92% |
| 12 | 9.0M/sec | 30.0M/sec | 83% |
| 24 | 11.2M/sec | 35.0M/sec | 65% |

<div class="callout callout-tip">
<strong>Scaling Tip:</strong> For optimal efficiency, use thread count equal to physical cores. Hyperthreading provides diminishing returns for CPU-bound rule evaluation.
</div>

### Throughput vs Latency Trade-offs

| Priority | Configuration | Throughput | P99 Latency |
|----------|--------------|-----------|-------------|
| Throughput | Batch, 8 threads | 22M/sec | 5ms |
| Balanced | Batch, 4 threads | 11M/sec | 1ms |
| Latency | Single, bytecode | 3M/sec | 500μs |
| Ultra-low | Single, hot cache | 3M/sec | 350μs |

---

## Memory Profile

### Cache Memory Usage

Memory consumption by cache component:

| Cache | Size | Memory | Per Entry |
|-------|------|--------|-----------|
| Products | 100 | ~10MB | ~100KB |
| Attributes | 10,000 | ~80MB | ~8KB |
| Rules | 10,000 | ~120MB | ~12KB |
| Compiled | 10,000 | ~50MB | ~5KB |
| **Total** | - | **~260MB** | - |

### Compilation Overhead

Memory and time for bytecode compilation:

| Rules | Compile Time | Memory Delta |
|-------|-------------|--------------|
| 10 | 2ms | +50KB |
| 100 | 18ms | +500KB |
| 1,000 | 180ms | +5MB |
| 10,000 | 1.8s | +50MB |

### Garbage Collection

Product-FARM uses Rust's ownership model - no GC pauses:

- **Zero GC pauses** - Deterministic memory management
- **Predictable latency** - No stop-the-world events
- **Memory safety** - Compile-time guarantees

---

## Comparison with Alternatives

### vs. Hardcoded Logic

| Metric | Product-FARM | Hardcoded |
|--------|-------------|-----------|
| Evaluation Time | ~330ns | ~50ns |
| Change Deployment | Seconds | Hours/Days |
| Non-technical Changes | Yes | No |
| Audit Trail | Built-in | Manual |
| A/B Testing | Native | Complex |

### vs. Other Rule Engines

| Feature | Product-FARM | Engine A | Engine B |
|---------|-------------|----------|----------|
| Evaluation Time | ~330ns | ~10μs | ~50μs |
| DAG Support | Native | Plugin | No |
| Visual Builder | Yes | Partial | No |
| Bytecode Compilation | Yes | No | Yes |
| gRPC API | Yes | No | Yes |

<div class="callout callout-info">
<strong>Note:</strong> Direct comparisons vary based on rule complexity and use case. Always benchmark with your specific workload.
</div>

---

## Test Coverage

### Backend Tests

| Crate | Tests | Notes |
|-------|-------|-------|
| core | 19 | Value types, serialization |
| json-logic | 358 | VM, compiler, evaluator, iter_eval |
| farmscript | 21 | Parser, interpreter |
| rule-engine | 60 | DAG, executor, context, 1M stress tests |
| yaml-loader | 65 | Schema, registry, interpreter |
| llm-evaluator | 18 | Claude, Ollama providers |
| persistence | 10 | DGraph operations |
| api | 31 | REST, gRPC, validation |
| **Total** | **551** | All passing |

### E2E Test Suites

| Suite | Tests | Focus |
|-------|-------|-------|
| Product Management | 15 | CRUD operations |
| Rule Evaluation | 12 | Expression evaluation |
| DAG Execution | 8 | Dependency resolution |
| API Endpoints | 20 | REST/gRPC validation |
| UI Workflows | 25 | User journeys |
| Performance | 10 | Load testing |
| Security | 8 | Auth/permissions |
| Data Integrity | 6 | Persistence |
| Edge Cases | 12 | Error handling |
| **Total** | **116** (9 suites) | - |

---

## Running Your Own Benchmarks

### Quick Benchmark

Run the standard benchmark suite:

```bash
cd backend
cargo bench
```

### Specific Benchmarks

Target specific benchmark groups:

```bash
# Rule evaluation benchmarks
cargo bench --bench evaluation

# DAG execution benchmarks
cargo bench --bench dag

# Throughput benchmarks
cargo bench --bench throughput
```

### Custom Benchmarks

Create benchmarks for your specific rules:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use product_farm::evaluate;

fn my_benchmark(c: &mut Criterion) {
    let product = load_product("my-product");
    let inputs = json!({"age": 30, "income": 75000});

    c.bench_function("my_rule_evaluation", |b| {
        b.iter(|| evaluate(&product, &inputs))
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

### Interpreting Results

Criterion output example:

```
rule_evaluation/bytecode
                        time:   [325.42 ns 330.15 ns 335.21 ns]
                        thrpt:  [2.9834 Melem/s 3.0290 Melem/s 3.0728 Melem/s]
                 change: [-1.2% +0.3% +1.8%] (p = 0.72 > 0.05)
                        No change in performance detected.
```

- **time**: [lower, mean, upper] confidence interval
- **thrpt**: Throughput (millions of evaluations per second)
- **change**: Comparison to baseline (if available)

---

## Optimization Recommendations

### For Maximum Throughput

1. **Enable bytecode compilation** for all production rules
2. **Use batch evaluation** for multiple inputs
3. **Scale threads** to physical core count
4. **Increase cache sizes** for large rule sets
5. **Use gRPC** instead of REST for evaluation

### For Minimum Latency

1. **Warm the cache** before traffic arrives
2. **Pre-compile rules** during product activation
3. **Use single-thread mode** for predictable latency
4. **Monitor P99 latency** not just mean
5. **Avoid GC** (Rust handles this automatically)

### For Memory Efficiency

1. **Tune cache sizes** based on working set
2. **Use lazy compilation** for infrequently used rules
3. **Monitor memory usage** with metrics
4. **Set appropriate limits** per product

---

## Next Steps

- **[Features](/FEATURES)** - Deep dive into capabilities
- **[Architecture](/ARCHITECTURE)** - System design
- **[API Reference](/API_REFERENCE)** - Complete API docs
- **[Quick Start](/QUICK_START)** - Get running in 5 minutes
