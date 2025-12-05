package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityStatusResponse
import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import org.springframework.stereotype.Component

@Component
class GetProductFunctionalityStatusTransformer : OneWayTransformer<ProductFunctionality, ProductFunctionalityStatusResponse> {
    override fun forward(input: ProductFunctionality) = ProductFunctionalityStatusResponse(status = input.status)
}
