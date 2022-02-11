package com.navi.insurance.productFarm.dto

data class ProductTemplateEnumerationDto(
    val name: String,
    val value: Set<String>,
    val description: String?,
)
