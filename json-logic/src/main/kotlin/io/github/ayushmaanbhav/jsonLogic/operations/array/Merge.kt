package io.github.ayushmaanbhav.jsonLogic.operations.array

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal object Merge : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any = expression.asList.mergeOrAdd()

    private fun List<Any?>.mergeOrAdd(): List<Any?> = flatMap {
        when (it) {
            is List<*> -> it
            else -> listOf(it)
        }
    }
}
