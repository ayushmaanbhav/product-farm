package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType

data class GetAttributeResponse(
    val displayName: String,
    val type: AttributeValueType,
    val value: JsonNode?,
    val rule: GetRuleResponse?,
    val datatype: DatatypeDto,
    val enumeration: ProductTemplateEnumerationDto?,
    val relatedAttributes: LinkedHashSet<String>?,
    val constraintExpression: GetRuleResponse?,
    val immutable: Boolean,
    val description: String?,
)
