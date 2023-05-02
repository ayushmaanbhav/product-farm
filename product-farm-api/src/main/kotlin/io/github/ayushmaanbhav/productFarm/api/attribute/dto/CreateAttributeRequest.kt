package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import com.fasterxml.jackson.databind.JsonNode

data class CreateAttributeRequest(
    val displayName: String,
    val value: JsonNode?,
    val rule: CreateRuleRequest?,
)
