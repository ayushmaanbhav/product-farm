//! Schema transformation from YAML to core types.
//!
//! Transforms parsed YAML documents into Product-FARM core types,
//! using intelligent interpretation for field classification.

use crate::error::{LoaderError, LoaderResult};
use crate::farmscript;
use crate::interpreter::{
    AttributeSemantics, Confidence, IntelligentInterpreter,
    InterpretationContext, extract_formula_references,
};
use crate::llm_prompt::{LlmPromptPreprocessor, FunctionMetadata, OutputMetadata};
use crate::parser::ParsedDocument;
use crate::report::{
    AttributeReport, EntityReport, FunctionReport, InferenceReport, LowConfidenceItem,
};
use crate::schema::{
    LayerDefinition, LayerVisibilityConfig, MasterSchema, YamlDocument, YamlEntity,
    YamlFieldDefinition, YamlFunction,
};
use product_farm_core::{
    AbstractAttribute, AbstractPath, DataType, DataTypeId, PrimitiveType,
    Product, ProductFunctionality, ProductFunctionalityStatus, ProductId, ProductStatus,
    Rule, RuleBuilder, RuleEvaluator, EvaluatorType, TemplateType, Value,
};
use std::collections::HashMap;

/// Transforms YAML documents into core types.
pub struct SchemaTransformer {
    interpreter: IntelligentInterpreter,
}

impl Default for SchemaTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaTransformer {
    pub fn new() -> Self {
        Self {
            interpreter: IntelligentInterpreter::new(),
        }
    }

    /// Transform parsed documents into a master schema.
    pub fn transform(&self, documents: Vec<ParsedDocument>) -> LoaderResult<MasterSchema> {
        let (schema, _) = self.transform_internal(documents)?;
        Ok(schema)
    }

    /// Transform with inference report generation.
    pub fn transform_with_report(
        &self,
        documents: Vec<ParsedDocument>,
    ) -> LoaderResult<(MasterSchema, InferenceReport)> {
        self.transform_internal(documents)
    }

    /// Internal transform that generates both schema and report.
    fn transform_internal(
        &self,
        documents: Vec<ParsedDocument>,
    ) -> LoaderResult<(MasterSchema, InferenceReport)> {
        // 1. Merge all documents
        let (merged, source_files) = self.merge_documents(documents)?;

        // 2. Derive product metadata
        let product = self.derive_product(&merged, &source_files)?;
        let product_id = product.id.clone();

        // 3. Build interpretation context
        let mut context = self.build_context(&merged);

        // 4. Transform types/enums
        let data_types = self.transform_types(&merged, &product_id)?;

        // 5. Transform entities to attributes
        let (attributes, entity_reports) =
            self.transform_entities(&merged, &product_id, &mut context)?;

        // 6. Transform functions to rules
        let (rules, function_reports) =
            self.transform_functions(&merged, &product_id, &attributes)?;

        // 7. Transform functionalities
        let functionalities = self.transform_functionalities(&merged, &product_id)?;

        // 8. Extract layer config
        let layer_config = self.extract_layer_config(&merged)?;

        // Collect low confidence items before moving reports
        let low_confidence_items = self.collect_low_confidence_items(&entity_reports, &function_reports);

        // Build master schema
        let schema = MasterSchema {
            product,
            data_types,
            attributes,
            rules,
            functionalities,
            layer_config,
        };

        // Build inference report
        let report = InferenceReport {
            product_id: product_id.to_string(),
            source_files,
            entities: entity_reports,
            functions: function_reports,
            warnings: Vec::new(),
            low_confidence_items,
        };

        Ok((schema, report))
    }

    /// Merge multiple documents into one.
    fn merge_documents(
        &self,
        documents: Vec<ParsedDocument>,
    ) -> LoaderResult<(YamlDocument, Vec<std::path::PathBuf>)> {
        let mut merged = YamlDocument::default();
        let mut source_files = Vec::new();

        for doc in documents {
            source_files.push(doc.source);

            // Merge product (first one wins)
            if merged.product.is_none() && doc.document.product.is_some() {
                merged.product = doc.document.product;
            }

            // Merge types
            if let Some(types) = doc.document.types {
                let target = merged.types.get_or_insert_with(HashMap::new);
                target.extend(types);
            }

            // Merge entities
            if let Some(entities) = doc.document.entities {
                let target = merged.entities.get_or_insert_with(HashMap::new);
                target.extend(entities);
            }

            // Merge functions
            if let Some(functions) = doc.document.functions {
                let target = merged.functions.get_or_insert_with(HashMap::new);
                target.extend(functions);
            }

            // Merge functionalities
            if let Some(functionalities) = doc.document.functionalities {
                let target = merged.functionalities.get_or_insert_with(HashMap::new);
                target.extend(functionalities);
            }

            // Merge constraints
            if let Some(constraints) = doc.document.constraints {
                let target = merged.constraints.get_or_insert_with(HashMap::new);
                target.extend(constraints);
            }

            // Merge layers
            if let Some(layers) = doc.document.layers {
                let target = merged.layers.get_or_insert_with(HashMap::new);
                target.extend(layers);
            }
        }

        Ok((merged, source_files))
    }

    /// Derive product metadata.
    fn derive_product(
        &self,
        merged: &YamlDocument,
        source_files: &[std::path::PathBuf],
    ) -> LoaderResult<Product> {
        let meta = merged.product.as_ref();

        // Try to get product ID from YAML, or derive from folder name
        let product_id = meta
            .and_then(|m| m.id.clone())
            .or_else(|| {
                source_files.first().and_then(|p| {
                    p.parent()
                        .and_then(|parent| parent.file_name())
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_string())
                })
            })
            .ok_or(LoaderError::MissingProductId)?;

        let name = meta
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| product_id.clone());

        let description = meta.and_then(|m| m.description.clone());

        let template_type = meta
            .and_then(|m| m.version.clone())
            .unwrap_or_else(|| "generic".to_string());

        let now = chrono::Utc::now();

        Ok(Product {
            id: ProductId::new(&product_id),
            name,
            status: ProductStatus::Draft,
            template_type: TemplateType::new(&template_type),
            parent_product_id: None,
            effective_from: now,
            expiry_at: None,
            description,
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    /// Build interpretation context from merged document.
    fn build_context(&self, merged: &YamlDocument) -> InterpretationContext {
        let mut context = InterpretationContext::default();

        // Collect known entity names
        if let Some(entities) = &merged.entities {
            for name in entities.keys() {
                context.known_entities.insert(name.clone());
            }
        }

        // Collect formula references
        if let Some(functions) = &merged.functions {
            for func in functions.values() {
                if let Some(expr) = &func.expression {
                    let expr_str = serde_json::to_string(expr).unwrap_or_default();
                    for ref_name in extract_formula_references(&expr_str) {
                        context.formula_references.insert(ref_name);
                    }
                }
            }
        }

        context
    }

    /// Transform types/enums to DataTypes.
    fn transform_types(
        &self,
        merged: &YamlDocument,
        product_id: &ProductId,
    ) -> LoaderResult<Vec<DataType>> {
        let mut data_types = Vec::new();

        if let Some(types) = &merged.types {
            for (name, _type_def) in types {
                // Create a simple DataType - actual type inference happens in interpreter
                let data_type = DataType::new(
                    format!("{}:{}", product_id, name),
                    PrimitiveType::String, // Default, will be refined
                );
                data_types.push(data_type);
            }
        }

        Ok(data_types)
    }

    /// Transform entities to AbstractAttributes.
    fn transform_entities(
        &self,
        merged: &YamlDocument,
        product_id: &ProductId,
        context: &mut InterpretationContext,
    ) -> LoaderResult<(Vec<AbstractAttribute>, Vec<EntityReport>)> {
        let mut attributes = Vec::new();
        let mut reports = Vec::new();

        if let Some(entities) = &merged.entities {
            for (entity_name, entity) in entities {
                context.current_entity = Some(entity_name.clone());

                let (entity_attrs, entity_report) =
                    self.transform_entity(entity_name, entity, product_id, context)?;

                attributes.extend(entity_attrs);
                reports.push(entity_report);
            }
        }

        Ok((attributes, reports))
    }

    /// Transform a single entity.
    fn transform_entity(
        &self,
        entity_name: &str,
        entity: &YamlEntity,
        product_id: &ProductId,
        context: &InterpretationContext,
    ) -> LoaderResult<(Vec<AbstractAttribute>, EntityReport)> {
        let mut attributes = Vec::new();
        let mut attr_reports = Vec::new();
        let mut rel_reports = Vec::new();

        let component_type = to_kebab_case(entity_name);

        // Process explicit attributes section
        if let Some(attrs) = &entity.attributes {
            for (attr_name, field_def) in attrs {
                let yaml_value = field_def_to_yaml_value(field_def);
                let interpreted = self.interpreter.interpret_field(attr_name, &yaml_value, context);

                let abstract_path = AbstractPath::new(&format!(
                    "{}:abstract-path:{}:{}",
                    product_id, component_type, to_kebab_case(attr_name)
                ));

                let attr = AbstractAttribute {
                    abstract_path,
                    product_id: product_id.clone(),
                    component_type: component_type.clone(),
                    component_id: None,
                    datatype_id: DataTypeId::new(interpreted.inferred_type.primitive.to_datatype_id()),
                    enum_name: interpreted.enum_values.as_ref().map(|_| attr_name.clone()),
                    constraint_expression: None,
                    immutable: false,
                    description: interpreted.description.clone(),
                    display_names: Vec::new(),
                    tags: Vec::new(),
                    related_attributes: Vec::new(),
                };

                attr_reports.push(AttributeReport {
                    name: attr_name.clone(),
                    inferred_type: interpreted.inferred_type.primitive.to_datatype_id().to_string(),
                    type_confidence: interpreted.inferred_type.confidence,
                    classification: if interpreted.semantics.is_static() { "static" } else { "instance" }.to_string(),
                    classification_confidence: match &interpreted.semantics {
                        AttributeSemantics::Static { confidence } => *confidence,
                        AttributeSemantics::Instance { confidence } => *confidence,
                    },
                    signals_used: interpreted.inferred_type.signals.clone(),
                });

                if let Some(rel) = &interpreted.relationship {
                    rel_reports.push(crate::report::RelationshipReport {
                        name: attr_name.clone(),
                        target: rel.target.clone(),
                        cardinality: rel.cardinality.as_str().to_string(),
                        confidence: rel.confidence,
                    });
                }

                attributes.push(attr);
            }
        }

        // Process flattened fields (catch-all)
        for (field_name, field_value) in &entity.fields {
            // Skip known keywords
            if ["description", "attributes", "relationships"].contains(&field_name.as_str()) {
                continue;
            }

            let interpreted = self.interpreter.interpret_field(field_name, field_value, context);

            let abstract_path = AbstractPath::new(&format!(
                "{}:abstract-path:{}:{}",
                product_id, component_type, to_kebab_case(field_name)
            ));

            let attr = AbstractAttribute {
                abstract_path,
                product_id: product_id.clone(),
                component_type: component_type.clone(),
                component_id: None,
                datatype_id: DataTypeId::new(interpreted.inferred_type.primitive.to_datatype_id()),
                enum_name: interpreted.enum_values.as_ref().map(|_| field_name.clone()),
                constraint_expression: None,
                immutable: false,
                description: interpreted.description.clone(),
                display_names: Vec::new(),
                tags: Vec::new(),
                related_attributes: Vec::new(),
            };

            attr_reports.push(AttributeReport {
                name: field_name.clone(),
                inferred_type: interpreted.inferred_type.primitive.to_datatype_id().to_string(),
                type_confidence: interpreted.inferred_type.confidence,
                classification: if interpreted.semantics.is_static() { "static" } else { "instance" }.to_string(),
                classification_confidence: match &interpreted.semantics {
                    AttributeSemantics::Static { confidence } => *confidence,
                    AttributeSemantics::Instance { confidence } => *confidence,
                },
                signals_used: interpreted.inferred_type.signals.clone(),
            });

            attributes.push(attr);
        }

        let report = EntityReport {
            name: entity_name.to_string(),
            detected_as: "entity".to_string(),
            attributes: attr_reports,
            relationships: rel_reports,
        };

        Ok((attributes, report))
    }

    /// Transform functions to Rules.
    fn transform_functions(
        &self,
        merged: &YamlDocument,
        product_id: &ProductId,
        _attributes: &[AbstractAttribute],
    ) -> LoaderResult<(Vec<Rule>, Vec<FunctionReport>)> {
        let mut rules = Vec::new();
        let mut reports = Vec::new();

        if let Some(functions) = &merged.functions {
            for (name, func) in functions {
                let (rule, report) = self.transform_function(name, func, product_id)?;
                rules.push(rule);
                reports.push(report);
            }
        }

        Ok((rules, reports))
    }

    /// Transform a single function to Rule.
    fn transform_function(
        &self,
        name: &str,
        func: &YamlFunction,
        product_id: &ProductId,
    ) -> LoaderResult<(Rule, FunctionReport)> {
        // Determine evaluator type and preprocess LLM config if needed
        let evaluator = match func.evaluator.as_deref() {
            Some("llm") | Some("large-language-model") => {
                // Build function metadata for prompt preprocessing
                let metadata = self.build_function_metadata(func);

                // Preprocess LLM configuration (validates and optimizes prompt)
                let llm_config = LlmPromptPreprocessor::preprocess(func, name, &metadata)?;

                // Convert to evaluator config
                let config_map = LlmPromptPreprocessor::to_evaluator_config(&llm_config);

                // Convert serde_json::Value to product_farm_core::Value
                let core_config: HashMap<String, Value> = config_map
                    .into_iter()
                    .map(|(k, v)| (k, json_to_core_value(v)))
                    .collect();

                RuleEvaluator {
                    name: EvaluatorType::LargeLanguageModel,
                    config: Some(core_config),
                }
            }
            Some("farmscript") => RuleEvaluator::json_logic(), // FarmScript compiles to JSON Logic
            Some(custom) if custom != "json-logic" => RuleEvaluator::custom(custom),
            _ => RuleEvaluator::json_logic(),
        };

        // Get expression - handle FarmScript compilation if needed
        let (expression, display_expression) = self.compile_expression(func, name)?;

        // Build rule using builder
        let mut builder = RuleBuilder::new(product_id.to_string(), name.to_string());

        builder = builder.display(display_expression);
        builder = builder.expression(expression);

        // Add input attributes - these map to var references in JSON Logic
        if let Some(inputs) = &func.inputs {
            for input in inputs {
                builder = builder.input(input.clone());
            }
        }

        // Add output attributes - these are where results are stored
        if let Some(outputs) = &func.outputs {
            for output in outputs {
                builder = builder.output(output.clone());
            }
        } else {
            // Default output if none specified: use function name as output path
            builder = builder.output(format!("{}:output:{}", product_id, name));
        }

        if let Some(desc) = &func.description {
            builder = builder.description(desc.clone());
        }

        builder = builder.evaluator(evaluator.clone());
        builder = builder.enabled(func.enabled);

        let rule = builder.build()?;

        let report = FunctionReport {
            name: name.to_string(),
            evaluator_type: match evaluator.name {
                EvaluatorType::JsonLogic => "json-logic".to_string(),
                EvaluatorType::LargeLanguageModel => "llm".to_string(),
                EvaluatorType::Custom(ref s) => s.clone(),
            },
            input_count: func.inputs.as_ref().map(|v| v.len()).unwrap_or(0),
            output_count: func.outputs.as_ref().map(|v| v.len()).unwrap_or(0),
            has_expression: func.expression.is_some(),
        };

        Ok((rule, report))
    }

    /// Compile expression, handling FarmScript if needed.
    fn compile_expression(
        &self,
        func: &YamlFunction,
        name: &str,
    ) -> LoaderResult<(serde_json::Value, String)> {
        let is_farmscript = func.evaluator.as_deref() == Some("farmscript");

        match &func.expression {
            Some(expr) => {
                if is_farmscript {
                    // FarmScript: expression should be a string to compile
                    let source = match expr {
                        serde_json::Value::String(s) => s.clone(),
                        other => {
                            // If it's not a string, try to serialize it as a fallback
                            return Err(LoaderError::InvalidExpression(format!(
                                "Function '{}': FarmScript expression must be a string, got: {:?}",
                                name, other
                            )));
                        }
                    };

                    // Compile FarmScript to JSON Logic
                    let compiled = farmscript::compile(&source).map_err(|e| {
                        LoaderError::InvalidExpression(format!(
                            "Function '{}': FarmScript compilation failed: {}",
                            name, e
                        ))
                    })?;

                    let display = source; // Keep original FarmScript for display
                    Ok((compiled, display))
                } else {
                    // JSON Logic: use as-is
                    let display = serde_json::to_string(expr).unwrap_or_default();
                    Ok((expr.clone(), display))
                }
            }
            None => {
                // No expression - use empty object
                Ok((serde_json::json!({}), String::new()))
            }
        }
    }

    /// Transform functionalities.
    fn transform_functionalities(
        &self,
        merged: &YamlDocument,
        product_id: &ProductId,
    ) -> LoaderResult<Vec<ProductFunctionality>> {
        let mut functionalities = Vec::new();

        if let Some(funcs) = &merged.functionalities {
            for (name, func) in funcs {
                let now = chrono::Utc::now();
                let functionality = ProductFunctionality {
                    id: product_farm_core::FunctionalityId::new(&format!("{}:{}", product_id, name)),
                    product_id: product_id.clone(),
                    name: name.clone(),
                    immutable: false,
                    description: func.description.clone().unwrap_or_default(),
                    status: ProductFunctionalityStatus::Draft,
                    required_attributes: Vec::new(),
                    created_at: now,
                    updated_at: now,
                };
                functionalities.push(functionality);
            }
        }

        Ok(functionalities)
    }

    /// Extract layer configuration.
    fn extract_layer_config(&self, merged: &YamlDocument) -> LoaderResult<LayerVisibilityConfig> {
        let mut config = LayerVisibilityConfig::default();

        if let Some(layers) = &merged.layers {
            for (name, layer) in layers {
                let def = LayerDefinition {
                    name: layer.name.clone().unwrap_or_else(|| name.clone()),
                    description: layer.description.clone(),
                    visible_entities: layer
                        .entities
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .collect(),
                    visible_attributes: layer
                        .attributes
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .collect(),
                    visible_functions: layer
                        .functions
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .collect(),
                };
                config.layers.insert(name.clone(), def);
            }
        }

        Ok(config)
    }

    /// Validate critical requirements only.
    pub fn validate_critical(&self, schema: &MasterSchema) -> LoaderResult<()> {
        // At least one rule/function should exist for evaluation
        if schema.rules.is_empty() {
            return Err(LoaderError::NoFunctions);
        }

        Ok(())
    }

    /// Collect low-confidence items for reporting.
    fn collect_low_confidence_items(
        &self,
        entity_reports: &[EntityReport],
        _function_reports: &[FunctionReport],
    ) -> Vec<LowConfidenceItem> {
        let mut items = Vec::new();

        for entity in entity_reports {
            for attr in &entity.attributes {
                // Flag items with confidence below threshold
                if attr.classification_confidence < 0.5 {
                    items.push(LowConfidenceItem {
                        path: format!("{}.{}", entity.name, attr.name),
                        item_type: "attribute".to_string(),
                        confidence: attr.classification_confidence,
                        reason: format!(
                            "Low confidence in {} classification",
                            attr.classification
                        ),
                        suggestion: format!(
                            "Consider adding explicit 'static: true/false' marker to {}",
                            attr.name
                        ),
                    });
                }

                if matches!(attr.type_confidence, Confidence::Low | Confidence::Default) {
                    items.push(LowConfidenceItem {
                        path: format!("{}.{}", entity.name, attr.name),
                        item_type: "type".to_string(),
                        confidence: attr.type_confidence.weight(),
                        reason: format!("Type '{}' was inferred with low confidence", attr.inferred_type),
                        suggestion: format!(
                            "Consider adding explicit 'type: {}' to {} definition",
                            attr.inferred_type, attr.name
                        ),
                    });
                }
            }
        }

        items
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '_' {
            result.push('-');
        } else {
            result.push(c);
        }
    }
    result
}

fn field_def_to_yaml_value(field_def: &YamlFieldDefinition) -> serde_yaml::Value {
    match field_def {
        YamlFieldDefinition::Simple(s) => serde_yaml::Value::String(s.clone()),
        YamlFieldDefinition::Typed { field_type, description, default, required, min, max, pattern, values, computed, static_, instance } => {
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                serde_yaml::Value::String("type".into()),
                serde_yaml::Value::String(field_type.clone()),
            );
            if let Some(d) = description {
                map.insert(
                    serde_yaml::Value::String("description".into()),
                    serde_yaml::Value::String(d.clone()),
                );
            }
            if let Some(d) = default {
                if let Ok(yaml) = serde_yaml::to_value(d) {
                    map.insert(serde_yaml::Value::String("default".into()), yaml);
                }
            }
            if let Some(r) = required {
                map.insert(
                    serde_yaml::Value::String("required".into()),
                    serde_yaml::Value::Bool(*r),
                );
            }
            if let Some(m) = min {
                if let Ok(yaml) = serde_yaml::to_value(m) {
                    map.insert(serde_yaml::Value::String("min".into()), yaml);
                }
            }
            if let Some(m) = max {
                if let Ok(yaml) = serde_yaml::to_value(m) {
                    map.insert(serde_yaml::Value::String("max".into()), yaml);
                }
            }
            if let Some(p) = pattern {
                map.insert(
                    serde_yaml::Value::String("pattern".into()),
                    serde_yaml::Value::String(p.clone()),
                );
            }
            if let Some(v) = values {
                let seq: Vec<serde_yaml::Value> = v.iter().map(|s| serde_yaml::Value::String(s.clone())).collect();
                map.insert(
                    serde_yaml::Value::String("values".into()),
                    serde_yaml::Value::Sequence(seq),
                );
            }
            if let Some(c) = computed {
                map.insert(
                    serde_yaml::Value::String("computed".into()),
                    serde_yaml::Value::String(c.clone()),
                );
            }
            if let Some(s) = static_ {
                map.insert(
                    serde_yaml::Value::String("static".into()),
                    serde_yaml::Value::Bool(*s),
                );
            }
            if let Some(i) = instance {
                map.insert(
                    serde_yaml::Value::String("instance".into()),
                    serde_yaml::Value::Bool(*i),
                );
            }
            serde_yaml::Value::Mapping(map)
        }
        YamlFieldDefinition::Array(arr) => {
            serde_yaml::Value::Sequence(arr.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())
        }
        YamlFieldDefinition::Nested(map) => {
            serde_yaml::Value::Mapping(
                map.iter()
                    .map(|(k, v)| (serde_yaml::Value::String(k.clone()), v.clone()))
                    .collect(),
            )
        }
    }
}

/// Convert serde_json::Value to product_farm_core::Value.
fn json_to_core_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_core_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, json_to_core_value(v))).collect())
        }
    }
}

impl SchemaTransformer {
    /// Build function metadata for LLM prompt preprocessing.
    fn build_function_metadata(&self, func: &YamlFunction) -> FunctionMetadata {
        let inputs = func.inputs.clone().unwrap_or_default();

        let outputs = func
            .outputs
            .as_ref()
            .map(|outs| {
                outs.iter()
                    .map(|name| {
                        // Try to extract type info from evaluator_config if available
                        let (data_type, enum_values) = func
                            .evaluator_config
                            .as_ref()
                            .and_then(|c| c.get("output_types"))
                            .and_then(|v| v.as_object())
                            .and_then(|types| types.get(name))
                            .map(|type_def| {
                                let dtype = type_def
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let enums = type_def
                                    .get("values")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    });
                                (dtype, enums)
                            })
                            .unwrap_or((None, None));

                        OutputMetadata {
                            name: name.clone(),
                            data_type,
                            enum_values,
                            constraints: None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        FunctionMetadata {
            inputs,
            outputs,
            description: func.description.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("UserProfile"), "user-profile");
        assert_eq!(to_kebab_case("user_name"), "user-name");
        assert_eq!(to_kebab_case("simple"), "simple");
    }
}
