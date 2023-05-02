package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.CreateProductFunctionalityRequest
import io.github.ayushmaanbhav.productFarm.constant.ProductFunctionalityStatus
import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import io.github.ayushmaanbhav.productFarm.entity.compositeId.FunctionalityRequiredAttributeId
import io.github.ayushmaanbhav.productFarm.entity.relationship.FunctionalityRequiredAttribute
import java.util.*
import org.springframework.stereotype.Component

@Component
class CreateProductFunctionalityTransformer : OneWayTransformer<Pair<String, CreateProductFunctionalityRequest>, ProductFunctionality>  {
    override fun forward(input: Pair<String, CreateProductFunctionalityRequest>): ProductFunctionality {
        val id = UUID.randomUUID().toString()
        return ProductFunctionality(
            id = id,
            name = input.second.name,
            productId = input.first,
            immutable = input.second.immutable,
            description = input.second.description,
            requiredAttributes = input.second.requiredAttributes.mapIndexed { index, it ->
                FunctionalityRequiredAttribute(
                    id = FunctionalityRequiredAttributeId(functionalityId = id, abstractPath = it.abstractPath),
                    description = it.description,
                    order = index
                )
            },
            status = ProductFunctionalityStatus.DRAFT
        )
    }
}
