package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat.HUMAN
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class GetAttributeTransformer(
    private val getRuleTransformer: GetRuleTransformer,
) : Transformer<Attribute, GetAttributeResponse>() {
    
    override fun forward(input: Attribute) =
        GetAttributeResponse(
            displayName = input.displayNames.find { it.displayNameFormat == HUMAN }!!.id.displayName,
            value = input.value,
            rule = input.rule?.let(getRuleTransformer::forward),
        )
    
    override fun reverse(input: GetAttributeResponse) =
        throw ProductFarmServiceException("Operation not supported")
}
