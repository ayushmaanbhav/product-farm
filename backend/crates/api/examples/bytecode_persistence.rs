//! Bytecode Persistence Example
//!
//! Demonstrates how to persist compiled rules with bytecode for warm starts.
//! This enables faster rule loading by avoiding recompilation.
//!
//! Run with: cargo run -p product-farm-api --example bytecode_persistence

use product_farm_json_logic::{parse, CompiledRule};
use product_farm_persistence::{
    CompiledRuleRepository, FileStorageConfig, FileCompiledRuleRepository,
    memory::InMemoryCompiledRuleRepository,
};
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bytecode Persistence Demo ===\n");

    // Example 1: In-memory persistence
    println!("--- In-Memory Persistence ---");
    demo_in_memory().await?;

    // Example 2: File-based persistence
    println!("\n--- File-Based Persistence ---");
    demo_file_based().await?;

    // Example 3: Warm start performance comparison
    println!("\n--- Warm Start Performance ---");
    demo_warm_start_performance().await?;

    Ok(())
}

async fn demo_in_memory() -> Result<(), Box<dyn std::error::Error>> {
    let repo = InMemoryCompiledRuleRepository::new();

    // Create a complex rule
    let expr = parse(&json!({
        "if": [
            {"and": [
                {"<": [{"var": "rsi"}, 30]},
                {">": [{"var": "price"}, {"var": "sma_50"}]}
            ]}, "STRONG_BUY",
            {"<": [{"var": "rsi"}, 40]}, "BUY",
            {">": [{"var": "rsi"}, 70]}, "SELL",
            "HOLD"
        ]
    }))?;

    // Create and promote to bytecode
    let mut rule = CompiledRule::new(expr);
    println!("Initial tier: {:?}", rule.tier());

    rule.promote_to_bytecode()?;
    println!("After promotion: {:?}", rule.tier());

    // Persist the compiled rule
    let persisted = rule.to_persisted();
    repo.save("trading-signal", &persisted).await?;
    println!("Rule saved to in-memory repository");

    // List stored rules
    let ids = repo.list_ids().await?;
    println!("Stored rules: {:?}", ids);

    // Restore and use
    let loaded = repo.get("trading-signal").await?.unwrap();
    let restored = CompiledRule::from_persisted(loaded);
    println!("Restored tier: {:?}", restored.tier());

    // Evaluate
    let result = restored.evaluate(&json!({
        "rsi": 25,
        "price": 105.0,
        "sma_50": 100.0
    }))?;
    println!("Signal: {:?}", result);

    Ok(())
}

async fn demo_file_based() -> Result<(), Box<dyn std::error::Error>> {
    // Use temp directory
    let temp_dir = tempfile::TempDir::new()?;
    let config = FileStorageConfig::new(temp_dir.path());
    let repo = FileCompiledRuleRepository::new(config).await?;

    // Create an insurance premium calculation rule
    let expr = parse(&json!({
        "*": [
            {"var": "coverage_amount"},
            {"/": [{"var": "rate_per_1000"}, 1000]},
            {"if": [
                {">": [{"var": "age"}, 60]}, 1.5,
                {">": [{"var": "age"}, 50]}, 1.3,
                {">": [{"var": "age"}, 40]}, 1.1,
                1.0
            ]},
            {"+": [
                1,
                {"if": [{"var": "smoker"}, 0.5, 0]},
                {"if": [{">": [{"var": "bmi"}, 30]}, 0.3, 0]}
            ]}
        ]
    }))?;

    let mut rule = CompiledRule::new(expr);
    rule.promote_to_bytecode()?;

    // Save to file
    let persisted = rule.to_persisted();
    repo.save("insurance-premium", &persisted).await?;

    // Show file path
    let file_path = temp_dir.path().join("compiled").join("insurance-premium.json");
    println!("Saved to: {:?}", file_path);
    println!("File exists: {}", file_path.exists());

    // Read back
    let loaded = repo.get("insurance-premium").await?.unwrap();
    println!("Tier from file: {:?}", loaded.tier);
    println!("Has bytecode: {}", loaded.bytecode.is_some());

    // Calculate premium
    let restored = CompiledRule::from_persisted(loaded);
    let premium = restored.evaluate(&json!({
        "coverage_amount": 500000,
        "rate_per_1000": 5.50,
        "age": 45,
        "smoker": false,
        "bmi": 28
    }))?;
    println!("Annual premium: ${:.2}", premium.to_number());

    Ok(())
}

async fn demo_warm_start_performance() -> Result<(), Box<dyn std::error::Error>> {
    let repo = InMemoryCompiledRuleRepository::new();
    let iterations = 10000;

    // Complex nested rule
    let expr = parse(&json!({
        "if": [
            {">": [{"var": "score"}, 90]}, "A",
            {">": [{"var": "score"}, 80]}, "B",
            {">": [{"var": "score"}, 70]}, "C",
            {">": [{"var": "score"}, 60]}, "D",
            "F"
        ]
    }))?;

    // === Cold Start: Parse + Compile ===
    let cold_start = Instant::now();
    for _ in 0..iterations {
        let parsed = parse(&json!({
            "if": [
                {">": [{"var": "score"}, 90]}, "A",
                {">": [{"var": "score"}, 80]}, "B",
                {">": [{"var": "score"}, 70]}, "C",
                {">": [{"var": "score"}, 60]}, "D",
                "F"
            ]
        }))?;
        let mut rule = CompiledRule::new(parsed);
        rule.promote_to_bytecode()?;
        let _ = rule.evaluate(&json!({"score": 75}))?;
    }
    let cold_time = cold_start.elapsed();

    // === Warm Start: Load from cache ===
    // First, save a pre-compiled rule
    let mut pre_compiled = CompiledRule::new(expr);
    pre_compiled.promote_to_bytecode()?;
    let persisted = pre_compiled.to_persisted();
    repo.save("grading-rule", &persisted).await?;

    let warm_start = Instant::now();
    for _ in 0..iterations {
        let loaded = repo.get("grading-rule").await?.unwrap();
        let restored = CompiledRule::from_persisted(loaded);
        let _ = restored.evaluate(&json!({"score": 75}))?;
    }
    let warm_time = warm_start.elapsed();

    // === Already loaded: Just evaluate ===
    let loaded = repo.get("grading-rule").await?.unwrap();
    let restored = CompiledRule::from_persisted(loaded);

    let hot_start = Instant::now();
    for _ in 0..iterations {
        let _ = restored.evaluate(&json!({"score": 75}))?;
    }
    let hot_time = hot_start.elapsed();

    println!("Performance comparison ({} iterations):", iterations);
    println!("  Cold start (parse + compile + eval): {:?}", cold_time);
    println!("  Warm start (load + eval):            {:?}", warm_time);
    println!("  Hot path (eval only):                {:?}", hot_time);
    println!();
    println!("Speedup:");
    println!(
        "  Warm vs Cold: {:.1}x faster",
        cold_time.as_nanos() as f64 / warm_time.as_nanos() as f64
    );
    println!(
        "  Hot vs Cold:  {:.1}x faster",
        cold_time.as_nanos() as f64 / hot_time.as_nanos() as f64
    );

    Ok(())
}
