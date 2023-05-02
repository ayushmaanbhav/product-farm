package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap.LenientUnwrapStrategy
import java.math.BigDecimal

internal object Subtraction : StandardLogicOperation, LenientUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?) =
        with(unwrapValueAsBigDecimal(config, expression)) {
            when (size) {
                0 -> null
                1 -> first()?.unaryMinus()
                else -> minusOrNull(first(), get(1))
            }
        }

    private fun minusOrNull(first: BigDecimal?, second: BigDecimal?) = if (first != null && second != null) {
        first.minus(second)
    } else null
}
