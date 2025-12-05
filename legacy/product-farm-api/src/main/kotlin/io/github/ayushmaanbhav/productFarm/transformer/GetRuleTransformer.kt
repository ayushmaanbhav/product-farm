package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetRuleResponse
import io.github.ayushmaanbhav.productFarm.entity.Rule
import org.springframework.stereotype.Component

@Component
class GetRuleTransformer(
    private val ruleTransformer: RuleTransformer,
) : OneWayTransformer<Rule, GetRuleResponse> {
    
    override fun forward(input: Rule): GetRuleResponse {
        val rule = ruleTransformer.forward(input)
        return GetRuleResponse(
            type = rule.type,
            inputAttributes = rule.inputAttributes,
            outputAttributes = rule.outputAttributes,
            displayExpression = rule.displayExpression,
            description = rule.description,
        )
    }
}
