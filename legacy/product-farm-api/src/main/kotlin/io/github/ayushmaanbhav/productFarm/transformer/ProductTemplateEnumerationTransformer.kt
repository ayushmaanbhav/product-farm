package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import io.github.ayushmaanbhav.productFarm.entity.ProductTemplateEnumeration
import io.github.ayushmaanbhav.productFarm.util.generateUUID
import org.springframework.stereotype.Component

@Component
class ProductTemplateEnumerationTransformer :
    Transformer<ProductTemplateEnumeration, Pair<ProductTemplateEnumerationDto, ProductTemplateType>> {
    
    override fun forward(input: ProductTemplateEnumeration) =
        Pair(
            ProductTemplateEnumerationDto(
                name = input.name,
                values = input.values,
                description = input.description,
            ),
            input.productTemplateType
        )
    
    override fun reverse(input: Pair<ProductTemplateEnumerationDto, ProductTemplateType>): ProductTemplateEnumeration {
        return ProductTemplateEnumeration(
            id = generateUUID(),
            name = input.first.name,
            values = input.first.values,
            description = input.first.description,
            productTemplateType = input.second
        )
    }
}
