package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetExecutableRuleResponse
import io.github.ayushmaanbhav.productFarm.entity.Rule
import org.springframework.stereotype.Component

@Component
class GetExecutableRuleTransformer(
    private val ruleTransformer: RuleTransformer,
) : OneWayTransformer<Rule, GetExecutableRuleResponse> {
    
    override fun forward(input: Rule): GetExecutableRuleResponse {
        val rule = ruleTransformer.forward(input)
        return GetExecutableRuleResponse(
            type = rule.type,
            inputAttributes = rule.inputAttributes,
            outputAttributes = rule.outputAttributes,
            compiledExpression = rule.compiledExpression,
        )
    }
}
