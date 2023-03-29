package io.github.ayushmaanbhav.jsonLogic.operations.string

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

internal object Cat : StandardLogicOperation, StringUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? = unwrapValueAsString(expression).joinToString("")
}
