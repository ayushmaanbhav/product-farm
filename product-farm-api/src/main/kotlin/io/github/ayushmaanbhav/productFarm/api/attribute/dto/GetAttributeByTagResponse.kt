package io.github.ayushmaanbhav.productFarm.api.attribute.dto

import io.github.ayushmaanbhav.productFarm.entity.Datatype
import io.github.ayushmaanbhav.productFarm.entity.ProductTemplateEnumeration

data class GetAttributeByTagResponse(
    val displayName: String,
    val datatype: Datatype,
    val enumeration: ProductTemplateEnumeration?,
    val relatedAttributes: LinkedHashSet<String>?,
    val constraintExpression: GetRuleResponse?,
    val immutable: Boolean,
    val description: String?,
)
