package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.product.dto.GetProductResponse
import io.github.ayushmaanbhav.productFarm.entity.Product
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class GetProductTransformer : Transformer<Product, GetProductResponse>() {
    
    override fun forward(input: Product) =
        GetProductResponse(
            id = input.id,
            name = input.name,
            status = input.status,
            effectiveFrom = input.effectiveFrom,
            expiryAt = input.expiryAt,
            templateType = input.templateType,
            description = input.description,
        )
    
    override fun reverse(input: GetProductResponse) = throw ProductFarmServiceException("Operation not supported")
}
