package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap.StrictUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.setScale
import java.math.BigDecimal

internal object Multiplication : StandardLogicOperation, BigDecimalTypeSensitiveOperation, StrictUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? {
        val values = expression.asList
        return when (values.size) {
            0 -> null
            1 -> values.first()
            else -> bigDecimalResultOrNull(config.mathContext, unwrapValue(config, expression)) {
                it.reduce { sum: BigDecimal, value: BigDecimal -> sum.multiply(value, config.mathContext.mc()).setScale(config.mathContext) }
            }
        }
    }
}
