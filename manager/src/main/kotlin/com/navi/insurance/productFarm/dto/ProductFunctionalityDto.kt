package com.navi.insurance.productFarm.dto

data class ProductFunctionalityDto(
    val name: String,
    val immutable: Boolean,
    val description: String,
    val requiredAttribute: Set<FunctionalityRequiredAttributeDto>
)
