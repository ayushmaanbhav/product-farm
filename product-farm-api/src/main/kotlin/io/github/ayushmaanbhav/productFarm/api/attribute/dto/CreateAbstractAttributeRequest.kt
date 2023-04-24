package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class CreateAbstractAttributeRequest(
    val name: String,
    val componentType: String,
    val componentId: String?,
    val tags: LinkedHashSet<String>,
    val datatype: String,
    val enumeration: String?,
    val relatedAttributes: LinkedHashSet<String>?,
    val constraintExpression: CreateRuleRequest?,
    val immutable: Boolean,
    val description: String?,
)
