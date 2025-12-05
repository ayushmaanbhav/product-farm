package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap.LenientUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.setScale
import java.math.BigDecimal

internal object Division : StandardLogicOperation, LenientUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?) =
        unwrapValueAsBigDecimal(config, expression).takeIf { it.size >= 2 }?.let {
            val second = it[1]
            val first = it.first()
            if (first != null && second != null && BigDecimal.ZERO.compareTo(second) != 0) {
                first.divide(second, config.mathContext.roundingMode).setScale(config.mathContext)
            } else null
        }
}
