package io.github.ayushmaanbhav.jsonLogic.stdlib.array

import io.github.ayushmaanbhav.jsonLogic.api.LogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.api.operation.FunctionalLogicOperation
import io.github.ayushmaanbhav.jsonLogic.api.unwrap.EvaluatingUnwrapper
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.getMappingOperationOrNull

object Find : FunctionalLogicOperation, EvaluatingUnwrapper {
    override fun evaluateLogic(expression: Any?, data: Any?, evaluator: LogicEvaluator): Any? {
        return expression.asList.let { expressionValues ->
            val inputData = unwrapDataByEvaluation(expressionValues, data, evaluator)
            val predicateOperation = expressionValues.getMappingOperationOrNull()

            predicateOperation?.let {
                inputData?.find { evaluator.evaluateLogic(predicateOperation, it) == true }
            }
        }
    }
}
