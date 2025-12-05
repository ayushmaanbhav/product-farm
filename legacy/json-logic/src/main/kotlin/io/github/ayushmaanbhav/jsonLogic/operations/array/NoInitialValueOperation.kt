package io.github.ayushmaanbhav.jsonLogic.operations.array

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal interface NoInitialValueOperation : ArrayOperation {
    fun invokeArrayOperation(
        expression: Any?,
        operationData: Any?,
        evaluator: LogicEvaluator,
        arrayOperation: (ArrayOperationInputData, LogicEvaluator) -> Any?
    ) = expression.asList.let { expressionValues ->
        val input = createOperationInput(expressionValues, operationData, evaluator)
        arrayOperation(input, evaluator)
    }
}
