package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAbstractAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat.HUMAN
import io.github.ayushmaanbhav.productFarm.entity.AbstractAttribute
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class GetAttributeTransformer(
    private val getRuleTransformer: GetRuleTransformer,
) : Transformer<Attribute, io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse>() {
    
    override fun forward(input: Attribute) =
            io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse(
                    abstractPath = input.abstractPath,
                    displayName = input.displayNames.find { it.displayNameFormat == HUMAN }!!.id.displayName,
                    componentType = input.componentType,
                    componentId = input.componentId,
                    tags = input.tags.map { it.id.tag }.toCollection(LinkedHashSet()),
                    datatype = input.datatype.name,
                    enumeration = input.enumeration?.name,
                    relatedAttributes = input.relatedAttributes
                            .map { it.id.referenceAbstractPath }.toCollection(LinkedHashSet()),
                    constraintExpression = input.constraintRule?.let { getRuleTransformer.forward(it) },
                    immutable = input.immutable,
                    description = input.description,
            )
    
    override fun reverse(input: io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAbstractAttributeResponse) =
        throw ProductFarmServiceException("Operation not supported")
}
