package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.product.dto.CreateProductRequest
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus
import io.github.ayushmaanbhav.productFarm.entity.Product
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class CreateProductTransformer : Transformer<CreateProductRequest, Product>() {
    
    override fun forward(input: CreateProductRequest) =
        Product(
            id = input.id,
            name = input.name,
            status = ProductStatus.DRAFT,
            effectiveFrom = input.effectiveFrom,
            expiryAt = input.expiryAt,
            templateType = input.templateType,
            parentProductId = null,
            description = input.description,
        )
    
    override fun reverse(input: Product) = throw ProductFarmServiceException("Operation not supported")
}
