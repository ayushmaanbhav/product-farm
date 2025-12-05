package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductApprovalRepo
import io.github.ayushmaanbhav.productFarm.transformer.CreateProductApprovalTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import jakarta.transaction.Transactional
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component

@Component
class ProductApprovalService(
    private val createProductApprovalTransformer: CreateProductApprovalTransformer,
    private val productApprovalRepo: ProductApprovalRepo,
) {
    @Transactional
    fun create(productId: String, request: ProductApprovalRequest) {
        if (productApprovalRepo.existsById(productId)) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product approval already exists for this id"))
            )
        }
        productApprovalRepo.save(createProductApprovalTransformer.forward(Pair(productId, request)))
    }
}
