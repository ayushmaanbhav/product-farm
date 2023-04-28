package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.model.*
import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import org.apache.logging.log4j.kotlin.Logging

class CacheEnabledRuleEngineTest : DescribeSpec(), Logging {
    private val cache = mockk<RuleEngineCache>()
    private val evaluator = mockk<JsonLogicEvaluator>()
    private val cacheEnabledRuleEngine = CacheEnabledRuleEngine(cache, evaluator)
    private val rdgBuilder = slot<() -> DependencyGraph<Rule>>()
    private val ruleBuilder = slot<(DependencyGraph<Rule>) -> List<Rule>>()

    init {
        describe("evaluate") {
            val rule1 = RuleImpl("rule1", "type-1", setOf("attribute-0"), setOf("attribute-1"), setOf("tag-1", "tag-2"))
            val rule2 = RuleImpl("rule2", "type-1", setOf("attribute-1"), setOf("attribute-2"), setOf("tag-1", "tag-2"))
            val rules = listOf(rule1, rule2)
            val queryContext = QueryContext("test_context", rules)
            val queries = listOf(Query("attribute-2", QueryType.ATTRIBUTE_PATH))
            val queryIdentifier = QueryIdentifier("test_context", queries)
            val queryInput = QueryInput(LinkedHashMap(mapOf("attribute-0" to "value")))

            it("should return expected output when cache is empty") {
                val expectedOutput = QueryOutput(LinkedHashMap(mapOf("attribute-2" to true)))
                every { cache.get(queryContext.identifier, queryIdentifier, capture(rdgBuilder), capture(ruleBuilder)) } answers {
                    val rdg = rdgBuilder.captured.invoke()
                    ruleBuilder.captured.invoke(rdg)
                }
                every { evaluator.evaluate(rules, queryInput.attributes) } returns LinkedHashMap(mapOf("attribute-2" to true))

                val actualOutput = cacheEnabledRuleEngine.evaluate(queryContext, queries, queryInput)

                actualOutput shouldBe expectedOutput
            }

            it("should return expected output when cache is not empty") {
                val expectedOutput = QueryOutput(LinkedHashMap(mapOf("attribute-2" to false)))
                every { cache.get(queryContext.identifier, queryIdentifier, any(), any()) } returns rules
                every { evaluator.evaluate(rules, queryInput.attributes) } returns LinkedHashMap(mapOf("attribute-2" to false))

                val actualOutput = cacheEnabledRuleEngine.evaluate(queryContext, queries, queryInput)

                actualOutput shouldBe expectedOutput
            }
        }
    }
}
