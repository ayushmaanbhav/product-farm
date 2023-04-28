package io.github.ayushmaanbhav.ruleEngine.algorithm

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model.SortOrder
import java.util.*
import kotlin.collections.LinkedHashSet

internal class TopologicalSort {
    companion object {
        fun <T> sort(
            adjacencyList: LinkedHashMap<Node<T>, LinkedHashSet<Node<T>>>, startNodes: Set<Node<T>>, sortOrder: SortOrder
        ): List<Node<T>> {
            val inDegrees = LinkedHashMap<Node<T>, Int>()
            val initialNodesToVisit = LinkedList(startNodes)
            while (initialNodesToVisit.isNotEmpty()) {
                val node = initialNodesToVisit.removeFirst()
                inDegrees.putIfAbsent(node, 0)
                for (nodeTo in adjacencyList[node]!!) {
                    inDegrees.putIfAbsent(nodeTo, 0)
                    inDegrees.computeIfPresent(nodeTo) { _: Node<T>, inDegree: Int -> inDegree + 1 }
                    if (startNodes.contains(nodeTo).not()) initialNodesToVisit.addLast(nodeTo)
                }
            }
            val nodesToVisit = LinkedList<Node<T>>()
            val topOrderedNodes = LinkedList<Node<T>>()
            for (node in inDegrees.keys) {
                if (inDegrees[node] == 0) {
                    nodesToVisit.add(node)
                }
            }
            while (nodesToVisit.isNotEmpty()) {
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
