package io.github.ayushmaanbhav.jsonLogic.api

interface LogicEvaluator {
    fun evaluateLogic(expression: Map<String, Any?>, data: Any?): Any?
}
