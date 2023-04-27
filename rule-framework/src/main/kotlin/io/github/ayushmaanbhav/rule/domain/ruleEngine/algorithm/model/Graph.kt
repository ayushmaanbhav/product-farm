package io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.model

interface Graph<T> {
    fun getTopologicalSort(startNodes: Set<Node<T>>, sortOrder: SortOrder): List<Node<T>>
}
