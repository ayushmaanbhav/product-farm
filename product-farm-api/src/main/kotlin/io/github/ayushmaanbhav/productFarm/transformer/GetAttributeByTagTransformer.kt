package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeByTagResponse
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat.HUMAN
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class GetAttributeByTagTransformer(
    private val getRuleTransformer: GetRuleTransformer,
) : Transformer<Attribute, GetAttributeByTagResponse>() {
    
    override fun forward(input: Attribute) =
        GetAttributeByTagResponse(
            displayName = input.displayNames.find { it.displayNameFormat == HUMAN }!!.id.displayName,
            value = input.value,
            rule = input.rule?.let(getRuleTransformer::forward),
            datatype = input.abstractAttribute.datatype,
            enumeration = input.abstractAttribute.enumeration,
            relatedAttributes = input.abstractAttribute.relatedAttributes.map { it.id.abstractPath }.toCollection(LinkedHashSet()),
            constraintExpression = input.abstractAttribute.constraintRule?.let { getRuleTransformer.forward(it) },
            immutable = input.abstractAttribute.immutable,
            description = input.abstractAttribute.description,
        )
    
    override fun reverse(input: GetAttributeByTagResponse) =
        throw ProductFarmServiceException("Operation not supported")
}
