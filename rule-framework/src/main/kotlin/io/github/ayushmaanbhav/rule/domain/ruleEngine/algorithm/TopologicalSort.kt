package io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.SortOrder
import java.util.*

internal class TopologicalSort {
    companion object {
        fun <T> sort(
            adjacencyList: LinkedHashMap<Node<T>, LinkedHashSet<Node<T>>>, startNodes: Collection<Node<T>>, sortOrder: SortOrder
        ): List<Node<T>> {
            val nodesToVisit = LinkedList<Node<T>>()
            val topOrderedNodes = LinkedList<Node<T>>()
            val inDegrees = LinkedHashMap<Node<T>, Int>()
            for (node in startNodes) {
                inDegrees.putIfAbsent(node, 0)
                for (nodeTo in adjacencyList[node]!!) {
                    inDegrees.putIfAbsent(nodeTo, 0)
                    inDegrees.computeIfPresent(nodeTo) { _: Node<T>, inDegree: Int -> inDegree + 1 }
                }
            }
            for (node in inDegrees.keys) {
                if (inDegrees[node] == 0) {
                    nodesToVisit.add(node)
                }
            }
            while (!nodesToVisit.isEmpty()) {
                val visitingNode = nodesToVisit.removeFirst()
                if (sortOrder == SortOrder.ASC) topOrderedNodes.addLast(visitingNode) else topOrderedNodes.addFirst(visitingNode)
                // Iterate through all neighbouring nodes and decrease their in-degree by 1
                for (nodeTo in adjacencyList[visitingNode]!!) {
                    inDegrees.computeIfPresent(nodeTo) { _: Node<T>, inDegree: Int -> inDegree - 1 }
                    // If in-degree becomes zero, add it to queue
                    if (inDegrees[nodeTo] == 0) {
                        nodesToVisit.addLast(nodeTo)
                    }
                }
            }
            return topOrderedNodes
        }
    }
}
