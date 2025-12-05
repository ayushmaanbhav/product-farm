package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.ruleEngine.model.QueryOutput
import io.github.ayushmaanbhav.ruleEngine.model.QueryType
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk

class RuleEngineImplTest : BehaviorSpec({

    val cacheEnabledRuleEngineMock = mockk<CacheEnabledRuleEngine>()
    val ruleEngine = RuleEngineImpl(cacheEnabledRuleEngineMock)
    val context = QueryContext("id", emptyList())
    val queries = listOf(Query("query1", QueryType.RULE_TYPE), Query("query2", QueryType.RULE_TYPE))
    val input = QueryInput(LinkedHashMap())

    Given("a list of queries") {
        val expectedOutput = QueryOutput(LinkedHashMap())
        every { cacheEnabledRuleEngineMock.evaluate(context, queries, input) } returns expectedOutput

        When("evaluate is called with context, queries, and input") {
            val output = ruleEngine.evaluate(context, queries, input)

            Then("the output should match the expected output") {
                output shouldBe expectedOutput
            }
        }
    }

    Given("a list of queries without input") {
        val expectedOutput = QueryOutput(LinkedHashMap())
        every { cacheEnabledRuleEngineMock.evaluate(context, queries, QueryInput(LinkedHashMap())) } returns expectedOutput

        When("evaluate is called with context and queries") {
            val output = ruleEngine.evaluate(context, queries)

            Then("the output should match the expected output") {
                output shouldBe expectedOutput
            }
        }
    }
})
