package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class CreateAttributeRequest(
    val displayName: String,
    val value: String?,
    val rule: CreateRuleRequest?,
)
