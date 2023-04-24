package io.github.ayushmaanbhav.productFarm.model

import io.github.ayushmaanbhav.productFarm.util.generateUUID
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule

data class Rule(
    val type: String,
    val inputAttributes: LinkedHashSet<String>,
    val outputAttributes: LinkedHashSet<String>,
    val displayExpression: RuleDisplayExpression,
    val displayExpressionVersion: String,
    val compiledExpression: String,
    val description: String?,
) : Rule {
    override fun getId() = generateUUID()
    override fun ruleType() = type
    override fun getInputAttributePaths() = inputAttributes
    override fun getOutputAttributePaths() = outputAttributes
    override fun getTags() = HashSet<String>()
    override fun getExpression() = compiledExpression
}
