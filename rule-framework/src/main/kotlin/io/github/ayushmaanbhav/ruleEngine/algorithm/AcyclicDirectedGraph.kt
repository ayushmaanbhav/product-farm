package io.github.ayushmaanbhav.ruleEngine.algorithm

import io.github.ayushmaanbhav.ruleEngine.algorithm.model.Graph
import io.github.ayushmaanbhav.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.ruleEngine.algorithm.model.SortOrder
import io.github.ayushmaanbhav.ruleEngine.exception.GraphContainsCycleException
import java.util.*

class AcyclicDirectedGraph<T>(
    private val adjacencyList: LinkedHashMap<Node<T>, LinkedHashSet<Node<T>>>
) : Graph<T> {

    override fun getTopologicalSort(startNodes: Set<Node<T>>, sortOrder: SortOrder): List<Node<T>> =
        TopologicalSort.sort(adjacencyList, startNodes, sortOrder)

    class AcyclicDirectedGraphBuilder<T>(private val adjacencyList: LinkedHashMap<Node<T>, LinkedHashSet<Node<T>>>) {
        fun build(): AcyclicDirectedGraph<T> {
            if (hasCycle()) {
                throw GraphContainsCycleException("The given adjacency list contains a cycle")
            }
            return AcyclicDirectedGraph(adjacencyList)
        }

        private fun hasCycle(): Boolean =
            TopologicalSort.sort(adjacencyList, adjacencyList.keys, SortOrder.ASC).size != adjacencyList.keys.size
    }
}
