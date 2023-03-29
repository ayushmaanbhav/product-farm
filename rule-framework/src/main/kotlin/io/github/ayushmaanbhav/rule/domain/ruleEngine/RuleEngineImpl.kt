package io.github.ayushmaanbhav.rule.domain.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.api.RuleEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.RuleEngineConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryOutput

class RuleEngineImpl(ruleEngineConfig: RuleEngineConfig) : RuleEngine {
    private val ruleEngine = CacheEnabledRuleEngine(ruleEngineConfig)

    override fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput {
        return ruleEngine.evaluate(context, queries, input)
    }

    override fun evaluate(context: QueryContext, queries: List<Query>): QueryOutput {
        return ruleEngine.evaluate(context, queries, QueryInput(LinkedHashMap()))
    }
}
