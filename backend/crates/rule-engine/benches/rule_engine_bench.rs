//! Benchmarks for DAG-based rule execution
//!
//! Run with: cargo bench -p product-farm-rule-engine

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use product_farm_core::Rule;
use product_farm_rule_engine::{ExecutionContext, RuleExecutor, RuleDag};
use serde_json::json;

/// Create a simple rule
fn make_rule(product: &str, inputs: &[&str], outputs: &[&str], expr: serde_json::Value, order: i32) -> Rule {
    Rule::new(product, "BENCH", expr.to_string())
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
        .with_order(order)
}

/// Benchmark DAG construction
fn bench_dag_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_construction");

    for size in [5, 10, 25, 50, 100] {
        // Create a linear chain of rules
        let rules: Vec<Rule> = (0..size)
            .map(|i| {
                let input = if i == 0 { "input" } else { &format!("v{}", i - 1) };
                let output = format!("v{}", i);
                make_rule(
                    "bench",
                    &[input],
                    &[&output],
                    json!({"+": [{"var": input}, 1]}),
                    i,
                )
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("linear_chain", size),
            &rules,
            |b, rules| {
                b.iter(|| RuleDag::from_rules(black_box(rules)))
            },
        );
    }

    // Diamond pattern: input -> a,b -> output
    for width in [2, 5, 10, 20] {
        let mut rules = Vec::new();

        // First rule creates base
        rules.push(make_rule(
            "bench",
            &["input"],
            &["base"],
            json!({"+": [{"var": "input"}, 1]}),
            0,
        ));

        // Middle rules in parallel
        for i in 0..width {
            let output = format!("mid{}", i);
            rules.push(make_rule(
                "bench",
                &["base"],
                &[&output],
                json!({"*": [{"var": "base"}, i + 1]}),
                1,
            ));
        }

        // Final rule combines all
        let mid_inputs: Vec<String> = (0..width).map(|i| format!("mid{}", i)).collect();
        let inputs: Vec<&str> = mid_inputs.iter().map(|s| s.as_str()).collect();
        rules.push(make_rule(
            "bench",
            &inputs,
            &["final"],
            json!({"+": mid_inputs.iter().map(|s| json!({"var": s})).collect::<Vec<_>>()}),
            2,
        ));

        group.bench_with_input(
            BenchmarkId::new("diamond_pattern", width),
            &rules,
            |b, rules| {
                b.iter(|| RuleDag::from_rules(black_box(rules)))
            },
        );
    }

    group.finish();
}

/// Benchmark rule execution
fn bench_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution");

    // Single rule
    let rules = vec![
        make_rule("bench", &["x"], &["y"], json!({"*": [{"var": "x"}, 2]}), 0),
    ];
    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    group.bench_function("single_rule", |b| {
        b.iter(|| {
            let mut ctx = ExecutionContext::from_json(&json!({"x": 21}));
            executor.execute(black_box(&rules), black_box(&mut ctx))
        })
    });

    // Linear chain of 10 rules
    let chain_rules: Vec<Rule> = (0..10)
        .map(|i| {
            let input = if i == 0 { "input".to_string() } else { format!("v{}", i - 1) };
            let output = format!("v{}", i);
            make_rule(
                "bench",
                &[&input],
                &[&output],
                json!({"+": [{"var": &input}, 1]}),
                i,
            )
        })
        .collect();

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&chain_rules).unwrap();

    group.bench_function("chain_10_rules", |b| {
        b.iter(|| {
            let mut ctx = ExecutionContext::from_json(&json!({"input": 0}));
            executor.execute(black_box(&chain_rules), black_box(&mut ctx))
        })
    });

    // Parallel rules (diamond pattern)
    let parallel_rules = vec![
        make_rule("bench", &["input"], &["a"], json!({"+": [{"var": "input"}, 1]}), 0),
        make_rule("bench", &["input"], &["b"], json!({"*": [{"var": "input"}, 2]}), 0),
        make_rule("bench", &["input"], &["c"], json!({"-": [{"var": "input"}, 1]}), 0),
        make_rule("bench", &["a", "b", "c"], &["result"],
            json!({"+": [{"var": "a"}, {"var": "b"}, {"var": "c"}]}), 1),
    ];

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&parallel_rules).unwrap();

    group.bench_function("parallel_4_rules", |b| {
        b.iter(|| {
            let mut ctx = ExecutionContext::from_json(&json!({"input": 10}));
            executor.execute(black_box(&parallel_rules), black_box(&mut ctx))
        })
    });

    group.finish();
}

/// Benchmark with varying rule counts
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    group.sample_size(50);

    for count in [5, 10, 25, 50] {
        // Create chain of rules
        let rules: Vec<Rule> = (0..count)
            .map(|i| {
                let input = if i == 0 { "input".to_string() } else { format!("v{}", i - 1) };
                let output = format!("v{}", i);
                make_rule(
                    "bench",
                    &[&input],
                    &[&output],
                    json!({"+": [{"var": &input}, 1]}),
                    i,
                )
            })
            .collect();

        let mut executor = RuleExecutor::new();
        executor.compile_rules(&rules).unwrap();

        group.bench_with_input(
            BenchmarkId::new("chain_rules", count),
            &rules,
            |b, rules| {
                b.iter(|| {
                    let mut ctx = ExecutionContext::from_json(&json!({"input": 0}));
                    executor.execute(black_box(rules), black_box(&mut ctx))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark realistic trading strategy
fn bench_trading_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("trading_strategy");

    let rules = vec![
        // RSI Signal
        make_rule(
            "strategy",
            &["rsi"],
            &["rsi_signal"],
            json!({
                "if": [
                    {"<": [{"var": "rsi"}, 30]}, "OVERSOLD",
                    {">": [{"var": "rsi"}, 70]}, "OVERBOUGHT",
                    "NEUTRAL"
                ]
            }),
            0,
        ),
        // MACD Signal
        make_rule(
            "strategy",
            &["macd", "macd_signal"],
            &["macd_trend"],
            json!({
                "if": [
                    {">": [{"var": "macd"}, {"var": "macd_signal"}]}, "BULLISH",
                    "BEARISH"
                ]
            }),
            0,
        ),
        // Price vs SMA
        make_rule(
            "strategy",
            &["price", "sma_50"],
            &["price_trend"],
            json!({
                "if": [
                    {">": [{"var": "price"}, {"var": "sma_50"}]}, "ABOVE",
                    "BELOW"
                ]
            }),
            0,
        ),
        // Combined Entry Signal
        make_rule(
            "strategy",
            &["rsi_signal", "macd_trend", "price_trend"],
            &["entry_signal"],
            json!({
                "if": [
                    {"and": [
                        {"==": [{"var": "rsi_signal"}, "OVERSOLD"]},
                        {"==": [{"var": "macd_trend"}, "BULLISH"]},
                        {"==": [{"var": "price_trend"}, "ABOVE"]}
                    ]},
                    "STRONG_BUY",
                    {"and": [
                        {"==": [{"var": "rsi_signal"}, "OVERSOLD"]},
                        {"==": [{"var": "price_trend"}, "ABOVE"]}
                    ]},
                    "BUY",
                    {"and": [
                        {"==": [{"var": "rsi_signal"}, "OVERBOUGHT"]},
                        {"==": [{"var": "macd_trend"}, "BEARISH"]},
                        {"==": [{"var": "price_trend"}, "BELOW"]}
                    ]},
                    "STRONG_SELL",
                    "HOLD"
                ]
            }),
            1,
        ),
        // Position Size
        make_rule(
            "strategy",
            &["entry_signal", "account_balance", "risk_percent"],
            &["position_size"],
            json!({
                "if": [
                    {"==": [{"var": "entry_signal"}, "HOLD"]},
                    0,
                    {"*": [
                        {"var": "account_balance"},
                        {"/": [{"var": "risk_percent"}, 100]},
                        {"if": [
                            {"in": [{"var": "entry_signal"}, ["STRONG_BUY", "STRONG_SELL"]]},
                            1.5,
                            1.0
                        ]}
                    ]}
                ]
            }),
            2,
        ),
    ];

    let market_data = json!({
        "rsi": 28.5,
        "macd": 0.5,
        "macd_signal": 0.3,
        "price": 105.0,
        "sma_50": 100.0,
        "account_balance": 100000,
        "risk_percent": 2.0
    });

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    group.bench_function("full_strategy", |b| {
        b.iter(|| {
            let mut ctx = ExecutionContext::from_json(&market_data);
            executor.execute(black_box(&rules), black_box(&mut ctx))
        })
    });

    group.finish();
}

/// Benchmark insurance premium calculation
fn bench_insurance_premium(c: &mut Criterion) {
    let mut group = c.benchmark_group("insurance_premium");

    let rules = vec![
        // Age factor
        make_rule(
            "insurance",
            &["age"],
            &["age_factor"],
            json!({
                "if": [
                    {">": [{"var": "age"}, 60]}, 1.5,
                    {">": [{"var": "age"}, 50]}, 1.3,
                    {">": [{"var": "age"}, 40]}, 1.1,
                    {"<": [{"var": "age"}, 25]}, 1.2,
                    1.0
                ]
            }),
            0,
        ),
        // Health factor
        make_rule(
            "insurance",
            &["smoker", "bmi"],
            &["health_factor"],
            json!({
                "+": [
                    1.0,
                    {"if": [{"var": "smoker"}, 0.5, 0]},
                    {"if": [{">": [{"var": "bmi"}, 30]}, 0.3, 0]},
                    {"if": [{">": [{"var": "bmi"}, 35]}, 0.2, 0]}
                ]
            }),
            0,
        ),
        // Occupation factor
        make_rule(
            "insurance",
            &["occupation_risk"],
            &["occupation_factor"],
            json!({
                "if": [
                    {"==": [{"var": "occupation_risk"}, "high"]}, 1.4,
                    {"==": [{"var": "occupation_risk"}, "medium"]}, 1.2,
                    1.0
                ]
            }),
            0,
        ),
        // Combined factor
        make_rule(
            "insurance",
            &["age_factor", "health_factor", "occupation_factor"],
            &["combined_factor"],
            json!({
                "*": [
                    {"var": "age_factor"},
                    {"var": "health_factor"},
                    {"var": "occupation_factor"}
                ]
            }),
            1,
        ),
        // Final premium
        make_rule(
            "insurance",
            &["base_premium", "coverage_amount", "combined_factor"],
            &["final_premium"],
            json!({
                "*": [
                    {"var": "base_premium"},
                    {"/": [{"var": "coverage_amount"}, 100000]},
                    {"var": "combined_factor"}
                ]
            }),
            2,
        ),
    ];

    let customer_data = json!({
        "age": 45,
        "smoker": false,
        "bmi": 27.5,
        "occupation_risk": "low",
        "base_premium": 500,
        "coverage_amount": 250000
    });

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    group.bench_function("full_premium_calc", |b| {
        b.iter(|| {
            let mut ctx = ExecutionContext::from_json(&customer_data);
            executor.execute(black_box(&rules), black_box(&mut ctx))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dag_construction,
    bench_execution,
    bench_scaling,
    bench_trading_strategy,
    bench_insurance_premium,
);

criterion_main!(benches);
