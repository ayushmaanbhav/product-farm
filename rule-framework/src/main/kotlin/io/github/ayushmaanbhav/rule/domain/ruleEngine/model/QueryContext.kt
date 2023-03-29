package io.github.ayushmaanbhav.rule.domain.ruleEngine.model

import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule

data class QueryContext(val identifier: String, val rules: Collection<Rule>)
