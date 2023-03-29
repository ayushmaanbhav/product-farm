package io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.api

interface Graph<T> {
    fun getTopologicalSort(startNodes: Collection<Node<T>>, sortOrder: SortOrder): List<Node<T>>
}
