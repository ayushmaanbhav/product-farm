package io.github.ayushmaanbhav.jsonLogic.stdlib.format

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import java.lang.String.format
import java.math.BigDecimal

object DecimalFormat : StandardLogicOperation, DecimalFormatter {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? {
        return formatDecimal(expression, data) { formatSequence: String, arg: BigDecimal ->
            format(formatSequence, arg)
        }
    }
}
