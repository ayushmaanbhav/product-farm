package io.github.ayushmaanbhav.jsonLogic.operations.array.occurence

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.api.operation.FunctionalLogicOperation

internal object Some : FunctionalLogicOperation, OccurrenceCheckOperation {
    override fun evaluateLogic(expression: Any?, data: Any?, evaluator: LogicEvaluator): Any? =
        checkOccurrence(expression, data, evaluator)

    override fun check(
        data: OccurrenceCheckInputData,
        evaluator: LogicEvaluator
    ) = with(data) {
        operationData.forEach { dataValue ->
            if (unwrapValueAsBoolean(evaluator.evaluateLogic(mappingOperation, dataValue))) {
                return@with true
            }
        }
        return@with operationDefault
    }
}
