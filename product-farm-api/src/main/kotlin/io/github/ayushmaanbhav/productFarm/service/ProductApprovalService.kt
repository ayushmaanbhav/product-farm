package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductApprovalRepo
import io.github.ayushmaanbhav.productFarm.transformer.ProductApprovalTransformer
import io.github.ayushmaanbhav.productFarm.validation.createError
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import jakarta.transaction.Transactional

@Component
class ProductApprovalService(
    private val productApprovalTransformer: ProductApprovalTransformer,
    private val productApprovalRepo: ProductApprovalRepo,
) {
    @Transactional
    fun create(productId: String, request: ProductApprovalRequest) {
        if (productApprovalRepo.existsById(productId)) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product approval already exists for this id"))
            )
        }
        productApprovalRepo.save(productApprovalTransformer.forward(Pair(productId, request)))
    }
}
