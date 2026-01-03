//! Product-FARM YAML Loader
//!
//! A flexible YAML-based product definition loader that intelligently parses
//! user-defined schemas into Product-FARM core types.
//!
//! # Features
//!
//! - **Flexible Parsing**: Detect YAML structure from file names and content tags
//! - **Intelligent Interpretation**: Infer attribute types from naming conventions
//! - **Master Schema Derivation**: Extract unified schema from multi-layer definitions
//! - **Inference Reports**: Generate confidence-based reports of parsing decisions
//!
//! # Quick Start
//!
//! ```ignore
//! use product_farm_yaml_loader::{init, State};
//!
//! // Initialize from a folder containing YAML files
//! let mut registry = init("./products/my-product")?;
//!
//! // Create state with input values
//! let mut state = State::new();
//! state.set("scenario.difficulty", "expert");
//! state.set("scenario.max_score", 100.0);
//!
//! // Evaluate a function
//! let result = registry.evaluate("my-product-v1", state, "calculate-score")?;
//!
//! println!("Score: {:?}", result.outputs.get("scenario.score"));
//! ```

pub mod error;
pub mod schema;
pub mod discovery;
pub mod parser;
pub mod interpreter;
pub mod transformer;
pub mod registry;
pub mod evaluator;
pub mod report;
pub mod output;
pub mod llm_prompt;

// Re-export farmscript from the separate crate
pub use product_farm_farmscript as farmscript;

pub use error::*;
pub use schema::*;
pub use registry::*;
pub use evaluator::{State, EvalResult};
pub use report::InferenceReport;

use std::path::Path;

/// Initialize a ProductRegistry from a YAML folder.
///
/// This function:
/// 1. Discovers all YAML files in the folder (recursive)
/// 2. Parses files based on names and content tags
/// 3. Intelligently interprets field definitions
/// 4. Transforms into core Product-FARM types
/// 5. Validates critical requirements only
/// 6. Pre-compiles rules for efficient evaluation
///
/// # Arguments
///
/// * `folder_path` - Path to folder containing YAML product definitions
///
/// # Returns
///
/// A `ProductRegistry` ready for evaluation, or an error if critical
/// information is missing.
///
/// # Example
///
/// ```ignore
/// let registry = init("./products/assessment")?;
/// ```
pub fn init<P: AsRef<Path>>(folder_path: P) -> LoaderResult<ProductRegistry> {
    let path = folder_path.as_ref();

    // 1. Discover YAML files
    let files = discovery::discover_yaml_files(path)?;

    // 2. Parse all files
    let documents = parser::parse_all(&files)?;

    // 3. Transform to master schema
    let transformer = transformer::SchemaTransformer::new();
    let schema = transformer.transform(documents)?;

    // 4. Validate critical requirements only
    transformer.validate_critical(&schema)?;

    // 5. Build registry with pre-compiled rules
    let mut registry = ProductRegistry::new();
    registry.register(schema)?;
    registry.compile_rules()?;

    Ok(registry)
}

/// Initialize and generate an inference report.
///
/// Same as `init()` but also returns a detailed inference report
/// showing what was detected and confidence levels.
///
/// # Returns
///
/// A tuple of (ProductRegistry, InferenceReport)
pub fn init_with_report<P: AsRef<Path>>(
    folder_path: P,
) -> LoaderResult<(ProductRegistry, InferenceReport)> {
    let path = folder_path.as_ref();

    // 1. Discover YAML files
    let files = discovery::discover_yaml_files(path)?;

    // 2. Parse all files
    let documents = parser::parse_all(&files)?;

    // 3. Transform to master schema (with report generation)
    let transformer = transformer::SchemaTransformer::new();
    let (schema, report) = transformer.transform_with_report(documents)?;

    // 4. Validate critical requirements only
    transformer.validate_critical(&schema)?;

    // 5. Build registry
    let mut registry = ProductRegistry::new();
    registry.register(schema)?;
    registry.compile_rules()?;

    Ok((registry, report))
}

/// Load a single product definition without building a registry.
///
/// Useful for validation, inspection, or custom processing.
pub fn load<P: AsRef<Path>>(path: P) -> LoaderResult<MasterSchema> {
    let path = path.as_ref();

    let files = discovery::discover_yaml_files(path)?;
    let documents = parser::parse_all(&files)?;

    let transformer = transformer::SchemaTransformer::new();
    transformer.transform(documents)
}
