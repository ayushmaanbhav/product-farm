package io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.api.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule
import io.github.ayushmaanbhav.rule.domain.ruleEngine.RuleDependencyGraph
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryType

class RuleDependencyGraphBuilder {
    private val ruleNodes: MutableSet<Node<Rule>> = HashSet()

    fun visit(rule: Rule) {
        ruleNodes.add(Node(rule))
    }

    fun build(): RuleDependencyGraph {
        val adjacencyList = buildAdjacencyList()
        val dependencyGraph = AcyclicDirectedGraph.AcyclicDirectedGraphBuilder(adjacencyList).build()
        val startNodesByQuery = buildQueryIndexes()
        return RuleDependencyGraph(dependencyGraph, startNodesByQuery)
    }

    private fun buildAdjacencyList(): LinkedHashMap<Node<Rule>, LinkedHashSet<Node<Rule>>> {
        val adjacencyList = LinkedHashMap<Node<Rule>, LinkedHashSet<Node<Rule>>>()
        val inputPathToRulesMap: MutableMap<String, MutableSet<Node<Rule>>> = LinkedHashMap()
        val outputPathToRuleMap: MutableMap<String, Node<Rule>> = LinkedHashMap()
        ruleNodes.forEach { ruleNode: Node<Rule> ->
            val inputPaths = ruleNode.value.getInputAttributePaths()
            val outputPaths = ruleNode.value.getOutputAttributePaths()
            outputPaths.forEach { outputPath: String -> outputPathToRuleMap[outputPath] = ruleNode }
            inputPaths.forEach { inputPath: String ->
                inputPathToRulesMap.putIfAbsent(inputPath, HashSet())
                inputPathToRulesMap[inputPath]!!.add(ruleNode)
            }
            adjacencyList.putIfAbsent(ruleNode, LinkedHashSet())
        }
        outputPathToRuleMap.forEach { (outputPath: String, ruleNode: Node<Rule>) ->
            inputPathToRulesMap[outputPath]?.let {
                it.forEach { parentRuleNode: Node<Rule> -> adjacencyList[parentRuleNode]!!.add(ruleNode) }
            }
        }
        return adjacencyList
    }

    private fun buildQueryIndexes(): LinkedHashMap<Query, LinkedHashSet<Node<Rule>>> {
        val startNodesByQuery = LinkedHashMap<Query, LinkedHashSet<Node<Rule>>>()
        ruleNodes.forEach { ruleNode: Node<Rule> ->
            val queries = mutableListOf<Query>()
            ruleNode.value.ruleType().let { queries.add(Query(it, QueryType.RULE_TYPE)) }
            ruleNode.value.getOutputAttributePaths().forEach { queries.add(Query(it, QueryType.ATTRIBUTE_PATH)) }
            ruleNode.value.getTags().forEach { queries.add(Query(it, QueryType.ATTRIBUTE_TAG)) }
            queries.forEach {
                startNodesByQuery.putIfAbsent(it, LinkedHashSet())
                startNodesByQuery[it]!!.add(ruleNode)
            }
        }
        return startNodesByQuery
    }
}
