package io.github.ayushmaanbhav.productFarm.util

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.rule.domain.ruleEngine.api.RuleEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.AcyclicDirectedGraph
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.DependencyGraphBuilder
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryType
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule
import java.util.UUID
import org.springframework.http.HttpStatus.BAD_REQUEST
import org.springframework.stereotype.Component

@Component
class RuleUtil(
    val productRuleEngine: RuleEngine,
    val objectMapper: ObjectMapper,
) {
    fun compileExpression(input: io.github.ayushmaanbhav.productFarm.model.Rule): String {
        // can implement custom expression compilation/parsing here
        return when {
            input.displayExpression.expression != null -> {
                objectMapper.readTree(input.displayExpression.expression) // try parsing for validation
                input.displayExpression.expression
            }
            input.displayExpression.slab != null -> {
                TODO()
                throw NotImplementedError("slab compilation not implemented")
            }
            else -> throw ValidatorException(
                BAD_REQUEST.value(), listOf(createError("unknown display expression type, all values null"))
            )
        }
    }
    
    fun <R : Rule> createRuleDependencyGraph(ruleList: LinkedHashSet<R>): AcyclicDirectedGraph<R> {
        val graphBuilder = DependencyGraphBuilder<R>()
        ruleList.forEach { graphBuilder.visit(it) }
        return graphBuilder.build().getGraph()
    }
    
    fun <R : Rule> executeConstraint(rule: R, input: JsonNode): Boolean {
        val context = QueryContext(UUID.randomUUID().toString(), listOf(rule))
        val ruleInput = QueryInput(linkedMapOf(Pair("value", input)))
        val queries = listOf(Query(rule.ruleType(), QueryType.RULE_TYPE))
        val ruleOutput = productRuleEngine.evaluate(context, queries, ruleInput)
        return ruleOutput.attributes["valid"]?.toString()?.toBooleanStrictOrNull() ?: false
    }
}
