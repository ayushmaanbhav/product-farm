package io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap

import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.utils.asBigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.setScale

internal interface LenientUnwrapStrategy {
    fun unwrapValueAsBigDecimal(config: StandardLogicOperationConfig, wrappedValue: Any?): List<BigDecimal?> =
        wrappedValue.asList.map { unwrap(config.mathContext, it) }

    private fun unwrap(mc: MathContext, value: Any?): BigDecimal? =
        when (value) {
            is Number -> value.toBigDecimal(mc)
            is String -> value.toBigDecimalOrNull(mc)
            is List<*> -> value.unwrap(mc)
            is Boolean -> value.asBigDecimal().setScale(mc)
            null -> BigDecimal.ZERO.setScale(mc)
            else -> null
        }

    private fun List<*>.unwrap(mc: MathContext) = when (size) {
        0 -> BigDecimal.ZERO.setScale(mc)
        1 -> unwrap(mc, first())
        else -> null
    }

}
