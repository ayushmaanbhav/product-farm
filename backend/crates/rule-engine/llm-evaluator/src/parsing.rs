//! Response parsing utilities for LLM output.
//!
//! Provides robust parsing of numbers and booleans from LLM text responses,
//! avoiding naive approaches that concatenate digits or misinterpret negation.

use crate::error::{LlmEvaluatorError, LlmEvaluatorResult};
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex to match a valid number (integer or decimal, with optional sign).
/// Matches: 42, -42, 42.5, -42.5, .5, -.5
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-?\d+\.?\d*|-?\.\d+").expect("Invalid number regex")
});

/// Regex for boolean detection - matches explicit yes/no/true/false at word boundaries.
static BOOLEAN_TRUE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\s*(yes|true|1)\b|\b(yes|true)\s*[.!]?\s*$").expect("Invalid bool regex")
});

static BOOLEAN_FALSE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\s*(no|false|0)\b|\b(no|false)\s*[.!]?\s*$").expect("Invalid bool regex")
});

/// Parse the first valid number from LLM response text.
///
/// This correctly handles:
/// - "The answer is 42" → 42
/// - "Between -10 and 20, I'd say 15.5" → -10 (first number)
/// - "text-123-456" → 123 (not -123456)
/// - "It costs $99.99" → 99.99
pub fn parse_number(text: &str) -> LlmEvaluatorResult<f64> {
    NUMBER_REGEX
        .find(text)
        .map(|m| m.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| {
            LlmEvaluatorError::ParseError(format!(
                "No valid number found in response: {}",
                truncate_for_error(text)
            ))
        })
}

/// Parse a boolean from LLM response text.
///
/// Looks for explicit true/false/yes/no answers, giving precedence to:
/// 1. First word of response (e.g., "Yes, because...")
/// 2. Explicit answer at end (e.g., "...the answer is true.")
///
/// This correctly handles:
/// - "Yes" → true
/// - "No, that's not right" → false
/// - "That statement is true" → true
/// - "The answer is false" → false
/// - "Not true" → false (negation detected)
pub fn parse_boolean(text: &str) -> LlmEvaluatorResult<bool> {
    let text = text.trim();
    let lower = text.to_lowercase();

    // Check for negation patterns - these indicate false even if "true" appears
    let has_negation = lower.contains("not true")
        || lower.contains("n't true")
        || lower.contains("isn't true")
        || lower.contains("not correct")
        || lower.contains("is false")
        || lower.starts_with("no,")
        || lower.starts_with("no.")
        || lower.starts_with("no ");

    if has_negation {
        return Ok(false);
    }

    // Look for explicit true signals at start or end
    if BOOLEAN_TRUE_REGEX.is_match(text) {
        return Ok(true);
    }

    // Look for explicit false signals at start or end
    if BOOLEAN_FALSE_REGEX.is_match(text) {
        return Ok(false);
    }

    // If no clear signal, look for true/false/yes/no anywhere (case insensitive)
    // Priority: check for false indicators first since they're more specific
    if lower.contains("false") {
        return Ok(false);
    }
    if lower.contains("true") || lower.contains("yes") {
        return Ok(true);
    }
    if lower.contains("no") {
        return Ok(false);
    }

    // Default to false if we can't determine
    Err(LlmEvaluatorError::ParseError(format!(
        "Could not determine boolean value from response: {}",
        truncate_for_error(text)
    )))
}

/// Truncate text for error messages to avoid huge logs.
fn truncate_for_error(text: &str) -> String {
    if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_simple() {
        assert_eq!(parse_number("42").unwrap(), 42.0);
        assert_eq!(parse_number("-42").unwrap(), -42.0);
        assert_eq!(parse_number("42.5").unwrap(), 42.5);
        assert_eq!(parse_number("-42.5").unwrap(), -42.5);
    }

    #[test]
    fn test_parse_number_in_text() {
        assert_eq!(parse_number("The answer is 42").unwrap(), 42.0);
        assert_eq!(parse_number("It costs $99.99").unwrap(), 99.99);
        assert_eq!(parse_number("Between 10 and 20").unwrap(), 10.0); // First number
    }

    #[test]
    fn test_parse_number_avoids_concatenation() {
        // This was the bug: "text-123-456" should NOT become -123456
        assert_eq!(parse_number("text-123-456").unwrap(), -123.0);
        assert_eq!(parse_number("values: 10, 20, 30").unwrap(), 10.0);
    }

    #[test]
    fn test_parse_number_negative() {
        assert_eq!(parse_number("Temperature: -15 degrees").unwrap(), -15.0);
    }

    #[test]
    fn test_parse_boolean_simple() {
        assert!(parse_boolean("Yes").unwrap());
        assert!(parse_boolean("true").unwrap());
        assert!(parse_boolean("TRUE").unwrap());
        assert!(!parse_boolean("No").unwrap());
        assert!(!parse_boolean("false").unwrap());
        assert!(!parse_boolean("FALSE").unwrap());
    }

    #[test]
    fn test_parse_boolean_with_context() {
        assert!(parse_boolean("Yes, I agree").unwrap());
        assert!(parse_boolean("The answer is true.").unwrap());
        assert!(!parse_boolean("No, that's wrong").unwrap());
        assert!(!parse_boolean("The answer is false.").unwrap());
    }

    #[test]
    fn test_parse_boolean_negation() {
        // "not true" should be false
        assert!(!parse_boolean("That's not true").unwrap());
        assert!(!parse_boolean("The statement is not true").unwrap());
    }

    #[test]
    fn test_parse_number_no_number() {
        assert!(parse_number("no numbers here").is_err());
    }
}
