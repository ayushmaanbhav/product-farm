package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetAbstractAttributeResponse(
        val abstractPath: String,
        val displayName: String,
        val componentType: String,
        val componentId: String?,
        val tags: LinkedHashSet<String>,
        val datatype: String,
        val enumeration: String?,
        val relatedAttributes: LinkedHashSet<String>?,
        val constraintExpression: GetRuleResponse?,
        val immutable: Boolean,
        val description: String?,
)
