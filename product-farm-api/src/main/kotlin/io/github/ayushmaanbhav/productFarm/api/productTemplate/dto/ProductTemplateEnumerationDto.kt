package io.github.ayushmaanbhav.productFarm.api.productTemplate.dto

data class ProductTemplateEnumerationDto(
    val name: String,
    val values: LinkedHashSet<String>,
    val description: String?,
)
