package com.navi.insurance.productFarm.dto

import java.time.LocalDateTime

data class ProductApprovalDto(
    val productId: String,
    val approvedBy: String,
    val discontinuedProductId: String?,
    val changeDescription: LocalDateTime,
)
