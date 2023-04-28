package io.github.ayushmaanbhav.ruleEngine.api

import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryOutput

interface RuleEngine {
    fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput
    fun evaluate(context: QueryContext, queries: List<Query>): QueryOutput
}
