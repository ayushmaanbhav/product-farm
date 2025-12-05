package io.github.ayushmaanbhav.ruleEngine.algorithm

import io.github.ayushmaanbhav.ruleEngine.DependencyGraph
import io.github.ayushmaanbhav.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.ruleEngine.exception.MultilpleRulesOutputAttributeException
import io.github.ayushmaanbhav.ruleEngine.exception.Rule_SameOutputAsInputAttributeException
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryType
import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule

class DependencyGraphBuilder<R : Rule> {
    private val ruleNodes: MutableSet<Node<R>> = HashSet()

    fun visit(rule: R) {
        ruleNodes.add(Node(rule))
    }

    fun build(): DependencyGraph<R> {
        val adjacencyList = buildAdjacencyList()
        val dependencyGraph = AcyclicDirectedGraph.AcyclicDirectedGraphBuilder(adjacencyList).build()
        val startNodesByQuery = buildQueryIndexes()
        return DependencyGraph(dependencyGraph, startNodesByQuery)
    }

    private fun buildAdjacencyList(): LinkedHashMap<Node<R>, LinkedHashSet<Node<R>>> {
        val adjacencyList = LinkedHashMap<Node<R>, LinkedHashSet<Node<R>>>()
        val inputPathToRulesMap: MutableMap<String, LinkedHashSet<Node<R>>> = LinkedHashMap()
        val outputPathToRuleMap: MutableMap<String, Node<R>> = LinkedHashMap()
        ruleNodes.forEach { ruleNode: Node<R> ->
            val inputPaths = ruleNode.value.getInputAttributePaths()
            val outputPaths = ruleNode.value.getOutputAttributePaths()
            val commonElements = inputPaths.intersect(outputPaths)
            if (commonElements.isNotEmpty()) {
                throw Rule_SameOutputAsInputAttributeException(
                    "Rule cannot produce same attribute to output as its input ${ruleNode.value.getId()}"
                )
            }
            outputPaths.forEach { outputPath: String ->
                val existingNode = outputPathToRuleMap[outputPath]
                if (existingNode != null) {
                    throw MultilpleRulesOutputAttributeException(
                        "Attribute $outputPath has multiple producers: ${existingNode.value.getId()}, ${ruleNode.value.getId()}"
                    )
                }
                outputPathToRuleMap[outputPath] = ruleNode
            }
            inputPaths.forEach { inputPath: String ->
                inputPathToRulesMap.putIfAbsent(inputPath, LinkedHashSet())
                inputPathToRulesMap[inputPath]!!.add(ruleNode)
            }
            adjacencyList.putIfAbsent(ruleNode, LinkedHashSet())
        }
        outputPathToRuleMap.forEach { (outputPath: String, ruleNode: Node<R>) ->
            inputPathToRulesMap[outputPath]?.let {
                it.forEach { parentRuleNode: Node<R> -> adjacencyList[parentRuleNode]!!.add(ruleNode) }
            }
        }
        return adjacencyList
    }

    private fun buildQueryIndexes(): LinkedHashMap<Query, LinkedHashSet<Node<R>>> {
        val startNodesByQuery = LinkedHashMap<Query, LinkedHashSet<Node<R>>>()
        ruleNodes.forEach { ruleNode: Node<R> ->
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
