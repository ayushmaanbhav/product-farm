package com.navi.insurance.productFarm.dto

import com.navi.insurance.productFarm.constant.ProductStatus
import com.navi.insurance.productFarm.constant.ProductTemplateType
import java.time.LocalDateTime

data class ProductDto(
    val id: String,
    val name: String,
    val status: ProductStatus,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    val templateType: ProductTemplateType,
    val description: String?,
)
