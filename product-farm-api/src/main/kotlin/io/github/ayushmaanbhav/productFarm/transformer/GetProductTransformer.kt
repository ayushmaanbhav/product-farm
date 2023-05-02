package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.product.dto.GetProductResponse
import io.github.ayushmaanbhav.productFarm.entity.Product
import org.springframework.stereotype.Component

@Component
class GetProductTransformer : OneWayTransformer<Product, GetProductResponse> {
    
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
}
