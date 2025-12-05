package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.config.Config
import io.github.ayushmaanbhav.ruleEngine.model.CachePolicy
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryIdentifier
import io.github.ayushmaanbhav.ruleEngine.model.QueryType
import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify

class RuleEngineCacheTest : StringSpec() {
    private val mockConfig = mockk<Config>()
    private val mockDependencyGraph = mockk<DependencyGraph<Rule>>()
    private val mockExecutableRules = listOf(mockk<Rule>(), mockk<Rule>())

    init {
        val queries = listOf(Query("attribute-2", QueryType.ATTRIBUTE_PATH))
        val queryIdentifier = QueryIdentifier("test_context", queries)

        "should call createRdg and createExecutableRules when cachePolicy is CachePolicy.DISABLED" {
            every { mockConfig.cachePolicy } returns CachePolicy.DISABLED
            every { mockConfig.maxRuleDgCacheSize } returns 10
            every { mockConfig.maxQueryCacheSize } returns 10

            val ruleEngineCache = RuleEngineCache(mockConfig)

            val createRdg = mockk<() -> DependencyGraph<Rule>>()
            val createExecutableRules = mockk<(DependencyGraph<Rule>) -> List<Rule>>()
            every { createRdg() } returns mockDependencyGraph
            every { createExecutableRules(mockDependencyGraph) } returns mockExecutableRules

            val result1 = ruleEngineCache.get("contextId", queryIdentifier, createRdg, createExecutableRules)
            val result2 = ruleEngineCache.get("contextId", queryIdentifier, createRdg, createExecutableRules)

            result1 shouldBe mockExecutableRules
            result2 shouldBe mockExecutableRules

            verify(exactly = 2) { createRdg() }
            verify(exactly = 2) { createExecutableRules(mockDependencyGraph) }
        }

        "should return cached executable rules when present in the cache" {
            every { mockConfig.cachePolicy } returns CachePolicy.LRU_CACHE
            every { mockConfig.maxRuleDgCacheSize } returns 10
            every { mockConfig.maxQueryCacheSize } returns 10

            val ruleEngineCache = RuleEngineCache(mockConfig)

            val createRdg = mockk<() -> DependencyGraph<Rule>>()
            val createExecutableRules = mockk<(DependencyGraph<Rule>) -> List<Rule>>()
            every { createRdg() } returns mockDependencyGraph
            every { createExecutableRules(mockDependencyGraph) } returns mockExecutableRules

            val result1 = ruleEngineCache.get("contextId", queryIdentifier, createRdg, createExecutableRules)
            val result2 = ruleEngineCache.get("contextId", queryIdentifier, createRdg, createExecutableRules)

            result1 shouldBe mockExecutableRules
            result2 shouldBe mockExecutableRules

            verify(exactly = 1) { createRdg() }
            verify(exactly = 1) { createExecutableRules(mockDependencyGraph) }
        }
    }
}
