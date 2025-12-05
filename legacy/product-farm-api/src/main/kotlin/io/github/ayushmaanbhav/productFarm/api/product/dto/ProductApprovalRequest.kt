package io.github.ayushmaanbhav.productFarm.api.product.dto

data class ProductApprovalRequest(
    val approvedBy: String,
    val discontinuedProductId: String?,
    val changeDescription: String,
)
