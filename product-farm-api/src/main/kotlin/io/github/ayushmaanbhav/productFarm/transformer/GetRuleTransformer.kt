package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import org.springframework.stereotype.Component

@Component
class GetRuleTransformer(
    private val ruleTransformer: RuleTransformer,
) : Transformer<Rule, io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse>() {
    
    override fun forward(input: Rule): io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse {
        val rule = ruleTransformer.forward(input)
        return io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse(
                type = rule.type,
                inputAttributes = rule.inputAttributes,
                outputAttributes = rule.outputAttributes,
                displayExpression = rule.displayExpression,
                description = rule.description,
        )
    }
    
    override fun reverse(input: io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse) = throw ProductFarmServiceException("Operation not supported")
}
