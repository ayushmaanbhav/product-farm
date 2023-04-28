package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.config.Config
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.ruleEngine.model.QueryOutput

data class TestInput(
    val config: Config? = null,
    val queryContext: QueryContext? = null,
    val queries: List<Query>? = null,
    val queryInput: QueryInput? = null,
    val queryOutput: QueryOutput? = null,
)
