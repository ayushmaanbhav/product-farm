package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeResponse
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import org.springframework.stereotype.Component

@Component
class GetFunctionalityAttributeTransformer(
    val ruleTransformer: GetExecutableRuleTransformer,
    val datatypeTransformer: DatatypeTransformer,
    val enumerationTransformer: ProductTemplateEnumerationTransformer,
) : OneWayTransformer<List<Attribute>, GetFunctionalityAttributeListResponse> {
    override fun forward(input: List<Attribute>) = GetFunctionalityAttributeListResponse(
        attributes = input.map { it ->
            GetFunctionalityAttributeResponse(
                path = it.path,
                type = it.type,
                tags = it.abstractAttribute.tags.sortedBy { it.order }.map { it.id.tag },
                displayName = it.displayNames.find { it.displayNameFormat == DisplayNameFormat.HUMAN }!!.id.displayName,
                value = it.value,
                rule = it.rule?.let(ruleTransformer::forward),
                datatype = it.abstractAttribute.datatype.let { datatypeTransformer.forward(it) },
                enumeration = it.abstractAttribute.enumeration?.let { enumerationTransformer.forward(it).first },
                constraintExpression = it.abstractAttribute.constraintRule?.let { ruleTransformer.forward(it) },
            )
        }.toCollection(LinkedHashSet())
    )
}
