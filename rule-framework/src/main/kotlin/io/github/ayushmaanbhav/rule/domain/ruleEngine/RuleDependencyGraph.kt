package io.github.ayushmaanbhav.rule.domain.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.AcyclicDirectedGraph
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.api.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.api.SortOrder
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule

class RuleDependencyGraph(
    private val ruleGraph: AcyclicDirectedGraph<Rule>,
    private val startNodesByQuery: LinkedHashMap<Query, LinkedHashSet<Node<Rule>>>
) {
    fun computeExecutableRules(queries: List<Query>): List<Rule> = queries
        .flatMap { startNodesByQuery[it]!! }
        .let { ruleGraph.getTopologicalSort(it, SortOrder.DSC).map { node -> node.value } }
}
