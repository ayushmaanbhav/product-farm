package io.github.ayushmaanbhav.jsonLogic.operations

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal class Log(private val logger: ((Any?) -> Unit)? = null) : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? {
        val loggedValue = expression.asList.firstOrNull()
        logger?.let { log -> log(loggedValue) }
        return loggedValue
    }
}
