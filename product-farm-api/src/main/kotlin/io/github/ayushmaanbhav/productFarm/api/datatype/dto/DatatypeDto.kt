package io.github.ayushmaanbhav.productFarm.api.datatype.dto

import io.github.ayushmaanbhav.productFarm.constant.DatatypeType

data class DatatypeDto(
    val name: String,
    val type: DatatypeType,
    val description: String?,
)
