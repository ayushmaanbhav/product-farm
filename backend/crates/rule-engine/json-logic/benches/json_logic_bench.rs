//! Benchmarks for JSON Logic evaluation
//!
//! Run with: cargo bench -p product-farm-json-logic

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use product_farm_json_logic::{compile, evaluate, Evaluator};
use serde_json::json;

/// Benchmark simple arithmetic operations
fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    // Simple addition
    let expr = json!({"+": [1, 2]});
    let data = json!({});
    group.bench_function("add_constants", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Variable addition
    let expr = json!({"+": [{"var": "x"}, {"var": "y"}]});
    let data = json!({"x": 100, "y": 200});
    group.bench_function("add_variables", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Complex arithmetic
    let expr = json!({
        "*": [
            {"+": [{"var": "a"}, {"var": "b"}]},
            {"-": [{"var": "c"}, {"var": "d"}]}
        ]
    });
    let data = json!({"a": 10, "b": 20, "c": 50, "d": 10});
    group.bench_function("complex_arithmetic", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    group.finish();
}

/// Benchmark conditional operations
fn bench_conditionals(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditionals");

    // Simple if
    let expr = json!({
        "if": [
            {">": [{"var": "x"}, 10]},
            "big",
            "small"
        ]
    });
    let data = json!({"x": 15});
    group.bench_function("simple_if", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Nested if (3 levels)
    let expr = json!({
        "if": [
            {">": [{"var": "age"}, 60]}, "senior",
            {">": [{"var": "age"}, 18]}, "adult",
            "minor"
        ]
    });
    let data = json!({"age": 35});
    group.bench_function("nested_if_3", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Deeply nested if (5 levels)
    let expr = json!({
        "if": [
            {">": [{"var": "score"}, 90]}, "A",
            {">": [{"var": "score"}, 80]}, "B",
            {">": [{"var": "score"}, 70]}, "C",
            {">": [{"var": "score"}, 60]}, "D",
            "F"
        ]
    });
    let data = json!({"score": 75});
    group.bench_function("nested_if_5", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    group.finish();
}

/// Benchmark array operations
fn bench_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("arrays");

    // Map operation
    let expr = json!({
        "map": [
            {"var": "items"},
            {"*": [{"var": ""}, 2]}
        ]
    });
    let data = json!({"items": [1, 2, 3, 4, 5]});
    group.bench_function("map_5_items", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Larger map
    let data = json!({"items": (1..=100).collect::<Vec<_>>()});
    group.bench_function("map_100_items", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Filter operation
    let expr = json!({
        "filter": [
            {"var": "items"},
            {">": [{"var": ""}, 50]}
        ]
    });
    let data = json!({"items": (1..=100).collect::<Vec<_>>()});
    group.bench_function("filter_100_items", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Reduce (sum)
    let expr = json!({
        "reduce": [
            {"var": "items"},
            {"+": [{"var": "accumulator"}, {"var": "current"}]},
            0
        ]
    });
    let data = json!({"items": (1..=100).collect::<Vec<_>>()});
    group.bench_function("reduce_sum_100", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    group.finish();
}

/// Benchmark cached vs uncached evaluation
fn bench_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching");

    let expr = json!({
        "*": [
            {"+": [{"var": "a"}, {"var": "b"}]},
            {"if": [
                {">": [{"var": "c"}, 0]},
                {"var": "c"},
                1
            ]}
        ]
    });
    let data = json!({"a": 10, "b": 20, "c": 5});

    // Uncached (parse + evaluate each time)
    group.bench_function("uncached", |b| {
        b.iter(|| evaluate(black_box(&expr), black_box(&data)))
    });

    // Cached (compile once, evaluate multiple times)
    let compiled = compile(&expr).unwrap();
    let mut evaluator = Evaluator::new();
    group.bench_function("cached", |b| {
        b.iter(|| evaluator.evaluate_cached(black_box(&compiled), black_box(&data)))
    });

    group.finish();
}

/// Benchmark with varying complexity
fn bench_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("complexity");

    for depth in [1, 5, 10, 20] {
        let expr = create_nested_expression(depth);
        let data = json!({"x": 1});

        group.bench_with_input(
            BenchmarkId::new("nested_depth", depth),
            &(expr, data),
            |b, (expr, data)| {
                b.iter(|| evaluate(black_box(expr), black_box(data)))
            },
        );
    }

    group.finish();
}

/// Create a nested arithmetic expression of given depth
fn create_nested_expression(depth: usize) -> serde_json::Value {
    if depth == 0 {
        json!({"var": "x"})
    } else {
        json!({
            "+": [
                create_nested_expression(depth - 1),
                1
            ]
        })
    }
}

/// Benchmark realistic trading rule evaluation
fn bench_trading_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("trading_rules");

    // Simple RSI signal
    let rsi_signal = json!({
        "if": [
            {"<": [{"var": "rsi"}, 30]}, "BUY",
            {">": [{"var": "rsi"}, 70]}, "SELL",
            "HOLD"
        ]
    });

    // Complex entry logic with multiple conditions
    let entry_logic = json!({
        "if": [
            {"and": [
                {"<": [{"var": "rsi"}, 30]},
                {">": [{"var": "price"}, {"var": "sma_50"}]},
                {">": [{"var": "volume"}, {"*": [{"var": "avg_volume"}, 1.5]}]}
            ]},
            "STRONG_BUY",
            {"and": [
                {"<": [{"var": "rsi"}, 40]},
                {">": [{"var": "price"}, {"var": "sma_50"}]}
            ]},
            "BUY",
            {"and": [
                {">": [{"var": "rsi"}, 70]},
                {"<": [{"var": "price"}, {"var": "sma_50"}]},
                {">": [{"var": "volume"}, {"*": [{"var": "avg_volume"}, 1.5]}]}
            ]},
            "STRONG_SELL",
            {"and": [
                {">": [{"var": "rsi"}, 60]},
                {"<": [{"var": "price"}, {"var": "sma_50"}]}
            ]},
            "SELL",
            "HOLD"
        ]
    });

    // Position sizing
    let position_size = json!({
        "*": [
            {"var": "account_balance"},
            {"/": [{"var": "risk_percent"}, 100]},
            {"if": [
                {"==": [{"var": "signal"}, "STRONG_BUY"]}, 2.0,
                {"==": [{"var": "signal"}, "STRONG_SELL"]}, 2.0,
                1.0
            ]}
        ]
    });

    let market_data = json!({
        "rsi": 25.5,
        "price": 105.50,
        "sma_50": 100.0,
        "volume": 1500000,
        "avg_volume": 1000000,
        "account_balance": 100000,
        "risk_percent": 2.0,
        "signal": "STRONG_BUY"
    });

    group.bench_function("rsi_signal", |b| {
        b.iter(|| evaluate(black_box(&rsi_signal), black_box(&market_data)))
    });

    group.bench_function("entry_logic", |b| {
        b.iter(|| evaluate(black_box(&entry_logic), black_box(&market_data)))
    });

    group.bench_function("position_size", |b| {
        b.iter(|| evaluate(black_box(&position_size), black_box(&market_data)))
    });

    group.finish();
}

/// Benchmark insurance premium calculation
fn bench_insurance_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("insurance_rules");

    // Age factor calculation
    let age_factor = json!({
        "if": [
            {">": [{"var": "age"}, 60]}, 1.5,
            {">": [{"var": "age"}, 50]}, 1.3,
            {">": [{"var": "age"}, 40]}, 1.1,
            {"<": [{"var": "age"}, 25]}, 1.2,
            1.0
        ]
    });

    // Coverage multiplier
    let coverage_multiplier = json!({
        "*": [
            {"var": "base_rate"},
            {"+": [
                1,
                {"if": [{"var": "smoker"}, 0.5, 0]},
                {"if": [{">": [{"var": "bmi"}, 30]}, 0.3, 0]},
                {"if": [{"var": "hazardous_occupation"}, 0.4, 0]}
            ]}
        ]
    });

    // Full premium calculation
    let premium_calc = json!({
        "*": [
            {"var": "coverage_amount"},
            {"/": [{"var": "rate_per_1000"}, 1000]},
            {"if": [
                {">": [{"var": "age"}, 60]}, 1.5,
                {">": [{"var": "age"}, 50]}, 1.3,
                1.0
            ]},
            {"+": [
                1,
                {"if": [{"var": "smoker"}, 0.5, 0]},
                {"if": [{">": [{"var": "bmi"}, 30]}, 0.3, 0]}
            ]}
        ]
    });

    let customer_data = json!({
        "age": 45,
        "smoker": false,
        "bmi": 28.5,
        "hazardous_occupation": false,
        "base_rate": 100,
        "coverage_amount": 500000,
        "rate_per_1000": 5.50
    });

    group.bench_function("age_factor", |b| {
        b.iter(|| evaluate(black_box(&age_factor), black_box(&customer_data)))
    });

    group.bench_function("coverage_multiplier", |b| {
        b.iter(|| evaluate(black_box(&coverage_multiplier), black_box(&customer_data)))
    });

    group.bench_function("full_premium", |b| {
        b.iter(|| evaluate(black_box(&premium_calc), black_box(&customer_data)))
    });

    group.finish();
}

/// Benchmark tiered compilation (AST vs Bytecode)
fn bench_tiered_compilation(c: &mut Criterion) {
    use product_farm_json_logic::{parse, RuleCache, CompiledRule, Compiler};

    let mut group = c.benchmark_group("tiered");

    let expr_json = json!({
        "*": [
            {"+": [{"var": "a"}, {"var": "b"}]},
            {"-": [{"var": "c"}, {"var": "d"}]}
        ]
    });
    let data = json!({"a": 10, "b": 20, "c": 50, "d": 10});

    // Parse once
    let expr = parse(&expr_json).unwrap();

    // Benchmark Tier 0 (AST) evaluation
    let ast_rule = CompiledRule::new(expr.clone());
    group.bench_function("tier0_ast", |b| {
        b.iter(|| ast_rule.evaluate(black_box(&data)))
    });

    // Benchmark Tier 1 (Bytecode) evaluation
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&expr).unwrap();
    let bytecode_rule = CompiledRule::with_bytecode(expr.clone(), bytecode);
    group.bench_function("tier1_bytecode", |b| {
        b.iter(|| bytecode_rule.evaluate(black_box(&data)))
    });

    // Benchmark RuleCache with auto-promotion (pre-warmed)
    let cache = RuleCache::with_threshold(5);
    // Pre-warm to trigger promotion
    for _ in 0..10 {
        cache.evaluate("bench_rule", &expr, &data).unwrap();
    }
    group.bench_function("cache_promoted", |b| {
        b.iter(|| cache.evaluate("bench_rule", black_box(&expr), black_box(&data)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_conditionals,
    bench_arrays,
    bench_caching,
    bench_complexity,
    bench_trading_rules,
    bench_insurance_rules,
    bench_tiered_compilation,
);

criterion_main!(benches);
