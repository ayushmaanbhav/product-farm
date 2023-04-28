package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.api.RuleEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryOutput
import org.springframework.stereotype.Component

@Component
class RuleEngineImpl(private val ruleEngine: io.github.ayushmaanbhav.ruleEngine.CacheEnabledRuleEngine) : RuleEngine {

    override fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput {
        return ruleEngine.evaluate(context, queries, input)
    }

    override fun evaluate(context: QueryContext, queries: List<Query>): QueryOutput {
        return ruleEngine.evaluate(context, queries, QueryInput(LinkedHashMap()))
    }
}
