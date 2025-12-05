//! In-memory storage for all Product-FARM entities
//!
//! This provides a shared storage layer that can be replaced with
//! persistent storage (Dgraph, PostgreSQL) in production.

use hashbrown::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use product_farm_core::{
    AbstractAttribute, Attribute, DataType, Product, ProductFunctionality,
    ProductTemplateEnumeration, Rule,
};

/// In-memory storage for all entities
#[derive(Debug, Default)]
pub struct EntityStore {
    // Products
    pub products: HashMap<String, Product>,

    // Abstract attributes (keyed by abstract_path)
    pub abstract_attributes: HashMap<String, AbstractAttribute>,
    pub abstract_attrs_by_product: HashMap<String, Vec<String>>,

    // Concrete attributes (keyed by path)
    pub attributes: HashMap<String, Attribute>,
    pub attrs_by_product: HashMap<String, Vec<String>>,

    // Rules (keyed by rule_id)
    pub rules: HashMap<String, Rule>,
    pub rules_by_product: HashMap<String, Vec<String>>,

    // Datatypes (keyed by id)
    pub datatypes: HashMap<String, DataType>,

    // Product functionalities (keyed by "product_id:name")
    pub functionalities: HashMap<String, ProductFunctionality>,
    pub funcs_by_product: HashMap<String, Vec<String>>,

    // Template enumerations (keyed by id)
    pub enumerations: HashMap<String, ProductTemplateEnumeration>,
    pub enums_by_template: HashMap<String, Vec<String>>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Product helpers
    pub fn get_product(&self, id: &str) -> Option<&Product> {
        self.products.get(id)
    }

    pub fn get_product_mut(&mut self, id: &str) -> Option<&mut Product> {
        self.products.get_mut(id)
    }

    // Abstract attribute helpers
    /// Generate a functionality key from product_id and name.
    ///
    /// # Safety
    /// This function assumes both `product_id` and `name` have been validated
    /// and do not contain the separator character `:`. The validation patterns
    /// in `product_farm_core::validation` ensure this when properly applied.
    pub fn functionality_key(product_id: &str, name: &str) -> String {
        debug_assert!(
            !product_id.contains(':') && !name.contains(':'),
            "functionality_key requires validated inputs without ':'",
        );
        format!("{}:{}", product_id, name)
    }

    pub fn get_abstract_attrs_for_product(&self, product_id: &str) -> Vec<&AbstractAttribute> {
        self.abstract_attrs_by_product
            .get(product_id)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|path| self.abstract_attributes.get(path))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_abstract_attrs_by_component(
        &self,
        product_id: &str,
        component_type: &str,
        component_id: Option<&str>,
    ) -> Vec<&AbstractAttribute> {
        self.get_abstract_attrs_for_product(product_id)
            .into_iter()
            .filter(|attr| {
                attr.component_type == component_type
                    && (component_id.is_none()
                        || attr.component_id.as_deref() == component_id)
            })
            .collect()
    }

    pub fn get_abstract_attrs_by_tag(&self, product_id: &str, tag: &str) -> Vec<&AbstractAttribute> {
        self.get_abstract_attrs_for_product(product_id)
            .into_iter()
            .filter(|attr| attr.has_tag(tag))
            .collect()
    }

    pub fn get_abstract_attrs_by_tags(
        &self,
        product_id: &str,
        tags: &[String],
        match_all: bool,
    ) -> Vec<&AbstractAttribute> {
        self.get_abstract_attrs_for_product(product_id)
            .into_iter()
            .filter(|attr| {
                if match_all {
                    tags.iter().all(|t| attr.has_tag(t))
                } else {
                    tags.iter().any(|t| attr.has_tag(t))
                }
            })
            .collect()
    }

    pub fn get_abstract_attrs_by_functionality(
        &self,
        product_id: &str,
        functionality_name: &str,
    ) -> Vec<&AbstractAttribute> {
        let key = Self::functionality_key(product_id, functionality_name);
        if let Some(func) = self.functionalities.get(&key) {
            func.required_attributes
                .iter()
                .filter_map(|req| self.abstract_attributes.get(req.abstract_path.as_str()))
                .collect()
        } else {
            vec![]
        }
    }

    // Attribute helpers
    pub fn get_attrs_for_product(&self, product_id: &str) -> Vec<&Attribute> {
        self.attrs_by_product
            .get(product_id)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|path| self.attributes.get(path))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_attrs_by_tag(&self, product_id: &str, tag: &str) -> Vec<&Attribute> {
        // Get attributes whose abstract attribute has the tag
        self.get_attrs_for_product(product_id)
            .into_iter()
            .filter(|attr| {
                self.abstract_attributes
                    .get(attr.abstract_path.as_str())
                    .map(|aa| aa.has_tag(tag))
                    .unwrap_or(false)
            })
            .collect()
    }

    // Rule helpers
    pub fn get_rules_for_product(&self, product_id: &str) -> Vec<&Rule> {
        self.rules_by_product
            .get(product_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.rules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    // Functionality helpers
    pub fn get_funcs_for_product(&self, product_id: &str) -> Vec<&ProductFunctionality> {
        self.funcs_by_product
            .get(product_id)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.functionalities.get(key))
                    .collect()
            })
            .unwrap_or_default()
    }

    // Enumeration helpers
    pub fn get_enums_for_template(&self, template_type: &str) -> Vec<&ProductTemplateEnumeration> {
        self.enums_by_template
            .get(template_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.enumerations.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Thread-safe shared store
pub type SharedStore = Arc<RwLock<EntityStore>>;

/// Create a new shared store
pub fn create_shared_store() -> SharedStore {
    Arc::new(RwLock::new(EntityStore::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    

    #[test]
    fn test_store_operations() {
        let mut store = EntityStore::new();

        // Add a product
        let product = Product::new("test-product", "Test Product", "insurance", Utc::now());
        store.products.insert("test-product".to_string(), product);

        // Add an abstract attribute
        let attr = AbstractAttribute::new(
            "test-product:abstract-path:cover:premium",
            "test-product",
            "cover",
            "decimal",
        )
        .with_tag_name("input", 0);

        let path = attr.abstract_path.as_str().to_string();
        store.abstract_attributes.insert(path.clone(), attr);
        store
            .abstract_attrs_by_product
            .entry("test-product".to_string())
            .or_default()
            .push(path);

        // Query by tag
        let by_tag = store.get_abstract_attrs_by_tag("test-product", "input");
        assert_eq!(by_tag.len(), 1);

        // Query by component
        let by_comp = store.get_abstract_attrs_by_component("test-product", "cover", None);
        assert_eq!(by_comp.len(), 1);
    }
}
