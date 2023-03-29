package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetFunctionalityAttributeResponse(
        val displayName: String,
        val path: String,
        val value: String?,
        val rule: io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetExecutableRuleResponse?,
        val constraintExpression: io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetExecutableRuleResponse?,
)
