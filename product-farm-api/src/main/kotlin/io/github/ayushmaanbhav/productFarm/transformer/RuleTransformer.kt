package io.github.ayushmaanbhav.productFarm.transformer

import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleInputAttributeId
import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleOutputAttributeId
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleInputAttribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleOutputAttribute
import io.github.ayushmaanbhav.productFarm.model.RuleDisplayExpression
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.util.generateUUID
import org.springframework.stereotype.Component

@Component
class RuleTransformer(
    private val objectMapper: ObjectMapper,
    private val ruleUtil: RuleUtil,
) : Transformer<Rule, io.github.ayushmaanbhav.productFarm.model.Rule>() {
    
    override fun forward(input: Rule) =
        io.github.ayushmaanbhav.productFarm.model.Rule(
            type = input.type,
            inputAttributes = input.inputAttributes.sortedBy { it.order }.map { it.id.path }
                .toCollection(LinkedHashSet()),
            outputAttributes = input.outputAttributes.sortedBy { it.order }.map { it.id.path }
                .toCollection(LinkedHashSet()),
            displayExpression = objectMapper.readValue(
                input.displayExpression, RuleDisplayExpression::class.java
            ),
            displayExpressionVersion = input.displayExpressionVersion,
            compiledExpression = input.compiledExpression,
            description = input.description,
        )
    
    override fun reverse(input: io.github.ayushmaanbhav.productFarm.model.Rule): Rule {
        val ruleId = generateUUID()
        return Rule(
            id = generateUUID(),
            type = input.type,
            inputAttributes = input.inputAttributes.mapIndexed { index, it ->
                RuleInputAttribute(RuleInputAttributeId(ruleId = ruleId, path = it), index)
            },
            outputAttributes = input.outputAttributes.mapIndexed { index, it ->
                RuleOutputAttribute(RuleOutputAttributeId(ruleId = ruleId, path = it), index)
            },
            displayExpression = objectMapper.writeValueAsString(input.displayExpression),
            displayExpressionVersion = input.displayExpressionVersion,
            compiledExpression = ruleUtil.compileExpression(input),
            description = input.description,
        )
    }
}
