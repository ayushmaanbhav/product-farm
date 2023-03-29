package io.github.ayushmaanbhav.productFarm.util

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.validation.createError
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.RuleEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.algorithm.AcyclicDirectedGraph
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.algorithm.api.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.model.RuleEngineContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.model.RuleEngineInput
import io.github.ayushmaanbhav.rule.domain.ruleExpression.ExpressionParser
import io.github.ayushmaanbhav.rule.domain.ruleExpression.wrapper.SlabConditionWrapper
import io.github.ayushmaanbhav.rule.domain.ruleExpression.wrapper.wrapperObjects.Cases
import io.github.ayushmaanbhav.rule.domain.ruleExpression.wrapper.wrapperObjects.Slab
import org.springframework.http.HttpStatus.BAD_REQUEST
import org.springframework.stereotype.Component

@Component
class RuleUtil(
    val productRuleEngine: RuleEngine,
    val objectMapper: ObjectMapper,
) {
    private val expressionParser = ExpressionParser()
    private val slabConditionWrapper = SlabConditionWrapper()
    
    fun compileExpression(input: io.github.ayushmaanbhav.productFarm.model.Rule): String {
        return when {
            input.displayExpression.expression != null -> {
                expressionParser.parseFromString(input.displayExpression.expression)
            }
            input.displayExpression.slab != null -> {
                val updatedIOSlab = Slab.builder()
                    .cases(input.displayExpression.slab.cases.map { objectMapper.convertValue(it, Cases::class.java) })
                    .commonExpression(input.displayExpression.slab.commonExpression)
                    .defaultReturn(input.displayExpression.slab.defaultReturnObject)
                    .inputs(input.inputAttributes.toCollection(mutableListOf()))
                    .outputs(input.outputAttributes.toCollection(mutableListOf()))
                    .build()
                slabConditionWrapper.createRuleExpressionFormSlabInput(updatedIOSlab)
            }
            else -> throw ValidatorException(
                BAD_REQUEST.value(), listOf(createError("unknown display expression type, all values null"))
            )
        }.buildJsonLogicExpression()
    }
    
    fun createRuleDependencyGraph(ruleList: LinkedHashSet<Rule>): AcyclicDirectedGraph<Rule> {
        val ruleNodeList = ruleList.map { Node.builder<Rule>().component(it).build() }
        val adjacencyList = LinkedHashMap<Node<Rule>, LinkedHashSet<Node<Rule>>>()
        val rootNodesByType = LinkedHashMap<String, LinkedHashSet<Node<Rule>>>()
        val inputPathToRulesMap = LinkedHashMap<String, LinkedHashSet<Node<Rule>>>()
        val outputPathToRuleMap = LinkedHashMap<String, Node<Rule>>()
        val ruleHasParentMap = LinkedHashMap<Node<Rule>, Boolean>()
        
        ruleNodeList.forEach { ruleNode ->
            val inputPaths = ruleNode.component.inputAttributes.map { it.id.path }
            val outputPaths = ruleNode.component.outputAttributes.map { it.id.path }
            outputPaths.forEach { outputPath -> outputPathToRuleMap[outputPath] = ruleNode }
            inputPaths.forEach { inputPath ->
                inputPathToRulesMap.putIfAbsent(inputPath, LinkedHashSet())
                inputPathToRulesMap[inputPath]?.add(ruleNode)
            }
            adjacencyList.putIfAbsent(ruleNode, LinkedHashSet())
            ruleHasParentMap[ruleNode] = false
        }
        outputPathToRuleMap.forEach { (outputPath: String, ruleNode: Node<Rule>) ->
            inputPathToRulesMap[outputPath]?.forEach { parentRuleNode ->
                adjacencyList[parentRuleNode]?.add(ruleNode)
            }
            ruleHasParentMap[ruleNode] = true
        }
        ruleHasParentMap.forEach { (ruleNode: Node<Rule>, hasParent: Boolean) ->
            if (!hasParent) {
                rootNodesByType.putIfAbsent(ruleNode.component.type, LinkedHashSet())
                rootNodesByType[ruleNode.component.type]?.add(ruleNode)
            }
        }
        return AcyclicDirectedGraph.builder<Rule>().adjacencyList(adjacencyList).build()
    }
    
    fun executeConstraint(rule: io.github.ayushmaanbhav.productFarm.model.Rule, input: JsonNode): Boolean {
        val context = RuleEngineContext.builder().rules(listOf(rule)).build()
        val ruleInput = RuleEngineInput.builder().attributes(linkedMapOf(Pair("value", input))).build()
        val ruleOutput = productRuleEngine.evaluateForType(context, rule.type, ruleInput, false)
        return ruleOutput.attributes["valid"]?.toString()?.toBooleanStrictOrNull() ?: false
    }
}
