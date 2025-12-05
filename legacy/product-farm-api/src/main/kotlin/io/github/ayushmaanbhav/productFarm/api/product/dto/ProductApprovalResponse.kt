package io.github.ayushmaanbhav.productFarm.api.product.dto

import io.github.ayushmaanbhav.productFarm.constant.ProductStatus
import java.time.LocalDateTime

data class ProductApprovalResponse(
    val status: ProductStatus,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    val approvedBy: String,
    val discontinuedProductId: String?,
    val changeDescription: String,
)
