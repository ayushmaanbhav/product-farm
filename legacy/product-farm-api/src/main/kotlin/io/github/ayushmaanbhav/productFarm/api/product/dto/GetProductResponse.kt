package io.github.ayushmaanbhav.productFarm.api.product.dto

import io.github.ayushmaanbhav.productFarm.constant.ProductStatus
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import java.time.LocalDateTime

data class GetProductResponse(
    val id: String,
    val name: String,
    val status: ProductStatus,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    val templateType: ProductTemplateType,
    val description: String?,
)
