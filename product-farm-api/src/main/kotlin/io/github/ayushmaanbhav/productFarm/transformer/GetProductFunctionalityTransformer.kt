package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.FunctionalityRequiredAttributeDto
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.GetProductFunctionalityResponse
import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import org.springframework.stereotype.Component

@Component
class GetProductFunctionalityTransformer : OneWayTransformer<ProductFunctionality, GetProductFunctionalityResponse> {

    override fun forward(input: ProductFunctionality) =
        GetProductFunctionalityResponse(
            name = input.name,
            immutable = input.immutable,
            description = input.description,
            status = input.status,
            requiredAttributes = input.requiredAttributes.map {
                FunctionalityRequiredAttributeDto(abstractPath = it.id.abstractPath, description = it.description)
            }.toCollection(LinkedHashSet())
        )
}
