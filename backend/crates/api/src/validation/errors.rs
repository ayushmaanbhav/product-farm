//! Validation error and result types
//!
//! Provides structured validation errors and warnings for rule validation.

/// Validation result for a set of rules
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether all validations passed
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Execution plan if valid
    pub execution_levels: Option<Vec<Vec<String>>>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            execution_levels: None,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// A validation error that prevents rule execution
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Rule ID that caused the error (if applicable)
    pub rule_id: Option<String>,
    /// Error code for categorization
    pub code: ValidationErrorCode,
    /// Human-readable error message
    pub message: String,
}

/// Validation error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    /// Invalid JSON Logic syntax
    InvalidSyntax,
    /// Circular dependency detected
    CyclicDependency,
    /// Missing required input
    MissingInput,
    /// Duplicate output attribute
    DuplicateOutput,
    /// Empty rule set
    EmptyRuleSet,
    /// Invalid rule configuration
    InvalidConfig,
}

/// A validation warning that doesn't prevent execution
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Rule ID that caused the warning (if applicable)
    pub rule_id: Option<String>,
    /// Warning code for categorization
    pub code: ValidationWarningCode,
    /// Human-readable warning message
    pub message: String,
}

/// Validation warning codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationWarningCode {
    /// Rule has no outputs defined
    NoOutputs,
    /// Rule has no inputs defined
    NoInputs,
    /// Unused rule output
    UnusedOutput,
    /// Disabled rule in chain
    DisabledRule,
}
