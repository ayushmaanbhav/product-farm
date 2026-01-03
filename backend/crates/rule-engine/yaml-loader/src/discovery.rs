//! File discovery for YAML product definitions.
//!
//! Scans folders to find and classify YAML files based on naming
//! conventions and content.

use crate::error::{LoaderError, LoaderResult};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Discovered YAML files categorized by type.
#[derive(Debug, Clone, Default)]
pub struct DiscoveredFiles {
    /// Product metadata files: product.yaml, *.product.yaml
    pub product_files: Vec<PathBuf>,

    /// Type/enum definition files: types.yaml, datatypes.yaml, enums.yaml
    pub type_files: Vec<PathBuf>,

    /// Entity/schema files: entities.yaml, schema.yaml, models.yaml
    pub entity_files: Vec<PathBuf>,

    /// Function/rule files: functions.yaml, rules.yaml, computations.yaml
    pub function_files: Vec<PathBuf>,

    /// Functionality files: functionalities.yaml, features.yaml
    pub functionality_files: Vec<PathBuf>,

    /// Combined files (contain multiple sections): combined.yaml, definition.yaml
    pub combined_files: Vec<PathBuf>,

    /// Unclassified YAML files (will be parsed as combined).
    pub other_files: Vec<PathBuf>,

    /// Source folder path.
    pub source_folder: PathBuf,
}

impl DiscoveredFiles {
    /// Check if any files were discovered.
    pub fn is_empty(&self) -> bool {
        self.product_files.is_empty()
            && self.type_files.is_empty()
            && self.entity_files.is_empty()
            && self.function_files.is_empty()
            && self.functionality_files.is_empty()
            && self.combined_files.is_empty()
            && self.other_files.is_empty()
    }

    /// Get all discovered files in processing order.
    pub fn all_files(&self) -> Vec<&PathBuf> {
        let mut files = Vec::new();
        // Product files first for metadata
        files.extend(self.product_files.iter());
        // Types before entities (for type references)
        files.extend(self.type_files.iter());
        // Entities before functions (for attribute references)
        files.extend(self.entity_files.iter());
        // Functions before functionalities
        files.extend(self.function_files.iter());
        files.extend(self.functionality_files.iter());
        // Combined files last
        files.extend(self.combined_files.iter());
        files.extend(self.other_files.iter());
        files
    }

    /// Get total file count.
    pub fn file_count(&self) -> usize {
        self.product_files.len()
            + self.type_files.len()
            + self.entity_files.len()
            + self.function_files.len()
            + self.functionality_files.len()
            + self.combined_files.len()
            + self.other_files.len()
    }
}

/// Discover YAML files in a folder.
///
/// Recursively scans the folder for .yaml and .yml files,
/// categorizing them based on filename patterns.
pub fn discover_yaml_files(folder: &Path) -> LoaderResult<DiscoveredFiles> {
    if !folder.exists() {
        return Err(LoaderError::PathNotFound(folder.to_path_buf()));
    }

    let mut discovered = DiscoveredFiles {
        source_folder: folder.to_path_buf(),
        ..Default::default()
    };

    // Walk the directory tree
    for entry in WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check for YAML extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != "yaml" && extension != "yml" {
            continue;
        }

        // Classify based on filename
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        classify_file(path, &file_name, &mut discovered);
    }

    if discovered.is_empty() {
        return Err(LoaderError::NoYamlFiles(folder.to_path_buf()));
    }

    Ok(discovered)
}

/// Classify a file based on its name.
fn classify_file(path: &Path, file_name: &str, discovered: &mut DiscoveredFiles) {
    let path_buf = path.to_path_buf();

    // Product files
    if file_name == "product"
        || file_name.ends_with(".product")
        || file_name == "meta"
        || file_name == "metadata"
    {
        discovered.product_files.push(path_buf);
        return;
    }

    // Type/enum files
    if file_name == "types"
        || file_name == "datatypes"
        || file_name == "data-types"
        || file_name == "enums"
        || file_name.ends_with(".types")
        || file_name.ends_with(".enums")
    {
        discovered.type_files.push(path_buf);
        return;
    }

    // Entity/schema files
    if file_name == "entities"
        || file_name == "schema"
        || file_name == "models"
        || file_name.ends_with(".entity")
        || file_name.ends_with(".schema")
        || file_name.ends_with(".model")
    {
        discovered.entity_files.push(path_buf);
        return;
    }

    // Function/rule files
    if file_name == "functions"
        || file_name == "rules"
        || file_name == "computations"
        || file_name == "logic"
        || file_name.ends_with(".function")
        || file_name.ends_with(".rule")
    {
        discovered.function_files.push(path_buf);
        return;
    }

    // Functionality files
    if file_name == "functionalities"
        || file_name == "features"
        || file_name == "capabilities"
        || file_name.ends_with(".functionality")
        || file_name.ends_with(".feature")
    {
        discovered.functionality_files.push(path_buf);
        return;
    }

    // Combined definition files
    if file_name == "definition"
        || file_name == "combined"
        || file_name == "config"
        || file_name == "index"
        || file_name == "main"
    {
        discovered.combined_files.push(path_buf);
        return;
    }

    // Unclassified - treat as combined/other
    discovered.other_files.push(path_buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_empty_folder() {
        let temp = TempDir::new().unwrap();
        let result = discover_yaml_files(temp.path());
        assert!(matches!(result, Err(LoaderError::NoYamlFiles(_))));
    }

    #[test]
    fn test_discover_nonexistent_folder() {
        let result = discover_yaml_files(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(LoaderError::PathNotFound(_))));
    }

    #[test]
    fn test_discover_and_classify() {
        let temp = TempDir::new().unwrap();

        // Create test files
        fs::write(temp.path().join("product.yaml"), "id: test").unwrap();
        fs::write(temp.path().join("types.yaml"), "types: {}").unwrap();
        fs::write(temp.path().join("entities.yaml"), "entities: {}").unwrap();
        fs::write(temp.path().join("functions.yaml"), "functions: {}").unwrap();
        fs::write(temp.path().join("random.yaml"), "other: {}").unwrap();

        let result = discover_yaml_files(temp.path()).unwrap();

        assert_eq!(result.product_files.len(), 1);
        assert_eq!(result.type_files.len(), 1);
        assert_eq!(result.entity_files.len(), 1);
        assert_eq!(result.function_files.len(), 1);
        assert_eq!(result.other_files.len(), 1);
        assert_eq!(result.file_count(), 5);
    }

    #[test]
    fn test_all_files_ordering() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("functions.yaml"), "").unwrap();
        fs::write(temp.path().join("product.yaml"), "").unwrap();
        fs::write(temp.path().join("entities.yaml"), "").unwrap();

        let result = discover_yaml_files(temp.path()).unwrap();
        let all = result.all_files();

        // Product should come before entities, entities before functions
        let product_idx = all
            .iter()
            .position(|p| p.file_name().unwrap() == "product.yaml")
            .unwrap();
        let entities_idx = all
            .iter()
            .position(|p| p.file_name().unwrap() == "entities.yaml")
            .unwrap();
        let functions_idx = all
            .iter()
            .position(|p| p.file_name().unwrap() == "functions.yaml")
            .unwrap();

        assert!(product_idx < entities_idx);
        assert!(entities_idx < functions_idx);
    }
}
