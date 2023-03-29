package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateRuleRequest
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.rule.domain.ruleExpression.ExpressionParser
import org.springframework.stereotype.Component

@Component
class CreateRuleTransformer(
    private val ruleTransformer: RuleTransformer,
) : Transformer<io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateRuleRequest, Rule>() {
    
    override fun forward(input: io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateRuleRequest): Rule {
        val rule = io.github.ayushmaanbhav.productFarm.model.Rule(
            type = input.type,
            inputAttributes = input.inputAttributes,
            outputAttributes = input.outputAttributes,
            displayExpression = input.displayExpression,
            displayExpressionVersion = ExpressionParser::class.java.`package`.implementationVersion,
            compiledExpression = "",
            description = input.description
        )
        return ruleTransformer.reverse(rule)
    }
    
    override fun reverse(input: Rule) = throw ProductFarmServiceException("Operation not supported")
}
