package com.navi.insurance.productFarm.dto

import com.navi.insurance.productFarm.entity.Datatype
import com.navi.insurance.productFarm.entity.ProductTemplateEnumeration

data class AbstractAttributeDto(
    val abstractPath: String,
    val displayName: String,
    val componentType: String,
    val componentId: String?,
    val tag: Set<String>,
    val datatype: Datatype,
    val enumeration: ProductTemplateEnumeration?,
    val referencesAttribute: Set<String>?,
    val constraintExpression: String?,
    val immutable: Boolean,
    val description: String?,
)
