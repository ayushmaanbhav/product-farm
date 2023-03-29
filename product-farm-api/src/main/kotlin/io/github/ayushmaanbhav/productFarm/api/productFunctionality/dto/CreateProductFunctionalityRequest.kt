package io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto

data class CreateProductFunctionalityRequest(
    val name: String,
    val immutable: Boolean,
    val description: String,
    val requiredAttributes: LinkedHashSet<FunctionalityRequiredAttributeDto>,
)
