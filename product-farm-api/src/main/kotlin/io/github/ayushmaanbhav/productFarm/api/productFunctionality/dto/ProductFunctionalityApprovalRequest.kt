package io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto

data class ProductFunctionalityApprovalRequest(
    val approvedBy: String,
    val changeDescription: String,
)
