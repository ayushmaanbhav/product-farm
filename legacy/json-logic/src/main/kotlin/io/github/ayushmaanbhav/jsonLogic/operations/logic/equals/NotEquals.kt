package io.github.ayushmaanbhav.jsonLogic.operations.logic.equals

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

internal object NotEquals : StandardLogicOperation, EqualsOperation {
    override fun evaluateLogic(
        config: StandardLogicOperationConfig, expression: Any?, data: Any?
    ): Boolean = !compare(expression) { first, second -> first == second }
}
