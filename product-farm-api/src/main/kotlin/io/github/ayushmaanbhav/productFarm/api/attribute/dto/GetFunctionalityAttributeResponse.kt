package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetFunctionalityAttributeResponse(
    val displayName: String,
    val path: String,
    val value: String?,
    val rule: GetExecutableRuleResponse?,
    val constraintExpression: GetExecutableRuleResponse?,
)
