package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetAttributeResponse(
        val displayName: String,
        val value: String?,
        val rule: GetRuleResponse?,
)
