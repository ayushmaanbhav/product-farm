package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.relationship.AttributeDisplayName
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.util.dissectAttributeDisplayName
import io.github.ayushmaanbhav.productFarm.util.generateDisplayNames
import io.github.ayushmaanbhav.productFarm.util.generatePath
import org.springframework.stereotype.Component

@Component
class CreateAttributeTransformer(
    private val createRuleTransformer: CreateRuleTransformer,
    private val abstractAttributeRepo: AbstractAttributeRepo,
) : Transformer<Pair<String, CreateAttributeRequest>, Attribute>() {
    
    override fun forward(input: Pair<String, CreateAttributeRequest>): Attribute {
        val productId = input.first
        val request = input.second
        val dissectedId = dissectAttributeDisplayName(request.displayName)!!
        val abstractAttribute = listOf(
            generatePath(productId, dissectedId.componentType, dissectedId.componentId, dissectedId.name),
            generatePath(productId, dissectedId.componentType, null, dissectedId.name)
        ).map(abstractAttributeRepo::getReferenceById).first()
        val path = generatePath(productId, dissectedId.componentType, dissectedId.componentId, dissectedId.name)
        val displayNames = generateDisplayNames(productId, dissectedId.componentType, dissectedId.componentId, dissectedId.name)
        return Attribute(
            path = path,
            displayNames = displayNames.mapIndexed { index, displayNamePair ->
                AttributeDisplayName(
                    id = AttributeDisplayNameId(
                        productId = productId,
                        displayName = displayNamePair.second,
                    ),
                    abstractPath = abstractAttribute.abstractPath,
                    path = path,
                    displayNameFormat = displayNamePair.first,
                    order = index
                )
            },
            abstractAttribute = abstractAttribute,
            type = request.value?.let { AttributeValueType.STATIC } ?: AttributeValueType.DYNAMIC,
            value = request.value,
            rule = request.rule?.let { createRuleTransformer.forward(it) },
            productId = productId
        )
    }
    
    override fun reverse(input: Attribute) = throw ProductFarmServiceException("Operation not supported")
}
