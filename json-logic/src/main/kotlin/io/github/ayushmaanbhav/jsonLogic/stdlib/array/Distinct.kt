package io.github.ayushmaanbhav.jsonLogic.stdlib.array

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig

object Distinct : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? = (expression as? List<*>)?.distinct()
}
