package io.github.ayushmaanbhav.jsonLogic.operations.logic

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap.TruthyUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal object Or : StandardLogicOperation, TruthyUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?) = with(expression.asList) {
        if (all { it is Boolean }) {
            firstOrNull { unwrapValueAsBoolean(it) } != null
        } else {
            (firstOrNull { unwrapValueAsBoolean(it) } ?: last())
        }
    }
}
