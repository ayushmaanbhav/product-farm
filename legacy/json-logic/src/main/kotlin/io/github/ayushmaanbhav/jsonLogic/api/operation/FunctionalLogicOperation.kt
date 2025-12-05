package io.github.ayushmaanbhav.jsonLogic.api.operation

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator

interface FunctionalLogicOperation {
    fun evaluateLogic(expression: Any?, data: Any?, evaluator: LogicEvaluator): Any?
}
