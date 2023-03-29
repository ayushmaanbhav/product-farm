package io.github.ayushmaanbhav.jsonLogic.operations.data

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal object Missing : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): List<Any?> {
        return expression.asList.mapNotNull {
            it.takeIf { Var.evaluateLogic(config, it, data).isNullOrEmptyString() }
        }
    }

    private fun Any?.isNullOrEmptyString() = this == null || (this is String && this.isEmpty())
}
