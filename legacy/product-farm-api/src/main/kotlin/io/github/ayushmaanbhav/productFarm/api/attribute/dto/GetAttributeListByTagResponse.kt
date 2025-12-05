package io.github.ayushmaanbhav.productFarm.api.attribute.dto

data class GetAttributeListByTagResponse(
    val attributes: LinkedHashSet<GetAttributeByTagResponse>,
)
