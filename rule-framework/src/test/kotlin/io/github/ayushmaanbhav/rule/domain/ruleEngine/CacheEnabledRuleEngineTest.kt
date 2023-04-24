package io.github.ayushmaanbhav.rule.domain.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryIdentifier
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryOutput
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify

class CacheEnabledRuleEngineTest : FunSpec({
    val cache = mockk<RuleEngineCache>()
    val evaluator = mockk<JsonLogicEvaluator>()
    val ruleEngine = CacheEnabledRuleEngine(cache, evaluator)

    withData(
        nameFn = { input -> "On any $input invoke cache and evaluator once" },
        ts = listOf(
            TestInput(
                queryContext = QueryContext("1", listOf()),
                queries = listOf(),
                queryInput = QueryInput(LinkedHashMap()),
                queryOutput = QueryOutput(LinkedHashMap())
            ),
            TestInput(
                queryContext = QueryContext("2", listOf(mockk())),
                queries = listOf(mockk(), mockk()),
                queryInput = QueryInput(LinkedHashMap(mapOf("a" to "a1"))),
                queryOutput = QueryOutput(LinkedHashMap(mapOf("a" to "a1", "b" to "b1")))
            ),
        )
        // given
    ) { testInput: TestInput ->
        // when
        every { cache.get(any(), any(), any(), any()) } returns ArrayList(testInput.queryContext!!.rules)
        every { evaluator.evaluate(any(), any()) } returns testInput.queryOutput!!.attributes

        val output = ruleEngine.evaluate(testInput.queryContext, testInput.queries!!, testInput.queryInput!!)

        // then
        verify(exactly = 1) {
            cache.get(testInput.queryContext.identifier,
                QueryIdentifier(testInput.queryContext.identifier, testInput.queries), any(), any())
            evaluator.evaluate(ArrayList(testInput.queryContext.rules), testInput.queryInput.attributes)
        }
        output shouldBe testInput.queryOutput
    }

    afterTest { clearAllMocks() }
})
