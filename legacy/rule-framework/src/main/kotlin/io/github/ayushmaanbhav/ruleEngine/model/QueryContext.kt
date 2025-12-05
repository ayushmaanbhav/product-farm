package io.github.ayushmaanbhav.ruleEngine.model

import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule

data class QueryContext(val identifier: String, val rules: Collection<Rule>)
