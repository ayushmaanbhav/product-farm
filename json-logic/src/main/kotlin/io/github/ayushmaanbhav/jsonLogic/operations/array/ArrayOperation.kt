package io.github.ayushmaanbhav.jsonLogic.operations.array

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.api.unwrap.EvaluatingUnwrapper
import io.github.ayushmaanbhav.jsonLogic.utils.getMappingOperationOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull
import kotlin.collections.Map

internal interface ArrayOperation : EvaluatingUnwrapper {
    fun createOperationInput(
        expressionValues: List<Any?>,
        operationData: Any?,
        evaluator: LogicEvaluator
    ): ArrayOperationInputData {
        val evaluatedOperationData = unwrapDataByEvaluation(expressionValues, operationData, evaluator)
        val mappingOperation = expressionValues.getMappingOperationOrNull()
        val operationDefault = getOperationDefault(mappingOperation, expressionValues)

        return ArrayOperationInputData(evaluatedOperationData, mappingOperation, operationDefault)
    }

    fun getOperationDefault(mappingOperation: Map<String, Any>?, expressionValues: List<Any?>) =
        if (mappingOperation == null) {
            expressionValues.secondOrNull()
        } else null
}
