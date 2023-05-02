package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap.StrictUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.setScale
import java.math.BigDecimal

internal object Addition : StandardLogicOperation, BigDecimalTypeSensitiveOperation, StrictUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        bigDecimalResultOrNull(config.mathContext, unwrapValue(config, expression)) {
            it.reduceOrNull(BigDecimal::plus) ?: BigDecimal.ZERO.setScale(config.mathContext)
        }
}
