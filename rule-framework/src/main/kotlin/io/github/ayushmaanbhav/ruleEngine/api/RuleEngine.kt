package io.github.ayushmaanbhav.ruleEngine.api

import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.ruleEngine.model.QueryOutput

interface RuleEngine {
    fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput
    fun evaluate(context: QueryContext, queries: List<Query>): QueryOutput
}
