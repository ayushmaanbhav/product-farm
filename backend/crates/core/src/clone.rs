//! Deep clone operations for Product and all related entities
//!
//! This module provides comprehensive cloning of products with all associated data:
//! - Product metadata
//! - AbstractAttributes (templates)
//! - Attributes (concrete instances)
//! - Rules (with updated path references)
//! - ProductFunctionality (functionality definitions)

use chrono::{DateTime, Utc};
use std::collections::HashMap;

// =============================================================================
// CLONE LIMITS (DoS Prevention)
// =============================================================================

/// Maximum number of abstract attributes that can be cloned
pub const MAX_ABSTRACT_ATTRIBUTES_TO_CLONE: usize = 10_000;

/// Maximum number of concrete attributes that can be cloned
pub const MAX_ATTRIBUTES_TO_CLONE: usize = 100_000;

/// Maximum number of rules that can be cloned
pub const MAX_RULES_TO_CLONE: usize = 10_000;

/// Maximum number of functionalities that can be cloned
pub const MAX_FUNCTIONALITIES_TO_CLONE: usize = 1_000;

use crate::{
    AbstractAttribute, AbstractAttributeRelatedAttribute, AbstractAttributeTag, AbstractPath,
    Attribute, AttributeDisplayName, ConcretePath, CoreError, CoreResult, Product, ProductFunctionality,
    ProductFunctionalityStatus, FunctionalityRequiredAttribute, ProductId, Rule, RuleId,
    RuleInputAttribute, RuleOutputAttribute,
};

/// Selection filters for selective cloning
#[derive(Debug, Clone, Default)]
pub struct CloneSelections {
    /// Component types to include (empty = all)
    pub components: Vec<String>,
    /// Datatype IDs to include (empty = all)
    pub datatypes: Vec<String>,
    /// Enumeration IDs to include (empty = all)
    pub enumerations: Vec<String>,
    /// Functionality names to include (empty = all)
    pub functionalities: Vec<String>,
    /// Abstract attribute paths to include (empty = all)
    pub abstract_attributes: Vec<String>,
}

impl CloneSelections {
    /// Returns true if no selections are specified (clone everything)
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
            && self.datatypes.is_empty()
            && self.enumerations.is_empty()
            && self.functionalities.is_empty()
            && self.abstract_attributes.is_empty()
    }
}

/// Request to clone a product
#[derive(Debug, Clone)]
pub struct CloneProductRequest {
    /// ID for the new product
    pub new_product_id: ProductId,
    /// Name for the new product
    pub new_name: String,
    /// Description for the new product (optional)
    pub new_description: Option<String>,
    /// Effective date for the new product
    pub effective_from: DateTime<Utc>,
    /// Optional selections for partial cloning
    pub selections: Option<CloneSelections>,
    /// Whether to clone concrete attributes (values). Default true for backward compat.
    pub clone_concrete_attributes: bool,
}

/// Result of a product clone operation
#[derive(Debug, Clone)]
pub struct CloneProductResult {
    /// The cloned product
    pub product: Product,
    /// Cloned abstract attributes (mapping: old path -> new attribute)
    pub abstract_attributes: Vec<AbstractAttribute>,
    /// Cloned concrete attributes
    pub attributes: Vec<Attribute>,
    /// Cloned rules
    pub rules: Vec<Rule>,
    /// Cloned functionalities
    pub functionalities: Vec<ProductFunctionality>,
    /// Path mapping from old to new (for reference updates)
    pub path_mapping: HashMap<String, String>,
}

/// Service for deep cloning products with all related entities
pub struct ProductCloneService;

impl ProductCloneService {
    /// Deep clone a product and all its related entities
    ///
    /// This clones:
    /// - Product metadata (with new ID, name, status=Draft)
    /// - All AbstractAttributes (with updated paths/product_id) - filtered by selections
    /// - All Attributes (with updated paths/product_id) - filtered by selections
    /// - All Rules (with updated product_id and attribute references)
    /// - All ProductFunctionality entries - filtered by selections
    pub fn clone_product(
        source_product: &Product,
        source_abstract_attrs: &[AbstractAttribute],
        source_attrs: &[Attribute],
        source_rules: &[Rule],
        source_functionalities: &[ProductFunctionality],
        request: CloneProductRequest,
    ) -> CoreResult<CloneProductResult> {
        let old_product_id = source_product.id.as_str();
        let new_product_id = request.new_product_id.as_str();

        // Build path mapping for later reference updates
        let mut path_mapping: HashMap<String, String> = HashMap::new();

        // 1. Clone the product
        let mut cloned_product = Product::clone_from(
            source_product,
            request.new_product_id.clone(),
            request.new_name,
            request.effective_from,
        );

        // Apply new description if provided
        if let Some(desc) = request.new_description {
            cloned_product = cloned_product.with_description(desc);
        }

        // 2. Filter and clone abstract attributes based on selections
        let filtered_abstract_attrs = Self::filter_abstract_attrs(
            source_abstract_attrs,
            &request.selections,
        );

        // Check bounds before allocation (DoS prevention)
        if filtered_abstract_attrs.len() > MAX_ABSTRACT_ATTRIBUTES_TO_CLONE {
            return Err(CoreError::ValidationFailed {
                field: "abstract_attributes".to_string(),
                message: format!(
                    "Too many abstract attributes to clone ({} > {})",
                    filtered_abstract_attrs.len(),
                    MAX_ABSTRACT_ATTRIBUTES_TO_CLONE
                ),
            });
        }

        let cloned_abstract_attrs: Vec<AbstractAttribute> = filtered_abstract_attrs
            .iter()
            .map(|attr| {
                let cloned = Self::clone_abstract_attribute(attr, old_product_id, new_product_id);
                path_mapping.insert(
                    attr.abstract_path.as_str().to_string(),
                    cloned.abstract_path.as_str().to_string(),
                );
                cloned
            })
            .collect();

        // 3. Clone concrete attributes only if requested
        let cloned_attrs: Vec<Attribute> = if request.clone_concrete_attributes {
            // Count first to check bounds
            let matching_count = source_attrs
                .iter()
                .filter(|attr| path_mapping.contains_key(attr.abstract_path.as_str()))
                .count();

            if matching_count > MAX_ATTRIBUTES_TO_CLONE {
                return Err(CoreError::ValidationFailed {
                    field: "attributes".to_string(),
                    message: format!(
                        "Too many attributes to clone ({} > {})",
                        matching_count,
                        MAX_ATTRIBUTES_TO_CLONE
                    ),
                });
            }

            // Filter concrete attrs to only include those whose abstract_path is in the cloned set
            source_attrs
                .iter()
                .filter(|attr| path_mapping.contains_key(attr.abstract_path.as_str()))
                .map(|attr| {
                    Self::clone_attribute(attr, old_product_id, new_product_id, &path_mapping)
                })
                .collect()
        } else {
            Vec::new()
        };

        // 4. Clone rules - only include rules that reference cloned attributes
        // Count first to check bounds
        let matching_rules_count = source_rules
            .iter()
            .filter(|rule| Self::rule_references_cloned_attrs(rule, &path_mapping))
            .count();

        if matching_rules_count > MAX_RULES_TO_CLONE {
            return Err(CoreError::ValidationFailed {
                field: "rules".to_string(),
                message: format!(
                    "Too many rules to clone ({} > {})",
                    matching_rules_count,
                    MAX_RULES_TO_CLONE
                ),
            });
        }

        let cloned_rules: Vec<Rule> = source_rules
            .iter()
            .filter(|rule| Self::rule_references_cloned_attrs(rule, &path_mapping))
            .map(|rule| Self::clone_rule(rule, new_product_id, &path_mapping))
            .collect();

        // 5. Filter and clone functionalities
        let filtered_funcs = Self::filter_functionalities(
            source_functionalities,
            &request.selections,
        );

        // Check bounds before allocation
        if filtered_funcs.len() > MAX_FUNCTIONALITIES_TO_CLONE {
            return Err(CoreError::ValidationFailed {
                field: "functionalities".to_string(),
                message: format!(
                    "Too many functionalities to clone ({} > {})",
                    filtered_funcs.len(),
                    MAX_FUNCTIONALITIES_TO_CLONE
                ),
            });
        }

        let cloned_functionalities: Vec<ProductFunctionality> = filtered_funcs
            .iter()
            .map(|func| Self::clone_functionality(func, &request.new_product_id, &path_mapping))
            .collect();

        Ok(CloneProductResult {
            product: cloned_product,
            abstract_attributes: cloned_abstract_attrs,
            attributes: cloned_attrs,
            rules: cloned_rules,
            functionalities: cloned_functionalities,
            path_mapping,
        })
    }

    /// Filter abstract attributes based on selections
    fn filter_abstract_attrs<'a>(
        attrs: &'a [AbstractAttribute],
        selections: &Option<CloneSelections>,
    ) -> Vec<&'a AbstractAttribute> {
        match selections {
            None => attrs.iter().collect(),
            Some(sel) if sel.is_empty() => attrs.iter().collect(),
            Some(sel) => {
                attrs.iter().filter(|attr| {
                    // Check component filter
                    let component_match = sel.components.is_empty()
                        || sel.components.contains(&attr.component_type);

                    // Check datatype filter
                    let datatype_match = sel.datatypes.is_empty()
                        || sel.datatypes.iter().any(|d| d == attr.datatype_id.as_str());

                    // Check enumeration filter (if attr has enum_name)
                    let enum_match = sel.enumerations.is_empty()
                        || attr.enum_name.as_ref().map(|e| sel.enumerations.contains(e)).unwrap_or(true);

                    // Check explicit abstract attribute path filter
                    let path_match = sel.abstract_attributes.is_empty()
                        || sel.abstract_attributes.contains(&attr.abstract_path.as_str().to_string());

                    // All conditions must pass (AND logic)
                    component_match && datatype_match && enum_match && path_match
                }).collect()
            }
        }
    }

    /// Filter functionalities based on selections
    fn filter_functionalities<'a>(
        funcs: &'a [ProductFunctionality],
        selections: &Option<CloneSelections>,
    ) -> Vec<&'a ProductFunctionality> {
        match selections {
            None => funcs.iter().collect(),
            Some(sel) if sel.is_empty() => funcs.iter().collect(),
            Some(sel) if sel.functionalities.is_empty() => funcs.iter().collect(),
            Some(sel) => {
                funcs.iter().filter(|func| {
                    sel.functionalities.contains(&func.name)
                }).collect()
            }
        }
    }

    /// Check if a rule references any of the cloned attributes
    fn rule_references_cloned_attrs(rule: &Rule, path_mapping: &HashMap<String, String>) -> bool {
        // A rule should be cloned if ANY of its input or output attributes are being cloned
        let has_cloned_input = rule.input_attributes.iter()
            .any(|attr| path_mapping.contains_key(attr.path.as_str()));
        let has_cloned_output = rule.output_attributes.iter()
            .any(|attr| path_mapping.contains_key(attr.path.as_str()));
        has_cloned_input || has_cloned_output
    }

    /// Clone an abstract attribute with updated paths
    fn clone_abstract_attribute(
        source: &AbstractAttribute,
        old_product_id: &str,
        new_product_id: &str,
    ) -> AbstractAttribute {
        // Parse original path to extract components
        let parsed = source.abstract_path.parse();

        // Build new path with new product ID
        let new_path = if let Some(p) = parsed {
            AbstractPath::build(
                new_product_id,
                &p.component_type,
                p.component_id.as_deref(),
                &p.attribute_name,
            )
        } else {
            // Fallback: simple string replacement
            AbstractPath::new(
                source.abstract_path.as_str().replacen(old_product_id, new_product_id, 1)
            )
        };

        // Clone display names with updated paths
        let new_display_names: Vec<AttributeDisplayName> = source
            .display_names
            .iter()
            .map(|dn| AttributeDisplayName {
                product_id: ProductId::new(new_product_id),
                display_name: dn.display_name.clone(),
                abstract_path: dn.abstract_path.as_ref().map(|ap| {
                    AbstractPath::new(ap.as_str().replacen(old_product_id, new_product_id, 1))
                }),
                path: dn.path.as_ref().map(|p| {
                    ConcretePath::new(p.as_str().replacen(old_product_id, new_product_id, 1))
                }),
                display_name_format: dn.display_name_format.clone(),
                order: dn.order,
            })
            .collect();

        // Clone tags with updated paths
        let new_tags: Vec<AbstractAttributeTag> = source
            .tags
            .iter()
            .map(|t| AbstractAttributeTag {
                abstract_path: AbstractPath::new(
                    t.abstract_path.as_str().replacen(old_product_id, new_product_id, 1)
                ),
                tag: t.tag.clone(),
                product_id: ProductId::new(new_product_id),
                order: t.order,
            })
            .collect();

        // Clone related attributes with updated paths
        let new_related: Vec<AbstractAttributeRelatedAttribute> = source
            .related_attributes
            .iter()
            .map(|r| AbstractAttributeRelatedAttribute {
                abstract_path: AbstractPath::new(
                    r.abstract_path.as_str().replacen(old_product_id, new_product_id, 1)
                ),
                reference_abstract_path: AbstractPath::new(
                    r.reference_abstract_path.as_str().replacen(old_product_id, new_product_id, 1)
                ),
                relationship: r.relationship.clone(),
                order: r.order,
            })
            .collect();

        AbstractAttribute {
            abstract_path: new_path,
            product_id: ProductId::new(new_product_id),
            component_type: source.component_type.clone(),
            component_id: source.component_id.clone(),
            datatype_id: source.datatype_id.clone(),
            enum_name: source.enum_name.clone(),
            constraint_expression: source.constraint_expression.clone(),
            immutable: source.immutable,
            description: source.description.clone(),
            display_names: new_display_names,
            tags: new_tags,
            related_attributes: new_related,
        }
    }

    /// Clone a concrete attribute with updated paths
    fn clone_attribute(
        source: &Attribute,
        old_product_id: &str,
        new_product_id: &str,
        path_mapping: &HashMap<String, String>,
    ) -> Attribute {
        // Update concrete path
        let new_path = ConcretePath::new(
            source.path.as_str().replacen(old_product_id, new_product_id, 1)
        );

        // Update abstract path reference
        let new_abstract_path = path_mapping
            .get(source.abstract_path.as_str())
            .map(AbstractPath::new)
            .unwrap_or_else(|| {
                AbstractPath::new(
                    source.abstract_path.as_str().replacen(old_product_id, new_product_id, 1)
                )
            });

        // Clone display names
        let new_display_names: Vec<AttributeDisplayName> = source
            .display_names
            .iter()
            .map(|dn| AttributeDisplayName {
                product_id: ProductId::new(new_product_id),
                display_name: dn.display_name.clone(),
                abstract_path: dn.abstract_path.as_ref().map(|ap| {
                    path_mapping
                        .get(ap.as_str())
                        .map(AbstractPath::new)
                        .unwrap_or_else(|| {
                            AbstractPath::new(ap.as_str().replacen(old_product_id, new_product_id, 1))
                        })
                }),
                path: dn.path.as_ref().map(|p| {
                    ConcretePath::new(p.as_str().replacen(old_product_id, new_product_id, 1))
                }),
                display_name_format: dn.display_name_format.clone(),
                order: dn.order,
            })
            .collect();

        // Note: rule_id is NOT cloned - it will be updated after rules are cloned
        // The caller should update rule_id references based on rule cloning results

        let now = chrono::Utc::now();
        Attribute {
            path: new_path,
            abstract_path: new_abstract_path,
            product_id: ProductId::new(new_product_id),
            value_type: source.value_type.clone(),
            value: source.value.clone(),
            rule_id: None, // Will be updated by caller after rules are cloned
            display_names: new_display_names,
            created_at: now,
            updated_at: now,
        }
    }

    /// Clone a rule with updated product ID and attribute references
    fn clone_rule(
        source: &Rule,
        new_product_id: &str,
        path_mapping: &HashMap<String, String>,
    ) -> Rule {
        // Generate new rule ID
        let new_rule_id = RuleId::new();

        // Update input attribute paths
        let new_input_attrs: Vec<RuleInputAttribute> = source
            .input_attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| {
                let path_str = attr.path.as_str();
                let new_path = path_mapping
                    .get(path_str)
                    .map(AbstractPath::new)
                    .unwrap_or_else(|| attr.path.clone());
                RuleInputAttribute {
                    rule_id: new_rule_id.clone(),
                    path: new_path,
                    order: idx as i32,
                }
            })
            .collect();

        // Update output attribute paths
        let new_output_attrs: Vec<RuleOutputAttribute> = source
            .output_attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| {
                let path_str = attr.path.as_str();
                let new_path = path_mapping
                    .get(path_str)
                    .map(AbstractPath::new)
                    .unwrap_or_else(|| attr.path.clone());
                RuleOutputAttribute {
                    rule_id: new_rule_id.clone(),
                    path: new_path,
                    order: idx as i32,
                }
            })
            .collect();

        let now = chrono::Utc::now();
        Rule {
            id: new_rule_id,
            product_id: ProductId::new(new_product_id),
            rule_type: source.rule_type.clone(),
            input_attributes: new_input_attrs,
            output_attributes: new_output_attrs,
            display_expression: source.display_expression.clone(),
            display_expression_version: source.display_expression_version.clone(),
            compiled_expression: source.compiled_expression.clone(),
            description: source.description.clone(),
            order_index: source.order_index,
            enabled: source.enabled,
            created_at: now,
            updated_at: now,
        }
    }

    /// Clone a functionality with updated product ID and attribute references
    fn clone_functionality(
        source: &ProductFunctionality,
        new_product_id: &ProductId,
        path_mapping: &HashMap<String, String>,
    ) -> ProductFunctionality {
        // Update required attributes with new paths
        let new_required_attrs: Vec<FunctionalityRequiredAttribute> = source
            .required_attributes
            .iter()
            .map(|req| {
                let new_path = path_mapping
                    .get(req.abstract_path.as_str())
                    .map(AbstractPath::new)
                    .unwrap_or_else(|| req.abstract_path.clone());

                FunctionalityRequiredAttribute {
                    functionality_id: req.functionality_id.clone(),
                    abstract_path: new_path,
                    description: req.description.clone(),
                    order: req.order,
                }
            })
            .collect();

        let now = chrono::Utc::now();
        ProductFunctionality {
            id: source.id.clone(),
            name: source.name.clone(),
            product_id: new_product_id.clone(),
            immutable: false, // Cloned functionality starts as mutable
            description: source.description.clone(),
            required_attributes: new_required_attrs,
            status: ProductFunctionalityStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FunctionalityId;
    use serde_json::json;

    fn create_test_product() -> Product {
        Product::new("product-v1", "Product V1", "trading", Utc::now())
    }

    fn create_test_abstract_attrs() -> Vec<AbstractAttribute> {
        vec![
            AbstractAttribute::new(
                "product-v1:abstract-path:MARKET:price",
                "product-v1",
                "MARKET",
                "decimal",
            )
            .with_tag_name("pricing", 0)
            .with_description("Market price"),

            AbstractAttribute::new(
                "product-v1:abstract-path:MARKET:volume",
                "product-v1",
                "MARKET",
                "integer",
            )
            .with_tag_name("volume", 0),
        ]
    }

    fn create_test_rules() -> Vec<Rule> {
        vec![
            Rule::from_json_logic("product-v1", "calc", json!({"*": [{"var": "price"}, {"var": "volume"}]}))
                .with_inputs(["product-v1:abstract-path:MARKET:price", "product-v1:abstract-path:MARKET:volume"])
                .with_outputs(["product-v1:abstract-path:MARKET:total"])
                .with_order(0),
        ]
    }

    fn create_test_functionalities() -> Vec<ProductFunctionality> {
        let mut func = ProductFunctionality::new(
            FunctionalityId::new("pricing"),
            "pricing-calculation",
            ProductId::new("product-v1"),
            "Calculate pricing",
        );
        func.add_required_attribute(
            AbstractPath::new("product-v1:abstract-path:MARKET:price"),
            "Market price input",
        );
        vec![func]
    }

    fn default_clone_request(id: &str, name: &str) -> CloneProductRequest {
        CloneProductRequest {
            new_product_id: ProductId::new(id),
            new_name: name.to_string(),
            new_description: None,
            effective_from: Utc::now(),
            selections: None,
            clone_concrete_attributes: true,
        }
    }

    #[test]
    fn test_clone_product_basic() {
        let source_product = create_test_product();
        let source_attrs = create_test_abstract_attrs();
        let source_concrete_attrs: Vec<Attribute> = vec![];
        let source_rules = create_test_rules();
        let source_funcs = create_test_functionalities();

        let request = default_clone_request("product-v2", "Product V2");

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_attrs,
            &source_concrete_attrs,
            &source_rules,
            &source_funcs,
            request,
        ).unwrap();

        // Verify product cloned
        assert_eq!(result.product.id.as_str(), "product-v2");
        assert_eq!(result.product.name, "Product V2");
        assert_eq!(result.product.parent_product_id.as_ref().unwrap().as_str(), "product-v1");

        // Verify abstract attributes cloned with new paths
        assert_eq!(result.abstract_attributes.len(), 2);
        assert!(result.abstract_attributes[0].abstract_path.as_str().contains("product-v2"));
        assert_eq!(result.abstract_attributes[0].product_id.as_str(), "product-v2");

        // Verify rules cloned with new references
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].product_id.as_str(), "product-v2");
        assert!(result.rules[0].input_attributes[0].path.as_str().contains("product-v2"));

        // Verify functionalities cloned
        assert_eq!(result.functionalities.len(), 1);
        assert_eq!(result.functionalities[0].product_id.as_str(), "product-v2");
    }

    #[test]
    fn test_clone_preserves_path_mapping() {
        let source_product = create_test_product();
        let source_attrs = create_test_abstract_attrs();
        let source_concrete_attrs: Vec<Attribute> = vec![];
        let source_rules = create_test_rules();
        let source_funcs: Vec<ProductFunctionality> = vec![];

        let request = default_clone_request("product-v2", "Product V2");

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_attrs,
            &source_concrete_attrs,
            &source_rules,
            &source_funcs,
            request,
        ).unwrap();

        // Path mapping should have entries for each abstract attribute
        assert_eq!(result.path_mapping.len(), 2);
        assert!(result.path_mapping.contains_key("product-v1:abstract-path:MARKET:price"));
        assert_eq!(
            result.path_mapping.get("product-v1:abstract-path:MARKET:price").unwrap(),
            "product-v2:abstract-path:MARKET:price"
        );
    }

    #[test]
    fn test_clone_updates_tags() {
        let source_product = create_test_product();
        let mut source_attrs = create_test_abstract_attrs();

        // Add tag with explicit path
        source_attrs[0] = source_attrs[0].clone().with_tag_name("test-tag", 1);

        let request = default_clone_request("product-v2", "Product V2");

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_attrs,
            &[],
            &[],
            &[],
            request,
        ).unwrap();

        // Tags should have updated paths
        let cloned_attr = &result.abstract_attributes[0];
        assert!(!cloned_attr.tags.is_empty());
        for tag in &cloned_attr.tags {
            assert!(tag.abstract_path.as_str().contains("product-v2"));
            assert_eq!(tag.product_id.as_str(), "product-v2");
        }
    }

    #[test]
    fn test_clone_generates_new_rule_ids() {
        let source_product = create_test_product();
        let source_attrs = create_test_abstract_attrs();
        let source_rules = create_test_rules();
        let original_rule_id = source_rules[0].id.clone();

        let request = default_clone_request("product-v2", "Product V2");

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_attrs,  // Need to include attrs so rules are cloned
            &[],
            &source_rules,
            &[],
            request,
        ).unwrap();

        // Rules referencing cloned attributes should be cloned
        assert_eq!(result.rules.len(), 1);
        // New rule should have different ID
        assert_ne!(result.rules[0].id, original_rule_id);
    }

    #[test]
    fn test_selective_clone_filters_by_component() {
        let source_product = create_test_product();
        let mut source_attrs = create_test_abstract_attrs();

        // Add an attribute with different component type
        source_attrs.push(
            AbstractAttribute::new(
                "product-v1:abstract-path:TRADE:quantity",
                "product-v1",
                "TRADE",  // Different component
                "integer",
            )
        );

        // Create request with component filter - only MARKET
        let request = CloneProductRequest {
            new_product_id: ProductId::new("product-v2"),
            new_name: "Product V2".to_string(),
            new_description: None,
            effective_from: Utc::now(),
            selections: Some(CloneSelections {
                components: vec!["MARKET".to_string()],
                datatypes: vec![],
                enumerations: vec![],
                functionalities: vec![],
                abstract_attributes: vec![],
            }),
            clone_concrete_attributes: true,
        };

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_attrs,
            &[],
            &[],
            &[],
            request,
        ).unwrap();

        // Should only clone MARKET attributes (2 of 3)
        assert_eq!(result.abstract_attributes.len(), 2);
        for attr in &result.abstract_attributes {
            assert_eq!(attr.component_type, "MARKET");
        }
    }

    #[test]
    fn test_selective_clone_without_concrete_attrs() {
        let source_product = create_test_product();
        let source_abstract_attrs = create_test_abstract_attrs();

        // Create a concrete attribute
        let source_attrs = vec![
            Attribute::new_fixed_value(
                ConcretePath::new("product-v1:path:MARKET:inst1:price"),
                AbstractPath::new("product-v1:abstract-path:MARKET:price"),
                ProductId::new("product-v1"),
                crate::Value::String("100.50".to_string()),
            ),
        ];

        // Clone without concrete attributes
        let request = CloneProductRequest {
            new_product_id: ProductId::new("product-v2"),
            new_name: "Product V2".to_string(),
            new_description: None,
            effective_from: Utc::now(),
            selections: None,
            clone_concrete_attributes: false,  // Don't clone concrete attrs
        };

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_abstract_attrs,
            &source_attrs,
            &[],
            &[],
            request,
        ).unwrap();

        // Abstract attributes should be cloned
        assert_eq!(result.abstract_attributes.len(), 2);
        // Concrete attributes should NOT be cloned
        assert_eq!(result.attributes.len(), 0);
    }
}
