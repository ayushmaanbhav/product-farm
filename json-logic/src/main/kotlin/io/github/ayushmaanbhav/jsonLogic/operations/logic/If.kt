package io.github.ayushmaanbhav.jsonLogic.operations.logic

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap.TruthyUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.thirdOrNull

@Suppress("MagicNumber")
internal object If : StandardLogicOperation, TruthyUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? = expression.asList.recursiveIf()

    private tailrec fun List<Any?>.recursiveIf(): Any? = when (size) {
        0 -> null
        1 -> firstOrNull()
        2 -> if (unwrapValueAsBoolean(firstOrNull())) secondOrNull() else null
        3 -> if (unwrapValueAsBoolean(firstOrNull())) secondOrNull() else thirdOrNull()
        else -> if (unwrapValueAsBoolean(firstOrNull())) secondOrNull() else subList(2, size).recursiveIf()
    }
}
