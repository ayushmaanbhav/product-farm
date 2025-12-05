package io.github.ayushmaanbhav.jsonLogic.operations.data

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.longOrZero
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull

internal object MissingSome : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any {
        val min = (expression as? List<Any?>?)?.firstOrNull()?.toString()?.longOrZero ?: 0
        val keys = ((expression as? List<Any?>?)?.secondOrNull() as? List<Any?>).orEmpty()
        val missing = Missing.evaluateLogic(config, keys, data)
        return missing.takeIf { keys.size - missing.size < min }.orEmpty()
    }
}
