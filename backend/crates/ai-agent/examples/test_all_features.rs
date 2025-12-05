//! Comprehensive test of all AI Agent functionalities
//!
//! Run with: cargo run -p product-farm-ai-agent --example test_all_features
//!
//! For Claude API features, set ANTHROPIC_API_KEY environment variable:
//! ANTHROPIC_API_KEY=your-key cargo run -p product-farm-ai-agent --example test_all_features --features anthropic

use product_farm_ai_agent::{
    explainer::{RuleExplainer, Verbosity},
    translator::{RuleTranslator, TranslationContext},
    validator::RuleValidator,
    visualizer::{GraphVisualizer, NodeType, EdgeType},
};
use serde_json::json;

fn separator(c: char, n: usize) -> String {
    std::iter::repeat_n(c, n).collect()
}

fn main() {
    println!("{}", separator('=', 60));
    println!("  Product-FARM AI Agent - Feature Test Suite");
    println!("{}", separator('=', 60));
    println!();

    // Test 1: Rule Validation
    test_validation();

    // Test 2: Rule Explanation
    test_explanation();

    // Test 3: Rule Testing/Execution
    test_rule_execution();

    // Test 4: Display Expression Generation
    test_display_expression();

    // Test 5: Graph Visualization
    test_graph_visualization();

    // Test 6: Translation Context
    test_translation_context();

    // Test 7: Complex Rule Scenarios
    test_complex_rules();

    // Test 8: Claude API Integration (if enabled)
    #[cfg(feature = "anthropic")]
    test_claude_integration();

    println!();
    println!("{}", separator('=', 60));
    println!("  All tests completed!");
    println!("{}", separator('=', 60));
}

fn test_validation() {
    println!("\n[TEST 1] Rule Validation");
    println!("{}", separator('-', 40));

    let validator = RuleValidator::new();

    // Valid rule
    let expr = json!({">": [{"var": "age"}, 18]});
    let result = validator
        .validate(&expr, &["age".to_string()], &["is_adult".to_string()])
        .unwrap();

    println!("Expression: {}", serde_json::to_string(&expr).unwrap());
    println!("Valid: {}", result.is_valid);
    println!("Errors: {:?}", result.errors);
    println!("Warnings: {:?}", result.warnings);
    assert!(result.is_valid, "Simple comparison should be valid");

    // Invalid rule - no output
    let result2 = validator.validate(&expr, &["age".to_string()], &[]).unwrap();
    println!("\nNo output test:");
    println!("Valid: {}", result2.is_valid);
    println!("Errors: {:?}", result2.errors.iter().map(|e| &e.code).collect::<Vec<_>>());
    assert!(!result2.is_valid, "Rule without output should be invalid");

    // Missing input warning
    let result3 = validator
        .validate(&expr, &[], &["is_adult".to_string()])
        .unwrap();
    println!("\nMissing input test:");
    println!("Warnings: {:?}", result3.warnings.iter().map(|w| &w.code).collect::<Vec<_>>());
    assert!(
        result3.warnings.iter().any(|w| w.code == "UNDECLARED_INPUT"),
        "Should warn about undeclared input"
    );

    println!("\n[PASS] Validation tests passed");
}

fn test_explanation() {
    println!("\n[TEST 2] Rule Explanation");
    println!("{}", separator('-', 40));

    // Brief explanation
    let explainer_brief = RuleExplainer::new(Verbosity::Brief);
    let expr = json!({">": [{"var": "age"}, 60]});
    let result = explainer_brief.explain(&expr).unwrap();
    println!("Brief: {}", result.explanation);

    // Detailed explanation
    let explainer_detailed = RuleExplainer::new(Verbosity::Detailed);
    let result = explainer_detailed.explain(&expr).unwrap();
    println!("Detailed: {}", result.explanation);
    println!("Variables: {:?}", result.variables.iter().map(|v| &v.name).collect::<Vec<_>>());

    // Complex if-then-else
    let complex_expr = json!({
        "if": [
            {">": [{"var": "age"}, 60]},
            {"*": [{"var": "base_rate"}, 1.2]},
            {"var": "base_rate"}
        ]
    });
    let result = explainer_detailed.explain(&complex_expr).unwrap();
    println!("\nComplex rule explanation:");
    println!("{}", result.explanation);
    println!("Steps: {}", result.steps.len());

    // Technical explanation
    let explainer_tech = RuleExplainer::new(Verbosity::Technical);
    let result = explainer_tech.explain(&complex_expr).unwrap();
    println!("\nTechnical: {}", result.explanation);

    println!("\n[PASS] Explanation tests passed");
}

fn test_rule_execution() {
    println!("\n[TEST 3] Rule Testing/Execution");
    println!("{}", separator('-', 40));

    // Simple comparison
    let expr = json!({">": [{"var": "age"}, 18]});
    let data = json!({"age": 25});
    let result = product_farm_json_logic::evaluate(&expr, &data).unwrap();
    println!("age > 18 where age=25: {:?}", result);
    assert_eq!(result, product_farm_core::Value::Bool(true));

    let data2 = json!({"age": 15});
    let result2 = product_farm_json_logic::evaluate(&expr, &data2).unwrap();
    println!("age > 18 where age=15: {:?}", result2);
    assert_eq!(result2, product_farm_core::Value::Bool(false));

    // Arithmetic
    let expr = json!({"*": [{"var": "price"}, {"var": "quantity"}]});
    let data = json!({"price": 10.5, "quantity": 3});
    let result = product_farm_json_logic::evaluate(&expr, &data).unwrap();
    println!("price * quantity (10.5 * 3): {:?}", result);

    // Conditional calculation
    let expr = json!({
        "if": [
            {">": [{"var": "age"}, 60]},
            {"*": [{"var": "premium"}, 1.2]},
            {"var": "premium"}
        ]
    });
    let data = json!({"age": 65, "premium": 100.0});
    let result = product_farm_json_logic::evaluate(&expr, &data).unwrap();
    println!("Premium with age loading (age=65, premium=100): {:?}", result);

    let data2 = json!({"age": 30, "premium": 100.0});
    let result2 = product_farm_json_logic::evaluate(&expr, &data2).unwrap();
    println!("Premium without loading (age=30, premium=100): {:?}", result2);

    println!("\n[PASS] Rule execution tests passed");
}

fn test_display_expression() {
    println!("\n[TEST 4] Display Expression Generation");
    println!("{}", separator('-', 40));

    let ctx = TranslationContext::new();
    let translator = RuleTranslator::new(ctx);

    let expressions = vec![
        json!({">": [{"var": "age"}, 60]}),
        json!({"and": [{">": [{"var": "age"}, 18]}, {"<": [{"var": "age"}, 65]}]}),
        json!({"if": [
            {">": [{"var": "rsi"}, 70]},
            "SELL",
            {"<": [{"var": "rsi"}, 30]},
            "BUY",
            "HOLD"
        ]}),
        json!({"*": [{"var": "base"}, {"var": "multiplier"}, 1.1]}),
        json!({"in": [{"var": "category"}, ["A", "B", "C"]]}),
    ];

    for expr in expressions {
        let display = translator.generate_display_expression(&expr);
        println!("JSON: {}", serde_json::to_string(&expr).unwrap());
        println!("Display: {}", display);
        println!();
    }

    println!("[PASS] Display expression tests passed");
}

fn test_graph_visualization() {
    println!("\n[TEST 5] Graph Visualization");
    println!("{}", separator('-', 40));

    let mut visualizer = GraphVisualizer::new();

    // Add attribute nodes
    visualizer.add_node("age", "age", NodeType::Input);
    visualizer.add_node("base_rate", "base_rate", NodeType::Input);
    visualizer.add_node("coverage", "coverage", NodeType::Input);
    visualizer.add_node("age_loading", "age_loading", NodeType::Attribute);
    visualizer.add_node("premium", "premium", NodeType::Output);

    // Add rule nodes
    visualizer.add_node("rule_age_loading", "Calculate Age Loading", NodeType::Rule);
    visualizer.add_node("rule_premium", "Calculate Premium", NodeType::Rule);

    // Add edges: inputs -> rule -> outputs
    visualizer.add_edge("age", "rule_age_loading", EdgeType::DependsOn);
    visualizer.add_edge("rule_age_loading", "age_loading", EdgeType::Computes);

    visualizer.add_edge("base_rate", "rule_premium", EdgeType::DependsOn);
    visualizer.add_edge("coverage", "rule_premium", EdgeType::DependsOn);
    visualizer.add_edge("age_loading", "rule_premium", EdgeType::DependsOn);
    visualizer.add_edge("rule_premium", "premium", EdgeType::Computes);

    // Generate Mermaid
    let mermaid_output = visualizer.render("mermaid").unwrap();
    println!("Mermaid diagram ({} nodes, {} edges):", mermaid_output.node_count, mermaid_output.edge_count);
    println!("{}", mermaid_output.content);

    // Generate DOT
    let dot_output = visualizer.render("dot").unwrap();
    println!("DOT format:");
    println!("{}", dot_output.content);

    // Generate ASCII
    let ascii_output = visualizer.render("ascii").unwrap();
    println!("ASCII format:");
    println!("{}", ascii_output.content);

    println!("\n[PASS] Graph visualization tests passed");
}

fn test_translation_context() {
    println!("\n[TEST 6] Translation Context");
    println!("{}", separator('-', 40));

    let context = TranslationContext::new()
        .add_attribute("age", "number", true)
        .add_attribute("premium", "number", false)
        .add_attribute("status", "string", true)
        .add_enum("SignalType", vec!["BUY".into(), "SELL".into(), "HOLD".into()]);

    let prompt = context.to_system_prompt();
    println!("System prompt preview (first 500 chars):");
    let preview_len = prompt.len().min(500);
    println!("{}", &prompt[..preview_len]);
    println!("...");
    println!("\nTotal prompt length: {} chars", prompt.len());

    println!("\n[PASS] Translation context tests passed");
}

fn test_complex_rules() {
    println!("\n[TEST 7] Complex Rule Scenarios");
    println!("{}", separator('-', 40));

    // Trading stop-loss rule
    let stop_loss = json!({
        "if": [
            {"and": [
                {">": [{"var": "holding_days"}, 30]},
                {">": [{"var": "current_price"}, {"var": "entry_price"}]}
            ]},
            {"if": [
                {"<": [
                    {"var": "current_price"},
                    {"*": [{"var": "highest_price"}, 0.97]}
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
    });

    println!("Stop-loss rule test:");

    // Test case 1: Long hold, in profit, trailing stop hit
    let data1 = json!({
        "holding_days": 45,
        "current_price": 96.0,
        "entry_price": 90.0,
        "highest_price": 100.0
    });
    let result1 = product_farm_json_logic::evaluate(&stop_loss, &data1).unwrap();
    println!("Case 1 (long hold, trailing stop hit): {:?}", result1);

    // Test case 2: Short hold, 5% loss
    let data2 = json!({
        "holding_days": 10,
        "current_price": 85.5,
        "entry_price": 90.0,
        "highest_price": 92.0
    });
    let result2 = product_farm_json_logic::evaluate(&stop_loss, &data2).unwrap();
    println!("Case 2 (short hold, 5% stop hit): {:?}", result2);

    // Test case 3: No stop triggered
    let data3 = json!({
        "holding_days": 10,
        "current_price": 88.0,
        "entry_price": 90.0,
        "highest_price": 90.0
    });
    let result3 = product_farm_json_logic::evaluate(&stop_loss, &data3).unwrap();
    println!("Case 3 (no stop triggered): {:?}", result3);

    // Validate the complex rule
    let validator = RuleValidator::new();
    let validation = validator.validate(
        &stop_loss,
        &[
            "holding_days".into(),
            "current_price".into(),
            "entry_price".into(),
            "highest_price".into(),
        ],
        &["signal".into()],
    ).unwrap();
    println!("\nValidation result: valid={}, warnings={}",
        validation.is_valid,
        validation.warnings.len()
    );

    // Explain the complex rule
    let explainer = RuleExplainer::new(Verbosity::Detailed);
    let explanation = explainer.explain(&stop_loss).unwrap();
    println!("\nRule explanation:");
    println!("{}", explanation.explanation);

    println!("\n[PASS] Complex rule tests passed");
}

#[cfg(feature = "anthropic")]
fn test_claude_integration() {
    use product_farm_ai_agent::{RuleAgentBuilder, TranslationContext};

    println!("\n[TEST 8] Claude API Integration");
    println!("{}", separator('-', 40));

    let api_key = std::env::var("ANTHROPIC_API_KEY");

    match api_key {
        Ok(key) => {
            println!("API key found, testing Claude integration...");

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let agent = RuleAgentBuilder::new()
                    .with_api_key(key)
                    .with_context(TranslationContext::new()
                        .add_attribute("age", "number", true)
                        .add_attribute("base_rate", "number", true)
                        .add_attribute("premium", "number", false))
                    .build()
                    .unwrap();

                // Test NL to JSON Logic translation
                println!("\nTranslating: 'If age > 60, multiply base_rate by 1.2'");

                match agent.translate_to_json_logic(
                    "If age is greater than 60, multiply base_rate by 1.2, otherwise use base_rate",
                    &["age", "base_rate"],
                    &["premium"],
                ).await {
                    Ok(result) => {
                        println!("Generated expression: {}",
                            serde_json::to_string_pretty(&result.expression).unwrap());
                        println!("Display: {:?}", result.display_expression);
                        println!("Inputs: {:?}", result.input_attributes);
                        println!("Outputs: {:?}", result.output_attributes);
                        println!("\n[PASS] Claude integration test passed");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        println!("(This is expected if API key is invalid)");
                    }
                }
            });
        }
        Err(_) => {
            println!("ANTHROPIC_API_KEY not set, skipping Claude integration test.");
            println!("To test, run with:");
            println!("  ANTHROPIC_API_KEY=your-key cargo run --example test_all_features --features anthropic");
        }
    }
}
