package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.productFarm.entity.Datatype
import io.github.ayushmaanbhav.productFarm.entity.ProductTemplateEnumeration

data class GetAttributeResponse(
    val displayName: String,
    val value: JsonNode?,
    val rule: GetRuleResponse?,
    val datatype: Datatype,
    val enumeration: ProductTemplateEnumeration?,
    val relatedAttributes: LinkedHashSet<String>?,
    val constraintExpression: GetRuleResponse?,
    val immutable: Boolean,
    val description: String?,
)
