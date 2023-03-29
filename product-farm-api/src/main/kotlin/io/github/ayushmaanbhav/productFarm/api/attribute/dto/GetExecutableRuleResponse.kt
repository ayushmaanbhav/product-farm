package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetExecutableRuleResponse(
    val type: String,
    val inputAttributes: LinkedHashSet<String>,
    val outputAttributes: LinkedHashSet<String>,
    val compiledExpression: String,
)
