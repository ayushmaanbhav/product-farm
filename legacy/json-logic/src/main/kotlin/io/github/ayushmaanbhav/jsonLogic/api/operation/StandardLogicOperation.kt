package io.github.ayushmaanbhav.jsonLogic.api.operation

import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

interface StandardLogicOperation {
    fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any?
}
