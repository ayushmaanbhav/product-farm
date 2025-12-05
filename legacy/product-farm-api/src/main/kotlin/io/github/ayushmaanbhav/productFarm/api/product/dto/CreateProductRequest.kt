package io.github.ayushmaanbhav.productFarm.api.product.dto

import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import java.time.LocalDateTime

data class CreateProductRequest(
    val id: String,
    val name: String,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    val templateType: ProductTemplateType,
    val description: String?,
)
