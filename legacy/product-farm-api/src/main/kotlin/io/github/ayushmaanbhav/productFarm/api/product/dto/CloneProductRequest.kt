package io.github.ayushmaanbhav.productFarm.api.product.dto

import java.time.LocalDateTime

data class CloneProductRequest(
    val productId: String,
    val name: String,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    val description: String?,
)
