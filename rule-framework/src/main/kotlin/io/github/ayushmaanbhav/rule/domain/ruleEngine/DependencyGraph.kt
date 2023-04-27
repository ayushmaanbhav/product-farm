package io.github.ayushmaanbhav.rule.domain.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.AcyclicDirectedGraph
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.SortOrder
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule

class DependencyGraph<R : Rule>(
    private val ruleGraph: AcyclicDirectedGraph<R>,
    private val startNodesByQuery: LinkedHashMap<Query, LinkedHashSet<Node<R>>>
) {
    fun getGraph(): AcyclicDirectedGraph<R> = ruleGraph
    fun computeExecutableRules(queries: List<Query>): List<R> = queries
        .flatMap { startNodesByQuery[it]!! }
        .let { ruleGraph.getTopologicalSort(LinkedHashSet(it), SortOrder.DSC).map { node -> node.value } }
}
