package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType

data class CreateAttributeRequest(
    val displayName: String,
    val value: JsonNode?,
    val rule: CreateRuleRequest?,
    val type: AttributeValueType,
)
