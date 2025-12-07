//! Rule conversions between proto and core types

use product_farm_core::{Rule as CoreRule, RuleId, RuleInputAttribute, RuleOutputAttribute};

use crate::grpc::proto;

pub fn core_to_proto_rule(r: &CoreRule) -> proto::Rule {
    proto::Rule {
        id: r.id.to_string(),
        product_id: r.product_id.as_str().to_string(),
        rule_type: r.rule_type.clone(),
        input_attributes: r
            .input_attributes
            .iter()
            .map(|a| proto::RuleAttribute {
                path: a.path.as_str().to_string(),
                order: a.order,
            })
            .collect(),
        output_attributes: r
            .output_attributes
            .iter()
            .map(|a| proto::RuleAttribute {
                path: a.path.as_str().to_string(),
                order: a.order,
            })
            .collect(),
        display_expression: r.display_expression.clone(),
        display_expression_version: r.display_expression_version.clone(),
        expression_json: r.compiled_expression.clone(),
        description: r.description.clone(),
        enabled: r.enabled,
        order_index: r.order_index,
    }
}

pub fn proto_to_core_rule(r: &proto::Rule) -> Result<CoreRule, String> {
    // Validate JSON
    let _: serde_json::Value = serde_json::from_str(&r.expression_json)
        .map_err(|e| format!("Invalid expression JSON: {}", e))?;

    let rule_id = if r.id.is_empty() {
        RuleId::new()
    } else {
        RuleId::from_string(&r.id)
    };

    let mut rule = CoreRule::new(
        r.product_id.as_str(),
        r.rule_type.as_str(),
        r.expression_json.as_str(),
    )
    .with_id(rule_id.clone())
    .with_display(r.display_expression.as_str())
    .with_display_version(r.display_expression_version.as_str())
    .with_order(r.order_index)
    .with_enabled(r.enabled);

    if let Some(desc) = &r.description {
        rule = rule.with_description(desc.as_str());
    }

    // Set inputs
    rule.input_attributes = r
        .input_attributes
        .iter()
        .map(|a| RuleInputAttribute::new(rule_id.clone(), a.path.as_str(), a.order))
        .collect();

    // Set outputs
    rule.output_attributes = r
        .output_attributes
        .iter()
        .map(|a| RuleOutputAttribute::new(rule_id.clone(), a.path.as_str(), a.order))
        .collect();

    Ok(rule)
}
