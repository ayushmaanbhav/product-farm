package io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.strict

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

internal object NotStrictEquals : StandardLogicOperation, StrictEqualsOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Boolean =
        !compare(expression) { first, second -> first == second }
}

