package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.algorithm.AcyclicDirectedGraph
import io.github.ayushmaanbhav.ruleEngine.algorithm.model.Node
import io.github.ayushmaanbhav.ruleEngine.algorithm.model.SortOrder
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlin.collections.LinkedHashSet

class DependencyGraphTest : FunSpec({
    val ruleGraph = mockk<AcyclicDirectedGraph<Rule>>()
    val startNodes = mockk<LinkedHashMap<Query, LinkedHashSet<Node<Rule>>>>()
    val graph = DependencyGraph<Rule>(ruleGraph, startNodes)

    withData(
        nameFn = { input -> "On any $input invoke cache and evaluator once" },
        ts = listOf(
            TestInput(
                queryContext = QueryContext("1", listOf()),
                queries = listOf(),
            ),
            TestInput(
                queryContext = QueryContext("2", listOf(mockk())),
                queries = listOf(mockk(), mockk()),
            ),
        )
        // given
    ) { testInput: TestInput ->
        // when
        every { ruleGraph.getTopologicalSort(any(), any()) } returns testInput.queryContext!!.rules.map { Node(it) }
        every { startNodes[any()] } returns LinkedHashSet(setOf(mockk()))

        val outputGraph = graph.getGraph()
        val rules = graph.computeExecutableRules(testInput.queries!!)

        // then
        verify(exactly = 1) {
            ruleGraph.getTopologicalSort(any(), SortOrder.DSC)
        }
        outputGraph shouldBe ruleGraph
        rules shouldBe testInput.queryContext.rules
    }

    afterTest { clearAllMocks() }
})
