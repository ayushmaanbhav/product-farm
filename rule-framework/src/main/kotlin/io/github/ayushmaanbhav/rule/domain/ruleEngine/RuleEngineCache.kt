package io.github.ayushmaanbhav.rule.domain.ruleEngine

import com.google.common.cache.Cache
import com.google.common.cache.CacheBuilder
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.CachePolicy
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryIdentifier
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.RuleEngineConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule

class RuleEngineCache(config: RuleEngineConfig) {
    private val cachePolicy = config.cachePolicy
    private val ruleDgByContextIdentifier: Cache<String, RuleDependencyGraph> = CacheBuilder.newBuilder()
        .let { if (config.maxRuleDgCacheSize > 0) it.maximumSize(config.maxRuleDgCacheSize) else it }.build()
    private val rulesByQueryIdentifier: Cache<QueryIdentifier, List<Rule>> = CacheBuilder.newBuilder()
        .let { if (config.maxQueryCacheSize > 0) it.maximumSize(config.maxQueryCacheSize) else it }.build()

    fun get(
        contextIdentifier: String, queryIdentifier: QueryIdentifier,
        createRdg: () -> RuleDependencyGraph, createExecutableRules: (RuleDependencyGraph) -> List<Rule>
    ): List<Rule> {
        return when (cachePolicy) {
            CachePolicy.DISABLED -> createExecutableRules(createRdg())
            else -> checkAndGetExecutableRulesFromCache(queryIdentifier) {
                createExecutableRules(checkAndGetRuleDgFromCache(contextIdentifier, createRdg))
            }
        }
    }

    private fun checkAndGetExecutableRulesFromCache(queryIdentifier: QueryIdentifier, default: () -> List<Rule>): List<Rule> =
        when (val cachedExecutableRules = rulesByQueryIdentifier.getIfPresent(queryIdentifier)) {
            null -> {
                val newExecutableRules = default()
                rulesByQueryIdentifier.put(queryIdentifier, newExecutableRules)
                newExecutableRules
            }
            else -> cachedExecutableRules
        }

    private fun checkAndGetRuleDgFromCache(contextIdentifier: String, default: () -> RuleDependencyGraph): RuleDependencyGraph =
        when (val cachedRuleDg = ruleDgByContextIdentifier.getIfPresent(contextIdentifier)) {
            null -> {
                val newRuleDg = default()
                ruleDgByContextIdentifier.put(contextIdentifier, newRuleDg)
                newRuleDg
            }
            else -> cachedRuleDg
        }
}
