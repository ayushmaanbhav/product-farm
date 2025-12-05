package io.github.ayushmaanbhav.jsonLogic.api.unwrap

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.utils.isExpression

interface EvaluatingUnwrapper {
    fun unwrapDataByEvaluation(expression: List<Any?>, data: Any?, evaluator: LogicEvaluator) =
        (expression.firstOrNull().unwrapOperationData(data, evaluator) as? List<Any?>)

    @Suppress("UNCHECKED_CAST")
    private fun Any?.unwrapOperationData(data: Any?, evaluator: LogicEvaluator): Any? = when {
        this is List<*> -> map { it.unwrapOperationData(data, evaluator) }
        isExpression() -> evaluator.evaluateLogic(this as Map<String, Any?>, data)
        else -> this
    }
}
