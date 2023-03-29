package io.github.ayushmaanbhav.jsonLogic.operations

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull

internal object In : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Boolean? {
        val first = expression.asList.firstOrNull()
        return when (val second = expression.asList.secondOrNull()) {
            is String -> second.contains(first.toString())
            is List<*> -> second.contains(first)
            else -> false
        }
    }
}
