package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import io.github.ayushmaanbhav.productFarm.model.RuleDisplayExpression

data class GetRuleResponse(
    val type: String,
    val inputAttributes: LinkedHashSet<String>,
    val outputAttributes: LinkedHashSet<String>,
    val displayExpression: RuleDisplayExpression,
    val description: String?,
)
