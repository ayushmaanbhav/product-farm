package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeByTagResponse
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat.HUMAN
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import org.springframework.stereotype.Component

@Component
class GetAttributeByTagTransformer(
    private val getRuleTransformer: GetRuleTransformer,
    val datatypeTransformer: DatatypeTransformer,
    val enumerationTransformer: ProductTemplateEnumerationTransformer,
) : OneWayTransformer<Attribute, GetAttributeByTagResponse> {
    
    override fun forward(input: Attribute) =
        GetAttributeByTagResponse(
            displayName = input.displayNames.find { it.displayNameFormat == HUMAN }!!.id.displayName,
            value = input.value,
            rule = input.rule?.let(getRuleTransformer::forward),
            datatype = input.abstractAttribute.datatype.let { datatypeTransformer.forward(it) },
            enumeration = input.abstractAttribute.enumeration?.let { enumerationTransformer.forward(it).first },
            relatedAttributes = input.abstractAttribute.relatedAttributes.map { it.id.abstractPath }.toCollection(LinkedHashSet()),
            constraintExpression = input.abstractAttribute.constraintRule?.let { getRuleTransformer.forward(it) },
            immutable = input.abstractAttribute.immutable,
            description = input.abstractAttribute.description,
            type = input.type,
        )
}
