package io.github.ayushmaanbhav.jsonLogic.stdlib.string

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

object Capitalize : StandardLogicOperation, StringUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        unwrapValueAsString(expression)?.replaceFirstChar { it.uppercase() }
}
