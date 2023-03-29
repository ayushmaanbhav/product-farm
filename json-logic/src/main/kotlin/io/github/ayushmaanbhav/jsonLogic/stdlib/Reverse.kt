package io.github.ayushmaanbhav.jsonLogic.stdlib

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

object Reverse : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? {
        return when (expression) {
            is String -> expression.reversed()
            is List<*> -> expression.reversed()
            else -> null

        }
    }
}
