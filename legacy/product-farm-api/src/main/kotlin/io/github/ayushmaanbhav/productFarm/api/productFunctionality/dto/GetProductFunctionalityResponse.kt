package io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto

import io.github.ayushmaanbhav.productFarm.constant.ProductFunctionalityStatus

data class GetProductFunctionalityResponse(
    val name: String,
    val immutable: Boolean,
    val description: String,
    val status: ProductFunctionalityStatus,
    val requiredAttributes: LinkedHashSet<FunctionalityRequiredAttributeDto>,
)
