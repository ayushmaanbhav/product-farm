package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType

data class GetFunctionalityAttributeResponse(
    val displayName: String,
    val path: String,
    val value: JsonNode?,
    val rule: GetExecutableRuleResponse?,
    val type: AttributeValueType,
    val constraintExpression: GetExecutableRuleResponse?,
    val datatype: DatatypeDto,
    val enumeration: ProductTemplateEnumerationDto?,
    val tags: List<String>,
)
