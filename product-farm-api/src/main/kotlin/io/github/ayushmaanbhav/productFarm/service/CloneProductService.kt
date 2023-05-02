package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.productFarm.api.product.dto.CloneProductRequest
import jakarta.transaction.Transactional
import org.springframework.stereotype.Component

@Component
class CloneProductService(
    private val productService: ProductService,
    private val abstractAttributeService: AbstractAttributeService,
    private val attributeService: AttributeService,
    private val productFunctionalityService: ProductFunctionalityService,
) {
    @Transactional
    fun clone(parentProductId: String, request: CloneProductRequest) {
        productService.clone(parentProductId, request)
        abstractAttributeService.clone(parentProductId, request.productId)
        attributeService.clone(parentProductId, request.productId)
        productFunctionalityService.clone(parentProductId, request.productId)
    }
}
