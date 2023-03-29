package io.github.ayushmaanbhav.jsonLogic.operations.logic

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap.TruthyUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal object Negation : StandardLogicOperation, TruthyUnwrapStrategy {
    override fun evaluateLogic(
        config: StandardLogicOperationConfig, expression: Any?, data: Any?
    ): Boolean = !unwrapValueAsBoolean(expression.asList.firstOrNull())
}
