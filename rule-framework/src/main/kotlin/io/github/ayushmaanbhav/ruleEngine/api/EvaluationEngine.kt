package io.github.ayushmaanbhav.ruleEngine.api

import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule
import java.util.LinkedHashMap

interface EvaluationEngine {
    fun evaluate(rules: List<Rule>, attributes: LinkedHashMap<String, Any>): LinkedHashMap<String, Any>
}
