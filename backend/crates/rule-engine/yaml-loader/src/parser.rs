//! YAML parsing with tag detection.
//!
//! Parses YAML files into intermediate document structures,
//! detecting content types from tags and structure.

use crate::discovery::DiscoveredFiles;
use crate::error::{LoaderError, LoaderResult};
use crate::schema::YamlDocument;
use std::fs;
use std::path::PathBuf;

/// Parsed YAML document with source information.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// The parsed document content.
    pub document: YamlDocument,

    /// Source file path.
    pub source: PathBuf,

    /// Detected content sections.
    pub detected_sections: Vec<DetectedSection>,
}

/// A detected content section in the document.
#[derive(Debug, Clone)]
pub struct DetectedSection {
    /// Section type.
    pub section_type: SectionType,

    /// Key name in the YAML.
    pub key: String,

    /// Number of items in the section.
    pub item_count: usize,
}

/// Types of content sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Product,
    Types,
    Entities,
    Functions,
    Functionalities,
    Constraints,
    Layers,
    Unknown,
}

impl SectionType {
    /// Detect section type from key name.
    pub fn from_key(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "product" | "meta" | "metadata" => Self::Product,
            "types" | "datatypes" | "data-types" | "enums" => Self::Types,
            "entities" | "schema" | "models" => Self::Entities,
            "functions" | "rules" | "computations" | "logic" => Self::Functions,
            "functionalities" | "features" | "capabilities" => Self::Functionalities,
            "constraints" | "validations" => Self::Constraints,
            "layers" | "interfaces" | "views" => Self::Layers,
            _ => Self::Unknown,
        }
    }
}

/// Parse all discovered files into documents.
pub fn parse_all(files: &DiscoveredFiles) -> LoaderResult<Vec<ParsedDocument>> {
    let mut documents = Vec::new();

    for path in files.all_files() {
        let doc = parse_file(path)?;
        documents.push(doc);
    }

    Ok(documents)
}

/// Parse a single YAML file.
pub fn parse_file(path: &PathBuf) -> LoaderResult<ParsedDocument> {
    let content = fs::read_to_string(path).map_err(|e| LoaderError::file_read(path, e.to_string()))?;

    // First, parse as raw YAML to detect sections
    let raw: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| LoaderError::yaml_parse(path, e.to_string()))?;

    let detected_sections = detect_sections(&raw);

    // Then parse as structured document
    let document: YamlDocument = serde_yaml::from_str(&content)
        .map_err(|e| LoaderError::yaml_parse(path, e.to_string()))?;

    Ok(ParsedDocument {
        document,
        source: path.clone(),
        detected_sections,
    })
}

/// Detect content sections in raw YAML.
fn detect_sections(value: &serde_yaml::Value) -> Vec<DetectedSection> {
    let mut sections = Vec::new();

    if let serde_yaml::Value::Mapping(map) = value {
        for (key, val) in map {
            if let serde_yaml::Value::String(key_str) = key {
                let section_type = SectionType::from_key(key_str);
                let item_count = count_items(val);

                sections.push(DetectedSection {
                    section_type,
                    key: key_str.clone(),
                    item_count,
                });
            }
        }
    }

    sections
}

/// Count items in a YAML value.
fn count_items(value: &serde_yaml::Value) -> usize {
    match value {
        serde_yaml::Value::Mapping(map) => map.len(),
        serde_yaml::Value::Sequence(seq) => seq.len(),
        _ => 1,
    }
}

/// Parse a YAML string directly (for testing).
pub fn parse_yaml_string(content: &str) -> LoaderResult<YamlDocument> {
    serde_yaml::from_str(content)
        .map_err(|e| LoaderError::yaml_parse(PathBuf::from("<string>"), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_type_detection() {
        assert_eq!(SectionType::from_key("product"), SectionType::Product);
        assert_eq!(SectionType::from_key("types"), SectionType::Types);
        assert_eq!(SectionType::from_key("datatypes"), SectionType::Types);
        assert_eq!(SectionType::from_key("entities"), SectionType::Entities);
        assert_eq!(SectionType::from_key("schema"), SectionType::Entities);
        assert_eq!(SectionType::from_key("functions"), SectionType::Functions);
        assert_eq!(SectionType::from_key("rules"), SectionType::Functions);
        assert_eq!(SectionType::from_key("random"), SectionType::Unknown);
    }

    #[test]
    fn test_parse_yaml_string() {
        let yaml = r#"
product:
  id: test
  name: Test Product

entities:
  User:
    name: string
    age: int
"#;
        let doc = parse_yaml_string(yaml).unwrap();
        assert!(doc.product.is_some());
        assert!(doc.entities.is_some());
    }

    #[test]
    fn test_detect_sections() {
        let yaml = r#"
product:
  id: test
entities:
  User: {}
  Order: {}
functions:
  calc: {}
"#;
        let raw: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let sections = detect_sections(&raw);

        assert_eq!(sections.len(), 3);

        let product_section = sections.iter().find(|s| s.key == "product").unwrap();
        assert_eq!(product_section.section_type, SectionType::Product);

        let entities_section = sections.iter().find(|s| s.key == "entities").unwrap();
        assert_eq!(entities_section.section_type, SectionType::Entities);
        assert_eq!(entities_section.item_count, 2);

        let functions_section = sections.iter().find(|s| s.key == "functions").unwrap();
        assert_eq!(functions_section.section_type, SectionType::Functions);
    }
}
