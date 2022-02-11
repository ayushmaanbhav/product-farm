package com.navi.insurance.productFarm.dto

import com.navi.insurance.productFarm.constant.DatatypeType

data class DatatypeDto(
    val name: String,
    val type: DatatypeType,
    val description: String?,
)
